//! Status line management functions.
//!
//! This module provides Rust implementations of status line management
//! functions from `src/nvim/window.c`: last_status, last_status_rec,
//! win_remove_status_line, find_horizontally_resizable_frame,
//! resize_frame_for_status, resize_frame_for_winbar.

use std::ffi::c_int;

use crate::{Frame, TabpageHandle, WinHandle, FR_COL, FR_LEAF};

// =============================================================================
// Constants
// =============================================================================

/// Height of a status line (1 row).
const STATUS_HEIGHT: c_int = 1;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_topframe() -> *mut Frame;
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;
    fn nvim_win_set_status_height(wp: WinHandle, val: c_int);
    fn nvim_win_set_hsep_height(wp: WinHandle, val: c_int);
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_prev_height(wp: WinHandle) -> c_int;
    fn nvim_win_set_prev_height(wp: WinHandle, val: c_int);
    fn nvim_comp_col();
    fn nvim_win_stl_clear_click_defs(wp: WinHandle);
    fn nvim_win_float_anchor_laststatus();
    fn nvim_emsg_id(id: c_int);
    fn rs_is_bottom_win(wp: WinHandle) -> c_int;
    fn rs_win_new_height(wp: WinHandle, height: c_int);
    #[link_name = "frame_new_height"]
    fn rs_frame_new_height(
        topfrp: *mut Frame,
        height: c_int,
        topfirst: c_int,
        wfh: c_int,
        set_ch: c_int,
    );
    fn rs_frame_fix_height(wp: WinHandle);
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;
    fn rs_win_comp_pos() -> c_int;
    fn rs_last_stl_height(morewin: c_int) -> c_int;
    fn rs_global_stl_height() -> c_int;

    // --- set_winbar_win dependencies ---
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;
    fn nvim_win_set_winbar_height(wp: WinHandle, val: c_int);
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;
    #[link_name = "win_set_inner_size"]
    fn rs_win_set_inner_size(wp: WinHandle, valid_cursor: bool);
    fn nvim_win_clear_winbar_click_defs(wp: WinHandle);
    /// Returns 1 if local w_p_wbr is empty (for floating window check).
    fn nvim_win_get_p_wbr_empty(wp: WinHandle) -> c_int;
    /// Returns 1 if BOTH global p_wbr and local w_p_wbr are empty (for non-floating).
    fn nvim_win_get_p_wbr_both_empty(wp: WinHandle) -> c_int;

    // --- set_winbar dependencies ---
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
}

// =============================================================================
// EMSG IDs
// =============================================================================

const EMSG_NOROOM: c_int = 13;

// =============================================================================
// Implementations
// =============================================================================

/// Look for a horizontally resizable frame, starting with frame `fr`.
/// Returns NULL if there are no resizable frames.
///
/// Equivalent to C `find_horizontally_resizable_frame()` (window.c L7505).
fn find_horizontally_resizable_frame_impl(fr: *mut Frame) -> *mut Frame {
    if fr.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let topframe = nvim_get_topframe();
        let mut fp = fr;

        while (*fp).fr_height <= rs_frame_minheight(fp, WinHandle::null()) {
            if fp == topframe {
                return std::ptr::null_mut();
            }
            let parent = (*fp).fr_parent;
            // In a column of frames: go to frame above. If already at
            // the top or in a row of frames: go to parent.
            if !parent.is_null() && (*parent).fr_layout == FR_COL && !(*fp).fr_prev.is_null() {
                fp = (*fp).fr_prev;
            } else {
                fp = parent;
            }
        }

        fp
    }
}

/// Remove status line from window, replacing it with a horizontal separator if needed.
///
/// Equivalent to C `win_remove_status_line()` (window.c L7487).
fn win_remove_status_line_impl(wp: WinHandle, add_hsep: bool) {
    if wp.is_null() {
        return;
    }

    unsafe {
        nvim_win_set_status_height(wp, 0);
        if add_hsep {
            nvim_win_set_hsep_height(wp, 1);
        } else {
            let height = if nvim_win_get_floating(wp) != 0 {
                nvim_win_get_view_height(wp)
            } else {
                nvim_win_get_w_height(wp)
            };
            rs_win_new_height(wp, height + STATUS_HEIGHT);
        }
        nvim_comp_col();
        nvim_win_stl_clear_click_defs(wp);
    }
}

/// Look for resizable frames and take lines from them to make room for the statusline.
///
/// Equivalent to C `resize_frame_for_status()` (window.c L7527).
///
/// Returns true on success, false on failure.
fn resize_frame_for_status_impl(fr: *mut Frame) -> bool {
    if fr.is_null() {
        return false;
    }

    unsafe {
        let wp = (*fr).fr_win;
        let fp = find_horizontally_resizable_frame_impl(fr);

        if fp.is_null() {
            nvim_emsg_id(EMSG_NOROOM);
            false
        } else if fp != fr {
            rs_frame_new_height(fp, (*fp).fr_height - 1, 0, 0, 0);
            rs_frame_fix_height(wp);
            rs_win_comp_pos();
            true
        } else {
            rs_win_new_height(wp, nvim_win_get_w_height(wp) - 1);
            true
        }
    }
}

/// Look for resizable frames and take lines from them to make room for the winbar.
///
/// Equivalent to C `resize_frame_for_winbar()` (window.c L7548).
///
/// Returns true on success, false on failure.
fn resize_frame_for_winbar_impl(fr: *mut Frame) -> bool {
    if fr.is_null() {
        return false;
    }

    unsafe {
        let wp = (*fr).fr_win;
        let fp = find_horizontally_resizable_frame_impl(fr);

        if fp.is_null() || fp == fr {
            nvim_emsg_id(EMSG_NOROOM);
            return false;
        }
        rs_frame_new_height(fp, (*fp).fr_height - 1, 0, 0, 0);
        rs_win_new_height(wp, nvim_win_get_w_height(wp) + 1);
        rs_frame_fix_height(wp);
        rs_win_comp_pos();
        true
    }
}

/// Recursive implementation for last_status.
///
/// Equivalent to C `last_status_rec()` (window.c L7565).
fn last_status_rec_impl(fr: *mut Frame, statusline: bool, is_stl_global: bool) {
    if fr.is_null() {
        return;
    }

    unsafe {
        if (*fr).fr_layout == FR_LEAF {
            let wp = (*fr).fr_win;
            let is_last = rs_is_bottom_win(wp) != 0;

            if is_last {
                if nvim_win_get_status_height(wp) != 0 && (!statusline || is_stl_global) {
                    win_remove_status_line_impl(wp, false);
                } else if nvim_win_get_status_height(wp) == 0 && !is_stl_global && statusline {
                    // Add statusline to window if needed.
                    nvim_win_set_status_height(wp, STATUS_HEIGHT);
                    if !resize_frame_for_status_impl(fr) {
                        return;
                    }
                    nvim_comp_col();
                }
                // Set prev_height when difference is due to 'laststatus'.
                let h = nvim_win_get_w_height(wp);
                let prev_h = nvim_win_get_prev_height(wp);
                if (h - prev_h).abs() == 1 {
                    nvim_win_set_prev_height(wp, h);
                }
            } else if nvim_win_get_status_height(wp) != 0 && is_stl_global {
                // If statusline is global and the window has a statusline,
                // replace it with a horizontal separator.
                win_remove_status_line_impl(wp, true);
            } else if nvim_win_get_status_height(wp) == 0 && !is_stl_global {
                // If statusline isn't global and the window doesn't have a
                // statusline, re-add it.
                nvim_win_set_status_height(wp, STATUS_HEIGHT);
                nvim_win_set_hsep_height(wp, 0);
                nvim_comp_col();
            }
        } else {
            // For a column or row frame, recursively call for all child frames.
            let mut fp = (*fr).fr_child;
            while !fp.is_null() {
                last_status_rec_impl(fp, statusline, is_stl_global);
                fp = (*fp).fr_next;
            }
        }
    }
}

/// Add or remove a status line from window(s), according to the
/// value of 'laststatus'.
///
/// Equivalent to C `last_status()` (window.c L7479).
fn last_status_impl(morewin: bool) {
    unsafe {
        let topframe = nvim_get_topframe();
        // Don't make a difference between horizontal or vertical split.
        last_status_rec_impl(
            topframe,
            rs_last_stl_height(c_int::from(morewin)) > 0,
            rs_global_stl_height() > 0,
        );
        nvim_win_float_anchor_laststatus();
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Add or remove a status line from window(s).
#[unsafe(no_mangle)]
pub extern "C" fn rs_last_status(morewin: c_int) {
    last_status_impl(morewin != 0);
}

/// FFI: Remove status line from window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_remove_status_line(wp: WinHandle, add_hsep: c_int) {
    win_remove_status_line_impl(wp, add_hsep != 0);
}

/// FFI: Look for resizable frames to make room for winbar.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_frame_for_winbar(fr: *mut Frame) -> c_int {
    c_int::from(resize_frame_for_winbar_impl(fr))
}

// =============================================================================
// set_winbar_win
// =============================================================================

/// Return codes matching C conventions.
const SET_WINBAR_WIN_OK: c_int = 1;
const SET_WINBAR_WIN_FAIL: c_int = 0;
const SET_WINBAR_WIN_NOTDONE: c_int = 2;

/// Add or remove a winbar from window `wp`.
///
/// Port of C `set_winbar_win()`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
unsafe fn set_winbar_win_impl(wp: WinHandle, make_room: bool, valid_cursor: bool) -> c_int {
    // Compute desired winbar height: 1 if winbar is configured, 0 otherwise.
    // Floating windows only use local 'winbar' (w_p_wbr).
    // Normal windows also check global 'winbar' (p_wbr).
    // Floating: show winbar only if local w_p_wbr is non-empty.
    // Non-floating: show winbar if global p_wbr OR local w_p_wbr is non-empty.
    // nvim_win_get_p_wbr_empty: returns 1 if local wbr is empty.
    // nvim_win_get_p_wbr_both_empty: returns 1 if both global and local wbr are empty.
    let winbar_height = if nvim_win_get_floating(wp) != 0 {
        i32::from(nvim_win_get_p_wbr_empty(wp) == 0)
    } else {
        i32::from(nvim_win_get_p_wbr_both_empty(wp) == 0)
    };

    if nvim_win_get_winbar_height(wp) != winbar_height {
        if winbar_height == 1 && nvim_win_get_view_height(wp) <= 1 {
            if nvim_win_get_floating(wp) != 0 {
                nvim_emsg_id(EMSG_NOROOM);
                return SET_WINBAR_WIN_NOTDONE;
            } else if !make_room || rs_resize_frame_for_winbar(nvim_win_get_frame(wp)) == 0 {
                return SET_WINBAR_WIN_FAIL;
            }
        }
        nvim_win_set_winbar_height(wp, winbar_height);
        rs_win_set_inner_size(wp, valid_cursor);

        if winbar_height == 0 {
            // When removing winbar, deallocate the w_winbar_click_defs array.
            nvim_win_clear_winbar_click_defs(wp);
        }
    }

    SET_WINBAR_WIN_OK
}

/// FFI export for `set_winbar_win`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[allow(clippy::must_use_candidate)]
#[export_name = "set_winbar_win"]
pub unsafe extern "C" fn rs_set_winbar_win(
    wp: WinHandle,
    make_room: bool,
    valid_cursor: bool,
) -> c_int {
    set_winbar_win_impl(wp, make_room, valid_cursor)
}

// =============================================================================
// set_winbar
// =============================================================================

/// Apply winbar setting to all windows in the current tab page.
///
/// Port of C `set_winbar()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn set_winbar_impl(make_room: bool) {
    let curtab = nvim_get_curtab();
    let mut wp = nvim_tabpage_get_firstwin(curtab);
    while !wp.is_null() {
        if set_winbar_win_impl(wp, make_room, true) == SET_WINBAR_WIN_FAIL {
            break;
        }
        wp = nvim_win_get_next(wp);
    }
}

/// FFI export for `set_winbar`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(export_name = "set_winbar")]
pub unsafe extern "C" fn rs_set_winbar(make_room: c_int) {
    set_winbar_impl(make_room != 0);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_height_constant() {
        assert_eq!(STATUS_HEIGHT, 1);
    }

    #[test]
    fn test_null_resizable_frame() {
        assert!(find_horizontally_resizable_frame_impl(std::ptr::null_mut()).is_null());
    }

    #[test]
    fn test_null_remove_status_line() {
        // Should not panic
        win_remove_status_line_impl(WinHandle::null(), false);
    }

    #[test]
    fn test_null_resize_for_status() {
        assert!(!resize_frame_for_status_impl(std::ptr::null_mut()));
    }

    #[test]
    fn test_null_resize_for_winbar() {
        assert!(!resize_frame_for_winbar_impl(std::ptr::null_mut()));
    }

    #[test]
    fn test_null_last_status_rec() {
        // Should not panic
        last_status_rec_impl(std::ptr::null_mut(), false, false);
    }
}
