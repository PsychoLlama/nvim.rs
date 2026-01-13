//! Window close validation functions.
//!
//! This module provides validation functions for window close operations,
//! checking whether closes are allowed and various window state queries.

use std::ffi::c_int;

use crate::{BufHandle, TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get firstwin.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get lastwin.
    fn nvim_get_lastwin() -> WinHandle;

    /// Get curtab.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get first tabpage.
    fn nvim_get_first_tabpage() -> TabpageHandle;

    /// Get next tabpage.
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;

    /// Get tp_firstwin from tabpage.
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    /// Get w_next from window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get w_prev from window.
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get w_buffer from window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get w_locked from window.
    fn nvim_win_get_locked(wp: WinHandle) -> c_int;

    /// Get b_locked from buffer.
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;

    /// Get b_nwindows from buffer.
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;

    /// Check if buffer has changes.
    fn bufIsChanged(buf: BufHandle) -> c_int;

    /// Check if buffer can be hidden.
    fn buf_hide(buf: BufHandle) -> c_int;

    /// Check if window is an aucmd_win.
    fn rs_is_aucmd_win(wp: WinHandle) -> c_int;

    /// Check cmdwin_type global.
    fn nvim_get_cmdwin_type() -> c_int;
}

// =============================================================================
// Window Close Validation
// =============================================================================

/// Check if a window is locked (cannot be closed).
fn win_locked_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    unsafe { nvim_win_get_locked(wp) != 0 }
}

/// Check if a window's buffer is locked.
fn buf_locked_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    unsafe {
        let buf = nvim_win_get_buffer(wp);
        if buf.is_null() {
            return false;
        }
        nvim_buf_get_locked(buf) > 0
    }
}

/// Check if a window or its buffer is locked.
fn win_or_buf_locked_impl(wp: WinHandle) -> bool {
    win_locked_impl(wp) || buf_locked_impl(wp)
}

/// Check if we're currently in cmdwin mode.
///
/// Returns 1 if in cmdwin, 0 otherwise.
/// The detailed cmdwin window check (which window is cmdwin) must be done in C.
fn in_cmdwin_impl() -> bool {
    unsafe { nvim_get_cmdwin_type() != 0 }
}

/// Check if floating windows in the current tabpage can be closed.
///
/// Returns true if all floating windows can be safely closed (their buffers
/// can be hidden or are unchanged).
///
/// Note: Only works for current tabpage (uses lastwin global).
fn can_close_floating_windows_impl() -> bool {
    unsafe {
        let lastwin = nvim_get_lastwin();

        if lastwin.is_null() {
            return true;
        }

        // Iterate backwards through floating windows
        let mut wp = lastwin;
        while !wp.is_null() && nvim_win_get_floating(wp) != 0 {
            let buf = nvim_win_get_buffer(wp);
            if !buf.is_null() {
                let is_changed = bufIsChanged(buf) != 0;
                let nwindows = nvim_buf_get_nwindows(buf);

                // Buffer needs hiding if changed and this is the only window
                let need_hide = is_changed && nwindows <= 1;

                // If buffer needs hiding but can't be hidden, we can't close
                if need_hide && buf_hide(buf) == 0 {
                    return false;
                }
            }
            wp = nvim_win_get_prev(wp);
        }
        true
    }
}

/// Check if window is the only non-floating window in a tabpage.
fn is_only_nonfloating_impl(wp: WinHandle, tp: TabpageHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        // Get first window in tabpage
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        // Window must be first
        if first != wp {
            return false;
        }

        // Next window must be null or floating
        let next = nvim_win_get_next(wp);
        next.is_null() || nvim_win_get_floating(next) != 0
    }
}

/// Check if there's only one tabpage.
fn only_one_tabpage_impl() -> bool {
    unsafe {
        let first = nvim_get_first_tabpage();
        nvim_tabpage_get_next(first).is_null()
    }
}

/// Check if this is the last window that can be closed.
///
/// Returns true if this is the only non-floating window in the only tabpage.
fn is_last_closable_window_impl(wp: WinHandle) -> bool {
    is_only_nonfloating_impl(wp, unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) })
        && only_one_tabpage_impl()
}

/// Check if closing this window would leave only floating windows.
///
/// Note: Only works for current tabpage (uses lastwin global).
fn would_leave_only_floating_impl(wp: WinHandle) -> bool {
    unsafe {
        let lastwin = nvim_get_lastwin();

        // If lastwin is floating and this is the only non-floating window
        nvim_win_get_floating(lastwin) != 0
            && is_only_nonfloating_impl(wp, TabpageHandle::from_ptr(std::ptr::null_mut()))
    }
}

/// Count non-floating windows in a tabpage.
fn count_nonfloating_impl(tp: TabpageHandle) -> c_int {
    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        let mut count = 0;
        let mut wp = first;
        while !wp.is_null() {
            if nvim_win_get_floating(wp) == 0 {
                count += 1;
            }
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

/// Count total windows in a tabpage.
fn count_total_windows_impl(tp: TabpageHandle) -> c_int {
    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        let mut count = 0;
        let mut wp = first;
        while !wp.is_null() {
            count += 1;
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

/// Check if window can be closed without error.
///
/// This performs basic validation:
/// - Not the last window
/// - Not locked (window or buffer)
/// - Not an aucmd window
/// - Not in cmdwin mode (detailed cmdwin check must be done in C)
///
/// Returns 0 if can close, error code otherwise:
/// - 1: Last window
/// - 2: Window locked
/// - 3: Buffer locked
/// - 4: Is aucmd window
/// - 5: In cmdwin mode (detailed check needed in C)
fn can_close_window_impl(wp: WinHandle, force: bool) -> c_int {
    if wp.is_null() {
        return 1;
    }

    unsafe {
        // Check if last window
        if is_last_closable_window_impl(wp) {
            return 1;
        }

        // Check locked status
        if nvim_win_get_locked(wp) != 0 {
            return 2;
        }

        let buf = nvim_win_get_buffer(wp);
        if !buf.is_null() && nvim_buf_get_locked(buf) > 0 {
            return 3;
        }

        // Check aucmd window
        if rs_is_aucmd_win(wp) != 0 {
            return 4;
        }

        // Check cmdwin mode (detailed check for which window is cmdwin must be in C)
        if in_cmdwin_impl() {
            return 5;
        }

        // If would leave only floating and not forced, check if floating can close
        if !force && would_leave_only_floating_impl(wp) {
            let lastwin = nvim_get_lastwin();
            if rs_is_aucmd_win(lastwin) != 0 || !can_close_floating_windows_impl() {
                return 1; // Treat as "can't close last window"
            }
        }

        0 // Can close
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if window is locked.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_win_locked(wp: WinHandle) -> c_int {
    c_int::from(win_locked_impl(wp))
}

/// FFI: Check if window's buffer is locked.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_buf_locked(wp: WinHandle) -> c_int {
    c_int::from(buf_locked_impl(wp))
}

/// FFI: Check if window or buffer is locked.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_win_or_buf_locked(wp: WinHandle) -> c_int {
    c_int::from(win_or_buf_locked_impl(wp))
}

/// FFI: Check if in cmdwin mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_in_cmdwin() -> c_int {
    c_int::from(in_cmdwin_impl())
}

/// FFI: Check if floating windows can be closed in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_can_close_floating() -> c_int {
    c_int::from(can_close_floating_windows_impl())
}

/// FFI: Check if window is the only non-floating in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_is_only_nonfloating(wp: WinHandle, tp: TabpageHandle) -> c_int {
    c_int::from(is_only_nonfloating_impl(wp, tp))
}

/// FFI: Check if there's only one tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_only_one_tabpage() -> c_int {
    c_int::from(only_one_tabpage_impl())
}

/// FFI: Check if this is the last closable window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_is_last_closable(wp: WinHandle) -> c_int {
    c_int::from(is_last_closable_window_impl(wp))
}

/// FFI: Check if closing would leave only floating windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_would_leave_floating(wp: WinHandle) -> c_int {
    c_int::from(would_leave_only_floating_impl(wp))
}

/// FFI: Count non-floating windows in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_count_nonfloating(tp: TabpageHandle) -> c_int {
    count_nonfloating_impl(tp)
}

/// FFI: Count total windows in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_count_total(tp: TabpageHandle) -> c_int {
    count_total_windows_impl(tp)
}

/// FFI: Check if window can be closed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_can_close_window(wp: WinHandle, force: c_int) -> c_int {
    can_close_window_impl(wp, force != 0)
}

/// FFI: Get the reason why window cannot be closed (0 = can close).
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_close_error(wp: WinHandle, force: c_int) -> c_int {
    can_close_window_impl(wp, force != 0)
}

/// FFI: Check if window is in a closable state (quick check).
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_is_closable(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    unsafe {
        // Quick checks that don't require full validation
        if nvim_win_get_locked(wp) != 0 {
            return 0;
        }
        if rs_is_aucmd_win(wp) != 0 {
            return 0;
        }
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_window_checks() {
        let null_wp = WinHandle::null();
        assert!(!win_locked_impl(null_wp));
        assert!(!buf_locked_impl(null_wp));
        assert!(!is_only_nonfloating_impl(null_wp, unsafe {
            TabpageHandle::from_ptr(std::ptr::null_mut())
        }));
    }
}
