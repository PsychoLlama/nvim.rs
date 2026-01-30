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
// Return Value Constants
// =============================================================================

/// Success return value
pub const OK: c_int = 1;
/// Failure return value
pub const FAIL: c_int = 0;

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

    // -------------------------------------------------------------------------
    // Cursor/Screen Column Functions
    // -------------------------------------------------------------------------

    /// Get character at cursor position (calls `get_cursor_pos_ptr` + `utf_ptr2char`)
    fn nvim_gchar_cursor() -> c_int;

    /// Get curwin pointer
    fn nvim_cursor_get_curwin() -> WinHandle;

    /// Get `curwin->w_cursor` pointer
    fn nvim_cursor_get_curwin_cursor() -> *mut CursorPos;

    /// Wrapper for `getvvcol` - gets virtual column positions
    fn nvim_getvvcol(
        wp: WinHandle,
        pos: *const CursorPos,
        scol: *mut i32,
        ccol: *mut i32,
        ecol: *mut i32,
    );

    /// Wrapper for `set_valid_virtcol`
    fn nvim_set_valid_virtcol(wp: WinHandle, vcol: i32);

    /// Wrapper for `virtual_active(win)`
    fn nvim_virtual_active_win(wp: WinHandle) -> bool;

    // -------------------------------------------------------------------------
    // Column Advancement Functions
    // -------------------------------------------------------------------------

    /// Wrapper for `getvpos` - advances cursor to screen column
    fn nvim_getvpos(wp: WinHandle, pos: *mut CursorPos, wcol: i32) -> c_int;

    /// Check if character at position is TAB
    fn nvim_char_at_pos_is_tab(wp: WinHandle, pos: *const CursorPos) -> bool;

    /// Clear `VALID_VIRTCOL` flag for window
    fn nvim_win_clear_valid_virtcol(wp: WinHandle);

    /// Get window cursor pointer (`wp->w_cursor`)
    fn nvim_win_get_cursor_ptr(wp: WinHandle) -> *mut CursorPos;
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
// Cursor Movement Helpers
// =============================================================================

/// Check if cursor can move up from current line.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_can_move_up(_win: WinHandle, lnum: i64) -> bool {
    lnum > 1
}

/// Check if cursor can move down from current line.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_can_move_down(win: WinHandle, lnum: i64) -> bool {
    let line_count = rs_cursor_get_line_count(win);
    lnum < line_count
}

/// Calculate the target line number when moving up by count lines.
/// Clamps to line 1 if count exceeds available lines.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_line_up(_win: WinHandle, lnum: i64, count: i64) -> i64 {
    let target = lnum - count;
    if target < 1 {
        1
    } else {
        target
    }
}

/// Calculate the target line number when moving down by count lines.
/// Clamps to last line if count exceeds available lines.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_line_down(win: WinHandle, lnum: i64, count: i64) -> i64 {
    let line_count = rs_cursor_get_line_count(win);
    let target = lnum + count;
    if target > line_count {
        line_count.max(1)
    } else {
        target
    }
}

/// Get the clamped column position for a line.
/// Returns the minimum of col and `line_len` - 1 (or 0 for empty lines).
/// When `allow_past_end` is true, allows col == `line_len`.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_clamp_col(
    win: WinHandle,
    lnum: i64,
    col: i32,
    allow_past_end: bool,
) -> i32 {
    let buf = nvim_win_get_buffer(win);
    if buf.is_null() {
        return 0;
    }
    let line_len = nvim_buf_get_line_len(buf, lnum);
    if line_len == 0 {
        return 0;
    }

    let max_col = if allow_past_end {
        line_len
    } else {
        (line_len - 1).max(0)
    };

    if col < 0 {
        0
    } else if col > max_col {
        max_col
    } else {
        col
    }
}

/// Check if the `one_more` condition is true.
/// This allows cursor to be past end of line when:
/// - In Insert mode
/// - In Terminal mode
/// - Insert mode restart is pending
/// - Visual mode is active with 'selection' != "old"
/// - 'virtualedit' has onemore flag
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_one_more(win: WinHandle) -> bool {
    let state = nvim_get_state();
    let ve_flags = nvim_get_ve_flags(win);
    let restart_edit = nvim_get_restart_edit();
    let visual_active = nvim_get_visual_active();
    let sel_first = nvim_get_p_sel_first();

    // Check each condition
    (state & MODE_INSERT) != 0
        || (state & MODE_TERMINAL) != 0
        || restart_edit != 0
        || (visual_active && sel_first != i32::from(b'o'))
        || (ve_flags & VE_ONEMORE) != 0
}

/// Check if position is at end of line (on the NUL byte).
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_at_eol(win: WinHandle, lnum: i64, col: i32) -> bool {
    let buf = nvim_win_get_buffer(win);
    if buf.is_null() {
        return true;
    }
    let line_len = nvim_buf_get_line_len(buf, lnum);
    col >= line_len
}

/// Check if position is at beginning of line.
#[no_mangle]
pub extern "C" fn rs_cursor_at_bol(col: i32) -> bool {
    col == 0
}

/// Check if position is at first line of buffer.
#[no_mangle]
pub extern "C" fn rs_cursor_at_first_line(lnum: i64) -> bool {
    lnum <= 1
}

/// Check if position is at last line of buffer.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_at_last_line(win: WinHandle, lnum: i64) -> bool {
    let line_count = rs_cursor_get_line_count(win);
    lnum >= line_count
}

// =============================================================================
// Character Access Functions
// =============================================================================

/// Get the character at the cursor position.
///
/// Returns the Unicode codepoint of the character under the cursor.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[no_mangle]
pub unsafe extern "C" fn rs_gchar_cursor() -> c_int {
    nvim_gchar_cursor()
}

// =============================================================================
// Screen Column Functions
// =============================================================================

/// Get the screen column of the cursor in the current window.
///
/// Returns the virtual column position (accounting for tabs, wide characters, etc.).
///
/// # Safety
/// Requires valid global state (curwin).
#[no_mangle]
pub unsafe extern "C" fn rs_getviscol() -> c_int {
    let curwin = nvim_cursor_get_curwin();
    let cursor = nvim_cursor_get_curwin_cursor();
    let mut x: i32 = 0;
    nvim_getvvcol(
        curwin,
        cursor,
        &raw mut x,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    x
}

/// Get the screen column for a given column and coladd in the cursor line.
///
/// # Arguments
/// * `col` - Column byte offset
/// * `coladd` - Virtual column addition
///
/// # Returns
/// The screen column position.
///
/// # Safety
/// Requires valid global state (curwin).
#[no_mangle]
pub unsafe extern "C" fn rs_getviscol2(col: i32, coladd: i32) -> c_int {
    let curwin = nvim_cursor_get_curwin();
    let cursor = nvim_cursor_get_curwin_cursor();

    // Build a temporary position with the cursor's line but specified col/coladd
    let pos = CursorPos {
        lnum: (*cursor).lnum,
        col,
        coladd,
    };

    let mut x: i32 = 0;
    nvim_getvvcol(
        curwin,
        &raw const pos,
        &raw mut x,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    x
}

// =============================================================================
// Column Advancement Functions
// =============================================================================

/// Return in `pos` the position of the cursor advanced to screen column `wcol`.
///
/// # Arguments
/// * `wp` - Window handle
/// * `pos` - Position to update
/// * `wcol` - Target screen column
///
/// # Returns
/// `OK` if desired column is reached, `FAIL` if not.
///
/// # Safety
/// `wp` and `pos` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_getvpos(wp: WinHandle, pos: *mut CursorPos, wcol: i32) -> c_int {
    nvim_getvpos(wp, pos, wcol)
}

/// Try to advance the cursor to the specified screen column.
///
/// If virtual editing is enabled, fine tunes the cursor position.
/// All virtual positions off the end of a line share a cursor.col value
/// (equal to strlen(line)), beginning at coladd 0.
///
/// # Arguments
/// * `wp` - Window handle
/// * `wcol` - Target screen column
///
/// # Returns
/// `OK` if desired column is reached, `FAIL` if not.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_coladvance(wp: WinHandle, wcol: i32) -> c_int {
    let cursor = nvim_win_get_cursor_ptr(wp);
    let rc = nvim_getvpos(wp, cursor, wcol);

    if wcol == MAXCOL || rc == FAIL {
        nvim_win_clear_valid_virtcol(wp);
    } else if !nvim_char_at_pos_is_tab(wp, cursor) {
        // Virtcol is valid when not on a TAB
        // Note: curwin is used here to match C behavior
        let curwin = nvim_cursor_get_curwin();
        nvim_set_valid_virtcol(curwin, wcol);
    }
    rc
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

    #[test]
    fn test_cursor_at_bol() {
        assert!(rs_cursor_at_bol(0));
        assert!(!rs_cursor_at_bol(1));
        assert!(!rs_cursor_at_bol(-1));
    }

    #[test]
    fn test_cursor_at_first_line() {
        assert!(rs_cursor_at_first_line(1));
        assert!(rs_cursor_at_first_line(0));
        assert!(rs_cursor_at_first_line(-1));
        assert!(!rs_cursor_at_first_line(2));
    }
}
