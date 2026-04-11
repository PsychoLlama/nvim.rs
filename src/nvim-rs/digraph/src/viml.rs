//! Vimscript function helpers for digraphs.
//!
//! This module provides Rust implementations of the core logic used by
//! Vimscript digraph functions (`digraph_get()`, `digraph_set()`, etc.),
//! as well as direct replacements for thin C wrappers.

use std::ffi::c_char;

use libc::c_int;

// External Rust functions from this crate
extern "C" {
    #[link_name = "digraph_get"]
    fn rs_digraph_get(char1: c_int, char2: c_int, meta_char: bool) -> c_int;
    fn rs_registerdigraph(char1: c_int, char2: c_int, result: c_int);
    fn rs_check_digraph_chars_valid(char1: c_int, char2: c_int) -> c_int;
}

// External C/Rust functions for UTF-8 handling
extern "C" {
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int;
}

// External C functions for error messages
extern "C" {
    fn emsg(s: *const c_char) -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn nvim_gettext(s: *const c_char) -> *const c_char;
    static e_number_exp: [u8; 0];
}

/// Error strings for digraph validation (matching C constants).
const E_DIGRAPH_MUST_BE_TWO_CHARS: &[u8] = b"E1214: Digraph must be just two characters: %s\0";
const E_ESCAPE_NOT_ALLOWED: &[u8] = b"E104: Escape not allowed in digraph\0";

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
    unsafe { rs_digraph_get(c_int::from(char1), c_int::from(char2), false) }
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

// =============================================================================
// Phase 1: Direct Rust exports replacing thin C wrappers
// =============================================================================

/// Static buffer for `get_digraph_for_char` return value.
/// SAFETY: Neovim is single-threaded; this matches C's `static char r[3]`.
static mut GET_DIGRAPH_FOR_CHAR_BUF: [u8; 3] = [0u8; 3];

/// Find a digraph for a given character value and return its display string.
///
/// Direct replacement for C's `get_digraph_for_char`.
/// Returns pointer to a static buffer `"{c1}{c2}\0"`, or NULL if not found.
///
/// # Safety
/// Must only be called from Neovim's single main thread.
#[export_name = "get_digraph_for_char"]
pub unsafe extern "C" fn rs_get_digraph_for_char_export(val_arg: c_int) -> *mut c_char {
    let mut char1: u8 = 0;
    let mut char2: u8 = 0;

    if crate::register::rs_get_digraph_for_char(val_arg, &raw mut char1, &raw mut char2) != 0 {
        let ptr = std::ptr::addr_of_mut!(GET_DIGRAPH_FOR_CHAR_BUF);
        (*ptr)[0] = char1;
        (*ptr)[1] = char2;
        (*ptr)[2] = 0;
        (*ptr).as_mut_ptr().cast::<c_char>()
    } else {
        std::ptr::null_mut()
    }
}

/// Check the characters are valid for a digraph.
///
/// Direct replacement for C's `check_digraph_chars_valid`.
/// Displays an error message and returns false if invalid.
///
/// # Safety
/// Must only be called from Neovim's single main thread.
#[export_name = "check_digraph_chars_valid"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_check_digraph_chars_valid_export(char1: c_int, char2: c_int) -> bool {
    let result = unsafe { rs_check_digraph_chars_valid(char1, char2) };
    match result {
        1 => {
            // char2 is 0 -- digraph must be two characters
            let mut msg = [0u8; 7];
            let len = unsafe { utf_char2bytes(char1, msg.as_mut_ptr().cast::<c_char>()) };
            #[allow(clippy::cast_sign_loss)]
            if (len as usize) < msg.len() {
                msg[len as usize] = 0;
            }
            let fmt =
                unsafe { nvim_gettext(E_DIGRAPH_MUST_BE_TWO_CHARS.as_ptr().cast::<c_char>()) };
            unsafe { semsg(fmt, msg.as_ptr()) };
            false
        }
        2 => {
            // ESC not allowed
            let msg = unsafe { nvim_gettext(E_ESCAPE_NOT_ALLOWED.as_ptr().cast::<c_char>()) };
            unsafe { emsg(msg) };
            false
        }
        _ => true,
    }
}

/// Add the digraphs in the argument to the digraph table.
///
/// Direct replacement for C's `putdigraph`.
///
/// # Safety
/// `str` must be a valid C string pointer.
#[export_name = "putdigraph"]
pub unsafe extern "C" fn rs_putdigraph_export(str: *mut c_char) {
    use crate::parse::{rs_putdigraph, PutdigraphResult};

    let mut result = PutdigraphResult {
        error_code: 0,
        char1: 0,
        char2: 0,
    };
    let mut str_ptr = str;

    if unsafe { rs_putdigraph(&raw mut str_ptr, &raw mut result) } == 0 {
        match result.error_code {
            1 => unsafe {
                rs_check_digraph_chars_valid_export(result.char1, result.char2);
            },
            2 => {
                let msg = unsafe { nvim_gettext(e_number_exp.as_ptr().cast::<c_char>()) };
                unsafe { emsg(msg) };
            }
            _ => {}
        }
    }
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
