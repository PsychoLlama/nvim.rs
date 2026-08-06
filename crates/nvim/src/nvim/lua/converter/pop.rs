//! The per-type pops, and the table classifier they share.
//!
//! [`nlua_traverse_table`] is the one place a Lua table's shape is decided:
//! it counts string keys against integer keys against the table's own
//! length, checks the `empty_dict` and special metatables, and hands back a
//! [`LuaTableProps`].  Everything else here pops exactly one value of a
//! known api type, with `nlua_check_type` producing the `E5107`-style
//! message when it is the wrong one.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C-unwind" fn nlua_traverse_table(
    lstate: *mut lua_State,
) -> LuaTableProps {
    unsafe {
        let mut tsize: size_t = 0 as size_t;
        let mut val_type: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut has_val_key: bool = false_0 != 0;
        let mut other_keys_num: size_t = 0 as size_t;
        let mut ret: LuaTableProps = LuaTableProps {
            maxidx: 0,
            string_keys_num: 0,
            has_string_with_nul: false,
            type_0: kObjectTypeNil,
            val: 0.,
            has_type_key: false,
        };
        memset(
            &raw mut ret as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<LuaTableProps>(),
        );
        if lua_checkstack(lstate, lua_gettop(lstate) + 3 as ::core::ffi::c_int) == 0 {
            semsg(
                gettext(b"E1502: Lua failed to grow stack to %i\0".as_ptr()
                    as *const ::core::ffi::c_char),
                lua_gettop(lstate) + 2 as ::core::ffi::c_int,
            );
            ret.type_0 = kObjectTypeNil;
            return ret;
        }
        lua_pushnil(lstate);
        while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 {
            match lua_type(lstate, -2 as ::core::ffi::c_int) {
                LUA_TSTRING => {
                    let mut len: size_t = 0;
                    let mut s: *const ::core::ffi::c_char =
                        lua_tolstring(lstate, -2 as ::core::ffi::c_int, &raw mut len);
                    if !memchr(s as *const ::core::ffi::c_void, NUL, len).is_null() {
                        ret.has_string_with_nul = true_0 != 0;
                    }
                    ret.string_keys_num = ret.string_keys_num.wrapping_add(1);
                }
                LUA_TNUMBER => {
                    let n: lua_Number = lua_tonumber(lstate, -2 as ::core::ffi::c_int);
                    if n > SIZE_MAX as lua_Number
                        || n <= 0 as ::core::ffi::c_int as lua_Number
                        || n as size_t as lua_Number != n
                    {
                        other_keys_num = other_keys_num.wrapping_add(1);
                    } else {
                        let idx: size_t = n as size_t;
                        if idx > ret.maxidx {
                            ret.maxidx = idx;
                        }
                    }
                }
                LUA_TBOOLEAN => {
                    let b: bool = lua_toboolean(lstate, -2 as ::core::ffi::c_int) != 0;
                    if b as ::core::ffi::c_int == TYPE_IDX_VALUE {
                        if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNUMBER {
                            let mut n_0: lua_Number =
                                lua_tonumber(lstate, -1 as ::core::ffi::c_int);
                            if n_0 == kObjectTypeFloat as ::core::ffi::c_int as lua_Number
                                || n_0 == kObjectTypeArray as ::core::ffi::c_int as lua_Number
                                || n_0 == kObjectTypeDict as ::core::ffi::c_int as lua_Number
                            {
                                ret.has_type_key = true_0 != 0;
                                ret.type_0 = n_0 as ObjectType;
                            } else {
                                other_keys_num = other_keys_num.wrapping_add(1);
                            }
                        } else {
                            other_keys_num = other_keys_num.wrapping_add(1);
                        }
                    } else {
                        has_val_key = true_0 != 0;
                        val_type = lua_type(lstate, -1 as ::core::ffi::c_int);
                        if val_type == LUA_TNUMBER {
                            ret.val = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
                        }
                    }
                }
                _ => {
                    other_keys_num = other_keys_num.wrapping_add(1);
                }
            }
            tsize = tsize.wrapping_add(1);
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        if ret.has_type_key {
            '_c2rust_label: {
                if tsize > 0 as size_t {
                } else {
                    __assert_fail(
                        b"tsize > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        124 as ::core::ffi::c_uint,
                        b"LuaTableProps nlua_traverse_table(lua_State *const)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if ret.type_0 as ::core::ffi::c_uint
                == kObjectTypeFloat as ::core::ffi::c_int as ::core::ffi::c_uint
                && (!has_val_key || val_type != LUA_TNUMBER)
            {
                ret.type_0 = kObjectTypeNil;
            } else if ret.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if ret.maxidx != 0 as size_t
                    && ret.maxidx
                        != tsize
                            .wrapping_sub(ret.has_type_key as size_t)
                            .wrapping_sub(other_keys_num)
                            .wrapping_sub(has_val_key as size_t)
                            .wrapping_sub(ret.string_keys_num)
                {
                    ret.maxidx = 0 as size_t;
                    loop {
                        lua_rawgeti(
                            lstate,
                            -1 as ::core::ffi::c_int,
                            ret.maxidx as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                        );
                        if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNIL {
                            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                            break;
                        } else {
                            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                            ret.maxidx = ret.maxidx.wrapping_add(1);
                        }
                    }
                }
            }
        } else if tsize == 0 as size_t
            || tsize <= ret.maxidx
                && other_keys_num == 0 as size_t
                && ret.string_keys_num == 0 as size_t
        {
            ret.type_0 = kObjectTypeArray;
            if tsize == 0 as size_t && lua_getmetatable(lstate, -1 as ::core::ffi::c_int) != 0 {
                nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
                if lua_rawequal(lstate, -2 as ::core::ffi::c_int, -1 as ::core::ffi::c_int) != 0 {
                    ret.type_0 = kObjectTypeDict;
                }
                lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        } else if ret.string_keys_num == tsize {
            ret.type_0 = kObjectTypeDict;
        } else {
            ret.type_0 = kObjectTypeNil;
        }
        return ret;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_String(
    mut lstate: *mut lua_State,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    unsafe {
        if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TSTRING {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Expected Lua string\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            };
        }
        let mut ret: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        ret.data = lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut ret.size)
            as *mut ::core::ffi::c_char;
        '_c2rust_label: {
            if !ret.data.is_null() {
            } else {
                __assert_fail(
                    b"ret.data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    797 as ::core::ffi::c_uint,
                    b"String nlua_pop_String(lua_State *, Arena *, Error *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        ret.data = arena_memdupz(arena, ret.data, ret.size);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_Integer(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut err: *mut Error,
) -> Integer {
    unsafe {
        if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TNUMBER {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Expected Lua number\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return 0 as Integer;
        }
        let n: lua_Number = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        if n > API_INTEGER_MAX as lua_Number
            || n < API_INTEGER_MIN as lua_Number
            || n as Integer as lua_Number != n
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"Number is not integral\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return 0 as Integer;
        }
        return n as Integer;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_Boolean(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut _err: *mut Error,
) -> Boolean {
    unsafe {
        let ret: Boolean = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_Boolean_strict(
    mut lstate: *mut lua_State,
    mut err: *mut Error,
) -> Boolean {
    unsafe {
        let mut ret: Boolean = false_0 != 0;
        match lua_type(lstate, -1 as ::core::ffi::c_int) {
            LUA_TBOOLEAN => {
                ret = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
            }
            LUA_TNUMBER => {
                ret = lua_tonumber(lstate, -1 as ::core::ffi::c_int)
                    != 0 as ::core::ffi::c_int as lua_Number;
            }
            LUA_TNIL => {
                ret = false_0 != 0;
            }
            _ => {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"not a boolean\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
}

#[inline]
unsafe extern "C-unwind" fn nlua_check_type(
    lstate: *mut lua_State,
    err: *mut Error,
    type_0: ObjectType,
) -> LuaTableProps {
    unsafe {
        if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TTABLE {
            if !err.is_null() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Expected Lua %s\0".as_ptr() as *const ::core::ffi::c_char,
                    if type_0 as ::core::ffi::c_uint
                        == kObjectTypeFloat as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        b"number\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"table\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
            }
            return LuaTableProps {
                maxidx: 0,
                string_keys_num: 0,
                has_string_with_nul: false,
                type_0: kObjectTypeNil,
                val: 0.,
                has_type_key: false,
            };
        }
        let mut table_props: LuaTableProps = nlua_traverse_table(lstate);
        if type_0 as ::core::ffi::c_uint
            == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
            && table_props.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            && table_props.maxidx == 0 as size_t
            && !table_props.has_type_key
        {
            table_props.type_0 = kObjectTypeDict;
        }
        if table_props.type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
            if !err.is_null() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Expected %s-like Lua table\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(type_0),
                );
            }
        }
        return table_props;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_Float(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut err: *mut Error,
) -> Float {
    unsafe {
        if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNUMBER {
            let ret: Float = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return ret;
        }
        let table_props: LuaTableProps = nlua_check_type(lstate, err, kObjectTypeFloat);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        if table_props.type_0 as ::core::ffi::c_uint
            != kObjectTypeFloat as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 0 as ::core::ffi::c_int as Float;
        }
        return table_props.val;
    }
}

unsafe extern "C-unwind" fn nlua_pop_Array_unchecked(
    lstate: *mut lua_State,
    table_props: LuaTableProps,
    mut arena: *mut Arena,
    err: *mut Error,
) -> Array {
    unsafe {
        let mut ret: Array = arena_array(arena, table_props.maxidx);
        if table_props.maxidx == 0 as size_t {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return ret;
        }
        let mut i: size_t = 1 as size_t;
        while i <= table_props.maxidx {
            let mut val: Object = Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
            lua_rawgeti(lstate, -1 as ::core::ffi::c_int, i as ::core::ffi::c_int);
            val = nlua_pop_Object(lstate, false_0 != 0, arena, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                if arena.is_null() {
                    api_free_array(ret);
                }
                return Array {
                    size: 0 as size_t,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<Object>(),
                };
            }
            let c2rust_fresh14 = ret.size;
            ret.size = ret.size.wrapping_add(1);
            *ret.items.offset(c2rust_fresh14 as isize) = val;
            i = i.wrapping_add(1);
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_Array(
    mut lstate: *mut lua_State,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let table_props: LuaTableProps = nlua_check_type(lstate, err, kObjectTypeArray);
        if table_props.type_0 as ::core::ffi::c_uint
            != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return Array {
                size: 0 as size_t,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        return nlua_pop_Array_unchecked(lstate, table_props, arena, err);
    }
}

unsafe extern "C-unwind" fn nlua_pop_Dict_unchecked(
    mut lstate: *mut lua_State,
    table_props: LuaTableProps,
    mut ref_0: bool,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut ret: Dict = arena_dict(arena, table_props.string_keys_num);
        if table_props.string_keys_num == 0 as size_t {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return ret;
        }
        lua_pushnil(lstate);
        let mut i: size_t = 0 as size_t;
        while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 && i < table_props.string_keys_num {
            if lua_type(lstate, -2 as ::core::ffi::c_int) == LUA_TSTRING {
                lua_pushvalue(lstate, -2 as ::core::ffi::c_int);
                let mut key: String_0 = nlua_pop_String(lstate, arena, err);
                if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                    let mut value: Object = nlua_pop_Object(lstate, ref_0, arena, err);
                    let c2rust_fresh22 = ret.size;
                    ret.size = ret.size.wrapping_add(1);
                    *ret.items.offset(c2rust_fresh22 as isize) = key_value_pair {
                        key: key,
                        value: value,
                    };
                } else {
                    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                }
                if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    if arena.is_null() {
                        api_free_dict(ret);
                    }
                    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    return Dict {
                        size: 0 as size_t,
                        capacity: 0,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    };
                }
                i = i.wrapping_add(1);
            } else {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_Dict(
    mut lstate: *mut lua_State,
    mut ref_0: bool,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let table_props: LuaTableProps = nlua_check_type(lstate, err, kObjectTypeDict);
        if table_props.type_0 as ::core::ffi::c_uint
            != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return Dict {
                size: 0 as size_t,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        return nlua_pop_Dict_unchecked(lstate, table_props, ref_0, arena, err);
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_LuaRef(
    lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut _err: *mut Error,
) -> LuaRef {
    unsafe {
        let mut rv: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return rv;
    }
}

pub unsafe extern "C-unwind" fn nlua_pop_handle(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut err: *mut Error,
) -> handle_T {
    unsafe {
        let mut ret: handle_T = 0;
        if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TNUMBER {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Expected Lua number\0".as_ptr() as *const ::core::ffi::c_char,
            );
            ret = -1 as ::core::ffi::c_int;
        } else {
            ret = lua_tonumber(lstate, -1 as ::core::ffi::c_int) as handle_T;
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
}
