//! Insert-mode completion (CTRL-X) mode checking for Neovim
//!
//! This module provides Rust implementations for checking the current CTRL-X
//! completion mode in insert mode.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

use std::os::raw::{c_char, c_int, c_uint};

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
const CTRL_X_DICTIONARY: c_int = 9 + CTRL_X_WANT_IDENT;
const CTRL_X_THESAURUS: c_int = 10 + CTRL_X_WANT_IDENT;
const CTRL_X_CMDLINE: c_int = 11;
const CTRL_X_FUNCTION: c_int = 12;
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_SPELL: c_int = 14;
// const CTRL_X_LOCAL_MSG: c_int = 15; // only used in ctrl_x_msgs
const CTRL_X_EVAL: c_int = 16;
const CTRL_X_CMDLINE_CTRL_X: c_int = 17;
const CTRL_X_BUFNAMES: c_int = 18;
const CTRL_X_REGISTER: c_int = 19;

// Control key constants (from ascii_defs.h)
// These are ASCII control codes: Ctrl_X = 'X' - 'A' + 1
const CTRL_D: c_int = 4;
const CTRL_E: c_int = 5;
const CTRL_F: c_int = 6;
const CTRL_I: c_int = 9;
const CTRL_K: c_int = 11;
const CTRL_L: c_int = 12;
const CTRL_N: c_int = 14;
const CTRL_O: c_int = 15;
const CTRL_P: c_int = 16;
const CTRL_Q: c_int = 17;
const CTRL_R: c_int = 18;
const CTRL_S: c_int = 19;
const CTRL_T: c_int = 20;
const CTRL_U: c_int = 21;
const CTRL_V: c_int = 22;
const CTRL_X: c_int = 24;
const CTRL_Y: c_int = 25;
const CTRL_Z: c_int = 26;
const CTRL_RSB: c_int = 29; // Right Square Bracket (']' - '@')

// Completion status flags (from insexpand.c)
const CONT_ADDING: c_int = 1;
const CONT_SOL: c_int = 16;
const CONT_LOCAL: c_int = 32;

// C accessors for static variables
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_time_slice_expired() -> c_int;
    fn nvim_get_compl_enter_selects() -> c_int;
    fn nvim_get_compl_used_match() -> c_int;
    fn nvim_get_compl_length() -> c_int;
    fn nvim_get_cot_flags_global() -> c_uint;
    fn nvim_curbuf_get_b_cot_flags() -> c_uint;
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_get_compl_from_nonkeyword() -> c_int;
    // Character checking functions from charset.c
    fn rs_vim_isIDc(c: c_int) -> c_int;
    fn rs_vim_isfilec(c: c_int) -> c_int;
    fn rs_vim_ispathsep(c: c_int) -> c_int;
    fn rs_vim_isprintc(c: c_int) -> c_int;
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn rs_vim_iswordc(c: c_int) -> c_int;
}

// completeopt flags (from optionstr.h)
const K_OPT_COT_FLAG_MENU: c_uint = 0x01;
const K_OPT_COT_FLAG_MENUONE: c_uint = 0x02;

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

/// Check if Insert completion is active.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_active() -> c_int {
    nvim_get_compl_started()
}

/// Check if completion was interrupted.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_interrupted() -> c_int {
    c_int::from(nvim_get_compl_interrupted() != 0 || nvim_get_compl_time_slice_expired() != 0)
}

/// Check if pressing Enter selects a match in the completion popup.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_enter_selects() -> c_int {
    nvim_get_compl_enter_selects()
}

/// Check if one of the matches was selected.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_used_match() -> c_int {
    nvim_get_compl_used_match()
}

/// Return the length in bytes of the text being completed.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_len() -> c_int {
    nvim_get_compl_length()
}

// =============================================================================
// Popup menu functions
// =============================================================================

/// Helper function to get the effective cot_flags (buffer-local or global).
#[inline]
unsafe fn get_cot_flags() -> c_uint {
    let b_cot_flags = nvim_curbuf_get_b_cot_flags();
    if b_cot_flags != 0 {
        b_cot_flags
    } else {
        nvim_get_cot_flags_global()
    }
}

/// Check if the popup menu should be displayed.
/// "completeopt" must contain "menu" or "menuone", or compl_autocomplete is set.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_wanted() -> c_int {
    let cot_flags = get_cot_flags();
    let has_menu_flag = (cot_flags & (K_OPT_COT_FLAG_MENU | K_OPT_COT_FLAG_MENUONE)) != 0;
    c_int::from(has_menu_flag || nvim_get_compl_autocomplete() != 0)
}

// =============================================================================
// Completion character acceptance functions
// =============================================================================

/// Check that character "c" is part of the item currently being completed.
/// Used to decide whether to abandon complete mode when the menu is visible.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_accept_char(c: c_int) -> c_int {
    // If autocomplete is active and started from non-keyword, reject all chars
    if nvim_get_compl_autocomplete() != 0 && nvim_get_compl_from_nonkeyword() != 0 {
        return 0;
    }

    let ctrl_x_mode = nvim_get_ctrl_x_mode();

    // When expanding an identifier only accept identifier chars
    if (ctrl_x_mode & CTRL_X_WANT_IDENT) != 0 {
        return rs_vim_isIDc(c);
    }

    match ctrl_x_mode {
        CTRL_X_FILES => {
            // When expanding file name only accept file name chars. But not
            // path separators, so that "proto/<Tab>" expands files in
            // "proto", not "proto/" as a whole
            c_int::from(rs_vim_isfilec(c) != 0 && rs_vim_ispathsep(c) == 0)
        }
        CTRL_X_CMDLINE | CTRL_X_CMDLINE_CTRL_X | CTRL_X_OMNI => {
            // Command line and Omni completion can work with just about any
            // printable character, but do stop at white space.
            c_int::from(rs_vim_isprintc(c) != 0 && rs_ascii_iswhite(c) == 0)
        }
        CTRL_X_WHOLE_LINE => {
            // For whole line completion a space can be part of the line.
            rs_vim_isprintc(c)
        }
        _ => rs_vim_iswordc(c),
    }
}

// =============================================================================
// Word boundary functions
// =============================================================================

extern "C" {
    fn rs_mb_get_class(p: *const c_char) -> c_int;
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;
}

/// Find the start of the next word.
/// Returns a pointer to the first char of the word. Also stops at a NUL.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_find_word_start(mut ptr: *mut c_char) -> *mut c_char {
    // while (*ptr != NUL && *ptr != '\n' && mb_get_class(ptr) <= 1)
    while *ptr != 0 && *ptr != b'\n' as c_char && rs_mb_get_class(ptr) <= 1 {
        ptr = ptr.add(rs_utfc_ptr2len(ptr) as usize);
    }
    ptr
}

/// Find the end of the word. Assumes it starts inside a word.
/// Returns a pointer to just after the word.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_find_word_end(mut ptr: *mut c_char) -> *mut c_char {
    let start_class = rs_mb_get_class(ptr);
    if start_class > 1 {
        while *ptr != 0 {
            ptr = ptr.add(rs_utfc_ptr2len(ptr) as usize);
            if rs_mb_get_class(ptr) != start_class {
                break;
            }
        }
    }
    ptr
}

/// ASCII constants
const CAR: c_char = 0x0D; // '\015' carriage return
const NL: c_char = 0x0A; // '\012' newline

/// Find the end of the line, omitting CR and NL at the end.
/// Returns a pointer to just after the line.
#[no_mangle]
pub unsafe extern "C" fn rs_find_line_end(ptr: *mut c_char) -> *mut c_char {
    // Find end of string
    let mut s = ptr;
    while *s != 0 {
        s = s.add(1);
    }
    // Back up over trailing CR and NL
    while s > ptr && (*s.sub(1) == CAR || *s.sub(1) == NL) {
        s = s.sub(1);
    }
    s
}

// =============================================================================
// CTRL-X key checking
// =============================================================================

/// Check if a character is a valid CTRL-X completion key for the current mode.
///
/// This determines which keys are accepted in each CTRL-X sub-mode.
/// Always allows ^R (except in register mode) to let its results be checked.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_vim_is_ctrl_x_key(c: c_int) -> c_int {
    let mode = nvim_get_ctrl_x_mode();

    // Always allow ^R - let its results then be checked
    if c == CTRL_R && mode != CTRL_X_REGISTER {
        return 1;
    }

    let result = match mode {
        0 => {
            // Not in any CTRL-X mode
            c == CTRL_N || c == CTRL_P || c == CTRL_X
        }
        m if m == CTRL_X_NOT_DEFINED_YET || m == CTRL_X_CMDLINE_CTRL_X => {
            c == CTRL_X
                || c == CTRL_Y
                || c == CTRL_E
                || c == CTRL_L
                || c == CTRL_F
                || c == CTRL_RSB
                || c == CTRL_I
                || c == CTRL_D
                || c == CTRL_P
                || c == CTRL_N
                || c == CTRL_T
                || c == CTRL_V
                || c == CTRL_Q
                || c == CTRL_U
                || c == CTRL_O
                || c == CTRL_S
                || c == CTRL_K
                || c == i32::from(b's')
                || c == CTRL_Z
                || c == CTRL_R
        }
        m if m == CTRL_X_SCROLL => c == CTRL_Y || c == CTRL_E,
        m if m == CTRL_X_WHOLE_LINE => c == CTRL_L || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_FILES => c == CTRL_F || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_DICTIONARY => c == CTRL_K || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_THESAURUS => c == CTRL_T || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_TAGS => c == CTRL_RSB || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_PATH_PATTERNS => c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_PATH_DEFINES => c == CTRL_D || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_CMDLINE => {
            c == CTRL_V || c == CTRL_Q || c == CTRL_P || c == CTRL_N || c == CTRL_X
        }
        m if m == CTRL_X_FUNCTION => c == CTRL_U || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_OMNI => c == CTRL_O || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_SPELL => c == CTRL_S || c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_EVAL => c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_BUFNAMES => c == CTRL_P || c == CTRL_N,
        m if m == CTRL_X_REGISTER => c == CTRL_R || c == CTRL_P || c == CTRL_N,
        _ => {
            // internal_error case - should not happen
            false
        }
    };

    c_int::from(result)
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
