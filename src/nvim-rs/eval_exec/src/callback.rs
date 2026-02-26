//! Callback dispatch -- callback_call migration.
//!
//! Migrated from `eval_shim.c` Phase 4.
//!
//! Handles funcref, partial, lua, and none callback types.

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};

use crate::eval::TypevalHandle;

// =============================================================================
// Constants (matching C kCallbackXxx enum)
// =============================================================================

const K_CALLBACK_NONE: c_int = 0;
const K_CALLBACK_FUNCREF: c_int = 1;
const K_CALLBACK_PARTIAL: c_int = 2;
const K_CALLBACK_LUA: c_int = 3;

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Callback accessors
    fn nvim_eval_cb_get_type(cb: *const c_void) -> c_int;
    fn nvim_cb_get_funcref_name(cb: *const c_void) -> *mut c_char;
    fn nvim_eval_cb_get_partial(cb: *const c_void) -> *mut c_void;
    fn nvim_cb_get_luaref(cb: *const c_void) -> i32;

    // callback_depth
    fn nvim_callback_depth_exceeded() -> bool;
    fn nvim_callback_depth_inc();
    fn nvim_callback_depth_dec();

    // Error message
    fn emsg(s: *const c_char) -> c_int;

    // v:lua funcref check
    fn nvim_cb_check_vlua_funcref(name: *const c_char) -> *const c_char;

    // VV_LUA partial getter (canonical name)
    fn nvim_get_vlua_partial() -> *mut c_void;

    // Lua callback call
    fn nvim_callback_call_lua(luaref: i32) -> bool;

    // Funcref/partial callback call
    fn nvim_callback_call_func(
        name: *const c_char,
        partial: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: TypevalHandle,
    ) -> bool;

    // rs_partial_name (already Rust, declared in eval.rs as const)
    fn rs_partial_name(pt: *const c_void) -> *mut c_char;
}

// Error message
static E_CMD_TOO_RECURSIVE: &[u8] = b"E169: Command too recursive\0";

// =============================================================================
// callback_call
// =============================================================================

/// Core callback dispatch: handles funcref, partial, lua, and none callback types.
///
/// Migrated from C `callback_call` in eval_shim.c.
///
/// # Safety
/// - `callback` must be a valid `Callback *`
/// - `argvars` must be a valid `typval_T[argcount]` or null if argcount == 0
/// - `rettv` must be a valid typval handle
pub unsafe fn callback_call_impl(
    callback: *const c_void,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: TypevalHandle,
) -> bool {
    if nvim_callback_depth_exceeded() {
        emsg(E_CMD_TOO_RECURSIVE.as_ptr() as *const c_char);
        return false;
    }

    let cb_type = nvim_eval_cb_get_type(callback);

    match cb_type {
        K_CALLBACK_FUNCREF => {
            let name = nvim_cb_get_funcref_name(callback);
            // Check if this is a v:lua.* funcref
            let lua_name = nvim_cb_check_vlua_funcref(name);
            let (call_name, partial) = if !lua_name.is_null() {
                let vv_lua_partial = nvim_get_vlua_partial();
                (lua_name, vv_lua_partial)
            } else {
                (name as *const c_char, std::ptr::null_mut())
            };
            nvim_callback_depth_inc();
            let ret = nvim_callback_call_func(call_name, partial, argcount, argvars, rettv);
            nvim_callback_depth_dec();
            ret
        }
        K_CALLBACK_PARTIAL => {
            let partial = nvim_eval_cb_get_partial(callback);
            let name = rs_partial_name(partial as *const c_void);
            nvim_callback_depth_inc();
            let ret = nvim_callback_call_func(name, partial, argcount, argvars, rettv);
            nvim_callback_depth_dec();
            ret
        }
        K_CALLBACK_LUA => {
            let luaref = nvim_cb_get_luaref(callback);
            nvim_callback_call_lua(luaref)
        }
        K_CALLBACK_NONE => false,
        _ => false,
    }
}

/// FFI export for callback_call.
///
/// # Safety
/// See `callback_call_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn callback_call(
    callback: *const c_void,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: TypevalHandle,
) -> bool {
    callback_call_impl(callback, argcount, argvars, rettv)
}
