//! Function reference counting and cleanup for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 4.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    fn find_func(name: *const c_char) -> *mut c_void;
    fn nvim_ufunc_decrement_refcount(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_increment_refcount(fp: *mut c_void);
    fn nvim_ufunc_get_calls(fp: *mut c_void) -> c_int;
    fn nvim_func_remove_impl(fp: *mut c_void) -> c_int;

    // Phase 14: For inlining nvim_func_clear_items_impl
    fn nvim_ufunc_get_args_ga(fp: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_get_def_args_ga(fp: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_get_lines_ga(fp: *mut c_void) -> *mut c_void;
    fn nvim_ga_clear_strings_wrapper(ga: *mut c_void);
    fn nvim_ufunc_clear_luaref(fp: *mut c_void);
    fn nvim_ufunc_xfree_tml(fp: *mut c_void);
    fn internal_error(msg: *const c_char);

    // Phase 9: Accessors for inlining nvim_func_clear_impl
    fn nvim_ufunc_get_cleared(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_set_cleared(fp: *mut c_void, v: c_int);
    fn nvim_ufunc_get_scoped(fp: *mut c_void) -> *mut c_void;

    // Phase 9: Accessors for inlining nvim_func_free_impl
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_clear_name_exp(fp: *mut c_void);
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// func_ptr_unref
// =============================================================================

/// Unreference a Function by pointer: decrement refcount and free if zero.
#[unsafe(export_name = "func_ptr_unref")]
pub unsafe extern "C" fn rs_func_ptr_unref(fp: *mut c_void) {
    if fp.is_null() {
        return;
    }
    let new_count = unsafe { nvim_ufunc_decrement_refcount(fp) };
    if new_count <= 0 {
        // Only delete it when it's not being used.
        if unsafe { nvim_ufunc_get_calls(fp) } == 0 {
            unsafe { rs_func_clear_free(fp, 0) };
        }
    }
}

// =============================================================================
// func_ptr_ref
// =============================================================================

/// Increment the reference count of a function.
#[unsafe(export_name = "func_ptr_ref")]
pub unsafe extern "C" fn rs_func_ptr_ref(fp: *mut c_void) {
    if !fp.is_null() {
        unsafe { nvim_ufunc_increment_refcount(fp) };
    }
}

// =============================================================================
// func_unref
// =============================================================================

/// Unreference a Function by name.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string or NULL.
#[unsafe(export_name = "func_unref")]
pub unsafe extern "C" fn rs_func_unref(name: *mut c_char) {
    if name.is_null() || unsafe { super::names::rs_func_name_refcount(name) } == 0 {
        return;
    }
    let fp = unsafe { find_func(name) };
    if fp.is_null() {
        let first = unsafe { *name.cast::<u8>() };
        if first.is_ascii_digit() {
            unsafe { internal_error(c"func_unref()".as_ptr()) };
            std::process::abort();
        }
        return;
    }
    unsafe { rs_func_ptr_unref(fp) };
}

// =============================================================================
// func_ref
// =============================================================================

/// Increment reference count of a Function by name.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string or NULL.
#[unsafe(export_name = "func_ref")]
pub unsafe extern "C" fn rs_func_ref(name: *mut c_char) {
    if name.is_null() || unsafe { super::names::rs_func_name_refcount(name) } == 0 {
        return;
    }
    let fp = unsafe { find_func(name) };
    if fp.is_null() {
        let first = unsafe { *name.cast::<u8>() };
        if first.is_ascii_digit() {
            unsafe { internal_error(c"func_ref()".as_ptr()) };
            // Note: C version doesn't abort here for func_ref, just emits error
        }
    } else {
        unsafe { nvim_ufunc_increment_refcount(fp) };
    }
}

// =============================================================================
// func_remove
// =============================================================================

/// Remove a function from the function hashtable.
/// Returns true if the entry was deleted.
#[no_mangle]
pub unsafe extern "C" fn rs_func_remove(fp: *mut c_void) -> c_int {
    unsafe { nvim_func_remove_impl(fp) }
}

// =============================================================================
// func_clear_items
// =============================================================================

/// Clear all items a function contains (garrays, profiling, lua refs).
///
/// Phase 14: inlined from nvim_func_clear_items_impl.
#[no_mangle]
pub unsafe extern "C" fn rs_func_clear_items(fp: *mut c_void) {
    unsafe { nvim_ga_clear_strings_wrapper(nvim_ufunc_get_args_ga(fp)) };
    unsafe { nvim_ga_clear_strings_wrapper(nvim_ufunc_get_def_args_ga(fp)) };
    unsafe { nvim_ga_clear_strings_wrapper(nvim_ufunc_get_lines_ga(fp)) };
    unsafe { nvim_ufunc_clear_luaref(fp) };
    unsafe { nvim_ufunc_xfree_tml(fp) };
}

// =============================================================================
// func_clear
// =============================================================================

/// Clear all things a function contains. Does not free the function itself.
#[no_mangle]
pub unsafe extern "C" fn rs_func_clear(fp: *mut c_void, force: c_int) {
    if unsafe { nvim_ufunc_get_cleared(fp) } != 0 {
        return;
    }
    unsafe { nvim_ufunc_set_cleared(fp, 1) };
    unsafe { rs_func_clear_items(fp) };
    let scoped = unsafe { nvim_ufunc_get_scoped(fp) };
    unsafe { super::funccal::rs_funccal_unref(scoped, fp, force) };
}

// =============================================================================
// func_free
// =============================================================================

// FC_DELETED and FC_REMOVED flags (matches userfunc.h)
const FC_DELETED: c_int = 0x10;
const FC_REMOVED: c_int = 0x20;

/// Free a function and remove it from the list. Does not free contents.
#[no_mangle]
pub unsafe extern "C" fn rs_func_free(fp: *mut c_void) {
    let flags = unsafe { nvim_ufunc_get_flags(fp) };
    if (flags & (FC_DELETED | FC_REMOVED)) == 0 {
        unsafe { rs_func_remove(fp) };
    }
    unsafe { nvim_ufunc_clear_name_exp(fp) };
    unsafe { xfree(fp) };
}

// =============================================================================
// func_clear_free
// =============================================================================

/// Clear and free a function (combined).
#[no_mangle]
pub unsafe extern "C" fn rs_func_clear_free(fp: *mut c_void, force: c_int) {
    unsafe { rs_func_clear(fp, force) };
    unsafe { rs_func_free(fp) };
}
