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
// Position Entry Validation
// =============================================================================

/// Result of validating a position entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosEntryResult {
    /// Valid position entry
    Valid,
    /// Entry should be skipped (line number <= 0)
    Skip,
    /// Invalid entry (wrong type or structure)
    Invalid,
}

/// Validate a single number as a position (line only).
///
/// A single number represents a whole line.
#[must_use]
pub const fn validate_number_position(lnum: i64) -> PosEntryResult {
    if lnum <= 0 {
        PosEntryResult::Skip
    } else {
        PosEntryResult::Valid
    }
}

/// Validate a list-based position [lnum] or [lnum, col] or [lnum, col, len].
#[must_use]
pub const fn validate_list_position(
    lnum: i64,
    col: i32,
    len: i32,
    list_len: usize,
) -> PosEntryResult {
    // Empty list is invalid
    if list_len == 0 {
        return PosEntryResult::Invalid;
    }

    // Line number <= 0 means skip
    if lnum <= 0 {
        return PosEntryResult::Skip;
    }

    // Negative col means skip
    if col < 0 {
        return PosEntryResult::Skip;
    }

    // Negative len means skip
    if len < 0 {
        return PosEntryResult::Skip;
    }

    PosEntryResult::Valid
}

// =============================================================================
// Position Overlap Detection
// =============================================================================

/// Check if a position overlaps with a line range.
///
/// A position overlaps if its line is within the range `[range_top, range_bot)`.
#[must_use]
pub const fn position_overlaps_range(lnum: i64, range_top: i64, range_bot: i64) -> bool {
    if lnum <= 0 || range_top <= 0 {
        return false;
    }
    lnum >= range_top && lnum < range_bot
}

/// Check if two positions on the same line overlap.
///
/// Positions are specified as (col, len) pairs.
/// Returns true if the character ranges overlap.
#[must_use]
pub const fn positions_overlap_on_line(col1: i32, len1: i32, col2: i32, len2: i32) -> bool {
    // Calculate effective start and end for both
    let start1 = calc_start_col(col1);
    let end1 = calc_end_col(col1, len1);

    let start2 = calc_start_col(col2);
    let end2 = calc_end_col(col2, len2);

    // Check for overlap
    start1 < end2 && start2 < end1
}

// =============================================================================
// Position Search
// =============================================================================

/// Find the first position that matches a given line.
///
/// Returns the index of the first matching position, or -1 if not found.
/// Positions array is (lnum, col, len) tuples.
#[must_use]
pub fn find_position_for_line(positions: &[(i64, i32, i32)], target_lnum: i64) -> i32 {
    for (i, &(lnum, _, _)) in positions.iter().enumerate() {
        if lnum == target_lnum {
            return i as i32;
        }
    }
    -1
}

/// Count positions that match a given line.
#[must_use]
pub fn count_positions_for_line(positions: &[(i64, i32, i32)], target_lnum: i64) -> i32 {
    let mut count = 0;
    for &(lnum, _, _) in positions {
        if lnum == target_lnum {
            count += 1;
        }
    }
    count
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

/// Validate a single number as position entry.
///
/// Returns: 0 = valid, 1 = skip, 2 = invalid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_validate_number_position(lnum: i64) -> c_int {
    match validate_number_position(lnum) {
        PosEntryResult::Valid => 0,
        PosEntryResult::Skip => 1,
        PosEntryResult::Invalid => 2,
    }
}

/// Validate a list-based position entry.
///
/// Returns: 0 = valid, 1 = skip, 2 = invalid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_validate_list_position(
    lnum: i64,
    col: c_int,
    len: c_int,
    list_len: c_int,
) -> c_int {
    if list_len < 0 {
        return 2; // Invalid
    }
    match validate_list_position(lnum, col, len, list_len as usize) {
        PosEntryResult::Valid => 0,
        PosEntryResult::Skip => 1,
        PosEntryResult::Invalid => 2,
    }
}

/// Check if position overlaps with line range.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_position_overlaps_range(
    lnum: i64,
    range_top: i64,
    range_bot: i64,
) -> c_int {
    c_int::from(position_overlaps_range(lnum, range_top, range_bot))
}

/// Check if two positions on same line overlap.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_positions_overlap_on_line(
    col1: c_int,
    len1: c_int,
    col2: c_int,
    len2: c_int,
) -> c_int {
    c_int::from(positions_overlap_on_line(col1, len1, col2, len2))
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

    #[test]
    fn test_validate_number_position() {
        assert_eq!(validate_number_position(1), PosEntryResult::Valid);
        assert_eq!(validate_number_position(100), PosEntryResult::Valid);
        assert_eq!(validate_number_position(0), PosEntryResult::Skip);
        assert_eq!(validate_number_position(-1), PosEntryResult::Skip);
    }

    #[test]
    fn test_validate_list_position() {
        // Valid positions
        assert_eq!(validate_list_position(1, 0, 0, 1), PosEntryResult::Valid);
        assert_eq!(validate_list_position(1, 5, 3, 3), PosEntryResult::Valid);

        // Empty list is invalid
        assert_eq!(validate_list_position(1, 0, 0, 0), PosEntryResult::Invalid);

        // Skip cases
        assert_eq!(validate_list_position(0, 0, 0, 1), PosEntryResult::Skip);
        assert_eq!(validate_list_position(-1, 0, 0, 1), PosEntryResult::Skip);
        assert_eq!(validate_list_position(1, -1, 0, 2), PosEntryResult::Skip);
        assert_eq!(validate_list_position(1, 1, -1, 3), PosEntryResult::Skip);
    }

    #[test]
    fn test_position_overlaps_range() {
        // In range
        assert!(position_overlaps_range(5, 3, 10));
        assert!(position_overlaps_range(3, 3, 10)); // At top
        assert!(position_overlaps_range(9, 3, 10)); // Just before bot

        // Out of range
        assert!(!position_overlaps_range(2, 3, 10)); // Before top
        assert!(!position_overlaps_range(10, 3, 10)); // At bot (exclusive)
        assert!(!position_overlaps_range(11, 3, 10)); // After bot

        // Invalid inputs
        assert!(!position_overlaps_range(0, 3, 10)); // Invalid lnum
        assert!(!position_overlaps_range(5, 0, 10)); // Invalid range_top
    }

    #[test]
    fn test_positions_overlap_on_line() {
        // Overlapping positions
        assert!(positions_overlap_on_line(5, 3, 6, 2)); // 5-7 overlaps 6-7
        assert!(positions_overlap_on_line(5, 5, 5, 5)); // Same position

        // Non-overlapping
        assert!(!positions_overlap_on_line(5, 2, 8, 2)); // 5-6 vs 8-9

        // Whole line (col=0) overlaps everything
        assert!(positions_overlap_on_line(0, 0, 5, 3));
        assert!(positions_overlap_on_line(5, 3, 0, 0));
    }

    #[test]
    fn test_find_position_for_line() {
        let positions = [(3, 1, 2), (5, 3, 1), (5, 10, 5), (10, 1, 1)];

        assert_eq!(find_position_for_line(&positions, 3), 0);
        assert_eq!(find_position_for_line(&positions, 5), 1); // First match
        assert_eq!(find_position_for_line(&positions, 10), 3);
        assert_eq!(find_position_for_line(&positions, 7), -1); // Not found
    }

    #[test]
    fn test_count_positions_for_line() {
        let positions = [(3, 1, 2), (5, 3, 1), (5, 10, 5), (10, 1, 1)];

        assert_eq!(count_positions_for_line(&positions, 3), 1);
        assert_eq!(count_positions_for_line(&positions, 5), 2);
        assert_eq!(count_positions_for_line(&positions, 10), 1);
        assert_eq!(count_positions_for_line(&positions, 7), 0);
    }
}
