//! Change operator commands.
//!
//! This module provides helper functions for change operations:
//! - nv_replace
//! - nv_subst
//! - nv_change
//! - nv_Undo

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use super::delete::OP_CHANGE;

// =============================================================================
// Change Mode Constants
// =============================================================================

/// Replace single character (r command).
pub const CHANGE_REPLACE: c_int = 0;
/// Substitute (s command).
pub const CHANGE_SUBST: c_int = 1;
/// Change operator (c command).
pub const CHANGE_OP: c_int = 2;
/// Change line (cc/S command).
pub const CHANGE_LINE: c_int = 3;

// =============================================================================
// Change Operation Helpers (Pure Rust)
// =============================================================================

/// Get change command character 'c'.
fn change_char() -> c_int {
    c_int::from(b'c')
}

/// Get substitute command character 's'.
fn subst_char() -> c_int {
    c_int::from(b's')
}

/// Get replace command character 'r'.
fn replace_char() -> c_int {
    c_int::from(b'r')
}

/// Get replace mode command character 'R'.
fn replace_mode_char() -> c_int {
    c_int::from(b'R')
}

/// Check if command is a change-type command.
fn is_change_cmd(cmdchar: c_int) -> bool {
    cmdchar == c_int::from(b'c')
        || cmdchar == c_int::from(b's')
        || cmdchar == c_int::from(b'S')
        || cmdchar == c_int::from(b'C')
}

/// Check if command is a replace-type command.
fn is_replace_cmd(cmdchar: c_int) -> bool {
    cmdchar == c_int::from(b'r') || cmdchar == c_int::from(b'R')
}

/// Check if operator goes to insert mode after.
fn op_enters_insert(op_type: c_int) -> bool {
    op_type == OP_CHANGE
}

/// Check if delete should stay at cursor (for change).
fn change_keeps_cursor(op_type: c_int) -> bool {
    op_type == OP_CHANGE
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get CHANGE_REPLACE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_change_replace() -> c_int {
    CHANGE_REPLACE
}

/// FFI: Get CHANGE_SUBST constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_change_subst() -> c_int {
    CHANGE_SUBST
}

/// FFI: Get CHANGE_OP constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_change_op() -> c_int {
    CHANGE_OP
}

/// FFI: Get CHANGE_LINE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_change_line() -> c_int {
    CHANGE_LINE
}

/// FFI: Get change character 'c'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_change_char() -> c_int {
    change_char()
}

/// FFI: Get substitute character 's'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_subst_char() -> c_int {
    subst_char()
}

/// FFI: Get replace character 'r'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_replace_char() -> c_int {
    replace_char()
}

/// FFI: Get replace mode character 'R'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_replace_mode_char() -> c_int {
    replace_mode_char()
}

/// FFI: Check if command is change-type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_change_cmd(cmdchar: c_int) -> c_int {
    c_int::from(is_change_cmd(cmdchar))
}

/// FFI: Check if command is replace-type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_replace_cmd(cmdchar: c_int) -> c_int {
    c_int::from(is_replace_cmd(cmdchar))
}

/// FFI: Check if operator enters insert mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_op_enters_insert(op_type: c_int) -> c_int {
    c_int::from(op_enters_insert(op_type))
}

/// FFI: Check if change keeps cursor position.
#[unsafe(no_mangle)]
pub extern "C" fn rs_change_keeps_cursor(op_type: c_int) -> c_int {
    c_int::from(change_keeps_cursor(op_type))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_constants() {
        assert_eq!(CHANGE_REPLACE, 0);
        assert_eq!(CHANGE_SUBST, 1);
        assert_eq!(CHANGE_OP, 2);
        assert_eq!(CHANGE_LINE, 3);
    }

    #[test]
    fn test_command_chars() {
        assert_eq!(change_char(), c_int::from(b'c'));
        assert_eq!(subst_char(), c_int::from(b's'));
        assert_eq!(replace_char(), c_int::from(b'r'));
        assert_eq!(replace_mode_char(), c_int::from(b'R'));
    }

    #[test]
    fn test_is_change_cmd() {
        assert!(is_change_cmd(c_int::from(b'c')));
        assert!(is_change_cmd(c_int::from(b's')));
        assert!(is_change_cmd(c_int::from(b'S')));
        assert!(is_change_cmd(c_int::from(b'C')));
        assert!(!is_change_cmd(c_int::from(b'd')));
        assert!(!is_change_cmd(c_int::from(b'r')));
    }

    #[test]
    fn test_is_replace_cmd() {
        assert!(is_replace_cmd(c_int::from(b'r')));
        assert!(is_replace_cmd(c_int::from(b'R')));
        assert!(!is_replace_cmd(c_int::from(b'c')));
        assert!(!is_replace_cmd(c_int::from(b's')));
    }

    #[test]
    fn test_op_enters_insert() {
        assert!(op_enters_insert(OP_CHANGE));
        // OP_DELETE = 1, test that it doesn't enter insert
        assert!(!op_enters_insert(1));
    }
}
