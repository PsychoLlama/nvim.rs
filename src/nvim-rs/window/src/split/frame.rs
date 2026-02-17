//! Frame tree helpers for window splitting.
//!
//! This module provides helper functions for frame tree manipulation
//! during window split operations, including frame reorganization,
//! position calculation, and size fixing.

// Frame pointers and window dimensions may need truncation when converting
// between types, but values are guaranteed to be in valid range.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::similar_names)]

use std::ffi::c_int;

use crate::frame::constants::{STATUS_HEIGHT, WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_TOP, WSP_VERT};
use crate::{Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get w_frame from a window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get w_winrow from a window.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Get w_wincol from a window.
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;

    /// Get w_width from a window.
    fn nvim_win_field_width(wp: WinHandle) -> c_int;

    /// Get w_height from a window.
    fn nvim_win_field_height(wp: WinHandle) -> c_int;

    /// Get w_vsep_width from a window.
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;

    /// Get w_status_height from a window.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    /// Get w_hsep_height from a window.
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;

    /// Get tabline_height().
    fn tabline_height() -> c_int;

    /// Get global statusline height.
    fn global_stl_height() -> c_int;

    /// Get p_ls (laststatus option).
    fn nvim_get_p_ls() -> i64;

    /// Get p_sb (splitbelow option).
    fn nvim_get_p_sb() -> c_int;

    /// Get p_spr (splitright option).
    fn nvim_get_p_spr() -> c_int;
}

// =============================================================================
// Frame Tree Reorganization
// =============================================================================

/// Check if frame tree needs reorganization for the split.
///
/// Returns true if a new branch needs to be created in the frame tree
/// because the current frame's parent layout doesn't match the split direction.
fn needs_frame_branch_impl(curfrp: *const Frame, layout: i8) -> bool {
    if curfrp.is_null() {
        return true;
    }

    unsafe {
        let parent = (*curfrp).fr_parent;
        parent.is_null() || (*parent).fr_layout != layout
    }
}

/// Get the frame to use as the split point for toplevel splits.
///
/// For toplevel splits, this finds the appropriate frame in the tree.
fn get_toplevel_split_frame_impl(flags: c_int) -> *mut Frame {
    unsafe {
        let topframe = nvim_get_topframe();
        if topframe.is_null() {
            return std::ptr::null_mut();
        }

        let vertical = (flags & WSP_VERT) != 0;
        let top_layout = (*topframe).fr_layout;

        // Check if topframe layout matches split direction
        if (top_layout == FR_COL && !vertical) || (top_layout == FR_ROW && vertical) {
            // Use child frame
            let mut curfrp = (*topframe).fr_child;

            // For WSP_BOT, go to last child
            if (flags & WSP_BOT) != 0 {
                while !curfrp.is_null() && !(*curfrp).fr_next.is_null() {
                    curfrp = (*curfrp).fr_next;
                }
            }
            curfrp
        } else {
            // Use topframe itself
            topframe
        }
    }
}

/// Determine if new window should be placed before current frame.
///
/// Based on split flags and options (splitbelow, splitright).
fn should_place_before_impl(flags: c_int, is_toplevel: bool) -> bool {
    if is_toplevel {
        return (flags & WSP_TOP) != 0;
    }

    unsafe {
        let vertical = (flags & WSP_VERT) != 0;

        if (flags & WSP_BELOW) != 0 {
            return false;
        }
        if (flags & WSP_ABOVE) != 0 {
            return true;
        }

        // Use option defaults
        if vertical {
            nvim_get_p_spr() == 0
        } else {
            nvim_get_p_sb() == 0
        }
    }
}

// =============================================================================
// Position Calculations
// =============================================================================

/// Calculate the winrow for a new horizontal split window.
fn calc_new_winrow_impl(oldwin: WinHandle, _new_height: c_int, before: bool) -> c_int {
    if oldwin.is_null() {
        return unsafe { tabline_height() };
    }

    unsafe {
        let old_winrow = nvim_win_get_winrow(oldwin);

        if before {
            // New window above - takes old window's position
            old_winrow
        } else {
            // New window below - after old window + status
            let old_height = nvim_win_field_height(oldwin);
            let old_status = nvim_win_get_status_height(oldwin);
            old_winrow + old_height + old_status
        }
    }
}

/// Calculate the wincol for a new vertical split window.
fn calc_new_wincol_impl(oldwin: WinHandle, _new_width: c_int, before: bool) -> c_int {
    if oldwin.is_null() {
        return 0;
    }

    unsafe {
        let old_wincol = nvim_win_get_wincol(oldwin);

        if before {
            // New window left - takes old window's position
            old_wincol
        } else {
            // New window right - after old window + separator
            let old_width = nvim_win_field_width(oldwin);
            old_wincol + old_width + 1
        }
    }
}

/// Calculate new position for old window after split.
fn calc_old_win_new_pos_impl(
    oldwin: WinHandle,
    new_size: c_int,
    vertical: bool,
    before: bool,
) -> c_int {
    if oldwin.is_null() {
        return 0;
    }

    unsafe {
        if vertical {
            let old_wincol = nvim_win_get_wincol(oldwin);
            if before {
                // New window to left, old window moves right
                old_wincol + new_size + 1
            } else {
                // New window to right, old window stays
                old_wincol
            }
        } else {
            let old_winrow = nvim_win_get_winrow(oldwin);
            if before {
                // New window above, old window moves down
                old_winrow + new_size + STATUS_HEIGHT
            } else {
                // New window below, old window stays
                old_winrow
            }
        }
    }
}

// =============================================================================
// Frame Size Fixing
// =============================================================================

/// Calculate frame height from window dimensions.
fn calc_frame_height_from_win_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        nvim_win_field_height(wp) + nvim_win_get_status_height(wp) + nvim_win_get_hsep_height(wp)
    }
}

/// Calculate frame width from window dimensions.
fn calc_frame_width_from_win_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe { nvim_win_field_width(wp) + nvim_win_get_vsep_width(wp) }
}

/// Get appropriate status height for new window.
fn get_new_win_status_height_impl(flags: c_int, oldwin: WinHandle, before: bool) -> c_int {
    unsafe {
        let p_ls = nvim_get_p_ls() as c_int;

        if (flags & WSP_VERT) != 0 {
            // Vertical split - match height-related properties
            if !oldwin.is_null() {
                return nvim_win_get_status_height(oldwin);
            }
            if p_ls == 1 || p_ls == 2 {
                STATUS_HEIGHT
            } else {
                0
            }
        } else {
            // Horizontal split
            let global_stl = global_stl_height() > 0;
            if global_stl {
                return 0;
            }

            if before {
                // New window above - always has status
                STATUS_HEIGHT
            } else if !oldwin.is_null() {
                // New window below - inherit from old
                nvim_win_get_status_height(oldwin)
            } else {
                STATUS_HEIGHT
            }
        }
    }
}

/// Get appropriate hsep height for new window.
fn get_new_win_hsep_height_impl(flags: c_int, oldwin: WinHandle, before: bool) -> c_int {
    unsafe {
        let global_stl = global_stl_height() > 0;

        if !global_stl {
            return 0;
        }

        if (flags & WSP_VERT) != 0 {
            // Vertical split - match old window
            if !oldwin.is_null() {
                return nvim_win_get_hsep_height(oldwin);
            }
            0
        } else if before {
            // New window above - gets hsep
            1
        } else if !oldwin.is_null() {
            // New window below - inherit from old
            nvim_win_get_hsep_height(oldwin)
        } else {
            0
        }
    }
}

/// Get appropriate vsep width for new window.
fn get_new_win_vsep_width_impl(_flags: c_int, oldwin: WinHandle, before: bool) -> c_int {
    if before {
        // New window to left - gets separator
        return 1;
    }

    if !oldwin.is_null() {
        // New window to right - inherit from old
        return unsafe { nvim_win_get_vsep_width(oldwin) };
    }

    0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if frame tree needs reorganization.
///
/// # Safety
/// Caller must ensure `curfrp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_split_needs_frame_branch(curfrp: *const Frame, layout: c_int) -> c_int {
    c_int::from(needs_frame_branch_impl(curfrp, layout as i8))
}

/// FFI: Get frame for toplevel split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_get_toplevel_frame(flags: c_int) -> *mut Frame {
    get_toplevel_split_frame_impl(flags)
}

/// FFI: Check if new window should be placed before current.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_place_before(flags: c_int, is_toplevel: c_int) -> c_int {
    c_int::from(should_place_before_impl(flags, is_toplevel != 0))
}

/// FFI: Calculate winrow for new horizontal split window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_calc_winrow(
    oldwin: WinHandle,
    new_height: c_int,
    before: c_int,
) -> c_int {
    calc_new_winrow_impl(oldwin, new_height, before != 0)
}

/// FFI: Calculate wincol for new vertical split window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_calc_wincol(
    oldwin: WinHandle,
    new_width: c_int,
    before: c_int,
) -> c_int {
    calc_new_wincol_impl(oldwin, new_width, before != 0)
}

/// FFI: Calculate new position for old window after split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_calc_old_pos(
    oldwin: WinHandle,
    new_size: c_int,
    vertical: c_int,
    before: c_int,
) -> c_int {
    calc_old_win_new_pos_impl(oldwin, new_size, vertical != 0, before != 0)
}

/// FFI: Calculate frame height from window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_frame_height_from_win(wp: WinHandle) -> c_int {
    calc_frame_height_from_win_impl(wp)
}

/// FFI: Calculate frame width from window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_frame_width_from_win(wp: WinHandle) -> c_int {
    calc_frame_width_from_win_impl(wp)
}

/// FFI: Get status height for new window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_new_status_height(
    flags: c_int,
    oldwin: WinHandle,
    before: c_int,
) -> c_int {
    get_new_win_status_height_impl(flags, oldwin, before != 0)
}

/// FFI: Get hsep height for new window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_new_hsep_height(
    flags: c_int,
    oldwin: WinHandle,
    before: c_int,
) -> c_int {
    get_new_win_hsep_height_impl(flags, oldwin, before != 0)
}

/// FFI: Get vsep width for new window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_new_vsep_width(flags: c_int, oldwin: WinHandle, before: c_int) -> c_int {
    get_new_win_vsep_width_impl(flags, oldwin, before != 0)
}

/// FFI: Check if frame is a leaf.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_split_frame_is_leaf(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from((*frp).fr_layout == FR_LEAF)
}

/// FFI: Get parent layout of a frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_split_parent_layout(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return -1;
    }
    let parent = (*frp).fr_parent;
    if parent.is_null() {
        return -1;
    }
    c_int::from((*parent).fr_layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_frame_branch() {
        assert!(needs_frame_branch_impl(std::ptr::null(), FR_ROW));
        assert!(needs_frame_branch_impl(std::ptr::null(), FR_COL));
    }

    #[test]
    fn test_null_calculations() {
        let null_win = WinHandle::null();
        assert_eq!(calc_frame_height_from_win_impl(null_win), 0);
        assert_eq!(calc_frame_width_from_win_impl(null_win), 0);
    }
}
