//! Match display and rendering utilities
//!
//! This module provides functions for match highlighting display:
//! - Column range calculations
//! - Search highlight state management
//! - Match visibility checks
//! - Multi-line match handling

use std::ffi::c_int;

/// Maximum column value (represents end of line)
pub const MAXCOL: i32 = i32::MAX;

// =============================================================================
// Column Range Calculations
// =============================================================================

/// Calculate the start column for highlighting.
///
/// If the match starts on this line, use the start column.
/// Otherwise, highlight from column 0.
#[must_use]
pub const fn calc_start_col(lnum: i64, match_lnum: i64, start_col: i32) -> i32 {
    if lnum == match_lnum {
        start_col
    } else {
        0
    }
}

/// Calculate the end column for highlighting.
///
/// If the match ends on this line, use the end column.
/// Otherwise, highlight to MAXCOL (end of line).
#[must_use]
pub const fn calc_end_col(lnum: i64, match_lnum: i64, end_lnum_offset: i64, end_col: i32) -> i32 {
    if lnum == match_lnum + end_lnum_offset {
        end_col
    } else {
        MAXCOL
    }
}

/// Check if we need to highlight one character for an empty match.
///
/// Returns the new end column if adjustment is needed.
#[must_use]
pub const fn adjust_empty_match_end(start_col: i32, end_col: i32) -> i32 {
    if start_col == end_col {
        end_col + 1
    } else {
        end_col
    }
}

/// Check if a column is within the match range.
#[must_use]
pub const fn is_in_match_range(col: i32, start_col: i32, end_col: i32) -> bool {
    start_col != MAXCOL && col >= start_col && col < end_col
}

/// Check if column is at the end of the match.
#[must_use]
pub const fn is_at_match_end(col: i32, end_col: i32) -> bool {
    col == end_col
}

/// Check if this is the last column of the match.
#[must_use]
pub const fn is_on_last_col(col: i32, end_col: i32) -> bool {
    col + 1 >= end_col
}

// =============================================================================
// Search Highlight State
// =============================================================================

/// Check if search highlight should process before match list.
///
/// Search highlight has priority 0, matches are sorted by priority.
#[must_use]
pub const fn should_process_search_hl_first(match_priority: i32) -> bool {
    match_priority > 0 // SEARCH_HL_PRIORITY
}

/// Check if the match on this line starts at the given column.
#[must_use]
pub const fn match_starts_at_col(lnum: i64, match_lnum: i64, start_col: i32, col: i32) -> bool {
    lnum == match_lnum && col == start_col
}

/// Calculate whether conceal highlighting should be applied.
///
/// Returns:
/// - 0: No conceal
/// - 1: In conceal region (not at start)
/// - 2: At start of conceal region
#[must_use]
pub const fn calc_conceal_state(col: i32, start_col: i32, is_conceal_group: bool) -> i32 {
    if is_conceal_group {
        if col == start_col {
            2
        } else {
            1
        }
    } else {
        0
    }
}

// =============================================================================
// Multi-line Match Handling
// =============================================================================

/// Check if we're within a multi-line match.
#[must_use]
pub const fn is_multiline_match(end_lnum_offset: i64) -> bool {
    end_lnum_offset > 0
}

/// Calculate the end line of a match.
#[must_use]
pub const fn calc_match_end_lnum(start_lnum: i64, end_lnum_offset: i64) -> i64 {
    start_lnum + end_lnum_offset
}

/// Check if the current line is within a multi-line match range.
#[must_use]
pub const fn is_in_multiline_match(
    lnum: i64,
    match_lnum: i64,
    start_lnum_offset: i64,
    end_lnum_offset: i64,
) -> bool {
    lnum >= match_lnum + start_lnum_offset && lnum <= match_lnum + end_lnum_offset
}

// =============================================================================
// Cursor Position Checks
// =============================================================================

/// Check if the cursor is within the match.
///
/// The cursor is in the match if:
/// - Cursor line is within match line range
/// - Cursor column is after start (on start line) or before end (on end line)
#[must_use]
pub const fn is_cursor_in_match(
    cursor_lnum: i64,
    cursor_col: i32,
    match_lnum: i64,
    start_col: i32,
    end_lnum_offset: i64,
    end_col: i32,
) -> bool {
    let end_lnum = match_lnum + end_lnum_offset;

    if cursor_lnum < match_lnum || cursor_lnum > end_lnum {
        return false;
    }

    if cursor_lnum > match_lnum && cursor_lnum < end_lnum {
        return true;
    }

    let in_start = cursor_lnum > match_lnum || cursor_col >= start_col;
    let in_end = cursor_lnum < end_lnum || cursor_col < end_col;

    in_start && in_end
}

// =============================================================================
// Previous Column Highlight Check
// =============================================================================

/// Check if the previous column should be highlighted.
///
/// This is needed for highlighting at end of line when match continues.
#[must_use]
pub const fn should_highlight_prevcol(
    prevcol: i32,
    start_col: i32,
    end_col: i32,
    is_addpos: bool,
) -> bool {
    !is_addpos && (prevcol == start_col || (prevcol > start_col && end_col == MAXCOL))
}

/// Adjust prevcol for wrapped or scrolled display.
#[must_use]
pub const fn adjust_prevcol(prevcol: i32, skip_col: i32) -> i32 {
    if skip_col > prevcol {
        prevcol + 1
    } else {
        prevcol
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get MAXCOL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_display_maxcol() -> c_int {
    MAXCOL
}

/// Calculate start column for highlighting.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_hl_start_col(
    lnum: i64,
    match_lnum: i64,
    start_col: c_int,
) -> c_int {
    calc_start_col(lnum, match_lnum, start_col)
}

/// Calculate end column for highlighting.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_hl_end_col(
    lnum: i64,
    match_lnum: i64,
    end_lnum_offset: i64,
    end_col: c_int,
) -> c_int {
    calc_end_col(lnum, match_lnum, end_lnum_offset, end_col)
}

/// Adjust end column for empty match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_adjust_empty_match_end(start_col: c_int, end_col: c_int) -> c_int {
    adjust_empty_match_end(start_col, end_col)
}

/// Check if column is in match range.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_in_range(col: c_int, start_col: c_int, end_col: c_int) -> c_int {
    c_int::from(is_in_match_range(col, start_col, end_col))
}

/// Check if column is at match end.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_at_end(col: c_int, end_col: c_int) -> c_int {
    c_int::from(is_at_match_end(col, end_col))
}

/// Check if on last column of match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_on_last_col(col: c_int, end_col: c_int) -> c_int {
    c_int::from(is_on_last_col(col, end_col))
}

/// Check if should process `search_hl` first.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_should_process_search_hl_first(match_priority: c_int) -> c_int {
    c_int::from(should_process_search_hl_first(match_priority))
}

/// Check if match starts at column.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_starts_at_col(
    lnum: i64,
    match_lnum: i64,
    start_col: c_int,
    col: c_int,
) -> c_int {
    c_int::from(match_starts_at_col(lnum, match_lnum, start_col, col))
}

/// Calculate conceal state.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_conceal_state(
    col: c_int,
    start_col: c_int,
    is_conceal_group: c_int,
) -> c_int {
    calc_conceal_state(col, start_col, is_conceal_group != 0)
}

/// Check if this is a multi-line match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_multiline(end_lnum_offset: i64) -> c_int {
    c_int::from(is_multiline_match(end_lnum_offset))
}

/// Calculate match end line.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_end_lnum(start_lnum: i64, end_lnum_offset: i64) -> i64 {
    calc_match_end_lnum(start_lnum, end_lnum_offset)
}

/// Check if line is in multi-line match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_in_multiline(
    lnum: i64,
    match_lnum: i64,
    start_lnum_offset: i64,
    end_lnum_offset: i64,
) -> c_int {
    c_int::from(is_in_multiline_match(
        lnum,
        match_lnum,
        start_lnum_offset,
        end_lnum_offset,
    ))
}

/// Check if cursor is in match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_cursor_in_match(
    cursor_lnum: i64,
    cursor_col: c_int,
    match_lnum: i64,
    start_col: c_int,
    end_lnum_offset: i64,
    end_col: c_int,
) -> c_int {
    c_int::from(is_cursor_in_match(
        cursor_lnum,
        cursor_col,
        match_lnum,
        start_col,
        end_lnum_offset,
        end_col,
    ))
}

/// Check if previous column should be highlighted.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_should_highlight_prevcol(
    prevcol: c_int,
    start_col: c_int,
    end_col: c_int,
    is_addpos: c_int,
) -> c_int {
    c_int::from(should_highlight_prevcol(
        prevcol,
        start_col,
        end_col,
        is_addpos != 0,
    ))
}

/// Adjust prevcol for display.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_adjust_prevcol(prevcol: c_int, skip_col: c_int) -> c_int {
    adjust_prevcol(prevcol, skip_col)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_start_col() {
        // Same line - use start col
        assert_eq!(calc_start_col(5, 5, 10), 10);
        // Different line - use 0
        assert_eq!(calc_start_col(6, 5, 10), 0);
    }

    #[test]
    fn test_calc_end_col() {
        // End on same line
        assert_eq!(calc_end_col(5, 5, 0, 20), 20);
        // Multi-line, on end line
        assert_eq!(calc_end_col(6, 5, 1, 20), 20);
        // Multi-line, not on end line
        assert_eq!(calc_end_col(5, 5, 1, 20), MAXCOL);
    }

    #[test]
    fn test_adjust_empty_match_end() {
        // Empty match - adjust
        assert_eq!(adjust_empty_match_end(5, 5), 6);
        // Non-empty match - no change
        assert_eq!(adjust_empty_match_end(5, 10), 10);
    }

    #[test]
    fn test_is_in_match_range() {
        assert!(is_in_match_range(5, 3, 10));
        assert!(is_in_match_range(3, 3, 10));
        assert!(!is_in_match_range(10, 3, 10)); // exclusive end
        assert!(!is_in_match_range(2, 3, 10));
        assert!(!is_in_match_range(5, MAXCOL, 10)); // MAXCOL start = no range
    }

    #[test]
    fn test_is_at_match_end() {
        assert!(is_at_match_end(10, 10));
        assert!(!is_at_match_end(9, 10));
        assert!(!is_at_match_end(11, 10));
    }

    #[test]
    fn test_is_on_last_col() {
        assert!(is_on_last_col(9, 10));
        assert!(is_on_last_col(10, 10));
        assert!(!is_on_last_col(8, 10));
    }

    #[test]
    fn test_should_process_search_hl_first() {
        assert!(should_process_search_hl_first(1));
        assert!(should_process_search_hl_first(10));
        assert!(!should_process_search_hl_first(0));
        assert!(!should_process_search_hl_first(-1));
    }

    #[test]
    fn test_match_starts_at_col() {
        assert!(match_starts_at_col(5, 5, 10, 10));
        assert!(!match_starts_at_col(6, 5, 10, 10)); // wrong line
        assert!(!match_starts_at_col(5, 5, 10, 11)); // wrong col
    }

    #[test]
    fn test_calc_conceal_state() {
        // At start of conceal
        assert_eq!(calc_conceal_state(10, 10, true), 2);
        // In conceal region
        assert_eq!(calc_conceal_state(15, 10, true), 1);
        // Not conceal
        assert_eq!(calc_conceal_state(10, 10, false), 0);
    }

    #[test]
    fn test_is_multiline_match() {
        assert!(is_multiline_match(1));
        assert!(is_multiline_match(5));
        assert!(!is_multiline_match(0));
    }

    #[test]
    fn test_calc_match_end_lnum() {
        assert_eq!(calc_match_end_lnum(5, 0), 5);
        assert_eq!(calc_match_end_lnum(5, 3), 8);
    }

    #[test]
    fn test_is_in_multiline_match() {
        // In range
        assert!(is_in_multiline_match(6, 5, 0, 2)); // line 6, match 5-7
        assert!(is_in_multiline_match(5, 5, 0, 2)); // start line
        assert!(is_in_multiline_match(7, 5, 0, 2)); // end line
                                                    // Out of range
        assert!(!is_in_multiline_match(4, 5, 0, 2));
        assert!(!is_in_multiline_match(8, 5, 0, 2));
    }

    #[test]
    fn test_is_cursor_in_match() {
        // Single line match
        assert!(is_cursor_in_match(5, 10, 5, 5, 0, 15));
        assert!(!is_cursor_in_match(5, 3, 5, 5, 0, 15)); // before start
        assert!(!is_cursor_in_match(5, 15, 5, 5, 0, 15)); // at end

        // Multi-line match
        assert!(is_cursor_in_match(6, 0, 5, 10, 2, 20)); // middle line
        assert!(is_cursor_in_match(5, 15, 5, 10, 2, 20)); // start line, after start
        assert!(is_cursor_in_match(7, 10, 5, 10, 2, 20)); // end line, before end
    }

    #[test]
    fn test_should_highlight_prevcol() {
        // At start col
        assert!(should_highlight_prevcol(10, 10, 20, false));
        // After start, extends to end of line
        assert!(should_highlight_prevcol(15, 10, MAXCOL, false));
        // Not matching
        assert!(!should_highlight_prevcol(5, 10, 20, false));
        // Position match (is_addpos)
        assert!(!should_highlight_prevcol(10, 10, 20, true));
    }

    #[test]
    fn test_adjust_prevcol() {
        // Skip col > prevcol
        assert_eq!(adjust_prevcol(5, 10), 6);
        // Normal case
        assert_eq!(adjust_prevcol(10, 5), 10);
    }
}
