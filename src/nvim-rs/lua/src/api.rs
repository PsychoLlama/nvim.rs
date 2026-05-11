//! API conversion helpers
//!
//! This module provides FFI helpers for API-specific type conversions
//! between Lua and Neovim's API types.

use std::ffi::{c_char, c_int, c_void};

use crate::state::LuaState;
use nvim_api::{Arena, Error, LuaRef};

/// Handle type (buffer, window, tabpage)
pub type HandleT = c_int;

/// Field hash function type for keydict
pub type FieldHashfn = *const c_void;

/// KeySetLink type for keydict
#[repr(C)]
pub struct KeySetLink {
    _private: [u8; 0],
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Typval conversion
    fn nlua_push_typval(lstate: *mut LuaState, tv: *const c_void, flags: c_int) -> bool;

    // Keydict operations
    fn nlua_pop_keydict(
        lstate: *mut LuaState,
        retval: *mut c_void,
        hashy: FieldHashfn,
        err_opt: *mut *mut c_char,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn nlua_push_keydict(lstate: *mut LuaState, value: *mut c_void, table: *mut KeySetLink);
}

// =============================================================================
// Rust FFI exports
// =============================================================================

/// Push a typval (VimL value) onto the Lua stack.
///
/// Converts a typval_T to its Lua equivalent.
///
/// # Arguments
/// * `tv` - Pointer to a typval_T
/// * `flags` - Conversion flags (kNluaPushSpecial, kNluaPushFreeRefs)
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `tv` must be a valid typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_typval(
    lstate: *mut LuaState,
    tv: *const c_void,
    flags: c_int,
) -> bool {
    nlua_push_typval(lstate, tv, flags)
}

/// Pop a LuaRef from the Lua stack.
///
/// Creates a reference to the Lua value at the top of the stack.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` can be NULL or a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_luaref(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> LuaRef {
    // Delegate to from_lua real implementation (Phase 2).
    // from_lua returns c_int (the actual C LuaRef type); nvim_api::LuaRef is i64
    // but that is a pre-existing widening; the value fits.
    #[allow(clippy::cast_lossless)]
    let rv = crate::from_lua::rs_nlua_pop_LuaRef(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    ) as LuaRef;
    rv
}

/// Pop a handle (Buffer/Window/Tabpage) from the Lua stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` must be a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_handle(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    // Delegate to from_lua real implementation (Phase 2)
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

/// Initialize Lua type tables.
///
/// Sets up the special type markers used for typed tables.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_init_types(lstate: *mut LuaState) {
    crate::to_lua::rs_nlua_init_types(lstate);
}

/// Pop a keydict (structured options table) from the Lua stack.
///
/// Parses a Lua table into a keydict structure using the provided hash function.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `retval` must be a valid pointer to the keydict structure.
/// - `hashy` must be a valid field hash function.
/// - `err_opt` can be NULL or a valid pointer.
/// - `arena` must be a valid Arena pointer.
/// - `err` must be a valid Error pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_keydict(
    lstate: *mut LuaState,
    retval: *mut c_void,
    hashy: FieldHashfn,
    err_opt: *mut *mut c_char,
    arena: *mut Arena,
    err: *mut Error,
) {
    nlua_pop_keydict(lstate, retval, hashy, err_opt, arena, err);
}

/// Push a keydict (structured options table) onto the Lua stack.
///
/// Converts a keydict structure to a Lua table.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `value` must be a valid pointer to the keydict structure.
/// - `table` must be a valid KeySetLink pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_keydict(
    lstate: *mut LuaState,
    value: *mut c_void,
    table: *mut KeySetLink,
) {
    nlua_push_keydict(lstate, value, table);
}

// =============================================================================
// Buffer/Window/Tabpage type aliases
// =============================================================================

/// Pop a Buffer handle from the Lua stack.
///
/// Alias for `rs_nlua_pop_handle`.
///
/// # Safety
///
/// Same as `rs_nlua_pop_handle`.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_buffer(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

/// Pop a Window handle from the Lua stack.
///
/// Alias for `rs_nlua_pop_handle`.
///
/// # Safety
///
/// Same as `rs_nlua_pop_handle`.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_window(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

/// Pop a Tabpage handle from the Lua stack.
///
/// Alias for `rs_nlua_pop_handle`.
///
/// # Safety
///
/// Same as `rs_nlua_pop_handle`.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_tabpage(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keysetlink_is_opaque() {
        assert_eq!(std::mem::size_of::<KeySetLink>(), 0);
    }

    #[test]
    fn test_handle_type() {
        // HandleT should be c_int
        assert_eq!(std::mem::size_of::<HandleT>(), std::mem::size_of::<c_int>());
    }
}
