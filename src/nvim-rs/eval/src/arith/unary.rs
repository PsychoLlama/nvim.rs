//! Unary arithmetic operations.
//!
//! This module provides helpers for unary arithmetic:
//! tv_negate, tv_logical_not, tv_modulo, float_* ops

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Unary Operator Constants
// =============================================================================

/// Unary negation (-x).
pub const UNOP_NEGATE: c_int = 0;
/// Unary plus (+x).
pub const UNOP_PLUS: c_int = 1;
/// Logical not (!x).
pub const UNOP_NOT: c_int = 2;

// =============================================================================
// Float Operation Constants
// =============================================================================

/// Float absolute value.
pub const FLOAT_ABS: c_int = 0;
/// Float ceiling.
pub const FLOAT_CEIL: c_int = 1;
/// Float floor.
pub const FLOAT_FLOOR: c_int = 2;
/// Float round.
pub const FLOAT_ROUND: c_int = 3;
/// Float truncate.
pub const FLOAT_TRUNC: c_int = 4;
/// Float square root.
pub const FLOAT_SQRT: c_int = 5;
/// Float sine.
pub const FLOAT_SIN: c_int = 6;
/// Float cosine.
pub const FLOAT_COS: c_int = 7;
/// Float tangent.
pub const FLOAT_TAN: c_int = 8;
/// Float natural log.
pub const FLOAT_LOG: c_int = 9;
/// Float base-10 log.
pub const FLOAT_LOG10: c_int = 10;
/// Float exponential.
pub const FLOAT_EXP: c_int = 11;

// =============================================================================
// Unary Operation Helpers
// =============================================================================

/// Negate an integer.
fn negate_int(val: i64) -> i64 {
    val.wrapping_neg()
}

/// Apply logical not to integer (0 -> 1, non-0 -> 0).
fn logical_not_int(val: i64) -> i64 {
    i64::from(val == 0)
}

/// Negate a float.
fn negate_float(val: f64) -> f64 {
    -val
}

/// Apply logical not to float (0.0 -> 1, non-0 -> 0).
fn logical_not_float(val: f64) -> i64 {
    i64::from(val == 0.0)
}

/// Convert integer to boolean (0 -> false, non-0 -> true).
fn int_to_bool(val: i64) -> bool {
    val != 0
}

/// Convert boolean to integer (false -> 0, true -> 1).
fn bool_to_int(val: bool) -> i64 {
    i64::from(val)
}

// =============================================================================
// Float Operation Helpers
// =============================================================================

/// Apply float operation.
fn apply_float_op(op: c_int, val: f64) -> f64 {
    match op {
        FLOAT_ABS => val.abs(),
        FLOAT_CEIL => val.ceil(),
        FLOAT_FLOOR => val.floor(),
        FLOAT_ROUND => val.round(),
        FLOAT_TRUNC => val.trunc(),
        FLOAT_SQRT => val.sqrt(),
        FLOAT_SIN => val.sin(),
        FLOAT_COS => val.cos(),
        FLOAT_TAN => val.tan(),
        FLOAT_LOG => val.ln(),
        FLOAT_LOG10 => val.log10(),
        FLOAT_EXP => val.exp(),
        _ => val,
    }
}

/// Check if float is special (NaN, infinity).
fn is_special_float(val: f64) -> bool {
    val.is_nan() || val.is_infinite()
}

/// Check if float operation can fail (sqrt of negative, log of non-positive).
fn float_op_can_fail(op: c_int, val: f64) -> bool {
    match op {
        FLOAT_SQRT => val < 0.0,
        FLOAT_LOG | FLOAT_LOG10 => val <= 0.0,
        _ => false,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get UNOP_NEGATE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_unop_negate() -> c_int {
    UNOP_NEGATE
}

/// FFI: Get UNOP_PLUS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_unop_plus() -> c_int {
    UNOP_PLUS
}

/// FFI: Get UNOP_NOT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_unop_not() -> c_int {
    UNOP_NOT
}

/// FFI: Negate integer.
#[unsafe(no_mangle)]
pub extern "C" fn rs_negate_int(val: i64) -> i64 {
    negate_int(val)
}

/// FFI: Logical not integer.
#[unsafe(no_mangle)]
pub extern "C" fn rs_logical_not_int(val: i64) -> i64 {
    logical_not_int(val)
}

/// FFI: Negate float.
#[unsafe(no_mangle)]
pub extern "C" fn rs_negate_float(val: f64) -> f64 {
    negate_float(val)
}

/// FFI: Logical not float.
#[unsafe(no_mangle)]
pub extern "C" fn rs_logical_not_float(val: f64) -> i64 {
    logical_not_float(val)
}

/// FFI: Integer to boolean.
#[unsafe(no_mangle)]
pub extern "C" fn rs_int_to_bool(val: i64) -> c_int {
    c_int::from(int_to_bool(val))
}

/// FFI: Boolean to integer.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bool_to_int(val: c_int) -> i64 {
    bool_to_int(val != 0)
}

/// FFI: Get FLOAT_ABS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_abs_op() -> c_int {
    FLOAT_ABS
}

/// FFI: Get FLOAT_SQRT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_sqrt_op() -> c_int {
    FLOAT_SQRT
}

/// FFI: Get FLOAT_SIN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_sin_op() -> c_int {
    FLOAT_SIN
}

/// FFI: Get FLOAT_COS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_cos_op() -> c_int {
    FLOAT_COS
}

/// FFI: Apply float operation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_apply_float_op(op: c_int, val: f64) -> f64 {
    apply_float_op(op, val)
}

/// FFI: Check if special float.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_special_float(val: f64) -> c_int {
    c_int::from(is_special_float(val))
}

/// FFI: Check if float op can fail.
#[unsafe(no_mangle)]
pub extern "C" fn rs_float_op_can_fail(op: c_int, val: f64) -> c_int {
    c_int::from(float_op_can_fail(op, val))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unop_constants() {
        assert_eq!(UNOP_NEGATE, 0);
        assert_eq!(UNOP_PLUS, 1);
        assert_eq!(UNOP_NOT, 2);
    }

    #[test]
    fn test_negate_int() {
        assert_eq!(negate_int(5), -5);
        assert_eq!(negate_int(-5), 5);
        assert_eq!(negate_int(0), 0);
    }

    #[test]
    fn test_logical_not_int() {
        assert_eq!(logical_not_int(0), 1);
        assert_eq!(logical_not_int(1), 0);
        assert_eq!(logical_not_int(42), 0);
        assert_eq!(logical_not_int(-1), 0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_negate_float() {
        assert_eq!(negate_float(5.0), -5.0);
        assert_eq!(negate_float(-5.0), 5.0);
    }

    #[test]
    fn test_logical_not_float() {
        assert_eq!(logical_not_float(0.0), 1);
        assert_eq!(logical_not_float(1.0), 0);
        assert_eq!(logical_not_float(0.5), 0);
    }

    #[test]
    fn test_bool_conversions() {
        assert!(!int_to_bool(0));
        assert!(int_to_bool(1));
        assert!(int_to_bool(-1));

        assert_eq!(bool_to_int(false), 0);
        assert_eq!(bool_to_int(true), 1);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_float_operations() {
        assert_eq!(apply_float_op(FLOAT_ABS, -5.0), 5.0);
        assert_eq!(apply_float_op(FLOAT_CEIL, 1.1), 2.0);
        assert_eq!(apply_float_op(FLOAT_FLOOR, 1.9), 1.0);
        assert_eq!(apply_float_op(FLOAT_ROUND, 1.5), 2.0);
        assert_eq!(apply_float_op(FLOAT_TRUNC, 1.9), 1.0);
    }

    #[test]
    fn test_is_special_float() {
        assert!(!is_special_float(1.0));
        assert!(is_special_float(f64::NAN));
        assert!(is_special_float(f64::INFINITY));
        assert!(is_special_float(f64::NEG_INFINITY));
    }

    #[test]
    fn test_float_op_can_fail() {
        assert!(float_op_can_fail(FLOAT_SQRT, -1.0));
        assert!(!float_op_can_fail(FLOAT_SQRT, 1.0));
        assert!(float_op_can_fail(FLOAT_LOG, 0.0));
        assert!(float_op_can_fail(FLOAT_LOG, -1.0));
        assert!(!float_op_can_fail(FLOAT_LOG, 1.0));
    }
}
