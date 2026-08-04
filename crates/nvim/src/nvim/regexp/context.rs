//! The match-time context: the lines an engine reads, the capture slots it
//! clears, and the position tests (`\%V`, `\<`, a back-reference) both
//! engines ask this layer rather than answering themselves.
//!
//! `rex` is that context. It is a global rather than a parameter because
//! the engines reach it from everywhere; [`super::api`] saves and restores
//! it around a nested match, which is what lets a `\=` expression run a
//! search of its own.
//!
//! Its fields are read through `GlobalCell::ptr`, not `with`/`with_mut`.
//! Everything here is on the per-character path of both engines, and
//! `with` is an outlined call that pushes and pops a debug borrow-table
//! entry: routing these accesses through it measurably slowed syntax
//! highlighting — enough to trip 'redrawtime' on a large file, which
//! disables highlighting for the buffer. `ptr` carries exactly the
//! obligations the C did.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::{
    Ctrl_V, MULTI_MULT, NSUBEXP, RA_FAIL, RA_MATCH, RA_NOMATCH, RE_NOBREAK, REGMAGIC, bt_regprog_T,
    cstrncmp, nfa_regengine, peekchr, re_multi_type, reg_endzp, reg_endzpos, reg_startzp,
    reg_startzpos, reg_tofree, reg_tofreelen, rex, rsm,
};
use crate::semsg;
use crate::src::nvim::charset::vim_iswordc_buf;
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_mode, curbuf, curwin, e_re_corr, got_int, p_sel, rc_did_emsg,
};
use crate::src::nvim::mbyte::{mb_get_class_tab, mb_strnicmp, utf_head_off};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::input::fast_breakcheck;
use crate::src::nvim::os::libc::{gettext, strcpy, strlen};
use crate::src::nvim::plines::{getvvcol, win_linetabsize};
use crate::src::nvim::pos::{MAXCOL, lt};
use crate::src::nvim::types::{
    buf_T, colnr_T, linenr_T, lpos_T, reg_extmatch_T, regmatch_T, regmmatch_T, regprog_T, uint8_t,
    win_T,
};

/// Let the user interrupt a long match, unless the caller asked for an
/// uninterruptible one (`RE_NOBREAK`, for matches run where input cannot
/// be read).
pub(crate) fn reg_breakcheck() {
    // SAFETY: reads `rex` and polls input state on the main thread.
    unsafe {
        if !(*rex.ptr()).reg_nobreak {
            fast_breakcheck();
        }
    }
}

/// Is `c` a keyword character? 'iskeyword' is buffer-local and the buffer
/// being matched is not always the current one.
pub(crate) fn reg_iswordc(c: c_int) -> bool {
    // SAFETY: `reg_buf` is the buffer the match was set up against.
    unsafe { vim_iswordc_buf(c, (*rex.ptr()).reg_buf) }
}

/// Which line numbering to resolve against: the running match, or the
/// snapshot `submatch()` and a `\=` expression see.
#[derive(Clone, Copy)]
pub(crate) enum LineOrigin {
    Exec,
    Submatch,
}

/// Where a line relative to the match's first line lands.
enum Located {
    /// Above line 1 — the caller asked for context before the buffer.
    Before,
    /// Past the match's last line.
    Past,
    At(linenr_T),
}

fn locate(lnum: linenr_T, first: linenr_T, maxline: linenr_T) -> Located {
    if first + lnum < 1 {
        Located::Before
    } else if lnum > maxline {
        Located::Past
    } else {
        Located::At(first + lnum)
    }
}

/// The text `lnum` lines into the match: NULL above the buffer, an empty
/// string past the match's last line. Note that a submatch line comes from
/// `rex`'s buffer even though its numbering comes from `rsm`.
pub(crate) fn reg_line(lnum: linenr_T, origin: LineOrigin) -> *mut c_char {
    // SAFETY: reads the match context; `locate` establishes the line is in
    // the buffer before it is fetched.
    unsafe {
        let (first, maxline) = match origin {
            LineOrigin::Exec => ((*rex.ptr()).reg_firstlnum, (*rex.ptr()).reg_maxline),
            LineOrigin::Submatch => ((*rsm.ptr()).sm_firstlnum, (*rsm.ptr()).sm_maxline),
        };
        match locate(lnum, first, maxline) {
            Located::Before => core::ptr::null_mut(),
            Located::Past => c"".as_ptr().cast_mut(),
            Located::At(lnum) => ml_get_buf((*rex.ptr()).reg_buf, lnum),
        }
    }
}

/// The length of [`reg_line`]'s text; 0 for either stand-in.
pub(crate) fn reg_line_len(lnum: linenr_T, origin: LineOrigin) -> colnr_T {
    // SAFETY: as `reg_line`.
    unsafe {
        let (first, maxline) = match origin {
            LineOrigin::Exec => ((*rex.ptr()).reg_firstlnum, (*rex.ptr()).reg_maxline),
            LineOrigin::Submatch => ((*rsm.ptr()).sm_firstlnum, (*rsm.ptr()).sm_maxline),
        };
        match locate(lnum, first, maxline) {
            Located::Before | Located::Past => 0,
            Located::At(lnum) => ml_get_buf_len((*rex.ptr()).reg_buf, lnum),
        }
    }
}

pub(crate) fn reg_getline(lnum: linenr_T) -> *mut c_char {
    reg_line(lnum, LineOrigin::Exec)
}

pub(crate) fn reg_getline_len(lnum: linenr_T) -> colnr_T {
    reg_line_len(lnum, LineOrigin::Exec)
}

/// A fresh `\z1`..`\z9` capture set, refcounted because a syntax item
/// hands it to a highlighter that outlives the match.
pub(crate) fn make_extmatch() -> *mut reg_extmatch_T {
    // SAFETY: xcalloc returns a zeroed allocation of the requested size.
    unsafe {
        let em = xcalloc(1, size_of::<reg_extmatch_T>()) as *mut reg_extmatch_T;
        (*em).refcnt = 1;
        em
    }
}

/// Take a reference to `em`, which may be NULL.
///
/// # Safety
///
/// `em` must be null or a live [`make_extmatch`] allocation.
pub unsafe fn ref_extmatch(em: *mut reg_extmatch_T) -> *mut reg_extmatch_T {
    if !em.is_null() {
        unsafe { (*em).refcnt += 1 };
    }
    em
}

/// Drop a reference to `em`, freeing it and its captures at zero.
///
/// # Safety
///
/// `em` must be null or a live [`make_extmatch`] allocation.
pub unsafe fn unref_extmatch(em: *mut reg_extmatch_T) {
    unsafe {
        if em.is_null() {
            return;
        }
        (*em).refcnt -= 1;
        if (*em).refcnt > 0 {
            return;
        }
        for m in (*em).matches {
            xfree(m.cast());
        }
        xfree(em.cast());
    }
}

/// The character class of the character before the cursor, or -1 at the
/// start of the line. Backs `\<` and `\>`.
pub(crate) fn reg_prev_class() -> c_int {
    // SAFETY: `input` and `line` point into the line being matched, and
    // `utf_head_off` walks back no further than `line`.
    unsafe {
        let line = (*rex.ptr()).line as *mut c_char;
        let input = (*rex.ptr()).input as *mut c_char;
        if input <= line {
            return -1;
        }
        let prev = input.sub(1);
        mb_get_class_tab(
            prev.sub(utf_head_off(line, prev) as usize),
            &raw mut (*(*rex.ptr()).reg_buf).b_chartab as *mut u64,
        )
    }
}

/// Is the position being matched inside the Visual area? Backs `\%V`.
pub(crate) fn reg_match_visual() -> bool {
    // SAFETY: reads window and buffer state on the main thread; every
    // pointer is `curwin`/`curbuf` or the match's own window.
    unsafe {
        let wp = match (*rex.ptr()).reg_win {
            w if w.is_null() => curwin.get(),
            w => w,
        };
        // `\%V` is a buffer-position test, so it only applies to a
        // multi-line match in the current buffer.
        if (*rex.ptr()).reg_buf != curbuf.get()
            || (*VIsual.ptr()).lnum == 0
            || !(*rex.ptr()).reg_match.is_null()
        {
            return false;
        }

        let (top, bot, mode, curswant) = if VIsual_active.get() {
            let (top, bot) = if lt(VIsual.get(), (*wp).w_cursor) {
                (VIsual.get(), (*wp).w_cursor)
            } else {
                ((*wp).w_cursor, VIsual.get())
            };
            (top, bot, VIsual_mode.get(), (*wp).w_curswant)
        } else {
            // Not in Visual mode: the buffer's last Visual area.
            let buf = curbuf.get();
            let (start, end) = ((*buf).b_visual.vi_start, (*buf).b_visual.vi_end);
            let (top, mut bot) = if lt(start, end) {
                (start, end)
            } else {
                (end, start)
            };
            bot.lnum = bot.lnum.min((*buf).b_ml.ml_line_count);
            (
                top,
                bot,
                (*buf).b_visual.vi_mode,
                (*buf).b_visual.vi_curswant,
            )
        };

        let lnum = (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum;
        if lnum < top.lnum || lnum > bot.lnum {
            return false;
        }
        let col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
        if mode == 'v' as c_int {
            // 'selection' decides whether the last character is included.
            let inclusive = (*p_sel.get() as u8 != b'e') as colnr_T;
            !((lnum == top.lnum && col < top.col)
                || (lnum == bot.lnum && col >= bot.col + inclusive))
        } else if mode == Ctrl_V {
            let (mut start, mut end, mut start2, mut end2) = (0, 0, 0, 0);
            let nul = core::ptr::null_mut();
            let (mut top, mut bot) = (top, bot);
            getvvcol(wp, &raw mut top, &raw mut start, nul, &raw mut end);
            getvvcol(wp, &raw mut bot, &raw mut start2, nul, &raw mut end2);
            start = start.min(start2);
            end = end.max(end2);
            // `$` in blockwise Visual stretches the block to end of line.
            if top.col == MAXCOL as c_int
                || bot.col == MAXCOL as c_int
                || curswant == MAXCOL as c_int
            {
                end = MAXCOL as c_int;
            }
            // `getvvcol` can have moved the line out from under the match.
            let line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
            (*rex.ptr()).line = line;
            (*rex.ptr()).input = line.offset(col as isize);
            let lnum = (*rex.ptr()).reg_firstlnum + (*rex.ptr()).lnum;
            let cols = win_linetabsize(wp, lnum, line as *mut c_char, col);
            cols >= start && cols <= end - (*p_sel.get() as u8 == b'e') as colnr_T
        } else {
            true
        }
    }
}

/// Does the running program belong to the backtracking engine, and has its
/// magic number survived? Only that engine's programs carry one.
pub(crate) fn prog_magic_wrong() -> c_int {
    // SAFETY: a running match holds a live program.
    unsafe {
        let prog: *mut regprog_T = if (*rex.ptr()).reg_match.is_null() {
            (*(*rex.ptr()).reg_mmatch).regprog
        } else {
            (*(*rex.ptr()).reg_match).regprog
        };
        if (*prog).engine == nfa_regengine.ptr() {
            return 0;
        }
        if *(&raw mut (*(prog as *mut bt_regprog_T)).program as *mut uint8_t) as c_int != REGMAGIC {
            emsg(gettext(&raw const e_re_corr as *const c_char));
            return 1;
        }
        0
    }
}

/// Reset the `\1`..`\9` slots, or with `z` the `\z1`..`\z9` ones — which
/// live in this module's own arrays rather than in the caller's match
/// structure. Lazy: an engine only pays for this if a match reaches a
/// back-reference.
fn cleanup(z: bool) {
    // SAFETY: the caller's match structure and this module's own arrays
    // both hold NSUBEXP of each.
    unsafe {
        let need = if z {
            (*rex.ptr()).need_clear_zsubexpr
        } else {
            (*rex.ptr()).need_clear_subexpr
        };
        if need == 0 {
            return;
        }
        // A buffer match records positions and marks a slot unset with -1;
        // a string match records pointers and marks it unset with NULL.
        let multi = (*rex.ptr()).reg_match.is_null();
        let n = NSUBEXP as usize;
        if multi {
            let (starts, ends) = if z {
                (reg_startzpos.ptr().cast(), reg_endzpos.ptr().cast())
            } else {
                ((*rex.ptr()).reg_startpos, (*rex.ptr()).reg_endpos)
            };
            blank(
                core::slice::from_raw_parts_mut(starts, n),
                lpos_T { lnum: -1, col: -1 },
            );
            blank(
                core::slice::from_raw_parts_mut(ends, n),
                lpos_T { lnum: -1, col: -1 },
            );
        } else {
            let (starts, ends) = if z {
                (reg_startzp.ptr().cast(), reg_endzp.ptr().cast())
            } else {
                ((*rex.ptr()).reg_startp, (*rex.ptr()).reg_endp)
            };
            blank(
                core::slice::from_raw_parts_mut(starts, n),
                core::ptr::null_mut::<uint8_t>(),
            );
            blank(
                core::slice::from_raw_parts_mut(ends, n),
                core::ptr::null_mut::<uint8_t>(),
            );
        }
        if z {
            (*rex.ptr()).need_clear_zsubexpr = 0;
        } else {
            (*rex.ptr()).need_clear_subexpr = 0;
        }
    }
}

fn blank<T: Copy>(slots: &mut [T], unset: T) {
    slots.fill(unset);
}

pub(crate) fn cleanup_subexpr() {
    cleanup(false);
}

pub(crate) fn cleanup_zsubexpr() {
    cleanup(true);
}

/// Step the match on to the next line.
pub(crate) fn reg_nextline() {
    // SAFETY: reads and advances the match context.
    unsafe {
        (*rex.ptr()).lnum += 1;
        (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
        (*rex.ptr()).input = (*rex.ptr()).line;
    }
    reg_breakcheck();
}

/// Match the text a back-reference captured, which may span lines. On
/// success `*bytelen` is the length matched on the *last* line, which is
/// what the caller advances by.
///
pub(crate) fn match_with_backref(
    start_lnum: linenr_T,
    start_col: colnr_T,
    end_lnum: linenr_T,
    end_col: colnr_T,
    mut bytelen: Option<&mut c_int>,
) -> c_int {
    // SAFETY: the positions come from this match's own capture slots, and
    // every pointer below is the match's own line or scratch buffer.
    unsafe {
        let mut clnum = start_lnum;
        let mut ccol = start_col;
        if let Some(n) = bytelen.as_deref_mut() {
            *n = 0;
        }
        loop {
            // `reg_getline` below hands out the memline's own buffer, so
            // it can invalidate the line being matched. Take a private
            // copy first, growing the scratch buffer when it no longer
            // fits.
            if (*rex.ptr()).line != reg_tofree.get() {
                let mut len = strlen((*rex.ptr()).line as *mut c_char) as c_int;
                if reg_tofree.get().is_null() || len >= reg_tofreelen.get() as c_int {
                    len += 50;
                    xfree(reg_tofree.get().cast());
                    reg_tofree.set(xmalloc(len as usize) as *mut uint8_t);
                    reg_tofreelen.set(len as u32);
                }
                strcpy(
                    reg_tofree.get() as *mut c_char,
                    (*rex.ptr()).line as *mut c_char,
                );
                (*rex.ptr()).input = reg_tofree
                    .get()
                    .offset((*rex.ptr()).input.offset_from((*rex.ptr()).line));
                (*rex.ptr()).line = reg_tofree.get();
            }

            let p = reg_getline(clnum);
            assert!(!p.is_null(), "p");
            let mut len = if clnum == end_lnum {
                end_col - ccol
            } else {
                reg_getline_len(clnum) - ccol
            };
            let captured = p.offset(ccol as isize);
            let input = (*rex.ptr()).input as *mut c_char;
            // `cstrncmp` can shorten `len` when a fold changed the encoded
            // length, and the caller is told how far to advance from it.
            let differs = if (*rex.ptr()).reg_ic {
                mb_strnicmp(captured, input, len as usize) != 0
            } else {
                cstrncmp(captured, input, &mut len) != 0
            };
            if differs {
                return RA_NOMATCH;
            }
            if let Some(n) = bytelen.as_deref_mut() {
                *n += len;
            }
            if clnum == end_lnum {
                return RA_MATCH;
            }
            if (*rex.ptr()).lnum >= (*rex.ptr()).reg_maxline {
                return RA_NOMATCH;
            }
            // The capture continues on the next line, so the match must
            // too, and `*bytelen` restarts from that line's column 0.
            reg_nextline();
            if let Some(n) = bytelen.as_deref_mut() {
                *n = 0;
            }
            clnum += 1;
            ccol = 0;
            if got_int.get() {
                return RA_FAIL;
            }
        }
    }
}

/// Reject a repeat applied to `what`, a zero-width atom such as `\zs`.
pub(crate) fn re_mult_next(what: &str) -> bool {
    if re_multi_type(peekchr()) == MULTI_MULT {
        semsg!("E888: (NFA regexp) cannot repeat {what}");
        rc_did_emsg.set(true);
        return false;
    }
    true
}

/// Point `rex` at a buffer match about to run.
///
/// `rmp` must be live, with a compiled program, for the match's duration,
/// and `buf` must be the buffer it runs over.
pub(crate) fn init_regexec_multi(
    rmp: *mut regmmatch_T,
    win: *mut win_T,
    buf: *mut buf_T,
    lnum: linenr_T,
) {
    // SAFETY: the caller's match structure and buffer, live for the match.
    unsafe {
        let r = &mut *rex.ptr();
        r.reg_match = core::ptr::null_mut::<regmatch_T>();
        r.reg_mmatch = rmp;
        r.reg_buf = buf;
        r.reg_win = win;
        r.reg_firstlnum = lnum;
        r.reg_maxline = (*buf).b_ml.ml_line_count - lnum;
        r.reg_line_lbr = false;
        r.reg_ic = (*rmp).rmm_ic != 0;
        r.reg_icombine = false;
        r.reg_nobreak = (*(*rmp).regprog).re_flags & RE_NOBREAK as u32 != 0;
        r.reg_maxcol = (*rmp).rmm_maxcol;
    }
}
