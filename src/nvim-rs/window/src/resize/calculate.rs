//! Resize calculation functions.
//!
//! This module provides calculation functions for window resize operations,
//! including room availability checks and minimum size calculations.

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
    fn nvim_win_field_height(wp: WinHandle) -> c_int;

    /// Get w_width from window.
    fn nvim_win_field_width(wp: WinHandle) -> c_int;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get frame minimum height.
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;

    /// Get frame minimum width.
    fn rs_frame_minwidth(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;
}

// =============================================================================
// Room Calculation Functions
// =============================================================================

/// Calculate available room in a frame for height change.
///
/// Returns how much the frame height can be reduced.
fn frame_room_height_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let frame = &*frp;
        let min_height = rs_frame_minheight(frp, WinHandle::null());
        (frame.fr_height - min_height).max(0)
    }
}

/// Calculate available room in a frame for width change.
///
/// Returns how much the frame width can be reduced.
fn frame_room_width_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let frame = &*frp;
        let min_width = rs_frame_minwidth(frp, WinHandle::null());
        (frame.fr_width - min_width).max(0)
    }
}

/// Sum room above a frame in a column layout (for drag up).
fn sum_room_above_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let frame = &*frp;
        let parent = frame.fr_parent;
        if parent.is_null() || (*parent).fr_layout != FR_COL {
            return frame_room_height_impl(frp);
        }

        // Sum room from start of column to this frame
        let mut total = 0;
        let mut child = (*parent).fr_child;
        while !child.is_null() {
            total += frame_room_height_impl(child);
            if std::ptr::eq(child, frp) {
                break;
            }
            child = (*child).fr_next;
        }
        total
    }
}

/// Sum room below a frame in a column layout (for drag down).
fn sum_room_below_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let frame = &*frp;

        // Sum room from next frame onwards
        let mut total = 0;
        let mut child = frame.fr_next;
        while !child.is_null() {
            total += frame_room_height_impl(child);
            child = (*child).fr_next;
        }
        total
    }
}

/// Sum room left of a frame in a row layout.
fn sum_room_left_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let frame = &*frp;
        let parent = frame.fr_parent;
        if parent.is_null() || (*parent).fr_layout != FR_ROW {
            return frame_room_width_impl(frp);
        }

        // Sum room from start of row to this frame
        let mut total = 0;
        let mut child = (*parent).fr_child;
        while !child.is_null() {
            total += frame_room_width_impl(child);
            if std::ptr::eq(child, frp) {
                break;
            }
            child = (*child).fr_next;
        }
        total
    }
}

/// Sum room right of a frame in a row layout.
fn sum_room_right_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let frame = &*frp;

        // Sum room from next frame onwards
        let mut total = 0;
        let mut child = frame.fr_next;
        while !child.is_null() {
            total += frame_room_width_impl(child);
            child = (*child).fr_next;
        }
        total
    }
}

// =============================================================================
// Frame Navigation for Resize
// =============================================================================

/// Find the parent frame for vertical resize (FR_COL).
fn find_vertical_parent_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut fr = nvim_win_get_frame(wp);
        if fr.is_null() {
            return std::ptr::null_mut();
        }

        // Walk up to find FR_COL parent
        fr = (*fr).fr_parent;
        while !fr.is_null() {
            if (*fr).fr_layout == FR_COL {
                return fr;
            }
            fr = (*fr).fr_parent;
        }
        std::ptr::null_mut()
    }
}

/// Find the parent frame for horizontal resize (FR_ROW).
fn find_horizontal_parent_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut fr = nvim_win_get_frame(wp);
        if fr.is_null() {
            return std::ptr::null_mut();
        }

        // Walk up to find FR_ROW parent
        fr = (*fr).fr_parent;
        while !fr.is_null() {
            if (*fr).fr_layout == FR_ROW {
                return fr;
            }
            fr = (*fr).fr_parent;
        }
        std::ptr::null_mut()
    }
}

/// Check if a frame is at the topframe level.
fn is_at_topframe_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        let topframe = nvim_get_topframe();
        std::ptr::eq(frp, topframe.cast_const())
    }
}

/// Check if window's frame has room to grow in height.
fn can_grow_height_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return false;
        }

        // Check if there's a sibling that can shrink
        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return false;
        }

        if (*parent).fr_layout == FR_COL {
            // Check siblings in column
            let mut child = (*parent).fr_child;
            while !child.is_null() {
                if child != frame && frame_room_height_impl(child) > 0 {
                    return true;
                }
                child = (*child).fr_next;
            }
        }
        false
    }
}

/// Check if window's frame has room to grow in width.
fn can_grow_width_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return false;
        }

        // Check if there's a sibling that can shrink
        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return false;
        }

        if (*parent).fr_layout == FR_ROW {
            // Check siblings in row
            let mut child = (*parent).fr_child;
            while !child.is_null() {
                if child != frame && frame_room_width_impl(child) > 0 {
                    return true;
                }
                child = (*child).fr_next;
            }
        }
        false
    }
}

// =============================================================================
// Size Validation
// =============================================================================

/// Check if requested height change is valid.
fn validate_height_change_impl(wp: WinHandle, delta: c_int) -> c_int {
    if wp.is_null() || delta == 0 {
        return 0;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return 0;
        }

        if delta > 0 {
            // Growing - check room in siblings
            let room = sum_room_above_impl(frame) + sum_room_below_impl(frame);
            delta.min(room)
        } else {
            // Shrinking - check own room
            let room = frame_room_height_impl(frame);
            delta.max(-room)
        }
    }
}

/// Check if requested width change is valid.
fn validate_width_change_impl(wp: WinHandle, delta: c_int) -> c_int {
    if wp.is_null() || delta == 0 {
        return 0;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return 0;
        }

        if delta > 0 {
            // Growing - check room in siblings
            let room = sum_room_left_impl(frame) + sum_room_right_impl(frame);
            delta.min(room)
        } else {
            // Shrinking - check own room
            let room = frame_room_width_impl(frame);
            delta.max(-room)
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Calculate available room in frame height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_frame_room_height(frp: *const Frame) -> c_int {
    frame_room_height_impl(frp)
}

/// FFI: Calculate available room in frame width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_frame_room_width(frp: *const Frame) -> c_int {
    frame_room_width_impl(frp)
}

/// FFI: Sum room above a frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_sum_room_above(frp: *const Frame) -> c_int {
    sum_room_above_impl(frp)
}

/// FFI: Sum room below a frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_sum_room_below(frp: *const Frame) -> c_int {
    sum_room_below_impl(frp)
}

/// FFI: Sum room left of a frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_sum_room_left(frp: *const Frame) -> c_int {
    sum_room_left_impl(frp)
}

/// FFI: Sum room right of a frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_sum_room_right(frp: *const Frame) -> c_int {
    sum_room_right_impl(frp)
}

/// FFI: Find vertical resize parent frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_find_vertical_parent(wp: WinHandle) -> *mut Frame {
    find_vertical_parent_impl(wp)
}

/// FFI: Find horizontal resize parent frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_find_horizontal_parent(wp: WinHandle) -> *mut Frame {
    find_horizontal_parent_impl(wp)
}

/// FFI: Check if frame is at topframe level.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_is_at_topframe(frp: *const Frame) -> c_int {
    c_int::from(is_at_topframe_impl(frp))
}

/// FFI: Check if window can grow in height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_can_grow_height(wp: WinHandle) -> c_int {
    c_int::from(can_grow_height_impl(wp))
}

/// FFI: Check if window can grow in width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_can_grow_width(wp: WinHandle) -> c_int {
    c_int::from(can_grow_width_impl(wp))
}

/// FFI: Validate and clamp height change.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_validate_height(wp: WinHandle, delta: c_int) -> c_int {
    validate_height_change_impl(wp, delta)
}

/// FFI: Validate and clamp width change.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_validate_width(wp: WinHandle, delta: c_int) -> c_int {
    validate_width_change_impl(wp, delta)
}

/// FFI: Get current window's frame room height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_curwin_room_height() -> c_int {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return 0;
        }
        let frame = nvim_win_get_frame(curwin);
        frame_room_height_impl(frame)
    }
}

/// FFI: Get current window's frame room width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_curwin_room_width() -> c_int {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return 0;
        }
        let frame = nvim_win_get_frame(curwin);
        frame_room_width_impl(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_frame_room() {
        assert_eq!(frame_room_height_impl(std::ptr::null()), 0);
        assert_eq!(frame_room_width_impl(std::ptr::null()), 0);
    }

    #[test]
    fn test_null_frame_sum() {
        assert_eq!(sum_room_above_impl(std::ptr::null()), 0);
        assert_eq!(sum_room_below_impl(std::ptr::null()), 0);
        assert_eq!(sum_room_left_impl(std::ptr::null()), 0);
        assert_eq!(sum_room_right_impl(std::ptr::null()), 0);
    }

    #[test]
    fn test_null_window_checks() {
        let null_wp = WinHandle::null();
        assert!(find_vertical_parent_impl(null_wp).is_null());
        assert!(find_horizontal_parent_impl(null_wp).is_null());
        assert!(!can_grow_height_impl(null_wp));
        assert!(!can_grow_width_impl(null_wp));
    }

    #[test]
    fn test_null_topframe() {
        assert!(!is_at_topframe_impl(std::ptr::null()));
    }
}
