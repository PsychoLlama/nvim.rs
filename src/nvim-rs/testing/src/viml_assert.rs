//! VimL assertion functions for testing
//!
//! This module provides Rust implementations of VimL's `assert_*` and `test_*`
//! functions from `src/nvim/testing.c`.
//!
//! ## Architecture
//!
//! The module uses an opaque handle pattern where `typval_T*` pointers are
//! treated as opaque handles, with field access done through C accessor
//! functions.

#![allow(clippy::doc_markdown)]
// Allow dead code for functions that will be used in later migration phases
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

use nvim_collections::garray::GArray;

// =============================================================================
// Type aliases for opaque handles
// =============================================================================

/// Opaque handle to a typval_T (VimL value).
pub type TypevalHandle = *const c_void;

/// Opaque handle to a mutable typval_T.
pub type TypevalHandleMut = *mut c_void;

// =============================================================================
// C accessor functions
// =============================================================================

extern "C" {
    // GArray operations (from collections crate, re-exported in C)
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_clear(gap: *mut GArray);
    fn ga_concat(gap: *mut GArray, s: *const c_char);
    fn ga_append(gap: *mut GArray, c: u8);

    // Sourcing information - for error location
    fn estack_sfile(which: c_int) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // Typval string extraction
    fn tv_get_string(tv: TypevalHandle) -> *const c_char;

    // Assert error reporting - adds to v:errors
    fn assert_error(gap: *mut GArray);
}

// =============================================================================
// Constants
// =============================================================================

/// ESTACK_NONE constant from runtime_defs.h
const ESTACK_NONE: c_int = 0;

// =============================================================================
// Helper functions
// =============================================================================

// Access SOURCING_LNUM through C accessor.
extern "C" {
    fn nvim_testing_get_sourcing_lnum() -> i64;
}

/// Prepare a GArray for an assert error and add the sourcing position.
///
/// This mirrors the C `prepare_assert_error` function.
fn prepare_assert_error(gap: *mut GArray) {
    unsafe {
        ga_init(gap, 1, 100);

        let sname = estack_sfile(ESTACK_NONE);
        let sourcing_lnum = nvim_testing_get_sourcing_lnum();

        if !sname.is_null() {
            ga_concat(gap, sname);
            if sourcing_lnum > 0 {
                ga_concat(gap, c" ".as_ptr());
            }
        }

        if sourcing_lnum > 0 {
            // Format "line <number>"
            let mut buf = [0u8; 64];
            let len = format_line_number(&mut buf, sourcing_lnum);
            ga_concat(gap, buf.as_ptr().cast());
            let _ = len; // silence unused warning
        }

        if !sname.is_null() || sourcing_lnum > 0 {
            ga_concat(gap, c": ".as_ptr());
        }

        if !sname.is_null() {
            xfree(sname.cast());
        }
    }
}

/// Format "line <number>" into a buffer. Returns the length written.
fn format_line_number(buf: &mut [u8; 64], lnum: i64) -> usize {
    let prefix = b"line ";
    buf[..prefix.len()].copy_from_slice(prefix);

    // Convert number to string
    let mut num = lnum;
    let mut digits = [0u8; 20];
    let mut digit_count = 0;

    if num == 0 {
        digit_count = 1;
        digits[0] = b'0';
    } else {
        let negative = num < 0;
        if negative {
            num = -num;
        }

        while num > 0 {
            digits[digit_count] = b'0' + (num % 10) as u8;
            digit_count += 1;
            num /= 10;
        }

        if negative {
            digits[digit_count] = b'-';
            digit_count += 1;
        }
    }

    // Copy digits in reverse order
    let mut pos = prefix.len();
    for i in (0..digit_count).rev() {
        buf[pos] = digits[i];
        pos += 1;
    }
    buf[pos] = 0; // null terminator

    pos
}

// =============================================================================
// String escaping functions
// =============================================================================

// ASCII control character constants
const BS: u8 = 0x08; // Backspace
const TAB: u8 = 0x09; // Tab
const NL: u8 = 0x0A; // Newline
const FF: u8 = 0x0C; // Form feed
const CAR: u8 = 0x0D; // Carriage return
const ESC: u8 = 0x1B; // Escape

/// Append a character (possibly multi-byte) to the GArray, escaping unprintable characters.
/// Changes NL to \n, CR to \r, etc.
///
/// This mirrors the C `ga_concat_esc` function.
fn ga_concat_esc(gap: *mut GArray, p: *const u8, clen: usize) {
    unsafe {
        // Multi-byte character: copy as-is
        if clen > 1 {
            let mut buf = [0u8; 8];
            let copy_len = clen.min(7);
            std::ptr::copy_nonoverlapping(p, buf.as_mut_ptr(), copy_len);
            buf[copy_len] = 0;
            ga_concat(gap, buf.as_ptr().cast());
            return;
        }

        let c = *p;
        match c {
            BS => ga_concat(gap, c"\\b".as_ptr()),
            ESC => ga_concat(gap, c"\\e".as_ptr()),
            FF => ga_concat(gap, c"\\f".as_ptr()),
            NL => ga_concat(gap, c"\\n".as_ptr()),
            TAB => ga_concat(gap, c"\\t".as_ptr()),
            CAR => ga_concat(gap, c"\\r".as_ptr()),
            b'\\' => ga_concat(gap, c"\\\\".as_ptr()),
            _ => {
                if c < b' ' || c == 0x7f {
                    // Format as \xNN
                    let mut buf = [0u8; 8];
                    buf[0] = b'\\';
                    buf[1] = b'x';
                    buf[2] = hex_digit(c >> 4);
                    buf[3] = hex_digit(c & 0x0f);
                    buf[4] = 0;
                    ga_concat(gap, buf.as_ptr().cast());
                } else {
                    ga_append(gap, c);
                }
            }
        }
    }
}

/// Convert a nibble (0-15) to a hex digit.
#[inline]
const fn hex_digit(n: u8) -> u8 {
    if n < 10 {
        b'0' + n
    } else {
        b'a' + (n - 10)
    }
}

/// Format an integer into a buffer. Returns the length written (excluding NUL).
fn format_int(buf: &mut [u8], value: i32) -> usize {
    let mut num = value;
    let mut digits = [0u8; 12];
    let mut digit_count = 0;

    if num == 0 {
        digit_count = 1;
        digits[0] = b'0';
    } else {
        let negative = num < 0;
        if negative {
            num = -num;
        }

        while num > 0 {
            digits[digit_count] = b'0' + (num % 10) as u8;
            digit_count += 1;
            num /= 10;
        }

        if negative {
            digits[digit_count] = b'-';
            digit_count += 1;
        }
    }

    // Copy digits in reverse order
    let mut pos = 0;
    for i in (0..digit_count).rev() {
        if pos < buf.len() - 1 {
            buf[pos] = digits[i];
            pos += 1;
        }
    }
    if pos < buf.len() {
        buf[pos] = 0;
    }

    pos
}

/// Append a string to the GArray, escaping unprintable characters.
/// If the same character appears more than 20 times, it's shortened.
///
/// This mirrors the C `ga_concat_shorten_esc` function.
fn ga_concat_shorten_esc(gap: *mut GArray, s: *const c_char) {
    unsafe {
        if s.is_null() {
            ga_concat(gap, c"NULL".as_ptr());
            return;
        }

        let mut p = s.cast::<u8>();

        while *p != 0 {
            // Get the character and its byte length
            let (c, clen) = nvim_mbyte::mb_cptr2char_adv(std::slice::from_raw_parts(p, 6));
            let clen = clen.max(1); // Ensure at least 1 byte

            // Count consecutive occurrences of the same character
            let mut same_len = 1;
            let mut scan = p.add(clen);
            while *scan != 0 {
                let scan_c = nvim_mbyte::utf_ptr2char(std::slice::from_raw_parts(scan, 6));
                if scan_c != c {
                    break;
                }
                same_len += 1;
                scan = scan.add(clen);
            }

            if same_len > 20 {
                // Shorten: "\[<char> occurs <n> times]"
                ga_concat(gap, c"\\[".as_ptr());
                ga_concat_esc(gap, p, clen);
                ga_concat(gap, c" occurs ".as_ptr());

                let mut buf = [0u8; 16];
                format_int(&mut buf, same_len);
                ga_concat(gap, buf.as_ptr().cast());

                ga_concat(gap, c" times]".as_ptr());
                p = scan;
            } else {
                ga_concat_esc(gap, p, clen);
                p = p.add(clen);
            }
        }
    }
}

// =============================================================================
// VimL function implementations
// =============================================================================

/// `assert_report(msg)` function implementation.
///
/// This is the simplest assert function - it just adds the message to v:errors.
///
/// # Safety
///
/// - `argvars` must point to a valid array of at least 1 `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[no_mangle]
pub unsafe extern "C" fn rs_f_assert_report(argvars: TypevalHandle, rettv: TypevalHandleMut) {
    let mut ga = GArray::default();

    prepare_assert_error(&raw mut ga);

    // Get the message string from argvars[0]
    let msg = tv_get_string(argvars);
    if !msg.is_null() {
        ga_concat(&raw mut ga, msg);
    }

    assert_error(&raw mut ga);
    ga_clear(&raw mut ga);

    // Set return value to 1 (failure count)
    set_rettv_number(rettv, 1);
}

/// `test_write_list_log(fname)` function implementation.
///
/// This is a no-op function in Neovim (`list_log` feature is disabled).
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[no_mangle]
pub unsafe extern "C" fn rs_f_test_write_list_log(
    _argvars: TypevalHandle,
    _rettv: TypevalHandleMut,
) {
    // This function is a no-op in Neovim
    // The original C code just extracts the filename and does nothing with it
}

// =============================================================================
// Return value helpers
// =============================================================================

extern "C" {
    fn nvim_testing_set_rettv_number(rettv: TypevalHandleMut, val: i64);
}

/// Set the return typval to a number.
#[inline]
fn set_rettv_number(rettv: TypevalHandleMut, val: i64) {
    unsafe {
        nvim_testing_set_rettv_number(rettv, val);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_line_number() {
        let mut buf = [0u8; 64];
        format_line_number(&mut buf, 42);
        let s = std::str::from_utf8(&buf[..8]).unwrap();
        assert_eq!(s, "line 42\0");
    }

    #[test]
    fn test_format_line_number_zero() {
        let mut buf = [0u8; 64];
        format_line_number(&mut buf, 0);
        let s = std::str::from_utf8(&buf[..7]).unwrap();
        assert_eq!(s, "line 0\0");
    }

    #[test]
    fn test_format_line_number_large() {
        let mut buf = [0u8; 64];
        format_line_number(&mut buf, 12345);
        let s = std::str::from_utf8(&buf[..11]).unwrap();
        assert_eq!(s, "line 12345\0");
    }

    #[test]
    fn test_hex_digit() {
        assert_eq!(hex_digit(0), b'0');
        assert_eq!(hex_digit(9), b'9');
        assert_eq!(hex_digit(10), b'a');
        assert_eq!(hex_digit(15), b'f');
    }

    #[test]
    fn test_format_int() {
        let mut buf = [0u8; 16];
        format_int(&mut buf, 42);
        assert_eq!(&buf[..3], b"42\0");

        format_int(&mut buf, 0);
        assert_eq!(&buf[..2], b"0\0");

        format_int(&mut buf, 12345);
        assert_eq!(&buf[..6], b"12345\0");
    }
}
