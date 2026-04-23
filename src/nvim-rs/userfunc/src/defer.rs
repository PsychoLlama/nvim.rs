//! Defer infrastructure for VimL `:defer` command.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 3.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    fn get_current_funccal() -> *mut c_void;
    fn nvim_fc_get_caller(fc: *mut c_void) -> *mut c_void;
    fn nvim_funccal_stack_head() -> *mut c_void;
    fn nvim_funccal_entry_top(fce: *mut c_void) -> *mut c_void;
    fn nvim_funccal_entry_next(fce: *mut c_void) -> *mut c_void;
    fn nvim_fc_defer_append(
        fc: *mut c_void,
        name: *mut c_char,
        argcount: c_int,
        argvars: *mut c_void,
    );
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // Phase 30: for rs_handle_defer_one migration
    fn nvim_fc_defer_ga_len(fc: *const c_void) -> c_int;
    fn nvim_fc_defer_item_take_name(fc: *mut c_void, idx: c_int) -> *mut c_char;
    fn nvim_fc_defer_item_argcount(fc: *const c_void, idx: c_int) -> c_int;
    fn nvim_fc_defer_item_argvars(fc: *mut c_void, idx: c_int) -> *mut c_void;
    fn nvim_fc_defer_ga_clear(fc: *mut c_void);
    fn exception_state_save(estate: *mut c_void);
    fn exception_state_restore(estate: *mut c_void);
    fn exception_state_clear();
    fn nvim_tv_set_unknown(tv: *mut c_void);
    fn nvim_emsg_defer_not_in_function();

    // Phase 24: for rs_ex_defer_inner migration
    fn nvim_partial_get_dict(pt: *const c_void) -> *mut c_void;
    fn nvim_partial_get_argc(pt: *const c_void) -> c_int;
    fn nvim_partial_get_argv(pt: *const c_void) -> *mut c_void;
    fn tv_copy(from: *const c_void, to: *mut c_void);
    fn tv_clear(tv: *mut c_void);
    fn find_func(name: *const c_char) -> *mut c_void;
    fn check_user_func_argcount(fp: *mut c_void, argcount: c_int) -> c_int;
    fn nvim_emsg_cannot_use_partial_with_dict();
    fn nvim_check_defer_builtin(name: *const c_char, argcount: c_int) -> c_int;
    // get_func_arguments is Rust (parsing.rs)
    fn get_func_arguments(
        arg: *mut *mut c_char,
        evalarg: *mut c_void,
        partial_argc: c_int,
        argvars: *mut c_void,
        argcount: *mut c_int,
    ) -> c_int;
    fn rs_builtin_function(name: *const c_char, len: c_int) -> c_int;
    fn rs_user_func_error(error: c_int, name: *const c_char, found_var: c_int);
    // add_defer is Rust (defer.rs), exported as "add_defer"
}

// =============================================================================
// can_add_defer
// =============================================================================

/// Return true if currently inside a function call.
/// Gives an error message and returns false when not.
#[unsafe(export_name = "can_add_defer")]
pub unsafe extern "C" fn rs_can_add_defer() -> c_int {
    if unsafe { get_current_funccal() }.is_null() {
        unsafe { nvim_emsg_defer_not_in_function() };
        return 0;
    }
    1
}

// =============================================================================
// add_defer
// =============================================================================

/// Add a deferred call for "name" with arguments "argvars[argcount]".
/// Consumes argvars[]. Caller must check that current_funccal is not NULL.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string; `argvars` must be a valid
/// typval_T array of at least `argcount` elements.
#[unsafe(export_name = "add_defer")]
pub unsafe extern "C" fn rs_add_defer(name: *mut c_char, argcount: c_int, argvars: *mut c_void) {
    let fc = unsafe { get_current_funccal() };
    let saved_name = unsafe { xstrdup(name) };
    unsafe { nvim_fc_defer_append(fc, saved_name, argcount, argvars) };
}

// =============================================================================
// handle_defer_one
// =============================================================================

// Size of funcexe_T in bytes (must match C's sizeof(funcexe_T) = 64).
const SIZEOF_FUNCEXE: usize = 64;
// Offset of fe_evaluate (bool) within funcexe_T.
const FE_EVALUATE_OFFSET: usize = 24;
// Size of typval_T in bytes (must match C's sizeof(typval_T) = 16).
const SIZEOF_TYPVAL_DEFER: usize = 16;
// Size of exception_state_T in bytes (must match C's sizeof(exception_state_T) = 24).
const SIZEOF_EXCEPTION_STATE: usize = 24;

/// Invoke deferred functions for one funccal.
///
/// Phase 30: inlined from rs_handle_defer_one.
///
/// # Safety
/// `funccal` must be a valid `funccall_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_defer_one(funccal: *mut c_void) {
    let len = unsafe { nvim_fc_defer_ga_len(funccal) };
    let mut idx = len - 1;
    while idx >= 0 {
        let name = unsafe { nvim_fc_defer_item_take_name(funccal, idx) };
        if name.is_null() {
            // already being called, can happen if function does ":qa"
            idx -= 1;
            continue;
        }
        let argcount = unsafe { nvim_fc_defer_item_argcount(funccal, idx) };
        let argvars = unsafe { nvim_fc_defer_item_argvars(funccal, idx) };

        // funcexe_T funcexe = { .fe_evaluate = true }
        let mut funcexe = [0u8; SIZEOF_FUNCEXE];
        funcexe[FE_EVALUATE_OFFSET] = 1u8;
        let funcexe_ptr = funcexe.as_mut_ptr().cast::<c_void>();

        // typval_T rettv; rettv.v_type = VAR_UNKNOWN
        let mut rettv = [0u8; SIZEOF_TYPVAL_DEFER];
        let rettv_ptr = rettv.as_mut_ptr().cast::<c_void>();
        unsafe { nvim_tv_set_unknown(rettv_ptr) };

        // exception_state_T estate; exception_state_save(&estate); exception_state_clear();
        let mut estate = [0u8; SIZEOF_EXCEPTION_STATE];
        let estate_ptr = estate.as_mut_ptr().cast::<c_void>();
        unsafe { exception_state_save(estate_ptr) };
        unsafe { exception_state_clear() };

        unsafe {
            crate::funccal::rs_call_func(name, -1, rettv_ptr, argcount, argvars, funcexe_ptr)
        };

        unsafe { exception_state_restore(estate_ptr) };

        unsafe { tv_clear(rettv_ptr) };
        unsafe { xfree(name.cast::<c_void>()) };

        let mut i = argcount - 1;
        while i >= 0 {
            let slot = unsafe {
                argvars
                    .cast::<u8>()
                    .add(i as usize * SIZEOF_TYPVAL_DEFER)
                    .cast::<c_void>()
            };
            unsafe { tv_clear(slot) };
            i -= 1;
        }

        idx -= 1;
    }
    unsafe { nvim_fc_defer_ga_clear(funccal) };
}

// =============================================================================
// invoke_all_defer
// =============================================================================

/// Called when exiting: call all defer functions.
#[unsafe(export_name = "invoke_all_defer")]
pub unsafe extern "C" fn rs_invoke_all_defer() {
    // Walk current_funccal chain
    let mut fc = unsafe { get_current_funccal() };
    while !fc.is_null() {
        unsafe { rs_handle_defer_one(fc) };
        fc = unsafe { nvim_fc_get_caller(fc) };
    }

    // Walk funccal_stack entries
    let mut fce = unsafe { nvim_funccal_stack_head() };
    while !fce.is_null() {
        let mut fc2 = unsafe { nvim_funccal_entry_top(fce) };
        while !fc2.is_null() {
            unsafe { rs_handle_defer_one(fc2) };
            fc2 = unsafe { nvim_fc_get_caller(fc2) };
        }
        fce = unsafe { nvim_funccal_entry_next(fce) };
    }
}

// =============================================================================
// ex_defer_inner
// =============================================================================

// Size of typval_T in bytes (matches C's sizeof(typval_T)).
const SIZEOF_TYPVAL: usize = 16;
// MAX_FUNC_ARGS must match C's MAX_FUNC_ARGS.
const MAX_FUNC_ARGS: usize = 20;
const OK: c_int = 1;
const FAIL: c_int = 0;
// FCERR_UNKNOWN = 0: no-error return from check_user_func_argcount.
const FCERR_UNKNOWN: c_int = 0;

/// Core part of `:defer func(arg)`. `arg` points to `(` and is advanced.
/// Returns OK or FAIL.
///
/// Phase 24: migrated from C `nvim_ex_defer_inner_impl`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ex_defer_inner(
    name: *mut c_char,
    arg: *mut *mut c_char,
    partial: *const c_void,
    evalarg: *mut c_void,
) -> c_int {
    // Stack buffer: typval_T argvars[MAX_FUNC_ARGS + 1]
    let mut argvars = [0u8; (MAX_FUNC_ARGS + 1) * SIZEOF_TYPVAL];
    let argvars_ptr = argvars.as_mut_ptr().cast::<c_void>();

    let mut partial_argc: c_int = 0;
    let mut argcount: c_int = 0;

    // Must be inside a function call.
    if unsafe { get_current_funccal() }.is_null() {
        unsafe { nvim_emsg_defer_not_in_function() };
        return FAIL;
    }

    // Handle partial arguments.
    if !partial.is_null() {
        if !unsafe { nvim_partial_get_dict(partial) }.is_null() {
            unsafe { nvim_emsg_cannot_use_partial_with_dict() };
            return FAIL;
        }
        let argc = unsafe { nvim_partial_get_argc(partial) };
        if argc > 0 {
            partial_argc = argc;
            let pt_argv = unsafe { nvim_partial_get_argv(partial) };
            for i in 0..argc as usize {
                let src = unsafe { pt_argv.cast::<u8>().add(i * SIZEOF_TYPVAL) }.cast::<c_void>();
                let dst =
                    unsafe { argvars_ptr.cast::<u8>().add(i * SIZEOF_TYPVAL) }.cast::<c_void>();
                unsafe { tv_copy(src, dst) };
            }
        }
    }

    // Parse function arguments from `(...)`.
    let argvars_after_partial = unsafe {
        argvars_ptr
            .cast::<u8>()
            .add(partial_argc as usize * SIZEOF_TYPVAL)
    }
    .cast::<c_void>();
    let r = unsafe {
        get_func_arguments(
            arg,
            evalarg,
            partial_argc,
            argvars_after_partial,
            &raw mut argcount,
        )
    };
    argcount += partial_argc;

    let mut r = r;
    if r == OK {
        if unsafe { rs_builtin_function(name, -1) } != 0 {
            // Builtin function: check arity.
            if unsafe { nvim_check_defer_builtin(name, argcount) } == -1 {
                r = FAIL;
            }
        } else {
            // User function: check arity if it exists.
            let ufunc = unsafe { find_func(name) };
            if !ufunc.is_null() {
                let error = unsafe { check_user_func_argcount(ufunc, argcount) };
                if error != FCERR_UNKNOWN {
                    unsafe { rs_user_func_error(error, name, 0) };
                    r = FAIL;
                }
            }
        }
    }

    if r == FAIL {
        let mut i = argcount - 1;
        while i >= 0 {
            let slot = unsafe { argvars_ptr.cast::<u8>().add(i as usize * SIZEOF_TYPVAL) }
                .cast::<c_void>();
            unsafe { tv_clear(slot) };
            i -= 1;
        }
        return FAIL;
    }

    unsafe { rs_add_defer(name, argcount, argvars_ptr) };
    OK
}
