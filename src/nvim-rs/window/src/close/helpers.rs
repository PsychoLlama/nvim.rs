//! Helper functions for window close operations.
//!
//! This module contains Rust implementations of C helper functions that
//! were previously static in `window.c` / `window_shim.c`.
//!
//! # Functions
//!
//! - [`rs_win_close_buffer`]: Close the buffer of a window, handling
//!   synblock, quickfix unlisting, and bufref validity.

use std::ffi::c_int;

use crate::{BufHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // --- Accessors ---
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_locked(wp: WinHandle) -> c_int;
    fn nvim_win_set_locked(wp: WinHandle, val: c_int);
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn nvim_buf_set_p_bl(buf: BufHandle, val: c_int);
    fn nvim_get_firstbuf_wrapper() -> BufHandle;
    fn nvim_set_curbuf(buf: BufHandle);

    // --- Logic helpers ---
    #[link_name = "rs_reset_synblock"]
    fn rs_reset_synblock_c(wp: WinHandle);
    #[link_name = "rs_bt_quickfix"]
    fn rs_bt_quickfix_c(buf: BufHandle) -> c_int;
    #[link_name = "rs_win_valid_any_tab"]
    fn rs_win_valid_any_tab_c(win: WinHandle) -> c_int;

    /// Compound: close_buffer with bufref guard; returns 1 if curbuf invalid.
    fn nvim_close_buffer_for_win(win: WinHandle, action: c_int, abort_if_last: c_int) -> c_int;
}

// =============================================================================
// Constants
// =============================================================================

/// DOBUF_UNLOAD from buffer.h.
const DOBUF_UNLOAD: c_int = 2;

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
#[no_mangle]
pub extern "C" fn rs_win_close_buffer(win: WinHandle, action: c_int, abort_if_last: c_int) {
    unsafe {
        let buf = nvim_win_get_buffer(win);
        if buf.is_null() {
            return;
        }

        // Free independent synblock before the buffer is freed.
        rs_reset_synblock_c(win);

        // When a quickfix/location list window is closed and the buffer is
        // displayed in only one window, then unlist the buffer.
        if rs_bt_quickfix_c(buf) != 0 && nvim_buf_get_nwindows(buf) == 1 {
            nvim_buf_set_p_bl(buf, 0);
        }

        // Close the link to the buffer (compound C wrapper handles bufref).
        let curbuf_invalid = nvim_close_buffer_for_win(win, action, abort_if_last);

        // Make sure curbuf is valid. It can become invalid if 'bufhidden' is "wipe".
        if curbuf_invalid != 0 {
            nvim_set_curbuf(nvim_get_firstbuf_wrapper());
        }
    }
}
