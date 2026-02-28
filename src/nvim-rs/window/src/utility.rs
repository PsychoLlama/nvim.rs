//! Utility and validation helper functions migrated from window_shim.c (Phase 1).
//!
//! This module contains small predicate/utility functions that were previously
//! implemented in C:
//! - `check_can_set_curbuf_disabled` / `check_can_set_curbuf_forceit`: winfixbuf guards
//! - `prevwin_curwin`: returns prevwin or curwin based on cmdwin state
//! - `check_split_disallowed`: checks split_disallowed counter and buffer lock
//! - `get_maximum_wincount`: computes max windows fitting in a frame height
//! - `make_windows`: creates N split windows using existing rs_* helpers

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin->w_p_wfb (winfixbuf option).
    fn nvim_get_curwin_p_wfb() -> c_int;

    /// Generic error message dispatcher.
    fn nvim_emsg_id(id: c_int);

    /// Get the split_disallowed counter.
    fn nvim_get_split_disallowed() -> c_int;

    /// Get wp->w_buffer->b_locked_split.
    fn nvim_win_buf_locked_split(wp: WinHandle) -> c_int;

    /// Get the curwin global.
    fn nvim_get_curwin() -> WinHandle;

    /// Get the prevwin global.
    fn nvim_get_prevwin() -> WinHandle;

    /// Check if we are in the command-line window (rs_is_in_cmdwin).
    #[link_name = "rs_is_in_cmdwin"]
    fn is_in_cmdwin() -> c_int;

    /// Get p_wmh (winminheight option).
    fn nvim_get_p_wmh() -> i64;

    /// Get w_winbar_height from a window.
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    /// Get frame2win result (leaf window for a frame).
    #[link_name = "rs_frame2win"]
    fn frame2win(frp: *mut Frame) -> WinHandle;

    /// Get rs_global_winbar_height.
    #[link_name = "rs_global_winbar_height"]
    fn global_winbar_height() -> c_int;

    /// rs_split_max_windows: max windows for make_windows.
    fn rs_split_max_windows(vertical: c_int) -> c_int;

    /// rs_last_status: add/remove last status line.
    fn rs_last_status(morewin: c_int);

    /// rs_split_make_windows_flags: flags for make_windows splits.
    fn rs_split_make_windows_flags(vertical: c_int) -> c_int;

    /// rs_split_iteration_size: split size for each step.
    fn rs_split_iteration_size(vertical: c_int, todo: c_int) -> c_int;

    /// nvim_win_split_wrapper: call win_split(size, flags).
    fn rs_win_split(size: c_int, flags: c_int) -> c_int;

    /// block_autocmds wrapper.
    fn nvim_block_autocmds();

    /// unblock_autocmds wrapper.
    fn nvim_unblock_autocmds();

    /// api_set_error for E242 (split while closing another).
    fn nvim_api_set_err_e242(err: *mut std::ffi::c_void);

    /// api_set_error for e_cannot_split_window_when_closing_buffer.
    fn nvim_api_set_err_cannot_split_closing(err: *mut std::ffi::c_void);
}

// OK / FAIL constants (matching C values from vim_defs.h)
const OK: c_int = 1;
const FAIL: c_int = 0;

// EMSG IDs for nvim_emsg_id dispatcher
const EMSG_E_WINFIXBUF: c_int = 11;
const EMSG_E242: c_int = 4;
const EMSG_E_CANNOT_SPLIT: c_int = 12;

// STATUS_HEIGHT constant
const STATUS_HEIGHT: c_int = 1;

// =============================================================================
// check_can_set_curbuf_disabled
// =============================================================================

/// Check if the current window is allowed to move to a different buffer.
///
/// Returns false (and emits an error) if the window has 'winfixbuf' set.
/// This is the Rust equivalent of `check_can_set_curbuf_disabled()`.
fn check_can_set_curbuf_disabled_impl() -> bool {
    unsafe {
        if nvim_get_curwin_p_wfb() != 0 {
            nvim_emsg_id(EMSG_E_WINFIXBUF);
            return false;
        }
        true
    }
}

/// FFI export for `check_can_set_curbuf_disabled`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_check_can_set_curbuf_disabled() -> c_int {
    c_int::from(check_can_set_curbuf_disabled_impl())
}

// =============================================================================
// check_can_set_curbuf_forceit
// =============================================================================

/// Check if the current window is allowed to move to a different buffer.
///
/// If `forceit` is non-zero, the winfixbuf check is bypassed.
/// Returns false (and emits an error) if `forceit` is zero and 'winfixbuf' is set.
/// This is the Rust equivalent of `check_can_set_curbuf_forceit()`.
fn check_can_set_curbuf_forceit_impl(forceit: c_int) -> bool {
    unsafe {
        if forceit == 0 && nvim_get_curwin_p_wfb() != 0 {
            nvim_emsg_id(EMSG_E_WINFIXBUF);
            return false;
        }
        true
    }
}

/// FFI export for `check_can_set_curbuf_forceit`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_check_can_set_curbuf_forceit(forceit: c_int) -> c_int {
    c_int::from(check_can_set_curbuf_forceit_impl(forceit))
}

// =============================================================================
// prevwin_curwin
// =============================================================================

/// Return the current window, unless in the cmdline window and prevwin is set,
/// in which case return prevwin.
///
/// This is the Rust equivalent of `prevwin_curwin()`.
fn prevwin_curwin_impl() -> WinHandle {
    unsafe {
        let prevwin = nvim_get_prevwin();
        if is_in_cmdwin() != 0 && !prevwin.is_null() {
            prevwin
        } else {
            nvim_get_curwin()
        }
    }
}

/// FFI export for `prevwin_curwin`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_prevwin_curwin() -> WinHandle {
    prevwin_curwin_impl()
}

/// C export: `prevwin_curwin` — eliminates the C thin wrapper.
#[unsafe(export_name = "prevwin_curwin")]
pub extern "C" fn prevwin_curwin() -> WinHandle {
    prevwin_curwin_impl()
}

// =============================================================================
// check_split_disallowed
// =============================================================================

/// Check if splitting is disallowed.
///
/// Returns FAIL and emits an error if:
/// - `split_disallowed` counter is positive (another window is being closed), or
/// - the window's buffer has `b_locked_split` set.
///
/// This is the Rust equivalent of `check_split_disallowed()`.
fn check_split_disallowed_impl(wp: WinHandle) -> bool {
    unsafe {
        if nvim_get_split_disallowed() > 0 {
            nvim_emsg_id(EMSG_E242);
            return false;
        }
        if !wp.is_null() && nvim_win_buf_locked_split(wp) != 0 {
            nvim_emsg_id(EMSG_E_CANNOT_SPLIT);
            return false;
        }
        true
    }
}

/// FFI export for `check_split_disallowed`.
///
/// Returns OK (1) if split is allowed, FAIL (0) otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn rs_check_split_disallowed(wp: WinHandle) -> c_int {
    if check_split_disallowed_impl(wp) {
        OK
    } else {
        FAIL
    }
}

/// C export: `check_split_disallowed` — eliminates the C thin wrapper.
///
/// Returns OK (1) if split is allowed, FAIL (0) otherwise.
#[unsafe(export_name = "check_split_disallowed")]
pub extern "C" fn check_split_disallowed(wp: WinHandle) -> c_int {
    if check_split_disallowed_impl(wp) {
        OK
    } else {
        FAIL
    }
}

// =============================================================================
// get_maximum_wincount
// =============================================================================

/// Compute the maximum number of windows that can fit within `height` in frame `fr`.
///
/// This is the Rust equivalent of the static `get_maximum_wincount()` in window_shim.c.
///
/// # Safety
/// Caller must ensure `fr` is a valid non-null Frame pointer.
unsafe fn get_maximum_wincount_impl(fr: *const Frame, height: c_int) -> c_int {
    if fr.is_null() {
        return 0;
    }

    let frame = &*fr;
    let p_wmh = nvim_get_p_wmh() as c_int;

    if frame.fr_layout != FR_COL {
        // Not a column: divide available height by (winminheight + status + winbar)
        let win = frame2win(fr.cast_mut());
        let winbar = if win.is_null() {
            0
        } else {
            nvim_win_get_winbar_height(win)
        };
        return height / (p_wmh + STATUS_HEIGHT + winbar);
    }

    if global_winbar_height() != 0 {
        // Winbar is globally enabled — all windows have it, use constant height
        return height / (p_wmh + STATUS_HEIGHT + 1);
    }

    // FR_COL: iterate children, fit as many as possible accounting for per-window winbar
    let mut remaining = height;
    let mut total_wincount = 0;

    let mut frp = frame.fr_child;
    while !frp.is_null() {
        let wp = frame2win(frp);
        let winbar = if wp.is_null() {
            0
        } else {
            nvim_win_get_winbar_height(wp)
        };

        if remaining < p_wmh + STATUS_HEIGHT + winbar {
            break;
        }
        remaining -= p_wmh + STATUS_HEIGHT + winbar;
        total_wincount += 1;

        frp = (*frp).fr_next;
    }

    // Any remaining height can fit windows without winbar (default 0)
    total_wincount += remaining / (p_wmh + STATUS_HEIGHT);

    total_wincount
}

/// FFI export for `get_maximum_wincount`.
///
/// # Safety
/// Caller must ensure `fr` is null or a valid Frame pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_maximum_wincount(fr: *const Frame, height: c_int) -> c_int {
    if fr.is_null() {
        return 0;
    }
    get_maximum_wincount_impl(fr, height)
}

// =============================================================================
// check_split_disallowed_err (Phase 2)
// =============================================================================

/// Check if split is disallowed, setting the API error struct on failure.
///
/// Returns true if split is allowed, false otherwise.
/// Equivalent to C `check_split_disallowed_err()`.
///
/// # Safety
/// `err` must be a valid pointer to a C `Error` struct, or null.
unsafe fn check_split_disallowed_err_impl(wp: WinHandle, err: *mut std::ffi::c_void) -> bool {
    if nvim_get_split_disallowed() > 0 {
        nvim_api_set_err_e242(err);
        return false;
    }
    if !wp.is_null() && nvim_win_buf_locked_split(wp) != 0 {
        nvim_api_set_err_cannot_split_closing(err);
        return false;
    }
    true
}

/// FFI export for `check_split_disallowed_err`.
///
/// Returns 1 if split is allowed, 0 otherwise. Sets the API error on failure.
///
/// # Safety
/// `err` must be a valid pointer to a C `Error` struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_check_split_disallowed_err(
    wp: WinHandle,
    err: *mut std::ffi::c_void,
) -> c_int {
    c_int::from(check_split_disallowed_err_impl(wp, err))
}

// =============================================================================
// make_windows
// =============================================================================

/// Create `count` windows on screen by splitting. Must be called when there is
/// just one window filling the whole screen.
///
/// Returns the actual number of windows created.
///
/// This is the Rust equivalent of `make_windows()`.
fn make_windows_impl(count: c_int, vertical: c_int) -> c_int {
    unsafe {
        // Calculate maximum number of windows
        let maxcount = rs_split_max_windows(vertical);
        let count = count.min(maxcount);

        // Add status line now, otherwise first window will be too big
        if count > 1 {
            rs_last_status(1);
        }

        // Don't execute autocommands while creating the windows.
        nvim_block_autocmds();

        let flags = rs_split_make_windows_flags(vertical);
        let mut todo = count - 1;

        while todo > 0 {
            let split_size = rs_split_iteration_size(vertical, todo);
            if rs_win_split(split_size, flags) == FAIL {
                break;
            }
            todo -= 1;
        }

        nvim_unblock_autocmds();

        // Return actual number of windows created
        count - todo
    }
}

/// FFI export for `make_windows`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_make_windows(count: c_int, vertical: c_int) -> c_int {
    make_windows_impl(count, vertical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_fail_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
    }

    #[test]
    fn test_status_height() {
        assert_eq!(STATUS_HEIGHT, 1);
    }
}
