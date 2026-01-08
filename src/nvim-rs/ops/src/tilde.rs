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
    fn mb_islower(c: c_int) -> c_int;
    fn mb_isupper(c: c_int) -> c_int;
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
    let is_lower = unsafe { mb_islower(c) != 0 };
    let is_upper = unsafe { mb_isupper(c) != 0 };

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
}
