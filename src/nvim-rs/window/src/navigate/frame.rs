//! Frame-based window navigation helpers.
//!
//! This module provides helper functions for frame tree traversal
//! during navigation operations, complementing the position-based
//! navigation in movement.rs.

// Frame pointer variables may have similar names (fr1/fr2, frp1/frp2).
// Layout values need truncation when converting from c_int to i8.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::similar_names)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;
}

// =============================================================================
// Frame Tree Navigation
// =============================================================================

/// Find the leftmost leaf frame in a tree.
fn find_leftmost_leaf_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut fr = frp.cast_mut();
        while (*fr).fr_layout != FR_LEAF {
            fr = (*fr).fr_child;
            if fr.is_null() {
                return std::ptr::null_mut();
            }
        }
        fr
    }
}

/// Find the rightmost leaf frame in a tree.
fn find_rightmost_leaf_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut fr = frp.cast_mut();
        while (*fr).fr_layout != FR_LEAF {
            // Go to last child
            fr = (*fr).fr_child;
            if fr.is_null() {
                return std::ptr::null_mut();
            }
            while !(*fr).fr_next.is_null() {
                fr = (*fr).fr_next;
            }
        }
        fr
    }
}

/// Find the topmost leaf frame in a tree.
fn find_topmost_leaf_impl(frp: *const Frame) -> *mut Frame {
    // For vertical layout (FR_COL), topmost is first child
    // Otherwise same as leftmost
    find_leftmost_leaf_impl(frp)
}

/// Find the bottommost leaf frame in a tree.
fn find_bottommost_leaf_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut fr = frp.cast_mut();
        while (*fr).fr_layout != FR_LEAF {
            // For FR_COL, go to last child for bottommost
            // For FR_ROW, go to first child (all have same bottom)
            if (*fr).fr_layout == FR_COL {
                fr = (*fr).fr_child;
                if fr.is_null() {
                    return std::ptr::null_mut();
                }
                while !(*fr).fr_next.is_null() {
                    fr = (*fr).fr_next;
                }
            } else {
                fr = (*fr).fr_child;
                if fr.is_null() {
                    return std::ptr::null_mut();
                }
            }
        }
        fr
    }
}

// =============================================================================
// Frame Sibling Navigation
// =============================================================================

/// Get the next leaf frame in tree order.
fn get_next_leaf_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // If there's a next sibling, find its leftmost leaf
        if !(*frp).fr_next.is_null() {
            return find_leftmost_leaf_impl((*frp).fr_next);
        }

        // Walk up to find an ancestor with a next sibling
        let mut fr = (*frp).fr_parent;
        while !fr.is_null() {
            if !(*fr).fr_next.is_null() {
                return find_leftmost_leaf_impl((*fr).fr_next);
            }
            fr = (*fr).fr_parent;
        }

        std::ptr::null_mut()
    }
}

/// Get the previous leaf frame in tree order.
fn get_prev_leaf_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // If there's a prev sibling, find its rightmost leaf
        if !(*frp).fr_prev.is_null() {
            return find_rightmost_leaf_impl((*frp).fr_prev);
        }

        // Walk up to find an ancestor with a prev sibling
        let mut fr = (*frp).fr_parent;
        while !fr.is_null() {
            if !(*fr).fr_prev.is_null() {
                return find_rightmost_leaf_impl((*fr).fr_prev);
            }
            fr = (*fr).fr_parent;
        }

        std::ptr::null_mut()
    }
}

// =============================================================================
// Frame Layout Queries
// =============================================================================

/// Check if frame is in a horizontal split (FR_ROW parent).
fn is_in_horizontal_split_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        !parent.is_null() && (*parent).fr_layout == FR_ROW
    }
}

/// Check if frame is in a vertical split (FR_COL parent).
fn is_in_vertical_split_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        !parent.is_null() && (*parent).fr_layout == FR_COL
    }
}

/// Get the depth of a frame in the tree.
fn get_frame_depth_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let topframe = nvim_get_topframe();
        let mut depth = 0;
        let mut fr = (*frp).fr_parent;

        while !fr.is_null() && fr != topframe {
            depth += 1;
            fr = (*fr).fr_parent;
        }
        depth
    }
}

/// Find the common ancestor of two frames.
fn find_common_ancestor_impl(frp1: *const Frame, frp2: *const Frame) -> *mut Frame {
    if frp1.is_null() || frp2.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Get depth of both frames
        let depth1 = get_frame_depth_impl(frp1);
        let depth2 = get_frame_depth_impl(frp2);

        let mut fr1 = frp1.cast_mut();
        let mut fr2 = frp2.cast_mut();

        // Bring both to same depth
        let mut d1 = depth1;
        while d1 > depth2 {
            fr1 = (*fr1).fr_parent;
            d1 -= 1;
        }

        let mut d2 = depth2;
        while d2 > depth1 {
            fr2 = (*fr2).fr_parent;
            d2 -= 1;
        }

        // Walk up together until we find common parent
        while !fr1.is_null() && !fr2.is_null() && fr1 != fr2 {
            fr1 = (*fr1).fr_parent;
            fr2 = (*fr2).fr_parent;
        }

        if fr1 == fr2 {
            fr1
        } else {
            std::ptr::null_mut()
        }
    }
}

// =============================================================================
// Window Frame Helpers
// =============================================================================

/// Get frame from window.
fn get_win_frame_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { nvim_win_get_frame(wp) }
}

/// Get current window's frame.
fn get_curwin_frame_impl() -> *mut Frame {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return std::ptr::null_mut();
        }
        nvim_win_get_frame(curwin)
    }
}

/// Check if two windows share a common layout ancestor.
fn windows_share_layout_impl(wp1: WinHandle, wp2: WinHandle, layout: i8) -> bool {
    if wp1.is_null() || wp2.is_null() {
        return false;
    }

    unsafe {
        let fr1 = nvim_win_get_frame(wp1);
        let fr2 = nvim_win_get_frame(wp2);

        if fr1.is_null() || fr2.is_null() {
            return false;
        }

        let common = find_common_ancestor_impl(fr1, fr2);
        if common.is_null() {
            return false;
        }

        (*common).fr_layout == layout
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Find leftmost leaf frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_leftmost_leaf(frp: *const Frame) -> *mut Frame {
    find_leftmost_leaf_impl(frp)
}

/// FFI: Find rightmost leaf frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_rightmost_leaf(frp: *const Frame) -> *mut Frame {
    find_rightmost_leaf_impl(frp)
}

/// FFI: Find topmost leaf frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_topmost_leaf(frp: *const Frame) -> *mut Frame {
    find_topmost_leaf_impl(frp)
}

/// FFI: Find bottommost leaf frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_bottommost_leaf(frp: *const Frame) -> *mut Frame {
    find_bottommost_leaf_impl(frp)
}

/// FFI: Get next leaf frame in tree order.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_next_leaf(frp: *const Frame) -> *mut Frame {
    get_next_leaf_impl(frp)
}

/// FFI: Get previous leaf frame in tree order.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_prev_leaf(frp: *const Frame) -> *mut Frame {
    get_prev_leaf_impl(frp)
}

/// FFI: Check if frame is in horizontal split.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_is_horizontal_split(frp: *const Frame) -> c_int {
    c_int::from(is_in_horizontal_split_impl(frp))
}

/// FFI: Check if frame is in vertical split.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_is_vertical_split(frp: *const Frame) -> c_int {
    c_int::from(is_in_vertical_split_impl(frp))
}

/// FFI: Get frame depth.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_frame_depth(frp: *const Frame) -> c_int {
    get_frame_depth_impl(frp)
}

/// FFI: Find common ancestor of two frames.
///
/// # Safety
/// Caller must ensure both pointers are null or valid pointers to Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_nav_common_ancestor(
    frp1: *const Frame,
    frp2: *const Frame,
) -> *mut Frame {
    find_common_ancestor_impl(frp1, frp2)
}

/// FFI: Get window's frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_win_frame(wp: WinHandle) -> *mut Frame {
    get_win_frame_impl(wp)
}

/// FFI: Get current window's frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_curwin_frame() -> *mut Frame {
    get_curwin_frame_impl()
}

/// FFI: Check if windows share layout ancestor.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_windows_share_layout(
    wp1: WinHandle,
    wp2: WinHandle,
    layout: c_int,
) -> c_int {
    c_int::from(windows_share_layout_impl(wp1, wp2, layout as i8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_leaf_navigation() {
        assert!(find_leftmost_leaf_impl(std::ptr::null()).is_null());
        assert!(find_rightmost_leaf_impl(std::ptr::null()).is_null());
        assert!(find_topmost_leaf_impl(std::ptr::null()).is_null());
        assert!(find_bottommost_leaf_impl(std::ptr::null()).is_null());
        assert!(get_next_leaf_impl(std::ptr::null()).is_null());
        assert!(get_prev_leaf_impl(std::ptr::null()).is_null());
    }

    #[test]
    fn test_null_layout_checks() {
        assert!(!is_in_horizontal_split_impl(std::ptr::null()));
        assert!(!is_in_vertical_split_impl(std::ptr::null()));
        assert_eq!(get_frame_depth_impl(std::ptr::null()), 0);
    }

    #[test]
    fn test_null_common_ancestor() {
        assert!(find_common_ancestor_impl(std::ptr::null(), std::ptr::null()).is_null());
    }

    #[test]
    fn test_null_window_frame() {
        assert!(get_win_frame_impl(WinHandle::null()).is_null());
    }
}
