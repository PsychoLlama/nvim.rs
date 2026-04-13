//! Implementation of changed_common: central change notification hub.
//!
//! This module migrates `changed_common` from C (src/nvim/change.c) to Rust.
//! It marks the buffer modified, updates the changelist/marks, invalidates
//! windows, and triggers redraw.

use std::ffi::{c_int, c_void};

use crate::{win_mut, win_ref, BufHandle, ColnrT, LinenrT, WinHandle};

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Buffer modification
    #[link_name = "changed"]
    fn rs_changed(buf: BufHandle);

    // Diff functions
    #[link_name = "rs_diff_internal"]
    fn nvim_diff_internal() -> c_int;
    #[link_name = "rs_diff_update_line"]
    fn nvim_diff_update_line(lnum: LinenrT);

    // curtab iteration (current tab windows only)
    fn nvim_curtab_first_win() -> WinHandle;
    fn nvim_win_get_next_in_tab(wp: WinHandle) -> WinHandle;

    // curtab diff update
    fn nvim_curtab_set_diff_update(val: bool);

    // Changelist/mark compound helper
    fn nvim_changed_common_update_changelist(buf: BufHandle, lnum: LinenrT, col: ColnrT);

    // KEEPJUMPS check
    fn nvim_cmod_keepjumps() -> bool;

    // Visual active + check visual pos
    fn nvim_get_VIsual_active_bool() -> bool;
    fn nvim_change_check_visual_pos();

    // Global state
    fn nvim_get_redraw_not_allowed() -> bool;

    // linetabsize_eol / sms_marker_overlap / set_topline wrappers
    fn nvim_change_linetabsize_eol(wp: WinHandle, lnum: LinenrT) -> c_int;
    fn nvim_change_sms_marker_overlap(wp: WinHandle, extra2: c_int) -> c_int;
    fn nvim_change_set_topline(wp: WinHandle, topline: LinenrT);

    // redraw_later
    #[link_name = "redraw_later"]
    fn nvim_redraw_later(wp: WinHandle, rtype: c_int);

    // Fold functions
    #[link_name = "rs_foldUpdate"]
    fn nvim_fold_update(wp: WinHandle, top: LinenrT, bot: LinenrT);
    fn hasFoldingWin(
        wp: WinHandle,
        lnum: LinenrT,
        firstp: *mut LinenrT,
        lastp: *mut LinenrT,
        cache: bool,
        infop: *mut c_void,
    ) -> bool;
    #[link_name = "rs_hasAnyFolding"]
    fn nvim_has_any_folding(wp: WinHandle) -> c_int;

    // Buffer accessor for window
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    // all-tab window iteration
    fn nvim_for_all_tab_windows_start() -> *mut c_void;
    fn nvim_for_all_tab_windows_next(iter: *mut c_void) -> WinHandle;
    fn nvim_for_all_tab_windows_end(iter: *mut c_void);

    // curwin/curbuf accessors
    fn nvim_get_curwin() -> WinHandle;

    // search_hl_has_cursor_lnum
    fn nvim_get_search_hl_has_cursor_lnum() -> LinenrT;
    fn nvim_set_search_hl_has_cursor_lnum(val: LinenrT);

    // set_must_redraw
    fn nvim_change_set_must_redraw(typ: c_int);

    // last_cursormoved reset check
    fn nvim_change_last_cursormoved_reset_check(
        buf: BufHandle,
        lnum: LinenrT,
        lnume: LinenrT,
        xtra: LinenrT,
    );
}

/// UPD_VALID redraw type constant.
const UPD_VALID: c_int = 10;

/// UPD_NOT_VALID redraw type constant.
const UPD_NOT_VALID: c_int = 20;

// =============================================================================
// Implementation
// =============================================================================

/// Common code for when a change was made.
///
/// See changed_lines() for the arguments. Careful: may trigger autocommands
/// that reload the buffer.
///
/// Migrated from change.c `changed_common()`.
pub fn changed_common_impl(
    buf: BufHandle,
    lnum: LinenrT,
    col: ColnrT,
    lnume: LinenrT,
    xtra: LinenrT,
) {
    // SAFETY: All extern calls are safe C functions with correct signatures.
    unsafe {
        // Mark the buffer as modified.
        rs_changed(buf);

        // FOR_ALL_WINDOWS_IN_TAB(curtab): update diff for current tab windows.
        let mut win = nvim_curtab_first_win();
        while !win.is_null() {
            if nvim_win_get_buffer(win) == buf
                && win_ref(win).w_p_diff() != 0
                && nvim_diff_internal() != 0
            {
                nvim_curtab_set_diff_update(true);
                nvim_diff_update_line(lnum);
            }
            win = nvim_win_get_next_in_tab(win);
        }

        // Set the '.' mark (unless keepjumps).
        if !nvim_cmod_keepjumps() {
            nvim_changed_common_update_changelist(buf, lnum, col);
        }

        // Check visual position if Visual mode is active in curwin on this buffer.
        let curwin = nvim_get_curwin();
        if nvim_win_get_buffer(curwin) == buf && nvim_get_VIsual_active_bool() {
            nvim_change_check_visual_pos();
        }

        // FOR_ALL_TAB_WINDOWS: iterate all windows across all tabs.
        let iter = nvim_for_all_tab_windows_start();
        loop {
            let wp = nvim_for_all_tab_windows_next(iter);
            if wp.is_null() {
                break;
            }

            if nvim_win_get_buffer(wp) == buf {
                // Mark this window to be redrawn later.
                if !nvim_get_redraw_not_allowed() && win_ref(wp).w_redr_type < UPD_VALID {
                    win_mut(wp).w_redr_type = UPD_VALID;
                }

                // When inserting/deleting lines and the window has specific lines
                // to be redrawn, w_redraw_top and w_redraw_bot may now be invalid,
                // so just redraw everything.
                if xtra != 0 && win_ref(wp).w_redraw_top != 0 {
                    nvim_redraw_later(wp, UPD_NOT_VALID);
                }

                let last = lnume + xtra - 1; // last line after the change

                // Reset "w_skipcol" if the topline length has become smaller to
                // such a degree that nothing will be visible anymore.
                let skipcol = win_ref(wp).w_skipcol;
                if skipcol > 0 {
                    let topline = win_ref(wp).w_topline;
                    let should_reset = last < topline
                        || (topline >= lnum
                            && topline < lnume
                            && nvim_change_linetabsize_eol(wp, topline)
                                <= skipcol + nvim_change_sms_marker_overlap(wp, -1));
                    if should_reset {
                        win_mut(wp).w_skipcol = 0;
                    }
                }

                // Update the folds for this window.
                nvim_fold_update(wp, lnum, last);

                // hasFoldingWin for lnum: update lnum and w_cline_folded.
                let mut fold_lnum = lnum;
                let folded_first = hasFoldingWin(
                    wp,
                    lnum,
                    std::ptr::addr_of_mut!(fold_lnum),
                    std::ptr::null_mut(),
                    false,
                    std::ptr::null_mut(),
                );
                let cursor_lnum = win_ref(wp).w_cursor.lnum;
                if cursor_lnum == fold_lnum {
                    win_mut(wp).w_cline_folded = folded_first;
                }

                // hasFoldingWin for last: update last and w_cline_folded.
                let mut fold_last = last;
                let folded_last = hasFoldingWin(
                    wp,
                    last,
                    std::ptr::null_mut(),
                    std::ptr::addr_of_mut!(fold_last),
                    false,
                    std::ptr::null_mut(),
                );
                if cursor_lnum == fold_last {
                    win_mut(wp).w_cline_folded = folded_last;
                }

                // Invalidate w_valid flags and w_lines[] entries.
                crate::invalidation::changed_lines_invalidate_win_impl(
                    wp, fold_lnum, col, lnume, xtra,
                );

                // Take care of side effects for setting w_topline when folds changed.
                if nvim_has_any_folding(wp) != 0 {
                    nvim_change_set_topline(wp, win_ref(wp).w_topline);
                }

                // Relative numbering always requires update if lines added/removed.
                if win_ref(wp).w_p_rnu() != 0 && xtra != 0 {
                    win_mut(wp).w_last_cursor_lnum_rnu = 0;
                }

                // Update cursorline tracking.
                if win_ref(wp).w_p_cul() != 0 {
                    let last_cursorline = win_ref(wp).w_last_cursorline;
                    if last_cursorline >= lnum {
                        if last_cursorline < lnume {
                            // cursorline was inside the change: already invalidated.
                            win_mut(wp).w_last_cursorline = 0;
                        } else {
                            // cursorline was below the change: adjust.
                            win_mut(wp).w_last_cursorline = last_cursorline + xtra;
                        }
                    }
                }
            }

            // search_hl_has_cursor_lnum adjustment (applies even if buf doesn't match).
            if wp == curwin && xtra != 0 {
                let hl_lnum = nvim_get_search_hl_has_cursor_lnum();
                if hl_lnum >= lnum {
                    nvim_set_search_hl_has_cursor_lnum(hl_lnum + xtra);
                }
            }
        }
        nvim_for_all_tab_windows_end(iter);

        // Call update_screen() later.
        nvim_change_set_must_redraw(UPD_VALID);

        // When the cursor line is changed, always trigger CursorMoved.
        nvim_change_last_cursormoved_reset_check(buf, lnum, lnume, xtra);
    }
}
