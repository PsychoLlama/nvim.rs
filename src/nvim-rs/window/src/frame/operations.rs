//! Frame tree manipulation operations.
//!
//! This module provides additional FFI functions for frame tree operations,
//! complementing the core operations in lib.rs (frame_append, frame_insert,
//! frame_remove, frame_comp_pos, frame_setheight, frame_setwidth).
//!
//! These functions handle:
//! - Frame initialization and clearing
//! - Frame tree traversal
//! - Frame validation and consistency checks
//! - Frame size calculations

use std::ffi::c_int;

use crate::{Frame, WinHandle};

use super::constants::{FR_COL, FR_LEAF, FR_ROW};

// =============================================================================
// Frame Initialization
// =============================================================================

/// Initialize a frame to default values.
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_init(frp: *mut Frame) {
    if frp.is_null() {
        return;
    }
    (*frp).fr_layout = FR_LEAF;
    (*frp).fr_width = 0;
    (*frp).fr_height = 0;
    (*frp).fr_newwidth = 0;
    (*frp).fr_newheight = 0;
    (*frp).fr_parent = std::ptr::null_mut();
    (*frp).fr_next = std::ptr::null_mut();
    (*frp).fr_prev = std::ptr::null_mut();
    (*frp).fr_child = std::ptr::null_mut();
    (*frp).fr_win = WinHandle::null();
}

/// Clear frame links (but keep layout and size).
///
/// # Safety
/// Caller must ensure `frp` is a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_clear_links(frp: *mut Frame) {
    if frp.is_null() {
        return;
    }
    (*frp).fr_parent = std::ptr::null_mut();
    (*frp).fr_next = std::ptr::null_mut();
    (*frp).fr_prev = std::ptr::null_mut();
    (*frp).fr_child = std::ptr::null_mut();
    (*frp).fr_win = WinHandle::null();
}

/// Copy frame size from source to dest.
///
/// # Safety
/// Caller must ensure both pointers are null or valid Frame pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_copy_size(dest: *mut Frame, src: *const Frame) {
    if dest.is_null() || src.is_null() {
        return;
    }
    (*dest).fr_width = (*src).fr_width;
    (*dest).fr_height = (*src).fr_height;
    (*dest).fr_newwidth = (*src).fr_newwidth;
    (*dest).fr_newheight = (*src).fr_newheight;
}

// =============================================================================
// Frame Traversal
// =============================================================================

/// Get the first leaf frame in a subtree (depth-first).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_first_leaf(frp: *mut Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    let mut current = frp;
    while (*current).fr_layout != FR_LEAF && !(*current).fr_child.is_null() {
        current = (*current).fr_child;
    }
    current
}

/// Get the last leaf frame in a subtree (depth-first).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_last_leaf(frp: *mut Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    let mut current = frp;
    while (*current).fr_layout != FR_LEAF {
        // Go to last child
        let mut child = (*current).fr_child;
        if child.is_null() {
            break;
        }
        while !(*child).fr_next.is_null() {
            child = (*child).fr_next;
        }
        current = child;
    }
    current
}

/// Get the next leaf frame in document order.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_next_leaf(frp: *mut Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    // Try next sibling first
    if !(*frp).fr_next.is_null() {
        return rs_frame_first_leaf((*frp).fr_next);
    }

    // Go up to parent and try parent's next
    let mut current = (*frp).fr_parent;
    while !current.is_null() {
        if !(*current).fr_next.is_null() {
            return rs_frame_first_leaf((*current).fr_next);
        }
        current = (*current).fr_parent;
    }

    std::ptr::null_mut()
}

/// Get the previous leaf frame in document order.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_prev_leaf(frp: *mut Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }

    // Try prev sibling first
    if !(*frp).fr_prev.is_null() {
        return rs_frame_last_leaf((*frp).fr_prev);
    }

    // Go up to parent and try parent's prev
    let mut current = (*frp).fr_parent;
    while !current.is_null() {
        if !(*current).fr_prev.is_null() {
            return rs_frame_last_leaf((*current).fr_prev);
        }
        current = (*current).fr_parent;
    }

    std::ptr::null_mut()
}

/// Count all leaf frames in a subtree.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_count_leaves(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }

    if (*frp).fr_layout == FR_LEAF {
        return 1;
    }

    let mut count = 0;
    let mut child = (*frp).fr_child;
    while !child.is_null() {
        count += rs_frame_count_leaves(child);
        child = (*child).fr_next;
    }
    count
}

// =============================================================================
// Frame Validation
// =============================================================================

/// Check if frame tree structure is valid.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_is_valid(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 1; // Null is valid (no frame)
    }

    let layout = (*frp).fr_layout;

    // Check layout is a valid value
    if layout != FR_LEAF && layout != FR_ROW && layout != FR_COL {
        return 0;
    }

    // Leaf frames must not have children
    if layout == FR_LEAF && !(*frp).fr_child.is_null() {
        return 0;
    }

    // Non-leaf frames must not have a window
    if layout != FR_LEAF && !(*frp).fr_win.is_null() {
        return 0;
    }

    // Recursively check children
    let mut child = (*frp).fr_child;
    while !child.is_null() {
        // Child's parent should point back to this frame
        if !std::ptr::eq((*child).fr_parent, frp) {
            return 0;
        }
        // Check child is valid
        if rs_frame_is_valid(child) == 0 {
            return 0;
        }
        child = (*child).fr_next;
    }

    1
}

/// Check if frame contains a specific window.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_contains_win(frp: *const Frame, wp: WinHandle) -> c_int {
    if frp.is_null() || wp.is_null() {
        return 0;
    }

    if (*frp).fr_layout == FR_LEAF {
        return c_int::from((*frp).fr_win == wp);
    }

    let mut child = (*frp).fr_child;
    while !child.is_null() {
        if rs_frame_contains_win(child, wp) != 0 {
            return 1;
        }
        child = (*child).fr_next;
    }
    0
}

// =============================================================================
// Frame Size Calculations
// =============================================================================

/// Get total width of all children (for row layout).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_children_width(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    let mut total = 0;
    let mut child = (*frp).fr_child;
    while !child.is_null() {
        total += (*child).fr_width;
        child = (*child).fr_next;
    }
    total
}

/// Get total height of all children (for column layout).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_children_height(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    let mut total = 0;
    let mut child = (*frp).fr_child;
    while !child.is_null() {
        total += (*child).fr_height;
        child = (*child).fr_next;
    }
    total
}

/// Get maximum width among children.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_max_child_width(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    let mut max = 0;
    let mut child = (*frp).fr_child;
    while !child.is_null() {
        if (*child).fr_width > max {
            max = (*child).fr_width;
        }
        child = (*child).fr_next;
    }
    max
}

/// Get maximum height among children.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_max_child_height(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    let mut max = 0;
    let mut child = (*frp).fr_child;
    while !child.is_null() {
        if (*child).fr_height > max {
            max = (*child).fr_height;
        }
        child = (*child).fr_next;
    }
    max
}

/// Propagate frame size to all children (set all children to same size).
/// Used for row layout to set equal heights, or column layout to set equal widths.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_propagate_size(frp: *mut Frame) {
    if frp.is_null() {
        return;
    }

    let layout = (*frp).fr_layout;
    let width = (*frp).fr_width;
    let height = (*frp).fr_height;

    let mut child = (*frp).fr_child;
    while !child.is_null() {
        if layout == FR_ROW {
            // In row layout, children have same height but different widths
            (*child).fr_height = height;
        } else if layout == FR_COL {
            // In column layout, children have same width but different heights
            (*child).fr_width = width;
        }
        child = (*child).fr_next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_constants() {
        assert_eq!(FR_LEAF, 0);
        assert_eq!(FR_ROW, 1);
        assert_eq!(FR_COL, 2);
    }

    #[test]
    fn test_frame_null_safety() {
        unsafe {
            // All functions should handle null gracefully
            rs_frame_init(std::ptr::null_mut());
            rs_frame_clear_links(std::ptr::null_mut());
            rs_frame_copy_size(std::ptr::null_mut(), std::ptr::null());

            assert!(rs_frame_first_leaf(std::ptr::null_mut()).is_null());
            assert!(rs_frame_last_leaf(std::ptr::null_mut()).is_null());
            assert!(rs_frame_next_leaf(std::ptr::null_mut()).is_null());
            assert!(rs_frame_prev_leaf(std::ptr::null_mut()).is_null());

            assert_eq!(rs_frame_count_leaves(std::ptr::null()), 0);
            assert_eq!(rs_frame_is_valid(std::ptr::null()), 1);
            assert_eq!(rs_frame_contains_win(std::ptr::null(), WinHandle::null()), 0);

            assert_eq!(rs_frame_children_width(std::ptr::null()), 0);
            assert_eq!(rs_frame_children_height(std::ptr::null()), 0);
            assert_eq!(rs_frame_max_child_width(std::ptr::null()), 0);
            assert_eq!(rs_frame_max_child_height(std::ptr::null()), 0);

            rs_frame_propagate_size(std::ptr::null_mut());
        }
    }
}
