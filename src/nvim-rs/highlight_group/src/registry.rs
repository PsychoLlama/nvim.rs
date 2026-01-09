//! Highlight group registry utilities.
//!
//! This module provides utilities for working with the highlight group
//! registry. The core registry functions (syn_name2id, syn_check_group, etc.)
//! are implemented in `nvim-highlight`, but this module provides additional
//! helpers for common operations.
//!
//! Note: The highlight group table itself lives in C code and is accessed
//! via FFI. This module provides Rust-side utilities that don't require
//! direct table access.

use std::ffi::c_int;

use crate::types::{SgSet, MAX_SYN_NAME};
use crate::{is_valid_hl_name, is_valid_hl_name_char};

/// Built-in highlight group names that are guaranteed to exist.
pub const BUILTIN_GROUPS: &[&str] = &[
    "Normal",
    "NormalNC",
    "NormalFloat",
    "FloatBorder",
    "FloatTitle",
    "FloatFooter",
    "Cursor",
    "lCursor",
    "CursorIM",
    "CursorLine",
    "CursorLineNr",
    "CursorColumn",
    "ColorColumn",
    "Conceal",
    "DiffAdd",
    "DiffChange",
    "DiffDelete",
    "DiffText",
    "Directory",
    "EndOfBuffer",
    "ErrorMsg",
    "Folded",
    "FoldColumn",
    "IncSearch",
    "CurSearch",
    "LineNr",
    "LineNrAbove",
    "LineNrBelow",
    "MatchParen",
    "ModeMsg",
    "MoreMsg",
    "MsgArea",
    "MsgSeparator",
    "NonText",
    "OkMsg",
    "Pmenu",
    "PmenuSel",
    "PmenuSbar",
    "PmenuThumb",
    "PmenuMatch",
    "PmenuMatchSel",
    "Question",
    "QuickFixLine",
    "Search",
    "SignColumn",
    "SpecialKey",
    "SpellBad",
    "SpellCap",
    "SpellLocal",
    "SpellRare",
    "StatusLine",
    "StatusLineNC",
    "Substitute",
    "TabLine",
    "TabLineFill",
    "TabLineSel",
    "TermCursor",
    "TermCursorNC",
    "Title",
    "Underlined",
    "Visual",
    "VisualNOS",
    "WarningMsg",
    "Whitespace",
    "WildMenu",
    "WinBar",
    "WinBarNC",
    "WinSeparator",
];

/// Check if a name refers to the special "Normal" highlight group.
#[inline]
pub fn is_normal_group(name: &str) -> bool {
    name.eq_ignore_ascii_case("Normal")
}

/// Check if a name refers to a built-in highlight group.
pub fn is_builtin_group(name: &str) -> bool {
    BUILTIN_GROUPS
        .iter()
        .any(|&builtin| name.eq_ignore_ascii_case(builtin))
}

/// Parse a highlight group name from a string slice.
///
/// Returns the group name and the remaining unconsumed string.
///
/// # Arguments
/// * `s` - Input string, may start with whitespace
///
/// # Returns
/// `Some((name, rest))` if a valid name was found, `None` otherwise
pub fn parse_group_name(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }

    // Find the end of the group name
    let name_end = s
        .find(|c: char| !is_valid_hl_name_char(c))
        .unwrap_or(s.len());

    if name_end == 0 {
        return None;
    }

    let name = &s[..name_end];
    let rest = &s[name_end..];

    // Validate the name
    if is_valid_hl_name(name) {
        Some((name, rest))
    } else {
        None
    }
}

/// Parse a treesitter-style capture name (e.g., "@keyword.function").
///
/// These names start with '@' and can contain dots for hierarchy.
///
/// # Arguments
/// * `s` - Input string starting with '@'
///
/// # Returns
/// `Some((full_name, parent))` where parent is the hierarchical parent
/// (e.g., for "@keyword.function", parent would be "@keyword")
pub fn parse_ts_capture_name(s: &str) -> Option<(&str, Option<&str>)> {
    if !s.starts_with('@') {
        return None;
    }

    // Find end of the capture name
    let name_end = s[1..]
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '.')
        .map(|i| i + 1)
        .unwrap_or(s.len());

    let name = &s[..name_end];

    // Find the parent by looking for the last dot
    let parent = if let Some(last_dot) = name.rfind('.') {
        if last_dot > 1 {
            Some(&name[..last_dot])
        } else {
            None
        }
    } else {
        None
    };

    Some((name, parent))
}

/// Determine if a highlight group has settings based on its flags.
///
/// # Arguments
/// * `sg_set` - The sg_set flags from the highlight group
/// * `sg_cleared` - Whether the group has been cleared
/// * `check_link` - Whether to check for link setting as well
///
/// # Returns
/// `true` if the group has any settings that would prevent using a default link
#[inline]
pub fn has_settings(sg_set: SgSet, sg_cleared: bool, check_link: bool) -> bool {
    !sg_cleared && (sg_set.has_cterm() || sg_set.has_gui() || (check_link && sg_set.has_link()))
}

/// Validate that a highlight ID is within valid bounds.
///
/// # Arguments
/// * `id` - Highlight group ID (1-based)
/// * `max_id` - Maximum valid ID (from highlight_ga.ga_len)
///
/// # Returns
/// `true` if the ID is valid
#[inline]
pub fn is_valid_hl_id(id: c_int, max_id: c_int) -> bool {
    id >= 1 && id <= max_id
}

/// Calculate the parent group name for a scoped highlight name.
///
/// Scoped names like "@keyword.function.lua" have parent "@keyword.function".
///
/// # Arguments
/// * `name` - The full highlight group name
///
/// # Returns
/// `Some(parent)` if there's a parent, `None` otherwise
pub fn get_parent_name(name: &str) -> Option<&str> {
    if name.starts_with('@') && name.len() > 1 {
        name.rfind('.').map(|idx| &name[..idx])
    } else {
        None
    }
}

/// Check if a highlight name is a treesitter capture.
#[inline]
pub fn is_ts_capture(name: &str) -> bool {
    name.starts_with('@')
}

/// Normalize a highlight group name to uppercase for lookup.
///
/// # Arguments
/// * `name` - The group name
/// * `buffer` - Buffer to write the uppercase name (must be at least `MAX_SYN_NAME + 1`)
///
/// # Returns
/// The uppercase name as a slice of the buffer, or `None` if name is too long
pub fn normalize_name<'a>(name: &str, buffer: &'a mut [u8]) -> Option<&'a str> {
    if name.len() > MAX_SYN_NAME || buffer.len() < name.len() {
        return None;
    }

    for (i, c) in name.chars().enumerate() {
        buffer[i] = c.to_ascii_uppercase() as u8;
    }

    // SAFETY: We only wrote ASCII characters (uppercase conversion)
    Some(unsafe { std::str::from_utf8_unchecked(&buffer[..name.len()]) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_normal_group() {
        assert!(is_normal_group("Normal"));
        assert!(is_normal_group("normal"));
        assert!(is_normal_group("NORMAL"));
        assert!(!is_normal_group("NormalNC"));
        assert!(!is_normal_group(""));
    }

    #[test]
    fn test_is_builtin_group() {
        assert!(is_builtin_group("Normal"));
        assert!(is_builtin_group("Cursor"));
        assert!(is_builtin_group("statusline")); // Case insensitive
        assert!(!is_builtin_group("MyCustomGroup"));
        assert!(!is_builtin_group("@treesitter.keyword"));
    }

    #[test]
    fn test_parse_group_name() {
        assert_eq!(parse_group_name("Normal"), Some(("Normal", "")));
        assert_eq!(
            parse_group_name("  StatusLine "),
            Some(("StatusLine", " "))
        );
        assert_eq!(
            parse_group_name("@keyword.function rest"),
            Some(("@keyword.function", " rest"))
        );
        assert_eq!(parse_group_name("A"), Some(("A", "")));
        assert_eq!(parse_group_name("_Private123"), Some(("_Private123", "")));

        // Invalid
        assert_eq!(parse_group_name(""), None);
        assert_eq!(parse_group_name("   "), None);
        assert_eq!(parse_group_name("123abc"), None);
    }

    #[test]
    fn test_parse_ts_capture_name() {
        assert_eq!(
            parse_ts_capture_name("@keyword"),
            Some(("@keyword", None))
        );
        assert_eq!(
            parse_ts_capture_name("@keyword.function"),
            Some(("@keyword.function", Some("@keyword")))
        );
        assert_eq!(
            parse_ts_capture_name("@lsp.type.function"),
            Some(("@lsp.type.function", Some("@lsp.type")))
        );

        // Not a treesitter capture
        assert_eq!(parse_ts_capture_name("Normal"), None);
    }

    #[test]
    fn test_has_settings() {
        // Cleared group has no settings
        assert!(!has_settings(SgSet::NONE, true, true));

        // Non-cleared with cterm set
        let mut flags = SgSet::NONE;
        flags.set_cterm();
        assert!(has_settings(flags, false, false));

        // Non-cleared with gui set
        let mut flags = SgSet::NONE;
        flags.set_gui();
        assert!(has_settings(flags, false, false));

        // Non-cleared with link set, but check_link is false
        let mut flags = SgSet::NONE;
        flags.set_link();
        assert!(!has_settings(flags, false, false));
        assert!(has_settings(flags, false, true));
    }

    #[test]
    fn test_is_valid_hl_id() {
        assert!(is_valid_hl_id(1, 100));
        assert!(is_valid_hl_id(100, 100));
        assert!(!is_valid_hl_id(0, 100));
        assert!(!is_valid_hl_id(-1, 100));
        assert!(!is_valid_hl_id(101, 100));
    }

    #[test]
    fn test_get_parent_name() {
        assert_eq!(get_parent_name("@keyword.function"), Some("@keyword"));
        assert_eq!(get_parent_name("@a.b.c"), Some("@a.b"));
        assert_eq!(get_parent_name("@keyword"), None);
        assert_eq!(get_parent_name("Normal"), None);
        assert_eq!(get_parent_name(""), None);
    }

    #[test]
    fn test_is_ts_capture() {
        assert!(is_ts_capture("@keyword"));
        assert!(is_ts_capture("@lsp.type.function"));
        assert!(!is_ts_capture("Normal"));
        assert!(!is_ts_capture(""));
    }

    #[test]
    fn test_normalize_name() {
        let mut buf = [0u8; 256];

        assert_eq!(normalize_name("Normal", &mut buf), Some("NORMAL"));
        assert_eq!(normalize_name("statusLine", &mut buf), Some("STATUSLINE"));
        assert_eq!(normalize_name("ABC", &mut buf), Some("ABC"));

        // Too long
        let long = "a".repeat(MAX_SYN_NAME + 1);
        assert_eq!(normalize_name(&long, &mut buf), None);
    }
}
