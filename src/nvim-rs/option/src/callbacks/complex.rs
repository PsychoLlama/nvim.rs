//! Complex string option callback implementations
//!
//! This module contains Rust implementations of complex string option validation
//! callbacks. These callbacks have more complex parsing logic or require
//! integration with C code for side effects.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

use std::ffi::{c_char, c_int};

use super::{callback_ok, CallbackResult};

// =============================================================================
// Error Messages
// =============================================================================

/// Error: Invalid argument
const E_INVARG: *const c_char = c"E474: Invalid argument".as_ptr();

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if bytes at pointer match expected bytes.
#[inline]
unsafe fn bytes_match(ptr: *const c_char, expected: &[u8]) -> bool {
    for (i, &expected_byte) in expected.iter().enumerate() {
        if *ptr.add(i) as u8 != expected_byte {
            return false;
        }
    }
    true
}

// =============================================================================
// 'helplang' Option Validation
// =============================================================================

/// Validate 'helplang' option value.
/// Format: "", "ab", "ab,cd", etc. (two-letter language codes)
#[no_mangle]
pub unsafe extern "C" fn rs_validate_helplang(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    // Empty string is valid
    if *value == 0 {
        return callback_ok();
    }

    // Check for "ab", "ab,cd", etc.
    let mut s = value;
    loop {
        // Must have at least two characters
        if *s == 0 || *s.add(1) == 0 {
            return E_INVARG;
        }

        // After two characters, must have comma or end
        let third = *s.add(2) as u8;
        if third == 0 {
            // End of string - valid
            break;
        }
        if third != b',' {
            return E_INVARG;
        }

        // After comma, must have more content
        if *s.add(3) == 0 {
            return E_INVARG;
        }

        // Move to next language code
        s = s.add(3);
    }

    callback_ok()
}

// =============================================================================
// 'shada' Option Validation
// =============================================================================

/// Valid first characters for 'shada' option items
const SHADA_CHARS: &[u8] = b"!\"%':<@cfhnrs";

/// Validate 'shada' option value.
///
/// Returns NULL on success, or an error message on failure.
/// Note: This only validates the syntax. The C side handles the
/// "must specify a ' value" check since it needs `get_shada_parameter()`.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_shada(
    value: *const c_char,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> CallbackResult {
    if value.is_null() || *value == 0 {
        return callback_ok();
    }

    let mut s = value;

    while *s != 0 {
        let ch = *s as u8;

        // Check it's a valid character
        if !SHADA_CHARS.contains(&ch) {
            // Return illegal character error
            if !errbuf.is_null() && errbuflen > 0 {
                write_illegal_char_error(errbuf, errbuflen, ch);
            }
            return if errbuf.is_null() {
                c"".as_ptr()
            } else {
                errbuf
            };
        }

        if ch == b'n' {
            // 'n' (name) is always last one - stop parsing
            break;
        } else if ch == b'r' {
            // skip until next ','
            s = s.add(1);
            while *s != 0 && *s as u8 != b',' {
                s = s.add(1);
            }
        } else if ch == b'%' {
            // optional number
            s = s.add(1);
            while *s != 0 && (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }
        } else if ch == b'!' || ch == b'h' || ch == b'c' {
            // no extra chars
            s = s.add(1);
        } else {
            // must have a number
            s = s.add(1);
            while *s != 0 && (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }

            // Check if we had at least one digit
            let prev = *s.sub(1) as u8;
            if !prev.is_ascii_digit() {
                // E526: Missing number after <x>
                if !errbuf.is_null() && errbuflen > 0 {
                    write_missing_number_error(errbuf, errbuflen, prev);
                }
                return if errbuf.is_null() {
                    c"".as_ptr()
                } else {
                    errbuf
                };
            }
        }

        // Check for comma or end
        if *s as u8 == b',' {
            s = s.add(1);
        } else if *s != 0 {
            // E527: Missing comma
            return c"E527: Missing comma".as_ptr();
        }
    }

    callback_ok()
}

/// Write an illegal character error message to the buffer.
#[allow(clippy::cast_possible_wrap)]
unsafe fn write_illegal_char_error(errbuf: *mut c_char, errbuflen: usize, ch: u8) {
    let msg = b"E langarg: Illegal character: ";
    let msg_len = msg.len().min(errbuflen.saturating_sub(2));

    for (i, &b) in msg.iter().take(msg_len).enumerate() {
        *errbuf.add(i) = b as c_char;
    }

    if msg_len < errbuflen.saturating_sub(1) {
        *errbuf.add(msg_len) = ch as c_char;
        *errbuf.add(msg_len + 1) = 0;
    } else {
        *errbuf.add(msg_len) = 0;
    }
}

/// Write a missing number error message to the buffer.
#[allow(clippy::cast_possible_wrap)]
unsafe fn write_missing_number_error(errbuf: *mut c_char, errbuflen: usize, ch: u8) {
    let msg = b"E526: Missing number after <";
    let msg_len = msg.len().min(errbuflen.saturating_sub(3));

    for (i, &b) in msg.iter().take(msg_len).enumerate() {
        *errbuf.add(i) = b as c_char;
    }

    if msg_len < errbuflen.saturating_sub(2) {
        *errbuf.add(msg_len) = ch as c_char;
        *errbuf.add(msg_len + 1) = b'>' as c_char;
        *errbuf.add(msg_len + 2) = 0;
    } else {
        *errbuf.add(msg_len) = 0;
    }
}

// =============================================================================
// 'mousescroll' Option Validation
// =============================================================================

/// Result from validating 'mousescroll' option.
#[repr(C)]
pub struct MouseScrollResult {
    /// Vertical scroll amount (-1 if not set)
    pub vertical: i64,
    /// Horizontal scroll amount (-1 if not set)
    pub horizontal: i64,
    /// 0 on success, non-zero on error
    pub error: c_int,
}

/// Validate 'mousescroll' option value.
/// Format: "ver:N,hor:N" or "hor:N,ver:N"
#[no_mangle]
pub unsafe extern "C" fn rs_validate_mousescroll(value: *const c_char) -> MouseScrollResult {
    let mut result = MouseScrollResult {
        vertical: -1,
        horizontal: -1,
        error: 0,
    };

    if value.is_null() || *value == 0 {
        result.error = 1;
        return result;
    }

    let mut s = value;

    loop {
        // Find end of this item (comma or null)
        let mut end = s;
        while *end != 0 && *end as u8 != b',' {
            end = end.add(1);
        }

        let len = end.offset_from(s) as usize;

        // Both "ver:" and "hor:" are 4 bytes long
        // They should be followed by at least one digit
        if len <= 4 {
            result.error = 1;
            return result;
        }

        // Check which direction
        let is_ver = *s as u8 == b'v'
            && *s.add(1) as u8 == b'e'
            && *s.add(2) as u8 == b'r'
            && *s.add(3) as u8 == b':';
        let is_hor = *s as u8 == b'h'
            && *s.add(1) as u8 == b'o'
            && *s.add(2) as u8 == b'r'
            && *s.add(3) as u8 == b':';

        if !is_ver && !is_hor {
            result.error = 1;
            return result;
        }

        // Parse the number
        let mut num_start = s.add(4);
        let mut num: i64 = 0;
        let mut has_digit = false;

        while num_start < end {
            let ch = *num_start as u8;
            if !ch.is_ascii_digit() {
                result.error = 1;
                return result;
            }
            has_digit = true;
            num = num * 10 + i64::from(ch - b'0');
            num_start = num_start.add(1);
        }

        if !has_digit {
            result.error = 1;
            return result;
        }

        // Store the value
        if is_ver {
            if result.vertical >= 0 {
                // Already set - duplicate
                result.error = 1;
                return result;
            }
            result.vertical = num;
        } else {
            if result.horizontal >= 0 {
                // Already set - duplicate
                result.error = 1;
                return result;
            }
            result.horizontal = num;
        }

        // Move to next item
        if *end == 0 {
            break;
        }
        s = end.add(1);
    }

    result
}

// =============================================================================
// 'colorcolumn' Option Validation
// =============================================================================

/// Validate 'colorcolumn' option value.
/// Format: comma-separated list of column numbers or +N/-N relative values.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_colorcolumn(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0; // Empty is valid
    }

    let mut s = value;

    while *s != 0 {
        // Skip leading whitespace (though unusual)
        while *s as u8 == b' ' {
            s = s.add(1);
        }

        if *s == 0 {
            break;
        }

        // Check for +/- prefix
        if *s as u8 == b'+' || *s as u8 == b'-' {
            s = s.add(1);
        }

        // Must have at least one digit
        if !(*s as u8).is_ascii_digit() {
            return 1;
        }

        // Skip digits
        while (*s as u8).is_ascii_digit() {
            s = s.add(1);
        }

        // Must be comma or end
        if *s != 0 {
            if *s as u8 != b',' {
                return 1;
            }
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// 'varsofttabstop' / 'vartabstop' Option Validation
// =============================================================================

/// Validate 'varsofttabstop' or 'vartabstop' option value.
/// Format: comma-separated list of positive integers.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_vartabs(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0; // Empty is valid
    }

    let mut s = value;

    while *s != 0 {
        // Must start with a digit
        if !(*s as u8).is_ascii_digit() {
            return 1;
        }

        // Parse number and check it's positive
        let mut num: i64 = 0;
        while (*s as u8).is_ascii_digit() {
            num = num * 10 + i64::from(*s as u8 - b'0');
            s = s.add(1);
        }

        // Must be positive (non-zero)
        if num <= 0 {
            return 1;
        }

        // Must be comma or end
        if *s != 0 {
            if *s as u8 != b',' {
                return 1;
            }
            s = s.add(1);

            // After comma, must have more content
            if *s == 0 {
                return 1;
            }
        }
    }

    0
}

// =============================================================================
// 'eventignore' Option Validation Helper
// =============================================================================

/// Check if a string looks like "all" (case-insensitive).
#[inline]
unsafe fn is_all(s: *const c_char, len: usize) -> bool {
    if len != 3 {
        return false;
    }
    let b0 = (*s as u8).to_ascii_lowercase();
    let b1 = (*s.add(1) as u8).to_ascii_lowercase();
    let b2 = (*s.add(2) as u8).to_ascii_lowercase();
    b0 == b'a' && b1 == b'l' && b2 == b'l'
}

/// Validate a single event name in 'eventignore'.
/// Returns 1 if the event name format is valid (actual event checking done in C).
/// Valid format: alphanumeric characters only.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_eventignore_item(item: *const c_char, len: usize) -> c_int {
    if item.is_null() || len == 0 {
        return 0;
    }

    // "all" is always valid
    if is_all(item, len) {
        return 1;
    }

    // Check all characters are alphanumeric
    for i in 0..len {
        let ch = *item.add(i) as u8;
        if !ch.is_ascii_alphanumeric() {
            return 0;
        }
    }

    1
}

// =============================================================================
// 'wildmode' Option Validation
// =============================================================================

/// Valid wildmode keywords
const WILDMODE_KEYWORDS: &[&[u8]] = &[b"", b"full", b"longest", b"list", b"lastused"];

/// Check if bytes match a wildmode keyword.
#[inline]
unsafe fn matches_wildmode_keyword(s: *const c_char, len: usize) -> bool {
    for kw in WILDMODE_KEYWORDS {
        if kw.len() == len {
            let mut matches = true;
            for i in 0..len {
                if *s.add(i) as u8 != kw[i] {
                    matches = false;
                    break;
                }
            }
            if matches {
                return true;
            }
        }
    }
    false
}

/// Validate 'wildmode' option value.
/// Format: colon-separated groups, each containing comma-separated keywords.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_wildmode(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0;
    }

    let mut s = value;

    while *s != 0 {
        // Find end of this keyword (comma, colon, or null)
        let start = s;
        while *s != 0 && *s as u8 != b',' && *s as u8 != b':' {
            s = s.add(1);
        }

        let len = s.offset_from(start) as usize;

        // Validate keyword
        if !matches_wildmode_keyword(start, len) {
            return 1;
        }

        // Skip delimiter
        if *s != 0 {
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// 'mkspellmem' Option Validation
// =============================================================================

/// Validate 'mkspellmem' option value.
/// Format: N,N,N (three comma-separated numbers).
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_mkspellmem(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 1; // Must not be empty
    }

    let mut s = value;
    let mut count = 0;

    while *s != 0 && count < 3 {
        // Must have at least one digit
        if !(*s as u8).is_ascii_digit() {
            return 1;
        }

        // Skip digits
        while (*s as u8).is_ascii_digit() {
            s = s.add(1);
        }

        count += 1;

        // After first two numbers, must have comma
        if count < 3 {
            if *s as u8 != b',' {
                return 1;
            }
            s = s.add(1);
        }
    }

    // Must have exactly 3 numbers and nothing after
    if count != 3 || *s != 0 {
        return 1;
    }

    0
}

// =============================================================================
// 'keymodel' Option Validation
// =============================================================================

/// Validate 'keymodel' option value.
/// Valid values: empty, startsel, stopsel, or comma-separated combination.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_keymodel(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0; // Empty is valid
    }

    let mut s = value;

    while *s != 0 {
        let start = s;

        // Find end of item
        while *s != 0 && *s as u8 != b',' {
            s = s.add(1);
        }

        let len = s.offset_from(start) as usize;

        // Check valid keywords: "startsel" or "stopsel"
        let valid = match len {
            8 => bytes_match(start, b"startsel"),
            7 => bytes_match(start, b"stopsel"),
            _ => false,
        };

        if !valid {
            return 1;
        }

        // Skip comma
        if *s as u8 == b',' {
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// 'messagesopt' Option Validation
// =============================================================================

/// Validate 'messagesopt' option value.
/// Format: comma-separated list of keywords with optional :N suffix.
/// Valid keywords: hit-enter, wait:N, history:N
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_messagesopt(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 1; // Must not be empty according to help
    }

    let mut s = value;

    while *s != 0 {
        let start = s;

        // Find end of keyword part (before : or ,)
        while *s != 0 && *s as u8 != b':' && *s as u8 != b',' {
            s = s.add(1);
        }

        let keyword_len = s.offset_from(start) as usize;

        // Validate keyword
        let valid_keyword = match keyword_len {
            9 => bytes_match(start, b"hit-enter"),
            4 => bytes_match(start, b"wait"),
            7 => bytes_match(start, b"history"),
            _ => false,
        };

        if !valid_keyword {
            return 1;
        }

        // Check for :N suffix (required for wait and history)
        if *s as u8 == b':' {
            s = s.add(1);
            if !(*s as u8).is_ascii_digit() {
                return 1;
            }
            while (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }
        }

        // Skip comma
        if *s as u8 == b',' {
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_validate_helplang() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_single = CString::new("en").unwrap();
            let valid_multi = CString::new("en,de,fr").unwrap();
            let invalid_short = CString::new("e").unwrap();
            let invalid_no_comma = CString::new("enen").unwrap();
            let invalid_trailing = CString::new("en,").unwrap();

            assert!(rs_validate_helplang(empty.as_ptr()).is_null());
            assert!(rs_validate_helplang(valid_single.as_ptr()).is_null());
            assert!(rs_validate_helplang(valid_multi.as_ptr()).is_null());
            assert!(!rs_validate_helplang(invalid_short.as_ptr()).is_null());
            assert!(!rs_validate_helplang(invalid_no_comma.as_ptr()).is_null());
            assert!(!rs_validate_helplang(invalid_trailing.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_shada() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid = CString::new("'100,<50,s10,h").unwrap();
            let valid_name = CString::new("'100,n~/.shada").unwrap();
            let mut errbuf = [0i8; 256];

            assert!(rs_validate_shada(empty.as_ptr(), errbuf.as_mut_ptr(), 256).is_null());
            assert!(rs_validate_shada(valid.as_ptr(), errbuf.as_mut_ptr(), 256).is_null());
            assert!(rs_validate_shada(valid_name.as_ptr(), errbuf.as_mut_ptr(), 256).is_null());
        }
    }

    #[test]
    fn test_validate_mousescroll() {
        unsafe {
            let valid = CString::new("ver:3,hor:6").unwrap();
            let valid_reverse = CString::new("hor:6,ver:3").unwrap();
            let valid_ver_only = CString::new("ver:5").unwrap();
            let invalid_empty = CString::new("").unwrap();
            let invalid_no_num = CString::new("ver:").unwrap();

            let result = rs_validate_mousescroll(valid.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.vertical, 3);
            assert_eq!(result.horizontal, 6);

            let result = rs_validate_mousescroll(valid_reverse.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.vertical, 3);
            assert_eq!(result.horizontal, 6);

            let result = rs_validate_mousescroll(valid_ver_only.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.vertical, 5);
            assert_eq!(result.horizontal, -1);

            let result = rs_validate_mousescroll(invalid_empty.as_ptr());
            assert_eq!(result.error, 1);

            let result = rs_validate_mousescroll(invalid_no_num.as_ptr());
            assert_eq!(result.error, 1);
        }
    }

    #[test]
    fn test_validate_colorcolumn() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_single = CString::new("80").unwrap();
            let valid_multi = CString::new("80,120").unwrap();
            let valid_relative = CString::new("+1,-2").unwrap();
            let invalid = CString::new("abc").unwrap();

            assert_eq!(rs_validate_colorcolumn(empty.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(valid_single.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(valid_multi.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(valid_relative.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(invalid.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_vartabs() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_single = CString::new("4").unwrap();
            let valid_multi = CString::new("4,8,4").unwrap();
            let invalid_zero = CString::new("0").unwrap();
            let invalid_trailing = CString::new("4,").unwrap();

            assert_eq!(rs_validate_vartabs(empty.as_ptr()), 0);
            assert_eq!(rs_validate_vartabs(valid_single.as_ptr()), 0);
            assert_eq!(rs_validate_vartabs(valid_multi.as_ptr()), 0);
            assert_eq!(rs_validate_vartabs(invalid_zero.as_ptr()), 1);
            assert_eq!(rs_validate_vartabs(invalid_trailing.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_wildmode() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid = CString::new("longest,list,full").unwrap();
            let valid_colon = CString::new("longest:full").unwrap();
            let invalid = CString::new("invalid").unwrap();

            assert_eq!(rs_validate_wildmode(empty.as_ptr()), 0);
            assert_eq!(rs_validate_wildmode(valid.as_ptr()), 0);
            assert_eq!(rs_validate_wildmode(valid_colon.as_ptr()), 0);
            assert_eq!(rs_validate_wildmode(invalid.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_mkspellmem() {
        unsafe {
            let valid = CString::new("460000,2000,500").unwrap();
            let invalid_empty = CString::new("").unwrap();
            let invalid_two = CString::new("100,200").unwrap();
            let invalid_four = CString::new("1,2,3,4").unwrap();

            assert_eq!(rs_validate_mkspellmem(valid.as_ptr()), 0);
            assert_eq!(rs_validate_mkspellmem(invalid_empty.as_ptr()), 1);
            assert_eq!(rs_validate_mkspellmem(invalid_two.as_ptr()), 1);
            assert_eq!(rs_validate_mkspellmem(invalid_four.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_keymodel() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_start = CString::new("startsel").unwrap();
            let valid_stop = CString::new("stopsel").unwrap();
            let valid_both = CString::new("startsel,stopsel").unwrap();
            let invalid = CString::new("invalid").unwrap();

            assert_eq!(rs_validate_keymodel(empty.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(valid_start.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(valid_stop.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(valid_both.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(invalid.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_eventignore_item() {
        unsafe {
            let all = CString::new("all").unwrap();
            let all_upper = CString::new("ALL").unwrap();
            let valid = CString::new("BufEnter").unwrap();
            let invalid = CString::new("buf-enter").unwrap();

            assert_eq!(rs_validate_eventignore_item(all.as_ptr(), 3), 1);
            assert_eq!(rs_validate_eventignore_item(all_upper.as_ptr(), 3), 1);
            assert_eq!(rs_validate_eventignore_item(valid.as_ptr(), 8), 1);
            assert_eq!(rs_validate_eventignore_item(invalid.as_ptr(), 9), 0);
        }
    }
}
