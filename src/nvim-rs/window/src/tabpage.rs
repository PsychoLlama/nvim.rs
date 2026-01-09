//! Tab page management functions.
//!
//! This module provides Rust implementations of tab page operations
//! from `src/nvim/window.c`.
//!
//! Note: The FFI exported functions (`rs_*`) are still defined in `lib.rs`
//! for now to avoid duplicate symbol errors. This module provides helper
//! functions that are used by the main implementations.

use std::ffi::c_int;

use crate::{TabpageHandle, WinHandle};

// Import list module functions we depend on
use crate::list::{
    get_tabpage_firstwin, nvim_get_first_tabpage, nvim_tabpage_get_next, nvim_win_get_next,
    win_valid_any_tab_impl,
};

/// Check if "tpc" is a pointer to an existing tabpage.
///
/// This is the Rust equivalent of `valid_tabpage()` in window.c.
#[inline]
#[must_use]
pub(crate) fn valid_tabpage_impl(tpc: TabpageHandle) -> bool {
    if tpc.is_null() {
        return false;
    }

    // Iterate over all tabpages using FOR_ALL_TABS pattern
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tp == tpc {
            return true;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    false
}

// Note: FFI wrapper rs_valid_tabpage is in lib.rs

/// Get the 1-based index of a tabpage.
///
/// This is the Rust equivalent of `tabpage_index()` in window.c.
/// Iterates through tabpages from `first_tabpage` to find the index.
#[inline]
fn tabpage_index_impl(ftp: TabpageHandle) -> c_int {
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut i: c_int = 1;
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() && tp != ftp {
        i += 1;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    i
}

// Note: FFI wrapper rs_tabpage_index is in lib.rs

/// Check if a tabpage has any valid window.
///
/// This is the Rust equivalent of `valid_tabpage_win()` in window.c.
/// Iterates through all tabpages to find `tpc`, then checks if any window
/// in that tabpage is valid (using `win_valid_any_tab`).
#[inline]
fn valid_tabpage_win_impl(tpc: TabpageHandle) -> bool {
    if tpc.is_null() {
        return false;
    }

    // Find the tabpage in the list
    // SAFETY: All accessors handle pointers safely
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tp == tpc {
            // Found the tabpage - check if any window is valid
            let mut wp = get_tabpage_firstwin(tp);
            while !wp.is_null() {
                if win_valid_any_tab_impl(wp) {
                    return true;
                }
                wp = unsafe { nvim_win_get_next(wp) };
            }
            return false;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    // Tabpage not found - shouldn't happen
    false
}

// Note: FFI wrapper rs_valid_tabpage_win is in lib.rs

/// Find tab page by 1-based number.
///
/// This is the Rust equivalent of `find_tabpage()` in window.c.
/// Iterates through tabpages from `first_tabpage` counting to n.
/// Returns NULL when not found.
#[inline]
fn find_tabpage_impl(n: c_int) -> TabpageHandle {
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut i: c_int = 1;
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() && i != n {
        i += 1;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    tp
}

// Note: FFI wrapper rs_find_tabpage is in lib.rs

/// Find the tabpage that contains a given window.
///
/// This is the Rust equivalent of `win_find_tabpage()` in window.c.
/// Iterates through all tabpages and windows using FOR_ALL_TAB_WINDOWS pattern.
#[inline]
fn win_find_tabpage_impl(win: WinHandle) -> TabpageHandle {
    if win.is_null() {
        return unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) };
    }

    // FOR_ALL_TAB_WINDOWS pattern: iterate through all tabpages and their windows
    // SAFETY: All accessors handle pointers safely
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        // Iterate through windows in this tabpage
        let mut wp = get_tabpage_firstwin(tp);
        while !wp.is_null() {
            if wp == win {
                return tp;
            }
            wp = unsafe { nvim_win_get_next(wp) };
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    // Return null if not found
    unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) }
}

// Note: FFI wrapper rs_win_find_tabpage is in lib.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabpage_handle_null() {
        let handle = unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!valid_tabpage_impl(handle));
    }
}
