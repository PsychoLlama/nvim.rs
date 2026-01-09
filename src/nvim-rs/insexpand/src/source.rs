//! Completion source management.
//!
//! This module provides helper functions for managing completion sources
//! and the 'complete' option.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_matches() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;

/// Check if completion needs to start scanning sources.
///
/// Returns true if completion has started but no matches found yet.
#[no_mangle]
pub unsafe extern "C" fn rs_source_needs_scan() -> c_int {
    let started = nvim_get_compl_started();
    let matches = nvim_get_compl_matches();
    c_int::from(started != 0 && matches == 0)
}

/// Check if we're in initial completion mode (before CTRL-X pressed).
#[no_mangle]
pub unsafe extern "C" fn rs_source_is_initial_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_NORMAL || mode == CTRL_X_NOT_DEFINED_YET)
}

/// Check if completion has any matches.
#[no_mangle]
pub unsafe extern "C" fn rs_source_has_matches() -> c_int {
    let matches = nvim_get_compl_matches();
    c_int::from(matches > 0)
}

/// Get the current match count.
#[no_mangle]
pub unsafe extern "C" fn rs_source_match_count() -> c_int {
    nvim_get_compl_matches()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
    }
}
