//! Sign Ex command handlers
//!
//! This module handles the :sign command and its subcommands.

use std::ffi::{c_char, c_int, c_void};

use nvim_ex_cmds_types::ExArg;

use crate::{LinenrT, SignBufHandle, SignCmd, SIGN_DEF_PRIO};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

// Expand context constants from cmdexpand_defs.h
mod expand_ctx {
    use std::ffi::c_int;
    pub const EXPAND_NOTHING: c_int = 0;
    pub const EXPAND_FILES: c_int = 2;
    pub const EXPAND_BUFFERS: c_int = 9;
    pub const EXPAND_HIGHLIGHT: c_int = 13;
    pub const EXPAND_SIGN: c_int = 34;
}

// expand_what values (mirrors the C static enum)
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExpandWhat {
    Subcmd,
    Define,
    Place,
    List,
    Unplace,
    SignNames,
    SignGroups,
}

// SAFETY: single-threaded Vim event loop; no concurrent mutation.
static mut EXPAND_WHAT: ExpandWhat = ExpandWhat::Subcmd;

extern "C" {
    // String utilities (Rust exports, linked as C symbols)
    fn skipwhite(p: *const c_char) -> *mut c_char;
    #[link_name = "skiptowhite_esc"]
    fn skiptowhite_esc_fn(p: *const c_char) -> *mut c_char;
    fn semsg(fmt: *const c_char, ...);
    fn emsg(msg: *const c_char) -> c_int;

    // Buffer list lookups
    fn nvim_buflist_findname_exp(ptr: *const c_char) -> SignBufHandle;
    fn rs_buflist_findnr(nr: c_int) -> SignBufHandle;

    // Current window accessors
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_curwin_get_buffer() -> SignBufHandle;

    // Sign operations (Rust exports, used here)
    fn rs_sign_list_placed(buf: SignBufHandle, group: *const c_char);
    fn rs_sign_place(
        id: *mut u32,
        group: *const c_char,
        name: *const c_char,
        buf: SignBufHandle,
        lnum: LinenrT,
        prio: c_int,
    ) -> c_int;
    fn rs_sign_unplace(
        buf: SignBufHandle,
        id: c_int,
        group: *const c_char,
        atlnum: LinenrT,
    ) -> c_int;
    fn rs_sign_jump(id: c_int, group: *const c_char, buf: SignBufHandle) -> LinenrT;
    fn rs_sign_define_by_name(
        name: *const c_char,
        icon: *const c_char,
        text: *const c_char,
        linehl: *const c_char,
        texthl: *const c_char,
        culhl: *const c_char,
        numhl: *const c_char,
        prio: c_int,
    ) -> c_int;

    // getdigits_int: advances pointer through digits, returns parsed int
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;

    // expand_T accessors
    fn nvim_xp_set_context(xp: *mut c_void, ctx: c_int);
    fn nvim_xp_set_pattern(xp: *mut c_void, pat: *mut c_char);

    // (nvim_eap_get_arg_local inlined via ExArg field access)

    // sign_map/sign_ns iteration
    fn nvim_sign_map_get_nth_key(idx: c_int) -> *mut c_char;
    fn nvim_sign_ns_get_name(idx: c_int) -> *mut c_char;

    // skiptowhite (not escaped) for subcommand parsing
    fn skiptowhite(p: *const c_char) -> *mut c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
}

// Error message constants
static E_INVARG: &[u8] = b"E474: Invalid argument\0";
static E_INVARG2: &[u8] = b"E475: Invalid value for argument %s\0";
static E_ARGREQ: &[u8] = b"E471: Argument required\0";
static E_INVALID_BUFFER_NAME: &[u8] = b"E158: Invalid buffer name: %s\0";
static E_TRAILING: &[u8] = b"E488: Trailing characters: %s\0";
static E_MISSING_SIGN: &[u8] = b"E159: Missing sign number\0";

const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// Phase 4: Migrated composite accessor implementations
// =============================================================================

/// ":sign define {name} ..." implementation — migrated from C.
///
/// # Safety
/// `name` and `cmdline` must be valid, writable C strings.
unsafe fn sign_define_cmd_impl(name: *mut c_char, cmdline: *mut c_char) {
    let mut icon: *const c_char = std::ptr::null();
    let mut text: *const c_char = std::ptr::null();
    let mut linehl: *const c_char = std::ptr::null();
    let mut texthl: *const c_char = std::ptr::null();
    let mut culhl: *const c_char = std::ptr::null();
    let mut numhl: *const c_char = std::ptr::null();
    let mut prio: c_int = -1;

    let mut cmdline = cmdline;
    loop {
        let arg = skipwhite(cmdline);
        if *arg == 0 {
            break;
        }
        cmdline = skiptowhite_esc_fn(arg);

        // NUL-terminate the current token if not at end
        let at_end = *cmdline == 0;
        if !at_end {
            *cmdline = 0;
            cmdline = cmdline.add(1);
        }

        // Match prefix and extract value
        let arg_bytes = arg.cast::<u8>();
        if starts_with_bytes(arg_bytes, b"icon=") {
            icon = arg.add(5);
        } else if starts_with_bytes(arg_bytes, b"text=") {
            text = arg.add(5);
        } else if starts_with_bytes(arg_bytes, b"linehl=") {
            linehl = arg.add(7);
        } else if starts_with_bytes(arg_bytes, b"texthl=") {
            texthl = arg.add(7);
        } else if starts_with_bytes(arg_bytes, b"culhl=") {
            culhl = arg.add(6);
        } else if starts_with_bytes(arg_bytes, b"numhl=") {
            numhl = arg.add(6);
        } else if starts_with_bytes(arg_bytes, b"priority=") {
            let val_ptr = arg.add(9);
            prio = atoi_ptr(val_ptr);
        } else {
            semsg(E_INVARG2.as_ptr().cast(), arg);
            return;
        }

        if at_end {
            break;
        }
    }

    rs_sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio);
}

/// ":sign place" implementation — migrated from C.
///
/// # Safety
/// All pointer arguments must be valid or null.
unsafe fn sign_place_cmd_impl(
    buf: SignBufHandle,
    lnum: LinenrT,
    name: *mut c_char,
    id: c_int,
    group: *mut c_char,
    prio: c_int,
) {
    if id <= 0 {
        if lnum >= 0 || !name.is_null() || (!group.is_null() && *group == 0) {
            emsg(E_INVARG.as_ptr().cast());
        } else {
            rs_sign_list_placed(buf, group);
        }
    } else {
        if name.is_null() || buf.is_null() || (!group.is_null() && *group == 0) {
            emsg(E_INVARG.as_ptr().cast());
            return;
        }
        #[allow(clippy::cast_sign_loss)]
        let mut uid = id as u32;
        rs_sign_place(std::ptr::addr_of_mut!(uid), group, name, buf, lnum, prio);
    }
}

/// ":sign unplace" implementation — migrated from C.
///
/// # Safety
/// All pointer arguments must be valid or null.
unsafe fn sign_unplace_cmd_impl(
    mut buf: SignBufHandle,
    lnum: LinenrT,
    name: *const c_char,
    mut id: c_int,
    group: *mut c_char,
) {
    if lnum >= 0 || !name.is_null() || (!group.is_null() && *group == 0) {
        emsg(E_INVARG.as_ptr().cast());
        return;
    }

    let mut atlnum: LinenrT = 0;
    if id == -1 {
        atlnum = nvim_get_curwin_cursor_lnum();
        buf = nvim_curwin_get_buffer();
        id = 0;
    }

    if rs_sign_unplace(buf, id.max(0), group, atlnum) == 0 && atlnum > 0 {
        emsg(E_MISSING_SIGN.as_ptr().cast());
    }
}

/// ":sign jump" implementation — migrated from C.
///
/// # Safety
/// All pointer arguments must be valid or null.
unsafe fn sign_jump_cmd_impl(
    buf: SignBufHandle,
    lnum: LinenrT,
    name: *const c_char,
    id: c_int,
    group: *mut c_char,
) {
    if name.is_null() && group.is_null() && id == -1 {
        emsg(E_ARGREQ.as_ptr().cast());
        return;
    }

    if buf.is_null() || (!group.is_null() && *group == 0) || lnum >= 0 || !name.is_null() {
        emsg(E_INVARG.as_ptr().cast());
        return;
    }

    rs_sign_jump(id, group, buf);
}

/// Parse sign command arguments — migrated from C.
///
/// # Safety
/// All pointer arguments must be valid.
#[allow(clippy::too_many_arguments)]
unsafe fn parse_sign_cmd_args_impl(
    cmd: c_int,
    arg: *mut c_char,
    name: *mut *mut c_char,
    id: *mut c_int,
    group: *mut *mut c_char,
    prio: *mut c_int,
    buf: *mut SignBufHandle,
    lnum: *mut LinenrT,
) -> c_int {
    let arg1 = arg;
    let mut arg = arg;
    let mut filename: *mut c_char = std::ptr::null_mut();
    let mut lnum_arg = false;

    // First arg could be placed sign id (digits)
    if ascii_isdigit(*arg.cast::<u8>()) {
        let parsed = getdigits_int(std::ptr::addr_of_mut!(arg), true, 0);
        if !ascii_iswhite(*arg.cast::<u8>()) && *arg != 0 {
            // Not a pure number token - reset
            *id = -1;
            arg = arg1;
        } else {
            *id = parsed;
            arg = skipwhite(arg);
        }
    }

    while *arg != 0 {
        let arg_bytes = arg.cast::<u8>();
        if starts_with_bytes(arg_bytes, b"line=") {
            arg = arg.add(5);
            *lnum = atoi_ptr(arg);
            arg = skiptowhite_fn(arg);
            lnum_arg = true;
        } else if starts_with_bytes(arg_bytes, b"*") && cmd == SignCmd::Unplace as c_int {
            if *id != -1 {
                emsg(E_INVARG.as_ptr().cast());
                return FAIL;
            }
            *id = -2;
            arg = skiptowhite_fn(arg.add(1));
        } else if starts_with_bytes(arg_bytes, b"name=") {
            arg = arg.add(5);
            let namep = arg;
            arg = skiptowhite_fn(arg);
            if *arg != 0 {
                *arg = 0;
                arg = arg.add(1);
            }
            // Strip leading zeros (but keep "0")
            let mut p = namep;
            while *p.cast::<u8>() == b'0' && *p.add(1) != 0 {
                p = p.add(1);
            }
            *name = p;
        } else if starts_with_bytes(arg_bytes, b"group=") {
            arg = arg.add(6);
            *group = arg;
            arg = skiptowhite_fn(arg);
            if *arg != 0 {
                *arg = 0;
                arg = arg.add(1);
            }
        } else if starts_with_bytes(arg_bytes, b"priority=") {
            arg = arg.add(9);
            *prio = atoi_ptr(arg);
            arg = skiptowhite_fn(arg);
        } else if starts_with_bytes(arg_bytes, b"file=") {
            arg = arg.add(5);
            filename = arg;
            *buf = nvim_buflist_findname_exp(arg);
            break;
        } else if starts_with_bytes(arg_bytes, b"buffer=") {
            arg = arg.add(7);
            filename = arg;
            *buf = rs_buflist_findnr(getdigits_int(std::ptr::addr_of_mut!(arg), true, 0));
            let after = skipwhite(arg);
            if *after != 0 {
                semsg(E_TRAILING.as_ptr().cast(), arg);
            }
            break;
        } else {
            emsg(E_INVARG.as_ptr().cast());
            return FAIL;
        }
        arg = skipwhite(arg);
    }

    if !filename.is_null() && (*buf).is_null() {
        semsg(E_INVALID_BUFFER_NAME.as_ptr().cast(), filename);
        return FAIL;
    }

    // If no filename for sign place (with lnum) or sign jump, use current buffer
    if filename.is_null()
        && ((cmd == SignCmd::Place as c_int && lnum_arg) || cmd == SignCmd::Jump as c_int)
    {
        *buf = nvim_curwin_get_buffer();
    }

    OK
}

// =============================================================================
// Private helpers
// =============================================================================

/// Check if a C byte string starts with a given prefix.
#[inline]
unsafe fn starts_with_bytes(s: *const u8, prefix: &[u8]) -> bool {
    for (i, &b) in prefix.iter().enumerate() {
        if *s.add(i) != b {
            return false;
        }
    }
    true
}

/// Simple atoi — parse decimal integer from null-terminated string.
#[inline]
unsafe fn atoi_ptr(s: *const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut p = s.cast::<u8>();
    while ascii_isdigit(*p) {
        result = result * 10 + c_int::from(*p - b'0');
        p = p.add(1);
    }
    result
}

/// Check if a byte is an ASCII digit.
#[inline]
fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Check if a byte is ASCII whitespace.
#[inline]
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Skip non-whitespace bytes (simple version, no escape handling).
#[inline]
unsafe fn skiptowhite_fn(p: *const c_char) -> *mut c_char {
    let mut p = p as *mut u8;
    while *p != 0 && !ascii_iswhite(*p) {
        p = p.add(1);
    }
    p.cast::<c_char>()
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
    sign_define_cmd_impl(name, cmdline);
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
    sign_place_cmd_impl(buf, lnum, name, id, group, prio);
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
    sign_unplace_cmd_impl(buf, lnum, name, id, group);
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
    sign_jump_cmd_impl(buf, lnum, name, id, group);
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
    parse_sign_cmd_args_impl(cmd, arg, name, id, group, prio, buf, lnum)
}

// SIGNCMD_LAST = SignCmd::COUNT (= 6)
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
const SIGNCMD_LAST: c_int = SignCmd::COUNT as c_int;

// Static C string constants for ex_sign messages
static E_UNKNOWN_SIGN_CMD: &[u8] = b"E160: Unknown sign command: %s\0";
static E_MISSING_SIGN_NAME: &[u8] = b"E156: Missing sign name\0";
static E_UNKNOWN_SIGN: &[u8] = b"E155: Unknown sign: %s\0";

/// ":sign" command — top-level dispatcher. Migrated from C nvim_ex_sign_impl.
///
/// # Safety
///
/// `eap` must be a valid exarg_T pointer.
#[unsafe(export_name = "ex_sign")]
pub unsafe extern "C" fn rs_ex_sign(eap: *mut c_void) {
    if eap.is_null() {
        return;
    }

    let arg = (*eap.cast::<ExArg>()).arg;
    if arg.is_null() {
        return;
    }

    // Parse subcommand: NUL-terminate at first whitespace
    let p = skiptowhite(arg);
    let save_p = *p;
    *p = 0;
    let idx = crate::rs_sign_cmd_idx(arg);
    *p = save_p;

    if idx == SIGNCMD_LAST {
        semsg(E_UNKNOWN_SIGN_CMD.as_ptr().cast(), arg);
        return;
    }

    let arg = skipwhite(p);

    if idx <= SignCmd::List as c_int {
        // Define, undefine, or list signs.
        if idx == SignCmd::List as c_int && *arg == 0 {
            // ":sign list" — list all defined signs
            // Iterate sign_map by index
            let mut i = 0;
            loop {
                let key = nvim_sign_map_get_nth_key(i);
                if key.is_null() {
                    break;
                }
                crate::query::rs_sign_list_by_name(key);
                i += 1;
            }
        } else if *arg == 0 {
            emsg(E_MISSING_SIGN_NAME.as_ptr().cast());
        } else {
            // Isolate the sign name. Skip leading zeros (but keep "0").
            let p = skiptowhite(arg);
            let at_end = *p == 0;
            if !at_end {
                *p = 0;
            }
            // Skip leading zeros
            let mut name = arg;
            while *name.cast::<u8>() == b'0' && *name.add(1) != 0 {
                name = name.add(1);
            }
            let rest = if at_end { p } else { p.add(1) };

            if idx == SignCmd::Define as c_int {
                sign_define_cmd_impl(name, rest);
            } else if idx == SignCmd::List as c_int {
                crate::query::rs_sign_list_by_name(name);
            } else {
                // undefine
                if crate::define::rs_sign_undefine_by_name(name) == FAIL {
                    semsg(E_UNKNOWN_SIGN.as_ptr().cast(), name);
                }
            }
        }
    } else {
        let mut id: c_int = -1;
        let mut lnum: LinenrT = -1;
        let mut name: *mut c_char = std::ptr::null_mut();
        let mut group: *mut c_char = std::ptr::null_mut();
        let mut prio: c_int = -1;
        let mut buf: SignBufHandle = SignBufHandle::null();

        if parse_sign_cmd_args_impl(
            idx,
            arg,
            std::ptr::addr_of_mut!(name),
            std::ptr::addr_of_mut!(id),
            std::ptr::addr_of_mut!(group),
            std::ptr::addr_of_mut!(prio),
            std::ptr::addr_of_mut!(buf),
            std::ptr::addr_of_mut!(lnum),
        ) == FAIL
        {
            return;
        }

        if idx == SignCmd::Place as c_int {
            sign_place_cmd_impl(buf, lnum, name, id, group, prio);
        } else if idx == SignCmd::Unplace as c_int {
            sign_unplace_cmd_impl(buf, lnum, name, id, group);
        } else if idx == SignCmd::Jump as c_int {
            sign_jump_cmd_impl(buf, lnum, name, id, group);
        }
    }
}

// =============================================================================
// Command Completion FFI Wrappers
// =============================================================================

// Static lists used by get_sign_name completion (NUL-terminated C strings)
static SIGN_SUBCMDS: &[&[u8]] = &[
    b"define\0",
    b"undefine\0",
    b"list\0",
    b"place\0",
    b"unplace\0",
    b"jump\0",
];
static SIGN_DEFINE_ARGS: &[&[u8]] = &[
    b"culhl=\0",
    b"icon=\0",
    b"linehl=\0",
    b"numhl=\0",
    b"text=\0",
    b"texthl=\0",
    b"priority=\0",
];
static SIGN_PLACE_ARGS: &[&[u8]] = &[
    b"line=\0",
    b"name=\0",
    b"group=\0",
    b"priority=\0",
    b"file=\0",
    b"buffer=\0",
];
static SIGN_LIST_ARGS: &[&[u8]] = &[b"group=\0", b"file=\0", b"buffer=\0"];
static SIGN_UNPLACE_ARGS: &[&[u8]] = &[b"group=\0", b"file=\0", b"buffer=\0"];

/// Get sign command expansion string for command line completion.
/// Migrated from C nvim_get_sign_name_impl.
///
/// # Safety
///
/// `xp` must be a valid expand_T pointer.
#[unsafe(export_name = "get_sign_name")]
pub unsafe extern "C" fn rs_get_sign_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    if idx < 0 {
        return std::ptr::null_mut();
    }
    #[allow(clippy::cast_sign_loss)]
    let idx = idx as usize;

    match EXPAND_WHAT {
        ExpandWhat::Subcmd => {
            if idx < SIGN_SUBCMDS.len() {
                static_str(SIGN_SUBCMDS[idx])
            } else {
                std::ptr::null_mut()
            }
        }
        ExpandWhat::Define => {
            if idx < SIGN_DEFINE_ARGS.len() {
                static_str(SIGN_DEFINE_ARGS[idx])
            } else {
                std::ptr::null_mut()
            }
        }
        ExpandWhat::Place => {
            if idx < SIGN_PLACE_ARGS.len() {
                static_str(SIGN_PLACE_ARGS[idx])
            } else {
                std::ptr::null_mut()
            }
        }
        ExpandWhat::List => {
            if idx < SIGN_LIST_ARGS.len() {
                static_str(SIGN_LIST_ARGS[idx])
            } else {
                std::ptr::null_mut()
            }
        }
        ExpandWhat::Unplace => {
            if idx < SIGN_UNPLACE_ARGS.len() {
                static_str(SIGN_UNPLACE_ARGS[idx])
            } else {
                std::ptr::null_mut()
            }
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        ExpandWhat::SignNames => nvim_sign_map_get_nth_key(idx as c_int),
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        ExpandWhat::SignGroups => nvim_sign_ns_get_name(idx as c_int),
    }
}

/// Set command line completion context for :sign command.
/// Migrated from C nvim_set_context_in_sign_cmd_impl.
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

    nvim_xp_set_context(xp, expand_ctx::EXPAND_SIGN);
    EXPAND_WHAT = ExpandWhat::Subcmd;
    nvim_xp_set_pattern(xp, arg);

    let end_subcmd = skiptowhite(arg);
    if *end_subcmd == 0 {
        return;
    }

    let save_end = *end_subcmd;
    *end_subcmd = 0;
    let cmd_idx = crate::rs_sign_cmd_idx(arg);
    *end_subcmd = save_end;

    let begin_subcmd_args = skipwhite(end_subcmd);

    // Loop to find the last whitespace-separated token (mirrors C do-while):
    // each iteration: skip whitespace → save as `last` → skip non-whitespace → check.
    let mut p = begin_subcmd_args;
    // Do the first iteration manually to satisfy Rust's definite initialization.
    p = skipwhite(p);
    let mut last = p;
    p = skiptowhite(p);
    while *p != 0 {
        p = skipwhite(p);
        last = p;
        p = skiptowhite(p);
    }

    // Check if last token contains '='
    if vim_strchr(last, c_int::from(b'=')).is_null() {
        // No '=' — completing a keyword
        nvim_xp_set_pattern(xp, last);
        match SignCmd::from_int(cmd_idx) {
            Some(SignCmd::Define) => {
                EXPAND_WHAT = ExpandWhat::Define;
            }
            Some(SignCmd::Place) => {
                if ascii_isdigit(*begin_subcmd_args.cast::<u8>()) {
                    EXPAND_WHAT = ExpandWhat::Place;
                } else {
                    EXPAND_WHAT = ExpandWhat::List;
                }
            }
            Some(SignCmd::List | SignCmd::Undefine) => {
                EXPAND_WHAT = ExpandWhat::SignNames;
            }
            Some(SignCmd::Jump | SignCmd::Unplace) => {
                EXPAND_WHAT = ExpandWhat::Unplace;
            }
            _ => {
                nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
            }
        }
    } else {
        // Has '=' — completing a value; xp_pattern is after the '='
        let eq_pos = vim_strchr(last, c_int::from(b'='));
        nvim_xp_set_pattern(xp, eq_pos.add(1));

        match SignCmd::from_int(cmd_idx) {
            Some(SignCmd::Define) => {
                if starts_with_cstr(last, b"texthl")
                    || starts_with_cstr(last, b"linehl")
                    || starts_with_cstr(last, b"culhl")
                    || starts_with_cstr(last, b"numhl")
                {
                    nvim_xp_set_context(xp, expand_ctx::EXPAND_HIGHLIGHT);
                } else if starts_with_cstr(last, b"icon") {
                    nvim_xp_set_context(xp, expand_ctx::EXPAND_FILES);
                } else {
                    nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
                }
            }
            Some(SignCmd::Place) => {
                if starts_with_cstr(last, b"name") {
                    EXPAND_WHAT = ExpandWhat::SignNames;
                } else if starts_with_cstr(last, b"group") {
                    EXPAND_WHAT = ExpandWhat::SignGroups;
                } else if starts_with_cstr(last, b"file") {
                    nvim_xp_set_context(xp, expand_ctx::EXPAND_BUFFERS);
                } else {
                    nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
                }
            }
            Some(SignCmd::Unplace | SignCmd::Jump) => {
                if starts_with_cstr(last, b"group") {
                    EXPAND_WHAT = ExpandWhat::SignGroups;
                } else if starts_with_cstr(last, b"file") {
                    nvim_xp_set_context(xp, expand_ctx::EXPAND_BUFFERS);
                } else {
                    nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
                }
            }
            _ => {
                nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
            }
        }
    }
}

/// Check if a C string starts with the given prefix bytes (no NUL needed in prefix).
#[inline]
unsafe fn starts_with_cstr(s: *const c_char, prefix: &[u8]) -> bool {
    starts_with_bytes(s.cast::<u8>(), prefix)
}

/// Return a static byte string as a mutable *mut c_char.
///
/// SAFETY: Vim's completion code treats this as immutable; the pointer
/// is valid for the lifetime of the static data.
#[inline]
fn static_str(s: &'static [u8]) -> *mut c_char {
    s.as_ptr().cast::<c_char>().cast_mut()
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
