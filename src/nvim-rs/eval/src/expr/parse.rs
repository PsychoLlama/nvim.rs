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
// Additional Token Constants (E1)
// =============================================================================

/// Method call separator (->).
pub const TOKEN_METHOD: c_int = 15;
/// Lambda arrow (=>).
pub const TOKEN_LAMBDA_ARROW: c_int = 16;
/// Ellipsis (...).
pub const TOKEN_ELLIPSIS: c_int = 17;
/// Question mark (for ternary).
pub const TOKEN_QUESTION: c_int = 18;
/// Double question mark (??).
pub const TOKEN_NULLISH: c_int = 19;

// =============================================================================
// Additional Character Classification Helpers (E1)
// =============================================================================

/// Check if character is octal digit.
fn is_octal(c: u8) -> bool {
    matches!(c, b'0'..=b'7')
}

/// Check if character is binary digit.
fn is_binary(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Check if character starts a literal number (including negative).
fn starts_literal_number(c: u8, next: u8) -> bool {
    is_digit(c) || (c == b'-' && is_digit(next))
}

/// Check if character is float exponent.
fn is_float_exp(c: u8) -> bool {
    c == b'e' || c == b'E'
}

/// Check if character is scope prefix (g:, l:, etc).
fn is_scope_prefix(c: u8) -> bool {
    matches!(c, b'g' | b'l' | b's' | b'a' | b'v' | b'b' | b'w' | b't')
}

/// Check if character can end an expression.
fn ends_expression(c: u8) -> bool {
    matches!(
        c,
        b'\0' | b'\n' | b'|' | b'"' | b'#' | b')' | b']' | b'}' | b',' | b':'
    )
}

/// Check if character is a subscript operator start.
fn is_subscript_start(c: u8) -> bool {
    c == b'[' || c == b'.'
}

/// Check if character is method call start.
fn is_method_start(c: u8, next: u8) -> bool {
    c == b'-' && next == b'>'
}

/// Skip whitespace returning the new offset.
fn skip_white_offset(s: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < s.len() && is_white(s[i]) {
        i += 1;
    }
    i
}

// =============================================================================
// Additional FFI Exports (E1)
// =============================================================================

/// FFI: Get TOKEN_METHOD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_method() -> c_int {
    TOKEN_METHOD
}

/// FFI: Get TOKEN_LAMBDA_ARROW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_lambda_arrow() -> c_int {
    TOKEN_LAMBDA_ARROW
}

/// FFI: Get TOKEN_ELLIPSIS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_ellipsis() -> c_int {
    TOKEN_ELLIPSIS
}

/// FFI: Get TOKEN_QUESTION constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_question() -> c_int {
    TOKEN_QUESTION
}

/// FFI: Get TOKEN_NULLISH constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_nullish() -> c_int {
    TOKEN_NULLISH
}

/// FFI: Get TOKEN_LPAREN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_lparen() -> c_int {
    TOKEN_LPAREN
}

/// FFI: Get TOKEN_RPAREN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_rparen() -> c_int {
    TOKEN_RPAREN
}

/// FFI: Get TOKEN_LBRACKET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_lbracket() -> c_int {
    TOKEN_LBRACKET
}

/// FFI: Get TOKEN_RBRACKET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_rbracket() -> c_int {
    TOKEN_RBRACKET
}

/// FFI: Get TOKEN_LBRACE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_lbrace() -> c_int {
    TOKEN_LBRACE
}

/// FFI: Get TOKEN_RBRACE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_rbrace() -> c_int {
    TOKEN_RBRACE
}

/// FFI: Get TOKEN_COMMA constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_comma() -> c_int {
    TOKEN_COMMA
}

/// FFI: Get TOKEN_COLON constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_colon() -> c_int {
    TOKEN_COLON
}

/// FFI: Get TOKEN_DOT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_dot() -> c_int {
    TOKEN_DOT
}

/// FFI: Get TOKEN_ARROW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_token_arrow() -> c_int {
    TOKEN_ARROW
}

/// FFI: Check if octal digit.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_octal_digit(c: c_int) -> c_int {
    c_int::from(is_octal(c as u8))
}

/// FFI: Check if binary digit.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_binary_digit(c: c_int) -> c_int {
    c_int::from(is_binary(c as u8))
}

/// FFI: Check if starts a literal number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_starts_literal_number(c: c_int, next: c_int) -> c_int {
    c_int::from(starts_literal_number(c as u8, next as u8))
}

/// FFI: Check if float exponent.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_float_exp(c: c_int) -> c_int {
    c_int::from(is_float_exp(c as u8))
}

/// FFI: Check if scope prefix.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_scope_prefix(c: c_int) -> c_int {
    c_int::from(is_scope_prefix(c as u8))
}

/// FFI: Check if ends expression.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_ends_expression(c: c_int) -> c_int {
    c_int::from(ends_expression(c as u8))
}

/// FFI: Check if subscript start.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_subscript_start(c: c_int) -> c_int {
    c_int::from(is_subscript_start(c as u8))
}

/// FFI: Check if method call start.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_is_method_start(c: c_int, next: c_int) -> c_int {
    c_int::from(is_method_start(c as u8, next as u8))
}

/// FFI: Skip whitespace in a string.
///
/// # Safety
/// `s` must point to a valid NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_parse_skip_white(s: *const u8) -> *const u8 {
    if s.is_null() {
        return s;
    }
    let mut p = s;
    while *p != 0 && is_white(*p) {
        p = p.add(1);
    }
    p
}

/// FFI: Skip whitespace and return offset.
///
/// # Safety
/// `s` must point to a valid string with at least `len` bytes.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_parse_skip_white_len(s: *const u8, len: c_int, start: c_int) -> c_int {
    if s.is_null() || len <= 0 || start < 0 {
        return start;
    }
    let slice = std::slice::from_raw_parts(s, len as usize);
    skip_white_offset(slice, start as usize) as c_int
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
