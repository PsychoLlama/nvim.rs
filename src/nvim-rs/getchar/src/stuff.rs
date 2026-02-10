//! Stuffbuffer and special key handling
//!
//! This module provides Rust implementations for stuffing characters
//! into the typeahead buffer and handling special key sequences.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use std::ffi::c_int;

// =============================================================================
// Special Key Constants
// =============================================================================

/// K_SPECIAL byte that introduces a special key sequence (0x80)
pub const K_SPECIAL: u8 = 0x80;

/// KS_SPECIAL - used with KE_FILLER for literal K_SPECIAL
pub const KS_SPECIAL: u8 = 254;

/// KS_ZERO - used with KE_FILLER for NUL character
pub const KS_ZERO: u8 = 255;

/// KS_MODIFIER - indicates a modifier key follows
pub const KS_MODIFIER: u8 = 252;

/// KS_EXTRA - indicates an extra key code follows
pub const KS_EXTRA: u8 = 253;

/// KE_FILLER - filler byte for special sequences
pub const KE_FILLER: u8 = b'X';

/// KE_IGNORE - special key to ignore
pub const KE_IGNORE: u8 = 4;

/// NUL character
pub const NUL: u8 = 0;

/// Carriage return
pub const CAR: c_int = 0x0d;

/// Newline
pub const NL: c_int = 0x0a;

/// Escape
pub const ESC: c_int = 0x1b;

/// Tab
pub const TAB: c_int = 0x09;

/// DEL character
pub const DEL: c_int = 0x7f;

/// Ctrl-V
pub const CTRL_V: c_int = 0x16;

// =============================================================================
// Special Key Encoding
// =============================================================================

/// Check if a character is a special key (negative value)
#[must_use]
pub const fn is_special(c: c_int) -> bool {
    c < 0
}

/// Encode two bytes into a special key code
#[must_use]
pub const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -(a + (b << 8) + 0x100)
}

/// Get the first termcap byte from a special key code
#[must_use]
pub const fn key2termcap0(x: c_int) -> c_int {
    ((-1 - x) & 0xff) as c_int
}

/// Get the second termcap byte from a special key code
#[must_use]
pub const fn key2termcap1(x: c_int) -> c_int {
    (((-1 - x) >> 8) & 0xff) as c_int
}

/// Convert KS_* and KE_* values to a special key
///
/// Note: This handles KS_SPECIAL and KS_ZERO specially:
/// - TO_SPECIAL(KS_SPECIAL, KE_FILLER) returns K_SPECIAL (literal 0x80)
/// - TO_SPECIAL(KS_ZERO, KE_FILLER) returns NUL (literal 0x00)
#[must_use]
pub const fn to_special(a: c_int, b: c_int) -> c_int {
    if a == KS_SPECIAL as c_int {
        K_SPECIAL as c_int
    } else if a == KS_ZERO as c_int {
        NUL as c_int
    } else {
        termcap2key(a, b)
    }
}

/// Get the second byte (KS_*) for an internal special key code
#[must_use]
pub const fn k_second(c: c_int) -> u8 {
    key2termcap0(c) as u8
}

/// Get the third byte (KE_*) for an internal special key code
#[must_use]
pub const fn k_third(c: c_int) -> u8 {
    key2termcap1(c) as u8
}

// =============================================================================
// Buffer for encoding special characters
// =============================================================================

/// Maximum bytes needed for a character in stuffbuffer
/// (K_SPECIAL + KS_* + KE_* = 3 for special, up to 4 for UTF-8)
pub const CHAR_BUF_SIZE: usize = 6;

/// Encode a character for the stuffbuffer.
///
/// Handles special keys, NUL, K_SPECIAL, and UTF-8 multibyte characters.
/// Returns the number of bytes written to the buffer.
///
/// # Arguments
/// * `c` - The character to encode
/// * `buf` - Buffer to write encoded bytes to (must be at least CHAR_BUF_SIZE)
///
/// # Returns
/// Number of bytes written to buffer
pub fn encode_char(c: c_int, buf: &mut [u8]) -> usize {
    if is_special(c) || c == K_SPECIAL as c_int || c == NUL as c_int {
        buf[0] = K_SPECIAL;
        if is_special(c) {
            buf[1] = k_second(c);
            buf[2] = k_third(c);
        } else if c == NUL as c_int {
            buf[1] = KS_ZERO;
            buf[2] = KE_FILLER;
        } else {
            // c == K_SPECIAL
            buf[1] = KS_SPECIAL;
            buf[2] = KE_FILLER;
        }
        3
    } else if c < 0x80 {
        // ASCII character
        buf[0] = c as u8;
        1
    } else {
        // UTF-8 multibyte character
        utf_char2bytes(c, buf)
    }
}

/// Convert a Unicode codepoint to UTF-8 bytes
fn utf_char2bytes(c: c_int, buf: &mut [u8]) -> usize {
    let c = c as u32;
    if c < 0x80 {
        buf[0] = c as u8;
        1
    } else if c < 0x800 {
        buf[0] = (0xc0 | (c >> 6)) as u8;
        buf[1] = (0x80 | (c & 0x3f)) as u8;
        2
    } else if c < 0x10000 {
        buf[0] = (0xe0 | (c >> 12)) as u8;
        buf[1] = (0x80 | ((c >> 6) & 0x3f)) as u8;
        buf[2] = (0x80 | (c & 0x3f)) as u8;
        3
    } else {
        buf[0] = (0xf0 | (c >> 18)) as u8;
        buf[1] = (0x80 | ((c >> 12) & 0x3f)) as u8;
        buf[2] = (0x80 | ((c >> 6) & 0x3f)) as u8;
        buf[3] = (0x80 | (c & 0x3f)) as u8;
        4
    }
}

// =============================================================================
// C FFI Accessor Functions
// =============================================================================

extern "C" {
    /// Get typeahead_char value
    fn nvim_get_typeahead_char() -> c_int;
    /// Set typeahead_char value
    fn nvim_set_typeahead_char(val: c_int);
    /// Get emsg_silent
    fn nvim_get_emsg_silent() -> c_int;
    /// Call flush_buffers(flush_type)
    fn nvim_call_flush_buffers(flush_type: c_int);
    /// Call vim_beep(flag)
    fn nvim_call_vim_beep(flag: c_int);
}

use crate::buffheader;

/// Convert a C pointer + len to a slice. If len < 0, treats s as NUL-terminated.
const unsafe fn ptr_to_slice<'a>(s: *const u8, len: isize) -> &'a [u8] {
    if len < 0 {
        let mut end = s;
        while *end != 0 {
            end = end.add(1);
        }
        std::slice::from_raw_parts(s, end.offset_from(s) as usize)
    } else {
        std::slice::from_raw_parts(s, len as usize)
    }
}

// =============================================================================
// Stuff Buffer Operations (readbuf1)
// =============================================================================

/// Append a string to the stuff buffer (readbuf1).
///
/// # Safety
/// `s` must be a valid pointer to a string of at least `len` bytes,
/// or if `len` is -1, must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn rs_stuffReadbuff(s: *const u8, len: isize) {
    let slice = ptr_to_slice(s, len);
    buffheader::readbuf1().add(slice);
}

/// Append a character to the stuff buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_stuffcharReadbuff(c: c_int) {
    buffheader::readbuf1().add_char(c);
}

/// Append a number to the stuff buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_stuffnumReadbuff(n: c_int) {
    buffheader::readbuf1().add_num(n);
}

/// Append a string to the redo stuff buffer (readbuf2).
///
/// # Safety
/// `s` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_stuffRedoReadbuff(s: *const u8) {
    let slice = ptr_to_slice(s, -1);
    buffheader::readbuf2().add(slice);
}

// =============================================================================
// Redo Buffer Operations
// =============================================================================

/// Append a string to the redo buffer.
///
/// Does nothing if block_redo is true.
///
/// # Safety
/// `s` must be a valid pointer to a string of at least `len` bytes,
/// or if `len` is -1, must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn rs_AppendToRedobuff(s: *const u8, len: isize) {
    if buffheader::is_block_redo() {
        return;
    }
    let slice = ptr_to_slice(s, len);
    buffheader::redobuff().add(slice);
}

/// Append a character to the redo buffer.
///
/// Does nothing if block_redo is true.
#[no_mangle]
pub unsafe extern "C" fn rs_AppendCharToRedobuff(c: c_int) {
    if !buffheader::is_block_redo() {
        buffheader::redobuff().add_char(c);
    }
}

/// Append a number to the redo buffer.
///
/// Does nothing if block_redo is true.
#[no_mangle]
pub unsafe extern "C" fn rs_AppendNumberToRedobuff(n: c_int) {
    if !buffheader::is_block_redo() {
        buffheader::redobuff().add_num(n);
    }
}

/// Get the typeahead character that won't be flushed.
#[no_mangle]
pub unsafe extern "C" fn rs_get_typeahead_char() -> c_int {
    nvim_get_typeahead_char()
}

/// Set the typeahead character that won't be flushed.
#[no_mangle]
pub unsafe extern "C" fn rs_set_typeahead_char(c: c_int) {
    nvim_set_typeahead_char(c);
}

/// Encode a character for the stuffbuffer.
///
/// Returns the number of bytes written to buf.
///
/// # Safety
/// `buf` must point to a buffer of at least 6 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_encode_char_for_stuff(c: c_int, buf: *mut u8) -> c_int {
    let slice = std::slice::from_raw_parts_mut(buf, CHAR_BUF_SIZE);
    encode_char(c, slice) as c_int
}

/// Check if a character needs special encoding for stuffbuffer.
///
/// Returns true for special keys, NUL, and K_SPECIAL.
#[no_mangle]
pub extern "C" fn rs_needs_special_encoding(c: c_int) -> c_int {
    c_int::from(is_special(c) || c == K_SPECIAL as c_int || c == NUL as c_int)
}

/// FLUSH_MINIMAL constant (matches C enum)
const FLUSH_MINIMAL: c_int = 0;

/// kOptBoFlagError constant (from generated option_vars.h)
const K_OPT_BO_FLAG_ERROR: c_int = 0x40;

/// Set typeahead character that won't be flushed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_typeahead_noflush(c: c_int) {
    nvim_set_typeahead_char(c);
}

/// Flush map and typeahead buffers and give a warning for an error.
///
/// # Safety
/// Calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_beep_flush() {
    if nvim_get_emsg_silent() == 0 {
        nvim_call_flush_buffers(FLUSH_MINIMAL);
        nvim_call_vim_beep(K_OPT_BO_FLAG_ERROR);
    }
}

/// Stop redo insert mode (unblock redo buffer).
#[no_mangle]
pub unsafe extern "C" fn rs_stop_redo_ins() {
    buffheader::set_block_redo(false);
}

// Note: rs_to_special and rs_is_special are already exported from input.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_special() {
        assert!(is_special(-1));
        assert!(is_special(-100));
        assert!(!is_special(0));
        assert!(!is_special(65)); // 'A'
    }

    #[test]
    fn test_termcap_roundtrip() {
        // Test with valid KS_*/KE_* style values (typically >= 4)
        // Note: (0, 0) doesn't roundtrip correctly - this is expected per C macro behavior
        for a in 1..10 {
            for b in 0..10 {
                let key = termcap2key(a, b);
                assert_eq!(key2termcap0(key), a);
                assert_eq!(key2termcap1(key), b);
            }
        }
    }

    #[test]
    fn test_to_special_ks_special() {
        let result = to_special(KS_SPECIAL as c_int, KE_FILLER as c_int);
        assert_eq!(result, K_SPECIAL as c_int);
    }

    #[test]
    fn test_to_special_ks_zero() {
        let result = to_special(KS_ZERO as c_int, KE_FILLER as c_int);
        assert_eq!(result, NUL as c_int);
    }

    #[test]
    fn test_encode_char_ascii() {
        let mut buf = [0u8; CHAR_BUF_SIZE];
        let len = encode_char(b'A' as c_int, &mut buf);
        assert_eq!(len, 1);
        assert_eq!(buf[0], b'A');
    }

    #[test]
    fn test_encode_char_nul() {
        let mut buf = [0u8; CHAR_BUF_SIZE];
        let len = encode_char(NUL as c_int, &mut buf);
        assert_eq!(len, 3);
        assert_eq!(buf[0], K_SPECIAL);
        assert_eq!(buf[1], KS_ZERO);
        assert_eq!(buf[2], KE_FILLER);
    }

    #[test]
    fn test_encode_char_k_special() {
        let mut buf = [0u8; CHAR_BUF_SIZE];
        let len = encode_char(K_SPECIAL as c_int, &mut buf);
        assert_eq!(len, 3);
        assert_eq!(buf[0], K_SPECIAL);
        assert_eq!(buf[1], KS_SPECIAL);
        assert_eq!(buf[2], KE_FILLER);
    }

    #[test]
    fn test_utf_char2bytes() {
        let mut buf = [0u8; 6];

        // 2-byte UTF-8 (e.g., é = U+00E9)
        let len = utf_char2bytes(0xe9, &mut buf);
        assert_eq!(len, 2);
        assert_eq!(buf[0], 0xc3);
        assert_eq!(buf[1], 0xa9);

        // 3-byte UTF-8 (e.g., € = U+20AC)
        let len = utf_char2bytes(0x20ac, &mut buf);
        assert_eq!(len, 3);
        assert_eq!(buf[0], 0xe2);
        assert_eq!(buf[1], 0x82);
        assert_eq!(buf[2], 0xac);

        // 4-byte UTF-8 (e.g., 😀 = U+1F600)
        let len = utf_char2bytes(0x1f600, &mut buf);
        assert_eq!(len, 4);
        assert_eq!(buf[0], 0xf0);
        assert_eq!(buf[1], 0x9f);
        assert_eq!(buf[2], 0x98);
        assert_eq!(buf[3], 0x80);
    }
}
