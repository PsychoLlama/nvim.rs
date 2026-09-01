//! `g~`, `gu`, `gU` and `g?` -- rewriting the characters in place.
//!
//! [`op_tilde`] walks the region and hands each position to [`swapchar`],
//! which is the whole of the per-character decision: which of the four
//! operators is running, whether the character has a case at all, and -- for
//! rot13 -- that only ASCII letters move.
//!
//! The awkward part is that a case change can change a character's *byte
//! length*: `İ` is two bytes and its lower case is one, so the walk cannot
//! simply overwrite. [`swapchar`] therefore has two writers -- a byte poke
//! through `pbyte` when both the old and the new character are ASCII, and a
//! delete-then-insert through the change layer otherwise -- and [`swapchars`]
//! re-measures the character under `pos` on every step rather than trusting
//! the length it was given.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ex_docmd::cmdmod_has;
use crate::message_fmt::report_msg;
use crate::tr_plural;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;

/// `g~`, `gu`, `gU`, `g?` over the operator's region.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub(crate) unsafe fn op_tilde(oap: *mut oparg_T) {
    // SAFETY: the caller's promise -- a live `oparg_T` of the current buffer.
    // `pos` walks that region, so it names a position of the buffer at every
    // step, which is what `swapchars`, `inc` and `ml_get_pos_len` ask for.
    let mut oap = unsafe { Op::new(oap) };
    let mut did_change = false;

    let (above, below) = (oap.start.lnum - 1, oap.end.lnum + 1);
    if u_save(above, below).is_err() {
        return;
    }

    let mut pos: pos_T = oap.start;
    if oap.motion_type == kMTBlockWise {
        let mut bd = block_def::ZERO;
        while pos.lnum <= oap.end.lnum {
            unsafe { block_prep(oap.raw(), &raw mut bd, pos.lnum, false) };
            pos.col = bd.textcol;
            did_change |= unsafe { swapchars(oap.op_type, &raw mut pos, bd.textlen) } != 0;
            pos.lnum += 1;
        }
        if did_change {
            let (first, last) = (oap.start.lnum, oap.end.lnum + 1);
            changed_lines(cur_buf(), first, 0, last, 0, true);
        }
    } else {
        if oap.motion_type == kMTLineWise {
            oap.start.col = 0;
            pos.col = 0;
            oap.end.col = ml_get_len(oap.end.lnum);
            if oap.end.col != 0 {
                oap.end.col -= 1;
            }
        } else if !oap.inclusive {
            unsafe { dec(&mut oap.end) };
        }

        if pos.lnum == oap.end.lnum {
            let len = oap.end.col - pos.col + 1;
            did_change = unsafe { swapchars(oap.op_type, &raw mut pos, len) } != 0;
        } else {
            loop {
                let len = if pos.lnum == oap.end.lnum {
                    oap.end.col + 1
                } else {
                    unsafe { ml_get_pos_len(&raw mut pos) }
                };
                did_change |= unsafe { swapchars(oap.op_type, &raw mut pos, len) } != 0;
                // `inc` answers -1 at the end of the buffer; either exit
                // leaves `pos` where the walk stopped.
                if ltoreq(oap.end, pos) || unsafe { inc(&mut pos) } == -1 {
                    break;
                }
            }
        }
        if did_change {
            let (first, col, last) = (oap.start.lnum, oap.start.col, oap.end.lnum + 1);
            changed_lines(cur_buf(), first, col, last, 0, true);
        }
    }

    if !did_change && oap.is_VIsual {
        // No change: the Visual selection still has to come off the screen.
        redraw_curbuf_later(UPD_INVERTED);
    }

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        cur_buf().b_op_start = oap.start;
        cur_buf().b_op_end = oap.end;
    }

    if oap.line_count as OptInt > p_report.get() {
        let n = oap.line_count as ::core::ffi::c_ulong;
        let fmt = ngettext(c"%ld line changed", c"%ld lines changed", n);
        let _: bool = report_msg(0, || tr_plural!(fmt, oap.line_count as int64_t));
    }
}

/// [`swapchar`] over `length` *bytes* from `pos`, which is left just after the
/// last character touched.
///
/// `length` is rounded up to a whole character: a multi-byte character
/// straddling the end is changed entirely. Because a change can alter the
/// character's byte length, the loop re-measures at `pos` each time rather
/// than stepping by what it was told.
///
/// # Safety
/// `pos` must point to a valid position in the current buffer.
unsafe fn swapchars(op_type: OpType, pos: *mut pos_T, length: c_int) -> c_int {
    // SAFETY: the caller's promise -- `pos` names a position of the current
    // buffer, and `inc` keeps it one until it answers -1.
    let mut did_change: c_int = 0;
    let mut todo = length;
    while todo > 0 {
        // We are counting bytes, not characters.
        let len = unsafe { utfc_ptr2len(ml_get_pos(pos)) };
        if len > 0 {
            todo -= len - 1;
        }
        did_change |= c_int::from(unsafe { swapchar(op_type, pos) });
        if unsafe { inc(&mut *pos) } == -1 {
            // At the end of the buffer; do not run the decrement.
            break;
        }
        todo -= 1;
    }
    did_change
}

/// Apply one case operator to the character at `pos`; `true` if it changed.
///
/// `op_type` is `OP_UPPER`, `OP_LOWER`, `OP_ROT13`, or anything else for
/// "swap the case".
///
/// # Safety
/// `pos` must point to a valid position in the current buffer.
pub unsafe fn swapchar(op_type: OpType, pos: *mut pos_T) -> bool {
    // SAFETY: the caller's promise -- `pos` names a position of the current
    // buffer, so the cursor may be put on it and the character rebuilt there.
    let c = unsafe { gchar_pos(pos) };

    // Only rot13 ASCII.
    if c >= 0x80 && op_type == OpType::Rot13 {
        return false;
    }

    let mut nc = c;
    if mb_islower(c) {
        if op_type == OpType::Rot13 {
            nc = rot13(c, 'a' as c_int);
        } else if op_type != OpType::Lower {
            nc = mb_toupper(c);
        }
    } else if mb_isupper(c) {
        if op_type == OpType::Rot13 {
            nc = rot13(c, 'A' as c_int);
        } else if op_type != OpType::Upper {
            nc = mb_tolower(c);
        }
    }
    if nc == c {
        return false;
    }

    if c >= 0x80 || nc >= 0x80 {
        // The byte length can differ, so rebuild the character through the
        // change layer. Not `del_char()`: that would take the composing
        // characters with it.
        let saved: pos_T = cur_win().w_cursor;
        cur_win().w_cursor = unsafe { *pos };
        let _ = unsafe { del_bytes(utf_ptr2len(get_cursor_pos_ptr()), false, false) };
        unsafe { ins_char(nc) };
        cur_win().w_cursor = saved;
    } else {
        unsafe { pbyte(*pos, nc) };
    }
    true
}

/// Upstream's `ROT13(c, a)`: rotate `c` by 13 within the 26 letters starting
/// at `a`.
fn rot13(c: c_int, a: c_int) -> c_int {
    (c - a + 13) % 26 + a
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
