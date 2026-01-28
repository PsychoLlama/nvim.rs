//! Sign text utilities
//!
//! This module provides sign text handling utilities including:
//! - Converting sign text (schar_T array) to display strings
//! - Initializing sign text from user input
//! - UTF-8 cell width handling for sign display

use std::ffi::{c_char, c_int};

use crate::SIGN_WIDTH;

/// schar_T type (matches C definition)
pub type ScharT = u32;

/// Maximum bytes per schar (4 bytes for inline, more for cached glyphs)
pub const MAX_SCHAR_SIZE: usize = 32;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    /// Get the UTF-8 bytes for an schar, with NUL terminator
    fn rs_schar_get(buf_out: *mut c_char, sc: ScharT) -> usize;

    /// Convert a string to an schar (for single character)
    fn utfc_ptr2schar(s: *const c_char, c: *mut c_int) -> ScharT;

    /// Get the byte length of a UTF-8 character
    fn utfc_ptr2len(s: *const c_char) -> c_int;

    /// Get the display cell width of a UTF-8 character
    fn utf_ptr2cells(s: *const c_char) -> c_int;

    /// Check if a character is printable
    fn vim_isprintc(c: c_int) -> c_int;
}

// =============================================================================
// Sign Text Description
// =============================================================================

/// Convert sign text (array of schar_T) to a UTF-8 string.
///
/// This is the Rust implementation of `describe_sign_text()` from sign.c.
/// The output buffer must be at least `SIGN_WIDTH * MAX_SCHAR_SIZE` bytes.
///
/// # Safety
/// - `buf` must be valid for writing at least `buflen` bytes
/// - `sign_text` must be valid for reading `SIGN_WIDTH` schar_T values
#[allow(clippy::cast_possible_truncation)]
pub unsafe fn describe_sign_text_impl(buf: &mut [u8], sign_text: &[ScharT; SIGN_WIDTH]) -> usize {
    let mut pos = 0;

    for &sc in sign_text {
        if sc == 0 {
            break;
        }

        // Get UTF-8 bytes for this schar
        let remaining = buf.len().saturating_sub(pos);
        if remaining < MAX_SCHAR_SIZE {
            break;
        }

        let len = rs_schar_get(buf.as_mut_ptr().add(pos).cast(), sc);
        if len == 0 {
            break;
        }
        pos += len;
    }

    pos
}

/// FFI export: Describe sign text to buffer.
///
/// Converts sign text (array of schar_T) to a NUL-terminated UTF-8 string.
/// Returns the number of bytes written (not including NUL).
///
/// # Safety
/// - `buf` must be valid for writing at least `buflen` bytes
/// - `sign_text` must be a valid pointer to SIGN_WIDTH schar_T values
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_describe_sign_text(
    buf: *mut c_char,
    buflen: usize,
    sign_text: *const ScharT,
) -> usize {
    if buf.is_null() || sign_text.is_null() || buflen == 0 {
        return 0;
    }

    // Create array from pointer
    let text_array: [ScharT; SIGN_WIDTH] = [*sign_text, *sign_text.add(1)];

    // Create mutable slice
    let buf_slice = std::slice::from_raw_parts_mut(buf.cast::<u8>(), buflen);

    let len = describe_sign_text_impl(buf_slice, &text_array);

    // NUL-terminate if there's space
    if len < buflen {
        *buf.add(len) = 0;
    }

    len
}

// =============================================================================
// Sign Text Initialization
// =============================================================================

/// Result of sign text initialization.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignTextResult {
    /// Success
    Ok = 0,
    /// Invalid (non-printable character or too wide)
    Invalid = 1,
}

/// Initialize sign text from a user-provided string.
///
/// This validates and converts the input text to schar_T array format.
/// Sign text must be 0-2 display cells wide with printable characters only.
///
/// # Safety
/// - `text` must be a valid NUL-terminated C string
/// - `sign_text` must be valid for writing SIGN_WIDTH schar_T values
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub unsafe fn init_sign_text_impl(
    sign_text: &mut [ScharT; SIGN_WIDTH],
    text: *const c_char,
    remove_backslash: bool,
) -> SignTextResult {
    if text.is_null() {
        sign_text[0] = 0;
        sign_text[1] = 0;
        return SignTextResult::Ok;
    }

    // Find end of text
    let mut text_len = 0;
    let mut p = text;
    while *p != 0 {
        text_len += 1;
        p = p.add(1);
    }

    // Copy to temporary mutable buffer if we need to process backslashes
    let mut temp_buf = [0u8; 256];

    let process_ptr: *const c_char = if remove_backslash && text_len > 0 {
        // Copy to temp buffer
        let copy_len = text_len.min(temp_buf.len() - 1);
        std::ptr::copy_nonoverlapping(text.cast::<u8>(), temp_buf.as_mut_ptr(), copy_len);
        temp_buf[copy_len] = 0;

        // Remove backslashes (allows escaping space)
        let mut src = 0;
        let mut dst = 0;
        while src < copy_len {
            if temp_buf[src] == b'\\' && src + 1 < copy_len {
                src += 1; // Skip backslash
            }
            temp_buf[dst] = temp_buf[src];
            dst += 1;
            src += 1;
        }
        temp_buf[dst] = 0;
        temp_buf.as_ptr().cast()
    } else {
        text
    };

    // Count cells and convert to schar_T
    let mut cells = 0;
    let mut s = process_ptr;

    while *s != 0 && cells < SIGN_WIDTH {
        let mut codepoint: c_int = 0;
        let sc = utfc_ptr2schar(s, std::ptr::addr_of_mut!(codepoint));

        // Check for non-printable characters
        if vim_isprintc(codepoint) == 0 {
            return SignTextResult::Invalid;
        }

        sign_text[cells] = sc;

        let width = utf_ptr2cells(s) as usize;
        if width == 2 && cells + 1 < SIGN_WIDTH {
            // Double-width character: next cell is empty
            sign_text[cells + 1] = 0;
        }

        cells += width;
        let char_len = utfc_ptr2len(s);
        s = s.add(char_len as usize);
    }

    // Check if we processed all input and didn't exceed width
    if *s != 0 || cells > SIGN_WIDTH {
        return SignTextResult::Invalid;
    }

    // Pad remaining cells
    if cells < 1 {
        sign_text[0] = 0;
        sign_text[1] = 0;
    } else if cells == 1 {
        // Single-width character: pad with space
        sign_text[1] = schar_from_ascii(b' ');
    }

    SignTextResult::Ok
}

/// FFI export: Initialize sign text from string.
///
/// Returns 0 (OK) on success, 1 (FAIL) on error.
///
/// # Safety
/// - `sign_text` must be valid for writing SIGN_WIDTH schar_T values
/// - `text` must be a valid NUL-terminated C string or null
#[no_mangle]
pub unsafe extern "C" fn rs_init_sign_text(
    sign_text: *mut ScharT,
    text: *const c_char,
    remove_backslash: c_int,
) -> c_int {
    if sign_text.is_null() {
        return 1; // FAIL
    }

    let mut text_array: [ScharT; SIGN_WIDTH] = [0; SIGN_WIDTH];
    let result = init_sign_text_impl(&mut text_array, text, remove_backslash != 0);

    // Copy result back
    *sign_text = text_array[0];
    *sign_text.add(1) = text_array[1];

    match result {
        SignTextResult::Ok => 0,      // OK
        SignTextResult::Invalid => 1, // FAIL
    }
}

// =============================================================================
// Schar Utilities
// =============================================================================

/// Create an schar_T from an ASCII byte.
///
/// This matches the C `schar_from_ascii` macro behavior.
#[inline]
pub const fn schar_from_ascii(c: u8) -> ScharT {
    #[cfg(target_endian = "big")]
    {
        (c as ScharT) << 24
    }
    #[cfg(target_endian = "little")]
    {
        c as ScharT
    }
}

/// Check if an schar represents a space character.
#[inline]
pub const fn schar_is_space(sc: ScharT) -> bool {
    sc == schar_from_ascii(b' ')
}

/// Check if an schar is empty (zero).
#[inline]
pub const fn schar_is_empty(sc: ScharT) -> bool {
    sc == 0
}

// =============================================================================
// Sign Text Validation
// =============================================================================

/// Check if sign text is valid (has content).
#[inline]
pub const fn sign_text_has_content(sign_text: &[ScharT; SIGN_WIDTH]) -> bool {
    sign_text[0] != 0
}

/// FFI export: Check if sign text has content.
///
/// # Safety
/// - `sign_text` must be valid for reading SIGN_WIDTH schar_T values
#[no_mangle]
pub unsafe extern "C" fn rs_sign_text_has_content(sign_text: *const ScharT) -> c_int {
    if sign_text.is_null() {
        return 0;
    }
    c_int::from(*sign_text != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schar_from_ascii() {
        let space = schar_from_ascii(b' ');
        assert_ne!(space, 0);

        let a = schar_from_ascii(b'A');
        assert_ne!(a, 0);
        assert_ne!(a, space);
    }

    #[test]
    fn test_schar_is_space() {
        assert!(schar_is_space(schar_from_ascii(b' ')));
        assert!(!schar_is_space(schar_from_ascii(b'A')));
        assert!(!schar_is_space(0));
    }

    #[test]
    fn test_schar_is_empty() {
        assert!(schar_is_empty(0));
        assert!(!schar_is_empty(schar_from_ascii(b' ')));
        assert!(!schar_is_empty(schar_from_ascii(b'A')));
    }

    #[test]
    fn test_sign_text_has_content() {
        let empty: [ScharT; SIGN_WIDTH] = [0, 0];
        assert!(!sign_text_has_content(&empty));

        let with_content: [ScharT; SIGN_WIDTH] = [schar_from_ascii(b'>'), schar_from_ascii(b'>')];
        assert!(sign_text_has_content(&with_content));
    }

    #[test]
    fn test_sign_text_result() {
        assert_eq!(SignTextResult::Ok as c_int, 0);
        assert_eq!(SignTextResult::Invalid as c_int, 1);
    }
}
