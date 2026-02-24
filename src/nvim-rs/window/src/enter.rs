//! Window enter (`win_enter_ext`) implementation.
//!
//! This module provides `rs_win_enter_ext`, the Rust replacement for
//! `win_enter_ext()` in `src/nvim/window_shim.c`.
//!
//! The function is the core window-switch orchestrator: it handles
//! BufLeave/WinLeave/WinNew/WinEnter/BufEnter autocmds, cursor positioning,
//! NormalNC highlight updating, and height/width enforcement.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_int;

use crate::{BufHandle, WinHandle};

// =============================================================================
// WEE flag constants (matching wee_flags_T in window_shim.c)
// =============================================================================

const WEE_UNDO_SYNC: c_int = 0x01;
const WEE_CURWIN_INVALID: c_int = 0x02;
const WEE_TRIGGER_NEW_AUTOCMDS: c_int = 0x04;
const WEE_TRIGGER_ENTER_AUTOCMDS: c_int = 0x08;
const WEE_TRIGGER_LEAVE_AUTOCMDS: c_int = 0x10;

// Mode constants (matching state_defs.h)
const MODE_NORMAL: c_int = 0x01;
const MODE_CMDLINE: c_int = 0x08;
const MODE_TERMINAL: c_int = 0x80;

// UPD_ redraw type constants
const UPD_VALID: c_int = 20;
const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Window validity
    fn rs_win_valid(wp: WinHandle) -> c_int;

    // Global accessors
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_prevwin() -> WinHandle;
    fn nvim_set_prevwin(wp: WinHandle);
    fn nvim_set_curwin(wp: WinHandle);
    fn nvim_set_curbuf(buf: BufHandle);
    fn nvim_get_curbuf() -> BufHandle;

    // Window field accessors
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_set_redr_status(wp: WinHandle, val: c_int);
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;
    fn nvim_win_get_wfw(wp: WinHandle) -> c_int;
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_hl_attr_normal_wrap(wp: WinHandle) -> c_int;
    fn nvim_win_get_hl_attr_normalnc_wrap(wp: WinHandle) -> c_int;
    fn nvim_win_set_cursor_coladd(wp: WinHandle, val: c_int);

    // p_spk (splitkeep option) -- returns first char as int
    fn nvim_win_get_p_spk_char() -> c_int;

    // p_wh (winheight), p_wiw (winwidth)
    fn nvim_get_p_wh() -> i64;
    fn nvim_get_p_wiw() -> i64;

    // Autocmd wrappers
    fn nvim_apply_autocmds_bufleave();
    fn nvim_apply_autocmds_winleave();
    fn nvim_apply_autocmds_winnew();
    fn nvim_apply_autocmds_winenter();
    fn nvim_apply_autocmds_bufenter();

    // aborting() check
    fn nvim_aborting() -> bool;

    // u_sync
    fn nvim_u_sync(force: bool);

    // update_topline for curwin
    fn nvim_update_topline_curwin_enter();

    // buf_copy_options(buf, BCO_ENTER | BCO_NOHELP)
    fn nvim_buf_copy_options_enter(buf: BufHandle);

    // check_cursor(curwin)
    fn nvim_check_cursor_win_wrapper(wp: WinHandle);

    // virtual_active(curwin)
    fn nvim_virtual_active() -> bool;

    // changed_line_abv_curs()
    fn nvim_changed_line_abv_curs_wrap();

    // win_fix_cursor (already in Rust: rs_win_fix_cursor)
    fn rs_win_fix_cursor(normal: c_int);

    // win_fix_current_dir
    fn win_fix_current_dir();

    // leaving_window / entering_window (already in Rust)
    fn rs_leaving_window(wp: WinHandle);
    fn rs_entering_window(wp: WinHandle);

    // maketitle()
    fn nvim_maketitle();

    // redraw_later(wp, type)
    fn nvim_redraw_later_wrapper(wp: WinHandle, type_: c_int);

    // redraw_tabline flag
    fn nvim_set_redraw_tabline(val: c_int);

    // restart_edit
    fn nvim_get_restart_edit_bool() -> c_int;

    // setmouse()
    fn nvim_setmouse();

    // do_autochdir()
    fn nvim_do_autochdir_wrap();

    // rs_win_setheight / rs_win_setwidth
    fn rs_win_setheight(height: c_int);
    fn rs_win_setwidth(width: c_int);

    // get_real_state()
    fn nvim_get_real_state() -> c_int;
}

// =============================================================================
// Implementation
// =============================================================================

/// Core window-switch function.
///
/// Rust port of C `win_enter_ext()`. Handles leaving/entering autocmds,
/// cursor positioning, height/width enforcement, and NormalNC redraws.
///
/// # Safety
///
/// Accesses global Neovim state through C accessor functions.
/// Autocmd calls can invalidate windows; we re-check `rs_win_valid(wp)` after each.
fn win_enter_ext_impl(wp: WinHandle, flags: c_int) {
    // SAFETY: All calls go through C accessor functions.
    unsafe {
        let curwin_invalid = (flags & WEE_CURWIN_INVALID) != 0;
        let curwin = nvim_get_curwin();

        // Nothing to do if switching to the same window (unless curwin is invalid).
        if wp == curwin && !curwin_invalid {
            return;
        }

        if !curwin_invalid {
            rs_leaving_window(curwin);
        }

        let mut other_buffer = false;

        if !curwin_invalid && (flags & WEE_TRIGGER_LEAVE_AUTOCMDS) != 0 {
            // Be careful: if autocommands delete the window, return now.
            let curbuf = nvim_get_curbuf();
            let wp_buffer = nvim_win_get_buffer(wp);
            if wp_buffer != curbuf {
                nvim_apply_autocmds_bufleave();
                other_buffer = true;
                if rs_win_valid(wp) == 0 {
                    return;
                }
            }
            nvim_apply_autocmds_winleave();
            if rs_win_valid(wp) == 0 {
                return;
            }
            // autocmds may abort script processing
            if nvim_aborting() {
                return;
            }
        }

        // sync undo before leaving the current buffer
        if (flags & WEE_UNDO_SYNC) != 0 {
            let curbuf = nvim_get_curbuf();
            let wp_buffer = nvim_win_get_buffer(wp);
            if curbuf != wp_buffer {
                nvim_u_sync(false);
            }
        }

        // Might need to scroll the old window before switching
        // ('splitkeep' == 'c' means cursor-based scrolling)
        let p_spk = nvim_win_get_p_spk_char();
        if p_spk == b'c' as c_int && !curwin_invalid {
            nvim_update_topline_curwin_enter();
        }

        // may have to copy the buffer options when 'cpo' contains 'S'
        let curbuf = nvim_get_curbuf();
        let wp_buffer = nvim_win_get_buffer(wp);
        if wp_buffer != curbuf {
            nvim_buf_copy_options_enter(wp_buffer);
        }

        if !curwin_invalid {
            // remember for CTRL-W p
            nvim_set_prevwin(curwin);
            nvim_win_set_redr_status(curwin, 1);
        }
        nvim_set_curwin(wp);
        nvim_set_curbuf(nvim_win_get_buffer(wp));

        // Re-read curwin (it's now wp)
        let new_curwin = nvim_get_curwin();

        nvim_check_cursor_win_wrapper(new_curwin);
        if !nvim_virtual_active() {
            nvim_win_set_cursor_coladd(new_curwin, 0);
        }

        let p_spk = nvim_win_get_p_spk_char();
        if p_spk == b'c' as c_int {
            nvim_changed_line_abv_curs_wrap();
        } else {
            // Make sure the cursor position is valid, either by moving the cursor
            // or by scrolling the text.
            let state = nvim_get_real_state() & (MODE_NORMAL | MODE_CMDLINE | MODE_TERMINAL);
            rs_win_fix_cursor(state);
        }

        win_fix_current_dir();

        rs_entering_window(new_curwin);

        // Careful: autocommands may close the window and make "wp" invalid
        if (flags & WEE_TRIGGER_NEW_AUTOCMDS) != 0 {
            nvim_apply_autocmds_winnew();
        }
        if (flags & WEE_TRIGGER_ENTER_AUTOCMDS) != 0 {
            nvim_apply_autocmds_winenter();
            if other_buffer {
                nvim_apply_autocmds_bufenter();
            }
        }

        nvim_maketitle();

        // Re-read curwin in case autocmds changed it
        let new_curwin = nvim_get_curwin();
        nvim_win_set_redr_status(new_curwin, 1);
        nvim_set_redraw_tabline(1);

        if nvim_get_restart_edit_bool() != 0 {
            nvim_redraw_later_wrapper(new_curwin, UPD_VALID);
        }

        // Change background color according to NormalNC,
        // but only if actually defined (otherwise no extra redraw).
        let hl_normal = nvim_win_get_hl_attr_normal_wrap(new_curwin);
        let hl_normalnc = nvim_win_get_hl_attr_normalnc_wrap(new_curwin);
        if hl_normal != hl_normalnc {
            nvim_redraw_later_wrapper(new_curwin, UPD_NOT_VALID);
        }

        let prevwin = nvim_get_prevwin();
        if !prevwin.is_null() {
            let pw_normal = nvim_win_get_hl_attr_normal_wrap(prevwin);
            let pw_normalnc = nvim_win_get_hl_attr_normalnc_wrap(prevwin);
            if pw_normal != pw_normalnc {
                nvim_redraw_later_wrapper(prevwin, UPD_NOT_VALID);
            }
        }

        // set window height to desired minimal value
        let p_wh = nvim_get_p_wh() as c_int;
        let height = nvim_win_get_w_height(new_curwin);
        let wfh = nvim_win_get_wfh(new_curwin);
        let floating = nvim_win_get_floating(new_curwin);
        if height < p_wh && wfh == 0 && floating == 0 {
            rs_win_setheight(p_wh);
        } else if height == 0 {
            rs_win_setheight(1);
        }

        // set window width to desired minimal value
        let p_wiw = nvim_get_p_wiw() as c_int;
        let width = nvim_win_get_w_width(new_curwin);
        let wfw = nvim_win_get_wfw(new_curwin);
        if width < p_wiw && wfw == 0 && floating == 0 {
            rs_win_setwidth(p_wiw);
        }

        nvim_setmouse(); // in case jumped to/from help buffer

        // Change directories when the 'acd' option is set.
        nvim_do_autochdir_wrap();
    }
}

// =============================================================================
// FFI Export
// =============================================================================

/// FFI wrapper for `win_enter_ext`.
///
/// Called from C's thin wrapper `static void win_enter_ext(...)`.
///
/// # Safety
///
/// Called from C via FFI. All pointer-based operations go through C wrappers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_enter_ext(wp: WinHandle, flags: c_int) {
    win_enter_ext_impl(wp, flags);
}
