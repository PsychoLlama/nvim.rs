//! Cursor positioning and validation for Neovim
//!
//! This crate provides Rust implementations of cursor-related functions
//! from `src/nvim/cursor.c`. It handles:
//! - Cursor position validation
//! - Line and column bounds checking
//! - Virtual column handling
//! - Cursor state management
//!
//! The crate uses the opaque handle pattern for window access.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(dead_code)] // Some extern declarations are pre-declared for future use

use std::ffi::c_int;

use nvim_window::WinHandle;

// =============================================================================
// Position Types
// =============================================================================

/// Cursor position type matching C `pos_T`.
///
/// This represents a position in a buffer with line number, column, and
/// virtual column addition for 'virtualedit'.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CursorPos {
    /// Line number (1-based)
    pub lnum: i64,
    /// Column (0-based byte offset)
    pub col: i32,
    /// Column addition for 'virtualedit'
    pub coladd: i32,
}

impl CursorPos {
    /// Create a new cursor position.
    #[inline]
    #[must_use]
    pub const fn new(lnum: i64, col: i32, coladd: i32) -> Self {
        Self { lnum, col, coladd }
    }

    /// Create a position at the beginning of a line.
    #[inline]
    #[must_use]
    pub const fn line_start(lnum: i64) -> Self {
        Self {
            lnum,
            col: 0,
            coladd: 0,
        }
    }

    /// Check if this position is valid (non-zero line number).
    #[inline]
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.lnum > 0
    }
}

// =============================================================================
// Cursor Validation Constants
// =============================================================================

/// Maximum column value (used for end of line positioning)
pub const MAXCOL: i32 = i32::MAX;

// =============================================================================
// Virtual Edit Flags (from option_vars.h)
// =============================================================================

/// Virtual edit flag: block mode
pub const VE_BLOCK: c_int = 0x01;
/// Virtual edit flag: insert mode
pub const VE_INSERT: c_int = 0x02;
/// Virtual edit flag: all modes
pub const VE_ALL: c_int = 0x04;
/// Virtual edit flag: one more than end of line
pub const VE_ONEMORE: c_int = 0x08;
/// Virtual edit flag: none set
pub const VE_NONE: c_int = 0x10;
/// Virtual edit flag: none or onemore
pub const VE_NONEU: c_int = 0x20;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    /// Get line count from buffer
    fn nvim_buf_get_line_count(buf: *mut std::ffi::c_void) -> i64;

    /// Get buffer from window
    fn nvim_win_get_buffer(win: WinHandle) -> *mut std::ffi::c_void;

    /// Get cursor position from window
    fn nvim_win_get_cursor_lnum(win: WinHandle) -> i64;
    fn nvim_win_get_cursor_col(win: WinHandle) -> i32;
    fn nvim_win_get_cursor_coladd(win: WinHandle) -> i32;

    /// Get length of a line in bytes
    fn nvim_buf_get_line_len(buf: *mut std::ffi::c_void, lnum: i64) -> i32;

    /// Check if 'virtualedit' allows cursor past end of line
    fn nvim_virtual_active(win: WinHandle) -> bool;

    /// Get current mode state (`MODE_INSERT`, `MODE_TERMINAL`, etc)
    fn nvim_get_state() -> c_int;

    /// Get 've' option flags for window
    fn nvim_get_ve_flags(win: WinHandle) -> c_int;

    /// Check if insert mode restart is pending
    fn nvim_get_restart_edit() -> c_int;

    /// Check if Visual mode is active
    fn nvim_get_visual_active() -> bool;

    /// Get 'selection' option first character
    fn nvim_get_p_sel_first() -> c_int;
}

// =============================================================================
// Mode Constants (from state_defs.h)
// =============================================================================

/// Insert mode flag
pub const MODE_INSERT: c_int = 0x10;
/// Terminal mode flag
pub const MODE_TERMINAL: c_int = 0x2000;

// =============================================================================
// Position Comparison Functions
// =============================================================================

/// Compare two cursor positions.
///
/// Returns:
/// - negative if a < b
/// - 0 if a == b
/// - positive if a > b
#[no_mangle]
pub extern "C" fn rs_cursor_pos_cmp(a: &CursorPos, b: &CursorPos) -> c_int {
    if a.lnum != b.lnum {
        if a.lnum < b.lnum {
            -1
        } else {
            1
        }
    } else if a.col != b.col {
        if a.col < b.col {
            -1
        } else {
            1
        }
    } else if a.coladd != b.coladd {
        if a.coladd < b.coladd {
            -1
        } else {
            1
        }
    } else {
        0
    }
}

/// Check if two positions are equal.
#[no_mangle]
pub extern "C" fn rs_cursor_pos_equal(a: &CursorPos, b: &CursorPos) -> bool {
    a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd
}

/// Check if position a is less than position b.
#[no_mangle]
pub extern "C" fn rs_cursor_pos_less(a: &CursorPos, b: &CursorPos) -> bool {
    if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd < b.coladd
    }
}

/// Check if position a is less than or equal to position b.
#[no_mangle]
pub extern "C" fn rs_cursor_pos_leq(a: &CursorPos, b: &CursorPos) -> bool {
    if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd <= b.coladd
    }
}

// =============================================================================
// Cursor Position Helpers
// =============================================================================

/// Get the line count for a window's buffer.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_get_line_count(win: WinHandle) -> i64 {
    let buf = nvim_win_get_buffer(win);
    if buf.is_null() {
        return 0;
    }
    nvim_buf_get_line_count(buf)
}

/// Clamp a line number to valid buffer range.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_clamp_lnum(win: WinHandle, lnum: i64) -> i64 {
    let line_count = rs_cursor_get_line_count(win);
    if lnum < 1 {
        1
    } else if lnum > line_count {
        line_count.max(1)
    } else {
        lnum
    }
}

/// Check if a line number is valid for a window's buffer.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_valid_lnum(win: WinHandle, lnum: i64) -> bool {
    if lnum < 1 {
        return false;
    }
    let line_count = rs_cursor_get_line_count(win);
    lnum <= line_count
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_pos_new() {
        let pos = CursorPos::new(10, 5, 2);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
        assert_eq!(pos.coladd, 2);
    }

    #[test]
    fn test_cursor_pos_line_start() {
        let pos = CursorPos::line_start(5);
        assert_eq!(pos.lnum, 5);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
    }

    #[test]
    fn test_cursor_pos_is_valid() {
        assert!(CursorPos::new(1, 0, 0).is_valid());
        assert!(!CursorPos::new(0, 0, 0).is_valid());
        assert!(!CursorPos::new(-1, 0, 0).is_valid());
    }

    #[test]
    fn test_cursor_pos_default() {
        let pos = CursorPos::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
        assert!(!pos.is_valid());
    }

    #[test]
    fn test_cursor_pos_cmp() {
        let a = CursorPos::new(1, 0, 0);
        let b = CursorPos::new(2, 0, 0);
        assert!(rs_cursor_pos_cmp(&a, &b) < 0);
        assert!(rs_cursor_pos_cmp(&b, &a) > 0);
        assert_eq!(rs_cursor_pos_cmp(&a, &a), 0);

        // Same line, different column
        let c = CursorPos::new(1, 5, 0);
        assert!(rs_cursor_pos_cmp(&a, &c) < 0);

        // Same line and column, different coladd
        let d = CursorPos::new(1, 5, 2);
        assert!(rs_cursor_pos_cmp(&c, &d) < 0);
    }

    #[test]
    fn test_cursor_pos_equal() {
        let a = CursorPos::new(1, 5, 2);
        let b = CursorPos::new(1, 5, 2);
        let c = CursorPos::new(1, 5, 3);

        assert!(rs_cursor_pos_equal(&a, &b));
        assert!(!rs_cursor_pos_equal(&a, &c));
    }

    #[test]
    fn test_cursor_pos_less() {
        let a = CursorPos::new(1, 5, 2);
        let b = CursorPos::new(1, 5, 3);
        let c = CursorPos::new(2, 0, 0);

        assert!(rs_cursor_pos_less(&a, &b));
        assert!(rs_cursor_pos_less(&a, &c));
        assert!(!rs_cursor_pos_less(&a, &a));
    }

    #[test]
    fn test_cursor_pos_leq() {
        let a = CursorPos::new(1, 5, 2);
        let b = CursorPos::new(1, 5, 2);
        let c = CursorPos::new(1, 5, 3);

        assert!(rs_cursor_pos_leq(&a, &b));
        assert!(rs_cursor_pos_leq(&a, &c));
        assert!(!rs_cursor_pos_leq(&c, &a));
    }

    #[test]
    fn test_maxcol() {
        assert_eq!(MAXCOL, i32::MAX);
    }

    #[test]
    fn test_ve_flags() {
        assert_eq!(VE_BLOCK, 0x01);
        assert_eq!(VE_INSERT, 0x02);
        assert_eq!(VE_ALL, 0x04);
        assert_eq!(VE_ONEMORE, 0x08);
        assert_eq!(VE_NONE, 0x10);
        assert_eq!(VE_NONEU, 0x20);
    }

    #[test]
    fn test_mode_flags() {
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(MODE_TERMINAL, 0x2000);
    }
}
