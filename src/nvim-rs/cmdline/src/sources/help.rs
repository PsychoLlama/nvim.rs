//! Help tag completion source
//!
//! This module provides helpers for completing help tags,
//! including language-specific help files.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Help Tag Constants
// =============================================================================

/// Maximum length of a help tag.
pub const MAX_HELP_TAG_LEN: usize = 200;

/// Help tag language suffix format: "@xx" where xx is language code.
pub const LANG_SUFFIX_LEN: usize = 3;

// =============================================================================
// Help Tag Parsing
// =============================================================================

/// Check if a string could be a help tag.
///
/// Help tags:
/// - Start with a non-space character
/// - Don't contain certain special characters
/// - Can have a language suffix (@xx)
#[must_use]
pub fn is_valid_help_tag(tag: &[u8]) -> bool {
    if tag.is_empty() || tag.len() > MAX_HELP_TAG_LEN {
        return false;
    }

    // Must not start with space
    if tag[0].is_ascii_whitespace() {
        return false;
    }

    // Check for invalid characters
    for &c in tag {
        // Help tags should be printable ASCII (mostly)
        if c < 0x20 || c == 0x7F {
            return false;
        }
    }

    true
}

/// Parse a language suffix from a help tag.
///
/// Returns the base tag and optional language code.
#[must_use]
pub fn parse_help_tag_lang(tag: &[u8]) -> (&[u8], Option<&[u8]>) {
    // Look for @xx suffix
    if tag.len() >= LANG_SUFFIX_LEN {
        let suffix_start = tag.len() - LANG_SUFFIX_LEN;
        if tag[suffix_start] == b'@' {
            let lang = &tag[suffix_start + 1..];
            // Validate language code (two lowercase letters)
            if lang.len() == 2 && lang[0].is_ascii_lowercase() && lang[1].is_ascii_lowercase() {
                return (&tag[..suffix_start], Some(lang));
            }
        }
    }

    (tag, None)
}

/// Check if a help pattern matches a help tag.
///
/// Help matching is case-insensitive and supports partial matches.
#[must_use]
pub fn help_tag_matches(tag: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return true;
    }

    if tag.len() < pattern.len() {
        return false;
    }

    // Case-insensitive prefix match
    tag[..pattern.len()].eq_ignore_ascii_case(pattern)
}

// =============================================================================
// Help Context Detection
// =============================================================================

/// Help topic type based on prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HelpTopicType {
    /// General help topic
    #[default]
    General,
    /// Option help ('option')
    Option,
    /// Normal mode command
    NormalCmd,
    /// Visual mode command (v_)
    VisualCmd,
    /// Insert mode command (i_)
    InsertCmd,
    /// Command-line mode command (c_)
    CmdlineCmd,
    /// Terminal mode command (t_)
    TerminalCmd,
    /// Ex command (:command)
    ExCommand,
}

impl HelpTopicType {
    /// Detect help topic type from pattern.
    #[must_use]
    pub fn from_pattern(pattern: &[u8]) -> Self {
        if pattern.is_empty() {
            return Self::General;
        }

        // Check for mode prefixes
        if pattern.len() >= 2 && pattern[1] == b'_' {
            match pattern[0] {
                b'v' => return Self::VisualCmd,
                b'i' => return Self::InsertCmd,
                b'c' => return Self::CmdlineCmd,
                b't' => return Self::TerminalCmd,
                _ => {}
            }
        }

        // Check for option ('option')
        if pattern[0] == b'\'' && pattern.len() > 1 {
            return Self::Option;
        }

        // Check for Ex command (:command)
        if pattern[0] == b':' {
            return Self::ExCommand;
        }

        Self::General
    }

    /// Get the mode prefix for this topic type.
    #[must_use]
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Option => "'",
            Self::VisualCmd => "v_",
            Self::InsertCmd => "i_",
            Self::CmdlineCmd => "c_",
            Self::TerminalCmd => "t_",
            Self::ExCommand => ":",
            Self::General | Self::NormalCmd => "",
        }
    }
}

// =============================================================================
// Help File Detection
// =============================================================================

/// Check if a filename is a help file.
///
/// Help files have .txt extension and are in doc/ directories.
#[must_use]
pub fn is_help_filename(filename: &[u8]) -> bool {
    // Check for .txt extension
    if filename.len() < 5 {
        return false;
    }

    let ext_start = filename.len() - 4;
    let ext = &filename[ext_start..];

    ext.eq_ignore_ascii_case(b".txt")
}

/// Extract the help file base name without .txt extension.
#[must_use]
pub fn help_file_base(filename: &[u8]) -> &[u8] {
    if filename.len() >= 4 {
        let ext_start = filename.len() - 4;
        if filename[ext_start..].eq_ignore_ascii_case(b".txt") {
            return &filename[..ext_start];
        }
    }
    filename
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Check if a string is a valid help tag (FFI).
///
/// # Safety
///
/// `tag` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_valid_help_tag(tag: *const c_char, len: usize) -> c_int {
    if tag.is_null() {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(tag.cast::<u8>(), len);
    c_int::from(is_valid_help_tag(bytes))
}

/// Check if a help pattern matches a help tag (FFI).
///
/// # Safety
///
/// `tag` and `pattern` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_help_tag_matches(
    tag: *const c_char,
    tag_len: usize,
    pattern: *const c_char,
    pattern_len: usize,
) -> c_int {
    if tag.is_null() {
        return 0;
    }

    let tag_bytes = std::slice::from_raw_parts(tag.cast::<u8>(), tag_len);
    let pattern_bytes = if pattern.is_null() || pattern_len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(pattern.cast::<u8>(), pattern_len)
    };

    c_int::from(help_tag_matches(tag_bytes, pattern_bytes))
}

/// Detect help topic type from pattern (FFI).
///
/// Returns the topic type as an integer.
///
/// # Safety
///
/// `pattern` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_help_topic_type(pattern: *const c_char, len: usize) -> c_int {
    if pattern.is_null() || len == 0 {
        return 0; // General
    }

    let bytes = std::slice::from_raw_parts(pattern.cast::<u8>(), len);
    let topic_type = HelpTopicType::from_pattern(bytes);

    match topic_type {
        HelpTopicType::General => 0,
        HelpTopicType::Option => 1,
        HelpTopicType::NormalCmd => 2,
        HelpTopicType::VisualCmd => 3,
        HelpTopicType::InsertCmd => 4,
        HelpTopicType::CmdlineCmd => 5,
        HelpTopicType::TerminalCmd => 6,
        HelpTopicType::ExCommand => 7,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_help_tag() {
        assert!(is_valid_help_tag(b"help"));
        assert!(is_valid_help_tag(b"'option'"));
        assert!(is_valid_help_tag(b":command"));
        assert!(is_valid_help_tag(b"v_d"));
        assert!(is_valid_help_tag(b"usr_01.txt"));

        assert!(!is_valid_help_tag(b""));
        assert!(!is_valid_help_tag(b" space"));
        assert!(!is_valid_help_tag(b"tab\there"));
    }

    #[test]
    fn test_parse_help_tag_lang() {
        assert_eq!(parse_help_tag_lang(b"help"), (b"help".as_slice(), None));
        assert_eq!(
            parse_help_tag_lang(b"help@en"),
            (b"help".as_slice(), Some(b"en".as_slice()))
        );
        assert_eq!(
            parse_help_tag_lang(b"help@ja"),
            (b"help".as_slice(), Some(b"ja".as_slice()))
        );
        // Invalid language code
        assert_eq!(
            parse_help_tag_lang(b"help@EN"),
            (b"help@EN".as_slice(), None)
        );
        assert_eq!(parse_help_tag_lang(b"help@1"), (b"help@1".as_slice(), None));
    }

    #[test]
    fn test_help_tag_matches() {
        assert!(help_tag_matches(b"help", b""));
        assert!(help_tag_matches(b"help", b"h"));
        assert!(help_tag_matches(b"help", b"help"));
        assert!(help_tag_matches(b"HELP", b"help")); // case insensitive
        assert!(!help_tag_matches(b"help", b"helps"));
        assert!(!help_tag_matches(b"help", b"x"));
    }

    #[test]
    fn test_help_topic_type() {
        assert_eq!(HelpTopicType::from_pattern(b"help"), HelpTopicType::General);
        assert_eq!(
            HelpTopicType::from_pattern(b"'autoindent'"),
            HelpTopicType::Option
        );
        assert_eq!(
            HelpTopicType::from_pattern(b":edit"),
            HelpTopicType::ExCommand
        );
        assert_eq!(
            HelpTopicType::from_pattern(b"v_d"),
            HelpTopicType::VisualCmd
        );
        assert_eq!(
            HelpTopicType::from_pattern(b"i_CTRL-V"),
            HelpTopicType::InsertCmd
        );
        assert_eq!(
            HelpTopicType::from_pattern(b"c_CTRL-D"),
            HelpTopicType::CmdlineCmd
        );
        assert_eq!(
            HelpTopicType::from_pattern(b"t_CTRL-W"),
            HelpTopicType::TerminalCmd
        );
    }

    #[test]
    fn test_is_help_filename() {
        assert!(is_help_filename(b"help.txt"));
        assert!(is_help_filename(b"usr_01.txt"));
        assert!(is_help_filename(b"HELP.TXT"));
        assert!(!is_help_filename(b"help.vim"));
        assert!(!is_help_filename(b".txt"));
        assert!(!is_help_filename(b"txt"));
    }

    #[test]
    fn test_help_file_base() {
        assert_eq!(help_file_base(b"help.txt"), b"help");
        assert_eq!(help_file_base(b"usr_01.txt"), b"usr_01");
        assert_eq!(help_file_base(b"noext"), b"noext");
    }
}
