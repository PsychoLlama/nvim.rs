//! User command definition handling
//!
//! This module provides Rust implementations for user command definition,
//! including command flags, attributes, and storage.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

// =============================================================================
// UC_* Constants — from usercmd.h
// =============================================================================

/// -buffer: local to current buffer (UC_BUFFER in C)
pub const UC_BUFFER: c_int = 1;

// =============================================================================
// EX_* Flag Constants — from ex_cmds_defs.h
// =============================================================================

pub const EX_RANGE: u32 = 0x001;
pub const EX_BANG: u32 = 0x002;
pub const EX_EXTRA: u32 = 0x004;
pub const EX_XFILE: u32 = 0x008;
pub const EX_NOSPC: u32 = 0x010;
pub const EX_DFLALL: u32 = 0x020;
pub const EX_WHOLEFOLD: u32 = 0x040;
pub const EX_NEEDARG: u32 = 0x080;
pub const EX_TRLBAR: u32 = 0x100;
pub const EX_REGSTR: u32 = 0x200;
pub const EX_COUNT: u32 = 0x400;
pub const EX_NOTRLCOM: u32 = 0x800;
pub const EX_ZEROR: u32 = 0x1000;
pub const EX_CTRLV: u32 = 0x2000;
pub const EX_CMDARG: u32 = 0x4000;
pub const EX_BUFNAME: u32 = 0x8000;
pub const EX_BUFUNL: u32 = 0x1_0000;
pub const EX_ARGOPT: u32 = 0x2_0000;
pub const EX_SBOXOK: u32 = 0x4_0000;
pub const EX_CMDWIN: u32 = 0x8_0000;
pub const EX_MODIFY: u32 = 0x10_0000;
pub const EX_FLAGS: u32 = 0x20_0000;
pub const EX_LOCK_OK: u32 = 0x100_0000;
pub const EX_KEEPSCRIPT: u32 = 0x400_0000;
pub const EX_PREVIEW: u32 = 0x800_0000;

// Composite flags
pub const EX_FILES: u32 = EX_XFILE | EX_EXTRA;
pub const EX_FILE1: u32 = EX_FILES | EX_NOSPC;
pub const EX_WORD1: u32 = EX_EXTRA | EX_NOSPC;

// =============================================================================
// Command Flags
// =============================================================================

/// User command definition flags (internal Rust tracking)
pub const UC_BANG_FLAG: u32 = 0x0002;
pub const UC_RANGE_FLAG: u32 = 0x0004;
pub const UC_COUNT_FLAG: u32 = 0x0008;
pub const UC_REGISTER_FLAG: u32 = 0x0010;
pub const UC_NARGS_FLAG: u32 = 0x0020;
pub const UC_COMPLETE_FLAG: u32 = 0x0040;
pub const UC_FORCE_FLAG: u32 = 0x0080;
pub const UC_KEEPSCRIPT_FLAG: u32 = 0x0100;
pub const UC_BAR_FLAG: u32 = 0x0200;

/// User command flags wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UserCmdFlags {
    flags: u32,
}

impl UserCmdFlags {
    /// Create with no flags
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw value
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if buffer-local
    pub const fn is_buffer_local(self) -> bool {
        (self.flags & UC_BUFFER as u32) != 0
    }

    /// Check if allows bang (!)
    pub const fn allows_bang(self) -> bool {
        (self.flags & UC_BANG_FLAG) != 0
    }

    /// Check if allows range
    pub const fn allows_range(self) -> bool {
        (self.flags & UC_RANGE_FLAG) != 0
    }

    /// Check if allows count
    pub const fn allows_count(self) -> bool {
        (self.flags & UC_COUNT_FLAG) != 0
    }

    /// Check if allows register
    pub const fn allows_register(self) -> bool {
        (self.flags & UC_REGISTER_FLAG) != 0
    }

    /// Check if has nargs specified
    pub const fn has_nargs(self) -> bool {
        (self.flags & UC_NARGS_FLAG) != 0
    }

    /// Check if has complete specified
    pub const fn has_complete(self) -> bool {
        (self.flags & UC_COMPLETE_FLAG) != 0
    }

    /// Check if allows bar (|)
    pub const fn allows_bar(self) -> bool {
        (self.flags & UC_BAR_FLAG) != 0
    }

    /// Set buffer-local flag
    pub fn set_buffer_local(&mut self, value: bool) {
        if value {
            self.flags |= UC_BUFFER as u32;
        } else {
            self.flags &= !(UC_BUFFER as u32);
        }
    }

    /// Set bang flag
    pub fn set_bang(&mut self, value: bool) {
        if value {
            self.flags |= UC_BANG_FLAG;
        } else {
            self.flags &= !UC_BANG_FLAG;
        }
    }

    /// Set range flag
    pub fn set_range(&mut self, value: bool) {
        if value {
            self.flags |= UC_RANGE_FLAG;
        } else {
            self.flags &= !UC_RANGE_FLAG;
        }
    }
}

// =============================================================================
// Command Definition Flags
// =============================================================================

/// Flags for :command definition parsing
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CmdDefFlags {
    flags: u32,
}

pub const DEF_REPLACE: u32 = 0x01;
pub const DEF_BANG: u32 = 0x02;
pub const DEF_VERBOSE: u32 = 0x04;

impl CmdDefFlags {
    /// Create with no flags
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw value
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if replacing existing command
    pub const fn is_replacing(self) -> bool {
        (self.flags & DEF_REPLACE) != 0
    }

    /// Check if bang was used (force)
    pub const fn has_bang(self) -> bool {
        (self.flags & DEF_BANG) != 0
    }

    /// Check if verbose mode
    pub const fn is_verbose(self) -> bool {
        (self.flags & DEF_VERBOSE) != 0
    }
}

// =============================================================================
// User Command Definition
// =============================================================================

/// User command definition structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserCmdDef {
    /// Command flags
    pub flags: UserCmdFlags,
    /// Definition flags
    pub def_flags: CmdDefFlags,
    /// Number of arguments (encoded)
    pub nargs: c_int,
    /// Address type
    pub addr_type: c_int,
    /// Completion type
    pub complete: c_int,
    /// Default count
    pub def_count: c_int,
}

impl Default for UserCmdDef {
    fn default() -> Self {
        Self {
            flags: UserCmdFlags::none(),
            def_flags: CmdDefFlags::none(),
            nargs: 0,
            addr_type: 0,
            complete: -1,
            def_count: 0,
        }
    }
}

impl UserCmdDef {
    /// Create a new command definition
    pub const fn new() -> Self {
        Self {
            flags: UserCmdFlags { flags: 0 },
            def_flags: CmdDefFlags { flags: 0 },
            nargs: 0,
            addr_type: 0,
            complete: -1,
            def_count: 0,
        }
    }

    /// Check if definition is valid
    pub const fn is_valid(&self) -> bool {
        self.flags.flags != 0 || self.nargs != 0
    }

    /// Check if command is buffer-local
    pub const fn is_buffer_local(&self) -> bool {
        self.flags.is_buffer_local()
    }

    /// Check if command allows bang
    pub const fn allows_bang(&self) -> bool {
        self.flags.allows_bang()
    }

    /// Check if command allows range
    pub const fn allows_range(&self) -> bool {
        self.flags.allows_range()
    }

    /// Check if command has completion
    pub const fn has_complete(&self) -> bool {
        self.complete >= 0
    }
}

// =============================================================================
// Nargs Encoding
// =============================================================================

/// Number of arguments encoding
pub const NARGS_ZERO: c_int = 0;
pub const NARGS_ONE: c_int = 1;
pub const NARGS_ANY: c_int = -1;
pub const NARGS_OPTIONAL: c_int = -2;
pub const NARGS_ONE_OR_MORE: c_int = -3;

/// Parse nargs string to encoded value
pub fn parse_nargs(s: &str) -> Option<c_int> {
    match s {
        "0" => Some(NARGS_ZERO),
        "1" => Some(NARGS_ONE),
        "*" => Some(NARGS_ANY),
        "?" => Some(NARGS_OPTIONAL),
        "+" => Some(NARGS_ONE_OR_MORE),
        _ => None,
    }
}

/// Get nargs description
pub const fn nargs_description(nargs: c_int) -> &'static str {
    match nargs {
        NARGS_ZERO => "0",
        NARGS_ONE => "1",
        NARGS_ANY => "*",
        NARGS_OPTIONAL => "?",
        NARGS_ONE_OR_MORE => "+",
        _ => "?",
    }
}

/// Check if nargs requires at least one argument
pub const fn nargs_requires_arg(nargs: c_int) -> bool {
    nargs == NARGS_ONE || nargs == NARGS_ONE_OR_MORE
}

/// Check if nargs allows arguments
pub const fn nargs_allows_args(nargs: c_int) -> bool {
    nargs != NARGS_ZERO
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if flags is buffer local
#[no_mangle]
pub extern "C" fn rs_usercmd_flags_is_buffer_local(flags: u32) -> c_int {
    c_int::from(UserCmdFlags::from_raw(flags).is_buffer_local())
}

/// FFI export: Check if flags allows bang
#[no_mangle]
pub extern "C" fn rs_usercmd_flags_allows_bang(flags: u32) -> c_int {
    c_int::from(UserCmdFlags::from_raw(flags).allows_bang())
}

/// FFI export: Check if flags allows range
#[no_mangle]
pub extern "C" fn rs_usercmd_flags_allows_range(flags: u32) -> c_int {
    c_int::from(UserCmdFlags::from_raw(flags).allows_range())
}

/// FFI export: Check if nargs requires argument
#[no_mangle]
pub extern "C" fn rs_usercmd_nargs_requires_arg(nargs: c_int) -> c_int {
    c_int::from(nargs_requires_arg(nargs))
}

/// FFI export: Check if nargs allows arguments
#[no_mangle]
pub extern "C" fn rs_usercmd_nargs_allows_args(nargs: c_int) -> c_int {
    c_int::from(nargs_allows_args(nargs))
}

/// FFI export: Create default definition
#[no_mangle]
pub extern "C" fn rs_usercmd_def_new() -> UserCmdDef {
    UserCmdDef::new()
}

/// FFI export: Check if definition is valid
#[no_mangle]
pub extern "C" fn rs_usercmd_def_is_valid(def: *const UserCmdDef) -> c_int {
    if def.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*def).is_valid() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ex_flag_values() {
        // Verify EX_* flags match ex_cmds_defs.h
        assert_eq!(EX_RANGE, 0x001);
        assert_eq!(EX_BANG, 0x002);
        assert_eq!(EX_EXTRA, 0x004);
        assert_eq!(EX_XFILE, 0x008);
        assert_eq!(EX_NOSPC, 0x010);
        assert_eq!(EX_DFLALL, 0x020);
        assert_eq!(EX_WHOLEFOLD, 0x040);
        assert_eq!(EX_NEEDARG, 0x080);
        assert_eq!(EX_TRLBAR, 0x100);
        assert_eq!(EX_REGSTR, 0x200);
        assert_eq!(EX_COUNT, 0x400);
        assert_eq!(EX_NOTRLCOM, 0x800);
        assert_eq!(EX_ZEROR, 0x1000);
        assert_eq!(EX_CTRLV, 0x2000);
        assert_eq!(EX_CMDARG, 0x4000);
        assert_eq!(EX_BUFNAME, 0x8000);
        assert_eq!(EX_BUFUNL, 0x1_0000);
        assert_eq!(EX_ARGOPT, 0x2_0000);
        assert_eq!(EX_SBOXOK, 0x4_0000);
        assert_eq!(EX_CMDWIN, 0x8_0000);
        assert_eq!(EX_MODIFY, 0x10_0000);
        assert_eq!(EX_FLAGS, 0x20_0000);
        assert_eq!(EX_LOCK_OK, 0x100_0000);
        assert_eq!(EX_KEEPSCRIPT, 0x400_0000);
        assert_eq!(EX_PREVIEW, 0x800_0000);
    }

    #[test]
    fn test_uc_buffer_value() {
        assert_eq!(UC_BUFFER, 1);
    }

    #[test]
    fn test_user_cmd_flags() {
        let flags = UserCmdFlags::none();
        assert!(!flags.is_buffer_local());
        assert!(!flags.allows_bang());

        let flags = UserCmdFlags::from_raw(UC_BUFFER as u32 | UC_BANG_FLAG);
        assert!(flags.is_buffer_local());
        assert!(flags.allows_bang());
        assert!(!flags.allows_range());
    }

    #[test]
    fn test_user_cmd_flags_set() {
        let mut flags = UserCmdFlags::none();
        flags.set_buffer_local(true);
        assert!(flags.is_buffer_local());

        flags.set_bang(true);
        assert!(flags.allows_bang());

        flags.set_buffer_local(false);
        assert!(!flags.is_buffer_local());
    }

    #[test]
    fn test_cmd_def_flags() {
        let flags = CmdDefFlags::none();
        assert!(!flags.is_replacing());
        assert!(!flags.has_bang());

        let flags = CmdDefFlags::from_raw(DEF_REPLACE | DEF_BANG);
        assert!(flags.is_replacing());
        assert!(flags.has_bang());
    }

    #[test]
    fn test_user_cmd_def() {
        let def = UserCmdDef::new();
        assert!(!def.is_valid());
        assert!(!def.is_buffer_local());
        assert!(!def.has_complete());

        let mut def = UserCmdDef::new();
        def.flags = UserCmdFlags::from_raw(UC_BUFFER as u32 | UC_BANG_FLAG);
        assert!(def.is_valid());
        assert!(def.is_buffer_local());
        assert!(def.allows_bang());
    }

    #[test]
    fn test_parse_nargs() {
        assert_eq!(parse_nargs("0"), Some(NARGS_ZERO));
        assert_eq!(parse_nargs("1"), Some(NARGS_ONE));
        assert_eq!(parse_nargs("*"), Some(NARGS_ANY));
        assert_eq!(parse_nargs("?"), Some(NARGS_OPTIONAL));
        assert_eq!(parse_nargs("+"), Some(NARGS_ONE_OR_MORE));
        assert_eq!(parse_nargs("x"), None);
    }

    #[test]
    fn test_nargs_properties() {
        assert!(nargs_requires_arg(NARGS_ONE));
        assert!(nargs_requires_arg(NARGS_ONE_OR_MORE));
        assert!(!nargs_requires_arg(NARGS_ZERO));
        assert!(!nargs_requires_arg(NARGS_OPTIONAL));

        assert!(!nargs_allows_args(NARGS_ZERO));
        assert!(nargs_allows_args(NARGS_ONE));
        assert!(nargs_allows_args(NARGS_ANY));
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_flags_is_buffer_local(UC_BUFFER as u32), 1);
        assert_eq!(rs_usercmd_flags_is_buffer_local(0), 0);

        assert_eq!(rs_usercmd_nargs_requires_arg(NARGS_ONE), 1);
        assert_eq!(rs_usercmd_nargs_requires_arg(NARGS_ZERO), 0);
    }
}
