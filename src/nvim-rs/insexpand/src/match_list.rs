//! Completion match list data structures and memory management.
//!
//! This module provides Rust implementations for managing the completion match list,
//! including allocation, freeing, and list manipulation operations.

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::os::raw::{c_char, c_int, c_void};
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

// Direct access to match list pointer globals
extern "C" {
    pub(crate) static mut compl_first_match: ComplMatch;
    pub(crate) static mut compl_curr_match: ComplMatch;
    #[allow(dead_code)]
    pub(crate) static mut compl_shown_match: ComplMatch;
    #[allow(dead_code)]
    pub(crate) static mut compl_old_match: ComplMatch;
}

// Helper inline accessors so callers can keep similar API
#[inline]
pub(crate) unsafe fn nvim_compl_get_first_match() -> ComplMatch {
    compl_first_match
}
#[inline]
pub(crate) unsafe fn nvim_compl_set_first_match(m: ComplMatch) {
    compl_first_match = m;
}
#[inline]
pub(crate) unsafe fn nvim_compl_get_curr_match() -> ComplMatch {
    compl_curr_match
}
#[inline]
pub(crate) unsafe fn nvim_compl_set_curr_match(m: ComplMatch) {
    compl_curr_match = m;
}
#[inline]
#[allow(dead_code)]
pub(crate) unsafe fn nvim_compl_get_shown_match() -> ComplMatch {
    compl_shown_match
}
#[inline]
pub(crate) unsafe fn nvim_compl_set_shown_match(m: ComplMatch) {
    compl_shown_match = m;
}
#[inline]
#[allow(dead_code)]
pub(crate) unsafe fn nvim_compl_get_old_match() -> ComplMatch {
    compl_old_match
}
#[inline]
pub(crate) unsafe fn nvim_compl_set_old_match(m: ComplMatch) {
    compl_old_match = m;
}

extern "C" {

    // Match node accessors
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_next(m: ComplMatch, next: ComplMatch);
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_prev(m: ComplMatch, prev: ComplMatch);
    #[allow(dead_code)]
    fn nvim_compl_match_get_flags(m: ComplMatch) -> c_int;

    // Match identification
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

    // cp_str accessors
    pub(crate) fn nvim_compl_match_get_cp_str_data(m: ComplMatch) -> *const c_char;
    pub(crate) fn nvim_compl_match_get_cp_str_size(m: ComplMatch) -> usize;

    // cp_score accessor
    pub(crate) fn nvim_compl_match_get_score(m: ComplMatch) -> c_int;

    // cp_cpt_source_idx accessor
    pub(crate) fn nvim_compl_match_get_cpt_source_idx(m: ComplMatch) -> c_int;

    // cp_fname accessor
    pub(crate) fn nvim_compl_match_has_fname(m: ComplMatch) -> c_int;

    // compl_orig_text and compl_leader accessors (String_T)
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_leader_data() -> *const c_char;

    // Direction check (from lib.rs)
    fn rs_compl_dir_forward() -> c_int;
}

// CP flags (must match C enum) - preserved for future use
#[allow(dead_code)]
const CP_ORIGINAL_TEXT: c_int = 1;

/// Check if a match is the first match (original text entry).
#[inline]
pub(crate) unsafe fn is_first_match(m: ComplMatch) -> bool {
    !m.is_null() && m == compl_first_match
}

/// Check if compl_shown_match is singular (only one item in ring: next == self).
#[inline]
pub(crate) unsafe fn shown_match_is_singular() -> bool {
    let sm = compl_shown_match;
    !sm.is_null() && sm == nvim_compl_match_get_next(sm)
}

/// Check if compl_shown_match is the first match (original text entry).
#[inline]
pub(crate) unsafe fn shown_match_is_first() -> bool {
    !compl_shown_match.is_null() && is_first_match(compl_shown_match)
}

/// Get the cp_str.size of compl_shown_match (0 if null or no data).
#[inline]
pub(crate) unsafe fn shown_match_str_size() -> usize {
    let sm = compl_shown_match;
    if sm.is_null() || nvim_compl_match_get_cp_str_data(sm).is_null() {
        0
    } else {
        nvim_compl_match_get_cp_str_size(sm)
    }
}

/// Check if compl_shown_match is at the original text entry.
#[inline]
pub(crate) unsafe fn shown_match_at_orig_text() -> bool {
    !compl_shown_match.is_null() && nvim_compl_match_at_original_text(compl_shown_match) != 0
}

/// Check if compl_curr_match is at the original text entry.
#[inline]
pub(crate) unsafe fn curr_match_at_original_text() -> bool {
    !compl_curr_match.is_null() && nvim_compl_match_at_original_text(compl_curr_match) != 0
}

/// Get cp_str.data of compl_shown_match (null if match is null).
#[inline]
pub(crate) unsafe fn shown_match_cp_str_data() -> *const c_char {
    if compl_shown_match.is_null() {
        std::ptr::null()
    } else {
        nvim_compl_match_get_cp_str_data(compl_shown_match)
    }
}

/// Get cp_str.size of compl_shown_match (0 if null).
#[inline]
pub(crate) unsafe fn shown_match_cp_str_size() -> usize {
    if compl_shown_match.is_null() {
        0
    } else {
        nvim_compl_match_get_cp_str_size(compl_shown_match)
    }
}

/// Get cp_cpt_source_idx of compl_shown_match (-1 if null).
#[inline]
pub(crate) unsafe fn shown_match_cpt_source_idx() -> c_int {
    if compl_shown_match.is_null() {
        -1
    } else {
        nvim_compl_match_get_cpt_source_idx(compl_shown_match)
    }
}

/// Get cp_score of compl_shown_match (FUZZY_SCORE_NONE=INT_MIN if null).
#[inline]
pub(crate) unsafe fn shown_match_score() -> c_int {
    if compl_shown_match.is_null() {
        c_int::MIN // FUZZY_SCORE_NONE = INT_MIN
    } else {
        nvim_compl_match_get_score(compl_shown_match)
    }
}

/// Check if compl_shown_match has a filename (cp_fname != NULL).
#[inline]
pub(crate) unsafe fn shown_match_has_fname() -> bool {
    !compl_shown_match.is_null() && nvim_compl_match_has_fname(compl_shown_match) != 0
}

/// Get cp_str.data of compl_curr_match (null if match is null).
#[inline]
pub(crate) unsafe fn curr_match_cp_str_data() -> *const c_char {
    if compl_curr_match.is_null() {
        std::ptr::null()
    } else {
        nvim_compl_match_get_cp_str_data(compl_curr_match)
    }
}

/// Advance compl_curr_match past compl_old_match.
///
/// At end of ins_compl_get_exp: if compl_old_match is set, move curr to
/// old_match->cp_next (forward) or cp_prev (backward). Fallback to old_match if null.
#[inline]
pub(crate) unsafe fn compl_old_match_advance_curr() {
    if !compl_old_match.is_null() {
        let next = if rs_compl_dir_forward() != 0 {
            nvim_compl_match_get_next(compl_old_match)
        } else {
            nvim_compl_match_get_prev(compl_old_match)
        };
        compl_curr_match = if next.is_null() {
            compl_old_match
        } else {
            next
        };
    }
}

/// Rewind compl_curr_match to the list head (^P: skip original text sentinel).
///
/// Walk back using cp_prev until we find a node whose prev is the original text
/// (match_at_original_text), then stop.
#[inline]
pub(crate) unsafe fn compl_curr_rewind_to_head() {
    loop {
        let curr = compl_curr_match;
        if curr.is_null() {
            break;
        }
        let prev = nvim_compl_match_get_prev(curr);
        if prev.is_null() || nvim_compl_match_at_original_text(prev) != 0 {
            break;
        }
        compl_curr_match = prev;
    }
}

/// Check if compl_shown_match cp_str has a newline character.
#[inline]
pub(crate) unsafe fn shown_match_has_newline() -> bool {
    let data = shown_match_cp_str_data();
    if data.is_null() {
        return false;
    }
    let s = std::ffi::CStr::from_ptr(data);
    s.to_bytes().contains(&b'\n')
}

/// Check if compl_shown_match cp_str equals compl_orig_text.
#[inline]
pub(crate) unsafe fn shown_match_str_eq_orig() -> bool {
    let orig = nvim_get_compl_orig_text_data();
    let shown = shown_match_cp_str_data();
    if shown.is_null() || orig.is_null() {
        return false;
    }
    let a = std::ffi::CStr::from_ptr(shown);
    let b = std::ffi::CStr::from_ptr(orig);
    a == b
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

/// Get cp_number of compl_curr_match (-1 if null).
#[inline]
pub(crate) unsafe fn curr_match_cp_number() -> c_int {
    if compl_curr_match.is_null() {
        -1
    } else {
        nvim_compl_match_get_cp_number(compl_curr_match)
    }
}

/// Set cp_number of compl_curr_match (no-op if null).
#[inline]
pub(crate) unsafe fn curr_match_set_cp_number(val: c_int) {
    if !compl_curr_match.is_null() {
        nvim_compl_match_set_cp_number(compl_curr_match, val);
    }
}

/// Check if compl_curr_match->cp_next == cp_prev (singleton ring).
#[inline]
pub(crate) unsafe fn curr_match_next_eq_prev() -> bool {
    if compl_curr_match.is_null() {
        return false;
    }
    let next = nvim_compl_match_get_next(compl_curr_match);
    let prev = nvim_compl_match_get_prev(compl_curr_match);
    !next.is_null() && next == prev
}

/// Check if compl_leader equals compl_orig_text (both non-null).
#[inline]
pub(crate) unsafe fn compl_leader_eq_orig_text() -> bool {
    let leader = nvim_get_compl_leader_data();
    let orig = nvim_get_compl_orig_text_data();
    if leader.is_null() || orig.is_null() {
        return false;
    }
    let a = std::ffi::CStr::from_ptr(leader);
    let b = std::ffi::CStr::from_ptr(orig);
    a == b
}

/// Set compl_shown_match to first match or first->cp_next based on no_select flag.
#[inline]
pub(crate) unsafe fn set_compl_shown_to_first_or_next(no_select: bool) {
    if compl_first_match.is_null() {
        return;
    }
    compl_shown_match = if no_select {
        compl_first_match
    } else {
        nvim_compl_match_get_next(compl_first_match)
    };
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
