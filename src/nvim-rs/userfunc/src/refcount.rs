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
    fn nvim_func_clear_impl(fp: *mut c_void, force: c_int);
    fn nvim_func_free_impl(fp: *mut c_void);
    fn nvim_func_clear_items_impl(fp: *mut c_void);
    fn nvim_func_remove_impl(fp: *mut c_void) -> c_int;
    fn internal_error(msg: *const c_char);
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
#[no_mangle]
pub unsafe extern "C" fn rs_func_clear_items(fp: *mut c_void) {
    unsafe { nvim_func_clear_items_impl(fp) };
}

// =============================================================================
// func_clear
// =============================================================================

/// Clear all things a function contains. Does not free the function itself.
#[no_mangle]
pub unsafe extern "C" fn rs_func_clear(fp: *mut c_void, force: c_int) {
    unsafe { nvim_func_clear_impl(fp, force) };
}

// =============================================================================
// func_free
// =============================================================================

/// Free a function and remove it from the list. Does not free contents.
#[no_mangle]
pub unsafe extern "C" fn rs_func_free(fp: *mut c_void) {
    unsafe { nvim_func_free_impl(fp) };
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
