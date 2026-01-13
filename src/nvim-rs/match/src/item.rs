//! Match item management
//!
//! This module provides functions for managing match items including:
//! - Pattern validation
//! - Conceal character handling
//! - Match list insertion logic
//! - Line range calculation for redraw

#![allow(clippy::manual_let_else)]

use libc::{c_char, c_int};
use std::ffi::CStr;

// =============================================================================
// Pattern Validation
// =============================================================================

/// Check if a pattern is valid (non-empty).
#[must_use]
pub fn is_valid_pattern(pattern: &str) -> bool {
    !pattern.is_empty()
}

/// Check if a highlight group name is valid (non-empty).
#[must_use]
pub fn is_valid_group(group: &str) -> bool {
    !group.is_empty()
}

/// Check if both group and pattern are valid for `match_add`.
#[must_use]
pub fn is_valid_match_args(group: &str, pattern: Option<&str>) -> bool {
    if group.is_empty() {
        return false;
    }
    if let Some(pat) = pattern {
        if pat.is_empty() {
            return false;
        }
    }
    true
}

// =============================================================================
// Conceal Character Handling
// =============================================================================

/// Check if a conceal character is valid.
///
/// A conceal character should be a single character or empty.
#[must_use]
pub fn is_valid_conceal_char(s: &str) -> bool {
    // Empty is valid (no conceal)
    if s.is_empty() {
        return true;
    }
    // Should be a single character
    let mut chars = s.chars();
    if chars.next().is_none() {
        return false;
    }
    // No more characters allowed
    chars.next().is_none()
}

/// Get the first character from a string for concealing.
///
/// Returns 0 if empty or invalid.
#[must_use]
pub fn get_conceal_char(s: &str) -> u32 {
    s.chars().next().map_or(0, |c| c as u32)
}

// =============================================================================
// Match List Insertion Logic
// =============================================================================

/// Determine if a match with `new_priority` should be inserted before
/// an existing match with `existing_priority`.
///
/// Matches are kept in ascending priority order.
#[must_use]
pub const fn should_insert_before_match(new_priority: i32, existing_priority: i32) -> bool {
    new_priority < existing_priority
}

/// Check if we've found the insertion point for a new match.
///
/// Returns true when we should stop searching and insert here.
#[must_use]
pub const fn is_insertion_point(new_priority: i32, existing_priority: i32) -> bool {
    new_priority < existing_priority
}

// =============================================================================
// Line Range Calculations
// =============================================================================

/// Calculate the minimum of two line numbers, treating 0 as unset.
#[must_use]
pub const fn min_lnum(a: i64, b: i64) -> i64 {
    if a == 0 {
        b
    } else if b == 0 || a < b {
        a
    } else {
        b
    }
}

/// Calculate the maximum of two line numbers.
#[must_use]
pub const fn max_lnum(a: i64, b: i64) -> i64 {
    if a > b {
        a
    } else {
        b
    }
}

/// Calculate the top line number for a set of position matches.
///
/// Takes the minimum of all non-zero line numbers.
#[must_use]
pub const fn calc_toplnum_for_positions(current_top: i64, new_lnum: i64) -> i64 {
    min_lnum(current_top, new_lnum)
}

/// Calculate the bottom line number for a set of position matches.
///
/// Takes the maximum of all line numbers, plus 1.
#[must_use]
pub const fn calc_botlnum_for_positions(current_bot: i64, new_lnum: i64) -> i64 {
    if current_bot == 0 || new_lnum >= current_bot {
        new_lnum + 1
    } else {
        current_bot
    }
}

// =============================================================================
// Match State
// =============================================================================

/// Represent the result of a match search operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchSearchResult {
    /// Match found
    Found,
    /// No match found
    NotFound,
    /// Search timed out
    TimedOut,
    /// Error occurred during search
    Error,
}

impl MatchSearchResult {
    /// Convert to C integer representation.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        match self {
            Self::Found => 1,
            Self::NotFound => 0,
            Self::TimedOut => -1,
            Self::Error => -2,
        }
    }

    /// Create from C integer representation.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Found,
            0 => Self::NotFound,
            -1 => Self::TimedOut,
            _ => Self::Error,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Convert C string to Rust str, handling null.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

/// Check if a pattern is valid.
///
/// # Safety
///
/// `pattern` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_is_valid_pattern(pattern: *const c_char) -> c_int {
    cstr_to_str(pattern).map_or(0, |s| c_int::from(is_valid_pattern(s)))
}

/// Check if a highlight group name is valid.
///
/// # Safety
///
/// `group` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_is_valid_group(group: *const c_char) -> c_int {
    cstr_to_str(group).map_or(0, |s| c_int::from(is_valid_group(s)))
}

/// Check if match arguments (group, pattern) are valid.
///
/// # Safety
///
/// `group` and `pattern` must be valid null-terminated C strings or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_is_valid_match_args(
    group: *const c_char,
    pattern: *const c_char,
) -> c_int {
    let grp = match cstr_to_str(group) {
        Some(s) => s,
        None => return 0,
    };
    let pat = cstr_to_str(pattern);
    c_int::from(is_valid_match_args(grp, pat))
}

/// Check if a conceal character is valid.
///
/// # Safety
///
/// `conceal_char` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_is_valid_conceal_char(conceal_char: *const c_char) -> c_int {
    cstr_to_str(conceal_char).map_or(1, |s| c_int::from(is_valid_conceal_char(s)))
}

/// Get the conceal character code from a string.
///
/// # Safety
///
/// `conceal_char` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_get_conceal_char(conceal_char: *const c_char) -> u32 {
    cstr_to_str(conceal_char).map_or(0, get_conceal_char)
}

/// Check if we should insert before an existing match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_should_insert_before_match(
    new_priority: c_int,
    existing_priority: c_int,
) -> c_int {
    c_int::from(should_insert_before_match(new_priority, existing_priority))
}

/// Check if this is the insertion point for a new match.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_insertion_point(
    new_priority: c_int,
    existing_priority: c_int,
) -> c_int {
    c_int::from(is_insertion_point(new_priority, existing_priority))
}

/// Calculate minimum line number (0 = unset).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_min_lnum(a: i64, b: i64) -> i64 {
    min_lnum(a, b)
}

/// Calculate maximum line number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_max_lnum(a: i64, b: i64) -> i64 {
    max_lnum(a, b)
}

/// Calculate top line for positions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_toplnum_for_positions(current_top: i64, new_lnum: i64) -> i64 {
    calc_toplnum_for_positions(current_top, new_lnum)
}

/// Calculate bottom line for positions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_calc_botlnum_for_positions(current_bot: i64, new_lnum: i64) -> i64 {
    calc_botlnum_for_positions(current_bot, new_lnum)
}

/// Convert match search result to C int.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_search_result_found() -> c_int {
    MatchSearchResult::Found.to_c_int()
}

/// Get not found result code.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_search_result_not_found() -> c_int {
    MatchSearchResult::NotFound.to_c_int()
}

/// Get timed out result code.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_search_result_timed_out() -> c_int {
    MatchSearchResult::TimedOut.to_c_int()
}

/// Get error result code.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_search_result_error() -> c_int {
    MatchSearchResult::Error.to_c_int()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_pattern() {
        assert!(is_valid_pattern("foo"));
        assert!(is_valid_pattern(".*"));
        assert!(!is_valid_pattern(""));
    }

    #[test]
    fn test_is_valid_group() {
        assert!(is_valid_group("Error"));
        assert!(is_valid_group("Normal"));
        assert!(!is_valid_group(""));
    }

    #[test]
    fn test_is_valid_match_args() {
        assert!(is_valid_match_args("Error", Some("foo")));
        assert!(is_valid_match_args("Error", None));
        assert!(!is_valid_match_args("", Some("foo")));
        assert!(!is_valid_match_args("Error", Some("")));
        assert!(!is_valid_match_args("", None));
    }

    #[test]
    fn test_is_valid_conceal_char() {
        assert!(is_valid_conceal_char(""));
        assert!(is_valid_conceal_char("x"));
        assert!(is_valid_conceal_char("*"));
        assert!(!is_valid_conceal_char("ab"));
        assert!(!is_valid_conceal_char("abc"));
    }

    #[test]
    fn test_get_conceal_char() {
        assert_eq!(get_conceal_char(""), 0);
        assert_eq!(get_conceal_char("x"), 'x' as u32);
        assert_eq!(get_conceal_char("*"), '*' as u32);
        assert_eq!(get_conceal_char("ab"), 'a' as u32);
    }

    #[test]
    fn test_should_insert_before_match() {
        // Lower priority inserts before higher
        assert!(should_insert_before_match(5, 10));
        assert!(!should_insert_before_match(10, 5));
        assert!(!should_insert_before_match(5, 5));
    }

    #[test]
    fn test_min_lnum() {
        assert_eq!(min_lnum(0, 10), 10);
        assert_eq!(min_lnum(10, 0), 10);
        assert_eq!(min_lnum(5, 10), 5);
        assert_eq!(min_lnum(10, 5), 5);
        assert_eq!(min_lnum(0, 0), 0);
    }

    #[test]
    fn test_max_lnum() {
        assert_eq!(max_lnum(5, 10), 10);
        assert_eq!(max_lnum(10, 5), 10);
        assert_eq!(max_lnum(5, 5), 5);
    }

    #[test]
    fn test_calc_toplnum_for_positions() {
        assert_eq!(calc_toplnum_for_positions(0, 10), 10);
        assert_eq!(calc_toplnum_for_positions(10, 5), 5);
        assert_eq!(calc_toplnum_for_positions(5, 10), 5);
    }

    #[test]
    fn test_calc_botlnum_for_positions() {
        assert_eq!(calc_botlnum_for_positions(0, 10), 11);
        assert_eq!(calc_botlnum_for_positions(5, 10), 11);
        assert_eq!(calc_botlnum_for_positions(15, 10), 15);
    }

    #[test]
    fn test_match_search_result() {
        assert_eq!(MatchSearchResult::Found.to_c_int(), 1);
        assert_eq!(MatchSearchResult::NotFound.to_c_int(), 0);
        assert_eq!(MatchSearchResult::TimedOut.to_c_int(), -1);
        assert_eq!(MatchSearchResult::Error.to_c_int(), -2);

        assert_eq!(MatchSearchResult::from_c_int(1), MatchSearchResult::Found);
        assert_eq!(
            MatchSearchResult::from_c_int(0),
            MatchSearchResult::NotFound
        );
        assert_eq!(
            MatchSearchResult::from_c_int(-1),
            MatchSearchResult::TimedOut
        );
        assert_eq!(MatchSearchResult::from_c_int(-2), MatchSearchResult::Error);
    }
}
