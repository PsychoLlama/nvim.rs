//! `LuaRef`s: the registry table every callback is held in.
//!
//! A `LuaRef` is an index into a per-state reference table, allocated by
//! [`nlua_ref`] and released by [`nlua_unref`]; [`nlua_ref_global`] and its
//! `_global` twins are the same against the *main* state, which is what an
//! api callback stored in a buffer or an autocommand needs.  The nil and
//! empty-dict sentinels live here too: both are single refs to a unique
//! table, which is how `vim.NIL` and `vim.empty_dict()` compare by
//! identity.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};

use super::get_global_lstate;
use crate::lua::ffi::{
    LUA_NOREF, LUA_REGISTRYINDEX, LUA_TFUNCTION, lua_getfield, lua_newuserdata, lua_pop,
    lua_pushstring, lua_pushvalue, lua_rawgeti, lua_touserdata, lua_type, luaL_ref, luaL_unref,
};
use crate::main::nlua_global_refs;
use crate::types::{LuaRef, lua_State, nlua_ref_state_t};
use ::libc::memset;

/// The registry key the per-state reference table is parked under.
const REF_STATE_KEY: &core::ffi::CStr = c"nlua.ref_state";

/// Allocate this state's reference bookkeeping, as a Lua userdatum so the
/// state owns it.  The main state's also becomes the global one.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
pub(crate) unsafe extern "C-unwind" fn nlua_new_ref_state(
    lstate: *mut lua_State,
    is_thread: bool,
) -> *mut nlua_ref_state_t {
    unsafe {
        let ref_state =
            lua_newuserdata(lstate, size_of::<nlua_ref_state_t>()).cast::<nlua_ref_state_t>();
        memset(ref_state.cast::<c_void>(), 0, size_of::<nlua_ref_state_t>());
        (*ref_state).nil_ref = LUA_NOREF;
        (*ref_state).empty_dict_ref = LUA_NOREF;
        if !is_thread {
            nlua_global_refs.set(ref_state);
        }
        ref_state
    }
}

/// This state's reference bookkeeping.
///
/// # Safety
/// `lstate` must be a live Lua state whose [`nlua_new_ref_state`] has run.
pub(crate) unsafe extern "C-unwind" fn nlua_get_ref_state(
    lstate: *mut lua_State,
) -> *mut nlua_ref_state_t {
    unsafe {
        lua_getfield(lstate, LUA_REGISTRYINDEX, REF_STATE_KEY.as_ptr());
        let ref_state = lua_touserdata(lstate, -1).cast::<nlua_ref_state_t>();
        lua_pop(lstate, 1);
        ref_state
    }
}

/// # Safety
/// As [`nlua_get_ref_state`].
pub unsafe extern "C-unwind" fn nlua_get_nil_ref(lstate: *mut lua_State) -> LuaRef {
    unsafe { (*nlua_get_ref_state(lstate)).nil_ref }
}

/// # Safety
/// As [`nlua_get_ref_state`].
pub unsafe extern "C-unwind" fn nlua_get_empty_dict_ref(lstate: *mut lua_State) -> LuaRef {
    unsafe { (*nlua_get_ref_state(lstate)).empty_dict_ref }
}

/// How many references the main state is holding — what the functional tests
/// assert against to catch a leak.
///
/// # Safety
/// The main state must exist.
pub unsafe extern "C-unwind" fn nlua_get_global_ref_count() -> c_int {
    unsafe { (*nlua_global_refs.get()).ref_count }
}

/// `vim.NIL`'s `__tostring`.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
pub(crate) unsafe extern "C-unwind" fn nlua_nil_tostring(lstate: *mut lua_State) -> c_int {
    unsafe {
        lua_pushstring(lstate, c"vim.NIL".as_ptr());
        1
    }
}

/// `vim.empty_dict()`'s `__tostring`.
///
/// # Safety
/// As [`nlua_nil_tostring`].
pub(crate) unsafe extern "C-unwind" fn nlua_empty_dict_tostring(lstate: *mut lua_State) -> c_int {
    unsafe {
        lua_pushstring(lstate, c"vim.empty_dict()".as_ptr());
        1
    }
}

/// Take a reference to the value at `index`, without popping it.
///
/// Only a *positive* reference counts: `LUA_REFNIL` and `LUA_NOREF` are
/// sentinels, not registry entries.
///
/// # Safety
/// `lstate` must be a live Lua state, `index` a valid index, and `ref_state`
/// that state's own bookkeeping.
pub unsafe extern "C-unwind" fn nlua_ref(
    lstate: *mut lua_State,
    ref_state: *mut nlua_ref_state_t,
    index: c_int,
) -> LuaRef {
    unsafe {
        lua_pushvalue(lstate, index);
        let ref_0 = luaL_ref(lstate, LUA_REGISTRYINDEX);
        if ref_0 > 0 {
            (*ref_state).ref_count += 1;
        }
        ref_0
    }
}

/// [`nlua_ref`] against the main state's bookkeeping.
///
/// # Safety
/// As [`nlua_ref`].
pub unsafe extern "C-unwind" fn nlua_ref_global(lstate: *mut lua_State, index: c_int) -> LuaRef {
    unsafe { nlua_ref(lstate, nlua_global_refs.get(), index) }
}

/// Release a reference.  A sentinel is a no-op.
///
/// # Safety
/// `ref_0` must have come from [`nlua_ref`] against `ref_state`.
pub unsafe extern "C-unwind" fn nlua_unref(
    lstate: *mut lua_State,
    ref_state: *mut nlua_ref_state_t,
    ref_0: LuaRef,
) {
    unsafe {
        if ref_0 > 0 {
            (*ref_state).ref_count -= 1;
            luaL_unref(lstate, LUA_REGISTRYINDEX, ref_0);
        }
    }
}

/// [`nlua_unref`] against the main state's bookkeeping.
///
/// # Safety
/// As [`nlua_unref`].
pub unsafe extern "C-unwind" fn nlua_unref_global(lstate: *mut lua_State, ref_0: LuaRef) {
    unsafe { nlua_unref(lstate, nlua_global_refs.get(), ref_0) };
}

/// [`nlua_unref_global`] against the main state, for api code that has no
/// `lua_State` to hand.
///
/// # Safety
/// The main state must exist and `ref_0` be one of its references.
pub unsafe extern "C-unwind" fn api_free_luaref(ref_0: LuaRef) {
    unsafe { nlua_unref_global(get_global_lstate(), ref_0) };
}

/// Push what `ref_0` refers to.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
pub unsafe extern "C-unwind" fn nlua_pushref(lstate: *mut lua_State, ref_0: LuaRef) {
    unsafe { lua_rawgeti(lstate, LUA_REGISTRYINDEX, ref_0) };
}

/// A second, independently owned reference to the same value.
///
/// # Safety
/// The main state must exist.
pub unsafe extern "C-unwind" fn api_new_luaref(original_ref: LuaRef) -> LuaRef {
    unsafe {
        if original_ref == LUA_NOREF {
            return LUA_NOREF;
        }
        let lstate = get_global_lstate();
        nlua_pushref(lstate, original_ref);
        let new_ref = nlua_ref_global(lstate, -1);
        lua_pop(lstate, 1);
        new_ref
    }
}

/// Whether `ref_0` refers to something callable — the check an api argument
/// that may be either a callback or a value needs.
///
/// # Safety
/// The main state must exist.
pub unsafe extern "C-unwind" fn nlua_ref_is_function(ref_0: LuaRef) -> bool {
    unsafe {
        let lstate = get_global_lstate();
        nlua_pushref(lstate, ref_0);
        let is_function = lua_type(lstate, -1) == LUA_TFUNCTION;
        lua_pop(lstate, 1);
        is_function
    }
}
