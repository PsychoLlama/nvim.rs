//! Keysets: a Lua options table as a generated `KeySet` struct.
//!
//! [`nlua_pop_keydict`] fills one of the api's generated option structs from
//! a Lua table, driven by the keyset's own [`KeySetLink`] hash function, and
//! [`nlua_push_keydict`] renders one back.  [`nlua_init_types`] installs the
//! names the api's type tags are known by on the Lua side.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::ffi::{CStr, c_char, c_int, c_void};

use super::{
    nlua_pop_array, nlua_pop_boolean_strict, nlua_pop_dict, nlua_pop_float, nlua_pop_handle,
    nlua_pop_integer, nlua_pop_luaref, nlua_pop_object, nlua_pop_string, nlua_push_array,
    nlua_push_dict, nlua_push_object, nlua_push_string, nlua_push_type_idx, nlua_push_val_idx,
};
use crate::api::private::helpers::keyset_field_is_set;
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
    ApiDict, Arena, Array, Boolean, Error, FieldHashfn, Float, Handle, Integer, KeySetLink, LuaRef,
    Object, String_0, kErrorTypeValidation, kObjectTypeArray, kObjectTypeBoolean,
    kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat, kObjectTypeInteger, kObjectTypeLuaRef,
    kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow, lua_Integer,
    lua_Number, lua_State, size_t,
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
) -> Result<(), Error> {
    /// Record the value the pop produced, which is also what records that
    /// the caller named the key. Defined out here so that its own lines are
    /// not counted as unchecked code; every expansion is inside the region
    /// below.
    macro_rules! store {
        ($at:expr, $ty:ty, $value:expr) => {
            *$at.cast::<Option<$ty>>() = Some($value)
        };
    }
    unsafe {
        if lua_type(lstate, -1) != LUA_TTABLE {
            // Upstream writes `lua_pop(L, -1)` here, which expands to
            // `lua_settop(L, 0)` -- it clears the *whole* stack rather than
            // popping the one value. Kept verbatim; see the divergence
            // docket.
            lua_settop(lstate, 0);
            return Err(Error::validation(c"Expected Lua table"));
        }

        let mut failed = None;
        lua_pushnil(lstate);
        while lua_next(lstate, -2) != 0 {
            let mut len: size_t = 0;
            let s = lua_tolstring(lstate, -2, &raw mut len);
            let field: *const KeySetLink = hashy.expect("non-null function pointer")(s, len);
            if field.is_null() {
                let key = c_str_len(s, len).null_as_empty();
                lua_pop(lstate, 3);
                return Err(api_error!(kErrorTypeValidation, "invalid key: {key}"));
            }
            let mem = retval.cast::<c_char>().add((*field).ptr_off);
            // Each arm stores what it popped and answers whether it could;
            // the field's own name is what a refusal is reported against.
            // Storing `Some` is what records that the caller named the key.
            let popped: Result<(), Error> = (|| {
                match (*field).type_0 as ObjectTypeInt {
                    T_ANY => store!(mem, Object, nlua_pop_object(lstate, true, arena)?),
                    T_INTEGER => {
                        // A highlight-group field takes the group's *name* as
                        // well as its id.
                        if (*field).is_hlgroup && lua_type(lstate, -1) == LUA_TSTRING {
                            let mut name_len: size_t = 0;
                            let name = lua_tolstring(lstate, -1, &raw mut name_len);
                            lua_pop(lstate, 1);
                            let id = if name_len > 0 {
                                syn_check_group(::core::slice::from_raw_parts(
                                    name.cast::<u8>(),
                                    name_len,
                                )) as Integer
                            } else {
                                0
                            };
                            store!(mem, Integer, id);
                        } else {
                            store!(mem, Integer, nlua_pop_integer(lstate, arena)?);
                        }
                    }
                    T_BOOLEAN => store!(mem, Boolean, nlua_pop_boolean_strict(lstate)?),
                    T_STRING => store!(mem, String_0, nlua_pop_string(lstate, arena)?),
                    T_FLOAT => store!(mem, Float, nlua_pop_float(lstate, arena)?),
                    T_BUFFER | T_WINDOW | T_TABPAGE => {
                        store!(mem, Handle, nlua_pop_handle(lstate, arena)?);
                    }
                    T_ARRAY => store!(mem, Array, nlua_pop_array(lstate, arena)?),
                    T_DICT => store!(mem, ApiDict, nlua_pop_dict(lstate, false, arena)?),
                    T_LUAREF => store!(mem, LuaRef, nlua_pop_luaref(lstate, arena)?),
                    _ => abort(),
                }
                Ok(())
            })();

            if let Err(e) = popped {
                *err_opt = (*field).str;
                failed = Some(e);
                break;
            }
        }
        lua_pop(lstate, 1);
        match failed {
            Some(e) => Err(e),
            None => Ok(()),
        }
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
    /// The field, which the presence test says is `Some`; the default is
    /// there to spell the arm's type, not because it can be reached.
    /// Defined out here so that its own lines are not counted as unchecked
    /// code; every expansion is inside the region below.
    macro_rules! field {
        ($at:expr, $ty:ty, $absent:expr) => {
            (*$at.cast::<Option<$ty>>()).unwrap_or($absent)
        };
    }
    /// [`field!`] for a field the keyset owns, which the push borrows: a
    /// keydict keeps its values and releases them itself.
    macro_rules! borrowed {
        ($at:expr, $ty:ty, $absent:expr) => {
            (*$at.cast::<Option<$ty>>()).as_ref().unwrap_or($absent)
        };
    }
    unsafe {
        lua_createtable(lstate, 0, 0);
        let mut i: size_t = 0;
        while !(*table.add(i)).str.is_null() {
            let field = table.add(i);
            i = i.wrapping_add(1);

            // A key the caller never named is not an entry of the table.
            let mem = value.cast::<c_char>().add((*field).ptr_off);
            if !keyset_field_is_set(mem.cast(), (*field).type_0) {
                continue;
            }
            lua_pushstring(lstate, (*field).str);
            match (*field).type_0 as ObjectTypeInt {
                T_ANY => {
                    let mut absent = Object::Nil;
                    let object = (*mem.cast::<Option<Object>>())
                        .as_mut()
                        .unwrap_or(&mut absent);
                    nlua_push_object(lstate, object, 0);
                }
                T_INTEGER => lua_pushinteger(lstate, field!(mem, Integer, 0) as lua_Integer),
                T_BUFFER | T_WINDOW | T_TABPAGE => {
                    lua_pushinteger(lstate, field!(mem, Handle, 0) as lua_Integer);
                }
                T_FLOAT => lua_pushnumber(lstate, field!(mem, Float, 0.0)),
                T_BOOLEAN => lua_pushboolean(lstate, c_int::from(field!(mem, Boolean, false))),
                T_STRING => nlua_push_string(lstate, borrowed!(mem, String_0, &String_0::NULL), 0),
                T_ARRAY => {
                    let mut absent = Array::EMPTY;
                    let array = (*mem.cast::<Option<Array>>())
                        .as_mut()
                        .unwrap_or(&mut absent);
                    nlua_push_array(lstate, array, 0);
                }
                T_DICT => {
                    let mut absent = ApiDict::EMPTY;
                    let dict = (*mem.cast::<Option<ApiDict>>())
                        .as_mut()
                        .unwrap_or(&mut absent);
                    nlua_push_dict(lstate, dict, 0);
                }
                T_LUAREF => nlua_pushref(lstate, field!(mem, LuaRef, 0)),
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
