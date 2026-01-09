//! Tag command helpers for Neovim C-to-Rust migration
//!
//! This module provides Rust helper functions for tag command processing,
//! including validation, argument parsing, and command dispatch helpers.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int, c_void};

use crate::tag_cmd;
use crate::TAGSTACKSIZE;

// =============================================================================
// Opaque handle types
// =============================================================================

/// Opaque handle to win_T (window)
type WinHandle = *const c_void;

/// Line number type
type LinenrT = i32;

/// Column number type
type ColnrT = c_int;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Window tag stack accessors
    fn nvim_win_get_tagstacklen(wp: WinHandle) -> c_int;
    fn nvim_win_get_tagstackidx(wp: WinHandle) -> c_int;

    // String functions
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// =============================================================================
// Command type helpers
// =============================================================================

/// Tag command result/action codes
pub mod cmd_result {
    use std::ffi::c_int;

    /// Command succeeded
    pub const OK: c_int = 0;
    /// Command failed
    pub const FAIL: c_int = -1;
    /// Stack is empty
    pub const STACK_EMPTY: c_int = -2;
    /// At bottom of stack
    pub const AT_BOTTOM: c_int = -3;
    /// At top of stack
    pub const AT_TOP: c_int = -4;
    /// No matches found
    pub const NO_MATCH: c_int = -5;
    /// Buffer check failed
    pub const BUF_CHECK_FAIL: c_int = -6;
    /// Inside tagfunc (cannot modify)
    pub const IN_TAGFUNC: c_int = -7;
}

/// Check if a command type needs a new search.
///
/// Returns true for commands that initiate new tag searches:
/// DT_TAG, DT_SELECT, DT_JUMP, DT_LTAG
#[no_mangle]
pub extern "C" fn rs_tag_cmd_needs_search(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        tag_cmd::DT_TAG | tag_cmd::DT_SELECT | tag_cmd::DT_JUMP | tag_cmd::DT_LTAG
    )
}

/// Check if a command type uses existing matches.
///
/// Returns true for commands that navigate existing matches:
/// DT_NEXT, DT_PREV, DT_FIRST, DT_LAST
#[no_mangle]
pub extern "C" fn rs_tag_cmd_uses_matches(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        tag_cmd::DT_NEXT | tag_cmd::DT_PREV | tag_cmd::DT_FIRST | tag_cmd::DT_LAST
    )
}

/// Check if a command type shows selection menu.
///
/// Returns true for commands that may show tag selection:
/// DT_SELECT, DT_JUMP
#[no_mangle]
pub extern "C" fn rs_tag_cmd_shows_select(cmd_type: c_int) -> bool {
    matches!(cmd_type, tag_cmd::DT_SELECT | tag_cmd::DT_JUMP)
}

/// Check if a command type is for help tags.
#[no_mangle]
pub extern "C" fn rs_tag_cmd_is_help(cmd_type: c_int) -> bool {
    cmd_type == tag_cmd::DT_HELP
}

/// Check if a command type is for location list.
#[no_mangle]
pub extern "C" fn rs_tag_cmd_is_ltag(cmd_type: c_int) -> bool {
    cmd_type == tag_cmd::DT_LTAG
}

/// Check if a command type should push to tag stack.
///
/// New searches (TAG, SELECT, JUMP, LTAG) push to the tag stack.
#[no_mangle]
pub extern "C" fn rs_tag_cmd_pushes_stack(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        tag_cmd::DT_TAG | tag_cmd::DT_SELECT | tag_cmd::DT_JUMP | tag_cmd::DT_LTAG
    )
}

// =============================================================================
// Argument validation helpers
// =============================================================================

/// Validate the tag argument string.
///
/// Returns true if the tag is valid (non-null and non-empty).
#[no_mangle]
pub unsafe extern "C" fn rs_tag_arg_valid(tag: *const c_char) -> bool {
    if tag.is_null() {
        return false;
    }
    *tag != 0
}

/// Get the length of a tag argument string.
///
/// Returns 0 if tag is null.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_arg_len(tag: *const c_char) -> usize {
    if tag.is_null() {
        return 0;
    }
    strlen(tag)
}

/// Check if a tag string matches another (case-sensitive).
///
/// Returns true if the tags are equal.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_arg_matches(tag1: *const c_char, tag2: *const c_char) -> bool {
    if tag1.is_null() || tag2.is_null() {
        return tag1.is_null() && tag2.is_null();
    }
    strcmp(tag1, tag2) == 0
}

// =============================================================================
// Stack navigation validation
// =============================================================================

/// Validate a pop operation on the tag stack.
///
/// Returns cmd_result::OK if pop is valid, or an error code.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_validate_pop(wp: WinHandle, count: c_int) -> c_int {
    if wp.is_null() {
        return cmd_result::FAIL;
    }

    let idx = nvim_win_get_tagstackidx(wp);
    let len = nvim_win_get_tagstacklen(wp);

    if len == 0 {
        return cmd_result::STACK_EMPTY;
    }

    if idx == 0 && count > 0 {
        return cmd_result::AT_BOTTOM;
    }

    cmd_result::OK
}

/// Calculate the new stack index after a pop operation.
///
/// Returns the new index, clamped to valid range [0, len-1].
#[no_mangle]
pub unsafe extern "C" fn rs_tag_calc_pop_idx(wp: WinHandle, count: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let idx = nvim_win_get_tagstackidx(wp);
    (idx - count).max(0)
}

/// Validate a tag navigation (to newer tags).
///
/// Returns cmd_result::OK if navigation is valid, or an error code.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_validate_newer(wp: WinHandle, count: c_int) -> c_int {
    if wp.is_null() {
        return cmd_result::FAIL;
    }

    let idx = nvim_win_get_tagstackidx(wp);
    let len = nvim_win_get_tagstacklen(wp);

    if len == 0 {
        return cmd_result::STACK_EMPTY;
    }

    if idx >= len && count > 0 {
        return cmd_result::AT_TOP;
    }

    cmd_result::OK
}

/// Calculate the new stack index after navigating to newer tag.
///
/// Returns the new index, clamped to valid range.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_calc_newer_idx(wp: WinHandle, count: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let idx = nvim_win_get_tagstackidx(wp);
    let len = nvim_win_get_tagstacklen(wp);
    (idx + count - 1).min(len - 1).max(0)
}

// =============================================================================
// Match navigation helpers
// =============================================================================

/// Calculate the next match index for navigation commands.
///
/// Returns the target match index based on the command type and current state.
///
/// # Arguments
///
/// * `cmd_type` - The tag command type (DT_NEXT, DT_PREV, DT_FIRST, DT_LAST)
/// * `cur_match` - The current match index
/// * `num_matches` - Total number of matches
/// * `count` - The count argument from the command
#[no_mangle]
pub extern "C" fn rs_tag_calc_match_idx(
    cmd_type: c_int,
    cur_match: c_int,
    num_matches: c_int,
    count: c_int,
) -> c_int {
    if num_matches <= 0 {
        return 0;
    }

    match cmd_type {
        tag_cmd::DT_FIRST => 0,
        tag_cmd::DT_LAST => num_matches - 1,
        tag_cmd::DT_NEXT => {
            let next = cur_match + count;
            if next >= num_matches {
                // Wrap around or stay at last
                num_matches - 1
            } else {
                next
            }
        }
        tag_cmd::DT_PREV => {
            let prev = cur_match - count;
            if prev < 0 {
                // Wrap around or stay at first
                0
            } else {
                prev
            }
        }
        _ => cur_match,
    }
}

/// Check if a match navigation would wrap around.
///
/// Returns true if navigating would wrap past the beginning or end.
#[no_mangle]
pub extern "C" fn rs_tag_match_would_wrap(
    cmd_type: c_int,
    cur_match: c_int,
    num_matches: c_int,
    count: c_int,
) -> bool {
    if num_matches <= 0 {
        return false;
    }

    match cmd_type {
        tag_cmd::DT_NEXT => cur_match + count >= num_matches,
        tag_cmd::DT_PREV => cur_match - count < 0,
        _ => false,
    }
}

/// Check if we're at the first match.
#[no_mangle]
pub extern "C" fn rs_tag_at_first_match(cur_match: c_int) -> bool {
    cur_match <= 0
}

/// Check if we're at the last match.
#[no_mangle]
pub extern "C" fn rs_tag_at_last_match(cur_match: c_int, num_matches: c_int) -> bool {
    num_matches <= 0 || cur_match >= num_matches - 1
}

// =============================================================================
// Stack size calculations
// =============================================================================

/// Calculate the new stack length after pushing a tag.
///
/// If the stack is full, returns TAGSTACKSIZE (oldest will be shifted out).
#[no_mangle]
pub extern "C" fn rs_tag_calc_push_len(current_len: c_int) -> c_int {
    if current_len >= TAGSTACKSIZE {
        TAGSTACKSIZE
    } else {
        current_len + 1
    }
}

/// Check if the stack needs to be shifted before push.
///
/// Returns true if the stack is at capacity.
#[no_mangle]
pub extern "C" fn rs_tag_needs_shift(current_len: c_int) -> bool {
    current_len >= TAGSTACKSIZE
}

/// Calculate the insert position for a new stack entry.
///
/// Accounts for truncation at current index.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_calc_push_idx(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let idx = nvim_win_get_tagstackidx(wp);
    let len = nvim_win_get_tagstacklen(wp);

    // New entry goes at current index position
    // (entries above will be truncated)
    idx.min(len).min(TAGSTACKSIZE - 1)
}

// =============================================================================
// Preview window helpers
// =============================================================================

/// Preview state tracking
#[repr(C)]
#[derive(Default)]
pub struct PreviewTagState {
    /// Whether preview is active
    pub active: bool,
    /// Current match index in preview
    pub cur_match: c_int,
    /// File number for preview match
    pub cur_fnum: c_int,
}

/// Create a new preview tag state.
#[no_mangle]
pub extern "C" fn rs_preview_state_new() -> *mut PreviewTagState {
    Box::into_raw(Box::new(PreviewTagState::default()))
}

/// Free a preview tag state.
#[no_mangle]
pub unsafe extern "C" fn rs_preview_state_free(state: *mut PreviewTagState) {
    if !state.is_null() {
        drop(Box::from_raw(state));
    }
}

/// Reset preview state.
#[no_mangle]
pub unsafe extern "C" fn rs_preview_state_reset(state: *mut PreviewTagState) {
    if state.is_null() {
        return;
    }
    *state = PreviewTagState::default();
}

/// Set preview state as active with match info.
#[no_mangle]
pub unsafe extern "C" fn rs_preview_state_set(
    state: *mut PreviewTagState,
    cur_match: c_int,
    cur_fnum: c_int,
) {
    if state.is_null() {
        return;
    }
    (*state).active = true;
    (*state).cur_match = cur_match;
    (*state).cur_fnum = cur_fnum;
}

/// Check if preview state is active.
#[no_mangle]
pub unsafe extern "C" fn rs_preview_state_active(state: *const PreviewTagState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).active
}

/// Get current match from preview state.
#[no_mangle]
pub unsafe extern "C" fn rs_preview_state_match(state: *const PreviewTagState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).cur_match
}

/// Get file number from preview state.
#[no_mangle]
pub unsafe extern "C" fn rs_preview_state_fnum(state: *const PreviewTagState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).cur_fnum
}

// =============================================================================
// Jump target calculation
// =============================================================================

/// Represents a jump target location
#[repr(C)]
#[derive(Default)]
pub struct JumpTarget {
    /// Line number
    pub lnum: LinenrT,
    /// Column number
    pub col: ColnrT,
    /// File/buffer number
    pub fnum: c_int,
    /// Whether the target is valid
    pub valid: bool,
}

/// Create a new jump target.
#[no_mangle]
pub extern "C" fn rs_jump_target_new() -> *mut JumpTarget {
    Box::into_raw(Box::new(JumpTarget::default()))
}

/// Free a jump target.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_target_free(target: *mut JumpTarget) {
    if !target.is_null() {
        drop(Box::from_raw(target));
    }
}

/// Set jump target values.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_target_set(
    target: *mut JumpTarget,
    lnum: LinenrT,
    col: ColnrT,
    fnum: c_int,
) {
    if target.is_null() {
        return;
    }
    (*target).lnum = lnum;
    (*target).col = col;
    (*target).fnum = fnum;
    (*target).valid = lnum > 0;
}

/// Check if jump target is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_target_valid(target: *const JumpTarget) -> bool {
    if target.is_null() {
        return false;
    }
    (*target).valid && (*target).lnum > 0
}

/// Check if jump target requires file change.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_target_needs_file_change(
    target: *const JumpTarget,
    cur_fnum: c_int,
) -> bool {
    if target.is_null() {
        return false;
    }
    (*target).valid && (*target).fnum != cur_fnum
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_needs_search() {
        assert!(rs_tag_cmd_needs_search(tag_cmd::DT_TAG));
        assert!(rs_tag_cmd_needs_search(tag_cmd::DT_SELECT));
        assert!(rs_tag_cmd_needs_search(tag_cmd::DT_JUMP));
        assert!(rs_tag_cmd_needs_search(tag_cmd::DT_LTAG));
        assert!(!rs_tag_cmd_needs_search(tag_cmd::DT_NEXT));
        assert!(!rs_tag_cmd_needs_search(tag_cmd::DT_POP));
    }

    #[test]
    fn test_cmd_uses_matches() {
        assert!(rs_tag_cmd_uses_matches(tag_cmd::DT_NEXT));
        assert!(rs_tag_cmd_uses_matches(tag_cmd::DT_PREV));
        assert!(rs_tag_cmd_uses_matches(tag_cmd::DT_FIRST));
        assert!(rs_tag_cmd_uses_matches(tag_cmd::DT_LAST));
        assert!(!rs_tag_cmd_uses_matches(tag_cmd::DT_TAG));
        assert!(!rs_tag_cmd_uses_matches(tag_cmd::DT_POP));
    }

    #[test]
    fn test_calc_match_idx() {
        // FIRST always returns 0
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_FIRST, 5, 10, 1), 0);

        // LAST always returns num_matches - 1
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_LAST, 0, 10, 1), 9);

        // NEXT increments
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_NEXT, 5, 10, 1), 6);
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_NEXT, 5, 10, 2), 7);

        // NEXT at end stays at end
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_NEXT, 9, 10, 1), 9);

        // PREV decrements
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_PREV, 5, 10, 1), 4);
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_PREV, 5, 10, 2), 3);

        // PREV at start stays at start
        assert_eq!(rs_tag_calc_match_idx(tag_cmd::DT_PREV, 0, 10, 1), 0);
    }

    #[test]
    fn test_match_would_wrap() {
        assert!(rs_tag_match_would_wrap(tag_cmd::DT_NEXT, 9, 10, 1));
        assert!(!rs_tag_match_would_wrap(tag_cmd::DT_NEXT, 8, 10, 1));
        assert!(rs_tag_match_would_wrap(tag_cmd::DT_PREV, 0, 10, 1));
        assert!(!rs_tag_match_would_wrap(tag_cmd::DT_PREV, 1, 10, 1));
    }

    #[test]
    fn test_at_first_last() {
        assert!(rs_tag_at_first_match(0));
        assert!(!rs_tag_at_first_match(1));
        assert!(rs_tag_at_last_match(9, 10));
        assert!(!rs_tag_at_last_match(8, 10));
    }

    #[test]
    fn test_calc_push_len() {
        assert_eq!(rs_tag_calc_push_len(0), 1);
        assert_eq!(rs_tag_calc_push_len(10), 11);
        assert_eq!(rs_tag_calc_push_len(TAGSTACKSIZE), TAGSTACKSIZE);
        assert_eq!(rs_tag_calc_push_len(TAGSTACKSIZE - 1), TAGSTACKSIZE);
    }

    #[test]
    fn test_needs_shift() {
        assert!(!rs_tag_needs_shift(0));
        assert!(!rs_tag_needs_shift(TAGSTACKSIZE - 1));
        assert!(rs_tag_needs_shift(TAGSTACKSIZE));
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            assert!(!rs_tag_arg_valid(std::ptr::null()));
            assert_eq!(rs_tag_arg_len(std::ptr::null()), 0);
            assert!(rs_tag_arg_matches(std::ptr::null(), std::ptr::null()));
            assert_eq!(rs_tag_validate_pop(std::ptr::null(), 1), cmd_result::FAIL);
            assert_eq!(rs_tag_calc_pop_idx(std::ptr::null(), 1), 0);
            assert_eq!(rs_tag_validate_newer(std::ptr::null(), 1), cmd_result::FAIL);
            assert_eq!(rs_tag_calc_newer_idx(std::ptr::null(), 1), 0);
            assert_eq!(rs_tag_calc_push_idx(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_preview_state() {
        unsafe {
            let state = rs_preview_state_new();
            assert!(!state.is_null());

            assert!(!rs_preview_state_active(state));
            assert_eq!(rs_preview_state_match(state), 0);
            assert_eq!(rs_preview_state_fnum(state), 0);

            rs_preview_state_set(state, 5, 10);
            assert!(rs_preview_state_active(state));
            assert_eq!(rs_preview_state_match(state), 5);
            assert_eq!(rs_preview_state_fnum(state), 10);

            rs_preview_state_reset(state);
            assert!(!rs_preview_state_active(state));

            rs_preview_state_free(state);
        }
    }

    #[test]
    fn test_jump_target() {
        unsafe {
            let target = rs_jump_target_new();
            assert!(!target.is_null());

            assert!(!rs_jump_target_valid(target));

            rs_jump_target_set(target, 100, 5, 1);
            assert!(rs_jump_target_valid(target));
            assert!(rs_jump_target_needs_file_change(target, 2));
            assert!(!rs_jump_target_needs_file_change(target, 1));

            rs_jump_target_free(target);
        }
    }
}
