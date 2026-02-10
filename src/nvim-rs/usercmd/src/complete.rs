//! User command completion handling
//!
//! This module provides Rust implementations for user command completion,
//! including completion types, matching, and result handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::if_same_then_else)]

use std::ffi::c_int;

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
// FFI Exports
// =============================================================================

/// FFI export: Check if EXPAND_* value is in valid range
#[no_mangle]
pub extern "C" fn rs_usercmd_expand_valid(expand: c_int) -> c_int {
    c_int::from((EXPAND_UNSUCCESSFUL..=EXPAND_LUA).contains(&expand))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_constant_values() {
        // Verify key EXPAND_* constants match cmdexpand_defs.h
        assert_eq!(EXPAND_UNSUCCESSFUL, -2);
        assert_eq!(EXPAND_OK, -1);
        assert_eq!(EXPAND_NOTHING, 0);
        assert_eq!(EXPAND_COMMANDS, 1);
        assert_eq!(EXPAND_FILES, 2);
        assert_eq!(EXPAND_DIRECTORIES, 3);
        assert_eq!(EXPAND_SETTINGS, 4);
        assert_eq!(EXPAND_BOOL_SETTINGS, 5);
        assert_eq!(EXPAND_TAGS, 6);
        assert_eq!(EXPAND_OLD_SETTING, 7);
        assert_eq!(EXPAND_HELP, 8);
        assert_eq!(EXPAND_BUFFERS, 9);
        assert_eq!(EXPAND_EVENTS, 10);
        assert_eq!(EXPAND_MENUS, 11);
        assert_eq!(EXPAND_SYNTAX, 12);
        assert_eq!(EXPAND_HIGHLIGHT, 13);
        assert_eq!(EXPAND_AUGROUP, 14);
        assert_eq!(EXPAND_USER_VARS, 15);
        assert_eq!(EXPAND_MAPPINGS, 16);
        assert_eq!(EXPAND_TAGS_LISTFILES, 17);
        assert_eq!(EXPAND_FUNCTIONS, 18);
        assert_eq!(EXPAND_USER_FUNC, 19);
        assert_eq!(EXPAND_EXPRESSION, 20);
        assert_eq!(EXPAND_MENUNAMES, 21);
        assert_eq!(EXPAND_USER_COMMANDS, 22);
        assert_eq!(EXPAND_USER_CMD_FLAGS, 23);
        assert_eq!(EXPAND_USER_NARGS, 24);
        assert_eq!(EXPAND_USER_COMPLETE, 25);
        assert_eq!(EXPAND_ENV_VARS, 26);
        assert_eq!(EXPAND_LANGUAGE, 27);
        assert_eq!(EXPAND_COLORS, 28);
        assert_eq!(EXPAND_COMPILER, 29);
        assert_eq!(EXPAND_USER_DEFINED, 30);
        assert_eq!(EXPAND_USER_LIST, 31);
        assert_eq!(EXPAND_USER_LUA, 32);
        assert_eq!(EXPAND_SHELLCMD, 33);
        assert_eq!(EXPAND_SIGN, 34);
        assert_eq!(EXPAND_PROFILE, 35);
        assert_eq!(EXPAND_FILETYPE, 36);
        assert_eq!(EXPAND_FILES_IN_PATH, 37);
        assert_eq!(EXPAND_OWNSYNTAX, 38);
        assert_eq!(EXPAND_LOCALES, 39);
        assert_eq!(EXPAND_HISTORY, 40);
        assert_eq!(EXPAND_USER, 41);
        assert_eq!(EXPAND_SYNTIME, 42);
        assert_eq!(EXPAND_USER_ADDR_TYPE, 43);
        assert_eq!(EXPAND_PACKADD, 44);
        assert_eq!(EXPAND_MESSAGES, 45);
        assert_eq!(EXPAND_MAPCLEAR, 46);
        assert_eq!(EXPAND_ARGLIST, 47);
        assert_eq!(EXPAND_DIFF_BUFFERS, 48);
        assert_eq!(EXPAND_BREAKPOINT, 49);
        assert_eq!(EXPAND_SCRIPTNAMES, 50);
        assert_eq!(EXPAND_RUNTIME, 51);
        assert_eq!(EXPAND_STRING_SETTING, 52);
        assert_eq!(EXPAND_SETTING_SUBTRACT, 53);
        assert_eq!(EXPAND_ARGOPT, 54);
        assert_eq!(EXPAND_KEYMAP, 55);
        assert_eq!(EXPAND_DIRS_IN_CDPATH, 56);
        assert_eq!(EXPAND_SHELLCMDLINE, 57);
        assert_eq!(EXPAND_FINDFUNC, 58);
        assert_eq!(EXPAND_FILETYPECMD, 59);
        assert_eq!(EXPAND_PATTERN_IN_BUF, 60);
        assert_eq!(EXPAND_RETAB, 61);
        assert_eq!(EXPAND_CHECKHEALTH, 62);
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
