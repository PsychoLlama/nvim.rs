//! Delete operations (d, x, D, etc.)
//!
//! This module implements calculation logic for delete operators.

use std::ffi::c_int;

use crate::types::{MotionType, OpType};

/// Calculate the number of bytes to delete in a single line.
///
/// This calculates `n` in the C code:
/// `int n = oap->end.col - oap->start.col + 1 - !oap->inclusive;`
///
/// # Arguments
/// * `start_col` - Starting column
/// * `end_col` - Ending column
/// * `inclusive` - Whether the motion is inclusive
///
/// # Returns
/// Number of bytes to delete
#[must_use]
#[inline]
pub const fn calc_delete_bytes(start_col: c_int, end_col: c_int, inclusive: bool) -> c_int {
    let adjustment = if inclusive { 0 } else { 1 };
    let n = end_col - start_col + 1 - adjustment;
    if n < 0 {
        0
    } else {
        n
    }
}

/// Calculate delete bytes with virtual edit adjustments.
///
/// In virtual edit mode, additional adjustments may be needed when
/// deleting at line end or with coladd.
///
/// # Arguments
/// * `start_col` - Starting column
/// * `end_col` - Ending column
/// * `inclusive` - Whether the motion is inclusive
/// * `end_coladd` - Virtual column offset at end
/// * `start_coladd` - Virtual column offset at start
/// * `line_len` - Length of the line
///
/// # Returns
/// Number of bytes to delete
#[must_use]
#[allow(clippy::too_many_arguments)]
pub const fn calc_delete_bytes_virtual(
    start_col: c_int,
    end_col: c_int,
    inclusive: bool,
    end_coladd: c_int,
    start_coladd: c_int,
    line_len: c_int,
) -> c_int {
    let mut n = calc_delete_bytes(start_col, end_col, inclusive);

    // Virtual edit adjustments from ops.c:
    // if (oap->end.coladd != 0
    //     && (int)oap->end.col >= len - 1
    //     && !(oap->start.coladd && (int)oap->end.col >= len - 1))
    //   n++;
    // Simplified: end_coladd != 0 && end_col >= line_len - 1 && start_coladd == 0
    if end_coladd != 0 && end_col >= line_len - 1 && start_coladd == 0 {
        n += 1;
    }

    // Delete at least one char when coladds differ
    if n == 0 && start_coladd != end_coladd {
        n = 1;
    }

    n
}

/// Check if a charwise delete spanning multiple lines should become linewise.
///
/// This implements the Vi-compatible behavior where a multi-line charwise
/// delete becomes linewise if it results in a blank line.
///
/// Note: The actual check for blank line (skipwhite + inindent) must be
/// done in C as it requires buffer access.
///
/// # Arguments
/// * `motion_type` - Current motion type
/// * `is_visual` - Whether in visual mode
/// * `line_count` - Number of lines affected
/// * `motion_force` - Motion force character (NUL if none)
/// * `op_type` - Operator type
///
/// # Returns
/// true if the delete might need to become linewise (caller must verify
/// the blank line condition)
#[must_use]
pub const fn should_check_linewise_delete(
    motion_type: MotionType,
    is_visual: bool,
    line_count: c_int,
    motion_force: c_int,
    op_type: OpType,
) -> bool {
    // From ops.c:
    // if (oap->motion_type == kMTCharWise
    //     && !oap->is_VIsual
    //     && oap->line_count > 1
    //     && oap->motion_force == NUL
    //     && oap->op_type == OP_DELETE)
    matches!(motion_type, MotionType::CharWise)
        && !is_visual
        && line_count > 1
        && motion_force == 0
        && matches!(op_type, OpType::Delete)
}

/// Check if operating on an empty region in an empty line.
///
/// This is an error condition in Vi-compatible mode.
///
/// # Arguments
/// * `motion_type` - Current motion type
/// * `line_count` - Number of lines affected
/// * `op_type` - Operator type
/// * `line_is_empty` - Whether the line is empty
///
/// # Returns
/// true if this is an error condition (empty line, non-linewise)
#[must_use]
pub const fn is_empty_line_delete(
    motion_type: MotionType,
    line_count: c_int,
    op_type: OpType,
    line_is_empty: bool,
) -> bool {
    // From ops.c:
    // if (oap->motion_type != kMTLineWise
    //     && oap->line_count == 1
    //     && oap->op_type == OP_DELETE
    //     && *ml_get(oap->start.lnum) == NUL)
    !matches!(motion_type, MotionType::LineWise)
        && line_count == 1
        && matches!(op_type, OpType::Delete)
        && line_is_empty
}

/// Calculate the number of bytes to delete on the last line of a multi-line delete.
///
/// # Arguments
/// * `end_col` - Ending column
/// * `inclusive` - Whether the motion is inclusive
///
/// # Returns
/// Number of bytes to delete from start of line
#[must_use]
#[inline]
pub const fn calc_delete_from_start(end_col: c_int, inclusive: bool) -> c_int {
    // int n = (oap->end.col + 1 - !oap->inclusive);
    let adjustment = if inclusive { 0 } else { 1 };
    let n = end_col + 1 - adjustment;
    if n < 0 {
        0
    } else {
        n
    }
}

/// Calculate size of block delete replacement.
///
/// When deleting in block mode, the deleted text is replaced with spaces.
///
/// # Arguments
/// * `textlen` - Length of text being deleted
/// * `startspaces` - Spaces at start (for tab handling)
/// * `endspaces` - Spaces at end (for tab handling)
///
/// # Returns
/// Number of characters deleted (may be negative if tabs expand)
#[must_use]
#[inline]
pub const fn calc_block_delete_chars(
    textlen: c_int,
    startspaces: c_int,
    endspaces: c_int,
) -> c_int {
    // int n = bd.textlen - bd.startspaces - bd.endspaces;
    textlen - startspaces - endspaces
}

/// Calculate new line size after block delete.
///
/// # Arguments
/// * `old_len` - Original line length
/// * `textlen` - Length of deleted text
/// * `startspaces` - Spaces at start
/// * `endspaces` - Spaces at end
///
/// # Returns
/// Size of new line
#[must_use]
#[inline]
pub const fn calc_block_delete_newlen(
    old_len: usize,
    textlen: usize,
    startspaces: usize,
    endspaces: usize,
) -> usize {
    // ml_get_len(lnum) - n + 1
    // where n = textlen - startspaces - endspaces
    // So: old_len - (textlen - startspaces - endspaces) + 1
    //   = old_len - textlen + startspaces + endspaces + 1
    old_len.saturating_sub(textlen) + startspaces + endspaces + 1
}

/// Determine which register to use for deleted text.
///
/// Returns flags indicating where deleted text should be stored.
///
/// # Arguments
/// * `regname` - Specified register name (0 for default)
/// * `motion_type` - Motion type
/// * `line_count` - Number of lines deleted
/// * `use_reg_one` - Whether to force use of register 1
///
/// # Returns
/// `(use_numbered, use_small_delete)` - flags for register usage
#[must_use]
pub const fn determine_delete_register(
    regname: c_int,
    motion_type: MotionType,
    line_count: c_int,
    use_reg_one: bool,
) -> (bool, bool) {
    // Use numbered registers (1-9) if:
    // - linewise motion, OR
    // - line_count > 1, OR
    // - use_reg_one flag is set
    let use_numbered = matches!(motion_type, MotionType::LineWise) || line_count > 1 || use_reg_one;

    // Use small delete register (-) if:
    // - no named register specified (regname == 0)
    // - not linewise
    // - single line
    let use_small_delete =
        regname == 0 && !matches!(motion_type, MotionType::LineWise) && line_count == 1;

    (use_numbered, use_small_delete)
}

/// FFI wrapper for calc_delete_bytes.
#[no_mangle]
pub extern "C" fn rs_calc_delete_bytes(
    start_col: c_int,
    end_col: c_int,
    inclusive: c_int,
) -> c_int {
    calc_delete_bytes(start_col, end_col, inclusive != 0)
}

/// FFI wrapper for calc_delete_bytes_virtual.
#[no_mangle]
pub extern "C" fn rs_calc_delete_bytes_virtual(
    start_col: c_int,
    end_col: c_int,
    inclusive: c_int,
    end_coladd: c_int,
    start_coladd: c_int,
    line_len: c_int,
) -> c_int {
    calc_delete_bytes_virtual(
        start_col,
        end_col,
        inclusive != 0,
        end_coladd,
        start_coladd,
        line_len,
    )
}

/// FFI wrapper for calc_delete_from_start.
#[no_mangle]
pub extern "C" fn rs_calc_delete_from_start(end_col: c_int, inclusive: c_int) -> c_int {
    calc_delete_from_start(end_col, inclusive != 0)
}

/// FFI wrapper for calc_block_delete_chars.
#[no_mangle]
pub extern "C" fn rs_calc_block_delete_chars(
    textlen: c_int,
    startspaces: c_int,
    endspaces: c_int,
) -> c_int {
    calc_block_delete_chars(textlen, startspaces, endspaces)
}

// =============================================================================
// Additional Delete Helpers
// =============================================================================

/// Calculate new cursor column after block delete.
///
/// In block delete, cursor moves to textcol + startspaces.
///
/// # Arguments
/// * `textcol` - Column where block starts
/// * `startspaces` - Number of start spaces
///
/// # Returns
/// New cursor column
#[must_use]
#[inline]
pub const fn calc_block_delete_cursor_col(textcol: c_int, startspaces: c_int) -> c_int {
    textcol + startspaces
}

/// Check if block delete should skip this line (nothing to delete).
///
/// # Arguments
/// * `textlen` - Length of text in block on this line
///
/// # Returns
/// true if this line should be skipped
#[must_use]
#[inline]
pub const fn should_skip_block_delete_line(textlen: c_int) -> bool {
    textlen == 0
}

/// Calculate size for block delete replacement buffer.
///
/// # Arguments
/// * `old_len` - Original line length
/// * `delete_chars` - Number of characters being deleted (textlen - startspaces - endspaces)
///
/// # Returns
/// Size of new buffer needed
#[must_use]
#[inline]
pub const fn calc_block_delete_buffer_size(old_len: usize, delete_chars: usize) -> usize {
    // ml_get_len(lnum) - n + 1 where n = delete_chars
    old_len.saturating_sub(delete_chars) + 1
}

/// Check if a multi-line charwise delete should be converted to linewise.
///
/// This is a Vi-compatible behavior where if the delete spans multiple lines
/// and results in a blank line, it becomes linewise.
///
/// # Arguments
/// * `motion_type` - Motion type
/// * `is_visual` - Whether in visual mode
/// * `line_count` - Number of lines affected
/// * `motion_force` - Motion force character
/// * `op_type` - Operator type
///
/// # Returns
/// true if we should check for linewise conversion
#[must_use]
#[inline]
pub const fn should_convert_to_linewise(
    motion_type: MotionType,
    is_visual: bool,
    line_count: c_int,
    motion_force: c_int,
    op_type: OpType,
) -> bool {
    // Same as should_check_linewise_delete, but clearer naming
    should_check_linewise_delete(motion_type, is_visual, line_count, motion_force, op_type)
}

/// Calculate the dollar display position for change operator.
///
/// When 'cpoptions' contains '$', we display '$' at end of change.
///
/// # Arguments
/// * `end_col` - End column
/// * `inclusive` - Whether motion is inclusive
///
/// # Returns
/// Column for dollar display
#[must_use]
#[inline]
pub const fn calc_dollar_display_col(end_col: c_int, inclusive: bool) -> c_int {
    let adjustment = if inclusive { 0 } else { 1 };
    end_col - adjustment
}

/// Check if the delete should use the black hole register.
///
/// The black hole register '_' discards deleted text.
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if using black hole register
#[must_use]
#[inline]
pub const fn is_black_hole_register(regname: c_int) -> bool {
    regname == b'_' as c_int
}

/// Calculate operation end column for block mode.
///
/// In block mode, op_end.col is set to op_start.col.
///
/// # Arguments
/// * `start_col` - Start column
///
/// # Returns
/// End column for marks
#[must_use]
#[inline]
pub const fn calc_block_op_end_col(start_col: c_int) -> c_int {
    start_col
}

/// Check if cursor is at end of line for virtual edit adjustments.
///
/// # Arguments
/// * `col` - Current column
/// * `line_len` - Line length
///
/// # Returns
/// true if cursor is at or past end of line
#[must_use]
#[inline]
pub const fn is_at_line_end(col: c_int, line_len: c_int) -> bool {
    let threshold = if line_len > 1 { line_len - 1 } else { 0 };
    col >= threshold
}

/// Calculate bytes to delete in multi-line charwise delete (last line).
///
/// # Arguments
/// * `end_col` - End column
/// * `inclusive` - Whether motion is inclusive
///
/// # Returns
/// Number of bytes to delete from start of last line
#[must_use]
#[inline]
pub const fn calc_multiline_delete_last_line(end_col: c_int, inclusive: bool) -> c_int {
    // int n = (oap->end.col + 1 - !oap->inclusive);
    calc_delete_from_start(end_col, inclusive)
}

// =============================================================================
// FFI Wrappers for Additional Helpers
// =============================================================================

/// FFI wrapper for calc_block_delete_cursor_col.
#[no_mangle]
pub extern "C" fn rs_calc_block_delete_cursor_col(textcol: c_int, startspaces: c_int) -> c_int {
    calc_block_delete_cursor_col(textcol, startspaces)
}

/// FFI wrapper for should_skip_block_delete_line.
#[no_mangle]
pub extern "C" fn rs_should_skip_block_delete_line(textlen: c_int) -> c_int {
    c_int::from(should_skip_block_delete_line(textlen))
}

/// FFI wrapper for calc_dollar_display_col.
#[no_mangle]
pub extern "C" fn rs_calc_dollar_display_col(end_col: c_int, inclusive: c_int) -> c_int {
    calc_dollar_display_col(end_col, inclusive != 0)
}

/// FFI wrapper for is_black_hole_register.
#[no_mangle]
pub extern "C" fn rs_is_black_hole_register(regname: c_int) -> c_int {
    c_int::from(is_black_hole_register(regname))
}

/// FFI wrapper for calc_block_op_end_col.
#[no_mangle]
pub extern "C" fn rs_calc_block_op_end_col(start_col: c_int) -> c_int {
    calc_block_op_end_col(start_col)
}

/// FFI wrapper for is_at_line_end.
#[no_mangle]
pub extern "C" fn rs_is_at_line_end(col: c_int, line_len: c_int) -> c_int {
    c_int::from(is_at_line_end(col, line_len))
}

/// FFI wrapper for should_check_linewise_delete.
#[no_mangle]
pub extern "C" fn rs_should_check_linewise_delete(
    motion_type: c_int,
    is_visual: c_int,
    line_count: c_int,
    motion_force: c_int,
    op_type: c_int,
) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(should_check_linewise_delete(
        mt,
        is_visual != 0,
        line_count,
        motion_force,
        op,
    ))
}

/// FFI wrapper for is_empty_line_delete.
#[no_mangle]
pub extern "C" fn rs_is_empty_line_delete(
    motion_type: c_int,
    line_count: c_int,
    op_type: c_int,
    line_is_empty: c_int,
) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(is_empty_line_delete(mt, line_count, op, line_is_empty != 0))
}

/// FFI wrapper for determine_delete_register.
///
/// # Safety
/// `use_numbered_out` and `use_small_delete_out` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_determine_delete_register(
    regname: c_int,
    motion_type: c_int,
    line_count: c_int,
    use_reg_one: c_int,
    use_numbered_out: *mut c_int,
    use_small_delete_out: *mut c_int,
) {
    let mt = MotionType::from_raw(motion_type);
    let (use_numbered, use_small_delete) =
        determine_delete_register(regname, mt, line_count, use_reg_one != 0);

    // SAFETY: Caller guarantees pointer validity
    if !use_numbered_out.is_null() {
        unsafe {
            *use_numbered_out = c_int::from(use_numbered);
        }
    }
    if !use_small_delete_out.is_null() {
        unsafe {
            *use_small_delete_out = c_int::from(use_small_delete);
        }
    }
}

// =============================================================================
// Phase O1: Additional Delete Operation Helpers
// =============================================================================

/// Check if we should copy deleted text to a register.
///
/// Deleted text is NOT copied when:
/// - Using black hole register '_'
///
/// # Arguments
/// * `regname` - Register name
///
/// # Returns
/// true if text should be copied to register
#[must_use]
#[inline]
pub const fn should_copy_to_register(regname: c_int) -> bool {
    regname != b'_' as c_int
}

/// Check if the delete operation needs undo save.
///
/// # Arguments
/// * `motion_type` - Motion type
/// * `start_lnum` - Starting line number
/// * `end_lnum` - Ending line number
///
/// # Returns
/// `(start_save, end_save)` - Line numbers for undo save
#[must_use]
#[inline]
pub const fn calc_delete_undo_range(
    motion_type: MotionType,
    start_lnum: c_int,
    end_lnum: c_int,
) -> (c_int, c_int) {
    match motion_type {
        MotionType::BlockWise | MotionType::LineWise => (start_lnum - 1, end_lnum + 1),
        MotionType::CharWise | MotionType::Unknown => (start_lnum - 1, start_lnum + 1),
    }
}

/// Calculate if a multiline charwise delete should use join.
///
/// When deleting across multiple lines charwise, we:
/// 1. Truncate the first line
/// 2. Delete middle lines
/// 3. Delete from start of last line
/// 4. Join the remaining parts
///
/// # Arguments
/// * `line_count` - Number of lines affected
///
/// # Returns
/// true if join is needed
#[must_use]
#[inline]
pub const fn needs_join_after_delete(line_count: c_int) -> bool {
    line_count > 1
}

/// Calculate the number of middle lines to delete.
///
/// In a multiline delete, middle lines (all but first and last) are deleted.
///
/// # Arguments
/// * `line_count` - Total number of lines affected
///
/// # Returns
/// Number of lines to delete with del_lines()
#[must_use]
#[inline]
pub const fn calc_middle_lines_to_delete(line_count: c_int) -> c_int {
    if line_count <= 2 {
        0
    } else {
        line_count - 2
    }
}

/// Check if delete should skip cursor coladd adjustment.
///
/// In virtual edit mode, when the cursor is on a real character,
/// coladd should be reset.
///
/// # Arguments
/// * `gchar_is_nul` - Whether gchar_cursor() == NUL
///
/// # Returns
/// true if coladd should be reset
#[must_use]
#[inline]
pub const fn should_reset_coladd(gchar_is_nul: bool) -> bool {
    !gchar_is_nul
}

/// Calculate extmark splice parameters for block delete.
///
/// # Arguments
/// * `textcol` - Column where block starts
/// * `textlen` - Length of deleted text
/// * `startspaces` - Start spaces
/// * `endspaces` - End spaces
///
/// # Returns
/// `(old_extent, new_extent)` - Parameters for extmark_splice_cols
#[must_use]
#[inline]
pub const fn calc_block_delete_extmark_params(
    textlen: c_int,
    startspaces: c_int,
    endspaces: c_int,
) -> (c_int, c_int) {
    (textlen, startspaces + endspaces)
}

/// Check if we should beep for operating on empty region.
///
/// Beeping happens when:
/// - 'E' is in 'cpoptions'
/// - Operation is on empty region
///
/// # Arguments
/// * `cpo_has_e` - Whether CPO_EMPTYREGION is set
///
/// # Returns
/// true if should beep
#[must_use]
#[inline]
pub const fn should_beep_empty_region(cpo_has_e: bool) -> bool {
    cpo_has_e
}

/// Calculate delete region byte count for extmark splice.
///
/// # Arguments
/// * `line_count` - Number of lines
/// * `end_col` - End column in last line
/// * `inclusive` - Whether motion is inclusive
///
/// # Returns
/// `(lines_to_delete, last_line_cols, inclusive_adjustment)` for extmark tracking
#[must_use]
#[inline]
pub const fn calc_delete_region_info(
    line_count: c_int,
    end_col: c_int,
    inclusive: bool,
) -> (c_int, c_int, c_int) {
    let inclusive_adj = if inclusive { 1 } else { 0 };
    let not_inclusive_adj = if inclusive { 0 } else { 1 };
    // Returns (lines_to_delete, last_line_cols, inclusive_adjustment)
    (
        line_count - 1,
        end_col + 1 - not_inclusive_adj,
        inclusive_adj,
    )
}

/// Determine block delete replacement character count.
///
/// When deleting in block mode, the deleted chars may be replaced with spaces.
///
/// # Arguments
/// * `startspaces` - Spaces at start
/// * `endspaces` - Spaces at end
///
/// # Returns
/// Total replacement spaces
#[must_use]
#[inline]
pub const fn calc_block_delete_replacement_spaces(startspaces: c_int, endspaces: c_int) -> c_int {
    startspaces + endspaces
}

/// Check if virtual edit adjustments are needed for delete.
///
/// # Arguments
/// * `virtual_op` - Whether in virtual edit mode
///
/// # Returns
/// true if virtual edit adjustments are needed
#[must_use]
#[inline]
pub const fn needs_virtual_edit_adjustments(virtual_op: bool) -> bool {
    virtual_op
}

/// Calculate the delete operation type for auto-format.
///
/// # Arguments
/// * `op_type` - Operator type
///
/// # Returns
/// true if auto_format should be called
#[must_use]
#[inline]
pub const fn should_call_auto_format(op_type: OpType) -> bool {
    matches!(op_type, OpType::Delete)
}

// =============================================================================
// FFI Wrappers for Phase O1 Additions
// =============================================================================

/// FFI wrapper for should_copy_to_register.
#[no_mangle]
pub extern "C" fn rs_should_copy_to_register(regname: c_int) -> c_int {
    c_int::from(should_copy_to_register(regname))
}

/// FFI wrapper for calc_delete_undo_range.
///
/// # Safety
/// `start_save_out` and `end_save_out` must be valid pointers if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_delete_undo_range(
    motion_type: c_int,
    start_lnum: c_int,
    end_lnum: c_int,
    start_save_out: *mut c_int,
    end_save_out: *mut c_int,
) {
    let mt = MotionType::from_raw(motion_type);
    let (start_save, end_save) = calc_delete_undo_range(mt, start_lnum, end_lnum);

    if !start_save_out.is_null() {
        unsafe {
            *start_save_out = start_save;
        }
    }
    if !end_save_out.is_null() {
        unsafe {
            *end_save_out = end_save;
        }
    }
}

/// FFI wrapper for needs_join_after_delete.
#[no_mangle]
pub extern "C" fn rs_needs_join_after_delete(line_count: c_int) -> c_int {
    c_int::from(needs_join_after_delete(line_count))
}

/// FFI wrapper for calc_middle_lines_to_delete.
#[no_mangle]
pub extern "C" fn rs_calc_middle_lines_to_delete(line_count: c_int) -> c_int {
    calc_middle_lines_to_delete(line_count)
}

/// FFI wrapper for should_reset_coladd.
#[no_mangle]
pub extern "C" fn rs_should_reset_coladd(gchar_is_nul: c_int) -> c_int {
    c_int::from(should_reset_coladd(gchar_is_nul != 0))
}

/// FFI wrapper for calc_block_delete_extmark_params.
///
/// # Safety
/// `old_extent_out` and `new_extent_out` must be valid pointers if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_block_delete_extmark_params(
    textlen: c_int,
    startspaces: c_int,
    endspaces: c_int,
    old_extent_out: *mut c_int,
    new_extent_out: *mut c_int,
) {
    let (old_extent, new_extent) =
        calc_block_delete_extmark_params(textlen, startspaces, endspaces);

    if !old_extent_out.is_null() {
        unsafe {
            *old_extent_out = old_extent;
        }
    }
    if !new_extent_out.is_null() {
        unsafe {
            *new_extent_out = new_extent;
        }
    }
}

/// FFI wrapper for should_beep_empty_region.
#[no_mangle]
pub extern "C" fn rs_should_beep_empty_region(cpo_has_e: c_int) -> c_int {
    c_int::from(should_beep_empty_region(cpo_has_e != 0))
}

/// FFI wrapper for calc_block_delete_replacement_spaces.
#[no_mangle]
pub extern "C" fn rs_calc_block_delete_replacement_spaces(
    startspaces: c_int,
    endspaces: c_int,
) -> c_int {
    calc_block_delete_replacement_spaces(startspaces, endspaces)
}

/// FFI wrapper for needs_virtual_edit_adjustments.
#[no_mangle]
pub extern "C" fn rs_needs_virtual_edit_adjustments(virtual_op: c_int) -> c_int {
    c_int::from(needs_virtual_edit_adjustments(virtual_op != 0))
}

/// FFI wrapper for should_call_auto_format.
#[no_mangle]
pub extern "C" fn rs_should_call_auto_format(op_type: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(should_call_auto_format(op))
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_delete_bytes() {
        // Inclusive: end - start + 1
        assert_eq!(calc_delete_bytes(0, 9, true), 10);
        assert_eq!(calc_delete_bytes(5, 10, true), 6);

        // Non-inclusive: end - start
        assert_eq!(calc_delete_bytes(0, 9, false), 9);
        assert_eq!(calc_delete_bytes(5, 10, false), 5);

        // Single column
        assert_eq!(calc_delete_bytes(5, 5, true), 1);
        assert_eq!(calc_delete_bytes(5, 5, false), 0);

        // Edge case: end before start (clamped to 0)
        assert_eq!(calc_delete_bytes(10, 5, true), 0);
    }

    #[test]
    fn test_calc_delete_bytes_virtual() {
        // Basic case (same as non-virtual)
        assert_eq!(calc_delete_bytes_virtual(0, 9, true, 0, 0, 100), 10);

        // With end coladd at line end
        // end_coladd != 0 && end_col >= len-1 && !(start_coladd && end_col >= len-1)
        assert_eq!(calc_delete_bytes_virtual(0, 99, true, 1, 0, 100), 101);

        // With both coladds at line end (no increment)
        assert_eq!(calc_delete_bytes_virtual(0, 99, true, 1, 1, 100), 100);

        // Zero bytes but different coladds
        assert_eq!(calc_delete_bytes_virtual(5, 5, false, 0, 1, 100), 1);
    }

    #[test]
    fn test_should_check_linewise_delete() {
        // Should check: charwise, not visual, multi-line, no force, delete op
        assert!(should_check_linewise_delete(
            MotionType::CharWise,
            false,
            2,
            0,
            OpType::Delete,
        ));

        // Should NOT check: linewise
        assert!(!should_check_linewise_delete(
            MotionType::LineWise,
            false,
            2,
            0,
            OpType::Delete,
        ));

        // Should NOT check: visual mode
        assert!(!should_check_linewise_delete(
            MotionType::CharWise,
            true,
            2,
            0,
            OpType::Delete,
        ));

        // Should NOT check: single line
        assert!(!should_check_linewise_delete(
            MotionType::CharWise,
            false,
            1,
            0,
            OpType::Delete,
        ));

        // Should NOT check: motion force set
        assert!(!should_check_linewise_delete(
            MotionType::CharWise,
            false,
            2,
            b'v' as c_int,
            OpType::Delete,
        ));

        // Should NOT check: not delete op
        assert!(!should_check_linewise_delete(
            MotionType::CharWise,
            false,
            2,
            0,
            OpType::Change,
        ));
    }

    #[test]
    fn test_is_empty_line_delete() {
        // Empty line delete
        assert!(is_empty_line_delete(
            MotionType::CharWise,
            1,
            OpType::Delete,
            true,
        ));

        // Not empty line
        assert!(!is_empty_line_delete(
            MotionType::CharWise,
            1,
            OpType::Delete,
            false,
        ));

        // Linewise (always ok)
        assert!(!is_empty_line_delete(
            MotionType::LineWise,
            1,
            OpType::Delete,
            true,
        ));

        // Multiple lines
        assert!(!is_empty_line_delete(
            MotionType::CharWise,
            2,
            OpType::Delete,
            true,
        ));

        // Change op (not delete)
        assert!(!is_empty_line_delete(
            MotionType::CharWise,
            1,
            OpType::Change,
            true,
        ));
    }

    #[test]
    fn test_calc_delete_from_start() {
        // Inclusive: col + 1
        assert_eq!(calc_delete_from_start(9, true), 10);
        assert_eq!(calc_delete_from_start(0, true), 1);

        // Non-inclusive: col
        assert_eq!(calc_delete_from_start(9, false), 9);
        assert_eq!(calc_delete_from_start(0, false), 0);
    }

    #[test]
    fn test_calc_block_delete_chars() {
        // Simple case: no spaces
        assert_eq!(calc_block_delete_chars(10, 0, 0), 10);

        // With spaces (tab handling)
        assert_eq!(calc_block_delete_chars(10, 2, 3), 5);

        // Can be negative (tabs expand)
        assert_eq!(calc_block_delete_chars(5, 3, 3), -1);
    }

    #[test]
    fn test_calc_block_delete_newlen() {
        // Simple: remove 10 chars from 100 char line
        assert_eq!(calc_block_delete_newlen(100, 10, 0, 0), 91);

        // With spaces added back
        assert_eq!(calc_block_delete_newlen(100, 10, 2, 3), 96);

        // Underflow protection
        assert_eq!(calc_block_delete_newlen(5, 10, 0, 0), 1);
    }

    #[test]
    fn test_determine_delete_register() {
        // Linewise: use numbered
        let (numbered, small) = determine_delete_register(0, MotionType::LineWise, 1, false);
        assert!(numbered);
        assert!(!small);

        // Multi-line: use numbered
        let (numbered, small) = determine_delete_register(0, MotionType::CharWise, 2, false);
        assert!(numbered);
        assert!(!small);

        // use_reg_one: use numbered (but also small for single line charwise)
        let (numbered, small) = determine_delete_register(0, MotionType::CharWise, 1, true);
        assert!(numbered);
        assert!(small); // Both can be true - they're independent checks

        // Single line charwise with no register: use small delete
        let (numbered, small) = determine_delete_register(0, MotionType::CharWise, 1, false);
        assert!(!numbered);
        assert!(small);

        // Named register: neither numbered nor small
        let (numbered, small) =
            determine_delete_register(b'a' as c_int, MotionType::CharWise, 1, false);
        assert!(!numbered);
        assert!(!small);
    }

    // =========================================================================
    // Additional Helper Function Tests
    // =========================================================================

    #[test]
    fn test_calc_block_delete_cursor_col() {
        assert_eq!(calc_block_delete_cursor_col(10, 0), 10);
        assert_eq!(calc_block_delete_cursor_col(10, 5), 15);
        assert_eq!(calc_block_delete_cursor_col(0, 3), 3);
    }

    #[test]
    fn test_should_skip_block_delete_line() {
        assert!(should_skip_block_delete_line(0));
        assert!(!should_skip_block_delete_line(1));
        assert!(!should_skip_block_delete_line(10));
    }

    #[test]
    fn test_calc_block_delete_buffer_size() {
        // Simple case: old_len=100, delete_chars=10
        assert_eq!(calc_block_delete_buffer_size(100, 10), 91);

        // Underflow protection
        assert_eq!(calc_block_delete_buffer_size(5, 10), 1);

        // No deletion
        assert_eq!(calc_block_delete_buffer_size(50, 0), 51);
    }

    #[test]
    fn test_calc_dollar_display_col() {
        // Inclusive: no adjustment
        assert_eq!(calc_dollar_display_col(10, true), 10);

        // Non-inclusive: subtract 1
        assert_eq!(calc_dollar_display_col(10, false), 9);

        // Edge case
        assert_eq!(calc_dollar_display_col(0, true), 0);
        assert_eq!(calc_dollar_display_col(0, false), -1);
    }

    #[test]
    fn test_is_black_hole_register() {
        assert!(is_black_hole_register(b'_' as c_int));
        assert!(!is_black_hole_register(b'a' as c_int));
        assert!(!is_black_hole_register(0));
        assert!(!is_black_hole_register(b'"' as c_int));
    }

    #[test]
    fn test_calc_block_op_end_col() {
        assert_eq!(calc_block_op_end_col(10), 10);
        assert_eq!(calc_block_op_end_col(0), 0);
        assert_eq!(calc_block_op_end_col(50), 50);
    }

    #[test]
    fn test_is_at_line_end() {
        // At end
        assert!(is_at_line_end(9, 10));
        assert!(is_at_line_end(10, 10)); // Past end

        // Not at end
        assert!(!is_at_line_end(5, 10));
        assert!(!is_at_line_end(0, 10));

        // Empty line
        assert!(is_at_line_end(0, 0));
        assert!(is_at_line_end(0, 1));
    }

    #[test]
    fn test_calc_multiline_delete_last_line() {
        // Same as calc_delete_from_start
        assert_eq!(calc_multiline_delete_last_line(9, true), 10);
        assert_eq!(calc_multiline_delete_last_line(9, false), 9);
    }

    #[test]
    fn test_additional_ffi_wrappers() {
        // rs_calc_block_delete_cursor_col
        assert_eq!(rs_calc_block_delete_cursor_col(10, 5), 15);

        // rs_should_skip_block_delete_line
        assert_eq!(rs_should_skip_block_delete_line(0), 1);
        assert_eq!(rs_should_skip_block_delete_line(5), 0);

        // rs_calc_dollar_display_col
        assert_eq!(rs_calc_dollar_display_col(10, 1), 10);
        assert_eq!(rs_calc_dollar_display_col(10, 0), 9);

        // rs_is_black_hole_register
        assert_eq!(rs_is_black_hole_register(b'_' as c_int), 1);
        assert_eq!(rs_is_black_hole_register(b'a' as c_int), 0);

        // rs_calc_block_op_end_col
        assert_eq!(rs_calc_block_op_end_col(10), 10);

        // rs_is_at_line_end
        assert_eq!(rs_is_at_line_end(9, 10), 1);
        assert_eq!(rs_is_at_line_end(5, 10), 0);

        // rs_should_check_linewise_delete (charwise=0, delete=1)
        assert_eq!(rs_should_check_linewise_delete(0, 0, 2, 0, 1), 1);
        assert_eq!(rs_should_check_linewise_delete(0, 1, 2, 0, 1), 0); // visual

        // rs_is_empty_line_delete (charwise=0, delete=1)
        assert_eq!(rs_is_empty_line_delete(0, 1, 1, 1), 1);
        assert_eq!(rs_is_empty_line_delete(0, 1, 1, 0), 0); // not empty
    }

    // =========================================================================
    // Phase O1 Addition Tests
    // =========================================================================

    #[test]
    fn test_should_copy_to_register() {
        // Black hole register: don't copy
        assert!(!should_copy_to_register(b'_' as c_int));

        // All other registers: copy
        assert!(should_copy_to_register(0)); // unnamed
        assert!(should_copy_to_register(b'a' as c_int));
        assert!(should_copy_to_register(b'"' as c_int));
        assert!(should_copy_to_register(b'1' as c_int));
    }

    #[test]
    fn test_calc_delete_undo_range() {
        // Block/linewise: full range
        let (start, end) = calc_delete_undo_range(MotionType::BlockWise, 10, 20);
        assert_eq!(start, 9);
        assert_eq!(end, 21);

        let (start, end) = calc_delete_undo_range(MotionType::LineWise, 10, 20);
        assert_eq!(start, 9);
        assert_eq!(end, 21);

        // Charwise: single line range
        let (start, end) = calc_delete_undo_range(MotionType::CharWise, 10, 20);
        assert_eq!(start, 9);
        assert_eq!(end, 11);
    }

    #[test]
    fn test_needs_join_after_delete() {
        assert!(!needs_join_after_delete(1));
        assert!(needs_join_after_delete(2));
        assert!(needs_join_after_delete(10));
    }

    #[test]
    fn test_calc_middle_lines_to_delete() {
        assert_eq!(calc_middle_lines_to_delete(1), 0);
        assert_eq!(calc_middle_lines_to_delete(2), 0);
        assert_eq!(calc_middle_lines_to_delete(3), 1);
        assert_eq!(calc_middle_lines_to_delete(10), 8);
    }

    #[test]
    fn test_should_reset_coladd() {
        assert!(should_reset_coladd(false)); // not NUL means reset
        assert!(!should_reset_coladd(true)); // NUL means don't reset
    }

    #[test]
    fn test_calc_block_delete_extmark_params() {
        let (old, new) = calc_block_delete_extmark_params(10, 2, 3);
        assert_eq!(old, 10);
        assert_eq!(new, 5);
    }

    #[test]
    fn test_should_beep_empty_region() {
        assert!(should_beep_empty_region(true));
        assert!(!should_beep_empty_region(false));
    }

    #[test]
    fn test_calc_block_delete_replacement_spaces() {
        assert_eq!(calc_block_delete_replacement_spaces(2, 3), 5);
        assert_eq!(calc_block_delete_replacement_spaces(0, 0), 0);
    }

    #[test]
    fn test_needs_virtual_edit_adjustments() {
        assert!(needs_virtual_edit_adjustments(true));
        assert!(!needs_virtual_edit_adjustments(false));
    }

    #[test]
    fn test_should_call_auto_format() {
        assert!(should_call_auto_format(OpType::Delete));
        assert!(!should_call_auto_format(OpType::Change));
        assert!(!should_call_auto_format(OpType::Yank));
    }

    #[test]
    fn test_phase_o1_ffi_wrappers() {
        // rs_should_copy_to_register
        assert_eq!(rs_should_copy_to_register(b'_' as c_int), 0);
        assert_eq!(rs_should_copy_to_register(b'a' as c_int), 1);

        // rs_needs_join_after_delete
        assert_eq!(rs_needs_join_after_delete(1), 0);
        assert_eq!(rs_needs_join_after_delete(2), 1);

        // rs_calc_middle_lines_to_delete
        assert_eq!(rs_calc_middle_lines_to_delete(3), 1);

        // rs_should_reset_coladd
        assert_eq!(rs_should_reset_coladd(0), 1);
        assert_eq!(rs_should_reset_coladd(1), 0);

        // rs_should_beep_empty_region
        assert_eq!(rs_should_beep_empty_region(1), 1);
        assert_eq!(rs_should_beep_empty_region(0), 0);

        // rs_calc_block_delete_replacement_spaces
        assert_eq!(rs_calc_block_delete_replacement_spaces(2, 3), 5);

        // rs_needs_virtual_edit_adjustments
        assert_eq!(rs_needs_virtual_edit_adjustments(1), 1);
        assert_eq!(rs_needs_virtual_edit_adjustments(0), 0);

        // rs_should_call_auto_format (delete=1)
        assert_eq!(rs_should_call_auto_format(1), 1);
        assert_eq!(rs_should_call_auto_format(2), 0); // yank
    }
}
