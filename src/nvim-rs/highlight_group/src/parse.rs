//! Attribute parsing for highlight groups.
//!
//! This module provides utilities for parsing highlight attribute strings
//! from `:highlight` command arguments (e.g., `gui=bold,italic guifg=Red`).

use std::ffi::c_int;

// Re-export highlight attribute flags for convenience
pub use nvim_highlight::hl_attr_flags::*;

/// Highlight attribute names and their corresponding flag values.
pub const HL_NAME_TABLE: &[(&str, i16)] = &[
    ("bold", HL_BOLD),
    ("standout", HL_STANDOUT),
    ("underline", HL_UNDERLINE),
    ("undercurl", HL_UNDERCURL),
    ("underdouble", HL_UNDERDOUBLE),
    ("underdotted", HL_UNDERDOTTED),
    ("underdashed", HL_UNDERDASHED),
    ("italic", HL_ITALIC),
    ("reverse", HL_INVERSE),
    ("inverse", HL_INVERSE), // Alias for reverse
    ("strikethrough", HL_STRIKETHROUGH),
    ("altfont", HL_ALTFONT),
    ("nocombine", HL_NOCOMBINE),
    ("NONE", 0), // Special: clears all attributes
];

/// Result of parsing attribute arguments (e.g., "bold,italic").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedAttrs {
    /// The combined attribute flags
    pub attrs: i16,
    /// Whether NONE was specified (clears all)
    pub is_none: bool,
    /// Error message if parsing failed
    pub error: Option<String>,
}

/// Parse an attribute string like "bold,italic,underline".
///
/// # Arguments
/// * `s` - The attribute string (comma-separated list)
///
/// # Returns
/// A `ParsedAttrs` with the combined flags or an error
pub fn parse_attr_string(s: &str) -> ParsedAttrs {
    let mut result = ParsedAttrs::default();
    let s = s.trim();

    if s.is_empty() {
        return result;
    }

    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let mut found = false;
        for &(name, flag) in HL_NAME_TABLE {
            if part.eq_ignore_ascii_case(name) {
                if flag == 0 {
                    // NONE clears everything
                    result.is_none = true;
                    result.attrs = 0;
                } else {
                    // Underline variants are mutually exclusive
                    if flag & HL_UNDERLINE_MASK != 0 {
                        result.attrs &= !HL_UNDERLINE_MASK;
                    }
                    result.attrs |= flag;
                }
                found = true;
                break;
            }
        }

        if !found {
            result.error = Some(format!("E418: Illegal value: {part}"));
            return result;
        }
    }

    result
}

/// Highlight key type for parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HlKey {
    Term,
    Cterm,
    CtermFg,
    CtermBg,
    Gui,
    GuiFg,
    GuiBg,
    GuiSp,
    Font,
    Blend,
    Start,
    Stop,
    Link,
    Default,
    Clear,
}

impl HlKey {
    /// Parse a key name to HlKey variant.
    pub fn parse(s: &str) -> Option<HlKey> {
        match s.to_ascii_uppercase().as_str() {
            "TERM" => Some(HlKey::Term),
            "CTERM" => Some(HlKey::Cterm),
            "CTERMFG" => Some(HlKey::CtermFg),
            "CTERMBG" => Some(HlKey::CtermBg),
            "GUI" => Some(HlKey::Gui),
            "GUIFG" => Some(HlKey::GuiFg),
            "GUIBG" => Some(HlKey::GuiBg),
            "GUISP" => Some(HlKey::GuiSp),
            "FONT" => Some(HlKey::Font),
            "BLEND" => Some(HlKey::Blend),
            "START" => Some(HlKey::Start),
            "STOP" => Some(HlKey::Stop),
            "LINK" => Some(HlKey::Link),
            "DEFAULT" => Some(HlKey::Default),
            "CLEAR" => Some(HlKey::Clear),
            "NONE" => None, // NONE is special-cased
            _ => None,
        }
    }

    /// Check if this key affects cterm settings.
    pub fn is_cterm(&self) -> bool {
        matches!(self, HlKey::Cterm | HlKey::CtermFg | HlKey::CtermBg)
    }

    /// Check if this key affects GUI settings.
    pub fn is_gui(&self) -> bool {
        matches!(
            self,
            HlKey::Gui | HlKey::GuiFg | HlKey::GuiBg | HlKey::GuiSp
        )
    }

    /// Check if this key is an attribute key (term/cterm/gui).
    pub fn is_attr(&self) -> bool {
        matches!(self, HlKey::Term | HlKey::Cterm | HlKey::Gui)
    }

    /// Check if this key is a color key.
    pub fn is_color(&self) -> bool {
        matches!(
            self,
            HlKey::CtermFg | HlKey::CtermBg | HlKey::GuiFg | HlKey::GuiBg | HlKey::GuiSp
        )
    }
}

/// A parsed key=value pair from a highlight command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HlKeyValue<'a> {
    pub key: HlKey,
    pub value: &'a str,
}

/// Parse a single key=value pair from a highlight command string.
///
/// # Arguments
/// * `s` - Input string starting at the key
///
/// # Returns
/// `Some((key_value, rest))` or `None` with error message
pub fn parse_key_value(s: &str) -> Result<(HlKeyValue<'_>, &str), String> {
    let s = s.trim_start();

    if s.is_empty() {
        return Err(String::new()); // End of input
    }

    // Check for leading '='
    if s.starts_with('=') {
        return Err(format!("E415: Unexpected equal sign: {s}"));
    }

    // Find the key (everything before '=' or whitespace)
    let key_end = s.find(|c: char| c == '=' || c.is_ascii_whitespace());
    let (key_str, rest) = match key_end {
        Some(idx) => (&s[..idx], &s[idx..]),
        None => (s, ""),
    };

    // Check for NONE (no value required)
    if key_str.eq_ignore_ascii_case("NONE") {
        return Err("NONE".to_string()); // Special marker
    }

    // Parse the key
    let key = HlKey::parse(key_str).ok_or_else(|| format!("E423: Illegal argument: {key_str}"))?;

    let rest = rest.trim_start();

    // Check for '='
    if !rest.starts_with('=') {
        return Err(format!("E416: Missing equal sign: {key_str}"));
    }

    let rest = rest[1..].trim_start();

    // Parse the value
    let (value, rest) = if let Some(content) = rest.strip_prefix('\'') {
        // Quoted value
        match content.find('\'') {
            Some(end) => (&content[..end], &content[end + 1..]),
            None => return Err(format!("E475: Invalid argument: {key_str}")),
        }
    } else {
        // Unquoted value (until whitespace)
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        (&rest[..end], &rest[end..])
    };

    if value.is_empty() {
        return Err(format!("E417: Missing argument: {key_str}"));
    }

    Ok((HlKeyValue { key, value }, rest))
}

/// Format attribute flags as a string.
///
/// # Arguments
/// * `attrs` - Attribute flags
///
/// # Returns
/// Comma-separated attribute names (e.g., "bold,italic")
pub fn format_attrs(attrs: i16) -> String {
    if attrs == 0 {
        return String::new();
    }

    let mut parts = Vec::new();
    let mut remaining = attrs;

    // Check underline variants first (mutually exclusive)
    let underline_bits = remaining & HL_UNDERLINE_MASK;
    if underline_bits != 0 {
        for &(name, flag) in HL_NAME_TABLE {
            if flag & HL_UNDERLINE_MASK != 0 && underline_bits == flag {
                parts.push(name);
                break;
            }
        }
        remaining &= !HL_UNDERLINE_MASK;
    }

    // Check other attributes
    for &(name, flag) in HL_NAME_TABLE {
        if flag == 0 || flag & HL_UNDERLINE_MASK != 0 {
            continue;
        }
        if remaining & flag != 0 {
            parts.push(name);
            remaining &= !flag;
        }
    }

    parts.join(",")
}

/// Parse a blend value (0-100).
///
/// # Arguments
/// * `s` - The blend value string
///
/// # Returns
/// The blend value (0-100) or -1 for NONE/invalid
pub fn parse_blend(s: &str) -> c_int {
    let s = s.trim();
    if s.eq_ignore_ascii_case("NONE") {
        return -1;
    }
    s.parse::<c_int>().unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attr_string() {
        // Simple attributes
        let r = parse_attr_string("bold");
        assert_eq!(r.attrs, HL_BOLD);
        assert!(r.error.is_none());

        let r = parse_attr_string("italic");
        assert_eq!(r.attrs, HL_ITALIC);

        // Multiple attributes
        let r = parse_attr_string("bold,italic");
        assert_eq!(r.attrs, HL_BOLD | HL_ITALIC);

        // Case insensitive
        let r = parse_attr_string("BOLD,ITALIC");
        assert_eq!(r.attrs, HL_BOLD | HL_ITALIC);

        // With spaces
        let r = parse_attr_string(" bold , italic ");
        assert_eq!(r.attrs, HL_BOLD | HL_ITALIC);

        // NONE clears all
        let r = parse_attr_string("NONE");
        assert_eq!(r.attrs, 0);
        assert!(r.is_none);

        // Invalid
        let r = parse_attr_string("invalid");
        assert!(r.error.is_some());
    }

    #[test]
    fn test_parse_attr_underline_variants() {
        // Underline variants are mutually exclusive
        let r = parse_attr_string("underline,undercurl");
        assert_eq!(r.attrs & HL_UNDERLINE_MASK, HL_UNDERCURL);

        let r = parse_attr_string("undercurl,underline");
        assert_eq!(r.attrs & HL_UNDERLINE_MASK, HL_UNDERLINE);
    }

    #[test]
    fn test_hl_key_parse() {
        assert_eq!(HlKey::parse("gui"), Some(HlKey::Gui));
        assert_eq!(HlKey::parse("GUI"), Some(HlKey::Gui));
        assert_eq!(HlKey::parse("guifg"), Some(HlKey::GuiFg));
        assert_eq!(HlKey::parse("CTERMFG"), Some(HlKey::CtermFg));
        assert_eq!(HlKey::parse("blend"), Some(HlKey::Blend));
        assert_eq!(HlKey::parse("invalid"), None);
    }

    #[test]
    fn test_hl_key_predicates() {
        assert!(HlKey::Cterm.is_cterm());
        assert!(HlKey::CtermFg.is_cterm());
        assert!(!HlKey::Gui.is_cterm());

        assert!(HlKey::Gui.is_gui());
        assert!(HlKey::GuiFg.is_gui());
        assert!(!HlKey::Cterm.is_gui());

        assert!(HlKey::Gui.is_attr());
        assert!(HlKey::Cterm.is_attr());
        assert!(!HlKey::GuiFg.is_attr());

        assert!(HlKey::GuiFg.is_color());
        assert!(HlKey::CtermBg.is_color());
        assert!(!HlKey::Gui.is_color());
    }

    #[test]
    fn test_parse_key_value() {
        // Simple case
        let r = parse_key_value("gui=bold rest");
        assert!(r.is_ok());
        let (kv, rest) = r.unwrap();
        assert_eq!(kv.key, HlKey::Gui);
        assert_eq!(kv.value, "bold");
        assert_eq!(rest, " rest");

        // Quoted value
        let r = parse_key_value("guifg='Light Blue'");
        assert!(r.is_ok());
        let (kv, _) = r.unwrap();
        assert_eq!(kv.key, HlKey::GuiFg);
        assert_eq!(kv.value, "Light Blue");

        // Missing equal sign
        let r = parse_key_value("gui bold");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("E416"));

        // Unexpected equal sign
        let r = parse_key_value("=value");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("E415"));
    }

    #[test]
    fn test_format_attrs() {
        assert_eq!(format_attrs(0), "");
        assert_eq!(format_attrs(HL_BOLD), "bold");
        assert_eq!(format_attrs(HL_UNDERLINE), "underline");
        assert!(format_attrs(HL_BOLD | HL_ITALIC).contains("bold"));
        assert!(format_attrs(HL_BOLD | HL_ITALIC).contains("italic"));
    }

    #[test]
    fn test_parse_blend() {
        assert_eq!(parse_blend("50"), 50);
        assert_eq!(parse_blend("0"), 0);
        assert_eq!(parse_blend("100"), 100);
        assert_eq!(parse_blend("NONE"), -1);
        assert_eq!(parse_blend("none"), -1);
        assert_eq!(parse_blend("invalid"), -1);
    }
}
