//! The per-type pops, and the table classifier they share.
//!
//! [`classify_table`] is the one place a Lua table's shape is decided: it
//! counts string keys against integer keys against the table's own length,
//! checks the `empty_dict` and special metatables, and hands back a
//! [`LuaTableProps`]. Everything else here pops exactly one value of a known
//! api type, with `check_type` producing the `E5107`-style message when it is
//! the wrong one.

#![forbid(unsafe_code)]

use crate::semsg;
use core::ffi::c_int;

use super::{API_INTEGER_MAX, API_INTEGER_MIN, LuaTableProps, TYPE_IDX_VALUE, pop_object};
use crate::api::private::helpers::api_typename;
use crate::api_error;
use crate::lua::stack::{At, Kind, Stack};
use crate::memory::memchr_in;
use crate::message_fmt::msg_cstr;
use crate::types::{
    ApiDict, Array, Boolean, DictKey, Error, Float, Handle, Integer, LuaRef, ObjectType, String_0,
    kErrorTypeValidation, kObjectTypeArray, kObjectTypeDict, kObjectTypeFloat, kObjectTypeNil,
    lua_Number, size_t,
};

/// How many Lua slots one turn of a `pop` walk may need: a table's next key
/// and value, plus room for the `vim.NIL` a userdata leaf compares itself
/// with.
pub(super) const TURN_SLOTS: c_int = 3;

/// Where a table walk keeps the key `next` handed back, and its value.
const KEY: At = At::from_top(2);
const VALUE: At = At::TOP;

/// Classify the table on top of the stack.
///
/// Leaves the stack exactly as it found it. Both `pop` walks call this before
/// deciding what to build, so every rule about what a Lua table *means*
/// lives here and nowhere else.
pub(crate) fn classify_table(stack: &mut Stack) -> LuaTableProps {
    let mut tsize: size_t = 0; // Total number of keys.
    let mut val_kind = Kind::None; // If has_val_key: what the value is.
    let mut has_val_key = false; // Whether the `_VAL` key was found.
    // Keys that are neither string, integral nor one of the two special
    // boolean keys.
    let mut other_keys_num: size_t = 0;
    let mut ret = LuaTableProps::NIL;
    if !stack.reserve(stack.top().index() + 3) {
        let need = stack.top().index() + 2;
        semsg!("E1502: Lua failed to grow stack to {need}");
        ret.type_0 = kObjectTypeNil;
        return ret;
    }
    stack.push_nil();
    while stack.next(At::from_top(2)) {
        match stack.kind(KEY) {
            Kind::String => {
                if memchr_in(stack.to_bytes(KEY).unwrap_or_default(), 0).is_some() {
                    ret.has_string_with_nul = true;
                }
                ret.string_keys_num += 1;
            }
            Kind::Number => {
                let n = stack.to_number(KEY);
                if n > size_t::MAX as lua_Number || n <= 0.0 || (n as size_t) as lua_Number != n {
                    other_keys_num += 1;
                } else {
                    ret.maxidx = ret.maxidx.max(n as size_t);
                }
            }
            Kind::Boolean => {
                if stack.to_boolean(KEY) == TYPE_IDX_VALUE {
                    let is_number = stack.kind(VALUE) == Kind::Number;
                    let n = if is_number {
                        stack.to_number(VALUE)
                    } else {
                        0.0
                    };
                    let recognised = is_number
                        && (n == kObjectTypeFloat as lua_Number
                            || n == kObjectTypeArray as lua_Number
                            || n == kObjectTypeDict as lua_Number);
                    if recognised {
                        ret.has_type_key = true;
                        ret.type_0 = n as ObjectType;
                    } else {
                        other_keys_num += 1;
                    }
                } else {
                    has_val_key = true;
                    val_kind = stack.kind(VALUE);
                    if val_kind == Kind::Number {
                        ret.val = stack.to_number(VALUE);
                    }
                }
            }
            _ => other_keys_num += 1,
        }
        tsize += 1;
        stack.pop(1);
    }
    if ret.has_type_key {
        debug_assert!(tsize > 0);
        if ret.type_0 == kObjectTypeFloat && (!has_val_key || val_kind != Kind::Number) {
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
            // The keys are not a contiguous run from 1, so the array stops at
            // the last number in the *sequence* -- which is what keeps a
            // table of `{[1]=…, [1000000]=…}` from allocating a million
            // slots.
            ret.maxidx = 0;
            loop {
                stack.raw_get_index(At::TOP, ret.maxidx as c_int + 1);
                let past_end = stack.is_nil(At::TOP);
                stack.pop(1);
                if past_end {
                    break;
                }
                ret.maxidx += 1;
            }
        }
    } else if tsize == 0 || tsize <= ret.maxidx && other_keys_num == 0 && ret.string_keys_num == 0 {
        ret.type_0 = kObjectTypeArray;
        if tsize == 0 && stack.get_metatable(At::TOP) {
            stack.push_empty_dict_metatable();
            if stack.raw_equal(At::from_top(2), At::TOP) {
                ret.type_0 = kObjectTypeDict;
            }
            stack.pop(2);
        }
    } else if ret.string_keys_num == tsize {
        ret.type_0 = kObjectTypeDict;
    } else {
        ret.type_0 = kObjectTypeNil;
    }
    ret
}

/// Pop a Lua string, copied out of the Lua state.
pub fn pop_string(stack: &mut Stack) -> Result<String_0, Error> {
    // The copy has to happen before the pop: the bytes are the Lua string's
    // own and the collector may take them afterwards.
    let popped = stack.string(At::TOP).map(String_0::from_bytes);
    stack.pop(1);
    popped.ok_or_else(|| Error::validation(c"Expected Lua string"))
}

/// Pop a Lua number that is an exact api integer.
pub fn pop_integer(stack: &mut Stack) -> Result<Integer, Error> {
    let is_number = stack.kind(At::TOP) == Kind::Number;
    let n = stack.to_number(At::TOP);
    stack.pop(1);
    if !is_number {
        return Err(Error::validation(c"Expected Lua number"));
    }
    if n > API_INTEGER_MAX as lua_Number
        || n < API_INTEGER_MIN as lua_Number
        || (n as Integer) as lua_Number != n
    {
        return Err(Error::exception(c"Number is not integral"));
    }
    Ok(n as Integer)
}

/// Pop any Lua value for its truthiness.
pub fn pop_boolean(stack: &mut Stack) -> Result<Boolean, Error> {
    let ret = stack.to_boolean(At::TOP);
    stack.pop(1);
    Ok(ret)
}

/// [`pop_boolean`] for a keyset field, where only a boolean, a number or nil
/// is accepted.
pub fn pop_boolean_strict(stack: &mut Stack) -> Result<Boolean, Error> {
    let ret = match stack.kind(At::TOP) {
        Kind::Boolean => Ok(stack.to_boolean(At::TOP)),
        Kind::Number => Ok(stack.to_number(At::TOP) != 0.0),
        Kind::Nil => Ok(false),
        _ => Err(Error::validation(c"not a boolean")),
    };
    stack.pop(1);
    ret
}

/// Classify the table on top and report when it is not the type the caller
/// asked for. Leaves the stack alone; the caller pops.
#[inline]
fn check_type(
    stack: &mut Stack,
    report: bool,
    type_0: ObjectType,
) -> (LuaTableProps, Option<Error>) {
    if stack.kind(At::TOP) != Kind::Table {
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
    let mut table_props = classify_table(stack);
    // An empty table is an array by default; asked for a dictionary, it is
    // one.
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

/// Pop a Lua number, or the `{_TYPE = float, _VAL = n}` special table.
pub fn pop_float(stack: &mut Stack) -> Result<Float, Error> {
    if stack.kind(At::TOP) == Kind::Number {
        let ret = stack.to_number(At::TOP);
        stack.pop(1);
        return Ok(ret);
    }
    let (table_props, why) = check_type(stack, true, kObjectTypeFloat);
    stack.pop(1);
    if let Some(why) = why {
        return Err(why);
    }
    if table_props.type_0 != kObjectTypeFloat {
        return Ok(0.0);
    }
    Ok(table_props.val)
}

/// [`pop_array`] once the table is known to be one.
fn pop_array_unchecked(stack: &mut Stack, table_props: LuaTableProps) -> Result<Array, Error> {
    let mut ret = Array::with_capacity(table_props.maxidx);
    for i in 1..=table_props.maxidx {
        stack.raw_get_index(At::TOP, i as c_int);
        match pop_object(stack, false) {
            Ok(val) => ret.push(val),
            Err(e) => {
                stack.pop(1);
                return Err(e);
            }
        }
    }
    stack.pop(1);
    Ok(ret)
}

/// Pop an array-shaped Lua table.
pub fn pop_array(stack: &mut Stack) -> Result<Array, Error> {
    let (table_props, why) = check_type(stack, true, kObjectTypeArray);
    if let Some(why) = why {
        return Err(why);
    }
    if table_props.type_0 != kObjectTypeArray {
        return Ok(Array::EMPTY);
    }
    pop_array_unchecked(stack, table_props)
}

/// [`pop_dict`] once the table is known to be one.
fn pop_dict_unchecked(
    stack: &mut Stack,
    table_props: LuaTableProps,
    as_ref: bool,
) -> Result<ApiDict, Error> {
    let mut ret = ApiDict::with_capacity(table_props.string_keys_num);
    if table_props.string_keys_num == 0 {
        stack.pop(1);
        return Ok(ret);
    }
    stack.push_nil();
    let mut i: size_t = 0;
    while stack.next(At::from_top(2)) && i < table_props.string_keys_num {
        if stack.kind(KEY) != Kind::String {
            stack.pop(1);
            continue;
        }
        // A dictionary key, not a value: it copies into the entry itself
        // when it is short, which an API key nearly always is, where
        // `pop_string` would be an `xmalloc` per key. The copy precedes the
        // conversion of the value, which may pop past it.
        let key = DictKey::new(stack.to_bytes(KEY).unwrap_or_default());
        let value = match pop_object(stack, as_ref) {
            Ok(value) => value,
            Err(e) => {
                stack.pop(2);
                return Err(e);
            }
        };
        ret.insert(key, value);
        i += 1;
    }
    stack.pop(1);
    Ok(ret)
}

/// Pop a dictionary-shaped Lua table. With `as_ref`, a function value becomes
/// a `LuaRef` rather than a refusal.
pub fn pop_dict(stack: &mut Stack, as_ref: bool) -> Result<ApiDict, Error> {
    let (table_props, why) = check_type(stack, true, kObjectTypeDict);
    if table_props.type_0 != kObjectTypeDict {
        stack.pop(1);
        return match why {
            Some(why) => Err(why),
            None => Ok(ApiDict::EMPTY),
        };
    }
    pop_dict_unchecked(stack, table_props, as_ref)
}

/// Pop any Lua value as a global reference to it.
pub fn pop_luaref(stack: &mut Stack) -> Result<LuaRef, Error> {
    let reference = stack.reference(At::TOP);
    stack.pop(1);
    Ok(reference)
}

/// Pop a buffer, window or tab page id.
pub fn pop_handle(stack: &mut Stack) -> Result<Handle, Error> {
    let ret = if stack.kind(At::TOP) != Kind::Number {
        Err(Error::validation(c"Expected Lua number"))
    } else {
        Ok(stack.to_number(At::TOP) as Handle)
    };
    stack.pop(1);
    ret
}
