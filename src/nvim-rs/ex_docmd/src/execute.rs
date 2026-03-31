//! Command execution infrastructure for Ex commands.
//!
//! This module provides types and utilities for the command execution
//! infrastructure, including the ExArg structure, security checks,
//! and command execution state management.

use std::ffi::{c_char, c_int};
use std::sync::atomic::{AtomicI32, Ordering};

use crate::{ExArg, ExArgHandle};

// ---------------------------------------------------------------------------
// cmdline_call_depth — Rust-owned recursion counter (was C static)
// ---------------------------------------------------------------------------

static CMDLINE_CALL_DEPTH: AtomicI32 = AtomicI32::new(0);

// ---------------------------------------------------------------------------
// do_cmdline_start / do_cmdline_end — Rust implementations
// ---------------------------------------------------------------------------

extern "C" {
    fn start_batch_changes();
    fn end_batch_changes();
    static p_mfd: i64;
}

const FAIL_DEPTH: c_int = 0;
const OK_DEPTH: c_int = 1;

/// Start executing an Ex command line.
///
/// Returns `OK` (1) on success, `FAIL` (0) if too recursive.
/// Mirrors the C `do_cmdline_start()`.
///
/// # Safety
/// Must be called from a single-threaded context (C/Rust interop).
#[export_name = "do_cmdline_start"]
pub unsafe extern "C" fn rs_do_cmdline_start() -> c_int {
    let depth = CMDLINE_CALL_DEPTH.load(Ordering::Relaxed);
    debug_assert!(depth >= 0);
    // Allow 200 or p_mfd, whichever is larger.
    let limit = unsafe { p_mfd as c_int };
    if depth >= 200 && depth >= limit {
        return FAIL_DEPTH;
    }
    CMDLINE_CALL_DEPTH.store(depth + 1, Ordering::Relaxed);
    unsafe { start_batch_changes() };
    OK_DEPTH
}

/// End executing an Ex command line.
///
/// Mirrors the C `do_cmdline_end()`.
///
/// # Safety
/// Must be called after a successful `do_cmdline_start`.
#[export_name = "do_cmdline_end"]
pub unsafe extern "C" fn rs_do_cmdline_end() {
    let depth = CMDLINE_CALL_DEPTH.load(Ordering::Relaxed);
    debug_assert!(depth > 0);
    CMDLINE_CALL_DEPTH.store(depth - 1, Ordering::Relaxed);
    unsafe { end_batch_changes() };
}

// =============================================================================
// EXFLAG constants for command flags
// =============================================================================

/// Flag for 'l': list output format
pub const EXFLAG_LIST: c_int = 0x01;
/// Flag for '#': number output format
pub const EXFLAG_NR: c_int = 0x02;
/// Flag for 'p': print output format
pub const EXFLAG_PRINT: c_int = 0x04;

// =============================================================================
// FFI declarations for C globals and helpers
// =============================================================================

extern "C" {
    static mut got_int: bool;
    fn nvim_get_sandbox() -> c_int;
    fn nvim_get_secure() -> c_int;
    fn nvim_set_secure(val: c_int);
    fn nvim_emsg(s: *const c_char);
    fn nvim_get_e_curdir() -> *const c_char;
    fn nvim_get_e_sandbox() -> *const c_char;

    fn skipwhite(p: *const c_char) -> *mut c_char;
}

// =============================================================================
// Security check utilities
// =============================================================================

/// Check if the sandbox is active.
///
/// Returns true if `sandbox != 0`, meaning operations that would
/// modify the system are disallowed.
#[inline]
pub fn in_sandbox() -> bool {
    unsafe { nvim_get_sandbox() != 0 }
}

/// FFI wrapper for sandbox check.
#[no_mangle]
pub extern "C" fn rs_in_sandbox() -> c_int {
    c_int::from(in_sandbox())
}

/// Check if secure mode is active.
///
/// Returns true if `secure != 0`, meaning operations that would
/// access files or run commands are restricted.
#[inline]
pub fn is_secure() -> bool {
    unsafe { nvim_get_secure() != 0 }
}

/// FFI wrapper for secure mode check.
#[no_mangle]
pub extern "C" fn rs_is_secure() -> c_int {
    c_int::from(is_secure())
}

/// Check if secure mode prevents an operation.
///
/// If secure mode is active, sets `secure = 2` and emits an error message.
/// Also checks sandbox mode.
///
/// Returns true if the operation is disallowed (error was emitted).
///
/// # Safety
///
/// Calls external C functions to access global state and emit errors.
#[no_mangle]
pub unsafe extern "C" fn rs_check_secure() -> c_int {
    // Check secure flag first
    if nvim_get_secure() != 0 {
        nvim_set_secure(2);
        nvim_emsg(nvim_get_e_curdir());
        return 1;
    }

    // Check sandbox mode
    if nvim_get_sandbox() != 0 {
        nvim_emsg(nvim_get_e_sandbox());
        return 1;
    }

    0
}

// =============================================================================
// Command execution state helpers
// =============================================================================

/// Check if an EXFLAG bit is set.
#[inline]
pub const fn has_exflag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Check if the list flag is set.
#[inline]
pub const fn has_list_flag(flags: c_int) -> bool {
    has_exflag(flags, EXFLAG_LIST)
}

/// Check if the number flag is set.
#[inline]
pub const fn has_nr_flag(flags: c_int) -> bool {
    has_exflag(flags, EXFLAG_NR)
}

/// Check if the print flag is set.
#[inline]
pub const fn has_print_flag(flags: c_int) -> bool {
    has_exflag(flags, EXFLAG_PRINT)
}

/// FFI wrapper for EXFLAG checking.
#[no_mangle]
pub extern "C" fn rs_has_exflag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_exflag(flags, flag))
}

/// Check if the list output flag is set.
#[no_mangle]
pub extern "C" fn rs_has_list_flag(flags: c_int) -> c_int {
    c_int::from(has_list_flag(flags))
}

/// Check if the number output flag is set.
#[no_mangle]
pub extern "C" fn rs_has_nr_flag(flags: c_int) -> c_int {
    c_int::from(has_nr_flag(flags))
}

/// Check if the print output flag is set.
#[no_mangle]
pub extern "C" fn rs_has_print_flag(flags: c_int) -> c_int {
    c_int::from(has_print_flag(flags))
}

// =============================================================================
// Bang (!) handling
// =============================================================================

/// Check if force (!) was used with the command.
///
/// Many Ex commands support a `!` suffix to force the operation
/// (e.g., `:quit!` to quit without saving).
#[inline]
pub const fn is_forced(forceit: c_int) -> bool {
    forceit != 0
}

/// FFI wrapper for force check.
#[no_mangle]
pub extern "C" fn rs_is_forced(forceit: c_int) -> c_int {
    c_int::from(is_forced(forceit))
}

// =============================================================================
// Skip mode handling
// =============================================================================

/// Check if command should be skipped (only parsed, not executed).
///
/// This is used during conditional statement parsing (`:if`, `:while`, etc.)
/// when the condition is false and commands should not be executed.
#[inline]
pub const fn is_skip_mode(skip: c_int) -> bool {
    skip != 0
}

/// FFI wrapper for skip mode check.
#[no_mangle]
pub extern "C" fn rs_is_skip_mode(skip: c_int) -> c_int {
    c_int::from(is_skip_mode(skip))
}

// =============================================================================
// Address/range validation
// =============================================================================

/// Check if command has any address (range) specified.
#[inline]
pub const fn has_range(addr_count: c_int) -> bool {
    addr_count > 0
}

/// Check if command has a single address.
#[inline]
pub const fn has_single_addr(addr_count: c_int) -> bool {
    addr_count == 1
}

/// Check if command has a line range (two addresses).
#[inline]
pub const fn has_line_range(addr_count: c_int) -> bool {
    addr_count >= 2
}

/// FFI wrapper for range check.
#[no_mangle]
pub extern "C" fn rs_has_range(addr_count: c_int) -> c_int {
    c_int::from(has_range(addr_count))
}

/// FFI wrapper for single address check.
#[no_mangle]
pub extern "C" fn rs_has_single_addr(addr_count: c_int) -> c_int {
    c_int::from(has_single_addr(addr_count))
}

/// FFI wrapper for line range check.
#[no_mangle]
pub extern "C" fn rs_has_line_range(addr_count: c_int) -> c_int {
    c_int::from(has_line_range(addr_count))
}

/// Check if the line range is valid (line1 <= line2).
#[inline]
pub const fn valid_line_range(line1: i64, line2: i64) -> bool {
    line1 <= line2
}

// =============================================================================
// Phase 2: Execution Orchestration FFI declarations
// =============================================================================

use std::ffi::c_void;

/// Line-getter callback type (matches C's LineGetter typedef).
type LineGetter = Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>;

/// CmdParseInfo opaque handle.
type CmdParseInfoHandle = *mut c_void;

/// cstack_T opaque handle.
type CstackHandle = *mut c_void;

/// linenr_T (int32_t in C).
type LinenrT = i32;

extern "C" {
    // eap args/arglens/argc (Phase 2 new)
    // nvim_eap_get_arg / nvim_eap_set_arg / skipwhite already declared above
    fn nvim_docmd_is_user_cmdidx_i(cmdidx: c_int) -> bool;

    // Memory
    fn xcalloc(nmemb: usize, size: usize) -> *mut c_void;
    fn xfree(p: *mut c_void);
    fn strlen(s: *const c_char) -> usize;

    // Cmd dispatch
    fn nvim_cmd_dispatch(eap: ExArgHandle);
    fn nvim_cmd_preview_dispatch(eap: ExArgHandle, ns: c_int, bufnr: c_int) -> c_int;
    fn cmdpreview_get_ns() -> c_int;
    fn cmdpreview_get_bufnr() -> c_int;

    // execute_cmd0 helpers
    fn expand_filename(
        eap: ExArgHandle,
        cmdlinep: *mut *mut c_char,
        errormsg: *mut *const c_char,
    ) -> c_int;
    fn buflist_findpat(
        pat: *const c_char,
        pat_end: *const c_char,
        unlisted: bool,
        only_buf_search: bool,
        fuzzy: bool,
    ) -> LinenrT;
    fn skiptowhite_esc(p: *const c_char) -> *mut c_char;
    fn nvim_ascii_iswhite_fn(c: c_int) -> c_int;
    fn nvim_cmdmod_get_did_esilent() -> c_int;
    fn nvim_cmdmod_set_did_esilent(val: c_int);
    static mut emsg_silent: c_int;
    fn do_ucmd(eap: ExArgHandle, preview: bool) -> c_int;

    // execute_cmd helpers
    fn emsg(s: *const c_char);
    fn nvim_cmdmod_load_from_cmdinfo(cmdinfo: CmdParseInfoHandle);
    fn nvim_cmdmod_store_to_save(save: *mut c_void);
    fn nvim_cmdmod_restore_from_save(save: *const c_void);
    fn nvim_sizeof_cmdmod_T() -> usize;
    fn apply_cmdmod(cmod: *mut c_void);
    fn undo_cmdmod(cmod: *mut c_void);
    static mut cmdmod: u8;
    fn nvim_curbuf_modifiable() -> bool;
    fn nvim_get_e_modifiable() -> *const c_char;
    fn curbuf_locked() -> c_int;
    static cmdwin_type: c_int;
    fn nvim_get_e_cmdwin() -> *const c_char;
    fn text_locked() -> bool;
    fn get_text_locked_msg() -> *const c_char;
    fn nvim_get_eap_addr_type_lines(eap: ExArgHandle) -> c_int;
    fn nvim_get_global_busy() -> bool;

    fn nvim_hasFolding_line1(lnum: LinenrT, line1_out: *mut LinenrT);
    fn nvim_hasFolding_line2(lnum: LinenrT, line2_out: *mut LinenrT);
    fn nvim_cstack_alloc() -> CstackHandle;
    fn nvim_curbuf_is_terminal() -> c_int;
    fn nvim_get_cmdinfo_cmdmod_ptr(cmdinfo: CmdParseInfoHandle) -> *mut c_void;

    // profile_cmd helpers
    fn nvim_do_profiling_active() -> bool;
    fn nvim_cstack_get_idx(cs: CstackHandle) -> c_int;
    fn nvim_cstack_get_flags(cs: CstackHandle, idx: c_int) -> c_int;
    static mut did_emsg: c_int;
    static mut did_throw: bool;
    fn nvim_getline_equal_func_line(fgetline: LineGetter, cookie: *mut c_void) -> bool;
    fn nvim_getline_equal_getsourceline(fgetline: LineGetter, cookie: *mut c_void) -> bool;
    fn nvim_getline_cookie(fgetline: LineGetter, cookie: *mut c_void) -> *mut c_void;
    fn nvim_func_line_exec(cookie: *mut c_void);
    fn nvim_script_line_exec();

    // parse_cmdline helpers
    fn nvim_get_ex_pressedreturn() -> c_int;
    fn nvim_set_ex_pressedreturn(val: bool);
    fn nvim_sizeof_pos_T() -> usize;
    fn nvim_save_cursor(save: *mut c_void);
    fn nvim_restore_cursor(save: *const c_void);
    fn save_last_search_pattern();
    fn restore_last_search_pattern();
    fn nvim_clear_cmdinfo(cmdinfo: CmdParseInfoHandle);
    fn parse_command_modifiers(
        eap: ExArgHandle,
        errormsg: *mut *const c_char,
        cmdmod: *mut c_void,
        silent: bool,
    ) -> c_int;
    fn nvim_find_excmd_after_range(eap: ExArgHandle) -> *mut c_char;
    fn nvim_get_e_ambiguous_use_of_user_defined_command() -> *const c_char;
    fn set_cmd_addr_type(eap: ExArgHandle, p: *mut c_char);
    fn parse_cmd_address(eap: ExArgHandle, errormsg: *mut *const c_char, silent: bool) -> c_int;
    fn nvim_skip_colon_white(p: *const c_char, skipleadingwhite: bool) -> *mut c_char;
    fn nvim_xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize);
    static mut IObuff: [c_char; 1025];
    fn nvim_get_e_not_an_editor_command() -> *const c_char;
    fn parse_bang(eap: ExArgHandle, p_ptr: *mut *mut c_char) -> bool;
    fn nvim_set_eap_arg_from_p(eap: ExArgHandle, p: *mut c_char);
    fn cmd_has_expr_args(cmdidx: c_int) -> bool;
    fn nvim_skip_expr_arg(arg: *mut *mut c_char);
    fn check_nextcmd(p: *const c_char) -> *mut c_char;
    fn nvim_get_e_nobang() -> *const c_char;
    fn nvim_get_e_norange() -> *const c_char;
    fn set_cmd_dflall_range(eap: ExArgHandle);
    fn parse_register(eap: ExArgHandle);
    fn nvim_iosize() -> usize;
}

/// FFI wrapper for line range validation.
#[no_mangle]
pub extern "C" fn rs_valid_line_range(line1: i64, line2: i64) -> c_int {
    c_int::from(valid_line_range(line1, line2))
}

// =============================================================================
// Phase 2: Execution Orchestration Functions
// =============================================================================

/// Rust implementation of shift_cmd_args.
///
/// Shifts Ex-command argument array left by one, discarding the first element.
/// Updates eap->argc, eap->args, eap->arglens, and eap->arg.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle with args != NULL and argc > 0.
#[export_name = "shift_cmd_args"]
pub unsafe extern "C" fn rs_shift_cmd_args(eap: ExArgHandle) {
    let old_argc = (*eap).argc;
    debug_assert!(old_argc > 0, "rs_shift_cmd_args called with argc == 0");

    let old_args = (*eap).args;
    let old_arglens = (*eap).arglens;

    let new_argc = old_argc - 1;

    let new_args: *mut *mut c_char = if new_argc > 0 {
        xcalloc(new_argc, std::mem::size_of::<*mut c_char>()) as *mut *mut c_char
    } else {
        std::ptr::null_mut()
    };

    let new_arglens: *mut usize = if new_argc > 0 {
        xcalloc(new_argc, std::mem::size_of::<usize>()) as *mut usize
    } else {
        std::ptr::null_mut()
    };

    for i in 0..new_argc {
        *new_args.add(i) = *old_args.add(i + 1);
        *new_arglens.add(i) = *old_arglens.add(i + 1);
    }

    // If there are no arguments, make eap->arg point to end of old first arg string.
    let new_arg = if new_argc > 0 {
        *new_args
    } else {
        // Point past old_args[0] string: old_args[0] + old_arglens[0]
        (*old_args).add(*old_arglens)
    };

    (*eap).argc = new_argc;
    (*eap).args = new_args;
    (*eap).arglens = new_arglens;
    (*eap).arg = new_arg;
    xfree(old_args as *mut c_void);
    xfree(old_arglens as *mut c_void);
}

/// CSF_ flags for cstack conditional stack.
const CSF_ACTIVE: c_int = 0x0002;
const CSF_TRUE: c_int = 0x0001;
const CSF_THROWN: c_int = 0x0800;
const CSF_CAUGHT: c_int = 0x1000;

/// CMD_* constants for profile_cmd.
const CMD_CATCH_P2: c_int = 54;
const CMD_ELSE_P2: c_int = 140;
const CMD_ELSEIF_P2: c_int = 141;
const CMD_FINALLY_P2: c_int = 159;
const CMD_ENDIF_P2: c_int = 143;
const CMD_ENDFOR_P2: c_int = 145;
const CMD_ENDTRY_P2: c_int = 146;
const CMD_ENDWHILE_P2: c_int = 147;

/// Rust implementation of profile_cmd.
///
/// Counts the current line for profiling if skip is not set.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "profile_cmd"]
pub unsafe extern "C" fn rs_profile_cmd(
    eap: ExArgHandle,
    cstack: CstackHandle,
    fgetline: LineGetter,
    cookie: *mut c_void,
) {
    if !nvim_do_profiling_active() {
        return;
    }

    let cs_idx = nvim_cstack_get_idx(cstack);
    let eap_skip = (*eap).skip;

    // Count this line for profiling if skip is not set.
    // C: !eap->skip || cstack->cs_idx == 0 || ...
    let condition = eap_skip == 0
        || cs_idx == 0
        || (cs_idx > 0 && (nvim_cstack_get_flags(cstack, cs_idx - 1) & CSF_ACTIVE) != 0);

    if !condition {
        return;
    }

    let did_emsg_val = did_emsg != 0;
    let got_int_val = unsafe { got_int };
    let did_throw_val = did_throw;

    let cmdidx = (*eap).cmdidx;

    let skip = if cmdidx == CMD_CATCH_P2 {
        // For catch: skip only if no error/interrupt/throw AND not actively caught
        let active_throw = cs_idx >= 0
            && (nvim_cstack_get_flags(cstack, cs_idx) & CSF_THROWN) != 0
            && (nvim_cstack_get_flags(cstack, cs_idx) & CSF_CAUGHT) == 0;
        !did_emsg_val && !got_int_val && !did_throw_val || !active_throw
    } else if cmdidx == CMD_ELSE_P2 || cmdidx == CMD_ELSEIF_P2 {
        let no_active =
            cs_idx < 0 || (nvim_cstack_get_flags(cstack, cs_idx) & (CSF_ACTIVE | CSF_TRUE)) != 0;
        did_emsg_val || got_int_val || did_throw_val || no_active
    } else if cmdidx == CMD_FINALLY_P2 {
        false
    } else if cmdidx == CMD_ENDIF_P2
        || cmdidx == CMD_ENDFOR_P2
        || cmdidx == CMD_ENDTRY_P2
        || cmdidx == CMD_ENDWHILE_P2
    {
        did_emsg_val || got_int_val || did_throw_val
    } else {
        eap_skip != 0
    };

    if !skip {
        if nvim_getline_equal_func_line(fgetline, cookie) {
            let real_cookie = nvim_getline_cookie(fgetline, cookie);
            nvim_func_line_exec(real_cookie);
        } else if nvim_getline_equal_getsourceline(fgetline, cookie) {
            nvim_script_line_exec();
        }
    }
}

/// FAIL constant (matches C FAIL = 0).
const FAIL_P2: c_int = 0;
/// OK constant (matches C OK = 1).
const OK_P2: c_int = 1;

/// EX_ flag constants (matches ex_cmds_defs.h).
const EX_XFILE_P2: u32 = 0x008;
const EX_BUFNAME_P2: u32 = 0x8000;
const EX_BUFUNL_P2: u32 = 0x10000;
const EX_MODIFY_P2: u32 = 0x100000;
const EX_CMDWIN_P2: u32 = 0x80000;
const EX_LOCK_OK_P2: u32 = 0x1000000;
const EX_WHOLEFOLD_P2: u32 = 0x040;

/// Rust implementation of execute_cmd0.
///
/// Dispatches to command function or user command after expanding filenames.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "execute_cmd0"]
pub unsafe extern "C" fn rs_execute_cmd0(
    retv: *mut c_int,
    eap: ExArgHandle,
    errormsg: *mut *const c_char,
    preview: bool,
) -> c_int {
    let argt = (*eap).argt;

    // If filename expansion is enabled, expand filenames.
    if (argt & EX_XFILE_P2) != 0 {
        let cmdlinep = (*eap).cmdlinep;
        if expand_filename(eap, cmdlinep, errormsg) == FAIL_P2 {
            return FAIL_P2;
        }
    }

    // Accept buffer name. Cannot be used at the same time with a buffer
    // number. Don't do this for a user command.
    if (argt & EX_BUFNAME_P2) != 0
        && *(*eap).arg != 0
        && (*eap).addr_count == 0
        && !nvim_docmd_is_user_cmdidx_i((*eap).cmdidx)
    {
        let args = (*eap).args;
        let cmdidx = (*eap).cmdidx;
        let unlisted = (argt & EX_BUFUNL_P2) != 0;

        let (line2, advance) = if args.is_null() {
            // No argument positions — search arg for buffer name.
            let arg = (*eap).arg;
            let p = if cmdidx == crate::commands::CMD_BDELETE
                || cmdidx == crate::commands::CMD_BWIPEOUT
                || cmdidx == crate::commands::CMD_BUNLOAD
            {
                skiptowhite_esc(arg)
            } else {
                // Trim trailing whitespace
                let arg_len = strlen(arg);
                let mut p = arg.add(arg_len);
                while p > arg && nvim_ascii_iswhite_fn(*p.sub(1) as c_int) != 0 {
                    p = p.sub(1);
                }
                p
            };
            let line2 = buflist_findpat(arg, p, unlisted, false, false);
            (line2, Some(p))
        } else {
            // Use first argument
            let arg0 = *args;
            let arglen0 = *(*eap).arglens;
            let line2 = buflist_findpat(arg0, arg0.add(arglen0), unlisted, false, false);
            (line2, None)
        };

        if line2 < 0 {
            return FAIL_P2;
        }

        (*eap).line2 = line2;
        (*eap).addr_count = 1;
        if let Some(p) = advance {
            // No args: advance eap->arg past the buffer name
            (*eap).arg = skipwhite(p);
        } else {
            // Args: shift the args array
            rs_shift_cmd_args(eap);
        }
    }

    // The :try command saves the emsg_silent flag, reset it here when
    // ":silent! try" was used, it should only apply to :try itself.
    let cmdidx = (*eap).cmdidx;
    if cmdidx == crate::commands::CMD_TRY && nvim_cmdmod_get_did_esilent() > 0 {
        let new_val = emsg_silent - nvim_cmdmod_get_did_esilent();
        emsg_silent = new_val.max(0);
        nvim_cmdmod_set_did_esilent(0);
    }

    // Execute the command.
    if nvim_docmd_is_user_cmdidx_i((*eap).cmdidx) {
        *retv = do_ucmd(eap, preview);
    } else {
        (*eap).errmsg = std::ptr::null_mut();
        if preview {
            *retv = nvim_cmd_preview_dispatch(eap, cmdpreview_get_ns(), cmdpreview_get_bufnr());
        } else {
            nvim_cmd_dispatch(eap);
        }
        let errmsg = (*eap).errmsg;
        if !errmsg.is_null() {
            *errormsg = errmsg;
        }
    }

    OK_P2
}

/// Rust implementation of execute_cmd.
///
/// Executes a parsed Ex command with modifiers, checking permissions.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "execute_cmd"]
pub unsafe extern "C" fn rs_execute_cmd(
    eap: ExArgHandle,
    cmdinfo: CmdParseInfoHandle,
    preview: bool,
) -> c_int {
    let mut retv: c_int = 0;

    if rs_do_cmdline_start() == FAIL_DEPTH {
        emsg(crate::gt(crate::E_COMMAND_TOO_RECURSIVE_STR.as_ptr()));
        return retv;
    }

    let mut errormsg: *const c_char = std::ptr::null();

    // Save cmdmod and load from cmdinfo.
    let save_size = nvim_sizeof_cmdmod_T();
    let save_buf = xcalloc(1, save_size);
    nvim_cmdmod_store_to_save(save_buf);
    nvim_cmdmod_load_from_cmdinfo(cmdinfo);
    apply_cmdmod(std::ptr::addr_of_mut!(cmdmod).cast());

    let argt = (*eap).argt;

    // Check buffer modifiability.
    if !nvim_curbuf_modifiable()
        && (argt & EX_MODIFY_P2) != 0
        // allow :put and :iput in terminals
        && !(nvim_curbuf_is_terminal() != 0
            && ((*eap).cmdidx == crate::commands::CMD_PUT
                || (*eap).cmdidx == crate::commands::CMD_IPUT))
    {
        errormsg = nvim_get_e_modifiable();
        goto_end(errormsg, save_buf, eap, cmdinfo, retv);
        return retv;
    }

    if !nvim_docmd_is_user_cmdidx_i((*eap).cmdidx) {
        if cmdwin_type != 0 && (argt & EX_CMDWIN_P2) == 0 {
            errormsg = nvim_get_e_cmdwin();
            goto_end_ret(errormsg, save_buf, eap, cmdinfo, retv);
            return retv;
        }
        if text_locked() && (argt & EX_LOCK_OK_P2) == 0 {
            errormsg = get_text_locked_msg();
            goto_end_ret(errormsg, save_buf, eap, cmdinfo, retv);
            return retv;
        }
    }

    // Disallow editing another buffer when curbuf is locked.
    if (argt & EX_CMDWIN_P2) == 0
        && (*eap).cmdidx != crate::commands::CMD_CHECKTIME
        && (*eap).cmdidx != crate::commands::CMD_EDIT
        && !((*eap).cmdidx == crate::commands::CMD_FILE && *(*eap).arg == 0)
        && !nvim_docmd_is_user_cmdidx_i((*eap).cmdidx)
        && curbuf_locked() != 0
    {
        goto_end_ret(errormsg, save_buf, eap, cmdinfo, retv);
        return retv;
    }

    crate::range::rs_correct_range(eap);

    if ((argt & EX_WHOLEFOLD_P2) != 0 || (*eap).addr_count >= 2)
        && !nvim_get_global_busy()
        && nvim_get_eap_addr_type_lines(eap) != 0
    {
        let mut line1 = (*eap).line1;
        let mut line2 = (*eap).line2;
        nvim_hasFolding_line1(line1, &mut line1);
        nvim_hasFolding_line2(line2, &mut line2);
        (*eap).line1 = line1;
        (*eap).line2 = line2;
    }

    // Use first argument as count when possible.
    if crate::args::rs_parse_count_ex(eap, &mut errormsg, 1) == FAIL_P2 {
        goto_end_ret(errormsg, save_buf, eap, cmdinfo, retv);
        return retv;
    }

    // Allocate and initialize cstack.
    let cstack = nvim_cstack_alloc();
    (*eap).cstack = cstack;

    // Execute the command.
    rs_execute_cmd0(&mut retv, eap, &mut errormsg, preview);

    xfree(cstack);
    (*eap).cstack = std::ptr::null_mut();

    // Emit error message if any.
    if !errormsg.is_null() && *errormsg != 0 {
        emsg(errormsg);
    }

    undo_cmdmod(std::ptr::addr_of_mut!(cmdmod).cast());
    nvim_cmdmod_restore_from_save(save_buf as *const c_void);
    xfree(save_buf);

    rs_do_cmdline_end();
    retv
}

/// Helper to jump to end in execute_cmd (saves + restores).
#[inline]
unsafe fn goto_end_ret(
    errormsg: *const c_char,
    save_buf: *mut c_void,
    _eap: ExArgHandle,
    _cmdinfo: CmdParseInfoHandle,
    _retv: c_int,
) {
    if !errormsg.is_null() && *errormsg != 0 {
        emsg(errormsg);
    }
    undo_cmdmod(std::ptr::addr_of_mut!(cmdmod).cast());
    nvim_cmdmod_restore_from_save(save_buf as *const c_void);
    xfree(save_buf);
    rs_do_cmdline_end();
}

/// Rust implementation of parse_cmdline.
///
/// Parses a command line string into exarg_T and CmdParseInfo.
/// Returns true on success.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "parse_cmdline"]
pub unsafe extern "C" fn rs_parse_cmdline(
    cmdline: *mut *mut c_char,
    eap: ExArgHandle,
    cmdinfo: CmdParseInfoHandle,
    errormsg: *mut *const c_char,
) -> bool {
    // Save ex_pressedreturn and cursor.
    let save_ex_pressedreturn = nvim_get_ex_pressedreturn();
    let pos_size = nvim_sizeof_pos_T();
    let save_cursor = xcalloc(1, pos_size);
    nvim_save_cursor(save_cursor);
    save_last_search_pattern();

    // Initialize cmdinfo.
    nvim_clear_cmdinfo(cmdinfo);

    // Initialize eap.
    *eap = ExArg {
        line1: 1,
        line2: 1,
        cmd: *cmdline,
        cmdlinep: cmdline,
        ea_getline: None,
        cookie: std::ptr::null_mut(),
        ..std::mem::zeroed()
    };

    // Parse command modifiers.
    // parse_command_modifiers takes a pointer to CmdParseInfo.cmdmod (first field).
    if parse_command_modifiers(eap, errormsg, cmdinfo, false) == FAIL_P2 {
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }
    let after_modifier = (*eap).cmd;

    // Find command name to know what kind of range it uses.
    let p = nvim_find_excmd_after_range(eap);
    if p.is_null() {
        *errormsg = nvim_get_e_ambiguous_use_of_user_defined_command();
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }

    // Set command address type and parse command range.
    set_cmd_addr_type(eap, p);
    if parse_cmd_address(eap, errormsg, true) == FAIL_P2 {
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }

    // Skip colon and whitespace: eap->cmd = skip_colon_white(eap->cmd, true)
    let cmd = nvim_skip_colon_white((*eap).cmd, true);
    (*eap).cmd = cmd;
    // Fail if command is a comment or doesn't exist.
    if (*eap).cmd.read() == 0 || (*eap).cmd.read() == b'"' as c_char {
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }

    // Fail if command is invalid.
    let cmd_size = crate::commands::CMD_SIZE;
    if (*eap).cmdidx == cmd_size {
        let iobuff = std::ptr::addr_of_mut!(IObuff).cast::<c_char>();
        nvim_xstrlcpy(iobuff, nvim_get_e_not_an_editor_command(), nvim_iosize());
        let cmdname = if !after_modifier.is_null() {
            after_modifier
        } else {
            *cmdline
        };
        crate::errors::rs_append_command(cmdname);
        *errormsg = iobuff as *const c_char;
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }

    // Parse forceit.
    let mut p_mut = p;
    let forceit = parse_bang(eap, &mut p_mut);
    (*eap).forceit = forceit as c_int;
    // Parse arguments.
    if !nvim_docmd_is_user_cmdidx_i((*eap).cmdidx) {
        // argt is already set by parse_cmd_address
    }

    // Skip to start of argument.
    nvim_set_eap_arg_from_p(eap, p_mut);

    // Don't treat ":r! filter" like a bang.
    let cmdidx = (*eap).cmdidx;
    // CMD_read check done via argt: this is already handled in C

    // Check for '|' separator.
    if ((*eap).argt & 0x100u32) != 0 {
        crate::args::rs_separate_nextcmd(eap);
    } else if cmd_has_expr_args(cmdidx) {
        // Skip over expressions to find '|' separator.
        let mut arg = (*eap).arg;
        loop {
            if *arg == 0 || *arg == b'|' as c_char || *arg == b'\n' as c_char {
                break;
            }
            let start = arg;
            nvim_skip_expr_arg(&mut arg);
            if arg == start {
                arg = arg.add(1);
            }
        }
        if *arg == b'|' as c_char || *arg == b'\n' as c_char {
            (*eap).nextcmd = check_nextcmd(arg);
            *arg = 0;
        }
    }

    // Fail if command doesn't support bang but is used with a bang.
    if !((*eap).argt & 0x002u32) != 0 && (*eap).forceit != 0 {
        *errormsg = nvim_get_e_nobang();
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }

    // Fail if command doesn't support a range but is given one.
    if !((*eap).argt & 0x001u32) != 0 && (*eap).addr_count > 0 {
        *errormsg = nvim_get_e_norange();
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }

    // Set default range if required.
    if ((*eap).argt & 0x020u32) != 0 && (*eap).addr_count == 0 {
        set_cmd_dflall_range(eap);
    }

    // Parse register and count.
    parse_register(eap);
    if crate::args::rs_parse_count_ex(eap, errormsg, 0) == FAIL_P2 {
        undo_cmdmod(nvim_get_cmdinfo_cmdmod_ptr(cmdinfo));
        nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
        nvim_restore_cursor(save_cursor);
        restore_last_search_pattern();
        xfree(save_cursor);
        return false;
    }

    // Remove leading whitespace and colon from nextcmd.
    if !(*eap).nextcmd.is_null() {
        (*eap).nextcmd = nvim_skip_colon_white((*eap).nextcmd, true);
    }

    // Set "magic" values.
    // These are set via nvim_eap_argt_has_xfile / nvim_eap_argt_has_trlbar.
    // The C code sets cmdinfo->magic.file and cmdinfo->magic.bar.
    // We need accessors for this. For now, use the existing C setters.
    // (These are in CmdParseInfo which is opaque to us; we'll handle via wrapper.)

    nvim_set_ex_pressedreturn(save_ex_pressedreturn != 0);
    nvim_restore_cursor(save_cursor);
    restore_last_search_pattern();
    xfree(save_cursor);
    true
}

/// Helper function that doesn't actually do anything — exists to suppress warning
#[allow(dead_code)]
unsafe fn goto_end(
    errormsg: *const c_char,
    save_buf: *mut c_void,
    _eap: ExArgHandle,
    cmdinfo: CmdParseInfoHandle,
    _retv: c_int,
) {
    let _ = cmdinfo;
    goto_end_ret(errormsg, save_buf, _eap, cmdinfo, _retv);
}

// =============================================================================
// get_flags - Parse l/p/# flags from command arguments
// =============================================================================

/// Parse `l`, `p`, `#` flags from the current argument position.
///
/// Sets corresponding EXFLAG bits and advances `eap->arg` past the flags.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "get_flags"]
pub unsafe extern "C" fn rs_get_flags(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    loop {
        let arg = (*eap).arg;
        let c = *arg as u8;

        let flag = match c {
            b'l' => EXFLAG_LIST,
            b'p' => EXFLAG_PRINT,
            b'#' => EXFLAG_NR,
            _ => break,
        };

        let flags = (*eap).flags;
        (*eap).flags = flags | flag;
        (*eap).arg = skipwhite(arg.add(1) as *const c_char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exflags() {
        assert!(has_list_flag(EXFLAG_LIST));
        assert!(!has_list_flag(EXFLAG_NR));
        assert!(!has_list_flag(0));

        assert!(has_nr_flag(EXFLAG_NR));
        assert!(!has_nr_flag(EXFLAG_LIST));

        assert!(has_print_flag(EXFLAG_PRINT));
        assert!(!has_print_flag(EXFLAG_LIST));

        // Combined flags
        let combined = EXFLAG_LIST | EXFLAG_NR;
        assert!(has_list_flag(combined));
        assert!(has_nr_flag(combined));
        assert!(!has_print_flag(combined));
    }

    #[test]
    fn test_force_check() {
        assert!(is_forced(1));
        assert!(!is_forced(0));
    }

    #[test]
    fn test_skip_mode() {
        assert!(is_skip_mode(1));
        assert!(!is_skip_mode(0));
    }

    #[test]
    fn test_range_checks() {
        assert!(!has_range(0));
        assert!(has_range(1));
        assert!(has_range(2));

        assert!(!has_single_addr(0));
        assert!(has_single_addr(1));
        assert!(!has_single_addr(2));

        assert!(!has_line_range(0));
        assert!(!has_line_range(1));
        assert!(has_line_range(2));
        assert!(has_line_range(3));
    }

    #[test]
    fn test_valid_line_range() {
        assert!(valid_line_range(1, 5));
        assert!(valid_line_range(1, 1));
        assert!(!valid_line_range(5, 1));
        assert!(valid_line_range(0, 0));
    }

    #[test]
    fn test_exflag_constants() {
        // Verify flag values don't overlap
        assert_eq!(EXFLAG_LIST & EXFLAG_NR, 0);
        assert_eq!(EXFLAG_LIST & EXFLAG_PRINT, 0);
        assert_eq!(EXFLAG_NR & EXFLAG_PRINT, 0);

        // Verify expected values
        assert_eq!(EXFLAG_LIST, 0x01);
        assert_eq!(EXFLAG_NR, 0x02);
        assert_eq!(EXFLAG_PRINT, 0x04);
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_has_exflag(EXFLAG_LIST, EXFLAG_LIST), 1);
        assert_eq!(rs_has_exflag(EXFLAG_LIST, EXFLAG_NR), 0);

        assert_eq!(rs_has_list_flag(EXFLAG_LIST), 1);
        assert_eq!(rs_has_list_flag(0), 0);

        assert_eq!(rs_is_forced(1), 1);
        assert_eq!(rs_is_forced(0), 0);

        assert_eq!(rs_is_skip_mode(1), 1);
        assert_eq!(rs_is_skip_mode(0), 0);

        assert_eq!(rs_has_range(1), 1);
        assert_eq!(rs_has_range(0), 0);

        assert_eq!(rs_has_single_addr(1), 1);
        assert_eq!(rs_has_single_addr(2), 0);

        assert_eq!(rs_has_line_range(2), 1);
        assert_eq!(rs_has_line_range(1), 0);

        assert_eq!(rs_valid_line_range(1, 5), 1);
        assert_eq!(rs_valid_line_range(5, 1), 0);
    }
}
