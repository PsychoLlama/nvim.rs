//! High-level regex match helpers.
//!
//! This module provides utilities for:
//! - Pattern matching with different configurations
//! - Match result extraction and conversion
//! - Case-sensitivity handling
//! - Pattern flags management

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Regex flag: Include newline in [^...].
pub const RE_MAGIC: c_int = 1;

/// Regex flag: 'ignorecase' option (case-insensitive).
pub const RE_IGNORECASE: c_int = 2;

/// Regex flag: 'smartcase' option.
pub const RE_SMARTCASE: c_int = 4;

/// Regex flag: Only match start of line.
pub const RE_BOL: c_int = 8;

/// Regex flag: Don't change anything.
pub const RE_NO_CHANGE: c_int = 16;

/// Regex flag: Newlines are special.
pub const RE_NEWLINE: c_int = 32;

/// Regex flag: Use extended syntax.
pub const RE_EXTENDED: c_int = 64;

/// Regex flag: First character is already backslash-escaped.
pub const RE_EXTENDED_START: c_int = 128;

/// Regex engine: auto-select.
pub const AUTOMATIC_ENGINE: c_int = 0;

/// Regex engine: backtracking (BT).
pub const BACKTRACKING_ENGINE: c_int = 1;

/// Regex engine: NFA.
pub const NFA_ENGINE: c_int = 2;

/// Maximum subexpressions.
pub const NSUBEXP: usize = 10;

// =============================================================================
// Match Configuration
// =============================================================================

/// Configuration for regex matching.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MatchConfig {
    /// Use case-insensitive matching.
    pub ignore_case: bool,
    /// Consider 'smartcase' option.
    pub smart_case: bool,
    /// Include newline in match.
    pub newline: bool,
    /// Start column for matching.
    pub start_col: c_int,
    /// Maximum column to search (0 = no limit).
    pub max_col: c_int,
    /// Timeout in milliseconds (0 = no timeout).
    pub timeout_ms: i64,
}

impl MatchConfig {
    /// Create a new match configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ignore_case: false,
            smart_case: false,
            newline: false,
            start_col: 0,
            max_col: 0,
            timeout_ms: 0,
        }
    }

    /// Set case-insensitive matching.
    #[must_use]
    pub const fn with_ignore_case(mut self, value: bool) -> Self {
        self.ignore_case = value;
        self
    }

    /// Set smartcase matching.
    #[must_use]
    pub const fn with_smart_case(mut self, value: bool) -> Self {
        self.smart_case = value;
        self
    }

    /// Set newline matching.
    #[must_use]
    pub const fn with_newline(mut self, value: bool) -> Self {
        self.newline = value;
        self
    }

    /// Set start column.
    #[must_use]
    pub const fn with_start_col(mut self, col: c_int) -> Self {
        self.start_col = col;
        self
    }

    /// Convert to regex flags.
    #[must_use]
    pub const fn to_flags(self) -> c_int {
        let mut flags = 0;
        if self.ignore_case {
            flags |= RE_IGNORECASE;
        }
        if self.smart_case {
            flags |= RE_SMARTCASE;
        }
        if self.newline {
            flags |= RE_NEWLINE;
        }
        flags
    }
}

// =============================================================================
// Match Position
// =============================================================================

/// A position in a match (line number and column).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MatchPos {
    /// Line number (1-based, 0 = invalid).
    pub lnum: i32,
    /// Column offset (0-based).
    pub col: c_int,
}

impl MatchPos {
    /// Create a new position.
    #[must_use]
    pub const fn new(lnum: i32, col: c_int) -> Self {
        Self { lnum, col }
    }

    /// Check if this position is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.lnum > 0
    }

    /// Create an invalid position.
    #[must_use]
    pub const fn invalid() -> Self {
        Self { lnum: 0, col: 0 }
    }
}

// =============================================================================
// Match Range
// =============================================================================

/// A range within a match (start and end positions).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MatchRange {
    /// Start position.
    pub start: MatchPos,
    /// End position (exclusive).
    pub end: MatchPos,
}

impl MatchRange {
    /// Create a new range.
    #[must_use]
    pub const fn new(start: MatchPos, end: MatchPos) -> Self {
        Self { start, end }
    }

    /// Check if this range is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.start.is_valid() && self.end.is_valid()
    }

    /// Create an invalid range.
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            start: MatchPos::invalid(),
            end: MatchPos::invalid(),
        }
    }

    /// Get the length (for single-line matches).
    #[must_use]
    pub const fn len(&self) -> c_int {
        if self.start.lnum == self.end.lnum {
            self.end.col - self.start.col
        } else {
            -1 // Multi-line
        }
    }

    /// Check if range is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        !self.is_valid() || (self.start.lnum == self.end.lnum && self.start.col == self.end.col)
    }
}

// =============================================================================
// Match Result
// =============================================================================

/// Result of a regex match operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MatchResult {
    /// Whether the match succeeded.
    pub matched: bool,
    /// The overall match range.
    pub range: MatchRange,
    /// Submatch start positions.
    pub sub_start: [MatchPos; NSUBEXP],
    /// Submatch end positions.
    pub sub_end: [MatchPos; NSUBEXP],
}

impl Default for MatchResult {
    fn default() -> Self {
        Self {
            matched: false,
            range: MatchRange::invalid(),
            sub_start: [MatchPos::invalid(); NSUBEXP],
            sub_end: [MatchPos::invalid(); NSUBEXP],
        }
    }
}

impl MatchResult {
    /// Create a new unmatched result.
    #[must_use]
    pub fn no_match() -> Self {
        Self::default()
    }

    /// Create a new matched result with the given range.
    #[must_use]
    pub fn matched_at(range: MatchRange) -> Self {
        let mut sub_start = [MatchPos::invalid(); NSUBEXP];
        let mut sub_end = [MatchPos::invalid(); NSUBEXP];
        sub_start[0] = range.start;
        sub_end[0] = range.end;
        Self {
            matched: true,
            range,
            sub_start,
            sub_end,
        }
    }

    /// Set a submatch position.
    pub fn set_submatch(&mut self, idx: usize, start: MatchPos, end: MatchPos) {
        if idx < NSUBEXP {
            self.sub_start[idx] = start;
            self.sub_end[idx] = end;
        }
    }

    /// Get a submatch range.
    #[must_use]
    pub fn get_submatch(&self, idx: usize) -> Option<MatchRange> {
        if idx < NSUBEXP && self.sub_start[idx].is_valid() {
            Some(MatchRange::new(self.sub_start[idx], self.sub_end[idx]))
        } else {
            None
        }
    }

    /// Count the number of valid submatches.
    #[must_use]
    pub fn submatch_count(&self) -> usize {
        self.sub_start.iter().filter(|p| p.is_valid()).count()
    }
}

// =============================================================================
// Pattern Flags Helpers
// =============================================================================

/// Parse magic mode from pattern prefix.
///
/// Returns (magic_mode, offset) where magic_mode is:
/// - 0: no change
/// - 1: nomagic (\M)
/// - 2: magic (\m)
/// - 3: very nomagic (\V)
/// - 4: very magic (\v)
#[must_use]
pub fn parse_magic_prefix(pattern: &[u8]) -> (c_int, usize) {
    if pattern.len() < 2 || pattern[0] != b'\\' {
        return (0, 0);
    }

    match pattern[1] {
        b'M' => (1, 2), // nomagic
        b'm' => (2, 2), // magic
        b'V' => (3, 2), // very nomagic
        b'v' => (4, 2), // very magic
        _ => (0, 0),
    }
}

/// Check if a character is special in the given magic mode.
#[must_use]
pub const fn is_special_char(c: u8, magic_mode: c_int) -> bool {
    match magic_mode {
        4 => {
            // Very magic: almost everything is special
            !c.is_ascii_alphanumeric() && c != b'_'
        }
        3 => {
            // Very nomagic: only backslash is special
            c == b'\\'
        }
        2 => {
            // Magic: standard magic characters
            matches!(c, b'^' | b'$' | b'.' | b'*' | b'~' | b'[' | b']' | b'\\')
        }
        1 => {
            // Nomagic: only ^, $, and \ are special
            matches!(c, b'^' | b'$' | b'\\')
        }
        _ => {
            // Default to magic
            matches!(c, b'^' | b'$' | b'.' | b'*' | b'~' | b'[' | b']' | b'\\')
        }
    }
}

/// Escape a pattern for use in very nomagic mode.
#[must_use]
pub fn escape_for_search(pattern: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(pattern.len() * 2);
    result.extend_from_slice(b"\\V");

    for &c in pattern {
        if c == b'\\' || c == b'/' {
            result.push(b'\\');
        }
        result.push(c);
    }

    result
}

/// Check if pattern contains uppercase characters (for smartcase).
#[must_use]
pub fn has_uppercase(pattern: &[u8]) -> bool {
    let mut i = 0;
    while i < pattern.len() {
        let c = pattern[i];
        if c == b'\\' && i + 1 < pattern.len() {
            // Skip escaped characters
            i += 2;
            continue;
        }
        if c.is_ascii_uppercase() {
            return true;
        }
        i += 1;
    }
    false
}

// =============================================================================
// Backslash Handling
// =============================================================================

/// Translate backslash escapes in pattern.
///
/// Returns the character value (possibly with magic offset).
#[must_use]
pub const fn backslash_trans(c: u8) -> c_int {
    match c {
        b'r' => 13, // CR
        b't' => 9,  // TAB
        b'e' => 27, // ESC
        b'b' => 8,  // BS
        b'n' => -1, // NL (special: matches end of line)
        _ => c as c_int,
    }
}

/// Check if character class is supported.
#[must_use]
pub const fn supports_char_class(c: u8) -> bool {
    matches!(
        c,
        b'i' | b'I'
            | b'k'
            | b'K'
            | b'f'
            | b'F'
            | b'p'
            | b'P'
            | b's'
            | b'S'
            | b'd'
            | b'D'
            | b'x'
            | b'X'
            | b'o'
            | b'O'
            | b'w'
            | b'W'
            | b'h'
            | b'H'
            | b'a'
            | b'A'
            | b'l'
            | b'L'
            | b'u'
            | b'U'
    )
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new match configuration.
#[no_mangle]
pub extern "C" fn rs_match_config_new() -> MatchConfig {
    MatchConfig::new()
}

/// Set ignore_case on match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_set_ignore_case(
    config: *mut MatchConfig,
    value: c_int,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    (*config).ignore_case = value != 0;
    1
}

/// Set smart_case on match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_set_smart_case(
    config: *mut MatchConfig,
    value: c_int,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    (*config).smart_case = value != 0;
    1
}

/// Convert match configuration to flags.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_to_flags(config: *const MatchConfig) -> c_int {
    if config.is_null() {
        return 0;
    }
    (*config).to_flags()
}

/// Create a new match position.
#[no_mangle]
pub extern "C" fn rs_match_pos_new(lnum: i32, col: c_int) -> MatchPos {
    MatchPos::new(lnum, col)
}

/// Check if match position is valid.
///
/// # Safety
/// `pos` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_pos_is_valid(pos: *const MatchPos) -> c_int {
    if pos.is_null() {
        return 0;
    }
    c_int::from((*pos).is_valid())
}

/// Create a new match range.
#[no_mangle]
pub extern "C" fn rs_match_range_new(start: MatchPos, end: MatchPos) -> MatchRange {
    MatchRange::new(start, end)
}

/// Check if match range is valid.
///
/// # Safety
/// `range` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_range_is_valid(range: *const MatchRange) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from((*range).is_valid())
}

/// Get match range length.
///
/// # Safety
/// `range` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_range_len(range: *const MatchRange) -> c_int {
    if range.is_null() {
        return -1;
    }
    (*range).len()
}

/// Create a new match result (no match).
#[no_mangle]
pub extern "C" fn rs_match_result_new() -> MatchResult {
    MatchResult::no_match()
}

/// Check if match result succeeded.
///
/// # Safety
/// `result` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_matched(result: *const MatchResult) -> c_int {
    if result.is_null() {
        return 0;
    }
    c_int::from((*result).matched)
}

/// Get submatch count from match result.
///
/// # Safety
/// `result` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_submatch_count(result: *const MatchResult) -> c_int {
    if result.is_null() {
        return 0;
    }
    (*result).submatch_count() as c_int
}

/// Get submatch start position.
///
/// # Safety
/// `result` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_get_start(
    result: *const MatchResult,
    idx: c_int,
) -> MatchPos {
    if result.is_null() || idx < 0 || idx as usize >= NSUBEXP {
        return MatchPos::invalid();
    }
    (*result).sub_start[idx as usize]
}

/// Get submatch end position.
///
/// # Safety
/// `result` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_get_end(
    result: *const MatchResult,
    idx: c_int,
) -> MatchPos {
    if result.is_null() || idx < 0 || idx as usize >= NSUBEXP {
        return MatchPos::invalid();
    }
    (*result).sub_end[idx as usize]
}

/// Parse magic prefix from pattern.
///
/// # Safety
/// `pattern` must be valid for `len` bytes.
/// `offset_out` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_magic_prefix(
    pattern: *const u8,
    len: usize,
    offset_out: *mut usize,
) -> c_int {
    if pattern.is_null() {
        if !offset_out.is_null() {
            *offset_out = 0;
        }
        return 0;
    }

    let slice = std::slice::from_raw_parts(pattern, len);
    let (mode, offset) = parse_magic_prefix(slice);

    if !offset_out.is_null() {
        *offset_out = offset;
    }

    mode
}

/// Check if character is special in given magic mode.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_is_special_char(c: c_int, magic_mode: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 0;
    }
    c_int::from(is_special_char(c as u8, magic_mode))
}

/// Check if pattern has uppercase (for smartcase).
///
/// # Safety
/// `pattern` must be valid for `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_pattern_has_uppercase(pattern: *const u8, len: usize) -> c_int {
    if pattern.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(pattern, len);
    c_int::from(has_uppercase(slice))
}

/// Check if character class is supported.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_supports_char_class(c: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 0;
    }
    c_int::from(supports_char_class(c as u8))
}

// =============================================================================
// Additional FFI Exports (R7)
// =============================================================================

/// Get RE_MAGIC constant.
#[no_mangle]
pub extern "C" fn rs_re_magic() -> c_int {
    RE_MAGIC
}

/// Get RE_IGNORECASE constant.
#[no_mangle]
pub extern "C" fn rs_re_ignorecase() -> c_int {
    RE_IGNORECASE
}

/// Get RE_SMARTCASE constant.
#[no_mangle]
pub extern "C" fn rs_re_smartcase() -> c_int {
    RE_SMARTCASE
}

/// Get RE_BOL constant.
#[no_mangle]
pub extern "C" fn rs_re_bol() -> c_int {
    RE_BOL
}

/// Get RE_NO_CHANGE constant.
#[no_mangle]
pub extern "C" fn rs_re_no_change() -> c_int {
    RE_NO_CHANGE
}

/// Get RE_NEWLINE constant.
#[no_mangle]
pub extern "C" fn rs_re_newline() -> c_int {
    RE_NEWLINE
}

/// Get RE_EXTENDED constant.
#[no_mangle]
pub extern "C" fn rs_re_extended() -> c_int {
    RE_EXTENDED
}

/// Get RE_EXTENDED_START constant.
#[no_mangle]
pub extern "C" fn rs_re_extended_start() -> c_int {
    RE_EXTENDED_START
}

/// Get AUTOMATIC_ENGINE constant.
#[no_mangle]
pub extern "C" fn rs_automatic_engine() -> c_int {
    AUTOMATIC_ENGINE
}

/// Get BACKTRACKING_ENGINE constant.
#[no_mangle]
pub extern "C" fn rs_backtracking_engine() -> c_int {
    BACKTRACKING_ENGINE
}

/// Get NFA_ENGINE constant.
#[no_mangle]
pub extern "C" fn rs_nfa_engine() -> c_int {
    NFA_ENGINE
}

/// Get NSUBEXP constant.
#[no_mangle]
pub extern "C" fn rs_match_nsubexp() -> c_int {
    NSUBEXP as c_int
}

/// Translate backslash escape character for match helpers.
#[no_mangle]
pub extern "C" fn rs_match_backslash_trans(c: u8) -> c_int {
    backslash_trans(c)
}

/// Set match result as matched.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_set_matched(result: *mut MatchResult, matched: c_int) {
    if !result.is_null() {
        (*result).matched = matched != 0;
    }
}

/// Set submatch start position.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_set_start(
    result: *mut MatchResult,
    idx: c_int,
    lnum: i32,
    col: c_int,
) {
    if !result.is_null() && idx >= 0 && (idx as usize) < NSUBEXP {
        (*result).sub_start[idx as usize] = MatchPos::new(lnum, col);
    }
}

/// Set submatch end position.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_set_end(
    result: *mut MatchResult,
    idx: c_int,
    lnum: i32,
    col: c_int,
) {
    if !result.is_null() && idx >= 0 && (idx as usize) < NSUBEXP {
        (*result).sub_end[idx as usize] = MatchPos::new(lnum, col);
    }
}

/// Set the main range on match result.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_set_range(
    result: *mut MatchResult,
    start_lnum: i32,
    start_col: c_int,
    end_lnum: i32,
    end_col: c_int,
) {
    if !result.is_null() {
        (*result).range =
            MatchRange::new(MatchPos::new(start_lnum, start_col), MatchPos::new(end_lnum, end_col));
    }
}

/// Get the start position line number from match result range.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_range_start_lnum(result: *const MatchResult) -> i32 {
    if result.is_null() {
        0
    } else {
        (*result).range.start.lnum
    }
}

/// Get the start position column from match result range.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_range_start_col(result: *const MatchResult) -> c_int {
    if result.is_null() {
        0
    } else {
        (*result).range.start.col
    }
}

/// Get the end position line number from match result range.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_range_end_lnum(result: *const MatchResult) -> i32 {
    if result.is_null() {
        0
    } else {
        (*result).range.end.lnum
    }
}

/// Get the end position column from match result range.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_result_range_end_col(result: *const MatchResult) -> c_int {
    if result.is_null() {
        0
    } else {
        (*result).range.end.col
    }
}

/// Set newline on match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_set_newline(
    config: *mut MatchConfig,
    value: c_int,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    (*config).newline = value != 0;
    1
}

/// Set start_col on match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_set_start_col(
    config: *mut MatchConfig,
    col: c_int,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    (*config).start_col = col;
    1
}

/// Set max_col on match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_set_max_col(
    config: *mut MatchConfig,
    col: c_int,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    (*config).max_col = col;
    1
}

/// Set timeout on match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_set_timeout(
    config: *mut MatchConfig,
    timeout_ms: i64,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    (*config).timeout_ms = timeout_ms;
    1
}

/// Get ignore_case from match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_get_ignore_case(config: *const MatchConfig) -> c_int {
    if config.is_null() {
        0
    } else {
        c_int::from((*config).ignore_case)
    }
}

/// Get smart_case from match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_get_smart_case(config: *const MatchConfig) -> c_int {
    if config.is_null() {
        0
    } else {
        c_int::from((*config).smart_case)
    }
}

/// Get start_col from match configuration.
///
/// # Safety
/// `config` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_match_config_get_start_col(config: *const MatchConfig) -> c_int {
    if config.is_null() {
        0
    } else {
        (*config).start_col
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_config() {
        let config = MatchConfig::new()
            .with_ignore_case(true)
            .with_smart_case(true)
            .with_newline(false)
            .with_start_col(5);

        assert!(config.ignore_case);
        assert!(config.smart_case);
        assert!(!config.newline);
        assert_eq!(config.start_col, 5);

        let flags = config.to_flags();
        assert!(flags & RE_IGNORECASE != 0);
        assert!(flags & RE_SMARTCASE != 0);
        assert!(flags & RE_NEWLINE == 0);
    }

    #[test]
    fn test_match_pos() {
        let pos = MatchPos::new(10, 5);
        assert!(pos.is_valid());
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);

        let invalid = MatchPos::invalid();
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_match_range() {
        let start = MatchPos::new(1, 0);
        let end = MatchPos::new(1, 10);
        let range = MatchRange::new(start, end);

        assert!(range.is_valid());
        assert_eq!(range.len(), 10);
        assert!(!range.is_empty());

        let invalid = MatchRange::invalid();
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_match_result() {
        let mut result =
            MatchResult::matched_at(MatchRange::new(MatchPos::new(1, 0), MatchPos::new(1, 5)));

        assert!(result.matched);
        assert_eq!(result.submatch_count(), 1); // First submatch auto-set

        result.set_submatch(1, MatchPos::new(1, 1), MatchPos::new(1, 3));
        assert_eq!(result.submatch_count(), 2);

        let sub = result.get_submatch(1).unwrap();
        assert_eq!(sub.start.col, 1);
        assert_eq!(sub.end.col, 3);
    }

    #[test]
    fn test_parse_magic_prefix() {
        assert_eq!(parse_magic_prefix(b"\\mpattern"), (2, 2));
        assert_eq!(parse_magic_prefix(b"\\Mpattern"), (1, 2));
        assert_eq!(parse_magic_prefix(b"\\vpattern"), (4, 2));
        assert_eq!(parse_magic_prefix(b"\\Vpattern"), (3, 2));
        assert_eq!(parse_magic_prefix(b"pattern"), (0, 0));
        assert_eq!(parse_magic_prefix(b"\\xpattern"), (0, 0));
    }

    #[test]
    fn test_is_special_char() {
        // Very magic (4): most non-alphanumerics are special
        assert!(is_special_char(b'^', 4));
        assert!(is_special_char(b'(', 4));
        assert!(!is_special_char(b'a', 4));
        assert!(!is_special_char(b'_', 4));

        // Very nomagic (3): only backslash
        assert!(is_special_char(b'\\', 3));
        assert!(!is_special_char(b'^', 3));
        assert!(!is_special_char(b'*', 3));

        // Magic (2): standard special chars
        assert!(is_special_char(b'^', 2));
        assert!(is_special_char(b'*', 2));
        assert!(is_special_char(b'.', 2));
        assert!(!is_special_char(b'(', 2));

        // Nomagic (1): ^, $, \
        assert!(is_special_char(b'^', 1));
        assert!(is_special_char(b'$', 1));
        assert!(is_special_char(b'\\', 1));
        assert!(!is_special_char(b'*', 1));
    }

    #[test]
    fn test_escape_for_search() {
        assert_eq!(escape_for_search(b"hello"), b"\\Vhello");
        assert_eq!(escape_for_search(b"a\\b"), b"\\Va\\\\b");
        assert_eq!(escape_for_search(b"a/b"), b"\\Va\\/b");
    }

    #[test]
    fn test_has_uppercase() {
        assert!(has_uppercase(b"Hello"));
        assert!(has_uppercase(b"hEllo"));
        assert!(!has_uppercase(b"hello"));
        assert!(!has_uppercase(b"\\Ahello")); // \A is escaped
        assert!(has_uppercase(b"\\ahEllo")); // E after \a
    }

    #[test]
    fn test_backslash_trans() {
        assert_eq!(backslash_trans(b'r'), 13);
        assert_eq!(backslash_trans(b't'), 9);
        assert_eq!(backslash_trans(b'e'), 27);
        assert_eq!(backslash_trans(b'b'), 8);
        assert_eq!(backslash_trans(b'n'), -1);
        assert_eq!(backslash_trans(b'x'), b'x' as c_int);
    }

    #[test]
    fn test_supports_char_class() {
        // Supported
        assert!(supports_char_class(b'd')); // digit
        assert!(supports_char_class(b'w')); // word
        assert!(supports_char_class(b's')); // whitespace
        assert!(supports_char_class(b'a')); // alpha
        assert!(supports_char_class(b'x')); // hex

        // Not supported
        assert!(!supports_char_class(b'z'));
        assert!(!supports_char_class(b'0'));
    }
}
