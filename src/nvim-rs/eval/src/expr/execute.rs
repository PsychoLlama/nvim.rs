//! Expression execution helpers.
//!
//! This module provides helpers for executing VimL expressions:
//! call_vim_function, eval_expr_typval, ex_let/unlet

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Execution Mode Constants
// =============================================================================

/// Normal execution mode.
pub const EXEC_NORMAL: c_int = 0;
/// Silent execution (no messages).
pub const EXEC_SILENT: c_int = 1;
/// Execution in sandbox.
pub const EXEC_SANDBOX: c_int = 2;
/// Execution with error handling.
pub const EXEC_CATCH_ERRORS: c_int = 4;

// =============================================================================
// Let Command Types
// =============================================================================

/// Simple assignment (:let var = expr).
pub const LET_ASSIGN: c_int = 0;
/// Add assignment (:let var += expr).
pub const LET_ADD: c_int = 1;
/// Subtract assignment (:let var -= expr).
pub const LET_SUB: c_int = 2;
/// Append assignment (:let var .= expr).
pub const LET_CONCAT: c_int = 3;
/// List unpack (:let [a, b] = list).
pub const LET_UNPACK: c_int = 4;
/// Environment variable (:let $VAR = expr).
pub const LET_ENV: c_int = 5;
/// Register (:let @r = expr).
pub const LET_REG: c_int = 6;
/// Option (:let &opt = expr).
pub const LET_OPTION: c_int = 7;

// =============================================================================
// Unlet Flags
// =============================================================================

/// Force unlet (! modifier).
pub const UNLET_FORCE: c_int = 0x01;
/// Silent unlet (no error if not exists).
pub const UNLET_SILENT: c_int = 0x02;

// =============================================================================
// Function Call Flags
// =============================================================================

/// Function is built-in.
pub const FUNC_BUILTIN: c_int = 0x01;
/// Function is autoloaded.
pub const FUNC_AUTOLOAD: c_int = 0x02;
/// Function is a method call.
pub const FUNC_METHOD: c_int = 0x04;
/// Function is script-local.
pub const FUNC_SCRIPT: c_int = 0x08;

// =============================================================================
// Execution Helpers
// =============================================================================

/// Get let type from operator string.
fn get_let_type(op: u8) -> c_int {
    match op {
        b'+' => LET_ADD,
        b'-' => LET_SUB,
        b'.' => LET_CONCAT,
        _ => LET_ASSIGN, // '=' and default
    }
}

/// Check if let type is compound assignment.
fn is_compound_let(let_type: c_int) -> bool {
    matches!(let_type, LET_ADD | LET_SUB | LET_CONCAT)
}

/// Check if target is special (env, reg, option).
fn is_special_let_target(let_type: c_int) -> bool {
    matches!(let_type, LET_ENV | LET_REG | LET_OPTION)
}

/// Check if execution mode is silent.
fn is_silent_exec(mode: c_int) -> bool {
    (mode & EXEC_SILENT) != 0
}

/// Check if execution is sandboxed.
fn is_sandbox_exec(mode: c_int) -> bool {
    (mode & EXEC_SANDBOX) != 0
}

/// Check if function is builtin.
fn is_builtin_func(flags: c_int) -> bool {
    (flags & FUNC_BUILTIN) != 0
}

/// Check if function is autoload.
fn is_autoload_func(flags: c_int) -> bool {
    (flags & FUNC_AUTOLOAD) != 0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get EXEC_NORMAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_normal_mode() -> c_int {
    EXEC_NORMAL
}

/// FFI: Get EXEC_SILENT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_silent_mode() -> c_int {
    EXEC_SILENT
}

/// FFI: Get EXEC_SANDBOX constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_sandbox_mode() -> c_int {
    EXEC_SANDBOX
}

/// FFI: Get LET_ASSIGN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_assign() -> c_int {
    LET_ASSIGN
}

/// FFI: Get LET_ADD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_add() -> c_int {
    LET_ADD
}

/// FFI: Get LET_SUB constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_sub() -> c_int {
    LET_SUB
}

/// FFI: Get LET_CONCAT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_concat() -> c_int {
    LET_CONCAT
}

/// FFI: Get LET_UNPACK constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_unpack() -> c_int {
    LET_UNPACK
}

/// FFI: Get LET_ENV constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_env() -> c_int {
    LET_ENV
}

/// FFI: Get LET_REG constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_reg() -> c_int {
    LET_REG
}

/// FFI: Get LET_OPTION constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_let_option() -> c_int {
    LET_OPTION
}

/// FFI: Get let type from operator.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_let_type(op: c_int) -> c_int {
    get_let_type(op as u8)
}

/// FFI: Check if compound let.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_compound_let(let_type: c_int) -> c_int {
    c_int::from(is_compound_let(let_type))
}

/// FFI: Check if special let target.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_special_let_target(let_type: c_int) -> c_int {
    c_int::from(is_special_let_target(let_type))
}

/// FFI: Check if silent execution.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_silent_exec(mode: c_int) -> c_int {
    c_int::from(is_silent_exec(mode))
}

/// FFI: Check if sandbox execution.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_sandbox_exec(mode: c_int) -> c_int {
    c_int::from(is_sandbox_exec(mode))
}

/// FFI: Get UNLET_FORCE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_unlet_force() -> c_int {
    UNLET_FORCE
}

/// FFI: Get UNLET_SILENT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_unlet_silent() -> c_int {
    UNLET_SILENT
}

/// FFI: Get FUNC_BUILTIN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_builtin() -> c_int {
    FUNC_BUILTIN
}

/// FFI: Get FUNC_AUTOLOAD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_func_autoload() -> c_int {
    FUNC_AUTOLOAD
}

/// FFI: Check if builtin function.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_builtin_func(flags: c_int) -> c_int {
    c_int::from(is_builtin_func(flags))
}

/// FFI: Check if autoload function.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_autoload_func(flags: c_int) -> c_int {
    c_int::from(is_autoload_func(flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_mode_constants() {
        assert_eq!(EXEC_NORMAL, 0);
        assert_eq!(EXEC_SILENT, 1);
        assert_eq!(EXEC_SANDBOX, 2);
    }

    #[test]
    fn test_let_type_constants() {
        assert_eq!(LET_ASSIGN, 0);
        assert_eq!(LET_ADD, 1);
        assert_eq!(LET_SUB, 2);
        assert_eq!(LET_CONCAT, 3);
    }

    #[test]
    fn test_get_let_type() {
        assert_eq!(get_let_type(b'='), LET_ASSIGN);
        assert_eq!(get_let_type(b'+'), LET_ADD);
        assert_eq!(get_let_type(b'-'), LET_SUB);
        assert_eq!(get_let_type(b'.'), LET_CONCAT);
        assert_eq!(get_let_type(b'x'), LET_ASSIGN); // default
    }

    #[test]
    fn test_is_compound_let() {
        assert!(is_compound_let(LET_ADD));
        assert!(is_compound_let(LET_SUB));
        assert!(is_compound_let(LET_CONCAT));
        assert!(!is_compound_let(LET_ASSIGN));
        assert!(!is_compound_let(LET_UNPACK));
    }

    #[test]
    fn test_is_special_let_target() {
        assert!(is_special_let_target(LET_ENV));
        assert!(is_special_let_target(LET_REG));
        assert!(is_special_let_target(LET_OPTION));
        assert!(!is_special_let_target(LET_ASSIGN));
    }

    #[test]
    fn test_exec_mode_checks() {
        assert!(!is_silent_exec(EXEC_NORMAL));
        assert!(is_silent_exec(EXEC_SILENT));
        assert!(is_silent_exec(EXEC_SILENT | EXEC_SANDBOX));

        assert!(!is_sandbox_exec(EXEC_NORMAL));
        assert!(is_sandbox_exec(EXEC_SANDBOX));
    }

    #[test]
    fn test_func_flags() {
        assert!(is_builtin_func(FUNC_BUILTIN));
        assert!(!is_builtin_func(FUNC_AUTOLOAD));
        assert!(is_autoload_func(FUNC_AUTOLOAD));
        assert!(!is_autoload_func(FUNC_BUILTIN));
    }
}
