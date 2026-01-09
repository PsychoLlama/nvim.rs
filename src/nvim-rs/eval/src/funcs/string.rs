//! String manipulation functions for VimL.
//!
//! This module implements string functions from `src/nvim/eval/funcs.c`:
//! - `strlen()` - string length in bytes
//! - `strchars()` - string length in characters
//! - `stridx()` - find substring index
//! - `strridx()` - find last substring index
//! - `tolower()` - convert to lowercase (ASCII)
//! - `toupper()` - convert to uppercase (ASCII)
//! - `trim()` - trim whitespace

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// Pure String Functions (No FFI)
// =============================================================================

/// Get length of string in bytes.
pub fn strlen_bytes(s: &[u8]) -> usize {
    s.len()
}

/// Get length of string in UTF-8 characters.
pub fn strlen_chars(s: &[u8]) -> usize {
    // Count UTF-8 code points
    s.iter().filter(|&&b| (b & 0xC0) != 0x80).count()
}

/// Find first occurrence of needle in haystack.
/// Returns byte index or None if not found.
pub fn str_index(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > haystack.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Find last occurrence of needle in haystack.
/// Returns byte index or None if not found.
pub fn str_rindex(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(haystack.len());
    }
    if needle.len() > haystack.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

/// Convert ASCII characters to lowercase.
pub fn to_lower_ascii(s: &[u8]) -> Vec<u8> {
    s.iter().map(u8::to_ascii_lowercase).collect()
}

/// Convert ASCII characters to uppercase.
pub fn to_upper_ascii(s: &[u8]) -> Vec<u8> {
    s.iter().map(u8::to_ascii_uppercase).collect()
}

/// Trim leading and trailing whitespace.
pub fn trim(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|&c| !c.is_ascii_whitespace());
    let end = s.iter().rposition(|&c| !c.is_ascii_whitespace());

    match (start, end) {
        (Some(start_idx), Some(end_idx)) => &s[start_idx..=end_idx],
        _ => &[],
    }
}

/// Trim leading whitespace.
pub fn trim_start(s: &[u8]) -> &[u8] {
    match s.iter().position(|&c| !c.is_ascii_whitespace()) {
        Some(start) => &s[start..],
        None => &[],
    }
}

/// Trim trailing whitespace.
pub fn trim_end(s: &[u8]) -> &[u8] {
    match s.iter().rposition(|&c| !c.is_ascii_whitespace()) {
        Some(end) => &s[..=end],
        None => &[],
    }
}

/// Repeat string n times.
pub fn str_repeat(s: &[u8], n: usize) -> Vec<u8> {
    s.repeat(n)
}

/// Reverse string (byte-wise).
pub fn str_reverse(s: &[u8]) -> Vec<u8> {
    s.iter().copied().rev().collect()
}

/// Check if string starts with prefix.
pub fn str_starts_with(s: &[u8], prefix: &[u8]) -> bool {
    s.starts_with(prefix)
}

/// Check if string ends with suffix.
pub fn str_ends_with(s: &[u8], suffix: &[u8]) -> bool {
    s.ends_with(suffix)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: string length in bytes.
///
/// # Safety
/// - `s` must be a valid pointer to at least `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_strlen_bytes(s: *const u8, len: c_int) -> c_int {
    if s.is_null() || len < 0 {
        return 0;
    }
    len // VimL strlen() returns byte length, which we already have
}

/// FFI export: string length in UTF-8 characters.
///
/// # Safety
/// - `s` must be a valid pointer to at least `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_strchars(s: *const u8, len: c_int) -> c_int {
    if s.is_null() || len < 0 {
        return 0;
    }

    // SAFETY: Caller guarantees s points to at least len bytes
    let slice = unsafe { std::slice::from_raw_parts(s, len as usize) };
    strlen_chars(slice) as c_int
}

/// FFI export: find substring (stridx).
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_stridx(
    haystack: *const u8,
    haystack_len: c_int,
    needle: *const u8,
    needle_len: c_int,
) -> c_int {
    if haystack.is_null() || haystack_len < 0 {
        return -1;
    }
    if needle.is_null() || needle_len < 0 {
        return 0; // Empty needle found at start
    }

    // SAFETY: Caller guarantees valid pointers
    let h = unsafe { std::slice::from_raw_parts(haystack, haystack_len as usize) };
    let n = unsafe { std::slice::from_raw_parts(needle, needle_len as usize) };

    str_index(h, n).map_or(-1, |i| i as c_int)
}

/// FFI export: find last substring (strridx).
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_strridx(
    haystack: *const u8,
    haystack_len: c_int,
    needle: *const u8,
    needle_len: c_int,
) -> c_int {
    if haystack.is_null() || haystack_len < 0 {
        return -1;
    }
    if needle.is_null() || needle_len < 0 {
        return haystack_len; // Empty needle found at end
    }

    // SAFETY: Caller guarantees valid pointers
    let h = unsafe { std::slice::from_raw_parts(haystack, haystack_len as usize) };
    let n = unsafe { std::slice::from_raw_parts(needle, needle_len as usize) };

    str_rindex(h, n).map_or(-1, |i| i as c_int)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strlen_bytes() {
        assert_eq!(strlen_bytes(b"hello"), 5);
        assert_eq!(strlen_bytes(b""), 0);
        assert_eq!(strlen_bytes("日本語".as_bytes()), 9); // 3 chars, 3 bytes each
    }

    #[test]
    fn test_strlen_chars() {
        assert_eq!(strlen_chars(b"hello"), 5);
        assert_eq!(strlen_chars(b""), 0);
        assert_eq!(strlen_chars("日本語".as_bytes()), 3); // 3 characters
    }

    #[test]
    fn test_str_index() {
        assert_eq!(str_index(b"hello world", b"world"), Some(6));
        assert_eq!(str_index(b"hello world", b"x"), None);
        assert_eq!(str_index(b"hello world", b""), Some(0));
        assert_eq!(str_index(b"aaa", b"aa"), Some(0));
    }

    #[test]
    fn test_str_rindex() {
        assert_eq!(str_rindex(b"hello world world", b"world"), Some(12));
        assert_eq!(str_rindex(b"hello", b"x"), None);
        assert_eq!(str_rindex(b"aaa", b"aa"), Some(1));
    }

    #[test]
    fn test_to_lower_ascii() {
        assert_eq!(to_lower_ascii(b"Hello WORLD"), b"hello world");
        assert_eq!(to_lower_ascii(b"123"), b"123");
    }

    #[test]
    fn test_to_upper_ascii() {
        assert_eq!(to_upper_ascii(b"Hello world"), b"HELLO WORLD");
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim_start(b"  hello"), b"hello");
        assert_eq!(trim_end(b"hello  "), b"hello");
    }

    #[test]
    fn test_str_repeat() {
        assert_eq!(str_repeat(b"ab", 3), b"ababab");
        assert_eq!(str_repeat(b"x", 0), b"");
    }

    #[test]
    fn test_str_reverse() {
        assert_eq!(str_reverse(b"hello"), b"olleh");
    }

    #[test]
    fn test_str_starts_ends() {
        assert!(str_starts_with(b"hello world", b"hello"));
        assert!(!str_starts_with(b"hello world", b"world"));
        assert!(str_ends_with(b"hello world", b"world"));
        assert!(!str_ends_with(b"hello world", b"hello"));
    }
}
