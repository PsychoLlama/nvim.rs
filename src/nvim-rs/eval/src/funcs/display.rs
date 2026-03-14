//! UI and display functions for VimL.
//!
//! This module implements display/UI-related VimL functions from `src/nvim/eval/funcs.c`:
//! - `screenrow()`, `screencol()` - Cursor screen position
//! - `winwidth()`, `winheight()` - Window dimensions
//! - `virtcol()` - Virtual column position
//! - `col()`, `line()` - Buffer position helpers
//!
//! These are mostly helpers; actual UI operations use the nvim-ui crate.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::manual_range_contains)]

use std::ffi::c_int;

// =============================================================================
// Screen Position Types
// =============================================================================

/// Screen position (row, col in screen coordinates).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScreenPos {
    /// Row (1-based, like Vim)
    pub row: c_int,
    /// Column (1-based)
    pub col: c_int,
}

impl ScreenPos {
    /// Create a new screen position.
    pub const fn new(row: c_int, col: c_int) -> Self {
        Self { row, col }
    }

    /// Create from 0-based coordinates.
    pub const fn from_0based(row: c_int, col: c_int) -> Self {
        Self {
            row: row + 1,
            col: col + 1,
        }
    }

    /// Convert to 0-based coordinates.
    pub const fn to_0based(self) -> (c_int, c_int) {
        (self.row - 1, self.col - 1)
    }

    /// Check if position is valid (positive).
    pub const fn is_valid(self) -> bool {
        self.row > 0 && self.col > 0
    }
}

/// Window dimensions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WinSize {
    /// Width in columns
    pub width: c_int,
    /// Height in rows
    pub height: c_int,
}

impl WinSize {
    /// Create new window size.
    pub const fn new(width: c_int, height: c_int) -> Self {
        Self { width, height }
    }

    /// Check if size is valid (positive).
    pub const fn is_valid(self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Calculate area (cells).
    pub const fn area(self) -> c_int {
        self.width * self.height
    }
}

// =============================================================================
// Virtual Column Helpers
// =============================================================================

/// Virtual column calculation flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtcolFlags {
    /// Count tabs as their display width
    pub count_tabs: bool,
    /// Include cursor position
    pub cursor_bound: bool,
    /// Count list mode chars
    pub list_mode: bool,
}

impl VirtcolFlags {
    /// Create default flags.
    pub const fn new() -> Self {
        Self {
            count_tabs: true,
            cursor_bound: false,
            list_mode: false,
        }
    }

    /// Create flags for cursor position.
    pub const fn cursor() -> Self {
        Self {
            count_tabs: true,
            cursor_bound: true,
            list_mode: false,
        }
    }
}

/// Calculate the display width of a character.
///
/// Tab characters expand to the next tabstop.
/// Wide characters (CJK) typically have width 2.
/// Control characters display as ^X.
pub fn char_display_width(c: u8, col: c_int, tabstop: c_int) -> c_int {
    match c {
        b'\t' => {
            // Tab expands to next tabstop
            if tabstop <= 0 {
                1
            } else {
                tabstop - (col % tabstop)
            }
        }
        0x00..=0x1F | 0x7F => {
            // Control characters display as ^X (width 2)
            2
        }
        _ => {
            // Normal characters
            1
        }
    }
}

/// FFI export: get character display width with tab expansion.
#[no_mangle]
pub extern "C" fn rs_char_display_width_with_tab(c: c_int, col: c_int, tabstop: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 1;
    }
    char_display_width(c as u8, col, tabstop)
}

/// Calculate virtual column from byte column.
///
/// Scans the line and accumulates display widths to find the virtual
/// column corresponding to a byte position.
pub fn byte_to_virtcol(line: &[u8], byte_col: usize, tabstop: c_int) -> c_int {
    let mut virtcol: c_int = 0;

    for (i, &c) in line.iter().enumerate() {
        if i >= byte_col {
            break;
        }
        virtcol += char_display_width(c, virtcol, tabstop);
    }

    // Virtual columns are 1-based in Vim
    virtcol + 1
}

/// Calculate byte column from virtual column.
///
/// Inverse of `byte_to_virtcol`. Returns the byte position that
/// corresponds to or just before the given virtual column.
pub fn virtcol_to_byte(line: &[u8], virtcol: c_int, tabstop: c_int) -> usize {
    if virtcol <= 1 {
        return 0;
    }

    let target = virtcol - 1; // Convert to 0-based
    let mut current_virtcol: c_int = 0;

    for (i, &c) in line.iter().enumerate() {
        let width = char_display_width(c, current_virtcol, tabstop);
        if current_virtcol + width > target {
            return i;
        }
        current_virtcol += width;
    }

    line.len()
}

/// FFI export: byte to virtual column conversion.
#[no_mangle]
pub unsafe extern "C" fn rs_byte_to_virtcol(
    line: *const u8,
    len: c_int,
    byte_col: c_int,
    tabstop: c_int,
) -> c_int {
    if line.is_null() || len < 0 || byte_col < 0 {
        return 1;
    }
    let slice = std::slice::from_raw_parts(line, len as usize);
    byte_to_virtcol(slice, byte_col as usize, tabstop)
}

/// FFI export: virtual column to byte conversion.
#[no_mangle]
pub unsafe extern "C" fn rs_virtcol_to_byte(
    line: *const u8,
    len: c_int,
    virtcol: c_int,
    tabstop: c_int,
) -> c_int {
    if line.is_null() || len < 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(line, len as usize);
    virtcol_to_byte(slice, virtcol, tabstop) as c_int
}

// =============================================================================
// Position Validation
// =============================================================================

/// Check if a line number is valid for a buffer.
pub const fn is_valid_lnum(lnum: c_int, buf_lines: c_int) -> bool {
    lnum >= 1 && lnum <= buf_lines
}

/// Helper for const max.
const fn const_max(a: c_int, b: c_int) -> c_int {
    if a > b {
        a
    } else {
        b
    }
}

/// Check if a column is valid for a line.
pub const fn is_valid_col(col: c_int, line_len: c_int, virtualedit: bool) -> bool {
    if virtualedit {
        col >= 1
    } else {
        col >= 1 && col <= const_max(line_len, 1)
    }
}

/// Clamp line number to valid range.
pub const fn clamp_lnum(lnum: c_int, buf_lines: c_int) -> c_int {
    if lnum < 1 {
        1
    } else if lnum > buf_lines {
        const_max(buf_lines, 1)
    } else {
        lnum
    }
}

/// Clamp column to valid range.
pub const fn clamp_col(col: c_int, line_len: c_int) -> c_int {
    if col < 1 {
        1
    } else if col > line_len {
        const_max(line_len, 1)
    } else {
        col
    }
}

/// FFI export: validate line number.
#[no_mangle]
pub extern "C" fn rs_is_valid_lnum(lnum: c_int, buf_lines: c_int) -> bool {
    is_valid_lnum(lnum, buf_lines)
}

/// FFI export: clamp line number.
#[no_mangle]
pub extern "C" fn rs_clamp_lnum(lnum: c_int, buf_lines: c_int) -> c_int {
    clamp_lnum(lnum, buf_lines)
}

/// FFI export: clamp column.
#[no_mangle]
pub extern "C" fn rs_clamp_col(col: c_int, line_len: c_int) -> c_int {
    clamp_col(col, line_len)
}

// =============================================================================
// Window Position Helpers
// =============================================================================

/// Convert window-local position to screen position.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WinScreenOffset {
    /// Window's row offset on screen
    pub row_offset: c_int,
    /// Window's column offset on screen
    pub col_offset: c_int,
}

impl WinScreenOffset {
    /// Create new offset.
    pub const fn new(row: c_int, col: c_int) -> Self {
        Self {
            row_offset: row,
            col_offset: col,
        }
    }

    /// Convert window position to screen position.
    pub const fn to_screen(self, win_row: c_int, win_col: c_int) -> ScreenPos {
        ScreenPos::new(self.row_offset + win_row, self.col_offset + win_col)
    }

    /// Convert screen position to window position.
    pub const fn from_screen(self, screen_row: c_int, screen_col: c_int) -> (c_int, c_int) {
        (screen_row - self.row_offset, screen_col - self.col_offset)
    }
}

/// FFI export: convert window position to screen.
#[no_mangle]
pub extern "C" fn rs_win_to_screen_pos(
    offset: *const WinScreenOffset,
    win_row: c_int,
    win_col: c_int,
) -> ScreenPos {
    if offset.is_null() {
        return ScreenPos::new(win_row, win_col);
    }
    unsafe { (*offset).to_screen(win_row, win_col) }
}

// =============================================================================
// Cursor Movement Bounds
// =============================================================================

/// Cursor movement boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CursorBounds {
    /// Minimum line number
    pub min_lnum: c_int,
    /// Maximum line number
    pub max_lnum: c_int,
    /// Minimum column (usually 1)
    pub min_col: c_int,
    /// Maximum column (line length or virtualedit)
    pub max_col: c_int,
    /// Virtual edit is enabled
    pub virtualedit: bool,
}

impl CursorBounds {
    /// Create bounds for a buffer.
    pub const fn for_buffer(buf_lines: c_int, line_len: c_int, virtualedit: bool) -> Self {
        Self {
            min_lnum: 1,
            max_lnum: const_max(buf_lines, 1),
            min_col: 1,
            max_col: if virtualedit {
                c_int::MAX
            } else {
                const_max(line_len, 1)
            },
            virtualedit,
        }
    }

    /// Check if position is within bounds.
    pub const fn contains(&self, lnum: c_int, col: c_int) -> bool {
        lnum >= self.min_lnum && lnum <= self.max_lnum && col >= self.min_col && col <= self.max_col
    }

    /// Clamp position to bounds.
    pub const fn clamp(&self, lnum: c_int, col: c_int) -> (c_int, c_int) {
        let clamped_lnum = if lnum < self.min_lnum {
            self.min_lnum
        } else if lnum > self.max_lnum {
            self.max_lnum
        } else {
            lnum
        };

        let clamped_col = if col < self.min_col {
            self.min_col
        } else if col > self.max_col {
            self.max_col
        } else {
            col
        };

        (clamped_lnum, clamped_col)
    }
}

impl Default for CursorBounds {
    fn default() -> Self {
        Self::for_buffer(1, 1, false)
    }
}

/// FFI export: create cursor bounds.
#[no_mangle]
pub extern "C" fn rs_cursor_bounds_for_buffer(
    buf_lines: c_int,
    line_len: c_int,
    virtualedit: bool,
) -> CursorBounds {
    CursorBounds::for_buffer(buf_lines, line_len, virtualedit)
}

/// FFI export: check cursor bounds.
#[no_mangle]
pub extern "C" fn rs_cursor_bounds_contains(
    bounds: *const CursorBounds,
    lnum: c_int,
    col: c_int,
) -> bool {
    if bounds.is_null() {
        return false;
    }
    unsafe { (*bounds).contains(lnum, col) }
}

// =============================================================================
// Screen Area Calculations
// =============================================================================

/// Calculate the number of screen lines needed to display a buffer line.
///
/// Takes line wrapping into account.
pub fn plines_for_line(line_len: c_int, win_width: c_int, wrap: bool) -> c_int {
    if !wrap || win_width <= 0 {
        return 1;
    }

    if line_len <= 0 {
        return 1;
    }

    // Number of wrapped lines
    (line_len + win_width - 1) / win_width
}

/// FFI export: calculate physical lines for a buffer line.
#[no_mangle]
pub extern "C" fn rs_plines_for_line(line_len: c_int, win_width: c_int, wrap: bool) -> c_int {
    plines_for_line(line_len, win_width, wrap)
}

// =============================================================================
// Screen character VimL functions (Phase 2)
// =============================================================================

use std::ffi::c_void;

extern "C" {
    fn nvim_eval_screenattr(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_screenchar(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_screenchars(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_screenstring(argvars: *const c_void, rettv: *mut c_void);
}

/// "screenattr(row, col)" function - get highlight attr at screen position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_screenattr"]
pub unsafe extern "C" fn rs_f_screenattr(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_screenattr(argvars, rettv);
}

/// "screenchar(row, col)" function - get character at screen position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_screenchar"]
pub unsafe extern "C" fn rs_f_screenchar(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_screenchar(argvars, rettv);
}

/// "screenchars(row, col)" function - get characters at screen position as list
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_screenchars"]
pub unsafe extern "C" fn rs_f_screenchars(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_screenchars(argvars, rettv);
}

/// "screenstring(row, col)" function - get string at screen position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_screenstring"]
pub unsafe extern "C" fn rs_f_screenstring(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_screenstring(argvars, rettv);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_pos() {
        let pos = ScreenPos::new(10, 20);
        assert!(pos.is_valid());
        assert_eq!(pos.to_0based(), (9, 19));

        let pos = ScreenPos::from_0based(5, 10);
        assert_eq!(pos.row, 6);
        assert_eq!(pos.col, 11);
    }

    #[test]
    fn test_win_size() {
        let size = WinSize::new(80, 24);
        assert!(size.is_valid());
        assert_eq!(size.area(), 1920);
    }

    #[test]
    fn test_char_display_width() {
        // Normal characters
        assert_eq!(char_display_width(b'a', 0, 8), 1);

        // Tabs
        assert_eq!(char_display_width(b'\t', 0, 8), 8);
        assert_eq!(char_display_width(b'\t', 4, 8), 4);
        assert_eq!(char_display_width(b'\t', 7, 8), 1);

        // Control characters
        assert_eq!(char_display_width(0x01, 0, 8), 2);
        assert_eq!(char_display_width(0x7F, 0, 8), 2);
    }

    #[test]
    fn test_byte_to_virtcol() {
        // Simple text
        assert_eq!(byte_to_virtcol(b"hello", 0, 8), 1);
        assert_eq!(byte_to_virtcol(b"hello", 3, 8), 4);
        assert_eq!(byte_to_virtcol(b"hello", 5, 8), 6);

        // With tabs
        assert_eq!(byte_to_virtcol(b"\thello", 0, 8), 1);
        assert_eq!(byte_to_virtcol(b"\thello", 1, 8), 9); // Tab is 8 wide at col 0
        assert_eq!(byte_to_virtcol(b"\thello", 2, 8), 10);
    }

    #[test]
    fn test_virtcol_to_byte() {
        // Simple text
        assert_eq!(virtcol_to_byte(b"hello", 1, 8), 0);
        assert_eq!(virtcol_to_byte(b"hello", 4, 8), 3);

        // With tabs
        assert_eq!(virtcol_to_byte(b"\thello", 1, 8), 0);
        assert_eq!(virtcol_to_byte(b"\thello", 8, 8), 0); // Still on tab
        assert_eq!(virtcol_to_byte(b"\thello", 9, 8), 1);
    }

    #[test]
    fn test_position_validation() {
        assert!(is_valid_lnum(1, 100));
        assert!(is_valid_lnum(100, 100));
        assert!(!is_valid_lnum(0, 100));
        assert!(!is_valid_lnum(101, 100));

        assert!(is_valid_col(1, 80, false));
        assert!(is_valid_col(80, 80, false));
        assert!(!is_valid_col(81, 80, false));
        assert!(is_valid_col(1000, 80, true)); // virtualedit
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp_lnum(0, 100), 1);
        assert_eq!(clamp_lnum(50, 100), 50);
        assert_eq!(clamp_lnum(150, 100), 100);

        assert_eq!(clamp_col(0, 80), 1);
        assert_eq!(clamp_col(40, 80), 40);
        assert_eq!(clamp_col(100, 80), 80);
    }

    #[test]
    fn test_win_screen_offset() {
        let offset = WinScreenOffset::new(5, 10);
        let screen = offset.to_screen(3, 4);
        assert_eq!(screen.row, 8);
        assert_eq!(screen.col, 14);

        let (win_row, win_col) = offset.from_screen(8, 14);
        assert_eq!(win_row, 3);
        assert_eq!(win_col, 4);
    }

    #[test]
    fn test_cursor_bounds() {
        let bounds = CursorBounds::for_buffer(100, 80, false);
        assert!(bounds.contains(50, 40));
        assert!(!bounds.contains(0, 40));
        assert!(!bounds.contains(50, 81));

        let (lnum, col) = bounds.clamp(150, 100);
        assert_eq!(lnum, 100);
        assert_eq!(col, 80);
    }

    #[test]
    fn test_plines() {
        // No wrap
        assert_eq!(plines_for_line(100, 80, false), 1);

        // With wrap
        assert_eq!(plines_for_line(80, 80, true), 1);
        assert_eq!(plines_for_line(81, 80, true), 2);
        assert_eq!(plines_for_line(160, 80, true), 2);
        assert_eq!(plines_for_line(161, 80, true), 3);

        // Edge cases
        assert_eq!(plines_for_line(0, 80, true), 1);
        assert_eq!(plines_for_line(100, 0, true), 1);
    }
}
