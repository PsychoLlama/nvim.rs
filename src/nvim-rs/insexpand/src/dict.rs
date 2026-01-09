//! Dictionary and thesaurus completion support.
//!
//! This module provides helper functions for dictionary and thesaurus completion.
//! The core file I/O and regex operations remain in C due to their complexity,
//! but Rust provides utilities for string processing and state management.

use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    // CTRL-X mode checking
    fn nvim_get_ctrl_x_mode() -> c_int;

    // State accessors
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;

    // UTF-8 functions
    fn rs_utfc_ptr2len(ptr: *const c_char) -> c_int;
    fn rs_mb_get_class(ptr: *const c_char) -> c_int;
}

// CTRL-X mode constants
const CTRL_X_THESAURUS: c_int = 13;
const CTRL_X_DICTIONARY: c_int = 9 + 0x100; // 9 + CTRL_X_WANT_IDENT

/// Check if we're in dictionary completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_is_dictionary_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_DICTIONARY)
}

/// Check if we're in thesaurus completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_is_thesaurus_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_THESAURUS)
}

/// Check if we're in any dictionary-like mode (dictionary or thesaurus).
#[no_mangle]
pub unsafe extern "C" fn rs_dict_is_dict_like_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_DICTIONARY || mode == CTRL_X_THESAURUS)
}

/// Check if completion was interrupted during dictionary search.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for dictionary search.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_get_direction() -> c_int {
    nvim_get_compl_direction()
}

/// Skip whitespace and punctuation to find word start.
///
/// This is similar to find_word_start but specifically for thesaurus processing.
/// Returns a pointer to the first character of the next word.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_find_word_start(mut ptr: *mut c_char) -> *mut c_char {
    // Skip whitespace and punctuation (class <= 1)
    while *ptr != 0 && *ptr != b'\n' as c_char && rs_mb_get_class(ptr) <= 1 {
        ptr = ptr.add(rs_utfc_ptr2len(ptr) as usize);
    }
    ptr
}

/// Find the end of a word for thesaurus processing.
///
/// Unlike the standard word end finder, this handles Japanese words
/// where characters may be in different classes, only separating words
/// with single-byte non-word characters.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_find_word_end(mut ptr: *mut c_char) -> *mut c_char {
    while *ptr != 0 {
        let len = rs_utfc_ptr2len(ptr);
        // For multi-byte characters, continue regardless of class
        if len > 1 {
            ptr = ptr.add(len as usize);
        } else if rs_mb_get_class(ptr) <= 1 {
            // Single-byte non-word character - stop here
            break;
        } else {
            ptr = ptr.add(1);
        }
    }
    ptr
}

/// Calculate the length of a word from start to end pointers.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_dict_word_len(start: *const c_char, end: *const c_char) -> c_int {
    if start.is_null() || end.is_null() || end < start {
        return 0;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        end.offset_from(start) as c_int
    }
}

/// Check if a word matches another word (case-sensitive).
///
/// Returns 1 if words match exactly, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_dict_words_match(
    word1: *const c_char,
    len1: c_int,
    word2: *const c_char,
    len2: c_int,
) -> c_int {
    if word1.is_null() || word2.is_null() || len1 != len2 || len1 < 0 {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    for i in 0..len1 as usize {
        if *word1.add(i) != *word2.add(i) {
            return 0;
        }
    }

    1
}

/// Check if a line is empty or contains only whitespace.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_line_is_empty(line: *const c_char) -> c_int {
    if line.is_null() {
        return 1;
    }

    let mut ptr = line;
    while *ptr != 0 && *ptr != b'\n' as c_char {
        // If we find any non-whitespace character
        if *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
            return 0;
        }
        ptr = ptr.add(1);
    }

    1
}

/// Skip a word in the line (move past current word and following whitespace).
///
/// Useful for iterating through words in a thesaurus line.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_skip_word(ptr: *mut c_char) -> *mut c_char {
    // Find end of current word
    let end = rs_dict_find_word_end(ptr);
    // Then find start of next word
    rs_dict_find_word_start(end)
}

/// Count words in a line.
///
/// Returns the number of whitespace-separated words.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_dict_count_words_in_line(mut ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut count = 0;

    loop {
        // Skip whitespace
        while *ptr != 0 && *ptr != b'\n' as c_char {
            if *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
                break;
            }
            ptr = ptr.add(1);
        }

        // End of line?
        if *ptr == 0 || *ptr == b'\n' as c_char {
            break;
        }

        // Found a word
        count += 1;

        // Skip to end of word
        while *ptr != 0 && *ptr != b'\n' as c_char {
            if *ptr == b' ' as c_char || *ptr == b'\t' as c_char {
                break;
            }
            ptr = ptr.add(1);
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_THESAURUS, 13);
        assert_eq!(CTRL_X_DICTIONARY, 9 + 0x100);
    }

    #[test]
    fn test_word_len_null() {
        unsafe {
            assert_eq!(rs_dict_word_len(std::ptr::null(), std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_word_len() {
        unsafe {
            let s = b"hello\0";
            let start = s.as_ptr().cast::<c_char>();
            let end = start.add(5);
            assert_eq!(rs_dict_word_len(start, end), 5);
        }
    }

    #[test]
    fn test_words_match() {
        unsafe {
            let w1 = b"hello\0";
            let w2 = b"hello\0";
            let w3 = b"world\0";

            assert_eq!(
                rs_dict_words_match(
                    w1.as_ptr().cast::<c_char>(),
                    5,
                    w2.as_ptr().cast::<c_char>(),
                    5
                ),
                1
            );
            assert_eq!(
                rs_dict_words_match(
                    w1.as_ptr().cast::<c_char>(),
                    5,
                    w3.as_ptr().cast::<c_char>(),
                    5
                ),
                0
            );
        }
    }

    #[test]
    fn test_words_match_different_lengths() {
        unsafe {
            let w1 = b"hi\0";
            let w2 = b"hello\0";

            assert_eq!(
                rs_dict_words_match(
                    w1.as_ptr().cast::<c_char>(),
                    2,
                    w2.as_ptr().cast::<c_char>(),
                    5
                ),
                0
            );
        }
    }

    #[test]
    fn test_line_is_empty() {
        unsafe {
            let empty = b"\0";
            let whitespace = b"   \t  \0";
            let content = b"  hello  \0";

            assert_eq!(rs_dict_line_is_empty(empty.as_ptr().cast::<c_char>()), 1);
            assert_eq!(
                rs_dict_line_is_empty(whitespace.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(rs_dict_line_is_empty(content.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_count_words() {
        unsafe {
            let one = b"hello\0";
            let three = b"one two three\0";
            let spaces = b"  word  \0";
            let empty = b"   \0";

            assert_eq!(
                rs_dict_count_words_in_line(one.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(
                rs_dict_count_words_in_line(three.as_ptr().cast::<c_char>()),
                3
            );
            assert_eq!(
                rs_dict_count_words_in_line(spaces.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(
                rs_dict_count_words_in_line(empty.as_ptr().cast::<c_char>()),
                0
            );
        }
    }
}
