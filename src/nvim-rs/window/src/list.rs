//! Window list management functions.
//!
//! This module provides Rust implementations of window list traversal and
//! initialization functions from `src/nvim/window.c`.
//!
//! Note: The FFI exported functions (`rs_*`) are still defined in `lib.rs`
//! for now to avoid duplicate symbol errors. This module provides helper
//! functions that are used by the main implementations.

use std::ffi::c_int;

use crate::{BufHandle, Frame, TabpageHandle, WinHandle, FR_LEAF};

// C accessor functions for window fields.
extern "C" {
    /// Get the `w_next` field from a window.
    pub(crate) fn nvim_win_get_next(win: WinHandle) -> WinHandle;

    /// Get the `w_prev` field from a window.
    pub(crate) fn nvim_win_get_prev(win: WinHandle) -> WinHandle;

    /// Get the current window.
    pub(crate) fn nvim_get_curwin() -> WinHandle;

    /// Get the first window in the current tab.
    pub(crate) fn nvim_get_firstwin() -> WinHandle;

    /// Get the last window in the current tab.
    pub(crate) fn nvim_get_lastwin() -> WinHandle;

    /// Get the current tabpage.
    pub(crate) fn nvim_get_curtab() -> TabpageHandle;

    /// Get the `tp_firstwin` field from a tabpage.
    pub(crate) fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    /// Get the `tp_next` field from a tabpage.
    pub(crate) fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;

    /// Get the first tabpage (`first_tabpage` global).
    pub(crate) fn nvim_get_first_tabpage() -> TabpageHandle;

    /// Get the `w_floating` field from a window.
    pub(crate) fn nvim_win_get_floating(win: WinHandle) -> c_int;

    /// Get the `w_p_pvw` (preview window) field from a window.
    fn nvim_win_get_pvw(win: WinHandle) -> c_int;

    /// Get the `w_buffer` field from a window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get the current buffer (`curbuf` global).
    fn nvim_get_curbuf() -> BufHandle;

    /// Check if buffer is a help buffer.
    fn rs_bt_help(buf: BufHandle) -> c_int;

    /// Check if window is an aucmd_win.
    fn rs_is_aucmd_win(win: WinHandle) -> c_int;

    /// Get the `handle` field from a window.
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;

    /// Get the `w_frame` field from a window (returns Frame pointer).
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get the `last_win_id` global.
    fn nvim_get_last_win_id() -> c_int;
}

/// Get the first window in a tabpage.
///
/// For the current tabpage, this returns `firstwin`. For other tabpages,
/// it returns `tp->tp_firstwin`.
#[inline]
#[must_use]
pub(crate) fn get_tabpage_firstwin(tp: TabpageHandle) -> WinHandle {
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
#[must_use]
pub(crate) fn tabpage_win_valid_impl(tp: TabpageHandle, win: WinHandle) -> bool {
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

// Note: FFI wrapper rs_tabpage_win_valid is in lib.rs

/// Check if "win" is a pointer to an existing window in the current tabpage.
///
/// This is the Rust equivalent of `win_valid()` in window.c.
#[inline]
#[must_use]
pub(crate) fn win_valid_impl(win: WinHandle) -> bool {
    // SAFETY: nvim_get_curtab returns a valid tabpage handle
    tabpage_win_valid_impl(unsafe { nvim_get_curtab() }, win)
}

// Note: FFI wrapper rs_win_valid is in lib.rs

/// Check if "win" is a floating window in the current tabpage.
///
/// Iterates through all windows in the current tabpage to find the given
/// window, then returns whether it has the floating flag set.
#[inline]
fn win_float_valid_impl(win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }

    // SAFETY: nvim_get_curtab returns a valid tabpage handle
    let curtab = unsafe { nvim_get_curtab() };
    let mut wp = get_tabpage_firstwin(curtab);
    while !wp.is_null() {
        if wp == win {
            // SAFETY: nvim_win_get_floating is a safe field accessor
            return unsafe { nvim_win_get_floating(wp) != 0 };
        }
        // SAFETY: nvim_win_get_next is a safe field accessor
        wp = unsafe { nvim_win_get_next(wp) };
    }
    false
}

// Note: FFI wrapper rs_win_float_valid is in lib.rs

/// Check that there is only one window (and only one tab page), not counting a
/// help or preview window, unless it is the current window. Does not count
/// "aucmd_win". Does not count floats unless it is current.
#[inline]
fn only_one_window_impl() -> bool {
    // SAFETY: All accessor functions are safe
    unsafe {
        // If there is another tab page there always is another window.
        let first_tabpage = nvim_get_first_tabpage();
        if !nvim_tabpage_get_next(first_tabpage).is_null() {
            return false;
        }

        let curwin = nvim_get_curwin();
        let curbuf = nvim_get_curbuf();
        let curbuf_is_help = rs_bt_help(curbuf) != 0;

        let curtab = nvim_get_curtab();
        let mut count = 0;
        let mut wp = get_tabpage_firstwin(curtab);

        while !wp.is_null() {
            let buf = nvim_win_get_buffer(wp);
            if !buf.is_null() {
                let is_help = rs_bt_help(buf) != 0;
                let is_floating = nvim_win_get_floating(wp) != 0;
                let is_pvw = nvim_win_get_pvw(wp) != 0;
                let is_curwin = wp == curwin;
                let is_aucmd = rs_is_aucmd_win(wp) != 0;

                // Count if:
                // - Not a help window (unless curbuf is also help) AND not floating AND not preview
                //   OR it's the current window
                // - AND not an aucmd_win
                let should_skip = (is_help && !curbuf_is_help) || is_floating || is_pvw;
                if (!should_skip || is_curwin) && !is_aucmd {
                    count += 1;
                }
            }
            wp = nvim_win_get_next(wp);
        }

        count <= 1
    }
}

// Note: FFI wrapper rs_only_one_window is in lib.rs

/// Check if there is only one window in the current tabpage (excluding floating windows).
///
/// This is the Rust equivalent of the `ONE_WINDOW` macro, which checks `firstwin == lastwin`.
#[inline]
fn one_window_impl() -> bool {
    // SAFETY: nvim_get_firstwin and nvim_get_lastwin are safe accessors
    unsafe { nvim_get_firstwin() == nvim_get_lastwin() }
}

// Note: FFI wrapper rs_one_window is in lib.rs

/// Check if "win" is a pointer to an existing window in any tabpage.
///
/// This is the Rust equivalent of `win_valid_any_tab()` in window.c.
#[inline]
#[must_use]
pub(crate) fn win_valid_any_tab_impl(win: WinHandle) -> bool {
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

// Note: FFI wrapper rs_win_valid_any_tab is in lib.rs

/// Check if "win" is the only non-floating window in a tabpage.
///
/// For `tp == NULL` (current tabpage), uses `firstwin`.
/// Otherwise uses `tp->tp_firstwin`.
///
/// This is the Rust equivalent of `one_window()` in window.c.
/// Note: The C version has an assert that `(!tp || tp != curtab) && !first->w_floating`,
/// meaning tp should not be curtab when non-NULL, and the first window should not be floating.
/// We don't check the assert here as the caller is responsible for ensuring this.
#[inline]
fn one_window_in_tab_impl(win: WinHandle, tp: TabpageHandle) -> bool {
    if win.is_null() {
        return false;
    }

    // Get the first window in the tabpage
    // SAFETY: All accessors are safe
    let first = if tp.is_null() {
        unsafe { nvim_get_firstwin() }
    } else {
        unsafe { nvim_tabpage_get_firstwin(tp) }
    };

    if first != win {
        return false;
    }

    // Check if win->w_next is NULL or floating
    // SAFETY: nvim_win_get_next and nvim_win_get_floating are safe accessors
    let next = unsafe { nvim_win_get_next(win) };
    next.is_null() || unsafe { nvim_win_get_floating(next) != 0 }
}

// Note: FFI wrapper rs_one_window_in_tab is in lib.rs

/// Check if there is only one tabpage (i.e., `first_tabpage->tp_next == NULL`).
///
/// This is used by `last_window()` to check if there's only one tab.
#[inline]
fn one_tabpage_impl() -> bool {
    // SAFETY: nvim_get_first_tabpage and nvim_tabpage_get_next are safe accessors
    unsafe {
        let first = nvim_get_first_tabpage();
        nvim_tabpage_get_next(first).is_null()
    }
}

// Note: FFI wrapper rs_one_tabpage is in lib.rs

/// Check if "win" is the last non-floating window that exists.
///
/// This checks: `one_window(win, NULL) && first_tabpage->tp_next == NULL`.
///
/// This is the Rust equivalent of `last_window()` in window.c.
#[inline]
fn last_window_impl(win: WinHandle) -> bool {
    // Check if there's only one non-floating window in current tabpage
    // AND there's only one tabpage
    one_window_in_tab_impl(win, unsafe {
        TabpageHandle::from_ptr(std::ptr::null_mut())
    }) && one_tabpage_impl()
}

// Note: FFI wrapper rs_last_window is in lib.rs

/// Count the number of windows in the current tabpage.
///
/// This is the Rust equivalent of `win_count()` in window.c.
/// Iterates through all windows in the current tab (firstwin -> `w_next`).
#[inline]
fn win_count_impl() -> c_int {
    // SAFETY: nvim_get_firstwin and nvim_win_get_next are safe accessors
    let mut count: c_int = 0;
    let mut wp = unsafe { nvim_get_firstwin() };
    while !wp.is_null() {
        count += 1;
        wp = unsafe { nvim_win_get_next(wp) };
    }
    count
}

// Note: FFI wrapper rs_win_count is in lib.rs

/// Find a window by its handle in the current tabpage.
///
/// This is the Rust equivalent of `win_find_by_handle()` in window.c.
/// Iterates through all windows in curtab, returning the one with the matching handle.
#[inline]
fn win_find_by_handle_impl(handle: c_int) -> WinHandle {
    // Get curtab to use FOR_ALL_WINDOWS_IN_TAB pattern
    // SAFETY: All accessors handle pointers safely
    let curtab = unsafe { nvim_get_curtab() };
    let mut wp = get_tabpage_firstwin(curtab);
    while !wp.is_null() {
        // SAFETY: nvim_win_get_handle is a safe accessor
        if unsafe { nvim_win_get_handle(wp) } == handle {
            return wp;
        }
        wp = unsafe { nvim_win_get_next(wp) };
    }
    // Return null if not found
    WinHandle::null()
}

// Note: FFI wrapper rs_win_find_by_handle is in lib.rs

/// Get the last window ID assigned.
///
/// This is the Rust equivalent of `get_last_winid()` in window.c.
/// Returns the global last_win_id value.
#[inline]
fn get_last_winid_impl() -> c_int {
    // SAFETY: nvim_get_last_win_id is a safe accessor
    unsafe { nvim_get_last_win_id() }
}

// Note: FFI wrapper rs_get_last_winid is in lib.rs

/// Find the last non-floating window.
///
/// This is the Rust equivalent of `lastwin_nofloating()` in window.c.
/// Iterates backwards from `lastwin` to find the first non-floating window.
#[inline]
#[must_use]
pub(crate) fn lastwin_nofloating_impl() -> WinHandle {
    // SAFETY: nvim_get_lastwin, nvim_win_get_prev, nvim_win_get_floating are safe accessors
    let mut res = unsafe { nvim_get_lastwin() };
    while !res.is_null() && unsafe { nvim_win_get_floating(res) } != 0 {
        res = unsafe { nvim_win_get_prev(res) };
    }
    res
}

// Note: FFI wrapper rs_lastwin_nofloating is in lib.rs

/// Find the left-upper window in frame.
///
/// Walks down the frame tree following fr_child until a leaf frame
/// with a window is found.
///
/// # Safety
/// `frp` must be a valid, non-null frame pointer.
#[inline]
#[must_use]
pub(crate) unsafe fn frame2win_impl(mut frp: *mut Frame) -> WinHandle {
    // SAFETY: The caller guarantees frp is non-null.
    // The loop walks down until we find a leaf with fr_win != NULL.
    while (*frp).fr_win.is_null() {
        frp = (*frp).fr_child;
    }
    (*frp).fr_win
}

// Note: FFI wrapper rs_frame2win is in lib.rs

/// Check if a frame tree contains a specific window.
///
/// This is the Rust equivalent of `frame_has_win()` in window.c.
/// Recursively searches the frame tree for the given window.
#[inline]
#[must_use]
pub(crate) fn frame_has_win_impl(frp: *const Frame, wp: WinHandle) -> bool {
    if frp.is_null() {
        return false;
    }

    // SAFETY: We check for null above and the caller guarantees valid frame pointer
    unsafe {
        let frame = &*frp;
        if frame.fr_layout == FR_LEAF {
            // Leaf frame - check if it contains the window
            return frame.fr_win == wp;
        }

        // Non-leaf frame - recursively check children
        let mut child = frame.fr_child;
        while !child.is_null() {
            if frame_has_win_impl(child, wp) {
                return true;
            }
            child = (*child).fr_next;
        }
    }
    false
}

// Note: FFI wrapper rs_frame_has_win is in lib.rs

/// Check if window is at the bottom of its column.
///
/// This is the Rust equivalent of `is_bottom_win()` in window.c.
/// Returns true if there are no windows below the current window.
/// Traverses up the frame tree, checking if any parent is a column
/// layout with a sibling frame below.
#[inline]
fn is_bottom_win_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }

    // Get the window's frame
    // SAFETY: wp is not null, and nvim_win_get_frame is a safe accessor
    let mut frp = unsafe { nvim_win_get_frame(wp) };

    // Traverse up the frame tree
    // SAFETY: We access frame fields directly
    unsafe {
        while !frp.is_null() {
            let parent = (*frp).fr_parent;
            if parent.is_null() {
                break;
            }

            // If parent is a column layout and there's a sibling below, not at bottom
            let parent_layout = (*parent).fr_layout;
            let next_sibling = (*frp).fr_next;

            if parent_layout == crate::FR_COL && !next_sibling.is_null() {
                return false;
            }

            frp = parent;
        }
    }
    true
}

// Note: FFI wrapper rs_is_bottom_win is in lib.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_handle_null() {
        let handle = WinHandle::null();
        assert!(handle.is_null());
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
