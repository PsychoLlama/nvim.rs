//! Expression constants and type definitions.
//!
//! This module provides VAR_* type constants, EVAL_* flags,
//! and expression precedence values.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// VimL Variable Type Constants (VAR_*)
// =============================================================================

/// Unknown/uninitialized type.
pub const VAR_UNKNOWN: c_int = 0;
/// Number type (integer).
pub const VAR_NUMBER: c_int = 1;
/// String type.
pub const VAR_STRING: c_int = 2;
/// Funcref type.
pub const VAR_FUNC: c_int = 3;
/// List type.
pub const VAR_LIST: c_int = 4;
/// Dictionary type.
pub const VAR_DICT: c_int = 5;
/// Float type.
pub const VAR_FLOAT: c_int = 6;
/// Boolean type.
pub const VAR_BOOL: c_int = 7;
/// Special type (v:null, v:none).
pub const VAR_SPECIAL: c_int = 8;
/// Partial funcref type.
pub const VAR_PARTIAL: c_int = 9;
/// Blob type.
pub const VAR_BLOB: c_int = 10;

// =============================================================================
// Evaluation Flags (EVAL_*)
// =============================================================================

/// Evaluate expression for completion.
pub const EVAL_COMPLETE: c_int = 0x01;
/// Evaluate as part of :let assignment.
pub const EVAL_LET: c_int = 0x02;
/// Evaluate in sandbox mode.
pub const EVAL_SANDBOX: c_int = 0x04;
/// Evaluate for :if/:elseif/:while.
pub const EVAL_CONDITIONAL: c_int = 0x08;
/// Evaluate and return typval.
pub const EVAL_TYPVAL: c_int = 0x10;

// =============================================================================
// Expression Precedence Levels
// =============================================================================

/// Lowest precedence (ternary ?:).
pub const PREC_TERNARY: c_int = 0;
/// Logical OR (||).
pub const PREC_OR: c_int = 1;
/// Logical AND (&&).
pub const PREC_AND: c_int = 2;
/// Comparison operators.
pub const PREC_COMPARE: c_int = 3;
/// Addition/subtraction (+, -).
pub const PREC_ADD: c_int = 4;
/// Multiplication/division (*, /, %).
pub const PREC_MUL: c_int = 5;
/// Unary operators (!, -, +).
pub const PREC_UNARY: c_int = 6;
/// Subscript and method call ([], .).
pub const PREC_SUBSCRIPT: c_int = 7;
/// Primary (literals, variables, parentheses).
pub const PREC_PRIMARY: c_int = 8;

// =============================================================================
// Special Values
// =============================================================================

/// v:false boolean value.
pub const VVAL_FALSE: c_int = 0;
/// v:true boolean value.
pub const VVAL_TRUE: c_int = 1;
/// v:none special value.
pub const VVAL_NONE: c_int = 2;
/// v:null special value.
pub const VVAL_NULL: c_int = 3;

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if type is a basic scalar type.
fn is_scalar_type(var_type: c_int) -> bool {
    matches!(
        var_type,
        VAR_NUMBER | VAR_STRING | VAR_FLOAT | VAR_BOOL | VAR_SPECIAL
    )
}

/// Check if type is a collection type.
fn is_collection_type(var_type: c_int) -> bool {
    matches!(var_type, VAR_LIST | VAR_DICT | VAR_BLOB)
}

/// Check if type is callable.
fn is_callable_type(var_type: c_int) -> bool {
    matches!(var_type, VAR_FUNC | VAR_PARTIAL)
}

/// Get type name string for type id.
#[allow(dead_code)]
fn type_name(var_type: c_int) -> &'static str {
    match var_type {
        VAR_NUMBER => "number",
        VAR_STRING => "string",
        VAR_FUNC => "funcref",
        VAR_LIST => "list",
        VAR_DICT => "dict",
        VAR_FLOAT => "float",
        VAR_BOOL => "bool",
        VAR_SPECIAL => "special",
        VAR_PARTIAL => "partial",
        VAR_BLOB => "blob",
        _ => "unknown", // VAR_UNKNOWN and unrecognized types
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get VAR_UNKNOWN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_unknown() -> c_int {
    VAR_UNKNOWN
}

/// FFI: Get VAR_NUMBER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_number() -> c_int {
    VAR_NUMBER
}

/// FFI: Get VAR_STRING constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_string() -> c_int {
    VAR_STRING
}

/// FFI: Get VAR_FUNC constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_func() -> c_int {
    VAR_FUNC
}

/// FFI: Get VAR_LIST constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_list() -> c_int {
    VAR_LIST
}

/// FFI: Get VAR_DICT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_dict() -> c_int {
    VAR_DICT
}

/// FFI: Get VAR_FLOAT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_float() -> c_int {
    VAR_FLOAT
}

/// FFI: Get VAR_BOOL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_bool() -> c_int {
    VAR_BOOL
}

/// FFI: Get VAR_SPECIAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_special() -> c_int {
    VAR_SPECIAL
}

/// FFI: Get VAR_PARTIAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_partial() -> c_int {
    VAR_PARTIAL
}

/// FFI: Get VAR_BLOB constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_blob() -> c_int {
    VAR_BLOB
}

/// FFI: Check if type is scalar.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_scalar_type(var_type: c_int) -> c_int {
    c_int::from(is_scalar_type(var_type))
}

/// FFI: Check if type is collection.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_collection_type(var_type: c_int) -> c_int {
    c_int::from(is_collection_type(var_type))
}

/// FFI: Check if type is callable.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_callable_type(var_type: c_int) -> c_int {
    c_int::from(is_callable_type(var_type))
}

/// FFI: Get VVAL_FALSE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vval_false() -> c_int {
    VVAL_FALSE
}

/// FFI: Get VVAL_TRUE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vval_true() -> c_int {
    VVAL_TRUE
}

/// FFI: Get VVAL_NONE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vval_none() -> c_int {
    VVAL_NONE
}

/// FFI: Get VVAL_NULL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_vval_null() -> c_int {
    VVAL_NULL
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_type_constants() {
        assert_eq!(VAR_UNKNOWN, 0);
        assert_eq!(VAR_NUMBER, 1);
        assert_eq!(VAR_STRING, 2);
        assert_eq!(VAR_FUNC, 3);
        assert_eq!(VAR_LIST, 4);
        assert_eq!(VAR_DICT, 5);
        assert_eq!(VAR_FLOAT, 6);
        assert_eq!(VAR_BOOL, 7);
        assert_eq!(VAR_SPECIAL, 8);
        assert_eq!(VAR_PARTIAL, 9);
        assert_eq!(VAR_BLOB, 10);
    }

    #[test]
    fn test_eval_flags() {
        assert_eq!(EVAL_COMPLETE, 0x01);
        assert_eq!(EVAL_LET, 0x02);
        assert_eq!(EVAL_SANDBOX, 0x04);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_precedence_order() {
        assert!(PREC_TERNARY < PREC_OR);
        assert!(PREC_OR < PREC_AND);
        assert!(PREC_AND < PREC_COMPARE);
        assert!(PREC_COMPARE < PREC_ADD);
        assert!(PREC_ADD < PREC_MUL);
        assert!(PREC_MUL < PREC_UNARY);
        assert!(PREC_UNARY < PREC_SUBSCRIPT);
        assert!(PREC_SUBSCRIPT < PREC_PRIMARY);
    }

    #[test]
    fn test_is_scalar_type() {
        assert!(is_scalar_type(VAR_NUMBER));
        assert!(is_scalar_type(VAR_STRING));
        assert!(is_scalar_type(VAR_FLOAT));
        assert!(is_scalar_type(VAR_BOOL));
        assert!(!is_scalar_type(VAR_LIST));
        assert!(!is_scalar_type(VAR_DICT));
        assert!(!is_scalar_type(VAR_FUNC));
    }

    #[test]
    fn test_is_collection_type() {
        assert!(is_collection_type(VAR_LIST));
        assert!(is_collection_type(VAR_DICT));
        assert!(is_collection_type(VAR_BLOB));
        assert!(!is_collection_type(VAR_NUMBER));
        assert!(!is_collection_type(VAR_STRING));
    }

    #[test]
    fn test_is_callable_type() {
        assert!(is_callable_type(VAR_FUNC));
        assert!(is_callable_type(VAR_PARTIAL));
        assert!(!is_callable_type(VAR_NUMBER));
        assert!(!is_callable_type(VAR_LIST));
    }

    #[test]
    fn test_type_name() {
        assert_eq!(type_name(VAR_NUMBER), "number");
        assert_eq!(type_name(VAR_STRING), "string");
        assert_eq!(type_name(VAR_LIST), "list");
        assert_eq!(type_name(VAR_DICT), "dict");
        assert_eq!(type_name(99), "unknown");
    }

    #[test]
    fn test_vval_constants() {
        assert_eq!(VVAL_FALSE, 0);
        assert_eq!(VVAL_TRUE, 1);
        assert_eq!(VVAL_NONE, 2);
        assert_eq!(VVAL_NULL, 3);
    }
}
