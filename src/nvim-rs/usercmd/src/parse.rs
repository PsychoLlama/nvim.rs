//! User command argument parsing
//!
//! This module provides Rust implementations for parsing user command
//! arguments, including tokenization, quoting, and escape handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::c_int;

// =============================================================================
// Parse State
// =============================================================================

/// State of argument parsing
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseState {
    /// Normal parsing
    Normal = 0,
    /// Inside single quotes
    SingleQuote = 1,
    /// Inside double quotes
    DoubleQuote = 2,
    /// After backslash escape
    Escape = 3,
    /// Inside bar-separated command
    BarSeparated = 4,
}

impl ParseState {
    /// Check if in a quoted context
    pub const fn is_quoted(self) -> bool {
        matches!(self, Self::SingleQuote | Self::DoubleQuote)
    }

    /// Check if in escape context
    pub const fn is_escape(self) -> bool {
        matches!(self, Self::Escape)
    }

    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::SingleQuote),
            2 => Some(Self::DoubleQuote),
            3 => Some(Self::Escape),
            4 => Some(Self::BarSeparated),
            _ => None,
        }
    }
}

impl Default for ParseState {
    fn default() -> Self {
        Self::Normal
    }
}

// =============================================================================
// Token Type
// =============================================================================

/// Type of parsed token
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Regular word
    Word = 0,
    /// Whitespace
    Whitespace = 1,
    /// Single-quoted string
    SingleQuoted = 2,
    /// Double-quoted string
    DoubleQuoted = 3,
    /// Command separator (|)
    Bar = 4,
    /// End of input
    End = 5,
    /// Error token
    Error = 6,
}

impl TokenType {
    /// Check if this is a string token
    pub const fn is_string(self) -> bool {
        matches!(self, Self::Word | Self::SingleQuoted | Self::DoubleQuoted)
    }

    /// Check if this is a separator
    pub const fn is_separator(self) -> bool {
        matches!(self, Self::Whitespace | Self::Bar | Self::End)
    }

    /// Check if this is an error
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }
}

impl Default for TokenType {
    fn default() -> Self {
        Self::End
    }
}

// =============================================================================
// Token
// =============================================================================

/// A parsed token with position information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Token {
    /// Token type
    pub ttype: TokenType,
    /// Start position in input (byte offset)
    pub start: c_int,
    /// End position in input (byte offset, exclusive)
    pub end: c_int,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            ttype: TokenType::End,
            start: 0,
            end: 0,
        }
    }
}

impl Token {
    /// Create a new token
    pub const fn new(ttype: TokenType, start: c_int, end: c_int) -> Self {
        Self { ttype, start, end }
    }

    /// Get token length
    pub const fn len(&self) -> c_int {
        self.end - self.start
    }

    /// Check if token is empty
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this is a valid token
    pub const fn is_valid(&self) -> bool {
        self.start >= 0 && self.end >= self.start && !self.ttype.is_error()
    }
}

// =============================================================================
// Parse Result
// =============================================================================

/// Result of parsing an argument list
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParseResult {
    /// Number of arguments found
    pub argc: c_int,
    /// Position of parse error (-1 if no error)
    pub error_pos: c_int,
    /// Whether parsing completed successfully
    pub success: bool,
    /// Whether there are more commands after bar
    pub has_bar: bool,
}

impl Default for ParseResult {
    fn default() -> Self {
        Self {
            argc: 0,
            error_pos: -1,
            success: true,
            has_bar: false,
        }
    }
}

impl ParseResult {
    /// Create a successful result
    pub const fn success(argc: c_int) -> Self {
        Self {
            argc,
            error_pos: -1,
            success: true,
            has_bar: false,
        }
    }

    /// Create an error result
    pub const fn error(pos: c_int) -> Self {
        Self {
            argc: 0,
            error_pos: pos,
            success: false,
            has_bar: false,
        }
    }

    /// Check if parse was successful
    pub const fn is_ok(&self) -> bool {
        self.success
    }

    /// Check if there was an error
    pub const fn is_err(&self) -> bool {
        !self.success
    }
}

// =============================================================================
// Character Classification
// =============================================================================

/// Check if character is whitespace for argument parsing
pub const fn is_arg_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if character starts a quote
pub const fn is_quote_start(c: u8) -> bool {
    c == b'\'' || c == b'"'
}

/// Check if character is escape
pub const fn is_escape_char(c: u8) -> bool {
    c == b'\\'
}

/// Check if character is command separator
pub const fn is_bar(c: u8) -> bool {
    c == b'|'
}

/// Check if character is special and needs escaping
pub const fn needs_escape(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\\' | b'"' | b'\'' | b'|' | b'<' | b'>')
}

/// Get the escaped form of a character (None if no escape needed)
pub const fn escape_char(c: u8) -> Option<u8> {
    match c {
        b'n' => Some(b'\n'),
        b'r' => Some(b'\r'),
        b't' => Some(b'\t'),
        b'e' => Some(0x1b), // ESC
        b'b' => Some(0x08), // Backspace
        _ => None,
    }
}

// =============================================================================
// Argument Counting
// =============================================================================

/// Count arguments in a string (simple whitespace split)
pub fn count_args(s: &[u8]) -> c_int {
    if s.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut in_word = false;
    let mut state = ParseState::Normal;

    for &c in s {
        match state {
            ParseState::Normal => {
                if is_arg_whitespace(c) {
                    in_word = false;
                } else if c == b'\'' {
                    if !in_word {
                        count += 1;
                    }
                    in_word = true;
                    state = ParseState::SingleQuote;
                } else if c == b'"' {
                    if !in_word {
                        count += 1;
                    }
                    in_word = true;
                    state = ParseState::DoubleQuote;
                } else if c == b'\\' {
                    if !in_word {
                        count += 1;
                        in_word = true;
                    }
                    state = ParseState::Escape;
                } else {
                    if !in_word {
                        count += 1;
                        in_word = true;
                    }
                }
            }
            ParseState::SingleQuote => {
                if c == b'\'' {
                    state = ParseState::Normal;
                }
            }
            ParseState::DoubleQuote => {
                if c == b'"' {
                    state = ParseState::Normal;
                } else if c == b'\\' {
                    state = ParseState::Escape;
                }
            }
            ParseState::Escape => {
                state = if matches!(state, ParseState::DoubleQuote) {
                    ParseState::DoubleQuote
                } else {
                    ParseState::Normal
                };
            }
            ParseState::BarSeparated => {
                // Stop counting at bar
                break;
            }
        }
    }

    count
}

// =============================================================================
// Quote Validation
// =============================================================================

/// Check if quotes are balanced in a string
pub fn quotes_balanced(s: &[u8]) -> bool {
    let mut state = ParseState::Normal;

    for &c in s {
        match state {
            ParseState::Normal => {
                if c == b'\'' {
                    state = ParseState::SingleQuote;
                } else if c == b'"' {
                    state = ParseState::DoubleQuote;
                } else if c == b'\\' {
                    state = ParseState::Escape;
                }
            }
            ParseState::SingleQuote => {
                if c == b'\'' {
                    state = ParseState::Normal;
                }
            }
            ParseState::DoubleQuote => {
                if c == b'"' {
                    state = ParseState::Normal;
                } else if c == b'\\' {
                    state = ParseState::Escape;
                }
            }
            ParseState::Escape => {
                state = ParseState::Normal;
            }
            ParseState::BarSeparated => {}
        }
    }

    matches!(state, ParseState::Normal)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if character is whitespace
#[no_mangle]
pub extern "C" fn rs_usercmd_is_arg_whitespace(c: u8) -> c_int {
    c_int::from(is_arg_whitespace(c))
}

/// FFI export: Check if character is quote start
#[no_mangle]
pub extern "C" fn rs_usercmd_is_quote_start(c: u8) -> c_int {
    c_int::from(is_quote_start(c))
}

/// FFI export: Check if character needs escaping
#[no_mangle]
pub extern "C" fn rs_usercmd_needs_escape(c: u8) -> c_int {
    c_int::from(needs_escape(c))
}

/// FFI export: Check if parse state is quoted
#[no_mangle]
pub extern "C" fn rs_usercmd_parse_state_is_quoted(state: c_int) -> c_int {
    ParseState::from_raw(state).map_or(0, |s| c_int::from(s.is_quoted()))
}

/// FFI export: Check if token type is string
#[no_mangle]
pub extern "C" fn rs_usercmd_token_is_string(ttype: c_int) -> c_int {
    let ttype = match ttype {
        0 => TokenType::Word,
        2 => TokenType::SingleQuoted,
        3 => TokenType::DoubleQuoted,
        _ => TokenType::End,
    };
    c_int::from(ttype.is_string())
}

/// FFI export: Create a token
#[no_mangle]
pub extern "C" fn rs_usercmd_token_new(ttype: c_int, start: c_int, end: c_int) -> Token {
    let token_type = match ttype {
        0 => TokenType::Word,
        1 => TokenType::Whitespace,
        2 => TokenType::SingleQuoted,
        3 => TokenType::DoubleQuoted,
        4 => TokenType::Bar,
        5 => TokenType::End,
        _ => TokenType::Error,
    };
    Token::new(token_type, start, end)
}

/// FFI export: Get token length
#[no_mangle]
pub extern "C" fn rs_usercmd_token_len(token: *const Token) -> c_int {
    if token.is_null() {
        return 0;
    }
    unsafe { (*token).len() }
}

/// FFI export: Check if token is valid
#[no_mangle]
pub extern "C" fn rs_usercmd_token_is_valid(token: *const Token) -> c_int {
    if token.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*token).is_valid() })
}

/// FFI export: Create successful parse result
#[no_mangle]
pub extern "C" fn rs_usercmd_parse_result_success(argc: c_int) -> ParseResult {
    ParseResult::success(argc)
}

/// FFI export: Create error parse result
#[no_mangle]
pub extern "C" fn rs_usercmd_parse_result_error(pos: c_int) -> ParseResult {
    ParseResult::error(pos)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state() {
        assert!(!ParseState::Normal.is_quoted());
        assert!(ParseState::SingleQuote.is_quoted());
        assert!(ParseState::DoubleQuote.is_quoted());
        assert!(ParseState::Escape.is_escape());

        assert_eq!(ParseState::from_raw(0), Some(ParseState::Normal));
        assert_eq!(ParseState::from_raw(100), None);
    }

    #[test]
    fn test_token_type() {
        assert!(TokenType::Word.is_string());
        assert!(TokenType::SingleQuoted.is_string());
        assert!(!TokenType::Whitespace.is_string());

        assert!(TokenType::Whitespace.is_separator());
        assert!(TokenType::Bar.is_separator());
        assert!(!TokenType::Word.is_separator());
    }

    #[test]
    fn test_token() {
        let token = Token::new(TokenType::Word, 0, 5);
        assert_eq!(token.len(), 5);
        assert!(!token.is_empty());
        assert!(token.is_valid());

        let empty = Token::new(TokenType::Word, 5, 5);
        assert!(empty.is_empty());

        let error = Token::new(TokenType::Error, 0, 1);
        assert!(!error.is_valid());
    }

    #[test]
    fn test_parse_result() {
        let success = ParseResult::success(3);
        assert!(success.is_ok());
        assert!(!success.is_err());
        assert_eq!(success.argc, 3);

        let error = ParseResult::error(10);
        assert!(!error.is_ok());
        assert!(error.is_err());
        assert_eq!(error.error_pos, 10);
    }

    #[test]
    fn test_char_classification() {
        assert!(is_arg_whitespace(b' '));
        assert!(is_arg_whitespace(b'\t'));
        assert!(!is_arg_whitespace(b'a'));

        assert!(is_quote_start(b'\''));
        assert!(is_quote_start(b'"'));
        assert!(!is_quote_start(b'a'));

        assert!(is_escape_char(b'\\'));
        assert!(!is_escape_char(b'/'));

        assert!(is_bar(b'|'));
        assert!(!is_bar(b'-'));
    }

    #[test]
    fn test_needs_escape() {
        assert!(needs_escape(b' '));
        assert!(needs_escape(b'\\'));
        assert!(needs_escape(b'"'));
        assert!(!needs_escape(b'a'));
    }

    #[test]
    fn test_escape_char() {
        assert_eq!(escape_char(b'n'), Some(b'\n'));
        assert_eq!(escape_char(b't'), Some(b'\t'));
        assert_eq!(escape_char(b'a'), None);
    }

    #[test]
    fn test_count_args() {
        assert_eq!(count_args(b""), 0);
        assert_eq!(count_args(b"one"), 1);
        assert_eq!(count_args(b"one two"), 2);
        assert_eq!(count_args(b"one two three"), 3);
        assert_eq!(count_args(b"  one  two  "), 2);
        assert_eq!(count_args(b"'one two'"), 1);
        assert_eq!(count_args(b"\"one two\""), 1);
    }

    #[test]
    fn test_quotes_balanced() {
        assert!(quotes_balanced(b"no quotes"));
        assert!(quotes_balanced(b"'single'"));
        assert!(quotes_balanced(b"\"double\""));
        assert!(quotes_balanced(b"'a' \"b\""));
        assert!(!quotes_balanced(b"'unbalanced"));
        assert!(!quotes_balanced(b"\"unbalanced"));
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_is_arg_whitespace(b' '), 1);
        assert_eq!(rs_usercmd_is_arg_whitespace(b'a'), 0);

        assert_eq!(rs_usercmd_is_quote_start(b'"'), 1);
        assert_eq!(rs_usercmd_is_quote_start(b'a'), 0);

        assert_eq!(rs_usercmd_needs_escape(b' '), 1);
        assert_eq!(rs_usercmd_needs_escape(b'a'), 0);

        assert_eq!(rs_usercmd_parse_state_is_quoted(1), 1);
        assert_eq!(rs_usercmd_parse_state_is_quoted(0), 0);

        let token = rs_usercmd_token_new(0, 0, 5);
        assert_eq!(rs_usercmd_token_len(&token), 5);
    }
}
