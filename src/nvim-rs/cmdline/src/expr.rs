//! Expression input handling for command-line mode
//!
//! This module provides utilities for handling expression input in command-line
//! mode (the = prompt), including expression parsing and validation.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Expression Input Type
// =============================================================================

/// The expression prompt character.
pub const EXPR_PROMPT_CHAR: u8 = b'=';

/// Check if a firstc is an expression prompt.
#[must_use]
pub const fn is_expr_firstc(firstc: u8) -> bool {
    firstc == EXPR_PROMPT_CHAR
}

// =============================================================================
// Expression Validation
// =============================================================================

/// Basic bracket matching result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketBalance {
    /// Brackets are balanced.
    Balanced,
    /// More open brackets than close.
    Unclosed,
    /// More close brackets than open.
    ExtraClose,
}

/// Check bracket balance in an expression.
///
/// Returns the balance state for (), [], and {}.
#[must_use]
pub fn check_bracket_balance(expr: &[u8]) -> BracketBalance {
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut in_string = false;
    let mut string_char: u8 = 0;

    let mut i = 0;
    while i < expr.len() {
        let c = expr[i];

        // Handle string literals
        if in_string {
            if c == string_char && (i == 0 || expr[i - 1] != b'\\') {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match c {
            b'"' | b'\'' => {
                in_string = true;
                string_char = c;
            }
            b'(' => paren_depth += 1,
            b')' => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    return BracketBalance::ExtraClose;
                }
            }
            b'[' => bracket_depth += 1,
            b']' => {
                bracket_depth -= 1;
                if bracket_depth < 0 {
                    return BracketBalance::ExtraClose;
                }
            }
            b'{' => brace_depth += 1,
            b'}' => {
                brace_depth -= 1;
                if brace_depth < 0 {
                    return BracketBalance::ExtraClose;
                }
            }
            _ => {}
        }

        i += 1;
    }

    if paren_depth != 0 || bracket_depth != 0 || brace_depth != 0 {
        BracketBalance::Unclosed
    } else {
        BracketBalance::Balanced
    }
}

/// Check if an expression looks complete (basic heuristic).
///
/// An expression is likely complete if:
/// - Brackets are balanced
/// - Not ending with an operator
#[must_use]
pub fn is_likely_complete(expr: &[u8]) -> bool {
    if expr.is_empty() {
        return true;
    }

    // Check bracket balance
    if check_bracket_balance(expr) != BracketBalance::Balanced {
        return false;
    }

    // Check for trailing operators
    let last = expr[expr.len() - 1];
    !matches!(
        last,
        b'+' | b'-' | b'*' | b'/' | b'%' | b'.' | b',' | b'=' | b'<' | b'>' | b'&' | b'|' | b'!'
    )
}

// =============================================================================
// Expression Parsing Utilities
// =============================================================================

/// Find the start of the last expression token.
///
/// Used for completion - finds where the current identifier/keyword starts.
#[must_use]
pub fn find_last_token_start(expr: &[u8]) -> usize {
    if expr.is_empty() {
        return 0;
    }

    let mut i = expr.len();
    while i > 0 {
        i -= 1;
        let c = expr[i];

        // Stop at operators or whitespace
        if c.is_ascii_whitespace()
            || matches!(
                c,
                b'+' | b'-'
                    | b'*'
                    | b'/'
                    | b'%'
                    | b'.'
                    | b','
                    | b'='
                    | b'<'
                    | b'>'
                    | b'&'
                    | b'|'
                    | b'!'
                    | b'('
                    | b')'
                    | b'['
                    | b']'
                    | b'{'
                    | b'}'
                    | b':'
                    | b'"'
                    | b'\''
            )
        {
            return i + 1;
        }
    }

    0
}

/// Extract the last token from an expression.
#[must_use]
pub fn get_last_token(expr: &[u8]) -> &[u8] {
    let start = find_last_token_start(expr);
    &expr[start..]
}

/// Check if a character can be part of an identifier.
#[must_use]
pub const fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Check if a character can start an identifier.
#[must_use]
pub const fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

// =============================================================================
// VimL Expression Types
// =============================================================================

/// Types of VimL expression tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprTokenType {
    /// Number literal.
    Number,
    /// String literal.
    String,
    /// Identifier (variable, function name).
    Identifier,
    /// Operator.
    Operator,
    /// Punctuation (brackets, etc.).
    Punctuation,
    /// Unknown/other.
    Unknown,
}

/// Detect the type of token at a position.
#[must_use]
pub fn detect_token_type(expr: &[u8], pos: usize) -> ExprTokenType {
    if pos >= expr.len() {
        return ExprTokenType::Unknown;
    }

    let c = expr[pos];

    if c.is_ascii_digit() || (c == b'-' && pos + 1 < expr.len() && expr[pos + 1].is_ascii_digit()) {
        return ExprTokenType::Number;
    }

    if c == b'"' || c == b'\'' {
        return ExprTokenType::String;
    }

    if is_ident_start(c) {
        return ExprTokenType::Identifier;
    }

    if matches!(
        c,
        b'+' | b'-' | b'*' | b'/' | b'%' | b'.' | b'=' | b'<' | b'>' | b'&' | b'|' | b'!'
    ) {
        return ExprTokenType::Operator;
    }

    if matches!(c, b'(' | b')' | b'[' | b']' | b'{' | b'}' | b',' | b':') {
        return ExprTokenType::Punctuation;
    }

    ExprTokenType::Unknown
}

// =============================================================================
// Input Function Support
// =============================================================================

/// Check if command line is in input() function mode.
///
/// The firstc for input() is typically '@' or a custom character.
#[must_use]
pub const fn is_input_func_prompt(firstc: u8) -> bool {
    // '@' is used for debug mode and input()
    firstc == b'@'
}

/// Check if the current prompt is for a confirm dialog.
///
/// Confirm dialogs have limited valid responses.
#[must_use]
pub const fn is_confirm_prompt(firstc: u8) -> bool {
    // Confirm dialogs typically use special firstc values
    firstc == b'@'
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Check if a firstc is an expression prompt (FFI).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_expr_firstc(firstc: c_int) -> c_int {
    if !(0..=255).contains(&firstc) {
        return 0;
    }
    c_int::from(is_expr_firstc(firstc as u8))
}

/// Check bracket balance in an expression (FFI).
///
/// Returns 0 for balanced, 1 for unclosed, -1 for extra close.
///
/// # Safety
///
/// `expr` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_check_bracket_balance(expr: *const c_char, len: usize) -> c_int {
    if expr.is_null() || len == 0 {
        return 0; // Empty is balanced
    }

    let bytes = std::slice::from_raw_parts(expr.cast::<u8>(), len);
    match check_bracket_balance(bytes) {
        BracketBalance::Balanced => 0,
        BracketBalance::Unclosed => 1,
        BracketBalance::ExtraClose => -1,
    }
}

/// Check if an expression looks complete (FFI).
///
/// # Safety
///
/// `expr` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_expr_likely_complete(expr: *const c_char, len: usize) -> c_int {
    if expr.is_null() {
        return 1; // Empty is complete
    }

    let bytes = std::slice::from_raw_parts(expr.cast::<u8>(), len);
    c_int::from(is_likely_complete(bytes))
}

/// Find the start of the last token in an expression (FFI).
///
/// # Safety
///
/// `expr` must be a valid pointer to a string of at least `len` bytes.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_find_last_token_start(expr: *const c_char, len: usize) -> c_int {
    if expr.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(expr.cast::<u8>(), len);
    find_last_token_start(bytes) as c_int
}

/// Check if a character is an identifier character (FFI).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_expr_ident_char(c: c_int) -> c_int {
    if !(0..=255).contains(&c) {
        return 0;
    }
    c_int::from(is_ident_char(c as u8))
}

/// Check if a firstc is an input() function prompt (FFI).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_input_func_prompt(firstc: c_int) -> c_int {
    if !(0..=255).contains(&firstc) {
        return 0;
    }
    c_int::from(is_input_func_prompt(firstc as u8))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_expr_firstc() {
        assert!(is_expr_firstc(b'='));
        assert!(!is_expr_firstc(b':'));
        assert!(!is_expr_firstc(b'/'));
    }

    #[test]
    fn test_bracket_balance() {
        assert_eq!(check_bracket_balance(b"()"), BracketBalance::Balanced);
        assert_eq!(check_bracket_balance(b"[]{}"), BracketBalance::Balanced);
        assert_eq!(check_bracket_balance(b"(a + b)"), BracketBalance::Balanced);
        assert_eq!(
            check_bracket_balance(b"(a + (b * c))"),
            BracketBalance::Balanced
        );

        assert_eq!(check_bracket_balance(b"("), BracketBalance::Unclosed);
        assert_eq!(check_bracket_balance(b"(("), BracketBalance::Unclosed);
        // (] has mismatched brackets - ] closes before [ was opened
        assert_eq!(check_bracket_balance(b"(]"), BracketBalance::ExtraClose);

        assert_eq!(check_bracket_balance(b")"), BracketBalance::ExtraClose);
        assert_eq!(check_bracket_balance(b"())"), BracketBalance::ExtraClose);

        // String literals
        assert_eq!(check_bracket_balance(b"\"(\""), BracketBalance::Balanced);
        assert_eq!(check_bracket_balance(b"')'"), BracketBalance::Balanced);
    }

    #[test]
    fn test_is_likely_complete() {
        assert!(is_likely_complete(b""));
        assert!(is_likely_complete(b"a + b"));
        assert!(is_likely_complete(b"foo(x)"));
        assert!(is_likely_complete(b"[1, 2, 3]"));

        assert!(!is_likely_complete(b"a +"));
        assert!(!is_likely_complete(b"foo("));
        assert!(!is_likely_complete(b"a ."));
    }

    #[test]
    fn test_find_last_token_start() {
        assert_eq!(find_last_token_start(b"foo"), 0);
        assert_eq!(find_last_token_start(b"foo + bar"), 6);
        assert_eq!(find_last_token_start(b"foo.bar"), 4);
        assert_eq!(find_last_token_start(b"foo(bar)"), 8);
        assert_eq!(find_last_token_start(b""), 0);
    }

    #[test]
    fn test_get_last_token() {
        assert_eq!(get_last_token(b"foo"), b"foo");
        assert_eq!(get_last_token(b"foo + bar"), b"bar");
        assert_eq!(get_last_token(b"foo.bar"), b"bar");
        assert_eq!(get_last_token(b""), b"");
    }

    #[test]
    fn test_is_ident_char() {
        assert!(is_ident_char(b'a'));
        assert!(is_ident_char(b'Z'));
        assert!(is_ident_char(b'0'));
        assert!(is_ident_char(b'_'));
        assert!(!is_ident_char(b' '));
        assert!(!is_ident_char(b'+'));
    }

    #[test]
    fn test_detect_token_type() {
        assert_eq!(detect_token_type(b"123", 0), ExprTokenType::Number);
        assert_eq!(detect_token_type(b"\"foo\"", 0), ExprTokenType::String);
        assert_eq!(detect_token_type(b"foo", 0), ExprTokenType::Identifier);
        assert_eq!(detect_token_type(b"a + b", 2), ExprTokenType::Operator);
        assert_eq!(detect_token_type(b"()", 0), ExprTokenType::Punctuation);
    }

    #[test]
    fn test_is_input_func_prompt() {
        assert!(is_input_func_prompt(b'@'));
        assert!(!is_input_func_prompt(b'='));
        assert!(!is_input_func_prompt(b':'));
    }
}
