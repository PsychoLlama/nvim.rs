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
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_safety_doc)]

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

    name.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_')
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
// Executable Helpers
// =============================================================================

/// Check if a filename looks like an executable name (has no path separators).
pub fn is_executable_name(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }
    // No path separators
    !name.iter().any(|&c| c == b'/' || c == b'\\')
}

/// Check if a path is absolute.
pub fn is_absolute_path(path: &[u8]) -> bool {
    if path.is_empty() {
        return false;
    }
    // Unix absolute path
    if path[0] == b'/' {
        return true;
    }
    // Windows absolute path (e.g., C:\)
    if path.len() >= 3 && path[1] == b':' && (path[2] == b'\\' || path[2] == b'/') {
        return true;
    }
    false
}

/// FFI export: check if name is an executable name (no path).
#[no_mangle]
pub unsafe extern "C" fn rs_is_executable_name(name: *const u8, len: c_int) -> bool {
    if name.is_null() || len < 0 {
        return false;
    }
    let slice = std::slice::from_raw_parts(name, len as usize);
    is_executable_name(slice)
}

/// FFI export: check if path is absolute.
#[no_mangle]
pub unsafe extern "C" fn rs_sys_is_absolute_path(path: *const u8, len: c_int) -> bool {
    if path.is_null() || len < 0 {
        return false;
    }
    let slice = std::slice::from_raw_parts(path, len as usize);
    is_absolute_path(slice)
}

// =============================================================================
// Tempname Helpers
// =============================================================================

/// Characters valid for temp file names.
const TEMP_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Generate a random temp filename suffix.
///
/// Returns the number of bytes written to `out`.
pub fn random_temp_suffix(out: &mut [u8], seed: u32) -> usize {
    if out.is_empty() {
        return 0;
    }

    let mut state = seed;
    let len = out.len().min(8);

    for byte in out.iter_mut().take(len) {
        // Simple LCG random
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let idx = (state >> 16) as usize % TEMP_CHARS.len();
        *byte = TEMP_CHARS[idx];
    }

    len
}

/// FFI export: generate random temp suffix.
#[no_mangle]
pub unsafe extern "C" fn rs_random_temp_suffix(out: *mut u8, len: c_int, seed: u32) -> c_int {
    if out.is_null() || len < 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(out, len as usize);
    random_temp_suffix(slice, seed) as c_int
}

// =============================================================================
// Feature Detection (has())
// =============================================================================

/// Feature categories for has() function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FeatureCategory {
    /// Unknown feature
    Unknown = 0,
    /// OS feature (win32, unix, mac)
    Os = 1,
    /// GUI feature (gui, gui_running)
    Gui = 2,
    /// Vim feature (vim_starting, etc.)
    Vim = 3,
    /// Patch feature (patch-X.Y.Z)
    Patch = 4,
    /// Nvim feature (nvim-X.Y.Z)
    Nvim = 5,
}

impl FeatureCategory {
    /// Categorize a feature name.
    pub fn categorize(name: &[u8]) -> Self {
        if name.is_empty() {
            return Self::Unknown;
        }

        // Check for patch- prefix
        if name.starts_with(b"patch-") {
            return Self::Patch;
        }

        // Check for nvim- prefix
        if name.starts_with(b"nvim-") {
            return Self::Nvim;
        }

        // OS features
        if matches!(
            name,
            b"unix" | b"win32" | b"win64" | b"mac" | b"macunix" | b"linux" | b"bsd"
        ) {
            return Self::Os;
        }

        // GUI features
        if name.starts_with(b"gui") {
            return Self::Gui;
        }

        // Vim-specific features
        if matches!(
            name,
            b"vim_starting"
                | b"vim_did_enter"
                | b"textlock"
                | b"autocmd"
                | b"eval"
                | b"syntax"
                | b"folding"
        ) {
            return Self::Vim;
        }

        Self::Unknown
    }
}

/// FFI export: categorize feature name.
#[no_mangle]
pub unsafe extern "C" fn rs_feature_category(name: *const u8, len: c_int) -> c_int {
    if name.is_null() || len < 0 {
        return FeatureCategory::Unknown as c_int;
    }
    let slice = std::slice::from_raw_parts(name, len as usize);
    FeatureCategory::categorize(slice) as c_int
}

// =============================================================================
// Hostname Helpers
// =============================================================================

/// Maximum hostname length (POSIX).
pub const MAX_HOSTNAME_LEN: usize = 255;

/// Validate a hostname string.
pub fn is_valid_hostname(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > MAX_HOSTNAME_LEN {
        return false;
    }

    // Hostname can contain alphanumeric, hyphen, and dot
    // Must not start or end with hyphen or dot
    if name[0] == b'-' || name[0] == b'.' {
        return false;
    }
    if name[name.len() - 1] == b'-' || name[name.len() - 1] == b'.' {
        return false;
    }

    name.iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'-' || c == b'.')
}

/// FFI export: validate hostname.
#[no_mangle]
pub unsafe extern "C" fn rs_is_valid_hostname(name: *const u8, len: c_int) -> bool {
    if name.is_null() || len < 0 {
        return false;
    }
    let slice = std::slice::from_raw_parts(name, len as usize);
    is_valid_hostname(slice)
}

// =============================================================================
// System Command Output Helpers
// =============================================================================

/// Maximum number of output lines to process (safety limit).
pub const MAX_OUTPUT_LINES: usize = 1_000_000;

/// Split system command output into lines.
///
/// Returns the number of lines found.
pub fn count_output_lines(output: &[u8]) -> usize {
    if output.is_empty() {
        return 0;
    }

    let mut count = 0;
    for &c in output {
        if c == b'\n' {
            count += 1;
        }
    }

    // Count last line if not ending with newline
    if output.last() != Some(&b'\n') {
        count += 1;
    }

    count.min(MAX_OUTPUT_LINES)
}

/// Check if output contains null bytes (binary data).
pub fn output_has_null(output: &[u8]) -> bool {
    output.contains(&0)
}

/// FFI export: count output lines.
#[no_mangle]
pub unsafe extern "C" fn rs_count_output_lines(output: *const u8, len: c_int) -> c_int {
    if output.is_null() || len < 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(output, len as usize);
    count_output_lines(slice) as c_int
}

/// FFI export: check for null bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_output_has_null(output: *const u8, len: c_int) -> bool {
    if output.is_null() || len < 0 {
        return false;
    }
    let slice = std::slice::from_raw_parts(output, len as usize);
    output_has_null(slice)
}

// =============================================================================
// Environment and path VimL functions (Phase 3)
// =============================================================================

use std::ffi::c_void;

extern "C" {
    fn nvim_eval_environ(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_stdpath(argvars: *const c_void, rettv: *mut c_void);
}

/// "environ()" function - get all environment variables as dict
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_environ"]
pub unsafe extern "C" fn rs_f_environ(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_environ(argvars, rettv);
}

/// "stdpath(type)" function - get standard path for given type
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_stdpath"]
pub unsafe extern "C" fn rs_f_stdpath(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_stdpath(argvars, rettv);
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

    #[test]
    fn test_executable_name() {
        assert!(is_executable_name(b"ls"));
        assert!(is_executable_name(b"nvim"));
        assert!(!is_executable_name(b"/usr/bin/ls"));
        assert!(!is_executable_name(b"./script"));
        assert!(!is_executable_name(b""));
    }

    #[test]
    fn test_absolute_path() {
        assert!(is_absolute_path(b"/usr/bin"));
        assert!(is_absolute_path(b"/"));
        assert!(!is_absolute_path(b"relative/path"));
        assert!(!is_absolute_path(b"./local"));
        assert!(!is_absolute_path(b""));

        // Windows paths
        assert!(is_absolute_path(b"C:\\Windows"));
        assert!(is_absolute_path(b"D:/data"));
    }

    #[test]
    fn test_random_temp_suffix() {
        let mut buf = [0u8; 8];
        let len = random_temp_suffix(&mut buf, 12345);
        assert_eq!(len, 8);
        // All characters should be valid temp chars
        for &c in &buf[..len] {
            assert!(TEMP_CHARS.contains(&c));
        }

        // Different seeds produce different results
        let mut buf2 = [0u8; 8];
        random_temp_suffix(&mut buf2, 54321);
        assert_ne!(buf, buf2);
    }

    #[test]
    fn test_feature_category() {
        assert_eq!(FeatureCategory::categorize(b"unix"), FeatureCategory::Os);
        assert_eq!(FeatureCategory::categorize(b"win32"), FeatureCategory::Os);
        assert_eq!(
            FeatureCategory::categorize(b"gui_running"),
            FeatureCategory::Gui
        );
        assert_eq!(
            FeatureCategory::categorize(b"patch-8.0.0001"),
            FeatureCategory::Patch
        );
        assert_eq!(
            FeatureCategory::categorize(b"nvim-0.5.0"),
            FeatureCategory::Nvim
        );
        assert_eq!(
            FeatureCategory::categorize(b"vim_starting"),
            FeatureCategory::Vim
        );
        assert_eq!(
            FeatureCategory::categorize(b"unknown_feat"),
            FeatureCategory::Unknown
        );
    }

    #[test]
    fn test_hostname_validation() {
        assert!(is_valid_hostname(b"localhost"));
        assert!(is_valid_hostname(b"my-host.domain.com"));
        assert!(is_valid_hostname(b"server01"));

        assert!(!is_valid_hostname(b""));
        assert!(!is_valid_hostname(b"-invalid"));
        assert!(!is_valid_hostname(b"invalid-"));
        assert!(!is_valid_hostname(b".invalid"));
        assert!(!is_valid_hostname(b"has space"));
    }

    #[test]
    fn test_output_lines() {
        assert_eq!(count_output_lines(b""), 0);
        assert_eq!(count_output_lines(b"one line"), 1);
        assert_eq!(count_output_lines(b"line1\nline2"), 2);
        assert_eq!(count_output_lines(b"line1\nline2\n"), 2);
        assert_eq!(count_output_lines(b"\n\n\n"), 3);
    }

    #[test]
    fn test_output_has_null() {
        assert!(!output_has_null(b"normal text"));
        assert!(output_has_null(b"has\x00null"));
        assert!(output_has_null(b"\x00"));
    }
}
