//! Funccal scope accessors and ex_return for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 6.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    fn nvim_get_funccal_impl() -> *mut c_void;
    fn nvim_get_funccal_local_dict_impl() -> *mut c_void;
    fn nvim_get_funccal_local_ht_impl() -> *mut c_void;
    fn nvim_get_funccal_local_var_impl() -> *mut c_void;
    fn nvim_get_funccal_args_dict_impl() -> *mut c_void;
    fn nvim_get_funccal_args_ht_impl() -> *mut c_void;
    fn nvim_get_funccal_args_var_impl() -> *mut c_void;
    fn nvim_list_func_vars_impl(first: *mut c_int);
    fn nvim_get_current_funccal_dict_impl(ht: *mut c_void) -> *mut c_void;
    fn nvim_find_hi_in_scoped_ht_impl(name: *const c_char, pht: *mut *mut c_void) -> *mut c_void;
    fn nvim_find_var_in_scoped_ht_impl(
        name: *const c_char,
        namelen: usize,
        no_autoload: c_int,
    ) -> *mut c_void;
    fn nvim_ex_return_impl(eap: *mut c_void);
    fn nvim_do_return_impl(
        eap: *mut c_void,
        reanimate: c_int,
        is_cmd: c_int,
        rettv: *mut c_void,
    ) -> c_int;
    fn nvim_get_return_cmd_impl(rettv: *mut c_void) -> *mut c_char;
}

// =============================================================================
// get_funccal
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_funccal() -> *mut c_void {
    unsafe { nvim_get_funccal_impl() }
}

// =============================================================================
// get_funccal_local_dict
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_funccal_local_dict() -> *mut c_void {
    unsafe { nvim_get_funccal_local_dict_impl() }
}

// =============================================================================
// get_funccal_local_ht
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_funccal_local_ht() -> *mut c_void {
    unsafe { nvim_get_funccal_local_ht_impl() }
}

// =============================================================================
// get_funccal_local_var
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_funccal_local_var() -> *mut c_void {
    unsafe { nvim_get_funccal_local_var_impl() }
}

// =============================================================================
// get_funccal_args_dict
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_funccal_args_dict() -> *mut c_void {
    unsafe { nvim_get_funccal_args_dict_impl() }
}

// =============================================================================
// get_funccal_args_ht
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_funccal_args_ht() -> *mut c_void {
    unsafe { nvim_get_funccal_args_ht_impl() }
}

// =============================================================================
// get_funccal_args_var
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_funccal_args_var() -> *mut c_void {
    unsafe { nvim_get_funccal_args_var_impl() }
}

// =============================================================================
// list_func_vars
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_list_func_vars(first: *mut c_int) {
    unsafe { nvim_list_func_vars_impl(first) };
}

// =============================================================================
// get_current_funccal_dict
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_current_funccal_dict(ht: *mut c_void) -> *mut c_void {
    unsafe { nvim_get_current_funccal_dict_impl(ht) }
}

// =============================================================================
// find_hi_in_scoped_ht
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_find_hi_in_scoped_ht(
    name: *const c_char,
    pht: *mut *mut c_void,
) -> *mut c_void {
    unsafe { nvim_find_hi_in_scoped_ht_impl(name, pht) }
}

// =============================================================================
// find_var_in_scoped_ht
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_find_var_in_scoped_ht(
    name: *const c_char,
    namelen: usize,
    no_autoload: c_int,
) -> *mut c_void {
    unsafe { nvim_find_var_in_scoped_ht_impl(name, namelen, no_autoload) }
}

// =============================================================================
// ex_return
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_ex_return(eap: *mut c_void) {
    unsafe { nvim_ex_return_impl(eap) };
}

// =============================================================================
// do_return
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_do_return(
    eap: *mut c_void,
    reanimate: c_int,
    is_cmd: c_int,
    rettv: *mut c_void,
) -> c_int {
    unsafe { nvim_do_return_impl(eap, reanimate, is_cmd, rettv) }
}

// =============================================================================
// get_return_cmd
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_get_return_cmd(rettv: *mut c_void) -> *mut c_char {
    unsafe { nvim_get_return_cmd_impl(rettv) }
}
