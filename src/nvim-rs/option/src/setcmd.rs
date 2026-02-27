//! Option :set command processing
//!
//! This module provides Rust FFI implementations for the `:set`, `:setlocal`,
//! and `:setglobal` command processing infrastructure.

use std::ffi::{c_char, c_int, c_uint};
use std::ptr;

use crate::index::{val_type, OptIndex};
use crate::opt_index::K_OPT_COUNT;
use crate::storage::OptVal;
use crate::{OptInt, OptScope, OptValType, SetPrefix, FAIL, OK};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // State accessors
    fn nvim_get_p_verbose() -> OptInt;
    // Option metadata (also used by rs_set_context_in_set_cmd)
    fn find_option_len(name: *const c_char, len: usize) -> OptIndex;
    fn is_option_hidden(opt_idx: OptIndex) -> c_int;
    fn get_special_key_code(name: *const c_char) -> c_int;
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
    // Option setting functions (wrappers for static functions)
    fn nvim_set_options_default(opt_flags: c_int);
    fn nvim_didset_options();
    fn nvim_didset_options2();
    fn ui_refresh_options();

    // Screen update
    fn redraw_all_later(type_: c_int);

    // Option lookup and validation
    fn find_option_end(arg: *const c_char, opt_idx: *mut c_int) -> *const c_char;
    fn rs_is_tty_option(name: *const c_char) -> c_int;
    fn nvim_validate_opt_idx(
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
    fn nvim_rs_set_option(
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
    fn rs_ascii_iswhite(c: c_int) -> c_int;
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

    // get_option_newval dependencies
    fn rs_optval_from_varp(opt_idx: c_int, varp: *mut std::ffi::c_void) -> OptVal;
    fn rs_optval_copy(o: OptVal) -> OptVal;
    fn rs_get_option_default(opt_idx: c_int, opt_flags: c_int) -> OptVal;
    fn rs_option_is_global_local(opt_idx: c_int) -> c_int;
    fn nvim_get_varp_opt(opt_idx: c_int) -> *mut std::ffi::c_void;
    fn nvim_call_unset_option_local_value(opt_idx: c_int) -> *const c_char;
    fn nvim_get_option_value_global(opt_idx: c_int) -> OptVal;
    fn nvim_call_vim_str2nr(arg: *const c_char, len_out: *mut c_int, num_out: *mut crate::OptInt);
    fn rs_string_to_key(arg: *mut c_char) -> c_int;
    fn rs_stropt_get_newval(
        nextchar: c_int,
        opt_idx: c_int,
        argp: *mut *mut c_char,
        varp: *mut std::ffi::c_void,
        origval: *const c_char,
        op_arg: *mut c_int,
        flags: u32,
    ) -> *mut c_char;
    fn nvim_option_get_p_wc_ptr() -> *const std::ffi::c_void;
    fn nvim_option_get_p_wcm_ptr() -> *const std::ffi::c_void;
    fn nvim_get_e_number_required_after_equal() -> *const c_char;
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

/// UPD_CLEAR constant for redraw (from drawscreen.h)
const UPD_CLEAR: c_int = 50;

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

/// Error messages
mod errmsg {
    pub const E_UNKNOWN_OPTION: &[u8] = b"E518: Unknown option\0";
    pub const E_INVARG: &[u8] = b"E474: Invalid argument\0";
    pub const E_TRAILING: &[u8] = b"E488: Trailing characters\0";
}

/// SetPrefix constants (must match set_prefix_T in C)
const PREFIX_NO: c_int = 0;
const PREFIX_INV: c_int = 2;

/// SetOp constants (must match set_op_T in C)
const OP_ADDING: c_int = 1;
const OP_PREPENDING: c_int = 2;
const OP_REMOVING: c_int = 3;

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
        rs_showoptions(0, opt_flags);
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
            nvim_set_options_default(opt_flags);
            nvim_didset_options();
            nvim_didset_options2();
            ui_refresh_options();
            redraw_all_later(UPD_CLEAR);
        } else {
            // ":set all" - show all options
            rs_showoptions(1, opt_flags);
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
    } else if rs_is_tty_option(arg) != 0 {
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
    while rs_ascii_iswhite(c_int::from(*p)) != 0 {
        p = p.add(1);
    }

    // Get operator (+=, ^=, -=)
    let op = rs_get_op_internal(p);
    if op != 0 {
        p = p.add(1);
    }

    let nextchar = *p as u8;
    let flags = nvim_get_option_flags(opt_idx);
    let varp = nvim_get_varp_scope_by_idx(opt_idx, opt_flags);

    // Validate option
    if nvim_validate_opt_idx(nvim_get_curwin(), opt_idx, opt_flags, flags, prefix, errmsg) == FAIL {
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
            && rs_ascii_iswhite(c_int::from(*(*argp).add(1))) == 0
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
        rs_showoneopt(opt_idx, opt_flags);

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

        if nextchar != b'?' && nextchar != 0 && rs_ascii_iswhite(c_int::from(afterchar)) == 0 {
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
            && rs_ascii_iswhite(c_int::from(afterchar)) == 0
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
    let newval = get_option_newval_impl(
        opt_idx, opt_flags, prefix, argp, nextchar, op, flags, varp, errbuf, errbuflen, errmsg,
    );

    if newval.type_ == OptValType::Nil || !(*errmsg).is_null() {
        return;
    }

    // Set the option
    let value_replaced = c_int::from(op == 0);
    *errmsg = nvim_rs_set_option(
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

/// Compute new option value from :set command arguments.
///
/// Rust translation of C `get_option_newval`. Handles boolean toggle/set/reset,
/// number parsing (decimal/octal/hex, wildchar special keys), string value via
/// `rs_stropt_get_newval`, operator application (+=, ^=, -=), and special
/// `&` (default) / `<` (global copy) modes.
///
/// # Safety
/// All pointers must be valid. `varp` must not be null.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::bool_to_int_with_if)]
unsafe fn get_option_newval_impl(
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
) -> OptVal {
    use crate::storage::{OptValData, String_};

    let _ = (errbuf, errbuflen); // Not used directly; errmsg is the output channel

    let mut newval = OptVal::nil();

    // ":set opt&": Reset to default value.
    // Use OPT_GLOBAL to avoid using the unset local value for global-local options.
    if nextchar == b'&' {
        return rs_optval_copy(rs_get_option_default(opt_idx, set_flags::OPT_GLOBAL));
    }

    // ":set opt<": Reset to global value.
    // ":setlocal opt<": Copy global value to local value.
    if nextchar == b'<' {
        if rs_option_is_global_local(opt_idx) != 0 && (opt_flags & set_flags::OPT_LOCAL) == 0 {
            // Ignore the error from unset_option_local_value -- C did the same
            nvim_call_unset_option_local_value(opt_idx);
        }
        return nvim_get_option_value_global(opt_idx);
    }

    // When setting the local value of a global option, the old value may be the global value.
    let oldval_is_global =
        rs_option_is_global_local(opt_idx) != 0 && (opt_flags & set_flags::OPT_LOCAL) != 0;
    let oldval_varp = if oldval_is_global {
        nvim_get_varp_opt(opt_idx)
    } else {
        varp
    };
    let oldval = rs_optval_from_varp(opt_idx, oldval_varp);

    match oldval.type_ {
        OptValType::Nil => {
            // Should not happen; mirror C abort() with a panic.
            panic!("get_option_newval_impl: oldval is Nil for opt_idx={opt_idx}");
        }
        OptValType::Boolean => {
            let newval_bool: c_int = if nextchar == b'!' {
                // ":set opt!": invert
                match oldval.data.boolean {
                    -1 => -1, // kNone stays kNone
                    0 => 1,   // kFalse -> kTrue
                    _ => 0,   // kTrue -> kFalse
                }
            } else {
                // ":set invopt": invert; ":set opt" / ":set noopt": set / reset
                if prefix == PREFIX_INV {
                    // XOR with 1, clamped to 0/1 (kFalse/kTrue)
                    let cur = *(varp.cast::<c_int>());
                    cur ^ 1
                } else if prefix == PREFIX_NO {
                    0 // kFalse
                } else {
                    1 // kTrue
                }
            };
            newval = OptVal {
                type_: OptValType::Boolean,
                data: OptValData {
                    boolean: newval_bool,
                },
            };
        }
        OptValType::Number => {
            let oldval_num = oldval.data.number;

            // Advance past '=' or ':'
            let arg = (*argp).add(1);

            let p_wc = nvim_option_get_p_wc_ptr();
            let p_wcm = nvim_option_get_p_wcm_ptr();
            let varp_as_optint = varp.cast::<crate::OptInt>();

            let newval_num: crate::OptInt;

            // Special handling for 'wildchar' / 'wildcharm'
            if (varp_as_optint == p_wc.cast::<crate::OptInt>().cast_mut()
                || varp_as_optint == p_wcm.cast::<crate::OptInt>().cast_mut())
                && (*arg as u8 == b'<'
                    || *arg as u8 == b'^'
                    || (*arg != 0
                        && (*arg.add(1) == 0
                            || rs_ascii_iswhite(c_int::from(*arg.add(1) as u8)) != 0)
                        && !(*arg as u8).is_ascii_digit()))
            {
                let key = rs_string_to_key(arg);
                if key == 0 {
                    *errmsg = errmsg::E_INVARG.as_ptr().cast();
                    return newval;
                }
                newval_num = crate::OptInt::from(key);
            } else if *arg as u8 == b'-' || (*arg as u8).is_ascii_digit() {
                // Allow negative, octal and hex numbers.
                let mut len: c_int = 0;
                let mut parsed: crate::OptInt = 0;
                nvim_call_vim_str2nr(arg, &raw mut len, &raw mut parsed);
                // Check that the whole token was consumed (no trailing non-whitespace).
                if len == 0
                    || (*arg.add(len as usize) != 0
                        && rs_ascii_iswhite(c_int::from(*arg.add(len as usize) as u8)) == 0)
                {
                    *errmsg = nvim_get_e_number_required_after_equal();
                    return newval;
                }
                newval_num = parsed;
            } else {
                *errmsg = nvim_get_e_number_required_after_equal();
                return newval;
            }

            let final_num = if op == OP_ADDING {
                oldval_num + newval_num
            } else if op == OP_PREPENDING {
                oldval_num * newval_num
            } else if op == OP_REMOVING {
                oldval_num - newval_num
            } else {
                newval_num
            };

            newval = OptVal {
                type_: OptValType::Number,
                data: OptValData { number: final_num },
            };
        }
        OptValType::String => {
            let oldval_str = oldval.data.string.data;
            let mut op_mut = op;
            // rs_stropt_get_newval advances *argp past the string value.
            let newval_str = rs_stropt_get_newval(
                c_int::from(nextchar),
                opt_idx,
                argp,
                varp,
                oldval_str,
                &raw mut op_mut,
                flags,
            );
            newval = OptVal {
                type_: OptValType::String,
                data: OptValData {
                    string: String_ {
                        data: newval_str,
                        size: if newval_str.is_null() {
                            0
                        } else {
                            libc_strlen(newval_str)
                        },
                    },
                },
            };
        }
    }

    newval
}

/// Compute the length of a C string (libc strlen equivalent).
#[inline]
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
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

// =============================================================================
// Phase 1: showoptions / showoneopt FFI declarations
// =============================================================================

extern "C" {
    fn msg_puts(s: *const c_char);
    fn msg_puts_title(s: *const c_char);
    fn msg_outtrans(s: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn nvim_message_filtered(msg: *const c_char) -> c_int;
    fn nvim_vim_strsize(s: *const c_char) -> c_int;
    fn os_breakcheck();
    fn nvim_get_got_int() -> c_int;
    fn nvim_get_Columns() -> c_int;
    fn nvim_excmds_set_msg_col(val: c_int);
    fn nvim_get_namebuff() -> *mut c_char;
    fn nvim_excmds_curbufIsChanged() -> c_int;
    fn nvim_varp_is_curbuf_b_changed(varp: *const std::ffi::c_void) -> c_int;
    fn nvim_get_varp_by_idx(opt_idx: c_int) -> *mut std::ffi::c_void;
    fn nvim_get_varp_scope_by_idx(opt_idx: c_int, opt_flags: c_int) -> *mut std::ffi::c_void;
    fn nvim_option_get_fullname(opt_idx: c_int) -> *const c_char;
    fn nvim_option_is_global_only(opt_idx: c_int) -> c_int;
    fn nvim_option_has_type(opt_idx: c_int, type_: c_int) -> c_int;
    fn nvim_opt_is_hidden(opt_idx: c_int) -> c_int;
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn strlen(s: *const c_char) -> usize;
}

// Constants for Phase 1
const INC: c_int = 20; // Column increment for showoptions
const GAP: c_int = 3; // Gap between columns

// Call rs_option_value2string/rs_optval_default by their exported names (#[no_mangle] in session.rs).
extern "C" {
    fn rs_option_value2string(opt_idx: c_int, opt_flags: c_int);
    fn rs_optval_default(opt_idx: c_int, varp: *mut std::ffi::c_void) -> c_int;
}

// =============================================================================
// Phase 1: rs_showoneopt implementation
// =============================================================================

/// Display a single option value (translation of C showoneopt).
///
/// # Safety
/// opt_idx must be a valid option index. opt_flags must be valid.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_showoneopt(opt_idx: c_int, opt_flags: c_int) {
    let save_silent = nvim_get_silent_mode();

    nvim_set_silent_mode(0);
    nvim_set_info_message(1); // use stdout, not stderr

    let varp = nvim_get_varp_scope_by_idx(opt_idx, opt_flags);
    let is_bool = nvim_option_has_type(opt_idx, K_OPT_VAL_TYPE_BOOLEAN) != 0;

    // Matches C logic:
    //   if (boolean && (is_modified_opt ? !curbufIsChanged() : !val)) -> "no"
    //   else if (boolean && val < 0) -> "--"
    //   else -> "  "
    if is_bool {
        let val = *(varp as *const c_int);
        let show_false = if nvim_varp_is_curbuf_b_changed(varp) != 0 {
            // 'modified' option: false when buffer not actually changed
            nvim_excmds_curbufIsChanged() == 0
        } else {
            val == 0
        };
        if show_false {
            msg_puts(c"no".as_ptr());
        } else if val < 0 {
            msg_puts(c"--".as_ptr());
        } else {
            msg_puts(c"  ".as_ptr());
        }
    } else {
        msg_puts(c"  ".as_ptr());
    }

    let fullname = nvim_option_get_fullname(opt_idx);
    msg_puts(fullname);

    if !is_bool {
        msg_putchar(c_int::from(b'='));
        // put value string in NameBuff
        rs_option_value2string(opt_idx, opt_flags);
        let namebuff = nvim_get_namebuff();
        msg_outtrans(namebuff, 0, false);
    }

    nvim_set_silent_mode(save_silent);
    nvim_set_info_message(0);
}

// =============================================================================
// Phase 1: rs_showoptions implementation
// =============================================================================

/// Display all or changed options (translation of C showoptions).
///
/// # Safety
/// opt_flags must be valid OPT_* flags.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_showoptions(all: c_int, opt_flags: c_int) {
    #[allow(clippy::cast_ptr_alignment)]
    let items: *mut c_int =
        xmalloc((K_OPT_COUNT as usize) * std::mem::size_of::<c_int>()).cast::<c_int>();

    msg_ext_set_kind(c"list_cmd".as_ptr());
    // Highlight title
    if (opt_flags & set_flags::OPT_GLOBAL) != 0 {
        msg_puts_title(c"\n--- Global option values ---".as_ptr());
    } else if (opt_flags & set_flags::OPT_LOCAL) != 0 {
        msg_puts_title(c"\n--- Local option values ---".as_ptr());
    } else {
        msg_puts_title(c"\n--- Options ---".as_ptr());
    }

    // Two-pass loop:
    // 1. display the short items
    // 2. display the long items (only strings and numbers)
    // When OPT_ONECOLUMN, do everything in run 2.
    let mut run: c_int = 1;
    while run <= 2 {
        if nvim_get_got_int() != 0 {
            break;
        }
        // Collect the items in items[]
        let mut item_count: c_int = 0;
        for opt_idx in 0..K_OPT_COUNT {
            // Skip hidden options
            if nvim_opt_is_hidden(opt_idx) != 0 {
                continue;
            }
            let fullname = nvim_option_get_fullname(opt_idx);
            // apply :filter /pat/
            if nvim_message_filtered(fullname) != 0 {
                continue;
            }

            #[allow(clippy::if_then_some_else_none)]
            let varp: *mut std::ffi::c_void =
                if (opt_flags & (set_flags::OPT_LOCAL | set_flags::OPT_GLOBAL)) != 0 {
                    if nvim_option_is_global_only(opt_idx) != 0 {
                        continue;
                    }
                    nvim_get_varp_scope_by_idx(opt_idx, opt_flags)
                } else {
                    nvim_get_varp_by_idx(opt_idx)
                };

            if varp.is_null() {
                continue;
            }
            if all == 0 && rs_optval_default(opt_idx, varp) != 0 {
                continue;
            }

            let len: c_int;
            if (opt_flags & set_flags::OPT_ONECOLUMN) != 0 {
                len = nvim_get_Columns();
            } else if nvim_option_has_type(opt_idx, K_OPT_VAL_TYPE_BOOLEAN) != 0 {
                len = 1; // a toggle option fits always
            } else {
                rs_option_value2string(opt_idx, opt_flags);
                let namebuff = nvim_get_namebuff();
                len = strlen(fullname) as c_int + nvim_vim_strsize(namebuff) + 1;
            }

            if (len <= INC - GAP && run == 1) || (len > INC - GAP && run == 2) {
                *items.add(item_count as usize) = opt_idx;
                item_count += 1;
            }
        }

        let rows: c_int = if run == 1 {
            let columns = nvim_get_Columns();
            let mut cols = (columns + GAP - 3) / INC;
            if cols == 0 {
                cols = 1;
            }
            (item_count + cols - 1) / cols
        } else {
            item_count
        };

        let mut row: c_int = 0;
        while row < rows && nvim_get_got_int() == 0 {
            msg_putchar(c_int::from(b'\n')); // go to next line
            if nvim_get_got_int() != 0 {
                // 'q' typed in more
                break;
            }
            let mut col: c_int = 0;
            let mut i = row;
            while i < item_count {
                nvim_excmds_set_msg_col(col); // make columns
                let show_opt_idx = *items.add(i as usize);
                rs_showoneopt(show_opt_idx, opt_flags);
                col += INC;
                i += rows;
            }
            os_breakcheck();
            row += 1;
        }
        run += 1;
    }

    xfree(items.cast::<c_char>());
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

// =============================================================================
// set_context_in_set_cmd migration (Phase 3)
// =============================================================================

/// Expand context constants (from cmdexpand_defs.h)
/// Values are computed from the C enum starting at EXPAND_NOTHING=0 and
/// incrementing by 1 for each subsequent enumerator.
mod expand_ctx {
    use std::ffi::c_int;
    pub const EXPAND_UNSUCCESSFUL: c_int = -2;
    pub const EXPAND_NOTHING: c_int = 0;
    // EXPAND_COMMANDS = 1
    pub const EXPAND_FILES: c_int = 2;
    pub const EXPAND_DIRECTORIES: c_int = 3;
    pub const EXPAND_SETTINGS: c_int = 4;
    pub const EXPAND_BOOL_SETTINGS: c_int = 5;
    // EXPAND_TAGS = 6
    pub const EXPAND_OLD_SETTING: c_int = 7;
    // ... (8 through 35 skipped)
    pub const EXPAND_FILETYPE: c_int = 36;
    // EXPAND_FILES_IN_PATH = 37
    pub const EXPAND_OWNSYNTAX: c_int = 38;
    // ... (39 through 51 skipped)
    pub const EXPAND_STRING_SETTING: c_int = 52;
    pub const EXPAND_SETTING_SUBTRACT: c_int = 53;
    // EXPAND_ARGOPT = 54
    pub const EXPAND_KEYMAP: c_int = 55;
}

/// XP_PREFIX values (from cmdexpand_defs.h)
mod xp_prefix {
    use std::ffi::c_int;
    pub const XP_PREFIX_NO: c_int = 1;
    pub const XP_PREFIX_INV: c_int = 2;
}

/// XP_BS values (from cmdexpand_defs.h)
mod xp_bs {
    use std::ffi::c_int;
    pub const XP_BS_ONE: c_int = 0x1;
    pub const XP_BS_THREE: c_int = 0x2;
    pub const XP_BS_COMMA: c_int = 0x4;
}

/// kOptFlag* values (from option_defs.h)
mod opt_flag {
    pub const EXPAND: u32 = 1 << 0;
    pub const COMMA: u32 = 1 << 10;
    pub const COLON: u32 = 1 << 26;
    pub const FLAG_LIST: u32 = 1 << 13;
}

/// NUL character
const NUL: c_char = 0;

extern "C" {
    // expand_T field accessors
    fn nvim_xp_get_context(xp: *mut std::ffi::c_void) -> c_int;
    fn nvim_xp_set_context(xp: *mut std::ffi::c_void, val: c_int);
    fn nvim_xp_get_pattern(xp: *mut std::ffi::c_void) -> *mut c_char;
    fn nvim_xp_set_pattern(xp: *mut std::ffi::c_void, val: *mut c_char);
    fn nvim_xp_set_prefix(xp: *mut std::ffi::c_void, val: c_int);
    fn nvim_xp_get_line(xp: *mut std::ffi::c_void) -> *mut c_char;
    fn nvim_xp_get_backslash(xp: *mut std::ffi::c_void) -> c_int;
    fn nvim_xp_set_backslash(xp: *mut std::ffi::c_void, val: c_int);

    // expand_option static variable accessors
    fn nvim_get_expand_option_idx() -> OptIndex;
    fn nvim_set_expand_option_idx(val: OptIndex);
    fn nvim_set_expand_option_start_col(val: c_int);
    fn nvim_set_expand_option_flags(val: c_int);
    fn nvim_set_expand_option_append(val: c_int);
    fn nvim_set_expand_option_name_chars(c2: c_char, c3: c_char);

    // options[] array accessors
    fn nvim_option_has_expand_cb(opt_idx: OptIndex) -> c_int;
    fn nvim_opt_var_is_p_syn(opt_idx: OptIndex) -> c_int;
    fn nvim_opt_var_is_p_ft(opt_idx: OptIndex) -> c_int;
    fn nvim_opt_var_is_p_keymap(opt_idx: OptIndex) -> c_int;
    fn nvim_opt_var_is_p_sps(opt_idx: OptIndex) -> c_int;
    /// Returns: 1=dir+XP_BS_THREE, 2=dir+XP_BS_ONE, 3=file+XP_BS_THREE, 4=file+XP_BS_ONE
    fn nvim_opt_var_expand_type(opt_idx: OptIndex) -> c_int;

}

/// Check if a byte is alphanumeric or underscore.
#[inline]
fn is_alnum_or_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'*'
}

/// Rust implementation of `set_context_in_set_cmd`.
///
/// Sets up expand context for `:set` command completion by parsing the
/// argument string and determining expansion type.
///
/// # Safety
/// All pointer arguments must be valid.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_set_context_in_set_cmd(
    xp: *mut std::ffi::c_void,
    arg: *mut c_char,
    opt_flags: c_int,
) {
    nvim_set_expand_option_flags(opt_flags);

    nvim_xp_set_context(xp, expand_ctx::EXPAND_SETTINGS);
    if *arg == NUL {
        nvim_xp_set_pattern(xp, arg);
        return;
    }

    // Find end of arg string
    let mut len: usize = 0;
    while *arg.add(len) != NUL {
        len += 1;
    }
    let argend = arg.add(len);

    let mut p = argend.sub(1);
    if *p == b' ' as c_char && *p.sub(1) != b'\\' as c_char {
        nvim_xp_set_pattern(xp, p.add(1));
        return;
    }

    // Walk backwards to find start of current token (unescaped space)
    while p > arg {
        let s = {
            let mut s = p;
            if *p == b' ' as c_char || *p == b',' as c_char {
                while s > arg && *s.sub(1) == b'\\' as c_char {
                    s = s.sub(1);
                }
            }
            s
        };
        if *p == b' ' as c_char && ((p.offset_from(s)) & 1) == 0 {
            p = p.add(1);
            break;
        }
        p = p.sub(1);
    }

    // Check for "no" / "inv" prefix
    if *p == b'n' as c_char && *p.add(1) == b'o' as c_char {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_BOOL_SETTINGS);
        nvim_xp_set_prefix(xp, xp_prefix::XP_PREFIX_NO);
        p = p.add(2);
    } else if *p == b'i' as c_char && *p.add(1) == b'n' as c_char && *p.add(2) == b'v' as c_char {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_BOOL_SETTINGS);
        nvim_xp_set_prefix(xp, xp_prefix::XP_PREFIX_INV);
        p = p.add(3);
    }

    nvim_xp_set_pattern(xp, p);
    let arg = p;

    let mut flags: u32 = 0;
    let mut opt_idx: OptIndex = 0;
    let mut is_term_option = false;

    let nextchar: c_char;

    if *arg == b'<' as c_char {
        // Terminal key: <key>
        while *p != b'>' as c_char {
            if *p == NUL {
                return; // expand terminal option name
            }
            p = p.add(1);
        }
        let key = get_special_key_code(arg.add(1));
        if key == 0 {
            nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
            return;
        }
        p = p.add(1);
        nextchar = *p;
        is_term_option = true;
        // KEY2TERMCAP0(key) = (-(key)) & 0xff
        // KEY2TERMCAP1(key) = ((-(key) as u32) >> 8) & 0xff
        let neg_key = (-(key as i32)) as u32;
        let c2 = (neg_key & 0xff) as u8 as c_char;
        let c3 = ((neg_key >> 8) & 0xff) as u8 as c_char;
        nvim_set_expand_option_name_chars(c2, c3);
    } else if *p == b't' as c_char && *p.add(1) == b'_' as c_char {
        // t_ terminal option
        p = p.add(2);
        if *p != NUL {
            p = p.add(1);
        }
        if *p == NUL {
            return; // expand option name
        }
        p = p.add(1);
        nextchar = *p;
        is_term_option = true;
        let c2 = *p.sub(2);
        let c3 = *p.sub(1);
        nvim_set_expand_option_name_chars(c2, c3);
    } else {
        // Regular option name: walk over alphanumeric + '_' + '*'
        while is_alnum_or_ident(*p as u8) {
            p = p.add(1);
        }
        if *p == NUL {
            return;
        }
        nextchar = *p;
        let name_len = p.offset_from(arg) as usize;
        opt_idx = find_option_len(arg, name_len);
        if opt_idx == K_OPT_INVALID || is_option_hidden(opt_idx) != 0 {
            nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
            return;
        }
        flags = nvim_get_option_flags(opt_idx);
        if option_has_type(opt_idx, val_type::BOOLEAN) != 0 {
            nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
            return;
        }
    }

    // Handle "-=", "+=", "^="
    nvim_set_expand_option_append(0);
    let mut expand_option_subtract = false;
    let nextchar =
        if (nextchar == b'-' as c_char || nextchar == b'+' as c_char || nextchar == b'^' as c_char)
            && *p.add(1) == b'=' as c_char
        {
            if nextchar == b'-' as c_char {
                expand_option_subtract = true;
            }
            if nextchar == b'+' as c_char || nextchar == b'^' as c_char {
                nvim_set_expand_option_append(1);
            }
            p = p.add(1);
            b'=' as c_char
        } else {
            nextchar
        };

    if (nextchar != b'=' as c_char && nextchar != b':' as c_char)
        || nvim_xp_get_context(xp) == expand_ctx::EXPAND_BOOL_SETTINGS
    {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_UNSUCCESSFUL);
        return;
    }

    // Set expand_option_idx
    if is_term_option {
        nvim_set_expand_option_idx(K_OPT_INVALID);
    } else {
        nvim_set_expand_option_idx(opt_idx);
    }

    nvim_xp_set_pattern(xp, p.add(1));
    let xp_line = nvim_xp_get_line(xp);
    let col = p.add(1).offset_from(xp_line);
    nvim_set_expand_option_start_col(col as c_int);

    // Special-case options that reuse expansion logic from other commands
    if nvim_opt_var_is_p_syn(opt_idx) != 0 {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_OWNSYNTAX);
        return;
    }
    if nvim_opt_var_is_p_ft(opt_idx) != 0 {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_FILETYPE);
        return;
    }
    if nvim_opt_var_is_p_keymap(opt_idx) != 0 {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_KEYMAP);
        return;
    }

    // Determine expansion context
    let current_expand_idx = nvim_get_expand_option_idx();
    if expand_option_subtract {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_SETTING_SUBTRACT);
        return;
    } else if current_expand_idx != K_OPT_INVALID
        && nvim_option_has_expand_cb(current_expand_idx) != 0
    {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_STRING_SETTING);
    } else if *nvim_xp_get_pattern(xp) == NUL {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_OLD_SETTING);
        return;
    } else {
        nvim_xp_set_context(xp, expand_ctx::EXPAND_NOTHING);
    }

    if is_term_option || option_has_type(opt_idx, val_type::NUMBER) != 0 {
        return;
    }

    // Only string options below. Handle kOptFlagExpand options.
    if flags & opt_flag::EXPAND != 0 {
        let expand_type = nvim_opt_var_expand_type(opt_idx);
        match expand_type {
            1 => {
                // EXPAND_DIRECTORIES + XP_BS_THREE (p_path or p_cdpath)
                nvim_xp_set_context(xp, expand_ctx::EXPAND_DIRECTORIES);
                nvim_xp_set_backslash(xp, xp_bs::XP_BS_THREE);
            }
            2 => {
                // EXPAND_DIRECTORIES + XP_BS_ONE
                nvim_xp_set_context(xp, expand_ctx::EXPAND_DIRECTORIES);
                nvim_xp_set_backslash(xp, xp_bs::XP_BS_ONE);
            }
            3 => {
                // EXPAND_FILES + XP_BS_THREE (p_tags)
                nvim_xp_set_context(xp, expand_ctx::EXPAND_FILES);
                nvim_xp_set_backslash(xp, xp_bs::XP_BS_THREE);
            }
            _ => {
                // EXPAND_FILES + XP_BS_ONE
                nvim_xp_set_context(xp, expand_ctx::EXPAND_FILES);
                nvim_xp_set_backslash(xp, xp_bs::XP_BS_ONE);
            }
        }
        if flags & opt_flag::COMMA != 0 {
            nvim_xp_set_backslash(xp, nvim_xp_get_backslash(xp) | xp_bs::XP_BS_COMMA);
        }
    }

    // For comma/colon-separated or file-list options: find start of current pattern
    if flags & (opt_flag::EXPAND | opt_flag::COMMA | opt_flag::COLON) != 0 {
        let xp_pattern_start = nvim_xp_get_pattern(xp);
        let cur_backslash = nvim_xp_get_backslash(xp);
        let mut p2 = argend.sub(1);
        while p2 > xp_pattern_start {
            if *p2 == b' ' as c_char
                || *p2 == b',' as c_char
                || (*p2 == b':' as c_char && flags & opt_flag::COLON != 0)
            {
                let mut s = p2;
                while s > xp_pattern_start && *s.sub(1) == b'\\' as c_char {
                    s = s.sub(1);
                }
                let bs_count = p2.offset_from(s);
                let break_here = (*p2 == b' ' as c_char
                    && cur_backslash & xp_bs::XP_BS_THREE != 0
                    && bs_count < 3)
                    || (*p2 == b',' as c_char && flags & opt_flag::COMMA != 0 && bs_count < 2)
                    || (*p2 == b':' as c_char && flags & opt_flag::COLON != 0);
                if break_here {
                    nvim_xp_set_pattern(xp, p2.add(1));
                    break;
                }
            }
            p2 = p2.sub(1);
        }
    }

    // Flag-list options always start at end
    if flags & opt_flag::FLAG_LIST != 0 {
        nvim_xp_set_pattern(xp, argend);
    }

    // Special case for 'spellsuggest': "file:" prefix triggers file expansion
    if nvim_opt_var_is_p_sps(opt_idx) != 0 {
        let xp_pat = nvim_xp_get_pattern(xp);
        // Compare first 5 bytes with "file:"
        let file_prefix = b"file:";
        let matches_file = (0..5usize).all(|i| *xp_pat.add(i) == file_prefix[i] as c_char);
        if matches_file {
            nvim_xp_set_pattern(xp, xp_pat.add(5));
        } else if nvim_option_has_expand_cb(current_expand_idx) != 0 {
            nvim_xp_set_context(xp, expand_ctx::EXPAND_STRING_SETTING);
        }
    }
}
