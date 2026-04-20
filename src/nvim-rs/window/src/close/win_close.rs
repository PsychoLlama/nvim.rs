//! Orchestrator helpers for `win_close`.
//!
//! This module provides the consolidated `rs_win_close` entry point as well as
//! the three helper functions that remain exported for legacy callers:
//!
//! 1. [`rs_win_close_validate`] — initial validation checks
//! 2. [`rs_win_close_structural`] — UI cleanup, win_free_mem, cursor transfer
//! 3. [`rs_win_close_post_layout`] — post-close layout adjustment
//! 4. [`rs_win_close`] — consolidated entry point replacing the C `win_close` body

use std::ffi::c_int;

use crate::win_struct::{win_mut, win_ref};
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
    fn nvim_win_get_config_hide(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_focusable(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_external(wp: WinHandle) -> c_int;
    fn nvim_buf_get_locked(buf: BufHandle) -> c_int;
    fn nvim_redraw_all_later(type_: c_int);

    // --- Existing Rust FFI helpers ---
    fn rs_last_window(win: WinHandle) -> c_int;
    #[link_name = "is_aucmd_win"]
    fn rs_is_aucmd_win(wp: WinHandle) -> c_int;
    fn rs_one_window_in_tab(win: WinHandle, tp: TabpageHandle) -> c_int;
    fn rs_win_valid_any_tab(win: WinHandle) -> c_int;
    fn rs_bt_quickfix(buf: BufHandle) -> bool;
    fn rs_bt_help(buf: BufHandle) -> bool;
    fn rs_get_snapshot_curwin(idx: c_int) -> WinHandle;
    fn rs_last_status(morewin: c_int);
    fn rs_win_comp_pos() -> c_int;
    fn rs_diffopt_closeoff() -> c_int;
    fn rs_do_autocmd_winclosed(win: WinHandle);
    fn rs_reset_VIsual_and_resel();
    fn rs_win_altframe(win: WinHandle) -> *mut Frame;
    fn rs_frame2win(frp: *mut Frame) -> WinHandle;
    fn rs_clear_snapshot(tp: TabpageHandle, idx: c_int);
    fn rs_restore_snapshot(idx: c_int, close_curwin: c_int);

    // --- Phase 2 wrappers (reused) ---
    fn nvim_emsg_id(id: c_int);
    #[link_name = "rs_can_close_floating_windows_tp"]
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
    #[link_name = "rs_win_equal"]
    fn nvim_win_equal_wrapper(wp: WinHandle, current: c_int, dir: c_int);
    fn win_fix_scroll(upd_topline: c_int);
    fn nvim_win_get_frame_parent(wp: WinHandle) -> *mut Frame;
    fn nvim_get_p_ea() -> c_int;
    fn nvim_get_p_ead_char() -> c_int;

    // --- Phase 11 new accessors ---
    fn nvim_win_close_force(wp: WinHandle, free_buf: c_int) -> c_int;
    fn nvim_getout_zero() -> !;
    fn nvim_do_cmdline_cmd_diffoff();
    fn nvim_one_window_and_locked_split() -> c_int;
    fn rs_win_close_othertab(
        wp: WinHandle,
        free_buf: c_int,
        tp: TabpageHandle,
        force: c_int,
    ) -> c_int;
    fn win_float_find_altwin(win: WinHandle, tp: TabpageHandle) -> WinHandle;
    fn nvim_apply_autocmds_event(event: std::ffi::c_int);
    fn aborting() -> c_int;
}

// =============================================================================
// Constants
// =============================================================================

/// EVENT_* constants matching auevents_enum.generated.h
const EVENT_BUFENTER: c_int = 3;
const EVENT_BUFLEAVE: c_int = 7;
const EVENT_TABLEAVE: c_int = 111;
const EVENT_WINLEAVE: c_int = 137;

/// EMSG_* IDs for nvim_emsg_id dispatcher (must match window_shim.c nvim_emsg_id).
const EMSG_E444: c_int = 0;
const EMSG_E814: c_int = 1;
const EMSG_E_AUTOCMD_CLOSE: c_int = 10;
const EMSG_E_FLOATONLY: c_int = 8;

/// `SNAP_HELP_IDX` from window.c.
const SNAP_HELP_IDX: c_int = 0;

/// `UPD_NOT_VALID` from drawscreen.h (value 40, verified by static assert in C).
const UPD_NOT_VALID: c_int = 40;

/// `DOBUF_UNLOAD` from buffer.h (value 2).
const DOBUF_UNLOAD: c_int = 2;

/// `WEE_CURWIN_INVALID | WEE_TRIGGER_ENTER_AUTOCMDS | WEE_TRIGGER_LEAVE_AUTOCMDS`
/// (0x02 | 0x08 | 0x10 = 0x1a = 26)
const WEE_CLOSE_FLAGS: c_int = 0x02 | 0x08 | 0x10;

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
// Consolidated entry point: rs_win_close
// =============================================================================

/// Consolidated Rust replacement for C `win_close`.
///
/// This function absorbs the entire `win_close` body (validation, autocmd
/// sections, structural close, post-layout, post-close autocmds).
///
/// Returns OK (0) on success, FAIL (1) on failure.
///
/// # Safety
///
/// `win` must be a valid `win_T*`.
#[allow(clippy::must_use_candidate)]
#[allow(clippy::too_many_lines)]
#[export_name = "win_close"]
pub unsafe extern "C" fn rs_win_close(win: WinHandle, free_buf: c_int, force: c_int) -> c_int {
    let prev_curtab = nvim_get_curtab();
    let win_floating = win_ref(win).w_floating;
    let win_frame: *mut Frame = if win_floating {
        std::ptr::null_mut()
    } else {
        nvim_win_get_frame_parent(win)
    };
    let had_diffmode = win_ref(win).w_p_diff() != 0;

    // --- Phase 1: Validation ---
    let vrc = rs_win_close_validate(win, free_buf, force);
    if vrc == 1 {
        // Specific error messages for last_window, locked, aucmd_win, E814.
        // Re-check conditions to emit the right message (mirrors C logic).
        if rs_last_window(win) != 0 {
            nvim_emsg_id(EMSG_E444);
        } else if rs_is_aucmd_win(win) != 0 {
            nvim_emsg_id(EMSG_E_AUTOCMD_CLOSE);
        } else {
            let lastwin = nvim_get_lastwin();
            if win_ref(lastwin).w_floating
                && rs_one_window_in_tab(win, TabpageHandle::null()) != 0
                && rs_is_aucmd_win(lastwin) != 0
            {
                nvim_emsg_id(EMSG_E814);
            }
            // else: locked window -- no error message (silent FAIL)
        }
        return 1; // FAIL
    }
    if vrc == 2 {
        // Floating-only: recursive close loop.
        if force != 0 || nvim_can_close_floating_windows(TabpageHandle::null()) != 0 {
            loop {
                let lastwin = nvim_get_lastwin();
                if c_int::from(win_ref(lastwin).w_floating) == 0 {
                    break;
                }
                if nvim_win_close_force(lastwin, free_buf) != 0 {
                    return 1; // FAIL
                }
            }
            if rs_win_valid_any_tab(win) == 0 {
                return 1; // FAIL
            }
        } else {
            nvim_emsg_id(EMSG_E_FLOATONLY);
            return 1; // FAIL
        }
    }

    // close_last_window_tabpage.
    if crate::close::helpers::rs_close_last_window_tabpage(win, free_buf, prev_curtab) != 0 {
        return 1; // FAIL
    }

    // --- Autocmd-heavy section ---

    let help_window = rs_bt_help(BufHandle(win_ref(win).w_buffer));
    if !help_window {
        rs_clear_snapshot(nvim_get_curtab(), SNAP_HELP_IDX);
    }

    let mut other_buffer = false;

    if nvim_get_curwin() == win {
        crate::focus::rs_leaving_window(win);

        // Find the alternate window.
        let wp_alt: WinHandle = if win_ref(win).w_floating {
            win_float_find_altwin(win, TabpageHandle::null())
        } else {
            rs_frame2win(rs_win_altframe(win))
        };

        // Check if alternate window has different buffer from curbuf.
        let alt_buf = BufHandle(win_ref(wp_alt).w_buffer);
        if alt_buf != nvim_get_curbuf_c() {
            // alt buffer differs from curbuf
            rs_reset_VIsual_and_resel();
            other_buffer = true;

            if nvim_win_valid_wrapper(win) == 0 {
                return 1; // FAIL
            }
            win_mut(win).w_locked = true;
            nvim_apply_autocmds_event(EVENT_BUFLEAVE);
            if nvim_win_valid_wrapper(win) == 0 {
                return 1; // FAIL
            }
            win_mut(win).w_locked = false;
            if rs_last_window(win) != 0 {
                return 1; // FAIL
            }
        }

        win_mut(win).w_locked = true;
        nvim_apply_autocmds_event(EVENT_WINLEAVE);
        if nvim_win_valid_wrapper(win) == 0 {
            return 1; // FAIL
        }
        win_mut(win).w_locked = false;
        if rs_last_window(win) != 0 {
            return 1; // FAIL
        }
        if aborting() != 0 {
            return 1; // FAIL
        }
    }

    rs_do_autocmd_winclosed(win);
    if rs_win_valid_any_tab(win) == 0 {
        return 0; // OK (window already gone)
    }

    crate::close::helpers::rs_win_close_buffer(
        win,
        if free_buf != 0 { DOBUF_UNLOAD } else { 0 },
        1,
    );

    // Getout edge case: last non-float window with NULL buffer.
    if nvim_win_valid_wrapper(win) != 0
        && BufHandle(win_ref(win).w_buffer).is_null()
        && c_int::from(win_ref(win).w_floating) == 0
        && rs_last_window(win) != 0
    {
        let curwin = nvim_get_curwin();
        if BufHandle(win_ref(curwin).w_buffer).is_null() {
            win_mut(curwin).w_buffer = nvim_get_curbuf_c().0;
        }
        nvim_getout_zero(); // never returns
    }

    // Cross-tabpage redirect: if win moved to another tab after close_buffer.
    let curtab_now = nvim_get_curtab();
    if curtab_now != prev_curtab
        && rs_win_valid_any_tab(win) != 0
        && BufHandle(win_ref(win).w_buffer).is_null()
    {
        rs_win_close_othertab(win, 0, prev_curtab, force);
        return 1; // FAIL
    }

    // Post-autocmd re-validation.
    if nvim_win_valid_wrapper(win) == 0
        || (c_int::from(win_ref(win).w_floating) == 0 && rs_last_window(win) != 0)
        || crate::close::helpers::rs_close_last_window_tabpage(win, free_buf, prev_curtab) != 0
    {
        return 1; // FAIL
    }

    // --- Phase 2: Structural close ---
    let res = rs_win_close_structural(win, c_int::from(help_window), win_frame);
    let wp = res.wp;

    // --- Phase 3: Post-layout ---
    rs_win_close_post_layout(res.was_floating, res.dir, win_frame);

    // --- Post-close autocmds ---
    if res.close_curwin != 0 {
        crate::enter::rs_win_enter_ext(wp, WEE_CLOSE_FLAGS);
        if other_buffer {
            nvim_apply_autocmds_event(EVENT_BUFENTER);
        }
    }

    if nvim_one_window_and_locked_split() != 0 {
        nvim_apply_autocmds_event(EVENT_TABLEAVE);
    }

    nvim_dec_split_disallowed();

    if help_window {
        rs_restore_snapshot(SNAP_HELP_IDX, res.close_curwin);
    }

    // Diff closeoff handling.
    if rs_diffopt_closeoff() != 0 && had_diffmode && nvim_get_curtab() == prev_curtab {
        let mut diffcount: c_int = 0;
        let mut dwin = nvim_get_firstwin();
        while !dwin.is_null() {
            if win_ref(dwin).w_p_diff() != 0 {
                diffcount += 1;
            }
            dwin = win_ref(dwin).w_next;
        }
        if diffcount == 1 {
            nvim_do_cmdline_cmd_diffoff();
        }
    }

    let curwin = nvim_get_curwin();
    win_mut(curwin).w_pos_changed = true;
    if res.was_floating == 0 {
        nvim_redraw_all_later(UPD_NOT_VALID);
    }

    0 // OK
}

/// Helper: get curbuf (needed inline without re-exporting).
#[inline]
unsafe fn nvim_get_curbuf_c() -> BufHandle {
    extern "C" {
        fn nvim_get_curbuf() -> BufHandle;
    }
    nvim_get_curbuf()
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
        if win_ref(win).w_locked {
            return 1;
        }
        let buf = BufHandle(win_ref(win).w_buffer);
        if !buf.is_null() && nvim_buf_get_locked(buf) != 0 {
            return 1;
        }

        // Cannot close aucmd window.
        if rs_is_aucmd_win(win) != 0 {
            return 1;
        }

        // Check if closing would leave only floating windows.
        let lastwin = nvim_get_lastwin();
        if win_ref(lastwin).w_floating && rs_one_window_in_tab(win, TabpageHandle::null()) != 0 {
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

        let was_floating = win_ref(win).w_floating;

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
            if win_ref(wp).w_p_pvw() != 0 || rs_bt_quickfix(BufHandle(win_ref(wp).w_buffer)) {
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
        let next = win_ref(wp).w_next;
        wp = if next.is_null() { firstwin } else { next };

        if wp == start {
            // Wrapped around, give up.
            break;
        }

        if win_ref(wp).w_p_pvw() != 0 {
            continue;
        }
        if rs_bt_quickfix(BufHandle(win_ref(wp).w_buffer)) {
            continue;
        }
        if win_ref(wp).w_floating
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
        if c_int::from(win_ref(curwin).w_floating) == 0 && nvim_get_p_ea() != 0 {
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
                win_fix_scroll(0);
            }
        } else {
            rs_win_comp_pos();
            win_fix_scroll(0);
        }
    }
}
