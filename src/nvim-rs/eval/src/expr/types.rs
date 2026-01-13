//! Expression evaluation types.
//!
//! This module provides EvalArg handle, ExprResult enum, and EvalFlags struct.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;
use std::ptr::NonNull;

use super::constants::{EVAL_COMPLETE, EVAL_CONDITIONAL, EVAL_LET, EVAL_SANDBOX, EVAL_TYPVAL};

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to C evalarg_T struct.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct EvalArgHandle(Option<NonNull<core::ffi::c_void>>);

impl EvalArgHandle {
    /// Create a null handle.
    #[must_use]
    pub const fn null() -> Self {
        Self(None)
    }

    /// Create from raw pointer.
    #[must_use]
    pub fn from_ptr(ptr: *mut core::ffi::c_void) -> Self {
        Self(NonNull::new(ptr))
    }

    /// Get raw pointer.
    #[must_use]
    pub fn as_ptr(self) -> *mut core::ffi::c_void {
        self.0
            .map_or(std::ptr::null_mut(), core::ptr::NonNull::as_ptr)
    }

    /// Check if handle is null.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.0.is_none()
    }
}

/// Opaque handle to C typval_T struct.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TypevalHandle(Option<NonNull<core::ffi::c_void>>);

impl TypevalHandle {
    /// Create a null handle.
    #[must_use]
    pub const fn null() -> Self {
        Self(None)
    }

    /// Create from raw pointer.
    #[must_use]
    pub fn from_ptr(ptr: *mut core::ffi::c_void) -> Self {
        Self(NonNull::new(ptr))
    }

    /// Get raw pointer.
    #[must_use]
    pub fn as_ptr(self) -> *mut core::ffi::c_void {
        self.0
            .map_or(std::ptr::null_mut(), core::ptr::NonNull::as_ptr)
    }

    /// Check if handle is null.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.0.is_none()
    }
}

// =============================================================================
// Expression Result Type
// =============================================================================

/// Result type for expression evaluation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExprResult {
    /// Evaluation succeeded.
    #[default]
    Ok = 0,
    /// Syntax error in expression.
    SyntaxError = 1,
    /// Type error (incompatible types).
    TypeError = 2,
    /// Variable not found.
    NotFound = 3,
    /// Division by zero.
    DivByZero = 4,
    /// Recursion limit exceeded.
    RecursionLimit = 5,
    /// Memory allocation failed.
    OutOfMemory = 6,
    /// User interrupt (CTRL-C).
    Interrupted = 7,
    /// Unknown/internal error.
    Unknown = -1,
}

impl ExprResult {
    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::SyntaxError,
            2 => Self::TypeError,
            3 => Self::NotFound,
            4 => Self::DivByZero,
            5 => Self::RecursionLimit,
            6 => Self::OutOfMemory,
            7 => Self::Interrupted,
            _ => Self::Unknown,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if result is OK.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if result is an error.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.is_ok()
    }
}

// =============================================================================
// Evaluation Flags Structure
// =============================================================================

/// Flags controlling expression evaluation behavior.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EvalFlags {
    /// Raw flags value.
    pub flags: c_int,
}

impl EvalFlags {
    /// Create empty flags.
    #[must_use]
    pub const fn new() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(flags: c_int) -> Self {
        Self { flags }
    }

    /// Check if completing.
    #[must_use]
    pub const fn is_completing(&self) -> bool {
        (self.flags & EVAL_COMPLETE) != 0
    }

    /// Check if for :let.
    #[must_use]
    pub const fn is_let(&self) -> bool {
        (self.flags & EVAL_LET) != 0
    }

    /// Check if sandbox mode.
    #[must_use]
    pub const fn is_sandbox(&self) -> bool {
        (self.flags & EVAL_SANDBOX) != 0
    }

    /// Check if conditional (:if/:while).
    #[must_use]
    pub const fn is_conditional(&self) -> bool {
        (self.flags & EVAL_CONDITIONAL) != 0
    }

    /// Check if returning typval.
    #[must_use]
    pub const fn returns_typval(&self) -> bool {
        (self.flags & EVAL_TYPVAL) != 0
    }

    /// Set completing flag.
    pub fn set_completing(&mut self, value: bool) {
        if value {
            self.flags |= EVAL_COMPLETE;
        } else {
            self.flags &= !EVAL_COMPLETE;
        }
    }

    /// Set let flag.
    pub fn set_let(&mut self, value: bool) {
        if value {
            self.flags |= EVAL_LET;
        } else {
            self.flags &= !EVAL_LET;
        }
    }

    /// Set sandbox flag.
    pub fn set_sandbox(&mut self, value: bool) {
        if value {
            self.flags |= EVAL_SANDBOX;
        } else {
            self.flags &= !EVAL_SANDBOX;
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Create null EvalArgHandle.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_arg_null() -> EvalArgHandle {
    EvalArgHandle::null()
}

/// FFI: Check if EvalArgHandle is null.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_arg_is_null(handle: EvalArgHandle) -> c_int {
    c_int::from(handle.is_null())
}

/// FFI: Create null TypevalHandle.
#[unsafe(no_mangle)]
pub extern "C" fn rs_typval_null() -> TypevalHandle {
    TypevalHandle::null()
}

/// FFI: Check if TypevalHandle is null.
#[unsafe(no_mangle)]
pub extern "C" fn rs_typval_is_null(handle: TypevalHandle) -> c_int {
    c_int::from(handle.is_null())
}

/// FFI: Get ExprResult OK value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_expr_result_ok() -> c_int {
    ExprResult::Ok.to_raw()
}

/// FFI: Get ExprResult SyntaxError value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_expr_result_syntax_error() -> c_int {
    ExprResult::SyntaxError.to_raw()
}

/// FFI: Get ExprResult TypeError value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_expr_result_type_error() -> c_int {
    ExprResult::TypeError.to_raw()
}

/// FFI: Check if ExprResult is OK.
#[unsafe(no_mangle)]
pub extern "C" fn rs_expr_result_is_ok(result: c_int) -> c_int {
    c_int::from(ExprResult::from_raw(result).is_ok())
}

/// FFI: Check if ExprResult is error.
#[unsafe(no_mangle)]
pub extern "C" fn rs_expr_result_is_error(result: c_int) -> c_int {
    c_int::from(ExprResult::from_raw(result).is_error())
}

/// FFI: Create EvalFlags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_flags_new() -> c_int {
    EvalFlags::new().flags
}

/// FFI: Check if completing.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_flags_is_completing(flags: c_int) -> c_int {
    c_int::from(EvalFlags::from_raw(flags).is_completing())
}

/// FFI: Check if sandbox.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_flags_is_sandbox(flags: c_int) -> c_int {
    c_int::from(EvalFlags::from_raw(flags).is_sandbox())
}

/// FFI: Check if conditional.
#[unsafe(no_mangle)]
pub extern "C" fn rs_eval_flags_is_conditional(flags: c_int) -> c_int {
    c_int::from(EvalFlags::from_raw(flags).is_conditional())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_arg_handle() {
        let null_handle = EvalArgHandle::null();
        assert!(null_handle.is_null());

        // Test from non-null pointer would require actual memory
    }

    #[test]
    fn test_typval_handle() {
        let null_handle = TypevalHandle::null();
        assert!(null_handle.is_null());
    }

    #[test]
    fn test_expr_result() {
        assert_eq!(ExprResult::Ok.to_raw(), 0);
        assert!(ExprResult::Ok.is_ok());
        assert!(!ExprResult::Ok.is_error());

        assert!(ExprResult::SyntaxError.is_error());
        assert!(!ExprResult::SyntaxError.is_ok());

        assert_eq!(ExprResult::from_raw(0), ExprResult::Ok);
        assert_eq!(ExprResult::from_raw(1), ExprResult::SyntaxError);
        assert_eq!(ExprResult::from_raw(99), ExprResult::Unknown);
    }

    #[test]
    fn test_eval_flags() {
        let mut flags = EvalFlags::new();
        assert!(!flags.is_completing());
        assert!(!flags.is_sandbox());

        flags.set_completing(true);
        assert!(flags.is_completing());

        flags.set_sandbox(true);
        assert!(flags.is_sandbox());

        flags.set_completing(false);
        assert!(!flags.is_completing());
        assert!(flags.is_sandbox()); // still set
    }
}
