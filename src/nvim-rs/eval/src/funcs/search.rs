//! Search functions for VimL.
//!
//! This module implements search-related functions from `src/nvim/eval/funcs.c`:
//! - Search flags and options
//! - Match result types
//! - Pattern matching helpers
//!
//! ## Note
//!
//! These are helper types and functions for search operations.
//! Actual pattern matching uses the regexp crate.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::branches_sharing_code)]

use std::ffi::c_int;

// =============================================================================
// Search Flags
// =============================================================================

/// Flags for search() function.
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchFlags {
    /// Search backward
    pub backward: bool,
    /// Accept match at cursor position
    pub accept_cursor: bool,
    /// Don't move cursor
    pub dont_move: bool,
    /// Return end position instead of start
    pub end_pos: bool,
    /// Stop at first match on each line
    pub stop_line: bool,
    /// Set '< and '> marks at match
    pub set_marks: bool,
    /// Include match count in result
    pub count: bool,
}

impl SearchFlags {
    /// Parse from flag string (like "bcenwW").
    pub fn parse(flags: &[u8]) -> Self {
        let mut result = Self::default();
        for &c in flags {
            match c {
                b'b' => result.backward = true,
                b'c' => result.accept_cursor = true,
                b'e' => result.end_pos = true,
                b'n' => result.dont_move = true,
                b'p' => result.stop_line = true,
                b's' => result.set_marks = true,
                b'z' => result.count = true,
                // b'w' (wrap) and b'W' (no wrap) are handled at call site
                _ => {}
            }
        }
        result
    }

    /// Convert to flag bits for C.
    pub const fn to_bits(&self) -> u32 {
        let mut bits = 0u32;
        if self.backward {
            bits |= 0x01;
        }
        if self.accept_cursor {
            bits |= 0x02;
        }
        if self.dont_move {
            bits |= 0x04;
        }
        if self.end_pos {
            bits |= 0x08;
        }
        if self.stop_line {
            bits |= 0x10;
        }
        if self.set_marks {
            bits |= 0x20;
        }
        if self.count {
            bits |= 0x40;
        }
        bits
    }

    /// Create from flag bits.
    pub const fn from_bits(bits: u32) -> Self {
        Self {
            backward: bits & 0x01 != 0,
            accept_cursor: bits & 0x02 != 0,
            dont_move: bits & 0x04 != 0,
            end_pos: bits & 0x08 != 0,
            stop_line: bits & 0x10 != 0,
            set_marks: bits & 0x20 != 0,
            count: bits & 0x40 != 0,
        }
    }
}

/// FFI export: parse search flags.
///
/// # Safety
/// - `flags` must be valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_parse_flags(flags: *const u8, len: c_int) -> u32 {
    if flags.is_null() || len < 0 {
        return 0;
    }

    let slice = unsafe { std::slice::from_raw_parts(flags, len as usize) };
    SearchFlags::parse(slice).to_bits()
}

// =============================================================================
// Match Result
// =============================================================================

/// Result of a search operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchResult {
    /// Line number of match (0 if not found)
    pub lnum: i64,
    /// Column of match start (0-based)
    pub col: i64,
    /// Column of match end (0-based, exclusive)
    pub end_col: i64,
    /// Number of matches found (if count flag set)
    pub count: i64,
}

impl SearchResult {
    /// Create a successful match result.
    pub const fn found(lnum: i64, col: i64, end_col: i64) -> Self {
        Self {
            lnum,
            col,
            end_col,
            count: 1,
        }
    }

    /// Create a not-found result.
    pub const fn not_found() -> Self {
        Self {
            lnum: 0,
            col: 0,
            end_col: 0,
            count: 0,
        }
    }

    /// Check if match was found.
    pub const fn is_found(&self) -> bool {
        self.lnum > 0
    }
}

// =============================================================================
// Match Types
// =============================================================================

/// Match type for match()/matchstr()/matchlist().
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MatchType {
    /// Return match position (match())
    Position = 0,
    /// Return matched string (matchstr())
    String = 1,
    /// Return list of submatches (matchlist())
    List = 2,
    /// Return match end position (matchend())
    End = 3,
}

impl MatchType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::String,
            2 => Self::List,
            3 => Self::End,
            _ => Self::Position,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Submatch Helpers
// =============================================================================

/// Maximum number of submatches (groups) in a pattern.
pub const MAX_SUBMATCHES: usize = 10;

/// Submatch capture information.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Submatch {
    /// Start offset (byte position, -1 if not matched)
    pub start: i64,
    /// End offset (byte position, exclusive)
    pub end: i64,
}

impl Submatch {
    /// Create a matched submatch.
    pub const fn matched(start: i64, end: i64) -> Self {
        Self { start, end }
    }

    /// Create an unmatched submatch.
    pub const fn unmatched() -> Self {
        Self { start: -1, end: -1 }
    }

    /// Check if this submatch was captured.
    pub const fn is_matched(&self) -> bool {
        self.start >= 0
    }

    /// Get length of match.
    pub const fn len(&self) -> i64 {
        if self.is_matched() {
            self.end - self.start
        } else {
            0
        }
    }

    /// Check if match is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// FFI export: check if submatch matched.
#[no_mangle]
pub extern "C" fn rs_submatch_is_matched(start: i64) -> bool {
    start >= 0
}

// =============================================================================
// Search Count
// =============================================================================

/// Search count information (for searchcount()).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchCount {
    /// Current match index (1-based)
    pub current: i64,
    /// Total matches
    pub total: i64,
    /// Whether count is exact or an estimate
    pub exact: bool,
    /// Whether search wrapped around
    pub incomplete: bool,
}

impl SearchCount {
    /// Create search count.
    pub const fn new(current: i64, total: i64, exact: bool) -> Self {
        Self {
            current,
            total,
            exact,
            incomplete: false,
        }
    }
}

// =============================================================================
// Pattern Matching Helpers (for match(), matchstr(), etc.)
// =============================================================================

/// Simple byte pattern match (literal, no regex).
///
/// Returns the byte offset of the match, or -1 if not found.
pub fn match_literal(haystack: &[u8], needle: &[u8], start: usize) -> i64 {
    if needle.is_empty() {
        return start as i64;
    }
    if start >= haystack.len() || needle.len() > haystack.len() - start {
        return -1;
    }

    for i in start..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle {
            return i as i64;
        }
    }

    -1
}

/// Case-insensitive literal match.
pub fn match_literal_ic(haystack: &[u8], needle: &[u8], start: usize) -> i64 {
    if needle.is_empty() {
        return start as i64;
    }
    if start >= haystack.len() || needle.len() > haystack.len() - start {
        return -1;
    }

    for i in start..=haystack.len() - needle.len() {
        if haystack[i..i + needle.len()].eq_ignore_ascii_case(needle) {
            return i as i64;
        }
    }

    -1
}

/// FFI: Simple literal pattern match.
#[no_mangle]
pub unsafe extern "C" fn rs_match_literal(
    haystack: *const u8,
    haystack_len: c_int,
    needle: *const u8,
    needle_len: c_int,
    start: c_int,
) -> i64 {
    if haystack.is_null() || needle.is_null() || haystack_len < 0 || needle_len < 0 || start < 0 {
        return -1;
    }

    let h = std::slice::from_raw_parts(haystack, haystack_len as usize);
    let n = std::slice::from_raw_parts(needle, needle_len as usize);
    match_literal(h, n, start as usize)
}

// =============================================================================
// Fuzzy Matching Helpers (for matchfuzzy(), matchfuzzypos())
// =============================================================================

/// Fuzzy match scoring constants.
pub mod fuzzy {
    /// Score for exact match
    pub const EXACT_MATCH: i32 = 100;
    /// Score for prefix match
    pub const PREFIX_MATCH: i32 = 50;
    /// Score for consecutive characters
    pub const CONSECUTIVE: i32 = 15;
    /// Score for matching after separator
    pub const AFTER_SEPARATOR: i32 = 30;
    /// Score for matching capital after lowercase (camelCase)
    pub const CAMEL_CASE: i32 = 25;
    /// Base score for any match
    pub const BASE_MATCH: i32 = 10;
    /// Penalty for unmatched characters
    pub const UNMATCHED_PENALTY: i32 = -1;
}

/// Characters that act as separators for fuzzy matching.
fn is_separator(c: u8) -> bool {
    matches!(c, b' ' | b'_' | b'-' | b'/' | b'\\' | b'.' | b':')
}

/// Calculate fuzzy match score.
///
/// Returns (score, positions) where:
/// - score: Match quality (higher is better, 0 if no match)
/// - positions: Byte positions of matched characters
pub fn fuzzy_match(text: &[u8], pattern: &[u8]) -> (i32, Vec<usize>) {
    if pattern.is_empty() {
        return (fuzzy::EXACT_MATCH, Vec::new());
    }
    if text.is_empty() {
        return (0, Vec::new());
    }

    #[allow(clippy::redundant_closure_for_method_calls)]
    let pattern_lower: Vec<u8> = pattern.iter().map(|c| c.to_ascii_lowercase()).collect();
    #[allow(clippy::redundant_closure_for_method_calls)]
    let text_lower: Vec<u8> = text.iter().map(|c| c.to_ascii_lowercase()).collect();

    let mut positions = Vec::with_capacity(pattern.len());
    let mut score = 0i32;
    let mut pattern_idx = 0;
    let mut prev_match_pos: Option<usize> = None;

    for (i, &tc) in text_lower.iter().enumerate() {
        if pattern_idx >= pattern_lower.len() {
            break;
        }

        if tc == pattern_lower[pattern_idx] {
            positions.push(i);

            // Score based on position and context
            if i == 0 {
                score += fuzzy::PREFIX_MATCH;
            } else if let Some(prev) = prev_match_pos {
                if i == prev + 1 {
                    score += fuzzy::CONSECUTIVE;
                }
            }

            // After separator bonus
            if i > 0 && is_separator(text[i - 1]) {
                score += fuzzy::AFTER_SEPARATOR;
            }

            // CamelCase bonus
            if i > 0 && text[i].is_ascii_uppercase() && text[i - 1].is_ascii_lowercase() {
                score += fuzzy::CAMEL_CASE;
            }

            score += fuzzy::BASE_MATCH;
            prev_match_pos = Some(i);
            pattern_idx += 1;
        }
    }

    // Check if all pattern characters matched
    if pattern_idx < pattern_lower.len() {
        return (0, Vec::new());
    }

    // Penalty for extra characters in text
    let unmatched = text.len().saturating_sub(positions.len());
    score += (unmatched as i32) * fuzzy::UNMATCHED_PENALTY;

    // Exact match bonus
    if text_lower == pattern_lower {
        score += fuzzy::EXACT_MATCH;
    }

    (score.max(1), positions)
}

/// FFI: Calculate fuzzy match score only.
#[no_mangle]
pub unsafe extern "C" fn rs_fuzzy_match_score(
    text: *const u8,
    text_len: c_int,
    pattern: *const u8,
    pattern_len: c_int,
) -> i32 {
    if text.is_null() || pattern.is_null() || text_len < 0 || pattern_len < 0 {
        return 0;
    }

    let t = std::slice::from_raw_parts(text, text_len as usize);
    let p = std::slice::from_raw_parts(pattern, pattern_len as usize);
    fuzzy_match(t, p).0
}

// =============================================================================
// Searchpair Helpers
// =============================================================================

/// Searchpair matching mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SearchpairMode {
    /// Find matching closing pair
    Forward = 0,
    /// Find matching opening pair
    Backward = 1,
}

impl SearchpairMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        if val == 1 {
            Self::Backward
        } else {
            Self::Forward
        }
    }
}

/// Stack-based pair matching for searchpair().
///
/// Returns the position of the matching pair, or None if not found.
pub fn find_matching_pair(
    start_pattern: &[u8],
    end_pattern: &[u8],
    text: &[u8],
    start_pos: usize,
    forward: bool,
) -> Option<usize> {
    if start_pattern.is_empty() || end_pattern.is_empty() {
        return None;
    }

    let mut depth = 1i32;

    if forward {
        let mut pos = start_pos;
        while pos < text.len() {
            // Check for end pattern first (looking for closing)
            if pos + end_pattern.len() <= text.len()
                && &text[pos..pos + end_pattern.len()] == end_pattern
            {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
                pos += end_pattern.len();
                continue;
            }
            // Check for nested start pattern
            if pos + start_pattern.len() <= text.len()
                && &text[pos..pos + start_pattern.len()] == start_pattern
            {
                depth += 1;
                pos += start_pattern.len();
                continue;
            }
            pos += 1;
        }
    } else {
        // Backward search
        let mut pos = start_pos;
        while pos > 0 {
            pos -= 1;
            // Check for start pattern (looking for opening)
            if pos + start_pattern.len() <= text.len()
                && &text[pos..pos + start_pattern.len()] == start_pattern
            {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
            }
            // Check for nested end pattern
            if pos + end_pattern.len() <= text.len()
                && &text[pos..pos + end_pattern.len()] == end_pattern
            {
                depth += 1;
            }
        }
    }

    None
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_flags() {
        let flags = SearchFlags::parse(b"bcn");
        assert!(flags.backward);
        assert!(flags.accept_cursor);
        assert!(flags.dont_move);
        assert!(!flags.end_pos);

        let bits = flags.to_bits();
        let restored = SearchFlags::from_bits(bits);
        assert_eq!(restored.backward, flags.backward);
        assert_eq!(restored.accept_cursor, flags.accept_cursor);
    }

    #[test]
    fn test_search_result() {
        let found = SearchResult::found(10, 5, 15);
        assert!(found.is_found());
        assert_eq!(found.lnum, 10);

        let not_found = SearchResult::not_found();
        assert!(!not_found.is_found());
    }

    #[test]
    fn test_match_type() {
        assert_eq!(MatchType::from_c_int(0), MatchType::Position);
        assert_eq!(MatchType::from_c_int(1), MatchType::String);
        assert_eq!(MatchType::from_c_int(2), MatchType::List);
    }

    #[test]
    fn test_submatch() {
        let matched = Submatch::matched(5, 10);
        assert!(matched.is_matched());
        assert_eq!(matched.len(), 5);

        let unmatched = Submatch::unmatched();
        assert!(!unmatched.is_matched());
        assert_eq!(unmatched.len(), 0);
    }

    #[test]
    fn test_search_count() {
        let count = SearchCount::new(3, 10, true);
        assert_eq!(count.current, 3);
        assert_eq!(count.total, 10);
        assert!(count.exact);
    }

    #[test]
    fn test_match_literal() {
        assert_eq!(match_literal(b"hello world", b"world", 0), 6);
        assert_eq!(match_literal(b"hello world", b"hello", 0), 0);
        assert_eq!(match_literal(b"hello world", b"xyz", 0), -1);
        assert_eq!(match_literal(b"hello world", b"world", 7), -1);
        assert_eq!(match_literal(b"abcabc", b"bc", 0), 1);
        assert_eq!(match_literal(b"abcabc", b"bc", 2), 4);
    }

    #[test]
    fn test_match_literal_ic() {
        assert_eq!(match_literal_ic(b"Hello World", b"world", 0), 6);
        assert_eq!(match_literal_ic(b"HELLO", b"hello", 0), 0);
        assert_eq!(match_literal_ic(b"ABC", b"xyz", 0), -1);
    }

    #[test]
    fn test_fuzzy_match_basic() {
        // Exact match
        let (score, positions) = fuzzy_match(b"hello", b"hello");
        assert!(score > 0);
        assert_eq!(positions.len(), 5);

        // Subsequence match
        let (score, positions) = fuzzy_match(b"hello world", b"hwd");
        assert!(score > 0);
        assert_eq!(positions, vec![0, 6, 10]);

        // No match
        let (score, _) = fuzzy_match(b"hello", b"xyz");
        assert_eq!(score, 0);
    }

    #[test]
    fn test_fuzzy_match_scoring() {
        // Prefix match should score higher
        let (prefix_score, _) = fuzzy_match(b"file_name", b"fil");
        let (middle_score, _) = fuzzy_match(b"some_file", b"fil");
        assert!(prefix_score > middle_score);

        // Exact match should score highest
        let (exact_score, _) = fuzzy_match(b"test", b"test");
        let (partial_score, _) = fuzzy_match(b"testing", b"test");
        assert!(exact_score > partial_score);
    }

    #[test]
    fn test_fuzzy_match_camel_case() {
        // CamelCase matching
        let (score, positions) = fuzzy_match(b"getFileName", b"gfn");
        assert!(score > 0);
        assert_eq!(positions, vec![0, 3, 7]); // g, F, N
    }

    #[test]
    fn test_find_matching_pair() {
        // Simple case: "()" - start at 1, find ) at position 1
        let text = b"()";
        let pos = find_matching_pair(b"(", b")", text, 1, true);
        assert_eq!(pos, Some(1));

        // Nested: "(())" - start at 1, should skip inner pair and find outer )
        // Text: "(())"
        //        0123
        // Starting at 1, we see ( at 1 (depth 2), ) at 2 (depth 1), ) at 3 (depth 0)
        let text = b"(())";
        let pos = find_matching_pair(b"(", b")", text, 1, true);
        assert_eq!(pos, Some(3));

        // Start inside inner pair
        let pos = find_matching_pair(b"(", b")", text, 2, true);
        assert_eq!(pos, Some(2));

        // Unbalanced - no match
        let pos = find_matching_pair(b"(", b")", b"(()", 1, true);
        // Text: "(()"
        //        012
        // Starting at 1: ( at 1 (depth 2), ) at 2 (depth 1) - no match for outer
        assert_eq!(pos, None);
    }
}
