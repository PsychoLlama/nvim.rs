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
}
