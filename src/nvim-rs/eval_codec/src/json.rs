//! JSON encoding and decoding helpers.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::doc_markdown)]

use std::ffi::c_int;

// =============================================================================
// JSON Value Types
// =============================================================================

/// JSON value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum JsonType {
    /// JSON null
    Null = 0,
    /// JSON boolean
    Bool = 1,
    /// JSON number (integer)
    Integer = 2,
    /// JSON number (float)
    Float = 3,
    /// JSON string
    String = 4,
    /// JSON array
    Array = 5,
    /// JSON object
    Object = 6,
}

impl JsonType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Bool,
            2 => Self::Integer,
            3 => Self::Float,
            4 => Self::String,
            5 => Self::Array,
            6 => Self::Object,
            _ => Self::Null,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if type is a container (array or object).
    pub const fn is_container(&self) -> bool {
        matches!(self, Self::Array | Self::Object)
    }

    /// Check if type is a primitive (not container).
    pub const fn is_primitive(&self) -> bool {
        !self.is_container()
    }
}

// =============================================================================
// JSON Token Types
// =============================================================================

/// JSON lexer token type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum JsonToken {
    /// End of input
    Eof = 0,
    /// Syntax error
    Error = 1,
    /// Left brace {
    LeftBrace = 2,
    /// Right brace }
    RightBrace = 3,
    /// Left bracket [
    LeftBracket = 4,
    /// Right bracket ]
    RightBracket = 5,
    /// Comma ,
    Comma = 6,
    /// Colon :
    Colon = 7,
    /// String literal
    String = 8,
    /// Number literal
    Number = 9,
    /// true keyword
    True = 10,
    /// false keyword
    False = 11,
    /// null keyword
    Null = 12,
}

impl JsonToken {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Eof,
            2 => Self::LeftBrace,
            3 => Self::RightBrace,
            4 => Self::LeftBracket,
            5 => Self::RightBracket,
            6 => Self::Comma,
            7 => Self::Colon,
            8 => Self::String,
            9 => Self::Number,
            10 => Self::True,
            11 => Self::False,
            12 => Self::Null,
            _ => Self::Error,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// JSON Parsing Helpers
// =============================================================================

/// Skip whitespace in JSON input.
pub fn skip_json_whitespace(input: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < input.len() {
        match input[i] {
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            _ => break,
        }
    }
    &input[i..]
}

/// Check if character is JSON whitespace.
pub const fn is_json_whitespace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r')
}

/// Check if character starts a JSON number.
pub const fn is_json_number_start(c: u8) -> bool {
    c.is_ascii_digit() || c == b'-'
}

/// Check if character is valid in JSON number.
pub const fn is_json_number_char(c: u8) -> bool {
    c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-')
}

// =============================================================================
// JSON Number Parsing
// =============================================================================

/// JSON number parsing result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonNumberResult {
    /// Whether parsing succeeded
    pub ok: bool,
    /// Whether number is a float
    pub is_float: bool,
    /// Integer value (if !is_float)
    pub int_val: i64,
    /// Float value (if is_float)
    pub float_val: f64,
    /// Bytes consumed
    pub consumed: i32,
}

/// Parse a JSON number from input.
///
/// Returns (is_float, int_value, float_value, bytes_consumed).
pub fn parse_json_number(input: &[u8]) -> Option<(bool, i64, f64, usize)> {
    if input.is_empty() || !is_json_number_start(input[0]) {
        return None;
    }

    let mut i = 0;
    let mut is_float = false;

    // Optional negative sign
    if i < input.len() && input[i] == b'-' {
        i += 1;
    }

    // Integer part
    if i >= input.len() || !input[i].is_ascii_digit() {
        return None;
    }

    // Leading zero must be followed by . or end
    if input[i] == b'0' {
        i += 1;
    } else {
        while i < input.len() && input[i].is_ascii_digit() {
            i += 1;
        }
    }

    // Fractional part
    if i < input.len() && input[i] == b'.' {
        is_float = true;
        i += 1;
        if i >= input.len() || !input[i].is_ascii_digit() {
            return None;
        }
        while i < input.len() && input[i].is_ascii_digit() {
            i += 1;
        }
    }

    // Exponent part
    if i < input.len() && matches!(input[i], b'e' | b'E') {
        is_float = true;
        i += 1;
        if i < input.len() && matches!(input[i], b'+' | b'-') {
            i += 1;
        }
        if i >= input.len() || !input[i].is_ascii_digit() {
            return None;
        }
        while i < input.len() && input[i].is_ascii_digit() {
            i += 1;
        }
    }

    // Parse the number string
    let num_str = std::str::from_utf8(&input[..i]).ok()?;

    if is_float {
        let f: f64 = num_str.parse().ok()?;
        Some((true, 0, f, i))
    } else {
        let n: i64 = num_str.parse().ok()?;
        Some((false, n, 0.0, i))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_type() {
        assert!(JsonType::Array.is_container());
        assert!(JsonType::Object.is_container());
        assert!(JsonType::String.is_primitive());
        assert!(JsonType::Null.is_primitive());
    }

    #[test]
    fn test_json_token() {
        assert_eq!(JsonToken::from_c_int(2), JsonToken::LeftBrace);
        assert_eq!(JsonToken::from_c_int(8), JsonToken::String);
    }

    #[test]
    fn test_skip_whitespace() {
        assert_eq!(skip_json_whitespace(b"  hello"), b"hello");
        assert_eq!(skip_json_whitespace(b"\t\n\r test"), b"test");
        assert_eq!(skip_json_whitespace(b"no space"), b"no space");
    }

    #[test]
    fn test_parse_json_number() {
        // Integer
        let (is_float, int_val, _, consumed) = parse_json_number(b"123").unwrap();
        assert!(!is_float);
        assert_eq!(int_val, 123);
        assert_eq!(consumed, 3);

        // Negative
        let (is_float, int_val, _, consumed) = parse_json_number(b"-456").unwrap();
        assert!(!is_float);
        assert_eq!(int_val, -456);
        assert_eq!(consumed, 4);

        // Float
        let (is_float, _, float_val, consumed) = parse_json_number(b"2.5").unwrap();
        assert!(is_float);
        assert!((float_val - 2.5).abs() < 0.001);
        assert_eq!(consumed, 3);

        // Exponent
        let (is_float, _, float_val, consumed) = parse_json_number(b"1e10").unwrap();
        assert!(is_float);
        assert!((float_val - 1e10).abs() < 1.0);
        assert_eq!(consumed, 4);
    }
}
