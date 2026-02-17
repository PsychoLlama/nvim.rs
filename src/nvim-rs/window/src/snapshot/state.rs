//! Snapshot state queries and validation functions.
//!
//! This module provides helper functions for window layout snapshot operations,
//! supporting the C implementation of make_snapshot, restore_snapshot, etc.
//!
//! Note: The main snapshot functions (make_snapshot, restore_snapshot, clear_snapshot)
//! and direct snapshot access remain in C due to the need for tp_snapshot array access.
//! This module provides constants, frame comparison helpers, and validation functions.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_LEAF};

// =============================================================================
// Snapshot Index Constants
// =============================================================================

/// Help window snapshot index.
pub const SNAP_HELP_IDX: c_int = 0;

/// Autocommand window snapshot index.
pub const SNAP_AUCMD_IDX: c_int = 1;

/// Total number of snapshot slots.
pub const SNAP_COUNT: c_int = 2;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Check if window is valid.
    #[link_name = "rs_win_valid"]
    fn win_valid(wp: WinHandle) -> c_int;
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if snapshot index is valid.
#[inline]
fn is_valid_idx(idx: c_int) -> bool {
    (0..SNAP_COUNT).contains(&idx)
}

// =============================================================================
// Frame Comparison Helpers
// =============================================================================

/// Check if two frames have the same basic structure (layout and children presence).
fn frames_match_structure_impl(fr1: *const Frame, fr2: *const Frame) -> bool {
    if fr1.is_null() && fr2.is_null() {
        return true;
    }
    if fr1.is_null() || fr2.is_null() {
        return false;
    }
    unsafe {
        (*fr1).fr_layout == (*fr2).fr_layout
            && ((*fr1).fr_next.is_null() == (*fr2).fr_next.is_null())
            && ((*fr1).fr_child.is_null() == (*fr2).fr_child.is_null())
    }
}

/// Check if two frames have the same dimensions.
fn frames_match_dimensions_impl(fr1: *const Frame, fr2: *const Frame) -> bool {
    if fr1.is_null() || fr2.is_null() {
        return false;
    }
    unsafe { (*fr1).fr_width == (*fr2).fr_width && (*fr1).fr_height == (*fr2).fr_height }
}

/// Check if a snapshot frame's window pointer is valid.
fn snapshot_frame_win_valid_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return true;
    }
    unsafe {
        let fr = &*frp;
        // If it's not a leaf frame, no window to check
        if fr.fr_layout != FR_LEAF {
            return true;
        }
        // If leaf has no window, that's ok
        if fr.fr_win.is_null() {
            return true;
        }
        // Check if the window is valid
        win_valid(fr.fr_win) != 0
    }
}

/// Get the window stored in a snapshot frame (if any).
fn snapshot_frame_win_impl(frp: *const Frame) -> WinHandle {
    if frp.is_null() {
        return WinHandle::null();
    }
    unsafe {
        let fr = &*frp;
        if fr.fr_layout != FR_LEAF || fr.fr_win.is_null() {
            return WinHandle::null();
        }
        fr.fr_win
    }
}

/// Check if frame is a leaf with a window.
fn frame_is_leaf_with_win_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }
    unsafe {
        let fr = &*frp;
        fr.fr_layout == FR_LEAF && !fr.fr_win.is_null()
    }
}

/// Get frame width.
fn frame_width_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    unsafe { (*frp).fr_width }
}

/// Get frame height.
fn frame_height_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    unsafe { (*frp).fr_height }
}

/// Get frame layout type.
fn frame_layout_impl(frp: *const Frame) -> c_int {
    if frp.is_null() {
        return 0;
    }
    unsafe { c_int::from((*frp).fr_layout) }
}

/// Check if frame has a next sibling.
fn frame_has_next_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }
    unsafe { !(*frp).fr_next.is_null() }
}

/// Check if frame has a child.
fn frame_has_child_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }
    unsafe { !(*frp).fr_child.is_null() }
}

/// Get next frame in snapshot traversal.
fn frame_next_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { (*frp).fr_next }
}

/// Get child frame in snapshot traversal.
fn frame_child_impl(frp: *const Frame) -> *mut Frame {
    if frp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { (*frp).fr_child }
}

// =============================================================================
// Snapshot Index Validation
// =============================================================================

/// Validate a snapshot index.
fn valid_snapshot_idx_impl(idx: c_int) -> bool {
    is_valid_idx(idx)
}

/// Get the help snapshot index constant.
fn get_help_snapshot_idx_impl() -> c_int {
    SNAP_HELP_IDX
}

/// Get the total snapshot count.
fn get_snapshot_count_impl() -> c_int {
    SNAP_COUNT
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if two frames have matching structure.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_frames_match(fr1: *const Frame, fr2: *const Frame) -> c_int {
    c_int::from(frames_match_structure_impl(fr1, fr2))
}

/// FFI: Check if two frames have matching dimensions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_frames_match_dims(fr1: *const Frame, fr2: *const Frame) -> c_int {
    c_int::from(frames_match_dimensions_impl(fr1, fr2))
}

/// FFI: Check if snapshot frame's window is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_frame_win_valid(frp: *const Frame) -> c_int {
    c_int::from(snapshot_frame_win_valid_impl(frp))
}

/// FFI: Get window from snapshot frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_frame_win(frp: *const Frame) -> WinHandle {
    snapshot_frame_win_impl(frp)
}

/// FFI: Check if frame is a leaf with window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_is_leaf_with_win(frp: *const Frame) -> c_int {
    c_int::from(frame_is_leaf_with_win_impl(frp))
}

/// FFI: Get frame width for snapshot comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_frame_width(frp: *const Frame) -> c_int {
    frame_width_impl(frp)
}

/// FFI: Get frame height for snapshot comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_frame_height(frp: *const Frame) -> c_int {
    frame_height_impl(frp)
}

/// FFI: Get frame layout for snapshot comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_frame_layout(frp: *const Frame) -> c_int {
    frame_layout_impl(frp)
}

/// FFI: Check if frame has next sibling.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_has_next(frp: *const Frame) -> c_int {
    c_int::from(frame_has_next_impl(frp))
}

/// FFI: Check if frame has child.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_has_child(frp: *const Frame) -> c_int {
    c_int::from(frame_has_child_impl(frp))
}

/// FFI: Get next frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_get_next(frp: *const Frame) -> *mut Frame {
    frame_next_impl(frp)
}

/// FFI: Get child frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_get_child(frp: *const Frame) -> *mut Frame {
    frame_child_impl(frp)
}

/// FFI: Validate snapshot index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_valid_idx(idx: c_int) -> c_int {
    c_int::from(valid_snapshot_idx_impl(idx))
}

/// FFI: Get help snapshot index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_help_idx() -> c_int {
    get_help_snapshot_idx_impl()
}

/// FFI: Get snapshot count.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snapshot_count() -> c_int {
    get_snapshot_count_impl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_idx_validation() {
        assert!(valid_snapshot_idx_impl(SNAP_HELP_IDX));
        assert!(!valid_snapshot_idx_impl(-1));
        assert!(!valid_snapshot_idx_impl(SNAP_COUNT));
        assert!(!valid_snapshot_idx_impl(100));
    }

    #[test]
    fn test_snapshot_constants() {
        assert_eq!(get_help_snapshot_idx_impl(), 0);
        assert_eq!(get_snapshot_count_impl(), 1);
    }

    #[test]
    fn test_null_frame_match() {
        assert!(frames_match_structure_impl(
            std::ptr::null(),
            std::ptr::null()
        ));
        assert!(!frames_match_structure_impl(
            std::ptr::null(),
            std::ptr::dangling::<Frame>()
        ));
    }

    #[test]
    fn test_null_frame_win_valid() {
        assert!(snapshot_frame_win_valid_impl(std::ptr::null()));
    }

    #[test]
    fn test_null_frame_win() {
        assert!(snapshot_frame_win_impl(std::ptr::null()).is_null());
    }

    #[test]
    fn test_null_frame_accessors() {
        assert_eq!(frame_width_impl(std::ptr::null()), 0);
        assert_eq!(frame_height_impl(std::ptr::null()), 0);
        assert_eq!(frame_layout_impl(std::ptr::null()), 0);
        assert!(!frame_has_next_impl(std::ptr::null()));
        assert!(!frame_has_child_impl(std::ptr::null()));
        assert!(frame_next_impl(std::ptr::null()).is_null());
        assert!(frame_child_impl(std::ptr::null()).is_null());
    }
}
