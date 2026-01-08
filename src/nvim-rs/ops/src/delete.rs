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
    if n < 0 { 0 } else { n }
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
    if n < 0 { 0 } else { n }
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
    let use_numbered = matches!(motion_type, MotionType::LineWise)
        || line_count > 1
        || use_reg_one;

    // Use small delete register (-) if:
    // - no named register specified (regname == 0)
    // - not linewise
    // - single line
    let use_small_delete = regname == 0
        && !matches!(motion_type, MotionType::LineWise)
        && line_count == 1;

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
        let (numbered, small) = determine_delete_register(
            0,
            MotionType::LineWise,
            1,
            false,
        );
        assert!(numbered);
        assert!(!small);

        // Multi-line: use numbered
        let (numbered, small) = determine_delete_register(
            0,
            MotionType::CharWise,
            2,
            false,
        );
        assert!(numbered);
        assert!(!small);

        // use_reg_one: use numbered (but also small for single line charwise)
        let (numbered, small) = determine_delete_register(
            0,
            MotionType::CharWise,
            1,
            true,
        );
        assert!(numbered);
        assert!(small); // Both can be true - they're independent checks

        // Single line charwise with no register: use small delete
        let (numbered, small) = determine_delete_register(
            0,
            MotionType::CharWise,
            1,
            false,
        );
        assert!(!numbered);
        assert!(small);

        // Named register: neither numbered nor small
        let (numbered, small) = determine_delete_register(
            b'a' as c_int,
            MotionType::CharWise,
            1,
            false,
        );
        assert!(!numbered);
        assert!(!small);
    }
}
