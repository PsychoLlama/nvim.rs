//! Command name completion source
//!
//! This module provides helpers for completing Ex command names.
//! Commands include built-in commands, user-defined commands, and
//! abbreviated forms.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Check if a command name matches a prefix
    fn nvim_cmd_matches_prefix(
        cmd: *const c_char,
        prefix: *const c_char,
        prefix_len: usize,
    ) -> c_int;

    // Get command by index
    fn get_command_name(xp: *const (), idx: c_int) -> *mut c_char;

    // Check if command is implemented
    fn nvim_cmdidx_from_cmd(cmd: *const c_char, len: usize) -> c_int;
}

// =============================================================================
// Command Completion Helpers
// =============================================================================

/// Command name case sensitivity for matching.
///
/// By default, command name matching is case-insensitive.
pub const COMMAND_MATCH_IGNORE_CASE: bool = true;

/// Check if a string could be an abbreviated command name.
///
/// Abbreviated command names are typically 1-4 characters long.
#[must_use]
pub const fn is_abbreviated_command_length(len: usize) -> bool {
    len > 0 && len <= 4
}

/// Check if a prefix matches the start of a command name.
///
/// Performs case-insensitive matching by default.
#[must_use]
pub fn command_matches_prefix(cmd: &[u8], prefix: &[u8]) -> bool {
    if prefix.is_empty() {
        return true;
    }

    if cmd.len() < prefix.len() {
        return false;
    }

    // Case-insensitive comparison
    cmd[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// Check if a character is valid in a command name.
///
/// Command names consist of ASCII letters only.
#[must_use]
pub const fn is_valid_command_char(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// Validate a command name prefix for completion.
///
/// Valid prefixes:
/// - All characters must be ASCII alphabetic
/// - Can be empty (matches all commands)
#[must_use]
pub fn is_valid_command_prefix(prefix: &[u8]) -> bool {
    prefix.iter().all(|&c| is_valid_command_char(c))
}

/// Calculate the common prefix length between a command and pattern.
///
/// Used for determining completion behavior.
#[must_use]
pub fn command_common_prefix_len(cmd: &[u8], pattern: &[u8]) -> usize {
    cmd.iter()
        .zip(pattern.iter())
        .take_while(|(c, p)| c.eq_ignore_ascii_case(p))
        .count()
}

// =============================================================================
// User Command Detection
// =============================================================================

/// Check if a command name could be a user-defined command.
///
/// User-defined commands must:
/// - Start with an uppercase letter
/// - Contain only letters, digits, or underscores
#[must_use]
pub fn could_be_user_command(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must start with uppercase
    if !name[0].is_ascii_uppercase() {
        return false;
    }

    // Rest must be alphanumeric or underscore
    name[1..]
        .iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

/// FFI wrapper for user command name check.
///
/// # Safety
///
/// `name` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_could_be_user_command(name: *const c_char, len: usize) -> c_int {
    if name.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(name.cast::<u8>(), len);
    c_int::from(could_be_user_command(bytes))
}

// =============================================================================
// Command Ambiguity Detection
// =============================================================================

/// Check if a prefix is ambiguous (matches multiple commands).
///
/// This is used to determine if abbreviation disambiguation is needed.
#[must_use]
pub const fn is_ambiguous_command_prefix(match_count: usize) -> bool {
    match_count > 1
}

/// Determine if completion should show command abbreviations.
///
/// Abbreviations are shown when:
/// - The prefix is short (1-4 chars)
/// - Multiple matches exist
#[must_use]
pub const fn should_show_abbreviations(prefix_len: usize, match_count: usize) -> bool {
    is_abbreviated_command_length(prefix_len) && is_ambiguous_command_prefix(match_count)
}

// =============================================================================
// Command Completion Flags
// =============================================================================

/// Flags for command completion behavior.
pub mod completion_flags {
    /// Include built-in commands
    pub const INCLUDE_BUILTIN: u32 = 0x01;
    /// Include user-defined commands
    pub const INCLUDE_USER: u32 = 0x02;
    /// Include command abbreviations
    pub const INCLUDE_ABBREV: u32 = 0x04;
    /// Default flags (all sources)
    pub const DEFAULT: u32 = INCLUDE_BUILTIN | INCLUDE_USER;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_abbreviated_command_length() {
        assert!(!is_abbreviated_command_length(0));
        assert!(is_abbreviated_command_length(1));
        assert!(is_abbreviated_command_length(2));
        assert!(is_abbreviated_command_length(3));
        assert!(is_abbreviated_command_length(4));
        assert!(!is_abbreviated_command_length(5));
    }

    #[test]
    fn test_command_matches_prefix() {
        assert!(command_matches_prefix(b"edit", b""));
        assert!(command_matches_prefix(b"edit", b"e"));
        assert!(command_matches_prefix(b"edit", b"ed"));
        assert!(command_matches_prefix(b"edit", b"edi"));
        assert!(command_matches_prefix(b"edit", b"edit"));
        assert!(!command_matches_prefix(b"edit", b"edits"));
        assert!(!command_matches_prefix(b"edit", b"x"));

        // Case insensitive
        assert!(command_matches_prefix(b"Edit", b"e"));
        assert!(command_matches_prefix(b"EDIT", b"edit"));
    }

    #[test]
    fn test_is_valid_command_char() {
        assert!(is_valid_command_char(b'a'));
        assert!(is_valid_command_char(b'Z'));
        assert!(!is_valid_command_char(b'1'));
        assert!(!is_valid_command_char(b'_'));
        assert!(!is_valid_command_char(b' '));
    }

    #[test]
    fn test_is_valid_command_prefix() {
        assert!(is_valid_command_prefix(b""));
        assert!(is_valid_command_prefix(b"e"));
        assert!(is_valid_command_prefix(b"edit"));
        assert!(!is_valid_command_prefix(b"e1"));
        assert!(!is_valid_command_prefix(b"e_"));
    }

    #[test]
    fn test_could_be_user_command() {
        // Valid user commands
        assert!(could_be_user_command(b"MyCommand"));
        assert!(could_be_user_command(b"X"));
        assert!(could_be_user_command(b"Test123"));
        assert!(could_be_user_command(b"My_Command"));

        // Invalid
        assert!(!could_be_user_command(b""));
        assert!(!could_be_user_command(b"mycommand")); // lowercase start
        assert!(!could_be_user_command(b"123")); // starts with digit
    }

    #[test]
    fn test_command_common_prefix_len() {
        assert_eq!(command_common_prefix_len(b"edit", b"edit"), 4);
        assert_eq!(command_common_prefix_len(b"edit", b"ed"), 2);
        assert_eq!(command_common_prefix_len(b"edit", b"ex"), 1);
        assert_eq!(command_common_prefix_len(b"edit", b""), 0);
        assert_eq!(command_common_prefix_len(b"edit", b"x"), 0);
    }

    #[test]
    fn test_should_show_abbreviations() {
        assert!(should_show_abbreviations(1, 2));
        assert!(should_show_abbreviations(4, 3));
        assert!(!should_show_abbreviations(5, 2)); // prefix too long
        assert!(!should_show_abbreviations(2, 1)); // only one match
    }
}
