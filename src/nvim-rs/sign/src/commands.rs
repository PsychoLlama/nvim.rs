//! Sign Ex command handlers
//!
//! This module handles the :sign command and its subcommands.

use std::ffi::{c_char, c_int, c_void};

use crate::{LinenrT, SignBufHandle, SignCmd, SIGN_DEF_PRIO};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

extern "C" {
    // Composite accessors for ex command handling
    fn nvim_sign_define_cmd_impl(name: *mut c_char, cmdline: *mut c_char);
    fn nvim_sign_place_cmd_impl(
        buf: SignBufHandle,
        lnum: LinenrT,
        name: *mut c_char,
        id: c_int,
        group: *mut c_char,
        prio: c_int,
    );
    fn nvim_sign_unplace_cmd_impl(
        buf: SignBufHandle,
        lnum: LinenrT,
        name: *const c_char,
        id: c_int,
        group: *mut c_char,
    );
    fn nvim_sign_jump_cmd_impl(
        buf: SignBufHandle,
        lnum: LinenrT,
        name: *const c_char,
        id: c_int,
        group: *mut c_char,
    );
    fn nvim_parse_sign_cmd_args_impl(
        cmd: c_int,
        arg: *mut c_char,
        name: *mut *mut c_char,
        id: *mut c_int,
        group: *mut *mut c_char,
        prio: *mut c_int,
        buf: *mut SignBufHandle,
        lnum: *mut LinenrT,
    ) -> c_int;
    fn nvim_ex_sign_impl(eap: *mut c_void);

    // Command completion composite accessors
    fn nvim_get_sign_name_impl(xp: *mut c_void, idx: c_int) -> *mut c_char;
    fn nvim_set_context_in_sign_cmd_impl(xp: *mut c_void, arg: *mut c_char);
}

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
// Sign Define Command Arguments
// =============================================================================

/// Arguments for :sign define command
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignDefineArgs {
    /// Sign name (required)
    pub name: *const c_char,
    /// Icon path (optional)
    pub icon: *const c_char,
    /// Sign text (optional)
    pub text: *const c_char,
    /// Line highlight group (optional)
    pub linehl: *const c_char,
    /// Text highlight group (optional)
    pub texthl: *const c_char,
    /// Cursorline highlight group (optional)
    pub culhl: *const c_char,
    /// Number column highlight group (optional)
    pub numhl: *const c_char,
    /// Priority (-1 for default)
    pub priority: c_int,
}

impl Default for SignDefineArgs {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            icon: std::ptr::null(),
            text: std::ptr::null(),
            linehl: std::ptr::null(),
            texthl: std::ptr::null(),
            culhl: std::ptr::null(),
            numhl: std::ptr::null(),
            priority: -1,
        }
    }
}

/// Create default sign define arguments.
#[no_mangle]
pub extern "C" fn rs_sign_define_args_default() -> SignDefineArgs {
    SignDefineArgs::default()
}

/// Check if sign define arguments have a valid name.
///
/// # Safety
/// `args.name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_define_args_has_name(args: &SignDefineArgs) -> bool {
    if args.name.is_null() {
        return false;
    }
    *args.name.cast::<u8>() != 0
}

/// Check if sign define arguments specify any visual attributes.
#[no_mangle]
pub extern "C" fn rs_sign_define_args_has_attrs(args: &SignDefineArgs) -> bool {
    !args.icon.is_null()
        || !args.text.is_null()
        || !args.linehl.is_null()
        || !args.texthl.is_null()
        || !args.culhl.is_null()
        || !args.numhl.is_null()
        || args.priority != -1
}

// =============================================================================
// Command Execution Results
// =============================================================================

/// Result of a sign command execution
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignCmdResult {
    /// Success
    Ok = 0,
    /// Invalid command index
    InvalidCmd = 1,
    /// Missing required argument
    MissingArg = 2,
    /// Invalid argument value
    InvalidArg = 3,
    /// Sign not found
    SignNotFound = 4,
    /// Buffer not found
    BufferNotFound = 5,
    /// Operation failed
    Failed = 6,
}

/// Convert sign command result to return code.
///
/// Returns 0 for success, -1 for failure (matches Vim convention).
#[no_mangle]
pub extern "C" fn rs_sign_cmd_result_to_rc(result: SignCmdResult) -> c_int {
    if result == SignCmdResult::Ok {
        0
    } else {
        -1
    }
}

// =============================================================================
// Argument Validation Helpers
// =============================================================================

/// Validate ID argument for sign commands.
///
/// Returns true if valid:
/// - For place: id > 0 (required) or id == 0 (auto-assign)
/// - For unplace: any id value
/// - For jump: id > 0 (required)
#[no_mangle]
pub extern "C" fn rs_sign_validate_id(id: c_int, cmd: c_int) -> bool {
    match SignCmd::from_int(cmd) {
        Some(SignCmd::Place) => id >= 0,
        Some(SignCmd::Jump) => id > 0,
        // Unplace and others accept any ID
        _ => true,
    }
}

/// Validate line number argument for sign commands.
///
/// Returns true if valid:
/// - For place: lnum > 0 for new placement, or lnum == 0 for modify
/// - For unplace: lnum >= -1 (all lines or specific)
/// - For jump: lnum not used
#[no_mangle]
pub extern "C" fn rs_sign_validate_lnum(lnum: LinenrT, cmd: c_int) -> bool {
    match SignCmd::from_int(cmd) {
        Some(SignCmd::Place) => lnum >= 0,
        Some(SignCmd::Unplace) => lnum >= -1,
        _ => true,
    }
}

// =============================================================================
// Command Output Formatting
// =============================================================================

/// Format mode for sign listing.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignListFormat {
    /// List all defined signs
    AllDefined = 0,
    /// List a specific defined sign
    SpecificDefined = 1,
    /// List all placed signs
    AllPlaced = 2,
    /// List placed signs in a buffer
    PlacedInBuffer = 3,
    /// List placed signs in a group
    PlacedInGroup = 4,
}

/// Determine the list format based on command arguments.
#[no_mangle]
pub extern "C" fn rs_sign_list_format(
    cmd: c_int,
    has_name: c_int,
    has_buf: c_int,
    has_group: c_int,
) -> SignListFormat {
    if cmd == SignCmd::List as c_int {
        if has_name != 0 {
            SignListFormat::SpecificDefined
        } else {
            SignListFormat::AllDefined
        }
    } else if has_buf != 0 {
        SignListFormat::PlacedInBuffer
    } else if has_group != 0 {
        SignListFormat::PlacedInGroup
    } else {
        SignListFormat::AllPlaced
    }
}

// =============================================================================
// Ex Command FFI Wrappers
// =============================================================================

/// ":sign define {name} ..." command.
///
/// Parses key=value pairs from the command line and defines the sign.
///
/// # Safety
///
/// `name` and `cmdline` must be valid, writable C strings.
#[unsafe(export_name = "sign_define_cmd")]
pub unsafe extern "C" fn rs_sign_define_cmd(name: *mut c_char, cmdline: *mut c_char) {
    nvim_sign_define_cmd_impl(name, cmdline);
}

/// ":sign place" command.
///
/// # Safety
///
/// All pointer arguments must be valid or null.
#[unsafe(export_name = "sign_place_cmd")]
pub unsafe extern "C" fn rs_sign_place_cmd(
    buf: SignBufHandle,
    lnum: LinenrT,
    name: *mut c_char,
    id: c_int,
    group: *mut c_char,
    prio: c_int,
) {
    nvim_sign_place_cmd_impl(buf, lnum, name, id, group, prio);
}

/// ":sign unplace" command.
///
/// # Safety
///
/// All pointer arguments must be valid or null.
#[unsafe(export_name = "sign_unplace_cmd")]
pub unsafe extern "C" fn rs_sign_unplace_cmd(
    buf: SignBufHandle,
    lnum: LinenrT,
    name: *const c_char,
    id: c_int,
    group: *mut c_char,
) {
    nvim_sign_unplace_cmd_impl(buf, lnum, name, id, group);
}

/// ":sign jump" command.
///
/// # Safety
///
/// All pointer arguments must be valid or null.
#[unsafe(export_name = "sign_jump_cmd")]
pub unsafe extern "C" fn rs_sign_jump_cmd(
    buf: SignBufHandle,
    lnum: LinenrT,
    name: *const c_char,
    id: c_int,
    group: *mut c_char,
) {
    nvim_sign_jump_cmd_impl(buf, lnum, name, id, group);
}

/// Parse command line arguments for ":sign place/unplace/jump".
///
/// # Safety
///
/// All pointer arguments must be valid.
#[unsafe(export_name = "parse_sign_cmd_args")]
pub unsafe extern "C" fn rs_parse_sign_cmd_args(
    cmd: c_int,
    arg: *mut c_char,
    name: *mut *mut c_char,
    id: *mut c_int,
    group: *mut *mut c_char,
    prio: *mut c_int,
    buf: *mut SignBufHandle,
    lnum: *mut LinenrT,
) -> c_int {
    nvim_parse_sign_cmd_args_impl(cmd, arg, name, id, group, prio, buf, lnum)
}

/// ":sign" command — top-level dispatcher.
///
/// # Safety
///
/// `eap` must be a valid exarg_T pointer.
#[unsafe(export_name = "ex_sign")]
pub unsafe extern "C" fn rs_ex_sign(eap: *mut c_void) {
    if eap.is_null() {
        return;
    }
    nvim_ex_sign_impl(eap);
}

// =============================================================================
// Command Completion FFI Wrappers
// =============================================================================

/// Get sign command expansion string for command line completion.
///
/// # Safety
///
/// `xp` must be a valid expand_T pointer.
#[unsafe(export_name = "get_sign_name")]
pub unsafe extern "C" fn rs_get_sign_name(xp: *mut c_void, idx: c_int) -> *mut c_char {
    nvim_get_sign_name_impl(xp, idx)
}

/// Set command line completion context for :sign command.
///
/// # Safety
///
/// `xp` must be a valid expand_T pointer.
/// `arg` must be a valid, writable C string.
#[unsafe(export_name = "set_context_in_sign_cmd")]
pub unsafe extern "C" fn rs_set_context_in_sign_cmd(xp: *mut c_void, arg: *mut c_char) {
    if xp.is_null() || arg.is_null() {
        return;
    }
    nvim_set_context_in_sign_cmd_impl(xp, arg);
}

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

    #[test]
    fn test_sign_define_args_default() {
        let args = rs_sign_define_args_default();
        assert!(args.name.is_null());
        assert!(args.icon.is_null());
        assert!(args.text.is_null());
        assert!(args.linehl.is_null());
        assert!(args.texthl.is_null());
        assert!(args.culhl.is_null());
        assert!(args.numhl.is_null());
        assert_eq!(args.priority, -1);
    }

    #[test]
    fn test_sign_define_args_has_attrs() {
        let default = rs_sign_define_args_default();
        assert!(!rs_sign_define_args_has_attrs(&default));

        let with_prio = SignDefineArgs {
            priority: 10,
            ..Default::default()
        };
        assert!(rs_sign_define_args_has_attrs(&with_prio));
    }

    #[test]
    fn test_sign_cmd_result_to_rc() {
        assert_eq!(rs_sign_cmd_result_to_rc(SignCmdResult::Ok), 0);
        assert_eq!(rs_sign_cmd_result_to_rc(SignCmdResult::InvalidCmd), -1);
        assert_eq!(rs_sign_cmd_result_to_rc(SignCmdResult::MissingArg), -1);
        assert_eq!(rs_sign_cmd_result_to_rc(SignCmdResult::Failed), -1);
    }

    #[test]
    fn test_sign_validate_id() {
        // Place: id >= 0
        assert!(rs_sign_validate_id(0, SignCmd::Place as c_int));
        assert!(rs_sign_validate_id(1, SignCmd::Place as c_int));
        assert!(!rs_sign_validate_id(-1, SignCmd::Place as c_int));

        // Unplace: any id
        assert!(rs_sign_validate_id(-1, SignCmd::Unplace as c_int));
        assert!(rs_sign_validate_id(0, SignCmd::Unplace as c_int));
        assert!(rs_sign_validate_id(1, SignCmd::Unplace as c_int));

        // Jump: id > 0
        assert!(!rs_sign_validate_id(-1, SignCmd::Jump as c_int));
        assert!(!rs_sign_validate_id(0, SignCmd::Jump as c_int));
        assert!(rs_sign_validate_id(1, SignCmd::Jump as c_int));
    }

    #[test]
    fn test_sign_validate_lnum() {
        // Place: lnum >= 0
        assert!(rs_sign_validate_lnum(0, SignCmd::Place as c_int));
        assert!(rs_sign_validate_lnum(1, SignCmd::Place as c_int));
        assert!(!rs_sign_validate_lnum(-1, SignCmd::Place as c_int));

        // Unplace: lnum >= -1
        assert!(rs_sign_validate_lnum(-1, SignCmd::Unplace as c_int));
        assert!(rs_sign_validate_lnum(0, SignCmd::Unplace as c_int));
        assert!(rs_sign_validate_lnum(1, SignCmd::Unplace as c_int));
    }

    #[test]
    fn test_sign_list_format() {
        // List command with name
        assert_eq!(
            rs_sign_list_format(SignCmd::List as c_int, 1, 0, 0),
            SignListFormat::SpecificDefined
        );
        // List command without name
        assert_eq!(
            rs_sign_list_format(SignCmd::List as c_int, 0, 0, 0),
            SignListFormat::AllDefined
        );
        // Place command with buffer
        assert_eq!(
            rs_sign_list_format(SignCmd::Place as c_int, 0, 1, 0),
            SignListFormat::PlacedInBuffer
        );
        // Place command with group
        assert_eq!(
            rs_sign_list_format(SignCmd::Place as c_int, 0, 0, 1),
            SignListFormat::PlacedInGroup
        );
        // Place command no filter
        assert_eq!(
            rs_sign_list_format(SignCmd::Place as c_int, 0, 0, 0),
            SignListFormat::AllPlaced
        );
    }
}
