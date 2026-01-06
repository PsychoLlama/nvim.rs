//! Option initialization utilities
//!
//! This module provides Rust implementations of helper functions used during
//! Neovim's option initialization. The actual initialization sequence remains
//! in C due to global state dependencies.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit
#![allow(clippy::cast_possible_wrap)] // FFI with C char types

use std::ffi::{c_char, c_int};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)] // FFI functions used when linked with C
extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
}

// =============================================================================
// Help Language Default
// =============================================================================

/// Result of processing a language string for 'helplang'.
#[repr(C)]
pub struct HelplangResult {
    /// Two-letter language code (or empty if invalid)
    pub code: [c_char; 3],
    /// Whether the result is valid
    pub valid: c_int,
}

/// Process a language string to extract the 'helplang' default.
///
/// Converts locale strings to two-letter language codes:
/// - "zh_CN" -> "cn"
/// - "zh_TW" -> "tw"
/// - "C", "C.UTF-8", etc. -> "en"
/// - "en_US" -> "en"
/// - Other -> first two letters lowercased
///
/// # Arguments
/// * `lang` - The locale/language string (e.g., from LANG environment)
///
/// # Returns
/// A struct containing the two-letter code and validity flag.
#[no_mangle]
pub unsafe extern "C" fn rs_compute_helplang(lang: *const c_char) -> HelplangResult {
    let mut result = HelplangResult {
        code: [0, 0, 0],
        valid: 0,
    };

    if lang.is_null() || *lang == 0 {
        return result;
    }

    // Get length
    let mut len: usize = 0;
    let mut p = lang;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    let b0 = *lang as u8;

    // Check for C locale (C, C.UTF-8, etc.) - handle single char case
    if b0 == b'C' && (len == 1 || *lang.add(1) as u8 == b'.') {
        result.code[0] = b'e' as c_char;
        result.code[1] = b'n' as c_char;
        result.valid = 1;
        return result;
    }

    // Need at least 2 characters for other checks
    if len < 2 {
        return result;
    }

    let b1 = *lang.add(1) as u8;

    // Check for zh_CN or zh_TW
    if len >= 5 && b0 == b'z' && b1 == b'h' && *lang.add(2) as u8 == b'_' {
        // zh_CN -> cn, zh_TW -> tw
        result.code[0] = (*lang.add(3) as u8).to_ascii_lowercase() as c_char;
        result.code[1] = (*lang.add(4) as u8).to_ascii_lowercase() as c_char;
        result.valid = 1;
        return result;
    }

    // POSIX locale
    if len >= 5
        && b0 == b'P'
        && b1 == b'O'
        && *lang.add(2) as u8 == b'S'
        && *lang.add(3) as u8 == b'I'
        && *lang.add(4) as u8 == b'X'
    {
        result.code[0] = b'e' as c_char;
        result.code[1] = b'n' as c_char;
        result.valid = 1;
        return result;
    }

    // Default: use first two letters, lowercased
    result.code[0] = b0.to_ascii_lowercase() as c_char;
    result.code[1] = b1.to_ascii_lowercase() as c_char;
    result.valid = 1;
    result
}

// =============================================================================
// Shell Default
// =============================================================================

/// Check if a shell path needs quoting (contains spaces).
///
/// # Returns
/// 1 if the shell path contains spaces and needs quoting, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_shell_needs_quoting(shell: *const c_char) -> c_int {
    if shell.is_null() || *shell == 0 {
        return 0;
    }

    let mut p = shell;
    while *p != 0 {
        if *p as u8 == b' ' {
            return 1;
        }
        p = p.add(1);
    }
    0
}

/// Compute the length needed for a quoted shell path.
///
/// If the shell path contains spaces, it will be quoted with double quotes.
/// Returns the length including quotes and null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_quoted_shell_len(shell: *const c_char) -> usize {
    if shell.is_null() || *shell == 0 {
        return 1; // Just the null terminator
    }

    let mut len: usize = 0;
    let mut p = shell;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if rs_shell_needs_quoting(shell) != 0 {
        len + 3 // Two quotes + null terminator
    } else {
        len + 1 // Just null terminator
    }
}

// =============================================================================
// CDPATH Processing
// =============================================================================

/// Compute the length needed for a converted CDPATH value.
///
/// CDPATH uses ':' as separator on Unix (';' on Windows), which gets
/// converted to ',' for Vim's internal format. Spaces and commas in
/// paths get escaped with backslash.
///
/// Returns the length needed including null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_cdpath_converted_len(cdpath: *const c_char) -> usize {
    if cdpath.is_null() || *cdpath == 0 {
        return 2; // Just ",\0" (current dir)
    }

    let mut len: usize = 2; // Start with "," for current dir, plus null
    let mut p = cdpath;

    while *p != 0 {
        let c = *p as u8;
        // Path list separator becomes comma
        if c == b':' || c == b';' {
            len += 1;
        } else if c == b' ' || c == b',' {
            // Spaces and commas need escaping
            len += 2;
        } else {
            len += 1;
        }
        p = p.add(1);
    }

    len
}

/// Convert a CDPATH value to Vim's internal format.
///
/// - Adds leading comma (current directory first)
/// - Converts path separators (':' or ';') to ','
/// - Escapes spaces and commas with backslash
///
/// # Arguments
/// * `cdpath` - The CDPATH environment variable value
/// * `buf` - Buffer to write the converted value
/// * `buflen` - Length of the buffer
///
/// # Returns
/// The length of the converted string (not including null terminator).
#[no_mangle]
pub unsafe extern "C" fn rs_convert_cdpath(
    cdpath: *const c_char,
    buf: *mut c_char,
    buflen: usize,
) -> usize {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    // Start with comma for current directory
    *buf = b',' as c_char;
    let mut j: usize = 1;

    if cdpath.is_null() || *cdpath == 0 {
        if buflen > 1 {
            *buf.add(1) = 0;
        }
        return 1;
    }

    let mut p = cdpath;
    while *p != 0 && j < buflen - 1 {
        let c = *p as u8;

        if c == b':' || c == b';' {
            // Path separator becomes comma
            *buf.add(j) = b',' as c_char;
            j += 1;
        } else if c == b' ' || c == b',' {
            // Escape spaces and commas
            if j < buflen - 2 {
                *buf.add(j) = b'\\' as c_char;
                j += 1;
                *buf.add(j) = c as c_char;
                j += 1;
            }
        } else {
            *buf.add(j) = c as c_char;
            j += 1;
        }

        p = p.add(1);
    }

    if j < buflen {
        *buf.add(j) = 0;
    }

    j
}

// =============================================================================
// Backupskip Default Helpers
// =============================================================================

/// Check if a path should be added to backupskip.
///
/// Returns 1 if the path is valid and non-empty, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_valid_backupskip_path(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }
    1
}

/// Check if a path has a trailing path separator.
///
/// # Returns
/// 1 if path ends with '/' (Unix) or '\\' (Windows), 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_has_trailing_pathsep(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find end of string
    let mut p = path;
    while *p.add(1) != 0 {
        p = p.add(1);
    }

    let last = *p as u8;
    c_int::from(last == b'/' || last == b'\\')
}

/// Compute the pattern for a backupskip entry.
///
/// Creates a pattern like "/tmp/*" from a path like "/tmp".
/// Handles trailing separators correctly.
///
/// # Arguments
/// * `path` - The directory path
/// * `buf` - Buffer to write the pattern
/// * `buflen` - Length of the buffer
///
/// # Returns
/// The length of the pattern (not including null terminator).
#[no_mangle]
pub unsafe extern "C" fn rs_make_backupskip_pattern(
    path: *const c_char,
    buf: *mut c_char,
    buflen: usize,
) -> usize {
    if buf.is_null() || buflen == 0 || path.is_null() {
        return 0;
    }

    // Copy path
    let mut j: usize = 0;
    let mut p = path;
    while *p != 0 && j < buflen - 3 {
        *buf.add(j) = *p;
        j += 1;
        p = p.add(1);
    }

    // Add path separator if not present
    if j > 0 && rs_has_trailing_pathsep(path) == 0 && j < buflen - 2 {
        *buf.add(j) = b'/' as c_char;
        j += 1;
    }

    // Add wildcard
    if j < buflen - 1 {
        *buf.add(j) = b'*' as c_char;
        j += 1;
    }

    // Null terminate
    if j < buflen {
        *buf.add(j) = 0;
    }

    j
}

// =============================================================================
// Default Detection Utilities
// =============================================================================

/// Detect if we're in a Unix-like environment.
///
/// This is a compile-time constant exposed to Rust code.
#[no_mangle]
pub extern "C" fn rs_is_unix() -> c_int {
    #[cfg(unix)]
    {
        1
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// Get the default temporary directory for the current platform.
///
/// Returns a pointer to a static string:
/// - macOS: "/private/tmp"
/// - Other Unix: "/tmp"
/// - Windows: NULL (use environment variable)
#[no_mangle]
pub extern "C" fn rs_default_tmpdir() -> *const c_char {
    #[cfg(target_os = "macos")]
    {
        c"/private/tmp".as_ptr()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        c"/tmp".as_ptr()
    }
    #[cfg(not(unix))]
    {
        std::ptr::null()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_compute_helplang() {
        unsafe {
            // Test zh_CN -> cn
            let zh_cn = CString::new("zh_CN.UTF-8").unwrap();
            let result = rs_compute_helplang(zh_cn.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'c');
            assert_eq!(result.code[1] as u8, b'n');

            // Test zh_TW -> tw
            let zh_tw = CString::new("zh_TW").unwrap();
            let result = rs_compute_helplang(zh_tw.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b't');
            assert_eq!(result.code[1] as u8, b'w');

            // Test C locale -> en
            let c_locale = CString::new("C").unwrap();
            let result = rs_compute_helplang(c_locale.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test C.UTF-8 -> en
            let c_utf8 = CString::new("C.UTF-8").unwrap();
            let result = rs_compute_helplang(c_utf8.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test POSIX -> en
            let posix = CString::new("POSIX").unwrap();
            let result = rs_compute_helplang(posix.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test en_US -> en
            let en_us = CString::new("en_US.UTF-8").unwrap();
            let result = rs_compute_helplang(en_us.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test de_DE -> de
            let de_de = CString::new("de_DE").unwrap();
            let result = rs_compute_helplang(de_de.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'd');
            assert_eq!(result.code[1] as u8, b'e');

            // Test invalid (too short)
            let short = CString::new("x").unwrap();
            let result = rs_compute_helplang(short.as_ptr());
            assert_eq!(result.valid, 0);

            // Test null
            let result = rs_compute_helplang(ptr::null());
            assert_eq!(result.valid, 0);
        }
    }

    #[test]
    fn test_shell_needs_quoting() {
        unsafe {
            let with_space = CString::new("/bin/my shell").unwrap();
            let without_space = CString::new("/bin/bash").unwrap();

            assert_eq!(rs_shell_needs_quoting(with_space.as_ptr()), 1);
            assert_eq!(rs_shell_needs_quoting(without_space.as_ptr()), 0);
            assert_eq!(rs_shell_needs_quoting(ptr::null()), 0);
        }
    }

    #[test]
    fn test_quoted_shell_len() {
        unsafe {
            let with_space = CString::new("/bin/my shell").unwrap();
            let without_space = CString::new("/bin/bash").unwrap();

            // "/bin/my shell" (13) + 2 quotes + null = 16
            assert_eq!(rs_quoted_shell_len(with_space.as_ptr()), 16);
            // "/bin/bash" (9) + null = 10
            assert_eq!(rs_quoted_shell_len(without_space.as_ptr()), 10);
        }
    }

    #[test]
    fn test_cdpath_converted_len() {
        unsafe {
            // "/foo:/bar" -> ",/foo,/bar" (10) + null = 11
            let cdpath = CString::new("/foo:/bar").unwrap();
            assert_eq!(rs_cdpath_converted_len(cdpath.as_ptr()), 11);

            // "/path with space" -> ",/path\\ with\\ space" (needs escaping)
            let with_space = CString::new("/path with space").unwrap();
            // 16 chars + 2 escapes + 1 leading comma + null = 20
            assert_eq!(rs_cdpath_converted_len(with_space.as_ptr()), 20);

            // Empty -> just ",\0"
            assert_eq!(rs_cdpath_converted_len(ptr::null()), 2);
        }
    }

    #[test]
    fn test_convert_cdpath() {
        unsafe {
            let cdpath = CString::new("/foo:/bar").unwrap();
            let mut buf = [0i8; 32];

            let len = rs_convert_cdpath(cdpath.as_ptr(), buf.as_mut_ptr(), 32);
            assert_eq!(len, 10); // ",/foo,/bar"

            // Check the result
            assert_eq!(buf[0] as u8, b',');
            assert_eq!(buf[1] as u8, b'/');
            assert_eq!(buf[2] as u8, b'f');
            assert_eq!(buf[5] as u8, b','); // Separator converted
        }
    }

    #[test]
    fn test_has_trailing_pathsep() {
        unsafe {
            let with_sep = CString::new("/tmp/").unwrap();
            let without_sep = CString::new("/tmp").unwrap();

            assert_eq!(rs_has_trailing_pathsep(with_sep.as_ptr()), 1);
            assert_eq!(rs_has_trailing_pathsep(without_sep.as_ptr()), 0);
            assert_eq!(rs_has_trailing_pathsep(ptr::null()), 0);
        }
    }

    #[test]
    fn test_make_backupskip_pattern() {
        unsafe {
            let path = CString::new("/tmp").unwrap();
            let mut buf = [0i8; 32];

            let len = rs_make_backupskip_pattern(path.as_ptr(), buf.as_mut_ptr(), 32);
            assert_eq!(len, 6); // "/tmp/*"

            // Check result
            assert_eq!(buf[0] as u8, b'/');
            assert_eq!(buf[1] as u8, b't');
            assert_eq!(buf[2] as u8, b'm');
            assert_eq!(buf[3] as u8, b'p');
            assert_eq!(buf[4] as u8, b'/');
            assert_eq!(buf[5] as u8, b'*');

            // With trailing separator
            let path_sep = CString::new("/tmp/").unwrap();
            let len = rs_make_backupskip_pattern(path_sep.as_ptr(), buf.as_mut_ptr(), 32);
            assert_eq!(len, 6); // "/tmp/*" (no double separator)
        }
    }

    #[test]
    fn test_is_unix() {
        #[cfg(unix)]
        assert_eq!(rs_is_unix(), 1);
        #[cfg(not(unix))]
        assert_eq!(rs_is_unix(), 0);
    }

    #[test]
    fn test_default_tmpdir() {
        let tmpdir = rs_default_tmpdir();
        #[cfg(unix)]
        assert!(!tmpdir.is_null());
        #[cfg(not(unix))]
        assert!(tmpdir.is_null());
    }
}
