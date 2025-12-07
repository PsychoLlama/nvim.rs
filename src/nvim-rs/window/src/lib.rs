//! Window handling utilities for Neovim
//!
//! This crate provides Rust implementations of window-related functions
//! from `src/nvim/window.c`. It uses an opaque handle pattern where
//! `win_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::c_int;

/// Opaque handle to a Neovim window (`win_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut std::ffi::c_void);

impl WinHandle {
    /// Create a new window handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `win_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Neovim tabpage (`tabpage_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabpageHandle(*mut std::ffi::c_void);

impl TabpageHandle {
    /// Create a new tabpage handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `tabpage_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for window fields.
// These are defined in window.c and provide safe access to win_T fields.
extern "C" {
    /// Get the `w_locked` field from a window.
    fn nvim_win_get_locked(win: WinHandle) -> c_int;

    /// Get the `w_floating` field from a window.
    fn nvim_win_get_floating(win: WinHandle) -> c_int;

    /// Get the `w_p_pvw` (preview window) field from a window.
    fn nvim_win_get_pvw(win: WinHandle) -> c_int;

    /// Get the `w_next` field from a window.
    fn nvim_win_get_next(win: WinHandle) -> WinHandle;

    // Global state accessors
    /// Get the current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Get the first window in the current tab.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get the last window in the current tab.
    fn nvim_get_lastwin() -> WinHandle;

    /// Get the current tabpage.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get the `tp_firstwin` field from a tabpage.
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    /// Get the `tp_next` field from a tabpage.
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;

    /// Get the first tabpage (`first_tabpage` global).
    fn nvim_get_first_tabpage() -> TabpageHandle;
}

/// Check if a window is locked (`w_locked` field).
///
/// A locked window cannot be closed by autocommands.
#[inline]
fn win_locked_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and nvim_win_get_locked
    // is a simple field accessor that handles the pointer safely.
    unsafe { nvim_win_get_locked(wp) != 0 }
}

/// FFI wrapper for `win_locked`.
///
/// Returns non-zero if the window is locked.
#[no_mangle]
pub extern "C" fn rs_win_locked(wp: WinHandle) -> c_int {
    c_int::from(win_locked_impl(wp))
}

/// Check if a window is floating (`w_floating` field).
///
/// A floating window is a popup window that appears above other windows.
#[inline]
fn win_floating_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and nvim_win_get_floating
    // is a simple field accessor that handles the pointer safely.
    unsafe { nvim_win_get_floating(wp) != 0 }
}

/// FFI wrapper for `win_floating`.
///
/// Returns non-zero if the window is floating.
#[no_mangle]
pub extern "C" fn rs_win_floating(wp: WinHandle) -> c_int {
    c_int::from(win_floating_impl(wp))
}

/// Check if a window is a preview window (`w_p_pvw` field).
///
/// A preview window is used for displaying preview information.
#[inline]
fn win_pvw_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and nvim_win_get_pvw
    // is a simple field accessor that handles the pointer safely.
    unsafe { nvim_win_get_pvw(wp) != 0 }
}

/// FFI wrapper for `win_pvw`.
///
/// Returns non-zero if the window is a preview window.
#[no_mangle]
pub extern "C" fn rs_win_pvw(wp: WinHandle) -> c_int {
    c_int::from(win_pvw_impl(wp))
}

// Window iteration helpers

/// Get the first window in a tabpage.
///
/// For the current tabpage, this returns `firstwin`. For other tabpages,
/// it returns `tp->tp_firstwin`.
#[inline]
fn get_tabpage_firstwin(tp: TabpageHandle) -> WinHandle {
    // SAFETY: nvim_get_curtab returns a valid tabpage handle (or the check would be invalid)
    // and nvim_get_firstwin/nvim_tabpage_get_firstwin are safe accessors.
    unsafe {
        if tp == nvim_get_curtab() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        }
    }
}

/// Check if "win" is a pointer to an existing window in tabpage "tp".
///
/// This is the Rust equivalent of `tabpage_win_valid()` in window.c.
#[inline]
fn tabpage_win_valid_impl(tp: TabpageHandle, win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }

    let mut wp = get_tabpage_firstwin(tp);
    while !wp.is_null() {
        if wp == win {
            return true;
        }
        // SAFETY: nvim_win_get_next is a safe field accessor
        wp = unsafe { nvim_win_get_next(wp) };
    }
    false
}

/// FFI wrapper for `tabpage_win_valid`.
///
/// Returns non-zero if the window is valid in the given tabpage.
#[no_mangle]
pub extern "C" fn rs_tabpage_win_valid(tp: TabpageHandle, win: WinHandle) -> c_int {
    c_int::from(tabpage_win_valid_impl(tp, win))
}

/// Check if "win" is a pointer to an existing window in the current tabpage.
///
/// This is the Rust equivalent of `win_valid()` in window.c.
#[inline]
fn win_valid_impl(win: WinHandle) -> bool {
    // SAFETY: nvim_get_curtab returns a valid tabpage handle
    tabpage_win_valid_impl(unsafe { nvim_get_curtab() }, win)
}

/// FFI wrapper for `win_valid`.
///
/// Returns non-zero if the window is valid in the current tabpage.
#[no_mangle]
pub extern "C" fn rs_win_valid(win: WinHandle) -> c_int {
    c_int::from(win_valid_impl(win))
}

/// Check if there is only one window in the current tabpage (excluding floating windows).
///
/// This is the Rust equivalent of the `ONE_WINDOW` macro, which checks `firstwin == lastwin`.
#[inline]
fn one_window_impl() -> bool {
    // SAFETY: nvim_get_firstwin and nvim_get_lastwin are safe accessors
    unsafe { nvim_get_firstwin() == nvim_get_lastwin() }
}

/// FFI wrapper for checking if there's only one window.
///
/// Returns non-zero if there is only one window in the current tabpage.
#[no_mangle]
pub extern "C" fn rs_one_window() -> c_int {
    c_int::from(one_window_impl())
}

/// Check if "win" is a pointer to an existing window in any tabpage.
///
/// This is the Rust equivalent of `win_valid_any_tab()` in window.c.
#[inline]
fn win_valid_any_tab_impl(win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }

    // Iterate over all tabpages using FOR_ALL_TABS pattern
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tabpage_win_valid_impl(tp, win) {
            return true;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    false
}

/// FFI wrapper for `win_valid_any_tab`.
///
/// Returns non-zero if the window is valid in any tabpage.
#[no_mangle]
pub extern "C" fn rs_win_valid_any_tab(win: WinHandle) -> c_int {
    c_int::from(win_valid_any_tab_impl(win))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_handle_null() {
        let handle = unsafe { WinHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!win_locked_impl(handle));
        assert!(!win_floating_impl(handle));
        assert!(!win_pvw_impl(handle));
    }

    #[test]
    fn test_win_handle_non_null() {
        // Create a fake non-null pointer for testing
        let fake_ptr = 0x1000 as *mut std::ffi::c_void;
        let handle = unsafe { WinHandle::from_ptr(fake_ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), fake_ptr);
    }
}
