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
// Location List Integration
// =============================================================================

/// Maximum tag name length for location list
pub const TAG_NAME_MAX_LEN: usize = 128;
/// Command buffer size for location list patterns
pub const CMDBUFFSIZE: usize = 256;

/// Tag list display entry information
#[derive(Default)]
#[repr(C)]
pub struct TagListEntry {
    /// Index in match list (0-based)
    pub index: c_int,
    /// Match type byte
    pub match_type: u8,
    /// Whether this entry is the current match
    pub is_current: bool,
}

/// Initialize a tag list entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_list_entry_init(
    entry: *mut TagListEntry,
    index: c_int,
    match_type: u8,
    is_current: bool,
) {
    if entry.is_null() {
        return;
    }
    (*entry).index = index;
    (*entry).match_type = match_type;
    (*entry).is_current = is_current;
}

/// Check if entry is the current match.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_list_entry_is_current(entry: *const TagListEntry) -> bool {
    if entry.is_null() {
        return false;
    }
    (*entry).is_current
}

/// Get formatted index string for tag list display.
///
/// Returns the 1-based index as would be displayed.
#[no_mangle]
pub extern "C" fn rs_tag_list_display_index(index: c_int) -> c_int {
    index + 1
}

/// Location list entry for tag match
#[repr(C)]
pub struct TagLocationEntry {
    /// Tag name (null-terminated, max TAG_NAME_MAX_LEN)
    pub tag_name: [c_char; TAG_NAME_MAX_LEN + 1],
    /// Tag name length
    pub tag_name_len: c_int,
    /// Line number (0 if pattern-based)
    pub lnum: LinenrT,
    /// Search pattern (if lnum is 0)
    pub pattern: [c_char; CMDBUFFSIZE + 1],
    /// Pattern length
    pub pattern_len: c_int,
    /// Whether entry is valid
    pub valid: bool,
}

impl Default for TagLocationEntry {
    fn default() -> Self {
        Self {
            tag_name: [0; TAG_NAME_MAX_LEN + 1],
            tag_name_len: 0,
            lnum: 0,
            pattern: [0; CMDBUFFSIZE + 1],
            pattern_len: 0,
            valid: false,
        }
    }
}

/// Allocate a new location entry.
#[no_mangle]
pub extern "C" fn rs_tag_location_entry_new() -> *mut TagLocationEntry {
    Box::into_raw(Box::new(TagLocationEntry::default()))
}

/// Free a location entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_location_entry_free(entry: *mut TagLocationEntry) {
    if !entry.is_null() {
        drop(Box::from_raw(entry));
    }
}

/// Initialize location entry with tag name.
///
/// Copies up to TAG_NAME_MAX_LEN characters of the tag name.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_location_set_name(
    entry: *mut TagLocationEntry,
    name: *const c_char,
    len: c_int,
) {
    if entry.is_null() || name.is_null() {
        return;
    }

    let copy_len = std::cmp::min(len as usize, TAG_NAME_MAX_LEN);

    for i in 0..copy_len {
        (*entry).tag_name[i] = *name.add(i);
    }
    (*entry).tag_name[copy_len] = 0;
    (*entry).tag_name_len = copy_len as c_int;
    (*entry).valid = true;
}

/// Set line number for location entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_location_set_lnum(entry: *mut TagLocationEntry, lnum: LinenrT) {
    if entry.is_null() {
        return;
    }
    (*entry).lnum = lnum;
}

/// Check if command string is a line number.
///
/// Returns true if the first character is a digit.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_cmd_is_linenr(cmd: *const c_char) -> bool {
    if cmd.is_null() {
        return false;
    }
    let c = *cmd as u8;
    c.is_ascii_digit()
}

/// Parse a line number from command string.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_cmd_parse_linenr(cmd: *const c_char) -> LinenrT {
    if cmd.is_null() {
        return 0;
    }

    let mut result: LinenrT = 0;
    let mut ptr = cmd;

    while !ptr.is_null() {
        let c = *ptr as u8;
        if !c.is_ascii_digit() {
            break;
        }
        result = result * 10 + (c - b'0') as LinenrT;
        ptr = ptr.add(1);
    }

    result
}

/// Skip leading '/' or '?' in search pattern.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_skip_delim(pat: *const c_char) -> *const c_char {
    if pat.is_null() {
        return pat;
    }

    let c = *pat as u8;
    if c == b'/' || c == b'?' {
        pat.add(1)
    } else {
        pat
    }
}

/// Check if character is a search delimiter ('/' or '?').
#[no_mangle]
pub extern "C" fn rs_tag_is_search_delim(c: c_char) -> bool {
    let c = c as u8;
    c == b'/' || c == b'?'
}

/// Check if pattern starts with '^' anchor.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_has_caret(pat: *const c_char) -> bool {
    if pat.is_null() {
        return false;
    }
    *pat as u8 == b'^'
}

/// Check if pattern ends with '$' anchor.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_has_dollar(pat: *const c_char, len: c_int) -> bool {
    if pat.is_null() || len <= 0 {
        return false;
    }
    *pat.add((len - 1) as usize) as u8 == b'$'
}

/// Format a search pattern for location list.
///
/// Prepends "^" if needed, adds "\V" for very nomagic, and handles "$" escaping.
/// Returns the length of the formatted pattern.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_format_llist_pattern(
    entry: *mut TagLocationEntry,
    cmd_start: *const c_char,
    cmd_end: *const c_char,
) -> c_int {
    if entry.is_null() || cmd_start.is_null() || cmd_end.is_null() {
        return 0;
    }

    // Skip delimiters
    let mut start = cmd_start;
    let mut end = cmd_end;

    if rs_tag_is_search_delim(*start) {
        start = start.add(1);
    }
    if end > start && rs_tag_is_search_delim(*end) {
        end = end.sub(1);
    }

    let mut pos = 0usize;

    // Handle "^" at start
    if start < end && *start as u8 == b'^' {
        (*entry).pattern[pos] = b'^' as c_char;
        pos += 1;
        start = start.add(1);
    }

    // Add "\V" for very nomagic
    if pos + 2 < CMDBUFFSIZE {
        (*entry).pattern[pos] = b'\\' as c_char;
        (*entry).pattern[pos + 1] = b'V' as c_char;
        pos += 2;
    }

    // Copy the pattern content
    let content_len = end.offset_from(start) as usize + 1;
    let max_copy = std::cmp::min(content_len, CMDBUFFSIZE - pos - 2);

    for i in 0..max_copy {
        (*entry).pattern[pos + i] = *start.add(i);
    }
    pos += max_copy;

    // Handle "$" at end - escape it
    if pos > 0 && (*entry).pattern[pos - 1] as u8 == b'$' && pos < CMDBUFFSIZE {
        (*entry).pattern[pos - 1] = b'\\' as c_char;
        (*entry).pattern[pos] = b'$' as c_char;
        pos += 1;
    }

    (*entry).pattern[pos] = 0;
    (*entry).pattern_len = pos as c_int;
    pos as c_int
}

/// Get pattern pointer from location entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_location_get_pattern(
    entry: *const TagLocationEntry,
) -> *const c_char {
    if entry.is_null() {
        return std::ptr::null();
    }
    (*entry).pattern.as_ptr()
}

/// Get tag name pointer from location entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_location_get_name(entry: *const TagLocationEntry) -> *const c_char {
    if entry.is_null() {
        return std::ptr::null();
    }
    (*entry).tag_name.as_ptr()
}

/// Get line number from location entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_location_get_lnum(entry: *const TagLocationEntry) -> LinenrT {
    if entry.is_null() {
        return 0;
    }
    (*entry).lnum
}

/// Check if location entry uses line number (vs pattern).
#[no_mangle]
pub unsafe extern "C" fn rs_tag_location_uses_lnum(entry: *const TagLocationEntry) -> bool {
    if entry.is_null() {
        return false;
    }
    (*entry).lnum > 0
}

// =============================================================================
// Tag List Display Helpers
// =============================================================================

/// Match type mask for extracting type from match byte
pub const MT_MASK: u8 = 0x0F;

/// Calculate the tag column width for display alignment.
///
/// Returns the width to use for tag name column, or MAXCOL if wrapping needed.
#[no_mangle]
pub extern "C" fn rs_tag_calc_display_width(taglen: c_int, columns: c_int) -> c_int {
    // Minimum width is 18
    let min_width = 18;
    let width = if taglen + 2 > min_width {
        taglen + 2
    } else {
        min_width
    };

    // If too wide for screen, use MAXCOL to indicate line wrap
    if width > columns - 25 {
        0x7FFF_FFFF // MAXCOL
    } else {
        width
    }
}

/// Check if display should wrap to next line.
#[no_mangle]
pub extern "C" fn rs_tag_display_should_wrap(taglen: c_int) -> bool {
    taglen == 0x7FFF_FFFF // MAXCOL
}

/// Get the display column for file name after tag.
#[no_mangle]
pub extern "C" fn rs_tag_display_file_column(taglen: c_int) -> c_int {
    if taglen == 0x7FFF_FFFF {
        24 // After wrap
    } else {
        13 + taglen
    }
}

/// Check if an entry is the current match in a tag list.
#[no_mangle]
pub extern "C" fn rs_tag_is_current_match(
    index: c_int,
    cur_match: c_int,
    new_tag: bool,
    use_tagstack: bool,
    preview_match: c_int,
    preview_active: bool,
) -> bool {
    if new_tag {
        return false;
    }

    if preview_active {
        index == preview_match
    } else if use_tagstack {
        index == cur_match
    } else {
        false
    }
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
