//! Window size save/restore functions.
//!
//! This module provides Rust implementations of `win_size_save` and
//! `win_size_restore` from `src/nvim/window.c`.

use std::ffi::{c_int, c_void};

use crate::WinHandle;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_frame(wp: WinHandle) -> *mut crate::Frame;
    fn nvim_get_rows_avail() -> c_int;
    fn rs_win_count() -> c_int;
    fn rs_global_stl_height() -> c_int;
    fn rs_last_stl_height(morewin: c_int) -> c_int;
    fn rs_frame_setwidth(curfrp: *mut crate::Frame, width: c_int);
    fn rs_win_setheight_win(height: c_int, win: WinHandle);
    fn rs_win_comp_pos() -> c_int;

    // Growarray accessors
    fn nvim_ga_init_int(gap: *mut c_void);
    fn nvim_ga_grow(gap: *mut c_void, n: c_int);
    fn nvim_ga_get_len(gap: *mut c_void) -> c_int;
    fn nvim_ga_set_len(gap: *mut c_void, len: c_int);
    fn nvim_ga_get_int(gap: *mut c_void, idx: c_int) -> c_int;
    fn nvim_ga_set_int(gap: *mut c_void, idx: c_int, val: c_int);
}

// =============================================================================
// Implementations
// =============================================================================

/// Save the size of all windows in "gap".
///
/// Equivalent to C `win_size_save()` (window.c L6941).
fn win_size_save_impl(gap: *mut c_void) {
    if gap.is_null() {
        return;
    }

    unsafe {
        nvim_ga_init_int(gap);
        let count = rs_win_count();
        nvim_ga_grow(gap, count * 2 + 1);

        // First entry is the total lines available for windows.
        let rows_avail = nvim_get_rows_avail() + rs_global_stl_height() - rs_last_stl_height(0);
        let mut len = nvim_ga_get_len(gap);
        nvim_ga_set_int(gap, len, rows_avail);
        len += 1;
        nvim_ga_set_len(gap, len);

        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            let width = nvim_win_get_w_width(wp) + nvim_win_get_vsep_width(wp);
            let height = nvim_win_get_w_height(wp);
            nvim_ga_set_int(gap, len, width);
            len += 1;
            nvim_ga_set_int(gap, len, height);
            len += 1;
            nvim_ga_set_len(gap, len);
            wp = nvim_win_get_next(wp);
        }
    }
}

/// Restore window sizes, but only if the number of windows is still the same
/// and total lines available for windows didn't change.
/// Does not free the growarray.
///
/// Equivalent to C `win_size_restore()` (window.c L6959).
fn win_size_restore_impl(gap: *mut c_void) {
    if gap.is_null() {
        return;
    }

    unsafe {
        let count = rs_win_count();
        let ga_len = nvim_ga_get_len(gap);

        if count * 2 + 1 != ga_len {
            return;
        }

        let rows_avail = nvim_get_rows_avail() + rs_global_stl_height() - rs_last_stl_height(0);
        if nvim_ga_get_int(gap, 0) != rows_avail {
            return;
        }

        // The order matters, because frames contain other frames, but it's
        // difficult to get right. The easy way out is to do it twice.
        for _j in 0..2 {
            let mut i = 1;
            let mut wp = nvim_get_firstwin();
            while !wp.is_null() {
                let width = nvim_ga_get_int(gap, i);
                i += 1;
                let height = nvim_ga_get_int(gap, i);
                i += 1;
                if nvim_win_get_floating(wp) == 0 {
                    rs_frame_setwidth(nvim_win_get_frame(wp), width);
                    rs_win_setheight_win(height, wp);
                }
                wp = nvim_win_get_next(wp);
            }
        }
        // Recompute the window positions.
        rs_win_comp_pos();
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Save the size of all windows in "gap".
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_size_save(gap: *mut c_void) {
    win_size_save_impl(gap);
}

/// FFI: Restore window sizes from "gap".
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_size_restore(gap: *mut c_void) {
    win_size_restore_impl(gap);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_save() {
        // Should not panic
        win_size_save_impl(std::ptr::null_mut());
    }

    #[test]
    fn test_null_restore() {
        // Should not panic
        win_size_restore_impl(std::ptr::null_mut());
    }
}
