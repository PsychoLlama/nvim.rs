//! Insert-mode completion (CTRL-X) mode checking for Neovim
//!
//! This module provides Rust implementations for checking the current CTRL-X
//! completion mode in insert mode.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

pub mod add;
pub mod match_list;
pub mod sort;

use std::os::raw::{c_char, c_int, c_uint};

use nvim_buffer::BufHandle;
use nvim_window::WinHandle;

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

// Completeopt flags (from option_vars.generated.h)
const K_OPT_COT_FLAG_MENU: c_uint = 0x01;
const K_OPT_COT_FLAG_MENUONE: c_uint = 0x02;
const K_OPT_COT_FLAG_LONGEST: c_uint = 0x04;
const K_OPT_COT_FLAG_PREVIEW: c_uint = 0x08;
const K_OPT_COT_FLAG_POPUP: c_uint = 0x10;
const K_OPT_COT_FLAG_NOINSERT: c_uint = 0x20;
const K_OPT_COT_FLAG_NOSELECT: c_uint = 0x40;
const K_OPT_COT_FLAG_FUZZY: c_uint = 0x80;
const K_OPT_COT_FLAG_NOSORT: c_uint = 0x100;
const K_OPT_COT_FLAG_PREINSERT: c_uint = 0x200;
const K_OPT_COT_FLAG_NEAREST: c_uint = 0x400;

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
    fn nvim_get_compl_was_interrupted() -> c_int;
    fn nvim_get_compl_opt_refresh_always() -> c_int;
    // Character checking functions from charset.c
    fn rs_vim_isIDc(c: c_int) -> c_int;
    fn rs_vim_isfilec(c: c_int) -> c_int;
    fn rs_vim_ispathsep(c: c_int) -> c_int;
    fn rs_vim_isprintc(c: c_int) -> c_int;
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn rs_vim_iswordc(c: c_int) -> c_int;

    // Completion window/buffer accessors
    fn nvim_get_compl_curr_win() -> WinHandle;
    fn nvim_get_compl_curr_buf() -> BufHandle;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_get_compl_col() -> c_int;

    // Direction and state accessors
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_shows_dir() -> c_int;

    // Option accessors
    fn nvim_get_p_ic() -> c_int;
    fn nvim_get_p_inf() -> c_int;
    fn nvim_get_p_ac() -> c_int;
    fn nvim_curbuf_get_b_p_ac() -> c_int;
    fn nvim_get_compl_ins_end_col() -> c_int;
    fn nvim_get_cursor_col() -> c_int;

    // Line/column accessors
    fn nvim_get_compl_lnum() -> c_int;
    fn nvim_get_curwin_cursor_lnum() -> c_int;

    // Leader accessors
    fn nvim_get_compl_leader_data() -> *const c_char;
    fn nvim_get_compl_leader_size() -> usize;
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_orig_text_size() -> usize;

    // Shown match accessors
    fn nvim_compl_shown_match_exists() -> c_int;
    fn nvim_compl_shown_match_is_singular() -> c_int;
    fn nvim_compl_shown_match_is_first() -> c_int;
    fn nvim_compl_shown_match_str_size() -> usize;
    fn nvim_compl_shown_match_has_newline() -> c_int;

    // Match list accessors (Phase 4)
    fn nvim_compl_first_match_is_null() -> c_int;
    fn nvim_compl_curr_match_is_null() -> c_int;
    fn nvim_compl_curr_match_at_original_text() -> c_int;
    fn nvim_get_compl_matches() -> c_int;
    fn nvim_compl_curr_match_has_str() -> c_int;

    // Popup menu and selection accessors
    fn nvim_get_compl_selected_item() -> c_int;
    fn nvim_get_pum_want_item() -> c_int;
    fn nvim_get_pum_want_active() -> c_int;
    fn nvim_get_pum_want_insert() -> c_int;
    fn nvim_pum_visible() -> c_int;
    fn nvim_pum_get_height() -> c_int;
}

// Direction constants
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

// =============================================================================
// Key code constants (from keycodes.h)
// =============================================================================

/// Convert termcap codes to internal key representation.
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

/// KS_EXTRA is used for keys that have no termcap name.
const KS_EXTRA: c_int = 253;

// Key extra codes for special keys
const KE_S_UP: c_int = 4;
const KE_S_DOWN: c_int = 5;
const KE_EVENT: c_int = 102;
const KE_COMMAND: c_int = 104;
const KE_LUA: c_int = 107;

// Standard key codes
const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
const K_PAGEUP: c_int = termcap2key(b'k' as c_int, b'P' as c_int);
const K_PAGEDOWN: c_int = termcap2key(b'k' as c_int, b'N' as c_int);
const K_KPAGEUP: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
const K_KPAGEDOWN: c_int = termcap2key(b'K' as c_int, b'5' as c_int);
const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
const K_EVENT: c_int = termcap2key(KS_EXTRA, KE_EVENT);
const K_COMMAND: c_int = termcap2key(KS_EXTRA, KE_COMMAND);
const K_LUA: c_int = termcap2key(KS_EXTRA, KE_LUA);

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

/// Return true when wp is the actual completion window.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_win_active(wp: WinHandle) -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }
    let curr_win = nvim_get_compl_curr_win();
    let curr_buf = nvim_get_compl_curr_buf();
    let wp_buf = nvim_win_get_buffer(wp);
    c_int::from(wp == curr_win && wp_buf == curr_buf)
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

/// Return the column where the completion text starts.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_col() -> c_int {
    nvim_get_compl_col()
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
// Completion refresh functions
// =============================================================================

/// Check if the complete function returned "always" in the "refresh" dictionary item.
/// Only applies to function and omni completion modes.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_refresh_always() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    let is_function = mode == CTRL_X_FUNCTION;
    let is_omni = mode == CTRL_X_OMNI;
    c_int::from((is_function || is_omni) && nvim_get_compl_opt_refresh_always() != 0)
}

/// Check that we need to find matches again (ins_compl_restart is to be called).
///
/// Returns true if we didn't complete finding matches or when the
/// "completefunc" returned "always" in the "refresh" dictionary item.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_need_restart() -> c_int {
    c_int::from(nvim_get_compl_was_interrupted() != 0 || rs_ins_compl_refresh_always() != 0)
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

// =============================================================================
// Direction functions
// =============================================================================

/// Return true if completion is using the forward direction matches.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_dir_forward() -> c_int {
    c_int::from(nvim_get_compl_direction() == FORWARD)
}

/// Return true if currently showing forward completion matches.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_shows_dir_forward() -> c_int {
    c_int::from(nvim_get_compl_shows_dir() == FORWARD)
}

/// Return true if currently showing backward completion matches.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_shows_dir_backward() -> c_int {
    c_int::from(nvim_get_compl_shows_dir() == BACKWARD)
}

// =============================================================================
// Preinsert functions
// =============================================================================

/// Return true when the 'completeopt' "preinsert" flag is in effect.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_has_preinsert() -> c_int {
    let cur_cot_flags = get_cot_flags();

    // If autocomplete is active and ignorecase is set but infercase is not, disable preinsert
    if nvim_get_compl_autocomplete() != 0 && nvim_get_p_ic() != 0 && nvim_get_p_inf() == 0 {
        return 0;
    }

    // When not in autocomplete mode:
    //   preinsert is active if (preinsert|fuzzy|menuone) == (preinsert|menuone)
    //   i.e., preinsert and menuone are set, but fuzzy is NOT set
    // When in autocomplete mode:
    //   preinsert is active if (preinsert|fuzzy) == preinsert
    //   i.e., preinsert is set but fuzzy is NOT set
    let result = if nvim_get_compl_autocomplete() == 0 {
        (cur_cot_flags & (K_OPT_COT_FLAG_PREINSERT | K_OPT_COT_FLAG_FUZZY | K_OPT_COT_FLAG_MENUONE))
            == (K_OPT_COT_FLAG_PREINSERT | K_OPT_COT_FLAG_MENUONE)
    } else {
        (cur_cot_flags & (K_OPT_COT_FLAG_PREINSERT | K_OPT_COT_FLAG_FUZZY))
            == K_OPT_COT_FLAG_PREINSERT
    };
    c_int::from(result)
}

/// Returns true if autocomplete is active and the pre-insert effect targets the
/// longest prefix.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_preinsert_longest() -> c_int {
    let cur_cot_flags = get_cot_flags();
    let result = nvim_get_compl_autocomplete() != 0
        && (cur_cot_flags
            & (K_OPT_COT_FLAG_LONGEST | K_OPT_COT_FLAG_PREINSERT | K_OPT_COT_FLAG_FUZZY))
            == K_OPT_COT_FLAG_LONGEST;
    c_int::from(result)
}

/// Returns true if the pre-insert effect is valid and the cursor is within
/// the `compl_ins_end_col` range.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_preinsert_effect() -> c_int {
    if rs_ins_compl_has_preinsert() == 0 && rs_ins_compl_preinsert_longest() == 0 {
        return 0;
    }
    c_int::from(nvim_get_cursor_col() < nvim_get_compl_ins_end_col())
}

// =============================================================================
// Autocomplete functions
// =============================================================================

/// Return true if 'autocomplete' option is set
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_has_autocomplete() -> c_int {
    // Use buffer-local setting if defined (>= 0), otherwise use global
    let b_p_ac = nvim_curbuf_get_b_p_ac();
    if b_p_ac >= 0 {
        c_int::from(b_p_ac != 0)
    } else {
        c_int::from(nvim_get_p_ac() != 0)
    }
}

// =============================================================================
// Leader string functions
// =============================================================================

/// Get the completion leader string data pointer.
///
/// Returns compl_leader.data if set, otherwise compl_orig_text.data.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_leader() -> *const c_char {
    let leader_data = nvim_get_compl_leader_data();
    if leader_data.is_null() {
        nvim_get_compl_orig_text_data()
    } else {
        leader_data
    }
}

/// Get the length of the completion leader.
///
/// Returns compl_leader.size if leader is set, otherwise compl_orig_text.size.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_leader_len() -> usize {
    let leader_data = nvim_get_compl_leader_data();
    if leader_data.is_null() {
        nvim_get_compl_orig_text_size()
    } else {
        nvim_get_compl_leader_size()
    }
}

// =============================================================================
// Match state functions
// =============================================================================

/// Return true if a match is selected (even if it is not inserted).
/// Returns true if compl_shown_match is not NULL and is not the first match
/// (the original text entry).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_is_match_selected() -> c_int {
    c_int::from(nvim_compl_shown_match_exists() != 0 && nvim_compl_shown_match_is_first() == 0)
}

/// Return whether there currently is a shown match.
/// Returns true if compl_shown_match is NULL or is not singular.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_has_shown_match() -> c_int {
    c_int::from(nvim_compl_shown_match_exists() == 0 || nvim_compl_shown_match_is_singular() == 0)
}

/// Return whether the shown match is long enough.
///
/// Returns true if the shown match text length is greater than (cursor_col - compl_col).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ins_compl_long_shown_match() -> c_int {
    if nvim_compl_shown_match_exists() == 0 {
        return 0;
    }
    let str_size = nvim_compl_shown_match_str_size();
    if str_size == 0 {
        return 0;
    }
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    c_int::from(str_size > (cursor_col - compl_col) as usize)
}

// =============================================================================
// Multi-line completion functions
// =============================================================================

/// Check if the current completion contains newline characters (multi-line).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_has_multiple() -> c_int {
    nvim_compl_shown_match_has_newline()
}

/// Check if line is in multi-line completion range.
///
/// Returns true if the given line number falls within the range of a multi-line
/// completion, i.e. between compl_lnum and current cursor line.
/// Always returns false for single-line completions.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_lnum_in_range(lnum: c_int) -> c_int {
    // First check if it's a multi-line completion
    if rs_ins_compl_has_multiple() == 0 {
        return 0;
    }
    let compl_lnum = nvim_get_compl_lnum();
    let cursor_lnum = nvim_get_curwin_cursor_lnum();
    c_int::from(lnum >= compl_lnum && lnum <= cursor_lnum)
}

// =============================================================================
// Phase 2: Key Handling and Direction
// =============================================================================

/// Decide the direction of Insert mode complete from the key typed.
///
/// Returns BACKWARD or FORWARD based on the key.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_key2dir(c: c_int) -> c_int {
    // For event/command/lua keys, compare pum_want.item with compl_selected_item
    if c == K_EVENT || c == K_COMMAND || c == K_LUA {
        let pum_want_item = nvim_get_pum_want_item();
        let selected = nvim_get_compl_selected_item();
        return if pum_want_item < selected {
            BACKWARD
        } else {
            FORWARD
        };
    }

    // CTRL-P, CTRL-L, pageup keys, shift-up, up all go backward
    if c == CTRL_P || c == CTRL_L || c == K_PAGEUP || c == K_KPAGEUP || c == K_S_UP || c == K_UP {
        return BACKWARD;
    }

    FORWARD
}

/// Check that "c" is a valid completion key only while the popup menu is shown.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_pum_key(c: c_int) -> c_int {
    if nvim_pum_visible() == 0 {
        return 0;
    }

    c_int::from(
        c == K_PAGEUP
            || c == K_KPAGEUP
            || c == K_S_UP
            || c == K_PAGEDOWN
            || c == K_KPAGEDOWN
            || c == K_S_DOWN
            || c == K_UP
            || c == K_DOWN,
    )
}

/// Decide the number of completions to move forward.
///
/// Returns 1 for most keys, height of the popup menu for page-up/down keys.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_key2count(c: c_int) -> c_int {
    // For event/command/lua keys, return absolute offset
    if c == K_EVENT || c == K_COMMAND || c == K_LUA {
        let offset = nvim_get_pum_want_item() - nvim_get_compl_selected_item();
        return offset.abs();
    }

    // For page keys (except plain up/down), return popup height minus context
    if rs_ins_compl_pum_key(c) != 0 && c != K_UP && c != K_DOWN {
        let mut h = nvim_pum_get_height();
        if h > 3 {
            h -= 2; // keep some context
        }
        return h;
    }

    1
}

/// Check that completion with "c" should insert the match.
///
/// Returns false if only to change the currently selected completion.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_use_match(c: c_int) -> c_int {
    match c {
        K_UP | K_DOWN | K_PAGEDOWN | K_KPAGEDOWN | K_S_DOWN | K_PAGEUP | K_KPAGEUP | K_S_UP => 0,
        K_EVENT | K_COMMAND | K_LUA => {
            c_int::from(nvim_get_pum_want_active() != 0 && nvim_get_pum_want_insert() != 0)
        }
        _ => 1,
    }
}

// =============================================================================
// Completeopt flag utilities (Phase 3)
// =============================================================================

/// Check if fuzzy matching is enabled in completeopt.
#[inline]
const fn cot_fuzzy_impl(flags: c_uint) -> bool {
    (flags & K_OPT_COT_FLAG_FUZZY) != 0
}

/// Check if fuzzy matching is enabled.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_fuzzy() -> c_int {
    c_int::from(cot_fuzzy_impl(get_cot_flags()))
}

/// Check if "nearest" sorting should be active.
///
/// Returns true if matches should be sorted based on proximity to the cursor.
#[no_mangle]
pub unsafe extern "C" fn rs_is_nearest_active() -> c_int {
    let flags = get_cot_flags();
    let autocomplete = nvim_get_compl_autocomplete() != 0;
    let nearest_flag = (flags & K_OPT_COT_FLAG_NEAREST) != 0;
    c_int::from((autocomplete || nearest_flag) && !cot_fuzzy_impl(flags))
}

/// Check if "longest" option is enabled in completeopt.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_longest() -> c_int {
    c_int::from((get_cot_flags() & K_OPT_COT_FLAG_LONGEST) != 0)
}

/// Check if "noinsert" option is enabled in completeopt.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_noinsert() -> c_int {
    c_int::from((get_cot_flags() & K_OPT_COT_FLAG_NOINSERT) != 0)
}

/// Check if "noselect" option is enabled in completeopt.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_noselect() -> c_int {
    c_int::from((get_cot_flags() & K_OPT_COT_FLAG_NOSELECT) != 0)
}

/// Check if "nosort" option is enabled in completeopt.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_nosort() -> c_int {
    c_int::from((get_cot_flags() & K_OPT_COT_FLAG_NOSORT) != 0)
}

/// Check if "menuone" option is enabled in completeopt.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_menuone() -> c_int {
    c_int::from((get_cot_flags() & K_OPT_COT_FLAG_MENUONE) != 0)
}

/// Check if "popup" option is enabled in completeopt.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_popup() -> c_int {
    c_int::from((get_cot_flags() & K_OPT_COT_FLAG_POPUP) != 0)
}

/// Check if "preview" option is enabled in completeopt.
#[no_mangle]
pub unsafe extern "C" fn rs_cot_preview() -> c_int {
    c_int::from((get_cot_flags() & K_OPT_COT_FLAG_PREVIEW) != 0)
}

/// Get the raw completeopt flags.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cot_flags() -> c_uint {
    get_cot_flags()
}

// =============================================================================
// Completion length utilities
// =============================================================================

/// Get the length of the completion from start column to cursor column.
///
/// Returns the number of characters typed in the completion, clamped to >= 0.
#[no_mangle]
pub unsafe extern "C" fn rs_get_compl_len() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let off = cursor_col - compl_col;
    if off < 0 {
        0
    } else {
        off
    }
}

// =============================================================================
// Match list queries (Phase 4)
// =============================================================================

/// Check if the match list has any matches (compl_first_match != NULL).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_has_matches() -> c_int {
    c_int::from(nvim_compl_first_match_is_null() == 0)
}

/// Get the number of matches in the completion list.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_match_count() -> c_int {
    nvim_get_compl_matches()
}

/// Check if there's a current match selected (compl_curr_match != NULL).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_has_curr_match() -> c_int {
    c_int::from(nvim_compl_curr_match_is_null() == 0)
}

/// Check if the current match is at original text position.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_at_original() -> c_int {
    nvim_compl_curr_match_at_original_text()
}

/// Check if the current match has valid string data.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_curr_has_str() -> c_int {
    nvim_compl_curr_match_has_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        // Verify the CTRL-X mode constants match expected values
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
        assert_eq!(CTRL_X_SCROLL, 2);
        assert_eq!(CTRL_X_WHOLE_LINE, 3);
        assert_eq!(CTRL_X_FILES, 4);
        assert_eq!(CTRL_X_TAGS, 5 + CTRL_X_WANT_IDENT);
        assert_eq!(CTRL_X_PATH_PATTERNS, 6 + CTRL_X_WANT_IDENT);
        assert_eq!(CTRL_X_PATH_DEFINES, 7 + CTRL_X_WANT_IDENT);
        assert_eq!(CTRL_X_DICTIONARY, 9 + CTRL_X_WANT_IDENT);
        assert_eq!(CTRL_X_THESAURUS, 10 + CTRL_X_WANT_IDENT);
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
    fn test_ctrl_x_want_ident_flag() {
        // CTRL_X_WANT_IDENT should be 0x100
        assert_eq!(CTRL_X_WANT_IDENT, 0x100);

        // Modes with CTRL_X_WANT_IDENT should have the flag set
        assert_ne!(CTRL_X_TAGS & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_PATH_PATTERNS & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_PATH_DEFINES & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_DICTIONARY & CTRL_X_WANT_IDENT, 0);
        assert_ne!(CTRL_X_THESAURUS & CTRL_X_WANT_IDENT, 0);

        // Modes without CTRL_X_WANT_IDENT should not have the flag
        assert_eq!(CTRL_X_NORMAL & CTRL_X_WANT_IDENT, 0);
        assert_eq!(CTRL_X_SCROLL & CTRL_X_WANT_IDENT, 0);
        assert_eq!(CTRL_X_FILES & CTRL_X_WANT_IDENT, 0);
        assert_eq!(CTRL_X_CMDLINE & CTRL_X_WANT_IDENT, 0);
    }

    #[test]
    fn test_ctrl_key_constants() {
        // Control keys are ASCII control codes: Ctrl_X = 'X' - 'A' + 1
        assert_eq!(CTRL_D, 4); // 'D' - 'A' + 1
        assert_eq!(CTRL_E, 5);
        assert_eq!(CTRL_F, 6);
        assert_eq!(CTRL_I, 9); // Tab
        assert_eq!(CTRL_K, 11);
        assert_eq!(CTRL_L, 12);
        assert_eq!(CTRL_N, 14);
        assert_eq!(CTRL_O, 15);
        assert_eq!(CTRL_P, 16);
        assert_eq!(CTRL_Q, 17);
        assert_eq!(CTRL_R, 18);
        assert_eq!(CTRL_S, 19);
        assert_eq!(CTRL_T, 20);
        assert_eq!(CTRL_U, 21);
        assert_eq!(CTRL_V, 22);
        assert_eq!(CTRL_X, 24);
        assert_eq!(CTRL_Y, 25);
        assert_eq!(CTRL_Z, 26);
        assert_eq!(CTRL_RSB, 29); // ']' - '@'
    }

    #[test]
    fn test_completion_status_flags() {
        // Completion status flags
        assert_eq!(CONT_ADDING, 1);
        assert_eq!(CONT_SOL, 16);
        assert_eq!(CONT_LOCAL, 32);
    }

    #[test]
    fn test_completeopt_flags() {
        // completeopt flags
        assert_eq!(K_OPT_COT_FLAG_MENU, 0x01);
        assert_eq!(K_OPT_COT_FLAG_MENUONE, 0x02);
        assert_eq!(K_OPT_COT_FLAG_LONGEST, 0x04);
        assert_eq!(K_OPT_COT_FLAG_FUZZY, 0x80);
        assert_eq!(K_OPT_COT_FLAG_PREINSERT, 0x200);
    }

    #[test]
    fn test_direction_constants() {
        // Direction constants
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }

    #[test]
    fn test_ascii_constants() {
        // CAR and NL should match standard ASCII values
        assert_eq!(CAR, 0x0D); // Carriage Return
        assert_eq!(NL, 0x0A); // Newline / Line Feed
    }

    #[test]
    fn test_ctrl_keys_formula() {
        // Verify control keys follow the formula: Ctrl_X = X - 'A' + 1
        let check_ctrl = |key: u8, expected: c_int| {
            let calculated = c_int::from(key - b'A' + 1);
            assert_eq!(
                calculated, expected,
                "Ctrl-{} should be {}",
                key as char, expected
            );
        };

        check_ctrl(b'D', CTRL_D);
        check_ctrl(b'E', CTRL_E);
        check_ctrl(b'F', CTRL_F);
        check_ctrl(b'I', CTRL_I);
        check_ctrl(b'K', CTRL_K);
        check_ctrl(b'L', CTRL_L);
        check_ctrl(b'N', CTRL_N);
        check_ctrl(b'O', CTRL_O);
        check_ctrl(b'P', CTRL_P);
        check_ctrl(b'Q', CTRL_Q);
        check_ctrl(b'R', CTRL_R);
        check_ctrl(b'S', CTRL_S);
        check_ctrl(b'T', CTRL_T);
        check_ctrl(b'U', CTRL_U);
        check_ctrl(b'V', CTRL_V);
        check_ctrl(b'X', CTRL_X);
        check_ctrl(b'Y', CTRL_Y);
        check_ctrl(b'Z', CTRL_Z);
    }

    #[test]
    fn test_completion_flags_distinct() {
        // Completion flags should not overlap
        assert_eq!(CONT_ADDING & CONT_SOL, 0);
        assert_eq!(CONT_ADDING & CONT_LOCAL, 0);
        assert_eq!(CONT_SOL & CONT_LOCAL, 0);
    }

    #[test]
    fn test_completion_flags_are_powers_of_two() {
        // Completion flags should be powers of two for bit masking
        assert!((CONT_ADDING as u32).is_power_of_two());
        assert!((CONT_SOL as u32).is_power_of_two());
        assert!((CONT_LOCAL as u32).is_power_of_two());
    }

    #[test]
    fn test_completeopt_flags_distinct() {
        // completeopt flags should not overlap
        assert_eq!(K_OPT_COT_FLAG_MENU & K_OPT_COT_FLAG_MENUONE, 0);
    }

    #[test]
    fn test_ctrl_x_modes_with_ident_base_values() {
        // Modes with CTRL_X_WANT_IDENT should have correct base values
        assert_eq!(CTRL_X_TAGS & !CTRL_X_WANT_IDENT, 5);
        assert_eq!(CTRL_X_PATH_PATTERNS & !CTRL_X_WANT_IDENT, 6);
        assert_eq!(CTRL_X_PATH_DEFINES & !CTRL_X_WANT_IDENT, 7);
        assert_eq!(CTRL_X_DICTIONARY & !CTRL_X_WANT_IDENT, 9);
        assert_eq!(CTRL_X_THESAURUS & !CTRL_X_WANT_IDENT, 10);
    }

    #[test]
    fn test_ctrl_x_modes_unique() {
        // All CTRL-X modes should have unique values
        let modes = [
            CTRL_X_NORMAL,
            CTRL_X_NOT_DEFINED_YET,
            CTRL_X_SCROLL,
            CTRL_X_WHOLE_LINE,
            CTRL_X_FILES,
            CTRL_X_TAGS,
            CTRL_X_PATH_PATTERNS,
            CTRL_X_PATH_DEFINES,
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

    #[test]
    fn test_ctrl_rsb_formula() {
        // CTRL_RSB should be ']' - '@' = 93 - 64 = 29
        assert_eq!(CTRL_RSB, i32::from(b']' - b'@'));
    }

    #[test]
    #[allow(clippy::cast_sign_loss)]
    fn test_ctrl_x_want_ident_is_high_bit() {
        // CTRL_X_WANT_IDENT should be a high bit that doesn't overlap with mode numbers
        let want_ident = CTRL_X_WANT_IDENT;
        let register = CTRL_X_REGISTER;
        assert!(
            want_ident > register,
            "CTRL_X_WANT_IDENT should be higher than CTRL_X_REGISTER"
        );
        // CTRL_X_WANT_IDENT is 0x100 which is positive, so casting to u32 is safe
        assert!(
            (want_ident as u32).is_power_of_two(),
            "CTRL_X_WANT_IDENT should be a power of two"
        );
    }
}
