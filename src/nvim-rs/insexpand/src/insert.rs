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

// =============================================================================
// Phase 7: Extended Insert Helper Functions
// =============================================================================

// Additional C accessor functions for Phase 7
extern "C" {
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_autocomplete() -> c_int;

    // Rust functions from lib.rs
    fn rs_ins_compl_has_preinsert() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_EVAL: c_int = 16;

/// Check if deletion should stop completion.
///
/// Returns true if the position is past the completion start in a mode
/// that should stop on delete.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_deletion_stops(new_col: c_int) -> c_int {
    let compl_col = nvim_get_compl_col();
    let mode = nvim_get_ctrl_x_mode();

    // Eval mode always stops on backspace
    if mode == CTRL_X_EVAL {
        return 1;
    }

    // Stop if we delete past the completion start, except omni
    if new_col < compl_col {
        return 1;
    }

    // Stop at the start column for non-omni modes
    if new_col == compl_col && mode != CTRL_X_OMNI {
        return 1;
    }

    0
}

/// Check if we can use backspace during completion.
///
/// Takes into account the backspace option (bs_start flag) and completion length.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_can_bs(new_col: c_int, can_bs_start: c_int) -> c_int {
    let compl_col = nvim_get_compl_col();
    let compl_length = nvim_get_compl_length();

    // If can't backspace before start, check against completion length
    if can_bs_start == 0 {
        let diff = new_col - compl_col - compl_length;
        if diff < 0 {
            return 0; // Would delete into original text
        }
    }

    1
}

/// Check if completion was stopped by interrupt.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Check if we should restart completion after backspace.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_needs_restart(new_col: c_int) -> c_int {
    let compl_col = nvim_get_compl_col();
    let compl_length = nvim_get_compl_length();

    // Need restart if we deleted into the completed area
    c_int::from(new_col <= compl_col + compl_length)
}

/// Check if the completion has preinsert enabled.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_has_preinsert() -> c_int {
    rs_ins_compl_has_preinsert()
}

/// Calculate new leader length after deletion.
///
/// The leader is the text from completion start to cursor.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_new_leader_len(new_col: c_int) -> c_int {
    let compl_col = nvim_get_compl_col();
    let len = new_col - compl_col;
    if len < 0 {
        0
    } else {
        len
    }
}

/// Check if completion should clear selection after backspace.
///
/// Returns true in autocomplete mode when selection should be reset.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_should_clear_selection() -> c_int {
    let autocomplete = nvim_get_compl_autocomplete();
    let started = nvim_get_compl_started();
    let has_preinsert = rs_ins_compl_has_preinsert();

    // Clear selection in autocomplete mode when not using preinsert
    c_int::from(autocomplete != 0 && started != 0 && has_preinsert == 0)
}

/// Get the typed text length relative to completion start.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_typed_len() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let len = cursor_col - compl_col;
    if len < 0 {
        0
    } else {
        len
    }
}

/// Check if the typed length is within valid bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_insert_typed_len_valid() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    c_int::from(cursor_col >= compl_col)
}

// =============================================================================
// Phase 3: rs_ins_compl_delete and rs_ins_compl_insert
// =============================================================================

extern "C" {
    // Accessors for rs_ins_compl_delete
    fn nvim_ins_compl_delete_body(col: c_int) -> c_int;
    fn nvim_set_cursor_col_to_ins_end();

    // Accessors for rs_ins_compl_insert
    fn nvim_compl_shown_cp_str_data() -> *const c_char;
    fn nvim_compl_shown_cp_str_size() -> usize;
    fn nvim_find_common_prefix_data(len_out: *mut usize, icase: c_int) -> *const c_char;
    fn nvim_compl_shown_cp_cpt_source_idx() -> c_int;
    fn nvim_get_cpt_source_startcol(idx: c_int) -> c_int;
    fn nvim_cpt_sources_array_exists() -> c_int;
    fn nvim_ins_compl_expand_multiple_skip(str_ptr: *const c_char, skip: c_int);
    fn nvim_ins_compl_insert_bytes_len(cp_str: *const c_char, compl_len: c_int, ins_len: c_int);
    fn nvim_cursor_col_sub(n: c_int);
    fn nvim_compl_shown_match_at_orig_text() -> c_int;
    fn nvim_ins_compl_dict_alloc_set_shown();
    fn nvim_set_compl_hi_on_longest(val: c_int);
    fn nvim_set_compl_used_match(val: c_int);
    fn rs_compl_status_adding() -> c_int;
    fn rs_ins_compl_preinsert_effect() -> c_int;
    fn rs_ins_compl_leader_len() -> usize;
    fn rs_get_compl_len() -> c_int;
}

/// Delete old completion text before inserting new match.
///
/// This is the Rust implementation of ins_compl_delete(). Orchestrates
/// the deletion via C callbacks for all buffer mutations.
///
/// # Safety
/// Requires valid completion state; called from insert mode only.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_delete(new_leader: c_int) {
    // Calculate orig_col: the common prefix length between orig_text and leader
    let mut orig_col: c_int = 0;
    if new_leader != 0 && rs_compl_status_adding() == 0 {
        let orig = nvim_get_compl_orig_text_data();
        let leader = nvim_get_compl_leader_data();
        let leader = if leader.is_null() { orig } else { leader };

        if !orig.is_null() && !leader.is_null() {
            let mut orig_ptr = orig;
            let mut leader_ptr = leader;
            while *orig_ptr != 0 {
                let c1 = *orig_ptr;
                let c2 = *leader_ptr;
                if c1 == 0 || c2 == 0 || c1 != c2 {
                    let orig_len = rs_utfc_ptr2len(orig_ptr);
                    let leader_len = rs_utfc_ptr2len(leader_ptr);
                    if orig_len != leader_len {
                        break;
                    }
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
                } else {
                    orig_ptr = orig_ptr.add(1);
                    leader_ptr = leader_ptr.add(1);
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
    let mut col = compl_col
        + if rs_compl_status_adding() != 0 {
            compl_length
        } else {
            orig_col
        };

    if rs_ins_compl_preinsert_effect() != 0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        {
            col += rs_ins_compl_leader_len() as c_int;
        }
        nvim_set_cursor_col_to_ins_end();
    }

    // Delegate the buffer mutation to C
    nvim_ins_compl_delete_body(col);
}

/// Insert the new completion text.
///
/// This is the Rust implementation of ins_compl_insert(). Orchestrates
/// the insertion via C callbacks for all buffer mutations.
///
/// # Safety
/// Requires valid completion state; called from insert mode only.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_insert(move_cursor: c_int, insert_prefix: c_int) {
    if nvim_compl_shown_match_exists() == 0 {
        return;
    }

    let compl_len = rs_get_compl_len();
    let preinsert = rs_ins_compl_has_preinsert();
    let leader_len = rs_ins_compl_leader_len();

    let mut cp_str = nvim_compl_shown_cp_str_data();
    let mut cp_str_len = nvim_compl_shown_cp_str_size();

    if insert_prefix != 0 {
        let mut plen: usize = cp_str_len;
        let p = nvim_find_common_prefix_data(&raw mut plen, 0);
        if p.is_null() {
            let mut plen2: usize = cp_str_len;
            let p2 = nvim_find_common_prefix_data(&raw mut plen2, 1);
            if p2.is_null() {
                // keep original cp_str/cp_str_len
            } else {
                cp_str = p2;
                cp_str_len = plen2;
            }
        } else {
            cp_str = p;
            cp_str_len = plen;
        }
    } else if nvim_cpt_sources_array_exists() != 0 {
        let cpt_idx = nvim_compl_shown_cp_cpt_source_idx();
        let compl_col = nvim_get_compl_col();
        if cpt_idx >= 0 && compl_col >= 0 {
            let startcol = nvim_get_cpt_source_startcol(cpt_idx);
            if startcol >= 0 && startcol < compl_col {
                let skip = compl_col - startcol;
                #[allow(clippy::cast_sign_loss)]
                if (skip as usize) <= cp_str_len {
                    #[allow(clippy::cast_sign_loss)]
                    {
                        cp_str_len -= skip as usize;
                        cp_str = cp_str.add(skip as usize);
                    }
                }
            }
        }
    }

    // Insert the bytes if there are more characters than already typed
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    if compl_len < cp_str_len as c_int {
        // Check for newlines
        let has_newline = {
            let mut ptr = cp_str;
            let mut found = false;
            while *ptr != 0 {
                if *ptr == b'\n' as i8 {
                    found = true;
                    break;
                }
                ptr = ptr.add(1);
            }
            found
        };

        if has_newline {
            nvim_ins_compl_expand_multiple_skip(cp_str, compl_len);
        } else {
            let ins_len = if insert_prefix != 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                {
                    cp_str_len as c_int - compl_len
                }
            } else {
                -1
            };
            nvim_ins_compl_insert_bytes_len(cp_str, compl_len, ins_len);
            if (preinsert != 0 || insert_prefix != 0) && move_cursor != 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                {
                    let adjust = (cp_str_len - leader_len) as c_int;
                    if adjust > 0 {
                        nvim_cursor_col_sub(adjust);
                    }
                }
            }
        }
    }

    let used_match =
        nvim_compl_shown_match_at_orig_text() == 0 && (preinsert == 0 || insert_prefix != 0);
    nvim_set_compl_used_match(c_int::from(used_match));

    nvim_ins_compl_dict_alloc_set_shown();
    nvim_set_compl_hi_on_longest(c_int::from(insert_prefix != 0 && move_cursor != 0));
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
