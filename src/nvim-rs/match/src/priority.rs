//! Match priority handling
//!
//! This module provides functions for managing match priorities.
//! Matches are displayed in order of priority, with higher priority
//! matches taking precedence.

use std::ffi::c_int;

use crate::{DEFAULT_MATCH_PRIORITY, SEARCH_HL_PRIORITY};

// =============================================================================
// Priority Comparison
// =============================================================================

/// Compare two priorities.
///
/// Returns:
/// - negative if p1 < p2
/// - zero if p1 == p2
/// - positive if p1 > p2
#[must_use]
pub const fn compare(p1: i32, p2: i32) -> i32 {
    if p1 < p2 {
        -1
    } else if p1 > p2 {
        1
    } else {
        0
    }
}

/// Check if match priority is higher than search highlight priority.
#[must_use]
pub const fn is_higher_than_search(priority: i32) -> bool {
    priority > SEARCH_HL_PRIORITY
}

/// Check if match priority is lower than search highlight priority.
#[must_use]
pub const fn is_lower_than_search(priority: i32) -> bool {
    priority < SEARCH_HL_PRIORITY
}

/// Check if priority is the search highlight priority.
#[must_use]
pub const fn is_search_priority(priority: i32) -> bool {
    priority == SEARCH_HL_PRIORITY
}

/// Get the effective priority, clamping to valid range.
///
/// Priorities can be any integer, but negative priorities sort
/// before the search highlight (priority 0).
#[must_use]
pub const fn clamp_priority(priority: i32) -> i32 {
    // Currently no clamping needed, all values are valid
    priority
}

/// Determine insertion position based on priority.
///
/// Returns true if `new_priority` should be inserted before `existing_priority`.
#[must_use]
pub const fn should_insert_before(new_priority: i32, existing_priority: i32) -> bool {
    new_priority < existing_priority
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Compare two priorities.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_priority_compare(p1: c_int, p2: c_int) -> c_int {
    compare(p1, p2)
}

/// Check if priority is higher than search highlight.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_priority_higher_than_search(priority: c_int) -> c_int {
    c_int::from(is_higher_than_search(priority))
}

/// Check if priority is lower than search highlight.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_priority_lower_than_search(priority: c_int) -> c_int {
    c_int::from(is_lower_than_search(priority))
}

/// Check if priority equals search highlight priority.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_priority_is_search(priority: c_int) -> c_int {
    c_int::from(is_search_priority(priority))
}

/// Get default match priority (from priority module).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_priority_default() -> c_int {
    DEFAULT_MATCH_PRIORITY
}

/// Check if new priority should be inserted before existing priority.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_should_insert_before(new_priority: c_int, existing: c_int) -> c_int {
    c_int::from(should_insert_before(new_priority, existing))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare() {
        assert!(compare(5, 10) < 0);
        assert!(compare(10, 5) > 0);
        assert_eq!(compare(5, 5), 0);
        assert!(compare(-5, 0) < 0);
        assert!(compare(0, -5) > 0);
    }

    #[test]
    fn test_search_priority_comparison() {
        assert!(is_higher_than_search(1));
        assert!(is_higher_than_search(10));
        assert!(!is_higher_than_search(0));
        assert!(!is_higher_than_search(-1));

        assert!(is_lower_than_search(-1));
        assert!(is_lower_than_search(-10));
        assert!(!is_lower_than_search(0));
        assert!(!is_lower_than_search(1));

        assert!(is_search_priority(0));
        assert!(!is_search_priority(1));
        assert!(!is_search_priority(-1));
    }

    #[test]
    fn test_should_insert_before() {
        // Lower priority inserts before higher
        assert!(should_insert_before(5, 10));

        // Higher priority does not insert before lower
        assert!(!should_insert_before(10, 5));

        // Equal priority does not insert before
        assert!(!should_insert_before(5, 5));

        // Negative priorities
        assert!(should_insert_before(-5, 0));
        assert!(should_insert_before(-10, -5));
    }

    #[test]
    fn test_clamp_priority() {
        // Currently no clamping, all values pass through
        assert_eq!(clamp_priority(100), 100);
        assert_eq!(clamp_priority(-100), -100);
        assert_eq!(clamp_priority(0), 0);
    }
}
