//! Search path cache management
//!
//! Manages the runtime search path cache, which pre-computes and caches
//! expanded runtime and package paths for fast file lookup.
//!
//! The actual data structure operations (kvec, Map, Set) remain in C.
//! This module handles the cache lifecycle: validation, reference counting,
//! and invalidation.

use std::ffi::{c_char, c_int};

// =============================================================================
// Opaque C Types
// =============================================================================

/// Opaque handle to optset_T
#[repr(C)]
pub struct OptsetT {
    _private: [u8; 0],
}

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Global state accessors (in runtime_ffi.c)
    fn nvim_rt_sp_get_valid() -> bool;
    fn nvim_rt_sp_set_valid(valid: bool);
    fn nvim_rt_sp_get_ref() -> *mut c_int;
    fn nvim_rt_sp_set_ref(ref_ptr: *mut c_int);
    fn nvim_rt_sp_mutex_init();
    fn nvim_rt_sp_mutex_lock();
    fn nvim_rt_sp_mutex_unlock();

    // C functions that operate on the global RuntimeSearchPath directly
    fn nvim_rt_sp_build_and_set();
    fn nvim_rt_sp_free_path();
    fn nvim_rt_sp_copy_to_thread();

    // Deferred-safe check
    fn nvim_rt_nlua_is_deferred_safe() -> bool;
}

// =============================================================================
// Public FFI Functions
// =============================================================================

/// Initialize runtime search path system.
///
/// Called once during startup to initialize the mutex.
#[export_name = "runtime_init"]
pub unsafe extern "C" fn rs_runtime_init() {
    nvim_rt_sp_mutex_init();
}

/// Callback for 'runtimepath' or 'packpath' option change.
///
/// Invalidates the cached search path so it gets rebuilt on next use.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_runtimepackpath(_args: *mut OptsetT) -> *const c_char {
    nvim_rt_sp_set_valid(false);
    std::ptr::null()
}

/// Validate and rebuild runtime search path if needed.
///
/// If the cache is invalid, rebuilds it from `p_rtp` and `p_pp`, then
/// copies the result to the thread-safe copy under mutex protection.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_search_path_validate() {
    if !nvim_rt_nlua_is_deferred_safe() {
        // Cannot rebuild search path in an async context. Prefer stale cache
        // over erroring out, as the plugin will likely already have the
        // sought module in the cached path.
        return;
    }

    if !nvim_rt_sp_get_valid() {
        let ref_ptr = nvim_rt_sp_get_ref();
        if ref_ptr.is_null() {
            // No one holds a reference, safe to free
            nvim_rt_sp_free_path();
        }

        // Build new search path and store it in the global
        nvim_rt_sp_build_and_set();
        nvim_rt_sp_set_valid(true);
        nvim_rt_sp_set_ref(std::ptr::null_mut()); // initially unowned

        // Update thread-safe copy
        nvim_rt_sp_mutex_lock();
        nvim_rt_sp_copy_to_thread();
        nvim_rt_sp_mutex_unlock();
    }
}

/// Get cached search path with reference counting.
///
/// This is called from C code that works with RuntimeSearchPath values.
/// The function validates the cache and manages reference counting.
///
/// The `ref_out` parameter tracks whether the caller holds a reference.
/// If no one else held a reference, this caller becomes the ref holder.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_search_path_get_cached(ref_out: *mut c_int) {
    rs_runtime_search_path_validate();

    *ref_out = 0;
    if nvim_rt_sp_get_ref().is_null() {
        // Cached path was unreferenced. Keep a ref to prevent
        // runtime_search_path_validate() from freeing it too early.
        *ref_out = 1;
        nvim_rt_sp_set_ref(ref_out);
    }
}

/// Release reference to runtime search path.
///
/// Called from C code when done using a cached search path.
/// `ref_ptr` is the reference counter from `runtime_search_path_get_cached`.
/// If we're the current holder, just release. Otherwise, free the path copy.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_search_path_unref(ref_ptr: *const c_int) -> bool {
    if ref_ptr.is_null() || *ref_ptr == 0 {
        return false;
    }

    let current_ref = nvim_rt_sp_get_ref();
    if std::ptr::eq(current_ref, ref_ptr.cast_mut()) {
        nvim_rt_sp_set_ref(std::ptr::null_mut());
        false // caller should NOT free the path
    } else {
        true // caller should free the path
    }
}
