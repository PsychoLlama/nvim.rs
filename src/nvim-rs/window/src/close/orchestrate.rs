//! Orchestrator helpers for `win_close_othertab`.
//!
//! The C function remains the orchestrator (it handles autocmd triggers,
//! `close_buffer`, and re-validations), but delegates three chunks of pure
//! structural logic to Rust:
//!
//! 1. [`rs_close_othertab_validate`] — initial validation (locked, aucmd, floating-only)
//! 2. [`rs_close_othertab_remove_tabpage`] — unlink tabpage from the list
//! 3. [`rs_close_othertab_leave_open`] — error-recovery buffer restore

use std::ffi::c_int;

use crate::{BufHandle, TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // --- Accessors ---
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_locked(wp: WinHandle) -> c_int;
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;
    fn nvim_tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    fn nvim_get_first_tabpage() -> TabpageHandle;

    // --- Existing Rust FFI helpers ---
    fn rs_is_aucmd_win(wp: WinHandle) -> c_int;
    fn rs_one_window_in_tab(win: WinHandle, tp: TabpageHandle) -> c_int;
    fn rs_win_valid_any_tab(win: WinHandle) -> c_int;
    fn rs_tabpage_index(ftp: TabpageHandle) -> c_int;
    fn rs_tabline_height() -> c_int;

    // --- New Phase 2 wrappers ---
    fn nvim_emsg_e_autocmd_close();
    fn nvim_emsg_e_floatonly();
    fn nvim_internal_error_othertab();
    fn nvim_set_first_tabpage(tp: TabpageHandle);
    fn nvim_tabpage_set_next(tp: TabpageHandle, next: TabpageHandle);
    fn nvim_set_redraw_tabline(val: c_int);
    fn nvim_win_new_screen_rows_wrapper();
    fn nvim_can_close_floating_windows(tp: TabpageHandle) -> c_int;
    fn nvim_win_set_buffer_raw(wp: WinHandle, buf: BufHandle);
    fn nvim_buf_inc_nwindows(buf: BufHandle);
    fn nvim_win_init_empty_wrapper(wp: WinHandle);
    fn nvim_get_firstbuf_wrapper() -> BufHandle;
}

// =============================================================================
// Return type
// =============================================================================

/// Result of [`rs_close_othertab_remove_tabpage`].
#[repr(C)]
pub struct TabRemoveResult {
    /// Tabpage index if we removed a tabpage (>0), 0 otherwise.
    pub free_tp_idx: c_int,
}

// =============================================================================
// Helper 1: Validation
// =============================================================================

/// Validate whether `win` in tabpage `tp` can be closed.
///
/// Returns:
/// - `0`: validation passed, proceed with close
/// - `1`: cannot close (locked / aucmd_win)
/// - `2`: floating-only, caller should do recursive close loop
/// - `3`: floating-only, caller should goto leave_open (e_floatonly emitted)
///
/// # Safety
///
/// `win` must be a valid `win_T*` in tabpage `tp`. `tp` must not be curtab.
#[no_mangle]
pub extern "C" fn rs_close_othertab_validate(
    win: WinHandle,
    tp: TabpageHandle,
    force: c_int,
) -> c_int {
    unsafe {
        // Check win_locked or buffer locked.
        if nvim_win_get_locked(win) != 0 {
            return 1;
        }
        let buf = nvim_win_get_buffer(win);
        if !buf.is_null() && nvim_buf_get_locked(buf) != 0 {
            return 1;
        }

        // Check aucmd_win.
        if rs_is_aucmd_win(win) != 0 {
            nvim_emsg_e_autocmd_close();
            return 1;
        }

        // Check if closing would leave only floating windows.
        let lastwin = nvim_tabpage_get_lastwin(tp);
        if nvim_win_get_floating(lastwin) != 0 && rs_one_window_in_tab(win, tp) != 0 {
            if force != 0 || nvim_can_close_floating_windows(tp) != 0 {
                // Signal caller to do recursive float close loop.
                return 2;
            }
            nvim_emsg_e_floatonly();
            return 3;
        }

        0 // all good
    }
}

// =============================================================================
// Helper 2: Remove tabpage from the linked list
// =============================================================================

/// If `win` is the last window in `tp`, unlink `tp` from `first_tabpage` list
/// and update screen rows. Otherwise do nothing.
///
/// Returns a [`TabRemoveResult`] with `free_tp_idx > 0` if the tabpage was
/// removed (the caller must still call `free_tabpage` and fire `TabClosed`).
///
/// # Safety
///
/// `win` and `tp` must be valid and `tp` must not be curtab.
#[no_mangle]
pub extern "C" fn rs_close_othertab_remove_tabpage(
    _win: WinHandle,
    tp: TabpageHandle,
) -> TabRemoveResult {
    unsafe {
        let firstwin = nvim_tabpage_get_firstwin(tp);
        let lastwin = nvim_tabpage_get_lastwin(tp);

        // Only remove tabpage if win is the sole window.
        if firstwin != lastwin {
            return TabRemoveResult { free_tp_idx: 0 };
        }

        let free_tp_idx = rs_tabpage_index(tp);
        let h = rs_tabline_height();

        // Unlink tp from the tabpage list.
        let first = nvim_get_first_tabpage();
        if first == tp {
            let next = nvim_tabpage_get_next(tp);
            nvim_set_first_tabpage(next);
        } else {
            let mut ptp = first;
            loop {
                if ptp.is_null() {
                    nvim_internal_error_othertab();
                    return TabRemoveResult { free_tp_idx: 0 };
                }
                let next = nvim_tabpage_get_next(ptp);
                if next == tp {
                    break;
                }
                ptp = next;
            }
            let tp_next = nvim_tabpage_get_next(tp);
            nvim_tabpage_set_next(ptp, tp_next);
        }

        nvim_set_redraw_tabline(1);
        if h != rs_tabline_height() {
            nvim_win_new_screen_rows_wrapper();
        }

        TabRemoveResult { free_tp_idx }
    }
}

// =============================================================================
// Helper 3: Error recovery (leave_open)
// =============================================================================

/// Restore the window's buffer association when a close is aborted.
///
/// - If the buffer was already removed (`win->w_buffer == NULL`), assign
///   `firstbuf` and increment its `b_nwindows`.
/// - If `close_buffer` decremented `b_nwindows` but we're keeping the window,
///   re-increment it.
///
/// # Safety
///
/// `win` must be a valid `win_T*` (checked via `rs_win_valid_any_tab` first in C).
#[no_mangle]
pub extern "C" fn rs_close_othertab_leave_open(
    win: WinHandle,
    did_decrement: c_int,
    bufref_buf: BufHandle,
    bufref_valid: c_int,
) {
    unsafe {
        if rs_win_valid_any_tab(win) == 0 {
            return;
        }

        let buf = nvim_win_get_buffer(win);
        if buf.is_null() {
            // Buffer was removed from the window; give it any buffer.
            let firstbuf = nvim_get_firstbuf_wrapper();
            nvim_win_set_buffer_raw(win, firstbuf);
            nvim_buf_inc_nwindows(firstbuf);
            nvim_win_init_empty_wrapper(win);
        } else if did_decrement != 0 && buf == bufref_buf && bufref_valid != 0 {
            // close_buffer decremented b_nwindows but we're keeping the window.
            nvim_buf_inc_nwindows(buf);
        }
    }
}
