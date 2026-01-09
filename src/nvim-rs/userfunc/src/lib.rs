//! User-defined functions for VimL.
//!
//! This module implements user-defined function support from `src/nvim/eval/userfunc.c`:
//! - Function definition parsing and storage
//! - Parameter binding (positional, optional, variadic, dict)
//! - Closure support with captured variables
//! - Lambda expressions
//! - Function references (Funcref type)
//!
//! ## Module Structure
//!
//! - `types`: Core types for function definitions
//! - `params`: Parameter parsing and validation
//! - `closure`: Closure and captured variable handling
//! - `funcref`: Function reference types

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::doc_markdown)]

pub mod closure;
pub mod funcref;
pub mod params;
pub mod types;

pub use closure::*;
pub use funcref::*;
pub use params::*;
pub use types::*;

use std::ffi::c_int;

// =============================================================================
// Function Definition Flags
// =============================================================================

/// Flags for function definitions.
#[derive(Debug, Clone, Copy, Default)]
pub struct FuncDefFlags {
    /// Function is a closure (captures variables)
    pub is_closure: bool,
    /// Function was defined in a script
    pub is_script: bool,
    /// Function is a dict function (has self)
    pub is_dict: bool,
    /// Function has range specifier
    pub has_range: bool,
    /// Function has abort flag (abort on error)
    pub abort_on_error: bool,
    /// Function is variadic (...)
    pub is_variadic: bool,
    /// Function is a lambda
    pub is_lambda: bool,
    /// Function is a builtin (not user-defined)
    pub is_builtin: bool,
}

impl FuncDefFlags {
    /// Create from bit flags.
    pub const fn from_bits(bits: u32) -> Self {
        Self {
            is_closure: bits & 0x01 != 0,
            is_script: bits & 0x02 != 0,
            is_dict: bits & 0x04 != 0,
            has_range: bits & 0x08 != 0,
            abort_on_error: bits & 0x10 != 0,
            is_variadic: bits & 0x20 != 0,
            is_lambda: bits & 0x40 != 0,
            is_builtin: bits & 0x80 != 0,
        }
    }

    /// Convert to bit flags.
    pub const fn to_bits(&self) -> u32 {
        let mut bits = 0u32;
        if self.is_closure {
            bits |= 0x01;
        }
        if self.is_script {
            bits |= 0x02;
        }
        if self.is_dict {
            bits |= 0x04;
        }
        if self.has_range {
            bits |= 0x08;
        }
        if self.abort_on_error {
            bits |= 0x10;
        }
        if self.is_variadic {
            bits |= 0x20;
        }
        if self.is_lambda {
            bits |= 0x40;
        }
        if self.is_builtin {
            bits |= 0x80;
        }
        bits
    }
}

/// FFI export: create flags from bits.
#[no_mangle]
pub extern "C" fn rs_userfunc_flags_from_bits(bits: u32) -> u32 {
    // Just pass through, but this validates the conversion
    FuncDefFlags::from_bits(bits).to_bits()
}

// =============================================================================
// Function Name Validation
// =============================================================================

/// Check if a character is valid for a function name start.
///
/// VimL function names must start with an uppercase letter or underscore,
/// or be a script-local function starting with 's:'.
pub const fn is_valid_func_name_start(c: u8) -> bool {
    c.is_ascii_uppercase() || c == b'_'
}

/// Check if a character is valid in a function name.
pub const fn is_valid_func_name_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Validate a function name.
///
/// Valid function names:
/// - Start with uppercase or underscore
/// - Contain only alphanumeric and underscore
/// - Script-local functions can start with 's:'
pub fn is_valid_func_name(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }

    // Check for script-local prefix
    let check_name = if name.len() > 2 && name[0] == b's' && name[1] == b':' {
        &name[2..]
    } else {
        name
    };

    if check_name.is_empty() {
        return false;
    }

    // First character must be uppercase or underscore
    if !is_valid_func_name_start(check_name[0]) {
        return false;
    }

    // Rest must be alphanumeric or underscore
    check_name
        .iter()
        .all(|&c| is_valid_func_name_char(c))
}

/// FFI export: validate function name.
///
/// # Safety
/// - `name` must be a valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_userfunc_is_valid_name(name: *const u8, len: c_int) -> bool {
    if name.is_null() || len < 0 {
        return false;
    }

    let slice = unsafe { std::slice::from_raw_parts(name, len as usize) };
    is_valid_func_name(slice)
}

// =============================================================================
// Function Call State
// =============================================================================

/// State during a function call.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FuncCallState {
    /// Current line number in function
    pub current_lnum: i64,
    /// First line of function body
    pub first_lnum: i64,
    /// Last line of function body
    pub last_lnum: i64,
    /// Line number when function was called
    pub breakpoint: i64,
    /// Call depth (for recursion limit)
    pub depth: i32,
    /// Function flags
    pub flags: u32,
}

impl FuncCallState {
    /// Create new call state.
    pub const fn new(first_lnum: i64, last_lnum: i64, depth: i32) -> Self {
        Self {
            current_lnum: first_lnum,
            first_lnum,
            last_lnum,
            breakpoint: 0,
            depth,
            flags: 0,
        }
    }
}

// =============================================================================
// Return Value State
// =============================================================================

/// Return state after function execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ReturnState {
    /// Normal return
    Normal = 0,
    /// Return with value
    Return = 1,
    /// Break from loop
    Break = 2,
    /// Continue in loop
    Continue = 3,
    /// Error occurred
    Error = 4,
    /// Exception thrown
    Exception = 5,
}

impl ReturnState {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Normal,
            1 => Self::Return,
            2 => Self::Break,
            3 => Self::Continue,
            4 => Self::Error,
            _ => Self::Exception,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a normal/successful return.
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Normal | Self::Return)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_def_flags() {
        let flags = FuncDefFlags {
            is_closure: true,
            is_dict: true,
            ..Default::default()
        };
        let bits = flags.to_bits();
        let restored = FuncDefFlags::from_bits(bits);
        assert!(restored.is_closure);
        assert!(restored.is_dict);
        assert!(!restored.is_lambda);
    }

    #[test]
    fn test_is_valid_func_name() {
        assert!(is_valid_func_name(b"MyFunc"));
        assert!(is_valid_func_name(b"_private"));
        assert!(is_valid_func_name(b"s:Local"));
        assert!(is_valid_func_name(b"Func123"));

        assert!(!is_valid_func_name(b""));
        assert!(!is_valid_func_name(b"myFunc")); // lowercase start
        assert!(!is_valid_func_name(b"123Func")); // digit start
        assert!(!is_valid_func_name(b"s:")); // empty after prefix
    }

    #[test]
    fn test_return_state() {
        assert!(ReturnState::Normal.is_ok());
        assert!(ReturnState::Return.is_ok());
        assert!(!ReturnState::Error.is_ok());
        assert!(!ReturnState::Exception.is_ok());
    }
}
