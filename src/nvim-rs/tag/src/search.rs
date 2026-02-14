//! Tag file search infrastructure for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for tag file searching including
//! binary search, linear search, and tag file state management.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Type aliases for C types
// =============================================================================

/// File offset type (matches off_T in Neovim)
type OffT = i64;

// =============================================================================
// Constants
// =============================================================================

/// Number of match type buckets
pub const MT_COUNT: usize = 16;

// =============================================================================
// Tag search state enum
// =============================================================================

/// States used during a tags search
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TagSearchState {
    /// At start of file
    #[default]
    Start = 0,
    /// Linear searching forward, till EOF
    Linear = 1,
    /// Binary searching
    Binary = 2,
    /// Skipping backwards
    SkipBack = 3,
    /// Stepping forwards
    StepForward = 4,
}

impl From<c_int> for TagSearchState {
    fn from(value: c_int) -> Self {
        match value {
            1 => Self::Linear,
            2 => Self::Binary,
            3 => Self::SkipBack,
            4 => Self::StepForward,
            _ => Self::Start,
        }
    }
}

// =============================================================================
// Tags read status
// =============================================================================

/// Return values used when reading lines from a tags file
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagsReadStatus {
    /// Successfully read a line
    Success = 1,
    /// End of file reached
    Eof = 2,
    /// Line should be ignored
    Ignore = 3,
}

// =============================================================================
// Tag match status
// =============================================================================

/// Return values used when matching tags against a pattern
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagMatchStatus {
    /// Tag matches successfully
    Success = 1,
    /// Tag does not match
    Fail = 2,
    /// Stop searching
    Stop = 3,
    /// Continue to next tag
    Next = 4,
}

// =============================================================================
// Binary search info structure
// =============================================================================

/// Binary search file offsets in a tags file
#[repr(C)]
#[derive(Default)]
pub struct TagSearchInfo {
    /// Offset for first char of first line that could match
    pub low_offset: OffT,
    /// Offset of char after last line that could match
    pub high_offset: OffT,
    /// Current file offset in search range
    pub curr_offset: OffT,
    /// curr_offset used when skipping back
    pub curr_offset_used: OffT,
    /// Where the binary search found a tag
    pub match_offset: OffT,
    /// First char at low_offset
    pub low_char: c_int,
    /// First char at high_offset
    pub high_char: c_int,
}

// =============================================================================
// Match arguments structure
// =============================================================================

/// Arguments used for matching tags against a pattern
#[repr(C)]
#[derive(Default)]
pub struct FindTagsMatchArgs {
    /// Tag match offset
    pub matchoff: c_int,
    /// True if the tag matches a regexp
    pub match_re: bool,
    /// True if the tag matches with case
    pub match_no_ic: bool,
    /// Regular expression used
    pub has_re: bool,
    /// Tags file sorted ignoring case (foldcase)
    pub sortic: bool,
    /// Tags file not sorted
    pub sort_error: bool,
}

// =============================================================================
// FindTags state structure (partial - most fields managed by C)
// =============================================================================

/// Core search state fields that are managed in Rust
#[repr(C)]
pub struct FindTagsStateCore {
    /// Tag search state
    pub state: TagSearchState,
    /// Stop when match found or error
    pub stop_searching: bool,
    /// Number of matches found
    pub match_count: c_int,
    /// Do a linear search
    pub linear: bool,
    /// !_TAG_FILE_SORTED value
    pub tag_file_sorted: c_int,
    /// Only search for help tags
    pub help_only: bool,
    /// Did open a tag file
    pub did_open: bool,
    /// MAXCOL: find all matches, other: minimal number of matches
    pub mincount: c_int,
}

impl Default for FindTagsStateCore {
    fn default() -> Self {
        Self {
            state: TagSearchState::Start,
            stop_searching: false,
            match_count: 0,
            linear: false,
            tag_file_sorted: 0,
            help_only: false,
            did_open: false,
            mincount: 0,
        }
    }
}

// =============================================================================
// Search state functions
// =============================================================================

/// Initialize search state to defaults.
#[no_mangle]
pub unsafe extern "C" fn rs_search_state_init(state: *mut FindTagsStateCore) {
    if state.is_null() {
        return;
    }
    *state = FindTagsStateCore::default();
}

/// Check if search state indicates binary search mode.
#[no_mangle]
pub unsafe extern "C" fn rs_search_is_binary(state: *const FindTagsStateCore) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).state == TagSearchState::Binary
}

/// Check if search state indicates linear search mode.
#[no_mangle]
pub unsafe extern "C" fn rs_search_is_linear(state: *const FindTagsStateCore) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).state == TagSearchState::Linear || (*state).linear
}

/// Check if search is complete.
#[no_mangle]
pub unsafe extern "C" fn rs_search_is_done(state: *const FindTagsStateCore) -> bool {
    if state.is_null() {
        return true;
    }
    (*state).stop_searching
        || matches!(
            (*state).state,
            TagSearchState::SkipBack | TagSearchState::StepForward
        )
}

/// Get the current search state value.
#[no_mangle]
pub unsafe extern "C" fn rs_search_get_state(state: *const FindTagsStateCore) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).state as c_int
}

/// Set the search state.
#[no_mangle]
pub unsafe extern "C" fn rs_search_set_state(state: *mut FindTagsStateCore, new_state: c_int) {
    if state.is_null() {
        return;
    }
    (*state).state = TagSearchState::from(new_state);
}

/// Increment the match count.
#[no_mangle]
pub unsafe extern "C" fn rs_search_inc_match_count(state: *mut FindTagsStateCore) {
    if state.is_null() {
        return;
    }
    (*state).match_count += 1;
}

/// Get the match count.
#[no_mangle]
pub unsafe extern "C" fn rs_search_get_match_count(state: *const FindTagsStateCore) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).match_count
}

/// Check if we should stop searching.
#[no_mangle]
pub unsafe extern "C" fn rs_search_should_stop(state: *const FindTagsStateCore) -> bool {
    if state.is_null() {
        return true;
    }
    (*state).stop_searching
}

/// Set the stop searching flag.
#[no_mangle]
pub unsafe extern "C" fn rs_search_set_stop(state: *mut FindTagsStateCore, stop: bool) {
    if state.is_null() {
        return;
    }
    (*state).stop_searching = stop;
}

// =============================================================================
// Binary search info functions
// =============================================================================

/// Initialize binary search info.
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_init(info: *mut TagSearchInfo) {
    if info.is_null() {
        return;
    }
    *info = TagSearchInfo::default();
}

/// Set the search range for binary search.
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_set_range(info: *mut TagSearchInfo, low: OffT, high: OffT) {
    if info.is_null() {
        return;
    }
    (*info).low_offset = low;
    (*info).high_offset = high;
}

/// Calculate the midpoint for binary search.
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_midpoint(info: *const TagSearchInfo) -> OffT {
    if info.is_null() {
        return 0;
    }
    i64::midpoint((*info).low_offset, (*info).high_offset)
}

/// Update binary search range based on comparison result.
///
/// If cmp < 0, tag is before target (search in upper half)
/// If cmp > 0, tag is after target (search in lower half)
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_update(info: *mut TagSearchInfo, curr: OffT, cmp: c_int) {
    if info.is_null() {
        return;
    }
    if cmp < 0 {
        // Tag comes before target - search upper half
        (*info).low_offset = curr;
    } else {
        // Tag comes after target - search lower half
        (*info).high_offset = curr;
    }
    (*info).curr_offset = curr;
}

/// Check if binary search has converged (range is too small to continue).
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_converged(info: *const TagSearchInfo) -> bool {
    if info.is_null() {
        return true;
    }
    (*info).high_offset - (*info).low_offset < 2
}

/// Record a match position during binary search.
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_record_match(info: *mut TagSearchInfo, offset: OffT) {
    if info.is_null() {
        return;
    }
    (*info).match_offset = offset;
}

/// Get the recorded match offset.
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_get_match_offset(info: *const TagSearchInfo) -> OffT {
    if info.is_null() {
        return 0;
    }
    (*info).match_offset
}

// =============================================================================
// Match args functions
// =============================================================================

/// Initialize match args.
#[no_mangle]
pub unsafe extern "C" fn rs_match_args_init(args: *mut FindTagsMatchArgs) {
    if args.is_null() {
        return;
    }
    *args = FindTagsMatchArgs::default();
}

/// Set the has_re flag.
#[no_mangle]
pub unsafe extern "C" fn rs_match_args_set_has_re(args: *mut FindTagsMatchArgs, has_re: bool) {
    if args.is_null() {
        return;
    }
    (*args).has_re = has_re;
}

/// Get the has_re flag.
#[no_mangle]
pub unsafe extern "C" fn rs_match_args_get_has_re(args: *const FindTagsMatchArgs) -> bool {
    if args.is_null() {
        return false;
    }
    (*args).has_re
}

/// Set match flags after a successful match.
#[no_mangle]
pub unsafe extern "C" fn rs_match_args_set_match(
    args: *mut FindTagsMatchArgs,
    matchoff: c_int,
    match_re: bool,
    match_no_ic: bool,
) {
    if args.is_null() {
        return;
    }
    (*args).matchoff = matchoff;
    (*args).match_re = match_re;
    (*args).match_no_ic = match_no_ic;
}

/// Record a sort error (tags file not sorted).
#[no_mangle]
pub unsafe extern "C" fn rs_match_args_set_sort_error(args: *mut FindTagsMatchArgs) {
    if args.is_null() {
        return;
    }
    (*args).sort_error = true;
}

/// Check if a sort error was detected.
#[no_mangle]
pub unsafe extern "C" fn rs_match_args_has_sort_error(args: *const FindTagsMatchArgs) -> bool {
    if args.is_null() {
        return false;
    }
    (*args).sort_error
}

// =============================================================================
// Tag file sorting detection
// =============================================================================

/// Values for tag_file_sorted field
pub mod sorted_status {
    use std::ffi::c_int;

    /// Tags file is not sorted
    pub const UNSORTED: c_int = 0;
    /// Tags file is sorted
    pub const SORTED: c_int = 1;
    /// Tags file is sorted ignoring case
    pub const FOLDED: c_int = 2;
}

/// Check if sorting status indicates sorted file.
#[no_mangle]
pub extern "C" fn rs_tag_file_is_sorted(status: c_int) -> bool {
    status > sorted_status::UNSORTED
}

/// Check if sorting status indicates case-insensitive sorting.
#[no_mangle]
pub extern "C" fn rs_tag_file_is_folded(status: c_int) -> bool {
    status == sorted_status::FOLDED
}

/// Parse the !_TAG_FILE_SORTED line value.
///
/// Returns the sorting status from the value string.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_sorted_value(value: *const c_char) -> c_int {
    if value.is_null() {
        return sorted_status::UNSORTED;
    }

    let c = *value as u8;
    if c == b'1' {
        sorted_status::SORTED
    } else if c == b'2' {
        sorted_status::FOLDED
    } else {
        sorted_status::UNSORTED
    }
}

// =============================================================================
// String comparison for binary search
// =============================================================================

/// Compare a tag name with pattern head for binary search.
///
/// Returns:
/// - negative if tagname < head
/// - positive if tagname > head
/// - 0 if they match for the head length
#[no_mangle]
pub unsafe extern "C" fn rs_tag_cmp_head(
    tagname: *const c_char,
    head: *const c_char,
    headlen: c_int,
    ignore_case: bool,
) -> c_int {
    if tagname.is_null() || head.is_null() || headlen <= 0 {
        return 0;
    }

    for i in 0..headlen as usize {
        let t = *tagname.add(i) as u8;
        let h = *head.add(i) as u8;

        // End of tagname before headlen
        if t == 0 {
            return -1;
        }

        let cmp = if ignore_case {
            t.to_ascii_lowercase() as i32 - h.to_ascii_lowercase() as i32
        } else {
            t as i32 - h as i32
        };

        if cmp != 0 {
            return cmp as c_int;
        }
    }

    0 // Match for headlen characters
}

/// Compare two tag names for sorting.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_cmp_names(
    name1: *const c_char,
    name2: *const c_char,
    ignore_case: bool,
) -> c_int {
    if name1.is_null() && name2.is_null() {
        return 0;
    }
    if name1.is_null() {
        return -1;
    }
    if name2.is_null() {
        return 1;
    }

    let mut i = 0usize;
    loop {
        let c1 = *name1.add(i) as u8;
        let c2 = *name2.add(i) as u8;

        if c1 == 0 && c2 == 0 {
            return 0;
        }
        if c1 == 0 {
            return -1;
        }
        if c2 == 0 {
            return 1;
        }

        let cmp = if ignore_case {
            c1.to_ascii_lowercase() as i32 - c2.to_ascii_lowercase() as i32
        } else {
            c1 as i32 - c2 as i32
        };

        if cmp != 0 {
            return cmp as c_int;
        }

        i += 1;
    }
}

// =============================================================================
// Allocate/Free functions for structures
// =============================================================================

/// Allocate a new TagSearchInfo structure.
#[no_mangle]
pub extern "C" fn rs_search_info_new() -> *mut TagSearchInfo {
    Box::into_raw(Box::new(TagSearchInfo::default()))
}

/// Free a TagSearchInfo structure.
#[no_mangle]
pub unsafe extern "C" fn rs_search_info_free(info: *mut TagSearchInfo) {
    if !info.is_null() {
        drop(Box::from_raw(info));
    }
}

/// Allocate a new FindTagsMatchArgs structure.
#[no_mangle]
pub extern "C" fn rs_match_args_new() -> *mut FindTagsMatchArgs {
    Box::into_raw(Box::new(FindTagsMatchArgs::default()))
}

/// Free a FindTagsMatchArgs structure.
#[no_mangle]
pub unsafe extern "C" fn rs_match_args_free(args: *mut FindTagsMatchArgs) {
    if !args.is_null() {
        drop(Box::from_raw(args));
    }
}

/// Allocate a new FindTagsStateCore structure.
#[no_mangle]
pub extern "C" fn rs_search_state_new() -> *mut FindTagsStateCore {
    Box::into_raw(Box::new(FindTagsStateCore::default()))
}

/// Free a FindTagsStateCore structure.
#[no_mangle]
pub unsafe extern "C" fn rs_search_state_free(state: *mut FindTagsStateCore) {
    if !state.is_null() {
        drop(Box::from_raw(state));
    }
}

// =============================================================================
// Find tags entry point helpers
// =============================================================================

/// Flags for find_tags()
pub mod find_tags_flags {
    use std::ffi::c_int;

    /// Only search for help tags
    pub const TAG_HELP: c_int = 1;
    /// Only return name of tag
    pub const TAG_NAMES: c_int = 2;
    /// Pattern is a regexp
    pub const TAG_REGEXP: c_int = 4;
    /// Don't always ignore case
    pub const TAG_NOIC: c_int = 8;
    /// Keep language for tag search
    pub const TAG_KEEP_LANG: c_int = 0x10;
    /// Don't call tagfunc
    pub const TAG_NO_TAGFUNC: c_int = 0x20;
    /// Verbose message
    pub const TAG_VERBOSE: c_int = 0x40;
    /// Complete tag only
    pub const TAG_CSCOPE: c_int = 0x80;
    /// Used for ins_compl_add_infercase_addchar()
    pub const TAG_INS_COMP: c_int = 0x100;
}

/// MAXCOL constant for findall
const MAXCOL: c_int = 0x7FFF_FFFF;
/// TAG_MANY constant for finding multiple matches
const TAG_MANY: c_int = 10000;

/// Structure for find_tags search context
#[repr(C)]
pub struct FindTagsContext {
    /// Pattern being searched
    pub pat: *const c_char,
    /// Pattern length
    pub pat_len: c_int,
    /// Search flags
    pub flags: c_int,
    /// Minimum number of matches to find
    pub mincount: c_int,
    /// Find all matches
    pub findall: bool,
    /// Using regexp
    pub has_re: bool,
    /// Don't ignore case for matching
    pub noic: bool,
    /// Verbose output
    pub verbose: bool,
    /// Help tags only
    pub help_only: bool,
    /// Current round (1 = case match, 2 = ignore case)
    pub round: c_int,
    /// Using linear search
    pub linear: bool,
    /// Pattern head length (non-regexp part)
    pub headlen: c_int,
}

impl Default for FindTagsContext {
    fn default() -> Self {
        Self {
            pat: std::ptr::null(),
            pat_len: 0,
            flags: 0,
            mincount: 0,
            findall: false,
            has_re: false,
            noic: false,
            verbose: false,
            help_only: false,
            round: 1,
            linear: false,
            headlen: 0,
        }
    }
}

/// Initialize a find_tags search context.
///
/// This sets up the initial state for a tag search based on the
/// pattern, flags, and mincount parameters.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_init(
    ctx: *mut FindTagsContext,
    pat: *const c_char,
    pat_len: c_int,
    flags: c_int,
    mincount: c_int,
) {
    if ctx.is_null() || pat.is_null() {
        return;
    }

    (*ctx).pat = pat;
    (*ctx).pat_len = pat_len;
    (*ctx).flags = flags;
    (*ctx).mincount = mincount;
    (*ctx).findall = mincount == MAXCOL || mincount == TAG_MANY;
    (*ctx).has_re = (flags & find_tags_flags::TAG_REGEXP) != 0;
    (*ctx).noic = (flags & find_tags_flags::TAG_NOIC) != 0;
    (*ctx).verbose = (flags & find_tags_flags::TAG_VERBOSE) != 0;
    (*ctx).help_only = (flags & find_tags_flags::TAG_HELP) != 0;
    (*ctx).round = 1;
    (*ctx).linear = false;
    (*ctx).headlen = 0;
}

/// Allocate a new find_tags context.
#[no_mangle]
pub extern "C" fn rs_find_tags_context_new() -> *mut FindTagsContext {
    Box::into_raw(Box::new(FindTagsContext::default()))
}

/// Free a find_tags context.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_context_free(ctx: *mut FindTagsContext) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx));
    }
}

/// Check if we should find all matches.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_is_findall(ctx: *const FindTagsContext) -> bool {
    if ctx.is_null() {
        return false;
    }
    (*ctx).findall
}

/// Check if search uses regexp.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_has_re(ctx: *const FindTagsContext) -> bool {
    if ctx.is_null() {
        return false;
    }
    (*ctx).has_re
}

/// Check if search should not ignore case.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_noic(ctx: *const FindTagsContext) -> bool {
    if ctx.is_null() {
        return false;
    }
    (*ctx).noic
}

/// Get current search round.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_get_round(ctx: *const FindTagsContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    (*ctx).round
}

/// Set current search round.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_set_round(ctx: *mut FindTagsContext, round: c_int) {
    if ctx.is_null() {
        return;
    }
    (*ctx).round = round;
}

/// Get linear search flag.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_is_linear(ctx: *const FindTagsContext) -> bool {
    if ctx.is_null() {
        return false;
    }
    (*ctx).linear
}

/// Set linear search flag.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_set_linear(ctx: *mut FindTagsContext, linear: bool) {
    if ctx.is_null() {
        return;
    }
    (*ctx).linear = linear;
}

/// Set the head length (non-regexp prefix).
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_set_headlen(ctx: *mut FindTagsContext, headlen: c_int) {
    if ctx.is_null() {
        return;
    }
    (*ctx).headlen = headlen;
}

/// Get the head length.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_get_headlen(ctx: *const FindTagsContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    (*ctx).headlen
}

/// Determine if linear search should be used for the current round.
///
/// Linear search is used when:
/// - headlen is 0 (no fixed prefix to binary search for)
/// - tagbsearch option is off (p_tbs is false)
/// - this is round 2 (case-insensitive fallback)
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_should_linear(
    ctx: *const FindTagsContext,
    p_tbs: bool,
) -> bool {
    if ctx.is_null() {
        return true;
    }
    (*ctx).headlen == 0 || !p_tbs || (*ctx).round == 2
}

/// Determine if another search round is needed.
///
/// Returns false (stop) if:
/// - linear search was already done
/// - TAG_NOIC was used and ignorecase is off
/// - case-insensitive search was already done
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_need_another_round(
    ctx: *const FindTagsContext,
    p_ic: bool,
    rm_ic: bool,
) -> bool {
    if ctx.is_null() {
        return false;
    }

    // Stop if linear search was done
    if (*ctx).linear {
        return false;
    }

    // Stop if TAG_NOIC used and 'ignorecase' not set
    if !p_ic && (*ctx).noic {
        return false;
    }

    // Stop if already did case-insensitive search
    if rm_ic {
        return false;
    }

    true
}

/// Calculate the ignore-case flag for the current search state.
///
/// The ignore-case flag is set when:
/// - 'ignorecase' is on OR TAG_NOIC not used
/// - AND one of: findall, headlen==0, or 'tagbsearch' is off
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_calc_rm_ic(
    ctx: *const FindTagsContext,
    p_ic: bool,
    p_tbs: bool,
) -> bool {
    if ctx.is_null() {
        return false;
    }
    (p_ic || !(*ctx).noic) && ((*ctx).findall || (*ctx).headlen == 0 || !p_tbs)
}

// =============================================================================
// Help language handling for find_tags
// =============================================================================

/// Maximum length for help language code (e.g., "en", "ja")
pub const HELP_LANG_LEN: usize = 2;

/// Extract help language from pattern if present.
///
/// If the pattern ends with "@xx" where xx is two ASCII letters,
/// returns the offset of the '@' character. Otherwise returns 0.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_extract_help_lang(
    pat: *const c_char,
    pat_len: c_int,
) -> c_int {
    if pat.is_null() || pat_len < 4 {
        return 0;
    }

    let len = pat_len as usize;

    // Check for @xx pattern at end
    let at_pos = len - 3;
    let c1 = *pat.add(at_pos) as u8;
    let c2 = *pat.add(at_pos + 1) as u8;
    let c3 = *pat.add(at_pos + 2) as u8;

    if c1 == b'@' && c2.is_ascii_alphabetic() && c3.is_ascii_alphabetic() {
        at_pos as c_int
    } else {
        0
    }
}

/// Get the help language code from a pattern.
///
/// Copies the 2-character language code to the output buffer.
/// Returns true if successful.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_get_help_lang(
    pat: *const c_char,
    pat_len: c_int,
    lang_out: *mut c_char,
) -> bool {
    if pat.is_null() || lang_out.is_null() || pat_len < 4 {
        return false;
    }

    let at_offset = rs_find_tags_extract_help_lang(pat, pat_len);
    if at_offset == 0 {
        return false;
    }

    // Copy the two-character language code
    *lang_out = *pat.add(at_offset as usize + 1);
    *lang_out.add(1) = *pat.add(at_offset as usize + 2);
    *lang_out.add(2) = 0; // null terminate

    true
}

/// Extract help language from a tag filename.
///
/// Tag files like "doc/tags-ja" have the language as the last two characters
/// after the dash. Returns true and copies to lang_out if found.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tags_lang_from_fname(
    fname: *const c_char,
    lang_out: *mut c_char,
) -> bool {
    if fname.is_null() || lang_out.is_null() {
        return false;
    }

    // Find the length
    let mut len = 0usize;
    while *fname.add(len) != 0 {
        len += 1;
    }

    // Check for "-xx" at end (tags-ja format)
    if len > 3 {
        let dash = *fname.add(len - 3) as u8;
        let c1 = *fname.add(len - 2) as u8;
        let c2 = *fname.add(len - 1) as u8;

        if dash == b'-' && c1.is_ascii_alphabetic() && c2.is_ascii_alphabetic() {
            *lang_out = c1 as c_char;
            *lang_out.add(1) = c2 as c_char;
            *lang_out.add(2) = 0;
            return true;
        }
    }

    // Default to "en"
    *lang_out = b'e' as c_char;
    *lang_out.add(1) = b'n' as c_char;
    *lang_out.add(2) = 0;
    true
}

/// Compare two help language codes case-insensitively.
#[no_mangle]
pub unsafe extern "C" fn rs_help_lang_matches(lang1: *const c_char, lang2: *const c_char) -> bool {
    if lang1.is_null() || lang2.is_null() {
        return false;
    }

    let c1a = (*lang1 as u8).to_ascii_lowercase();
    let c1b = (*lang1.add(1) as u8).to_ascii_lowercase();
    let c2a = (*lang2 as u8).to_ascii_lowercase();
    let c2b = (*lang2.add(1) as u8).to_ascii_lowercase();

    c1a == c2a && c1b == c2b
}

// =============================================================================
// Match type classification for find_tags
// =============================================================================

/// Match type values for tag matching priority.
///
/// These correspond to the MT_* constants in tag.c and are used to
/// classify matches into priority buckets.
pub mod match_type {
    use std::ffi::c_int;

    /// Offset added to match type for current file
    pub const MT_IC_OFF: c_int = 0;
    /// Global tag, ignore case, other file
    pub const MT_GL_IC_OTH: c_int = 1;
    /// Static tag, ignore case, other file
    pub const MT_ST_IC_OTH: c_int = 2;
    /// Global tag, ignore case, current file
    pub const MT_GL_IC_CUR: c_int = 3;
    /// Static tag, ignore case, current file
    pub const MT_ST_IC_CUR: c_int = 4;
    /// Offset added for no-ignore-case matches
    pub const MT_NO_IC_OFF: c_int = 4;
    /// Global tag, no ignore case, other file
    pub const MT_GL_NO_IC_OTH: c_int = 5;
    /// Static tag, no ignore case, other file
    pub const MT_ST_NO_IC_OTH: c_int = 6;
    /// Global tag, no ignore case, current file
    pub const MT_GL_NO_IC_CUR: c_int = 7;
    /// Static tag, no ignore case, current file
    pub const MT_ST_NO_IC_CUR: c_int = 8;
    /// Offset for regexp matches
    pub const MT_RE_OFF: c_int = 8;
    /// Total number of match type buckets
    pub const MT_COUNT: c_int = 16;
}

/// Calculate the match type for a tag match.
///
/// The match type determines the priority bucket for the match.
#[no_mangle]
pub extern "C" fn rs_calc_match_type(
    is_static: bool,
    is_current_file: bool,
    match_re: bool,
    match_no_ic: bool,
) -> c_int {
    // Start with static/global distinction
    let mut mtype = if is_static {
        if is_current_file {
            match_type::MT_ST_IC_CUR
        } else {
            match_type::MT_ST_IC_OTH
        }
    } else if is_current_file {
        match_type::MT_GL_IC_CUR
    } else {
        match_type::MT_GL_IC_OTH
    };

    // Adjust for case-sensitive match
    if match_no_ic {
        mtype += match_type::MT_NO_IC_OFF;
    }

    // Adjust for regexp match
    if match_re {
        mtype += match_type::MT_RE_OFF;
    }

    mtype
}

/// Check if a match type indicates a current file match.
#[no_mangle]
pub extern "C" fn rs_match_type_is_current_file(mtype: c_int) -> bool {
    let base = mtype % match_type::MT_RE_OFF;
    base == match_type::MT_GL_IC_CUR
        || base == match_type::MT_ST_IC_CUR
        || base == match_type::MT_GL_NO_IC_CUR
        || base == match_type::MT_ST_NO_IC_CUR
}

/// Check if a match type indicates a static tag.
#[no_mangle]
pub extern "C" fn rs_match_type_is_static(mtype: c_int) -> bool {
    let base = mtype % match_type::MT_RE_OFF;
    base == match_type::MT_ST_IC_OTH
        || base == match_type::MT_ST_IC_CUR
        || base == match_type::MT_ST_NO_IC_OTH
        || base == match_type::MT_ST_NO_IC_CUR
}

// =============================================================================
// Tag comparison for sorting in find_tags
// =============================================================================

/// Compare two tag matches for sorting.
///
/// This compares tag matches by:
/// 1. Match type (priority bucket)
/// 2. Help language priority (if applicable)
/// 3. Tag name
#[no_mangle]
pub unsafe extern "C" fn rs_tag_match_cmp(
    mtype1: c_int,
    mtype2: c_int,
    name1: *const c_char,
    name2: *const c_char,
) -> c_int {
    // First compare by match type
    if mtype1 != mtype2 {
        return mtype1 - mtype2;
    }

    // Then by name
    rs_tag_cmp_names(name1, name2, false)
}

// =============================================================================
// Phase 2: C struct initialization via FFI
// =============================================================================

/// Opaque handle to `findtags_state_T`
type FindTagsStateHandle = *mut c_void;
/// Opaque handle to `findtags_match_args_T`
type FindTagsMatchArgsHandle = *mut c_void;

extern "C" {
    fn nvim_findtags_init_tag_fname(st: FindTagsStateHandle);
    fn nvim_findtags_set_fp_null(st: FindTagsStateHandle);
    fn nvim_findtags_init_orgpat(st: FindTagsStateHandle, pat: *mut c_char);
    fn nvim_findtags_set_fields(st: FindTagsStateHandle, flags: c_int, mincount: c_int);
    fn nvim_findtags_init_match_arrays(st: FindTagsStateHandle);
    fn nvim_findtags_state_free_inner(st: FindTagsStateHandle);
    fn nvim_findtags_matchargs_init(margs: FindTagsMatchArgsHandle, flags: c_int);
}

/// Initialize a `findtags_state_T` struct for a tag search.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
/// - `pat` must be a valid C string that outlives the search
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_state_init(
    st: FindTagsStateHandle,
    pat: *mut c_char,
    flags: c_int,
    mincount: c_int,
) {
    if st.is_null() {
        return;
    }
    nvim_findtags_init_tag_fname(st);
    nvim_findtags_set_fp_null(st);
    nvim_findtags_init_orgpat(st, pat);
    nvim_findtags_set_fields(st, flags, mincount);
    nvim_findtags_init_match_arrays(st);
}

/// Free the inner resources of a `findtags_state_T` struct.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_state_free(st: FindTagsStateHandle) {
    if st.is_null() {
        return;
    }
    nvim_findtags_state_free_inner(st);
}

/// Initialize a `findtags_match_args_T` struct.
///
/// # Safety
///
/// - `margs` must be a valid pointer to a `findtags_match_args_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_matchargs_init(margs: FindTagsMatchArgsHandle, flags: c_int) {
    if margs.is_null() {
        return;
    }
    nvim_findtags_matchargs_init(margs, flags);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_tag_search_state_default() {
        assert_eq!(TagSearchState::default(), TagSearchState::Start);
    }

    #[test]
    fn test_tag_search_state_from() {
        assert_eq!(TagSearchState::from(0), TagSearchState::Start);
        assert_eq!(TagSearchState::from(1), TagSearchState::Linear);
        assert_eq!(TagSearchState::from(2), TagSearchState::Binary);
        assert_eq!(TagSearchState::from(3), TagSearchState::SkipBack);
        assert_eq!(TagSearchState::from(4), TagSearchState::StepForward);
        assert_eq!(TagSearchState::from(99), TagSearchState::Start);
    }

    #[test]
    fn test_tag_search_info_default() {
        let info = TagSearchInfo::default();
        assert_eq!(info.low_offset, 0);
        assert_eq!(info.high_offset, 0);
        assert_eq!(info.curr_offset, 0);
        assert_eq!(info.match_offset, 0);
    }

    #[test]
    fn test_findtags_match_args_default() {
        let args = FindTagsMatchArgs::default();
        assert_eq!(args.matchoff, 0);
        assert!(!args.match_re);
        assert!(!args.match_no_ic);
        assert!(!args.has_re);
        assert!(!args.sortic);
        assert!(!args.sort_error);
    }

    #[test]
    fn test_search_info_midpoint() {
        unsafe {
            let info = TagSearchInfo {
                low_offset: 100,
                high_offset: 200,
                ..Default::default()
            };
            assert_eq!(rs_search_info_midpoint(std::ptr::addr_of!(info)), 150);
        }
    }

    #[test]
    fn test_search_info_converged() {
        unsafe {
            // Not converged
            let info = TagSearchInfo {
                low_offset: 0,
                high_offset: 100,
                ..Default::default()
            };
            assert!(!rs_search_info_converged(std::ptr::addr_of!(info)));

            // Converged
            let info2 = TagSearchInfo {
                low_offset: 100,
                high_offset: 101,
                ..Default::default()
            };
            assert!(rs_search_info_converged(std::ptr::addr_of!(info2)));
        }
    }

    #[test]
    fn test_tag_file_sorting() {
        assert!(!rs_tag_file_is_sorted(sorted_status::UNSORTED));
        assert!(rs_tag_file_is_sorted(sorted_status::SORTED));
        assert!(rs_tag_file_is_sorted(sorted_status::FOLDED));

        assert!(!rs_tag_file_is_folded(sorted_status::UNSORTED));
        assert!(!rs_tag_file_is_folded(sorted_status::SORTED));
        assert!(rs_tag_file_is_folded(sorted_status::FOLDED));
    }

    #[test]
    fn test_parse_sorted_value() {
        unsafe {
            assert_eq!(rs_parse_sorted_value(ptr::null()), sorted_status::UNSORTED);
            assert_eq!(
                rs_parse_sorted_value(c"0".as_ptr()),
                sorted_status::UNSORTED
            );
            assert_eq!(rs_parse_sorted_value(c"1".as_ptr()), sorted_status::SORTED);
            assert_eq!(rs_parse_sorted_value(c"2".as_ptr()), sorted_status::FOLDED);
        }
    }

    #[test]
    fn test_tag_cmp_head() {
        unsafe {
            // Basic comparisons
            assert_eq!(
                rs_tag_cmp_head(c"abc".as_ptr(), c"abc".as_ptr(), 3, false),
                0
            );
            assert!(rs_tag_cmp_head(c"abd".as_ptr(), c"abc".as_ptr(), 3, false) > 0);
            assert!(rs_tag_cmp_head(c"abb".as_ptr(), c"abc".as_ptr(), 3, false) < 0);

            // Case insensitive
            assert_eq!(
                rs_tag_cmp_head(c"ABC".as_ptr(), c"abc".as_ptr(), 3, true),
                0
            );

            // Shorter tagname
            assert!(rs_tag_cmp_head(c"ab".as_ptr(), c"abc".as_ptr(), 3, false) < 0);

            // Null handling
            assert_eq!(rs_tag_cmp_head(ptr::null(), c"abc".as_ptr(), 3, false), 0);
        }
    }

    #[test]
    fn test_tag_cmp_names() {
        unsafe {
            assert_eq!(rs_tag_cmp_names(c"abc".as_ptr(), c"abc".as_ptr(), false), 0);
            assert!(rs_tag_cmp_names(c"abd".as_ptr(), c"abc".as_ptr(), false) > 0);
            assert!(rs_tag_cmp_names(c"abb".as_ptr(), c"abc".as_ptr(), false) < 0);

            // Different lengths
            assert!(rs_tag_cmp_names(c"ab".as_ptr(), c"abc".as_ptr(), false) < 0);
            assert!(rs_tag_cmp_names(c"abcd".as_ptr(), c"abc".as_ptr(), false) > 0);

            // Case insensitive
            assert_eq!(rs_tag_cmp_names(c"ABC".as_ptr(), c"abc".as_ptr(), true), 0);

            // Null handling
            assert_eq!(rs_tag_cmp_names(ptr::null(), ptr::null(), false), 0);
            assert!(rs_tag_cmp_names(ptr::null(), c"abc".as_ptr(), false) < 0);
            assert!(rs_tag_cmp_names(c"abc".as_ptr(), ptr::null(), false) > 0);
        }
    }

    #[test]
    fn test_alloc_free() {
        unsafe {
            let info = rs_search_info_new();
            assert!(!info.is_null());
            rs_search_info_free(info);

            let args = rs_match_args_new();
            assert!(!args.is_null());
            rs_match_args_free(args);

            let state = rs_search_state_new();
            assert!(!state.is_null());
            rs_search_state_free(state);
        }
    }

    // =========================================================================
    // Phase 5: find_tags entry point tests
    // =========================================================================

    #[test]
    fn test_find_tags_context_default() {
        let ctx = FindTagsContext::default();
        assert!(ctx.pat.is_null());
        assert_eq!(ctx.pat_len, 0);
        assert_eq!(ctx.flags, 0);
        assert!(!ctx.findall);
        assert!(!ctx.has_re);
        assert_eq!(ctx.round, 1);
    }

    #[test]
    fn test_find_tags_context_alloc_free() {
        unsafe {
            let ctx = rs_find_tags_context_new();
            assert!(!ctx.is_null());
            rs_find_tags_context_free(ctx);
        }
    }

    #[test]
    fn test_find_tags_init() {
        unsafe {
            let mut ctx = FindTagsContext::default();
            let pat = c"test".as_ptr();
            rs_find_tags_init(
                std::ptr::addr_of_mut!(ctx),
                pat,
                4,
                find_tags_flags::TAG_REGEXP | find_tags_flags::TAG_NOIC,
                10,
            );

            assert_eq!(ctx.pat, pat);
            assert_eq!(ctx.pat_len, 4);
            assert!(ctx.has_re);
            assert!(ctx.noic);
            assert!(!ctx.findall);
        }
    }

    #[test]
    fn test_find_tags_findall() {
        unsafe {
            let mut ctx = FindTagsContext::default();
            let pat = c"test".as_ptr();

            // MAXCOL should set findall
            rs_find_tags_init(std::ptr::addr_of_mut!(ctx), pat, 4, 0, 0x7FFF_FFFF);
            assert!(ctx.findall);

            // TAG_MANY should set findall
            rs_find_tags_init(std::ptr::addr_of_mut!(ctx), pat, 4, 0, 10000);
            assert!(ctx.findall);

            // Normal mincount should not set findall
            rs_find_tags_init(std::ptr::addr_of_mut!(ctx), pat, 4, 0, 5);
            assert!(!ctx.findall);
        }
    }

    #[test]
    fn test_find_tags_should_linear() {
        unsafe {
            let mut ctx = FindTagsContext::default();
            ctx.headlen = 5;
            ctx.round = 1;

            // With tagbsearch on and headlen > 0 and round 1: no linear
            assert!(!rs_find_tags_should_linear(std::ptr::addr_of!(ctx), true));

            // With headlen 0: linear
            ctx.headlen = 0;
            assert!(rs_find_tags_should_linear(std::ptr::addr_of!(ctx), true));

            // With tagbsearch off: linear
            ctx.headlen = 5;
            assert!(rs_find_tags_should_linear(std::ptr::addr_of!(ctx), false));

            // Round 2: linear
            ctx.round = 2;
            assert!(rs_find_tags_should_linear(std::ptr::addr_of!(ctx), true));
        }
    }

    #[test]
    fn test_find_tags_need_another_round() {
        unsafe {
            let mut ctx = FindTagsContext::default();
            ctx.linear = false;
            ctx.noic = false;

            // Need another round: not linear, p_ic or no noic, not rm_ic
            assert!(rs_find_tags_need_another_round(
                std::ptr::addr_of!(ctx),
                true,
                false
            ));

            // Already linear: no more rounds
            ctx.linear = true;
            assert!(!rs_find_tags_need_another_round(
                std::ptr::addr_of!(ctx),
                true,
                false
            ));

            // TAG_NOIC used and ignorecase off: no more rounds
            ctx.linear = false;
            ctx.noic = true;
            assert!(!rs_find_tags_need_another_round(
                std::ptr::addr_of!(ctx),
                false,
                false
            ));

            // Already did case-insensitive: no more rounds
            ctx.noic = false;
            assert!(!rs_find_tags_need_another_round(
                std::ptr::addr_of!(ctx),
                true,
                true
            ));
        }
    }

    #[test]
    fn test_find_tags_calc_rm_ic() {
        unsafe {
            let mut ctx = FindTagsContext::default();
            ctx.findall = false;
            ctx.headlen = 5;
            ctx.noic = false;

            // p_ic on, p_tbs on, headlen > 0, not findall: rm_ic false
            assert!(!rs_find_tags_calc_rm_ic(
                std::ptr::addr_of!(ctx),
                true,
                true
            ));

            // findall: rm_ic true (if p_ic or not noic)
            ctx.findall = true;
            assert!(rs_find_tags_calc_rm_ic(std::ptr::addr_of!(ctx), true, true));

            // headlen 0: rm_ic true
            ctx.findall = false;
            ctx.headlen = 0;
            assert!(rs_find_tags_calc_rm_ic(std::ptr::addr_of!(ctx), true, true));

            // p_tbs off: rm_ic true
            ctx.headlen = 5;
            assert!(rs_find_tags_calc_rm_ic(
                std::ptr::addr_of!(ctx),
                true,
                false
            ));
        }
    }

    #[test]
    fn test_extract_help_lang() {
        unsafe {
            // Pattern with @en at end
            assert_eq!(rs_find_tags_extract_help_lang(c"foo@en".as_ptr(), 6), 3);

            // Pattern with @ja at end
            assert_eq!(rs_find_tags_extract_help_lang(c"help@ja".as_ptr(), 7), 4);

            // Pattern too short
            assert_eq!(rs_find_tags_extract_help_lang(c"@en".as_ptr(), 3), 0);

            // No @ pattern
            assert_eq!(rs_find_tags_extract_help_lang(c"foobar".as_ptr(), 6), 0);

            // Non-alpha after @
            assert_eq!(rs_find_tags_extract_help_lang(c"foo@12".as_ptr(), 6), 0);
        }
    }

    #[test]
    fn test_get_help_lang() {
        unsafe {
            let mut lang = [0i8; 4];

            // Valid pattern
            assert!(rs_find_tags_get_help_lang(
                c"help@ja".as_ptr(),
                7,
                lang.as_mut_ptr()
            ));
            assert_eq!(lang[0] as u8, b'j');
            assert_eq!(lang[1] as u8, b'a');
            assert_eq!(lang[2], 0);

            // Invalid pattern
            assert!(!rs_find_tags_get_help_lang(
                c"help".as_ptr(),
                4,
                lang.as_mut_ptr()
            ));
        }
    }

    #[test]
    fn test_lang_from_fname() {
        unsafe {
            let mut lang = [0i8; 4];

            // tags-ja format
            assert!(rs_find_tags_lang_from_fname(
                c"doc/tags-ja".as_ptr(),
                lang.as_mut_ptr()
            ));
            assert_eq!(lang[0] as u8, b'j');
            assert_eq!(lang[1] as u8, b'a');

            // plain tags (defaults to en)
            assert!(rs_find_tags_lang_from_fname(
                c"doc/tags".as_ptr(),
                lang.as_mut_ptr()
            ));
            assert_eq!(lang[0] as u8, b'e');
            assert_eq!(lang[1] as u8, b'n');
        }
    }

    #[test]
    fn test_help_lang_matches() {
        unsafe {
            assert!(rs_help_lang_matches(c"en".as_ptr(), c"en".as_ptr()));
            assert!(rs_help_lang_matches(c"EN".as_ptr(), c"en".as_ptr()));
            assert!(rs_help_lang_matches(c"ja".as_ptr(), c"JA".as_ptr()));
            assert!(!rs_help_lang_matches(c"en".as_ptr(), c"ja".as_ptr()));
        }
    }

    #[test]
    fn test_calc_match_type() {
        // Global, current file, ignore case
        let mtype = rs_calc_match_type(false, true, false, false);
        assert_eq!(mtype, match_type::MT_GL_IC_CUR);

        // Static, other file, ignore case
        let mtype = rs_calc_match_type(true, false, false, false);
        assert_eq!(mtype, match_type::MT_ST_IC_OTH);

        // Global, other file, no ignore case
        let mtype = rs_calc_match_type(false, false, false, true);
        assert_eq!(mtype, match_type::MT_GL_NO_IC_OTH);

        // Static, current file, no ignore case
        let mtype = rs_calc_match_type(true, true, false, true);
        assert_eq!(mtype, match_type::MT_ST_NO_IC_CUR);

        // With regexp: adds MT_RE_OFF
        let mtype = rs_calc_match_type(false, true, true, false);
        assert_eq!(mtype, match_type::MT_GL_IC_CUR + match_type::MT_RE_OFF);
    }

    #[test]
    fn test_match_type_is_current_file() {
        assert!(rs_match_type_is_current_file(match_type::MT_GL_IC_CUR));
        assert!(rs_match_type_is_current_file(match_type::MT_ST_IC_CUR));
        assert!(rs_match_type_is_current_file(match_type::MT_GL_NO_IC_CUR));
        assert!(rs_match_type_is_current_file(match_type::MT_ST_NO_IC_CUR));

        assert!(!rs_match_type_is_current_file(match_type::MT_GL_IC_OTH));
        assert!(!rs_match_type_is_current_file(match_type::MT_ST_IC_OTH));
    }

    #[test]
    fn test_match_type_is_static() {
        assert!(rs_match_type_is_static(match_type::MT_ST_IC_OTH));
        assert!(rs_match_type_is_static(match_type::MT_ST_IC_CUR));
        assert!(rs_match_type_is_static(match_type::MT_ST_NO_IC_OTH));
        assert!(rs_match_type_is_static(match_type::MT_ST_NO_IC_CUR));

        assert!(!rs_match_type_is_static(match_type::MT_GL_IC_OTH));
        assert!(!rs_match_type_is_static(match_type::MT_GL_IC_CUR));
    }

    #[test]
    fn test_tag_match_cmp() {
        unsafe {
            // Different match types: compare by type
            let cmp = rs_tag_match_cmp(
                match_type::MT_GL_IC_OTH,
                match_type::MT_ST_IC_OTH,
                c"a".as_ptr(),
                c"b".as_ptr(),
            );
            assert!(cmp < 0);

            // Same match type: compare by name
            let cmp = rs_tag_match_cmp(
                match_type::MT_GL_IC_OTH,
                match_type::MT_GL_IC_OTH,
                c"alpha".as_ptr(),
                c"beta".as_ptr(),
            );
            assert!(cmp < 0);

            let cmp = rs_tag_match_cmp(
                match_type::MT_GL_IC_OTH,
                match_type::MT_GL_IC_OTH,
                c"beta".as_ptr(),
                c"alpha".as_ptr(),
            );
            assert!(cmp > 0);
        }
    }
}
