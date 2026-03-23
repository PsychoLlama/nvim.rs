//! Match navigation support.
//!
//! This module provides helper functions for navigating through completion matches.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_int, c_void};

use crate::match_list::ComplMatch;

// C accessor functions
extern "C" {
    fn nvim_compl_get_first_match() -> ComplMatch;
    fn nvim_compl_set_first_match(m: ComplMatch);
    fn nvim_compl_get_curr_match() -> ComplMatch;
    fn nvim_compl_set_curr_match(m: ComplMatch);
    fn nvim_compl_get_shown_match() -> ComplMatch;
    fn nvim_compl_set_shown_match(m: ComplMatch);

    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_next(m: ComplMatch, next: ComplMatch);
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_prev(m: ComplMatch, prev: ComplMatch);

    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;
    fn nvim_compl_item_free(m: ComplMatch);

    fn nvim_compl_match_get_cpt_source_idx(m: ComplMatch) -> c_int;
    fn nvim_compl_match_get_in_match_array(m: ComplMatch) -> c_int;

    // leader-for-startcol (Rust implementation in leader.rs)
    fn rs_get_leader_for_startcol_data(m: ComplMatch, cached: c_int) -> *const std::ffi::c_char;
    fn rs_get_leader_for_startcol_size(m: ComplMatch, cached: c_int) -> usize;

    fn rs_ins_compl_equal(m: ComplMatch, str_: *const std::ffi::c_char, len: usize) -> c_int;
    fn rs_compl_shows_dir_forward() -> c_int;
    fn rs_compl_shows_dir_backward() -> c_int;
}

// Direction constants
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

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

// =============================================================================
// Phase 4: shown_match navigation
// =============================================================================

/// Update `compl_shown_match` to the actually shown match.
///
/// It may differ when `compl_leader` is used to omit some matches. Walks
/// the match list forward (and optionally backward) until a matching entry
/// is found.
///
/// # Safety
/// Requires valid global completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_update_shown_match() {
    // Clear the cache, then get the leader for the current shown match
    let _ = rs_get_leader_for_startcol_data(ComplMatch::null(), 1); // clear cache
    let shown = nvim_compl_get_shown_match();
    if shown.is_null() {
        return;
    }

    let mut current = shown;

    let mut leader_data = rs_get_leader_for_startcol_data(current, 1);
    let mut leader_size = rs_get_leader_for_startcol_size(current, 1);

    while rs_ins_compl_equal(current, leader_data, leader_size) == 0 {
        let next = nvim_compl_match_get_next(current);
        if next.is_null() || is_first_match(next) {
            break;
        }
        current = next;
        leader_data = rs_get_leader_for_startcol_data(current, 1);
        leader_size = rs_get_leader_for_startcol_size(current, 1);
    }

    // If we didn't find it searching forward, and compl_shows_dir is
    // backward, find the last match.
    if rs_compl_shows_dir_backward() != 0
        && rs_ins_compl_equal(current, leader_data, leader_size) == 0
    {
        let next = nvim_compl_match_get_next(current);
        if next.is_null() || is_first_match(next) {
            // Try searching backward
            while rs_ins_compl_equal(current, leader_data, leader_size) == 0 {
                let prev = nvim_compl_match_get_prev(current);
                if prev.is_null() || is_first_match(prev) {
                    break;
                }
                current = prev;
                leader_data = rs_get_leader_for_startcol_data(current, 1);
                leader_size = rs_get_leader_for_startcol_size(current, 1);
            }
        }
    }

    nvim_compl_set_shown_match(current);
}

/// Advance `compl_shown_match` to the next match that is present in the
/// popup menu match array (`cp_in_match_array`).
///
/// Used when `complete` ('cpt') includes a `max_matches` postfix so the menu
/// only shows a subset of the full match list.
///
/// # Safety
/// Requires valid global completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_find_next_match_in_menu() {
    let is_forward = rs_compl_shows_dir_forward() != 0;
    let mut m = nvim_compl_get_shown_match();
    if m.is_null() {
        return;
    }

    loop {
        m = if is_forward {
            nvim_compl_match_get_next(m)
        } else {
            nvim_compl_match_get_prev(m)
        };
        if m.is_null() {
            break;
        }
        let has_next = !nvim_compl_match_get_next(m).is_null();
        if !has_next || nvim_compl_match_get_in_match_array(m) != 0 || match_at_original_text(m) {
            break;
        }
    }

    if !m.is_null() {
        nvim_compl_set_shown_match(m);
    }
}

// =============================================================================
// Phase 10 (pass 10): remove_old_matches migrated from C compound accessor
// =============================================================================

/// Remove completion matches from the current cpt source index.
///
/// Transliteration of the C `nvim_remove_old_matches_impl` compound accessor
/// that was previously at the bottom of `insexpand_shim.c`.
///
/// # Safety
/// Requires valid global completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_remove_old_matches() {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return;
    }
    let cpt_sources_index = crate::vars::nvim_get_cpt_sources_index();
    if cpt_sources_index < 0 {
        return;
    }

    let forward = nvim_compl_match_get_cpt_source_idx(first) < 0;
    let direction = if forward { FORWARD } else { BACKWARD };
    crate::vars::nvim_set_compl_direction(direction);
    crate::vars::nvim_set_compl_shows_dir(direction);

    let shown_before_removal = nvim_compl_get_shown_match();
    let mut shown_match_removed = false;

    // Walk the list and remove items matching cpt_sources_index
    let mut current = first;
    loop {
        if current.is_null() {
            break;
        }
        if nvim_compl_match_get_cpt_source_idx(current) == cpt_sources_index {
            let to_delete = current;

            if !shown_match_removed && to_delete == shown_before_removal {
                shown_match_removed = true;
            }

            // Advance before we mutate/free
            current = nvim_compl_match_get_next(to_delete);

            let first_now = nvim_compl_get_first_match();
            if to_delete == first_now {
                // Node to remove is at head
                let new_first = nvim_compl_match_get_next(to_delete);
                nvim_compl_set_first_match(new_first);
                nvim_compl_match_set_prev(new_first, ComplMatch::null());
            } else {
                let prev = nvim_compl_match_get_prev(to_delete);
                let next = nvim_compl_match_get_next(to_delete);
                if next.is_null() {
                    // Node is at tail
                    nvim_compl_match_set_next(prev, ComplMatch::null());
                } else {
                    // Node is in the middle
                    nvim_compl_match_set_next(prev, next);
                    nvim_compl_match_set_prev(next, prev);
                }
            }
            nvim_compl_item_free(to_delete);
        } else {
            current = nvim_compl_match_get_next(current);
        }
    }

    // Re-assign compl_shown_match if necessary
    if shown_match_removed {
        if forward {
            nvim_compl_set_shown_match(nvim_compl_get_first_match());
        } else {
            // Last node will have the prefix that is being completed
            let mut node = nvim_compl_get_first_match();
            loop {
                let next = nvim_compl_match_get_next(node);
                if next.is_null() {
                    break;
                }
                node = next;
            }
            nvim_compl_set_shown_match(node);
        }
    }

    // Re-assign compl_curr_match
    nvim_compl_set_curr_match(nvim_compl_get_first_match());
    let mut cur = nvim_compl_get_first_match();
    loop {
        if cur.is_null() {
            break;
        }
        let src_idx = nvim_compl_match_get_cpt_source_idx(cur);
        let should_advance = if forward {
            src_idx < cpt_sources_index
        } else {
            src_idx > cpt_sources_index
        };
        if should_advance {
            let next = nvim_compl_match_get_next(cur);
            nvim_compl_set_curr_match(if forward { cur } else { next });
            cur = next;
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }
}
