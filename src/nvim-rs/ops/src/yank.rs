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

// =============================================================================
// Phase O1: Additional Yank Operation Helpers
// =============================================================================

/// Check if a register name is valid for yanking.
///
/// Valid registers for yank:
/// - Named registers 'a'-'z', 'A'-'Z' (append)
/// - Numbered registers '0'-'9'
/// - Unnamed register NUL (0)
/// - Small delete register '-'
/// - Special registers '_' (black hole), '+'/'*' (clipboard), etc.
///
/// # Arguments
/// * `regname` - Register name character
///
/// # Returns
/// true if the register is valid for yanking
#[must_use]
pub const fn is_valid_yank_register(regname: c_int) -> bool {
    // Unnamed register
    if regname == 0 {
        return true;
    }

    // Named registers a-z (97-122)
    if regname >= b'a' as c_int && regname <= b'z' as c_int {
        return true;
    }

    // Append registers A-Z (65-90)
    if regname >= b'A' as c_int && regname <= b'Z' as c_int {
        return true;
    }

    // Numbered registers 0-9 (48-57)
    if regname >= b'0' as c_int && regname <= b'9' as c_int {
        return true;
    }

    // Special registers
    matches!(
        regname,
        45  // '-' small delete
            | 95  // '_' black hole
            | 43  // '+' system clipboard
            | 42  // '*' selection
            | 34  // '"' unnamed (default)
            | 47  // '/' search pattern
            | 35  // '#' alternate buffer
            | 58  // ':' last command
            | 46  // '.' last inserted text
            | 37 // '%' current file name
    )
}

/// Check if a register is an append register (uppercase).
///
/// Uppercase letters A-Z indicate appending to the lowercase register.
///
/// # Arguments
/// * `regname` - Register name character
///
/// # Returns
/// true if this is an append register
#[must_use]
#[inline]
pub const fn is_append_register(regname: c_int) -> bool {
    // ASCII uppercase A-Z is 65-90
    regname >= b'A' as c_int && regname <= b'Z' as c_int
}

/// Get the base register name (lowercase for append registers).
///
/// # Arguments
/// * `regname` - Register name character
///
/// # Returns
/// Lowercase register name for A-Z, original otherwise
#[must_use]
#[inline]
pub const fn get_base_register(regname: c_int) -> c_int {
    // ASCII uppercase A-Z is 65-90, lowercase a-z is 97-122 (difference of 32)
    if regname >= b'A' as c_int && regname <= b'Z' as c_int {
        regname + 32 // Convert to lowercase
    } else {
        regname
    }
}

/// Check if register is the black hole register.
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if this is the black hole register
#[must_use]
#[inline]
pub const fn is_black_hole_yank_register(regname: c_int) -> bool {
    regname == b'_' as c_int
}

/// Check if register is a clipboard register.
///
/// '+' and '*' are system clipboard registers.
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if this is a clipboard register
#[must_use]
#[inline]
pub const fn is_clipboard_register(regname: c_int) -> bool {
    regname == b'+' as c_int || regname == b'*' as c_int
}

/// Check if register is a numbered register.
///
/// '0'-'9' are numbered registers.
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if this is a numbered register
#[must_use]
#[inline]
pub const fn is_numbered_register(regname: c_int) -> bool {
    // ASCII digits 0-9 are 48-57
    regname >= b'0' as c_int && regname <= b'9' as c_int
}

/// Check if register is a named register (a-z).
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if this is a named register
#[must_use]
#[inline]
pub const fn is_named_register(regname: c_int) -> bool {
    // ASCII lowercase a-z is 97-122
    regname >= b'a' as c_int && regname <= b'z' as c_int
}

/// Calculate the effective yank line count for messaging.
///
/// For charwise yank of a single line, the count is 0 (no message).
///
/// # Arguments
/// * `motion_type` - Motion type
/// * `line_count` - Number of lines yanked
///
/// # Returns
/// Effective line count for messaging
#[must_use]
#[inline]
pub const fn calc_yank_message_line_count(motion_type: MotionType, line_count: c_int) -> c_int {
    if matches!(motion_type, MotionType::CharWise) && line_count == 1 {
        0
    } else {
        line_count
    }
}

/// Calculate size needed for yank register array.
///
/// # Arguments
/// * `line_count` - Number of lines to yank
/// * `sizeof_string` - Size of String struct
///
/// # Returns
/// Total bytes needed for allocation
#[must_use]
#[inline]
pub const fn calc_yank_array_size(line_count: usize, sizeof_string: usize) -> usize {
    line_count.saturating_mul(sizeof_string)
}

/// Determine if yank should set marks.
///
/// # Arguments
/// * `lockmarks` - Whether CMOD_LOCKMARKS is set
///
/// # Returns
/// true if marks should be set
#[must_use]
#[inline]
pub const fn should_set_yank_marks(lockmarks: bool) -> bool {
    !lockmarks
}

/// Calculate yank register width for block mode.
///
/// # Arguments
/// * `end_vcol` - End virtual column
/// * `start_vcol` - Start virtual column
/// * `curswant_maxcol` - Whether curswant is MAXCOL
///
/// # Returns
/// Register width
#[must_use]
#[inline]
pub const fn calc_yank_register_width(
    end_vcol: c_int,
    start_vcol: c_int,
    curswant_maxcol: bool,
) -> c_int {
    let mut width = end_vcol - start_vcol;
    if curswant_maxcol && width > 0 {
        width -= 1;
    }
    if width < 0 {
        0
    } else {
        width
    }
}

/// Check if yank needs to copy text to clipboard.
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if clipboard copy is needed
#[must_use]
#[inline]
pub const fn needs_clipboard_copy(regname: c_int) -> bool {
    // Copy to clipboard when using unnamed, +, or * register
    regname == 0 || regname == b'+' as c_int || regname == b'*' as c_int
}

/// Determine yank type after potential conversion.
///
/// Charwise yank may be converted to linewise under certain conditions.
///
/// # Arguments
/// * `motion_type` - Original motion type
/// * `converted_to_linewise` - Whether conversion happened
///
/// # Returns
/// Final yank type
#[must_use]
#[inline]
pub const fn get_final_yank_type(
    motion_type: MotionType,
    converted_to_linewise: bool,
) -> MotionType {
    if converted_to_linewise {
        MotionType::LineWise
    } else {
        motion_type
    }
}

/// Check if yank requires charwise line preparation.
///
/// # Arguments
/// * `motion_type` - Motion type
///
/// # Returns
/// true if charwise prep is needed
#[must_use]
#[inline]
pub const fn needs_charwise_yank_prep(motion_type: MotionType) -> bool {
    matches!(motion_type, MotionType::CharWise)
}

/// Check if yank requires block preparation.
///
/// # Arguments
/// * `motion_type` - Motion type
///
/// # Returns
/// true if block prep is needed
#[must_use]
#[inline]
pub const fn needs_block_yank_prep(motion_type: MotionType) -> bool {
    matches!(motion_type, MotionType::BlockWise)
}

// =============================================================================
// Phase O6 Clipboard & System Integration Helpers
// =============================================================================

/// Check if register is the system clipboard ('+').
#[must_use]
#[inline]
pub const fn is_system_clipboard_register(regname: c_int) -> bool {
    regname == b'+' as c_int
}

/// Check if register is the selection clipboard ('*').
#[must_use]
#[inline]
pub const fn is_selection_register(regname: c_int) -> bool {
    regname == b'*' as c_int
}

/// Check if register is the unnamed register.
#[must_use]
#[inline]
pub const fn is_unnamed_register(regname: c_int) -> bool {
    regname == 0 || regname == b'"' as c_int
}

/// Check if register is the small delete register ('-').
#[must_use]
#[inline]
pub const fn is_small_delete_register(regname: c_int) -> bool {
    regname == b'-' as c_int
}

/// Check if register is the search register ('/').
#[must_use]
#[inline]
pub const fn is_search_register(regname: c_int) -> bool {
    regname == b'/' as c_int
}

/// Check if register is the last command register (':').
#[must_use]
#[inline]
pub const fn is_command_register(regname: c_int) -> bool {
    regname == b':' as c_int
}

/// Check if register is the expression register ('=').
#[must_use]
#[inline]
pub const fn is_expression_register(regname: c_int) -> bool {
    regname == b'=' as c_int
}

/// Check if register is the last inserted text register ('.').
#[must_use]
#[inline]
pub const fn is_last_insert_register(regname: c_int) -> bool {
    regname == b'.' as c_int
}

/// Check if register is the alternate buffer register ('#').
#[must_use]
#[inline]
pub const fn is_alternate_register(regname: c_int) -> bool {
    regname == b'#' as c_int
}

/// Check if register is the current file name register ('%').
#[must_use]
#[inline]
pub const fn is_filename_register(regname: c_int) -> bool {
    regname == b'%' as c_int
}

/// Check if register is read-only.
#[must_use]
#[inline]
pub const fn is_readonly_register(regname: c_int) -> bool {
    regname == b'.' as c_int  // last insert
        || regname == b':' as c_int  // command
        || regname == b'/' as c_int  // search
        || regname == b'%' as c_int  // file name
        || regname == b'#' as c_int // alternate
}

/// Check if register needs async clipboard operation.
#[must_use]
#[inline]
pub const fn needs_async_clipboard(regname: c_int) -> bool {
    regname == b'+' as c_int || regname == b'*' as c_int
}

/// Get the corresponding register index for numbered registers.
#[must_use]
#[inline]
pub const fn get_numbered_register_index(regname: c_int) -> c_int {
    if regname >= b'0' as c_int && regname <= b'9' as c_int {
        regname - b'0' as c_int
    } else {
        -1
    }
}

/// Get the corresponding register index for named registers (a-z).
#[must_use]
#[inline]
pub const fn get_named_register_index(regname: c_int) -> c_int {
    if regname >= b'a' as c_int && regname <= b'z' as c_int {
        regname - b'a' as c_int + 10 // After numbered registers 0-9
    } else {
        -1
    }
}

/// Check if clipboard provider is available.
#[must_use]
#[inline]
pub const fn clipboard_provider_available(has_provider: bool) -> bool {
    has_provider
}

/// Determine if system clipboard should sync with unnamed.
#[must_use]
#[inline]
pub const fn should_sync_unnamed_clipboard(clipboard_unnamed: bool, regname: c_int) -> bool {
    clipboard_unnamed && regname == 0
}

/// Calculate clipboard register type.
#[must_use]
#[inline]
pub const fn get_clipboard_type(regname: c_int) -> c_int {
    if regname == b'+' as c_int {
        1 // CLIPBOARD
    } else if regname == b'*' as c_int {
        2 // PRIMARY (selection)
    } else {
        0 // None
    }
}

/// Check if yank should rotate numbered registers.
#[must_use]
#[inline]
pub const fn should_rotate_numbered_registers(
    regname: c_int,
    motion_type: MotionType,
    use_reg_one: bool,
) -> bool {
    // Rotate when using unnamed register and linewise, or when use_reg_one is set
    (regname == 0 && matches!(motion_type, MotionType::LineWise)) || use_reg_one
}

/// Check if delete should use small delete register.
#[must_use]
#[inline]
pub const fn should_use_small_delete(
    regname: c_int,
    motion_type: MotionType,
    line_count: c_int,
) -> bool {
    regname == 0 && !matches!(motion_type, MotionType::LineWise) && line_count == 1
}

/// Get register name for display.
#[must_use]
#[inline]
pub const fn get_register_display_char(regname: c_int) -> c_int {
    if regname == 0 {
        b'"' as c_int
    } else {
        regname
    }
}

// =============================================================================
// FFI Wrappers for Phase O1 Additions
// =============================================================================

/// FFI wrapper for is_valid_yank_register.
#[no_mangle]
pub extern "C" fn rs_is_valid_yank_register(regname: c_int) -> c_int {
    c_int::from(is_valid_yank_register(regname))
}

// Note: rs_is_append_register, rs_is_clipboard_register, rs_is_numbered_register,
// rs_is_named_register are already exported from register crate and normal crate.
// We provide internal Rust functions here but don't duplicate the FFI exports.

/// FFI wrapper for get_base_register.
#[no_mangle]
pub extern "C" fn rs_get_base_register(regname: c_int) -> c_int {
    get_base_register(regname)
}

/// FFI wrapper for is_black_hole_yank_register.
#[no_mangle]
pub extern "C" fn rs_is_black_hole_yank_register(regname: c_int) -> c_int {
    c_int::from(is_black_hole_yank_register(regname))
}

/// FFI wrapper for calc_yank_message_line_count.
#[no_mangle]
pub extern "C" fn rs_calc_yank_message_line_count(motion_type: c_int, line_count: c_int) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    calc_yank_message_line_count(mt, line_count)
}

/// FFI wrapper for should_set_yank_marks.
#[no_mangle]
pub extern "C" fn rs_should_set_yank_marks(lockmarks: c_int) -> c_int {
    c_int::from(should_set_yank_marks(lockmarks != 0))
}

/// FFI wrapper for calc_yank_register_width.
#[no_mangle]
pub extern "C" fn rs_calc_yank_register_width(
    end_vcol: c_int,
    start_vcol: c_int,
    curswant_maxcol: c_int,
) -> c_int {
    calc_yank_register_width(end_vcol, start_vcol, curswant_maxcol != 0)
}

/// FFI wrapper for needs_clipboard_copy.
#[no_mangle]
pub extern "C" fn rs_needs_clipboard_copy(regname: c_int) -> c_int {
    c_int::from(needs_clipboard_copy(regname))
}

/// FFI wrapper for get_final_yank_type.
#[no_mangle]
pub extern "C" fn rs_get_final_yank_type(motion_type: c_int, converted: c_int) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    get_final_yank_type(mt, converted != 0).as_raw()
}

/// FFI wrapper for needs_charwise_yank_prep.
#[no_mangle]
pub extern "C" fn rs_needs_charwise_yank_prep(motion_type: c_int) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    c_int::from(needs_charwise_yank_prep(mt))
}

/// FFI wrapper for needs_block_yank_prep.
#[no_mangle]
pub extern "C" fn rs_needs_block_yank_prep(motion_type: c_int) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    c_int::from(needs_block_yank_prep(mt))
}

// =============================================================================
// FFI Wrappers for Phase O6 Additions
// =============================================================================

/// FFI: Check if system clipboard register.
#[no_mangle]
pub extern "C" fn rs_is_system_clipboard_register(regname: c_int) -> c_int {
    c_int::from(is_system_clipboard_register(regname))
}

/// FFI: Check if selection register.
#[no_mangle]
pub extern "C" fn rs_is_selection_register(regname: c_int) -> c_int {
    c_int::from(is_selection_register(regname))
}

/// FFI: Check if unnamed register.
#[no_mangle]
pub extern "C" fn rs_is_unnamed_register(regname: c_int) -> c_int {
    c_int::from(is_unnamed_register(regname))
}

/// FFI: Check if small delete register.
#[no_mangle]
pub extern "C" fn rs_is_small_delete_register(regname: c_int) -> c_int {
    c_int::from(is_small_delete_register(regname))
}

/// FFI: Check if search register.
#[no_mangle]
pub extern "C" fn rs_is_search_register(regname: c_int) -> c_int {
    c_int::from(is_search_register(regname))
}

/// FFI: Check if command register.
#[no_mangle]
pub extern "C" fn rs_is_command_register(regname: c_int) -> c_int {
    c_int::from(is_command_register(regname))
}

/// FFI: Check if expression register.
#[no_mangle]
pub extern "C" fn rs_is_expression_register(regname: c_int) -> c_int {
    c_int::from(is_expression_register(regname))
}

// Note: rs_is_last_insert_register already exists in register crate

/// FFI: Check if alternate register.
#[no_mangle]
pub extern "C" fn rs_is_alternate_register(regname: c_int) -> c_int {
    c_int::from(is_alternate_register(regname))
}

/// FFI: Check if filename register.
#[no_mangle]
pub extern "C" fn rs_is_filename_register(regname: c_int) -> c_int {
    c_int::from(is_filename_register(regname))
}

// Note: rs_is_readonly_register already exists in ex_docmd crate

/// FFI: Check if needs async clipboard.
#[no_mangle]
pub extern "C" fn rs_needs_async_clipboard(regname: c_int) -> c_int {
    c_int::from(needs_async_clipboard(regname))
}

/// FFI: Get numbered register index.
#[no_mangle]
pub extern "C" fn rs_get_numbered_register_index(regname: c_int) -> c_int {
    get_numbered_register_index(regname)
}

/// FFI: Get named register index.
#[no_mangle]
pub extern "C" fn rs_get_named_register_index(regname: c_int) -> c_int {
    get_named_register_index(regname)
}

// Note: rs_clipboard_provider_available already exists in clipboard crate

/// FFI: Check if should sync unnamed clipboard.
#[no_mangle]
pub extern "C" fn rs_should_sync_unnamed_clipboard(
    clipboard_unnamed: c_int,
    regname: c_int,
) -> c_int {
    c_int::from(should_sync_unnamed_clipboard(
        clipboard_unnamed != 0,
        regname,
    ))
}

/// FFI: Get clipboard type.
#[no_mangle]
pub extern "C" fn rs_get_clipboard_type(regname: c_int) -> c_int {
    get_clipboard_type(regname)
}

/// FFI: Check if should rotate numbered registers.
#[no_mangle]
pub extern "C" fn rs_should_rotate_numbered_registers(
    regname: c_int,
    motion_type: c_int,
    use_reg_one: c_int,
) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    c_int::from(should_rotate_numbered_registers(
        regname,
        mt,
        use_reg_one != 0,
    ))
}

/// FFI: Check if should use small delete register.
#[no_mangle]
pub extern "C" fn rs_should_use_small_delete(
    regname: c_int,
    motion_type: c_int,
    line_count: c_int,
) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    c_int::from(should_use_small_delete(regname, mt, line_count))
}

/// FFI: Get register display char.
#[no_mangle]
pub extern "C" fn rs_get_register_display_char(regname: c_int) -> c_int {
    get_register_display_char(regname)
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

    // =========================================================================
    // Phase O1 Addition Tests
    // =========================================================================

    #[test]
    fn test_is_valid_yank_register() {
        // Unnamed register
        assert!(is_valid_yank_register(0));

        // Named registers
        assert!(is_valid_yank_register(b'a' as c_int));
        assert!(is_valid_yank_register(b'z' as c_int));
        assert!(is_valid_yank_register(b'A' as c_int)); // append
        assert!(is_valid_yank_register(b'Z' as c_int));

        // Numbered registers
        assert!(is_valid_yank_register(b'0' as c_int));
        assert!(is_valid_yank_register(b'9' as c_int));

        // Special registers
        assert!(is_valid_yank_register(b'-' as c_int));
        assert!(is_valid_yank_register(b'_' as c_int));
        assert!(is_valid_yank_register(b'+' as c_int));
        assert!(is_valid_yank_register(b'*' as c_int));
        assert!(is_valid_yank_register(b'"' as c_int));
    }

    #[test]
    fn test_is_append_register() {
        // Uppercase = append
        assert!(is_append_register(b'A' as c_int));
        assert!(is_append_register(b'Z' as c_int));

        // Lowercase = not append
        assert!(!is_append_register(b'a' as c_int));
        assert!(!is_append_register(b'z' as c_int));

        // Other = not append
        assert!(!is_append_register(0));
        assert!(!is_append_register(b'0' as c_int));
    }

    #[test]
    fn test_get_base_register() {
        // Uppercase -> lowercase
        assert_eq!(get_base_register(b'A' as c_int), b'a' as c_int);
        assert_eq!(get_base_register(b'Z' as c_int), b'z' as c_int);

        // Lowercase unchanged
        assert_eq!(get_base_register(b'a' as c_int), b'a' as c_int);

        // Other unchanged
        assert_eq!(get_base_register(0), 0);
        assert_eq!(get_base_register(b'0' as c_int), b'0' as c_int);
    }

    #[test]
    fn test_is_black_hole_yank_register() {
        assert!(is_black_hole_yank_register(b'_' as c_int));
        assert!(!is_black_hole_yank_register(b'a' as c_int));
        assert!(!is_black_hole_yank_register(0));
    }

    #[test]
    fn test_is_clipboard_register() {
        assert!(is_clipboard_register(b'+' as c_int));
        assert!(is_clipboard_register(b'*' as c_int));
        assert!(!is_clipboard_register(b'a' as c_int));
        assert!(!is_clipboard_register(0));
    }

    #[test]
    fn test_is_numbered_register() {
        assert!(is_numbered_register(b'0' as c_int));
        assert!(is_numbered_register(b'9' as c_int));
        assert!(!is_numbered_register(b'a' as c_int));
        assert!(!is_numbered_register(0));
    }

    #[test]
    fn test_is_named_register() {
        assert!(is_named_register(b'a' as c_int));
        assert!(is_named_register(b'z' as c_int));
        assert!(!is_named_register(b'A' as c_int)); // uppercase = append
        assert!(!is_named_register(b'0' as c_int));
    }

    #[test]
    fn test_calc_yank_message_line_count() {
        // Charwise single line = 0
        assert_eq!(calc_yank_message_line_count(MotionType::CharWise, 1), 0);

        // Charwise multi-line = actual count
        assert_eq!(calc_yank_message_line_count(MotionType::CharWise, 5), 5);

        // Linewise = actual count
        assert_eq!(calc_yank_message_line_count(MotionType::LineWise, 1), 1);
        assert_eq!(calc_yank_message_line_count(MotionType::LineWise, 5), 5);
    }

    #[test]
    fn test_calc_yank_array_size() {
        assert_eq!(calc_yank_array_size(5, 16), 80);
        assert_eq!(calc_yank_array_size(0, 16), 0);
    }

    #[test]
    fn test_should_set_yank_marks() {
        assert!(should_set_yank_marks(false));
        assert!(!should_set_yank_marks(true));
    }

    #[test]
    fn test_calc_yank_register_width() {
        assert_eq!(calc_yank_register_width(20, 10, false), 10);
        assert_eq!(calc_yank_register_width(20, 10, true), 9); // MAXCOL
        assert_eq!(calc_yank_register_width(10, 10, true), 0); // width 0 stays 0
    }

    #[test]
    fn test_needs_clipboard_copy() {
        assert!(needs_clipboard_copy(0)); // unnamed
        assert!(needs_clipboard_copy(b'+' as c_int));
        assert!(needs_clipboard_copy(b'*' as c_int));
        assert!(!needs_clipboard_copy(b'a' as c_int));
    }

    #[test]
    fn test_get_final_yank_type() {
        // Not converted
        assert_eq!(
            get_final_yank_type(MotionType::CharWise, false),
            MotionType::CharWise
        );
        assert_eq!(
            get_final_yank_type(MotionType::BlockWise, false),
            MotionType::BlockWise
        );

        // Converted to linewise
        assert_eq!(
            get_final_yank_type(MotionType::CharWise, true),
            MotionType::LineWise
        );
    }

    #[test]
    fn test_needs_charwise_yank_prep() {
        assert!(needs_charwise_yank_prep(MotionType::CharWise));
        assert!(!needs_charwise_yank_prep(MotionType::LineWise));
        assert!(!needs_charwise_yank_prep(MotionType::BlockWise));
    }

    #[test]
    fn test_needs_block_yank_prep() {
        assert!(needs_block_yank_prep(MotionType::BlockWise));
        assert!(!needs_block_yank_prep(MotionType::CharWise));
        assert!(!needs_block_yank_prep(MotionType::LineWise));
    }

    #[test]
    fn test_phase_o1_yank_ffi_wrappers() {
        // rs_is_valid_yank_register
        assert_eq!(rs_is_valid_yank_register(b'a' as c_int), 1);

        // Note: rs_is_append_register, rs_is_clipboard_register, etc. are
        // exported from other crates - test internal functions instead
        assert!(is_append_register(b'A' as c_int));
        assert!(!is_append_register(b'a' as c_int));
        assert!(is_clipboard_register(b'+' as c_int));
        assert!(is_numbered_register(b'0' as c_int));
        assert!(is_named_register(b'a' as c_int));

        // rs_get_base_register
        assert_eq!(rs_get_base_register(b'A' as c_int), b'a' as c_int);

        // rs_is_black_hole_yank_register
        assert_eq!(rs_is_black_hole_yank_register(b'_' as c_int), 1);

        // rs_calc_yank_message_line_count (charwise=0)
        assert_eq!(rs_calc_yank_message_line_count(0, 1), 0);

        // rs_should_set_yank_marks
        assert_eq!(rs_should_set_yank_marks(0), 1);
        assert_eq!(rs_should_set_yank_marks(1), 0);

        // rs_calc_yank_register_width
        assert_eq!(rs_calc_yank_register_width(20, 10, 0), 10);

        // rs_needs_clipboard_copy
        assert_eq!(rs_needs_clipboard_copy(0), 1);

        // rs_get_final_yank_type (charwise=0, linewise=1)
        assert_eq!(rs_get_final_yank_type(0, 0), 0);
        assert_eq!(rs_get_final_yank_type(0, 1), 1);

        // rs_needs_charwise_yank_prep (charwise=0)
        assert_eq!(rs_needs_charwise_yank_prep(0), 1);

        // rs_needs_block_yank_prep (blockwise=2)
        assert_eq!(rs_needs_block_yank_prep(2), 1);
    }

    // =========================================================================
    // Phase O6 Clipboard & System Integration Tests
    // =========================================================================

    #[test]
    fn test_is_system_clipboard_register() {
        assert!(is_system_clipboard_register(b'+' as c_int));
        assert!(!is_system_clipboard_register(b'*' as c_int));
        assert!(!is_system_clipboard_register(b'a' as c_int));
    }

    #[test]
    fn test_is_selection_register() {
        assert!(is_selection_register(b'*' as c_int));
        assert!(!is_selection_register(b'+' as c_int));
        assert!(!is_selection_register(b'a' as c_int));
    }

    #[test]
    fn test_is_unnamed_register() {
        assert!(is_unnamed_register(0));
        assert!(is_unnamed_register(b'"' as c_int));
        assert!(!is_unnamed_register(b'a' as c_int));
    }

    #[test]
    fn test_is_small_delete_register() {
        assert!(is_small_delete_register(b'-' as c_int));
        assert!(!is_small_delete_register(b'a' as c_int));
    }

    #[test]
    fn test_is_search_register() {
        assert!(is_search_register(b'/' as c_int));
        assert!(!is_search_register(b':' as c_int));
    }

    #[test]
    fn test_is_command_register() {
        assert!(is_command_register(b':' as c_int));
        assert!(!is_command_register(b'/' as c_int));
    }

    #[test]
    fn test_is_expression_register() {
        assert!(is_expression_register(b'=' as c_int));
        assert!(!is_expression_register(b'a' as c_int));
    }

    #[test]
    fn test_is_last_insert_register() {
        assert!(is_last_insert_register(b'.' as c_int));
        assert!(!is_last_insert_register(b'a' as c_int));
    }

    #[test]
    fn test_is_alternate_register() {
        assert!(is_alternate_register(b'#' as c_int));
        assert!(!is_alternate_register(b'%' as c_int));
    }

    #[test]
    fn test_is_filename_register() {
        assert!(is_filename_register(b'%' as c_int));
        assert!(!is_filename_register(b'#' as c_int));
    }

    #[test]
    fn test_is_readonly_register() {
        assert!(is_readonly_register(b'.' as c_int));
        assert!(is_readonly_register(b':' as c_int));
        assert!(is_readonly_register(b'/' as c_int));
        assert!(is_readonly_register(b'%' as c_int));
        assert!(is_readonly_register(b'#' as c_int));
        assert!(!is_readonly_register(b'a' as c_int));
        assert!(!is_readonly_register(b'+' as c_int));
    }

    #[test]
    fn test_needs_async_clipboard() {
        assert!(needs_async_clipboard(b'+' as c_int));
        assert!(needs_async_clipboard(b'*' as c_int));
        assert!(!needs_async_clipboard(b'a' as c_int));
    }

    #[test]
    fn test_get_numbered_register_index() {
        assert_eq!(get_numbered_register_index(b'0' as c_int), 0);
        assert_eq!(get_numbered_register_index(b'5' as c_int), 5);
        assert_eq!(get_numbered_register_index(b'9' as c_int), 9);
        assert_eq!(get_numbered_register_index(b'a' as c_int), -1);
    }

    #[test]
    fn test_get_named_register_index() {
        assert_eq!(get_named_register_index(b'a' as c_int), 10);
        assert_eq!(get_named_register_index(b'z' as c_int), 35);
        assert_eq!(get_named_register_index(b'0' as c_int), -1);
    }

    #[test]
    fn test_clipboard_provider_available() {
        assert!(clipboard_provider_available(true));
        assert!(!clipboard_provider_available(false));
    }

    #[test]
    fn test_should_sync_unnamed_clipboard() {
        assert!(should_sync_unnamed_clipboard(true, 0));
        assert!(!should_sync_unnamed_clipboard(true, b'a' as c_int));
        assert!(!should_sync_unnamed_clipboard(false, 0));
    }

    #[test]
    fn test_get_clipboard_type() {
        assert_eq!(get_clipboard_type(b'+' as c_int), 1);
        assert_eq!(get_clipboard_type(b'*' as c_int), 2);
        assert_eq!(get_clipboard_type(b'a' as c_int), 0);
    }

    #[test]
    fn test_should_rotate_numbered_registers() {
        // Unnamed + linewise
        assert!(should_rotate_numbered_registers(
            0,
            MotionType::LineWise,
            false
        ));
        // Unnamed + charwise without use_reg_one
        assert!(!should_rotate_numbered_registers(
            0,
            MotionType::CharWise,
            false
        ));
        // Any register with use_reg_one
        assert!(should_rotate_numbered_registers(
            b'a' as c_int,
            MotionType::CharWise,
            true
        ));
    }

    #[test]
    fn test_should_use_small_delete() {
        // Unnamed, charwise, single line
        assert!(should_use_small_delete(0, MotionType::CharWise, 1));
        // Unnamed, linewise
        assert!(!should_use_small_delete(0, MotionType::LineWise, 1));
        // Unnamed, charwise, multi-line
        assert!(!should_use_small_delete(0, MotionType::CharWise, 5));
        // Named register
        assert!(!should_use_small_delete(
            b'a' as c_int,
            MotionType::CharWise,
            1
        ));
    }

    #[test]
    fn test_get_register_display_char() {
        assert_eq!(get_register_display_char(0), b'"' as c_int);
        assert_eq!(get_register_display_char(b'a' as c_int), b'a' as c_int);
    }

    #[test]
    fn test_phase_o6_ffi_wrappers() {
        // Register type checks
        assert_eq!(rs_is_system_clipboard_register(b'+' as c_int), 1);
        assert_eq!(rs_is_selection_register(b'*' as c_int), 1);
        assert_eq!(rs_is_unnamed_register(0), 1);
        assert_eq!(rs_is_small_delete_register(b'-' as c_int), 1);
        assert_eq!(rs_is_search_register(b'/' as c_int), 1);
        assert_eq!(rs_is_command_register(b':' as c_int), 1);
        assert_eq!(rs_is_expression_register(b'=' as c_int), 1);
        // rs_is_last_insert_register is in register crate
        assert!(is_last_insert_register(b'.' as c_int));
        assert_eq!(rs_is_alternate_register(b'#' as c_int), 1);
        assert_eq!(rs_is_filename_register(b'%' as c_int), 1);
        // rs_is_readonly_register is in ex_docmd crate
        assert!(is_readonly_register(b'.' as c_int));
        assert_eq!(rs_needs_async_clipboard(b'+' as c_int), 1);

        // Index calculations
        assert_eq!(rs_get_numbered_register_index(b'5' as c_int), 5);
        assert_eq!(rs_get_named_register_index(b'a' as c_int), 10);

        // Clipboard helpers
        // rs_clipboard_provider_available is in clipboard crate
        assert!(clipboard_provider_available(true));
        assert_eq!(rs_should_sync_unnamed_clipboard(1, 0), 1);
        assert_eq!(rs_get_clipboard_type(b'+' as c_int), 1);

        // Register rotation
        assert_eq!(rs_should_rotate_numbered_registers(0, 1, 0), 1); // linewise=1
        assert_eq!(rs_should_use_small_delete(0, 0, 1), 1); // charwise=0

        // Display char
        assert_eq!(rs_get_register_display_char(0), b'"' as c_int);
    }
}
