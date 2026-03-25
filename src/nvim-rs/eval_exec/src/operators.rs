//! VimL operator implementations.
//!
//! This module provides operator implementations for the VimL expression evaluator,
//! migrated from `src/nvim/eval.c`. It handles:
//!
//! - Comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`, `=~`, `!~`, `is`, `isnot`)
//! - Type coercion for comparisons (number, float, string, etc.)
//! - Case sensitivity handling (`#` for case-sensitive, `?` for case-insensitive)
//!
//! ## Comparison Rules
//!
//! VimL has specific comparison semantics:
//! - If either operand is a number, compare as numbers
//! - If either operand is a float, compare as floats
//! - For lists/dicts/blobs, only `==` and `!=` are supported
//! - `is` and `isnot` check for same instance (identity)
//! - `=~` and `!~` are regex match operations

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)] // Some functions are for future use

use std::ffi::{c_char, c_int, c_void};

use nvim_eval::typval::TypvalT as TypvalTRepr;

use crate::eval::TypevalHandle;

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

// VarType constants
const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;
const VAR_PARTIAL: c_int = 9;
const VAR_BLOB: c_int = 10;

// Expression type constants
const EXPR_UNKNOWN: c_int = 0;
const EXPR_EQUAL: c_int = 1;
const EXPR_NEQUAL: c_int = 2;
const EXPR_GREATER: c_int = 3;
const EXPR_GEQUAL: c_int = 4;
const EXPR_SMALLER: c_int = 5;
const EXPR_SEQUAL: c_int = 6;
const EXPR_MATCH: c_int = 7;
const EXPR_NOMATCH: c_int = 8;
const EXPR_IS: c_int = 9;
const EXPR_ISNOT: c_int = 10;

// =============================================================================
// Comparison Result Type
// =============================================================================

/// Result of a comparison operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CompareResult {
    /// Values are equal
    Equal = 0,
    /// First value is less than second
    Less = -1,
    /// First value is greater than second
    Greater = 1,
    /// Values cannot be compared (type mismatch)
    Incomparable = -2,
}

impl CompareResult {
    /// Convert from comparison integer (-1, 0, 1) to CompareResult.
    pub const fn from_cmp(cmp: i32) -> Self {
        if cmp < 0 {
            Self::Less
        } else if cmp > 0 {
            Self::Greater
        } else {
            Self::Equal
        }
    }

    /// Check if this satisfies the given expression type.
    pub const fn satisfies(self, expr_type: c_int) -> bool {
        match expr_type {
            EXPR_EQUAL | EXPR_IS => matches!(self, Self::Equal),
            EXPR_NEQUAL | EXPR_ISNOT => !matches!(self, Self::Equal),
            EXPR_GREATER => matches!(self, Self::Greater),
            EXPR_GEQUAL => matches!(self, Self::Equal | Self::Greater),
            EXPR_SMALLER => matches!(self, Self::Less),
            EXPR_SEQUAL => matches!(self, Self::Equal | Self::Less),
            _ => false,
        }
    }
}

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Type checking
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;

    // Value extraction
    fn tv_get_number(tv: TypevalHandle) -> i64;
    fn tv_get_float(tv: TypevalHandle) -> f64;
    fn tv_get_string_buf(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;

    // Typval operations
    fn tv_clear(tv: TypevalHandle);

    // List operations
    fn nvim_tv_get_list(tv: TypevalHandle) -> *mut c_void;
    fn tv_list_equal(l1: *mut c_void, l2: *mut c_void, ic: c_int) -> c_int;

    // Dict operations
    fn nvim_tv_get_dict(tv: TypevalHandle) -> *mut c_void;
    fn tv_dict_equal(d1: *mut c_void, d2: *mut c_void, ic: c_int) -> c_int;

    // Blob operations
    fn nvim_tv_get_blob(tv: TypevalHandle) -> *mut c_void;
    fn tv_blob_equal(b1: *mut c_void, b2: *mut c_void) -> c_int;

    // String comparison
    fn mb_strcmp_ic(ic: c_int, s1: *const c_char, s2: *const c_char) -> c_int;

    // Pattern matching
    fn pattern_match(pat: *const c_char, text: *const c_char, ic: c_int) -> c_int;

    // Type checking helpers
    fn nvim_tv_is_func(tv: TypevalHandle) -> c_int;

    // General equality
    fn tv_equal(tv1: TypevalHandle, tv2: TypevalHandle, ic: bool) -> bool;

    // Error messages
    fn emsg(s: *const c_char) -> c_int;

    // Typval setters
    fn nvim_tv_set_number(tv: TypevalHandle, n: i64);
}

// =============================================================================
// Error Messages
// =============================================================================

static E_CMP_BLOB: &[u8] = b"E977: Can only compare Blob with Blob\0";
static E_INVALID_BLOB: &[u8] = b"E978: Invalid operation for Blob\0";
static E_CMP_LIST: &[u8] = b"E691: Can only compare List with List\0";
static E_INVALID_LIST: &[u8] = b"E692: Invalid operation for List\0";
static E_CMP_DICT: &[u8] = b"E735: Can only compare Dictionary with Dictionary\0";
static E_INVALID_DICT: &[u8] = b"E736: Invalid operation for Dictionary\0";
static E_INVALID_FUNCREF: &[u8] = b"E694: Invalid operation for Funcrefs\0";

// =============================================================================
// Number Comparison
// =============================================================================

/// Compare two numbers and return the result.
#[inline]
pub const fn compare_numbers(n1: i64, n2: i64) -> CompareResult {
    if n1 < n2 {
        CompareResult::Less
    } else if n1 > n2 {
        CompareResult::Greater
    } else {
        CompareResult::Equal
    }
}

/// FFI export for number comparison.
#[no_mangle]
pub extern "C" fn rs_compare_numbers(n1: i64, n2: i64) -> c_int {
    compare_numbers(n1, n2) as c_int
}

// =============================================================================
// Float Comparison
// =============================================================================

/// Compare two floats and return the result.
#[inline]
pub fn compare_floats(f1: f64, f2: f64) -> CompareResult {
    if f1.is_nan() || f2.is_nan() {
        // NaN comparisons are always unequal
        CompareResult::Incomparable
    } else if f1 < f2 {
        CompareResult::Less
    } else if f1 > f2 {
        CompareResult::Greater
    } else {
        CompareResult::Equal
    }
}

/// FFI export for float comparison.
#[no_mangle]
pub extern "C" fn rs_compare_floats(f1: f64, f2: f64) -> c_int {
    compare_floats(f1, f2) as c_int
}

// =============================================================================
// Apply Comparison Result to Expression Type
// =============================================================================

/// Apply a comparison result to an expression type to get a boolean.
#[inline]
pub const fn apply_comparison(cmp: c_int, expr_type: c_int) -> bool {
    match expr_type {
        EXPR_EQUAL | EXPR_IS => cmp == 0,
        EXPR_NEQUAL | EXPR_ISNOT => cmp != 0,
        EXPR_GREATER => cmp > 0,
        EXPR_GEQUAL => cmp >= 0,
        EXPR_SMALLER => cmp < 0,
        EXPR_SEQUAL => cmp <= 0,
        _ => false,
    }
}

/// FFI export for applying comparison.
#[no_mangle]
pub extern "C" fn rs_apply_comparison(cmp: c_int, expr_type: c_int) -> c_int {
    c_int::from(apply_comparison(cmp, expr_type))
}

// =============================================================================
// Type Comparison
// =============================================================================

/// Compare two typvals.
///
/// This is the main comparison function that handles all VimL types.
///
/// # Safety
/// - `typ1` and `typ2` must be valid typval handles
pub unsafe fn typval_compare_impl(
    typ1: TypevalHandle,
    typ2: TypevalHandle,
    expr_type: c_int,
    ic: c_int,
) -> c_int {
    let t1 = nvim_tv_get_type(typ1);
    let t2 = nvim_tv_get_type(typ2);
    let type_is = expr_type == EXPR_IS || expr_type == EXPR_ISNOT;

    let result: i64;

    // Check for type mismatch with is/isnot
    if type_is && t1 != t2 {
        // For "is" a different type always means false, for "isnot" it means true
        result = if expr_type == EXPR_ISNOT { 1 } else { 0 };
    }
    // Blob comparison
    else if t1 == VAR_BLOB || t2 == VAR_BLOB {
        if type_is {
            let same = t1 == t2 && nvim_tv_get_blob(typ1) == nvim_tv_get_blob(typ2);
            result = if (expr_type == EXPR_ISNOT) != same {
                1
            } else {
                0
            };
        } else if t1 != t2 || (expr_type != EXPR_EQUAL && expr_type != EXPR_NEQUAL) {
            if t1 != t2 {
                emsg(E_CMP_BLOB.as_ptr() as *const c_char);
            } else {
                emsg(E_INVALID_BLOB.as_ptr() as *const c_char);
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            let eq = tv_blob_equal(nvim_tv_get_blob(typ1), nvim_tv_get_blob(typ2)) != 0;
            result = if (expr_type == EXPR_NEQUAL) != eq {
                1
            } else {
                0
            };
        }
    }
    // List comparison
    else if t1 == VAR_LIST || t2 == VAR_LIST {
        if type_is {
            let same = t1 == t2 && nvim_tv_get_list(typ1) == nvim_tv_get_list(typ2);
            result = if (expr_type == EXPR_ISNOT) != same {
                1
            } else {
                0
            };
        } else if t1 != t2 || (expr_type != EXPR_EQUAL && expr_type != EXPR_NEQUAL) {
            if t1 != t2 {
                emsg(E_CMP_LIST.as_ptr() as *const c_char);
            } else {
                emsg(E_INVALID_LIST.as_ptr() as *const c_char);
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            let eq = tv_list_equal(nvim_tv_get_list(typ1), nvim_tv_get_list(typ2), ic) != 0;
            result = if (expr_type == EXPR_NEQUAL) != eq {
                1
            } else {
                0
            };
        }
    }
    // Dict comparison
    else if t1 == VAR_DICT || t2 == VAR_DICT {
        if type_is {
            let same = t1 == t2 && nvim_tv_get_dict(typ1) == nvim_tv_get_dict(typ2);
            result = if (expr_type == EXPR_ISNOT) != same {
                1
            } else {
                0
            };
        } else if t1 != t2 || (expr_type != EXPR_EQUAL && expr_type != EXPR_NEQUAL) {
            if t1 != t2 {
                emsg(E_CMP_DICT.as_ptr() as *const c_char);
            } else {
                emsg(E_INVALID_DICT.as_ptr() as *const c_char);
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            let eq = tv_dict_equal(nvim_tv_get_dict(typ1), nvim_tv_get_dict(typ2), ic) != 0;
            result = if (expr_type == EXPR_NEQUAL) != eq {
                1
            } else {
                0
            };
        }
    }
    // Funcref comparison
    else if nvim_tv_is_func(typ1) != 0 || nvim_tv_is_func(typ2) != 0 {
        if expr_type != EXPR_EQUAL
            && expr_type != EXPR_NEQUAL
            && expr_type != EXPR_IS
            && expr_type != EXPR_ISNOT
        {
            emsg(E_INVALID_FUNCREF.as_ptr() as *const c_char);
            tv_clear(typ1);
            return FAIL;
        }

        let eq: bool;
        let p1 = if t1 == VAR_PARTIAL {
            (*typ1.as_ptr().cast::<TypvalTRepr>()).vval.v_partial
        } else {
            std::ptr::null_mut()
        };
        let p2 = if t2 == VAR_PARTIAL {
            (*typ2.as_ptr().cast::<TypvalTRepr>()).vval.v_partial
        } else {
            std::ptr::null_mut()
        };

        if (t1 == VAR_PARTIAL && p1.is_null()) || (t2 == VAR_PARTIAL && p2.is_null()) {
            // When both partials are NULL, they are equal
            eq = p1 == p2;
        } else if type_is {
            if t1 == VAR_FUNC && t2 == VAR_FUNC {
                eq = tv_equal(typ1, typ2, ic != 0);
            } else if t1 == VAR_PARTIAL && t2 == VAR_PARTIAL {
                eq = p1 == p2;
            } else {
                eq = false;
            }
        } else {
            eq = tv_equal(typ1, typ2, ic != 0);
        }

        result = if (expr_type == EXPR_NEQUAL || expr_type == EXPR_ISNOT) != eq {
            1
        } else {
            0
        };
    }
    // Float comparison
    else if (t1 == VAR_FLOAT || t2 == VAR_FLOAT)
        && expr_type != EXPR_MATCH
        && expr_type != EXPR_NOMATCH
    {
        let f1 = tv_get_float(typ1);
        let f2 = tv_get_float(typ2);

        let cmp_result = match expr_type {
            EXPR_IS | EXPR_EQUAL => f1 == f2,
            EXPR_ISNOT | EXPR_NEQUAL => f1 != f2,
            EXPR_GREATER => f1 > f2,
            EXPR_GEQUAL => f1 >= f2,
            EXPR_SMALLER => f1 < f2,
            EXPR_SEQUAL => f1 <= f2,
            _ => false,
        };
        result = if cmp_result { 1 } else { 0 };
    }
    // Number comparison
    else if (t1 == VAR_NUMBER || t2 == VAR_NUMBER)
        && expr_type != EXPR_MATCH
        && expr_type != EXPR_NOMATCH
    {
        let n1 = tv_get_number(typ1);
        let n2 = tv_get_number(typ2);

        let cmp_result = match expr_type {
            EXPR_IS | EXPR_EQUAL => n1 == n2,
            EXPR_ISNOT | EXPR_NEQUAL => n1 != n2,
            EXPR_GREATER => n1 > n2,
            EXPR_GEQUAL => n1 >= n2,
            EXPR_SMALLER => n1 < n2,
            EXPR_SEQUAL => n1 <= n2,
            _ => false,
        };
        result = if cmp_result { 1 } else { 0 };
    }
    // String comparison
    else {
        let mut buf1 = [0i8; 64]; // NUMBUFLEN
        let mut buf2 = [0i8; 64];
        let s1 = tv_get_string_buf(typ1, buf1.as_mut_ptr());
        let s2 = tv_get_string_buf(typ2, buf2.as_mut_ptr());

        if expr_type == EXPR_MATCH || expr_type == EXPR_NOMATCH {
            // Pattern matching
            let matched = pattern_match(s2, s1, ic) != 0;
            result = if (expr_type == EXPR_NOMATCH) != matched {
                1
            } else {
                0
            };
        } else {
            // String comparison
            let cmp = mb_strcmp_ic(ic, s1, s2);
            result = if apply_comparison(cmp, expr_type) {
                1
            } else {
                0
            };
        }
    }

    // Set the result
    tv_clear(typ1);
    typ1.set_type(VAR_NUMBER);
    nvim_tv_set_number(typ1, result);

    OK
}

/// FFI export for typval comparison.
///
/// # Safety
/// See `typval_compare_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_typval_compare(
    typ1: TypevalHandle,
    typ2: TypevalHandle,
    expr_type: c_int,
    ic: c_int,
) -> c_int {
    typval_compare_impl(typ1, typ2, expr_type, ic)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_numbers() {
        assert_eq!(compare_numbers(1, 2), CompareResult::Less);
        assert_eq!(compare_numbers(2, 2), CompareResult::Equal);
        assert_eq!(compare_numbers(3, 2), CompareResult::Greater);
    }

    #[test]
    fn test_compare_floats() {
        assert_eq!(compare_floats(1.0, 2.0), CompareResult::Less);
        assert_eq!(compare_floats(2.0, 2.0), CompareResult::Equal);
        assert_eq!(compare_floats(3.0, 2.0), CompareResult::Greater);
        assert_eq!(compare_floats(f64::NAN, 1.0), CompareResult::Incomparable);
    }

    #[test]
    fn test_apply_comparison() {
        // Equal
        assert!(apply_comparison(0, EXPR_EQUAL));
        assert!(!apply_comparison(1, EXPR_EQUAL));
        assert!(!apply_comparison(-1, EXPR_EQUAL));

        // Not equal
        assert!(!apply_comparison(0, EXPR_NEQUAL));
        assert!(apply_comparison(1, EXPR_NEQUAL));
        assert!(apply_comparison(-1, EXPR_NEQUAL));

        // Greater
        assert!(!apply_comparison(0, EXPR_GREATER));
        assert!(apply_comparison(1, EXPR_GREATER));
        assert!(!apply_comparison(-1, EXPR_GREATER));

        // Greater or equal
        assert!(apply_comparison(0, EXPR_GEQUAL));
        assert!(apply_comparison(1, EXPR_GEQUAL));
        assert!(!apply_comparison(-1, EXPR_GEQUAL));

        // Smaller
        assert!(!apply_comparison(0, EXPR_SMALLER));
        assert!(!apply_comparison(1, EXPR_SMALLER));
        assert!(apply_comparison(-1, EXPR_SMALLER));

        // Smaller or equal
        assert!(apply_comparison(0, EXPR_SEQUAL));
        assert!(!apply_comparison(1, EXPR_SEQUAL));
        assert!(apply_comparison(-1, EXPR_SEQUAL));
    }

    #[test]
    fn test_compare_result_satisfies() {
        assert!(CompareResult::Equal.satisfies(EXPR_EQUAL));
        assert!(!CompareResult::Less.satisfies(EXPR_EQUAL));
        assert!(CompareResult::Greater.satisfies(EXPR_NEQUAL));
    }
}
