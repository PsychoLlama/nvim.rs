//! Option :set command processing
//!
//! This module provides Rust FFI implementations for the `:set`, `:setlocal`,
//! and `:setglobal` command processing infrastructure.

use std::ffi::{c_char, c_int, c_uint};
use std::ptr;

use crate::{OptInt, OptScope, SetPrefix, FAIL, OK};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // State accessors
    fn nvim_get_p_verbose() -> OptInt;
}

// =============================================================================
// Set Command Flags
// =============================================================================

/// Flags for :set command processing.
pub mod set_flags {
    use std::ffi::c_int;

    /// OPT_LOCAL - set local value
    pub const OPT_LOCAL: c_int = 0x02;
    /// OPT_GLOBAL - set global value
    pub const OPT_GLOBAL: c_int = 0x01;
    /// OPT_MODELINE - set from modeline
    pub const OPT_MODELINE: c_int = 0x04;
    /// OPT_WINONLY - only set window-local options
    pub const OPT_WINONLY: c_int = 0x08;
    /// OPT_NOWIN - don't set window-local options
    pub const OPT_NOWIN: c_int = 0x10;
    /// OPT_ONECOLUMN - show options in one column
    pub const OPT_ONECOLUMN: c_int = 0x20;
    /// OPT_NO_REDRAW - don't redraw screen
    pub const OPT_NO_REDRAW: c_int = 0x40;
    /// OPT_SKIPRTP - don't change runtimepath
    pub const OPT_SKIPRTP: c_int = 0x80;
}

// =============================================================================
// Set Command Type
// =============================================================================

/// Type of :set command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetCommandType {
    /// :set - set both scopes
    #[default]
    Set = 0,
    /// :setlocal - set local scope only
    SetLocal = 1,
    /// :setglobal - set global scope only
    SetGlobal = 2,
}

impl SetCommandType {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::SetLocal,
            2 => Self::SetGlobal,
            _ => Self::Set,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Get the scope flags for this command type.
    #[must_use]
    pub const fn to_opt_flags(self) -> c_int {
        match self {
            Self::Set => 0,
            Self::SetLocal => set_flags::OPT_LOCAL,
            Self::SetGlobal => set_flags::OPT_GLOBAL,
        }
    }

    /// Check if this is :setlocal.
    #[must_use]
    pub const fn is_local(self) -> bool {
        matches!(self, Self::SetLocal)
    }

    /// Check if this is :setglobal.
    #[must_use]
    pub const fn is_global(self) -> bool {
        matches!(self, Self::SetGlobal)
    }
}

/// FFI: Create SetCommandType from flags.
#[no_mangle]
pub extern "C" fn rs_set_command_type_from_flags(flags: c_int) -> c_int {
    if (flags & set_flags::OPT_LOCAL) != 0 {
        SetCommandType::SetLocal.to_c_int()
    } else if (flags & set_flags::OPT_GLOBAL) != 0 {
        SetCommandType::SetGlobal.to_c_int()
    } else {
        SetCommandType::Set.to_c_int()
    }
}

/// FFI: Get opt flags from command type.
#[no_mangle]
pub extern "C" fn rs_set_command_type_to_flags(cmd_type: c_int) -> c_int {
    SetCommandType::from_c_int(cmd_type).to_opt_flags()
}

// =============================================================================
// Set Argument Parsing
// =============================================================================

/// Argument type in :set command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetArgType {
    /// Show value (no assignment)
    #[default]
    Show = 0,
    /// Query value with ?
    Query = 1,
    /// Set to value
    Assign = 2,
    /// Reset to default with &
    Reset = 3,
    /// Toggle (invert) with inv or !
    Toggle = 4,
    /// Append with +=
    Append = 5,
    /// Prepend with ^=
    Prepend = 6,
    /// Remove with -=
    Remove = 7,
}

impl SetArgType {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Query,
            2 => Self::Assign,
            3 => Self::Reset,
            4 => Self::Toggle,
            5 => Self::Append,
            6 => Self::Prepend,
            7 => Self::Remove,
            _ => Self::Show,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this modifies the value.
    #[must_use]
    pub const fn modifies_value(self) -> bool {
        !matches!(self, Self::Show | Self::Query)
    }

    /// Check if this is a compound assignment (+=, ^=, -=).
    #[must_use]
    pub const fn is_compound(self) -> bool {
        matches!(self, Self::Append | Self::Prepend | Self::Remove)
    }
}

/// FFI: Check if arg type modifies value.
#[no_mangle]
pub extern "C" fn rs_set_arg_modifies(arg_type: c_int) -> c_int {
    c_int::from(SetArgType::from_c_int(arg_type).modifies_value())
}

/// FFI: Check if arg type is compound assignment.
#[no_mangle]
pub extern "C" fn rs_set_arg_is_compound(arg_type: c_int) -> c_int {
    c_int::from(SetArgType::from_c_int(arg_type).is_compound())
}

// =============================================================================
// Set Command Result
// =============================================================================

/// Result of a :set command operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SetCommandResult {
    /// Error message (NULL if success)
    pub errmsg: *const c_char,
    /// Return code (OK or FAIL)
    pub retval: c_int,
    /// Number of options processed
    pub count: c_int,
    /// Whether any option was changed
    pub changed: c_int,
}

impl Default for SetCommandResult {
    fn default() -> Self {
        Self {
            errmsg: ptr::null(),
            retval: OK,
            count: 0,
            changed: 0,
        }
    }
}

/// FFI: Create a success result.
#[no_mangle]
pub extern "C" fn rs_set_result_success(count: c_int, changed: c_int) -> SetCommandResult {
    SetCommandResult {
        errmsg: ptr::null(),
        retval: OK,
        count,
        changed,
    }
}

/// FFI: Create a failure result.
#[no_mangle]
pub extern "C" fn rs_set_result_fail(errmsg: *const c_char) -> SetCommandResult {
    SetCommandResult {
        errmsg,
        retval: FAIL,
        count: 0,
        changed: 0,
    }
}

/// FFI: Check if SetCommandResult is success.
///
/// # Safety
/// `result` must be a valid pointer to a `SetCommandResult` or null.
#[no_mangle]
pub unsafe extern "C" fn rs_setcmd_result_is_ok(result: *const SetCommandResult) -> c_int {
    if result.is_null() {
        return 0;
    }
    c_int::from((*result).retval == OK)
}

// =============================================================================
// Show Options Processing
// =============================================================================

/// Mode for showing options.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShowOptionsMode {
    /// Show current value
    #[default]
    Value = 0,
    /// Show all options
    All = 1,
    /// Show options that differ from default
    Changed = 2,
    /// Show terminal options
    Terminal = 3,
}

impl ShowOptionsMode {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::All,
            2 => Self::Changed,
            3 => Self::Terminal,
            _ => Self::Value,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// FFI: Check if showing all options.
#[no_mangle]
pub extern "C" fn rs_show_mode_is_all(mode: c_int) -> c_int {
    c_int::from(ShowOptionsMode::from_c_int(mode) == ShowOptionsMode::All)
}

/// FFI: Check if showing changed options.
#[no_mangle]
pub extern "C" fn rs_show_mode_is_changed(mode: c_int) -> c_int {
    c_int::from(ShowOptionsMode::from_c_int(mode) == ShowOptionsMode::Changed)
}

/// FFI: Check if showing terminal options.
#[no_mangle]
pub extern "C" fn rs_show_mode_is_terminal(mode: c_int) -> c_int {
    c_int::from(ShowOptionsMode::from_c_int(mode) == ShowOptionsMode::Terminal)
}

// =============================================================================
// Silent Mode Checking
// =============================================================================

// Note: rs_is_emsg_silent is defined in the message crate

/// Check if verbose mode is enabled for :set commands.
#[no_mangle]
pub unsafe extern "C" fn rs_setcmd_is_verbose() -> c_int {
    c_int::from(nvim_get_p_verbose() > 0)
}

/// Get verbose level for :set commands.
#[no_mangle]
pub unsafe extern "C" fn rs_setcmd_get_verbose_level() -> OptInt {
    nvim_get_p_verbose()
}

// =============================================================================
// Option Scope Resolution
// =============================================================================

/// Resolve effective scope for an option based on command type and option support.
///
/// # Arguments
/// * `cmd_type` - The :set command type (0=set, 1=setlocal, 2=setglobal)
/// * `opt_scope_support` - Bitmask of scopes the option supports
///
/// # Returns
/// The effective scope to use.
#[no_mangle]
pub extern "C" fn rs_resolve_effective_scope(cmd_type: c_int, opt_scope_support: c_uint) -> c_int {
    let cmd = SetCommandType::from_c_int(cmd_type);

    // For :setglobal, always use global scope
    if cmd.is_global() {
        return OptScope::Global as c_int;
    }

    // For :setlocal, prefer window > buffer > global
    if cmd.is_local() {
        if (opt_scope_support & 0x04) != 0 {
            // SCOPE_WINDOW
            return OptScope::Win as c_int;
        }
        if (opt_scope_support & 0x02) != 0 {
            // SCOPE_BUFFER
            return OptScope::Buf as c_int;
        }
    }

    // Default to global
    OptScope::Global as c_int
}

/// Check if an option should be set at a given scope.
///
/// # Arguments
/// * `cmd_type` - The :set command type
/// * `scope` - The scope to check
/// * `opt_scope_support` - Bitmask of scopes the option supports
///
/// # Returns
/// 1 if the option should be set at this scope, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_should_set_at_scope(
    cmd_type: c_int,
    scope: c_int,
    opt_scope_support: c_uint,
) -> c_int {
    let cmd = SetCommandType::from_c_int(cmd_type);

    // For :setglobal, only set global
    if cmd.is_global() {
        return c_int::from(scope == OptScope::Global as c_int);
    }

    // For :setlocal, only set local
    if cmd.is_local() {
        return c_int::from(scope != OptScope::Global as c_int);
    }

    // For :set, set both if the option supports both
    // Check if the requested scope is supported
    let scope_bit = match scope {
        0 => 0x01, // Global
        1 => 0x02, // Buffer
        2 => 0x04, // Window
        _ => return 0,
    };

    c_int::from((opt_scope_support & scope_bit) != 0)
}

// =============================================================================
// Option Copy Infrastructure
// =============================================================================

/// Direction for copying option values.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CopyOptDirection {
    /// Copy from global to local (for new window/buffer)
    #[default]
    GlobalToLocal = 0,
    /// Copy from local to global
    LocalToGlobal = 1,
}

impl CopyOptDirection {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::LocalToGlobal,
            _ => Self::GlobalToLocal,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// FFI: Get copy direction for new window initialization.
#[no_mangle]
pub extern "C" fn rs_copy_direction_for_new_win() -> c_int {
    CopyOptDirection::GlobalToLocal.to_c_int()
}

/// FFI: Get copy direction for :setglobal from local.
#[no_mangle]
pub extern "C" fn rs_copy_direction_local_to_global() -> c_int {
    CopyOptDirection::LocalToGlobal.to_c_int()
}

// =============================================================================
// C Function Declarations for :set command processing
// =============================================================================

extern "C" {
    // Option display functions
    fn showoptions(all: c_int, opt_flags: c_int);
    fn showoneopt(opt: *const std::ffi::c_void, opt_flags: c_int);

    // Option setting functions
    fn set_options_default(opt_flags: c_int);
    fn didset_options();
    fn didset_options2();
    fn ui_refresh_options();

    // Screen update
    fn redraw_all_later(type_: c_int);

    // Option lookup and validation
    fn find_option_end(arg: *const c_char, opt_idx: *mut c_int) -> *const c_char;
    fn is_tty_option(name: *const c_char) -> c_int;
    fn get_varp_scope(opt: *const std::ffi::c_void, opt_flags: c_int) -> *mut std::ffi::c_void;
    fn validate_opt_idx(
        win: *const std::ffi::c_void,
        opt_idx: c_int,
        opt_flags: c_int,
        flags: u32,
        prefix: c_int,
        errmsg: *mut *const c_char,
    ) -> c_int;
    fn option_has_type(opt_idx: c_int, type_: c_int) -> c_int;
    fn option_has_scope(opt_idx: c_int, scope: c_int) -> c_int;
    fn option_scope_idx(opt_idx: c_int, scope: c_int) -> c_int;
    fn get_option_newval(
        opt_idx: c_int,
        opt_flags: c_int,
        prefix: c_int,
        argp: *mut *mut c_char,
        nextchar: u8,
        op: c_int,
        flags: u32,
        varp: *mut std::ffi::c_void,
        errbuf: *mut c_char,
        errbuflen: usize,
        errmsg: *mut *const c_char,
    ) -> OptVal;
    fn set_option(
        opt_idx: c_int,
        value: OptVal,
        opt_flags: c_int,
        set_sid: c_int,
        direct: c_int,
        value_replaced: c_int,
        errbuf: *mut c_char,
        errbuflen: usize,
    ) -> *const c_char;

    // String functions
    fn skiptowhite_esc(arg: *const c_char) -> *mut c_char;
    fn skipwhite(arg: *const c_char) -> *mut c_char;
    fn ascii_iswhite(c: c_int) -> c_int;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // Message functions
    fn msg_putchar(c: c_int);
    fn msg_ext_set_kind(kind: *const c_char);
    fn gotocmdline(clr: c_int) -> c_int;
    fn last_set_msg(sctx: ScriptContext);
    fn emsg(s: *const c_char) -> c_int;
    fn trans_characters(buf: *mut c_char, bufsize: c_int);
    fn vim_snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, size: usize) -> usize;

    // State accessors
    fn nvim_get_curwin() -> *const std::ffi::c_void;
    fn nvim_get_curbuf() -> *mut std::ffi::c_void;
    fn nvim_get_options_array() -> *const std::ffi::c_void;
    fn nvim_get_option_flags(opt_idx: c_int) -> u32;
    fn nvim_get_option_var(opt_idx: c_int) -> *mut std::ffi::c_void;
    fn nvim_get_option_script_ctx(opt_idx: c_int) -> ScriptContext;
    fn nvim_get_win_p_script_ctx(win: *const std::ffi::c_void, idx: c_int) -> ScriptContext;
    fn nvim_get_buf_p_script_ctx(buf: *const std::ffi::c_void, idx: c_int) -> ScriptContext;
    fn nvim_get_silent_mode() -> c_int;
    fn nvim_set_silent_mode(val: c_int);
    fn nvim_set_info_message(val: c_int);
    fn nvim_get_no_wait_return() -> c_int;
    fn nvim_set_no_wait_return(val: c_int);
    fn nvim_get_iobuff() -> *mut c_char;
}

// =============================================================================
// Types from C
// =============================================================================

/// Invalid option index constant
const K_OPT_INVALID: c_int = -1;

/// Option value type for booleans
const K_OPT_VAL_TYPE_BOOLEAN: c_int = 0;

/// Option scope constants
const K_OPT_SCOPE_WIN: c_int = 1;
const K_OPT_SCOPE_BUF: c_int = 2;

/// UPD_CLEAR constant for redraw
const UPD_CLEAR: c_int = 70;

/// IOSIZE constant
const IOSIZE: usize = 1025;

/// Error buffer length
const ERR_BUFLEN: usize = 256;

/// Script context structure (matches C's sctx_T)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScriptContext {
    pub sc_sid: c_int,
    pub sc_seq: c_int,
    pub sc_lnum: i64,
}

/// Option value union (matches C's OptVal)
#[repr(C)]
#[derive(Clone, Copy)]
pub union OptValData {
    pub boolean: c_int,
    pub number: i64,
    pub string: *mut c_char,
}

/// Option value structure (matches C's OptVal)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OptVal {
    pub type_: c_int,
    pub data: OptValData,
}

impl Default for OptVal {
    fn default() -> Self {
        Self {
            type_: -1, // kOptValTypeNil
            data: OptValData { boolean: 0 },
        }
    }
}

/// Error messages
mod errmsg {
    pub const E_UNKNOWN_OPTION: &[u8] = b"E518: Unknown option\0";
    pub const E_INVARG: &[u8] = b"E474: Invalid argument\0";
    pub const E_TRAILING: &[u8] = b"E488: Trailing characters\0";
}

// =============================================================================
// :set Command Implementation
// =============================================================================

/// Process the :set command.
///
/// This is the main entry point for `:set`, `:setlocal`, and `:setglobal` commands.
///
/// # Arguments
/// * `arg` - The argument string (may be modified in place)
/// * `opt_flags` - Option flags (OPT_LOCAL, OPT_GLOBAL, OPT_MODELINE, etc.)
///
/// # Returns
/// OK on success, FAIL on error.
///
/// # Safety
/// `arg` must be a valid, writable C string.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_do_set(arg: *mut c_char, opt_flags: c_int) -> c_int {
    let did_show: c_int = if arg.is_null() || *arg == 0 {
        // ":set" without arguments - show modified options
        showoptions(0, opt_flags);
        1
    } else {
        do_set_process_args(arg, opt_flags)
    };

    // Handle silent mode display
    if nvim_get_silent_mode() != 0 && did_show != 0 {
        nvim_set_silent_mode(0);
        nvim_set_info_message(1);
        msg_putchar(c_int::from(b'\n'));
        nvim_set_silent_mode(1);
        nvim_set_info_message(0);
    }

    // Return value is in the sign bit: negative means FAIL was returned
    if did_show < 0 {
        FAIL
    } else {
        OK
    }
}

/// Process :set arguments. Returns did_show flag, or negative on error.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
unsafe fn do_set_process_args(arg: *mut c_char, opt_flags: c_int) -> c_int {
    let mut did_show: c_int = 0;
    let mut p = arg;

    while *p != 0 {
        // Check for "all" keyword
        if check_set_all(&raw mut p, opt_flags, &mut did_show) {
            // "all" was processed, continue to next arg
        } else {
            // Process a single option
            let startarg = p;
            let mut errmsg: *const c_char = ptr::null();
            let mut errbuf: [c_char; ERR_BUFLEN] = [0; ERR_BUFLEN];

            rs_do_one_set_option(
                opt_flags,
                &raw mut p,
                &raw mut did_show,
                errbuf.as_mut_ptr(),
                ERR_BUFLEN,
                &raw mut errmsg,
            );

            // Advance to next argument:
            // - skip until a blank found, taking care of backslashes
            // - skip blanks
            // - skip one "=val" argument (for hidden options ":set gfn =xx")
            for _ in 0..2 {
                p = skiptowhite_esc(p);
                p = skipwhite(p);
                if *p as u8 != b'=' {
                    break;
                }
            }

            if !errmsg.is_null() {
                format_and_show_error(startarg, p, errmsg);
                return -1; // Signal FAIL
            }
        }

        p = skipwhite(p);
    }

    did_show
}

/// Format and display an error message for :set command.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
unsafe fn format_and_show_error(
    startarg: *const c_char,
    endarg: *const c_char,
    errmsg: *const c_char,
) {
    let iobuff = nvim_get_iobuff();
    let i = vim_snprintf(iobuff, IOSIZE, c"%s".as_ptr(), errmsg) + 2;

    let arg_len = endarg.offset_from(startarg);
    if i + (arg_len as c_int) < IOSIZE as c_int {
        // Append the argument with the error
        xstrlcpy(
            iobuff.add((i - 2) as usize),
            c": ".as_ptr(),
            IOSIZE - (i as usize) + 2,
        );
        ptr::copy_nonoverlapping(startarg, iobuff.add(i as usize), arg_len as usize);
        *iobuff.add((i as usize) + (arg_len as usize)) = 0;
    }

    // Make sure all characters are printable
    trans_characters(iobuff, IOSIZE as c_int);

    // Show error
    let no_wait = nvim_get_no_wait_return();
    nvim_set_no_wait_return(no_wait + 1);
    emsg(iobuff);
    nvim_set_no_wait_return(no_wait);
}

/// Check and process "all" keyword in :set command.
///
/// Returns true if "all" was found and processed.
#[inline]
unsafe fn check_set_all(argp: *mut *mut c_char, opt_flags: c_int, did_show: &mut c_int) -> bool {
    let arg = *argp;

    // Check for "all" (not from modeline)
    if *arg as u8 == b'a'
        && *arg.add(1) as u8 == b'l'
        && *arg.add(2) as u8 == b'l'
        && !(*arg.add(3) as u8).is_ascii_alphabetic()
        && (opt_flags & set_flags::OPT_MODELINE) == 0
    {
        *argp = arg.add(3);

        if *(*argp) as u8 == b'&' {
            // ":set all&" - reset all options to default
            *argp = (*argp).add(1);
            set_options_default(opt_flags);
            didset_options();
            didset_options2();
            ui_refresh_options();
            redraw_all_later(UPD_CLEAR);
        } else {
            // ":set all" - show all options
            showoptions(1, opt_flags);
            *did_show = 1;
        }

        return true;
    }

    false
}

/// Process a single option in :set command.
///
/// # Arguments
/// * `opt_flags` - Option flags
/// * `argp` - Pointer to argument string pointer (updated)
/// * `did_show` - Whether we've shown an option value
/// * `errbuf` - Buffer for error messages
/// * `errbuflen` - Size of error buffer
/// * `errmsg` - Pointer to store error message
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_do_one_set_option(
    opt_flags: c_int,
    argp: *mut *mut c_char,
    did_show: *mut c_int,
    errbuf: *mut c_char,
    errbuflen: usize,
    errmsg: *mut *const c_char,
) {
    // Get option prefix (no/inv)
    let prefix = rs_get_option_prefix_internal(argp);
    let arg = *argp;

    // Find end of option name and get option index
    let mut opt_idx: c_int = K_OPT_INVALID;
    let option_end = find_option_end(arg, &raw mut opt_idx);

    if opt_idx != K_OPT_INVALID {
        // Valid option found
    } else if is_tty_option(arg) != 0 {
        // Silently ignore TTY options
        return;
    } else {
        // Invalid option name
        *errmsg = errmsg::E_UNKNOWN_OPTION.as_ptr().cast();
        return;
    }

    // Remember character after option name
    let afterchar = *option_end as u8;
    let mut p = option_end.cast_mut();

    // Skip whitespace
    while ascii_iswhite(c_int::from(*p)) != 0 {
        p = p.add(1);
    }

    // Get operator (+=, ^=, -=)
    let op = rs_get_op_internal(p);
    if op != 0 {
        p = p.add(1);
    }

    let nextchar = *p as u8;
    let flags = nvim_get_option_flags(opt_idx);
    let opt = nvim_get_options_array()
        .cast::<u8>()
        .add(opt_idx as usize * get_option_struct_size());
    let varp = get_varp_scope(opt.cast(), opt_flags);

    // Validate option
    if validate_opt_idx(nvim_get_curwin(), opt_idx, opt_flags, flags, prefix, errmsg) == FAIL {
        return;
    }

    // Check for special characters
    if !vim_strchr(c"?=:!&<".as_ptr(), c_int::from(nextchar)).is_null() {
        *argp = p;

        // Handle &vi and &vim
        if nextchar == b'&' && *(*argp).add(1) as u8 == b'v' && *(*argp).add(2) as u8 == b'i' {
            if *(*argp).add(3) as u8 == b'm' {
                // "opt&vim": set to Vim default
                *argp = (*argp).add(3);
            } else {
                // "opt&vi": set to Vi default
                *argp = (*argp).add(2);
            }
        }

        // Check for trailing characters after special chars
        if !vim_strchr(c"?!&<".as_ptr(), c_int::from(nextchar)).is_null()
            && *(*argp).add(1) != 0
            && ascii_iswhite(c_int::from(*(*argp).add(1))) == 0
        {
            *errmsg = errmsg::E_TRAILING.as_ptr().cast();
            return;
        }
    }

    // Determine what action to take
    let is_bool = option_has_type(opt_idx, K_OPT_VAL_TYPE_BOOLEAN) != 0;

    if nextchar == b'?'
        || (prefix == SetPrefix::None as c_int
            && vim_strchr(c"=:&<".as_ptr(), c_int::from(nextchar)).is_null()
            && !is_bool)
    {
        // Print value
        if *did_show != 0 {
            msg_putchar(c_int::from(b'\n'));
        } else {
            msg_ext_set_kind(c"list_cmd".as_ptr());
            gotocmdline(1);
            *did_show = 1;
        }
        showoneopt(opt.cast(), opt_flags);

        // Verbose mode: show where option was last set
        if nvim_get_p_verbose() > 0 {
            let opt_var = nvim_get_option_var(opt_idx);
            if varp == opt_var {
                last_set_msg(nvim_get_option_script_ctx(opt_idx));
            } else if option_has_scope(opt_idx, K_OPT_SCOPE_WIN) != 0 {
                let idx = option_scope_idx(opt_idx, K_OPT_SCOPE_WIN);
                last_set_msg(nvim_get_win_p_script_ctx(nvim_get_curwin(), idx));
            } else if option_has_scope(opt_idx, K_OPT_SCOPE_BUF) != 0 {
                let idx = option_scope_idx(opt_idx, K_OPT_SCOPE_BUF);
                last_set_msg(nvim_get_buf_p_script_ctx(nvim_get_curbuf(), idx));
            }
        }

        if nextchar != b'?' && nextchar != 0 && ascii_iswhite(c_int::from(afterchar)) == 0 {
            *errmsg = errmsg::E_TRAILING.as_ptr().cast();
        }
        return;
    }

    // Handle boolean options
    if is_bool {
        if !vim_strchr(c"=:".as_ptr(), c_int::from(nextchar)).is_null() {
            *errmsg = errmsg::E_INVARG.as_ptr().cast();
            return;
        }

        if vim_strchr(c"!&<".as_ptr(), c_int::from(nextchar)).is_null()
            && nextchar != 0
            && ascii_iswhite(c_int::from(afterchar)) == 0
        {
            *errmsg = errmsg::E_TRAILING.as_ptr().cast();
            return;
        }
    } else {
        // Non-boolean: must have =, :, &, or <
        if vim_strchr(c"=:&<".as_ptr(), c_int::from(nextchar)).is_null() {
            *errmsg = errmsg::E_INVARG.as_ptr().cast();
            return;
        }
    }

    // Get new value
    let newval = get_option_newval(
        opt_idx, opt_flags, prefix, argp, nextchar, op, flags, varp, errbuf, errbuflen, errmsg,
    );

    if newval.type_ == -1 || !(*errmsg).is_null() {
        return;
    }

    // Set the option
    let value_replaced = c_int::from(op == 0);
    *errmsg = set_option(
        opt_idx,
        newval,
        opt_flags,
        0,
        0,
        value_replaced,
        errbuf,
        errbuflen,
    );
}

/// Get option prefix (no/inv) - internal version that returns c_int.
#[inline]
unsafe fn rs_get_option_prefix_internal(argp: *mut *mut c_char) -> c_int {
    if argp.is_null() || (*argp).is_null() {
        return SetPrefix::None as c_int;
    }

    let arg = *argp;

    // Check for "no" prefix
    if *arg as u8 == b'n' && *arg.add(1) as u8 == b'o' {
        *argp = arg.add(2);
        return SetPrefix::No as c_int;
    }

    // Check for "inv" prefix
    if *arg as u8 == b'i' && *arg.add(1) as u8 == b'n' && *arg.add(2) as u8 == b'v' {
        *argp = arg.add(3);
        return SetPrefix::Inv as c_int;
    }

    SetPrefix::None as c_int
}

/// Get operator (+=, ^=, -=) - internal version that returns c_int.
#[inline]
unsafe fn rs_get_op_internal(p: *const c_char) -> c_int {
    if p.is_null() || *p == 0 {
        return 0;
    }

    let c0 = *p as u8;
    let c1 = *p.add(1) as u8;

    if c1 == b'=' {
        match c0 {
            b'+' => 1, // OP_ADDING
            b'^' => 2, // OP_PREPENDING
            b'-' => 3, // OP_REMOVING
            _ => 0,
        }
    } else {
        0
    }
}

/// Get the size of the vimoption_T struct.
/// This is needed for array indexing into options[].
#[inline]
const fn get_option_struct_size() -> usize {
    // vimoption_T is roughly 120 bytes on 64-bit systems
    // This should match the C struct size
    128
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_command_type() {
        assert_eq!(SetCommandType::Set.to_c_int(), 0);
        assert_eq!(SetCommandType::SetLocal.to_c_int(), 1);
        assert_eq!(SetCommandType::SetGlobal.to_c_int(), 2);

        assert_eq!(SetCommandType::from_c_int(0), SetCommandType::Set);
        assert_eq!(SetCommandType::from_c_int(1), SetCommandType::SetLocal);
        assert_eq!(SetCommandType::from_c_int(2), SetCommandType::SetGlobal);
    }

    #[test]
    fn test_set_command_type_flags() {
        assert_eq!(SetCommandType::Set.to_opt_flags(), 0);
        assert_eq!(
            SetCommandType::SetLocal.to_opt_flags(),
            set_flags::OPT_LOCAL
        );
        assert_eq!(
            SetCommandType::SetGlobal.to_opt_flags(),
            set_flags::OPT_GLOBAL
        );

        assert!(SetCommandType::SetLocal.is_local());
        assert!(!SetCommandType::SetLocal.is_global());
        assert!(SetCommandType::SetGlobal.is_global());
        assert!(!SetCommandType::SetGlobal.is_local());
    }

    #[test]
    fn test_set_arg_type() {
        assert!(!SetArgType::Show.modifies_value());
        assert!(!SetArgType::Query.modifies_value());
        assert!(SetArgType::Assign.modifies_value());
        assert!(SetArgType::Reset.modifies_value());
        assert!(SetArgType::Toggle.modifies_value());

        assert!(!SetArgType::Show.is_compound());
        assert!(SetArgType::Append.is_compound());
        assert!(SetArgType::Prepend.is_compound());
        assert!(SetArgType::Remove.is_compound());
    }

    #[test]
    fn test_show_options_mode() {
        assert_eq!(ShowOptionsMode::from_c_int(0), ShowOptionsMode::Value);
        assert_eq!(ShowOptionsMode::from_c_int(1), ShowOptionsMode::All);
        assert_eq!(ShowOptionsMode::from_c_int(2), ShowOptionsMode::Changed);
        assert_eq!(ShowOptionsMode::from_c_int(3), ShowOptionsMode::Terminal);

        assert_eq!(rs_show_mode_is_all(1), 1);
        assert_eq!(rs_show_mode_is_all(0), 0);
        assert_eq!(rs_show_mode_is_changed(2), 1);
        assert_eq!(rs_show_mode_is_terminal(3), 1);
    }

    #[test]
    fn test_set_result() {
        let success = rs_set_result_success(5, 1);
        assert!(success.errmsg.is_null());
        assert_eq!(success.retval, OK);
        assert_eq!(success.count, 5);
        assert_eq!(success.changed, 1);

        let errmsg = c"test error".as_ptr();
        let fail = rs_set_result_fail(errmsg);
        assert_eq!(fail.errmsg, errmsg);
        assert_eq!(fail.retval, FAIL);
    }

    #[test]
    fn test_copy_direction() {
        assert_eq!(rs_copy_direction_for_new_win(), 0);
        assert_eq!(rs_copy_direction_local_to_global(), 1);
    }

    #[test]
    fn test_resolve_effective_scope() {
        // :setglobal always returns global
        assert_eq!(
            rs_resolve_effective_scope(2, 0x07),
            OptScope::Global as c_int
        );

        // :setlocal with window support returns window
        assert_eq!(rs_resolve_effective_scope(1, 0x04), OptScope::Win as c_int);

        // :setlocal with buffer support returns buffer
        assert_eq!(rs_resolve_effective_scope(1, 0x02), OptScope::Buf as c_int);

        // :set returns global by default
        assert_eq!(
            rs_resolve_effective_scope(0, 0x01),
            OptScope::Global as c_int
        );
    }
}
