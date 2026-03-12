//! Completion match list data structures and memory management.
//!
//! This module provides Rust implementations for managing the completion match list,
//! including allocation, freeing, and list manipulation operations.

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::os::raw::{c_int, c_void};
use std::ptr;

/// Opaque handle to a completion match (compl_T).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplMatch(pub *mut c_void);

impl ComplMatch {
    /// Create a null match handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for the match list
extern "C" {
    // Match list accessors
    fn nvim_compl_get_first_match() -> ComplMatch;
    fn nvim_compl_set_first_match(m: ComplMatch);
    fn nvim_compl_get_curr_match() -> ComplMatch;
    fn nvim_compl_set_curr_match(m: ComplMatch);
    #[allow(dead_code)]
    fn nvim_compl_get_shown_match() -> ComplMatch;
    fn nvim_compl_set_shown_match(m: ComplMatch);
    #[allow(dead_code)]
    fn nvim_compl_get_old_match() -> ComplMatch;
    fn nvim_compl_set_old_match(m: ComplMatch);

    // Match node accessors
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_next(m: ComplMatch, next: ComplMatch);
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_prev(m: ComplMatch, prev: ComplMatch);
    #[allow(dead_code)]
    fn nvim_compl_match_get_flags(m: ComplMatch) -> c_int;

    // Match identification
    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;
    #[allow(dead_code)]
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    // Item freeing (C handles actual memory)
    fn nvim_compl_item_free(m: ComplMatch);

    // Pattern and leader clearing
    fn nvim_compl_clear_pattern();
    fn nvim_compl_clear_leader();

    // Popup menu operations
    fn pum_clear();

    // cp_number accessors
    fn nvim_compl_match_get_cp_number(m: ComplMatch) -> c_int;
    fn nvim_compl_match_set_cp_number(m: ComplMatch, num: c_int);

    // Direction check (from lib.rs)
    fn rs_compl_dir_forward() -> c_int;
}

// CP flags (must match C enum) - preserved for future use
#[allow(dead_code)]
const CP_ORIGINAL_TEXT: c_int = 1;

/// Check if a match is the first match (original text entry).
#[inline]
unsafe fn is_first_match(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_is_first_match(m) != 0
}

/// Check if a match represents the original text.
#[inline]
#[allow(dead_code)]
unsafe fn match_at_original_text(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_match_at_original_text(m) != 0
}

/// Make the completion list cyclic (connect tail to head).
///
/// Returns the number of matches (excluding the original text entry).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_make_cyclic() -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }

    // Find the end of the list
    let mut current = first;
    let mut count = 0;

    // There's always an entry for the compl_orig_text, it doesn't count
    loop {
        let next = nvim_compl_match_get_next(current);
        if next.is_null() || is_first_match(next) {
            break;
        }
        current = next;
        count += 1;
    }

    // Make it cyclic: connect tail to head
    nvim_compl_match_set_next(current, first);
    nvim_compl_match_set_prev(first, current);

    count
}

/// Make the completion list non-cyclic (linear).
///
/// Breaks the connection between the tail and head of the list.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_make_linear() {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return;
    }

    let prev = nvim_compl_match_get_prev(first);
    if prev.is_null() {
        return;
    }

    // Break the cycle
    nvim_compl_match_set_next(prev, ComplMatch::null());
    nvim_compl_match_set_prev(first, ComplMatch::null());
}

/// Free the entire list of completions.
///
/// This clears the pattern and leader, removes the popup menu,
/// and frees all match items.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_free() {
    nvim_compl_clear_pattern();
    nvim_compl_clear_leader();

    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return;
    }

    crate::pum::rs_ins_compl_del_pum();
    pum_clear();

    // Set curr_match to first_match to start iteration
    nvim_compl_set_curr_match(first);

    loop {
        let current = nvim_compl_get_curr_match();
        let next = nvim_compl_match_get_next(current);

        // Free the current item (C handles actual memory deallocation)
        nvim_compl_item_free(current);

        // Move to next
        if next.is_null() || is_first_match(next) {
            break;
        }
        nvim_compl_set_curr_match(next);
    }

    // Clear all match pointers
    nvim_compl_set_first_match(ComplMatch::null());
    nvim_compl_set_curr_match(ComplMatch::null());
    nvim_compl_set_shown_match(ComplMatch::null());
    nvim_compl_set_old_match(ComplMatch::null());
}

// =============================================================================
// Phase 2: Match List Navigation and Selection
// =============================================================================

/// Assign sequential numbers to completion matches that don't have one yet.
///
/// Traverses the match linked list starting from `compl_curr_match` to find
/// the first already-numbered entry, then walks in the opposite direction
/// assigning incrementing numbers.
///
/// In FORWARD mode: searches backward for a numbered entry, then walks
/// forward assigning numbers. In BACKWARD mode: searches forward for a
/// numbered entry, then walks backward assigning numbers.
///
/// # Safety
/// Requires valid completion list state (`compl_curr_match` non-null).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_update_sequence_numbers() {
    let curr = nvim_compl_get_curr_match();
    if curr.is_null() {
        return;
    }

    let mut number: c_int = 0;

    if rs_compl_dir_forward() != 0 {
        // Search backwards for the first valid (!= -1) number.
        let mut match_ = nvim_compl_match_get_prev(curr);
        while !match_.is_null() && !is_first_match(match_) {
            if nvim_compl_match_get_cp_number(match_) != -1 {
                number = nvim_compl_match_get_cp_number(match_);
                break;
            }
            match_ = nvim_compl_match_get_prev(match_);
        }
        if !match_.is_null() {
            // Go forward and assign all numbers which are not assigned yet
            let mut assign = nvim_compl_match_get_next(match_);
            while !assign.is_null() && nvim_compl_match_get_cp_number(assign) == -1 {
                number += 1;
                nvim_compl_match_set_cp_number(assign, number);
                assign = nvim_compl_match_get_next(assign);
            }
        }
    } else {
        // BACKWARD: search forwards for the first valid (!= -1) number.
        let mut match_ = nvim_compl_match_get_next(curr);
        while !match_.is_null() && !is_first_match(match_) {
            if nvim_compl_match_get_cp_number(match_) != -1 {
                number = nvim_compl_match_get_cp_number(match_);
                break;
            }
            match_ = nvim_compl_match_get_next(match_);
        }
        if !match_.is_null() {
            // Go backward and assign all numbers which are not assigned yet
            let mut assign = nvim_compl_match_get_prev(match_);
            while !assign.is_null() && nvim_compl_match_get_cp_number(assign) == -1 {
                number += 1;
                nvim_compl_match_set_cp_number(assign, number);
                assign = nvim_compl_match_get_prev(assign);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compl_match_null() {
        let m = ComplMatch::null();
        assert!(m.is_null());
        assert_eq!(m.0, ptr::null_mut());
    }

    #[test]
    fn test_cp_flag_constant() {
        assert_eq!(CP_ORIGINAL_TEXT, 1);
    }
}
