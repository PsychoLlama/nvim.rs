//! Completion state management.
//!
//! This module provides functions for managing and querying completion state,
//! consolidating state-related operations. Many of these are already implemented
//! in lib.rs but this module provides additional utilities and documentation.

use std::os::raw::c_int;

// C accessor functions for state
extern "C" {
    // State flag accessors
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_time_slice_expired() -> c_int;
    fn nvim_get_compl_enter_selects() -> c_int;
    fn nvim_get_compl_used_match() -> c_int;
    fn nvim_get_compl_was_interrupted() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;
    fn nvim_get_compl_restarting() -> c_int;
    fn nvim_get_compl_autocomplete() -> c_int;

    // Match state
    fn nvim_compl_first_match_is_null() -> c_int;
    fn nvim_compl_curr_match_is_null() -> c_int;
    fn nvim_compl_shown_match_exists() -> c_int;
    fn nvim_compl_shown_match_is_singular() -> c_int;
    fn nvim_compl_shown_match_is_first() -> c_int;

    // Numeric state
    fn nvim_get_compl_matches() -> c_int;
    fn nvim_get_compl_length() -> c_int;
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_compl_selected_item() -> c_int;
}

// Continuation status flags (must match C defines)
const CONT_ADDING: c_int = 1;
#[allow(dead_code)]
const CONT_INTRPT: c_int = 6; // 2 + 4
#[allow(dead_code)]
const CONT_N_ADDS: c_int = 4;
#[allow(dead_code)]
const CONT_S_IPOS: c_int = 8;
const CONT_SOL: c_int = 16;
const CONT_LOCAL: c_int = 32;

/// Check if completion has been started.
///
/// Returns true if compl_started is set.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_started() -> c_int {
    nvim_get_compl_started()
}

/// Check if completion was interrupted by user input.
///
/// Returns true if compl_interrupted or compl_time_slice_expired is set.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_was_interrupted_full() -> c_int {
    c_int::from(nvim_get_compl_interrupted() != 0 || nvim_get_compl_time_slice_expired() != 0)
}

/// Check if the completion is in "adding" mode.
///
/// In adding mode, new matches are added to the existing list
/// rather than replacing it.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_adding_check() -> c_int {
    c_int::from((nvim_get_compl_cont_status() & CONT_ADDING) != 0)
}

/// Check if the completion pattern includes start of line.
///
/// Used for word-wise expansion.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_sol_check() -> c_int {
    c_int::from((nvim_get_compl_cont_status() & CONT_SOL) != 0)
}

/// Check if local completion is active.
///
/// When true, ^X^P/^X^N will do local expansion.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_local_check() -> c_int {
    c_int::from((nvim_get_compl_cont_status() & CONT_LOCAL) != 0)
}

/// Get the total number of completion matches.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_match_count() -> c_int {
    nvim_get_compl_matches()
}

/// Get the length of the text being completed.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_text_length() -> c_int {
    nvim_get_compl_length()
}

/// Get the column where completion text starts.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_start_col() -> c_int {
    nvim_get_compl_col()
}

/// Get the currently selected item index in the popup menu.
///
/// Returns -1 if no item is selected.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_selected() -> c_int {
    nvim_get_compl_selected_item()
}

/// Check if there are any completion matches.
#[no_mangle]
pub unsafe extern "C" fn rs_has_compl_matches() -> c_int {
    c_int::from(nvim_compl_first_match_is_null() == 0)
}

/// Check if there's a current match selected.
#[no_mangle]
pub unsafe extern "C" fn rs_has_compl_curr_match() -> c_int {
    c_int::from(nvim_compl_curr_match_is_null() == 0)
}

/// Check if completion is restarting (not inserting match).
#[no_mangle]
pub unsafe extern "C" fn rs_compl_is_restarting() -> c_int {
    nvim_get_compl_restarting()
}

/// Check if autocomplete mode is active.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_is_autocomplete() -> c_int {
    nvim_get_compl_autocomplete()
}

/// Check if a shown match exists and is selectable.
///
/// Returns true if:
/// - compl_shown_match exists, AND
/// - it's not the first match (original text entry)
#[no_mangle]
pub unsafe extern "C" fn rs_has_selectable_shown_match() -> c_int {
    c_int::from(nvim_compl_shown_match_exists() != 0 && nvim_compl_shown_match_is_first() == 0)
}

/// Check if the shown match is singular (only one match).
#[no_mangle]
pub unsafe extern "C" fn rs_shown_match_is_singular() -> c_int {
    nvim_compl_shown_match_is_singular()
}

/// Get completion state summary as a bitfield.
///
/// Bits:
/// - 0: compl_started
/// - 1: compl_enter_selects
/// - 2: compl_used_match
/// - 3: compl_was_interrupted
/// - 4: compl_autocomplete
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_state_flags() -> c_int {
    let mut flags = 0;
    if nvim_get_compl_started() != 0 {
        flags |= 1;
    }
    if nvim_get_compl_enter_selects() != 0 {
        flags |= 2;
    }
    if nvim_get_compl_used_match() != 0 {
        flags |= 4;
    }
    if nvim_get_compl_was_interrupted() != 0 {
        flags |= 8;
    }
    if nvim_get_compl_autocomplete() != 0 {
        flags |= 16;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cont_flags() {
        assert_eq!(CONT_ADDING, 1);
        assert_eq!(CONT_INTRPT, 6);
        assert_eq!(CONT_N_ADDS, 4);
        assert_eq!(CONT_S_IPOS, 8);
        assert_eq!(CONT_SOL, 16);
        assert_eq!(CONT_LOCAL, 32);
    }

    #[test]
    fn test_cont_flags_are_distinct() {
        // Flags should not overlap (except CONT_INTRPT which is 2 + 4)
        assert_eq!(CONT_ADDING & CONT_SOL, 0);
        assert_eq!(CONT_ADDING & CONT_LOCAL, 0);
        assert_eq!(CONT_SOL & CONT_LOCAL, 0);
    }
}
