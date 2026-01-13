//! Yank operator commands.
//!
//! This module provides helper functions for yank operations:
//! - nv_yank
//! - nv_put
//! - nv_record
//! - nv_at

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Yank/Put Constants
// =============================================================================

/// Put after cursor (p command).
pub const PUT_AFTER: c_int = 0;
/// Put before cursor (P command).
pub const PUT_BEFORE: c_int = 1;
/// Put from register (specified).
pub const PUT_REGISTER: c_int = 2;

// =============================================================================
// Register Constants
// =============================================================================

/// Unnamed register (default).
pub const REG_UNNAMED: c_int = 0;
/// Named register a-z.
pub const REG_NAMED_START: c_int = c_int::from_ne_bytes([b'a', 0, 0, 0]);
/// Small delete register "-".
pub const REG_SMALL_DELETE: c_int = c_int::from_ne_bytes([b'-', 0, 0, 0]);
/// Expression register "=".
pub const REG_EXPRESSION: c_int = c_int::from_ne_bytes([b'=', 0, 0, 0]);
/// Clipboard register "+".
pub const REG_CLIPBOARD: c_int = c_int::from_ne_bytes([b'+', 0, 0, 0]);
/// Selection register "*".
pub const REG_SELECTION: c_int = c_int::from_ne_bytes([b'*', 0, 0, 0]);
/// Black hole register "_".
pub const REG_BLACKHOLE: c_int = c_int::from_ne_bytes([b'_', 0, 0, 0]);
/// Last search pattern register "/".
pub const REG_SEARCH: c_int = c_int::from_ne_bytes([b'/', 0, 0, 0]);

// =============================================================================
// Yank Operation Helpers (Pure Rust)
// =============================================================================

/// Get yank command character 'y'.
fn yank_char() -> c_int {
    c_int::from(b'y')
}

/// Get put command character 'p'.
fn put_char() -> c_int {
    c_int::from(b'p')
}

/// Get put before character 'P'.
fn put_before_char() -> c_int {
    c_int::from(b'P')
}

/// Check if register is named (a-z or A-Z).
fn is_named_register(reg: c_int) -> bool {
    // Check ranges directly using c_int
    (reg >= c_int::from(b'a') && reg <= c_int::from(b'z'))
        || (reg >= c_int::from(b'A') && reg <= c_int::from(b'Z'))
}

/// Check if register is numbered (0-9).
fn is_numbered_register(reg: c_int) -> bool {
    reg >= c_int::from(b'0') && reg <= c_int::from(b'9')
}

/// Check if register is the black hole register.
fn is_blackhole_register(reg: c_int) -> bool {
    reg == c_int::from(b'_')
}

/// Check if register is clipboard or selection.
fn is_clipboard_register(reg: c_int) -> bool {
    reg == c_int::from(b'+') || reg == c_int::from(b'*')
}

/// Check if character is valid register name.
fn is_valid_register(reg: c_int) -> bool {
    if reg == 0 {
        return true; // unnamed
    }
    is_named_register(reg)
        || is_numbered_register(reg)
        || reg == c_int::from(b'"')
        || reg == c_int::from(b'-')
        || reg == c_int::from(b'_')
        || reg == c_int::from(b'+')
        || reg == c_int::from(b'*')
        || reg == c_int::from(b'=')
        || reg == c_int::from(b'/')
        || reg == c_int::from(b'#')
        || reg == c_int::from(b'%')
        || reg == c_int::from(b':')
        || reg == c_int::from(b'.')
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get PUT_AFTER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_put_after() -> c_int {
    PUT_AFTER
}

/// FFI: Get PUT_BEFORE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_put_before() -> c_int {
    PUT_BEFORE
}

/// FFI: Get yank character 'y'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_yank_char() -> c_int {
    yank_char()
}

/// FFI: Get put character 'p'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_put_char() -> c_int {
    put_char()
}

/// FFI: Get put before character 'P'.
#[unsafe(no_mangle)]
pub extern "C" fn rs_put_before_char() -> c_int {
    put_before_char()
}

/// FFI: Check if register is named.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_named_register(reg: c_int) -> c_int {
    c_int::from(is_named_register(reg))
}

/// FFI: Check if register is numbered.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_numbered_register(reg: c_int) -> c_int {
    c_int::from(is_numbered_register(reg))
}

/// FFI: Check if register is blackhole.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_blackhole_register(reg: c_int) -> c_int {
    c_int::from(is_blackhole_register(reg))
}

/// FFI: Check if register is clipboard.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_clipboard_register(reg: c_int) -> c_int {
    c_int::from(is_clipboard_register(reg))
}

/// FFI: Check if valid register name (for yank operations).
/// Note: rs_is_valid_register exists in ex_docmd, this provides yank-specific validation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_yank_is_valid_register(reg: c_int) -> c_int {
    c_int::from(is_valid_register(reg))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_constants() {
        assert_eq!(PUT_AFTER, 0);
        assert_eq!(PUT_BEFORE, 1);
    }

    #[test]
    fn test_command_chars() {
        assert_eq!(yank_char(), c_int::from(b'y'));
        assert_eq!(put_char(), c_int::from(b'p'));
        assert_eq!(put_before_char(), c_int::from(b'P'));
    }

    #[test]
    fn test_is_named_register() {
        assert!(is_named_register(c_int::from(b'a')));
        assert!(is_named_register(c_int::from(b'z')));
        assert!(is_named_register(c_int::from(b'A')));
        assert!(is_named_register(c_int::from(b'Z')));
        assert!(!is_named_register(c_int::from(b'0')));
        assert!(!is_named_register(c_int::from(b'+')));
    }

    #[test]
    fn test_is_numbered_register() {
        assert!(is_numbered_register(c_int::from(b'0')));
        assert!(is_numbered_register(c_int::from(b'9')));
        assert!(!is_numbered_register(c_int::from(b'a')));
    }

    #[test]
    fn test_is_blackhole_register() {
        assert!(is_blackhole_register(c_int::from(b'_')));
        assert!(!is_blackhole_register(c_int::from(b'a')));
    }

    #[test]
    fn test_is_clipboard_register() {
        assert!(is_clipboard_register(c_int::from(b'+')));
        assert!(is_clipboard_register(c_int::from(b'*')));
        assert!(!is_clipboard_register(c_int::from(b'a')));
    }

    #[test]
    fn test_is_valid_register() {
        assert!(is_valid_register(0)); // unnamed
        assert!(is_valid_register(c_int::from(b'a')));
        assert!(is_valid_register(c_int::from(b'0')));
        assert!(is_valid_register(c_int::from(b'"')));
        assert!(is_valid_register(c_int::from(b'+')));
        assert!(is_valid_register(c_int::from(b'_')));
    }
}
