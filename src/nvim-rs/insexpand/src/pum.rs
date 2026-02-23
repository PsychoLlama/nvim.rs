//! Popup menu building and display operations.
//!
//! This module provides Rust implementations for popup menu related
//! functionality, including match counting and selection logic.

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::os::raw::{c_int, c_uint};

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

/// Count visible matches (non-original-text entries), stopping at 2.
unsafe fn pum_count_visible_matches() -> c_int {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return 0;
    }
    let mut count = 0;
    let mut comp = first;
    loop {
        if !match_at_original_text(comp) {
            count += 1;
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
    let count = pum_count_visible_matches();
    let threshold = if menuone != 0 || nvim_get_compl_autocomplete() != 0 {
        1
    } else {
        2
    };
    c_int::from(count >= threshold)
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

// Accessors for rs_ins_compl_show_pum
extern "C" {
    fn nvim_update_screen();
    fn nvim_ins_compl_build_pum() -> c_int;
    fn nvim_find_shown_match_in_array() -> c_int;
    fn nvim_trigger_complete_changed(cur: c_int);
    fn nvim_has_completechanged_event() -> c_int;
    fn nvim_set_dollar_vcol_minus_one();
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col_to_compl_col();
    fn nvim_restore_cursor_col(col: c_int);
    fn nvim_pum_display_compl(cur: c_int, array_changed: c_int);
    fn nvim_compl_curr_neq_shown() -> c_int;
    fn nvim_compl_set_curr_to_shown();
    fn nvim_set_compl_selected_item(val: c_int);
}

/// Show the popup menu for completion matches.
///
/// This is the Rust implementation of ins_compl_show_pum(). It orchestrates
/// the popup menu display by calling back into C for all state mutations.
///
/// # Safety
/// Requires valid completion state; called from insert mode only.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_show_pum() {
    // kOptCotFlagMenuone = 0x001 (matches C constant)
    const K_OPT_COT_FLAG_MENUONE: c_uint = 0x001;

    if crate::rs_pum_wanted() == 0
        || rs_pum_enough_matches(c_int::from(
            (crate::rs_get_cot_flags() & K_OPT_COT_FLAG_MENUONE) != 0
                || nvim_get_compl_autocomplete() != 0,
        )) == 0
    {
        return;
    }

    // Update the screen before drawing the popup menu over it.
    nvim_update_screen();

    let (cur, array_changed): (c_int, c_int);

    if nvim_get_compl_match_array_exists() == 0 {
        array_changed = 1;
        // Need to build the popup menu list.
        cur = nvim_ins_compl_build_pum();
    } else {
        array_changed = 0;
        // popup menu already exists, only need to find the current item.
        cur = nvim_find_shown_match_in_array();
    }

    if nvim_get_compl_match_array_exists() == 0 {
        if nvim_get_compl_started() != 0 && nvim_has_completechanged_event() != 0 {
            nvim_trigger_complete_changed(cur);
        }
        return;
    }

    // In Replace mode when a $ is displayed at the end of the line only
    // part of the screen would be updated.  We do need to redraw here.
    nvim_set_dollar_vcol_minus_one();

    // Compute the screen column of the start of the completed text.
    // Use the cursor to get all wrapping and other settings right.
    let col = nvim_get_cursor_col();
    nvim_set_cursor_col_to_compl_col();
    nvim_set_compl_selected_item(cur);
    nvim_pum_display_compl(cur, array_changed);
    nvim_restore_cursor_col(col);

    // After adding leader, set the current match to shown match.
    if nvim_get_compl_started() != 0 && nvim_compl_curr_neq_shown() != 0 {
        nvim_compl_set_curr_to_shown();
    }

    if nvim_has_completechanged_event() != 0 {
        nvim_trigger_complete_changed(cur);
    }
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
