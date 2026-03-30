//! Garbage collection support for VimL function calls.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 5.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    fn nvim_fc_referenced_impl(fc: *const c_void) -> c_int;
    fn nvim_can_free_funccal_impl(fc: *mut c_void, copy_id: c_int) -> c_int;
    fn nvim_free_unref_funccal_impl(copy_id: c_int, testing: c_int) -> c_int;
    fn nvim_set_ref_in_previous_funccal_impl(copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_funccal_impl(fc: *mut c_void, copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_call_stack_impl(copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_functions_impl(copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_func_args_impl(copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_func_impl(name: *mut c_char, fp: *mut c_void, copy_id: c_int) -> c_int;
}

// =============================================================================
// fc_referenced
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_fc_referenced(fc: *const c_void) -> c_int {
    unsafe { nvim_fc_referenced_impl(fc) }
}

// =============================================================================
// can_free_funccal
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_can_free_funccal(fc: *mut c_void, copy_id: c_int) -> c_int {
    unsafe { nvim_can_free_funccal_impl(fc, copy_id) }
}

// =============================================================================
// free_unref_funccal
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_free_unref_funccal(copy_id: c_int, testing: c_int) -> c_int {
    unsafe { nvim_free_unref_funccal_impl(copy_id, testing) }
}

// =============================================================================
// set_ref_in_previous_funccal
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_previous_funccal(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_previous_funccal_impl(copy_id) }
}

// =============================================================================
// set_ref_in_funccal (static helper)
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_funccal(fc: *mut c_void, copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_funccal_impl(fc, copy_id) }
}

// =============================================================================
// set_ref_in_call_stack
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_call_stack(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_call_stack_impl(copy_id) }
}

// =============================================================================
// set_ref_in_functions
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_functions(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_functions_impl(copy_id) }
}

// =============================================================================
// set_ref_in_func_args
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_func_args(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_func_args_impl(copy_id) }
}

// =============================================================================
// set_ref_in_func
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_func(
    name: *mut c_char,
    fp: *mut c_void,
    copy_id: c_int,
) -> c_int {
    unsafe { nvim_set_ref_in_func_impl(name, fp, copy_id) }
}
