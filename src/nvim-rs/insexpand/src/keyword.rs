//! Default (keyword) completion support.
//!
//! This module provides helper functions for default keyword completion
//! (CTRL-N / CTRL-P without CTRL-X prefix).
//! The core keyword scanning operations remain in C.

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

/// Check if we're in normal (keyword) completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_normal_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_NORMAL)
}

/// Check if completion is searching forward.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_forward() -> c_int {
    let dir = nvim_get_compl_direction();
    c_int::from(dir == FORWARD)
}

/// Check if completion is searching backward.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_backward() -> c_int {
    let dir = nvim_get_compl_direction();
    c_int::from(dir == BACKWARD)
}

/// Check if completion was interrupted during keyword search.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for keyword search.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_get_direction() -> c_int {
    nvim_get_compl_direction()
}

// =============================================================================
// Phase 3: Keyword Completion Engine Functions
// =============================================================================

/// Check if keyword completion is active.
///
/// Returns true if completion started and in normal mode.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_active() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    let started = nvim_get_compl_started();
    c_int::from(mode == CTRL_X_NORMAL && started != 0)
}

/// Check if we're in "adding" mode for keyword completion.
///
/// Returns true if CONT_ADDING flag is set.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_adding() -> c_int {
    let cont_status = nvim_get_compl_cont_status();
    c_int::from((cont_status & CONT_ADDING) != 0)
}

/// Check if keyword search should start at beginning of line.
///
/// Returns true if CONT_SOL flag is set.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_start_at_sol() -> c_int {
    let cont_status = nvim_get_compl_cont_status();
    c_int::from((cont_status & CONT_SOL) != 0)
}

/// Check if keyword search is local to current buffer.
///
/// Returns true if CONT_LOCAL flag is set.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_local() -> c_int {
    let cont_status = nvim_get_compl_cont_status();
    c_int::from((cont_status & CONT_LOCAL) != 0)
}

/// Get the minimum length required for keyword matches.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_min_length() -> c_int {
    nvim_get_compl_length()
}

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

/// Check if a character at the pointer is a word character.
///
/// Returns true if the character class is > 1 (not whitespace or non-word).
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_word_char(ptr: *const c_char) -> c_int {
    if ptr.is_null() || *ptr == 0 {
        return 0;
    }
    c_int::from(rs_mb_get_class(ptr) > 1)
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

/// Check if keyword completion can continue with the same pattern.
///
/// Returns true if completion is active and can continue.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_can_continue() -> c_int {
    let started = nvim_get_compl_started();
    let interrupted = nvim_get_compl_interrupted();
    c_int::from(started != 0 && interrupted == 0)
}

/// Get continuation status flags.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_cont_status() -> c_int {
    nvim_get_compl_cont_status()
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
