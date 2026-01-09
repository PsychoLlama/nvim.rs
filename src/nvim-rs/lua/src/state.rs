//! Lua state accessors for Neovim
//!
//! This module provides safe Rust wrappers for accessing Lua state information
//! and callback infrastructure.

use std::ffi::c_int;
use std::ptr;

/// Opaque handle to Lua state (lua_State in C)
#[repr(C)]
pub struct LuaState {
    _private: [u8; 0],
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Global Lua state accessor
    fn get_global_lstate() -> *mut LuaState;

    // in_fast_callback accessor
    fn nvim_get_in_fast_callback() -> c_int;

    // nlua_global_refs->ref_count accessor
    fn nvim_get_nlua_global_ref_count() -> c_int;
}

// =============================================================================
// Rust FFI exports
// =============================================================================

/// Get the global Lua state.
///
/// Returns NULL if Lua has not been initialized yet.
///
/// # Safety
///
/// The returned pointer is only valid while Lua is initialized.
/// Do not store the pointer across Lua reinitializations.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_get_state() -> *mut LuaState {
    get_global_lstate()
}

/// Check if Lua has been initialized.
///
/// Returns 1 if `global_lstate` is non-NULL, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_is_initialized() -> c_int {
    c_int::from(!get_global_lstate().is_null())
}

/// Get the current callback depth.
///
/// The `in_fast_callback` counter tracks nesting of fast callbacks.
/// This includes luv callbacks and some UI event callbacks where
/// the API is not safe to call directly.
///
/// Returns the current nesting level (0 means not in fast callback).
#[no_mangle]
pub unsafe extern "C" fn rs_lua_callback_depth() -> c_int {
    nvim_get_in_fast_callback()
}

/// Check if currently in a fast callback context.
///
/// Fast callbacks are luv callbacks and some UI event callbacks where
/// the API is not safe to call directly. During fast callbacks, deferred
/// API methods should be used instead.
///
/// Returns 1 if `in_fast_callback > 0`, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_in_fast_callback() -> c_int {
    c_int::from(nvim_get_in_fast_callback() > 0)
}

/// Get the global Lua reference count.
///
/// Returns the number of active Lua references in `nlua_global_refs->ref_count`.
/// This is useful for debugging and tracking Lua reference leaks.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_get_ref_count() -> c_int {
    nvim_get_nlua_global_ref_count()
}

// =============================================================================
// Safe Rust API
// =============================================================================

/// Check if Lua has been initialized.
///
/// # Safety
///
/// Accesses global state.
#[inline]
#[must_use]
pub unsafe fn is_initialized() -> bool {
    !get_global_lstate().is_null()
}

/// Check if currently in a fast callback context.
///
/// # Safety
///
/// Accesses global state.
#[inline]
#[must_use]
pub unsafe fn in_fast_callback() -> bool {
    nvim_get_in_fast_callback() > 0
}

/// Get the current callback depth.
///
/// # Safety
///
/// Accesses global state.
#[inline]
#[must_use]
pub unsafe fn callback_depth() -> i32 {
    nvim_get_in_fast_callback()
}

/// Check if deferred API calls are safe.
///
/// Returns true when not in a fast callback context.
///
/// # Safety
///
/// Accesses global state.
#[inline]
#[must_use]
pub unsafe fn is_deferred_safe() -> bool {
    nvim_get_in_fast_callback() == 0
}

/// Get the global Lua state, if initialized.
///
/// # Safety
///
/// The returned pointer is only valid while Lua is initialized.
#[inline]
#[must_use]
pub unsafe fn get_state() -> Option<ptr::NonNull<LuaState>> {
    ptr::NonNull::new(get_global_lstate())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_state_is_opaque() {
        // LuaState should be a zero-sized type used as an opaque handle
        assert_eq!(std::mem::size_of::<LuaState>(), 0);
    }
}
