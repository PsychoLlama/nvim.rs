//! Popup menu building and display operations.
//!
//! This module provides Rust implementations for popup menu related
//! functionality, including match counting and selection logic.

use std::os::raw::c_int;

use crate::match_list::ComplMatch;

// C accessor functions
extern "C" {
    // Match list accessors
    fn nvim_compl_get_first_match() -> ComplMatch;

    // Match node accessors
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;

    // Match identification
    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    // Match array accessor
    fn nvim_get_compl_match_array_exists() -> c_int;

    // Completeopt flags
    fn nvim_get_compl_autocomplete() -> c_int;
}

/// Check if a match is the first match.
#[inline]
unsafe fn is_first_match(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_is_first_match(m) != 0
}

/// Check if a match is at the original text position.
#[inline]
unsafe fn match_at_original_text(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_match_at_original_text(m) != 0
}

/// Check if the popup menu array exists.
///
/// Returns true if compl_match_array is not NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_array_exists() -> c_int {
    nvim_get_compl_match_array_exists()
}

/// Count visible matches (excluding original text entry).
///
/// Used to determine if there are enough matches to show the popup menu.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_count_visible_matches() -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut comp = first;

    loop {
        if !match_at_original_text(comp) {
            count += 1;
            // Early exit if we found 2 (enough for most cases)
            if count >= 2 {
                break;
            }
        }

        let next = nvim_compl_match_get_next(comp);
        if next.is_null() || is_first_match(next) {
            break;
        }
        comp = next;
    }

    count
}

/// Check if there are enough matches to show the popup menu.
///
/// - If "menuone" option is set or autocomplete is active, needs >= 1 match
/// - Otherwise, needs >= 2 matches
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_enough_matches(menuone: c_int) -> c_int {
    let count = rs_pum_count_visible_matches();
    let threshold = if menuone != 0 || nvim_get_compl_autocomplete() != 0 {
        1
    } else {
        2
    };
    c_int::from(count >= threshold)
}

/// Determine the minimum number of matches needed for popup display.
///
/// Returns 1 if menuone is set or autocomplete is active, otherwise 2.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_min_matches_needed() -> c_int {
    if nvim_get_compl_autocomplete() != 0 {
        1
    } else {
        2
    }
}

/// Calculate the selected item index when building the popup menu.
///
/// This implements the logic for tracking which item should be selected
/// based on the shown_match state and completion mode.
///
/// Parameters:
/// - `shown_match_ok`: whether a valid shown match was found
/// - `compl_no_select`: whether noselect mode is active
/// - `did_find_shown_match`: whether the shown match was found in the visible list
/// - `index`: the current item index
///
/// Returns the selected index, or -1 if nothing should be selected.
#[no_mangle]
pub const extern "C" fn rs_pum_calculate_selected(
    shown_match_ok: c_int,
    compl_no_select: c_int,
    _did_find_shown_match: c_int,
    index: c_int,
) -> c_int {
    // If no valid shown match and noselect mode, return -1
    if shown_match_ok == 0 && compl_no_select != 0 {
        return -1;
    }

    // If shown_match_ok is true, return the index
    if shown_match_ok != 0 {
        return index;
    }

    -1
}

/// Check if we should skip showing the popup menu.
///
/// The popup menu should be skipped if:
/// - pum_wanted is false, OR
/// - there aren't enough matches
#[no_mangle]
pub unsafe extern "C" fn rs_should_skip_pum(pum_wanted: c_int, menuone: c_int) -> c_int {
    if pum_wanted == 0 {
        return 1;
    }
    c_int::from(rs_pum_enough_matches(menuone) == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_selected() {
        // When shown_match_ok, return index
        assert_eq!(rs_pum_calculate_selected(1, 0, 0, 5), 5);
        assert_eq!(rs_pum_calculate_selected(1, 1, 0, 3), 3);

        // When not shown_match_ok and noselect, return -1
        assert_eq!(rs_pum_calculate_selected(0, 1, 0, 5), -1);

        // When not shown_match_ok and not noselect, return -1
        assert_eq!(rs_pum_calculate_selected(0, 0, 0, 5), -1);
    }
}
