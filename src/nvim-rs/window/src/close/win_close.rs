//! Orchestrator helpers for `win_close`.
//!
//! The C function remains the orchestrator for autocmd-heavy sections
//! (BufLeave, WinLeave, WinClosed, win_close_buffer, getout, re-validations).
//! Three chunks of pure structural/layout logic are delegated to Rust:
//!
//! 1. [`rs_win_close_validate`] — initial validation checks
//! 2. [`rs_win_close_structural`] — UI cleanup, win_free_mem, cursor transfer
//! 3. [`rs_win_close_post_layout`] — post-close layout adjustment

use std::ffi::c_int;

use crate::{BufHandle, Frame, TabpageHandle, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // --- Accessors ---
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_lastwin() -> WinHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_first_tabpage() -> TabpageHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_curwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_locked(wp: WinHandle) -> c_int;
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_pvw(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_hide(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_focusable(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_external(wp: WinHandle) -> c_int;
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;

    // --- Existing Rust FFI helpers ---
    fn rs_last_window(win: WinHandle) -> c_int;
    fn rs_is_aucmd_win(wp: WinHandle) -> c_int;
    fn rs_one_window_in_tab(win: WinHandle, tp: TabpageHandle) -> c_int;
    fn rs_win_valid_any_tab(win: WinHandle) -> c_int;
    fn rs_bt_quickfix(buf: BufHandle) -> c_int;
    fn rs_get_snapshot_curwin(idx: c_int) -> WinHandle;
    fn rs_last_status(morewin: c_int);
    fn rs_win_comp_pos() -> c_int;

    // --- Phase 2 wrappers (reused) ---
    fn nvim_emsg_e_autocmd_close();
    fn nvim_emsg_e_floatonly();
    fn nvim_can_close_floating_windows(tp: TabpageHandle) -> c_int;

    // --- Phase 3 wrappers ---
    fn nvim_inc_split_disallowed();
    fn nvim_dec_split_disallowed();
    fn nvim_ui_has_multigrid() -> c_int;
    fn nvim_ui_call_win_close_win(wp: WinHandle);
    fn nvim_ui_comp_remove_grid_win(wp: WinHandle);
    fn nvim_tabpage_set_curwin(tp: TabpageHandle, wp: WinHandle);
    fn nvim_set_curwin(wp: WinHandle);
    fn nvim_set_curbuf_from_curwin();
    fn nvim_check_cursor_win_wrapper(wp: WinHandle);
    #[link_name = "rs_win_valid"]
    fn nvim_win_valid_wrapper(wp: WinHandle) -> c_int;
    // nvim_win_free_mem_wrapper removed: rs_win_close_structural now calls rs_win_free_mem directly (Phase 10)
    #[link_name = "rs_win_equal"]
    fn nvim_win_equal_wrapper(wp: WinHandle, current: c_int, dir: c_int);
    fn nvim_win_fix_scroll(upd_topline: bool);
    fn nvim_win_get_frame_parent(wp: WinHandle) -> *mut Frame;
    fn nvim_get_p_ea() -> c_int;
    fn nvim_get_p_ead_char() -> c_int;
}

// =============================================================================
// Constants
// =============================================================================

/// `SNAP_HELP_IDX` from window.c.
const SNAP_HELP_IDX: c_int = 0;

// =============================================================================
// Return type
// =============================================================================

/// Result of [`rs_win_close_structural`].
#[repr(C)]
pub struct WinCloseStructResult {
    /// New curwin candidate (from `win_free_mem` or cursor transfer).
    pub wp: WinHandle,
    /// 1 if curwin was the closed window.
    pub close_curwin: c_int,
    /// 1 if the closed window was floating.
    pub was_floating: c_int,
    /// Direction from `win_free_mem` ('v' or 'h').
    pub dir: c_int,
}

// =============================================================================
// Helper 1: Validation
// =============================================================================

/// Initial validation for `win_close`.
///
/// Returns:
/// - `0`: validation passed, proceed
/// - `1`: FAIL (cannot close: last window, locked, aucmd_win)
/// - `2`: floating-only, caller should do recursive close loop
/// - `3`: floating-only but `can_close_floating_windows` failed
///
/// # Safety
///
/// `win` must be a valid `win_T*`.
#[no_mangle]
pub extern "C" fn rs_win_close_validate(win: WinHandle, _free_buf: c_int, _force: c_int) -> c_int {
    unsafe {
        // E444: Cannot close last window.
        if rs_last_window(win) != 0 {
            return 1;
        }

        // Window or buffer locked.
        if nvim_win_get_locked(win) != 0 {
            return 1;
        }
        let buf = nvim_win_get_buffer(win);
        if !buf.is_null() && nvim_buf_get_locked(buf) != 0 {
            return 1;
        }

        // Cannot close aucmd window.
        if rs_is_aucmd_win(win) != 0 {
            return 1;
        }

        // Check if closing would leave only floating windows.
        let lastwin = nvim_get_lastwin();
        if nvim_win_get_floating(lastwin) != 0
            && rs_one_window_in_tab(win, TabpageHandle::null()) != 0
        {
            if rs_is_aucmd_win(lastwin) != 0 {
                // E814
                return 1;
            }
            // Signal caller: need to handle floating close.
            // Caller checks force || can_close_floating_windows.
            return 2;
        }

        0
    }
}

// =============================================================================
// Helper 2: Structural close
// =============================================================================

/// Core structural work for `win_close`: increment `split_disallowed`,
/// UI cleanup, external window fixup, `win_free_mem`, and cursor transfer.
///
/// This function must be called after all autocmd sections have completed
/// and re-validations have passed.
///
/// # Safety
///
/// `win` must be a valid `win_T*`. `win_frame` is the parent frame (may be NULL
/// for floating windows).
#[no_mangle]
pub extern "C" fn rs_win_close_structural(
    win: WinHandle,
    help_window: c_int,
    _win_frame: *mut Frame,
) -> WinCloseStructResult {
    unsafe {
        // Disallow splits while closing.
        nvim_inc_split_disallowed();

        let was_floating = nvim_win_get_floating(win) != 0;

        // UI notifications.
        if nvim_ui_has_multigrid() != 0 {
            nvim_ui_call_win_close_win(win);
        }

        // Floating window cleanup.
        if was_floating {
            nvim_ui_comp_remove_grid_win(win);
            // Fix external window curwin references in other tabpages.
            if nvim_win_get_config_external(win) != 0 {
                fixup_external_curwin(win);
            }
        }

        // Free the memory and get the window that received the space.
        let mut dir: c_int = 0;
        let mut wp = crate::close::helpers::rs_win_free_mem(
            win,
            std::ptr::addr_of_mut!(dir),
            TabpageHandle::null(),
        );

        // For help windows, try restoring snapshot curwin.
        if help_window != 0 {
            let prev_win = rs_get_snapshot_curwin(SNAP_HELP_IDX);
            if nvim_win_valid_wrapper(prev_win) != 0 {
                wp = prev_win;
            }
        }

        // Cursor transfer: if we closed curwin, pick a new one.
        let close_curwin = win == nvim_get_curwin();
        if close_curwin {
            nvim_set_curwin(wp);

            // If cursor goes to preview or quickfix window, find another.
            if nvim_win_get_pvw(wp) != 0 || rs_bt_quickfix(nvim_win_get_buffer(wp)) != 0 {
                wp = find_non_preview_quickfix_win(wp);
                nvim_set_curwin(wp);
            }

            nvim_set_curbuf_from_curwin();

            // Cursor position may be invalid if the buffer changed.
            let curwin = nvim_get_curwin();
            nvim_check_cursor_win_wrapper(curwin);
        }

        WinCloseStructResult {
            wp,
            close_curwin: c_int::from(close_curwin),
            was_floating: c_int::from(was_floating),
            dir,
        }
    }
}

/// Fix external window curwin references in other tabpages.
///
/// When closing an external floating window that is `tp_curwin` for some
/// other tabpage, reset that tabpage's `tp_curwin` to its `tp_firstwin`.
///
/// This replaces C `nvim_fixup_external_curwin` (Phase 8).
pub(crate) unsafe fn fixup_external_curwin(win: WinHandle) {
    let curtab = nvim_get_curtab();
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        if tp != curtab && nvim_tabpage_get_curwin(tp) == win {
            let firstwin = nvim_tabpage_get_firstwin(tp);
            nvim_tabpage_set_curwin(tp, firstwin);
        }
        tp = nvim_tabpage_get_next(tp);
    }
}

/// Find a window that is not a preview or quickfix window and is
/// not a hidden/unfocusable float. Used for cursor transfer.
unsafe fn find_non_preview_quickfix_win(start: WinHandle) -> WinHandle {
    let firstwin = nvim_get_firstwin();
    let mut wp = start;

    loop {
        let next = nvim_win_get_next(wp);
        wp = if next.is_null() { firstwin } else { next };

        if wp == start {
            // Wrapped around, give up.
            break;
        }

        if nvim_win_get_pvw(wp) != 0 {
            continue;
        }
        if rs_bt_quickfix(nvim_win_get_buffer(wp)) != 0 {
            continue;
        }
        if nvim_win_get_floating(wp) != 0
            && (nvim_win_get_config_hide(wp) != 0 || nvim_win_get_config_focusable(wp) == 0)
        {
            continue;
        }

        // Found a suitable window.
        return wp;
    }

    start
}

// =============================================================================
// Helper 3: Post-layout adjustment
// =============================================================================

/// Adjust layout after closing a window.
///
/// For non-floating windows: run `last_status`, then either equalize
/// windows or just recompute positions + fix scroll.
///
/// # Safety
///
/// Must be called after `rs_win_close_structural`.
#[no_mangle]
pub extern "C" fn rs_win_close_post_layout(was_floating: c_int, dir: c_int, win_frame: *mut Frame) {
    if was_floating != 0 {
        return;
    }

    unsafe {
        // If last window has a status line now and we don't want one,
        // remove it. Do this before win_equal().
        rs_last_status(0); // morewin = false

        let curwin = nvim_get_curwin();
        if nvim_win_get_floating(curwin) == 0 && nvim_get_p_ea() != 0 {
            let ead = nvim_get_p_ead_char();
            if ead == i32::from(b'b') || ead == dir {
                // If the frame of the closed window contains the new current window,
                // only resize that frame.
                let cur_parent = nvim_win_get_frame_parent(curwin);
                let current = (!win_frame.is_null())
                    && (!cur_parent.is_null())
                    && std::ptr::eq(cur_parent, win_frame);
                nvim_win_equal_wrapper(curwin, c_int::from(current), dir);
            } else {
                rs_win_comp_pos();
                nvim_win_fix_scroll(false);
            }
        } else {
            rs_win_comp_pos();
            nvim_win_fix_scroll(false);
        }
    }
}
