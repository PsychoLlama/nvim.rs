//! Highlight command support utilities.
//!
//! This module provides utilities for implementing the `:highlight` command,
//! including argument parsing, option validation, and command execution helpers.
//!
//! The actual command execution still happens in C code (do_highlight),
//! but this module provides Rust-side helpers for parsing and validation.

use std::ffi::c_int;

use crate::parse::{parse_attr_string, parse_key_value, HlKey, HlKeyValue, ParsedAttrs};
use crate::registry::is_normal_group;

/// Result of parsing a `:highlight` command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightCommand<'a> {
    /// List all highlight groups (no arguments)
    ListAll,
    /// List a single highlight group
    ListOne(&'a str),
    /// Clear a highlight group
    Clear { name: &'a str, is_default: bool },
    /// Link one highlight group to another
    Link {
        from: &'a str,
        to: &'a str,
        is_default: bool,
    },
    /// Set highlight group attributes
    Set {
        name: &'a str,
        is_default: bool,
        settings: Vec<HlSetting<'a>>,
    },
}

/// A single highlight setting from the command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HlSetting<'a> {
    /// Attribute setting (term/cterm/gui)
    Attr { key: HlKey, attrs: ParsedAttrs },
    /// Color setting (ctermfg/ctermbg/guifg/guibg/guisp)
    Color { key: HlKey, value: &'a str },
    /// Font setting
    Font(&'a str),
    /// Blend value (0-100)
    Blend(c_int),
    /// Start terminal escape code
    Start(&'a str),
    /// Stop terminal escape code
    Stop(&'a str),
}

/// Error from parsing a highlight command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Missing group name
    MissingGroupName,
    /// Invalid group name
    InvalidGroupName(String),
    /// Missing argument after key
    MissingArgument(String),
    /// Invalid key name
    InvalidKey(String),
    /// Invalid attribute value
    InvalidAttribute(String),
    /// Too many arguments
    TooManyArguments(String),
    /// Not enough arguments
    NotEnoughArguments(String),
    /// General syntax error
    SyntaxError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::MissingGroupName => write!(f, "E411: Highlight group name missing"),
            ParseError::InvalidGroupName(s) => {
                write!(f, "E411: Highlight group name not found: {s}")
            }
            ParseError::MissingArgument(s) => write!(f, "E417: Missing argument: {s}"),
            ParseError::InvalidKey(s) => write!(f, "E423: Illegal argument: {s}"),
            ParseError::InvalidAttribute(s) => write!(f, "E418: Illegal value: {s}"),
            ParseError::TooManyArguments(s) => write!(f, "E413: Too many arguments: {s}"),
            ParseError::NotEnoughArguments(s) => write!(f, "E412: Not enough arguments: {s}"),
            ParseError::SyntaxError(s) => write!(f, "E475: Invalid argument: {s}"),
        }
    }
}

/// Check if a string ends an Ex command (newline, bar, or NUL).
#[inline]
pub fn ends_excmd(c: char) -> bool {
    c == '\0' || c == '\n' || c == '|' || c == '"'
}

/// Skip whitespace and return the rest of the string.
#[inline]
pub fn skip_whitespace(s: &str) -> &str {
    s.trim_start()
}

/// Skip to the next whitespace and return (word, rest).
pub fn skip_to_whitespace(s: &str) -> (&str, &str) {
    match s.find(char::is_whitespace) {
        Some(idx) => (&s[..idx], &s[idx..]),
        None => (s, ""),
    }
}

/// Parse the command portion of a highlight command line.
///
/// This handles the initial parsing of:
/// - `default` modifier
/// - `clear` command
/// - `link` command
/// - group name
///
/// # Arguments
/// * `line` - The command line after `:highlight`
///
/// # Returns
/// A parsed command variant or an error
pub fn parse_command(line: &str) -> Result<HighlightCommand<'_>, ParseError> {
    let line = skip_whitespace(line);

    // Empty line = list all
    if line.is_empty() || ends_excmd(line.chars().next().unwrap_or('\0')) {
        return Ok(HighlightCommand::ListAll);
    }

    let (first_word, rest) = skip_to_whitespace(line);
    let rest = skip_whitespace(rest);

    // Check for "default" modifier
    let (is_default, name_part, args_part) = if first_word.eq_ignore_ascii_case("default") {
        let (name, rest2) = skip_to_whitespace(rest);
        (true, name, skip_whitespace(rest2))
    } else {
        (false, first_word, rest)
    };

    // Empty name means list all
    if name_part.is_empty() {
        return Ok(HighlightCommand::ListAll);
    }

    // Check for "clear" command
    if name_part.eq_ignore_ascii_case("clear") {
        if args_part.is_empty() || ends_excmd(args_part.chars().next().unwrap_or('\0')) {
            // "highlight clear" without group name clears all
            return Ok(HighlightCommand::Clear {
                name: "",
                is_default,
            });
        }
        let (group_name, trailing) = skip_to_whitespace(args_part);
        let trailing = skip_whitespace(trailing);
        if !trailing.is_empty() && !ends_excmd(trailing.chars().next().unwrap_or('\0')) {
            return Err(ParseError::TooManyArguments(format!(
                "highlight clear {group_name}"
            )));
        }
        return Ok(HighlightCommand::Clear {
            name: group_name,
            is_default,
        });
    }

    // Check for "link" command
    if name_part.eq_ignore_ascii_case("link") {
        let (from_name, rest2) = skip_to_whitespace(args_part);
        let rest2 = skip_whitespace(rest2);
        let (to_name, trailing) = skip_to_whitespace(rest2);
        let trailing = skip_whitespace(trailing);

        if from_name.is_empty() || to_name.is_empty() {
            return Err(ParseError::NotEnoughArguments(format!(
                "highlight link {args_part}"
            )));
        }
        if !trailing.is_empty() && !ends_excmd(trailing.chars().next().unwrap_or('\0')) {
            return Err(ParseError::TooManyArguments(format!(
                "highlight link {args_part}"
            )));
        }
        return Ok(HighlightCommand::Link {
            from: from_name,
            to: to_name,
            is_default,
        });
    }

    // If no arguments after group name, it's a list request
    if args_part.is_empty() || ends_excmd(args_part.chars().next().unwrap_or('\0')) {
        return Ok(HighlightCommand::ListOne(name_part));
    }

    // Parse settings
    let mut settings = Vec::new();
    let mut remaining = args_part;

    while !remaining.is_empty() {
        let remaining_trimmed = skip_whitespace(remaining);
        if remaining_trimmed.is_empty()
            || ends_excmd(remaining_trimmed.chars().next().unwrap_or('\0'))
        {
            break;
        }

        match parse_key_value(remaining_trimmed) {
            Ok((kv, rest)) => {
                let setting = convert_key_value_to_setting(&kv)?;
                settings.push(setting);
                remaining = rest;
            }
            Err(e) if e == "NONE" => {
                // NONE clears all attributes
                settings.push(HlSetting::Attr {
                    key: HlKey::Gui,
                    attrs: ParsedAttrs {
                        attrs: 0,
                        is_none: true,
                        error: None,
                    },
                });
                // Skip past NONE
                let (_, rest) = skip_to_whitespace(remaining_trimmed);
                remaining = rest;
            }
            Err(e) if e.is_empty() => break,
            Err(e) => return Err(ParseError::SyntaxError(e)),
        }
    }

    Ok(HighlightCommand::Set {
        name: name_part,
        is_default,
        settings,
    })
}

/// Convert a parsed key-value pair to a highlight setting.
fn convert_key_value_to_setting<'a>(kv: &HlKeyValue<'a>) -> Result<HlSetting<'a>, ParseError> {
    match kv.key {
        HlKey::Term | HlKey::Cterm | HlKey::Gui => {
            let parsed = parse_attr_string(kv.value);
            if let Some(ref err) = parsed.error {
                return Err(ParseError::InvalidAttribute(err.clone()));
            }
            Ok(HlSetting::Attr {
                key: kv.key,
                attrs: parsed,
            })
        }
        HlKey::CtermFg | HlKey::CtermBg | HlKey::GuiFg | HlKey::GuiBg | HlKey::GuiSp => {
            Ok(HlSetting::Color {
                key: kv.key,
                value: kv.value,
            })
        }
        HlKey::Font => Ok(HlSetting::Font(kv.value)),
        HlKey::Blend => {
            let blend = crate::parse::parse_blend(kv.value);
            Ok(HlSetting::Blend(blend))
        }
        HlKey::Start => Ok(HlSetting::Start(kv.value)),
        HlKey::Stop => Ok(HlSetting::Stop(kv.value)),
        HlKey::Link | HlKey::Default | HlKey::Clear => {
            // These are handled at the command level
            Err(ParseError::InvalidKey(format!("{:?}", kv.key)))
        }
    }
}

/// Options for highlighting operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct HighlightOptions {
    /// Whether this is a default setting (can be overridden)
    pub is_default: bool,
    /// Whether to force the operation (bang modifier)
    pub force: bool,
    /// Whether this is during initialization
    pub is_init: bool,
}

/// Check if a highlight group setting would affect the Normal group.
///
/// Changes to Normal require special handling (updating default colors,
/// refreshing dependent groups).
#[inline]
pub fn affects_normal(name: &str) -> bool {
    is_normal_group(name)
}

/// Check if a command would modify highlight attributes (vs. just listing).
pub fn is_modifying_command(cmd: &HighlightCommand<'_>) -> bool {
    matches!(
        cmd,
        HighlightCommand::Clear { .. }
            | HighlightCommand::Link { .. }
            | HighlightCommand::Set { .. }
    )
}

/// Format a highlight command back to a string.
///
/// This is useful for debugging or generating highlight commands.
pub fn format_command(cmd: &HighlightCommand<'_>) -> String {
    match cmd {
        HighlightCommand::ListAll => String::new(),
        HighlightCommand::ListOne(name) => name.to_string(),
        HighlightCommand::Clear { name, is_default } => {
            if *is_default {
                format!("default clear {name}")
            } else if name.is_empty() {
                "clear".to_string()
            } else {
                format!("clear {name}")
            }
        }
        HighlightCommand::Link {
            from,
            to,
            is_default,
        } => {
            if *is_default {
                format!("default link {from} {to}")
            } else {
                format!("link {from} {to}")
            }
        }
        HighlightCommand::Set {
            name,
            is_default,
            settings,
        } => {
            let mut parts = Vec::new();
            if *is_default {
                parts.push("default".to_string());
            }
            parts.push(name.to_string());
            for setting in settings {
                parts.push(format_setting(setting));
            }
            parts.join(" ")
        }
    }
}

/// Format a single highlight setting.
fn format_setting(setting: &HlSetting<'_>) -> String {
    match setting {
        HlSetting::Attr { key, attrs } => {
            let key_name = match key {
                HlKey::Term => "term",
                HlKey::Cterm => "cterm",
                HlKey::Gui => "gui",
                _ => "gui",
            };
            if attrs.is_none {
                format!("{key_name}=NONE")
            } else {
                let attr_str = crate::parse::format_attrs(attrs.attrs);
                format!("{key_name}={attr_str}")
            }
        }
        HlSetting::Color { key, value } => {
            let key_name = match key {
                HlKey::CtermFg => "ctermfg",
                HlKey::CtermBg => "ctermbg",
                HlKey::GuiFg => "guifg",
                HlKey::GuiBg => "guibg",
                HlKey::GuiSp => "guisp",
                _ => "guifg",
            };
            format!("{key_name}={value}")
        }
        HlSetting::Font(f) => format!("font={f}"),
        HlSetting::Blend(b) => format!("blend={b}"),
        HlSetting::Start(s) => format!("start={s}"),
        HlSetting::Stop(s) => format!("stop={s}"),
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

use std::ffi::{c_char, CStr};

/// Check if a character ends an Ex command.
///
/// # Safety
/// This function is safe as it takes a simple integer.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_cmd_ends_excmd(c: c_int) -> c_int {
    let ch = char::from_u32(c as u32).unwrap_or('\0');
    c_int::from(ends_excmd(ch))
}

/// Check if a highlight command line would list all groups.
///
/// # Safety
/// `line` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_is_list_all(line: *const c_char) -> c_int {
    if line.is_null() {
        return 1; // Empty line = list all
    }
    let line_str = match unsafe { CStr::from_ptr(line) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match parse_command(line_str) {
        Ok(HighlightCommand::ListAll) => 1,
        _ => 0,
    }
}

/// Check if a highlight command line would list a single group.
///
/// # Safety
/// `line` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_is_list_one(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let line_str = match unsafe { CStr::from_ptr(line) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match parse_command(line_str) {
        Ok(HighlightCommand::ListOne(_)) => 1,
        _ => 0,
    }
}

/// Check if a highlight command line is a clear command.
///
/// # Safety
/// `line` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_is_clear(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let line_str = match unsafe { CStr::from_ptr(line) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match parse_command(line_str) {
        Ok(HighlightCommand::Clear { .. }) => 1,
        _ => 0,
    }
}

/// Check if a highlight command line is a link command.
///
/// # Safety
/// `line` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_is_link(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let line_str = match unsafe { CStr::from_ptr(line) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match parse_command(line_str) {
        Ok(HighlightCommand::Link { .. }) => 1,
        _ => 0,
    }
}

/// Check if a highlight command line is a set command.
///
/// # Safety
/// `line` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_is_set(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let line_str = match unsafe { CStr::from_ptr(line) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match parse_command(line_str) {
        Ok(HighlightCommand::Set { .. }) => 1,
        _ => 0,
    }
}

/// Check if a highlight command line has the default modifier.
///
/// # Safety
/// `line` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_has_default(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let line_str = match unsafe { CStr::from_ptr(line) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match parse_command(line_str) {
        Ok(HighlightCommand::Clear { is_default, .. })
        | Ok(HighlightCommand::Link { is_default, .. })
        | Ok(HighlightCommand::Set { is_default, .. }) => c_int::from(is_default),
        _ => 0,
    }
}

/// Check if a highlight command would modify (vs. just list).
///
/// # Safety
/// `line` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_is_modifying(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let line_str = match unsafe { CStr::from_ptr(line) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match parse_command(line_str) {
        Ok(ref cmd) => c_int::from(is_modifying_command(cmd)),
        _ => 0,
    }
}

/// Check if a highlight command would affect the Normal group.
///
/// # Safety
/// `name` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_cmd_affects_normal(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    c_int::from(affects_normal(name_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list_all() {
        assert_eq!(parse_command("").unwrap(), HighlightCommand::ListAll);
        assert_eq!(parse_command("  ").unwrap(), HighlightCommand::ListAll);
    }

    #[test]
    fn test_parse_list_one() {
        assert_eq!(
            parse_command("Normal").unwrap(),
            HighlightCommand::ListOne("Normal")
        );
        assert_eq!(
            parse_command("  StatusLine  ").unwrap(),
            HighlightCommand::ListOne("StatusLine")
        );
    }

    #[test]
    fn test_parse_clear() {
        assert_eq!(
            parse_command("clear").unwrap(),
            HighlightCommand::Clear {
                name: "",
                is_default: false
            }
        );
        assert_eq!(
            parse_command("clear Normal").unwrap(),
            HighlightCommand::Clear {
                name: "Normal",
                is_default: false
            }
        );
        assert_eq!(
            parse_command("default clear Normal").unwrap(),
            HighlightCommand::Clear {
                name: "Normal",
                is_default: true
            }
        );
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_command("link MyGroup Normal").unwrap(),
            HighlightCommand::Link {
                from: "MyGroup",
                to: "Normal",
                is_default: false
            }
        );
        assert_eq!(
            parse_command("default link MyGroup Normal").unwrap(),
            HighlightCommand::Link {
                from: "MyGroup",
                to: "Normal",
                is_default: true
            }
        );
    }

    #[test]
    fn test_parse_link_errors() {
        assert!(matches!(
            parse_command("link").unwrap_err(),
            ParseError::NotEnoughArguments(_)
        ));
        assert!(matches!(
            parse_command("link FromGroup").unwrap_err(),
            ParseError::NotEnoughArguments(_)
        ));
    }

    #[test]
    fn test_parse_set() {
        let cmd = parse_command("Normal gui=bold guifg=#FF0000").unwrap();
        match cmd {
            HighlightCommand::Set {
                name,
                is_default,
                settings,
            } => {
                assert_eq!(name, "Normal");
                assert!(!is_default);
                assert_eq!(settings.len(), 2);
            }
            _ => panic!("Expected Set command"),
        }
    }

    #[test]
    fn test_parse_default_set() {
        let cmd = parse_command("default Comment gui=italic guifg=Gray").unwrap();
        match cmd {
            HighlightCommand::Set {
                name,
                is_default,
                settings,
            } => {
                assert_eq!(name, "Comment");
                assert!(is_default);
                assert_eq!(settings.len(), 2);
            }
            _ => panic!("Expected Set command"),
        }
    }

    #[test]
    fn test_ends_excmd() {
        assert!(ends_excmd('\0'));
        assert!(ends_excmd('\n'));
        assert!(ends_excmd('|'));
        assert!(ends_excmd('"'));
        assert!(!ends_excmd('a'));
        assert!(!ends_excmd(' '));
    }

    #[test]
    fn test_is_modifying_command() {
        assert!(!is_modifying_command(&HighlightCommand::ListAll));
        assert!(!is_modifying_command(&HighlightCommand::ListOne("Normal")));
        assert!(is_modifying_command(&HighlightCommand::Clear {
            name: "",
            is_default: false
        }));
        assert!(is_modifying_command(&HighlightCommand::Link {
            from: "A",
            to: "B",
            is_default: false
        }));
    }

    #[test]
    fn test_format_command() {
        assert_eq!(format_command(&HighlightCommand::ListAll), "");
        assert_eq!(
            format_command(&HighlightCommand::ListOne("Normal")),
            "Normal"
        );
        assert_eq!(
            format_command(&HighlightCommand::Clear {
                name: "Normal",
                is_default: false
            }),
            "clear Normal"
        );
        assert_eq!(
            format_command(&HighlightCommand::Link {
                from: "A",
                to: "B",
                is_default: true
            }),
            "default link A B"
        );
    }

    #[test]
    fn test_affects_normal() {
        assert!(affects_normal("Normal"));
        assert!(affects_normal("normal"));
        assert!(affects_normal("NORMAL"));
        assert!(!affects_normal("NormalNC"));
        assert!(!affects_normal("StatusLine"));
    }
}
