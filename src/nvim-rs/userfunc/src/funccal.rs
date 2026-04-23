//! Funccal management, ex_delfunction, and helper functions for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 7.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    fn nvim_free_funccal_impl(fc: *mut c_void);
    fn nvim_free_funccal_contents_impl(fc: *mut c_void);
    fn nvim_cleanup_function_call_impl(fc: *mut c_void);
    fn nvim_funccal_unref_impl(fc: *mut c_void, fp: *mut c_void, force: c_int);
    fn nvim_create_funccal_impl(fp: *mut c_void, rettv: *mut c_void) -> *mut c_void;
    fn nvim_save_funccal_impl(entry: *mut c_void);
    fn nvim_restore_funccal_impl();
    fn nvim_ex_delfunction_impl(eap: *mut c_void);
    fn nvim_emsg_funcname_impl(errmsg: *const c_char, name: *const c_char);
    fn nvim_user_func_error_impl(error: c_int, name: *const c_char, found_var: c_int);

    // current_funccal access (for inlining remove_funccal)
    fn get_current_funccal() -> *mut c_void;
    fn set_current_funccal(fc: *mut c_void);
    fn nvim_fc_get_caller(fc: *mut c_void) -> *mut c_void;
}

// =============================================================================
// free_funccal
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_free_funccal(fc: *mut c_void) {
    unsafe { nvim_free_funccal_impl(fc) };
}

// =============================================================================
// free_funccal_contents
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_free_funccal_contents(fc: *mut c_void) {
    unsafe { nvim_free_funccal_contents_impl(fc) };
}

// =============================================================================
// cleanup_function_call
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_function_call(fc: *mut c_void) {
    unsafe { nvim_cleanup_function_call_impl(fc) };
}

// =============================================================================
// funccal_unref
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_funccal_unref(fc: *mut c_void, fp: *mut c_void, force: c_int) {
    unsafe { nvim_funccal_unref_impl(fc, fp, force) };
}

// =============================================================================
// create_funccal
// =============================================================================

#[unsafe(export_name = "create_funccal")]
pub unsafe extern "C" fn rs_create_funccal(fp: *mut c_void, rettv: *mut c_void) -> *mut c_void {
    unsafe { nvim_create_funccal_impl(fp, rettv) }
}

// =============================================================================
// remove_funccal
// =============================================================================

#[unsafe(export_name = "remove_funccal")]
pub unsafe extern "C" fn rs_remove_funccal() {
    let fc = unsafe { get_current_funccal() };
    let caller = unsafe { nvim_fc_get_caller(fc) };
    unsafe { set_current_funccal(caller) };
    unsafe { rs_free_funccal(fc) };
}

// =============================================================================
// save_funccal
// =============================================================================

#[unsafe(export_name = "save_funccal")]
pub unsafe extern "C" fn rs_save_funccal(entry: *mut c_void) {
    unsafe { nvim_save_funccal_impl(entry) };
}

// =============================================================================
// restore_funccal
// =============================================================================

#[unsafe(export_name = "restore_funccal")]
pub unsafe extern "C" fn rs_restore_funccal() {
    unsafe { nvim_restore_funccal_impl() };
}

// =============================================================================
// ex_delfunction
// =============================================================================

#[unsafe(export_name = "ex_delfunction")]
pub unsafe extern "C" fn rs_ex_delfunction(eap: *mut c_void) {
    unsafe { nvim_ex_delfunction_impl(eap) };
}

// =============================================================================
// emsg_funcname
// =============================================================================

#[unsafe(export_name = "emsg_funcname")]
pub unsafe extern "C" fn rs_emsg_funcname(errmsg: *const c_char, name: *const c_char) {
    unsafe { nvim_emsg_funcname_impl(errmsg, name) };
}

// =============================================================================
// user_func_error
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_user_func_error(error: c_int, name: *const c_char, found_var: c_int) {
    unsafe { nvim_user_func_error_impl(error, name, found_var) };
}
