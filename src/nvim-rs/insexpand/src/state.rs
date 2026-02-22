//! Completion state management.
//!
//! This module provides functions for managing and querying completion state,
//! consolidating state-related operations. Many of these are already implemented
//! in lib.rs but this module provides additional utilities and documentation.
//!
//! The completion state machine has several modes:
//! - CTRL_X_NORMAL (0): Default keyword completion (^N/^P)
//! - CTRL_X_NOT_DEFINED_YET (1): Just pressed ^X, waiting for next key
//! - CTRL_X_SCROLL (2): Scrolling without completing
//! - CTRL_X_WHOLE_LINE (3): Line completion (^X^L)
//! - CTRL_X_FILES (4): File name completion (^X^F)
//! - CTRL_X_TAGS: Tag completion (^X^])
//! - etc.

#![allow(clippy::missing_const_for_fn)]

use std::os::raw::c_int;

// C accessor functions for state
extern "C" {
    // State flag accessors
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_time_slice_expired() -> c_int;
    fn nvim_get_compl_enter_selects() -> c_int;
    fn nvim_get_compl_used_match() -> c_int;
    fn nvim_get_compl_was_interrupted() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;
    fn nvim_get_compl_restarting() -> c_int;
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_get_compl_get_longest() -> c_int;

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
    fn nvim_get_compl_cont_mode() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_WANT_IDENT: c_int = 0x100;

const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;
const CTRL_X_SCROLL: c_int = 2;
#[allow(dead_code)]
const CTRL_X_WHOLE_LINE: c_int = 3;
#[allow(dead_code)]
const CTRL_X_FILES: c_int = 4;
#[allow(dead_code)]
const CTRL_X_TAGS: c_int = 5 + CTRL_X_WANT_IDENT;
#[allow(dead_code)]
const CTRL_X_PATH_PATTERNS: c_int = 6 + CTRL_X_WANT_IDENT;
#[allow(dead_code)]
const CTRL_X_PATH_DEFINES: c_int = 7 + CTRL_X_WANT_IDENT;
const CTRL_X_FINISHED: c_int = 8;
#[allow(dead_code)]
const CTRL_X_DICTIONARY: c_int = 9 + CTRL_X_WANT_IDENT;
#[allow(dead_code)]
const CTRL_X_THESAURUS: c_int = 10 + CTRL_X_WANT_IDENT;
#[allow(dead_code)]
const CTRL_X_CMDLINE: c_int = 11;
#[allow(dead_code)]
const CTRL_X_FUNCTION: c_int = 12;
#[allow(dead_code)]
const CTRL_X_OMNI: c_int = 13;
#[allow(dead_code)]
const CTRL_X_SPELL: c_int = 14;
const CTRL_X_EVAL: c_int = 16;
const CTRL_X_CMDLINE_CTRL_X: c_int = 17;
const CTRL_X_BUFNAMES: c_int = 18;
#[allow(dead_code)]
const CTRL_X_REGISTER: c_int = 19;

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

/// FFI export: Get CONT_ADDING constant.
#[no_mangle]
pub extern "C" fn rs_cont_adding() -> c_int {
    CONT_ADDING
}

/// FFI export: Get CONT_INTRPT constant.
#[no_mangle]
pub extern "C" fn rs_cont_intrpt() -> c_int {
    CONT_INTRPT
}

/// FFI export: Get CONT_N_ADDS constant.
#[no_mangle]
pub extern "C" fn rs_cont_n_adds() -> c_int {
    CONT_N_ADDS
}

/// FFI export: Get CONT_S_IPOS constant.
#[no_mangle]
pub extern "C" fn rs_cont_s_ipos() -> c_int {
    CONT_S_IPOS
}

/// FFI export: Get CONT_SOL constant.
#[no_mangle]
pub extern "C" fn rs_cont_sol() -> c_int {
    CONT_SOL
}

/// FFI export: Get CONT_LOCAL constant.
#[no_mangle]
pub extern "C" fn rs_cont_local() -> c_int {
    CONT_LOCAL
}

/// FFI export: Get raw continuation status.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_cont_status() -> c_int {
    nvim_get_compl_cont_status()
}

/// FFI export: Get compl_enter_selects flag.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_enter_selects() -> c_int {
    nvim_get_compl_enter_selects()
}

/// FFI export: Get compl_used_match flag.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_used_match() -> c_int {
    nvim_get_compl_used_match()
}

// =============================================================================
// State Reset (ins_compl_clear)
// =============================================================================

extern "C" {
    fn nvim_set_compl_cont_status(val: c_int);
    fn nvim_set_compl_started(val: c_int);
    fn nvim_set_compl_matches(val: c_int);
    fn nvim_set_compl_selected_item(val: c_int);
    fn nvim_set_compl_ins_end_col(val: c_int);
    fn nvim_clear_compl_curr_win();
    fn nvim_clear_compl_curr_buf();
    fn nvim_compl_clear_pattern();
    fn nvim_compl_clear_leader();
    fn nvim_clear_edit_submode_extra();
    fn nvim_clear_compl_orig_extmarks();
    fn nvim_compl_clear_orig_text();
    fn nvim_set_compl_enter_selects(val: c_int);
    fn nvim_cpt_sources_clear();
    fn nvim_set_compl_autocomplete(val: c_int);
    fn nvim_set_compl_get_longest(val: c_int);
    fn nvim_set_compl_from_nonkeyword(val: c_int);
    fn nvim_set_compl_num_bests(val: c_int);
    fn nvim_set_completed_item_empty();
}

/// Clear all completion state.
///
/// Resets all global completion variables to their default values.
/// This is called when completion is finished or abandoned.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_clear() {
    nvim_set_compl_cont_status(0);
    nvim_set_compl_started(0);
    nvim_set_compl_matches(0);
    nvim_set_compl_selected_item(-1);
    nvim_set_compl_ins_end_col(0);
    nvim_clear_compl_curr_win();
    nvim_clear_compl_curr_buf();
    nvim_compl_clear_pattern();
    nvim_compl_clear_leader();
    nvim_clear_edit_submode_extra();
    nvim_clear_compl_orig_extmarks();
    nvim_compl_clear_orig_text();
    nvim_set_compl_enter_selects(0);
    nvim_cpt_sources_clear();
    nvim_set_compl_autocomplete(0);
    nvim_set_compl_from_nonkeyword(0);
    nvim_set_compl_num_bests(0);
    nvim_set_completed_item_empty();
}

/// Clear the completion status flags (compl_cont_status = 0).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_clear() {
    nvim_set_compl_cont_status(0);
}

/// Initialize get longest common string (compl_get_longest = false).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_init_get_longest() {
    nvim_set_compl_get_longest(0);
}

/// Enable autocompletion (compl_autocomplete = true, compl_get_longest = false).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_enable_autocomplete() {
    nvim_set_compl_autocomplete(1);
    nvim_set_compl_get_longest(0);
}

// =============================================================================
// Phase 1: State Machine Core Functions
// =============================================================================

/// Get the raw CTRL-X mode state value.
///
/// Returns the current ctrl_x_mode value directly.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_state() -> c_int {
    nvim_get_ctrl_x_mode()
}

/// Check if the CTRL-X mode wants an identifier.
///
/// Returns true if the current mode has the CTRL_X_WANT_IDENT flag set.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_wants_ident_current() -> c_int {
    c_int::from((nvim_get_ctrl_x_mode() & CTRL_X_WANT_IDENT) != 0)
}

/// Get the base CTRL-X mode (without the WANT_IDENT flag).
///
/// Strips the CTRL_X_WANT_IDENT flag from the current mode value.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_base_current() -> c_int {
    nvim_get_ctrl_x_mode() & !CTRL_X_WANT_IDENT
}

/// Check if completion mode is finished.
///
/// Returns true if ctrl_x_mode is CTRL_X_FINISHED.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_finished() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_FINISHED)
}

/// Check if completion mode is eval (builtin complete()).
///
/// Returns true if ctrl_x_mode is CTRL_X_EVAL.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_eval() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_EVAL)
}

/// Check if completion mode is buffer names.
///
/// Returns true if ctrl_x_mode is CTRL_X_BUFNAMES.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_bufnames() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_BUFNAMES)
}

/// Check if CTRL-X CTRL-X was typed in command-line mode.
///
/// Returns true if ctrl_x_mode is CTRL_X_CMDLINE_CTRL_X.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_cmdline_ctrl_x() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_CMDLINE_CTRL_X)
}

/// Get the continuation mode.
///
/// Returns the value of compl_cont_mode.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_cont_mode() -> c_int {
    nvim_get_compl_cont_mode()
}

/// Check if compl_get_longest is set.
///
/// Returns true if completion should find the longest common match.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_get_longest() -> c_int {
    nvim_get_compl_get_longest()
}

/// Get a summary of completion mode state.
///
/// Returns a bitfield with mode information:
/// - Bit 0: completion started
/// - Bit 1: mode is not default
/// - Bit 2: mode is finished
/// - Bit 3: mode wants ident
/// - Bit 4: mode is scroll
#[no_mangle]
pub unsafe extern "C" fn rs_compl_mode_summary() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    let mut summary = 0;

    if nvim_get_compl_started() != 0 {
        summary |= 1;
    }
    if mode != CTRL_X_NORMAL {
        summary |= 2;
    }
    if mode == CTRL_X_FINISHED {
        summary |= 4;
    }
    if (mode & CTRL_X_WANT_IDENT) != 0 {
        summary |= 8;
    }
    if mode == CTRL_X_SCROLL {
        summary |= 16;
    }

    summary
}

/// Check if completion should transition to finished state.
///
/// Returns true if we're in a non-scroll CTRL-X mode and should finish.
#[no_mangle]
pub unsafe extern "C" fn rs_should_finish_ctrl_x() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    // If we're in scroll mode, we go to NORMAL; otherwise to FINISHED
    c_int::from(mode != CTRL_X_NORMAL && mode != CTRL_X_SCROLL && mode != CTRL_X_FINISHED)
}

/// Get the next state after leaving a non-default CTRL-X mode.
///
/// Returns CTRL_X_NORMAL if in scroll mode, CTRL_X_FINISHED otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_next_ctrl_x_state_on_leave() -> c_int {
    if nvim_get_ctrl_x_mode() == CTRL_X_SCROLL {
        CTRL_X_NORMAL
    } else {
        CTRL_X_FINISHED
    }
}

/// Check if we're in a state where completion can be started.
///
/// Returns true if in CTRL_X_NOT_DEFINED_YET or normal mode without completion.
#[no_mangle]
pub unsafe extern "C" fn rs_can_start_completion() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    let started = nvim_get_compl_started() != 0;
    c_int::from(mode == CTRL_X_NOT_DEFINED_YET || (mode == CTRL_X_NORMAL && !started))
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

    #[test]
    fn test_ctrl_x_mode_constants() {
        // Verify CTRL-X mode constants match expected values
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
        assert_eq!(CTRL_X_SCROLL, 2);
        assert_eq!(CTRL_X_WHOLE_LINE, 3);
        assert_eq!(CTRL_X_FILES, 4);
        assert_eq!(CTRL_X_FINISHED, 8);
        assert_eq!(CTRL_X_CMDLINE, 11);
        assert_eq!(CTRL_X_FUNCTION, 12);
        assert_eq!(CTRL_X_OMNI, 13);
        assert_eq!(CTRL_X_SPELL, 14);
        assert_eq!(CTRL_X_EVAL, 16);
        assert_eq!(CTRL_X_CMDLINE_CTRL_X, 17);
        assert_eq!(CTRL_X_BUFNAMES, 18);
        assert_eq!(CTRL_X_REGISTER, 19);
    }

    #[test]
    fn test_ctrl_x_want_ident() {
        // Modes with CTRL_X_WANT_IDENT should have the flag set
        assert_eq!(CTRL_X_WANT_IDENT, 0x100);
        assert_ne!(CTRL_X_TAGS & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_PATH_PATTERNS & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_PATH_DEFINES & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_DICTIONARY & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_THESAURUS & CTRL_X_WANT_IDENT, 0);

        // Modes without CTRL_X_WANT_IDENT should not have the flag
        assert_eq!(CTRL_X_NORMAL & CTRL_X_WANT_IDENT, 0);
        assert_eq!(CTRL_X_SCROLL & CTRL_X_WANT_IDENT, 0);
        assert_eq!(CTRL_X_FILES & CTRL_X_WANT_IDENT, 0);
    }

    #[test]
    fn test_ctrl_x_modes_unique() {
        let modes = [
            CTRL_X_NORMAL,
            CTRL_X_NOT_DEFINED_YET,
            CTRL_X_SCROLL,
            CTRL_X_WHOLE_LINE,
            CTRL_X_FILES,
            CTRL_X_TAGS,
            CTRL_X_PATH_PATTERNS,
            CTRL_X_PATH_DEFINES,
            CTRL_X_FINISHED,
            CTRL_X_DICTIONARY,
            CTRL_X_THESAURUS,
            CTRL_X_CMDLINE,
            CTRL_X_FUNCTION,
            CTRL_X_OMNI,
            CTRL_X_SPELL,
            CTRL_X_EVAL,
            CTRL_X_CMDLINE_CTRL_X,
            CTRL_X_BUFNAMES,
            CTRL_X_REGISTER,
        ];

        for i in 0..modes.len() {
            for j in (i + 1)..modes.len() {
                assert_ne!(modes[i], modes[j], "Modes at {i} and {j} are equal");
            }
        }
    }
}
