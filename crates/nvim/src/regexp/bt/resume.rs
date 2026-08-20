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
    BEHIND, BRANCH, MatchPos, NOBEHIND, NOMATCH, RA_BREAK, RA_CONT, RA_FAIL, RA_MATCH, RA_NOMATCH,
    RS_BEHIND1, RS_BEHIND2, RS_BRANCH, RS_BRCPLX_LONG, RS_BRCPLX_MORE, RS_BRCPLX_SHORT, RS_MCLOSE,
    RS_MOPEN, RS_NOMATCH, RS_NOPEN, RS_STAR_LONG, RS_STAR_SHORT, RS_ZCLOSE, RS_ZOPEN, Rex, SUBPAT,
    SavedInput, backpos, behind_pos, brace_count, reg_breakcheck, reg_endzp, reg_endzpos,
    reg_getline, reg_getline_len, reg_restore, reg_save, reg_startzp, reg_startzpos, regstate_T,
    restore_subexpr,
};
use crate::types::{NUL, colnr_T, int64_t, uint8_t};
use ::libc::strlen;

/// Pop frames until one of them has something new to try, or the stack runs
/// out. Updates `scan` and `status` in place.
pub(crate) fn resume(rex: Rex, stack: &mut RegStack, scan: &mut *mut uint8_t, status: &mut c_int) {
    while stack.depth() > 0 && *status != RA_FAIL {
        let depth = stack.depth();
        match stack.top().rs_state {
            // `\%(`: nothing was saved.
            RS_NOPEN => stack.pop(scan),

            // A capture boundary: put the slot back if the rest of the
            // pattern did not work out.
            RS_MOPEN | RS_MCLOSE | RS_ZOPEN | RS_ZCLOSE => {
                if *status == RA_NOMATCH {
                    let rp = stack.top();
                    // SAFETY: the frame was pushed for a capture opcode of
                    // the running program, so `rs_no` names a group the
                    // match holds both shapes of slot for.
                    unsafe { undo_capture(rex, rp.rs_state, rp.rs_no as usize, rp.rs_saved.pos) };
                }
                stack.pop(scan);
            }

            // The next alternative of a `\|`, if there is one.
            RS_BRANCH => {
                if *status == RA_MATCH {
                    stack.pop(scan);
                } else {
                    // RA_BREAK means the branch was only just pushed and the
                    // walk has not moved yet.
                    if *status != RA_BREAK {
                        reg_restore(rex, &stack.top().rs_saved, backpos.ptr());
                        *scan = stack.top().rs_scan;
                    }
                    // SAFETY: `scan` is null or a node of the running
                    // program, whose opcode byte and three-byte header are
                    // there to read.
                    if scan.is_null() || unsafe { **scan } as c_int != BRANCH {
                        *status = RA_NOMATCH;
                        stack.pop(scan);
                    } else {
                        let next = regnext(*scan);
                        let rp = stack.top_mut();
                        rp.rs_scan = next;
                        reg_save(rex, &mut rp.rs_saved, backpos.ptr());
                        // SAFETY: as above.
                        *scan = unsafe { scan.add(3) };
                    }
                }
            }

            // A mandatory `\{n,m}` pass that did not work out: give the
            // count back and let the failure propagate.
            RS_BRCPLX_MORE => {
                if *status == RA_NOMATCH {
                    let rp = stack.top();
                    reg_restore(rex, &rp.rs_saved, backpos.ptr());
                    give_back_a_pass(rp.rs_no as usize);
                }
                stack.pop(scan);
            }
            // A greedy optional pass: on failure, stop looping instead.
            RS_BRCPLX_LONG => {
                if *status == RA_NOMATCH {
                    let rp = stack.top();
                    reg_restore(rex, &rp.rs_saved, backpos.ptr());
                    give_back_a_pass(rp.rs_no as usize);
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
                    reg_restore(rex, &stack.top().rs_saved, backpos.ptr());
                }
                stack.pop(scan);
                if *status == RA_NOMATCH {
                    // SAFETY: `scan` is a node of the running program, whose
                    // three-byte header is there.
                    *scan = unsafe { scan.add(3) };
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
                        reg_restore(rex, &stack.top().rs_saved, backpos.ptr());
                    }
                }
                stack.pop(scan);
                if *status == RA_CONT {
                    *scan = regnext(*scan);
                }
            }

            // SAFETY: the frames these three read were pushed by this match
            // and still describe live positions in the program and the
            // input; `rex` is the running match.
            RS_BEHIND1 => unsafe { behind_start(rex, stack, scan, status) },
            RS_BEHIND2 => unsafe { behind_step(rex, stack, scan, status) },
            RS_STAR_LONG | RS_STAR_SHORT => unsafe { star(rex, stack, scan, status) },
            _ => {}
        }

        // Stop once something wants to run forward again, or once a state
        // kept its own frame and would just be re-entered.
        if *status == RA_CONT || stack.depth() == depth {
            break;
        }
    }
}

/// Put capture group `no`'s slot back to what the frame saved. Which of the
/// four slot arrays it is, is the frame's state.
///
/// # Safety
///
/// `no` must name a capture group the running match holds slots for.
unsafe fn undo_capture(rex: Rex, state: regstate_T, no: usize, saved: MatchPos) {
    // SAFETY: the caller promises the group; only the array this match's own
    // kind uses is reached, so the null one is never indexed.
    unsafe {
        if rex.multi() {
            let slot = match state {
                RS_MOPEN => rex.reg_startpos().add(no),
                RS_MCLOSE => rex.reg_endpos().add(no),
                RS_ZOPEN => &raw mut (*reg_startzpos.ptr())[no],
                _ => &raw mut (*reg_endzpos.ptr())[no],
            };
            *slot = saved.as_pos();
        } else {
            let slot = match state {
                RS_MOPEN => rex.reg_startp().add(no),
                RS_MCLOSE => rex.reg_endp().add(no),
                RS_ZOPEN => &raw mut (*reg_startzp.ptr())[no],
                _ => &raw mut (*reg_endzp.ptr())[no],
            };
            *slot = saved.as_ptr();
        }
    }
}

/// Undo one `\{n,m}` pass of the counted repeat in slot `no`.
fn give_back_a_pass(no: usize) {
    // SAFETY: `brace_count` is a fixed ten-slot global and `no` is a
    // `BRACE_COMPLEX` operand, which the compiler bounds to nine.
    unsafe { (*brace_count.ptr())[no] -= 1 };
}

/// `RS_BEHIND1`: the pattern *after* the look-behind has been reached, so now
/// the look-behind's own operand has to be run, ending here.
///
/// # Safety
///
/// The top frame must be an `RS_BEHIND1` one this match pushed, so that it
/// names a look-behind node of the running program and carries a
/// `regbehind_T`.
unsafe fn behind_start(
    rex: Rex,
    stack: &mut RegStack,
    scan: &mut *mut uint8_t,
    status: &mut c_int,
) {
    if *status == RA_NOMATCH {
        stack.pop_behind(scan);
        return;
    }
    let (rp, bp) = stack.top_behind();
    reg_save(rex, &mut bp.save_after, backpos.ptr());
    bp.save_behind = behind_pos.get();
    // The position the operand has to end at.
    behind_pos.set(rp.rs_saved);
    rp.rs_state = RS_BEHIND2;
    reg_restore(rex, &rp.rs_saved, backpos.ptr());
    // Past the node header and the four-byte limit.
    // SAFETY: the caller promises a look-behind node, which is seven bytes
    // of header and limit followed by its operand.
    *scan = unsafe { rp.rs_scan.add(3 + 4) };
}

/// `RS_BEHIND2`: one attempt at the look-behind is over. Either it ended in
/// the right place, or the start position steps back one character and the
/// operand runs again.
///
/// # Safety
/// As [`behind_start`], for an `RS_BEHIND2` frame.
unsafe fn behind_step(rex: Rex, stack: &mut RegStack, scan: &mut *mut uint8_t, status: &mut c_int) {
    let (rp, bp) = stack.top_behind();

    // It matched, and it ended exactly where the look-behind sits.
    if *status == RA_MATCH && rex.is_at(behind_pos.get().pos) {
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

    // Step the start position back one character, unless that would run past
    // the `\{n}` limit on how far back to look.
    // SAFETY: the caller promises a look-behind node, whose four-byte limit
    // follows its three-byte header.
    let limit = unsafe { operand_u32(rp.rs_scan, 3) };
    let stop = behind_pos.get().pos;
    // SAFETY: `rex` is the running match and `rs_saved` a position in it.
    let stepped = unsafe {
        if rex.multi() {
            step_back_lines(rex, &mut rp.rs_saved, stop, limit)
        } else {
            step_back_string(rex, &mut rp.rs_saved, stop, limit)
        }
    };

    if stepped {
        reg_restore(rex, &rp.rs_saved, backpos.ptr());
        // SAFETY: as the `limit` read above.
        *scan = unsafe { rp.rs_scan.add(3 + 4) };
        if *status == RA_MATCH {
            // It matched, but ending in the wrong place: try again from one
            // character further back.
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

/// Step a buffer match's look-behind start back one character, crossing into
/// the line before when it was already at a line start.
///
/// False when there is nowhere left to look — `stop` has been reached, or the
/// `\{n}` limit has.
///
/// # Safety
///
/// `rex` must be a live buffer match, and `start` a position in it.
unsafe fn step_back_lines(
    rex: Rex,
    start: &mut SavedInput,
    stop: MatchPos,
    limit: int64_t,
) -> bool {
    let (was, stop) = (start.pos.as_pos(), stop.as_pos());
    let end_col = if was.lnum < stop.lnum {
        // SAFETY: `rex.line` is the NUL-terminated line being matched.
        unsafe { strlen(rex.line().cast()) as colnr_T }
    } else {
        stop.col
    };
    if limit > 0 && (end_col - was.col) as i64 >= limit {
        return false;
    }
    if was.col == 0 {
        // At the start of a line: continue on the line before it. The
        // decrement is deliberately inside the short circuit.
        if was.lnum < stop.lnum || {
            start.pos.pos_mut().lnum -= 1;
            reg_getline(rex, start.pos.as_pos().lnum).is_null()
        } {
            return false;
        }
        reg_restore(rex, start, backpos.ptr());
        // SAFETY: as above.
        start.pos.pos_mut().col = unsafe { strlen(rex.line().cast()) } as colnr_T;
    } else {
        let line = reg_getline(rex, was.lnum);
        // SAFETY: `was.col` is a column of `line` and is not zero, so the
        // byte before it is on the line too.
        let head = unsafe { utf_head_off(line, line.add(was.col as usize).sub(1)) };
        start.pos.pos_mut().col -= head + 1;
    }
    true
}

/// [`step_back_lines`] for a string match, where there is one line and the
/// position is a pointer into it.
///
/// # Safety
///
/// `rex` must be a live string match, and `start` and `stop` positions in the
/// string it is matching.
unsafe fn step_back_string(
    rex: Rex,
    start: &mut SavedInput,
    stop: MatchPos,
    limit: int64_t,
) -> bool {
    let was = start.pos.as_ptr();
    if was == rex.line() {
        return false;
    }
    // SAFETY: `was` is in the string being matched and is not its first
    // byte, so the character before it is there to measure.
    let back = unsafe { utf_head_off(rex.line().cast(), was.cast::<c_char>().sub(1)) } as usize + 1;
    // SAFETY: `back` is that character's length, so it stays in the string.
    start.pos.set_ptr(unsafe { was.sub(back) });
    // SAFETY: both positions are in the one string being matched.
    limit <= 0 || unsafe { stop.as_ptr().offset_from(start.pos.as_ptr()) } <= limit as isize
}

/// `RS_STAR_LONG`/`RS_STAR_SHORT`: hand back one of the matches
/// [`super::repeat::regrepeat`] counted, greedily from the end or
/// non-greedily from the start.
///
/// # Safety
///
/// The top frame must be an `RS_STAR_*` one this match pushed, so that it
/// names a repeat node of the running program and carries a `regstar_T`.
unsafe fn star(rex: Rex, stack: &mut RegStack, scan: &mut *mut uint8_t, status: &mut c_int) {
    if *status == RA_MATCH {
        stack.pop_star(scan);
        return;
    }
    let (rp, rst) = stack.top_star();
    if *status != RA_BREAK {
        reg_restore(rex, &rp.rs_saved, backpos.ptr());
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
                    let len = reg_getline_len(rex, rex.lnum()) as usize;
                    // SAFETY: `len` is that line's length, so it lands on its
                    // NUL at worst.
                    rex.set_input(unsafe { rex.line().add(len) });
                    reg_breakcheck(rex);
                } else {
                    // SAFETY: the cursor is not at the start of the line, so
                    // the character before it is on the line.
                    let back = unsafe {
                        utf_head_off(rex.line().cast(), rex.input().cast::<c_char>().sub(1))
                    } as usize
                        + 1;
                    // SAFETY: `back` is that character's length.
                    rex.set_input(unsafe { rex.input().sub(back) });
                }
            } else {
                // Non-greedy: take one more match.
                // SAFETY: the caller promises a repeat node, whose operand
                // follows its three-byte header.
                let one_more =
                    unsafe { rst.count != rst.minval && regrepeat(rex, rp.rs_scan.add(3), 1) != 0 };
                if !one_more {
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
        // The byte the pattern needs next is known; positions that cannot
        // supply it are skipped without re-entering the walk.
        if !(rst.nextb == NUL
            || rex.byte() as c_int == rst.nextb
            || rex.byte() as c_int == rst.nextb_ic)
        {
            continue;
        }
        reg_save(rex, &mut rp.rs_saved, backpos.ptr());
        *scan = regnext(rp.rs_scan);
        *status = RA_CONT;
        break;
    }

    if *status != RA_CONT {
        stack.pop_star(scan);
        *status = RA_NOMATCH;
    }
}
