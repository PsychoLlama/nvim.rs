//! Indentation helper functions for Neovim
//!
//! This module provides additional Rust helper functions for indentation operations.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::manual_range_contains)]

use std::ffi::{c_char, c_int};

// Constants
const SPACE: c_char = b' ' as c_char;
const TAB: c_char = b'\t' as c_char;
const NUL: c_char = 0;

// Indent flags (for indent operations)
const INDENT_SET: c_int = 1;
const INDENT_INC: c_int = 2;
const INDENT_DEC: c_int = 3;

// Shiftround modes
const SR_ROUND: c_int = 0;
const SR_LEFT: c_int = 1;
const SR_RIGHT: c_int = 2;

// =============================================================================
// Character classification helpers
// =============================================================================

/// Check if a character is a space.
#[no_mangle]
pub extern "C" fn rs_indent_is_space(c: c_char) -> c_int {
    c_int::from(c == SPACE)
}

/// Check if a character is a tab.
#[no_mangle]
pub extern "C" fn rs_indent_is_tab(c: c_char) -> c_int {
    c_int::from(c == TAB)
}

/// Check if a character is whitespace (space or tab).
#[no_mangle]
pub extern "C" fn rs_indent_is_white(c: c_char) -> c_int {
    c_int::from(c == SPACE || c == TAB)
}

/// Check if a character is end of line (NUL).
#[no_mangle]
pub extern "C" fn rs_indent_is_eol(c: c_char) -> c_int {
    c_int::from(c == NUL)
}

// =============================================================================
// Indent flags helpers
// =============================================================================

/// Get INDENT_SET constant.
#[no_mangle]
pub extern "C" fn rs_indent_flag_set() -> c_int {
    INDENT_SET
}

/// Get INDENT_INC constant.
#[no_mangle]
pub extern "C" fn rs_indent_flag_inc() -> c_int {
    INDENT_INC
}

/// Get INDENT_DEC constant.
#[no_mangle]
pub extern "C" fn rs_indent_flag_dec() -> c_int {
    INDENT_DEC
}

/// Check if indent action is SET.
#[no_mangle]
pub extern "C" fn rs_indent_is_set(action: c_int) -> c_int {
    c_int::from(action == INDENT_SET)
}

/// Check if indent action is INC.
#[no_mangle]
pub extern "C" fn rs_indent_is_inc(action: c_int) -> c_int {
    c_int::from(action == INDENT_INC)
}

/// Check if indent action is DEC.
#[no_mangle]
pub extern "C" fn rs_indent_is_dec(action: c_int) -> c_int {
    c_int::from(action == INDENT_DEC)
}

// =============================================================================
// Shiftround helpers
// =============================================================================

/// Get SR_ROUND constant.
#[no_mangle]
pub extern "C" fn rs_sr_round() -> c_int {
    SR_ROUND
}

/// Get SR_LEFT constant.
#[no_mangle]
pub extern "C" fn rs_sr_left() -> c_int {
    SR_LEFT
}

/// Get SR_RIGHT constant.
#[no_mangle]
pub extern "C" fn rs_sr_right() -> c_int {
    SR_RIGHT
}

/// Check if shiftround is ROUND mode.
#[no_mangle]
pub extern "C" fn rs_sr_is_round(mode: c_int) -> c_int {
    c_int::from(mode == SR_ROUND)
}

/// Check if shiftround is LEFT mode.
#[no_mangle]
pub extern "C" fn rs_sr_is_left(mode: c_int) -> c_int {
    c_int::from(mode == SR_LEFT)
}

/// Check if shiftround is RIGHT mode.
#[no_mangle]
pub extern "C" fn rs_sr_is_right(mode: c_int) -> c_int {
    c_int::from(mode == SR_RIGHT)
}

// =============================================================================
// Indent calculation helpers
// =============================================================================

/// Round indent to the nearest shiftwidth boundary.
#[no_mangle]
pub extern "C" fn rs_indent_round(indent: c_int, sw: c_int) -> c_int {
    if sw <= 0 {
        return indent;
    }
    (indent + sw / 2) / sw * sw
}

/// Round indent down to the nearest shiftwidth boundary.
#[no_mangle]
pub extern "C" fn rs_indent_floor(indent: c_int, sw: c_int) -> c_int {
    if sw <= 0 {
        return indent;
    }
    indent / sw * sw
}

/// Round indent up to the nearest shiftwidth boundary.
#[no_mangle]
pub extern "C" fn rs_indent_ceil(indent: c_int, sw: c_int) -> c_int {
    if sw <= 0 {
        return indent;
    }
    (indent + sw - 1) / sw * sw
}

/// Calculate the new indent after shifting.
///
/// # Arguments
/// * `cur_indent` - Current indentation
/// * `sw` - Shiftwidth
/// * `count` - Number of shifts (positive = right, negative = left)
/// * `round` - Whether to round to shiftwidth boundary
#[no_mangle]
pub extern "C" fn rs_indent_shift(
    cur_indent: c_int,
    sw: c_int,
    count: c_int,
    round: c_int,
) -> c_int {
    if sw <= 0 {
        return cur_indent;
    }

    let shift_amount = sw * count.abs();
    let new_indent = if count > 0 {
        cur_indent + shift_amount
    } else {
        cur_indent - shift_amount
    };

    let new_indent = if new_indent < 0 { 0 } else { new_indent };

    if round != 0 {
        if count > 0 {
            rs_indent_ceil(new_indent, sw)
        } else {
            rs_indent_floor(new_indent, sw)
        }
    } else {
        new_indent
    }
}

/// Check if indent is on a shiftwidth boundary.
#[no_mangle]
pub extern "C" fn rs_indent_on_boundary(indent: c_int, sw: c_int) -> c_int {
    if sw <= 0 {
        return 1;
    }
    c_int::from(indent % sw == 0)
}

// =============================================================================
// Whitespace counting helpers
// =============================================================================

/// Count leading whitespace characters (spaces and tabs).
///
/// # Safety
/// `ptr` must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_count_leading_white(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut p = ptr;
    while *p == SPACE || *p == TAB {
        count += 1;
        p = p.add(1);
    }
    count
}

/// Count leading spaces only (not tabs).
///
/// # Safety
/// `ptr` must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_count_leading_spaces(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut p = ptr;
    while *p == SPACE {
        count += 1;
        p = p.add(1);
    }
    count
}

/// Count leading tabs only (not spaces).
///
/// # Safety
/// `ptr` must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_count_leading_tabs(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut p = ptr;
    while *p == TAB {
        count += 1;
        p = p.add(1);
    }
    count
}

/// Check if a line has only whitespace.
///
/// # Safety
/// `ptr` must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_line_is_blank(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 1;
    }

    let mut p = ptr;
    while *p == SPACE || *p == TAB {
        p = p.add(1);
    }
    c_int::from(*p == NUL)
}

/// Check if a line has no whitespace at all.
///
/// # Safety
/// `ptr` must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_line_no_indent(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 1;
    }
    c_int::from(*ptr != SPACE && *ptr != TAB)
}

// =============================================================================
// Column/width calculation helpers
// =============================================================================

/// Calculate the column position after adding spaces.
#[no_mangle]
pub extern "C" fn rs_col_add_spaces(col: c_int, spaces: c_int) -> c_int {
    col + spaces
}

/// Calculate the column position after adding a tab.
#[no_mangle]
pub extern "C" fn rs_col_add_tab(col: c_int, ts: c_int) -> c_int {
    if ts <= 0 {
        return col + 8; // Default
    }
    col + ts - (col % ts)
}

/// Calculate the number of spaces equivalent to tabs up to a column.
#[no_mangle]
pub extern "C" fn rs_tabs_to_spaces(ntabs: c_int, ts: c_int) -> c_int {
    if ts <= 0 {
        return ntabs * 8;
    }
    ntabs * ts
}

/// Calculate the number of tabs to cover a given number of columns.
#[no_mangle]
pub extern "C" fn rs_spaces_to_tabs(spaces: c_int, ts: c_int) -> c_int {
    if ts <= 0 {
        return spaces / 8;
    }
    spaces / ts
}

/// Get remaining spaces after tab expansion.
#[no_mangle]
pub extern "C" fn rs_spaces_after_tabs(spaces: c_int, ts: c_int) -> c_int {
    if ts <= 0 {
        return spaces % 8;
    }
    spaces % ts
}

// =============================================================================
// Expandtab helpers
// =============================================================================

/// Check if we should use spaces instead of tabs (expandtab).
/// This is a helper that returns the passed-in value for C compatibility.
#[no_mangle]
pub extern "C" fn rs_use_expandtab(expandtab: c_int) -> c_int {
    c_int::from(expandtab != 0)
}

/// Calculate number of characters needed for indent.
///
/// If using expandtab, returns spaces needed.
/// Otherwise, returns tabs + spaces.
#[no_mangle]
pub extern "C" fn rs_indent_chars_needed(indent: c_int, ts: c_int, expandtab: c_int) -> c_int {
    if expandtab != 0 {
        // All spaces
        indent
    } else {
        // Tabs + spaces
        let ts_val = if ts <= 0 { 8 } else { ts };
        let tabs = indent / ts_val;
        let spaces = indent % ts_val;
        tabs + spaces
    }
}

// =============================================================================
// Default value helpers
// =============================================================================

/// Get default tabstop value.
#[no_mangle]
pub extern "C" fn rs_default_ts() -> c_int {
    8
}

/// Get default shiftwidth value.
#[no_mangle]
pub extern "C" fn rs_default_sw() -> c_int {
    8
}

/// Get default softtabstop value.
#[no_mangle]
pub extern "C" fn rs_default_sts() -> c_int {
    0
}

/// Normalize tabstop value (0 becomes 8).
#[no_mangle]
pub extern "C" fn rs_normalize_ts(ts: c_int) -> c_int {
    if ts <= 0 {
        8
    } else {
        ts
    }
}

/// Normalize shiftwidth value (0 means use tabstop).
#[no_mangle]
pub extern "C" fn rs_normalize_sw(sw: c_int, ts: c_int) -> c_int {
    if sw <= 0 {
        rs_normalize_ts(ts)
    } else {
        sw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_char_classification() {
        assert_eq!(rs_indent_is_space(b' ' as c_char), 1);
        assert_eq!(rs_indent_is_space(b'\t' as c_char), 0);

        assert_eq!(rs_indent_is_tab(b'\t' as c_char), 1);
        assert_eq!(rs_indent_is_tab(b' ' as c_char), 0);

        assert_eq!(rs_indent_is_white(b' ' as c_char), 1);
        assert_eq!(rs_indent_is_white(b'\t' as c_char), 1);
        assert_eq!(rs_indent_is_white(b'a' as c_char), 0);
    }

    #[test]
    fn test_indent_flags() {
        assert_eq!(rs_indent_flag_set(), 1);
        assert_eq!(rs_indent_flag_inc(), 2);
        assert_eq!(rs_indent_flag_dec(), 3);

        assert_eq!(rs_indent_is_set(1), 1);
        assert_eq!(rs_indent_is_set(2), 0);
    }

    #[test]
    fn test_indent_round() {
        assert_eq!(rs_indent_round(0, 4), 0);
        assert_eq!(rs_indent_round(1, 4), 0);
        assert_eq!(rs_indent_round(2, 4), 4);
        assert_eq!(rs_indent_round(3, 4), 4);
        assert_eq!(rs_indent_round(4, 4), 4);
        assert_eq!(rs_indent_round(5, 4), 4);
        assert_eq!(rs_indent_round(6, 4), 8);
    }

    #[test]
    fn test_indent_floor_ceil() {
        assert_eq!(rs_indent_floor(5, 4), 4);
        assert_eq!(rs_indent_floor(7, 4), 4);
        assert_eq!(rs_indent_floor(8, 4), 8);

        assert_eq!(rs_indent_ceil(5, 4), 8);
        assert_eq!(rs_indent_ceil(4, 4), 4);
        assert_eq!(rs_indent_ceil(1, 4), 4);
    }

    #[test]
    fn test_indent_shift() {
        // Shift right without rounding
        assert_eq!(rs_indent_shift(0, 4, 1, 0), 4);
        assert_eq!(rs_indent_shift(2, 4, 1, 0), 6);

        // Shift left without rounding
        assert_eq!(rs_indent_shift(8, 4, -1, 0), 4);
        assert_eq!(rs_indent_shift(6, 4, -1, 0), 2);

        // Shift right with rounding
        assert_eq!(rs_indent_shift(2, 4, 1, 1), 8); // 2+4=6, ceil to 8
        assert_eq!(rs_indent_shift(0, 4, 1, 1), 4);

        // Shift left with rounding
        assert_eq!(rs_indent_shift(6, 4, -1, 1), 0); // 6-4=2, floor to 0
    }

    #[test]
    fn test_indent_on_boundary() {
        assert_eq!(rs_indent_on_boundary(0, 4), 1);
        assert_eq!(rs_indent_on_boundary(4, 4), 1);
        assert_eq!(rs_indent_on_boundary(8, 4), 1);
        assert_eq!(rs_indent_on_boundary(2, 4), 0);
        assert_eq!(rs_indent_on_boundary(5, 4), 0);
    }

    #[test]
    fn test_count_leading() {
        unsafe {
            let s = CString::new("    hello").unwrap();
            assert_eq!(rs_count_leading_white(s.as_ptr()), 4);
            assert_eq!(rs_count_leading_spaces(s.as_ptr()), 4);
            assert_eq!(rs_count_leading_tabs(s.as_ptr()), 0);

            let s = CString::new("\t\thello").unwrap();
            assert_eq!(rs_count_leading_white(s.as_ptr()), 2);
            assert_eq!(rs_count_leading_spaces(s.as_ptr()), 0);
            assert_eq!(rs_count_leading_tabs(s.as_ptr()), 2);

            let s = CString::new("hello").unwrap();
            assert_eq!(rs_count_leading_white(s.as_ptr()), 0);
        }
    }

    #[test]
    fn test_line_blank() {
        unsafe {
            let blank = CString::new("   \t  ").unwrap();
            assert_eq!(rs_line_is_blank(blank.as_ptr()), 1);

            let empty = CString::new("").unwrap();
            assert_eq!(rs_line_is_blank(empty.as_ptr()), 1);

            let content = CString::new("  hello").unwrap();
            assert_eq!(rs_line_is_blank(content.as_ptr()), 0);
        }
    }

    #[test]
    fn test_col_calculations() {
        assert_eq!(rs_col_add_spaces(0, 4), 4);
        assert_eq!(rs_col_add_tab(0, 8), 8);
        assert_eq!(rs_col_add_tab(2, 8), 8);
        assert_eq!(rs_col_add_tab(8, 8), 16);

        assert_eq!(rs_tabs_to_spaces(2, 8), 16);
        assert_eq!(rs_spaces_to_tabs(16, 8), 2);
        assert_eq!(rs_spaces_after_tabs(10, 8), 2);
    }

    #[test]
    fn test_indent_chars_needed() {
        // With expandtab, need 8 spaces
        assert_eq!(rs_indent_chars_needed(8, 4, 1), 8);

        // Without expandtab, need 2 tabs
        assert_eq!(rs_indent_chars_needed(8, 4, 0), 2);

        // Without expandtab, need 1 tab + 2 spaces
        assert_eq!(rs_indent_chars_needed(6, 4, 0), 3);
    }

    #[test]
    fn test_defaults() {
        assert_eq!(rs_default_ts(), 8);
        assert_eq!(rs_default_sw(), 8);
        assert_eq!(rs_default_sts(), 0);

        assert_eq!(rs_normalize_ts(0), 8);
        assert_eq!(rs_normalize_ts(4), 4);

        assert_eq!(rs_normalize_sw(0, 4), 4);
        assert_eq!(rs_normalize_sw(2, 4), 2);
    }
}
