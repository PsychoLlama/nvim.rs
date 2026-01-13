//! Function reference operations.
//!
//! This module provides helpers for funcref operations:
//! funcref allocation, partial creation, method calls

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;
use std::ptr::NonNull;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a ufunc_T (user function) structure.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct FuncHandle(NonNull<std::ffi::c_void>);

/// Opaque handle to a partial_T structure.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PartialHandle(NonNull<std::ffi::c_void>);

// =============================================================================
// Function Type Constants
// =============================================================================

/// Unknown function type.
pub const FUNC_UNKNOWN: c_int = 0;
/// Built-in function.
pub const FUNC_BUILTIN: c_int = 1;
/// User-defined function.
pub const FUNC_USER: c_int = 2;
/// Lambda function.
pub const FUNC_LAMBDA: c_int = 3;
/// Partial function (funcref with bound args).
pub const FUNC_PARTIAL: c_int = 4;

// =============================================================================
// Function Flags
// =============================================================================

/// Function is a closure.
pub const FUNC_FLAG_CLOSURE: c_int = 0x01;
/// Function is a method.
pub const FUNC_FLAG_METHOD: c_int = 0x02;
/// Function is script-local.
pub const FUNC_FLAG_SCRIPT: c_int = 0x04;
/// Function accepts varargs.
pub const FUNC_FLAG_VARARGS: c_int = 0x08;
/// Function is dict function.
pub const FUNC_FLAG_DICT: c_int = 0x10;
/// Function is abort on error.
pub const FUNC_FLAG_ABORT: c_int = 0x20;

// =============================================================================
// Call Flags
// =============================================================================

/// Normal function call.
pub const CALL_NORMAL: c_int = 0;
/// Call as method (obj.method()).
pub const CALL_METHOD: c_int = 1;
/// Call with :call command.
pub const CALL_EXCMD: c_int = 2;

// =============================================================================
// Function Helpers
// =============================================================================

/// Check if function name is a builtin name (starts with lowercase or digit).
fn is_builtin_name(first_char: u8) -> bool {
    first_char.is_ascii_lowercase() || first_char.is_ascii_digit()
}

/// Check if function name is user-defined (starts with uppercase or s:).
fn is_user_func_name(first_char: u8) -> bool {
    first_char.is_ascii_uppercase()
}

/// Check if function name is lambda (<lambda>N).
fn is_lambda_name(first_char: u8) -> bool {
    first_char == b'<'
}

/// Get function type from name.
fn get_func_type_from_name(first_char: u8) -> c_int {
    if is_lambda_name(first_char) {
        FUNC_LAMBDA
    } else if is_builtin_name(first_char) {
        FUNC_BUILTIN
    } else if is_user_func_name(first_char) {
        FUNC_USER
    } else {
        FUNC_UNKNOWN
    }
}

/// Check if function is a closure.
fn is_closure(flags: c_int) -> bool {
    (flags & FUNC_FLAG_CLOSURE) != 0
}

/// Check if function is a method.
fn is_method(flags: c_int) -> bool {
    (flags & FUNC_FLAG_METHOD) != 0
}

/// Check if function accepts varargs.
fn has_varargs(flags: c_int) -> bool {
    (flags & FUNC_FLAG_VARARGS) != 0
}

/// Check if function is dict function.
fn is_dict_func(flags: c_int) -> bool {
    (flags & FUNC_FLAG_DICT) != 0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get FUNC_UNKNOWN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_unknown_type() -> c_int {
    FUNC_UNKNOWN
}

/// FFI: Get FUNC_BUILTIN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_builtin_type() -> c_int {
    FUNC_BUILTIN
}

/// FFI: Get FUNC_USER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_user_type() -> c_int {
    FUNC_USER
}

/// FFI: Get FUNC_LAMBDA constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_lambda_type() -> c_int {
    FUNC_LAMBDA
}

/// FFI: Get FUNC_PARTIAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_partial_type() -> c_int {
    FUNC_PARTIAL
}

/// FFI: Get FUNC_FLAG_CLOSURE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_flag_closure() -> c_int {
    FUNC_FLAG_CLOSURE
}

/// FFI: Get FUNC_FLAG_VARARGS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_flag_varargs() -> c_int {
    FUNC_FLAG_VARARGS
}

/// FFI: Get FUNC_FLAG_DICT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_flag_dict() -> c_int {
    FUNC_FLAG_DICT
}

/// FFI: Get CALL_NORMAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_call_normal() -> c_int {
    CALL_NORMAL
}

/// FFI: Get CALL_METHOD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_call_method() -> c_int {
    CALL_METHOD
}

/// FFI: Check if builtin name.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_builtin_name(first_char: c_int) -> c_int {
    c_int::from(is_builtin_name(first_char as u8))
}

/// FFI: Check if user function name.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_user_func_name(first_char: c_int) -> c_int {
    c_int::from(is_user_func_name(first_char as u8))
}

/// FFI: Check if lambda name.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_lambda_name(first_char: c_int) -> c_int {
    c_int::from(is_lambda_name(first_char as u8))
}

/// FFI: Get function type from name.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_func_type_from_name(first_char: c_int) -> c_int {
    get_func_type_from_name(first_char as u8)
}

/// FFI: Check if function is closure.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_is_closure(flags: c_int) -> c_int {
    c_int::from(is_closure(flags))
}

/// FFI: Check if function is method.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_is_method(flags: c_int) -> c_int {
    c_int::from(is_method(flags))
}

/// FFI: Check if function has varargs.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_has_varargs(flags: c_int) -> c_int {
    c_int::from(has_varargs(flags))
}

/// FFI: Check if function is dict function.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_is_dict_func(flags: c_int) -> c_int {
    c_int::from(is_dict_func(flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
mod tests {
    use super::*;

    #[test]
    fn test_func_type_constants() {
        assert_eq!(FUNC_UNKNOWN, 0);
        assert_eq!(FUNC_BUILTIN, 1);
        assert_eq!(FUNC_USER, 2);
        assert_eq!(FUNC_LAMBDA, 3);
        assert_eq!(FUNC_PARTIAL, 4);
    }

    #[test]
    fn test_func_flags() {
        assert_eq!(FUNC_FLAG_CLOSURE, 0x01);
        assert_eq!(FUNC_FLAG_METHOD, 0x02);
        assert_eq!(FUNC_FLAG_VARARGS, 0x08);
    }

    #[test]
    fn test_call_constants() {
        assert_eq!(CALL_NORMAL, 0);
        assert_eq!(CALL_METHOD, 1);
    }

    #[test]
    fn test_is_builtin_name() {
        assert!(is_builtin_name(b'a'));
        assert!(is_builtin_name(b'z'));
        assert!(is_builtin_name(b'0'));
        assert!(!is_builtin_name(b'A'));
        assert!(!is_builtin_name(b'<'));
    }

    #[test]
    fn test_is_user_func_name() {
        assert!(is_user_func_name(b'A'));
        assert!(is_user_func_name(b'Z'));
        assert!(!is_user_func_name(b'a'));
        assert!(!is_user_func_name(b'<'));
    }

    #[test]
    fn test_is_lambda_name() {
        assert!(is_lambda_name(b'<'));
        assert!(!is_lambda_name(b'a'));
        assert!(!is_lambda_name(b'A'));
    }

    #[test]
    fn test_get_func_type_from_name() {
        assert_eq!(get_func_type_from_name(b'<'), FUNC_LAMBDA);
        assert_eq!(get_func_type_from_name(b'a'), FUNC_BUILTIN);
        assert_eq!(get_func_type_from_name(b'A'), FUNC_USER);
        assert_eq!(get_func_type_from_name(b'_'), FUNC_UNKNOWN);
    }

    #[test]
    fn test_func_flag_checks() {
        assert!(is_closure(FUNC_FLAG_CLOSURE));
        assert!(!is_closure(0));

        assert!(is_method(FUNC_FLAG_METHOD));
        assert!(!is_method(FUNC_FLAG_CLOSURE));

        assert!(has_varargs(FUNC_FLAG_VARARGS));
        assert!(!has_varargs(0));

        assert!(is_dict_func(FUNC_FLAG_DICT));
        assert!(!is_dict_func(FUNC_FLAG_METHOD));
    }
}
