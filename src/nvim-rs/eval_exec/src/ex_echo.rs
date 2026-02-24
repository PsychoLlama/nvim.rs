//! `:echo`, `:echon`, `:execute`, `:echomsg`, `:echoerr` implementations.
//!
//! Migrated from `eval_shim.c` Phase 6.

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};

use crate::eval::{EvalargHandle, ExargHandle, TypevalHandle};

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Evalarg management (heap-allocated)
    fn nvim_evalarg_alloc_from_eap(eap: ExargHandle, skip: bool) -> EvalargHandle;
    fn nvim_evalarg_clear_and_free(ea: EvalargHandle, eap: ExargHandle);

    // eval1 (Rust FFI export, takes arg as *mut *mut c_char)
    fn eval1(arg: *mut *mut c_char, rettv: TypevalHandle, evalarg: EvalargHandle) -> c_int;

    // eval1_emsg wrapper (non-static wrapper)
    fn nvim_eval1_emsg_wrapper(
        arg: *mut *mut c_char,
        rettv: TypevalHandle,
        eap: ExargHandle,
    ) -> c_int;

    // Typval operations
    fn tv_clear(tv: TypevalHandle);
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;

    // String utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // Globals accessors
    fn nvim_got_int() -> c_int;
    fn nvim_set_need_clr_eos(val: c_int);
    fn nvim_aborting() -> bool;
    fn did_emsg_get() -> c_int;
    fn called_emsg_get() -> c_int;
    fn nvim_get_msg_didout() -> c_int;
    fn nvim_emsg_skip_inc();
    fn nvim_emsg_skip_dec();
    fn nvim_get_did_emsg() -> c_int;
    fn nvim_set_did_emsg(val: c_int);
    fn nvim_get_force_abort() -> c_int;

    // semsg with e_invexpr2
    fn nvim_semsg_invexpr2(p: *const c_char);

    // EAP accessors
    fn nvim_eap_get_skip_local(eap: ExargHandle) -> c_int;
    fn nvim_eap_get_arg_local(eap: ExargHandle) -> *mut c_char;
    fn nvim_eap_set_nextcmd_checked(eap: ExargHandle, arg: *mut c_char);
    fn nvim_eap_get_cmdidx(eap: ExargHandle) -> c_int;

    // CMD_xxx constants
    fn nvim_docmd_cmd_echo() -> c_int;
    fn nvim_docmd_cmd_echon() -> c_int;
    fn nvim_docmd_cmd_execute() -> c_int;
    fn nvim_docmd_cmd_echomsg() -> c_int;
    fn nvim_docmd_cmd_echoerr() -> c_int;

    // Message functions
    fn nvim_msg_ext_set_kind(kind: *const c_char);
    fn nvim_msg_sb_eol();
    fn nvim_msg_start();
    fn nvim_msg_puts_hl(msg: *const c_char, attr: c_int, right: bool);
    fn nvim_msg_multiline_cstr(
        s: *const c_char,
        hl_id: c_int,
        check_int: bool,
        hist: bool,
        need_clear: *mut bool,
    );
    fn nvim_msg_clr_eos();
    fn nvim_msg_end();
    fn nvim_set_msg_ext_append(val: bool);
    fn nvim_msg_echomsg(str: *const c_char, hl_id: c_int);

    // encode functions
    fn nvim_encode_tv2echo(tv: TypevalHandle) -> *mut c_char;
    fn nvim_encode_tv2string_wrapper(tv: TypevalHandle) -> *mut c_char;
    fn nvim_eval_tv_get_str(tv: TypevalHandle) -> *mut c_char;
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;

    // echoerr
    fn nvim_emsg_multiline_echoerr(str: *const c_char);

    // garray for ex_execute
    fn nvim_ga_alloc_execute() -> *mut c_void;
    fn nvim_ga_grow_wrapper(ga: *mut c_void, n: c_int);
    fn nvim_ga_is_empty_execute(ga: *mut c_void) -> bool;
    fn nvim_ga_append_space(ga: *mut c_void);
    fn nvim_ga_append_str_len(ga: *mut c_void, str: *const c_char, len: c_int);
    fn nvim_ga_get_data(ga: *const c_void) -> *mut c_char;
    fn nvim_ga_data_is_null(ga: *const c_void) -> bool;
    fn nvim_ga_clear_and_free(ga: *mut c_void);

    // do_cmdline for :execute
    fn nvim_do_cmdline_execute(cmd: *mut c_char, eap: ExargHandle);

    // syn_name2id wrapper for ex_echohl
    fn nvim_syn_name2id_wrapper(name: *const c_char) -> c_int;
}

// =============================================================================
// echo_hl_id state (migrated from C echo_hl_id static in eval_shim.c)
// =============================================================================

/// Highlight ID used for `:echo`. Equivalent to C `echo_hl_id` in eval_shim.c.
static mut ECHO_HL_ID: c_int = 0;

/// Get the current echo highlight ID.
///
/// Called from C (replaces nvim_get_echo_hl_id accessor) and from Rust.
///
/// # Safety
/// Safe to call from C; accesses a static mut (single-threaded).
#[no_mangle]
pub unsafe extern "C" fn rs_get_echo_hl_id() -> c_int {
    ECHO_HL_ID
}

/// Implementation of `:echohl {name}`.
///
/// Migrated from C `ex_echohl` in eval_shim.c.
///
/// # Safety
/// - `eap` must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_echohl(eap: ExargHandle) {
    let arg = nvim_eap_get_arg_local(eap);
    ECHO_HL_ID = nvim_syn_name2id_wrapper(arg);
}

// =============================================================================
// Constants
// =============================================================================

// Constants
const OK: c_int = 1;
const FAIL: c_int = 0;
const VAR_STRING: c_int = 2;

// C string literals
static KIND_ECHO: &[u8] = b"echo\0";
static KIND_ECHOMSG: &[u8] = b"echomsg\0";
static KIND_SPACE: &[u8] = b" \0";

/// Allocate a typval on the heap (64 bytes, zeroed).
#[inline]
unsafe fn alloc_typval() -> TypevalHandle {
    let ptr = xmalloc(64);
    std::ptr::write_bytes(ptr as *mut u8, 0, 64);
    TypevalHandle::from_ptr(ptr)
}

/// Free a heap-allocated typval.
#[inline]
unsafe fn free_typval(tv: TypevalHandle) {
    if !tv.is_null() {
        xfree(tv.as_ptr());
    }
}

/// Get a byte at a pointer (0 if null).
#[inline]
unsafe fn get_byte(p: *const c_char) -> u8 {
    if p.is_null() {
        0
    } else {
        *p as u8
    }
}

/// Compute strlen of a C string.
#[inline]
unsafe fn cstr_len(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut p = s;
    while get_byte(p) != 0 {
        p = p.add(1);
    }
    (p as usize).wrapping_sub(s as usize)
}

// =============================================================================
// ex_echo
// =============================================================================

/// Implementation of ex_echo: `:echo` and `:echon`.
///
/// Migrated from C `ex_echo` in eval_shim.c.
///
/// # Safety
/// - `eap` must be a valid exarg_T pointer
pub unsafe fn ex_echo_impl(eap: ExargHandle) {
    let mut arg = nvim_eap_get_arg_local(eap);
    let skip = nvim_eap_get_skip_local(eap) != 0;
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let cmd_echo = nvim_docmd_cmd_echo();

    let mut atstart = true;
    let mut need_clear = true;
    let did_emsg_before = did_emsg_get();
    let called_emsg_before = called_emsg_get();

    let evalarg = nvim_evalarg_alloc_from_eap(eap, skip);

    if skip {
        nvim_emsg_skip_inc();
    }

    loop {
        let ch = get_byte(arg);
        if ch == 0 || ch == b'|' || ch == b'\n' || nvim_got_int() != 0 {
            break;
        }

        // If eval1() causes an error message the text from the command may
        // still need to be cleared. E.g., "echo 22,44".
        nvim_set_need_clr_eos(1);

        let p = arg;
        let rettv = alloc_typval();
        let eval_ret = eval1(&mut arg, rettv, evalarg);
        if eval_ret == FAIL {
            // Report the invalid expression unless the expression evaluation
            // has been cancelled due to an aborting error, an interrupt, or an
            // exception.
            if !nvim_aborting()
                && nvim_get_did_emsg() == did_emsg_before
                && called_emsg_get() == called_emsg_before
            {
                nvim_semsg_invexpr2(p);
            }
            nvim_set_need_clr_eos(0);
            free_typval(rettv);
            break;
        }
        nvim_set_need_clr_eos(0);

        if !skip {
            let echo_hl_id = ECHO_HL_ID;
            if atstart {
                atstart = false;
                nvim_msg_ext_set_kind(KIND_ECHO.as_ptr() as *const c_char);
                // Call msg_start() after eval1(), evaluating the expression
                // may cause a message to appear.
                if cmdidx == cmd_echo {
                    if nvim_get_msg_didout() == 0 {
                        // Mark the saved text as finishing the line, so that what
                        // follows is displayed on a new line when scrolling back
                        // at the more prompt.
                        nvim_msg_sb_eol();
                    }
                    nvim_msg_start();
                }
            } else if cmdidx == cmd_echo {
                nvim_msg_puts_hl(KIND_SPACE.as_ptr() as *const c_char, echo_hl_id, false);
            }
            let tofree = nvim_encode_tv2echo(rettv);
            nvim_set_msg_ext_append(cmdidx == nvim_docmd_cmd_echon());
            nvim_msg_multiline_cstr(tofree, echo_hl_id, true, false, &mut need_clear);
            xfree(tofree as *mut c_void);
        }
        tv_clear(rettv);
        free_typval(rettv);
        arg = skipwhite(arg);
    }

    nvim_eap_set_nextcmd_checked(eap, arg);
    nvim_evalarg_clear_and_free(evalarg, eap);

    if skip {
        nvim_emsg_skip_dec();
    } else {
        // remove text that may still be there from the command
        if need_clear {
            nvim_msg_clr_eos();
        }
        if cmdidx == cmd_echo {
            nvim_msg_end();
        }
    }
}

/// FFI export for ex_echo.
///
/// # Safety
/// See `ex_echo_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_echo(eap: ExargHandle) {
    ex_echo_impl(eap);
}

// =============================================================================
// ex_execute
// =============================================================================

/// Implementation of ex_execute: `:execute`, `:echomsg`, `:echoerr`.
///
/// Migrated from C `ex_execute` in eval_shim.c.
///
/// # Safety
/// - `eap` must be a valid exarg_T pointer
pub unsafe fn ex_execute_impl(eap: ExargHandle) {
    let mut arg = nvim_eap_get_arg_local(eap);
    let skip = nvim_eap_get_skip_local(eap) != 0;
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let cmd_execute = nvim_docmd_cmd_execute();
    let cmd_echomsg = nvim_docmd_cmd_echomsg();
    let cmd_echoerr = nvim_docmd_cmd_echoerr();

    let ga = nvim_ga_alloc_execute();
    let mut ret = OK;

    if skip {
        nvim_emsg_skip_inc();
    }

    loop {
        let ch = get_byte(arg);
        if ch == 0 || ch == b'|' || ch == b'\n' {
            break;
        }

        let rettv = alloc_typval();
        ret = nvim_eval1_emsg_wrapper(&mut arg, rettv, eap);
        if ret == FAIL {
            free_typval(rettv);
            break;
        }

        if !skip {
            // Get string representation of the typval
            let tv_type = nvim_tv_get_type(rettv);
            let argstr: *const c_char = if cmdidx == cmd_execute {
                nvim_eval_tv_get_str(rettv) as *const c_char
            } else if tv_type == VAR_STRING {
                nvim_encode_tv2echo(rettv) as *const c_char
            } else {
                nvim_encode_tv2string_wrapper(rettv) as *const c_char
            };

            let len = cstr_len(argstr);

            nvim_ga_grow_wrapper(ga, len as c_int + 2);
            if !nvim_ga_is_empty_execute(ga) {
                nvim_ga_append_space(ga);
            }
            nvim_ga_append_str_len(ga, argstr, len as c_int);

            if cmdidx != cmd_execute {
                xfree(argstr as *mut c_void);
            }
        }

        tv_clear(rettv);
        free_typval(rettv);
        arg = skipwhite(arg);
    }

    if ret != FAIL && !nvim_ga_data_is_null(ga) {
        let data = nvim_ga_get_data(ga);
        let echo_hl_id = ECHO_HL_ID;
        if cmdidx == cmd_echomsg {
            nvim_msg_ext_set_kind(KIND_ECHOMSG.as_ptr() as *const c_char);
            nvim_msg_echomsg(data, echo_hl_id);
        } else if cmdidx == cmd_echoerr {
            // We don't want to abort following commands, restore did_emsg.
            let save_did_emsg = nvim_get_did_emsg();
            nvim_emsg_multiline_echoerr(data);
            if nvim_get_force_abort() == 0 {
                nvim_set_did_emsg(save_did_emsg);
            }
        } else if cmdidx == cmd_execute {
            nvim_do_cmdline_execute(data, eap);
        }
    }

    nvim_ga_clear_and_free(ga);

    if skip {
        nvim_emsg_skip_dec();
    }

    nvim_eap_set_nextcmd_checked(eap, arg);
}

/// FFI export for ex_execute.
///
/// # Safety
/// See `ex_execute_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_execute(eap: ExargHandle) {
    ex_execute_impl(eap);
}
