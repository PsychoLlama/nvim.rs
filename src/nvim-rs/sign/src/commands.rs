//! Sign Ex command handlers
//!
//! This module handles the :sign command and its subcommands.

use std::ffi::{c_char, c_int};

use crate::{LinenrT, SignBufHandle, SignCmd, SIGN_DEF_PRIO};

// =============================================================================
// Command Argument Parsing
// =============================================================================

/// Parsed arguments for :sign place/unplace/jump commands
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignCmdArgs {
    /// Sign ID (-1 if not specified)
    pub id: c_int,
    /// Line number (-1 if not specified)
    pub lnum: LinenrT,
    /// Priority (-1 for default)
    pub priority: c_int,
    /// Name pointer (may be null)
    pub name: *const c_char,
    /// Group pointer (may be null)
    pub group: *const c_char,
    /// Buffer handle (may be null)
    pub buf: SignBufHandle,
}

impl Default for SignCmdArgs {
    fn default() -> Self {
        Self {
            id: -1,
            lnum: -1,
            priority: -1,
            name: std::ptr::null(),
            group: std::ptr::null(),
            buf: SignBufHandle::null(),
        }
    }
}

/// Create default sign command arguments.
#[no_mangle]
pub extern "C" fn rs_sign_cmd_args_default() -> SignCmdArgs {
    SignCmdArgs::default()
}

/// Check if sign command arguments are valid for the given command.
///
/// # Safety
///
/// `args.group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_cmd_args_valid(cmd: c_int, args: &SignCmdArgs) -> bool {
    match SignCmd::from_int(cmd) {
        Some(SignCmd::Place) => {
            if args.id <= 0 {
                // List mode - check invalid combinations
                !(args.lnum >= 0 || !args.name.is_null() || rs_sign_group_is_empty(args.group))
            } else {
                // Place mode - need name and buffer
                !args.name.is_null() && !args.buf.is_null() && !rs_sign_group_is_empty(args.group)
            }
        }
        Some(SignCmd::Unplace) => {
            // Invalid: lnum specified, name specified, or empty group
            !(args.lnum >= 0 || !args.name.is_null() || rs_sign_group_is_empty(args.group))
        }
        Some(SignCmd::Jump) => {
            // Need buffer, valid ID, no invalid params
            !args.buf.is_null()
                && !rs_sign_group_is_empty(args.group)
                && args.lnum < 0
                && args.name.is_null()
        }
        _ => true,
    }
}

// =============================================================================
// Group String Utilities
// =============================================================================

/// Check if a group string is empty (but not null).
///
/// Returns true if group points to an empty string.
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_group_is_empty(group: *const c_char) -> bool {
    if group.is_null() {
        return false;
    }
    *group.cast::<u8>() == 0
}

/// Check if a group represents "all" groups ("*").
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_group_is_all(group: *const c_char) -> bool {
    if group.is_null() {
        return false;
    }
    *group.cast::<u8>() == b'*'
}

/// Normalize a group pointer.
///
/// Returns null if the group is empty, otherwise returns the original pointer.
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_normalize_group(group: *const c_char) -> *const c_char {
    if group.is_null() {
        return std::ptr::null();
    }
    if *group.cast::<u8>() == 0 {
        std::ptr::null()
    } else {
        group
    }
}

// =============================================================================
// Command Dispatch Helpers
// =============================================================================

/// Check if a sign command operates on definitions (define/undefine/list).
#[no_mangle]
pub extern "C" fn rs_sign_cmd_is_definition(cmd: c_int) -> bool {
    matches!(
        SignCmd::from_int(cmd),
        Some(SignCmd::Define | SignCmd::Undefine | SignCmd::List)
    )
}

/// Check if a sign command operates on placed signs (place/unplace/jump).
#[no_mangle]
pub extern "C" fn rs_sign_cmd_is_placement(cmd: c_int) -> bool {
    matches!(
        SignCmd::from_int(cmd),
        Some(SignCmd::Place | SignCmd::Unplace | SignCmd::Jump)
    )
}

/// Get the command index for a sign command.
///
/// Returns SIGNCMD_LAST (6) if invalid.
#[no_mangle]
pub extern "C" fn rs_sign_get_cmd_idx(cmd: SignCmd) -> c_int {
    cmd as c_int
}

// =============================================================================
// Priority Handling
// =============================================================================

/// Get effective priority from command args and sign definition.
///
/// Priority precedence:
/// 1. Explicit priority in command (-1 means not specified)
/// 2. Sign definition priority
/// 3. SIGN_DEF_PRIO default
#[no_mangle]
pub extern "C" fn rs_sign_cmd_get_priority(cmd_prio: c_int, def_prio: c_int) -> c_int {
    if cmd_prio != -1 {
        cmd_prio
    } else if def_prio != -1 {
        def_prio
    } else {
        SIGN_DEF_PRIO
    }
}

// =============================================================================
// Place Command Modes
// =============================================================================

/// Sign place command mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignPlaceMode {
    /// List placed signs
    List = 0,
    /// Place a new sign
    Place = 1,
    /// Modify an existing sign
    Modify = 2,
}

/// Determine the mode for :sign place command.
#[no_mangle]
pub extern "C" fn rs_sign_place_mode(id: c_int, lnum: LinenrT) -> SignPlaceMode {
    if id <= 0 {
        SignPlaceMode::List
    } else if lnum > 0 {
        SignPlaceMode::Place
    } else {
        SignPlaceMode::Modify
    }
}

// =============================================================================
// Unplace Command Modes
// =============================================================================

/// Sign unplace command mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignUnplaceMode {
    /// Unplace at cursor line
    AtCursor = 0,
    /// Unplace specific ID
    ById = 1,
    /// Unplace all matching
    All = 2,
}

/// Determine the mode for :sign unplace command.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_mode(id: c_int) -> SignUnplaceMode {
    match id {
        -1 => SignUnplaceMode::AtCursor,
        0 => SignUnplaceMode::All,
        _ => SignUnplaceMode::ById,
    }
}

// =============================================================================
// Command String Parsing
// =============================================================================

/// Sign argument type indices
pub const SIGN_ARG_LINE: c_int = 1;
pub const SIGN_ARG_NAME: c_int = 2;
pub const SIGN_ARG_GROUP: c_int = 3;
pub const SIGN_ARG_PRIORITY: c_int = 4;
pub const SIGN_ARG_FILE: c_int = 5;
pub const SIGN_ARG_BUFFER: c_int = 6;

/// Sign define argument type indices
pub const SIGN_DEF_ARG_ICON: c_int = 1;
pub const SIGN_DEF_ARG_TEXT: c_int = 2;
pub const SIGN_DEF_ARG_LINEHL: c_int = 3;
pub const SIGN_DEF_ARG_TEXTHL: c_int = 4;
pub const SIGN_DEF_ARG_CULHL: c_int = 5;
pub const SIGN_DEF_ARG_NUMHL: c_int = 6;
pub const SIGN_DEF_ARG_PRIORITY: c_int = 7;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_cmd_args_default() {
        let args = rs_sign_cmd_args_default();
        assert_eq!(args.id, -1);
        assert_eq!(args.lnum, -1);
        assert_eq!(args.priority, -1);
        assert!(args.name.is_null());
        assert!(args.group.is_null());
        assert!(args.buf.is_null());
    }

    #[test]
    fn test_sign_cmd_is_definition() {
        assert!(rs_sign_cmd_is_definition(0)); // Define
        assert!(rs_sign_cmd_is_definition(1)); // Undefine
        assert!(rs_sign_cmd_is_definition(2)); // List
        assert!(!rs_sign_cmd_is_definition(3)); // Place
        assert!(!rs_sign_cmd_is_definition(4)); // Unplace
        assert!(!rs_sign_cmd_is_definition(5)); // Jump
    }

    #[test]
    fn test_sign_cmd_is_placement() {
        assert!(!rs_sign_cmd_is_placement(0)); // Define
        assert!(!rs_sign_cmd_is_placement(1)); // Undefine
        assert!(!rs_sign_cmd_is_placement(2)); // List
        assert!(rs_sign_cmd_is_placement(3)); // Place
        assert!(rs_sign_cmd_is_placement(4)); // Unplace
        assert!(rs_sign_cmd_is_placement(5)); // Jump
    }

    #[test]
    fn test_sign_cmd_get_priority() {
        assert_eq!(rs_sign_cmd_get_priority(5, 10), 5);
        assert_eq!(rs_sign_cmd_get_priority(-1, 10), 10);
        assert_eq!(rs_sign_cmd_get_priority(-1, -1), SIGN_DEF_PRIO);
    }

    #[test]
    fn test_sign_place_mode() {
        assert_eq!(rs_sign_place_mode(-1, 0), SignPlaceMode::List);
        assert_eq!(rs_sign_place_mode(0, 0), SignPlaceMode::List);
        assert_eq!(rs_sign_place_mode(1, 10), SignPlaceMode::Place);
        assert_eq!(rs_sign_place_mode(1, 0), SignPlaceMode::Modify);
    }

    #[test]
    fn test_sign_unplace_mode() {
        assert_eq!(rs_sign_unplace_mode(-1), SignUnplaceMode::AtCursor);
        assert_eq!(rs_sign_unplace_mode(0), SignUnplaceMode::All);
        assert_eq!(rs_sign_unplace_mode(1), SignUnplaceMode::ById);
        assert_eq!(rs_sign_unplace_mode(100), SignUnplaceMode::ById);
    }
}
