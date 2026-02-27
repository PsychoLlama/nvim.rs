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

use std::ffi::c_int;

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

    /// Set w_lines_valid field.
    fn nvim_win_set_lines_valid(wp: WinHandle, val: c_int);

    /// Set cursor lnum (linenr_T = i32).
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: LinenrT);

    /// Set cursor col (colnr_T = c_int).
    fn nvim_win_set_cursor_col(wp: WinHandle, col: c_int);

    /// Set cursor coladd (colnr_T = c_int).
    fn nvim_win_set_cursor_coladd(wp: WinHandle, val: c_int);

    /// Set w_curswant (colnr_T = c_int).
    fn nvim_win_set_curswant(wp: WinHandle, val: c_int);

    /// Set w_topline (linenr_T = i32).
    fn nvim_win_set_topline(wp: WinHandle, val: LinenrT);

    /// Set w_topfill.
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int);

    /// Set w_botline (takes int in C, casts to linenr_T).
    fn nvim_win_set_botline(wp: WinHandle, val: c_int);

    /// Set w_valid.
    fn nvim_win_set_valid(wp: WinHandle, val: c_int);

    /// Set w_pcmark (lnum: linenr_T = i32, col: colnr_T = c_int).
    fn nvim_win_set_pcmark(wp: WinHandle, lnum: LinenrT, col: c_int);

    /// Set w_prev_pcmark (lnum: linenr_T = i32, col: colnr_T = c_int).
    fn nvim_win_set_prev_pcmark(wp: WinHandle, lnum: LinenrT, col: c_int);

    /// Sync w_s to point to the window buffer's b_s.
    fn nvim_win_sync_s(wp: WinHandle);
}

/// Initialize an empty window's cursor and scroll state.
///
/// Port of C `win_init_empty()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn win_init_empty_impl(wp: WinHandle) {
    nvim_redraw_later_wrapper(wp, UPD_NOT_VALID);
    nvim_win_set_lines_valid(wp, 0);
    nvim_win_set_cursor_lnum(wp, 1);
    nvim_win_set_cursor_col(wp, 0);
    nvim_win_set_cursor_coladd(wp, 0);
    nvim_win_set_curswant(wp, 0);
    nvim_win_set_pcmark(wp, 1, 0); // pcmark not cleared but set to line 1
    nvim_win_set_prev_pcmark(wp, 0, 0);
    nvim_win_set_topline(wp, 1);
    nvim_win_set_topfill(wp, 0);
    nvim_win_set_botline(wp, 2);
    nvim_win_set_valid(wp, 0);
    nvim_win_sync_s(wp);
}

/// FFI export for `win_init_empty`.
///
/// # Safety
/// Calls C accessor functions with a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_init_empty(wp: WinHandle) {
    win_init_empty_impl(wp);
}

// =============================================================================
// rs_new_frame -- allocate a frame_T and link it to a window
// =============================================================================

extern "C" {
    /// Allocate a raw frame_T via xcalloc.
    fn nvim_alloc_frame_raw() -> *mut crate::Frame;

    /// Set wp->w_frame.
    fn nvim_win_set_frame(wp: WinHandle, frp: *mut crate::Frame);
}

/// Allocate a new frame_T and link it to window `wp`.
///
/// Port of C `new_frame()`.
///
/// # Safety
/// Calls C accessor functions. `wp` must be a valid, non-null window.
unsafe fn new_frame_impl(wp: WinHandle) {
    let frp = nvim_alloc_frame_raw();
    (*frp).fr_layout = crate::FR_LEAF;
    (*frp).fr_win = wp;
    nvim_win_set_frame(wp, frp);
}

/// FFI export for `new_frame`.
///
/// Allocates a frame_T and links it to window `wp`.
///
/// # Safety
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
    fn nvim_block_autocmds();

    /// Unblock autocmds.
    fn nvim_unblock_autocmds();

    /// Find the tabpage containing `wp`.
    fn rs_win_find_tabpage(wp: WinHandle) -> TabpageHandle;

    /// Set w_wincol.
    fn nvim_win_set_wincol(wp: WinHandle, val: c_int);

    /// Set w_width (inner).
    fn nvim_win_set_field_width(wp: WinHandle, val: c_int);

    /// Get Columns global.
    fn nvim_get_Columns() -> c_int;

    /// Set w_scbind_pos.
    fn nvim_win_set_scbind_pos(wp: WinHandle, val: c_int);

    /// Set w_floating.
    fn nvim_win_set_floating(wp: WinHandle, val: c_int);

    /// Set w_config = WIN_CONFIG_INIT.
    fn nvim_win_set_config_init(wp: WinHandle);

    /// Set w_viewport_invalid.
    fn nvim_win_set_viewport_invalid(wp: WinHandle, val: c_int);

    /// Bulk-set w_viewport_last_* fields from a WinViewportSnapshot.
    fn nvim_win_set_viewport_snapshot(wp: WinHandle, s: *const WinViewportSnapshot);

    /// Initialize w_ns_set (SET_INIT) and w_ns_hl = -1.
    fn nvim_win_init_ns_set(wp: WinHandle);

    /// Set global-local scroll offset options to -1.
    fn nvim_win_init_global_local_opts(wp: WinHandle);

    /// Set w_fraction.
    fn nvim_win_set_fraction(wp: WinHandle, val: c_int);

    /// Set w_prev_fraction_row.
    fn nvim_win_set_prev_fraction_row(wp: WinHandle, val: c_int);

    /// Init fold state for window.
    fn rs_foldInitWin(wp: WinHandle);

    /// Set w_next_match_id = 1000.
    fn nvim_win_set_next_match_id(wp: WinHandle);

    /// Free a WinInfo (compound: clear_winopt + deleteFoldRecurse + xfree).
    fn nvim_free_wininfo_raw(wip: *mut std::ffi::c_void, bp: crate::BufHandle);
}

/// Allocate and initialize a new win_T.
///
/// Port of C `win_alloc()`.
///
/// # Safety
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

    nvim_win_set_wincol(wp, 0);
    nvim_win_set_field_width(wp, nvim_get_Columns());

    // Position display and cursor at top of file.
    nvim_win_set_topline(wp, 1);
    nvim_win_set_topfill(wp, 0);
    nvim_win_set_botline(wp, 2);
    nvim_win_set_cursor_lnum(wp, 1);
    nvim_win_set_scbind_pos(wp, 1);
    nvim_win_set_floating(wp, 0);
    nvim_win_set_config_init(wp);
    nvim_win_set_viewport_invalid(wp, 1);
    let vsnap = WinViewportSnapshot {
        topline: 1,
        ..WinViewportSnapshot::default()
    };
    nvim_win_set_viewport_snapshot(wp, std::ptr::addr_of!(vsnap));

    nvim_win_init_ns_set(wp);
    nvim_win_init_global_local_opts(wp);

    nvim_win_set_fraction(wp, 0);
    nvim_win_set_prev_fraction_row(wp, -1);

    rs_foldInitWin(wp);
    nvim_unblock_autocmds();
    nvim_win_set_next_match_id(wp);

    wp
}

/// FFI export for `win_alloc`.
///
/// Allocates and initializes a new win_T.
///
/// # Safety
/// `after` must be null or a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_alloc(after: WinHandle, hidden: c_int) -> WinHandle {
    win_alloc_impl(after, hidden != 0)
}

/// Free a WinInfo struct.
///
/// Port of C `free_wininfo()`.
///
/// # Safety
/// `wip` must be a valid, non-null WinInfo pointer. `bp` must be valid or null.
unsafe fn free_wininfo_impl(wip: *mut std::ffi::c_void, bp: crate::BufHandle) {
    nvim_free_wininfo_raw(wip, bp);
}

/// FFI export for `free_wininfo`.
///
/// Frees the WinInfo struct `wip`.
///
/// # Safety
/// `wip` must be a valid, non-null WinInfo pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_free_wininfo(wip: *mut std::ffi::c_void, bp: crate::BufHandle) {
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

/// Allocate the first window in a tabpage.
///
/// When `oldwin` is null, create an empty buffer.
/// When `oldwin` is non-null, copy info from it.
///
/// Port of C `win_alloc_firstwin()`.
///
/// Returns OK (1) or FAIL (0).
///
/// # Safety
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

/// FFI export for `win_alloc_firstwin`.
///
/// # Safety
/// `oldwin` must be null or a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_alloc_firstwin(oldwin: WinHandle) -> c_int {
    win_alloc_firstwin_impl(oldwin)
}

/// Allocate the first window and put an empty buffer in it.
///
/// Port of C `win_alloc_first()`. Only called from `main()`.
///
/// # Safety
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

/// FFI export for `win_alloc_first`.
///
/// # Safety
/// Must only be called once at startup.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_alloc_first() {
    win_alloc_first_impl();
}

/// Initialize `aucmd_win[idx]` as a hidden floating window.
///
/// Port of C `win_alloc_aucmd_win()`. Must only be called after
/// the first window is fully initialized.
///
/// # Safety
/// Must only be called after startup.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_alloc_aucmd_win(idx: c_int) {
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
