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
    /// Verbose message
    pub const TAG_VERBOSE: c_int = 32;
    /// Currently doing insert completion
    pub const TAG_INS_COMP: c_int = 64;
    /// Keep language for tag search
    pub const TAG_KEEP_LANG: c_int = 128;
    /// Don't call tagfunc
    pub const TAG_NO_TAGFUNC: c_int = 256;
}

/// MAXCOL constant for findall
const MAXCOL: c_int = 0x7FFF_FFFF;
/// TAG_MANY constant for finding multiple matches
const TAG_MANY: c_int = 300;

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
    // Fine-grained orgpat initialization accessors
    fn nvim_findtags_alloc_orgpat(st: FindTagsStateHandle);
    fn nvim_findtags_clear_orgpat_regprog(st: FindTagsStateHandle);
    // Fine-grained field setters
    fn nvim_findtags_set_flags(st: FindTagsStateHandle, flags: c_int);
    fn nvim_findtags_set_help_only_from_flags(st: FindTagsStateHandle, flags: c_int);
    fn nvim_findtags_set_mincount(st: FindTagsStateHandle, mincount: c_int);
    fn nvim_findtags_alloc_lbuf(st: FindTagsStateHandle);
    // Fine-grained free accessors
    fn nvim_findtags_free_tag_fname(st: FindTagsStateHandle);
    fn nvim_findtags_free_lbuf(st: FindTagsStateHandle);
    fn nvim_findtags_free_orgpat_regprog(st: FindTagsStateHandle);
    fn nvim_findtags_free_orgpat(st: FindTagsStateHandle);
    // Match array init (keeps C macro loop)
    fn nvim_findtags_init_match_arrays(st: FindTagsStateHandle);
    fn nvim_findtags_matchargs_init(margs: FindTagsMatchArgsHandle, flags: c_int);
}

/// Initialize a `findtags_state_T` struct for a tag search.
///
/// The struct must have been zero-initialized (via xcalloc) before calling this.
///
/// # Safety
///
/// - `st` must be a valid pointer to a zero-initialized `findtags_state_T` struct
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
    // Allocate tag_fname buffer and set fp = NULL (xcalloc already NULL-ed it)
    nvim_findtags_init_tag_fname(st);
    nvim_findtags_set_fp_null(st);

    // Initialize orgpat (inline of nvim_findtags_init_orgpat)
    nvim_findtags_alloc_orgpat(st);
    nvim_findtags_set_orgpat_pat(st, pat);
    nvim_findtags_set_orgpat_len(st, strlen(pat) as c_int);
    nvim_findtags_clear_orgpat_regprog(st);

    // Set scalar fields (inline of nvim_findtags_set_fields)
    // xcalloc zeroed: tag_file_sorted, help_lang_find, is_txt, did_open,
    //                 get_searchpat, help_lang[0], help_pri, match_count, stop_searching
    nvim_findtags_set_flags(st, flags);
    nvim_findtags_set_help_only_from_flags(st, flags);
    nvim_findtags_set_mincount(st, mincount);
    nvim_findtags_alloc_lbuf(st);

    // Initialize match arrays
    nvim_findtags_init_match_arrays(st);
}

/// Free the inner resources of a `findtags_state_T` struct.
///
/// Inlines the logic of `nvim_findtags_state_free_inner` using fine-grained
/// accessors: frees tag_fname, lbuf, orgpat->regmatch.regprog, and orgpat.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_state_free(st: FindTagsStateHandle) {
    if st.is_null() {
        return;
    }
    nvim_findtags_free_tag_fname(st);
    nvim_findtags_free_lbuf(st);
    nvim_findtags_free_orgpat_regprog(st);
    nvim_findtags_free_orgpat(st);
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
// Phase 4: Search state machine — file reading and header parsing
// =============================================================================

extern "C" {
    // findtags_state_T field accessors
    fn nvim_findtags_get_state(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_set_state_val(st: FindTagsStateHandle, state: c_int);
    fn nvim_findtags_get_lbuf(st: FindTagsStateHandle) -> *mut c_char;
    fn nvim_findtags_get_lbuf_size(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_set_lbuf(st: FindTagsStateHandle, lbuf: *mut c_char, lbuf_size: c_int);

    // File I/O through findtags_state_T
    fn nvim_findtags_fgets(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_fseek(st: FindTagsStateHandle, offset: i64, whence: c_int) -> c_int;
    fn nvim_findtags_ftell(st: FindTagsStateHandle) -> i64;
    fn nvim_findtags_fseek_zero(st: FindTagsStateHandle);
    fn nvim_findtags_lbuf_is_blank(st: FindTagsStateHandle) -> bool;

    // Other field accessors
    fn nvim_findtags_get_flags(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_get_linear(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_set_linear(st: FindTagsStateHandle, linear: bool);
    fn nvim_findtags_get_sorted(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_set_sorted(st: FindTagsStateHandle, val: c_int);
    fn nvim_findtags_get_orgpat_rm_ic(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_set_orgpat_rm_ic(st: FindTagsStateHandle, ic: bool);
    fn nvim_get_p_ic() -> c_int;

    // Encoding conversion
    fn nvim_findtags_convert_setup(st: FindTagsStateHandle, from: *const c_char);
    fn nvim_findtags_string_convert(st: FindTagsStateHandle) -> *mut c_char;

    // Memory
    fn xfree(ptr: *mut c_void);
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strlen(s: *const c_char) -> usize;
}

/// C `tagsearch_state_T` enum values
const TS_LINEAR: c_int = 1;
const TS_BINARY: c_int = 2;
const TS_SKIP_BACK: c_int = 3;
const TS_STEP_FORWARD: c_int = 4;

/// C tags_read_status_T enum values
const TAGS_READ_SUCCESS: c_int = 1;
const TAGS_READ_EOF: c_int = 2;
const TAGS_READ_IGNORE: c_int = 3;

/// SEEK_SET and SEEK_END
const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;

/// TAG_NOIC flag
const TAG_NOIC: c_int = 8;

/// Read the next line from a tags file during search.
///
/// Handles binary search seeking, linear reading, and blank line skipping.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T`
/// - `sinfo_p` must be a valid pointer to a `tagsearch_info_T`
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_get_next_line(
    st: FindTagsStateHandle,
    sinfo_p: *mut TagSearchInfo,
) -> c_int {
    if st.is_null() || sinfo_p.is_null() {
        return TAGS_READ_EOF;
    }

    let sinfo = &mut *sinfo_p;
    let state = nvim_findtags_get_state(st);

    // For binary search: compute the next offset to use.
    if state == TS_BINARY {
        let offset = sinfo.low_offset + ((sinfo.high_offset - sinfo.low_offset) / 2);
        if offset == sinfo.curr_offset {
            return TAGS_READ_EOF; // End the binary search without a match.
        }
        sinfo.curr_offset = offset;
    } else if state == TS_SKIP_BACK {
        // Skipping back (after a match during binary search).
        let lbuf_size = nvim_findtags_get_lbuf_size(st) as i64;
        sinfo.curr_offset -= lbuf_size * 2;
        if sinfo.curr_offset < 0 {
            sinfo.curr_offset = 0;
            nvim_findtags_fseek_zero(st);
            nvim_findtags_set_state_val(st, TS_STEP_FORWARD);
        }
    }

    let state = nvim_findtags_get_state(st);

    // When jumping around in the file, first read a line to find the
    // start of the next line.
    if state == TS_BINARY || state == TS_SKIP_BACK {
        // Adjust the search file offset to the correct position
        sinfo.curr_offset_used = sinfo.curr_offset;
        let _ = nvim_findtags_fseek(st, sinfo.curr_offset, SEEK_SET);
        let mut eof = nvim_findtags_fgets(st);
        if !eof && sinfo.curr_offset != 0 {
            sinfo.curr_offset = nvim_findtags_ftell(st);
            if sinfo.curr_offset == sinfo.high_offset {
                // oops, gone a bit too far; try from low offset
                let _ = nvim_findtags_fseek(st, sinfo.low_offset, SEEK_SET);
                sinfo.curr_offset = sinfo.low_offset;
            }
            eof = nvim_findtags_fgets(st);
        }
        // skip empty and blank lines
        while !eof && nvim_findtags_lbuf_is_blank(st) {
            sinfo.curr_offset = nvim_findtags_ftell(st);
            eof = nvim_findtags_fgets(st);
        }
        if eof {
            // Hit end of file.  Skip backwards.
            nvim_findtags_set_state_val(st, TS_SKIP_BACK);
            sinfo.match_offset = nvim_findtags_ftell(st);
            sinfo.curr_offset = sinfo.curr_offset_used;
            return TAGS_READ_IGNORE;
        }
    } else {
        // Not jumping around in the file: Read the next line.
        // skip empty and blank lines
        let mut eof = nvim_findtags_fgets(st);
        while !eof && nvim_findtags_lbuf_is_blank(st) {
            eof = nvim_findtags_fgets(st);
        }
        if eof {
            return TAGS_READ_EOF;
        }
    }

    TAGS_READ_SUCCESS
}

/// Parse a tags file header line in `st->lbuf`.
///
/// Returns true if the current line is not a tags header line and should be
/// parsed as a regular tag line. Returns false if the line is a header and
/// the next header line should be read.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T`
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_hdr_parse(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return true;
    }

    let lbuf = nvim_findtags_get_lbuf(st);
    if lbuf.is_null() {
        return true;
    }

    // Header lines in a tags file start with "!_TAG_"
    if strncmp(lbuf, c"!_TAG_".as_ptr(), 6) != 0 {
        // Non-header item before the header, e.g. "!" itself.
        return true;
    }

    // Process the header line.
    if strncmp(lbuf, c"!_TAG_FILE_SORTED\t".as_ptr(), 18) == 0 {
        nvim_findtags_set_sorted(st, c_int::from(*lbuf.add(18) as u8));
    }
    if strncmp(lbuf, c"!_TAG_FILE_ENCODING\t".as_ptr(), 20) == 0 {
        // Prepare to convert every line from the specified encoding to
        // 'encoding'.
        let mut p = lbuf.add(20);
        while *p > b' ' as c_char && (*p as u8) < 127 {
            p = p.add(1);
        }
        *p = 0;
        nvim_findtags_convert_setup(st, lbuf.add(20));
    }

    // Read the next line.  Unrecognized flags are ignored.
    false
}

/// Handler to initialize the state when starting to process a new tags file.
///
/// Called in the TS_START state when finding tags from a tags file.
/// Returns true if the line read from the tags file should be parsed and
/// false if the line should be ignored.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T`
/// - `sortic` must be a valid pointer to a bool
/// - `sinfo_p` must be a valid pointer to a `tagsearch_info_T`
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_start_state_handler(
    st: FindTagsStateHandle,
    sortic: *mut bool,
    sinfo_p: *mut TagSearchInfo,
) -> bool {
    if st.is_null() || sortic.is_null() || sinfo_p.is_null() {
        return true;
    }

    let sinfo = &mut *sinfo_p;
    let flags = nvim_findtags_get_flags(st);
    let noic = (flags & TAG_NOIC) != 0;
    let lbuf = nvim_findtags_get_lbuf(st);

    // The header ends when the line sorts below "!_TAG_".  When case is
    // folded lower case letters sort before "_".
    if strncmp(lbuf, c"!_TAG_".as_ptr(), 6) <= 0
        || (*lbuf == b'!' as c_char && (*lbuf.add(1) as u8).is_ascii_lowercase())
    {
        return rs_findtags_hdr_parse(st);
    }

    // Headers ends.

    let linear = nvim_findtags_get_linear(st);
    let tag_file_sorted = nvim_findtags_get_sorted(st);

    // When there is no tag head, or ignoring case, need to do a linear search.
    // When no "!_TAG_" is found, default to binary search.
    // When "!_TAG_FILE_SORTED" found: start binary search if flag set.
    if linear {
        nvim_findtags_set_state_val(st, TS_LINEAR);
    } else if tag_file_sorted == 0 {
        // NUL
        nvim_findtags_set_state_val(st, TS_BINARY);
    } else if tag_file_sorted == i32::from(b'1') {
        nvim_findtags_set_state_val(st, TS_BINARY);
    } else if tag_file_sorted == i32::from(b'2') {
        nvim_findtags_set_state_val(st, TS_BINARY);
        *sortic = true;
        nvim_findtags_set_orgpat_rm_ic(st, nvim_get_p_ic() != 0 || !noic);
    } else {
        nvim_findtags_set_state_val(st, TS_LINEAR);
    }

    let state = nvim_findtags_get_state(st);
    if state == TS_BINARY && nvim_findtags_get_orgpat_rm_ic(st) && !*sortic {
        // Binary search won't work for ignoring case, use linear search.
        nvim_findtags_set_linear(st, true);
        nvim_findtags_set_state_val(st, TS_LINEAR);
    }

    // When starting a binary search, get the size of the file and
    // compute the first offset.
    let state = nvim_findtags_get_state(st);
    if state == TS_BINARY {
        if nvim_findtags_fseek(st, 0, SEEK_END) != 0 {
            // can't seek, don't use binary search
            nvim_findtags_set_state_val(st, TS_LINEAR);
        } else {
            // Get the tag file size.
            let filesize = nvim_findtags_ftell(st);
            let _ = nvim_findtags_fseek(st, 0, SEEK_SET);

            // Calculate the first read offset in the file.
            sinfo.low_offset = 0;
            sinfo.low_char = 0;
            sinfo.high_offset = filesize;
            sinfo.curr_offset = 0;
            sinfo.high_char = 0xff;
        }
        return false;
    }

    true
}

/// Convert the current line in `st->lbuf` using the encoding conversion
/// set up by the `!_TAG_FILE_ENCODING` header.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T`
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_string_convert(st: FindTagsStateHandle) {
    if st.is_null() {
        return;
    }

    let conv_line = nvim_findtags_string_convert(st);
    if conv_line.is_null() {
        return;
    }

    // Copy or swap lbuf and conv_line.
    let len = strlen(conv_line) as c_int + 1;
    let lbuf_size = nvim_findtags_get_lbuf_size(st);
    let lbuf = nvim_findtags_get_lbuf(st);

    if len > lbuf_size {
        xfree(lbuf.cast::<c_void>());
        nvim_findtags_set_lbuf(st, conv_line, len);
    } else {
        strcpy(lbuf, conv_line);
        xfree(conv_line.cast::<c_void>());
    }
}

// =============================================================================
// Phase 5: Search state machine — line parsing and matching
// =============================================================================

/// Opaque handle to `tagptrs_T`
type TagPtrsHandle = *mut c_void;

extern "C" {
    // orgpat field accessors
    fn nvim_findtags_get_orgpat_headlen(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_get_orgpat_head(st: FindTagsStateHandle) -> *const c_char;
    fn nvim_findtags_get_orgpat_pat(st: FindTagsStateHandle) -> *const c_char;
    fn nvim_findtags_get_orgpat_len(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_has_regprog(st: FindTagsStateHandle) -> bool;

    // State field accessors (const)
    fn nvim_findtags_get_help_only(st: FindTagsStateHandle) -> bool;

    // Regex operations
    fn nvim_findtags_vim_regexec(st: FindTagsStateHandle, tagname: *const c_char) -> bool;
    fn nvim_findtags_get_regmatch_startoff(
        st: FindTagsStateHandle,
        tagname: *const c_char,
    ) -> c_int;

    // Multi-byte string comparison
    fn nvim_mb_strnicmp(s1: *const c_char, s2: *const c_char, len: usize) -> c_int;

    // Tag file state accessors
    fn nvim_findtags_get_tag_fname(st: FindTagsStateHandle) -> *const c_char;
    fn nvim_findtags_get_help_lang(st: FindTagsStateHandle) -> *const c_char;
    fn nvim_findtags_get_help_pri(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_get_searchpat(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_set_searchpat(st: FindTagsStateHandle, val: bool);

    // Match count
    fn nvim_findtags_get_match_count(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_inc_match_count(st: FindTagsStateHandle);
    fn nvim_findtags_set_match_count(st: FindTagsStateHandle, count: c_int);

    // Global state accessors
    fn nvim_get_current_State() -> c_int;
    fn nvim_get_p_sft() -> bool;
    fn nvim_get_p_tl() -> i64;

    // Help heuristic
    fn nvim_help_heuristic(tagname: *const c_char, match_offset: c_int, wrong_case: bool) -> c_int;

    // Match entry operations
    fn nvim_findtags_add_match_entry(
        st: FindTagsStateHandle,
        mtt: c_int,
        mfp: *mut c_char,
        hash: *mut usize,
    ) -> bool;
    fn nvim_findtags_ga_match_len(st: FindTagsStateHandle, mtt: c_int) -> c_int;
    fn nvim_findtags_ga_match_get(st: FindTagsStateHandle, mtt: c_int, idx: c_int) -> *mut c_char;
    fn nvim_findtags_clear_match(st: FindTagsStateHandle, mtt: c_int);

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xmemcpyz(dst: *mut c_char, src: *const c_char, len: usize);

    // Existing Rust functions we call across crate boundary
    fn rs_parse_tag_line(lbuf: *mut c_char, tagp: *mut c_void) -> c_int;
    fn rs_tag_strnicmp(s1: *const c_char, s2: *const c_char, len: usize) -> c_int;
    fn rs_test_for_current(
        fname: *const c_char,
        fname_end: *const c_char,
        tag_fname: *const c_char,
        buf_ffname: *const c_char,
    ) -> bool;
    fn rs_test_for_static(tagp: *const c_void) -> bool;

    // String operations
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
}

/// TAG_MATCH_SUCCESS etc. — tagmatch_status_T enum values
const TAG_MATCH_SUCCESS: c_int = 1;
const TAG_MATCH_FAIL: c_int = 2;
const TAG_MATCH_STOP: c_int = 3;
const TAG_MATCH_NEXT: c_int = 4;

/// TAG_REGEXP flag
const TAG_REGEXP: c_int = 4;
/// TAG_NAMES flag
const TAG_NAMES: c_int = 2;

/// TAB character
const TAB: c_int = 0x09;
/// TAG_SEP character (0x02)
const TAG_SEP: u8 = 0x02;

/// MT_* match type constants
const MT_ST_CUR: c_int = 0;
const MT_GL_CUR: c_int = 1;
const MT_GL_OTH: c_int = 2;
const MT_ST_OTH: c_int = 3;
const MT_IC_OFF: c_int = 4;
const MT_RE_OFF: c_int = 8;

/// MODE_INSERT
const MODE_INSERT: c_int = 0x10;

/// TOUPPER_ASC macro equivalent
fn toupper_asc(c: u8) -> u8 {
    if c.is_ascii_lowercase() {
        c - b'a' + b'A'
    } else {
        c
    }
}

/// Handle binary search state in `findtags_parse_line`.
///
/// Returns `Some(status)` if the function should return early,
/// or `None` if matching should continue.
#[allow(clippy::too_many_arguments)]
unsafe fn parse_line_check_state(
    st: FindTagsStateHandle,
    tagpp: *mut TagPtrsFields,
    margs: &mut FindTagsMatchArgs,
    sinfo: &mut TagSearchInfo,
    state: c_int,
    cmplen: c_int,
    headlen: c_int,
    head: *const c_char,
) -> Option<c_int> {
    if state == TS_BINARY {
        let first_byte = *(*tagpp).tagname as u8;
        let i = if margs.sortic {
            toupper_asc(first_byte)
        } else {
            first_byte
        };
        if (i as c_int) < sinfo.low_char || (i as c_int) > sinfo.high_char {
            margs.sort_error = true;
        }

        let tagcmp = if margs.sortic {
            rs_tag_strnicmp((*tagpp).tagname, head, cmplen as usize)
        } else {
            strncmp((*tagpp).tagname, head, cmplen as usize)
        };
        let tagcmp = if tagcmp == 0 {
            match cmplen.cmp(&headlen) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        } else {
            tagcmp
        };

        if tagcmp == 0 {
            nvim_findtags_set_state_val(st, TS_SKIP_BACK);
            sinfo.match_offset = sinfo.curr_offset;
            return Some(TAG_MATCH_NEXT);
        }
        if tagcmp < 0 {
            sinfo.curr_offset = nvim_findtags_ftell(st);
            if sinfo.curr_offset < sinfo.high_offset {
                sinfo.low_offset = sinfo.curr_offset;
                sinfo.low_char = if margs.sortic {
                    toupper_asc(first_byte) as c_int
                } else {
                    first_byte as c_int
                };
                return Some(TAG_MATCH_NEXT);
            }
        }
        if tagcmp > 0 && sinfo.curr_offset != sinfo.high_offset {
            sinfo.high_offset = sinfo.curr_offset;
            sinfo.high_char = if margs.sortic {
                toupper_asc(first_byte) as c_int
            } else {
                first_byte as c_int
            };
            return Some(TAG_MATCH_NEXT);
        }
        return Some(TAG_MATCH_STOP);
    } else if state == TS_SKIP_BACK {
        if nvim_mb_strnicmp((*tagpp).tagname, head, cmplen as usize) != 0 {
            nvim_findtags_set_state_val(st, TS_STEP_FORWARD);
        } else {
            sinfo.curr_offset = sinfo.curr_offset_used;
        }
        return Some(TAG_MATCH_NEXT);
    } else if state == TS_STEP_FORWARD {
        if nvim_mb_strnicmp((*tagpp).tagname, head, cmplen as usize) != 0 {
            return Some(if nvim_findtags_ftell(st) > sinfo.match_offset {
                TAG_MATCH_STOP
            } else {
                TAG_MATCH_NEXT
            });
        }
    } else if nvim_mb_strnicmp((*tagpp).tagname, head, cmplen as usize) != 0 {
        return Some(TAG_MATCH_NEXT);
    }
    None
}

/// Parse a tag line read from a tags file.
///
/// Also compares the tag name in `tagpp->tagname` with a search pattern in
/// `st->orgpat->head` as a quick check if the tag may match.
///
/// Returns:
/// - `TAG_MATCH_SUCCESS` if the tag may match
/// - `TAG_MATCH_FAIL` if the tag doesn't match
/// - `TAG_MATCH_NEXT` to look for the next matching tag (used in a binary search)
/// - `TAG_MATCH_STOP` if all the tags are processed without a match.
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_parse_line(
    st: FindTagsStateHandle,
    tagpp: TagPtrsHandle,
    margs: *mut FindTagsMatchArgs,
    sinfo_p: *mut TagSearchInfo,
) -> c_int {
    if st.is_null() || tagpp.is_null() || margs.is_null() || sinfo_p.is_null() {
        return TAG_MATCH_FAIL;
    }

    let margs = &mut *margs;
    let sinfo = &mut *sinfo_p;
    let headlen = nvim_findtags_get_orgpat_headlen(st);
    let lbuf = nvim_findtags_get_lbuf(st);
    let tagpp_typed = tagpp.cast::<TagPtrsFields>();

    let status;

    if headlen != 0 {
        std::ptr::write_bytes(tagpp_typed, 0, 1);
        (*tagpp_typed).tagname = lbuf;
        (*tagpp_typed).tagname_end = vim_strchr(lbuf, TAB);
        if (*tagpp_typed).tagname_end.is_null() {
            return TAG_MATCH_FAIL;
        }

        let mut cmplen = (*tagpp_typed)
            .tagname_end
            .offset_from((*tagpp_typed).tagname) as c_int;
        let p_tl = nvim_get_p_tl();
        if p_tl != 0 && (cmplen as i64) > p_tl {
            cmplen = p_tl as c_int;
        }

        let flags = nvim_findtags_get_flags(st);
        let state = nvim_findtags_get_state(st);

        if (flags & TAG_REGEXP) != 0 && headlen < cmplen {
            cmplen = headlen;
        } else if state == TS_LINEAR && headlen != cmplen {
            return TAG_MATCH_NEXT;
        }

        let head = nvim_findtags_get_orgpat_head(st);

        if let Some(result) =
            parse_line_check_state(st, tagpp_typed, margs, sinfo, state, cmplen, headlen, head)
        {
            return result;
        }

        // Can be a matching tag, isolate the file name and command.
        (*tagpp_typed).fname = (*tagpp_typed).tagname_end.add(1);
        (*tagpp_typed).fname_end = vim_strchr((*tagpp_typed).fname, TAB);
        if (*tagpp_typed).fname_end.is_null() {
            status = 0; // FAIL
        } else {
            (*tagpp_typed).command = (*tagpp_typed).fname_end.add(1);
            status = 1; // OK
        }
    } else {
        status = rs_parse_tag_line(lbuf, tagpp);
    }

    if status == 0 {
        TAG_MATCH_FAIL
    } else {
        TAG_MATCH_SUCCESS
    }
}

/// Compares the tag name in `tagpp->tagname` with a search pattern in
/// `st->orgpat->pat`.
/// Returns true if the tag matches, false if the tag doesn't match.
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_match_tag(
    st: FindTagsStateHandle,
    tagpp: TagPtrsHandle,
    margs: *mut FindTagsMatchArgs,
) -> bool {
    if st.is_null() || tagpp.is_null() || margs.is_null() {
        return false;
    }

    let margs = &mut *margs;
    let tagpp_typed = tagpp.cast::<TagPtrsFields>();

    // First try matching with the pattern literally (also when it is a regexp).
    let mut cmplen = (*tagpp_typed)
        .tagname_end
        .offset_from((*tagpp_typed).tagname) as c_int;
    let p_tl = nvim_get_p_tl();
    if p_tl != 0 && (cmplen as i64) > p_tl {
        cmplen = p_tl as c_int;
    }

    let orgpat_len = nvim_findtags_get_orgpat_len(st);

    // if tag length does not match, don't try comparing
    let mut matched = if orgpat_len == cmplen {
        let rm_ic = nvim_findtags_get_orgpat_rm_ic(st);
        let pat = nvim_findtags_get_orgpat_pat(st);

        if rm_ic {
            let m = nvim_mb_strnicmp((*tagpp_typed).tagname, pat, cmplen as usize) == 0;
            if m {
                margs.match_no_ic = strncmp((*tagpp_typed).tagname, pat, cmplen as usize) == 0;
            }
            m
        } else {
            strncmp((*tagpp_typed).tagname, pat, cmplen as usize) == 0
        }
    } else {
        false
    };

    // Has a regexp: Also find tags matching regexp.
    margs.match_re = false;
    if !matched && nvim_findtags_has_regprog(st) {
        let cc = *(*tagpp_typed).tagname_end;
        *(*tagpp_typed).tagname_end = 0; // NUL

        matched = nvim_findtags_vim_regexec(st, (*tagpp_typed).tagname);
        if matched {
            margs.matchoff = nvim_findtags_get_regmatch_startoff(st, (*tagpp_typed).tagname);
            if nvim_findtags_get_orgpat_rm_ic(st) {
                nvim_findtags_set_orgpat_rm_ic(st, false);
                margs.match_no_ic = nvim_findtags_vim_regexec(st, (*tagpp_typed).tagname);
                nvim_findtags_set_orgpat_rm_ic(st, true);
            }
        }
        *(*tagpp_typed).tagname_end = cc;
        margs.match_re = true;
    }

    matched
}

/// Build the match string for a help tag match.
unsafe fn build_help_match(
    st: FindTagsStateHandle,
    tagpp: *mut TagPtrsFields,
    margs: &FindTagsMatchArgs,
) -> *mut c_char {
    const ML_EXTRA: usize = 3;
    *(*tagpp).tagname_end = 0; // NUL
    let len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as usize;
    let mfp_size = 1 + len + 10 + ML_EXTRA + 1;
    let mfp = xmalloc(mfp_size).cast::<c_char>();

    xmemcpyz(mfp, (*tagpp).tagname, len);
    *mfp.add(len) = b'@' as c_char;
    let help_lang = nvim_findtags_get_help_lang(st);
    xmemcpyz(mfp.add(len + 1), help_lang, ML_EXTRA);
    let heuristic = nvim_help_heuristic(
        (*tagpp).tagname,
        if margs.match_re { margs.matchoff } else { 0 },
        !margs.match_no_ic,
    ) + nvim_findtags_get_help_pri(st);
    let remaining = mfp_size - (len + 1 + ML_EXTRA);
    snprintf(
        mfp.add(len + 1 + ML_EXTRA),
        remaining,
        c"%06d".as_ptr(),
        heuristic,
    );
    *(*tagpp).tagname_end = TAB as c_char;
    mfp
}

/// Build the match string for a name-only tag match.
unsafe fn build_name_only_match(st: FindTagsStateHandle, tagpp: *mut TagPtrsFields) -> *mut c_char {
    if nvim_findtags_get_searchpat(st) {
        let mut temp_end = (*tagpp).command;
        if *temp_end as u8 == b'/' {
            while *temp_end != 0
                && *temp_end as u8 != b'\r'
                && *temp_end as u8 != b'\n'
                && *temp_end as u8 != b'$'
            {
                temp_end = temp_end.add(1);
            }
        }
        nvim_findtags_set_searchpat(st, false);
        if (*tagpp).command.add(2) < temp_end {
            let len = temp_end.offset_from((*tagpp).command) as usize - 2;
            let mfp = xmalloc(len + 2).cast::<c_char>();
            xmemcpyz(mfp, (*tagpp).command.add(2), len);
            mfp
        } else {
            std::ptr::null_mut()
        }
    } else {
        let len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as usize;
        let mfp = xmalloc(1 + len + 1).cast::<c_char>();
        xmemcpyz(mfp, (*tagpp).tagname, len);
        if (nvim_get_current_State() & MODE_INSERT) != 0 {
            nvim_findtags_set_searchpat(st, nvim_get_p_sft());
        }
        mfp
    }
}

/// Add a matching tag found in a tags file to `st->ht_match` and `st->ga_match`.
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_add_match(
    st: FindTagsStateHandle,
    tagpp: TagPtrsHandle,
    margs: *const FindTagsMatchArgs,
    buf_ffname: *const c_char,
    hash: *mut usize,
) {
    if st.is_null() || tagpp.is_null() || margs.is_null() {
        return;
    }

    let margs = &*margs;
    let tagpp_typed = tagpp.cast::<TagPtrsFields>();
    let flags = nvim_findtags_get_flags(st);
    let name_only = (flags & TAG_NAMES) != 0;

    let is_current = rs_test_for_current(
        (*tagpp_typed).fname,
        (*tagpp_typed).fname_end,
        nvim_findtags_get_tag_fname(st),
        buf_ffname,
    );
    let is_static = rs_test_for_static(tagpp.cast::<c_void>());

    let mut mtt = if is_static {
        if is_current {
            MT_ST_CUR
        } else {
            MT_ST_OTH
        }
    } else if is_current {
        MT_GL_CUR
    } else {
        MT_GL_OTH
    };

    if nvim_findtags_get_orgpat_rm_ic(st) && !margs.match_no_ic {
        mtt += MT_IC_OFF;
    }
    if margs.match_re {
        mtt += MT_RE_OFF;
    }

    let mfp = if nvim_findtags_get_help_only(st) {
        build_help_match(st, tagpp_typed, margs)
    } else if name_only {
        build_name_only_match(st, tagpp_typed)
    } else {
        let tag_fname = nvim_findtags_get_tag_fname(st);
        let tag_fname_len = strlen(tag_fname);
        let lbuf = nvim_findtags_get_lbuf(st);
        let len = tag_fname_len + strlen(lbuf) + 3;
        let p = xmalloc(1 + len + 1).cast::<c_char>();
        *p = (mtt + 1) as u8 as c_char;
        strcpy(p.add(1), tag_fname);
        *p.add(tag_fname_len + 1) = TAG_SEP as c_char;
        strcpy(p.add(1 + tag_fname_len + 1), lbuf);
        p
    };

    if !mfp.is_null() && !nvim_findtags_add_match_entry(st, mtt, mfp, hash) {
        xfree(mfp.cast::<c_void>());
    }
}

/// Copy the tags found by `find_tags()` to `matchesp`.
/// Returns the number of matches copied.
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_copy_matches(
    st: FindTagsStateHandle,
    matchesp: *mut *mut *mut c_char,
) -> c_int {
    if st.is_null() || matchesp.is_null() {
        return 0;
    }

    let flags = nvim_findtags_get_flags(st);
    let name_only = (flags & TAG_NAMES) != 0;
    let total_match_count = nvim_findtags_get_match_count(st);

    let matches: *mut *mut c_char = if total_match_count > 0 {
        xmalloc((total_match_count as usize) * std::mem::size_of::<*mut c_char>())
            .cast::<*mut c_char>()
    } else {
        std::ptr::null_mut()
    };

    nvim_findtags_set_match_count(st, 0);

    for mtt in 0..MT_COUNT as c_int {
        let ga_len = nvim_findtags_ga_match_len(st, mtt);
        for i in 0..ga_len {
            let mfp = nvim_findtags_ga_match_get(st, mtt, i);
            if matches.is_null() {
                xfree(mfp.cast::<c_void>());
            } else {
                if !name_only {
                    // Change mtt back to zero-based.
                    *mfp = (*mfp as u8).wrapping_sub(1) as c_char;

                    // change the TAG_SEP back to NUL
                    let mut p = mfp.add(1);
                    while *p != 0 {
                        if *p as u8 == TAG_SEP {
                            *p = 0;
                        }
                        p = p.add(1);
                    }
                }
                let idx = nvim_findtags_get_match_count(st);
                *matches.add(idx as usize) = mfp;
                nvim_findtags_inc_match_count(st);
            }
        }

        nvim_findtags_clear_match(st, mtt);
    }

    *matchesp = matches;
    nvim_findtags_get_match_count(st)
}

/// Helper struct matching the first few fields of `tagptrs_T` / `TagPtrs`
/// so we can access them from Rust without importing the parse module.
#[repr(C)]
struct TagPtrsFields {
    tagname: *mut c_char,
    tagname_end: *mut c_char,
    fname: *mut c_char,
    fname_end: *mut c_char,
    command: *mut c_char,
    command_end: *mut c_char,
    tag_fname: *mut c_char,
    tagkind: *mut c_char,
    tagkind_end: *mut c_char,
    user_data: *mut c_char,
    user_data_end: *mut c_char,
    tagline: i32, // linenr_T
}

// =============================================================================
// Phase 6: Search orchestration — per-file and top-level search
// =============================================================================

/// Constants for Phase 6
const TS_START: c_int = 0;
const CONV_NONE: c_int = 0;
const NUL_BYTE: u8 = 0;

/// OK/FAIL/NOTDONE return values (matching C)
#[allow(dead_code)]
const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;
#[allow(dead_code)]
const NOTDONE: c_int = 2;

/// kOptTcFlag values from option_vars.generated.h
#[allow(dead_code)]
const K_OPT_TC_FLAG_FOLLOWIC: c_int = 0x01;
#[allow(dead_code)]
const K_OPT_TC_FLAG_IGNORE: c_int = 0x02;
#[allow(dead_code)]
const K_OPT_TC_FLAG_MATCH: c_int = 0x04;
#[allow(dead_code)]
const K_OPT_TC_FLAG_FOLLOWSCS: c_int = 0x08;
#[allow(dead_code)]
const K_OPT_TC_FLAG_SMART: c_int = 0x10;

#[allow(dead_code)]
extern "C" {
    // Phase 6 findtags_state_T field accessors
    fn nvim_findtags_get_is_txt(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_set_is_txt(st: FindTagsStateHandle, val: bool);
    fn nvim_findtags_get_help_lang_find(st: FindTagsStateHandle) -> *const c_char;
    fn nvim_findtags_set_help_lang_find(st: FindTagsStateHandle, val: *const c_char);
    fn nvim_findtags_set_help_pri(st: FindTagsStateHandle, pri: c_int);
    fn nvim_findtags_set_help_lang(st: FindTagsStateHandle, lang: *const c_char);
    fn nvim_findtags_get_vimconv_type(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_set_vimconv_none(st: FindTagsStateHandle);
    fn nvim_findtags_convert_cleanup(st: FindTagsStateHandle);
    fn nvim_findtags_fopen(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_fclose(st: FindTagsStateHandle);
    fn nvim_findtags_set_did_open(st: FindTagsStateHandle);
    fn nvim_findtags_get_did_open(st: FindTagsStateHandle) -> bool;
    fn nvim_findtags_set_state_start(st: FindTagsStateHandle);
    fn nvim_findtags_get_mincount(st: FindTagsStateHandle) -> c_int;
    fn nvim_findtags_grow_lbuf(st: FindTagsStateHandle, sinfo: *mut c_void) -> bool;
    fn nvim_findtags_set_orgpat_len(st: FindTagsStateHandle, len: c_int);
    fn nvim_findtags_set_orgpat_pat(st: FindTagsStateHandle, pat: *mut c_char);
    fn nvim_findtags_set_stop_searching(st: FindTagsStateHandle, val: bool);
    fn nvim_findtags_get_stop_searching(st: FindTagsStateHandle) -> bool;

    // Global variable accessors
    fn nvim_set_p_ic(val: c_int);
    fn nvim_get_p_tbs() -> bool;
    fn nvim_get_tc_flags() -> c_int;
    fn nvim_get_curbuf_tc_flags() -> c_int;
    fn nvim_get_p_hlg() -> *const c_char;
    fn nvim_get_p_verbose() -> c_int;
    fn nvim_get_emsg_off() -> c_int;
    fn nvim_set_emsg_off(val: c_int);
    fn nvim_get_curbuf_b_fname() -> *const c_char;
    fn nvim_get_curbuf_p_tfu() -> *const c_char;
    fn nvim_set_curbuf_b_help(val: c_int);
    fn nvim_get_curbuf_b_help() -> c_int;

    // Function wrappers
    fn nvim_line_breakcheck();
    fn nvim_fast_breakcheck();
    fn nvim_get_got_int() -> c_int;
    fn nvim_ins_compl_check_keys(interval: c_int, pum_wanted: bool);
    fn rs_ins_compl_interrupted() -> c_int;
    fn nvim_verbose_searching_tags(tag_fname: *const c_char);
    fn nvim_ignorecase(pat: *const c_char) -> bool;
    fn nvim_ignorecase_opt(pat: *const c_char, ic_strstrp: bool, ic_strstrp2: bool) -> bool;

    // Error messages
    fn nvim_semsg_e431(tag_fname: *const c_char);
    fn nvim_semsg_before_byte(offset: i64);
    fn nvim_semsg_e432(tag_fname: *const c_char);
    fn nvim_emsg_e433();

    // tfu_in_use guard accessors
    fn nvim_tag_get_tfu_in_use() -> bool;
    fn nvim_tag_set_tfu_in_use(val: bool);
    // ga_match and match_count pointer accessors for rs_findtags_apply_tfu
    fn nvim_findtags_get_ga_match_ptr(st: FindTagsStateHandle) -> *mut c_void;
    fn nvim_findtags_get_match_count_ptr(st: FindTagsStateHandle) -> *mut c_int;

    // Pattern preparation
    fn nvim_findtags_prepare_pats(st: FindTagsStateHandle, has_re: bool);

    // Memory
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // String functions
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
}

/// Apply the tagfunc option to find tags (Rust port of nvim_findtags_apply_tfu).
///
/// Guards against recursive tagfunc calls via the `tfu_in_use` flag.
/// Returns NOTDONE if tagfunc should not be called, or the tagfunc's result.
///
/// # Safety
///
/// - `st` must be a valid `findtags_state_T` pointer
/// - `pat` and `buf_ffname` must be valid C strings (buf_ffname may be null)
unsafe fn rs_findtags_apply_tfu(
    st: FindTagsStateHandle,
    pat: *mut c_char,
    buf_ffname: *mut c_char,
) -> c_int {
    let flags = nvim_findtags_get_flags(st);
    let use_tfu = (flags & find_tags_flags::TAG_NO_TAGFUNC) == 0;

    if !use_tfu || nvim_tag_get_tfu_in_use() {
        return NOTDONE;
    }

    let p_tfu = nvim_get_curbuf_p_tfu();
    if p_tfu.is_null() || *p_tfu == 0 {
        return NOTDONE;
    }

    nvim_tag_set_tfu_in_use(true);
    let ga = nvim_findtags_get_ga_match_ptr(st);
    let match_count = nvim_findtags_get_match_count_ptr(st);
    let retval = rs_find_tagfunc_tags(pat, ga, match_count, flags, buf_ffname);
    nvim_tag_set_tfu_in_use(false);
    retval
}

/// Initialize help language and priority for a tag search.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T`
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_in_help_init(st: FindTagsStateHandle) -> bool {
    if st.is_null() {
        return false;
    }

    let tag_fname = nvim_findtags_get_tag_fname(st);
    let is_txt = nvim_findtags_get_is_txt(st);

    // Keep "en" as the language if the file extension is ".txt"
    if is_txt {
        nvim_findtags_set_help_lang(st, c"en".as_ptr());
    } else {
        // Prefer help tags according to 'helplang'. Put the two-letter
        // language name in help_lang[].
        let fname_len = strlen(tag_fname) as c_int;
        if fname_len > 3 && *tag_fname.offset((fname_len - 3) as isize) == b'-' as c_char {
            // Copy the 2-char language code from the end of the filename
            nvim_findtags_set_help_lang(st, tag_fname.offset((fname_len - 2) as isize));
        } else {
            nvim_findtags_set_help_lang(st, c"en".as_ptr());
        }
    }

    // When searching for a specific language skip tags files for other languages.
    let help_lang_find = nvim_findtags_get_help_lang_find(st);
    let help_lang = nvim_findtags_get_help_lang(st);
    if !help_lang_find.is_null() {
        // STRICMP: case-insensitive compare
        if rs_tag_strnicmp(help_lang, help_lang_find, 2) != 0 {
            return false;
        }
    }

    let flags = nvim_findtags_get_flags(st);

    // For CTRL-] in a help file prefer a match with the same language.
    let curbuf_fname = nvim_get_curbuf_b_fname();
    if (flags & find_tags_flags::TAG_KEEP_LANG) != 0
        && help_lang_find.is_null()
        && !curbuf_fname.is_null()
    {
        let fname_len = strlen(curbuf_fname) as c_int;
        if fname_len > 4
            && *curbuf_fname.offset((fname_len - 1) as isize) == b'x' as c_char
            && *curbuf_fname.offset((fname_len - 4) as isize) == b'.' as c_char
            && nvim_mb_strnicmp(curbuf_fname.offset((fname_len - 3) as isize), help_lang, 2) == 0
        {
            nvim_findtags_set_help_pri(st, 0);
            return true;
        }
    }

    // Calculate help priority based on 'helplang' option
    nvim_findtags_set_help_pri(st, 1);
    let p_hlg = nvim_get_p_hlg();
    let mut s = p_hlg.cast_mut();
    let mut pri = 1;
    let mut found = false;

    while !s.is_null() && *s != NUL_BYTE as c_char {
        if nvim_mb_strnicmp(s, help_lang, 2) == 0 {
            found = true;
            break;
        }
        pri += 1;
        s = nvim_vim_strchr(s, b',' as c_int);
        if s.is_null() {
            break;
        }
        // skip past the comma
        s = s.offset(1);
    }

    if !found {
        // Language not in 'helplang': use last, prefer English
        pri += 1;
        // Check if help_lang is NOT "en"
        if nvim_mb_strnicmp(help_lang, c"en".as_ptr(), 2) != 0 {
            pri += 1;
        }
    }

    nvim_findtags_set_help_pri(st, pri);
    true
}

/// Main loop reading and parsing tags from a file.
///
/// Calls Phase 4-5 functions for each line:
/// get_next_line, string_convert, start_state_handler, parse_line,
/// match_tag, add_match.
///
/// The C `goto line_read_in` is replaced with a `get_searchpat` flag check.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T`
/// - `margs` must be a valid pointer to a `findtags_match_args_T`
/// - `buf_ffname` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_get_all_tags(
    st: FindTagsStateHandle,
    margs: FindTagsMatchArgsHandle,
    buf_ffname: *mut c_char,
) {
    if st.is_null() || margs.is_null() {
        return;
    }

    // tagptrs_T is allocated on stack, zeroed
    let tagp_size = 256usize; // generous allocation for tagptrs_T
    let tagp: TagPtrsHandle = xmalloc(tagp_size);
    std::ptr::write_bytes(tagp.cast::<u8>(), 0, tagp_size);

    // tagsearch_info_T on stack, zeroed
    let mut sinfo = TagSearchInfo::default();

    let mut hash: usize = 0;

    let flags = nvim_findtags_get_flags(st);

    // Cast margs to typed pointer for Phase 5 functions
    let margs_typed: *mut FindTagsMatchArgs = margs.cast();

    loop {
        // Check for CTRL-C typed
        let state = nvim_findtags_get_state(st);
        if state == TS_BINARY || state == TS_SKIP_BACK {
            nvim_line_breakcheck();
        } else {
            nvim_fast_breakcheck();
        }

        if (flags & find_tags_flags::TAG_INS_COMP) != 0 {
            nvim_ins_compl_check_keys(30, false);
        }

        if nvim_get_got_int() != 0 || rs_ins_compl_interrupted() != 0 {
            nvim_findtags_set_stop_searching(st, true);
            break;
        }

        // When mincount is TAG_MANY, stop when enough matches found
        let mincount = nvim_findtags_get_mincount(st);
        let match_count = nvim_findtags_get_match_count(st);
        if mincount == TAG_MANY && match_count >= TAG_MANY {
            nvim_findtags_set_stop_searching(st, true);
            break;
        }

        // Handle goto line_read_in pattern
        let get_searchpat = nvim_findtags_get_searchpat(st);
        if !get_searchpat {
            // Normal path: read next line
            let retval = rs_findtags_get_next_line(st, &raw mut sinfo) as c_int;

            if retval == TAGS_READ_IGNORE {
                continue;
            }
            if retval == TAGS_READ_EOF {
                break;
            }
        }

        // line_read_in: (reached via get_searchpat or after successful read)

        if nvim_findtags_get_vimconv_type(st) != CONV_NONE {
            rs_findtags_string_convert(st);
        }

        // When still at the start of the file, check for Emacs tags file
        // format, and for "not sorted" flag.
        let state = nvim_findtags_get_state(st);
        if state == TS_START {
            let sortic_ptr = &raw mut (*margs_typed).sortic;
            if !rs_findtags_start_state_handler(st, sortic_ptr, &raw mut sinfo) {
                continue;
            }
        }

        // Check if line is too long (needs lbuf grow)
        if nvim_findtags_grow_lbuf(st, (&raw mut sinfo).cast()) {
            continue;
        }

        // Parse the line and check for match
        let retval = rs_findtags_parse_line(st, tagp, margs_typed, &raw mut sinfo) as c_int;

        if retval == TAG_MATCH_NEXT {
            continue;
        }
        if retval == TAG_MATCH_STOP {
            break;
        }
        if retval == TAG_MATCH_FAIL {
            let tag_fname = nvim_findtags_get_tag_fname(st);
            nvim_semsg_e431(tag_fname);
            let offset = nvim_findtags_ftell(st);
            nvim_semsg_before_byte(offset);
            nvim_findtags_set_stop_searching(st, true);
            xfree(tagp);
            return;
        }

        // If a match is found, add it to ht_match[] and ga_match[].
        if rs_findtags_match_tag(st, tagp, margs_typed) {
            rs_findtags_add_match(st, tagp, margs_typed, buf_ffname, &raw mut hash);
        }
    }

    xfree(tagp);
}

/// Per-file search orchestration.
///
/// Sets up state, opens file, calls rs_findtags_get_all_tags, cleans up.
///
/// # Safety
///
/// - `st` must be a valid pointer to a `findtags_state_T`
/// - `buf_ffname` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_findtags_in_file(
    st: FindTagsStateHandle,
    _flags: c_int,
    buf_ffname: *mut c_char,
) {
    if st.is_null() {
        return;
    }

    // Initialize per-file state
    nvim_findtags_set_vimconv_none(st);
    nvim_findtags_set_sorted(st, 0); // NUL
    nvim_findtags_set_fp_null(st);

    // Initialize match args on stack
    let mut margs = FindTagsMatchArgs::default();
    rs_findtags_matchargs_init((&raw mut margs).cast(), nvim_findtags_get_flags(st));

    // For help files, initialize language/priority
    if nvim_get_curbuf_b_help() != 0 && !rs_findtags_in_help_init(st) {
        return;
    }

    // Open the tag file
    if !nvim_findtags_fopen(st) {
        return;
    }

    // Verbose message
    if nvim_get_p_verbose() >= 5 {
        let tag_fname = nvim_findtags_get_tag_fname(st);
        nvim_verbose_searching_tags(tag_fname);
    }

    nvim_findtags_set_did_open(st);
    nvim_findtags_set_state_start(st);

    // Read and parse the lines in the file one by one
    rs_findtags_get_all_tags(st, (&raw mut margs).cast(), buf_ffname);

    // Cleanup
    nvim_findtags_fclose(st);

    if nvim_findtags_get_vimconv_type(st) != CONV_NONE {
        nvim_findtags_convert_cleanup(st);
    }

    if margs.sort_error {
        let tag_fname = nvim_findtags_get_tag_fname(st);
        nvim_semsg_e432(tag_fname);
    }

    // Stop searching if sufficient tags have been found.
    let match_count = nvim_findtags_get_match_count(st);
    let mincount = nvim_findtags_get_mincount(st);
    if match_count >= mincount {
        nvim_findtags_set_stop_searching(st, true);
    }
}

// =============================================================================
// Phase 2 tag.c migration: rs_find_tags
// =============================================================================

extern "C" {
    /// Heap-allocate a zero-initialized findtags_state_T (caller must call rs_findtags_state_init).
    fn nvim_findtags_state_xcalloc() -> FindTagsStateHandle;
    /// Free the findtags_state_T struct itself (after rs_findtags_state_free).
    fn nvim_findtags_state_delete(st: FindTagsStateHandle);
    /// Get the mutable tag_fname buffer pointer from the state.
    fn nvim_findtags_get_tag_fname_buf(st: FindTagsStateHandle) -> *mut c_char;
}

/// Top-level tag search orchestrator: Rust implementation of find_tags().
///
/// # Safety
///
/// - `pat` must be a valid, mutable C string
/// - `num_matches` and `matchesp` must be valid output pointers
/// - `buf_ffname` may be null
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_find_tags(
    pat: *mut c_char,
    num_matches: *mut c_int,
    matchesp: *mut *mut *mut c_char,
    flags: c_int,
    mincount: c_int,
    buf_ffname: *mut c_char,
) -> c_int {
    let save_p_ic = nvim_get_p_ic();

    // Adjust p_ic per tagcase option
    let effective_tc = if nvim_get_curbuf_tc_flags() != 0 {
        nvim_get_curbuf_tc_flags()
    } else {
        nvim_get_tc_flags()
    };
    match effective_tc {
        x if x == K_OPT_TC_FLAG_FOLLOWIC => {}
        x if x == K_OPT_TC_FLAG_IGNORE => nvim_set_p_ic(1),
        x if x == K_OPT_TC_FLAG_MATCH => nvim_set_p_ic(0),
        x if x == K_OPT_TC_FLAG_FOLLOWSCS => {
            nvim_set_p_ic(c_int::from(nvim_ignorecase(pat)));
        }
        x if x == K_OPT_TC_FLAG_SMART => {
            nvim_set_p_ic(c_int::from(nvim_ignorecase_opt(pat, true, true)));
        }
        _ => {}
    }

    let help_save = nvim_get_curbuf_b_help();

    // Heap-allocate (xcalloc) and initialize the findtags state
    let st = nvim_findtags_state_xcalloc();
    if st.is_null() {
        nvim_set_p_ic(save_p_ic);
        *num_matches = 0;
        return FAIL;
    }
    rs_findtags_state_init(st, pat, flags, mincount);

    let mut saved_pat: *mut c_char = std::ptr::null_mut();
    #[allow(unused_assignments)]
    let mut retval: c_int = FAIL;

    // If help_only, temporarily set b_help
    if nvim_findtags_get_help_only(st) {
        nvim_set_curbuf_b_help(1);
    }

    // Help language suffix extraction: "@ab" at end of pattern
    if nvim_get_curbuf_b_help() != 0 {
        let pat_len = nvim_findtags_get_orgpat_len(st);
        if pat_len > 3 {
            let p = pat.offset((pat_len - 3) as isize);
            let at_char = *p as u8;
            let c1 = *p.offset(1) as u8;
            let c2 = *p.offset(2) as u8;
            if at_char == b'@' && c1.is_ascii_alphabetic() && c2.is_ascii_alphabetic() {
                saved_pat = nvim_xstrnsave(pat, (pat_len - 3) as usize);
                nvim_findtags_set_help_lang_find(st, p.offset(1));
                nvim_findtags_set_orgpat_pat(st, saved_pat);
                nvim_findtags_set_orgpat_len(st, pat_len - 3);
            }
        }
    }

    // Adjust for 'taglength' option
    {
        let tl = nvim_get_p_tl();
        let pat_len = nvim_findtags_get_orgpat_len(st);
        if tl != 0 && (pat_len as i64) > tl {
            nvim_findtags_set_orgpat_len(st, tl as c_int);
        }
    }

    // Prepare patterns (regexp compilation), suppressing errors
    let save_emsg_off = nvim_get_emsg_off();
    nvim_set_emsg_off(1);
    let has_re = (flags & find_tags_flags::TAG_REGEXP) != 0;
    nvim_findtags_prepare_pats(st, has_re);
    nvim_set_emsg_off(save_emsg_off);

    if has_re && !nvim_findtags_has_regprog(st) {
        retval = FAIL;
    } else {
        retval = rs_findtags_apply_tfu(st, pat, buf_ffname);
        if retval == NOTDONE {
            retval = FAIL;

            // Check .txt extension for help priority
            let cur_fname = nvim_get_curbuf_b_fname();
            if (flags & find_tags_flags::TAG_KEEP_LANG) != 0
                && nvim_findtags_get_help_lang_find(st).is_null()
                && !cur_fname.is_null()
            {
                let fname_len = strlen(cur_fname);
                if fname_len > 4 {
                    let ext = cur_fname.add(fname_len - 4);
                    if rs_tag_strnicmp(ext, c".txt".as_ptr(), 4) == 0 {
                        nvim_findtags_set_is_txt(st, true);
                    }
                }
            }

            // Two-round search loop
            let p_ic = nvim_get_p_ic();
            let noic = (flags & find_tags_flags::TAG_NOIC) != 0;
            // MAXCOL = 0x7FFF_FFFF, TAG_MANY = 300
            let findall = mincount == 0x7FFF_FFFF || mincount == 300;

            let headlen = nvim_findtags_get_orgpat_headlen(st);
            let p_tbs = nvim_get_p_tbs();
            let initial_rm_ic = (p_ic != 0 || !noic) && (findall || headlen == 0 || !p_tbs);
            nvim_findtags_set_orgpat_rm_ic(st, initial_rm_ic);

            // Stack-allocate TagFileIterator (mirrors tagname_T)
            let mut tn = crate::files::TagFileIterator::default();
            let tn_ptr: *mut crate::files::TagFileIterator = &raw mut tn;

            'outer: for round in 1..=2i32 {
                let cur_headlen = nvim_findtags_get_orgpat_headlen(st);
                let cur_p_tbs = nvim_get_p_tbs();
                let linear = cur_headlen == 0 || !cur_p_tbs || round == 2;
                nvim_findtags_set_linear(st, linear);

                let mut first_file = 1i32;
                loop {
                    let tag_fname_buf = nvim_findtags_get_tag_fname_buf(st);
                    if crate::files::rs_get_tagfname(tn_ptr, first_file, tag_fname_buf) != OK {
                        break;
                    }
                    first_file = 0;
                    rs_findtags_in_file(st, flags, buf_ffname);
                    if nvim_findtags_get_stop_searching(st) {
                        retval = OK;
                        break;
                    }
                }

                crate::rs_tagname_free(tn_ptr.cast::<c_void>());

                let stop = nvim_findtags_get_stop_searching(st)
                    || nvim_findtags_get_linear(st)
                    || (nvim_get_p_ic() == 0 && noic)
                    || nvim_findtags_get_orgpat_rm_ic(st);
                if stop {
                    break 'outer;
                }
                // Second round: ignore case
                nvim_findtags_set_orgpat_rm_ic(st, true);
            }

            if !nvim_findtags_get_stop_searching(st) {
                let verbose = (flags & find_tags_flags::TAG_VERBOSE) != 0;
                if !nvim_findtags_get_did_open(st) && verbose {
                    nvim_emsg_e433();
                }
                retval = OK;
            }
        }
    }

    // Free inner resources (tag_fname, lbuf, orgpat)
    rs_findtags_state_free(st);

    // If FAIL, discard matches
    if retval == FAIL {
        nvim_findtags_set_match_count(st, 0);
    }
    *num_matches = rs_findtags_copy_matches(st, matchesp);

    nvim_set_curbuf_b_help(help_save);
    xfree(saved_pat.cast::<c_void>());
    nvim_set_p_ic(save_p_ic);

    // Free the heap-allocated struct
    nvim_findtags_state_delete(st);

    retval
}

// =============================================================================
// Phase 1 tag.c migration: rs_tag_call_tagfunc
// =============================================================================

extern "C" {
    // Phase 2: fine-grained accessors replacing nvim_tag_curbuf_tfu_is_ready
    fn nvim_tag_curbuf_b_p_tfu_is_empty() -> bool;
    fn nvim_tag_curbuf_tfu_cb_is_none() -> bool;
    // Accessor: get g_tag_at_cursor flag
    fn nvim_tag_get_g_tag_at_cursor() -> bool;
    // Phase 2: tagstack accessors replacing nvim_tag_get_curwin_tagstack_user_data
    fn nvim_tag_get_curwin() -> *mut c_void;
    fn nvim_win_get_tagstacklen(wp: *const c_void) -> c_int;
    fn nvim_win_get_tagstackidx(wp: *const c_void) -> c_int;
    fn nvim_win_get_tagstack_entry(wp: *const c_void, idx: c_int) -> *const c_void;
    fn nvim_taggy_get_user_data(tg: *const c_void) -> *const c_char;
    // Accessor: allocate a VAR_FIXED-locked dict
    fn nvim_tag_dict_alloc_lock_fixed() -> *mut c_void;
    // Accessor: increment/decrement dict refcount
    fn nvim_tag_dict_refcount_inc(dict: *mut c_void);
    fn nvim_tag_dict_refcount_dec(dict: *mut c_void);
    // Accessor: invoke the curbuf tagfunc callback
    fn nvim_tag_do_callback_call_tfu(
        pat: *const c_char,
        flag_str: *const c_char,
        dict: *mut c_void,
        rettv_storage: *mut c_void,
    ) -> c_int;
    // Accessor: save/restore cursor with check
    fn nvim_tag_save_cursor(pos_storage: *mut c_void);
    fn nvim_tag_restore_cursor_check(pos_storage: *mut c_void);
    // Accessor: rettv type inspection
    fn nvim_tag_rettv_is_null_special(rettv_storage: *const c_void) -> bool;
    fn nvim_tag_rettv_get_list(rettv_storage: *const c_void) -> *mut c_void;
    // Accessor: size of pos_T for stack allocation
    fn nvim_tag_pos_size() -> usize;
    // Accessor: add string to dict
    fn nvim_tag_tv_dict_add_str(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *mut c_char,
    ) -> c_int;
    // Accessor: emit error message for invalid tagfunc return
    fn nvim_tag_emsg_invalid_tagfunc();
    // tv_clear for rettv storage
    fn nvim_tag_tv_clear_rettv(rettv_storage: *mut c_void);
}

/// Implement the tagfunc callback invocation in Rust.
///
/// Calls the VimL `tagfunc` callback with pattern, flags, and buf_ffname.
/// Validates the return value.
///
/// Returns:
///   0 (OK) with *out_list set to the returned list
///   1 (FAIL) if callback call failed
///   2 (NOTDONE) if result was v:null
///   3 if result was not a list (emsg already shown)
///   4 if curbuf tfu is empty or callback is none
///
/// # Safety
/// - `pat` must be a valid C string
/// - `buf_ffname` may be null
/// - `out_list` must be a valid pointer to store the result list
/// - `rettv_storage` must point to at least `sizeof(typval_T)` bytes of zeroed storage
///
/// # Panics
/// Panics if `pos_T` is larger than 16 bytes (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_tag_call_tagfunc(
    pat: *const c_char,
    flags: c_int,
    buf_ffname: *const c_char,
    out_list: *mut *mut c_void,
    rettv_storage: *mut c_void,
) -> c_int {
    // Phase 2: inline nvim_tag_curbuf_tfu_is_ready check with fine-grained accessors
    if nvim_tag_curbuf_b_p_tfu_is_empty() || nvim_tag_curbuf_tfu_cb_is_none() {
        return 4;
    }

    // Build the flag string: "c" for at-cursor, "i" for insert-completion, "r" for regexp
    let at_cursor = nvim_tag_get_g_tag_at_cursor();
    let ins_comp = (flags & find_tags_flags::TAG_INS_COMP) != 0;
    let regexp = (flags & find_tags_flags::TAG_REGEXP) != 0;

    // Build flag string into a 4-byte buffer (max "cir\0")
    let mut flag_buf = [0u8; 4];
    let mut fpos = 0usize;
    if at_cursor {
        flag_buf[fpos] = b'c';
        fpos += 1;
    }
    if ins_comp {
        flag_buf[fpos] = b'i';
        fpos += 1;
    }
    if regexp {
        flag_buf[fpos] = b'r';
        fpos += 1;
    }
    let _ = fpos; // null terminator already in place (initialized to 0)
    let flag_str = flag_buf.as_ptr().cast::<c_char>();

    // Allocate the info dict with VAR_FIXED lock
    let d = nvim_tag_dict_alloc_lock_fixed();

    // Phase 2: inline nvim_tag_get_curwin_tagstack_user_data logic using fine-grained accessors
    let curwin = nvim_tag_get_curwin();
    let tagstacklen = nvim_win_get_tagstacklen(curwin);
    let user_data: *const c_char = if tagstacklen > 0 {
        let tagstackidx = nvim_win_get_tagstackidx(curwin);
        let idx = if tagstackidx == tagstacklen {
            tagstackidx - 1
        } else {
            tagstackidx
        };
        let tg = nvim_win_get_tagstack_entry(curwin, idx);
        nvim_taggy_get_user_data(tg)
    } else {
        std::ptr::null()
    };
    if !user_data.is_null() {
        // key = "user_data", key_len = 9
        nvim_tag_tv_dict_add_str(d, c"user_data".as_ptr(), 9, user_data.cast_mut());
    }

    // Add buf_ffname if provided
    if !buf_ffname.is_null() {
        nvim_tag_tv_dict_add_str(d, c"buf_ffname".as_ptr(), 10, buf_ffname.cast_mut());
    }

    // Increment dict refcount to keep it alive during the call
    nvim_tag_dict_refcount_inc(d);

    // Save cursor position (pos_T on stack via byte array)
    let pos_sz = nvim_tag_pos_size();
    assert!(pos_sz <= 16, "pos_T too large");
    let mut pos_buf = [0u8; 16];
    nvim_tag_save_cursor(pos_buf.as_mut_ptr().cast::<c_void>());

    // Invoke the callback
    let result = nvim_tag_do_callback_call_tfu(pat, flag_str, d, rettv_storage);

    // Restore cursor and check
    nvim_tag_restore_cursor_check(pos_buf.as_mut_ptr().cast::<c_void>());

    // Decrement dict refcount
    nvim_tag_dict_refcount_dec(d);

    // FAIL (0) from C callback_call means failure
    if result == 0 {
        return 1;
    }

    // Check rettv result type
    if nvim_tag_rettv_is_null_special(rettv_storage) {
        nvim_tag_tv_clear_rettv(rettv_storage);
        return 2;
    }

    let list = nvim_tag_rettv_get_list(rettv_storage);
    if list.is_null() {
        nvim_tag_tv_clear_rettv(rettv_storage);
        nvim_tag_emsg_invalid_tagfunc();
        return 3;
    }

    *out_list = list;
    0
}

// =============================================================================
// Phase 9: find_tagfunc_tags — invoke user tagfunc and build match strings
// =============================================================================

extern "C" {
    fn nvim_tag_rettv_size() -> usize;
    fn nvim_tag_tv_list_first(list: *const c_void) -> *mut c_void;
    fn nvim_tag_tv_list_item_next(list: *const c_void, li: *const c_void) -> *mut c_void;
    fn nvim_tag_listitem_is_dict(li: *const c_void) -> bool;
    fn nvim_tag_listitem_get_dict(li: *const c_void) -> *mut c_void;
    // Dict iteration API (Phase 2)
    fn nvim_tag_dict_iter_start(dict: *const c_void) -> *mut c_void;
    fn nvim_tag_dict_iter_next(dict: *const c_void, cur: *const c_void) -> *mut c_void;
    fn nvim_tag_dict_iter_key(iter: *const c_void) -> *const c_char;
    fn nvim_tag_dict_iter_value_is_string(iter: *const c_void) -> bool;
    fn nvim_tag_dict_iter_value_string(iter: *const c_void) -> *const c_char;
    fn nvim_tag_emsg_invalid_tagfunc_return();
    fn nvim_tag_ga_grow_append(ga: *mut c_void, mfp: *mut c_char);
    fn xstrdup(s: *const c_char) -> *mut c_char;
}

/// Check if a key (C string) matches one of the 4 standard tag fields.
unsafe fn is_standard_field(key: *const c_char) -> bool {
    use std::ffi::CStr;
    let k = CStr::from_ptr(key).to_bytes();
    k == b"name" || k == b"filename" || k == b"cmd" || k == b"kind"
}

/// Rust port of nvim_tag_dict_compute_match_len:
/// Compute total byte length needed for building the tagfunc match string.
unsafe fn dict_compute_match_len(dict: *const c_void) -> usize {
    let mut len = 2usize; // base overhead
    let mut iter = nvim_tag_dict_iter_start(dict);
    while !iter.is_null() {
        if nvim_tag_dict_iter_value_is_string(iter) {
            let val = nvim_tag_dict_iter_value_string(iter);
            if !val.is_null() {
                len += std::ffi::CStr::from_ptr(val).to_bytes().len() + 1; // "\tVALUE"
                let key = nvim_tag_dict_iter_key(iter);
                if !is_standard_field(key) {
                    len += std::ffi::CStr::from_ptr(key).to_bytes().len() + 1; // "KEY:"
                }
            }
        }
        iter = nvim_tag_dict_iter_next(dict, iter);
    }
    len
}

/// Rust port of nvim_tag_dict_get_tag_fields:
/// Fill in name/filename/cmd/kind pointers and has_extra flag from a dict.
#[allow(clippy::similar_names)]
unsafe fn dict_get_tag_fields(
    dict: *const c_void,
    res_name: *mut *const c_char,
    res_fname: *mut *const c_char,
    res_cmd: *mut *const c_char,
    res_kind: *mut *const c_char,
    has_extra: *mut bool,
) {
    *res_name = std::ptr::null();
    *res_fname = std::ptr::null();
    *res_cmd = std::ptr::null();
    *res_kind = std::ptr::null();
    *has_extra = false;

    let mut iter = nvim_tag_dict_iter_start(dict);
    while !iter.is_null() {
        if nvim_tag_dict_iter_value_is_string(iter) {
            let key = nvim_tag_dict_iter_key(iter);
            let val = nvim_tag_dict_iter_value_string(iter);
            let k = std::ffi::CStr::from_ptr(key).to_bytes();
            if k == b"name" {
                *res_name = val;
            } else if k == b"filename" {
                *res_fname = val;
            } else if k == b"cmd" {
                *res_cmd = val;
            } else if k == b"kind" {
                *res_kind = val;
                *has_extra = true;
            } else {
                *has_extra = true;
            }
        }
        iter = nvim_tag_dict_iter_next(dict, iter);
    }
}

/// Rust port of nvim_tag_dict_write_extra_fields:
/// Write non-standard key:value pairs as tab-separated entries.
/// Returns the number of bytes written.
unsafe fn dict_write_extra_fields(dict: *const c_void, mut p: *mut c_char) -> usize {
    let start = p;
    let mut iter = nvim_tag_dict_iter_start(dict);
    while !iter.is_null() {
        if nvim_tag_dict_iter_value_is_string(iter) {
            let key = nvim_tag_dict_iter_key(iter);
            let val = nvim_tag_dict_iter_value_string(iter);
            if !val.is_null() && !is_standard_field(key) {
                let key_bytes = std::ffi::CStr::from_ptr(key).to_bytes();
                let val_bytes = std::ffi::CStr::from_ptr(val).to_bytes();
                *p = b'\t' as c_char;
                p = p.add(1);
                std::ptr::copy_nonoverlapping(
                    key_bytes.as_ptr().cast::<c_char>(),
                    p,
                    key_bytes.len(),
                );
                p = p.add(key_bytes.len());
                *p = b':' as c_char;
                p = p.add(1);
                std::ptr::copy_nonoverlapping(
                    val_bytes.as_ptr().cast::<c_char>(),
                    p,
                    val_bytes.len(),
                );
                p = p.add(val_bytes.len());
            }
        }
        iter = nvim_tag_dict_iter_next(dict, iter);
    }
    p.offset_from(start) as usize
}

/// Invoke the user-defined tagfunc to get tag matches.
///
/// This replaces `find_tagfunc_tags()` in C. It calls the tagfunc callback
/// via a C accessor, then iterates the returned list of dicts in Rust to
/// build the encoded match strings.
///
/// # Panics
/// Panics if `sizeof(typval_T)` exceeds 64 bytes (should never happen).
///
/// # Safety
/// - `pat` must be a valid C string
/// - `ga` must be a valid garray_T pointer (from findtags_state_T.ga_match)
/// - `match_count` must be a valid pointer
/// - `buf_ffname` may be null
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_find_tagfunc_tags(
    pat: *mut c_char,
    ga: *mut c_void,
    match_count: *mut c_int,
    flags: c_int,
    buf_ffname: *mut c_char,
) -> c_int {
    // Allocate rettv storage on the stack (use max reasonable size)
    // We use a byte array large enough for typval_T
    let rettv_size = nvim_tag_rettv_size();
    // typval_T should be <= 24 bytes; allocate 64 for safety
    assert!(rettv_size <= 64);
    let mut rettv_buf = [0u8; 64];
    let rettv_ptr = rettv_buf.as_mut_ptr().cast::<c_void>();

    let mut list: *mut c_void = std::ptr::null_mut();

    let call_result = rs_tag_call_tagfunc(pat, flags, buf_ffname, &raw mut list, rettv_ptr);

    match call_result {
        0 => {} // OK, list is valid
        2 => return NOTDONE,
        // 1=callback failed, 3=invalid return, 4=no tagfunc, other=unexpected
        _ => return FAIL,
    }

    let name_only = (flags & TAG_NAMES) != 0;
    let mut ntags: c_int = 0;
    let mut result = FAIL;

    let mut li = nvim_tag_tv_list_first(list);
    while !li.is_null() {
        if !nvim_tag_listitem_is_dict(li) {
            nvim_tag_emsg_invalid_tagfunc_return();
            break;
        }

        let dict = nvim_tag_listitem_get_dict(li);
        if dict.is_null() {
            nvim_tag_emsg_invalid_tagfunc_return();
            break;
        }

        // Extract the standard fields
        let mut tag_name: *const c_char = std::ptr::null();
        let mut tag_file: *const c_char = std::ptr::null();
        let mut tag_cmd: *const c_char = std::ptr::null();
        let mut tag_kind: *const c_char = std::ptr::null();
        let mut has_extra = false;

        dict_get_tag_fields(
            dict,
            &raw mut tag_name,
            &raw mut tag_file,
            &raw mut tag_cmd,
            &raw mut tag_kind,
            &raw mut has_extra,
        );

        if tag_name.is_null() || tag_file.is_null() || tag_cmd.is_null() {
            nvim_tag_emsg_invalid_tagfunc_return();
            break;
        }

        if name_only {
            // Just return the tag name
            let mfp = xstrdup(tag_name);
            nvim_tag_ga_grow_append(ga, mfp);
        } else {
            // Compute total length needed
            let mut len = dict_compute_match_len(dict);
            if has_extra {
                len += 2; // for ;\"
            }

            let mfp = xmalloc(len + 2).cast::<c_char>();
            let mut p = mfp;

            // mtt byte + TAG_SEP
            *p = (MT_GL_OTH + 1) as c_char;
            p = p.add(1);
            *p = TAG_SEP as c_char;
            p = p.add(1);

            // name
            let name_len = strlen(tag_name);
            std::ptr::copy_nonoverlapping(tag_name.cast::<u8>(), p.cast::<u8>(), name_len);
            p = p.add(name_len);

            // TAB + filename
            *p = TAB as c_char;
            p = p.add(1);
            let file_len = strlen(tag_file);
            std::ptr::copy_nonoverlapping(tag_file.cast::<u8>(), p.cast::<u8>(), file_len);
            p = p.add(file_len);

            // TAB + cmd
            *p = TAB as c_char;
            p = p.add(1);
            let cmd_len = strlen(tag_cmd);
            std::ptr::copy_nonoverlapping(tag_cmd.cast::<u8>(), p.cast::<u8>(), cmd_len);
            p = p.add(cmd_len);

            if has_extra {
                // ;\"
                *p = b';' as c_char;
                p = p.add(1);
                *p = b'"' as c_char;
                p = p.add(1);

                // kind field first
                if !tag_kind.is_null() {
                    *p = TAB as c_char;
                    p = p.add(1);
                    let kind_len = strlen(tag_kind);
                    std::ptr::copy_nonoverlapping(tag_kind.cast::<u8>(), p.cast::<u8>(), kind_len);
                    p = p.add(kind_len);
                }

                // Extra fields (KEY:VALUE)
                let extra_written = dict_write_extra_fields(dict, p);
                p = p.add(extra_written);
            }

            // Null-terminate
            *p = 0;

            nvim_tag_ga_grow_append(ga, mfp);
        }

        ntags += 1;
        result = OK;

        li = nvim_tag_tv_list_item_next(list, li);
    }

    nvim_tag_tv_clear_rettv(rettv_ptr);

    *match_count = ntags;
    result
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
