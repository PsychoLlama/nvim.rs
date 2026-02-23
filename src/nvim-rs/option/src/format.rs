//! Option value formatting helpers
//!
//! This module provides helpers for formatting option values for display,
//! including escaping, quoting, and type-specific formatting.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]

use std::ffi::c_int;

use crate::{OptInt, OptValType};

// =============================================================================
// Format Flags
// =============================================================================

/// Flags controlling option value formatting.
pub mod format_flags {
    use std::ffi::c_int;

    /// Quote string values
    pub const FMT_QUOTE: c_int = 0x01;
    /// Escape special characters
    pub const FMT_ESCAPE: c_int = 0x02;
    /// Show type prefix (n:, s:, b:)
    pub const FMT_TYPE_PREFIX: c_int = 0x04;
    /// Abbreviate long values
    pub const FMT_ABBREVIATE: c_int = 0x08;
    /// Show as Lua value
    pub const FMT_LUA: c_int = 0x10;
    /// Show as Vim expression
    pub const FMT_EXPR: c_int = 0x20;
}

/// Check if format flags include a specific flag.
#[must_use]
#[inline]
pub const fn has_format_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

// =============================================================================
// Boolean Formatting
// =============================================================================

/// Result of formatting a boolean option.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolFormat {
    /// "on"
    On = 0,
    /// "off"
    Off = 1,
    /// "true"
    True = 2,
    /// "false"
    False = 3,
    /// "1"
    One = 4,
    /// "0"
    Zero = 5,
}

impl BoolFormat {
    /// Get the string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
            Self::True => "true",
            Self::False => "false",
            Self::One => "1",
            Self::Zero => "0",
        }
    }

    /// Get the length of the string.
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::On => 2,
            Self::Off => 3,
            Self::True => 4,
            Self::False => 5,
            Self::One => 1,
            Self::Zero => 1,
        }
    }
}

/// Format a boolean value for display.
#[must_use]
pub const fn format_bool(value: bool, flags: c_int) -> BoolFormat {
    if has_format_flag(flags, format_flags::FMT_LUA) {
        if value {
            BoolFormat::True
        } else {
            BoolFormat::False
        }
    } else if has_format_flag(flags, format_flags::FMT_EXPR) {
        if value {
            BoolFormat::One
        } else {
            BoolFormat::Zero
        }
    } else if value {
        BoolFormat::On
    } else {
        BoolFormat::Off
    }
}

// =============================================================================
// Number Formatting
// =============================================================================

/// Number format style.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberStyle {
    /// Decimal (default)
    #[default]
    Decimal = 0,
    /// Hexadecimal (0x prefix)
    Hex = 1,
    /// Octal (0 prefix)
    Octal = 2,
    /// Binary (0b prefix)
    Binary = 3,
}

impl NumberStyle {
    /// Get the prefix for this style.
    #[must_use]
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Decimal => "",
            Self::Hex => "0x",
            Self::Octal => "0",
            Self::Binary => "0b",
        }
    }

    /// Get the radix for this style.
    #[must_use]
    pub const fn radix(&self) -> u32 {
        match self {
            Self::Decimal => 10,
            Self::Hex => 16,
            Self::Octal => 8,
            Self::Binary => 2,
        }
    }
}

/// State for formatting a number.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NumberFormat {
    /// The style to use
    pub style: NumberStyle,
    /// Minimum width (for padding)
    pub min_width: c_int,
    /// Whether to show sign for positive
    pub show_plus: bool,
    /// Whether to pad with zeros
    pub zero_pad: bool,
}

impl NumberFormat {
    /// Create a default decimal format.
    #[must_use]
    pub const fn decimal() -> Self {
        Self {
            style: NumberStyle::Decimal,
            min_width: 0,
            show_plus: false,
            zero_pad: false,
        }
    }

    /// Create a hex format.
    #[must_use]
    pub const fn hex() -> Self {
        Self {
            style: NumberStyle::Hex,
            min_width: 0,
            show_plus: false,
            zero_pad: false,
        }
    }

    /// Estimate buffer size needed for a number.
    #[must_use]
    pub const fn estimate_size(&self, value: OptInt) -> usize {
        // Sign + prefix + digits
        let sign_size = if value < 0 || self.show_plus { 1 } else { 0 };
        let prefix_size = self.style.prefix().len();
        // Max digits for i64: 20 decimal, 16 hex, etc.
        let digit_size = match self.style {
            NumberStyle::Decimal => 20,
            NumberStyle::Hex => 16,
            NumberStyle::Octal => 22,
            NumberStyle::Binary => 64,
        };
        sign_size + prefix_size + digit_size
    }
}

// =============================================================================
// String Escaping
// =============================================================================

/// Characters that need escaping in string values.
pub mod escape_chars {
    /// Space character
    pub const SPACE: u8 = b' ';
    /// Backslash
    pub const BACKSLASH: u8 = b'\\';
    /// Double quote
    pub const DQUOTE: u8 = b'"';
    /// Single quote
    pub const SQUOTE: u8 = b'\'';
    /// Tab
    pub const TAB: u8 = b'\t';
    /// Newline
    pub const NEWLINE: u8 = b'\n';
    /// Carriage return
    pub const CR: u8 = b'\r';
    /// Comma (in comma-separated options)
    pub const COMMA: u8 = b',';
    /// Colon (in colon-separated options)
    pub const COLON: u8 = b':';
}

/// Check if a character needs escaping.
#[must_use]
pub const fn needs_escape(c: u8) -> bool {
    matches!(
        c,
        escape_chars::SPACE
            | escape_chars::BACKSLASH
            | escape_chars::DQUOTE
            | escape_chars::TAB
            | escape_chars::NEWLINE
            | escape_chars::CR
            | escape_chars::COMMA
    )
}

/// Check if a string needs quoting.
#[must_use]
pub fn needs_quoting(s: &[u8]) -> bool {
    if s.is_empty() {
        return true;
    }

    for &c in s {
        if needs_escape(c) {
            return true;
        }
    }

    false
}

/// Estimate escaped string length.
#[must_use]
pub fn escaped_len(s: &[u8]) -> usize {
    let mut len = 0;
    for &c in s {
        if needs_escape(c) {
            len += 2; // backslash + char
        } else if c < 32 {
            len += 4; // \xNN
        } else {
            len += 1;
        }
    }
    len
}

// =============================================================================
// Value Display State
// =============================================================================

/// State for displaying an option value.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueDisplay {
    /// Option type
    pub opt_type: c_int,
    /// Format flags
    pub format_flags: c_int,
    /// Maximum display length (0 = unlimited)
    pub max_len: usize,
    /// Whether value is truncated
    pub truncated: bool,
}

impl ValueDisplay {
    /// Create a new value display state.
    #[must_use]
    pub const fn new(opt_type: OptValType) -> Self {
        Self {
            opt_type: opt_type as c_int,
            format_flags: 0,
            max_len: 0,
            truncated: false,
        }
    }

    /// Create with quoting enabled.
    #[must_use]
    pub const fn quoted(opt_type: OptValType) -> Self {
        Self {
            opt_type: opt_type as c_int,
            format_flags: format_flags::FMT_QUOTE | format_flags::FMT_ESCAPE,
            max_len: 0,
            truncated: false,
        }
    }

    /// Create for Lua output.
    #[must_use]
    pub const fn for_lua(opt_type: OptValType) -> Self {
        Self {
            opt_type: opt_type as c_int,
            format_flags: format_flags::FMT_LUA | format_flags::FMT_QUOTE,
            max_len: 0,
            truncated: false,
        }
    }

    /// Get the option type.
    #[must_use]
    pub const fn get_type(&self) -> OptValType {
        match self.opt_type {
            0 => OptValType::Boolean,
            1 => OptValType::Number,
            2 => OptValType::String,
            _ => OptValType::Nil,
        }
    }

    /// Check if quoting is enabled.
    #[must_use]
    pub const fn is_quoted(&self) -> bool {
        has_format_flag(self.format_flags, format_flags::FMT_QUOTE)
    }

    /// Check if escaping is enabled.
    #[must_use]
    pub const fn is_escaped(&self) -> bool {
        has_format_flag(self.format_flags, format_flags::FMT_ESCAPE)
    }
}

// =============================================================================
// List Formatting
// =============================================================================

/// State for formatting comma-separated list options.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ListFormat {
    /// Separator character (usually comma or colon)
    pub separator: u8,
    /// Whether to sort items
    pub sort: bool,
    /// Whether to remove duplicates
    pub unique: bool,
    /// Maximum items to show (0 = all)
    pub max_items: c_int,
}

impl ListFormat {
    /// Create a comma-separated format.
    #[must_use]
    pub const fn comma() -> Self {
        Self {
            separator: b',',
            sort: false,
            unique: false,
            max_items: 0,
        }
    }

    /// Create a colon-separated format.
    #[must_use]
    pub const fn colon() -> Self {
        Self {
            separator: b':',
            sort: false,
            unique: false,
            max_items: 0,
        }
    }

    /// Create a flag list format (single chars).
    #[must_use]
    pub const fn flags() -> Self {
        Self {
            separator: 0, // No separator for flags
            sort: true,
            unique: true,
            max_items: 0,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Format a boolean value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_format_bool(value: c_int, flags: c_int) -> c_int {
    format_bool(value != 0, flags) as c_int
}

/// Check if a byte needs escaping.
#[unsafe(no_mangle)]
pub extern "C" fn rs_needs_escape(c: c_int) -> c_int {
    if c < 0 || c > 255 {
        return 0;
    }
    c_int::from(needs_escape(c as u8))
}

/// Get number format radix.
#[unsafe(no_mangle)]
pub extern "C" fn rs_number_radix(style: c_int) -> c_int {
    let style = match style {
        1 => NumberStyle::Hex,
        2 => NumberStyle::Octal,
        3 => NumberStyle::Binary,
        _ => NumberStyle::Decimal,
    };
    style.radix() as c_int
}

/// Estimate escaped string length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_escaped_len(s: *const u8, len: usize) -> usize {
    if s.is_null() || len == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(s, len);
    escaped_len(slice)
}

// =============================================================================
// File Format Option Setter
// =============================================================================

/// EOL style constants (from fileio.h)
const EOL_UNIX: c_int = 0;
const EOL_DOS: c_int = 1;
const EOL_MAC: c_int = 2;

extern "C" {
    /// C helper: set 'fileformat' option string and trigger redraws.
    fn nvim_set_fileformat_option(p: *const std::ffi::c_char, opt_flags: c_int);
}

/// Set the 'fileformat' option to match an EOL style, then trigger redraws.
///
/// Maps EOL style codes to their string names ("unix", "dos", "mac").
/// If `eol_style` is not a recognized code, no option change is made but
/// redraws are still triggered.
#[no_mangle]
pub unsafe extern "C" fn rs_set_fileformat(eol_style: c_int, opt_flags: c_int) {
    let p: *const std::ffi::c_char = match eol_style {
        EOL_UNIX => c"unix".as_ptr(),
        EOL_DOS => c"dos".as_ptr(),
        EOL_MAC => c"mac".as_ptr(),
        _ => std::ptr::null(),
    };
    nvim_set_fileformat_option(p, opt_flags);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_flags() {
        let flags = format_flags::FMT_QUOTE | format_flags::FMT_ESCAPE;
        assert!(has_format_flag(flags, format_flags::FMT_QUOTE));
        assert!(has_format_flag(flags, format_flags::FMT_ESCAPE));
        assert!(!has_format_flag(flags, format_flags::FMT_LUA));
    }

    #[test]
    fn test_bool_format() {
        // Default format
        assert_eq!(format_bool(true, 0), BoolFormat::On);
        assert_eq!(format_bool(false, 0), BoolFormat::Off);

        // Lua format
        assert_eq!(format_bool(true, format_flags::FMT_LUA), BoolFormat::True);
        assert_eq!(format_bool(false, format_flags::FMT_LUA), BoolFormat::False);

        // Expr format
        assert_eq!(format_bool(true, format_flags::FMT_EXPR), BoolFormat::One);
        assert_eq!(format_bool(false, format_flags::FMT_EXPR), BoolFormat::Zero);
    }

    #[test]
    fn test_bool_format_str() {
        assert_eq!(BoolFormat::On.as_str(), "on");
        assert_eq!(BoolFormat::Off.as_str(), "off");
        assert_eq!(BoolFormat::True.as_str(), "true");
        assert_eq!(BoolFormat::False.as_str(), "false");
        assert_eq!(BoolFormat::On.len(), 2);
        assert_eq!(BoolFormat::True.len(), 4);
    }

    #[test]
    fn test_number_style() {
        assert_eq!(NumberStyle::Decimal.prefix(), "");
        assert_eq!(NumberStyle::Hex.prefix(), "0x");
        assert_eq!(NumberStyle::Octal.prefix(), "0");
        assert_eq!(NumberStyle::Binary.prefix(), "0b");

        assert_eq!(NumberStyle::Decimal.radix(), 10);
        assert_eq!(NumberStyle::Hex.radix(), 16);
        assert_eq!(NumberStyle::Octal.radix(), 8);
        assert_eq!(NumberStyle::Binary.radix(), 2);
    }

    #[test]
    fn test_number_format() {
        let fmt = NumberFormat::decimal();
        assert!(fmt.estimate_size(12345) >= 5);

        let fmt = NumberFormat::hex();
        assert_eq!(fmt.style.prefix(), "0x");
    }

    #[test]
    fn test_escape_chars() {
        assert!(needs_escape(b' '));
        assert!(needs_escape(b'\\'));
        assert!(needs_escape(b'\t'));
        assert!(needs_escape(b','));
        assert!(!needs_escape(b'a'));
        assert!(!needs_escape(b'Z'));
    }

    #[test]
    fn test_needs_quoting() {
        assert!(needs_quoting(b""));
        assert!(needs_quoting(b"hello world"));
        assert!(needs_quoting(b"a,b"));
        assert!(!needs_quoting(b"hello"));
        assert!(!needs_quoting(b"foobar123"));
    }

    #[test]
    fn test_escaped_len() {
        assert_eq!(escaped_len(b"hello"), 5);
        assert_eq!(escaped_len(b"a b"), 4); // space needs escape
        assert_eq!(escaped_len(b"a\\b"), 4); // backslash needs escape
    }

    #[test]
    fn test_value_display() {
        let disp = ValueDisplay::new(OptValType::String);
        assert_eq!(disp.get_type(), OptValType::String);
        assert!(!disp.is_quoted());

        let quoted = ValueDisplay::quoted(OptValType::String);
        assert!(quoted.is_quoted());
        assert!(quoted.is_escaped());

        let lua = ValueDisplay::for_lua(OptValType::Boolean);
        assert!(has_format_flag(lua.format_flags, format_flags::FMT_LUA));
    }

    #[test]
    fn test_list_format() {
        let comma = ListFormat::comma();
        assert_eq!(comma.separator, b',');

        let colon = ListFormat::colon();
        assert_eq!(colon.separator, b':');

        let flags = ListFormat::flags();
        assert_eq!(flags.separator, 0);
        assert!(flags.sort);
        assert!(flags.unique);
    }
}
