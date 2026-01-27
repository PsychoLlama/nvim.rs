//! Window equalization functions.
//!
//! This module provides Rust helper functions for window equalization operations
//! from `src/nvim/window.c`, including win_equal, win_equal_rec, and related
//! size distribution functions.

// Window dimensions may need truncation when converting between types.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL};

/// Status height constant (same as in C).
const STATUS_HEIGHT: c_int = 1;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get w_winbar_height from window.
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    /// Get global p_wmh (winminheight).
    fn nvim_get_p_wmh() -> i64;

    /// Get global p_wmw (winminwidth).
    fn nvim_get_p_wmw() -> i64;

    /// Get global p_wh (winheight).
    fn nvim_get_p_wh() -> i64;

    /// Get global p_wiw (winwidth).
    fn nvim_get_p_wiw() -> i64;

    /// Get global cmdline_row.
    fn nvim_get_cmdline_row() -> c_int;

    /// Get global p_ls (laststatus).
    fn nvim_get_p_ls() -> i64;

    /// Get global Columns.
    fn nvim_get_Columns() -> c_int;

    /// Check if winbar is globally enabled.
    fn nvim_global_winbar_height() -> c_int;

    /// Get global_stl_height().
    fn nvim_global_stl_height() -> c_int;

    /// Frame minheight calculation.
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;

    /// Frame minwidth calculation.
    fn rs_frame_minwidth(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;

    /// Check if frame has window.
    fn rs_frame_has_win(frp: *const Frame, wp: WinHandle) -> c_int;

    /// Check if frame has fixed height.
    fn rs_frame_fixed_height(frp: *const Frame) -> c_int;

    /// Check if frame has fixed width.
    fn rs_frame_fixed_width(frp: *const Frame) -> c_int;
}

/// Get the window from a frame (first leaf window).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
unsafe fn frame2win(frp: *const Frame) -> WinHandle {
    if frp.is_null() {
        return WinHandle::null();
    }

    let frame = &*frp;
    if !frame.fr_win.is_null() {
        return frame.fr_win;
    }

    // Not a leaf, recurse to first child
    if !frame.fr_child.is_null() {
        return frame2win(frame.fr_child);
    }

    WinHandle::null()
}

// =============================================================================
// Maximum Window Count Calculation
// =============================================================================

/// Compute maximum number of windows that can fit within "height" in frame "fr".
///
/// This is the Rust equivalent of `get_maximum_wincount()` in window.c.
fn get_maximum_wincount_impl(frp: *const Frame, height: c_int) -> c_int {
    if frp.is_null() || height <= 0 {
        return 0;
    }

    unsafe {
        let frame = &*frp;
        let p_wmh = nvim_get_p_wmh() as c_int;

        if frame.fr_layout != FR_COL {
            // Not a column layout - simple calculation
            let wp = frame2win(frp);
            let winbar_height = if wp.is_null() {
                0
            } else {
                nvim_win_get_winbar_height(wp)
            };
            return height / (p_wmh + STATUS_HEIGHT + winbar_height);
        }

        // Check if winbar is globally enabled
        if nvim_global_winbar_height() > 0 {
            return height / (p_wmh + STATUS_HEIGHT + 1);
        }

        // Column layout - sum up children
        let mut total_wincount = 0;
        let mut child = frame.fr_child;

        while !child.is_null() {
            let child_frame = &*child;
            let wp = frame2win(child);
            let winbar_height = if wp.is_null() {
                0
            } else {
                nvim_win_get_winbar_height(wp)
            };

            let child_min = p_wmh + STATUS_HEIGHT + winbar_height;
            if child_min > 0 {
                // Each child contributes at least one window worth of count
                total_wincount += height / child_min;
            }

            child = child_frame.fr_next;
        }

        // Return at least 1 if there are any children
        if total_wincount == 0 && !frame.fr_child.is_null() {
            return 1;
        }

        total_wincount
    }
}

// =============================================================================
// Equalization Helpers
// =============================================================================

/// Calculate extra separator adjustment for vertical equalization.
///
/// Returns 1 if this is the rightmost column (no separator needed), 0 otherwise.
fn compute_extra_sep_horizontal_impl(col: c_int, width: c_int) -> c_int {
    unsafe { c_int::from(col + width == nvim_get_Columns()) }
}

/// Calculate extra separator adjustment for horizontal equalization.
///
/// Returns STATUS_HEIGHT if at bottom without statusline, 1 if global statusline,
/// 0 otherwise.
fn compute_extra_sep_vertical_impl(row: c_int, height: c_int) -> c_int {
    unsafe {
        let cmdline_row = nvim_get_cmdline_row();
        let p_ls = nvim_get_p_ls();

        if row + height >= cmdline_row && p_ls == 0 {
            STATUS_HEIGHT
        } else {
            c_int::from(nvim_global_stl_height() > 0)
        }
    }
}

/// Calculate total window count for horizontal (FR_ROW) equalization.
fn compute_total_wincount_horizontal_impl(frp: *const Frame, extra_sep: c_int) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let p_wmw = nvim_get_p_wmw() as c_int;
        let n = rs_frame_minwidth(frp, WinHandle::null());
        (n + extra_sep) / (p_wmw + 1)
    }
}

/// Calculate total window count for vertical (FR_COL) equalization.
fn compute_total_wincount_vertical_impl(frp: *const Frame, extra_sep: c_int) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let n = rs_frame_minheight(frp, WinHandle::null());
        get_maximum_wincount_impl(frp, n + extra_sep)
    }
}

/// Calculate room available for width distribution.
fn compute_room_horizontal_impl(frp: *const Frame, width: c_int, next_curwin: WinHandle) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let m = rs_frame_minwidth(frp, next_curwin);
        (width - m).max(0)
    }
}

/// Calculate room available for height distribution.
fn compute_room_vertical_impl(frp: *const Frame, height: c_int, next_curwin: WinHandle) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let m = rs_frame_minheight(frp, next_curwin);
        (height - m).max(0)
    }
}

/// Calculate the size for next_curwin in horizontal equalization.
///
/// Returns -1 if size should be computed later based on room distribution.
fn compute_next_curwin_width_impl(
    frp: *const Frame,
    width: c_int,
    next_curwin: WinHandle,
) -> c_int {
    if frp.is_null() || next_curwin.is_null() {
        return 0;
    }

    unsafe {
        let p_wiw = nvim_get_p_wiw() as c_int;
        let m = rs_frame_minwidth(frp, next_curwin);
        let room = width - m;

        if room < 0 {
            // Not enough room - give next_curwin as much as possible
            (p_wiw + room).max(0)
        } else {
            -1 // Will be computed during distribution
        }
    }
}

/// Calculate the size for next_curwin in vertical equalization.
///
/// Returns -1 if size should be computed later based on room distribution.
fn compute_next_curwin_height_impl(
    frp: *const Frame,
    height: c_int,
    next_curwin: WinHandle,
) -> c_int {
    if frp.is_null() || next_curwin.is_null() {
        return 0;
    }

    unsafe {
        let p_wh = nvim_get_p_wh() as c_int;
        let m = rs_frame_minheight(frp, next_curwin);
        let room = height - m;

        if room < 0 {
            // Not enough room - give next_curwin as much as possible
            (p_wh + room).max(0)
        } else {
            -1 // Will be computed during distribution
        }
    }
}

/// Check if a frame should be skipped during equalization.
///
/// Returns true if the frame is the curwin and equalizing only current frame.
fn should_skip_frame_impl(
    frp: *const Frame,
    next_curwin: WinHandle,
    current: bool,
    dir: c_int,
    new_size: c_int,
    is_height: bool,
) -> bool {
    if frp.is_null() {
        return true;
    }

    unsafe {
        let frame = &*frp;

        // Skip if equalizing only current and this frame doesn't need change
        if current {
            let dir_char = if is_height { b'h' } else { b'v' };
            if dir == c_int::from(dir_char) && frame.fr_parent.is_null() {
                let current_size = if is_height {
                    frame.fr_height
                } else {
                    frame.fr_width
                };
                if new_size == current_size && rs_frame_has_win(frp, next_curwin) == 0 {
                    return true;
                }
            }
        }

        false
    }
}

/// Distribute room among windows proportionally.
///
/// Given total room and window count, calculate the share for a specific window.
fn distribute_room_impl(room: c_int, wincount: c_int, totwincount: c_int) -> c_int {
    if totwincount == 0 {
        return room;
    }
    (wincount * room + totwincount / 2) / totwincount
}

/// Check if frame contains any window with fixed height.
fn frame_has_fixed_height_window_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe { rs_frame_fixed_height(frp) != 0 }
}

/// Check if frame contains any window with fixed width.
fn frame_has_fixed_width_window_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe { rs_frame_fixed_width(frp) != 0 }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Compute maximum window count for a frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_max_wincount(frp: *const Frame, height: c_int) -> c_int {
    get_maximum_wincount_impl(frp, height)
}

/// FFI: Compute extra separator for horizontal equalization.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_extra_sep_h(col: c_int, width: c_int) -> c_int {
    compute_extra_sep_horizontal_impl(col, width)
}

/// FFI: Compute extra separator for vertical equalization.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_extra_sep_v(row: c_int, height: c_int) -> c_int {
    compute_extra_sep_vertical_impl(row, height)
}

/// FFI: Compute total window count for horizontal equalization.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_total_wincount_h(
    frp: *const Frame,
    extra_sep: c_int,
) -> c_int {
    compute_total_wincount_horizontal_impl(frp, extra_sep)
}

/// FFI: Compute total window count for vertical equalization.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_total_wincount_v(
    frp: *const Frame,
    extra_sep: c_int,
) -> c_int {
    compute_total_wincount_vertical_impl(frp, extra_sep)
}

/// FFI: Compute room for horizontal distribution.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_room_h(
    frp: *const Frame,
    width: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_room_horizontal_impl(frp, width, next_curwin)
}

/// FFI: Compute room for vertical distribution.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_room_v(
    frp: *const Frame,
    height: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_room_vertical_impl(frp, height, next_curwin)
}

/// FFI: Compute next_curwin width.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_next_curwin_width(
    frp: *const Frame,
    width: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_next_curwin_width_impl(frp, width, next_curwin)
}

/// FFI: Compute next_curwin height.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_next_curwin_height(
    frp: *const Frame,
    height: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_next_curwin_height_impl(frp, height, next_curwin)
}

/// FFI: Check if frame should be skipped during equalization.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_should_skip(
    frp: *const Frame,
    next_curwin: WinHandle,
    current: c_int,
    dir: c_int,
    new_size: c_int,
    is_height: c_int,
) -> c_int {
    c_int::from(should_skip_frame_impl(
        frp,
        next_curwin,
        current != 0,
        dir,
        new_size,
        is_height != 0,
    ))
}

/// FFI: Distribute room among windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_distribute_room(
    room: c_int,
    wincount: c_int,
    totwincount: c_int,
) -> c_int {
    distribute_room_impl(room, wincount, totwincount)
}

/// FFI: Check if frame has fixed height window.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_has_fixed_height(frp: *const Frame) -> c_int {
    c_int::from(frame_has_fixed_height_window_impl(frp))
}

/// FFI: Check if frame has fixed width window.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_has_fixed_width(frp: *const Frame) -> c_int {
    c_int::from(frame_has_fixed_width_window_impl(frp))
}

/// FFI: Get p_wh (winheight) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wh() -> c_int {
    unsafe { nvim_get_p_wh() as c_int }
}

/// FFI: Get p_wiw (winwidth) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wiw() -> c_int {
    unsafe { nvim_get_p_wiw() as c_int }
}

/// FFI: Get p_wmh (winminheight) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wmh() -> c_int {
    unsafe { nvim_get_p_wmh() as c_int }
}

/// FFI: Get p_wmw (winminwidth) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wmw() -> c_int {
    unsafe { nvim_get_p_wmw() as c_int }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_frame_max_wincount() {
        assert_eq!(get_maximum_wincount_impl(std::ptr::null(), 100), 0);
    }

    #[test]
    fn test_zero_height_max_wincount() {
        // With null frame, should return 0
        assert_eq!(get_maximum_wincount_impl(std::ptr::null(), 0), 0);
    }

    #[test]
    fn test_distribute_room() {
        // Test even distribution
        assert_eq!(distribute_room_impl(100, 1, 4), 25);
        assert_eq!(distribute_room_impl(100, 2, 4), 50);

        // Test with rounding
        assert_eq!(distribute_room_impl(10, 1, 3), 3); // 10/3 ≈ 3.33, rounds to 3

        // Test edge cases
        assert_eq!(distribute_room_impl(100, 1, 0), 100); // No windows, get all room
        assert_eq!(distribute_room_impl(0, 1, 4), 0); // No room to distribute
    }

    #[test]
    fn test_null_frame_helpers() {
        let null_frame: *const Frame = std::ptr::null();
        assert!(!frame_has_fixed_height_window_impl(null_frame));
        assert!(!frame_has_fixed_width_window_impl(null_frame));
        assert!(!should_skip_frame_impl(
            null_frame,
            WinHandle::null(),
            false,
            0,
            10,
            true
        ));
    }
}
