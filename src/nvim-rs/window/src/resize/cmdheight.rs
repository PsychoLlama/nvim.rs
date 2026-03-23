//! Command-height resize logic (`command_height`).
//!
//! This module provides `rs_command_height`, the Rust replacement for
//! `command_height()` in `src/nvim/window_shim.c`.
//!
//! The function is called whenever the `p_ch` ('cmdheight') option changes.
//! It adjusts frame heights, recomputes positions, and clears the cmdline area.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_LEAF};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    // curtab->tp_ch_used
    fn nvim_get_curtab_ch_used() -> c_int;

    // lastwin_nofloating() (already a Rust function)
    #[link_name = "rs_lastwin_nofloating"]
    fn nvim_lastwin_nofloating() -> WinHandle;

    // w_frame from window
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    // Columns global

    // p_ch option
    static mut p_ch: i64;

    // command_frame_height static
    fn nvim_get_command_frame_height() -> c_int;

    // rs_frame_minheight
    fn rs_frame_minheight(frp: *const Frame, next_curwin: WinHandle) -> c_int;

    // rs_frame_add_height
    fn rs_frame_add_height(frp: *mut Frame, n: c_int);

    // rs_win_comp_pos
    fn rs_win_comp_pos() -> c_int;

    // cmdline_row setter via: Rows - p_ch
    fn nvim_update_cmdline_row();

    // set redraw_cmdline = true
    fn nvim_set_redraw_cmdline(val: bool);

    // grid_clear for cmdheight area (handles msg_grid_adj vs default_gridview)
    fn nvim_grid_clear_cmd_area();

    // curtab->tp_ch_used = p_ch
    fn nvim_set_curtab_ch_used(val: i64);

    // min_set_ch = p_ch
    fn nvim_set_min_set_ch(val: i64);

    // emsg(e_noroom)
    fn nvim_emsg_id(id: c_int);

    // set p_ch (only for restoring when no room)
    fn nvim_set_p_ch(val: i64);

    // w_p_wfh: winfixheight option
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;
}

// =============================================================================
// EMSG IDs
// =============================================================================

const EMSG_NOROOM: c_int = 13;

// =============================================================================
// Implementation
// =============================================================================

/// Resize the command-line area when `p_ch` changes.
///
/// Rust port of C `command_height()`.
///
/// # Safety
///
/// Accesses global Neovim state through C accessor functions.
/// Modifies frame heights directly via the repr(C) Frame structure.
fn command_height_impl() {
    unsafe {
        let old_p_ch = nvim_get_curtab_ch_used();

        // Find bottom frame with width of screen.
        let lastwin = nvim_lastwin_nofloating();
        if lastwin.is_null() {
            return;
        }
        let mut frp = nvim_win_get_frame(lastwin);
        if frp.is_null() {
            return;
        }

        let columns = Columns;
        while !frp.is_null() && (*frp).fr_width != columns {
            let parent = (*frp).fr_parent;
            if parent.is_null() {
                break;
            }
            frp = parent;
        }

        // Avoid changing the height of a window with 'winfixheight' set.
        while !frp.is_null() {
            let prev = (*frp).fr_prev;
            if prev.is_null() {
                break;
            }
            // Only skip FR_LEAF frames with w_p_wfh set.
            if (*frp).fr_layout != FR_LEAF {
                break;
            }
            let fr_win = (*frp).fr_win;
            if fr_win.is_null() {
                break;
            }
            // Check w_p_wfh via the already-available accessor
            if nvim_win_get_wfh(fr_win) == 0 {
                break;
            }
            frp = prev;
        }

        let p_ch_int = p_ch as c_int;
        let command_frame_height = nvim_get_command_frame_height() != 0;

        // Grow cmdheight: shrink frames from bottom.
        let mut cur_p_ch = old_p_ch;
        while p_ch_int > cur_p_ch && command_frame_height {
            if frp.is_null() {
                nvim_emsg_id(EMSG_NOROOM);
                nvim_set_p_ch(i64::from(cur_p_ch));
                break;
            }
            let available = (*frp).fr_height - rs_frame_minheight(frp, WinHandle::null());
            let h = (p_ch_int - cur_p_ch).min(available);
            rs_frame_add_height(frp, -h);
            cur_p_ch += h;
            frp = (*frp).fr_prev;
        }

        // Shrink cmdheight: grow the frame.
        if p_ch_int < cur_p_ch && command_frame_height && !frp.is_null() {
            rs_frame_add_height(frp, cur_p_ch - p_ch_int);
        }

        // Recompute window positions.
        rs_win_comp_pos();
        nvim_update_cmdline_row();
        nvim_set_redraw_cmdline(true);

        // Clear the cmdheight area.
        nvim_grid_clear_cmd_area();

        // Use the value of p_ch that we remembered. This is needed for when the
        // GUI starts up, and when p_ch was changed in another tab page.
        nvim_set_curtab_ch_used(p_ch);
        nvim_set_min_set_ch(p_ch);
    }
}

// =============================================================================
// FFI Export
// =============================================================================

/// FFI wrapper for `command_height`.
///
/// Called from C thin wrapper `void command_height()`.
///
/// # Safety
///
/// Called from C via FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_command_height() {
    command_height_impl();
}

/// C export: `command_height` — eliminates the C thin wrapper.
///
/// # Safety
/// Called from C via FFI.
#[unsafe(export_name = "command_height")]
pub unsafe extern "C" fn command_height() {
    command_height_impl();
}
