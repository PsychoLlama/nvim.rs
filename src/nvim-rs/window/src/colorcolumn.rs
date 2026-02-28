//! Colorcolumn option parsing and validation.
//!
//! This module provides the Rust implementation of `check_colorcolumn()` from
//! `window_shim.c`, which validates and applies the 'colorcolumn' option.

#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_char, c_int, c_void};

use crate::WinHandle;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Allocate memory using the C allocator (must be freed with xfree).
    fn xmalloc(size: usize) -> *mut c_void;

    /// Free memory allocated with xmalloc/xcalloc/xrealloc.
    fn xfree(ptr: *mut c_void);

    /// Get the w_p_cc (colorcolumn option string) from a window.
    fn nvim_win_get_p_cc(wp: WinHandle) -> *const c_char;

    /// Get the buffer's b_p_tw (textwidth) from a window's buffer.
    fn nvim_win_get_buf_b_p_tw(wp: WinHandle) -> i64;

    /// Check if a window's buffer is NULL (closed buffer).
    fn nvim_win_has_buffer(wp: WinHandle) -> c_int;

    /// Get wp->w_p_cc_cols.
    fn nvim_win_get_p_cc_cols(wp: WinHandle) -> *mut c_int;

    /// Set wp->w_p_cc_cols.
    fn nvim_win_set_p_cc_cols(wp: WinHandle, cols: *mut c_int);

    /// Free and NULL wp->w_p_cc_cols.
    fn nvim_win_free_p_cc_cols(wp: WinHandle);

    /// Get the e_invarg error string pointer.
    fn nvim_get_e_invarg() -> *const c_char;

    /// Get the empty_string_option pointer.
    fn nvim_get_empty_string_option() -> *const c_char;
}

// =============================================================================
// Colorcolumn Parsing
// =============================================================================

/// Parse a sequence of ASCII decimal digits, advancing the slice.
///
/// Returns the parsed value. The slice is advanced past all digits.
fn parse_digits(s: &mut &[u8]) -> c_int {
    let mut val: c_int = 0;
    while let Some(&b) = s.first() {
        if !b.is_ascii_digit() {
            break;
        }
        // Saturating arithmetic to avoid overflow on pathological input.
        val = val.saturating_mul(10).saturating_add(c_int::from(b - b'0'));
        *s = &s[1..];
    }
    val
}

/// Check whether a byte is an ASCII decimal digit.
const fn is_ascii_digit(b: u8) -> bool {
    b.is_ascii_digit()
}

/// Rust implementation of `check_colorcolumn()`.
///
/// Parses the colorcolumn string `cc` (or `wp->w_p_cc` when `cc` is NULL),
/// validates it, and stores the resulting 0-based column indices in
/// `wp->w_p_cc_cols`.
///
/// Returns `NULL` on success, or a pointer to `e_invarg` on error.
///
/// # Safety
/// - `cc` must be NULL or a valid NUL-terminated C string.
/// - `wp` must be NULL or a valid window handle.
unsafe fn check_colorcolumn_impl(cc: *const c_char, wp: WinHandle) -> *const c_char {
    // If wp is given but its buffer was closed, skip silently.
    if !wp.is_null() && nvim_win_has_buffer(wp) == 0 {
        return std::ptr::null();
    }

    // Determine the option string to parse.
    let s_ptr: *const c_char = if !cc.is_null() {
        cc
    } else if !wp.is_null() {
        nvim_win_get_p_cc(wp)
    } else {
        nvim_get_empty_string_option()
    };

    // Safety: s_ptr is always a valid NUL-terminated C string at this point.
    let full_str = std::ffi::CStr::from_ptr(s_ptr);
    let mut s: &[u8] = full_str.to_bytes(); // excludes the NUL

    // Determine textwidth (relative offsets are added to this).
    let tw: i64 = if wp.is_null() {
        0
    } else {
        nvim_win_get_buf_b_p_tw(wp)
    };

    // Parse up to 255 column entries.
    let mut color_cols: Vec<c_int> = Vec::with_capacity(32);

    while !s.is_empty() && color_cols.len() < 255 {
        let first = s[0];
        let col: Option<c_int>; // None means "skip this item"

        if first == b'-' || first == b'+' {
            let sign: i64 = if first == b'-' { -1 } else { 1 };
            s = &s[1..]; // consume sign
            if s.is_empty() || !is_ascii_digit(s[0]) {
                return nvim_get_e_invarg();
            }
            let n = i64::from(parse_digits(&mut s));
            let offset = sign * n;
            if tw == 0 {
                // 'textwidth' not set, skip this item.
                col = None;
            } else {
                let abs_col = tw + offset;
                if abs_col < 0 {
                    col = None; // negative column, skip
                } else {
                    // Convert from 1-based to 0-based.
                    col = Some((abs_col as c_int) - 1);
                }
            }
        } else if is_ascii_digit(first) {
            let n = parse_digits(&mut s);
            // Convert from 1-based to 0-based.
            col = Some(n - 1);
        } else {
            return nvim_get_e_invarg();
        }

        if let Some(c) = col {
            color_cols.push(c);
        }

        if s.is_empty() {
            break;
        }
        if s[0] != b',' {
            return nvim_get_e_invarg();
        }
        s = &s[1..]; // consume comma
        if s.is_empty() {
            // Trailing comma is illegal (e.g., "set cc=80,").
            return nvim_get_e_invarg();
        }
    }

    if wp.is_null() {
        // Only parse, do not apply.
        return std::ptr::null();
    }

    // Free the old column array.
    nvim_win_free_p_cc_cols(wp);

    if color_cols.is_empty() {
        nvim_win_set_p_cc_cols(wp, std::ptr::null_mut());
    } else {
        // Sort and deduplicate.
        color_cols.sort_unstable();
        color_cols.dedup();

        // Allocate a C array (count + 1 entries: values followed by -1 sentinel).
        let count = color_cols.len();
        let ptr = xmalloc(std::mem::size_of::<c_int>() * (count + 1)).cast::<c_int>();
        for (i, &val) in color_cols.iter().enumerate() {
            *ptr.add(i) = val;
        }
        *ptr.add(count) = -1; // end marker
        nvim_win_set_p_cc_cols(wp, ptr);
    }

    std::ptr::null() // no error
}

// =============================================================================
// FFI Export
// =============================================================================

/// FFI: Parse and apply the 'colorcolumn' option for a window.
///
/// # Safety
/// - `cc` must be NULL or a valid NUL-terminated C string.
/// - `wp` must be NULL or a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_check_colorcolumn(cc: *const c_char, wp: WinHandle) -> *const c_char {
    check_colorcolumn_impl(cc, wp)
}

/// C export: `check_colorcolumn` — eliminates the C thin wrapper.
///
/// # Safety
/// - `cc` must be NULL or a valid NUL-terminated C string.
/// - `wp` must be NULL or a valid window handle.
#[unsafe(export_name = "check_colorcolumn")]
pub unsafe extern "C" fn check_colorcolumn(cc: *const c_char, wp: WinHandle) -> *const c_char {
    check_colorcolumn_impl(cc, wp)
}
