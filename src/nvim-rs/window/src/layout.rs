//! Window layout tree operations
//!
//! This module provides helpers for working with window layout trees,
//! including tree traversal, split direction, and layout manipulation.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_same_arms)]

use std::ffi::c_int;

// =============================================================================
// Split Direction
// =============================================================================

/// Direction for window splits.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitDir {
    /// No split (error)
    #[default]
    None = 0,
    /// Split horizontally (new window above/below)
    Horizontal = 1,
    /// Split vertically (new window left/right)
    Vertical = 2,
}

impl SplitDir {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Horizontal,
            2 => Self::Vertical,
            _ => Self::None,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this is a valid split direction.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Get the opposite direction.
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
            Self::None => Self::None,
        }
    }
}

// =============================================================================
// Split Position
// =============================================================================

/// Position for new window in a split.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitPos {
    /// Above current window
    Above = 0,
    /// Below current window
    #[default]
    Below = 1,
    /// Left of current window
    Left = 2,
    /// Right of current window
    Right = 3,
}

impl SplitPos {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Above,
            1 => Self::Below,
            2 => Self::Left,
            _ => Self::Right,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Get the split direction for this position.
    #[must_use]
    pub const fn direction(&self) -> SplitDir {
        match self {
            Self::Above | Self::Below => SplitDir::Horizontal,
            Self::Left | Self::Right => SplitDir::Vertical,
        }
    }

    /// Check if new window comes before current.
    #[must_use]
    pub const fn is_before(&self) -> bool {
        matches!(self, Self::Above | Self::Left)
    }
}

// =============================================================================
// Layout Type
// =============================================================================

/// Layout type for frames.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutType {
    /// Leaf frame (contains window)
    #[default]
    Leaf = 0,
    /// Row layout (horizontal arrangement)
    Row = 1,
    /// Column layout (vertical arrangement)
    Col = 2,
}

impl LayoutType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Row,
            2 => Self::Col,
            _ => Self::Leaf,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this is a container layout.
    #[must_use]
    pub const fn is_container(&self) -> bool {
        !matches!(self, Self::Leaf)
    }

    /// Get the split direction that creates this layout.
    #[must_use]
    pub const fn split_dir(&self) -> SplitDir {
        match self {
            Self::Leaf => SplitDir::None,
            Self::Row => SplitDir::Vertical,
            Self::Col => SplitDir::Horizontal,
        }
    }
}

// =============================================================================
// Layout Constraints
// =============================================================================

/// Size constraints for a layout element.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SizeConstraints {
    /// Minimum width
    pub min_width: c_int,
    /// Minimum height
    pub min_height: c_int,
    /// Maximum width (0 = no limit)
    pub max_width: c_int,
    /// Maximum height (0 = no limit)
    pub max_height: c_int,
    /// Whether width is fixed
    pub fixed_width: bool,
    /// Whether height is fixed
    pub fixed_height: bool,
}

impl SizeConstraints {
    /// Create new unconstrained size.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_width: 1,
            min_height: 1,
            max_width: 0,
            max_height: 0,
            fixed_width: false,
            fixed_height: false,
        }
    }

    /// Create with minimum size.
    #[must_use]
    pub const fn with_min(min_width: c_int, min_height: c_int) -> Self {
        Self {
            min_width,
            min_height,
            max_width: 0,
            max_height: 0,
            fixed_width: false,
            fixed_height: false,
        }
    }

    /// Check if width is flexible.
    #[must_use]
    pub const fn flex_width(&self) -> bool {
        !self.fixed_width && (self.max_width == 0 || self.max_width > self.min_width)
    }

    /// Check if height is flexible.
    #[must_use]
    pub const fn flex_height(&self) -> bool {
        !self.fixed_height && (self.max_height == 0 || self.max_height > self.min_height)
    }

    /// Clamp a width value to constraints.
    #[must_use]
    pub const fn clamp_width(&self, width: c_int) -> c_int {
        let mut result = width;
        if result < self.min_width {
            result = self.min_width;
        }
        if self.max_width > 0 && result > self.max_width {
            result = self.max_width;
        }
        result
    }

    /// Clamp a height value to constraints.
    #[must_use]
    pub const fn clamp_height(&self, height: c_int) -> c_int {
        let mut result = height;
        if result < self.min_height {
            result = self.min_height;
        }
        if self.max_height > 0 && result > self.max_height {
            result = self.max_height;
        }
        result
    }
}

// =============================================================================
// Layout Statistics
// =============================================================================

/// Statistics about a layout tree.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutStats {
    /// Total number of windows
    pub window_count: c_int,
    /// Total number of frames
    pub frame_count: c_int,
    /// Maximum nesting depth
    pub max_depth: c_int,
    /// Number of fixed-height windows
    pub fixed_height_count: c_int,
    /// Number of fixed-width windows
    pub fixed_width_count: c_int,
}

impl LayoutStats {
    /// Create empty stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            window_count: 0,
            frame_count: 0,
            max_depth: 0,
            fixed_height_count: 0,
            fixed_width_count: 0,
        }
    }

    /// Check if layout is simple (single window).
    #[must_use]
    pub const fn is_simple(&self) -> bool {
        self.window_count == 1
    }

    /// Check if layout has fixed constraints.
    #[must_use]
    pub const fn has_fixed(&self) -> bool {
        self.fixed_height_count > 0 || self.fixed_width_count > 0
    }
}

// =============================================================================
// Resize Direction
// =============================================================================

/// Direction for window resize operations.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResizeDir {
    /// Resize width
    #[default]
    Width = 0,
    /// Resize height
    Height = 1,
    /// Resize both
    Both = 2,
}

impl ResizeDir {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Height,
            2 => Self::Both,
            _ => Self::Width,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this includes width.
    #[must_use]
    pub const fn includes_width(&self) -> bool {
        matches!(self, Self::Width | Self::Both)
    }

    /// Check if this includes height.
    #[must_use]
    pub const fn includes_height(&self) -> bool {
        matches!(self, Self::Height | Self::Both)
    }
}

// =============================================================================
// Size Distribution
// =============================================================================

/// Result of distributing size among children.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SizeDistribution {
    /// Size per flexible element
    pub per_flex: c_int,
    /// Extra pixels for first N elements
    pub extra: c_int,
    /// Number of flexible elements
    pub flex_count: c_int,
    /// Total fixed size
    pub fixed_total: c_int,
}

impl SizeDistribution {
    /// Calculate distribution for given total and constraints.
    #[must_use]
    pub const fn calculate(total: c_int, fixed: c_int, flex_count: c_int) -> Self {
        if flex_count == 0 {
            return Self {
                per_flex: 0,
                extra: 0,
                flex_count: 0,
                fixed_total: fixed,
            };
        }

        let available = total - fixed;
        if available <= 0 {
            return Self {
                per_flex: 0,
                extra: 0,
                flex_count,
                fixed_total: fixed,
            };
        }

        let per_flex = available / flex_count;
        let extra = available % flex_count;

        Self {
            per_flex,
            extra,
            flex_count,
            fixed_total: fixed,
        }
    }

    /// Get size for element at index.
    #[must_use]
    pub const fn size_at(&self, index: c_int) -> c_int {
        if index < self.extra {
            self.per_flex + 1
        } else {
            self.per_flex
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get split direction from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_dir(value: c_int) -> c_int {
    SplitDir::from_raw(value).to_raw()
}

/// Get opposite split direction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_dir_opposite(value: c_int) -> c_int {
    SplitDir::from_raw(value).opposite().to_raw()
}

/// Get layout type from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_type(value: c_int) -> c_int {
    LayoutType::from_raw(value).to_raw()
}

/// Check if layout type is a container.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_is_container(value: c_int) -> c_int {
    c_int::from(LayoutType::from_raw(value).is_container())
}

/// Calculate size distribution.
#[unsafe(no_mangle)]
pub extern "C" fn rs_size_distribution(total: c_int, fixed: c_int, flex_count: c_int) -> c_int {
    SizeDistribution::calculate(total, fixed, flex_count).per_flex
}

/// Get size distribution extra pixels.
#[unsafe(no_mangle)]
pub extern "C" fn rs_size_distribution_extra(
    total: c_int,
    fixed: c_int,
    flex_count: c_int,
) -> c_int {
    SizeDistribution::calculate(total, fixed, flex_count).extra
}

/// Check if split direction is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_dir_is_valid(value: c_int) -> c_int {
    c_int::from(SplitDir::from_raw(value).is_valid())
}

/// Get split direction for position.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_pos_direction(value: c_int) -> c_int {
    SplitPos::from_raw(value).direction().to_raw()
}

/// Check if split position is before.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_pos_is_before(value: c_int) -> c_int {
    c_int::from(SplitPos::from_raw(value).is_before())
}

/// Get layout split direction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_layout_split_dir(value: c_int) -> c_int {
    LayoutType::from_raw(value).split_dir().to_raw()
}

/// Clamp width to constraints.
#[unsafe(no_mangle)]
pub extern "C" fn rs_size_clamp_width(
    width: c_int,
    min_width: c_int,
    max_width: c_int,
) -> c_int {
    let constraints = SizeConstraints {
        min_width,
        min_height: 1,
        max_width,
        max_height: 0,
        fixed_width: false,
        fixed_height: false,
    };
    constraints.clamp_width(width)
}

/// Clamp height to constraints.
#[unsafe(no_mangle)]
pub extern "C" fn rs_size_clamp_height(
    height: c_int,
    min_height: c_int,
    max_height: c_int,
) -> c_int {
    let constraints = SizeConstraints {
        min_width: 1,
        min_height,
        max_width: 0,
        max_height,
        fixed_width: false,
        fixed_height: false,
    };
    constraints.clamp_height(height)
}

/// Get resize direction includes width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_includes_width(value: c_int) -> c_int {
    c_int::from(ResizeDir::from_raw(value).includes_width())
}

/// Get resize direction includes height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_includes_height(value: c_int) -> c_int {
    c_int::from(ResizeDir::from_raw(value).includes_height())
}

/// Get SplitDir::None constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_dir_none() -> c_int {
    SplitDir::None.to_raw()
}

/// Get SplitDir::Horizontal constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_dir_horizontal() -> c_int {
    SplitDir::Horizontal.to_raw()
}

/// Get SplitDir::Vertical constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_dir_vertical() -> c_int {
    SplitDir::Vertical.to_raw()
}

/// Get SplitPos::Above constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_pos_above() -> c_int {
    SplitPos::Above.to_raw()
}

/// Get SplitPos::Below constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_pos_below() -> c_int {
    SplitPos::Below.to_raw()
}

/// Get SplitPos::Left constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_pos_left() -> c_int {
    SplitPos::Left.to_raw()
}

/// Get SplitPos::Right constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_split_pos_right() -> c_int {
    SplitPos::Right.to_raw()
}

/// Get ResizeDir::Width constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_dir_width() -> c_int {
    ResizeDir::Width.to_raw()
}

/// Get ResizeDir::Height constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_dir_height() -> c_int {
    ResizeDir::Height.to_raw()
}

/// Get ResizeDir::Both constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_resize_dir_both() -> c_int {
    ResizeDir::Both.to_raw()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_dir() {
        assert_eq!(SplitDir::from_raw(1), SplitDir::Horizontal);
        assert_eq!(SplitDir::from_raw(2), SplitDir::Vertical);
        assert_eq!(SplitDir::from_raw(0), SplitDir::None);

        assert!(SplitDir::Horizontal.is_valid());
        assert!(!SplitDir::None.is_valid());

        assert_eq!(SplitDir::Horizontal.opposite(), SplitDir::Vertical);
        assert_eq!(SplitDir::Vertical.opposite(), SplitDir::Horizontal);
    }

    #[test]
    fn test_split_pos() {
        assert_eq!(SplitPos::Above.direction(), SplitDir::Horizontal);
        assert_eq!(SplitPos::Below.direction(), SplitDir::Horizontal);
        assert_eq!(SplitPos::Left.direction(), SplitDir::Vertical);
        assert_eq!(SplitPos::Right.direction(), SplitDir::Vertical);

        assert!(SplitPos::Above.is_before());
        assert!(SplitPos::Left.is_before());
        assert!(!SplitPos::Below.is_before());
        assert!(!SplitPos::Right.is_before());
    }

    #[test]
    fn test_layout_type() {
        assert_eq!(LayoutType::from_raw(0), LayoutType::Leaf);
        assert_eq!(LayoutType::from_raw(1), LayoutType::Row);
        assert_eq!(LayoutType::from_raw(2), LayoutType::Col);

        assert!(!LayoutType::Leaf.is_container());
        assert!(LayoutType::Row.is_container());
        assert!(LayoutType::Col.is_container());

        assert_eq!(LayoutType::Row.split_dir(), SplitDir::Vertical);
        assert_eq!(LayoutType::Col.split_dir(), SplitDir::Horizontal);
    }

    #[test]
    fn test_size_constraints() {
        let constraints = SizeConstraints::new();
        assert!(constraints.flex_width());
        assert!(constraints.flex_height());

        let clamped = constraints.clamp_width(0);
        assert_eq!(clamped, 1); // min_width

        let constraints = SizeConstraints::with_min(10, 5);
        assert_eq!(constraints.clamp_width(3), 10);
        assert_eq!(constraints.clamp_height(3), 5);
    }

    #[test]
    fn test_layout_stats() {
        let stats = LayoutStats::new();
        assert!(!stats.is_simple()); // 0 windows is not simple
        assert!(!stats.has_fixed());

        let stats = LayoutStats {
            window_count: 1,
            ..LayoutStats::new()
        };
        assert!(stats.is_simple());
    }

    #[test]
    fn test_resize_dir() {
        assert!(ResizeDir::Width.includes_width());
        assert!(!ResizeDir::Width.includes_height());
        assert!(!ResizeDir::Height.includes_width());
        assert!(ResizeDir::Height.includes_height());
        assert!(ResizeDir::Both.includes_width());
        assert!(ResizeDir::Both.includes_height());
    }

    #[test]
    fn test_size_distribution() {
        // 100 pixels, 20 fixed, 4 flexible elements
        let dist = SizeDistribution::calculate(100, 20, 4);
        assert_eq!(dist.per_flex, 20); // 80 / 4
        assert_eq!(dist.extra, 0);

        // 100 pixels, 20 fixed, 3 flexible elements
        let dist = SizeDistribution::calculate(100, 20, 3);
        assert_eq!(dist.per_flex, 26); // 80 / 3 = 26
        assert_eq!(dist.extra, 2); // 80 % 3 = 2

        // First 2 elements get 27, rest get 26
        assert_eq!(dist.size_at(0), 27);
        assert_eq!(dist.size_at(1), 27);
        assert_eq!(dist.size_at(2), 26);
    }
}
