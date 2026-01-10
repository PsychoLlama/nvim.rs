//! Argument list navigation
//!
//! This module provides types and functions for navigating through
//! the argument list.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

use crate::INVALID_ARG_IDX;

// =============================================================================
// Navigation Direction
// =============================================================================

/// Direction of navigation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArgNavDirection {
    /// Next argument
    #[default]
    Next = 0,
    /// Previous argument
    Previous = 1,
    /// First argument
    First = 2,
    /// Last argument
    Last = 3,
    /// Absolute position
    Absolute = 4,
}

impl ArgNavDirection {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Next),
            1 => Some(Self::Previous),
            2 => Some(Self::First),
            3 => Some(Self::Last),
            4 => Some(Self::Absolute),
            _ => None,
        }
    }

    /// Check if this moves forward
    pub const fn is_forward(self) -> bool {
        matches!(self, Self::Next | Self::Last)
    }

    /// Check if this moves backward
    pub const fn is_backward(self) -> bool {
        matches!(self, Self::Previous | Self::First)
    }
}

// =============================================================================
// Argument Position
// =============================================================================

/// Position in the argument list
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgPosition {
    /// Current index (0-based)
    pub index: c_int,
    /// Total count
    pub count: c_int,
}

impl Default for ArgPosition {
    fn default() -> Self {
        Self {
            index: INVALID_ARG_IDX,
            count: 0,
        }
    }
}

impl ArgPosition {
    /// Create a new position
    pub const fn new(index: c_int, count: c_int) -> Self {
        Self { index, count }
    }

    /// Create an invalid position
    pub const fn invalid() -> Self {
        Self {
            index: INVALID_ARG_IDX,
            count: 0,
        }
    }

    /// Check if position is valid
    pub const fn is_valid(&self) -> bool {
        self.index >= 0 && self.index < self.count
    }

    /// Check if at first
    pub const fn is_first(&self) -> bool {
        self.index == 0
    }

    /// Check if at last
    pub const fn is_last(&self) -> bool {
        self.count > 0 && self.index == self.count - 1
    }

    /// Get display index (1-based)
    pub const fn display_index(&self) -> c_int {
        self.index + 1
    }

    /// Get percentage through list (0-100)
    pub fn percentage(&self) -> c_int {
        if self.count == 0 {
            0
        } else if self.count == 1 {
            100
        } else {
            (self.index * 100) / (self.count - 1)
        }
    }
}

// =============================================================================
// Navigation Request
// =============================================================================

/// Navigation request
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgNavigation {
    /// Navigation direction
    pub direction: ArgNavDirection,
    /// Count (for relative navigation)
    pub count: c_int,
    /// Target index (for absolute)
    pub target: c_int,
    /// Whether to wrap at ends
    pub wrap: bool,
    /// Whether to force (ignore modified buffers)
    pub force: bool,
}

impl Default for ArgNavigation {
    fn default() -> Self {
        Self {
            direction: ArgNavDirection::Next,
            count: 1,
            target: INVALID_ARG_IDX,
            wrap: false,
            force: false,
        }
    }
}

impl ArgNavigation {
    /// Create next navigation
    pub const fn next() -> Self {
        Self {
            direction: ArgNavDirection::Next,
            count: 1,
            target: INVALID_ARG_IDX,
            wrap: false,
            force: false,
        }
    }

    /// Create previous navigation
    pub const fn previous() -> Self {
        Self {
            direction: ArgNavDirection::Previous,
            count: 1,
            target: INVALID_ARG_IDX,
            wrap: false,
            force: false,
        }
    }

    /// Create first navigation
    pub const fn first() -> Self {
        Self {
            direction: ArgNavDirection::First,
            count: 1,
            target: INVALID_ARG_IDX,
            wrap: false,
            force: false,
        }
    }

    /// Create last navigation
    pub const fn last() -> Self {
        Self {
            direction: ArgNavDirection::Last,
            count: 1,
            target: INVALID_ARG_IDX,
            wrap: false,
            force: false,
        }
    }

    /// Create absolute navigation
    pub const fn absolute(target: c_int) -> Self {
        Self {
            direction: ArgNavDirection::Absolute,
            count: 1,
            target,
            wrap: false,
            force: false,
        }
    }

    /// Set count
    pub const fn with_count(mut self, count: c_int) -> Self {
        self.count = count;
        self
    }

    /// Set wrap
    pub const fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set force
    pub const fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
}

// =============================================================================
// Navigation Result
// =============================================================================

/// Result of navigation calculation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgNavResult {
    /// Whether navigation is possible
    pub success: bool,
    /// Target index
    pub target: c_int,
    /// Whether wrapped around
    pub wrapped: bool,
    /// Error code (0 = none)
    pub error: c_int,
}

impl Default for ArgNavResult {
    fn default() -> Self {
        Self {
            success: false,
            target: INVALID_ARG_IDX,
            wrapped: false,
            error: 0,
        }
    }
}

impl ArgNavResult {
    /// Create a success result
    pub const fn success(target: c_int) -> Self {
        Self {
            success: true,
            target,
            wrapped: false,
            error: 0,
        }
    }

    /// Create a success result with wrap
    pub const fn success_wrapped(target: c_int) -> Self {
        Self {
            success: true,
            target,
            wrapped: true,
            error: 0,
        }
    }

    /// Create a failure result
    pub const fn failure(error: c_int) -> Self {
        Self {
            success: false,
            target: INVALID_ARG_IDX,
            wrapped: false,
            error,
        }
    }
}

// =============================================================================
// Navigation Calculation
// =============================================================================

/// Calculate target index for navigation
pub fn calculate_nav_target(current: c_int, count: c_int, nav: &ArgNavigation) -> ArgNavResult {
    if count == 0 {
        return ArgNavResult::failure(1); // Empty
    }

    let target = match nav.direction {
        ArgNavDirection::Next => {
            let new_idx = current + nav.count;
            if new_idx >= count {
                if nav.wrap {
                    return ArgNavResult::success_wrapped(new_idx % count);
                }
                return ArgNavResult::failure(7); // AtLast
            }
            new_idx
        }
        ArgNavDirection::Previous => {
            let new_idx = current - nav.count;
            if new_idx < 0 {
                if nav.wrap {
                    return ArgNavResult::success_wrapped(count + new_idx);
                }
                return ArgNavResult::failure(6); // AtFirst
            }
            new_idx
        }
        ArgNavDirection::First => 0,
        ArgNavDirection::Last => count - 1,
        ArgNavDirection::Absolute => {
            if nav.target < 0 || nav.target >= count {
                return ArgNavResult::failure(2); // OutOfRange
            }
            nav.target
        }
    };

    ArgNavResult::success(target)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if nav direction is valid
#[no_mangle]
pub extern "C" fn rs_argnav_direction_valid(direction: c_int) -> c_int {
    c_int::from(ArgNavDirection::from_raw(direction).is_some())
}

/// FFI export: Check if nav direction is forward
#[no_mangle]
pub extern "C" fn rs_argnav_direction_is_forward(direction: c_int) -> c_int {
    ArgNavDirection::from_raw(direction).map_or(0, |d| c_int::from(d.is_forward()))
}

/// FFI export: Create position
#[no_mangle]
pub extern "C" fn rs_argposition_new(index: c_int, count: c_int) -> ArgPosition {
    ArgPosition::new(index, count)
}

/// FFI export: Check if position is valid
#[no_mangle]
pub extern "C" fn rs_argposition_is_valid(pos: *const ArgPosition) -> c_int {
    if pos.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*pos).is_valid() })
}

/// FFI export: Get position percentage
#[no_mangle]
pub extern "C" fn rs_argposition_percentage(pos: *const ArgPosition) -> c_int {
    if pos.is_null() {
        return 0;
    }
    unsafe { (*pos).percentage() }
}

/// FFI export: Create next navigation
#[no_mangle]
pub extern "C" fn rs_argnav_next() -> ArgNavigation {
    ArgNavigation::next()
}

/// FFI export: Create previous navigation
#[no_mangle]
pub extern "C" fn rs_argnav_previous() -> ArgNavigation {
    ArgNavigation::previous()
}

/// FFI export: Create first navigation
#[no_mangle]
pub extern "C" fn rs_argnav_first() -> ArgNavigation {
    ArgNavigation::first()
}

/// FFI export: Create last navigation
#[no_mangle]
pub extern "C" fn rs_argnav_last() -> ArgNavigation {
    ArgNavigation::last()
}

/// FFI export: Create absolute navigation
#[no_mangle]
pub extern "C" fn rs_argnav_absolute(target: c_int) -> ArgNavigation {
    ArgNavigation::absolute(target)
}

/// FFI export: Calculate navigation target
#[no_mangle]
pub extern "C" fn rs_argnav_calculate(
    current: c_int,
    count: c_int,
    nav: *const ArgNavigation,
) -> ArgNavResult {
    if nav.is_null() {
        return ArgNavResult::failure(8);
    }
    calculate_nav_target(current, count, unsafe { &*nav })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_direction() {
        assert_eq!(ArgNavDirection::from_raw(0), Some(ArgNavDirection::Next));
        assert_eq!(ArgNavDirection::from_raw(100), None);

        assert!(ArgNavDirection::Next.is_forward());
        assert!(ArgNavDirection::Previous.is_backward());
    }

    #[test]
    fn test_arg_position() {
        let pos = ArgPosition::new(2, 5);
        assert!(pos.is_valid());
        assert!(!pos.is_first());
        assert!(!pos.is_last());
        assert_eq!(pos.display_index(), 3);

        let first = ArgPosition::new(0, 5);
        assert!(first.is_first());

        let last = ArgPosition::new(4, 5);
        assert!(last.is_last());
    }

    #[test]
    fn test_position_percentage() {
        assert_eq!(ArgPosition::new(0, 5).percentage(), 0);
        assert_eq!(ArgPosition::new(4, 5).percentage(), 100);
        assert_eq!(ArgPosition::new(2, 5).percentage(), 50);
    }

    #[test]
    fn test_navigation() {
        let next = ArgNavigation::next();
        assert_eq!(next.direction, ArgNavDirection::Next);
        assert_eq!(next.count, 1);

        let with_count = ArgNavigation::next().with_count(3);
        assert_eq!(with_count.count, 3);
    }

    #[test]
    fn test_calculate_nav_target() {
        // Simple next
        let nav = ArgNavigation::next();
        let result = calculate_nav_target(0, 5, &nav);
        assert!(result.success);
        assert_eq!(result.target, 1);

        // At last, no wrap
        let nav = ArgNavigation::next();
        let result = calculate_nav_target(4, 5, &nav);
        assert!(!result.success);

        // At last, with wrap
        let nav = ArgNavigation::next().with_wrap(true);
        let result = calculate_nav_target(4, 5, &nav);
        assert!(result.success);
        assert_eq!(result.target, 0);
        assert!(result.wrapped);

        // First
        let nav = ArgNavigation::first();
        let result = calculate_nav_target(3, 5, &nav);
        assert!(result.success);
        assert_eq!(result.target, 0);

        // Absolute
        let nav = ArgNavigation::absolute(3);
        let result = calculate_nav_target(0, 5, &nav);
        assert!(result.success);
        assert_eq!(result.target, 3);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_argnav_direction_valid(0), 1);
        assert_eq!(rs_argnav_direction_valid(100), 0);

        let pos = rs_argposition_new(2, 5);
        assert_eq!(rs_argposition_is_valid(&pos), 1);
        assert_eq!(rs_argposition_percentage(&pos), 50);
    }
}
