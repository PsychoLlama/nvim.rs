//! Window deallocation functions.
//!
//! This module provides Rust implementations of `win_free` and `win_free_grid`.

use std::ffi::c_int;

use crate::{TabpageHandle, WinHandle};

// Imports for inlined compound accessors (Phase 13)
use crate::list::{nvim_get_first_tabpage, nvim_tabpage_get_next};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // --- Compound accessors (Phase 12) ---

    /// pmap_del the window handle from window_handles.
    fn nvim_win_pmap_del(wp: WinHandle);

    /// rs_clearFolding.
    fn rs_clearFolding(wp: WinHandle);

    /// alist_unlink for the window's argument list.
    fn nvim_win_alist_unlink(wp: WinHandle);

    /// Block autocmds.
    fn nvim_block_autocmds();

    /// Destroy the w_ns_set.
    fn nvim_win_clear_ns_set(wp: WinHandle);

    /// Clear both winopt structs.
    fn nvim_win_clear_winopts(wp: WinHandle);

    /// Free lcs_chars multispace/leadmultispace.
    fn nvim_win_free_lcs_chars(wp: WinHandle);

    /// Free all w: variables.
    fn nvim_win_clear_vars(wp: WinHandle);

    /// Free w_lines.
    fn nvim_win_free_lines(wp: WinHandle);

    /// Clear the tagstack entries.
    fn nvim_win_clear_tagstack(wp: WinHandle);

    /// Free w_localdir and w_prevdir.
    fn nvim_win_free_dirs(wp: WinHandle);

    /// Clear all three click_defs arrays.
    fn nvim_win_clear_click_defs_all(wp: WinHandle);

    /// Remove window from all b_wininfo kvecs.
    fn nvim_win_cleanup_b_wininfo(wp: WinHandle);

    /// Clear border text virttext.
    fn nvim_win_clear_config_virttext(wp: WinHandle);

    /// Free w_p_cc_cols.
    fn nvim_win_free_cc_cols(wp: WinHandle);

    /// grid_free for the window's grid.
    fn nvim_win_grid_free(wp: WinHandle);

    /// CLEAR_FIELD the window's grid (for reinit).
    fn nvim_win_grid_clear_field(wp: WinHandle);

    /// Unblock autocmds.
    fn nvim_unblock_autocmds();

    /// Check if win is valid in any tab.
    fn rs_win_valid_any_tab(wp: WinHandle) -> c_int;

    /// Remove window from the window list.
    fn rs_win_remove(wp: WinHandle, tp: TabpageHandle);

    // --- Phase 13: Inlined compound accessor helpers ---

    /// Get the prevwin global.
    fn nvim_get_prevwin() -> WinHandle;

    /// Set the prevwin global.
    fn nvim_set_prevwin(wp: WinHandle);

    /// Get tp->tp_prevwin.
    fn nvim_tabpage_get_prevwin(tp: TabpageHandle) -> WinHandle;

    /// Set tp->tp_prevwin.
    fn nvim_tabpage_set_prevwin(tp: TabpageHandle, wp: WinHandle);

    /// Get the win's grid alloc handle.
    fn nvim_win_get_grid_alloc_handle(wp: WinHandle) -> c_int;

    /// Check if kUIMultigrid is active.
    fn nvim_ui_has_multigrid() -> c_int;

    /// Call ui_call_grid_destroy with a raw handle.
    fn nvim_ui_call_grid_destroy_handle(handle: c_int);

    /// clear_matches(wp).
    fn nvim_clear_matches_win(wp: WinHandle);

    /// free_jumplist(wp).
    fn nvim_free_jumplist_win(wp: WinHandle);

    /// qf_free_all(wp) -- from quickfix_shim.c, takes void*.
    fn nvim_qf_free_all_win(wp: *mut std::ffi::c_void);

    /// Get au_pending_free_win global.
    fn nvim_get_au_pending_free_win() -> WinHandle;

    /// Set au_pending_free_win global.
    fn nvim_set_au_pending_free_win(wp: WinHandle);

    /// Whether autocmd_busy is set.
    fn nvim_get_autocmd_busy() -> bool;

    /// Set wp->w_next.
    fn nvim_win_set_next(wp: WinHandle, next: WinHandle);

    /// xfree(wp) -- raw deallocation of a window struct.
    fn nvim_xfree_win_raw(wp: WinHandle);
}

// =============================================================================
// rs_win_free_grid
// =============================================================================

/// Free (and optionally reinitialize) a window's grid.
///
/// Port of C `win_free_grid()`.
///
/// # Safety
/// `wp` must be a valid `win_T*`.
unsafe fn win_free_grid_impl(wp: WinHandle, reinit: bool) {
    // Inlined nvim_win_grid_destroy: conditionally call ui_call_grid_destroy
    let grid_handle = nvim_win_get_grid_alloc_handle(wp);
    if grid_handle != 0 && nvim_ui_has_multigrid() != 0 {
        nvim_ui_call_grid_destroy_handle(grid_handle);
    }
    nvim_win_grid_free(wp);
    if reinit {
        nvim_win_grid_clear_field(wp);
    }
}

/// FFI export for `win_free_grid`.
///
/// # Safety
/// `wp` must be a valid `win_T*`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_free_grid(wp: WinHandle, reinit: c_int) {
    win_free_grid_impl(wp, reinit != 0);
}

// =============================================================================
// rs_win_free
// =============================================================================

/// Remove window `wp` from the window list and free its memory.
///
/// Port of C `win_free()`.
///
/// # Safety
/// `wp` must be a valid `win_T*`. `tp` must be NULL or a valid tabpage (not
/// current tabpage).
unsafe fn win_free_impl(wp: WinHandle, tp: TabpageHandle) {
    nvim_win_pmap_del(wp);
    rs_clearFolding(wp);

    // Reduce the reference count to the argument list.
    nvim_win_alist_unlink(wp);

    // Don't execute autocommands while the window is halfway being deleted.
    nvim_block_autocmds();

    nvim_win_clear_ns_set(wp);
    nvim_win_clear_winopts(wp);
    nvim_win_free_lcs_chars(wp);

    // Free all w: variables.
    nvim_win_clear_vars(wp);

    // Inlined nvim_win_fix_prevwin: NULL out prevwin==wp, tp_prevwin across all tabs.
    if nvim_get_prevwin() == wp {
        nvim_set_prevwin(WinHandle::null());
    }
    let mut ttp = nvim_get_first_tabpage();
    while !ttp.is_null() {
        if nvim_tabpage_get_prevwin(ttp) == wp {
            nvim_tabpage_set_prevwin(ttp, WinHandle::null());
        }
        ttp = nvim_tabpage_get_next(ttp);
    }

    nvim_win_free_lines(wp);
    nvim_win_clear_tagstack(wp);
    nvim_win_free_dirs(wp);
    nvim_win_clear_click_defs_all(wp);

    // Remove the window from the b_wininfo lists.
    nvim_win_cleanup_b_wininfo(wp);

    // Free the border text.
    nvim_win_clear_config_virttext(wp);

    // Inlined nvim_win_clear_matches_and_jumplist.
    nvim_clear_matches_win(wp);
    nvim_free_jumplist_win(wp);
    nvim_qf_free_all_win(wp.as_ptr());

    nvim_win_free_cc_cols(wp);

    win_free_grid_impl(wp, false);

    if rs_win_valid_any_tab(wp) != 0 {
        rs_win_remove(wp, tp);
    }

    // Inlined nvim_win_handle_pending_free: chain wp or xfree.
    if nvim_get_autocmd_busy() {
        nvim_win_set_next(wp, nvim_get_au_pending_free_win());
        nvim_set_au_pending_free_win(wp);
    } else {
        nvim_xfree_win_raw(wp);
    }

    nvim_unblock_autocmds();
}

/// FFI export for `win_free`.
///
/// # Safety
/// `wp` must be a valid `win_T*`. `tp` is the tabpage `wp` is in, or NULL for
/// the current tabpage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_free(wp: WinHandle, tp: TabpageHandle) {
    win_free_impl(wp, tp);
}

/// C export: `win_free` — eliminates the C thin wrapper.
///
/// # Safety
/// `wp` must be a valid `win_T*`. `tp` is the tabpage `wp` is in, or NULL for
/// the current tabpage.
#[unsafe(export_name = "win_free")]
pub unsafe extern "C" fn win_free(wp: WinHandle, tp: TabpageHandle) {
    win_free_impl(wp, tp);
}

// =============================================================================
// rs_win_free_all (EXITFREE)
// =============================================================================

extern "C" {
    /// Clear cmdwin state (for win_free_all).
    fn nvim_clear_cmdwin_state();

    /// tabpage_close(true) -- close a tab during EXITFREE.
    fn nvim_tabpage_close_once();

    /// Returns 1 if first_tabpage->tp_next != NULL.
    fn nvim_first_tabpage_has_next() -> c_int;

    /// lastwin pointer (0 = NULL).
    fn nvim_get_lastwin() -> WinHandle;

    /// Returns 1 if wp->w_floating.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// rs_win_free_mem(wp, &dummy, NULL) -- returns dummy direction.
    fn rs_win_free_mem(wp: WinHandle, dirp: *mut c_int, tp: TabpageHandle) -> WinHandle;

    /// firstwin pointer (0 = NULL).
    fn nvim_get_firstwin() -> WinHandle;

    /// Returns AUCMD_WIN_COUNT.
    fn nvim_aucmd_win_count() -> c_int;

    /// Returns aucmd_win[idx].auc_win.
    fn nvim_aucmd_win_get(idx: c_int) -> WinHandle;

    /// aucmd_win[idx].auc_win = NULL.
    fn nvim_aucmd_win_clear(idx: c_int);

    /// kv_destroy(aucmd_win_vec).
    fn nvim_kv_destroy_aucmd_win_vec();

    /// curwin = NULL.
    fn nvim_set_curwin_null();
}

/// Free all windows on exit (EXITFREE).
///
/// Port of C `win_free_all()`.
///
/// # Safety
/// Must only be called from the EXITFREE path.
unsafe fn win_free_all_impl() {
    // Avoid an error for switching tabpage with the cmdline window open.
    nvim_clear_cmdwin_state();

    while nvim_first_tabpage_has_next() != 0 {
        nvim_tabpage_close_once();
    }

    loop {
        let lw = nvim_get_lastwin();
        if lw.is_null() || nvim_win_get_floating(lw) == 0 {
            break;
        }
        rs_win_remove(lw, TabpageHandle::null());
        let mut dummy: c_int = 0;
        rs_win_free_mem(lw, std::ptr::addr_of_mut!(dummy), TabpageHandle::null());
        let count = nvim_aucmd_win_count();
        for i in 0..count {
            if nvim_aucmd_win_get(i) == lw {
                nvim_aucmd_win_clear(i);
            }
        }
    }

    let count = nvim_aucmd_win_count();
    for i in 0..count {
        let aw = nvim_aucmd_win_get(i);
        if !aw.is_null() {
            let mut dummy: c_int = 0;
            rs_win_free_mem(aw, std::ptr::addr_of_mut!(dummy), TabpageHandle::null());
            nvim_aucmd_win_clear(i);
        }
    }

    nvim_kv_destroy_aucmd_win_vec();

    loop {
        let fw = nvim_get_firstwin();
        if fw.is_null() {
            break;
        }
        let mut dummy: c_int = 0;
        rs_win_free_mem(fw, std::ptr::addr_of_mut!(dummy), TabpageHandle::null());
    }

    // No window should be used after this.
    nvim_set_curwin_null();
}

/// FFI export for `win_free_all`.
///
/// # Safety
/// Must only be called from the EXITFREE path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_free_all() {
    win_free_all_impl();
}
