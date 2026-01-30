//! Digraph string parsing.
//!
//! This module provides the Rust implementation for parsing digraph strings
//! from the `:digraph` command format: `{c1}{c2} char {c1}{c2} char ...`

use std::ffi::c_char;

use libc::c_int;

/// ESC key code (27).
const ESC: u8 = 27;

// External functions from charset crate (via FFI)
extern "C" {
    /// Skip over whitespace characters.
    fn rs_skipwhite(p: *const c_char) -> *const c_char;

    /// Get an integer from a string, advancing the pointer.
    fn rs_getdigits_int(pp: *mut *mut c_char, strict: c_int, def: c_int) -> c_int;
}

// External Rust functions from this crate
extern "C" {
    fn rs_registerdigraph(char1: c_int, char2: c_int, result: c_int);
}

/// Check if character is ASCII digit ('0'-'9')
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Result codes for putdigraph parsing.
/// These are designed so C can determine what error message to emit.
#[repr(C)]
pub struct PutdigraphResult {
    /// 0 = success, 1 = char validation error, 2 = number expected
    pub error_code: c_int,
    /// First character (for error messages)
    pub char1: c_int,
    /// Second character (for error messages)
    pub char2: c_int,
}

/// Parse digraph definitions from a string.
///
/// Format: `{c1}{c2} char {c1}{c2} char ...`
///
/// Where `{c1}` and `{c2}` are the two characters forming the digraph,
/// and `char` is the decimal Unicode codepoint result.
///
/// # Arguments
/// * `str_ptr` - Pointer to the input string (modified to point past parsed content)
/// * `result` - Output: result structure with error info if failed
///
/// # Returns
/// * `1` - Success (all digraphs parsed)
/// * `0` - Failure (check result struct for error details)
///
/// Error codes in result:
/// * `0` - Success
/// * `1` - Invalid digraph characters (char1/char2 populated)
/// * `2` - Number expected (no digits found)
///
/// # Safety
/// `str_ptr` and `result` must be valid pointers.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_putdigraph(
    str_ptr: *mut *mut c_char,
    result: *mut PutdigraphResult,
) -> c_int {
    if str_ptr.is_null() || result.is_null() {
        return 0;
    }

    // Initialize result
    unsafe {
        (*result).error_code = 0;
        (*result).char1 = 0;
        (*result).char2 = 0;
    }

    let mut p = unsafe { *str_ptr };
    if p.is_null() {
        return 1; // Empty input is success
    }

    loop {
        // Skip whitespace
        p = unsafe { rs_skipwhite(p) }.cast_mut();

        // Check for end of string
        if unsafe { *p } == 0 {
            unsafe { *str_ptr = p };
            return 1; // Success - parsed all digraphs
        }

        // Get first character of digraph
        let char1 = unsafe { *p } as u8;
        if char1 == 0 {
            unsafe { *str_ptr = p };
            return 1;
        }
        p = unsafe { p.add(1) };

        // Get second character of digraph
        let char2 = unsafe { *p } as u8;

        // Validate: char2 must not be NUL
        if char2 == 0 {
            // Only one character - digraph must be two characters
            unsafe {
                (*result).error_code = 1;
                (*result).char1 = c_int::from(char1);
                (*result).char2 = 0;
                *str_ptr = p;
            }
            return 0;
        }
        p = unsafe { p.add(1) };

        // Validate: neither char can be ESC (27)
        if char1 == ESC || char2 == ESC {
            unsafe {
                (*result).error_code = 1;
                (*result).char1 = c_int::from(char1);
                (*result).char2 = c_int::from(char2);
                *str_ptr = p;
            }
            return 0;
        }

        // Skip whitespace before the number
        p = unsafe { rs_skipwhite(p) }.cast_mut();

        // Check for digit
        let c = unsafe { *p } as u8;
        if !ascii_isdigit(c) {
            unsafe {
                (*result).error_code = 2;
                *str_ptr = p;
            }
            return 0;
        }

        // Parse the result number
        let n = unsafe { rs_getdigits_int(std::ptr::addr_of_mut!(p), 1, 0) };

        // Register the digraph
        unsafe { rs_registerdigraph(c_int::from(char1), c_int::from(char2), n) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_isdigit() {
        assert!(ascii_isdigit(b'0'));
        assert!(ascii_isdigit(b'5'));
        assert!(ascii_isdigit(b'9'));
        assert!(!ascii_isdigit(b'a'));
        assert!(!ascii_isdigit(b' '));
    }

    #[test]
    fn test_putdigraph_null() {
        // Null pointer should return 0
        let mut result = PutdigraphResult {
            error_code: 0,
            char1: 0,
            char2: 0,
        };
        let ret = unsafe { rs_putdigraph(std::ptr::null_mut(), &raw mut result) };
        assert_eq!(ret, 0);
    }

    #[test]
    fn test_putdigraph_result_size() {
        // Verify the struct has expected alignment for FFI
        use std::mem::{align_of, size_of};
        assert!(size_of::<PutdigraphResult>() >= 12); // 3 * c_int minimum
        assert!(align_of::<PutdigraphResult>() >= align_of::<c_int>());
    }

    #[test]
    fn test_esc_constant() {
        assert_eq!(ESC, 27);
    }
}
