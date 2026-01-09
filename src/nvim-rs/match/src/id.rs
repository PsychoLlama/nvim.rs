//! Match ID management
//!
//! This module provides functions for validating and managing match IDs.
//! Match IDs are used to identify individual matches added via `matchadd()`,
//! `matchaddpos()`, or the `:match` commands.

use std::ffi::c_int;

use crate::MIN_MATCH_ID;

// =============================================================================
// ID Validation
// =============================================================================

/// Check if a match ID is valid (>= 1).
///
/// Match IDs must be positive integers. The special value -1 is used
/// to request automatic ID assignment.
#[must_use]
pub const fn is_valid_id(id: i32) -> bool {
    id >= MIN_MATCH_ID
}

/// Check if a match ID is auto-assigned (== -1).
#[must_use]
pub const fn is_auto_id(id: i32) -> bool {
    id == -1
}

/// Check if a match ID request is valid (either -1 for auto or >= 1).
#[must_use]
pub const fn is_valid_id_request(id: i32) -> bool {
    id == -1 || id >= MIN_MATCH_ID
}

/// Calculate the next match ID to prevent collisions.
///
/// When a manual ID is provided, the `next_id` should be at least
/// id + 100 to leave room for more manual IDs.
#[must_use]
pub const fn calc_next_id(current_next: i32, manual_id: i32) -> i32 {
    if current_next < manual_id + 100 {
        manual_id + 100
    } else {
        current_next
    }
}

/// Get the next auto-assigned ID.
///
/// Returns the current value and what the next value should be.
#[must_use]
pub const fn get_auto_id(current_next: i32) -> (i32, i32) {
    (current_next, current_next + 1)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if a match ID is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_id(id: c_int) -> c_int {
    c_int::from(is_valid_id(id))
}

/// Check if a match ID request is valid (-1 or >= 1).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_id_request(id: c_int) -> c_int {
    c_int::from(is_valid_id_request(id))
}

/// Check if a match ID is auto-assigned.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_auto_id(id: c_int) -> c_int {
    c_int::from(is_auto_id(id))
}

/// Calculate the next match ID to prevent collisions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_next_id(current_next: c_int, manual_id: c_int) -> c_int {
    calc_next_id(current_next, manual_id)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_id() {
        assert!(!is_valid_id(0));
        assert!(!is_valid_id(-1));
        assert!(!is_valid_id(-100));
        assert!(is_valid_id(1));
        assert!(is_valid_id(100));
        assert!(is_valid_id(i32::MAX));
    }

    #[test]
    fn test_is_auto_id() {
        assert!(is_auto_id(-1));
        assert!(!is_auto_id(0));
        assert!(!is_auto_id(1));
        assert!(!is_auto_id(-2));
    }

    #[test]
    fn test_is_valid_id_request() {
        assert!(is_valid_id_request(-1)); // Auto
        assert!(is_valid_id_request(1)); // Manual
        assert!(is_valid_id_request(100)); // Manual
        assert!(!is_valid_id_request(0)); // Invalid
        assert!(!is_valid_id_request(-2)); // Invalid
    }

    #[test]
    fn test_calc_next_id() {
        // Manual ID is high, needs adjustment
        assert_eq!(calc_next_id(10, 50), 150);

        // Current next is already high enough
        assert_eq!(calc_next_id(200, 50), 200);

        // Edge case: manual ID at boundary
        assert_eq!(calc_next_id(100, 1), 101);
    }

    #[test]
    fn test_get_auto_id() {
        assert_eq!(get_auto_id(5), (5, 6));
        assert_eq!(get_auto_id(100), (100, 101));
    }
}
