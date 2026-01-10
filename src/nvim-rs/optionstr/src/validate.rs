//! Option value validation utilities
//!
//! This module provides validation functions for various option types.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int};

// =============================================================================
// String Value Validation
// =============================================================================

/// Check if a string matches one of the allowed values
///
/// # Safety
/// The `value` and `allowed` pointers must be valid for reading.
/// `allowed` must be a null-terminated array of null-terminated strings.
#[no_mangle]
pub unsafe extern "C" fn rs_opt_value_in_list(
    value: *const c_char,
    allowed: *const *const c_char,
) -> bool {
    if value.is_null() || allowed.is_null() {
        return false;
    }

    let mut idx = 0;
    loop {
        let allowed_val = *allowed.add(idx);
        if allowed_val.is_null() {
            break;
        }

        if strcmp_c(value, allowed_val) == 0 {
            return true;
        }

        idx += 1;
    }

    false
}

/// Compare two C strings
unsafe fn strcmp_c(s1: *const c_char, s2: *const c_char) -> c_int {
    let mut p1 = s1;
    let mut p2 = s2;

    while *p1 != 0 && *p2 != 0 && *p1 == *p2 {
        p1 = p1.add(1);
        p2 = p2.add(1);
    }

    (*p1 as c_int) - (*p2 as c_int)
}

/// Find index of value in allowed values list
///
/// Returns -1 if not found.
///
/// # Safety
/// The `value` and `allowed` pointers must be valid for reading.
#[no_mangle]
pub unsafe extern "C" fn rs_opt_value_index(
    value: *const c_char,
    allowed: *const *const c_char,
) -> c_int {
    if value.is_null() || allowed.is_null() {
        return -1;
    }

    let mut idx = 0;
    loop {
        let allowed_val = *allowed.add(idx);
        if allowed_val.is_null() {
            break;
        }

        if strcmp_c(value, allowed_val) == 0 {
            return idx as c_int;
        }

        idx += 1;
    }

    -1
}

// =============================================================================
// Number Validation
// =============================================================================

/// Validate number is within range
#[no_mangle]
pub extern "C" fn rs_opt_num_in_range(value: i64, min: i64, max: i64) -> bool {
    value >= min && value <= max
}

/// Validate number is positive
#[no_mangle]
pub extern "C" fn rs_opt_num_is_positive(value: i64) -> bool {
    value > 0
}

/// Validate number is non-negative
#[no_mangle]
pub extern "C" fn rs_opt_num_is_nonneg(value: i64) -> bool {
    value >= 0
}

/// Validate number is a percentage (0-100)
#[no_mangle]
pub extern "C" fn rs_opt_num_is_percent(value: i64) -> bool {
    (0..=100).contains(&value)
}

// =============================================================================
// Path Validation
// =============================================================================

/// Check if a character is valid in a file name
#[no_mangle]
pub extern "C" fn rs_is_valid_fname_char(c: c_int) -> bool {
    if !(0..=255).contains(&c) {
        return false;
    }

    let c = c as u8;

    // Control characters and certain special chars are not allowed
    !matches!(c, 0..=31 | b'*' | b'?' | b'|' | b'<' | b'>' | b'"')
}

/// Check if a character is valid in a directory name
#[no_mangle]
pub extern "C" fn rs_is_valid_dname_char(c: c_int) -> bool {
    // Same as fname but also excludes path separators
    if !rs_is_valid_fname_char(c) {
        return false;
    }

    let c = c as u8;
    c != b'/' && c != b'\\'
}

/// Validate file name characters in a string
///
/// Returns 0 if valid, or the invalid character.
///
/// # Safety
/// The `s` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_fname_chars(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let mut ptr = s;
    while *ptr != 0 {
        let c = *ptr as c_int;
        if !rs_is_valid_fname_char(c) {
            return c;
        }
        ptr = ptr.add(1);
    }

    0
}

/// Validate directory name characters in a string
///
/// Returns 0 if valid, or the invalid character.
///
/// # Safety
/// The `s` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_dname_chars(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let mut ptr = s;
    while *ptr != 0 {
        let c = *ptr as c_int;
        if !rs_is_valid_dname_char(c) {
            return c;
        }
        ptr = ptr.add(1);
    }

    0
}

// =============================================================================
// Spelllang Validation
// =============================================================================

/// Check if a character is valid for spelllang
#[no_mangle]
pub extern "C" fn rs_is_valid_spelllang_char(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }

    let c = c as u8;

    // Alphabetic, underscore, or specific suffixes
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'.' || c == b','
}

/// Validate spelllang option value
///
/// # Safety
/// The `s` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_spelllang(s: *const c_char) -> bool {
    if s.is_null() || *s == 0 {
        return true; // Empty is valid
    }

    let mut ptr = s;
    while *ptr != 0 {
        let c = *ptr as c_int;
        if !rs_is_valid_spelllang_char(c) {
            return false;
        }
        ptr = ptr.add(1);
    }

    true
}

// =============================================================================
// Encoding Validation
// =============================================================================

/// Common encoding names for validation
pub const COMMON_ENCODINGS: &[&str] = &[
    "utf-8",
    "utf-16",
    "utf-16le",
    "utf-16be",
    "utf-32",
    "utf-32le",
    "utf-32be",
    "latin1",
    "iso-8859-1",
    "iso-8859-2",
    "iso-8859-15",
    "cp437",
    "cp850",
    "cp1250",
    "cp1251",
    "cp1252",
    "koi8-r",
    "koi8-u",
    "euc-jp",
    "euc-kr",
    "euc-cn",
    "shift_jis",
    "cp932",
    "cp936",
    "cp949",
    "cp950",
    "gb2312",
    "gbk",
    "gb18030",
    "big5",
];

/// Check if string starts with a common encoding prefix
///
/// # Safety
/// The `s` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_is_known_encoding(s: *const c_char, len: usize) -> bool {
    if s.is_null() || len == 0 {
        return false;
    }

    let slice = std::slice::from_raw_parts(s.cast::<u8>(), len);
    let encoding = match std::str::from_utf8(slice) {
        Ok(s) => s.to_ascii_lowercase(),
        Err(_) => return false,
    };

    COMMON_ENCODINGS.iter().any(|&e| e == encoding)
}

// =============================================================================
// Highlight Group Name Validation
// =============================================================================

/// Check if character is valid for highlight group name
#[no_mangle]
pub extern "C" fn rs_is_valid_hlgroup_char(c: c_int, first: bool) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }

    let c = c as u8;

    if first {
        // First character must be alphabetic or underscore
        c.is_ascii_alphabetic() || c == b'_'
    } else {
        // Rest can be alphanumeric or underscore
        c.is_ascii_alphanumeric() || c == b'_'
    }
}

/// Validate highlight group name
///
/// # Safety
/// The `s` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_hlgroup_name(s: *const c_char) -> bool {
    if s.is_null() || *s == 0 {
        return false; // Empty is not valid
    }

    let mut ptr = s;
    let mut first = true;

    while *ptr != 0 {
        let c = *ptr as c_int;
        if !rs_is_valid_hlgroup_char(c, first) {
            return false;
        }
        first = false;
        ptr = ptr.add(1);
    }

    true
}

// =============================================================================
// Regex Pattern Validation
// =============================================================================

/// Check if character could be a regex delimiter
#[no_mangle]
pub extern "C" fn rs_is_regex_delim(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }

    let c = c as u8;

    // Common delimiters: /, |, #, @, etc.
    // Not alphanumeric, not backslash, not space
    !c.is_ascii_alphanumeric() && c != b'\\' && c != b' ' && c != b'\t' && c != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_validation() {
        assert!(rs_opt_num_in_range(5, 0, 10));
        assert!(!rs_opt_num_in_range(15, 0, 10));
        assert!(rs_opt_num_is_positive(1));
        assert!(!rs_opt_num_is_positive(0));
        assert!(rs_opt_num_is_nonneg(0));
        assert!(!rs_opt_num_is_nonneg(-1));
        assert!(rs_opt_num_is_percent(50));
        assert!(!rs_opt_num_is_percent(101));
    }

    #[test]
    fn test_fname_chars() {
        assert!(rs_is_valid_fname_char(b'a' as c_int));
        assert!(rs_is_valid_fname_char(b'.' as c_int));
        assert!(!rs_is_valid_fname_char(b'*' as c_int));
        assert!(!rs_is_valid_fname_char(0));
    }

    #[test]
    fn test_dname_chars() {
        assert!(rs_is_valid_dname_char(b'a' as c_int));
        assert!(!rs_is_valid_dname_char(b'/' as c_int));
        assert!(!rs_is_valid_dname_char(b'\\' as c_int));
    }

    #[test]
    fn test_spelllang() {
        unsafe {
            assert!(rs_validate_spelllang(b"en\0".as_ptr().cast()));
            assert!(rs_validate_spelllang(b"en_US\0".as_ptr().cast()));
            assert!(rs_validate_spelllang(b"en,de\0".as_ptr().cast()));
            assert!(!rs_validate_spelllang(b"en@bad\0".as_ptr().cast()));
        }
    }

    #[test]
    fn test_hlgroup_name() {
        unsafe {
            assert!(rs_validate_hlgroup_name(b"Normal\0".as_ptr().cast()));
            assert!(rs_validate_hlgroup_name(b"MyGroup1\0".as_ptr().cast()));
            assert!(rs_validate_hlgroup_name(b"_underscore\0".as_ptr().cast()));
            assert!(!rs_validate_hlgroup_name(b"1BadStart\0".as_ptr().cast()));
            assert!(!rs_validate_hlgroup_name(b"\0".as_ptr().cast()));
        }
    }

    #[test]
    fn test_regex_delim() {
        assert!(rs_is_regex_delim(b'/' as c_int));
        assert!(rs_is_regex_delim(b'|' as c_int));
        assert!(rs_is_regex_delim(b'#' as c_int));
        assert!(!rs_is_regex_delim(b'a' as c_int));
        assert!(!rs_is_regex_delim(b'\\' as c_int));
    }
}
