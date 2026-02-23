//! Default (keyword) completion support.
//!
//! This module provides helper functions for default keyword completion
//! (CTRL-N / CTRL-P without CTRL-X prefix).
//! The core keyword scanning operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;
    fn nvim_get_compl_length() -> c_int;

    // UTF-8 and character class functions
    fn rs_utfc_ptr2len(ptr: *const c_char) -> c_int;
    fn rs_mb_get_class(ptr: *const c_char) -> c_int;
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;

// Direction constants
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

// Completion continuation status flags
const CONT_ADDING: c_int = 1;
const CONT_SOL: c_int = 16;
const CONT_LOCAL: c_int = 32;

// =============================================================================
// Phase 3: Keyword Completion Engine Functions
// =============================================================================

/// Check if a word at the given pointer is long enough to be a match.
///
/// Counts UTF-8 characters and compares with compl_length.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_keyword_word_is_long_enough(
    ptr: *const c_char,
    end: *const c_char,
) -> c_int {
    if ptr.is_null() || end.is_null() || ptr >= end {
        return 0;
    }

    let min_len = nvim_get_compl_length();
    if min_len <= 0 {
        return 1; // No minimum length requirement
    }

    let mut char_count = 0;
    let mut p = ptr;

    while p < end {
        let char_len = rs_utfc_ptr2len(p);
        if char_len <= 0 {
            break;
        }
        char_count += 1;
        if char_count >= min_len {
            return 1; // Long enough
        }
        p = p.add(char_len as usize);
    }

    0 // Not long enough
}

/// Skip whitespace and non-word characters.
///
/// Returns pointer to the first word character or end of string.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_keyword_skip_non_word(mut ptr: *mut c_char) -> *mut c_char {
    if ptr.is_null() {
        return ptr;
    }

    while *ptr != 0 && rs_mb_get_class(ptr) <= 1 {
        let char_len = rs_utfc_ptr2len(ptr);
        if char_len <= 0 {
            break;
        }
        ptr = ptr.add(char_len as usize);
    }

    ptr
}

/// Skip to end of current word.
///
/// Assumes pointer is at start of a word. Returns pointer past end of word.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_keyword_skip_word(mut ptr: *mut c_char) -> *mut c_char {
    if ptr.is_null() || *ptr == 0 {
        return ptr;
    }

    let start_class = rs_mb_get_class(ptr);
    if start_class <= 1 {
        return ptr; // Not in a word
    }

    while *ptr != 0 {
        let char_len = rs_utfc_ptr2len(ptr);
        if char_len <= 0 {
            break;
        }
        ptr = ptr.add(char_len as usize);
        if rs_mb_get_class(ptr) != start_class {
            break;
        }
    }

    ptr
}

/// Count UTF-8 characters in a byte range.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_keyword_count_chars(ptr: *const c_char, len: c_int) -> c_int {
    if ptr.is_null() || len <= 0 {
        return 0;
    }

    let mut count = 0;
    let mut pos = 0usize;
    let end = len as usize;

    while pos < end {
        let char_len = rs_utfc_ptr2len(ptr.add(pos));
        if char_len <= 0 {
            break;
        }
        pos += char_len as usize;
        count += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_NORMAL, 0);
    }
}
