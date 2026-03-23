//! Completion insert and delete operations.
//!
//! This module provides helper functions for inserting and deleting completion text.
//! The core operations remain in C due to heavy buffer interaction, but Rust provides
//! the logic and utilities for common operations.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    // State accessors
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_cursor_col() -> c_int;

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

    // UTF-8 functions
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;
}

// Continuation status flags
const CONT_ADDING: c_int = 1;

/// Check if we're in "adding" mode (compl_cont_status & CONT_ADDING).
#[inline]
unsafe fn is_compl_adding() -> bool {
    (crate::vars::nvim_get_compl_cont_status() & CONT_ADDING) != 0
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

    // Rust functions from lib.rs
    fn rs_ins_compl_has_preinsert() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_EVAL: c_int = 16;

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
    // rs_find_common_prefix is defined in Rust (leader.rs)
    fn nvim_compl_shown_cp_cpt_source_idx() -> c_int;
    fn nvim_get_cpt_source_startcol(idx: c_int) -> c_int;
    fn nvim_cpt_sources_array_exists() -> c_int;
    fn nvim_ins_compl_expand_multiple_skip(str_ptr: *const c_char, skip: c_int);
    fn nvim_ins_compl_insert_bytes_len(cp_str: *const c_char, compl_len: c_int, ins_len: c_int);
    fn nvim_ins_compl_insert_bytes(p: *const c_char, len: c_int);
    fn nvim_cursor_col_sub(n: c_int);
    fn nvim_compl_shown_match_at_orig_text() -> c_int;
    fn nvim_ins_compl_dict_alloc_set_shown();
    // (compl_hi_on_autocompl_longest moved to Rust static in state.rs)
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
                    let orig_len = utfc_ptr2len(orig_ptr);
                    let leader_len = utfc_ptr2len(leader_ptr);
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
    let compl_length = crate::vars::nvim_get_compl_length();
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
        let p = crate::leader::rs_find_common_prefix(&raw mut plen, 0);
        if p.is_null() {
            let mut plen2: usize = cp_str_len;
            let p2 = crate::leader::rs_find_common_prefix(&raw mut plen2, 1);
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
    crate::vars::nvim_set_compl_used_match(c_int::from(used_match));

    nvim_ins_compl_dict_alloc_set_shown();
    crate::state::COMPL_HI_ON_AUTOCOMPL_LONGEST = insert_prefix != 0 && move_cursor != 0;
}

// =============================================================================
// Phase 4: rs_ins_compl_set_original_text and rs_ins_compl_addleader
// =============================================================================

extern "C" {
    // Accessors for rs_ins_compl_set_original_text
    fn nvim_ins_compl_set_original_text_impl(str_ptr: *const c_char, len: usize);

    // Accessors for rs_ins_compl_addleader
    fn stop_arrow() -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn nvim_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn ins_char(c: c_int);
    fn ins_char_bytes(buf: *const c_char, len: usize);
    fn rs_ins_compl_need_restart() -> c_int;
    fn rs_ins_compl_restart();
    fn nvim_api_clear_compl_leader();
    fn nvim_set_compl_leader_from_cursor();
}

/// Set the original text for the first completion match.
///
/// Replaces `compl_first_match->cp_str` (or its `cp_prev` if that's the
/// original text entry) with a copy of `str`.
///
/// # Safety
/// Requires valid completion list state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_set_original_text(str_ptr: *const c_char, len: usize) {
    nvim_ins_compl_set_original_text_impl(str_ptr, len);
}

// Maximum multi-byte character size in bytes
const MB_MAXCHAR: usize = 4;

/// Append one character to the completion match leader.
///
/// May reduce the number of matches. Called when the user types a character
/// while completion is active.
///
/// # Safety
/// Requires valid completion state; called from insert mode only.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ins_compl_addleader(c: c_int) {
    if rs_ins_compl_preinsert_effect() != 0 {
        rs_ins_compl_delete(0);
    }

    if stop_arrow() != 0 {
        // stop_arrow() returned FAIL
        return;
    }

    let cc = utf_char2len(c);
    if cc > 1 {
        let mut buf = [0i8; MB_MAXCHAR + 1];
        nvim_utf_char2bytes(c, buf.as_mut_ptr());
        buf[cc as usize] = 0;
        ins_char_bytes(buf.as_ptr(), cc as usize);
    } else {
        ins_char(c);
    }

    // If we didn't complete finding matches we must search again.
    if rs_ins_compl_need_restart() != 0 {
        rs_ins_compl_restart();
    }

    nvim_api_clear_compl_leader();
    nvim_set_compl_leader_from_cursor();
    crate::leader::rs_ins_compl_new_leader();
}

// =============================================================================
// Phase 6 (pass 4): rs_ins_compl_insert_bytes and rs_ins_compl_expand_multiple
// =============================================================================

/// Insert bytes at the cursor position and update compl_ins_end_col.
///
/// Delegates to the C compound accessor `nvim_ins_compl_insert_bytes`.
/// When `len` is -1, the full NUL-terminated length is used.
///
/// # Safety
/// `p` must point to a valid byte sequence of at least `len` bytes (or be
/// NUL-terminated when `len == -1`). Requires valid buffer and cursor state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_insert_bytes(p: *const c_char, len: c_int) {
    nvim_ins_compl_insert_bytes(p, len);
}

/// Insert a completion string that may contain newlines.
///
/// Delegates to the C compound accessor `nvim_ins_compl_expand_multiple_skip`
/// with `skip = 0` (no prefix to skip).
///
/// # Safety
/// `str_ptr` must point to a valid NUL-terminated C string.
/// Requires valid buffer, cursor, and indent state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_expand_multiple(str_ptr: *const c_char) {
    nvim_ins_compl_expand_multiple_skip(str_ptr, 0);
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
