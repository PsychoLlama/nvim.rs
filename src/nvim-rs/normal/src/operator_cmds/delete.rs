//! Delete operator commands.
//!
//! This module provides helper functions for delete operations:
//! - nv_operator (d/x)
//! - delete_range
//! - del_lines
//! - truncate_line

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Operator Type Constants
// =============================================================================

/// No operation.
pub const OP_NOP: c_int = 0;
/// Delete operator.
pub const OP_DELETE: c_int = 1;
/// Yank operator.
pub const OP_YANK: c_int = 2;
/// Change operator.
pub const OP_CHANGE: c_int = 3;
/// Shift left operator.
pub const OP_LSHIFT: c_int = 4;
/// Shift right operator.
pub const OP_RSHIFT: c_int = 5;
/// Filter operator.
pub const OP_FILTER: c_int = 6;
/// Tilde operator.
pub const OP_TILDE: c_int = 7;
/// Indent operator.
pub const OP_INDENT: c_int = 8;
/// Format operator.
pub const OP_FORMAT: c_int = 9;
/// Colon operator.
pub const OP_COLON: c_int = 10;
/// Uppercase operator.
pub const OP_UPPER: c_int = 11;
/// Lowercase operator.
pub const OP_LOWER: c_int = 12;
/// Join operator.
pub const OP_JOIN: c_int = 13;
/// Join without spaces operator.
pub const OP_JOIN_NS: c_int = 14;
/// Rot13 operator.
pub const OP_ROT13: c_int = 15;
/// Replace operator.
pub const OP_REPLACE: c_int = 16;
/// Fold create operator.
pub const OP_FOLD: c_int = 17;
/// Fold open operator.
pub const OP_FOLDOPEN: c_int = 18;
/// Fold open recursive operator.
pub const OP_FOLDOPENREC: c_int = 19;
/// Fold close operator.
pub const OP_FOLDCLOSE: c_int = 20;
/// Fold close recursive operator.
pub const OP_FOLDCLOSEREC: c_int = 21;
/// Fold delete operator.
pub const OP_FOLDDEL: c_int = 22;
/// Fold delete recursive operator.
pub const OP_FOLDDELREC: c_int = 23;

// =============================================================================
// Delete Operation Helpers (Pure Rust)
// =============================================================================

/// Check if operator is a delete type.
#[allow(dead_code)]
fn is_delete_op(op_type: c_int) -> bool {
    op_type == OP_DELETE
}

/// Check if operator is a change type.
#[allow(dead_code)]
fn is_change_op(op_type: c_int) -> bool {
    op_type == OP_CHANGE
}

/// Check if operator is a yank type.
#[allow(dead_code)]
fn is_yank_op(op_type: c_int) -> bool {
    op_type == OP_YANK
}

/// Check if operator modifies text (delete, change, or any shift/indent).
fn op_modifies_text(op_type: c_int) -> bool {
    matches!(
        op_type,
        OP_DELETE
            | OP_CHANGE
            | OP_LSHIFT
            | OP_RSHIFT
            | OP_FILTER
            | OP_TILDE
            | OP_INDENT
            | OP_FORMAT
            | OP_UPPER
            | OP_LOWER
            | OP_JOIN
            | OP_JOIN_NS
            | OP_ROT13
            | OP_REPLACE
    )
}

/// Check if operator is a fold operation.
#[allow(dead_code)]
fn op_is_fold(op_type: c_int) -> bool {
    matches!(
        op_type,
        OP_FOLD
            | OP_FOLDOPEN
            | OP_FOLDOPENREC
            | OP_FOLDCLOSE
            | OP_FOLDCLOSEREC
            | OP_FOLDDEL
            | OP_FOLDDELREC
    )
}

/// Get operator character for 'd' (delete).
fn delete_char() -> c_int {
    c_int::from(b'd')
}

/// Get operator character for 'x' (delete char).
fn delete_char_char() -> c_int {
    c_int::from(b'x')
}

// =============================================================================
// FFI Exports
// =============================================================================
//
// Note: OP_* constants are already exported from operator.rs and pending.rs.
// This module provides additional helper functions.

/// FFI: Check if operator modifies text.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opcmd_modifies_text(op_type: c_int) -> c_int {
    c_int::from(op_modifies_text(op_type))
}

/// FFI: Get delete character 'd'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opcmd_delete_char() -> c_int {
    delete_char()
}

/// FFI: Get delete char character 'x'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opcmd_delete_char_char() -> c_int {
    delete_char_char()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_constants() {
        assert_eq!(OP_NOP, 0);
        assert_eq!(OP_DELETE, 1);
        assert_eq!(OP_YANK, 2);
        assert_eq!(OP_CHANGE, 3);
    }

    #[test]
    fn test_is_delete_op() {
        assert!(is_delete_op(OP_DELETE));
        assert!(!is_delete_op(OP_YANK));
        assert!(!is_delete_op(OP_CHANGE));
    }

    #[test]
    fn test_op_modifies_text() {
        assert!(op_modifies_text(OP_DELETE));
        assert!(op_modifies_text(OP_CHANGE));
        assert!(!op_modifies_text(OP_YANK));
        assert!(!op_modifies_text(OP_NOP));
        assert!(op_modifies_text(OP_LSHIFT));
        assert!(op_modifies_text(OP_REPLACE));
    }

    #[test]
    fn test_op_is_fold() {
        assert!(op_is_fold(OP_FOLD));
        assert!(op_is_fold(OP_FOLDOPEN));
        assert!(op_is_fold(OP_FOLDCLOSE));
        assert!(!op_is_fold(OP_DELETE));
        assert!(!op_is_fold(OP_NOP));
    }

    #[test]
    fn test_operator_chars() {
        assert_eq!(delete_char(), c_int::from(b'd'));
        assert_eq!(delete_char_char(), c_int::from(b'x'));
    }
}
