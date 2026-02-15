//! Display invalidation functions for window display updates after changes.
//!
//! This module provides functions to invalidate window display regions
//! after buffer modifications.

use std::ffi::{c_int, c_void};

use crate::{BufHandle, ColnrT, LinenrT, WinHandle};

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Window accessors
    fn nvim_win_get_buffer(win: WinHandle) -> BufHandle;
    fn nvim_win_get_cursor_lnum(win: WinHandle) -> LinenrT;
    fn nvim_win_get_cursor_col(win: WinHandle) -> ColnrT;
    fn nvim_win_get_botline(win: WinHandle) -> LinenrT;
    fn nvim_win_get_p_wrap(win: WinHandle) -> c_int;

    // w_lines[] accessors
    fn nvim_win_get_lines_valid(win: WinHandle) -> c_int;
    fn nvim_win_get_lines_wl_valid(win: WinHandle, idx: c_int) -> bool;
    fn nvim_win_set_lines_wl_valid(win: WinHandle, idx: c_int, val: bool);
    fn nvim_win_get_lines_wl_lnum(win: WinHandle, idx: c_int) -> LinenrT;
    fn nvim_win_set_lines_wl_lnum(win: WinHandle, idx: c_int, val: LinenrT);
    fn nvim_win_get_lines_wl_foldend(win: WinHandle, idx: c_int) -> LinenrT;
    fn nvim_win_set_lines_wl_foldend(win: WinHandle, idx: c_int, val: LinenrT);
    fn nvim_win_get_lines_wl_lastlnum(win: WinHandle, idx: c_int) -> LinenrT;
    fn nvim_win_set_lines_wl_lastlnum(win: WinHandle, idx: c_int, val: LinenrT);

    // Window state functions
    fn nvim_changed_line_abv_curs_win(win: WinHandle);
    fn nvim_changed_cline_bef_curs(win: WinHandle);
    fn nvim_approximate_botline_win(win: WinHandle);
    fn nvim_find_wl_entry(win: WinHandle, lnum: LinenrT) -> c_int;

    // Buffer metadata
    fn nvim_buf_meta_total(buf: BufHandle, meta_type: c_int) -> c_int;

    // Window iteration
    fn nvim_for_all_tab_windows_start() -> *mut c_void;
    fn nvim_for_all_tab_windows_next(iter: *mut c_void) -> WinHandle;
    fn nvim_for_all_tab_windows_end(iter: *mut c_void);
}

/// Meta type for inline virtual text.
const KMT_META_INLINE: c_int = 0;

/// Meta type for virtual lines.
const KMT_META_LINES: c_int = 1;

// =============================================================================
// Window Invalidation Functions
// =============================================================================

/// Invalidate a window's w_valid flags and w_lines[] entries after changing lines.
///
/// This is called for each window displaying the changed buffer.
fn changed_lines_invalidate_win_impl(
    wp: WinHandle,
    lnum: LinenrT,
    col: ColnrT,
    lnume: LinenrT,
    xtra: LinenrT,
) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        let cursor_lnum = nvim_win_get_cursor_lnum(wp);
        let cursor_col = nvim_win_get_cursor_col(wp);
        let buf = nvim_win_get_buffer(wp);

        // If the changed line is in a range of previously folded lines,
        // compare with the first line in that range.
        if cursor_lnum <= lnum {
            let i = nvim_find_wl_entry(wp, lnum);
            if i >= 0 {
                let wl_lnum = nvim_win_get_lines_wl_lnum(wp, i);
                if cursor_lnum > wl_lnum {
                    nvim_changed_line_abv_curs_win(wp);
                }
            }
        }

        if cursor_lnum > lnum {
            nvim_changed_line_abv_curs_win(wp);
        } else if cursor_lnum == lnum && cursor_col >= col {
            nvim_changed_cline_bef_curs(wp);
        }

        let botline = nvim_win_get_botline(wp);
        if botline >= lnum {
            // Assume that botline doesn't change (inserted lines make
            // other lines scroll down below botline).
            nvim_approximate_botline_win(wp);
        }

        // Adjust lnume for virtual lines and inline text
        let mut lnume_adj = lnume;
        if (xtra < 0
            && nvim_win_get_p_wrap(wp) != 0
            && nvim_buf_meta_total(buf, KMT_META_INLINE) != 0)
            || (xtra != 0 && nvim_buf_meta_total(buf, KMT_META_LINES) != 0)
        {
            lnume_adj += 1;
        }

        // Check if any w_lines[] entries have become invalid.
        // For entries below the change: Correct the lnums for inserted/deleted lines.
        // Makes it possible to stop displaying after the change.
        let lines_valid = nvim_win_get_lines_valid(wp);
        for i in 0..lines_valid {
            if nvim_win_get_lines_wl_valid(wp, i) {
                let wl_lnum = nvim_win_get_lines_wl_lnum(wp, i);
                if wl_lnum >= lnum {
                    // Do not change wl_lnum at index zero, it is used to compare with w_topline.
                    // Invalidate it instead.
                    if i == 0 || wl_lnum < lnume_adj {
                        // line included in change
                        nvim_win_set_lines_wl_valid(wp, i, false);
                    } else if xtra != 0 {
                        // line below change
                        nvim_win_set_lines_wl_lnum(wp, i, wl_lnum + xtra);
                        let wl_foldend = nvim_win_get_lines_wl_foldend(wp, i);
                        nvim_win_set_lines_wl_foldend(wp, i, wl_foldend + xtra);
                        let wl_lastlnum = nvim_win_get_lines_wl_lastlnum(wp, i);
                        nvim_win_set_lines_wl_lastlnum(wp, i, wl_lastlnum + xtra);
                    }
                } else {
                    let wl_lastlnum = nvim_win_get_lines_wl_lastlnum(wp, i);
                    if wl_lastlnum >= lnum {
                        // change somewhere inside this range of folded or concealed lines,
                        // may need to be redrawn
                        nvim_win_set_lines_wl_valid(wp, i, false);
                    }
                }
            }
        }
    }
}

/// Invalidate display for all windows displaying a buffer.
///
/// Like `changed_lines_invalidate_win()`, but for all windows displaying a buffer.
fn changed_lines_invalidate_buf_impl(
    buf: BufHandle,
    lnum: LinenrT,
    col: ColnrT,
    lnume: LinenrT,
    xtra: LinenrT,
) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        let iter = nvim_for_all_tab_windows_start();
        loop {
            let wp = nvim_for_all_tab_windows_next(iter);
            if wp.is_null() {
                break;
            }
            if nvim_win_get_buffer(wp) == buf {
                changed_lines_invalidate_win_impl(wp, lnum, col, lnume, xtra);
            }
        }
        nvim_for_all_tab_windows_end(iter);
    }
}

/// FFI wrapper for `changed_lines_invalidate_buf`.
///
/// Invalidate display for all windows displaying a buffer after line changes.
#[export_name = "changed_lines_invalidate_buf"]
pub extern "C" fn rs_changed_lines_invalidate_buf(
    buf: BufHandle,
    lnum: LinenrT,
    col: ColnrT,
    lnume: LinenrT,
    xtra: LinenrT,
) {
    changed_lines_invalidate_buf_impl(buf, lnum, col, lnume, xtra);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(KMT_META_INLINE, 0);
        assert_eq!(KMT_META_LINES, 1);
    }
}
