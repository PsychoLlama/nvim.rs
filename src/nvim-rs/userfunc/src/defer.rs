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
    fn nvim_handle_defer_one_impl(fc: *mut c_void);
    fn nvim_ex_defer_inner_impl(
        name: *mut c_char,
        arg: *mut *mut c_char,
        partial: *const c_void,
        evalarg: *mut c_void,
    ) -> c_int;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_emsg_defer_not_in_function();
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

/// Invoke deferred functions for one funccal.
///
/// # Safety
/// `funccal` must be a valid `funccall_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_defer_one(funccal: *mut c_void) {
    unsafe { nvim_handle_defer_one_impl(funccal) };
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
        unsafe { nvim_handle_defer_one_impl(fc) };
        fc = unsafe { nvim_fc_get_caller(fc) };
    }

    // Walk funccal_stack entries
    let mut fce = unsafe { nvim_funccal_stack_head() };
    while !fce.is_null() {
        let mut fc2 = unsafe { nvim_funccal_entry_top(fce) };
        while !fc2.is_null() {
            unsafe { nvim_handle_defer_one_impl(fc2) };
            fc2 = unsafe { nvim_fc_get_caller(fc2) };
        }
        fce = unsafe { nvim_funccal_entry_next(fce) };
    }
}

// =============================================================================
// ex_defer_inner
// =============================================================================

/// Core part of `:defer func(arg)`. `arg` points to `(` and is advanced.
/// Returns OK or FAIL.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_defer_inner(
    name: *mut c_char,
    arg: *mut *mut c_char,
    partial: *const c_void,
    evalarg: *mut c_void,
) -> c_int {
    unsafe { nvim_ex_defer_inner_impl(name, arg, partial, evalarg) }
}
