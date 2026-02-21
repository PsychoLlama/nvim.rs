//! Window allocation functions.
//!
//! This module provides Rust implementations of window and frame allocation
//! functions from `src/nvim/window.c`.
//!
//! Note: The main allocation functions (`win_alloc`, `win_free`) remain in C
//! due to their complex dependencies on memory management, autocmds, and
//! other subsystems. This module provides helper functions and simpler
//! allocation utilities.

// The complex win_alloc() and win_free() functions have too many dependencies
// on C subsystems to be easily migrated:
// - Memory allocation (xcalloc, xfree)
// - Window handles map (pmap_put, pmap_del)
// - Grid allocation
// - Variable dictionaries
// - Autocmd blocking
// - Fold initialization
// - Argument lists
// - Buffer lists

// However, window list operations (win_append, win_remove) can be migrated
// since they are pure linked-list manipulations that use accessor functions.

use crate::{TabpageHandle, WinHandle};

// C accessor functions for window list manipulation.
extern "C" {
    // Getters
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_lastwin() -> WinHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle;

    // Setters
    fn nvim_win_set_next(wp: WinHandle, next: WinHandle);
    fn nvim_win_set_prev(wp: WinHandle, prev: WinHandle);
    fn nvim_set_firstwin(wp: WinHandle);
    fn nvim_set_lastwin(wp: WinHandle);
    fn nvim_tabpage_set_firstwin(tp: TabpageHandle, wp: WinHandle);
    fn nvim_tabpage_set_lastwin(tp: TabpageHandle, wp: WinHandle);
}

/// Append window "wp" in the window list after window "after".
///
/// This is the Rust implementation of `win_append()` in window.c.
///
/// # Arguments
/// * `after` - Window to insert after (NULL = insert at beginning)
/// * `wp` - Window to insert (must not be NULL)
/// * `tp` - Tab page "win" is in (NULL for current tabpage)
///
/// # Safety
/// - `wp` must be a valid, non-null window pointer
/// - `tp` must be NULL or a valid tabpage that is NOT the current tabpage
/// - `after` must be NULL or a valid window in the specified tabpage
fn win_append_impl(after: WinHandle, wp: WinHandle, tp: TabpageHandle) {
    // SAFETY: All accessor functions handle pointers safely.
    // The assertion from C (tp == NULL || tp != curtab) should be ensured by caller.
    unsafe {
        // Determine first window pointer based on tabpage
        // Note: we only need first, last is updated via the links
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        // after NULL is in front of the first
        let before = if after.is_null() {
            first
        } else {
            nvim_win_get_next(after)
        };

        // Link wp into the list
        nvim_win_set_next(wp, before);
        nvim_win_set_prev(wp, after);

        // Update previous link
        if after.is_null() {
            // wp becomes the new first window
            if tp.is_null() {
                nvim_set_firstwin(wp);
            } else {
                nvim_tabpage_set_firstwin(tp, wp);
            }
        } else {
            nvim_win_set_next(after, wp);
        }

        // Update next link
        if before.is_null() {
            // wp becomes the new last window
            if tp.is_null() {
                nvim_set_lastwin(wp);
            } else {
                nvim_tabpage_set_lastwin(tp, wp);
            }
        } else {
            nvim_win_set_prev(before, wp);
        }
    }
}

/// FFI wrapper for `win_append`.
///
/// Appends a window in the window list after another window.
#[no_mangle]
pub extern "C" fn rs_win_append(after: WinHandle, wp: WinHandle, tp: TabpageHandle) {
    unsafe {
        debug_assert!(tp.is_null() || tp != nvim_get_curtab());
    }
    win_append_impl(after, wp, tp);
}

/// Remove a window from the window list.
///
/// This is the Rust implementation of `win_remove()` in window.c.
///
/// # Arguments
/// * `wp` - Window to remove (must not be NULL)
/// * `tp` - Tab page "win" is in (NULL for current tabpage)
///
/// # Safety
/// - `wp` must be a valid, non-null window pointer
/// - `tp` must be NULL or a valid tabpage that is NOT the current tabpage
fn win_remove_impl(wp: WinHandle, tp: TabpageHandle) {
    // SAFETY: All accessor functions handle pointers safely.
    // The assertion from C (tp == NULL || tp != curtab) should be ensured by caller.
    unsafe {
        let prev = nvim_win_get_prev(wp);
        let next = nvim_win_get_next(wp);

        // Update previous window's next pointer
        if !prev.is_null() {
            nvim_win_set_next(prev, next);
        } else if tp.is_null() {
            // wp was the first window
            nvim_set_firstwin(next);
        } else {
            nvim_tabpage_set_firstwin(tp, next);
        }

        // Update next window's prev pointer
        if !next.is_null() {
            nvim_win_set_prev(next, prev);
        } else if tp.is_null() {
            // wp was the last window
            nvim_set_lastwin(prev);
        } else {
            nvim_tabpage_set_lastwin(tp, prev);
        }
    }
}

/// FFI wrapper for `win_remove`.
///
/// Removes a window from the window list.
#[no_mangle]
pub extern "C" fn rs_win_remove(wp: WinHandle, tp: TabpageHandle) {
    unsafe {
        debug_assert!(tp.is_null() || tp != nvim_get_curtab());
    }
    win_remove_impl(wp, tp);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_handle_null_check() {
        let null_handle = WinHandle::null();
        assert!(null_handle.is_null());
    }
}
