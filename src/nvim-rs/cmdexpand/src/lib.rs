//! Command-line completion and expansion for Neovim
//!
//! This crate provides the command-line completion engine, including:
//! - Wildcard expansion
//! - Completion source management
//! - Fuzzy matching integration
//! - Popup menu support for completions

#![allow(unsafe_code)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use libc::{c_char, c_int};
use std::ffi::CStr;

// =============================================================================
// Constants - Expansion context types (mirrors cmdexpand_defs.h)
// =============================================================================

/// Expansion context type constants
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandContext {
    Unsuccessful = -2,
    Ok = -1,
    Nothing = 0,
    Commands = 1,
    Files = 2,
    Directories = 3,
    Settings = 4,
    BoolSettings = 5,
    Tags = 6,
    OldSetting = 7,
    Help = 8,
    Buffers = 9,
    Events = 10,
    Menus = 11,
    Syntax = 12,
    Highlight = 13,
    Augroup = 14,
    UserVars = 15,
    Mappings = 16,
    TagsListfiles = 17,
    Functions = 18,
    UserFunc = 19,
    Expression = 20,
    Menunames = 21,
    UserCommands = 22,
    UserCmdFlags = 23,
    UserNargs = 24,
    UserComplete = 25,
    EnvVars = 26,
    Language = 27,
    Colors = 28,
    Compiler = 29,
    UserDefined = 30,
    UserList = 31,
    UserLua = 32,
    Shellcmd = 33,
    Sign = 34,
    Profile = 35,
    Filetype = 36,
    FilesInPath = 37,
    Ownsyntax = 38,
    Locales = 39,
    History = 40,
    User = 41,
    Syntime = 42,
    UserAddrType = 43,
    Packadd = 44,
    Messages = 45,
    Mapclear = 46,
    Arglist = 47,
    DiffBuffers = 48,
    Breakpoint = 49,
    Scriptnames = 50,
    Runtime = 51,
    StringSetting = 52,
    SettingSubtract = 53,
    Argopt = 54,
    Keymap = 55,
    DirsInCdpath = 56,
    Shellcmdline = 57,
    Findfunc = 58,
    Filetypecmd = 59,
    PatternInBuf = 60,
    Retab = 61,
    Checkhealth = 62,
    Lua = 63,
}

impl ExpandContext {
    /// Convert from raw i32 value
    #[must_use]
    pub const fn from_raw(value: i32) -> Option<Self> {
        match value {
            -2 => Some(Self::Unsuccessful),
            -1 => Some(Self::Ok),
            0 => Some(Self::Nothing),
            1 => Some(Self::Commands),
            2 => Some(Self::Files),
            3 => Some(Self::Directories),
            4 => Some(Self::Settings),
            5 => Some(Self::BoolSettings),
            6 => Some(Self::Tags),
            7 => Some(Self::OldSetting),
            8 => Some(Self::Help),
            9 => Some(Self::Buffers),
            10 => Some(Self::Events),
            11 => Some(Self::Menus),
            12 => Some(Self::Syntax),
            13 => Some(Self::Highlight),
            14 => Some(Self::Augroup),
            15 => Some(Self::UserVars),
            16 => Some(Self::Mappings),
            17 => Some(Self::TagsListfiles),
            18 => Some(Self::Functions),
            19 => Some(Self::UserFunc),
            20 => Some(Self::Expression),
            21 => Some(Self::Menunames),
            22 => Some(Self::UserCommands),
            23 => Some(Self::UserCmdFlags),
            24 => Some(Self::UserNargs),
            25 => Some(Self::UserComplete),
            26 => Some(Self::EnvVars),
            27 => Some(Self::Language),
            28 => Some(Self::Colors),
            29 => Some(Self::Compiler),
            30 => Some(Self::UserDefined),
            31 => Some(Self::UserList),
            32 => Some(Self::UserLua),
            33 => Some(Self::Shellcmd),
            34 => Some(Self::Sign),
            35 => Some(Self::Profile),
            36 => Some(Self::Filetype),
            37 => Some(Self::FilesInPath),
            38 => Some(Self::Ownsyntax),
            39 => Some(Self::Locales),
            40 => Some(Self::History),
            41 => Some(Self::User),
            42 => Some(Self::Syntime),
            43 => Some(Self::UserAddrType),
            44 => Some(Self::Packadd),
            45 => Some(Self::Messages),
            46 => Some(Self::Mapclear),
            47 => Some(Self::Arglist),
            48 => Some(Self::DiffBuffers),
            49 => Some(Self::Breakpoint),
            50 => Some(Self::Scriptnames),
            51 => Some(Self::Runtime),
            52 => Some(Self::StringSetting),
            53 => Some(Self::SettingSubtract),
            54 => Some(Self::Argopt),
            55 => Some(Self::Keymap),
            56 => Some(Self::DirsInCdpath),
            57 => Some(Self::Shellcmdline),
            58 => Some(Self::Findfunc),
            59 => Some(Self::Filetypecmd),
            60 => Some(Self::PatternInBuf),
            61 => Some(Self::Retab),
            62 => Some(Self::Checkhealth),
            63 => Some(Self::Lua),
            _ => None,
        }
    }

    /// Convert to raw i32 value
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        self as i32
    }
}

// =============================================================================
// Constants - Backslash handling flags
// =============================================================================

/// Backslash handling flags for expansion
pub mod backslash {
    /// Nothing special for backslashes
    pub const XP_BS_NONE: i32 = 0;
    /// Uses one backslash before a space
    pub const XP_BS_ONE: i32 = 0x1;
    /// Uses three backslashes before a space
    pub const XP_BS_THREE: i32 = 0x2;
    /// Commas need to be escaped with a backslash
    pub const XP_BS_COMMA: i32 = 0x4;
}

// =============================================================================
// Constants - Wild mode types
// =============================================================================

/// Wild expansion mode constants
pub mod wild_mode {
    pub const WILD_FREE: i32 = 1;
    pub const WILD_EXPAND_FREE: i32 = 2;
    pub const WILD_EXPAND_KEEP: i32 = 3;
    pub const WILD_NEXT: i32 = 4;
    pub const WILD_PREV: i32 = 5;
    pub const WILD_ALL: i32 = 6;
    pub const WILD_LONGEST: i32 = 7;
    pub const WILD_ALL_KEEP: i32 = 8;
    pub const WILD_CANCEL: i32 = 9;
    pub const WILD_APPLY: i32 = 10;
    pub const WILD_PAGEUP: i32 = 11;
    pub const WILD_PAGEDOWN: i32 = 12;
    pub const WILD_PUM_WANT: i32 = 13;
}

// =============================================================================
// Constants - Wild option flags
// =============================================================================

/// Wild expansion option flags
pub mod wild_options {
    pub const WILD_LIST_NOTFOUND: i32 = 0x01;
    pub const WILD_HOME_REPLACE: i32 = 0x02;
    pub const WILD_USE_NL: i32 = 0x04;
    pub const WILD_NO_BEEP: i32 = 0x08;
    pub const WILD_ADD_SLASH: i32 = 0x10;
    pub const WILD_KEEP_ALL: i32 = 0x20;
    pub const WILD_SILENT: i32 = 0x40;
    pub const WILD_ESCAPE: i32 = 0x80;
    pub const WILD_ICASE: i32 = 0x100;
    pub const WILD_ALLLINKS: i32 = 0x200;
    pub const WILD_IGNORE_COMPLETESLASH: i32 = 0x400;
    pub const WILD_NOERROR: i32 = 0x800;
    pub const WILD_BUFLASTUSED: i32 = 0x1000;
    pub const BUF_DIFF_FILTER: i32 = 0x2000;
    pub const WILD_NOSELECT: i32 = 0x4000;
    pub const WILD_MAY_EXPAND_PATTERN: i32 = 0x8000;
    pub const WILD_FUNC_TRIGGER: i32 = 0x1_0000;
}

// =============================================================================
// Constants - Wildoptions flags (from option_vars.h)
// =============================================================================

/// Flag for 'wildoptions' fuzzy matching
const K_OPT_WOP_FLAG_FUZZY: u32 = 0x01;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    fn nvim_get_wop_flags() -> libc::c_uint;
    fn nvim_get_compl_match_array_not_null() -> c_int;
}

// =============================================================================
// Fuzzy completion support
// =============================================================================

/// Returns true if fuzzy completion is supported for the given context.
///
/// Not all completion contexts support fuzzy matching. This function
/// checks the context type and returns whether fuzzy completion can be used.
#[must_use]
pub const fn cmdline_fuzzy_completion_supported(context: i32) -> bool {
    // These contexts do NOT support fuzzy completion
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return false;
    };

    !matches!(
        ctx,
        ExpandContext::BoolSettings
            | ExpandContext::Colors
            | ExpandContext::Compiler
            | ExpandContext::Directories
            | ExpandContext::DirsInCdpath
            | ExpandContext::Files
            | ExpandContext::FilesInPath
            | ExpandContext::Filetype
            | ExpandContext::Filetypecmd
            | ExpandContext::Findfunc
            | ExpandContext::Help
            | ExpandContext::Keymap
            | ExpandContext::Lua
            | ExpandContext::OldSetting
            | ExpandContext::StringSetting
            | ExpandContext::SettingSubtract
            | ExpandContext::Ownsyntax
            | ExpandContext::Packadd
            | ExpandContext::Runtime
            | ExpandContext::Shellcmd
            | ExpandContext::Shellcmdline
            | ExpandContext::Tags
            | ExpandContext::TagsListfiles
            | ExpandContext::UserList
            | ExpandContext::UserLua
    )
}

/// Check if fuzzy completion is enabled and the pattern is not empty.
///
/// Returns true if:
/// 1. The 'wildoptions' setting has the fuzzy flag set
/// 2. The fuzzy string is not empty
#[must_use]
pub fn cmdline_fuzzy_complete(fuzzystr: &str) -> bool {
    if fuzzystr.is_empty() {
        return false;
    }

    // Check if fuzzy flag is set in wildoptions
    // SAFETY: nvim_get_wop_flags is a simple accessor that reads a global variable
    let wop_flags = unsafe { nvim_get_wop_flags() };
    (wop_flags & K_OPT_WOP_FLAG_FUZZY) != 0
}

/// Check if the cmdline popup menu is active.
#[must_use]
pub fn cmdline_pum_active() -> bool {
    // SAFETY: nvim_get_compl_match_array_not_null is a simple accessor
    unsafe { nvim_get_compl_match_array_not_null() != 0 }
}

// =============================================================================
// FFI Interface
// =============================================================================

/// Convert C string pointer to Rust &str
///
/// # Safety
///
/// `ptr` must be a valid null-terminated C string or null.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

/// Check if fuzzy completion is enabled for the given string (FFI version).
///
/// # Safety
///
/// `fuzzystr` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_fuzzy_complete(fuzzystr: *const c_char) -> c_int {
    let Some(s) = cstr_to_str(fuzzystr) else {
        return 0;
    };

    c_int::from(cmdline_fuzzy_complete(s))
}

/// Check if cmdline popup menu is active (FFI version).
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_pum_active() -> c_int {
    c_int::from(cmdline_pum_active())
}

/// Check if fuzzy completion is supported for the given context (FFI version).
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_fuzzy_completion_supported(context: c_int) -> c_int {
    c_int::from(cmdline_fuzzy_completion_supported(context))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_context_conversion() {
        // Test round-trip conversion
        assert_eq!(
            ExpandContext::from_raw(ExpandContext::Commands.to_raw()),
            Some(ExpandContext::Commands)
        );
        assert_eq!(
            ExpandContext::from_raw(ExpandContext::Files.to_raw()),
            Some(ExpandContext::Files)
        );
        assert_eq!(
            ExpandContext::from_raw(ExpandContext::Nothing.to_raw()),
            Some(ExpandContext::Nothing)
        );

        // Test invalid values
        assert_eq!(ExpandContext::from_raw(999), None);
        assert_eq!(ExpandContext::from_raw(-10), None);
    }

    #[test]
    fn test_expand_context_values() {
        // Verify specific values match C header
        assert_eq!(ExpandContext::Unsuccessful.to_raw(), -2);
        assert_eq!(ExpandContext::Ok.to_raw(), -1);
        assert_eq!(ExpandContext::Nothing.to_raw(), 0);
        assert_eq!(ExpandContext::Commands.to_raw(), 1);
        assert_eq!(ExpandContext::Files.to_raw(), 2);
    }

    #[test]
    fn test_backslash_constants() {
        use backslash::*;
        assert_eq!(XP_BS_NONE, 0);
        assert_eq!(XP_BS_ONE, 0x1);
        assert_eq!(XP_BS_THREE, 0x2);
        assert_eq!(XP_BS_COMMA, 0x4);
    }

    #[test]
    fn test_wild_mode_constants() {
        use wild_mode::*;
        assert_eq!(WILD_FREE, 1);
        assert_eq!(WILD_EXPAND_FREE, 2);
        assert_eq!(WILD_NEXT, 4);
        assert_eq!(WILD_PREV, 5);
        assert_eq!(WILD_ALL, 6);
        assert_eq!(WILD_LONGEST, 7);
    }

    #[test]
    fn test_wild_options_constants() {
        use wild_options::*;
        assert_eq!(WILD_LIST_NOTFOUND, 0x01);
        assert_eq!(WILD_HOME_REPLACE, 0x02);
        assert_eq!(WILD_SILENT, 0x40);
        assert_eq!(WILD_ESCAPE, 0x80);
    }

    #[test]
    fn test_fuzzy_completion_supported() {
        // Files/directories do NOT support fuzzy completion
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Files.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Directories.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Help.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Tags.to_raw()
        ));

        // Commands and other contexts DO support fuzzy completion
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Commands.to_raw()
        ));
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Buffers.to_raw()
        ));
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Functions.to_raw()
        ));

        // Invalid context
        assert!(!cmdline_fuzzy_completion_supported(999));
    }
}
