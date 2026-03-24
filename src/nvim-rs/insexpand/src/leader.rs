//! Leader and original text management.
//!
//! This module provides functions for managing the completion leader string
//! and original text. The leader is the text typed while completing, and the
//! original text is the text that was present before completion started.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions for leader and original text
extern "C" {
    // Cursor and column accessors
    fn nvim_get_cursor_col() -> c_int;

    // UTF-8 functions
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;
    fn mb_get_class(ptr: *const c_char) -> c_int;
}

// ASCII constants
const CAR: c_char = 0x0D; // '\015' carriage return
const NL: c_char = 0x0A; // '\012' newline

/// Get the completion leader data pointer (or orig_text if leader is not set).
#[inline]
unsafe fn leader_get_data() -> *const c_char {
    let leader_data = crate::vars::nvim_get_compl_leader_data();
    if leader_data.is_null() {
        crate::vars::nvim_get_compl_orig_text_data()
    } else {
        leader_data
    }
}

/// Get the length of the completion leader (or orig_text length if leader is not set).
#[inline]
unsafe fn leader_get_len() -> usize {
    let leader_data = crate::vars::nvim_get_compl_leader_data();
    if leader_data.is_null() {
        crate::vars::nvim_get_compl_orig_text_size()
    } else {
        crate::vars::nvim_get_compl_leader_size()
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
    while *ptr != 0 && *ptr != b'\n' as c_char && mb_get_class(ptr) <= 1 {
        ptr = ptr.add(utfc_ptr2len(ptr) as usize);
    }
    ptr
}

/// Find the end of the word. Assumes it starts inside a word.
///
/// Returns a pointer to just after the word.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_leader_find_word_end(mut ptr: *mut c_char) -> *mut c_char {
    let start_class = mb_get_class(ptr);
    if start_class > 1 {
        while *ptr != 0 {
            ptr = ptr.add(utfc_ptr2len(ptr) as usize);
            if mb_get_class(ptr) != start_class {
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
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;
    #[link_name = "AppendCharToRedobuff"]
    fn nvim_append_char_to_redobuff(c: c_int);
    #[link_name = "AppendToRedobuffLit"]
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
        let leader_data = crate::vars::nvim_get_compl_leader_data();
        if leader_data.is_null() {
            return; // nothing to do
        }
        leader_data
    } else {
        ptr_arg
    };

    let orig_data = crate::vars::nvim_get_compl_orig_text_data();
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
            len -= utf_head_off(p_start, p_start.offset(len as isize));
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

/// Calculate how much extra text was typed beyond the original.
///
/// Returns the byte difference between leader and original text lengths.
/// Positive means more was typed, negative means backspace was used.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_leader_extra_len() -> c_int {
    let leader_len = leader_get_len();
    let orig_len = crate::vars::nvim_get_compl_orig_text_size();
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
// Phase 1 (pass 11): get_leader_for_startcol + prepend_startcol_text
// =============================================================================

extern "C" {
    fn nvim_ml_get_curline() -> *const c_char;
    // nvim_get_compl_col is in crate::vars
    // (nvim_cpt_sources_array_exists, nvim_get_cpt_source_startcol: inlined in vars.rs Phase 23)
    fn nvim_xfree(ptr: *mut u8);
    fn nvim_xmalloc(size: usize) -> *mut u8;
    fn nvim_compl_match_get_cpt_source_idx(m: crate::match_list::ComplMatch) -> c_int;
}

/// Cache for the adjusted leader string.
/// When non-zero, holds a heap-allocated (via nvim_xmalloc) byte buffer
/// (pointer + size). Freed via nvim_xfree before reallocation.
/// NULL pointer means no cached value.
static mut ADJUSTED_LEADER_PTR: *mut c_char = std::ptr::null_mut();
static mut ADJUSTED_LEADER_SIZE: usize = 0;

/// Clear the adjusted leader cache, freeing any xmalloc'd memory.
///
/// # Safety
/// Must only be called from single-threaded context (Neovim main thread).
#[allow(clippy::ptr_cast_constness)]
unsafe fn adjusted_leader_clear() {
    // Safety: single-threaded; raw pointer read avoids static_mut_refs lint
    let ptr = std::ptr::read(&raw const ADJUSTED_LEADER_PTR);
    if !ptr.is_null() {
        nvim_xfree(ptr.cast::<u8>());
        std::ptr::write(&raw mut ADJUSTED_LEADER_PTR, std::ptr::null_mut());
        std::ptr::write(&raw mut ADJUSTED_LEADER_SIZE, 0);
    }
}

/// Constructs a string by prepending the line bytes from `startcol` up
/// to `compl_col` before `src`.
///
/// Returns a heap-allocated buffer (via nvim_xmalloc) and its length.
///
/// # Safety
/// Requires valid C allocation state (nvim_xmalloc / nvim_xfree).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
unsafe fn prepend_startcol_text_rs(
    src_data: *const c_char,
    src_size: usize,
    startcol: c_int,
) -> (*mut c_char, usize) {
    let compl_col = crate::vars::nvim_get_compl_col();
    let prepend_len = (compl_col - startcol) as usize;
    let new_length = prepend_len + src_size;

    let buf = nvim_xmalloc(new_length + 1).cast::<c_char>();
    let line = nvim_ml_get_curline();

    // copy line[startcol..compl_col]
    std::ptr::copy_nonoverlapping(line.add(startcol as usize), buf, prepend_len);
    // copy src
    std::ptr::copy_nonoverlapping(src_data, buf.add(prepend_len), src_size);
    // NUL-terminate
    *buf.add(new_length) = 0;

    (buf, new_length)
}

/// Returns the completion leader data adjusted for a specific source's startcol.
///
/// If the source's startcol is before compl_col, prepends text from the
/// buffer line to the original compl_leader. Uses a module-level static cache.
///
/// If `match` is null, clears the cache and returns null.
///
/// # Safety
/// Requires valid completion and buffer state.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_cast_constness
)]
pub unsafe extern "C" fn rs_get_leader_for_startcol_data(
    match_: crate::match_list::ComplMatch,
    cached: c_int,
) -> *const c_char {
    // Clear-cache call: match == NULL
    if match_.is_null() {
        adjusted_leader_clear();
        return std::ptr::null();
    }

    // No cpt_sources or no leader -> return compl_leader directly
    if crate::vars::nvim_cpt_sources_array_exists() == 0 {
        return crate::vars::nvim_get_compl_leader_data();
    }
    let leader_data = crate::vars::nvim_get_compl_leader_data();
    if leader_data.is_null() {
        return leader_data;
    }

    let cpt_idx = nvim_compl_match_get_cpt_source_idx(match_);
    let compl_col = crate::vars::nvim_get_compl_col();
    if cpt_idx < 0 || compl_col <= 0 {
        return leader_data;
    }

    let startcol = crate::vars::nvim_get_cpt_source_startcol(cpt_idx);
    if startcol >= 0 && startcol < compl_col {
        let leader_size = crate::vars::nvim_get_compl_leader_size();
        let prepend_len = (compl_col - startcol) as usize;
        let new_length = prepend_len + leader_size;

        // Return cached if sizes match
        if cached != 0 {
            let cached_ptr = std::ptr::read(&raw const ADJUSTED_LEADER_PTR);
            let cached_size = std::ptr::read(&raw const ADJUSTED_LEADER_SIZE);
            if !cached_ptr.is_null() && cached_size == new_length {
                return cached_ptr.cast_const();
            }
        }

        // Free old cache and allocate new one
        adjusted_leader_clear();
        let (buf, len) = prepend_startcol_text_rs(leader_data, leader_size, startcol);
        std::ptr::write(&raw mut ADJUSTED_LEADER_PTR, buf);
        std::ptr::write(&raw mut ADJUSTED_LEADER_SIZE, len);
        buf.cast_const()
    } else {
        leader_data
    }
}

/// Returns the byte length of the adjusted leader for a specific source's startcol.
///
/// # Safety
/// Requires valid completion and buffer state.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_get_leader_for_startcol_size(
    match_: crate::match_list::ComplMatch,
    cached: c_int,
) -> usize {
    // Clear-cache call
    if match_.is_null() {
        return 0;
    }

    if crate::vars::nvim_cpt_sources_array_exists() == 0 {
        return crate::vars::nvim_get_compl_leader_size();
    }
    let leader_data = crate::vars::nvim_get_compl_leader_data();
    if leader_data.is_null() {
        return 0;
    }

    let cpt_idx = nvim_compl_match_get_cpt_source_idx(match_);
    let compl_col = crate::vars::nvim_get_compl_col();
    if cpt_idx < 0 || compl_col <= 0 {
        return crate::vars::nvim_get_compl_leader_size();
    }

    let startcol = crate::vars::nvim_get_cpt_source_startcol(cpt_idx);
    if startcol >= 0 && startcol < compl_col {
        let leader_size = crate::vars::nvim_get_compl_leader_size();
        let prepend_len = (compl_col - startcol) as usize;
        let new_length = prepend_len + leader_size;

        // Return from cache if available and size matches
        if cached != 0 {
            let cached_ptr = std::ptr::read(&raw const ADJUSTED_LEADER_PTR);
            let cached_size = std::ptr::read(&raw const ADJUSTED_LEADER_SIZE);
            if !cached_ptr.is_null() && cached_size == new_length {
                return cached_size;
            }
        }
        new_length
    } else {
        crate::vars::nvim_get_compl_leader_size()
    }
}

// =============================================================================
// Phase 2: rs_ins_compl_bs
// =============================================================================

use crate::match_list::{
    compl_first_match, compl_shown_match, is_first_match, nvim_compl_get_curr_match,
    nvim_compl_get_first_match, nvim_compl_get_shown_match, nvim_compl_set_curr_match,
    nvim_compl_set_shown_match,
};

extern "C" {
    // For ins_compl_bs
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_mb_ptr_back(line: *const c_char, p: *const c_char) -> *const c_char;
    // nvim_can_bs_start: deleted (Phase 1), use can_bs(BS_START) directly
    #[link_name = "can_bs"]
    fn can_bs(what: c_int) -> c_int;
    fn nvim_api_clear_and_set_compl_leader(data: *const c_char, len: usize);
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

    let compl_col = crate::vars::nvim_get_compl_col();
    let compl_length = crate::vars::nvim_get_compl_length();

    // Stop completion when the whole word was deleted. For Omni completion
    // allow the word to be deleted. Respect the 'backspace' option.
    if p_off - compl_col < 0
        || (p_off - compl_col == 0 && rs_ctrl_x_mode_omni() == 0)
        || rs_ctrl_x_mode_eval() != 0
        || (can_bs(c_int::from(b's')) == 0 && p_off - compl_col - compl_length < 0) // BS_START = 's' (from option_vars.h)
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
    let compl_col = crate::vars::nvim_get_compl_col();

    nvim_api_clear_and_set_compl_leader(
        line.add(compl_col as usize).cast_const(),
        (p_off - compl_col) as usize,
    );

    // Clear selection if a menu item is currently selected in autocompletion
    if crate::vars::nvim_get_compl_autocomplete() != 0
        && !compl_first_match.is_null()
        && rs_ins_compl_has_preinsert() == 0
    {
        compl_shown_match = compl_first_match;
    }

    rs_ins_compl_new_leader();
    if !compl_shown_match.is_null() {
        // Make sure current match is not a hidden item.
        nvim_compl_set_curr_match(compl_shown_match);
    }
    NUL
}

// =============================================================================
// Phase 4: rs_ins_compl_new_leader
// =============================================================================

extern "C" {
    // For ins_compl_new_leader
    // nvim_get_p_acl: inlined in vars.rs (Phase 28)
    #[link_name = "pum_undisplay"]
    fn nvim_pum_undisplay(undo: c_int);
    // nvim_redraw_later_valid: deleted (Phase 1), use redraw_later(curwin, UPD_VALID) directly
    #[link_name = "redraw_later"]
    fn redraw_later(wp: *mut std::ffi::c_void, r#type: c_int);
    #[link_name = "curwin"]
    static mut g_curwin: *mut std::ffi::c_void;
    fn nvim_update_screen();
    fn nvim_ui_flush();
    fn rs_ins_compl_set_original_text(str_ptr: *const c_char, len: usize);
    fn nvim_is_cpt_func_refresh_always() -> c_int;
    fn nvim_cpt_compl_refresh();
    fn rs_cot_fuzzy() -> c_int;
    fn rs_ins_compl_fuzzy_sort();
    // (compl_restarting moved to Rust static in state.rs)
    fn rs_ins_compl_has_autocomplete() -> c_int;
    fn rs_ins_compl_enable_autocomplete();
    // nvim_ins_complete_ctrl_n: deleted (Phase 1), use ins_complete(CTRL_N, 1) directly
    #[link_name = "ins_complete"]
    fn ins_complete(c: c_int, enable_pum: c_int) -> c_int;
    fn rs_ins_compl_show_pum();
    fn rs_ins_compl_insert(move_cursor: c_int, insert_prefix: c_int);
    fn rs_ins_compl_preinsert_longest() -> c_int;
    fn rs_get_compl_len() -> c_int;
    fn nvim_ins_compl_insert_bytes(p: *const c_char, len: c_int);
    fn rs_ins_compl_refresh_always() -> c_int;
}

const FAIL: c_int = 0;

/// Called after changing compl_leader.
///
/// Shows the popup menu with a different set of matches.
/// May also search for matches again if the previous search was interrupted.
///
/// # Safety
/// Requires valid completion list state; called from insert mode only.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ins_compl_new_leader() {
    crate::pum::rs_ins_compl_del_pum();
    crate::insert::rs_ins_compl_delete(1);
    let leader_data = crate::vars::nvim_get_compl_leader_data();
    if !leader_data.is_null() {
        let compl_len = rs_get_compl_len();
        nvim_ins_compl_insert_bytes(leader_data.add(compl_len as usize), -1);
    }
    crate::vars::nvim_set_compl_used_match(0);

    if crate::vars::nvim_get_p_acl() > 0 {
        nvim_pum_undisplay(1);
        redraw_later(g_curwin, 10); // UPD_VALID = 10
        nvim_update_screen();
        nvim_ui_flush();
    }

    if crate::vars::nvim_get_compl_started() != 0 {
        let leader_data = crate::vars::nvim_get_compl_leader_data();
        let leader_size = crate::vars::nvim_get_compl_leader_size();
        if !leader_data.is_null() {
            rs_ins_compl_set_original_text(leader_data, leader_size);
        }
        if nvim_is_cpt_func_refresh_always() != 0 {
            nvim_cpt_compl_refresh();
        }
        if rs_cot_fuzzy() != 0 {
            rs_ins_compl_fuzzy_sort();
        }
    } else {
        crate::vars::nvim_set_spell_bad_len(0);
        // Matches were cleared, need to search for them now.
        // Set compl_restarting to avoid that the first match is inserted.
        crate::state::COMPL_RESTARTING = true;
        if rs_ins_compl_has_autocomplete() != 0 {
            rs_ins_compl_enable_autocomplete();
        } else {
            crate::vars::nvim_set_compl_autocomplete(0);
        }
        if ins_complete(14, 1) == FAIL { // CTRL_N = 14, enable_pum = 1
            crate::vars::nvim_set_compl_cont_status(0);
        }
        crate::state::COMPL_RESTARTING = false;
    }

    crate::vars::nvim_set_compl_enter_selects(c_int::from(
        crate::vars::nvim_get_compl_used_match() == 0
            && crate::vars::nvim_get_compl_selected_item() != -1,
    ));

    // Show the popup menu with a different set of matches.
    rs_ins_compl_show_pum();

    // Don't let Enter select the original text when there is no popup menu.
    if crate::vars::nvim_get_compl_match_array_exists() == 0 {
        crate::vars::nvim_set_compl_enter_selects(0);
    } else if rs_ins_compl_has_preinsert() != 0 && crate::vars::nvim_get_compl_leader_size() > 0 {
        rs_ins_compl_insert(1, 0);
    } else if crate::vars::nvim_get_compl_started() != 0
        && rs_ins_compl_preinsert_longest() != 0
        && crate::vars::nvim_get_compl_leader_size() > 0
        && rs_ins_compl_preinsert_effect() == 0
    {
        rs_ins_compl_insert(1, 1);
    }
    // Don't let Enter select when use user function and refresh_always is set
    if rs_ins_compl_refresh_always() != 0 {
        crate::vars::nvim_set_compl_enter_selects(0);
    }
}

// =============================================================================
// Phase 2 (pass 12): ins_compl_longest_match -- full Rust implementation
// =============================================================================

use crate::match_list::ComplMatch;

extern "C" {
    fn nvim_utf_ptr2char(p: *const c_char) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;
    fn nvim_compl_match_get_flags(m: ComplMatch) -> c_int;
    #[link_name = "ins_redraw"]
    fn nvim_ins_redraw(ready: c_int);
}

// CP_ICASE flag (must match C definition)
const CP_ICASE: c_int = 16;

/// Reduce compl_leader to the longest common prefix with the given match.
///
/// Rust port of C `ins_compl_longest_match()`. Handles the first-match case
/// (sets leader = match's cp_str) and the subsequent case (truncates leader
/// at first character that differs, considering CP_ICASE flag).
///
/// # Safety
/// `m` must be a valid, non-null completion match pointer.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_ins_compl_longest_match(m: ComplMatch) {
    let leader_data = crate::vars::nvim_get_compl_leader_data();

    if leader_data.is_null() {
        // First match: use the whole cp_str as the leader.
        let cp_data = nvim_compl_match_get_cp_str_data(m);
        let cp_size = nvim_compl_match_get_cp_str_size(m);
        nvim_api_clear_and_set_compl_leader(cp_data, cp_size);

        let had_match = nvim_get_cursor_col() > crate::vars::nvim_get_compl_col();
        let leader_after = crate::vars::nvim_get_compl_leader_data();
        let compl_len = rs_get_compl_len() as usize;
        rs_ins_compl_delete(0);
        nvim_ins_compl_insert_bytes(leader_after.add(compl_len), -1);
        nvim_ins_redraw(0);

        if !had_match {
            rs_ins_compl_delete(0);
        }
        crate::vars::nvim_set_compl_used_match(0);
        return;
    }

    // Subsequent match: find longest common prefix with current leader.
    let match_data = nvim_compl_match_get_cp_str_data(m);
    let flags = nvim_compl_match_get_flags(m);
    let icase = (flags & CP_ICASE) != 0;

    let mut p = leader_data;
    let mut s = match_data;

    while *p != 0 {
        let c1 = nvim_utf_ptr2char(p);
        let c2 = nvim_utf_ptr2char(s);

        let differs = if icase {
            mb_tolower(c1) != mb_tolower(c2)
        } else {
            c1 != c2
        };

        if differs {
            break;
        }

        // MB_PTR_ADV: advance by utfc_ptr2len bytes
        p = p.add(utfc_ptr2len(p) as usize);
        s = s.add(utfc_ptr2len(s) as usize);
    }

    if *p != 0 {
        // Leader was shortened -- update it.
        let new_len = p.offset_from(leader_data) as usize;

        // Build truncated copy of current leader and set it.
        // We read from leader_data which is valid for compl_leader.size bytes.
        let leader_bytes = std::slice::from_raw_parts(leader_data.cast::<u8>(), new_len);
        let mut buf = leader_bytes.to_vec();
        buf.push(0); // NUL terminate (cbuf_to_string doesn't need it but it's safe)
        nvim_api_clear_and_set_compl_leader(buf.as_ptr().cast::<c_char>(), new_len);

        let had_match = nvim_get_cursor_col() > crate::vars::nvim_get_compl_col();
        let new_leader = crate::vars::nvim_get_compl_leader_data();
        let compl_len = rs_get_compl_len() as usize;
        rs_ins_compl_delete(0);
        nvim_ins_compl_insert_bytes(new_leader.add(compl_len), -1);
        nvim_ins_redraw(0);

        if !had_match {
            rs_ins_compl_delete(0);
        }
    }

    crate::vars::nvim_set_compl_used_match(0);
}

// =============================================================================
// Phase 3 (pass 11): find_common_prefix migrated from C
// =============================================================================

extern "C" {
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;
    fn nvim_compl_match_clear_icase(m: ComplMatch);
    fn nvim_compl_match_get_cp_str_data(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_cp_str_size(m: ComplMatch) -> usize;
    // (nvim_get_cpt_source_cs_flag, nvim_get_cpt_source_cs_max_matches: inlined in vars.rs Phase 23)
    fn xcalloc(count: usize, size: usize) -> *mut u8;
    // nvim_xfree already declared above
    fn nvim_get_p_inf() -> c_int;
    fn nvim_ignorecase(pat: *const c_char) -> bool;
    fn rs_ins_compl_equal(m: ComplMatch, str_: *const c_char, len: usize) -> c_int;
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn rs_ins_compl_leader() -> *const c_char;
    fn rs_ins_compl_leader_len() -> usize;
    fn nvim_mb_byte2len(b: c_int) -> c_int;
    fn nvim_get_curwin_cursor_col() -> c_int;
    fn rs_ascii_iswhite_or_nul(c: c_int) -> c_int;
    fn rs_find_word_end(ptr: *mut c_char) -> *mut c_char;
    // nvim_get_cursor_line_ptr already declared in Phase 2 extern block above
}

/// Find the longest common prefix among the current completion matches.
///
/// Returns a pointer to the first match string (C-owned), with `*prefix_len`
/// set to the byte length of the common prefix. Returns NULL if no prefix
/// longer than the current leader was found, or if not a cpt-completion.
///
/// `curbuf_only` restricts matches to the current buffer ('.' source).
///
/// # Safety
/// `prefix_len` must be a valid non-null pointer. The returned pointer is
/// valid as long as the completion match list is not modified.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_cast_constness,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_find_common_prefix(
    prefix_len: *mut usize,
    curbuf_only: c_int,
) -> *const c_char {
    // Only applies to cpt-completion
    if crate::vars::nvim_cpt_sources_array_exists() == 0 {
        return std::ptr::null();
    }

    let sources_count = crate::vars::nvim_get_cpt_sources_count();
    if sources_count <= 0 {
        return std::ptr::null();
    }

    #[allow(clippy::cast_ptr_alignment)]
    let match_count = xcalloc(sources_count as usize, std::mem::size_of::<c_int>()).cast::<c_int>();

    // Clear the adjusted-leader cache
    let _ = rs_get_leader_for_startcol_data(ComplMatch::null(), 1);

    let mut compl = nvim_compl_get_first_match();
    let mut first: *const c_char = std::ptr::null();
    let mut len: c_int = -1;

    'outer: loop {
        if compl.is_null() {
            break;
        }
        // Stop when we wrap back to the first match
        if !first.is_null() && is_first_match(compl) {
            break;
        }

        let leader_data = rs_get_leader_for_startcol_data(compl, 1);
        let leader_size = rs_get_leader_for_startcol_size(compl, 1);

        // Apply 'smartcase' behaviour in normal ctrl-x mode
        if rs_ctrl_x_mode_normal() != 0
            && nvim_get_p_inf() == 0
            && !leader_data.is_null()
            && !nvim_ignorecase(leader_data)
        {
            nvim_compl_match_clear_icase(compl);
        }

        if nvim_compl_match_at_original_text(compl) == 0
            && (leader_data.is_null() || rs_ins_compl_equal(compl, leader_data, leader_size) != 0)
        {
            let mut match_limit_exceeded = false;
            let cur_source = nvim_compl_match_get_cpt_source_idx(compl);

            if cur_source != -1 {
                let idx = cur_source as usize;
                let cur = std::ptr::read(match_count.add(idx));
                std::ptr::write(match_count.add(idx), cur + 1);
                let max_matches = crate::vars::nvim_get_cpt_source_cs_max_matches(cur_source);
                if max_matches > 0 && cur + 1 > max_matches {
                    match_limit_exceeded = true;
                }
            }

            // Check cs_flag == '.' (ascii 0x2E = 46) for curbuf_only
            let cs_flag = crate::vars::nvim_get_cpt_source_cs_flag(cur_source);
            if !match_limit_exceeded && (curbuf_only == 0 || cs_flag == c_int::from(b'.')) {
                let cp_str = nvim_compl_match_get_cp_str_data(compl);
                if cp_str.is_null() {
                    // skip
                } else if first.is_null() {
                    // Check that cp_str starts with the leader
                    let leader = rs_ins_compl_leader();
                    let leader_len = rs_ins_compl_leader_len();
                    let matches = if leader.is_null() || leader_len == 0 {
                        true
                    } else {
                        let cp_size = nvim_compl_match_get_cp_str_size(compl);
                        if cp_size < leader_len {
                            false
                        } else {
                            std::slice::from_raw_parts(cp_str.cast::<u8>(), leader_len)
                                == std::slice::from_raw_parts(leader.cast::<u8>(), leader_len)
                        }
                    };
                    if matches {
                        first = cp_str;
                        // strlen: walk to NUL
                        let mut p = cp_str;
                        while *p != 0 {
                            p = p.add(1);
                        }
                        len = p.offset_from(cp_str) as c_int;
                    }
                } else {
                    // Intersect the common prefix using MB_BYTE2LEN
                    let mut j: c_int = 0;
                    let mut s1 = first;
                    let mut s2 = cp_str;

                    while j < len && *s1 != 0 && *s2 != 0 {
                        let b1 = nvim_mb_byte2len(c_int::from(*s1));
                        let b2 = nvim_mb_byte2len(c_int::from(*s2));
                        if b1 != b2
                            || std::slice::from_raw_parts(s1.cast::<u8>(), b1 as usize)
                                != std::slice::from_raw_parts(s2.cast::<u8>(), b2 as usize)
                        {
                            break;
                        }
                        j += b1;
                        s1 = s1.add(b1 as usize);
                        s2 = s2.add(b2 as usize);
                    }
                    len = j;

                    if len == 0 {
                        break 'outer;
                    }
                }
            }
        }

        compl = nvim_compl_match_get_next(compl);
        // is_first_match guard at top of loop handles the do-while termination
        if compl.is_null() {
            break;
        }
        if is_first_match(compl) {
            break;
        }
    }

    nvim_xfree(match_count.cast::<u8>());

    let leader_len = rs_ins_compl_leader_len();
    if len > leader_len as c_int {
        // Avoid inserting text that duplicates text already present after the cursor
        let cp_size = if first.is_null() {
            0
        } else {
            let mut p = first;
            while *p != 0 {
                p = p.add(1);
            }
            p.offset_from(first) as c_int
        };
        if len == cp_size {
            let line = nvim_get_cursor_line_ptr();
            let cursor_col = nvim_get_curwin_cursor_col();
            let p = line.add(cursor_col as usize);
            if !p.is_null() && rs_ascii_iswhite_or_nul(c_int::from(*p)) == 0 {
                let end = rs_find_word_end(p);
                let text_len = end.offset_from(p) as c_int;
                if text_len > 0
                    && text_len < len - leader_len as c_int
                    && std::slice::from_raw_parts(
                        first.add((len - text_len) as usize).cast::<u8>(),
                        text_len as usize,
                    ) == std::slice::from_raw_parts(p.cast::<u8>(), text_len as usize)
                {
                    len -= text_len;
                }
            }
        }

        *prefix_len = len as usize;
        return first;
    }

    std::ptr::null()
}

// =============================================================================
// Phase 15 Phase 2: rs_ins_compl_addfrommatch -- Rust port of
// nvim_ins_compl_addfrommatch_body (called from edit.c)
// =============================================================================

// rs_ins_compl_addleader is defined in crate::insert, call directly

/// Add one character from the currently shown match to the leader.
///
/// Rust port of C `nvim_ins_compl_addfrommatch_body()`. Traverses the match
/// list to find the next character after the current cursor offset, then
/// appends it via `rs_ins_compl_addleader`. Called from edit.c on CTRL-L.
///
/// # Safety
/// Must be called from insert mode with valid completion state.
/// `compl_shown_match` must not be NULL (asserted at the C level).
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_ins_compl_addfrommatch() {
    use crate::match_list::ComplMatch;

    let len = nvim_get_cursor_col() - crate::vars::nvim_get_compl_col();
    let shown = nvim_compl_get_shown_match();
    // If shown is null, return (safety: C caller checks this, but be defensive)
    if shown.is_null() {
        return;
    }

    let cp_data = nvim_compl_match_get_cp_str_data(shown);
    let cp_size = nvim_compl_match_get_cp_str_size(shown);

    // len is cursor_col - compl_col, always >= 0 at this call site
    let len_usize = len as usize;

    let p: *const c_char = if (cp_size as c_int) <= len {
        // The shown match is shorter than what was already inserted.
        // Only possible for the sentinel (original text).
        if nvim_compl_match_at_original_text(shown) == 0 {
            return;
        }
        // Walk cp_next looking for an equal match with enough length.
        let mut found: *const c_char = std::ptr::null();
        let mut found_size: usize = 0;
        let mut cp: ComplMatch = nvim_compl_match_get_next(shown);
        while !cp.is_null() && !is_first_match(cp) {
            let leader_data = crate::vars::nvim_get_compl_leader_data();
            let leader_size = crate::vars::nvim_get_compl_leader_size();
            if leader_data.is_null() || rs_ins_compl_equal(cp, leader_data, leader_size) != 0 {
                found = nvim_compl_match_get_cp_str_data(cp);
                found_size = nvim_compl_match_get_cp_str_size(cp);
                break;
            }
            cp = nvim_compl_match_get_next(cp);
        }
        if found.is_null() || (found_size as c_int) <= len {
            return;
        }
        found
    } else {
        cp_data
    };

    // Advance p by len bytes, then extract the next character.
    let c = nvim_utf_ptr2char(p.add(len_usize));
    crate::insert::rs_ins_compl_addleader(c);
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
