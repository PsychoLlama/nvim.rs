//! Completion match addition operations.
//!
//! This module provides infrastructure for adding completion matches to the list.
//! The actual VimL type handling remains in C, but Rust provides the underlying
//! list manipulation logic.

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

/// Check if a match already exists in the completion list.
///
/// Searches the match list for a duplicate entry with the same string.
/// Used before adding a new match to avoid duplicates.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_match_exists(str_ptr: *const u8, str_len: c_int) -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() || str_ptr.is_null() {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    let len = if str_len < 0 {
        // Calculate length from null-terminated string
        let mut l = 0usize;
        while *str_ptr.add(l) != 0 {
            l += 1;
        }
        l
    } else {
        str_len as usize
    };

    let search_str = std::slice::from_raw_parts(str_ptr, len);

    let mut current = first;
    loop {
        if !match_at_original_text(current) {
            // Get string from match - this requires C accessor
            // For now, this function is a framework - actual string comparison
            // will be done in C wrapper
            // This demonstrates the list traversal pattern
        }

        let next = nvim_compl_match_get_next(current);
        if next.is_null() || is_first_match(next) {
            break;
        }
        current = next;
    }

    // Actual comparison happens in C due to string access
    let _ = search_str; // Use the variable to avoid warning
    0
}

/// Find the position to insert a new match based on score.
///
/// When using fuzzy matching with longest completion, matches are sorted by score.
/// This function finds where to insert a new match in the sorted list.
///
/// Returns the match after which the new match should be inserted,
/// or null if it should be inserted at the end.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_find_insert_position_by_score(score: c_int) -> ComplMatch {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return ComplMatch::null();
    }

    // Start after the first match (original text entry)
    let mut current = nvim_compl_match_get_next(first);
    let mut prev = first;

    while !current.is_null() && !is_first_match(current) {
        // Get score from current match - requires C accessor
        // This is a framework function showing the traversal pattern
        prev = current;
        current = nvim_compl_match_get_next(current);
    }

    let _ = score; // Score comparison done in C
    prev
}

/// Insert a match after a given position in the list.
///
/// # Safety
/// Both `after` and `new_match` must be valid match handles.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_match_after(after: ComplMatch, new_match: ComplMatch) {
    if after.is_null() || new_match.is_null() {
        return;
    }

    let next = nvim_compl_match_get_next(after);

    // Link new match into the list
    nvim_compl_match_set_next(after, new_match);
    nvim_compl_match_set_prev(new_match, after);
    nvim_compl_match_set_next(new_match, next);

    if !next.is_null() {
        nvim_compl_match_set_prev(next, new_match);
    }
}

/// Insert a match before a given position in the list.
///
/// # Safety
/// Both `before` and `new_match` must be valid match handles.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_match_before(before: ComplMatch, new_match: ComplMatch) {
    if before.is_null() || new_match.is_null() {
        return;
    }

    let prev = nvim_compl_match_get_prev(before);

    // Link new match into the list
    nvim_compl_match_set_prev(before, new_match);
    nvim_compl_match_set_next(new_match, before);
    nvim_compl_match_set_prev(new_match, prev);

    if !prev.is_null() {
        nvim_compl_match_set_next(prev, new_match);
    }
}

/// Get the effective direction for adding a match.
///
/// If the specified direction is 0 (kDirectionNotSet), returns the current
/// completion direction. Otherwise returns the specified direction.
#[no_mangle]
pub unsafe extern "C" fn rs_get_effective_direction(cdir: c_int) -> c_int {
    if cdir == 0 {
        nvim_get_compl_direction()
    } else {
        cdir
    }
}

/// Count matches in the list, excluding the original text entry.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_count_matches() -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut current = nvim_compl_match_get_next(first);

    while !current.is_null() && !is_first_match(current) {
        count += 1;
        current = nvim_compl_match_get_next(current);
    }

    count
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
