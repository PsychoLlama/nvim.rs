//! User command infrastructure
//!
//! This crate provides Rust implementations for user-defined command handling,
//! including command definition, completion, execution, and argument parsing.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

pub mod complete;
pub mod define;
pub mod execute;
pub mod parse;

use std::ffi::c_int;

// Re-export key types
pub use complete::{CompleteContext, CompleteMatch, CompleteState, CompleteType};
pub use define::{CmdDefFlags, UserCmdDef, UserCmdFlags};
pub use execute::{CmdModifiers, ExecContext, ExecResult, ExecState, SpecialArg};
pub use parse::{ParseResult, ParseState, Token, TokenType};

// =============================================================================
// Constants
// =============================================================================

/// Maximum user command name length
pub const USERCMD_NAME_MAX: usize = 200;

/// Maximum number of arguments
pub const USERCMD_MAX_ARGS: c_int = 100;

// =============================================================================
// User Command Address Types
// =============================================================================

/// Address type for command
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddrType {
    /// No address
    #[default]
    None = 0,
    /// Line address
    Lines = 1,
    /// Argument address
    Arguments = 2,
    /// Buffer address
    Buffers = 3,
    /// Loaded buffers
    LoadedBuffers = 4,
    /// Windows
    Windows = 5,
    /// Tabs
    Tabs = 6,
    /// Quickfix entries
    Quickfix = 7,
    /// Other (generic)
    Other = 8,
}

impl AddrType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Lines),
            2 => Some(Self::Arguments),
            3 => Some(Self::Buffers),
            4 => Some(Self::LoadedBuffers),
            5 => Some(Self::Windows),
            6 => Some(Self::Tabs),
            7 => Some(Self::Quickfix),
            8 => Some(Self::Other),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this allows range
    pub const fn allows_range(self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// User Command Error Types
// =============================================================================

/// Error types for user command operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCmdError {
    /// No error
    None = 0,
    /// Command not found
    NotFound = 1,
    /// Invalid name
    InvalidName = 2,
    /// Invalid arguments
    InvalidArgs = 3,
    /// Command already exists
    AlreadyExists = 4,
    /// Cannot delete (built-in)
    CannotDelete = 5,
    /// Missing argument
    MissingArg = 6,
    /// Too many arguments
    TooManyArgs = 7,
    /// Invalid range
    InvalidRange = 8,
    /// Execution failed
    ExecFailed = 9,
}

impl UserCmdError {
    /// Check if this is success
    pub const fn is_success(self) -> bool {
        matches!(self, Self::None)
    }

    /// Check if this is an error
    pub const fn is_error(self) -> bool {
        !self.is_success()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if address type is valid
#[no_mangle]
pub extern "C" fn rs_usercmd_addr_type_valid(addr_type: c_int) -> c_int {
    c_int::from(AddrType::from_raw(addr_type).is_some())
}

/// FFI export: Check if address type allows range
#[no_mangle]
pub extern "C" fn rs_usercmd_addr_allows_range(addr_type: c_int) -> c_int {
    AddrType::from_raw(addr_type).map_or(0, |a| c_int::from(a.allows_range()))
}

/// FFI export: Check if error is success
#[no_mangle]
pub extern "C" fn rs_usercmd_error_is_success(error: UserCmdError) -> c_int {
    c_int::from(error.is_success())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addr_type() {
        assert_eq!(AddrType::from_raw(0), Some(AddrType::None));
        assert_eq!(AddrType::from_raw(1), Some(AddrType::Lines));
        assert_eq!(AddrType::from_raw(100), None);

        assert!(!AddrType::None.allows_range());
        assert!(AddrType::Lines.allows_range());
        assert!(AddrType::Buffers.allows_range());
    }

    #[test]
    fn test_error_type() {
        assert!(UserCmdError::None.is_success());
        assert!(!UserCmdError::NotFound.is_success());
        assert!(UserCmdError::InvalidName.is_error());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_addr_type_valid(0), 1);
        assert_eq!(rs_usercmd_addr_type_valid(100), 0);

        assert_eq!(rs_usercmd_addr_allows_range(0), 0);
        assert_eq!(rs_usercmd_addr_allows_range(1), 1);
    }
}
