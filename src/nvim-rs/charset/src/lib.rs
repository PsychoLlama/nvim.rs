//! Character set utilities for Neovim.
//!
//! This crate provides FFI-compatible character classification and
//! string skipping utilities.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::ffi::c_char;
use std::ffi::c_int;

// ASCII character classification helpers (inline, pure functions)

/// Check if character is ASCII whitespace (' ' or '\t')
#[inline]
const fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if character is ASCII digit ('0'-'9')
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if character is binary digit ('0' or '1')
#[inline]
const fn ascii_isbdigit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Check if character is hex digit ('0'-'9', 'A'-'F', 'a'-'f')
#[inline]
const fn ascii_isxdigit(c: u8) -> bool {
    (c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f')
}

// ============================================================================
// Skip functions - Skip over characters matching certain criteria
// ============================================================================

/// Skip over ' ' and '\t' characters.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipwhite(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while ascii_iswhite(*ptr as u8) {
        ptr = ptr.add(1);
    }
    ptr
}

/// Skip over ' ' and '\t' characters up to `len` bytes.
///
/// # Safety
/// The pointer must be valid and accessible for at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_skipwhite_len(p: *const c_char, len: usize) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    let mut remaining = len;
    while remaining > 0 && ascii_iswhite(*ptr as u8) {
        ptr = ptr.add(1);
        remaining -= 1;
    }
    ptr
}

/// Skip over digit characters ('0'-'9').
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipdigits(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while ascii_isdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip over binary digit characters ('0' or '1').
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipbin(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while ascii_isbdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip over hex digit characters ('0'-'9', 'A'-'F', 'a'-'f').
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiphex(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while ascii_isxdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip to the next digit character or end of string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptodigit(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while *p != 0 && !ascii_isdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip to the next binary digit character or end of string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptobin(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while *p != 0 && !ascii_isbdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip to the next hex digit character or end of string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptohex(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while *p != 0 && !ascii_isxdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip over text until ' ', '\t', or NUL.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptowhite(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while *ptr != 0 && *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
        ptr = ptr.add(1);
    }
    ptr
}

// Ctrl_V constant (ASCII 22)
const CTRL_V: u8 = 22;

/// Skip to whitespace, respecting escaped characters.
/// Like `skiptowhite()`, but also skips escaped chars (backslash or `Ctrl-V`).
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptowhite_esc(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while *ptr != 0 && *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
        // If we see a backslash or Ctrl-V, and the next char is not NUL, skip it
        if (*ptr == b'\\' as c_char || *ptr == CTRL_V as c_char) && *ptr.add(1) != 0 {
            ptr = ptr.add(1);
        }
        ptr = ptr.add(1);
    }
    ptr
}

/// Return the number of whitespace columns (bytes) at the start of a string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_getwhitecols(p: *const c_char) -> isize {
    if p.is_null() {
        return 0;
    }
    let result = rs_skipwhite(p);
    result.offset_from(p)
}

/// Skip over text until '\n' (newline) or NUL.
///
/// Returns a pointer to the next '\n' or the NUL terminator.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_to_newline(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while *ptr != 0 && *ptr != b'\n' as c_char {
        ptr = ptr.add(1);
    }
    ptr
}

// ============================================================================
// Line classification functions
// ============================================================================

/// Check that the string is empty or only contains whitespace (blanks/tabs).
///
/// Returns true if the line is blank (empty, whitespace only, or ends at line terminator).
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_isblankline(lbuf: *const c_char) -> bool {
    if lbuf.is_null() {
        return true;
    }
    let p = rs_skipwhite(lbuf);
    // NUL, CR, or LF all count as "blank line" (end of line or empty)
    *p == 0 || *p == b'\r' as c_char || *p == b'\n' as c_char
}

// ============================================================================
// Hex/Number conversion functions
// ============================================================================

/// Convert the lower 4 bits of a byte to its hex character.
///
/// Lower case letters are used to avoid the confusion of <F1> being 0xf1 or
/// function key 1.
///
/// Returns the hex character ('0'-'9', 'a'-'f').
#[no_mangle]
pub extern "C" fn rs_nr2hex(n: u32) -> u32 {
    let nibble = n & 0xf;
    if nibble <= 9 {
        nibble + u32::from(b'0')
    } else {
        nibble - 10 + u32::from(b'a')
    }
}

/// Return the value of a single hex character.
/// Only valid when the argument is '0'-'9', 'A'-'F', or 'a'-'f'.
///
/// Returns the numeric value (0-15) of the hex digit.
#[no_mangle]
pub extern "C" fn rs_hex2nr(c: c_int) -> c_int {
    let c = c as u8;
    if (b'a'..=b'f').contains(&c) {
        c_int::from(c - b'a' + 10)
    } else if (b'A'..=b'F').contains(&c) {
        c_int::from(c - b'A' + 10)
    } else {
        c_int::from(c.wrapping_sub(b'0'))
    }
}

/// Convert two hex characters to a byte.
///
/// Returns -1 if one of the characters is not hex.
///
/// # Safety
/// The pointer must be valid and point to at least 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_hexhex2nr(p: *const c_char) -> c_int {
    if p.is_null() {
        return -1;
    }
    let c0 = *p as u8;
    let c1 = *p.add(1) as u8;

    if !ascii_isxdigit(c0) || !ascii_isxdigit(c1) {
        return -1;
    }

    (rs_hex2nr(c_int::from(c0)) << 4) + rs_hex2nr(c_int::from(c1))
}

/// Convert a non-printable character to hex C string like "<FFFF>".
///
/// Formats the character as `<XX>` for values <= 0xFF,
/// `<XXXX>` for values <= 0xFFFF, or `<XXXXXX>` for larger values.
///
/// Returns the number of bytes written (excluding the NUL terminator).
///
/// # Safety
/// The buffer must be valid and have at least 9 bytes of space
/// (for the longest format: `<XXXXXX>\0`).
#[no_mangle]
pub unsafe extern "C" fn rs_transchar_hex(buf: *mut c_char, c: c_int) -> usize {
    if buf.is_null() {
        return 0;
    }

    let c = c as u32;
    let mut i = 0usize;

    *buf.add(i) = b'<' as c_char;
    i += 1;

    if c > 0xFF {
        if c > 0xFFFF {
            *buf.add(i) = rs_nr2hex(c >> 20) as c_char;
            i += 1;
            *buf.add(i) = rs_nr2hex(c >> 16) as c_char;
            i += 1;
        }
        *buf.add(i) = rs_nr2hex(c >> 12) as c_char;
        i += 1;
        *buf.add(i) = rs_nr2hex(c >> 8) as c_char;
        i += 1;
    }

    *buf.add(i) = rs_nr2hex(c >> 4) as c_char;
    i += 1;
    *buf.add(i) = rs_nr2hex(c) as c_char;
    i += 1;
    *buf.add(i) = b'>' as c_char;
    i += 1;
    *buf.add(i) = 0; // NUL terminator

    i
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_skipwhite() {
        unsafe {
            // Test skipping spaces
            let s = CString::new("   hello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test skipping tabs
            let s = CString::new("\t\thello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test mixed spaces and tabs
            let s = CString::new(" \t \thello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test no whitespace
            let s = CString::new("hello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test empty string
            let s = CString::new("").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, 0);

            // Test null pointer
            let result = rs_skipwhite(std::ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_skipwhite_len() {
        unsafe {
            // Test with len limit
            let s = CString::new("     hello").unwrap();
            let result = rs_skipwhite_len(s.as_ptr(), 3);
            // Should stop after 3 spaces
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 3);

            // Test with len > whitespace count
            let s = CString::new("  hello").unwrap();
            let result = rs_skipwhite_len(s.as_ptr(), 10);
            assert_eq!(*result, b'h' as c_char);

            // Test with len = 0
            let s = CString::new("  hello").unwrap();
            let result = rs_skipwhite_len(s.as_ptr(), 0);
            assert_eq!(*result, b' ' as c_char);
        }
    }

    #[test]
    fn test_skipdigits() {
        unsafe {
            let s = CString::new("12345abc").unwrap();
            let result = rs_skipdigits(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);

            let s = CString::new("abc123").unwrap();
            let result = rs_skipdigits(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);

            let s = CString::new("12345").unwrap();
            let result = rs_skipdigits(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skipbin() {
        unsafe {
            let s = CString::new("01010abc").unwrap();
            let result = rs_skipbin(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);

            let s = CString::new("01012345").unwrap();
            let result = rs_skipbin(s.as_ptr());
            assert_eq!(*result, b'2' as c_char);

            let s = CString::new("abc").unwrap();
            let result = rs_skipbin(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);
        }
    }

    #[test]
    fn test_skiphex() {
        unsafe {
            let s = CString::new("1a2b3cGHI").unwrap();
            let result = rs_skiphex(s.as_ptr());
            assert_eq!(*result, b'G' as c_char);

            let s = CString::new("ABCDEF123xyz").unwrap();
            let result = rs_skiphex(s.as_ptr());
            assert_eq!(*result, b'x' as c_char);

            let s = CString::new("xyz").unwrap();
            let result = rs_skiphex(s.as_ptr());
            assert_eq!(*result, b'x' as c_char);
        }
    }

    #[test]
    fn test_skiptodigit() {
        unsafe {
            let s = CString::new("abc123").unwrap();
            let result = rs_skiptodigit(s.as_ptr());
            assert_eq!(*result, b'1' as c_char);

            let s = CString::new("123").unwrap();
            let result = rs_skiptodigit(s.as_ptr());
            assert_eq!(*result, b'1' as c_char);

            let s = CString::new("abc").unwrap();
            let result = rs_skiptodigit(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skiptobin() {
        unsafe {
            let s = CString::new("abc0101").unwrap();
            let result = rs_skiptobin(s.as_ptr());
            assert_eq!(*result, b'0' as c_char);

            let s = CString::new("0101").unwrap();
            let result = rs_skiptobin(s.as_ptr());
            assert_eq!(*result, b'0' as c_char);

            let s = CString::new("abc").unwrap();
            let result = rs_skiptobin(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skiptohex() {
        unsafe {
            let s = CString::new("xyz1aF").unwrap();
            let result = rs_skiptohex(s.as_ptr());
            assert_eq!(*result, b'1' as c_char);

            let s = CString::new("AbCd").unwrap();
            let result = rs_skiptohex(s.as_ptr());
            assert_eq!(*result, b'A' as c_char);

            let s = CString::new("ghi").unwrap();
            let result = rs_skiptohex(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skiptowhite() {
        unsafe {
            let s = CString::new("hello world").unwrap();
            let result = rs_skiptowhite(s.as_ptr());
            assert_eq!(*result, b' ' as c_char);

            let s = CString::new("hello\tworld").unwrap();
            let result = rs_skiptowhite(s.as_ptr());
            assert_eq!(*result, b'\t' as c_char);

            let s = CString::new("hello").unwrap();
            let result = rs_skiptowhite(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_nr2hex() {
        // Test 0-9 -> '0'-'9'
        assert_eq!(rs_nr2hex(0), u32::from(b'0'));
        assert_eq!(rs_nr2hex(5), u32::from(b'5'));
        assert_eq!(rs_nr2hex(9), u32::from(b'9'));

        // Test 10-15 -> 'a'-'f' (lowercase)
        assert_eq!(rs_nr2hex(10), u32::from(b'a'));
        assert_eq!(rs_nr2hex(11), u32::from(b'b'));
        assert_eq!(rs_nr2hex(15), u32::from(b'f'));

        // Test that only lower 4 bits are used
        assert_eq!(rs_nr2hex(0x10), u32::from(b'0')); // 16 -> 0
        assert_eq!(rs_nr2hex(0x1f), u32::from(b'f')); // 31 -> 15 -> 'f'
        assert_eq!(rs_nr2hex(0xff), u32::from(b'f')); // 255 -> 15 -> 'f'
    }

    #[test]
    fn test_hex2nr() {
        // Test digits
        assert_eq!(rs_hex2nr(c_int::from(b'0')), 0);
        assert_eq!(rs_hex2nr(c_int::from(b'5')), 5);
        assert_eq!(rs_hex2nr(c_int::from(b'9')), 9);

        // Test uppercase
        assert_eq!(rs_hex2nr(c_int::from(b'A')), 10);
        assert_eq!(rs_hex2nr(c_int::from(b'F')), 15);

        // Test lowercase
        assert_eq!(rs_hex2nr(c_int::from(b'a')), 10);
        assert_eq!(rs_hex2nr(c_int::from(b'f')), 15);
    }

    #[test]
    fn test_hexhex2nr() {
        unsafe {
            let s = CString::new("FF").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 255);

            let s = CString::new("00").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 0);

            let s = CString::new("1a").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 26);

            let s = CString::new("a1").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 161);

            // Test invalid hex
            let s = CString::new("GG").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), -1);

            let s = CString::new("1G").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), -1);

            let s = CString::new("G1").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), -1);
        }
    }

    #[test]
    fn test_skiptowhite_esc() {
        unsafe {
            // Normal case - skip to space
            let s = CString::new("hello world").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, b' ' as c_char);

            // Escaped space with backslash - should skip over it and continue
            // "hello\ world" - the backslash escapes the space, so we continue to end
            let s = CString::new("hello\\ world").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 12); // Scans entire string "hello\ world" (12 bytes)

            // With actual space after escaped char
            let s = CString::new("hello\\x world").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            // backslash escapes 'x', then continues to space
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 7); // "hello\x" then hits space

            // Escaped space with Ctrl-V (ASCII 22)
            // "hi<Ctrl-V> x" - Ctrl-V escapes space, then continues to scan "x" to NUL
            let s: [u8; 6] = [b'h', b'i', 22, b' ', b'x', 0]; // "hi<Ctrl-V> x"
            let result = rs_skiptowhite_esc(s.as_ptr().cast::<c_char>());
            let offset = result.offset_from(s.as_ptr().cast::<c_char>());
            assert_eq!(offset, 5); // Scans to NUL at position 5

            // No whitespace
            let s = CString::new("hello").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, 0);

            // Backslash at end (next char is NUL)
            let s = CString::new("hello\\").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, 0);

            // Tab
            let s = CString::new("hello\tworld").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, b'\t' as c_char);
        }
    }

    #[test]
    fn test_getwhitecols() {
        unsafe {
            // Spaces at start
            let s = CString::new("   hello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 3);

            // Tabs at start
            let s = CString::new("\t\thello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 2);

            // Mixed spaces and tabs
            let s = CString::new(" \t \thello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 4);

            // No whitespace
            let s = CString::new("hello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 0);

            // Empty string
            let s = CString::new("").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 0);

            // All whitespace
            let s = CString::new("   ").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 3);

            // Null pointer
            assert_eq!(rs_getwhitecols(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_skip_to_newline() {
        unsafe {
            // Skip to newline
            let s = CString::new("hello\nworld").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, b'\n' as c_char);
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 5);

            // No newline - skip to NUL
            let s = CString::new("hello world").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, 0);

            // Newline at start
            let s = CString::new("\nhello").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, b'\n' as c_char);
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 0);

            // Empty string
            let s = CString::new("").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, 0);

            // Multiple newlines - stops at first
            let s = CString::new("line1\nline2\nline3").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, b'\n' as c_char);
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 5);

            // Null pointer
            let result = rs_skip_to_newline(std::ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_vim_isblankline() {
        unsafe {
            // Empty string - blank
            let s = CString::new("").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Only spaces - blank
            let s = CString::new("   ").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Only tabs - blank
            let s = CString::new("\t\t").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Mixed whitespace - blank
            let s = CString::new(" \t \t ").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Contains text - not blank
            let s = CString::new("hello").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Whitespace before text - not blank
            let s = CString::new("   hello").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Text before whitespace - not blank
            let s = CString::new("hello   ").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Single character - not blank
            let s = CString::new("x").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Line ending with \n (newline) - blank
            let s = CString::new("\n").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Whitespace followed by \n - blank
            let s = CString::new("   \n").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Line ending with \r (carriage return) - blank
            let s = CString::new("\r").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Whitespace followed by \r - blank
            let s = CString::new("   \r").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Whitespace followed by \r\n (CRLF) - blank
            let s = CString::new("   \r\n").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Null pointer - blank (edge case)
            assert!(rs_vim_isblankline(std::ptr::null()));
        }
    }

    #[test]
    fn test_transchar_hex() {
        unsafe {
            let mut buf = [0i8; 16];

            // Single byte value (0x00 - 0xFF) -> "<XX>"
            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x00);
            assert_eq!(len, 4);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'0' as i8, b'0' as i8, b'>' as i8, 0]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x1A);
            assert_eq!(len, 4);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'1' as i8, b'a' as i8, b'>' as i8, 0]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0xFF);
            assert_eq!(len, 4);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'f' as i8, b'f' as i8, b'>' as i8, 0]
            );

            // Two byte value (0x100 - 0xFFFF) -> "<XXXX>"
            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x100);
            assert_eq!(len, 6);
            assert_eq!(
                &buf[..7],
                [b'<' as i8, b'0' as i8, b'1' as i8, b'0' as i8, b'0' as i8, b'>' as i8, 0]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0xABCD);
            assert_eq!(len, 6);
            assert_eq!(
                &buf[..7],
                [b'<' as i8, b'a' as i8, b'b' as i8, b'c' as i8, b'd' as i8, b'>' as i8, 0]
            );

            // Three byte value (> 0xFFFF) -> "<XXXXXX>"
            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x10000);
            assert_eq!(len, 8);
            assert_eq!(
                &buf[..9],
                [
                    b'<' as i8, b'0' as i8, b'1' as i8, b'0' as i8, b'0' as i8, b'0' as i8,
                    b'0' as i8, b'>' as i8, 0
                ]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x0012_ABCD);
            assert_eq!(len, 8);
            assert_eq!(
                &buf[..9],
                [
                    b'<' as i8, b'1' as i8, b'2' as i8, b'a' as i8, b'b' as i8, b'c' as i8,
                    b'd' as i8, b'>' as i8, 0
                ]
            );

            // Null buffer returns 0
            assert_eq!(rs_transchar_hex(std::ptr::null_mut(), 0x42), 0);
        }
    }
}
