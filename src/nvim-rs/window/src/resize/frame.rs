//! Frame tree helpers for window resizing.
//!
//! This module provides helper functions for frame tree manipulation
//! during resize operations, including constraint propagation and
//! size distribution.

// Option values and window dimensions may need truncation when converting
// between i64 and c_int, but values are guaranteed to be in valid range.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get p_wmh (winminheight).
    fn nvim_get_p_wmh() -> i64;

    /// Get p_wmw (winminwidth).
    fn nvim_get_p_wmw() -> i64;

    /// Check if frame has fixed width (winfixwidth set).
    fn rs_frame_fixed_width(frp: *const Frame) -> c_int;

    /// Get minimum width of a frame.
    fn rs_frame_minwidth(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;

    /// Set new width of a window.
    fn rs_win_new_width(wp: WinHandle, width: c_int);

    /// Get w_vsep_width from a window.
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;

    /// Set w_vsep_width on a window.
    fn nvim_win_set_vsep_width(wp: WinHandle, val: c_int);

    /// Check if frame has fixed height (winfixheight set).
    fn rs_frame_fixed_height(frp: *const Frame) -> c_int;

    /// Get minimum height of a frame.
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;

    /// Set new height of a window.
    fn rs_win_new_height(wp: WinHandle, height: c_int);

    /// Check if a window is at the bottom of the screen.
    fn rs_is_bottom_win(wp: WinHandle) -> c_int;

    /// Get w_hsep_height from a window.
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;

    /// Set w_hsep_height on a window.
    fn nvim_win_set_hsep_height(wp: WinHandle, val: c_int);

    /// Get w_status_height from a window.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    // nvim_set_cmdheight_option removed: logic inlined in Rust (Phase 8)

    /// Call set_option_value(kOptCmdheight, val, 0) (Phase 8).
    fn nvim_set_option_cmdheight(val: i64);

    /// Set min_set_ch static (Phase 8).
    fn nvim_set_min_set_ch(val: i64);

    /// Get p_ch (global cmdheight option value).
    fn nvim_get_window_p_ch() -> i64;

    /// Get min_set_ch (minimum cmdheight as set by user).
    fn nvim_get_min_set_ch() -> i64;

    /// Get ROWS_AVAIL (usable screen rows).
    fn nvim_get_rows_avail() -> c_int;
}

// =============================================================================
// Frame Size Distribution
// =============================================================================

/// Calculate new heights for frame and its siblings when growing.
///
/// Returns the amount actually distributed.
fn calculate_grow_distribution_impl(frp: *const Frame, delta: c_int) -> c_int {
    if frp.is_null() || delta <= 0 {
        return 0;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        if parent.is_null() || (*parent).fr_layout != FR_COL {
            return 0;
        }

        // Calculate how much siblings can give
        let mut available = 0;
        let mut child = (*parent).fr_child;
        while !child.is_null() {
            if child != frp.cast_mut() {
                let min_height = (nvim_get_p_wmh() as c_int).max(1);
                let room = (*child).fr_height - min_height;
                if room > 0 {
                    available += room;
                }
            }
            child = (*child).fr_next;
        }

        delta.min(available)
    }
}

/// Calculate how much each sibling should shrink when one grows.
///
/// Returns the shrink amount for the next sibling after frp.
fn calculate_sibling_shrink_impl(frp: *const Frame, total_shrink: c_int) -> c_int {
    if frp.is_null() || total_shrink <= 0 {
        return 0;
    }

    unsafe {
        let next = (*frp).fr_next;
        if next.is_null() {
            // No next sibling - distribute to prev
            return 0;
        }

        let min_height = (nvim_get_p_wmh() as c_int).max(1);
        let room = (*next).fr_height - min_height;
        room.min(total_shrink).max(0)
    }
}

// =============================================================================
// Frame Constraint Checking
// =============================================================================

/// Check if a frame height can be changed by delta.
fn frame_can_change_height_impl(frp: *const Frame, delta: c_int) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        if delta < 0 {
            // Shrinking - check we don't go below minimum
            let min_height = (nvim_get_p_wmh() as c_int).max(1);
            (*frp).fr_height + delta >= min_height
        } else {
            // Growing - always allowed if there's space in siblings
            true
        }
    }
}

/// Check if a frame width can be changed by delta.
fn frame_can_change_width_impl(frp: *const Frame, delta: c_int) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        if delta < 0 {
            // Shrinking - check we don't go below minimum
            let min_width = (nvim_get_p_wmw() as c_int).max(1);
            (*frp).fr_width + delta >= min_width
        } else {
            // Growing - always allowed if there's space in siblings
            true
        }
    }
}

/// Get maximum amount a frame height can decrease.
fn frame_max_height_decrease_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let min_height = (nvim_get_p_wmh() as c_int).max(1);
        ((*frp).fr_height - min_height).max(0)
    }
}

/// Get maximum amount a frame width can decrease.
fn frame_max_width_decrease_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let min_width = (nvim_get_p_wmw() as c_int).max(1);
        ((*frp).fr_width - min_width).max(0)
    }
}

// =============================================================================
// Frame Tree Traversal for Resize
// =============================================================================

/// Find the nearest ancestor with the specified layout.
fn find_ancestor_with_layout_impl(frp: *const Frame, layout: i8) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let topframe = nvim_get_topframe();
        let mut parent = (*frp).fr_parent;

        while !parent.is_null() && parent != topframe {
            if (*parent).fr_layout == layout {
                return parent;
            }
            parent = (*parent).fr_parent;
        }

        std::ptr::null_mut()
    }
}

/// Count children of a frame.
fn count_children_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let mut count = 0;
        let mut child = (*frp).fr_child;
        while !child.is_null() {
            count += 1;
            child = (*child).fr_next;
        }
        count
    }
}

/// Get the child at a specific index (0-based).
fn get_child_at_index_impl(frp: *const Frame, index: c_int) -> *mut Frame {
    if frp.is_null() || index < 0 {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut i = 0;
        let mut child = (*frp).fr_child;
        while !child.is_null() {
            if i == index {
                return child;
            }
            i += 1;
            child = (*child).fr_next;
        }
        std::ptr::null_mut()
    }
}

/// Get the index of a child in its parent (0-based).
fn get_child_index_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return -1;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        if parent.is_null() {
            return -1;
        }

        let mut index = 0;
        let mut child = (*parent).fr_child;
        while !child.is_null() {
            if child == frp.cast_mut() {
                return index;
            }
            index += 1;
            child = (*child).fr_next;
        }
        -1
    }
}

// =============================================================================
// Resize State Helpers
// =============================================================================

/// Check if frame is in a resizable configuration.
fn frame_is_resizable_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        // Must have a parent with appropriate layout
        let parent = (*frp).fr_parent;
        if parent.is_null() {
            return false;
        }

        let layout = (*parent).fr_layout;
        // Must be in a row or column layout with siblings
        layout == FR_COL || layout == FR_ROW
    }
}

/// Get the layout direction for resize operations.
///
/// Returns 'v' for vertical resizing (FR_COL), 'h' for horizontal (FR_ROW).
fn get_resize_direction_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let parent = (*frp).fr_parent;
        if parent.is_null() {
            return 0;
        }

        if (*parent).fr_layout == FR_COL {
            c_int::from(b'v')
        } else if (*parent).fr_layout == FR_ROW {
            c_int::from(b'h')
        } else {
            0
        }
    }
}

// =============================================================================
// frame_new_width / frame_new_height Implementations
// =============================================================================

/// Recursively set the width of a frame tree.
///
/// This is the Rust implementation of `frame_new_width()` from `window_shim.c`.
///
/// Handles:
/// - FR_LEAF: clears `w_vsep_width` if this is the rightmost window, then calls
///   `rs_win_new_width`
/// - FR_COL: propagates the same width to all children, restarting if any child
///   ends up wider
/// - FR_ROW: distributes extra columns starting from the leftmost (or rightmost
///   if `leftfirst` is false) non-fixed-width frame
///
/// # Safety
/// `topfrp` must be non-null and point to a valid Frame.
pub(crate) unsafe fn frame_new_width_impl(
    topfrp: *mut Frame,
    mut width: c_int,
    leftfirst: bool,
    wfw: bool,
) {
    let layout = (*topfrp).fr_layout;

    if layout == FR_LEAF {
        let wp = (*topfrp).fr_win;
        // Find out if there are any windows to the right of this one.
        let mut frp = topfrp;
        loop {
            let parent = (*frp).fr_parent;
            if parent.is_null() {
                break;
            }
            if (*parent).fr_layout == FR_ROW && !(*frp).fr_next.is_null() {
                break;
            }
            frp = parent;
        }
        if (*frp).fr_parent.is_null() {
            nvim_win_set_vsep_width(wp, 0);
        }
        rs_win_new_width(wp, width - nvim_win_get_vsep_width(wp));
    } else if layout == FR_COL {
        // All frames in this column get the same new width.
        loop {
            let mut frp = (*topfrp).fr_child;
            let mut restart = false;
            while !frp.is_null() {
                frame_new_width_impl(frp, width, leftfirst, wfw);
                if (*frp).fr_width > width {
                    // Could not fit the windows, make whole column wider.
                    width = (*frp).fr_width;
                    restart = true;
                    break;
                }
                frp = (*frp).fr_next;
            }
            if !restart {
                break;
            }
        }
    } else {
        // FR_ROW: distribute width across row frames.
        let mut frp = (*topfrp).fr_child;

        if wfw {
            // Advance past frames with 'winfixwidth' set.
            while rs_frame_fixed_width(frp) != 0 {
                frp = (*frp).fr_next;
                if frp.is_null() {
                    // No frame without wfw, give up.
                    (*topfrp).fr_width = width;
                    return;
                }
            }
        }

        if !leftfirst {
            // Find the rightmost frame of this row.
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            if wfw {
                // Advance back past frames with 'winfixwidth' set.
                while rs_frame_fixed_width(frp) != 0 {
                    frp = (*frp).fr_prev;
                }
            }
        }

        let mut extra_cols = width - (*topfrp).fr_width;
        if extra_cols < 0 {
            // Reduce frame width, rightmost (or leftmost) frame first.
            while !frp.is_null() {
                let w = rs_frame_minwidth(frp, WinHandle::null());
                if (*frp).fr_width + extra_cols < w {
                    extra_cols += (*frp).fr_width - w;
                    frame_new_width_impl(frp, w, leftfirst, wfw);
                } else {
                    frame_new_width_impl(frp, (*frp).fr_width + extra_cols, leftfirst, wfw);
                    break;
                }
                if leftfirst {
                    loop {
                        frp = (*frp).fr_next;
                        if !(wfw && !frp.is_null() && rs_frame_fixed_width(frp) != 0) {
                            break;
                        }
                    }
                } else {
                    loop {
                        frp = (*frp).fr_prev;
                        if !(wfw && !frp.is_null() && rs_frame_fixed_width(frp) != 0) {
                            break;
                        }
                    }
                }
                // Increase "width" if we could not reduce enough frames.
                if frp.is_null() {
                    width -= extra_cols;
                }
            }
        } else if extra_cols > 0 {
            // Increase width of rightmost (or leftmost) frame.
            frame_new_width_impl(frp, (*frp).fr_width + extra_cols, leftfirst, wfw);
        }
    }

    (*topfrp).fr_width = width;
}

/// Recursively set the height of a frame tree.
///
/// This is the Rust implementation of `frame_new_height()` from `window_shim.c`.
///
/// Handles:
/// - Top-frame + set_ch: adjusts command-line height (with save/restore of
///   min_set_ch), then clamps height to ROWS_AVAIL
/// - FR_LEAF (fr_win != NULL): clears `w_hsep_height` if bottom window, then
///   calls `rs_win_new_height`
/// - FR_ROW: propagates same height to all children, restarting if any child
///   ends up taller
/// - FR_COL: distributes extra lines starting from topmost or bottommost
///   non-fixed-height frame
///
/// # Safety
/// `topfrp` must be non-null and point to a valid Frame.
pub(crate) unsafe fn frame_new_height_impl(
    topfrp: *mut Frame,
    mut height: c_int,
    topfirst: bool,
    wfh: bool,
    set_ch: bool,
) {
    if (*topfrp).fr_parent.is_null() && set_ch {
        // topframe: update the command line height, with side effects.
        let p_ch = nvim_get_window_p_ch() as c_int;
        let min_set_ch = nvim_get_min_set_ch() as c_int;
        let new_ch = std::cmp::max(min_set_ch, p_ch + (*topfrp).fr_height - height);
        if new_ch != p_ch {
            // Inline nvim_set_cmdheight_option: save/restore min_set_ch around set_option_value
            let save_ch = nvim_get_min_set_ch();
            nvim_set_option_cmdheight(i64::from(new_ch));
            nvim_set_min_set_ch(save_ch);
        }
        height = std::cmp::min(nvim_get_rows_avail(), height);
    }

    if !(*topfrp).fr_win.is_null() {
        // Simple case: just one window.
        let wp = (*topfrp).fr_win;
        if rs_is_bottom_win(wp) != 0 {
            nvim_win_set_hsep_height(wp, 0);
        }
        rs_win_new_height(
            wp,
            height - nvim_win_get_hsep_height(wp) - nvim_win_get_status_height(wp),
        );
    } else if (*topfrp).fr_layout == FR_ROW {
        // All frames in this row get the same new height.
        loop {
            let mut frp = (*topfrp).fr_child;
            let mut restart = false;
            while !frp.is_null() {
                frame_new_height_impl(frp, height, topfirst, wfh, set_ch);
                if (*frp).fr_height > height {
                    // Could not fit the windows, make the whole row higher.
                    height = (*frp).fr_height;
                    restart = true;
                    break;
                }
                frp = (*frp).fr_next;
            }
            if !restart {
                break;
            }
        }
    } else {
        // FR_COL: resize a column of frames. Resize the bottom (or top) frame
        // first, frames above (or below) that when needed.
        let mut frp = (*topfrp).fr_child;

        if wfh {
            // Advance past frames with 'winfixheight' set.
            while rs_frame_fixed_height(frp) != 0 {
                frp = (*frp).fr_next;
                if frp.is_null() {
                    // No frame without wfh, give up.
                    (*topfrp).fr_height = height;
                    return;
                }
            }
        }

        if !topfirst {
            // Find the bottom frame of this column.
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            if wfh {
                // Advance back past frames with 'winfixheight' set.
                while rs_frame_fixed_height(frp) != 0 {
                    frp = (*frp).fr_prev;
                }
            }
        }

        let mut extra_lines = height - (*topfrp).fr_height;
        if extra_lines < 0 {
            // Reduce height of contained frames, bottom or top frame first.
            while !frp.is_null() {
                let h = rs_frame_minheight(frp, WinHandle::null());
                if (*frp).fr_height + extra_lines < h {
                    extra_lines += (*frp).fr_height - h;
                    frame_new_height_impl(frp, h, topfirst, wfh, set_ch);
                } else {
                    frame_new_height_impl(
                        frp,
                        (*frp).fr_height + extra_lines,
                        topfirst,
                        wfh,
                        set_ch,
                    );
                    break;
                }
                if topfirst {
                    loop {
                        frp = (*frp).fr_next;
                        if !(wfh && !frp.is_null() && rs_frame_fixed_height(frp) != 0) {
                            break;
                        }
                    }
                } else {
                    loop {
                        frp = (*frp).fr_prev;
                        if !(wfh && !frp.is_null() && rs_frame_fixed_height(frp) != 0) {
                            break;
                        }
                    }
                }
                // Increase "height" if we could not reduce enough frames.
                if frp.is_null() {
                    height -= extra_lines;
                }
            }
        } else if extra_lines > 0 {
            // Increase height of bottom or top frame.
            frame_new_height_impl(frp, (*frp).fr_height + extra_lines, topfirst, wfh, set_ch);
        }
    }

    (*topfrp).fr_height = height;
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Calculate grow distribution amount.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_grow_distribution(frp: *const Frame, delta: c_int) -> c_int {
    calculate_grow_distribution_impl(frp, delta)
}

/// FFI: Calculate sibling shrink amount.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_sibling_shrink(frp: *const Frame, total_shrink: c_int) -> c_int {
    calculate_sibling_shrink_impl(frp, total_shrink)
}

/// FFI: Check if frame height can change by delta.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_can_change_height(frp: *const Frame, delta: c_int) -> c_int {
    c_int::from(frame_can_change_height_impl(frp, delta))
}

/// FFI: Check if frame width can change by delta.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_can_change_width(frp: *const Frame, delta: c_int) -> c_int {
    c_int::from(frame_can_change_width_impl(frp, delta))
}

/// FFI: Get maximum height decrease for frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_max_height_decrease(frp: *const Frame) -> c_int {
    frame_max_height_decrease_impl(frp)
}

/// FFI: Get maximum width decrease for frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_max_width_decrease(frp: *const Frame) -> c_int {
    frame_max_width_decrease_impl(frp)
}

/// FFI: Find ancestor with layout (FR_COL=2, FR_ROW=1).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_find_ancestor_layout(
    frp: *const Frame,
    layout: c_int,
) -> *mut Frame {
    find_ancestor_with_layout_impl(frp, layout as i8)
}

/// FFI: Count frame children.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_count_children(frp: *const Frame) -> c_int {
    count_children_impl(frp)
}

/// FFI: Get child at index.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_child_at_index(frp: *const Frame, index: c_int) -> *mut Frame {
    get_child_at_index_impl(frp, index)
}

/// FFI: Get child index in parent.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_child_index(frp: *const Frame) -> c_int {
    get_child_index_impl(frp)
}

/// FFI: Check if frame is resizable.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_frame_is_resizable(frp: *const Frame) -> c_int {
    c_int::from(frame_is_resizable_impl(frp))
}

/// FFI: Get resize direction ('v' or 'h', 0 if none).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resize_get_direction(frp: *const Frame) -> c_int {
    get_resize_direction_impl(frp)
}

/// FFI: Get window's frame for resize operations.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_get_frame(wp: WinHandle) -> *mut Frame {
    if wp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { nvim_win_get_frame(wp) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_distribution() {
        assert_eq!(calculate_grow_distribution_impl(std::ptr::null(), 10), 0);
        assert_eq!(calculate_sibling_shrink_impl(std::ptr::null(), 10), 0);
    }

    #[test]
    fn test_null_constraints() {
        assert!(!frame_can_change_height_impl(std::ptr::null(), -10));
        assert!(!frame_can_change_width_impl(std::ptr::null(), -10));
        assert_eq!(frame_max_height_decrease_impl(std::ptr::null()), 0);
        assert_eq!(frame_max_width_decrease_impl(std::ptr::null()), 0);
    }

    #[test]
    fn test_null_traversal() {
        assert!(find_ancestor_with_layout_impl(std::ptr::null(), FR_COL).is_null());
        assert_eq!(count_children_impl(std::ptr::null()), 0);
        assert!(get_child_at_index_impl(std::ptr::null(), 0).is_null());
        assert_eq!(get_child_index_impl(std::ptr::null()), -1);
    }

    #[test]
    fn test_null_state() {
        assert!(!frame_is_resizable_impl(std::ptr::null()));
        assert_eq!(get_resize_direction_impl(std::ptr::null()), 0);
    }
}
