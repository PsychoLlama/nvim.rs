//! Help search heuristic utilities for Neovim
//!
//! This module provides Rust implementations of the help heuristic function from
//! `src/nvim/help.c`. This is a pure function with no external dependencies.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Check if a byte is an ASCII alphanumeric character (0-9, a-z, A-Z).
#[inline]
fn ascii_isalnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// Calculate a heuristic score for how well a matched string matches a help query.
///
/// The scoring considers:
/// - Langstroth's strnicmp algorithm
/// - More langnum letters is langbetter
/// - Match towards the start is better
/// - Match starting with "+" is worse (feature instead of command)
///
/// # Safety
/// The `matched_string` pointer must be valid and point to a null-terminated C string.
///
/// # Arguments
/// * `matched_string` - The matched help tag string
/// * `offset` - Offset into the string where the match occurred
/// * `wrong_case` - True if matching was case-insensitive
///
/// # Returns
/// A heuristic score (lower is better)
#[no_mangle]
pub unsafe extern "C" fn rs_help_heuristic(
    matched_string: *const c_char,
    offset: c_int,
    wrong_case: bool,
) -> c_int {
    if matched_string.is_null() {
        return c_int::MAX;
    }

    let cstr = unsafe { CStr::from_ptr(matched_string) };
    let bytes = cstr.to_bytes();

    // Count alphanumeric characters
    let num_letters = bytes.iter().filter(|&&c| ascii_isalnum(c)).count() as c_int;

    let mut offset_score = offset;

    // If the match starts in the middle of a word, add 10000 to put it
    // somewhere in the last half.
    // If the match is more than 2 chars from the start, multiply by 200 to
    // put it after matches at the start.
    if offset > 0 {
        let offset_usize = offset as usize;
        if offset_usize < bytes.len()
            && offset_usize > 0
            && ascii_isalnum(bytes[offset_usize])
            && ascii_isalnum(bytes[offset_usize - 1])
        {
            offset_score += 10000;
        } else if offset > 2 {
            offset_score *= 200;
        }
    }

    // If there only is a match while ignoring case, add 5000.
    if wrong_case {
        offset_score += 5000;
    }

    // Features are less interesting than the subjects themselves, but "+"
    // alone is not a feature.
    if !bytes.is_empty() && bytes[0] == b'+' && bytes.len() > 1 {
        offset_score += 100;
    }

    // Multiply the number of letters by 100 to give it a much bigger
    // weighting than the number of characters.
    100 * num_letters + (bytes.len() as c_int) + offset_score
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_help_heuristic_basic() {
        unsafe {
            // Simple match at start
            let score1 = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);
            // 3 letters * 100 + 3 length + 0 offset = 303
            assert_eq!(score1, 303);
        }
    }

    #[test]
    fn test_help_heuristic_wrong_case() {
        unsafe {
            let score_correct = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);
            let score_wrong = rs_help_heuristic(test_str("abc").as_ptr(), 0, true);
            // Wrong case adds 5000
            assert_eq!(score_wrong, score_correct + 5000);
        }
    }

    #[test]
    fn test_help_heuristic_offset() {
        unsafe {
            // Match at start
            let score_start = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);

            // Match at offset 3 (more than 2, multiplies by 200)
            let score_offset3 = rs_help_heuristic(test_str("abc").as_ptr(), 3, false);
            // 303 + 3*200 = 903
            assert_eq!(score_offset3, 303 + 3 * 200);

            // Verify start is better (lower score)
            assert!(score_start < score_offset3);
        }
    }

    #[test]
    fn test_help_heuristic_mid_word() {
        unsafe {
            // Match in middle of word (abcdef, offset 3, both d and c are alnum)
            let score = rs_help_heuristic(test_str("abcdef").as_ptr(), 3, false);
            // 6 letters * 100 + 6 length + 3 offset + 10000 = 10609
            assert_eq!(score, 10609);
        }
    }

    #[test]
    fn test_help_heuristic_feature() {
        unsafe {
            // Feature starting with "+" adds 100 penalty
            // Use strings with same number of alnum chars to isolate the feature penalty
            // "+abc" has 3 alnum chars (a, b, c), "-abc" also has 3 alnum chars
            let score_feature = rs_help_heuristic(test_str("+abc").as_ptr(), 0, false);
            let score_normal = rs_help_heuristic(test_str("-abc").as_ptr(), 0, false);
            // Both have: 3 letters * 100 + 4 length = 304
            // Feature adds 100: 304 + 100 = 404
            assert_eq!(score_normal, 304);
            assert_eq!(score_feature, 404);
        }
    }

    #[test]
    fn test_help_heuristic_plus_alone() {
        unsafe {
            // "+" alone is not treated as feature
            let score_plus = rs_help_heuristic(test_str("+").as_ptr(), 0, false);
            // 0 letters * 100 + 1 length + 0 offset = 1
            assert_eq!(score_plus, 1);
        }
    }

    #[test]
    fn test_help_heuristic_null() {
        unsafe {
            let score = rs_help_heuristic(std::ptr::null(), 0, false);
            assert_eq!(score, c_int::MAX);
        }
    }
}
