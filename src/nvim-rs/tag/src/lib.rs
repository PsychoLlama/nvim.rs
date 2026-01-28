//! Tag navigation functions for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations of tag stack and navigation functions.

use std::ffi::{c_char, c_int, c_void};

// Sub-modules
pub mod binary;
pub mod commands;
pub mod files;
pub mod jump;
pub mod matches;
pub mod parse;
pub mod pattern;
pub mod preview;
pub mod search;
pub mod select;
pub mod stack;

// Re-export parse types and functions
pub use binary::{BinarySearchResult, BinarySearchState, FileOffset, TagFileSortInfo};
pub use commands::{JumpTarget, PreviewTagState};
pub use files::TagFileIterator;
pub use jump::JumpTagState;
pub use matches::MatchStorage;
pub use parse::TagPtrs;
pub use pattern::{RegMatch, TagPattern};
pub use preview::{
    GlobalPreviewState, PreviewMode, PreviewRequest, PreviewResult, PreviewTagEntry,
    PreviewWindowInfo, PreviewWindowState,
};
pub use search::{
    FindTagsContext, FindTagsMatchArgs, FindTagsStateCore, TagMatchStatus, TagSearchInfo,
    TagSearchState, TagsReadStatus,
};
pub use select::{SelectionAction, SelectionResult, TagMatchDisplayInfo, TagSelectionState};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of tags in the tag stack.
pub const TAGSTACKSIZE: c_int = 20;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `win_T` (window)
type WinHandle = *const c_void;
/// Opaque handle to `taggy_T` (tag stack entry)
type TaggyHandle = *const c_void;
/// Opaque handle to `fmark_T` (file mark)
type FmarkHandle = *const c_void;
/// Opaque handle to `findtags_state_T` (tag search state)
type FindTagsStateHandle = *const c_void;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Window tag stack accessors
    fn nvim_win_get_tagstacklen(wp: WinHandle) -> c_int;
    fn nvim_win_get_tagstackidx(wp: WinHandle) -> c_int;
    fn nvim_win_get_tagstack_entry(wp: WinHandle, idx: c_int) -> TaggyHandle;

    // Taggy accessors
    fn nvim_taggy_get_tagname(tg: TaggyHandle) -> *const c_char;
    fn nvim_taggy_get_cur_match(tg: TaggyHandle) -> c_int;
    fn nvim_taggy_get_cur_fnum(tg: TaggyHandle) -> c_int;
    fn nvim_taggy_get_fmark(tg: TaggyHandle) -> FmarkHandle;
    fn nvim_taggy_get_user_data(tg: TaggyHandle) -> *const c_char;

    // Fmark accessors
    fn nvim_fmark_get_lnum(fm: FmarkHandle) -> LinenrT;
    fn nvim_fmark_get_col(fm: FmarkHandle) -> c_int;
    fn nvim_fmark_get_fnum(fm: FmarkHandle) -> c_int;

    // Findtags state accessors
    fn nvim_findtags_get_state(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_get_match_count(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_get_help_only(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_get_linear(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_get_tag_file_sorted(st: FindTagsStateHandle) -> c_int;
}

// =============================================================================
// Phase 1: Tag Stack Query Functions
// =============================================================================

/// Returns true if the tag stack for the specified window is empty.
///
/// A stack is empty if the pointer is null or `w_tagstacklen <= 0`.
///
/// # Safety
///
/// - `wp` may be null (in which case it's considered empty)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_stack_empty(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }
    nvim_win_get_tagstacklen(wp) <= 0
}

/// Returns the number of entries in the tag stack.
///
/// Returns 0 if the window pointer is null.
///
/// # Safety
///
/// - `wp` may be null (in which case 0 is returned)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_stack_len(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_tagstacklen(wp)
}

/// Returns the current index in the tag stack.
///
/// The index points to the entry below the active one.
/// Returns 0 if the window pointer is null.
///
/// # Safety
///
/// - `wp` may be null (in which case 0 is returned)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_stack_idx(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_tagstackidx(wp)
}

/// Returns true if we are at the bottom of the tag stack (cannot pop).
///
/// # Safety
///
/// - `wp` may be null (in which case it returns true)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_at_bottom(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }
    nvim_win_get_tagstackidx(wp) <= 0
}

/// Returns true if we are at the top of the tag stack.
///
/// # Safety
///
/// - `wp` may be null (in which case it returns true)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_at_top(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }
    nvim_win_get_tagstackidx(wp) >= nvim_win_get_tagstacklen(wp)
}

/// Returns the current match index from a tag stack entry.
///
/// # Safety
///
/// - `tg` must be a valid pointer to a `taggy_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_taggy_get_cur_match(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    nvim_taggy_get_cur_match(tg)
}

/// Returns the file number from a tag stack entry.
///
/// # Safety
///
/// - `tg` must be a valid pointer to a `taggy_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_taggy_get_cur_fnum(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    nvim_taggy_get_cur_fnum(tg)
}

/// Returns true if the tag stack entry has a tag name.
///
/// # Safety
///
/// - `tg` may be null (in which case it returns false)
/// - If non-null, `tg` must be a valid pointer to a `taggy_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_taggy_has_name(tg: TaggyHandle) -> bool {
    if tg.is_null() {
        return false;
    }
    !nvim_taggy_get_tagname(tg).is_null()
}

// =============================================================================
// Phase 2: Match Priority Constants & Helpers
// =============================================================================

/// Tag match type constants.
///
/// The matching tags are stored in hash tables based on priority.
/// These values determine the priority order for displaying matches.
pub mod match_type {
    use std::ffi::c_int;

    /// Static match in current file (highest priority).
    pub const MT_ST_CUR: c_int = 0;
    /// Global match in current file.
    pub const MT_GL_CUR: c_int = 1;
    /// Global match in other file.
    pub const MT_GL_OTH: c_int = 2;
    /// Static match in other file.
    pub const MT_ST_OTH: c_int = 3;
    /// Add for case-insensitive match.
    pub const MT_IC_OFF: c_int = 4;
    /// Add for regexp match.
    pub const MT_RE_OFF: c_int = 8;
    /// Mask for printing priority (0-7).
    pub const MT_MASK: c_int = 7;
    /// Total number of match type buckets.
    pub const MT_COUNT: c_int = 16;
}

/// Short names for match types (for display).
/// Order: "FSC", "F C", "F  ", "FS ", " SC", "  C", "   ", " S "
/// F = current File, S = Static, C = Case-sensitive match
static MT_NAMES: [&std::ffi::CStr; 8] = [
    c"FSC", // MT_ST_CUR: File, Static, Case
    c"F C", // MT_GL_CUR: File, Case
    c"F  ", // MT_GL_OTH: File only
    c"FS ", // MT_ST_OTH: File, Static
    c" SC", // MT_ST_CUR + MT_IC_OFF: Static, Case (no file)
    c"  C", // MT_GL_CUR + MT_IC_OFF: Case only
    c"   ", // MT_GL_OTH + MT_IC_OFF: nothing
    c" S ", // MT_ST_OTH + MT_IC_OFF: Static only
];

/// Returns a pointer to the name string for the given match type.
///
/// The returned string is one of: "FSC", "F C", "F  ", "FS ", " SC", "  C", "   ", " S "
/// where F = current File, S = Static, C = Case-sensitive match.
///
/// Returns an empty string for invalid match types.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_tag_match_type_name(mt: c_int) -> *const c_char {
    let idx = (mt & match_type::MT_MASK) as usize;
    if idx < MT_NAMES.len() {
        MT_NAMES[idx].as_ptr()
    } else {
        c"   ".as_ptr()
    }
}

/// Returns true if the match type indicates a static (file-local) match.
///
/// Static matches are `MT_ST_CUR` and `MT_ST_OTH`.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_is_static_match(mt: c_int) -> bool {
    let base = mt & match_type::MT_MASK;
    base == match_type::MT_ST_CUR || base == match_type::MT_ST_OTH
}

/// Returns true if the match type indicates a match in the current file.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_is_current_file(mt: c_int) -> bool {
    let base = mt & match_type::MT_MASK;
    base == match_type::MT_ST_CUR || base == match_type::MT_GL_CUR
}

/// Returns true if the match was case-insensitive.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_is_icase_match(mt: c_int) -> bool {
    (mt & match_type::MT_IC_OFF) != 0
}

/// Returns true if the match was via regexp.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_is_regexp_match(mt: c_int) -> bool {
    (mt & match_type::MT_RE_OFF) != 0
}

/// Returns the base priority of a match type (with IC/RE flags masked off).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_match_priority(mt: c_int) -> c_int {
    mt & match_type::MT_MASK
}

/// Compares two match types for sorting.
///
/// Returns:
/// - negative if `mt1` has higher priority (should come first)
/// - positive if `mt2` has higher priority
/// - 0 if they are equal
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_cmp_match_type(mt1: c_int, mt2: c_int) -> c_int {
    // Lower match type value = higher priority
    mt1 - mt2
}

/// Returns the better (higher priority) of two match types.
///
/// Lower numeric value = higher priority.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_best_match_type(mt1: c_int, mt2: c_int) -> c_int {
    if mt1 <= mt2 {
        mt1
    } else {
        mt2
    }
}

// =============================================================================
// Phase 3: Tag Navigation Helpers
// =============================================================================

/// Returns true if we can pop from the tag stack (go to older position).
///
/// # Safety
///
/// - `wp` may be null (in which case it returns false)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_can_pop(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    nvim_win_get_tagstackidx(wp) > 0
}

/// Returns true if the tag stack has room for more entries.
///
/// # Safety
///
/// - `wp` may be null (in which case it returns false)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_can_push(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    nvim_win_get_tagstacklen(wp) < TAGSTACKSIZE
}

/// Calculates the next match index with wrapping.
///
/// If `cur + 1 >= count`, wraps to 0.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_next_match_idx(cur: c_int, count: c_int) -> c_int {
    if count <= 0 {
        return 0;
    }
    let next = cur + 1;
    if next >= count {
        0
    } else {
        next
    }
}

/// Calculates the previous match index with wrapping.
///
/// If `cur <= 0`, wraps to `count - 1`.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_prev_match_idx(cur: c_int, count: c_int) -> c_int {
    if count <= 0 {
        return 0;
    }
    if cur <= 0 {
        count - 1
    } else {
        cur - 1
    }
}

/// Returns the index of the first match (always 0).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_first_match_idx() -> c_int {
    0
}

/// Returns the index of the last match.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_last_match_idx(count: c_int) -> c_int {
    if count <= 0 {
        0
    } else {
        count - 1
    }
}

/// Clamps an index to a valid range [min, max].
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_clamp_idx(idx: c_int, min: c_int, max: c_int) -> c_int {
    if idx < min {
        min
    } else if idx > max {
        max
    } else {
        idx
    }
}

/// Returns true if the given stack position is valid for the window.
///
/// # Safety
///
/// - `wp` may be null (in which case it returns false)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_stack_pos_valid(wp: WinHandle, idx: c_int) -> bool {
    if wp.is_null() {
        return false;
    }
    idx >= 0 && idx < nvim_win_get_tagstacklen(wp)
}

// =============================================================================
// Phase 4: Tag Entry Accessors & Comparisons
// =============================================================================

/// Compares two tag entries by tag name.
///
/// Returns:
/// - negative if `tg1` name comes before `tg2` name
/// - positive if `tg1` name comes after `tg2` name
/// - 0 if they are equal or either is null
///
/// # Safety
///
/// - Both `tg1` and `tg2` must be valid pointers to `taggy_T` structs or null
#[no_mangle]
pub unsafe extern "C" fn rs_taggy_cmp_name(tg1: TaggyHandle, tg2: TaggyHandle) -> c_int {
    if tg1.is_null() || tg2.is_null() {
        return 0;
    }
    let name1 = nvim_taggy_get_tagname(tg1);
    let name2 = nvim_taggy_get_tagname(tg2);

    if name1.is_null() && name2.is_null() {
        return 0;
    }
    if name1.is_null() {
        return -1;
    }
    if name2.is_null() {
        return 1;
    }

    // Use libc strcmp for comparison
    #[allow(clippy::items_after_statements)]
    extern "C" {
        fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    }
    strcmp(name1, name2)
}

/// Compares two tag entries by file number.
///
/// Returns the difference between the file numbers.
///
/// # Safety
///
/// - Both `tg1` and `tg2` must be valid pointers to `taggy_T` structs or null
#[no_mangle]
pub unsafe extern "C" fn rs_taggy_cmp_fnum(tg1: TaggyHandle, tg2: TaggyHandle) -> c_int {
    if tg1.is_null() || tg2.is_null() {
        return 0;
    }
    nvim_taggy_get_cur_fnum(tg1) - nvim_taggy_get_cur_fnum(tg2)
}

/// Returns true if two tag entries refer to the same tag.
///
/// Two entries are considered the same if they have the same tag name.
///
/// # Safety
///
/// - Both `tg1` and `tg2` must be valid pointers to `taggy_T` structs or null
#[no_mangle]
pub unsafe extern "C" fn rs_taggy_same_tag(tg1: TaggyHandle, tg2: TaggyHandle) -> bool {
    rs_taggy_cmp_name(tg1, tg2) == 0 && rs_taggy_has_name(tg1)
}

/// Returns true if the tag entry's file matches the given buffer number.
///
/// # Safety
///
/// - `tg` must be a valid pointer to a `taggy_T` struct or null
#[no_mangle]
pub unsafe extern "C" fn rs_taggy_in_buffer(tg: TaggyHandle, bufnr: c_int) -> bool {
    if tg.is_null() {
        return false;
    }
    nvim_taggy_get_cur_fnum(tg) == bufnr
}

/// Returns true if the tag entry's fmark has a valid position.
///
/// A position is valid if lnum > 0.
///
/// # Safety
///
/// - `tg` must be a valid pointer to a `taggy_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_fmark_valid(tg: TaggyHandle) -> bool {
    if tg.is_null() {
        return false;
    }
    let fm = nvim_taggy_get_fmark(tg);
    if fm.is_null() {
        return false;
    }
    nvim_fmark_get_lnum(fm) > 0
}

/// Returns the line number from the tag entry's fmark.
///
/// # Safety
///
/// - `tg` must be a valid pointer to a `taggy_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_fmark_lnum(tg: TaggyHandle) -> LinenrT {
    if tg.is_null() {
        return 0;
    }
    let fm = nvim_taggy_get_fmark(tg);
    if fm.is_null() {
        return 0;
    }
    nvim_fmark_get_lnum(fm)
}

/// Returns the column from the tag entry's fmark.
///
/// # Safety
///
/// - `tg` must be a valid pointer to a `taggy_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_fmark_col(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    let fm = nvim_taggy_get_fmark(tg);
    if fm.is_null() {
        return 0;
    }
    nvim_fmark_get_col(fm)
}

/// Returns the file number from the tag entry's fmark.
///
/// # Safety
///
/// - `tg` must be a valid pointer to a `taggy_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_fmark_fnum(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    let fm = nvim_taggy_get_fmark(tg);
    if fm.is_null() {
        return 0;
    }
    nvim_fmark_get_fnum(fm)
}

// =============================================================================
// Phase 5: Search State Queries
// =============================================================================

/// Tag search state constants (from `tagsearch_state_T` enum).
pub mod search_state {
    use std::ffi::c_int;

    /// At start of file.
    pub const TS_START: c_int = 0;
    /// Linear searching forward, till EOF.
    pub const TS_LINEAR: c_int = 1;
    /// Binary searching.
    pub const TS_BINARY: c_int = 2;
    /// Skipping backwards.
    pub const TS_SKIP_BACK: c_int = 3;
    /// Stepping forwards.
    pub const TS_STEP_FORWARD: c_int = 4;
}

/// State name strings for display.
static SEARCH_STATE_NAMES: [&std::ffi::CStr; 5] = [
    c"start",
    c"linear",
    c"binary",
    c"skip_back",
    c"step_forward",
];

/// Returns true if the tag search is doing a linear search.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_search_is_linear(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return false;
    }
    nvim_findtags_get_linear(st)
}

/// Returns true if the tag search is doing a binary search.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_search_is_binary(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return false;
    }
    let state = nvim_findtags_get_state(st);
    state == search_state::TS_BINARY
}

/// Returns true if the tag search has completed.
///
/// Search is done when state is not START, LINEAR, or BINARY.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_search_done(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return true;
    }
    let state = nvim_findtags_get_state(st);
    state == search_state::TS_SKIP_BACK || state == search_state::TS_STEP_FORWARD
}

/// Returns the current match count from the search state.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_search_match_count(st: FindTagsStateHandle) -> c_int {
    if st.is_null() {
        return 0;
    }
    nvim_findtags_get_match_count(st)
}

/// Returns true if the search is for help tags only.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_search_help_only(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return false;
    }
    nvim_findtags_get_help_only(st)
}

/// Returns true if the search has found any matches.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_search_has_matches(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return false;
    }
    nvim_findtags_get_match_count(st) > 0
}

/// Returns true if the current tag file is sorted.
///
/// Sorted values: 0 = unsorted, 1 = sorted, 2 = sorted ignoring case.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_tag_file_sorted(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return false;
    }
    nvim_findtags_get_tag_file_sorted(st) > 0
}

/// Returns a pointer to the name string for the search state.
///
/// # Safety
///
/// - `state` should be a valid `tagsearch_state_T` value
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_tag_search_state_name(state: c_int) -> *const c_char {
    let idx = state as usize;
    if idx < SEARCH_STATE_NAMES.len() {
        SEARCH_STATE_NAMES[idx].as_ptr()
    } else {
        c"unknown".as_ptr()
    }
}

// =============================================================================
// Phase 6: Tag Command Type Constants & Validation
// =============================================================================

/// Tag command type constants (from tag.h `DT_*` enum).
pub mod tag_cmd {
    use std::ffi::c_int;

    /// Jump to newer position or same tag again.
    pub const DT_TAG: c_int = 1;
    /// Jump to older position (pop).
    pub const DT_POP: c_int = 2;
    /// Jump to next match of same tag.
    pub const DT_NEXT: c_int = 3;
    /// Jump to previous match of same tag.
    pub const DT_PREV: c_int = 4;
    /// Jump to first match of same tag.
    pub const DT_FIRST: c_int = 5;
    /// Jump to last match of same tag.
    pub const DT_LAST: c_int = 6;
    /// Jump to selection from list.
    pub const DT_SELECT: c_int = 7;
    /// Like `DT_TAG`, but no wildcards.
    pub const DT_HELP: c_int = 8;
    /// Jump to new tag or selection from list.
    pub const DT_JUMP: c_int = 9;
    /// Tag using location list.
    pub const DT_LTAG: c_int = 11;
    /// Free cached matches.
    pub const DT_FREE: c_int = 99;
}

/// Command name strings for display.
static TAG_CMD_NAMES: [(&std::ffi::CStr, c_int); 11] = [
    (c"tag", tag_cmd::DT_TAG),
    (c"pop", tag_cmd::DT_POP),
    (c"tnext", tag_cmd::DT_NEXT),
    (c"tprevious", tag_cmd::DT_PREV),
    (c"tfirst", tag_cmd::DT_FIRST),
    (c"tlast", tag_cmd::DT_LAST),
    (c"tselect", tag_cmd::DT_SELECT),
    (c"help", tag_cmd::DT_HELP),
    (c"tjump", tag_cmd::DT_JUMP),
    (c"ltag", tag_cmd::DT_LTAG),
    (c"free", tag_cmd::DT_FREE),
];

/// Returns true if the command type involves navigation in the tag stack.
///
/// Navigation commands: `DT_POP`
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_cmd_is_navigation(cmd_type: c_int) -> bool {
    cmd_type == tag_cmd::DT_POP
}

/// Returns true if the command type involves navigating between matches.
///
/// Match navigation commands: `DT_NEXT`, `DT_PREV`, `DT_FIRST`, `DT_LAST`
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_cmd_is_match_nav(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        tag_cmd::DT_NEXT | tag_cmd::DT_PREV | tag_cmd::DT_FIRST | tag_cmd::DT_LAST
    )
}

/// Returns true if the command type uses a tag pattern.
///
/// Pattern commands: `DT_TAG`, `DT_SELECT`, `DT_JUMP`, `DT_LTAG`
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_cmd_uses_pattern(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        tag_cmd::DT_TAG | tag_cmd::DT_SELECT | tag_cmd::DT_JUMP | tag_cmd::DT_LTAG
    )
}

/// Returns true if the command type requires a tag argument.
///
/// Commands that need a tag: `DT_TAG`, `DT_HELP`, `DT_SELECT`, `DT_JUMP`
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_cmd_needs_tag(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        tag_cmd::DT_TAG | tag_cmd::DT_HELP | tag_cmd::DT_SELECT | tag_cmd::DT_JUMP
    )
}

/// Returns the command name string for the given command type.
#[no_mangle]
pub extern "C" fn rs_tag_cmd_name(cmd_type: c_int) -> *const c_char {
    for (name, t) in &TAG_CMD_NAMES {
        if *t == cmd_type {
            return name.as_ptr();
        }
    }
    c"unknown".as_ptr()
}

/// Returns true if the command type value is valid.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_cmd_valid(cmd_type: c_int) -> bool {
    matches!(
        cmd_type,
        tag_cmd::DT_TAG
            | tag_cmd::DT_POP
            | tag_cmd::DT_NEXT
            | tag_cmd::DT_PREV
            | tag_cmd::DT_FIRST
            | tag_cmd::DT_LAST
            | tag_cmd::DT_SELECT
            | tag_cmd::DT_HELP
            | tag_cmd::DT_JUMP
            | tag_cmd::DT_LTAG
            | tag_cmd::DT_FREE
    )
}

/// Returns the direction for the command type.
///
/// Returns:
/// - 1 for forward navigation (`DT_NEXT`, `DT_LAST`)
/// - -1 for backward navigation (`DT_PREV`, `DT_FIRST`, `DT_POP`)
/// - 0 for non-directional commands
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_tag_cmd_direction(cmd_type: c_int) -> c_int {
    match cmd_type {
        tag_cmd::DT_NEXT | tag_cmd::DT_LAST => 1,
        tag_cmd::DT_PREV | tag_cmd::DT_FIRST | tag_cmd::DT_POP => -1,
        _ => 0,
    }
}

/// Converts a `DT_*` command type to an Ex command name.
///
/// This is the same as `rs_tag_cmd_name` - provided for naming consistency.
#[no_mangle]
pub extern "C" fn rs_tag_type_to_cmd(cmd_type: c_int) -> *const c_char {
    rs_tag_cmd_name(cmd_type)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Phase 1 tests

    #[test]
    fn test_null_stack_is_empty() {
        unsafe {
            assert!(rs_tag_stack_empty(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_stack_len() {
        unsafe {
            assert_eq!(rs_tag_stack_len(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_stack_idx() {
        unsafe {
            assert_eq!(rs_tag_stack_idx(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_at_bottom() {
        unsafe {
            assert!(rs_tag_at_bottom(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_at_top() {
        unsafe {
            assert!(rs_tag_at_top(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_taggy_cur_match() {
        unsafe {
            assert_eq!(rs_taggy_get_cur_match(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_taggy_cur_fnum() {
        unsafe {
            assert_eq!(rs_taggy_get_cur_fnum(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_taggy_has_name() {
        unsafe {
            assert!(!rs_taggy_has_name(std::ptr::null()));
        }
    }

    // Phase 2 tests

    #[test]
    fn test_match_type_constants() {
        assert_eq!(match_type::MT_ST_CUR, 0);
        assert_eq!(match_type::MT_GL_CUR, 1);
        assert_eq!(match_type::MT_GL_OTH, 2);
        assert_eq!(match_type::MT_ST_OTH, 3);
        assert_eq!(match_type::MT_IC_OFF, 4);
        assert_eq!(match_type::MT_RE_OFF, 8);
        assert_eq!(match_type::MT_MASK, 7);
        assert_eq!(match_type::MT_COUNT, 16);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_match_type_name() {
        let name = rs_tag_match_type_name(match_type::MT_ST_CUR);
        unsafe {
            assert_eq!(*name, b'F' as c_char);
        }
    }

    #[test]
    fn test_is_static_match() {
        assert!(rs_tag_is_static_match(match_type::MT_ST_CUR));
        assert!(rs_tag_is_static_match(match_type::MT_ST_OTH));
        assert!(!rs_tag_is_static_match(match_type::MT_GL_CUR));
        assert!(!rs_tag_is_static_match(match_type::MT_GL_OTH));
    }

    #[test]
    fn test_is_current_file() {
        assert!(rs_tag_is_current_file(match_type::MT_ST_CUR));
        assert!(rs_tag_is_current_file(match_type::MT_GL_CUR));
        assert!(!rs_tag_is_current_file(match_type::MT_GL_OTH));
        assert!(!rs_tag_is_current_file(match_type::MT_ST_OTH));
    }

    #[test]
    fn test_is_icase_match() {
        assert!(!rs_tag_is_icase_match(match_type::MT_ST_CUR));
        assert!(rs_tag_is_icase_match(
            match_type::MT_ST_CUR + match_type::MT_IC_OFF
        ));
    }

    #[test]
    fn test_is_regexp_match() {
        assert!(!rs_tag_is_regexp_match(match_type::MT_ST_CUR));
        assert!(rs_tag_is_regexp_match(
            match_type::MT_ST_CUR + match_type::MT_RE_OFF
        ));
    }

    #[test]
    fn test_match_priority() {
        assert_eq!(rs_tag_match_priority(match_type::MT_ST_CUR), 0);
        assert_eq!(
            rs_tag_match_priority(match_type::MT_GL_CUR + match_type::MT_IC_OFF),
            1
        );
    }

    #[test]
    fn test_cmp_match_type() {
        assert!(rs_tag_cmp_match_type(match_type::MT_ST_CUR, match_type::MT_GL_CUR) < 0);
        assert!(rs_tag_cmp_match_type(match_type::MT_GL_OTH, match_type::MT_GL_CUR) > 0);
        assert_eq!(
            rs_tag_cmp_match_type(match_type::MT_ST_CUR, match_type::MT_ST_CUR),
            0
        );
    }

    #[test]
    fn test_best_match_type() {
        assert_eq!(
            rs_tag_best_match_type(match_type::MT_ST_CUR, match_type::MT_GL_CUR),
            match_type::MT_ST_CUR
        );
        assert_eq!(
            rs_tag_best_match_type(match_type::MT_GL_OTH, match_type::MT_ST_CUR),
            match_type::MT_ST_CUR
        );
    }

    // Phase 3 tests

    #[test]
    fn test_null_can_pop() {
        unsafe {
            assert!(!rs_tag_can_pop(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_can_push() {
        unsafe {
            assert!(!rs_tag_can_push(std::ptr::null()));
        }
    }

    #[test]
    fn test_next_match_idx() {
        assert_eq!(rs_tag_next_match_idx(0, 3), 1);
        assert_eq!(rs_tag_next_match_idx(1, 3), 2);
        assert_eq!(rs_tag_next_match_idx(2, 3), 0); // wraps
        assert_eq!(rs_tag_next_match_idx(0, 0), 0); // empty
    }

    #[test]
    fn test_prev_match_idx() {
        assert_eq!(rs_tag_prev_match_idx(2, 3), 1);
        assert_eq!(rs_tag_prev_match_idx(1, 3), 0);
        assert_eq!(rs_tag_prev_match_idx(0, 3), 2); // wraps
        assert_eq!(rs_tag_prev_match_idx(0, 0), 0); // empty
    }

    #[test]
    fn test_first_match_idx() {
        assert_eq!(rs_tag_first_match_idx(), 0);
    }

    #[test]
    fn test_last_match_idx() {
        assert_eq!(rs_tag_last_match_idx(3), 2);
        assert_eq!(rs_tag_last_match_idx(1), 0);
        assert_eq!(rs_tag_last_match_idx(0), 0);
    }

    #[test]
    fn test_clamp_idx() {
        assert_eq!(rs_tag_clamp_idx(5, 0, 10), 5);
        assert_eq!(rs_tag_clamp_idx(-1, 0, 10), 0);
        assert_eq!(rs_tag_clamp_idx(15, 0, 10), 10);
    }

    #[test]
    fn test_null_stack_pos_valid() {
        unsafe {
            assert!(!rs_tag_stack_pos_valid(std::ptr::null(), 0));
        }
    }

    // Phase 4 tests

    #[test]
    fn test_null_taggy_cmp_name() {
        unsafe {
            assert_eq!(rs_taggy_cmp_name(std::ptr::null(), std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_taggy_cmp_fnum() {
        unsafe {
            assert_eq!(rs_taggy_cmp_fnum(std::ptr::null(), std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_taggy_same_tag() {
        unsafe {
            assert!(!rs_taggy_same_tag(std::ptr::null(), std::ptr::null()));
        }
    }

    #[test]
    fn test_null_taggy_in_buffer() {
        unsafe {
            assert!(!rs_taggy_in_buffer(std::ptr::null(), 1));
        }
    }

    #[test]
    fn test_null_tag_fmark_valid() {
        unsafe {
            assert!(!rs_tag_fmark_valid(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_tag_fmark_lnum() {
        unsafe {
            assert_eq!(rs_tag_fmark_lnum(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_tag_fmark_col() {
        unsafe {
            assert_eq!(rs_tag_fmark_col(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_tag_fmark_fnum() {
        unsafe {
            assert_eq!(rs_tag_fmark_fnum(std::ptr::null()), 0);
        }
    }

    // Phase 5 tests

    #[test]
    fn test_search_state_constants() {
        assert_eq!(search_state::TS_START, 0);
        assert_eq!(search_state::TS_LINEAR, 1);
        assert_eq!(search_state::TS_BINARY, 2);
        assert_eq!(search_state::TS_SKIP_BACK, 3);
        assert_eq!(search_state::TS_STEP_FORWARD, 4);
    }

    #[test]
    fn test_null_search_is_linear() {
        unsafe {
            assert!(!rs_tag_search_is_linear(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_search_is_binary() {
        unsafe {
            assert!(!rs_tag_search_is_binary(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_search_done() {
        unsafe {
            assert!(rs_tag_search_done(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_search_match_count() {
        unsafe {
            assert_eq!(rs_tag_search_match_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_search_help_only() {
        unsafe {
            assert!(!rs_tag_search_help_only(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_search_has_matches() {
        unsafe {
            assert!(!rs_tag_search_has_matches(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_tag_file_sorted() {
        unsafe {
            assert!(!rs_tag_file_sorted(std::ptr::null()));
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_search_state_name() {
        let name = rs_tag_search_state_name(search_state::TS_LINEAR);
        unsafe {
            assert_eq!(*name, b'l' as c_char);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_search_state_name_invalid() {
        let name = rs_tag_search_state_name(100);
        unsafe {
            assert_eq!(*name, b'u' as c_char); // "unknown"
        }
    }

    // Phase 6 tests

    #[test]
    fn test_tag_cmd_constants() {
        assert_eq!(tag_cmd::DT_TAG, 1);
        assert_eq!(tag_cmd::DT_POP, 2);
        assert_eq!(tag_cmd::DT_NEXT, 3);
        assert_eq!(tag_cmd::DT_PREV, 4);
        assert_eq!(tag_cmd::DT_FIRST, 5);
        assert_eq!(tag_cmd::DT_LAST, 6);
        assert_eq!(tag_cmd::DT_SELECT, 7);
        assert_eq!(tag_cmd::DT_HELP, 8);
        assert_eq!(tag_cmd::DT_JUMP, 9);
        assert_eq!(tag_cmd::DT_LTAG, 11);
        assert_eq!(tag_cmd::DT_FREE, 99);
    }

    #[test]
    fn test_cmd_is_navigation() {
        assert!(rs_tag_cmd_is_navigation(tag_cmd::DT_POP));
        assert!(!rs_tag_cmd_is_navigation(tag_cmd::DT_TAG));
        assert!(!rs_tag_cmd_is_navigation(tag_cmd::DT_NEXT));
    }

    #[test]
    fn test_cmd_is_match_nav() {
        assert!(rs_tag_cmd_is_match_nav(tag_cmd::DT_NEXT));
        assert!(rs_tag_cmd_is_match_nav(tag_cmd::DT_PREV));
        assert!(rs_tag_cmd_is_match_nav(tag_cmd::DT_FIRST));
        assert!(rs_tag_cmd_is_match_nav(tag_cmd::DT_LAST));
        assert!(!rs_tag_cmd_is_match_nav(tag_cmd::DT_TAG));
        assert!(!rs_tag_cmd_is_match_nav(tag_cmd::DT_POP));
    }

    #[test]
    fn test_cmd_uses_pattern() {
        assert!(rs_tag_cmd_uses_pattern(tag_cmd::DT_TAG));
        assert!(rs_tag_cmd_uses_pattern(tag_cmd::DT_SELECT));
        assert!(rs_tag_cmd_uses_pattern(tag_cmd::DT_JUMP));
        assert!(rs_tag_cmd_uses_pattern(tag_cmd::DT_LTAG));
        assert!(!rs_tag_cmd_uses_pattern(tag_cmd::DT_POP));
        assert!(!rs_tag_cmd_uses_pattern(tag_cmd::DT_NEXT));
    }

    #[test]
    fn test_cmd_needs_tag() {
        assert!(rs_tag_cmd_needs_tag(tag_cmd::DT_TAG));
        assert!(rs_tag_cmd_needs_tag(tag_cmd::DT_HELP));
        assert!(rs_tag_cmd_needs_tag(tag_cmd::DT_SELECT));
        assert!(rs_tag_cmd_needs_tag(tag_cmd::DT_JUMP));
        assert!(!rs_tag_cmd_needs_tag(tag_cmd::DT_POP));
        assert!(!rs_tag_cmd_needs_tag(tag_cmd::DT_NEXT));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_cmd_name() {
        let name = rs_tag_cmd_name(tag_cmd::DT_TAG);
        unsafe {
            assert_eq!(*name, b't' as c_char);
        }
    }

    #[test]
    fn test_cmd_valid() {
        assert!(rs_tag_cmd_valid(tag_cmd::DT_TAG));
        assert!(rs_tag_cmd_valid(tag_cmd::DT_POP));
        assert!(rs_tag_cmd_valid(tag_cmd::DT_FREE));
        assert!(!rs_tag_cmd_valid(0));
        assert!(!rs_tag_cmd_valid(100));
    }

    #[test]
    fn test_cmd_direction() {
        assert_eq!(rs_tag_cmd_direction(tag_cmd::DT_NEXT), 1);
        assert_eq!(rs_tag_cmd_direction(tag_cmd::DT_LAST), 1);
        assert_eq!(rs_tag_cmd_direction(tag_cmd::DT_PREV), -1);
        assert_eq!(rs_tag_cmd_direction(tag_cmd::DT_FIRST), -1);
        assert_eq!(rs_tag_cmd_direction(tag_cmd::DT_POP), -1);
        assert_eq!(rs_tag_cmd_direction(tag_cmd::DT_TAG), 0);
    }

    #[test]
    fn test_type_to_cmd() {
        // Should be same as cmd_name
        assert_eq!(
            rs_tag_type_to_cmd(tag_cmd::DT_TAG),
            rs_tag_cmd_name(tag_cmd::DT_TAG)
        );
    }

    // TAGSTACKSIZE constant test
    #[test]
    fn test_tagstacksize() {
        assert_eq!(TAGSTACKSIZE, 20);
    }
}
