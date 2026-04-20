//! Orchestrator helpers for `win_close_othertab`.
//!
//! This module provides the consolidated `rs_win_close_othertab` entry point
//! as well as the three helper functions that remain exported for legacy callers:
//!
//! 1. [`rs_close_othertab_validate`] — initial validation (locked, aucmd, floating-only)
//! 2. [`rs_close_othertab_remove_tabpage`] — unlink tabpage from the list
//! 3. [`rs_close_othertab_leave_open`] — error-recovery buffer restore
//! 4. [`rs_win_close_othertab`] — consolidated entry point replacing the C body

use std::ffi::c_int;

use crate::win_struct::win_ref;
use crate::{BufHandle, TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    static mut redraw_tabline: bool;
}

extern "C" {
    // --- Accessors ---
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;
    fn nvim_tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    fn nvim_get_first_tabpage() -> TabpageHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn buf_valid(buf: BufHandle) -> bool;

    // --- Existing Rust FFI helpers ---
    #[link_name = "is_aucmd_win"]
    fn rs_is_aucmd_win(wp: WinHandle) -> c_int;
    fn rs_one_window_in_tab(win: WinHandle, tp: TabpageHandle) -> c_int;
    fn rs_win_valid_any_tab(win: WinHandle) -> c_int;
    fn rs_tabpage_index(ftp: TabpageHandle) -> c_int;
    fn rs_tabline_height() -> c_int;
    fn rs_valid_tabpage(tp: TabpageHandle) -> c_int;
    fn rs_tabpage_win_valid(tp: TabpageHandle, win: WinHandle) -> c_int;
    fn rs_do_autocmd_winclosed(win: WinHandle);

    // --- New Phase 2 wrappers ---
    fn nvim_emsg_id(id: c_int);
    fn nvim_internal_error_othertab();
    fn nvim_set_first_tabpage(tp: TabpageHandle);
    fn nvim_tabpage_set_next(tp: TabpageHandle, next: TabpageHandle);
    fn rs_win_new_screen_rows();
    #[link_name = "rs_can_close_floating_windows_tp"]
    fn nvim_can_close_floating_windows(tp: TabpageHandle) -> c_int;
    fn nvim_win_set_buffer_raw(wp: WinHandle, buf: BufHandle);
    fn nvim_buf_inc_nwindows(buf: BufHandle);
    fn rs_win_init_empty(wp: WinHandle);
    fn nvim_get_firstbuf_wrapper() -> BufHandle;
    fn rs_free_tabpage(tp: TabpageHandle);

    // --- Phase 11 new accessors ---
    fn close_buffer(
        win: WinHandle,
        buf: BufHandle,
        action: c_int,
        abort_if_last: bool,
        ignore_abort: bool,
    ) -> bool;
    fn nvim_apply_autocmds_tabclosed(idx_str: *const std::ffi::c_char, buf: BufHandle);
    fn nvim_has_event_tabclosed() -> c_int;
}

// =============================================================================
// EMSG IDs for nvim_emsg_id dispatcher
// =============================================================================

const EMSG_E_AUTOCMD_CLOSE: c_int = 10;
const EMSG_E_FLOATONLY: c_int = 8;
const DOBUF_UNLOAD: c_int = 2;

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
// Consolidated entry point: rs_win_close_othertab
// =============================================================================

/// Consolidated Rust replacement for C `win_close_othertab`.
///
/// Returns 1 (true) if window was closed, 0 (false) if not.
///
/// # Safety
///
/// `win` must be a valid `win_T*` in tabpage `tp`. `tp` must not be curtab.
#[no_mangle]
pub unsafe extern "C" fn rs_win_close_othertab(
    win: WinHandle,
    free_buf: c_int,
    tp: TabpageHandle,
    force: c_int,
) -> c_int {
    let mut did_decrement: c_int = 0;
    let mut bufref_buf: BufHandle = BufHandle::null();

    // Phase 1: Validation.
    let vrc = rs_close_othertab_validate(win, tp, force);
    if vrc == 1 {
        return 0; // locked or aucmd_win
    }
    if vrc == 2 {
        // Floating-only: recursively close floating windows.
        loop {
            let lastwin = nvim_tabpage_get_lastwin(tp);
            if c_int::from(win_ref(lastwin).w_floating) == 0 {
                break;
            }
            if rs_win_close_othertab(lastwin, free_buf, tp, 1) == 0 {
                // goto leave_open
                rs_close_othertab_leave_open(win, did_decrement, bufref_buf, 0);
                return 0;
            }
        }
        if rs_win_valid_any_tab(win) == 0 {
            return 0;
        }
    }
    if vrc == 3 {
        // e_floatonly already emitted by validate
        rs_close_othertab_leave_open(win, did_decrement, bufref_buf, 0);
        return 0;
    }

    // --- Autocmd section ---

    // Fire WinClosed before freeing window-related resources.
    if !BufHandle(win_ref(win).w_buffer).is_null() {
        rs_do_autocmd_winclosed(win);
        if rs_win_valid_any_tab(win) == 0 {
            return 0;
        }
    }

    // Save bufref before close_buffer (for error recovery).
    bufref_buf = BufHandle(win_ref(win).w_buffer);

    if !BufHandle(win_ref(win).w_buffer).is_null() {
        let buf = BufHandle(win_ref(win).w_buffer);
        did_decrement = c_int::from(close_buffer(
            win,
            buf,
            if free_buf != 0 { DOBUF_UNLOAD } else { 0 },
            false,
            true,
        ));
    }

    // Re-validate after autocmds.
    if rs_valid_tabpage(tp) == 0 || tp == nvim_get_curtab() {
        // goto leave_open
        let bufref_valid = if bufref_buf.is_null() {
            0
        } else {
            c_int::from(buf_valid(bufref_buf))
        };
        rs_close_othertab_leave_open(win, did_decrement, bufref_buf, bufref_valid);
        return 0;
    }
    if rs_tabpage_win_valid(tp, win) == 0 {
        let bufref_valid = if bufref_buf.is_null() {
            0
        } else {
            c_int::from(buf_valid(bufref_buf))
        };
        rs_close_othertab_leave_open(win, did_decrement, bufref_buf, bufref_valid);
        return 0;
    }
    let lastwin = nvim_tabpage_get_lastwin(tp);
    if win_ref(lastwin).w_floating && rs_one_window_in_tab(win, tp) != 0 {
        nvim_emsg_id(EMSG_E_FLOATONLY);
        let bufref_valid = if bufref_buf.is_null() {
            0
        } else {
            c_int::from(buf_valid(bufref_buf))
        };
        rs_close_othertab_leave_open(win, did_decrement, bufref_buf, bufref_valid);
        return 0;
    }

    // Phase 2: Tabpage removal.
    let res = rs_close_othertab_remove_tabpage(win, tp);

    // Free the window memory.
    let buf = BufHandle(win_ref(win).w_buffer);
    let mut dir: c_int = 0;
    crate::close::helpers::rs_win_free_mem(win, std::ptr::addr_of_mut!(dir), tp);

    if res.free_tp_idx > 0 {
        rs_free_tabpage(tp);

        if nvim_has_event_tabclosed() != 0 {
            // Format the tabpage index as a C string.
            let idx_str = format_tabpage_idx(res.free_tp_idx);
            nvim_apply_autocmds_tabclosed(idx_str.as_ptr(), buf);
        }
    }

    1 // true: window was closed
}

/// Format a tabpage index as a NUL-terminated C string.
/// Returns a small fixed-size buffer (same as C's NUMBUFLEN=20).
unsafe fn format_tabpage_idx(idx: c_int) -> [std::ffi::c_char; 20] {
    let mut buf = [0i8; 20];
    let s = format!("{idx}");
    for (i, b) in s.bytes().enumerate() {
        if i + 1 < buf.len() {
            // All digit characters are 0x30-0x39, safe to reinterpret as i8.
            buf[i] = i8::from_ne_bytes([b]);
        }
    }
    buf
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
        if win_ref(win).w_locked {
            return 1;
        }
        let buf = BufHandle(win_ref(win).w_buffer);
        if !buf.is_null() && nvim_buf_get_locked(buf) != 0 {
            return 1;
        }

        // Check aucmd_win.
        if rs_is_aucmd_win(win) != 0 {
            nvim_emsg_id(EMSG_E_AUTOCMD_CLOSE);
            return 1;
        }

        // Check if closing would leave only floating windows.
        let lastwin = nvim_tabpage_get_lastwin(tp);
        if win_ref(lastwin).w_floating && rs_one_window_in_tab(win, tp) != 0 {
            if force != 0 || nvim_can_close_floating_windows(tp) != 0 {
                // Signal caller to do recursive float close loop.
                return 2;
            }
            nvim_emsg_id(EMSG_E_FLOATONLY);
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

        redraw_tabline = true;
        if h != rs_tabline_height() {
            rs_win_new_screen_rows();
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

        let buf = BufHandle(win_ref(win).w_buffer);
        if buf.is_null() {
            // Buffer was removed from the window; give it any buffer.
            let firstbuf = nvim_get_firstbuf_wrapper();
            nvim_win_set_buffer_raw(win, firstbuf);
            nvim_buf_inc_nwindows(firstbuf);
            rs_win_init_empty(win);
        } else if did_decrement != 0 && buf == bufref_buf && bufref_valid != 0 {
            // close_buffer decremented b_nwindows but we're keeping the window.
            nvim_buf_inc_nwindows(buf);
        }
    }
}
