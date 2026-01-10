//! Window Ex commands (:split, :vsplit, :close, :only, :new, :vnew)
//!
//! This module implements Ex commands for window management.

use std::ffi::c_int;

// =============================================================================
// Split Direction
// =============================================================================

/// Direction for window split.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitDirection {
    /// Horizontal split (:split)
    #[default]
    Horizontal = 0,
    /// Vertical split (:vsplit)
    Vertical = 1,
}

impl SplitDirection {
    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Vertical,
            _ => Self::Horizontal,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this is a vertical split.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Vertical)
    }
}

// =============================================================================
// Split Position
// =============================================================================

/// Position for new window after split.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitPosition {
    /// New window above/left (default for horizontal/vertical)
    #[default]
    Before = 0,
    /// New window below/right
    After = 1,
}

impl SplitPosition {
    /// Create from splitbelow/splitright options.
    #[must_use]
    pub const fn from_options(splitbelow: bool, splitright: bool, is_vertical: bool) -> Self {
        if is_vertical {
            if splitright {
                Self::After
            } else {
                Self::Before
            }
        } else if splitbelow {
            Self::After
        } else {
            Self::Before
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Window Size Calculations
// =============================================================================

/// Minimum window height (excluding status line).
pub const MIN_WINDOW_HEIGHT: c_int = 1;

/// Minimum window width.
pub const MIN_WINDOW_WIDTH: c_int = 1;

/// Calculate the size for a new split window.
///
/// # Arguments
/// * `total_size` - Total available size
/// * `requested` - Requested size (0 for half)
/// * `min_size` - Minimum allowed size
///
/// # Returns
/// Size for the new window
#[must_use]
pub const fn calc_split_size(total_size: c_int, requested: c_int, min_size: c_int) -> c_int {
    if requested > 0 {
        // Use requested size, clamped to available
        let max_new = total_size - min_size;
        if requested > max_new {
            max_new
        } else if requested < min_size {
            min_size
        } else {
            requested
        }
    } else {
        // Split in half
        total_size / 2
    }
}

/// Check if there's enough room for a new window.
///
/// # Arguments
/// * `total_size` - Total available size
/// * `min_size` - Minimum size for each window
///
/// # Returns
/// true if split is possible
#[must_use]
pub const fn can_split(total_size: c_int, min_size: c_int) -> bool {
    total_size >= min_size * 2
}

// =============================================================================
// Window Close Behavior
// =============================================================================

/// Behavior when closing a window.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CloseAction {
    /// Just close the window (:close)
    #[default]
    Close = 0,
    /// Close other windows (:only)
    Only = 1,
    /// Hide the window
    Hide = 2,
}

impl CloseAction {
    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Only,
            2 => Self::Hide,
            _ => Self::Close,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

/// Check if window can be closed.
///
/// # Arguments
/// * `is_last` - Whether this is the last window
/// * `is_last_tab` - Whether this is the last tab
/// * `force` - Whether ! was used
/// * `modified` - Whether buffer is modified
/// * `hidden` - Whether 'hidden' option is set
///
/// # Returns
/// true if window can be closed
#[must_use]
pub const fn can_close_window(
    is_last: bool,
    is_last_tab: bool,
    force: bool,
    modified: bool,
    hidden: bool,
) -> bool {
    // Can't close the last window in the last tab
    if is_last && is_last_tab {
        return false;
    }

    // Check for unsaved changes
    if modified && !force && !hidden {
        return false;
    }

    true
}

/// Check if :only command can be executed.
///
/// # Arguments
/// * `win_count` - Number of windows in current tab
/// * `has_modified` - Whether any windows have modified buffers
/// * `force` - Whether ! was used
///
/// # Returns
/// true if :only can proceed
#[must_use]
pub const fn can_do_only(win_count: c_int, has_modified: bool, force: bool) -> bool {
    if win_count <= 1 {
        return true; // Already only one window
    }

    // With modified buffers, need force
    !has_modified || force
}

// =============================================================================
// Window Navigation
// =============================================================================

/// Direction for window navigation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WinNavDirection {
    /// Move to window above
    Up = 0,
    /// Move to window below
    #[default]
    Down = 1,
    /// Move to window left
    Left = 2,
    /// Move to window right
    Right = 3,
}

impl WinNavDirection {
    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Up,
            2 => Self::Left,
            3 => Self::Right,
            _ => Self::Down,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this is a horizontal movement.
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Check if this is a vertical movement.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get split direction from raw value.
#[no_mangle]
pub extern "C" fn rs_split_direction_from_raw(value: c_int) -> c_int {
    SplitDirection::from_raw(value).to_raw()
}

/// FFI: Check if split direction is vertical.
#[no_mangle]
pub extern "C" fn rs_split_is_vertical(dir: c_int) -> c_int {
    c_int::from(SplitDirection::from_raw(dir).is_vertical())
}

/// FFI: Get split position from options.
#[no_mangle]
pub extern "C" fn rs_split_position_from_options(
    splitbelow: c_int,
    splitright: c_int,
    is_vertical: c_int,
) -> c_int {
    SplitPosition::from_options(splitbelow != 0, splitright != 0, is_vertical != 0).to_raw()
}

/// FFI: Calculate split size.
#[no_mangle]
pub extern "C" fn rs_calc_split_size(
    total_size: c_int,
    requested: c_int,
    min_size: c_int,
) -> c_int {
    calc_split_size(total_size, requested, min_size)
}

/// FFI: Check if split is possible.
#[no_mangle]
pub extern "C" fn rs_can_split(total_size: c_int, min_size: c_int) -> c_int {
    c_int::from(can_split(total_size, min_size))
}

/// FFI: Get close action from raw value.
#[no_mangle]
pub extern "C" fn rs_close_action_from_raw(value: c_int) -> c_int {
    CloseAction::from_raw(value).to_raw()
}

/// FFI: Check if window can be closed.
#[no_mangle]
pub extern "C" fn rs_can_close_window(
    is_last: c_int,
    is_last_tab: c_int,
    force: c_int,
    modified: c_int,
    hidden: c_int,
) -> c_int {
    c_int::from(can_close_window(
        is_last != 0,
        is_last_tab != 0,
        force != 0,
        modified != 0,
        hidden != 0,
    ))
}

/// FFI: Check if :only can proceed.
#[no_mangle]
pub extern "C" fn rs_can_do_only(win_count: c_int, has_modified: c_int, force: c_int) -> c_int {
    c_int::from(can_do_only(win_count, has_modified != 0, force != 0))
}

/// FFI: Get window navigation direction from raw value.
#[no_mangle]
pub extern "C" fn rs_win_nav_from_raw(value: c_int) -> c_int {
    WinNavDirection::from_raw(value).to_raw()
}

/// FFI: Check if navigation direction is horizontal.
#[no_mangle]
pub extern "C" fn rs_win_nav_is_horizontal(dir: c_int) -> c_int {
    c_int::from(WinNavDirection::from_raw(dir).is_horizontal())
}

/// FFI: Get minimum window height.
#[no_mangle]
pub extern "C" fn rs_min_window_height() -> c_int {
    MIN_WINDOW_HEIGHT
}

/// FFI: Get minimum window width.
#[no_mangle]
pub extern "C" fn rs_min_window_width() -> c_int {
    MIN_WINDOW_WIDTH
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_direction() {
        assert!(!SplitDirection::Horizontal.is_vertical());
        assert!(SplitDirection::Vertical.is_vertical());

        assert_eq!(SplitDirection::from_raw(0), SplitDirection::Horizontal);
        assert_eq!(SplitDirection::from_raw(1), SplitDirection::Vertical);
    }

    #[test]
    fn test_split_position() {
        // splitbelow=false, splitright=false, horizontal
        assert_eq!(
            SplitPosition::from_options(false, false, false),
            SplitPosition::Before
        );

        // splitbelow=true, horizontal
        assert_eq!(
            SplitPosition::from_options(true, false, false),
            SplitPosition::After
        );

        // splitright=true, vertical
        assert_eq!(
            SplitPosition::from_options(false, true, true),
            SplitPosition::After
        );
    }

    #[test]
    fn test_calc_split_size() {
        // No request - split in half
        assert_eq!(calc_split_size(100, 0, 1), 50);

        // Requested size within bounds
        assert_eq!(calc_split_size(100, 30, 1), 30);

        // Requested too large - clamp
        assert_eq!(calc_split_size(100, 120, 1), 99);

        // Requested too small - clamp to min
        assert_eq!(calc_split_size(100, 0, 10), 50);
    }

    #[test]
    fn test_can_split() {
        assert!(can_split(10, 5));
        assert!(can_split(10, 4));
        assert!(!can_split(10, 6));
        assert!(!can_split(5, 10));
    }

    #[test]
    fn test_close_action() {
        assert_eq!(CloseAction::from_raw(0), CloseAction::Close);
        assert_eq!(CloseAction::from_raw(1), CloseAction::Only);
        assert_eq!(CloseAction::from_raw(2), CloseAction::Hide);
    }

    #[test]
    fn test_can_close_window() {
        // Can close non-last window
        assert!(can_close_window(false, false, false, false, false));

        // Cannot close last window in last tab
        assert!(!can_close_window(true, true, false, false, false));

        // Can close last window if not last tab
        assert!(can_close_window(true, false, false, false, false));

        // Cannot close with unsaved changes without force
        assert!(!can_close_window(false, false, false, true, false));

        // Can close with unsaved if force
        assert!(can_close_window(false, false, true, true, false));

        // Can close with unsaved if hidden
        assert!(can_close_window(false, false, false, true, true));
    }

    #[test]
    fn test_can_do_only() {
        // One window - always ok
        assert!(can_do_only(1, false, false));
        assert!(can_do_only(1, true, false));

        // Multiple windows, no modified
        assert!(can_do_only(3, false, false));

        // Multiple windows, modified, no force
        assert!(!can_do_only(3, true, false));

        // Multiple windows, modified, with force
        assert!(can_do_only(3, true, true));
    }

    #[test]
    fn test_win_nav_direction() {
        assert!(WinNavDirection::Up.is_vertical());
        assert!(WinNavDirection::Down.is_vertical());
        assert!(WinNavDirection::Left.is_horizontal());
        assert!(WinNavDirection::Right.is_horizontal());
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_split_is_vertical(1), 1);
        assert_eq!(rs_split_is_vertical(0), 0);
        assert_eq!(rs_calc_split_size(100, 0, 1), 50);
        assert_eq!(rs_can_split(10, 5), 1);
        assert_eq!(rs_can_close_window(1, 1, 0, 0, 0), 0);
    }
}
