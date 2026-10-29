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

#![forbid(unsafe_code)]

use super::events::{linegrid, remote_ui_cursor_goto, remote_ui_highlight_set, remote_ui_put};
use super::packer::{MAX_CELLS_PENDING, UI_BUF_SIZE, prepare_call, push_call, ui_flush_buf};
use crate::grid::{SCHAR_BUF_SIZE, SCharBuf, schar_copy, schar_get_adv};
use crate::mbyte::ambiguous_width_at;
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_array_dyn16, mpack_bool, mpack_str_small, mpack_uint,
};
use crate::types::builders::ArrayBuf;
use crate::types::ui::kLineFlagWrap;
use crate::types::{Integer, LineFlags, RemoteUI, ScreenAttr, ScreenChar};
use crate::ui::state::Columns;
use core::ffi::c_int;

/// The largest a single cell can pack to: a fixarray header, the text with
/// its header, the highlight id and the repeat count. Two of those plus the
/// closing `wrap` byte is what has to be free before a cell is emitted.
const MAX_CELL_SIZE: usize = 1 + 2 + MAX_SCHAR_SIZE + 5 + 5;

/// The longest a single cell's text can be, as [`schar_get_adv`] writes it.
const MAX_SCHAR_SIZE: usize = 32;

/// The event this module packs. Named once, because the mid-run flush has
/// to reopen the same event.
const GRID_LINE: &core::ffi::CStr = c"grid_line";

/// Sends cells `startcol..endcol` of `row`, plus the clearing that runs on
/// to `clearcol`. `chunk` and `attrs` hold the drawn cells, at least
/// `endcol - startcol` of each; anything past that is not read.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub fn remote_ui_raw_line(
    ui: &mut RemoteUI,
    grid: Integer,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    flags: LineFlags,
    chunk: &[ScreenChar],
    attrs: &[ScreenAttr],
) {
    let ncells = usize::try_from(endcol - startcol).unwrap_or(0);
    let cells = &chunk[..ncells];
    let attrs = &attrs[..ncells];
    if linegrid(ui) {
        let span = (row, startcol, endcol, clearcol);
        raw_line_linegrid(ui, grid, span, clearattr, flags, cells, attrs);
    } else {
        raw_line_legacy(ui, row, startcol, endcol, clearcol, clearattr, cells, attrs);
    }
}

/// [`remote_ui_raw_line`] for a UI on the modern protocol. `span` is the
/// row, the start and end columns and the clear column; `cells` and
/// `attrs` are exactly the drawn cells.
fn raw_line_linegrid(
    ui: &mut RemoteUI,
    grid: Integer,
    span: (Integer, Integer, Integer, Integer),
    clearattr: Integer,
    flags: LineFlags,
    cells: &[ScreenChar],
    attrs: &[ScreenAttr],
) {
    let (row, startcol, endcol, clearcol) = span;
    prepare_call(ui, GRID_LINE);
    // `prepare_call` left a block with room for the header.
    let mut lenpos = open_line(ui, grid, row, startcol);

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

    let mut drawn = cells.iter().zip(attrs).enumerate().peekable();
    while let Some((i, (&text, &hl))) = drawn.next() {
        repeat += 1;
        let run_continues = drawn
            .peek()
            .is_some_and(|&(_, (&next_text, &next_hl))| next_hl == hl && next_text == text);
        if run_continues {
            continue;
        }

        // Saturating because an overfull buffer must flush rather
        // than wrap to "plenty of room", which is what C's unsigned
        // subtraction would have done.
        let used = ui.packer.used();
        if UI_BUF_SIZE.saturating_sub(used) < 2 * MAX_CELL_SIZE + 1
            || ui.ncells_pending >= MAX_CELLS_PENDING
        {
            // Out of room mid-run. Close this `grid_line` and open
            // another at the column reached, so that the reader sees
            // two complete events rather than one truncated one. A
            // trailing space is re-sent as a clear first, because the
            // reader's "rest of the line is this cell" shorthand cannot
            // span two events.
            if was_space {
                nelem += 1;
                ui.ncells_pending += 1;
                // The reserve covers one more cell.
                push_clear(ui, clearattr, 0);
            }
            // `lenpos` is this event's placeholder, in the block still
            // being packed into.
            ui.packer.patch_be16(lenpos, nelem);
            mpack_bool(&mut ui.packer, false);
            ui_flush_buf(ui);
            prepare_call(ui, GRID_LINE);
            let reached = startcol + (i - (repeat as usize - 1)) as Integer;
            lenpos = open_line(ui, grid, row, reached);
            nelem = 0;
            last_hl = -1;
        }

        let fields: u32 = if repeat > 1 {
            3
        } else if hl != last_hl as ScreenAttr {
            2
        } else {
            1
        };
        nelem += 1;
        // The check above left room for a whole cell.
        mpack_array(&mut ui.packer, fields);
        // The text's fixstr header carries its length, so the text is
        // copied out first and goes down after it.
        let mut glyph = [0u8; MAX_SCHAR_SIZE];
        let len = schar_get_adv(&mut &mut glyph[..], text);
        ui.packer.push_byte((0xa0 | len) as u8);
        ui.packer.push(&glyph[..len]);
        if fields >= 2 {
            mpack_uint(&mut ui.packer, hl as u32);
            if fields >= 3 {
                mpack_uint(&mut ui.packer, repeat);
            }
        }

        // A repeat counts as two cells however long it is: what the
        // budget is really measuring is how much the reader has to
        // draw before it can show something.
        ui.ncells_pending += repeat.min(2) as usize;
        last_hl = hl as c_int;
        repeat = 0;
        was_space = text == b' ' as ScreenChar;
    }

    if endcol < clearcol || was_space {
        // Clearing to `clearcol` is one cell with a repeat. A trailing
        // space is folded into it so that the reader does not have to
        // decide whether the run continues.
        nelem += 1;
        ui.ncells_pending += 1;
        // The reserve covers one more cell.
        push_clear(ui, clearattr, (clearcol - endcol) as u32);
    }
    // `lenpos` is the last event's placeholder, and the reserve covers the
    // closing byte.
    ui.packer.patch_be16(lenpos, nelem);
    mpack_bool(&mut ui.packer, flags & kLineFlagWrap != 0);
}

/// Writes a `grid_line` argument list up to its cell array, returning where
/// that array's length has to be patched in. The block must have room for
/// the header.
fn open_line(ui: &mut RemoteUI, grid: Integer, row: Integer, startcol: Integer) -> usize {
    mpack_array(&mut ui.packer, 5);
    for value in [grid, row, startcol] {
        mpack_uint(&mut ui.packer, value as u32);
    }
    mpack_array_dyn16(&mut ui.packer)
}

/// Writes a cell that clears `repeat` columns with `attr`. The block must
/// have room for it.
fn push_clear(ui: &mut RemoteUI, attr: Integer, repeat: u32) {
    mpack_array(&mut ui.packer, 3);
    mpack_str_small(&mut ui.packer, b" ");
    mpack_uint(&mut ui.packer, attr as u32);
    mpack_uint(&mut ui.packer, repeat);
}

/// [`remote_ui_raw_line`] for a UI on the pre-`ext_linegrid` protocol.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
fn raw_line_legacy(
    ui: &mut RemoteUI,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    cells: &[ScreenChar],
    attrs: &[ScreenAttr],
) {
    for (col, (&text, &hl)) in (startcol..).zip(cells.iter().zip(attrs)) {
        let mut cell: SCharBuf = [0; SCHAR_BUF_SIZE];
        let len = schar_copy(&mut cell, text);
        remote_ui_cursor_goto(ui, row, col);
        remote_ui_highlight_set(ui, hl as c_int);
        remote_ui_put(ui, &cell[..len]);
        if ambiguous_width_at(&cell[..len]) {
            // The UI and the server disagree about how wide that cell
            // was drawn, so the tracked column is no longer usable and
            // the next cell must move the cursor explicitly.
            ui.client_col = -1;
        }
    }
    if endcol >= clearcol {
        return;
    }
    remote_ui_cursor_goto(ui, row, endcol);
    remote_ui_highlight_set(ui, clearattr as c_int);
    if clearattr == 0 && clearcol == Integer::from(Columns.get()) {
        // Clearing the rest of the line with the default highlight has
        // its own call; anything else has to be spelled out cell by
        // cell, because the legacy protocol has no repeat count.
        let mut args = ArrayBuf::<0>::new();
        push_call(ui, c"eol_clear", args.array());
    } else {
        for _ in endcol..clearcol {
            remote_ui_put(ui, b" ");
        }
    }
}
