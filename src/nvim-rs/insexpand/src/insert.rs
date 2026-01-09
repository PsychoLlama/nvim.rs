//! Completion insert and delete operations.
//!
//! This module provides helper functions for inserting and deleting completion text.
//! The core operations remain in C due to heavy buffer interaction, but Rust provides
//! the logic and utilities for common operations.

use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    // State accessors
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_compl_length() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_get_compl_ins_end_col() -> c_int;

    // Match accessors
    fn nvim_compl_shown_match_exists() -> c_int;
    fn nvim_compl_shown_match_str_size() -> usize;
    #[allow(dead_code)]
    fn nvim_compl_shown_match_has_newline() -> c_int;

    // Leader accessors
    fn nvim_get_compl_leader_data() -> *const c_char;
    fn nvim_get_compl_leader_size() -> usize;
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_orig_text_size() -> usize;

    // Status functions
    fn nvim_get_compl_cont_status() -> c_int;

    // UTF-8 functions
    fn rs_utfc_ptr2len(ptr: *const c_char) -> c_int;
}

// Continuation status flags
const CONT_ADDING: c_int = 1;

/// Check if we're in "adding" mode (compl_cont_status & CONT_ADDING).
#[inline]
unsafe fn is_compl_adding() -> bool {
    (nvim_get_compl_cont_status() & CONT_ADDING) != 0
}

/// Get the completion length from start column to cursor column.
///
/// This is how much text has been typed for the completion.
/// Returns 0 if the result would be negative.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_get_compl_len() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let off = cursor_col - compl_col;
    if off < 0 {
        0
    } else {
        off
    }
}

/// Calculate the column where deletion should start.
///
/// Used by ins_compl_delete to calculate the target column.
/// If new_leader is true and we're not adding, calculates the common
/// prefix length between original text and leader.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_calc_delete_col(new_leader: c_int) -> c_int {
    let mut orig_col = 0;

    if new_leader != 0 && !is_compl_adding() {
        let orig = nvim_get_compl_orig_text_data();
        let leader = nvim_get_compl_leader_data();

        // Use original text if leader is not set
        let leader = if leader.is_null() { orig } else { leader };

        if !orig.is_null() && !leader.is_null() {
            let mut orig_ptr = orig;
            let mut leader_ptr = leader;

            // Find common prefix
            while *orig_ptr != 0 {
                // Compare characters (simplified - full UTF-8 comparison in C)
                if *orig_ptr != *leader_ptr {
                    break;
                }

                let orig_len = rs_utfc_ptr2len(orig_ptr);
                let leader_len = rs_utfc_ptr2len(leader_ptr);

                if orig_len != leader_len {
                    break;
                }

                // Check full multi-byte character
                let mut same = true;
                #[allow(clippy::cast_sign_loss)]
                for i in 0..orig_len as usize {
                    if *orig_ptr.add(i) != *leader_ptr.add(i) {
                        same = false;
                        break;
                    }
                }

                if !same {
                    break;
                }

                #[allow(clippy::cast_sign_loss)]
                {
                    orig_ptr = orig_ptr.add(orig_len as usize);
                    leader_ptr = leader_ptr.add(leader_len as usize);
                }
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            {
                orig_col = orig_ptr.offset_from(orig) as c_int;
            }
        }
    }

    let compl_col = nvim_get_compl_col();
    let compl_length = nvim_get_compl_length();

    compl_col
        + if is_compl_adding() {
            compl_length
        } else {
            orig_col
        }
}

/// Check if the completion string is longer than what's been typed.
///
/// Returns true if we need to insert more characters.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_needs_more_chars() -> c_int {
    if nvim_compl_shown_match_exists() == 0 {
        return 0;
    }

    let compl_len = rs_insert_get_compl_len();
    let str_size = nvim_compl_shown_match_str_size();

    #[allow(clippy::cast_sign_loss)]
    {
        c_int::from((compl_len as usize) < str_size)
    }
}

/// Calculate how many characters to insert from the completion string.
///
/// Returns the number of bytes to insert (completion string length minus typed length).
/// Returns 0 if nothing needs to be inserted.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_calc_bytes_to_insert() -> c_int {
    if nvim_compl_shown_match_exists() == 0 {
        return 0;
    }

    let compl_len = rs_insert_get_compl_len();
    let str_size = nvim_compl_shown_match_str_size();

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    {
        let to_insert = str_size.saturating_sub(compl_len as usize);
        to_insert as c_int
    }
}

/// Get the leader length for cursor adjustment.
///
/// Returns the leader length, falling back to original text length if not set.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_get_leader_len() -> usize {
    let leader_data = nvim_get_compl_leader_data();
    if leader_data.is_null() {
        nvim_get_compl_orig_text_size()
    } else {
        nvim_get_compl_leader_size()
    }
}

/// Calculate cursor adjustment for preinsert mode.
///
/// Returns the amount to move the cursor back after insertion.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_calc_cursor_adjust(cp_str_len: usize) -> c_int {
    let leader_len = rs_insert_get_leader_len();

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        cp_str_len.saturating_sub(leader_len) as c_int
    }
}

/// Check if the insertion column is valid.
///
/// Returns true if the insertion end column is greater than the completion column.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_col_is_valid() -> c_int {
    let compl_col = nvim_get_compl_col();
    let ins_end_col = nvim_get_compl_ins_end_col();
    c_int::from(ins_end_col > compl_col)
}

/// Find the common prefix length between two strings.
///
/// Stops at NUL or when characters differ.
/// Returns the byte length of the common prefix.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_insert_common_prefix_len(
    s1: *const c_char,
    s2: *const c_char,
) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }

    let mut len = 0usize;
    while *s1.add(len) != 0 && *s1.add(len) == *s2.add(len) {
        len += 1;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        len as c_int
    }
}

/// Count newlines in a string.
///
/// Returns the number of newline characters found.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_insert_count_newlines(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut ptr = s;
    while *ptr != 0 {
        if *ptr == b'\n' as c_char {
            count += 1;
        }
        ptr = ptr.add(1);
    }

    count
}

/// Check if a string contains any newlines.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_insert_has_newlines(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let mut ptr = s;
    while *ptr != 0 {
        if *ptr == b'\n' as c_char {
            return 1;
        }
        ptr = ptr.add(1);
    }

    0
}

/// Find the position of the first newline in a string.
///
/// Returns the byte offset of the newline, or -1 if not found.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_insert_find_newline(s: *const c_char) -> c_int {
    if s.is_null() {
        return -1;
    }

    let mut offset = 0;
    let mut ptr = s;
    while *ptr != 0 {
        if *ptr == b'\n' as c_char {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            {
                return offset as c_int;
            }
        }
        ptr = ptr.add(1);
        offset += 1;
    }

    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cont_adding_flag() {
        assert_eq!(CONT_ADDING, 1);
    }

    #[test]
    fn test_common_prefix_len_null() {
        unsafe {
            assert_eq!(
                rs_insert_common_prefix_len(std::ptr::null(), std::ptr::null()),
                0
            );
        }
    }

    #[test]
    fn test_common_prefix_len() {
        unsafe {
            let s1 = b"hello\0";
            let s2 = b"help\0";
            assert_eq!(
                rs_insert_common_prefix_len(
                    s1.as_ptr().cast::<c_char>(),
                    s2.as_ptr().cast::<c_char>()
                ),
                3
            );
        }
    }

    #[test]
    fn test_count_newlines() {
        unsafe {
            let s = b"line1\nline2\nline3\0";
            assert_eq!(rs_insert_count_newlines(s.as_ptr().cast::<c_char>()), 2);
        }
    }

    #[test]
    fn test_count_newlines_none() {
        unsafe {
            let s = b"no newlines here\0";
            assert_eq!(rs_insert_count_newlines(s.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_has_newlines() {
        unsafe {
            let s1 = b"has\nnewline\0";
            let s2 = b"no newline\0";
            assert_eq!(rs_insert_has_newlines(s1.as_ptr().cast::<c_char>()), 1);
            assert_eq!(rs_insert_has_newlines(s2.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_find_newline() {
        unsafe {
            let s1 = b"hello\nworld\0";
            let s2 = b"no newline\0";
            assert_eq!(rs_insert_find_newline(s1.as_ptr().cast::<c_char>()), 5);
            assert_eq!(rs_insert_find_newline(s2.as_ptr().cast::<c_char>()), -1);
        }
    }

    #[test]
    fn test_find_newline_null() {
        unsafe {
            assert_eq!(rs_insert_find_newline(std::ptr::null()), -1);
        }
    }
}
