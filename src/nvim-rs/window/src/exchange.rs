//! Window exchange and rotation functions.
//!
//! This module provides Rust helper functions for window exchange and rotation
//! operations from `src/nvim/window.c`, including win_exchange, win_rotate,
//! and win_move_after.

// Window dimensions may need truncation when converting between types.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_LEAF};

// =============================================================================
// External C Functions
// =============================================================================

use crate::BufHandle;

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get firstwin.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get w_next from window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get w_prev from window.
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get w_status_height from window.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    /// Set w_status_height on window.
    fn nvim_win_set_status_height(wp: WinHandle, val: c_int);

    /// Get w_vsep_width from window.
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;

    /// Set w_vsep_width on window.
    fn nvim_win_set_vsep_width(wp: WinHandle, val: c_int);

    /// Get w_hsep_height from window.
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;

    /// Set w_hsep_height on window.
    fn nvim_win_set_hsep_height(wp: WinHandle, val: c_int);

    /// Check if only one non-floating window in the tab.
    fn rs_one_window_in_tab(wp: WinHandle, tp: crate::TabpageHandle) -> c_int;

    /// Check if text or buffer is locked.
    fn nvim_text_or_buf_locked() -> c_int;

    /// Emit the e_floatexchange error message.
    fn nvim_emsg_e_floatexchange();

    /// Call beep_flush().
    fn nvim_beep_flush_wrapper();

    /// Remove a window from the window list.
    fn rs_win_remove(wp: WinHandle, tp: crate::TabpageHandle);

    /// Remove a frame from the frame tree.
    fn rs_frame_remove(frp: *mut Frame);

    /// Append a window after another in the window list.
    fn rs_win_append(after: WinHandle, wp: WinHandle, tp: crate::TabpageHandle);

    /// Insert a frame before another.
    fn rs_frame_insert(before: *mut Frame, frp: *mut Frame);

    /// Append a frame after another.
    fn rs_frame_append(after: *mut Frame, frp: *mut Frame);

    /// Fix height-related frame sizes for a window's frame.
    fn rs_frame_fix_height(wp: WinHandle);

    /// Fix width-related frame sizes for a window's frame.
    fn rs_frame_fix_width(wp: WinHandle);

    /// Recompute window positions.
    fn rs_win_comp_pos() -> c_int;

    /// Get w_buffer from window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get curbuf.
    fn nvim_get_curbuf() -> BufHandle;

    /// Get VIsual_active global.
    fn nvim_get_VIsual_active() -> c_int;

    /// Reset VIsual mode and resel.
    fn rs_reset_VIsual_and_resel();

    /// Copy cursor position from src window to dst window.
    fn nvim_win_copy_cursor(dst: WinHandle, src: WinHandle);

    /// Enter a window (triggers autocommands).
    fn nvim_win_enter(wp: WinHandle, undo_sync: c_int);

    /// Schedule redraw for a window.
    fn nvim_redraw_later_wrapper(wp: WinHandle, r#type: c_int);
}

// =============================================================================
// Exchange Validation
// =============================================================================

/// Check if a window can be exchanged.
///
/// Returns 0 if can exchange, error code otherwise:
/// - 1: Floating window
/// - 2: Only one window
/// - 3: Text/buffer locked (checked in C)
fn can_exchange_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 1;
    }

    unsafe {
        // Cannot exchange floating windows
        if nvim_win_get_floating(wp) != 0 {
            return 1;
        }

        // Check if only one non-floating window (simplified check)
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return 2;
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return 2;
        }

        // Check if there are siblings
        let has_siblings = !(*frame).fr_prev.is_null() || !(*frame).fr_next.is_null();
        if !has_siblings {
            return 2;
        }

        0 // Can exchange
    }
}

/// Find the target frame for exchange based on Prenum.
///
/// If Prenum > 0: find nth frame in parent
/// If Prenum == 0: find next frame, or prev if at end
fn find_exchange_target_impl(wp: WinHandle, prenum: c_int) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return std::ptr::null_mut();
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return std::ptr::null_mut();
        }

        if prenum > 0 {
            // Find nth frame
            let mut frp = (*parent).fr_child;
            let mut count = prenum;
            while !frp.is_null() && count > 1 {
                frp = (*frp).fr_next;
                count -= 1;
            }
            frp
        } else if !(*frame).fr_next.is_null() {
            // Swap with next
            (*frame).fr_next
        } else {
            // Swap with prev (last in row/col)
            (*frame).fr_prev
        }
    }
}

/// Check if target frame is valid for exchange.
///
/// Must be a leaf frame (containing a single window) and not the same window.
fn validate_exchange_target_impl(wp: WinHandle, target_frp: *const Frame) -> bool {
    if wp.is_null() || target_frp.is_null() {
        return false;
    }

    unsafe {
        // Must be a leaf frame
        if (*target_frp).fr_layout != FR_LEAF {
            return false;
        }

        // Must have a window
        if (*target_frp).fr_win.is_null() {
            return false;
        }

        // Must not be the same window
        (*target_frp).fr_win != wp
    }
}

// =============================================================================
// Rotation Validation
// =============================================================================

/// Check if windows can be rotated.
///
/// Returns 0 if can rotate, error code otherwise:
/// - 1: Floating window
/// - 2: Only one window
/// - 3: A frame in the group contains nested windows (not leaf)
fn can_rotate_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 1;
    }

    unsafe {
        // Cannot rotate floating windows
        if nvim_win_get_floating(wp) != 0 {
            return 1;
        }

        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return 2;
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return 2;
        }

        // Check all frames in parent are leaf frames
        let mut frp = (*parent).fr_child;
        while !frp.is_null() {
            if (*frp).fr_layout != FR_LEAF || (*frp).fr_win.is_null() {
                return 3;
            }
            frp = (*frp).fr_next;
        }

        // Check there are multiple frames
        let first = (*parent).fr_child;
        if first.is_null() || (*first).fr_next.is_null() {
            return 2;
        }

        0 // Can rotate
    }
}

/// Count frames in rotation group.
fn count_rotation_frames_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return 0;
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return 0;
        }

        let mut count = 0;
        let mut frp = (*parent).fr_child;
        while !frp.is_null() {
            count += 1;
            frp = (*frp).fr_next;
        }
        count
    }
}

/// Get the first frame in the rotation group.
fn get_first_rotation_frame_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return std::ptr::null_mut();
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return std::ptr::null_mut();
        }

        (*parent).fr_child
    }
}

/// Get the last frame in the rotation group.
fn get_last_rotation_frame_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return std::ptr::null_mut();
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return std::ptr::null_mut();
        }

        let mut frp = (*parent).fr_child;
        if frp.is_null() {
            return std::ptr::null_mut();
        }

        while !(*frp).fr_next.is_null() {
            frp = (*frp).fr_next;
        }
        frp
    }
}

// =============================================================================
// Exchange State Helpers
// =============================================================================

/// Get separator properties for exchange.
///
/// Returns (status_height, vsep_width, hsep_height) for a window.
fn get_separator_props_impl(wp: WinHandle) -> (c_int, c_int, c_int) {
    if wp.is_null() {
        return (0, 0, 0);
    }

    unsafe {
        (
            nvim_win_get_status_height(wp),
            nvim_win_get_vsep_width(wp),
            nvim_win_get_hsep_height(wp),
        )
    }
}

/// Check if two windows are adjacent.
fn windows_are_adjacent_impl(wp1: WinHandle, wp2: WinHandle) -> bool {
    if wp1.is_null() || wp2.is_null() {
        return false;
    }

    unsafe { nvim_win_get_prev(wp1) == wp2 || nvim_win_get_next(wp1) == wp2 }
}

/// Get the window that will become the new last in rotation.
///
/// For both upwards and downwards rotation, the second-to-last window
/// becomes the new last window (since either the first or last moves).
fn get_new_last_window_impl(first_frp: *const Frame, _upwards: bool) -> WinHandle {
    if first_frp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        // Find second-to-last frame in the list
        let mut frp = first_frp.cast_mut();
        let mut second_to_last: *mut Frame = std::ptr::null_mut();

        while !frp.is_null() && !(*frp).fr_next.is_null() {
            second_to_last = frp;
            frp = (*frp).fr_next;
        }

        if !second_to_last.is_null() {
            return (*second_to_last).fr_win;
        }
        WinHandle::null()
    }
}

// =============================================================================
// Full win_exchange implementation
// =============================================================================

/// Rust implementation of `win_exchange()`.
///
/// Exchange the current window with another window (determined by `prenum`).
/// When `prenum` is 0, exchange with the next window (or previous if at end).
/// When `prenum` > 0, exchange with the nth window in the parent frame.
///
/// # Safety
/// Requires valid global Neovim state (curwin, frames, window list).
unsafe fn win_exchange_impl(prenum: c_int) {
    let curwin = nvim_get_curwin();
    if curwin.is_null() {
        return;
    }

    // Cannot exchange floating windows.
    if nvim_win_get_floating(curwin) != 0 {
        nvim_emsg_e_floatexchange();
        return;
    }

    // Only one non-floating window in the tab — nothing to exchange.
    if rs_one_window_in_tab(curwin, crate::TabpageHandle::null()) != 0 {
        nvim_beep_flush_wrapper();
        return;
    }

    // Do not exchange if text or buffer is locked.
    if nvim_text_or_buf_locked() != 0 {
        nvim_beep_flush_wrapper();
        return;
    }

    // Find the target frame.
    let curframe = nvim_win_get_frame(curwin);
    if curframe.is_null() {
        return;
    }
    let parent = (*curframe).fr_parent;
    if parent.is_null() {
        return;
    }

    let frp: *mut Frame = if prenum > 0 {
        // Find the nth frame in the parent.
        let mut f = (*parent).fr_child;
        let mut n = prenum;
        while !f.is_null() && n > 1 {
            f = (*f).fr_next;
            n -= 1;
        }
        f
    } else if !(*curframe).fr_next.is_null() {
        // Swap with next frame.
        (*curframe).fr_next
    } else {
        // Swap last frame in row/col with previous.
        (*curframe).fr_prev
    };

    // We can only exchange with another leaf frame that has a different window.
    if frp.is_null() || (*frp).fr_win.is_null() || (*frp).fr_win == curwin {
        return;
    }
    let wp = (*frp).fr_win;

    // Step 1: remove curwin from the window list; remember where it was (wp2).
    // Step 2: insert curwin before wp in the window list.
    // Step 3 (if needed): remove wp and re-insert it after wp2.
    // Step 4: swap separator properties.
    let wp2 = nvim_win_get_prev(curwin);
    let frp2 = (*curframe).fr_prev;

    if nvim_win_get_prev(wp) != curwin {
        rs_win_remove(curwin, crate::TabpageHandle::null());
        rs_frame_remove(curframe);
        rs_win_append(nvim_win_get_prev(wp), curwin, crate::TabpageHandle::null());
        rs_frame_insert(frp, curframe);
    }

    if wp != wp2 {
        rs_win_remove(wp, crate::TabpageHandle::null());
        rs_frame_remove(nvim_win_get_frame(wp));
        rs_win_append(wp2, wp, crate::TabpageHandle::null());
        if frp2.is_null() {
            // Insert wp's frame as the first child of parent.
            let first_child = (*parent).fr_child;
            rs_frame_insert(first_child, nvim_win_get_frame(wp));
        } else {
            rs_frame_append(frp2, nvim_win_get_frame(wp));
        }
    }

    // Swap status_height, vsep_width, hsep_height between curwin and wp.
    let temp_sh = nvim_win_get_status_height(curwin);
    nvim_win_set_status_height(curwin, nvim_win_get_status_height(wp));
    nvim_win_set_status_height(wp, temp_sh);

    let temp_vw = nvim_win_get_vsep_width(curwin);
    nvim_win_set_vsep_width(curwin, nvim_win_get_vsep_width(wp));
    nvim_win_set_vsep_width(wp, temp_vw);

    let old_hsep = nvim_win_get_hsep_height(curwin);
    nvim_win_set_hsep_height(curwin, nvim_win_get_hsep_height(wp));
    nvim_win_set_hsep_height(wp, old_hsep);

    // Fix frame geometry and recompute positions.
    rs_frame_fix_height(curwin);
    rs_frame_fix_height(wp);
    rs_frame_fix_width(curwin);
    rs_frame_fix_width(wp);
    rs_win_comp_pos();

    // Handle VIsual selection.
    if nvim_win_get_buffer(wp) != nvim_get_curbuf() {
        rs_reset_VIsual_and_resel();
    } else if nvim_get_VIsual_active() != 0 {
        nvim_win_copy_cursor(wp, curwin);
    }

    // Enter the target window and mark both for redraw.
    // UPD_NOT_VALID = 40 (verified by _Static_assert in window_shim.c).
    nvim_win_enter(wp, 1);
    nvim_redraw_later_wrapper(curwin, 40);
    nvim_redraw_later_wrapper(wp, 40);
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Exchange the current window with another.
///
/// Direct Rust replacement for the C `win_exchange()` function.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_exchange(prenum: c_int) {
    unsafe { win_exchange_impl(prenum) }
}

/// FFI: Check if window can be exchanged.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_can_exchange(wp: WinHandle) -> c_int {
    can_exchange_impl(wp)
}

/// FFI: Find target frame for exchange.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_find_target(wp: WinHandle, prenum: c_int) -> *mut Frame {
    find_exchange_target_impl(wp, prenum)
}

/// FFI: Validate exchange target.
///
/// # Safety
/// Caller must ensure `target_frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_exchange_validate_target(
    wp: WinHandle,
    target_frp: *const Frame,
) -> c_int {
    c_int::from(validate_exchange_target_impl(wp, target_frp))
}

/// FFI: Check if windows can be rotated.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_can_rotate(wp: WinHandle) -> c_int {
    can_rotate_impl(wp)
}

/// FFI: Count frames in rotation group.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_count_rotation_frames(wp: WinHandle) -> c_int {
    count_rotation_frames_impl(wp)
}

/// FFI: Get first frame in rotation group.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_first_rotation_frame(wp: WinHandle) -> *mut Frame {
    get_first_rotation_frame_impl(wp)
}

/// FFI: Get last frame in rotation group.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_last_rotation_frame(wp: WinHandle) -> *mut Frame {
    get_last_rotation_frame_impl(wp)
}

/// FFI: Get window status height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_get_status_height(wp: WinHandle) -> c_int {
    get_separator_props_impl(wp).0
}

/// FFI: Get window vsep width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_get_vsep_width(wp: WinHandle) -> c_int {
    get_separator_props_impl(wp).1
}

/// FFI: Get window hsep height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_get_hsep_height(wp: WinHandle) -> c_int {
    get_separator_props_impl(wp).2
}

/// FFI: Check if windows are adjacent.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exchange_are_adjacent(wp1: WinHandle, wp2: WinHandle) -> c_int {
    c_int::from(windows_are_adjacent_impl(wp1, wp2))
}

/// FFI: Get new last window after rotation.
///
/// # Safety
/// Caller must ensure `first_frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_exchange_new_last_window(
    first_frp: *const Frame,
    upwards: c_int,
) -> WinHandle {
    get_new_last_window_impl(first_frp, upwards != 0)
}

/// FFI: Get window from frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_exchange_frame_window(frp: *const Frame) -> WinHandle {
    if frp.is_null() {
        return WinHandle::null();
    }
    (*frp).fr_win
}

/// FFI: Check if frame is a leaf.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_exchange_frame_is_leaf(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from((*frp).fr_layout == FR_LEAF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_window_exchange() {
        let null_wp = WinHandle::null();
        assert_eq!(can_exchange_impl(null_wp), 1);
        assert!(find_exchange_target_impl(null_wp, 0).is_null());
    }

    #[test]
    fn test_null_window_rotate() {
        let null_wp = WinHandle::null();
        assert_eq!(can_rotate_impl(null_wp), 1);
        assert_eq!(count_rotation_frames_impl(null_wp), 0);
        assert!(get_first_rotation_frame_impl(null_wp).is_null());
        assert!(get_last_rotation_frame_impl(null_wp).is_null());
    }

    #[test]
    fn test_null_separator_props() {
        let null_wp = WinHandle::null();
        assert_eq!(get_separator_props_impl(null_wp), (0, 0, 0));
    }

    #[test]
    fn test_null_adjacent() {
        let null_wp = WinHandle::null();
        assert!(!windows_are_adjacent_impl(null_wp, null_wp));
    }
}
