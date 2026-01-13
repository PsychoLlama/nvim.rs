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
//!
//! # Submodules
//!
//! - [`constants`]: Frame direction constants, layout flags, split constraints
//! - [`types`]: Frame handle types, LayoutDir enum, SplitInfo struct
//! - [`accessors`]: Frame field getters/setters
//! - [`operations`]: Frame tree manipulation operations

#![allow(clippy::missing_const_for_fn)]

pub mod accessors;
pub mod constants;
pub mod operations;
pub mod types;

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

// =============================================================================
// FFI Exports for Frame Operations
// =============================================================================

/// FFI export: Get frame layout type
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_layout(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return -1;
    }
    c_int::from((*frp).fr_layout)
}

/// FFI export: Get frame width
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_width(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    (*frp).fr_width
}

/// FFI export: Get frame height
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_height(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    (*frp).fr_height
}

/// FFI export: Get frame newwidth
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_newwidth(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    (*frp).fr_newwidth
}

/// FFI export: Get frame newheight
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_newheight(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    (*frp).fr_newheight
}

/// FFI export: Get frame parent
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_parent(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    (*frp).fr_parent
}

/// FFI export: Get frame next sibling
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_next(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    (*frp).fr_next
}

/// FFI export: Get frame prev sibling
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_prev(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    (*frp).fr_prev
}

/// FFI export: Get frame first child
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_child(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    (*frp).fr_child
}

/// FFI export: Get frame window
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_win(frp: *const Frame) -> WinHandle {
    if frp.is_null() {
        return WinHandle::null();
    }
    (*frp).fr_win
}

/// FFI export: Check if frame is a leaf
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_is_leaf(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from((*frp).fr_layout == FR_LEAF)
}

/// FFI export: Check if frame is a row
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_is_row(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from((*frp).fr_layout == FR_ROW)
}

/// FFI export: Check if frame is a column
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_is_col(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from((*frp).fr_layout == FR_COL)
}

/// FFI export: Count children in frame
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_child_count(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    let mut count = 0;
    let mut child = (*frp).fr_child;
    while !child.is_null() {
        count += 1;
        child = (*child).fr_next;
    }
    count
}

/// FFI export: Check if frame has no children
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_has_no_children(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 1;
    }
    c_int::from((*frp).fr_child.is_null())
}

/// FFI export: Check if frame is the only child
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_is_only_child(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    let frame = &*frp;
    c_int::from(frame.fr_prev.is_null() && frame.fr_next.is_null())
}

/// FFI export: Get the FR_LEAF constant
#[unsafe(no_mangle)]
pub extern "C" fn rs_fr_leaf() -> c_int {
    c_int::from(FR_LEAF)
}

/// FFI export: Get the FR_ROW constant
#[unsafe(no_mangle)]
pub extern "C" fn rs_fr_row() -> c_int {
    c_int::from(FR_ROW)
}

/// FFI export: Get the FR_COL constant
#[unsafe(no_mangle)]
pub extern "C" fn rs_fr_col() -> c_int {
    c_int::from(FR_COL)
}

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
