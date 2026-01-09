//! Command line constants
//!
//! This module provides constants used in command-line mode, including
//! mode types, key handlers, and redraw states.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Command Line Mode Types
// =============================================================================

/// Command-line first character types
///
/// These characters determine the type of command line being edited.
pub mod firstc {
    /// Ex command prompt ':'
    pub const EX_CMD: i32 = b':' as i32;
    /// Forward search prompt '/'
    pub const FORWARD_SEARCH: i32 = b'/' as i32;
    /// Backward search prompt '?'
    pub const BACKWARD_SEARCH: i32 = b'?' as i32;
    /// Expression evaluation prompt '='
    pub const EXPRESSION: i32 = b'=' as i32;
    /// Debug command prompt '>'
    pub const DEBUG: i32 = b'>' as i32;
    /// Input function prompt '@'
    pub const INPUT_FN: i32 = b'@' as i32;
    /// No prompt
    pub const NONE: i32 = 0;
}

// =============================================================================
// Command Line Return Values
// =============================================================================

/// Return values from command_line_handle_key
pub mod cmdline_result {
    /// Command line was not changed
    pub const NOT_CHANGED: i32 = 1;
    /// Command line was changed
    pub const CHANGED: i32 = 2;
    /// Go to normal mode (ESC pressed)
    pub const GOTO_NORMAL_MODE: i32 = 3;
    /// Process next key
    pub const PROCESS_NEXT_KEY: i32 = 4;
}

// =============================================================================
// Redraw State
// =============================================================================

/// Redraw state for external cmdline
pub mod redraw_state {
    use std::ffi::c_int;

    /// No redraw needed
    pub const NONE: c_int = 0;
    /// Only position changed
    pub const POS: c_int = 1;
    /// Full redraw needed
    pub const ALL: c_int = 2;
}

// =============================================================================
// Expansion Context Types
// =============================================================================

/// Expansion context types for command-line completion
pub mod expand_context {
    use std::ffi::c_int;

    /// Unknown expansion
    pub const UNKNOWN: c_int = 0;
    /// Nothing to expand
    pub const NOTHING: c_int = 1;
    /// Expand command names
    pub const COMMANDS: c_int = 2;
    /// Expand file names
    pub const FILES: c_int = 3;
    /// Expand directories
    pub const DIRECTORIES: c_int = 4;
    /// Expand settings
    pub const SETTINGS: c_int = 5;
    /// Expand boolean settings
    pub const BOOL_SETTINGS: c_int = 6;
    /// Expand tags
    pub const TAGS: c_int = 7;
    /// Expand old tags
    pub const TAGS_LISTFILES: c_int = 8;
    /// Expand help
    pub const HELP: c_int = 9;
    /// Expand buffers
    pub const BUFFERS: c_int = 10;
    /// Expand shell commands
    pub const SHELLCMD: c_int = 11;
    /// Expand menus
    pub const MENUS: c_int = 12;
    /// Expand syntax items
    pub const SYNTAX: c_int = 13;
    /// Expand highlight groups
    pub const HIGHLIGHT: c_int = 14;
    /// Expand command-line history
    pub const HISTORY: c_int = 15;
    /// Expand user commands
    pub const USER_COMMANDS: c_int = 16;
    /// Expand user vars
    pub const USER_VARS: c_int = 17;
    /// Expand user-defined
    pub const USER_DEFINED: c_int = 18;
    /// Expand user-defined list
    pub const USER_LIST: c_int = 19;
    /// Expand mappings
    pub const MAPPINGS: c_int = 20;
    /// Expand functions
    pub const FUNCTIONS: c_int = 21;
    /// Expand user functions
    pub const USER_FUNC: c_int = 22;
    /// Expand expression
    pub const EXPRESSION: c_int = 23;
    /// Expand menunames
    pub const MENUNAMES: c_int = 24;
    /// Expand colors
    pub const COLORS: c_int = 25;
    /// Expand compilers
    pub const COMPILER: c_int = 26;
    /// Expand ownsyntax
    pub const OWNSYNTAX: c_int = 27;
    /// Expand locales
    pub const LOCALES: c_int = 28;
    /// Expand environment vars
    pub const ENV: c_int = 29;
    /// Expand language
    pub const LANGUAGE: c_int = 30;
    /// Expand cscope
    pub const CSCOPE: c_int = 31;
    /// Expand signs
    pub const SIGN: c_int = 32;
    /// Expand profiles
    pub const PROFILE: c_int = 33;
    /// Expand behave
    pub const BEHAVE: c_int = 34;
    /// Expand filetypes
    pub const FILETYPE: c_int = 35;
    /// Expand messages
    pub const MESSAGES: c_int = 36;
    /// Expand mapclear
    pub const MAPCLEAR: c_int = 37;
    /// Expand arglist
    pub const ARGLIST: c_int = 38;
    /// Expand diff_buffer
    pub const DIFF_BUFFERS: c_int = 39;
    /// Expand breakpoints
    pub const BREAKPOINT: c_int = 40;
    /// Expand scriptnames
    pub const SCRIPTNAMES: c_int = 41;
    /// Expand runtime
    pub const RUNTIME: c_int = 42;
    /// Expand checkhealth
    pub const CHECKHEALTH: c_int = 43;
    /// Expand lua
    pub const LUA: c_int = 44;
}

// =============================================================================
// Key Code Constants
// =============================================================================

/// Key code constants for command-line mode
pub mod keys {
    use std::ffi::c_int;

    /// Escape key
    pub const ESC: c_int = 27;
    /// Carriage return
    pub const CR: c_int = 13;
    /// Newline
    pub const NL: c_int = 10;
    /// Backspace
    pub const BS: c_int = 8;
    /// Tab
    pub const TAB: c_int = 9;

    // Control keys
    /// Ctrl-A
    pub const CTRL_A: c_int = 1;
    /// Ctrl-B
    pub const CTRL_B: c_int = 2;
    /// Ctrl-C
    pub const CTRL_C: c_int = 3;
    /// Ctrl-D
    pub const CTRL_D: c_int = 4;
    /// Ctrl-E
    pub const CTRL_E: c_int = 5;
    /// Ctrl-F
    pub const CTRL_F: c_int = 6;
    /// Ctrl-H (backspace)
    pub const CTRL_H: c_int = 8;
    /// Ctrl-I (tab)
    pub const CTRL_I: c_int = 9;
    /// Ctrl-J (newline)
    pub const CTRL_J: c_int = 10;
    /// Ctrl-K
    pub const CTRL_K: c_int = 11;
    /// Ctrl-L
    pub const CTRL_L: c_int = 12;
    /// Ctrl-M (carriage return)
    pub const CTRL_M: c_int = 13;
    /// Ctrl-N
    pub const CTRL_N: c_int = 14;
    /// Ctrl-O
    pub const CTRL_O: c_int = 15;
    /// Ctrl-P
    pub const CTRL_P: c_int = 16;
    /// Ctrl-Q
    pub const CTRL_Q: c_int = 17;
    /// Ctrl-R
    pub const CTRL_R: c_int = 18;
    /// Ctrl-S
    pub const CTRL_S: c_int = 19;
    /// Ctrl-T
    pub const CTRL_T: c_int = 20;
    /// Ctrl-U
    pub const CTRL_U: c_int = 21;
    /// Ctrl-V
    pub const CTRL_V: c_int = 22;
    /// Ctrl-W
    pub const CTRL_W: c_int = 23;
    /// Ctrl-X
    pub const CTRL_X: c_int = 24;
    /// Ctrl-Y
    pub const CTRL_Y: c_int = 25;
    /// Ctrl-Z
    pub const CTRL_Z: c_int = 26;

    // Special key base
    const K_SPECIAL: c_int = 0x100;
    const KS_EXTRA: c_int = K_SPECIAL + 0x5E;

    /// Up arrow
    pub const K_UP: c_int = K_SPECIAL + 0x48;
    /// Down arrow
    pub const K_DOWN: c_int = K_SPECIAL + 0x49;
    /// Left arrow
    pub const K_LEFT: c_int = K_SPECIAL + 0x4A;
    /// Right arrow
    pub const K_RIGHT: c_int = K_SPECIAL + 0x4B;
    /// Shift-Up
    pub const K_S_UP: c_int = KS_EXTRA + 11;
    /// Shift-Down
    pub const K_S_DOWN: c_int = KS_EXTRA + 12;
    /// Shift-Left
    pub const K_S_LEFT: c_int = KS_EXTRA + 7;
    /// Shift-Right
    pub const K_S_RIGHT: c_int = KS_EXTRA + 8;
    /// Home key
    pub const K_HOME: c_int = KS_EXTRA + 1;
    /// End key
    pub const K_END: c_int = KS_EXTRA + 3;
    /// Delete key
    pub const K_DEL: c_int = KS_EXTRA + 9;
    /// Insert key
    pub const K_INS: c_int = KS_EXTRA + 10;
    /// Page Up
    pub const K_PAGEUP: c_int = KS_EXTRA + 7;
    /// Page Down
    pub const K_PAGEDOWN: c_int = KS_EXTRA + 8;
}

// =============================================================================
// Maximum Values
// =============================================================================

/// Maximum command line recursion depth
pub const MAX_CMDLINE_LEVEL: c_int = 50;

/// Maximum length for a command line (1 MB)
pub const MAXCOL: c_int = 0x7FFF_FFFF;

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if a character is a command-line prompt character.
#[no_mangle]
pub extern "C" fn rs_is_cmdline_firstc(c: c_int) -> c_int {
    c_int::from(matches!(
        c,
        firstc::EX_CMD
            | firstc::FORWARD_SEARCH
            | firstc::BACKWARD_SEARCH
            | firstc::EXPRESSION
            | firstc::DEBUG
            | firstc::INPUT_FN
    ))
}

// Note: rs_is_search_firstc is defined in search.rs

/// Check if the firstc indicates an Ex command.
#[no_mangle]
pub extern "C" fn rs_is_ex_cmd_firstc(c: c_int) -> c_int {
    c_int::from(c == firstc::EX_CMD)
}

/// Get the command line result for "not changed".
#[no_mangle]
pub extern "C" fn rs_cmdline_result_not_changed() -> c_int {
    cmdline_result::NOT_CHANGED
}

/// Get the command line result for "changed".
#[no_mangle]
pub extern "C" fn rs_cmdline_result_changed() -> c_int {
    cmdline_result::CHANGED
}

/// Get the command line result for "goto normal mode".
#[no_mangle]
pub extern "C" fn rs_cmdline_result_goto_normal() -> c_int {
    cmdline_result::GOTO_NORMAL_MODE
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firstc_values() {
        assert_eq!(firstc::EX_CMD, i32::from(b':'));
        assert_eq!(firstc::FORWARD_SEARCH, i32::from(b'/'));
        assert_eq!(firstc::BACKWARD_SEARCH, i32::from(b'?'));
        assert_eq!(firstc::EXPRESSION, i32::from(b'='));
        assert_eq!(firstc::DEBUG, i32::from(b'>'));
        assert_eq!(firstc::INPUT_FN, i32::from(b'@'));
        assert_eq!(firstc::NONE, 0);
    }

    #[test]
    fn test_cmdline_result_values() {
        assert_eq!(cmdline_result::NOT_CHANGED, 1);
        assert_eq!(cmdline_result::CHANGED, 2);
        assert_eq!(cmdline_result::GOTO_NORMAL_MODE, 3);
        assert_eq!(cmdline_result::PROCESS_NEXT_KEY, 4);
    }

    #[test]
    fn test_redraw_state_values() {
        assert_eq!(redraw_state::NONE, 0);
        assert_eq!(redraw_state::POS, 1);
        assert_eq!(redraw_state::ALL, 2);
    }

    #[test]
    fn test_control_keys() {
        assert_eq!(keys::CTRL_A, 1);
        assert_eq!(keys::CTRL_Z, 26);
        assert_eq!(keys::ESC, 27);
    }

    #[test]
    fn test_is_cmdline_firstc() {
        assert_eq!(rs_is_cmdline_firstc(firstc::EX_CMD), 1);
        assert_eq!(rs_is_cmdline_firstc(firstc::FORWARD_SEARCH), 1);
        assert_eq!(rs_is_cmdline_firstc(firstc::NONE), 0);
        assert_eq!(rs_is_cmdline_firstc(c_int::from(b'a')), 0);
    }

    // Note: rs_is_search_firstc is tested in search.rs

    #[test]
    fn test_is_ex_cmd_firstc() {
        assert_eq!(rs_is_ex_cmd_firstc(firstc::EX_CMD), 1);
        assert_eq!(rs_is_ex_cmd_firstc(firstc::FORWARD_SEARCH), 0);
    }
}
