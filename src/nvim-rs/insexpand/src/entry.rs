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
    fn rs_ctrl_x_mode_not_default() -> c_int;
    fn rs_thesaurus_func_complete(mode: c_int) -> c_int;
    fn rs_compl_status_adding() -> c_int;
    fn rs_get_normal_compl_info(line: *mut c_char, startcol: c_int, curs_col: c_int) -> c_int;
    fn rs_get_wholeline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int;
    fn rs_get_filename_compl_info(line: *mut c_char, startcol: c_int, curs_col: c_int) -> c_int;
    fn rs_get_spell_compl_info(startcol: c_int, curs_col: c_int) -> c_int;
    fn rs_get_cmdline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int;
    fn rs_ins_compl_fixRedoBufForLeader(ptr_arg: *const c_char);
    fn rs_ins_compl_continue_search(line: *mut c_char);
    fn nvim_get_userdefined_compl_info_impl(curs_col: c_int) -> c_int;
    fn nvim_internal_error_compl_get_info();

    // Accessors for ins_compl_start
    fn nvim_ins_compl_start_init_impl(save_did_ai_out: *mut c_int) -> c_int;
    fn nvim_get_compl_cont_mode() -> c_int;
    fn nvim_set_compl_cont_status(val: c_int);
    fn nvim_compl_cont_status_or(mask: c_int);
    fn nvim_set_compl_cont_mode(val: c_int);
    fn nvim_set_compl_startpos_to_cursor();
    fn nvim_set_compl_col_zero();
    fn nvim_set_compl_startpos_col_to_compl_col();
    fn nvim_restore_did_ai(saved_val: c_int);
    fn nvim_set_edit_submode_ctrl_x_local_or_mode();
    fn nvim_ins_compl_start_add_orig_impl(line: *mut c_char, save_did_ai: c_int) -> c_int;
    fn nvim_ins_compl_start_show_searching_impl();
    fn nvim_ins_compl_start_adding_eol_impl();
    fn nvim_set_edit_submode_adding();
    fn nvim_clear_edit_submode_pre();
    fn nvim_shortmess_completionmenu() -> bool;
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_ml_get_curline() -> *const c_char;
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_set_compl_pending(val: c_int);
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;
const CTRL_X_SCROLL: c_int = 2;
const CTRL_X_WANT_IDENT: c_int = 0x100;
const CTRL_X_CMDLINE: c_int = 11;

// Completion status flags
const CONT_ADDING: c_int = 1;
const CONT_INTRPT: c_int = 6; // 2 + 4
const CONT_N_ADDS: c_int = 4;
const CONT_LOCAL: c_int = 32;

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

/// Start insert mode completion.
///
/// Initializes all completion state: saves did_ai, calls stop_arrow, gets
/// line/col, handles continuation, sets pattern/original text, adds original
/// text as first match, shows status messages.
///
/// # Safety
/// Requires valid global completion state. Mutates many C static globals.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_ins_compl_start() -> c_int {
    // Block 1: init flags, stop_arrow, get line/col
    let mut save_did_ai: c_int = 0;
    if nvim_ins_compl_start_init_impl(&raw mut save_did_ai) == FAIL {
        return FAIL;
    }
    // line and curs_col are obtained via accessors
    let line = nvim_ml_get_curline().cast_mut();
    let curs_col = nvim_get_cursor_col();

    // Block 2: check for continued search or reset cont_status
    let cont_status = nvim_get_compl_cont_status();
    let cont_mode = nvim_get_compl_cont_mode();
    let ctrl_x_mode = nvim_get_ctrl_x_mode();

    if (cont_status & CONT_INTRPT) == CONT_INTRPT && cont_mode == ctrl_x_mode {
        // This same ctrl_x_mode was interrupted previously. Continue the completion.
        rs_ins_compl_continue_search(line);
    } else {
        nvim_set_compl_cont_status(cont_status & CONT_LOCAL);
    }

    // Block 3: set up startcol for normal (non-adding) expansion
    let startcol: c_int = if rs_compl_status_adding() == 0 {
        // Normal expansion
        nvim_set_compl_cont_mode(ctrl_x_mode);
        if rs_ctrl_x_mode_not_default() != 0 {
            // Remove LOCAL if ctrl_x_mode != CTRL_X_NORMAL
            nvim_set_compl_cont_status(0);
        }
        nvim_compl_cont_status_or(CONT_N_ADDS);
        nvim_set_compl_startpos_to_cursor();
        nvim_set_compl_col_zero();
        curs_col
    } else {
        0
    };

    // Block 4: get completion pattern info (may invalidate line)
    let mut line_invalid: c_int = 0;
    if rs_compl_get_info(line, startcol, curs_col, &raw mut line_invalid) == FAIL {
        if rs_ctrl_x_mode_function() != 0
            || rs_ctrl_x_mode_omni() != 0
            || rs_thesaurus_func_complete(ctrl_x_mode) != 0
        {
            // Restore did_ai so that adding comment leader works
            nvim_restore_did_ai(save_did_ai);
        }
        return FAIL;
    }

    // Refresh line pointer if it was invalidated
    let line = if line_invalid != 0 {
        nvim_ml_get_curline().cast_mut()
    } else {
        line
    };

    // Block 5: set up submode pre-text and compl_startpos for adding vs normal
    if rs_compl_status_adding() != 0 {
        if !nvim_shortmess_completionmenu() {
            nvim_set_edit_submode_adding();
        }
        if rs_ctrl_x_mode_line_or_eval() != 0 {
            // Insert a new line, keep indentation but ignore 'comments'.
            nvim_ins_compl_start_adding_eol_impl();
        }
    } else {
        nvim_clear_edit_submode_pre();
        nvim_set_compl_startpos_col_to_compl_col();
    }

    // Block 6: set edit_submode to the CTRL-X mode message
    if !nvim_shortmess_completionmenu() && nvim_get_compl_autocomplete() == 0 {
        nvim_set_edit_submode_ctrl_x_local_or_mode();
    }

    // Block 7: fix redo buffer for leader
    rs_ins_compl_fixRedoBufForLeader(std::ptr::null());

    // Block 8: add the original text as the first completion match
    if nvim_ins_compl_start_add_orig_impl(line, save_did_ai) == FAIL {
        return FAIL;
    }

    // Block 9: show "Searching..." status message
    if !nvim_shortmess_completionmenu() && nvim_get_compl_autocomplete() == 0 {
        nvim_ins_compl_start_show_searching_impl();
    }

    nvim_restore_did_ai(save_did_ai);
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

    #[test]
    fn test_cont_flags() {
        assert_eq!(CONT_INTRPT, 6);
        assert_eq!(CONT_N_ADDS, 4);
        assert_eq!(CONT_LOCAL, 32);
    }
}
