//! User-defined command completion source
//!
//! This module provides helpers for completing user-defined command names
//! and their arguments.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

// =============================================================================
// User Command Name Validation
// =============================================================================

/// Check if a name is valid for a user-defined command.
///
/// User command names must:
/// - Start with an uppercase letter
/// - Contain only letters and digits
/// - Not match a built-in command exactly (checked separately)
#[must_use]
pub fn is_valid_usercmd_name(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must start with uppercase letter
    if !name[0].is_ascii_uppercase() {
        return false;
    }

    // Rest must be alphanumeric
    name[1..].iter().all(|&c| c.is_ascii_alphanumeric())
}

/// Check if a string could be the start of a user command name.
///
/// For completion purposes, allows partial names that could become valid.
#[must_use]
pub fn is_valid_usercmd_prefix(prefix: &[u8]) -> bool {
    if prefix.is_empty() {
        return true; // Empty matches all
    }

    // First character must be uppercase or matches start of one
    if !prefix[0].is_ascii_uppercase() {
        return false;
    }

    // Rest must be alphanumeric
    prefix[1..].iter().all(|&c| c.is_ascii_alphanumeric())
}

/// Check if a user command name matches a prefix.
#[must_use]
pub fn usercmd_matches_prefix(name: &[u8], prefix: &[u8]) -> bool {
    if prefix.is_empty() {
        return true;
    }

    if name.len() < prefix.len() {
        return false;
    }

    &name[..prefix.len()] == prefix
}

// =============================================================================
// User Command Argument Types
// =============================================================================

/// Argument completion types for user commands.
///
/// These correspond to the -complete= option values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum UserCmdComplete {
    /// No completion
    #[default]
    None = 0,
    /// File arguments
    File = 1,
    /// Directory arguments
    Dir = 2,
    /// Buffer name arguments
    Buffer = 3,
    /// Command name arguments
    Command = 4,
    /// Event name arguments
    Event = 5,
    /// Expression arguments
    Expression = 6,
    /// Function name arguments
    Function = 7,
    /// Help tag arguments
    Help = 8,
    /// Highlight group arguments
    Highlight = 9,
    /// Mapping name arguments
    Mapping = 10,
    /// Menu name arguments
    Menu = 11,
    /// Option name arguments
    Option = 12,
    /// Shell command arguments
    Shellcmd = 13,
    /// Sign name arguments
    Sign = 14,
    /// Syntax group arguments
    Syntax = 15,
    /// Tag arguments
    Tag = 16,
    /// User name arguments
    User = 17,
    /// Variable name arguments
    Var = 18,
    /// Color scheme arguments
    Color = 19,
    /// Compiler arguments
    Compiler = 20,
    /// Filetype arguments
    Filetype = 21,
    /// Locale arguments
    Locale = 22,
    /// User-defined custom completion
    Custom = 23,
    /// User-defined custom list completion
    CustomList = 24,
    /// Lua function completion
    Lua = 25,
    /// Argument list arguments
    Arglist = 26,
    /// Environment variable arguments
    Environment = 27,
    /// Keymap arguments
    Keymap = 28,
    /// Messages arguments
    Messages = 29,
    /// Runtime paths
    Runtime = 30,
    /// Checkhealth arguments
    Checkhealth = 31,
}

impl UserCmdComplete {
    /// Convert from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::File),
            2 => Some(Self::Dir),
            3 => Some(Self::Buffer),
            4 => Some(Self::Command),
            5 => Some(Self::Event),
            6 => Some(Self::Expression),
            7 => Some(Self::Function),
            8 => Some(Self::Help),
            9 => Some(Self::Highlight),
            10 => Some(Self::Mapping),
            11 => Some(Self::Menu),
            12 => Some(Self::Option),
            13 => Some(Self::Shellcmd),
            14 => Some(Self::Sign),
            15 => Some(Self::Syntax),
            16 => Some(Self::Tag),
            17 => Some(Self::User),
            18 => Some(Self::Var),
            19 => Some(Self::Color),
            20 => Some(Self::Compiler),
            21 => Some(Self::Filetype),
            22 => Some(Self::Locale),
            23 => Some(Self::Custom),
            24 => Some(Self::CustomList),
            25 => Some(Self::Lua),
            26 => Some(Self::Arglist),
            27 => Some(Self::Environment),
            28 => Some(Self::Keymap),
            29 => Some(Self::Messages),
            30 => Some(Self::Runtime),
            31 => Some(Self::Checkhealth),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Parse from string name (e.g., "file", "buffer").
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "file" => Some(Self::File),
            "dir" => Some(Self::Dir),
            "buffer" => Some(Self::Buffer),
            "command" => Some(Self::Command),
            "event" => Some(Self::Event),
            "expression" => Some(Self::Expression),
            "function" => Some(Self::Function),
            "help" => Some(Self::Help),
            "highlight" => Some(Self::Highlight),
            "mapping" => Some(Self::Mapping),
            "menu" => Some(Self::Menu),
            "option" => Some(Self::Option),
            "shellcmd" => Some(Self::Shellcmd),
            "sign" => Some(Self::Sign),
            "syntax" => Some(Self::Syntax),
            "tag" => Some(Self::Tag),
            "user" => Some(Self::User),
            "var" => Some(Self::Var),
            "color" => Some(Self::Color),
            "compiler" => Some(Self::Compiler),
            "filetype" => Some(Self::Filetype),
            "locale" => Some(Self::Locale),
            "custom" => Some(Self::Custom),
            "customlist" => Some(Self::CustomList),
            "lua" => Some(Self::Lua),
            "arglist" => Some(Self::Arglist),
            "environment" => Some(Self::Environment),
            "keymap" => Some(Self::Keymap),
            "messages" => Some(Self::Messages),
            "runtime" => Some(Self::Runtime),
            "checkhealth" => Some(Self::Checkhealth),
            _ => None,
        }
    }

    /// Get the string name for this completion type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::File => "file",
            Self::Dir => "dir",
            Self::Buffer => "buffer",
            Self::Command => "command",
            Self::Event => "event",
            Self::Expression => "expression",
            Self::Function => "function",
            Self::Help => "help",
            Self::Highlight => "highlight",
            Self::Mapping => "mapping",
            Self::Menu => "menu",
            Self::Option => "option",
            Self::Shellcmd => "shellcmd",
            Self::Sign => "sign",
            Self::Syntax => "syntax",
            Self::Tag => "tag",
            Self::User => "user",
            Self::Var => "var",
            Self::Color => "color",
            Self::Compiler => "compiler",
            Self::Filetype => "filetype",
            Self::Locale => "locale",
            Self::Custom => "custom",
            Self::CustomList => "customlist",
            Self::Lua => "lua",
            Self::Arglist => "arglist",
            Self::Environment => "environment",
            Self::Keymap => "keymap",
            Self::Messages => "messages",
            Self::Runtime => "runtime",
            Self::Checkhealth => "checkhealth",
        }
    }
}

// =============================================================================
// User Command Nargs
// =============================================================================

/// Number of arguments specification for user commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UserCmdNargs {
    /// No arguments
    #[default]
    Zero,
    /// Zero or one argument
    ZeroOrOne,
    /// One argument
    One,
    /// Zero or more arguments
    ZeroOrMore,
    /// One or more arguments
    OneOrMore,
}

impl UserCmdNargs {
    /// Parse from string specification.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "0" => Some(Self::Zero),
            "?" => Some(Self::ZeroOrOne),
            "1" => Some(Self::One),
            "*" => Some(Self::ZeroOrMore),
            "+" => Some(Self::OneOrMore),
            _ => None,
        }
    }

    /// Check if arguments are required.
    #[must_use]
    pub const fn requires_args(&self) -> bool {
        matches!(self, Self::One | Self::OneOrMore)
    }

    /// Check if arguments are allowed.
    #[must_use]
    pub const fn allows_args(&self) -> bool {
        !matches!(self, Self::Zero)
    }
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Check if a string is a valid user command name (FFI).
///
/// # Safety
///
/// `name` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_valid_usercmd_name(name: *const c_char, len: usize) -> c_int {
    if name.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(name.cast::<u8>(), len);
    c_int::from(is_valid_usercmd_name(bytes))
}

/// Check if a user command name matches a prefix (FFI).
///
/// # Safety
///
/// `name` and `prefix` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_usercmd_matches_prefix(
    name: *const c_char,
    name_len: usize,
    prefix: *const c_char,
    prefix_len: usize,
) -> c_int {
    if name.is_null() {
        return 0;
    }

    let name_bytes = std::slice::from_raw_parts(name.cast::<u8>(), name_len);
    let prefix_bytes = if prefix.is_null() || prefix_len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(prefix.cast::<u8>(), prefix_len)
    };

    c_int::from(usercmd_matches_prefix(name_bytes, prefix_bytes))
}

/// Parse user command completion type from string (FFI).
///
/// # Safety
///
/// `s` must be a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_parse_usercmd_complete(s: *const c_char) -> c_int {
    if s.is_null() {
        return UserCmdComplete::None.to_raw();
    }

    let c_str = std::ffi::CStr::from_ptr(s);
    let Ok(str_slice) = c_str.to_str() else {
        return UserCmdComplete::None.to_raw();
    };

    UserCmdComplete::parse(str_slice)
        .map_or(UserCmdComplete::None.to_raw(), UserCmdComplete::to_raw)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_usercmd_name() {
        assert!(is_valid_usercmd_name(b"MyCommand"));
        assert!(is_valid_usercmd_name(b"X"));
        assert!(is_valid_usercmd_name(b"Test123"));

        assert!(!is_valid_usercmd_name(b""));
        assert!(!is_valid_usercmd_name(b"mycommand")); // lowercase start
        assert!(!is_valid_usercmd_name(b"123")); // starts with digit
        assert!(!is_valid_usercmd_name(b"My_Command")); // underscore
        assert!(!is_valid_usercmd_name(b"My-Command")); // hyphen
    }

    #[test]
    fn test_is_valid_usercmd_prefix() {
        assert!(is_valid_usercmd_prefix(b""));
        assert!(is_valid_usercmd_prefix(b"M"));
        assert!(is_valid_usercmd_prefix(b"My"));
        assert!(is_valid_usercmd_prefix(b"MyC"));

        assert!(!is_valid_usercmd_prefix(b"m"));
        assert!(!is_valid_usercmd_prefix(b"1"));
    }

    #[test]
    fn test_usercmd_matches_prefix() {
        assert!(usercmd_matches_prefix(b"MyCommand", b""));
        assert!(usercmd_matches_prefix(b"MyCommand", b"M"));
        assert!(usercmd_matches_prefix(b"MyCommand", b"My"));
        assert!(usercmd_matches_prefix(b"MyCommand", b"MyCommand"));

        assert!(!usercmd_matches_prefix(b"MyCommand", b"MyCommands"));
        assert!(!usercmd_matches_prefix(b"MyCommand", b"X"));
    }

    #[test]
    fn test_usercmd_complete_roundtrip() {
        for i in 0..=31 {
            if let Some(complete) = UserCmdComplete::from_raw(i) {
                assert_eq!(complete.to_raw(), i);
            }
        }
    }

    #[test]
    fn test_usercmd_complete_parse() {
        assert_eq!(UserCmdComplete::parse("file"), Some(UserCmdComplete::File));
        assert_eq!(
            UserCmdComplete::parse("buffer"),
            Some(UserCmdComplete::Buffer)
        );
        assert_eq!(
            UserCmdComplete::parse("command"),
            Some(UserCmdComplete::Command)
        );
        assert_eq!(UserCmdComplete::parse("unknown"), None);
    }

    #[test]
    fn test_usercmd_nargs() {
        assert_eq!(UserCmdNargs::parse("0"), Some(UserCmdNargs::Zero));
        assert_eq!(UserCmdNargs::parse("?"), Some(UserCmdNargs::ZeroOrOne));
        assert_eq!(UserCmdNargs::parse("1"), Some(UserCmdNargs::One));
        assert_eq!(UserCmdNargs::parse("*"), Some(UserCmdNargs::ZeroOrMore));
        assert_eq!(UserCmdNargs::parse("+"), Some(UserCmdNargs::OneOrMore));
        assert_eq!(UserCmdNargs::parse("x"), None);

        assert!(!UserCmdNargs::Zero.requires_args());
        assert!(UserCmdNargs::One.requires_args());
        assert!(UserCmdNargs::OneOrMore.requires_args());
        assert!(!UserCmdNargs::ZeroOrMore.requires_args());

        assert!(!UserCmdNargs::Zero.allows_args());
        assert!(UserCmdNargs::ZeroOrOne.allows_args());
        assert!(UserCmdNargs::One.allows_args());
    }
}
