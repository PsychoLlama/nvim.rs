//! Comparison operations.
//!
//! This module provides helpers for comparison operations:
//! tv_compare, type checking, type coercion

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Comparison Type Constants
// =============================================================================

/// Equal comparison (==).
pub const CMP_EQUAL: c_int = 0;
/// Not equal comparison (!=).
pub const CMP_NEQUAL: c_int = 1;
/// Greater than comparison (>).
pub const CMP_GREATER: c_int = 2;
/// Greater or equal comparison (>=).
pub const CMP_GEQUAL: c_int = 3;
/// Less than comparison (<).
pub const CMP_SMALLER: c_int = 4;
/// Less or equal comparison (<=).
pub const CMP_SEQUAL: c_int = 5;
/// Match comparison (=~).
pub const CMP_MATCH: c_int = 6;
/// No match comparison (!~).
pub const CMP_NOMATCH: c_int = 7;
/// Identity comparison (is).
pub const CMP_IS: c_int = 8;
/// Not identity comparison (isnot).
pub const CMP_ISNOT: c_int = 9;

// =============================================================================
// Case Sensitivity Constants
// =============================================================================

/// Case-sensitive comparison (default).
pub const CASE_MATCH: c_int = 0;
/// Case-insensitive comparison (? suffix).
pub const CASE_IGNORE: c_int = 1;
/// Follow 'ignorecase' option (# suffix).
pub const CASE_FOLLOW_IC: c_int = 2;

// =============================================================================
// Comparison Result Constants
// =============================================================================

/// Result: false/not equal.
pub const RESULT_FALSE: c_int = 0;
/// Result: true/equal.
pub const RESULT_TRUE: c_int = 1;
/// Result: comparison error.
pub const RESULT_ERROR: c_int = -1;

// =============================================================================
// Comparison Helpers
// =============================================================================

/// Get the inverse of a comparison type.
fn invert_cmp(cmp_type: c_int) -> c_int {
    match cmp_type {
        CMP_EQUAL => CMP_NEQUAL,
        CMP_NEQUAL => CMP_EQUAL,
        CMP_GREATER => CMP_SEQUAL,
        CMP_GEQUAL => CMP_SMALLER,
        CMP_SMALLER => CMP_GEQUAL,
        CMP_SEQUAL => CMP_GREATER,
        CMP_MATCH => CMP_NOMATCH,
        CMP_NOMATCH => CMP_MATCH,
        CMP_IS => CMP_ISNOT,
        CMP_ISNOT => CMP_IS,
        _ => cmp_type,
    }
}

/// Check if comparison is equality-based.
fn is_equality_cmp(cmp_type: c_int) -> bool {
    matches!(cmp_type, CMP_EQUAL | CMP_NEQUAL | CMP_IS | CMP_ISNOT)
}

/// Check if comparison is relational.
fn is_relational_cmp(cmp_type: c_int) -> bool {
    matches!(
        cmp_type,
        CMP_GREATER | CMP_GEQUAL | CMP_SMALLER | CMP_SEQUAL
    )
}

/// Check if comparison is pattern-based.
fn is_pattern_cmp(cmp_type: c_int) -> bool {
    matches!(cmp_type, CMP_MATCH | CMP_NOMATCH)
}

/// Check if comparison is identity-based.
fn is_identity_cmp(cmp_type: c_int) -> bool {
    matches!(cmp_type, CMP_IS | CMP_ISNOT)
}

/// Compare two integers.
fn cmp_int(a: i64, b: i64, cmp_type: c_int) -> c_int {
    let result = match cmp_type {
        CMP_EQUAL | CMP_IS => a == b,
        CMP_NEQUAL | CMP_ISNOT => a != b,
        CMP_GREATER => a > b,
        CMP_GEQUAL => a >= b,
        CMP_SMALLER => a < b,
        CMP_SEQUAL => a <= b,
        _ => false,
    };
    c_int::from(result)
}

/// Compare two floats.
#[allow(clippy::float_cmp)]
fn cmp_float(a: f64, b: f64, cmp_type: c_int) -> c_int {
    let result = match cmp_type {
        CMP_EQUAL | CMP_IS => a == b,
        CMP_NEQUAL | CMP_ISNOT => a != b,
        CMP_GREATER => a > b,
        CMP_GEQUAL => a >= b,
        CMP_SMALLER => a < b,
        CMP_SEQUAL => a <= b,
        _ => false,
    };
    c_int::from(result)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get CMP_EQUAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_equal() -> c_int {
    CMP_EQUAL
}

/// FFI: Get CMP_NEQUAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_nequal() -> c_int {
    CMP_NEQUAL
}

/// FFI: Get CMP_GREATER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_greater() -> c_int {
    CMP_GREATER
}

/// FFI: Get CMP_SMALLER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_smaller() -> c_int {
    CMP_SMALLER
}

/// FFI: Get CMP_MATCH constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_match() -> c_int {
    CMP_MATCH
}

/// FFI: Get CMP_IS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_is() -> c_int {
    CMP_IS
}

/// FFI: Get CASE_MATCH constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_case_match() -> c_int {
    CASE_MATCH
}

/// FFI: Get CASE_IGNORE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_case_ignore() -> c_int {
    CASE_IGNORE
}

/// FFI: Get CASE_FOLLOW_IC constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_case_follow_ic() -> c_int {
    CASE_FOLLOW_IC
}

/// FFI: Get inverse comparison type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_invert_cmp(cmp_type: c_int) -> c_int {
    invert_cmp(cmp_type)
}

/// FFI: Check if equality comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_equality_cmp(cmp_type: c_int) -> c_int {
    c_int::from(is_equality_cmp(cmp_type))
}

/// FFI: Check if relational comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_relational_cmp(cmp_type: c_int) -> c_int {
    c_int::from(is_relational_cmp(cmp_type))
}

/// FFI: Check if pattern comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_pattern_cmp(cmp_type: c_int) -> c_int {
    c_int::from(is_pattern_cmp(cmp_type))
}

/// FFI: Check if identity comparison.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_identity_cmp(cmp_type: c_int) -> c_int {
    c_int::from(is_identity_cmp(cmp_type))
}

/// FFI: Compare two integers.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_int(a: i64, b: i64, cmp_type: c_int) -> c_int {
    cmp_int(a, b, cmp_type)
}

/// FFI: Compare two floats.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmp_float(a: f64, b: f64, cmp_type: c_int) -> c_int {
    cmp_float(a, b, cmp_type)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_constants() {
        assert_eq!(CMP_EQUAL, 0);
        assert_eq!(CMP_NEQUAL, 1);
        assert_eq!(CMP_GREATER, 2);
        assert_eq!(CMP_SMALLER, 4);
    }

    #[test]
    fn test_case_constants() {
        assert_eq!(CASE_MATCH, 0);
        assert_eq!(CASE_IGNORE, 1);
        assert_eq!(CASE_FOLLOW_IC, 2);
    }

    #[test]
    fn test_invert_cmp() {
        assert_eq!(invert_cmp(CMP_EQUAL), CMP_NEQUAL);
        assert_eq!(invert_cmp(CMP_NEQUAL), CMP_EQUAL);
        assert_eq!(invert_cmp(CMP_GREATER), CMP_SEQUAL);
        assert_eq!(invert_cmp(CMP_SMALLER), CMP_GEQUAL);
        assert_eq!(invert_cmp(CMP_IS), CMP_ISNOT);
    }

    #[test]
    fn test_cmp_type_checks() {
        assert!(is_equality_cmp(CMP_EQUAL));
        assert!(is_equality_cmp(CMP_NEQUAL));
        assert!(!is_equality_cmp(CMP_GREATER));

        assert!(is_relational_cmp(CMP_GREATER));
        assert!(is_relational_cmp(CMP_SMALLER));
        assert!(!is_relational_cmp(CMP_EQUAL));

        assert!(is_pattern_cmp(CMP_MATCH));
        assert!(is_pattern_cmp(CMP_NOMATCH));
        assert!(!is_pattern_cmp(CMP_EQUAL));

        assert!(is_identity_cmp(CMP_IS));
        assert!(is_identity_cmp(CMP_ISNOT));
        assert!(!is_identity_cmp(CMP_EQUAL));
    }

    #[test]
    fn test_cmp_int() {
        assert_eq!(cmp_int(5, 5, CMP_EQUAL), 1);
        assert_eq!(cmp_int(5, 3, CMP_EQUAL), 0);
        assert_eq!(cmp_int(5, 3, CMP_GREATER), 1);
        assert_eq!(cmp_int(5, 3, CMP_SMALLER), 0);
        assert_eq!(cmp_int(5, 5, CMP_GEQUAL), 1);
        assert_eq!(cmp_int(5, 5, CMP_SEQUAL), 1);
    }

    #[test]
    fn test_cmp_float() {
        assert_eq!(cmp_float(5.0, 5.0, CMP_EQUAL), 1);
        assert_eq!(cmp_float(5.0, 3.0, CMP_GREATER), 1);
        assert_eq!(cmp_float(3.0, 5.0, CMP_SMALLER), 1);
    }
}
