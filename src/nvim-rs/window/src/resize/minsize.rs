//! Minimum size calculations for tabpages.
//!
//! This module provides Rust implementations of `min_rows` and
//! `min_rows_for_all_tabpages` from `src/nvim/window.c`.

use std::ffi::c_int;

use crate::list::{nvim_get_first_tabpage, nvim_get_firstwin, nvim_tabpage_get_next};
use crate::{Frame, TabpageHandle};

use super::fraction::MIN_LINES;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_window_p_ch() -> i64;
    fn nvim_tabpage_get_ch_used(tp: TabpageHandle) -> c_int;
    fn nvim_tabpage_get_topframe(tp: TabpageHandle) -> *mut Frame;
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: crate::WinHandle) -> c_int;
    fn rs_tabline_height() -> c_int;
    fn rs_global_stl_height() -> c_int;
}

// =============================================================================
// Implementations
// =============================================================================

/// Return the minimal number of rows needed to display the current number
/// of windows for the given tab page.
///
/// Equivalent to C `min_rows()` (window.c L7688).
fn min_rows_impl(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        return MIN_LINES;
    }

    unsafe {
        let firstwin = nvim_get_firstwin();
        if firstwin.is_null() {
            return MIN_LINES;
        }

        let topframe = nvim_tabpage_get_topframe(tp);
        let mut total = rs_frame_minheight(topframe, crate::WinHandle::null());
        total += rs_tabline_height() + rs_global_stl_height();

        let curtab = nvim_get_curtab();
        let p_ch = if tp == curtab {
            nvim_get_window_p_ch()
        } else {
            i64::from(nvim_tabpage_get_ch_used(tp))
        };
        if p_ch > 0 {
            total += 1; // count the room for the command line
        }
        total
    }
}

/// Return the minimal number of rows needed to display the current number
/// of windows for all tab pages.
///
/// Equivalent to C `min_rows_for_all_tabpages()` (window.c L7704).
fn min_rows_for_all_tabpages_impl() -> c_int {
    unsafe {
        let firstwin = nvim_get_firstwin();
        if firstwin.is_null() {
            return MIN_LINES;
        }

        let curtab = nvim_get_curtab();
        let mut total: c_int = 0;

        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let topframe = nvim_tabpage_get_topframe(tp);
            let mut n = rs_frame_minheight(topframe, crate::WinHandle::null());

            let p_ch = if tp == curtab {
                nvim_get_window_p_ch()
            } else {
                i64::from(nvim_tabpage_get_ch_used(tp))
            };
            if p_ch > 0 {
                n += 1;
            }
            if n > total {
                total = n;
            }
            tp = nvim_tabpage_get_next(tp);
        }

        total += rs_tabline_height() + rs_global_stl_height();
        total
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Return minimal rows for a tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_min_rows(tp: TabpageHandle) -> c_int {
    min_rows_impl(tp)
}

/// FFI: Return minimal rows for all tabpages.
#[unsafe(no_mangle)]
pub extern "C" fn rs_min_rows_for_all_tabpages() -> c_int {
    min_rows_for_all_tabpages_impl()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_tabpage_min_rows() {
        assert_eq!(min_rows_impl(TabpageHandle::null()), MIN_LINES);
    }
}
