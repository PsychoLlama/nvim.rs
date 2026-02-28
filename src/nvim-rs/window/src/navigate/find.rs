//! Window finding functions.
//!
//! This module provides functions for finding windows by various criteria
//! such as handle, number, buffer, position, etc.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{BufHandle, Frame, TabpageHandle, WinHandle};

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

    /// Get curtab.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get first tabpage.
    fn nvim_get_first_tabpage() -> TabpageHandle;

    /// Get next tabpage.
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;

    /// Get tp_firstwin from tabpage.
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    /// Get w_next from window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get w_prev from window.
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;

    /// Get w_buffer from window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get window handle.
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;

    /// Get window row position.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Get window column position.
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;

    /// Get window height.
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;

    /// Get window width.
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
}

// =============================================================================
// Window Finding by Handle/Number
// =============================================================================

/// Find a window by its handle in current tabpage.
fn find_by_handle_curtab_impl(handle: c_int) -> WinHandle {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_handle(wp) == handle {
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

/// Find a window by its handle in all tabpages.
fn find_by_handle_impl(handle: c_int) -> WinHandle {
    unsafe {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let first = nvim_tabpage_get_firstwin(tp);
            let mut wp = first;
            while !wp.is_null() {
                if nvim_win_get_handle(wp) == handle {
                    return wp;
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
        WinHandle::null()
    }
}

/// Find a window by window number (1-based) in current tabpage.
fn find_by_nr_impl(nr: c_int) -> WinHandle {
    if nr <= 0 {
        return WinHandle::null();
    }

    unsafe {
        let mut count = nr;
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            count -= 1;
            if count == 0 {
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

/// Find a window by window number in a specific tabpage.
fn find_by_nr_in_tab_impl(nr: c_int, tp: TabpageHandle) -> WinHandle {
    if nr <= 0 {
        return WinHandle::null();
    }

    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        let mut count = nr;
        let mut wp = first;
        while !wp.is_null() {
            count -= 1;
            if count == 0 {
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

// =============================================================================
// Window Finding by Buffer
// =============================================================================

/// Find the first window displaying a buffer in current tabpage.
fn find_by_buffer_curtab_impl(buf: BufHandle) -> WinHandle {
    if buf.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_buffer(wp) == buf {
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

/// Find the first window displaying a buffer in all tabpages.
fn find_by_buffer_impl(buf: BufHandle) -> WinHandle {
    if buf.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let first = nvim_tabpage_get_firstwin(tp);
            let mut wp = first;
            while !wp.is_null() {
                if nvim_win_get_buffer(wp) == buf {
                    return wp;
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
        WinHandle::null()
    }
}

/// Count windows displaying a buffer in current tabpage.
fn count_windows_for_buffer_impl(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }

    unsafe {
        let mut count = 0;
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_buffer(wp) == buf {
                count += 1;
            }
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

/// Count windows displaying a buffer across all tabpages.
fn count_windows_for_buffer_all_impl(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }

    unsafe {
        let mut count = 0;
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let first = nvim_tabpage_get_firstwin(tp);
            let mut wp = first;
            while !wp.is_null() {
                if nvim_win_get_buffer(wp) == buf {
                    count += 1;
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
        count
    }
}

// =============================================================================
// Window Finding by Position
// =============================================================================

/// Find window at screen position (row, col).
fn find_at_pos_impl(row: c_int, col: c_int) -> WinHandle {
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            let winrow = nvim_win_get_winrow(wp);
            let wincol = nvim_win_get_wincol(wp);
            let height = nvim_win_get_w_height(wp);
            let width = nvim_win_get_w_width(wp);

            // Check if position is within window bounds
            if row >= winrow && row < winrow + height && col >= wincol && col < wincol + width {
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

// =============================================================================
// Window Number Functions
// =============================================================================

/// Get the window number (1-based) for a window.
fn get_winnr_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let mut nr = 1;
        let mut w = nvim_get_firstwin();
        while !w.is_null() {
            if w == wp {
                return nr;
            }
            nr += 1;
            w = nvim_win_get_next(w);
        }
        0
    }
}

/// Get the window number for a window in a specific tabpage.
fn get_winnr_in_tab_impl(wp: WinHandle, tp: TabpageHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        let mut nr = 1;
        let mut w = first;
        while !w.is_null() {
            if w == wp {
                return nr;
            }
            nr += 1;
            w = nvim_win_get_next(w);
        }
        0
    }
}

/// Count total windows in current tabpage.
fn count_windows_impl() -> c_int {
    unsafe {
        let mut count = 0;
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            count += 1;
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

/// Count total windows in a specific tabpage.
fn count_windows_in_tab_impl(tp: TabpageHandle) -> c_int {
    unsafe {
        let first = if tp.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        };

        let mut count = 0;
        let mut wp = first;
        while !wp.is_null() {
            count += 1;
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Find window by handle in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_by_handle_curtab(handle: c_int) -> WinHandle {
    find_by_handle_curtab_impl(handle)
}

/// FFI: Find window by handle in all tabpages.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_by_handle(handle: c_int) -> WinHandle {
    find_by_handle_impl(handle)
}

/// FFI: Find window by number in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_by_nr(nr: c_int) -> WinHandle {
    find_by_nr_impl(nr)
}

/// FFI: Find window by number in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_by_nr_in_tab(nr: c_int, tp: TabpageHandle) -> WinHandle {
    find_by_nr_in_tab_impl(nr, tp)
}

/// FFI: Find first window for buffer in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_by_buffer_curtab(buf: BufHandle) -> WinHandle {
    find_by_buffer_curtab_impl(buf)
}

/// FFI: Find first window for buffer in all tabpages.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_by_buffer(buf: BufHandle) -> WinHandle {
    find_by_buffer_impl(buf)
}

/// FFI: Count windows for buffer in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_count_buffer_windows(buf: BufHandle) -> c_int {
    count_windows_for_buffer_impl(buf)
}

/// FFI: Count windows for buffer in all tabpages.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_count_buffer_windows_all(buf: BufHandle) -> c_int {
    count_windows_for_buffer_all_impl(buf)
}

/// FFI: Find window at screen position.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_find_at_pos(row: c_int, col: c_int) -> WinHandle {
    find_at_pos_impl(row, col)
}

/// FFI: Get window number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_winnr(wp: WinHandle) -> c_int {
    get_winnr_impl(wp)
}

/// FFI: Get window number in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_winnr_in_tab(wp: WinHandle, tp: TabpageHandle) -> c_int {
    get_winnr_in_tab_impl(wp, tp)
}

/// FFI: Count windows in current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_count_windows() -> c_int {
    count_windows_impl()
}

/// FFI: Count windows in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_count_windows_in_tab(tp: TabpageHandle) -> c_int {
    count_windows_in_tab_impl(tp)
}

/// FFI: Get current window handle.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_curwin() -> WinHandle {
    unsafe { nvim_get_curwin() }
}

/// FFI: Get first window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_firstwin() -> WinHandle {
    unsafe { nvim_get_firstwin() }
}

/// FFI: Get last window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_nav_get_lastwin() -> WinHandle {
    unsafe { nvim_get_lastwin() }
}

// =============================================================================
// buf_jump_open_win / buf_jump_open_tab / swbuf_goto_win_with_buf (Phase 2)
// =============================================================================

extern "C" {
    /// win_enter(wp, false) -- enter window without undo sync.
    fn nvim_win_enter(wp: WinHandle, undo_sync: c_int);

    /// goto_tabpage_win(tp, wp).
    fn rs_goto_tabpage_win(tp: crate::TabpageHandle, wp: WinHandle);

    /// swb_flags & kOptSwbFlagUseopen.
    fn nvim_swb_has_useopen() -> c_int;

    /// swb_flags & kOptSwbFlagUsetab.
    fn nvim_swb_has_usetab() -> c_int;
}

/// Jump to the first open window that contains buffer "buf", if one exists.
///
/// Rust port of C `buf_jump_open_win()`.
fn buf_jump_open_win_impl(buf: BufHandle) -> WinHandle {
    if buf.is_null() {
        return WinHandle::null();
    }

    unsafe {
        let curtab = nvim_get_curtab();
        // Check curwin first.
        let curwin = nvim_get_curwin();
        if !curwin.is_null() && nvim_win_get_buffer(curwin) == buf {
            nvim_win_enter(curwin, 0);
            return nvim_get_curwin(); // re-read after enter
        }

        // Iterate all windows in current tab.
        let first = if curtab.is_null() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(curtab)
        };
        let mut wp = first;
        while !wp.is_null() {
            if nvim_win_get_buffer(wp) == buf {
                nvim_win_enter(wp, 0);
                return wp;
            }
            wp = nvim_win_get_next(wp);
        }
        WinHandle::null()
    }
}

/// Jump to the first open window in any tab page that contains buffer "buf".
///
/// Rust port of C `buf_jump_open_tab()`.
fn buf_jump_open_tab_impl(buf: BufHandle) -> WinHandle {
    if buf.is_null() {
        return WinHandle::null();
    }

    // First try current tab.
    let wp = buf_jump_open_win_impl(buf);
    if !wp.is_null() {
        return wp;
    }

    unsafe {
        let curtab = nvim_get_curtab();
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            // Skip the current tab since we already checked it.
            if tp != curtab {
                let first = nvim_tabpage_get_firstwin(tp);
                let mut wp = first;
                while !wp.is_null() {
                    if nvim_win_get_buffer(wp) == buf {
                        rs_goto_tabpage_win(tp, wp);
                        // If curwin didn't switch, something went wrong.
                        let curwin = nvim_get_curwin();
                        if curwin != wp {
                            return WinHandle::null();
                        }
                        return wp;
                    }
                    wp = nvim_win_get_next(wp);
                }
            }
            tp = nvim_tabpage_get_next(tp);
        }
        WinHandle::null()
    }
}

/// If 'switchbuf' contains "useopen" or "usetab", jump to a window containing "buf".
///
/// Rust port of C `swbuf_goto_win_with_buf()`.
fn swbuf_goto_win_with_buf_impl(buf: BufHandle) -> WinHandle {
    if buf.is_null() {
        return WinHandle::null();
    }

    unsafe {
        // If 'switchbuf' contains "useopen": jump to first window in current tab.
        let mut wp = if nvim_swb_has_useopen() != 0 {
            buf_jump_open_win_impl(buf)
        } else {
            WinHandle::null()
        };

        // If 'switchbuf' contains "usetab": jump to first window in any tab.
        if wp.is_null() && nvim_swb_has_usetab() != 0 {
            wp = buf_jump_open_tab_impl(buf);
        }

        wp
    }
}

/// FFI: Jump to first open window for buffer in current tab.
#[unsafe(no_mangle)]
pub extern "C" fn rs_buf_jump_open_win(buf: BufHandle) -> WinHandle {
    buf_jump_open_win_impl(buf)
}

/// C export: `buf_jump_open_win` — eliminates the C thin wrapper.
#[unsafe(export_name = "buf_jump_open_win")]
pub extern "C" fn buf_jump_open_win(buf: BufHandle) -> WinHandle {
    buf_jump_open_win_impl(buf)
}

/// FFI: Jump to first open window for buffer in any tab.
#[unsafe(no_mangle)]
pub extern "C" fn rs_buf_jump_open_tab(buf: BufHandle) -> WinHandle {
    buf_jump_open_tab_impl(buf)
}

/// C export: `buf_jump_open_tab` — eliminates the C thin wrapper.
#[unsafe(export_name = "buf_jump_open_tab")]
pub extern "C" fn buf_jump_open_tab(buf: BufHandle) -> WinHandle {
    buf_jump_open_tab_impl(buf)
}

/// FFI: Jump to window with buf using 'switchbuf' flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_swbuf_goto_win_with_buf(buf: BufHandle) -> WinHandle {
    swbuf_goto_win_with_buf_impl(buf)
}

/// C export: `swbuf_goto_win_with_buf` — eliminates the C thin wrapper.
#[unsafe(export_name = "swbuf_goto_win_with_buf")]
pub extern "C" fn swbuf_goto_win_with_buf(buf: BufHandle) -> WinHandle {
    swbuf_goto_win_with_buf_impl(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_inputs() {
        assert!(find_by_nr_impl(0).is_null());
        assert!(find_by_nr_impl(-1).is_null());
        assert!(find_by_buffer_curtab_impl(BufHandle::null()).is_null());
        assert_eq!(count_windows_for_buffer_impl(BufHandle::null()), 0);
    }

    #[test]
    fn test_null_window_winnr() {
        let null_wp = WinHandle::null();
        assert_eq!(get_winnr_impl(null_wp), 0);
    }
}
