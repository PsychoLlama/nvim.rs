//! Completion insert and delete operations.
//!
//! This module provides helper functions for inserting and deleting completion text.
//! The core operations remain in C due to heavy buffer interaction, but Rust provides
//! the logic and utilities for common operations.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

use crate::match_list::compl_shown_match;

// C accessor functions
extern "C" {
    // State accessors
    fn nvim_get_cursor_col() -> c_int;

    // (nvim_compl_shown_match_has_newline: inlined in match_list.rs)

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
    // nvim_ins_compl_delete_body: deleted (Phase 16), inlined below
    fn nvim_set_cursor_col(col: c_int);
    // Helpers for inlined nvim_ins_compl_delete_body
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn get_cursor_line_len() -> c_int;
    #[link_name = "get_cursor_pos_ptr"]
    fn get_cursor_pos_ptr_insert() -> *mut c_char;
    fn get_cursor_pos_len() -> c_int;
    fn ml_delete(lnum: c_int) -> c_int;
    fn deleted_lines_mark(lnum: c_int, count: c_int);
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn backspace_until_column(col: c_int);
    fn ins_str(str_: *const c_char, len: usize);
    fn changed_cline_bef_curs(wp: *mut u8);
    fn nvim_get_curwin() -> *mut u8;
    fn nvim_set_completed_item_empty();
    #[link_name = "xfree"]
    fn xfree_ins(p: *mut u8);

    // Accessors for rs_ins_compl_insert
    // rs_find_common_prefix is defined in Rust (leader.rs)
    // (nvim_get_cpt_source_startcol, nvim_cpt_sources_array_exists: inlined in vars.rs Phase 23)
    // nvim_ins_compl_expand_multiple_skip: deleted (Phase 17), inlined below
    // Helpers for inlined nvim_ins_compl_expand_multiple_skip
    fn get_indent() -> c_int;
    fn open_line(
        dir: c_int,
        flags: c_int,
        second_line_indent: c_int,
        did_do_comment: *mut bool,
    ) -> bool;
    // nvim_ins_compl_insert_bytes: deleted (Phase 2), inlined below via ins_bytes_len
    #[link_name = "ins_bytes_len"]
    fn ins_bytes_len(p: *const c_char, len: usize);
    fn nvim_compl_set_vim_var_dict_shown();
    // (compl_hi_on_autocompl_longest moved to Rust static in state.rs)
    fn rs_compl_status_adding() -> c_int;
    fn rs_ins_compl_preinsert_effect() -> c_int;
    fn rs_ins_compl_leader_len() -> usize;
    fn rs_get_compl_len() -> c_int;
}

// Constants for expand_multiple_skip_impl
const FORWARD_INSERT: c_int = 1;
const OPENLINE_KEEPTRAIL_INSERT: c_int = 0x04;
const OPENLINE_FORCE_INDENT_INSERT: c_int = 0x40;

/// Insert a completion string that may contain newlines, skipping a prefix.
///
/// Inlined from the deleted C function `nvim_ins_compl_expand_multiple_skip` (Phase 17).
/// Walks the string starting at `str_ptr + skip`, inserting characters and opening new
/// lines as needed.
///
/// # Safety
/// `str_ptr` must point to a valid NUL-terminated C string.
/// Requires valid buffer, cursor, and indent state.
#[allow(clippy::cast_possible_wrap)]
unsafe fn expand_multiple_skip_impl(str_ptr: *const c_char, skip: c_int) {
    #[allow(clippy::cast_sign_loss)]
    let mut start: *mut c_char = str_ptr.add(skip as usize).cast_mut();
    let mut curr = start;
    let base_indent = get_indent();
    while *curr != 0 {
        if *curr == b'\n' as c_char {
            if curr > start {
                #[allow(clippy::cast_sign_loss)]
                ins_char_bytes(start.cast_const(), curr.offset_from(start) as usize);
            }
            open_line(
                FORWARD_INSERT,
                OPENLINE_KEEPTRAIL_INSERT | OPENLINE_FORCE_INDENT_INSERT,
                base_indent,
                core::ptr::null_mut(),
            );
            start = curr.add(1);
        }
        curr = curr.add(1);
    }
    if curr > start {
        #[allow(clippy::cast_sign_loss)]
        ins_char_bytes(start.cast_const(), curr.offset_from(start) as usize);
    }
    crate::vars::nvim_set_compl_ins_end_col(nvim_get_cursor_col());
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
        let orig = crate::vars::nvim_get_compl_orig_text_data();
        let leader = crate::vars::nvim_get_compl_leader_data();
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

    let compl_col = crate::vars::nvim_get_compl_col();
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
        nvim_set_cursor_col(crate::vars::nvim_get_compl_ins_end_col());
    }

    // Inline nvim_ins_compl_delete_body (Phase 16):
    let compl_lnum = crate::vars::nvim_get_compl_lnum();
    let mut remaining_data: *mut c_char = core::ptr::null_mut();
    let mut remaining_size: usize = 0;

    #[allow(clippy::cast_sign_loss)]
    if nvim_get_curwin_cursor_lnum() > compl_lnum {
        if nvim_get_cursor_col() < get_cursor_line_len() {
            let pos_len = get_cursor_pos_len();
            let tmp =
                cbuf_to_string_insert(get_cursor_pos_ptr_insert().cast_const(), pos_len as usize);
            remaining_data = tmp.data;
            remaining_size = tmp.size;
        }
        while nvim_get_curwin_cursor_lnum() > compl_lnum {
            let lnum = nvim_get_curwin_cursor_lnum();
            if ml_delete(lnum) == 0 {
                xfree_ins(remaining_data.cast());
                return;
            }
            deleted_lines_mark(lnum, 1);
            nvim_set_cursor_lnum(nvim_get_curwin_cursor_lnum() - 1);
        }
        nvim_set_cursor_col(get_cursor_line_len());
    }

    if nvim_get_cursor_col() > col {
        if stop_arrow() == 0 {
            xfree_ins(remaining_data.cast());
            return;
        }
        backspace_until_column(col);
        crate::vars::nvim_set_compl_ins_end_col(nvim_get_cursor_col());
    }

    if !remaining_data.is_null() {
        let orig_col = nvim_get_cursor_col();
        ins_str(remaining_data.cast_const(), remaining_size);
        nvim_set_cursor_col(orig_col);
        xfree_ins(remaining_data.cast());
    }

    changed_cline_bef_curs(nvim_get_curwin());
    nvim_set_completed_item_empty();
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
    if compl_shown_match.is_null() {
        return;
    }

    let compl_len = rs_get_compl_len();
    let preinsert = rs_ins_compl_has_preinsert();
    let leader_len = rs_ins_compl_leader_len();

    let mut cp_str = crate::match_list::shown_match_cp_str_data();
    let mut cp_str_len = crate::match_list::shown_match_cp_str_size();

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
    } else if crate::vars::nvim_cpt_sources_array_exists() != 0 {
        let cpt_idx = crate::match_list::shown_match_cpt_source_idx();
        let compl_col = crate::vars::nvim_get_compl_col();
        if cpt_idx >= 0 && compl_col >= 0 {
            let startcol = crate::vars::nvim_get_cpt_source_startcol(cpt_idx);
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
            expand_multiple_skip_impl(cp_str, compl_len);
        } else {
            let ins_len = if insert_prefix != 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                {
                    cp_str_len as c_int - compl_len
                }
            } else {
                -1
            };
            rs_ins_compl_insert_bytes(cp_str.add(compl_len as usize), ins_len);
            if (preinsert != 0 || insert_prefix != 0) && move_cursor != 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                {
                    let adjust = (cp_str_len - leader_len) as c_int;
                    if adjust > 0 {
                        nvim_set_cursor_col(nvim_get_cursor_col() - adjust);
                    }
                }
            }
        }
    }

    let used_match =
        !crate::match_list::shown_match_at_orig_text() && (preinsert == 0 || insert_prefix != 0);
    crate::vars::nvim_set_compl_used_match(c_int::from(used_match));

    nvim_compl_set_vim_var_dict_shown();
    crate::state::COMPL_HI_ON_AUTOCOMPL_LONGEST = insert_prefix != 0 && move_cursor != 0;
}

// =============================================================================
// Phase 4: rs_ins_compl_set_original_text and rs_ins_compl_addleader
// =============================================================================

extern "C" {
    // Accessors for rs_ins_compl_set_original_text
    #[link_name = "nvim_compl_match_at_original_text"]
    fn nvim_compl_match_at_original_text_insert(m: *mut std::ffi::c_void) -> c_int;
    #[link_name = "nvim_compl_match_get_prev"]
    fn nvim_compl_match_get_prev_insert(m: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn nvim_compl_match_replace_cp_str(m: *mut std::ffi::c_void, s: *const c_char, l: usize);

    // Accessors for rs_ins_compl_addleader
    fn stop_arrow() -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn nvim_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn ins_char(c: c_int);
    fn ins_char_bytes(buf: *const c_char, len: usize);
    fn rs_ins_compl_need_restart() -> c_int;
    fn rs_ins_compl_restart();
    // nvim_api_clear_compl_leader: inlined in vars.rs as nvim_compl_clear_leader (Phase 25)
    // nvim_set_compl_leader_from_cursor: deleted (Phase 12), inlined below
    // Helpers for inlined nvim_set_compl_leader_from_cursor
    #[link_name = "get_cursor_line_ptr"]
    fn get_cursor_line_ptr_insert() -> *mut c_char;
    #[link_name = "cbuf_to_string"]
    fn cbuf_to_string_insert(buf: *const c_char, size: usize) -> crate::vars::NvimString;
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
    let first = crate::match_list::nvim_compl_get_first_match().0;
    if first.is_null() {
        return;
    }
    if nvim_compl_match_at_original_text_insert(first) != 0 {
        nvim_compl_match_replace_cp_str(first, str_ptr, len);
    } else {
        let prev = nvim_compl_match_get_prev_insert(first);
        if !prev.is_null() && nvim_compl_match_at_original_text_insert(prev) != 0 {
            nvim_compl_match_replace_cp_str(prev, str_ptr, len);
        }
    }
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

    crate::vars::nvim_compl_clear_leader();
    // nvim_set_compl_leader_from_cursor inlined (Phase 12):
    #[allow(clippy::cast_sign_loss)]
    {
        let line_ptr = get_cursor_line_ptr_insert();
        let compl_col = crate::vars::nvim_get_compl_col();
        let cursor_col = nvim_get_cursor_col();
        debug_assert!(cursor_col >= compl_col);
        let leader_len = (cursor_col - compl_col) as usize;
        crate::vars::compl_leader =
            cbuf_to_string_insert(line_ptr.add(compl_col as usize).cast_const(), leader_len);
    }
    crate::leader::rs_ins_compl_new_leader();
}

// =============================================================================
// Phase 6 (pass 4): rs_ins_compl_insert_bytes and rs_ins_compl_expand_multiple
// =============================================================================

/// Insert bytes at the cursor position and update compl_ins_end_col.
///
/// When `len` is -1, the full NUL-terminated length is used.
///
/// # Safety
/// `p` must point to a valid byte sequence of at least `len` bytes (or be
/// NUL-terminated when `len == -1`). Requires valid buffer and cursor state.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ins_compl_insert_bytes(p: *const c_char, len: c_int) {
    let actual_len = if len == -1 {
        std::ffi::CStr::from_ptr(p).to_bytes().len()
    } else {
        debug_assert!(len >= 0);
        len as usize
    };
    ins_bytes_len(p, actual_len);
    crate::vars::nvim_set_compl_ins_end_col(nvim_get_cursor_col());
}

/// Insert a completion string that may contain newlines.
///
/// Insert a completion string that may contain newlines.
///
/// Wraps `expand_multiple_skip_impl` with `skip = 0` (no prefix to skip).
///
/// # Safety
/// `str_ptr` must point to a valid NUL-terminated C string.
/// Requires valid buffer, cursor, and indent state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_expand_multiple(str_ptr: *const c_char) {
    expand_multiple_skip_impl(str_ptr, 0);
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
