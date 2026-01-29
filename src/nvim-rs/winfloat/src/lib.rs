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

// =============================================================================
// Phase D3: Floating Window Positioning Helpers
// =============================================================================

use std::ffi::c_void;

/// Opaque handle to window (`win_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// C accessor functions
extern "C" {
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_relative(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_window(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_zindex(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_focusable(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_hide(wp: WinHandle) -> c_int;
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_border_adj(wp: WinHandle, idx: c_int) -> c_int;
}

/// Check if window is a floating window.
#[no_mangle]
pub unsafe extern "C" fn rs_win_is_floating(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_floating(wp)
}

/// Get the border-adjusted width of a floating window.
///
/// Returns the window width plus left and right border widths.
#[no_mangle]
pub unsafe extern "C" fn rs_win_float_total_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let width = nvim_win_get_w_width(wp);
    let left_border = nvim_win_get_border_adj(wp, 3); // Left border
    let right_border = nvim_win_get_border_adj(wp, 1); // Right border
    width + left_border + right_border
}

/// Get the border-adjusted height of a floating window.
///
/// Returns the window height plus top and bottom border heights.
#[no_mangle]
pub unsafe extern "C" fn rs_win_float_total_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let height = nvim_win_get_w_height(wp);
    let top_border = nvim_win_get_border_adj(wp, 0); // Top border
    let bottom_border = nvim_win_get_border_adj(wp, 2); // Bottom border
    height + top_border + bottom_border
}

/// Calculate row position for a floating window based on anchor.
///
/// Adjusts the configured row based on the anchor direction.
#[no_mangle]
pub unsafe extern "C" fn rs_float_anchor_row(row: c_int, height: c_int, anchor: c_int) -> c_int {
    // If anchor has south (bit 0 set), subtract height
    if anchor & 1 != 0 {
        row - height
    } else {
        row
    }
}

/// Calculate column position for a floating window based on anchor.
///
/// Adjusts the configured column based on the anchor direction.
#[no_mangle]
pub unsafe extern "C" fn rs_float_anchor_col(col: c_int, width: c_int, anchor: c_int) -> c_int {
    // If anchor has east (bit 1 set), subtract width
    if anchor & 2 != 0 {
        col - width
    } else {
        col
    }
}

/// Check if a floating window is anchored to a specific window.
#[no_mangle]
pub unsafe extern "C" fn rs_float_anchored_to(wp: WinHandle, parent_handle: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let relative = nvim_win_get_config_relative(wp);
    // kFloatRelativeWindow = 1
    if relative != 1 {
        return 0;
    }

    let config_window = nvim_win_get_config_window(wp);
    c_int::from(config_window == parent_handle)
}

/// Check if a floating window overlaps a given screen region.
///
/// Returns 1 if the float overlaps the region defined by (row, col, width, height).
#[no_mangle]
pub unsafe extern "C" fn rs_float_overlaps_region(
    wp: WinHandle,
    region_row: c_int,
    region_col: c_int,
    region_width: c_int,
    region_height: c_int,
) -> c_int {
    if wp.is_null() || nvim_win_get_floating(wp) == 0 {
        return 0;
    }

    let win_row = nvim_win_get_winrow(wp);
    let win_col = nvim_win_get_wincol(wp);
    let win_width = rs_win_float_total_width(wp);
    let win_height = rs_win_float_total_height(wp);

    // Check for overlap
    let overlaps = win_row < region_row + region_height
        && win_row + win_height > region_row
        && win_col < region_col + region_width
        && win_col + win_width > region_col;

    c_int::from(overlaps)
}

/// Get the zindex of a floating window.
#[no_mangle]
pub unsafe extern "C" fn rs_float_get_zindex(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return ZINDEX_FLOAT_DEFAULT;
    }
    nvim_win_get_config_zindex(wp)
}

/// Check if a floating window is focusable.
#[no_mangle]
pub unsafe extern "C" fn rs_float_is_focusable(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_config_focusable(wp)
}

/// Check if a floating window is hidden.
#[no_mangle]
pub unsafe extern "C" fn rs_float_is_hidden(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_config_hide(wp)
}

/// Check if cursor relative positions should be used.
///
/// Returns 1 if the window's relative is Cursor or Mouse.
#[no_mangle]
pub unsafe extern "C" fn rs_float_uses_cursor_relative(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let relative = nvim_win_get_config_relative(wp);
    // kFloatRelativeCursor = 2, kFloatRelativeMouse = 3
    c_int::from(relative == 2 || relative == 3)
}

// =============================================================================
// Additional Positioning Helpers
// =============================================================================

// Note: rs_float_effective_row and rs_float_effective_col are defined in
// the compositor crate (compositor/src/floating.rs) to avoid duplication.

/// Check if a floating window would be visible on screen.
///
/// Returns 1 if the window position keeps it at least partially visible.
#[no_mangle]
pub const extern "C" fn rs_float_is_visible(
    row: c_int,
    col: c_int,
    width: c_int,
    height: c_int,
    screen_rows: c_int,
    screen_cols: c_int,
) -> c_int {
    // Window is visible if any part of it is on screen
    let row_visible = row < screen_rows && row + height > 0;
    let col_visible = col < screen_cols && col + width > 0;
    if row_visible && col_visible {
        1
    } else {
        0
    }
}

/// Calculate the clamped position to keep a window fully on screen.
///
/// Returns the adjusted row position that keeps the window fully visible.
#[no_mangle]
pub const extern "C" fn rs_float_clamp_position_row(
    row: c_int,
    height: c_int,
    screen_rows: c_int,
) -> c_int {
    if row < 0 {
        0
    } else if row + height > screen_rows {
        if height >= screen_rows {
            0
        } else {
            screen_rows - height
        }
    } else {
        row
    }
}

/// Calculate the clamped column position to keep a window fully on screen.
///
/// Returns the adjusted column position that keeps the window fully visible.
#[no_mangle]
pub const extern "C" fn rs_float_clamp_position_col(
    col: c_int,
    width: c_int,
    screen_cols: c_int,
) -> c_int {
    if col < 0 {
        0
    } else if col + width > screen_cols {
        if width >= screen_cols {
            0
        } else {
            screen_cols - width
        }
    } else {
        col
    }
}

/// Check if two rectangles overlap.
///
/// Returns 1 if the rectangles defined by (r1_row, r1_col, r1_w, r1_h) and
/// (r2_row, r2_col, r2_w, r2_h) overlap.
#[no_mangle]
pub const extern "C" fn rs_float_rects_overlap(
    r1_row: c_int,
    r1_col: c_int,
    r1_w: c_int,
    r1_h: c_int,
    r2_row: c_int,
    r2_col: c_int,
    r2_w: c_int,
    r2_h: c_int,
) -> c_int {
    let overlap = r1_row < r2_row + r2_h
        && r1_row + r1_h > r2_row
        && r1_col < r2_col + r2_w
        && r1_col + r1_w > r2_col;
    if overlap {
        1
    } else {
        0
    }
}

/// Calculate zindex comparison result for sorting.
///
/// Returns positive if za > zb, negative if za < zb, 0 if equal.
/// This is useful for sorting floating windows by z-index.
#[no_mangle]
pub const extern "C" fn rs_float_zindex_cmp(za: c_int, zb: c_int) -> c_int {
    if za == zb {
        0
    } else if za < zb {
        1 // Higher zindex first (descending order)
    } else {
        -1
    }
}

/// Check if a point is inside a rectangle.
///
/// Returns 1 if (point_row, point_col) is inside the rectangle
/// defined by (rect_row, rect_col, rect_w, rect_h).
#[no_mangle]
pub const extern "C" fn rs_float_point_in_rect(
    point_row: c_int,
    point_col: c_int,
    rect_row: c_int,
    rect_col: c_int,
    rect_w: c_int,
    rect_h: c_int,
) -> c_int {
    let inside = point_row >= rect_row
        && point_row < rect_row + rect_h
        && point_col >= rect_col
        && point_col < rect_col + rect_w;
    if inside {
        1
    } else {
        0
    }
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

    #[test]
    fn test_float_is_visible() {
        // Window fully on screen
        assert_eq!(rs_float_is_visible(5, 5, 10, 10, 50, 80), 1);

        // Window partially on screen (top-left corner visible)
        assert_eq!(rs_float_is_visible(-5, -5, 10, 10, 50, 80), 1);

        // Window partially on screen (bottom-right corner visible)
        assert_eq!(rs_float_is_visible(45, 75, 10, 10, 50, 80), 1);

        // Window completely off screen (above)
        assert_eq!(rs_float_is_visible(-15, 5, 10, 10, 50, 80), 0);

        // Window completely off screen (left)
        assert_eq!(rs_float_is_visible(5, -15, 10, 10, 50, 80), 0);

        // Window completely off screen (below)
        assert_eq!(rs_float_is_visible(55, 5, 10, 10, 50, 80), 0);

        // Window completely off screen (right)
        assert_eq!(rs_float_is_visible(5, 85, 10, 10, 50, 80), 0);
    }

    #[test]
    fn test_float_clamp_position() {
        // Row clamping
        assert_eq!(rs_float_clamp_position_row(-5, 10, 50), 0); // clamp negative
        assert_eq!(rs_float_clamp_position_row(10, 10, 50), 10); // no change needed
        assert_eq!(rs_float_clamp_position_row(45, 10, 50), 40); // clamp bottom
        assert_eq!(rs_float_clamp_position_row(0, 60, 50), 0); // window taller than screen

        // Column clamping
        assert_eq!(rs_float_clamp_position_col(-5, 10, 80), 0); // clamp negative
        assert_eq!(rs_float_clamp_position_col(20, 10, 80), 20); // no change needed
        assert_eq!(rs_float_clamp_position_col(75, 10, 80), 70); // clamp right
        assert_eq!(rs_float_clamp_position_col(0, 90, 80), 0); // window wider than screen
    }

    #[test]
    fn test_float_rects_overlap() {
        // Overlapping rectangles
        assert_eq!(rs_float_rects_overlap(0, 0, 10, 10, 5, 5, 10, 10), 1);

        // Non-overlapping rectangles (side by side)
        assert_eq!(rs_float_rects_overlap(0, 0, 10, 10, 15, 0, 10, 10), 0);

        // Non-overlapping rectangles (above/below)
        assert_eq!(rs_float_rects_overlap(0, 0, 10, 10, 0, 15, 10, 10), 0);

        // Touching but not overlapping
        assert_eq!(rs_float_rects_overlap(0, 0, 10, 10, 10, 0, 10, 10), 0);

        // One inside the other
        assert_eq!(rs_float_rects_overlap(0, 0, 20, 20, 5, 5, 5, 5), 1);

        // Partial overlap
        assert_eq!(rs_float_rects_overlap(0, 0, 10, 10, 9, 9, 10, 10), 1);
    }

    #[test]
    fn test_float_zindex_cmp() {
        // Equal z-indices
        assert_eq!(rs_float_zindex_cmp(50, 50), 0);

        // za < zb - higher zindex first (returns positive)
        assert!(rs_float_zindex_cmp(30, 50) > 0);

        // za > zb - higher zindex first (returns negative)
        assert!(rs_float_zindex_cmp(70, 50) < 0);
    }

    #[test]
    fn test_float_point_in_rect() {
        // Point inside rectangle
        assert_eq!(rs_float_point_in_rect(5, 5, 0, 0, 10, 10), 1);

        // Point at top-left corner (inside)
        assert_eq!(rs_float_point_in_rect(0, 0, 0, 0, 10, 10), 1);

        // Point at bottom-right corner (outside - exclusive)
        assert_eq!(rs_float_point_in_rect(10, 10, 0, 0, 10, 10), 0);

        // Point outside (above)
        assert_eq!(rs_float_point_in_rect(-1, 5, 0, 0, 10, 10), 0);

        // Point outside (left)
        assert_eq!(rs_float_point_in_rect(5, -1, 0, 0, 10, 10), 0);

        // Point outside (below)
        assert_eq!(rs_float_point_in_rect(15, 5, 0, 0, 10, 10), 0);

        // Point outside (right)
        assert_eq!(rs_float_point_in_rect(5, 15, 0, 0, 10, 10), 0);
    }

    #[test]
    fn test_anchor_row_calculation() {
        // These test the anchor position calculations
        unsafe {
            // NW anchor (0) - position unchanged
            assert_eq!(rs_float_anchor_row(10, 5, 0), 10);

            // SW anchor (1) - subtract height
            assert_eq!(rs_float_anchor_row(10, 5, 1), 5);

            // NE anchor (2) - row unchanged
            assert_eq!(rs_float_anchor_row(10, 5, 2), 10);

            // SE anchor (3) - subtract height
            assert_eq!(rs_float_anchor_row(10, 5, 3), 5);
        }
    }

    #[test]
    fn test_anchor_col_calculation() {
        unsafe {
            // NW anchor (0) - position unchanged
            assert_eq!(rs_float_anchor_col(20, 10, 0), 20);

            // SW anchor (1) - col unchanged
            assert_eq!(rs_float_anchor_col(20, 10, 1), 20);

            // NE anchor (2) - subtract width
            assert_eq!(rs_float_anchor_col(20, 10, 2), 10);

            // SE anchor (3) - subtract width
            assert_eq!(rs_float_anchor_col(20, 10, 3), 10);
        }
    }
}
