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

use std::ffi::{c_char, c_int, c_void, CStr};

use nvim_memory::xrealloc;

use crate::tag_cmd;
use crate::TAGSTACKSIZE;

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

// Error message string constants
const E_CANNOT_MODIFY_TAG_STACK_WITHIN_TAGFUNC: &CStr =
    c"E986: Cannot modify the tag stack within tagfunc";
const E_LISTREQ: &CStr = c"E714: List required";

// =============================================================================
// Opaque handle types
// =============================================================================

/// Opaque handle to win_T (window)
type WinHandle = *const c_void;

/// Access `WinStruct` fields from a const `win_T` pointer.
#[allow(clippy::missing_const_for_fn)]
#[inline]
unsafe fn win_ref_raw<'a>(wp: *const std::ffi::c_void) -> &'a nvim_window::win_struct::WinStruct {
    nvim_window::win_struct::win_ref(nvim_window::WinHandle::from_ptr(wp.cast_mut()))
}
/// Mutable access to `WinStruct` fields from a mut `win_T` pointer.
#[inline]
unsafe fn win_mut_raw<'a>(wp: *mut std::ffi::c_void) -> &'a mut nvim_window::win_struct::WinStruct {
    nvim_window::win_struct::win_mut(nvim_window::WinHandle::from_ptr(wp))
}

/// Line number type
type LinenrT = i32;

/// Column number type
type ColnrT = c_int;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    static Columns: c_int;
    static mut got_int: bool;
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

    let idx = win_ref_raw(wp).w_tagstackidx;
    let len = win_ref_raw(wp).w_tagstacklen;

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

    let idx = win_ref_raw(wp).w_tagstackidx;
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

    let idx = win_ref_raw(wp).w_tagstackidx;
    let len = win_ref_raw(wp).w_tagstacklen;

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

    let idx = win_ref_raw(wp).w_tagstackidx;
    let len = win_ref_raw(wp).w_tagstacklen;
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

    let idx = win_ref_raw(wp).w_tagstackidx;
    let len = win_ref_raw(wp).w_tagstacklen;

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
// Phase 7: Tag display and location list functions
// =============================================================================

// Additional type aliases
type DictHandle = *mut c_void;
type ListHandle = *mut c_void;

// Constants
const TAB: u8 = 0x09;
const OK: c_int = 1;
const FAIL: c_int = 0;
const HLF_T: c_int = 23;
const HLF_D: c_int = 5;
const HLF_CM: c_int = 11;
const MAXPATHL: usize = 4096;
const CMDBUFFSIZE_C: usize = 1024;
const MAXCOL: c_int = 0x7FFF_FFFF;

#[allow(dead_code)]
extern "C" {
    // Parse and file name functions (already in Rust, callable via FFI)
    fn rs_parse_match(lbuf: *mut c_char, tagp: *mut c_void) -> c_int;
    fn rs_tag_full_fname(tagp: *mut c_void) -> *mut c_char;
    fn rs_taglen_advance(l: c_int);

    // Tag stack accessors (also in stack.rs extern block, re-declared here)
    fn nvim_win_get_tagstack_entry(wp: *const c_void, idx: c_int) -> *const c_void;
    fn nvim_taggy_get_cur_match(tg: *const c_void) -> c_int;
    fn nvim_taggy_get_tagname(tg: *const c_void) -> *const c_char;
    fn nvim_taggy_get_fmark(tg: *const c_void) -> *const c_void;

    // Message output functions
    fn msg_ext_set_kind(kind: *const c_char);
    fn msg_start();
    fn msg_puts_hl(msg: *const c_char, attr: c_int, right: bool);
    fn msg_clr_eos();
    fn msg_advance(col: c_int);
    fn msg_puts(s: *const c_char);
    fn msg_puts_title(s: *const c_char);
    fn msg_outtrans(str: *const c_char, attr: c_int, right: bool);
    fn msg_outtrans_len(str: *const c_char, len: c_int, attr: c_int, right: bool);
    fn msg_outtrans_one(p: *const c_char, hl_id: c_int, right: bool) -> *const c_char;
    fn msg_putchar(c: c_int);
    fn os_breakcheck();
    fn verbose_enter();
    fn verbose_leave();
    fn smsg(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    fn emsg(s: *const c_char) -> c_int;
    fn gettext(msgid: *const c_char) -> *const c_char;

    // Global variable accessors
    static mut msg_col: c_int;
    static mut msg_didout: bool;
    fn nvim_get_p_verbose() -> c_int;
    fn ui_has(ext: c_int) -> bool;
    #[link_name = "ptr2cells"]
    fn nvim_ptr2cells(p: *const c_char) -> c_int;
    fn nvim_get_curbuf_fnum() -> c_int;

    // Tag-specific accessors
    fn nvim_get_g_do_tagpreview() -> c_int;
    fn nvim_tag_get_ptag_cur_match() -> c_int;
    fn nvim_tag_get_curwin() -> *mut c_void;
    fn nvim_tag_fm_getname(tg: *const c_void, lead_len: c_int) -> *mut c_char;
    fn nvim_taggy_get_fmark_fnum(tg: *const c_void) -> c_int;

    // Dictionary/list operations
    fn tv_dict_alloc() -> DictHandle;
    fn nvim_tag_tv_dict_find(dict: DictHandle, key: *const c_char, key_len: c_int) -> bool;
    fn tv_dict_add_str(
        dict: DictHandle,
        key: *const c_char,
        key_len: usize,
        val: *mut c_char,
    ) -> c_int;
    #[link_name = "tv_dict_add_nr"]
    fn nvim_tag_tv_dict_add_nr(
        dict: DictHandle,
        key: *const c_char,
        key_len: usize,
        nr: i64,
    ) -> c_int;
    fn tv_list_alloc(count: isize) -> ListHandle;
    fn tv_list_append_dict(list: ListHandle, dict: DictHandle);
    fn tv_list_free(list: ListHandle);
    fn nvim_tag_set_errorlist(list: ListHandle, title: *const c_char);

    // Memory management
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
    fn xmemcpyz(dst: *mut c_char, src: *const c_char, len: usize);
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn atoi(s: *const c_char) -> c_int;
}

// Helper to get TagPtrs fields. We know the TagPtrs layout from parse.rs.
// We access it through the opaque handle because commands.rs doesn't import parse directly.
// TagPtrs is a #[repr(C)] struct with known offsets.
use crate::parse::TagPtrs;

/// Access a field from tagptrs_T (tagname: offset 0)
unsafe fn tagp_tagname(tagp: *const TagPtrs) -> *mut c_char {
    (*tagp).tagname
}
unsafe fn tagp_tagname_end(tagp: *const TagPtrs) -> *mut c_char {
    (*tagp).tagname_end
}
unsafe fn tagp_command(tagp: *const TagPtrs) -> *mut c_char {
    (*tagp).command
}
unsafe fn tagp_command_end(tagp: *const TagPtrs) -> *mut c_char {
    (*tagp).command_end
}
unsafe fn tagp_tagkind(tagp: *const TagPtrs) -> *mut c_char {
    (*tagp).tagkind
}
unsafe fn tagp_tagkind_end(tagp: *const TagPtrs) -> *mut c_char {
    (*tagp).tagkind_end
}

/// Check if a byte is an ASCII space character
fn ascii_isspace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
}

// -------------------------------------------------------------------------
// rs_print_tag_list — format tag matches for :tselect display
// -------------------------------------------------------------------------

/// Print the tag list for :tselect display.
///
/// # Safety
/// - `matches` must be a valid pointer to `num_matches` C strings
/// - Each match string must be valid and parseable by `parse_match`
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_print_tag_list(
    new_tag: bool,
    use_tagstack: bool,
    num_matches: c_int,
    matches: *mut *mut c_char,
) {
    if matches.is_null() || num_matches <= 0 {
        return;
    }

    let curwin = nvim_tag_get_curwin();

    // Get current match info from tag stack
    let tagstackidx = win_ref_raw(curwin).w_tagstackidx;
    let tagstack_entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
    let tagstack_cur_match = if tagstack_entry.is_null() {
        -1
    } else {
        nvim_taggy_get_cur_match(tagstack_entry)
    };

    // Parse first match to determine tag name column width
    let mut tagp: TagPtrs = TagPtrs::default();
    let tagp_ptr: *mut TagPtrs = &raw mut tagp;
    rs_parse_match(*matches, tagp_ptr.cast());

    let tagname_len = tagp_tagname_end(tagp_ptr).offset_from(tagp_tagname(tagp_ptr)) as c_int + 2;
    let taglen = if tagname_len < 18 { 18 } else { tagname_len };
    let taglen = if taglen > Columns - 25 {
        MAXCOL
    } else {
        taglen
    };

    if msg_col == 0 {
        msg_didout = false; // overwrite previous message
    }
    msg_ext_set_kind(c"confirm".as_ptr());
    msg_start();
    msg_puts_hl(c"  # pri kind tag".as_ptr(), HLF_T, false);
    msg_clr_eos();
    rs_taglen_advance(taglen);
    msg_puts_hl(c"file\n".as_ptr(), HLF_T, false);

    for i in 0..num_matches {
        if unsafe { got_int } {
            break;
        }

        rs_parse_match(*matches.add(i as usize), tagp_ptr.cast());

        // Determine if this is the current match
        let is_current = !new_tag
            && ((nvim_get_g_do_tagpreview() != 0 && i == nvim_tag_get_ptag_cur_match())
                || (use_tagstack && i == tagstack_cur_match));

        // Format and print entry header (inline port of nvim_tag_list_format_entry)
        let match_byte = *(*matches.add(i as usize)) as u8;
        let mt_idx = (match_byte & MT_MASK) as c_int;
        let mt_name = crate::rs_tag_match_type_name(mt_idx);
        {
            // Format: "<prefix>%2d <mt_name> " where prefix is '>' or ' '
            let prefix = if is_current {
                b'>' as c_char
            } else {
                b' ' as c_char
            };
            let mut buf = [0u8; 32];
            buf[0] = prefix as u8;
            snprintf(
                buf.as_mut_ptr().add(1).cast(),
                31,
                c"%2d %s ".as_ptr(),
                i + 1,
                mt_name,
            );
            msg_puts(buf.as_ptr().cast());
        }

        // Print tag kind if available
        let tagkind = tagp_tagkind(tagp_ptr);
        if !tagkind.is_null() {
            let tagkind_end = tagp_tagkind_end(tagp_ptr);
            let kind_len = tagkind_end.offset_from(tagkind) as c_int;
            msg_outtrans_len(tagkind, kind_len, 0, false);
        }
        msg_advance(13);

        // Print tag name
        let tagname = tagp_tagname(tagp_ptr);
        let tagname_end = tagp_tagname_end(tagp_ptr);
        let name_len = tagname_end.offset_from(tagname) as c_int;
        msg_outtrans_len(tagname, name_len, HLF_T, false);
        msg_putchar(b' ' as c_int);
        rs_taglen_advance(taglen);

        // Print file name
        let p = rs_tag_full_fname(tagp_ptr.cast());
        if !p.is_null() {
            msg_outtrans(p, HLF_D, false);
            xfree(p.cast());
        }
        if msg_col > 0 {
            msg_putchar(b'\n' as c_int);
        }
        if unsafe { got_int } {
            break;
        }
        msg_advance(15);

        // Print extra fields
        let command_end = tagp_command_end(tagp_ptr);
        if command_end.is_null() {
            // No command_end: find end of command
            let mut p = tagp_command(tagp_ptr);
            while *p != 0 && *p as u8 != b'\r' && *p as u8 != b'\n' {
                p = p.add(1);
            }
        } else {
            let mut p = command_end.add(3);
            while *p != 0 && *p as u8 != b'\r' && *p as u8 != b'\n' {
                // Skip TAB characters
                while *p as u8 == TAB {
                    p = p.add(1);
                }

                // skip "file:" without a value (static tag)
                if strncmp(p, c"file:".as_ptr(), 5) == 0 && ascii_isspace(*p.add(5) as u8) {
                    p = p.add(5);
                    continue;
                }

                // skip "kind:<kind>" and "<kind>"
                let tagkind = tagp_tagkind(tagp_ptr);
                if p == tagkind || (p.add(5) == tagkind && strncmp(p, c"kind:".as_ptr(), 5) == 0) {
                    p = tagp_tagkind_end(tagp_ptr);
                    continue;
                }

                // print all other extra fields
                let mut hl_id: c_int = HLF_CM;
                while *p != 0 && *p as u8 != b'\r' && *p as u8 != b'\n' {
                    if msg_col + nvim_ptr2cells(p) >= Columns {
                        msg_putchar(b'\n' as c_int);
                        if unsafe { got_int } {
                            break;
                        }
                        msg_advance(15);
                    }
                    p = msg_outtrans_one(p, hl_id, false).cast_mut();
                    if *p as u8 == TAB {
                        msg_puts_hl(c" ".as_ptr(), hl_id, false);
                        break;
                    }
                    if *p as u8 == b':' {
                        hl_id = 0;
                    }
                }
            }
            if msg_col > 15 {
                msg_putchar(b'\n' as c_int);
                if unsafe { got_int } {
                    break;
                }
                msg_advance(15);
            }
        }

        // Put the info (in several lines) at column 15.
        // Don't display "/^" and "?^".
        let command_end_for_display = if tagp_command_end(tagp_ptr).is_null() {
            // find end of command
            let mut pe = tagp_command(tagp_ptr);
            while *pe != 0 && *pe as u8 != b'\r' && *pe as u8 != b'\n' {
                pe = pe.add(1);
            }
            pe
        } else {
            tagp_command_end(tagp_ptr)
        };

        let mut p = tagp_command(tagp_ptr);
        if *p as u8 == b'/' || *p as u8 == b'?' {
            p = p.add(1);
            if *p as u8 == b'^' {
                p = p.add(1);
            }
        }
        // Remove leading whitespace from pattern
        while p != command_end_for_display && ascii_isspace(*p as u8) {
            p = p.add(1);
        }

        let command_char = *tagp_command(tagp_ptr) as u8;
        while p != command_end_for_display {
            let cell_width = if *p as u8 == TAB {
                1
            } else {
                nvim_ptr2cells(p)
            };
            if msg_col + cell_width > Columns {
                msg_putchar(b'\n' as c_int);
            }
            if unsafe { got_int } {
                break;
            }
            msg_advance(15);

            // skip backslash used for escaping
            if *p as u8 == b'\\' && (*p.add(1) as u8 == command_char || *p.add(1) as u8 == b'\\') {
                p = p.add(1);
            }

            if *p as u8 == TAB {
                msg_putchar(b' ' as c_int);
                p = p.add(1);
            } else {
                p = msg_outtrans_one(p, 0, false).cast_mut();
            }

            // don't display the "$/;\"" and "$?;\""
            if p == command_end_for_display.sub(2)
                && *p as u8 == b'$'
                && *p.add(1) as u8 == command_char
            {
                break;
            }
            // don't display matching '/' or '?'
            if p == command_end_for_display.sub(1)
                && *p as u8 == command_char
                && (command_char == b'/' || command_char == b'?')
            {
                break;
            }
        }

        if msg_col != 0 && (!ui_has(K_UI_MESSAGES) || i < num_matches - 1) {
            msg_putchar(b'\n' as c_int);
        }
        os_breakcheck();
    }

    if unsafe { got_int } {
        unsafe {
            got_int = false;
        } // only stop the listing
    }
}

// -------------------------------------------------------------------------
// rs_add_llist_tags — add matching tags to location list
// -------------------------------------------------------------------------

/// Add the matching tags to the location list for the current window.
///
/// # Safety
/// - `tag` must be a valid C string
/// - `matches` must point to `num_matches` valid C strings
#[no_mangle]
pub unsafe extern "C" fn rs_add_llist_tags(
    tag: *const c_char,
    num_matches: c_int,
    matches: *mut *mut c_char,
) -> c_int {
    if tag.is_null() || matches.is_null() || num_matches <= 0 {
        return OK;
    }

    let fname: *mut c_char = xmalloc(MAXPATHL + 1).cast();
    let cmd: *mut c_char = xmalloc(CMDBUFFSIZE_C + 1).cast();
    let list = tv_list_alloc(0);

    let mut tagp: TagPtrs = TagPtrs::default();
    let tagp_ptr: *mut TagPtrs = &raw mut tagp;

    for i in 0..num_matches {
        rs_parse_match(*matches.add(i as usize), tagp_ptr.cast());

        // Save the tag name (max 128 chars)
        let tagname = tagp_tagname(tagp_ptr);
        let tagname_end = tagp_tagname_end(tagp_ptr);
        let name_len = tagname_end.offset_from(tagname) as usize;
        let name_len = if name_len > 128 { 128 } else { name_len };
        let mut tag_name_buf = [0u8; 129];
        std::ptr::copy_nonoverlapping(tagname as *const u8, tag_name_buf.as_mut_ptr(), name_len);
        tag_name_buf[name_len] = 0;

        // Save the tag file name
        let p = rs_tag_full_fname(tagp_ptr.cast());
        if p.is_null() {
            continue;
        }
        xstrlcpy(fname, p, MAXPATHL);
        xfree(p.cast());

        // Get the line number or the search pattern
        let mut lnum: LinenrT = 0;
        let command = tagp_command(tagp_ptr);
        if (*command as u8).is_ascii_digit() {
            // Line number
            lnum = atoi(command) as LinenrT;
        } else {
            // Search pattern
            let mut cmd_start = command;
            let mut cmd_end = tagp_command_end(tagp_ptr);
            if cmd_end.is_null() {
                let mut pe = command;
                while *pe != 0 && *pe as u8 != b'\r' && *pe as u8 != b'\n' {
                    pe = pe.add(1);
                }
                cmd_end = pe;
            }

            // Adjust to point to last character
            cmd_end = cmd_end.sub(1);

            // Skip leading/trailing delimiters
            if *cmd_start as u8 == b'/' || *cmd_start as u8 == b'?' {
                cmd_start = cmd_start.add(1);
            }
            if *cmd_end as u8 == b'/' || *cmd_end as u8 == b'?' {
                cmd_end = cmd_end.sub(1);
            }

            let mut pos = 0usize;
            *cmd = 0;

            // If "^" is present, copy it first
            if *cmd_start as u8 == b'^' {
                *cmd = b'^' as c_char;
                pos = 1;
                cmd_start = cmd_start.add(1);
            }

            // Precede with \V for very nomagic
            *cmd.add(pos) = b'\\' as c_char;
            *cmd.add(pos + 1) = b'V' as c_char;
            pos += 2;

            let content_len = cmd_end.offset_from(cmd_start) as usize + 1;
            let max_copy = if content_len > CMDBUFFSIZE_C - 5 {
                CMDBUFFSIZE_C - 5
            } else {
                content_len
            };
            snprintf(
                cmd.add(pos),
                CMDBUFFSIZE_C + 1 - pos,
                c"%.*s".as_ptr(),
                max_copy as c_int,
                cmd_start,
            );
            pos += max_copy;

            // Replace '$' at end with '\$'
            if pos > 0 && *cmd.add(pos - 1) as u8 == b'$' {
                *cmd.add(pos - 1) = b'\\' as c_char;
                *cmd.add(pos) = b'$' as c_char;
                pos += 1;
            }

            *cmd.add(pos) = 0;
        }

        let dict = tv_dict_alloc();
        tv_list_append_dict(list, dict);

        tv_dict_add_str(dict, c"text".as_ptr(), 4, tag_name_buf.as_mut_ptr().cast());
        tv_dict_add_str(dict, c"filename".as_ptr(), 8, fname);
        nvim_tag_tv_dict_add_nr(dict, c"lnum".as_ptr(), 4, lnum as i64);
        if lnum == 0 {
            tv_dict_add_str(dict, c"pattern".as_ptr(), 7, cmd);
        }
    }

    // Format "ltag <tag>" as the errorlist title directly into a local buffer
    let mut title_buf = [0u8; MAXPATHL + 8];
    snprintf(
        title_buf.as_mut_ptr().cast(),
        title_buf.len(),
        c"ltag %s".as_ptr(),
        tag,
    );
    nvim_tag_set_errorlist(list, title_buf.as_ptr().cast());

    tv_list_free(list);
    xfree(fname.cast());
    xfree(cmd.cast());

    OK
}

// -------------------------------------------------------------------------
// rs_do_tags — print the tag stack
// -------------------------------------------------------------------------

/// Print the tag stack (`:tags` command).
///
/// # Safety
/// - Called from C with a valid `exarg_T` pointer (unused in this implementation)
#[export_name = "do_tags"]
pub unsafe extern "C" fn rs_do_tags(_eap: *mut std::ffi::c_void) {
    let curwin = nvim_tag_get_curwin();
    let tagstackidx = win_ref_raw(curwin).w_tagstackidx;
    let tagstacklen = win_ref_raw(curwin).w_tagstacklen;

    // Highlight title
    msg_puts_title(c"\n  # TO tag         FROM line  in file/text".as_ptr());

    for i in 0..tagstacklen {
        let entry = nvim_win_get_tagstack_entry(curwin, i);
        let tagname = nvim_taggy_get_tagname(entry);
        if tagname.is_null() {
            continue;
        }

        let name = nvim_tag_fm_getname(entry, 30);
        if name.is_null() {
            continue; // file name not available
        }

        msg_putchar(b'\n' as c_int);

        // Get fmark lnum
        let fmark = nvim_taggy_get_fmark(entry);
        let lnum = nvim_fmark_get_lnum(fmark) as i64;
        let cur_match = nvim_taggy_get_cur_match(entry);

        // Inline port of nvim_tag_format_tags_line (Phase 1 migration)
        // Format: "%c%2d %2d %-15s %5lld  "
        let is_current_char = if i == tagstackidx {
            b'>' as c_char
        } else {
            b' ' as c_char
        };
        let mut line_buf = [0u8; 64];
        snprintf(
            line_buf.as_mut_ptr().cast(),
            64,
            c"%c%2d %2d %-15s %5lld  ".as_ptr(),
            is_current_char as c_int,
            i + 1,
            cur_match + 1,
            tagname,
            lnum,
        );
        msg_outtrans(line_buf.as_ptr().cast(), 0, false);

        let fmark_fnum = nvim_taggy_get_fmark_fnum(entry);
        let curbuf_fnum = nvim_get_curbuf_fnum();
        let attr = if fmark_fnum == curbuf_fnum { HLF_D } else { 0 };
        msg_outtrans(name, attr, false);
        xfree(name.cast());
    }

    if tagstackidx == tagstacklen {
        // idx at top of stack
        msg_puts(c"\n>".as_ptr());
    }
}

extern "C" {
    fn nvim_fmark_get_lnum(fm: *const c_void) -> LinenrT;
}

// -------------------------------------------------------------------------
// rs_add_tag_field — add a tag field to a dictionary
// -------------------------------------------------------------------------

/// Add a tag field to a dictionary, checking for duplicate field names.
///
/// # Safety
/// - `dict` must be a valid dict_T pointer
/// - `field_name` must be a valid C string
/// - `start` may be null (empty value will be added)
/// - `end` may be null (will use strlen to find end of start)
#[no_mangle]
pub unsafe extern "C" fn rs_add_tag_field(
    dict: DictHandle,
    field_name: *const c_char,
    start: *const c_char,
    end: *const c_char,
) -> c_int {
    if dict.is_null() || field_name.is_null() {
        return FAIL;
    }

    // Check that the field name doesn't exist yet
    if nvim_tag_tv_dict_find(dict, field_name, -1) {
        if nvim_get_p_verbose() > 0 {
            verbose_enter();
            smsg(0, gettext(c"Duplicate field name: %s".as_ptr()), field_name);
            verbose_leave();
        }
        return FAIL;
    }

    let buf: *mut c_char = xmalloc(MAXPATHL).cast();
    let mut len: usize = 0;

    if !start.is_null() {
        let actual_end = if end.is_null() {
            let mut e = start.add(strlen(start));
            while e > start && (*e.sub(1) as u8 == b'\r' || *e.sub(1) as u8 == b'\n') {
                e = e.sub(1);
            }
            e
        } else {
            end
        };

        len = actual_end.offset_from(start) as usize;
        if len > MAXPATHL - 1 {
            len = MAXPATHL - 1;
        }
        xmemcpyz(buf, start, len);
    }
    *buf.add(len) = 0;

    let field_name_len = strlen(field_name);
    let retval = tv_dict_add_str(dict, field_name, field_name_len, buf);
    xfree(buf.cast());
    retval
}

// =============================================================================
// Phase 8: VimL API and tag stack setters
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Phase 8 FFI functions (direct C functions)
    fn find_tags(
        pat: *mut c_char,
        num_matches: *mut c_int,
        matchesp: *mut *mut *mut c_char,
        flags: c_int,
        mincount: c_int,
        buf_ffname: *mut c_char,
    ) -> c_int;
    fn FreeWild(count: c_int, files: *mut *mut c_char);
    fn nvim_tag_get_curbuf_ffname() -> *mut c_char;
    fn nvim_tag_mb_ptr_adv(p: *const c_char) -> *mut c_char;
    fn nvim_tag_tv_dict_find_item(
        dict: *const c_void,
        key: *const c_char,
        key_len: c_int,
    ) -> *mut c_void;
    fn nvim_tag_dictitem_tv(di: *mut c_void) -> *mut c_void;
    fn nvim_tag_tv_is_list(tv: *const c_void) -> bool;
    fn nvim_tag_tv_get_list(tv: *const c_void) -> *mut c_void;
    fn tv_get_number(tv: *const c_void) -> i64;
    fn tv_dict_get_string(dict: *const c_void, key: *const c_char, save: bool) -> *mut c_char;
    fn tv_dict_get_number(dict: *const c_void, key: *const c_char) -> i64;
    fn nvim_tag_tv_list_first(list: *const c_void) -> *mut c_void;
    fn nvim_tag_tv_list_item_next(list: *const c_void, li: *const c_void) -> *mut c_void;
    fn nvim_tag_listitem_get_dict(li: *const c_void) -> *mut c_void;
    fn nvim_tag_list2fpos(
        tv: *mut c_void,
        lnum: *mut i32,
        col: *mut i32,
        coladd: *mut i32,
        fnum: *mut c_int,
    ) -> c_int;
    fn tv_list_append_number(list: *mut c_void, nr: i64);
    fn tv_dict_add_list(dict: *mut c_void, key: *const c_char, key_len: usize, list: *mut c_void);
    fn nvim_taggy_get_user_data(tg: *const c_void) -> *const c_char;
    fn nvim_taggy_get_fmark_col(tg: *const c_void) -> c_int;
    fn nvim_tag_taggy_fmark_coladd(tg: *const c_void) -> c_int;

    // Already-migrated Rust functions callable via FFI
    fn rs_test_for_static(tagp: *const c_void) -> bool;
    fn rs_tagstack_clear(wp: *mut c_void);
    fn rs_tagstack_truncate(wp: *mut c_void);
    fn rs_tagstack_set_idx(wp: *mut c_void, idx: c_int);
    fn rs_tagstack_push(
        wp: *mut c_void,
        tagname: *mut c_char,
        cur_fnum: c_int,
        cur_match: c_int,
        mark_lnum: i32,
        mark_col: c_int,
        fnum: c_int,
        user_data: *mut c_char,
    );

    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

use crate::search::find_tags_flags;

// -------------------------------------------------------------------------
// rs_expand_tags — tag name completion
// -------------------------------------------------------------------------

/// Expand tag names matching pattern for command-line completion.
///
/// # Safety
/// - `pat` must be a valid C string
/// - `num_file` and `file` must be valid pointers
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_expand_tags(
    tagnames: bool,
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
) -> c_int {
    if pat.is_null() || num_file.is_null() || file.is_null() {
        return FAIL;
    }

    let mut name_buf_size: usize = 100;
    let mut name_buf: *mut c_char = xmalloc(name_buf_size).cast();

    let extra_flag = if tagnames {
        find_tags_flags::TAG_NAMES
    } else {
        0
    };

    let buf_ffname = nvim_tag_get_curbuf_ffname();
    let tag_many: c_int = 300; // TAG_MANY

    let ret = if *pat as u8 == b'/' {
        find_tags(
            pat.add(1),
            num_file,
            file,
            find_tags_flags::TAG_REGEXP
                | extra_flag
                | find_tags_flags::TAG_VERBOSE
                | find_tags_flags::TAG_NO_TAGFUNC,
            tag_many,
            buf_ffname,
        )
    } else {
        find_tags(
            pat,
            num_file,
            file,
            find_tags_flags::TAG_REGEXP
                | extra_flag
                | find_tags_flags::TAG_VERBOSE
                | find_tags_flags::TAG_NO_TAGFUNC
                | find_tags_flags::TAG_NOIC,
            tag_many,
            buf_ffname,
        )
    };

    if ret == OK && !tagnames {
        // Reorganize the tags for display and matching as strings of:
        // "<tagname>\0<kind>\0<filename>\0"
        let n = *num_file;
        for i in 0..n as usize {
            let mut tagp: TagPtrs = TagPtrs::default();
            let tagp_ptr: *mut TagPtrs = &raw mut tagp;
            rs_parse_match(*(*file).add(i), tagp_ptr.cast());

            let tagname = tagp_tagname(tagp_ptr);
            let tagname_end = tagp_tagname_end(tagp_ptr);
            let len = tagname_end.offset_from(tagname) as usize;

            if len > name_buf_size - 3 {
                name_buf_size = len + 3;
                name_buf = xrealloc(name_buf.cast(), name_buf_size).cast();
            }

            memmove(name_buf.cast(), tagname.cast(), len);
            let mut pos = len;
            *name_buf.add(pos) = 0;
            pos += 1;

            // kind character
            let tagkind = tagp_tagkind(tagp_ptr);
            let kind_char = if !tagkind.is_null() && *tagkind != 0 {
                *tagkind
            } else {
                b'f' as c_char
            };
            *name_buf.add(pos) = kind_char;
            pos += 1;
            *name_buf.add(pos) = 0;
            pos += 1;

            // Copy filename after the tag+kind prefix
            let fname = (*tagp_ptr).fname;
            let fname_end = (*tagp_ptr).fname_end;
            let fname_len = fname_end.offset_from(fname) as usize;
            memmove((*(*file).add(i)).add(pos).cast(), fname.cast(), fname_len);
            *(*(*file).add(i)).add(pos + fname_len) = 0;
            memmove((*(*file).add(i)).cast(), name_buf.cast(), pos);
        }
    }

    xfree(name_buf.cast());
    ret
}

// -------------------------------------------------------------------------
// rs_get_tags — return tag info as VimL list of dicts
// -------------------------------------------------------------------------

/// Add matching tags to a VimL list as dictionaries.
///
/// # Safety
/// - `list` must be a valid list_T pointer
/// - `pat` must be a valid C string
/// - `buf_fname` may be null
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_get_tags(
    list: *mut c_void,
    pat: *mut c_char,
    buf_fname: *mut c_char,
) -> c_int {
    if list.is_null() || pat.is_null() {
        return FAIL;
    }

    let mut num_matches: c_int = 0;
    let mut matches: *mut *mut c_char = std::ptr::null_mut();

    let ret = find_tags(
        pat,
        &raw mut num_matches,
        &raw mut matches,
        find_tags_flags::TAG_REGEXP | find_tags_flags::TAG_NOIC,
        MAXCOL,
        buf_fname,
    );

    if ret != OK || num_matches <= 0 {
        return ret;
    }

    let mut ret = ret;

    for i in 0..num_matches as usize {
        let mut tagp: TagPtrs = TagPtrs::default();
        let tagp_ptr: *mut TagPtrs = &raw mut tagp;

        if rs_parse_match(*matches.add(i), tagp_ptr.cast()) == FAIL {
            xfree((*matches.add(i)).cast());
            continue;
        }

        let is_static = rs_test_for_static(tagp_ptr.cast());

        // Skip pseudo-tag lines
        let tagname = tagp_tagname(tagp_ptr);
        if strncmp(tagname, c"!_TAG_".as_ptr(), 6) == 0 {
            xfree((*matches.add(i)).cast());
            continue;
        }

        let dict = tv_dict_alloc();
        tv_list_append_dict(list, dict);

        let full_fname = rs_tag_full_fname(tagp_ptr.cast());
        let tagname_end = tagp_tagname_end(tagp_ptr);
        let command = tagp_command(tagp_ptr);
        let command_end = tagp_command_end(tagp_ptr);
        let tagkind = tagp_tagkind(tagp_ptr);
        let tagkind_end = if tagkind.is_null() {
            std::ptr::null()
        } else {
            tagp_tagkind_end(tagp_ptr).cast_const()
        };

        if rs_add_tag_field(dict, c"name".as_ptr(), tagname, tagname_end) == FAIL
            || rs_add_tag_field(dict, c"filename".as_ptr(), full_fname, std::ptr::null()) == FAIL
            || rs_add_tag_field(dict, c"cmd".as_ptr(), command, command_end) == FAIL
            || rs_add_tag_field(dict, c"kind".as_ptr(), tagkind, tagkind_end) == FAIL
            || nvim_tag_tv_dict_add_nr(dict, c"static".as_ptr(), 6, i64::from(is_static)) == FAIL
        {
            ret = FAIL;
        }

        xfree(full_fname.cast());

        // Parse extra fields after command_end
        if !command_end.is_null() {
            let mut p = command_end.add(3);
            while *p != 0 && *p as u8 != b'\n' && *p as u8 != b'\r' {
                if p == tagkind
                    || (!tagkind.is_null()
                        && p.add(5) == tagkind
                        && strncmp(p, c"kind:".as_ptr(), 5) == 0)
                {
                    // skip "kind:<kind>" and "<kind>"
                    p = tagp_tagkind_end(tagp_ptr).sub(1);
                } else if strncmp(p, c"file:".as_ptr(), 5) == 0 {
                    // skip "file:" (static tag)
                    p = p.add(4);
                } else if !matches!(*p as u8, b' ' | b'\t') {
                    // Add extra field as a dict entry
                    let n = p;
                    while *p != 0 && *p as u8 >= b' ' && (*p as u8) < 127 && *p as u8 != b':' {
                        p = p.add(1);
                    }
                    let field_len = p.offset_from(n) as usize;
                    if *p as u8 == b':' && field_len > 0 {
                        let s = p.add(1);
                        p = s;
                        while *p != 0 && *p as u8 >= b' ' {
                            p = p.add(1);
                        }
                        // Temporarily null-terminate the field name
                        let saved = *n.add(field_len);
                        *n.cast::<c_char>().add(field_len) = 0;
                        if rs_add_tag_field(dict, n, s, p) == FAIL {
                            ret = FAIL;
                        }
                        *n.cast::<c_char>().add(field_len) = saved;
                    } else {
                        // Skip field without colon
                        while *p != 0 && *p as u8 >= b' ' {
                            p = p.add(1);
                        }
                    }
                    if *p == 0 {
                        break;
                    }
                }
                p = nvim_tag_mb_ptr_adv(p);
            }
        }

        xfree((*matches.add(i)).cast());
    }

    xfree(matches.cast());
    ret
}

// -------------------------------------------------------------------------
// rs_get_tag_details — helper to add tag details to a dict
// -------------------------------------------------------------------------

/// Add tag details from a taggy_T entry to a dictionary.
#[no_mangle]
pub unsafe extern "C" fn rs_get_tag_details(entry: *const c_void, retdict: *mut c_void) {
    let tagname = nvim_taggy_get_tagname(entry);
    let cur_match = nvim_taggy_get_cur_match(entry);
    let cur_fnum = nvim_taggy_get_cur_fnum(entry);
    let user_data = nvim_taggy_get_user_data(entry);

    tv_dict_add_str(retdict, c"tagname".as_ptr(), 7, tagname.cast_mut());
    nvim_tag_tv_dict_add_nr(retdict, c"matchnr".as_ptr(), 7, (cur_match + 1) as i64);
    nvim_tag_tv_dict_add_nr(retdict, c"bufnr".as_ptr(), 5, cur_fnum as i64);

    if !user_data.is_null() {
        tv_dict_add_str(retdict, c"user_data".as_ptr(), 9, user_data.cast_mut());
    }

    let pos = tv_list_alloc(4);
    tv_dict_add_list(retdict, c"from".as_ptr(), 4, pos);

    let fmark = nvim_taggy_get_fmark(entry);
    let mark_fnum = nvim_fmark_get_fnum(fmark);
    let fmark_lnum = nvim_fmark_get_lnum(fmark) as i64;
    let fmark_col = nvim_taggy_get_fmark_col(entry);
    let fmark_coladd = nvim_tag_taggy_fmark_coladd(entry);

    tv_list_append_number(pos, if mark_fnum == -1 { 0 } else { mark_fnum as i64 });
    tv_list_append_number(pos, fmark_lnum);
    tv_list_append_number(
        pos,
        if fmark_col == MAXCOL {
            MAXCOL as i64
        } else {
            (fmark_col + 1) as i64
        },
    );
    tv_list_append_number(pos, fmark_coladd as i64);
}

extern "C" {
    fn nvim_fmark_get_fnum(fm: *const c_void) -> c_int;
    fn nvim_taggy_get_cur_fnum(tg: *const c_void) -> c_int;
}

// -------------------------------------------------------------------------
// rs_get_tagstack — return tag stack as VimL dict
// -------------------------------------------------------------------------

/// Return the tag stack entries of the specified window in a dictionary.
///
/// # Safety
/// - `wp` must be a valid win_T pointer
/// - `retdict` must be a valid dict_T pointer
#[no_mangle]
pub unsafe extern "C" fn rs_get_tagstack(wp: *mut c_void, retdict: *mut c_void) {
    if wp.is_null() || retdict.is_null() {
        return;
    }

    let tagstacklen = win_ref_raw(wp).w_tagstacklen;
    let tagstackidx = win_ref_raw(wp).w_tagstackidx;

    nvim_tag_tv_dict_add_nr(retdict, c"length".as_ptr(), 6, tagstacklen as i64);
    nvim_tag_tv_dict_add_nr(retdict, c"curidx".as_ptr(), 6, (tagstackidx + 1) as i64);

    let items_list = tv_list_alloc(2);
    tv_dict_add_list(retdict, c"items".as_ptr(), 5, items_list);

    for i in 0..tagstacklen {
        let entry = nvim_win_get_tagstack_entry(wp, i);
        let d = tv_dict_alloc();
        tv_list_append_dict(items_list, d);
        rs_get_tag_details(entry, d);
    }
}

// -------------------------------------------------------------------------
// rs_set_tagstack — set tag stack from VimL dict
// -------------------------------------------------------------------------

/// Set the tag stack entries of the specified window.
///
/// `action` is one of: 'a' (append), 'r' (replace), 't' (truncate)
///
/// # Safety
/// - `wp` must be a valid win_T pointer
/// - `d` must be a valid dict_T pointer
#[no_mangle]
pub unsafe extern "C" fn rs_set_tagstack(
    wp: *mut c_void,
    d: *const c_void,
    action: c_int,
) -> c_int {
    if wp.is_null() {
        return FAIL;
    }

    // not allowed to alter the tag stack entries from inside tagfunc
    if crate::tag_get_tfu_in_use() {
        emsg(gettext(E_CANNOT_MODIFY_TAG_STACK_WITHIN_TAGFUNC.as_ptr()));
        return FAIL;
    }

    let mut list: *mut c_void = std::ptr::null_mut();

    // Check for "items" key
    let di = nvim_tag_tv_dict_find_item(d, c"items".as_ptr(), -1);
    if !di.is_null() {
        let tv = nvim_tag_dictitem_tv(di);
        if !nvim_tag_tv_is_list(tv) {
            emsg(gettext(E_LISTREQ.as_ptr()));
            return FAIL;
        }
        list = nvim_tag_tv_get_list(tv);
    }

    // Check for "curidx" key
    let di = nvim_tag_tv_dict_find_item(d, c"curidx".as_ptr(), -1);
    if !di.is_null() {
        let tv = nvim_tag_dictitem_tv(di);
        let curidx = tv_get_number(tv) as c_int - 1;
        rs_tagstack_set_idx(wp, curidx);
    }

    if action == b't' as c_int {
        // truncate the stack
        rs_tagstack_truncate(wp);
    }

    if !list.is_null() {
        if action == b'r' as c_int {
            // replace the stack
            rs_tagstack_clear(wp);
        }

        // Push items from the list
        tagstack_push_items_from_list(wp, list);

        // set the current index after the last entry
        let len = win_ref_raw(wp).w_tagstacklen;
        win_mut_raw(wp).w_tagstackidx = len;
    }

    OK
}

/// Push tag stack items from a VimL list.
unsafe fn tagstack_push_items_from_list(wp: *mut c_void, list: *mut c_void) {
    let mut li = nvim_tag_tv_list_first(list);
    while !li.is_null() {
        let itemdict = nvim_tag_listitem_get_dict(li);
        if itemdict.is_null() {
            li = nvim_tag_tv_list_item_next(list, li);
            continue;
        }

        // Parse 'from' for the cursor position
        let di = nvim_tag_tv_dict_find_item(itemdict, c"from".as_ptr(), -1);
        if di.is_null() {
            li = nvim_tag_tv_list_item_next(list, li);
            continue;
        }

        let tv = nvim_tag_dictitem_tv(di);
        let mut lnum: i32 = 0;
        let mut col: i32 = 0;
        let mut coladd: i32 = 0;
        let mut fnum: c_int = 0;

        if nvim_tag_list2fpos(
            tv,
            &raw mut lnum,
            &raw mut col,
            &raw mut coladd,
            &raw mut fnum,
        ) != OK
        {
            li = nvim_tag_tv_list_item_next(list, li);
            continue;
        }

        let tagname = tv_dict_get_string(itemdict, c"tagname".as_ptr(), true);
        if tagname.is_null() {
            li = nvim_tag_tv_list_item_next(list, li);
            continue;
        }

        if col > 0 {
            col -= 1;
        }

        let bufnr = tv_dict_get_number(itemdict, c"bufnr".as_ptr()) as c_int;
        let matchnr = tv_dict_get_number(itemdict, c"matchnr".as_ptr()) as c_int - 1;
        let user_data = tv_dict_get_string(itemdict, c"user_data".as_ptr(), true);

        rs_tagstack_push(wp, tagname, bufnr, matchnr, lnum, col, fnum, user_data);

        li = nvim_tag_tv_list_item_next(list, li);
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
