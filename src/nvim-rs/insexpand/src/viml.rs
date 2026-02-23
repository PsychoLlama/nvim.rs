//! VimL function support for completion.
//!
//! This module provides helper functions for VimL completion functions.
//! The actual VimL function implementations remain in C (eval/funcs.c),
//! but Rust provides utilities for completion info gathering.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_compl_length() -> c_int;
    fn nvim_get_compl_matches() -> c_int;
    fn nvim_get_compl_selected_item() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_pum_visible() -> c_int;
    fn nvim_get_pum_want_item() -> c_int;

    // Match info accessors
    fn nvim_compl_first_match_is_null() -> c_int;
    fn nvim_compl_curr_match_is_null() -> c_int;
}

// CTRL-X mode constants (for mode string conversion)
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
const CTRL_X_EVAL: c_int = 16;
const CTRL_X_CMDLINE_CTRL_X: c_int = 17;
const CTRL_X_BUFNAMES: c_int = 18;
const CTRL_X_REGISTER: c_int = 19;

// Mode strings (static, null-terminated)
static MODE_KEYWORD: &[u8] = b"keyword\0";
static MODE_WHOLE_LINE: &[u8] = b"whole_line\0";
static MODE_FILES: &[u8] = b"files\0";
static MODE_TAGS: &[u8] = b"tags\0";
static MODE_PATH_DEFINES: &[u8] = b"path_defines\0";
static MODE_PATH_PATTERNS: &[u8] = b"path_patterns\0";
static MODE_DICTIONARY: &[u8] = b"dictionary\0";
static MODE_THESAURUS: &[u8] = b"thesaurus\0";
static MODE_CMDLINE: &[u8] = b"cmdline\0";
static MODE_FUNCTION: &[u8] = b"function\0";
static MODE_OMNI: &[u8] = b"omni\0";
static MODE_SPELL: &[u8] = b"spell\0";
static MODE_EVAL: &[u8] = b"eval\0";
static MODE_BUFNAMES: &[u8] = b"buffer\0";
static MODE_REGISTER: &[u8] = b"register\0";
static MODE_UNKNOWN: &[u8] = b"unknown\0";

/// Get the completion mode as a short string.
///
/// Returns a static string describing the current completion mode.
/// This is used by complete_info() for the "mode" field.
#[no_mangle]
#[allow(clippy::match_same_arms)]
pub unsafe extern "C" fn rs_viml_get_mode_str() -> *const c_char {
    let mode = nvim_get_ctrl_x_mode();

    let s = match mode {
        CTRL_X_NORMAL | CTRL_X_NOT_DEFINED_YET | CTRL_X_SCROLL => MODE_KEYWORD,
        CTRL_X_WHOLE_LINE => MODE_WHOLE_LINE,
        CTRL_X_FILES => MODE_FILES,
        CTRL_X_TAGS => MODE_TAGS,
        CTRL_X_PATH_DEFINES => MODE_PATH_DEFINES,
        CTRL_X_PATH_PATTERNS => MODE_PATH_PATTERNS,
        CTRL_X_DICTIONARY => MODE_DICTIONARY,
        CTRL_X_THESAURUS => MODE_THESAURUS,
        CTRL_X_CMDLINE | CTRL_X_CMDLINE_CTRL_X => MODE_CMDLINE,
        CTRL_X_FUNCTION => MODE_FUNCTION,
        CTRL_X_OMNI => MODE_OMNI,
        CTRL_X_SPELL => MODE_SPELL,
        CTRL_X_EVAL => MODE_EVAL,
        CTRL_X_BUFNAMES => MODE_BUFNAMES,
        CTRL_X_REGISTER => MODE_REGISTER,
        _ => MODE_UNKNOWN,
    };

    s.as_ptr().cast::<c_char>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_constants() {
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
        assert_eq!(CTRL_X_SCROLL, 2);
        assert_eq!(CTRL_X_WHOLE_LINE, 3);
        assert_eq!(CTRL_X_FILES, 4);
    }

    #[test]
    fn test_mode_with_ident() {
        assert_eq!(CTRL_X_TAGS, 5 + CTRL_X_WANT_IDENT);
        assert_eq!(CTRL_X_DICTIONARY, 9 + CTRL_X_WANT_IDENT);
    }

    #[test]
    fn test_mode_str_static() {
        // Verify the mode strings are valid C strings (null-terminated)
        assert_eq!(MODE_KEYWORD[MODE_KEYWORD.len() - 1], 0);
        assert_eq!(MODE_UNKNOWN[MODE_UNKNOWN.len() - 1], 0);
    }
}
