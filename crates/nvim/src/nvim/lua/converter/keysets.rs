//! Keysets: a Lua options table as a generated `KeySet` struct.
//!
//! `nlua_pop_keydict` fills one of the api's generated option structs from a
//! Lua table, driven by the keyset's own [`KeySetLink`] hash function, and
//! `nlua_push_keydict` renders one back.  `nlua_init_types` installs the
//! metatables the api's handle types (buffer, window, tabpage) are
//! recognised by.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_init_types(lstate: *mut lua_State) {
    unsafe {
        lua_pushlstring(
            lstate,
            b"type_idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        );
        nlua_push_type_idx(lstate);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_pushlstring(
            lstate,
            b"val_idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        );
        nlua_push_val_idx(lstate);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_pushlstring(
            lstate,
            b"types\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
        lua_pushlstring(
            lstate,
            b"float\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        lua_pushnumber(lstate, kObjectTypeFloat as ::core::ffi::c_int as lua_Number);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_pushnumber(lstate, kObjectTypeFloat as ::core::ffi::c_int as lua_Number);
        lua_pushlstring(
            lstate,
            b"float\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_pushlstring(
            lstate,
            b"array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        lua_pushnumber(lstate, kObjectTypeArray as ::core::ffi::c_int as lua_Number);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_pushnumber(lstate, kObjectTypeArray as ::core::ffi::c_int as lua_Number);
        lua_pushlstring(
            lstate,
            b"array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_pushlstring(
            lstate,
            b"dictionary\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
        );
        lua_pushnumber(lstate, kObjectTypeDict as ::core::ffi::c_int as lua_Number);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_pushnumber(lstate, kObjectTypeDict as ::core::ffi::c_int as lua_Number);
        lua_pushlstring(
            lstate,
            b"dictionary\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
        );
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_keydict(
    mut L: *mut lua_State,
    mut retval: *mut ::core::ffi::c_void,
    mut hashy: FieldHashfn,
    mut err_opt: *mut *mut ::core::ffi::c_char,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TTABLE) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Expected Lua table\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_settop(L, -(-1 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int);
            return;
        }
        lua_pushnil(L);
        while lua_next(L, -2 as ::core::ffi::c_int) != 0 {
            let mut len: size_t = 0;
            let mut s: *const ::core::ffi::c_char =
                lua_tolstring(L, -2 as ::core::ffi::c_int, &raw mut len);
            let mut field: *mut KeySetLink = hashy.expect("non-null function pointer")(s, len);
            if field.is_null() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"invalid key: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
                    len as ::core::ffi::c_int,
                    s,
                );
                lua_settop(L, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                return;
            }
            if (*field).opt_index >= 0 as ::core::ffi::c_int {
                let mut ks: *mut OptKeySet = retval as *mut OptKeySet;
                (*ks).is_set_ = ((*ks).is_set_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << (*field).opt_index)
                    as OptionalKeys;
            }
            let mut mem: *mut ::core::ffi::c_char =
                (retval as *mut ::core::ffi::c_char).offset((*field).ptr_off as isize);
            if (*field).type_0 == kObjectTypeNil as ::core::ffi::c_int {
                *(mem as *mut Object) = nlua_pop_Object(L, true_0 != 0, arena, err);
            } else if (*field).type_0 == kObjectTypeInteger as ::core::ffi::c_int {
                if (*field).is_hlgroup as ::core::ffi::c_int != 0
                    && lua_type(L, -1 as ::core::ffi::c_int) == LUA_TSTRING
                {
                    let mut name_len: size_t = 0;
                    let mut name: *const ::core::ffi::c_char =
                        lua_tolstring(L, -1 as ::core::ffi::c_int, &raw mut name_len);
                    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    *(mem as *mut Integer) = (if name_len > 0 as size_t {
                        syn_check_group(name, name_len)
                    } else {
                        0 as ::core::ffi::c_int
                    }) as Integer;
                } else {
                    *(mem as *mut Integer) = nlua_pop_Integer(L, arena, err);
                }
            } else if (*field).type_0 == kObjectTypeBoolean as ::core::ffi::c_int {
                *(mem as *mut Boolean) = nlua_pop_Boolean_strict(L, err);
            } else if (*field).type_0 == kObjectTypeString as ::core::ffi::c_int {
                *(mem as *mut String_0) = nlua_pop_String(L, arena, err);
            } else if (*field).type_0 == kObjectTypeFloat as ::core::ffi::c_int {
                *(mem as *mut Float) = nlua_pop_Float(L, arena, err);
            } else if (*field).type_0 == kObjectTypeBuffer as ::core::ffi::c_int
                || (*field).type_0 == kObjectTypeWindow as ::core::ffi::c_int
                || (*field).type_0 == kObjectTypeTabpage as ::core::ffi::c_int
            {
                *(mem as *mut handle_T) = nlua_pop_handle(L, arena, err);
            } else if (*field).type_0 == kObjectTypeArray as ::core::ffi::c_int {
                *(mem as *mut Array) = nlua_pop_Array(L, arena, err);
            } else if (*field).type_0 == kObjectTypeDict as ::core::ffi::c_int {
                *(mem as *mut Dict) = nlua_pop_Dict(L, false_0 != 0, arena, err);
            } else if (*field).type_0 == kObjectTypeLuaRef as ::core::ffi::c_int {
                *(mem as *mut LuaRef) = nlua_pop_LuaRef(L, arena, err);
            } else {
                abort();
            }
            if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                continue;
            }
            *err_opt = (*field).str;
            break;
        }
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C-unwind" fn nlua_push_keydict(
    mut L: *mut lua_State,
    mut value: *mut ::core::ffi::c_void,
    mut table: *mut KeySetLink,
) {
    unsafe {
        lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        let mut i: size_t = 0 as size_t;
        while !(*table.offset(i as isize)).str.is_null() {
            let mut field: *mut KeySetLink = table.offset(i as isize);
            let mut is_set: bool = true_0 != 0;
            if (*field).opt_index >= 0 as ::core::ffi::c_int {
                let mut ks: *mut OptKeySet = value as *mut OptKeySet;
                is_set = (*ks).is_set_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << (*field).opt_index
                    != 0;
            }
            if is_set {
                let mut mem: *mut ::core::ffi::c_char =
                    (value as *mut ::core::ffi::c_char).offset((*field).ptr_off as isize);
                lua_pushstring(L, (*field).str);
                if (*field).type_0 == kObjectTypeNil as ::core::ffi::c_int {
                    nlua_push_Object(L, mem as *mut Object, 0 as ::core::ffi::c_int);
                } else if (*field).type_0 == kObjectTypeInteger as ::core::ffi::c_int {
                    lua_pushinteger(L, *(mem as *mut Integer) as lua_Integer);
                } else if (*field).type_0 == kObjectTypeBuffer as ::core::ffi::c_int
                    || (*field).type_0 == kObjectTypeWindow as ::core::ffi::c_int
                    || (*field).type_0 == kObjectTypeTabpage as ::core::ffi::c_int
                {
                    lua_pushinteger(L, *(mem as *mut handle_T) as lua_Integer);
                } else if (*field).type_0 == kObjectTypeFloat as ::core::ffi::c_int {
                    lua_pushnumber(L, *(mem as *mut Float) as lua_Number);
                } else if (*field).type_0 == kObjectTypeBoolean as ::core::ffi::c_int {
                    lua_pushboolean(L, *(mem as *mut Boolean) as ::core::ffi::c_int);
                } else if (*field).type_0 == kObjectTypeString as ::core::ffi::c_int {
                    nlua_push_String(L, *(mem as *mut String_0), 0 as ::core::ffi::c_int);
                } else if (*field).type_0 == kObjectTypeArray as ::core::ffi::c_int {
                    nlua_push_Array(L, *(mem as *mut Array), 0 as ::core::ffi::c_int);
                } else if (*field).type_0 == kObjectTypeDict as ::core::ffi::c_int {
                    nlua_push_Dict(L, *(mem as *mut Dict), 0 as ::core::ffi::c_int);
                } else if (*field).type_0 == kObjectTypeLuaRef as ::core::ffi::c_int {
                    nlua_pushref(L, *(mem as *mut LuaRef));
                } else {
                    abort();
                }
                lua_rawset(L, -3 as ::core::ffi::c_int);
            }
            i = i.wrapping_add(1);
        }
    }
}
