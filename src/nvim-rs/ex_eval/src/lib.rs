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
    fn report_pending(action: c_int, pending: c_int, value: *mut c_void);
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
}

// VimVarIndex constants matching eval_defs.h VimVarIndex enum
// Counted from the enum definition:
// VV_COUNT=0..VV_DYING=29, VV_EXCEPTION=30, VV_THROWPOINT=31, ...VV_STACKTRACE=125
const VV_EXCEPTION: c_int = 30;
const VV_THROWPOINT: c_int = 31;
const VV_STACKTRACE: c_int = 125;

// CSF_ flag constants matching ex_eval_defs.h
const CSF_FOR: c_int = 0x0010;

// ESTACK_NONE constant from runtime_defs.h
const ESTACK_NONE: c_int = 0;

// OK constant from vim_defs.h
const OK: c_int = 1;

// Rust-owned static replacing the C file-local `static bool cause_abort`
static mut CAUSE_ABORT: bool = false;

// RP_ constants for report_pending
const RP_MAKE: c_int = 0;

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
