//! Position-based match handling
//!
//! This module provides functions for managing position-based matches,
//! which are used by `matchaddpos()` to highlight specific positions in
//! a buffer without using a regex pattern.

use std::ffi::c_int;

use crate::MAX_POS_MATCHES;

// =============================================================================
// Position Validation
// =============================================================================

/// Check if a line number is valid for matching.
///
/// Line numbers must be positive (1-based indexing).
#[must_use]
pub const fn is_valid_lnum(lnum: i64) -> bool {
    lnum > 0
}

/// Check if a column number is valid.
///
/// Column 0 means the whole line.
/// Positive values are 1-based column positions.
#[must_use]
pub const fn is_valid_col(col: i32) -> bool {
    col >= 0
}

/// Check if a match length is valid.
///
/// Length 0 means default (1 character or whole line if col is 0).
/// Positive values specify the number of bytes to highlight.
#[must_use]
pub const fn is_valid_len(len: i32) -> bool {
    len >= 0
}

/// Check if a position match entry is valid.
#[must_use]
pub const fn is_valid_position(lnum: i64, col: i32, len: i32) -> bool {
    is_valid_lnum(lnum) && is_valid_col(col) && is_valid_len(len)
}

// =============================================================================
// Position Bounds
// =============================================================================

/// Calculate the end column for a position match.
///
/// If len is 0 and col is 0, returns MAXCOL (whole line).
/// If len is 0 and col > 0, returns col + 1 (single character).
/// Otherwise returns col + len.
#[must_use]
pub const fn calc_end_col(col: i32, len: i32) -> i32 {
    if col == 0 {
        i32::MAX // MAXCOL - whole line
    } else if len == 0 {
        col + 1
    } else {
        col + len
    }
}

/// Calculate start column (0-based) from 1-based col.
///
/// Column 0 means start of line (0).
/// Column > 0 is converted from 1-based to 0-based.
#[must_use]
pub const fn calc_start_col(col: i32) -> i32 {
    if col == 0 {
        0
    } else {
        col - 1
    }
}

/// Check if a position match covers the whole line.
#[must_use]
pub const fn is_whole_line(col: i32) -> bool {
    col == 0
}

// =============================================================================
// Position List Management
// =============================================================================

/// Check if position count is within limits.
#[must_use]
pub const fn is_valid_pos_count(count: usize) -> bool {
    count <= MAX_POS_MATCHES
}

/// Compare two position matches for sorting.
///
/// Sorts by line number first, then by column.
/// Returns negative if pos1 < pos2, positive if pos1 > pos2, 0 if equal.
#[must_use]
pub const fn compare_positions(lnum1: i64, col1: i32, lnum2: i64, col2: i32) -> i32 {
    if lnum1 < lnum2 {
        -1
    } else if lnum1 > lnum2 {
        1
    } else if col1 < col2 {
        -1
    } else if col1 > col2 {
        1
    } else {
        0
    }
}

// =============================================================================
// Line Range Calculation
// =============================================================================

/// Update top line number with a new position.
///
/// Returns the minimum of current top and new lnum.
#[must_use]
pub const fn update_toplnum(current: i64, new_lnum: i64) -> i64 {
    if current == 0 || new_lnum < current {
        new_lnum
    } else {
        current
    }
}

/// Update bottom line number with a new position.
///
/// Returns the maximum of current bottom and new lnum + 1.
#[must_use]
pub const fn update_botlnum(current: i64, new_lnum: i64) -> i64 {
    if current == 0 || new_lnum >= current {
        new_lnum + 1
    } else {
        current
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if line number is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_lnum(lnum: i64) -> c_int {
    c_int::from(is_valid_lnum(lnum))
}

/// Check if column is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_col(col: c_int) -> c_int {
    c_int::from(is_valid_col(col))
}

/// Check if length is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_len(len: c_int) -> c_int {
    c_int::from(is_valid_len(len))
}

/// Check if position is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_position(lnum: i64, col: c_int, len: c_int) -> c_int {
    c_int::from(is_valid_position(lnum, col, len))
}

/// Calculate end column for position match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_end_col(col: c_int, len: c_int) -> c_int {
    calc_end_col(col, len)
}

/// Calculate start column (0-based).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_start_col(col: c_int) -> c_int {
    calc_start_col(col)
}

/// Check if position covers whole line.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_whole_line(col: c_int) -> c_int {
    c_int::from(is_whole_line(col))
}

/// Check if position count is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_pos_count(count: c_int) -> c_int {
    if count < 0 {
        return 0;
    }
    c_int::from(is_valid_pos_count(count as usize))
}

/// Compare two positions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_compare_positions(
    lnum1: i64,
    col1: c_int,
    lnum2: i64,
    col2: c_int,
) -> c_int {
    compare_positions(lnum1, col1, lnum2, col2)
}

/// Update top line number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_update_toplnum(current: i64, new_lnum: i64) -> i64 {
    update_toplnum(current, new_lnum)
}

/// Update bottom line number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_update_botlnum(current: i64, new_lnum: i64) -> i64 {
    update_botlnum(current, new_lnum)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_lnum() {
        assert!(is_valid_lnum(1));
        assert!(is_valid_lnum(100));
        assert!(!is_valid_lnum(0));
        assert!(!is_valid_lnum(-1));
    }

    #[test]
    fn test_is_valid_col() {
        assert!(is_valid_col(0)); // Whole line
        assert!(is_valid_col(1)); // First column
        assert!(is_valid_col(100));
        assert!(!is_valid_col(-1));
    }

    #[test]
    fn test_is_valid_len() {
        assert!(is_valid_len(0)); // Default
        assert!(is_valid_len(1));
        assert!(is_valid_len(100));
        assert!(!is_valid_len(-1));
    }

    #[test]
    fn test_is_valid_position() {
        assert!(is_valid_position(1, 0, 0)); // Whole line
        assert!(is_valid_position(1, 1, 1)); // Single char
        assert!(is_valid_position(1, 5, 10)); // Range
        assert!(!is_valid_position(0, 0, 0)); // Invalid lnum
        assert!(!is_valid_position(1, -1, 0)); // Invalid col
        assert!(!is_valid_position(1, 1, -1)); // Invalid len
    }

    #[test]
    fn test_calc_end_col() {
        assert_eq!(calc_end_col(0, 0), i32::MAX); // Whole line
        assert_eq!(calc_end_col(5, 0), 6); // Single char at col 5
        assert_eq!(calc_end_col(5, 3), 8); // 3 chars starting at col 5
    }

    #[test]
    fn test_calc_start_col() {
        assert_eq!(calc_start_col(0), 0); // Whole line
        assert_eq!(calc_start_col(1), 0); // First col (1-based) -> 0
        assert_eq!(calc_start_col(5), 4); // Col 5 (1-based) -> 4
    }

    #[test]
    fn test_is_whole_line() {
        assert!(is_whole_line(0));
        assert!(!is_whole_line(1));
        assert!(!is_whole_line(5));
    }

    #[test]
    fn test_is_valid_pos_count() {
        assert!(is_valid_pos_count(0));
        assert!(is_valid_pos_count(8));
        assert!(!is_valid_pos_count(9));
        assert!(!is_valid_pos_count(100));
    }

    #[test]
    fn test_compare_positions() {
        // Different lines
        assert!(compare_positions(1, 5, 2, 1) < 0);
        assert!(compare_positions(2, 1, 1, 5) > 0);

        // Same line, different columns
        assert!(compare_positions(1, 1, 1, 5) < 0);
        assert!(compare_positions(1, 5, 1, 1) > 0);

        // Equal
        assert_eq!(compare_positions(1, 5, 1, 5), 0);
    }

    #[test]
    fn test_update_toplnum() {
        assert_eq!(update_toplnum(0, 10), 10); // Initial
        assert_eq!(update_toplnum(10, 5), 5); // New is smaller
        assert_eq!(update_toplnum(5, 10), 5); // Existing is smaller
    }

    #[test]
    fn test_update_botlnum() {
        assert_eq!(update_botlnum(0, 10), 11); // Initial (lnum + 1)
        assert_eq!(update_botlnum(5, 10), 11); // New is larger
        assert_eq!(update_botlnum(15, 10), 15); // Existing is larger
    }
}
