//! System functions for VimL.
//!
//! This module implements system-related functions from `src/nvim/eval/funcs.c`:
//! - Environment variable helpers
//! - Executable path helpers
//! - Shell escape helpers
//!
//! ## Note
//!
//! These are helper functions for system operations.
//! Actual system calls should use the nvim-os crate.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Shell Type
// =============================================================================

/// Shell type for command execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ShellType {
    /// Default shell
    Default = 0,
    /// POSIX shell
    Posix = 1,
    /// Windows cmd.exe
    Cmd = 2,
    /// PowerShell
    PowerShell = 3,
}

impl ShellType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Posix,
            2 => Self::Cmd,
            3 => Self::PowerShell,
            _ => Self::Default,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Exit Code Helpers
// =============================================================================

/// Special shell exit codes.
pub mod exit_codes {
    /// Successful execution
    pub const SUCCESS: i64 = 0;
    /// Command not found
    pub const NOT_FOUND: i64 = 127;
    /// Killed by signal (128 + signal number)
    pub const SIGNAL_BASE: i64 = 128;
}

/// Check if exit code indicates success.
pub const fn is_success(code: i64) -> bool {
    code == exit_codes::SUCCESS
}

/// Check if exit code indicates command not found.
pub const fn is_not_found(code: i64) -> bool {
    code == exit_codes::NOT_FOUND
}

/// Check if exit code indicates killed by signal.
pub const fn is_signal(code: i64) -> bool {
    code >= exit_codes::SIGNAL_BASE && code < 256
}

/// Get signal number from exit code.
pub const fn get_signal(code: i64) -> Option<i64> {
    if is_signal(code) {
        Some(code - exit_codes::SIGNAL_BASE)
    } else {
        None
    }
}

/// FFI export: check if exit code is success.
#[no_mangle]
pub extern "C" fn rs_sys_is_success(code: i64) -> bool {
    is_success(code)
}

/// FFI export: check if killed by signal.
#[no_mangle]
pub extern "C" fn rs_sys_is_signal(code: i64) -> bool {
    is_signal(code)
}

// =============================================================================
// Environment Variable Helpers
// =============================================================================

/// Check if a string is a valid environment variable name.
///
/// Environment variable names should:
/// - Start with a letter or underscore
/// - Contain only letters, digits, and underscores
pub fn is_valid_env_name(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }

    let first = name[0];
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }

    name.iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

/// FFI export: validate environment variable name.
///
/// # Safety
/// - `name` must be a valid pointer to at least `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sys_is_valid_env_name(name: *const u8, len: c_int) -> bool {
    if name.is_null() || len < 0 {
        return false;
    }

    // SAFETY: Caller guarantees name points to at least len bytes
    let slice = unsafe { std::slice::from_raw_parts(name, len as usize) };
    is_valid_env_name(slice)
}

// =============================================================================
// Shell Escape Helpers
// =============================================================================

/// Characters that need escaping in POSIX shell.
const POSIX_SPECIAL: &[u8] = b"\"$`\\!";

/// Characters that need escaping in Windows cmd.
const CMD_SPECIAL: &[u8] = b"\"&|<>^%";

/// Check if character needs escaping for POSIX shell.
pub fn needs_posix_escape(c: u8) -> bool {
    POSIX_SPECIAL.contains(&c) || c.is_ascii_whitespace()
}

/// Check if character needs escaping for Windows cmd.
pub fn needs_cmd_escape(c: u8) -> bool {
    CMD_SPECIAL.contains(&c) || c.is_ascii_whitespace()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type() {
        assert_eq!(ShellType::from_c_int(0), ShellType::Default);
        assert_eq!(ShellType::from_c_int(1), ShellType::Posix);
    }

    #[test]
    fn test_exit_codes() {
        assert!(is_success(0));
        assert!(!is_success(1));
        assert!(is_not_found(127));
        assert!(is_signal(128)); // SIGHUP on many systems
        assert!(is_signal(137)); // SIGKILL (128 + 9)
        assert_eq!(get_signal(137), Some(9));
        assert_eq!(get_signal(0), None);
    }

    #[test]
    fn test_is_valid_env_name() {
        assert!(is_valid_env_name(b"PATH"));
        assert!(is_valid_env_name(b"_foo"));
        assert!(is_valid_env_name(b"FOO_BAR"));
        assert!(is_valid_env_name(b"foo123"));

        assert!(!is_valid_env_name(b""));
        assert!(!is_valid_env_name(b"123foo"));
        assert!(!is_valid_env_name(b"foo-bar"));
        assert!(!is_valid_env_name(b"foo.bar"));
    }

    #[test]
    fn test_shell_escape() {
        assert!(needs_posix_escape(b'"'));
        assert!(needs_posix_escape(b'$'));
        assert!(needs_posix_escape(b' '));
        assert!(!needs_posix_escape(b'a'));

        assert!(needs_cmd_escape(b'"'));
        assert!(needs_cmd_escape(b'|'));
        assert!(needs_cmd_escape(b' '));
        assert!(!needs_cmd_escape(b'a'));
    }
}
