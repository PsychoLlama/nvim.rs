//! Frame handle types, layout enums, and split info structs.
//!
//! This module provides type definitions for frame operations, extending
//! the core types in lib.rs and layout.rs with frame-specific structures.

use std::ffi::c_int;
use std::ptr;

use crate::{Frame, WinHandle};

// =============================================================================
// Frame Layout Direction
// =============================================================================

/// Direction of a frame's layout (for FR_ROW or FR_COL).
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutDir {
    /// Frame is a leaf (contains a window)
    #[default]
    Leaf = 0,
    /// Children are arranged in a row (horizontal)
    Row = 1,
    /// Children are arranged in a column (vertical)
    Col = 2,
}

impl LayoutDir {
    /// Create from raw frame layout value.
    #[must_use]
    pub const fn from_raw(value: i8) -> Self {
        match value {
            1 => Self::Row,
            2 => Self::Col,
            _ => Self::Leaf,
        }
    }

    /// Convert to raw frame layout value.
    #[must_use]
    pub const fn to_raw(self) -> i8 {
        self as i8
    }

    /// Check if this is a leaf frame.
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf)
    }

    /// Check if this is a row layout.
    #[must_use]
    pub const fn is_row(&self) -> bool {
        matches!(self, Self::Row)
    }

    /// Check if this is a column layout.
    #[must_use]
    pub const fn is_col(&self) -> bool {
        matches!(self, Self::Col)
    }

    /// Check if children are arranged horizontally.
    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Row)
    }

    /// Check if children are arranged vertically.
    #[must_use]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Col)
    }

    /// Get the opposite layout direction.
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Leaf => Self::Leaf,
            Self::Row => Self::Col,
            Self::Col => Self::Row,
        }
    }

    /// Get from a frame pointer.
    ///
    /// # Safety
    /// The frame pointer must be null or valid.
    #[must_use]
    pub unsafe fn from_frame(frp: *const Frame) -> Self {
        if frp.is_null() {
            return Self::Leaf;
        }
        Self::from_raw((*frp).fr_layout)
    }
}

// =============================================================================
// Frame Dimensions
// =============================================================================

/// Current and pending dimensions of a frame.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameDimensions {
    /// Current width
    pub width: c_int,
    /// Current height
    pub height: c_int,
    /// New width (used during resize operations)
    pub new_width: c_int,
    /// New height (used during resize operations)
    pub new_height: c_int,
}

impl FrameDimensions {
    /// Create new dimensions.
    #[must_use]
    pub const fn new(width: c_int, height: c_int) -> Self {
        Self {
            width,
            height,
            new_width: width,
            new_height: height,
        }
    }

    /// Check if a resize is pending.
    #[must_use]
    pub const fn has_pending_resize(&self) -> bool {
        self.width != self.new_width || self.height != self.new_height
    }

    /// Apply pending resize (copy new to current).
    pub fn apply_pending(&mut self) {
        self.width = self.new_width;
        self.height = self.new_height;
    }

    /// Extract from a frame pointer.
    ///
    /// # Safety
    /// The frame pointer must be null or valid.
    #[must_use]
    pub unsafe fn from_frame(frp: *const Frame) -> Self {
        if frp.is_null() {
            return Self::default();
        }
        let frame = &*frp;
        Self {
            width: frame.fr_width,
            height: frame.fr_height,
            new_width: frame.fr_newwidth,
            new_height: frame.fr_newheight,
        }
    }
}

// =============================================================================
// Frame Position (in terminal coordinates)
// =============================================================================

/// Position of a frame in terminal screen coordinates.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FramePosition {
    /// Row position (0-based from top)
    pub row: c_int,
    /// Column position (0-based from left)
    pub col: c_int,
}

impl FramePosition {
    /// Create new position.
    #[must_use]
    pub const fn new(row: c_int, col: c_int) -> Self {
        Self { row, col }
    }

    /// Create position at origin.
    #[must_use]
    pub const fn origin() -> Self {
        Self { row: 0, col: 0 }
    }

    /// Offset by row.
    #[must_use]
    pub const fn offset_row(self, delta: c_int) -> Self {
        Self {
            row: self.row + delta,
            col: self.col,
        }
    }

    /// Offset by column.
    #[must_use]
    pub const fn offset_col(self, delta: c_int) -> Self {
        Self {
            row: self.row,
            col: self.col + delta,
        }
    }
}

// =============================================================================
// Split Info
// =============================================================================

/// Information about a pending or completed split operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SplitInfo {
    /// Requested size (0 = auto)
    pub size: c_int,
    /// Split flags (WSP_* constants)
    pub flags: c_int,
    /// Direction to insert new window (0=above/left, 1=below/right)
    pub direction: c_int,
    /// Window to split (NULL = current window)
    pub target_win: WinHandle,
    /// Frame to flatten after split (NULL = none)
    pub flatten_frame: *mut Frame,
}

impl Default for SplitInfo {
    fn default() -> Self {
        Self {
            size: 0,
            flags: 0,
            direction: 0,
            target_win: WinHandle::null(),
            flatten_frame: ptr::null_mut(),
        }
    }
}

impl SplitInfo {
    /// Create new split info with size and flags.
    #[must_use]
    pub const fn new(size: c_int, flags: c_int) -> Self {
        Self {
            size,
            flags,
            direction: 0,
            target_win: WinHandle::null(),
            flatten_frame: ptr::null_mut(),
        }
    }

    /// Check if this is a vertical split.
    #[must_use]
    pub const fn is_vertical(&self) -> bool {
        (self.flags & super::constants::WSP_VERT) != 0
    }

    /// Check if this is a horizontal split.
    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        !self.is_vertical()
    }

    /// Check if room check is required.
    #[must_use]
    pub const fn needs_room_check(&self) -> bool {
        (self.flags & super::constants::WSP_ROOM) != 0
    }

    /// Check if this creates a help window.
    #[must_use]
    pub const fn is_help_split(&self) -> bool {
        (self.flags & super::constants::WSP_HELP) != 0
    }

    /// Check if new window should not be entered.
    #[must_use]
    pub const fn no_enter(&self) -> bool {
        (self.flags & super::constants::WSP_NOENTER) != 0
    }
}

// =============================================================================
// Frame Iterator State
// =============================================================================

/// State for iterating over frames in a tree.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FrameIterState {
    /// Current frame
    pub current: *mut Frame,
    /// Root frame (where iteration started)
    pub root: *mut Frame,
    /// Current depth in tree
    pub depth: c_int,
}

impl Default for FrameIterState {
    fn default() -> Self {
        Self {
            current: ptr::null_mut(),
            root: ptr::null_mut(),
            depth: 0,
        }
    }
}

impl FrameIterState {
    /// Create new iterator starting at frame.
    #[must_use]
    pub const fn new(root: *mut Frame) -> Self {
        Self {
            current: root,
            root,
            depth: 0,
        }
    }

    /// Check if iteration is complete.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.current.is_null()
    }

    /// Check if at root.
    #[must_use]
    pub fn at_root(&self) -> bool {
        std::ptr::eq(self.current, self.root)
    }
}

// =============================================================================
// Frame Handle Wrapper
// =============================================================================

/// A safe wrapper around a frame pointer with layout info.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FrameRef {
    /// Pointer to the frame
    pub ptr: *mut Frame,
    /// Cached layout direction
    pub layout: LayoutDir,
}

impl Default for FrameRef {
    fn default() -> Self {
        Self {
            ptr: ptr::null_mut(),
            layout: LayoutDir::Leaf,
        }
    }
}

impl FrameRef {
    /// Create from frame pointer.
    ///
    /// # Safety
    /// The frame pointer must be null or valid.
    #[must_use]
    pub unsafe fn from_ptr(ptr: *mut Frame) -> Self {
        let layout = LayoutDir::from_frame(ptr);
        Self { ptr, layout }
    }

    /// Check if this is null.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Check if this is a leaf.
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        self.layout.is_leaf()
    }

    /// Check if this is a row.
    #[must_use]
    pub const fn is_row(&self) -> bool {
        self.layout.is_row()
    }

    /// Check if this is a column.
    #[must_use]
    pub const fn is_col(&self) -> bool {
        self.layout.is_col()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

// Helper to safely convert c_int to i8 for layout values (0-2)
#[inline]
fn layout_value_to_i8(value: c_int) -> i8 {
    // Layout values are 0, 1, or 2 - safe truncation
    (value & 0x7F) as i8
}

/// FFI export: Create LayoutDir from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_dir_from_raw(value: c_int) -> c_int {
    c_int::from(LayoutDir::from_raw(layout_value_to_i8(value)).to_raw())
}

/// FFI export: Check if layout is leaf.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_dir_is_leaf(value: c_int) -> c_int {
    c_int::from(LayoutDir::from_raw(layout_value_to_i8(value)).is_leaf())
}

/// FFI export: Check if layout is row.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_dir_is_row(value: c_int) -> c_int {
    c_int::from(LayoutDir::from_raw(layout_value_to_i8(value)).is_row())
}

/// FFI export: Check if layout is column.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_dir_is_col(value: c_int) -> c_int {
    c_int::from(LayoutDir::from_raw(layout_value_to_i8(value)).is_col())
}

/// FFI export: Get opposite layout direction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_dir_opposite(value: c_int) -> c_int {
    c_int::from(
        LayoutDir::from_raw(layout_value_to_i8(value))
            .opposite()
            .to_raw(),
    )
}

/// FFI export: Get layout from frame.
///
/// # Safety
/// Caller must ensure frame pointer is null or valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_layout_dir(frp: *const Frame) -> c_int {
    c_int::from(LayoutDir::from_frame(frp).to_raw())
}

/// FFI export: Create frame dimensions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_dims_new(width: c_int, height: c_int) -> FrameDimensions {
    FrameDimensions::new(width, height)
}

/// FFI export: Check if frame has pending resize.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_dims_has_pending(dims: FrameDimensions) -> c_int {
    c_int::from(dims.has_pending_resize())
}

/// FFI export: Get frame dimensions from frame.
///
/// # Safety
/// Caller must ensure frame pointer is null or valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_frame_dims_from_frame(frp: *const Frame) -> FrameDimensions {
    FrameDimensions::from_frame(frp)
}

/// FFI export: Create frame position.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_pos_new(row: c_int, col: c_int) -> FramePosition {
    FramePosition::new(row, col)
}

/// FFI export: Offset frame position by row.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_pos_offset_row(pos: FramePosition, delta: c_int) -> FramePosition {
    pos.offset_row(delta)
}

/// FFI export: Offset frame position by column.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_pos_offset_col(pos: FramePosition, delta: c_int) -> FramePosition {
    pos.offset_col(delta)
}

/// FFI export: Create split info.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_info_new(size: c_int, flags: c_int) -> SplitInfo {
    SplitInfo::new(size, flags)
}

/// FFI export: Check if split is vertical.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_info_is_vertical(info: SplitInfo) -> c_int {
    c_int::from(info.is_vertical())
}

/// FFI export: Check if split needs room check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_info_needs_room(info: SplitInfo) -> c_int {
    c_int::from(info.needs_room_check())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_dir() {
        // 0 = FR_LEAF, 1 = FR_ROW, 2 = FR_COL
        assert_eq!(LayoutDir::from_raw(0), LayoutDir::Leaf);
        assert_eq!(LayoutDir::from_raw(1), LayoutDir::Row);
        assert_eq!(LayoutDir::from_raw(2), LayoutDir::Col);

        assert!(LayoutDir::Leaf.is_leaf());
        assert!(LayoutDir::Row.is_row());
        assert!(LayoutDir::Col.is_col());

        assert_eq!(LayoutDir::Row.opposite(), LayoutDir::Col);
        assert_eq!(LayoutDir::Col.opposite(), LayoutDir::Row);
        assert_eq!(LayoutDir::Leaf.opposite(), LayoutDir::Leaf);
    }

    #[test]
    fn test_frame_dimensions() {
        let dims = FrameDimensions::new(80, 24);
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
        assert!(!dims.has_pending_resize());

        let mut dims2 = FrameDimensions {
            width: 80,
            height: 24,
            new_width: 100,
            new_height: 30,
        };
        assert!(dims2.has_pending_resize());
        dims2.apply_pending();
        assert_eq!(dims2.width, 100);
        assert_eq!(dims2.height, 30);
        assert!(!dims2.has_pending_resize());
    }

    #[test]
    fn test_frame_position() {
        let pos = FramePosition::origin();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);

        let pos2 = pos.offset_row(5).offset_col(10);
        assert_eq!(pos2.row, 5);
        assert_eq!(pos2.col, 10);
    }

    #[test]
    fn test_split_info() {
        use super::super::constants::{WSP_HELP, WSP_NOENTER, WSP_ROOM, WSP_VERT};

        let info = SplitInfo::new(0, WSP_VERT | WSP_ROOM);
        assert!(info.is_vertical());
        assert!(info.needs_room_check());
        assert!(!info.is_horizontal());

        let info2 = SplitInfo::new(10, WSP_HELP | WSP_NOENTER);
        assert!(!info2.is_vertical());
        assert!(info2.is_horizontal());
        assert!(info2.is_help_split());
        assert!(info2.no_enter());
    }

    #[test]
    fn test_frame_iter_state() {
        let state = FrameIterState::default();
        assert!(state.is_done());

        // Non-null root (simulated)
        let fake_frame = 0x1000 as *mut Frame;
        let state2 = FrameIterState::new(fake_frame);
        assert!(!state2.is_done());
        assert!(state2.at_root());
    }
}
