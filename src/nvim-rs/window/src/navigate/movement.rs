//! Window movement and navigation functions.
//!
//! This module provides helper functions for window cursor movement
//! and window navigation operations.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get firstwin.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get lastwin.
    fn nvim_get_lastwin() -> WinHandle;

    /// Get w_next from window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get w_prev from window.
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get window row position.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Get window column position.
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;

    /// Get window height.
    fn nvim_win_get_height(wp: WinHandle) -> c_int;

    /// Get window width.
    fn nvim_win_get_width(wp: WinHandle) -> c_int;
}

// =============================================================================
// Direction-based Navigation
// =============================================================================

/// Find window in a specific direction from current window.
///
/// Direction: 'h' = left, 'j' = down, 'k' = up, 'l' = right
fn find_in_direction_impl(dir: u8) -> WinHandle {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() || nvim_win_get_floating(curwin) != 0 {
            return WinHandle::null();
        }

        let frame = nvim_win_get_frame(curwin);
        if frame.is_null() {
            return WinHandle::null();
        }

        match dir {
            b'h' => find_left_impl(curwin),
            b'j' => find_below_impl(curwin),
            b'k' => find_above_impl(curwin),
            b'l' => find_right_impl(curwin),
            _ => WinHandle::null(),
        }
    }
}

/// Find the window to the left.
fn find_left_impl(wp: WinHandle) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let col = nvim_win_get_wincol(wp);
        let row = nvim_win_get_winrow(wp);
        let height = nvim_win_get_height(wp);
        let mid_row = row + height / 2;

        // Find rightmost window left of current that overlaps vertically
        let mut best = WinHandle::null();
        let mut best_col = -1;

        let mut w = nvim_get_firstwin();
        while !w.is_null() {
            if w != wp && nvim_win_get_floating(w) == 0 {
                let w_col = nvim_win_get_wincol(w);
                let w_width = nvim_win_get_width(w);
                let w_row = nvim_win_get_winrow(w);
                let w_height = nvim_win_get_height(w);

                // Must be left of current window
                if w_col + w_width <= col {
                    // Check vertical overlap
                    let w_mid_row = w_row + w_height / 2;
                    if (w_row <= mid_row && mid_row < w_row + w_height)
                        || (row <= w_mid_row && w_mid_row < row + height)
                    {
                        // Prefer rightmost
                        if w_col + w_width > best_col {
                            best_col = w_col + w_width;
                            best = w;
                        }
                    }
                }
            }
            w = nvim_win_get_next(w);
        }
        best
    }
}

/// Find the window to the right.
fn find_right_impl(wp: WinHandle) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let col = nvim_win_get_wincol(wp);
        let width = nvim_win_get_width(wp);
        let row = nvim_win_get_winrow(wp);
        let height = nvim_win_get_height(wp);
        let mid_row = row + height / 2;
        let right_edge = col + width;

        // Find leftmost window right of current that overlaps vertically
        let mut best = WinHandle::null();
        let mut best_col = c_int::MAX;

        let mut w = nvim_get_firstwin();
        while !w.is_null() {
            if w != wp && nvim_win_get_floating(w) == 0 {
                let w_col = nvim_win_get_wincol(w);
                let w_row = nvim_win_get_winrow(w);
                let w_height = nvim_win_get_height(w);

                // Must be right of current window
                if w_col >= right_edge {
                    // Check vertical overlap
                    let w_mid_row = w_row + w_height / 2;
                    if (w_row <= mid_row && mid_row < w_row + w_height)
                        || (row <= w_mid_row && w_mid_row < row + height)
                    {
                        // Prefer leftmost
                        if w_col < best_col {
                            best_col = w_col;
                            best = w;
                        }
                    }
                }
            }
            w = nvim_win_get_next(w);
        }
        best
    }
}

/// Find the window above.
fn find_above_impl(wp: WinHandle) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let col = nvim_win_get_wincol(wp);
        let width = nvim_win_get_width(wp);
        let row = nvim_win_get_winrow(wp);
        let mid_col = col + width / 2;

        // Find bottommost window above current that overlaps horizontally
        let mut best = WinHandle::null();
        let mut best_row = -1;

        let mut w = nvim_get_firstwin();
        while !w.is_null() {
            if w != wp && nvim_win_get_floating(w) == 0 {
                let w_col = nvim_win_get_wincol(w);
                let w_width = nvim_win_get_width(w);
                let w_row = nvim_win_get_winrow(w);
                let w_height = nvim_win_get_height(w);

                // Must be above current window
                if w_row + w_height <= row {
                    // Check horizontal overlap
                    let w_mid_col = w_col + w_width / 2;
                    if (w_col <= mid_col && mid_col < w_col + w_width)
                        || (col <= w_mid_col && w_mid_col < col + width)
                    {
                        // Prefer bottommost
                        if w_row + w_height > best_row {
                            best_row = w_row + w_height;
                            best = w;
                        }
                    }
                }
            }
            w = nvim_win_get_next(w);
        }
        best
    }
}

/// Find the window below.
fn find_below_impl(wp: WinHandle) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let col = nvim_win_get_wincol(wp);
        let width = nvim_win_get_width(wp);
        let row = nvim_win_get_winrow(wp);
        let height = nvim_win_get_height(wp);
        let mid_col = col + width / 2;
        let bottom_edge = row + height;

        // Find topmost window below current that overlaps horizontally
        let mut best = WinHandle::null();
        let mut best_row = c_int::MAX;

        let mut w = nvim_get_firstwin();
        while !w.is_null() {
            if w != wp && nvim_win_get_floating(w) == 0 {
                let w_col = nvim_win_get_wincol(w);
                let w_width = nvim_win_get_width(w);
                let w_row = nvim_win_get_winrow(w);

                // Must be below current window
                if w_row >= bottom_edge {
                    // Check horizontal overlap
                    let w_mid_col = w_col + w_width / 2;
                    if (w_col <= mid_col && mid_col < w_col + w_width)
                        || (col <= w_mid_col && w_mid_col < col + width)
                    {
                        // Prefer topmost
                        if w_row < best_row {
                            best_row = w_row;
                            best = w;
                        }
                    }
                }
            }
            w = nvim_win_get_next(w);
        }
        best
    }
}

// =============================================================================
// Window List Navigation
// =============================================================================

/// Get the next window in the window list.
fn get_next_win_impl(wp: WinHandle, wrap: bool) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let next = nvim_win_get_next(wp);
        if !next.is_null() {
            return next;
        }

        if wrap {
            return nvim_get_firstwin();
        }

        WinHandle::null()
    }
}

/// Get the previous window in the window list.
fn get_prev_win_impl(wp: WinHandle, wrap: bool) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let prev = nvim_win_get_prev(wp);
        if !prev.is_null() {
            return prev;
        }

        if wrap {
            return nvim_get_lastwin();
        }

        WinHandle::null()
    }
}

/// Get the next non-floating window.
fn get_next_nonfloat_impl(wp: WinHandle, wrap: bool) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut w = nvim_win_get_next(wp);

        // Skip floating windows
        while !w.is_null() && nvim_win_get_floating(w) != 0 {
            w = nvim_win_get_next(w);
        }

        if !w.is_null() {
            return w;
        }

        if wrap {
            w = nvim_get_firstwin();
            while !w.is_null() && w != wp && nvim_win_get_floating(w) != 0 {
                w = nvim_win_get_next(w);
            }
            if !w.is_null() && w != wp {
                return w;
            }
        }

        WinHandle::null()
    }
}

/// Get the previous non-floating window.
fn get_prev_nonfloat_impl(wp: WinHandle, wrap: bool) -> WinHandle {
    if wp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut w = nvim_win_get_prev(wp);

        // Skip floating windows
        while !w.is_null() && nvim_win_get_floating(w) != 0 {
            w = nvim_win_get_prev(w);
        }

        if !w.is_null() {
            return w;
        }

        if wrap {
            // Find last non-floating window
            let mut last = nvim_get_lastwin();
            while !last.is_null() && nvim_win_get_floating(last) != 0 {
                last = nvim_win_get_prev(last);
            }
            if !last.is_null() && last != wp {
                return last;
            }
        }

        WinHandle::null()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Find window in direction from current.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub extern "C" fn rs_nav_find_in_direction(dir: c_int) -> WinHandle {
    find_in_direction_impl(dir as u8)
}

/// FFI: Find window to the left.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_left(wp: WinHandle) -> WinHandle {
    find_left_impl(wp)
}

/// FFI: Find window to the right.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_right(wp: WinHandle) -> WinHandle {
    find_right_impl(wp)
}

/// FFI: Find window above.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_above(wp: WinHandle) -> WinHandle {
    find_above_impl(wp)
}

/// FFI: Find window below.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_below(wp: WinHandle) -> WinHandle {
    find_below_impl(wp)
}

/// FFI: Get next window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_next(wp: WinHandle, wrap: c_int) -> WinHandle {
    get_next_win_impl(wp, wrap != 0)
}

/// FFI: Get previous window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_prev(wp: WinHandle, wrap: c_int) -> WinHandle {
    get_prev_win_impl(wp, wrap != 0)
}

/// FFI: Get next non-floating window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_next_nonfloat(wp: WinHandle, wrap: c_int) -> WinHandle {
    get_next_nonfloat_impl(wp, wrap != 0)
}

/// FFI: Get previous non-floating window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_prev_nonfloat(wp: WinHandle, wrap: c_int) -> WinHandle {
    get_prev_nonfloat_impl(wp, wrap != 0)
}

/// FFI: Check if direction is horizontal.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub extern "C" fn rs_nav_is_horizontal_dir(dir: c_int) -> c_int {
    let d = dir as u8;
    c_int::from(d == b'h' || d == b'l')
}

/// FFI: Check if direction is vertical.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub extern "C" fn rs_nav_is_vertical_dir(dir: c_int) -> c_int {
    let d = dir as u8;
    c_int::from(d == b'j' || d == b'k')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_window_navigation() {
        let null_wp = WinHandle::null();
        assert!(find_left_impl(null_wp).is_null());
        assert!(find_right_impl(null_wp).is_null());
        assert!(find_above_impl(null_wp).is_null());
        assert!(find_below_impl(null_wp).is_null());
        assert!(get_next_win_impl(null_wp, true).is_null());
        assert!(get_prev_win_impl(null_wp, true).is_null());
    }

    #[test]
    fn test_direction_checks() {
        assert_eq!(rs_nav_is_horizontal_dir(c_int::from(b'h')), 1);
        assert_eq!(rs_nav_is_horizontal_dir(c_int::from(b'l')), 1);
        assert_eq!(rs_nav_is_horizontal_dir(c_int::from(b'j')), 0);
        assert_eq!(rs_nav_is_horizontal_dir(c_int::from(b'k')), 0);

        assert_eq!(rs_nav_is_vertical_dir(c_int::from(b'j')), 1);
        assert_eq!(rs_nav_is_vertical_dir(c_int::from(b'k')), 1);
        assert_eq!(rs_nav_is_vertical_dir(c_int::from(b'h')), 0);
        assert_eq!(rs_nav_is_vertical_dir(c_int::from(b'l')), 0);
    }
}
