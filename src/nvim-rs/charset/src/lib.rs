//! Character set utilities for Neovim.
//!
//! This crate provides FFI-compatible character classification and
//! string skipping utilities.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

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
/// Like skiptowhite(), but also skips escaped chars (backslash or Ctrl-V).
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
    if c >= b'a' && c <= b'f' {
        c_int::from(c - b'a' + 10)
    } else if c >= b'A' && c <= b'F' {
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
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
        assert_eq!(rs_nr2hex(0), b'0' as u32);
        assert_eq!(rs_nr2hex(5), b'5' as u32);
        assert_eq!(rs_nr2hex(9), b'9' as u32);

        // Test 10-15 -> 'a'-'f' (lowercase)
        assert_eq!(rs_nr2hex(10), b'a' as u32);
        assert_eq!(rs_nr2hex(11), b'b' as u32);
        assert_eq!(rs_nr2hex(15), b'f' as u32);

        // Test that only lower 4 bits are used
        assert_eq!(rs_nr2hex(0x10), b'0' as u32); // 16 -> 0
        assert_eq!(rs_nr2hex(0x1f), b'f' as u32); // 31 -> 15 -> 'f'
        assert_eq!(rs_nr2hex(0xff), b'f' as u32); // 255 -> 15 -> 'f'
    }

    #[test]
    fn test_hex2nr() {
        // Test digits
        assert_eq!(rs_hex2nr(b'0' as c_int), 0);
        assert_eq!(rs_hex2nr(b'5' as c_int), 5);
        assert_eq!(rs_hex2nr(b'9' as c_int), 9);

        // Test uppercase
        assert_eq!(rs_hex2nr(b'A' as c_int), 10);
        assert_eq!(rs_hex2nr(b'F' as c_int), 15);

        // Test lowercase
        assert_eq!(rs_hex2nr(b'a' as c_int), 10);
        assert_eq!(rs_hex2nr(b'f' as c_int), 15);
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
            let s = vec![b'h', b'i', 22, b' ', b'x', 0]; // "hi<Ctrl-V> x"
            let result = rs_skiptowhite_esc(s.as_ptr() as *const c_char);
            let offset = result.offset_from(s.as_ptr() as *const c_char);
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
}
