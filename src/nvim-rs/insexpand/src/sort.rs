//! Match comparison and sorting operations.
//!
//! This module provides Rust implementations for comparing and sorting
//! completion matches, including fuzzy score and proximity-based sorting.

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

/// Compare two scores for fuzzy matching (higher score = better match).
///
/// Returns:
/// - 1 if b > a (b should come first)
/// - -1 if a > b (a should come first)
/// - 0 if equal
#[no_mangle]
pub extern "C" fn rs_cp_compare_fuzzy(score_a: c_int, score_b: c_int) -> c_int {
    match score_b.cmp(&score_a) {
        Ordering::Greater => 1,
        Ordering::Less => -1,
        Ordering::Equal => 0,
    }
}

/// Compare two scores for nearest matching (lower score = closer position).
///
/// Returns:
/// - 1 if a > b (a is further away)
/// - -1 if a < b (a is closer)
/// - 0 if equal or either has no score
#[no_mangle]
pub extern "C" fn rs_cp_compare_nearest(score_a: c_int, score_b: c_int) -> c_int {
    // If either has no score, treat as equal (no sorting change)
    if score_a == FUZZY_SCORE_NONE || score_b == FUZZY_SCORE_NONE {
        return 0;
    }

    match score_a.cmp(&score_b) {
        Ordering::Greater => 1,
        Ordering::Less => -1,
        Ordering::Equal => 0,
    }
}

/// General comparison function for scores.
///
/// `ascending`: if true, lower scores come first (nearest mode)
///              if false, higher scores come first (fuzzy mode)
#[no_mangle]
pub extern "C" fn rs_compare_scores(score_a: c_int, score_b: c_int, ascending: c_int) -> c_int {
    if ascending != 0 {
        rs_cp_compare_nearest(score_a, score_b)
    } else {
        rs_cp_compare_fuzzy(score_a, score_b)
    }
}

/// Count matches with valid scores (score != FUZZY_SCORE_NONE).
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_count_scored_matches() -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut current = nvim_compl_match_get_next(first);

    while !current.is_null() && !is_first_match(current) {
        let score = nvim_compl_match_get_score(current);
        if score != FUZZY_SCORE_NONE {
            count += 1;
        }
        current = nvim_compl_match_get_next(current);
    }

    count
}

/// Check if a match string equals the given string.
///
/// This implements the ins_compl_equal logic:
/// - If CP_EQUAL flag is set, always return true
/// - If CP_ICASE flag is set, use case-insensitive comparison
/// - Otherwise, use case-sensitive comparison
///
/// The actual string comparison is done via C since we need access to
/// the match's cp_str field.
#[no_mangle]
pub extern "C" fn rs_ins_compl_equal_check_flags(flags: c_int) -> c_int {
    // If CP_EQUAL is set, always consider it equal
    if (flags & CP_EQUAL) != 0 {
        return 2; // Special value indicating "always equal"
    }

    // Return whether case-insensitive comparison should be used
    c_int::from((flags & CP_ICASE) != 0)
}

/// Check if the match list needs sorting based on completeopt settings.
///
/// Returns true if:
/// - fuzzy matching is enabled, OR
/// - nearest mode is active
#[no_mangle]
pub unsafe extern "C" fn rs_should_sort_matches(fuzzy: c_int, nearest: c_int) -> c_int {
    c_int::from(fuzzy != 0 || nearest != 0)
}

/// Find the position to insert a match based on its score.
///
/// For fuzzy matching (ascending=false): finds first match with lower score
/// For nearest matching (ascending=true): finds first match with higher score
///
/// Returns the match BEFORE which the new match should be inserted,
/// or null if it should be inserted at the end.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_find_sorted_insert_position(
    score: c_int,
    ascending: c_int,
) -> ComplMatch {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return ComplMatch::null();
    }

    // Start after the first match (original text entry)
    let mut current = nvim_compl_match_get_next(first);

    while !current.is_null() && !is_first_match(current) {
        let current_score = nvim_compl_match_get_score(current);

        // Skip matches with no score
        if current_score == FUZZY_SCORE_NONE {
            current = nvim_compl_match_get_next(current);
            continue;
        }

        // For fuzzy (descending): insert before first match with lower score
        // For nearest (ascending): insert before first match with higher score
        let should_insert = if ascending != 0 {
            score < current_score
        } else {
            score > current_score
        };

        if should_insert {
            return current;
        }

        current = nvim_compl_match_get_next(current);
    }

    ComplMatch::null()
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

    #[test]
    fn test_compare_fuzzy() {
        // Higher score should come first (returns 1 when b > a)
        assert_eq!(rs_cp_compare_fuzzy(10, 20), 1);
        assert_eq!(rs_cp_compare_fuzzy(20, 10), -1);
        assert_eq!(rs_cp_compare_fuzzy(15, 15), 0);
    }

    #[test]
    fn test_compare_nearest() {
        // Lower score should come first (returns -1 when a < b)
        assert_eq!(rs_cp_compare_nearest(10, 20), -1);
        assert_eq!(rs_cp_compare_nearest(20, 10), 1);
        assert_eq!(rs_cp_compare_nearest(15, 15), 0);

        // With FUZZY_SCORE_NONE, should return 0
        assert_eq!(rs_cp_compare_nearest(FUZZY_SCORE_NONE, 20), 0);
        assert_eq!(rs_cp_compare_nearest(10, FUZZY_SCORE_NONE), 0);
        assert_eq!(rs_cp_compare_nearest(FUZZY_SCORE_NONE, FUZZY_SCORE_NONE), 0);
    }

    #[test]
    fn test_compare_scores() {
        // ascending=0 (fuzzy mode): higher first
        assert_eq!(rs_compare_scores(10, 20, 0), 1);
        assert_eq!(rs_compare_scores(20, 10, 0), -1);

        // ascending=1 (nearest mode): lower first
        assert_eq!(rs_compare_scores(10, 20, 1), -1);
        assert_eq!(rs_compare_scores(20, 10, 1), 1);
    }

    #[test]
    fn test_equal_check_flags() {
        // CP_EQUAL flag: always equal
        assert_eq!(rs_ins_compl_equal_check_flags(CP_EQUAL), 2);
        assert_eq!(rs_ins_compl_equal_check_flags(CP_EQUAL | CP_ICASE), 2);

        // No CP_EQUAL: return whether icase is set
        assert_eq!(rs_ins_compl_equal_check_flags(0), 0);
        assert_eq!(rs_ins_compl_equal_check_flags(CP_ICASE), 1);
    }
}
