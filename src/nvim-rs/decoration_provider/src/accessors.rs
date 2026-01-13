//! Accessor functions for DecorProvider fields
//!
//! This module provides FFI wrapper functions to access DecorProvider struct
//! fields from Rust. The actual struct is owned by C, and Rust uses accessor
//! functions to read/write fields.
//!
//! # Implementation Status
//!
//! This module will be expanded in Phase 204.

use std::ffi::c_int;

use crate::types::DecorProviderHandle;

// Placeholder - will be implemented in Phase 204
// =============================================================================
// C Accessor Function Declarations
// =============================================================================

extern "C" {
    // These functions are defined in decoration_provider.c
    fn nvim_decor_provider_get_hl_valid(ns_id: c_int) -> c_int;
    fn nvim_decor_provider_get_hl_cached(ns_id: c_int) -> bool;
    fn nvim_decor_provider_set_hl_cached(ns_id: c_int, cached: bool, force: bool);
    fn nvim_decor_provider_hl_def_prepare(ns_id: c_int) -> c_int;
    fn nvim_decor_provider_has_hl_def(ns_id: c_int) -> bool;
}

// =============================================================================
// Rust Wrapper Functions
// =============================================================================

/// Get hl_valid for a namespace.
/// Returns -1 if provider doesn't exist.
pub fn get_hl_valid(ns_id: c_int) -> c_int {
    unsafe { nvim_decor_provider_get_hl_valid(ns_id) }
}

/// Get hl_cached for a namespace.
/// Returns false if provider doesn't exist.
pub fn get_hl_cached(ns_id: c_int) -> bool {
    unsafe { nvim_decor_provider_get_hl_cached(ns_id) }
}

/// Set hl_cached for a namespace.
/// Creates provider if force=true.
pub fn set_hl_cached(ns_id: c_int, cached: bool, force: bool) {
    unsafe { nvim_decor_provider_set_hl_cached(ns_id, cached, force) }
}

/// Get hl_valid and set hl_cached=false atomically.
/// Creates provider if needed.
pub fn hl_def_prepare(ns_id: c_int) -> c_int {
    unsafe { nvim_decor_provider_hl_def_prepare(ns_id) }
}

/// Check if namespace has a hl_def callback defined.
pub fn has_hl_def(ns_id: c_int) -> bool {
    unsafe { nvim_decor_provider_has_hl_def(ns_id) }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get hl_valid for namespace.
#[no_mangle]
pub extern "C" fn rs_decor_provider_get_hl_valid(ns_id: c_int) -> c_int {
    get_hl_valid(ns_id)
}

/// FFI: Get hl_cached for namespace.
#[no_mangle]
pub extern "C" fn rs_decor_provider_get_hl_cached(ns_id: c_int) -> bool {
    get_hl_cached(ns_id)
}

/// FFI: Set hl_cached for namespace.
#[no_mangle]
pub extern "C" fn rs_decor_provider_set_hl_cached(ns_id: c_int, cached: bool, force: bool) {
    set_hl_cached(ns_id, cached, force);
}

/// FFI: Prepare hl_def (get hl_valid, set hl_cached=false).
#[no_mangle]
pub extern "C" fn rs_decor_provider_hl_def_prepare(ns_id: c_int) -> c_int {
    hl_def_prepare(ns_id)
}

/// FFI: Check if namespace has hl_def callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_hl_def(ns_id: c_int) -> bool {
    has_hl_def(ns_id)
}

/// FFI: Check if handle is null.
#[no_mangle]
pub extern "C" fn rs_decor_provider_handle_is_null(handle: DecorProviderHandle) -> bool {
    handle.is_null()
}
