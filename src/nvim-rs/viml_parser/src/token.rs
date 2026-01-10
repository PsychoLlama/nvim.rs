//! VimL token types and classification.
//!
//! This module defines token types used in VimL expression parsing,
//! including operators, literals, and delimiters.

use std::ffi::c_int;

/// Token kinds in VimL expressions.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // End/Error
    /// End of input
    Eof = 0,
    /// Invalid/error token
    Error = 1,

    // Literals (10-19)
    /// Number literal (integer or float)
    Number = 10,
    /// String literal (single or double quoted)
    String = 11,
    /// Blob literal (0z...)
    Blob = 12,
    /// Special value (v:true, v:false, v:null, v:none)
    Special = 13,

    // Identifiers (20-29)
    /// Variable name
    Ident = 20,
    /// Option name (&option)
    Option = 21,
    /// Register (@a)
    Register = 22,
    /// Environment variable ($VAR)
    EnvVar = 23,

    // Operators - Arithmetic (30-39)
    /// +
    Plus = 30,
    /// -
    Minus = 31,
    /// *
    Star = 32,
    /// /
    Slash = 33,
    /// %
    Percent = 34,

    // Operators - Comparison (40-59)
    /// ==
    Equal = 40,
    /// !=
    NotEqual = 41,
    /// <
    Less = 42,
    /// <=
    LessEqual = 43,
    /// >
    Greater = 44,
    /// >=
    GreaterEqual = 45,
    /// =~
    Match = 46,
    /// !~
    NoMatch = 47,
    /// is
    Is = 48,
    /// isnot
    IsNot = 49,

    // Operators - Logical (60-69)
    /// &&
    And = 60,
    /// ||
    Or = 61,
    /// !
    Not = 62,

    // Operators - String (70-79)
    /// .
    Dot = 70,
    /// ..
    DotDot = 71,

    // Delimiters (80-99)
    /// (
    LParen = 80,
    /// )
    RParen = 81,
    /// [
    LBracket = 82,
    /// ]
    RBracket = 83,
    /// {
    LBrace = 84,
    /// }
    RBrace = 85,
    /// ,
    Comma = 86,
    /// :
    Colon = 87,
    /// ->
    Arrow = 88,
    /// ?
    Question = 89,

    // Special (100-109)
    /// lambda expression {args -> expr}
    Lambda = 100,
    /// Funcref
    Funcref = 101,
}

impl TokenKind {
    /// Check if this token is an operator.
    #[must_use]
    pub const fn is_operator(self) -> bool {
        let v = self as i32;
        (v >= 30 && v <= 69) || (v >= 70 && v <= 79)
    }

    /// Check if this token is a comparison operator.
    #[must_use]
    pub const fn is_comparison(self) -> bool {
        let v = self as i32;
        v >= 40 && v <= 49
    }

    /// Check if this token is a literal.
    #[must_use]
    pub const fn is_literal(self) -> bool {
        let v = self as i32;
        v >= 10 && v <= 13
    }

    /// Check if this token is an identifier-like token.
    #[must_use]
    pub const fn is_ident_like(self) -> bool {
        let v = self as i32;
        v >= 20 && v <= 23
    }

    /// Check if this token starts a primary expression.
    #[must_use]
    pub const fn is_primary_start(self) -> bool {
        self.is_literal()
            || self.is_ident_like()
            || matches!(
                self,
                Self::LParen | Self::LBracket | Self::LBrace | Self::Not | Self::Minus
            )
    }

    /// Get operator precedence (higher = binds tighter).
    /// Returns 0 for non-operators.
    #[must_use]
    pub const fn precedence(self) -> i32 {
        match self {
            // Ternary ?: has lowest precedence
            Self::Question => 1,
            // Logical OR
            Self::Or => 2,
            // Logical AND
            Self::And => 3,
            // Comparison operators
            Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::Match
            | Self::NoMatch
            | Self::Is
            | Self::IsNot => 4,
            // String concatenation
            Self::Dot | Self::DotDot => 5,
            // Addition/Subtraction
            Self::Plus | Self::Minus => 6,
            // Multiplication/Division/Modulo
            Self::Star | Self::Slash | Self::Percent => 7,
            // Unary (not used for precedence comparison, handled specially)
            Self::Not => 8,
            _ => 0,
        }
    }

    /// Check if operator is right-associative.
    #[must_use]
    pub const fn is_right_assoc(self) -> bool {
        matches!(self, Self::Question)
    }
}

impl From<TokenKind> for i32 {
    fn from(kind: TokenKind) -> i32 {
        kind as i32
    }
}

impl TryFrom<i32> for TokenKind {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::Eof),
            1 => Ok(Self::Error),
            10 => Ok(Self::Number),
            11 => Ok(Self::String),
            12 => Ok(Self::Blob),
            13 => Ok(Self::Special),
            20 => Ok(Self::Ident),
            21 => Ok(Self::Option),
            22 => Ok(Self::Register),
            23 => Ok(Self::EnvVar),
            30 => Ok(Self::Plus),
            31 => Ok(Self::Minus),
            32 => Ok(Self::Star),
            33 => Ok(Self::Slash),
            34 => Ok(Self::Percent),
            40 => Ok(Self::Equal),
            41 => Ok(Self::NotEqual),
            42 => Ok(Self::Less),
            43 => Ok(Self::LessEqual),
            44 => Ok(Self::Greater),
            45 => Ok(Self::GreaterEqual),
            46 => Ok(Self::Match),
            47 => Ok(Self::NoMatch),
            48 => Ok(Self::Is),
            49 => Ok(Self::IsNot),
            60 => Ok(Self::And),
            61 => Ok(Self::Or),
            62 => Ok(Self::Not),
            70 => Ok(Self::Dot),
            71 => Ok(Self::DotDot),
            80 => Ok(Self::LParen),
            81 => Ok(Self::RParen),
            82 => Ok(Self::LBracket),
            83 => Ok(Self::RBracket),
            84 => Ok(Self::LBrace),
            85 => Ok(Self::RBrace),
            86 => Ok(Self::Comma),
            87 => Ok(Self::Colon),
            88 => Ok(Self::Arrow),
            89 => Ok(Self::Question),
            100 => Ok(Self::Lambda),
            101 => Ok(Self::Funcref),
            _ => Err(()),
        }
    }
}

/// A token with its kind and position.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Token {
    /// Token kind
    pub kind: TokenKind,
    /// Start offset in input
    pub start: u32,
    /// Length in bytes
    pub len: u32,
}

impl Token {
    /// Create a new token.
    #[must_use]
    pub const fn new(kind: TokenKind, start: u32, len: u32) -> Self {
        Self { kind, start, len }
    }

    /// Create an EOF token at the given position.
    #[must_use]
    pub const fn eof(pos: u32) -> Self {
        Self::new(TokenKind::Eof, pos, 0)
    }

    /// Create an error token at the given position.
    #[must_use]
    pub const fn error(start: u32, len: u32) -> Self {
        Self::new(TokenKind::Error, start, len)
    }

    /// Get the end position of this token.
    #[must_use]
    pub const fn end(&self) -> u32 {
        self.start + self.len
    }

    /// Check if this is an EOF token.
    #[must_use]
    pub const fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }

    /// Check if this is an error token.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.kind, TokenKind::Error)
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::eof(0)
    }
}

// =============================================================================
// Character Classification
// =============================================================================

/// Check if a byte is a valid first character of an identifier.
#[must_use]
pub const fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

/// Check if a byte is a valid identifier continuation character.
#[must_use]
pub const fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Check if a byte is a digit.
#[must_use]
pub const fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Check if a byte is a hex digit.
#[must_use]
pub const fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Check if a byte is an octal digit.
#[must_use]
pub const fn is_octal_digit(c: u8) -> bool {
    c >= b'0' && c <= b'7'
}

/// Check if a byte is a binary digit.
#[must_use]
pub const fn is_binary_digit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Check if a byte is whitespace.
#[must_use]
pub const fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
}

// =============================================================================
// Single-Character Token Recognition
// =============================================================================

/// Try to recognize a single-character operator or delimiter.
#[must_use]
pub const fn single_char_token(c: u8) -> Option<TokenKind> {
    match c {
        b'+' => Some(TokenKind::Plus),
        b'-' => Some(TokenKind::Minus),
        b'*' => Some(TokenKind::Star),
        b'/' => Some(TokenKind::Slash),
        b'%' => Some(TokenKind::Percent),
        b'(' => Some(TokenKind::LParen),
        b')' => Some(TokenKind::RParen),
        b'[' => Some(TokenKind::LBracket),
        b']' => Some(TokenKind::RBracket),
        b'{' => Some(TokenKind::LBrace),
        b'}' => Some(TokenKind::RBrace),
        b',' => Some(TokenKind::Comma),
        b':' => Some(TokenKind::Colon),
        b'?' => Some(TokenKind::Question),
        _ => None,
    }
}

/// Try to recognize a two-character operator.
#[must_use]
pub const fn two_char_token(c1: u8, c2: u8) -> Option<TokenKind> {
    match (c1, c2) {
        (b'=', b'=') => Some(TokenKind::Equal),
        (b'!', b'=') => Some(TokenKind::NotEqual),
        (b'<', b'=') => Some(TokenKind::LessEqual),
        (b'>', b'=') => Some(TokenKind::GreaterEqual),
        (b'=', b'~') => Some(TokenKind::Match),
        (b'!', b'~') => Some(TokenKind::NoMatch),
        (b'&', b'&') => Some(TokenKind::And),
        (b'|', b'|') => Some(TokenKind::Or),
        (b'.', b'.') => Some(TokenKind::DotDot),
        (b'-', b'>') => Some(TokenKind::Arrow),
        _ => None,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get token kind value.
#[no_mangle]
pub extern "C" fn rs_token_kind_value(kind: c_int) -> c_int {
    kind
}

/// Check if token kind is an operator.
#[no_mangle]
pub extern "C" fn rs_token_is_operator(kind: c_int) -> c_int {
    TokenKind::try_from(kind).map_or(0, |k| c_int::from(k.is_operator()))
}

/// Check if token kind is a comparison operator.
#[no_mangle]
pub extern "C" fn rs_token_is_comparison(kind: c_int) -> c_int {
    TokenKind::try_from(kind).map_or(0, |k| c_int::from(k.is_comparison()))
}

/// Check if token kind is a literal.
#[no_mangle]
pub extern "C" fn rs_token_is_literal(kind: c_int) -> c_int {
    TokenKind::try_from(kind).map_or(0, |k| c_int::from(k.is_literal()))
}

/// Get operator precedence.
#[no_mangle]
pub extern "C" fn rs_token_precedence(kind: c_int) -> c_int {
    TokenKind::try_from(kind).map_or(0, TokenKind::precedence)
}

/// Check if character is identifier start.
#[no_mangle]
pub extern "C" fn rs_is_ident_start(c: c_int) -> c_int {
    c_int::from(c >= 0 && c <= 127 && is_ident_start(c as u8))
}

/// Check if character is identifier char.
#[no_mangle]
pub extern "C" fn rs_is_ident_char(c: c_int) -> c_int {
    c_int::from(c >= 0 && c <= 127 && is_ident_char(c as u8))
}

/// Try to get single-char token kind.
#[no_mangle]
pub extern "C" fn rs_single_char_token(c: c_int) -> c_int {
    if c < 0 || c > 127 {
        return -1;
    }
    single_char_token(c as u8).map_or(-1, |k| k as c_int)
}

/// Try to get two-char token kind.
#[no_mangle]
pub extern "C" fn rs_two_char_token(c1: c_int, c2: c_int) -> c_int {
    if c1 < 0 || c1 > 127 || c2 < 0 || c2 > 127 {
        return -1;
    }
    two_char_token(c1 as u8, c2 as u8).map_or(-1, |k| k as c_int)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_kind_values() {
        assert_eq!(TokenKind::Eof as i32, 0);
        assert_eq!(TokenKind::Error as i32, 1);
        assert_eq!(TokenKind::Number as i32, 10);
        assert_eq!(TokenKind::Plus as i32, 30);
        assert_eq!(TokenKind::Equal as i32, 40);
    }

    #[test]
    fn test_token_kind_is_operator() {
        assert!(TokenKind::Plus.is_operator());
        assert!(TokenKind::Equal.is_operator());
        assert!(TokenKind::And.is_operator());
        assert!(TokenKind::Dot.is_operator());
        assert!(!TokenKind::Number.is_operator());
        assert!(!TokenKind::LParen.is_operator());
    }

    #[test]
    fn test_token_kind_is_comparison() {
        assert!(TokenKind::Equal.is_comparison());
        assert!(TokenKind::NotEqual.is_comparison());
        assert!(TokenKind::Less.is_comparison());
        assert!(TokenKind::Is.is_comparison());
        assert!(!TokenKind::Plus.is_comparison());
        assert!(!TokenKind::And.is_comparison());
    }

    #[test]
    fn test_token_kind_is_literal() {
        assert!(TokenKind::Number.is_literal());
        assert!(TokenKind::String.is_literal());
        assert!(TokenKind::Blob.is_literal());
        assert!(!TokenKind::Ident.is_literal());
        assert!(!TokenKind::Plus.is_literal());
    }

    #[test]
    fn test_token_kind_precedence() {
        // Higher precedence = binds tighter
        assert!(TokenKind::Star.precedence() > TokenKind::Plus.precedence());
        assert!(TokenKind::Plus.precedence() > TokenKind::Equal.precedence());
        assert!(TokenKind::And.precedence() > TokenKind::Or.precedence());
        assert!(TokenKind::Equal.precedence() > TokenKind::And.precedence());
        assert_eq!(TokenKind::LParen.precedence(), 0);
    }

    #[test]
    fn test_token_kind_try_from() {
        assert_eq!(TokenKind::try_from(0), Ok(TokenKind::Eof));
        assert_eq!(TokenKind::try_from(10), Ok(TokenKind::Number));
        assert_eq!(TokenKind::try_from(30), Ok(TokenKind::Plus));
        assert!(TokenKind::try_from(999).is_err());
    }

    #[test]
    fn test_token() {
        let tok = Token::new(TokenKind::Number, 5, 3);
        assert_eq!(tok.kind, TokenKind::Number);
        assert_eq!(tok.start, 5);
        assert_eq!(tok.len, 3);
        assert_eq!(tok.end(), 8);
        assert!(!tok.is_eof());
        assert!(!tok.is_error());

        let eof = Token::eof(10);
        assert!(eof.is_eof());

        let err = Token::error(2, 1);
        assert!(err.is_error());
    }

    #[test]
    fn test_is_ident_start() {
        assert!(is_ident_start(b'a'));
        assert!(is_ident_start(b'Z'));
        assert!(is_ident_start(b'_'));
        assert!(!is_ident_start(b'0'));
        assert!(!is_ident_start(b' '));
    }

    #[test]
    fn test_is_ident_char() {
        assert!(is_ident_char(b'a'));
        assert!(is_ident_char(b'Z'));
        assert!(is_ident_char(b'0'));
        assert!(is_ident_char(b'_'));
        assert!(!is_ident_char(b' '));
        assert!(!is_ident_char(b'-'));
    }

    #[test]
    fn test_single_char_token() {
        assert_eq!(single_char_token(b'+'), Some(TokenKind::Plus));
        assert_eq!(single_char_token(b'-'), Some(TokenKind::Minus));
        assert_eq!(single_char_token(b'('), Some(TokenKind::LParen));
        assert_eq!(single_char_token(b','), Some(TokenKind::Comma));
        assert_eq!(single_char_token(b'a'), None);
        assert_eq!(single_char_token(b'='), None); // Needs second char
    }

    #[test]
    fn test_two_char_token() {
        assert_eq!(two_char_token(b'=', b'='), Some(TokenKind::Equal));
        assert_eq!(two_char_token(b'!', b'='), Some(TokenKind::NotEqual));
        assert_eq!(two_char_token(b'&', b'&'), Some(TokenKind::And));
        assert_eq!(two_char_token(b'|', b'|'), Some(TokenKind::Or));
        assert_eq!(two_char_token(b'-', b'>'), Some(TokenKind::Arrow));
        assert_eq!(two_char_token(b'+', b'+'), None);
    }
}
