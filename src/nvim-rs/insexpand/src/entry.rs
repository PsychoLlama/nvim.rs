//! Main entry point support for completion.
//!
//! This module provides helper functions for the main completion entry points.
//! The core orchestration logic remains in C due to its complexity, but Rust
//! provides utilities for state checking and setup.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_was_interrupted() -> c_int;
    fn nvim_get_compl_used_match() -> c_int;
    fn nvim_get_compl_enter_selects() -> c_int;
    fn nvim_get_compl_matches() -> c_int;
    fn nvim_pum_visible() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;
const CTRL_X_SCROLL: c_int = 2;

// Completion status flags
const CONT_ADDING: c_int = 1;

// Control key constants
const CTRL_X: c_int = 24;
const CTRL_N: c_int = 14;
const CTRL_P: c_int = 16;

/// Check if we can start a new completion cycle.
///
/// Returns true if no completion is active or we're in a mode that allows restart.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_can_start() -> c_int {
    let started = nvim_get_compl_started();
    if started == 0 {
        return 1;
    }
    // Can restart if we're in adding mode
    c_int::from((nvim_get_compl_cont_status() & CONT_ADDING) != 0)
}

/// Check if we should show completion.
///
/// Returns true if there are matches and the popup should be shown.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_should_show() -> c_int {
    let started = nvim_get_compl_started();
    let matches = nvim_get_compl_matches();
    c_int::from(started != 0 && matches > 0)
}

/// Check if completion is ready to be stopped.
///
/// Returns true if completion can be safely stopped.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_can_stop() -> c_int {
    let started = nvim_get_compl_started();
    if started == 0 {
        return 0; // Nothing to stop
    }
    // Can stop if not in the middle of an operation
    c_int::from(nvim_get_compl_interrupted() == 0)
}

/// Check if we're in a state where CTRL-X is expected.
///
/// Returns true if CTRL-X mode is not defined yet or we're at normal.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_awaiting_ctrl_x() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_NORMAL || mode == CTRL_X_NOT_DEFINED_YET)
}

/// Check if we're in scroll mode.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_in_scroll_mode() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_SCROLL)
}

/// Check if completion was cancelled (user typed CTRL-E or ESC).
///
/// Returns true if completion was cancelled without selecting a match.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_was_cancelled() -> c_int {
    let started = nvim_get_compl_started();
    let used_match = nvim_get_compl_used_match();
    c_int::from(started == 0 && used_match == 0)
}

/// Check if completion finished with a selection.
///
/// Returns true if completion was accepted with a match selected.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_was_completed() -> c_int {
    let started = nvim_get_compl_started();
    let used_match = nvim_get_compl_used_match();
    c_int::from(started == 0 && used_match != 0)
}

/// Check if cursor is at the completion start column.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_cursor_at_start() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    c_int::from(cursor_col == compl_col)
}

/// Check if cursor is before the completion start column.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_cursor_before_start() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    c_int::from(cursor_col < compl_col)
}

/// Check if a key is one that navigates completion (Ctrl-N, Ctrl-P).
#[no_mangle]
pub unsafe extern "C" fn rs_entry_is_navigate_key(c: c_int) -> c_int {
    c_int::from(c == CTRL_N || c == CTRL_P)
}

/// Check if a key is Ctrl-X for entering sub-mode.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_is_ctrl_x(c: c_int) -> c_int {
    c_int::from(c == CTRL_X)
}

/// Check if completion should continue after a key press.
///
/// Returns true if the key doesn't end completion.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_should_continue(_c: c_int) -> c_int {
    // This is a placeholder - the full logic is complex and stays in C
    // This helper just checks basic conditions
    let started = nvim_get_compl_started();
    c_int::from(started != 0 && nvim_get_compl_interrupted() == 0)
}

/// Check if popup menu is visible and completion active.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_pum_active() -> c_int {
    c_int::from(nvim_get_compl_started() != 0 && nvim_pum_visible() != 0)
}

/// Check if Enter should select the current match.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_enter_selects() -> c_int {
    nvim_get_compl_enter_selects()
}

/// Get the typed length for completion.
///
/// Returns the number of characters typed since completion started.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_get_typed_len() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let len = cursor_col - compl_col;
    if len < 0 {
        0
    } else {
        len
    }
}

/// Check if completion needs to restart.
///
/// Returns true if completion was interrupted or needs refresh.
#[no_mangle]
pub unsafe extern "C" fn rs_entry_needs_restart() -> c_int {
    nvim_get_compl_was_interrupted()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_constants() {
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
        assert_eq!(CTRL_X_SCROLL, 2);
    }

    #[test]
    fn test_ctrl_key_constants() {
        assert_eq!(CTRL_X, 24);
        assert_eq!(CTRL_N, 14);
        assert_eq!(CTRL_P, 16);
    }

    #[test]
    fn test_cont_adding() {
        assert_eq!(CONT_ADDING, 1);
    }
}
