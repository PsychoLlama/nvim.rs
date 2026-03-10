//! Popup menu preview window management.
//!
//! This module handles the floating preview window that shows
//! completion item info text alongside the popup menu.

use std::ffi::{c_char, c_int};

use crate::display::{BufHandle, WinHandle};
use crate::PUM_STATE;

// C accessor functions for preview window operations.
extern "C" {
    /// Check if selected item matches current completion selection.
    fn nvim_pum_compl_match_curr_select(selected: c_int) -> c_int;
    /// Block autocmds.
    fn nvim_pum_block_autocmds();
    /// Unblock autocmds.
    fn nvim_pum_unblock_autocmds();
    /// Increment `RedrawingDisabled`.
    fn nvim_pum_redrawing_disabled_inc();
    /// Decrement `RedrawingDisabled`.
    fn nvim_pum_redrawing_disabled_dec();
    /// Increment `no_u_sync`.
    fn nvim_pum_no_u_sync_inc();
    /// Decrement `no_u_sync`.
    fn nvim_pum_no_u_sync_dec();
    /// Find the floating preview window.
    fn nvim_pum_win_float_find_preview() -> *mut WinHandle;
    /// Create a floating info window.
    fn nvim_pum_win_float_create_info() -> *mut WinHandle;
    /// Set `w_topline` for a window.
    fn nvim_pum_win_set_topline(wp: *mut WinHandle, val: c_int);
    /// Set `w_p_wfb` for a window.
    fn nvim_pum_win_set_wfb(wp: *mut WinHandle, val: c_int);
    /// Get buffer from a window.
    fn nvim_pum_win_get_buffer(wp: *mut WinHandle) -> *mut BufHandle;
    /// Call `redraw_later` for a window.
    fn nvim_pum_redraw_later_win(wp: *mut WinHandle, update_type: c_int);
    /// Set preview text in buffer (C wrapper for `nvim_buf_set_lines`).
    fn nvim_pum_preview_set_text_impl(
        buf: *mut BufHandle,
        info: *mut c_char,
        lnum: *mut i32,
        max_width: *mut c_int,
    );
}

// C accessor functions for adjust_info_position.
extern "C" {
    /// Get `Columns`.
    fn nvim_get_Columns() -> c_int;
    /// Get `Rows`.
    fn nvim_get_Rows() -> c_int;
    /// Get line count for window's buffer.
    fn nvim_pum_win_get_line_count(wp: *mut WinHandle) -> c_int;
    /// Wrapper for `plines_m_win`.
    fn nvim_pum_plines_m_win(wp: *mut WinHandle, first: c_int, last: c_int, max: c_int) -> c_int;
    /// Set window config fields and apply via `win_config_float`.
    fn nvim_pum_win_config_set_and_apply(
        wp: *mut WinHandle,
        width: c_int,
        col: c_int,
        anchor: c_int,
        height: c_int,
        row: c_int,
        hide: c_int,
    );
    /// Get border width from Rust.
    fn rs_pum_border_width() -> c_int;
}

/// `UPD_NOT_VALID` from drawscreen.h.
const UPD_NOT_VALID: c_int = 40;

/// `kFloatAnchorSouth` from `buffer_defs.h`.
const K_FLOAT_ANCHOR_SOUTH: c_int = 2;

/// Set the informational text in the preview buffer.
///
/// Delegates to C wrapper that handles Arena/Array/`nvim_buf_set_lines`.
///
/// # Safety
/// All pointers must be valid. `info` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_preview_set_text(
    buf: *mut BufHandle,
    info: *mut c_char,
    lnum: *mut i32,
    max_width: *mut c_int,
) {
    nvim_pum_preview_set_text_impl(buf, info, lnum, max_width);
}

/// Adjust floating info preview window position.
///
/// Calculates the optimal position for the info preview window
/// relative to the popup menu, placing it to the right or left
/// depending on available space.
///
/// # Safety
/// `wp` must be a valid `win_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_adjust_info_position(wp: *mut WinHandle, width: c_int) {
    let border_width = rs_pum_border_width();
    let pum_col = PUM_STATE.col;
    let pum_width = PUM_STATE.width;
    let pum_scrollbar = PUM_STATE.scrollbar;
    let columns = nvim_get_Columns();
    let pum_above = PUM_STATE.above != 0;
    let pum_row = PUM_STATE.row;

    let mut col = pum_col + pum_width + 1 + border_width;
    if border_width < 0 {
        col += pum_scrollbar;
    }

    let right_extra = columns - col;
    let left_extra = pum_col - 2;

    let (cfg_width, cfg_col) = if right_extra > width {
        // Place to the right
        (width, col - 1)
    } else if left_extra > width {
        // Place to the left
        (width, pum_col - width - 1)
    } else {
        // Use whichever side has more space
        let place_right = right_extra > left_extra;
        if place_right {
            (right_extra, col - 1)
        } else {
            (left_extra, pum_col - left_extra - 1)
        }
    };

    let anchor = if pum_above { K_FLOAT_ANCHOR_SOUTH } else { 0 };
    let line_count = nvim_pum_win_get_line_count(wp);
    let rows = nvim_get_Rows();
    let height = nvim_pum_plines_m_win(wp, 1, line_count, rows);
    let row = if pum_above { pum_row + height } else { pum_row };

    nvim_pum_win_config_set_and_apply(wp, cfg_width, cfg_col, anchor, height, row, 0);
}

/// Set info for a completed item, returning a window pointer.
///
/// Creates or reuses a floating preview window, sets the info text,
/// and positions it next to the popup menu.
///
/// Returns a window pointer, or null if not visible or no match.
///
/// # Safety
/// `info` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_info(selected: c_int, info: *mut c_char) -> *mut WinHandle {
    if PUM_STATE.is_visible == 0 || nvim_pum_compl_match_curr_select(selected) == 0 {
        return std::ptr::null_mut();
    }

    nvim_pum_block_autocmds();
    nvim_pum_redrawing_disabled_inc();
    nvim_pum_no_u_sync_inc();

    let mut wp = nvim_pum_win_float_find_preview();
    if wp.is_null() {
        wp = nvim_pum_win_float_create_info();
        if wp.is_null() {
            nvim_pum_no_u_sync_dec();
            nvim_pum_redrawing_disabled_dec();
            nvim_pum_unblock_autocmds();
            return std::ptr::null_mut();
        }
        nvim_pum_win_set_topline(wp, 1);
        nvim_pum_win_set_wfb(wp, 1);
    }

    let mut lnum: i32 = 0;
    let mut max_info_width: c_int = 0;
    let buf = nvim_pum_win_get_buffer(wp);
    nvim_pum_preview_set_text_impl(
        buf,
        info,
        std::ptr::addr_of_mut!(lnum),
        std::ptr::addr_of_mut!(max_info_width),
    );

    nvim_pum_no_u_sync_dec();
    nvim_pum_redrawing_disabled_dec();
    nvim_pum_redraw_later_win(wp, UPD_NOT_VALID);

    rs_pum_adjust_info_position(wp, max_info_width);
    nvim_pum_unblock_autocmds();
    wp
}
