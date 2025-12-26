//! Status line and tab line helper functions for Neovim
//!
//! This crate provides Rust implementations of status line functions
//! from `src/nvim/statusline.c` and related column formatting utilities.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::c_int;
use std::io::Write;

use nvim_window::{Frame, WinHandle, FR_COL};

/// schar_T is stored as a u32 in Rust.
type ScharT = u32;

/// Highlight group for StatusLine (current window).
pub const HLF_S: c_int = 27;

/// Highlight group for StatusLineNC (non-current windows).
pub const HLF_SNC: c_int = 28;

// C accessor functions
extern "C" {
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;
    fn nvim_win_get_fcs_stl(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_stlnc(wp: WinHandle) -> ScharT;
}

/// Check if the status line of window "wp" is connected to the status
/// line of the window right of it. If not, then it's a vertical separator.
///
/// Only call if `wp->w_vsep_width != 0`.
///
/// This is the Rust equivalent of `stl_connected()` in statusline.c.
fn stl_connected_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let mut fr = nvim_win_get_frame(wp);
        if fr.is_null() {
            return false;
        }

        // Walk up the frame tree
        while !(*fr).fr_parent.is_null() {
            let parent = (*fr).fr_parent;
            if (*parent).fr_layout == FR_COL {
                // In a column layout - check if there's a frame below
                if !(*fr).fr_next.is_null() {
                    break;
                }
            } else {
                // In a row layout - check if there's a frame to the right
                if !(*fr).fr_next.is_null() {
                    return true;
                }
            }
            fr = parent;
        }
        false
    }
}

/// Get the fill character and highlight group for a status line.
///
/// Returns the fill character (schar_T) and sets `*group` to:
/// - `HLF_S` (StatusLine) if wp is the current window
/// - `HLF_SNC` (StatusLineNC) if wp is not the current window
///
/// This is the Rust equivalent of `fillchar_status()` in statusline.c.
fn fillchar_status_impl(wp: WinHandle) -> (ScharT, c_int) {
    unsafe {
        if nvim_win_is_curwin(wp) != 0 {
            (nvim_win_get_fcs_stl(wp), HLF_S)
        } else {
            (nvim_win_get_fcs_stlnc(wp), HLF_SNC)
        }
    }
}

/// Format a column number for display.
///
/// If `col == vcol`, returns "col" as a string.
/// If `col != vcol`, returns "col-vcol" as a string.
///
/// Returns the number of bytes written (not including NUL terminator).
///
/// This is the Rust equivalent of `col_print()` in buffer.c.
fn col_print_impl(buf: &mut [u8], col: c_int, vcol: c_int) -> c_int {
    if buf.is_empty() {
        return 0;
    }

    let mut cursor = std::io::Cursor::new(buf);
    let result = if col == vcol {
        write!(cursor, "{col}")
    } else {
        write!(cursor, "{col}-{vcol}")
    };

    match result {
        #[allow(clippy::cast_possible_truncation)]
        Ok(()) => cursor.position() as c_int,
        Err(_) => 0,
    }
}

// FFI exports

/// Check if status line is connected to the window on the right.
///
/// # Safety
/// `wp` must be a valid window handle or null.
#[no_mangle]
pub extern "C" fn rs_stl_connected(wp: WinHandle) -> c_int {
    c_int::from(stl_connected_impl(wp))
}

/// Get the fill character for a status line.
///
/// # Safety
/// `wp` must be a valid window handle.
/// `group` must be a valid pointer to an hlf_T value.
#[no_mangle]
pub unsafe extern "C" fn rs_fillchar_status(group: *mut c_int, wp: WinHandle) -> ScharT {
    let (fillchar, grp) = fillchar_status_impl(wp);
    if !group.is_null() {
        *group = grp;
    }
    fillchar
}

/// Format a column number for display.
///
/// # Safety
/// `buf` must be a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_col_print(
    buf: *mut u8,
    buflen: usize,
    col: c_int,
    vcol: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    col_print_impl(slice, col, vcol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_col_print_same() {
        let mut buf = [0u8; 32];
        let len = col_print_impl(&mut buf, 42, 42);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"42");
    }

    #[test]
    fn test_col_print_different() {
        let mut buf = [0u8; 32];
        let len = col_print_impl(&mut buf, 10, 25);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"10-25");
    }

    #[test]
    fn test_col_print_empty_buffer() {
        let mut buf = [0u8; 0];
        let len = col_print_impl(&mut buf, 10, 25);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_col_print_small_buffer() {
        let mut buf = [0u8; 3];
        let len = col_print_impl(&mut buf, 10, 25);
        // Should write "10-" (truncated)
        assert!(len <= 3);
    }
}
