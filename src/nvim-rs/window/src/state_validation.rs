//! Cursor line number validation functions.
//!
//! This module provides Rust implementations of `check_lnums_both`,
//! `check_lnums`, `check_lnums_nested`, and `reset_lnums` from
//! `src/nvim/window.c`.

use std::ffi::c_int;

use crate::{TabpageHandle, WinHandle};

// =============================================================================
// Constants
// =============================================================================

/// w_topline validity flag. Must match C `VALID_TOPLINE` (buffer_defs.h).
const VALID_TOPLINE: c_int = 0x80;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_get_first_tabpage() -> TabpageHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    // Buffer/cursor accessors
    fn nvim_win_buf_is_curbuf(wp: WinHandle) -> c_int;
    fn nvim_curbuf_line_count() -> c_int;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: c_int);
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;
    fn nvim_win_set_topline(wp: WinHandle, val: c_int);
    fn nvim_win_clear_valid_bits(wp: WinHandle, bits: c_int);

    // Save cursor compound accessors
    fn nvim_win_save_cursor_to_save(wp: WinHandle);
    fn nvim_win_save_topline_to_save(wp: WinHandle);
    fn nvim_win_save_cursor_to_corr(wp: WinHandle);
    fn nvim_win_save_topline_to_corr(wp: WinHandle);
    fn nvim_win_cursor_eq_save_corr(wp: WinHandle) -> c_int;
    fn nvim_win_topline_eq_save_corr(wp: WinHandle) -> c_int;
    fn nvim_win_get_save_cursor_save_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_save_topline_save(wp: WinHandle) -> c_int;
    fn nvim_win_restore_cursor_from_save(wp: WinHandle);
    fn nvim_win_restore_topline_from_save(wp: WinHandle);
    fn nvim_win_save_topline_gt_buf_line_count(wp: WinHandle) -> c_int;
}

// =============================================================================
// Helpers
// =============================================================================

/// Get the first window in a tabpage, handling curtab specially.
#[inline]
unsafe fn tabpage_first_win(tp: TabpageHandle) -> WinHandle {
    if tp == nvim_get_curtab() {
        nvim_get_firstwin()
    } else {
        nvim_tabpage_get_firstwin(tp)
    }
}

// =============================================================================
// Implementations
// =============================================================================

/// Implementation of check_lnums() and check_lnums_nested().
///
/// Equivalent to C `check_lnums_both()` (window.c L7684).
fn check_lnums_both_impl(do_curwin: bool, nested: bool) {
    unsafe {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let mut wp = tabpage_first_win(tp);
            while !wp.is_null() {
                if (do_curwin || nvim_get_curwin() != wp) && nvim_win_buf_is_curbuf(wp) != 0 {
                    if !nested {
                        // Save the original cursor position and topline.
                        nvim_win_save_cursor_to_save(wp);
                        nvim_win_save_topline_to_save(wp);
                    }

                    let line_count = nvim_curbuf_line_count();

                    let mut need_adjust = nvim_win_get_cursor_lnum(wp) > line_count;
                    if need_adjust {
                        nvim_win_set_cursor_lnum(wp, line_count);
                    }
                    if need_adjust || !nested {
                        // Save the (corrected) cursor position.
                        nvim_win_save_cursor_to_corr(wp);
                    }

                    need_adjust = nvim_win_get_topline(wp) > line_count;
                    if need_adjust {
                        nvim_win_set_topline(wp, line_count);
                    }
                    if need_adjust || !nested {
                        // Save the (corrected) topline.
                        nvim_win_save_topline_to_corr(wp);
                    }
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
    }
}

/// Reset cursor and topline to its stored values from check_lnums().
/// check_lnums() must have been called first!
///
/// Equivalent to C `reset_lnums()` (window.c L7732).
fn reset_lnums_impl() {
    unsafe {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let mut wp = tabpage_first_win(tp);
            while !wp.is_null() {
                if nvim_win_buf_is_curbuf(wp) != 0 {
                    // Restore the value if the autocommand didn't change it
                    // and it was set.
                    if nvim_win_cursor_eq_save_corr(wp) != 0
                        && nvim_win_get_save_cursor_save_lnum(wp) != 0
                    {
                        nvim_win_restore_cursor_from_save(wp);
                    }
                    if nvim_win_topline_eq_save_corr(wp) != 0
                        && nvim_win_get_save_topline_save(wp) != 0
                    {
                        nvim_win_restore_topline_from_save(wp);
                    }
                    if nvim_win_save_topline_gt_buf_line_count(wp) != 0 {
                        nvim_win_clear_valid_bits(wp, VALID_TOPLINE);
                    }
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Correct cursor line number in other windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_check_lnums(do_curwin: c_int) {
    check_lnums_both_impl(do_curwin != 0, false);
}

/// FFI: Like check_lnums() but for when check_lnums() was already called.
#[unsafe(no_mangle)]
pub extern "C" fn rs_check_lnums_nested(do_curwin: c_int) {
    check_lnums_both_impl(do_curwin != 0, true);
}

/// FFI: Reset cursor and topline to stored values from check_lnums().
#[unsafe(no_mangle)]
pub extern "C" fn rs_reset_lnums() {
    reset_lnums_impl();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_topline_constant() {
        assert_eq!(VALID_TOPLINE, 0x80);
    }
}
