//! The shape of the screen, and where the cursor is allowed to be.
//!
//! The scroll region, the width of a row and the tab stops all come from
//! plain fields of the state plus the line marks, so all of it is arithmetic
//! over [`VTermState`] and none of it touches the consumer's callbacks.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_uint};

use crate::types::{VTermRect, VTermState};

/// A row is drawn at its normal width, or at half of it when the line is
/// marked double-width.
pub const DWL_OFF: bool = false;
pub const DWL_ON: bool = true;

/// A double-height line is drawn as a top half and a bottom half.
pub const DHL_OFF: c_uint = 0;
pub const DHL_TOP: c_uint = 1;
pub const DHL_BOTTOM: c_uint = 2;

impl VTermState {
    /// The first row below the scroll region. An unset bottom margin means
    /// the region runs to the bottom of the screen.
    pub(super) fn scroll_bottom(&self) -> c_int {
        if self.scrollregion_bottom > -1 {
            self.scrollregion_bottom
        } else {
            self.rows
        }
    }

    /// The leftmost column of the scroll region. The margins only bite while
    /// left/right margin mode is on.
    pub(super) fn scroll_left(&self) -> c_int {
        if self.mode.leftrightmargin() != 0 {
            self.scrollregion_left
        } else {
            0
        }
    }

    /// The first column right of the scroll region.
    pub(super) fn scroll_right(&self) -> c_int {
        if self.mode.leftrightmargin() != 0 && self.scrollregion_right > -1 {
            self.scrollregion_right
        } else {
            self.cols
        }
    }

    /// The whole scroll region as a rectangle.
    pub(super) fn scroll_region(&self) -> VTermRect {
        VTermRect {
            start_row: self.scrollregion_top,
            end_row: self.scroll_bottom(),
            start_col: self.scroll_left(),
            end_col: self.scroll_right(),
        }
    }

    /// How many columns row `row` holds: half the screen when the line is
    /// marked double-width.
    pub(super) fn row_width(&self, row: c_int) -> c_int {
        let row = usize::try_from(row).expect("a screen row is never negative");
        if self.lineinfo()[row].doublewidth() != 0 {
            self.cols / 2
        } else {
            self.cols
        }
    }

    /// How many columns the cursor's own row holds.
    pub(super) fn cursor_row_width(&self) -> c_int {
        self.row_width(self.pos.row)
    }

    /// Whether the cursor is inside the scroll region, which is what decides
    /// whether an insert or delete does anything at all.
    pub(super) fn cursor_in_scroll_region(&self) -> bool {
        (self.scrollregion_top..self.scroll_bottom()).contains(&self.pos.row)
            && (self.scroll_left()..self.scroll_right()).contains(&self.pos.col)
    }

    /// Pulls the cursor back inside the screen, or inside the scroll region
    /// when origin mode confines it there.
    pub(super) fn clamp_cursor(&mut self) {
        // Lower bound first, then upper, so that a region narrower than one
        // cell lands on its far edge exactly as upstream left it. The row is
        // settled before the column, because the row decides how wide the
        // column's own row is.
        if self.mode.origin() != 0 {
            let (top, bottom) = (self.scrollregion_top, self.scroll_bottom());
            let (left, right) = (self.scroll_left(), self.scroll_right());
            self.pos.row = self.pos.row.max(top).min(bottom - 1);
            self.pos.col = self.pos.col.max(left).min(right - 1);
        } else {
            self.pos.row = self.pos.row.max(0).min(self.rows - 1);
            self.pos.col = self.pos.col.max(0).min(self.cursor_row_width() - 1);
        }
    }

    /// Moves the cursor down a line, scrolling the region when it is already
    /// on the last line of it.
    pub(super) fn linefeed(&mut self) {
        if self.pos.row == self.scroll_bottom() - 1 {
            let rect = self.scroll_region();
            self.scroll(rect, 1, 0);
        } else if self.pos.row < self.rows - 1 {
            self.pos.row += 1;
        }
    }

    /// Moves the cursor to the `count`th tab stop, forwards when `forward`.
    /// Running out of row stops the walk short.
    pub(super) fn tab(&mut self, count: c_int, forward: bool) {
        let mut count = count;
        while count > 0 {
            if forward {
                if self.pos.col >= self.cursor_row_width() - 1 {
                    return;
                }
                self.pos.col += 1;
            } else {
                if self.pos.col < 1 {
                    return;
                }
                self.pos.col -= 1;
            }
            if self.is_tabstop(self.pos.col) {
                count -= 1;
            }
        }
    }

    /// Which byte of the tab-stop bitmap holds `col`, and its bit in it.
    fn tabstop_bit(col: c_int) -> (usize, u8) {
        let col = usize::try_from(col).expect("a screen column is never negative");
        (col >> 3, 1 << (col & 7))
    }

    pub(super) fn is_tabstop(&mut self, col: c_int) -> bool {
        let (byte, bit) = Self::tabstop_bit(col);
        self.tabstops_mut()[byte] & bit != 0
    }

    pub(super) fn set_tabstop(&mut self, col: c_int) {
        let (byte, bit) = Self::tabstop_bit(col);
        self.tabstops_mut()[byte] |= bit;
    }

    pub(super) fn clear_tabstop(&mut self, col: c_int) {
        let (byte, bit) = Self::tabstop_bit(col);
        self.tabstops_mut()[byte] &= !bit;
    }

    /// The power-on tab stops: one every eight columns.
    pub(super) fn reset_tabstops(&mut self) {
        for col in 0..self.cols {
            if col % 8 == 0 {
                self.set_tabstop(col);
            } else {
                self.clear_tabstop(col);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::VTermState;
    use crate::vterm::state::{BLANK_LINE, test_state};

    /// A screen with room for the margins the tests set.
    fn screen(state: &mut VTermState) {
        state.reset_tabstops();
    }

    #[test]
    fn an_unset_margin_reaches_the_edge_of_the_screen() {
        let (mut marks, mut stops) = ([BLANK_LINE; 8], [0u8; 2]);
        let mut state = test_state(&mut marks, &mut stops, 16);
        assert_eq!(state.scroll_bottom(), 8);
        assert_eq!((state.scroll_left(), state.scroll_right()), (0, 16));

        state.scrollregion_bottom = 5;
        assert_eq!(state.scroll_bottom(), 5);

        // The horizontal margins are only honoured once margin mode is on.
        state.scrollregion_left = 2;
        state.scrollregion_right = 12;
        assert_eq!((state.scroll_left(), state.scroll_right()), (0, 16));
        state.mode.set_leftrightmargin(1);
        assert_eq!((state.scroll_left(), state.scroll_right()), (2, 12));
    }

    #[test]
    fn a_row_marked_double_width_is_half_as_wide() {
        let (mut marks, mut stops) = ([BLANK_LINE; 4], [0u8; 2]);
        let mut state = test_state(&mut marks, &mut stops, 16);
        assert_eq!(state.row_width(1), 16);
        state.lineinfo_mut()[1].set_doublewidth(1);
        assert_eq!(state.row_width(1), 8);
        assert_eq!(state.row_width(0), 16);

        state.pos.row = 1;
        assert_eq!(state.cursor_row_width(), 8);
    }

    #[test]
    fn the_cursor_is_pulled_inside_the_screen_or_the_origin() {
        let (mut marks, mut stops) = ([BLANK_LINE; 8], [0u8; 2]);
        let mut state = test_state(&mut marks, &mut stops, 16);

        state.pos = crate::types::VTermPos { row: 99, col: -3 };
        state.clamp_cursor();
        assert_eq!((state.pos.row, state.pos.col), (7, 0));

        // Origin mode confines the cursor to the scroll region instead.
        state.scrollregion_top = 2;
        state.scrollregion_bottom = 6;
        state.scrollregion_left = 4;
        state.scrollregion_right = 10;
        state.mode.set_origin(1);
        state.mode.set_leftrightmargin(1);
        state.pos = crate::types::VTermPos { row: 0, col: 99 };
        state.clamp_cursor();
        assert_eq!((state.pos.row, state.pos.col), (2, 9));

        // A double-width row halves the column bound outside origin mode.
        state.mode.set_origin(0);
        state.mode.set_leftrightmargin(0);
        state.lineinfo_mut()[2].set_doublewidth(1);
        state.pos.col = 99;
        state.clamp_cursor();
        assert_eq!(state.pos.col, 7);
    }

    #[test]
    fn the_cursor_is_only_in_the_scroll_region_within_both_margins() {
        let (mut marks, mut stops) = ([BLANK_LINE; 8], [0u8; 2]);
        let mut state = test_state(&mut marks, &mut stops, 16);
        state.scrollregion_top = 2;
        state.scrollregion_bottom = 6;
        state.scrollregion_left = 4;
        state.scrollregion_right = 10;
        state.mode.set_leftrightmargin(1);

        state.pos = crate::types::VTermPos { row: 3, col: 5 };
        assert!(state.cursor_in_scroll_region());
        state.pos.row = 6;
        assert!(!state.cursor_in_scroll_region());
        state.pos.row = 1;
        assert!(!state.cursor_in_scroll_region());
        state.pos = crate::types::VTermPos { row: 3, col: 10 };
        assert!(!state.cursor_in_scroll_region());
        state.pos.col = 3;
        assert!(!state.cursor_in_scroll_region());
    }

    #[test]
    fn tab_stops_start_every_eighth_column_and_bound_the_walk() {
        let (mut marks, mut stops) = ([BLANK_LINE; 4], [0u8; 3]);
        let mut state = test_state(&mut marks, &mut stops, 24);
        screen(&mut state);
        assert!(state.is_tabstop(0) && state.is_tabstop(8) && state.is_tabstop(16));
        assert!(!state.is_tabstop(7));

        state.tab(1, true);
        assert_eq!(state.pos.col, 8);
        state.tab(2, true);
        assert_eq!(state.pos.col, 23); // the last column, not the stop past it
        state.tab(1, false);
        assert_eq!(state.pos.col, 16);
        state.tab(9, false);
        assert_eq!(state.pos.col, 0);

        state.clear_tabstop(8);
        state.pos.col = 0;
        state.tab(1, true);
        assert_eq!(state.pos.col, 16);
    }

    #[test]
    fn a_linefeed_on_the_last_row_of_the_region_scrolls_it() {
        let (mut marks, mut stops) = ([BLANK_LINE; 8], [0u8; 2]);
        let mut state = test_state(&mut marks, &mut stops, 16);
        state.pos.row = 3;
        state.linefeed();
        assert_eq!(state.pos.row, 4);

        // With a bottom margin the cursor stays put and the region scrolls.
        state.scrollregion_bottom = 5;
        state.pos.row = 4;
        state.lineinfo_mut()[3].set_doublewidth(1);
        state.linefeed();
        assert_eq!(state.pos.row, 4);
        // The marks moved up with their rows, leaving a blank line behind.
        assert_eq!(state.lineinfo()[2].doublewidth(), 1);
        assert_eq!(state.lineinfo()[3].doublewidth(), 0);
        assert_eq!(state.lineinfo()[4].doublewidth(), 0);
    }
}
