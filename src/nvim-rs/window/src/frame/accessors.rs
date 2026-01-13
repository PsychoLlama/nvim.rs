//! Frame field accessors (getters and setters).
//!
//! This module provides safe FFI functions for accessing and modifying
//! frame structure fields. These complement the basic getters in mod.rs
//! with setters and additional utility accessors.

use std::ffi::c_int;

use crate::{Frame, WinHandle};

use super::constants::{FR_COL, FR_LEAF, FR_ROW};

// =============================================================================
// Frame Layout Setters
// =============================================================================

/// Set the frame layout type.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_layout(frp: *mut Frame, layout: c_int) {
    if !frp.is_null() {
        // Layout values are 0, 1, or 2 - safe truncation
        (*frp).fr_layout = (layout & 0x7F) as i8;
    }
}

/// Set frame to leaf layout.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_layout_leaf(frp: *mut Frame) {
    if !frp.is_null() {
        (*frp).fr_layout = FR_LEAF;
    }
}

/// Set frame to row layout.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_layout_row(frp: *mut Frame) {
    if !frp.is_null() {
        (*frp).fr_layout = FR_ROW;
    }
}

/// Set frame to column layout.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_layout_col(frp: *mut Frame) {
    if !frp.is_null() {
        (*frp).fr_layout = FR_COL;
    }
}

// =============================================================================
// Frame Dimension Setters
// =============================================================================

/// Set the frame width.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_width(frp: *mut Frame, width: c_int) {
    if !frp.is_null() {
        (*frp).fr_width = width;
    }
}

/// Set the frame height.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_height(frp: *mut Frame, height: c_int) {
    if !frp.is_null() {
        (*frp).fr_height = height;
    }
}

/// Set the frame newwidth (pending width for resize).
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_newwidth(frp: *mut Frame, newwidth: c_int) {
    if !frp.is_null() {
        (*frp).fr_newwidth = newwidth;
    }
}

/// Set the frame newheight (pending height for resize).
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_newheight(frp: *mut Frame, newheight: c_int) {
    if !frp.is_null() {
        (*frp).fr_newheight = newheight;
    }
}

/// Set both width and height at once.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_size(frp: *mut Frame, width: c_int, height: c_int) {
    if !frp.is_null() {
        (*frp).fr_width = width;
        (*frp).fr_height = height;
    }
}

/// Set both newwidth and newheight at once.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_newsize(frp: *mut Frame, newwidth: c_int, newheight: c_int) {
    if !frp.is_null() {
        (*frp).fr_newwidth = newwidth;
        (*frp).fr_newheight = newheight;
    }
}

// =============================================================================
// Frame Link Setters
// =============================================================================

/// Set the frame parent.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_parent(frp: *mut Frame, parent: *mut Frame) {
    if !frp.is_null() {
        (*frp).fr_parent = parent;
    }
}

/// Set the frame next sibling.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_next(frp: *mut Frame, next: *mut Frame) {
    if !frp.is_null() {
        (*frp).fr_next = next;
    }
}

/// Set the frame prev sibling.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_prev(frp: *mut Frame, prev: *mut Frame) {
    if !frp.is_null() {
        (*frp).fr_prev = prev;
    }
}

/// Set the frame first child.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_child(frp: *mut Frame, child: *mut Frame) {
    if !frp.is_null() {
        (*frp).fr_child = child;
    }
}

/// Set the frame window.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_set_win(frp: *mut Frame, win: WinHandle) {
    if !frp.is_null() {
        (*frp).fr_win = win;
    }
}

// =============================================================================
// Compound Accessors
// =============================================================================

/// Get frame area (width * height).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_area(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    (*frp).fr_width * (*frp).fr_height
}

/// Check if frame has any siblings.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_has_siblings(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from(!(*frp).fr_prev.is_null() || !(*frp).fr_next.is_null())
}

/// Check if frame has a parent.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_has_parent(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from(!(*frp).fr_parent.is_null())
}

/// Check if frame has children.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_has_children(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    c_int::from(!(*frp).fr_child.is_null())
}

/// Get the last child of a frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_last_child(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    let mut child = (*frp).fr_child;
    if child.is_null() {
        return std::ptr::null_mut();
    }
    while !(*child).fr_next.is_null() {
        child = (*child).fr_next;
    }
    child
}

/// Get depth of frame in tree (0 = root).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_depth(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    let mut depth = 0;
    let mut parent = (*frp).fr_parent;
    while !parent.is_null() {
        depth += 1;
        parent = (*parent).fr_parent;
    }
    depth
}

/// Get the root frame (topmost ancestor).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_get_root(frp: *mut Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    let mut current = frp;
    while !(*current).fr_parent.is_null() {
        current = (*current).fr_parent;
    }
    current
}

/// Check if two frames are siblings.
///
/// # Safety
/// Caller must ensure both pointers are null or valid Frame pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_are_siblings(frp1: *const Frame, frp2: *const Frame) -> c_int {
    if frp1.is_null() || frp2.is_null() {
        return 0;
    }
    if (*frp1).fr_parent.is_null() || (*frp2).fr_parent.is_null() {
        return 0;
    }
    c_int::from(std::ptr::eq((*frp1).fr_parent, (*frp2).fr_parent))
}

/// Count siblings (including self).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_sibling_count(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    // Go to first sibling
    let mut first = frp.cast_mut();
    while !(*first).fr_prev.is_null() {
        first = (*first).fr_prev;
    }

    // Count all siblings
    let mut count = 0;
    let mut current = first;
    while !current.is_null() {
        count += 1;
        current = (*current).fr_next;
    }
    count
}

/// Get sibling index (0-based position among siblings).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_sibling_index(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return -1;
    }

    let mut index = 0;
    let mut prev = (*frp).fr_prev;
    while !prev.is_null() {
        index += 1;
        prev = (*prev).fr_prev;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constants() {
        assert_eq!(FR_LEAF, 0);
        assert_eq!(FR_ROW, 1);
        assert_eq!(FR_COL, 2);
    }
}
