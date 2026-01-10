//! Abbreviation helpers for Neovim
//!
//! This module provides helpers for working with abbreviations,
//! including word matching, trigger detection, and expansion.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::manual_range_contains)]

use std::ffi::c_int;

// =============================================================================
// Abbreviation Types
// =============================================================================

/// Types of abbreviations.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AbbrType {
    /// Full-id: Trigger contains only keyword characters
    #[default]
    FullId = 0,
    /// End-id: Trigger ends with keyword character
    EndId = 1,
    /// Non-id: Trigger contains no keyword characters
    NonId = 2,
}

impl AbbrType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::FullId,
            1 => Self::EndId,
            _ => Self::NonId,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Abbreviation Context
// =============================================================================

/// Context for abbreviation expansion.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AbbrContext {
    /// Type of the abbreviation
    pub abbr_type: c_int,
    /// Start column of the trigger
    pub start_col: c_int,
    /// End column of the trigger
    pub end_col: c_int,
    /// Whether trigger is at word boundary
    pub at_word_boundary: bool,
}

impl AbbrContext {
    /// Create a new abbreviation context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            abbr_type: 0,
            start_col: 0,
            end_col: 0,
            at_word_boundary: false,
        }
    }

    /// Get the trigger length.
    #[must_use]
    pub const fn trigger_len(&self) -> c_int {
        if self.end_col > self.start_col {
            self.end_col - self.start_col
        } else {
            0
        }
    }

    /// Check if this is a valid abbreviation match.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.trigger_len() > 0
    }
}

// =============================================================================
// Character Classification
// =============================================================================

/// Check if a character is a keyword character (for abbreviations).
///
/// By default, keyword characters are: a-z, A-Z, 0-9, _
/// This is a simplified version; the full check uses 'iskeyword' option.
#[must_use]
#[inline]
pub const fn is_keyword_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Check if a character is an abbreviation word boundary.
///
/// Word boundaries are: space, tab, and other non-keyword characters.
#[must_use]
#[inline]
pub const fn is_abbr_boundary(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == 0 || !is_keyword_char(c)
}

/// Classify an abbreviation trigger.
///
/// Returns the abbreviation type based on the characters in the trigger.
#[must_use]
pub fn classify_abbr(trigger: &[u8]) -> AbbrType {
    if trigger.is_empty() {
        return AbbrType::NonId;
    }

    let mut has_keyword = false;
    let mut has_non_keyword = false;

    for &c in trigger {
        if c == 0 {
            break;
        }
        if is_keyword_char(c) {
            has_keyword = true;
        } else {
            has_non_keyword = true;
        }
    }

    if !has_non_keyword {
        AbbrType::FullId
    } else if has_keyword {
        // Check if last char is keyword
        let last = trigger
            .iter()
            .rev()
            .find(|&&c| c != 0)
            .copied()
            .unwrap_or(0);
        if is_keyword_char(last) {
            AbbrType::EndId
        } else {
            AbbrType::NonId
        }
    } else {
        AbbrType::NonId
    }
}

// =============================================================================
// Trigger Matching
// =============================================================================

/// Result of abbreviation trigger check.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbbrMatchResult {
    /// No match
    NoMatch = 0,
    /// Full match
    Match = 1,
    /// Partial match (trigger is prefix)
    Partial = 2,
}

impl AbbrMatchResult {
    /// Check if this is any kind of match.
    #[must_use]
    pub const fn is_match(&self) -> bool {
        !matches!(self, Self::NoMatch)
    }

    /// Check if this is a full match.
    #[must_use]
    pub const fn is_full_match(&self) -> bool {
        matches!(self, Self::Match)
    }
}

/// Check if a typed word matches an abbreviation trigger.
///
/// # Arguments
///
/// * `trigger` - The abbreviation trigger
/// * `typed` - The typed text to check
/// * `abbr_type` - The type of the abbreviation
///
/// # Returns
///
/// The match result.
#[must_use]
pub fn check_abbr_match(trigger: &[u8], typed: &[u8], abbr_type: AbbrType) -> AbbrMatchResult {
    if trigger.is_empty() || typed.is_empty() {
        return AbbrMatchResult::NoMatch;
    }

    // Get lengths (excluding null terminators)
    let trig_len = trigger.iter().take_while(|&&c| c != 0).count();
    let typed_len = typed.iter().take_while(|&&c| c != 0).count();

    if trig_len == 0 {
        return AbbrMatchResult::NoMatch;
    }

    // Check if typed ends with trigger
    if typed_len < trig_len {
        // Check if trigger starts with typed (partial match)
        for (i, &tc) in typed.iter().enumerate().take(typed_len) {
            if tc == 0 || trigger[i] != tc {
                return AbbrMatchResult::NoMatch;
            }
        }
        return AbbrMatchResult::Partial;
    }

    // Check if typed ends with trigger
    let start = typed_len - trig_len;
    for i in 0..trig_len {
        if typed[start + i] != trigger[i] {
            return AbbrMatchResult::NoMatch;
        }
    }

    // For full-id abbreviations, check word boundary before trigger
    if abbr_type == AbbrType::FullId && start > 0 && !is_abbr_boundary(typed[start - 1]) {
        return AbbrMatchResult::NoMatch;
    }

    AbbrMatchResult::Match
}

// =============================================================================
// Expansion State
// =============================================================================

/// State for abbreviation expansion.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AbbrExpansionState {
    /// Whether expansion is active
    pub active: bool,
    /// Column where trigger starts
    pub start_col: c_int,
    /// Length of trigger to delete
    pub delete_len: c_int,
    /// Whether to add a trailing char
    pub add_trailing: bool,
}

impl AbbrExpansionState {
    /// Create inactive expansion state.
    #[must_use]
    pub const fn inactive() -> Self {
        Self {
            active: false,
            start_col: 0,
            delete_len: 0,
            add_trailing: false,
        }
    }

    /// Create active expansion state.
    #[must_use]
    pub const fn new(start_col: c_int, delete_len: c_int, add_trailing: bool) -> Self {
        Self {
            active: true,
            start_col,
            delete_len,
            add_trailing,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get abbreviation type from trigger.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_abbr_classify(trigger: *const u8, len: usize) -> c_int {
    if trigger.is_null() || len == 0 {
        return AbbrType::NonId.to_raw();
    }
    let slice = std::slice::from_raw_parts(trigger, len);
    classify_abbr(slice).to_raw()
}

/// Check if a character is a keyword character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_keyword_char(c: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 0;
    }
    c_int::from(is_keyword_char(c as u8))
}

/// Check if a character is an abbreviation boundary.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_abbr_boundary(c: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 1;
    }
    c_int::from(is_abbr_boundary(c as u8))
}

/// Check abbreviation trigger match.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_check_abbr_match(
    trigger: *const u8,
    trig_len: usize,
    typed: *const u8,
    typed_len: usize,
    abbr_type: c_int,
) -> c_int {
    if trigger.is_null() || typed.is_null() {
        return AbbrMatchResult::NoMatch as c_int;
    }
    let trig_slice = std::slice::from_raw_parts(trigger, trig_len);
    let typed_slice = std::slice::from_raw_parts(typed, typed_len);
    check_abbr_match(trig_slice, typed_slice, AbbrType::from_raw(abbr_type)) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbr_type() {
        assert_eq!(AbbrType::from_raw(0), AbbrType::FullId);
        assert_eq!(AbbrType::from_raw(1), AbbrType::EndId);
        assert_eq!(AbbrType::from_raw(2), AbbrType::NonId);
        assert_eq!(AbbrType::from_raw(99), AbbrType::NonId);

        assert_eq!(AbbrType::FullId.to_raw(), 0);
        assert_eq!(AbbrType::EndId.to_raw(), 1);
        assert_eq!(AbbrType::NonId.to_raw(), 2);
    }

    #[test]
    fn test_abbr_context() {
        let mut ctx = AbbrContext::new();
        assert_eq!(ctx.trigger_len(), 0);
        assert!(!ctx.is_valid());

        ctx.start_col = 5;
        ctx.end_col = 10;
        assert_eq!(ctx.trigger_len(), 5);
        assert!(ctx.is_valid());
    }

    #[test]
    fn test_is_keyword_char() {
        assert!(is_keyword_char(b'a'));
        assert!(is_keyword_char(b'Z'));
        assert!(is_keyword_char(b'0'));
        assert!(is_keyword_char(b'_'));
        assert!(!is_keyword_char(b' '));
        assert!(!is_keyword_char(b'.'));
        assert!(!is_keyword_char(b'-'));
    }

    #[test]
    fn test_is_abbr_boundary() {
        assert!(is_abbr_boundary(b' '));
        assert!(is_abbr_boundary(b'\t'));
        assert!(is_abbr_boundary(0));
        assert!(is_abbr_boundary(b'.'));
        assert!(!is_abbr_boundary(b'a'));
        assert!(!is_abbr_boundary(b'0'));
    }

    #[test]
    fn test_classify_abbr() {
        // Full-id: all keyword chars
        assert_eq!(classify_abbr(b"abc"), AbbrType::FullId);
        assert_eq!(classify_abbr(b"abc123"), AbbrType::FullId);
        assert_eq!(classify_abbr(b"_foo"), AbbrType::FullId);

        // End-id: ends with keyword char
        assert_eq!(classify_abbr(b"#include"), AbbrType::EndId);
        assert_eq!(classify_abbr(b".foo"), AbbrType::EndId);

        // Non-id: no keyword chars or doesn't end with one
        assert_eq!(classify_abbr(b"..."), AbbrType::NonId);
        assert_eq!(classify_abbr(b"foo."), AbbrType::NonId);
        assert_eq!(classify_abbr(b""), AbbrType::NonId);
    }

    #[test]
    fn test_abbr_match() {
        // Full match
        assert_eq!(
            check_abbr_match(b"abc", b"xyzabc", AbbrType::FullId),
            AbbrMatchResult::NoMatch // No boundary before 'a'
        );
        assert_eq!(
            check_abbr_match(b"abc", b" abc", AbbrType::FullId),
            AbbrMatchResult::Match
        );

        // End-id doesn't need boundary
        assert_eq!(
            check_abbr_match(b"#i", b"xyz#i", AbbrType::EndId),
            AbbrMatchResult::Match
        );

        // Partial match
        assert_eq!(
            check_abbr_match(b"abc", b"ab", AbbrType::FullId),
            AbbrMatchResult::Partial
        );

        // No match
        assert_eq!(
            check_abbr_match(b"abc", b"xyz", AbbrType::FullId),
            AbbrMatchResult::NoMatch
        );
    }

    #[test]
    fn test_expansion_state() {
        let inactive = AbbrExpansionState::inactive();
        assert!(!inactive.active);

        let active = AbbrExpansionState::new(5, 3, true);
        assert!(active.active);
        assert_eq!(active.start_col, 5);
        assert_eq!(active.delete_len, 3);
        assert!(active.add_trailing);
    }
}
