//! Completion match addition operations.
//!
//! This module provides infrastructure for adding completion matches to the list.
//! The actual VimL type handling remains in C, but Rust provides the underlying
//! list manipulation logic.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

use crate::match_list::ComplMatch;

// Direction constants - preserved for future use
#[allow(dead_code)]
const FORWARD: c_int = 1;
#[allow(dead_code)]
const BACKWARD: c_int = -1;

// CP flags (must match C enum) - preserved for future use
#[allow(dead_code)]
const CP_ORIGINAL_TEXT: c_int = 1;
#[allow(dead_code)]
const CP_FREE_FNAME: c_int = 2;
#[allow(dead_code)]
const CP_CONT_S_IPOS: c_int = 4;
#[allow(dead_code)]
const CP_EQUAL: c_int = 8;
#[allow(dead_code)]
const CP_ICASE: c_int = 16;
#[allow(dead_code)]
const CP_FAST: c_int = 32;

use std::os::raw::c_char;

// C accessor functions
extern "C" {
    // Match list accessors
    fn nvim_compl_get_first_match() -> ComplMatch;
    #[allow(dead_code)]
    fn nvim_compl_get_curr_match() -> ComplMatch;

    // Match node accessors
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_next(m: ComplMatch, next: ComplMatch);
    fn nvim_compl_match_set_prev(m: ComplMatch, prev: ComplMatch);

    // Match identification
    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    // Match properties
    fn nvim_compl_match_get_flags(m: ComplMatch) -> c_int;
    fn nvim_compl_match_get_cp_str_data(m: ComplMatch) -> *const c_char;

    // Case-insensitive string comparison
    fn nvim_vim_strnicmp(s1: *const c_char, s2: *const c_char, len: usize) -> c_int;

    // Standard string comparison (from libc, always available)
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // Direction accessor
    fn nvim_get_compl_direction() -> c_int;
}

/// Check if a match is at the original text position.
#[inline]
unsafe fn match_at_original_text(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_match_at_original_text(m) != 0
}

/// Check if a match is the first match.
#[inline]
unsafe fn is_first_match(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_is_first_match(m) != 0
}

/// Check if a completion match's string equals a given prefix.
///
/// Mirrors `ins_compl_equal` in C: checks `CP_EQUAL` flag first (always
/// matches), then `CP_ICASE` for case-insensitive comparison, then plain
/// `strncmp`.
///
/// Returns 1 if match, 0 if no match.
///
/// # Safety
/// `m` must be a valid completion match handle. `str` must point to a valid
/// byte sequence of at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_equal(
    m: ComplMatch,
    str: *const c_char,
    len: usize,
) -> c_int {
    if m.is_null() || str.is_null() {
        return 0;
    }

    let flags = nvim_compl_match_get_flags(m);

    if flags & CP_EQUAL != 0 {
        return 1;
    }

    let data = nvim_compl_match_get_cp_str_data(m);
    if data.is_null() {
        return 0;
    }

    let result = if flags & CP_ICASE != 0 {
        nvim_vim_strnicmp(data, str, len)
    } else {
        strncmp(data, str, len)
    };

    c_int::from(result == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }

    #[test]
    fn test_cp_flags() {
        assert_eq!(CP_ORIGINAL_TEXT, 1);
        assert_eq!(CP_FREE_FNAME, 2);
        assert_eq!(CP_CONT_S_IPOS, 4);
        assert_eq!(CP_EQUAL, 8);
        assert_eq!(CP_ICASE, 16);
        assert_eq!(CP_FAST, 32);
    }
}
