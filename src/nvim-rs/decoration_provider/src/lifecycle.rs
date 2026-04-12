//! Provider lifecycle functions
//!
//! This module implements the lifecycle management functions migrated from C:
//! - `get_decor_provider`: find or create a provider by namespace ID
//! - `decor_provider_clear`: clear all callbacks on a provider
//! - `decor_free_all_mem`: free all providers at shutdown
//! - `decor_provider_invalidate_hl`: invalidate highlight cache on all providers

use std::ffi::{c_int, c_void};

use crate::types::DecorProviderHandle;

// =============================================================================
// C Accessor Declarations
// =============================================================================

extern "C" {
    // kvec accessors
    fn nvim_decor_providers_size() -> usize;
    fn nvim_decor_providers_get_ns_id(i: usize) -> c_int;
    fn nvim_decor_providers_get_ptr(i: usize) -> *mut c_void;
    fn nvim_decor_providers_push_init(ns_id: c_int) -> *mut c_void;
    fn nvim_decor_providers_set_hl_cached(i: usize, val: bool);
    fn nvim_decor_providers_kv_destroy();

    // Provider clear helper
    fn nvim_decor_provider_clear_callbacks(p: *mut c_void);

    // ns_hl_active + hl_check_ns
    fn nvim_decor_get_ns_hl_active() -> c_int;
    fn nvim_decor_set_ns_hl_active(val: c_int);
    fn nvim_decor_hl_check_ns();
}

// =============================================================================
// Phase 1 Lifecycle Functions
// =============================================================================

/// Find a decoration provider by namespace ID, optionally creating one.
///
/// # Safety
/// Returns a raw pointer into the decor_providers kvec. The caller must use
/// it immediately; any subsequent call that may grow the vector invalidates it.
///
/// # Panics
/// Panics if `ns_id <= 0`.
#[unsafe(export_name = "get_decor_provider")]
pub unsafe extern "C" fn get_decor_provider(ns_id: c_int, force: bool) -> *mut c_void {
    assert!(ns_id > 0);
    let len = unsafe { nvim_decor_providers_size() };
    for i in 0..len {
        let id = unsafe { nvim_decor_providers_get_ns_id(i) };
        if id == ns_id {
            return unsafe { nvim_decor_providers_get_ptr(i) };
        }
    }
    if !force {
        return std::ptr::null_mut();
    }
    // NOTE: this may reallocate the vector
    unsafe { nvim_decor_providers_push_init(ns_id) }
}

/// Clear all LuaRef callbacks on a provider and set state to disabled.
///
/// Matches the C `decor_provider_clear` behavior: does NOT clear `hl_def`.
#[unsafe(export_name = "decor_provider_clear")]
pub unsafe extern "C" fn decor_provider_clear(p: *mut c_void) {
    if p.is_null() {
        return;
    }
    unsafe { nvim_decor_provider_clear_callbacks(p) };
}

/// Free all decoration providers (called at shutdown).
#[unsafe(export_name = "decor_free_all_mem")]
pub unsafe extern "C" fn decor_free_all_mem() {
    let len = unsafe { nvim_decor_providers_size() };
    for i in 0..len {
        let p = unsafe { nvim_decor_providers_get_ptr(i) };
        unsafe { nvim_decor_provider_clear_callbacks(p) };
    }
    unsafe { nvim_decor_providers_kv_destroy() };
}

/// Invalidate all cached highlight state and revalidate current namespace.
#[unsafe(export_name = "decor_provider_invalidate_hl")]
pub unsafe extern "C" fn decor_provider_invalidate_hl() {
    let len = unsafe { nvim_decor_providers_size() };
    for i in 0..len {
        unsafe { nvim_decor_providers_set_hl_cached(i, false) };
    }
    let ns_hl_active = unsafe { nvim_decor_get_ns_hl_active() };
    if ns_hl_active != 0 {
        unsafe { nvim_decor_set_ns_hl_active(-1) };
        unsafe { nvim_decor_hl_check_ns() };
    }
}

// =============================================================================
// FFI Handle Accessor (used by other C code via DecorProviderHandle)
// =============================================================================

/// Find a provider by ns_id and return an opaque handle.
#[no_mangle]
pub extern "C" fn rs_decor_provider_find(ns_id: c_int) -> DecorProviderHandle {
    let p = unsafe { get_decor_provider(ns_id, false) };
    // SAFETY: p is either null or a valid DecorProvider pointer from the kvec
    unsafe { DecorProviderHandle::from_ptr(p) }
}
