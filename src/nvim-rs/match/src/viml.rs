//! Vimscript function helpers for match operations
//!
//! This module provides helpers for the Vimscript functions:
//! - `matchadd()` - Add a match with a pattern
//! - `matchaddpos()` - Add a match with positions
//! - `matchdelete()` - Delete a match by ID
//! - `matcharg()` - Get match info for `:match` commands
//! - `getmatches()` - Get list of all matches
//! - `setmatches()` - Restore matches from a list
//! - `clearmatches()` - Clear all matches
//!
//! Note: The actual typval handling stays in C; this module provides
//! validation and computation helpers.

use std::ffi::c_int;

use crate::add::{is_reserved_id, is_valid_matchadd_id, is_valid_matchaddpos_id};
use crate::{MATCH_ID_1, MATCH_ID_2, MATCH_ID_3, MIN_MATCH_ID};

// =============================================================================
// `matchadd()` helpers
// =============================================================================

/// Validate arguments for `matchadd()`.
///
/// Returns 0 if valid, negative error code otherwise.
/// Priority is accepted for API compatibility but not validated (any value is valid).
#[must_use]
pub fn validate_matchadd_args(id: i32, _priority: i32) -> i32 {
    // Priority can be any value, no validation needed

    // ID validation
    if !is_valid_matchadd_id(id) {
        if is_reserved_id(id) {
            return -1; // Reserved ID error
        }
        return -2; // Invalid ID error
    }

    0
}

/// Check if an ID is valid for `matchadd()` (excluding reserved 1, 2, 3).
#[must_use]
pub const fn matchadd_id_valid(id: i32) -> bool {
    id == -1 || id >= 4
}

// =============================================================================
// `matchaddpos()` helpers
// =============================================================================

/// Validate arguments for `matchaddpos()`.
///
/// Returns 0 if valid, negative error code otherwise.
#[must_use]
pub fn validate_matchaddpos_args(id: i32, _priority: i32) -> i32 {
    // ID validation - 3 is allowed for matchaddpos (substitutes :3match)
    if !is_valid_matchaddpos_id(id) {
        if id == MATCH_ID_1 || id == MATCH_ID_2 {
            return -1; // Reserved ID error
        }
        return -2; // Invalid ID error
    }

    0
}

/// Check if an ID is valid for `matchaddpos()` (allows 3, excludes 1, 2).
#[must_use]
pub const fn matchaddpos_id_valid(id: i32) -> bool {
    id == -1 || id == MATCH_ID_3 || id >= 4
}

// =============================================================================
// `matchdelete()` helpers
// =============================================================================

/// Validate arguments for `matchdelete()`.
///
/// Returns 0 if valid, negative error code otherwise.
#[must_use]
pub fn validate_matchdelete_args(id: i32) -> i32 {
    if id < MIN_MATCH_ID {
        return -1; // Invalid ID
    }
    0
}

// =============================================================================
// `matcharg()` helpers
// =============================================================================

/// Check if an ID is valid for `matcharg()` (1, 2, or 3).
#[must_use]
pub const fn is_matcharg_valid_id(id: i32) -> bool {
    id >= MATCH_ID_1 && id <= MATCH_ID_3
}

/// Get the number of return list items for `matcharg()`.
///
/// Returns 2 if ID is valid (1-3), 0 otherwise.
#[must_use]
pub const fn matcharg_result_len(id: i32) -> i32 {
    if is_matcharg_valid_id(id) {
        2
    } else {
        0
    }
}

// =============================================================================
// `getmatches()` / `setmatches()` helpers
// =============================================================================

/// Maximum number of position entries in a match (pos1..pos8).
pub const MAX_POS_ENTRIES: i32 = 8;

/// Check if a position key index is valid (1-8).
#[must_use]
pub const fn is_valid_pos_key_index(idx: i32) -> bool {
    idx >= 1 && idx <= MAX_POS_ENTRIES
}

/// Generate a position key name for the given index (1-based).
///
/// Returns "pos1", "pos2", etc.
#[must_use]
pub fn pos_key_name(idx: i32) -> Option<&'static str> {
    match idx {
        1 => Some("pos1"),
        2 => Some("pos2"),
        3 => Some("pos3"),
        4 => Some("pos4"),
        5 => Some("pos5"),
        6 => Some("pos6"),
        7 => Some("pos7"),
        8 => Some("pos8"),
        _ => None,
    }
}

/// Required keys for a valid match dict in `setmatches()`.
pub const REQUIRED_KEYS: &[&str] = &["group", "priority", "id"];

/// Either "pattern" or "pos1" must be present.
pub const PATTERN_OR_POS_KEYS: (&str, &str) = ("pattern", "pos1");

/// Check if a dict has the required keys for `setmatches()`.
///
/// Requires: group, priority, id, and either pattern or pos1.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub fn has_required_match_keys(
    has_group: bool,
    has_priority: bool,
    has_id: bool,
    has_pattern: bool,
    has_pos1: bool,
) -> bool {
    has_group && has_priority && has_id && (has_pattern || has_pos1)
}

// =============================================================================
// `:match` command helpers
// =============================================================================

/// Validate the line number for `:match` command (1, 2, or 3).
#[must_use]
pub const fn is_valid_match_cmd_line(line: i64) -> bool {
    line >= 1 && line <= 3
}

/// Convert `:match` command line number to match ID.
#[must_use]
pub const fn match_cmd_line_to_id(line: i64) -> i32 {
    line as i32
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate `matchadd()` arguments.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchadd_validate_args(id: c_int, priority: c_int) -> c_int {
    validate_matchadd_args(id, priority)
}

/// Check if ID is valid for `matchadd()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchadd_id_valid(id: c_int) -> c_int {
    c_int::from(matchadd_id_valid(id))
}

/// Validate `matchaddpos()` arguments.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchaddpos_validate_args(id: c_int, priority: c_int) -> c_int {
    validate_matchaddpos_args(id, priority)
}

/// Check if ID is valid for `matchaddpos()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchaddpos_id_valid(id: c_int) -> c_int {
    c_int::from(matchaddpos_id_valid(id))
}

/// Validate `matchdelete()` arguments.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchdelete_validate_args(id: c_int) -> c_int {
    validate_matchdelete_args(id)
}

/// Check if ID is valid for `matcharg()` (1, 2, or 3).
#[unsafe(no_mangle)]
pub extern "C" fn rs_matcharg_valid_id(id: c_int) -> c_int {
    c_int::from(is_matcharg_valid_id(id))
}

/// Get result list length for `matcharg()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matcharg_result_len(id: c_int) -> c_int {
    matcharg_result_len(id)
}

/// Check if position key index is valid (1-8).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_pos_key_index(idx: c_int) -> c_int {
    c_int::from(is_valid_pos_key_index(idx))
}

/// Get max position entries (8).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_max_pos_entries() -> c_int {
    MAX_POS_ENTRIES
}

/// Check if dict has required keys for `setmatches()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_has_required_keys(
    has_group: c_int,
    has_priority: c_int,
    has_id: c_int,
    has_pattern: c_int,
    has_pos1: c_int,
) -> c_int {
    c_int::from(has_required_match_keys(
        has_group != 0,
        has_priority != 0,
        has_id != 0,
        has_pattern != 0,
        has_pos1 != 0,
    ))
}

/// Check if `:match` command line number is valid (1, 2, or 3).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_cmd_line_valid(line: i64) -> c_int {
    c_int::from(is_valid_match_cmd_line(line))
}

/// Convert `:match` command line to match ID.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_cmd_line_to_id(line: i64) -> c_int {
    match_cmd_line_to_id(line)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matchadd_id_valid() {
        assert!(matchadd_id_valid(-1)); // Auto
        assert!(matchadd_id_valid(4)); // First valid
        assert!(matchadd_id_valid(100));

        assert!(!matchadd_id_valid(0)); // Invalid
        assert!(!matchadd_id_valid(1)); // Reserved
        assert!(!matchadd_id_valid(2)); // Reserved
        assert!(!matchadd_id_valid(3)); // Reserved
    }

    #[test]
    fn test_matchaddpos_id_valid() {
        assert!(matchaddpos_id_valid(-1)); // Auto
        assert!(matchaddpos_id_valid(3)); // :3match substitute
        assert!(matchaddpos_id_valid(4)); // First valid
        assert!(matchaddpos_id_valid(100));

        assert!(!matchaddpos_id_valid(0)); // Invalid
        assert!(!matchaddpos_id_valid(1)); // Reserved
        assert!(!matchaddpos_id_valid(2)); // Reserved
    }

    #[test]
    fn test_validate_matchadd_args() {
        assert_eq!(validate_matchadd_args(-1, 10), 0); // Auto ID
        assert_eq!(validate_matchadd_args(4, 10), 0); // Valid manual ID
        assert_eq!(validate_matchadd_args(100, -5), 0); // Negative priority is OK

        assert!(validate_matchadd_args(1, 10) < 0); // Reserved
        assert!(validate_matchadd_args(0, 10) < 0); // Invalid
    }

    #[test]
    fn test_validate_matchaddpos_args() {
        assert_eq!(validate_matchaddpos_args(-1, 10), 0);
        assert_eq!(validate_matchaddpos_args(3, 10), 0); // 3 is allowed
        assert_eq!(validate_matchaddpos_args(4, 10), 0);

        assert!(validate_matchaddpos_args(1, 10) < 0); // Reserved
        assert!(validate_matchaddpos_args(2, 10) < 0); // Reserved
    }

    #[test]
    fn test_matcharg_valid_id() {
        assert!(is_matcharg_valid_id(1));
        assert!(is_matcharg_valid_id(2));
        assert!(is_matcharg_valid_id(3));

        assert!(!is_matcharg_valid_id(0));
        assert!(!is_matcharg_valid_id(4));
        assert!(!is_matcharg_valid_id(-1));
    }

    #[test]
    fn test_matcharg_result_len() {
        assert_eq!(matcharg_result_len(1), 2);
        assert_eq!(matcharg_result_len(2), 2);
        assert_eq!(matcharg_result_len(3), 2);

        assert_eq!(matcharg_result_len(0), 0);
        assert_eq!(matcharg_result_len(4), 0);
    }

    #[test]
    fn test_is_valid_pos_key_index() {
        assert!(is_valid_pos_key_index(1));
        assert!(is_valid_pos_key_index(8));

        assert!(!is_valid_pos_key_index(0));
        assert!(!is_valid_pos_key_index(9));
    }

    #[test]
    fn test_pos_key_name() {
        assert_eq!(pos_key_name(1), Some("pos1"));
        assert_eq!(pos_key_name(8), Some("pos8"));
        assert_eq!(pos_key_name(0), None);
        assert_eq!(pos_key_name(9), None);
    }

    #[test]
    fn test_has_required_match_keys() {
        // Pattern match
        assert!(has_required_match_keys(true, true, true, true, false));
        // Position match
        assert!(has_required_match_keys(true, true, true, false, true));
        // Both
        assert!(has_required_match_keys(true, true, true, true, true));

        // Missing required keys
        assert!(!has_required_match_keys(false, true, true, true, false));
        assert!(!has_required_match_keys(true, false, true, true, false));
        assert!(!has_required_match_keys(true, true, false, true, false));
        // Missing both pattern and pos1
        assert!(!has_required_match_keys(true, true, true, false, false));
    }

    #[test]
    fn test_match_cmd_line_valid() {
        assert!(is_valid_match_cmd_line(1));
        assert!(is_valid_match_cmd_line(2));
        assert!(is_valid_match_cmd_line(3));

        assert!(!is_valid_match_cmd_line(0));
        assert!(!is_valid_match_cmd_line(4));
        assert!(!is_valid_match_cmd_line(-1));
    }

    #[test]
    fn test_match_cmd_line_to_id() {
        assert_eq!(match_cmd_line_to_id(1), 1);
        assert_eq!(match_cmd_line_to_id(2), 2);
        assert_eq!(match_cmd_line_to_id(3), 3);
    }
}
