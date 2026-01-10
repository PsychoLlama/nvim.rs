//! Helper functions for extmark operations
//!
//! This module provides FFI functions for common extmark operations used in C code.

use std::ffi::c_int;

use crate::{
    MT_FLAG_DECOR_EXT, MT_FLAG_DECOR_MASK, MT_FLAG_END, MT_FLAG_INVALID, MT_FLAG_INVALIDATE,
    MT_FLAG_NO_UNDO, MT_FLAG_PAIRED, MT_FLAG_REAL, MT_FLAG_RIGHT_GRAVITY,
};

// =============================================================================
// Mark Flag Checks (FFI wrappers)
// =============================================================================

/// FFI: Check if mark flags indicate paired mark.
#[no_mangle]
pub extern "C" fn rs_flags_paired(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_PAIRED != 0)
}

/// FFI: Check if mark flags indicate end of a pair.
#[no_mangle]
pub extern "C" fn rs_flags_end(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_END != 0)
}

/// FFI: Check if mark flags indicate right gravity.
#[no_mangle]
pub extern "C" fn rs_flags_right(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_RIGHT_GRAVITY != 0)
}

/// FFI: Check if mark flags indicate invalid.
#[no_mangle]
pub extern "C" fn rs_flags_invalid(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_INVALID != 0)
}

/// FFI: Check if mark flags indicate should be invalidated.
#[no_mangle]
pub extern "C" fn rs_flags_invalidate(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_INVALIDATE != 0)
}

/// FFI: Check if mark flags indicate no_undo.
#[no_mangle]
pub extern "C" fn rs_flags_no_undo(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_NO_UNDO != 0)
}

/// FFI: Check if mark flags indicate any decoration.
#[no_mangle]
pub extern "C" fn rs_flags_decor_any(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_DECOR_MASK != 0)
}

/// FFI: Check if mark flags indicate external decoration.
#[no_mangle]
pub extern "C" fn rs_flags_decor_ext(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_DECOR_EXT != 0)
}

/// FFI: Check if mark flags indicate "real" (user-created).
#[no_mangle]
pub extern "C" fn rs_flags_real(flags: u16) -> c_int {
    c_int::from(flags & MT_FLAG_REAL != 0)
}

// =============================================================================
// Mark Flag Operations
// =============================================================================

/// FFI: Compute flags for a new mark.
#[no_mangle]
pub extern "C" fn rs_flags_compute(
    right_gravity: c_int,
    no_undo: c_int,
    invalidate: c_int,
    decor_ext: c_int,
) -> u16 {
    crate::mt_flags(
        right_gravity != 0,
        no_undo != 0,
        invalidate != 0,
        decor_ext != 0,
    )
}

/// FFI: Set the invalid flag on a flags value.
#[no_mangle]
pub extern "C" fn rs_flags_set_invalid(flags: u16) -> u16 {
    flags | MT_FLAG_INVALID
}

/// FFI: Clear the invalid flag on a flags value.
#[no_mangle]
pub extern "C" fn rs_flags_clear_invalid(flags: u16) -> u16 {
    flags & !MT_FLAG_INVALID
}

// =============================================================================
// Position Comparisons
// =============================================================================

/// FFI: Compare two positions, returning -1, 0, or 1.
#[no_mangle]
pub extern "C" fn rs_pos_cmp(row1: c_int, col1: c_int, row2: c_int, col2: c_int) -> c_int {
    match row1.cmp(&row2) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Equal => match col1.cmp(&col2) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
        },
    }
}

/// FFI: Check if two positions are equal.
#[no_mangle]
pub extern "C" fn rs_pos_eq(row1: c_int, col1: c_int, row2: c_int, col2: c_int) -> c_int {
    c_int::from(row1 == row2 && col1 == col2)
}

/// FFI: Check if position row is valid (>= 0).
#[no_mangle]
pub extern "C" fn rs_row_valid(row: c_int) -> c_int {
    c_int::from(row >= 0)
}

/// FFI: Check if position row is invalid (< 0).
#[no_mangle]
pub extern "C" fn rs_row_invalid(row: c_int) -> c_int {
    c_int::from(row < 0)
}

/// FFI: Check if position 1 is before position 2.
#[no_mangle]
pub extern "C" fn rs_pos_before(row1: c_int, col1: c_int, row2: c_int, col2: c_int) -> c_int {
    c_int::from(row1 < row2 || (row1 == row2 && col1 < col2))
}

/// FFI: Check if position 1 is before or equal to position 2.
#[no_mangle]
pub extern "C" fn rs_pos_before_or_eq(row1: c_int, col1: c_int, row2: c_int, col2: c_int) -> c_int {
    c_int::from(row1 < row2 || (row1 == row2 && col1 <= col2))
}

/// FFI: Check if position 1 is after position 2.
#[no_mangle]
pub extern "C" fn rs_pos_after(row1: c_int, col1: c_int, row2: c_int, col2: c_int) -> c_int {
    c_int::from(row1 > row2 || (row1 == row2 && col1 > col2))
}

/// FFI: Check if position 1 is after or equal to position 2.
#[no_mangle]
pub extern "C" fn rs_pos_after_or_eq(row1: c_int, col1: c_int, row2: c_int, col2: c_int) -> c_int {
    c_int::from(row1 > row2 || (row1 == row2 && col1 >= col2))
}

// =============================================================================
// Range Checks
// =============================================================================

/// FFI: Check if position is within a range (inclusive start, exclusive end).
#[no_mangle]
pub extern "C" fn rs_pos_between(
    pos_row: c_int,
    pos_col: c_int,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) -> c_int {
    let after_start = pos_row > start_row || (pos_row == start_row && pos_col >= start_col);
    let before_end = pos_row < end_row || (pos_row == end_row && pos_col < end_col);
    c_int::from(after_start && before_end)
}

/// FFI: Check if position is within a range (inclusive both ends).
#[no_mangle]
pub extern "C" fn rs_pos_between_inclusive(
    pos_row: c_int,
    pos_col: c_int,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) -> c_int {
    let after_start = pos_row > start_row || (pos_row == start_row && pos_col >= start_col);
    let before_end = pos_row < end_row || (pos_row == end_row && pos_col <= end_col);
    c_int::from(after_start && before_end)
}

// =============================================================================
// Row/Column Adjustments
// =============================================================================

/// FFI: Compute new row after adjustment.
#[no_mangle]
pub extern "C" fn rs_adjust_row(row: c_int, start_row: c_int, old_rows: c_int, new_rows: c_int) -> c_int {
    if row < start_row {
        row
    } else if row < start_row + old_rows {
        // Within deleted range - clamp to start
        start_row
    } else {
        // After deleted range - shift
        row - old_rows + new_rows
    }
}

/// FFI: Check if row is within a delete/change range.
#[no_mangle]
pub extern "C" fn rs_row_in_change_range(row: c_int, start_row: c_int, old_rows: c_int) -> c_int {
    c_int::from(row >= start_row && row < start_row + old_rows)
}

// =============================================================================
// Mark Key Operations
// =============================================================================

/// FFI: Compute lookup key from namespace and ID.
/// The lookup key encodes ns, id, and end flag.
#[no_mangle]
pub extern "C" fn rs_extmark_lookup_key(ns: u32, id: u32, is_end: c_int) -> u64 {
    (u64::from(ns) << 32) | u64::from(id) | u64::from(is_end != 0)
}

/// FFI: Extract namespace from lookup key.
#[no_mangle]
pub extern "C" fn rs_extmark_lookup_key_ns(key: u64) -> u32 {
    (key >> 32) as u32
}

/// FFI: Extract ID from lookup key.
#[no_mangle]
pub extern "C" fn rs_extmark_lookup_key_id(key: u64) -> u32 {
    (key & 0xFFFF_FFFF) as u32
}

// =============================================================================
// Undo Operation Checks
// =============================================================================

/// Check if undo operation is Noop.
#[no_mangle]
pub extern "C" fn rs_extmark_op_is_noop(op: c_int) -> c_int {
    c_int::from(op == 0)
}

/// Check if undo operation requires undo tracking.
#[no_mangle]
pub extern "C" fn rs_extmark_op_is_undo(op: c_int) -> c_int {
    c_int::from(op == 1)
}

/// Check if undo operation is NoUndo.
#[no_mangle]
pub extern "C" fn rs_extmark_op_is_no_undo(op: c_int) -> c_int {
    c_int::from(op == 2)
}

/// Check if undo operation is UndoNoRedo.
#[no_mangle]
pub extern "C" fn rs_extmark_op_is_undo_no_redo(op: c_int) -> c_int {
    c_int::from(op == 3)
}

// =============================================================================
// Splice Helpers
// =============================================================================

/// FFI: Check if a splice operation affects marks.
#[no_mangle]
pub extern "C" fn rs_splice_affects_marks(old_row: c_int, old_col: c_int, new_row: c_int, new_col: c_int) -> c_int {
    c_int::from(old_row > 0 || old_col > 0 || new_row > 0 || new_col > 0)
}

/// FFI: Check if a splice is row-only (no column change on same row).
#[no_mangle]
pub extern "C" fn rs_splice_is_row_only(old_row: c_int, old_col: c_int, new_row: c_int, new_col: c_int) -> c_int {
    c_int::from((old_row > 0 || new_row > 0) && old_col == 0 && new_col == 0)
}

/// FFI: Check if a splice is column-only (within a single row).
#[no_mangle]
pub extern "C" fn rs_splice_is_col_only(old_row: c_int, new_row: c_int) -> c_int {
    c_int::from(old_row == 0 && new_row == 0)
}

/// FFI: Compute end position after splice.
#[no_mangle]
pub extern "C" fn rs_splice_end_row(start_row: c_int, old_row: c_int) -> c_int {
    start_row + old_row
}

/// FFI: Compute end column after splice.
#[no_mangle]
pub extern "C" fn rs_splice_end_col(start_col: c_int, old_row: c_int, old_col: c_int) -> c_int {
    if old_row != 0 {
        old_col
    } else {
        start_col + old_col
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags_checks() {
        assert_eq!(rs_flags_paired(MT_FLAG_PAIRED), 1);
        assert_eq!(rs_flags_paired(0), 0);

        assert_eq!(rs_flags_end(MT_FLAG_END), 1);
        assert_eq!(rs_flags_end(0), 0);

        assert_eq!(rs_flags_right(MT_FLAG_RIGHT_GRAVITY), 1);
        assert_eq!(rs_flags_right(0), 0);

        assert_eq!(rs_flags_invalid(MT_FLAG_INVALID), 1);
        assert_eq!(rs_flags_invalid(0), 0);

        assert_eq!(rs_flags_no_undo(MT_FLAG_NO_UNDO), 1);
        assert_eq!(rs_flags_no_undo(0), 0);
    }

    #[test]
    fn test_pos_cmp() {
        assert_eq!(rs_pos_cmp(1, 5, 2, 3), -1); // row1 < row2
        assert_eq!(rs_pos_cmp(2, 5, 1, 3), 1); // row1 > row2
        assert_eq!(rs_pos_cmp(2, 3, 2, 5), -1); // same row, col1 < col2
        assert_eq!(rs_pos_cmp(2, 5, 2, 3), 1); // same row, col1 > col2
        assert_eq!(rs_pos_cmp(2, 5, 2, 5), 0); // equal
    }

    #[test]
    fn test_pos_eq() {
        assert_eq!(rs_pos_eq(2, 5, 2, 5), 1);
        assert_eq!(rs_pos_eq(2, 5, 2, 3), 0);
        assert_eq!(rs_pos_eq(1, 5, 2, 5), 0);
    }

    #[test]
    fn test_row_valid_invalid() {
        assert_eq!(rs_row_valid(0), 1);
        assert_eq!(rs_row_valid(10), 1);
        assert_eq!(rs_row_valid(-1), 0);
        assert_eq!(rs_row_invalid(-1), 1);
        assert_eq!(rs_row_invalid(0), 0);
    }

    #[test]
    fn test_pos_before() {
        assert_eq!(rs_pos_before(1, 5, 2, 3), 1);
        assert_eq!(rs_pos_before(2, 3, 2, 5), 1);
        assert_eq!(rs_pos_before(2, 5, 2, 5), 0); // not strictly before
        assert_eq!(rs_pos_before(3, 0, 2, 5), 0);
    }

    #[test]
    fn test_pos_between() {
        // In range
        assert_eq!(rs_pos_between(5, 10, 5, 0, 10, 0), 1);
        assert_eq!(rs_pos_between(7, 5, 5, 0, 10, 0), 1);
        // At start (inclusive)
        assert_eq!(rs_pos_between(5, 0, 5, 0, 10, 0), 1);
        // At end (exclusive)
        assert_eq!(rs_pos_between(10, 0, 5, 0, 10, 0), 0);
        // Before range
        assert_eq!(rs_pos_between(4, 0, 5, 0, 10, 0), 0);
        // After range
        assert_eq!(rs_pos_between(11, 0, 5, 0, 10, 0), 0);
    }

    #[test]
    fn test_adjust_row() {
        // Before change
        assert_eq!(rs_adjust_row(3, 5, 2, 1), 3);
        // Within deleted range
        assert_eq!(rs_adjust_row(5, 5, 2, 1), 5);
        assert_eq!(rs_adjust_row(6, 5, 2, 1), 5);
        // After deleted range
        assert_eq!(rs_adjust_row(7, 5, 2, 1), 6); // 7 - 2 + 1 = 6
        assert_eq!(rs_adjust_row(10, 5, 2, 1), 9); // 10 - 2 + 1 = 9
    }

    #[test]
    fn test_extmark_lookup_key() {
        let key = rs_extmark_lookup_key(100, 42, 0);
        assert_eq!(rs_extmark_lookup_key_ns(key), 100);
        assert_eq!(rs_extmark_lookup_key_id(key), 42);

        let end_key = rs_extmark_lookup_key(100, 42, 1);
        assert_ne!(key, end_key); // End flag makes them different
    }

    #[test]
    fn test_splice_checks() {
        assert_eq!(rs_splice_affects_marks(0, 0, 0, 0), 0);
        assert_eq!(rs_splice_affects_marks(1, 0, 0, 0), 1);
        assert_eq!(rs_splice_affects_marks(0, 5, 0, 0), 1);
        assert_eq!(rs_splice_affects_marks(0, 0, 1, 0), 1);
        assert_eq!(rs_splice_affects_marks(0, 0, 0, 5), 1);

        assert_eq!(rs_splice_is_col_only(0, 0), 1);
        assert_eq!(rs_splice_is_col_only(1, 0), 0);
        assert_eq!(rs_splice_is_col_only(0, 1), 0);
    }

    #[test]
    fn test_splice_end() {
        assert_eq!(rs_splice_end_row(5, 3), 8);
        assert_eq!(rs_splice_end_col(10, 0, 5), 15); // same row: start + old
        assert_eq!(rs_splice_end_col(10, 2, 5), 5); // different row: just old_col
    }
}
