//! Frame tree helpers for window closing.
//!
//! This module provides helper functions for frame tree manipulation
//! during window close operations, including finding which frame receives
//! the freed space and determining how to merge frames.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get w_frame from a window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get p_sb (splitbelow) option.
    fn nvim_get_p_sb() -> c_int;

    /// Get p_spr (splitright) option.
    fn nvim_get_p_spr() -> c_int;

    /// Check if frame has fixed height.
    fn rs_frame_fixed_height(frp: *const Frame) -> c_int;

    /// Check if frame has fixed width.
    fn rs_frame_fixed_width(frp: *const Frame) -> c_int;
}

// =============================================================================
// Frame Space Redistribution
// =============================================================================

/// Find the frame that will receive the space from a closed window.
///
/// This implements the winframe_find_altwin logic:
/// - Prefers previous sibling
/// - Falls back to next sibling
/// - For nested frames, considers parent layout
///
/// # Safety
/// Caller must ensure `wp` is a valid window handle with a valid frame.
fn find_altframe_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return std::ptr::null_mut();
        }

        // Try prev sibling first
        if !(*frame).fr_prev.is_null() {
            return (*frame).fr_prev;
        }

        // Then try next sibling
        if !(*frame).fr_next.is_null() {
            return (*frame).fr_next;
        }

        // No siblings - parent will receive the space
        (*frame).fr_parent
    }
}

/// Check if a frame should be flattened after child removal.
///
/// A frame should be flattened when it has only one child left.
fn frame_should_flatten_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        // Must be non-leaf
        if (*frp).fr_layout == FR_LEAF {
            return false;
        }

        // Must have exactly one child
        let child = (*frp).fr_child;
        if child.is_null() {
            return false;
        }

        (*child).fr_next.is_null() && (*child).fr_prev.is_null()
    }
}

/// Get the layout direction that the alt frame will expand in.
///
/// Returns 'v' for vertical expansion (FR_COL parent), 'h' for horizontal (FR_ROW).
fn get_expansion_direction_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return c_int::from(b'h');
    }

    unsafe {
        let parent = (*frp).fr_parent;
        if parent.is_null() {
            return c_int::from(b'h');
        }

        if (*parent).fr_layout == FR_COL {
            c_int::from(b'v')
        } else {
            c_int::from(b'h')
        }
    }
}

/// Check if removing frame would leave parent with single child.
fn removal_would_orphan_parent_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        if parent.is_null() {
            return false;
        }

        // Count siblings including self
        let mut count = 0;
        let mut child = (*parent).fr_child;
        while !child.is_null() {
            count += 1;
            child = (*child).fr_next;
        }

        count <= 2 // If 2 or less, removing one leaves 1 or 0
    }
}

/// Get the frame parent.
fn get_frame_parent_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { (*frp).fr_parent }
}

/// Check if frame is topframe.
fn is_topframe_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }
    unsafe {
        let topframe = nvim_get_topframe();
        std::ptr::eq(frp, topframe)
    }
}

// =============================================================================
// Frame Size Calculations for Close
// =============================================================================

/// Get the total height being freed from a frame and its children.
fn total_freed_height_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    unsafe { (*frp).fr_height }
}

/// Get the total width being freed from a frame and its children.
fn total_freed_width_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    unsafe { (*frp).fr_width }
}

/// Calculate how much height the alt frame will gain.
///
/// For FR_COL parent: alt frame gains the full height
/// For FR_ROW parent: no height change
fn height_gain_for_alt_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        if parent.is_null() {
            return (*frp).fr_height;
        }

        if (*parent).fr_layout == FR_COL {
            (*frp).fr_height
        } else {
            0
        }
    }
}

/// Calculate how much width the alt frame will gain.
///
/// For FR_ROW parent: alt frame gains the full width
/// For FR_COL parent: no width change
fn width_gain_for_alt_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        if parent.is_null() {
            return (*frp).fr_width;
        }

        if (*parent).fr_layout == FR_ROW {
            (*frp).fr_width
        } else {
            0
        }
    }
}

// =============================================================================
// Frame Validation for Close
// =============================================================================

/// Check if frame can be safely removed.
///
/// A frame can be removed if:
/// - It's not the topframe
/// - Its parent exists
fn can_remove_frame_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        // Can't remove topframe
        let topframe = nvim_get_topframe();
        if std::ptr::eq(frp, topframe) {
            return false;
        }

        // Must have a parent
        !(*frp).fr_parent.is_null()
    }
}

/// Check if closing this window would require layout adjustment.
///
/// Returns true if the frame tree structure would change beyond just
/// removing the window's frame.
fn close_needs_layout_adjustment_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return false;
        }

        // If no siblings, parent structure may change
        if (*frame).fr_prev.is_null() && (*frame).fr_next.is_null() {
            return true;
        }

        // If parent would be orphaned
        removal_would_orphan_parent_impl(frame)
    }
}

// =============================================================================
// Win Altframe Selection
// =============================================================================

/// Find the alternate frame that receives space when a window is closed.
///
/// This implements the win_altframe logic:
/// - Prefers next sibling by default
/// - If 'splitbelow' is set for FR_COL, prefer previous sibling
/// - If 'splitright' is set for FR_ROW, prefer previous sibling
/// - If target has wfh/wfw but other doesn't, reverse the selection
///
/// # Safety
/// Caller must ensure `wp` is a valid window handle with a valid frame.
fn win_altframe_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frp = nvim_win_get_frame(wp);
        if frp.is_null() {
            return std::ptr::null_mut();
        }

        // If no previous sibling, must use next
        if (*frp).fr_prev.is_null() {
            return (*frp).fr_next;
        }

        // If no next sibling, must use prev
        if (*frp).fr_next.is_null() {
            return (*frp).fr_prev;
        }

        // By default the next window will get the space
        let mut target_fr = (*frp).fr_next;
        let mut other_fr = (*frp).fr_prev;

        let parent = (*frp).fr_parent;
        if !parent.is_null() {
            let layout = (*parent).fr_layout;

            // If this is part of a column and 'splitbelow' is true,
            // the previous window gets the space
            if layout == FR_COL && nvim_get_p_sb() != 0 {
                target_fr = (*frp).fr_prev;
                other_fr = (*frp).fr_next;
            }

            // If this is part of a row and 'splitright' is true,
            // the previous window gets the space
            if layout == FR_ROW && nvim_get_p_spr() != 0 {
                target_fr = (*frp).fr_prev;
                other_fr = (*frp).fr_next;
            }

            // If 'wfh' or 'wfw' is set for target but not for other, reverse
            if layout == FR_ROW {
                if rs_frame_fixed_width(target_fr) != 0 && rs_frame_fixed_width(other_fr) == 0 {
                    target_fr = other_fr;
                }
            } else if rs_frame_fixed_height(target_fr) != 0 && rs_frame_fixed_height(other_fr) == 0
            {
                target_fr = other_fr;
            }
        }

        target_fr
    }
}

// =============================================================================
// Winframe Find Altwin Helper
// =============================================================================

/// Result structure for winframe_find_altwin.
/// This is used to return multiple values from the Rust implementation.
#[repr(C)]
pub struct WinframeResult {
    /// The alternate frame that will receive the space.
    pub altfr: *mut Frame,
    /// The direction ('v' for vertical, 'h' for horizontal).
    pub dir: c_int,
}

extern "C" {
    /// Get window from frame (recursive).
    fn rs_frame2win(frp: *mut Frame) -> WinHandle;

    /// Get w_p_wfh from window.
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;

    /// Get w_p_wfw from window.
    fn nvim_win_get_wfw(wp: WinHandle) -> c_int;
}

/// Find the best alternate frame considering winfixheight/winfixwidth constraints.
///
/// When the initial altframe has wfh/wfw set, search outward from the closing
/// window to find a frame that can accept the space.
///
/// # Arguments
/// * `frp_close` - Frame of the window being closed
/// * `altfr` - Initial alternate frame from win_altframe
///
/// # Returns
/// The best frame to receive the space (may be same as altfr)
fn find_best_altframe_for_col(frp_close: *const Frame, altfr: *mut Frame) -> *mut Frame {
    if frp_close.is_null() || altfr.is_null() {
        return altfr;
    }

    unsafe {
        // Check if altfr has a leaf window with wfh set
        let alt_win = (*altfr).fr_win;
        if alt_win.is_null() || nvim_win_get_wfh(alt_win) == 0 {
            return altfr; // No wfh, use as-is
        }

        // Search outward from frp_close for a frame without fixed height
        let mut frp_prev = (*frp_close).fr_prev;
        let mut frp_next = (*frp_close).fr_next;

        while !frp_prev.is_null() || !frp_next.is_null() {
            if !frp_prev.is_null() {
                if rs_frame_fixed_height(frp_prev) == 0 {
                    return frp_prev;
                }
                frp_prev = (*frp_prev).fr_prev;
            }
            if !frp_next.is_null() {
                let frp_next_win = (*frp_next).fr_win;
                if !frp_next_win.is_null() && nvim_win_get_wfh(frp_next_win) == 0 {
                    return frp_next;
                }
                frp_next = (*frp_next).fr_next;
            }
        }

        altfr
    }
}

/// Find the best alternate frame for horizontal layout (row).
fn find_best_altframe_for_row(frp_close: *const Frame, altfr: *mut Frame) -> *mut Frame {
    if frp_close.is_null() || altfr.is_null() {
        return altfr;
    }

    unsafe {
        // Check if altfr has a leaf window with wfw set
        let alt_win = (*altfr).fr_win;
        if alt_win.is_null() || nvim_win_get_wfw(alt_win) == 0 {
            return altfr; // No wfw, use as-is
        }

        // Search outward from frp_close for a frame without fixed width
        let mut frp_prev = (*frp_close).fr_prev;
        let mut frp_next = (*frp_close).fr_next;

        while !frp_prev.is_null() || !frp_next.is_null() {
            if !frp_prev.is_null() {
                if rs_frame_fixed_width(frp_prev) == 0 {
                    return frp_prev;
                }
                frp_prev = (*frp_prev).fr_prev;
            }
            if !frp_next.is_null() {
                let frp_next_win = (*frp_next).fr_win;
                if !frp_next_win.is_null() && nvim_win_get_wfw(frp_next_win) == 0 {
                    return frp_next;
                }
                frp_next = (*frp_next).fr_next;
            }
        }

        altfr
    }
}

/// Core implementation of winframe_find_altwin.
///
/// Finds the frame and direction for space redistribution when closing a window.
///
/// # Arguments
/// * `wp` - Window being closed
/// * `altfr_initial` - Initial alternate frame from win_altframe
///
/// # Returns
/// WinframeResult with the best altframe and direction
fn winframe_find_altwin_impl(wp: WinHandle, altfr_initial: *mut Frame) -> WinframeResult {
    let null_result = WinframeResult {
        altfr: std::ptr::null_mut(),
        dir: 0,
    };

    if wp.is_null() || altfr_initial.is_null() {
        return null_result;
    }

    unsafe {
        let frp_close = nvim_win_get_frame(wp);
        if frp_close.is_null() {
            return null_result;
        }

        let parent = (*frp_close).fr_parent;
        if parent.is_null() {
            return WinframeResult {
                altfr: altfr_initial,
                dir: c_int::from(b'h'),
            };
        }

        let layout = (*parent).fr_layout;

        if layout == FR_COL {
            let best_altfr = find_best_altframe_for_col(frp_close, altfr_initial);
            WinframeResult {
                altfr: best_altfr,
                dir: c_int::from(b'v'),
            }
        } else {
            let best_altfr = find_best_altframe_for_row(frp_close, altfr_initial);
            WinframeResult {
                altfr: best_altfr,
                dir: c_int::from(b'h'),
            }
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Find the alternate frame that receives the freed space.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_find_altframe(wp: WinHandle) -> *mut Frame {
    find_altframe_impl(wp)
}

/// FFI: Find the alternate frame using splitbelow/splitright and wfh/wfw logic.
///
/// This is the more sophisticated altframe selection that considers options.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_altframe(wp: WinHandle) -> *mut Frame {
    win_altframe_impl(wp)
}

/// FFI: Find the best altframe and direction considering wfh/wfw constraints.
///
/// # Arguments
/// * `wp` - Window being closed
/// * `altfr_initial` - Initial alternate frame from win_altframe
///
/// # Returns
/// WinframeResult with the best altframe and direction
#[unsafe(no_mangle)]
pub extern "C" fn rs_winframe_find_altwin(
    wp: WinHandle,
    altfr_initial: *mut Frame,
) -> WinframeResult {
    winframe_find_altwin_impl(wp, altfr_initial)
}

/// FFI: Check if frame should be flattened after removal.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_should_flatten(frp: *const Frame) -> c_int {
    c_int::from(frame_should_flatten_impl(frp))
}

/// FFI: Get expansion direction for alt frame ('v' or 'h').
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_expansion_direction(frp: *const Frame) -> c_int {
    get_expansion_direction_impl(frp)
}

/// FFI: Check if removal would orphan parent (leave single child).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_would_orphan_parent(frp: *const Frame) -> c_int {
    c_int::from(removal_would_orphan_parent_impl(frp))
}

/// FFI: Get frame parent.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_get_parent(frp: *const Frame) -> *mut Frame {
    get_frame_parent_impl(frp)
}

/// FFI: Check if frame is topframe.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_is_topframe(frp: *const Frame) -> c_int {
    c_int::from(is_topframe_impl(frp))
}

/// FFI: Get total freed height.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_total_freed_height(frp: *const Frame) -> c_int {
    total_freed_height_impl(frp)
}

/// FFI: Get total freed width.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_total_freed_width(frp: *const Frame) -> c_int {
    total_freed_width_impl(frp)
}

/// FFI: Calculate height gain for alt frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_height_gain(frp: *const Frame) -> c_int {
    height_gain_for_alt_impl(frp)
}

/// FFI: Calculate width gain for alt frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_width_gain(frp: *const Frame) -> c_int {
    width_gain_for_alt_impl(frp)
}

/// FFI: Check if frame can be safely removed.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_can_remove_frame(frp: *const Frame) -> c_int {
    c_int::from(can_remove_frame_impl(frp))
}

/// FFI: Check if close needs layout adjustment.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_needs_layout_adjustment(wp: WinHandle) -> c_int {
    c_int::from(close_needs_layout_adjustment_impl(wp))
}

/// FFI: Get window's frame for close operations.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_frame(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { nvim_win_get_frame(wp) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_frame_safety() {
        assert!(find_altframe_impl(WinHandle::null()).is_null());
        assert!(!frame_should_flatten_impl(std::ptr::null()));
        assert_eq!(
            get_expansion_direction_impl(std::ptr::null()),
            c_int::from(b'h')
        );
        assert!(!removal_would_orphan_parent_impl(std::ptr::null()));
        assert!(get_frame_parent_impl(std::ptr::null()).is_null());
        assert!(!is_topframe_impl(std::ptr::null()));
        assert_eq!(total_freed_height_impl(std::ptr::null()), 0);
        assert_eq!(total_freed_width_impl(std::ptr::null()), 0);
        assert_eq!(height_gain_for_alt_impl(std::ptr::null()), 0);
        assert_eq!(width_gain_for_alt_impl(std::ptr::null()), 0);
        assert!(!can_remove_frame_impl(std::ptr::null()));
    }

    #[test]
    fn test_null_window_safety() {
        assert!(!close_needs_layout_adjustment_impl(WinHandle::null()));
    }
}
