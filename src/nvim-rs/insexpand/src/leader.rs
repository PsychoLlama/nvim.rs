//! Leader and original text management.
//!
//! This module provides functions for managing the completion leader string
//! and original text. The leader is the text typed while completing, and the
//! original text is the text that was present before completion started.

#![allow(dead_code, unused_imports)]
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

/// Get the completion leader data pointer (or orig_text if leader is not set).
#[inline]
unsafe fn leader_get_data() -> *const c_char {
    let leader_data = nvim_get_compl_leader_data();
    if leader_data.is_null() {
        nvim_get_compl_orig_text_data()
    } else {
        leader_data
    }
}

/// Get the length of the completion leader (or orig_text length if leader is not set).
#[inline]
unsafe fn leader_get_len() -> usize {
    let leader_data = nvim_get_compl_leader_data();
    if leader_data.is_null() {
        nvim_get_compl_orig_text_size()
    } else {
        nvim_get_compl_leader_size()
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

// =============================================================================
// Redo buffer fixup
// =============================================================================

extern "C" {
    fn nvim_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_append_char_to_redobuff(c: c_int);
    fn nvim_append_to_redobuff_lit(s: *const c_char, len: c_int);
}

// K_BS = TERMCAP2KEY('k', 'b') = -(('k') + (('b') << 8)) = -25195
const K_BS: c_int = -25195;

/// Fix the redo buffer when the completion leader differs from the original text.
///
/// Compares `compl_orig_text` with the given pointer (or `compl_leader` if
/// `ptr_arg` is null), emits backspaces for removed characters, and appends
/// the new text via `AppendToRedobuffLit`.
///
/// # Safety
/// Requires valid completion state and redo buffer.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ins_compl_fixRedoBufForLeader(ptr_arg: *mut c_char) {
    let mut len: c_int = 0;

    let ptr = if ptr_arg.is_null() {
        let leader_data = nvim_get_compl_leader_data();
        if leader_data.is_null() {
            return; // nothing to do
        }
        leader_data
    } else {
        ptr_arg
    };

    let orig_data = nvim_get_compl_orig_text_data();
    if !orig_data.is_null() {
        let p_start = orig_data;
        // Find length of common prefix between original text and new completion
        while *p_start.offset(len as isize) != 0
            && *p_start.offset(len as isize) == *ptr.offset(len as isize)
        {
            len += 1;
        }
        // Adjust length to not break inside a multi-byte character
        if len > 0 {
            len -= nvim_utf_head_off(p_start, p_start.offset(len as isize));
        }
        // Add backspace characters for each remaining character in original text
        let mut p = p_start.offset(len as isize);
        while *p != 0 {
            nvim_append_char_to_redobuff(K_BS);
            // MB_PTR_ADV equivalent
            p = p.add(nvim_utfc_ptr2len(p) as usize);
        }
    }
    nvim_append_to_redobuff_lit(ptr.offset(len as isize), -1);
}

// =============================================================================
// Phase 8: Leader Update and Cleanup Helpers
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_length() -> c_int;
}

/// Calculate how much extra text was typed beyond the original.
///
/// Returns the byte difference between leader and original text lengths.
/// Positive means more was typed, negative means backspace was used.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_leader_extra_len() -> c_int {
    let leader_len = leader_get_len();
    let orig_len = nvim_get_compl_orig_text_size();
    (leader_len as c_int) - (orig_len as c_int)
}

/// Get how much text to insert from the leader after deletion.
///
/// This is the portion of the leader that extends beyond the common prefix.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_leader_insert_len(compl_len: c_int) -> c_int {
    let leader_len = leader_get_len();

    #[allow(clippy::cast_sign_loss)]
    {
        if compl_len < 0 {
            return leader_len as c_int;
        }
        let to_insert = leader_len.saturating_sub(compl_len as usize);
        to_insert as c_int
    }
}

/// Compare a string with the leader.
///
/// Returns 1 if the string matches the leader up to the leader's length.
/// This is used to check if a match starts with the leader.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_leader_str_matches(s: *const c_char, s_len: usize) -> c_int {
    let leader_data = leader_get_data();
    let leader_len = leader_get_len();

    if leader_data.is_null() || leader_len == 0 {
        return 1; // Empty leader matches everything
    }

    if s.is_null() || s_len < leader_len {
        return 0; // String too short
    }

    // Compare byte by byte
    for i in 0..leader_len {
        if *s.add(i) != *leader_data.add(i) {
            return 0;
        }
    }

    1
}

/// Calculate bytes to delete when updating leader.
///
/// Returns how many bytes need to be deleted before inserting new leader text.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_leader_bytes_to_delete(cursor_col: c_int, compl_col: c_int) -> c_int {
    let diff = cursor_col - compl_col;
    if diff < 0 {
        0
    } else {
        diff
    }
}

// =============================================================================
// Phase 2: rs_ins_compl_bs
// =============================================================================

extern "C" {
    // For ins_compl_bs
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_mb_ptr_back(line: *const c_char, p: *const c_char) -> *const c_char;
    fn nvim_can_bs_start() -> c_int;
    fn nvim_api_clear_and_set_compl_leader(data: *const c_char, len: usize);
    fn nvim_compl_shown_match_is_null() -> c_int;
    fn nvim_compl_set_shown_to_first();
    fn nvim_ins_compl_new_leader_wrapper();
    fn nvim_compl_set_curr_to_shown();
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_compl_first_match_is_null() -> c_int;
    fn rs_ins_compl_preinsert_effect() -> c_int;
    fn rs_ins_compl_delete(new_leader: c_int);
    fn rs_ins_compl_need_restart() -> c_int;
    fn rs_ins_compl_restart();
    fn rs_ins_compl_has_preinsert() -> c_int;
    fn rs_ctrl_x_mode_omni() -> c_int;
    fn rs_ctrl_x_mode_eval() -> c_int;
}

const NUL: c_int = 0;

/// Handle backspace during insert completion.
///
/// Deletes one character before the cursor and shows the subset of matches
/// that match the word now before the cursor.
///
/// Returns K_BS if completion should stop, NUL if work is done.
///
/// # Safety
/// Must be called from insert mode with valid completion state.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_ins_compl_bs() -> c_int {
    if rs_ins_compl_preinsert_effect() != 0 {
        rs_ins_compl_delete(0);
    }

    let line = nvim_get_cursor_line_ptr();
    let cursor_col = nvim_get_cursor_col();
    let p = line.add(cursor_col as usize);
    let p = nvim_mb_ptr_back(line.cast_const(), p.cast_const());
    let p_off = p.offset_from(line.cast_const()) as c_int;

    let compl_col = nvim_get_compl_col();
    let compl_length = nvim_get_compl_length();

    // Stop completion when the whole word was deleted. For Omni completion
    // allow the word to be deleted. Respect the 'backspace' option.
    if p_off - compl_col < 0
        || (p_off - compl_col == 0 && rs_ctrl_x_mode_omni() == 0)
        || rs_ctrl_x_mode_eval() != 0
        || (nvim_can_bs_start() == 0 && p_off - compl_col - compl_length < 0)
    {
        return K_BS;
    }

    // Deleted more than what was used to find matches or didn't finish
    // finding all matches: need to look for matches all over again.
    if nvim_get_cursor_col() <= compl_col + compl_length || rs_ins_compl_need_restart() != 0 {
        rs_ins_compl_restart();
    }

    // rs_ins_compl_restart() calls update_screen() which may invalidate the pointer
    let line = nvim_get_cursor_line_ptr();
    let compl_col = nvim_get_compl_col();

    nvim_api_clear_and_set_compl_leader(
        line.add(compl_col as usize).cast_const(),
        (p_off - compl_col) as usize,
    );

    // Clear selection if a menu item is currently selected in autocompletion
    if nvim_get_compl_autocomplete() != 0
        && nvim_compl_first_match_is_null() == 0
        && rs_ins_compl_has_preinsert() == 0
    {
        nvim_compl_set_shown_to_first();
    }

    nvim_ins_compl_new_leader_wrapper();
    if nvim_compl_shown_match_is_null() == 0 {
        // Make sure current match is not a hidden item.
        nvim_compl_set_curr_to_shown();
    }
    NUL
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
