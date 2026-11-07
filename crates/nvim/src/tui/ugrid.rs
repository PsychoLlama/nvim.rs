//! The TUI's shadow copy of the terminal screen.
//!
//! The TUI is told what the editor's screen should look like, and decides what
//! to write by comparing that against what it last drew. This grid is that
//! record: one [`UCell`] per terminal cell, plus the cursor position the TUI
//! believes the terminal is at.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::narrow::as_index;
use crate::types::{ScreenAttr, ScreenChar};
use core::ffi::c_int;

/// One terminal cell: a grapheme handle and the highlight attributes it was
/// last drawn with.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct UCell {
    pub data: ScreenChar,
    pub attr: ScreenAttr,
}

/// A [`UGrid`]'s cells, row-major. Carries its own width so a row slice stays
/// correct no matter what the grid's public dimensions say.
pub struct UGridCells {
    pub(crate) width: usize,
    pub(crate) cells: Vec<UCell>,
}

/// The TUI's shadow copy of what the terminal is showing, plus where it
/// believes the cursor sits (`row` is -1 when that is unknown).
///
/// `cells` is `None` until the first [`UGrid::resize`]. That is also what the
/// zeroed `TUIData` the TUI allocates itself out of leaves behind, because
/// `Option<Box<_>>` spells `None` as a null pointer — so there is nothing to
/// initialise before the first resize.
pub struct UGrid {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub cells: Option<Box<UGridCells>>,
}

/// A cleared cell: an ASCII space carrying `attr`.
fn blank(attr: ScreenAttr) -> UCell {
    UCell {
        data: ScreenChar::from(b' '),
        attr,
    }
}

impl UGridCells {
    fn row(&self, row: c_int) -> &[UCell] {
        let start = as_index(row) * self.width;
        &self.cells[start..start + self.width]
    }

    fn row_mut(&mut self, row: c_int) -> &mut [UCell] {
        let start = as_index(row) * self.width;
        &mut self.cells[start..start + self.width]
    }
}

impl UGrid {
    /// Discard the old contents and allocate a blank grid of the new size.
    ///
    /// Cells start zeroed rather than blank — a zero grapheme handle is the
    /// "nothing drawn here yet" marker the printing path tests for.
    pub fn resize(&mut self, width: c_int, height: c_int) {
        self.cells = Some(Box::new(UGridCells {
            width: as_index(width),
            cells: vec![UCell::default(); as_index(width) * as_index(height)],
        }));
        self.width = width;
        self.height = height;
    }

    /// Blank the whole grid with the default attributes.
    pub fn clear(&mut self) {
        self.clear_region(0, self.height - 1, 0, self.width - 1, 0);
    }

    /// Blank `[col, endcol)` of one row.
    pub fn clear_chunk(&mut self, row: c_int, col: c_int, endcol: c_int, attr: ScreenAttr) {
        self.clear_region(row, row, col, endcol - 1, attr);
    }

    /// Blank the inclusive rectangle `[top, bot] x [left, right]`.
    fn clear_region(
        &mut self,
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
        attr: ScreenAttr,
    ) {
        let cells = self.cells();
        for row in top..=bot {
            cells.row_mut(row)[as_index(left)..=as_index(right)].fill(blank(attr));
        }
    }

    /// Record where the TUI has left the terminal's cursor.
    pub fn goto(&mut self, row: c_int, col: c_int) {
        self.row = row;
        self.col = col;
    }

    /// Move the inclusive column range `[left, right]` of rows `[top, bot]` by
    /// `count` rows, the way the terminal will when it scrolls that region.
    /// A positive `count` scrolls up (text moves towards `top`).
    ///
    /// Rows scrolled out of the region keep their old contents; the caller
    /// repaints them.
    pub fn scroll(&mut self, top: c_int, bot: c_int, left: c_int, right: c_int, count: c_int) {
        debug_assert!(right >= left && left >= 0);
        // Walk in the direction that copies a row only after its old contents
        // are no longer needed.
        let (start, stop, step) = if count > 0 {
            (top, bot - count + 1, 1)
        } else {
            (bot, top - count - 1, -1)
        };
        let cells = self.cells();
        let width = cells.width;
        let span = as_index(right - left + 1);
        let mut row = start;
        while row != stop {
            let source = as_index(row + count) * width + as_index(left);
            let target = as_index(row) * width + as_index(left);
            cells.cells.copy_within(source..source + span, target);
            row += step;
        }
    }

    /// One row of cells. Panics if the grid has not been sized yet.
    pub fn row(&self, row: c_int) -> &[UCell] {
        self.cells
            .as_deref()
            .expect("grid used before its first resize")
            .row(row)
    }

    /// One cell, copied out. Cells are two words; callers that want to keep
    /// looking at the grid while they print take a copy rather than a borrow.
    pub fn cell(&self, row: c_int, col: c_int) -> UCell {
        self.row(row)[as_index(col)]
    }

    /// Overwrite one cell.
    pub fn set_cell(&mut self, row: c_int, col: c_int, cell: UCell) {
        self.cells().row_mut(row)[as_index(col)] = cell;
    }

    fn cells(&mut self) -> &mut UGridCells {
        self.cells
            .as_deref_mut()
            .expect("grid used before its first resize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(width: c_int, height: c_int) -> UGrid {
        let mut grid = UGrid {
            row: 0,
            col: 0,
            width: 0,
            height: 0,
            cells: None,
        };
        grid.resize(width, height);
        for row in 0..height {
            for col in 0..width {
                grid.set_cell(
                    row,
                    col,
                    UCell {
                        data: (row * width + col).cast_unsigned(),
                        attr: row as ScreenAttr,
                    },
                );
            }
        }
        grid
    }

    fn data(grid: &UGrid, row: c_int) -> Vec<ScreenChar> {
        grid.row(row).iter().map(|c| c.data).collect()
    }

    #[test]
    fn resize_zeroes_every_cell() {
        let mut grid = filled(3, 2);
        grid.resize(4, 3);
        assert_eq!(grid.width, 4);
        assert_eq!(grid.height, 3);
        for row in 0..3 {
            assert_eq!(data(&grid, row), vec![0; 4]);
        }
    }

    #[test]
    fn clear_blanks_with_spaces() {
        let mut grid = filled(3, 2);
        grid.clear();
        for row in 0..2 {
            assert_eq!(grid.row(row), [blank(0); 3]);
        }
    }

    #[test]
    fn clear_chunk_stops_before_endcol() {
        let mut grid = filled(4, 1);
        grid.clear_chunk(0, 1, 3, 7);
        assert_eq!(grid.cell(0, 0).data, 0);
        assert_eq!(grid.row(0)[1..3], [blank(7); 2]);
        assert_eq!(grid.cell(0, 3).data, 3);
    }

    #[test]
    fn scroll_up_moves_rows_towards_the_top() {
        let mut grid = filled(2, 4);
        grid.scroll(0, 3, 0, 1, 1);
        assert_eq!(data(&grid, 0), vec![2, 3]);
        assert_eq!(data(&grid, 1), vec![4, 5]);
        assert_eq!(data(&grid, 2), vec![6, 7]);
        // The vacated row keeps its old contents until the caller repaints it.
        assert_eq!(data(&grid, 3), vec![6, 7]);
    }

    #[test]
    fn scroll_down_moves_rows_towards_the_bottom() {
        let mut grid = filled(2, 4);
        grid.scroll(0, 3, 0, 1, -1);
        assert_eq!(data(&grid, 0), vec![0, 1]);
        assert_eq!(data(&grid, 1), vec![0, 1]);
        assert_eq!(data(&grid, 2), vec![2, 3]);
        assert_eq!(data(&grid, 3), vec![4, 5]);
    }

    #[test]
    fn scroll_leaves_columns_outside_the_region_alone() {
        let mut grid = filled(4, 3);
        grid.scroll(0, 2, 1, 2, 1);
        assert_eq!(data(&grid, 0), vec![0, 5, 6, 3]);
        assert_eq!(data(&grid, 1), vec![4, 9, 10, 7]);
    }
}
