//! String parsing utilities for option values
//!
//! This module provides pure string parsing functions with no side effects,
//! used for validating and processing option values.

use std::ffi::{c_char, c_int, c_uint};

// =============================================================================
// External C Functions (only used when not testing)
// =============================================================================

#[cfg(not(test))]
extern "C" {
    // From strings crate
    fn rs_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn rs_vim_snprintf(str: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;

    // From charset.c
    fn transchar(c: c_int) -> *const c_char;

    // Global variable accessors
    fn nvim_option_get_secure() -> c_int;
}

// =============================================================================
// Test Stubs (provide mock implementations for unit tests)
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_possible_truncation)]
unsafe fn rs_vim_strchr(s: *const c_char, c: c_int) -> *const c_char {
    if s.is_null() {
        return std::ptr::null();
    }
    let target = c as u8;
    let mut p = s;
    while *p != 0 {
        if (*p as u8) == target {
            return p;
        }
        p = p.add(1);
    }
    std::ptr::null()
}

// Note: rs_vim_snprintf and transchar are not mocked for tests because:
// - rs_illegal_char and rs_illegal_char_after are not tested directly
// - They require C linkage for variadic functions which doesn't work in test mode

#[cfg(test)]
unsafe fn nvim_option_get_secure() -> c_int {
    0 // Not in secure mode for tests
}

// =============================================================================
// Constants (only used when not testing)
// =============================================================================

/// Error message for illegal character.
#[cfg(not(test))]
const E_ILLEGAL_CHAR: &[u8] = b"E539: Illegal character <%s>\0";

/// Error message for illegal character after specific character.
#[cfg(not(test))]
const E_ILLEGAL_CHAR_AFTER: &[u8] = b"E535: Illegal character after <%c>\0";

// =============================================================================
// Illegal Character Reporting
// =============================================================================

/// Report an illegal character in an option value.
///
/// Formats an error message like "E539: Illegal character <x>" into the
/// provided error buffer.
///
/// # Arguments
///
/// * `errbuf` - Buffer to write the error message to
/// * `errbuflen` - Size of the error buffer
/// * `c` - The illegal character
///
/// # Returns
///
/// Pointer to errbuf on success, or empty string if errbuf is NULL.
///
/// # Safety
///
/// `errbuf` must be a valid buffer of at least `errbuflen` bytes, or NULL.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn rs_illegal_char(
    errbuf: *mut c_char,
    errbuflen: usize,
    c: c_int,
) -> *const c_char {
    static EMPTY: &[u8] = b"\0";

    if errbuf.is_null() {
        return EMPTY.as_ptr().cast();
    }

    let transchar_result = transchar(c);
    rs_vim_snprintf(
        errbuf,
        errbuflen,
        E_ILLEGAL_CHAR.as_ptr().cast(),
        transchar_result,
    );

    errbuf
}

/// Report an illegal character appearing after a specific character.
///
/// Formats an error message like "E535: Illegal character after <x>" into the
/// provided error buffer.
///
/// # Arguments
///
/// * `errbuf` - Buffer to write the error message to
/// * `errbuflen` - Size of the error buffer
/// * `c` - The character that was before the illegal character
///
/// # Returns
///
/// Pointer to errbuf on success, or empty string if errbuf is NULL.
///
/// # Safety
///
/// `errbuf` must be a valid buffer of at least `errbuflen` bytes, or NULL.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn rs_illegal_char_after(
    errbuf: *mut c_char,
    errbuflen: usize,
    c: c_int,
) -> *const c_char {
    static EMPTY: &[u8] = b"\0";

    if errbuf.is_null() {
        return EMPTY.as_ptr().cast();
    }

    rs_vim_snprintf(errbuf, errbuflen, E_ILLEGAL_CHAR_AFTER.as_ptr().cast(), c);

    errbuf
}

// =============================================================================
// Filetype Validation
// =============================================================================

/// Check if a string is a valid filetype name.
///
/// A valid filetype name consists only of alphanumeric ASCII characters,
/// dots (`.`), hyphens (`-`), and underscores (`_`).
///
/// Also used for 'syntax' and 'keymap' validation.
///
/// # Arguments
///
/// * `val` - The filetype string to validate
///
/// # Returns
///
/// 1 if valid, 0 if invalid.
///
/// # Safety
///
/// `val` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_valid_filetype(val: *const c_char) -> c_int {
    // Uses the same logic as valid_name with ".-_" as allowed characters
    rs_valid_name_with_allowed(val, c".-_".as_ptr())
}

/// Check if a string is a valid name with specified allowed extra characters.
///
/// # Safety
///
/// Both `val` and `allowed` must be valid null-terminated C strings.
#[inline]
unsafe fn rs_valid_name_with_allowed(val: *const c_char, allowed: *const c_char) -> c_int {
    if val.is_null() {
        return 1; // Empty/null is considered valid
    }

    let mut s = val;
    loop {
        let c = *s as u8;
        if c == 0 {
            break;
        }

        // Check if alphanumeric
        if !c.is_ascii_alphanumeric() {
            // Check if in allowed set
            if allowed.is_null() || rs_vim_strchr(allowed, c_int::from(c)).is_null() {
                return 0;
            }
        }

        s = s.add(1);
    }

    1
}

// =============================================================================
// Path Name Validation
// =============================================================================

/// Check for illegal characters in path names based on option flags.
///
/// Disallows path separators, wildcards and characters that are often illegal
/// in file names. More permissive if `secure` mode is off.
///
/// # Arguments
///
/// * `val` - The path string to check
/// * `flags` - Option flags (checks kOptFlagNFname and kOptFlagNDname)
///
/// # Returns
///
/// 1 if the path contains illegal characters, 0 if it's valid.
///
/// # Safety
///
/// `val` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_check_illegal_path_names(val: *const c_char, flags: c_uint) -> c_int {
    if val.is_null() {
        return 0;
    }

    let secure = nvim_option_get_secure() != 0;

    // kOptFlagNFname = (1 << 17) - only normal file name chars allowed
    let check_file = (flags & (1 << 17)) != 0;
    // kOptFlagNDname = (1 << 22) - only normal directory name chars allowed
    let check_dir = (flags & (1 << 22)) != 0;

    // Characters to check for file names
    // More restrictive in secure mode
    let file_illegal: &[u8] = if secure {
        b"/\\*?[|;&<>\r\n"
    } else {
        b"/\\*?[<>\r\n"
    };

    // Characters to check for directory names
    let dir_illegal: &[u8] = b"*?[|;&<>\r\n";

    let illegal_for_file = check_file && contains_any_char(val, file_illegal);
    let illegal_for_dir = check_dir && contains_any_char(val, dir_illegal);

    c_int::from(illegal_for_file || illegal_for_dir)
}

/// Check if a string contains any character from the given set.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string.
#[inline]
unsafe fn contains_any_char(s: *const c_char, chars: &[u8]) -> bool {
    let mut p = s;
    while *p != 0 {
        let c = *p as u8;
        if chars.contains(&c) {
            return true;
        }
        p = p.add(1);
    }
    false
}

// =============================================================================
// Duplicate Item Finding
// =============================================================================

/// Find a duplicate item in a comma-separated option value.
///
/// For an option value that contains comma-separated items, find `newval`
/// in `origval`. Handles escaped commas (backslashes before commas).
///
/// # Arguments
///
/// * `origval` - The original comma-separated list
/// * `newval` - The value to search for
/// * `newvallen` - Length of newval
/// * `flags` - Option flags (checks kOptFlagComma)
///
/// # Returns
///
/// Pointer to the found item in origval, or NULL if not found.
///
/// # Safety
///
/// `origval` (if non-null) and `newval` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_find_dup_item(
    origval: *const c_char,
    newval: *const c_char,
    newvallen: usize,
    flags: c_uint,
) -> *const c_char {
    if origval.is_null() || newval.is_null() || newvallen == 0 {
        return std::ptr::null();
    }

    // kOptFlagComma = (1 << 10) - comma-separated list
    let is_comma_list = (flags & (1 << 10)) != 0;

    let mut bs = 0_i32; // backslash count

    let mut s = origval;
    while *s != 0 {
        // Check if we're at a valid start position for comparison
        let at_start = if !is_comma_list {
            // Non-comma list: can match anywhere
            true
        } else if s == origval {
            // First character is always a valid start
            true
        } else {
            // In comma list: valid start is after a comma with even backslashes
            let prev_char = *s.sub(1) as u8;
            prev_char == b',' && (bs & 1) == 0
        };

        if at_start {
            // Check if the substring matches
            if strncmp_len(s, newval, newvallen) {
                // Check if we're at end of item (for comma lists)
                if !is_comma_list {
                    return s;
                }
                let next_char = *s.add(newvallen) as u8;
                if next_char == b',' || next_char == 0 {
                    return s;
                }
            }
        }

        // Count backslashes.
        // Only a comma with an even number of backslashes or a single backslash
        // preceded by a comma before it is recognized as a separator.
        let cur_is_backslash = (*s as u8) == b'\\';
        if cur_is_backslash {
            if s > origval.add(1) {
                let prev = *s.sub(1) as u8;
                let prev2 = *s.sub(2) as u8;
                if prev == b'\\' && prev2 != b',' {
                    bs += 1;
                } else {
                    bs = 1;
                }
            } else if s == origval.add(1) && (*s.sub(1) as u8) == b'\\' {
                bs += 1;
            } else {
                bs = 1;
            }
        } else {
            bs = 0;
        }

        s = s.add(1);
    }

    std::ptr::null()
}

/// Compare `n` bytes of two strings for equality.
///
/// # Safety
///
/// Both pointers must be valid for reads of `len` bytes.
#[inline]
unsafe fn strncmp_len(s1: *const c_char, s2: *const c_char, len: usize) -> bool {
    for i in 0..len {
        if *s1.add(i) != *s2.add(i) {
            return false;
        }
    }
    true
}

// =============================================================================
// Option String Parsing Utilities
// =============================================================================

// Note: rs_skip_to_option_part is defined in the strings crate.

/// Copy an option part from a comma-separated list.
///
/// Copies characters from `p` into `buf` until a comma, NUL, or buffer limit.
/// Advances `p` past the copied part and any trailing comma.
///
/// # Arguments
///
/// * `pp` - Pointer to string pointer (updated on return)
/// * `buf` - Buffer to copy into
/// * `maxlen` - Maximum number of bytes to copy (including NUL)
/// * `sep` - Separator string (usually ",")
///
/// # Returns
///
/// Length of the copied string.
///
/// # Safety
///
/// `pp` and `*pp` must be valid, `buf` must have at least `maxlen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_option_part(
    pp: *mut *const c_char,
    buf: *mut c_char,
    maxlen: usize,
    sep: *const c_char,
) -> usize {
    if pp.is_null() || (*pp).is_null() || buf.is_null() || maxlen == 0 {
        return 0;
    }

    let mut p = *pp;
    let mut len: usize = 0;

    // Copy until separator or end
    while *p != 0 && len < maxlen - 1 {
        let c = *p as u8;

        // Check if current char is a separator
        let is_sep = if sep.is_null() {
            c == b','
        } else {
            !rs_vim_strchr(sep, c_int::from(c)).is_null()
        };

        if is_sep {
            break;
        }

        *buf.add(len) = *p;
        len += 1;
        p = p.add(1);
    }

    // NUL-terminate
    *buf.add(len) = 0;

    // Skip the separator
    if *p != 0 {
        p = p.add(1);
    }

    *pp = p;
    len
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_valid_filetype() {
        unsafe {
            // Valid filetypes
            let valid1 = CString::new("python").unwrap();
            let valid2 = CString::new("c").unwrap();
            let valid3 = CString::new("vim-plug").unwrap();
            let valid4 = CString::new("config.yaml").unwrap();
            let valid5 = CString::new("my_filetype").unwrap();

            assert_eq!(rs_valid_filetype(valid1.as_ptr()), 1);
            assert_eq!(rs_valid_filetype(valid2.as_ptr()), 1);
            assert_eq!(rs_valid_filetype(valid3.as_ptr()), 1);
            assert_eq!(rs_valid_filetype(valid4.as_ptr()), 1);
            assert_eq!(rs_valid_filetype(valid5.as_ptr()), 1);

            // Invalid filetypes (contain disallowed characters)
            let invalid1 = CString::new("file type").unwrap(); // space
            let invalid2 = CString::new("file:type").unwrap(); // colon
            let invalid3 = CString::new("file/type").unwrap(); // slash

            assert_eq!(rs_valid_filetype(invalid1.as_ptr()), 0);
            assert_eq!(rs_valid_filetype(invalid2.as_ptr()), 0);
            assert_eq!(rs_valid_filetype(invalid3.as_ptr()), 0);

            // Edge cases
            let empty = CString::new("").unwrap();
            assert_eq!(rs_valid_filetype(empty.as_ptr()), 1);
            assert_eq!(rs_valid_filetype(std::ptr::null()), 1);
        }
    }

    #[test]
    fn test_find_dup_item_non_comma() {
        unsafe {
            let origval = CString::new("abcdef").unwrap();
            let search = CString::new("cde").unwrap();

            let result = rs_find_dup_item(origval.as_ptr(), search.as_ptr(), 3, 0);
            assert!(!result.is_null());
            assert_eq!(result.offset_from(origval.as_ptr()), 2);
        }
    }

    #[test]
    fn test_find_dup_item_comma_list() {
        unsafe {
            let origval = CString::new("one,two,three").unwrap();

            // Find "two"
            let search1 = CString::new("two").unwrap();
            let result1 = rs_find_dup_item(origval.as_ptr(), search1.as_ptr(), 3, 1 << 10);
            assert!(!result1.is_null());

            // Find "three"
            let search2 = CString::new("three").unwrap();
            let result2 = rs_find_dup_item(origval.as_ptr(), search2.as_ptr(), 5, 1 << 10);
            assert!(!result2.is_null());

            // Substring "wo" should not be found as a separate item
            let search3 = CString::new("wo").unwrap();
            let result3 = rs_find_dup_item(origval.as_ptr(), search3.as_ptr(), 2, 1 << 10);
            assert!(result3.is_null());

            // "one" should be found
            let search4 = CString::new("one").unwrap();
            let result4 = rs_find_dup_item(origval.as_ptr(), search4.as_ptr(), 3, 1 << 10);
            assert!(!result4.is_null());
            assert_eq!(result4, origval.as_ptr());
        }
    }

    #[test]
    fn test_find_dup_item_not_found() {
        unsafe {
            let origval = CString::new("one,two,three").unwrap();
            let search = CString::new("four").unwrap();

            let result = rs_find_dup_item(origval.as_ptr(), search.as_ptr(), 4, 1 << 10);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_find_dup_item_null() {
        unsafe {
            let search = CString::new("test").unwrap();

            // origval is null
            let result = rs_find_dup_item(std::ptr::null(), search.as_ptr(), 4, 0);
            assert!(result.is_null());

            // newval is null
            let origval = CString::new("test").unwrap();
            let result2 = rs_find_dup_item(origval.as_ptr(), std::ptr::null(), 4, 0);
            assert!(result2.is_null());
        }
    }

    // Note: test_skip_to_option_part is in the strings crate

    #[test]
    fn test_copy_option_part() {
        unsafe {
            let input = CString::new("first,second,third").unwrap();
            let mut p = input.as_ptr();
            let mut buf = [0i8; 32];
            let sep = CString::new(",").unwrap();

            // Copy first part
            let len1 = rs_copy_option_part(&raw mut p, buf.as_mut_ptr(), 32, sep.as_ptr());
            assert_eq!(len1, 5);
            // Check the buffer content directly
            assert_eq!(buf[0] as u8, b'f');
            assert_eq!(buf[1] as u8, b'i');
            assert_eq!(buf[2] as u8, b'r');
            assert_eq!(buf[3] as u8, b's');
            assert_eq!(buf[4] as u8, b't');
            assert_eq!(buf[5], 0);

            // Copy second part
            let mut buf2 = [0i8; 32];
            let len2 = rs_copy_option_part(&raw mut p, buf2.as_mut_ptr(), 32, sep.as_ptr());
            assert_eq!(len2, 6);
            assert_eq!(buf2[0] as u8, b's');
            assert_eq!(buf2[1] as u8, b'e');
        }
    }
}
