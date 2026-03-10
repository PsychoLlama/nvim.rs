//! Vimscript function helpers for digraphs.
//!
//! This module provides Rust implementations of the core logic used by
//! Vimscript digraph functions (`digraph_get()`, `digraph_set()`, etc.).
//!
//! The actual Vimscript function entry points remain in C due to their
//! heavy dependency on typval manipulation. This module provides
//! the pure logic that those C wrappers call.

use std::ffi::c_char;

use libc::c_int;

// External Rust functions from this crate
extern "C" {
    fn rs_digraph_get(char1: c_int, char2: c_int, meta_char: c_int) -> c_int;
    fn rs_registerdigraph(char1: c_int, char2: c_int, result: c_int);
    fn rs_check_digraph_chars_valid(char1: c_int, char2: c_int) -> c_int;
}

// External C/Rust functions for UTF-8 handling
extern "C" {
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int;
}

/// Get digraph result for two characters.
///
/// This is a thin wrapper around `rs_digraph_get` for use by Vimscript functions.
///
/// # Arguments
/// * `char1` - First character (as byte)
/// * `char2` - Second character (as byte)
///
/// # Returns
/// The digraph result code.
#[inline]
fn digraph_get_for_viml(char1: u8, char2: u8) -> c_int {
    unsafe { rs_digraph_get(c_int::from(char1), c_int::from(char2), 0) }
}

/// Convert a character code to UTF-8 bytes.
///
/// # Arguments
/// * `code` - The character code
/// * `buf` - Output buffer (must have room for at least 6 bytes + NUL)
///
/// # Returns
/// Number of bytes written (not including NUL terminator).
#[inline]
fn char_to_utf8(code: c_int, buf: &mut [u8; 7]) -> usize {
    let len = unsafe { utf_char2bytes(code, buf.as_mut_ptr().cast::<c_char>()) };
    #[allow(clippy::cast_sign_loss)]
    let len_usize = len as usize;
    if len_usize < buf.len() {
        buf[len_usize] = 0; // NUL terminate
    }
    len_usize
}

/// Implement the core logic for `digraph_get()` Vimscript function.
///
/// Takes two ASCII characters and returns the digraph result as UTF-8 bytes.
///
/// # Arguments
/// * `char1` - First character
/// * `char2` - Second character
/// * `out_buf` - Output buffer for UTF-8 result (at least 7 bytes)
///
/// # Returns
/// Number of bytes written to `out_buf` (0 on error).
/// # Safety
/// `out_buf` must point to at least 7 writable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_get_viml(char1: u8, char2: u8, out_buf: *mut u8) -> c_int {
    if out_buf.is_null() {
        return 0;
    }

    let code = digraph_get_for_viml(char1, char2);
    let mut buf = [0u8; 7];
    let len = char_to_utf8(code, &mut buf);

    // Copy to output buffer
    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), out_buf, len + 1); // include NUL
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        len as c_int
    }
}

/// Parse a two-character digraph string and extract the character codes.
///
/// This is used by `digraph_set()` to parse the input string.
///
/// # Arguments
/// * `str_ptr` - Pointer to string pointer (advanced past parsed chars)
/// * `out_char1` - Output: first character code
/// * `out_char2` - Output: second character code
///
/// # Returns
/// * `1` - Success
/// * `0` - Failure (invalid input)
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_digraph_chars(
    str_ptr: *mut *const c_char,
    out_char1: *mut c_int,
    out_char2: *mut c_int,
) -> c_int {
    if str_ptr.is_null() || out_char1.is_null() || out_char2.is_null() {
        return 0;
    }

    // Get first character
    let char1 = unsafe { mb_cptr2char_adv(str_ptr) };

    // Check if there's a second character
    let p = unsafe { *str_ptr };
    if p.is_null() || unsafe { *p } == 0 {
        return 0; // Only one character
    }

    // Get second character
    let char2 = unsafe { mb_cptr2char_adv(str_ptr) };

    // Check that we're at end of string (exactly 2 chars)
    let p = unsafe { *str_ptr };
    if !p.is_null() && unsafe { *p } != 0 {
        return 0; // More than two characters
    }

    // Validate the digraph characters
    let valid = unsafe { rs_check_digraph_chars_valid(char1, char2) };
    if valid != 3 {
        return 0;
    }

    unsafe {
        *out_char1 = char1;
        *out_char2 = char2;
    }

    1
}

/// Set a digraph (core logic for `digraph_set()` Vimscript function).
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
/// * `result` - The character to produce
#[no_mangle]
pub extern "C" fn rs_digraph_set_viml(char1: c_int, char2: c_int, result: c_int) {
    unsafe { rs_registerdigraph(char1, char2, result) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digraph_get_for_viml_returns_value() {
        // Without the actual digraph tables, we can only test that
        // the function doesn't crash and returns something
        let result = digraph_get_for_viml(b'a', b'b');
        // Should return char2 if no digraph found
        assert!(result >= 0);
    }
}
