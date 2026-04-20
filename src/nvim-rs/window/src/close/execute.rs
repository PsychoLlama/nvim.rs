//! Window close execution helper functions.
//!
//! This module provides helper functions for window close execution,
//! supporting the C implementation of win_close, win_close_othertab, etc.
//!
//! Note: The main closing functions (`win_close`, `win_close_othertab`) remain in C
//! due to their complexity (~275 lines for `win_close`) and dependencies on:
//! - Extensive autocmd handling (WinLeave, WinClosed, BufLeave, etc.)
//! - Buffer management
//! - Tabpage management
//! - Memory deallocation
//! - Layout restoration
//!
//! This module provides helper functions that can be called from C
//! to assist with window close operations.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::win_struct::win_ref;
use crate::{Frame, TabpageHandle, WinHandle, FR_COL};

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

}

// =============================================================================
// Close Direction Helpers
// =============================================================================

/// Returns 'v' for vertical direction (FR_COL parent) or 'h' for horizontal.
fn get_close_direction_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return c_int::from(b'h');
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return c_int::from(b'h');
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return c_int::from(b'h');
        }

        if (*parent).fr_layout == FR_COL {
            c_int::from(b'v')
        } else {
            c_int::from(b'h')
        }
    }
}

/// Returns next sibling if available, otherwise prev.
fn get_basic_sibling_frame_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return std::ptr::null_mut();
        }

        let next = (*frame).fr_next;
        if !next.is_null() {
            return next;
        }

        (*frame).fr_prev
    }
}

// =============================================================================
// Frame Tree Navigation for Close
// =============================================================================

/// Get the first non-floating window in a tabpage.
fn first_nonfloat_impl(tp: TabpageHandle) -> WinHandle {
    // firstwin should always be non-floating
    unsafe {
        if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        }
    }
}

/// Get the last non-floating window in a tabpage.
fn last_nonfloat_impl(tp: TabpageHandle) -> WinHandle {
    unsafe {
        let lastwin = if tp.is_null() {
            nvim_get_lastwin()
        } else {
            // For other tabpages, we need to find lastwin
            // by iterating from firstwin
            let first = nvim_tabpage_get_firstwin(tp);
            let mut wp = first;
            let mut last = first;
            while !wp.is_null() {
                last = wp;
                wp = win_ref(wp).w_next;
            }
            last
        };

        // Walk backwards to find last non-floating
        let mut wp = lastwin;
        while !wp.is_null() && win_ref(wp).w_floating {
            wp = win_ref(wp).w_prev;
        }
        wp
    }
}

/// Count tabpages.
fn count_tabpages_impl() -> c_int {
    unsafe {
        let mut count = 0;
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            count += 1;
            tp = nvim_tabpage_get_next(tp);
        }
        count
    }
}

/// Check if window is the first non-floating window.
fn is_first_nonfloat_impl(wp: WinHandle, tp: TabpageHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    first_nonfloat_impl(tp) == wp
}

/// Check if window is the last non-floating window.
fn is_last_nonfloat_impl(wp: WinHandle, tp: TabpageHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    last_nonfloat_impl(tp) == wp
}

// =============================================================================
// Close Operation State Helpers
// =============================================================================

/// True if this is the only non-floating window in its tabpage.
fn close_triggers_tabpage_close_impl(wp: WinHandle, tp: TabpageHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        // Window must be first
        if first != wp {
            return false;
        }

        // Next must be null or floating
        let next = win_ref(wp).w_next;
        next.is_null() || win_ref(next).w_floating
    }
}

/// Check if window is a floating window.
fn is_floating_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    unsafe { win_ref(wp).w_floating }
}

/// Check if window frame has siblings.
fn frame_has_siblings_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return false;
        }

        !(*frame).fr_prev.is_null() || !(*frame).fr_next.is_null()
    }
}

/// Check if window frame has a previous sibling.
fn frame_has_prev_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return false;
        }

        !(*frame).fr_prev.is_null()
    }
}

/// Check if window frame has a next sibling.
fn frame_has_next_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return false;
        }

        !(*frame).fr_next.is_null()
    }
}

/// Get the frame height that will be freed.
fn freed_height_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return 0;
        }

        (*frame).fr_height
    }
}

/// Get the frame width that will be freed.
fn freed_width_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return 0;
        }

        (*frame).fr_width
    }
}

/// Get the previous sibling frame.
fn get_prev_sibling_frame_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return std::ptr::null_mut();
        }

        (*frame).fr_prev
    }
}

/// Get the next sibling frame.
fn get_next_sibling_frame_impl(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return std::ptr::null_mut();
        }

        (*frame).fr_next
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get direction character for close operation ('v' or 'h').
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_direction(wp: WinHandle) -> c_int {
    get_close_direction_impl(wp)
}

/// FFI: Get the basic sibling frame that receives space.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_basic_sibling(wp: WinHandle) -> *mut Frame {
    get_basic_sibling_frame_impl(wp)
}

/// FFI: Get first non-floating window in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_first_nonfloat(tp: TabpageHandle) -> WinHandle {
    first_nonfloat_impl(tp)
}

/// FFI: Get last non-floating window in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_last_nonfloat(tp: TabpageHandle) -> WinHandle {
    last_nonfloat_impl(tp)
}

/// FFI: Count tabpages.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_count_tabpages() -> c_int {
    count_tabpages_impl()
}

/// FFI: Check if window is first non-floating.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_is_first_nonfloat(wp: WinHandle, tp: TabpageHandle) -> c_int {
    c_int::from(is_first_nonfloat_impl(wp, tp))
}

/// FFI: Check if window is last non-floating.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_is_last_nonfloat(wp: WinHandle, tp: TabpageHandle) -> c_int {
    c_int::from(is_last_nonfloat_impl(wp, tp))
}

/// FFI: Check if close triggers tabpage close.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_triggers_tabpage(wp: WinHandle, tp: TabpageHandle) -> c_int {
    c_int::from(close_triggers_tabpage_close_impl(wp, tp))
}

/// FFI: Check if window is floating.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_is_floating(wp: WinHandle) -> c_int {
    c_int::from(is_floating_impl(wp))
}

/// FFI: Check if frame has siblings.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_frame_has_siblings(wp: WinHandle) -> c_int {
    c_int::from(frame_has_siblings_impl(wp))
}

/// FFI: Check if frame has previous sibling.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_frame_has_prev(wp: WinHandle) -> c_int {
    c_int::from(frame_has_prev_impl(wp))
}

/// FFI: Check if frame has next sibling.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_frame_has_next(wp: WinHandle) -> c_int {
    c_int::from(frame_has_next_impl(wp))
}

/// FFI: Get freed frame height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_freed_height(wp: WinHandle) -> c_int {
    freed_height_impl(wp)
}

/// FFI: Get freed frame width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_freed_width(wp: WinHandle) -> c_int {
    freed_width_impl(wp)
}

/// FFI: Get current window for close operation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_curwin() -> WinHandle {
    unsafe { nvim_get_curwin() }
}

/// FFI: Get current tabpage for close operation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_curtab() -> TabpageHandle {
    unsafe { nvim_get_curtab() }
}

/// FFI: Check if window has a frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_has_frame(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    unsafe { c_int::from(!win_ref(wp).w_frame.is_null()) }
}

/// FFI: Get window's frame parent layout.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_parent_layout(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let frame = win_ref(wp).w_frame;
        if frame.is_null() {
            return 0;
        }

        let parent = (*frame).fr_parent;
        if parent.is_null() {
            return 0;
        }

        c_int::from((*parent).fr_layout)
    }
}

/// FFI: Get the previous sibling frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_prev_sibling(wp: WinHandle) -> *mut Frame {
    get_prev_sibling_frame_impl(wp)
}

/// FFI: Get the next sibling frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_close_get_next_sibling(wp: WinHandle) -> *mut Frame {
    get_next_sibling_frame_impl(wp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_window_checks() {
        let null_wp = WinHandle::null();
        assert!(!is_floating_impl(null_wp));
        assert!(!frame_has_siblings_impl(null_wp));
        assert!(!frame_has_prev_impl(null_wp));
        assert!(!frame_has_next_impl(null_wp));
        assert_eq!(freed_height_impl(null_wp), 0);
        assert_eq!(freed_width_impl(null_wp), 0);
    }

    #[test]
    fn test_direction_default() {
        // Null window should return 'h' (horizontal)
        let null_wp = WinHandle::null();
        assert_eq!(get_close_direction_impl(null_wp), c_int::from(b'h'));
    }

    #[test]
    fn test_null_sibling_frames() {
        let null_wp = WinHandle::null();
        assert!(get_prev_sibling_frame_impl(null_wp).is_null());
        assert!(get_next_sibling_frame_impl(null_wp).is_null());
        assert!(get_basic_sibling_frame_impl(null_wp).is_null());
    }
}
