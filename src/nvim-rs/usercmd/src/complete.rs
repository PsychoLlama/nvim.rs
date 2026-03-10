//! User command completion handling
//!
//! This module provides Rust implementations for user command completion,
//! including completion types, lookup tables, and string conversion.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::as_ptr_cast_mut)]

use std::ffi::{c_char, c_int, c_void};

use crate::define::EX_XFILE;
use crate::{AddrType, ExpandHandle};

// =============================================================================
// EXPAND_* Constants — match cmdexpand_defs.h exactly
// =============================================================================

pub const EXPAND_UNSUCCESSFUL: c_int = -2;
pub const EXPAND_OK: c_int = -1;
pub const EXPAND_NOTHING: c_int = 0;
pub const EXPAND_COMMANDS: c_int = 1;
pub const EXPAND_FILES: c_int = 2;
pub const EXPAND_DIRECTORIES: c_int = 3;
pub const EXPAND_SETTINGS: c_int = 4;
pub const EXPAND_BOOL_SETTINGS: c_int = 5;
pub const EXPAND_TAGS: c_int = 6;
pub const EXPAND_OLD_SETTING: c_int = 7;
pub const EXPAND_HELP: c_int = 8;
pub const EXPAND_BUFFERS: c_int = 9;
pub const EXPAND_EVENTS: c_int = 10;
pub const EXPAND_MENUS: c_int = 11;
pub const EXPAND_SYNTAX: c_int = 12;
pub const EXPAND_HIGHLIGHT: c_int = 13;
pub const EXPAND_AUGROUP: c_int = 14;
pub const EXPAND_USER_VARS: c_int = 15;
pub const EXPAND_MAPPINGS: c_int = 16;
pub const EXPAND_TAGS_LISTFILES: c_int = 17;
pub const EXPAND_FUNCTIONS: c_int = 18;
pub const EXPAND_USER_FUNC: c_int = 19;
pub const EXPAND_EXPRESSION: c_int = 20;
pub const EXPAND_MENUNAMES: c_int = 21;
pub const EXPAND_USER_COMMANDS: c_int = 22;
pub const EXPAND_USER_CMD_FLAGS: c_int = 23;
pub const EXPAND_USER_NARGS: c_int = 24;
pub const EXPAND_USER_COMPLETE: c_int = 25;
pub const EXPAND_ENV_VARS: c_int = 26;
pub const EXPAND_LANGUAGE: c_int = 27;
pub const EXPAND_COLORS: c_int = 28;
pub const EXPAND_COMPILER: c_int = 29;
pub const EXPAND_USER_DEFINED: c_int = 30;
pub const EXPAND_USER_LIST: c_int = 31;
pub const EXPAND_USER_LUA: c_int = 32;
pub const EXPAND_SHELLCMD: c_int = 33;
pub const EXPAND_SIGN: c_int = 34;
pub const EXPAND_PROFILE: c_int = 35;
pub const EXPAND_FILETYPE: c_int = 36;
pub const EXPAND_FILES_IN_PATH: c_int = 37;
pub const EXPAND_OWNSYNTAX: c_int = 38;
pub const EXPAND_LOCALES: c_int = 39;
pub const EXPAND_HISTORY: c_int = 40;
pub const EXPAND_USER: c_int = 41;
pub const EXPAND_SYNTIME: c_int = 42;
pub const EXPAND_USER_ADDR_TYPE: c_int = 43;
pub const EXPAND_PACKADD: c_int = 44;
pub const EXPAND_MESSAGES: c_int = 45;
pub const EXPAND_MAPCLEAR: c_int = 46;
pub const EXPAND_ARGLIST: c_int = 47;
pub const EXPAND_DIFF_BUFFERS: c_int = 48;
pub const EXPAND_BREAKPOINT: c_int = 49;
pub const EXPAND_SCRIPTNAMES: c_int = 50;
pub const EXPAND_RUNTIME: c_int = 51;
pub const EXPAND_STRING_SETTING: c_int = 52;
pub const EXPAND_SETTING_SUBTRACT: c_int = 53;
pub const EXPAND_ARGOPT: c_int = 54;
pub const EXPAND_KEYMAP: c_int = 55;
pub const EXPAND_DIRS_IN_CDPATH: c_int = 56;
pub const EXPAND_SHELLCMDLINE: c_int = 57;
pub const EXPAND_FINDFUNC: c_int = 58;
pub const EXPAND_FILETYPECMD: c_int = 59;
pub const EXPAND_PATTERN_IN_BUF: c_int = 60;
pub const EXPAND_RETAB: c_int = 61;
pub const EXPAND_CHECKHEALTH: c_int = 62;
pub const EXPAND_LUA: c_int = 63;

/// Total number of EXPAND_* entries (0 through EXPAND_LUA)
pub const EXPAND_COUNT: usize = 64;

// =============================================================================
// command_complete[] — sparse table indexed by EXPAND_* value
// Mirrors the C `command_complete` array in usercmd.c.
// Entries not in the table are NULL (None here).
// =============================================================================

/// Get the completion name for an EXPAND_* value (mirrors C `get_command_complete`).
/// Returns None for out-of-range or unset entries.
pub fn get_command_complete(arg: c_int) -> Option<&'static [u8]> {
    if arg < 0 || arg >= EXPAND_COUNT as c_int {
        return None;
    }
    COMMAND_COMPLETE[arg as usize]
}

/// Sparse table mapping EXPAND_* index → name (NUL-terminated byte string).
/// Indices with no entry are None. Matches usercmd.c `command_complete[]`.
pub static COMMAND_COMPLETE: [Option<&[u8]>; EXPAND_COUNT] = {
    let mut table: [Option<&[u8]>; EXPAND_COUNT] = [None; EXPAND_COUNT];
    table[EXPAND_ARGLIST as usize] = Some(b"arglist\0");
    table[EXPAND_AUGROUP as usize] = Some(b"augroup\0");
    table[EXPAND_BUFFERS as usize] = Some(b"buffer\0");
    table[EXPAND_CHECKHEALTH as usize] = Some(b"checkhealth\0");
    table[EXPAND_COLORS as usize] = Some(b"color\0");
    table[EXPAND_COMMANDS as usize] = Some(b"command\0");
    table[EXPAND_COMPILER as usize] = Some(b"compiler\0");
    table[EXPAND_USER_DEFINED as usize] = Some(b"custom\0");
    table[EXPAND_USER_LIST as usize] = Some(b"customlist\0");
    table[EXPAND_USER_LUA as usize] = Some(b"<Lua function>\0");
    table[EXPAND_DIFF_BUFFERS as usize] = Some(b"diff_buffer\0");
    table[EXPAND_DIRECTORIES as usize] = Some(b"dir\0");
    table[EXPAND_ENV_VARS as usize] = Some(b"environment\0");
    table[EXPAND_EVENTS as usize] = Some(b"event\0");
    table[EXPAND_EXPRESSION as usize] = Some(b"expression\0");
    table[EXPAND_FILES as usize] = Some(b"file\0");
    table[EXPAND_FILES_IN_PATH as usize] = Some(b"file_in_path\0");
    table[EXPAND_FILETYPE as usize] = Some(b"filetype\0");
    table[EXPAND_FILETYPECMD as usize] = Some(b"filetypecmd\0");
    table[EXPAND_FUNCTIONS as usize] = Some(b"function\0");
    table[EXPAND_HELP as usize] = Some(b"help\0");
    table[EXPAND_HIGHLIGHT as usize] = Some(b"highlight\0");
    table[EXPAND_HISTORY as usize] = Some(b"history\0");
    table[EXPAND_KEYMAP as usize] = Some(b"keymap\0");
    table[EXPAND_LOCALES as usize] = Some(b"locale\0");
    table[EXPAND_LUA as usize] = Some(b"lua\0");
    table[EXPAND_MAPCLEAR as usize] = Some(b"mapclear\0");
    table[EXPAND_MAPPINGS as usize] = Some(b"mapping\0");
    table[EXPAND_MENUS as usize] = Some(b"menu\0");
    table[EXPAND_MESSAGES as usize] = Some(b"messages\0");
    table[EXPAND_OWNSYNTAX as usize] = Some(b"syntax\0");
    table[EXPAND_SYNTIME as usize] = Some(b"syntime\0");
    table[EXPAND_SETTINGS as usize] = Some(b"option\0");
    table[EXPAND_PACKADD as usize] = Some(b"packadd\0");
    table[EXPAND_RETAB as usize] = Some(b"retab\0");
    table[EXPAND_RUNTIME as usize] = Some(b"runtime\0");
    table[EXPAND_SHELLCMD as usize] = Some(b"shellcmd\0");
    table[EXPAND_SHELLCMDLINE as usize] = Some(b"shellcmdline\0");
    table[EXPAND_SIGN as usize] = Some(b"sign\0");
    table[EXPAND_TAGS as usize] = Some(b"tag\0");
    table[EXPAND_TAGS_LISTFILES as usize] = Some(b"tag_listfiles\0");
    table[EXPAND_USER as usize] = Some(b"user\0");
    table[EXPAND_USER_VARS as usize] = Some(b"var\0");
    table[EXPAND_BREAKPOINT as usize] = Some(b"breakpoint\0");
    table[EXPAND_SCRIPTNAMES as usize] = Some(b"scriptnames\0");
    table[EXPAND_DIRS_IN_CDPATH as usize] = Some(b"dir_in_path\0");
    table
};

// =============================================================================
// addr_type_complete[] — address type lookup table
// =============================================================================

/// Address type completion entry
pub struct AddrTypeEntry {
    pub expand: AddrType,
    pub name: &'static [u8],
    pub shortname: &'static [u8],
}

/// Address type completion table (matches C `addr_type_complete[]`).
/// Last entry has expand=ADDR_NONE, name/shortname empty (sentinel).
pub static ADDR_TYPE_COMPLETE: &[AddrTypeEntry] = &[
    AddrTypeEntry {
        expand: AddrType::Arguments,
        name: b"arguments\0",
        shortname: b"arg\0",
    },
    AddrTypeEntry {
        expand: AddrType::Lines,
        name: b"lines\0",
        shortname: b"line\0",
    },
    AddrTypeEntry {
        expand: AddrType::LoadedBuffers,
        name: b"loaded_buffers\0",
        shortname: b"load\0",
    },
    AddrTypeEntry {
        expand: AddrType::Tabs,
        name: b"tabs\0",
        shortname: b"tab\0",
    },
    AddrTypeEntry {
        expand: AddrType::Buffers,
        name: b"buffers\0",
        shortname: b"buf\0",
    },
    AddrTypeEntry {
        expand: AddrType::Windows,
        name: b"windows\0",
        shortname: b"win\0",
    },
    AddrTypeEntry {
        expand: AddrType::Quickfix,
        name: b"quickfix\0",
        shortname: b"qf\0",
    },
    AddrTypeEntry {
        expand: AddrType::Other,
        name: b"other\0",
        shortname: b"?\0",
    },
    // Sentinel
    AddrTypeEntry {
        expand: AddrType::None,
        name: b"\0",
        shortname: b"\0",
    },
];

/// User command flags for `:command -` completion
static USER_CMD_FLAGS: &[&[u8]] = &[
    b"addr\0",
    b"bang\0",
    b"bar\0",
    b"buffer\0",
    b"complete\0",
    b"count\0",
    b"nargs\0",
    b"range\0",
    b"register\0",
    b"keepscript\0",
];

/// Nargs values for `:command -nargs=` completion
static USER_CMD_NARGS: &[&[u8]] = &[b"0\0", b"1\0", b"*\0", b"?\0", b"+\0"];

// =============================================================================
// Completion Type Enum (Rust-only, for type-safe use in Rust code)
// =============================================================================

/// Completion type for user commands.
/// This is a Rust-side type with sequential values for internal use.
/// When interfacing with C, use the EXPAND_* constants directly.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompleteType {
    /// No completion
    None = 0,
    /// Argument list
    Arglist = 1,
    /// Autocommand groups
    Augroup = 2,
    /// Buffer names
    Buffer = 3,
    /// Color schemes
    Color = 4,
    /// Commands
    Command = 5,
    /// Compilers
    Compiler = 6,
    /// Directories
    Dir = 7,
    /// Environment variables
    Environment = 8,
    /// Autocommand events
    Event = 9,
    /// Expression
    Expression = 10,
    /// Files and directories
    File = 11,
    /// Files in path
    FileInPath = 12,
    /// Filetype names
    Filetype = 13,
    /// Functions
    Function = 14,
    /// Help topics
    Help = 15,
    /// Highlight groups
    Highlight = 16,
    /// History types
    History = 17,
    /// Locale names
    Locale = 18,
    /// Lua expression
    Lua = 19,
    /// Mappings
    Mapping = 20,
    /// Menus
    Menu = 21,
    /// Messages
    Messages = 22,
    /// Options
    Option = 23,
    /// Packages
    Packadd = 24,
    /// Shell commands
    Shellcmd = 25,
    /// Signs
    Sign = 26,
    /// Syntax items
    Syntax = 27,
    /// Syntax time
    Syntime = 28,
    /// Tags
    Tag = 29,
    /// Tag files
    TagListfiles = 30,
    /// User names
    User = 31,
    /// User variables
    Var = 32,
    /// Custom function (user-defined function)
    Custom = 33,
    /// Custom list (user-defined function returning list)
    CustomList = 34,
}

impl Default for CompleteType {
    fn default() -> Self {
        Self::None
    }
}

// =============================================================================
// FFI Exports — Phase 1 functions
// =============================================================================

/// Get the completion name for an EXPAND_* value.
///
/// Returns NULL for out-of-range or unset entries.
/// Mirrors C `get_command_complete`.
#[export_name = "get_command_complete"]
pub extern "C" fn rs_get_command_complete(arg: c_int) -> *const c_char {
    get_command_complete(arg).map_or(std::ptr::null(), |s| s.as_ptr().cast::<c_char>())
}

/// Get completion type names for expansion (ExpandGeneric callback).
///
/// Returns NULL if idx is out of range.
/// Skips entries that are NULL or EXPAND_USER_LUA (shown as `"<Lua function>"`).
/// Mirrors C `get_user_cmd_complete`.
#[export_name = "get_user_cmd_complete"]
pub extern "C" fn rs_get_user_cmd_complete(idx: c_int) -> *const c_char {
    if idx < 0 || idx >= EXPAND_COUNT as c_int {
        return std::ptr::null();
    }
    let entry = get_command_complete(idx);
    match entry {
        None | Some(b"<Lua function>\0") => c"".as_ptr(),
        Some(s) => s.as_ptr().cast::<c_char>(),
    }
}

/// Convert an EXPAND_* value + compl_arg to a string in the provided buffer.
///
/// Returns the length written (or needed if buf is NULL), or -1 on error.
/// Internal helper used by both Rust tests and the public FFI export.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_cmdcomplete_type_to_str(
    expand: c_int,
    compl_arg: *const c_char,
    buf: *mut c_char,
    buflen: usize,
) -> c_int {
    let Some(cmd_compl) = get_command_complete(expand) else {
        return -1;
    };
    // EXPAND_USER_LUA is not representable as a string
    if expand == EXPAND_USER_LUA {
        return -1;
    }

    // For custom/customlist, format as "type,arg"
    if (expand == EXPAND_USER_LIST || expand == EXPAND_USER_DEFINED) && !compl_arg.is_null() {
        if buf.is_null() {
            // Return needed length (without NUL)
            let name_len = cmd_compl.len() - 1; // exclude our trailing NUL
            let arg_len = unsafe { libc_strlen(compl_arg) };
            return (name_len + 1 + arg_len) as c_int; // name + comma + arg
        }
        // Write "type,arg" into buf
        let name_len = cmd_compl.len() - 1;
        let arg_len = unsafe { libc_strlen(compl_arg) };
        let total = name_len + 1 + arg_len;
        if total >= buflen {
            return -1;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(cmd_compl.as_ptr(), buf.cast::<u8>(), name_len);
            *buf.add(name_len) = b',' as c_char;
            std::ptr::copy_nonoverlapping(
                compl_arg.cast::<u8>(),
                buf.cast::<u8>().add(name_len + 1),
                arg_len,
            );
            *buf.add(total) = 0;
        }
        return total as c_int;
    }

    // Simple case: just the name
    if buf.is_null() {
        return (cmd_compl.len() - 1) as c_int; // exclude NUL
    }
    let name_len = cmd_compl.len() - 1;
    if name_len >= buflen {
        return -1;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(cmd_compl.as_ptr(), buf.cast::<u8>(), name_len);
        *buf.add(name_len) = 0;
    }
    name_len as c_int
}

extern "C" {
    #[link_name = "xmalloc"]
    fn c_xmalloc(size: usize) -> *mut c_void;
}

/// Get completion type as an allocated C string.
///
/// "compl_arg" is the function name for "custom" and "customlist" types.
/// Returns NULL if no completion is available. Replaces C `cmdcomplete_type_to_str`.
#[export_name = "cmdcomplete_type_to_str"]
pub unsafe extern "C" fn cmdcomplete_type_to_str_export(
    expand: c_int,
    compl_arg: *const c_char,
) -> *mut c_char {
    // Query length from helper
    let len = rs_cmdcomplete_type_to_str(expand, compl_arg, std::ptr::null_mut(), 0);
    if len < 0 {
        return std::ptr::null_mut();
    }
    let buf: *mut c_char = unsafe { c_xmalloc((len as usize) + 1).cast::<c_char>() };
    rs_cmdcomplete_type_to_str(expand, compl_arg, buf, (len as usize) + 1);
    buf
}

/// Convert a completion string to its EXPAND_* value.
///
/// Returns EXPAND_NOTHING if not found.
/// Mirrors C `cmdcomplete_str_to_type`.
#[export_name = "cmdcomplete_str_to_type"]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_usercmd_str_to_type(complete_str: *const c_char) -> c_int {
    if complete_str.is_null() {
        return EXPAND_NOTHING;
    }

    let s = unsafe { c_str_as_bytes(complete_str) };

    // Check "custom," and "customlist," prefixes
    if s.starts_with(b"custom,") {
        return EXPAND_USER_DEFINED;
    }
    if s.starts_with(b"customlist,") {
        return EXPAND_USER_LIST;
    }

    // Search the command_complete table
    for (i, slot) in COMMAND_COMPLETE.iter().enumerate() {
        if let Some(entry) = slot {
            let name = &entry[..entry.len() - 1]; // strip NUL
            if s == name {
                return i as c_int;
            }
        }
    }

    EXPAND_NOTHING
}

/// Get addr type name by index (ExpandGeneric callback).
/// Returns NULL if idx is past the table (sentinel has NULL name).
/// Mirrors C `get_user_cmd_addr_type`.
#[export_name = "get_user_cmd_addr_type"]
pub extern "C" fn rs_get_user_cmd_addr_type(idx: c_int) -> *const c_char {
    if idx < 0 {
        return std::ptr::null();
    }
    let i = idx as usize;
    if i >= ADDR_TYPE_COMPLETE.len() {
        return std::ptr::null();
    }
    let entry = &ADDR_TYPE_COMPLETE[i];
    if entry.expand == AddrType::None {
        return std::ptr::null();
    }
    entry.name.as_ptr().cast::<c_char>()
}

/// Get flag name by index (ExpandGeneric callback).
/// Returns NULL if idx is out of range.
/// Mirrors C `get_user_cmd_flags`.
#[export_name = "get_user_cmd_flags"]
pub extern "C" fn rs_get_user_cmd_flags(idx: c_int) -> *const c_char {
    if idx < 0 {
        return std::ptr::null();
    }
    let i = idx as usize;
    if i >= USER_CMD_FLAGS.len() {
        return std::ptr::null();
    }
    USER_CMD_FLAGS[i].as_ptr().cast::<c_char>()
}

/// Get nargs value by index (ExpandGeneric callback).
/// Returns NULL if idx is out of range.
/// Mirrors C `get_user_cmd_nargs`.
#[export_name = "get_user_cmd_nargs"]
pub extern "C" fn rs_get_user_cmd_nargs(idx: c_int) -> *const c_char {
    if idx < 0 {
        return std::ptr::null();
    }
    let i = idx as usize;
    if i >= USER_CMD_NARGS.len() {
        return std::ptr::null();
    }
    USER_CMD_NARGS[i].as_ptr().cast::<c_char>()
}

/// FFI export: Check if EXPAND_* value is in valid range
#[no_mangle]
pub extern "C" fn rs_usercmd_expand_valid(expand: c_int) -> c_int {
    c_int::from((EXPAND_UNSUCCESSFUL..=EXPAND_LUA).contains(&expand))
}

// =============================================================================
// Helpers
// =============================================================================

/// Get length of a C string (like strlen).
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

/// View a C string as a byte slice (without the NUL terminator).
unsafe fn c_str_as_bytes(s: *const c_char) -> &'static [u8] {
    let len = unsafe { libc_strlen(s) };
    unsafe { std::slice::from_raw_parts(s.cast::<u8>(), len) }
}

// =============================================================================
// C Accessor Functions (Phase 8 — completion context)
// =============================================================================

/// CMD_USER from ex_cmds_enum.generated.h
const CMD_USER: c_int = -1;
/// CMD_USER_BUF from ex_cmds_enum.generated.h
const CMD_USER_BUF: c_int = -2;
/// CMD_SIZE from ex_cmds_enum.generated.h (number of built-in commands)
const CMD_SIZE: c_int = 556;

extern "C" {
    // String navigation helpers
    /// Returns skiptowhite(p) — pointer to first whitespace
    fn nvim_uc_skiptowhite(p: *const c_char) -> *mut c_char;
    /// Returns skipwhite(p) — pointer past whitespace
    fn nvim_uc_skipwhite(p: *const c_char) -> *mut c_char;

    // expand_T (xp) accessors
    /// Sets xp->xp_context = context
    fn nvim_uc_xp_set_context(xp: ExpandHandle, context: c_int);
    /// Sets xp->xp_pattern = pattern
    fn nvim_uc_xp_set_pattern(xp: ExpandHandle, pattern: *mut c_char);

    // set_context_in_user_cmdarg delegations
    /// Calls set_context_in_menu_cmd(xp, cmd, arg, forceit != 0)
    fn nvim_uc_set_context_in_menu_cmd(
        xp: ExpandHandle,
        cmd: *const c_char,
        arg: *mut c_char,
        forceit: c_int,
    ) -> *const c_char;
    /// Calls set_context_in_map_cmd(xp, "map", arg, forceit, false, false, CMD_map)
    fn nvim_uc_set_context_in_map_cmd(
        xp: ExpandHandle,
        arg: *mut c_char,
        forceit: c_int,
    ) -> *const c_char;
    /// Does MB_PTR_ADV(*pp)
    fn nvim_uc_MB_PTR_ADV(pp: *mut *const c_char);

    // garray operations for user commands
    /// Returns prevwin_curwin()->w_buffer->b_ucmds.ga_len
    fn nvim_uc_prevwin_curwin_buf_ucmds_len() -> c_int;
    /// Returns USER_CMD_GA(&prevwin_curwin()->w_buffer->b_ucmds, i)
    fn nvim_uc_prevwin_curwin_buf_ucmd_ga(i: c_int) -> *mut c_void;
    /// Returns ucmds.ga_len
    fn nvim_uc_get_ucmds_len() -> c_int;
    /// Returns USER_CMD(idx) (global ucmds array)
    fn nvim_uc_user_cmd_global(idx: c_int) -> *mut c_void;

    // ucmd_T field getters
    /// Returns cmd->uc_name
    fn nvim_uc_cmd_get_name(cmd: *const c_void) -> *const c_char;
}

// =============================================================================
// Phase 8: Completion Context Functions
// =============================================================================

/// Case-insensitive comparison of first `n` bytes.
/// Returns true if the first `n` bytes of `a` and `b` are equal ignoring case.
///
/// # Safety
///
/// `a` and `b` must point to at least `n` bytes.
unsafe fn strnicmp_eq_bytes(a: *const c_char, b: &[u8], n: usize) -> bool {
    if n > b.len() {
        return false;
    }
    for i in 0..n {
        let ca = (unsafe { *a.add(i) } as u8).to_ascii_lowercase();
        let cb = b[i].to_ascii_lowercase();
        if ca != cb {
            return false;
        }
    }
    true
}

/// Implementation of `set_context_in_user_cmd`.
///
/// Sets completion context for the `:command` command.
///
/// # Safety
///
/// `xp` must be a valid expand_T pointer. `arg_in` must be a valid C string.
unsafe fn set_context_in_user_cmd_impl(xp: ExpandHandle, arg_in: *const c_char) -> *const c_char {
    let mut arg = arg_in;

    // Check for attributes
    while unsafe { *arg } == b'-' as c_char {
        arg = unsafe { arg.add(1) }; // Skip "-"
        let p_end = nvim_uc_skiptowhite(arg);
        if unsafe { *p_end } == 0 {
            // Cursor is still in the attribute.
            // Find '=' in arg
            let mut eq_pos: *const c_char = std::ptr::null();
            let mut scan = arg;
            while unsafe { *scan } != 0 {
                if unsafe { *scan } == b'=' as c_char {
                    eq_pos = scan;
                    break;
                }
                scan = unsafe { scan.add(1) };
            }

            if eq_pos.is_null() {
                // No "=", so complete attribute names.
                nvim_uc_xp_set_context(xp, EXPAND_USER_CMD_FLAGS);
                nvim_uc_xp_set_pattern(xp, arg as *mut c_char);
                return std::ptr::null();
            }

            // For the -complete, -nargs and -addr attributes, we complete
            // their arguments as well.
            let attr_len = unsafe { eq_pos.offset_from(arg) } as usize;
            if unsafe { strnicmp_eq_bytes(arg, b"complete", attr_len) } {
                nvim_uc_xp_set_context(xp, EXPAND_USER_COMPLETE);
                nvim_uc_xp_set_pattern(xp, unsafe { eq_pos.add(1) } as *mut c_char);
                return std::ptr::null();
            } else if unsafe { strnicmp_eq_bytes(arg, b"nargs", attr_len) } {
                nvim_uc_xp_set_context(xp, EXPAND_USER_NARGS);
                nvim_uc_xp_set_pattern(xp, unsafe { eq_pos.add(1) } as *mut c_char);
                return std::ptr::null();
            } else if unsafe { strnicmp_eq_bytes(arg, b"addr", attr_len) } {
                nvim_uc_xp_set_context(xp, EXPAND_USER_ADDR_TYPE);
                nvim_uc_xp_set_pattern(xp, unsafe { eq_pos.add(1) } as *mut c_char);
                return std::ptr::null();
            }
            return std::ptr::null();
        }
        arg = nvim_uc_skipwhite(p_end) as *const c_char;
    }

    // After the attributes comes the new command name.
    let p = nvim_uc_skiptowhite(arg);
    if unsafe { *p } == 0 {
        nvim_uc_xp_set_context(xp, EXPAND_USER_COMMANDS);
        nvim_uc_xp_set_pattern(xp, arg as *mut c_char);
        return std::ptr::null();
    }

    // And finally comes a normal command.
    nvim_uc_skipwhite(p) as *const c_char
}

/// Implementation of `set_context_in_user_cmdarg`.
///
/// Sets the completion context for the argument of a user defined command.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn set_context_in_user_cmdarg_impl(
    cmd: *const c_char,
    arg: *const c_char,
    argt: u32,
    context: c_int,
    xp: ExpandHandle,
    forceit: c_int,
) -> *const c_char {
    if context == EXPAND_NOTHING {
        return std::ptr::null();
    }

    if (argt & EX_XFILE) != 0 {
        // EX_XFILE: file names are handled before this call.
        return std::ptr::null();
    }

    if context == EXPAND_MENUS {
        return nvim_uc_set_context_in_menu_cmd(xp, cmd, arg as *mut c_char, forceit);
    }
    if context == EXPAND_COMMANDS {
        return arg;
    }
    if context == EXPAND_MAPPINGS {
        return nvim_uc_set_context_in_map_cmd(xp, arg as *mut c_char, forceit);
    }

    // Find start of last argument.
    let mut p = arg;
    let mut last_arg = arg;
    while unsafe { *p } != 0 {
        if unsafe { *p } == b' ' as c_char {
            // argument starts after a space
            last_arg = unsafe { p.add(1) };
        } else if unsafe { *p } == b'\\' as c_char && unsafe { *p.add(1) } != 0 {
            p = unsafe { p.add(1) }; // skip over escaped character
        }
        nvim_uc_MB_PTR_ADV(&mut p);
    }
    nvim_uc_xp_set_pattern(xp, last_arg as *mut c_char);
    nvim_uc_xp_set_context(xp, context);

    std::ptr::null()
}

/// Implementation of `expand_user_command_name`.
///
/// Returns the command name at index `idx - CMD_SIZE`.
unsafe fn expand_user_command_name_impl(idx: c_int) -> *mut c_char {
    get_user_commands_impl(idx - CMD_SIZE)
}

/// Implementation of `get_user_commands`.
///
/// Gets user command names for expansion. Buffer-local first, then global.
/// Global commands overruled by buffer-local ones return "".
unsafe fn get_user_commands_impl(idx: c_int) -> *mut c_char {
    let buf_len = nvim_uc_prevwin_curwin_buf_ucmds_len();

    if idx < buf_len {
        return nvim_uc_cmd_get_name(nvim_uc_prevwin_curwin_buf_ucmd_ga(idx)) as *mut c_char;
    }

    let adjusted = idx - buf_len;
    let ucmds_len = nvim_uc_get_ucmds_len();
    if adjusted < ucmds_len {
        let name = nvim_uc_cmd_get_name(nvim_uc_user_cmd_global(adjusted));

        // Check if global command is overruled by buffer-local one
        for i in 0..buf_len {
            let buf_cmd_name = nvim_uc_cmd_get_name(nvim_uc_prevwin_curwin_buf_ucmd_ga(i));
            if strcmp_c(name, buf_cmd_name) == 0 {
                // global command is overruled by buffer-local one
                return c"".as_ptr() as *mut c_char;
            }
        }
        return name as *mut c_char;
    }
    std::ptr::null_mut()
}

/// Implementation of `get_user_command_name`.
///
/// Gets command name by index and cmdidx (CMD_USER or CMD_USER_BUF).
unsafe fn get_user_command_name_impl(idx: c_int, cmdidx: c_int) -> *mut c_char {
    if cmdidx == CMD_USER && idx < nvim_uc_get_ucmds_len() {
        return nvim_uc_cmd_get_name(nvim_uc_user_cmd_global(idx)) as *mut c_char;
    }
    if cmdidx == CMD_USER_BUF {
        let buf_len = nvim_uc_prevwin_curwin_buf_ucmds_len();
        if idx < buf_len {
            return nvim_uc_cmd_get_name(nvim_uc_prevwin_curwin_buf_ucmd_ga(idx)) as *mut c_char;
        }
    }
    std::ptr::null_mut()
}

/// Compare two NUL-terminated C strings (like C strcmp).
/// Returns 0 if equal.
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
// Phase 8: FFI Exports
// =============================================================================

/// FFI export: set_context_in_user_cmd.
#[export_name = "set_context_in_user_cmd"]
pub unsafe extern "C" fn rs_set_context_in_user_cmd(
    xp: ExpandHandle,
    arg_in: *const c_char,
) -> *const c_char {
    set_context_in_user_cmd_impl(xp, arg_in)
}

/// FFI export: set_context_in_user_cmdarg.
/// C signature: `(const char *cmd, const char *arg, uint32_t argt, int context, expand_T *xp, bool forceit)`
#[export_name = "set_context_in_user_cmdarg"]
pub unsafe extern "C" fn rs_set_context_in_user_cmdarg(
    cmd: *const c_char,
    arg: *const c_char,
    argt: u32,
    context: c_int,
    xp: ExpandHandle,
    forceit: bool,
) -> *const c_char {
    set_context_in_user_cmdarg_impl(cmd, arg, argt, context, xp, c_int::from(forceit))
}

/// FFI export: expand_user_command_name.
#[export_name = "expand_user_command_name"]
pub unsafe extern "C" fn rs_expand_user_command_name(idx: c_int) -> *mut c_char {
    expand_user_command_name_impl(idx)
}

/// FFI export: get_user_commands.
#[export_name = "get_user_commands"]
pub unsafe extern "C" fn rs_get_user_commands(_xp: ExpandHandle, idx: c_int) -> *mut c_char {
    get_user_commands_impl(idx)
}

/// FFI export: get_user_command_name.
#[export_name = "get_user_command_name"]
pub unsafe extern "C" fn rs_get_user_command_name(idx: c_int, cmdidx: c_int) -> *mut c_char {
    get_user_command_name_impl(idx, cmdidx)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_complete() {
        // Valid entries
        assert_eq!(
            get_command_complete(EXPAND_ARGLIST),
            Some(&b"arglist\0"[..])
        );
        assert_eq!(get_command_complete(EXPAND_BUFFERS), Some(&b"buffer\0"[..]));
        assert_eq!(get_command_complete(EXPAND_LUA), Some(&b"lua\0"[..]));
        assert_eq!(
            get_command_complete(EXPAND_USER_LUA),
            Some(&b"<Lua function>\0"[..])
        );
        assert_eq!(
            get_command_complete(EXPAND_DIRS_IN_CDPATH),
            Some(&b"dir_in_path\0"[..])
        );

        // NULL / out-of-range entries
        assert_eq!(get_command_complete(-1), None);
        assert_eq!(get_command_complete(64), None);
        // EXPAND_NOTHING (0) has no entry
        assert_eq!(get_command_complete(EXPAND_NOTHING), None);
        // EXPAND_OLD_SETTING (7) has no entry
        assert_eq!(get_command_complete(EXPAND_OLD_SETTING), None);
    }

    #[test]
    fn test_addr_type_table() {
        // First entry is Arguments
        assert_eq!(ADDR_TYPE_COMPLETE[0].expand, AddrType::Arguments);
        assert_eq!(&ADDR_TYPE_COMPLETE[0].name[..9], b"arguments");

        // Last real entry is Other
        assert_eq!(ADDR_TYPE_COMPLETE[7].expand, AddrType::Other);

        // Sentinel
        assert_eq!(ADDR_TYPE_COMPLETE[8].expand, AddrType::None);
    }

    #[test]
    fn test_user_cmd_flags() {
        assert_eq!(USER_CMD_FLAGS.len(), 10);
        assert_eq!(&USER_CMD_FLAGS[0][..4], b"addr");
        assert_eq!(&USER_CMD_FLAGS[9][..10], b"keepscript");
    }

    #[test]
    fn test_user_cmd_nargs() {
        assert_eq!(USER_CMD_NARGS.len(), 5);
        assert_eq!(USER_CMD_NARGS[0], b"0\0");
        assert_eq!(USER_CMD_NARGS[4], b"+\0");
    }

    #[test]
    fn test_cmdcomplete_str_to_type() {
        // Basic lookups via FFI
        assert_eq!(rs_usercmd_str_to_type(c"arglist".as_ptr()), EXPAND_ARGLIST);
        assert_eq!(rs_usercmd_str_to_type(c"buffer".as_ptr()), EXPAND_BUFFERS);
        assert_eq!(rs_usercmd_str_to_type(c"lua".as_ptr()), EXPAND_LUA);

        // custom, / customlist, prefixes
        assert_eq!(
            rs_usercmd_str_to_type(c"custom,MyFunc".as_ptr()),
            EXPAND_USER_DEFINED
        );
        assert_eq!(
            rs_usercmd_str_to_type(c"customlist,MyFunc".as_ptr()),
            EXPAND_USER_LIST
        );

        // Unknown
        assert_eq!(
            rs_usercmd_str_to_type(c"nonexistent".as_ptr()),
            EXPAND_NOTHING
        );

        // NULL
        assert_eq!(rs_usercmd_str_to_type(std::ptr::null()), EXPAND_NOTHING);
    }

    #[test]
    fn test_rs_get_user_cmd_flags() {
        // Valid
        assert!(!rs_get_user_cmd_flags(0).is_null());
        assert!(!rs_get_user_cmd_flags(9).is_null());
        // Out of range
        assert!(rs_get_user_cmd_flags(10).is_null());
        assert!(rs_get_user_cmd_flags(-1).is_null());
    }

    #[test]
    fn test_rs_get_user_cmd_nargs() {
        assert!(!rs_get_user_cmd_nargs(0).is_null());
        assert!(!rs_get_user_cmd_nargs(4).is_null());
        assert!(rs_get_user_cmd_nargs(5).is_null());
        assert!(rs_get_user_cmd_nargs(-1).is_null());
    }

    #[test]
    fn test_rs_get_user_cmd_addr_type() {
        // First entry
        assert!(!rs_get_user_cmd_addr_type(0).is_null());
        // Last real entry (Other, index 7)
        assert!(!rs_get_user_cmd_addr_type(7).is_null());
        // Sentinel (index 8) — expand==ADDR_NONE, returns NULL
        assert!(rs_get_user_cmd_addr_type(8).is_null());
        // Out of range
        assert!(rs_get_user_cmd_addr_type(9).is_null());
        assert!(rs_get_user_cmd_addr_type(-1).is_null());
    }

    #[test]
    fn test_rs_get_command_complete() {
        // Valid
        let p = rs_get_command_complete(EXPAND_BUFFERS);
        assert!(!p.is_null());
        let s = unsafe { c_str_as_bytes(p) };
        assert_eq!(s, b"buffer");

        // NULL entry
        assert!(rs_get_command_complete(EXPAND_NOTHING).is_null());
        assert!(rs_get_command_complete(-1).is_null());
    }

    #[test]
    fn test_rs_get_user_cmd_complete() {
        // Normal entry
        let p = rs_get_user_cmd_complete(EXPAND_BUFFERS);
        assert!(!p.is_null());
        let s = unsafe { c_str_as_bytes(p) };
        assert_eq!(s, b"buffer");

        // USER_LUA returns empty string (not the "<Lua function>" entry)
        let p = rs_get_user_cmd_complete(EXPAND_USER_LUA);
        assert!(!p.is_null());
        let s = unsafe { c_str_as_bytes(p) };
        assert_eq!(s, b"");

        // NULL entry returns empty string
        let p = rs_get_user_cmd_complete(EXPAND_NOTHING);
        assert!(!p.is_null());
        let s = unsafe { c_str_as_bytes(p) };
        assert_eq!(s, b"");

        // Out of range returns NULL
        assert!(rs_get_user_cmd_complete(64).is_null());
    }

    #[test]
    fn test_cmdcomplete_type_to_str_simple() {
        let mut buf = [0u8; 64];
        // Simple case: "buffer"
        let len = rs_cmdcomplete_type_to_str(
            EXPAND_BUFFERS,
            std::ptr::null(),
            buf.as_mut_ptr().cast(),
            buf.len(),
        );
        assert_eq!(len, 6);
        assert_eq!(&buf[..6], b"buffer");

        // NULL buf → just return length
        let len =
            rs_cmdcomplete_type_to_str(EXPAND_BUFFERS, std::ptr::null(), std::ptr::null_mut(), 0);
        assert_eq!(len, 6);

        // EXPAND_USER_LUA → -1
        let len =
            rs_cmdcomplete_type_to_str(EXPAND_USER_LUA, std::ptr::null(), std::ptr::null_mut(), 0);
        assert_eq!(len, -1);

        // Out of range → -1
        let len = rs_cmdcomplete_type_to_str(-1, std::ptr::null(), std::ptr::null_mut(), 0);
        assert_eq!(len, -1);
    }

    #[test]
    fn test_cmdcomplete_type_to_str_custom() {
        let mut buf = [0u8; 64];
        let arg = c"MyFunc";
        // "custom,MyFunc"
        let len = rs_cmdcomplete_type_to_str(
            EXPAND_USER_DEFINED,
            arg.as_ptr(),
            buf.as_mut_ptr().cast(),
            buf.len(),
        );
        assert_eq!(len, 13); // "custom,MyFunc" = 13
        assert_eq!(&buf[..13], b"custom,MyFunc");

        // Length query
        let len =
            rs_cmdcomplete_type_to_str(EXPAND_USER_DEFINED, arg.as_ptr(), std::ptr::null_mut(), 0);
        assert_eq!(len, 13);
    }

    #[test]
    fn test_expand_constant_values() {
        assert_eq!(EXPAND_UNSUCCESSFUL, -2);
        assert_eq!(EXPAND_OK, -1);
        assert_eq!(EXPAND_NOTHING, 0);
        assert_eq!(EXPAND_COMMANDS, 1);
        assert_eq!(EXPAND_LUA, 63);
    }

    #[test]
    fn test_expand_valid() {
        assert_eq!(rs_usercmd_expand_valid(EXPAND_NOTHING), 1);
        assert_eq!(rs_usercmd_expand_valid(EXPAND_LUA), 1);
        assert_eq!(rs_usercmd_expand_valid(EXPAND_UNSUCCESSFUL), 1);
        assert_eq!(rs_usercmd_expand_valid(64), 0);
        assert_eq!(rs_usercmd_expand_valid(-3), 0);
    }

    // =========================================================================
    // Phase 8: completion context tests
    // =========================================================================

    #[test]
    fn test_phase8_constants() {
        assert_eq!(CMD_USER, -1);
        assert_eq!(CMD_USER_BUF, -2);
        assert_eq!(CMD_SIZE, 556);
    }

    #[test]
    fn test_strnicmp_eq_bytes() {
        assert!(unsafe { strnicmp_eq_bytes(c"complete".as_ptr(), b"complete", 8) });
        assert!(unsafe { strnicmp_eq_bytes(c"COMPLETE".as_ptr(), b"complete", 8) });
        assert!(unsafe { strnicmp_eq_bytes(c"Complete".as_ptr(), b"complete", 8) });
        assert!(unsafe { strnicmp_eq_bytes(c"comp".as_ptr(), b"complete", 4) });
        assert!(!unsafe { strnicmp_eq_bytes(c"compXete".as_ptr(), b"complete", 8) });
        // n > target length
        assert!(!unsafe { strnicmp_eq_bytes(c"complete".as_ptr(), b"comp", 8) });
    }

    #[test]
    fn test_strcmp_c() {
        assert_eq!(unsafe { strcmp_c(c"abc".as_ptr(), c"abc".as_ptr()) }, 0);
        assert!(unsafe { strcmp_c(c"abc".as_ptr(), c"abd".as_ptr()) } < 0);
        assert!(unsafe { strcmp_c(c"abd".as_ptr(), c"abc".as_ptr()) } > 0);
        assert!(unsafe { strcmp_c(c"abc".as_ptr(), c"abcd".as_ptr()) } < 0);
    }
}
