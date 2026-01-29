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
// Additional FFI Exports (E8)
// =============================================================================

/// FFI: Get FUNC_FLAG_METHOD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_flag_method() -> c_int {
    FUNC_FLAG_METHOD
}

/// FFI: Get FUNC_FLAG_SCRIPT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_flag_script() -> c_int {
    FUNC_FLAG_SCRIPT
}

/// FFI: Get FUNC_FLAG_ABORT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_flag_abort() -> c_int {
    FUNC_FLAG_ABORT
}

/// FFI: Get CALL_EXCMD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_call_excmd() -> c_int {
    CALL_EXCMD
}

/// FFI: Check if function is script-local.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_is_script_local(flags: c_int) -> c_int {
    c_int::from((flags & FUNC_FLAG_SCRIPT) != 0)
}

/// FFI: Check if function aborts on error.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_aborts_on_error(flags: c_int) -> c_int {
    c_int::from((flags & FUNC_FLAG_ABORT) != 0)
}

/// FFI: Combine function flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_combine_func_flags(flag1: c_int, flag2: c_int) -> c_int {
    flag1 | flag2
}

/// FFI: Check if function flags contain a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from((flags & flag) != 0)
}

/// FFI: Clear a specific flag from function flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_clear_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

/// FFI: Set a specific flag in function flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_set_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// FFI: Check if function type is valid (not unknown).
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_type_is_valid(func_type: c_int) -> c_int {
    c_int::from(func_type != FUNC_UNKNOWN)
}

/// FFI: Check if function type is callable.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_type_is_callable(func_type: c_int) -> c_int {
    c_int::from(matches!(
        func_type,
        FUNC_BUILTIN | FUNC_USER | FUNC_LAMBDA | FUNC_PARTIAL
    ))
}

/// FFI: Check if call is a method call.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_method_call(call_type: c_int) -> c_int {
    c_int::from(call_type == CALL_METHOD)
}

/// FFI: Check if call is from :call command.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_excmd_call(call_type: c_int) -> c_int {
    c_int::from(call_type == CALL_EXCMD)
}

/// FFI: Get minimum args for function (0 = no bound args).
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_min_args(has_varargs: c_int, defined_args: c_int) -> c_int {
    if has_varargs != 0 {
        0
    } else {
        defined_args
    }
}

/// FFI: Check if arg count is valid for function.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_argc_valid(argc: c_int, min_args: c_int, has_varargs: c_int) -> c_int {
    if argc < min_args {
        return 0;
    }
    // With varargs, any count >= min is valid
    // Without varargs, caller should also check max
    c_int::from(has_varargs != 0 || argc >= min_args)
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
