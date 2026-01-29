//! Indent getter functions for Neovim.
//!
//! This module provides getter functions for indentation values from buffers
//! and the current window/cursor position.

use std::ffi::{c_char, c_int};

use nvim_buffer::BufHandle;

use crate::{rs_get_sw_value_col, rs_indent_size_ts};

// External C accessor functions
extern "C" {
    fn nvim_buf_get_p_ts(buf: BufHandle) -> i64;
    fn nvim_buf_get_p_vts_array(buf: BufHandle) -> *const c_int;
    fn nvim_buf_get_p_sts(buf: BufHandle) -> i64;

    // Line getters
    fn nvim_curbuf_get_line_ptr() -> *const c_char;
    fn nvim_curbuf_get_line_at(lnum: i32) -> *const c_char;
    fn nvim_buf_get_line_at(buf: BufHandle, lnum: i32) -> *const c_char;

    // Cursor position
    fn nvim_curwin_get_cursor_col() -> c_int;

    // Current buffer handle
    fn nvim_get_curbuf() -> BufHandle;

    // Getwhitecols
    fn nvim_getwhitecols_curline() -> c_int;
}

// Type alias for line number
type LineNr = i32;

// =============================================================================
// Shiftwidth getters
// =============================================================================

/// Get the effective shiftwidth value for a buffer.
///
/// Returns the 'shiftwidth' value if set, otherwise uses 'tabstop'.
///
/// # Safety
/// The `buf` parameter must be a valid buffer pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sw_value(buf: BufHandle) -> c_int {
    rs_get_sw_value_col(buf, 0, false)
}

/// Get the effective shiftwidth at the first non-blank in current line.
///
/// # Safety
/// Accesses current buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sw_value_indent(buf: BufHandle, left: bool) -> c_int {
    let whitecols = nvim_getwhitecols_curline();
    rs_get_sw_value_col(buf, whitecols, left)
}

// =============================================================================
// Softtabstop getter
// =============================================================================

/// Get the effective softtabstop value for the current buffer.
///
/// If 'softtabstop' is negative, returns the shiftwidth value instead.
///
/// # Safety
/// Accesses current buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sts_value() -> c_int {
    let buf = nvim_get_curbuf();
    let sts = nvim_buf_get_p_sts(buf);
    if sts < 0 {
        rs_get_sw_value(buf)
    } else {
        sts as c_int
    }
}

// =============================================================================
// Indent getters
// =============================================================================

/// Get the indent (in window cells) of the current line.
///
/// # Safety
/// Accesses current buffer and cursor state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_indent() -> c_int {
    let buf = nvim_get_curbuf();
    let line = nvim_curbuf_get_line_ptr();
    let ts = nvim_buf_get_p_ts(buf);
    let vts = nvim_buf_get_p_vts_array(buf);
    rs_indent_size_ts(line, ts, vts)
}

/// Get the indent (in window cells) at a specific line number.
///
/// # Safety
/// The line number must be valid for the current buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_indent_lnum(lnum: LineNr) -> c_int {
    let buf = nvim_get_curbuf();
    let line = nvim_curbuf_get_line_at(lnum);
    let ts = nvim_buf_get_p_ts(buf);
    let vts = nvim_buf_get_p_vts_array(buf);
    rs_indent_size_ts(line, ts, vts)
}

/// Get the indent (in window cells) at a specific line in a specific buffer.
///
/// # Safety
/// The buffer and line number must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_indent_buf(buf: BufHandle, lnum: LineNr) -> c_int {
    let line = nvim_buf_get_line_at(buf, lnum);
    let ts = nvim_buf_get_p_ts(buf);
    let vts = nvim_buf_get_p_vts_array(buf);
    rs_indent_size_ts(line, ts, vts)
}

// =============================================================================
// Cursor position helpers
// =============================================================================

/// Check if the cursor is in the indentation area of the current line.
///
/// When `extra` == 0: Returns true if cursor is before or on first non-blank.
/// When `extra` == 1: Returns true if cursor is before first non-blank.
///
/// # Safety
/// Accesses current window and buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_inindent(extra: c_int) -> bool {
    let line = nvim_curbuf_get_line_ptr();
    if line.is_null() {
        return false;
    }

    // Count whitespace characters at start of line
    let mut col: c_int = 0;
    let mut ptr = line;
    while *ptr == b' ' as c_char || *ptr == b'\t' as c_char {
        col += 1;
        ptr = ptr.add(1);
    }

    // Get cursor column
    let cursor_col = nvim_curwin_get_cursor_col();

    col >= cursor_col + extra
}

// =============================================================================
// Tabstop array copy
// =============================================================================

/// Copy a tabstop array, allocating new memory.
///
/// The caller is responsible for freeing the returned memory.
///
/// # Safety
/// If `oldts` is non-null, it must point to a valid tabstop array.
/// The returned pointer must be freed by the caller.
#[no_mangle]
pub unsafe extern "C" fn rs_tabstop_copy(oldts: *const c_int) -> *mut c_int {
    if oldts.is_null() {
        return std::ptr::null_mut();
    }

    let count = *oldts;
    if count < 0 {
        return std::ptr::null_mut();
    }

    // Allocate count+1 integers (count at index 0, then count values)
    let size = (count + 1) as usize;
    let layout = std::alloc::Layout::array::<c_int>(size).unwrap();
    let newts = std::alloc::alloc(layout) as *mut c_int;

    if newts.is_null() {
        return std::ptr::null_mut();
    }

    // Copy the array
    std::ptr::copy_nonoverlapping(oldts, newts, size);

    newts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabstop_copy_null() {
        unsafe {
            let result = rs_tabstop_copy(std::ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_tabstop_copy_valid() {
        unsafe {
            // Create a tabstop array: [2, 4, 8] = 2 tabstops at positions 4 and 8
            let original: [c_int; 3] = [2, 4, 8];
            let copy = rs_tabstop_copy(original.as_ptr());

            assert!(!copy.is_null());
            assert_eq!(*copy, 2);
            assert_eq!(*copy.add(1), 4);
            assert_eq!(*copy.add(2), 8);

            // Free the copy
            let layout = std::alloc::Layout::array::<c_int>(3).unwrap();
            std::alloc::dealloc(copy as *mut u8, layout);
        }
    }
}
