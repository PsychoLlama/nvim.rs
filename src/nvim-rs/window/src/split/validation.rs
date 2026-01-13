//! Split validation functions.
//!
//! This module provides validation functions for window splitting operations,
//! checking whether splits are allowed and calculating minimum sizes.

// Option values (p_wmh, p_wmw) are i64 but are guaranteed to be small
// enough to fit in c_int, so truncation is safe.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::only_used_in_recursion)]

use std::ffi::c_int;

use crate::frame::constants::{MIN_LINES, STATUS_HEIGHT, WSP_VERT};
use crate::{Frame, TabpageHandle, WinHandle, FR_COL, FR_ROW};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get current tabpage.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get first tabpage.
    fn nvim_get_first_tabpage() -> TabpageHandle;

    /// Get next tabpage.
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;

    /// Get topframe for a tabpage.
    fn nvim_tabpage_get_topframe(tp: TabpageHandle) -> *mut Frame;

    /// Get firstwin for a tabpage.
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    /// Get global topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get w_winbar_height from a window.
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    /// Get w_status_height from a window.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    /// Get w_hsep_height from a window.
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;

    /// Get w_next from a window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Check if window is floating.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get tabline height.
    fn tabline_height() -> c_int;

    /// Get global statusline height.
    fn global_stl_height() -> c_int;

    /// Get p_wmh (winminheight option).
    fn nvim_get_p_wmh() -> i64;

    /// Get p_wmw (winminwidth option).
    fn nvim_get_p_wmw() -> i64;
}

// =============================================================================
// Minimum Size Calculations
// =============================================================================

/// Calculate minimum height for a single window.
///
/// This accounts for winbar, status line, and separator.
#[inline]
fn min_win_height(wp: WinHandle, include_status: bool) -> c_int {
    if wp.is_null() {
        return 1;
    }

    unsafe {
        let mut height = nvim_get_p_wmh() as c_int;
        height += nvim_win_get_winbar_height(wp);
        if include_status {
            height += nvim_win_get_status_height(wp);
            height += nvim_win_get_hsep_height(wp);
        }
        height.max(1)
    }
}

/// Calculate minimum height for a frame tree.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid Frame pointer.
unsafe fn frame_min_height_impl(frp: *const Frame, next_curwin: WinHandle) -> c_int {
    if frp.is_null() {
        return 0;
    }

    let frame = &*frp;

    if frame.fr_layout == FR_ROW {
        // Row layout: max height of children
        let mut max_height = 0;
        let mut child = frame.fr_child;
        while !child.is_null() {
            let h = frame_min_height_impl(child, next_curwin);
            if h > max_height {
                max_height = h;
            }
            child = (*child).fr_next;
        }
        return max_height;
    }

    if frame.fr_layout == FR_COL {
        // Column layout: sum of children heights
        let mut total = 0;
        let mut child = frame.fr_child;
        while !child.is_null() {
            total += frame_min_height_impl(child, next_curwin);
            child = (*child).fr_next;
        }
        return total;
    }

    // Leaf frame
    let wp = frame.fr_win;
    if wp.is_null() {
        return 1;
    }

    // Include status line unless this is next_curwin
    let include_status = wp != next_curwin;
    min_win_height(wp, include_status)
}

/// Calculate minimum width for a frame tree.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid Frame pointer.
unsafe fn frame_min_width_impl(frp: *const Frame, next_curwin: WinHandle) -> c_int {
    if frp.is_null() {
        return 0;
    }

    let frame = &*frp;

    if frame.fr_layout == FR_COL {
        // Column layout: max width of children
        let mut max_width = 0;
        let mut child = frame.fr_child;
        while !child.is_null() {
            let w = frame_min_width_impl(child, next_curwin);
            if w > max_width {
                max_width = w;
            }
            child = (*child).fr_next;
        }
        return max_width;
    }

    if frame.fr_layout == FR_ROW {
        // Row layout: sum of children widths
        let mut total = 0;
        let mut child = frame.fr_child;
        while !child.is_null() {
            total += frame_min_width_impl(child, next_curwin);
            child = (*child).fr_next;
        }
        return total;
    }

    // Leaf frame
    (nvim_get_p_wmw() as c_int).max(1)
}

/// Calculate minimum rows needed for a tabpage.
fn min_rows_for_tabpage(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        return MIN_LINES;
    }

    unsafe {
        let topframe = if tp == nvim_get_curtab() {
            nvim_get_topframe()
        } else {
            nvim_tabpage_get_topframe(tp)
        };

        let frame_height = frame_min_height_impl(topframe, WinHandle::null());

        // Add tabline, cmdline height, global statusline
        frame_height + tabline_height() + 1 + global_stl_height()
    }
}

/// Calculate minimum rows needed across all tabpages.
fn min_rows_for_all_tabpages_impl() -> c_int {
    unsafe {
        let mut max_rows = MIN_LINES;
        let mut tp = nvim_get_first_tabpage();

        while !tp.is_null() {
            let rows = min_rows_for_tabpage(tp);
            if rows > max_rows {
                max_rows = rows;
            }
            tp = nvim_tabpage_get_next(tp);
        }

        max_rows
    }
}

// =============================================================================
// Split Validation
// =============================================================================

/// Check if there's room for a split given total size and minimum size.
#[inline]
fn can_split_impl(total_size: c_int, min_size: c_int) -> bool {
    // Need room for at least two windows of minimum size
    total_size >= min_size * 2 + STATUS_HEIGHT
}

/// Check if vertical split is possible given current dimensions.
fn can_split_vertical_impl(available_width: c_int) -> bool {
    let min_width = unsafe { (nvim_get_p_wmw() as c_int).max(1) };
    // Need room for two windows plus separator
    available_width > min_width * 2
}

/// Check if horizontal split is possible given current dimensions.
fn can_split_horizontal_impl(available_height: c_int) -> bool {
    let min_height = unsafe { (nvim_get_p_wmh() as c_int).max(1) };
    // Need room for two windows plus status line
    available_height >= min_height * 2 + STATUS_HEIGHT
}

/// Determine if a split is allowed based on flags.
fn check_split_possible_impl(flags: c_int, width: c_int, height: c_int) -> bool {
    if (flags & WSP_VERT) != 0 {
        can_split_vertical_impl(width)
    } else {
        can_split_horizontal_impl(height)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Calculate minimum height for a frame tree.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid Frame pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_split_frame_min_height(
    frp: *const Frame,
    next_curwin: WinHandle,
) -> c_int {
    frame_min_height_impl(frp, next_curwin)
}

/// FFI: Calculate minimum width for a frame tree.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid Frame pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_split_frame_min_width(
    frp: *const Frame,
    next_curwin: WinHandle,
) -> c_int {
    frame_min_width_impl(frp, next_curwin)
}

/// FFI: Calculate minimum rows for current tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_min_rows_curtab() -> c_int {
    unsafe { min_rows_for_tabpage(nvim_get_curtab()) }
}

/// FFI: Calculate minimum rows for a specific tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_min_rows_tabpage(tp: TabpageHandle) -> c_int {
    min_rows_for_tabpage(tp)
}

/// FFI: Calculate minimum rows for all tabpages.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_min_rows_all_tabpages() -> c_int {
    min_rows_for_all_tabpages_impl()
}

/// FFI: Check if split is possible given total and minimum size.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_can_split(total_size: c_int, min_size: c_int) -> c_int {
    c_int::from(can_split_impl(total_size, min_size))
}

/// FFI: Check if vertical split is possible.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_can_split_vertical(available_width: c_int) -> c_int {
    c_int::from(can_split_vertical_impl(available_width))
}

/// FFI: Check if horizontal split is possible.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_can_split_horizontal(available_height: c_int) -> c_int {
    c_int::from(can_split_horizontal_impl(available_height))
}

/// FFI: Check if split with given flags is possible.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_check_possible(flags: c_int, width: c_int, height: c_int) -> c_int {
    c_int::from(check_split_possible_impl(flags, width, height))
}

/// FFI: Get minimum window height (accounting for winbar etc).
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_min_win_height(wp: WinHandle, include_status: c_int) -> c_int {
    min_win_height(wp, include_status != 0)
}

/// FFI: Get minimum window width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_min_win_width() -> c_int {
    unsafe { (nvim_get_p_wmw() as c_int).max(1) }
}

/// FFI: Calculate required size for a split.
/// Returns the minimum size needed for two windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_required_size(flags: c_int) -> c_int {
    if (flags & WSP_VERT) != 0 {
        // Vertical split: need width for 2 windows + separator
        let min_width = unsafe { (nvim_get_p_wmw() as c_int).max(1) };
        min_width * 2 + 1
    } else {
        // Horizontal split: need height for 2 windows + status
        let min_height = unsafe { (nvim_get_p_wmh() as c_int).max(1) };
        min_height * 2 + STATUS_HEIGHT
    }
}

/// FFI: Get the default split size when none specified.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_default_size(flags: c_int, total_size: c_int) -> c_int {
    // Default is half of available space
    let half = total_size / 2;
    if (flags & WSP_VERT) != 0 {
        let min_width = unsafe { (nvim_get_p_wmw() as c_int).max(1) };
        half.max(min_width)
    } else {
        let min_height = unsafe { (nvim_get_p_wmh() as c_int).max(1) };
        half.max(min_height)
    }
}

/// FFI: Clamp split size to valid range.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_clamp_size(flags: c_int, requested: c_int, total_size: c_int) -> c_int {
    let (min_size, max_size) = if (flags & WSP_VERT) != 0 {
        let min = unsafe { (nvim_get_p_wmw() as c_int).max(1) };
        let max = total_size - min - 1; // Leave room for other window + separator
        (min, max)
    } else {
        let min = unsafe { (nvim_get_p_wmh() as c_int).max(1) };
        let max = total_size - min - STATUS_HEIGHT;
        (min, max)
    };

    requested.clamp(min_size, max_size.max(min_size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::constants::MIN_COLUMNS;

    #[test]
    fn test_can_split() {
        // With status height of 1, need at least 2*min + 1
        assert!(can_split_impl(11, 5)); // 11 >= 10 + 1
        assert!(!can_split_impl(10, 5)); // 10 < 10 + 1
        assert!(can_split_impl(3, 1)); // 3 >= 2 + 1
        assert!(!can_split_impl(2, 1)); // 2 < 2 + 1
    }

    #[test]
    fn test_constants() {
        assert_eq!(STATUS_HEIGHT, 1);
        assert_eq!(MIN_LINES, 2);
        assert_eq!(MIN_COLUMNS, 12);
    }
}
