//! Command-line completion for highlight groups.
//!
//! This module provides completion utilities for the `:highlight` command,
//! including:
//! - Highlight group name completion
//! - Attribute name completion (gui, cterm, etc.)
//! - Color name completion
//!
//! The actual completion integration with the command line is handled by C code,
//! but this module provides the completion data and matching logic.

use crate::registry::BUILTIN_GROUPS;
use crate::types::COLOR_NAMES;

/// Highlight command keywords that can be completed.
pub const HIGHLIGHT_KEYWORDS: &[&str] = &["default", "clear", "link", "NONE"];

/// Highlight key names that can appear in key=value pairs.
pub const HIGHLIGHT_KEYS: &[&str] = &[
    "term", "start", "stop", "cterm", "ctermfg", "ctermbg", "gui", "guifg", "guibg", "guisp",
    "font", "blend",
];

/// Attribute values that can be used with term/cterm/gui.
pub const ATTRIBUTE_VALUES: &[&str] = &[
    "bold",
    "standout",
    "underline",
    "undercurl",
    "underdouble",
    "underdotted",
    "underdashed",
    "italic",
    "reverse",
    "inverse",
    "strikethrough",
    "altfont",
    "nocombine",
    "NONE",
];

/// Completion context for highlight commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionContext {
    /// Completing the first word (group name or keyword)
    FirstWord,
    /// Completing after "link" - first group name
    LinkFrom,
    /// Completing after "link GroupName" - target group name
    LinkTo,
    /// Completing after "clear" - group name
    ClearGroup,
    /// Completing a key name (gui=, cterm=, etc.)
    KeyName,
    /// Completing an attribute value (bold, italic, etc.)
    AttrValue,
    /// Completing a color name (for guifg, guibg, etc.)
    ColorName,
    /// Completing a cterm color number
    CtermColor,
}

/// Iterator for highlight group names.
///
/// This provides group names for completion, optionally including
/// special keywords like "NONE", "default", "link", "clear".
pub struct GroupNameIterator<'a> {
    groups: &'a [&'a str],
    index: usize,
    include_keywords: bool,
    keywords_index: usize,
    prefix: &'a str,
}

impl<'a> GroupNameIterator<'a> {
    /// Create a new iterator over group names.
    ///
    /// # Arguments
    /// * `groups` - List of group names
    /// * `include_keywords` - Whether to include NONE, default, link, clear
    /// * `prefix` - Only return names starting with this prefix
    pub fn new(groups: &'a [&'a str], include_keywords: bool, prefix: &'a str) -> Self {
        GroupNameIterator {
            groups,
            index: 0,
            include_keywords,
            keywords_index: 0,
            prefix,
        }
    }
}

impl<'a> Iterator for GroupNameIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // First iterate through group names
        while self.index < self.groups.len() {
            let name = self.groups[self.index];
            self.index += 1;
            if name.to_lowercase().starts_with(&self.prefix.to_lowercase()) {
                return Some(name);
            }
        }

        // Then iterate through keywords if enabled
        if self.include_keywords {
            while self.keywords_index < HIGHLIGHT_KEYWORDS.len() {
                let keyword = HIGHLIGHT_KEYWORDS[self.keywords_index];
                self.keywords_index += 1;
                if keyword
                    .to_lowercase()
                    .starts_with(&self.prefix.to_lowercase())
                {
                    return Some(keyword);
                }
            }
        }

        None
    }
}

/// Check if a string matches a prefix (case-insensitive).
#[inline]
pub fn matches_prefix(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// Get attribute names that match a prefix.
pub fn complete_attr_names(prefix: &str) -> Vec<&'static str> {
    ATTRIBUTE_VALUES
        .iter()
        .copied()
        .filter(|name| matches_prefix(name, prefix))
        .collect()
}

/// Get key names that match a prefix.
pub fn complete_key_names(prefix: &str) -> Vec<&'static str> {
    HIGHLIGHT_KEYS
        .iter()
        .copied()
        .filter(|name| matches_prefix(name, prefix))
        .collect()
}

/// Get basic color names that match a prefix.
pub fn complete_color_names(prefix: &str) -> Vec<&'static str> {
    COLOR_NAMES
        .iter()
        .copied()
        .filter(|name| *name != "NONE" && matches_prefix(name, prefix))
        .collect()
}

/// Get builtin group names that match a prefix.
pub fn complete_builtin_groups(prefix: &str) -> Vec<&'static str> {
    BUILTIN_GROUPS
        .iter()
        .copied()
        .filter(|name| matches_prefix(name, prefix))
        .collect()
}

/// Determine the completion context from command line position.
///
/// # Arguments
/// * `line` - The command line after `:highlight`
/// * `cursor_pos` - Cursor position in the line
///
/// # Returns
/// The appropriate completion context
pub fn determine_context(line: &str, cursor_pos: usize) -> CompletionContext {
    let before_cursor = if cursor_pos <= line.len() {
        &line[..cursor_pos]
    } else {
        line
    };

    let words: Vec<&str> = before_cursor.split_whitespace().collect();
    let ends_with_space =
        before_cursor.ends_with(' ') || before_cursor.ends_with('\t') || before_cursor.is_empty();

    // Adjust word count based on whether we're completing a new word
    let effective_words = if ends_with_space {
        words.len() + 1
    } else {
        words.len()
    };

    // Check for "link" command
    if words.iter().any(|w| w.eq_ignore_ascii_case("link")) {
        let link_pos = words
            .iter()
            .position(|w| w.eq_ignore_ascii_case("link"))
            .unwrap();
        let words_after_link = effective_words - link_pos - 1;
        return match words_after_link {
            1 => CompletionContext::LinkFrom,
            2 => CompletionContext::LinkTo,
            _ => CompletionContext::LinkTo,
        };
    }

    // Check for "clear" command
    if words.iter().any(|w| w.eq_ignore_ascii_case("clear")) {
        return CompletionContext::ClearGroup;
    }

    // Check if we're completing a key=value
    if let Some(last_word) = words.last() {
        if !ends_with_space {
            // Check if we're completing after '='
            if let Some(eq_pos) = last_word.find('=') {
                let key = &last_word[..eq_pos];
                if key.eq_ignore_ascii_case("cterm")
                    || key.eq_ignore_ascii_case("term")
                    || key.eq_ignore_ascii_case("gui")
                {
                    return CompletionContext::AttrValue;
                }
                if key.eq_ignore_ascii_case("guifg")
                    || key.eq_ignore_ascii_case("guibg")
                    || key.eq_ignore_ascii_case("guisp")
                {
                    return CompletionContext::ColorName;
                }
                if key.eq_ignore_ascii_case("ctermfg") || key.eq_ignore_ascii_case("ctermbg") {
                    return CompletionContext::CtermColor;
                }
            }
        }
    }

    // After group name, complete keys
    if effective_words > 1 {
        // Skip "default" if present
        let start = if words
            .first()
            .is_some_and(|w| w.eq_ignore_ascii_case("default"))
        {
            2
        } else {
            1
        };
        if effective_words > start {
            return CompletionContext::KeyName;
        }
    }

    CompletionContext::FirstWord
}

/// Parse the value part after '=' for multi-value completion.
///
/// For example, "bold,it" should complete "italic" after "bold,".
pub fn parse_partial_attr_value(value: &str) -> (&str, &str) {
    match value.rfind(',') {
        Some(pos) => (&value[..=pos], &value[pos + 1..]),
        None => ("", value),
    }
}

/// Generate cterm color numbers for completion.
pub fn complete_cterm_colors(prefix: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Add basic color names that work for cterm
    for name in COLOR_NAMES.iter() {
        if *name != "NONE" && matches_prefix(name, prefix) {
            results.push((*name).to_string());
        }
    }

    // Add numeric colors if prefix is numeric
    if prefix.is_empty() || prefix.chars().all(|c| c.is_ascii_digit()) {
        let start: u16 = prefix.parse().unwrap_or(0);
        for n in start..256.min(start + 20) {
            let s = n.to_string();
            if matches_prefix(&s, prefix) {
                results.push(s);
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_prefix() {
        assert!(matches_prefix("Normal", "nor"));
        assert!(matches_prefix("Normal", "Normal"));
        assert!(matches_prefix("NORMAL", "normal"));
        assert!(!matches_prefix("Normal", "xyz"));
        assert!(!matches_prefix("", "a"));
        assert!(matches_prefix("anything", ""));
    }

    #[test]
    fn test_complete_attr_names() {
        let results = complete_attr_names("bo");
        assert!(results.contains(&"bold"));
        assert!(!results.contains(&"italic"));

        let results = complete_attr_names("under");
        assert!(results.contains(&"underline"));
        assert!(results.contains(&"undercurl"));
        assert!(results.contains(&"underdouble"));
    }

    #[test]
    fn test_complete_key_names() {
        let results = complete_key_names("gui");
        assert!(results.contains(&"gui"));
        assert!(results.contains(&"guifg"));
        assert!(results.contains(&"guibg"));
        assert!(results.contains(&"guisp"));

        let results = complete_key_names("cterm");
        assert!(results.contains(&"cterm"));
        assert!(results.contains(&"ctermfg"));
        assert!(results.contains(&"ctermbg"));
    }

    #[test]
    fn test_complete_color_names() {
        let results = complete_color_names("Bl");
        assert!(results.contains(&"Black"));
        assert!(results.contains(&"Blue"));
        assert!(!results.contains(&"Red"));
    }

    #[test]
    fn test_complete_builtin_groups() {
        let results = complete_builtin_groups("Sta");
        assert!(results.contains(&"StatusLine"));
        assert!(results.contains(&"StatusLineNC"));
    }

    #[test]
    fn test_determine_context() {
        // First word
        assert_eq!(determine_context("", 0), CompletionContext::FirstWord);
        assert_eq!(determine_context("Nor", 3), CompletionContext::FirstWord);

        // After link
        assert_eq!(determine_context("link ", 5), CompletionContext::LinkFrom);
        assert_eq!(
            determine_context("link MyGroup ", 13),
            CompletionContext::LinkTo
        );

        // After clear
        assert_eq!(
            determine_context("clear ", 6),
            CompletionContext::ClearGroup
        );

        // Key completion
        assert_eq!(determine_context("Normal ", 7), CompletionContext::KeyName);

        // Attribute value
        assert_eq!(
            determine_context("Normal gui=bo", 13),
            CompletionContext::AttrValue
        );

        // Color name
        assert_eq!(
            determine_context("Normal guifg=", 13),
            CompletionContext::ColorName
        );
    }

    #[test]
    fn test_parse_partial_attr_value() {
        assert_eq!(parse_partial_attr_value("bold"), ("", "bold"));
        assert_eq!(parse_partial_attr_value("bold,"), ("bold,", ""));
        assert_eq!(parse_partial_attr_value("bold,ital"), ("bold,", "ital"));
        assert_eq!(
            parse_partial_attr_value("bold,italic,under"),
            ("bold,italic,", "under")
        );
    }

    #[test]
    fn test_group_name_iterator() {
        let groups = &["Normal", "StatusLine", "Cursor"];
        let iter = GroupNameIterator::new(groups, false, "");
        let names: Vec<_> = iter.collect();
        assert_eq!(names, vec!["Normal", "StatusLine", "Cursor"]);

        let iter = GroupNameIterator::new(groups, true, "");
        let names: Vec<_> = iter.collect();
        assert!(names.contains(&"Normal"));
        assert!(names.contains(&"NONE"));
        assert!(names.contains(&"default"));

        let iter = GroupNameIterator::new(groups, false, "sta");
        let names: Vec<_> = iter.collect();
        assert_eq!(names, vec!["StatusLine"]);
    }

    #[test]
    fn test_complete_cterm_colors() {
        let results = complete_cterm_colors("Bl");
        assert!(results.iter().any(|s| s == "Black"));
        assert!(results.iter().any(|s| s == "Blue"));

        let results = complete_cterm_colors("1");
        assert!(results.iter().any(|s| s == "1"));
        assert!(results.iter().any(|s| s == "10"));
    }
}
