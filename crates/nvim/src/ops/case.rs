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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ex_docmd::cmdmod_has;
use crate::message_fmt::report_msg;
use crate::tr_plural;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;

/// `g~`, `gu`, `gU`, `g?` over the operator's region.
///
/// # Safety
/// `op` must point to a live `OpArg` describing a region of the current
/// buffer.
pub(crate) unsafe fn op_tilde(op: *mut OpArg) {
    // SAFETY: the caller's promise -- a live `OpArg` of the current buffer.
    // `pos` walks that region, so it names a position of the buffer at every
    // step, which is what `swapchars`, `inc` and `ml_get_pos_len` ask for.
    let mut op = unsafe { Op::new(op) };
    let mut did_change = false;

    let (above, below) = (op.start.lnum - 1, op.end.lnum + 1);
    if u_save(above, below).is_err() {
        return;
    }

    let mut pos: Pos = op.start;
    if op.motion_type == kMTBlockWise {
        let mut bd = BlockDef::ZERO;
        while pos.lnum <= op.end.lnum {
            unsafe { block_prep(op.raw(), &raw mut bd, pos.lnum, false) };
            pos.col = bd.textcol;
            did_change |= unsafe { swapchars(op.op_type, &raw mut pos, bd.textlen) } != 0;
            pos.lnum += 1;
        }
        if did_change {
            let (first, last) = (op.start.lnum, op.end.lnum + 1);
            changed_lines(Buf::current(), first, 0, last, 0, true);
        }
    } else {
        if op.motion_type == kMTLineWise {
            op.start.col = 0;
            pos.col = 0;
            op.end.col = ml_get_len(op.end.lnum);
            if op.end.col != 0 {
                op.end.col -= 1;
            }
        } else if !op.inclusive {
            unsafe { dec(&mut op.end) };
        }

        if pos.lnum == op.end.lnum {
            let len = op.end.col - pos.col + 1;
            did_change = unsafe { swapchars(op.op_type, &raw mut pos, len) } != 0;
        } else {
            loop {
                let len = if pos.lnum == op.end.lnum {
                    op.end.col + 1
                } else {
                    unsafe { ml_get_pos_len(&raw mut pos) }
                };
                did_change |= unsafe { swapchars(op.op_type, &raw mut pos, len) } != 0;
                // `inc` answers -1 at the end of the buffer; either exit
                // leaves `pos` where the walk stopped.
                if ltoreq(op.end, pos) || unsafe { inc(&mut pos) } == -1 {
                    break;
                }
            }
        }
        if did_change {
            let (first, col, last) = (op.start.lnum, op.start.col, op.end.lnum + 1);
            changed_lines(Buf::current(), first, col, last, 0, true);
        }
    }

    if !did_change && op.is_visual {
        // No change: the Visual selection still has to come off the screen.
        redraw_curbuf_later(UPD_INVERTED);
    }

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = op.start;
        Buf::current().b_op_end = op.end;
    }

    if OptInt::from(op.line_count) > p_report.get() {
        let n = ::core::ffi::c_ulong::try_from(op.line_count)
            .expect("an operator's line count is never negative");
        let fmt = ngettext(c"%ld line changed", c"%ld lines changed", n);
        let _: bool = report_msg(0, || tr_plural!(fmt, int64_t::from(op.line_count)));
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
unsafe fn swapchars(op_type: OpType, pos: *mut Pos, length: c_int) -> c_int {
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
pub unsafe fn swapchar(op_type: OpType, pos: *mut Pos) -> bool {
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
        let saved: Pos = Win::current().w_cursor;
        Win::current().w_cursor = unsafe { *pos };
        let _ = unsafe { del_bytes(utf_ptr2len(get_cursor_pos_ptr()), false, false) };
        unsafe { ins_char(nc) };
        Win::current().w_cursor = saved;
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
