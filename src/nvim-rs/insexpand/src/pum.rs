//! Popup menu building and display operations.
//!
//! This module provides Rust implementations for popup menu related
//! functionality, including match counting and selection logic.

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::os::raw::{c_int, c_uint};

use crate::match_list::ComplMatch;

use crate::match_list::{
    is_first_match, nvim_compl_get_curr_match, nvim_compl_get_first_match,
    nvim_compl_get_shown_match, nvim_compl_set_curr_match, nvim_compl_set_shown_match,
};

// C accessor functions
extern "C" {
    fn xfree(ptr: *mut u8);
    // Match node accessors
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;

    // Match identification
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    // For rs_ins_compl_del_pum
    #[link_name = "pum_undisplay"]
    fn nvim_pum_undisplay(undo: c_int);
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
    let threshold = if menuone != 0 || crate::vars::nvim_get_compl_autocomplete() != 0 {
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
    fn pum_visible() -> bool;
}

// Accessors for rs_ins_compl_show_pum
extern "C" {
    fn nvim_update_screen();
    fn nvim_find_shown_match_in_match_array() -> c_int;
    fn nvim_trigger_complete_changed_guarded(cur: c_int);
    fn nvim_has_completechanged_event() -> c_int;
    fn nvim_set_dollar_vcol(val: c_int);
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_pum_display_compl(cur: c_int, array_changed: c_int);
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
    // kOptCotFlagMenuone = 0x002 (bit 1 in options.lua values list)
    const K_OPT_COT_FLAG_MENUONE: c_uint = 0x002;

    if crate::rs_pum_wanted() == 0
        || rs_pum_enough_matches(c_int::from(
            (crate::rs_get_cot_flags() & K_OPT_COT_FLAG_MENUONE) != 0
                || crate::vars::nvim_get_compl_autocomplete() != 0,
        )) == 0
    {
        return;
    }

    // Update the screen before drawing the popup menu over it.
    nvim_update_screen();

    let (cur, array_changed): (c_int, c_int);

    if crate::vars::nvim_get_compl_match_array_exists() == 0 {
        array_changed = 1;
        // Need to build the popup menu list.
        cur = rs_ins_compl_build_pum();
    } else {
        array_changed = 0;
        // popup menu already exists, only need to find the current item.
        cur = nvim_find_shown_match_in_match_array();
    }

    if crate::vars::nvim_get_compl_match_array_exists() == 0 {
        if crate::vars::nvim_get_compl_started() != 0 && nvim_has_completechanged_event() != 0 {
            nvim_trigger_complete_changed_guarded(cur);
        }
        return;
    }

    // In Replace mode when a $ is displayed at the end of the line only
    // part of the screen would be updated.  We do need to redraw here.
    nvim_set_dollar_vcol(-1);

    // Compute the screen column of the start of the completed text.
    // Use the cursor to get all wrapping and other settings right.
    let col = nvim_get_cursor_col();
    nvim_set_cursor_col(crate::vars::nvim_get_compl_col());
    crate::vars::nvim_set_compl_selected_item(cur);
    nvim_pum_display_compl(cur, array_changed);
    nvim_set_cursor_col(col);

    // After adding leader, set the current match to shown match.
    if crate::vars::nvim_get_compl_started() != 0
        && nvim_compl_get_curr_match() != nvim_compl_get_shown_match()
    {
        nvim_compl_set_curr_match(nvim_compl_get_shown_match());
    }

    if nvim_has_completechanged_event() != 0 {
        nvim_trigger_complete_changed_guarded(cur);
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

// =============================================================================
// Phase 2 (pass 11): rs_ins_compl_build_pum
// =============================================================================

// Additional C accessors for Phase 2 (pass 11)
extern "C" {
    fn nvim_compl_match_set_in_match_array(m: ComplMatch, val: c_int);
    fn nvim_compl_match_get_match_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_match_next(m: ComplMatch, next: ComplMatch);
    fn nvim_compl_match_clear_icase(m: ComplMatch);
    // (nvim_compl_leader_eq_orig_text and nvim_set_compl_shown_to_first_or_next: inlined in match_list.rs)
    fn nvim_build_pum_fill_array(match_head: ComplMatch, count: c_int) -> c_int;
    // (nvim_cpt_sources_array_exists, nvim_get_cpt_source_cs_max_matches: inlined in vars.rs Phase 23)
    #[allow(clashing_extern_declarations)]
    fn xmalloc(size: usize) -> *mut c_int;

    // From leader.rs (pass 11)
    fn rs_get_leader_for_startcol_data(m: ComplMatch, cached: c_int) -> *const std::ffi::c_char;
    fn rs_get_leader_for_startcol_size(m: ComplMatch, cached: c_int) -> usize;

    // Options
    // nvim_get_compl_autocomplete already declared above
    fn nvim_compl_match_get_cpt_source_idx(m: ComplMatch) -> c_int;
    fn nvim_compl_match_get_score(m: ComplMatch) -> c_int;
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn nvim_get_p_inf() -> c_int;
    fn nvim_ignorecase(pat: *const std::ffi::c_char) -> bool;
    fn rs_ins_compl_equal(m: ComplMatch, str_: *const std::ffi::c_char, len: usize) -> c_int;
    // nvim_api_clear_compl_leader: inlined in vars.rs as nvim_compl_clear_leader (Phase 25)
    fn rs_ins_compl_need_restart() -> c_int;
    fn rs_get_cpt_sources_count() -> c_int;
}

// kOptCotFlagNoselect = 0x40 (from option_vars.generated.h)
const K_OPT_COT_FLAG_NOSELECT: c_uint = 0x40;

// FUZZY_SCORE_NONE = -1 (from insexpand_shim.c)
const FUZZY_SCORE_NONE: c_int = -1;

/// Build the popup menu list from the completion match list.
///
/// Iterates the match list, filters by leader and max_matches, tracks the
/// shown_match selection, and fills the compl_match_array via C compound accessor.
///
/// Returns the selected index, or -1 if nothing should be selected.
///
/// # Safety
/// Requires valid global completion state.
#[no_mangle]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ins_compl_build_pum() -> c_int {
    // Reset match array size
    crate::vars::nvim_set_compl_match_arraysize(0);

    // If it's user complete function and refresh_always,
    // do not use "compl_leader" as prefix filter.
    if rs_ins_compl_need_restart() != 0 {
        crate::vars::nvim_compl_clear_leader();
    }

    let compl_no_select = (crate::rs_get_cot_flags() & K_OPT_COT_FLAG_NOSELECT) != 0
        || (crate::vars::nvim_get_compl_autocomplete() != 0
            && crate::rs_ins_compl_has_preinsert() == 0);

    let is_forward = crate::rs_compl_shows_dir_forward() != 0;
    let is_cpt_completion = crate::vars::nvim_cpt_sources_array_exists() != 0;

    // If the current match is the original text don't find the first
    // match after it, don't highlight anything.
    let shown_match = nvim_compl_get_shown_match();
    let mut shown_match_ok = !shown_match.is_null() && match_at_original_text(shown_match);

    if crate::match_list::compl_leader_eq_orig_text() && !shown_match_ok {
        crate::match_list::set_compl_shown_to_first_or_next(compl_no_select);
    }

    let mut did_find_shown_match = false;
    let mut shown_compl = ComplMatch::null();
    let mut match_head = ComplMatch::null();
    let mut match_tail = ComplMatch::null();
    let mut array_size: c_int = 0;
    let mut i: c_int = 0;
    let mut cur: c_int = -1;

    // Allocate per-source match counts if cpt completion
    let match_count_ptr: *mut c_int = if is_cpt_completion {
        let cnt = rs_get_cpt_sources_count();
        if cnt > 0 {
            let ptr = xmalloc(cnt as usize * std::mem::size_of::<c_int>());
            // Zero-initialize
            std::ptr::write_bytes(ptr, 0, cnt as usize);
            ptr
        } else {
            std::ptr::null_mut()
        }
    } else {
        std::ptr::null_mut()
    };

    // Clear the leader-for-startcol cache
    let _ = rs_get_leader_for_startcol_data(ComplMatch::null(), 1);

    let first = nvim_compl_get_first_match();
    if first.is_null() {
        if !match_count_ptr.is_null() {
            xfree(match_count_ptr.cast());
        }
        return -1;
    }

    let mut comp = first;
    loop {
        // Mark as not in match array
        nvim_compl_match_set_in_match_array(comp, 0);

        let leader_data = rs_get_leader_for_startcol_data(comp, 1);
        let leader_size = rs_get_leader_for_startcol_size(comp, 1);

        // Apply 'smartcase' behavior during normal mode
        if rs_ctrl_x_mode_normal() != 0
            && nvim_get_p_inf() == 0
            && !leader_data.is_null()
            && !nvim_ignorecase(leader_data)
            && crate::rs_cot_fuzzy() == 0
        {
            nvim_compl_match_clear_icase(comp);
        }

        if !match_at_original_text(comp)
            && (leader_data.is_null()
                || rs_ins_compl_equal(comp, leader_data, leader_size) != 0
                || (crate::rs_cot_fuzzy() != 0
                    && nvim_compl_match_get_score(comp) != FUZZY_SCORE_NONE))
        {
            // Limit number of items from each source if max_matches is set.
            let mut match_limit_exceeded = false;
            let cur_source = nvim_compl_match_get_cpt_source_idx(comp);
            if is_forward && cur_source >= 0 && is_cpt_completion && !match_count_ptr.is_null() {
                let cnt = match_count_ptr.add(cur_source as usize);
                *cnt += 1;
                let max_matches = crate::vars::nvim_get_cpt_source_cs_max_matches(cur_source);
                if max_matches > 0 && *cnt > max_matches {
                    match_limit_exceeded = true;
                }
            }

            if !match_limit_exceeded {
                array_size += 1;
                nvim_compl_match_set_in_match_array(comp, 1);
                if match_head.is_null() {
                    match_head = comp;
                } else {
                    nvim_compl_match_set_match_next(match_tail, comp);
                }
                match_tail = comp;

                if !shown_match_ok && crate::rs_cot_fuzzy() == 0 {
                    let current_shown = nvim_compl_get_shown_match();
                    if comp == current_shown || did_find_shown_match {
                        // This item is the shown match or the first displayed
                        // item after the shown match.
                        nvim_compl_set_shown_match(comp);
                        did_find_shown_match = true;
                        shown_match_ok = true;
                    } else {
                        // Remember this displayed match for when the shown
                        // match is just below it.
                        shown_compl = comp;
                    }
                    cur = i;
                } else if crate::rs_cot_fuzzy() != 0 {
                    if i == 0 {
                        shown_compl = comp;
                    }
                    let current_shown = nvim_compl_get_shown_match();
                    if !shown_match_ok && comp == current_shown {
                        cur = i;
                        shown_match_ok = true;
                    }
                }
                i += 1;
            }
        }

        if crate::rs_cot_fuzzy() == 0 {
            let current_shown = nvim_compl_get_shown_match();
            if comp == current_shown {
                did_find_shown_match = true;
                // When the original text is the shown match don't set compl_shown_match.
                if match_at_original_text(comp) {
                    shown_match_ok = true;
                }
                if !shown_match_ok && !shown_compl.is_null() {
                    // The shown match isn't displayed, set it to the previously displayed match.
                    nvim_compl_set_shown_match(shown_compl);
                    shown_match_ok = true;
                }
            }
        }

        let next = nvim_compl_match_get_next(comp);
        if next.is_null() || is_first_match(next) {
            break;
        }
        comp = next;
    }

    if !match_count_ptr.is_null() {
        xfree(match_count_ptr.cast());
    }

    if array_size == 0 {
        return -1;
    }

    crate::vars::nvim_set_compl_match_arraysize(array_size);

    if crate::rs_cot_fuzzy() != 0 && !compl_no_select && !shown_match_ok {
        nvim_compl_set_shown_match(shown_compl);
        cur = 0;
    }

    // Build the pumitem_T array via C compound accessor
    nvim_build_pum_fill_array(match_head, array_size);

    if !shown_match_ok {
        cur = -1;
    }

    cur
}

// =============================================================================
// Phase 5: rs_ins_compl_del_pum
// =============================================================================

/// Remove the popup menu if it is displayed.
///
/// Calls pum_undisplay and frees the match array.
///
/// # Safety
/// Requires valid completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_del_pum() {
    if crate::vars::nvim_get_compl_match_array_exists() == 0 {
        return;
    }
    nvim_pum_undisplay(0);
    crate::vars::nvim_xfree_compl_match_array();
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
