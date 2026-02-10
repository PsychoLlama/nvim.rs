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

use std::ffi::{c_char, c_int};

use crate::AddrType;

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
static COMMAND_COMPLETE: [Option<&[u8]>; EXPAND_COUNT] = {
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
#[no_mangle]
pub extern "C" fn rs_get_command_complete(arg: c_int) -> *const c_char {
    get_command_complete(arg).map_or(std::ptr::null(), |s| s.as_ptr().cast::<c_char>())
}

/// Get completion type names for expansion (ExpandGeneric callback).
///
/// Returns NULL if idx is out of range.
/// Skips entries that are NULL or EXPAND_USER_LUA (shown as `"<Lua function>"`).
/// Mirrors C `get_user_cmd_complete`.
#[no_mangle]
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
/// Mirrors C `cmdcomplete_type_to_str`.
#[no_mangle]
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

/// Convert a completion string to its EXPAND_* value.
///
/// Returns EXPAND_NOTHING if not found.
/// Mirrors C `cmdcomplete_str_to_type`.
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
}
