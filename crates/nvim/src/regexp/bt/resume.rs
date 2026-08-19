//! Unwinding the saved-state stack.
//!
//! Every frame [`super::matcher::push_frame`] pushed describes one decision
//! the forward walk made. When the walk stops — because it failed, or because
//! it reached `END` — control comes here and the frames are popped in turn.
//! Each state either accepts the outcome and pops, or produces another thing
//! to try and hands control back to the forward walk with `RA_CONT`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::compile::regnext;
use super::matcher::operand_u32;
use super::repeat::regrepeat;
use super::state::RegStack;
use crate::main::got_int;
use crate::mbyte::utf_head_off;
use crate::regexp::{
    BEHIND, BRANCH, NOBEHIND, NOMATCH, RA_BREAK, RA_CONT, RA_FAIL, RA_MATCH, RA_NOMATCH,
    RS_BEHIND1, RS_BEHIND2, RS_BRANCH, RS_BRCPLX_LONG, RS_BRCPLX_MORE, RS_BRCPLX_SHORT, RS_MCLOSE,
    RS_MOPEN, RS_NOMATCH, RS_NOPEN, RS_STAR_LONG, RS_STAR_SHORT, RS_ZCLOSE, RS_ZOPEN, Rex, SUBPAT,
    backpos, behind_pos, brace_count, reg_breakcheck, reg_endzp, reg_endzpos, reg_getline,
    reg_getline_len, reg_restore, reg_save, reg_save_equal, reg_startzp, reg_startzpos,
    restore_subexpr,
};
use crate::types::{FAIL, NUL, OK, colnr_T, uint8_t};
use ::libc::strlen;

/// Pop frames until one of them has something new to try, or the stack runs
/// out. Updates `scan` and `status` in place.
pub(crate) fn resume(rex: Rex, stack: &mut RegStack, scan: &mut *mut uint8_t, status: &mut c_int) {
    // SAFETY: the frames on `stack` were pushed by this match and still
    // describe live positions in the program and the input; `rex` is the
    // running match.
    unsafe {
        while stack.depth() > 0 && *status != RA_FAIL {
            let depth = stack.depth();
            match stack.top().rs_state {
                // `\%(`: nothing was saved.
                RS_NOPEN => stack.pop(scan),

                // A capture boundary: put the slot back if the rest of the
                // pattern did not work out.
                RS_MOPEN => {
                    if *status == RA_NOMATCH {
                        let rp = stack.top();
                        let no = rp.rs_no as usize;
                        if rex.multi() {
                            *rex.reg_startpos().add(no) = rp.rs_un.sesave.se_u.pos;
                        } else {
                            *rex.reg_startp().add(no) = rp.rs_un.sesave.se_u.ptr;
                        }
                    }
                    stack.pop(scan);
                }
                RS_MCLOSE => {
                    if *status == RA_NOMATCH {
                        let rp = stack.top();
                        let no = rp.rs_no as usize;
                        if rex.multi() {
                            *rex.reg_endpos().add(no) = rp.rs_un.sesave.se_u.pos;
                        } else {
                            *rex.reg_endp().add(no) = rp.rs_un.sesave.se_u.ptr;
                        }
                    }
                    stack.pop(scan);
                }
                RS_ZOPEN => {
                    if *status == RA_NOMATCH {
                        let rp = stack.top();
                        let no = rp.rs_no as usize;
                        if rex.multi() {
                            (*reg_startzpos.ptr())[no] = rp.rs_un.sesave.se_u.pos;
                        } else {
                            (*reg_startzp.ptr())[no] = rp.rs_un.sesave.se_u.ptr;
                        }
                    }
                    stack.pop(scan);
                }
                RS_ZCLOSE => {
                    if *status == RA_NOMATCH {
                        let rp = stack.top();
                        let no = rp.rs_no as usize;
                        if rex.multi() {
                            (*reg_endzpos.ptr())[no] = rp.rs_un.sesave.se_u.pos;
                        } else {
                            (*reg_endzp.ptr())[no] = rp.rs_un.sesave.se_u.ptr;
                        }
                    }
                    stack.pop(scan);
                }

                // The next alternative of a `\|`, if there is one.
                RS_BRANCH => {
                    if *status == RA_MATCH {
                        stack.pop(scan);
                    } else {
                        // RA_BREAK means the branch was only just pushed and
                        // the walk has not moved yet.
                        if *status != RA_BREAK {
                            reg_restore(rex, &stack.top().rs_un.regsave, backpos.ptr());
                            *scan = stack.top().rs_scan;
                        }
                        if scan.is_null() || **scan as c_int != BRANCH {
                            *status = RA_NOMATCH;
                            stack.pop(scan);
                        } else {
                            let next = regnext(*scan);
                            let rp = stack.top_mut();
                            rp.rs_scan = next;
                            reg_save(rex, &mut rp.rs_un.regsave, backpos.ptr());
                            *scan = scan.add(3);
                        }
                    }
                }

                // A mandatory `\{n,m}` pass that did not work out: give the
                // count back and let the failure propagate.
                RS_BRCPLX_MORE => {
                    if *status == RA_NOMATCH {
                        let rp = stack.top();
                        reg_restore(rex, &rp.rs_un.regsave, backpos.ptr());
                        (*brace_count.ptr())[rp.rs_no as usize] -= 1;
                    }
                    stack.pop(scan);
                }
                // A greedy optional pass: on failure, stop looping instead.
                RS_BRCPLX_LONG => {
                    if *status == RA_NOMATCH {
                        let rp = stack.top();
                        reg_restore(rex, &rp.rs_un.regsave, backpos.ptr());
                        (*brace_count.ptr())[rp.rs_no as usize] -= 1;
                        *status = RA_CONT;
                    }
                    stack.pop(scan);
                    if *status == RA_CONT {
                        *scan = regnext(*scan);
                    }
                }
                // A non-greedy `\{-n,m}`: stopping was tried first, so on
                // failure take another pass after all.
                RS_BRCPLX_SHORT => {
                    if *status == RA_NOMATCH {
                        reg_restore(rex, &stack.top().rs_un.regsave, backpos.ptr());
                    }
                    stack.pop(scan);
                    if *status == RA_NOMATCH {
                        *scan = scan.add(3);
                        *status = RA_CONT;
                    }
                }

                // `\@=`, `\@!`, `\@>`: the operand's outcome, inverted for
                // `\@!`. `\@>` keeps whatever the operand consumed.
                RS_NOMATCH => {
                    let no = stack.top().rs_no as c_int;
                    let want = if no == NOMATCH { RA_MATCH } else { RA_NOMATCH };
                    if *status == want {
                        *status = RA_NOMATCH;
                    } else {
                        *status = RA_CONT;
                        if no != SUBPAT {
                            reg_restore(rex, &stack.top().rs_un.regsave, backpos.ptr());
                        }
                    }
                    stack.pop(scan);
                    if *status == RA_CONT {
                        *scan = regnext(*scan);
                    }
                }

                RS_BEHIND1 => behind_start(rex, stack, scan, status),
                RS_BEHIND2 => behind_step(rex, stack, scan, status),
                RS_STAR_LONG | RS_STAR_SHORT => star(rex, stack, scan, status),
                _ => {}
            }

            // Stop once something wants to run forward again, or once a
            // state kept its own frame and would just be re-entered.
            if *status == RA_CONT || stack.depth() == depth {
                break;
            }
        }
    }
}

/// `RS_BEHIND1`: the pattern *after* the look-behind has been reached, so now
/// the look-behind's own operand has to be run, ending here.
///
/// # Safety
/// As `resume`.
unsafe fn behind_start(
    rex: Rex,
    stack: &mut RegStack,
    scan: &mut *mut uint8_t,
    status: &mut c_int,
) {
    // SAFETY: as `resume`; an `RS_BEHIND1` frame carries a `regbehind_T`.
    unsafe {
        if *status == RA_NOMATCH {
            stack.pop_behind(scan);
            return;
        }
        let (rp, bp) = stack.top_behind();
        reg_save(rex, &mut bp.save_after, backpos.ptr());
        bp.save_behind = behind_pos.get();
        // The position the operand has to end at.
        behind_pos.set(rp.rs_un.regsave);
        rp.rs_state = RS_BEHIND2;
        reg_restore(rex, &rp.rs_un.regsave, backpos.ptr());
        // Past the node header and the four-byte limit.
        *scan = rp.rs_scan.add(3 + 4);
    }
}

/// `RS_BEHIND2`: one attempt at the look-behind is over. Either it ended in
/// the right place, or the start position steps back one character and the
/// operand runs again.
///
/// # Safety
/// As `resume`.
unsafe fn behind_step(rex: Rex, stack: &mut RegStack, scan: &mut *mut uint8_t, status: &mut c_int) {
    // SAFETY: as `behind_start`.
    unsafe {
        let (rp, bp) = stack.top_behind();

        // It matched, and it ended exactly where the look-behind sits.
        if *status == RA_MATCH && reg_save_equal(rex, &*behind_pos.ptr()) {
            behind_pos.set(bp.save_behind);
            if rp.rs_no as c_int == BEHIND {
                reg_restore(rex, &bp.save_after, backpos.ptr());
            } else {
                // `\@<!` wanted it *not* to match.
                *status = RA_NOMATCH;
                restore_subexpr(rex, bp);
            }
            stack.pop_behind(scan);
            return;
        }

        // Step the start position back one character, unless that would run
        // past the `\{n}` limit on how far back to look.
        let limit = operand_u32(rp.rs_scan, 3);
        let mut no = OK;
        if rex.multi() {
            let start = &mut rp.rs_un.regsave;
            let end_col = if start.rs_u.pos.lnum < (*behind_pos.ptr()).rs_u.pos.lnum {
                strlen(rex.line().cast()) as colnr_T
            } else {
                (*behind_pos.ptr()).rs_u.pos.col
            };
            if limit > 0 && (end_col - start.rs_u.pos.col) as i64 >= limit {
                no = FAIL;
            } else if start.rs_u.pos.col == 0 {
                // At the start of a line: continue on the line before it.
                // The decrement is deliberately inside the short circuit.
                if start.rs_u.pos.lnum < (*behind_pos.ptr()).rs_u.pos.lnum || {
                    start.rs_u.pos.lnum -= 1;
                    reg_getline(rex, start.rs_u.pos.lnum).is_null()
                } {
                    no = FAIL;
                } else {
                    reg_restore(rex, start, backpos.ptr());
                    start.rs_u.pos.col = strlen(rex.line().cast()) as colnr_T;
                }
            } else {
                let line = reg_getline(rex, start.rs_u.pos.lnum);
                start.rs_u.pos.col -=
                    utf_head_off(line, line.add(start.rs_u.pos.col as usize).sub(1)) + 1;
            }
        } else if rp.rs_un.regsave.rs_u.ptr == rex.line() {
            no = FAIL;
        } else {
            let start = &mut rp.rs_un.regsave;
            start.rs_u.ptr = start.rs_u.ptr.sub(
                utf_head_off(rex.line().cast(), start.rs_u.ptr.cast::<c_char>().sub(1)) as usize
                    + 1,
            );
            if limit > 0
                && (*behind_pos.ptr()).rs_u.ptr.offset_from(start.rs_u.ptr) > limit as isize
            {
                no = FAIL;
            }
        }

        if no == OK {
            reg_restore(rex, &rp.rs_un.regsave, backpos.ptr());
            *scan = rp.rs_scan.add(3 + 4);
            if *status == RA_MATCH {
                // It matched, but ending in the wrong place: try again from
                // one character further back.
                *status = RA_NOMATCH;
                restore_subexpr(rex, bp);
            }
        } else {
            // Nowhere left to look.
            behind_pos.set(bp.save_behind);
            if rp.rs_no as c_int == NOBEHIND {
                reg_restore(rex, &bp.save_after, backpos.ptr());
                *status = RA_MATCH;
            } else if *status == RA_MATCH {
                *status = RA_NOMATCH;
                restore_subexpr(rex, bp);
            }
            stack.pop_behind(scan);
        }
    }
}

/// `RS_STAR_LONG`/`RS_STAR_SHORT`: hand back one of the matches
/// [`super::repeat::regrepeat`] counted, greedily from the end or
/// non-greedily from the start.
///
/// # Safety
/// As `resume`.
unsafe fn star(rex: Rex, stack: &mut RegStack, scan: &mut *mut uint8_t, status: &mut c_int) {
    // SAFETY: as `resume`; an `RS_STAR_*` frame carries a `regstar_T`.
    unsafe {
        if *status == RA_MATCH {
            stack.pop_star(scan);
            return;
        }
        let (rp, rst) = stack.top_star();
        if *status != RA_BREAK {
            reg_restore(rex, &rp.rs_un.regsave, backpos.ptr());
        }

        loop {
            if *status != RA_BREAK {
                if rp.rs_state == RS_STAR_LONG {
                    // Greedy: give one match back.
                    rst.count -= 1;
                    if rst.count < rst.minval {
                        break;
                    }
                    if rex.at_bol() {
                        // Back over a line break, which a `\_x` may have
                        // consumed.
                        if rex.lnum() == 0 {
                            *status = RA_NOMATCH;
                            break;
                        }
                        rex.set_lnum(rex.lnum() - 1);
                        rex.set_line(reg_getline(rex, rex.lnum()).cast());
                        if rex.line().is_null() {
                            break;
                        }
                        rex.set_input(rex.line().add(reg_getline_len(rex, rex.lnum()) as usize));
                        reg_breakcheck(rex);
                    } else {
                        rex.set_input(rex.input().sub(
                            utf_head_off(rex.line().cast(), rex.input().cast::<c_char>().sub(1))
                                as usize
                                + 1,
                        ));
                    }
                } else {
                    // Non-greedy: take one more match.
                    if rst.count == rst.minval || regrepeat(rex, rp.rs_scan.add(3), 1) == 0 {
                        break;
                    }
                    rst.count += 1;
                }
                if got_int.get() {
                    break;
                }
            } else {
                *status = RA_NOMATCH;
            }
            // The byte the pattern needs next is known; positions that
            // cannot supply it are skipped without re-entering the walk.
            if !(rst.nextb == NUL
                || rex.byte() as c_int == rst.nextb
                || rex.byte() as c_int == rst.nextb_ic)
            {
                continue;
            }
            reg_save(rex, &mut rp.rs_un.regsave, backpos.ptr());
            *scan = regnext(rp.rs_scan);
            *status = RA_CONT;
            break;
        }

        if *status != RA_CONT {
            stack.pop_star(scan);
            *status = RA_NOMATCH;
        }
    }
}
