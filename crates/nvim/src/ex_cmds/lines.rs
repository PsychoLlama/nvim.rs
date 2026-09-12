//! `:move` and `:copy` -- relocating a range of lines within the buffer.
//!
//! [`do_move`] is the harder one: it has to move the lines, then fix up every
//! mark, extmark and fold that pointed into either the source or the
//! destination, and it does that by adjusting the ranges rather than replaying
//! the move.  [`ex_copy`] is `:copy`/`:t`, which only ever appends.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::LineCopy;
use super::say;
use super::{CmdModFlags, ML_DEL_MESSAGE, kExtmarkNOOP, kExtmarkUndo};
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::{appended_lines_mark, changed_lines};
use crate::cursor::check_pos;
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::state::global_busy;
use crate::extmark::extmark_move_region;
use crate::fold::fold_move_range;
use crate::guard::Suppress;
use crate::mark::mark_adjust_nofold;
use crate::memline::{ml_append, ml_delete_flags, ml_find_line_or_offset};
use crate::message::emsg;
use crate::message_fmt::report_msg;
use crate::normal::{visual_active, with_visual_anchor};
use crate::option::vars::p_report;
use crate::os::cshim::{gettext, ngettext};
use crate::tr_plural;
use crate::types::{BCount, Failed, LineNr, OptInt, int64_t};
use crate::undo::u_save;
use crate::winlayer::Buf;
use crate::winlayer::{Win, tab_windows};
use core::ffi::{c_int, c_ulong};
use core::ptr;

/// `:move` -- move lines `line1`..`line2` to sit after line `dest`.
///
/// Answers `Err` for failure.
pub fn do_move(line1: LineNr, line2: LineNr, dest: LineNr) -> Result<(), Failed> {
    if dest >= line1 && dest < line2 {
        emsg(gettext(c"E134: Cannot move a range of lines into itself"));
        return Err(Failed);
    }

    // Do nothing if we are not actually moving any lines.  This will prevent
    // the 'modified' flag from being set without cause.  The cursor still
    // moves as if the lines had, to stay backwards compatible.
    if dest == line1 - 1 || dest == line2 {
        // SAFETY: `curwin` is the live current window.
        Win::current().w_cursor.lnum = last_moved_line(line1, line2, dest);
        return Ok(());
    }

    // SAFETY: `curbuf` is live and the three line numbers are inside it.  A
    // NULL length is upstream's way of asking only for the byte offset.
    let (start_byte, end_byte, dest_byte) = unsafe {
        (
            ml_find_line_or_offset(Buf::current(), line1, ptr::null_mut(), true) as BCount,
            ml_find_line_or_offset(Buf::current(), line2 + 1, ptr::null_mut(), true) as BCount,
            ml_find_line_or_offset(Buf::current(), dest + 1, ptr::null_mut(), true) as BCount,
        )
    };
    let extent_byte = end_byte - start_byte;
    let num_lines = line2 - line1 + 1;

    // First we copy the old text to its new location -- webb
    // Also copy the flag that ":global" command uses.
    // SAFETY: `dest` is a line of the current buffer, or zero.
    u_save(dest, dest + 1)?;

    // How many lines the copies added before `line1`.
    let mut extra = 0;
    let mut copy = LineCopy::new();
    for l in line1..=line2 {
        copy.fill_line(l + extra);
        let _ = unsafe { ml_append(dest + l - line1, copy.as_ptr(), 0, false) };
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
    let last_line = Buf::current().b_ml.ml_line_count;
    mark_adjust_nofold(line1, line2, last_line - line2, 0, kExtmarkNOOP);
    folds_frozen(|| {
        changed_lines(
            Buf::current(),
            last_line - num_lines + 1,
            0,
            last_line + 1,
            num_lines,
            false,
        );
    });

    let (line_off, byte_off) = if dest >= line2 {
        mark_adjust_nofold(line2 + 1, dest, -num_lines, 0, kExtmarkNOOP);
        move_folds_in_windows(line1, line2, dest);
        set_op_range(dest - num_lines + 1, dest);
        (-num_lines, -extent_byte)
    } else {
        mark_adjust_nofold(dest + 1, line1 - 1, num_lines, 0, kExtmarkNOOP);
        move_folds_in_windows(dest + 1, line1 - 1, line2);
        set_op_range(dest + 1, dest + num_lines);
        (0, 0)
    };

    mark_adjust_nofold(
        last_line - num_lines + 1,
        last_line,
        -(last_line - dest - extra),
        0,
        kExtmarkNOOP,
    );
    folds_frozen(|| {
        changed_lines(
            Buf::current(),
            last_line - num_lines + 1,
            0,
            last_line + 1,
            -extra,
            false,
        );
    });

    // Send an update regarding the new lines that were added.
    buf_updates_send_changes(Buf::current(), dest + 1, num_lines as int64_t, 0);

    // Now we delete the original text -- webb
    // SAFETY: the original range sits at `line1 + extra` now.
    u_save(line1 + extra - 1, line2 + extra + 1)?;
    for _ in line1..=line2 {
        let _ = ml_delete_flags(line1 + extra, ML_DEL_MESSAGE as c_int);
    }

    if global_busy.get() == 0 && num_lines as OptInt > p_report.get() {
        let moved = ngettext(c"%ld line moved", c"%ld lines moved", num_lines as c_ulong);
        let _: bool = report_msg(0, || tr_plural!(moved, num_lines as int64_t));
    }

    extmark_move_region(
        Buf::current(),
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
    );

    // Leave the cursor on the last of the moved lines.
    // SAFETY: `curwin` is the live current window.
    Win::current().w_cursor.lnum = last_moved_line(line1, line2, dest);

    // SAFETY: `curbuf` is live; the redrawn span reaches from the first line
    // that moved to the last, whichever direction the move went.
    if line1 < dest {
        let end = (dest + num_lines + 1).min(Buf::current().b_ml.ml_line_count + 1);
        changed_lines(Buf::current(), line1, 0, end, 0, false);
    } else {
        changed_lines(Buf::current(), dest + 1, 0, line1 + num_lines, 0, false);
    }
    // Send nvim_buf_lines_event regarding lines that were deleted.
    buf_updates_send_changes(Buf::current(), line1 + extra, 0, num_lines as int64_t);

    Ok(())
}

/// Where `:move` leaves the cursor: on the last line it moved.
fn last_moved_line(line1: LineNr, line2: LineNr, dest: LineNr) -> LineNr {
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
    let _frozen = Suppress::fold_update();
    f()
}

/// Move the folds of `line1`..`line2` to `dest` in every window showing the
/// current buffer -- a window on another tab page holds folds of its own.
fn move_folds_in_windows(line1: LineNr, line2: LineNr, dest: LineNr) {
    for wp in tab_windows().map(Win::raw) {
        // SAFETY: `wp` is a live window.
        if unsafe { (*wp).w_buffer } == Buf::current_raw() {
            unsafe { fold_move_range(&raw mut (*wp).w_folds, line1, line2, dest) };
        }
    }
}

/// Set the `'[` and `']` marks around what the command touched, unless
/// `:lockmarks` asked for them to be left alone.
pub(super) fn set_op_range(start: LineNr, end: LineNr) {
    if cmdmod_has(CmdModFlags::LOCKMARKS) {
        return;
    }
    Buf::current().b_op_start.lnum = start;
    Buf::current().b_op_start.col = 0;
    Buf::current().b_op_end.lnum = end;
    Buf::current().b_op_end.col = 0;
}

/// `:copy` and `:t` -- copy lines `line1`..`line2` to below line `n`.
pub fn ex_copy(mut line1: LineNr, mut line2: LineNr, n: LineNr) {
    let count = line2 - line1 + 1;
    set_op_range(n + 1, n + count);

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
    if u_save(n, n + 1).is_err() {
        return;
    }

    Win::current().w_cursor.lnum = n;
    let mut copy = LineCopy::new();
    while line1 <= line2 {
        // Need to make a copy because the line will be unlocked within
        // `ml_append`.
        copy.fill_line(line1);
        let at = Win::current().w_cursor.lnum;
        // SAFETY: the text is this call's own NUL-terminated copy.
        let _ = unsafe { ml_append(at, copy.as_ptr(), 0, false) };
        let cursor = &mut Win::current().w_cursor;

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

    appended_lines_mark(n, count);
    if visual_active() {
        with_visual_anchor(|anchor| check_pos(Buf::current(), anchor));
    }
    // SAFETY: message state, main thread.
    say::more(count);
}
