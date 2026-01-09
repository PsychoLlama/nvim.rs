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
}
