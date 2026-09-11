//! The per-type pops, and the table classifier they share.
//!
//! [`nlua_traverse_table`] is the one place a Lua table's shape is decided:
//! it counts string keys against integer keys against the table's own
//! length, checks the `empty_dict` and special metatables, and hands back a
//! [`LuaTableProps`].  Everything else here pops exactly one value of a
//! known api type, with `nlua_check_type` producing the `E5107`-style
//! message when it is the wrong one.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use crate::semsg;
use core::ffi::{c_int, c_void};

use super::{API_INTEGER_MAX, API_INTEGER_MIN, LuaTableProps, TYPE_IDX_VALUE, nlua_pop_object};
use crate::api::private::helpers::api_typename;
use crate::api_error;
use crate::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::lua::ffi::{
    LUA_TBOOLEAN, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, lua_checkstack, lua_getmetatable,
    lua_gettop, lua_next, lua_pop, lua_pushnil, lua_pushvalue, lua_rawequal, lua_rawgeti,
    lua_toboolean, lua_tolstring, lua_tonumber, lua_type,
};
use crate::lua::state::nlua_global_refs;
use crate::message_fmt::msg_cstr;
use crate::types::{
    ApiDict, Arena, Array, Boolean, Error, Float, Handle, Integer, LuaRef, ObjectType, String_0,
    kErrorTypeValidation, kObjectTypeArray, kObjectTypeDict, kObjectTypeFloat, kObjectTypeNil,
    lua_Number, lua_State, size_t,
};
use ::libc::memchr;

/// Classify the table on top of the stack.
///
/// Leaves the stack exactly as it found it.  Both `pop` walks call this
/// before deciding what to build, so every rule about what a Lua table
/// *means* lives here and nowhere else.
///
/// # Safety
/// `lstate` must be a live Lua state with a table on top.
pub(crate) unsafe fn nlua_traverse_table(lstate: *mut lua_State) -> LuaTableProps {
    unsafe {
        let mut tsize: size_t = 0; // Total number of keys.
        let mut val_type: c_int = 0; // If has_val_key: Lua type of the value.
        let mut has_val_key = false; // Whether the `_VAL` key was found.
        // Keys that are neither string, integral nor one of the two special
        // boolean keys.
        let mut other_keys_num: size_t = 0;
        let mut ret = LuaTableProps::NIL;
        if lua_checkstack(lstate, lua_gettop(lstate) + 3) == 0 {
            let need = lua_gettop(lstate) + 2;
            semsg!("E1502: Lua failed to grow stack to {need}");
            ret.type_0 = kObjectTypeNil;
            return ret;
        }
        lua_pushnil(lstate);
        while lua_next(lstate, -2) != 0 {
            match lua_type(lstate, -2) {
                LUA_TSTRING => {
                    let mut len: size_t = 0;
                    let s = lua_tolstring(lstate, -2, &raw mut len);
                    if !memchr(s.cast::<c_void>(), 0, len).is_null() {
                        ret.has_string_with_nul = true;
                    }
                    ret.string_keys_num = ret.string_keys_num.wrapping_add(1);
                }
                LUA_TNUMBER => {
                    let n = lua_tonumber(lstate, -2);
                    if n > size_t::MAX as lua_Number || n <= 0.0 || (n as size_t) as lua_Number != n
                    {
                        other_keys_num = other_keys_num.wrapping_add(1);
                    } else {
                        let idx = n as size_t;
                        if idx > ret.maxidx {
                            ret.maxidx = idx;
                        }
                    }
                }
                LUA_TBOOLEAN => {
                    if (lua_toboolean(lstate, -2) != 0) == TYPE_IDX_VALUE {
                        let mut recognised = false;
                        if lua_type(lstate, -1) == LUA_TNUMBER {
                            let n = lua_tonumber(lstate, -1);
                            if n == kObjectTypeFloat as lua_Number
                                || n == kObjectTypeArray as lua_Number
                                || n == kObjectTypeDict as lua_Number
                            {
                                ret.has_type_key = true;
                                ret.type_0 = n as ObjectType;
                                recognised = true;
                            }
                        }
                        if !recognised {
                            other_keys_num = other_keys_num.wrapping_add(1);
                        }
                    } else {
                        has_val_key = true;
                        val_type = lua_type(lstate, -1);
                        if val_type == LUA_TNUMBER {
                            ret.val = lua_tonumber(lstate, -1);
                        }
                    }
                }
                _ => {
                    other_keys_num = other_keys_num.wrapping_add(1);
                }
            }
            tsize = tsize.wrapping_add(1);
            lua_pop(lstate, 1);
        }
        if ret.has_type_key {
            debug_assert!(tsize > 0);
            if ret.type_0 == kObjectTypeFloat && (!has_val_key || val_type != LUA_TNUMBER) {
                ret.type_0 = kObjectTypeNil;
            } else if ret.type_0 == kObjectTypeArray
                && ret.maxidx != 0
                && ret.maxidx
                    != tsize
                        .wrapping_sub(ret.has_type_key as size_t)
                        .wrapping_sub(other_keys_num)
                        .wrapping_sub(has_val_key as size_t)
                        .wrapping_sub(ret.string_keys_num)
            {
                // The keys are not a contiguous run from 1, so the array
                // stops at the last number in the *sequence* -- which is what
                // keeps a table of `{[1]=…, [1000000]=…}` from allocating a
                // million slots.
                ret.maxidx = 0;
                loop {
                    lua_rawgeti(lstate, -1, ret.maxidx as c_int + 1);
                    let past_end = lua_type(lstate, -1) == LUA_TNIL;
                    lua_pop(lstate, 1);
                    if past_end {
                        break;
                    }
                    ret.maxidx = ret.maxidx.wrapping_add(1);
                }
            }
        } else if tsize == 0
            || tsize <= ret.maxidx && other_keys_num == 0 && ret.string_keys_num == 0
        {
            ret.type_0 = kObjectTypeArray;
            if tsize == 0 && lua_getmetatable(lstate, -1) != 0 {
                nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
                if lua_rawequal(lstate, -2, -1) != 0 {
                    ret.type_0 = kObjectTypeDict;
                }
                lua_pop(lstate, 2);
            }
        } else if ret.string_keys_num == tsize {
            ret.type_0 = kObjectTypeDict;
        } else {
            ret.type_0 = kObjectTypeNil;
        }
        ret
    }
}

/// Pop a Lua string, copied out of the Lua state.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top.
pub unsafe fn nlua_pop_string(
    lstate: *mut lua_State,
    _arena: *mut Arena,
) -> Result<String_0, Error> {
    unsafe {
        if lua_type(lstate, -1) != LUA_TSTRING {
            lua_pop(lstate, 1);
            return Err(Error::validation(c"Expected Lua string"));
        }
        let mut len: size_t = 0;
        let data = lua_tolstring(lstate, -1, &raw mut len);
        debug_assert!(!data.is_null());
        // The copy has to happen before the pop: the bytes are the Lua
        // string's own and the collector may take them afterwards.
        let ret = String_0::from_raw_bytes(data, len);
        lua_pop(lstate, 1);
        Ok(ret)
    }
}

/// Pop a Lua number that is an exact api integer.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_integer(
    lstate: *mut lua_State,
    _arena: *mut Arena,
) -> Result<Integer, Error> {
    unsafe {
        if lua_type(lstate, -1) != LUA_TNUMBER {
            lua_pop(lstate, 1);
            return Err(Error::validation(c"Expected Lua number"));
        }
        let n = lua_tonumber(lstate, -1);
        lua_pop(lstate, 1);
        if n > API_INTEGER_MAX as lua_Number
            || n < API_INTEGER_MIN as lua_Number
            || (n as Integer) as lua_Number != n
        {
            return Err(Error::exception(c"Number is not integral"));
        }
        Ok(n as Integer)
    }
}

/// Pop any Lua value for its truthiness.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_boolean(
    lstate: *mut lua_State,
    _arena: *mut Arena,
) -> Result<Boolean, Error> {
    unsafe {
        let ret = lua_toboolean(lstate, -1) != 0;
        lua_pop(lstate, 1);
        Ok(ret)
    }
}

/// [`nlua_pop_boolean`] for a keyset field, where only a boolean, a number or
/// nil is accepted.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_boolean_strict(lstate: *mut lua_State) -> Result<Boolean, Error> {
    unsafe {
        let ret = match lua_type(lstate, -1) {
            LUA_TBOOLEAN => Ok(lua_toboolean(lstate, -1) != 0),
            LUA_TNUMBER => Ok(lua_tonumber(lstate, -1) != 0.0),
            LUA_TNIL => Ok(false),
            _ => Err(Error::validation(c"not a boolean")),
        };
        lua_pop(lstate, 1);
        ret
    }
}

/// Classify the table on top and report when it is not the type the caller
/// asked for. Leaves the stack alone; the caller pops.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top.
#[inline]
unsafe fn nlua_check_type(
    lstate: *mut lua_State,
    report: bool,
    type_0: ObjectType,
) -> (LuaTableProps, Option<Error>) {
    unsafe {
        if lua_type(lstate, -1) != LUA_TTABLE {
            let why = report.then(|| {
                let wanted = if type_0 == kObjectTypeFloat {
                    c"number"
                } else {
                    c"table"
                };
                let wanted = msg_cstr(wanted);
                api_error!(kErrorTypeValidation, "Expected Lua {wanted}")
            });
            return (LuaTableProps::NIL, why);
        }
        let mut table_props = nlua_traverse_table(lstate);
        // An empty table is an array by default; asked for a dictionary, it
        // is one.
        if type_0 == kObjectTypeDict
            && table_props.type_0 == kObjectTypeArray
            && table_props.maxidx == 0
            && !table_props.has_type_key
        {
            table_props.type_0 = kObjectTypeDict;
        }
        let mut why = None;
        if table_props.type_0 != type_0 && report {
            let want = msg_cstr(api_typename(type_0));
            why = Some(api_error!(
                kErrorTypeValidation,
                "Expected {want}-like Lua table"
            ));
        }
        (table_props, why)
    }
}

/// Pop a Lua number, or the `{_TYPE = float, _VAL = n}` special table.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_float(lstate: *mut lua_State, _arena: *mut Arena) -> Result<Float, Error> {
    unsafe {
        if lua_type(lstate, -1) == LUA_TNUMBER {
            let ret = lua_tonumber(lstate, -1);
            lua_pop(lstate, 1);
            return Ok(ret);
        }
        let (table_props, why) = nlua_check_type(lstate, true, kObjectTypeFloat);
        lua_pop(lstate, 1);
        if let Some(why) = why {
            return Err(why);
        }
        if table_props.type_0 != kObjectTypeFloat {
            return Ok(0.0);
        }
        Ok(table_props.val)
    }
}

/// [`nlua_pop_array`] once the table is known to be one.
///
/// # Safety
/// As [`nlua_pop_string`], with `table_props` this table's own.
unsafe fn nlua_pop_array_unchecked(
    lstate: *mut lua_State,
    table_props: LuaTableProps,
    arena: *mut Arena,
) -> Result<Array, Error> {
    unsafe {
        let mut ret = Array::with_capacity(table_props.maxidx);
        if table_props.maxidx == 0 {
            lua_pop(lstate, 1);
            return Ok(ret);
        }

        for i in 1..=table_props.maxidx {
            lua_rawgeti(lstate, -1, i as c_int);
            let val = match nlua_pop_object(lstate, false, arena) {
                Ok(val) => val,
                Err(e) => {
                    lua_pop(lstate, 1);
                    return Err(e);
                }
            };
            ret.push(val);
        }
        lua_pop(lstate, 1);
        Ok(ret)
    }
}

/// Pop an array-shaped Lua table.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_array(lstate: *mut lua_State, arena: *mut Arena) -> Result<Array, Error> {
    unsafe {
        let (table_props, why) = nlua_check_type(lstate, true, kObjectTypeArray);
        if let Some(why) = why {
            return Err(why);
        }
        if table_props.type_0 != kObjectTypeArray {
            return Ok(Array::EMPTY);
        }
        nlua_pop_array_unchecked(lstate, table_props, arena)
    }
}

/// [`nlua_pop_dict`] once the table is known to be one.
///
/// # Safety
/// As [`nlua_pop_array_unchecked`].
unsafe fn nlua_pop_dict_unchecked(
    lstate: *mut lua_State,
    table_props: LuaTableProps,
    ref_0: bool,
    arena: *mut Arena,
) -> Result<ApiDict, Error> {
    unsafe {
        let mut ret = ApiDict::with_capacity(table_props.string_keys_num);
        if table_props.string_keys_num == 0 {
            lua_pop(lstate, 1);
            return Ok(ret);
        }
        lua_pushnil(lstate);
        let mut i: size_t = 0;
        while lua_next(lstate, -2) != 0 && i < table_props.string_keys_num {
            if lua_type(lstate, -2) != LUA_TSTRING {
                lua_pop(lstate, 1);
                continue;
            }
            // The key is popped from a copy, so lua_next still has its own.
            lua_pushvalue(lstate, -2);
            let pair = match nlua_pop_string(lstate, arena) {
                Ok(key) => nlua_pop_object(lstate, ref_0, arena).map(|value| (key, value)),
                Err(e) => {
                    lua_pop(lstate, 1);
                    Err(e)
                }
            };
            let (key, value) = match pair {
                Ok(pair) => pair,
                Err(e) => {
                    lua_pop(lstate, 3);
                    return Err(e);
                }
            };
            ret.insert(key, value);
            i = i.wrapping_add(1);
        }
        lua_pop(lstate, 1);
        Ok(ret)
    }
}

/// Pop a dictionary-shaped Lua table.  With `ref_0`, a function value becomes
/// a `LuaRef` rather than a refusal.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_dict(
    lstate: *mut lua_State,
    ref_0: bool,
    arena: *mut Arena,
) -> Result<ApiDict, Error> {
    unsafe {
        let (table_props, why) = nlua_check_type(lstate, true, kObjectTypeDict);
        if table_props.type_0 != kObjectTypeDict {
            lua_pop(lstate, 1);
            return match why {
                Some(why) => Err(why),
                None => Ok(ApiDict::EMPTY),
            };
        }
        nlua_pop_dict_unchecked(lstate, table_props, ref_0, arena)
    }
}

/// Pop any Lua value as a global reference to it.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_luaref(lstate: *mut lua_State, _arena: *mut Arena) -> Result<LuaRef, Error> {
    unsafe {
        let rv = nlua_ref_global(lstate, -1);
        lua_pop(lstate, 1);
        Ok(rv)
    }
}

/// Pop a buffer, window or tab page id.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_handle(lstate: *mut lua_State, _arena: *mut Arena) -> Result<Handle, Error> {
    unsafe {
        let ret = if lua_type(lstate, -1) != LUA_TNUMBER {
            Err(Error::validation(c"Expected Lua number"))
        } else {
            Ok(lua_tonumber(lstate, -1) as Handle)
        };
        lua_pop(lstate, 1);
        ret
    }
}
