//! The api [`Object`] -> Lua direction.
//!
//! One `nlua_push_*` per api type.  The three `nlua_push_type*` helpers plus
//! [`nlua_create_typed_table`] are what build a `{_TYPE, _VAL}` special
//! dictionary -- the representation a value keeps when it has no Lua
//! equivalent -- and [`nlua_push_object`] is the dispatch over `ObjectType`.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::ffi::c_int;

use super::{TYPE_IDX_VALUE, VAL_IDX_VALUE, kNluaPushFreeRefs, kNluaPushSpecial};
use crate::lua::executor::{api_free_luaref, nlua_pushref};
use crate::lua::ffi::{
    LUA_NOREF, lua_createtable, lua_pushboolean, lua_pushlstring, lua_pushnil, lua_pushnumber,
    lua_rawset, lua_rawseti, lua_setmetatable,
};
use crate::lua::state::nlua_global_refs;
use crate::types::{
    ApiDict, Array, Boolean, Float, Handle, Integer, LuaRef, Object, ObjectType, String_0,
    kObjectTypeFloat, lua_Number, lua_State, size_t,
};

/// Push the key a special table's type tag is stored under.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
#[inline]
pub(crate) unsafe fn nlua_push_type_idx(lstate: *mut lua_State) {
    unsafe { lua_pushboolean(lstate, TYPE_IDX_VALUE as c_int) };
}

/// Push the key a special table's value is stored under.
///
/// # Safety
/// As [`nlua_push_type_idx`].
#[inline]
pub(crate) unsafe fn nlua_push_val_idx(lstate: *mut lua_State) {
    unsafe { lua_pushboolean(lstate, VAL_IDX_VALUE as c_int) };
}

/// Push a fresh table already carrying its `_TYPE` tag, with room for `narr`
/// array and `nrec` record entries beside it.
///
/// # Safety
/// As [`nlua_push_type_idx`].
#[inline]
pub(crate) unsafe fn nlua_create_typed_table(
    lstate: *mut lua_State,
    narr: size_t,
    nrec: size_t,
    type_0: ObjectType,
) {
    unsafe {
        lua_createtable(lstate, narr as c_int, nrec.wrapping_add(1) as c_int);
        nlua_push_type_idx(lstate);
        lua_pushnumber(lstate, type_0 as lua_Number);
        lua_rawset(lstate, -3);
    }
}

/// Push an api string as a Lua string, NULs and all.
///
/// The string stays the caller's: Lua copies the bytes it is handed.
///
/// # Safety
/// `lstate` must be a live Lua state.
pub unsafe fn nlua_push_string(lstate: *mut lua_State, s: &String_0, _flags: c_int) {
    unsafe {
        // A zero-length api string may carry a null pointer, which
        // lua_pushlstring will not take even for zero bytes.
        let data = if !s.is_empty() {
            s.data().cast_const()
        } else {
            c"".as_ptr()
        };
        lua_pushlstring(lstate, data, s.len());
    }
}

/// # Safety
/// `lstate` must be a live Lua state.
pub unsafe fn nlua_push_integer(lstate: *mut lua_State, n: Integer, _flags: c_int) {
    unsafe { lua_pushnumber(lstate, n as lua_Number) };
}

/// Push a float, as a plain number or -- with `kNluaPushSpecial` -- as the
/// `{_TYPE = float, _VAL = f}` table that survives a round trip through
/// Vimscript.
///
/// # Safety
/// `lstate` must be a live Lua state.
pub unsafe fn nlua_push_float(lstate: *mut lua_State, f: Float, flags: c_int) {
    unsafe {
        if flags & kNluaPushSpecial != 0 {
            nlua_create_typed_table(lstate, 0, 1, kObjectTypeFloat);
            nlua_push_val_idx(lstate);
            lua_pushnumber(lstate, f);
            lua_rawset(lstate, -3);
        } else {
            lua_pushnumber(lstate, f);
        }
    }
}

/// # Safety
/// `lstate` must be a live Lua state.
pub unsafe fn nlua_push_boolean(lstate: *mut lua_State, b: Boolean, _flags: c_int) {
    unsafe { lua_pushboolean(lstate, b as c_int) };
}

/// Push an api dictionary as a Lua table.  An empty one carries the
/// `vim.empty_dict()` metatable, because an empty Lua table is otherwise
/// indistinguishable from an empty list.
///
/// # Safety
/// `lstate` must be a live Lua state.
///
/// The borrow is mutable because `kNluaPushFreeRefs` clears each `LuaRef` it
/// releases, one level down.
pub unsafe fn nlua_push_dict(lstate: *mut lua_State, dict: &mut ApiDict, flags: c_int) {
    unsafe {
        lua_createtable(lstate, 0, dict.len() as c_int);
        if dict.is_empty() {
            nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
            lua_setmetatable(lstate, -2);
        }
        for entry in dict.iter_mut() {
            // The key is never the null string, so it needs none of
            // `nlua_push_string`'s care about one.
            lua_pushlstring(lstate, entry.key.as_ptr(), entry.key.len());
            nlua_push_object(lstate, &raw mut entry.value, flags);
            lua_rawset(lstate, -3);
        }
    }
}

/// # Safety
/// `lstate` must be a live Lua state. See [`nlua_push_dict`] for the mutable
/// borrow.
pub unsafe fn nlua_push_array(lstate: *mut lua_State, array: &mut Array, flags: c_int) {
    unsafe {
        lua_createtable(lstate, array.len() as c_int, 0);
        for (i, item) in array.iter_mut().enumerate() {
            nlua_push_object(lstate, item, flags);
            lua_rawseti(lstate, -2, i as c_int + 1);
        }
    }
}

/// # Safety
/// `lstate` must be a live Lua state.
pub unsafe fn nlua_push_handle(lstate: *mut lua_State, item: Handle, _flags: c_int) {
    unsafe { lua_pushnumber(lstate, item as lua_Number) };
}

/// Push any api value.
///
/// With `kNluaPushFreeRefs` a `LuaRef` is released as it is pushed, and the
/// object's copy of it is cleared: the caller owns the object and is done
/// with it.
///
/// # Safety
/// `lstate` must be a live Lua state and `obj` a live api object.
pub unsafe fn nlua_push_object(lstate: *mut lua_State, obj: *mut Object, flags: c_int) {
    unsafe {
        match &mut *obj {
            Object::Nil => {
                if flags & kNluaPushSpecial != 0 {
                    lua_pushnil(lstate);
                } else {
                    nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                }
            }
            Object::LuaRef(reference) => {
                nlua_pushref(lstate, *reference);
                if flags & kNluaPushFreeRefs != 0 {
                    api_free_luaref(*reference);
                    // The arm keeps its tag and gives up the reference, which
                    // is what stops the object's own `Drop` releasing it
                    // again.
                    *reference = LUA_NOREF as LuaRef;
                }
            }
            Object::Boolean(b) => nlua_push_boolean(lstate, *b, flags),
            Object::Integer(n) => nlua_push_integer(lstate, *n, flags),
            Object::Float(f) => nlua_push_float(lstate, *f, flags),
            Object::String(s) => nlua_push_string(lstate, s, flags),
            Object::Array(a) => nlua_push_array(lstate, a, flags),
            Object::Dict(d) => nlua_push_dict(lstate, d, flags),
            Object::Buffer(h) | Object::Window(h) | Object::Tabpage(h) => {
                nlua_push_handle(lstate, *h as Handle, flags);
            }
        }
    }
}
