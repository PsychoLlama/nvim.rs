//! The TUI's shadow copy of the terminal screen.
//!
//! The TUI is told what the editor's screen should look like, and decides what
//! to write by comparing that against what it last drew. This grid is that
//! record: one [`UCell`] per terminal cell, plus the cursor position the TUI
//! believes the terminal is at.

#![forbid(unsafe_code)]

use crate::src::nvim::types::{UCell, UGrid, UGridCells, sattr_T, schar_T};
use core::ffi::c_int;

/// A cleared cell: an ASCII space carrying `attr`.
fn blank(attr: sattr_T) -> UCell {
    UCell {
        data: b' ' as schar_T,
        attr,
    }
}

impl UGridCells {
    fn row(&self, row: c_int) -> &[UCell] {
        let start = row as usize * self.width;
        &self.cells[start..start + self.width]
    }

    fn row_mut(&mut self, row: c_int) -> &mut [UCell] {
        let start = row as usize * self.width;
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
            width: width as usize,
            cells: vec![UCell::default(); width as usize * height as usize],
        }));
        self.width = width;
        self.height = height;
    }

    /// Blank the whole grid with the default attributes.
    pub fn clear(&mut self) {
        self.clear_region(0, self.height - 1, 0, self.width - 1, 0);
    }

    /// Blank `[col, endcol)` of one row.
    pub fn clear_chunk(&mut self, row: c_int, col: c_int, endcol: c_int, attr: sattr_T) {
        self.clear_region(row, row, col, endcol - 1, attr);
    }

    /// Blank the inclusive rectangle `[top, bot] x [left, right]`.
    fn clear_region(&mut self, top: c_int, bot: c_int, left: c_int, right: c_int, attr: sattr_T) {
        let cells = self.cells();
        for row in top..=bot {
            cells.row_mut(row)[left as usize..=right as usize].fill(blank(attr));
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
        let span = (right - left + 1) as usize;
        let mut row = start;
        while row != stop {
            let source = (row + count) as usize * width + left as usize;
            let target = row as usize * width + left as usize;
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
        self.row(row)[col as usize]
    }

    /// Overwrite one cell.
    pub fn set_cell(&mut self, row: c_int, col: c_int, cell: UCell) {
        self.cells().row_mut(row)[col as usize] = cell;
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
                        data: (row * width + col) as schar_T,
                        attr: row as sattr_T,
                    },
                );
            }
        }
        grid
    }

    fn data(grid: &UGrid, row: c_int) -> Vec<schar_T> {
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
