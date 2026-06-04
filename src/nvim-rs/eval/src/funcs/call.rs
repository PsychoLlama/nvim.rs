//! Rust implementation of the VimL `call()` built-in function.
//!
//! Ported from `f_call` in `src/nvim/eval/funcs_shim.c` (Phase 30).
//! The C body has been deleted; this file is the authoritative implementation.

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void};

use crate::typval::TypvalT;

// ─── Constants ────────────────────────────────────────────────────────────────

const VAR_FUNC: c_int = 3;
const VAR_STRING: c_int = 2;
const VAR_PARTIAL: c_int = 9;
const VAR_UNKNOWN: c_int = 0;

/// FAIL = 0 (return value from tv_check_for_* functions)
const FAIL: c_int = 0;

/// `TFN_INT = 1` — may use internal function name
const TFN_INT: c_int = 1;
/// `TFN_QUIET = 2` — do not emit error messages
const TFN_QUIET: c_int = 2;

/// EvalFuncData — opaque union passed by value
type EvalFuncData = *mut c_void;

// ─── FFI imports ─────────────────────────────────────────────────────────────

extern "C" {
    fn tv_check_for_list_arg(args: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_dict_arg(args: *const c_void, idx: c_int) -> c_int;
    fn tv_get_string(tv: *const c_void) -> *const c_char;

    fn rs_partial_name(pt: *const c_void) -> *mut c_char;
    fn nlua_is_table_from_lua(arg: *const c_void) -> bool;
    fn nlua_register_table_as_callable(arg: *const c_void) -> *mut c_char;

    fn trans_function_name(
        pp: *mut *mut c_char,
        skip: bool,
        flags: c_int,
        fdp: *mut c_void,
        partial_out: *mut c_void,
    ) -> *mut c_char;
    fn emsg_funcname(errmsg: *const c_char, name: *const c_char);
    static e_unknown_function_str: [c_char; 0];

    fn func_call(
        name: *const c_char,
        args: *const c_void,
        partial: *const c_void,
        selfdict: *mut c_void,
        rettv: *mut c_void,
    ) -> c_int;
    fn func_unref(name: *mut c_char);
    fn xfree(p: *mut c_void);
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Return a pointer to `argvars[i]` (each typval_T is 16 bytes).
///
/// # Safety
///
/// `argvars` must be valid for at least `i+1` elements.
#[inline]
unsafe fn argvar(argvars: *const c_void, i: usize) -> *const c_void {
    argvars.cast::<u8>().add(i * 16).cast::<c_void>()
}

// ─── f_call ──────────────────────────────────────────────────────────────────

/// `call()` VimL function — invoke a function/partial/Lua-callable with arglist.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *`.
#[export_name = "f_call"]
pub unsafe extern "C" fn rs_f_call(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: EvalFuncData,
) {
    // argvars[1] must be a list
    if tv_check_for_list_arg(argvars, 1) == FAIL {
        return;
    }
    // NULL list is also invalid
    let arg1 = argvar(argvars, 1);
    if (*arg1.cast::<TypvalT>()).vval.v_list.is_null() {
        return;
    }

    let arg0 = argvar(argvars, 0);
    let arg0_type = (*arg0.cast::<TypvalT>()).v_type;

    let mut owned = false;
    let func: *mut c_char;
    let mut partial: *mut c_void = std::ptr::null_mut();

    if arg0_type == VAR_FUNC {
        func = (*arg0.cast::<TypvalT>()).vval.v_string;
    } else if arg0_type == VAR_PARTIAL {
        partial = (*arg0.cast::<TypvalT>()).vval.v_partial;
        func = rs_partial_name(partial);
    } else if nlua_is_table_from_lua(arg0) {
        // TODO(tjdevries): UnifiedCallback
        func = nlua_register_table_as_callable(arg0);
        owned = true;
    } else {
        func = tv_get_string(arg0).cast_mut();
    }

    if func.is_null() || *func == 0 {
        return; // type error, empty name or null function
    }

    let mut tofree: *mut c_char = std::ptr::null_mut();
    if arg0_type == VAR_STRING {
        let mut p = func;
        tofree = trans_function_name(
            &raw mut p,
            false,
            TFN_INT | TFN_QUIET,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if tofree.is_null() {
            emsg_funcname(e_unknown_function_str.as_ptr(), func);
            return;
        }
        // func = tofree — use tofree directly below
    }

    let effective_func: *const c_char = if tofree.is_null() { func } else { tofree };

    // Optional self-dict argument
    let mut selfdict: *mut c_void = std::ptr::null_mut();
    let arg2 = argvar(argvars, 2);
    if (*arg2.cast::<TypvalT>()).v_type != VAR_UNKNOWN {
        if tv_check_for_dict_arg(argvars, 2) == FAIL {
            // goto done — skip func_call, still cleanup
            cleanup(owned, func, tofree);
            return;
        }
        selfdict = (*arg2.cast::<TypvalT>()).vval.v_dict;
    }

    func_call(effective_func, arg1, partial, selfdict, rettv);

    cleanup(owned, func, tofree);
}

/// Handle cleanup of owned func ref and tofree allocation (mirrors `done:` label in C).
///
/// # Safety
///
/// `func` and `tofree` must be valid C strings or null.
#[inline]
unsafe fn cleanup(owned: bool, func: *mut c_char, tofree: *mut c_char) {
    if owned {
        func_unref(func);
    }
    xfree(tofree.cast::<c_void>());
}
