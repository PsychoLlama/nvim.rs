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
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

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

// =============================================================================
// Name Validation
// =============================================================================

/// Check if byte is ASCII alphabetic (a-zA-Z)
const fn ascii_is_alpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// Check if byte is ASCII alphanumeric (a-zA-Z0-9)
const fn ascii_is_alnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// Check if byte ends an ex-command (NUL, '|', '"', '\n')
const fn ends_excmd(c: u8) -> bool {
    c == 0 || c == b'|' || c == b'"' || c == b'\n'
}

/// Check if byte is whitespace for Vim purposes (space or tab)
const fn ascii_is_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Validate a user command name.
///
/// Scans from the beginning of the name: it must start with an ASCII letter,
/// followed by zero or more ASCII alphanumeric characters. The character
/// immediately after the valid prefix must be a whitespace or an ex-command
/// terminator (NUL, '|', '"', '\n').
///
/// Returns the number of valid bytes in the name prefix, or -1 if the name
/// is invalid (doesn't start with alpha, or the character after the
/// alphanumeric prefix is neither whitespace nor an excmd terminator).
pub fn uc_validate_name(name: &[u8]) -> isize {
    if name.is_empty() || !ascii_is_alpha(name[0]) {
        // If the first character is not alpha, check if it's an excmd
        // terminator or whitespace — the C code checks `*name` after the
        // while loop, so an empty/non-alpha lead goes straight to the
        // terminator check.
        let first = if name.is_empty() { 0u8 } else { name[0] };
        if !ends_excmd(first) && !ascii_is_white(first) {
            return -1;
        }
        return 0;
    }

    let mut i = 0;
    // First character is alpha (checked above), advance past alnum
    while i < name.len() && ascii_is_alnum(name[i]) {
        i += 1;
    }

    // Check the character after the valid prefix
    let next = if i < name.len() { name[i] } else { 0u8 };
    if !ends_excmd(next) && !ascii_is_white(next) {
        return -1;
    }

    i as isize
}

/// FFI export: Validate a user command name.
///
/// Takes a NUL-terminated C string. Returns a pointer past the valid name
/// prefix, or NULL if the name is invalid. Matches the C `uc_validate_name`
/// signature exactly.
#[no_mangle]
pub unsafe extern "C" fn rs_uc_validate_name(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return std::ptr::null();
    }
    // Build a slice up to and including the NUL terminator so we can check
    // the character after the valid prefix.  We scan for the NUL.
    let mut len = 0usize;
    while unsafe { *name.add(len) } != 0 {
        len += 1;
    }
    // Include the NUL in the slice so ends_excmd(0) works at the boundary
    let slice = unsafe { std::slice::from_raw_parts(name.cast::<u8>(), len + 1) };
    let result = uc_validate_name(slice);
    if result < 0 {
        std::ptr::null()
    } else {
        unsafe { name.add(result as usize) }
    }
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

    // =========================================================================
    // uc_validate_name tests
    // =========================================================================

    #[test]
    fn test_uc_validate_name_simple() {
        // "Hello" followed by NUL
        assert_eq!(uc_validate_name(b"Hello\0"), 5);
    }

    #[test]
    fn test_uc_validate_name_with_trailing_space() {
        // "Cmd arg" — stops at space
        assert_eq!(uc_validate_name(b"Cmd arg"), 3);
    }

    #[test]
    fn test_uc_validate_name_with_trailing_tab() {
        assert_eq!(uc_validate_name(b"Cmd\targ"), 3);
    }

    #[test]
    fn test_uc_validate_name_with_bar() {
        assert_eq!(uc_validate_name(b"Cmd|other"), 3);
    }

    #[test]
    fn test_uc_validate_name_with_quote() {
        assert_eq!(uc_validate_name(b"Cmd\"comment"), 3);
    }

    #[test]
    fn test_uc_validate_name_with_newline() {
        assert_eq!(uc_validate_name(b"Cmd\nrest"), 3);
    }

    #[test]
    fn test_uc_validate_name_alphanumeric() {
        // Contains digits after alpha
        assert_eq!(uc_validate_name(b"Cmd123\0"), 6);
    }

    #[test]
    fn test_uc_validate_name_starts_with_digit() {
        // Doesn't start with alpha → checks first char, '1' is not excmd/white
        assert_eq!(uc_validate_name(b"1Cmd\0"), -1);
    }

    #[test]
    fn test_uc_validate_name_starts_with_special() {
        // '@' is not alpha, not excmd, not white
        assert_eq!(uc_validate_name(b"@Cmd\0"), -1);
    }

    #[test]
    fn test_uc_validate_name_invalid_after_alnum() {
        // "Cmd@" — '@' is not excmd/white
        assert_eq!(uc_validate_name(b"Cmd@rest"), -1);
    }

    #[test]
    fn test_uc_validate_name_empty() {
        // Empty input with just NUL
        assert_eq!(uc_validate_name(b"\0"), 0);
    }

    #[test]
    fn test_uc_validate_name_truly_empty() {
        // Zero-length slice — first char is 0 (implicit NUL), ends_excmd(0) is true
        assert_eq!(uc_validate_name(b""), 0);
    }

    #[test]
    fn test_uc_validate_name_just_bar() {
        // '|' is not alpha → check '|' which is ends_excmd → return 0
        assert_eq!(uc_validate_name(b"|\0"), 0);
    }

    #[test]
    fn test_uc_validate_name_ffi() {
        // Test via FFI wrapper
        let result = unsafe { rs_uc_validate_name(c"Hello".as_ptr()) };
        assert!(!result.is_null());
        // Should point 5 bytes past start
        let offset = unsafe { result.offset_from(c"Hello".as_ptr()) };
        assert_eq!(offset, 5);

        // Invalid name
        let result = unsafe { rs_uc_validate_name(c"1Bad".as_ptr()) };
        assert!(result.is_null());

        // NULL input
        let result = unsafe { rs_uc_validate_name(std::ptr::null()) };
        assert!(result.is_null());

        // Name with space after
        let s = c"Cmd rest";
        let result = unsafe { rs_uc_validate_name(s.as_ptr()) };
        assert!(!result.is_null());
        let offset = unsafe { result.offset_from(s.as_ptr()) };
        assert_eq!(offset, 3);
    }
}
