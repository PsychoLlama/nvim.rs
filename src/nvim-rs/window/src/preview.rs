//! Preview window operations.
//!
//! This module provides helper functions for preview window operations,
//! supporting the C implementation of preview window functionality.
//!
//! Note: The main preview window functions remain in C due to their
//! integration with tag preview and tag stack operations.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get firstwin.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get w_next from window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get w_p_pvw (preview window flag) from window.
    fn nvim_win_get_pvw(wp: WinHandle) -> c_int;

    /// Get curtab.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get tp_firstwin from tabpage.
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
}

// =============================================================================
// Preview Window Finding
// =============================================================================

/// Find the preview window in the current tabpage.
fn find_preview_window_impl() -> WinHandle {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_pvw(wp) != 0 {
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

/// Find the preview window in a specific tabpage.
fn find_preview_window_in_tab_impl(tp: TabpageHandle) -> WinHandle {
    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        let mut wp = first;
        while !wp.is_null() {
            if nvim_win_get_pvw(wp) != 0 {
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

/// Check if there is a preview window in the current tabpage.
fn has_preview_window_impl() -> bool {
    !find_preview_window_impl().is_null()
}

/// Check if there is a preview window in a specific tabpage.
fn has_preview_window_in_tab_impl(tp: TabpageHandle) -> bool {
    !find_preview_window_in_tab_impl(tp).is_null()
}

// =============================================================================
// Preview Window State
// =============================================================================

/// Check if a window is the preview window.
fn is_preview_window_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    unsafe { nvim_win_get_pvw(wp) != 0 }
}

/// Check if the current window is the preview window.
fn curwin_is_preview_impl() -> bool {
    unsafe {
        let curwin = nvim_get_curwin();
        is_preview_window_impl(curwin)
    }
}

/// Check if the preview window is floating.
fn preview_is_floating_impl() -> bool {
    let preview = find_preview_window_impl();
    if preview.is_null() {
        return false;
    }
    unsafe { nvim_win_get_floating(preview) != 0 }
}

// =============================================================================
// Preview Window Counting
// =============================================================================

/// Count preview windows in the current tabpage (should be 0 or 1).
fn count_preview_windows_impl() -> c_int {
    unsafe {
        let mut count = 0;
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_pvw(wp) != 0 {
                count += 1;
            }
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

/// Count preview windows in a specific tabpage.
fn count_preview_windows_in_tab_impl(tp: TabpageHandle) -> c_int {
    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        let mut count = 0;
        let mut wp = first;
        while !wp.is_null() {
            if nvim_win_get_pvw(wp) != 0 {
                count += 1;
            }
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Find preview window in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_find() -> WinHandle {
    find_preview_window_impl()
}

/// FFI: Find preview window in specific tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_find_in_tab(tp: TabpageHandle) -> WinHandle {
    find_preview_window_in_tab_impl(tp)
}

/// FFI: Check if preview window exists.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_exists() -> c_int {
    c_int::from(has_preview_window_impl())
}

/// FFI: Check if preview window exists in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_exists_in_tab(tp: TabpageHandle) -> c_int {
    c_int::from(has_preview_window_in_tab_impl(tp))
}

/// FFI: Check if window is preview window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_is_preview(wp: WinHandle) -> c_int {
    c_int::from(is_preview_window_impl(wp))
}

/// FFI: Check if current window is preview.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_curwin_is_preview() -> c_int {
    c_int::from(curwin_is_preview_impl())
}

/// FFI: Check if preview window is floating.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_is_floating() -> c_int {
    c_int::from(preview_is_floating_impl())
}

/// FFI: Count preview windows in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_count() -> c_int {
    count_preview_windows_impl()
}

/// FFI: Count preview windows in specific tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_preview_count_in_tab(tp: TabpageHandle) -> c_int {
    count_preview_windows_in_tab_impl(tp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_window_preview() {
        let null_wp = WinHandle::null();
        assert!(!is_preview_window_impl(null_wp));
    }
}
