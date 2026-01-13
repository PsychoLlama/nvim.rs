//! Format operator commands.
//!
//! This module provides helper functions for format operations:
//! - nv_shift (>/< operators)
//! - nv_filter (! operator)
//! - nv_join (J command)
//! - nv_format (gq/gw commands)
//! - nv_tilde (~ command)

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use super::delete::{OP_FILTER, OP_FORMAT, OP_INDENT, OP_JOIN, OP_JOIN_NS, OP_LSHIFT, OP_RSHIFT};

// =============================================================================
// Shift Constants
// =============================================================================

/// Shift left amount (default shiftwidth).
pub const SHIFT_LEFT: c_int = -1;
/// Shift right amount (default shiftwidth).
pub const SHIFT_RIGHT: c_int = 1;
/// No shift.
pub const SHIFT_NONE: c_int = 0;

// =============================================================================
// Format Mode Constants
// =============================================================================

/// Format with cursor at end (gq).
pub const FORMAT_GQ: c_int = 0;
/// Format keeping cursor position (gw).
pub const FORMAT_GW: c_int = 1;

// =============================================================================
// Format Operation Helpers (Pure Rust)
// =============================================================================

/// Get shift left character '<'.
fn shift_left_char() -> c_int {
    c_int::from(b'<')
}

/// Get shift right character '>'.
fn shift_right_char() -> c_int {
    c_int::from(b'>')
}

/// Get filter character '!'.
fn filter_char() -> c_int {
    c_int::from(b'!')
}

/// Get join character 'J'.
fn join_char() -> c_int {
    c_int::from(b'J')
}

/// Get tilde character '~'.
fn tilde_char() -> c_int {
    c_int::from(b'~')
}

/// Get format character 'Q' (old vim format).
fn format_char() -> c_int {
    c_int::from(b'Q')
}

/// Check if operator is shift type.
fn is_shift_op(op_type: c_int) -> bool {
    op_type == OP_LSHIFT || op_type == OP_RSHIFT
}

/// Check if operator is format type.
fn is_format_op(op_type: c_int) -> bool {
    op_type == OP_FORMAT || op_type == OP_INDENT
}

/// Check if operator is join type.
fn is_join_op(op_type: c_int) -> bool {
    op_type == OP_JOIN || op_type == OP_JOIN_NS
}

/// Check if operator is filter.
fn is_filter_op(op_type: c_int) -> bool {
    op_type == OP_FILTER
}

/// Get shift direction from operator type.
fn get_shift_dir(op_type: c_int) -> c_int {
    if op_type == OP_LSHIFT {
        SHIFT_LEFT
    } else if op_type == OP_RSHIFT {
        SHIFT_RIGHT
    } else {
        SHIFT_NONE
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get SHIFT_LEFT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_shift_left() -> c_int {
    SHIFT_LEFT
}

/// FFI: Get SHIFT_RIGHT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_shift_right() -> c_int {
    SHIFT_RIGHT
}

/// FFI: Get SHIFT_NONE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_shift_none() -> c_int {
    SHIFT_NONE
}

/// FFI: Get FORMAT_GQ constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_format_gq() -> c_int {
    FORMAT_GQ
}

/// FFI: Get FORMAT_GW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_format_gw() -> c_int {
    FORMAT_GW
}

/// FFI: Get shift left character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_shift_left_char() -> c_int {
    shift_left_char()
}

/// FFI: Get shift right character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_shift_right_char() -> c_int {
    shift_right_char()
}

/// FFI: Get filter character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_filter_char() -> c_int {
    filter_char()
}

/// FFI: Get join character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_join_char() -> c_int {
    join_char()
}

/// FFI: Get tilde character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tilde_char() -> c_int {
    tilde_char()
}

/// FFI: Get format character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_format_char() -> c_int {
    format_char()
}

/// FFI: Check if operator is shift type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_shift_op(op_type: c_int) -> c_int {
    c_int::from(is_shift_op(op_type))
}

/// FFI: Check if operator is format type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_format_op(op_type: c_int) -> c_int {
    c_int::from(is_format_op(op_type))
}

/// FFI: Check if operator is join type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_join_op(op_type: c_int) -> c_int {
    c_int::from(is_join_op(op_type))
}

/// FFI: Check if operator is filter.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_filter_op(op_type: c_int) -> c_int {
    c_int::from(is_filter_op(op_type))
}

/// FFI: Get shift direction from operator.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_shift_dir(op_type: c_int) -> c_int {
    get_shift_dir(op_type)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_constants() {
        assert_eq!(SHIFT_LEFT, -1);
        assert_eq!(SHIFT_RIGHT, 1);
        assert_eq!(SHIFT_NONE, 0);
    }

    #[test]
    fn test_format_constants() {
        assert_eq!(FORMAT_GQ, 0);
        assert_eq!(FORMAT_GW, 1);
    }

    #[test]
    fn test_command_chars() {
        assert_eq!(shift_left_char(), c_int::from(b'<'));
        assert_eq!(shift_right_char(), c_int::from(b'>'));
        assert_eq!(filter_char(), c_int::from(b'!'));
        assert_eq!(join_char(), c_int::from(b'J'));
        assert_eq!(tilde_char(), c_int::from(b'~'));
    }

    #[test]
    fn test_is_shift_op() {
        assert!(is_shift_op(OP_LSHIFT));
        assert!(is_shift_op(OP_RSHIFT));
        assert!(!is_shift_op(OP_FORMAT));
        assert!(!is_shift_op(OP_JOIN));
    }

    #[test]
    fn test_is_format_op() {
        assert!(is_format_op(OP_FORMAT));
        assert!(is_format_op(OP_INDENT));
        assert!(!is_format_op(OP_LSHIFT));
    }

    #[test]
    fn test_is_join_op() {
        assert!(is_join_op(OP_JOIN));
        assert!(is_join_op(OP_JOIN_NS));
        assert!(!is_join_op(OP_FORMAT));
    }

    #[test]
    fn test_get_shift_dir() {
        assert_eq!(get_shift_dir(OP_LSHIFT), SHIFT_LEFT);
        assert_eq!(get_shift_dir(OP_RSHIFT), SHIFT_RIGHT);
        assert_eq!(get_shift_dir(OP_FORMAT), SHIFT_NONE);
    }
}
