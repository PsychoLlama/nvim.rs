//! Boolean and numeric option callback implementations
//!
//! This module contains Rust implementations of boolean and numeric option
//! validation callbacks. These callbacks validate option values and may
//! trigger side effects through C functions.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

use std::ffi::{c_char, c_int};

use super::{callback_ok, CallbackResult};
use crate::OptInt;

// =============================================================================
// Error Messages
// =============================================================================

/// Error: Invalid argument
const E_INVARG: *const c_char = c"E474: Invalid argument".as_ptr();

/// Error: Number required after =
const E_NUMBER_REQ: *const c_char = c"E521: Number required after =".as_ptr();

// =============================================================================
// Numeric Validation Helpers
// =============================================================================

/// Validate that a numeric value is within a range (inclusive).
/// Returns NULL on success, or an error message on failure.
#[no_mangle]
pub extern "C" fn rs_validate_num_range(value: OptInt, min: OptInt, max: OptInt) -> CallbackResult {
    if value >= min && value <= max {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate that a numeric value is non-negative (>= 0).
#[no_mangle]
pub extern "C" fn rs_validate_num_nonnegative(value: OptInt) -> CallbackResult {
    if value >= 0 {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate that a numeric value is positive (> 0).
#[no_mangle]
pub extern "C" fn rs_validate_num_positive(value: OptInt) -> CallbackResult {
    if value > 0 {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate 'cmdheight' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_cmdheight(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'columns' option value.
/// Must be >= 12.
#[no_mangle]
pub extern "C" fn rs_validate_columns(value: OptInt) -> CallbackResult {
    if value >= 12 {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate 'lines' option value.
/// Must be >= 2.
#[no_mangle]
pub extern "C" fn rs_validate_lines(value: OptInt) -> CallbackResult {
    if value >= 2 {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate 'cmdwinheight' option value.
/// Must be between 1 and max screen height (we check >= 1 here).
#[no_mangle]
pub extern "C" fn rs_validate_cmdwinheight(value: OptInt) -> CallbackResult {
    rs_validate_num_positive(value)
}

/// Validate 'helpheight' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_helpheight(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

// Note: rs_validate_history already exists in lib.rs with proper max check

/// Validate 'laststatus' option value.
/// Must be 0, 1, 2, or 3.
#[no_mangle]
pub extern "C" fn rs_validate_laststatus(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 0, 3)
}

/// Validate 'lazyredraw' related numeric options.
/// Just validates it's a valid boolean (0 or 1) - but it's actually a number.
#[no_mangle]
pub extern "C" fn rs_validate_boolean_int(value: OptInt) -> CallbackResult {
    if value == 0 || value == 1 {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate 'numberwidth' option value.
/// Must be between 1 and 20.
#[no_mangle]
pub extern "C" fn rs_validate_numberwidth(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 1, 20)
}

/// Validate 'pumheight' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_pumheight(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'pumwidth' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_pumwidth(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'report' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_report(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'scroll' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_scroll(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'scrolljump' option value.
/// Can be negative (for percentage) or positive.
#[no_mangle]
pub extern "C" fn rs_validate_scrolljump(_value: OptInt) -> CallbackResult {
    // scrolljump can be any value (negative means percentage)
    callback_ok()
}

/// Validate 'scrolloff' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_scrolloff(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'shiftwidth' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_shiftwidth(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'showtabline' option value.
/// Must be 0, 1, or 2.
#[no_mangle]
pub extern "C" fn rs_validate_showtabline(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 0, 2)
}

/// Validate 'sidescroll' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_sidescroll(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'sidescrolloff' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_sidescrolloff(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'softtabstop' option value.
/// Can be negative (-1 for shiftwidth) or >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_softtabstop(value: OptInt) -> CallbackResult {
    if value >= -1 {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate 'tabstop' option value.
/// Must be >= 1 and <= 9999.
#[no_mangle]
pub extern "C" fn rs_validate_tabstop(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 1, 9999)
}

/// Validate 'textwidth' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_textwidth(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'timeoutlen' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_timeoutlen(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'titlelen' option value.
/// Must be >= 0 and <= 100 (percentage of screen width).
#[no_mangle]
pub extern "C" fn rs_validate_titlelen(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 0, 100)
}

/// Validate 'updatecount' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_updatecount(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'updatetime' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_updatetime(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'winheight' option value.
/// Must be >= 1.
#[no_mangle]
pub extern "C" fn rs_validate_winheight(value: OptInt) -> CallbackResult {
    rs_validate_num_positive(value)
}

/// Validate 'winminheight' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_winminheight(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'winminwidth' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_winminwidth(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'winwidth' option value.
/// Must be >= 1.
#[no_mangle]
pub extern "C" fn rs_validate_winwidth(value: OptInt) -> CallbackResult {
    rs_validate_num_positive(value)
}

/// Validate 'wrapmargin' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_wrapmargin(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'foldlevel' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_foldlevel(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'foldminlines' option value.
/// Must be >= 0.
#[no_mangle]
pub extern "C" fn rs_validate_foldminlines(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'foldnestmax' option value.
/// Must be >= 0 and <= 20.
#[no_mangle]
pub extern "C" fn rs_validate_foldnestmax(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 0, 20)
}

/// Validate 'foldcolumn' option value.
/// Must be >= 0 and <= 12.
#[no_mangle]
pub extern "C" fn rs_validate_foldcolumn(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 0, 12)
}

/// Validate 'conceallevel' option value.
/// Must be 0, 1, 2, or 3.
#[no_mangle]
pub extern "C" fn rs_validate_conceallevel(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 0, 3)
}

// Note: rs_validate_regexpengine already exists in lib.rs

/// Validate 'iminsert' / 'imsearch' option value.
/// Must be 0, 1, or 2 (but we'll just check >= 0).
#[no_mangle]
pub extern "C" fn rs_validate_iminsert(value: OptInt) -> CallbackResult {
    rs_validate_num_range(value, 0, 2)
}

// =============================================================================
// Percentage Validation
// =============================================================================

// Note: rs_validate_percentage already exists in lib.rs
// Note: rs_validate_pumblend is in validate.rs (Phase 2)
// Note: rs_validate_winblend needs to use the lib.rs percentage validator

// =============================================================================
// Special Numeric Validators
// =============================================================================

/// Validate 'verbose' option value.
/// Can be any integer value.
#[no_mangle]
pub extern "C" fn rs_validate_verbose(_value: OptInt) -> CallbackResult {
    callback_ok()
}

/// Validate 'undolevels' option value.
/// Can be -1 (no undo) or any positive value.
#[no_mangle]
pub extern "C" fn rs_validate_undolevels(value: OptInt) -> CallbackResult {
    if value >= -1 {
        callback_ok()
    } else {
        E_INVARG
    }
}

/// Validate 'synmaxcol' option value.
/// Must be >= 0 (0 means no limit).
#[no_mangle]
pub extern "C" fn rs_validate_synmaxcol(value: OptInt) -> CallbackResult {
    rs_validate_num_nonnegative(value)
}

/// Validate 'colorcolumn' parsed values.
/// Each column must be > 0 or be a relative offset.
/// This validates a single parsed column value.
#[no_mangle]
pub extern "C" fn rs_validate_colorcolumn_value(
    value: OptInt,
    is_relative: c_int,
) -> CallbackResult {
    if is_relative != 0 {
        // Relative values can be positive or negative
        callback_ok()
    } else if value > 0 {
        callback_ok()
    } else {
        E_INVARG
    }
}

// =============================================================================
// Parse Functions
// =============================================================================

/// Parse a simple integer from a string.
/// Returns the parsed value and updates the pointer position.
/// Returns 0 and sets error to 1 if parsing fails.
#[repr(C)]
pub struct ParseIntResult {
    /// The parsed value
    pub value: OptInt,
    /// Number of characters consumed
    pub consumed: usize,
    /// 0 on success, 1 on error
    pub error: c_int,
}

/// Parse an integer from a string.
/// Handles optional leading sign (+ or -).
#[no_mangle]
pub unsafe extern "C" fn rs_parse_int(s: *const c_char) -> ParseIntResult {
    let mut result = ParseIntResult {
        value: 0,
        consumed: 0,
        error: 0,
    };

    if s.is_null() || *s == 0 {
        result.error = 1;
        return result;
    }

    let mut p = s;
    let mut negative = false;

    // Handle sign
    if *p as u8 == b'-' {
        negative = true;
        p = p.add(1);
        result.consumed += 1;
    } else if *p as u8 == b'+' {
        p = p.add(1);
        result.consumed += 1;
    }

    // Must have at least one digit
    if !(*p as u8).is_ascii_digit() {
        result.error = 1;
        return result;
    }

    // Parse digits
    let mut value: i64 = 0;
    while (*p as u8).is_ascii_digit() {
        value = value * 10 + i64::from(*p as u8 - b'0');
        p = p.add(1);
        result.consumed += 1;
    }

    result.value = if negative { -value } else { value };
    result
}

// =============================================================================
// Error Message Accessor
// =============================================================================

/// Get the "Number required" error message.
#[no_mangle]
pub extern "C" fn rs_error_number_required() -> *const c_char {
    E_NUMBER_REQ
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_validate_num_range() {
        assert!(rs_validate_num_range(5, 0, 10).is_null());
        assert!(rs_validate_num_range(0, 0, 10).is_null());
        assert!(rs_validate_num_range(10, 0, 10).is_null());
        assert!(!rs_validate_num_range(-1, 0, 10).is_null());
        assert!(!rs_validate_num_range(11, 0, 10).is_null());
    }

    #[test]
    fn test_validate_num_nonnegative() {
        assert!(rs_validate_num_nonnegative(0).is_null());
        assert!(rs_validate_num_nonnegative(100).is_null());
        assert!(!rs_validate_num_nonnegative(-1).is_null());
    }

    #[test]
    fn test_validate_num_positive() {
        assert!(rs_validate_num_positive(1).is_null());
        assert!(rs_validate_num_positive(100).is_null());
        assert!(!rs_validate_num_positive(0).is_null());
        assert!(!rs_validate_num_positive(-1).is_null());
    }

    #[test]
    fn test_validate_specific_options() {
        // cmdheight
        assert!(rs_validate_cmdheight(0).is_null());
        assert!(rs_validate_cmdheight(5).is_null());
        assert!(!rs_validate_cmdheight(-1).is_null());

        // columns
        assert!(rs_validate_columns(12).is_null());
        assert!(rs_validate_columns(80).is_null());
        assert!(!rs_validate_columns(11).is_null());

        // lines
        assert!(rs_validate_lines(2).is_null());
        assert!(rs_validate_lines(24).is_null());
        assert!(!rs_validate_lines(1).is_null());

        // laststatus
        assert!(rs_validate_laststatus(0).is_null());
        assert!(rs_validate_laststatus(3).is_null());
        assert!(!rs_validate_laststatus(4).is_null());

        // showtabline
        assert!(rs_validate_showtabline(0).is_null());
        assert!(rs_validate_showtabline(2).is_null());
        assert!(!rs_validate_showtabline(3).is_null());

        // tabstop
        assert!(rs_validate_tabstop(1).is_null());
        assert!(rs_validate_tabstop(8).is_null());
        assert!(!rs_validate_tabstop(0).is_null());
        assert!(!rs_validate_tabstop(10000).is_null());

        // softtabstop
        assert!(rs_validate_softtabstop(-1).is_null());
        assert!(rs_validate_softtabstop(0).is_null());
        assert!(rs_validate_softtabstop(4).is_null());
        assert!(!rs_validate_softtabstop(-2).is_null());

        // Note: percentage options (pumblend, winblend) are tested in lib.rs

        // undolevels
        assert!(rs_validate_undolevels(-1).is_null());
        assert!(rs_validate_undolevels(0).is_null());
        assert!(rs_validate_undolevels(1000).is_null());
        assert!(!rs_validate_undolevels(-2).is_null());
    }

    #[test]
    fn test_parse_int() {
        unsafe {
            let pos = CString::new("123").unwrap();
            let result = rs_parse_int(pos.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.value, 123);
            assert_eq!(result.consumed, 3);

            let neg = CString::new("-456").unwrap();
            let result = rs_parse_int(neg.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.value, -456);
            assert_eq!(result.consumed, 4);

            let with_plus = CString::new("+789").unwrap();
            let result = rs_parse_int(with_plus.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.value, 789);
            assert_eq!(result.consumed, 4);

            let invalid = CString::new("abc").unwrap();
            let result = rs_parse_int(invalid.as_ptr());
            assert_eq!(result.error, 1);

            let empty = CString::new("").unwrap();
            let result = rs_parse_int(empty.as_ptr());
            assert_eq!(result.error, 1);
        }
    }
}
