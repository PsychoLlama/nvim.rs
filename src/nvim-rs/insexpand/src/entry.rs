//! Main entry point support for completion.
//!
//! This module provides helper functions for the main completion entry points.
//! The core orchestration logic remains in C due to its complexity, but Rust
//! provides utilities for state checking and setup.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

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

    // Dispatch helpers for compl_get_info
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn rs_ctrl_x_mode_register() -> c_int;
    fn rs_ctrl_x_mode_line_or_eval() -> c_int;
    fn rs_ctrl_x_mode_files() -> c_int;
    fn rs_ctrl_x_mode_function() -> c_int;
    fn rs_ctrl_x_mode_omni() -> c_int;
    fn rs_ctrl_x_mode_spell() -> c_int;
    fn rs_thesaurus_func_complete(mode: c_int) -> c_int;
    fn rs_get_normal_compl_info(line: *mut c_char, startcol: c_int, curs_col: c_int) -> c_int;
    fn rs_get_wholeline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int;
    fn rs_get_filename_compl_info(line: *mut c_char, startcol: c_int, curs_col: c_int) -> c_int;
    fn rs_get_spell_compl_info(startcol: c_int, curs_col: c_int) -> c_int;
    fn rs_get_cmdline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int;
    fn nvim_get_userdefined_compl_info_impl(curs_col: c_int) -> c_int;
    fn nvim_internal_error_compl_get_info();
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;
const CTRL_X_SCROLL: c_int = 2;
const CTRL_X_WANT_IDENT: c_int = 0x100;
const CTRL_X_CMDLINE: c_int = 11;

// Completion status flags
const CONT_ADDING: c_int = 1;

// Control key constants
const CTRL_X: c_int = 24;
const CTRL_N: c_int = 14;
const CTRL_P: c_int = 16;

// Return values
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Get the completion pattern, column and length.
///
/// Dispatches to the appropriate info-getter based on `ctrl_x_mode`.
/// On return, `*line_invalid` is set to 1 if the current line may have become
/// invalid and needs to be fetched again.
///
/// # Safety
/// `line` must be a valid C string. `line_invalid` must be a valid pointer.
/// Requires valid global completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_get_info(
    line: *mut c_char,
    startcol: c_int,
    curs_col: c_int,
    line_invalid: *mut c_int,
) -> c_int {
    let ctrl_x_mode = nvim_get_ctrl_x_mode();

    if rs_ctrl_x_mode_normal() != 0
        || rs_ctrl_x_mode_register() != 0
        || ((ctrl_x_mode & CTRL_X_WANT_IDENT) != 0 && rs_thesaurus_func_complete(ctrl_x_mode) == 0)
    {
        if rs_get_normal_compl_info(line, startcol, curs_col) != OK {
            return FAIL;
        }
        // 'cpt' func may have invalidated "line"
        *line_invalid = 1;
    } else if rs_ctrl_x_mode_line_or_eval() != 0 {
        return rs_get_wholeline_compl_info(line, curs_col);
    } else if rs_ctrl_x_mode_files() != 0 {
        return rs_get_filename_compl_info(line, startcol, curs_col);
    } else if ctrl_x_mode == CTRL_X_CMDLINE {
        return rs_get_cmdline_compl_info(line, curs_col);
    } else if rs_ctrl_x_mode_function() != 0
        || rs_ctrl_x_mode_omni() != 0
        || rs_thesaurus_func_complete(ctrl_x_mode) != 0
    {
        if nvim_get_userdefined_compl_info_impl(curs_col) != OK {
            return FAIL;
        }
        // "line" may have become invalid
        *line_invalid = 1;
    } else if rs_ctrl_x_mode_spell() != 0 {
        if rs_get_spell_compl_info(startcol, curs_col) == FAIL {
            return FAIL;
        }
        // "line" may have become invalid
        *line_invalid = 1;
    } else {
        nvim_internal_error_compl_get_info();
        return FAIL;
    }

    OK
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
