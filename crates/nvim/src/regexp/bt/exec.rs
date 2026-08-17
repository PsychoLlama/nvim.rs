//! The engine's entry points: the two `regexec` shapes the engine table
//! names, and the loop over start columns between them.
//!
//! `bt_regexec_both` is where the shortcuts live — a `regmust` string the
//! line has to contain somewhere, an anchored pattern that only tries column
//! 0, and a known first character to skip to — and `regtry` is one attempt at
//! one column.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::matcher::regmatch;
use super::state::regstack;
use crate::garray::{ga_clear, ga_grow, ga_init, ga_set_growsize};
use crate::main::{e_null, got_int, re_extmatch_out};
use crate::mbyte::{mb_tolower, utf_fold, utf_ptr2char, utfc_ptr2len};
use crate::memory::xfree;
use crate::message::iemsg;
use crate::os::libc::gettext;
use crate::profile::profile_passed_limit;
use crate::regexp::{
    BACKPOS_INITIAL, NSUBEXP, NUL, REX_SET, RF_ICASE, RF_ICOMBINE, RF_NOICASE, Rex, backpos,
    backpos_T, bt_regprog_T, cleanup_subexpr, cleanup_zsubexpr, cstrchr, cstrncmp, init_regexec,
    init_regexec_multi, make_extmatch, prog_magic_wrong, reg_endzp, reg_endzpos, reg_getline,
    reg_startzp, reg_startzpos, reg_tofree, reg_tofreelen, reg_toolong, unref_extmatch,
};
use crate::strings::{vim_strchr, xstrnsave};
use crate::types::{
    buf_T, colnr_T, linenr_T, proftime_T, reg_extmatch_T, regmatch_T, regmmatch_T, uint8_t,
    uint32_t, win_T,
};

/// How many start columns may be tried between two reads of the caller's
/// time limit.
const TIME_CHECK_INTERVAL: c_int = 20;

/// How large `reg_tofree` — the copy of the line a `\n`-crossing match works
/// over — may be left lying around between matches.
const REG_TOFREE_KEEP: u32 = 400;

/// Try to match the whole pattern starting at column `col`.
///
/// Returns 0 for no match, or one more than the line the match ended on.
fn regtry(
    rex: Rex,
    prog: *mut bt_regprog_T,
    col: colnr_T,
    tm: *const proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    // SAFETY: `prog` is a live program of this engine and `rex` the match
    // context `bt_regexec_both` set up, including the caller's capture
    // arrays.
    unsafe {
        rex.set_input(rex.line().offset(col as isize));
        rex.set_need_clear_subexpr(1);
        rex.set_need_clear_zsubexpr(((*prog).reghasz as c_int == REX_SET) as c_int);

        // The program starts one byte in: the first byte is the `MAGIC`
        // stamp `prog_magic_wrong` checks.
        if !regmatch(
            rex,
            (&raw mut (*prog).program).cast::<uint8_t>().add(1),
            tm,
            timed_out,
        ) {
            return 0;
        }

        cleanup_subexpr(rex);
        if rex.multi() {
            // A `\zs` before the start, or a `\ze` before the end, can leave
            // group 0 unset; it then covers what the matcher actually walked.
            let start = rex.reg_startpos();
            let end = rex.reg_endpos();
            if (*start).lnum < 0 {
                (*start).lnum = 0;
                (*start).col = col;
            }
            if (*end).lnum < 0 {
                (*end).lnum = rex.lnum();
                (*end).col = rex.col();
            } else {
                rex.set_lnum((*end).lnum);
            }
        } else {
            if (*rex.reg_startp()).is_null() {
                *rex.reg_startp() = rex.line().offset(col as isize);
            }
            if (*rex.reg_endp()).is_null() {
                *rex.reg_endp() = rex.input();
            }
        }

        // The `\z(` captures go to the syntax highlighter as fresh copies,
        // because it keeps them past the end of this match.
        unref_extmatch(re_extmatch_out.get());
        re_extmatch_out.set(core::ptr::null_mut::<reg_extmatch_T>());
        if (*prog).reghasz as c_int == REX_SET {
            cleanup_zsubexpr(rex);
            re_extmatch_out.set(make_extmatch());
            save_z_captures(rex);
        }
        1 + rex.lnum() as c_int
    }
}

/// Copy what the `\z(` groups matched into the set the highlighter reads.
///
/// SAFETY: `re_extmatch_out` holds a fresh capture set, the capture arrays
/// hold `NSUBEXP` slots each, and the match context is still the one that
/// filled them.
fn save_z_captures(rex: Rex) {
    unsafe {
        for i in 0..NSUBEXP as usize {
            let text = if rex.multi() {
                let (start, end) = ((*reg_startzpos.ptr())[i], (*reg_endzpos.ptr())[i]);
                // A capture that spans lines cannot be handed over as one
                // string, so it is dropped.
                if start.lnum < 0 || end.lnum != start.lnum || end.col < start.col {
                    continue;
                }
                xstrnsave(
                    reg_getline(rex, start.lnum).offset(start.col as isize),
                    (end.col - start.col) as usize,
                )
            } else {
                let (start, end) = ((*reg_startzp.ptr())[i], (*reg_endzp.ptr())[i]);
                if start.is_null() || end.is_null() {
                    continue;
                }
                xstrnsave(start.cast::<c_char>(), end.offset_from(start) as usize)
            };
            (*re_extmatch_out.get()).matches[i] = text as *mut uint8_t;
        }
    }
}

/// Match `rex`'s program against `line`, starting at `startcol`.
///
/// SAFETY: `rex` has been pointed at the caller's match structure.
fn bt_regexec_both(
    rex: Rex,
    mut line: *mut uint8_t,
    startcol: colnr_T,
    tm: *const proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    unsafe {
        // The loop back-edge record is kept between calls, so that an
        // ordinary match never allocates. `bt_regexec_both` is not
        // re-entered: nothing the matcher runs calls back into the editor.
        // The saved-state stack does the same for itself, in `RegStack`.
        if (*backpos.ptr()).ga_data.is_null() {
            ga_init(
                backpos.ptr(),
                size_of::<backpos_T>() as c_int,
                BACKPOS_INITIAL,
            );
            ga_grow(backpos.ptr(), BACKPOS_INITIAL);
            ga_set_growsize(backpos.ptr(), BACKPOS_INITIAL * 8);
        }

        let mut col = startcol;
        let prog: *mut bt_regprog_T = if rex.multi() {
            line = reg_getline(rex, 0) as *mut uint8_t;
            rex.set_reg_startpos((&raw mut (*rex.reg_mmatch()).startpos).cast());
            rex.set_reg_endpos((&raw mut (*rex.reg_mmatch()).endpos).cast());
            (*rex.reg_mmatch()).regprog.cast()
        } else {
            rex.set_reg_startp((&raw mut (*rex.reg_match()).startp).cast());
            rex.set_reg_endp((&raw mut (*rex.reg_match()).endp).cast());
            (*rex.reg_match()).regprog.cast()
        };

        let mut retval = 0;
        if prog.is_null() || line.is_null() {
            iemsg(gettext(&raw const e_null as *const c_char));
        } else if prog_magic_wrong(rex) == 0 && !(rex.reg_maxcol() > 0 && col >= rex.reg_maxcol()) {
            // The pattern's own `\c`/`\C`/`\Z` override what the caller asked
            // for.
            if (*prog).regflags & RF_ICASE as u32 != 0 {
                rex.set_reg_ic(true);
            } else if (*prog).regflags & RF_NOICASE as u32 != 0 {
                rex.set_reg_ic(false);
            }
            if (*prog).regflags & RF_ICOMBINE as u32 != 0 {
                rex.set_reg_icombine(true);
            }

            // A pattern with a literal run in it cannot match a line that
            // does not hold that run anywhere past `col`.
            if (*prog).regmust.is_null() || has_regmust(rex, prog, line, col) {
                rex.set_line(line);
                rex.set_lnum(0);
                reg_toolong.set(0);
                retval = if (*prog).reganch != 0 {
                    try_anchored(rex, prog, col, tm, timed_out)
                } else {
                    scan_columns(rex, prog, &mut col, tm, timed_out)
                };
            }
        }

        // The line copy a `\n`-crossing match works over is kept for the next
        // one, but not when it grew large.
        if reg_tofreelen.get() > REG_TOFREE_KEEP {
            xfree(reg_tofree.get().cast());
            reg_tofree.set(core::ptr::null_mut());
        }
        // Likewise the two stacks: a pathological pattern must not leave its
        // working set behind.
        (*regstack.ptr()).trim();
        if (*backpos.ptr()).ga_maxlen > BACKPOS_INITIAL {
            ga_clear(backpos.ptr());
        }

        if retval > 0 {
            // A `\ze` can put the end before the start; report an empty match
            // rather than a backwards one.
            if rex.multi() {
                let rmm = rex.reg_mmatch();
                let (start, end) = ((*rmm).startpos[0], (*rmm).endpos[0]);
                if end.lnum < start.lnum || (end.lnum == start.lnum && end.col < start.col) {
                    (*rmm).endpos[0] = start;
                }
                (*rmm).rmm_matchcol = col;
            } else {
                let rm = rex.reg_match();
                if (*rm).endp[0] < (*rm).startp[0] {
                    (*rm).endp[0] = (*rm).startp[0];
                }
                (*rm).rm_matchcol = col;
            }
        }
        retval
    }
}

/// Does `line` hold the pattern's `regmust` run at or after `col`?
///
/// SAFETY: as `bt_regexec_both`. `regmust` is a NUL-terminated run the
/// compiler kept and the walk below stops at `line`'s terminator.
fn has_regmust(rex: Rex, prog: *mut bt_regprog_T, line: *mut uint8_t, col: colnr_T) -> bool {
    unsafe {
        let c = utf_ptr2char((*prog).regmust as *mut c_char);
        let mut s = line.offset(col as isize).cast::<c_char>();
        loop {
            // Case-insensitively, the first character has to be looked for
            // with folding, which is what `cstrchr` adds over `vim_strchr`.
            s = if rex.reg_ic() {
                cstrchr(rex, s, c)
            } else {
                vim_strchr(s, c)
            };
            if s.is_null() {
                return false;
            }
            if cstrncmp(rex, s, (*prog).regmust as *mut c_char, &mut (*prog).regmlen) == 0 {
                return true;
            }
            s = s.add(utfc_ptr2len(s) as usize);
        }
    }
}

/// An anchored pattern can only match at `col`, and only if the character
/// there is the one it starts with.
///
/// SAFETY: as `bt_regexec_both`; `rex.line` is set and `col` inside it.
fn try_anchored(
    rex: Rex,
    prog: *mut bt_regprog_T,
    col: colnr_T,
    tm: *const proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    unsafe {
        let start = (*prog).regstart;
        let c = utf_ptr2char((rex.line() as *mut c_char).offset(col as isize));
        let matches = start == NUL
            || start == c
            || (rex.reg_ic()
                && (utf_fold(start) == utf_fold(c)
                    // Latin-1 has case pairs that do not fold together.
                    || (c < 255 && start < 255 && mb_tolower(start) == mb_tolower(c))));
        if matches {
            regtry(rex, prog, col, tm, timed_out)
        } else {
            0
        }
    }
}

/// Try every column from `col` on, until one matches or the line runs out.
///
/// `col` comes back holding the column the reported match starts at.
///
/// SAFETY: as `bt_regexec_both`.
fn scan_columns(
    rex: Rex,
    prog: *mut bt_regprog_T,
    col: &mut colnr_T,
    tm: *const proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    unsafe {
        let mut tm_count = 0;
        while !got_int.get() {
            // The pattern's first character is known: skip straight to the
            // next place it occurs.
            if (*prog).regstart != NUL {
                let from = (rex.line() as *mut c_char).offset(*col as isize);
                let s = cstrchr(rex, from, (*prog).regstart);
                if s.is_null() {
                    return 0;
                }
                *col = s.cast::<uint8_t>().offset_from(rex.line()) as colnr_T;
            }
            if rex.reg_maxcol() > 0 && *col >= rex.reg_maxcol() {
                return 0;
            }

            let retval = regtry(rex, prog, *col, tm, timed_out);
            if retval > 0 {
                return retval;
            }

            // A failed attempt may have walked onto a later line.
            if rex.lnum() != 0 {
                rex.set_lnum(0);
                rex.set_line(reg_getline(rex, 0) as *mut uint8_t);
            }
            if *rex.line().offset(*col as isize) as c_int == NUL {
                return 0;
            }
            *col += utfc_ptr2len((rex.line() as *mut c_char).offset(*col as isize));

            if !tm.is_null() {
                tm_count += 1;
                if tm_count == TIME_CHECK_INTERVAL {
                    tm_count = 0;
                    if profile_passed_limit(*tm) {
                        if !timed_out.is_null() {
                            *timed_out = 1;
                        }
                        return 0;
                    }
                }
            }
        }
        0
    }
}

/// Match against a string, treating `\n` as an ordinary character when
/// `line_lbr` is set.
///
/// # Safety
///
/// `rmp` must hold a program this engine compiled, and `line` be a
/// NUL-terminated string.
pub(crate) unsafe extern "C" fn bt_regexec_nl(
    rmp: *mut regmatch_T,
    line: *mut uint8_t,
    col: colnr_T,
    line_lbr: bool,
) -> c_int {
    // SAFETY: the caller holds the context (`with_rex`) and hands us a
    // live match structure and a NUL-terminated line.
    let rex = unsafe { Rex::acquire() };
    init_regexec(rex, rmp, line_lbr);
    bt_regexec_both(rex, line, col, core::ptr::null(), core::ptr::null_mut())
}

/// Match against a buffer, starting at `lnum`.
///
/// # Safety
///
/// `rmp` must hold a program this engine compiled, and `buf`/`win` be the
/// buffer and window the match runs over.
pub(crate) unsafe extern "C" fn bt_regexec_multi(
    rmp: *mut regmmatch_T,
    win: *mut win_T,
    buf: *mut buf_T,
    lnum: linenr_T,
    col: colnr_T,
    tm: *mut proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    // SAFETY: the caller holds the context (`with_rex`) and hands us a live
    // match structure over a live buffer; `init_regexec_multi` points the
    // context at them, which is what `bt_regexec_both` reads it out of.
    let rex = unsafe { Rex::acquire() };
    init_regexec_multi(rex, rmp, win, buf, lnum);
    bt_regexec_both(rex, core::ptr::null_mut::<uint8_t>(), col, tm, timed_out)
}

/// The `\%23l`-family comparison: the node's operand against `val`, with the
/// `>`/`<`/`=` the pattern spelled out riding in the node's eighth byte.
///
/// SAFETY: `scan` is a `RE_LNUM`/`RE_COL`/`RE_VCOL` node, which the compiler
/// emits eight bytes long.
pub(crate) fn re_num_cmp(val: uint32_t, scan: *const uint8_t) -> bool {
    unsafe {
        let b = |i: usize| *scan.add(i) as u32;
        let n = (b(3) << 24) + (b(4) << 16) + (b(5) << 8) + b(6);
        match *scan.add(7) as c_int {
            c if c == '>' as c_int => val > n,
            c if c == '<' as c_int => val < n,
            _ => val == n,
        }
    }
}
