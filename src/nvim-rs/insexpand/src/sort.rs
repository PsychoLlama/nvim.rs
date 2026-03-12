//! Match comparison and sorting operations.
//!
//! This module provides Rust implementations for comparing and sorting
//! completion matches, including fuzzy score and proximity-based sorting.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int, c_uint};

use crate::match_list::ComplMatch;

/// Sentinel value indicating no fuzzy score (match excluded from fuzzy sorting).
pub const FUZZY_SCORE_NONE: c_int = -1;

// CP flags for match comparison
const CP_EQUAL: c_int = 8;
const CP_ICASE: c_int = 16;

// compare_type constants for nvim_mergesort_compl_list
const COMPARE_FUZZY: c_int = 0;
#[allow(dead_code)]
const COMPARE_NEAREST: c_int = 1;

// cot flag constants
const K_OPT_COT_FLAG_NOSORT: c_uint = 0x100;
const K_OPT_COT_FLAG_NOINSERT: c_uint = 0x20;
const K_OPT_COT_FLAG_NOSELECT: c_uint = 0x40;

// C accessor functions
extern "C" {
    // Match list accessors
    fn nvim_compl_get_first_match() -> ComplMatch;

    // Match node accessors
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_next(m: ComplMatch, next: ComplMatch);
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_prev(m: ComplMatch, prev: ComplMatch);

    // Match list mutations
    fn nvim_compl_set_first_match(m: ComplMatch);
    fn nvim_compl_first_match_get_prev() -> ComplMatch;
    fn nvim_compl_first_match_next_is_first() -> c_int;

    // Match identification
    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;

    // Match score accessor
    fn nvim_compl_match_get_score(m: ComplMatch) -> c_int;
    #[allow(dead_code)]
    fn nvim_compl_match_get_flags(m: ComplMatch) -> c_int;

    // Fuzzy scoring accessors
    fn nvim_compl_match_set_score(m: ComplMatch, score: c_int);
    fn nvim_compl_match_get_cp_str_data(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_cp_str_size(m: ComplMatch) -> usize;
    fn fuzzy_match_str(str: *mut c_char, pat: *const c_char) -> c_int;
    // rs_get_leader_for_startcol_data is defined in Rust (leader.rs)
    fn nvim_get_compl_leader_data() -> *const c_char;
    fn nvim_get_compl_leader_size() -> usize;
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_orig_text_size() -> usize;

    // For rs_sort_compl_match_list and rs_ins_compl_fuzzy_sort
    fn nvim_mergesort_compl_list_raw(head: ComplMatch, compare_type: c_int) -> ComplMatch;
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_compl_get_shown_match() -> ComplMatch;
    fn nvim_compl_set_shown_match(m: ComplMatch);
    fn nvim_compl_shown_match_is_sentinel(forward: c_int) -> c_int;
    fn rs_get_cot_flags() -> c_uint;
    fn rs_compl_shows_dir_forward() -> c_int;

    // For rs_fuzzy_longest_match
    fn nvim_ins_redraw(ready: c_int);
    fn nvim_get_compl_num_bests() -> c_int;
    fn nvim_set_compl_num_bests(val: c_int);
    fn nvim_clear_compl_best_matches();
    fn rs_ins_compl_delete(new_leader: c_int);
    fn nvim_ins_compl_insert_bytes(p: *const c_char, len: c_int);
    fn rs_get_compl_len() -> c_int;
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;
    fn rs_ins_compl_leader() -> *const c_char;
    fn rs_ins_compl_leader_len() -> usize;
    fn rs_ctrl_x_mode_whole_line() -> c_int;
    fn nvim_xmalloc(size: usize) -> *mut u8;
    fn nvim_xfree(ptr: *mut u8);
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
        let _ = crate::leader::rs_get_leader_for_startcol_data(ComplMatch::null(), 1);
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
            let p = crate::leader::rs_get_leader_for_startcol_data(comp, 1);
            if p.is_null() {
                pattern
            } else {
                p
            }
        } else {
            pattern
        };

        let str_data = nvim_compl_match_get_cp_str_data(comp);
        let score = fuzzy_match_str(str_data.cast_mut(), pat);
        nvim_compl_match_set_score(comp, score);

        let next = nvim_compl_match_get_next(comp);
        if next.is_null() || is_first_match(next) {
            break;
        }
        comp = next;
    }
}

// =============================================================================
// Phase 3: rs_sort_compl_match_list and rs_ins_compl_fuzzy_sort
// =============================================================================

/// Sort completion matches using a comparator (fuzzy or nearest).
///
/// Implements the full `sort_compl_match_list` logic: detaches the leader node,
/// calls mergesort via `nvim_mergesort_compl_list_raw`, reattaches, and
/// makes the list cyclic.
///
/// The compare_type selects the comparator: COMPARE_FUZZY (0) or COMPARE_NEAREST (1).
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_sort_compl_match_list(compare_type: c_int) {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return;
    }
    // No items to sort if first->cp_next is back to first (only leader node)
    if nvim_compl_first_match_next_is_first() != 0 {
        return;
    }

    // comp = tail of the list (compl_first_match->cp_prev)
    let comp = nvim_compl_first_match_get_prev();

    crate::match_list::rs_ins_compl_make_linear();

    if rs_compl_shows_dir_forward() != 0 {
        // Forward: sort all nodes after the leader (compl_first_match->cp_next)
        let sort_head = nvim_compl_match_get_next(first);
        // Detach sort_head from leader
        nvim_compl_match_set_prev(sort_head, ComplMatch::null());
        // Sort
        let new_head = nvim_mergesort_compl_list_raw(sort_head, compare_type);
        // Reattach: first->cp_next = new_head, new_head->cp_prev = first
        nvim_compl_match_set_next(first, new_head);
        nvim_compl_match_set_prev(new_head, first);
    } else {
        // Backward: sort from compl_first_match up to (but not including) comp.
        // Detach: comp->cp_prev->cp_next = NULL
        let comp_prev = nvim_compl_match_get_prev(comp);
        nvim_compl_match_set_next(comp_prev, ComplMatch::null());
        // Sort the detached list
        let new_first = nvim_mergesort_compl_list_raw(first, compare_type);
        nvim_compl_set_first_match(new_first);
        // Find the new tail
        let mut tail = new_first;
        loop {
            let next = nvim_compl_match_get_next(tail);
            if next.is_null() {
                break;
            }
            tail = next;
        }
        // Reattach comp at the end
        nvim_compl_match_set_next(tail, comp);
        nvim_compl_match_set_prev(comp, tail);
    }

    let _ = crate::match_list::rs_ins_compl_make_cyclic();
}

/// Calculate fuzzy scores and sort completion matches.
///
/// Sets fuzzy scores via `rs_set_fuzzy_score()`, then sorts via
/// `rs_sort_compl_match_list(COMPARE_FUZZY)` unless nosort is set.
/// Adjusts the shown match after sorting.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_fuzzy_sort() {
    let cur_cot_flags = rs_get_cot_flags();

    rs_set_fuzzy_score();
    if (cur_cot_flags & K_OPT_COT_FLAG_NOSORT) != 0 {
        return;
    }

    rs_sort_compl_match_list(COMPARE_FUZZY);

    // Reset the shown item since sorting reorders items.
    // Only adjust if the flag combination is exactly NOINSERT (not NOSELECT).
    if (cur_cot_flags & (K_OPT_COT_FLAG_NOINSERT | K_OPT_COT_FLAG_NOSELECT))
        != K_OPT_COT_FLAG_NOINSERT
    {
        return;
    }

    let forward = rs_compl_shows_dir_forward() != 0;
    let none_selected = nvim_compl_shown_match_is_sentinel(c_int::from(forward)) != 0;
    if none_selected {
        return;
    }

    let first = nvim_compl_get_first_match();
    let new_shown = if nvim_get_compl_autocomplete() == 0 && forward {
        nvim_compl_match_get_next(first)
    } else {
        first
    };
    nvim_compl_set_shown_match(new_shown);
}

// =============================================================================
// Phase 1 (pass 12): rs_fuzzy_longest_match -- full Rust implementation
// =============================================================================

/// Calculate the longest common prefix among the best fuzzy matches and insert it.
///
/// Iterates `compl_best_matches` (top-scoring matches), computes the longest
/// common prefix using UTF-8 character-level comparison, and inserts it.
///
/// # Safety
/// Requires valid completion list state with compl_num_bests > 0.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_fuzzy_longest_match() {
    let num_bests = nvim_get_compl_num_bests();
    if num_bests == 0 {
        return;
    }

    let first = nvim_compl_get_first_match();
    if first.is_null() {
        nvim_set_compl_num_bests(0);
        return;
    }

    // nn_compl = first->cp_next->cp_next
    let next1 = nvim_compl_match_get_next(first);
    let nn_compl = if next1.is_null() {
        ComplMatch::null()
    } else {
        nvim_compl_match_get_next(next1)
    };
    let more_candidates = !nn_compl.is_null() && nvim_compl_is_first_match(nn_compl) == 0;

    // compl = whole_line ? first : first->cp_next
    let compl = if rs_ctrl_x_mode_whole_line() != 0 {
        first
    } else {
        nvim_compl_match_get_next(first)
    };

    if num_bests == 1 {
        if !more_candidates && !compl.is_null() {
            let str_data = nvim_compl_match_get_cp_str_data(compl);
            let compl_len = rs_get_compl_len() as usize;
            rs_ins_compl_delete(0);
            nvim_ins_compl_insert_bytes(str_data.add(compl_len), -1);
            nvim_ins_redraw(0);
        }
        nvim_set_compl_num_bests(0);
        return;
    }

    // Collect best matches into a Vec
    let mut best_matches: Vec<ComplMatch> = Vec::with_capacity(num_bests as usize);
    let mut cur = compl;
    let mut i = 0;
    while !cur.is_null() && i < num_bests {
        best_matches.push(cur);
        cur = nvim_compl_match_get_next(cur);
        i += 1;
    }

    if best_matches.is_empty() {
        nvim_set_compl_num_bests(0);
        nvim_clear_compl_best_matches();
        return;
    }

    // prefix starts as data from first best match
    let prefix = nvim_compl_match_get_cp_str_data(best_matches[0]);
    let mut prefix_len = nvim_compl_match_get_cp_str_size(best_matches[0]) as c_int;

    // Find the common prefix across all best matches (character-count based, matching C)
    for m in best_matches.iter().skip(1) {
        let cand_data = nvim_compl_match_get_cp_str_data(*m);
        let mut pp = prefix;
        let mut cp = cand_data;
        let mut j: c_int = 0;

        while j < prefix_len && *cp != 0 && *pp != 0 {
            let char_len = utfc_ptr2len(pp) as usize;
            // Compare char_len bytes at pp vs cp
            let mut eq = true;
            for k in 0..char_len {
                if *pp.add(k) != *cp.add(k) {
                    eq = false;
                    break;
                }
            }
            if !eq {
                break;
            }
            // MB_PTR_ADV equivalent
            pp = pp.add(utfc_ptr2len(pp) as usize);
            cp = cp.add(utfc_ptr2len(cp) as usize);
            j += 1;
        }

        if j > 0 {
            prefix_len = j;
        }
    }

    let leader = rs_ins_compl_leader();
    let leader_len = rs_ins_compl_leader_len();

    // If leader is set, check prefix starts with leader
    let skip_insert = leader_len > 0 && {
        // strncmp(prefix, leader, leader_len)
        let mut differs = false;
        for k in 0..leader_len {
            if *prefix.add(k) != *leader.add(k) {
                differs = true;
                break;
            }
        }
        differs
    };

    if !skip_insert {
        // xmemdupz(prefix, prefix_len)
        let dup_len = prefix_len as usize;
        let dup = nvim_xmalloc(dup_len + 1).cast::<c_char>();
        std::ptr::copy_nonoverlapping(prefix, dup, dup_len);
        *dup.add(dup_len) = 0;

        let compl_len = rs_get_compl_len() as usize;
        rs_ins_compl_delete(0);
        nvim_ins_compl_insert_bytes(dup.add(compl_len), -1);
        nvim_ins_redraw(0);
        nvim_xfree(dup.cast::<u8>());
    }

    nvim_clear_compl_best_matches();
    nvim_set_compl_num_bests(0);
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
