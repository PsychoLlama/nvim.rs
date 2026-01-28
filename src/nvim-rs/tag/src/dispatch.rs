//! Tag command dispatcher infrastructure for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for tag command dispatching
//! including command type classification, stack manipulation decisions,
//! and match navigation logic.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Command type constants (match tag.h enum values)
// =============================================================================

/// Command type values for `do_tag()`
pub mod cmd_type {
    use std::ffi::c_int;

    /// Jump to newer position or same tag again
    pub const DT_TAG: c_int = 1;
    /// Jump to older position
    pub const DT_POP: c_int = 2;
    /// Jump to next match of same tag
    pub const DT_NEXT: c_int = 3;
    /// Jump to previous match of same tag
    pub const DT_PREV: c_int = 4;
    /// Jump to first match of same tag
    pub const DT_FIRST: c_int = 5;
    /// Jump to last match of same tag
    pub const DT_LAST: c_int = 6;
    /// Jump to selection from list
    pub const DT_SELECT: c_int = 7;
    /// Like DT_TAG but no wildcards
    pub const DT_HELP: c_int = 8;
    /// Jump to new tag or selection from list
    pub const DT_JUMP: c_int = 9;
    /// Tag using location list
    pub const DT_LTAG: c_int = 11;
    /// Free cached matches
    pub const DT_FREE: c_int = 99;
}

/// MAXCOL constant for maximum match index
const MAXCOL: c_int = 0x7FFF_FFFF;

// =============================================================================
// External C accessor functions
// =============================================================================

extern "C" {
    fn nvim_get_postponed_split() -> c_int;
    fn nvim_get_g_do_tagpreview() -> c_int;
}

// =============================================================================
// Command type classification
// =============================================================================

/// Check if command type requires a new tag search.
///
/// Returns true for DT_TAG, DT_SELECT, DT_JUMP, DT_LTAG - commands that
/// need to search for a new tag pattern.
#[no_mangle]
pub extern "C" fn rs_cmd_needs_new_search(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        cmd_type::DT_TAG | cmd_type::DT_SELECT | cmd_type::DT_JUMP | cmd_type::DT_LTAG
    )
}

/// Check if command type is a navigation command (not a new search).
///
/// Navigation commands move through existing matches without searching.
#[no_mangle]
pub extern "C" fn rs_cmd_is_navigation(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        cmd_type::DT_NEXT | cmd_type::DT_PREV | cmd_type::DT_FIRST | cmd_type::DT_LAST
    )
}

/// Check if command type is DT_POP (go to older position).
#[no_mangle]
pub extern "C" fn rs_cmd_is_pop(cmd_type: c_int) -> bool {
    cmd_type == cmd_type::DT_POP
}

/// Check if command type is DT_HELP.
#[no_mangle]
pub extern "C" fn rs_cmd_is_help(cmd_type: c_int) -> bool {
    cmd_type == cmd_type::DT_HELP
}

/// Check if command type is DT_SELECT or DT_JUMP.
#[no_mangle]
pub extern "C" fn rs_cmd_is_selection(cmd_type: c_int) -> bool {
    cmd_type == cmd_type::DT_SELECT || cmd_type == cmd_type::DT_JUMP
}

/// Check if command type is DT_LTAG (location list).
#[no_mangle]
pub extern "C" fn rs_cmd_is_ltag(cmd_type: c_int) -> bool {
    cmd_type == cmd_type::DT_LTAG
}

/// Check if command type is DT_FREE (free cached matches).
#[no_mangle]
pub extern "C" fn rs_cmd_is_free(cmd_type: c_int) -> bool {
    cmd_type == cmd_type::DT_FREE
}

/// Convert DT_HELP to DT_TAG (DT_HELP is like DT_TAG but no regexp).
#[no_mangle]
pub extern "C" fn rs_normalize_cmd_type(cmd_type: c_int) -> c_int {
    if cmd_type == cmd_type::DT_HELP {
        cmd_type::DT_TAG
    } else {
        cmd_type
    }
}

// =============================================================================
// Tag stack usage decisions
// =============================================================================

/// Determine if the tag stack should be used for this command.
///
/// The tag stack is not used when:
/// - `g_do_tagpreview` is set (preview window mode)
/// - 'tagstack' option is off and tag is not empty
#[no_mangle]
pub unsafe extern "C" fn rs_should_use_tagstack(tag: *const c_char, p_tgst: bool) -> bool {
    // Check if in preview mode
    if nvim_get_g_do_tagpreview() != 0 {
        return false;
    }

    // If tagstack option is off and we have a tag, don't use stack
    if !p_tgst && !tag.is_null() && *tag != 0 {
        return false;
    }

    true
}

/// Determine if a new tag entry should be created.
///
/// A new tag entry is created for DT_TAG, DT_SELECT, DT_JUMP, DT_LTAG
/// commands when a non-empty tag is provided.
#[no_mangle]
pub unsafe extern "C" fn rs_should_create_new_tag(
    tag: *const c_char,
    cmd_type: c_int,
    _p_tgst: bool,
) -> bool {
    // Empty tag means use existing stack entry
    if tag.is_null() || *tag == 0 {
        return false;
    }

    // Check if command type creates new tag entries
    if !rs_cmd_needs_new_search(cmd_type) {
        return false;
    }

    // If tagstack is off, still create new_tag for proper behavior
    // but don't add to the actual stack
    true
}

/// Check if position should be saved before the jump.
///
/// Position is saved for ":tag" with argument and ":tag" without argument
/// (going to newer position in stack).
#[no_mangle]
pub extern "C" fn rs_should_save_position(cmd_type: c_int, _has_tag_arg: bool) -> bool {
    // ":tag [arg]" or ":ltag [arg]"
    cmd_type == cmd_type::DT_TAG || cmd_type == cmd_type::DT_LTAG
}

// =============================================================================
// Match index calculation
// =============================================================================

/// Calculate the new match index for navigation commands.
///
/// Handles DT_FIRST, DT_LAST, DT_NEXT, DT_PREV, and DT_SELECT/DT_JUMP.
#[no_mangle]
pub extern "C" fn rs_calc_new_match_index(
    cmd_type: c_int,
    cur_match: c_int,
    count: c_int,
) -> c_int {
    match cmd_type {
        cmd_type::DT_FIRST => count - 1,
        cmd_type::DT_SELECT | cmd_type::DT_JUMP | cmd_type::DT_LAST => MAXCOL - 1,
        cmd_type::DT_NEXT => cur_match + count,
        cmd_type::DT_PREV => cur_match - count,
        _ => cur_match,
    }
}

/// Clamp match index to valid range.
#[no_mangle]
pub extern "C" fn rs_clamp_match_index(cur_match: c_int, num_matches: c_int) -> c_int {
    if cur_match == MAXCOL {
        MAXCOL - 1
    } else if cur_match < 0 {
        0
    } else if cur_match >= num_matches {
        num_matches - 1
    } else {
        cur_match
    }
}

/// Check if match index is at the beginning (before first match).
#[no_mangle]
pub extern "C" fn rs_match_at_beginning(cur_match: c_int) -> bool {
    cur_match < 0
}

/// Check if match index is beyond the last match.
#[no_mangle]
pub extern "C" fn rs_match_beyond_end(cur_match: c_int, num_matches: c_int) -> bool {
    cur_match >= num_matches
}

// =============================================================================
// Tag stack index calculations
// =============================================================================

/// Calculate new stack index after DT_POP command.
///
/// Returns the new index, which may be clamped to valid range.
#[no_mangle]
pub extern "C" fn rs_calc_pop_index(tagstackidx: c_int, count: c_int) -> c_int {
    let new_idx = tagstackidx - count;
    if new_idx < 0 {
        0
    } else {
        new_idx
    }
}

/// Check if stack index is at the bottom.
#[no_mangle]
pub extern "C" fn rs_at_stack_bottom(tagstackidx: c_int, count: c_int) -> bool {
    tagstackidx - count < 0
}

/// Check if stack index is at or beyond the top.
#[no_mangle]
pub extern "C" fn rs_at_stack_top(tagstackidx: c_int, tagstacklen: c_int) -> bool {
    tagstackidx >= tagstacklen
}

/// Calculate new stack index for going to newer tag.
#[no_mangle]
pub extern "C" fn rs_calc_newer_index(
    tagstackidx: c_int,
    tagstacklen: c_int,
    count: c_int,
) -> c_int {
    let new_idx = tagstackidx + count - 1;
    if new_idx >= tagstacklen {
        tagstacklen - 1
    } else if new_idx < 0 {
        0
    } else {
        new_idx
    }
}

/// Check if going to newer would exceed stack top.
#[no_mangle]
pub extern "C" fn rs_newer_exceeds_stack(
    tagstackidx: c_int,
    tagstacklen: c_int,
    count: c_int,
) -> bool {
    tagstackidx + count > tagstacklen
}

// =============================================================================
// Command dispatch state
// =============================================================================

/// State for `do_tag` command dispatch
#[repr(C)]
pub struct DoTagState {
    /// Original command type
    pub orig_cmd_type: c_int,
    /// Current command type (may change during execution)
    pub cmd_type: c_int,
    /// Count argument
    pub count: c_int,
    /// Force flag
    pub forceit: c_int,
    /// Verbose flag
    pub verbose: bool,
    /// Whether to use the tag stack
    pub use_tagstack: bool,
    /// Whether this is a new tag search
    pub new_tag: bool,
    /// Whether to save position before jump
    pub save_pos: bool,
    /// Current match index
    pub cur_match: c_int,
    /// Current file number for match priority
    pub cur_fnum: c_int,
    /// Previous tag stack index (for cancellation)
    pub prev_tagstackidx: c_int,
    /// Skip "tag X of Y" message
    pub skip_msg: bool,
    /// No regexp matching (for DT_HELP)
    pub no_regexp: bool,
    /// Use tagfunc
    pub use_tfu: bool,
}

impl Default for DoTagState {
    fn default() -> Self {
        Self {
            orig_cmd_type: 0,
            cmd_type: 0,
            count: 1,
            forceit: 0,
            verbose: false,
            use_tagstack: true,
            new_tag: false,
            save_pos: false,
            cur_match: 0,
            cur_fnum: 0,
            prev_tagstackidx: 0,
            skip_msg: false,
            no_regexp: false,
            use_tfu: true,
        }
    }
}

/// Initialize `do_tag` state with command parameters.
#[no_mangle]
pub unsafe extern "C" fn rs_do_tag_state_init(
    state: *mut DoTagState,
    tag: *const c_char,
    cmd_type: c_int,
    count: c_int,
    forceit: c_int,
    verbose: bool,
    p_tgst: bool,
    cur_fnum: c_int,
) {
    if state.is_null() {
        return;
    }

    (*state).orig_cmd_type = cmd_type;
    (*state).cmd_type = rs_normalize_cmd_type(cmd_type);
    (*state).count = count;
    (*state).forceit = forceit;
    (*state).verbose = verbose;
    (*state).cur_fnum = cur_fnum;
    (*state).prev_tagstackidx = 0;
    (*state).skip_msg = false;

    // DT_HELP specific settings
    (*state).no_regexp = cmd_type == cmd_type::DT_HELP;
    (*state).use_tfu = cmd_type != cmd_type::DT_HELP;

    // Determine stack usage
    (*state).use_tagstack = rs_should_use_tagstack(tag, p_tgst);
    (*state).new_tag = rs_should_create_new_tag(tag, cmd_type, p_tgst);

    // Initial save_pos setting
    (*state).save_pos = false;
    (*state).cur_match = 0;
}

/// Allocate a new `do_tag` state.
#[no_mangle]
pub extern "C" fn rs_do_tag_state_new() -> *mut DoTagState {
    Box::into_raw(Box::new(DoTagState::default()))
}

/// Free a `do_tag` state.
#[no_mangle]
pub unsafe extern "C" fn rs_do_tag_state_free(state: *mut DoTagState) {
    if !state.is_null() {
        drop(Box::from_raw(state));
    }
}

/// Get the current command type from state.
#[no_mangle]
pub unsafe extern "C" fn rs_do_tag_get_cmd_type(state: *const DoTagState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).cmd_type
}

/// Set the current command type in state.
#[no_mangle]
pub unsafe extern "C" fn rs_do_tag_set_cmd_type(state: *mut DoTagState, cmd_type: c_int) {
    if state.is_null() {
        return;
    }
    (*state).cmd_type = cmd_type;
}

/// Check if command should use regexp.
#[no_mangle]
pub unsafe extern "C" fn rs_do_tag_use_regexp(
    state: *const DoTagState,
    tag: *const c_char,
) -> bool {
    if state.is_null() || tag.is_null() {
        return false;
    }

    // Don't use regexp if no_regexp is set (DT_HELP)
    if (*state).no_regexp {
        return false;
    }

    // Use regexp if tag starts with '/'
    *tag == b'/' as c_char
}

// =============================================================================
// Flags calculation for find_tags
// =============================================================================

/// Calculate the flags for `find_tags()` based on command state.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_find_tags_flags(
    state: *const DoTagState,
    tag: *const c_char,
) -> c_int {
    use crate::search::find_tags_flags;

    if state.is_null() {
        return 0;
    }

    let mut flags = 0;

    // Check if tag starts with '/' (regexp)
    if rs_do_tag_use_regexp(state, tag) {
        flags |= find_tags_flags::TAG_REGEXP;
    } else {
        flags |= find_tags_flags::TAG_NOIC;
    }

    if (*state).verbose {
        flags |= find_tags_flags::TAG_VERBOSE;
    }

    if !(*state).use_tfu {
        flags |= find_tags_flags::TAG_NO_TAGFUNC;
    }

    flags
}

/// Calculate `max_num_matches` for `find_tags()`.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_max_matches(state: *const DoTagState) -> c_int {
    if state.is_null() {
        return MAXCOL;
    }

    if (*state).cmd_type == cmd_type::DT_TAG {
        MAXCOL
    } else {
        (*state).cur_match + 1
    }
}

// =============================================================================
// Error condition checks
// =============================================================================

/// Check if we can proceed with tag command (not in tagfunc).
#[no_mangle]
pub extern "C" fn rs_can_proceed_with_tag(tfu_in_use: bool) -> bool {
    !tfu_in_use
}

/// Check if postponed split allows the command.
#[no_mangle]
pub unsafe extern "C" fn rs_postponed_split_allows(_forceit: c_int, can_set_curbuf: bool) -> bool {
    nvim_get_postponed_split() != 0 || can_set_curbuf
}

// =============================================================================
// File not found retry logic
// =============================================================================

/// Check if we should retry with another match after file not found.
#[no_mangle]
pub extern "C" fn rs_should_retry_match(
    cmd_type: c_int,
    cur_match: c_int,
    num_matches: c_int,
    max_num_matches: c_int,
) -> bool {
    if cmd_type == cmd_type::DT_PREV && cur_match > 0 {
        return true;
    }

    if matches!(
        cmd_type,
        cmd_type::DT_TAG | cmd_type::DT_NEXT | cmd_type::DT_FIRST
    ) && (max_num_matches != MAXCOL || cur_match < num_matches - 1)
    {
        return true;
    }

    false
}

/// Calculate the next match index after file not found.
#[no_mangle]
pub extern "C" fn rs_calc_retry_match(cmd_type: c_int, cur_match: c_int) -> c_int {
    if cmd_type == cmd_type::DT_PREV {
        cur_match - 1
    } else {
        cur_match + 1
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_classification() {
        // New search commands
        assert!(rs_cmd_needs_new_search(cmd_type::DT_TAG));
        assert!(rs_cmd_needs_new_search(cmd_type::DT_SELECT));
        assert!(rs_cmd_needs_new_search(cmd_type::DT_JUMP));
        assert!(rs_cmd_needs_new_search(cmd_type::DT_LTAG));
        assert!(!rs_cmd_needs_new_search(cmd_type::DT_POP));
        assert!(!rs_cmd_needs_new_search(cmd_type::DT_NEXT));

        // Navigation commands
        assert!(rs_cmd_is_navigation(cmd_type::DT_NEXT));
        assert!(rs_cmd_is_navigation(cmd_type::DT_PREV));
        assert!(rs_cmd_is_navigation(cmd_type::DT_FIRST));
        assert!(rs_cmd_is_navigation(cmd_type::DT_LAST));
        assert!(!rs_cmd_is_navigation(cmd_type::DT_TAG));

        // Specific checks
        assert!(rs_cmd_is_pop(cmd_type::DT_POP));
        assert!(!rs_cmd_is_pop(cmd_type::DT_TAG));

        assert!(rs_cmd_is_help(cmd_type::DT_HELP));
        assert!(!rs_cmd_is_help(cmd_type::DT_TAG));

        assert!(rs_cmd_is_selection(cmd_type::DT_SELECT));
        assert!(rs_cmd_is_selection(cmd_type::DT_JUMP));
        assert!(!rs_cmd_is_selection(cmd_type::DT_TAG));

        assert!(rs_cmd_is_ltag(cmd_type::DT_LTAG));
        assert!(!rs_cmd_is_ltag(cmd_type::DT_TAG));
    }

    #[test]
    fn test_normalize_cmd_type() {
        assert_eq!(rs_normalize_cmd_type(cmd_type::DT_HELP), cmd_type::DT_TAG);
        assert_eq!(rs_normalize_cmd_type(cmd_type::DT_TAG), cmd_type::DT_TAG);
        assert_eq!(rs_normalize_cmd_type(cmd_type::DT_POP), cmd_type::DT_POP);
    }

    #[test]
    fn test_calc_new_match_index() {
        // count - 1
        assert_eq!(rs_calc_new_match_index(cmd_type::DT_FIRST, 5, 3), 2);
        // MAXCOL - 1
        assert_eq!(
            rs_calc_new_match_index(cmd_type::DT_LAST, 5, 1),
            0x7FFF_FFFE
        );
        // cur + count
        assert_eq!(rs_calc_new_match_index(cmd_type::DT_NEXT, 5, 3), 8);
        // cur - count
        assert_eq!(rs_calc_new_match_index(cmd_type::DT_PREV, 5, 3), 2);
        // unchanged
        assert_eq!(rs_calc_new_match_index(cmd_type::DT_TAG, 5, 3), 5);
    }

    #[test]
    fn test_clamp_match_index() {
        assert_eq!(rs_clamp_match_index(5, 10), 5);
        assert_eq!(rs_clamp_match_index(-1, 10), 0);
        assert_eq!(rs_clamp_match_index(15, 10), 9);
        assert_eq!(rs_clamp_match_index(0x7FFF_FFFF, 10), 0x7FFF_FFFE);
    }

    #[test]
    fn test_stack_index_calculations() {
        assert_eq!(rs_calc_pop_index(5, 3), 2);
        assert_eq!(rs_calc_pop_index(2, 5), 0);

        assert!(rs_at_stack_bottom(2, 5));
        assert!(!rs_at_stack_bottom(5, 2));

        assert!(rs_at_stack_top(5, 5));
        assert!(rs_at_stack_top(6, 5));
        assert!(!rs_at_stack_top(4, 5));

        // 2 + 2 - 1 = 3
        assert_eq!(rs_calc_newer_index(2, 5, 2), 3);
        // clamped to len - 1
        assert_eq!(rs_calc_newer_index(4, 5, 3), 4);
    }

    #[test]
    fn test_should_retry_match() {
        // DT_PREV with matches remaining
        assert!(rs_should_retry_match(cmd_type::DT_PREV, 5, 10, 0x7FFF_FFFF));
        assert!(!rs_should_retry_match(
            cmd_type::DT_PREV,
            0,
            10,
            0x7FFF_FFFF
        ));

        // DT_NEXT with matches remaining
        assert!(rs_should_retry_match(cmd_type::DT_NEXT, 5, 10, 0x7FFF_FFFF));
        assert!(!rs_should_retry_match(
            cmd_type::DT_NEXT,
            9,
            10,
            0x7FFF_FFFF
        ));

        // DT_TAG when max not reached
        assert!(rs_should_retry_match(cmd_type::DT_TAG, 5, 10, 20));
    }

    #[test]
    fn test_calc_retry_match() {
        assert_eq!(rs_calc_retry_match(cmd_type::DT_PREV, 5), 4);
        assert_eq!(rs_calc_retry_match(cmd_type::DT_NEXT, 5), 6);
        assert_eq!(rs_calc_retry_match(cmd_type::DT_TAG, 5), 6);
    }

    #[test]
    fn test_do_tag_state_default() {
        let state = DoTagState::default();
        assert_eq!(state.cmd_type, 0);
        assert_eq!(state.count, 1);
        assert!(state.use_tagstack);
        assert!(!state.new_tag);
        assert!(state.use_tfu);
    }
}
