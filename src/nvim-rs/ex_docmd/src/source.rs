//! Source file processing utilities for Ex commands.
//!
//! This module provides types and utilities for processing sourced files,
//! including tracking the current source file and line number.

use std::ffi::{c_char, c_int};

// =============================================================================
// FFI declarations for C sourcing state
// =============================================================================

extern "C" {
    fn nvim_get_sourcing_name() -> *const c_char;
    fn nvim_get_sourcing_lnum() -> c_int;
    fn nvim_get_exestack_len() -> c_int;
}

// =============================================================================
// Sourcing state accessors
// =============================================================================

/// Get the current sourcing file name.
///
/// Returns the name of the file currently being sourced, or NULL if
/// no file is being sourced.
///
/// # Safety
///
/// Returns a pointer to internal C storage. Caller must not free.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sourcing_name() -> *const c_char {
    nvim_get_sourcing_name()
}

/// Get the current sourcing line number.
///
/// Returns the line number in the file currently being sourced,
/// or 0 if no file is being sourced.
#[no_mangle]
pub extern "C" fn rs_get_sourcing_lnum() -> c_int {
    unsafe { nvim_get_sourcing_lnum() }
}

/// Check if we're currently sourcing a file.
///
/// Returns true if a file is currently being sourced (executed).
#[inline]
pub fn is_sourcing() -> bool {
    unsafe { nvim_get_exestack_len() > 0 && !nvim_get_sourcing_name().is_null() }
}

/// FFI wrapper for sourcing check.
#[no_mangle]
pub extern "C" fn rs_is_sourcing() -> c_int {
    c_int::from(is_sourcing())
}

/// Check if we have valid sourcing information.
///
/// Returns true if we have both a sourcing name and it's not empty.
#[no_mangle]
pub unsafe extern "C" fn rs_have_sourcing_info() -> c_int {
    let name = nvim_get_sourcing_name();
    if name.is_null() {
        return 0;
    }
    // Check if first character is not NUL
    c_int::from(*name != 0)
}

// =============================================================================
// Source file path utilities
// =============================================================================

/// Check if a filename looks like an autoload script.
///
/// Autoload scripts have paths like "autoload/foo.vim" or "autoload/foo/bar.vim".
#[inline]
pub fn is_autoload_path(path: &[u8]) -> bool {
    // Look for "autoload/" or "autoload\" in the path
    let autoload = b"autoload";
    for i in 0..path.len().saturating_sub(autoload.len()) {
        if &path[i..i + autoload.len()] == autoload {
            let next = path.get(i + autoload.len()).copied().unwrap_or(0);
            if next == b'/' || next == b'\\' {
                return true;
            }
        }
    }
    false
}

/// FFI wrapper for autoload path check.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_is_autoload_path(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    // Find length
    let mut len = 0;
    let mut p = path;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(path as *const u8, len);
    c_int::from(is_autoload_path(slice))
}

/// Check if a filename ends with ".vim" or ".lua".
///
/// These are the standard script extensions for Neovim.
#[inline]
pub fn is_script_extension(path: &[u8]) -> bool {
    if path.len() < 4 {
        return false;
    }
    let suffix = &path[path.len() - 4..];
    suffix == b".vim" || suffix == b".lua"
}

/// FFI wrapper for script extension check.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_is_script_extension(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    // Find length
    let mut len = 0;
    let mut p = path;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if len < 4 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(path as *const u8, len);
    c_int::from(is_script_extension(slice))
}

// =============================================================================
// Source execution state
// =============================================================================

/// Source execution flags for do_source().
pub const DIP_ALL: c_int = 0x01; // Execute all matching files
pub const DIP_DIR: c_int = 0x02; // Allow directories
pub const DIP_ERR: c_int = 0x04; // Show errors
pub const DIP_START: c_int = 0x08; // Source start script
pub const DIP_OPT: c_int = 0x10; // Source optional script
pub const DIP_NORTP: c_int = 0x20; // Don't use runtimepath
pub const DIP_NOAFTER: c_int = 0x40; // Don't use after directories
pub const DIP_ONCE: c_int = 0x80; // Source only if not done before

/// Check if DIP_ALL flag is set.
#[inline]
pub const fn has_dip_all(flags: c_int) -> bool {
    (flags & DIP_ALL) != 0
}

/// Check if DIP_ERR flag is set.
#[inline]
pub const fn has_dip_err(flags: c_int) -> bool {
    (flags & DIP_ERR) != 0
}

/// Check if DIP_ONCE flag is set.
#[inline]
pub const fn has_dip_once(flags: c_int) -> bool {
    (flags & DIP_ONCE) != 0
}

/// FFI wrapper for DIP_ALL flag check.
#[no_mangle]
pub extern "C" fn rs_has_dip_all(flags: c_int) -> c_int {
    c_int::from(has_dip_all(flags))
}

/// FFI wrapper for DIP_ERR flag check.
#[no_mangle]
pub extern "C" fn rs_has_dip_err(flags: c_int) -> c_int {
    c_int::from(has_dip_err(flags))
}

/// FFI wrapper for DIP_ONCE flag check.
#[no_mangle]
pub extern "C" fn rs_has_dip_once(flags: c_int) -> c_int {
    c_int::from(has_dip_once(flags))
}

// =============================================================================
// Line reading helpers
// =============================================================================

/// Check if a line is a continuation line (starts with backslash).
///
/// In Vim script, a line that starts with a backslash continues
/// the previous line.
#[inline]
pub fn is_continuation_line(line: &[u8]) -> bool {
    !line.is_empty() && line[0] == b'\\'
}

/// FFI wrapper for continuation line check.
///
/// # Safety
///
/// `line` must be a valid C string pointer or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_is_continuation_line(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    c_int::from(*line as u8 == b'\\')
}

/// Check if a line is blank (empty or whitespace only).
#[inline]
pub fn line_is_blank(line: &[u8]) -> bool {
    for &c in line {
        if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' && c != 0 {
            return false;
        }
    }
    true
}

/// Check if a line is a comment (starts with " after whitespace).
#[inline]
pub fn line_is_comment(line: &[u8]) -> bool {
    let mut i = 0;
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }
    i < line.len() && line[i] == b'"'
}

/// FFI wrapper for comment line check.
///
/// # Safety
///
/// `line` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_line_is_comment(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let mut p = line;
    // Skip whitespace
    while (*p as u8) == b' ' || (*p as u8) == b'\t' {
        p = p.add(1);
    }
    // Check for comment character
    c_int::from((*p as u8) == b'"')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_autoload_path() {
        assert!(is_autoload_path(b"autoload/foo.vim"));
        assert!(is_autoload_path(b"/path/autoload/bar.vim"));
        // Windows path separator
        assert!(is_autoload_path(b"autoload\\foo.vim"));
        // No match - no separator after "autoload"
        assert!(!is_autoload_path(b"autoloader"));
        assert!(!is_autoload_path(b"plugin/foo.vim"));
        assert!(!is_autoload_path(b""));
    }

    #[test]
    fn test_is_script_extension() {
        assert!(is_script_extension(b"foo.vim"));
        assert!(is_script_extension(b"bar.lua"));
        assert!(is_script_extension(b"/path/to/script.vim"));
        assert!(!is_script_extension(b"foo.txt"));
        assert!(!is_script_extension(b"vim"));
        assert!(!is_script_extension(b""));
    }

    #[test]
    fn test_dip_flags() {
        assert!(has_dip_all(DIP_ALL));
        assert!(!has_dip_all(DIP_ERR));
        assert!(has_dip_err(DIP_ERR));
        assert!(has_dip_once(DIP_ONCE));

        let combined = DIP_ALL | DIP_ERR | DIP_ONCE;
        assert!(has_dip_all(combined));
        assert!(has_dip_err(combined));
        assert!(has_dip_once(combined));
    }

    #[test]
    fn test_is_continuation_line() {
        assert!(is_continuation_line(b"\\continued"));
        assert!(is_continuation_line(b"\\ text"));
        assert!(!is_continuation_line(b" \\not"));
        assert!(!is_continuation_line(b"normal"));
        assert!(!is_continuation_line(b""));
    }

    #[test]
    fn test_line_is_blank() {
        assert!(line_is_blank(b""));
        assert!(line_is_blank(b"   "));
        assert!(line_is_blank(b"\t\t"));
        assert!(line_is_blank(b" \t "));
        assert!(line_is_blank(b"\n"));
        assert!(!line_is_blank(b"x"));
        assert!(!line_is_blank(b"  x  "));
    }

    #[test]
    fn test_line_is_comment() {
        assert!(line_is_comment(b"\" comment"));
        assert!(line_is_comment(b"  \" comment"));
        assert!(line_is_comment(b"\t\" comment"));
        assert!(line_is_comment(b"\""));
        assert!(!line_is_comment(b"not a comment"));
        assert!(!line_is_comment(b""));
        assert!(!line_is_comment(b"x \" not comment"));
    }

    #[test]
    fn test_ffi_wrappers() {
        use std::ffi::CString;

        unsafe {
            // Autoload path
            let path = CString::new("autoload/foo.vim").unwrap();
            assert_eq!(rs_is_autoload_path(path.as_ptr()), 1);

            let path = CString::new("normal/foo.vim").unwrap();
            assert_eq!(rs_is_autoload_path(path.as_ptr()), 0);

            // Script extension
            let path = CString::new("script.vim").unwrap();
            assert_eq!(rs_is_script_extension(path.as_ptr()), 1);

            let path = CString::new("script.txt").unwrap();
            assert_eq!(rs_is_script_extension(path.as_ptr()), 0);

            // Continuation line
            let line = CString::new("\\continued").unwrap();
            assert_eq!(rs_is_continuation_line(line.as_ptr()), 1);

            let line = CString::new("normal").unwrap();
            assert_eq!(rs_is_continuation_line(line.as_ptr()), 0);

            // Comment line
            let line = CString::new("\" comment").unwrap();
            assert_eq!(rs_line_is_comment(line.as_ptr()), 1);

            let line = CString::new("code").unwrap();
            assert_eq!(rs_line_is_comment(line.as_ptr()), 0);
        }
    }

    #[test]
    fn test_dip_flag_values() {
        // Verify flag values don't overlap
        assert_eq!(DIP_ALL & DIP_DIR, 0);
        assert_eq!(DIP_ALL & DIP_ERR, 0);
        assert_eq!(DIP_DIR & DIP_ERR, 0);

        // Verify expected values
        assert_eq!(DIP_ALL, 0x01);
        assert_eq!(DIP_DIR, 0x02);
        assert_eq!(DIP_ERR, 0x04);
        assert_eq!(DIP_START, 0x08);
        assert_eq!(DIP_OPT, 0x10);
        assert_eq!(DIP_NORTP, 0x20);
        assert_eq!(DIP_NOAFTER, 0x40);
        assert_eq!(DIP_ONCE, 0x80);
    }
}
