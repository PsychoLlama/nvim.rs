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

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::os::raw::c_int;

// C accessor functions for state

// =============================================================================
// Phase 2: Scalar state variables moved from C to Rust (Phase 1 of plan)
// Previously held in insexpand_shim.c as C static variables.
// =============================================================================

/// Whether to highlight match on autocompl longest (compl_hi_on_autocompl_longest).
pub(crate) static mut COMPL_HI_ON_AUTOCOMPL_LONGEST: bool = false;

/// Number of pending completions (compl_pending).
pub(crate) static mut COMPL_PENDING: c_int = 0;

/// Don't insert match, completion is restarting (compl_restarting).
pub(crate) static mut COMPL_RESTARTING: bool = false;

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

// =============================================================================
// State Reset (ins_compl_clear)
// =============================================================================

extern "C" {
    fn nvim_compl_clear_pattern();
    #[link_name = "edit_submode_extra"]
    static mut g_edit_submode_extra: *mut c_char;
    #[link_name = "edit_submode_highl"]
    static mut g_edit_submode_highl: c_int;
    fn nvim_clear_compl_orig_extmarks();
    fn nvim_cpt_sources_clear();
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
    crate::vars::nvim_set_compl_cont_status(0);
    crate::vars::nvim_set_compl_started(0);
    crate::vars::nvim_set_compl_matches(0);
    crate::vars::nvim_set_compl_selected_item(-1);
    crate::vars::nvim_set_compl_ins_end_col(0);
    crate::vars::nvim_clear_compl_curr_win();
    crate::vars::nvim_clear_compl_curr_buf();
    nvim_compl_clear_pattern();
    crate::vars::nvim_compl_clear_leader();
    g_edit_submode_extra = core::ptr::null_mut();
    nvim_clear_compl_orig_extmarks();
    crate::vars::nvim_compl_clear_orig_text();
    crate::vars::nvim_set_compl_enter_selects(0);
    nvim_cpt_sources_clear();
    crate::vars::nvim_set_compl_autocomplete(0);
    crate::vars::nvim_set_compl_from_nonkeyword(0);
    crate::vars::nvim_set_compl_num_bests(0);
    nvim_set_completed_item_empty();
}

/// Clear the completion status flags (compl_cont_status = 0).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_status_clear() {
    crate::vars::nvim_set_compl_cont_status(0);
}

/// Initialize get longest common string (compl_get_longest = false).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_init_get_longest() {
    crate::vars::nvim_set_compl_get_longest(0);
}

/// Enable autocompletion (compl_autocomplete = true, compl_get_longest = false).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_enable_autocomplete() {
    crate::vars::nvim_set_compl_autocomplete(1);
    crate::vars::nvim_set_compl_get_longest(0);
}

// =============================================================================
// Phase 1: State Machine Core Functions
// =============================================================================

/// Check if completion mode is eval (builtin complete()).
///
/// Returns true if ctrl_x_mode is CTRL_X_EVAL.
#[no_mangle]
pub unsafe extern "C" fn rs_ctrl_x_mode_eval() -> c_int {
    c_int::from(crate::vars::nvim_get_ctrl_x_mode() == CTRL_X_EVAL)
}

// =============================================================================
// Phase 4: rs_ins_compl_restart
// =============================================================================

extern "C" {
    fn nvim_update_screen();
    fn rs_ins_compl_free();
}

/// Setup for finding completions again without leaving CTRL-X mode.
///
/// Used when BS or a key was typed while still searching for matches.
/// Updates screen, frees completion data, and resets all state counters.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_restart() {
    // update screen before restart so if complete is blocked,
    // will stay to the last popup menu and reduce flicker
    nvim_update_screen();
    rs_ins_compl_free();
    crate::vars::nvim_set_compl_started(0);
    crate::vars::nvim_set_compl_matches(0);
    crate::vars::nvim_set_compl_cont_status(0);
    crate::vars::nvim_set_compl_cont_mode(0);
    nvim_cpt_sources_clear();
    crate::vars::nvim_set_compl_autocomplete(0);
    crate::vars::nvim_set_compl_from_nonkeyword(0);
    crate::vars::nvim_set_compl_num_bests(0);
}

// =============================================================================
// Phase 1: ins_compl_mode and thesaurus_func_complete
// =============================================================================

// ctrl_x_mode_names[] indexed by (ctrl_x_mode & ~CTRL_X_WANT_IDENT)
// Must match the C static array definition exactly.
// NULL entries are represented as empty strings since they shouldn't be returned.
static CTRL_X_MODE_NAMES: [Option<&'static [u8]>; 20] = [
    Some(b"keyword"),       // 0  CTRL_X_NORMAL
    Some(b"ctrl_x"),        // 1  CTRL_X_NOT_DEFINED_YET
    Some(b"scroll"),        // 2  CTRL_X_SCROLL
    Some(b"whole_line"),    // 3  CTRL_X_WHOLE_LINE
    Some(b"files"),         // 4  CTRL_X_FILES
    Some(b"tags"),          // 5  CTRL_X_TAGS (base)
    Some(b"path_patterns"), // 6  CTRL_X_PATH_PATTERNS (base)
    Some(b"path_defines"),  // 7  CTRL_X_PATH_DEFINES (base)
    Some(b"unknown"),       // 8  CTRL_X_FINISHED
    Some(b"dictionary"),    // 9  CTRL_X_DICTIONARY (base)
    Some(b"thesaurus"),     // 10 CTRL_X_THESAURUS (base)
    Some(b"cmdline"),       // 11 CTRL_X_CMDLINE
    Some(b"function"),      // 12 CTRL_X_FUNCTION
    Some(b"omni"),          // 13 CTRL_X_OMNI
    Some(b"spell"),         // 14 CTRL_X_SPELL
    None,                   // 15 CTRL_X_LOCAL_MSG (NULL in C)
    Some(b"eval"),          // 16 CTRL_X_EVAL
    Some(b"cmdline"),       // 17 CTRL_X_CMDLINE_CTRL_X
    None,                   // 18 CTRL_X_BUFNAMES (NULL in C)
    Some(b"register"),      // 19 CTRL_X_REGISTER
];

use std::os::raw::c_char;

extern "C" {
    // New accessors for Phase 1
    fn nvim_get_p_tsrfu_nonempty() -> c_int;
    fn nvim_get_curbuf_b_p_tsrfu_nonempty() -> c_int;
    // Compound accessors for complex functions
    fn nvim_get_next_include_file_completion(compl_type: c_int);
    fn nvim_get_next_cmdline_completion_impl();
    fn nvim_get_next_spell_completion_impl(lnum: c_int);
    fn nvim_do_autocmd_completedone_impl(c: c_int, mode: c_int, word: *const c_char);
    fn nvim_ins_compl_show_filename_impl();
}

/// Return the Insert completion mode name string.
///
/// Returns a pointer to a static NUL-terminated string, or a pointer to an
/// empty string if no mode is active. Never returns NULL.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_mode() -> *const c_char {
    let mode = crate::vars::nvim_get_ctrl_x_mode();
    let started = crate::vars::nvim_get_compl_started();

    // Check conditions: not-defined-yet, scroll, or compl_started
    let not_defined_yet = mode == CTRL_X_NOT_DEFINED_YET;
    let is_scroll = mode == CTRL_X_SCROLL;

    if not_defined_yet || is_scroll || started != 0 {
        // Index into mode names array: mode & ~CTRL_X_WANT_IDENT
        let masked = mode & !CTRL_X_WANT_IDENT;
        if masked >= 0 {
            #[allow(clippy::cast_sign_loss)]
            let idx = masked as usize;
            if idx < CTRL_X_MODE_NAMES.len() {
                if let Some(name) = CTRL_X_MODE_NAMES[idx] {
                    // Return a pointer to the static NUL-terminated byte string
                    return name.as_ptr().cast::<c_char>();
                }
            }
        }
    }

    // Return pointer to a static empty string (not NULL)
    c"".as_ptr()
}

/// Returns true when using a user-defined function for thesaurus completion.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_thesaurus_func_complete(compl_type: c_int) -> c_int {
    c_int::from(
        compl_type == CTRL_X_THESAURUS
            && (nvim_get_curbuf_b_p_tsrfu_nonempty() != 0 || nvim_get_p_tsrfu_nonempty() != 0),
    )
}

/// Get the next set of identifiers or defines matching compl_pattern in included files.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_include_file_completion(compl_type: c_int) {
    nvim_get_next_include_file_completion(compl_type);
}

/// Get the next set of command-line completions matching compl_pattern.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_cmdline_completion() {
    nvim_get_next_cmdline_completion_impl();
}

/// Get the next set of spell suggestions matching compl_pattern.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_spell_completion(lnum: c_int) {
    nvim_get_next_spell_completion_impl(lnum);
}

/// Build v_event dict and fire EVENT_COMPLETEDONE autocmd.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_do_autocmd_completedone(c: c_int, mode: c_int, word: *const c_char) {
    nvim_do_autocmd_completedone_impl(c, mode, word);
}

/// Show the file name for the completion match (if any).
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_show_filename() {
    nvim_ins_compl_show_filename_impl();
}

// =============================================================================
// Phase 10 (pass 10): ins_compl_continue_search and ins_compl_show_statusmsg
// migrated from C compound accessors to Rust.
// =============================================================================

extern "C" {
    // New accessors for continue_search (Phase 10)
    // (startpos get/set functions: inlined in vars.rs)
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_getwhitecols_of_line(line: *const c_char) -> c_int;
    fn nvim_skipwhite_offset(line: *const c_char, length: c_int, start_col: c_int) -> c_int;
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn rs_ctrl_x_mode_path_patterns() -> c_int;
    fn rs_ctrl_x_mode_path_defines() -> c_int;
    fn rs_ctrl_x_mode_line_or_eval() -> c_int;
    fn rs_ctrl_x_mode_register() -> c_int;

    // New accessors for show_statusmsg (Phase 10)
    fn nvim_compl_match_get_next(m: crate::match_list::ComplMatch)
        -> crate::match_list::ComplMatch;
    fn rs_compl_status_adding() -> c_int;
    // (nvim_compl_curr_match_next_eq_prev, _cp_number, _set_cp_number: inlined in match_list.rs)
    fn rs_ins_compl_update_sequence_numbers();
    fn nvim_get_dollar_vcol() -> c_int;
    fn nvim_curs_columns_curwin();
    #[link_name = "redraw_mode"]
    static mut g_redraw_mode: bool;
    fn nvim_shortmess_completionmenu() -> bool;
    fn nvim_get_p_smd() -> c_int;
    fn nvim_set_edit_submode_extra_hitend();
    fn nvim_set_edit_submode_extra_patnotf();
    fn nvim_set_edit_submode_extra_back_at_original();
    fn nvim_set_edit_submode_extra_word_from_other_line();
    fn nvim_set_edit_submode_extra_the_only_match();
    fn nvim_set_edit_submode_extra_match_ref(cp_number: c_int, compl_matches: c_int);
    fn nvim_get_edit_submode_highl_attr() -> c_int;
    fn nvim_get_edit_submode_extra_ptr() -> *const c_char;
    #[link_name = "msg_hist_off"]
    static mut g_msg_hist_off: bool;
    fn nvim_msg_ext_set_kind_completion();
    #[link_name = "msg"]
    fn nvim_msg_with_attr(s: *const c_char, attr: c_int) -> bool;
    #[link_name = "msg_clr_cmdline"]
    fn nvim_msg_clr_cmdline_wrap();
}

// Continuation status flags (already defined at top of file, re-use)
// CONT_INTRPT = 6, CONT_SOL = 16, CONT_S_IPOS = 8,
// CONT_ADDING = 1, CONT_N_ADDS = 4, CONT_LOCAL = 32

/// Buffer size used by C (IOSIZE = 1025, MIN_SPACE = 75)
const IOSIZE_MINUS_MIN_SPACE: c_int = 1025 - 75;

// HLF enum values (from highlight_defs.h, verified by test)
const HLF_E: c_int = 6; // ErrorMsg
const HLF_W: c_int = 26; // WarningMsg
const HLF_R: c_int = 18; // Question
const HLF_COUNT: c_int = 76; // sentinel (not a real highlight)

/// Resume interrupted completion: adjust compl_startpos, compl_col,
/// compl_length, and compl_cont_status.
///
/// Transliteration of the C `nvim_ins_compl_continue_search_impl` compound accessor.
///
/// # Safety
/// Requires valid global state; line must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_continue_search(line: *mut c_char) {
    // Remove CONT_INTRPT flag
    let mut cont_status = crate::vars::nvim_get_compl_cont_status();
    cont_status &= !CONT_INTRPT;
    crate::vars::nvim_set_compl_cont_status(cont_status);

    if rs_ctrl_x_mode_normal() != 0
        || rs_ctrl_x_mode_path_patterns() != 0
        || rs_ctrl_x_mode_path_defines() != 0
    {
        if crate::vars::nvim_get_compl_startpos_lnum() == nvim_get_curwin_cursor_lnum() {
            cont_status = crate::vars::nvim_get_compl_cont_status();
            if (cont_status & CONT_S_IPOS) != 0 {
                cont_status |= CONT_SOL;
                crate::vars::nvim_set_compl_cont_status(cont_status);
                let new_col = nvim_skipwhite_offset(
                    line,
                    crate::vars::nvim_get_compl_length(),
                    crate::vars::nvim_get_compl_startpos_col(),
                );
                crate::vars::nvim_set_compl_startpos_col(new_col);
            }
            crate::vars::nvim_set_compl_col(crate::vars::nvim_get_compl_startpos_col());
        } else {
            let wcols = nvim_getwhitecols_of_line(line);
            crate::vars::nvim_set_compl_col(wcols);
            crate::vars::nvim_set_compl_startpos_col(wcols);
            crate::vars::nvim_set_compl_startpos_lnum_to_cursor();
            // Clear CONT_SOL
            cont_status = crate::vars::nvim_get_compl_cont_status();
            cont_status &= !CONT_SOL;
            crate::vars::nvim_set_compl_cont_status(cont_status);
        }
        let new_length = nvim_get_cursor_col() - crate::vars::nvim_get_compl_col();
        crate::vars::nvim_set_compl_length(new_length);
        if crate::vars::nvim_get_compl_length() > IOSIZE_MINUS_MIN_SPACE {
            // Clear CONT_SOL and clamp length
            cont_status = crate::vars::nvim_get_compl_cont_status();
            cont_status &= !CONT_SOL;
            crate::vars::nvim_set_compl_cont_status(cont_status);
            crate::vars::nvim_set_compl_length(IOSIZE_MINUS_MIN_SPACE);
            crate::vars::nvim_set_compl_col(
                nvim_get_cursor_col() - crate::vars::nvim_get_compl_length(),
            );
        }
        cont_status = crate::vars::nvim_get_compl_cont_status();
        cont_status |= CONT_ADDING | CONT_N_ADDS;
        crate::vars::nvim_set_compl_cont_status(cont_status);
        if crate::vars::nvim_get_compl_length() < 1 {
            cont_status = crate::vars::nvim_get_compl_cont_status();
            cont_status &= CONT_LOCAL;
            crate::vars::nvim_set_compl_cont_status(cont_status);
        }
    } else if rs_ctrl_x_mode_line_or_eval() != 0 || rs_ctrl_x_mode_register() != 0 {
        crate::vars::nvim_set_compl_cont_status(CONT_ADDING | CONT_N_ADDS);
    } else {
        crate::vars::nvim_set_compl_cont_status(0);
    }
}

/// Construct and display the completion status message
/// ("match N of M", "Back at original", etc.).
///
/// Transliteration of the C `nvim_ins_compl_show_statusmsg_impl` compound accessor.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_show_statusmsg() {
    // We found no match if the list has only the "compl_orig_text"-entry
    let first = crate::match_list::compl_first_match;
    if !first.is_null() && crate::match_list::is_first_match(nvim_compl_match_get_next(first)) {
        if rs_compl_status_adding() != 0 && crate::vars::nvim_get_compl_length() > 1 {
            nvim_set_edit_submode_extra_hitend();
        } else {
            nvim_set_edit_submode_extra_patnotf();
        }
        g_edit_submode_highl = HLF_E;
    }

    if g_edit_submode_extra.is_null() {
        if crate::match_list::curr_match_at_original_text() {
            nvim_set_edit_submode_extra_back_at_original();
            g_edit_submode_highl = HLF_W;
        } else {
            let cont_status = crate::vars::nvim_get_compl_cont_status();
            if (cont_status & CONT_S_IPOS) != 0 {
                nvim_set_edit_submode_extra_word_from_other_line();
                g_edit_submode_highl = HLF_COUNT;
            } else if crate::match_list::curr_match_next_eq_prev() {
                nvim_set_edit_submode_extra_the_only_match();
                g_edit_submode_highl = HLF_COUNT;
                crate::match_list::curr_match_set_cp_number(1);
            } else {
                // Update completion sequence number when needed.
                if crate::match_list::curr_match_cp_number() == -1 {
                    rs_ins_compl_update_sequence_numbers();
                }

                // The match should always have a sequence number now;
                // this is just a safety check.
                if crate::match_list::curr_match_cp_number() != -1 {
                    nvim_set_edit_submode_extra_match_ref(
                        crate::match_list::curr_match_cp_number(),
                        crate::vars::nvim_get_compl_matches(),
                    );
                    g_edit_submode_highl = HLF_R;
                    if nvim_get_dollar_vcol() >= 0 {
                        nvim_curs_columns_curwin();
                    }
                }
            }
        }
    }

    // Show a message about what (completion) mode we're in.
    g_redraw_mode = true;
    if !nvim_shortmess_completionmenu() {
        if g_edit_submode_extra.is_null() {
            nvim_msg_clr_cmdline_wrap(); // necessary for "noshowmode"
        } else {
            // edit_submode_extra is non-null
            if nvim_get_p_smd() == 0 {
                g_msg_hist_off = true;
                nvim_msg_ext_set_kind_completion();
                nvim_msg_with_attr(
                    nvim_get_edit_submode_extra_ptr(),
                    nvim_get_edit_submode_highl_attr(),
                );
                g_msg_hist_off = false;
            }
        }
    }
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
