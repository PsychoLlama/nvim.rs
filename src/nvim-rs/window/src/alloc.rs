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

#![allow(clippy::missing_safety_doc)]

use std::ffi::c_int;

use crate::win_struct::{win_mut, win_ref};
use crate::{TabpageHandle, WinHandle};

// C accessor functions for window list manipulation.
extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    // Getters
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_lastwin() -> WinHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle;

    // Setters
    fn nvim_set_firstwin(wp: WinHandle);
    fn nvim_set_lastwin(wp: WinHandle);
    fn nvim_tabpage_set_firstwin(tp: TabpageHandle, wp: WinHandle);
    fn nvim_tabpage_set_lastwin(tp: TabpageHandle, wp: WinHandle);
}

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
            win_ref(after).w_next
        };

        // Link wp into the list
        win_mut(wp).w_next = before;
        win_mut(wp).w_prev = after;

        // Update previous link
        if after.is_null() {
            // wp becomes the new first window
            if tp.is_null() {
                nvim_set_firstwin(wp);
            } else {
                nvim_tabpage_set_firstwin(tp, wp);
            }
        } else {
            win_mut(after).w_next = wp;
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
            win_mut(before).w_prev = wp;
        }
    }
}

/// Appends a window in the window list after another window.
#[no_mangle]
pub extern "C" fn rs_win_append(after: WinHandle, wp: WinHandle, tp: TabpageHandle) {
    unsafe {
        debug_assert!(tp.is_null() || tp != nvim_get_curtab());
    }
    win_append_impl(after, wp, tp);
}

/// - `tp` must be NULL or a valid tabpage that is NOT the current tabpage
fn win_remove_impl(wp: WinHandle, tp: TabpageHandle) {
    // SAFETY: All accessor functions handle pointers safely.
    // The assertion from C (tp == NULL || tp != curtab) should be ensured by caller.
    unsafe {
        let prev = win_ref(wp).w_prev;
        let next = win_ref(wp).w_next;

        // Update previous window's next pointer
        if !prev.is_null() {
            win_mut(prev).w_next = next;
        } else if tp.is_null() {
            // wp was the first window
            nvim_set_firstwin(next);
        } else {
            nvim_tabpage_set_firstwin(tp, next);
        }

        // Update next window's prev pointer
        if !next.is_null() {
            win_mut(next).w_prev = prev;
        } else if tp.is_null() {
            // wp was the last window
            nvim_set_lastwin(prev);
        } else {
            nvim_tabpage_set_lastwin(tp, prev);
        }
    }
}

/// Removes a window from the window list.
#[no_mangle]
pub extern "C" fn rs_win_remove(wp: WinHandle, tp: TabpageHandle) {
    unsafe {
        debug_assert!(tp.is_null() || tp != nvim_get_curtab());
    }
    win_remove_impl(wp, tp);
}

// =============================================================================
// win_init_empty -- initialize an empty window's cursor/topline state
// =============================================================================

/// UPD_NOT_VALID value (buffer needs complete redraw).
const UPD_NOT_VALID: c_int = 40;

/// linenr_T is int32_t in C.
type LinenrT = i32;

/// Bulk viewport snapshot (matches C WinViewportSnapshot exactly).
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WinViewportSnapshot {
    topline: i32,
    botline: i32,
    topfill: i32,
    skipcol: i32,
}

extern "C" {
    /// Schedule a later redraw for the window.
    fn nvim_redraw_later_wrapper(wp: WinHandle, update_type: c_int);

    /// Sync w_s to point to the window buffer's b_s.
    fn nvim_win_sync_s(wp: WinHandle);
}

/// Calls C accessor functions.
unsafe fn win_init_empty_impl(wp: WinHandle) {
    nvim_redraw_later_wrapper(wp, UPD_NOT_VALID);
    win_mut(wp).w_lines_valid = 0;
    win_mut(wp).w_cursor.lnum = 1;
    win_mut(wp).w_cursor.col = 0;
    win_mut(wp).w_cursor.coladd = 0;
    win_mut(wp).w_curswant = 0;
    {
        let ws = win_mut(wp);
        ws.w_pcmark.lnum = 1;
        ws.w_pcmark.col = 0;
    }; // pcmark not cleared but set to line 1
    {
        let ws = win_mut(wp);
        ws.w_prev_pcmark.lnum = 0;
        ws.w_prev_pcmark.col = 0;
    };
    win_mut(wp).w_topline = 1;
    win_mut(wp).w_topfill = 0;
    win_mut(wp).w_botline = 2;
    win_mut(wp).w_valid = 0;
    nvim_win_sync_s(wp);
}

/// Calls C accessor functions with a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_init_empty(wp: WinHandle) {
    win_init_empty_impl(wp);
}

/// Calls C accessor functions with a valid window handle.
#[unsafe(export_name = "win_init_empty")]
pub unsafe extern "C" fn win_init_empty(wp: WinHandle) {
    win_init_empty_impl(wp);
}

/// Must be called from a valid initialized context.
#[unsafe(export_name = "curwin_init")]
pub unsafe extern "C" fn curwin_init() {
    let curwin = nvim_get_curwin();
    win_init_empty_impl(curwin);
}

// =============================================================================
// rs_new_frame -- allocate a frame_T and link it to a window
// =============================================================================

extern "C" {
    /// Allocate a raw frame_T via xcalloc.
    fn nvim_alloc_frame_raw() -> *mut crate::Frame;

}

/// Calls C accessor functions. `wp` must be a valid, non-null window.
unsafe fn new_frame_impl(wp: WinHandle) {
    let frp = nvim_alloc_frame_raw();
    (*frp).fr_layout = crate::FR_LEAF;
    (*frp).fr_win = wp;
    win_mut(wp).w_frame = frp;
}

/// `wp` must be a valid, non-null window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_new_frame(wp: WinHandle) {
    new_frame_impl(wp);
}

// =============================================================================
// rs_win_alloc + rs_free_wininfo -- Phase 2
// =============================================================================

extern "C" {
    /// Allocate raw win_T via xcalloc.
    fn nvim_alloc_win_raw() -> WinHandle;

    /// Init handle field and pmap_put into window_handles.
    fn nvim_win_init_handle(wp: WinHandle);

    /// Enable mouse on grid and assign a grid handle.
    fn nvim_win_init_grid(wp: WinHandle);

    /// Allocate w_vars dict and initialize w: variable scope.
    fn nvim_win_init_vars(wp: WinHandle);

    /// Block autocmds.
    #[link_name = "block_autocmds"]
    fn nvim_block_autocmds();

    /// Unblock autocmds.
    #[link_name = "unblock_autocmds"]
    fn nvim_unblock_autocmds();

    /// Find the tabpage containing `wp`.
    fn rs_win_find_tabpage(wp: WinHandle) -> TabpageHandle;

    /// Set w_config = WIN_CONFIG_INIT.
    fn nvim_win_set_config_init(wp: WinHandle);

    /// Bulk-set w_viewport_last_* fields from a WinViewportSnapshot.
    fn nvim_win_set_viewport_snapshot(wp: WinHandle, s: *const WinViewportSnapshot);

    /// Initialize w_ns_set (SET_INIT) and w_ns_hl = -1.
    fn nvim_win_init_ns_set(wp: WinHandle);

    /// Set global-local scroll offset options to -1.
    fn nvim_win_init_global_local_opts(wp: WinHandle);

    /// Init fold state for window.
    fn rs_foldInitWin(wp: WinHandle);

    /// Free a WinInfo (compound: clear_winopt + deleteFoldRecurse + xfree).
    fn nvim_free_wininfo_raw(wip: *mut std::ffi::c_void, bp: crate::BufHandle);
}

/// Calls C accessor functions. `after` must be null or a valid window.
unsafe fn win_alloc_impl(after: WinHandle, hidden: bool) -> WinHandle {
    let wp = nvim_alloc_win_raw();

    nvim_win_init_handle(wp);
    nvim_win_init_grid(wp);
    nvim_win_init_vars(wp);

    // Don't execute autocommands while the window is not properly initialized.
    nvim_block_autocmds();

    // Link the window into the window list unless hidden.
    if !hidden {
        let tp = if after.is_null() {
            TabpageHandle::null()
        } else {
            let tp = rs_win_find_tabpage(after);
            let curtab = nvim_get_curtab();
            if tp == curtab {
                TabpageHandle::null()
            } else {
                tp
            }
        };
        rs_win_append(after, wp, tp);
    }

    win_mut(wp).w_wincol = 0;
    win_mut(wp).w_width = Columns;

    // Position display and cursor at top of file.
    win_mut(wp).w_topline = 1;
    win_mut(wp).w_topfill = 0;
    win_mut(wp).w_botline = 2;
    win_mut(wp).w_cursor.lnum = 1;
    win_mut(wp).w_scbind_pos = 1;
    win_mut(wp).w_floating = false;
    nvim_win_set_config_init(wp);
    win_mut(wp).w_viewport_invalid = true;
    let vsnap = WinViewportSnapshot {
        topline: 1,
        ..WinViewportSnapshot::default()
    };
    nvim_win_set_viewport_snapshot(wp, std::ptr::addr_of!(vsnap));

    nvim_win_init_ns_set(wp);
    nvim_win_init_global_local_opts(wp);

    win_mut(wp).w_fraction = 0;
    win_mut(wp).w_prev_fraction_row = -1;

    rs_foldInitWin(wp);
    nvim_unblock_autocmds();
    crate::win_struct::win_mut(wp).w_next_match_id = 1000;

    wp
}

/// `after` must be null or a valid window handle.
#[allow(clippy::must_use_candidate)]
#[export_name = "win_alloc"]
pub unsafe extern "C" fn rs_win_alloc(after: WinHandle, hidden: bool) -> WinHandle {
    win_alloc_impl(after, hidden)
}

/// `wip` must be a valid, non-null WinInfo pointer. `bp` must be valid or null.
unsafe fn free_wininfo_impl(wip: *mut std::ffi::c_void, bp: crate::BufHandle) {
    nvim_free_wininfo_raw(wip, bp);
}

/// `wip` must be a valid, non-null WinInfo pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_free_wininfo(wip: *mut std::ffi::c_void, bp: crate::BufHandle) {
    free_wininfo_impl(wip, bp);
}

/// `wip` must be a valid, non-null WinInfo pointer.
#[unsafe(export_name = "free_wininfo")]
pub unsafe extern "C" fn free_wininfo(wip: *mut std::ffi::c_void, bp: crate::BufHandle) {
    free_wininfo_impl(wip, bp);
}

// =============================================================================
// Phase 4: rs_win_alloc_first + rs_win_alloc_firstwin + rs_win_alloc_aucmd_win
// =============================================================================

extern "C" {
    /// buflist_new(NULL, NULL, 1, BLN_LISTED): allocate initial buffer.
    fn nvim_buflist_new_initial() -> crate::BufHandle;

    /// Set curwin/curbuf and wire up the first buffer.
    fn nvim_win_setup_first_buffer(wp: WinHandle, buf: crate::BufHandle);

    /// curwin_init() for current window.
    fn nvim_curwin_init();

    /// RESET_BINDING for window wp.
    fn nvim_win_reset_binding(wp: WinHandle);

    /// Set topframe from wp->w_frame and compute frame dimensions.
    fn nvim_alloc_firstwin_set_topframe(wp: WinHandle);

    /// Set curwin = wp.
    fn nvim_set_curwin_to_wp(wp: WinHandle);

    /// curtab = tp.
    fn nvim_set_curtab(tp: TabpageHandle);

    /// first_tabpage = tp.
    fn nvim_set_first_tabpage(tp: TabpageHandle);

    /// unuse_tabpage(tp).
    fn rs_unuse_tabpage(tp: TabpageHandle);

    /// win_init(newp, oldp, 0).
    fn rs_win_init(wp: WinHandle, oldwin: WinHandle, flags: c_int);

    /// Allocate and initialize aucmd_win[idx] as a hidden float.
    fn nvim_win_alloc_aucmd_win_impl(idx: c_int);
}

// OK / FAIL constants (matching C).
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Calls C accessor functions. Must only be called at startup or tabnew time.
unsafe fn win_alloc_firstwin_impl(oldwin: WinHandle) -> c_int {
    let wp = win_alloc_impl(WinHandle::null(), false);

    if oldwin.is_null() {
        // Very first window: create an empty buffer.
        let buf = nvim_buflist_new_initial();
        if buf.is_null() {
            return FAIL;
        }
        nvim_win_setup_first_buffer(wp, buf);
        nvim_curwin_init();
    } else {
        // First window in a new tabpage: initialize from oldwin.
        nvim_set_curwin_to_wp(wp);
        rs_win_init(wp, oldwin, 0);
        // We don't want cursor- and scroll-binding in the first window.
        nvim_win_reset_binding(wp);
    }

    rs_new_frame(wp);
    nvim_alloc_firstwin_set_topframe(wp);

    OK
}

/// `oldwin` must be null or a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_alloc_firstwin(oldwin: WinHandle) -> c_int {
    win_alloc_firstwin_impl(oldwin)
}

/// Must only be called once at startup.
unsafe fn win_alloc_first_impl() {
    if win_alloc_firstwin_impl(WinHandle::null()) == FAIL {
        // Allocating first buffer before any autocmds should not fail.
        std::process::abort();
    }

    let tp = crate::tabpage::rs_alloc_tabpage();
    nvim_set_first_tabpage(tp);
    nvim_set_curtab(tp);
    rs_unuse_tabpage(tp);
}

/// Must only be called once at startup.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_alloc_first() {
    win_alloc_first_impl();
}

/// Must only be called once at startup.
#[unsafe(export_name = "win_alloc_first")]
pub unsafe extern "C" fn win_alloc_first() {
    win_alloc_first_impl();
}

/// Must only be called after startup.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_alloc_aucmd_win(idx: c_int) {
    nvim_win_alloc_aucmd_win_impl(idx);
}

/// Must only be called after startup.
#[unsafe(export_name = "win_alloc_aucmd_win")]
pub unsafe extern "C" fn win_alloc_aucmd_win(idx: c_int) {
    nvim_win_alloc_aucmd_win_impl(idx);
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
