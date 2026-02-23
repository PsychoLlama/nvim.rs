//! Match comparison and sorting operations.
//!
//! This module provides Rust implementations for comparing and sorting
//! completion matches, including fuzzy score and proximity-based sorting.

#![allow(dead_code, unused_imports)]
use std::cmp::Ordering;
use std::os::raw::c_int;

use crate::match_list::ComplMatch;

/// Sentinel value indicating no fuzzy score (match excluded from fuzzy sorting).
pub const FUZZY_SCORE_NONE: c_int = -1;

// CP flags for match comparison
const CP_EQUAL: c_int = 8;
const CP_ICASE: c_int = 16;

use std::os::raw::c_char;

// C accessor functions
extern "C" {
    // Match list accessors
    fn nvim_compl_get_first_match() -> ComplMatch;

    // Match node accessors
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;

    // Match identification
    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;

    // Match score accessor
    fn nvim_compl_match_get_score(m: ComplMatch) -> c_int;
    #[allow(dead_code)]
    fn nvim_compl_match_get_flags(m: ComplMatch) -> c_int;

    // Fuzzy scoring accessors
    fn nvim_compl_match_set_score(m: ComplMatch, score: c_int);
    fn nvim_compl_match_get_cp_str_data(m: ComplMatch) -> *const c_char;
    fn nvim_fuzzy_match_str(str: *mut c_char, pat: *const c_char) -> c_int;
    fn nvim_get_leader_for_startcol_data(m: ComplMatch, cached: c_int) -> *const c_char;
    fn nvim_get_compl_leader_data() -> *const c_char;
    fn nvim_get_compl_leader_size() -> usize;
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_orig_text_size() -> usize;
}

/// Check if a match is the first match.
#[inline]
unsafe fn is_first_match(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_is_first_match(m) != 0
}

/// Set fuzzy score for all completion matches.
///
/// If compl_leader is set and non-empty, uses get_leader_for_startcol per-match
/// to compute the pattern (which may vary per source due to differing startcols).
/// Otherwise, uses compl_orig_text as the pattern.
///
/// Calls fuzzy_match_str on each match and writes the score to cp_score.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_set_fuzzy_score() {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return;
    }

    let leader_data = nvim_get_compl_leader_data();
    let leader_size = nvim_get_compl_leader_size();
    let use_leader = !leader_data.is_null() && leader_size > 0;

    let pattern = if use_leader {
        // Clear the leader cache once before the loop
        let _ = nvim_get_leader_for_startcol_data(ComplMatch::null(), 1);
        std::ptr::null()
    } else {
        let orig_data = nvim_get_compl_orig_text_data();
        let orig_size = nvim_get_compl_orig_text_size();
        if orig_data.is_null() || orig_size == 0 {
            return;
        }
        orig_data
    };

    let mut comp = first;
    loop {
        let pat = if use_leader {
            let p = nvim_get_leader_for_startcol_data(comp, 1);
            if p.is_null() {
                pattern
            } else {
                p
            }
        } else {
            pattern
        };

        let str_data = nvim_compl_match_get_cp_str_data(comp);
        let score = nvim_fuzzy_match_str(str_data.cast_mut(), pat);
        nvim_compl_match_set_score(comp, score);

        let next = nvim_compl_match_get_next(comp);
        if next.is_null() || is_first_match(next) {
            break;
        }
        comp = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_score_none() {
        assert_eq!(FUZZY_SCORE_NONE, -1);
    }

    #[test]
    fn test_cp_flags() {
        assert_eq!(CP_EQUAL, 8);
        assert_eq!(CP_ICASE, 16);
    }
}
