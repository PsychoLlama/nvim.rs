//! Leader and original text management.
//!
//! This module provides functions for managing the completion leader string
//! and original text. The leader is the text typed while completing, and the
//! original text is the text that was present before completion started.

use std::os::raw::{c_char, c_int};

// C accessor functions for leader and original text
extern "C" {
    // Leader data
    fn nvim_get_compl_leader_data() -> *const c_char;
    fn nvim_get_compl_leader_size() -> usize;

    // Original text data
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_orig_text_size() -> usize;

    // Cursor and column accessors
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_get_compl_col() -> c_int;

    // UTF-8 functions
    fn rs_utfc_ptr2len(ptr: *const c_char) -> c_int;
    fn rs_mb_get_class(ptr: *const c_char) -> c_int;
}

// ASCII constants
const CAR: c_char = 0x0D; // '\015' carriage return
const NL: c_char = 0x0A; // '\012' newline

/// Get the completion leader string data pointer.
///
/// Returns compl_leader.data if set, otherwise compl_orig_text.data.
/// This is the text that has been typed so far during completion.
#[no_mangle]
pub unsafe extern "C" fn rs_leader_get_data() -> *const c_char {
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
pub unsafe extern "C" fn rs_leader_get_len() -> usize {
    let leader_data = nvim_get_compl_leader_data();
    if leader_data.is_null() {
        nvim_get_compl_orig_text_size()
    } else {
        nvim_get_compl_leader_size()
    }
}

/// Check if the leader is set (not using original text as fallback).
#[no_mangle]
pub unsafe extern "C" fn rs_leader_is_set() -> c_int {
    c_int::from(!nvim_get_compl_leader_data().is_null())
}

/// Get the length of the original text.
#[no_mangle]
pub unsafe extern "C" fn rs_orig_text_get_len() -> usize {
    nvim_get_compl_orig_text_size()
}

/// Get the original text data pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_orig_text_get_data() -> *const c_char {
    nvim_get_compl_orig_text_data()
}

/// Get the completion length (cursor column - completion start column).
///
/// This is how much text has been typed from the completion start.
/// Returns 0 if the result would be negative.
#[no_mangle]
pub unsafe extern "C" fn rs_leader_compl_len() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let off = cursor_col - compl_col;
    if off < 0 {
        0
    } else {
        off
    }
}

/// Find the start of the next word.
///
/// Skips over whitespace and non-word characters.
/// Returns a pointer to the first char of the word. Also stops at NUL.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_leader_find_word_start(mut ptr: *mut c_char) -> *mut c_char {
    // Skip characters that are not part of a word (class <= 1)
    while *ptr != 0 && *ptr != b'\n' as c_char && rs_mb_get_class(ptr) <= 1 {
        ptr = ptr.add(rs_utfc_ptr2len(ptr) as usize);
    }
    ptr
}

/// Find the end of the word. Assumes it starts inside a word.
///
/// Returns a pointer to just after the word.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_leader_find_word_end(mut ptr: *mut c_char) -> *mut c_char {
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

/// Find the end of the line, omitting CR and NL at the end.
///
/// Returns a pointer to just after the line content (before trailing CR/NL).
#[no_mangle]
pub unsafe extern "C" fn rs_leader_find_line_end(ptr: *mut c_char) -> *mut c_char {
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

/// Calculate the common prefix length between two strings.
///
/// Returns the byte length of the common prefix.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_common_prefix_len(s1: *const c_char, s2: *const c_char) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }

    let mut len = 0;
    while *s1.add(len) != 0 && *s1.add(len) == *s2.add(len) {
        len += 1;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        len as c_int
    }
}

/// Compare the leader with original text to check if they match.
///
/// Returns 1 if they match (same content and length), 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_leader_matches_orig() -> c_int {
    let leader_data = nvim_get_compl_leader_data();
    let orig_data = nvim_get_compl_orig_text_data();

    // If leader is not set, they "match" (leader falls back to orig)
    if leader_data.is_null() {
        return 1;
    }

    // If orig is null but leader is set, no match
    if orig_data.is_null() {
        return 0;
    }

    let leader_size = nvim_get_compl_leader_size();
    let orig_size = nvim_get_compl_orig_text_size();

    // Different lengths means no match
    if leader_size != orig_size {
        return 0;
    }

    // Compare byte by byte
    for i in 0..leader_size {
        if *leader_data.add(i) != *orig_data.add(i) {
            return 0;
        }
    }

    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_constants() {
        assert_eq!(CAR, 0x0D);
        assert_eq!(NL, 0x0A);
    }

    #[test]
    fn test_common_prefix_len_null() {
        unsafe {
            assert_eq!(rs_common_prefix_len(std::ptr::null(), std::ptr::null()), 0);
            let s = b"test\0";
            assert_eq!(
                rs_common_prefix_len(s.as_ptr().cast::<c_char>(), std::ptr::null()),
                0
            );
        }
    }

    #[test]
    fn test_common_prefix_len_same() {
        unsafe {
            let s1 = b"hello\0";
            let s2 = b"hello\0";
            assert_eq!(
                rs_common_prefix_len(s1.as_ptr().cast::<c_char>(), s2.as_ptr().cast::<c_char>()),
                5
            );
        }
    }

    #[test]
    fn test_common_prefix_len_partial() {
        unsafe {
            let s1 = b"hello\0";
            let s2 = b"help\0";
            assert_eq!(
                rs_common_prefix_len(s1.as_ptr().cast::<c_char>(), s2.as_ptr().cast::<c_char>()),
                3
            );
        }
    }

    #[test]
    fn test_common_prefix_len_no_common() {
        unsafe {
            let s1 = b"abc\0";
            let s2 = b"xyz\0";
            assert_eq!(
                rs_common_prefix_len(s1.as_ptr().cast::<c_char>(), s2.as_ptr().cast::<c_char>()),
                0
            );
        }
    }
}
