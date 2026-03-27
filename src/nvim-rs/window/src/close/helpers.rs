//! Helper functions for window close operations.
//!
//! This module contains Rust implementations of C helper functions that
//! were previously static in `window.c` / `window_shim.c`.
//!
//! # Functions
//!
//! - [`rs_win_close_buffer`]: Close the buffer of a window, handling
//!   synblock, quickfix unlisting, and bufref validity.
//! - [`rs_close_last_window_tabpage`]: Close the last window in a tabpage.
//! - [`rs_win_free_mem`]: Free window memory (frame, float alt-win, win_free).

use std::ffi::c_int;

use crate::{BufHandle, Frame, TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // --- Accessors ---
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn nvim_buf_set_p_bl(buf: BufHandle, val: c_int);
    fn nvim_get_firstbuf_wrapper() -> BufHandle;
    fn nvim_set_curbuf(buf: BufHandle);
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_lastwin() -> WinHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_curwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_set_curwin(tp: TabpageHandle, wp: WinHandle);
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    // --- Logic helpers ---
    #[link_name = "reset_synblock"]
    fn rs_reset_synblock_c(wp: WinHandle);
    #[link_name = "rs_bt_quickfix"]
    fn rs_bt_quickfix_c(buf: BufHandle) -> bool;
    #[link_name = "rs_valid_tabpage"]
    fn rs_valid_tabpage_c(tp: TabpageHandle) -> c_int;
    #[link_name = "rs_winframe_remove"]
    fn rs_winframe_remove_c(
        win: WinHandle,
        dirp: *mut c_int,
        tp: TabpageHandle,
        unflat_altfr: *mut *mut Frame,
    ) -> WinHandle;

    /// Compound: close_buffer with bufref guard; returns 1 if curbuf invalid.
    fn nvim_close_buffer_for_win(win: WinHandle, action: c_int, abort_if_last: c_int) -> c_int;

    // --- close_last_window_tabpage helpers ---
    /// Safe terminal check: 1 if w_buffer != NULL && w_buffer->terminal != NULL.
    fn nvim_win_buf_has_terminal_safe(win: WinHandle) -> c_int;
    /// Call rs_win_close_othertab directly.
    fn rs_win_close_othertab(
        wp: WinHandle,
        free_buf: c_int,
        tp: TabpageHandle,
        force: c_int,
    ) -> c_int;
    /// entering_window(curwin) wrapper (= rs_entering_window).
    #[link_name = "rs_entering_window"]
    fn nvim_entering_window_c(win: WinHandle);
    /// Generic autocmd dispatcher.
    fn nvim_apply_autocmds_event(event: std::ffi::c_int);
    /// apply_autocmds(EVENT_BUFENTER) if old_curbuf != curbuf.
    fn nvim_apply_autocmds_bufenter_if_changed(old_curbuf: BufHandle);

    // --- win_free_mem helpers ---
    fn win_float_find_altwin(win: WinHandle, tp: TabpageHandle) -> WinHandle;
    /// xfree for frame_T*.
    fn nvim_xfree_frame(frp: *mut std::ffi::c_void);
    /// win_free(win, tp) -- direct Rust call.
    fn rs_win_free(win: WinHandle, tp: TabpageHandle);
    /// Check if win == cmdline_win; returns 1 if so.
    fn nvim_win_is_cmdline_win(win: WinHandle) -> c_int;
    /// Set cmdline_win = NULL.
    fn nvim_set_cmdline_win_null();
    /// get curwin.
    fn nvim_get_curwin() -> WinHandle;
}

// =============================================================================
// Constants
// =============================================================================

/// DOBUF_UNLOAD from buffer.h.
const DOBUF_UNLOAD: c_int = 2;

/// EVENT_* constants matching auevents_enum.generated.h
const EVENT_TABENTER: c_int = 110;
const EVENT_WINENTER: c_int = 136;

// =============================================================================
// Rust implementation of win_close_buffer
// =============================================================================

/// Rust implementation of `win_close_buffer`.
///
/// Closes the buffer of `win`:
/// - Frees independent synblock if buffer exists.
/// - Unlists quickfix buffer if it's only open in one window.
/// - Calls `close_buffer` via compound wrapper (handles bufref guard).
/// - Resets `curbuf` to `firstbuf` if it became invalid.
///
/// # Safety
///
/// `win` must be a valid `win_T*`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_close_buffer(win: WinHandle, action: c_int, abort_if_last: c_int) {
    let buf = nvim_win_get_buffer(win);
    if buf.is_null() {
        return;
    }

    // Free independent synblock before the buffer is freed.
    rs_reset_synblock_c(win);

    // When a quickfix/location list window is closed and the buffer is
    // displayed in only one window, then unlist the buffer.
    if rs_bt_quickfix_c(buf) && nvim_buf_get_nwindows(buf) == 1 {
        nvim_buf_set_p_bl(buf, 0);
    }

    // Close the link to the buffer (compound C wrapper handles bufref).
    let curbuf_invalid = nvim_close_buffer_for_win(win, action, abort_if_last);

    // Make sure curbuf is valid. It can become invalid if 'bufhidden' is "wipe".
    if curbuf_invalid != 0 {
        nvim_set_curbuf(nvim_get_firstbuf_wrapper());
    }
}

// =============================================================================
// Rust implementation of close_last_window_tabpage
// =============================================================================

/// Rust implementation of `close_last_window_tabpage`.
///
/// Returns 1 if handled (last window in tab), 0 if not.
/// When returning 1, the caller should `return FAIL`.
///
/// # Safety
///
/// `win` must be a valid `win_T*`. `prev_curtab` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_last_window_tabpage(
    win: WinHandle,
    free_buf: c_int,
    prev_curtab: TabpageHandle,
) -> c_int {
    // ONE_WINDOW = firstwin == lastwin.
    if nvim_get_firstwin() != nvim_get_lastwin() {
        return 0;
    }

    let old_curbuf = nvim_get_curbuf();

    // Don't free terminal buffers.
    let do_free_buf = if nvim_win_buf_has_terminal_safe(win) != 0 {
        0
    } else {
        free_buf
    };

    // Go to another tab page first, then close the window and tab.
    // Don't trigger *Enter autocommands yet.
    // Do trigger *Leave autocommands unless win->w_buffer is NULL.
    let buf = nvim_win_get_buffer(win);
    let trigger_leave = c_int::from(!buf.is_null());
    crate::tabpage::rs_goto_tabpage_tp_impl(
        crate::tabpage::rs_alt_tabpage(),
        0, // trigger_enter = false
        trigger_leave,
    );

    // Safety check: autocmds may have switched back or closed the window.
    let curtab = nvim_get_curtab();
    if curtab != prev_curtab
        && rs_valid_tabpage_c(prev_curtab) != 0
        && nvim_tabpage_get_firstwin(prev_curtab) == win
    {
        rs_win_close_othertab(win, do_free_buf, prev_curtab, 0);
    }

    // Now trigger *Enter autocommands.
    let curwin = nvim_get_curwin();
    nvim_entering_window_c(curwin);

    nvim_apply_autocmds_event(EVENT_WINENTER);
    nvim_apply_autocmds_event(EVENT_TABENTER);
    nvim_apply_autocmds_bufenter_if_changed(old_curbuf);

    1
}

// =============================================================================
// Rust implementation of win_free_mem
// =============================================================================

/// Rust implementation of `win_free_mem`.
///
/// Frees the memory used by `win`:
/// - Non-floating: removes frame, frees frame struct.
/// - Floating: finds alt window.
/// - Calls `win_free`.
/// - Updates `tp_curwin` if needed.
/// - Clears `cmdline_win` if it was `win`.
///
/// Returns the window that got the freed space.
///
/// # Safety
///
/// `win` must be a valid `win_T*`. `dirp` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_free_mem(
    win: WinHandle,
    dirp: *mut c_int,
    tp: TabpageHandle,
) -> WinHandle {
    let win_tp = if tp.is_null() { nvim_get_curtab() } else { tp };

    let wp;
    if nvim_win_get_floating(win) == 0 {
        // Remove the window and its frame from the tree of frames.
        let frp = nvim_win_get_frame(win);
        wp = rs_winframe_remove_c(win, dirp, tp, std::ptr::null_mut());
        if !frp.is_null() {
            nvim_xfree_frame(frp.cast());
        }
    } else {
        *dirp = c_int::from(b'h'); // Dummy value.
        wp = win_float_find_altwin(win, tp);
    }

    rs_win_free(win, tp);

    // When deleting the current window in the tab, select a new current window.
    if nvim_tabpage_get_curwin(win_tp) == win {
        nvim_tabpage_set_curwin(win_tp, wp);
    }

    // Avoid executing cmdline_win logic after it is closed.
    if nvim_win_is_cmdline_win(win) != 0 {
        nvim_set_cmdline_win_null();
    }

    wp
}
