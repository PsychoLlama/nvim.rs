//! Sending a run of screen cells.
//!
//! This is the hot path: a full redraw is thousands of calls here and
//! almost nothing else, so `grid_line` is packed straight into the buffer
//! instead of being built as an [`Array`](crate::types::Array)
//! first. The wire form is
//!
//! ```text
//! grid_line(grid, row, startcol, [cell, cell, ...], wrap)
//! ```
//!
//! where a cell is `[text]`, `[text, hl]` or `[text, hl, repeat]` — the
//! shorter forms mean "same highlight as the previous cell" and "once".
//! Collapsing runs that way is most of the compression the protocol has,
//! and it is why the loop below only emits a cell when it sees the next one
//! differ.
//!
//! A run can outlast the buffer. When it does the current `grid_line` is
//! closed and a new one opened at the column reached so far, which is what
//! the mid-loop flush does; the reserve [`prepare_call`] keeps is not
//! enough for an unbounded number of cells, so this is the one caller that
//! checks for space itself.
//!
//! The legacy branch is the pre-`ext_linegrid` protocol: no runs, no
//! highlight table, one cell per `put` at a cursor the server moves
//! explicitly. See [`events`](super::events).

#![deny(unsafe_op_in_unsafe_fn)]

use super::Ui;
use super::events::{linegrid, remote_ui_cursor_goto, remote_ui_highlight_set, remote_ui_put};
use super::packer::{MAX_CELLS_PENDING, UI_BUF_SIZE, prepare_call, push_call, ui_flush_buf};
use crate::grid::{schar_get, schar_get_adv};
use crate::main::Columns;
use crate::mbyte::utf_ambiguous_width;
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_array_dyn16, mpack_be16, mpack_bool, mpack_str_small, mpack_uint,
};
use crate::types::builders::ArrayBuf;
use crate::types::ui::kLineFlagWrap;
use crate::types::{Integer, LineFlags, RemoteUI, sattr_T, schar_T};
use core::ffi::{c_char, c_int};

/// The largest a single cell can pack to: a fixarray header, the text with
/// its header, the highlight id and the repeat count. Two of those plus the
/// closing `wrap` byte is what has to be free before a cell is emitted.
const MAX_CELL_SIZE: usize = 1 + 2 + MAX_SCHAR_SIZE + 5 + 5;

/// The longest a single cell's text can be, as [`schar_get`] writes it.
const MAX_SCHAR_SIZE: usize = 32;

/// The event this module packs. Named once, because the mid-run flush has
/// to reopen the same event.
const GRID_LINE: &core::ffi::CStr = c"grid_line";

/// Sends cells `startcol..endcol` of `row`, plus the clearing that runs on
/// to `clearcol`.
///
/// # Safety
///
/// `ui` must be live and `chunk`/`attrs` must each have at least
/// `endcol - startcol` readable elements.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub unsafe fn remote_ui_raw_line(
    ui: *mut RemoteUI,
    grid: Integer,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    flags: LineFlags,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    // SAFETY: the caller's promise, passed on unchanged.
    if unsafe { linegrid(ui) } {
        // SAFETY: as above.
        unsafe {
            raw_line_linegrid(
                ui, grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
            );
        }
    } else {
        // SAFETY: as above.
        unsafe { raw_line_legacy(ui, row, startcol, endcol, clearcol, clearattr, chunk, attrs) };
    }
}

/// [`remote_ui_raw_line`] for a UI on the modern protocol.
///
/// # Safety
///
/// As [`remote_ui_raw_line`].
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
unsafe fn raw_line_linegrid(
    ui: *mut RemoteUI,
    grid: Integer,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    flags: LineFlags,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    // SAFETY: the caller's promise -- `ui` is live.
    let mut live = unsafe { Ui::new(ui) };
    // SAFETY: as above.
    unsafe { prepare_call(ui, GRID_LINE) };
    // SAFETY: `prepare_call` left a block with room for the header.
    let mut lenpos = unsafe { open_line(ui, grid, row, startcol) };

    let ncells = (endcol - startcol) as usize;
    // Cells since the last one emitted, i.e. the length of the run in
    // progress including the cell being looked at.
    let mut repeat: u32 = 0;
    // Elements written to the current `grid_line`, for its back-patched
    // array header.
    let mut nelem: u32 = 0;
    // The highlight the previous cell was emitted with, so that an
    // unchanged one can be left off. Reset on every new `grid_line`,
    // because the reader's memory of it resets too.
    let mut last_hl: c_int = -1;
    // Whether the last cell emitted was a space, which decides whether
    // the trailing clear can be merged into it.
    let mut was_space = false;

    for i in 0..ncells {
        repeat += 1;
        // SAFETY: the caller's promise -- both arrays hold `ncells`.
        let (hl, text) = unsafe { (*attrs.add(i), *chunk.add(i)) };
        let last = i == ncells - 1;
        // SAFETY: as above; `i + 1` is still one of them.
        let run_continues =
            !last && unsafe { *attrs.add(i + 1) == hl && *chunk.add(i + 1) == text };
        if run_continues {
            continue;
        }

        // Saturating because an overfull buffer must flush rather
        // than wrap to "plenty of room", which is what C's unsigned
        // subtraction would have done.
        let used = live.packer.ptr.addr() - live.packer.startptr.addr();
        if UI_BUF_SIZE.saturating_sub(used) < 2 * MAX_CELL_SIZE + 1
            || live.ncells_pending >= MAX_CELLS_PENDING
        {
            // Out of room mid-run. Close this `grid_line` and open
            // another at the column reached, so that the reader sees
            // two complete events rather than one truncated one. A
            // trailing space is re-sent as a clear first, because the
            // reader's "rest of the line is this cell" shorthand cannot
            // span two events.
            if was_space {
                nelem += 1;
                live.ncells_pending += 1;
                // SAFETY: the reserve covers one more cell.
                unsafe { push_clear(ui, clearattr, 0) };
            }
            // `lenpos` is this event's placeholder, in the block still
            // being packed into.
            mpack_be16(&mut lenpos, nelem);
            mpack_bool(&mut live.packer.ptr, false);
            // SAFETY: `ui` is live.
            unsafe { ui_flush_buf(ui, false) };
            // SAFETY: as above.
            unsafe { prepare_call(ui, GRID_LINE) };
            let reached = startcol + (i - (repeat as usize - 1)) as Integer;
            // SAFETY: `prepare_call` left a block with room for the header.
            lenpos = unsafe { open_line(ui, grid, row, reached) };
            nelem = 0;
            last_hl = -1;
        }

        let fields: u32 = if repeat > 1 {
            3
        } else if hl != last_hl as sattr_T {
            2
        } else {
            1
        };
        nelem += 1;
        // The check above left room for a whole cell.
        mpack_array(&mut live.packer.ptr, fields);
        // The text's length is only known once it is written, so the
        // fixstr header goes down first and is patched in place.
        let size_byte = live.packer.ptr;
        // SAFETY: as above -- the reserve covers the header and the text.
        unsafe {
            live.packer.ptr = live.packer.ptr.add(1);
            let len = schar_get_adv(&raw mut live.packer.ptr, text);
            *size_byte = (0xa0 | len) as c_char;
        }
        if fields >= 2 {
            mpack_uint(&mut live.packer.ptr, hl as u32);
            if fields >= 3 {
                mpack_uint(&mut live.packer.ptr, repeat);
            }
        }

        // A repeat counts as two cells however long it is: what the
        // budget is really measuring is how much the reader has to
        // draw before it can show something.
        live.ncells_pending += repeat.min(2) as usize;
        last_hl = hl as c_int;
        repeat = 0;
        was_space = text == b' ' as schar_T;
    }

    if endcol < clearcol || was_space {
        // Clearing to `clearcol` is one cell with a repeat. A trailing
        // space is folded into it so that the reader does not have to
        // decide whether the run continues.
        nelem += 1;
        live.ncells_pending += 1;
        // SAFETY: the reserve covers one more cell.
        unsafe { push_clear(ui, clearattr, (clearcol - endcol) as u32) };
    }
    // `lenpos` is the last event's placeholder, and the reserve covers the
    // closing byte.
    mpack_be16(&mut lenpos, nelem);
    mpack_bool(&mut live.packer.ptr, flags & kLineFlagWrap != 0);
}

/// Writes a `grid_line` argument list up to its cell array, returning where
/// that array's length has to be patched in.
///
/// # Safety
///
/// `ui` must be live with a buffer that has room for the header.
unsafe fn open_line(
    ui: *mut RemoteUI,
    grid: Integer,
    row: Integer,
    startcol: Integer,
) -> *mut c_char {
    // SAFETY: the caller's promise -- `ui` is live with room for this.
    let mut live = unsafe { Ui::new(ui) };
    mpack_array(&mut live.packer.ptr, 5);
    for value in [grid, row, startcol] {
        mpack_uint(&mut live.packer.ptr, value as u32);
    }
    mpack_array_dyn16(&mut live.packer.ptr)
}

/// Writes a cell that clears `repeat` columns with `attr`.
///
/// # Safety
///
/// `ui` must be live with a buffer that has room for the cell.
unsafe fn push_clear(ui: *mut RemoteUI, attr: Integer, repeat: u32) {
    // SAFETY: the caller's promise -- `ui` is live with room for this.
    let mut live = unsafe { Ui::new(ui) };
    mpack_array(&mut live.packer.ptr, 3);
    mpack_str_small(&mut live.packer.ptr, b" ");
    mpack_uint(&mut live.packer.ptr, attr as u32);
    mpack_uint(&mut live.packer.ptr, repeat);
}

/// [`remote_ui_raw_line`] for a UI on the pre-`ext_linegrid` protocol.
///
/// # Safety
///
/// As [`remote_ui_raw_line`].
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
unsafe fn raw_line_legacy(
    ui: *mut RemoteUI,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    // SAFETY: the caller's promise -- `ui` is live.
    let mut live = unsafe { Ui::new(ui) };
    for i in 0..(endcol - startcol) {
        // SAFETY: the caller's promise -- both arrays hold this many.
        let (hl, text) = unsafe { (*attrs.offset(i as isize), *chunk.offset(i as isize)) };
        let mut cell = [0 as c_char; MAX_SCHAR_SIZE];
        // SAFETY: `cell` is `MAX_SCHAR_SIZE` bytes, which is what
        // `schar_get` writes at most.
        unsafe { schar_get(cell.as_mut_ptr(), text) };
        // SAFETY: `ui` is live, and `cell` is NUL-terminated by the write
        // above.
        unsafe {
            remote_ui_cursor_goto(ui, row, startcol + i);
            remote_ui_highlight_set(ui, hl as c_int);
            remote_ui_put(ui, cell.as_ptr());
        }
        // SAFETY: as above.
        if unsafe { utf_ambiguous_width(cell.as_ptr()) } {
            // The UI and the server disagree about how wide that cell
            // was drawn, so the tracked column is no longer usable and
            // the next cell must move the cursor explicitly.
            live.client_col = -1;
        }
    }
    if endcol >= clearcol {
        return;
    }
    // SAFETY: `ui` is live.
    unsafe {
        remote_ui_cursor_goto(ui, row, endcol);
        remote_ui_highlight_set(ui, clearattr as c_int);
    }
    if clearattr == 0 && clearcol == Integer::from(Columns.get()) {
        // Clearing the rest of the line with the default highlight has
        // its own call; anything else has to be spelled out cell by
        // cell, because the legacy protocol has no repeat count.
        let mut args = ArrayBuf::<0>::new();
        // SAFETY: as above; the empty array borrows this frame's buffer.
        unsafe { push_call(ui, c"eol_clear", args.array()) };
    } else {
        for _ in endcol..clearcol {
            // SAFETY: as above.
            unsafe { remote_ui_put(ui, c" ".as_ptr()) };
        }
    }
}
