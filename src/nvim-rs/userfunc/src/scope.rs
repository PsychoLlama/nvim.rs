//! Funccal scope accessors and ex_return for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 6.
//! Phase 13: Several scope impl shims inlined directly.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    // Phase 13: still calling C for complex shims
    fn nvim_find_hi_in_scoped_ht_impl(name: *const c_char, pht: *mut *mut c_void) -> *mut c_void;
    fn nvim_find_var_in_scoped_ht_impl(
        name: *const c_char,
        namelen: usize,
        no_autoload: c_int,
    ) -> *mut c_void;
    // nvim_ex_return_impl moved to Rust (funccal.rs Phase 28)
    fn nvim_do_return_impl(
        eap: *mut c_void,
        reanimate: c_int,
        is_cmd: c_int,
        rettv: *mut c_void,
    ) -> c_int;
    fn nvim_get_return_cmd_impl(rettv: *mut c_void) -> *mut c_char;

    // Phase 13: accessors for inlining scope shims
    fn nvim_get_current_funccal() -> *mut c_void;
    fn nvim_get_debug_backtrace_level() -> c_int;
    fn nvim_set_debug_backtrace_level(v: c_int);
    fn nvim_fc_get_caller(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_vars_dv_refcount(fc: *const c_void) -> c_int;
    fn nvim_fc_l_vars_dict(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_avars_dict(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_vars_var_ptr(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_avars_var_ptr(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_vars_ht(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_avars_ht(fc: *mut c_void) -> *mut c_void;
    fn nvim_list_hashtable_vars(ht: *mut c_void, prefix: *const c_char, first: *mut c_int);
}

// =============================================================================
// get_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_get_funccal_impl.

#[unsafe(export_name = "get_funccal")]
pub unsafe extern "C" fn rs_get_funccal() -> *mut c_void {
    let mut funccal = unsafe { nvim_get_current_funccal() };
    let level = unsafe { nvim_get_debug_backtrace_level() };
    if level > 0 {
        for i in 0..level {
            let next = unsafe { nvim_fc_get_caller(funccal) };
            if next.is_null() {
                unsafe { nvim_set_debug_backtrace_level(i) };
                break;
            }
            funccal = next;
        }
    }
    funccal
}

// =============================================================================
// get_funccal_local_dict
// =============================================================================
//
// Phase 13: inlined from nvim_get_funccal_local_dict_impl.

#[unsafe(export_name = "get_funccal_local_dict")]
pub unsafe extern "C" fn rs_get_funccal_local_dict() -> *mut c_void {
    let cur = unsafe { nvim_get_current_funccal() };
    if cur.is_null() || unsafe { nvim_fc_l_vars_dv_refcount(cur) } == 0 {
        return std::ptr::null_mut();
    }
    let fc = unsafe { rs_get_funccal() };
    unsafe { nvim_fc_l_vars_dict(fc) }
}

// =============================================================================
// get_funccal_local_ht
// =============================================================================
//
// Phase 13: inlined from nvim_get_funccal_local_ht_impl.

#[unsafe(export_name = "get_funccal_local_ht")]
pub unsafe extern "C" fn rs_get_funccal_local_ht() -> *mut c_void {
    let d = unsafe { rs_get_funccal_local_dict() };
    if d.is_null() {
        std::ptr::null_mut()
    } else {
        let fc = unsafe { rs_get_funccal() };
        unsafe { nvim_fc_l_vars_ht(fc) }
    }
}

// =============================================================================
// get_funccal_local_var
// =============================================================================
//
// Phase 13: inlined from nvim_get_funccal_local_var_impl.

#[unsafe(export_name = "get_funccal_local_var")]
pub unsafe extern "C" fn rs_get_funccal_local_var() -> *mut c_void {
    let cur = unsafe { nvim_get_current_funccal() };
    if cur.is_null() || unsafe { nvim_fc_l_vars_dv_refcount(cur) } == 0 {
        return std::ptr::null_mut();
    }
    let fc = unsafe { rs_get_funccal() };
    unsafe { nvim_fc_l_vars_var_ptr(fc) }
}

// =============================================================================
// get_funccal_args_dict
// =============================================================================
//
// Phase 13: inlined from nvim_get_funccal_args_dict_impl.

#[unsafe(export_name = "get_funccal_args_dict")]
pub unsafe extern "C" fn rs_get_funccal_args_dict() -> *mut c_void {
    let cur = unsafe { nvim_get_current_funccal() };
    if cur.is_null() || unsafe { nvim_fc_l_vars_dv_refcount(cur) } == 0 {
        return std::ptr::null_mut();
    }
    let fc = unsafe { rs_get_funccal() };
    unsafe { nvim_fc_l_avars_dict(fc) }
}

// =============================================================================
// get_funccal_args_ht
// =============================================================================
//
// Phase 13: inlined from nvim_get_funccal_args_ht_impl.

#[unsafe(export_name = "get_funccal_args_ht")]
pub unsafe extern "C" fn rs_get_funccal_args_ht() -> *mut c_void {
    let d = unsafe { rs_get_funccal_args_dict() };
    if d.is_null() {
        std::ptr::null_mut()
    } else {
        let fc = unsafe { rs_get_funccal() };
        unsafe { nvim_fc_l_avars_ht(fc) }
    }
}

// =============================================================================
// get_funccal_args_var
// =============================================================================
//
// Phase 13: inlined from nvim_get_funccal_args_var_impl.

#[unsafe(export_name = "get_funccal_args_var")]
pub unsafe extern "C" fn rs_get_funccal_args_var() -> *mut c_void {
    let cur = unsafe { nvim_get_current_funccal() };
    if cur.is_null() || unsafe { nvim_fc_l_vars_dv_refcount(cur) } == 0 {
        return std::ptr::null_mut();
    }
    let fc = unsafe { rs_get_funccal() };
    unsafe { nvim_fc_l_avars_var_ptr(fc) }
}

// =============================================================================
// list_func_vars
// =============================================================================
//
// Phase 13: inlined from nvim_list_func_vars_impl.

#[unsafe(export_name = "list_func_vars")]
pub unsafe extern "C" fn rs_list_func_vars(first: *mut c_int) {
    let cur = unsafe { nvim_get_current_funccal() };
    if !cur.is_null() && unsafe { nvim_fc_l_vars_dv_refcount(cur) } > 0 {
        let ht = unsafe { nvim_fc_l_vars_ht(cur) };
        unsafe { nvim_list_hashtable_vars(ht, c"l:".as_ptr(), first) };
    }
}

// =============================================================================
// get_current_funccal_dict
// =============================================================================
//
// Phase 13: inlined from nvim_get_current_funccal_dict_impl.

#[unsafe(export_name = "get_current_funccal_dict")]
pub unsafe extern "C" fn rs_get_current_funccal_dict(ht: *mut c_void) -> *mut c_void {
    let cur = unsafe { nvim_get_current_funccal() };
    if !cur.is_null() {
        let local_ht = unsafe { nvim_fc_l_vars_ht(cur) };
        if ht == local_ht {
            return unsafe { nvim_fc_l_vars_dict(cur) };
        }
    }
    std::ptr::null_mut()
}

// =============================================================================
// find_hi_in_scoped_ht
// =============================================================================
//
// Cannot inline: requires HASHITEM_EMPTY macro and modifies current_funccal.

#[unsafe(export_name = "find_hi_in_scoped_ht")]
pub unsafe extern "C" fn rs_find_hi_in_scoped_ht(
    name: *const c_char,
    pht: *mut *mut c_void,
) -> *mut c_void {
    unsafe { nvim_find_hi_in_scoped_ht_impl(name, pht) }
}

// =============================================================================
// find_var_in_scoped_ht
// =============================================================================
//
// Cannot inline: requires modifying current_funccal and calls find_var_ht.

#[unsafe(export_name = "find_var_in_scoped_ht")]
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

#[unsafe(export_name = "ex_return")]
pub unsafe extern "C" fn rs_ex_return(eap: *mut c_void) {
    // Phase 28: calls Rust implementation in funccal.rs
    unsafe { crate::funccal::nvim_ex_return_impl(eap) };
}

// =============================================================================
// do_return
// =============================================================================

#[unsafe(export_name = "do_return")]
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

#[unsafe(export_name = "get_return_cmd")]
pub unsafe extern "C" fn rs_get_return_cmd(rettv: *mut c_void) -> *mut c_char {
    unsafe { nvim_get_return_cmd_impl(rettv) }
}
