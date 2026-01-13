//! Modeline parsing and validation.
//!
//! This module provides helpers for modeline handling:
//! - Modeline detection
//! - Modeline parsing helpers
//! - Security validation

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::c_int;

// =============================================================================
// Modeline Type Constants
// =============================================================================

/// Standard vim modeline (vim: ...).
pub const MODELINE_VIM: c_int = 0;
/// First-line modeline (# vim: ...).
pub const MODELINE_FIRST: c_int = 1;
/// Last-line modeline.
pub const MODELINE_LAST: c_int = 2;
/// Ex command modeline.
pub const MODELINE_EX: c_int = 3;

// =============================================================================
// Modeline Security Flags
// =============================================================================

/// Modeline is secure (safe options only).
pub const ML_SECURE: c_int = 0x01;
/// Modeline allows expressions.
pub const ML_EXPR_OK: c_int = 0x02;
/// Modeline is in sandbox.
pub const ML_SANDBOX: c_int = 0x04;

// =============================================================================
// Modeline Helpers
// =============================================================================

/// Check if character could start modeline marker.
fn could_start_modeline(c: u8) -> bool {
    // Modelines start with whitespace, #, /*, etc.
    c == b' ' || c == b'\t' || c == b'#' || c == b'/' || c == b'*' || c == b'"'
}

/// Check if line contains "vim:" marker.
#[allow(dead_code)]
fn has_vim_marker(line: &[u8]) -> bool {
    // Look for "vim:" or "vi:" or "ex:"
    for window in line.windows(4) {
        if window == b"vim:" || window == b"Vim:" || window == b"VIM:" {
            return true;
        }
    }
    for window in line.windows(3) {
        if window == b"vi:" || window == b"Vi:" || window == b"VI:" {
            return true;
        }
        if window == b"ex:" || window == b"Ex:" || window == b"EX:" {
            return true;
        }
    }
    false
}

/// Check if modeline is secure.
fn is_secure_modeline(flags: c_int) -> bool {
    (flags & ML_SECURE) != 0
}

/// Check if modeline allows expressions.
fn allows_expressions(flags: c_int) -> bool {
    (flags & ML_EXPR_OK) != 0
}

/// Check if modeline is sandboxed.
fn is_sandboxed(flags: c_int) -> bool {
    (flags & ML_SANDBOX) != 0
}

/// Get modeline check range (number of lines to check).
#[allow(dead_code)]
fn modeline_check_range(total_lines: i64, modelines_setting: i64) -> (i64, i64) {
    // Check first N lines and last N lines
    let n = modelines_setting.max(0);
    if total_lines <= n * 2 {
        // Check all lines
        (1, total_lines)
    } else {
        // First N and last N
        (n, n)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get MODELINE_VIM constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_modeline_vim() -> c_int {
    MODELINE_VIM
}

/// FFI: Get MODELINE_FIRST constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_modeline_first() -> c_int {
    MODELINE_FIRST
}

/// FFI: Get MODELINE_LAST constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_modeline_last() -> c_int {
    MODELINE_LAST
}

/// FFI: Get MODELINE_EX constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_modeline_ex() -> c_int {
    MODELINE_EX
}

/// FFI: Get ML_SECURE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_ml_secure_flag() -> c_int {
    ML_SECURE
}

/// FFI: Get ML_EXPR_OK constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_ml_expr_ok_flag() -> c_int {
    ML_EXPR_OK
}

/// FFI: Get ML_SANDBOX constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_ml_sandbox_flag() -> c_int {
    ML_SANDBOX
}

/// FFI: Check if character could start modeline.
#[unsafe(no_mangle)]
pub extern "C" fn rs_could_start_modeline(c: c_int) -> c_int {
    c_int::from(could_start_modeline(c as u8))
}

/// FFI: Check if modeline is secure.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_secure_modeline(flags: c_int) -> c_int {
    c_int::from(is_secure_modeline(flags))
}

/// FFI: Check if modeline allows expressions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_ml_allows_expressions(flags: c_int) -> c_int {
    c_int::from(allows_expressions(flags))
}

/// FFI: Check if modeline is sandboxed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_sandboxed_modeline(flags: c_int) -> c_int {
    c_int::from(is_sandboxed(flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modeline_type_constants() {
        assert_eq!(MODELINE_VIM, 0);
        assert_eq!(MODELINE_FIRST, 1);
        assert_eq!(MODELINE_LAST, 2);
        assert_eq!(MODELINE_EX, 3);
    }

    #[test]
    fn test_modeline_security_flags() {
        assert_eq!(ML_SECURE, 0x01);
        assert_eq!(ML_EXPR_OK, 0x02);
        assert_eq!(ML_SANDBOX, 0x04);
    }

    #[test]
    fn test_could_start_modeline() {
        assert!(could_start_modeline(b' '));
        assert!(could_start_modeline(b'\t'));
        assert!(could_start_modeline(b'#'));
        assert!(could_start_modeline(b'/'));
        assert!(could_start_modeline(b'*'));
        assert!(could_start_modeline(b'"'));
        assert!(!could_start_modeline(b'a'));
        assert!(!could_start_modeline(b'v'));
    }

    #[test]
    fn test_has_vim_marker() {
        assert!(has_vim_marker(b"/* vim: set ts=4 : */"));
        assert!(has_vim_marker(b"# vim:ts=4"));
        assert!(has_vim_marker(b"// Vim: sw=2"));
        assert!(has_vim_marker(b"# vi:ts=4"));
        assert!(has_vim_marker(b"# ex:ts=4"));
        assert!(!has_vim_marker(b"hello world"));
        assert!(!has_vim_marker(b"vimrc"));
    }

    #[test]
    fn test_security_checks() {
        assert!(is_secure_modeline(ML_SECURE));
        assert!(!is_secure_modeline(0));

        assert!(allows_expressions(ML_EXPR_OK));
        assert!(!allows_expressions(ML_SECURE));

        assert!(is_sandboxed(ML_SANDBOX));
        assert!(!is_sandboxed(ML_SECURE));
    }

    #[test]
    fn test_modeline_check_range() {
        // Small file - check all
        assert_eq!(modeline_check_range(10, 5), (1, 10));

        // Large file - check first N and last N
        assert_eq!(modeline_check_range(100, 5), (5, 5));

        // Zero modelines setting
        assert_eq!(modeline_check_range(100, 0), (0, 0));

        // Negative setting (use 0)
        assert_eq!(modeline_check_range(100, -5), (0, 0));
    }
}
