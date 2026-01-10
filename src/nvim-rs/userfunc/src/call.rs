//! Function call infrastructure for VimL.
//!
//! This module implements the infrastructure for calling user-defined functions:
//! - Call stack management
//! - Argument passing and binding
//! - Return value handling
//! - Error propagation

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]

use std::ffi::c_int;

// =============================================================================
// Call Stack
// =============================================================================

/// Maximum depth of function call stack.
pub const MAX_FUNC_DEPTH: c_int = 100;

/// Function call depth tracking.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CallDepth {
    /// Current call depth
    pub current: c_int,
    /// Maximum allowed depth
    pub max: c_int,
}

impl CallDepth {
    /// Create new call depth tracker.
    pub const fn new() -> Self {
        Self {
            current: 0,
            max: MAX_FUNC_DEPTH,
        }
    }

    /// Check if we can make another call.
    pub const fn can_call(&self) -> bool {
        self.current < self.max
    }

    /// Push a call (increment depth).
    pub fn push(&mut self) -> bool {
        if self.can_call() {
            self.current += 1;
            true
        } else {
            false
        }
    }

    /// Pop a call (decrement depth).
    pub fn pop(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    /// Get current depth.
    pub const fn depth(&self) -> c_int {
        self.current
    }

    /// Check if at top level.
    pub const fn is_top_level(&self) -> bool {
        self.current == 0
    }
}

/// FFI export: create call depth.
#[no_mangle]
pub extern "C" fn rs_call_depth_new() -> CallDepth {
    CallDepth::new()
}

/// FFI export: check if can call.
#[no_mangle]
pub extern "C" fn rs_call_depth_can_call(depth: *const CallDepth) -> bool {
    if depth.is_null() {
        return false;
    }
    unsafe { (*depth).can_call() }
}

/// FFI export: push call.
#[no_mangle]
pub extern "C" fn rs_call_depth_push(depth: *mut CallDepth) -> bool {
    if depth.is_null() {
        return false;
    }
    unsafe { (*depth).push() }
}

/// FFI export: pop call.
#[no_mangle]
pub extern "C" fn rs_call_depth_pop(depth: *mut CallDepth) {
    if !depth.is_null() {
        unsafe { (*depth).pop() }
    }
}

// =============================================================================
// Function Call State
// =============================================================================

/// State for a function call.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuncCallState {
    /// Number of arguments passed
    pub argcount: c_int,
    /// Expected number of arguments (minimum)
    pub min_args: c_int,
    /// Maximum arguments (or -1 for variadic)
    pub max_args: c_int,
    /// Whether function has 'dict' attribute
    pub is_dict_func: bool,
    /// Whether function has 'range' attribute
    pub has_range: bool,
    /// Line number for range start (if has_range)
    pub firstline: c_int,
    /// Line number for range end (if has_range)
    pub lastline: c_int,
    /// Whether an error occurred during call
    pub error: bool,
    /// Whether function returned a value
    pub has_return: bool,
}

impl FuncCallState {
    /// Create new call state.
    pub const fn new(argcount: c_int, min_args: c_int, max_args: c_int) -> Self {
        Self {
            argcount,
            min_args,
            max_args,
            is_dict_func: false,
            has_range: false,
            firstline: 0,
            lastline: 0,
            error: false,
            has_return: false,
        }
    }

    /// Check if argument count is valid.
    pub const fn args_valid(&self) -> bool {
        if self.argcount < self.min_args {
            return false;
        }
        if self.max_args >= 0 && self.argcount > self.max_args {
            return false;
        }
        true
    }

    /// Get number of missing required args.
    pub const fn missing_args(&self) -> c_int {
        if self.argcount < self.min_args {
            self.min_args - self.argcount
        } else {
            0
        }
    }

    /// Get number of extra args.
    pub const fn extra_args(&self) -> c_int {
        if self.max_args >= 0 && self.argcount > self.max_args {
            self.argcount - self.max_args
        } else {
            0
        }
    }

    /// Set range values.
    pub fn set_range(&mut self, firstline: c_int, lastline: c_int) {
        self.has_range = true;
        self.firstline = firstline;
        self.lastline = lastline;
    }

    /// Mark as error.
    pub fn set_error(&mut self) {
        self.error = true;
    }

    /// Mark as having return value.
    pub fn set_returned(&mut self) {
        self.has_return = true;
    }
}

impl Default for FuncCallState {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

/// FFI export: create call state.
#[no_mangle]
pub extern "C" fn rs_func_call_state_new(
    argcount: c_int,
    min_args: c_int,
    max_args: c_int,
) -> FuncCallState {
    FuncCallState::new(argcount, min_args, max_args)
}

/// FFI export: check args valid.
#[no_mangle]
pub extern "C" fn rs_func_call_args_valid(state: *const FuncCallState) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { (*state).args_valid() }
}

/// FFI export: get missing args count.
#[no_mangle]
pub extern "C" fn rs_func_call_missing_args(state: *const FuncCallState) -> c_int {
    if state.is_null() {
        return 0;
    }
    unsafe { (*state).missing_args() }
}

// =============================================================================
// Argument Binding
// =============================================================================

/// Mode for handling extra arguments.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraArgsMode {
    /// Error on extra args
    Error = 0,
    /// Ignore extra args
    Ignore = 1,
    /// Collect extra args into list (variadic)
    Collect = 2,
}

impl ExtraArgsMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Error),
            1 => Some(Self::Ignore),
            2 => Some(Self::Collect),
            _ => None,
        }
    }
}

/// Argument binding state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ArgBindState {
    /// Current argument being bound
    pub current: c_int,
    /// Total arguments to bind
    pub total: c_int,
    /// Parameters bound so far
    pub bound: c_int,
    /// Whether binding failed
    pub failed: bool,
    /// Reason for failure (if failed)
    pub error_code: c_int,
}

impl ArgBindState {
    /// Create new binding state.
    pub const fn new(total: c_int) -> Self {
        Self {
            current: 0,
            total,
            bound: 0,
            failed: false,
            error_code: 0,
        }
    }

    /// Check if there are more args to bind.
    pub const fn has_more(&self) -> bool {
        !self.failed && self.current < self.total
    }

    /// Advance to next argument.
    pub fn advance(&mut self) {
        self.current += 1;
        self.bound += 1;
    }

    /// Mark as failed with error code.
    pub fn fail(&mut self, code: c_int) {
        self.failed = true;
        self.error_code = code;
    }
}

/// FFI export: create arg bind state.
#[no_mangle]
pub extern "C" fn rs_arg_bind_state_new(total: c_int) -> ArgBindState {
    ArgBindState::new(total)
}

/// FFI export: check if more args.
#[no_mangle]
pub extern "C" fn rs_arg_bind_has_more(state: *const ArgBindState) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { (*state).has_more() }
}

/// FFI export: advance binding.
#[no_mangle]
pub extern "C" fn rs_arg_bind_advance(state: *mut ArgBindState) {
    if !state.is_null() {
        unsafe { (*state).advance() }
    }
}

/// FFI export: fail binding.
#[no_mangle]
pub extern "C" fn rs_arg_bind_fail(state: *mut ArgBindState, code: c_int) {
    if !state.is_null() {
        unsafe { (*state).fail(code) }
    }
}

// =============================================================================
// Return Value Handling
// =============================================================================

/// Return value state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReturnState {
    /// Whether a return statement was executed
    pub returned: bool,
    /// Whether the return had a value
    pub has_value: bool,
    /// Error code if return failed
    pub error_code: c_int,
}

impl ReturnState {
    /// Create empty return state.
    pub const fn new() -> Self {
        Self {
            returned: false,
            has_value: false,
            error_code: 0,
        }
    }

    /// Create returned state with value.
    pub const fn with_value() -> Self {
        Self {
            returned: true,
            has_value: true,
            error_code: 0,
        }
    }

    /// Create returned state without value.
    pub const fn without_value() -> Self {
        Self {
            returned: true,
            has_value: false,
            error_code: 0,
        }
    }

    /// Check if function has completed (returned).
    pub const fn is_complete(&self) -> bool {
        self.returned
    }
}

/// FFI export: create return state.
#[no_mangle]
pub extern "C" fn rs_return_state_new() -> ReturnState {
    ReturnState::new()
}

/// FFI export: create return with value.
#[no_mangle]
pub extern "C" fn rs_return_state_with_value() -> ReturnState {
    ReturnState::with_value()
}

/// FFI export: check if complete.
#[no_mangle]
pub extern "C" fn rs_return_state_is_complete(state: *const ReturnState) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { (*state).is_complete() }
}

// =============================================================================
// Error Handling
// =============================================================================

/// Function call error types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncCallError {
    /// No error
    None = 0,
    /// Too few arguments
    TooFewArgs = 1,
    /// Too many arguments
    TooManyArgs = 2,
    /// Invalid argument type
    InvalidArgType = 3,
    /// Stack overflow (too many nested calls)
    StackOverflow = 4,
    /// Function not found
    NotFound = 5,
    /// Not a function
    NotFunction = 6,
    /// Recursive call not allowed
    Recursive = 7,
    /// Function aborted (abort flag)
    Aborted = 8,
    /// Interrupt received
    Interrupted = 9,
}

impl FuncCallError {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::None),
            1 => Some(Self::TooFewArgs),
            2 => Some(Self::TooManyArgs),
            3 => Some(Self::InvalidArgType),
            4 => Some(Self::StackOverflow),
            5 => Some(Self::NotFound),
            6 => Some(Self::NotFunction),
            7 => Some(Self::Recursive),
            8 => Some(Self::Aborted),
            9 => Some(Self::Interrupted),
            _ => None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is an error.
    pub const fn is_error(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// FFI export: check if error.
#[no_mangle]
pub extern "C" fn rs_func_error_is_error(code: c_int) -> bool {
    FuncCallError::from_c_int(code)
        .map(|e| e.is_error())
        .unwrap_or(true)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_depth() {
        let mut depth = CallDepth::new();
        assert!(depth.is_top_level());
        assert!(depth.can_call());

        assert!(depth.push());
        assert!(!depth.is_top_level());
        assert_eq!(depth.depth(), 1);

        depth.pop();
        assert!(depth.is_top_level());
    }

    #[test]
    fn test_call_depth_overflow() {
        let mut depth = CallDepth::new();
        for _ in 0..MAX_FUNC_DEPTH {
            assert!(depth.push());
        }
        assert!(!depth.can_call());
        assert!(!depth.push());
    }

    #[test]
    fn test_func_call_state() {
        // Normal case
        let state = FuncCallState::new(3, 2, 5);
        assert!(state.args_valid());
        assert_eq!(state.missing_args(), 0);
        assert_eq!(state.extra_args(), 0);

        // Too few args
        let state = FuncCallState::new(1, 2, 5);
        assert!(!state.args_valid());
        assert_eq!(state.missing_args(), 1);

        // Too many args
        let state = FuncCallState::new(6, 2, 5);
        assert!(!state.args_valid());
        assert_eq!(state.extra_args(), 1);

        // Variadic (max_args = -1)
        let state = FuncCallState::new(100, 1, -1);
        assert!(state.args_valid());
        assert_eq!(state.extra_args(), 0);
    }

    #[test]
    fn test_func_call_state_range() {
        let mut state = FuncCallState::new(0, 0, 0);
        assert!(!state.has_range);

        state.set_range(10, 20);
        assert!(state.has_range);
        assert_eq!(state.firstline, 10);
        assert_eq!(state.lastline, 20);
    }

    #[test]
    fn test_extra_args_mode() {
        assert_eq!(ExtraArgsMode::from_c_int(0), Some(ExtraArgsMode::Error));
        assert_eq!(ExtraArgsMode::from_c_int(2), Some(ExtraArgsMode::Collect));
        assert_eq!(ExtraArgsMode::from_c_int(99), None);
    }

    #[test]
    fn test_arg_bind_state() {
        let mut state = ArgBindState::new(3);
        assert!(state.has_more());
        assert_eq!(state.current, 0);

        state.advance();
        assert!(state.has_more());
        assert_eq!(state.current, 1);
        assert_eq!(state.bound, 1);

        state.advance();
        state.advance();
        assert!(!state.has_more());
    }

    #[test]
    fn test_arg_bind_fail() {
        let mut state = ArgBindState::new(3);
        state.fail(3);
        assert!(state.failed);
        assert_eq!(state.error_code, 3);
        assert!(!state.has_more());
    }

    #[test]
    fn test_return_state() {
        let empty = ReturnState::new();
        assert!(!empty.is_complete());

        let with_val = ReturnState::with_value();
        assert!(with_val.is_complete());
        assert!(with_val.has_value);

        let without_val = ReturnState::without_value();
        assert!(without_val.is_complete());
        assert!(!without_val.has_value);
    }

    #[test]
    fn test_func_call_error() {
        assert!(!FuncCallError::None.is_error());
        assert!(FuncCallError::TooFewArgs.is_error());
        assert!(FuncCallError::StackOverflow.is_error());

        assert_eq!(
            FuncCallError::from_c_int(1),
            Some(FuncCallError::TooFewArgs)
        );
        assert_eq!(FuncCallError::TooManyArgs.to_c_int(), 2);
    }
}
