//! The api [`Object`] -> Lua direction.
//!
//! One `nlua_push_*` per api type.  The three `nlua_push_type*` helpers plus
//! `nlua_create_typed_table` are what build a `{_TYPE, _VAL}` special
//! dictionary -- the representation a value keeps when it has no Lua
//! equivalent -- and `nlua_push_Object` is the dispatch over `ObjectType`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[inline]
pub(crate) unsafe extern "C-unwind" fn nlua_push_type_idx(mut lstate: *mut lua_State) {
    unsafe {
        lua_pushboolean(lstate, TYPE_IDX_VALUE);
    }
}

#[inline]
pub(crate) unsafe extern "C-unwind" fn nlua_push_val_idx(mut lstate: *mut lua_State) {
    unsafe {
        lua_pushboolean(lstate, VAL_IDX_VALUE);
    }
}

#[inline]
unsafe extern "C-unwind" fn nlua_push_type(mut lstate: *mut lua_State, mut type_0: ObjectType) {
    unsafe {
        lua_pushnumber(lstate, type_0 as lua_Number);
    }
}

#[inline]
pub(crate) unsafe extern "C-unwind" fn nlua_create_typed_table(
    mut lstate: *mut lua_State,
    narr: size_t,
    nrec: size_t,
    type_0: ObjectType,
) {
    unsafe {
        lua_createtable(
            lstate,
            narr as ::core::ffi::c_int,
            (1 as size_t).wrapping_add(nrec) as ::core::ffi::c_int,
        );
        nlua_push_type_idx(lstate);
        nlua_push_type(lstate, type_0);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C-unwind" fn nlua_push_String(
    mut lstate: *mut lua_State,
    s: String_0,
    mut _flags: ::core::ffi::c_int,
) {
    unsafe {
        lua_pushlstring(
            lstate,
            if s.size != 0 {
                s.data as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            s.size,
        );
    }
}

pub unsafe extern "C-unwind" fn nlua_push_Integer(
    mut lstate: *mut lua_State,
    n: Integer,
    mut _flags: ::core::ffi::c_int,
) {
    unsafe {
        lua_pushnumber(lstate, n as lua_Number);
    }
}

pub unsafe extern "C-unwind" fn nlua_push_Float(
    mut lstate: *mut lua_State,
    f: Float,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        if flags & kNluaPushSpecial as ::core::ffi::c_int != 0 {
            nlua_create_typed_table(lstate, 0 as size_t, 1 as size_t, kObjectTypeFloat);
            nlua_push_val_idx(lstate);
            lua_pushnumber(lstate, f);
            lua_rawset(lstate, -3 as ::core::ffi::c_int);
        } else {
            lua_pushnumber(lstate, f);
        };
    }
}

pub unsafe extern "C-unwind" fn nlua_push_Boolean(
    mut lstate: *mut lua_State,
    b: Boolean,
    mut _flags: ::core::ffi::c_int,
) {
    unsafe {
        lua_pushboolean(lstate, b as ::core::ffi::c_int);
    }
}

pub unsafe extern "C-unwind" fn nlua_push_Dict(
    mut lstate: *mut lua_State,
    dict: Dict,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        lua_createtable(
            lstate,
            0 as ::core::ffi::c_int,
            dict.size as ::core::ffi::c_int,
        );
        if dict.size == 0 as size_t {
            nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
            lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
        }
        let mut i: size_t = 0 as size_t;
        while i < dict.size {
            nlua_push_String(lstate, (*dict.items.offset(i as isize)).key, flags);
            nlua_push_Object(
                lstate,
                &raw mut (*dict.items.offset(i as isize)).value,
                flags,
            );
            lua_rawset(lstate, -3 as ::core::ffi::c_int);
            i = i.wrapping_add(1);
        }
    }
}

pub unsafe extern "C-unwind" fn nlua_push_Array(
    mut lstate: *mut lua_State,
    array: Array,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        lua_createtable(
            lstate,
            array.size as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        let mut i: size_t = 0 as size_t;
        while i < array.size {
            nlua_push_Object(lstate, array.items.offset(i as isize), flags);
            lua_rawseti(
                lstate,
                -2 as ::core::ffi::c_int,
                i as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }
    }
}

pub unsafe extern "C-unwind" fn nlua_push_handle(
    mut lstate: *mut lua_State,
    item: handle_T,
    mut _flags: ::core::ffi::c_int,
) {
    unsafe {
        lua_pushnumber(lstate, item as lua_Number);
    }
}

pub unsafe extern "C-unwind" fn nlua_push_Object(
    mut lstate: *mut lua_State,
    mut obj: *mut Object,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        match (*obj).type_0 as ::core::ffi::c_uint {
            0 => {
                if flags & kNluaPushSpecial as ::core::ffi::c_int != 0 {
                    lua_pushnil(lstate);
                } else {
                    nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                }
            }
            7 => {
                nlua_pushref(lstate, (*obj).data.luaref);
                if flags & kNluaPushFreeRefs as ::core::ffi::c_int != 0 {
                    api_free_luaref((*obj).data.luaref);
                    (*obj).data.luaref = LUA_NOREF as LuaRef;
                }
            }
            1 => {
                nlua_push_Boolean(lstate, (*obj).data.boolean, flags);
            }
            2 => {
                nlua_push_Integer(lstate, (*obj).data.integer, flags);
            }
            3 => {
                nlua_push_Float(lstate, (*obj).data.floating, flags);
            }
            4 => {
                nlua_push_String(lstate, (*obj).data.string, flags);
            }
            5 => {
                nlua_push_Array(lstate, (*obj).data.array, flags);
            }
            6 => {
                nlua_push_Dict(lstate, (*obj).data.dict, flags);
            }
            8 => {
                nlua_push_handle(lstate, (*obj).data.integer as handle_T, flags);
            }
            9 => {
                nlua_push_handle(lstate, (*obj).data.integer as handle_T, flags);
            }
            10 => {
                nlua_push_handle(lstate, (*obj).data.integer as handle_T, flags);
            }
            _ => {}
        };
    }
}
