//! Option name completion source
//!
//! This module provides helpers for completing Vim option names,
//! including both short and long forms, and boolean option prefixes.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Option Prefix Types
// =============================================================================

/// Prefix for boolean option completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptionPrefix {
    /// No prefix
    #[default]
    None,
    /// "no" prefix (e.g., "noautoindent")
    No,
    /// "inv" prefix (e.g., "invautoindent")
    Inv,
}

impl OptionPrefix {
    /// Convert from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::No),
            2 => Some(Self::Inv),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        match self {
            Self::None => 0,
            Self::No => 1,
            Self::Inv => 2,
        }
    }

    /// Get the prefix string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::No => "no",
            Self::Inv => "inv",
        }
    }

    /// Get the prefix bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &'static [u8] {
        match self {
            Self::None => b"",
            Self::No => b"no",
            Self::Inv => b"inv",
        }
    }

    /// Get the prefix length.
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::No => 2,
            Self::Inv => 3,
        }
    }

    /// Check if prefix is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }
}

// =============================================================================
// Option Parsing
// =============================================================================

/// Parse an option prefix from the start of a string.
///
/// Returns the prefix type and the remaining option name.
#[must_use]
pub fn parse_option_prefix(s: &[u8]) -> (OptionPrefix, &[u8]) {
    if s.starts_with(b"inv") && s.len() > 3 {
        (OptionPrefix::Inv, &s[3..])
    } else if s.starts_with(b"no") && s.len() > 2 {
        (OptionPrefix::No, &s[2..])
    } else {
        (OptionPrefix::None, s)
    }
}

/// Check if a string could be an option name.
///
/// Valid option names consist of lowercase letters only.
#[must_use]
pub fn is_valid_option_name(name: &[u8]) -> bool {
    !name.is_empty() && name.iter().all(|&c| c.is_ascii_lowercase())
}

/// Check if an option name matches a prefix.
///
/// Performs case-insensitive matching.
#[must_use]
pub fn option_matches_prefix(option: &[u8], prefix: &[u8]) -> bool {
    if prefix.is_empty() {
        return true;
    }

    if option.len() < prefix.len() {
        return false;
    }

    option[..prefix.len()].eq_ignore_ascii_case(prefix)
}

// =============================================================================
// Option Set/Unset Detection
// =============================================================================

/// Characters that can follow an option name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionSuffix {
    /// Assignment (=)
    Assign,
    /// Append (+=)
    Append,
    /// Prepend (^=)
    Prepend,
    /// Remove (-=)
    Remove,
    /// Toggle (!)
    Toggle,
    /// Query (?)
    Query,
    /// None (just the option name)
    None,
}

impl OptionSuffix {
    /// Parse suffix from a character.
    #[must_use]
    pub const fn from_char(c: u8) -> Self {
        match c {
            b'=' => Self::Assign,
            b'+' => Self::Append,  // Note: += needs special handling
            b'^' => Self::Prepend, // Note: ^= needs special handling
            b'-' => Self::Remove,  // Note: -= needs special handling
            b'!' => Self::Toggle,
            b'?' => Self::Query,
            _ => Self::None,
        }
    }

    /// Check if this suffix expects a value.
    #[must_use]
    pub const fn expects_value(&self) -> bool {
        matches!(
            self,
            Self::Assign | Self::Append | Self::Prepend | Self::Remove
        )
    }
}

/// Parse an option setting from a string.
///
/// Returns (prefix, option_name, suffix, value).
#[must_use]
pub fn parse_option_setting(s: &[u8]) -> (OptionPrefix, &[u8], OptionSuffix, Option<&[u8]>) {
    let (prefix, rest) = parse_option_prefix(s);

    // Find the end of the option name
    let mut name_end = 0;
    for (i, &c) in rest.iter().enumerate() {
        if !c.is_ascii_lowercase() {
            name_end = i;
            break;
        }
        name_end = i + 1;
    }

    let name = &rest[..name_end];
    let after_name = &rest[name_end..];

    if after_name.is_empty() {
        return (prefix, name, OptionSuffix::None, None);
    }

    // Parse suffix
    let first = after_name[0];
    let suffix = OptionSuffix::from_char(first);

    // Check for compound operators (+=, ^=, -=)
    let (actual_suffix, value_start) = if after_name.len() >= 2 && after_name[1] == b'=' {
        match first {
            b'+' => (OptionSuffix::Append, 2),
            b'^' => (OptionSuffix::Prepend, 2),
            b'-' => (OptionSuffix::Remove, 2),
            _ => (suffix, 1),
        }
    } else {
        (suffix, 1)
    };

    let value = if actual_suffix.expects_value() && after_name.len() > value_start {
        Some(&after_name[value_start..])
    } else {
        None
    };

    (prefix, name, actual_suffix, value)
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Parse option prefix from string (FFI).
///
/// Returns the prefix type (0=none, 1=no, 2=inv).
///
/// # Safety
///
/// `s` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_parse_option_prefix(s: *const c_char, len: usize) -> c_int {
    if s.is_null() || len == 0 {
        return OptionPrefix::None.to_raw();
    }

    let bytes = std::slice::from_raw_parts(s.cast::<u8>(), len);
    let (prefix, _) = parse_option_prefix(bytes);
    prefix.to_raw()
}

/// Get the length of the option prefix (FFI).
///
/// Returns the number of bytes to skip for the prefix.
///
/// # Safety
///
/// `s` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_option_prefix_len(s: *const c_char, len: usize) -> usize {
    if s.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(s.cast::<u8>(), len);
    let (prefix, _) = parse_option_prefix(bytes);
    prefix.len()
}

/// Check if a string is a valid option name (FFI).
///
/// # Safety
///
/// `name` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_valid_option_name(name: *const c_char, len: usize) -> c_int {
    if name.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(name.cast::<u8>(), len);
    c_int::from(is_valid_option_name(bytes))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_prefix_conversion() {
        assert_eq!(OptionPrefix::from_raw(0), Some(OptionPrefix::None));
        assert_eq!(OptionPrefix::from_raw(1), Some(OptionPrefix::No));
        assert_eq!(OptionPrefix::from_raw(2), Some(OptionPrefix::Inv));
        assert_eq!(OptionPrefix::from_raw(3), None);

        assert_eq!(OptionPrefix::None.to_raw(), 0);
        assert_eq!(OptionPrefix::No.to_raw(), 1);
        assert_eq!(OptionPrefix::Inv.to_raw(), 2);
    }

    #[test]
    fn test_option_prefix_str() {
        assert_eq!(OptionPrefix::None.as_str(), "");
        assert_eq!(OptionPrefix::No.as_str(), "no");
        assert_eq!(OptionPrefix::Inv.as_str(), "inv");
    }

    #[test]
    fn test_parse_option_prefix() {
        assert_eq!(
            parse_option_prefix(b"noautoindent"),
            (OptionPrefix::No, b"autoindent".as_slice())
        );
        assert_eq!(
            parse_option_prefix(b"invautoindent"),
            (OptionPrefix::Inv, b"autoindent".as_slice())
        );
        assert_eq!(
            parse_option_prefix(b"autoindent"),
            (OptionPrefix::None, b"autoindent".as_slice())
        );
        // Too short for prefix
        assert_eq!(
            parse_option_prefix(b"no"),
            (OptionPrefix::None, b"no".as_slice())
        );
    }

    #[test]
    fn test_is_valid_option_name() {
        assert!(is_valid_option_name(b"autoindent"));
        assert!(is_valid_option_name(b"ai"));
        assert!(!is_valid_option_name(b""));
        assert!(!is_valid_option_name(b"AutoIndent")); // uppercase
        assert!(!is_valid_option_name(b"auto-indent")); // hyphen
        assert!(!is_valid_option_name(b"auto1")); // digit
    }

    #[test]
    fn test_option_matches_prefix() {
        assert!(option_matches_prefix(b"autoindent", b""));
        assert!(option_matches_prefix(b"autoindent", b"a"));
        assert!(option_matches_prefix(b"autoindent", b"auto"));
        assert!(option_matches_prefix(b"autoindent", b"autoindent"));
        assert!(!option_matches_prefix(b"autoindent", b"autoindents"));
        assert!(!option_matches_prefix(b"autoindent", b"x"));
    }

    #[test]
    fn test_option_suffix() {
        assert_eq!(OptionSuffix::from_char(b'='), OptionSuffix::Assign);
        assert_eq!(OptionSuffix::from_char(b'!'), OptionSuffix::Toggle);
        assert_eq!(OptionSuffix::from_char(b'?'), OptionSuffix::Query);
        assert_eq!(OptionSuffix::from_char(b'x'), OptionSuffix::None);

        assert!(OptionSuffix::Assign.expects_value());
        assert!(OptionSuffix::Append.expects_value());
        assert!(!OptionSuffix::Toggle.expects_value());
        assert!(!OptionSuffix::Query.expects_value());
    }

    #[test]
    fn test_parse_option_setting() {
        // Simple option
        let (prefix, name, suffix, value) = parse_option_setting(b"autoindent");
        assert_eq!(prefix, OptionPrefix::None);
        assert_eq!(name, b"autoindent");
        assert_eq!(suffix, OptionSuffix::None);
        assert!(value.is_none());

        // With no prefix
        let (prefix, name, suffix, value) = parse_option_setting(b"noautoindent");
        assert_eq!(prefix, OptionPrefix::No);
        assert_eq!(name, b"autoindent");
        assert_eq!(suffix, OptionSuffix::None);
        assert!(value.is_none());

        // Assignment
        let (prefix, name, suffix, value) = parse_option_setting(b"tabstop=4");
        assert_eq!(prefix, OptionPrefix::None);
        assert_eq!(name, b"tabstop");
        assert_eq!(suffix, OptionSuffix::Assign);
        assert_eq!(value, Some(b"4".as_slice()));

        // Append
        let (prefix, name, suffix, value) = parse_option_setting(b"path+=,/usr");
        assert_eq!(prefix, OptionPrefix::None);
        assert_eq!(name, b"path");
        assert_eq!(suffix, OptionSuffix::Append);
        assert_eq!(value, Some(b",/usr".as_slice()));
    }
}
