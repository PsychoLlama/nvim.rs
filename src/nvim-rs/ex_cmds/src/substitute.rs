//! `:substitute` command implementation.
//!
//! The `:substitute` (`:s`) command performs search and replace operations
//! on buffer text using regular expressions.
//!
//! ## Usage
//! - `:s/pattern/replacement/` - Substitute on current line
//! - `:%s/pattern/replacement/g` - Substitute all occurrences in buffer
//! - `:{range}s/pattern/replacement/flags` - Substitute in range with flags
//!
//! ## Flags
//! - `g` - Global: replace all occurrences on each line
//! - `c` - Confirm: ask for confirmation for each replacement
//! - `i` - Ignore case
//! - `I` - Don't ignore case (match case)
//! - `n` - Count only, don't substitute
//! - `e` - Don't report errors if no match
//! - `p` - Print the last line with a substitution
//! - `l` - Like 'p' but list the line
//! - `#` - Like 'p' but show line number
//!
//! ## Implementation Notes
//!
//! This module provides type definitions and flag parsing. The actual
//! regex matching and text replacement is performed by Neovim's core
//! substitution engine.

use std::ffi::c_int;

use crate::range::LineNr;
use crate::SubIgnoreType;

/// Flags for the `:substitute` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubFlags {
    /// Do multiple substitutions per line (g flag).
    pub do_all: bool,
    /// Ask for confirmation (c flag).
    pub do_ask: bool,
    /// Count only, don't substitute (n flag).
    pub do_count: bool,
    /// If false, ignore errors when no match (e flag).
    pub do_error: bool,
    /// Print last line with subs (p flag).
    pub do_print: bool,
    /// List last line with subs (l flag).
    pub do_list: bool,
    /// List last line with line number (# flag).
    pub do_number: bool,
    /// Case sensitivity mode.
    pub do_ic: SubIgnoreType,
}

impl SubFlags {
    /// Create flags with default values (honor options, errors enabled).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            do_all: false,
            do_ask: false,
            do_count: false,
            do_error: true,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Create flags for a simple global substitute.
    #[must_use]
    pub const fn global() -> Self {
        Self {
            do_all: true,
            do_error: true,
            do_ask: false,
            do_count: false,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Create flags for count-only mode.
    #[must_use]
    pub const fn count_only() -> Self {
        Self {
            do_all: true,
            do_count: true,
            do_error: true,
            do_ask: false,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Parse flags from a flag string.
    ///
    /// # Arguments
    /// * `flags` - A string containing flag characters (e.g., "gc" for global + confirm)
    ///
    /// # Returns
    /// The parsed flags, or an error if an invalid flag is found.
    pub fn parse(flags: &str) -> Result<Self, SubstituteError> {
        let mut result = Self::new();

        for c in flags.chars() {
            match c {
                'g' => result.do_all = true,
                'c' => result.do_ask = true,
                'n' => result.do_count = true,
                'e' => result.do_error = false,
                'p' => result.do_print = true,
                'l' => result.do_list = true,
                '#' => result.do_number = true,
                'i' => result.do_ic = SubIgnoreType::IgnoreCase,
                'I' => result.do_ic = SubIgnoreType::MatchCase,
                'r' => { /* use last search pattern - handled elsewhere */ }
                ' ' | '\t' => { /* skip whitespace */ }
                _ => return Err(SubstituteError::InvalidFlag(c)),
            }
        }

        Ok(result)
    }

    /// Check if this is a counting-only operation.
    #[inline]
    #[must_use]
    pub const fn is_count_only(&self) -> bool {
        self.do_count
    }

    /// Check if confirmation is required.
    #[inline]
    #[must_use]
    pub const fn needs_confirm(&self) -> bool {
        self.do_ask
    }
}

/// Result of a substitution operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubResult {
    /// Number of substitutions made.
    pub count: i32,
    /// Number of lines changed.
    pub lines: i32,
    /// Whether the operation was interrupted.
    pub interrupted: bool,
}

impl SubResult {
    /// Create a new empty result.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            count: 0,
            lines: 0,
            interrupted: false,
        }
    }

    /// Check if any substitutions were made.
    #[inline]
    #[must_use]
    pub const fn has_matches(&self) -> bool {
        self.count > 0
    }

    /// Add a match to the result.
    #[inline]
    pub fn add_match(&mut self) {
        self.count += 1;
    }

    /// Record a changed line.
    #[inline]
    pub fn add_line(&mut self) {
        self.lines += 1;
    }

    /// Mark as interrupted.
    #[inline]
    pub fn set_interrupted(&mut self) {
        self.interrupted = true;
    }
}

/// Statistics for a substitution operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubStats {
    /// Total number of substitutions across all operations.
    pub total_subs: i32,
    /// Total number of lines changed across all operations.
    pub total_lines: i32,
}

impl SubStats {
    /// Create new statistics.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_subs: 0,
            total_lines: 0,
        }
    }

    /// Add a result to the statistics.
    pub fn add_result(&mut self, result: &SubResult) {
        self.total_subs += result.count;
        self.total_lines += result.lines;
    }

    /// Reset the statistics.
    pub fn reset(&mut self) {
        self.total_subs = 0;
        self.total_lines = 0;
    }
}

/// Error type for substitution operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstituteError {
    /// Invalid flag character.
    InvalidFlag(char),
    /// Invalid delimiter (alphanumeric).
    InvalidDelimiter(char),
    /// Empty pattern with no previous pattern.
    NoPreviousPattern,
    /// Empty replacement with no previous replacement.
    NoPreviousReplacement,
    /// Invalid regular expression.
    InvalidRegex(String),
    /// Invalid range.
    InvalidRange,
    /// Zero count given.
    ZeroCount,
    /// Operation was interrupted.
    Interrupted,
}

impl std::fmt::Display for SubstituteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstituteError::InvalidFlag(c) => write!(f, "invalid flag: {c}"),
            SubstituteError::InvalidDelimiter(c) => {
                write!(f, "regular expressions can't be delimited by letters: {c}")
            }
            SubstituteError::NoPreviousPattern => write!(f, "no previous pattern"),
            SubstituteError::NoPreviousReplacement => write!(f, "no previous substitute command"),
            SubstituteError::InvalidRegex(msg) => write!(f, "invalid regex: {msg}"),
            SubstituteError::InvalidRange => write!(f, "invalid range"),
            SubstituteError::ZeroCount => write!(f, "zero count"),
            SubstituteError::Interrupted => write!(f, "interrupted"),
        }
    }
}

impl std::error::Error for SubstituteError {}

/// Check if a character is a valid delimiter.
///
/// Delimiters cannot be alphanumeric characters.
#[inline]
#[must_use]
pub fn is_valid_delimiter(c: char) -> bool {
    !c.is_alphanumeric()
}

/// Position within a match result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchPosition {
    /// Line number (1-based).
    pub lnum: LineNr,
    /// Column offset (0-based).
    pub col: i32,
}

impl MatchPosition {
    /// Create a new match position.
    #[must_use]
    pub const fn new(lnum: LineNr, col: i32) -> Self {
        Self { lnum, col }
    }
}

/// A range of matched text for preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchRange {
    /// Start position.
    pub start: MatchPosition,
    /// End position.
    pub end: MatchPosition,
}

impl MatchRange {
    /// Create a new match range.
    #[must_use]
    pub const fn new(start: MatchPosition, end: MatchPosition) -> Self {
        Self { start, end }
    }

    /// Check if this is a single-line match.
    #[inline]
    #[must_use]
    pub const fn is_single_line(&self) -> bool {
        self.start.lnum == self.end.lnum
    }

    /// Get the number of lines spanned by this match.
    #[inline]
    #[must_use]
    pub const fn line_span(&self) -> LineNr {
        self.end.lnum - self.start.lnum + 1
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Parse substitute flags from a string.
///
/// Returns a bitmask of flags:
/// - bit 0: do_all (g)
/// - bit 1: do_ask (c)
/// - bit 2: do_count (n)
/// - bit 3: do_error (inverted: set if errors should be reported)
/// - bit 4: do_print (p)
/// - bit 5: do_list (l)
/// - bit 6: do_number (#)
/// - bits 7-8: do_ic (0=honor, 1=ignore, 2=match)
///
/// Returns -1 on error.
///
/// # Safety
/// The `flags` pointer must be null or point to a valid null-terminated C string.
pub unsafe extern "C" fn rs_parse_sub_flags(flags: *const std::ffi::c_char) -> c_int {
    if flags.is_null() {
        return 0; // No flags = default
    }

    let flags_str = match std::ffi::CStr::from_ptr(flags).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match SubFlags::parse(flags_str) {
        Ok(f) => {
            let mut result: c_int = 0;
            if f.do_all {
                result |= 1 << 0;
            }
            if f.do_ask {
                result |= 1 << 1;
            }
            if f.do_count {
                result |= 1 << 2;
            }
            if f.do_error {
                result |= 1 << 3;
            }
            if f.do_print {
                result |= 1 << 4;
            }
            if f.do_list {
                result |= 1 << 5;
            }
            if f.do_number {
                result |= 1 << 6;
            }
            result |= (f.do_ic.to_c() & 0x3) << 7;
            result
        }
        Err(_) => -1,
    }
}

/// Check if a delimiter character is valid.
///
/// Returns 1 if valid, 0 if invalid (alphanumeric).
pub extern "C" fn rs_is_valid_delimiter(c: c_int) -> c_int {
    if !(0..=127).contains(&c) {
        return 0;
    }
    c_int::from(is_valid_delimiter(char::from(c as u8)))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_flags_new() {
        let flags = SubFlags::new();
        assert!(!flags.do_all);
        assert!(!flags.do_ask);
        assert!(!flags.do_count);
        assert!(flags.do_error); // Default is to report errors
        assert!(!flags.do_print);
        assert!(!flags.do_list);
        assert!(!flags.do_number);
        assert_eq!(flags.do_ic, SubIgnoreType::HonorOptions);
    }

    #[test]
    fn test_sub_flags_global() {
        let flags = SubFlags::global();
        assert!(flags.do_all);
        assert!(!flags.do_ask);
    }

    #[test]
    fn test_sub_flags_count_only() {
        let flags = SubFlags::count_only();
        assert!(flags.do_all);
        assert!(flags.do_count);
        assert!(flags.is_count_only());
    }

    #[test]
    fn test_sub_flags_parse() {
        // Empty string
        let flags = SubFlags::parse("").unwrap();
        assert!(!flags.do_all);

        // Global flag
        let flags = SubFlags::parse("g").unwrap();
        assert!(flags.do_all);

        // Multiple flags
        let flags = SubFlags::parse("gc").unwrap();
        assert!(flags.do_all);
        assert!(flags.do_ask);
        assert!(flags.needs_confirm());

        // Case flags
        let flags = SubFlags::parse("gi").unwrap();
        assert!(flags.do_all);
        assert_eq!(flags.do_ic, SubIgnoreType::IgnoreCase);

        let flags = SubFlags::parse("gI").unwrap();
        assert!(flags.do_all);
        assert_eq!(flags.do_ic, SubIgnoreType::MatchCase);

        // Error suppression
        let flags = SubFlags::parse("e").unwrap();
        assert!(!flags.do_error);

        // Print flags
        let flags = SubFlags::parse("p").unwrap();
        assert!(flags.do_print);

        let flags = SubFlags::parse("l").unwrap();
        assert!(flags.do_list);

        let flags = SubFlags::parse("#").unwrap();
        assert!(flags.do_number);
    }

    #[test]
    fn test_sub_flags_parse_invalid() {
        // Invalid flag character
        let result = SubFlags::parse("gx");
        assert!(matches!(result, Err(SubstituteError::InvalidFlag('x'))));
    }

    #[test]
    fn test_sub_result() {
        let mut result = SubResult::new();
        assert!(!result.has_matches());

        result.add_match();
        assert!(result.has_matches());
        assert_eq!(result.count, 1);

        result.add_line();
        assert_eq!(result.lines, 1);

        result.set_interrupted();
        assert!(result.interrupted);
    }

    #[test]
    fn test_sub_stats() {
        let mut stats = SubStats::new();
        assert_eq!(stats.total_subs, 0);

        let result = SubResult {
            count: 5,
            lines: 3,
            interrupted: false,
        };
        stats.add_result(&result);
        assert_eq!(stats.total_subs, 5);
        assert_eq!(stats.total_lines, 3);

        stats.reset();
        assert_eq!(stats.total_subs, 0);
        assert_eq!(stats.total_lines, 0);
    }

    #[test]
    fn test_is_valid_delimiter() {
        // Valid delimiters
        assert!(is_valid_delimiter('/'));
        assert!(is_valid_delimiter('#'));
        assert!(is_valid_delimiter('@'));
        assert!(is_valid_delimiter('!'));
        assert!(is_valid_delimiter(':'));

        // Invalid delimiters (alphanumeric)
        assert!(!is_valid_delimiter('a'));
        assert!(!is_valid_delimiter('Z'));
        assert!(!is_valid_delimiter('0'));
        assert!(!is_valid_delimiter('9'));
    }

    #[test]
    fn test_match_position() {
        let pos = MatchPosition::new(10, 5);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_match_range() {
        let start = MatchPosition::new(10, 0);
        let end = MatchPosition::new(10, 5);
        let range = MatchRange::new(start, end);

        assert!(range.is_single_line());
        assert_eq!(range.line_span(), 1);

        // Multi-line range
        let end = MatchPosition::new(12, 5);
        let range = MatchRange::new(start, end);

        assert!(!range.is_single_line());
        assert_eq!(range.line_span(), 3);
    }

    #[test]
    fn test_substitute_error_display() {
        let err = SubstituteError::InvalidFlag('x');
        assert_eq!(format!("{err}"), "invalid flag: x");

        let err = SubstituteError::InvalidDelimiter('a');
        assert!(format!("{err}").contains("delimited by letters"));

        let err = SubstituteError::NoPreviousPattern;
        assert_eq!(format!("{err}"), "no previous pattern");

        let err = SubstituteError::ZeroCount;
        assert_eq!(format!("{err}"), "zero count");
    }

    #[test]
    fn test_rs_is_valid_delimiter() {
        assert_eq!(rs_is_valid_delimiter(b'/' as c_int), 1);
        assert_eq!(rs_is_valid_delimiter(b'#' as c_int), 1);
        assert_eq!(rs_is_valid_delimiter(b'a' as c_int), 0);
        assert_eq!(rs_is_valid_delimiter(b'0' as c_int), 0);
    }

    #[test]
    fn test_rs_parse_sub_flags() {
        use std::ffi::CString;

        let flags = CString::new("g").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert!(result >= 0);
        assert_eq!(result & 1, 1); // do_all

        let flags = CString::new("gc").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert!(result >= 0);
        assert_eq!(result & 1, 1); // do_all
        assert_eq!(result & 2, 2); // do_ask

        // Invalid flag
        let flags = CString::new("gx").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert_eq!(result, -1);
    }
}
