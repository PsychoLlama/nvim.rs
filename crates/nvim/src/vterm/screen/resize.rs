//! Resizing the screen: rebuilding both cell buffers at a new size.
//!
//! A resize walks the old grid from the bottom up, so that the content
//! nearest the cursor survives; with reflow on it treats a run of
//! continuation rows as one logical line and re-wraps it to the new width;
//! what falls off the top goes out to the host's scrollback, and lines come
//! back from scrollback to fill whatever room is left at the bottom.
//!
//! The state machine's line marks are rebuilt alongside the cells: the
//! resize is handed the state's cursor and its two mark arrays, and replaces
//! the arrays it rebuilds.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::narrow::as_index;
use core::ffi::c_int;

use crate::vterm::cell::{blank_cells, import_row};
use crate::vterm::types::{ScreenCell, VTermLineInfo, VTermPos, VTermScreen};

use super::{BUFIDX_ALTSCREEN, BUFIDX_PRIMARY, ZERO_CELL, line_popcount};

/// A line with no double-width, double-height or continuation marks.
const BLANK_LINEINFO: VTermLineInfo = VTermLineInfo {
    doublewidth_doubleheight_continuation: [0; 1],
    _pad: [0; 3],
};

/// The cell grid and line marks a resize builds before installing them.
struct NewBuffer {
    cells: Vec<ScreenCell>,
    lineinfo: Vec<VTermLineInfo>,
    cols: c_int,
}

impl NewBuffer {
    /// A grid and its marks for `rows` x `cols`, every byte zero.
    fn alloc(rows: c_int, cols: c_int) -> Self {
        let (rows, width) = (as_index(rows.max(0)), as_index(cols.max(0)));
        NewBuffer {
            cells: vec![ZERO_CELL; rows * width],
            lineinfo: vec![BLANK_LINEINFO; rows],
            cols,
        }
    }

    /// Row `row` of the new grid.
    fn row(&mut self, row: c_int) -> &mut [ScreenCell] {
        let width = as_index(self.cols);
        let start = as_index(row) * width;
        &mut self.cells[start..start + width]
    }

    /// The line marks of row `row`.
    fn info(&mut self, row: c_int) -> &mut VTermLineInfo {
        &mut self.lineinfo[as_index(row)]
    }

    /// Slides `count` rows starting at row 0 down by `downwards` rows, cells
    /// and line marks together.
    fn shift_down(&mut self, downwards: c_int, count: c_int) {
        let width = as_index(self.cols);
        let (downwards, count) = (as_index(downwards), as_index(count));
        self.cells.copy_within(..count * width, downwards * width);
        self.lineinfo.copy_within(..count, downwards);
    }

    /// Slides the `count` rows starting at row `from` up to row 0.
    fn shift_up(&mut self, from: c_int, count: c_int) {
        let width = as_index(self.cols);
        let (from, count) = (as_index(from), as_index(count));
        self.cells
            .copy_within(from * width..(from + count) * width, 0);
        self.lineinfo.copy_within(from..from + count, 0);
    }
}

/// The grid a resize is reading from.
struct OldBuffer<'a> {
    cells: &'a [ScreenCell],
    lineinfo: &'a [VTermLineInfo],
    rows: c_int,
    cols: c_int,
}

impl OldBuffer<'_> {
    /// Row `row` of the old grid.
    fn row(&self, row: c_int) -> &[ScreenCell] {
        let width = as_index(self.cols);
        let start = as_index(row) * width;
        &self.cells[start..start + width]
    }

    /// Whether row `row` continues the line above it.
    fn continues(&self, row: c_int) -> bool {
        self.lineinfo
            .get(as_index(row))
            .is_some_and(|info| info.continuation() != 0)
    }
}

impl VTermScreen {
    /// Rebuilds one of the screen's buffers at a new size.
    ///
    /// Rows are laid out from the bottom up, so that the content nearest the
    /// cursor survives. With reflow on, a run of continuation rows is one
    /// logical line and is re-wrapped to the new width; otherwise every row
    /// stays a row. Content that falls off the top goes to scrollback, and if
    /// there is room left at the bottom, scrollback is popped back in to fill
    /// it. `active` marks the buffer holding the cursor, `pos`, which is
    /// rewritten.
    fn resize_buffer(
        &mut self,
        bufidx: usize,
        size: (c_int, c_int),
        active: bool,
        pos: &mut VTermPos,
        lineinfos: &mut [Vec<VTermLineInfo>; 2],
    ) {
        let (new_rows, new_cols) = size;
        let mut new = NewBuffer::alloc(new_rows, new_cols);
        let pen = self.pen;
        let (old_rows, old_cols) = (self.rows, self.cols);
        let old_cursor = *pos;
        let do_reflow = self.reflow() != 0 && bufidx == BUFIDX_PRIMARY;

        let mut new_cursor = VTermPos { row: -1, col: -1 };
        // The topmost row known to be blank, i.e. how much room there is to
        // scroll content down into.
        let mut final_blank_row = new_rows;
        let mut old_row = old_rows - 1;
        let mut new_row = new_rows - 1;
        {
            let old = OldBuffer {
                cells: &self.owned.buffers[bufidx],
                lineinfo: &lineinfos[bufidx],
                rows: old_rows,
                cols: old_cols,
            };

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
                    && (!active
                        || new_cursor.row == -1
                        || new_cursor.row - new_row_start < new_rows)
                {
                    // The line would fall off the top; push what is already
                    // placed down into the blank rows at the bottom to make
                    // room.
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
                    // Out of room: this line and everything above it is
                    // scrollback.
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
                        new.row(new_row)[as_index(new_col)] = old.row(old_row)[as_index(old_col)];
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
                    blank_cells(&mut new.row(new_row)[as_index(new_col)..], &pen);
                    let continuation = u32::from(new_row > new_row_start);
                    new.info(new_row).set_continuation(continuation);
                    new_row += 1;
                }

                old_row = old_row_start - 1;
                new_row = new_row_start - 1;
            }
        }

        if old_cursor.row <= old_row {
            // The cursor was on a row that fell off the top; bring it into
            // range.
            new_cursor.row = 0;
            new_cursor.col = old_cursor.col.min(new_cols - 1);
        }
        if active && (new_cursor.row == -1 || new_cursor.col == -1) {
            eprintln!("screen_resize failed to update cursor position");
            std::process::abort();
        }

        if old_row >= 0 && bufidx == BUFIDX_PRIMARY {
            // Pushing a line hands it to the host.
            if self.takes_scrollback() {
                for row in 0..=old_row {
                    self.push_line(row);
                }
            }
            if active {
                pos.row -= old_row + 1;
            }
        }
        if new_row >= 0 && bufidx == BUFIDX_PRIMARY {
            let popped_rows = self.backfill_from_scrollback(&mut new, &mut new_row, old_cols);
            if active {
                pos.row += popped_rows;
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
                *new.info(row) = BLANK_LINEINFO;
            }
        }

        self.owned.buffers[bufidx] = new.cells;
        lineinfos[bufidx] = new.lineinfo;
        if active {
            *pos = new_cursor;
        }
    }

    /// Pops lines off the host's scrollback into the rows above `*new_row`,
    /// until the host runs out or the space does. Leaves `*new_row` one above
    /// the topmost row it filled and answers how many rows it took.
    fn backfill_from_scrollback(
        &mut self,
        new: &mut NewBuffer,
        new_row: &mut c_int,
        old_cols: c_int,
    ) -> c_int {
        let mut popped_rows = 0;
        while *new_row >= 0 {
            let (global_reverse, pen) = (self.global_reverse() != 0, self.pen);
            let Some(popped) = self.pop_line(old_cols) else {
                break;
            };
            import_row(popped, new.row(*new_row), global_reverse, &pen);
            *new_row -= 1;
            popped_rows += 1;
        }
        popped_rows
    }

    /// Asks the host for the line above the top of the screen, as `old_cols`
    /// cells in the staging buffer. `None` once its scrollback is empty.
    fn pop_line(&mut self, old_cols: c_int) -> Option<&[crate::vterm::types::VTermScreenCell]> {
        let owned = &mut *self.owned;
        let host = owned.host.as_deref_mut()?;
        let cells = &mut owned.sb_buffer[..as_index(old_cols)];
        if host.sb_popline(cells)? == 0 {
            return None;
        }
        Some(&owned.sb_buffer[..as_index(old_cols)])
    }

    /// The state's resize, as the screen takes it: both buffers rebuilt, the
    /// cursor moved, and the host told.
    pub(crate) fn resize(
        &mut self,
        new_rows: c_int,
        new_cols: c_int,
        pos: &mut VTermPos,
        lineinfos: &mut [Vec<VTermLineInfo>; 2],
    ) -> c_int {
        let has_altscreen = self.owned.has_altscreen;
        let altscreen_active = self.altscreen_active();
        let (old_rows, old_cols) = (self.rows, self.cols);

        // The scrollback staging buffer has to hold a row of either width, so
        // it is grown before the resize and shrunk after it.
        if new_cols > old_cols {
            self.realloc_sb_buffer(new_cols);
        }
        let size = (new_rows, new_cols);
        self.resize_buffer(BUFIDX_PRIMARY, size, !altscreen_active, pos, lineinfos);
        if has_altscreen {
            self.resize_buffer(BUFIDX_ALTSCREEN, size, altscreen_active, pos, lineinfos);
        } else if new_rows != old_rows {
            // The altscreen itself is not allocated, but its line marks still
            // have to match the new height.
            lineinfos[BUFIDX_ALTSCREEN] = vec![BLANK_LINEINFO; as_index(new_rows.max(0))];
        }

        self.owned.active = if altscreen_active {
            BUFIDX_ALTSCREEN
        } else {
            BUFIDX_PRIMARY
        };
        self.rows = new_rows;
        self.cols = new_cols;
        if new_cols <= old_cols {
            self.realloc_sb_buffer(new_cols);
        }

        self.damage_screen();
        self.host()
            .and_then(|host| host.resize(new_rows, new_cols))
            .unwrap_or(1)
    }
}
