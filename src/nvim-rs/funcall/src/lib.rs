//! Function call infrastructure for VimL.
//!
//! This module provides function call utilities migrated from `src/nvim/eval/`:
//! - Argument parsing and validation
//! - Default argument handling
//! - Variadic function support
//! - Return value propagation
//! - Error/exception handling
//!
//! ## VimL Function Call Semantics
//!
//! VimL functions can be:
//! - Built-in functions (implemented in C/Rust)
//! - User-defined functions (Vimscript or Lua)
//! - Lambda expressions
//! - Partial functions (with bound arguments)
//!
//! ## FFI Pattern
//!
//! These functions provide the infrastructure for C callers to set up and
//! execute function calls with proper argument handling and error propagation.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]

use std::ffi::c_int;

// =============================================================================
// Function Call Result
// =============================================================================

/// Result of a function call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FuncCallResult {
    /// Call succeeded
    Ok = 0,
    /// Function not found
    NotFound = -1,
    /// Wrong number of arguments
    ArgCount = -2,
    /// Invalid argument type
    ArgType = -3,
    /// Invalid argument value
    ArgValue = -4,
    /// Function threw an error
    Error = -5,
    /// Function was interrupted
    Interrupted = -6,
    /// Function aborted (e.g., :finish in script)
    Aborted = -7,
    /// Unknown failure
    Unknown = -100,
}

impl FuncCallResult {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            -1 => Self::NotFound,
            -2 => Self::ArgCount,
            -3 => Self::ArgType,
            -4 => Self::ArgValue,
            -5 => Self::Error,
            -6 => Self::Interrupted,
            -7 => Self::Aborted,
            _ => Self::Unknown,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if call succeeded.
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if this is an error.
    pub const fn is_error(self) -> bool {
        !self.is_ok()
    }
}

// =============================================================================
// Argument Specification
// =============================================================================

/// Specification for a function argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ArgSpec {
    /// Minimum number of arguments
    pub min_args: i32,
    /// Maximum number of arguments (-1 for unlimited/variadic)
    pub max_args: i32,
    /// Number of optional arguments at the end
    pub optional_args: i32,
}

impl ArgSpec {
    /// Create a new argument spec.
    pub const fn new(min: i32, max: i32) -> Self {
        Self {
            min_args: min,
            max_args: max,
            optional_args: if max < 0 { 0 } else { max - min },
        }
    }

    /// Create a spec for a function with exactly n arguments.
    pub const fn exact(n: i32) -> Self {
        Self {
            min_args: n,
            max_args: n,
            optional_args: 0,
        }
    }

    /// Create a spec for a variadic function.
    pub const fn variadic(min: i32) -> Self {
        Self {
            min_args: min,
            max_args: -1,
            optional_args: 0,
        }
    }

    /// Check if argument count is valid.
    pub const fn validate_count(&self, count: i32) -> bool {
        if count < self.min_args {
            return false;
        }
        if self.max_args >= 0 && count > self.max_args {
            return false;
        }
        true
    }

    /// Check if function is variadic.
    pub const fn is_variadic(&self) -> bool {
        self.max_args < 0
    }
}

/// FFI export: create arg spec.
#[no_mangle]
pub extern "C" fn rs_funcall_argspec_new(min: c_int, max: c_int) -> ArgSpec {
    ArgSpec::new(min, max)
}

/// FFI export: validate argument count.
#[no_mangle]
pub extern "C" fn rs_funcall_validate_argcount(spec: ArgSpec, count: c_int) -> bool {
    spec.validate_count(count)
}

// =============================================================================
// Argument Type
// =============================================================================

/// Expected type for a function argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ArgType {
    /// Any type accepted
    Any = 0,
    /// Number
    Number = 1,
    /// Float
    Float = 2,
    /// String
    String = 3,
    /// List
    List = 4,
    /// Dictionary
    Dict = 5,
    /// Funcref
    Func = 6,
    /// Blob
    Blob = 7,
    /// Boolean
    Bool = 8,
    /// Number or Float
    NumericType = 10,
    /// String or Number (coercible to string)
    StringLike = 11,
    /// List or Dict
    Container = 12,
    /// Callable (Funcref, Partial, or Lambda)
    Callable = 13,
    /// Buffer identifier (number or string)
    Buffer = 14,
    /// Window identifier (number)
    Window = 15,
    /// Tab identifier (number)
    Tab = 16,
}

impl ArgType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Any,
            1 => Self::Number,
            2 => Self::Float,
            3 => Self::String,
            4 => Self::List,
            5 => Self::Dict,
            6 => Self::Func,
            7 => Self::Blob,
            8 => Self::Bool,
            10 => Self::NumericType,
            11 => Self::StringLike,
            12 => Self::Container,
            13 => Self::Callable,
            14 => Self::Buffer,
            15 => Self::Window,
            16 => Self::Tab,
            _ => Self::Any,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Function Type
// =============================================================================

/// Type of function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FuncType {
    /// Built-in function (C implementation)
    Builtin = 0,
    /// User-defined function (Vimscript)
    User = 1,
    /// Lambda expression
    Lambda = 2,
    /// Partial function (with bound arguments)
    Partial = 3,
    /// Lua function
    Lua = 4,
    /// :def function (Vim9)
    Def = 5,
}

impl FuncType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Builtin,
            1 => Self::User,
            2 => Self::Lambda,
            3 => Self::Partial,
            4 => Self::Lua,
            5 => Self::Def,
            _ => Self::Builtin,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a user-defined function type.
    pub const fn is_user_defined(self) -> bool {
        matches!(self, Self::User | Self::Lambda | Self::Def)
    }
}

// =============================================================================
// Function Flags
// =============================================================================

/// Flags for function definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncFlags(u32);

impl FuncFlags {
    /// No flags.
    pub const NONE: Self = Self(0);

    /// Function can modify the range (e.g., :call).
    pub const RANGE: Self = Self(1 << 0);

    /// Function receives range as args.
    pub const RANGE_ARGS: Self = Self(1 << 1);

    /// Function aborts on error.
    pub const ABORT: Self = Self(1 << 2);

    /// Function is a closure (captures local scope).
    pub const CLOSURE: Self = Self(1 << 3);

    /// Function is defined with :def (Vim9).
    pub const DEF: Self = Self(1 << 4);

    /// Function is variadic (uses a:000).
    pub const VARARGS: Self = Self(1 << 5);

    /// Function is a dict function (uses dict self).
    pub const DICT: Self = Self(1 << 6);

    /// Function should be profiled.
    pub const PROFILING: Self = Self(1 << 7);

    /// Function is a script-local autoload.
    pub const AUTOLOAD: Self = Self(1 << 8);

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

    /// Combine flags.
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// =============================================================================
// Call Context
// =============================================================================

/// Context for a function call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct CallContext {
    /// Flags for this call.
    pub flags: CallFlags,
    /// First line of range (if applicable).
    pub range_start: i64,
    /// Last line of range (if applicable).
    pub range_end: i64,
    /// Whether a range was specified.
    pub has_range: bool,
}

impl CallContext {
    /// Create a new call context.
    pub const fn new() -> Self {
        Self {
            flags: CallFlags::NONE,
            range_start: 0,
            range_end: 0,
            has_range: false,
        }
    }

    /// Create a call context with range.
    pub const fn with_range(start: i64, end: i64) -> Self {
        Self {
            flags: CallFlags::NONE,
            range_start: start,
            range_end: end,
            has_range: true,
        }
    }
}

impl Default for CallContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Flags for function call context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallFlags(u32);

impl CallFlags {
    /// No special flags.
    pub const NONE: Self = Self(0);

    /// Called with :call (command context).
    pub const COMMAND: Self = Self(1 << 0);

    /// Called from :def function.
    pub const FROM_DEF: Self = Self(1 << 1);

    /// Called with bang (!).
    pub const BANG: Self = Self(1 << 2);

    /// Return value is needed.
    pub const NEED_RETVAL: Self = Self(1 << 3);

    /// Skip argument evaluation (used for try/catch).
    pub const SKIP_EVAL: Self = Self(1 << 4);

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
// Error Handling
// =============================================================================

/// Error information for function calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FuncError {
    /// No error.
    None = 0,
    /// Wrong number of arguments.
    E118WrongArgCount = 118,
    /// Not enough arguments.
    E119NotEnoughArgs = 119,
    /// Function not found.
    E117FuncNotFound = 117,
    /// Invalid argument.
    E474InvalidArg = 474,
    /// Function is being deleted.
    E131CannotDelete = 131,
    /// Cannot use function as method.
    E276CannotUseAsMethod = 276,
}

impl FuncError {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::None,
            118 => Self::E118WrongArgCount,
            119 => Self::E119NotEnoughArgs,
            117 => Self::E117FuncNotFound,
            474 => Self::E474InvalidArg,
            131 => Self::E131CannotDelete,
            276 => Self::E276CannotUseAsMethod,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if there's an error.
    pub const fn is_error(self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// Argument Validation Helpers
// =============================================================================

/// Validate that argument count is within spec.
pub fn validate_arg_count(min: i32, max: i32, actual: i32) -> FuncCallResult {
    if actual < min {
        return FuncCallResult::ArgCount;
    }
    if max >= 0 && actual > max {
        return FuncCallResult::ArgCount;
    }
    FuncCallResult::Ok
}

/// FFI export: validate argument count.
#[no_mangle]
pub extern "C" fn rs_funcall_validate_args(
    min: c_int,
    max: c_int,
    actual: c_int,
) -> FuncCallResult {
    validate_arg_count(min, max, actual)
}

// =============================================================================
// Default Argument Handling
// =============================================================================

/// Default value specification for optional arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DefaultValue {
    /// No default (argument is required).
    None = 0,
    /// Default is 0.
    Zero = 1,
    /// Default is 1.
    One = 2,
    /// Default is -1.
    NegOne = 3,
    /// Default is empty string.
    EmptyString = 4,
    /// Default is empty list.
    EmptyList = 5,
    /// Default is empty dict.
    EmptyDict = 6,
    /// Default is v:null.
    Null = 7,
    /// Default is v:false.
    False = 8,
    /// Default is v:true.
    True = 9,
    /// Default is current buffer.
    CurrentBuffer = 10,
    /// Default is current window.
    CurrentWindow = 11,
    /// Default is current line.
    CurrentLine = 12,
    /// Default is cursor position.
    CursorPos = 13,
}

impl DefaultValue {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::None,
            1 => Self::Zero,
            2 => Self::One,
            3 => Self::NegOne,
            4 => Self::EmptyString,
            5 => Self::EmptyList,
            6 => Self::EmptyDict,
            7 => Self::Null,
            8 => Self::False,
            9 => Self::True,
            10 => Self::CurrentBuffer,
            11 => Self::CurrentWindow,
            12 => Self::CurrentLine,
            13 => Self::CursorPos,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this default needs context to resolve.
    pub const fn needs_context(self) -> bool {
        matches!(
            self,
            Self::CurrentBuffer | Self::CurrentWindow | Self::CurrentLine | Self::CursorPos
        )
    }
}

// =============================================================================
// Return Value Handling
// =============================================================================

/// Specification for function return value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ReturnSpec {
    /// Function returns nothing (void).
    Void = 0,
    /// Function returns a value of any type.
    Any = 1,
    /// Function returns a number.
    Number = 2,
    /// Function returns a string.
    String = 3,
    /// Function returns a list.
    List = 4,
    /// Function returns a dict.
    Dict = 5,
    /// Function returns a float.
    Float = 6,
    /// Function returns a boolean.
    Bool = 7,
    /// Function returns a blob.
    Blob = 8,
}

impl ReturnSpec {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Void,
            1 => Self::Any,
            2 => Self::Number,
            3 => Self::String,
            4 => Self::List,
            5 => Self::Dict,
            6 => Self::Float,
            7 => Self::Bool,
            8 => Self::Blob,
            _ => Self::Any,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if function has a return value.
    pub const fn has_return(self) -> bool {
        !matches!(self, Self::Void)
    }
}

// =============================================================================
// Variadic Argument State
// =============================================================================

/// State for handling variadic arguments (a:000 list).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VarargState {
    /// Index of first variadic argument.
    pub first_vararg: i32,
    /// Total number of arguments provided.
    pub total_args: i32,
    /// Number of variadic arguments.
    pub vararg_count: i32,
}

impl VarargState {
    /// Create a new vararg state.
    pub const fn new(fixed_count: i32, total_args: i32) -> Self {
        Self {
            first_vararg: fixed_count,
            total_args,
            vararg_count: if total_args > fixed_count {
                total_args - fixed_count
            } else {
                0
            },
        }
    }

    /// Check if there are variadic arguments.
    pub const fn has_varargs(&self) -> bool {
        self.vararg_count > 0
    }

    /// Get the index of a variadic argument (0-based within varargs).
    pub const fn vararg_index(&self, idx: i32) -> Option<i32> {
        if idx >= 0 && idx < self.vararg_count {
            Some(self.first_vararg + idx)
        } else {
            None
        }
    }
}

/// FFI export: create vararg state.
#[no_mangle]
pub extern "C" fn rs_funcall_vararg_state(fixed_count: c_int, total_args: c_int) -> VarargState {
    VarargState::new(fixed_count, total_args)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_call_result() {
        assert!(FuncCallResult::Ok.is_ok());
        assert!(!FuncCallResult::NotFound.is_ok());
        assert!(FuncCallResult::ArgCount.is_error());
    }

    #[test]
    fn test_arg_spec() {
        let spec = ArgSpec::new(1, 3);
        assert!(!spec.validate_count(0));
        assert!(spec.validate_count(1));
        assert!(spec.validate_count(2));
        assert!(spec.validate_count(3));
        assert!(!spec.validate_count(4));

        let exact = ArgSpec::exact(2);
        assert!(!exact.validate_count(1));
        assert!(exact.validate_count(2));
        assert!(!exact.validate_count(3));

        let variadic = ArgSpec::variadic(1);
        assert!(variadic.is_variadic());
        assert!(!variadic.validate_count(0));
        assert!(variadic.validate_count(1));
        assert!(variadic.validate_count(100));
    }

    #[test]
    fn test_func_type() {
        assert!(!FuncType::Builtin.is_user_defined());
        assert!(FuncType::User.is_user_defined());
        assert!(FuncType::Lambda.is_user_defined());
        assert!(!FuncType::Lua.is_user_defined());
    }

    #[test]
    fn test_func_flags() {
        let flags = FuncFlags::NONE;
        assert!(!flags.contains(FuncFlags::RANGE));

        let flags = FuncFlags::RANGE.or(FuncFlags::ABORT);
        assert!(flags.contains(FuncFlags::RANGE));
        assert!(flags.contains(FuncFlags::ABORT));
        assert!(!flags.contains(FuncFlags::CLOSURE));
    }

    #[test]
    fn test_call_context() {
        let ctx = CallContext::new();
        assert!(!ctx.has_range);

        let ctx = CallContext::with_range(1, 10);
        assert!(ctx.has_range);
        assert_eq!(ctx.range_start, 1);
        assert_eq!(ctx.range_end, 10);
    }

    #[test]
    fn test_func_error() {
        assert!(!FuncError::None.is_error());
        assert!(FuncError::E117FuncNotFound.is_error());
        assert_eq!(FuncError::E117FuncNotFound.to_c_int(), 117);
    }

    #[test]
    fn test_validate_arg_count() {
        assert!(validate_arg_count(1, 3, 2).is_ok());
        assert!(validate_arg_count(1, 3, 0).is_error());
        assert!(validate_arg_count(1, 3, 4).is_error());
        assert!(validate_arg_count(1, -1, 100).is_ok());
    }

    #[test]
    fn test_default_value() {
        assert!(!DefaultValue::Zero.needs_context());
        assert!(DefaultValue::CurrentBuffer.needs_context());
        assert!(DefaultValue::CursorPos.needs_context());
    }

    #[test]
    fn test_return_spec() {
        assert!(!ReturnSpec::Void.has_return());
        assert!(ReturnSpec::Any.has_return());
        assert!(ReturnSpec::Number.has_return());
    }

    #[test]
    fn test_vararg_state() {
        let state = VarargState::new(2, 5);
        assert!(state.has_varargs());
        assert_eq!(state.vararg_count, 3);
        assert_eq!(state.vararg_index(0), Some(2));
        assert_eq!(state.vararg_index(1), Some(3));
        assert_eq!(state.vararg_index(2), Some(4));
        assert_eq!(state.vararg_index(3), None);

        let state = VarargState::new(2, 2);
        assert!(!state.has_varargs());
    }

    #[test]
    fn test_arg_type() {
        assert_eq!(ArgType::from_c_int(0), ArgType::Any);
        assert_eq!(ArgType::from_c_int(1), ArgType::Number);
        assert_eq!(ArgType::NumericType.to_c_int(), 10);
    }

    #[test]
    fn test_call_flags() {
        let flags = CallFlags::COMMAND;
        assert!(flags.contains(CallFlags::COMMAND));
        assert!(!flags.contains(CallFlags::BANG));
    }
}
