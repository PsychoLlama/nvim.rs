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

/// Get substring by byte positions (strpart).
///
/// VimL: `strpart(str, start [, len [, chars]])`
/// - If `start` < 0, treat as 0
/// - If `len` < 0, return empty string
/// - If `start` + `len` > string length, return to end
pub fn strpart(s: &[u8], start: i64, len: Option<i64>) -> &[u8] {
    if s.is_empty() {
        return &[];
    }

    let start_idx = if start < 0 { 0 } else { start as usize };
    if start_idx >= s.len() {
        return &[];
    }

    match len {
        Some(l) if l <= 0 => &[],
        Some(l) => {
            let end_idx = start_idx.saturating_add(l as usize).min(s.len());
            &s[start_idx..end_idx]
        }
        None => &s[start_idx..],
    }
}

/// Get substring by character positions (strcharpart).
///
/// VimL: `strcharpart(str, start [, len [, skipcc]])`
/// Works with UTF-8 character positions rather than byte positions.
pub fn strcharpart(s: &[u8], start: i64, len: Option<i64>) -> Vec<u8> {
    if s.is_empty() {
        return Vec::new();
    }

    let start_idx = if start < 0 { 0 } else { start as usize };

    // Collect character start byte positions
    let mut char_starts: Vec<usize> = Vec::new();
    for (i, &b) in s.iter().enumerate() {
        // Each byte that is NOT a continuation byte starts a new character
        if (b & 0xC0) != 0x80 {
            char_starts.push(i);
        }
    }

    if start_idx >= char_starts.len() {
        return Vec::new();
    }

    let start_byte = char_starts[start_idx];

    match len {
        Some(l) if l <= 0 => Vec::new(),
        Some(l) => {
            let end_char_idx = start_idx + l as usize;
            let end_byte = if end_char_idx >= char_starts.len() {
                s.len()
            } else {
                char_starts[end_char_idx]
            };
            s[start_byte..end_byte].to_vec()
        }
        None => s[start_byte..].to_vec(),
    }
}

/// Translate characters (tr).
///
/// VimL: `tr(str, from, to)`
/// Replace each character in `from` with corresponding character in `to`.
pub fn tr(s: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    if from.is_empty() || to.is_empty() {
        return s.to_vec();
    }

    s.iter()
        .map(|&c| {
            from.iter()
                .position(|&f| f == c)
                .and_then(|idx| to.get(idx).copied())
                .unwrap_or(c)
        })
        .collect()
}

/// Split string by delimiter.
///
/// VimL: `split(str [, pattern [, keepempty]])`
/// Returns byte ranges of split parts.
pub fn split_by_delim(s: &[u8], delim: &[u8], keepempty: bool) -> Vec<(usize, usize)> {
    if s.is_empty() {
        return if keepempty { vec![(0, 0)] } else { vec![] };
    }

    if delim.is_empty() {
        // Split on whitespace
        let mut result = Vec::new();
        let mut in_word = false;
        let mut word_start = 0;

        for (i, &c) in s.iter().enumerate() {
            if c.is_ascii_whitespace() {
                if in_word {
                    result.push((word_start, i));
                    in_word = false;
                } else if keepempty && i == 0 {
                    result.push((0, 0));
                }
            } else if !in_word {
                word_start = i;
                in_word = true;
            }
        }

        if in_word {
            result.push((word_start, s.len()));
        }

        return result;
    }

    let mut result = Vec::new();
    let mut pos = 0;

    while pos <= s.len() {
        // Find next delimiter
        let next_delim = if pos + delim.len() <= s.len() {
            s[pos..]
                .windows(delim.len())
                .position(|w| w == delim)
                .map(|p| pos + p)
        } else {
            None
        };

        if let Some(delim_pos) = next_delim {
            if keepempty || delim_pos > pos {
                result.push((pos, delim_pos));
            }
            pos = delim_pos + delim.len();
        } else {
            if keepempty || pos < s.len() {
                result.push((pos, s.len()));
            }
            break;
        }
    }

    result
}

/// Join strings with separator.
///
/// VimL: `join(list [, sep])`
pub fn join_with_sep(parts: &[&[u8]], sep: &[u8]) -> Vec<u8> {
    if parts.is_empty() {
        return Vec::new();
    }

    let total_len: usize =
        parts.iter().map(|p| p.len()).sum::<usize>() + sep.len() * parts.len().saturating_sub(1);

    let mut result = Vec::with_capacity(total_len);

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.extend_from_slice(sep);
        }
        result.extend_from_slice(part);
    }

    result
}

/// Escape characters in string.
///
/// VimL: `escape(str, chars)`
pub fn escape(s: &[u8], chars: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len() * 2);

    for &c in s {
        if chars.contains(&c) {
            result.push(b'\\');
        }
        result.push(c);
    }

    result
}

/// Shellescape string.
///
/// VimL: `shellescape(str [, special])`
/// Escape string for use in shell command.
pub fn shellescape(s: &[u8], special: bool) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len() + 2);

    result.push(b'\'');
    for &c in s {
        if c == b'\'' {
            // End quote, add escaped quote, restart quote
            result.extend_from_slice(b"'\\''");
        } else if special && (c == b'!' || c == b'%' || c == b'#' || c == b'<') {
            // Escape special characters for :! commands
            result.push(b'\\');
            result.push(c);
        } else {
            result.push(c);
        }
    }
    result.push(b'\'');

    result
}

/// Character to number conversion.
///
/// VimL: `char2nr(str [, utf8])`
pub fn char2nr(s: &[u8]) -> u32 {
    if s.is_empty() {
        return 0;
    }

    // Decode first UTF-8 character
    let first = s[0];
    if first < 0x80 {
        return u32::from(first);
    }

    // Multi-byte UTF-8
    let (len, mask): (usize, u8) = if first & 0xE0 == 0xC0 {
        (2, 0x1F)
    } else if first & 0xF0 == 0xE0 {
        (3, 0x0F)
    } else if first & 0xF8 == 0xF0 {
        (4, 0x07)
    } else {
        return u32::from(first); // Invalid UTF-8
    };

    if s.len() < len {
        return u32::from(first);
    }

    let mut codepoint = u32::from(first & mask);
    for byte in s.iter().take(len).skip(1) {
        if (byte & 0xC0) != 0x80 {
            return u32::from(first); // Invalid continuation
        }
        codepoint = (codepoint << 6) | u32::from(byte & 0x3F);
    }

    codepoint
}

/// Number to character conversion.
///
/// VimL: `nr2char(nr [, utf8])`
pub fn nr2char(nr: u32) -> Vec<u8> {
    if nr < 0x80 {
        vec![nr as u8]
    } else if nr < 0x800 {
        vec![0xC0 | ((nr >> 6) as u8), 0x80 | ((nr & 0x3F) as u8)]
    } else if nr < 0x10000 {
        vec![
            0xE0 | ((nr >> 12) as u8),
            0x80 | (((nr >> 6) & 0x3F) as u8),
            0x80 | ((nr & 0x3F) as u8),
        ]
    } else if nr <= 0x10_FFFF {
        vec![
            0xF0 | ((nr >> 18) as u8),
            0x80 | (((nr >> 12) & 0x3F) as u8),
            0x80 | (((nr >> 6) & 0x3F) as u8),
            0x80 | ((nr & 0x3F) as u8),
        ]
    } else {
        vec![] // Invalid codepoint
    }
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

/// FFI export: strpart - get substring by byte positions.
///
/// # Safety
/// - `s` must be valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_strpart(
    s: *const u8,
    s_len: c_int,
    start: i64,
    len: i64,
    has_len: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || s_len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s, s_len as usize);
    let len_opt = if has_len != 0 { Some(len) } else { None };
    let result = strpart(slice, start, len_opt);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: strcharpart - get substring by character positions.
///
/// # Safety
/// - `s` must be valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_strcharpart(
    s: *const u8,
    s_len: c_int,
    start: i64,
    len: i64,
    has_len: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || s_len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s, s_len as usize);
    let len_opt = if has_len != 0 { Some(len) } else { None };
    let result = strcharpart(slice, start, len_opt);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: tr - translate characters.
///
/// # Safety
/// - All pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tr(
    s: *const u8,
    s_len: c_int,
    from: *const u8,
    from_len: c_int,
    to: *const u8,
    to_len: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || s_len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let s_slice = std::slice::from_raw_parts(s, s_len as usize);
    let from_slice = if from.is_null() || from_len < 0 {
        &[]
    } else {
        std::slice::from_raw_parts(from, from_len as usize)
    };
    let to_slice = if to.is_null() || to_len < 0 {
        &[]
    } else {
        std::slice::from_raw_parts(to, to_len as usize)
    };

    let result = tr(s_slice, from_slice, to_slice);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: tolower - convert to lowercase.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tolower(
    s: *const u8,
    len: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s, len as usize);
    let result = to_lower_ascii(slice);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: toupper - convert to uppercase.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_toupper(
    s: *const u8,
    len: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s, len as usize);
    let result = to_upper_ascii(slice);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: trim - trim whitespace.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_trim(
    s: *const u8,
    len: c_int,
    mode: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s, len as usize);
    let result = match mode {
        1 => trim_start(slice),
        2 => trim_end(slice),
        _ => trim(slice),
    };

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: escape - escape characters.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_escape(
    s: *const u8,
    s_len: c_int,
    chars: *const u8,
    chars_len: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || s_len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let s_slice = std::slice::from_raw_parts(s, s_len as usize);
    let chars_slice = if chars.is_null() || chars_len < 0 {
        &[]
    } else {
        std::slice::from_raw_parts(chars, chars_len as usize)
    };

    let result = escape(s_slice, chars_slice);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: shellescape - escape string for shell.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_shellescape(
    s: *const u8,
    len: c_int,
    special: c_int,
    out: *mut u8,
    out_capacity: c_int,
) -> c_int {
    if s.is_null() || len < 0 || out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s, len as usize);
    let result = shellescape(slice, special != 0);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: char2nr - character to number.
///
/// # Safety
/// - `s` must be valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_char2nr(s: *const u8, len: c_int) -> u32 {
    if s.is_null() || len <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s, len as usize);
    char2nr(slice)
}

/// FFI export: nr2char - number to character.
///
/// # Safety
/// - `out` must be valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_f_nr2char(nr: u32, out: *mut u8, out_capacity: c_int) -> c_int {
    if out.is_null() || out_capacity <= 0 {
        return 0;
    }

    let result = nr2char(nr);

    let copy_len = result.len().min(out_capacity as usize);
    std::ptr::copy_nonoverlapping(result.as_ptr(), out, copy_len);

    copy_len as c_int
}

/// FFI export: split count - count parts in split result.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_split_count(
    s: *const u8,
    s_len: c_int,
    delim: *const u8,
    delim_len: c_int,
    keepempty: c_int,
) -> c_int {
    if s.is_null() || s_len < 0 {
        return 0;
    }

    let s_slice = std::slice::from_raw_parts(s, s_len as usize);
    let delim_slice = if delim.is_null() || delim_len < 0 {
        &[]
    } else {
        std::slice::from_raw_parts(delim, delim_len as usize)
    };

    split_by_delim(s_slice, delim_slice, keepempty != 0).len() as c_int
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

    #[test]
    fn test_strpart() {
        assert_eq!(strpart(b"hello world", 0, Some(5)), b"hello");
        assert_eq!(strpart(b"hello world", 6, Some(5)), b"world");
        assert_eq!(strpart(b"hello world", 6, None), b"world");
        assert_eq!(strpart(b"hello", -5, Some(5)), b"hello"); // Negative start treated as 0
        assert_eq!(strpart(b"hello", 0, Some(0)), b""); // Zero length
        assert_eq!(strpart(b"hello", 10, Some(5)), b""); // Past end
    }

    #[test]
    fn test_strcharpart() {
        // ASCII
        assert_eq!(strcharpart(b"hello world", 0, Some(5)), b"hello");
        assert_eq!(strcharpart(b"hello world", 6, Some(5)), b"world");
        // UTF-8 - "日本語" (3 chars, 9 bytes)
        let jp = "日本語".as_bytes();
        assert_eq!(strcharpart(jp, 0, Some(1)), "日".as_bytes());
        assert_eq!(strcharpart(jp, 1, Some(1)), "本".as_bytes());
        assert_eq!(strcharpart(jp, 0, Some(2)), "日本".as_bytes());
    }

    #[test]
    fn test_tr() {
        assert_eq!(tr(b"hello", b"el", b"ip"), b"hippo");
        assert_eq!(tr(b"hello", b"aeiou", b"AEIOU"), b"hEllO");
        assert_eq!(tr(b"hello", b"", b""), b"hello"); // Empty from/to
    }

    #[test]
    fn test_split_by_delim() {
        let parts = split_by_delim(b"a,b,c", b",", false);
        assert_eq!(parts.len(), 3);
        assert_eq!(&b"a,b,c"[parts[0].0..parts[0].1], b"a");
        assert_eq!(&b"a,b,c"[parts[1].0..parts[1].1], b"b");
        assert_eq!(&b"a,b,c"[parts[2].0..parts[2].1], b"c");

        // Empty delimiter: split on whitespace
        let parts = split_by_delim(b"hello world", b"", false);
        assert_eq!(parts.len(), 2);

        // keepempty
        let parts = split_by_delim(b"a,,b", b",", true);
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_join_with_sep() {
        let parts: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        assert_eq!(join_with_sep(&parts, b","), b"a,b,c");
        assert_eq!(join_with_sep(&parts, b""), b"abc");
        assert_eq!(join_with_sep(&[], b","), b"");
    }

    #[test]
    fn test_escape() {
        assert_eq!(escape(b"hello|world", b"|"), b"hello\\|world");
        assert_eq!(escape(b"a.b*c", b".*"), b"a\\.b\\*c");
    }

    #[test]
    fn test_shellescape() {
        assert_eq!(shellescape(b"hello", false), b"'hello'");
        assert_eq!(shellescape(b"it's", false), b"'it'\\''s'");
    }

    #[test]
    fn test_char2nr() {
        assert_eq!(char2nr(b"A"), 65);
        assert_eq!(char2nr(b""), 0);
        assert_eq!(char2nr("日".as_bytes()), 0x65E5);
    }

    #[test]
    fn test_nr2char() {
        assert_eq!(nr2char(65), b"A");
        assert_eq!(nr2char(0x65E5), "日".as_bytes());
    }
}
