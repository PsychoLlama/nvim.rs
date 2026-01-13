//! Expression parsing helpers.
//!
//! This module provides helpers for parsing VimL expressions:
//! eval0, eval1-eval9 (expression levels), skipwhite

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Parser Token Types
// =============================================================================

/// End of expression.
pub const TOKEN_END: c_int = 0;
/// Number literal.
pub const TOKEN_NUMBER: c_int = 1;
/// String literal.
pub const TOKEN_STRING: c_int = 2;
/// Identifier (variable/function name).
pub const TOKEN_IDENT: c_int = 3;
/// Operator.
pub const TOKEN_OPERATOR: c_int = 4;
/// Open parenthesis.
pub const TOKEN_LPAREN: c_int = 5;
/// Close parenthesis.
pub const TOKEN_RPAREN: c_int = 6;
/// Open bracket.
pub const TOKEN_LBRACKET: c_int = 7;
/// Close bracket.
pub const TOKEN_RBRACKET: c_int = 8;
/// Open brace.
pub const TOKEN_LBRACE: c_int = 9;
/// Close brace.
pub const TOKEN_RBRACE: c_int = 10;
/// Comma.
pub const TOKEN_COMMA: c_int = 11;
/// Colon.
pub const TOKEN_COLON: c_int = 12;
/// Dot.
pub const TOKEN_DOT: c_int = 13;
/// Arrow (->).
pub const TOKEN_ARROW: c_int = 14;

// =============================================================================
// Character Classification Helpers
// =============================================================================

/// Check if character is whitespace.
fn is_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if character is digit.
fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Check if character is hex digit.
fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Check if character is octal digit.
#[allow(dead_code)]
fn is_octal_digit(c: u8) -> bool {
    matches!(c, b'0'..=b'7')
}

/// Check if character is binary digit.
#[allow(dead_code)]
fn is_binary_digit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Check if character can start identifier.
fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

/// Check if character can be in identifier.
fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Check if character is string quote.
fn is_string_quote(c: u8) -> bool {
    c == b'"' || c == b'\''
}

// =============================================================================
// Number Parsing Helpers
// =============================================================================

/// Detect number base from prefix.
fn detect_number_base(first: u8, second: u8) -> c_int {
    if first == b'0' {
        if second == b'x' || second == b'X' {
            16 // hex
        } else if second == b'o' || second == b'O' {
            8 // octal
        } else if second == b'b' || second == b'B' {
            2 // binary
        } else {
            10 // decimal (0...)
        }
    } else {
        10 // decimal
    }
}

/// Check if string starts with a number.
fn starts_with_number(c: u8) -> bool {
    is_digit(c)
}

/// Check if string starts with a float (digit or dot-digit).
#[allow(dead_code)]
fn could_be_float(first: u8, second: u8) -> bool {
    is_digit(first) || (first == b'.' && is_digit(second))
}

// =============================================================================
// Operator Detection
// =============================================================================

/// Check if character is comparison operator start.
fn is_compare_op_start(c: u8) -> bool {
    matches!(c, b'=' | b'!' | b'<' | b'>' | b'~' | b'#')
}

/// Check if character is arithmetic operator.
fn is_arith_op(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'*' | b'/' | b'%')
}

/// Check if character is logical operator start.
fn is_logical_op_start(c: u8) -> bool {
    matches!(c, b'&' | b'|')
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get TOKEN_END constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_end() -> c_int {
    TOKEN_END
}

/// FFI: Get TOKEN_NUMBER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_number() -> c_int {
    TOKEN_NUMBER
}

/// FFI: Get TOKEN_STRING constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_string() -> c_int {
    TOKEN_STRING
}

/// FFI: Get TOKEN_IDENT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_ident() -> c_int {
    TOKEN_IDENT
}

/// FFI: Get TOKEN_OPERATOR constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_operator() -> c_int {
    TOKEN_OPERATOR
}

/// FFI: Check if whitespace.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_white(c: c_int) -> c_int {
    c_int::from(is_white(c as u8))
}

/// FFI: Check if digit.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_digit(c: c_int) -> c_int {
    c_int::from(is_digit(c as u8))
}

/// FFI: Check if hex digit.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_hex_digit(c: c_int) -> c_int {
    c_int::from(is_hex_digit(c as u8))
}

/// FFI: Check if identifier start.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_ident_start(c: c_int) -> c_int {
    c_int::from(is_ident_start(c as u8))
}

/// FFI: Check if identifier character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_ident_char(c: c_int) -> c_int {
    c_int::from(is_ident_char(c as u8))
}

/// FFI: Check if string quote.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_string_quote(c: c_int) -> c_int {
    c_int::from(is_string_quote(c as u8))
}

/// FFI: Detect number base.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_detect_number_base(first: c_int, second: c_int) -> c_int {
    detect_number_base(first as u8, second as u8)
}

/// FFI: Check if starts with number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_starts_with_number(c: c_int) -> c_int {
    c_int::from(starts_with_number(c as u8))
}

/// FFI: Check if comparison operator start.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_compare_op(c: c_int) -> c_int {
    c_int::from(is_compare_op_start(c as u8))
}

/// FFI: Check if arithmetic operator.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_arith_op(c: c_int) -> c_int {
    c_int::from(is_arith_op(c as u8))
}

/// FFI: Check if logical operator start.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_logical_op(c: c_int) -> c_int {
    c_int::from(is_logical_op_start(c as u8))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_constants() {
        assert_eq!(TOKEN_END, 0);
        assert_eq!(TOKEN_NUMBER, 1);
        assert_eq!(TOKEN_STRING, 2);
        assert_eq!(TOKEN_IDENT, 3);
    }

    #[test]
    fn test_is_white() {
        assert!(is_white(b' '));
        assert!(is_white(b'\t'));
        assert!(!is_white(b'a'));
        assert!(!is_white(b'\n'));
    }

    #[test]
    fn test_is_digit() {
        assert!(is_digit(b'0'));
        assert!(is_digit(b'9'));
        assert!(!is_digit(b'a'));
    }

    #[test]
    fn test_is_hex_digit() {
        assert!(is_hex_digit(b'0'));
        assert!(is_hex_digit(b'a'));
        assert!(is_hex_digit(b'F'));
        assert!(!is_hex_digit(b'g'));
    }

    #[test]
    fn test_is_ident() {
        assert!(is_ident_start(b'a'));
        assert!(is_ident_start(b'_'));
        assert!(!is_ident_start(b'0'));

        assert!(is_ident_char(b'a'));
        assert!(is_ident_char(b'0'));
        assert!(is_ident_char(b'_'));
        assert!(!is_ident_char(b'-'));
    }

    #[test]
    fn test_detect_number_base() {
        assert_eq!(detect_number_base(b'0', b'x'), 16);
        assert_eq!(detect_number_base(b'0', b'X'), 16);
        assert_eq!(detect_number_base(b'0', b'o'), 8);
        assert_eq!(detect_number_base(b'0', b'b'), 2);
        assert_eq!(detect_number_base(b'0', b'0'), 10);
        assert_eq!(detect_number_base(b'1', b'2'), 10);
    }

    #[test]
    fn test_operator_detection() {
        assert!(is_compare_op_start(b'='));
        assert!(is_compare_op_start(b'<'));
        assert!(is_compare_op_start(b'>'));
        assert!(!is_compare_op_start(b'+'));

        assert!(is_arith_op(b'+'));
        assert!(is_arith_op(b'-'));
        assert!(is_arith_op(b'*'));
        assert!(is_arith_op(b'/'));
        assert!(!is_arith_op(b'='));

        assert!(is_logical_op_start(b'&'));
        assert!(is_logical_op_start(b'|'));
        assert!(!is_logical_op_start(b'+'));
    }

    #[test]
    fn test_could_be_float() {
        assert!(could_be_float(b'1', b'.'));
        assert!(could_be_float(b'.', b'5'));
        assert!(!could_be_float(b'.', b'.'));
        assert!(!could_be_float(b'a', b'b'));
    }

    #[test]
    fn test_binary_octal_digits() {
        assert!(is_binary_digit(b'0'));
        assert!(is_binary_digit(b'1'));
        assert!(!is_binary_digit(b'2'));

        assert!(is_octal_digit(b'0'));
        assert!(is_octal_digit(b'7'));
        assert!(!is_octal_digit(b'8'));
    }
}
