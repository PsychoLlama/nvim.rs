//! Expression executor for VimL.
//!
//! This module provides expression evaluation utilities migrated from `src/nvim/eval/executor.c`:
//! - Binary operators (arithmetic, comparison, logical, string concat)
//! - Unary operators (negation, not)
//! - Ternary conditional expressions
//! - Short-circuit evaluation
//! - Subscript and slice expressions
//!
//! ## VimL Expression Semantics
//!
//! VimL expressions follow these rules:
//! - Type coercion happens implicitly (strings to numbers, etc.)
//! - Comparison can be case-sensitive or case-insensitive
//! - String concatenation uses `.` operator
//! - Logical operators short-circuit
//!
//! ## FFI Pattern
//!
//! These functions work with typval types and return evaluation results.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]

use std::ffi::c_int;

// =============================================================================
// Binary Operators
// =============================================================================

/// Binary operator type for VimL expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BinaryOp {
    // Arithmetic
    /// Addition (+)
    Add = 0,
    /// Subtraction (-)
    Sub = 1,
    /// Multiplication (*)
    Mul = 2,
    /// Division (/)
    Div = 3,
    /// Modulo (%)
    Mod = 4,

    // Comparison
    /// Equal (==)
    Equal = 10,
    /// Not equal (!=)
    NotEqual = 11,
    /// Greater than (>)
    Greater = 12,
    /// Greater or equal (>=)
    GreaterEqual = 13,
    /// Less than (<)
    Less = 14,
    /// Less or equal (<=)
    LessEqual = 15,
    /// Equal (case-sensitive) (==#)
    EqualCase = 16,
    /// Not equal (case-sensitive) (!=#)
    NotEqualCase = 17,
    /// Equal (case-insensitive) (==?)
    EqualIgnore = 18,
    /// Not equal (case-insensitive) (!=?)
    NotEqualIgnore = 19,
    /// Regex match (=~)
    Match = 20,
    /// Regex not match (!~)
    NoMatch = 21,
    /// Regex match (case-sensitive) (=~#)
    MatchCase = 22,
    /// Regex not match (case-sensitive) (!~#)
    NoMatchCase = 23,
    /// Regex match (case-insensitive) (=~?)
    MatchIgnore = 24,
    /// Regex not match (case-insensitive) (!~?)
    NoMatchIgnore = 25,
    /// Same instance (is)
    Is = 26,
    /// Not same instance (isnot)
    IsNot = 27,

    // Logical
    /// Logical and (&&)
    And = 30,
    /// Logical or (||)
    Or = 31,

    // String
    /// String concatenation (.)
    Concat = 40,
    /// String concatenation with space (..)
    ConcatSpaced = 41,
}

impl BinaryOp {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Add),
            1 => Some(Self::Sub),
            2 => Some(Self::Mul),
            3 => Some(Self::Div),
            4 => Some(Self::Mod),
            10 => Some(Self::Equal),
            11 => Some(Self::NotEqual),
            12 => Some(Self::Greater),
            13 => Some(Self::GreaterEqual),
            14 => Some(Self::Less),
            15 => Some(Self::LessEqual),
            16 => Some(Self::EqualCase),
            17 => Some(Self::NotEqualCase),
            18 => Some(Self::EqualIgnore),
            19 => Some(Self::NotEqualIgnore),
            20 => Some(Self::Match),
            21 => Some(Self::NoMatch),
            22 => Some(Self::MatchCase),
            23 => Some(Self::NoMatchCase),
            24 => Some(Self::MatchIgnore),
            25 => Some(Self::NoMatchIgnore),
            26 => Some(Self::Is),
            27 => Some(Self::IsNot),
            30 => Some(Self::And),
            31 => Some(Self::Or),
            40 => Some(Self::Concat),
            41 => Some(Self::ConcatSpaced),
            _ => None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is an arithmetic operator.
    pub const fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod
        )
    }

    /// Check if this is a comparison operator.
    pub const fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Equal
                | Self::NotEqual
                | Self::Greater
                | Self::GreaterEqual
                | Self::Less
                | Self::LessEqual
                | Self::EqualCase
                | Self::NotEqualCase
                | Self::EqualIgnore
                | Self::NotEqualIgnore
                | Self::Match
                | Self::NoMatch
                | Self::MatchCase
                | Self::NoMatchCase
                | Self::MatchIgnore
                | Self::NoMatchIgnore
                | Self::Is
                | Self::IsNot
        )
    }

    /// Check if this is a logical operator.
    pub const fn is_logical(self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    /// Check if this is a string operator.
    pub const fn is_string(self) -> bool {
        matches!(self, Self::Concat | Self::ConcatSpaced)
    }

    /// Check if operator short-circuits.
    pub const fn short_circuits(self) -> bool {
        matches!(self, Self::And | Self::Or)
    }
}

// =============================================================================
// Unary Operators
// =============================================================================

/// Unary operator type for VimL expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum UnaryOp {
    /// Negation (-)
    Negate = 0,
    /// Logical not (!)
    Not = 1,
    /// Unary plus (+)
    Plus = 2,
}

impl UnaryOp {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Negate),
            1 => Some(Self::Not),
            2 => Some(Self::Plus),
            _ => None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Expression Type
// =============================================================================

/// Type of expression node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExprType {
    /// Literal value (number, string, etc.)
    Literal = 0,
    /// Variable reference
    Variable = 1,
    /// Binary operation
    Binary = 2,
    /// Unary operation
    Unary = 3,
    /// Ternary conditional (a ? b : c)
    Ternary = 4,
    /// Function call
    FuncCall = 5,
    /// Subscript access (a[b])
    Subscript = 6,
    /// Slice access (a[b:c])
    Slice = 7,
    /// Member access (a.b)
    Member = 8,
    /// Method call (a->b())
    MethodCall = 9,
    /// Lambda expression
    Lambda = 10,
    /// List literal [a, b, c]
    ListLiteral = 11,
    /// Dict literal {a: b, c: d}
    DictLiteral = 12,
}

impl ExprType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Literal),
            1 => Some(Self::Variable),
            2 => Some(Self::Binary),
            3 => Some(Self::Unary),
            4 => Some(Self::Ternary),
            5 => Some(Self::FuncCall),
            6 => Some(Self::Subscript),
            7 => Some(Self::Slice),
            8 => Some(Self::Member),
            9 => Some(Self::MethodCall),
            10 => Some(Self::Lambda),
            11 => Some(Self::ListLiteral),
            12 => Some(Self::DictLiteral),
            _ => None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Evaluation Result
// =============================================================================

/// Result of expression evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum EvalResult {
    /// Evaluation succeeded
    Ok = 0,
    /// Type error (incompatible types)
    TypeError = -1,
    /// Division by zero
    DivByZero = -2,
    /// Invalid operand
    InvalidOperand = -3,
    /// Variable not found
    VarNotFound = -4,
    /// Function not found
    FuncNotFound = -5,
    /// Index out of range
    IndexError = -6,
    /// Key not found
    KeyError = -7,
    /// Invalid regex pattern
    RegexError = -8,
    /// Recursion limit exceeded
    RecursionError = -9,
    /// Interrupt (Ctrl-C)
    Interrupted = -10,
    /// Generic failure
    Failure = -100,
}

impl EvalResult {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            -1 => Self::TypeError,
            -2 => Self::DivByZero,
            -3 => Self::InvalidOperand,
            -4 => Self::VarNotFound,
            -5 => Self::FuncNotFound,
            -6 => Self::IndexError,
            -7 => Self::KeyError,
            -8 => Self::RegexError,
            -9 => Self::RecursionError,
            -10 => Self::Interrupted,
            _ => Self::Failure,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if evaluation succeeded.
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if this is an error.
    pub const fn is_error(self) -> bool {
        !self.is_ok()
    }
}

// =============================================================================
// Comparison Case Mode
// =============================================================================

/// Case sensitivity mode for string comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CaseMode {
    /// Use 'ignorecase' option setting
    Default = 0,
    /// Case-sensitive comparison (#)
    Sensitive = 1,
    /// Case-insensitive comparison (?)
    Insensitive = 2,
}

impl CaseMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Default,
            1 => Self::Sensitive,
            2 => Self::Insensitive,
            _ => Self::Default,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Arithmetic Helpers
// =============================================================================

/// Perform integer addition with overflow check.
pub const fn checked_add(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}

/// Perform integer subtraction with overflow check.
pub const fn checked_sub(a: i64, b: i64) -> Option<i64> {
    a.checked_sub(b)
}

/// Perform integer multiplication with overflow check.
pub const fn checked_mul(a: i64, b: i64) -> Option<i64> {
    a.checked_mul(b)
}

/// Perform integer division with zero check.
pub const fn checked_div(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None
    } else {
        a.checked_div(b)
    }
}

/// Perform integer modulo with zero check.
pub const fn checked_mod(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None
    } else {
        a.checked_rem(b)
    }
}

/// FFI export: checked integer add.
///
/// # Safety
/// - `result` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_checked_add(a: i64, b: i64, result: *mut i64) -> bool {
    match checked_add(a, b) {
        Some(r) => {
            if !result.is_null() {
                // SAFETY: Caller guarantees result is valid if non-null
                unsafe { *result = r };
            }
            true
        }
        None => false,
    }
}

/// FFI export: checked integer sub.
///
/// # Safety
/// - `result` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_checked_sub(a: i64, b: i64, result: *mut i64) -> bool {
    match checked_sub(a, b) {
        Some(r) => {
            if !result.is_null() {
                // SAFETY: Caller guarantees result is valid if non-null
                unsafe { *result = r };
            }
            true
        }
        None => false,
    }
}

/// FFI export: checked integer mul.
///
/// # Safety
/// - `result` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_checked_mul(a: i64, b: i64, result: *mut i64) -> bool {
    match checked_mul(a, b) {
        Some(r) => {
            if !result.is_null() {
                // SAFETY: Caller guarantees result is valid if non-null
                unsafe { *result = r };
            }
            true
        }
        None => false,
    }
}

/// FFI export: checked integer div.
///
/// # Safety
/// - `result` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_checked_div(a: i64, b: i64, result: *mut i64) -> bool {
    match checked_div(a, b) {
        Some(r) => {
            if !result.is_null() {
                // SAFETY: Caller guarantees result is valid if non-null
                unsafe { *result = r };
            }
            true
        }
        None => false,
    }
}

/// FFI export: checked integer mod.
///
/// # Safety
/// - `result` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_checked_mod(a: i64, b: i64, result: *mut i64) -> bool {
    match checked_mod(a, b) {
        Some(r) => {
            if !result.is_null() {
                // SAFETY: Caller guarantees result is valid if non-null
                unsafe { *result = r };
            }
            true
        }
        None => false,
    }
}

// =============================================================================
// Float Arithmetic Helpers
// =============================================================================

/// Perform float division with special handling.
///
/// VimL float division returns inf for division by zero (not error).
pub fn float_div(a: f64, b: f64) -> f64 {
    a / b // Rust/IEEE handles inf correctly
}

/// Perform float modulo.
pub fn float_mod(a: f64, b: f64) -> f64 {
    a % b // Returns NaN for b == 0
}

/// FFI export: float division.
#[no_mangle]
pub extern "C" fn rs_eval_float_div(a: f64, b: f64) -> f64 {
    float_div(a, b)
}

/// FFI export: float modulo.
#[no_mangle]
pub extern "C" fn rs_eval_float_mod(a: f64, b: f64) -> f64 {
    float_mod(a, b)
}

// =============================================================================
// Logical Evaluation
// =============================================================================

/// Convert VimL value to boolean for logical operations.
///
/// VimL truthiness rules:
/// - 0 is false
/// - Non-zero numbers are true
/// - Empty string is false
/// - Non-empty string is true
/// - Empty list/dict is false
/// - v:false, v:null are false
/// - v:true is true
pub const fn is_truthy_number(n: i64) -> bool {
    n != 0
}

/// Convert float to boolean.
pub fn is_truthy_float(f: f64) -> bool {
    f != 0.0 && !f.is_nan()
}

/// FFI export: check if number is truthy.
#[no_mangle]
pub extern "C" fn rs_eval_is_truthy_number(n: i64) -> bool {
    is_truthy_number(n)
}

/// FFI export: check if float is truthy.
#[no_mangle]
pub extern "C" fn rs_eval_is_truthy_float(f: f64) -> bool {
    is_truthy_float(f)
}

// =============================================================================
// Comparison Helpers
// =============================================================================

/// Compare two numbers.
///
/// Returns:
/// - -1 if a < b
/// - 0 if a == b
/// - 1 if a > b
pub const fn compare_numbers(a: i64, b: i64) -> c_int {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// Compare two floats.
///
/// Returns:
/// - -1 if a < b
/// - 0 if a == b
/// - 1 if a > b
/// - 0 if either is NaN (VimL semantics)
pub fn compare_floats(a: f64, b: f64) -> c_int {
    if a.is_nan() || b.is_nan() {
        0 // NaN comparisons in VimL return "equal"
    } else if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// FFI export: compare numbers.
#[no_mangle]
pub extern "C" fn rs_eval_compare_numbers(a: i64, b: i64) -> c_int {
    compare_numbers(a, b)
}

/// FFI export: compare floats.
#[no_mangle]
pub extern "C" fn rs_eval_compare_floats(a: f64, b: f64) -> c_int {
    compare_floats(a, b)
}

// =============================================================================
// String to Number Conversion
// =============================================================================

/// Convert string to number (VimL semantics).
///
/// VimL conversion rules:
/// - Leading whitespace is skipped
/// - Leading sign (+ or -) is recognized
/// - Hex (0x), octal (0o), binary (0b) prefixes are recognized
/// - Stops at first non-digit character
/// - Returns 0 for empty or non-numeric strings
pub fn string_to_number(s: &[u8]) -> i64 {
    let s = s.iter().copied();
    let mut s = s.skip_while(|&c| c == b' ' || c == b'\t');

    // Check for sign
    let negative = match s.clone().next() {
        Some(b'-') => {
            s.next();
            true
        }
        Some(b'+') => {
            s.next();
            false
        }
        _ => false,
    };

    // Collect remaining bytes
    let bytes: Vec<u8> = s.collect();
    let mut idx = 0;

    // Check for base prefix
    let (base, skip) = if bytes.len() >= 2 && bytes[0] == b'0' {
        match bytes.get(1) {
            Some(b'x' | b'X') => (16, 2),
            Some(b'o' | b'O') => (8, 2),
            Some(b'b' | b'B') => (2, 2),
            _ => (10, 0),
        }
    } else {
        (10, 0)
    };

    idx += skip;

    // Parse digits
    let mut result: i64 = 0;
    while idx < bytes.len() {
        let digit = match bytes[idx] {
            b'0'..=b'9' => bytes[idx] - b'0',
            b'a'..=b'f' if base == 16 => bytes[idx] - b'a' + 10,
            b'A'..=b'F' if base == 16 => bytes[idx] - b'A' + 10,
            _ => break,
        };

        if i64::from(digit) >= base {
            break;
        }

        result = result.saturating_mul(base).saturating_add(i64::from(digit));
        idx += 1;
    }

    if negative {
        result.saturating_neg()
    } else {
        result
    }
}

/// FFI export: convert string to number.
///
/// # Safety
/// - `s` must be a valid pointer to at least `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_string_to_number(s: *const u8, len: c_int) -> i64 {
    if s.is_null() || len < 0 {
        return 0;
    }

    // SAFETY: Caller guarantees s points to at least len bytes
    let slice = unsafe { std::slice::from_raw_parts(s, len as usize) };
    string_to_number(slice)
}

// =============================================================================
// String to Float Conversion
// =============================================================================

/// Convert string to float (VimL semantics).
pub fn string_to_float(s: &[u8]) -> f64 {
    // Skip leading whitespace
    let s: Vec<u8> = s
        .iter()
        .copied()
        .skip_while(|&c| c == b' ' || c == b'\t')
        .collect();

    // Try to parse as float
    if let Ok(string) = std::str::from_utf8(&s) {
        // Find end of float (stop at non-float chars)
        let end = string
            .find(|c: char| {
                !c.is_ascii_digit() && c != '.' && c != 'e' && c != 'E' && c != '+' && c != '-'
            })
            .unwrap_or(string.len());

        string[..end].parse::<f64>().unwrap_or(0.0)
    } else {
        0.0
    }
}

/// FFI export: convert string to float.
///
/// # Safety
/// - `s` must be a valid pointer to at least `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_string_to_float(s: *const u8, len: c_int) -> f64 {
    if s.is_null() || len < 0 {
        return 0.0;
    }

    // SAFETY: Caller guarantees s points to at least len bytes
    let slice = unsafe { std::slice::from_raw_parts(s, len as usize) };
    string_to_float(slice)
}

// =============================================================================
// Index/Subscript Helpers
// =============================================================================

/// Subscript range for slice operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscriptRange {
    /// Start index (inclusive)
    pub start: i64,
    /// End index (inclusive, VimL semantics)
    pub end: i64,
    /// Whether start was omitted ([:end])
    pub start_omitted: bool,
    /// Whether end was omitted ([start:])
    pub end_omitted: bool,
}

impl SubscriptRange {
    /// Create a new range.
    pub const fn new(start: i64, end: i64) -> Self {
        Self {
            start,
            end,
            start_omitted: false,
            end_omitted: false,
        }
    }

    /// Create a range with start omitted.
    pub const fn from_start_omitted(end: i64) -> Self {
        Self {
            start: 0,
            end,
            start_omitted: true,
            end_omitted: false,
        }
    }

    /// Create a range with end omitted.
    pub const fn to_end_omitted(start: i64) -> Self {
        Self {
            start,
            end: -1, // Will be resolved to len-1
            start_omitted: false,
            end_omitted: true,
        }
    }

    /// Create a full range ([:]).
    pub const fn full() -> Self {
        Self {
            start: 0,
            end: -1,
            start_omitted: true,
            end_omitted: true,
        }
    }
}

// =============================================================================
// Expression Context Flags
// =============================================================================

/// Flags for expression evaluation context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalFlags(u32);

impl EvalFlags {
    /// No special flags.
    pub const NONE: Self = Self(0);

    /// Allow range expressions.
    pub const ALLOW_RANGE: Self = Self(1 << 0);

    /// Evaluating for assignment target.
    pub const LVALUE: Self = Self(1 << 1);

    /// In a :def function.
    pub const DEF_FUNCTION: Self = Self(1 << 2);

    /// Skip side effects.
    pub const NO_SIDE_EFFECTS: Self = Self(1 << 3);

    /// Create new flags.
    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Check if flag is set.
    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    /// Get raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }
}

// =============================================================================
// Tests
// =============================================================================

// =============================================================================
// Expression Evaluator Module
// =============================================================================

pub mod context;
pub mod errors;
pub mod eval;
pub mod index;
pub mod lval;
pub mod operators;

// Re-export FFI functions from the context module
pub use context::rs_set_context_for_expression;

// Re-export FFI functions from the eval module
pub use eval::{
    rs_eval0, rs_eval1, rs_eval2, rs_eval3, rs_eval4, rs_eval5, rs_eval_dict, rs_eval_lit_dict,
    rs_eval_lit_string, rs_eval_method, rs_eval_string, EvalargHandle, ExargHandle, TypevalHandle,
};

// Re-export FFI functions from the index module
pub use index::{rs_check_can_index, rs_eval_index, rs_eval_index_inner, rs_f_slice};

// Re-export FFI functions from the operators module
pub use operators::{
    rs_apply_comparison, rs_compare_floats, rs_compare_numbers, rs_typval_compare, CompareResult,
};

// Re-export error types
pub use errors::{
    rs_eval_emsg_silent, rs_eval_error_number, rs_eval_has_emsg, rs_eval_is_aborting,
    rs_eval_is_function_error, rs_eval_is_lock_error, rs_eval_is_type_error, ErrorState, EvalError,
    EvalErrorCode, EvalOpResult,
};

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_op() {
        assert!(BinaryOp::Add.is_arithmetic());
        assert!(BinaryOp::Equal.is_comparison());
        assert!(BinaryOp::And.is_logical());
        assert!(BinaryOp::And.short_circuits());
        assert!(BinaryOp::Concat.is_string());

        assert_eq!(BinaryOp::from_c_int(0), Some(BinaryOp::Add));
        assert_eq!(BinaryOp::from_c_int(99), None);
    }

    #[test]
    fn test_unary_op() {
        assert_eq!(UnaryOp::from_c_int(0), Some(UnaryOp::Negate));
        assert_eq!(UnaryOp::from_c_int(1), Some(UnaryOp::Not));
        assert_eq!(UnaryOp::from_c_int(99), None);
    }

    #[test]
    fn test_eval_result() {
        assert!(EvalResult::Ok.is_ok());
        assert!(!EvalResult::TypeError.is_ok());
        assert!(EvalResult::DivByZero.is_error());
    }

    #[test]
    fn test_checked_arithmetic() {
        assert_eq!(checked_add(1, 2), Some(3));
        assert_eq!(checked_sub(5, 3), Some(2));
        assert_eq!(checked_mul(3, 4), Some(12));
        assert_eq!(checked_div(10, 2), Some(5));
        assert_eq!(checked_div(10, 0), None);
        assert_eq!(checked_mod(10, 3), Some(1));
        assert_eq!(checked_mod(10, 0), None);

        // Overflow
        assert_eq!(checked_add(i64::MAX, 1), None);
        assert_eq!(checked_mul(i64::MAX, 2), None);
    }

    #[test]
    fn test_float_ops() {
        assert!(float_div(1.0, 0.0).is_infinite());
        assert!(float_mod(1.0, 0.0).is_nan());
    }

    #[test]
    fn test_truthy() {
        assert!(!is_truthy_number(0));
        assert!(is_truthy_number(1));
        assert!(is_truthy_number(-1));

        assert!(!is_truthy_float(0.0));
        assert!(is_truthy_float(1.0));
        assert!(!is_truthy_float(f64::NAN));
    }

    #[test]
    fn test_compare_numbers() {
        assert_eq!(compare_numbers(1, 2), -1);
        assert_eq!(compare_numbers(2, 2), 0);
        assert_eq!(compare_numbers(3, 2), 1);
    }

    #[test]
    fn test_compare_floats() {
        assert_eq!(compare_floats(1.0, 2.0), -1);
        assert_eq!(compare_floats(2.0, 2.0), 0);
        assert_eq!(compare_floats(3.0, 2.0), 1);
        assert_eq!(compare_floats(f64::NAN, 1.0), 0);
    }

    #[test]
    fn test_string_to_number() {
        assert_eq!(string_to_number(b"42"), 42);
        assert_eq!(string_to_number(b"-42"), -42);
        assert_eq!(string_to_number(b"  123"), 123);
        assert_eq!(string_to_number(b"0x10"), 16);
        assert_eq!(string_to_number(b"0o10"), 8);
        assert_eq!(string_to_number(b"0b10"), 2);
        assert_eq!(string_to_number(b"12abc"), 12);
        assert_eq!(string_to_number(b""), 0);
        assert_eq!(string_to_number(b"abc"), 0);
    }

    #[test]
    fn test_string_to_float() {
        assert!((string_to_float(b"2.5") - 2.5).abs() < f64::EPSILON);
        assert!((string_to_float(b"  1.5") - 1.5).abs() < f64::EPSILON);
        assert!((string_to_float(b"1e10") - 1e10).abs() < 1.0);
        assert!((string_to_float(b"abc") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_subscript_range() {
        let r = SubscriptRange::new(0, 5);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 5);
        assert!(!r.start_omitted);
        assert!(!r.end_omitted);

        let full = SubscriptRange::full();
        assert!(full.start_omitted);
        assert!(full.end_omitted);
    }

    #[test]
    fn test_eval_flags() {
        let flags = EvalFlags::NONE;
        assert!(!flags.contains(EvalFlags::ALLOW_RANGE));

        let flags = EvalFlags::new(EvalFlags::ALLOW_RANGE.bits() | EvalFlags::LVALUE.bits());
        assert!(flags.contains(EvalFlags::ALLOW_RANGE));
        assert!(flags.contains(EvalFlags::LVALUE));
        assert!(!flags.contains(EvalFlags::DEF_FUNCTION));
    }

    #[test]
    fn test_case_mode() {
        assert_eq!(CaseMode::from_c_int(0), CaseMode::Default);
        assert_eq!(CaseMode::from_c_int(1), CaseMode::Sensitive);
        assert_eq!(CaseMode::from_c_int(2), CaseMode::Insensitive);
    }

    #[test]
    fn test_expr_type() {
        assert_eq!(ExprType::from_c_int(0), Some(ExprType::Literal));
        assert_eq!(ExprType::from_c_int(5), Some(ExprType::FuncCall));
        assert_eq!(ExprType::from_c_int(99), None);
    }
}
