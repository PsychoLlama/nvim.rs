//! Floating Window Compositing Utilities
//!
//! This module provides utilities for compositing floating windows,
//! including clipping calculations, overlap detection, and positioning.
//! Phase 173 of Rust migration.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_lossless)]

use std::ffi::c_int;

use crate::ScreenGridHandle;

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    fn nvim_screengrid_get_comp_row(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_col(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_rows(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_cols(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_zindex(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_blending(grid: ScreenGridHandle) -> bool;
}

// =============================================================================
// Clipping Rectangle
// =============================================================================

/// A rectangular region for clipping calculations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClipRect {
    /// Top row (inclusive)
    pub top: c_int,
    /// Bottom row (exclusive)
    pub bottom: c_int,
    /// Left column (inclusive)
    pub left: c_int,
    /// Right column (exclusive)
    pub right: c_int,
}

impl ClipRect {
    /// Create a new clip rectangle.
    #[inline]
    pub const fn new(top: c_int, bottom: c_int, left: c_int, right: c_int) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Create an empty clip rectangle.
    #[inline]
    pub const fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Check if rectangle is empty (has zero or negative area).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.right <= self.left || self.bottom <= self.top
    }

    /// Get the width of the rectangle.
    #[inline]
    pub const fn width(&self) -> c_int {
        if self.right > self.left {
            self.right - self.left
        } else {
            0
        }
    }

    /// Get the height of the rectangle.
    #[inline]
    pub const fn height(&self) -> c_int {
        if self.bottom > self.top {
            self.bottom - self.top
        } else {
            0
        }
    }

    /// Check if a point is inside the rectangle.
    #[inline]
    pub const fn contains_point(&self, row: c_int, col: c_int) -> bool {
        row >= self.top && row < self.bottom && col >= self.left && col < self.right
    }

    /// Check if another rectangle overlaps with this one.
    #[inline]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top < other.bottom
            && self.bottom > other.top
    }

    /// Compute the intersection with another rectangle.
    #[inline]
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        let top = self.top.max(other.top);
        let bottom = self.bottom.min(other.bottom);
        let left = self.left.max(other.left);
        let right = self.right.min(other.right);

        if top >= bottom || left >= right {
            Self::empty()
        } else {
            Self::new(top, bottom, left, right)
        }
    }

    /// Compute the union (bounding box) with another rectangle.
    #[inline]
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        Self::new(
            self.top.min(other.top),
            self.bottom.max(other.bottom),
            self.left.min(other.left),
            self.right.max(other.right),
        )
    }
}

// =============================================================================
// FFI Exports for ClipRect
// =============================================================================

/// Create a new clip rectangle.
#[no_mangle]
pub extern "C" fn rs_clip_rect_new(
    top: c_int,
    bottom: c_int,
    left: c_int,
    right: c_int,
) -> ClipRect {
    ClipRect::new(top, bottom, left, right)
}

/// Check if clip rectangle is empty.
#[no_mangle]
pub extern "C" fn rs_clip_rect_is_empty(rect: &ClipRect) -> c_int {
    c_int::from(rect.is_empty())
}

/// Get clip rectangle width.
#[no_mangle]
pub extern "C" fn rs_clip_rect_width(rect: &ClipRect) -> c_int {
    rect.width()
}

/// Get clip rectangle height.
#[no_mangle]
pub extern "C" fn rs_clip_rect_height(rect: &ClipRect) -> c_int {
    rect.height()
}

/// Check if point is inside clip rectangle.
#[no_mangle]
pub extern "C" fn rs_clip_rect_contains(rect: &ClipRect, row: c_int, col: c_int) -> c_int {
    c_int::from(rect.contains_point(row, col))
}

/// Check if two clip rectangles overlap.
#[no_mangle]
pub extern "C" fn rs_clip_rect_overlaps(rect1: &ClipRect, rect2: &ClipRect) -> c_int {
    c_int::from(rect1.overlaps(rect2))
}

/// Compute intersection of two clip rectangles.
#[no_mangle]
pub extern "C" fn rs_clip_rect_intersect(rect1: &ClipRect, rect2: &ClipRect) -> ClipRect {
    rect1.intersect(rect2)
}

/// Compute union of two clip rectangles.
#[no_mangle]
pub extern "C" fn rs_clip_rect_union(rect1: &ClipRect, rect2: &ClipRect) -> ClipRect {
    rect1.union(rect2)
}

// =============================================================================
// Grid Clipping Helpers
// =============================================================================

/// Get the bounding rectangle of a grid.
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_bounds(grid: ScreenGridHandle) -> ClipRect {
    if grid.is_null() {
        return ClipRect::empty();
    }
    let row = nvim_screengrid_get_comp_row(grid);
    let col = nvim_screengrid_get_comp_col(grid);
    let rows = nvim_screengrid_get_rows(grid);
    let cols = nvim_screengrid_get_cols(grid);
    ClipRect::new(row, row + rows, col, col + cols)
}

/// Check if a grid overlaps with a given rectangle.
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_overlaps_rect(grid: ScreenGridHandle, rect: &ClipRect) -> c_int {
    let grid_bounds = rs_grid_bounds(grid);
    c_int::from(grid_bounds.overlaps(rect))
}

/// Compute the intersection of a grid with a clipping rectangle.
///
/// Returns the area of the grid that is visible within the clip rect.
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_clip_to_rect(grid: ScreenGridHandle, clip: &ClipRect) -> ClipRect {
    let grid_bounds = rs_grid_bounds(grid);
    grid_bounds.intersect(clip)
}

// =============================================================================
// Floating Window Overlap Detection
// =============================================================================

/// Check if grid1 is above grid2 in z-order (higher z-index).
///
/// # Safety
/// Both handles must be valid ScreenGrid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_is_above(
    grid1: ScreenGridHandle,
    grid2: ScreenGridHandle,
) -> c_int {
    if grid1.is_null() || grid2.is_null() {
        return 0;
    }
    let z1 = nvim_screengrid_get_zindex(grid1);
    let z2 = nvim_screengrid_get_zindex(grid2);
    c_int::from(z1 > z2)
}

/// Check if grid1 is at the same z-level as grid2.
///
/// # Safety
/// Both handles must be valid ScreenGrid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_same_zlevel(
    grid1: ScreenGridHandle,
    grid2: ScreenGridHandle,
) -> c_int {
    if grid1.is_null() || grid2.is_null() {
        return 0;
    }
    let z1 = nvim_screengrid_get_zindex(grid1);
    let z2 = nvim_screengrid_get_zindex(grid2);
    c_int::from(z1 == z2)
}

/// Check if two grids have overlapping areas.
///
/// # Safety
/// Both handles must be valid ScreenGrid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_grids_overlap(
    grid1: ScreenGridHandle,
    grid2: ScreenGridHandle,
) -> c_int {
    if grid1.is_null() || grid2.is_null() {
        return 0;
    }
    let bounds1 = rs_grid_bounds(grid1);
    let bounds2 = rs_grid_bounds(grid2);
    c_int::from(bounds1.overlaps(&bounds2))
}

/// Compute the overlapping area between two grids.
///
/// # Safety
/// Both handles must be valid ScreenGrid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_grids_overlap_area(
    grid1: ScreenGridHandle,
    grid2: ScreenGridHandle,
) -> ClipRect {
    if grid1.is_null() || grid2.is_null() {
        return ClipRect::empty();
    }
    let bounds1 = rs_grid_bounds(grid1);
    let bounds2 = rs_grid_bounds(grid2);
    bounds1.intersect(&bounds2)
}

// =============================================================================
// Blending Support
// =============================================================================

/// Check if a grid uses blending (transparency).
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_uses_blending(grid: ScreenGridHandle) -> c_int {
    if grid.is_null() {
        return 0;
    }
    c_int::from(nvim_screengrid_get_blending(grid))
}

/// Check if blending is needed between two overlapping grids.
///
/// Blending is needed if either grid uses transparency and they overlap.
///
/// # Safety
/// Both handles must be valid ScreenGrid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_grids_need_blending(
    top_grid: ScreenGridHandle,
    bottom_grid: ScreenGridHandle,
) -> c_int {
    if top_grid.is_null() || bottom_grid.is_null() {
        return 0;
    }

    // Check if they overlap
    let bounds_top = rs_grid_bounds(top_grid);
    let bounds_bottom = rs_grid_bounds(bottom_grid);
    if !bounds_top.overlaps(&bounds_bottom) {
        return 0;
    }

    // Blending needed if top grid has blending enabled
    c_int::from(nvim_screengrid_get_blending(top_grid))
}

// =============================================================================
// Position Calculations
// =============================================================================

/// Calculate the effective row for a floating window position.
///
/// Accounts for anchor position (north vs south).
#[no_mangle]
pub extern "C" fn rs_float_effective_row(
    base_row: c_int,
    height: c_int,
    anchor_south: c_int,
) -> c_int {
    if anchor_south != 0 {
        base_row - height
    } else {
        base_row
    }
}

/// Calculate the effective column for a floating window position.
///
/// Accounts for anchor position (west vs east).
#[no_mangle]
pub extern "C" fn rs_float_effective_col(
    base_col: c_int,
    width: c_int,
    anchor_east: c_int,
) -> c_int {
    if anchor_east != 0 {
        base_col - width
    } else {
        base_col
    }
}

/// Clamp a floating window row to screen bounds.
#[no_mangle]
pub extern "C" fn rs_float_clamp_row(row: c_int, height: c_int, screen_rows: c_int) -> c_int {
    if row < 0 {
        0
    } else if row + height > screen_rows {
        (screen_rows - height).max(0)
    } else {
        row
    }
}

/// Clamp a floating window column to screen bounds.
#[no_mangle]
pub extern "C" fn rs_float_clamp_col(col: c_int, width: c_int, screen_cols: c_int) -> c_int {
    if col < 0 {
        0
    } else if col + width > screen_cols {
        (screen_cols - width).max(0)
    } else {
        col
    }
}

/// Calculate the visible portion of a floating window.
///
/// Returns the clipped bounds after accounting for screen edges.
#[no_mangle]
pub extern "C" fn rs_float_visible_bounds(
    row: c_int,
    col: c_int,
    height: c_int,
    width: c_int,
    screen_rows: c_int,
    screen_cols: c_int,
) -> ClipRect {
    let screen_bounds = ClipRect::new(0, screen_rows, 0, screen_cols);
    let float_bounds = ClipRect::new(row, row + height, col, col + width);
    float_bounds.intersect(&screen_bounds)
}

// =============================================================================
// Border Calculations
// =============================================================================

/// Calculate total height including border.
#[no_mangle]
pub extern "C" fn rs_float_total_height(
    content_height: c_int,
    has_top_border: c_int,
    has_bottom_border: c_int,
) -> c_int {
    content_height + (has_top_border != 0) as c_int + (has_bottom_border != 0) as c_int
}

/// Calculate total width including border.
#[no_mangle]
pub extern "C" fn rs_float_total_width(
    content_width: c_int,
    has_left_border: c_int,
    has_right_border: c_int,
) -> c_int {
    content_width + (has_left_border != 0) as c_int + (has_right_border != 0) as c_int
}

/// Get content offset from border (row offset).
#[no_mangle]
pub extern "C" fn rs_float_content_row_offset(has_top_border: c_int) -> c_int {
    (has_top_border != 0) as c_int
}

/// Get content offset from border (column offset).
#[no_mangle]
pub extern "C" fn rs_float_content_col_offset(has_left_border: c_int) -> c_int {
    (has_left_border != 0) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_rect_basic() {
        let rect = ClipRect::new(10, 20, 5, 15);
        assert!(!rect.is_empty());
        assert_eq!(rect.width(), 10);
        assert_eq!(rect.height(), 10);
    }

    #[test]
    fn test_clip_rect_empty() {
        let empty = ClipRect::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.width(), 0);
        assert_eq!(empty.height(), 0);

        // Invalid rectangles
        let invalid1 = ClipRect::new(20, 10, 5, 15); // bottom < top
        assert!(invalid1.is_empty());

        let invalid2 = ClipRect::new(10, 20, 15, 5); // right < left
        assert!(invalid2.is_empty());
    }

    #[test]
    fn test_clip_rect_contains() {
        let rect = ClipRect::new(10, 20, 5, 15);
        assert!(rect.contains_point(10, 5)); // top-left corner
        assert!(rect.contains_point(15, 10)); // middle
        assert!(!rect.contains_point(20, 10)); // at bottom (exclusive)
        assert!(!rect.contains_point(15, 15)); // at right (exclusive)
        assert!(!rect.contains_point(5, 10)); // outside
    }

    #[test]
    fn test_clip_rect_overlaps() {
        let rect1 = ClipRect::new(10, 20, 5, 15);
        let rect2 = ClipRect::new(15, 25, 10, 20);
        assert!(rect1.overlaps(&rect2));

        let rect3 = ClipRect::new(25, 30, 5, 15); // no overlap
        assert!(!rect1.overlaps(&rect3));

        // Adjacent but not overlapping
        let rect4 = ClipRect::new(20, 30, 5, 15);
        assert!(!rect1.overlaps(&rect4));
    }

    #[test]
    fn test_clip_rect_intersect() {
        let rect1 = ClipRect::new(10, 20, 5, 15);
        let rect2 = ClipRect::new(15, 25, 10, 20);
        let intersection = rect1.intersect(&rect2);
        assert_eq!(intersection.top, 15);
        assert_eq!(intersection.bottom, 20);
        assert_eq!(intersection.left, 10);
        assert_eq!(intersection.right, 15);
    }

    #[test]
    fn test_clip_rect_union() {
        let rect1 = ClipRect::new(10, 20, 5, 15);
        let rect2 = ClipRect::new(15, 25, 10, 20);
        let union = rect1.union(&rect2);
        assert_eq!(union.top, 10);
        assert_eq!(union.bottom, 25);
        assert_eq!(union.left, 5);
        assert_eq!(union.right, 20);
    }

    #[test]
    fn test_float_effective_row() {
        assert_eq!(rs_float_effective_row(10, 5, 0), 10); // north anchor
        assert_eq!(rs_float_effective_row(10, 5, 1), 5); // south anchor
    }

    #[test]
    fn test_float_effective_col() {
        assert_eq!(rs_float_effective_col(20, 10, 0), 20); // west anchor
        assert_eq!(rs_float_effective_col(20, 10, 1), 10); // east anchor
    }

    #[test]
    fn test_float_clamp_row() {
        assert_eq!(rs_float_clamp_row(-5, 10, 50), 0); // clamp to top
        assert_eq!(rs_float_clamp_row(10, 10, 50), 10); // no clamp
        assert_eq!(rs_float_clamp_row(45, 10, 50), 40); // clamp to bottom
    }

    #[test]
    fn test_float_clamp_col() {
        assert_eq!(rs_float_clamp_col(-5, 10, 80), 0); // clamp to left
        assert_eq!(rs_float_clamp_col(20, 10, 80), 20); // no clamp
        assert_eq!(rs_float_clamp_col(75, 10, 80), 70); // clamp to right
    }

    #[test]
    fn test_float_visible_bounds() {
        // Fully visible
        let bounds = rs_float_visible_bounds(10, 20, 5, 10, 50, 80);
        assert_eq!(bounds, ClipRect::new(10, 15, 20, 30));

        // Partially clipped
        let bounds = rs_float_visible_bounds(45, 75, 10, 10, 50, 80);
        assert_eq!(bounds, ClipRect::new(45, 50, 75, 80));

        // Completely outside
        let bounds = rs_float_visible_bounds(-20, -20, 5, 5, 50, 80);
        assert!(bounds.is_empty());
    }

    #[test]
    fn test_float_total_dimensions() {
        assert_eq!(rs_float_total_height(10, 0, 0), 10);
        assert_eq!(rs_float_total_height(10, 1, 0), 11);
        assert_eq!(rs_float_total_height(10, 0, 1), 11);
        assert_eq!(rs_float_total_height(10, 1, 1), 12);

        assert_eq!(rs_float_total_width(20, 0, 0), 20);
        assert_eq!(rs_float_total_width(20, 1, 0), 21);
        assert_eq!(rs_float_total_width(20, 0, 1), 21);
        assert_eq!(rs_float_total_width(20, 1, 1), 22);
    }

    #[test]
    fn test_float_content_offset() {
        assert_eq!(rs_float_content_row_offset(0), 0);
        assert_eq!(rs_float_content_row_offset(1), 1);
        assert_eq!(rs_float_content_col_offset(0), 0);
        assert_eq!(rs_float_content_col_offset(1), 1);
    }

    #[test]
    fn test_ffi_clip_rect() {
        let rect = rs_clip_rect_new(10, 20, 5, 15);
        assert_eq!(rs_clip_rect_is_empty(&rect), 0);
        assert_eq!(rs_clip_rect_width(&rect), 10);
        assert_eq!(rs_clip_rect_height(&rect), 10);
        assert_eq!(rs_clip_rect_contains(&rect, 15, 10), 1);
        assert_eq!(rs_clip_rect_contains(&rect, 25, 10), 0);
    }
}
