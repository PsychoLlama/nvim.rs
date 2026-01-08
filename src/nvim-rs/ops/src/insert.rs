//! Insert and Change operations (I, A, c)
//!
//! This module implements calculation logic for insert and change operators.

use std::ffi::c_int;

use crate::types::{MotionType, OpType};

/// Calculate pre-textlen for block insert/append operations.
///
/// pre_textlen is the amount of text after the block position on the first line
/// before the insert begins. It's used to determine how much new text was inserted.
///
/// # Arguments
/// * `line_len` - Length of the current line
/// * `textcol` - Column where block starts
/// * `textlen` - Length of block text
/// * `op_type` - OP_INSERT or OP_APPEND
///
/// # Returns
/// The pre_textlen value
#[must_use]
#[inline]
pub const fn calc_pre_textlen(
    line_len: c_int,
    textcol: c_int,
    textlen: c_int,
    op_type: OpType,
) -> c_int {
    // pre_textlen = ml_get_len(oap->start.lnum) - bd.textcol;
    // if (oap->op_type == OP_APPEND) {
    //   pre_textlen -= bd.textlen;
    // }
    let mut pre_textlen = line_len - textcol;
    if matches!(op_type, OpType::Append) {
        pre_textlen -= textlen;
    }
    if pre_textlen < 0 {
        0
    } else {
        pre_textlen
    }
}

/// Calculate the length of inserted text.
///
/// After an insert operation, this calculates how many characters were inserted
/// by comparing the new line length to the original.
///
/// # Arguments
/// * `new_line_len` - Length of line after insert
/// * `pre_textlen` - pre_textlen value from before insert
///
/// # Returns
/// Length of inserted text
#[must_use]
#[inline]
pub const fn calc_inserted_len(new_line_len: c_int, pre_textlen: c_int) -> c_int {
    // ins_len = ml_get_len(oap->start.lnum) - pre_textlen;
    let ins_len = new_line_len - pre_textlen;
    if ins_len < 0 {
        0
    } else {
        ins_len
    }
}

/// Adjust block position for indent changes.
///
/// When auto-indent kicks in during a block insert, the text column needs
/// to be adjusted to account for the new indentation.
///
/// # Arguments
/// * `textcol` - Original text column
/// * `new_indent` - New indentation (columns)
/// * `old_indent` - Original indentation (columns)
///
/// # Returns
/// Adjusted text column
#[must_use]
#[inline]
pub const fn adjust_textcol_for_indent(
    textcol: c_int,
    new_indent: c_int,
    old_indent: c_int,
) -> c_int {
    // bd.textcol += new_indent - pre_indent;
    textcol + (new_indent - old_indent)
}

/// Calculate new line size for block change operation.
///
/// When changing block text on subsequent lines, calculates the new line size.
///
/// # Arguments
/// * `old_len` - Original line length
/// * `coladd` - Virtual column offset (for virtual edit)
/// * `ins_len` - Length of text to insert
///
/// # Returns
/// Size needed for new line
#[must_use]
#[inline]
pub const fn calc_block_change_newlen(old_len: usize, coladd: usize, ins_len: usize) -> usize {
    // xmalloc(ml_get_len(linenr) + vpos.coladd + ins_len + 1)
    old_len + coladd + ins_len + 1
}

/// Determine if cursor should move right after op_append positioning.
///
/// For OP_APPEND in block mode, the cursor needs to move to the right
/// of the block. This function determines if movement should occur.
///
/// # Arguments
/// * `cursor_col` - Current cursor column
/// * `textcol` - Block text column
/// * `textlen` - Block text length
///
/// # Returns
/// true if cursor should advance
#[must_use]
#[inline]
pub const fn should_advance_cursor_append(
    cursor_col: c_int,
    textcol: c_int,
    textlen: c_int,
) -> bool {
    // while (*get_cursor_pos_ptr() != NUL
    //        && (curwin->w_cursor.col < bd.textcol + bd.textlen))
    //   curwin->w_cursor.col++;
    cursor_col < textcol + textlen
}

/// Calculate the target column for block append.
///
/// For OP_APPEND, returns the target column position (textcol + textlen).
///
/// # Arguments
/// * `textcol` - Block text column
/// * `textlen` - Block text length
///
/// # Returns
/// Target column for append
#[must_use]
#[inline]
pub const fn calc_append_target_col(textcol: c_int, textlen: c_int) -> c_int {
    textcol + textlen
}

/// Check if change operation needs cursor increment.
///
/// After deleting text in a change operation, the cursor may need to be
/// incremented if we're not at the end of the line.
///
/// # Arguments
/// * `original_col` - Original cursor column before delete
/// * `current_col` - Current cursor column after delete
/// * `line_is_empty` - Whether the line is now empty
/// * `virtual_op` - Whether virtual edit is active
///
/// # Returns
/// true if cursor should be incremented
#[must_use]
#[inline]
pub const fn should_inc_cursor_after_change(
    original_col: c_int,
    current_col: c_int,
    line_is_empty: bool,
    virtual_op: bool,
) -> bool {
    // if ((l > curwin->w_cursor.col) && !LINEEMPTY(curwin->w_cursor.lnum)
    //     && !virtual_op)
    //   inc_cursor();
    original_col > current_col && !line_is_empty && !virtual_op
}

/// Determine starting column for change operation based on motion type.
///
/// # Arguments
/// * `motion_type` - The motion type
/// * `start_col` - Original start column
///
/// # Returns
/// Effective starting column
#[must_use]
#[inline]
pub const fn calc_change_start_col(motion_type: MotionType, start_col: c_int) -> c_int {
    // if (oap->motion_type == kMTLineWise) {
    //   l = 0;
    // }
    if matches!(motion_type, MotionType::LineWise) {
        0
    } else {
        start_col
    }
}

/// Check if block change should propagate to subsequent lines.
///
/// # Arguments
/// * `motion_type` - The motion type
/// * `start_lnum` - Starting line number
/// * `end_lnum` - Ending line number
/// * `got_int` - Whether Ctrl-C was pressed
///
/// # Returns
/// true if changes should propagate to other lines
#[must_use]
#[inline]
pub const fn should_propagate_block_change(
    motion_type: MotionType,
    start_lnum: c_int,
    end_lnum: c_int,
    got_int: bool,
) -> bool {
    // if (oap->motion_type == kMTBlockWise
    //     && oap->start.lnum != oap->end.lnum && !got_int)
    matches!(motion_type, MotionType::BlockWise) && start_lnum != end_lnum && !got_int
}

/// FFI wrapper for calc_pre_textlen.
#[no_mangle]
pub extern "C" fn rs_calc_pre_textlen(
    line_len: c_int,
    textcol: c_int,
    textlen: c_int,
    op_type: c_int,
) -> c_int {
    let op = crate::types::OpType::from_raw(op_type).unwrap_or(crate::types::OpType::Nop);
    calc_pre_textlen(line_len, textcol, textlen, op)
}

/// FFI wrapper for calc_inserted_len.
#[no_mangle]
pub extern "C" fn rs_calc_inserted_len(new_line_len: c_int, pre_textlen: c_int) -> c_int {
    calc_inserted_len(new_line_len, pre_textlen)
}

/// FFI wrapper for adjust_textcol_for_indent.
#[no_mangle]
pub extern "C" fn rs_adjust_textcol_for_indent(
    textcol: c_int,
    new_indent: c_int,
    old_indent: c_int,
) -> c_int {
    adjust_textcol_for_indent(textcol, new_indent, old_indent)
}

/// FFI wrapper for should_inc_cursor_after_change.
#[no_mangle]
pub extern "C" fn rs_should_inc_cursor_after_change(
    original_col: c_int,
    current_col: c_int,
    line_is_empty: c_int,
    virtual_op: c_int,
) -> c_int {
    c_int::from(should_inc_cursor_after_change(
        original_col,
        current_col,
        line_is_empty != 0,
        virtual_op != 0,
    ))
}

// =============================================================================
// Additional Insert Helpers
// =============================================================================

/// Check if virtual edit spaces need to be inserted before block_prep.
///
/// # Arguments
/// * `coladd` - Cursor coladd value
///
/// # Returns
/// true if virtual spaces need to be inserted
#[must_use]
#[inline]
pub const fn needs_virtual_spaces(coladd: c_int) -> bool {
    coladd > 0
}

/// Calculate target column for coladvance in block append.
///
/// For OP_APPEND, advance to end_vcol + 1.
/// For OP_INSERT, advance to current visual column.
///
/// # Arguments
/// * `op_type` - OP_INSERT or OP_APPEND
/// * `end_vcol` - End virtual column for append
/// * `cur_vcol` - Current visual column
///
/// # Returns
/// Target column for coladvance
#[must_use]
#[inline]
pub const fn calc_coladvance_target(op_type: OpType, end_vcol: c_int, cur_vcol: c_int) -> c_int {
    if matches!(op_type, OpType::Append) {
        end_vcol + 1
    } else {
        cur_vcol
    }
}

/// Check if first line needs extension for short block append.
///
/// # Arguments
/// * `is_short` - Whether the block line is short
/// * `is_max` - Whether the block extends to end of line
///
/// # Returns
/// true if the line needs extension
#[must_use]
#[inline]
pub const fn needs_short_line_extension(is_short: bool, is_max: bool) -> bool {
    is_short && !is_max
}

/// Adjust textlen for endspaces added.
///
/// # Arguments
/// * `textlen` - Current text length
/// * `endspaces` - Number of end spaces added
///
/// # Returns
/// Adjusted text length
#[must_use]
#[inline]
pub const fn adjust_textlen_for_endspaces(textlen: c_int, endspaces: c_int) -> c_int {
    textlen + endspaces
}

/// Check if cursor should be incremented after append positioning.
///
/// For non-block append mode, cursor is incremented if:
/// - Line is not empty
/// - Start vcol differs from end vcol
///
/// # Arguments
/// * `line_is_empty` - Whether the current line is empty
/// * `start_vcol` - Start virtual column
/// * `end_vcol` - End virtual column
///
/// # Returns
/// true if cursor should be incremented
#[must_use]
#[inline]
pub const fn should_inc_cursor_for_append(
    line_is_empty: bool,
    start_vcol: c_int,
    end_vcol: c_int,
) -> bool {
    !line_is_empty && start_vcol != end_vcol
}

/// Check if user moved off the start line (invalidates block propagation).
///
/// # Arguments
/// * `cursor_lnum` - Current cursor line number
/// * `start_lnum` - Original start line number
/// * `got_int` - Whether Ctrl-C was pressed
///
/// # Returns
/// true if block propagation should be aborted
#[must_use]
#[inline]
pub const fn should_abort_block_insert(
    cursor_lnum: c_int,
    start_lnum: c_int,
    got_int: bool,
) -> bool {
    cursor_lnum != start_lnum || got_int
}

/// Check if indent was added during insert.
///
/// # Arguments
/// * `op_start_col` - b_op_start.col value
/// * `ind_pre_col` - Indent column before insert
/// * `ind_post_col` - Indent column after insert
///
/// # Returns
/// true if indent was added
#[must_use]
#[inline]
pub const fn did_indent_increase(
    op_start_col: c_int,
    ind_pre_col: c_int,
    ind_post_col: c_int,
) -> bool {
    op_start_col > ind_pre_col && ind_post_col > ind_pre_col
}

/// Adjust block values for indent change.
///
/// # Arguments
/// * `textcol` - Current textcol
/// * `start_vcol` - Current start_vcol
/// * `col_delta` - Indent column change (post - pre)
/// * `vcol_delta` - Indent vcol change (post - pre)
///
/// # Returns
/// (new_textcol, new_start_vcol)
#[must_use]
#[inline]
pub const fn adjust_block_for_indent(
    textcol: c_int,
    start_vcol: c_int,
    col_delta: c_int,
    vcol_delta: c_int,
) -> (c_int, c_int) {
    let new_textcol = textcol + col_delta;
    let new_start_vcol = start_vcol + vcol_delta;
    (new_textcol, new_start_vcol)
}

/// Calculate insert length from line lengths.
///
/// # Arguments
/// * `line_len` - Current line length
/// * `pre_textlen` - Pre-insert text length
/// * `offset` - Cursor offset
///
/// # Returns
/// Calculated insert length (may be negative)
#[must_use]
#[inline]
pub const fn calc_ins_len(line_len: c_int, pre_textlen: c_int, offset: c_int) -> c_int {
    line_len - pre_textlen - offset
}

/// Check if block insert should proceed.
///
/// # Arguments
/// * `pre_textlen` - Pre-insert text length
/// * `ins_len` - Insert length
///
/// # Returns
/// true if block insert should proceed
#[must_use]
#[inline]
pub const fn should_do_block_insert(pre_textlen: c_int, ins_len: c_int) -> bool {
    pre_textlen >= 0 && ins_len > 0
}

/// Calculate the add offset for extracting inserted text.
///
/// # Arguments
/// * `textcol` - Block text column
/// * `textlen` - Block text length (for append)
/// * `op_type` - OP_INSERT or OP_APPEND
///
/// # Returns
/// Offset into line for inserted text
#[must_use]
#[inline]
pub const fn calc_insert_text_offset(textcol: c_int, textlen: c_int, op_type: OpType) -> c_int {
    if matches!(op_type, OpType::Append) {
        textcol + textlen
    } else {
        textcol
    }
}

// =============================================================================
// FFI Wrappers for Additional Helpers
// =============================================================================

/// FFI wrapper for needs_virtual_spaces.
#[no_mangle]
pub extern "C" fn rs_needs_virtual_spaces(coladd: c_int) -> c_int {
    c_int::from(needs_virtual_spaces(coladd))
}

/// FFI wrapper for calc_coladvance_target.
#[no_mangle]
pub extern "C" fn rs_calc_coladvance_target(
    op_type: c_int,
    end_vcol: c_int,
    cur_vcol: c_int,
) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    calc_coladvance_target(op, end_vcol, cur_vcol)
}

/// FFI wrapper for needs_short_line_extension.
#[no_mangle]
pub extern "C" fn rs_needs_short_line_extension(is_short: c_int, is_max: c_int) -> c_int {
    c_int::from(needs_short_line_extension(is_short != 0, is_max != 0))
}

/// FFI wrapper for adjust_textlen_for_endspaces.
#[no_mangle]
pub extern "C" fn rs_adjust_textlen_for_endspaces(textlen: c_int, endspaces: c_int) -> c_int {
    adjust_textlen_for_endspaces(textlen, endspaces)
}

/// FFI wrapper for should_inc_cursor_for_append.
#[no_mangle]
pub extern "C" fn rs_should_inc_cursor_for_append(
    line_is_empty: c_int,
    start_vcol: c_int,
    end_vcol: c_int,
) -> c_int {
    c_int::from(should_inc_cursor_for_append(
        line_is_empty != 0,
        start_vcol,
        end_vcol,
    ))
}

/// FFI wrapper for should_abort_block_insert.
#[no_mangle]
pub extern "C" fn rs_should_abort_block_insert(
    cursor_lnum: c_int,
    start_lnum: c_int,
    got_int: c_int,
) -> c_int {
    c_int::from(should_abort_block_insert(
        cursor_lnum,
        start_lnum,
        got_int != 0,
    ))
}

/// FFI wrapper for did_indent_increase.
#[no_mangle]
pub extern "C" fn rs_did_indent_increase(
    op_start_col: c_int,
    ind_pre_col: c_int,
    ind_post_col: c_int,
) -> c_int {
    c_int::from(did_indent_increase(op_start_col, ind_pre_col, ind_post_col))
}

/// FFI wrapper for calc_ins_len.
#[no_mangle]
pub extern "C" fn rs_calc_ins_len(line_len: c_int, pre_textlen: c_int, offset: c_int) -> c_int {
    calc_ins_len(line_len, pre_textlen, offset)
}

/// FFI wrapper for should_do_block_insert.
#[no_mangle]
pub extern "C" fn rs_should_do_block_insert(pre_textlen: c_int, ins_len: c_int) -> c_int {
    c_int::from(should_do_block_insert(pre_textlen, ins_len))
}

/// FFI wrapper for calc_insert_text_offset.
#[no_mangle]
pub extern "C" fn rs_calc_insert_text_offset(
    textcol: c_int,
    textlen: c_int,
    op_type: c_int,
) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    calc_insert_text_offset(textcol, textlen, op)
}

/// FFI wrapper for should_propagate_block_change.
#[no_mangle]
pub extern "C" fn rs_should_propagate_block_change(
    motion_type: c_int,
    start_lnum: c_int,
    end_lnum: c_int,
    got_int: c_int,
) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    c_int::from(should_propagate_block_change(
        mt,
        start_lnum,
        end_lnum,
        got_int != 0,
    ))
}

/// FFI wrapper for calc_change_start_col.
#[no_mangle]
pub extern "C" fn rs_calc_change_start_col(motion_type: c_int, start_col: c_int) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    calc_change_start_col(mt, start_col)
}

/// FFI wrapper for calc_append_target_col.
#[no_mangle]
pub extern "C" fn rs_calc_append_target_col(textcol: c_int, textlen: c_int) -> c_int {
    calc_append_target_col(textcol, textlen)
}

/// FFI wrapper for should_advance_cursor_append.
#[no_mangle]
pub extern "C" fn rs_should_advance_cursor_append(
    cursor_col: c_int,
    textcol: c_int,
    textlen: c_int,
) -> c_int {
    c_int::from(should_advance_cursor_append(cursor_col, textcol, textlen))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_pre_textlen() {
        // OP_INSERT: line_len - textcol
        assert_eq!(calc_pre_textlen(100, 30, 20, OpType::Insert), 70);

        // OP_APPEND: line_len - textcol - textlen
        assert_eq!(calc_pre_textlen(100, 30, 20, OpType::Append), 50);

        // Edge case: textcol at end
        assert_eq!(calc_pre_textlen(100, 100, 0, OpType::Insert), 0);

        // Negative clamped to 0
        assert_eq!(calc_pre_textlen(50, 60, 0, OpType::Insert), 0);
    }

    #[test]
    fn test_calc_inserted_len() {
        // Normal case: new_len > pre_textlen
        assert_eq!(calc_inserted_len(100, 70), 30);

        // No insertion
        assert_eq!(calc_inserted_len(70, 70), 0);

        // Deletion (shouldn't happen but handle gracefully)
        assert_eq!(calc_inserted_len(50, 70), 0);
    }

    #[test]
    fn test_adjust_textcol_for_indent() {
        // Indent increased
        assert_eq!(adjust_textcol_for_indent(10, 8, 4), 14);

        // Indent decreased
        assert_eq!(adjust_textcol_for_indent(10, 2, 4), 8);

        // No change
        assert_eq!(adjust_textcol_for_indent(10, 4, 4), 10);
    }

    #[test]
    fn test_calc_block_change_newlen() {
        // Normal case
        assert_eq!(calc_block_change_newlen(50, 0, 10), 61);

        // With coladd
        assert_eq!(calc_block_change_newlen(50, 5, 10), 66);
    }

    #[test]
    fn test_should_advance_cursor_append() {
        // Cursor before target
        assert!(should_advance_cursor_append(5, 10, 5));

        // Cursor at target
        assert!(!should_advance_cursor_append(15, 10, 5));

        // Cursor past target
        assert!(!should_advance_cursor_append(20, 10, 5));
    }

    #[test]
    fn test_calc_append_target_col() {
        assert_eq!(calc_append_target_col(10, 5), 15);
        assert_eq!(calc_append_target_col(0, 10), 10);
    }

    #[test]
    fn test_should_inc_cursor_after_change() {
        // Normal case: original > current, line not empty, not virtual
        assert!(should_inc_cursor_after_change(10, 5, false, false));

        // Original <= current
        assert!(!should_inc_cursor_after_change(5, 10, false, false));

        // Line is empty
        assert!(!should_inc_cursor_after_change(10, 5, true, false));

        // Virtual op
        assert!(!should_inc_cursor_after_change(10, 5, false, true));
    }

    #[test]
    fn test_calc_change_start_col() {
        // Linewise: always 0
        assert_eq!(calc_change_start_col(MotionType::LineWise, 10), 0);

        // Charwise: use start_col
        assert_eq!(calc_change_start_col(MotionType::CharWise, 10), 10);

        // Blockwise: use start_col
        assert_eq!(calc_change_start_col(MotionType::BlockWise, 10), 10);
    }

    #[test]
    fn test_should_propagate_block_change() {
        // Block mode, different lines, no interrupt
        assert!(should_propagate_block_change(
            MotionType::BlockWise,
            1,
            5,
            false
        ));

        // Same line
        assert!(!should_propagate_block_change(
            MotionType::BlockWise,
            1,
            1,
            false
        ));

        // Got interrupt
        assert!(!should_propagate_block_change(
            MotionType::BlockWise,
            1,
            5,
            true
        ));

        // Not block mode
        assert!(!should_propagate_block_change(
            MotionType::CharWise,
            1,
            5,
            false
        ));
        assert!(!should_propagate_block_change(
            MotionType::LineWise,
            1,
            5,
            false
        ));
    }

    // =============================================================================
    // Tests for Additional Insert Helpers
    // =============================================================================

    #[test]
    fn test_needs_virtual_spaces() {
        assert!(needs_virtual_spaces(1));
        assert!(needs_virtual_spaces(10));
        assert!(!needs_virtual_spaces(0));
        assert!(!needs_virtual_spaces(-1)); // Negative shouldn't happen but handle gracefully
    }

    #[test]
    fn test_calc_coladvance_target() {
        // OP_APPEND: end_vcol + 1
        assert_eq!(calc_coladvance_target(OpType::Append, 10, 5), 11);
        assert_eq!(calc_coladvance_target(OpType::Append, 0, 5), 1);

        // OP_INSERT: cur_vcol
        assert_eq!(calc_coladvance_target(OpType::Insert, 10, 5), 5);

        // Other op types: cur_vcol
        assert_eq!(calc_coladvance_target(OpType::Nop, 10, 5), 5);
        assert_eq!(calc_coladvance_target(OpType::Delete, 10, 5), 5);
    }

    #[test]
    fn test_needs_short_line_extension() {
        // Short and not max: needs extension
        assert!(needs_short_line_extension(true, false));

        // Short but is max: no extension (block goes to end of line)
        assert!(!needs_short_line_extension(true, true));

        // Not short: no extension
        assert!(!needs_short_line_extension(false, false));
        assert!(!needs_short_line_extension(false, true));
    }

    #[test]
    fn test_adjust_textlen_for_endspaces() {
        assert_eq!(adjust_textlen_for_endspaces(10, 5), 15);
        assert_eq!(adjust_textlen_for_endspaces(0, 5), 5);
        assert_eq!(adjust_textlen_for_endspaces(10, 0), 10);
    }

    #[test]
    fn test_should_inc_cursor_for_append() {
        // Non-empty line, different vcols: yes
        assert!(should_inc_cursor_for_append(false, 0, 10));

        // Empty line: no
        assert!(!should_inc_cursor_for_append(true, 0, 10));

        // Same vcols: no
        assert!(!should_inc_cursor_for_append(false, 10, 10));
    }

    #[test]
    fn test_should_abort_block_insert() {
        // Cursor moved to different line
        assert!(should_abort_block_insert(5, 1, false));

        // Got interrupt
        assert!(should_abort_block_insert(1, 1, true));

        // Both
        assert!(should_abort_block_insert(5, 1, true));

        // Normal: same line, no interrupt
        assert!(!should_abort_block_insert(1, 1, false));
    }

    #[test]
    fn test_did_indent_increase() {
        // Indent increased: op_start_col > ind_pre_col && ind_post_col > ind_pre_col
        assert!(did_indent_increase(10, 5, 8));

        // No indent increase: op_start_col <= ind_pre_col
        assert!(!did_indent_increase(5, 5, 8));
        assert!(!did_indent_increase(3, 5, 8));

        // No indent increase: ind_post_col <= ind_pre_col
        assert!(!did_indent_increase(10, 5, 5));
        assert!(!did_indent_increase(10, 5, 3));
    }

    #[test]
    fn test_adjust_block_for_indent() {
        // Indent increased by 4 columns and 8 vcols
        let (textcol, start_vcol) = adjust_block_for_indent(10, 20, 4, 8);
        assert_eq!(textcol, 14); // 10 + 4
        assert_eq!(start_vcol, 28); // 20 + 8

        // Indent decreased
        let (textcol, start_vcol) = adjust_block_for_indent(10, 20, -2, -4);
        assert_eq!(textcol, 8); // 10 - 2
        assert_eq!(start_vcol, 16); // 20 - 4

        // No change
        let (textcol, start_vcol) = adjust_block_for_indent(10, 20, 0, 0);
        assert_eq!(textcol, 10);
        assert_eq!(start_vcol, 20);
    }

    #[test]
    fn test_calc_ins_len() {
        // Normal case
        assert_eq!(calc_ins_len(100, 70, 5), 25);

        // No offset
        assert_eq!(calc_ins_len(100, 70, 0), 30);

        // Can be negative if pre_textlen + offset > line_len
        assert_eq!(calc_ins_len(50, 70, 0), -20);
    }

    #[test]
    fn test_should_do_block_insert() {
        // Valid: pre_textlen >= 0 and ins_len > 0
        assert!(should_do_block_insert(0, 1));
        assert!(should_do_block_insert(10, 5));

        // Invalid: pre_textlen < 0
        assert!(!should_do_block_insert(-1, 5));

        // Invalid: ins_len <= 0
        assert!(!should_do_block_insert(10, 0));
        assert!(!should_do_block_insert(10, -5));
    }

    #[test]
    fn test_calc_insert_text_offset() {
        // OP_APPEND: textcol + textlen
        assert_eq!(calc_insert_text_offset(10, 5, OpType::Append), 15);

        // OP_INSERT: textcol
        assert_eq!(calc_insert_text_offset(10, 5, OpType::Insert), 10);

        // Other op types: textcol
        assert_eq!(calc_insert_text_offset(10, 5, OpType::Nop), 10);
    }
}
