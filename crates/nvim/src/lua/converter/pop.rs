//! The per-type pops, and the table classifier they share.
//!
//! [`nlua_traverse_table`] is the one place a Lua table's shape is decided:
//! it counts string keys against integer keys against the table's own
//! length, checks the `empty_dict` and special metatables, and hands back a
//! [`LuaTableProps`].  Everything else here pops exactly one value of a
//! known api type, with `nlua_check_type` producing the `E5107`-style
//! message when it is the wrong one.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_int, c_void};

use super::{API_INTEGER_MAX, API_INTEGER_MIN, LuaTableProps, TYPE_IDX_VALUE, nlua_pop_object};
use crate::api::private::helpers::{
    api_free_array, api_free_dict, api_set_error, api_typename, arena_array, arena_dict,
};
use crate::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::lua::ffi::{
    LUA_TBOOLEAN, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, lua_checkstack, lua_getmetatable,
    lua_gettop, lua_next, lua_pop, lua_pushnil, lua_pushvalue, lua_rawequal, lua_rawgeti,
    lua_toboolean, lua_tolstring, lua_tonumber, lua_type,
};
use crate::main::nlua_global_refs;
use crate::memory::arena_memdupz;
use crate::os::cshim::gettext;
use crate::types::{
    Arena, Array, Boolean, Dict, Error, Float, Integer, LuaRef, ObjectType, String_0, handle_T,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeArray, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeNil, key_value_pair, lua_Number, lua_State, size_t,
};
use ::libc::memchr;

/// Refused when the Lua stack will not grow far enough to walk the table.
const E1502_GROW_STACK: &CStr = c"E1502: Lua failed to grow stack to %i";

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
            semsg_c!(gettext(E1502_GROW_STACK), lua_gettop(lstate) + 2);
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

/// Pop a Lua string, copied into `arena`.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top; `err` the caller's
/// error slot.
pub unsafe fn nlua_pop_string(
    lstate: *mut lua_State,
    arena: *mut Arena,
    err: *mut Error,
) -> String_0 {
    unsafe {
        if lua_type(lstate, -1) != LUA_TSTRING {
            lua_pop(lstate, 1);
            api_set_error(err, kErrorTypeValidation, c"Expected Lua string".as_ptr());
            return String_0::NULL;
        }
        let mut ret = String_0::NULL;
        let data = lua_tolstring(lstate, -1, ret.len_mut()).cast_mut();
        ret.set_data(data);
        debug_assert!(!ret.data().is_null());
        ret.set_data(arena_memdupz(arena, ret.data(), ret.len()));
        lua_pop(lstate, 1);
        ret
    }
}

/// Pop a Lua number that is an exact api integer.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_integer(
    lstate: *mut lua_State,
    _arena: *mut Arena,
    err: *mut Error,
) -> Integer {
    unsafe {
        if lua_type(lstate, -1) != LUA_TNUMBER {
            lua_pop(lstate, 1);
            api_set_error(err, kErrorTypeValidation, c"Expected Lua number".as_ptr());
            return 0;
        }
        let n = lua_tonumber(lstate, -1);
        lua_pop(lstate, 1);
        if n > API_INTEGER_MAX as lua_Number
            || n < API_INTEGER_MIN as lua_Number
            || (n as Integer) as lua_Number != n
        {
            api_set_error(err, kErrorTypeException, c"Number is not integral".as_ptr());
            return 0;
        }
        n as Integer
    }
}

/// Pop any Lua value for its truthiness.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_boolean(
    lstate: *mut lua_State,
    _arena: *mut Arena,
    _err: *mut Error,
) -> Boolean {
    unsafe {
        let ret = lua_toboolean(lstate, -1) != 0;
        lua_pop(lstate, 1);
        ret
    }
}

/// [`nlua_pop_boolean`] for a keyset field, where only a boolean, a number or
/// nil is accepted.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_boolean_strict(lstate: *mut lua_State, err: *mut Error) -> Boolean {
    unsafe {
        let ret = match lua_type(lstate, -1) {
            LUA_TBOOLEAN => lua_toboolean(lstate, -1) != 0,
            LUA_TNUMBER => lua_tonumber(lstate, -1) != 0.0,
            LUA_TNIL => false,
            _ => {
                api_set_error(err, kErrorTypeValidation, c"not a boolean".as_ptr());
                false
            }
        };
        lua_pop(lstate, 1);
        ret
    }
}

/// Classify the table on top and report when it is not the type the caller
/// asked for. Leaves the stack alone; the caller pops.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top; `err` may be null.
#[inline]
unsafe fn nlua_check_type(
    lstate: *mut lua_State,
    err: *mut Error,
    type_0: ObjectType,
) -> LuaTableProps {
    unsafe {
        if lua_type(lstate, -1) != LUA_TTABLE {
            if !err.is_null() {
                let wanted = if type_0 == kObjectTypeFloat {
                    c"number"
                } else {
                    c"table"
                };
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Expected Lua %s".as_ptr(),
                    wanted.as_ptr(),
                );
            }
            return LuaTableProps::NIL;
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
        if table_props.type_0 != type_0 && !err.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Expected %s-like Lua table".as_ptr(),
                api_typename(type_0),
            );
        }
        table_props
    }
}

/// Pop a Lua number, or the `{_TYPE = float, _VAL = n}` special table.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_float(lstate: *mut lua_State, _arena: *mut Arena, err: *mut Error) -> Float {
    unsafe {
        if lua_type(lstate, -1) == LUA_TNUMBER {
            let ret = lua_tonumber(lstate, -1);
            lua_pop(lstate, 1);
            return ret;
        }
        let table_props = nlua_check_type(lstate, err, kObjectTypeFloat);
        lua_pop(lstate, 1);
        if table_props.type_0 != kObjectTypeFloat {
            return 0.0;
        }
        table_props.val
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
    err: *mut Error,
) -> Array {
    unsafe {
        let mut ret = arena_array(arena, table_props.maxidx);
        if table_props.maxidx == 0 {
            lua_pop(lstate, 1);
            return ret;
        }

        for i in 1..=table_props.maxidx {
            lua_rawgeti(lstate, -1, i as c_int);
            let val = nlua_pop_object(lstate, false, arena, err);
            if (*err).type_0 != kErrorTypeNone {
                lua_pop(lstate, 1);
                if arena.is_null() {
                    api_free_array(ret);
                }
                return Array::EMPTY;
            }
            *ret.items.add(ret.size) = val;
            ret.size = ret.size.wrapping_add(1);
        }
        lua_pop(lstate, 1);
        ret
    }
}

/// Pop an array-shaped Lua table.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_array(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> Array {
    unsafe {
        let table_props = nlua_check_type(lstate, err, kObjectTypeArray);
        if table_props.type_0 != kObjectTypeArray {
            return Array::EMPTY;
        }
        nlua_pop_array_unchecked(lstate, table_props, arena, err)
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
    err: *mut Error,
) -> Dict {
    unsafe {
        let mut ret = arena_dict(arena, table_props.string_keys_num);
        if table_props.string_keys_num == 0 {
            lua_pop(lstate, 1);
            return ret;
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
            let key = nlua_pop_string(lstate, arena, err);
            if (*err).type_0 == kErrorTypeNone {
                let value = nlua_pop_object(lstate, ref_0, arena, err);
                *ret.items.add(ret.size) = key_value_pair { key, value };
                ret.size = ret.size.wrapping_add(1);
            } else {
                lua_pop(lstate, 1);
            }
            if (*err).type_0 != kErrorTypeNone {
                if arena.is_null() {
                    api_free_dict(ret);
                }
                lua_pop(lstate, 3);
                return Dict::EMPTY;
            }
            i = i.wrapping_add(1);
        }
        lua_pop(lstate, 1);
        ret
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
    err: *mut Error,
) -> Dict {
    unsafe {
        let table_props = nlua_check_type(lstate, err, kObjectTypeDict);
        if table_props.type_0 != kObjectTypeDict {
            lua_pop(lstate, 1);
            return Dict::EMPTY;
        }
        nlua_pop_dict_unchecked(lstate, table_props, ref_0, arena, err)
    }
}

/// Pop any Lua value as a global reference to it.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_luaref(
    lstate: *mut lua_State,
    _arena: *mut Arena,
    _err: *mut Error,
) -> LuaRef {
    unsafe {
        let rv = nlua_ref_global(lstate, -1);
        lua_pop(lstate, 1);
        rv
    }
}

/// Pop a buffer, window or tab page id.
///
/// # Safety
/// As [`nlua_pop_string`].
pub unsafe fn nlua_pop_handle(
    lstate: *mut lua_State,
    _arena: *mut Arena,
    err: *mut Error,
) -> handle_T {
    unsafe {
        let ret = if lua_type(lstate, -1) != LUA_TNUMBER {
            api_set_error(err, kErrorTypeValidation, c"Expected Lua number".as_ptr());
            -1
        } else {
            lua_tonumber(lstate, -1) as handle_T
        };
        lua_pop(lstate, 1);
        ret
    }
}
