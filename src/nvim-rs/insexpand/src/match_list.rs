//! Completion match list data structures and memory management.
//!
//! This module provides Rust implementations for managing the completion match list,
//! including allocation, freeing, and list manipulation operations.

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
    fn nvim_ins_compl_del_pum();
    fn nvim_pum_clear();
}

// CP flags (must match C enum) - preserved for future use
#[allow(dead_code)]
const CP_ORIGINAL_TEXT: c_int = 1;

/// Get the next match in the list.
#[no_mangle]
pub unsafe extern "C" fn rs_cp_get_next(node: *mut c_void) -> *mut c_void {
    let m = ComplMatch(node);
    if m.is_null() {
        return ptr::null_mut();
    }
    nvim_compl_match_get_next(m).0
}

/// Set the next match in the list.
#[no_mangle]
pub unsafe extern "C" fn rs_cp_set_next(node: *mut c_void, next: *mut c_void) {
    let m = ComplMatch(node);
    if !m.is_null() {
        nvim_compl_match_set_next(m, ComplMatch(next));
    }
}

/// Get the previous match in the list.
#[no_mangle]
pub unsafe extern "C" fn rs_cp_get_prev(node: *mut c_void) -> *mut c_void {
    let m = ComplMatch(node);
    if m.is_null() {
        return ptr::null_mut();
    }
    nvim_compl_match_get_prev(m).0
}

/// Set the previous match in the list.
#[no_mangle]
pub unsafe extern "C" fn rs_cp_set_prev(node: *mut c_void, prev: *mut c_void) {
    let m = ComplMatch(node);
    if !m.is_null() {
        nvim_compl_match_set_prev(m, ComplMatch(prev));
    }
}

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

    nvim_ins_compl_del_pum();
    nvim_pum_clear();

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

/// FFI export: Get CP_ORIGINAL_TEXT constant.
#[no_mangle]
pub extern "C" fn rs_cp_original_text() -> c_int {
    CP_ORIGINAL_TEXT
}

/// FFI export: Get first match in the list.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_get_first_match() -> *mut c_void {
    nvim_compl_get_first_match().0
}

/// FFI export: Get current match.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_get_curr_match() -> *mut c_void {
    nvim_compl_get_curr_match().0
}

/// FFI export: Check if match handle is null.
#[no_mangle]
pub extern "C" fn rs_compl_match_is_null(node: *mut c_void) -> c_int {
    c_int::from(ComplMatch(node).is_null())
}

/// FFI export: Check if a match is the first match.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_is_first_match(node: *mut c_void) -> c_int {
    let m = ComplMatch(node);
    if m.is_null() {
        return 0;
    }
    nvim_compl_is_first_match(m)
}

/// FFI export: Check if a match is at original text.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_at_original_text(node: *mut c_void) -> c_int {
    let m = ComplMatch(node);
    if m.is_null() {
        return 0;
    }
    nvim_compl_match_at_original_text(m)
}

/// FFI export: Get match flags.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_match_flags(node: *mut c_void) -> c_int {
    let m = ComplMatch(node);
    if m.is_null() {
        return 0;
    }
    nvim_compl_match_get_flags(m)
}

// =============================================================================
// Phase 2: Match List Navigation and Selection
// =============================================================================

/// Navigate to the next match, skipping original text entry.
///
/// Returns the next match in the list, or null if at end.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_next_match(current: *mut c_void) -> *mut c_void {
    let m = ComplMatch(current);
    if m.is_null() {
        return ptr::null_mut();
    }

    let next = nvim_compl_match_get_next(m);
    if next.is_null() {
        return ptr::null_mut();
    }

    // If we wrapped around to first, check if it's original text
    if is_first_match(next) && match_at_original_text(next) {
        // Skip original text and return the one after
        let after_orig = nvim_compl_match_get_next(next);
        if is_first_match(after_orig) {
            // Only one match (the original text)
            return ptr::null_mut();
        }
        return after_orig.0;
    }

    next.0
}

/// Navigate to the previous match, skipping original text entry.
///
/// Returns the previous match in the list, or null if at start.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_prev_match(current: *mut c_void) -> *mut c_void {
    let m = ComplMatch(current);
    if m.is_null() {
        return ptr::null_mut();
    }

    let prev = nvim_compl_match_get_prev(m);
    if prev.is_null() {
        return ptr::null_mut();
    }

    // If we hit the original text, skip it
    if match_at_original_text(prev) {
        let before_orig = nvim_compl_match_get_prev(prev);
        if before_orig.is_null() || is_first_match(before_orig) {
            return ptr::null_mut();
        }
        return before_orig.0;
    }

    prev.0
}

/// Get the match at a specific index (0-based, excluding original text).
///
/// Returns the match at the given index, or null if index is out of bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_match_at_index(index: c_int) -> *mut c_void {
    if index < 0 {
        return ptr::null_mut();
    }

    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return ptr::null_mut();
    }

    // Start after first match (skip original text)
    let mut current = nvim_compl_match_get_next(first);
    let mut i = 0;

    while !current.is_null() && !is_first_match(current) {
        if i == index {
            return current.0;
        }
        i += 1;
        current = nvim_compl_match_get_next(current);
    }

    ptr::null_mut()
}

/// Get the index of a match in the list (0-based, excluding original text).
///
/// Returns the index of the match, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_match_index(target: *mut c_void) -> c_int {
    let target_match = ComplMatch(target);
    if target_match.is_null() {
        return -1;
    }

    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return -1;
    }

    // Start after first match (skip original text)
    let mut current = nvim_compl_match_get_next(first);
    let mut i = 0;

    while !current.is_null() && !is_first_match(current) {
        if current == target_match {
            return i;
        }
        i += 1;
        current = nvim_compl_match_get_next(current);
    }

    -1
}

/// Check if the match list is cyclic.
///
/// Returns true if the list wraps around (tail points to head).
#[no_mangle]
pub unsafe extern "C" fn rs_compl_list_is_cyclic() -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }

    let prev = nvim_compl_match_get_prev(first);
    c_int::from(!prev.is_null())
}

/// Get the last match in the list (before wrapping to first).
///
/// Returns the last match, or null if list is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_last_match() -> *mut c_void {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return ptr::null_mut();
    }

    // If cyclic, prev of first is the last
    let prev = nvim_compl_match_get_prev(first);
    if !prev.is_null() {
        return prev.0;
    }

    // Not cyclic, traverse to find last
    let mut current = first;
    loop {
        let next = nvim_compl_match_get_next(current);
        if next.is_null() {
            break;
        }
        current = next;
    }

    current.0
}

/// Count total matches in the list, excluding original text entry.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_count_matches() -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut current = first;

    loop {
        if !match_at_original_text(current) {
            count += 1;
        }

        let next = nvim_compl_match_get_next(current);
        if next.is_null() || is_first_match(next) {
            break;
        }
        current = next;
    }

    count
}

/// Remove a match from the list without freeing it.
///
/// Just unlinks the match from the list. The match itself is not freed.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_unlink_match(node: *mut c_void) {
    let m = ComplMatch(node);
    if m.is_null() {
        return;
    }

    let prev = nvim_compl_match_get_prev(m);
    let next = nvim_compl_match_get_next(m);

    if !prev.is_null() {
        nvim_compl_match_set_next(prev, next);
    }
    if !next.is_null() {
        nvim_compl_match_set_prev(next, prev);
    }

    // Clear the match's links
    nvim_compl_match_set_prev(m, ComplMatch::null());
    nvim_compl_match_set_next(m, ComplMatch::null());
}

/// Check if a match is the only non-original entry in the list.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_is_only_match(node: *mut c_void) -> c_int {
    let m = ComplMatch(node);
    if m.is_null() || match_at_original_text(m) {
        return 0;
    }

    let prev = nvim_compl_match_get_prev(m);
    let next = nvim_compl_match_get_next(m);

    // It's the only match if prev and next are both the original text or null
    let prev_is_orig = prev.is_null() || match_at_original_text(prev);
    let next_is_orig = next.is_null() || is_first_match(next);

    c_int::from(prev_is_orig && next_is_orig)
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
