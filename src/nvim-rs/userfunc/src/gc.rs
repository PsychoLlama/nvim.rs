//! Garbage collection support for VimL function calls.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 5.
//! Phase 11: nvim_set_ref_in_func_impl inlined into rs_set_ref_in_func.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

// FLEN_FIXED must match the C define
const FLEN_FIXED: usize = 40;

// FCERR_NONE = 5 (matches C enum)
const FCERR_NONE: c_int = 5;

extern "C" {
    fn nvim_fc_referenced_impl(fc: *const c_void) -> c_int;
    fn nvim_can_free_funccal_impl(fc: *mut c_void, copy_id: c_int) -> c_int;
    fn nvim_free_unref_funccal_impl(copy_id: c_int, testing: c_int) -> c_int;
    fn nvim_set_ref_in_previous_funccal_impl(copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_funccal_impl(fc: *mut c_void, copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_call_stack_impl(copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_functions_impl(copy_id: c_int) -> c_int;
    fn nvim_set_ref_in_func_args_impl(copy_id: c_int) -> c_int;

    // Phase 11: For inlining nvim_set_ref_in_func_impl:
    fn nvim_ufunc_get_scoped(fp: *mut c_void) -> *mut c_void;
    fn nvim_fc_get_func(fc: *mut c_void) -> *mut c_void;
    fn find_func(name: *const c_char) -> *mut c_void;
    fn rs_fname_trans_sid(
        name: *const c_char,
        fname_buf: *mut c_char,
        tofree: *mut *mut c_char,
        error: *mut c_int,
    ) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
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

#[unsafe(export_name = "free_unref_funccal")]
pub unsafe extern "C" fn rs_free_unref_funccal(copy_id: c_int, testing: c_int) -> c_int {
    unsafe { nvim_free_unref_funccal_impl(copy_id, testing) }
}

// =============================================================================
// set_ref_in_previous_funccal
// =============================================================================

#[unsafe(export_name = "set_ref_in_previous_funccal")]
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

#[unsafe(export_name = "set_ref_in_call_stack")]
pub unsafe extern "C" fn rs_set_ref_in_call_stack(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_call_stack_impl(copy_id) }
}

// =============================================================================
// set_ref_in_functions
// =============================================================================

#[unsafe(export_name = "set_ref_in_functions")]
pub unsafe extern "C" fn rs_set_ref_in_functions(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_functions_impl(copy_id) }
}

// =============================================================================
// set_ref_in_func_args
// =============================================================================

#[unsafe(export_name = "set_ref_in_func_args")]
pub unsafe extern "C" fn rs_set_ref_in_func_args(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_func_args_impl(copy_id) }
}

// =============================================================================
// set_ref_in_func
// =============================================================================
//
// Phase 11: inlined from nvim_set_ref_in_func_impl (previously a C shim).

#[unsafe(export_name = "set_ref_in_func")]
pub unsafe extern "C" fn rs_set_ref_in_func(
    name: *mut c_char,
    fp_in: *mut c_void,
    copy_id: c_int,
) -> c_int {
    if name.is_null() && fp_in.is_null() {
        return 0; // false
    }

    let mut tofree: *mut c_char = std::ptr::null_mut();
    let mut error: c_int = FCERR_NONE;
    let mut fname_buf = [0u8; FLEN_FIXED + 1];

    let fp = if fp_in.is_null() {
        let fname = unsafe {
            rs_fname_trans_sid(
                name,
                fname_buf.as_mut_ptr().cast::<c_char>(),
                std::ptr::addr_of_mut!(tofree),
                std::ptr::addr_of_mut!(error),
            )
        };
        let found = unsafe { find_func(fname) };
        unsafe { xfree(tofree.cast::<c_void>()) };
        found
    } else {
        fp_in
    };

    let mut abort = false;
    if !fp.is_null() {
        let mut fc = unsafe { nvim_ufunc_get_scoped(fp) };
        while !fc.is_null() {
            if unsafe { rs_set_ref_in_funccal(fc, copy_id) } != 0 {
                abort = true;
            }
            let func = unsafe { nvim_fc_get_func(fc) };
            fc = unsafe { nvim_ufunc_get_scoped(func) };
        }
    }

    c_int::from(abort)
}
