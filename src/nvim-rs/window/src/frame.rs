//! Frame tree operations.
//!
//! This module provides Rust implementations of frame tree operations
//! from `src/nvim/window.c`.
//!
//! The frame tree represents the hierarchical layout of windows. Each frame
//! is either:
//! - A leaf (`FR_LEAF`) containing a single window
//! - A row (`FR_ROW`) containing horizontally arranged child frames
//! - A column (`FR_COL`) containing vertically arranged child frames

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

extern "C" {
    /// Get the `w_p_wfh` (winfixheight) field from a window.
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;

    /// Get the `w_p_wfw` (winfixwidth) field from a window.
    fn nvim_win_get_wfw(wp: WinHandle) -> c_int;
}

/// Check if a frame has fixed height (due to 'winfixheight').
///
/// This is the Rust equivalent of `frame_fixed_height()` in window.c.
/// - Leaf frame: fixed if window has 'winfixheight' set
/// - Row frame: fixed if ANY child is fixed
/// - Column frame: fixed if ALL children are fixed
#[inline]
#[must_use]
pub(crate) fn frame_fixed_height_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*frp;
        let layout = frame.fr_layout;

        if layout == FR_LEAF {
            // Leaf frame with a window - check w_p_wfh
            let win = frame.fr_win;
            return !win.is_null() && nvim_win_get_wfh(win) != 0;
        }

        if layout == FR_ROW {
            // Row: fixed if ONE of the frames in the row is fixed
            let mut child = frame.fr_child;
            while !child.is_null() {
                if frame_fixed_height_impl(child) {
                    return true;
                }
                child = (*child).fr_next;
            }
            return false;
        }

        // FR_COL: fixed if ALL frames in the column are fixed
        let mut child = frame.fr_child;
        while !child.is_null() {
            if !frame_fixed_height_impl(child) {
                return false;
            }
            child = (*child).fr_next;
        }
        // All children are fixed (or no children)
        !frame.fr_child.is_null()
    }
}

/// Check if a frame has fixed width (due to 'winfixwidth').
///
/// This is the Rust equivalent of `frame_fixed_width()` in window.c.
/// - Leaf frame: fixed if window has 'winfixwidth' set
/// - Column frame: fixed if ANY child is fixed
/// - Row frame: fixed if ALL children are fixed
#[inline]
#[must_use]
pub(crate) fn frame_fixed_width_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: We check for null above
    unsafe {
        let frame = &*frp;
        let layout = frame.fr_layout;

        if layout == FR_LEAF {
            // Leaf frame with a window - check w_p_wfw
            let win = frame.fr_win;
            return !win.is_null() && nvim_win_get_wfw(win) != 0;
        }

        if layout == FR_COL {
            // Column: fixed if ONE of the frames in the column is fixed
            let mut child = frame.fr_child;
            while !child.is_null() {
                if frame_fixed_width_impl(child) {
                    return true;
                }
                child = (*child).fr_next;
            }
            return false;
        }

        // FR_ROW: fixed if ALL frames in the row are fixed
        let mut child = frame.fr_child;
        while !child.is_null() {
            if !frame_fixed_width_impl(child) {
                return false;
            }
            child = (*child).fr_next;
        }
        // All children are fixed (or no children)
        !frame.fr_child.is_null()
    }
}

/// Check that "topfrp" and its children are at the right height.
///
/// This is the Rust equivalent of `frame_check_height()` in window.c.
/// If the frame is a FR_ROW layout, all children must have the same height.
#[inline]
#[must_use]
pub(crate) fn frame_check_height_impl(topfrp: *const Frame, height: c_int) -> bool {
    if topfrp.is_null() {
        return false;
    }

    // SAFETY: We check for null above.
    unsafe {
        let frame = &*topfrp;
        if frame.fr_height != height {
            return false;
        }
        // If it's a row layout, check all children have the same height
        if frame.fr_layout == FR_ROW {
            let mut child = frame.fr_child;
            while !child.is_null() {
                if (*child).fr_height != height {
                    return false;
                }
                child = (*child).fr_next;
            }
        }
    }
    true
}

/// Check that "topfrp" and its children are at the right width.
///
/// This is the Rust equivalent of `frame_check_width()` in window.c.
/// If the frame is a FR_COL layout, all children must have the same width.
#[inline]
#[must_use]
pub(crate) fn frame_check_width_impl(topfrp: *const Frame, width: c_int) -> bool {
    if topfrp.is_null() {
        return false;
    }

    // SAFETY: We check for null above.
    unsafe {
        let frame = &*topfrp;
        if frame.fr_width != width {
            return false;
        }
        // If it's a column layout, check all children have the same width
        if frame.fr_layout == FR_COL {
            let mut child = frame.fr_child;
            while !child.is_null() {
                if (*child).fr_width != width {
                    return false;
                }
                child = (*child).fr_next;
            }
        }
    }
    true
}

// Note: FFI wrappers rs_frame_fixed_height, rs_frame_fixed_width,
// rs_frame_check_height, rs_frame_check_width are in lib.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_null() {
        let null_frame: *const Frame = std::ptr::null();
        assert!(!frame_fixed_height_impl(null_frame));
        assert!(!frame_fixed_width_impl(null_frame));
        assert!(!frame_check_height_impl(null_frame, 10));
        assert!(!frame_check_width_impl(null_frame, 80));
    }
}
