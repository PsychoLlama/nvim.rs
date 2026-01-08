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
    if ins_len < 0 { 0 } else { ins_len }
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
        assert!(should_propagate_block_change(MotionType::BlockWise, 1, 5, false));

        // Same line
        assert!(!should_propagate_block_change(MotionType::BlockWise, 1, 1, false));

        // Got interrupt
        assert!(!should_propagate_block_change(MotionType::BlockWise, 1, 5, true));

        // Not block mode
        assert!(!should_propagate_block_change(MotionType::CharWise, 1, 5, false));
        assert!(!should_propagate_block_change(MotionType::LineWise, 1, 5, false));
    }
}
