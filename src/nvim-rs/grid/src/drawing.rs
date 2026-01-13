//! Grid Drawing Primitives
//!
//! This module provides grid drawing helper functions for rendering text,
//! fills, and other visual elements to the grid.
//! Phase 171 of Rust migration.
//!
//! Note: Core drawing functions like rs_grid_put_linebuf, rs_grid_line_puts,
//! and rs_grid_line_fill are implemented in lib.rs. This module provides
//! additional drawing utilities and helpers.

use std::ffi::c_int;

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
type ScharT = u32;

/// Type alias for screen attribute (matches C's `sattr_T` which is `int32_t`).
type SattrT = i32;

// =============================================================================
// Drawing State Constants
// =============================================================================

/// Default background attribute
const DEFAULT_BG_ATTR: SattrT = 0;

/// Invalid/sentinel attribute value
const INVALID_ATTR: SattrT = -1;

// =============================================================================
// Drawing Range Calculations
// =============================================================================

/// Calculate the effective drawing range, clamped to grid bounds.
///
/// Returns the clamped end column value.
#[no_mangle]
pub extern "C" fn rs_draw_range_clamp(_start_col: c_int, end_col: c_int, max_col: c_int) -> c_int {
    if end_col > max_col {
        max_col
    } else if end_col < 0 {
        0
    } else {
        end_col
    }
}

/// Check if a drawing range is valid (non-empty and within bounds).
#[no_mangle]
pub extern "C" fn rs_draw_range_valid(start_col: c_int, end_col: c_int, max_col: c_int) -> c_int {
    c_int::from(start_col >= 0 && end_col > start_col && end_col <= max_col)
}

/// Calculate the width of a drawing range.
#[no_mangle]
pub extern "C" fn rs_draw_range_width(start_col: c_int, end_col: c_int) -> c_int {
    if end_col > start_col {
        end_col - start_col
    } else {
        0
    }
}

// =============================================================================
// Fill Operations
// =============================================================================

/// Check if a fill operation is needed.
///
/// Returns true if there are columns to fill and the fill character is valid.
#[no_mangle]
pub extern "C" fn rs_fill_needed(start_col: c_int, end_col: c_int, fill_char: ScharT) -> c_int {
    c_int::from(end_col > start_col && fill_char != 0)
}

/// Calculate number of columns to fill for a clear operation.
///
/// This considers rightleft mode where clearing may need different calculation.
#[no_mangle]
pub extern "C" fn rs_clear_width(
    start_col: c_int,
    end_col: c_int,
    clear_to: c_int,
    rightleft: c_int,
) -> c_int {
    if rightleft != 0 {
        // In rightleft mode, clear from start to clear_to
        if clear_to > start_col {
            clear_to - start_col
        } else {
            0
        }
    } else {
        // Normal mode, clear from end_col to clear_to
        if clear_to > end_col {
            clear_to - end_col
        } else {
            0
        }
    }
}

/// Determine the starting column for clear operations.
#[no_mangle]
pub extern "C" fn rs_clear_start(end_col: c_int, _clear_to: c_int, rightleft: c_int) -> c_int {
    if rightleft != 0 {
        // In rightleft, clearing happens before the content
        0
    } else {
        // Normal mode, clearing happens after end_col
        end_col
    }
}

// =============================================================================
// Attribute Handling
// =============================================================================

/// Combine background and clear attributes for a grid drawing operation.
///
/// The clear attribute is typically combined with the background attribute.
#[no_mangle]
pub extern "C" fn rs_draw_combine_attrs(bg_attr: SattrT, clear_attr: SattrT) -> SattrT {
    if clear_attr != DEFAULT_BG_ATTR {
        // Clear attribute takes precedence if non-default
        // In Neovim, attributes are combined via OR in some cases
        bg_attr | clear_attr
    } else {
        bg_attr
    }
}

/// Check if an attribute represents the default (no highlighting).
#[no_mangle]
pub extern "C" fn rs_attr_is_default(attr: SattrT) -> c_int {
    c_int::from(attr == DEFAULT_BG_ATTR)
}

/// Check if an attribute is invalid (sentinel value).
#[no_mangle]
pub extern "C" fn rs_attr_is_invalid(attr: SattrT) -> c_int {
    c_int::from(attr == INVALID_ATTR)
}

/// Get the effective attribute, defaulting invalid to a specified fallback.
#[no_mangle]
pub extern "C" fn rs_attr_or_default(attr: SattrT, fallback: SattrT) -> SattrT {
    if attr == INVALID_ATTR {
        fallback
    } else {
        attr
    }
}

// =============================================================================
// Line Drawing State
// =============================================================================

/// Check if a line needs redrawing based on dirty column tracking.
///
/// Returns true if any column in the range [0, end_col) is marked dirty.
#[no_mangle]
pub extern "C" fn rs_line_needs_redraw(dirty_col: c_int, end_col: c_int) -> c_int {
    // dirty_col < end_col means some column in range needs redrawing
    c_int::from(dirty_col < end_col)
}

/// Calculate the first dirty column in a line.
///
/// Given the current dirty_col and a new update range, returns the new dirty_col.
#[no_mangle]
pub extern "C" fn rs_update_dirty_col(
    current_dirty: c_int,
    update_start: c_int,
    _update_end: c_int,
) -> c_int {
    // Track the minimum column that needs redrawing
    if update_start < current_dirty {
        update_start
    } else {
        current_dirty
    }
}

/// Reset dirty column tracking for a line.
///
/// Returns the maximum possible column value to indicate "no dirty columns".
#[no_mangle]
pub extern "C" fn rs_reset_dirty_col(max_col: c_int) -> c_int {
    max_col
}

// =============================================================================
// Character Drawing Helpers
// =============================================================================

/// Check if a character can be drawn (non-NUL and valid).
#[no_mangle]
pub extern "C" fn rs_char_drawable(sc: ScharT) -> c_int {
    // A character is drawable if it's non-zero (not a continuation cell)
    c_int::from(sc != 0)
}

/// Get the character to use when a drawing position would be truncated.
///
/// Returns '>' for rightleft mode, '<' otherwise.
#[no_mangle]
pub extern "C" fn rs_truncation_char(rightleft: c_int) -> ScharT {
    if rightleft != 0 {
        // In rightleft mode, truncation indicator is '<'
        schar_from_ascii(b'<')
    } else {
        // Normal mode, truncation indicator is '>'
        schar_from_ascii(b'>')
    }
}

/// Get the filler character for empty lines (typically '~').
#[no_mangle]
pub extern "C" fn rs_filler_char() -> ScharT {
    schar_from_ascii(b'~')
}

/// Get the space character as an schar.
#[no_mangle]
pub extern "C" fn rs_space_char() -> ScharT {
    schar_from_ascii(b' ')
}

/// Convert ASCII byte to schar_T.
#[inline]
const fn schar_from_ascii(c: u8) -> ScharT {
    #[cfg(target_endian = "big")]
    {
        (c as ScharT) << 24
    }
    #[cfg(target_endian = "little")]
    {
        c as ScharT
    }
}

// =============================================================================
// Grid Row Operations
// =============================================================================

/// Check if a row is within drawable area.
#[no_mangle]
pub extern "C" fn rs_row_drawable(row: c_int, first_row: c_int, last_row: c_int) -> c_int {
    c_int::from(row >= first_row && row < last_row)
}

/// Calculate the screen row for a window-relative row.
#[no_mangle]
pub extern "C" fn rs_screen_row(win_row: c_int, row_offset: c_int) -> c_int {
    win_row + row_offset
}

/// Calculate the window-relative row from a screen row.
#[no_mangle]
pub extern "C" fn rs_window_row(screen_row: c_int, row_offset: c_int) -> c_int {
    screen_row - row_offset
}

// =============================================================================
// Drawing Boundary Calculations
// =============================================================================

/// Calculate the left boundary for a drawing operation.
///
/// Takes into account column offset and window boundaries.
#[no_mangle]
pub extern "C" fn rs_draw_left_bound(col_offset: c_int, start_col: c_int) -> c_int {
    col_offset + start_col
}

/// Calculate the right boundary for a drawing operation.
#[no_mangle]
pub extern "C" fn rs_draw_right_bound(col_offset: c_int, end_col: c_int) -> c_int {
    col_offset + end_col
}

/// Check if a drawing position is within the current viewport.
#[no_mangle]
pub extern "C" fn rs_pos_in_viewport(
    row: c_int,
    col: c_int,
    top_row: c_int,
    bot_row: c_int,
    left_col: c_int,
    right_col: c_int,
) -> c_int {
    c_int::from(row >= top_row && row < bot_row && col >= left_col && col < right_col)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_range_clamp() {
        assert_eq!(rs_draw_range_clamp(0, 80, 100), 80);
        assert_eq!(rs_draw_range_clamp(0, 120, 100), 100);
        assert_eq!(rs_draw_range_clamp(0, -5, 100), 0);
    }

    #[test]
    fn test_draw_range_valid() {
        assert_eq!(rs_draw_range_valid(0, 80, 100), 1);
        assert_eq!(rs_draw_range_valid(-1, 80, 100), 0);
        assert_eq!(rs_draw_range_valid(80, 80, 100), 0); // empty range
        assert_eq!(rs_draw_range_valid(0, 120, 100), 0); // exceeds max
    }

    #[test]
    fn test_draw_range_width() {
        assert_eq!(rs_draw_range_width(0, 80), 80);
        assert_eq!(rs_draw_range_width(10, 50), 40);
        assert_eq!(rs_draw_range_width(50, 10), 0); // invalid range
    }

    #[test]
    fn test_fill_needed() {
        assert_eq!(rs_fill_needed(0, 80, schar_from_ascii(b' ')), 1);
        assert_eq!(rs_fill_needed(0, 0, schar_from_ascii(b' ')), 0); // empty range
        assert_eq!(rs_fill_needed(0, 80, 0), 0); // NUL fill char
    }

    #[test]
    fn test_clear_width() {
        // Normal mode
        assert_eq!(rs_clear_width(0, 50, 80, 0), 30); // clear from 50 to 80
        assert_eq!(rs_clear_width(0, 80, 80, 0), 0); // nothing to clear

        // Rightleft mode
        assert_eq!(rs_clear_width(0, 50, 80, 1), 80); // clear from 0 to 80
        assert_eq!(rs_clear_width(20, 50, 80, 1), 60); // clear from 20 to 80
    }

    #[test]
    fn test_draw_combine_attrs() {
        assert_eq!(rs_draw_combine_attrs(1, 0), 1);
        assert_eq!(rs_draw_combine_attrs(1, 2), 3); // 1 | 2
        assert_eq!(rs_draw_combine_attrs(0, 4), 4);
    }

    #[test]
    fn test_attr_helpers() {
        assert_eq!(rs_attr_is_default(0), 1);
        assert_eq!(rs_attr_is_default(1), 0);
        assert_eq!(rs_attr_is_invalid(-1), 1);
        assert_eq!(rs_attr_is_invalid(0), 0);
        assert_eq!(rs_attr_or_default(-1, 5), 5);
        assert_eq!(rs_attr_or_default(3, 5), 3);
    }

    #[test]
    fn test_line_dirty_tracking() {
        assert_eq!(rs_line_needs_redraw(50, 80), 1);
        assert_eq!(rs_line_needs_redraw(80, 80), 0);
        assert_eq!(rs_update_dirty_col(80, 50, 60), 50);
        assert_eq!(rs_update_dirty_col(50, 60, 70), 50);
        assert_eq!(rs_reset_dirty_col(80), 80);
    }

    #[test]
    fn test_special_chars() {
        assert_ne!(rs_truncation_char(0), rs_truncation_char(1));
        assert_eq!(rs_filler_char(), schar_from_ascii(b'~'));
        assert_eq!(rs_space_char(), schar_from_ascii(b' '));
    }

    #[test]
    fn test_row_operations() {
        assert_eq!(rs_row_drawable(5, 0, 10), 1);
        assert_eq!(rs_row_drawable(10, 0, 10), 0); // at boundary
        assert_eq!(rs_row_drawable(-1, 0, 10), 0);
        assert_eq!(rs_screen_row(5, 3), 8);
        assert_eq!(rs_window_row(8, 3), 5);
    }

    #[test]
    fn test_drawing_bounds() {
        assert_eq!(rs_draw_left_bound(10, 5), 15);
        assert_eq!(rs_draw_right_bound(10, 80), 90);
        assert_eq!(rs_pos_in_viewport(5, 40, 0, 24, 0, 80), 1);
        assert_eq!(rs_pos_in_viewport(30, 40, 0, 24, 0, 80), 0); // row out of bounds
        assert_eq!(rs_pos_in_viewport(5, 100, 0, 24, 0, 80), 0); // col out of bounds
    }
}
