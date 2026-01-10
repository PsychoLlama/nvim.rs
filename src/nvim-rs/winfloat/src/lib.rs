//! Floating window types and utilities for Neovim
//!
//! This module provides types and utilities for floating windows,
//! including window configuration, relative positioning, and anchor types.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::redundant_closure_for_method_calls)]

use std::ffi::c_int;

// =============================================================================
// Float Relative Position
// =============================================================================

/// Floating window relative position types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatRelative {
    /// Relative to editor
    Editor = 0,
    /// Relative to window
    Window = 1,
    /// Relative to cursor
    Cursor = 2,
    /// Relative to mouse
    Mouse = 3,
    /// Relative to tabline
    Tabline = 4,
    /// Relative to laststatus
    Laststatus = 5,
}

impl FloatRelative {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Editor),
            1 => Some(Self::Window),
            2 => Some(Self::Cursor),
            3 => Some(Self::Mouse),
            4 => Some(Self::Tabline),
            5 => Some(Self::Laststatus),
            _ => None,
        }
    }

    /// Check if position requires a window reference
    pub fn needs_window(self) -> bool {
        self == Self::Window
    }

    /// Check if position is based on cursor
    pub fn is_cursor_based(self) -> bool {
        matches!(self, Self::Cursor | Self::Mouse)
    }
}

/// Get kFloatRelativeEditor value
#[no_mangle]
pub extern "C" fn rs_float_relative_editor() -> c_int {
    FloatRelative::Editor as c_int
}

/// Get kFloatRelativeWindow value
#[no_mangle]
pub extern "C" fn rs_float_relative_window() -> c_int {
    FloatRelative::Window as c_int
}

/// Get kFloatRelativeCursor value
#[no_mangle]
pub extern "C" fn rs_float_relative_cursor() -> c_int {
    FloatRelative::Cursor as c_int
}

/// Get kFloatRelativeMouse value
#[no_mangle]
pub extern "C" fn rs_float_relative_mouse() -> c_int {
    FloatRelative::Mouse as c_int
}

/// Get kFloatRelativeTabline value
#[no_mangle]
pub extern "C" fn rs_float_relative_tabline() -> c_int {
    FloatRelative::Tabline as c_int
}

/// Get kFloatRelativeLaststatus value
#[no_mangle]
pub extern "C" fn rs_float_relative_laststatus() -> c_int {
    FloatRelative::Laststatus as c_int
}

/// Check if relative type needs a window reference
#[no_mangle]
pub extern "C" fn rs_float_relative_needs_window(rel: c_int) -> bool {
    FloatRelative::from_int(rel).is_some_and(|r| r.needs_window())
}

/// Check if relative type is cursor-based
#[no_mangle]
pub extern "C" fn rs_float_relative_is_cursor_based(rel: c_int) -> bool {
    FloatRelative::from_int(rel).is_some_and(|r| r.is_cursor_based())
}

// =============================================================================
// Window Split Direction
// =============================================================================

/// Window split direction
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinSplit {
    /// Split to left
    Left = 0,
    /// Split to right
    Right = 1,
    /// Split above
    Above = 2,
    /// Split below
    Below = 3,
}

impl WinSplit {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Left),
            1 => Some(Self::Right),
            2 => Some(Self::Above),
            3 => Some(Self::Below),
            _ => None,
        }
    }

    /// Check if split is horizontal
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Check if split is vertical
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Above | Self::Below)
    }
}

/// Get kWinSplitLeft value
#[no_mangle]
pub extern "C" fn rs_win_split_left() -> c_int {
    WinSplit::Left as c_int
}

/// Get kWinSplitRight value
#[no_mangle]
pub extern "C" fn rs_win_split_right() -> c_int {
    WinSplit::Right as c_int
}

/// Get kWinSplitAbove value
#[no_mangle]
pub extern "C" fn rs_win_split_above() -> c_int {
    WinSplit::Above as c_int
}

/// Get kWinSplitBelow value
#[no_mangle]
pub extern "C" fn rs_win_split_below() -> c_int {
    WinSplit::Below as c_int
}

/// Check if split is horizontal
#[no_mangle]
pub extern "C" fn rs_win_split_is_horizontal(split: c_int) -> bool {
    WinSplit::from_int(split).is_some_and(|s| s.is_horizontal())
}

/// Check if split is vertical
#[no_mangle]
pub extern "C" fn rs_win_split_is_vertical(split: c_int) -> bool {
    WinSplit::from_int(split).is_some_and(|s| s.is_vertical())
}

// =============================================================================
// Window Style
// =============================================================================

/// Window style
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinStyle {
    /// No special style
    Unused = 0,
    /// Minimal UI: no number column, eob markers, etc
    Minimal = 1,
}

impl WinStyle {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Unused),
            1 => Some(Self::Minimal),
            _ => None,
        }
    }
}

/// Get kWinStyleUnused value
#[no_mangle]
pub extern "C" fn rs_win_style_unused() -> c_int {
    WinStyle::Unused as c_int
}

/// Get kWinStyleMinimal value
#[no_mangle]
pub extern "C" fn rs_win_style_minimal() -> c_int {
    WinStyle::Minimal as c_int
}

/// Check if style is minimal
#[no_mangle]
pub extern "C" fn rs_win_style_is_minimal(style: c_int) -> bool {
    WinStyle::from_int(style) == Some(WinStyle::Minimal)
}

// =============================================================================
// Text Alignment
// =============================================================================

/// Text alignment position
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignTextPos {
    /// Align left
    Left = 0,
    /// Align center
    Center = 1,
    /// Align right
    Right = 2,
}

impl AlignTextPos {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Left),
            1 => Some(Self::Center),
            2 => Some(Self::Right),
            _ => None,
        }
    }
}

/// Get kAlignLeft value
#[no_mangle]
pub extern "C" fn rs_align_left() -> c_int {
    AlignTextPos::Left as c_int
}

/// Get kAlignCenter value
#[no_mangle]
pub extern "C" fn rs_align_center() -> c_int {
    AlignTextPos::Center as c_int
}

/// Get kAlignRight value
#[no_mangle]
pub extern "C" fn rs_align_right() -> c_int {
    AlignTextPos::Right as c_int
}

// =============================================================================
// Border Text Type
// =============================================================================

/// Border text type (title or footer)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderTextType {
    /// Window title
    Title = 0,
    /// Window footer
    Footer = 1,
}

/// Get kBorderTextTitle value
#[no_mangle]
pub extern "C" fn rs_border_text_title() -> c_int {
    BorderTextType::Title as c_int
}

/// Get kBorderTextFooter value
#[no_mangle]
pub extern "C" fn rs_border_text_footer() -> c_int {
    BorderTextType::Footer as c_int
}

// =============================================================================
// Float Anchor
// =============================================================================

bitflags::bitflags! {
    /// Floating window anchor position
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FloatAnchor: u8 {
        /// Anchor at south (bottom)
        const SOUTH = 0b01;
        /// Anchor at east (right)
        const EAST = 0b10;
    }
}

/// Check if anchor is at NW (north-west)
#[no_mangle]
pub extern "C" fn rs_float_anchor_is_nw(anchor: u8) -> bool {
    FloatAnchor::from_bits_truncate(anchor).is_empty()
}

/// Check if anchor is at NE (north-east)
#[no_mangle]
pub extern "C" fn rs_float_anchor_is_ne(anchor: u8) -> bool {
    FloatAnchor::from_bits_truncate(anchor) == FloatAnchor::EAST
}

/// Check if anchor is at SW (south-west)
#[no_mangle]
pub extern "C" fn rs_float_anchor_is_sw(anchor: u8) -> bool {
    FloatAnchor::from_bits_truncate(anchor) == FloatAnchor::SOUTH
}

/// Check if anchor is at SE (south-east)
#[no_mangle]
pub extern "C" fn rs_float_anchor_is_se(anchor: u8) -> bool {
    FloatAnchor::from_bits_truncate(anchor) == (FloatAnchor::SOUTH | FloatAnchor::EAST)
}

/// Check if anchor has south component
#[no_mangle]
pub extern "C" fn rs_float_anchor_has_south(anchor: u8) -> bool {
    FloatAnchor::from_bits_truncate(anchor).contains(FloatAnchor::SOUTH)
}

/// Check if anchor has east component
#[no_mangle]
pub extern "C" fn rs_float_anchor_has_east(anchor: u8) -> bool {
    FloatAnchor::from_bits_truncate(anchor).contains(FloatAnchor::EAST)
}

// =============================================================================
// Z-Index Constants
// =============================================================================

/// Default z-index for floating windows
pub const ZINDEX_FLOAT_DEFAULT: c_int = 50;

/// Get default floating window z-index
#[no_mangle]
pub extern "C" fn rs_zindex_float_default() -> c_int {
    ZINDEX_FLOAT_DEFAULT
}

/// Check if z-index is valid (positive)
#[no_mangle]
pub extern "C" fn rs_zindex_is_valid(zindex: c_int) -> bool {
    zindex > 0
}

// =============================================================================
// Border Character Indices
// =============================================================================

/// Border character positions (8 characters total)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderCharIndex {
    /// Top-left corner
    TopLeft = 0,
    /// Top edge
    Top = 1,
    /// Top-right corner
    TopRight = 2,
    /// Right edge
    Right = 3,
    /// Bottom-right corner
    BottomRight = 4,
    /// Bottom edge
    Bottom = 5,
    /// Bottom-left corner
    BottomLeft = 6,
    /// Left edge
    Left = 7,
}

/// Number of border characters
pub const BORDER_CHAR_COUNT: usize = 8;

/// Get number of border characters
#[no_mangle]
pub extern "C" fn rs_border_char_count() -> c_int {
    BORDER_CHAR_COUNT as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_relative() {
        assert_eq!(rs_float_relative_editor(), 0);
        assert_eq!(rs_float_relative_window(), 1);
        assert_eq!(rs_float_relative_cursor(), 2);
        assert_eq!(rs_float_relative_mouse(), 3);
        assert_eq!(rs_float_relative_tabline(), 4);
        assert_eq!(rs_float_relative_laststatus(), 5);

        assert!(!rs_float_relative_needs_window(0));
        assert!(rs_float_relative_needs_window(1));
        assert!(!rs_float_relative_needs_window(2));

        assert!(!rs_float_relative_is_cursor_based(0));
        assert!(rs_float_relative_is_cursor_based(2));
        assert!(rs_float_relative_is_cursor_based(3));
    }

    #[test]
    fn test_win_split() {
        assert_eq!(rs_win_split_left(), 0);
        assert_eq!(rs_win_split_right(), 1);
        assert_eq!(rs_win_split_above(), 2);
        assert_eq!(rs_win_split_below(), 3);

        assert!(rs_win_split_is_horizontal(0));
        assert!(rs_win_split_is_horizontal(1));
        assert!(!rs_win_split_is_horizontal(2));

        assert!(!rs_win_split_is_vertical(0));
        assert!(rs_win_split_is_vertical(2));
        assert!(rs_win_split_is_vertical(3));
    }

    #[test]
    fn test_win_style() {
        assert_eq!(rs_win_style_unused(), 0);
        assert_eq!(rs_win_style_minimal(), 1);

        assert!(!rs_win_style_is_minimal(0));
        assert!(rs_win_style_is_minimal(1));
    }

    #[test]
    fn test_align() {
        assert_eq!(rs_align_left(), 0);
        assert_eq!(rs_align_center(), 1);
        assert_eq!(rs_align_right(), 2);
    }

    #[test]
    fn test_border_text_type() {
        assert_eq!(rs_border_text_title(), 0);
        assert_eq!(rs_border_text_footer(), 1);
    }

    #[test]
    fn test_float_anchor() {
        assert!(rs_float_anchor_is_nw(0));
        assert!(!rs_float_anchor_is_nw(1));

        assert!(rs_float_anchor_is_ne(2));
        assert!(!rs_float_anchor_is_ne(0));

        assert!(rs_float_anchor_is_sw(1));
        assert!(!rs_float_anchor_is_sw(0));

        assert!(rs_float_anchor_is_se(3));
        assert!(!rs_float_anchor_is_se(0));

        assert!(rs_float_anchor_has_south(1));
        assert!(rs_float_anchor_has_south(3));
        assert!(!rs_float_anchor_has_south(0));
        assert!(!rs_float_anchor_has_south(2));

        assert!(rs_float_anchor_has_east(2));
        assert!(rs_float_anchor_has_east(3));
        assert!(!rs_float_anchor_has_east(0));
        assert!(!rs_float_anchor_has_east(1));
    }

    #[test]
    fn test_zindex() {
        assert_eq!(rs_zindex_float_default(), 50);
        assert!(rs_zindex_is_valid(50));
        assert!(rs_zindex_is_valid(1));
        assert!(!rs_zindex_is_valid(0));
        assert!(!rs_zindex_is_valid(-1));
    }

    #[test]
    fn test_border_char_count() {
        assert_eq!(rs_border_char_count(), 8);
    }
}
