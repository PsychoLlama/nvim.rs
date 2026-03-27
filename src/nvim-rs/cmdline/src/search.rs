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

use std::ffi::{c_char, c_int, c_void};

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

extern "C" {
    static mut got_int: bool;
    static mut magic_overruled: c_int;
}

// skip_regexp_ex from C regexp module
extern "C" {
    #[link_name = "skip_regexp_ex"]
    fn c_skip_regexp_ex(
        startp: *mut c_char,
        dirc: c_int,
        magic: c_int,
        newp: *mut *mut c_char,
        dropped: *mut c_int,
        magic_val: *mut c_int,
    ) -> *mut c_char;
    fn rs_magic_isset() -> c_int;
}

// Magic constants from regexp_defs.h
const MAGIC_ON: c_int = 3;
const MAGIC_ALL: c_int = 4;

// External C functions for accessing global state
extern "C" {
    fn nvim_get_curwin_handle() -> c_int;
    fn nvim_get_curwin_cursor_pos(pos: *mut PosT);
    fn nvim_set_curwin_cursor_pos(pos: *const PosT);
    fn nvim_save_viewstate(vs: *mut ViewStateT);
    fn nvim_restore_viewstate(vs: *const ViewStateT);

    // parse_pattern_and_range: still in C, called from do_incsearch_highlighting
    fn parse_pattern_and_range(
        incsearch_start: *mut PosT,
        search_delim: *mut c_int,
        skiplen: *mut c_int,
        patlen: *mut c_int,
    ) -> bool;

    // Accessors for ccline fields
    fn nvim_get_ccline_cmdlen() -> c_int;

    // emsg_off accessors
    fn nvim_inc_emsg_off();
    fn nvim_dec_emsg_off();

    // Incsearch highlighting C dependencies
    fn nvim_get_p_is() -> c_int;
    static mut cmd_silent: bool;
    fn nvim_char_avail() -> c_int;
    fn nvim_set_highlight_match(value: c_int);
    fn nvim_set_search_first_line(value: i32);
    fn nvim_set_search_last_line(value: i32);
    fn nvim_validate_cursor();
    fn nvim_status_redraw_all();
    fn nvim_redraw_all_later(upd_type: c_int);
    fn nvim_update_screen();
    fn setpcmark();
    fn nvim_equalpos(pos1: *const PosT, pos2: *const PosT) -> c_int;

    // For may_add_char_to_search (Rust exports from search crate)
    fn save_last_search_pattern();
    fn restore_last_search_pattern();
    fn nvim_gchar_cursor() -> c_int;
    fn nvim_get_p_ic() -> c_int;
    fn nvim_get_p_scs() -> c_int;
    fn pat_has_uppercase(pat: *mut c_char) -> bool;
    fn mb_tolower(c: c_int) -> c_int;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn nvim_stuffcharReadbuff(c: c_int);
    fn utf_char2len(c: c_int) -> c_int;
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_get_cursor_pos_ptr() -> *mut c_char;
    fn nvim_get_curwin_cursor_col() -> c_int;
    fn nvim_set_curwin_cursor_col(col: c_int);
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
}

// Update type constants (from nvim/drawscreen.h)
const UPD_SOME_VALID: c_int = 35;

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
    s.magic_overruled_save = magic_overruled;

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
            setpcmark();
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
    magic_overruled = s.magic_overruled_save;

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
    if nvim_get_p_is() == 0 || cmd_silent {
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

/// Direct C replacement for cmdline_parse_search_delim().
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[must_use]
#[export_name = "cmdline_parse_search_delim"]
pub unsafe extern "C" fn cmdline_parse_search_delim_rs(
    pattern: *const c_char,
    len: usize,
) -> c_int {
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

/// Direct C replacement for cmdline_is_literal_pattern().
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[must_use]
#[export_name = "cmdline_is_literal_pattern"]
pub unsafe extern "C" fn cmdline_is_literal_pattern_rs(pattern: *const c_char, len: usize) -> bool {
    if pattern.is_null() || len == 0 {
        return true; // Empty is literal
    }
    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    is_literal_pattern(bytes)
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

/// Direct C replacement for cmdline_has_word_boundary().
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[must_use]
#[export_name = "cmdline_has_word_boundary"]
pub unsafe extern "C" fn cmdline_has_word_boundary_rs(pattern: *const c_char, len: usize) -> bool {
    if pattern.is_null() || len == 0 {
        return false;
    }
    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    has_word_boundary(bytes)
}

// =============================================================================
// Empty Pattern Detection
// =============================================================================

/// Check if a pattern is empty given a magic value.
///
/// Removes trailing `\v`, `\m`, `\M`, `\V`, `\c`, `\C`, `\Z` modifiers and
/// checks if the remaining pattern is empty or ends with `\|` (magic) or `|`
/// (very-magic).
///
/// This is the pure-Rust version of `empty_pattern_magic()` from ex_getln.c.
#[must_use]
pub fn empty_pattern_magic(p: &[u8], magic_val: c_int) -> bool {
    let mut len = p.len();

    // Remove trailing \v and similar modifiers
    while len >= 2 && p[len - 2] == b'\\' {
        let c = p[len - 1];
        if matches!(c, b'm' | b'M' | b'v' | b'V' | b'c' | b'C' | b'Z') {
            len -= 2;
        } else {
            break;
        }
    }

    // Pattern is empty, or ends with \| (magic on) or | (very magic)
    len == 0
        || (len > 1
            && p[len - 1] == b'|'
            && ((p[len - 2] == b'\\' && magic_val == MAGIC_ON)
                || (p[len - 2] != b'\\' && magic_val == MAGIC_ALL)))
}

/// Guess that the pattern matches everything.
///
/// Calls `skip_regexp_ex` to advance past the pattern and determine the
/// effective magic mode, then delegates to `empty_pattern_magic`.
///
/// This is the pure-Rust version of `empty_pattern()` from ex_getln.c.
///
/// # Safety
///
/// `p` must be a valid pointer to a NUL-terminated string with at least `len`
/// bytes before the NUL. `delim` must be a valid delimiter character.
pub unsafe fn empty_pattern(p: *mut c_char, len: usize, delim: c_int) -> bool {
    let mut magic_val: c_int = MAGIC_ON;

    if len == 0 {
        return true;
    }

    c_skip_regexp_ex(
        p,
        delim,
        rs_magic_isset(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &raw mut magic_val,
    );

    let bytes = std::slice::from_raw_parts(p.cast::<u8>(), len);
    empty_pattern_magic(bytes, magic_val)
}

/// FFI export: check if pattern is empty given magic value.
///
/// # Safety
///
/// `p` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_empty_pattern_magic(
    p: *const c_char,
    len: usize,
    magic_val: c_int,
) -> c_int {
    if p.is_null() {
        return 1;
    }
    let bytes = std::slice::from_raw_parts(p.cast::<u8>(), len);
    c_int::from(empty_pattern_magic(bytes, magic_val))
}

/// FFI export: guess that the pattern matches everything.
///
/// # Safety
///
/// `p` must be a valid mutable pointer to a NUL-terminated string with at
/// least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_empty_pattern(p: *mut c_char, len: usize, delim: c_int) -> c_int {
    if p.is_null() {
        return 1;
    }
    c_int::from(empty_pattern(p, len, delim))
}

// =============================================================================
// may_add_char_to_search: CTRL-L adds char from match to search pattern
// =============================================================================

// NUL constant
const NUL: c_int = 0;

/// Add character from match under cursor to the search pattern (CTRL-L).
///
/// Direct replacement for C `may_add_char_to_search()`.
/// Returns OK (1) when command_line_not_changed should be called, FAIL (0) otherwise.
///
/// # Safety
///
/// `c` and `is_state` must be valid non-null pointers.
#[export_name = "may_add_char_to_search"]
pub unsafe extern "C" fn may_add_char_to_search_rs(
    firstc: c_int,
    c: *mut c_int,
    is_state: *mut IncsearchStateT,
) -> c_int {
    let mut skiplen: c_int = 0;
    let mut patlen: c_int = 0;
    let mut search_delim: c_int = 0;

    // NOTE: must call restore_last_search_pattern() before returning!
    save_last_search_pattern();

    // Add a character from under the cursor for 'incsearch'
    if !do_incsearch_highlighting_rs(
        firstc,
        &raw mut search_delim,
        is_state,
        &raw mut skiplen,
        &raw mut patlen,
    ) {
        restore_last_search_pattern();
        return 0; // FAIL
    }
    restore_last_search_pattern();

    let s = &mut *is_state;
    if s.did_incsearch {
        // Move cursor to match end
        nvim_set_curwin_cursor_pos(&raw const s.match_end);
        *c = nvim_gchar_cursor();
        if *c != NUL {
            // If 'ignorecase' and 'smartcase' are set and pattern has no uppercase,
            // convert to lowercase
            if nvim_get_p_ic() != 0 && nvim_get_p_scs() != 0 {
                let cmdbuff = nvim_get_ccline_cmdbuff();
                if !cmdbuff.is_null() {
                    let pat_ptr = cmdbuff.add(skiplen as usize);
                    if !pat_has_uppercase(pat_ptr) {
                        *c = mb_tolower(*c);
                    }
                }
            }

            // Put backslash before special characters
            let magic_chars: *const c_char = if rs_magic_isset() != 0 {
                c"\\~^$.*[".as_ptr()
            } else {
                c"\\^$".as_ptr()
            };
            if *c == search_delim || !vim_strchr(magic_chars, *c).is_null() {
                nvim_stuffcharReadbuff(*c);
                *c = b'\\' as c_int;
            }

            // Add any composing characters
            let cursor_ptr = nvim_get_cursor_pos_ptr();
            if !cursor_ptr.is_null() && utf_char2len(*c) != nvim_utfc_ptr2len(cursor_ptr) {
                let save_c = *c;
                loop {
                    let cursor_ptr2 = nvim_get_cursor_pos_ptr();
                    if cursor_ptr2.is_null() || utf_char2len(*c) == nvim_utfc_ptr2len(cursor_ptr2) {
                        break;
                    }
                    let new_col = nvim_get_curwin_cursor_col() + utf_char2len(*c);
                    nvim_set_curwin_cursor_col(new_col);
                    *c = nvim_gchar_cursor();
                    nvim_stuffcharReadbuff(*c);
                }
                *c = save_c;
            }
            return 0; // FAIL - char was added
        }
    }
    1 // OK
}

// =============================================================================
// do_incsearch_highlighting: check if incsearch highlighting should happen
// =============================================================================

/// Return true when 'incsearch' highlighting is to be done.
/// Sets search_first_line and search_last_line for the address range.
/// May change the last search pattern.
///
/// Direct replacement for C `do_incsearch_highlighting()`.
///
/// # Safety
///
/// `is_state` must be a valid pointer to an IncsearchStateT.
/// `search_delim`, `skiplen`, `patlen` must be valid non-null output pointers.
#[export_name = "do_incsearch_highlighting"]
pub unsafe extern "C" fn do_incsearch_highlighting_rs(
    firstc: c_int,
    search_delim: *mut c_int,
    is_state: *mut IncsearchStateT,
    skiplen: *mut c_int,
    patlen: *mut c_int,
) -> bool {
    *skiplen = 0;
    *patlen = nvim_get_ccline_cmdlen();

    // Check 'incsearch' and silent mode
    if nvim_get_p_is() == 0 || cmd_silent {
        return false;
    }

    // Default: search all lines
    nvim_set_search_first_line(0);
    nvim_set_search_last_line(MAXLNUM);

    // For / and ? searches, always highlight
    if firstc == b'/' as c_int || firstc == b'?' as c_int {
        *search_delim = firstc;
        return true;
    }

    // Only : (ex command) can have incsearch via parse_pattern_and_range
    if firstc != b':' as c_int {
        return false;
    }

    nvim_inc_emsg_off();
    let retval = parse_pattern_and_range(
        &raw mut (*is_state).search_start,
        search_delim,
        skiplen,
        patlen,
    );
    nvim_dec_emsg_off();

    retval
}

// =============================================================================
// may_do_incsearch_highlighting and may_do_command_line_next_incsearch
// =============================================================================

// Search flag constants (from search.h)
const SEARCH_OPT: c_int = 0x10;
const SEARCH_NOOF: c_int = 0x80;
const SEARCH_START: c_int = 0x100;
const SEARCH_KEEP: c_int = 0x400;
const SEARCH_PEEK: c_int = 0x800;
const SEARCH_COL: c_int = 0x1000;

// Direction constants (from vim_defs.h)
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

// RE_SEARCH (use normal search pattern)
const RE_SEARCH: c_int = 0;

// UPD_NOT_VALID (from drawscreen.h)
const UPD_NOT_VALID: c_int = 38;

/// Optional extra arguments for searchit() (matches C searchit_arg_T).
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct SearchitArgT {
    sa_stop_lnum: i32, // linenr_T
    sa_tm: *mut u64,   // proftime_T * (proftime_T = uint64_t)
    sa_timed_out: c_int,
    sa_wrapped: c_int,
}

unsafe extern "C" {
    fn ui_busy_start();
    fn ui_busy_stop();
    fn ui_flush();
    fn vpeekc() -> c_int;
    fn profile_setlimit(msec: i64) -> u64; // returns proftime_T
    fn set_no_hlsearch(flag: c_int);
    fn do_search(
        oap: *mut c_void,
        dirc: c_int,
        search_delim: c_int,
        pat: *mut c_char,
        patlen: usize,
        count: c_int,
        options: c_int,
        sia: *mut SearchitArgT,
    ) -> c_int;
    fn searchit(
        win: *mut c_void,
        buf: *mut c_void,
        pos: *mut PosT,
        end_pos: *mut PosT,
        dir: c_int,
        pat: *mut c_char,
        patlen: usize,
        count: c_int,
        options: c_int,
        pat_use: c_int,
        extra_arg: *mut SearchitArgT,
    ) -> c_int;
    fn nvim_get_search_first_line() -> i32;
    fn nvim_get_search_last_line() -> i32;
    fn nvim_get_p_ru() -> c_int;
    fn nvim_get_no_hlsearch() -> c_int;
    fn nvim_get_p_hls() -> c_int;
    fn nvim_curbuf_get_ml_line_count() -> c_int;
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_get_curbuf() -> *mut c_void;
    fn nvim_win_get_status_height(wp: *mut ()) -> c_int;
    fn nvim_win_set_redr_status(wp: *mut c_void, val: c_int);
    fn nvim_win_set_valid_cursor(wp: *mut c_void, lnum: i32, col: c_int, coladd: c_int);
    fn redraw_later(wp: *mut c_void, redraw_type: c_int);
    fn nvim_set_search_match(t: *mut PosT);
    fn rs_global_stl_height() -> c_int;
    fn msg_starthere();
    fn changed_cline_bef_curs(wp: *mut c_void);
    fn update_topline(wp: *mut c_void);
    fn validate_cursor(wp: *mut c_void);
    fn vim_beep(flag: c_int);
    fn decl(pos: *mut PosT);
    fn incl(pos: *mut PosT);
    fn last_search_pattern() -> *mut c_char;
    fn last_search_pattern_len() -> usize;
    #[link_name = "rs_save_viewstate_win"]
    fn save_viewstate_win_vs(wp: *mut c_void, vs: *mut ViewStateT);
    #[link_name = "rs_restore_viewstate_win"]
    fn restore_viewstate_win_vs(wp: *mut c_void, vs: *const ViewStateT);
}

// kOptBoFlagError for vim_beep (from option_defs.h)
const K_OPT_BO_FLAG_ERROR: c_int = 0x01;

// MAXCOL for column (from pos_defs.h)
const MAXCOL: c_int = 0x7FFF_FFFF;

// Return codes (from ex_getln.c)
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Comparison: is pos1 < pos2? (lt macro)
unsafe fn pos_lt(pos1: &PosT, pos2: &PosT) -> bool {
    pos1.lnum < pos2.lnum || (pos1.lnum == pos2.lnum && pos1.col < pos2.col)
}

/// May do incsearch highlighting.
/// Rust replacement for `may_do_incsearch_highlighting` in ex_getln.c.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to IncsearchStateT.
#[allow(clippy::too_many_lines)]
#[no_mangle]
pub unsafe extern "C" fn rs_may_do_incsearch_highlighting(
    firstc: c_int,
    count: c_int,
    s: *mut IncsearchStateT,
) {
    let mut skiplen: c_int = 0;
    let mut patlen: c_int = 0;
    let mut search_delim: c_int = 0;

    // Parsing range may already set the last search pattern.
    // NOTE: must call restore_last_search_pattern() before returning!
    save_last_search_pattern();

    if !do_incsearch_highlighting_rs(
        firstc,
        &raw mut search_delim,
        s,
        &raw mut skiplen,
        &raw mut patlen,
    ) {
        restore_last_search_pattern();
        rs_finish_incsearch_highlighting(0, s, 1);
        return;
    }

    // if there is a character waiting, search and redraw later
    if nvim_char_avail() != 0 {
        restore_last_search_pattern();
        (*s).incsearch_postponed = true;
        return;
    }
    (*s).incsearch_postponed = false;

    // Use previous pattern for ":s//".
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let next_char_idx = skiplen + patlen;
    let next_char = *cmdbuff.add(next_char_idx as usize);
    let use_last_pat =
        patlen == 0 && skiplen > 0 && *cmdbuff.add((skiplen - 1) as usize) == next_char;

    if patlen != 0 || use_last_pat {
        ui_busy_start();
        ui_flush();
    }

    let search_first_line = nvim_get_search_first_line();
    let curwin = nvim_get_curwin();

    if search_first_line == 0 {
        // start at the original cursor position
        nvim_set_curwin_cursor_pos(std::ptr::addr_of!((*s).search_start));
    } else if search_first_line > nvim_curbuf_get_ml_line_count() {
        // start after the last line
        let mut pos = PosT::default();
        nvim_get_curwin_cursor_pos(&raw mut pos);
        pos.lnum = nvim_curbuf_get_ml_line_count();
        pos.col = MAXCOL as c_int;
        nvim_set_curwin_cursor_pos(&raw const pos);
    } else {
        // start at the first line in the range
        let mut pos = PosT::default();
        nvim_get_curwin_cursor_pos(&raw mut pos);
        pos.lnum = search_first_line;
        pos.col = 0;
        nvim_set_curwin_cursor_pos(&raw const pos);
    }

    let mut found: c_int = 0;

    if patlen != 0 || use_last_pat {
        let mut search_flags = SEARCH_OPT | SEARCH_NOOF | SEARCH_PEEK;
        if nvim_get_p_hls() == 0 {
            search_flags |= SEARCH_KEEP;
        }
        if search_first_line != 0 {
            search_flags |= SEARCH_START;
        }
        let mut tm = profile_setlimit(500);
        let mut sia = SearchitArgT {
            sa_stop_lnum: 0,
            sa_tm: &raw mut tm,
            sa_timed_out: 0,
            sa_wrapped: 0,
        };
        *cmdbuff.add(next_char_idx as usize) = 0;
        nvim_inc_emsg_off();
        found = do_search(
            std::ptr::null_mut(),
            if firstc == b':' as c_int {
                b'/' as c_int
            } else {
                firstc
            },
            search_delim,
            cmdbuff.add(skiplen as usize),
            patlen as usize,
            count,
            search_flags,
            &raw mut sia,
        );
        nvim_dec_emsg_off();
        *cmdbuff.add(next_char_idx as usize) = next_char;

        let search_last_line = nvim_get_search_last_line();
        let mut cur_pos = PosT::default();
        nvim_get_curwin_cursor_pos(&raw mut cur_pos);
        if cur_pos.lnum < search_first_line || cur_pos.lnum > search_last_line {
            // match outside of address range
            found = 0;
            nvim_set_curwin_cursor_pos(std::ptr::addr_of!((*s).search_start));
        }

        // if interrupted while searching, behave like it failed
        if unsafe { got_int } {
            vpeekc(); // remove <C-C> from input stream
            unsafe {
                got_int = false;
            } // don't abandon the command line
            found = 0;
        } else if nvim_char_avail() != 0 {
            // cancelled searching because a char was typed
            (*s).incsearch_postponed = true;
        }
        ui_busy_stop();
    } else {
        set_no_hlsearch(1); // turn off previous highlight
        nvim_redraw_all_later(UPD_SOME_VALID);
    }

    nvim_set_highlight_match(i32::from(found != 0));

    // first restore the old curwin values, so the screen is
    // positioned in the same way as the actual search command
    restore_viewstate_win_vs(curwin, &raw const (*s).old_viewstate);
    changed_cline_bef_curs(curwin);
    update_topline(curwin);

    let mut end_pos = PosT::default();
    nvim_get_curwin_cursor_pos(&raw mut end_pos);
    if found != 0 {
        nvim_get_curwin_cursor_pos(&raw mut (*s).match_start);
        nvim_set_search_match(&raw mut (*s).match_start);
        // Actually: set_search_match moves cursor to end, then call validate_cursor
        // Re-fetch cursor as match_end
        validate_cursor(curwin);
        nvim_get_curwin_cursor_pos(&raw mut (*s).match_end);
        nvim_set_curwin_cursor_pos(&raw const end_pos);
        end_pos = (*s).match_end;
    }

    // Disable 'hlsearch' highlighting if the pattern matches everything.
    if !use_last_pat {
        let nc = *cmdbuff.add(next_char_idx as usize);
        *cmdbuff.add(next_char_idx as usize) = 0;
        if rs_empty_pattern(cmdbuff.add(skiplen as usize), patlen as usize, search_delim) != 0
            && nvim_get_no_hlsearch() == 0
        {
            nvim_redraw_all_later(UPD_SOME_VALID);
            set_no_hlsearch(1);
        }
        *cmdbuff.add(next_char_idx as usize) = nc;
    }

    validate_cursor(curwin);

    // May redraw the status line to show cursor position.
    if nvim_get_p_ru() != 0
        && (nvim_win_get_status_height(curwin.cast::<()>()) > 0 || rs_global_stl_height() > 0)
    {
        nvim_win_set_redr_status(curwin, 1);
    }

    redraw_later(curwin, UPD_SOME_VALID);
    nvim_update_screen();
    nvim_set_highlight_match(0);
    restore_last_search_pattern();

    // Leave cursor at end to make CTRL-R CTRL-W work. But not when beyond
    // end of pattern, e.g. for ":s/pat/".
    if *cmdbuff.add(next_char_idx as usize) != 0 {
        nvim_set_curwin_cursor_pos(std::ptr::addr_of!((*s).search_start));
    } else if found != 0 {
        nvim_set_curwin_cursor_pos(&raw const end_pos);
        nvim_win_set_valid_cursor(curwin, end_pos.lnum, end_pos.col, end_pos.coladd);
    }

    msg_starthere();
    crate::screen::rs_redrawcmdline();
    (*s).did_incsearch = true;
}

/// Handle CTRL-G/CTRL-T for next/prev incsearch match.
/// Rust replacement for `may_do_command_line_next_incsearch` in ex_getln.c.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to IncsearchStateT.
#[allow(clippy::too_many_lines)]
#[no_mangle]
pub unsafe extern "C" fn rs_may_do_command_line_next_incsearch(
    firstc: c_int,
    count: c_int,
    s: *mut IncsearchStateT,
    next_match: bool,
) -> c_int {
    let mut skiplen: c_int = 0;
    let mut patlen: c_int = 0;
    let mut search_delim: c_int = 0;

    // Parsing range may already set the last search pattern.
    // NOTE: must call restore_last_search_pattern() before returning!
    save_last_search_pattern();

    if !do_incsearch_highlighting_rs(
        firstc,
        &raw mut search_delim,
        s,
        &raw mut skiplen,
        &raw mut patlen,
    ) {
        restore_last_search_pattern();
        return OK;
    }

    let cmdbuff = nvim_get_ccline_cmdbuff();
    if patlen == 0 && *cmdbuff.add(skiplen as usize) == 0 {
        restore_last_search_pattern();
        return FAIL;
    }

    ui_busy_start();
    ui_flush();

    let mut t: PosT;
    let pat: *mut c_char;
    let mut search_flags: c_int = SEARCH_NOOF;

    if search_delim == *cmdbuff.add(skiplen as usize) as c_int {
        pat = last_search_pattern();
        if pat.is_null() {
            restore_last_search_pattern();
            return FAIL;
        }
        let _ = skiplen; // skiplen not needed when using last_search_pattern
        patlen = last_search_pattern_len() as c_int;
    } else {
        pat = cmdbuff.add(skiplen as usize);
    }

    let mut bslsh = false;
    // do not search for the search end delimiter, unless part of pattern
    if patlen > 2 && firstc == *pat.add((patlen - 1) as usize) as c_int {
        patlen -= 1;
        if *pat.add((patlen - 1) as usize) == b'\\' as c_char {
            *pat.add((patlen - 1) as usize) = firstc as c_char;
            bslsh = true;
        }
    }

    if next_match {
        t = (*s).match_end;
        if pos_lt(&(*s).match_start, &(*s).match_end) {
            // start searching at end of match, not beginning of next column
            decl(&raw mut t);
        }
        search_flags |= SEARCH_COL;
    } else {
        t = (*s).match_start;
    }
    if nvim_get_p_hls() == 0 {
        search_flags |= SEARCH_KEEP;
    }
    nvim_inc_emsg_off();
    let save_char = *pat.add(patlen as usize);
    *pat.add(patlen as usize) = 0;
    let curwin = nvim_get_curwin();
    let curbuf = nvim_get_curbuf();
    let found = searchit(
        curwin,
        curbuf,
        &raw mut t,
        std::ptr::null_mut(),
        if next_match { FORWARD } else { BACKWARD },
        pat,
        patlen as usize,
        count,
        search_flags,
        RE_SEARCH,
        std::ptr::null_mut(),
    );
    nvim_dec_emsg_off();
    *pat.add(patlen as usize) = save_char;
    if bslsh {
        *pat.add((patlen - 1) as usize) = b'\\' as c_char;
    }
    ui_busy_stop();
    if found != 0 {
        (*s).search_start = (*s).match_start;
        (*s).match_end = t;
        (*s).match_start = t;
        if !next_match && firstc != b'?' as c_int {
            // move just before current match
            (*s).search_start = t;
            decl(&raw mut (*s).search_start);
        } else if next_match && firstc == b'?' as c_int {
            // move just after current match
            (*s).search_start = t;
            incl(&raw mut (*s).search_start);
        }
        if pos_lt(&t, &(*s).search_start) && next_match {
            // wrap around
            (*s).search_start = t;
            if firstc == b'?' as c_int {
                incl(&raw mut (*s).search_start);
            } else {
                decl(&raw mut (*s).search_start);
            }
        }

        nvim_set_search_match(&raw mut (*s).match_end);
        nvim_set_curwin_cursor_pos(std::ptr::addr_of!((*s).match_start));
        changed_cline_bef_curs(curwin);
        update_topline(curwin);
        validate_cursor(curwin);
        nvim_set_highlight_match(1);
        save_viewstate_win_vs(curwin, &raw mut (*s).old_viewstate);
        redraw_later(curwin, UPD_NOT_VALID);
        nvim_update_screen();
        nvim_set_highlight_match(0);
        crate::screen::rs_redrawcmdline();
        nvim_get_curwin_cursor_pos(&raw mut (*s).match_end);
        // Actually: curwin->w_cursor = s->match_end
        nvim_set_curwin_cursor_pos(std::ptr::addr_of!((*s).match_end));
    } else {
        vim_beep(K_OPT_BO_FLAG_ERROR);
    }
    restore_last_search_pattern();
    FAIL
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
