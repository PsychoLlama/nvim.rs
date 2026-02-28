//! Implementation of `close_others`.
//!
//! Closes all windows except the current one. Used by `:only` and related
//! commands. Autocommands may change the window layout during iteration,
//! so we faithfully replicate the C control flow including the
//! "restart from firstwin" pattern.

use std::ffi::c_int;

use crate::{BufHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get the current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Set curwin and curbuf from wp.
    fn nvim_set_curwin_from_wp(wp: WinHandle);

    /// Get the first window in the current tab.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get the last window in the current tab.
    fn nvim_get_lastwin() -> WinHandle;

    /// Get w_next from a window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Validate a window pointer.
    fn rs_win_valid(wp: WinHandle) -> c_int;

    /// Check if there is only one window in the tab.
    fn rs_one_window_in_tab(win: WinHandle, tp: crate::TabpageHandle) -> c_int;

    /// Get raw buffer pointer for a window.
    fn nvim_win_get_buffer_raw(wp: WinHandle) -> BufHandle;

    /// Set buffer to NULL on a window.
    fn nvim_win_set_buffer_raw(wp: WinHandle, buf: BufHandle);

    /// Validate a buffer pointer.
    fn nvim_buf_valid_ptr(buf: BufHandle) -> c_int;

    /// Check if buffer can be abandoned.
    fn nvim_win_can_abandon(wp: WinHandle, forceit: c_int) -> c_int;

    /// Prompt user to save (dialog_changed).
    fn nvim_win_dialog_changed(wp: WinHandle);

    /// Check if window's buffer has unsaved changes.
    fn nvim_win_bufIsChanged(wp: WinHandle) -> c_int;

    /// Check if window's buffer should be hidden.
    fn nvim_win_buf_hide(wp: WinHandle) -> c_int;

    /// Get p_confirm option.
    fn nvim_get_p_confirm() -> c_int;

    /// Check CMOD_CONFIRM flag.
    fn nvim_get_cmdmod_confirm() -> c_int;

    /// Get p_write option.
    fn nvim_get_p_write() -> c_int;

    /// Get autocmd_busy flag (defined in change_ffi.c, returns bool).
    fn nvim_get_autocmd_busy() -> bool;

    /// Generic error-message dispatcher.
    fn nvim_emsg_id(id: std::ffi::c_int);

    /// Display "Already only one window" message.
    fn nvim_msg_onlyone();

    /// Call rs_win_close(wp, free_buf, 0) directly.
    fn rs_win_close(wp: WinHandle, free_buf: c_int, force: c_int) -> c_int;
}

// =============================================================================
// EMSG IDs for nvim_emsg_id dispatcher
// =============================================================================

const EMSG_E445: std::ffi::c_int = 5;
const EMSG_E_FLOATONLY: std::ffi::c_int = 8;

// =============================================================================
// Implementation
// =============================================================================

/// Close all windows except curwin.
///
/// Port of C `close_others()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
unsafe fn close_others_impl(message: c_int, forceit: c_int) {
    let old_curwin = nvim_get_curwin();

    // If current window is floating, we can only close other floating windows.
    if nvim_win_get_floating(old_curwin) != 0 {
        if message != 0 && !nvim_get_autocmd_busy() {
            nvim_emsg_id(EMSG_E_FLOATONLY);
        }
        return;
    }

    // Check if already only one window (ignoring floating windows at the end).
    let firstwin = nvim_get_firstwin();
    let lastwin = nvim_get_lastwin();
    if rs_one_window_in_tab(firstwin, crate::TabpageHandle::null()) != 0
        && nvim_win_get_floating(lastwin) == 0
    {
        if message != 0 && !nvim_get_autocmd_busy() {
            nvim_msg_onlyone();
        }
        return;
    }

    // Be very careful here: autocommands may change the window layout.
    let mut wp = nvim_get_firstwin();
    while rs_win_valid(wp) != 0 {
        let nextwp = nvim_win_get_next(wp);

        // Autocommands messed curwin up: restore it.
        let curwin = nvim_get_curwin();
        if old_curwin != curwin && rs_win_valid(old_curwin) != 0 {
            nvim_set_curwin_from_wp(old_curwin);
        }

        let curwin = nvim_get_curwin();
        if wp == curwin {
            // Don't close current window.
            wp = nextwp;
            continue;
        }

        // Autocommands messed the buffer pointer up.
        let buf = nvim_win_get_buffer_raw(wp);
        if nvim_buf_valid_ptr(buf) == 0 && rs_win_valid(wp) != 0 {
            nvim_win_set_buffer_raw(wp, BufHandle::null());
            rs_win_close(wp, 0, 0);
            wp = nvim_get_firstwin();
            continue;
        }

        // Check if it's allowed to abandon this window.
        let r = nvim_win_can_abandon(wp, forceit);
        if rs_win_valid(wp) == 0 {
            // Autocommands messed wp up.
            wp = nvim_get_firstwin();
            continue;
        }
        if r == 0 {
            if message != 0
                && (nvim_get_p_confirm() != 0 || nvim_get_cmdmod_confirm() != 0)
                && nvim_get_p_write() != 0
            {
                nvim_win_dialog_changed(wp);
                if rs_win_valid(wp) == 0 {
                    // Autocommands messed wp up.
                    wp = nvim_get_firstwin();
                    continue;
                }
            }
            if nvim_win_bufIsChanged(wp) != 0 {
                wp = nextwp;
                continue;
            }
        }

        let free_buf = i32::from(nvim_win_buf_hide(wp) == 0 && nvim_win_bufIsChanged(wp) == 0);
        rs_win_close(wp, free_buf, 0);

        // After closing a window, restart from firstwin since autocommands
        // may have changed the layout.
        wp = nvim_get_firstwin();
    }

    // Emit E445 if there are still multiple windows with changes.
    let firstwin = nvim_get_firstwin();
    let lastwin = nvim_get_lastwin();
    let one_window = firstwin == lastwin;
    if message != 0 && !one_window {
        nvim_emsg_id(EMSG_E445);
    }
}

// =============================================================================
// FFI Export
// =============================================================================

/// FFI: Close all windows except curwin.
///
/// Replaces C `close_others()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_others(message: c_int, forceit: c_int) {
    close_others_impl(message, forceit);
}

/// C export: `close_others` — eliminates the C thin wrapper.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
#[unsafe(export_name = "close_others")]
pub unsafe extern "C" fn close_others(message: c_int, forceit: c_int) {
    close_others_impl(message, forceit);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // close_others logic requires live Neovim state
    }
}
