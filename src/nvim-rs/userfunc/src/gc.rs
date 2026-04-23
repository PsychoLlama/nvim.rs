//! Garbage collection support for VimL function calls.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 5.
//! Phase 11: nvim_set_ref_in_func_impl inlined into rs_set_ref_in_func.
//! Phase 12: Several GC impl shims inlined directly.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

// FLEN_FIXED must match the C define
const FLEN_FIXED: usize = 40;

// FCERR_NONE = 5 (matches C enum)
const FCERR_NONE: c_int = 5;

// DO_NOT_FREE_CNT = INT_MAX / 2 (matches C define in typval_defs.h)
const DO_NOT_FREE_CNT: c_int = c_int::MAX / 2;

extern "C" {
    fn nvim_set_ref_in_functions_impl(copy_id: c_int) -> c_int;

    // Phase 26: for free_unref_funccal inlining
    fn nvim_set_previous_funccal(fc: *mut c_void);
    fn garbage_collect(testing: c_int) -> bool;
    fn rs_free_funccal_contents(fc: *mut c_void);
    fn nvim_fc_set_caller(fc: *mut c_void, caller: *mut c_void);

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

    // Phase 12: funccall_T field accessors
    fn nvim_get_previous_funccal() -> *mut c_void;
    fn nvim_get_current_funccal() -> *mut c_void;
    fn nvim_get_fc_copyID(fc: *const c_void) -> c_int;
    fn nvim_set_fc_copyID(fc: *mut c_void, v: c_int);
    fn nvim_get_fc_refcount(fc: *const c_void) -> c_int;
    fn nvim_fc_varlist_lv_refcount(fc: *const c_void) -> c_int;
    fn nvim_fc_l_vars_dv_refcount(fc: *const c_void) -> c_int;
    fn nvim_fc_l_avars_dv_refcount(fc: *const c_void) -> c_int;
    fn nvim_fc_varlist_lv_copyID(fc: *const c_void) -> c_int;
    fn nvim_fc_l_vars_dv_copyID(fc: *const c_void) -> c_int;
    fn nvim_fc_l_avars_dv_copyID(fc: *const c_void) -> c_int;
    fn nvim_fc_l_vars_ht(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_avars_ht(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_l_varlist(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_get_caller(fc: *mut c_void) -> *mut c_void;
    fn nvim_funccal_stack_head() -> *mut c_void;
    fn nvim_funccal_entry_next(fce: *mut c_void) -> *mut c_void;
    fn nvim_funccal_entry_top(fce: *mut c_void) -> *mut c_void;
    fn nvim_funcargs_len() -> c_int;
    fn nvim_funcargs_item(i: c_int) -> *mut c_void;

    // Phase 12: functions called by inlined shims
    fn rs_set_ref_in_ht(ht: *mut c_void, copy_id: c_int, list_stack: *mut *mut c_void) -> bool;
    fn rs_set_ref_in_list_items(l: *mut c_void, copy_id: c_int, ht_stack: *mut *mut c_void)
        -> bool;
    fn rs_set_ref_in_item(
        tv: *mut c_void,
        copy_id: c_int,
        ht_stack: *mut *mut c_void,
        list_stack: *mut *mut c_void,
    ) -> bool;
}

// =============================================================================
// fc_referenced
// =============================================================================
//
// Phase 12: inlined from nvim_fc_referenced_impl.

#[no_mangle]
pub unsafe extern "C" fn rs_fc_referenced(fc: *const c_void) -> c_int {
    if fc.is_null() {
        return 0;
    }
    let referenced = unsafe { nvim_fc_varlist_lv_refcount(fc) } != DO_NOT_FREE_CNT
        || unsafe { nvim_fc_l_vars_dv_refcount(fc) } != DO_NOT_FREE_CNT
        || unsafe { nvim_fc_l_avars_dv_refcount(fc) } != DO_NOT_FREE_CNT
        || unsafe { nvim_get_fc_refcount(fc) } > 0;
    c_int::from(referenced)
}

// =============================================================================
// can_free_funccal
// =============================================================================
//
// Phase 12: inlined from nvim_can_free_funccal_impl.

#[no_mangle]
pub unsafe extern "C" fn rs_can_free_funccal(fc: *mut c_void, copy_id: c_int) -> c_int {
    if fc.is_null() {
        return 0;
    }
    let can_free = unsafe { nvim_fc_varlist_lv_copyID(fc) } != copy_id
        && unsafe { nvim_fc_l_vars_dv_copyID(fc) } != copy_id
        && unsafe { nvim_fc_l_avars_dv_copyID(fc) } != copy_id
        && unsafe { nvim_get_fc_copyID(fc) } != copy_id;
    c_int::from(can_free)
}

// =============================================================================
// free_unref_funccal
// =============================================================================
//
// Phase 26: inlined from nvim_free_unref_funccal_impl.
// Iterates previous_funccal linked list, freeing entries that can be freed.

#[unsafe(export_name = "free_unref_funccal")]
pub unsafe extern "C" fn rs_free_unref_funccal(copy_id: c_int, testing: c_int) -> c_int {
    let mut did_free = false;
    let mut did_free_funccal = false;

    // Walk the previous_funccal linked list, deleting freeable entries.
    let mut prev: *mut c_void = std::ptr::null_mut(); // previous fc (or null for head)
    let mut current = unsafe { nvim_get_previous_funccal() };
    while !current.is_null() {
        let next = unsafe { nvim_fc_get_caller(current) };
        if unsafe { rs_can_free_funccal(current, copy_id) } != 0 {
            // Remove from linked list.
            if prev.is_null() {
                unsafe { nvim_set_previous_funccal(next) };
            } else {
                unsafe { nvim_fc_set_caller(prev, next) };
            }
            unsafe { rs_free_funccal_contents(current) };
            did_free = true;
            did_free_funccal = true;
            // Don't advance prev; next entry takes current's position.
        } else {
            prev = current;
        }
        current = next;
    }

    if did_free_funccal {
        unsafe { garbage_collect(testing) };
    }
    c_int::from(did_free)
}

// =============================================================================
// set_ref_in_previous_funccal
// =============================================================================
//
// Phase 12: inlined from nvim_set_ref_in_previous_funccal_impl.

#[unsafe(export_name = "set_ref_in_previous_funccal")]
pub unsafe extern "C" fn rs_set_ref_in_previous_funccal(copy_id: c_int) -> c_int {
    let mut fc = unsafe { nvim_get_previous_funccal() };
    while !fc.is_null() {
        unsafe { nvim_set_fc_copyID(fc, copy_id + 1) };
        let local_ht = unsafe { nvim_fc_l_vars_ht(fc) };
        let args_ht = unsafe { nvim_fc_l_avars_ht(fc) };
        let varlist = unsafe { nvim_fc_l_varlist(fc) };
        if unsafe { rs_set_ref_in_ht(local_ht, copy_id + 1, std::ptr::null_mut()) }
            || unsafe { rs_set_ref_in_ht(args_ht, copy_id + 1, std::ptr::null_mut()) }
            || unsafe { rs_set_ref_in_list_items(varlist, copy_id + 1, std::ptr::null_mut()) }
        {
            return 1; // true
        }
        fc = unsafe { nvim_fc_get_caller(fc) };
    }
    0 // false
}

// =============================================================================
// set_ref_in_funccal (static helper)
// =============================================================================
//
// Phase 12: inlined from nvim_set_ref_in_funccal_impl.

#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_funccal(fc: *mut c_void, copy_id: c_int) -> c_int {
    if fc.is_null() {
        return 0;
    }
    if unsafe { nvim_get_fc_copyID(fc) } != copy_id {
        unsafe { nvim_set_fc_copyID(fc, copy_id) };
        let local_ht = unsafe { nvim_fc_l_vars_ht(fc) };
        let args_ht = unsafe { nvim_fc_l_avars_ht(fc) };
        let varlist = unsafe { nvim_fc_l_varlist(fc) };
        let func_ptr = unsafe { nvim_fc_get_func(fc) };
        if unsafe { rs_set_ref_in_ht(local_ht, copy_id, std::ptr::null_mut()) }
            || unsafe { rs_set_ref_in_ht(args_ht, copy_id, std::ptr::null_mut()) }
            || unsafe { rs_set_ref_in_list_items(varlist, copy_id, std::ptr::null_mut()) }
            || unsafe { rs_set_ref_in_func(std::ptr::null_mut(), func_ptr, copy_id) } != 0
        {
            return 1; // true
        }
    }
    0 // false
}

// =============================================================================
// set_ref_in_call_stack
// =============================================================================
//
// Phase 12: inlined from nvim_set_ref_in_call_stack_impl.

#[unsafe(export_name = "set_ref_in_call_stack")]
pub unsafe extern "C" fn rs_set_ref_in_call_stack(copy_id: c_int) -> c_int {
    // Walk current_funccal linked list
    let mut fc = unsafe { nvim_get_current_funccal() };
    while !fc.is_null() {
        if unsafe { rs_set_ref_in_funccal(fc, copy_id) } != 0 {
            return 1;
        }
        fc = unsafe { nvim_fc_get_caller(fc) };
    }
    // Walk funccal_stack
    let mut entry = unsafe { nvim_funccal_stack_head() };
    while !entry.is_null() {
        let mut fc2 = unsafe { nvim_funccal_entry_top(entry) };
        while !fc2.is_null() {
            if unsafe { rs_set_ref_in_funccal(fc2, copy_id) } != 0 {
                return 1;
            }
            fc2 = unsafe { nvim_fc_get_caller(fc2) };
        }
        entry = unsafe { nvim_funccal_entry_next(entry) };
    }
    0
}

// =============================================================================
// set_ref_in_functions
// =============================================================================
//
// Cannot inline: requires HASHITEM_EMPTY and HI2UF macros for hash table iteration.
// Remains as C impl shim.

#[unsafe(export_name = "set_ref_in_functions")]
pub unsafe extern "C" fn rs_set_ref_in_functions(copy_id: c_int) -> c_int {
    unsafe { nvim_set_ref_in_functions_impl(copy_id) }
}

// =============================================================================
// set_ref_in_func_args
// =============================================================================
//
// Phase 12: inlined from nvim_set_ref_in_func_args_impl.

#[unsafe(export_name = "set_ref_in_func_args")]
pub unsafe extern "C" fn rs_set_ref_in_func_args(copy_id: c_int) -> c_int {
    let len = unsafe { nvim_funcargs_len() };
    for i in 0..len {
        let tv = unsafe { nvim_funcargs_item(i) };
        if unsafe { rs_set_ref_in_item(tv, copy_id, std::ptr::null_mut(), std::ptr::null_mut()) } {
            return 1;
        }
    }
    0
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
