//! Split execution helper functions.
//!
//! This module provides helper functions for window split execution,
//! supporting the C implementation of win_split_ins, make_windows, etc.

// Option values and window dimensions may need truncation when converting
// between i64 and c_int, but values are guaranteed to be in valid range.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::similar_names)]

use std::ffi::c_int;

use crate::frame::constants::{STATUS_HEIGHT, WSP_ABOVE, WSP_BOT, WSP_ROOM, WSP_TOP, WSP_VERT};
use crate::{Frame, WinHandle, FR_COL, FR_ROW};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get firstwin.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get p_wmw (winminwidth option).
    fn nvim_get_p_wmw() -> i64;

    /// Get p_wmh (winminheight option).
    fn nvim_get_p_wmh() -> i64;

    /// Get p_wiw (winwidth option).
    fn nvim_get_p_wiw() -> i64;

    /// Get p_wh (winheight option).
    fn nvim_get_p_wh() -> i64;

    /// Get w_width from a window.
    fn nvim_win_field_width(wp: WinHandle) -> c_int;

    /// Get w_height from a window.
    fn nvim_win_field_height(wp: WinHandle) -> c_int;

    /// Get w_vsep_width from a window.
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;

    /// Get w_hsep_height from a window.
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;

    /// Get w_status_height from a window.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    /// Get w_frame from a window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get w_p_wfw (winfixwidth option) from a window.
    fn nvim_win_get_wfw(wp: WinHandle) -> c_int;

    /// Get w_p_wfh (winfixheight option) from a window.
    fn nvim_win_get_wfh(wp: WinHandle) -> c_int;

    /// Check if window is floating.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get global_winbar_height.
    #[link_name = "rs_global_winbar_height"]
    fn global_winbar_height() -> c_int;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get lastwin_nofloating().
    #[link_name = "rs_lastwin_nofloating"]
    fn lastwin_nofloating() -> WinHandle;
}

// =============================================================================
// Split Size Calculations
// =============================================================================

/// Calculate the maximum number of windows that can be created from a split.
///
/// This implements the maxcount calculation from make_windows().
fn calculate_max_windows_impl(vertical: bool) -> c_int {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return 2;
        }

        let maxcount = if vertical {
            // Each window needs at least 'winminwidth' lines and a separator column.
            let w_width = nvim_win_field_width(curwin);
            let w_vsep = nvim_win_get_vsep_width(curwin);
            let p_wiw = nvim_get_p_wiw() as c_int;
            let p_wmw = nvim_get_p_wmw() as c_int;
            (w_width + w_vsep - (p_wiw - p_wmw)) / (p_wmw + 1)
        } else {
            // Each window needs at least 'winminheight' lines.
            // If statusline isn't global, each window also needs a statusline.
            // If 'winbar' is set, each window also needs a winbar.
            let w_height = nvim_win_field_height(curwin);
            let w_hsep = nvim_win_get_hsep_height(curwin);
            let w_status = nvim_win_get_status_height(curwin);
            let p_wh = nvim_get_p_wh() as c_int;
            let p_wmh = nvim_get_p_wmh() as c_int;
            let winbar = global_winbar_height();
            (w_height + w_hsep + w_status - (p_wh - p_wmh)) / (p_wmh + STATUS_HEIGHT + winbar)
        };

        maxcount.max(2)
    }
}

/// Calculate the needed space for a new split.
///
/// For vertical splits: minwidth + separator
/// For horizontal splits: minheight + status line
fn calculate_needed_space_impl(flags: c_int, with_room: bool) -> c_int {
    let vertical = (flags & WSP_VERT) != 0;

    unsafe {
        if vertical {
            let wmw1 = (nvim_get_p_wmw() as c_int).max(1);
            let mut needed = wmw1 + 1;
            if with_room {
                let p_wiw = nvim_get_p_wiw() as c_int;
                needed += p_wiw - wmw1;
            }
            needed
        } else {
            let wmh1 = (nvim_get_p_wmh() as c_int).max(1);
            let mut needed = wmh1 + STATUS_HEIGHT;
            if with_room {
                let p_wh = nvim_get_p_wh() as c_int;
                needed += p_wh - wmh1;
            }
            needed
        }
    }
}

/// Calculate split size for make_windows iteration.
///
/// This computes the size for each split in the make_windows loop.
fn calculate_split_size_for_iteration_impl(vertical: bool, todo: c_int) -> c_int {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() || todo <= 0 {
            return 0;
        }

        if vertical {
            let w_width = nvim_win_field_width(curwin);
            w_width - (w_width - todo) / (todo + 1) - 1
        } else {
            let w_height = nvim_win_field_height(curwin);
            w_height - (w_height - todo * STATUS_HEIGHT) / (todo + 1) - STATUS_HEIGHT
        }
    }
}

/// Get the default size for a new split window.
///
/// Returns half of the old window's dimension.
fn get_default_new_size_impl(flags: c_int, oldwin: WinHandle) -> c_int {
    if oldwin.is_null() {
        return 0;
    }

    let vertical = (flags & WSP_VERT) != 0;

    unsafe {
        if vertical {
            nvim_win_field_width(oldwin) / 2
        } else {
            nvim_win_field_height(oldwin) / 2
        }
    }
}

/// Constrain a new split size to valid bounds.
///
/// Makes sure the new size is at least min width/height and leaves
/// room for the old window.
fn constrain_new_size_impl(
    flags: c_int,
    new_size: c_int,
    available: c_int,
    minsize: c_int,
) -> c_int {
    let vertical = (flags & WSP_VERT) != 0;

    unsafe {
        let min_win_size = if vertical {
            (nvim_get_p_wmw() as c_int).max(1)
        } else {
            (nvim_get_p_wmh() as c_int).max(1)
        };

        let separator = if vertical { 1 } else { STATUS_HEIGHT };

        // new_size = MAX(MIN(new_size, available - minsize - separator), min_win_size)
        new_size
            .min(available - minsize - separator)
            .max(min_win_size)
    }
}

/// Check if window needs equalization after split.
///
/// Returns true if the new window doesn't fit in the current window's
/// available space.
fn needs_equalization_impl(flags: c_int, new_size: c_int, oldwin: WinHandle) -> bool {
    if oldwin.is_null() {
        return false;
    }

    let vertical = (flags & WSP_VERT) != 0;

    unsafe {
        if vertical {
            let old_width = nvim_win_field_width(oldwin);
            let p_wmw = nvim_get_p_wmw() as c_int;
            old_width - new_size - 1 < p_wmw
        } else {
            let old_height = nvim_win_field_height(oldwin);
            let p_wmh = nvim_get_p_wmh() as c_int;
            old_height - new_size - STATUS_HEIGHT < p_wmh
        }
    }
}

/// Get the old window for a split operation.
///
/// Based on flags, returns firstwin, lastwin_nofloating, or curwin.
fn get_oldwin_for_split_impl(flags: c_int) -> WinHandle {
    unsafe {
        let curwin = nvim_get_curwin();

        if (flags & WSP_TOP) != 0 {
            nvim_get_firstwin()
        } else if (flags & WSP_BOT) != 0 || nvim_win_get_floating(curwin) != 0 {
            lastwin_nofloating()
        } else {
            curwin
        }
    }
}

/// Get the frame layout type for a split direction.
///
/// Vertical splits create FR_ROW layouts, horizontal create FR_COL.
fn get_split_layout_type_impl(flags: c_int) -> i8 {
    if (flags & WSP_VERT) != 0 {
        FR_ROW
    } else {
        FR_COL
    }
}

/// Check if a window has fixed width/height and should be skipped.
fn window_is_fixed_impl(wp: WinHandle, vertical: bool) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        if vertical {
            nvim_win_get_wfw(wp) != 0
        } else {
            nvim_win_get_wfh(wp) != 0
        }
    }
}

/// Calculate flags for make_windows split calls.
fn make_windows_split_flags_impl(vertical: bool) -> c_int {
    if vertical {
        WSP_VERT | WSP_ABOVE
    } else {
        WSP_ABOVE
    }
}

/// Check if a toplevel split is being performed.
fn is_toplevel_split_impl(flags: c_int) -> bool {
    (flags & (WSP_TOP | WSP_BOT)) != 0
}

/// Get the available space for a split from topframe.
fn get_toplevel_available_impl(vertical: bool) -> c_int {
    unsafe {
        let topframe = nvim_get_topframe();
        if topframe.is_null() {
            return 0;
        }

        if vertical {
            (*topframe).fr_width
        } else {
            (*topframe).fr_height
        }
    }
}

/// Get the available space for a split from a window's frame.
fn get_window_available_impl(wp: WinHandle, vertical: bool) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let frame = nvim_win_get_frame(wp);
        if frame.is_null() {
            return 0;
        }

        if vertical {
            (*frame).fr_width
        } else {
            (*frame).fr_height
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Calculate maximum windows for make_windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_max_windows(vertical: c_int) -> c_int {
    calculate_max_windows_impl(vertical != 0)
}

/// FFI: Calculate needed space for a new split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_needed_space(flags: c_int) -> c_int {
    let with_room = (flags & WSP_ROOM) != 0;
    calculate_needed_space_impl(flags, with_room)
}

/// FFI: Calculate split size for make_windows iteration.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_iteration_size(vertical: c_int, todo: c_int) -> c_int {
    calculate_split_size_for_iteration_impl(vertical != 0, todo)
}

/// FFI: Get default new size for split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_default_new_size(flags: c_int, oldwin: WinHandle) -> c_int {
    get_default_new_size_impl(flags, oldwin)
}

/// FFI: Constrain new size to valid bounds.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_constrain_size(
    flags: c_int,
    new_size: c_int,
    available: c_int,
    minsize: c_int,
) -> c_int {
    constrain_new_size_impl(flags, new_size, available, minsize)
}

/// FFI: Check if equalization is needed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_needs_equal(flags: c_int, new_size: c_int, oldwin: WinHandle) -> c_int {
    c_int::from(needs_equalization_impl(flags, new_size, oldwin))
}

/// FFI: Get old window for split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_get_oldwin(flags: c_int) -> WinHandle {
    get_oldwin_for_split_impl(flags)
}

/// FFI: Get frame layout type for split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_layout_type(flags: c_int) -> c_int {
    c_int::from(get_split_layout_type_impl(flags))
}

/// FFI: Check if window is fixed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_win_is_fixed(wp: WinHandle, vertical: c_int) -> c_int {
    c_int::from(window_is_fixed_impl(wp, vertical != 0))
}

/// FFI: Get make_windows split flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_make_windows_flags(vertical: c_int) -> c_int {
    make_windows_split_flags_impl(vertical != 0)
}

/// FFI: Check if toplevel split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_is_toplevel(flags: c_int) -> c_int {
    c_int::from(is_toplevel_split_impl(flags))
}

/// FFI: Get toplevel available space.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_toplevel_available(vertical: c_int) -> c_int {
    get_toplevel_available_impl(vertical != 0)
}

/// FFI: Get window available space.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_window_available(wp: WinHandle, vertical: c_int) -> c_int {
    get_window_available_impl(wp, vertical != 0)
}

/// FFI: Calculate remaining size for old window after split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_remaining_size(flags: c_int, old_size: c_int, new_size: c_int) -> c_int {
    let separator = if (flags & WSP_VERT) != 0 {
        1
    } else {
        STATUS_HEIGHT
    };
    (old_size - new_size - separator).max(0)
}

/// FFI: Check if split direction is vertical.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_is_vertical(flags: c_int) -> c_int {
    c_int::from((flags & WSP_VERT) != 0)
}

/// FFI: Check if split direction places new window above/left.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_is_above(flags: c_int) -> c_int {
    c_int::from((flags & WSP_ABOVE) != 0)
}

/// FFI: Get minimum window dimension for split.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_min_dimension(vertical: c_int) -> c_int {
    unsafe {
        if vertical != 0 {
            (nvim_get_p_wmw() as c_int).max(1)
        } else {
            (nvim_get_p_wmh() as c_int).max(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_layout_type() {
        assert_eq!(get_split_layout_type_impl(WSP_VERT), FR_ROW);
        assert_eq!(get_split_layout_type_impl(0), FR_COL);
    }

    #[test]
    fn test_is_toplevel() {
        assert!(is_toplevel_split_impl(WSP_TOP));
        assert!(is_toplevel_split_impl(WSP_BOT));
        assert!(is_toplevel_split_impl(WSP_TOP | WSP_BOT));
        assert!(!is_toplevel_split_impl(WSP_VERT));
        assert!(!is_toplevel_split_impl(0));
    }

    #[test]
    fn test_make_windows_flags() {
        assert_eq!(make_windows_split_flags_impl(true), WSP_VERT | WSP_ABOVE);
        assert_eq!(make_windows_split_flags_impl(false), WSP_ABOVE);
    }

    #[test]
    fn test_remaining_size() {
        // Vertical: separator is 1
        assert_eq!(rs_split_remaining_size(WSP_VERT, 80, 40), 39);
        // Horizontal: separator is STATUS_HEIGHT (1)
        assert_eq!(rs_split_remaining_size(0, 24, 12), 11);
        // Don't go negative
        assert_eq!(rs_split_remaining_size(WSP_VERT, 10, 10), 0);
    }
}
