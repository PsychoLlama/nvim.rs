//! Insert-mode completion (CTRL-X) mode checking for Neovim
//!
//! This module provides Rust implementations for checking the current CTRL-X
//! completion mode in insert mode.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_int;

// CTRL-X mode constants (from insexpand.c)
// These must match the C enum values exactly
const CTRL_X_WANT_IDENT: c_int = 0x100;

const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;
const CTRL_X_SCROLL: c_int = 2;
const CTRL_X_WHOLE_LINE: c_int = 3;
const CTRL_X_FILES: c_int = 4;
const CTRL_X_TAGS: c_int = 5 + CTRL_X_WANT_IDENT;
const CTRL_X_PATH_PATTERNS: c_int = 6 + CTRL_X_WANT_IDENT;
const CTRL_X_PATH_DEFINES: c_int = 7 + CTRL_X_WANT_IDENT;
const CTRL_X_FINISHED: c_int = 8;
const CTRL_X_DICTIONARY: c_int = 9 + CTRL_X_WANT_IDENT;
const CTRL_X_THESAURUS: c_int = 10 + CTRL_X_WANT_IDENT;
const CTRL_X_CMDLINE: c_int = 11;
const CTRL_X_FUNCTION: c_int = 12;
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_SPELL: c_int = 14;
// const CTRL_X_LOCAL_MSG: c_int = 15; // only used in ctrl_x_msgs
const CTRL_X_EVAL: c_int = 16;
const CTRL_X_CMDLINE_CTRL_X: c_int = 17;
// const CTRL_X_BUFNAMES: c_int = 18;
const CTRL_X_REGISTER: c_int = 19;

// Completion status flags (from insexpand.c)
const CONT_ADDING: c_int = 1;
const CONT_SOL: c_int = 16;
const CONT_LOCAL: c_int = 32;

// C accessors for static variables
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;
}

/// Check if CTRL-X mode is none (0).
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_none() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == 0)
}

/// Check if CTRL-X mode is normal (CTRL_X_NORMAL).
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_normal() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_NORMAL)
}

/// Check if CTRL-X mode is scroll.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_scroll() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_SCROLL)
}

/// Check if CTRL-X mode is whole line completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_whole_line() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_WHOLE_LINE)
}

/// Check if CTRL-X mode is file name completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_files() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_FILES)
}

/// Check if CTRL-X mode is tag completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_tags() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_TAGS)
}

/// Check if CTRL-X mode is path pattern completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_path_patterns() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_PATH_PATTERNS)
}

/// Check if CTRL-X mode is path defines completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_path_defines() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_PATH_DEFINES)
}

/// Check if CTRL-X mode is dictionary completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_dictionary() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_DICTIONARY)
}

/// Check if CTRL-X mode is thesaurus completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_thesaurus() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_THESAURUS)
}

/// Check if CTRL-X mode is command-line completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_cmdline() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_CMDLINE || mode == CTRL_X_CMDLINE_CTRL_X)
}

/// Check if CTRL-X mode is user-defined function completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_function() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_FUNCTION)
}

/// Check if CTRL-X mode is omni completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_omni() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_OMNI)
}

/// Check if CTRL-X mode is spell completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_spell() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_SPELL)
}

/// Check if CTRL-X mode is whole line or eval completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_line_or_eval() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_WHOLE_LINE || mode == CTRL_X_EVAL)
}

/// Check if CTRL-X mode is register completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_register() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_REGISTER)
}

/// Check if other than default completion has been selected.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_not_default() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() != CTRL_X_NORMAL)
}

/// Check if CTRL-X was typed without a following character.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_not_defined_yet() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_NOT_DEFINED_YET)
}

// =============================================================================
// Completion status functions
// =============================================================================

/// Check if in "normal" or "adding" completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_adding() -> c_int {
    c_int::from((nvim_get_compl_cont_status() & CONT_ADDING) != 0)
}

/// Check if completion pattern includes start of line.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_sol() -> c_int {
    c_int::from((nvim_get_compl_cont_status() & CONT_SOL) != 0)
}

/// Check if ^X^P/^X^N will do local completion.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_local() -> c_int {
    c_int::from((nvim_get_compl_cont_status() & CONT_LOCAL) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify the constants match expected values
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_SCROLL, 2);
        assert_eq!(CTRL_X_TAGS, 5 + 0x100); // 261
        assert_eq!(CTRL_X_DICTIONARY, 9 + 0x100); // 265
    }
}
