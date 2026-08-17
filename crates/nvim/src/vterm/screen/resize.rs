//! Resizing the screen: rebuilding both cell buffers at a new size.
//!
//! This is where the screen does its heaviest pointer work, so it sits apart
//! from the callback table that drives it. A resize walks the old grid from
//! the bottom up, so that the content nearest the cursor survives; with
//! reflow on it treats a run of continuation rows as one logical line and
//! re-wraps it to the new width; what falls off the top goes out to the
//! host's scrollback, and lines come back from scrollback to fill whatever
//! room is left at the bottom.
//!
//! Everything here works through the parent's [`Screen`] wrapper for the
//! reason the parent module gives: the host callbacks a resize drives —
//! `sb_pushline`, `sb_popline` and `resize` itself — re-enter the screen, so
//! no borrow of it may live across one. The two grids being built are raw
//! until they are installed, because the screen does not own them yet.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};

use crate::os::libc::{abort, fprintf, memmove, stderr};
use crate::types::{
    ScreenCell, VTermLineInfo, VTermPos, VTermScreenCell, VTermStateFields, size_t,
};
use crate::vterm::cell::{blank_cells, import_row};
use crate::vterm::vterm::{vterm_alloc, vterm_dealloc};

use super::{BUFIDX_ALTSCREEN, BUFIDX_PRIMARY, Screen, line_popcount, row_cells};

/// The two blocks a resize builds before installing them: a cell grid and the
/// line info that goes with it. Neither belongs to the screen until the swap
/// at the end, so they stay raw, and every use says which of `new_rows` rows
/// and `new_cols` columns it is inside.
struct NewBuffer {
    cells: *mut ScreenCell,
    lineinfo: *mut VTermLineInfo,
    rows: c_int,
    cols: c_int,
}

impl NewBuffer {
    /// A pair of blocks large enough for `rows` x `cols`, uninitialised.
    fn alloc(rows: c_int, cols: c_int) -> Self {
        let cell_bytes = size_of::<ScreenCell>() * rows as size_t * cols as size_t;
        let info_bytes = size_of::<VTermLineInfo>() * rows as size_t;
        // SAFETY: `vterm_alloc` answers live blocks of exactly those sizes,
        // and nothing else has a pointer to either yet.
        let (cells, lineinfo) = unsafe { (vterm_alloc(cell_bytes), vterm_alloc(info_bytes)) };
        NewBuffer {
            cells: cells as *mut ScreenCell,
            lineinfo: lineinfo as *mut VTermLineInfo,
            rows,
            cols,
        }
    }

    /// Row `row` of the new grid.
    fn row(&mut self, row: c_int) -> &mut [ScreenCell] {
        // SAFETY: the block holds `rows * cols` cells; callers only ask for a
        // row they have already kept inside `0..rows`.
        unsafe { row_cells(self.cells, row, self.cols) }
    }

    /// The line info of row `row`.
    fn info(&mut self, row: c_int) -> &mut VTermLineInfo {
        // SAFETY: the block holds `rows` infos, and `row` is inside them.
        unsafe { &mut *self.lineinfo.offset(row as isize) }
    }

    /// Slides `count` rows starting at row 0 down by `downwards` rows, cells
    /// and line info together. The ranges overlap, which is what `memmove`
    /// is for.
    fn shift_down(&mut self, downwards: c_int, count: c_int) {
        let cell_bytes = count as size_t * self.cols as size_t * size_of::<ScreenCell>();
        let info_bytes = count as size_t * size_of::<VTermLineInfo>();
        // SAFETY: `downwards + count <= rows`, so both destinations are
        // inside their blocks; `memmove` tolerates the overlap.
        let cells = unsafe { self.cells.offset((downwards * self.cols) as isize) };
        let lineinfo = unsafe { self.lineinfo.offset(downwards as isize) };
        unsafe { memmove(cells.cast(), self.cells.cast(), cell_bytes) };
        unsafe { memmove(lineinfo.cast(), self.lineinfo.cast(), info_bytes) };
    }

    /// Slides the `count` rows starting at row `from` up to row 0.
    fn shift_up(&mut self, from: c_int, count: c_int) {
        let cell_bytes = count as size_t * self.cols as size_t * size_of::<ScreenCell>();
        let info_bytes = count as size_t * size_of::<VTermLineInfo>();
        // SAFETY: as for `shift_down`; `from + count <= rows`.
        let cells = unsafe { self.cells.offset((from * self.cols) as isize) };
        let lineinfo = unsafe { self.lineinfo.offset(from as isize) };
        unsafe { memmove(self.cells.cast(), cells.cast(), cell_bytes) };
        unsafe { memmove(self.lineinfo.cast(), lineinfo.cast(), info_bytes) };
    }
}

/// The grid a resize is reading from, which the screen still owns.
struct OldBuffer {
    cells: *mut ScreenCell,
    lineinfo: *mut VTermLineInfo,
    rows: c_int,
    cols: c_int,
}

impl OldBuffer {
    /// Row `row` of the old grid.
    fn row(&self, row: c_int) -> &[ScreenCell] {
        // SAFETY: the screen's own buffer holds `rows * cols` cells, and
        // callers only ask for a row inside `0..rows`.
        unsafe { row_cells(self.cells, row, self.cols) }
    }

    /// Whether row `row` continues the line above it. False when the screen
    /// has no line info at all, which is how a resize before the first paint
    /// arrives.
    fn continues(&self, row: c_int) -> bool {
        if self.lineinfo.is_null() {
            return false;
        }
        // SAFETY: the state's line info has one entry per row of the grid it
        // belongs to, and `row` is inside them.
        unsafe { (*self.lineinfo.offset(row as isize)).continuation() != 0 }
    }
}

/// A line with no double-width, double-height or continuation marks.
fn blank_lineinfo() -> VTermLineInfo {
    VTermLineInfo {
        doublewidth_doubleheight_continuation: [0; 1],
        c2rust_padding: [0; 3],
    }
}

/// Rebuilds one of the screen's buffers at a new size.
///
/// Rows are laid out from the bottom up, so that the content nearest the
/// cursor survives. With reflow on, a run of continuation rows is one logical
/// line and is re-wrapped to the new width; otherwise every row stays a row.
/// Content that falls off the top goes to scrollback, and if there is room
/// left at the bottom, scrollback is popped back in to fill it. `active`
/// marks the buffer holding the cursor, whose position is rewritten.
///
/// # Safety
///
/// `statefields` must point at the live state's fields for the length of the
/// call.
unsafe fn resize_buffer(
    screen: &mut Screen,
    bufidx: usize,
    size: (c_int, c_int),
    active: bool,
    statefields: *mut VTermStateFields,
) {
    let (new_rows, new_cols) = size;
    // SAFETY: the caller promised the state's fields outlive the call, and
    // nothing between here and the writes at the end re-enters the state.
    let fields = unsafe { &mut *statefields };
    let old = OldBuffer {
        cells: screen.buffers[bufidx],
        lineinfo: fields.lineinfos[bufidx],
        rows: screen.rows,
        cols: screen.cols,
    };
    let mut new = NewBuffer::alloc(new_rows, new_cols);
    let pen = screen.pen;

    let mut old_row = old.rows - 1;
    let mut new_row = new_rows - 1;
    let old_cursor = fields.pos;
    let mut new_cursor = VTermPos { row: -1, col: -1 };
    // The topmost row known to be blank, i.e. how much room there is to
    // scroll content down into.
    let mut final_blank_row = new_rows;
    let do_reflow = screen.reflow() != 0 && bufidx == BUFIDX_PRIMARY;

    while old_row >= 0 {
        // Walk back over the continuation rows of one logical line.
        let old_row_end = old_row;
        while do_reflow && old_row > 0 && old.continues(old_row) {
            old_row -= 1;
        }
        let old_row_start = old_row;

        let mut width = 0;
        for row in old_row_start..=old_row_end {
            let wrapped = do_reflow && row < old.rows - 1 && old.continues(row + 1);
            width += if wrapped {
                old.cols
            } else {
                line_popcount(old.row(row))
            };
        }

        if final_blank_row == new_row + 1 && width == 0 {
            final_blank_row = new_row;
        }

        let new_height = if do_reflow && width != 0 {
            (width + new_cols - 1) / new_cols
        } else {
            1
        };
        let mut new_row_end = new_row;
        let mut new_row_start = new_row - new_height + 1;
        let spare_rows = new_rows - final_blank_row;

        if new_row_start < 0
            && spare_rows >= 0
            && (!active || new_cursor.row == -1 || new_cursor.row - new_row_start < new_rows)
        {
            // The line would fall off the top; push what is already placed
            // down into the blank rows at the bottom to make room.
            let downwards = (-new_row_start).min(spare_rows);
            new.shift_down(downwards, new_rows - downwards);
            new_row += downwards;
            new_row_start += downwards;
            new_row_end += downwards;
            if new_cursor.row >= 0 {
                new_cursor.row += downwards;
            }
            final_blank_row += downwards;
        }

        if new_row_start < 0 {
            // Out of room: this line and everything above it is scrollback.
            if old_row_start <= old_cursor.row && old_cursor.row <= old_row_end {
                new_cursor.row = 0;
                new_cursor.col = old_cursor.col.min(new_cols - 1);
            }
            break;
        }

        old_row = old_row_start;
        let mut old_col = 0;
        new_row = new_row_start;
        while new_row <= new_row_end {
            let mut count = width.min(new_cols);
            width -= count;
            let mut new_col = 0;
            while count != 0 {
                new.row(new_row)[new_col as usize] = old.row(old_row)[old_col as usize];
                if old_cursor.row == old_row && old_cursor.col == old_col {
                    new_cursor = VTermPos {
                        row: new_row,
                        col: new_col,
                    };
                }
                old_col += 1;
                if old_col == old.cols {
                    old_row += 1;
                    if !do_reflow {
                        new_col += 1;
                        break;
                    }
                    old_col = 0;
                }
                new_col += 1;
                count -= 1;
            }
            // The cursor sat in the blank tail of the old row.
            if old_cursor.row == old_row && old_cursor.col >= old_col {
                new_cursor.row = new_row;
                new_cursor.col = (old_cursor.col - old_col + new_col).min(new_cols - 1);
            }
            blank_cells(&mut new.row(new_row)[new_col as usize..], &pen);
            let continuation = (new_row > new_row_start) as u32;
            new.info(new_row).set_continuation(continuation);
            new_row += 1;
        }

        old_row = old_row_start - 1;
        new_row = new_row_start - 1;
    }

    if old_cursor.row <= old_row {
        // The cursor was on a row that fell off the top; bring it into range.
        new_cursor.row = 0;
        new_cursor.col = old_cursor.col.min(new_cols - 1);
    }
    if active && (new_cursor.row == -1 || new_cursor.col == -1) {
        let message = c"screen_resize failed to update cursor position\n".as_ptr();
        // SAFETY: a literal format string with no arguments, and `stderr` is
        // the C runtime's own stream.
        unsafe { fprintf(stderr, message) };
        // SAFETY: the process is over; `abort` does not return.
        unsafe { abort() };
    }

    if old_row >= 0 && bufidx == BUFIDX_PRIMARY {
        // Pushing a line hands it to the host, which may re-enter the screen,
        // so the state's fields are read and written around the loop rather
        // than held across it.
        if screen.takes_scrollback() {
            for row in 0..=old_row {
                screen.push_line(row);
            }
        }
        if active {
            // SAFETY: as for the borrow at the top — this is the first read
            // after the host callbacks above, so it is taken afresh.
            unsafe { (*statefields).pos.row -= old_row + 1 };
        }
    }
    if new_row >= 0 && bufidx == BUFIDX_PRIMARY {
        // SAFETY: same promise as above, taken afresh after the pushes.
        let popped_rows = backfill_from_scrollback(screen, &mut new, &mut new_row, old.cols);
        if active {
            unsafe { (*statefields).pos.row += popped_rows };
        }
    }
    if new_row >= 0 {
        // Content ended up low in the buffer; slide it up to the top and
        // blank whatever is left at the bottom.
        let moverows = new_rows - new_row - 1;
        new.shift_up(new_row + 1, moverows);
        new_cursor.row -= new_row + 1;
        for row in moverows..new_rows {
            blank_cells(new.row(row), &pen);
            *new.info(row) = blank_lineinfo();
        }
    }

    // SAFETY: both old blocks came from `vterm_alloc` and nothing reads them
    // after the swap below.
    unsafe { vterm_dealloc(old.cells.cast()) };
    unsafe { vterm_dealloc(old.lineinfo.cast()) };
    screen.buffers[bufidx] = new.cells;
    // SAFETY: the state's fields, taken afresh after the last callback.
    let fields = unsafe { &mut *statefields };
    fields.lineinfos[bufidx] = new.lineinfo;
    if active {
        fields.pos = new_cursor;
    }
}

/// Pops lines off the host's scrollback into the rows above `*new_row`, until
/// the host runs out or the space does. Leaves `*new_row` one above the
/// topmost row it filled and answers how many rows it took.
fn backfill_from_scrollback(
    screen: &mut Screen,
    new: &mut NewBuffer,
    new_row: &mut c_int,
    old_cols: c_int,
) -> c_int {
    let mut popped_rows = 0;
    while *new_row >= 0 {
        let Some(popped) = screen.pop_line(old_cols) else {
            break;
        };
        let (global_reverse, pen) = (screen.global_reverse() != 0, screen.pen);
        import_row(popped, new.row(*new_row), global_reverse, &pen);
        *new_row -= 1;
        popped_rows += 1;
    }
    popped_rows
}

impl Screen {
    /// Whether the host takes rows scrolled off the top.
    fn takes_scrollback(&self) -> bool {
        self.host()
            .is_some_and(|(host, _)| host.sb_pushline.is_some())
    }

    /// Asks the host for the line above the top of the screen, as `old_cols`
    /// cells in the staging buffer. `None` once its scrollback is empty.
    ///
    /// The answer borrows the staging buffer, not the screen: the buffer is
    /// only replaced by `realloc_sb_buffer`, which a resize calls at its two
    /// ends and never in the middle of a backfill.
    fn pop_line<'a>(&mut self, old_cols: c_int) -> Option<&'a [VTermScreenCell]> {
        let (host, data) = self.host()?;
        let sb_popline = host.sb_popline?;
        let sb_buffer = self.sb_buffer;
        // SAFETY: the host's own callback, reached with nothing borrowed; the
        // staging buffer holds a row of at least `old_cols` cells, which is
        // what the host is being asked to fill.
        if unsafe { sb_popline(old_cols, sb_buffer, data) } == 0 {
            return None;
        }
        // SAFETY: the host answered non-zero, so it wrote `old_cols` cells.
        Some(unsafe { core::slice::from_raw_parts(sb_buffer, old_cols as usize) })
    }

    /// Rebuilds the alternate screen's line info at a new height, for the
    /// case where the alternate grid itself was never allocated.
    fn reset_altscreen_lineinfo(&mut self, fields: &mut VTermStateFields, new_rows: c_int) {
        let old = fields.lineinfos[BUFIDX_ALTSCREEN];
        let bytes = size_of::<VTermLineInfo>() * new_rows as size_t;
        // SAFETY: the old block came from `vterm_alloc` and nothing reads it
        // again; the new one is live and holds `new_rows` infos.
        unsafe { vterm_dealloc(old.cast()) };
        let lineinfo = unsafe { vterm_alloc(bytes) } as *mut VTermLineInfo;
        let infos = unsafe { core::slice::from_raw_parts_mut(lineinfo, new_rows as usize) };
        infos.fill(blank_lineinfo());
        fields.lineinfos[BUFIDX_ALTSCREEN] = lineinfo;
    }
}

pub(super) unsafe extern "C" fn resize(
    new_rows: c_int,
    new_cols: c_int,
    fields: *mut VTermStateFields,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    let altscreen = screen.buffers[BUFIDX_ALTSCREEN];
    let altscreen_active = !altscreen.is_null() && screen.buffer == altscreen;
    let (old_rows, old_cols) = (screen.rows, screen.cols);

    // The scrollback staging buffer has to hold a row of either width, so it
    // is grown before the resize and shrunk after it.
    if new_cols > old_cols {
        realloc_sb_buffer(&mut screen, new_cols);
    }
    let (size, primary, alt) = ((new_rows, new_cols), BUFIDX_PRIMARY, BUFIDX_ALTSCREEN);
    // SAFETY: the state owns `fields` for the length of the call, which is
    // what `resize_buffer` promises to.
    unsafe { resize_buffer(&mut screen, primary, size, !altscreen_active, fields) };
    if !altscreen.is_null() {
        // SAFETY: as above.
        unsafe { resize_buffer(&mut screen, alt, size, altscreen_active, fields) };
    } else if new_rows != old_rows {
        // The altscreen itself is not allocated, but its line info still has
        // to match the new height.
        // SAFETY: as above; nothing here re-enters the state.
        let fields = unsafe { &mut *fields };
        screen.reset_altscreen_lineinfo(fields, new_rows);
    }

    screen.buffer = if altscreen_active {
        screen.buffers[BUFIDX_ALTSCREEN]
    } else {
        screen.buffers[BUFIDX_PRIMARY]
    };
    screen.rows = new_rows;
    screen.cols = new_cols;
    if new_cols <= old_cols {
        realloc_sb_buffer(&mut screen, new_cols);
    }

    screen.damage_screen();
    screen.report(
        |host| host.resize,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |resize, data| unsafe { resize(new_rows, new_cols, data) },
        1,
    )
}

/// Resizes the one-row staging buffer that carries cells to and from the
/// host's scrollback.
pub(super) fn realloc_sb_buffer(screen: &mut Screen, cols: c_int) {
    let old = screen.sb_buffer;
    let bytes = size_of::<VTermScreenCell>() * cols as size_t;
    // SAFETY: the old buffer came from `vterm_alloc` and nothing reads it
    // again; `vterm_alloc` answers a live block of `cols` cells.
    if !old.is_null() {
        unsafe { vterm_dealloc(old.cast()) };
    }
    screen.sb_buffer = unsafe { vterm_alloc(bytes) } as *mut VTermScreenCell;
}
