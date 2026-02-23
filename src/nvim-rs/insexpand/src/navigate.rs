//! Match navigation support.
//!
//! This module provides helper functions for navigating through completion matches.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_int, c_void};

use crate::match_list::ComplMatch;

// C accessor functions
extern "C" {
    fn nvim_compl_get_first_match() -> ComplMatch;

    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;

    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    fn nvim_get_compl_direction() -> c_int;

    // Phase 4 compound accessors: implement the original C logic
    fn nvim_ins_compl_update_shown_match_impl();
    fn nvim_find_next_match_in_menu_impl();
}

// Direction constants
const FORWARD: c_int = 1;
#[allow(dead_code)]
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
    nvim_ins_compl_update_shown_match_impl();
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
    nvim_find_next_match_in_menu_impl();
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
