//! Window focus and navigation functions.
//!
//! This module provides Rust implementations of window focus and navigation
//! functions from `src/nvim/window.c`.

use std::ffi::c_int;

use crate::win_struct::{win_mut, win_ref};
use crate::{BufHandle, Frame, TabpageHandle, WinHandle, FR_COL, FR_LEAF, FR_ROW};

use crate::list::{frame2win_impl, nvim_get_firstwin, win_valid_impl};

extern "C" {
    static mut State: c_int;
    /// Get the `w_wcol` field from a window (cursor column in window).

    /// Get the `w_wrow` field from a window (cursor row in window).

    /// Get the `w_winrow` field from a window.

    /// Get the `w_wincol` field from a window.

    /// Get the `tp_topframe` field from a tabpage.
    fn nvim_tabpage_get_topframe(tp: TabpageHandle) -> *mut Frame;

    /// Get the prevwin global.
    fn nvim_get_prevwin() -> WinHandle;

    // --- win_goto dependencies ---

    /// Get curwin global.
    fn nvim_get_curwin() -> WinHandle;

    fn text_or_buf_locked() -> bool;

    /// Call beep_flush().
    #[link_name = "beep_flush"]
    fn nvim_beep_flush();

    /// Get wp->w_buffer.

    /// Get curbuf.
    fn nvim_get_curbuf() -> BufHandle;

    /// Reset VIsual and resel.
    fn rs_reset_VIsual_and_resel();

    /// Get VIsual_active flag.
    static mut VIsual_active: bool;

    /// Get wp->w_cursor.lnum.

    /// Set wp->w_cursor.lnum.

    /// Get wp->w_cursor.col.

    /// Set wp->w_cursor.col.

    /// Get wp->w_cursor.coladd.

    /// Set wp->w_cursor.coladd.

    /// Check if window is valid. Returns 1 if valid.
    fn rs_win_valid(wp: WinHandle) -> c_int;

    /// Enter window wp (calls win_enter).
    fn nvim_win_enter(wp: WinHandle, undo_sync: c_int);

    /// Get wp->w_p_cole (conceal option).
    fn nvim_win_get_p_cole(wp: WinHandle) -> i64;

    /// Get msg_scrolled flag.
    static mut msg_scrolled: c_int;

    /// Redraw window line.
    #[link_name = "redrawWinline"]
    fn nvim_redrawWinline(wp: WinHandle, lnum: c_int);

    // --- can_close_in_cmdwin dependencies (Phase 8 inlined) ---

    /// Get cmdwin_type global.
    static cmdwin_type: c_int;

    /// Get cmdwin_win global.
    fn nvim_get_cmdwin_win() -> WinHandle;

    /// Get cmdwin_old_curwin global.
    fn nvim_get_cmdwin_old_curwin() -> WinHandle;

    /// Set cmdwin_result to val.
    fn nvim_set_cmdwin_result(val: c_int);

    /// Set api_error for cmdwin.
    fn nvim_api_set_error_e_cmdwin(err: *mut std::ffi::c_void);
}

/// Get the above or below neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specified window is a floating window.
///
/// This is the Rust equivalent of `win_vert_neighbor()` in window.c.
#[allow(clippy::too_many_lines)]
pub(crate) fn win_vert_neighbor_impl(
    tp: TabpageHandle,
    wp: WinHandle,
    up: bool,
    mut count: c_int,
) -> WinHandle {
    if wp.is_null() || tp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut foundfr = win_ref(wp).w_frame;

        // If floating window, return prevwin if valid and non-floating, else firstwin
        if win_ref(wp).w_floating {
            let prevwin = nvim_get_prevwin();
            let firstwin = nvim_get_firstwin();
            return if win_valid_impl(prevwin) && c_int::from(win_ref(prevwin).w_floating) == 0 {
                prevwin
            } else {
                firstwin
            };
        }

        let topframe = nvim_tabpage_get_topframe(tp);

        while count > 0 {
            count -= 1;
            // First go upwards in the tree of frames until we find an upwards or
            // downwards neighbor.
            let mut fr = foundfr;
            let mut nfr;
            loop {
                if fr == topframe {
                    // Reached top, return what we found
                    return if foundfr.is_null() {
                        WinHandle::null()
                    } else {
                        (*foundfr).fr_win
                    };
                }
                nfr = if up { (*fr).fr_prev } else { (*fr).fr_next };
                let parent = (*fr).fr_parent;
                if !parent.is_null() && (*parent).fr_layout == FR_COL && !nfr.is_null() {
                    break;
                }
                fr = parent;
            }

            // Now go downwards to find the bottom or top frame in it.
            loop {
                if (*nfr).fr_layout == FR_LEAF {
                    foundfr = nfr;
                    break;
                }
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout == FR_ROW {
                    // Find the frame at the cursor column.
                    let wp_wincol = win_ref(wp).w_wincol;
                    let wp_wcol = win_ref(wp).w_wcol;
                    while !(*fr).fr_next.is_null() {
                        let fr_win = frame2win_impl(fr);
                        let fr_wincol = win_ref(fr_win).w_wincol;
                        if fr_wincol + (*fr).fr_width > wp_wincol + wp_wcol {
                            break;
                        }
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout == FR_COL && up {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }

        if foundfr.is_null() {
            WinHandle::null()
        } else {
            (*foundfr).fr_win
        }
    }
}

/// Get the left or right neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specified window is a floating window.
///
/// This is the Rust equivalent of `win_horz_neighbor()` in window.c.
#[allow(clippy::too_many_lines)]
pub(crate) fn win_horz_neighbor_impl(
    tp: TabpageHandle,
    wp: WinHandle,
    left: bool,
    mut count: c_int,
) -> WinHandle {
    if wp.is_null() || tp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut foundfr = win_ref(wp).w_frame;

        // If floating window, return prevwin if valid and non-floating, else firstwin
        if win_ref(wp).w_floating {
            let prevwin = nvim_get_prevwin();
            let firstwin = nvim_get_firstwin();
            return if win_valid_impl(prevwin) && c_int::from(win_ref(prevwin).w_floating) == 0 {
                prevwin
            } else {
                firstwin
            };
        }

        let topframe = nvim_tabpage_get_topframe(tp);

        while count > 0 {
            count -= 1;
            // First go upwards in the tree of frames until we find a left or
            // right neighbor.
            let mut fr = foundfr;
            let mut nfr;
            loop {
                if fr == topframe {
                    // Reached top, return what we found
                    return if foundfr.is_null() {
                        WinHandle::null()
                    } else {
                        (*foundfr).fr_win
                    };
                }
                nfr = if left { (*fr).fr_prev } else { (*fr).fr_next };
                let parent = (*fr).fr_parent;
                if !parent.is_null() && (*parent).fr_layout == FR_ROW && !nfr.is_null() {
                    break;
                }
                fr = parent;
            }

            // Now go downwards to find the leftmost or rightmost frame in it.
            loop {
                if (*nfr).fr_layout == FR_LEAF {
                    foundfr = nfr;
                    break;
                }
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout == FR_COL {
                    // Find the frame at the cursor row.
                    let wp_winrow = win_ref(wp).w_winrow;
                    let wp_wrow = win_ref(wp).w_wrow;
                    while !(*fr).fr_next.is_null() {
                        let fr_win = frame2win_impl(fr);
                        let fr_winrow = win_ref(fr_win).w_winrow;
                        if fr_winrow + (*fr).fr_height > wp_winrow + wp_wrow {
                            break;
                        }
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout == FR_ROW && left {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }

        if foundfr.is_null() {
            WinHandle::null()
        } else {
            (*foundfr).fr_win
        }
    }
}

// Note: FFI wrappers rs_win_vert_neighbor and rs_win_horz_neighbor are in lib.rs

// =============================================================================
// leaving_window / entering_window -- prompt buffer state management
// =============================================================================

/// MODE_INSERT state flag (same value as in C: 0x10).
const MODE_INSERT: c_int = 0x10;

/// NUL character value.
const NUL: c_int = 0;

extern "C" {
    fn rs_bt_prompt(buf: BufHandle) -> bool;

    /// Get b_prompt_insert from window's buffer.
    fn nvim_buf_get_prompt_insert(buf: *mut std::ffi::c_void) -> c_int;

    /// Set b_prompt_insert on window's buffer.
    fn nvim_buf_set_prompt_insert(buf: *mut std::ffi::c_void, val: c_int);

    /// Get the restart_edit global.
    static mut restart_edit: c_int;

    /// Set the restart_edit global.
    /// Get the mode_displayed global (returns bool from C).
    fn nvim_get_mode_displayed() -> bool;

    /// Set clear_cmdline global (takes bool in C).
    fn nvim_set_clear_cmdline(val: bool);

    /// Get stop_insert_mode global.
    fn nvim_get_stop_insert_mode() -> c_int;

    /// Set stop_insert_mode global.
    fn nvim_set_stop_insert_mode(val: c_int);

    /// Get win's buffer pointer.
    fn nvim_win_get_buf_ptr(wp: WinHandle) -> *mut std::ffi::c_void;
}

/// Handle prompt buffer state when leaving a window.
///
/// Port of C `leaving_window()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn leaving_window_impl(win: WinHandle) {
    if !rs_bt_prompt(BufHandle(win_ref(win).w_buffer)) {
        return;
    }

    let buf = nvim_win_get_buf_ptr(win);

    // Save restart_edit into b_prompt_insert.
    nvim_buf_set_prompt_insert(buf, restart_edit);

    // If we were showing Insert mode, unshow it later.
    if restart_edit != NUL && nvim_get_mode_displayed() {
        nvim_set_clear_cmdline(true);
    }
    restart_edit = NUL;

    // If in Insert mode and not stopping already, break out and restart on re-entry.
    if (State & MODE_INSERT) != 0 && nvim_get_stop_insert_mode() == 0 {
        nvim_set_stop_insert_mode(1);
        if nvim_buf_get_prompt_insert(buf) == NUL {
            nvim_buf_set_prompt_insert(buf, c_int::from(b'A'));
        }
    }
}

/// Handle prompt buffer state when entering a window.
///
/// Port of C `entering_window()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn entering_window_impl(win: WinHandle) {
    if !rs_bt_prompt(BufHandle(win_ref(win).w_buffer)) {
        return;
    }

    let buf = nvim_win_get_buf_ptr(win);

    // Don't stop Insert mode if we were in it when we left.
    if nvim_buf_get_prompt_insert(buf) != NUL {
        nvim_set_stop_insert_mode(0);
    }

    // Restart Insert mode if we were in it and not already in Insert mode.
    if (State & MODE_INSERT) == 0 {
        restart_edit = nvim_buf_get_prompt_insert(buf);
    }
}

/// FFI export for `leaving_window`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_leaving_window(win: WinHandle) {
    leaving_window_impl(win);
}

/// C export: `leaving_window` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(export_name = "leaving_window")]
pub unsafe extern "C" fn leaving_window(win: WinHandle) {
    leaving_window_impl(win);
}

/// FFI export for `entering_window`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_entering_window(win: WinHandle) {
    entering_window_impl(win);
}

/// C export: `entering_window` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(export_name = "entering_window")]
pub unsafe extern "C" fn entering_window(win: WinHandle) {
    entering_window_impl(win);
}

// =============================================================================
// win_goto
// =============================================================================

/// Switch to window `wp` respecting Visual mode, conceal, and text-lock.
///
/// Port of C `win_goto()`.
///
/// # Safety
/// Calls C accessor functions with valid window handles.
unsafe fn win_goto_impl(wp: WinHandle) {
    let owp = nvim_get_curwin();

    if text_or_buf_locked() {
        nvim_beep_flush();
        return;
    }

    if BufHandle(win_ref(wp).w_buffer) != nvim_get_curbuf() {
        // careful: triggers ModeChanged autocommand
        rs_reset_VIsual_and_resel();
    } else if VIsual_active {
        // Set wp->w_cursor = curwin->w_cursor
        win_mut(wp).w_cursor.lnum = win_ref(owp).w_cursor.lnum;
        win_mut(wp).w_cursor.col = win_ref(owp).w_cursor.col;
        win_mut(wp).w_cursor.coladd = win_ref(owp).w_cursor.coladd;
    }

    // autocommand may have made wp invalid
    if rs_win_valid(wp) == 0 {
        return;
    }

    nvim_win_enter(wp, 1);

    // Conceal cursor line in previous window, unconceal in current window.
    if rs_win_valid(owp) != 0 && nvim_win_get_p_cole(owp) > 0 && msg_scrolled == 0 {
        nvim_redrawWinline(owp, win_ref(owp).w_cursor.lnum);
    }
    let curwin = nvim_get_curwin();
    if nvim_win_get_p_cole(curwin) > 0 && msg_scrolled == 0 {
        nvim_redrawWinline(curwin, win_ref(curwin).w_cursor.lnum);
    }
}

/// FFI export for `win_goto`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_goto(wp: WinHandle) {
    win_goto_impl(wp);
}

/// C export: `win_goto` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(export_name = "win_goto")]
pub unsafe extern "C" fn win_goto(wp: WinHandle) {
    win_goto_impl(wp);
}

// =============================================================================
// can_close_in_cmdwin
// =============================================================================

/// Check if window `win` can close in a command-line window context.
///
/// Inlined port of C `nvim_can_close_in_cmdwin_check()` (Phase 8).
/// Returns `true` if the window can close (no cmdwin conflict), `false` otherwise.
/// Side effects: may set `cmdwin_result = Ctrl_C` or set an API error.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
unsafe fn can_close_in_cmdwin_impl(win: WinHandle, err: *mut std::ffi::c_void) -> bool {
    if cmdwin_type != 0 {
        let cmdwin_win = nvim_get_cmdwin_win();
        if win == cmdwin_win {
            // Ctrl-C (3) closes the cmdwin itself
            nvim_set_cmdwin_result(3); // Ctrl_C
            return false;
        }
        let cmdwin_old = nvim_get_cmdwin_old_curwin();
        if win == cmdwin_old {
            nvim_api_set_error_e_cmdwin(err);
            return false;
        }
    }
    true
}

/// FFI export for `can_close_in_cmdwin`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(export_name = "can_close_in_cmdwin")]
pub unsafe extern "C" fn rs_can_close_in_cmdwin(
    win: WinHandle,
    err: *mut std::ffi::c_void,
) -> bool {
    can_close_in_cmdwin_impl(win, err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_args() {
        // Test with null window
        assert!(win_vert_neighbor_impl(
            unsafe { TabpageHandle::from_ptr(0x1000 as *mut _) },
            WinHandle::null(),
            true,
            1
        )
        .is_null());

        // Test with null tabpage
        assert!(win_horz_neighbor_impl(
            unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) },
            unsafe { WinHandle::from_ptr(0x1000 as *mut _) },
            true,
            1
        )
        .is_null());
    }
}
