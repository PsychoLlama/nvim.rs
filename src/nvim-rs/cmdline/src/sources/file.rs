//! File and directory completion source
//!
//! This module provides helpers for completing file paths, directories,
//! and shell commands.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Path Constants
// =============================================================================

/// Path separator character (platform-specific)
#[cfg(windows)]
pub const PATH_SEP: u8 = b'\\';
#[cfg(not(windows))]
pub const PATH_SEP: u8 = b'/';

/// Alternative path separator for Windows
#[cfg(windows)]
pub const ALT_PATH_SEP: Option<u8> = Some(b'/');
#[cfg(not(windows))]
pub const ALT_PATH_SEP: Option<u8> = None;

// =============================================================================
// Path Utilities
// =============================================================================

/// Check if a character is a path separator.
#[must_use]
pub const fn is_path_sep(c: u8) -> bool {
    if c == PATH_SEP {
        return true;
    }
    if let Some(alt) = ALT_PATH_SEP {
        if c == alt {
            return true;
        }
    }
    false
}

/// Find the tail (filename) portion of a path.
///
/// Returns the index after the last path separator, or 0 if none found.
#[must_use]
pub fn path_tail_index(path: &[u8]) -> usize {
    for (i, &c) in path.iter().enumerate().rev() {
        if is_path_sep(c) {
            return i + 1;
        }
    }
    0
}

/// Get the directory portion of a path.
///
/// Returns everything up to and including the last path separator.
#[must_use]
pub fn path_head(path: &[u8]) -> &[u8] {
    let tail = path_tail_index(path);
    &path[..tail]
}

/// Get the filename portion of a path.
#[must_use]
pub fn path_tail(path: &[u8]) -> &[u8] {
    let tail = path_tail_index(path);
    &path[tail..]
}

/// Check if a path is absolute.
#[must_use]
pub fn is_absolute_path(path: &[u8]) -> bool {
    if path.is_empty() {
        return false;
    }

    #[cfg(not(windows))]
    {
        path[0] == b'/'
    }

    #[cfg(windows)]
    {
        // Drive letter (C:) or UNC path (\\)
        if path.len() >= 2 {
            if path[0].is_ascii_alphabetic() && path[1] == b':' {
                return true;
            }
            if path[0] == b'\\' && path[1] == b'\\' {
                return true;
            }
        }
        path[0] == b'\\' || path[0] == b'/'
    }
}

/// Check if a path starts with home directory marker (~).
#[must_use]
pub const fn starts_with_home(path: &[u8]) -> bool {
    !path.is_empty() && path[0] == b'~'
}

/// Check if a path ends with a path separator.
#[must_use]
pub fn ends_with_sep(path: &[u8]) -> bool {
    !path.is_empty() && is_path_sep(path[path.len() - 1])
}

// =============================================================================
// File Pattern Utilities
// =============================================================================

/// Check if a pattern contains wildcards.
#[must_use]
pub fn has_wildcards(pattern: &[u8]) -> bool {
    for &c in pattern {
        match c {
            b'*' | b'?' | b'[' => return true,
            _ => {}
        }
    }
    false
}

/// Check if a pattern contains special expansion characters.
///
/// These characters trigger special handling:
/// - `$` - environment variable
/// - `` ` `` - command substitution
/// - `~` - home directory
#[must_use]
pub fn has_special_chars(pattern: &[u8]) -> bool {
    for &c in pattern {
        match c {
            b'$' | b'`' | b'~' => return true,
            _ => {}
        }
    }
    false
}

/// Count unescaped wildcards in a pattern.
#[must_use]
pub fn count_wildcards(pattern: &[u8]) -> usize {
    let mut count = 0;
    let mut escaped = false;

    for &c in pattern {
        if escaped {
            escaped = false;
            continue;
        }

        if c == b'\\' {
            escaped = true;
            continue;
        }

        if matches!(c, b'*' | b'?' | b'[') {
            count += 1;
        }
    }

    count
}

// =============================================================================
// File Completion Behavior
// =============================================================================

/// Determine if a trailing slash should be added to a directory match.
///
/// Slash is added when:
/// - The match is a directory
/// - The pattern doesn't already end with a separator
/// - Not in certain modes that suppress slash addition
#[must_use]
pub const fn should_add_trailing_slash(is_dir: bool, pattern_ends_sep: bool) -> bool {
    is_dir && !pattern_ends_sep
}

/// Check if file completion should be case-sensitive.
///
/// Case sensitivity is platform-dependent by default.
#[must_use]
pub const fn file_completion_case_sensitive() -> bool {
    #[cfg(windows)]
    {
        false // Windows is case-insensitive
    }
    #[cfg(not(windows))]
    {
        true // Unix is case-sensitive
    }
}

/// Normalize a path for comparison.
///
/// On Windows, converts backslashes to forward slashes and lowercases.
#[must_use]
pub fn normalize_for_compare(path: &[u8]) -> Vec<u8> {
    #[cfg(windows)]
    {
        path.iter()
            .map(|&c| {
                if c == b'\\' {
                    b'/'
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        path.to_vec()
    }
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Check if a character is a path separator (FFI).
#[unsafe(no_mangle)]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_is_path_sep(c: c_int) -> c_int {
    if !(0..=255).contains(&c) {
        return 0;
    }
    c_int::from(is_path_sep(c as u8))
}

/// Get the path tail index (FFI).
///
/// # Safety
///
/// `path` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_path_tail_index(path: *const c_char, len: usize) -> usize {
    if path.is_null() {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(path.cast::<u8>(), len);
    path_tail_index(bytes)
}

/// Check if a path is absolute (FFI).
///
/// # Safety
///
/// `path` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_absolute_path(path: *const c_char, len: usize) -> c_int {
    if path.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(path.cast::<u8>(), len);
    c_int::from(is_absolute_path(bytes))
}

/// Check if a pattern has wildcards (FFI).
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_has_wildcards(pattern: *const c_char, len: usize) -> c_int {
    if pattern.is_null() {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    c_int::from(has_wildcards(bytes))
}

/// Check if file completion is case-sensitive (FFI).
#[unsafe(no_mangle)]
pub extern "C" fn rs_file_completion_case_sensitive() -> c_int {
    c_int::from(file_completion_case_sensitive())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_path_sep() {
        #[cfg(not(windows))]
        {
            assert!(is_path_sep(b'/'));
            assert!(!is_path_sep(b'\\'));
        }

        #[cfg(windows)]
        {
            assert!(is_path_sep(b'/'));
            assert!(is_path_sep(b'\\'));
        }

        assert!(!is_path_sep(b'a'));
        assert!(!is_path_sep(b' '));
    }

    #[test]
    fn test_path_tail_index() {
        #[cfg(not(windows))]
        {
            assert_eq!(path_tail_index(b"/home/user/file.txt"), 11);
            assert_eq!(path_tail_index(b"/file.txt"), 1);
            assert_eq!(path_tail_index(b"file.txt"), 0);
            assert_eq!(path_tail_index(b""), 0);
        }
    }

    #[test]
    fn test_path_head_tail() {
        #[cfg(not(windows))]
        {
            assert_eq!(path_head(b"/home/user/file.txt"), b"/home/user/");
            assert_eq!(path_tail(b"/home/user/file.txt"), b"file.txt");
            assert_eq!(path_head(b"file.txt"), b"");
            assert_eq!(path_tail(b"file.txt"), b"file.txt");
        }
    }

    #[test]
    fn test_is_absolute_path() {
        #[cfg(not(windows))]
        {
            assert!(is_absolute_path(b"/home"));
            assert!(is_absolute_path(b"/"));
            assert!(!is_absolute_path(b"home"));
            assert!(!is_absolute_path(b"./home"));
            assert!(!is_absolute_path(b""));
        }

        #[cfg(windows)]
        {
            assert!(is_absolute_path(b"C:\\"));
            assert!(is_absolute_path(b"C:/"));
            assert!(is_absolute_path(b"\\\\server"));
            assert!(!is_absolute_path(b"relative"));
        }
    }

    #[test]
    fn test_starts_with_home() {
        assert!(starts_with_home(b"~"));
        assert!(starts_with_home(b"~/"));
        assert!(starts_with_home(b"~user"));
        assert!(!starts_with_home(b"/home"));
        assert!(!starts_with_home(b""));
    }

    #[test]
    fn test_has_wildcards() {
        assert!(has_wildcards(b"*.txt"));
        assert!(has_wildcards(b"file?"));
        assert!(has_wildcards(b"[abc]"));
        assert!(!has_wildcards(b"file.txt"));
        assert!(!has_wildcards(b""));
    }

    #[test]
    fn test_has_special_chars() {
        assert!(has_special_chars(b"$HOME"));
        assert!(has_special_chars(b"`cmd`"));
        assert!(has_special_chars(b"~"));
        assert!(!has_special_chars(b"file.txt"));
    }

    #[test]
    fn test_count_wildcards() {
        assert_eq!(count_wildcards(b"*.txt"), 1);
        assert_eq!(count_wildcards(b"*.*"), 2);
        assert_eq!(count_wildcards(b"file?[abc]*"), 3);
        assert_eq!(count_wildcards(b"\\*.txt"), 0); // escaped
        assert_eq!(count_wildcards(b"file.txt"), 0);
    }

    #[test]
    fn test_should_add_trailing_slash() {
        assert!(should_add_trailing_slash(true, false));
        assert!(!should_add_trailing_slash(true, true));
        assert!(!should_add_trailing_slash(false, false));
    }

    #[test]
    fn test_ends_with_sep() {
        #[cfg(not(windows))]
        {
            assert!(ends_with_sep(b"/home/"));
            assert!(!ends_with_sep(b"/home"));
            assert!(!ends_with_sep(b""));
        }
    }
}
