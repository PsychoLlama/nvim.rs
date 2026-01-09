//! Core types for user-defined functions.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Function Definition
// =============================================================================

/// Function scope identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FuncScope {
    /// Global function
    Global = 0,
    /// Script-local function (s:)
    Script = 1,
    /// Buffer-local function
    Buffer = 2,
    /// Lambda/anonymous function
    Lambda = 3,
}

impl FuncScope {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Script,
            2 => Self::Buffer,
            3 => Self::Lambda,
            _ => Self::Global,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// Function definition info (metadata only, body stored in C).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuncInfo {
    /// Number of required arguments
    pub min_args: i32,
    /// Maximum arguments (-1 for variadic)
    pub max_args: i32,
    /// Number of optional arguments with defaults
    pub optional_args: i32,
    /// Function scope
    pub scope: i32,
    /// Definition flags
    pub flags: u32,
    /// Script ID where defined
    pub script_id: i32,
    /// Line number where defined
    pub def_lnum: i64,
}

impl Default for FuncInfo {
    fn default() -> Self {
        Self {
            min_args: 0,
            max_args: 0,
            optional_args: 0,
            scope: FuncScope::Global as i32,
            flags: 0,
            script_id: 0,
            def_lnum: 0,
        }
    }
}

impl FuncInfo {
    /// Create new function info.
    pub const fn new(min_args: i32, max_args: i32) -> Self {
        Self {
            min_args,
            max_args,
            optional_args: 0,
            scope: 0,
            flags: 0,
            script_id: 0,
            def_lnum: 0,
        }
    }

    /// Check if function is variadic.
    pub const fn is_variadic(&self) -> bool {
        self.max_args < 0
    }

    /// Check if argument count is valid.
    pub fn is_valid_arg_count(&self, count: i32) -> bool {
        if count < self.min_args {
            return false;
        }
        if self.max_args >= 0 && count > self.max_args {
            return false;
        }
        true
    }
}

/// FFI export: create function info.
#[no_mangle]
pub extern "C" fn rs_userfunc_info_new(min_args: c_int, max_args: c_int) -> FuncInfo {
    FuncInfo::new(min_args, max_args)
}

/// FFI export: check if arg count is valid.
#[no_mangle]
pub extern "C" fn rs_userfunc_valid_arg_count(
    min_args: c_int,
    max_args: c_int,
    count: c_int,
) -> bool {
    let info = FuncInfo::new(min_args, max_args);
    info.is_valid_arg_count(count)
}

// =============================================================================
// Partial Application
// =============================================================================

/// Partial application info (for funcref with bound arguments).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PartialInfo {
    /// Number of bound arguments
    pub bound_count: i32,
    /// Whether self/dict is bound
    pub has_self: bool,
    /// Whether we're calling a method
    pub is_method: bool,
}

impl PartialInfo {
    /// Create new partial info.
    pub const fn new(bound_count: i32, has_self: bool) -> Self {
        Self {
            bound_count,
            has_self,
            is_method: false,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_scope() {
        assert_eq!(FuncScope::from_c_int(0), FuncScope::Global);
        assert_eq!(FuncScope::from_c_int(1), FuncScope::Script);
        assert_eq!(FuncScope::from_c_int(99), FuncScope::Global);
    }

    #[test]
    fn test_func_info() {
        let info = FuncInfo::new(1, 3);
        assert!(info.is_valid_arg_count(1));
        assert!(info.is_valid_arg_count(2));
        assert!(info.is_valid_arg_count(3));
        assert!(!info.is_valid_arg_count(0));
        assert!(!info.is_valid_arg_count(4));

        let variadic = FuncInfo::new(1, -1);
        assert!(variadic.is_variadic());
        assert!(variadic.is_valid_arg_count(1));
        assert!(variadic.is_valid_arg_count(100));
    }
}
