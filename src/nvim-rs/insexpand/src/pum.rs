//! Popup menu building and display operations.
//!
//! This module provides Rust implementations for popup menu related
//! functionality, including match counting and selection logic.

#![allow(clippy::missing_const_for_fn)]

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

/// FFI export: Default minimum matches needed for popup (2).
#[no_mangle]
pub extern "C" fn rs_pum_default_min_matches() -> c_int {
    2
}

/// FFI export: Minimum matches needed for popup with menuone (1).
#[no_mangle]
pub extern "C" fn rs_pum_menuone_min_matches() -> c_int {
    1
}

/// FFI export: Check if autocomplete is active.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_autocomplete() -> c_int {
    nvim_get_compl_autocomplete()
}

/// FFI export: Check if first match is null.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_first_match_is_null() -> c_int {
    c_int::from(nvim_compl_get_first_match().is_null())
}

/// FFI export: Constant for no selection (-1).
#[no_mangle]
pub extern "C" fn rs_pum_no_selection() -> c_int {
    -1
}

/// FFI export: Check if match array exists.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_match_array_exists() -> c_int {
    nvim_get_compl_match_array_exists()
}

// =============================================================================
// Match selection query
// =============================================================================

extern "C" {
    fn nvim_compl_get_curr_match() -> ComplMatch;
    fn nvim_compl_match_get_cp_number(m: ComplMatch) -> c_int;
}

/// Check if the currently selected completion match is at the given popup index.
///
/// Iterates through the match list (skipping original text entries), finds the
/// index of compl_curr_match by comparing cp_number, and returns whether it
/// equals the given `selected` index.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_match_curr_select(selected: c_int) -> c_int {
    if selected < 0 {
        return 0;
    }

    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }

    let curr = nvim_compl_get_curr_match();
    if curr.is_null() {
        return c_int::from(selected == -1);
    }

    let curr_number = nvim_compl_match_get_cp_number(curr);

    let mut m = first;
    let mut selected_idx: c_int = -1;
    let mut list_idx: c_int = 0;

    loop {
        if !match_at_original_text(m) {
            if nvim_compl_match_get_cp_number(m) == curr_number {
                selected_idx = list_idx;
                break;
            }
            list_idx += 1;
        }
        let next = nvim_compl_match_get_next(m);
        if next.is_null() || is_first_match(next) {
            break;
        }
        m = next;
    }

    c_int::from(selected == selected_idx)
}

// =============================================================================
// Phase 2: Popup Menu Update Functions
// =============================================================================

// Additional C accessors for Phase 2
extern "C" {
    fn nvim_pum_visible() -> c_int;
    fn nvim_get_compl_selected_item() -> c_int;
    fn nvim_get_compl_started() -> c_int;
}

/// Check if the popup menu is visible.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_visible() -> c_int {
    nvim_pum_visible()
}

/// Check if the popup menu needs to be updated.
///
/// Returns true if completion is active but popup isn't visible.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_needs_update() -> c_int {
    let started = nvim_get_compl_started();
    let visible = nvim_pum_visible();
    c_int::from(started != 0 && visible == 0)
}

/// Check if the popup menu should be hidden.
///
/// Returns true if completion is not active but popup is visible.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_should_hide() -> c_int {
    let started = nvim_get_compl_started();
    let visible = nvim_pum_visible();
    c_int::from(started == 0 && visible != 0)
}

/// Get the current selected item in the popup menu.
///
/// Returns -1 if nothing is selected.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_selected_item() -> c_int {
    nvim_get_compl_selected_item()
}

/// Check if an item is selected in the popup menu.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_has_selection() -> c_int {
    c_int::from(nvim_get_compl_selected_item() >= 0)
}

/// Calculate the new selected index after navigation.
///
/// Parameters:
/// - `current_idx`: current selected index (-1 if none)
/// - `delta`: how much to move (positive = forward, negative = backward)
/// - `total_count`: total number of items in the list
///
/// Returns the new selected index, handling wrap-around.
#[no_mangle]
pub const extern "C" fn rs_pum_calc_new_selection(
    current_idx: c_int,
    delta: c_int,
    total_count: c_int,
) -> c_int {
    if total_count <= 0 {
        return -1;
    }

    // When starting from no selection, just select first or last
    if current_idx < 0 {
        return if delta > 0 { 0 } else { total_count - 1 };
    }

    let new_idx = current_idx + delta;

    // Handle wrap-around
    if new_idx < 0 {
        // Wrap to end
        ((new_idx % total_count) + total_count) % total_count
    } else if new_idx >= total_count {
        // Wrap to start
        new_idx % total_count
    } else {
        new_idx
    }
}

/// Check if the popup menu selection changed.
///
/// Compares old selection with current.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_selection_changed(old_selected: c_int) -> c_int {
    c_int::from(nvim_get_compl_selected_item() != old_selected)
}

/// Check if the popup is active (visible and has selection).
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_active() -> c_int {
    c_int::from(nvim_pum_visible() != 0 && nvim_get_compl_selected_item() >= 0)
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

    #[test]
    fn test_calc_new_selection() {
        // Basic forward navigation
        assert_eq!(rs_pum_calc_new_selection(0, 1, 5), 1);
        assert_eq!(rs_pum_calc_new_selection(3, 1, 5), 4);

        // Basic backward navigation
        assert_eq!(rs_pum_calc_new_selection(4, -1, 5), 3);
        assert_eq!(rs_pum_calc_new_selection(1, -1, 5), 0);

        // Wrap forward
        assert_eq!(rs_pum_calc_new_selection(4, 1, 5), 0);
        assert_eq!(rs_pum_calc_new_selection(4, 2, 5), 1);

        // Wrap backward
        assert_eq!(rs_pum_calc_new_selection(0, -1, 5), 4);
        assert_eq!(rs_pum_calc_new_selection(0, -2, 5), 3);

        // From no selection (-1)
        assert_eq!(rs_pum_calc_new_selection(-1, 1, 5), 0); // Forward starts at 0
        assert_eq!(rs_pum_calc_new_selection(-1, -1, 5), 4); // Backward starts at end

        // Empty list
        assert_eq!(rs_pum_calc_new_selection(0, 1, 0), -1);
    }
}
