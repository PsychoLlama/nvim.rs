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

use std::ffi::{c_char, c_int, c_void};

// Re-export key types
pub use complete::CompleteType;
pub use define::UserCmdFlags;
pub use execute::SpecialArg;
pub use parse::{ParseResult, Token, TokenType};

// =============================================================================
// Opaque Handle Types (pointers to C structs)
// =============================================================================

/// Opaque handle to exarg_T
pub type ExargHandle = *mut c_void;
/// Opaque handle to expand_T
pub type ExpandHandle = *mut c_void;
/// Opaque handle to garray_T (opaque, for passing to C)
pub type GarrayHandle = *mut c_void;
/// Opaque handle to ucmd_T (opaque, for passing to C)
pub type UcmdHandle = *mut c_void;
/// Opaque handle to buf_T
pub type BufHandle = *mut c_void;
/// Opaque handle to cmdmod_T
pub type CmdmodHandle = *mut c_void;

// =============================================================================
// repr(C) Struct Definitions
// =============================================================================

/// Script context — matches `sctx_T` in C (eval/typval_defs.h).
/// Layout: sc_sid:i32@0, sc_seq:i32@4, sc_lnum:i32@8, _pad:i32@12, sc_chan:u64@16 = 24 bytes.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SctxT {
    pub sc_sid: c_int,
    pub sc_seq: c_int,
    pub sc_lnum: c_int,
    _pad: c_int,
    pub sc_chan: u64,
}

/// Growing array — matches `garray_T` in C (garray_defs.h).
/// Layout: ga_len:i32@0, ga_maxlen:i32@4, ga_itemsize:i32@8, ga_growsize:i32@12, ga_data:ptr@16 = 24 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

/// User command entry — matches `ucmd_T` in C (usercmd.h).
/// Layout verified against C struct (see size assertion in tests).
#[repr(C)]
pub struct UcmdT {
    pub uc_name: *mut c_char,
    pub uc_argt: u32,
    _pad_argt: u32,
    pub uc_rep: *mut c_char,
    pub uc_def: i64,
    pub uc_compl: c_int,
    pub uc_addr_type: c_int,
    pub uc_script_ctx: SctxT,
    pub uc_compl_arg: *mut c_char,
    pub uc_compl_luaref: c_int,
    pub uc_preview_luaref: c_int,
    pub uc_luaref: c_int,
    _pad_end: c_int,
}

// =============================================================================
// Global State Access
// =============================================================================

extern "C" {
    /// Global user commands array (ucmds in C)
    pub static mut ucmds: GarrayT;
    /// Current script context (current_sctx in C)
    pub static mut current_sctx: SctxT;
    /// api_free_luaref — free a Lua reference (used in NLUA_CLEAR_REF logic)
    #[link_name = "api_free_luaref"]
    pub fn api_free_luaref(luaref: c_int);
    /// ga_init — initialize a garray_T
    #[link_name = "ga_init"]
    pub fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    /// nlua_set_sctx — set nlua script context
    #[link_name = "nlua_set_sctx"]
    pub fn nlua_set_sctx(sctx: *mut SctxT);
    /// last_set_msg — display "Last set from" message
    #[link_name = "last_set_msg"]
    pub fn last_set_msg(sctx: SctxT);
}

/// LUA_NOREF constant (matches C LUA_NOREF = -2)
pub const LUA_NOREF: c_int = -2;

// =============================================================================
// Constants
// =============================================================================

/// Maximum user command name length
pub const USERCMD_NAME_MAX: usize = 200;

/// Maximum number of arguments
pub const USERCMD_MAX_ARGS: c_int = 100;

// =============================================================================
// User Command Address Types (cmd_addr_T in C)
// =============================================================================

/// Address type for command — matches cmd_addr_T enum in ex_cmds_defs.h
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddrType {
    /// Line address (ADDR_LINES = 0)
    Lines = 0,
    /// Window number (ADDR_WINDOWS = 1)
    Windows = 1,
    /// Argument number (ADDR_ARGUMENTS = 2)
    Arguments = 2,
    /// Loaded buffer number (ADDR_LOADED_BUFFERS = 3)
    LoadedBuffers = 3,
    /// Buffer number (ADDR_BUFFERS = 4)
    Buffers = 4,
    /// Tab page number (ADDR_TABS = 5)
    Tabs = 5,
    /// Tab page relative (ADDR_TABS_RELATIVE = 6)
    TabsRelative = 6,
    /// Quickfix valid entry number (ADDR_QUICKFIX_VALID = 7)
    QuickfixValid = 7,
    /// Quickfix entry number (ADDR_QUICKFIX = 8)
    Quickfix = 8,
    /// Positive count or zero (ADDR_UNSIGNED = 9)
    Unsigned = 9,
    /// Other / generic (ADDR_OTHER = 10)
    Other = 10,
    /// No range used (ADDR_NONE = 11)
    #[default]
    None = 11,
}

impl AddrType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Lines),
            1 => Some(Self::Windows),
            2 => Some(Self::Arguments),
            3 => Some(Self::LoadedBuffers),
            4 => Some(Self::Buffers),
            5 => Some(Self::Tabs),
            6 => Some(Self::TabsRelative),
            7 => Some(Self::QuickfixValid),
            8 => Some(Self::Quickfix),
            9 => Some(Self::Unsigned),
            10 => Some(Self::Other),
            11 => Some(Self::None),
            _ => Option::None,
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
    Ok = 0,
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
        matches!(self, Self::Ok)
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
    fn test_addr_type_values() {
        // Verify values match C enum cmd_addr_T
        assert_eq!(AddrType::Lines as c_int, 0);
        assert_eq!(AddrType::Windows as c_int, 1);
        assert_eq!(AddrType::Arguments as c_int, 2);
        assert_eq!(AddrType::LoadedBuffers as c_int, 3);
        assert_eq!(AddrType::Buffers as c_int, 4);
        assert_eq!(AddrType::Tabs as c_int, 5);
        assert_eq!(AddrType::TabsRelative as c_int, 6);
        assert_eq!(AddrType::QuickfixValid as c_int, 7);
        assert_eq!(AddrType::Quickfix as c_int, 8);
        assert_eq!(AddrType::Unsigned as c_int, 9);
        assert_eq!(AddrType::Other as c_int, 10);
        assert_eq!(AddrType::None as c_int, 11);
    }

    #[test]
    fn test_addr_type_from_raw() {
        assert_eq!(AddrType::from_raw(0), Some(AddrType::Lines));
        assert_eq!(AddrType::from_raw(11), Some(AddrType::None));
        assert_eq!(AddrType::from_raw(12), Option::None);
        assert_eq!(AddrType::from_raw(-1), Option::None);
    }

    #[test]
    fn test_addr_type_allows_range() {
        assert!(!AddrType::None.allows_range());
        assert!(AddrType::Lines.allows_range());
        assert!(AddrType::Buffers.allows_range());
        assert!(AddrType::Windows.allows_range());
        assert!(AddrType::TabsRelative.allows_range());
        assert!(AddrType::QuickfixValid.allows_range());
        assert!(AddrType::Unsigned.allows_range());
    }

    #[test]
    fn test_error_type() {
        assert!(UserCmdError::Ok.is_success());
        assert!(!UserCmdError::NotFound.is_success());
        assert!(UserCmdError::InvalidName.is_error());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_addr_type_valid(0), 1);
        assert_eq!(rs_usercmd_addr_type_valid(11), 1);
        assert_eq!(rs_usercmd_addr_type_valid(12), 0);

        assert_eq!(rs_usercmd_addr_allows_range(11), 0); // None
        assert_eq!(rs_usercmd_addr_allows_range(0), 1); // Lines
    }
}
