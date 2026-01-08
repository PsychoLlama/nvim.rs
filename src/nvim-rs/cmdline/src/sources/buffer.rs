//! Buffer name completion source
//!
//! This module provides helpers for completing buffer names,
//! including filtering by various buffer properties.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Buffer Flags
// =============================================================================

/// Buffer completion filter flags.
pub mod filter_flags {
    /// Include listed buffers (shown in :ls)
    pub const LISTED: u32 = 0x01;
    /// Include unlisted buffers
    pub const UNLISTED: u32 = 0x02;
    /// Include loaded buffers only
    pub const LOADED: u32 = 0x04;
    /// Include modified buffers only
    pub const MODIFIED: u32 = 0x08;
    /// Include unmodified buffers only
    pub const UNMODIFIED: u32 = 0x10;
    /// Sort by last used time
    pub const SORT_LASTUSED: u32 = 0x20;
    /// Include diff buffers only (for :diffget/:diffput)
    pub const DIFF_ONLY: u32 = 0x40;
    /// Default filter (listed buffers)
    pub const DEFAULT: u32 = LISTED;
}

// =============================================================================
// Buffer Matching
// =============================================================================

/// Buffer matching mode for completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferMatchMode {
    /// Match buffer name from start
    #[default]
    Prefix,
    /// Match anywhere in buffer name
    Substring,
    /// Match buffer number
    Number,
    /// Fuzzy match
    Fuzzy,
}

impl BufferMatchMode {
    /// Convert from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Prefix),
            1 => Some(Self::Substring),
            2 => Some(Self::Number),
            3 => Some(Self::Fuzzy),
            _ => None,
        }
    }
}

/// Check if a buffer name matches a pattern.
///
/// Uses prefix matching by default.
#[must_use]
pub fn buffer_name_matches(name: &[u8], pattern: &[u8], mode: BufferMatchMode) -> bool {
    if pattern.is_empty() {
        return true;
    }

    match mode {
        BufferMatchMode::Prefix => name_starts_with(name, pattern),
        BufferMatchMode::Substring => name_contains(name, pattern),
        // Caller handles number and fuzzy matching
        BufferMatchMode::Number | BufferMatchMode::Fuzzy => true,
    }
}

/// Check if name starts with pattern (case-insensitive on Windows).
#[must_use]
fn name_starts_with(name: &[u8], pattern: &[u8]) -> bool {
    if name.len() < pattern.len() {
        return false;
    }

    #[cfg(windows)]
    {
        name[..pattern.len()]
            .iter()
            .zip(pattern.iter())
            .all(|(n, p)| n.to_ascii_lowercase() == p.to_ascii_lowercase())
    }
    #[cfg(not(windows))]
    {
        &name[..pattern.len()] == pattern
    }
}

/// Check if name contains pattern (case-insensitive on Windows).
#[must_use]
fn name_contains(name: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return true;
    }

    if name.len() < pattern.len() {
        return false;
    }

    #[cfg(windows)]
    {
        let pattern_lower: Vec<u8> = pattern.iter().map(|c| c.to_ascii_lowercase()).collect();
        name.windows(pattern.len()).any(|window| {
            window
                .iter()
                .zip(pattern_lower.iter())
                .all(|(n, p)| n.to_ascii_lowercase() == *p)
        })
    }
    #[cfg(not(windows))]
    {
        name.windows(pattern.len()).any(|window| window == pattern)
    }
}

// =============================================================================
// Buffer Name Formatting
// =============================================================================

/// Get the display name for a buffer in completion.
///
/// Shortens paths using ~ for home directory and . for current directory.
#[must_use]
pub fn should_shorten_path(full_path: &[u8], pattern: &[u8]) -> bool {
    // Don't shorten if pattern contains path separators
    for &c in pattern {
        if c == b'/' || c == b'\\' {
            return false;
        }
    }

    // Shorten if path is absolute
    #[cfg(not(windows))]
    {
        full_path.first() == Some(&b'/')
    }
    #[cfg(windows)]
    {
        full_path.len() >= 2
            && (full_path[0].is_ascii_alphabetic() && full_path[1] == b':' || full_path[0] == b'\\')
    }
}

/// Extract just the filename from a buffer path for display.
#[must_use]
pub fn extract_filename(path: &[u8]) -> &[u8] {
    for (i, &c) in path.iter().enumerate().rev() {
        if c == b'/' || c == b'\\' {
            return &path[i + 1..];
        }
    }
    path
}

// =============================================================================
// Buffer Number Parsing
// =============================================================================

/// Parse a buffer number from a string.
///
/// Returns `Some(num)` if the string is a valid buffer number, `None` otherwise.
#[must_use]
pub fn parse_buffer_number(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }

    // Check for # prefix (alternate buffer)
    if s[0] == b'#' {
        if s.len() == 1 {
            return Some(0); // Special: alternate buffer
        }
        return parse_buffer_number(&s[1..]);
    }

    // Check for % prefix (current buffer)
    if s[0] == b'%' {
        return Some(0); // Special: current buffer
    }

    // Parse as number
    let mut num: u32 = 0;
    for &c in s {
        if !c.is_ascii_digit() {
            return None;
        }
        num = num.checked_mul(10)?.checked_add(u32::from(c - b'0'))?;
    }

    Some(num)
}

/// Check if a string looks like a buffer number reference.
#[must_use]
pub const fn is_buffer_number_ref(s: &[u8]) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s[0];
    first == b'#' || first == b'%' || first.is_ascii_digit()
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Check if a buffer name matches a pattern (FFI).
///
/// # Safety
///
/// `name` and `pattern` must be valid pointers to strings of at least
/// `name_len` and `pattern_len` bytes respectively.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buffer_name_matches(
    name: *const c_char,
    name_len: usize,
    pattern: *const c_char,
    pattern_len: usize,
    mode: c_int,
) -> c_int {
    if name.is_null() {
        return 0;
    }

    let name_bytes = std::slice::from_raw_parts(name.cast::<u8>(), name_len);
    let pattern_bytes = if pattern.is_null() || pattern_len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(pattern.cast::<u8>(), pattern_len)
    };

    let match_mode = BufferMatchMode::from_raw(mode).unwrap_or_default();
    c_int::from(buffer_name_matches(name_bytes, pattern_bytes, match_mode))
}

/// Parse a buffer number from string (FFI).
///
/// Returns -1 if not a valid buffer number.
///
/// # Safety
///
/// `s` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_parse_buffer_number(s: *const c_char, len: usize) -> c_int {
    if s.is_null() || len == 0 {
        return -1;
    }

    let bytes = std::slice::from_raw_parts(s.cast::<u8>(), len);
    parse_buffer_number(bytes).map_or(-1, |n| n as c_int)
}

/// Check if a string looks like a buffer number reference (FFI).
///
/// # Safety
///
/// `s` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_buffer_number_ref(s: *const c_char, len: usize) -> c_int {
    if s.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(s.cast::<u8>(), len);
    c_int::from(is_buffer_number_ref(bytes))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_name_matches_prefix() {
        assert!(buffer_name_matches(
            b"file.txt",
            b"",
            BufferMatchMode::Prefix
        ));
        assert!(buffer_name_matches(
            b"file.txt",
            b"f",
            BufferMatchMode::Prefix
        ));
        assert!(buffer_name_matches(
            b"file.txt",
            b"file",
            BufferMatchMode::Prefix
        ));
        assert!(buffer_name_matches(
            b"file.txt",
            b"file.txt",
            BufferMatchMode::Prefix
        ));
        assert!(!buffer_name_matches(
            b"file.txt",
            b"txt",
            BufferMatchMode::Prefix
        ));
    }

    #[test]
    fn test_buffer_name_matches_substring() {
        assert!(buffer_name_matches(
            b"file.txt",
            b"ile",
            BufferMatchMode::Substring
        ));
        assert!(buffer_name_matches(
            b"file.txt",
            b"txt",
            BufferMatchMode::Substring
        ));
        assert!(buffer_name_matches(
            b"file.txt",
            b".t",
            BufferMatchMode::Substring
        ));
        assert!(!buffer_name_matches(
            b"file.txt",
            b"xyz",
            BufferMatchMode::Substring
        ));
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename(b"/home/user/file.txt"), b"file.txt");
        assert_eq!(extract_filename(b"file.txt"), b"file.txt");
        assert_eq!(extract_filename(b"dir/file.txt"), b"file.txt");

        #[cfg(windows)]
        {
            assert_eq!(extract_filename(b"C:\\Users\\file.txt"), b"file.txt");
        }
    }

    #[test]
    fn test_parse_buffer_number() {
        assert_eq!(parse_buffer_number(b"123"), Some(123));
        assert_eq!(parse_buffer_number(b"0"), Some(0));
        assert_eq!(parse_buffer_number(b"#"), Some(0));
        assert_eq!(parse_buffer_number(b"#5"), Some(5));
        assert_eq!(parse_buffer_number(b"%"), Some(0));
        assert_eq!(parse_buffer_number(b""), None);
        assert_eq!(parse_buffer_number(b"abc"), None);
        assert_eq!(parse_buffer_number(b"12a"), None);
    }

    #[test]
    fn test_is_buffer_number_ref() {
        assert!(is_buffer_number_ref(b"123"));
        assert!(is_buffer_number_ref(b"#"));
        assert!(is_buffer_number_ref(b"#5"));
        assert!(is_buffer_number_ref(b"%"));
        assert!(!is_buffer_number_ref(b""));
        assert!(!is_buffer_number_ref(b"abc"));
    }

    #[test]
    fn test_should_shorten_path() {
        #[cfg(not(windows))]
        {
            assert!(should_shorten_path(b"/home/user/file.txt", b""));
            assert!(should_shorten_path(b"/home/user/file.txt", b"file"));
            assert!(!should_shorten_path(b"/home/user/file.txt", b"home/user"));
        }
    }
}
