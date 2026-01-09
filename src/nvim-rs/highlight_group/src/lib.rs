//! Highlight group management for Neovim.
//!
//! This crate manages highlight groups, which control how text appears in Neovim.
//! It handles:
//! - Highlight group registration and lookup
//! - Color palette management (terminal and GUI colors)
//! - Attribute parsing from `:highlight` commands
//! - Integration with the highlighting system
//!
//! Note: Some FFI functions like `rs_highlight_num_groups` and `rs_lookup_color`
//! are implemented in the `nvim-highlight` crate, since they were migrated earlier.
//! This crate extends that functionality with additional highlight group management.

#![allow(clippy::missing_safety_doc)]

pub mod color;
pub mod command;
pub mod normal;
pub mod parse;
pub mod registry;
mod types;

use std::ffi::c_int;

pub use types::*;

// Re-export HlAttrs from highlight crate for convenience
pub use nvim_highlight::HlAttrs;

// ============================================================================
// Utility Functions
// ============================================================================

/// Look up a basic color name (case-insensitive) and return its index.
///
/// # Arguments
/// * `name` - The color name to look up
///
/// # Returns
/// The index into `COLOR_NAMES` or -1 if not found
pub fn lookup_color_name(name: &str) -> c_int {
    for (i, &color_name) in COLOR_NAMES.iter().enumerate() {
        if name.eq_ignore_ascii_case(color_name) {
            return i as c_int;
        }
    }
    -1
}

/// Check if a string matches "NONE" (case-insensitive).
///
/// This is used in highlight parsing to check for the "NONE" keyword.
#[inline]
pub fn is_none_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("NONE")
}

/// Check if a character is a valid highlight name character.
///
/// Valid characters are: alphanumeric, '_', '.', '@'
#[inline]
pub fn is_valid_hl_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '@'
}

/// Validate a highlight group name.
///
/// A valid name:
/// - Is not empty
/// - Starts with a letter, '@', or '_'
/// - Contains only valid highlight name characters
/// - Is not longer than MAX_SYN_NAME
pub fn is_valid_hl_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_SYN_NAME {
        return false;
    }

    let mut chars = name.chars();

    // First character must be a letter, '@', or '_'
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '@' || c == '_' => {}
        _ => return false,
    }

    // Remaining characters must be valid
    chars.all(is_valid_hl_name_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_color_name() {
        assert_eq!(lookup_color_name("Black"), 0);
        assert_eq!(lookup_color_name("black"), 0);
        assert_eq!(lookup_color_name("BLACK"), 0);
        assert_eq!(lookup_color_name("White"), 26);
        assert_eq!(lookup_color_name("NONE"), 27);
        assert_eq!(lookup_color_name("InvalidColor"), -1);
    }

    #[test]
    fn test_is_none_name() {
        assert!(is_none_name("NONE"));
        assert!(is_none_name("none"));
        assert!(is_none_name("None"));
        assert!(is_none_name("nOnE"));
        assert!(!is_none_name(""));
        assert!(!is_none_name("NO"));
        assert!(!is_none_name("NONES"));
    }

    #[test]
    fn test_is_valid_hl_name_char() {
        assert!(is_valid_hl_name_char('a'));
        assert!(is_valid_hl_name_char('Z'));
        assert!(is_valid_hl_name_char('0'));
        assert!(is_valid_hl_name_char('_'));
        assert!(is_valid_hl_name_char('.'));
        assert!(is_valid_hl_name_char('@'));
        assert!(!is_valid_hl_name_char(' '));
        assert!(!is_valid_hl_name_char('-'));
        assert!(!is_valid_hl_name_char('#'));
    }

    #[test]
    fn test_is_valid_hl_name() {
        // Valid names
        assert!(is_valid_hl_name("Normal"));
        assert!(is_valid_hl_name("StatusLine"));
        assert!(is_valid_hl_name("_Private"));
        assert!(is_valid_hl_name("@treesitter.keyword"));
        assert!(is_valid_hl_name("Foo123"));
        assert!(is_valid_hl_name("A"));

        // Invalid names
        assert!(!is_valid_hl_name("")); // empty
        assert!(!is_valid_hl_name("123Abc")); // starts with digit
        assert!(!is_valid_hl_name(".dotfirst")); // starts with dot
        assert!(!is_valid_hl_name("has space")); // contains space
        assert!(!is_valid_hl_name("has-dash")); // contains dash

        // Too long name
        let long_name = "a".repeat(MAX_SYN_NAME + 1);
        assert!(!is_valid_hl_name(&long_name));

        // Maximum length name is valid
        let max_name = "a".repeat(MAX_SYN_NAME);
        assert!(is_valid_hl_name(&max_name));
    }

    #[test]
    fn test_color_tables_validity() {
        // Verify all color tables have the same length as COLOR_NAMES
        assert_eq!(COLOR_NAMES.len(), 28);
        assert_eq!(COLOR_NUMBERS_16.len(), 28);
        assert_eq!(COLOR_NUMBERS_88.len(), 28);
        assert_eq!(COLOR_NUMBERS_256.len(), 28);
        assert_eq!(COLOR_NUMBERS_8.len(), 28);

        // Verify NONE is the last entry with -1
        assert_eq!(COLOR_NAMES[27], "NONE");
        assert_eq!(COLOR_NUMBERS_16[27], -1);
        assert_eq!(COLOR_NUMBERS_88[27], -1);
        assert_eq!(COLOR_NUMBERS_256[27], -1);
        assert_eq!(COLOR_NUMBERS_8[27], -1);
    }
}
