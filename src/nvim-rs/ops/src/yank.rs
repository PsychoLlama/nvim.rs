//! Yank operations (y)
//!
//! This module implements calculation logic for the `y` operator
//! (yank/copy text to register).

use std::ffi::c_int;

use crate::types::MotionType;

// =============================================================================
// Core Yank Logic Helpers
// =============================================================================

/// Check if a charwise yank should be converted to linewise.
///
/// This happens when:
/// - Motion is charwise
/// - Cursor was at column 0 before and after movement
/// - Motion is not inclusive
/// - Not visual mode, or 'selection' is 'old'
/// - End column is 0
/// - More than one line
///
/// # Arguments
/// * `motion_type` - Current motion type
/// * `start_col` - Starting column
/// * `end_col` - Ending column
/// * `inclusive` - Whether the motion is inclusive
/// * `is_visual` - Whether in visual mode
/// * `selection_old` - Whether 'selection' option is 'old'
/// * `line_count` - Number of lines affected
///
/// # Returns
/// `(should_convert, adjusted_line_count)` - Whether to convert and new line count
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub const fn should_convert_yank_to_linewise(
    motion_type: MotionType,
    start_col: c_int,
    end_col: c_int,
    inclusive: bool,
    is_visual: bool,
    selection_old: bool,
    line_count: c_int,
) -> (bool, c_int) {
    // From register.c:
    // if (oap->motion_type == kMTCharWise
    //     && oap->start.col == 0
    //     && !oap->inclusive
    //     && (!oap->is_VIsual || *p_sel == 'o')
    //     && oap->end.col == 0
    //     && yanklines > 1)
    let should_convert = matches!(motion_type, MotionType::CharWise)
        && start_col == 0
        && !inclusive
        && (!is_visual || selection_old)
        && end_col == 0
        && line_count > 1;

    if should_convert {
        (true, line_count - 1)
    } else {
        (false, line_count)
    }
}

/// Calculate block width for visual block yank.
///
/// # Arguments
/// * `start_vcol` - Start virtual column
/// * `end_vcol` - End virtual column
/// * `curswant_is_maxcol` - Whether curswant is MAXCOL
///
/// # Returns
/// Block width for register
#[must_use]
#[inline]
pub const fn calc_yank_block_width(
    start_vcol: c_int,
    end_vcol: c_int,
    curswant_is_maxcol: bool,
) -> c_int {
    let mut width = end_vcol - start_vcol;

    // If at end of line (MAXCOL), decrement width if positive
    if curswant_is_maxcol && width > 0 {
        width -= 1;
    }

    // Width should be at least 0
    if width < 0 {
        0
    } else {
        width
    }
}

/// Check if yank message should be displayed.
///
/// For charwise yank of a single line, no message is shown.
/// Message is shown when yanklines > p_report.
///
/// # Arguments
/// * `yank_type` - The yank motion type
/// * `line_count` - Number of lines yanked
/// * `report_threshold` - Value of 'report' option
///
/// # Returns
/// `(should_show, adjusted_count)` - Whether to show message and line count for message
#[must_use]
#[inline]
pub const fn should_show_yank_message(
    yank_type: MotionType,
    line_count: c_int,
    report_threshold: c_int,
) -> (bool, c_int) {
    // For charwise single-line yank, effective count is 0 (no message)
    let effective_count = if matches!(yank_type, MotionType::CharWise) && line_count == 1 {
        0
    } else {
        line_count
    };

    (effective_count > report_threshold, effective_count)
}

/// Calculate the size of concatenated string for append register.
///
/// When appending to a charwise register, the last line of the old
/// content is concatenated with the first line of the new content.
///
/// # Arguments
/// * `old_last_size` - Size of last line in old register
/// * `new_first_size` - Size of first line in new register
///
/// # Returns
/// Size of concatenated string (excluding NUL terminator)
#[must_use]
#[inline]
pub const fn calc_append_concat_size(old_last_size: usize, new_first_size: usize) -> usize {
    old_last_size + new_first_size
}

/// Calculate total size of yanked register after append.
///
/// # Arguments
/// * `old_size` - Number of lines in old register
/// * `new_size` - Number of lines in new register
/// * `did_concat` - Whether first line was concatenated
///
/// # Returns
/// Total number of lines in result register
#[must_use]
#[inline]
pub const fn calc_append_total_size(old_size: usize, new_size: usize, did_concat: bool) -> usize {
    if did_concat {
        // First line of new was merged, so -1
        old_size + new_size - 1
    } else {
        old_size + new_size
    }
}

/// Determine if register append should concatenate lines.
///
/// Concatenation happens for charwise registers unless CPO_REGAPPEND is set.
///
/// # Arguments
/// * `reg_type` - Register motion type
/// * `cpo_regappend` - Whether CPO_REGAPPEND is in 'cpoptions'
///
/// # Returns
/// true if lines should be concatenated
#[must_use]
#[inline]
pub const fn should_concat_append_lines(reg_type: MotionType, cpo_regappend: bool) -> bool {
    matches!(reg_type, MotionType::CharWise) && !cpo_regappend
}

/// Adjust op_end marks for linewise yank.
///
/// For linewise yank, start.col is set to 0 and end.col to MAXCOL.
///
/// # Arguments
/// * `yank_type` - The yank motion type
/// * `start_col` - Current start column
/// * `end_col` - Current end column
/// * `maxcol` - Maximum column value (MAXCOL)
///
/// # Returns
/// `(new_start_col, new_end_col)` - Adjusted column values
#[must_use]
#[inline]
pub const fn adjust_yank_marks_linewise(
    yank_type: MotionType,
    start_col: c_int,
    end_col: c_int,
    maxcol: c_int,
) -> (c_int, c_int) {
    if matches!(yank_type, MotionType::LineWise) {
        (0, maxcol)
    } else {
        (start_col, end_col)
    }
}

/// Check if op_end should be decremented for non-inclusive yank.
///
/// # Arguments
/// * `yank_type` - The yank motion type
/// * `inclusive` - Whether motion is inclusive
///
/// # Returns
/// true if op_end position should be decremented
#[must_use]
#[inline]
pub const fn should_decrement_yank_end(yank_type: MotionType, inclusive: bool) -> bool {
    !matches!(yank_type, MotionType::LineWise) && !inclusive
}

// =============================================================================
// FFI Wrappers
// =============================================================================

/// FFI wrapper for should_convert_yank_to_linewise.
///
/// # Safety
/// `adjusted_line_count_out` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_should_convert_yank_to_linewise(
    motion_type: c_int,
    start_col: c_int,
    end_col: c_int,
    inclusive: c_int,
    is_visual: c_int,
    selection_old: c_int,
    line_count: c_int,
    adjusted_line_count_out: *mut c_int,
) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    let (should_convert, adjusted) = should_convert_yank_to_linewise(
        mt,
        start_col,
        end_col,
        inclusive != 0,
        is_visual != 0,
        selection_old != 0,
        line_count,
    );

    if !adjusted_line_count_out.is_null() {
        // SAFETY: Caller guarantees pointer validity
        unsafe {
            *adjusted_line_count_out = adjusted;
        }
    }

    c_int::from(should_convert)
}

/// FFI wrapper for calc_yank_block_width.
#[no_mangle]
pub extern "C" fn rs_calc_yank_block_width(
    start_vcol: c_int,
    end_vcol: c_int,
    curswant_is_maxcol: c_int,
) -> c_int {
    calc_yank_block_width(start_vcol, end_vcol, curswant_is_maxcol != 0)
}

/// FFI wrapper for should_show_yank_message.
///
/// # Safety
/// `adjusted_count_out` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_should_show_yank_message(
    yank_type: c_int,
    line_count: c_int,
    report_threshold: c_int,
    adjusted_count_out: *mut c_int,
) -> c_int {
    let mt = MotionType::from_raw(yank_type);
    let (should_show, adjusted) = should_show_yank_message(mt, line_count, report_threshold);

    if !adjusted_count_out.is_null() {
        // SAFETY: Caller guarantees pointer validity
        unsafe {
            *adjusted_count_out = adjusted;
        }
    }

    c_int::from(should_show)
}

/// FFI wrapper for should_concat_append_lines.
#[no_mangle]
pub extern "C" fn rs_should_concat_append_lines(reg_type: c_int, cpo_regappend: c_int) -> c_int {
    let mt = MotionType::from_raw(reg_type);
    c_int::from(should_concat_append_lines(mt, cpo_regappend != 0))
}

/// FFI wrapper for should_decrement_yank_end.
#[no_mangle]
pub extern "C" fn rs_should_decrement_yank_end(yank_type: c_int, inclusive: c_int) -> c_int {
    let mt = MotionType::from_raw(yank_type);
    c_int::from(should_decrement_yank_end(mt, inclusive != 0))
}

/// FFI wrapper for adjust_yank_marks_linewise.
///
/// # Safety
/// `new_start_col_out` and `new_end_col_out` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_adjust_yank_marks_linewise(
    yank_type: c_int,
    start_col: c_int,
    end_col: c_int,
    maxcol: c_int,
    new_start_col_out: *mut c_int,
    new_end_col_out: *mut c_int,
) {
    let mt = MotionType::from_raw(yank_type);
    let (new_start, new_end) = adjust_yank_marks_linewise(mt, start_col, end_col, maxcol);

    // SAFETY: Caller guarantees pointer validity
    if !new_start_col_out.is_null() {
        unsafe {
            *new_start_col_out = new_start;
        }
    }
    if !new_end_col_out.is_null() {
        unsafe {
            *new_end_col_out = new_end;
        }
    }
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_should_convert_yank_to_linewise() {
        // Should convert: charwise, col 0, not inclusive, not visual, end col 0, multi-line
        let (convert, adjusted) =
            should_convert_yank_to_linewise(MotionType::CharWise, 0, 0, false, false, false, 5);
        assert!(convert);
        assert_eq!(adjusted, 4);

        // Should NOT convert: linewise
        let (convert, _) =
            should_convert_yank_to_linewise(MotionType::LineWise, 0, 0, false, false, false, 5);
        assert!(!convert);

        // Should NOT convert: start col not 0
        let (convert, _) =
            should_convert_yank_to_linewise(MotionType::CharWise, 5, 0, false, false, false, 5);
        assert!(!convert);

        // Should NOT convert: inclusive
        let (convert, _) =
            should_convert_yank_to_linewise(MotionType::CharWise, 0, 0, true, false, false, 5);
        assert!(!convert);

        // Should NOT convert: end col not 0
        let (convert, _) =
            should_convert_yank_to_linewise(MotionType::CharWise, 0, 5, false, false, false, 5);
        assert!(!convert);

        // Should NOT convert: single line
        let (convert, _) =
            should_convert_yank_to_linewise(MotionType::CharWise, 0, 0, false, false, false, 1);
        assert!(!convert);

        // Should convert: visual with selection='old'
        let (convert, _) =
            should_convert_yank_to_linewise(MotionType::CharWise, 0, 0, false, true, true, 5);
        assert!(convert);

        // Should NOT convert: visual without selection='old'
        let (convert, _) =
            should_convert_yank_to_linewise(MotionType::CharWise, 0, 0, false, true, false, 5);
        assert!(!convert);
    }

    #[test]
    fn test_calc_yank_block_width() {
        // Normal case
        assert_eq!(calc_yank_block_width(0, 10, false), 10);
        assert_eq!(calc_yank_block_width(5, 15, false), 10);

        // With MAXCOL (curswant_is_maxcol = true)
        assert_eq!(calc_yank_block_width(0, 10, true), 9);
        assert_eq!(calc_yank_block_width(5, 15, true), 9);

        // MAXCOL with width 0 - stays 0
        assert_eq!(calc_yank_block_width(10, 10, true), 0);

        // Edge case: end before start
        assert_eq!(calc_yank_block_width(10, 5, false), 0);
    }

    #[test]
    fn test_should_show_yank_message() {
        // Charwise single line: effective count is 0
        let (show, count) = should_show_yank_message(MotionType::CharWise, 1, 2);
        assert!(!show);
        assert_eq!(count, 0);

        // Charwise multi-line: normal count
        let (show, count) = should_show_yank_message(MotionType::CharWise, 5, 2);
        assert!(show);
        assert_eq!(count, 5);

        // Linewise: normal count
        let (show, count) = should_show_yank_message(MotionType::LineWise, 1, 2);
        assert!(!show); // 1 <= 2
        assert_eq!(count, 1);

        let (show, count) = should_show_yank_message(MotionType::LineWise, 5, 2);
        assert!(show); // 5 > 2
        assert_eq!(count, 5);
    }

    #[test]
    fn test_calc_append_concat_size() {
        assert_eq!(calc_append_concat_size(10, 5), 15);
        assert_eq!(calc_append_concat_size(0, 10), 10);
        assert_eq!(calc_append_concat_size(10, 0), 10);
    }

    #[test]
    fn test_calc_append_total_size() {
        // Without concatenation
        assert_eq!(calc_append_total_size(5, 3, false), 8);

        // With concatenation
        assert_eq!(calc_append_total_size(5, 3, true), 7);
    }

    #[test]
    fn test_should_concat_append_lines() {
        // Charwise without CPO_REGAPPEND: concat
        assert!(should_concat_append_lines(MotionType::CharWise, false));

        // Charwise with CPO_REGAPPEND: no concat
        assert!(!should_concat_append_lines(MotionType::CharWise, true));

        // Linewise: no concat
        assert!(!should_concat_append_lines(MotionType::LineWise, false));

        // Blockwise: no concat
        assert!(!should_concat_append_lines(MotionType::BlockWise, false));
    }

    #[test]
    fn test_adjust_yank_marks_linewise() {
        const MAXCOL: c_int = 0x7FFF_FFFF;

        // Linewise: adjust to 0 and MAXCOL
        let (start, end) = adjust_yank_marks_linewise(MotionType::LineWise, 5, 10, MAXCOL);
        assert_eq!(start, 0);
        assert_eq!(end, MAXCOL);

        // Charwise: keep original
        let (start, end) = adjust_yank_marks_linewise(MotionType::CharWise, 5, 10, MAXCOL);
        assert_eq!(start, 5);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_should_decrement_yank_end() {
        // Non-linewise and non-inclusive: decrement
        assert!(should_decrement_yank_end(MotionType::CharWise, false));
        assert!(should_decrement_yank_end(MotionType::BlockWise, false));

        // Linewise: never decrement
        assert!(!should_decrement_yank_end(MotionType::LineWise, false));
        assert!(!should_decrement_yank_end(MotionType::LineWise, true));

        // Inclusive: don't decrement
        assert!(!should_decrement_yank_end(MotionType::CharWise, true));
    }

    #[test]
    fn test_ffi_wrappers() {
        // rs_calc_yank_block_width
        assert_eq!(rs_calc_yank_block_width(0, 10, 0), 10);
        assert_eq!(rs_calc_yank_block_width(0, 10, 1), 9);

        // rs_should_concat_append_lines (charwise=0)
        assert_eq!(rs_should_concat_append_lines(0, 0), 1);
        assert_eq!(rs_should_concat_append_lines(0, 1), 0);

        // rs_should_decrement_yank_end (charwise=0)
        assert_eq!(rs_should_decrement_yank_end(0, 0), 1);
        assert_eq!(rs_should_decrement_yank_end(0, 1), 0);
    }
}
