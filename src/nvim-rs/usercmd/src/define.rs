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
#![allow(clippy::too_many_arguments)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::explicit_iter_loop)]

use std::ffi::{c_char, c_int, c_void, CStr};

use crate::complete::ADDR_TYPE_COMPLETE;
use crate::{AddrType, ExargHandle, ExpandHandle, GarrayHandle};

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
#[export_name = "uc_validate_name"]
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
// C Return Value Constants
// =============================================================================

/// OK return value (matches C OK = 1)
const C_OK: c_int = 1;
/// FAIL return value (matches C FAIL = 0)
const C_FAIL: c_int = 0;
/// LUA_NOREF value (matches C LUA_NOREF = -2)
const LUA_NOREF: c_int = -2;

// =============================================================================
// C Accessor Functions (Phase 5 — command definition management)
// =============================================================================

extern "C" {
    // garray operations
    /// Returns &curbuf->b_ucmds (GarrayHandle)
    fn nvim_uc_get_curbuf_ucmds() -> GarrayHandle;
    /// Returns &ucmds (GarrayHandle)
    fn nvim_uc_get_ucmds() -> GarrayHandle;
    /// Returns gap->ga_len
    fn nvim_uc_ga_get_len(gap: GarrayHandle) -> c_int;
    /// Returns gap->ga_itemsize
    fn nvim_uc_ga_get_itemsize(gap: GarrayHandle) -> c_int;
    /// Sets gap->ga_len = len
    fn nvim_uc_ga_set_len(gap: GarrayHandle, len: c_int);
    /// ga_init(gap, sizeof(ucmd_T), 4)
    fn nvim_uc_ga_init_ucmd(gap: GarrayHandle);
    /// ga_grow(gap, n)
    fn nvim_uc_ga_grow(gap: GarrayHandle, n: c_int);
    /// ga_clear(gap)
    fn nvim_uc_ga_clear(gap: GarrayHandle);

    // ucmd_T element access
    /// Returns USER_CMD_GA(gap, i) as opaque pointer
    fn nvim_uc_ga_get_cmd(gap: GarrayHandle, i: c_int) -> *mut c_void;
    /// memmove(cmd+1, cmd, (gap->ga_len - i) * sizeof(ucmd_T))
    fn nvim_uc_cmd_memmove_down(gap: GarrayHandle, i: c_int);

    // ucmd_T field getters
    /// Returns cmd->uc_name
    fn nvim_uc_cmd_get_name(cmd: *const c_void) -> *const c_char;
    /// Returns cmd->uc_script_ctx.sc_sid
    fn nvim_uc_cmd_get_sc_sid(cmd: *const c_void) -> c_int;
    /// Returns cmd->uc_script_ctx.sc_seq
    fn nvim_uc_cmd_get_sc_seq(cmd: *const c_void) -> c_int;

    // ucmd_T field setters
    /// Sets cmd->uc_name = name
    fn nvim_uc_cmd_set_name(cmd: *mut c_void, name: *mut c_char);
    /// Sets cmd->uc_rep = rep
    fn nvim_uc_cmd_set_rep(cmd: *mut c_void, rep: *mut c_char);
    /// Sets cmd->uc_argt = argt
    fn nvim_uc_cmd_set_argt(cmd: *mut c_void, argt: u32);
    /// Sets cmd->uc_def = def
    fn nvim_uc_cmd_set_def(cmd: *mut c_void, def: i64);
    /// Sets cmd->uc_compl = compl
    fn nvim_uc_cmd_set_compl(cmd: *mut c_void, compl_val: c_int);
    /// Sets cmd->uc_compl_arg = arg
    fn nvim_uc_cmd_set_compl_arg(cmd: *mut c_void, arg: *mut c_char);
    /// Sets cmd->uc_addr_type = addr_type
    fn nvim_uc_cmd_set_addr_type(cmd: *mut c_void, addr_type: c_int);
    /// Sets cmd->uc_luaref = luaref
    fn nvim_uc_cmd_set_luaref(cmd: *mut c_void, luaref: c_int);
    /// Sets cmd->uc_compl_luaref = luaref
    fn nvim_uc_cmd_set_compl_luaref(cmd: *mut c_void, luaref: c_int);
    /// Sets cmd->uc_preview_luaref = luaref
    fn nvim_uc_cmd_set_preview_luaref(cmd: *mut c_void, luaref: c_int);
    /// Sets cmd->uc_script_ctx = current_sctx; sc_lnum += SOURCING_LNUM;
    /// nlua_set_sctx(&cmd->uc_script_ctx)
    fn nvim_uc_cmd_set_script_ctx(cmd: *mut c_void);

    // ucmd_T field cleanup (XFREE_CLEAR / NLUA_CLEAR_REF on individual fields)
    /// XFREE_CLEAR(cmd->uc_rep)
    fn nvim_uc_cmd_free_rep(cmd: *mut c_void);
    /// XFREE_CLEAR(cmd->uc_compl_arg)
    fn nvim_uc_cmd_free_compl_arg(cmd: *mut c_void);
    /// NLUA_CLEAR_REF(cmd->uc_luaref)
    fn nvim_uc_cmd_clear_luaref(cmd: *mut c_void);
    /// NLUA_CLEAR_REF(cmd->uc_compl_luaref)
    fn nvim_uc_cmd_clear_compl_luaref(cmd: *mut c_void);
    /// NLUA_CLEAR_REF(cmd->uc_preview_luaref)
    fn nvim_uc_cmd_clear_preview_luaref(cmd: *mut c_void);

    // Whole-struct cleanup
    /// Calls free_ucmd(cmd) — frees all fields of a ucmd_T
    fn nvim_uc_free_ucmd(cmd: *mut c_void);

    // Memory operations
    /// xfree(ptr)
    fn nvim_uc_xfree(ptr: *mut c_void);
    /// NLUA_CLEAR_REF(ref) — for standalone LuaRef values (not in a struct)
    fn nvim_uc_nlua_clear_ref(luaref: c_int);
    /// replace_termcodes(rep, replen, &buf, 0, 0, NULL, p_cpo) then
    /// returns buf (or xstrdup(rep) if buf is NULL). Caller owns result.
    fn nvim_uc_replace_termcodes(rep: *const c_char, replen: usize) -> *mut c_char;
    /// xstrnsave(s, len) — already exists from Phase 2
    fn nvim_uc_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // Global state
    /// Returns current_sctx.sc_sid
    fn nvim_uc_get_current_sctx_sid() -> c_int;
    /// Returns current_sctx.sc_seq
    fn nvim_uc_get_current_sctx_seq() -> c_int;

    // Error reporting — already exists from Phase 2
    /// semsg(_(fmt), arg)
    fn nvim_uc_semsg_1(fmt: *const c_char, arg: *const c_char);
}

// =============================================================================
// Phase 5: Command Definition Management
// =============================================================================

/// Find the insertion point for a command name in a sorted garray.
///
/// Returns (index, cmp) where:
/// - If cmp == 0: exact match found at index
/// - If cmp < 0: name sorts before index
/// - If cmp > 0: name sorts after all entries (index == len)
unsafe fn find_cmd_index(
    gap: GarrayHandle,
    name: *const c_char,
    name_len: usize,
) -> (c_int, c_int) {
    let ga_len = nvim_uc_ga_get_len(gap);
    let mut cmp: c_int = 1;

    for i in 0..ga_len {
        let cmd = nvim_uc_ga_get_cmd(gap, i);
        let cmd_name = nvim_uc_cmd_get_name(cmd);
        let cmd_name_cstr = CStr::from_ptr(cmd_name);
        let cmd_name_bytes = cmd_name_cstr.to_bytes();
        let cmd_name_len = cmd_name_bytes.len();

        // Compare using the input name length for strncmp equivalent
        let name_slice = std::slice::from_raw_parts(name.cast::<u8>(), name_len);
        let compare_len = name_len.min(cmd_name_len);
        let left = &name_slice[..compare_len];
        let right = &cmd_name_bytes[..compare_len];

        cmp = match left.cmp(right) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
        };

        // If prefix matches, compare by length (like strncmp + length check)
        if cmp == 0 {
            if name_len < cmd_name_len {
                cmp = -1;
            } else if name_len > cmd_name_len {
                cmp = 1;
            }
        }

        if cmp <= 0 {
            return (i, cmp);
        }
    }

    (ga_len, cmp)
}

/// Create or replace a user command in a sorted garray.
///
/// This is the Rust implementation of `uc_add_command` from usercmd.c.
///
/// # Safety
///
/// All pointer arguments must be valid. `name` and `rep` must be non-null
/// NUL-terminated C strings. The caller is responsible for memory ownership
/// as described in the C API contract.
unsafe fn uc_add_command_impl(
    name: *mut c_char,
    name_len: usize,
    rep: *const c_char,
    argt: u32,
    def: i64,
    flags: c_int,
    context: c_int,
    compl_arg: *mut c_char,
    compl_luaref: c_int,
    preview_luaref: c_int,
    addr_type: c_int,
    luaref: c_int,
    force: c_int,
) -> c_int {
    // replace_termcodes on the replacement string
    let rep_cstr = CStr::from_ptr(rep);
    let rep_len = rep_cstr.to_bytes().len();
    let rep_buf = nvim_uc_replace_termcodes(rep, rep_len);
    // nvim_uc_replace_termcodes handles the NULL case internally
    // (returns xstrdup(rep) if replace_termcodes returns NULL)

    // Get the appropriate garray (buffer-local or global)
    let gap = if (flags & UC_BUFFER) != 0 {
        let gap = nvim_uc_get_curbuf_ucmds();
        if nvim_uc_ga_get_itemsize(gap) == 0 {
            nvim_uc_ga_init_ucmd(gap);
        }
        gap
    } else {
        nvim_uc_get_ucmds()
    };

    // Search for the command in the sorted array
    let (i, cmp) = find_cmd_index(gap, name, name_len);

    if cmp == 0 {
        // Exact match found — check if we can replace
        let cmd = nvim_uc_ga_get_cmd(gap, i);

        if force == 0
            && (nvim_uc_cmd_get_sc_sid(cmd) != nvim_uc_get_current_sctx_sid()
                || nvim_uc_cmd_get_sc_seq(cmd) == nvim_uc_get_current_sctx_seq())
        {
            // Cannot replace: emit error and clean up
            nvim_uc_semsg_1(
                c"E174: Command already exists: add ! to replace it: %s".as_ptr(),
                name,
            );
            // Cleanup on failure
            nvim_uc_xfree(rep_buf.cast::<c_void>());
            nvim_uc_xfree(compl_arg.cast::<c_void>());
            if luaref != LUA_NOREF {
                nvim_uc_nlua_clear_ref(luaref);
            }
            if compl_luaref != LUA_NOREF {
                nvim_uc_nlua_clear_ref(compl_luaref);
            }
            if preview_luaref != LUA_NOREF {
                nvim_uc_nlua_clear_ref(preview_luaref);
            }
            return C_FAIL;
        }

        // Replace existing: free old fields
        nvim_uc_cmd_free_rep(cmd);
        nvim_uc_cmd_free_compl_arg(cmd);
        nvim_uc_cmd_clear_luaref(cmd);
        nvim_uc_cmd_clear_compl_luaref(cmd);
        nvim_uc_cmd_clear_preview_luaref(cmd);
    }

    let cmd = if cmp != 0 {
        // Insert new command at position i
        nvim_uc_ga_grow(gap, 1);
        let p = nvim_uc_xstrnsave(name, name_len);
        // memmove existing entries down to make room
        nvim_uc_cmd_memmove_down(gap, i);
        let ga_len = nvim_uc_ga_get_len(gap);
        nvim_uc_ga_set_len(gap, ga_len + 1);
        let cmd = nvim_uc_ga_get_cmd(gap, i);
        nvim_uc_cmd_set_name(cmd, p);
        cmd
    } else {
        nvim_uc_ga_get_cmd(gap, i)
    };

    // Set all fields
    nvim_uc_cmd_set_rep(cmd, rep_buf);
    nvim_uc_cmd_set_argt(cmd, argt);
    nvim_uc_cmd_set_def(cmd, def);
    nvim_uc_cmd_set_compl(cmd, context);
    nvim_uc_cmd_set_script_ctx(cmd);
    nvim_uc_cmd_set_compl_arg(cmd, compl_arg);
    nvim_uc_cmd_set_compl_luaref(cmd, compl_luaref);
    nvim_uc_cmd_set_preview_luaref(cmd, preview_luaref);
    nvim_uc_cmd_set_addr_type(cmd, addr_type);
    nvim_uc_cmd_set_luaref(cmd, luaref);

    C_OK
}

/// Free all fields of a single ucmd_T.
///
/// # Safety
///
/// `cmd` must point to a valid ucmd_T.
unsafe fn free_ucmd_impl(cmd: *mut c_void) {
    nvim_uc_free_ucmd(cmd);
}

/// Clear all user commands in a garray.
///
/// Iterates over each element, calls free_ucmd on it, then calls ga_clear.
///
/// # Safety
///
/// `gap` must point to a valid garray_T containing ucmd_T elements.
unsafe fn uc_clear_impl(gap: GarrayHandle) {
    let ga_len = nvim_uc_ga_get_len(gap);
    for i in 0..ga_len {
        let cmd = nvim_uc_ga_get_cmd(gap, i);
        nvim_uc_free_ucmd(cmd);
    }
    nvim_uc_ga_clear(gap);
}

// =============================================================================
// Phase 5: FFI Exports
// =============================================================================

/// FFI export: Create or replace a user command.
///
/// Direct replacement for C `uc_add_command`.
/// C signature has `bool force` (1 byte), so we accept `bool` here.
#[export_name = "uc_add_command"]
pub unsafe extern "C" fn rs_uc_add_command(
    name: *mut c_char,
    name_len: usize,
    rep: *const c_char,
    argt: u32,
    def: i64,
    flags: c_int,
    context: c_int,
    compl_arg: *mut c_char,
    compl_luaref: c_int,
    preview_luaref: c_int,
    addr_type: c_int,
    luaref: c_int,
    force: bool,
) -> c_int {
    uc_add_command_impl(
        name,
        name_len,
        rep,
        argt,
        def,
        flags,
        context,
        compl_arg,
        compl_luaref,
        preview_luaref,
        addr_type,
        luaref,
        c_int::from(force),
    )
}

/// FFI export: Free all fields of a single ucmd_T.
///
/// Direct replacement for C `free_ucmd`.
#[export_name = "free_ucmd"]
pub unsafe extern "C" fn rs_free_ucmd(cmd: *mut c_void) {
    free_ucmd_impl(cmd);
}

/// FFI export: Clear all user commands in a garray.
///
/// Direct replacement for C `uc_clear`.
#[export_name = "uc_clear"]
pub unsafe extern "C" fn rs_uc_clear(gap: GarrayHandle) {
    uc_clear_impl(gap);
}

// =============================================================================
// C Accessor Functions (Phase 6 — ex command handlers)
// =============================================================================

extern "C" {
    // eap accessors (from Phase 4)
    /// Returns eap->arg
    fn nvim_uc_eap_get_arg(eap: ExargHandle) -> *const c_char;
    /// Returns eap->forceit (as int: 0 or 1)
    fn nvim_uc_eap_get_forceit(eap: ExargHandle) -> c_int;

    // String navigation helpers
    /// Returns skiptowhite(p) — pointer to first whitespace
    fn nvim_uc_skiptowhite(p: *const c_char) -> *mut c_char;
    /// Returns skipwhite(p) — pointer past whitespace
    fn nvim_uc_skipwhite(p: *const c_char) -> *mut c_char;
    /// Returns ends_excmd(c) — 1 if c ends an ex command, 0 otherwise
    fn nvim_uc_ends_excmd(c: c_int) -> c_int;

    // Deletion helper
    /// memmove(cmd, cmd+1, (ga_len - i) * sizeof(ucmd_T)) — shift entries up
    fn nvim_uc_cmd_memmove_up(gap: GarrayHandle, i: c_int);

    // curbuf null check
    /// Returns 1 if curbuf is NULL, 0 otherwise
    fn nvim_uc_curbuf_is_null() -> c_int;

    // Error reporting (already declared in Phase 2 extern block in parse.rs,
    // but we need it here too for emsg calls)
    /// Calls emsg(_(msg)) in C
    fn nvim_uc_emsg(msg: *const c_char);
}

// Already-migrated Rust functions called via extern "C" (same crate, different module)
extern "C" {
    /// uc_scan_attr — in parse.rs (exported as "uc_scan_attr")
    #[link_name = "uc_scan_attr"]
    fn rs_uc_scan_attr(
        attr: *mut c_char,
        len: usize,
        argt: *mut u32,
        def: *mut c_int,
        flags: *mut c_int,
        complp: *mut c_int,
        compl_arg: *mut *mut c_char,
        addr_type_arg: *mut c_int,
    ) -> c_int;
}

// =============================================================================
// Phase 6: Ex Command Handler Implementations
// =============================================================================

/// EXPAND_NOTHING constant (matches C EXPAND_NOTHING = 0)
const EXPAND_NOTHING: c_int = 0;

/// ADDR_NONE constant (matches C ADDR_NONE = 11)
const ADDR_NONE: c_int = 11;

/// Check if a byte is ASCII uppercase (A-Z)
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Implementation of `:command` ex handler.
///
/// Parses attributes, validates name, and either lists commands or adds a new one.
///
/// # Safety
///
/// `eap` must be a valid ExargHandle (pointer to exarg_T).
unsafe fn ex_command_impl(eap: ExargHandle) {
    let mut argt: u32 = 0;
    let mut def: c_int = -1;
    let mut flags: c_int = 0;
    let mut context: c_int = EXPAND_NOTHING;
    let mut compl_arg: *mut c_char = std::ptr::null_mut();
    let mut addr_type_arg: c_int = ADDR_NONE;

    let arg = nvim_uc_eap_get_arg(eap);
    let has_attr = unsafe { *arg.cast::<u8>() } == b'-';

    let mut p: *const c_char = arg;

    // Check for attributes
    while unsafe { *p.cast::<u8>() } == b'-' {
        p = unsafe { p.add(1) };
        let end = nvim_uc_skiptowhite(p);
        let attr_len = unsafe { end.offset_from(p) } as usize;
        // rs_uc_scan_attr needs *mut c_char because it temporarily NUL-terminates
        if rs_uc_scan_attr(
            p.cast_mut(),
            attr_len,
            &mut argt,
            &mut def,
            &mut flags,
            &mut context,
            &mut compl_arg,
            &mut addr_type_arg,
        ) == C_FAIL
        {
            // Cleanup on failure
            nvim_uc_xfree(compl_arg.cast::<c_void>());
            return;
        }
        p = nvim_uc_skipwhite(end);
    }

    // Get the name (if any) and skip to the following argument.
    let name = p;
    let end = rs_uc_validate_name(name);
    if end.is_null() {
        nvim_uc_emsg(c"E182: Invalid command name".as_ptr());
        nvim_uc_xfree(compl_arg.cast::<c_void>());
        return;
    }
    let name_len = unsafe { end.offset_from(name) } as usize;

    // If there is nothing after the name, and no attributes were specified,
    // we are listing commands
    p = nvim_uc_skipwhite(end);
    let p_byte = unsafe { *p.cast::<u8>() };

    if !has_attr && nvim_uc_ends_excmd(c_int::from(p_byte)) != 0 {
        uc_list_impl(name, name_len);
    } else if !ascii_isupper(unsafe { *name.cast::<u8>() }) {
        nvim_uc_emsg(c"E183: User defined commands must start with an uppercase letter".as_ptr());
    } else if name_len <= 4 {
        // Check for reserved name "Next"
        let next = b"Next";
        let name_slice = unsafe { std::slice::from_raw_parts(name.cast::<u8>(), name_len) };
        if name_slice == &next[..name_len] {
            nvim_uc_emsg(c"E841: Reserved name, cannot be used for user defined command".as_ptr());
        } else if context > 0 && (argt & EX_EXTRA) == 0 {
            nvim_uc_emsg(c"E1208: -complete used without allowing arguments".as_ptr());
        } else {
            uc_add_command_impl(
                name.cast_mut(),
                name_len,
                p,
                argt,
                i64::from(def),
                flags,
                context,
                compl_arg,
                LUA_NOREF,
                LUA_NOREF,
                addr_type_arg,
                LUA_NOREF,
                nvim_uc_eap_get_forceit(eap),
            );
            return; // success — ownership of compl_arg transferred
        }
    } else if context > 0 && (argt & EX_EXTRA) == 0 {
        nvim_uc_emsg(c"E1208: -complete used without allowing arguments".as_ptr());
    } else {
        uc_add_command_impl(
            name.cast_mut(),
            name_len,
            p,
            argt,
            i64::from(def),
            flags,
            context,
            compl_arg,
            LUA_NOREF,
            LUA_NOREF,
            addr_type_arg,
            LUA_NOREF,
            nvim_uc_eap_get_forceit(eap),
        );
        return; // success — ownership of compl_arg transferred
    }

    // Cleanup on non-success paths
    nvim_uc_xfree(compl_arg.cast::<c_void>());
}

/// Implementation of `:comclear` ex handler.
///
/// Clears all global and buffer-local user commands.
///
/// # Safety
///
/// `_eap` must be a valid ExargHandle (unused but required by signature).
unsafe fn ex_comclear_impl(_eap: ExargHandle) {
    let ucmds = nvim_uc_get_ucmds();
    uc_clear_impl(ucmds);
    if nvim_uc_curbuf_is_null() == 0 {
        let buf_ucmds = nvim_uc_get_curbuf_ucmds();
        uc_clear_impl(buf_ucmds);
    }
}

/// Implementation of `:delcommand` ex handler.
///
/// Deletes a user command by name, searching buffer-local first then global.
///
/// # Safety
///
/// `eap` must be a valid ExargHandle (pointer to exarg_T).
unsafe fn ex_delcommand_impl(eap: ExargHandle) {
    let mut cmd: *mut c_void = std::ptr::null_mut();
    let mut res: c_int = -1;
    let mut arg: *const c_char = nvim_uc_eap_get_arg(eap);
    let mut buffer_only = false;

    // Check for -buffer flag
    let arg_len = strlen_safe(arg);
    if arg_len >= 7 {
        let arg_slice = unsafe { std::slice::from_raw_parts(arg.cast::<u8>(), arg_len) };
        if &arg_slice[..7] == b"-buffer" {
            let after = arg_slice.get(7).copied().unwrap_or(0);
            if after == b' ' || after == b'\t' {
                buffer_only = true;
                arg = nvim_uc_skipwhite(unsafe { arg.add(7) });
            }
        }
    }

    let ucmds_ptr = nvim_uc_get_ucmds();
    let mut gap = nvim_uc_get_curbuf_ucmds();
    let mut i: c_int;

    loop {
        let ga_len = nvim_uc_ga_get_len(gap);
        i = 0;
        while i < ga_len {
            cmd = nvim_uc_ga_get_cmd(gap, i);
            let cmd_name = nvim_uc_cmd_get_name(cmd);
            res = strcmp_c(arg, cmd_name);
            if res <= 0 {
                break;
            }
            i += 1;
        }
        // If we didn't find it in any iteration, res stays from last compare
        // or stays -1 if ga_len was 0
        if gap == ucmds_ptr || res == 0 || buffer_only {
            break;
        }
        gap = ucmds_ptr;
    }

    if res != 0 {
        if buffer_only {
            nvim_uc_semsg_1(
                c"E1237: No such user-defined command in current buffer: %s".as_ptr(),
                arg,
            );
        } else {
            nvim_uc_semsg_1(c"E184: No such user-defined command: %s".as_ptr(), arg);
        }
        return;
    }

    nvim_uc_free_ucmd(cmd);

    let ga_len = nvim_uc_ga_get_len(gap);
    nvim_uc_ga_set_len(gap, ga_len - 1);

    if i < ga_len - 1 {
        nvim_uc_cmd_memmove_up(gap, i);
    }
}

/// Compute strlen of a NUL-terminated C string, safely.
unsafe fn strlen_safe(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0usize;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

/// Compare two NUL-terminated C strings (like C strcmp).
/// Returns negative if a < b, 0 if equal, positive if a > b.
unsafe fn strcmp_c(a: *const c_char, b: *const c_char) -> c_int {
    let mut i = 0usize;
    loop {
        let ca = unsafe { *a.add(i) as u8 };
        let cb = unsafe { *b.add(i) as u8 };
        if ca != cb {
            return c_int::from(ca) - c_int::from(cb);
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
}

// =============================================================================
// Phase 6: FFI Exports
// =============================================================================

/// FFI export: `:command` handler.
///
/// Direct replacement for C `ex_command`.
#[export_name = "ex_command"]
pub unsafe extern "C" fn rs_ex_command(eap: ExargHandle) {
    ex_command_impl(eap);
}

/// FFI export: `:comclear` handler.
///
/// Direct replacement for C `ex_comclear`.
#[export_name = "ex_comclear"]
pub unsafe extern "C" fn rs_ex_comclear(eap: ExargHandle) {
    ex_comclear_impl(eap);
}

/// FFI export: `:delcommand` handler.
///
/// Direct replacement for C `ex_delcommand`.
#[export_name = "ex_delcommand"]
pub unsafe extern "C" fn rs_ex_delcommand(eap: ExargHandle) {
    ex_delcommand_impl(eap);
}

// =============================================================================
// C Accessor Functions (Phase 8 — find_ucmd, uc_list, completion)
// =============================================================================

/// CMD_USER from ex_cmds_enum.generated.h
const CMD_USER: c_int = -1;
/// CMD_USER_BUF from ex_cmds_enum.generated.h
const CMD_USER_BUF: c_int = -2;
/// EXPAND_UNSUCCESSFUL from cmdexpand_defs.h
const EXPAND_UNSUCCESSFUL: c_int = -2;
/// HLF_D — directories in CTRL-D listing (highlight_defs.h, enum value 5)
const HLF_D: c_int = 5;
/// HLF_8 — meta & special keys (highlight_defs.h, enum value 1)
const HLF_8: c_int = 1;

extern "C" {
    // --- find_ucmd accessors ---
    /// Returns &prevwin_curwin()->w_buffer->b_ucmds (garray)
    fn nvim_uc_prevwin_curwin_buf_ucmds() -> GarrayHandle;
    /// Sets eap->cmdidx = (cmdidx_T)cmdidx
    fn nvim_uc_eap_set_cmdidx(eap: ExargHandle, cmdidx: c_int);
    /// Sets eap->argt = argt
    fn nvim_uc_eap_set_argt(eap: ExargHandle, argt: u32);
    /// Sets eap->useridx = useridx
    fn nvim_uc_eap_set_useridx(eap: ExargHandle, useridx: c_int);
    /// Sets eap->addr_type = (cmd_addr_T)addr_type
    fn nvim_uc_eap_set_addr_type(eap: ExargHandle, addr_type: c_int);
    /// Returns eap->cmd
    fn nvim_uc_eap_get_cmd(eap: ExargHandle) -> *const c_char;
    /// Returns cmd->uc_compl
    fn nvim_uc_cmd_get_compl(cmd: *const c_void) -> c_int;
    /// Returns (int)cmd->uc_addr_type
    fn nvim_uc_cmd_get_addr_type(cmd: *const c_void) -> c_int;
    /// Returns cmd->uc_compl_luaref
    fn nvim_uc_cmd_get_compl_luaref(cmd: *const c_void) -> c_int;
    /// Returns cmd->uc_compl_arg
    fn nvim_uc_cmd_get_compl_arg(cmd: *const c_void) -> *const c_char;
    /// Returns ascii_isdigit(c) ? 1 : 0
    fn nvim_uc_ascii_isdigit(c: c_int) -> c_int;

    // --- expand_T (xp) accessors ---
    /// Sets xp->xp_context = context
    fn nvim_uc_xp_set_context(xp: ExpandHandle, context: c_int);
    /// Sets xp->xp_luaref = luaref
    fn nvim_uc_xp_set_luaref(xp: ExpandHandle, luaref: c_int);
    /// Sets xp->xp_arg = arg
    fn nvim_uc_xp_set_arg(xp: ExpandHandle, arg: *mut c_char);
    /// Sets xp->xp_script_ctx from cmd->uc_script_ctx + SOURCING_LNUM
    fn nvim_uc_xp_set_script_ctx(xp: ExpandHandle, cmd: *const c_void);

    // --- uc_list accessors ---
    /// Calls msg_ext_set_kind(kind)
    fn nvim_uc_msg_ext_set_kind(kind: *const c_char);
    /// Calls msg_puts_title(_(s))
    fn nvim_uc_msg_puts_title(s: *const c_char);
    /// Calls msg_putchar(c)
    fn nvim_uc_msg_putchar(c: c_int);
    /// Calls msg_puts(s)
    fn nvim_uc_msg_puts(s: *const c_char);
    /// Calls msg_outtrans(s, attr, keep != 0)
    fn nvim_uc_msg_outtrans(s: *const c_char, attr: c_int, keep: c_int);
    /// Calls msg_outtrans_special(s, from_part != 0, maxlen)
    fn nvim_uc_msg_outtrans_special(s: *const c_char, from_part: c_int, maxlen: c_int);
    /// Calls msg_puts_hl(s, attr, keep != 0)
    fn nvim_uc_msg_puts_hl(s: *const c_char, attr: c_int, keep: c_int);
    /// Calls msg(_(s), attr)
    fn nvim_uc_msg(s: *const c_char, attr: c_int);
    /// Returns got_int
    fn nvim_uc_got_int() -> c_int;
    /// Calls line_breakcheck()
    fn nvim_uc_line_breakcheck();
    /// Returns message_filtered(msg) ? 1 : 0
    fn nvim_uc_message_filtered(msg: *const c_char) -> c_int;
    /// Returns p_verbose
    fn nvim_uc_get_p_verbose() -> c_int;
    /// Returns Columns
    fn nvim_uc_get_Columns() -> c_int;
    /// Returns IObuff pointer
    fn nvim_uc_get_IObuff() -> *mut c_char;
    /// Returns IOSIZE
    fn nvim_uc_get_IOSIZE() -> usize;
    /// Returns nlua_funcref_str(luaref, NULL) — caller must xfree
    fn nvim_uc_nlua_funcref_str(luaref: c_int) -> *mut c_char;
    /// Calls last_set_msg(cmd->uc_script_ctx)
    fn nvim_uc_last_set_msg(cmd: *const c_void);
    /// Returns cmd->uc_luaref
    fn nvim_uc_cmd_get_luaref(cmd: *mut c_void) -> c_int;
    /// Returns cmd->uc_rep
    fn nvim_uc_cmd_get_rep(cmd: *mut c_void) -> *mut c_char;
    /// Returns cmd->uc_argt
    fn nvim_uc_cmd_get_argt(cmd: *mut c_void) -> u32;
    /// Returns cmd->uc_def (as int64_t)
    fn nvim_uc_cmd_get_def(cmd: *mut c_void) -> i64;

    // --- Completion type lookup (already migrated to Rust) ---
    /// get_command_complete — returns the name for an EXPAND_* value (exported from complete.rs)
    #[link_name = "get_command_complete"]
    fn rs_get_command_complete(arg: c_int) -> *const c_char;
}

// =============================================================================
// Phase 8: find_ucmd implementation
// =============================================================================

/// Search for a user command that matches `eap->cmd`.
///
/// Sets cmdidx, argt, useridx, addr_type in eap.
/// Optionally sets xp fields and complp.
/// Returns a pointer to just after the command, or NULL if no match/ambiguous.
///
/// # Safety
///
/// `eap` and `p` must be valid pointers. `full`, `xp`, `complp` may be null.
unsafe fn find_ucmd_impl(
    eap: ExargHandle,
    p: *mut c_char,
    full: *mut c_int,
    xp: ExpandHandle,
    complp: *mut c_int,
) -> *mut c_char {
    let eap_cmd = nvim_uc_eap_get_cmd(eap);
    let len = unsafe { p.offset_from(eap_cmd) } as c_int;
    let mut matchlen: c_int = 0;
    let mut found = false;
    let mut possible = false;
    let mut amb_local = false;

    let ucmds = nvim_uc_get_ucmds();
    // Look for buffer-local user commands first, then global ones.
    let mut gap = nvim_uc_prevwin_curwin_buf_ucmds();
    loop {
        let ga_len = nvim_uc_ga_get_len(gap);
        let mut j = 0;
        while j < ga_len {
            let uc = nvim_uc_ga_get_cmd(gap, j);
            let uc_name = nvim_uc_cmd_get_name(uc);
            let mut cp = eap_cmd;
            let mut np = uc_name;
            let mut k: c_int = 0;
            while k < len && unsafe { *np } != 0 && unsafe { *cp } == unsafe { *np } {
                cp = unsafe { cp.add(1) };
                np = unsafe { np.add(1) };
                k += 1;
            }

            if k == len
                || (unsafe { *np } == 0
                    && nvim_uc_ascii_isdigit(
                        c_int::from(unsafe { *eap_cmd.add(k as usize) } as u8),
                    ) != 0)
            {
                // If finding a second match, the command is ambiguous.
                // But not if a buffer-local command wasn't a full match and
                // a global command is a full match.
                if k == len && found && unsafe { *np } != 0 {
                    if gap == ucmds {
                        return std::ptr::null_mut();
                    }
                    amb_local = true;
                }

                if !found || (k == len && unsafe { *np } == 0) {
                    if k == len {
                        found = true;
                    } else {
                        possible = true;
                    }

                    if gap == ucmds {
                        nvim_uc_eap_set_cmdidx(eap, CMD_USER);
                    } else {
                        nvim_uc_eap_set_cmdidx(eap, CMD_USER_BUF);
                    }
                    nvim_uc_eap_set_argt(eap, nvim_uc_cmd_get_argt(uc));
                    nvim_uc_eap_set_useridx(eap, j);
                    nvim_uc_eap_set_addr_type(eap, nvim_uc_cmd_get_addr_type(uc));

                    if !complp.is_null() {
                        unsafe { *complp = nvim_uc_cmd_get_compl(uc) };
                    }
                    if !xp.is_null() {
                        nvim_uc_xp_set_luaref(xp, nvim_uc_cmd_get_compl_luaref(uc));
                        nvim_uc_xp_set_arg(xp, nvim_uc_cmd_get_compl_arg(uc) as *mut c_char);
                        nvim_uc_xp_set_script_ctx(xp, uc);
                    }
                    // Do not search for further abbreviations if this is an exact match.
                    matchlen = k;
                    if k == len && unsafe { *np } == 0 {
                        if !full.is_null() {
                            unsafe { *full = 1 };
                        }
                        amb_local = false;
                        break;
                    }
                }
            }
            j += 1;
        }

        // Stop if we found a full match or searched all.
        if j < ga_len || gap == ucmds {
            break;
        }
        gap = ucmds;
    }

    // Only found ambiguous matches.
    if amb_local {
        if !xp.is_null() {
            nvim_uc_xp_set_context(xp, EXPAND_UNSUCCESSFUL);
        }
        return std::ptr::null_mut();
    }

    // The match we found may be followed immediately by a number. Move "p"
    // back to point to it.
    if found || possible {
        return unsafe { p.offset((matchlen - len) as isize) };
    }
    p
}

// =============================================================================
// Phase 8: uc_list implementation
// =============================================================================

/// List user commands matching a name prefix.
///
/// This is the Rust implementation of `uc_list` from usercmd.c.
///
/// # Safety
///
/// `name` must be a valid NUL-terminated C string. `name_len` is the prefix length.
unsafe fn uc_list_impl(name: *const c_char, name_len: usize) {
    let mut found = false;

    nvim_uc_msg_ext_set_kind(c"list_cmd".as_ptr());

    let ucmds = nvim_uc_get_ucmds();
    // In cmdwin, the alternative buffer should be used.
    let mut gap = nvim_uc_prevwin_curwin_buf_ucmds();
    loop {
        let ga_len = nvim_uc_ga_get_len(gap);
        let mut i = 0;
        while i < ga_len {
            let cmd = nvim_uc_ga_get_cmd(gap, i);
            let a = nvim_uc_cmd_get_argt(cmd);
            let cmd_name = nvim_uc_cmd_get_name(cmd);

            // Skip commands which don't match the requested prefix and
            // commands filtered out.
            if !strncmp_eq(name, cmd_name, name_len) || nvim_uc_message_filtered(cmd_name) != 0 {
                i += 1;
                continue;
            }

            // Put out the title first time
            if !found {
                nvim_uc_msg_puts_title(
                    c"\n    Name              Args Address Complete    Definition".as_ptr(),
                );
            }
            found = true;
            nvim_uc_msg_putchar(b'\n' as c_int);
            if nvim_uc_got_int() != 0 {
                break;
            }

            // Special cases
            let mut flag_len: usize = 4;
            if (a & EX_BANG) != 0 {
                nvim_uc_msg_putchar(b'!' as c_int);
                flag_len -= 1;
            }
            if (a & EX_REGSTR) != 0 {
                nvim_uc_msg_putchar(b'"' as c_int);
                flag_len -= 1;
            }
            if gap != ucmds {
                nvim_uc_msg_putchar(b'b' as c_int);
                flag_len -= 1;
            }
            if (a & EX_TRLBAR) != 0 {
                nvim_uc_msg_putchar(b'|' as c_int);
                flag_len -= 1;
            }
            if flag_len != 0 {
                // Emit spaces for remaining flag_len
                static SPACES4: &[u8] = b"    \0";
                nvim_uc_msg_puts(unsafe { SPACES4.as_ptr().add(4 - flag_len) }.cast::<c_char>());
            }

            nvim_uc_msg_outtrans(cmd_name, HLF_D, 0);
            let name_slen = strlen_safe(cmd_name);
            let mut col_len: usize = name_slen + 4;

            if col_len < 21 {
                // Pad with spaces to column 21
                static SPACES17: &[u8] = b"                 \0";
                nvim_uc_msg_puts(unsafe { SPACES17.as_ptr().add(col_len - 4) }.cast::<c_char>());
                col_len = 21;
            }
            nvim_uc_msg_putchar(b' ' as c_int);
            col_len += 1;

            // "over" is how much longer the name is than the column width
            let over: i64 = col_len as i64 - 22;

            // Build the IObuff content
            let iobuff = nvim_uc_get_IObuff();
            let iosize = nvim_uc_get_IOSIZE();
            let mut pos: usize = 0;

            // Arguments
            let nargs_char = match a & (EX_EXTRA | EX_NOSPC | EX_NEEDARG) {
                0 => b'0',
                x if x == EX_EXTRA => b'*',
                x if x == (EX_EXTRA | EX_NOSPC) => b'?',
                x if x == (EX_EXTRA | EX_NEEDARG) => b'+',
                x if x == (EX_EXTRA | EX_NOSPC | EX_NEEDARG) => b'1',
                _ => b' ',
            };
            unsafe { *iobuff.add(pos) = nargs_char as c_char };
            pos += 1;

            // Pad to column 5 - over
            loop {
                unsafe { *iobuff.add(pos) = b' ' as c_char };
                pos += 1;
                if (pos as i64) >= 5 - over {
                    break;
                }
            }

            // Address / Range
            if (a & (EX_RANGE | EX_COUNT)) != 0 {
                if (a & EX_COUNT) != 0 {
                    // -count=N
                    let def_val = nvim_uc_cmd_get_def(cmd);
                    let formatted = format!("{def_val}c");
                    let bytes = formatted.as_bytes();
                    let copy_len = bytes.len().min(iosize - pos - 1);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            iobuff.add(pos).cast::<u8>(),
                            copy_len,
                        );
                    }
                    pos += copy_len;
                } else if (a & EX_DFLALL) != 0 {
                    unsafe { *iobuff.add(pos) = b'%' as c_char };
                    pos += 1;
                } else if nvim_uc_cmd_get_def(cmd) >= 0 {
                    // -range=N
                    let def_val = nvim_uc_cmd_get_def(cmd);
                    let formatted = format!("{def_val}");
                    let bytes = formatted.as_bytes();
                    let copy_len = bytes.len().min(iosize - pos - 1);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            iobuff.add(pos).cast::<u8>(),
                            copy_len,
                        );
                    }
                    pos += copy_len;
                } else {
                    unsafe { *iobuff.add(pos) = b'.' as c_char };
                    pos += 1;
                }
            }

            // Pad to column 8 - over
            loop {
                unsafe { *iobuff.add(pos) = b' ' as c_char };
                pos += 1;
                if (pos as i64) >= 8 - over {
                    break;
                }
            }

            // Address Type
            let cmd_addr_type = nvim_uc_cmd_get_addr_type(cmd);
            for entry in ADDR_TYPE_COMPLETE.iter() {
                if entry.expand == AddrType::None {
                    break;
                }
                if entry.expand != AddrType::Lines && entry.expand as c_int == cmd_addr_type {
                    // Copy shortname (without trailing NUL from the static)
                    let shortname = &entry.shortname[..entry.shortname.len() - 1]; // strip \0
                    let copy_len = shortname.len().min(iosize - pos - 1);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            shortname.as_ptr(),
                            iobuff.add(pos).cast::<u8>(),
                            copy_len,
                        );
                    }
                    pos += copy_len;
                    break;
                }
            }

            // Pad to column 13 - over
            loop {
                unsafe { *iobuff.add(pos) = b' ' as c_char };
                pos += 1;
                if (pos as i64) >= 13 - over {
                    break;
                }
            }

            // Completion
            let cmd_compl_val = nvim_uc_cmd_get_compl(cmd);
            let cmd_compl = rs_get_command_complete(cmd_compl_val);
            if !cmd_compl.is_null() {
                let compl_len = strlen_safe(cmd_compl);
                let copy_len = compl_len.min(iosize - pos - 1);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        cmd_compl.cast::<u8>(),
                        iobuff.add(pos).cast::<u8>(),
                        copy_len,
                    );
                }
                pos += copy_len;
            }

            // Pad to column 25 - over
            loop {
                unsafe { *iobuff.add(pos) = b' ' as c_char };
                pos += 1;
                if (pos as i64) >= 25 - over {
                    break;
                }
            }

            // NUL-terminate
            unsafe { *iobuff.add(pos) = 0 };
            nvim_uc_msg_outtrans(iobuff, 0, 0);

            // Lua function reference
            let luaref = nvim_uc_cmd_get_luaref(cmd);
            if luaref != LUA_NOREF {
                let fn_str = nvim_uc_nlua_funcref_str(luaref);
                nvim_uc_msg_puts_hl(fn_str, HLF_8, 0);
                nvim_uc_xfree(fn_str.cast::<c_void>());
                // put the description on a new line
                let rep = nvim_uc_cmd_get_rep(cmd);
                if !rep.is_null() && unsafe { *rep } != 0 {
                    nvim_uc_msg_puts(c"\n                                               ".as_ptr());
                }
            }

            let rep = nvim_uc_cmd_get_rep(cmd);
            let maxlen = if name_len == 0 {
                nvim_uc_get_Columns() - 47
            } else {
                0
            };
            nvim_uc_msg_outtrans_special(rep, 0, maxlen);
            if nvim_uc_get_p_verbose() > 0 {
                nvim_uc_last_set_msg(cmd);
            }
            nvim_uc_line_breakcheck();
            if nvim_uc_got_int() != 0 {
                break;
            }
            i += 1;
        }
        if gap == ucmds || i < ga_len {
            break;
        }
        gap = ucmds;
    }

    if !found {
        nvim_uc_msg(c"No user-defined commands found".as_ptr(), 0);
    }
}

/// Compare first `n` bytes of two C strings (like strncmp == 0).
unsafe fn strncmp_eq(a: *const c_char, b: *const c_char, n: usize) -> bool {
    for i in 0..n {
        let ca = unsafe { *a.add(i) as u8 };
        let cb = unsafe { *b.add(i) as u8 };
        if ca != cb {
            return false;
        }
    }
    true
}

// =============================================================================
// Phase 8: FFI Exports
// =============================================================================

/// FFI export: find_ucmd.
///
/// Direct replacement for C `find_ucmd`.
#[export_name = "find_ucmd"]
pub unsafe extern "C" fn rs_find_ucmd(
    eap: ExargHandle,
    p: *mut c_char,
    full: *mut c_int,
    xp: ExpandHandle,
    complp: *mut c_int,
) -> *mut c_char {
    find_ucmd_impl(eap, p, full, xp, complp)
}

/// FFI export: uc_list.
///
/// Direct replacement for C `uc_list`.
#[export_name = "uc_list"]
pub unsafe extern "C" fn rs_uc_list(name: *const c_char, name_len: usize) {
    uc_list_impl(name, name_len);
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

    // =========================================================================
    // Phase 5: constants and logic tests
    // =========================================================================

    #[test]
    fn test_c_return_constants() {
        assert_eq!(C_OK, 1);
        assert_eq!(C_FAIL, 0);
        assert_eq!(LUA_NOREF, -2);
    }

    #[test]
    fn test_uc_buffer_flag_usage() {
        // Test that UC_BUFFER flag check works the same way as in C
        let flags_with_buffer: c_int = UC_BUFFER;
        assert_ne!(flags_with_buffer & UC_BUFFER, 0);

        let flags_without_buffer: c_int = 0;
        assert_eq!(flags_without_buffer & UC_BUFFER, 0);
    }

    #[test]
    fn test_name_comparison_logic() {
        // Test the comparison logic used in find_cmd_index
        // This mirrors the strncmp + length comparison from uc_add_command

        // Same prefix, different lengths
        let name = b"Cmd";
        let existing = b"CmdLonger";
        let compare_len = name.len().min(existing.len());
        let cmp = name[..compare_len].cmp(&existing[..compare_len]);
        assert_eq!(cmp, std::cmp::Ordering::Equal);
        // name_len (3) < existing_len (9) → should be -1
        assert!(name.len() < existing.len());

        // Exact match
        let name = b"Hello";
        let existing = b"Hello";
        let compare_len = name.len().min(existing.len());
        let cmp = name[..compare_len].cmp(&existing[..compare_len]);
        assert_eq!(cmp, std::cmp::Ordering::Equal);
        assert_eq!(name.len(), existing.len());

        // Different prefix
        let name = b"Alpha";
        let existing = b"Beta";
        let compare_len = name.len().min(existing.len());
        let cmp = name[..compare_len].cmp(&existing[..compare_len]);
        assert_eq!(cmp, std::cmp::Ordering::Less);

        let name = b"Zebra";
        let existing = b"Alpha";
        let compare_len = name.len().min(existing.len());
        let cmp = name[..compare_len].cmp(&existing[..compare_len]);
        assert_eq!(cmp, std::cmp::Ordering::Greater);
    }

    // =========================================================================
    // Phase 6: ex command handler tests
    // =========================================================================

    #[test]
    fn test_ascii_isupper() {
        assert!(ascii_isupper(b'A'));
        assert!(ascii_isupper(b'Z'));
        assert!(ascii_isupper(b'M'));
        assert!(!ascii_isupper(b'a'));
        assert!(!ascii_isupper(b'z'));
        assert!(!ascii_isupper(b'0'));
        assert!(!ascii_isupper(b' '));
        assert!(!ascii_isupper(b'@'));
        assert!(!ascii_isupper(b'['));
    }

    #[test]
    fn test_strcmp_c_equal() {
        let a = c"hello";
        let b = c"hello";
        assert_eq!(unsafe { strcmp_c(a.as_ptr(), b.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_c_less() {
        let a = c"abc";
        let b = c"abd";
        assert!(unsafe { strcmp_c(a.as_ptr(), b.as_ptr()) } < 0);
    }

    #[test]
    fn test_strcmp_c_greater() {
        let a = c"abd";
        let b = c"abc";
        assert!(unsafe { strcmp_c(a.as_ptr(), b.as_ptr()) } > 0);
    }

    #[test]
    fn test_strcmp_c_prefix() {
        let a = c"abc";
        let b = c"abcdef";
        assert!(unsafe { strcmp_c(a.as_ptr(), b.as_ptr()) } < 0);
    }

    #[test]
    fn test_strcmp_c_empty() {
        let a = c"";
        let b = c"";
        assert_eq!(unsafe { strcmp_c(a.as_ptr(), b.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_c_one_empty() {
        let a = c"";
        let b = c"a";
        assert!(unsafe { strcmp_c(a.as_ptr(), b.as_ptr()) } < 0);
    }

    #[test]
    fn test_strlen_safe_basic() {
        let s = c"hello";
        assert_eq!(unsafe { strlen_safe(s.as_ptr()) }, 5);
    }

    #[test]
    fn test_strlen_safe_empty() {
        let s = c"";
        assert_eq!(unsafe { strlen_safe(s.as_ptr()) }, 0);
    }

    #[test]
    fn test_strlen_safe_null() {
        assert_eq!(unsafe { strlen_safe(std::ptr::null()) }, 0);
    }

    #[test]
    fn test_phase6_constants() {
        assert_eq!(EXPAND_NOTHING, 0);
        assert_eq!(ADDR_NONE, 11);
    }

    #[test]
    fn test_reserved_name_check() {
        // Test the reserved name "Next" check logic
        let next = b"Next";

        // Exact match with length 4
        let name = b"Next";
        assert_eq!(&name[..4], &next[..4]);

        // Prefix match with length 3
        let name = b"Nex";
        assert_eq!(&name[..3], &next[..3]);

        // Prefix match with length 2
        let name = b"Ne";
        assert_eq!(&name[..2], &next[..2]);

        // Non-match
        let name = b"Noxt";
        assert_ne!(&name[..4], &next[..4]);
    }

    // =========================================================================
    // Phase 8: constants and logic tests
    // =========================================================================

    #[test]
    fn test_phase8_constants() {
        assert_eq!(CMD_USER, -1);
        assert_eq!(CMD_USER_BUF, -2);
        assert_eq!(EXPAND_UNSUCCESSFUL, -2);
        assert_eq!(HLF_D, 5);
        assert_eq!(HLF_8, 1);
    }

    #[test]
    fn test_strncmp_eq() {
        let a = c"Hello";
        let b = c"Hello World";
        assert!(unsafe { strncmp_eq(a.as_ptr(), b.as_ptr(), 5) });
        assert!(!unsafe { strncmp_eq(a.as_ptr(), c"Hxllo".as_ptr(), 5) });
        // Zero-length always matches
        assert!(unsafe { strncmp_eq(a.as_ptr(), b.as_ptr(), 0) });
    }

    #[test]
    fn test_nargs_char_mapping() {
        // Verify the nargs character mapping matches C
        let test_cases: [(u32, u8); 5] = [
            (0, b'0'),
            (EX_EXTRA, b'*'),
            (EX_EXTRA | EX_NOSPC, b'?'),
            (EX_EXTRA | EX_NEEDARG, b'+'),
            (EX_EXTRA | EX_NOSPC | EX_NEEDARG, b'1'),
        ];
        for (flags, expected) in &test_cases {
            let ch = match *flags & (EX_EXTRA | EX_NOSPC | EX_NEEDARG) {
                0 => b'0',
                x if x == EX_EXTRA => b'*',
                x if x == (EX_EXTRA | EX_NOSPC) => b'?',
                x if x == (EX_EXTRA | EX_NEEDARG) => b'+',
                x if x == (EX_EXTRA | EX_NOSPC | EX_NEEDARG) => b'1',
                _ => b' ',
            };
            assert_eq!(ch, *expected, "flags={flags:#x}");
        }
    }
}
