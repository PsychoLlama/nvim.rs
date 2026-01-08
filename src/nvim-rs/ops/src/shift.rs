//! Shift operations (< and >)
//!
//! This module implements indent shifting logic used by the `<` and `>`
//! operators. This includes both the simple shiftwidth-based indentation
//! and the more complex variable tabstop (`'vartabstop'`) support.

use std::ffi::c_int;

// =============================================================================
// Variable Tabstop FFI Functions
// =============================================================================

extern "C" {
    /// Get the tabstop width at a given index in the vartabstop array.
    /// If index > array length, returns the last tabstop width.
    fn nvim_get_vts(vts_array: *const c_int, index: c_int) -> c_int;

    /// Get the sum of all tabstops through the index-th.
    fn nvim_get_vts_sum(vts_array: *const c_int, index: c_int) -> c_int;
}

// =============================================================================
// Indentation Calculation
// =============================================================================

/// Calculate new indentation when shifting with simple shiftwidth.
///
/// This implements the `get_new_sw_indent` logic from ops.c.
///
/// # Arguments
/// * `left` - true if shifting left (<), false if shifting right (>)
/// * `round` - true if 'shiftround' is set
/// * `amount` - number of shift operations
/// * `sw_val` - shiftwidth value
/// * `current_indent` - current indentation in spaces
///
/// # Returns
/// The new indentation value (always >= 0)
#[must_use]
pub fn calc_new_indent(
    left: bool,
    round: bool,
    amount: i64,
    sw_val: i64,
    current_indent: i64,
) -> i64 {
    if sw_val == 0 {
        return current_indent;
    }

    if round {
        // Round off indent
        let i = current_indent / sw_val; // Number of 'shiftwidth' rounded down
        let j = current_indent % sw_val; // Extra spaces
        let mut amount = amount;

        // First remove extra spaces when shifting left
        if j != 0 && left {
            amount -= 1;
        }

        let new_units = if left {
            (i - amount).max(0)
        } else {
            i + amount
        };

        new_units * sw_val
    } else {
        // Original vi indent
        if left {
            (current_indent - sw_val * amount).max(0)
        } else {
            current_indent + sw_val * amount
        }
    }
}

/// Calculate new indentation when shifting with variable tabstops.
///
/// This implements the `get_new_vts_indent` logic from ops.c.
/// Variable tabstops allow different tab widths at different positions.
///
/// # Arguments
/// * `left` - true if shifting left (<), false if shifting right (>)
/// * `round` - true if 'shiftround' is set
/// * `amount` - number of shift operations
/// * `current_indent` - current indentation in spaces
/// * `vts_array` - pointer to variable tabstop array (first element is length)
///
/// # Returns
/// The new indentation value (always >= 0)
///
/// # Safety
/// `vts_array` must be a valid pointer to a vartabstop array or null.
#[must_use]
pub unsafe fn calc_new_vts_indent(
    left: bool,
    round: bool,
    amount: c_int,
    current_indent: i64,
    vts_array: *const c_int,
) -> i64 {
    if vts_array.is_null() {
        return current_indent;
    }

    let mut vtsi: c_int = 0;
    let mut vts_indent: i64 = 0;
    let mut ts: c_int;

    // Find the tabstop at or to the left of the current indent.
    loop {
        vtsi += 1;
        ts = nvim_get_vts(vts_array, vtsi);
        vts_indent += i64::from(ts);
        if vts_indent > current_indent {
            break;
        }
    }
    vts_indent -= i64::from(ts);
    vtsi -= 1;

    // Extra indent spaces to the right of the tabstop
    let offset = current_indent - vts_indent;

    let indent = if round {
        if left {
            if offset == 0 {
                i64::from(nvim_get_vts_sum(vts_array, vtsi - amount))
            } else {
                i64::from(nvim_get_vts_sum(vts_array, vtsi - (amount - 1)))
            }
        } else {
            i64::from(nvim_get_vts_sum(vts_array, vtsi + amount))
        }
    } else if left {
        if amount > vtsi {
            0
        } else {
            i64::from(nvim_get_vts_sum(vts_array, vtsi - amount)) + offset
        }
    } else {
        i64::from(nvim_get_vts_sum(vts_array, vtsi + amount)) + offset
    };

    indent.max(0)
}

// =============================================================================
// Block Shift Calculations
// =============================================================================

/// Calculate total screen columns to shift for a block operation.
///
/// # Arguments
/// * `amount` - number of shift operations (must be >= 0)
/// * `sw_val` - shiftwidth value (must be >= 0)
///
/// # Returns
/// Total screen columns to shift, or None if overflow occurred or inputs are negative.
#[must_use]
#[inline]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub const fn calc_block_shift_total(amount: c_int, sw_val: c_int) -> Option<c_int> {
    // Negative values are invalid
    if amount < 0 || sw_val < 0 {
        return None;
    }

    // Safe casts since we've checked for negative values
    let total = (amount as u32).wrapping_mul(sw_val as u32);
    if total > c_int::MAX as u32 {
        return None;
    }
    let total = total as c_int;

    // Verify no overflow: (total / sw_val) should equal amount
    if sw_val != 0 && (total / sw_val) != amount {
        return None;
    }

    Some(total)
}

/// Calculate the number of tabs and spaces needed for indentation.
///
/// Given a starting virtual column and total whitespace to fill,
/// calculates how many tabs and spaces are needed based on tabstop settings.
///
/// # Arguments
/// * `start_vcol` - starting virtual column
/// * `end_vcol` - ending virtual column (start + total whitespace)
/// * `ts_val` - tabstop value
/// * `use_expandtab` - whether 'expandtab' is set
///
/// # Returns
/// `(tabs, spaces)` - number of tabs and spaces needed
#[must_use]
#[inline]
pub const fn calc_tabs_and_spaces(
    start_vcol: c_int,
    end_vcol: c_int,
    ts_val: c_int,
    use_expandtab: bool,
) -> (c_int, c_int) {
    if use_expandtab || ts_val == 0 {
        // All spaces when expandtab is set
        let total = end_vcol - start_vcol;
        (0, if total < 0 { 0 } else { total })
    } else {
        // Calculate tabs to reach next tabstop, then fill with more tabs
        let end_tab = end_vcol / ts_val;
        let start_tab = (start_vcol + ts_val - 1) / ts_val; // Round up
        let tabs = if end_tab > start_tab {
            end_tab - start_tab
        } else {
            0
        };
        let space_start = if tabs > 0 {
            end_tab * ts_val
        } else {
            start_vcol
        };
        let spaces = end_vcol - space_start;
        (tabs, if spaces < 0 { 0 } else { spaces })
    }
}

/// Calculate the shift amount for left block shift.
///
/// When shifting a block left, we can only shift as much as the
/// available whitespace in the block allows.
///
/// # Arguments
/// * `block_space_width` - total whitespace width available in block
/// * `total` - total shift amount requested
///
/// # Returns
/// The actual shift amount (minimum of the two)
#[must_use]
#[inline]
pub const fn calc_left_shift_amount(block_space_width: c_int, total: c_int) -> c_int {
    if block_space_width < total {
        block_space_width
    } else {
        total
    }
}

/// Calculate destination column after left shift.
///
/// # Arguments
/// * `non_white_col` - column of first non-whitespace char in block
/// * `shift_amount` - amount to shift left
///
/// # Returns
/// Destination column for the shifted text
#[must_use]
#[inline]
pub const fn calc_left_shift_destination(non_white_col: c_int, shift_amount: c_int) -> c_int {
    let dest = non_white_col - shift_amount;
    if dest < 0 {
        0
    } else {
        dest
    }
}

/// Calculate fill spaces needed when a tab is partially replaced.
///
/// When shifting left causes a tab to be split, we need spaces to fill
/// the gap between the verbatim copy end and the destination column.
///
/// # Arguments
/// * `destination_col` - target column for shifted text
/// * `verbatim_copy_width` - width of text copied verbatim
///
/// # Returns
/// Number of fill spaces needed (always >= 0)
#[must_use]
#[inline]
pub const fn calc_fill_spaces(destination_col: c_int, verbatim_copy_width: c_int) -> c_int {
    let fill = destination_col - verbatim_copy_width;
    if fill < 0 {
        0
    } else {
        fill
    }
}

/// Check if a line should be skipped during shift operation.
///
/// Empty lines and (optionally) lines starting with '#' may be skipped.
///
/// # Arguments
/// * `first_char` - first character of the line (0 for empty)
/// * `skip_preproc` - whether to skip preprocessor lines ('#')
///
/// # Returns
/// true if the line should be skipped
#[must_use]
#[inline]
pub const fn should_skip_shift_line(first_char: u8, skip_preproc: bool) -> bool {
    first_char == 0 || (first_char == b'#' && skip_preproc)
}

// =============================================================================
// FFI Wrappers
// =============================================================================

/// FFI wrapper for calc_new_indent.
#[no_mangle]
pub extern "C" fn rs_calc_new_indent(
    left: c_int,
    round: c_int,
    amount: i64,
    sw_val: i64,
    current_indent: i64,
) -> i64 {
    calc_new_indent(left != 0, round != 0, amount, sw_val, current_indent)
}

/// FFI wrapper for calc_new_vts_indent.
///
/// # Safety
/// `vts_array` must be a valid pointer to a vartabstop array or null.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_new_vts_indent(
    left: c_int,
    round: c_int,
    amount: c_int,
    current_indent: i64,
    vts_array: *const c_int,
) -> i64 {
    calc_new_vts_indent(left != 0, round != 0, amount, current_indent, vts_array)
}

/// FFI wrapper for calc_block_shift_total.
///
/// Returns -1 on overflow.
#[no_mangle]
pub extern "C" fn rs_calc_block_shift_total(amount: c_int, sw_val: c_int) -> c_int {
    calc_block_shift_total(amount, sw_val).unwrap_or(-1)
}

/// FFI wrapper for calc_tabs_and_spaces.
///
/// # Safety
/// `out_tabs` and `out_spaces` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_tabs_and_spaces(
    start_vcol: c_int,
    end_vcol: c_int,
    ts_val: c_int,
    use_expandtab: c_int,
    out_tabs: *mut c_int,
    out_spaces: *mut c_int,
) {
    let (tabs, spaces) = calc_tabs_and_spaces(start_vcol, end_vcol, ts_val, use_expandtab != 0);
    if !out_tabs.is_null() {
        *out_tabs = tabs;
    }
    if !out_spaces.is_null() {
        *out_spaces = spaces;
    }
}

/// FFI wrapper for calc_left_shift_amount.
#[no_mangle]
pub extern "C" fn rs_calc_left_shift_amount(block_space_width: c_int, total: c_int) -> c_int {
    calc_left_shift_amount(block_space_width, total)
}

/// FFI wrapper for calc_left_shift_destination.
#[no_mangle]
pub extern "C" fn rs_calc_left_shift_destination(
    non_white_col: c_int,
    shift_amount: c_int,
) -> c_int {
    calc_left_shift_destination(non_white_col, shift_amount)
}

/// FFI wrapper for calc_fill_spaces.
#[no_mangle]
pub extern "C" fn rs_calc_fill_spaces(destination_col: c_int, verbatim_copy_width: c_int) -> c_int {
    calc_fill_spaces(destination_col, verbatim_copy_width)
}

/// FFI wrapper for should_skip_shift_line.
#[no_mangle]
pub extern "C" fn rs_should_skip_shift_line(first_char: u8, skip_preproc: c_int) -> c_int {
    c_int::from(should_skip_shift_line(first_char, skip_preproc != 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // calc_new_indent tests
    // =========================================================================

    #[test]
    fn test_shift_right_no_round() {
        // Shift right without rounding
        assert_eq!(calc_new_indent(false, false, 1, 4, 0), 4);
        assert_eq!(calc_new_indent(false, false, 1, 4, 4), 8);
        assert_eq!(calc_new_indent(false, false, 2, 4, 0), 8);
        assert_eq!(calc_new_indent(false, false, 1, 2, 3), 5);
    }

    #[test]
    fn test_shift_left_no_round() {
        // Shift left without rounding
        assert_eq!(calc_new_indent(true, false, 1, 4, 8), 4);
        assert_eq!(calc_new_indent(true, false, 1, 4, 4), 0);
        assert_eq!(calc_new_indent(true, false, 2, 4, 8), 0);
        // Cannot go negative
        assert_eq!(calc_new_indent(true, false, 1, 4, 2), 0);
        assert_eq!(calc_new_indent(true, false, 1, 4, 0), 0);
    }

    #[test]
    fn test_shift_right_with_round() {
        // Shift right with rounding to shiftwidth boundary
        // When shifting right, extra spaces don't affect the amount
        // i = count / sw_val (integer division)
        // i += amount
        // result = i * sw_val
        assert_eq!(calc_new_indent(false, true, 1, 4, 0), 4); // 0/4=0, 0+1=1, 1*4=4
        assert_eq!(calc_new_indent(false, true, 1, 4, 2), 4); // 2/4=0, 0+1=1, 1*4=4
        assert_eq!(calc_new_indent(false, true, 1, 4, 4), 8); // 4/4=1, 1+1=2, 2*4=8
        assert_eq!(calc_new_indent(false, true, 1, 4, 5), 8); // 5/4=1, 1+1=2, 2*4=8
    }

    #[test]
    fn test_shift_left_with_round() {
        // Shift left with rounding - first removes extra spaces
        assert_eq!(calc_new_indent(true, true, 1, 4, 8), 4);
        assert_eq!(calc_new_indent(true, true, 1, 4, 4), 0);
        // Extra spaces: 6 = 1*4 + 2, so first shift removes the 2 extra -> 4
        assert_eq!(calc_new_indent(true, true, 1, 4, 6), 4);
        // After removing extra, continues shifting
        assert_eq!(calc_new_indent(true, true, 2, 4, 6), 0);
    }

    #[test]
    fn test_shift_zero_shiftwidth() {
        // Zero shiftwidth should return current indent unchanged
        assert_eq!(calc_new_indent(false, false, 1, 0, 4), 4);
        assert_eq!(calc_new_indent(true, true, 1, 0, 4), 4);
    }

    #[test]
    fn test_shift_large_amount() {
        // Large shift amounts
        assert_eq!(calc_new_indent(false, false, 10, 4, 0), 40);
        // Cannot go below 0
        assert_eq!(calc_new_indent(true, false, 10, 4, 8), 0);
    }

    // =========================================================================
    // calc_block_shift_total tests
    // =========================================================================

    #[test]
    fn test_calc_block_shift_total() {
        // Normal cases
        assert_eq!(calc_block_shift_total(1, 4), Some(4));
        assert_eq!(calc_block_shift_total(2, 4), Some(8));
        assert_eq!(calc_block_shift_total(5, 8), Some(40));

        // Edge cases
        assert_eq!(calc_block_shift_total(0, 4), Some(0));
        assert_eq!(calc_block_shift_total(1, 0), Some(0));
    }

    // =========================================================================
    // calc_tabs_and_spaces tests
    // =========================================================================

    #[test]
    fn test_calc_tabs_and_spaces_expandtab() {
        // With expandtab, all spaces
        assert_eq!(calc_tabs_and_spaces(0, 8, 4, true), (0, 8));
        assert_eq!(calc_tabs_and_spaces(2, 10, 4, true), (0, 8));
    }

    #[test]
    fn test_calc_tabs_and_spaces_no_expandtab() {
        // Without expandtab, mix of tabs and spaces
        // From 0 to 8 with ts=4: 2 tabs
        assert_eq!(calc_tabs_and_spaces(0, 8, 4, false), (2, 0));
        // From 0 to 10 with ts=4: 2 tabs + 2 spaces
        assert_eq!(calc_tabs_and_spaces(0, 10, 4, false), (2, 2));
        // From 2 to 10 with ts=4: need to reach 4, then 8, then 2 spaces
        // start_tab = (2+4-1)/4 = 1, end_tab = 10/4 = 2
        // tabs = 2 - 1 = 1, space_start = 8, spaces = 10-8 = 2
        assert_eq!(calc_tabs_and_spaces(2, 10, 4, false), (1, 2));
    }

    #[test]
    fn test_calc_tabs_and_spaces_zero_ts() {
        // Zero tabstop should give all spaces
        assert_eq!(calc_tabs_and_spaces(0, 8, 0, false), (0, 8));
    }

    // =========================================================================
    // calc_left_shift_amount tests
    // =========================================================================

    #[test]
    fn test_calc_left_shift_amount() {
        // Shift less than available space
        assert_eq!(calc_left_shift_amount(10, 4), 4);
        // Shift more than available space
        assert_eq!(calc_left_shift_amount(4, 10), 4);
        // Equal
        assert_eq!(calc_left_shift_amount(8, 8), 8);
    }

    // =========================================================================
    // calc_left_shift_destination tests
    // =========================================================================

    #[test]
    fn test_calc_left_shift_destination() {
        // Normal case
        assert_eq!(calc_left_shift_destination(10, 4), 6);
        // Shift to 0
        assert_eq!(calc_left_shift_destination(4, 4), 0);
        // Would go negative (clamped to 0)
        assert_eq!(calc_left_shift_destination(4, 10), 0);
    }

    // =========================================================================
    // calc_fill_spaces tests
    // =========================================================================

    #[test]
    fn test_calc_fill_spaces() {
        // Need fill spaces
        assert_eq!(calc_fill_spaces(10, 6), 4);
        // No fill needed
        assert_eq!(calc_fill_spaces(10, 10), 0);
        // Would be negative (clamped to 0)
        assert_eq!(calc_fill_spaces(5, 10), 0);
    }

    // =========================================================================
    // should_skip_shift_line tests
    // =========================================================================

    #[test]
    fn test_should_skip_shift_line() {
        // Empty line (first_char == 0)
        assert!(should_skip_shift_line(0, false));
        assert!(should_skip_shift_line(0, true));

        // Preprocessor line with skip enabled
        assert!(should_skip_shift_line(b'#', true));
        // Preprocessor line with skip disabled
        assert!(!should_skip_shift_line(b'#', false));

        // Normal line
        assert!(!should_skip_shift_line(b'a', false));
        assert!(!should_skip_shift_line(b'a', true));
        assert!(!should_skip_shift_line(b' ', false));
    }

    // =========================================================================
    // FFI wrapper tests
    // =========================================================================

    #[test]
    fn test_ffi_wrappers() {
        // rs_calc_new_indent
        assert_eq!(rs_calc_new_indent(0, 0, 1, 4, 0), 4);
        assert_eq!(rs_calc_new_indent(1, 0, 1, 4, 8), 4);

        // rs_calc_block_shift_total
        assert_eq!(rs_calc_block_shift_total(2, 4), 8);

        // rs_calc_left_shift_amount
        assert_eq!(rs_calc_left_shift_amount(10, 4), 4);

        // rs_calc_left_shift_destination
        assert_eq!(rs_calc_left_shift_destination(10, 4), 6);

        // rs_calc_fill_spaces
        assert_eq!(rs_calc_fill_spaces(10, 6), 4);

        // rs_should_skip_shift_line
        assert_eq!(rs_should_skip_shift_line(0, 0), 1);
        assert_eq!(rs_should_skip_shift_line(b'a', 0), 0);
    }
}
