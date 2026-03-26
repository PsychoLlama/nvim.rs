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

use std::ffi::{c_char, c_int, c_void, CStr};
use std::ptr;

use libc::strcmp;
use nvim_memory::{xmemdupz, xstrdup};

// =============================================================================
// Error message string constants (replaces C static error strings)
// =============================================================================

const E_TAG_STACK_EMPTY: &CStr = c"E73: Tag stack empty";
const E_AT_BOTTOM_OF_TAG_STACK: &CStr = c"E555: At bottom of tag stack";
const E_AT_TOP_OF_TAG_STACK: &CStr = c"E556: At top of tag stack";
const E_CANNOT_MODIFY_TAG_STACK_WITHIN_TAGFUNC: &CStr =
    c"E986: Cannot modify the tag stack within tagfunc";
const E_TAG_NOT_FOUND_STR: &CStr = c"E426: Tag not found: %s";
const E_WINDOW_UNEXPECTEDLY_CLOSE_WHILE_SEARCHING_FOR_TAGS: &CStr =
    c"E1299: Window unexpectedly closed while searching for tags";

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

type WinHandle = *const c_void;

extern "C" {
    fn nvim_get_postponed_split() -> c_int;
    fn nvim_set_postponed_split(val: c_int);
    fn nvim_get_g_do_tagpreview() -> c_int;
    fn nvim_set_g_do_tagpreview(val: c_int);
    fn nvim_check_can_set_curbuf_forceit(forceit: c_int) -> bool;

    // Phase 11 accessors
    fn nvim_tag_get_p_tgst() -> bool;
    fn nvim_tag_get_curbuf_fnum() -> c_int;
    fn nvim_tag_get_got_int() -> bool;
    fn nvim_tag_get_tfu_in_use() -> bool;
    fn emsg(s: *const c_char) -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn smsg(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    fn gettext(msgid: *const c_char) -> *const c_char;
    fn nvim_tag_buflist_findnr_ffname(fnum: c_int) -> *mut c_char;
    fn nvim_tag_buflist_getfile_with_result(
        fnum: c_int,
        lnum: i32,
        flags: c_int,
        forceit: c_int,
    ) -> c_int;
    fn nvim_tag_tagstack_changed(saved_tagstack: *mut c_void) -> bool;
    fn nvim_tag_get_tagstack_ptr() -> *mut c_void;
    fn nvim_tag_save_cursor_in_entry(tagstack: *mut c_void, idx: c_int);
    fn nvim_tag_copy_fmark_from_entry(tagstack: *mut c_void, idx: c_int, out_buf: *mut c_void);
    fn nvim_tag_restore_fmark_to_entry(tagstack: *mut c_void, idx: c_int, buf: *const c_void);
    fn nvim_tag_prompt_for_selection() -> c_int;
    fn nvim_tag_set_vim_var_swapcommand(cmd: *const c_char);
    fn nvim_tag_clear_swap_command();
    fn nvim_tag_snprintf_match_msg(
        buf: *mut c_char,
        buf_size: c_int,
        cur_match: c_int,
        num_matches: c_int,
        max_num_matches: c_int,
    );
    fn nvim_tag_append_ic_warning_to_buf(buf: *mut c_char, buf_size: c_int);
    fn nvim_tag_give_warning(msg: *const c_char, ic: bool);
    fn nvim_tag_get_KeyTyped() -> bool;
    fn nvim_taggy_get_fmark(tg: *const c_void) -> *const c_void;
    fn nvim_fmark_get_fnum(fm: *const c_void) -> c_int;
    fn nvim_fmark_get_lnum(fm: *const c_void) -> i32;
    fn nvim_fmark_get_col(fm: *const c_void) -> c_int;
    fn nvim_setpcmark();
    fn nvim_set_cursor_lnum(lnum: i32);
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_curwin_set_curswant(val: bool);
    fn nvim_check_cursor();
    fn nvim_get_fdo_flags() -> u32;
    fn rs_foldOpenCursor();
    fn nvim_tag_set_msg_scroll(val: c_int);
    fn nvim_tag_get_msg_scrolled() -> c_int;
    fn nvim_tag_get_msg_silent() -> c_int;
    fn nvim_tag_ui_has_messages() -> bool;
    fn nvim_tag_ui_flush();
    fn nvim_tag_os_delay(msec: c_int);
    fn msg(msg: *const c_char, hlf: c_int) -> c_int;
    fn nvim_tag_free_nofile_fname();
    fn nvim_tag_nofile_fname_is_null() -> bool;
    fn nvim_get_nofile_fname() -> *const c_char;

    // Tag match cache
    fn nvim_tag_find_tags(
        pat: *mut c_char,
        num_matches: *mut c_int,
        matchesp: *mut *mut *mut c_char,
        flags: c_int,
        mincount: c_int,
        buf_ffname: *mut c_char,
    ) -> c_int;
    fn nvim_tag_free_wild(count: c_int, files: *mut *mut c_char);

    // Tag match name
    fn nvim_get_tagmatchname() -> *const c_char;
    fn nvim_xfree_clear_tagmatchname();
    fn nvim_set_tagmatchname(name: *mut c_char);

    // ptag_entry — use *const for getters, *mut for setters
    fn nvim_get_ptag_entry() -> *mut c_void;

    // Win/stack accessors
    fn nvim_tag_get_curwin() -> *mut c_void;
    fn nvim_win_get_tagstackidx(wp: WinHandle) -> c_int;
    fn nvim_win_get_tagstacklen(wp: WinHandle) -> c_int;
    fn nvim_win_set_tagstackidx(wp: *mut c_void, idx: c_int);

    // Taggy getters (take *const c_void to match commands.rs)
    fn nvim_taggy_get_cur_match(tg: *const c_void) -> c_int;
    fn nvim_taggy_get_cur_fnum(tg: *const c_void) -> c_int;
    fn nvim_taggy_get_tagname(tg: *const c_void) -> *const c_char;
    fn nvim_taggy_get_user_data(tg: *const c_void) -> *const c_char;

    // Taggy setters (take *mut c_void)
    fn nvim_taggy_set_cur_match(tg: *mut c_void, match_idx: c_int);
    fn nvim_taggy_set_cur_fnum(tg: *mut c_void, fnum: c_int);
    fn nvim_taggy_set_user_data(tg: *mut c_void, data: *mut c_char);
    fn nvim_taggy_set_tagname(tg: *mut c_void, name: *mut c_char);

    // Stack entry accessor (returns *const to match commands.rs)
    fn nvim_win_get_tagstack_entry(wp: WinHandle, idx: c_int) -> *const c_void;
    fn nvim_win_set_tagstacklen(wp: *mut c_void, len: c_int);
    fn nvim_tag_get_curbuf_ffname() -> *mut c_char;
    fn xfree(ptr: *mut c_void);
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
// Constants
// =============================================================================

/// NOTAGFILE return value from `jumpto_tag`
const NOTAGFILE: c_int = 99;

/// OK return value
const OK: c_int = 1;

/// FAIL return value
const FAIL: c_int = 0;

/// `MT_IC_OFF` flag in match priority byte
const MT_IC_OFF: u8 = 4;

/// TAGSTACKSIZE constant
const TAGSTACKSIZE: c_int = 20;

/// GETF_SETMARK flag for buflist_getfile
const GETF_SETMARK: c_int = 0x01;

/// kOptFdoFlagTag flag value (verified by _Static_assert in tag_shim.c)
const FDO_FLAG_TAG: u32 = 0x80;

/// HLF_W highlight attribute (verified by _Static_assert in tag_shim.c)
const HLF_W: c_int = 26;

// =============================================================================
// Main do_tag() implementation
// =============================================================================

/// Cached tag match state (replaces C static locals in `do_tag()`).
///
/// These are module-level statics that persist across calls, matching
/// the behavior of the C static locals.
static mut NUM_MATCHES: c_int = 0;
static mut MAX_NUM_MATCHES: c_int = 0;
static mut MATCHES: *mut *mut c_char = ptr::null_mut();
static mut FLAGS: c_int = 0;

/// Main tag command dispatcher.
///
/// Handles `:tag`, `:pop`, `:tnext`, `:tprev`, `:tfirst`, `:tlast`,
/// `:tselect`, `:tjump`, `:ltag`, and tag preview commands.
///
/// # Safety
///
/// Calls many C accessor functions and manipulates global state.
///
/// # Panics
///
/// Panics if `tag` is null.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_do_tag(
    tag: *mut c_char,
    typ: c_int,
    count: c_int,
    forceit: c_int,
    verbose: bool,
) {
    // DT_FREE: free cached matches and return
    if typ == cmd_type::DT_FREE {
        nvim_tag_free_wild(NUM_MATCHES, MATCHES);
        NUM_MATCHES = 0;
        MATCHES = ptr::null_mut();
        return;
    }

    // Disallow recursive tagfunc calls
    if nvim_tag_get_tfu_in_use() {
        emsg(gettext(E_CANNOT_MODIFY_TAG_STACK_WITHIN_TAGFUNC.as_ptr()));
        return;
    }

    if nvim_get_postponed_split() == 0 && !nvim_check_can_set_curbuf_forceit(forceit) {
        return;
    }

    let mut cur_type = typ;
    let is_help = cur_type == cmd_type::DT_HELP;
    if is_help {
        cur_type = cmd_type::DT_TAG;
    }
    let no_regexp = is_help;
    let use_tfu = !is_help;

    let prev_num_matches = NUM_MATCHES;
    nvim_tag_free_nofile_fname();

    // Determine stack/preview usage
    assert!(!tag.is_null());
    let tag_is_empty = *tag == 0;
    let p_tgst = nvim_tag_get_p_tgst();
    let g_do_tagpreview = nvim_get_g_do_tagpreview();
    let curwin = nvim_tag_get_curwin();
    let use_tagstack;
    let mut new_tag = false;
    let mut save_pos = false;
    let mut cur_match: c_int = 0;
    let mut cur_fnum = nvim_tag_get_curbuf_fnum();
    let mut skip_msg = false;
    let mut error_cur_match: c_int = 0;
    let mut tofree: *mut c_char = ptr::null_mut();

    // Opaque buffer for saved fmark (max 64 bytes per _Static_assert)
    let mut saved_fmark_buf = [0u8; 64];

    let mut tagstackidx = nvim_win_get_tagstackidx(curwin);
    let mut tagstacklen = nvim_win_get_tagstacklen(curwin);
    let oldtagstackidx = tagstackidx;
    let mut prevtagstackidx = tagstackidx;
    let tagstack_ptr = nvim_tag_get_tagstack_ptr();

    let ptag_entry = nvim_get_ptag_entry();

    if !p_tgst && !tag_is_empty {
        // Don't add a tag to the tagstack if 'tagstack' has been reset.
        use_tagstack = false;
        new_tag = true;
        if g_do_tagpreview != 0 {
            crate::stack::rs_tagstack_clear_entry(ptag_entry);
            nvim_taggy_set_tagname(ptag_entry, xstrdup(tag));
        }
    } else {
        use_tagstack = g_do_tagpreview == 0;

        if !tag_is_empty
            && (cur_type == cmd_type::DT_TAG
                || cur_type == cmd_type::DT_SELECT
                || cur_type == cmd_type::DT_JUMP
                || cur_type == cmd_type::DT_LTAG)
        {
            // New pattern, add to tag stack
            if g_do_tagpreview != 0 {
                let ptag_name = nvim_taggy_get_tagname(ptag_entry);
                if !ptag_name.is_null() && strcmp(ptag_name, tag) == 0 {
                    // Same tag: keep current match
                    cur_match = nvim_taggy_get_cur_match(ptag_entry);
                    cur_fnum = nvim_taggy_get_cur_fnum(ptag_entry);
                } else {
                    crate::stack::rs_tagstack_clear_entry(ptag_entry);
                    nvim_taggy_set_tagname(ptag_entry, xstrdup(tag));
                }
            } else {
                // Delete entries above current position
                while tagstackidx < tagstacklen {
                    tagstacklen -= 1;
                    let entry = nvim_win_get_tagstack_entry(curwin, tagstacklen);
                    crate::stack::rs_tagstack_clear_entry(entry);
                }

                // If stack is full, remove oldest entry
                tagstacklen += 1;
                if tagstacklen > TAGSTACKSIZE {
                    tagstacklen = TAGSTACKSIZE;
                    let oldest = nvim_win_get_tagstack_entry(curwin, 0);
                    crate::stack::rs_tagstack_clear_entry(oldest);
                    for i in 1..tagstacklen {
                        let dest = nvim_win_get_tagstack_entry(curwin, i - 1);
                        let src = nvim_win_get_tagstack_entry(curwin, i);
                        crate::stack::rs_tagstack_copy_entry(dest, src);
                    }
                    tagstackidx -= 1;
                    let entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
                    nvim_taggy_set_user_data(entry.cast_mut(), ptr::null_mut());
                }

                let entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
                nvim_taggy_set_tagname(entry.cast_mut(), xstrdup(tag));
                nvim_win_set_tagstacklen(curwin, tagstacklen);
                save_pos = true;
            }
            new_tag = true;
        } else {
            // No new tag — use existing stack
            let stack_empty = if g_do_tagpreview != 0 {
                nvim_taggy_get_tagname(ptag_entry).is_null()
            } else {
                tagstacklen == 0
            };
            if stack_empty {
                emsg(gettext(E_TAG_STACK_EMPTY.as_ptr()));
                do_tag_cleanup(use_tagstack, tagstackidx, tofree);
                return;
            }

            if cur_type == cmd_type::DT_POP {
                // Go to older position
                tagstackidx -= count;
                if tagstackidx < 0 {
                    emsg(gettext(E_AT_BOTTOM_OF_TAG_STACK.as_ptr()));
                    if tagstackidx + count == 0 {
                        tagstackidx = 0;
                        do_tag_cleanup(use_tagstack, tagstackidx, tofree);
                        return;
                    }
                    tagstackidx = 0;
                }

                // Inline Rust port of nvim_tag_do_pop_jump (Phase 1)
                if tagstackidx >= tagstacklen {
                    // count == 0 case
                    emsg(gettext(E_AT_TOP_OF_TAG_STACK.as_ptr()));
                    do_tag_cleanup(use_tagstack, tagstackidx, tofree);
                    return;
                }
                let old_key_typed = nvim_tag_get_KeyTyped();
                // Read the fmark fields at tagstackidx before any autocommands can
                // fire (analogous to the C code's fmark_T saved_fmark = ... copy).
                let entry_for_pop = nvim_win_get_tagstack_entry(curwin, tagstackidx);
                let pop_fmark = nvim_taggy_get_fmark(entry_for_pop);
                let pop_bufnum = nvim_fmark_get_fnum(pop_fmark);
                let pop_linenum = nvim_fmark_get_lnum(pop_fmark);
                let pop_colnum = nvim_fmark_get_col(pop_fmark);
                if pop_bufnum == nvim_tag_get_curbuf_fnum() {
                    nvim_setpcmark();
                } else if nvim_tag_buflist_getfile_with_result(
                    pop_bufnum,
                    pop_linenum,
                    GETF_SETMARK,
                    forceit,
                ) == FAIL
                {
                    tagstackidx = oldtagstackidx;
                    do_tag_cleanup(use_tagstack, tagstackidx, tofree);
                    return;
                }
                nvim_set_cursor_lnum(pop_linenum);
                nvim_set_cursor_col(pop_colnum);
                nvim_curwin_set_curswant(true);
                nvim_check_cursor();
                if (nvim_get_fdo_flags() & FDO_FLAG_TAG) != 0 && old_key_typed {
                    rs_foldOpenCursor();
                }

                // Free old matches
                nvim_tag_free_wild(NUM_MATCHES, MATCHES);
                NUM_MATCHES = 0;
                MATCHES = ptr::null_mut();
                crate::rs_tag_freematch();
                do_tag_cleanup(use_tagstack, tagstackidx, tofree);
                return;
            }

            if cur_type == cmd_type::DT_TAG || cur_type == cmd_type::DT_LTAG {
                if g_do_tagpreview != 0 {
                    cur_match = nvim_taggy_get_cur_match(ptag_entry);
                    cur_fnum = nvim_taggy_get_cur_fnum(ptag_entry);
                } else {
                    // Go to newer pattern
                    save_pos = true;
                    tagstackidx += count - 1;
                    if tagstackidx >= tagstacklen {
                        tagstackidx = tagstacklen - 1;
                        emsg(gettext(E_AT_TOP_OF_TAG_STACK.as_ptr()));
                        save_pos = false;
                    } else if tagstackidx < 0 {
                        emsg(gettext(E_AT_BOTTOM_OF_TAG_STACK.as_ptr()));
                        tagstackidx = 0;
                        do_tag_cleanup(use_tagstack, tagstackidx, tofree);
                        return;
                    }
                    let entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
                    cur_match = nvim_taggy_get_cur_match(entry);
                    cur_fnum = nvim_taggy_get_cur_fnum(entry);
                }
                new_tag = true;
            } else {
                // Navigation: DT_NEXT, DT_PREV, DT_FIRST, DT_LAST, DT_SELECT, DT_JUMP
                prevtagstackidx = tagstackidx;

                if g_do_tagpreview != 0 {
                    cur_match = nvim_taggy_get_cur_match(ptag_entry);
                    cur_fnum = nvim_taggy_get_cur_fnum(ptag_entry);
                } else {
                    tagstackidx -= 1;
                    if tagstackidx < 0 {
                        tagstackidx = 0;
                    }
                    let entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
                    cur_match = nvim_taggy_get_cur_match(entry);
                    cur_fnum = nvim_taggy_get_cur_fnum(entry);
                }

                match cur_type {
                    cmd_type::DT_FIRST => cur_match = count - 1,
                    cmd_type::DT_SELECT | cmd_type::DT_JUMP | cmd_type::DT_LAST => {
                        cur_match = MAXCOL - 1;
                    }
                    cmd_type::DT_NEXT => cur_match += count,
                    cmd_type::DT_PREV => cur_match -= count,
                    _ => {}
                }

                if cur_match == MAXCOL {
                    cur_match = MAXCOL - 1;
                } else if cur_match < 0 {
                    emsg(c"E425: Cannot go before first matching tag".as_ptr());
                    skip_msg = true;
                    cur_match = 0;
                    cur_fnum = nvim_tag_get_curbuf_fnum();
                }
            }
        }

        // Save/update state in preview or tagstack
        if g_do_tagpreview != 0 {
            if cur_type != cmd_type::DT_SELECT && cur_type != cmd_type::DT_JUMP {
                nvim_taggy_set_cur_match(ptag_entry, cur_match);
                nvim_taggy_set_cur_fnum(ptag_entry, cur_fnum);
            }
        } else {
            // Save the fmark before modifying the entry
            let entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
            nvim_tag_copy_fmark_from_entry(
                tagstack_ptr,
                tagstackidx,
                saved_fmark_buf.as_mut_ptr().cast(),
            );
            if save_pos {
                nvim_tag_save_cursor_in_entry(tagstack_ptr, tagstackidx);
            }

            nvim_win_set_tagstackidx(curwin, tagstackidx);
            if cur_type != cmd_type::DT_SELECT && cur_type != cmd_type::DT_JUMP {
                nvim_taggy_set_cur_match(entry.cast_mut(), cur_match);
                nvim_taggy_set_cur_fnum(entry.cast_mut(), cur_fnum);
            }
        }
    }

    // Get buf_ffname for match priority
    let mut buf_ffname = nvim_tag_get_curbuf_ffname();
    if cur_fnum != nvim_tag_get_curbuf_fnum() {
        let found = nvim_tag_buflist_findnr_ffname(cur_fnum);
        if !found.is_null() {
            buf_ffname = found;
        }
    }

    // =========================================================================
    // Search loop — repeat searching when a file is not found
    // =========================================================================
    loop {
        let name: *mut c_char;

        // Get the tag name to search for
        if use_tagstack {
            let entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
            let tname = nvim_taggy_get_tagname(entry);
            name = xstrdup(tname);
            xfree(tofree.cast::<c_void>());
            tofree = name;
        } else if g_do_tagpreview != 0 {
            name = nvim_taggy_get_tagname(ptag_entry).cast_mut();
        } else {
            name = tag;
        }

        let tagmatchname = nvim_get_tagmatchname();
        let other_name = tagmatchname.is_null() || strcmp(tagmatchname, name) != 0;

        if new_tag || (cur_match >= NUM_MATCHES && MAX_NUM_MATCHES != MAXCOL) || other_name {
            if other_name {
                nvim_xfree_clear_tagmatchname();
                nvim_set_tagmatchname(xstrdup(name));
            }

            if cur_type == cmd_type::DT_SELECT
                || cur_type == cmd_type::DT_JUMP
                || cur_type == cmd_type::DT_LTAG
            {
                cur_match = MAXCOL - 1;
            }
            MAX_NUM_MATCHES = if cur_type == cmd_type::DT_TAG {
                MAXCOL
            } else {
                cur_match + 1
            };

            // Build flags for find_tags
            let mut search_name = name;
            FLAGS = if !no_regexp && *name == b'/' as c_char {
                search_name = name.add(1);
                crate::search::find_tags_flags::TAG_REGEXP
            } else {
                crate::search::find_tags_flags::TAG_NOIC
            };
            if verbose {
                FLAGS |= crate::search::find_tags_flags::TAG_VERBOSE;
            }
            if !use_tfu {
                FLAGS |= crate::search::find_tags_flags::TAG_NO_TAGFUNC;
            }

            let mut new_num_matches: c_int = 0;
            let mut new_matches: *mut *mut c_char = ptr::null_mut();

            if nvim_tag_find_tags(
                search_name,
                &raw mut new_num_matches,
                &raw mut new_matches,
                FLAGS,
                MAX_NUM_MATCHES,
                buf_ffname,
            ) == OK
                && new_num_matches < MAX_NUM_MATCHES
            {
                MAX_NUM_MATCHES = MAXCOL;
            }

            // Check if tagstack pointer changed (window closed)
            if nvim_tag_tagstack_changed(tagstack_ptr) {
                emsg(gettext(
                    E_WINDOW_UNEXPECTEDLY_CLOSE_WHILE_SEARCHING_FOR_TAGS.as_ptr(),
                ));
                nvim_tag_free_wild(new_num_matches, new_matches);
                break;
            }

            // Reorder matches to preserve order from previous search
            if !new_tag && !other_name {
                reorder_matches(NUM_MATCHES, MATCHES, new_num_matches, new_matches);
            }

            nvim_tag_free_wild(NUM_MATCHES, MATCHES);
            NUM_MATCHES = new_num_matches;
            MATCHES = new_matches;
        }

        if NUM_MATCHES <= 0 {
            if verbose {
                semsg(gettext(E_TAG_NOT_FOUND_STR.as_ptr()), name);
            }
            nvim_set_g_do_tagpreview(0);
        } else {
            let mut ask_for_selection = false;

            if cur_type == cmd_type::DT_TAG && !tag_is_empty {
                cur_match = if count > 0 { count - 1 } else { 0 };
            } else if cur_type == cmd_type::DT_SELECT
                || (cur_type == cmd_type::DT_JUMP && NUM_MATCHES > 1)
            {
                crate::commands::rs_print_tag_list(new_tag, use_tagstack, NUM_MATCHES, MATCHES);
                ask_for_selection = true;
            } else if cur_type == cmd_type::DT_LTAG {
                if crate::commands::rs_add_llist_tags(tag, NUM_MATCHES, MATCHES) == FAIL {
                    do_tag_cleanup(use_tagstack, tagstackidx, tofree);
                    return;
                }
                cur_match = 0;
            }

            if ask_for_selection {
                let selection = nvim_tag_prompt_for_selection();
                if selection <= 0 || selection > NUM_MATCHES || nvim_tag_get_got_int() {
                    // Cancelled — restore state
                    if use_tagstack {
                        nvim_tag_restore_fmark_to_entry(
                            tagstack_ptr,
                            tagstackidx,
                            saved_fmark_buf.as_ptr().cast(),
                        );
                        tagstackidx = prevtagstackidx;
                    }
                    break;
                }
                cur_match = selection - 1;
            }

            if cur_match >= NUM_MATCHES {
                if (cur_type == cmd_type::DT_NEXT || cur_type == cmd_type::DT_FIRST)
                    && nvim_tag_nofile_fname_is_null()
                {
                    if NUM_MATCHES == 1 {
                        emsg(c"E427: There is only one matching tag".as_ptr());
                    } else {
                        emsg(c"E428: Cannot go beyond last matching tag".as_ptr());
                    }
                    skip_msg = true;
                }
                cur_match = NUM_MATCHES - 1;
            }

            // Update tagstack with current match
            if use_tagstack {
                let entry = nvim_win_get_tagstack_entry(curwin, tagstackidx);
                let entry_mut = entry.cast_mut();
                nvim_taggy_set_cur_match(entry_mut, cur_match);
                nvim_taggy_set_cur_fnum(entry_mut, cur_fnum);

                // Store user_data from tagfunc
                if use_tfu {
                    let match_ptr = *MATCHES.offset(cur_match as isize);
                    let mut tagp2 = crate::parse::TagPtrs::default();
                    if crate::parse::rs_parse_match(match_ptr, &raw mut tagp2) == OK
                        && !tagp2.user_data.is_null()
                    {
                        let ud_len = tagp2.user_data_end.offset_from(tagp2.user_data) as usize;
                        // Free existing user_data
                        let old_ud = nvim_taggy_get_user_data(entry);
                        if !old_ud.is_null() {
                            xfree(old_ud.cast_mut().cast());
                        }
                        nvim_taggy_set_user_data(
                            entry_mut,
                            xmemdupz(tagp2.user_data.cast(), ud_len).cast(),
                        );
                    }
                }

                tagstackidx += 1;
            } else if g_do_tagpreview != 0 {
                nvim_taggy_set_cur_match(ptag_entry, cur_match);
                nvim_taggy_set_cur_fnum(ptag_entry, cur_fnum);
            }

            // Report previous file-not-found
            if !nvim_tag_nofile_fname_is_null() && error_cur_match != cur_match {
                smsg(
                    0,
                    c"File \"%s\" does not exist".as_ptr(),
                    nvim_get_nofile_fname(),
                );
            }

            // Show "tag X of Y" message
            let match_ptr = *MATCHES.offset(cur_match as isize);
            let ic = (*match_ptr as u8 & MT_IC_OFF) != 0;
            if cur_type != cmd_type::DT_TAG
                && cur_type != cmd_type::DT_SELECT
                && cur_type != cmd_type::DT_JUMP
                && (NUM_MATCHES > 1 || ic)
                && !skip_msg
            {
                // Inline Rust port of nvim_tag_show_match_msg (Phase 1)
                let mut msg_buf = [0u8; 256];
                nvim_tag_snprintf_match_msg(
                    msg_buf.as_mut_ptr().cast(),
                    msg_buf.len() as c_int,
                    cur_match,
                    NUM_MATCHES,
                    MAX_NUM_MATCHES,
                );
                if ic {
                    nvim_tag_append_ic_warning_to_buf(
                        msg_buf.as_mut_ptr().cast(),
                        msg_buf.len() as c_int,
                    );
                }
                let msg_str: *const c_char = msg_buf.as_ptr().cast();
                if (NUM_MATCHES > prev_num_matches || new_tag) && NUM_MATCHES > 1 {
                    msg(msg_str, if ic { HLF_W } else { 0 });
                    nvim_tag_set_msg_scroll(1);
                } else {
                    nvim_tag_give_warning(msg_str, ic);
                }
                if ic
                    && nvim_tag_get_msg_scrolled() == 0
                    && nvim_tag_get_msg_silent() == 0
                    && !nvim_tag_ui_has_messages()
                {
                    nvim_tag_ui_flush();
                    nvim_tag_os_delay(1007);
                }
            }

            // Set VV_SWAPCOMMAND and jump (inline port of nvim_tag_set_swap_command)
            let mut swap_cmd_buf = [0u8; 4096 + 8]; // name up to MAXPATHL + ":ta \r\0"
            {
                extern "C" {
                    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
                }
                snprintf(
                    swap_cmd_buf.as_mut_ptr().cast(),
                    swap_cmd_buf.len(),
                    c":ta %s\r".as_ptr(),
                    name,
                );
            }
            nvim_tag_set_vim_var_swapcommand(swap_cmd_buf.as_ptr().cast());
            let jump_result =
                crate::jump::rs_jumpto_tag(*MATCHES.offset(cur_match as isize), forceit, true);
            nvim_tag_clear_swap_command();

            if jump_result == NOTAGFILE {
                // File not found: try next match
                if rs_should_retry_match(cur_type, cur_match, NUM_MATCHES, MAX_NUM_MATCHES) {
                    error_cur_match = cur_match;
                    if use_tagstack {
                        tagstackidx -= 1;
                    }
                    if cur_type == cmd_type::DT_PREV {
                        cur_match -= 1;
                    } else {
                        cur_type = cmd_type::DT_NEXT;
                        cur_match += 1;
                    }
                    continue;
                }
                semsg(
                    c"E429: File \"%s\" does not exist".as_ptr(),
                    nvim_get_nofile_fname(),
                );
            } else {
                // May have jumped to another window
                if use_tagstack && tagstackidx > nvim_win_get_tagstacklen(nvim_tag_get_curwin()) {
                    tagstackidx = nvim_win_get_tagstackidx(nvim_tag_get_curwin());
                }
            }
        }
        break;
    }

    do_tag_cleanup(use_tagstack, tagstackidx, tofree);
}

/// Cleanup at end of `do_tag`: save tagstack index and reset globals.
unsafe fn do_tag_cleanup(use_tagstack: bool, tagstackidx: c_int, tofree: *mut c_char) {
    let curwin = nvim_tag_get_curwin();
    if use_tagstack && tagstackidx <= nvim_win_get_tagstacklen(curwin) {
        nvim_win_set_tagstackidx(curwin, tagstackidx);
    }
    nvim_set_postponed_split(0);
    nvim_set_g_do_tagpreview(0);
    xfree(tofree.cast::<c_void>());
}

/// Reorder new matches to preserve the order of old matches at the front.
///
/// For each old match, find it in the new list and move it to the front.
unsafe fn reorder_matches(
    old_count: c_int,
    old_matches: *mut *mut c_char,
    new_count: c_int,
    new_matches: *mut *mut c_char,
) {
    let mut idx: c_int = 0;
    let mut tagp = crate::parse::TagPtrs::default();
    let mut tagp2 = crate::parse::TagPtrs::default();

    for j in 0..old_count {
        crate::parse::rs_parse_match(*old_matches.offset(j as isize), &raw mut tagp);
        for i in idx..new_count {
            crate::parse::rs_parse_match(*new_matches.offset(i as isize), &raw mut tagp2);
            if strcmp(tagp.tagname, tagp2.tagname) == 0 {
                let p = *new_matches.offset(i as isize);
                // Shift entries right to make room
                let mut k = i;
                while k > idx {
                    *new_matches.offset(k as isize) = *new_matches.offset((k - 1) as isize);
                    k -= 1;
                }
                *new_matches.offset(idx as isize) = p;
                idx += 1;
                break;
            }
        }
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
