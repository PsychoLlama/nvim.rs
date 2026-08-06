//! `LuaRef`s: the registry table every callback is held in.
//!
//! A `LuaRef` is an index into a per-state reference table, allocated by
//! `nlua_ref` and released by `nlua_unref`; `nlua_ref_global` and its
//! `_global` twins are the same against the *main* state, which is what an
//! api callback stored in a buffer or an autocommand needs.  The nil and
//! empty-dict sentinels live here too: both are single refs to a unique
//! table, which is how `vim.NIL` and `vim.empty_dict()` compare by
//! identity.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C-unwind" fn nlua_new_ref_state(
    mut lstate: *mut lua_State,
    mut is_thread: bool,
) -> *mut nlua_ref_state_t {
    unsafe {
        let mut ref_state: *mut nlua_ref_state_t =
            lua_newuserdata(lstate, ::core::mem::size_of::<nlua_ref_state_t>())
                as *mut nlua_ref_state_t;
        memset(
            ref_state as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<nlua_ref_state_t>(),
        );
        (*ref_state).nil_ref = LUA_NOREF as LuaRef;
        (*ref_state).empty_dict_ref = LUA_NOREF as LuaRef;
        if !is_thread {
            nlua_global_refs.set(ref_state);
        }
        return ref_state;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_get_ref_state(
    mut lstate: *mut lua_State,
) -> *mut nlua_ref_state_t {
    unsafe {
        lua_getfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"nlua.ref_state\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut ref_state: *mut nlua_ref_state_t =
            lua_touserdata(lstate, -1 as ::core::ffi::c_int) as *mut nlua_ref_state_t;
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ref_state;
    }
}

pub unsafe extern "C-unwind" fn nlua_get_nil_ref(mut lstate: *mut lua_State) -> LuaRef {
    unsafe {
        let mut ref_state: *mut nlua_ref_state_t = nlua_get_ref_state(lstate);
        return (*ref_state).nil_ref;
    }
}

pub unsafe extern "C-unwind" fn nlua_get_empty_dict_ref(mut lstate: *mut lua_State) -> LuaRef {
    unsafe {
        let mut ref_state: *mut nlua_ref_state_t = nlua_get_ref_state(lstate);
        return (*ref_state).empty_dict_ref;
    }
}

pub unsafe extern "C-unwind" fn nlua_get_global_ref_count() -> ::core::ffi::c_int {
    unsafe {
        return (*nlua_global_refs.get()).ref_count;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_nil_tostring(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(lstate, b"vim.NIL\0".as_ptr() as *const ::core::ffi::c_char);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_empty_dict_tostring(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(
            lstate,
            b"vim.empty_dict()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C-unwind" fn nlua_ref(
    mut lstate: *mut lua_State,
    mut ref_state: *mut nlua_ref_state_t,
    mut index: ::core::ffi::c_int,
) -> LuaRef {
    unsafe {
        lua_pushvalue(lstate, index);
        let mut ref_0: LuaRef = luaL_ref(lstate, LUA_REGISTRYINDEX);
        if ref_0 > 0 as ::core::ffi::c_int {
            (*ref_state).ref_count += 1;
        }
        return ref_0;
    }
}

pub unsafe extern "C-unwind" fn nlua_ref_global(
    mut lstate: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> LuaRef {
    unsafe {
        return nlua_ref(lstate, nlua_global_refs.get(), index);
    }
}

pub unsafe extern "C-unwind" fn nlua_unref(
    mut lstate: *mut lua_State,
    mut ref_state: *mut nlua_ref_state_t,
    mut ref_0: LuaRef,
) {
    unsafe {
        if ref_0 > 0 as ::core::ffi::c_int {
            (*ref_state).ref_count -= 1;
            luaL_unref(lstate, LUA_REGISTRYINDEX, ref_0 as ::core::ffi::c_int);
        }
    }
}

pub unsafe extern "C-unwind" fn nlua_unref_global(mut lstate: *mut lua_State, mut ref_0: LuaRef) {
    unsafe {
        nlua_unref(lstate, nlua_global_refs.get(), ref_0);
    }
}

pub unsafe extern "C-unwind" fn api_free_luaref(mut ref_0: LuaRef) {
    unsafe {
        nlua_unref_global(global_lstate.get(), ref_0);
    }
}

pub unsafe extern "C-unwind" fn nlua_pushref(mut lstate: *mut lua_State, mut ref_0: LuaRef) {
    unsafe {
        lua_rawgeti(lstate, LUA_REGISTRYINDEX, ref_0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C-unwind" fn api_new_luaref(mut original_ref: LuaRef) -> LuaRef {
    unsafe {
        if original_ref == LUA_NOREF {
            return LUA_NOREF;
        }
        let lstate: *mut lua_State = global_lstate.get();
        nlua_pushref(lstate, original_ref);
        let mut new_ref: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return new_ref;
    }
}

pub unsafe extern "C-unwind" fn nlua_ref_is_function(mut ref_0: LuaRef) -> bool {
    unsafe {
        let lstate: *mut lua_State = global_lstate.get();
        nlua_pushref(lstate, ref_0);
        let mut is_function: bool = lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION;
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return is_function;
    }
}
