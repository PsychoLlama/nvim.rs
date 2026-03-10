//! Case swapping operations (g~, gU, gu, g?)
//!
//! This module implements the character case transformation logic used by
//! the tilde operator family: `g~` (toggle), `gU` (uppercase), `gu` (lowercase),
//! and `g?` (ROT13).
//!
//! The main entry point is `op_tilde()` in C which calls into these Rust functions
//! for the core character transformation logic.

use std::ffi::c_int;

use crate::types::{MotionType, OpType};

// =============================================================================
// FFI Functions from C
// =============================================================================

extern "C" {
    fn mb_islower(c: c_int) -> bool;
    fn mb_isupper(c: c_int) -> bool;
    fn mb_toupper(c: c_int) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;
}

// =============================================================================
// Core Character Operations
// =============================================================================

/// ROT13 transformation for ASCII letters.
///
/// Rotates the character by 13 positions in the alphabet.
/// `base` should be `'a'` for lowercase or `'A'` for uppercase.
#[inline]
#[must_use]
pub const fn rot13(c: c_int, base: u8) -> c_int {
    let base = base as c_int;
    ((c - base + 13) % 26) + base
}

/// Determine the new character after applying a case operation.
///
/// Returns the transformed character, or the original if no change is needed.
///
/// - `OP_UPPER`: convert to uppercase
/// - `OP_LOWER`: convert to lowercase
/// - `OP_ROT13`: apply ROT13 encoding (ASCII only)
/// - `OP_TILDE`: toggle case
#[must_use]
pub fn swap_char(op_type: OpType, c: c_int) -> c_int {
    // ROT13 only works on ASCII
    if c >= 0x80 && op_type == OpType::Rot13 {
        return c;
    }

    // SAFETY: These are safe FFI calls to character classification functions
    let is_lower = unsafe { mb_islower(c) };
    let is_upper = unsafe { mb_isupper(c) };

    if is_lower {
        match op_type {
            OpType::Rot13 => rot13(c, b'a'),
            OpType::Lower => c, // Already lowercase
            _ => unsafe { mb_toupper(c) },
        }
    } else if is_upper {
        match op_type {
            OpType::Rot13 => rot13(c, b'A'),
            OpType::Upper => c, // Already uppercase
            _ => unsafe { mb_tolower(c) },
        }
    } else {
        c // Not a letter, no change
    }
}

/// Check if a character would be changed by the swap operation.
#[inline]
#[must_use]
pub fn would_change(op_type: OpType, c: c_int) -> bool {
    swap_char(op_type, c) != c
}

/// FFI wrapper for swap_char.
///
/// Returns the transformed character based on the operator type.
#[no_mangle]
pub extern "C" fn rs_swap_char(op_type: c_int, c: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Tilde);
    swap_char(op, c)
}

/// FFI wrapper to check if a character would change.
#[no_mangle]
pub extern "C" fn rs_would_change(op_type: c_int, c: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Tilde);
    c_int::from(would_change(op, c))
}

// =============================================================================
// Operator Logic Helpers
// =============================================================================

/// Check if a character requires multi-byte handling for case swap.
///
/// Characters >= 0x80 or the resulting character >= 0x80 require
/// special handling (delete + insert instead of simple byte replacement).
#[must_use]
#[inline]
pub const fn needs_multibyte_handling(original: c_int, transformed: c_int) -> bool {
    original >= 0x80 || transformed >= 0x80
}

/// Calculate the end column for linewise tilde operation.
///
/// For linewise operations, the end column is set to the line length - 1.
///
/// # Arguments
/// * `line_len` - Length of the line
///
/// # Returns
/// The adjusted end column
#[must_use]
#[inline]
pub const fn calc_linewise_end_col(line_len: c_int) -> c_int {
    if line_len > 0 {
        line_len - 1
    } else {
        0
    }
}

/// Calculate the length of text to swap on a single-line operation.
///
/// # Arguments
/// * `start_col` - Starting column
/// * `end_col` - Ending column
///
/// # Returns
/// Number of bytes to process
#[must_use]
#[inline]
pub const fn calc_tilde_length(start_col: c_int, end_col: c_int) -> c_int {
    let len = end_col - start_col + 1;
    if len < 0 {
        0
    } else {
        len
    }
}

/// Adjust end position for non-inclusive motion.
///
/// When the motion is not inclusive, the end position should be decremented.
/// This returns whether the adjustment should be made.
///
/// # Arguments
/// * `motion_type` - The motion type
/// * `inclusive` - Whether the motion is inclusive
///
/// # Returns
/// true if end position should be decremented
#[must_use]
#[inline]
pub const fn should_decrement_end(motion_type: MotionType, inclusive: bool) -> bool {
    !matches!(motion_type, MotionType::LineWise) && !inclusive
}

/// Check if the operator is a case-changing operator.
///
/// Returns true for OP_TILDE, OP_UPPER, OP_LOWER, and OP_ROT13.
#[must_use]
#[inline]
pub const fn is_case_operator(op_type: OpType) -> bool {
    matches!(
        op_type,
        OpType::Tilde | OpType::Upper | OpType::Lower | OpType::Rot13
    )
}

/// Get the swap length for block mode on a single line.
///
/// In block mode, we swap `textlen` characters starting at `textcol`.
///
/// # Arguments
/// * `textlen` - Length of text in the block on this line
///
/// # Returns
/// Number of bytes to process
#[must_use]
#[inline]
pub const fn calc_block_tilde_length(textlen: c_int) -> c_int {
    if textlen < 0 {
        0
    } else {
        textlen
    }
}

// =============================================================================
// FFI Wrappers for Logic Helpers
// =============================================================================

/// FFI wrapper for needs_multibyte_handling.
#[no_mangle]
pub extern "C" fn rs_needs_multibyte_handling(original: c_int, transformed: c_int) -> c_int {
    c_int::from(needs_multibyte_handling(original, transformed))
}

/// FFI wrapper for calc_linewise_end_col.
#[no_mangle]
pub extern "C" fn rs_calc_linewise_end_col(line_len: c_int) -> c_int {
    calc_linewise_end_col(line_len)
}

/// FFI wrapper for calc_tilde_length.
#[no_mangle]
pub extern "C" fn rs_calc_tilde_length(start_col: c_int, end_col: c_int) -> c_int {
    calc_tilde_length(start_col, end_col)
}

/// FFI wrapper for should_decrement_end.
#[no_mangle]
pub extern "C" fn rs_should_decrement_end(motion_type: c_int, inclusive: c_int) -> c_int {
    let mt = MotionType::from_raw(motion_type);
    c_int::from(should_decrement_end(mt, inclusive != 0))
}

/// FFI wrapper for is_case_operator.
#[no_mangle]
pub extern "C" fn rs_is_case_operator(op_type: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(is_case_operator(op))
}

/// FFI wrapper for calc_block_tilde_length.
#[no_mangle]
pub extern "C" fn rs_calc_block_tilde_length(textlen: c_int) -> c_int {
    calc_block_tilde_length(textlen)
}

// =============================================================================
// Phase O4 Case Helpers
// =============================================================================

/// Check if operator is OP_UPPER (gU).
#[must_use]
#[inline]
pub const fn is_upper_operator(op_type: OpType) -> bool {
    matches!(op_type, OpType::Upper)
}

/// Check if operator is OP_LOWER (gu).
#[must_use]
#[inline]
pub const fn is_lower_operator(op_type: OpType) -> bool {
    matches!(op_type, OpType::Lower)
}

/// Check if operator is OP_ROT13 (g?).
#[must_use]
#[inline]
pub const fn is_rot13_operator(op_type: OpType) -> bool {
    matches!(op_type, OpType::Rot13)
}

/// Check if operator is OP_TILDE (~).
#[must_use]
#[inline]
pub const fn is_tilde_operator(op_type: OpType) -> bool {
    matches!(op_type, OpType::Tilde)
}

/// Calculate adjusted end column for case operation.
///
/// For non-inclusive motion, decrement end column.
#[must_use]
#[inline]
pub const fn calc_case_end_col(end_col: c_int, inclusive: bool, is_linewise: bool) -> c_int {
    if is_linewise {
        end_col
    } else if !inclusive && end_col > 0 {
        end_col - 1
    } else {
        end_col
    }
}

/// Calculate number of characters for case operation.
#[must_use]
#[inline]
pub const fn calc_case_char_count(start_col: c_int, end_col: c_int) -> c_int {
    if end_col >= start_col {
        end_col - start_col + 1
    } else {
        0
    }
}

/// Check if message should be shown after case operation.
#[must_use]
#[inline]
pub const fn should_show_case_message(line_count: c_int, report_threshold: c_int) -> bool {
    line_count > 0 && report_threshold >= 0 && line_count > report_threshold
}

/// Get message type based on operator.
#[must_use]
#[inline]
pub const fn get_case_message_type(op_type: OpType) -> c_int {
    match op_type {
        OpType::Upper => 1, // "changed to uppercase"
        OpType::Lower => 2, // "changed to lowercase"
        OpType::Rot13 => 3, // "ROT13 encoded"
        OpType::Tilde => 4, // "case changed"
        _ => 0,
    }
}

/// Calculate line count for multiline case operation.
#[must_use]
#[inline]
pub const fn calc_case_line_count(start_lnum: c_int, end_lnum: c_int) -> c_int {
    if end_lnum >= start_lnum {
        end_lnum - start_lnum + 1
    } else {
        0
    }
}

/// Check if case operation is on a single line.
#[must_use]
#[inline]
pub const fn is_single_line_case(start_lnum: c_int, end_lnum: c_int) -> bool {
    start_lnum == end_lnum
}

// =============================================================================
// Phase O4 FFI Wrappers
// =============================================================================

/// FFI: Check if operator is OP_UPPER.
#[no_mangle]
pub extern "C" fn rs_is_upper_operator(op_type: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(is_upper_operator(op))
}

/// FFI: Check if operator is OP_LOWER.
#[no_mangle]
pub extern "C" fn rs_is_lower_operator(op_type: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(is_lower_operator(op))
}

/// FFI: Check if operator is OP_ROT13.
#[no_mangle]
pub extern "C" fn rs_is_rot13_operator(op_type: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(is_rot13_operator(op))
}

/// FFI: Check if operator is OP_TILDE.
#[no_mangle]
pub extern "C" fn rs_is_tilde_operator(op_type: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    c_int::from(is_tilde_operator(op))
}

/// FFI: Calculate adjusted end column for case operation.
#[no_mangle]
pub extern "C" fn rs_calc_case_end_col(
    end_col: c_int,
    inclusive: c_int,
    is_linewise: c_int,
) -> c_int {
    calc_case_end_col(end_col, inclusive != 0, is_linewise != 0)
}

/// FFI: Calculate character count for case operation.
#[no_mangle]
pub extern "C" fn rs_calc_case_char_count(start_col: c_int, end_col: c_int) -> c_int {
    calc_case_char_count(start_col, end_col)
}

/// FFI: Check if message should be shown after case operation.
#[no_mangle]
pub extern "C" fn rs_should_show_case_message(line_count: c_int, report_threshold: c_int) -> c_int {
    c_int::from(should_show_case_message(line_count, report_threshold))
}

/// FFI: Get message type for case operation.
#[no_mangle]
pub extern "C" fn rs_get_case_message_type(op_type: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Nop);
    get_case_message_type(op)
}

/// FFI: Calculate line count for case operation.
#[no_mangle]
pub extern "C" fn rs_calc_case_line_count(start_lnum: c_int, end_lnum: c_int) -> c_int {
    calc_case_line_count(start_lnum, end_lnum)
}

/// FFI: Check if single line case operation.
#[no_mangle]
pub extern "C" fn rs_is_single_line_case(start_lnum: c_int, end_lnum: c_int) -> c_int {
    c_int::from(is_single_line_case(start_lnum, end_lnum))
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    // =========================================================================
    // ROT13 Tests
    // =========================================================================

    #[test]
    fn test_rot13_lowercase() {
        // 'a' -> 'n', 'n' -> 'a'
        assert_eq!(rot13(b'a' as c_int, b'a'), b'n' as c_int);
        assert_eq!(rot13(b'n' as c_int, b'a'), b'a' as c_int);
        assert_eq!(rot13(b'z' as c_int, b'a'), b'm' as c_int);
        assert_eq!(rot13(b'm' as c_int, b'a'), b'z' as c_int);
    }

    #[test]
    fn test_rot13_uppercase() {
        // 'A' -> 'N', 'N' -> 'A'
        assert_eq!(rot13(b'A' as c_int, b'A'), b'N' as c_int);
        assert_eq!(rot13(b'N' as c_int, b'A'), b'A' as c_int);
        assert_eq!(rot13(b'Z' as c_int, b'A'), b'M' as c_int);
        assert_eq!(rot13(b'M' as c_int, b'A'), b'Z' as c_int);
    }

    #[test]
    fn test_rot13_double_application() {
        // ROT13 applied twice should return to original
        for c in b'a'..=b'z' {
            let c = c as c_int;
            assert_eq!(rot13(rot13(c, b'a'), b'a'), c);
        }
        for c in b'A'..=b'Z' {
            let c = c as c_int;
            assert_eq!(rot13(rot13(c, b'A'), b'A'), c);
        }
    }

    // =========================================================================
    // needs_multibyte_handling Tests
    // =========================================================================

    #[test]
    fn test_needs_multibyte_handling() {
        // ASCII only - no multibyte handling needed
        assert!(!needs_multibyte_handling(b'a' as c_int, b'A' as c_int));
        assert!(!needs_multibyte_handling(b'Z' as c_int, b'z' as c_int));

        // Original is multibyte
        assert!(needs_multibyte_handling(0x80, b'A' as c_int));
        assert!(needs_multibyte_handling(0x100, b'a' as c_int));

        // Transformed is multibyte
        assert!(needs_multibyte_handling(b'a' as c_int, 0x80));
        assert!(needs_multibyte_handling(b'a' as c_int, 0x100));

        // Both multibyte
        assert!(needs_multibyte_handling(0x100, 0x200));
    }

    // =========================================================================
    // calc_linewise_end_col Tests
    // =========================================================================

    #[test]
    fn test_calc_linewise_end_col() {
        // Normal cases
        assert_eq!(calc_linewise_end_col(10), 9);
        assert_eq!(calc_linewise_end_col(1), 0);

        // Edge case: empty line
        assert_eq!(calc_linewise_end_col(0), 0);

        // Edge case: negative (shouldn't happen but handle gracefully)
        // Note: this would produce negative result but clamped to 0
    }

    // =========================================================================
    // calc_tilde_length Tests
    // =========================================================================

    #[test]
    fn test_calc_tilde_length() {
        // Normal cases
        assert_eq!(calc_tilde_length(0, 9), 10);
        assert_eq!(calc_tilde_length(5, 10), 6);

        // Single character
        assert_eq!(calc_tilde_length(5, 5), 1);

        // Edge case: end before start (clamped to 0)
        assert_eq!(calc_tilde_length(10, 5), 0);
    }

    // =========================================================================
    // should_decrement_end Tests
    // =========================================================================

    #[test]
    fn test_should_decrement_end() {
        // Linewise - never decrement (end is calculated differently)
        assert!(!should_decrement_end(MotionType::LineWise, false));
        assert!(!should_decrement_end(MotionType::LineWise, true));

        // Charwise - decrement only if not inclusive
        assert!(should_decrement_end(MotionType::CharWise, false));
        assert!(!should_decrement_end(MotionType::CharWise, true));

        // Blockwise - decrement only if not inclusive
        assert!(should_decrement_end(MotionType::BlockWise, false));
        assert!(!should_decrement_end(MotionType::BlockWise, true));
    }

    // =========================================================================
    // is_case_operator Tests
    // =========================================================================

    #[test]
    fn test_is_case_operator() {
        // Case operators
        assert!(is_case_operator(OpType::Tilde));
        assert!(is_case_operator(OpType::Upper));
        assert!(is_case_operator(OpType::Lower));
        assert!(is_case_operator(OpType::Rot13));

        // Non-case operators
        assert!(!is_case_operator(OpType::Nop));
        assert!(!is_case_operator(OpType::Delete));
        assert!(!is_case_operator(OpType::Yank));
        assert!(!is_case_operator(OpType::Change));
        assert!(!is_case_operator(OpType::LShift));
        assert!(!is_case_operator(OpType::RShift));
    }

    // =========================================================================
    // calc_block_tilde_length Tests
    // =========================================================================

    #[test]
    fn test_calc_block_tilde_length() {
        // Normal cases
        assert_eq!(calc_block_tilde_length(10), 10);
        assert_eq!(calc_block_tilde_length(0), 0);

        // Edge case: negative (clamped to 0)
        assert_eq!(calc_block_tilde_length(-5), 0);
    }

    // =========================================================================
    // FFI Wrapper Tests
    // =========================================================================

    #[test]
    fn test_ffi_wrappers() {
        // rs_needs_multibyte_handling
        assert_eq!(rs_needs_multibyte_handling(b'a' as c_int, b'A' as c_int), 0);
        assert_eq!(rs_needs_multibyte_handling(0x100, b'a' as c_int), 1);

        // rs_calc_linewise_end_col
        assert_eq!(rs_calc_linewise_end_col(10), 9);

        // rs_calc_tilde_length
        assert_eq!(rs_calc_tilde_length(0, 9), 10);

        // rs_should_decrement_end - charwise, not inclusive
        assert_eq!(rs_should_decrement_end(0, 0), 1); // CharWise=0, not inclusive

        // rs_is_case_operator
        assert_eq!(rs_is_case_operator(7), 1); // OP_TILDE = 7
        assert_eq!(rs_is_case_operator(1), 0); // OP_DELETE = 1

        // rs_calc_block_tilde_length
        assert_eq!(rs_calc_block_tilde_length(10), 10);
    }

    // =========================================================================
    // Phase O4 Case Helper Tests
    // =========================================================================

    #[test]
    fn test_case_operator_type_checks() {
        assert!(is_upper_operator(OpType::Upper));
        assert!(!is_upper_operator(OpType::Lower));
        assert!(!is_upper_operator(OpType::Tilde));

        assert!(is_lower_operator(OpType::Lower));
        assert!(!is_lower_operator(OpType::Upper));
        assert!(!is_lower_operator(OpType::Tilde));

        assert!(is_rot13_operator(OpType::Rot13));
        assert!(!is_rot13_operator(OpType::Upper));
        assert!(!is_rot13_operator(OpType::Lower));

        assert!(is_tilde_operator(OpType::Tilde));
        assert!(!is_tilde_operator(OpType::Upper));
        assert!(!is_tilde_operator(OpType::Lower));
    }

    #[test]
    fn test_calc_case_end_col() {
        // Linewise - no adjustment
        assert_eq!(calc_case_end_col(10, false, true), 10);
        assert_eq!(calc_case_end_col(10, true, true), 10);

        // Non-inclusive charwise - decrement
        assert_eq!(calc_case_end_col(10, false, false), 9);

        // Inclusive charwise - no decrement
        assert_eq!(calc_case_end_col(10, true, false), 10);

        // Edge case: end_col = 0 and not inclusive
        assert_eq!(calc_case_end_col(0, false, false), 0);
    }

    #[test]
    fn test_calc_case_char_count() {
        assert_eq!(calc_case_char_count(0, 9), 10);
        assert_eq!(calc_case_char_count(5, 5), 1);
        assert_eq!(calc_case_char_count(10, 5), 0);
    }

    #[test]
    fn test_should_show_case_message() {
        assert!(should_show_case_message(10, 5));
        assert!(!should_show_case_message(3, 5));
        assert!(!should_show_case_message(5, -1));
        assert!(should_show_case_message(1, 0));
    }

    #[test]
    fn test_get_case_message_type() {
        assert_eq!(get_case_message_type(OpType::Upper), 1);
        assert_eq!(get_case_message_type(OpType::Lower), 2);
        assert_eq!(get_case_message_type(OpType::Rot13), 3);
        assert_eq!(get_case_message_type(OpType::Tilde), 4);
        assert_eq!(get_case_message_type(OpType::Nop), 0);
    }

    #[test]
    fn test_calc_case_line_count() {
        assert_eq!(calc_case_line_count(1, 10), 10);
        assert_eq!(calc_case_line_count(5, 5), 1);
        assert_eq!(calc_case_line_count(10, 5), 0);
    }

    #[test]
    fn test_is_single_line_case() {
        assert!(is_single_line_case(5, 5));
        assert!(!is_single_line_case(1, 10));
    }

    #[test]
    fn test_phase_o4_ffi_wrappers() {
        // Case operator checks
        assert_eq!(rs_is_upper_operator(11), 1); // OP_UPPER = 11
        assert_eq!(rs_is_lower_operator(12), 1); // OP_LOWER = 12
        assert_eq!(rs_is_rot13_operator(15), 1); // OP_ROT13 = 15
        assert_eq!(rs_is_tilde_operator(7), 1); // OP_TILDE = 7

        // calc_case_end_col
        assert_eq!(rs_calc_case_end_col(10, 0, 0), 9);
        assert_eq!(rs_calc_case_end_col(10, 1, 0), 10);
        assert_eq!(rs_calc_case_end_col(10, 0, 1), 10);

        // calc_case_char_count
        assert_eq!(rs_calc_case_char_count(0, 9), 10);

        // should_show_case_message
        assert_eq!(rs_should_show_case_message(10, 5), 1);
        assert_eq!(rs_should_show_case_message(3, 5), 0);

        // get_case_message_type
        assert_eq!(rs_get_case_message_type(11), 1); // Upper
        assert_eq!(rs_get_case_message_type(12), 2); // Lower

        // calc_case_line_count
        assert_eq!(rs_calc_case_line_count(1, 10), 10);

        // is_single_line_case
        assert_eq!(rs_is_single_line_case(5, 5), 1);
        assert_eq!(rs_is_single_line_case(1, 10), 0);
    }
}
