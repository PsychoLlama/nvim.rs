//! Implementation of `close_windows`.
//!
//! Closes all windows showing a specific buffer. Used from buffer.c when
//! unloading/deleting a buffer. Autocommands may change the window layout
//! during iteration, so we use a restart-from-lastwin pattern.

use std::ffi::c_int;

use crate::{BufHandle, TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get the last window in the current tab.
    fn nvim_get_lastwin() -> WinHandle;

    /// Get the current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Get the current tabpage.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get the first tabpage.
    fn nvim_get_first_tabpage() -> TabpageHandle;

    /// Get tp->tp_next.
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;

    /// Get tp->tp_lastwin.
    fn nvim_tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle;

    /// Get w_prev from a window.
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;

    /// Check if wp->w_buffer == buf.
    fn nvim_win_buffer_eq(wp: WinHandle, buf: BufHandle) -> c_int;

    /// Check if window is current window.
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;

    /// Check if window is locked (rs_win_locked).
    fn rs_win_locked(wp: WinHandle) -> c_int;

    /// Check if wp->w_buffer->b_locked > 0.
    fn nvim_win_buf_b_locked(wp: WinHandle) -> c_int;

    /// Check if window is an aucmd_win.
    fn nvim_is_aucmd_win(wp: WinHandle) -> c_int;

    /// Check if there is only one window in the tab.
    fn rs_one_window_in_tab(win: WinHandle, tp: TabpageHandle) -> c_int;

    /// Call win_close(wp, false, false) and return FAIL/OK.
    fn nvim_win_close_wrapper(wp: WinHandle, free_buf: c_int) -> c_int;

    /// Call win_close_othertab(wp, free_buf, tp, force) -> 0=FAIL, 1=OK.
    fn nvim_win_close_othertab_wrapper(
        wp: WinHandle,
        free_buf: c_int,
        tp: TabpageHandle,
        force: c_int,
    ) -> c_int;

    /// Increment RedrawingDisabled.
    fn nvim_inc_RedrawingDisabled();

    /// Decrement RedrawingDisabled.
    fn nvim_dec_RedrawingDisabled();
}

// =============================================================================
// Implementation
// =============================================================================

const FAIL: c_int = 0;

/// Close all windows showing buffer `buf`.
///
/// Port of C `close_windows()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
unsafe fn close_windows_impl(buf: BufHandle, keep_curwin: c_int) {
    nvim_inc_RedrawingDisabled();

    // Start from lastwin to close floating windows with the same buffer first.
    // When the autocommand window is involved win_close() may need to print an error message.
    let mut wp = nvim_get_lastwin();
    while !wp.is_null() {
        let lastwin = nvim_get_lastwin();
        let is_aucmd = nvim_is_aucmd_win(lastwin) != 0;
        let one_win = rs_one_window_in_tab(wp, TabpageHandle::null()) != 0;
        if !is_aucmd && one_win {
            break;
        }

        if nvim_win_buffer_eq(wp, buf) != 0
            && (keep_curwin == 0 || nvim_win_is_curwin(wp) == 0)
            && rs_win_locked(wp) == 0
            && nvim_win_buf_b_locked(wp) == 0
        {
            if nvim_win_close_wrapper(wp, 0) == FAIL {
                // If closing the window fails, give up to avoid looping forever.
                break;
            }
            // Start all over, autocommands may change the window layout.
            wp = nvim_get_lastwin();
        } else {
            wp = nvim_win_get_prev(wp);
        }
    }

    // Also check windows in other tab pages.
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        let nexttp = nvim_tabpage_get_next(tp);

        let curtab = nvim_get_curtab();
        if tp == curtab {
            tp = nexttp;
        } else {
            // Start from tp_lastwin to close floating windows with the same buffer first.
            let mut inner_wp = nvim_tabpage_get_lastwin(tp);
            let mut restarted = false;
            while !inner_wp.is_null() {
                if nvim_win_buffer_eq(inner_wp, buf) != 0
                    && rs_win_locked(inner_wp) == 0
                    && nvim_win_buf_b_locked(inner_wp) == 0
                {
                    if nvim_win_close_othertab_wrapper(inner_wp, 0, tp, 0) == 0 {
                        // If closing fails, give up to avoid looping forever.
                        break;
                    }
                    // Start all over, the tab page may be closed and
                    // autocommands may change the window layout.
                    // Set tp to first_tabpage so the outer loop restarts.
                    tp = nvim_get_first_tabpage();
                    restarted = true;
                    break;
                }
                inner_wp = nvim_win_get_prev(inner_wp);
            }
            if !restarted {
                tp = nexttp;
            }
        }
    }

    nvim_dec_RedrawingDisabled();
}

// =============================================================================
// FFI Export
// =============================================================================

/// FFI: Close all windows showing buffer `buf`.
///
/// Replaces C `close_windows()`.
///
/// # Safety
/// Accesses global Neovim state via C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_windows(buf: BufHandle, keep_curwin: c_int) {
    close_windows_impl(buf, keep_curwin);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // close_windows logic requires live Neovim state
    }
}
