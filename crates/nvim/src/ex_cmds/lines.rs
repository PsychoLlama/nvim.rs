//! `:move` and `:copy` -- relocating a range of lines within the buffer.
//!
//! [`do_move`] is the harder one: it has to move the lines, then fix up every
//! mark, extmark and fold that pointed into either the source or the
//! destination, and it does that by adjusting the ranges rather than replaying
//! the move.  [`ex_copy`] is `:copy`/`:t`, which only ever appends.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{CmdModFlags, FAIL, ML_DEL_MESSAGE, kExtmarkNOOP, kExtmarkUndo};
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::{appended_lines_mark, changed_lines};
use crate::cursor::check_pos;
use crate::ex_docmd::cmdmod_has;
use crate::extmark::extmark_move_region;
use crate::fold::fold_move_range;
use crate::main::{curbuf, curwin, disable_fold_update, global_busy, p_report};
use crate::mark::mark_adjust_nofold;
use crate::memline::{ml_append, ml_delete_flags, ml_find_line_or_offset, ml_get, ml_get_len};
use crate::memory::xfree;
use crate::message::{emsg, msgmore};
use crate::message_fmt::report_msg;
use crate::normal::{visual_active, with_visual_anchor};
use crate::os::cshim::{gettext, ngettext};
use crate::strings::xstrnsave;
use crate::tr_plural;
use crate::types::{OK, OptInt, bcount_t, int64_t, linenr_T, size_t};
use crate::undo::u_save;
use crate::winlayer::Buf;
use crate::winlayer::{Win, tab_windows};
use core::ffi::{c_int, c_ulong};
use core::ptr;

/// `:move` -- move lines `line1`..`line2` to sit after line `dest`.
///
/// Returns `FAIL` for failure, `OK` otherwise.
///
/// # Safety
/// The range and the destination must be lines of the current buffer, or one
/// short of its first line.
pub unsafe fn do_move(line1: linenr_T, line2: linenr_T, dest: linenr_T) -> c_int {
    if dest >= line1 && dest < line2 {
        emsg(gettext(c"E134: Cannot move a range of lines into itself"));
        return FAIL;
    }

    // Do nothing if we are not actually moving any lines.  This will prevent
    // the 'modified' flag from being set without cause.  The cursor still
    // moves as if the lines had, to stay backwards compatible.
    if dest == line1 - 1 || dest == line2 {
        // SAFETY: `curwin` is the live current window.
        cur_win().w_cursor.lnum = last_moved_line(line1, line2, dest);
        return OK;
    }

    // SAFETY: `curbuf` is live and the three line numbers are inside it.  A
    // NULL length is upstream's way of asking only for the byte offset.
    let (start_byte, end_byte, dest_byte) = unsafe {
        (
            ml_find_line_or_offset(curbuf.get(), line1, ptr::null_mut(), true) as bcount_t,
            ml_find_line_or_offset(curbuf.get(), line2 + 1, ptr::null_mut(), true) as bcount_t,
            ml_find_line_or_offset(curbuf.get(), dest + 1, ptr::null_mut(), true) as bcount_t,
        )
    };
    let extent_byte = end_byte - start_byte;
    let num_lines = line2 - line1 + 1;

    // First we copy the old text to its new location -- webb
    // Also copy the flag that ":global" command uses.
    // SAFETY: `dest` is a line of the current buffer, or zero.
    if u_save(dest, dest + 1) == FAIL {
        return FAIL;
    }

    // How many lines the copies added before `line1`.
    let mut extra = 0;
    for l in line1..=line2 {
        // SAFETY: `l + extra` tracks the source line as the copies push it
        // down, and `ml_append` takes ownership of nothing.
        let text = unsafe { xstrnsave(ml_get(l + extra), ml_get_len(l + extra) as size_t) };
        unsafe { ml_append(dest + l - line1, text, 0, false) };
        unsafe { xfree(text.cast()) };
        if dest < line1 {
            extra += 1;
        }
    }

    // Now we must be careful adjusting our marks so that we don't overlap our
    // mark_adjust() calls.
    //
    // We adjust the marks within the old text so that they refer to the
    // last lines of the file (temporarily), because we know no other marks
    // will be set there since these line numbers did not exist until we added
    // our new lines.
    //
    // Then we adjust the marks on lines between the old and new text positions
    // (either forwards or backwards).
    //
    // And Finally we adjust the marks we put at the end of the file back to
    // their final destination at the new text position -- webb

    // The last line in the file now that the copies are in.
    // SAFETY: `curbuf` is live.
    let last_line = cur_buf().b_ml.ml_line_count;
    // SAFETY: as above; the range is the one just copied.
    unsafe { mark_adjust_nofold(line1, line2, last_line - line2, 0, kExtmarkNOOP) };
    folds_frozen(|| {
        // SAFETY: the copies occupy the tail of the buffer.
        unsafe {
            changed_lines(
                Buf::new(curbuf.get()),
                last_line - num_lines + 1,
                0,
                last_line + 1,
                num_lines,
                false,
            )
        };
    });

    let (line_off, byte_off) = if dest >= line2 {
        // SAFETY: the lines the move stepped over are still in the buffer.
        unsafe { mark_adjust_nofold(line2 + 1, dest, -num_lines, 0, kExtmarkNOOP) };
        unsafe { move_folds_in_windows(line1, line2, dest) };
        // SAFETY: `curbuf` is live.
        unsafe { set_op_range(dest - num_lines + 1, dest) };
        (-num_lines, -extent_byte)
    } else {
        // SAFETY: as above.
        unsafe { mark_adjust_nofold(dest + 1, line1 - 1, num_lines, 0, kExtmarkNOOP) };
        unsafe { move_folds_in_windows(dest + 1, line1 - 1, line2) };
        // SAFETY: `curbuf` is live.
        unsafe { set_op_range(dest + 1, dest + num_lines) };
        (0, 0)
    };

    // SAFETY: the tail of the buffer still holds the copies.
    unsafe {
        mark_adjust_nofold(
            last_line - num_lines + 1,
            last_line,
            -(last_line - dest - extra),
            0,
            kExtmarkNOOP,
        )
    };
    folds_frozen(|| {
        // SAFETY: as above.
        unsafe {
            changed_lines(
                Buf::new(curbuf.get()),
                last_line - num_lines + 1,
                0,
                last_line + 1,
                -extra,
                false,
            )
        };
    });

    // Send an update regarding the new lines that were added.
    // SAFETY: `curbuf` is live.
    unsafe { buf_updates_send_changes(curbuf.get(), dest + 1, num_lines as int64_t, 0) };

    // Now we delete the original text -- webb
    // SAFETY: the original range sits at `line1 + extra` now.
    if u_save(line1 + extra - 1, line2 + extra + 1) == FAIL {
        return FAIL;
    }
    for _ in line1..=line2 {
        // SAFETY: as above; each delete pulls the next line into place.
        unsafe { ml_delete_flags(line1 + extra, ML_DEL_MESSAGE as c_int) };
    }

    if global_busy.get() == 0 && num_lines as OptInt > p_report.get() {
        let moved = ngettext(c"%ld line moved", c"%ld lines moved", num_lines as c_ulong);
        let _: bool = report_msg(0, || tr_plural!(moved, num_lines as int64_t));
    }

    // SAFETY: `curbuf` is live and the byte extents were measured before the
    // move; `line_off`/`byte_off` correct the destination for the deletion.
    unsafe {
        extmark_move_region(
            curbuf.get(),
            line1 - 1,
            0,
            start_byte,
            line2 - line1 + 1,
            0,
            extent_byte,
            dest + line_off,
            0,
            dest_byte + byte_off,
            kExtmarkUndo,
        )
    };

    // Leave the cursor on the last of the moved lines.
    // SAFETY: `curwin` is the live current window.
    cur_win().w_cursor.lnum = last_moved_line(line1, line2, dest);

    // SAFETY: `curbuf` is live; the redrawn span reaches from the first line
    // that moved to the last, whichever direction the move went.
    if line1 < dest {
        let end = (dest + num_lines + 1).min(cur_buf().b_ml.ml_line_count + 1);
        changed_lines(cur_buf(), line1, 0, end, 0, false);
    } else {
        changed_lines(cur_buf(), dest + 1, 0, line1 + num_lines, 0, false);
    }
    // Send nvim_buf_lines_event regarding lines that were deleted.
    unsafe { buf_updates_send_changes(curbuf.get(), line1 + extra, 0, num_lines as int64_t) };

    OK
}

/// Where `:move` leaves the cursor: on the last line it moved.
fn last_moved_line(line1: linenr_T, line2: linenr_T, dest: linenr_T) -> linenr_T {
    if dest >= line1 {
        dest
    } else {
        dest + (line2 - line1) + 1
    }
}

/// Run `f` with the fold update held off.
///
/// `:move` repairs the folds itself with `move_folds_in_windows`, so the
/// `changed_lines` calls that only shuffle line numbers past each other must
/// not have a fold update run over the half-moved buffer.
fn folds_frozen<R>(f: impl FnOnce() -> R) -> R {
    disable_fold_update.set(disable_fold_update.get() + 1);
    let result = f();
    disable_fold_update.set(disable_fold_update.get() - 1);
    result
}

/// Move the folds of `line1`..`line2` to `dest` in every window showing the
/// current buffer -- a window on another tab page holds folds of its own.
///
/// # Safety
/// The three line numbers must be a `:move` range of the current
/// buffer.
unsafe fn move_folds_in_windows(line1: linenr_T, line2: linenr_T, dest: linenr_T) {
    for wp in tab_windows().map(Win::raw) {
        // SAFETY: `wp` is a live window.
        if unsafe { (*wp).w_buffer } == curbuf.get() {
            unsafe { fold_move_range(&raw mut (*wp).w_folds, line1, line2, dest) };
        }
    }
}

/// Set the `'[` and `']` marks around what the command touched, unless
/// `:lockmarks` asked for them to be left alone.
///
/// # Safety
/// The current buffer must be live.
pub(super) unsafe fn set_op_range(start: linenr_T, end: linenr_T) {
    if cmdmod_has(CmdModFlags::LOCKMARKS) {
        return;
    }
    // SAFETY: caller's contract.  `coladd` is deliberately left alone, as
    // upstream leaves it.
    cur_buf().b_op_start.lnum = start;
    cur_buf().b_op_start.col = 0;
    cur_buf().b_op_end.lnum = end;
    cur_buf().b_op_end.col = 0;
}

/// `:copy` and `:t` -- copy lines `line1`..`line2` to below line `n`.
///
/// # Safety
/// The range and the destination must be lines of the current buffer, or one
/// short of its first line.
pub unsafe fn ex_copy(mut line1: linenr_T, mut line2: linenr_T, n: linenr_T) {
    let count = line2 - line1 + 1;
    // SAFETY: `curbuf` is live.
    unsafe { set_op_range(n + 1, n + count) };

    // There are three situations:
    //   1. destination is above line1
    //   2. destination is between line1 and line2
    //   3. destination is below line2
    //
    // n = destination (when starting)
    // curwin->w_cursor.lnum = destination (while copying)
    // line1 = start of source (while copying)
    // line2 = end of source (while copying)
    // SAFETY: `n` is a line of the current buffer, or zero.
    if u_save(n, n + 1) == FAIL {
        return;
    }

    // SAFETY: `curwin` is the live current window.
    cur_win().w_cursor.lnum = n;
    while line1 <= line2 {
        // Need to make a copy because the line will be unlocked within
        // `ml_append`.
        // SAFETY: `line1` is a line of the current buffer throughout.
        let cursor = unsafe {
            let text = xstrnsave(ml_get(line1), ml_get_len(line1) as size_t);
            ml_append(cur_win().w_cursor.lnum, text, 0, false);
            xfree(text.cast());
            &mut (*curwin.get()).w_cursor
        };

        // Situation 2: skip the lines already copied.
        if line1 == n {
            line1 = cursor.lnum;
        }
        line1 += 1;
        if cursor.lnum < line1 {
            line1 += 1;
        }
        if cursor.lnum < line2 {
            line2 += 1;
        }
        cursor.lnum += 1;
    }

    // SAFETY: `count` lines were appended after `n`.
    unsafe { appended_lines_mark(n, count) };
    if visual_active() {
        // SAFETY: `curbuf` is live and `VIsual` is a global position.
        with_visual_anchor(|anchor| unsafe { check_pos(Buf::current(), anchor) });
    }
    // SAFETY: message state, main thread.
    unsafe { msgmore(count) };
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
