//! Search pattern handling for command-line mode
//!
//! This module provides utilities for handling search patterns in command-line
//! mode, including incremental search (incsearch), pattern parsing, and
//! search direction detection.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Search Direction
// =============================================================================

/// Direction for search operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchDirection {
    /// Search forward (/)
    #[default]
    Forward,
    /// Search backward (?)
    Backward,
}

impl SearchDirection {
    /// Get the character representing this direction.
    #[must_use]
    pub const fn to_char(self) -> u8 {
        match self {
            Self::Forward => b'/',
            Self::Backward => b'?',
        }
    }

    /// Parse from a character.
    #[must_use]
    pub const fn from_char(c: u8) -> Option<Self> {
        match c {
            b'/' => Some(Self::Forward),
            b'?' => Some(Self::Backward),
            _ => None,
        }
    }

    /// Check if a firstc character represents a search prompt.
    #[must_use]
    pub const fn is_search_firstc(firstc: u8) -> bool {
        firstc == b'/' || firstc == b'?'
    }

    /// Get the opposite direction.
    #[must_use]
    pub const fn reverse(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

// =============================================================================
// Search Delimiter
// =============================================================================

/// Parse the search delimiter from a pattern.
///
/// The delimiter is the first character of the pattern (/ or ?),
/// or the first character after a command prefix.
#[must_use]
pub fn parse_search_delimiter(pattern: &[u8]) -> Option<u8> {
    if pattern.is_empty() {
        return None;
    }

    let first = pattern[0];
    if first == b'/' || first == b'?' {
        return Some(first);
    }

    None
}

/// Find the end of a search pattern.
///
/// Returns the index after the closing delimiter, or pattern length if not found.
#[must_use]
pub fn find_pattern_end(pattern: &[u8], delimiter: u8) -> usize {
    if pattern.is_empty() {
        return 0;
    }

    // Skip the opening delimiter
    let start = usize::from(pattern[0] == delimiter);

    let mut i = start;
    while i < pattern.len() {
        if pattern[i] == delimiter && (i == 0 || pattern[i - 1] != b'\\') {
            return i + 1;
        }
        i += 1;
    }

    pattern.len()
}

/// Extract the search pattern from a command line.
///
/// Returns the pattern without delimiters.
#[must_use]
pub fn extract_pattern(cmdline: &[u8], delimiter: u8) -> Option<&[u8]> {
    if cmdline.is_empty() {
        return None;
    }

    // Skip opening delimiter
    let start = usize::from(cmdline[0] == delimiter);

    // Find closing delimiter
    let mut end = start;
    while end < cmdline.len() {
        if cmdline[end] == delimiter && (end == 0 || cmdline[end - 1] != b'\\') {
            break;
        }
        end += 1;
    }

    if start < end {
        Some(&cmdline[start..end])
    } else {
        None
    }
}

// =============================================================================
// Incsearch State
// =============================================================================

/// State for incremental search.
#[derive(Debug, Clone, Copy, Default)]
pub struct IncsearchState {
    /// Whether incremental search is currently active.
    pub active: bool,
    /// Whether incsearch highlighting has been done.
    pub did_incsearch: bool,
    /// Whether incsearch is postponed (e.g., pattern too short).
    pub postponed: bool,
    /// The search direction.
    pub direction: SearchDirection,
    /// Search count (for n pattern).
    pub count: i32,
}

impl IncsearchState {
    /// Create a new incsearch state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active: false,
            did_incsearch: false,
            postponed: false,
            direction: SearchDirection::Forward,
            count: 0,
        }
    }

    /// Initialize for a new search.
    pub fn init(&mut self, firstc: u8) {
        self.active = SearchDirection::is_search_firstc(firstc);
        self.did_incsearch = false;
        self.postponed = false;
        self.direction = SearchDirection::from_char(firstc).unwrap_or_default();
        self.count = 0;
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Mark incsearch as done.
    pub fn mark_done(&mut self) {
        self.did_incsearch = true;
    }

    /// Mark incsearch as postponed.
    pub fn postpone(&mut self) {
        self.postponed = true;
    }
}

// =============================================================================
// Pattern Validation
// =============================================================================

/// Check if a pattern is valid for searching.
///
/// Empty patterns are invalid unless the previous search pattern should be used.
#[must_use]
pub const fn is_valid_search_pattern(pattern: &[u8]) -> bool {
    !pattern.is_empty()
}

/// Check if a pattern is a magic pattern (starts with \v or \m).
#[must_use]
pub fn is_magic_pattern(pattern: &[u8]) -> Option<bool> {
    if pattern.len() < 2 || pattern[0] != b'\\' {
        return None;
    }

    match pattern[1] {
        b'v' | b'm' => Some(true),  // very magic / magic
        b'M' | b'V' => Some(false), // nomagic / very nomagic
        _ => None,
    }
}

/// Check if a pattern contains only basic characters (no regex).
#[must_use]
pub fn is_literal_pattern(pattern: &[u8]) -> bool {
    for &c in pattern {
        match c {
            // Regex special characters
            b'.' | b'*' | b'+' | b'?' | b'[' | b']' | b'^' | b'$' | b'\\' | b'|' => {
                return false;
            }
            _ => {}
        }
    }
    true
}

// =============================================================================
// Search Command Parsing
// =============================================================================

/// Types of search-related commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCommandType {
    /// Simple search (/, ?)
    Search,
    /// Global command (:g/pattern/)
    Global,
    /// Substitute command (:s/pattern/replace/)
    Substitute,
    /// Vimgrep command (:vimgrep /pattern/)
    Vimgrep,
}

/// Parse the offset from after a search pattern.
///
/// Offsets are like `/pattern/+2` or `/pattern/e`.
#[must_use]
pub fn parse_search_offset(cmdline: &[u8], pattern_end: usize) -> Option<&[u8]> {
    if pattern_end >= cmdline.len() {
        return None;
    }

    Some(&cmdline[pattern_end..])
}

/// Check if a search command uses word boundary markers.
///
/// Patterns like \<word\> use word boundaries.
#[must_use]
pub fn has_word_boundary(pattern: &[u8]) -> bool {
    if pattern.len() < 2 {
        return false;
    }

    let mut i = 0;
    while i < pattern.len() - 1 {
        if pattern[i] == b'\\' && (pattern[i + 1] == b'<' || pattern[i + 1] == b'>') {
            return true;
        }
        i += 1;
    }

    false
}

// =============================================================================
// Incremental Search Highlighting
// =============================================================================

/// Result of incremental search highlighting decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncsearchHighlight {
    /// Perform highlighting.
    Highlight,
    /// Skip highlighting (pattern too short or invalid).
    Skip,
    /// Postpone highlighting.
    Postpone,
}

/// Check if incsearch highlighting should be done.
///
/// Returns the highlighting decision based on pattern and settings.
#[must_use]
pub fn should_do_incsearch_highlighting(
    pattern: &[u8],
    incsearch_enabled: bool,
    min_pattern_len: usize,
) -> IncsearchHighlight {
    if !incsearch_enabled {
        return IncsearchHighlight::Skip;
    }

    // Check for valid pattern first (empty pattern is invalid, not postponed)
    if !is_valid_search_pattern(pattern) {
        return IncsearchHighlight::Skip;
    }

    // Postpone if pattern is too short (but not empty)
    if pattern.len() < min_pattern_len {
        return IncsearchHighlight::Postpone;
    }

    IncsearchHighlight::Highlight
}

// =============================================================================
// FFI-compatible Types
// =============================================================================

/// Position in file or buffer (matches C pos_T).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    /// Line number
    pub lnum: i32,
    /// Column number
    pub col: c_int,
    /// Column add (for virtual columns)
    pub coladd: c_int,
}

impl PosT {
    /// Create a new position.
    #[must_use]
    pub const fn new(lnum: i32, col: c_int, coladd: c_int) -> Self {
        Self { lnum, col, coladd }
    }

    /// Clear the position (set to zero).
    pub fn clear(&mut self) {
        self.lnum = 0;
        self.col = 0;
        self.coladd = 0;
    }
}

/// View state for a window (matches C viewstate_T).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewStateT {
    pub vs_curswant: c_int,
    pub vs_leftcol: c_int,
    pub vs_skipcol: c_int,
    pub vs_topline: i32,
    pub vs_topfill: c_int,
    pub vs_botline: i32,
    pub vs_empty_rows: c_int,
}

/// Magic override state (matches C optmagic_T).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptMagicT {
    #[default]
    NotSet = 0,
    On = 1,
    Off = 2,
}

/// FFI-compatible incsearch state (matches C incsearch_state_T).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IncsearchStateT {
    /// Where 'incsearch' starts searching
    pub search_start: PosT,
    /// Saved cursor position
    pub save_cursor: PosT,
    /// Window where this state is valid
    pub winid: c_int,
    /// Initial view state
    pub init_viewstate: ViewStateT,
    /// Old view state (for restore)
    pub old_viewstate: ViewStateT,
    /// Match start position
    pub match_start: PosT,
    /// Match end position
    pub match_end: PosT,
    /// Whether incsearch highlighting has been done
    pub did_incsearch: bool,
    /// Whether incsearch is postponed
    pub incsearch_postponed: bool,
    /// Saved magic_overruled value
    pub magic_overruled_save: c_int,
}

// =============================================================================
// FFI Functions
// =============================================================================

// External C functions for accessing global state
extern "C" {
    fn nvim_get_curwin_handle() -> c_int;
    fn nvim_get_curwin_cursor_pos(pos: *mut PosT);
    fn nvim_set_curwin_cursor_pos(pos: *const PosT);
    fn nvim_get_magic_overruled() -> c_int;
    fn nvim_save_viewstate(vs: *mut ViewStateT);
    fn nvim_restore_viewstate(vs: *const ViewStateT);
    fn nvim_option_set_magic_overruled(value: c_int);

    // Incsearch highlighting C dependencies
    fn nvim_get_p_is() -> c_int;
    fn nvim_cmd_silent() -> c_int;
    fn nvim_char_avail() -> c_int;
    fn nvim_set_highlight_match(value: c_int);
    fn nvim_set_search_first_line(value: i32);
    fn nvim_set_search_last_line(value: i32);
    fn nvim_validate_cursor();
    fn nvim_status_redraw_all();
    fn nvim_redraw_all_later(upd_type: c_int);
    fn nvim_update_screen();
    fn nvim_setpcmark();
    fn nvim_equalpos(pos1: *const PosT, pos2: *const PosT) -> c_int;
}

// Update type constants (from nvim/types_defs.h)
const UPD_SOME_VALID: c_int = 10;

// MAXLNUM constant
const MAXLNUM: i32 = 0x7fff_ffff;

/// Initialize incsearch state (FFI).
///
/// This implements the init_incsearch_state function from C.
///
/// # Safety
///
/// `state` must be a valid pointer to an IncsearchStateT struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_init_incsearch_state(state: *mut IncsearchStateT) {
    if state.is_null() {
        return;
    }

    let s = &mut *state;

    // Get current window handle
    s.winid = nvim_get_curwin_handle();

    // Get current cursor position
    nvim_get_curwin_cursor_pos(std::ptr::addr_of_mut!(s.match_start));

    // Reset flags
    s.did_incsearch = false;
    s.incsearch_postponed = false;

    // Save magic_overruled
    s.magic_overruled_save = nvim_get_magic_overruled();

    // Clear match_end
    s.match_end.clear();

    // Copy cursor to save_cursor and search_start
    s.save_cursor = s.match_start;
    s.search_start = s.match_start;

    // Save view state
    nvim_save_viewstate(std::ptr::addr_of_mut!(s.init_viewstate));
    nvim_save_viewstate(std::ptr::addr_of_mut!(s.old_viewstate));
}

/// Finish incsearch highlighting (FFI).
///
/// Cleans up after incremental search, restoring cursor position and view state.
///
/// # Safety
///
/// `state` must be a valid pointer to an IncsearchStateT struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_finish_incsearch_highlighting(
    gotesc: c_int,
    state: *mut IncsearchStateT,
    call_update_screen: c_int,
) {
    if state.is_null() {
        return;
    }

    let s = &mut *state;

    if !s.did_incsearch {
        return;
    }

    s.did_incsearch = false;

    if gotesc != 0 {
        // Restore cursor to saved position on escape
        nvim_set_curwin_cursor_pos(std::ptr::addr_of!(s.save_cursor));
    } else {
        // Check if we need to set the mark
        if nvim_equalpos(
            std::ptr::addr_of!(s.save_cursor),
            std::ptr::addr_of!(s.search_start),
        ) == 0
        {
            // Put the '" mark at the original position
            nvim_set_curwin_cursor_pos(std::ptr::addr_of!(s.save_cursor));
            nvim_setpcmark();
        }
        nvim_set_curwin_cursor_pos(std::ptr::addr_of!(s.search_start));
    }

    // Restore view state
    nvim_restore_viewstate(std::ptr::addr_of!(s.old_viewstate));

    // Turn off highlight match
    nvim_set_highlight_match(0);

    // Reset search line range to default
    nvim_set_search_first_line(0);
    nvim_set_search_last_line(MAXLNUM);

    // Restore magic_overruled
    nvim_option_set_magic_overruled(s.magic_overruled_save);

    // Validation and redraw
    nvim_validate_cursor();
    nvim_status_redraw_all();
    nvim_redraw_all_later(UPD_SOME_VALID);

    if call_update_screen != 0 {
        nvim_update_screen();
    }
}

/// Check if incsearch highlighting should be done (FFI).
///
/// This is a simplified check for whether 'incsearch' is enabled and
/// we're in a context where highlighting makes sense.
///
/// Returns 1 if highlighting should be done, 0 otherwise.
///
/// # Safety
///
/// This function calls C functions to check global state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_should_do_incsearch(firstc: c_int) -> c_int {
    // Check if 'incsearch' option is set and not in silent mode
    if nvim_get_p_is() == 0 || nvim_cmd_silent() != 0 {
        return 0;
    }

    // For search prompts, always do incsearch
    if firstc == b'/' as c_int || firstc == b'?' as c_int {
        return 1;
    }

    // For ex commands, need to parse the pattern (handled in C for now)
    if firstc == b':' as c_int {
        return 1; // Let caller handle pattern parsing
    }

    0
}

/// Check if input is available and incsearch should be postponed (FFI).
///
/// # Safety
///
/// This function calls C functions to check input availability.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_incsearch_should_postpone() -> c_int {
    nvim_char_avail()
}

/// Check if a firstc is a search prompt (FFI).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_search_firstc(firstc: c_int) -> c_int {
    if !(0..=255).contains(&firstc) {
        return 0;
    }
    c_int::from(SearchDirection::is_search_firstc(firstc as u8))
}

/// Get the search direction from firstc (FFI).
///
/// Returns 1 for forward, -1 for backward, 0 for not a search.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_search_direction(firstc: c_int) -> c_int {
    if !(0..=255).contains(&firstc) {
        return 0;
    }

    match SearchDirection::from_char(firstc as u8) {
        Some(SearchDirection::Forward) => 1,
        Some(SearchDirection::Backward) => -1,
        None => 0,
    }
}

/// Parse search delimiter from pattern (FFI).
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_parse_search_delimiter(pattern: *const c_char, len: usize) -> c_int {
    if pattern.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    parse_search_delimiter(bytes).map_or(0, c_int::from)
}

/// Find end of search pattern (FFI).
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_find_pattern_end(
    pattern: *const c_char,
    len: usize,
    delimiter: c_int,
) -> c_int {
    if pattern.is_null() || len == 0 || !(0..=255).contains(&delimiter) {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    find_pattern_end(bytes, delimiter as u8) as c_int
}

/// Check if pattern is literal (no regex) (FFI).
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_literal_pattern(pattern: *const c_char, len: usize) -> c_int {
    if pattern.is_null() || len == 0 {
        return 1; // Empty is literal
    }

    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    c_int::from(is_literal_pattern(bytes))
}

/// Check if pattern has word boundary markers (FFI).
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_has_word_boundary(pattern: *const c_char, len: usize) -> c_int {
    if pattern.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    c_int::from(has_word_boundary(bytes))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_direction() {
        assert_eq!(SearchDirection::Forward.to_char(), b'/');
        assert_eq!(SearchDirection::Backward.to_char(), b'?');

        assert_eq!(
            SearchDirection::from_char(b'/'),
            Some(SearchDirection::Forward)
        );
        assert_eq!(
            SearchDirection::from_char(b'?'),
            Some(SearchDirection::Backward)
        );
        assert_eq!(SearchDirection::from_char(b':'), None);

        assert!(SearchDirection::is_search_firstc(b'/'));
        assert!(SearchDirection::is_search_firstc(b'?'));
        assert!(!SearchDirection::is_search_firstc(b':'));

        assert_eq!(
            SearchDirection::Forward.reverse(),
            SearchDirection::Backward
        );
        assert_eq!(
            SearchDirection::Backward.reverse(),
            SearchDirection::Forward
        );
    }

    #[test]
    fn test_parse_search_delimiter() {
        assert_eq!(parse_search_delimiter(b"/pattern"), Some(b'/'));
        assert_eq!(parse_search_delimiter(b"?pattern"), Some(b'?'));
        assert_eq!(parse_search_delimiter(b"pattern"), None);
        assert_eq!(parse_search_delimiter(b""), None);
    }

    #[test]
    fn test_find_pattern_end() {
        assert_eq!(find_pattern_end(b"/foo/bar", b'/'), 5);
        assert_eq!(find_pattern_end(b"/foo", b'/'), 4);
        assert_eq!(find_pattern_end(b"/foo\\/bar/x", b'/'), 10);
        assert_eq!(find_pattern_end(b"", b'/'), 0);
    }

    #[test]
    fn test_extract_pattern() {
        assert_eq!(extract_pattern(b"/foo/bar", b'/'), Some(b"foo".as_slice()));
        assert_eq!(extract_pattern(b"/foo", b'/'), Some(b"foo".as_slice()));
        assert_eq!(extract_pattern(b"//", b'/'), None);
        assert_eq!(extract_pattern(b"", b'/'), None);
    }

    #[test]
    fn test_incsearch_state() {
        let mut state = IncsearchState::new();
        assert!(!state.active);
        assert!(!state.did_incsearch);

        state.init(b'/');
        assert!(state.active);
        assert_eq!(state.direction, SearchDirection::Forward);

        state.init(b'?');
        assert!(state.active);
        assert_eq!(state.direction, SearchDirection::Backward);

        state.init(b':');
        assert!(!state.active);
    }

    #[test]
    fn test_is_valid_search_pattern() {
        assert!(is_valid_search_pattern(b"foo"));
        assert!(!is_valid_search_pattern(b""));
    }

    #[test]
    fn test_is_magic_pattern() {
        assert_eq!(is_magic_pattern(b"\\vfoo"), Some(true));
        assert_eq!(is_magic_pattern(b"\\mfoo"), Some(true));
        assert_eq!(is_magic_pattern(b"\\Mfoo"), Some(false));
        assert_eq!(is_magic_pattern(b"\\Vfoo"), Some(false));
        assert_eq!(is_magic_pattern(b"foo"), None);
        assert_eq!(is_magic_pattern(b"\\x"), None);
    }

    #[test]
    fn test_is_literal_pattern() {
        assert!(is_literal_pattern(b"foo"));
        assert!(is_literal_pattern(b"foo bar"));
        assert!(!is_literal_pattern(b"foo.*"));
        assert!(!is_literal_pattern(b"^foo"));
        assert!(!is_literal_pattern(b"foo$"));
        assert!(!is_literal_pattern(b"[abc]"));
    }

    #[test]
    fn test_has_word_boundary() {
        assert!(has_word_boundary(b"\\<word\\>"));
        assert!(has_word_boundary(b"\\<word"));
        assert!(has_word_boundary(b"word\\>"));
        assert!(!has_word_boundary(b"word"));
        assert!(!has_word_boundary(b"<word>"));
    }

    #[test]
    fn test_should_do_incsearch_highlighting() {
        assert_eq!(
            should_do_incsearch_highlighting(b"foo", true, 1),
            IncsearchHighlight::Highlight
        );
        assert_eq!(
            should_do_incsearch_highlighting(b"", true, 1),
            IncsearchHighlight::Skip
        );
        assert_eq!(
            should_do_incsearch_highlighting(b"f", true, 2),
            IncsearchHighlight::Postpone
        );
        assert_eq!(
            should_do_incsearch_highlighting(b"foo", false, 1),
            IncsearchHighlight::Skip
        );
    }
}
