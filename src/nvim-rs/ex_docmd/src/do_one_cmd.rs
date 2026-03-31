//! do_one_cmd: parse and execute a single Ex command.
//!
//! This module implements the core Ex command execution function,
//! ported from the C `do_one_cmd` in ex_docmd.c.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::ExArgHandle;

// =============================================================================
// Type aliases
// =============================================================================

pub type LineGetter = Option<
    unsafe extern "C" fn(
        c: c_int,
        cookie: *mut c_void,
        indent: c_int,
        do_concat: bool,
    ) -> *mut c_char,
>;
pub type CstackHandle = *mut c_void;
type LineGetterFn = unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char;

// =============================================================================
// FAIL constant
// =============================================================================

const FAIL: c_int = 0;

// =============================================================================
// DOCMD_* flags
// =============================================================================

const DOCMD_VERBOSE: c_int = 0x01;

// =============================================================================
// CSF constants
// =============================================================================

const CSF_ACTIVE: c_int = 0x0002;

// =============================================================================
// CMD_* constants needed by do_one_cmd
// =============================================================================

// These are loaded lazily via accessor functions to avoid hardcoding indices.

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Globals
    static mut did_emsg: c_int;
    static mut got_int: bool;

    // reg_executing / pending_end_reg_executing
    fn nvim_get_reg_executing() -> c_int;
    fn nvim_set_reg_executing(val: c_int);
    fn nvim_get_pending_end_reg_executing() -> c_int;
    fn nvim_set_pending_end_reg_executing(val: c_int);

    // quitmore
    fn nvim_docmd_get_quitmore() -> c_int;
    fn nvim_docmd_dec_quitmore();

    // did_throw
    static mut did_throw: bool;

    // exarg_T allocation

    // exarg_T field accessors

    // cmdlinep and char accessors

    // cstack field accessors
    fn nvim_cstack_get_idx(cs: CstackHandle) -> c_int;
    fn nvim_cstack_get_flags(cs: CstackHandle, idx: c_int) -> c_int;

    // cmdmod save/restore
    fn nvim_docmd_save_cmdmod() -> *mut c_void;
    fn nvim_docmd_restore_cmdmod(save: *mut c_void);

    // Command modifier application
    // nvim_docmd_parse_command_modifiers_global calls parse_command_modifiers(eap, errormsg, &cmdmod, false)
    fn nvim_docmd_parse_command_modifiers_global(
        eap: ExArgHandle,
        errormsg: *mut *const c_char,
    ) -> c_int;
    fn apply_cmdmod(cmod: *mut c_void);
    fn undo_cmdmod(cmod: *mut c_void);
    static mut cmdmod: u8;

    // find_excmd_after_range (called via C wrapper until Rust version is ready)
    fn nvim_find_excmd_after_range(eap: ExArgHandle) -> *mut c_char;

    // profile_cmd
    fn profile_cmd(
        eap: ExArgHandle,
        cstack: CstackHandle,
        fgetline: LineGetter,
        cookie: *mut c_void,
    );

    // Breakpoint
    fn dbg_check_breakpoint(eap: ExArgHandle);

    // do_intthrow
    fn do_intthrow(cstack: CstackHandle) -> bool;

    // Address/range
    fn set_cmd_addr_type(eap: ExArgHandle, p: *mut c_char);
    fn parse_cmd_address(eap: ExArgHandle, errormsg: *mut *const c_char, silent: bool) -> c_int;

    // String helpers
    fn nvim_skip_colon_white(p: *const c_char, skipleadingwhite: bool) -> *mut c_char;
    fn check_nextcmd(p: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn getargcmd(argp: *mut *mut c_char) -> *mut c_char;

    // Range without command
    fn ex_range_without_command(eap: ExArgHandle) -> *const c_char;

    // has_event / EVENT_CMDUNDEFINED
    fn has_event(event: c_int) -> bool;
    fn nvim_docmd_apply_autocmds_cmdundefined(cmdname: *const c_char) -> bool;
    fn aborting() -> bool;
    fn xmemdupz(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_docmd_ascii_isalnum(c: c_char) -> bool;
    fn xfree(p: *mut c_void);

    // CMD_SIZE constant
    fn nvim_docmd_get_command_count() -> c_int; // CMD_SIZE is command_count - 1

    // is_cmd_ni
    fn nvim_docmd_cmdnames_func_is_ni(cmdidx: c_int) -> c_int;

    // parse_bang
    fn parse_bang(eap: ExArgHandle, p_ptr: *mut *mut c_char) -> bool;

    // Security checks
    fn nvim_get_sandbox() -> c_int;
    fn nvim_docmd_curbuf_modifiable() -> bool;
    fn nvim_curbuf_is_terminal() -> c_int;
    static cmdwin_type: c_int;
    fn nvim_get_e_cmdwin() -> *const c_char;
    fn nvim_docmd_is_user_cmdidx_i(cmdidx: c_int) -> bool;
    fn curbuf_locked() -> c_int;

    // Range validation
    fn nvim_docmd_ask_yesno_backwards() -> c_char;
    fn nvim_docmd_invalid_range(eap: ExArgHandle) -> *const c_char;

    fn nvim_get_eap_addr_type_lines(eap: ExArgHandle) -> c_int;
    fn nvim_hasFolding_line1(lnum: i32, line1_out: *mut i32);
    fn nvim_hasFolding_line2(lnum: i32, line2_out: *mut i32);

    // makeprg replacement
    fn replace_makeprg(
        eap: ExArgHandle,
        arg: *mut c_char,
        cmdlinep: *mut *mut c_char,
    ) -> *mut c_char;

    // arg helpers

    // CMD_* constants now use crate::cmd_idx::{CMD_*} from build.rs-generated file.

    // +command

    // trlbar / nextcmd

    // dflall
    fn set_cmd_dflall_range(eap: ExArgHandle);

    // register / count / flags
    fn parse_register(eap: ExArgHandle);

    // trailing / needarg
    fn nvim_docmd_ex_errmsg_trailing(arg: *const c_char) -> *mut c_char;

    // skip_cmd / execute_cmd0

    // post-execute: rethrow/finish/return
    fn do_throw(cstack: CstackHandle);
    fn nvim_docmd_do_finish(eap: ExArgHandle);
    fn do_return(eap: ExArgHandle, reanimate: bool, is_cmd: bool, rettv: *mut c_void) -> bool;
    fn source_finished(fgetline: LineGetter, cookie: *mut c_void) -> bool;
    fn current_func_returned() -> c_int;
    fn get_func_line(c: c_int, cookie: *mut c_void, indent: c_int, do_concat: bool) -> *mut c_char;
    fn getnextac(c: c_int, cookie: *mut c_void, indent: c_int, do_concat: bool) -> *mut c_char;

    // doend: error message + do_errthrow
    fn nvim_docmd_do_one_cmd_doend(
        cstack: CstackHandle,
        errormsg: *const c_char,
        flags: c_int,
        eap: ExArgHandle,
    );

    // xstrlcpy / IObuff / append_command (for error formatting)
    static mut IObuff: [c_char; 1025];
    fn nvim_xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize);
    fn nvim_iosize() -> usize;

    // argt bit checks

    // skipwhite
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // find_ex_command (already in Rust but need for do_one_cmd retry)
    fn find_ex_command(eap: ExArgHandle, full: *mut c_int) -> *mut c_char;
}

// =============================================================================
// ex_errmsg: format error message with argument
// =============================================================================

/// Thread-local buffer for ex_errmsg.
///
/// Matches the C static buffer `ex_error_buf[MSG_BUF_LEN]`.
static mut EX_ERROR_BUF: [u8; 1024] = [0u8; 1024];

/// Format an error message with an argument.
///
/// Returns a pointer to a static buffer containing the formatted message.
/// The message format string `msg` is translated via `_()`.
///
/// # Safety
///
/// `msg` and `arg` must be valid null-terminated C strings.
#[export_name = "ex_errmsg"]
pub unsafe extern "C" fn rs_ex_errmsg(msg: *const c_char, arg: *const c_char) -> *mut c_char {
    // Use vim_snprintf equivalent: format msg with arg into the buffer.
    // We call C's vim_snprintf directly via a shim.
    let buf = ptr::addr_of_mut!(EX_ERROR_BUF).cast::<c_char>();
    nvim_docmd_ex_errmsg_format(msg, arg, buf, 1024)
}

extern "C" {
    /// C-side format helper: vim_snprintf(buf, len, _(msg), arg) -> buf.
    fn nvim_docmd_ex_errmsg_format(
        msg: *const c_char,
        arg: *const c_char,
        buf: *mut c_char,
        buflen: usize,
    ) -> *mut c_char;
}

// =============================================================================
// excmd_get_argt: get command argt flags
// =============================================================================

/// Get the `cmd_argt` flags for a command index.
///
/// # Safety
///
/// `idx` must be a valid `cmdidx_T` value (not CMD_SIZE or negative for user cmds).
#[export_name = "excmd_get_argt"]
pub unsafe extern "C" fn rs_excmd_get_argt(idx: c_int) -> u32 {
    nvim_docmd_get_argt_for_idx(idx)
}

extern "C" {
    fn nvim_docmd_get_argt_for_idx(idx: c_int) -> u32;
}

// =============================================================================
// find_excmd_after_range: find command after range specifiers
// =============================================================================

/// Find the command name after skipping range specifiers.
///
/// Saves and restores `eap->cmd` while skipping the range to find
/// the command name.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "find_excmd_after_range"]
pub unsafe extern "C" fn rs_find_excmd_after_range(eap: ExArgHandle) -> *mut c_char {
    nvim_find_excmd_after_range(eap)
}

// =============================================================================
// do_one_cmd: parse and execute a single Ex command
// =============================================================================

/// Parse and execute a single Ex command.
///
/// This is the core function that:
/// 1. Skips comments and leading whitespace
/// 2. Handles command modifiers
/// 3. Skips over the range to find the command
/// 4. Parses the range
/// 5. Parses the command
/// 6. Parses arguments
/// 7. Executes the command
///
/// # Safety
///
/// All pointers must be valid for their types.
#[no_mangle]
pub unsafe extern "C" fn do_one_cmd(
    cmdlinep: *mut *mut c_char,
    flags: c_int,
    cstack: CstackHandle,
    fgetline: LineGetter,
    cookie: *mut c_void,
) -> *mut c_char {
    let mut errormsg: *const c_char = ptr::null();

    // Save register execution state.
    let save_reg_executing = nvim_get_reg_executing();
    let save_pending_end_reg_executing = nvim_get_pending_end_reg_executing();

    // Allocate exarg_T on the heap (line1=1, line2=1).
    let eap = crate::ExArg::alloc();

    crate::ex_nesting_level += 1;

    // When the last file has not been edited :q has to be typed twice.
    if nvim_docmd_get_quitmore() != 0
        // avoid function call in 'statusline'
        && !crate::do_cmdline::getline_equal(fgetline, cookie, Some(get_func_line as LineGetterFn))
        // avoid autocommand (e.g. QuitPre)
        && !crate::do_cmdline::getline_equal(fgetline, cookie, Some(getnextac as LineGetterFn))
    {
        nvim_docmd_dec_quitmore();
    }

    // Save cmdmod -- will be restored on return.
    let save_cmdmod = nvim_docmd_save_cmdmod();

    // Initialize ea fields first (needed before any accessor calls).
    (*eap).cmdlinep = cmdlinep;
    (*eap).cmd = (*eap).cmdlinep.read();
    (*eap).ea_getline = fgetline;
    (*eap).cookie = cookie;
    (*eap).cstack = cstack;

    // "#!anything" is handled like a comment.
    if (*(*eap).cmdlinep).read() == b'#' as c_char
        && (*(*eap).cmdlinep).add(1).read() == b'!' as c_char
    {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // 1-2. Parse command modifiers using the global cmdmod.
    if nvim_docmd_parse_command_modifiers_global(eap, &mut errormsg) == FAIL {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }
    apply_cmdmod(std::ptr::addr_of_mut!(cmdmod).cast());

    let after_modifier = (*eap).cmd;

    // Set skip based on error/interrupt/throw/cstack state.
    let cs_idx = nvim_cstack_get_idx(cstack);
    let skip = did_emsg != 0
        || got_int
        || did_throw
        || (cs_idx >= 0 && (nvim_cstack_get_flags(cstack, cs_idx) & CSF_ACTIVE) == 0);
    (*eap).skip = skip as c_int;
    // 3. Skip over the range to find the command.
    let mut p = nvim_find_excmd_after_range(eap);
    profile_cmd(eap, cstack, fgetline, cookie);

    if !crate::exiting {
        dbg_check_breakpoint(eap);
    }
    if !(*eap).skip != 0 && got_int {
        (*eap).skip = (true) as c_int;
        do_intthrow(cstack);
    }

    // 4. Parse range.
    set_cmd_addr_type(eap, p);
    if parse_cmd_address(eap, &mut errormsg, false) == FAIL {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // 5. Parse the command.
    let cmd = nvim_skip_colon_white((*eap).cmd, true);
    (*eap).cmd = cmd;
    // If no command, go to the line.
    {
        let cmd_ptr = (*eap).cmd;
        let first = *cmd_ptr as u8;
        if first == 0 || first == b'"' {
            if !(*eap).skip != 0 {
                errormsg = ex_range_without_command(eap);
            }
            do_one_cmd_doend(
                eap,
                cstack,
                errormsg,
                flags,
                save_cmdmod,
                save_reg_executing,
                save_pending_end_reg_executing,
                cmdlinep,
            );
            return ea_cleanup_and_return(eap, None);
        }
        // Check nextcmd
        let next = check_nextcmd(cmd_ptr);
        if !next.is_null() {
            (*eap).nextcmd = next;
            if !(*eap).skip != 0 {
                errormsg = ex_range_without_command(eap);
            }
            do_one_cmd_doend(
                eap,
                cstack,
                errormsg,
                flags,
                save_cmdmod,
                save_reg_executing,
                save_pending_end_reg_executing,
                cmdlinep,
            );
            return ea_cleanup_and_return(eap, None);
        }
    }

    // If looks like undefined user command, trigger CmdUndefined autocmds.
    let cmd_size = nvim_docmd_get_command_count() - 1; // CMD_SIZE
    if !p.is_null()
        && (*eap).cmdidx == cmd_size
        && !(*eap).skip != 0
        && {
            let c = *(*eap).cmd as u8;
            c.is_ascii_uppercase()
        }
        && has_event(29)
    {
        // Build cmdname as copy up to first non-alnum.
        let mut cmdname_end = (*eap).cmd;
        while nvim_docmd_ascii_isalnum(*cmdname_end) {
            cmdname_end = cmdname_end.add(1);
        }
        let cmdname_len = cmdname_end.offset_from((*eap).cmd) as usize;
        let cmdname = xmemdupz((*eap).cmd, cmdname_len);
        let ret = nvim_docmd_apply_autocmds_cmdundefined(cmdname);
        xfree(cmdname as *mut c_void);
        // Retry if autocommand succeeded and didn't abort.
        p = if ret && !aborting() {
            find_ex_command(eap, ptr::null_mut())
        } else {
            (*eap).cmd
        };
    }

    if p.is_null() {
        if !(*eap).skip != 0 {
            errormsg = crate::errors::gt(
                crate::errors::E_AMBIGUOUS_USE_OF_USER_DEFINED_COMMAND_STR.as_ptr(),
            );
        }
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // Check for wrong commands.
    if (*eap).cmdidx == cmd_size {
        if !(*eap).skip != 0 {
            let iobuff = std::ptr::addr_of_mut!(IObuff).cast::<c_char>();
            nvim_xstrlcpy(
                iobuff,
                crate::errors::gt(crate::errors::E_NOT_AN_EDITOR_COMMAND_STR.as_ptr()),
                nvim_iosize(),
            );
            let cmdname = if !after_modifier.is_null() {
                after_modifier
            } else {
                *cmdlinep
            };
            if (flags & DOCMD_VERBOSE) == 0 {
                crate::errors::rs_append_command(cmdname);
            }
            errormsg = iobuff as *const c_char;
            crate::did_emsg_syntax = true;
            crate::commands::rs_verify_command(cmdname);
        }
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // Set when Not Implemented.
    let ni = nvim_docmd_cmdnames_func_is_ni((*eap).cmdidx) != 0;

    // Determine if command has forceit flag.
    let mut p_mut = p;
    let forceit = parse_bang(eap, &mut p_mut);
    p = p_mut;
    (*eap).forceit = forceit as c_int;
    // 6. Parse arguments. Then check for errors.
    if !nvim_docmd_is_user_cmdidx_i((*eap).cmdidx) {
        let argt = nvim_docmd_get_argt_for_idx((*eap).cmdidx);
        (*eap).argt = argt;
    }

    if !(*eap).skip != 0 {
        if nvim_get_sandbox() != 0 && !((*eap).argt & 0x40000u32) != 0 {
            errormsg = crate::gt(crate::E_SANDBOX_STR.as_ptr());
            do_one_cmd_doend(
                eap,
                cstack,
                errormsg,
                flags,
                save_cmdmod,
                save_reg_executing,
                save_pending_end_reg_executing,
                cmdlinep,
            );
            return ea_cleanup_and_return(eap, None);
        }
        if !nvim_docmd_curbuf_modifiable()
            && ((*eap).argt & 0x100000u32) != 0
            && !(nvim_curbuf_is_terminal() != 0
                && ((*eap).cmdidx == crate::cmd_idx::CMD_put
                    || (*eap).cmdidx == crate::cmd_idx::CMD_iput))
        {
            errormsg = crate::errors::gt(crate::errors::E_MODIFIABLE_STR.as_ptr());
            do_one_cmd_doend(
                eap,
                cstack,
                errormsg,
                flags,
                save_cmdmod,
                save_reg_executing,
                save_pending_end_reg_executing,
                cmdlinep,
            );
            return ea_cleanup_and_return(eap, None);
        }

        if !nvim_docmd_is_user_cmdidx_i((*eap).cmdidx) {
            if cmdwin_type != 0 && !((*eap).argt & 0x40000u32) != 0 {
                // Use EX_CMDWIN check via argt
                if ((*eap).argt & 0x80000) == 0 {
                    // EX_CMDWIN = 0x80000
                    errormsg = nvim_get_e_cmdwin();
                    do_one_cmd_doend(
                        eap,
                        cstack,
                        errormsg,
                        flags,
                        save_cmdmod,
                        save_reg_executing,
                        save_pending_end_reg_executing,
                        cmdlinep,
                    );
                    return ea_cleanup_and_return(eap, None);
                }
            }
            if crate::rs_text_locked() != 0 && ((*eap).argt & 0x1000000) == 0 {
                // EX_LOCK_OK = 0x1000000
                errormsg = crate::rs_get_text_locked_msg();
                do_one_cmd_doend(
                    eap,
                    cstack,
                    errormsg,
                    flags,
                    save_cmdmod,
                    save_reg_executing,
                    save_pending_end_reg_executing,
                    cmdlinep,
                );
                return ea_cleanup_and_return(eap, None);
            }
        }

        // Disallow editing another buffer when curbuf_locked.
        if ((*eap).argt & 0x80000) == 0  // not EX_CMDWIN
            && (*eap).cmdidx != crate::cmd_idx::CMD_checktime
            && (*eap).cmdidx != crate::cmd_idx::CMD_edit
            && (*eap).cmdidx != crate::cmd_idx::CMD_file
            && !nvim_docmd_is_user_cmdidx_i((*eap).cmdidx)
            && curbuf_locked() != 0
        {
            do_one_cmd_doend(
                eap,
                cstack,
                errormsg,
                flags,
                save_cmdmod,
                save_reg_executing,
                save_pending_end_reg_executing,
                cmdlinep,
            );
            return ea_cleanup_and_return(eap, None);
        }

        if !ni && !((*eap).argt & 0x001u32) != 0 && (*eap).addr_count > 0 {
            errormsg = crate::gt(crate::E_NORANGE_STR.as_ptr());
            do_one_cmd_doend(
                eap,
                cstack,
                errormsg,
                flags,
                save_cmdmod,
                save_reg_executing,
                save_pending_end_reg_executing,
                cmdlinep,
            );
            return ea_cleanup_and_return(eap, None);
        }
    }

    if !ni && !((*eap).argt & 0x002u32) != 0 && (*eap).forceit != 0 {
        errormsg = crate::errors::gt(crate::errors::E_NOBANG_STR.as_ptr());
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // Don't complain about range if not used.
    if !(*eap).skip != 0 && !ni && ((*eap).argt & 0x001u32) != 0 {
        if crate::global_busy == 0 && (*eap).line1 > (*eap).line2 {
            if crate::msg_silent == 0 {
                if (flags & DOCMD_VERBOSE) != 0 || crate::exmode_active {
                    errormsg = crate::gt(crate::E_BACKWARDS_RANGE_STR.as_ptr());
                    do_one_cmd_doend(
                        eap,
                        cstack,
                        errormsg,
                        flags,
                        save_cmdmod,
                        save_reg_executing,
                        save_pending_end_reg_executing,
                        cmdlinep,
                    );
                    return ea_cleanup_and_return(eap, None);
                }
                if nvim_docmd_ask_yesno_backwards() != b'y' as c_char {
                    do_one_cmd_doend(
                        eap,
                        cstack,
                        errormsg,
                        flags,
                        save_cmdmod,
                        save_reg_executing,
                        save_pending_end_reg_executing,
                        cmdlinep,
                    );
                    return ea_cleanup_and_return(eap, None);
                }
            }
            // Swap line1 and line2.
            std::mem::swap(&mut (*eap).line1, &mut (*eap).line2);
        }
        let inv_err = nvim_docmd_invalid_range(eap);
        if !inv_err.is_null() {
            errormsg = inv_err;
            do_one_cmd_doend(
                eap,
                cstack,
                errormsg,
                flags,
                save_cmdmod,
                save_reg_executing,
                save_pending_end_reg_executing,
                cmdlinep,
            );
            return ea_cleanup_and_return(eap, None);
        }
    }

    if (*eap).addr_type == crate::address::ADDR_OTHER && (*eap).addr_count == 0 {
        (*eap).line2 = 1;
    }

    crate::range::rs_correct_range(eap);

    if (((*eap).argt & 0x040u32) != 0 || (*eap).addr_count >= 2)
        && crate::global_busy == 0
        && nvim_get_eap_addr_type_lines(eap) != 0
    {
        let mut line1 = (*eap).line1;
        let mut line2 = (*eap).line2;
        nvim_hasFolding_line1(line1, &mut line1);
        nvim_hasFolding_line2(line2, &mut line2);
        (*eap).line1 = line1;
        (*eap).line2 = line2;
    }

    // Replace makeprg/grepprg.
    let cmdlinep_val = (*eap).cmdlinep;
    p = replace_makeprg(eap, p, cmdlinep_val);
    if p.is_null() {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // Skip to start of argument.
    let arg = if (*eap).cmdidx == crate::cmd_idx::CMD_bang {
        p
    } else {
        skipwhite(p)
    };
    (*eap).arg = arg;
    // ":file" cannot be run with an argument when curbuf_locked.
    if (*eap).cmdidx == crate::cmd_idx::CMD_file && *(*eap).arg != 0 && curbuf_locked() != 0 {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // Check for "++opt=val" argument.
    if ((*eap).argt & 0x20000u32) != 0 {
        let mut arg_ptr = (*eap).arg;
        while *arg_ptr == b'+' as c_char && *arg_ptr.add(1) == b'+' as c_char {
            if crate::args::rs_getargopt(eap) == FAIL && !ni {
                errormsg = crate::gt(crate::E_INVARG_STR.as_ptr());
                do_one_cmd_doend(
                    eap,
                    cstack,
                    errormsg,
                    flags,
                    save_cmdmod,
                    save_reg_executing,
                    save_pending_end_reg_executing,
                    cmdlinep,
                );
                return ea_cleanup_and_return(eap, None);
            }
            arg_ptr = (*eap).arg;
        }
    }

    // Handle write/update, read, lshift/rshift special cases.
    let cmdidx = (*eap).cmdidx;
    if cmdidx == crate::cmd_idx::CMD_write || cmdidx == crate::cmd_idx::CMD_update {
        let arg0 = (*eap).arg.read();
        if arg0 == b'>' as c_char {
            let arg1 = *(*eap).arg.add(1);
            if arg1 != b'>' as c_char {
                errormsg = crate::gt(crate::E_W_USAGE_STR.as_ptr());
                do_one_cmd_doend(
                    eap,
                    cstack,
                    errormsg,
                    flags,
                    save_cmdmod,
                    save_reg_executing,
                    save_pending_end_reg_executing,
                    cmdlinep,
                );
                return ea_cleanup_and_return(eap, None);
            }
            (*eap).arg = (*eap).arg.add(2);
            (*eap).arg = skipwhite((*eap).arg);
            (*eap).append = (true) as c_int;
        } else if arg0 == b'!' as c_char && cmdidx == crate::cmd_idx::CMD_write {
            (*eap).arg = (*eap).arg.add(1);
            (*eap).usefilter = (true) as c_int;
        }
    } else if cmdidx == crate::cmd_idx::CMD_read {
        if (*eap).forceit != 0 {
            (*eap).usefilter = (true) as c_int;
            (*eap).forceit = (false) as c_int;
        } else if (*eap).arg.read() == b'!' as c_char {
            (*eap).arg = (*eap).arg.add(1);
            (*eap).usefilter = (true) as c_int;
        }
    } else if cmdidx == crate::cmd_idx::CMD_lshift || cmdidx == crate::cmd_idx::CMD_rshift {
        (*eap).amount = 1;
        let cmd_char = (*eap).cmd.read();
        let mut arg_ptr = (*eap).arg;
        while *arg_ptr == cmd_char {
            arg_ptr = arg_ptr.add(1);
            (*eap).arg = arg_ptr;
            (*eap).amount += 1;
        }
        (*eap).arg = skipwhite((*eap).arg);
    }

    // Check for "+command" argument.
    if ((*eap).argt & 0x4000u32) != 0 && ((*eap).usefilter == 0) {
        (*eap).do_ecmd_cmd = getargcmd(&mut (*eap).arg);
    }

    // Check for '|' separator.
    if ((*eap).argt & 0x100u32) != 0 && ((*eap).usefilter == 0) {
        crate::args::rs_separate_nextcmd(eap);
    } else if cmdidx == crate::cmd_idx::CMD_bang
        || cmdidx == crate::cmd_idx::CMD_terminal
        || cmdidx == crate::cmd_idx::CMD_global
        || cmdidx == crate::cmd_idx::CMD_vglobal
        || (*eap).usefilter != 0
    {
        // Scan arg for newline: set nextcmd, NUL-terminate, handle backslash-newline.
        let mut s = (*eap).arg;
        while !s.is_null() && *s != 0 {
            if *s == b'\\' as c_char && *s.add(1) == b'\n' as c_char {
                memmove(
                    s as *mut c_void,
                    s.add(1) as *const c_void,
                    strlen(s.add(1)) + 1,
                );
            } else if *s == b'\n' as c_char {
                (*eap).nextcmd = s.add(1);
                *s = 0;
                break;
            }
            s = s.add(1);
        }
    }

    if ((*eap).argt & 0x020u32) != 0 && (*eap).addr_count == 0 {
        set_cmd_dflall_range(eap);
    }

    // Parse register and count.
    parse_register(eap);
    if crate::args::rs_parse_count_ex(eap, &mut errormsg, 1) == FAIL {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // Check for flags.
    if ((*eap).argt & 0x200000u32) != 0 {
        crate::execute::rs_get_flags(eap);
    }

    // Check for trailing arguments.
    let arg_char = *(*eap).arg as u8;
    if !ni
        && !((*eap).argt & 0x004u32) != 0
        && arg_char != 0
        && arg_char != b'"'
        && !(arg_char == b'|' && ((*eap).argt & 0x100u32) != 0)
    {
        errormsg = nvim_docmd_ex_errmsg_trailing((*eap).arg) as *const c_char;
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    if !ni && ((*eap).argt & 0x080u32) != 0 && arg_char == 0 {
        errormsg = crate::gt(crate::E_ARGREQ_STR.as_ptr());
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    if crate::commands::rs_skip_cmd(eap) != 0 {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // 7. Execute the command.
    let mut retv: c_int = 0;
    if crate::execute::rs_execute_cmd0(&mut retv, eap, &mut errormsg, false) == FAIL {
        do_one_cmd_doend(
            eap,
            cstack,
            errormsg,
            flags,
            save_cmdmod,
            save_reg_executing,
            save_pending_end_reg_executing,
            cmdlinep,
        );
        return ea_cleanup_and_return(eap, None);
    }

    // Post-execute: rethrow/return/finish.
    if crate::need_rethrow {
        do_throw(cstack);
    } else if crate::check_cstack {
        if source_finished(fgetline, cookie) {
            nvim_docmd_do_finish(eap);
        } else if crate::do_cmdline::getline_equal(
            fgetline,
            cookie,
            Some(get_func_line as LineGetterFn),
        ) && current_func_returned() != 0
        {
            do_return(eap, true, false, std::ptr::null_mut());
        }
    }
    crate::need_rethrow = false;
    crate::check_cstack = false;

    // doend: cleanup and return.
    let nextcmd = (*eap).nextcmd;
    do_one_cmd_doend(
        eap,
        cstack,
        errormsg,
        flags,
        save_cmdmod,
        save_reg_executing,
        save_pending_end_reg_executing,
        cmdlinep,
    );
    ea_cleanup_and_return(eap, Some(nextcmd))
}

// CMD_* constants use crate::cmd_idx::{CMD_*} from build.rs-generated file.

/// Perform the doend cleanup in do_one_cmd.
/// Calls nvim_docmd_do_one_cmd_doend and then restores cmdmod and register state.
#[inline]
#[allow(clippy::too_many_arguments)]
unsafe fn do_one_cmd_doend(
    eap: ExArgHandle,
    cstack: CstackHandle,
    errormsg: *const c_char,
    flags: c_int,
    save_cmdmod: *mut c_void,
    save_reg_executing: c_int,
    save_pending_end_reg_executing: c_int,
    _cmdlinep: *mut *mut c_char,
) {
    // Fix cursor lnum if zero (can happen with zero line number).
    nvim_docmd_fix_cursor_if_zero();

    // Emit error message and do_errthrow.
    nvim_docmd_do_one_cmd_doend(cstack, errormsg, flags, eap);

    // Undo and restore cmdmod.
    undo_cmdmod(std::ptr::addr_of_mut!(cmdmod).cast());
    nvim_docmd_restore_cmdmod(save_cmdmod);

    // Restore register execution state.
    nvim_set_reg_executing(save_reg_executing);
    nvim_set_pending_end_reg_executing(save_pending_end_reg_executing);
}

extern "C" {
    fn nvim_docmd_fix_cursor_if_zero();
}

/// Clean up eap and return the next command pointer.
#[inline]
unsafe fn ea_cleanup_and_return(eap: ExArgHandle, nextcmd: Option<*mut c_char>) -> *mut c_char {
    // Fix nextcmd if it's an empty string.
    let result = if let Some(nc) = nextcmd {
        if !nc.is_null() && *nc == 0 {
            ptr::null_mut()
        } else {
            nc
        }
    } else {
        let nc = (*eap).nextcmd;
        if !nc.is_null() && *nc == 0 {
            ptr::null_mut()
        } else {
            nc
        }
    };

    crate::ex_nesting_level -= 1;

    // Free cmdline_tofree.
    let tofree = (*eap).cmdline_tofree;
    if !tofree.is_null() {
        xfree(tofree as *mut c_void);
    }

    xfree(eap as *mut c_void);
    result
}
