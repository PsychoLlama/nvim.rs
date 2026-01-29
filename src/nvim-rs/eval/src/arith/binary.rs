//! Binary arithmetic operations.
//!
//! This module provides helpers for binary arithmetic:
//! eval_addlist/subtractlist, tv_add/subtract/multiply/divide

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

// =============================================================================
// Binary Operator Constants
// =============================================================================

/// Addition operator (+).
pub const BINOP_ADD: c_int = 0;
/// Subtraction operator (-).
pub const BINOP_SUB: c_int = 1;
/// Multiplication operator (*).
pub const BINOP_MUL: c_int = 2;
/// Division operator (/).
pub const BINOP_DIV: c_int = 3;
/// Modulo operator (%).
pub const BINOP_MOD: c_int = 4;
/// String concatenation operator (.).
pub const BINOP_CONCAT: c_int = 5;
/// List concatenation operator (+).
pub const BINOP_LIST_CONCAT: c_int = 6;

// =============================================================================
// Arithmetic Error Constants
// =============================================================================

/// No error.
pub const ARITH_OK: c_int = 0;
/// Division by zero.
pub const ARITH_DIV_ZERO: c_int = 1;
/// Overflow.
pub const ARITH_OVERFLOW: c_int = 2;
/// Type mismatch.
pub const ARITH_TYPE_ERROR: c_int = 3;

// =============================================================================
// Binary Operation Helpers
// =============================================================================

/// Safe integer addition with overflow check.
fn safe_add(a: i64, b: i64) -> (i64, bool) {
    a.overflowing_add(b)
}

/// Safe integer subtraction with overflow check.
fn safe_sub(a: i64, b: i64) -> (i64, bool) {
    a.overflowing_sub(b)
}

/// Safe integer multiplication with overflow check.
fn safe_mul(a: i64, b: i64) -> (i64, bool) {
    a.overflowing_mul(b)
}

/// Safe integer division (returns error code if divide by zero).
fn safe_div(a: i64, b: i64) -> (i64, c_int) {
    if b == 0 {
        (0, ARITH_DIV_ZERO)
    } else {
        (a / b, ARITH_OK)
    }
}

/// Safe integer modulo (returns error code if divide by zero).
fn safe_mod(a: i64, b: i64) -> (i64, c_int) {
    if b == 0 {
        (0, ARITH_DIV_ZERO)
    } else {
        (a % b, ARITH_OK)
    }
}

/// Get operator name for error messages.
#[allow(dead_code)]
fn binop_name(op: c_int) -> &'static str {
    match op {
        BINOP_ADD | BINOP_LIST_CONCAT => "+",
        BINOP_SUB => "-",
        BINOP_MUL => "*",
        BINOP_DIV => "/",
        BINOP_MOD => "%",
        BINOP_CONCAT => ".",
        _ => "?",
    }
}

/// Check if operator is arithmetic (number-based).
fn is_arithmetic_op(op: c_int) -> bool {
    matches!(
        op,
        BINOP_ADD | BINOP_SUB | BINOP_MUL | BINOP_DIV | BINOP_MOD
    )
}

/// Check if operator produces integer result.
fn produces_integer(op: c_int) -> bool {
    matches!(op, BINOP_DIV | BINOP_MOD)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get BINOP_ADD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_binop_add() -> c_int {
    BINOP_ADD
}

/// FFI: Get BINOP_SUB constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_binop_sub() -> c_int {
    BINOP_SUB
}

/// FFI: Get BINOP_MUL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_binop_mul() -> c_int {
    BINOP_MUL
}

/// FFI: Get BINOP_DIV constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_binop_div() -> c_int {
    BINOP_DIV
}

/// FFI: Get BINOP_MOD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_binop_mod() -> c_int {
    BINOP_MOD
}

/// FFI: Get BINOP_CONCAT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_binop_concat() -> c_int {
    BINOP_CONCAT
}

/// FFI: Get ARITH_OK constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_arith_ok() -> c_int {
    ARITH_OK
}

/// FFI: Get ARITH_DIV_ZERO constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_arith_div_zero() -> c_int {
    ARITH_DIV_ZERO
}

/// FFI: Get ARITH_OVERFLOW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_arith_overflow() -> c_int {
    ARITH_OVERFLOW
}

/// FFI: Safe integer addition.
#[unsafe(no_mangle)]
pub extern "C" fn rs_safe_add(a: i64, b: i64, overflow: *mut c_int) -> i64 {
    let (result, did_overflow) = safe_add(a, b);
    if !overflow.is_null() {
        unsafe {
            *overflow = c_int::from(did_overflow);
        }
    }
    result
}

/// FFI: Safe integer subtraction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_safe_sub(a: i64, b: i64, overflow: *mut c_int) -> i64 {
    let (result, did_overflow) = safe_sub(a, b);
    if !overflow.is_null() {
        unsafe {
            *overflow = c_int::from(did_overflow);
        }
    }
    result
}

/// FFI: Safe integer multiplication.
#[unsafe(no_mangle)]
pub extern "C" fn rs_safe_mul(a: i64, b: i64, overflow: *mut c_int) -> i64 {
    let (result, did_overflow) = safe_mul(a, b);
    if !overflow.is_null() {
        unsafe {
            *overflow = c_int::from(did_overflow);
        }
    }
    result
}

/// FFI: Safe integer division.
#[unsafe(no_mangle)]
pub extern "C" fn rs_safe_div(a: i64, b: i64, error: *mut c_int) -> i64 {
    let (result, err) = safe_div(a, b);
    if !error.is_null() {
        unsafe {
            *error = err;
        }
    }
    result
}

/// FFI: Safe integer modulo.
#[unsafe(no_mangle)]
pub extern "C" fn rs_safe_mod(a: i64, b: i64, error: *mut c_int) -> i64 {
    let (result, err) = safe_mod(a, b);
    if !error.is_null() {
        unsafe {
            *error = err;
        }
    }
    result
}

/// FFI: Check if arithmetic operator.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_arithmetic_op(op: c_int) -> c_int {
    c_int::from(is_arithmetic_op(op))
}

/// FFI: Check if operator produces integer.
#[unsafe(no_mangle)]
pub extern "C" fn rs_produces_integer(op: c_int) -> c_int {
    c_int::from(produces_integer(op))
}

// =============================================================================
// Additional FFI Exports (E2)
// =============================================================================

/// FFI: Get ARITH_TYPE_ERROR constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_arith_type_error() -> c_int {
    ARITH_TYPE_ERROR
}

/// FFI: Get BINOP_LIST_CONCAT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_binop_list_concat() -> c_int {
    BINOP_LIST_CONCAT
}

/// FFI: Float addition.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_add(a: f64, b: f64) -> f64 {
    a + b
}

/// FFI: Float subtraction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_sub(a: f64, b: f64) -> f64 {
    a - b
}

/// FFI: Float multiplication.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_mul(a: f64, b: f64) -> f64 {
    a * b
}

/// FFI: Float division.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_div(a: f64, b: f64) -> f64 {
    a / b
}

/// FFI: Float modulo (fmod).
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_mod(a: f64, b: f64) -> f64 {
    a % b
}

/// FFI: Check if float is NaN.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_is_nan(f: f64) -> c_int {
    c_int::from(f.is_nan())
}

/// FFI: Check if float is infinite.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_is_infinite(f: f64) -> c_int {
    c_int::from(f.is_infinite())
}

/// FFI: Check if float is finite.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_is_finite(f: f64) -> c_int {
    c_int::from(f.is_finite())
}

/// FFI: Convert integer to float.
#[unsafe(no_mangle)]
#[allow(clippy::cast_precision_loss)]
pub extern "C" fn rs_int_to_float(i: i64) -> f64 {
    i as f64
}

/// FFI: Convert float to integer (truncating).
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_float_to_int(f: f64) -> i64 {
    f as i64
}

/// FFI: Apply binary operator to integers.
#[unsafe(no_mangle)]
pub extern "C" fn rs_apply_binop_int(op: c_int, a: i64, b: i64, error: *mut c_int) -> i64 {
    let (result, err) = match op {
        BINOP_ADD => {
            let (r, overflow) = safe_add(a, b);
            (r, if overflow { ARITH_OVERFLOW } else { ARITH_OK })
        }
        BINOP_SUB => {
            let (r, overflow) = safe_sub(a, b);
            (r, if overflow { ARITH_OVERFLOW } else { ARITH_OK })
        }
        BINOP_MUL => {
            let (r, overflow) = safe_mul(a, b);
            (r, if overflow { ARITH_OVERFLOW } else { ARITH_OK })
        }
        BINOP_DIV => safe_div(a, b),
        BINOP_MOD => safe_mod(a, b),
        _ => (0, ARITH_TYPE_ERROR),
    };
    if !error.is_null() {
        unsafe {
            *error = err;
        }
    }
    result
}

/// FFI: Apply binary operator to floats.
#[unsafe(no_mangle)]
pub extern "C" fn rs_apply_binop_float(op: c_int, a: f64, b: f64) -> f64 {
    match op {
        BINOP_ADD => a + b,
        BINOP_SUB => a - b,
        BINOP_MUL => a * b,
        BINOP_DIV => a / b,
        BINOP_MOD => a % b,
        _ => f64::NAN,
    }
}

/// FFI: Compute power (a ** b) for floats.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_pow(a: f64, b: f64) -> f64 {
    a.powf(b)
}

/// FFI: Integer power (a ** b) for integers.
#[unsafe(no_mangle)]
pub extern "C" fn rs_int_pow(base: i64, exp: u32) -> i64 {
    base.saturating_pow(exp)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binop_constants() {
        assert_eq!(BINOP_ADD, 0);
        assert_eq!(BINOP_SUB, 1);
        assert_eq!(BINOP_MUL, 2);
        assert_eq!(BINOP_DIV, 3);
        assert_eq!(BINOP_MOD, 4);
    }

    #[test]
    fn test_safe_add() {
        let (result, overflow) = safe_add(1, 2);
        assert_eq!(result, 3);
        assert!(!overflow);

        let (_, overflow) = safe_add(i64::MAX, 1);
        assert!(overflow);
    }

    #[test]
    fn test_safe_sub() {
        let (result, overflow) = safe_sub(5, 3);
        assert_eq!(result, 2);
        assert!(!overflow);

        let (_, overflow) = safe_sub(i64::MIN, 1);
        assert!(overflow);
    }

    #[test]
    fn test_safe_mul() {
        let (result, overflow) = safe_mul(3, 4);
        assert_eq!(result, 12);
        assert!(!overflow);

        let (_, overflow) = safe_mul(i64::MAX, 2);
        assert!(overflow);
    }

    #[test]
    fn test_safe_div() {
        let (result, error) = safe_div(10, 3);
        assert_eq!(result, 3);
        assert_eq!(error, ARITH_OK);

        let (_, error) = safe_div(10, 0);
        assert_eq!(error, ARITH_DIV_ZERO);
    }

    #[test]
    fn test_safe_mod() {
        let (result, error) = safe_mod(10, 3);
        assert_eq!(result, 1);
        assert_eq!(error, ARITH_OK);

        let (_, error) = safe_mod(10, 0);
        assert_eq!(error, ARITH_DIV_ZERO);
    }

    #[test]
    fn test_is_arithmetic_op() {
        assert!(is_arithmetic_op(BINOP_ADD));
        assert!(is_arithmetic_op(BINOP_DIV));
        assert!(!is_arithmetic_op(BINOP_CONCAT));
    }

    #[test]
    fn test_produces_integer() {
        assert!(produces_integer(BINOP_DIV));
        assert!(produces_integer(BINOP_MOD));
        assert!(!produces_integer(BINOP_ADD));
    }

    #[test]
    fn test_binop_name() {
        assert_eq!(binop_name(BINOP_ADD), "+");
        assert_eq!(binop_name(BINOP_MUL), "*");
        assert_eq!(binop_name(BINOP_CONCAT), ".");
    }
}
