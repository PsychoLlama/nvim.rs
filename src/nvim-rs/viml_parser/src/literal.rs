//! VimL literal parsing utilities.
//!
//! This module provides utilities for parsing VimL numeric and string literals.

use std::ffi::c_int;

// =============================================================================
// Number Parsing
// =============================================================================

/// Base for number literals.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberBase {
    /// Decimal (base 10)
    #[default]
    Decimal = 10,
    /// Hexadecimal (base 16, 0x prefix)
    Hex = 16,
    /// Octal (base 8, 0o or leading 0 prefix)
    Octal = 8,
    /// Binary (base 2, 0b prefix)
    Binary = 2,
}

impl NumberBase {
    /// Get the radix value.
    #[must_use]
    pub const fn radix(self) -> u32 {
        self as u32
    }

    /// Check if a digit is valid for this base.
    #[must_use]
    pub const fn is_valid_digit(self, c: u8) -> bool {
        match self {
            Self::Decimal => c.is_ascii_digit(),
            Self::Hex => c.is_ascii_hexdigit(),
            Self::Octal => c >= b'0' && c <= b'7',
            Self::Binary => c == b'0' || c == b'1',
        }
    }

    /// Get the prefix string for this base.
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Decimal => "",
            Self::Hex => "0x",
            Self::Octal => "0o",
            Self::Binary => "0b",
        }
    }
}

/// Result of parsing a number literal.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParsedNumber {
    /// The parsed integer value (for integer literals)
    pub int_value: i64,
    /// The parsed float value (for float literals)
    pub float_value: f64,
    /// Number of bytes consumed
    pub len: u32,
    /// The base (radix) of the number
    pub base: NumberBase,
    /// Whether this is a float
    pub is_float: bool,
    /// Whether parsing succeeded
    pub valid: bool,
}

impl Default for ParsedNumber {
    fn default() -> Self {
        Self {
            int_value: 0,
            float_value: 0.0,
            len: 0,
            base: NumberBase::Decimal,
            is_float: false,
            valid: false,
        }
    }
}

impl ParsedNumber {
    /// Create a new invalid result.
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            int_value: 0,
            float_value: 0.0,
            len: 0,
            base: NumberBase::Decimal,
            is_float: false,
            valid: false,
        }
    }

    /// Create a new integer result.
    #[must_use]
    pub const fn int(value: i64, len: u32, base: NumberBase) -> Self {
        Self {
            int_value: value,
            float_value: value as f64,
            len,
            base,
            is_float: false,
            valid: true,
        }
    }

    /// Create a new float result.
    #[must_use]
    pub const fn float(value: f64, len: u32) -> Self {
        Self {
            int_value: value as i64,
            float_value: value,
            len,
            base: NumberBase::Decimal,
            is_float: true,
            valid: true,
        }
    }
}

/// Detect the base of a number literal from its prefix.
///
/// Returns (base, prefix_len).
#[must_use]
pub fn detect_number_base(input: &[u8]) -> (NumberBase, usize) {
    if input.len() < 2 {
        return (NumberBase::Decimal, 0);
    }

    if input[0] != b'0' {
        return (NumberBase::Decimal, 0);
    }

    match input[1] {
        b'x' | b'X' => (NumberBase::Hex, 2),
        b'o' | b'O' => (NumberBase::Octal, 2),
        b'b' | b'B' => (NumberBase::Binary, 2),
        c if c >= b'0' && c <= b'7' => {
            // Legacy octal: 0777
            (NumberBase::Octal, 0)
        }
        _ => (NumberBase::Decimal, 0),
    }
}

/// Parse an integer from input bytes.
///
/// Handles decimal, hex (0x), octal (0o or leading 0), and binary (0b).
#[must_use]
pub fn parse_integer(input: &[u8]) -> ParsedNumber {
    if input.is_empty() {
        return ParsedNumber::invalid();
    }

    // Handle negative sign
    let (is_negative, start) = if input[0] == b'-' {
        (true, 1)
    } else {
        (false, 0)
    };

    if start >= input.len() {
        return ParsedNumber::invalid();
    }

    let rest = &input[start..];
    let (base, prefix_len) = detect_number_base(rest);

    let digits_start = prefix_len;
    if digits_start >= rest.len() {
        return ParsedNumber::invalid();
    }

    // Count digits
    let mut pos = digits_start;
    while pos < rest.len() && base.is_valid_digit(rest[pos]) {
        pos += 1;
    }

    if pos == digits_start {
        // No digits found
        return ParsedNumber::invalid();
    }

    // Parse the value
    let digits = &rest[digits_start..pos];
    let mut value: i64 = 0;
    let radix = base.radix() as i64;

    for &d in digits {
        let digit = match d {
            b'0'..=b'9' => (d - b'0') as i64,
            b'a'..=b'f' => (d - b'a' + 10) as i64,
            b'A'..=b'F' => (d - b'A' + 10) as i64,
            _ => return ParsedNumber::invalid(),
        };
        value = value.saturating_mul(radix).saturating_add(digit);
    }

    if is_negative {
        value = -value;
    }

    let total_len = start + pos;
    ParsedNumber::int(value, total_len as u32, base)
}

/// Parse a float from input bytes.
///
/// Handles standard float format: digits.digits[eE][+-]digits
#[must_use]
pub fn parse_float(input: &[u8]) -> ParsedNumber {
    if input.is_empty() {
        return ParsedNumber::invalid();
    }

    // Try to parse as string and use std float parsing
    let mut end = 0;

    // Handle optional sign
    if end < input.len() && (input[end] == b'-' || input[end] == b'+') {
        end += 1;
    }

    // Integer part
    while end < input.len() && input[end].is_ascii_digit() {
        end += 1;
    }

    // Need at least one digit
    if end == 0 || (end == 1 && (input[0] == b'-' || input[0] == b'+')) {
        return ParsedNumber::invalid();
    }

    // Check for decimal point or exponent
    let mut has_decimal = false;
    let mut has_exponent = false;

    if end < input.len() && input[end] == b'.' {
        has_decimal = true;
        end += 1;

        // Fractional part
        while end < input.len() && input[end].is_ascii_digit() {
            end += 1;
        }
    }

    if end < input.len() && (input[end] == b'e' || input[end] == b'E') {
        has_exponent = true;
        end += 1;

        // Optional exponent sign
        if end < input.len() && (input[end] == b'-' || input[end] == b'+') {
            end += 1;
        }

        // Exponent digits
        let exp_start = end;
        while end < input.len() && input[end].is_ascii_digit() {
            end += 1;
        }

        // Need at least one exponent digit
        if end == exp_start {
            return ParsedNumber::invalid();
        }
    }

    // Must have decimal point or exponent to be a float
    if !has_decimal && !has_exponent {
        return ParsedNumber::invalid();
    }

    // Convert to string and parse
    let s = match std::str::from_utf8(&input[..end]) {
        Ok(s) => s,
        Err(_) => return ParsedNumber::invalid(),
    };

    match s.parse::<f64>() {
        Ok(v) => ParsedNumber::float(v, end as u32),
        Err(_) => ParsedNumber::invalid(),
    }
}

/// Try to parse a number (integer or float).
#[must_use]
pub fn parse_number(input: &[u8]) -> ParsedNumber {
    // First try float (if it has decimal point or exponent)
    let float_result = parse_float(input);
    if float_result.valid && float_result.is_float {
        return float_result;
    }

    // Otherwise try integer
    parse_integer(input)
}

// =============================================================================
// String Escape Handling
// =============================================================================

/// String escape sequence types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeKind {
    /// Invalid escape sequence
    Invalid = 0,
    /// Single character escape (\n, \t, etc.)
    Char = 1,
    /// Octal escape (\nnn)
    Octal = 2,
    /// Hex escape (\xNN)
    Hex = 3,
    /// Unicode escape (\uNNNN)
    Unicode = 4,
    /// Unicode long escape (\UNNNNNNNN)
    UnicodeLong = 5,
    /// Special key escape (<CR>, <Tab>, etc.)
    Special = 6,
}

/// Result of parsing an escape sequence.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParsedEscape {
    /// The unescaped character value
    pub value: u32,
    /// Number of bytes consumed (including backslash)
    pub len: u32,
    /// Type of escape sequence
    pub kind: EscapeKind,
    /// Whether parsing succeeded
    pub valid: bool,
}

impl Default for ParsedEscape {
    fn default() -> Self {
        Self {
            value: 0,
            len: 0,
            kind: EscapeKind::Invalid,
            valid: false,
        }
    }
}

impl ParsedEscape {
    /// Create an invalid result.
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            value: 0,
            len: 0,
            kind: EscapeKind::Invalid,
            valid: false,
        }
    }

    /// Create a valid result.
    #[must_use]
    pub const fn new(value: u32, len: u32, kind: EscapeKind) -> Self {
        Self {
            value,
            len,
            kind,
            valid: true,
        }
    }
}

/// Parse a single-character escape sequence.
///
/// Input should start AFTER the backslash.
#[must_use]
pub fn parse_char_escape(c: u8) -> Option<u8> {
    match c {
        b'n' => Some(b'\n'),
        b'r' => Some(b'\r'),
        b't' => Some(b'\t'),
        b'b' => Some(0x08), // backspace
        b'e' => Some(0x1b), // escape
        b'f' => Some(0x0c), // form feed
        b'\\' => Some(b'\\'),
        b'"' => Some(b'"'),
        b'\'' => Some(b'\''),
        b'<' => Some(b'<'),
        _ => None,
    }
}

/// Parse an escape sequence from input.
///
/// Input should start AFTER the backslash.
#[must_use]
pub fn parse_escape(input: &[u8]) -> ParsedEscape {
    if input.is_empty() {
        return ParsedEscape::invalid();
    }

    let c = input[0];

    // Single character escape
    if let Some(value) = parse_char_escape(c) {
        return ParsedEscape::new(value as u32, 2, EscapeKind::Char);
    }

    // Octal escape \nnn (up to 3 digits)
    if c >= b'0' && c <= b'7' {
        let mut value: u32 = (c - b'0') as u32;
        let mut len = 1;

        while len < 3 && len < input.len() {
            let d = input[len];
            if d >= b'0' && d <= b'7' {
                value = value * 8 + (d - b'0') as u32;
                len += 1;
            } else {
                break;
            }
        }

        return ParsedEscape::new(value, (len + 1) as u32, EscapeKind::Octal);
    }

    // Hex escape \xNN
    if c == b'x' && input.len() >= 3 {
        let d1 = input[1];
        let d2 = input[2];
        if d1.is_ascii_hexdigit() && d2.is_ascii_hexdigit() {
            let value = hex_digit(d1) * 16 + hex_digit(d2);
            return ParsedEscape::new(value, 4, EscapeKind::Hex);
        }
    }

    // Unicode escape \uNNNN
    if c == b'u' && input.len() >= 5 {
        if let Some(value) = parse_hex_digits(&input[1..5]) {
            return ParsedEscape::new(value, 6, EscapeKind::Unicode);
        }
    }

    // Unicode long escape \UNNNNNNNN
    if c == b'U' && input.len() >= 9 {
        if let Some(value) = parse_hex_digits(&input[1..9]) {
            return ParsedEscape::new(value, 10, EscapeKind::UnicodeLong);
        }
    }

    // Unknown escape - return the character as-is
    ParsedEscape::new(c as u32, 2, EscapeKind::Char)
}

/// Convert hex digit to value.
#[must_use]
const fn hex_digit(c: u8) -> u32 {
    match c {
        b'0'..=b'9' => (c - b'0') as u32,
        b'a'..=b'f' => (c - b'a' + 10) as u32,
        b'A'..=b'F' => (c - b'A' + 10) as u32,
        _ => 0,
    }
}

/// Parse a sequence of hex digits.
fn parse_hex_digits(input: &[u8]) -> Option<u32> {
    let mut value: u32 = 0;
    for &c in input {
        if !c.is_ascii_hexdigit() {
            return None;
        }
        value = value * 16 + hex_digit(c);
    }
    Some(value)
}

// =============================================================================
// Blob Literal Parsing
// =============================================================================

/// Check if input starts with blob prefix (0z).
#[must_use]
pub fn is_blob_prefix(input: &[u8]) -> bool {
    input.len() >= 2 && input[0] == b'0' && (input[1] == b'z' || input[1] == b'Z')
}

/// Parse blob literal bytes.
///
/// Returns (bytes, consumed_len) or None on error.
/// Input should start AFTER the 0z prefix.
#[must_use]
pub fn parse_blob_content(input: &[u8]) -> Option<(Vec<u8>, usize)> {
    let mut result = Vec::new();
    let mut pos = 0;

    while pos + 1 < input.len() {
        let d1 = input[pos];
        let d2 = input[pos + 1];

        // Skip dots between byte pairs
        if d1 == b'.' {
            pos += 1;
            continue;
        }

        if !d1.is_ascii_hexdigit() || !d2.is_ascii_hexdigit() {
            break;
        }

        let byte = (hex_digit(d1) * 16 + hex_digit(d2)) as u8;
        result.push(byte);
        pos += 2;
    }

    Some((result, pos))
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Detect number base from input.
///
/// # Safety
/// `input` must be valid for `input_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_detect_number_base(
    input: *const u8,
    input_len: usize,
    prefix_len_out: *mut usize,
) -> c_int {
    if input.is_null() || prefix_len_out.is_null() {
        return NumberBase::Decimal as c_int;
    }

    let slice = std::slice::from_raw_parts(input, input_len);
    let (base, prefix_len) = detect_number_base(slice);
    *prefix_len_out = prefix_len;
    base as c_int
}

/// Parse integer from input.
///
/// # Safety
/// `input` must be valid for `input_len` bytes.
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_integer(
    input: *const u8,
    input_len: usize,
    result: *mut ParsedNumber,
) -> c_int {
    if input.is_null() || result.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(input, input_len);
    *result = parse_integer(slice);
    c_int::from((*result).valid)
}

/// Parse number (int or float) from input.
///
/// # Safety
/// `input` must be valid for `input_len` bytes.
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_number(
    input: *const u8,
    input_len: usize,
    result: *mut ParsedNumber,
) -> c_int {
    if input.is_null() || result.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(input, input_len);
    *result = parse_number(slice);
    c_int::from((*result).valid)
}

/// Parse escape sequence.
///
/// # Safety
/// `input` must be valid for `input_len` bytes (should start after backslash).
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_escape(
    input: *const u8,
    input_len: usize,
    result: *mut ParsedEscape,
) -> c_int {
    if input.is_null() || result.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(input, input_len);
    *result = parse_escape(slice);
    c_int::from((*result).valid)
}

/// Check if input starts with blob prefix.
///
/// # Safety
/// `input` must be valid for `input_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_is_blob_prefix(input: *const u8, input_len: usize) -> c_int {
    if input.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(input, input_len);
    c_int::from(is_blob_prefix(slice))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_base() {
        assert_eq!(NumberBase::Decimal.radix(), 10);
        assert_eq!(NumberBase::Hex.radix(), 16);
        assert_eq!(NumberBase::Octal.radix(), 8);
        assert_eq!(NumberBase::Binary.radix(), 2);

        assert!(NumberBase::Decimal.is_valid_digit(b'5'));
        assert!(!NumberBase::Decimal.is_valid_digit(b'a'));

        assert!(NumberBase::Hex.is_valid_digit(b'a'));
        assert!(NumberBase::Hex.is_valid_digit(b'F'));
        assert!(!NumberBase::Hex.is_valid_digit(b'g'));

        assert!(NumberBase::Octal.is_valid_digit(b'7'));
        assert!(!NumberBase::Octal.is_valid_digit(b'8'));

        assert!(NumberBase::Binary.is_valid_digit(b'0'));
        assert!(NumberBase::Binary.is_valid_digit(b'1'));
        assert!(!NumberBase::Binary.is_valid_digit(b'2'));
    }

    #[test]
    fn test_detect_number_base() {
        assert_eq!(detect_number_base(b"123"), (NumberBase::Decimal, 0));
        assert_eq!(detect_number_base(b"0x1f"), (NumberBase::Hex, 2));
        assert_eq!(detect_number_base(b"0X1F"), (NumberBase::Hex, 2));
        assert_eq!(detect_number_base(b"0o17"), (NumberBase::Octal, 2));
        assert_eq!(detect_number_base(b"0O17"), (NumberBase::Octal, 2));
        assert_eq!(detect_number_base(b"0b10"), (NumberBase::Binary, 2));
        assert_eq!(detect_number_base(b"0B10"), (NumberBase::Binary, 2));
        // Legacy octal
        assert_eq!(detect_number_base(b"0777"), (NumberBase::Octal, 0));
    }

    #[test]
    fn test_parse_integer_decimal() {
        let result = parse_integer(b"123");
        assert!(result.valid);
        assert_eq!(result.int_value, 123);
        assert_eq!(result.len, 3);
        assert_eq!(result.base, NumberBase::Decimal);
        assert!(!result.is_float);

        let result = parse_integer(b"-456");
        assert!(result.valid);
        assert_eq!(result.int_value, -456);
        assert_eq!(result.len, 4);
    }

    #[test]
    fn test_parse_integer_hex() {
        let result = parse_integer(b"0xff");
        assert!(result.valid);
        assert_eq!(result.int_value, 255);
        assert_eq!(result.len, 4);
        assert_eq!(result.base, NumberBase::Hex);

        let result = parse_integer(b"0X1A2B");
        assert!(result.valid);
        assert_eq!(result.int_value, 0x1A2B);
    }

    #[test]
    fn test_parse_integer_octal() {
        let result = parse_integer(b"0o17");
        assert!(result.valid);
        assert_eq!(result.int_value, 15);
        assert_eq!(result.base, NumberBase::Octal);
    }

    #[test]
    fn test_parse_integer_binary() {
        let result = parse_integer(b"0b1010");
        assert!(result.valid);
        assert_eq!(result.int_value, 10);
        assert_eq!(result.base, NumberBase::Binary);
    }

    #[test]
    fn test_parse_float() {
        let result = parse_float(b"3.14");
        assert!(result.valid);
        assert!(result.is_float);
        assert!((result.float_value - 3.14).abs() < 0.001);
        assert_eq!(result.len, 4);

        let result = parse_float(b"2.5e10");
        assert!(result.valid);
        assert!(result.is_float);
        assert!((result.float_value - 2.5e10).abs() < 1e5);

        let result = parse_float(b"-1.5E-3");
        assert!(result.valid);
        assert!((result.float_value - (-0.0015)).abs() < 0.0001);
    }

    #[test]
    fn test_parse_number() {
        // Integer
        let result = parse_number(b"42");
        assert!(result.valid);
        assert!(!result.is_float);
        assert_eq!(result.int_value, 42);

        // Float
        let result = parse_number(b"3.14");
        assert!(result.valid);
        assert!(result.is_float);

        // Hex
        let result = parse_number(b"0xff");
        assert!(result.valid);
        assert!(!result.is_float);
        assert_eq!(result.int_value, 255);
    }

    #[test]
    fn test_parse_char_escape() {
        assert_eq!(parse_char_escape(b'n'), Some(b'\n'));
        assert_eq!(parse_char_escape(b'r'), Some(b'\r'));
        assert_eq!(parse_char_escape(b't'), Some(b'\t'));
        assert_eq!(parse_char_escape(b'\\'), Some(b'\\'));
        assert_eq!(parse_char_escape(b'"'), Some(b'"'));
        assert_eq!(parse_char_escape(b'z'), None);
    }

    #[test]
    fn test_parse_escape() {
        // Single char
        let result = parse_escape(b"n");
        assert!(result.valid);
        assert_eq!(result.value, b'\n' as u32);
        assert_eq!(result.len, 2);
        assert_eq!(result.kind, EscapeKind::Char);

        // Octal
        let result = parse_escape(b"101");
        assert!(result.valid);
        assert_eq!(result.value, 65); // 'A'
        assert_eq!(result.kind, EscapeKind::Octal);

        // Hex
        let result = parse_escape(b"x41");
        assert!(result.valid);
        assert_eq!(result.value, 65); // 'A'
        assert_eq!(result.kind, EscapeKind::Hex);

        // Unicode
        let result = parse_escape(b"u0041");
        assert!(result.valid);
        assert_eq!(result.value, 65); // 'A'
        assert_eq!(result.kind, EscapeKind::Unicode);
    }

    #[test]
    fn test_is_blob_prefix() {
        assert!(is_blob_prefix(b"0z"));
        assert!(is_blob_prefix(b"0Z"));
        assert!(is_blob_prefix(b"0zabc"));
        assert!(!is_blob_prefix(b"0x"));
        assert!(!is_blob_prefix(b"z0"));
        assert!(!is_blob_prefix(b"0"));
    }

    #[test]
    fn test_parse_blob_content() {
        let (bytes, len) = parse_blob_content(b"00112233").unwrap();
        assert_eq!(bytes, vec![0x00, 0x11, 0x22, 0x33]);
        assert_eq!(len, 8);

        let (bytes, len) = parse_blob_content(b"AB.CD").unwrap();
        assert_eq!(bytes, vec![0xAB, 0xCD]);
        assert_eq!(len, 5);
    }
}
