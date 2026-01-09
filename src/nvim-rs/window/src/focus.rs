//! Window focus and navigation functions.
//!
//! This module provides Rust implementations of window focus and navigation
//! functions from `src/nvim/window.c`.

use std::ffi::c_int;

use crate::{Frame, TabpageHandle, WinHandle, FR_COL, FR_LEAF, FR_ROW};

use crate::list::{
    frame2win_impl, nvim_get_firstwin, nvim_win_get_floating, nvim_win_get_frame, win_valid_impl,
};

extern "C" {
    /// Get the `w_wcol` field from a window (cursor column in window).
    fn nvim_win_get_wcol(wp: WinHandle) -> c_int;

    /// Get the `w_wrow` field from a window (cursor row in window).
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;

    /// Get the `w_winrow` field from a window.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Get the `w_wincol` field from a window.
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;

    /// Get the `tp_topframe` field from a tabpage.
    fn nvim_tabpage_get_topframe(tp: TabpageHandle) -> *mut Frame;

    /// Get the prevwin global.
    fn nvim_get_prevwin() -> WinHandle;
}

/// Get the above or below neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specified window is a floating window.
///
/// This is the Rust equivalent of `win_vert_neighbor()` in window.c.
#[allow(clippy::too_many_lines)]
pub(crate) fn win_vert_neighbor_impl(
    tp: TabpageHandle,
    wp: WinHandle,
    up: bool,
    mut count: c_int,
) -> WinHandle {
    if wp.is_null() || tp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut foundfr = nvim_win_get_frame(wp);

        // If floating window, return prevwin if valid and non-floating, else firstwin
        if nvim_win_get_floating(wp) != 0 {
            let prevwin = nvim_get_prevwin();
            let firstwin = nvim_get_firstwin();
            return if win_valid_impl(prevwin) && nvim_win_get_floating(prevwin) == 0 {
                prevwin
            } else {
                firstwin
            };
        }

        let topframe = nvim_tabpage_get_topframe(tp);

        while count > 0 {
            count -= 1;
            // First go upwards in the tree of frames until we find an upwards or
            // downwards neighbor.
            let mut fr = foundfr;
            let mut nfr;
            loop {
                if fr == topframe {
                    // Reached top, return what we found
                    return if foundfr.is_null() {
                        WinHandle::null()
                    } else {
                        (*foundfr).fr_win
                    };
                }
                nfr = if up { (*fr).fr_prev } else { (*fr).fr_next };
                let parent = (*fr).fr_parent;
                if !parent.is_null() && (*parent).fr_layout == FR_COL && !nfr.is_null() {
                    break;
                }
                fr = parent;
            }

            // Now go downwards to find the bottom or top frame in it.
            loop {
                if (*nfr).fr_layout == FR_LEAF {
                    foundfr = nfr;
                    break;
                }
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout == FR_ROW {
                    // Find the frame at the cursor column.
                    let wp_wincol = nvim_win_get_wincol(wp);
                    let wp_wcol = nvim_win_get_wcol(wp);
                    while !(*fr).fr_next.is_null() {
                        let fr_win = frame2win_impl(fr);
                        let fr_wincol = nvim_win_get_wincol(fr_win);
                        if fr_wincol + (*fr).fr_width > wp_wincol + wp_wcol {
                            break;
                        }
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout == FR_COL && up {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }

        if foundfr.is_null() {
            WinHandle::null()
        } else {
            (*foundfr).fr_win
        }
    }
}

/// Get the left or right neighbor window of the specified window.
///
/// Returns the specified window if the neighbor is not found.
/// Returns the previous window if the specified window is a floating window.
///
/// This is the Rust equivalent of `win_horz_neighbor()` in window.c.
#[allow(clippy::too_many_lines)]
pub(crate) fn win_horz_neighbor_impl(
    tp: TabpageHandle,
    wp: WinHandle,
    left: bool,
    mut count: c_int,
) -> WinHandle {
    if wp.is_null() || tp.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut foundfr = nvim_win_get_frame(wp);

        // If floating window, return prevwin if valid and non-floating, else firstwin
        if nvim_win_get_floating(wp) != 0 {
            let prevwin = nvim_get_prevwin();
            let firstwin = nvim_get_firstwin();
            return if win_valid_impl(prevwin) && nvim_win_get_floating(prevwin) == 0 {
                prevwin
            } else {
                firstwin
            };
        }

        let topframe = nvim_tabpage_get_topframe(tp);

        while count > 0 {
            count -= 1;
            // First go upwards in the tree of frames until we find a left or
            // right neighbor.
            let mut fr = foundfr;
            let mut nfr;
            loop {
                if fr == topframe {
                    // Reached top, return what we found
                    return if foundfr.is_null() {
                        WinHandle::null()
                    } else {
                        (*foundfr).fr_win
                    };
                }
                nfr = if left { (*fr).fr_prev } else { (*fr).fr_next };
                let parent = (*fr).fr_parent;
                if !parent.is_null() && (*parent).fr_layout == FR_ROW && !nfr.is_null() {
                    break;
                }
                fr = parent;
            }

            // Now go downwards to find the leftmost or rightmost frame in it.
            loop {
                if (*nfr).fr_layout == FR_LEAF {
                    foundfr = nfr;
                    break;
                }
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout == FR_COL {
                    // Find the frame at the cursor row.
                    let wp_winrow = nvim_win_get_winrow(wp);
                    let wp_wrow = nvim_win_get_wrow(wp);
                    while !(*fr).fr_next.is_null() {
                        let fr_win = frame2win_impl(fr);
                        let fr_winrow = nvim_win_get_winrow(fr_win);
                        if fr_winrow + (*fr).fr_height > wp_winrow + wp_wrow {
                            break;
                        }
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout == FR_ROW && left {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }

        if foundfr.is_null() {
            WinHandle::null()
        } else {
            (*foundfr).fr_win
        }
    }
}

// Note: FFI wrappers rs_win_vert_neighbor and rs_win_horz_neighbor are in lib.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_args() {
        // Test with null window
        assert!(win_vert_neighbor_impl(
            unsafe { TabpageHandle::from_ptr(0x1000 as *mut _) },
            WinHandle::null(),
            true,
            1
        )
        .is_null());

        // Test with null tabpage
        assert!(win_horz_neighbor_impl(
            unsafe { TabpageHandle::from_ptr(std::ptr::null_mut()) },
            unsafe { WinHandle::from_ptr(0x1000 as *mut _) },
            true,
            1
        )
        .is_null());
    }
}
