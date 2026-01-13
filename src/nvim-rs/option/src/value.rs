//! Option value processing functions
//!
//! This module provides Rust implementations for option value construction,
//! conversion, and bounds checking. These functions are used during :set
//! command processing to create and validate new option values.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::OptInt;

// =============================================================================
// Type Definitions
// =============================================================================

/// Option value type enum (matches kOptValType* in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptValType {
    Nil = -1,
    Boolean = 0,
    Number = 1,
    String = 2,
}

/// Bounds check result
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BoundsCheckResult {
    /// Adjusted value (may be clamped to bounds)
    pub value: OptInt,
    /// Error message (NULL if no error)
    pub errmsg: *const c_char,
    /// Whether value was modified
    pub modified: c_int,
}

impl Default for BoundsCheckResult {
    fn default() -> Self {
        Self {
            value: 0,
            errmsg: ptr::null(),
            modified: 0,
        }
    }
}

// =============================================================================
// Error Messages
// =============================================================================

static E_POSITIVE: &[u8] = b"E487: Argument must be positive\0";
static E_INVARG: &[u8] = b"E474: Invalid argument\0";
static E_SCROLL: &[u8] = b"E49: Invalid scroll size\0";
static E_WINHEIGHT: &[u8] = b"E591: 'winheight' cannot be smaller than 'winminheight'\0";
static E_WINWIDTH: &[u8] = b"E592: 'winwidth' cannot be smaller than 'winminwidth'\0";

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    // Full screen state (option module specific)
    fn nvim_option_get_full_screen() -> c_int;

    // Screen dimensions
    fn nvim_option_get_rows() -> c_int;

    // Window accessor for view height
    fn nvim_option_win_get_view_height(win: *mut std::ffi::c_void) -> c_int;

    // Global option value accessors
    fn nvim_option_get_p_wmh() -> OptInt;
    fn nvim_option_get_p_wh() -> OptInt;
    fn nvim_option_get_p_wmw() -> OptInt;
    fn nvim_option_get_p_wiw() -> OptInt;

    // Min rows for all tabpages
    fn min_rows_for_all_tabpages() -> c_int;

    // Window default scroll
    fn win_default_scroll(win: *mut std::ffi::c_void) -> OptInt;
}

// =============================================================================
// Boolean Value Processing
// =============================================================================

/// Compute new boolean value based on prefix and current value.
///
/// Handles:
/// - `:set opt` -> true
/// - `:set noopt` -> false
/// - `:set invopt` -> toggle
/// - `:set opt!` -> toggle (nextchar == '!')
#[no_mangle]
pub extern "C" fn rs_compute_boolean_newval(
    current: c_int,
    prefix: c_int,
    nextchar: c_int,
) -> c_int {
    // `:set opt!` - invert
    if nextchar == i32::from(b'!') {
        return i32::from(current == 0);
    }

    // Handle prefix
    match prefix {
        0 => 0,                       // PREFIX_NO -> false
        2 => i32::from(current == 0), // PREFIX_INV -> toggle
        _ => 1,                       // PREFIX_NONE -> true
    }
}

/// Compute new TriState value for tri-state boolean options.
#[no_mangle]
pub extern "C" fn rs_compute_tristate_newval(
    current: c_int,
    prefix: c_int,
    nextchar: c_int,
) -> c_int {
    // `:set opt!` - invert (kNone stays kNone)
    if nextchar == i32::from(b'!') {
        return match current {
            0 => 0, // kNone -> kNone (stays unchanged)
            1 => 2, // kFalse -> kTrue
            2 => 1, // kTrue -> kFalse
            _ => current,
        };
    }

    // Handle prefix
    match prefix {
        0 => 1, // PREFIX_NO -> kFalse
        2 => {
            if current == 2 {
                1
            } else {
                2
            }
        } // PREFIX_INV -> toggle
        _ => 2, // PREFIX_NONE -> kTrue
    }
}

// =============================================================================
// Number Value Processing
// =============================================================================

/// Apply numeric operator to compute new value.
///
/// Operators:
/// - OP_NONE (0): newval = operand
/// - OP_ADDING (1): newval = oldval + operand
/// - OP_PREPENDING (2): newval = oldval * operand
/// - OP_REMOVING (3): newval = oldval - operand
#[no_mangle]
pub extern "C" fn rs_apply_number_op(oldval: OptInt, operand: OptInt, op: c_int) -> OptInt {
    match op {
        1 => oldval + operand, // OP_ADDING
        2 => oldval * operand, // OP_PREPENDING
        3 => oldval - operand, // OP_REMOVING
        _ => operand,          // OP_NONE
    }
}

// =============================================================================
// Number Bounds Checking
// =============================================================================

/// Constants for number validation
const MIN_COLUMNS: OptInt = 12;
const INT_MAX: OptInt = i32::MAX as OptInt;

/// Check and bound 'lines' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_lines_bounds(
    value: OptInt,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    let full_screen = nvim_option_get_full_screen() != 0;
    let min_rows = OptInt::from(min_rows_for_all_tabpages());

    if value < min_rows && full_screen {
        // Format error message
        if !errbuf.is_null() && errbuflen > 0 {
            let msg = format!("E593: Need at least {min_rows} lines\0");
            let bytes = msg.as_bytes();
            let copy_len = bytes.len().min(errbuflen - 1);
            ptr::copy_nonoverlapping(bytes.as_ptr(), errbuf.cast::<u8>(), copy_len);
            *errbuf.add(copy_len) = 0;
            result.errmsg = errbuf;
        }
        result.value = min_rows;
        result.modified = 1;
    }

    // Clamp to INT_MAX
    if result.value > INT_MAX {
        result.value = INT_MAX;
        result.modified = 1;
    }

    result
}

/// Check and bound 'columns' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_columns_bounds(
    value: OptInt,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    let full_screen = nvim_option_get_full_screen() != 0;

    if value < MIN_COLUMNS && full_screen {
        // Format error message
        if !errbuf.is_null() && errbuflen > 0 {
            let msg = format!("E594: Need at least {MIN_COLUMNS} columns\0");
            let bytes = msg.as_bytes();
            let copy_len = bytes.len().min(errbuflen - 1);
            ptr::copy_nonoverlapping(bytes.as_ptr(), errbuf.cast::<u8>(), copy_len);
            *errbuf.add(copy_len) = 0;
            result.errmsg = errbuf;
        }
        result.value = MIN_COLUMNS;
        result.modified = 1;
    }

    // Clamp to INT_MAX
    if result.value > INT_MAX {
        result.value = INT_MAX;
        result.modified = 1;
    }

    result
}

/// Check and bound 'pumblend' option value (0-100).
#[no_mangle]
pub extern "C" fn rs_check_pumblend_bounds(value: OptInt) -> BoundsCheckResult {
    let clamped = value.clamp(0, 100);
    BoundsCheckResult {
        value: clamped,
        errmsg: ptr::null(),
        modified: i32::from(clamped != value),
    }
}

/// Check and bound 'scrolljump' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_scrolljump_bounds(value: OptInt) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    let full_screen = nvim_option_get_full_screen() != 0;
    let rows = OptInt::from(nvim_option_get_rows());

    if full_screen && (value < -100 || value >= rows) {
        result.errmsg = E_SCROLL.as_ptr().cast();
        result.value = 1;
        result.modified = 1;
    }

    result
}

/// Check and bound 'scroll' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_scroll_bounds(
    value: OptInt,
    win: *mut std::ffi::c_void,
) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    let full_screen = nvim_option_get_full_screen() != 0;
    let view_height = if win.is_null() {
        0
    } else {
        OptInt::from(nvim_option_win_get_view_height(win))
    };

    if full_screen && (value <= 0 || (value > view_height && view_height > 0)) {
        if value != 0 {
            result.errmsg = E_SCROLL.as_ptr().cast();
        }
        result.value = if win.is_null() {
            value
        } else {
            win_default_scroll(win)
        };
        result.modified = 1;
    }

    result
}

// =============================================================================
// Number Validation
// =============================================================================

/// Validate numeric option value.
/// Returns error message or NULL if valid.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_num_option(opt_idx: c_int, value: OptInt) -> *const c_char {
    // Check INT range
    if value < OptInt::from(i32::MIN) || value > OptInt::from(i32::MAX) {
        return E_INVARG.as_ptr().cast();
    }

    // Option-specific validation is handled through option indices
    // These match kOpt* enum values
    match opt_idx {
        // Options that must be >= 0
        idx if is_nonnegative_option(idx) => {
            if value < 0 {
                E_POSITIVE.as_ptr().cast()
            } else {
                ptr::null()
            }
        }
        // Options that must be > 0
        idx if is_positive_option(idx) => {
            if value <= 0 {
                E_POSITIVE.as_ptr().cast()
            } else {
                ptr::null()
            }
        }
        // Default: no validation
        _ => ptr::null(),
    }
}

/// Check if option index requires non-negative value.
fn is_nonnegative_option(_idx: c_int) -> bool {
    // This would need to match against specific option indices
    // For now, return false and let C handle specific validation
    false
}

/// Check if option index requires positive value.
fn is_positive_option(_idx: c_int) -> bool {
    // This would need to match against specific option indices
    false
}

/// Validate 'winheight' against 'winminheight' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winheight_cross(value: OptInt) -> *const c_char {
    if value < 1 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wmh = nvim_option_get_p_wmh();
    if wmh > value {
        return E_WINHEIGHT.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'winminheight' against 'winheight' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winminheight_cross(value: OptInt) -> *const c_char {
    if value < 0 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wh = nvim_option_get_p_wh();
    if value > wh {
        return E_WINHEIGHT.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'winwidth' against 'winminwidth' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winwidth_cross(value: OptInt) -> *const c_char {
    if value < 1 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wmw = nvim_option_get_p_wmw();
    if wmw > value {
        return E_WINWIDTH.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'winminwidth' against 'winwidth' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winminwidth_cross(value: OptInt) -> *const c_char {
    if value < 0 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wiw = nvim_option_get_p_wiw();
    if value > wiw {
        return E_WINWIDTH.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'history' option value with bounds.
#[no_mangle]
pub extern "C" fn rs_validate_history_bounds(value: OptInt) -> *const c_char {
    if value < 0 {
        E_POSITIVE.as_ptr().cast()
    } else if value > 10000 {
        E_INVARG.as_ptr().cast()
    } else {
        ptr::null()
    }
}

/// Validate 'regexpengine' option value with bounds.
#[no_mangle]
pub extern "C" fn rs_validate_regexpengine_bounds(value: OptInt) -> *const c_char {
    if (0..=2).contains(&value) {
        ptr::null()
    } else {
        E_INVARG.as_ptr().cast()
    }
}

// =============================================================================
// String Option Helpers
// =============================================================================

/// Check if a string option value is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_string_option_is_empty(val: *const c_char) -> c_int {
    i32::from(val.is_null() || *val == 0)
}

/// Check string against empty option value.
/// Used by check_string_option to replace NULL with empty string.
#[no_mangle]
pub unsafe extern "C" fn rs_check_string_option(valp: *mut *mut c_char, empty: *mut c_char) {
    if valp.is_null() {
        return;
    }

    if (*valp).is_null() {
        *valp = empty;
    }
}

// =============================================================================
// Option Reset Helpers
// =============================================================================

/// Check if nextchar indicates reset to default ('&').
#[no_mangle]
pub extern "C" fn rs_is_reset_to_default(nextchar: c_int) -> c_int {
    i32::from(nextchar == i32::from(b'&'))
}

/// Check if nextchar indicates reset to global ('<').
#[no_mangle]
pub extern "C" fn rs_is_reset_to_global(nextchar: c_int) -> c_int {
    i32::from(nextchar == i32::from(b'<'))
}

/// Check if nextchar indicates invert ('!').
#[no_mangle]
pub extern "C" fn rs_is_invert(nextchar: c_int) -> c_int {
    i32::from(nextchar == i32::from(b'!'))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_boolean_newval() {
        // PREFIX_NONE (1) -> true
        assert_eq!(rs_compute_boolean_newval(0, 1, 0), 1);
        assert_eq!(rs_compute_boolean_newval(1, 1, 0), 1);

        // PREFIX_NO (0) -> false
        assert_eq!(rs_compute_boolean_newval(0, 0, 0), 0);
        assert_eq!(rs_compute_boolean_newval(1, 0, 0), 0);

        // PREFIX_INV (2) -> toggle
        assert_eq!(rs_compute_boolean_newval(0, 2, 0), 1);
        assert_eq!(rs_compute_boolean_newval(1, 2, 0), 0);

        // '!' -> toggle
        assert_eq!(rs_compute_boolean_newval(0, 1, i32::from(b'!')), 1);
        assert_eq!(rs_compute_boolean_newval(1, 1, i32::from(b'!')), 0);
    }

    #[test]
    fn test_apply_number_op() {
        // OP_NONE
        assert_eq!(rs_apply_number_op(10, 5, 0), 5);

        // OP_ADDING
        assert_eq!(rs_apply_number_op(10, 5, 1), 15);

        // OP_PREPENDING (multiply)
        assert_eq!(rs_apply_number_op(10, 5, 2), 50);

        // OP_REMOVING
        assert_eq!(rs_apply_number_op(10, 5, 3), 5);
    }

    #[test]
    fn test_check_pumblend_bounds() {
        let result = rs_check_pumblend_bounds(50);
        assert_eq!(result.value, 50);
        assert_eq!(result.modified, 0);

        let result = rs_check_pumblend_bounds(-10);
        assert_eq!(result.value, 0);
        assert_eq!(result.modified, 1);

        let result = rs_check_pumblend_bounds(150);
        assert_eq!(result.value, 100);
        assert_eq!(result.modified, 1);
    }

    #[test]
    fn test_validate_history_bounds() {
        assert!(rs_validate_history_bounds(0).is_null());
        assert!(rs_validate_history_bounds(500).is_null());
        assert!(rs_validate_history_bounds(10000).is_null());
        assert!(!rs_validate_history_bounds(-1).is_null());
        assert!(!rs_validate_history_bounds(10001).is_null());
    }

    #[test]
    fn test_validate_regexpengine_bounds() {
        assert!(rs_validate_regexpengine_bounds(0).is_null());
        assert!(rs_validate_regexpengine_bounds(1).is_null());
        assert!(rs_validate_regexpengine_bounds(2).is_null());
        assert!(!rs_validate_regexpengine_bounds(-1).is_null());
        assert!(!rs_validate_regexpengine_bounds(3).is_null());
    }

    #[test]
    fn test_nextchar_helpers() {
        assert_eq!(rs_is_reset_to_default(i32::from(b'&')), 1);
        assert_eq!(rs_is_reset_to_default(i32::from(b'=')), 0);

        assert_eq!(rs_is_reset_to_global(i32::from(b'<')), 1);
        assert_eq!(rs_is_reset_to_global(i32::from(b'=')), 0);

        assert_eq!(rs_is_invert(i32::from(b'!')), 1);
        assert_eq!(rs_is_invert(i32::from(b'=')), 0);
    }
}
