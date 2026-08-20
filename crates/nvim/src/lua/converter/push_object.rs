//! The api [`Object`] -> Lua direction.
//!
//! One `nlua_push_*` per api type.  The three `nlua_push_type*` helpers plus
//! [`nlua_create_typed_table`] are what build a `{_TYPE, _VAL}` special
//! dictionary -- the representation a value keeps when it has no Lua
//! equivalent -- and [`nlua_push_object`] is the dispatch over `ObjectType`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::{TYPE_IDX_VALUE, VAL_IDX_VALUE, kNluaPushFreeRefs, kNluaPushSpecial};
use crate::lua::executor::{api_free_luaref, nlua_pushref};
use crate::lua::ffi::{
    LUA_NOREF, lua_createtable, lua_pushboolean, lua_pushlstring, lua_pushnil, lua_pushnumber,
    lua_rawset, lua_rawseti, lua_setmetatable,
};
use crate::main::nlua_global_refs;
use crate::types::{
    Array, Boolean, Dict, Float, Integer, LuaRef, Object, ObjectType, String_0, handle_T,
    kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage,
    kObjectTypeWindow, lua_Number, lua_State, size_t,
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
/// # Safety
/// `lstate` must be a live Lua state and `s` a live api string.
pub unsafe fn nlua_push_string(lstate: *mut lua_State, s: String_0, _flags: c_int) {
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
        if flags & kNluaPushSpecial as c_int != 0 {
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
/// `lstate` must be a live Lua state and `dict` a live api dictionary.
pub unsafe fn nlua_push_dict(lstate: *mut lua_State, dict: Dict, flags: c_int) {
    unsafe {
        lua_createtable(lstate, 0, dict.size as c_int);
        if dict.size == 0 {
            nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
            lua_setmetatable(lstate, -2);
        }
        for i in 0..dict.size {
            nlua_push_string(lstate, (*dict.items.add(i)).key, flags);
            nlua_push_object(lstate, &raw mut (*dict.items.add(i)).value, flags);
            lua_rawset(lstate, -3);
        }
    }
}

/// # Safety
/// `lstate` must be a live Lua state and `array` a live api array.
pub unsafe fn nlua_push_array(lstate: *mut lua_State, array: Array, flags: c_int) {
    unsafe {
        lua_createtable(lstate, array.size as c_int, 0);
        for i in 0..array.size {
            nlua_push_object(lstate, array.items.add(i), flags);
            lua_rawseti(lstate, -2, i as c_int + 1);
        }
    }
}

/// # Safety
/// `lstate` must be a live Lua state.
pub unsafe fn nlua_push_handle(lstate: *mut lua_State, item: handle_T, _flags: c_int) {
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
        match (*obj).type_0 {
            kObjectTypeNil => {
                if flags & kNluaPushSpecial as c_int != 0 {
                    lua_pushnil(lstate);
                } else {
                    nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                }
            }
            kObjectTypeLuaRef => {
                nlua_pushref(lstate, (*obj).data.luaref);
                if flags & kNluaPushFreeRefs as c_int != 0 {
                    api_free_luaref((*obj).data.luaref);
                    (*obj).data.luaref = LUA_NOREF as LuaRef;
                }
            }
            kObjectTypeBoolean => nlua_push_boolean(lstate, (*obj).data.boolean, flags),
            kObjectTypeInteger => nlua_push_integer(lstate, (*obj).data.integer, flags),
            kObjectTypeFloat => nlua_push_float(lstate, (*obj).data.floating, flags),
            kObjectTypeString => nlua_push_string(lstate, (*obj).data.string, flags),
            kObjectTypeArray => nlua_push_array(lstate, (*obj).data.array, flags),
            kObjectTypeDict => nlua_push_dict(lstate, (*obj).data.dict, flags),
            kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
                nlua_push_handle(lstate, (*obj).data.integer as handle_T, flags);
            }
            _ => {}
        }
    }
}
