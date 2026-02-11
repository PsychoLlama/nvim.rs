//! Resize execution helper functions.
//!
//! This module provides helper functions for window resize execution,
//! supporting the C implementation of win_setheight, win_setwidth,
//! win_drag_status_line, win_drag_vsep_line, etc.
//!
//! Note: The main resize functions remain in lib.rs due to their complexity
//! and integration with frame operations.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL, FR_ROW};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get w_height from window.
    fn nvim_win_get_height(wp: WinHandle) -> c_int;

    /// Get w_width from window.
    fn nvim_win_get_width(wp: WinHandle) -> c_int;

    /// Get w_vsep_width from window.
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;

    /// Get w_status_height from window.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    /// Get w_hsep_height from window.
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
}

// =============================================================================
// Drag Status Line Helpers
// =============================================================================

/// Find the appropriate frame for vertical drag operations.
///
/// Returns the parent frame that's a FR_COL layout.
fn find_drag_frame_vertical_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut fr = nvim_win_get_frame(wp);
        if fr.is_null() {
            return std::ptr::null_mut();
        }

        let topframe = nvim_get_topframe();
        if fr == topframe {
            return std::ptr::null_mut();
        }

        // Find the parent that's a column layout
        let mut curfr = fr;
        fr = (*fr).fr_parent;

        while !fr.is_null() && (*fr).fr_layout != FR_COL {
            if fr == topframe {
                return std::ptr::null_mut();
            }
            curfr = fr;
            fr = (*fr).fr_parent;
        }

        if fr.is_null() {
            return std::ptr::null_mut();
        }

        curfr
    }
}

/// Find the appropriate frame for horizontal drag operations.
///
/// Returns the parent frame that's a FR_ROW layout.
fn find_drag_frame_horizontal_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut fr = nvim_win_get_frame(wp);
        if fr.is_null() {
            return std::ptr::null_mut();
        }

        let topframe = nvim_get_topframe();
        if fr == topframe {
            return std::ptr::null_mut();
        }

        // Find the parent that's a row layout
        let mut curfr = fr;
        fr = (*fr).fr_parent;

        while !fr.is_null() && (*fr).fr_layout != FR_ROW {
            if fr == topframe {
                return std::ptr::null_mut();
            }
            curfr = fr;
            fr = (*fr).fr_parent;
        }

        if fr.is_null() {
            return std::ptr::null_mut();
        }

        curfr
    }
}

/// Check if this is the last frame in its row (needs parent resize).
fn is_last_in_row_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return true;
    }

    unsafe { (*frp).fr_next.is_null() }
}

/// Check if this is the last frame in its column.
fn is_last_in_col_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return true;
    }

    unsafe { (*frp).fr_next.is_null() }
}

// =============================================================================
// Window Size Calculation Helpers
// =============================================================================

/// Get the total height a window uses including separators.
fn win_total_height_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        nvim_win_get_height(wp) + nvim_win_get_status_height(wp) + nvim_win_get_hsep_height(wp)
    }
}

/// Get the total width a window uses including separator.
fn win_total_width_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe { nvim_win_get_width(wp) + nvim_win_get_vsep_width(wp) }
}

/// Calculate how much height is available for the text area.
///
/// Subtracts status line and separator from frame height.
fn win_text_height_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return nvim_win_get_height(wp);
        }

        let frame_height = (*frame).fr_height;
        let status = nvim_win_get_status_height(wp);
        let hsep = nvim_win_get_hsep_height(wp);

        (frame_height - status - hsep).max(1)
    }
}

/// Calculate how much width is available for the text area.
///
/// Subtracts separator from frame width.
fn win_text_width_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return nvim_win_get_width(wp);
        }

        let frame_width = (*frame).fr_width;
        let vsep = nvim_win_get_vsep_width(wp);

        (frame_width - vsep).max(1)
    }
}

// =============================================================================
// Resize Direction Helpers
// =============================================================================

/// Check if window can be resized in vertical direction.
fn can_resize_vertical_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        if nvim_win_get_floating(wp) != 0 {
            return true; // Floating windows can always be resized
        }

        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return false;
        }

        // Check if there's a column layout in the parent chain
        let mut fr = (*frame).fr_parent;
        let topframe = nvim_get_topframe();

        while !fr.is_null() && fr != topframe {
            if (*fr).fr_layout == FR_COL {
                return true;
            }
            fr = (*fr).fr_parent;
        }

        // Can also resize if at topframe (adjusts cmdline)
        true
    }
}

/// Check if window can be resized in horizontal direction.
fn can_resize_horizontal_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        if nvim_win_get_floating(wp) != 0 {
            return true; // Floating windows can always be resized
        }

        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return false;
        }

        // Check if there's a row layout in the parent chain
        let mut fr = (*frame).fr_parent;
        let topframe = nvim_get_topframe();

        while !fr.is_null() && fr != topframe {
            if (*fr).fr_layout == FR_ROW {
                return true;
            }
            fr = (*fr).fr_parent;
        }

        false
    }
}

/// Get the frame that will grow when dragging down.
fn get_grow_frame_down_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*frp).fr_next }
}

/// Get the frame that will grow when dragging up.
fn get_grow_frame_up_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe { (*frp).fr_prev }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Find frame for vertical drag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_find_drag_vertical(wp: WinHandle) -> *mut Frame {
    find_drag_frame_vertical_impl(wp)
}

/// FFI: Find frame for horizontal drag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_find_drag_horizontal(wp: WinHandle) -> *mut Frame {
    find_drag_frame_horizontal_impl(wp)
}

/// FFI: Check if frame is last in row.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_is_last_in_row(frp: *const Frame) -> c_int {
    c_int::from(is_last_in_row_impl(frp))
}

/// FFI: Check if frame is last in column.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_is_last_in_col(frp: *const Frame) -> c_int {
    c_int::from(is_last_in_col_impl(frp))
}

/// FFI: Get total window height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_win_total_height(wp: WinHandle) -> c_int {
    win_total_height_impl(wp)
}

/// FFI: Get total window width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_win_total_width(wp: WinHandle) -> c_int {
    win_total_width_impl(wp)
}

/// FFI: Get text area height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_win_text_height(wp: WinHandle) -> c_int {
    win_text_height_impl(wp)
}

/// FFI: Get text area width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_win_text_width(wp: WinHandle) -> c_int {
    win_text_width_impl(wp)
}

/// FFI: Check if window can resize vertically.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_can_vertical(wp: WinHandle) -> c_int {
    c_int::from(can_resize_vertical_impl(wp))
}

/// FFI: Check if window can resize horizontally.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_can_horizontal(wp: WinHandle) -> c_int {
    c_int::from(can_resize_horizontal_impl(wp))
}

/// FFI: Get frame that grows when dragging down.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_grow_frame_down(frp: *const Frame) -> *mut Frame {
    get_grow_frame_down_impl(frp)
}

/// FFI: Get frame that grows when dragging up.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_grow_frame_up(frp: *const Frame) -> *mut Frame {
    get_grow_frame_up_impl(frp)
}

/// FFI: Get current window total height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_curwin_total_height() -> c_int {
    unsafe { win_total_height_impl(nvim_get_curwin()) }
}

/// FFI: Get current window total width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_curwin_total_width() -> c_int {
    unsafe { win_total_width_impl(nvim_get_curwin()) }
}

// =============================================================================
// Height/Width Setter Wrappers
// =============================================================================

extern "C" {
    fn rs_win_setheight_win(height: c_int, win: WinHandle);
    fn rs_win_setwidth_win(width: c_int, wp: WinHandle);
    fn nvim_win_field_height(wp: WinHandle) -> c_int;
    fn nvim_win_field_set_height(wp: WinHandle, val: c_int);
    fn nvim_win_field_set_width(wp: WinHandle, val: c_int);
    fn nvim_win_set_pos_changed(wp: WinHandle, val: c_int);
    fn nvim_win_set_inner_size(wp: WinHandle, valid_cursor: c_int);
}

/// Set current window height and take care of repositioning other windows.
///
/// Equivalent to C `win_setheight()` (window.c L6979).
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_setheight(height: c_int) {
    unsafe {
        let curwin = nvim_get_curwin();
        rs_win_setheight_win(height, curwin);
    }
}

/// Set current window width and take care of repositioning other windows.
///
/// Equivalent to C `win_setwidth()` (window.c L7009).
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_setwidth(width: c_int) {
    unsafe {
        let curwin = nvim_get_curwin();
        rs_win_setwidth_win(width, curwin);
    }
}

// =============================================================================
// Window Height/Width Setters
// =============================================================================

/// Set the height of a window.
/// "height" excludes any window toolbar.
/// This takes care of the things inside the window, not what happens to the
/// window position, the frame or to other windows.
///
/// Equivalent to C `win_new_height()` (window.c L7214).
pub(crate) fn win_new_height_impl(wp: WinHandle, height: c_int) {
    if wp.is_null() {
        return;
    }

    // Don't want a negative height. Happens when splitting a tiny window.
    let height = height.max(0);

    unsafe {
        if nvim_win_field_height(wp) == height {
            return; // nothing to do
        }

        nvim_win_field_set_height(wp, height);
        nvim_win_set_pos_changed(wp, 1);
        nvim_win_set_inner_size(wp, 1);
    }
}

/// Set the width of a window.
///
/// Equivalent to C `win_new_width()` (window.c L7393).
pub(crate) fn win_new_width_impl(wp: WinHandle, width: c_int) {
    if wp.is_null() {
        return;
    }

    unsafe {
        let w = if width < 0 { 0 } else { width };
        nvim_win_field_set_width(wp, w);
        nvim_win_set_pos_changed(wp, 1);
        nvim_win_set_inner_size(wp, 1);
    }
}

/// FFI: Set the height of a window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_new_height(wp: WinHandle, height: c_int) {
    win_new_height_impl(wp, height);
}

/// FFI: Set the width of a window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_new_width(wp: WinHandle, width: c_int) {
    win_new_width_impl(wp, width);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_window_total() {
        let null_wp = WinHandle::null();
        assert_eq!(win_total_height_impl(null_wp), 0);
        assert_eq!(win_total_width_impl(null_wp), 0);
    }

    #[test]
    fn test_null_frame_last() {
        assert!(is_last_in_row_impl(std::ptr::null()));
        assert!(is_last_in_col_impl(std::ptr::null()));
    }

    #[test]
    fn test_null_resize_checks() {
        let null_wp = WinHandle::null();
        assert!(!can_resize_vertical_impl(null_wp));
        assert!(!can_resize_horizontal_impl(null_wp));
    }

    #[test]
    fn test_null_drag_frames() {
        let null_wp = WinHandle::null();
        assert!(find_drag_frame_vertical_impl(null_wp).is_null());
        assert!(find_drag_frame_horizontal_impl(null_wp).is_null());
    }

    #[test]
    fn test_null_grow_frames() {
        assert!(get_grow_frame_down_impl(std::ptr::null()).is_null());
        assert!(get_grow_frame_up_impl(std::ptr::null()).is_null());
    }
}
