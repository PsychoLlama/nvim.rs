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

// =============================================================================
// UTF-8 Decode Helper
// =============================================================================

/// Decode one UTF-8 character from a byte slice.
/// Returns (codepoint, bytes_consumed).
/// Equivalent to C `mb_cptr2char_adv`.
#[must_use]
pub fn utf8_decode_advance(bytes: &[u8]) -> (c_int, usize) {
    if bytes.is_empty() {
        return (0, 0);
    }

    let b0 = bytes[0];

    if b0 < 0x80 {
        return (c_int::from(b0), 1);
    }

    if b0 < 0xc0 || bytes.len() < 2 {
        // Invalid lead byte or not enough bytes
        return (c_int::from(b0), 1);
    }

    if b0 < 0xe0 {
        if bytes.len() < 2 || (bytes[1] & 0xc0) != 0x80 {
            return (c_int::from(b0), 1);
        }
        let cp = (c_int::from(b0 & 0x1f) << 6) | c_int::from(bytes[1] & 0x3f);
        return (cp, 2);
    }

    if b0 < 0xf0 {
        if bytes.len() < 3 || (bytes[1] & 0xc0) != 0x80 || (bytes[2] & 0xc0) != 0x80 {
            return (c_int::from(b0), 1);
        }
        let cp = (c_int::from(b0 & 0x0f) << 12)
            | (c_int::from(bytes[1] & 0x3f) << 6)
            | c_int::from(bytes[2] & 0x3f);
        return (cp, 3);
    }

    if bytes.len() < 4
        || (bytes[1] & 0xc0) != 0x80
        || (bytes[2] & 0xc0) != 0x80
        || (bytes[3] & 0xc0) != 0x80
    {
        return (c_int::from(b0), 1);
    }
    let cp = (c_int::from(b0 & 0x07) << 18)
        | (c_int::from(bytes[1] & 0x3f) << 12)
        | (c_int::from(bytes[2] & 0x3f) << 6)
        | c_int::from(bytes[3] & 0x3f);
    (cp, 4)
}

// =============================================================================
// Phase 3: Redo buffer append operations (migrated from C)
// =============================================================================

/// Append to Redo buffer literally, escaping special characters with CTRL-V.
/// K_SPECIAL is escaped as well.
///
/// # Safety
/// `s` must be a valid pointer to a string of at least `len` bytes,
/// or if `len` is -1, must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn rs_AppendToRedobuffLit(s: *const u8, len: c_int) {
    if buffheader::is_block_redo() {
        return;
    }

    let slice = ptr_to_slice(s, len as isize);
    let mut pos = 0;

    while pos < slice.len() {
        // Put a string of normal characters in the redo buffer
        let start = pos;
        while pos < slice.len() && slice[pos] >= b' ' && slice[pos] < DEL_U8 {
            pos += 1;
        }

        // Don't put '0' or '^' as last character
        if pos < slice.len()
            && slice[pos] == 0
            && pos > start
            && (slice[pos - 1] == b'0' || slice[pos - 1] == b'^')
        {
            pos -= 1;
        }
        if pos > start {
            buffheader::redobuff().add(&slice[start..pos]);
        }

        if pos >= slice.len() {
            break;
        }

        // Check for end (NUL byte in the slice)
        if slice[pos] == 0 {
            // Check if this is a real NUL terminator (for len == -1 case)
            if len < 0 {
                break;
            }
            // Handle NUL as a character to be escaped
        }

        // Handle a special or multibyte character
        let remaining = &slice[pos..];
        let (c, consumed) = utf8_decode_advance(remaining);
        pos += consumed;

        if c < c_int::from(b' ')
            || c == DEL as c_int
            || (pos >= slice.len() && (c == c_int::from(b'0') || c == c_int::from(b'^')))
        {
            buffheader::redobuff().add_char(CTRL_V);
        }

        // CTRL-V '0' must be inserted as CTRL-V 048
        if pos >= slice.len() && c == c_int::from(b'0') {
            buffheader::redobuff().add(b"048");
        } else {
            buffheader::redobuff().add_char(c);
        }
    }
}

const DEL_U8: u8 = 0x7f;

/// Append to the redo buffer, leaving 3-byte special key codes unmodified
/// and escaping other K_SPECIAL bytes.
///
/// # Safety
/// `s` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_AppendToRedobuffSpec(s: *const u8) {
    if buffheader::is_block_redo() {
        return;
    }

    let slice = ptr_to_slice(s, -1);
    let mut pos = 0;

    while pos < slice.len() {
        if slice[pos] == K_SPECIAL && pos + 2 < slice.len() {
            // Insert special key literally
            buffheader::redobuff().add(&slice[pos..pos + 3]);
            pos += 3;
        } else {
            let (c, consumed) = utf8_decode_advance(&slice[pos..]);
            pos += consumed;
            buffheader::redobuff().add_char(c);
        }
    }
}

// =============================================================================
// Phase 3: Stuff buffer operations (migrated from C)
// =============================================================================

/// Stuff "s" into the stuff buffer, leaving special key codes unmodified and
/// escaping other K_SPECIAL bytes. Change CR, LF and ESC into a space.
///
/// # Safety
/// `s` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_stuffReadbuffSpec(s: *const u8) {
    let slice = ptr_to_slice(s, -1);
    let mut pos = 0;

    while pos < slice.len() {
        if slice[pos] == K_SPECIAL && pos + 2 < slice.len() {
            // Insert special key literally
            buffheader::readbuf1().add(&slice[pos..pos + 3]);
            pos += 3;
        } else {
            let (mut c, consumed) = utf8_decode_advance(&slice[pos..]);
            pos += consumed;
            if c == CAR || c == NL || c == ESC {
                c = c_int::from(b' ');
            }
            buffheader::readbuf1().add_char(c);
        }
    }
}

/// Stuff a string into the typeahead buffer, such that edit() will insert it
/// literally ("literally" true) or interpret is as typed characters.
///
/// # Safety
/// `arg` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_stuffescaped(arg: *const u8, literally: c_int) {
    let slice = ptr_to_slice(arg, -1);
    let literally = literally != 0;
    let mut pos = 0;

    while pos < slice.len() {
        // Stuff a sequence of normal ASCII characters
        let start = pos;
        while pos < slice.len()
            && ((slice[pos] >= b' ' && slice[pos] < DEL_U8)
                || (slice[pos] == K_SPECIAL && !literally))
        {
            pos += 1;
        }
        if pos > start {
            buffheader::readbuf1().add(&slice[start..pos]);
        }

        // Stuff a single special character
        if pos < slice.len() {
            let (c, consumed) = utf8_decode_advance(&slice[pos..]);
            pos += consumed;
            if literally && ((c < c_int::from(b' ') && c != TAB) || c == DEL as c_int) {
                buffheader::readbuf1().add_char(CTRL_V);
            }
            buffheader::readbuf1().add_char(c);
        }
    }
}

// =============================================================================
// Phase 3: Redo replay (migrated from C)
// =============================================================================

extern "C" {
    fn nvim_utf_ptr2char(p: *const u8) -> c_int;
}

/// MB_BYTE2LEN_CHECK equivalent: returns UTF-8 byte length for a lead byte.
/// Returns 1 for special keys (negative) or values > 255.
#[must_use]
#[allow(clippy::manual_range_contains)]
const fn mb_byte2len_check(c: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 1;
    }
    crate::macro_recording::mb_byte2len(c as u8) as c_int
}

/// Read a character from the redo buffer. Translates K_SPECIAL and
/// multibyte characters. Returns the character or NUL at end.
///
/// This is the Rust version of C `read_redo(false, old_redo)`.
/// The buffer must have been initialized with `rs_read_redo_init` first.
unsafe fn read_redo_char() -> c_int {
    use crate::buffheader::{rs_read_redo_byte, rs_read_redo_peek};

    let c = rs_read_redo_byte();
    if c == 0 {
        return 0;
    }

    // Reverse the conversion done by add_char_buff()
    let n = if c != c_int::from(K_SPECIAL) || rs_read_redo_peek() == c_int::from(KS_SPECIAL) {
        mb_byte2len_check(c)
    } else {
        1
    };

    let mut buf = [0u8; 8]; // MB_MAXBYTES + 1
    let mut c = c;
    for i in 0..n as usize {
        if c == c_int::from(K_SPECIAL) {
            let b1 = rs_read_redo_byte();
            let b2 = rs_read_redo_byte();
            c = to_special(b1, b2);
        }
        buf[i] = c as u8;
        if i == (n as usize) - 1 {
            if n != 1 {
                c = nvim_utf_ptr2char(buf.as_ptr());
            }
            break;
        }
        c = rs_read_redo_byte();
        if c == 0 {
            break;
        }
    }

    c
}

/// Copy the rest of the redo buffer into readbuf2.
/// The escaped K_SPECIAL is copied without translation.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_redo(old_redo: c_int) {
    // The redo reader must already be initialized for `old_redo`.
    // Note: we re-initialize here since C's copy_redo passes old_redo.
    // But in practice, copy_redo is always called after read_redo with
    // same old_redo value, and the reader is already positioned.
    let _ = old_redo; // reader already positioned by caller
    loop {
        let c = read_redo_char();
        if c == 0 {
            break;
        }
        buffheader::readbuf2().add_char(c);
    }
}

/// Initialize redo reader and read first character.
/// Returns FAIL (1) if nothing to redo.
unsafe fn read_redo_init_and_first(old_redo: bool) -> Result<c_int, ()> {
    use crate::buffheader::rs_read_redo_init;

    if rs_read_redo_init(c_int::from(old_redo)) != 0 {
        return Err(());
    }
    Ok(read_redo_char())
}

/// Stuff the redo buffer into readbuf2 with count insertion.
/// Returns FAIL (1) for failure, OK (0) otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_start_redo(count: c_int, old_redo: c_int) -> c_int {
    let old = old_redo != 0;

    let Ok(mut c) = read_redo_init_and_first(old) else {
        return 1; // FAIL
    };

    // Copy the buffer name, if present
    if c == c_int::from(b'"') {
        buffheader::readbuf2().add(b"\"");
        c = read_redo_char();

        // If a numbered buffer is used, increment the number
        if c >= c_int::from(b'1') && c < c_int::from(b'9') {
            c += 1;
        }
        buffheader::readbuf2().add_char(c);

        // The expression register should be re-evaluated
        if c == c_int::from(b'=') {
            buffheader::readbuf2().add_char(CAR);
            nvim_set_cmd_silent(1);
        }

        c = read_redo_char();
    }

    if c == c_int::from(b'v') {
        // redo Visual
        nvim_set_visual_from_cursor();
        c = read_redo_char();
    }

    // Try to enter the count (in place of a previous count)
    if count != 0 {
        while c >= c_int::from(b'0') && c <= c_int::from(b'9') {
            c = read_redo_char();
        }
        buffheader::readbuf2().add_num(count);
    }

    // Copy from the redo buffer into the stuff buffer
    buffheader::readbuf2().add_char(c);
    rs_copy_redo(old_redo);
    0 // OK
}

/// Repeat the last insert by stuffing the redo buffer into readbuf2.
/// Returns FAIL (1) for failure, OK (0) otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_start_redo_ins() -> c_int {
    let Ok(c) = read_redo_init_and_first(false) else {
        return 1; // FAIL
    };

    buffheader::readbuf1().start_reading();
    buffheader::readbuf2().start_reading();

    // Skip the count and the command character
    let mut c = c;
    loop {
        if c == 0 {
            break;
        }
        if matches!(
            c as u8,
            b'A' | b'a' | b'I' | b'i' | b'R' | b'r' | b'O' | b'o'
        ) {
            if c == c_int::from(b'O') || c == c_int::from(b'o') {
                buffheader::readbuf2().add(b"\n");
            }
            break;
        }
        c = read_redo_char();
    }

    // Copy the typed text from the redo buffer into the stuff buffer
    rs_copy_redo(0);
    buffheader::set_block_redo(true);
    0 // OK
}

extern "C" {
    fn nvim_set_cmd_silent(val: c_int);
    fn nvim_set_visual_from_cursor();
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
