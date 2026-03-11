//! Exception handling evaluation state for Neovim
//!
//! This module provides Rust implementations for checking exception handling
//! state during command execution.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(dead_code)] // Phase 3 internal functions used in Phase 4

use std::ffi::{c_char, c_int, c_void};

// Re-export types used across modules
pub type OptInt = i64;
pub type LinenrT = i32;

/// Enum matching except_type_T in ex_eval_defs.h
#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ExceptTypeT {
    EtUser = 0,
    EtError = 1,
    EtInterrupt = 2,
}

/// Representation of except_T matching C layout (sizeof=56).
#[repr(C)]
pub struct ExceptT {
    pub type_: ExceptTypeT,
    pub value: *mut c_char,
    pub messages: *mut MsglistT,
    pub throw_name: *mut c_char,
    pub throw_lnum: LinenrT,
    pub stacktrace: *mut c_void, // list_T * (opaque)
    pub caught: *mut ExceptT,
}

/// Representation of cstack_T matching C layout (sizeof=1288).
#[repr(C)]
pub struct CstackT {
    pub cs_flags: [c_int; 50],
    pub cs_pending: [c_char; 50],
    _padding_pend: [u8; 6],
    pub cs_pend: CstackPend,
    pub cs_forinfo: [*mut c_void; 50],
    pub cs_line: [c_int; 50],
    pub cs_idx: c_int,
    pub cs_looplevel: c_int,
    pub cs_trylevel: c_int,
    _padding_try: [u8; 4],
    pub cs_emsg_silent_list: *mut c_void, // eslist_T *
    pub cs_lflags: c_int,
}

/// Union cs_pend in cstack_T
#[repr(C)]
pub union CstackPend {
    pub csp_rv: [*mut c_void; 50],
    pub csp_ex: [*mut c_void; 50],
}

// Direct access to C globals for exception state variables
extern "C" {
    static mut force_abort: bool;
    static mut did_emsg: c_int;
    static mut got_int: bool;
    static mut did_throw: bool;
    static mut trylevel: c_int;
    static mut emsg_silent: c_int;
    static mut current_exception: *mut c_void;
    static mut need_rethrow: bool;
    static mut msg_list: *mut *mut c_void;
    static mut p_verbose: OptInt;
    static mut debug_break_level: c_int;
    static mut suppress_errthrow: bool;
    static mut caught_stack: *mut c_void; // except_T *
    static mut msg_silent: c_int;
    static mut msg_scroll: c_int;
    static mut no_wait_return: c_int;
    static mut cmdline_row: c_int;
    static mut msg_row: c_int;
    static mut p_vfile: *mut c_char;
    static mut IObuff: [c_char; 1025];
}

// Phase 4 additional C functions
extern "C" {
    fn cleanup_conditionals(cstack: *mut CstackT, searched_cond: c_int, inclusive: c_int) -> c_int;
    fn get_return_cmd(rettv: *mut c_void) -> *mut c_char;
    fn concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;
    fn gettext(s: *const c_char) -> *const c_char;
}

// C functions callable from Rust
extern "C" {
    fn handle_did_throw();
    fn modifier_len(p: *mut c_char) -> c_int;
    fn tv_free(tv: *mut c_void);
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn semsg(fmt: *const c_char, ...);
    fn emsg(s: *const c_char) -> bool;
    fn smsg(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    fn msg_puts(s: *const c_char);
    fn internal_error(s: *const c_char);
    fn verbose_enter();
    fn verbose_leave();
    fn estack_sfile(which: c_int) -> *mut c_char; // estack_arg_T is c_int
    fn stacktrace_create() -> *mut c_void; // returns list_T *
    fn tv_list_unref(list: *mut c_void);
    fn set_vim_var_string(idx: c_int, val: *const c_char, len: c_int);
    fn set_vim_var_list(idx: c_int, val: *mut c_void);
    fn vim_snprintf(str: *mut c_char, str_m: usize, fmt: *const c_char, ...) -> c_int;
    fn free_for_info(fi: *mut c_void);
    fn nvim_get_sourcing_lnum_direct() -> LinenrT;
    fn nvim_tv_list_ref(list: *mut c_void);
}

// Error strings from errors.h / globals
extern "C" {
    static e_str_not_inside_function: *const c_char;
    static e_outofmem: *const c_char;
    static e_interr: *const c_char;
}

// VimVarIndex constants matching eval_defs.h VimVarIndex enum
// Counted from the enum definition:
// VV_COUNT=0..VV_DYING=29, VV_EXCEPTION=30, VV_THROWPOINT=31, ...VV_STACKTRACE=125
const VV_EXCEPTION: c_int = 30;
const VV_THROWPOINT: c_int = 31;
const VV_STACKTRACE: c_int = 125;

// CSF_ flag constants matching ex_eval_defs.h
const CSF_FOR: c_int = 0x0010;
const CSF_TRY: c_int = 0x0100;
const CSF_THROWN: c_int = 0x0800;
const CSF_CAUGHT: c_int = 0x1000;
const CSF_ACTIVE: c_int = 0x0002;

// CSTP_ pending type constants matching ex_eval_defs.h
const CSTP_NONE: c_int = 0;
const CSTP_ERROR: c_int = 1;
const CSTP_INTERRUPT: c_int = 2;
const CSTP_THROW: c_int = 4;
const CSTP_BREAK: c_int = 8;
const CSTP_CONTINUE: c_int = 16;
const CSTP_RETURN: c_int = 24;
const CSTP_FINISH: c_int = 32;

// RP_ constants for report_pending
const RP_MAKE: c_int = 0;
const RP_RESUME: c_int = 1;
const RP_DISCARD: c_int = 2;

// ESTACK_NONE constant from runtime_defs.h
const ESTACK_NONE: c_int = 0;

// OK constant from vim_defs.h
const OK: c_int = 1;

// Rust-owned static replacing the C file-local `static bool cause_abort`
static mut CAUSE_ABORT: bool = false;

/// FAIL constant from vim_defs.h
const FAIL: c_int = 0;

/// Returns true if a function with the "abort" flag should not be considered
/// ended on an error. Parsing commands is continued in order to find finally
/// clauses to be executed, and some errors in skipped commands are still reported.
#[export_name = "aborted_in_try"]
pub unsafe extern "C" fn aborted_in_try_impl() -> bool {
    force_abort
}

/// Returns true when immediately aborting on error, or when an interrupt
/// occurred or an exception was thrown but not caught.
///
/// Use for ":{range}call" to check whether an aborted function that does not
/// handle a range itself should be called again for the next line in the range.
#[export_name = "aborting"]
pub unsafe extern "C" fn aborting_impl() -> bool {
    (did_emsg != 0 && force_abort) || got_int || did_throw
}

/// Returns true if a command with a subcommand resulting in `retcode` should
/// abort the script processing.
#[export_name = "should_abort"]
pub unsafe extern "C" fn should_abort_impl(retcode: c_int) -> bool {
    (retcode == FAIL && trylevel != 0 && emsg_silent == 0) || aborting_impl()
}

/// Updates `force_abort` if `cause_abort` is set.
///
/// This is necessary to restore "force_abort" even before the throw point
/// for the error message has been reached.
#[export_name = "update_force_abort"]
pub unsafe extern "C" fn update_force_abort_impl() {
    if CAUSE_ABORT {
        force_abort = true;
    }
}

/// Get the Rust-owned cause_abort value (for C callers that still reference it).
#[no_mangle]
pub unsafe extern "C" fn rs_get_cause_abort() -> bool {
    CAUSE_ABORT
}

/// Set the Rust-owned cause_abort value (for C callers that still reference it).
#[no_mangle]
pub unsafe extern "C" fn rs_set_cause_abort(val: bool) {
    CAUSE_ABORT = val;
}

/// Representation of exception_state_T matching C layout (sizeof=24).
#[repr(C)]
pub struct ExceptionStateT {
    pub estate_current_exception: *mut c_void,
    pub estate_did_throw: bool,
    pub estate_need_rethrow: bool,
    pub estate_trylevel: c_int,
    pub estate_did_emsg: c_int,
}

/// Representation of cleanup_T matching C layout (sizeof=16).
#[repr(C)]
pub struct CleanupT {
    pub pending: c_int,
    _padding: [u8; 4],
    pub exception: *mut ExceptT,
}

/// Save the current exception state in "estate".
#[export_name = "exception_state_save"]
pub unsafe extern "C" fn exception_state_save(estate: *mut ExceptionStateT) {
    (*estate).estate_current_exception = current_exception;
    (*estate).estate_did_throw = did_throw;
    (*estate).estate_need_rethrow = need_rethrow;
    (*estate).estate_trylevel = trylevel;
    (*estate).estate_did_emsg = did_emsg;
}

/// Restore the current exception state from "estate".
#[export_name = "exception_state_restore"]
pub unsafe extern "C" fn exception_state_restore(estate: *mut ExceptionStateT) {
    // Handle any outstanding exceptions before restoring the state
    if did_throw {
        handle_did_throw();
    }
    current_exception = (*estate).estate_current_exception;
    did_throw = (*estate).estate_did_throw;
    need_rethrow = (*estate).estate_need_rethrow;
    trylevel = (*estate).estate_trylevel;
    did_emsg = (*estate).estate_did_emsg;
}

/// Clear the current exception state.
#[export_name = "exception_state_clear"]
pub unsafe extern "C" fn exception_state_clear() {
    current_exception = std::ptr::null_mut();
    did_throw = false;
    need_rethrow = false;
    trylevel = 0;
    did_emsg = 0;
}

/// Representation of msglist_T matching C layout (sizeof=40).
#[repr(C)]
pub struct MsglistT {
    pub next: *mut MsglistT,
    pub msg: *mut c_char,
    pub throw_msg: *mut c_char,
    pub sfile: *mut c_char,
    pub slnum: LinenrT,
    pub multiline: bool,
}

/// Free a "msg_list" and the messages it contains.
#[no_mangle]
pub unsafe extern "C" fn free_msglist(l: *mut MsglistT) {
    let mut messages = l;
    while !messages.is_null() {
        let next = (*messages).next;
        xfree((*messages).msg.cast::<c_void>());
        xfree((*messages).sfile.cast::<c_void>());
        xfree(messages.cast::<c_void>());
        messages = next;
    }
}

/// Free global "*msg_list" and the messages it contains, then set "*msg_list" to NULL.
#[export_name = "free_global_msglist"]
pub unsafe extern "C" fn free_global_msglist_impl() {
    free_msglist((*msg_list).cast::<MsglistT>());
    *msg_list = std::ptr::null_mut();
}

/// Discard a pending return value (calls tv_free).
#[no_mangle]
pub unsafe extern "C" fn discard_pending_return(p: *mut c_void) {
    tv_free(p);
}

/// Handle ":endfunction" when not after a ":function".
#[export_name = "ex_endfunction"]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn ex_endfunction_impl(eap: *mut c_void) {
    let _ = eap;
    semsg(e_str_not_inside_function, c":endfunction".as_ptr());
}

/// Returns true if the string "p" looks like a ":while" or ":for" command.
#[export_name = "has_loop_cmd"]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn has_loop_cmd(mut p: *mut c_char) -> bool {
    // skip modifiers, white space and ':'
    loop {
        while *p == b' ' as i8 || *p == b'\t' as i8 || *p == b':' as i8 {
            p = p.add(1);
        }
        let len = modifier_len(p);
        if len == 0 {
            break;
        }
        p = p.add(len as usize);
    }
    (*p == b'w' as i8 && *p.add(1) == b'h' as i8)
        || (*p == b'f' as i8 && *p.add(1) == b'o' as i8 && *p.add(2) == b'r' as i8)
}

/// Report information about something pending in a finally clause if required by
/// the 'verbose' option or when debugging, when something is made pending.
#[export_name = "report_make_pending"]
pub unsafe extern "C" fn report_make_pending_impl(pending: c_int, value: *mut c_void) {
    if p_verbose >= 14 || debug_break_level > 0 {
        if debug_break_level <= 0 {
            verbose_enter();
        }
        report_pending(RP_MAKE, pending, value);
        if debug_break_level <= 0 {
            verbose_leave();
        }
    }
}

/// Get an exception message that is to be stored in current_exception->value.
///
/// Returns an allocated string (set *should_free=true for ET_ERROR) or a
/// pointer into value (for other types, *should_free=false).
#[export_name = "get_exception_string"]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment
)]
pub unsafe extern "C" fn get_exception_string(
    value: *mut c_void,
    type_: ExceptTypeT,
    cmdname: *mut c_char,
    should_free: *mut bool,
) -> *mut c_char {
    if type_ == ExceptTypeT::EtError {
        *should_free = true;
        let mesg = (*value.cast::<MsglistT>()).throw_msg;
        let mesg_len = libc_strlen(mesg);

        let (ret, val) = if !cmdname.is_null() && *cmdname != 0 {
            let cmdlen = libc_strlen(cmdname);
            let r = xstrnsave(c"Vim(".as_ptr(), 4 + cmdlen + 2 + mesg_len);
            // Copy cmdname into ret[4..]
            std::ptr::copy_nonoverlapping(cmdname, r.add(4), cmdlen);
            // Copy "):" into ret[4+cmdlen..]
            *r.add(4 + cmdlen) = b')' as i8;
            *r.add(4 + cmdlen + 1) = b':' as i8;
            (r, r.add(4 + cmdlen + 2))
        } else {
            let r = xstrnsave(c"Vim:".as_ptr(), 4 + mesg_len);
            (r, r.add(4))
        };

        // Null-terminate val
        *val = 0;

        // msg_add_fname may have been used to prefix the message with a file
        // name in quotes. In the exception value, put the file name in
        // parentheses and move it to the end.
        let mut p = mesg;
        loop {
            if *p == 0
                || (*p == b'E' as i8
                    && ascii_isdigit(c_int::from(*p.add(1)))
                    && (*p.add(2) == b':' as i8
                        || (ascii_isdigit(c_int::from(*p.add(2)))
                            && (*p.add(3) == b':' as i8
                                || (ascii_isdigit(c_int::from(*p.add(3)))
                                    && *p.add(4) == b':' as i8)))))
            {
                if *p == 0 || p == mesg {
                    // 'E123' missing or at beginning
                    // strcat(val, mesg)
                    let val_len = libc_strlen(val);
                    std::ptr::copy_nonoverlapping(mesg, val.add(val_len), mesg_len + 1);
                } else {
                    // '"filename" E123: message text'
                    if *mesg != b'"' as i8
                        || p.offset_from(mesg) < 3
                        || *p.sub(2) != b'"' as i8
                        || *p.sub(1) != b' ' as i8
                    {
                        // "E123:" is part of the file name, continue
                        p = p.add(1);
                        continue;
                    }

                    // strcat(val, p)
                    let p_len = libc_strlen(p);
                    let val_len = libc_strlen(val);
                    std::ptr::copy_nonoverlapping(p, val.add(val_len), p_len + 1);

                    // p[-2] = NUL; snprintf(val + strlen(p), " (%s)", &mesg[1]); p[-2] = '"';
                    let save = *p.sub(2);
                    *p.sub(2) = 0;
                    let suffix_len = libc_strlen(c" (%s)".as_ptr().cast::<c_char>()) + mesg_len;
                    vim_snprintf(
                        val.add(p_len),
                        suffix_len + 1,
                        c" (%s)".as_ptr(),
                        mesg.add(1),
                    );
                    *p.sub(2) = save;
                }
                break;
            }
            p = p.add(1);
        }

        ret
    } else {
        *should_free = false;
        value.cast::<c_char>()
    }
}

/// Throw a new exception. `value` is the exception string for user or interrupt
/// exceptions, or points to a message list for error exceptions.
///
/// Returns OK on success, FAIL when out of memory or illegal user exception.
#[no_mangle]
pub unsafe extern "C" fn throw_exception(
    value: *mut c_void,
    type_: ExceptTypeT,
    cmdname: *mut c_char,
) -> c_int {
    // Disallow faking Interrupt or error exceptions as user exceptions.
    if type_ == ExceptTypeT::EtUser {
        let v = value.cast::<c_char>();
        if libc_strncmp(v, c"Vim".as_ptr(), 3) == 0 {
            let c3 = *v.add(3);
            #[allow(clippy::cast_possible_wrap)]
            if c3 == 0 || c3 == b':' as i8 || c3 == b'(' as i8 {
                emsg(c"E608: Cannot :throw exceptions with 'Vim' prefix".as_ptr());
                current_exception = std::ptr::null_mut();
                return FAIL;
            }
        }
    }

    let excp = xmalloc(std::mem::size_of::<ExceptT>()).cast::<ExceptT>();

    if type_ == ExceptTypeT::EtError {
        (*excp).messages = value.cast::<MsglistT>();
    } else {
        (*excp).messages = std::ptr::null_mut();
    }

    let mut should_free = false;
    (*excp).value = get_exception_string(value, type_, cmdname, &raw mut should_free);
    if (*excp).value.is_null() && should_free {
        // nomem
        xfree(excp.cast::<c_void>());
        suppress_errthrow = true;
        emsg(e_outofmem);
        current_exception = std::ptr::null_mut();
        return FAIL;
    }

    (*excp).type_ = type_;
    if (*excp).type_ == ExceptTypeT::EtError {
        let entry = value.cast::<MsglistT>();
        if (*entry).sfile.is_null() {
            (*excp).throw_name = estack_sfile(ESTACK_NONE);
            if (*excp).throw_name.is_null() {
                (*excp).throw_name = xstrdup(c"".as_ptr());
            }
            (*excp).throw_lnum = nvim_get_sourcing_lnum_direct();
        } else {
            (*excp).throw_name = (*entry).sfile;
            (*entry).sfile = std::ptr::null_mut();
            (*excp).throw_lnum = (*entry).slnum;
        }
    } else {
        (*excp).throw_name = estack_sfile(ESTACK_NONE);
        if (*excp).throw_name.is_null() {
            (*excp).throw_name = xstrdup(c"".as_ptr());
        }
        (*excp).throw_lnum = nvim_get_sourcing_lnum_direct();
    }

    (*excp).stacktrace = stacktrace_create();
    nvim_tv_list_ref((*excp).stacktrace);
    (*excp).caught = std::ptr::null_mut();

    if p_verbose >= 13 || debug_break_level > 0 {
        let save_msg_silent = msg_silent;
        if debug_break_level > 0 {
            msg_silent = 0; // display messages
        } else {
            verbose_enter();
        }
        no_wait_return += 1;
        if debug_break_level > 0 || *p_vfile == 0 {
            msg_scroll = 1; // always scroll up, don't overwrite
        }
        smsg(0, c"Exception thrown: %s".as_ptr(), (*excp).value);
        msg_puts(c"\n".as_ptr());
        if debug_break_level > 0 || *p_vfile == 0 {
            cmdline_row = msg_row;
        }
        no_wait_return -= 1;
        if debug_break_level > 0 {
            msg_silent = save_msg_silent;
        } else {
            verbose_leave();
        }
    }

    current_exception = excp.cast::<c_void>();
    OK
}

/// Discard an exception. `was_finished` is set when the exception has been
/// caught and the catch clause has been ended normally.
#[no_mangle]
pub unsafe extern "C" fn discard_exception(excp: *mut ExceptT, was_finished: bool) {
    if current_exception == excp.cast::<c_void>() {
        current_exception = std::ptr::null_mut();
    }
    if excp.is_null() {
        internal_error(c"discard_exception()".as_ptr());
        return;
    }

    if p_verbose >= 13 || debug_break_level > 0 {
        let save_msg_silent = msg_silent;
        // Save IObuff to restore it afterwards
        let saved_iobuff = xstrdup((&raw const IObuff).cast::<c_char>());
        if debug_break_level > 0 {
            msg_silent = 0;
        } else {
            verbose_enter();
        }
        no_wait_return += 1;
        if debug_break_level > 0 || *p_vfile == 0 {
            msg_scroll = 1;
        }
        if was_finished {
            smsg(0, c"Exception finished: %s".as_ptr(), (*excp).value);
        } else {
            smsg(0, c"Exception discarded: %s".as_ptr(), (*excp).value);
        }
        msg_puts(c"\n".as_ptr());
        if debug_break_level > 0 || *p_vfile == 0 {
            cmdline_row = msg_row;
        }
        no_wait_return -= 1;
        if debug_break_level > 0 {
            msg_silent = save_msg_silent;
        } else {
            verbose_leave();
        }
        xstrlcpy((&raw mut IObuff).cast::<c_char>(), saved_iobuff, 1025);
        xfree(saved_iobuff.cast::<c_void>());
    }

    if (*excp).type_ != ExceptTypeT::EtInterrupt {
        xfree((*excp).value.cast::<c_void>());
    }
    if (*excp).type_ == ExceptTypeT::EtError {
        free_msglist((*excp).messages);
    }
    xfree((*excp).throw_name.cast::<c_void>());
    tv_list_unref((*excp).stacktrace);
    xfree(excp.cast::<c_void>());
}

/// Discard the exception currently being thrown.
#[export_name = "discard_current_exception"]
pub unsafe extern "C" fn discard_current_exception() {
    if !current_exception.is_null() {
        discard_exception(current_exception.cast::<ExceptT>(), false);
    }
    // Note: all globals manipulated here should be saved/restored in
    // try_enter/try_leave.
    did_throw = false;
    need_rethrow = false;
}

/// Put an exception on the caught stack.
#[no_mangle]
pub unsafe extern "C" fn catch_exception(excp: *mut ExceptT) {
    (*excp).caught = caught_stack.cast::<ExceptT>();
    caught_stack = excp.cast::<c_void>();
    set_vim_var_string(VV_EXCEPTION, (*excp).value, -1);
    set_vim_var_list(VV_STACKTRACE, (*excp).stacktrace);
    if !(*excp).throw_name.is_null() && *(*excp).throw_name != 0 {
        if (*excp).throw_lnum != 0 {
            vim_snprintf(
                (&raw mut IObuff).cast::<c_char>(),
                1025,
                c"%s, line %ld".as_ptr(),
                (*excp).throw_name,
                i64::from((*excp).throw_lnum),
            );
        } else {
            vim_snprintf(
                (&raw mut IObuff).cast::<c_char>(),
                1025,
                c"%s".as_ptr(),
                (*excp).throw_name,
            );
        }
        set_vim_var_string(VV_THROWPOINT, (&raw const IObuff).cast::<c_char>(), -1);
    } else {
        set_vim_var_string(VV_THROWPOINT, std::ptr::null(), -1);
    }

    if p_verbose >= 13 || debug_break_level > 0 {
        let save_msg_silent = msg_silent;
        if debug_break_level > 0 {
            msg_silent = 0;
        } else {
            verbose_enter();
        }
        no_wait_return += 1;
        if debug_break_level > 0 || *p_vfile == 0 {
            msg_scroll = 1;
        }
        smsg(0, c"Exception caught: %s".as_ptr(), (*excp).value);
        msg_puts(c"\n".as_ptr());
        if debug_break_level > 0 || *p_vfile == 0 {
            cmdline_row = msg_row;
        }
        no_wait_return -= 1;
        if debug_break_level > 0 {
            msg_silent = save_msg_silent;
        } else {
            verbose_leave();
        }
    }
}

/// Remove an exception from the caught stack.
#[no_mangle]
pub unsafe extern "C" fn finish_exception(excp: *mut ExceptT) {
    if excp != caught_stack.cast::<ExceptT>() {
        internal_error(c"finish_exception()".as_ptr());
    }
    let caught = (*caught_stack.cast::<ExceptT>()).caught;
    caught_stack = caught.cast::<c_void>();
    if caught_stack.is_null() {
        set_vim_var_string(VV_EXCEPTION, std::ptr::null(), -1);
        set_vim_var_string(VV_THROWPOINT, std::ptr::null(), -1);
        set_vim_var_list(VV_STACKTRACE, std::ptr::null_mut());
    } else {
        let cs = caught_stack.cast::<ExceptT>();
        set_vim_var_string(VV_EXCEPTION, (*cs).value, -1);
        set_vim_var_list(VV_STACKTRACE, (*cs).stacktrace);
        if !(*cs).throw_name.is_null() && *(*cs).throw_name != 0 {
            if (*cs).throw_lnum != 0 {
                vim_snprintf(
                    (&raw mut IObuff).cast::<c_char>(),
                    1025,
                    c"%s, line %ld".as_ptr(),
                    (*cs).throw_name,
                    i64::from((*cs).throw_lnum),
                );
            } else {
                vim_snprintf(
                    (&raw mut IObuff).cast::<c_char>(),
                    1025,
                    c"%s".as_ptr(),
                    (*cs).throw_name,
                );
            }
            set_vim_var_string(VV_THROWPOINT, (&raw const IObuff).cast::<c_char>(), -1);
        } else {
            set_vim_var_string(VV_THROWPOINT, std::ptr::null(), -1);
        }
    }
    discard_exception(excp, true);
}

/// Rewind conditionals until index `idx` is reached. `cond_type` and
/// `cond_level` specify a conditional type and the address of a level variable
/// which is to be decremented with each skipped conditional.
#[export_name = "rewind_conditionals"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rewind_conditionals(
    cstack: *mut CstackT,
    idx: c_int,
    cond_type: c_int,
    cond_level: *mut c_int,
) {
    while (*cstack).cs_idx > idx {
        let i = (*cstack).cs_idx as usize;
        if (*cstack).cs_flags[i] & cond_type != 0 {
            *cond_level -= 1;
        }
        if (*cstack).cs_flags[i] & CSF_FOR != 0 {
            free_for_info((*cstack).cs_forinfo[i]);
        }
        (*cstack).cs_idx -= 1;
    }
}

/// Report information about something pending in a finally clause.
/// "action" tells whether something is made pending/resumed/discarded.
#[export_name = "report_pending"]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn report_pending(action: c_int, pending: c_int, value: *mut c_void) {
    let action_mesg: *const c_char = match action {
        RP_MAKE => gettext(c"%s made pending".as_ptr()),
        RP_RESUME => gettext(c"%s resumed".as_ptr()),
        _ => gettext(c"%s discarded".as_ptr()),
    };

    if pending == CSTP_NONE {
        return;
    }

    let s: *mut c_char;
    let mut free_s = false;
    // mesg may be replaced for CSTP_THROW case
    let mut mesg: *mut c_char = action_mesg.cast_mut();
    let mut free_mesg = false;

    match pending {
        CSTP_CONTINUE => s = c":continue".as_ptr().cast_mut(),
        CSTP_BREAK => s = c":break".as_ptr().cast_mut(),
        CSTP_FINISH => s = c":finish".as_ptr().cast_mut(),
        CSTP_RETURN => {
            s = get_return_cmd(value);
            free_s = true;
        }
        _ => {
            if pending & CSTP_THROW != 0 {
                vim_snprintf(
                    (&raw mut IObuff).cast::<c_char>(),
                    1025,
                    action_mesg,
                    gettext(c"Exception".as_ptr()),
                );
                mesg = concat_str((&raw const IObuff).cast::<c_char>(), c": %s".as_ptr());
                free_mesg = true;
                s = (*value.cast::<ExceptT>()).value;
            } else if pending & CSTP_ERROR != 0 && pending & CSTP_INTERRUPT != 0 {
                s = gettext(c"Error and interrupt".as_ptr()).cast_mut();
            } else if pending & CSTP_ERROR != 0 {
                s = gettext(c"Error".as_ptr()).cast_mut();
            } else {
                s = gettext(c"Interrupt".as_ptr()).cast_mut();
            }
        }
    }

    let save_msg_silent = msg_silent;
    if debug_break_level > 0 {
        msg_silent = 0;
    }
    no_wait_return += 1;
    msg_scroll = 1;
    smsg(0, mesg, s);
    msg_puts(c"\n".as_ptr());
    cmdline_row = msg_row;
    no_wait_return -= 1;
    if debug_break_level > 0 {
        msg_silent = save_msg_silent;
    }

    if free_s {
        xfree(s.cast::<c_void>());
    }
    if free_mesg {
        xfree(mesg.cast::<c_void>());
    }
}

/// report_resume_pending: report something pending in a finally clause being resumed.
#[no_mangle]
pub unsafe extern "C" fn report_resume_pending(pending: c_int, value: *mut c_void) {
    if p_verbose >= 14 || debug_break_level > 0 {
        if debug_break_level <= 0 {
            verbose_enter();
        }
        report_pending(RP_RESUME, pending, value);
        if debug_break_level <= 0 {
            verbose_leave();
        }
    }
}

/// report_discard_pending: report something pending in a finally clause being discarded.
#[no_mangle]
pub unsafe extern "C" fn report_discard_pending(pending: c_int, value: *mut c_void) {
    if p_verbose >= 14 || debug_break_level > 0 {
        if debug_break_level <= 0 {
            verbose_enter();
        }
        report_pending(RP_DISCARD, pending, value);
        if debug_break_level <= 0 {
            verbose_leave();
        }
    }
}

/// Throw the current exception through the specified cstack.
#[export_name = "do_throw"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn do_throw(cstack: *mut CstackT) {
    let idx = cleanup_conditionals(cstack, 0, 0);
    if idx >= 0 {
        let i = idx as usize;
        if (*cstack).cs_flags[i] & CSF_CAUGHT == 0 {
            if (*cstack).cs_flags[i] & CSF_ACTIVE != 0 {
                (*cstack).cs_flags[i] |= CSF_THROWN;
            } else {
                (*cstack).cs_flags[i] &= !CSF_THROWN;
            }
        }
        (*cstack).cs_flags[i] &= !CSF_ACTIVE;
        (*cstack).cs_pend.csp_ex[i] = current_exception;
    }
    did_throw = true;
}

/// Cause a throw of an error exception if appropriate.
///
/// Returns true if the error message should not be displayed by emsg().
/// Sets *ignore if the emsg() call should be ignored completely.
#[export_name = "cause_errthrow"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn cause_errthrow(
    mesg: *const c_char,
    multiline: bool,
    severe: bool,
    ignore: *mut bool,
) -> bool {
    // Do nothing when suppress_errthrow is set.
    if suppress_errthrow {
        return false;
    }

    // If emsg() has not been called previously, temporarily reset force_abort.
    if did_emsg == 0 {
        rs_set_cause_abort(force_abort);
        force_abort = false;
    }

    // If no try conditional is active and no exception is being thrown, do nothing.
    if ((trylevel == 0 && !rs_get_cause_abort()) || emsg_silent != 0) && !did_throw {
        return false;
    }

    // Ignore interrupt message when inside a try conditional.
    if mesg == e_interr {
        *ignore = true;
        return true;
    }

    // Ensure that all commands in nested function calls and sourced files are aborted.
    rs_set_cause_abort(true);

    // Discard exception currently being thrown to prevent it from being caught.
    if did_throw {
        // When discarding an interrupt exception, reset got_int.
        if (*current_exception.cast::<ExceptT>()).type_ == ExceptTypeT::EtInterrupt {
            got_int = false;
        }
        discard_current_exception();
    }

    // Prepare the throw of an error exception.
    if !msg_list.is_null() {
        // Cast msg_list to the correct type (*mut *mut MsglistT).
        let msg_list_typed = msg_list.cast::<*mut MsglistT>();

        // Find the tail of the list, maintaining a pointer to the last ->next field.
        let mut plist: *mut *mut MsglistT = msg_list_typed;
        while !(*plist).is_null() {
            plist = &raw mut (**plist).next;
        }

        let elem = xmalloc(std::mem::size_of::<MsglistT>()).cast::<MsglistT>();
        (*elem).msg = xstrdup(mesg);
        (*elem).multiline = multiline;
        (*elem).next = std::ptr::null_mut();
        (*elem).throw_msg = std::ptr::null_mut();
        *plist = elem;

        let is_first = plist == msg_list_typed;
        if is_first || severe {
            let tmsg = (*elem).msg;
            // Skip the extra "Vim " prefix for message "E458".
            #[allow(clippy::cast_possible_wrap)]
            if libc_strncmp(tmsg, c"Vim E".as_ptr(), 5) == 0
                && ascii_isdigit(c_int::from(*tmsg.add(5)))
                && ascii_isdigit(c_int::from(*tmsg.add(6)))
                && ascii_isdigit(c_int::from(*tmsg.add(7)))
                && *tmsg.add(8) == b':' as i8
                && *tmsg.add(9) == b' ' as i8
            {
                (*(*msg_list_typed)).throw_msg = tmsg.add(4);
            } else {
                (*(*msg_list_typed)).throw_msg = tmsg;
            }
        }

        // Get the source name and lnum now, it may change before reaching do_errthrow().
        (*elem).sfile = estack_sfile(ESTACK_NONE);
        (*elem).slnum = nvim_get_sourcing_lnum_direct();
    }
    true
}

/// Throw the message specified in cause_errthrow() as an error exception.
#[export_name = "do_errthrow"]
pub unsafe extern "C" fn do_errthrow(cstack: *mut CstackT, cmdname: *mut c_char) {
    // Ensure all commands in nested function calls/sourced files are aborted.
    if rs_get_cause_abort() {
        rs_set_cause_abort(false);
        force_abort = true;
    }

    // If no exception is to be thrown, do nothing.
    if msg_list.is_null() || (*msg_list).is_null() {
        return;
    }

    if throw_exception(*msg_list, ExceptTypeT::EtError, cmdname) == FAIL {
        free_msglist((*msg_list).cast::<MsglistT>());
    } else if !cstack.is_null() {
        do_throw(cstack);
    } else {
        need_rethrow = true;
    }
    *msg_list = std::ptr::null_mut();
}

/// Replace the current exception by an interrupt exception if appropriate.
///
/// Returns true if the current exception is discarded.
#[export_name = "do_intthrow"]
pub unsafe extern "C" fn do_intthrow(cstack: *mut CstackT) -> bool {
    // If no interrupt occurred or no try conditional is active and no exception
    // is being thrown, do nothing.
    if !got_int || (trylevel == 0 && !did_throw) {
        return false;
    }

    // Throw an interrupt exception, so that everything will be aborted
    // (except for executing finally clauses), until the interrupt exception
    // is caught. If an interrupt exception is already being thrown, do nothing.
    if did_throw {
        if (*current_exception.cast::<ExceptT>()).type_ == ExceptTypeT::EtInterrupt {
            return false;
        }
        // An interrupt exception replaces any user or error exception.
        discard_current_exception();
    }
    if throw_exception(
        c"Vim:Interrupt".as_ptr().cast::<c_void>().cast_mut(),
        ExceptTypeT::EtInterrupt,
        std::ptr::null_mut(),
    ) != FAIL
    {
        do_throw(cstack);
    }

    true
}

/// Save exception state for cleanup autocommand execution.
#[export_name = "enter_cleanup"]
pub unsafe extern "C" fn enter_cleanup(csp: *mut CleanupT) {
    if did_emsg != 0 || got_int || did_throw || need_rethrow {
        let throw_pending = if did_throw || need_rethrow {
            CSTP_THROW
        } else {
            0
        };
        (*csp).pending = (if did_emsg != 0 { CSTP_ERROR } else { 0 })
            | (if got_int { CSTP_INTERRUPT } else { 0 })
            | throw_pending;

        if did_throw || need_rethrow {
            (*csp).exception = current_exception.cast::<ExceptT>();
            current_exception = std::ptr::null_mut();
        } else {
            (*csp).exception = std::ptr::null_mut();
            if did_emsg != 0 {
                force_abort |= rs_get_cause_abort();
                rs_set_cause_abort(false);
            }
        }
        did_emsg = 0;
        got_int = false;
        did_throw = false;
        need_rethrow = false;

        // Report if required by the 'verbose' option or when debugging.
        report_make_pending_impl((*csp).pending, (*csp).exception.cast::<c_void>());
    } else {
        (*csp).pending = CSTP_NONE;
        (*csp).exception = std::ptr::null_mut();
    }
}

/// Restore exception state after cleanup autocommand execution.
#[export_name = "leave_cleanup"]
pub unsafe extern "C" fn leave_cleanup(csp: *mut CleanupT) {
    let pending = (*csp).pending;

    if pending == CSTP_NONE {
        return;
    }

    // If there was an aborting error, interrupt, or uncaught exception after
    // enter_cleanup(), discard what has been made pending.
    if aborting_impl() || need_rethrow {
        if pending & CSTP_THROW != 0 {
            // Cancel the pending exception (includes report).
            discard_exception((*csp).exception, false);
        } else {
            report_discard_pending(pending, std::ptr::null_mut());
        }

        // If an error was about to be converted to an exception, free the message list.
        if !msg_list.is_null() {
            free_global_msglist_impl();
        }
    } else {
        // Restore the pending error/interrupt/exception state.
        if pending & CSTP_THROW != 0 {
            current_exception = (*csp).exception.cast::<c_void>();
        } else if pending & CSTP_ERROR != 0 {
            // Let cause_abort take the part of force_abort.
            rs_set_cause_abort(force_abort);
            force_abort = false;
        }

        // Restore the pending values of did_emsg, got_int, and did_throw.
        if pending & CSTP_ERROR != 0 {
            did_emsg = 1;
        }
        if pending & CSTP_INTERRUPT != 0 {
            got_int = true;
        }
        if pending & CSTP_THROW != 0 {
            need_rethrow = true; // did_throw will be set by do_one_cmd()
        }

        // Report if required by the 'verbose' option or when debugging.
        report_resume_pending(
            pending,
            if pending & CSTP_THROW != 0 {
                current_exception
            } else {
                std::ptr::null_mut()
            },
        );
    }
}

// Helper: check if a character is an ASCII digit
fn ascii_isdigit(c: c_int) -> bool {
    c >= i32::from(b'0') && c <= i32::from(b'9')
}

// Helper: compute strlen of a C string
const unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// Helper: strncmp of two C strings
#[allow(clippy::cast_sign_loss)]
unsafe fn libc_strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int {
    for i in 0..n {
        let ca = *a.add(i) as u8;
        let cb = *b.add(i) as u8;
        if ca != cb {
            return c_int::from(ca) - c_int::from(cb);
        }
        if ca == 0 {
            return 0;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_constant() {
        assert_eq!(FAIL, 0);
    }

    #[test]
    fn test_ok_constant() {
        assert_eq!(OK, 1);
    }

    #[test]
    fn test_exception_state_size() {
        // ExceptionStateT must be 24 bytes to match C's exception_state_T
        assert_eq!(std::mem::size_of::<ExceptionStateT>(), 24);
    }

    #[test]
    fn test_except_t_size() {
        // ExceptT must be 56 bytes to match C's except_T
        assert_eq!(std::mem::size_of::<ExceptT>(), 56);
    }

    #[test]
    fn test_msglist_t_size() {
        // MsglistT must be 40 bytes to match C's msglist_T
        assert_eq!(std::mem::size_of::<MsglistT>(), 40);
    }

    #[test]
    fn test_cstack_t_size() {
        // CstackT must be 1288 bytes to match C's cstack_T
        assert_eq!(std::mem::size_of::<CstackT>(), 1288);
    }
}
