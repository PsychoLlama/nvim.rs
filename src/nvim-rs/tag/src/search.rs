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

use std::ffi::{c_char, c_int};

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
// Tests
// =============================================================================

#[cfg(test)]
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
}
