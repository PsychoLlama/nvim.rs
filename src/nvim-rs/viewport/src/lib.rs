//! Viewport and scroll management for Neovim
//!
//! This crate provides Rust implementations of viewport-related functions
//! from `src/nvim/move.c`. It handles:
//! - Viewport position tracking (topline, botline)
//! - Scroll offset calculations
//! - Cursor-to-screen mapping utilities
//! - Scroll margin handling
//!
//! The crate uses the opaque handle pattern for window access.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(dead_code)] // Some extern declarations are pre-declared for future use

use std::ffi::c_int;

use nvim_window::win_struct::win_ref;
use nvim_window::WinHandle;

// =============================================================================
// Scroll Direction Constants
// =============================================================================

/// Scroll direction: up (towards beginning of buffer)
pub const SCROLL_UP: c_int = 1;
/// Scroll direction: down (towards end of buffer)
pub const SCROLL_DOWN: c_int = 2;

// =============================================================================
// Cursor Position Flags (from move.c)
// =============================================================================

/// Valid window cursor row (`w_wrow`)
pub const VALID_WROW: c_int = 0x01;
/// Valid window cursor column (`w_wcol`)
pub const VALID_WCOL: c_int = 0x02;
/// Valid virtual column (`w_virtcol`)
pub const VALID_VIRTCOL: c_int = 0x04;
/// Valid `check_cursor_col` state
pub const VALID_CHEIGHT: c_int = 0x08;
/// Valid `w_crow`
pub const VALID_CROW: c_int = 0x10;
/// Valid botline/topline
pub const VALID_BOTLINE: c_int = 0x20;
/// `VALID_BOTLINE` is approximate
pub const VALID_BOTLINE_AP: c_int = 0x40;
/// Valid topline
pub const VALID_TOPLINE: c_int = 0x80;

// =============================================================================
// Line Offset Structure
// =============================================================================

/// Line offset for scroll calculations.
///
/// Used to track position during scroll operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LineOff {
    /// Line number
    pub lnum: i64,
    /// Offset for fill lines
    pub fill: c_int,
    /// Height in screen lines
    pub height: c_int,
}

impl LineOff {
    /// Create a new line offset.
    #[inline]
    #[must_use]
    pub const fn new(lnum: i64) -> Self {
        Self {
            lnum,
            fill: 0,
            height: 0,
        }
    }
}

// =============================================================================
// C Accessor Functions (kept for non-field accesses)
// =============================================================================

extern "C" {
    /// Get line count from buffer
    fn nvim_buf_get_line_count(buf: *mut std::ffi::c_void) -> i64;
    /// Get scrolloff option value for window
    fn rs_get_scrolloff_value(win: WinHandle) -> c_int;
    /// Get sidescrolloff option value for window
    fn rs_get_sidescrolloff_value(win: WinHandle) -> c_int;
}

// =============================================================================
// Viewport Query Functions
// =============================================================================

/// Get the topline (first visible line) of a window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_get_topline(win: WinHandle) -> i64 {
    i64::from(win_ref(win).w_topline)
}

/// Get the botline (last visible line + 1) of a window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_get_botline(win: WinHandle) -> i64 {
    i64::from(win_ref(win).w_botline)
}

/// Get the topfill (filler lines at top) of a window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_get_topfill(win: WinHandle) -> c_int {
    win_ref(win).w_topfill
}

/// Get the skipcol (columns skipped at start of first line) of a window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_get_skipcol(win: WinHandle) -> c_int {
    win_ref(win).w_skipcol
}

/// Get the leftcol (horizontal scroll position) of a window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_get_leftcol(win: WinHandle) -> c_int {
    win_ref(win).w_leftcol
}

/// Check if a line is visible in the window.
///
/// A line is visible if it's between topline and botline (exclusive).
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_line_visible(win: WinHandle, lnum: i64) -> bool {
    let ws = win_ref(win);
    let topline = i64::from(ws.w_topline);
    let botline = i64::from(ws.w_botline);
    lnum >= topline && lnum < botline
}

/// Get the number of visible lines in the window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_visible_lines(win: WinHandle) -> i64 {
    let ws = win_ref(win);
    let topline = i64::from(ws.w_topline);
    let botline = i64::from(ws.w_botline);
    (botline - topline).max(0)
}

/// Check if the viewport's topline is valid.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_topline_valid(win: WinHandle) -> bool {
    (win_ref(win).w_valid & VALID_TOPLINE) != 0
}

/// Check if the viewport's botline is valid.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_botline_valid(win: WinHandle) -> bool {
    (win_ref(win).w_valid & VALID_BOTLINE) != 0
}

// =============================================================================
// Scroll Margin Calculations
// =============================================================================

/// Get the effective scrolloff value for a window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_scrolloff(win: WinHandle) -> c_int {
    rs_get_scrolloff_value(win)
}

/// Get the effective sidescrolloff value for a window.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_sidescrolloff(win: WinHandle) -> c_int {
    rs_get_sidescrolloff_value(win)
}

/// Calculate the minimum topline to keep cursor visible with scrolloff.
///
/// Returns the minimum line that topline can be set to while keeping
/// the cursor line visible with the scrolloff margin.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_min_topline(win: WinHandle, cursor_lnum: i64) -> i64 {
    let scrolloff = rs_get_scrolloff_value(win);
    let min_top = cursor_lnum - i64::from(scrolloff);
    min_top.max(1)
}

/// Calculate the maximum topline to keep cursor visible with scrolloff.
///
/// Returns the maximum line that topline can be set to while keeping
/// the cursor line visible with the scrolloff margin.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_max_topline(win: WinHandle, cursor_lnum: i64) -> i64 {
    let view_height = win_ref(win).w_view_height;
    let scrolloff = rs_get_scrolloff_value(win);

    // topline can be at most cursor_lnum - (view_height - scrolloff - 1)
    let max_top = cursor_lnum - i64::from(view_height) + i64::from(scrolloff) + 1;
    max_top.max(1)
}

/// Check if the cursor is within the scroll margins.
///
/// Returns true if the cursor line is within the scrolloff margins
/// of the visible viewport.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_viewport_cursor_in_margin(win: WinHandle, cursor_lnum: i64) -> bool {
    let ws = win_ref(win);
    let topline = i64::from(ws.w_topline);
    let botline = i64::from(ws.w_botline);
    let scrolloff = rs_get_scrolloff_value(win);

    let top_margin = topline + i64::from(scrolloff);
    let bot_margin = botline - i64::from(scrolloff) - 1;

    cursor_lnum < top_margin || cursor_lnum > bot_margin
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_direction_constants() {
        assert_eq!(SCROLL_UP, 1);
        assert_eq!(SCROLL_DOWN, 2);
    }

    #[test]
    fn test_valid_flags() {
        assert_eq!(VALID_WROW, 0x01);
        assert_eq!(VALID_WCOL, 0x02);
        assert_eq!(VALID_VIRTCOL, 0x04);
        assert_eq!(VALID_CHEIGHT, 0x08);
        assert_eq!(VALID_CROW, 0x10);
        assert_eq!(VALID_BOTLINE, 0x20);
        assert_eq!(VALID_BOTLINE_AP, 0x40);
        assert_eq!(VALID_TOPLINE, 0x80);
    }

    #[test]
    fn test_lineoff_new() {
        let loff = LineOff::new(10);
        assert_eq!(loff.lnum, 10);
        assert_eq!(loff.fill, 0);
        assert_eq!(loff.height, 0);
    }

    #[test]
    fn test_lineoff_default() {
        let loff = LineOff::default();
        assert_eq!(loff.lnum, 0);
        assert_eq!(loff.fill, 0);
        assert_eq!(loff.height, 0);
    }
}
