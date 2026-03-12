//! Main entry point support for completion.
//!
//! This module provides helper functions for the main completion entry points.
//! The core orchestration logic remains in C due to its complexity, but Rust
//! provides utilities for state checking and setup.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

use nvim_window::WinHandle;

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
    fn rs_get_userdefined_compl_info(
        curs_col: c_int,
        cb_opaque: *mut std::ffi::c_void,
        startcol_out: *mut c_int,
    ) -> c_int;
    fn nvim_internal_error_compl_get_info();

    // Accessors for ins_compl_start (Phase 10)
    fn nvim_get_did_ai() -> bool;
    fn nvim_set_did_ai(val: bool);
    fn nvim_clear_indent_flags();
    fn nvim_set_compl_lnum_to_cursor();
    fn nvim_ins_eol_wrap(c: c_int);
    fn nvim_get_curbuf_b_p_com() -> *const c_char;
    fn nvim_set_curbuf_b_p_com_empty();
    fn nvim_restore_curbuf_b_p_com(old_val: *const c_char);
    fn nvim_set_compl_startpos_lnum_col(lnum_to_cursor: c_int, col: c_int);
    fn nvim_set_compl_orig_text_from_line(line: *const c_char);
    fn nvim_ins_compl_add_orig_text(flags: c_int, save_did_ai: c_int) -> c_int;
    fn rs_save_orig_extmarks();
    fn nvim_set_edit_submode_extra_searching();
    fn nvim_showmode_wrap();
    fn nvim_set_compl_col(val: c_int);
    fn nvim_set_compl_length(val: c_int);
    fn nvim_get_compl_cont_mode() -> c_int;
    fn nvim_set_compl_cont_status(val: c_int);
    fn nvim_compl_cont_status_or(mask: c_int);
    fn nvim_set_compl_cont_mode(val: c_int);
    fn nvim_set_compl_startpos_to_cursor();
    fn nvim_set_compl_col_zero();
    fn nvim_set_compl_startpos_col_to_compl_col();
    fn nvim_restore_did_ai(saved_val: c_int);
    fn nvim_set_edit_submode_ctrl_x_local_or_mode();
    fn nvim_set_edit_submode_adding();
    fn nvim_clear_edit_submode_pre();
    fn nvim_set_edit_submode_highl_count();
    fn nvim_clear_edit_submode_extra();
    fn nvim_shortmess_completionmenu() -> bool;
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_ml_get_curline() -> *const c_char;
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_set_compl_pending(val: c_int);
    fn nvim_get_p_ic() -> c_int;
}

// CP flags (must match C enum)
const CP_ORIGINAL_TEXT: c_int = 1;
const CP_ICASE: c_int = 16;

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
        if rs_get_userdefined_compl_info(curs_col, std::ptr::null_mut(), std::ptr::null_mut()) != OK
        {
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
    // (was nvim_ins_compl_start_init_impl; inlined here in Phase 10)
    let save_did_ai: bool = nvim_get_did_ai();
    nvim_set_did_ai(false);
    nvim_clear_indent_flags();
    if nvim_stop_arrow() == FAIL {
        nvim_restore_did_ai(c_int::from(save_did_ai));
        return FAIL;
    }
    nvim_set_compl_pending(0);
    nvim_set_compl_lnum_to_cursor();
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
            nvim_restore_did_ai(c_int::from(save_did_ai));
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
            // (was nvim_ins_compl_start_adding_eol_impl; inlined here in Phase 10)
            let old_b_p_com = nvim_get_curbuf_b_p_com();
            nvim_set_curbuf_b_p_com_empty();
            let compl_col = nvim_get_compl_col();
            nvim_set_compl_startpos_lnum_col(1, compl_col);
            nvim_ins_eol_wrap(c_int::from(b'\r'));
            nvim_restore_curbuf_b_p_com(old_b_p_com);
            nvim_set_compl_length(0);
            nvim_set_compl_col(nvim_get_cursor_col());
            nvim_set_compl_lnum_to_cursor();
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
    // (was nvim_ins_compl_start_add_orig_impl; inlined here in Phase 10)
    nvim_set_compl_orig_text_from_line(line);
    rs_save_orig_extmarks();
    let mut orig_flags: c_int = CP_ORIGINAL_TEXT;
    if nvim_get_p_ic() != 0 {
        orig_flags |= CP_ICASE;
    }
    if nvim_ins_compl_add_orig_text(orig_flags, c_int::from(save_did_ai)) == FAIL {
        return FAIL;
    }

    // Block 9: show "Searching..." status message
    // (was nvim_ins_compl_start_show_searching_impl; inlined here in Phase 10)
    if !nvim_shortmess_completionmenu() && nvim_get_compl_autocomplete() == 0 {
        nvim_set_edit_submode_extra_searching();
        nvim_set_edit_submode_highl_count();
        nvim_showmode_wrap();
        nvim_clear_edit_submode_extra();
        nvim_ui_flush();
    }

    nvim_restore_did_ai(c_int::from(save_did_ai));
    OK
}

// Additional extern declarations for rs_ins_complete
extern "C" {
    fn nvim_get_compl_length() -> c_int;
    fn rs_ins_compl_key2dir(c: c_int) -> c_int;
    fn rs_ins_compl_use_match(c: c_int) -> c_int;
    fn rs_ins_compl_pum_key(c: c_int) -> c_int;
    fn rs_ins_compl_key2count(c: c_int) -> c_int;
    fn nvim_stop_arrow() -> c_int;
    fn nvim_get_p_acl() -> c_int;
    fn nvim_os_hrtime() -> u64;
    fn nvim_ins_complete_setup_match_state(direction: c_int);
    fn nvim_get_curwin_w_wrow() -> c_int;
    fn nvim_get_curwin_w_leftcol() -> c_int;
    fn nvim_ins_compl_next_wrap_ret(
        allow_get_expansion: c_int,
        todo: c_int,
        advance: c_int,
    ) -> c_int;
    fn nvim_set_compl_matches(val: c_int);
    fn nvim_compl_set_curr_to_shown();
    fn nvim_set_compl_direction(val: c_int);
    fn nvim_get_compl_shows_dir() -> c_int;
    fn nvim_ins_complete_eat_got_int();
    fn nvim_compl_first_match_next_is_first() -> c_int;
    fn nvim_compl_cont_status_remove_n_adds();
    fn rs_ctrl_x_mode_path_patterns() -> c_int;
    fn rs_ctrl_x_mode_path_defines() -> c_int;
    fn nvim_ins_complete_update_cont_s_ipos();
    fn rs_ins_compl_show_statusmsg();
    fn nvim_setcursor();
    fn nvim_ui_flush();
    fn nvim_char_avail() -> c_int;
    fn rs_ins_compl_preinsert_effect() -> c_int;
    // rs_ins_compl_win_active and nvim_get_curwin use *mut u8 (opaque pointer)
    fn rs_ins_compl_win_active(wp: *mut u8) -> c_int;
    fn nvim_get_curwin() -> *mut u8;
    fn rs_ins_compl_delete(new_leader: c_int);
    fn rs_ins_compl_restart();
    fn nvim_set_compl_interrupted(val: c_int);
    fn nvim_set_compl_ins_end_col_to_compl_col();
    // nvim_os_delay: ms is c_long, allow_input is bool
    fn nvim_os_delay(ms: std::os::raw::c_long, allow_input: bool);
    fn nvim_ins_complete_finish_interrupted();
    fn rs_show_pum(prev_w_wrow: c_int, prev_w_leftcol: c_int);
}

// Additional key constants for ins_complete
const CTRL_R: c_int = 18;
const CONT_S_IPOS: c_int = 8;

/// Do Insert mode completion.
///
/// Called when character `c` was typed which has a meaning for completion.
/// Returns OK if completion was done, FAIL if something failed.
///
/// # Safety
/// Requires valid global completion state. Mutates many C static globals.
#[allow(clippy::must_use_candidate)]
#[allow(clippy::too_many_lines)]
#[export_name = "ins_complete"]
pub unsafe extern "C" fn rs_ins_complete(c: c_int, enable_pum: c_int) -> c_int {
    // Compute disable_ac_delay: disable autocomplete delay if already started
    // and key is a regular forward/backward completion key or pum key.
    let compl_started = nvim_get_compl_started() != 0;
    let disable_ac_delay = compl_started
        && rs_ctrl_x_mode_normal() != 0
        && (c == CTRL_N || c == CTRL_P || c == CTRL_R || rs_ins_compl_pum_key(c) != 0);

    // Set direction and insert_match from the key
    let direction = rs_ins_compl_key2dir(c);
    nvim_set_compl_direction(direction);
    let insert_match = rs_ins_compl_use_match(c);

    // Start completion if not already started; otherwise call stop_arrow if inserting
    if !compl_started {
        if rs_ins_compl_start() == FAIL {
            return FAIL;
        }
    } else if insert_match != 0 && nvim_stop_arrow() == FAIL {
        return FAIL;
    }

    // Set up timestamp for autocomplete delay
    let compl_start_tv: u64 =
        if nvim_get_compl_autocomplete() != 0 && nvim_get_p_acl() > 0 && !disable_ac_delay {
            nvim_os_hrtime()
        } else {
            0
        };

    // Set up completion window/buffer/match/direction state
    nvim_ins_complete_setup_match_state(direction);

    // Find next match (and following matches)
    let save_w_wrow = nvim_get_curwin_w_wrow();
    let save_w_leftcol = nvim_get_curwin_w_leftcol();
    let n = nvim_ins_compl_next_wrap_ret(1, rs_ins_compl_key2count(c), insert_match);

    if n > 1 {
        // All matches have been found
        nvim_set_compl_matches(n);
    }
    nvim_compl_set_curr_to_shown();
    nvim_set_compl_direction(nvim_get_compl_shows_dir());

    // Eat the ESC that vgetc() returns after a CTRL-C to avoid leaving Insert mode
    nvim_ins_complete_eat_got_int();

    // Check if no matches found (list has only the compl_orig_text entry)
    let no_matches_found = nvim_compl_first_match_next_is_first() != 0;
    if no_matches_found {
        // Remove N_ADDS flag so next ^X<> won't try to go to ADDING mode,
        // unless we might want to add-expand a single-char-word.
        let compl_length = nvim_get_compl_length();
        if compl_length > 1
            || rs_compl_status_adding() != 0
            || (rs_ctrl_x_mode_not_default() != 0
                && rs_ctrl_x_mode_path_patterns() == 0
                && rs_ctrl_x_mode_path_defines() == 0)
        {
            nvim_compl_cont_status_remove_n_adds();
        }
    }

    // Update CONT_S_IPOS based on current match flags
    nvim_ins_complete_update_cont_s_ipos();

    // Show status message if appropriate
    if !nvim_shortmess_completionmenu() && nvim_get_compl_autocomplete() == 0 {
        rs_ins_compl_show_statusmsg();
    }

    // Wait for the autocompletion delay to expire
    let p_acl = nvim_get_p_acl();
    #[allow(clippy::cast_sign_loss)]
    let p_acl_ms: u64 = if p_acl > 0 { p_acl as u64 } else { 0 };
    if nvim_get_compl_autocomplete() != 0
        && p_acl > 0
        && !disable_ac_delay
        && !no_matches_found
        && (nvim_os_hrtime() - compl_start_tv) / 1_000_000 < p_acl_ms
    {
        nvim_setcursor();
        nvim_ui_flush();
        loop {
            if nvim_char_avail() != 0 {
                if rs_ins_compl_preinsert_effect() != 0
                    && rs_ins_compl_win_active(nvim_get_curwin()) != 0
                {
                    rs_ins_compl_delete(0); // Remove pre-inserted text
                    nvim_set_compl_ins_end_col_to_compl_col();
                }
                rs_ins_compl_restart();
                nvim_set_compl_interrupted(1);
                break;
            }
            nvim_os_delay(2, false);
            if (nvim_os_hrtime() - compl_start_tv) / 1_000_000 >= p_acl_ms {
                break;
            }
        }
    }

    // Show the popup menu, unless we got interrupted
    if enable_pum != 0 && nvim_get_compl_interrupted() == 0 {
        rs_show_pum(save_w_wrow, save_w_leftcol);
    }
    nvim_ins_complete_finish_interrupted();

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
