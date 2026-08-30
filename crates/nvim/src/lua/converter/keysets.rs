//! Keysets: a Lua options table as a generated `KeySet` struct.
//!
//! [`nlua_pop_keydict`] fills one of the api's generated option structs from
//! a Lua table, driven by the keyset's own [`KeySetLink`] hash function, and
//! [`nlua_push_keydict`] renders one back.  [`nlua_init_types`] installs the
//! names the api's type tags are known by on the Lua side.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};

use super::{
    nlua_pop_array, nlua_pop_boolean_strict, nlua_pop_dict, nlua_pop_float, nlua_pop_handle,
    nlua_pop_integer, nlua_pop_luaref, nlua_pop_object, nlua_pop_string, nlua_push_array,
    nlua_push_dict, nlua_push_object, nlua_push_string, nlua_push_type_idx, nlua_push_val_idx,
};
use crate::api_error;
use crate::highlight_group::syn_check_group;
use crate::lua::executor::nlua_pushref;
use crate::lua::ffi::{
    LUA_TSTRING, LUA_TTABLE, lua_createtable, lua_next, lua_pop, lua_pushboolean, lua_pushinteger,
    lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_rawset, lua_settop,
    lua_tolstring, lua_type,
};
use crate::message_fmt::c_str_len;
use crate::types::{
    Arena, Array, Boolean, Dict, Error, FieldHashfn, Float, Integer, KeySetLink, LuaRef, Object,
    OptKeySet, OptionalKeys, String_0, handle_T, kErrorTypeValidation, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat, kObjectTypeInteger,
    kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow,
    lua_Integer, lua_Number, lua_State, size_t,
};
use ::libc::abort;

/// The three api types that need a name of their own on the Lua side, both
/// ways round: `vim.types.float` is the tag and `vim.types[tag]` the name.
const NAMED_TYPES: [(&CStr, c_int); 3] = [
    (c"float", kObjectTypeFloat as c_int),
    (c"array", kObjectTypeArray as c_int),
    (c"dictionary", kObjectTypeDict as c_int),
];

/// Install `type_idx`, `val_idx` and `types` on the `vim` table below the top
/// of the stack.
///
/// # Safety
/// `lstate` must be a live Lua state with that table at -3.
pub unsafe fn nlua_init_types(lstate: *mut lua_State) {
    unsafe {
        // A Lua string, without its terminator.
        let push_cstr = |s: &CStr| lua_pushlstring(lstate, s.as_ptr(), s.count_bytes());

        push_cstr(c"type_idx");
        nlua_push_type_idx(lstate);
        lua_rawset(lstate, -3);

        push_cstr(c"val_idx");
        nlua_push_val_idx(lstate);
        lua_rawset(lstate, -3);

        push_cstr(c"types");
        lua_createtable(lstate, 0, 3);
        for (name, tag) in NAMED_TYPES {
            push_cstr(name);
            lua_pushnumber(lstate, tag as lua_Number);
            lua_rawset(lstate, -3);

            lua_pushnumber(lstate, tag as lua_Number);
            push_cstr(name);
            lua_rawset(lstate, -3);
        }
        lua_rawset(lstate, -3);
    }
}

/// Fill the generated keyset at `retval` from the Lua table on top.
///
/// `hashy` is the keyset's own perfect hash over its field names; a key it
/// does not know is a refusal. On failure `*err_opt` names the field that
/// failed, for the caller's message.
///
/// # Safety
/// `retval` must point at the keyset `hashy` belongs to, and `lstate` have a
/// value on top.
pub unsafe fn nlua_pop_keydict(
    lstate: *mut lua_State,
    retval: *mut c_void,
    hashy: FieldHashfn,
    err_opt: *mut *mut c_char,
    arena: *mut Arena,
    err: &mut Error,
) {
    unsafe {
        if lua_type(lstate, -1) != LUA_TTABLE {
            *err = Error::from_message(kErrorTypeValidation, c"Expected Lua table");
            // Upstream writes `lua_pop(L, -1)` here, which expands to
            // `lua_settop(L, 0)` -- it clears the *whole* stack rather than
            // popping the one value. Kept verbatim; see the divergence
            // docket.
            lua_settop(lstate, 0);
            return;
        }

        lua_pushnil(lstate);
        while lua_next(lstate, -2) != 0 {
            let mut len: size_t = 0;
            let s = lua_tolstring(lstate, -2, &raw mut len);
            let field: *const KeySetLink = hashy.expect("non-null function pointer")(s, len);
            if field.is_null() {
                let key = c_str_len(s, len);
                *err = api_error!(kErrorTypeValidation, "invalid key: {key}");
                lua_pop(lstate, 3);
                return;
            }
            if (*field).opt_index >= 0 {
                let ks = retval.cast::<OptKeySet>();
                (*ks).is_set_ |= (1_u64 << (*field).opt_index) as OptionalKeys;
            }

            let mem = retval.cast::<c_char>().add((*field).ptr_off);
            match (*field).type_0 as ObjectTypeInt {
                T_ANY => *mem.cast::<Object>() = nlua_pop_object(lstate, true, arena, err),
                T_INTEGER => {
                    // A highlight-group field takes the group's *name* as
                    // well as its id.
                    if (*field).is_hlgroup && lua_type(lstate, -1) == LUA_TSTRING {
                        let mut name_len: size_t = 0;
                        let name = lua_tolstring(lstate, -1, &raw mut name_len);
                        lua_pop(lstate, 1);
                        *mem.cast::<Integer>() = if name_len > 0 {
                            syn_check_group(name, name_len) as Integer
                        } else {
                            0
                        };
                    } else {
                        *mem.cast::<Integer>() = nlua_pop_integer(lstate, arena, err);
                    }
                }
                T_BOOLEAN => *mem.cast::<Boolean>() = nlua_pop_boolean_strict(lstate, err),
                T_STRING => *mem.cast::<String_0>() = nlua_pop_string(lstate, arena, err),
                T_FLOAT => *mem.cast::<Float>() = nlua_pop_float(lstate, arena, err),
                T_BUFFER | T_WINDOW | T_TABPAGE => {
                    *mem.cast::<handle_T>() = nlua_pop_handle(lstate, arena, err);
                }
                T_ARRAY => *mem.cast::<Array>() = nlua_pop_array(lstate, arena, err),
                T_DICT => *mem.cast::<Dict>() = nlua_pop_dict(lstate, false, arena, err),
                T_LUAREF => *mem.cast::<LuaRef>() = nlua_pop_luaref(lstate, arena, err),
                _ => abort(),
            }

            if err.is_set() {
                *err_opt = (*field).str;
                break;
            }
        }
        lua_pop(lstate, 1);
    }
}

/// Push a generated keyset as a Lua table, one entry per field that is set.
///
/// # Safety
/// `value` must point at the keyset `table` describes, terminated by a row
/// with a null `str`.
pub unsafe fn nlua_push_keydict(
    lstate: *mut lua_State,
    value: *mut c_void,
    table: *const KeySetLink,
) {
    unsafe {
        lua_createtable(lstate, 0, 0);
        let mut i: size_t = 0;
        while !(*table.add(i)).str.is_null() {
            let field = table.add(i);
            i = i.wrapping_add(1);

            // A field with an `opt_index` is only present when its bit is on;
            // one without is always there.
            if (*field).opt_index >= 0 {
                let ks = value.cast::<OptKeySet>();
                if (*ks).is_set_ & (1_u64 << (*field).opt_index) == 0 {
                    continue;
                }
            }

            let mem = value.cast::<c_char>().add((*field).ptr_off);
            lua_pushstring(lstate, (*field).str);
            match (*field).type_0 as ObjectTypeInt {
                T_ANY => nlua_push_object(lstate, mem.cast::<Object>(), 0),
                T_INTEGER => lua_pushinteger(lstate, *mem.cast::<Integer>() as lua_Integer),
                T_BUFFER | T_WINDOW | T_TABPAGE => {
                    lua_pushinteger(lstate, *mem.cast::<handle_T>() as lua_Integer);
                }
                T_FLOAT => lua_pushnumber(lstate, *mem.cast::<Float>()),
                T_BOOLEAN => lua_pushboolean(lstate, *mem.cast::<Boolean>() as c_int),
                T_STRING => nlua_push_string(lstate, *mem.cast::<String_0>(), 0),
                T_ARRAY => nlua_push_array(lstate, *mem.cast::<Array>(), 0),
                T_DICT => nlua_push_dict(lstate, *mem.cast::<Dict>(), 0),
                T_LUAREF => nlua_pushref(lstate, *mem.cast::<LuaRef>()),
                _ => abort(),
            }
            lua_rawset(lstate, -3);
        }
    }
}

/// `KeySetLink::type_0` is a plain `int`, so the `ObjectType` tags have to be
/// compared at that width.
type ObjectTypeInt = c_int;
const T_ANY: ObjectTypeInt = kObjectTypeNil as ObjectTypeInt;
const T_BOOLEAN: ObjectTypeInt = kObjectTypeBoolean as ObjectTypeInt;
const T_INTEGER: ObjectTypeInt = kObjectTypeInteger as ObjectTypeInt;
const T_FLOAT: ObjectTypeInt = kObjectTypeFloat as ObjectTypeInt;
const T_STRING: ObjectTypeInt = kObjectTypeString as ObjectTypeInt;
const T_ARRAY: ObjectTypeInt = kObjectTypeArray as ObjectTypeInt;
const T_DICT: ObjectTypeInt = kObjectTypeDict as ObjectTypeInt;
const T_LUAREF: ObjectTypeInt = kObjectTypeLuaRef as ObjectTypeInt;
const T_BUFFER: ObjectTypeInt = kObjectTypeBuffer as ObjectTypeInt;
const T_WINDOW: ObjectTypeInt = kObjectTypeWindow as ObjectTypeInt;
const T_TABPAGE: ObjectTypeInt = kObjectTypeTabpage as ObjectTypeInt;
