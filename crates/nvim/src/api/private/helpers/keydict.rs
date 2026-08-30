//! Keydicts: the generated structs the typed `nvim_*` signatures take their
//! options as.
//!
//! There is one such struct per function, so the code here is untyped and
//! works off the generated `KeySetLink` table instead: each entry names a
//! key, the offset of its field, the `ObjectType` that field holds, and —
//! for an optional key — the bit in `OptKeySet::is_set_` that records
//! whether the caller supplied it at all.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    EMPTY_DICT, NIL, api_luarefs_free_dict, api_luarefs_free_object, api_object_to_bool,
    api_typename, arena_dict, cstr_as_string, object_to_hl_id,
};
use crate::api::private::validate::err_expected_ptr;
use crate::api_error;
use crate::lua::executor::api_free_luaref;
use crate::message_fmt::c_str_len;
use crate::types::{
    Arena, Array, Boolean, Dict, Error, FieldHashfn, Float, Integer, KeySetLink, LuaRef, Object,
    ObjectType, OptKeySet, OptionalKeys, String_0, handle_T, kErrorTypeNone, kErrorTypeValidation,
    kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage,
    kObjectTypeWindow, key_value_pair, size_t,
};
use ::libc::abort;
use core::ffi::{c_char, c_int, c_void};

/// Whether the caller supplied the optional key at `idx`.
///
/// The keysets carry one `is_set__<kind>_` mask whose bits are indexed by the
/// generated `KEYSET_OPTIDX_<kind>__<key>` constants. c2rust expanded the
/// macro at every use, which is three lines of shifting and casting per
/// question asked.
pub(crate) const fn has_key(set: OptionalKeys, idx: c_int) -> bool {
    set & (1 as OptionalKeys) << idx != 0
}

/// Record that the optional key at `idx` was supplied.
///
/// `PUT_KEY`'s half of the same mask, expanded at every use for the same
/// reason.
pub(crate) const fn set_key(set: OptionalKeys, idx: c_int) -> OptionalKeys {
    set | (1 as OptionalKeys) << idx
}

/// [`api_luarefs_free_object`] over a keydict, walking `table` to find which
/// of its fields can hold a reference.
pub(crate) unsafe fn api_luarefs_free_keydict(dict: *mut c_void, table: *const KeySetLink) {
    // SAFETY: `table` is the generated table for `dict`'s type, so its
    // offsets and types describe `dict`'s fields; it ends with a null name.
    for field in unsafe { keyset_fields(table) } {
        // SAFETY: as above -- the offset is inside `dict` and the row says
        // which type lives there.
        unsafe {
            let mem = dict.cast::<c_char>().add(field.ptr_off);
            match field.type_0 as ObjectType {
                kObjectTypeNil => api_luarefs_free_object(*mem.cast::<Object>()),
                kObjectTypeLuaRef => api_free_luaref(*mem.cast::<LuaRef>()),
                kObjectTypeDict => api_luarefs_free_dict(*mem.cast::<Dict>()),
                _ => {}
            }
        }
    }
}

// -- Keydicts --------------------------------------------------------------

/// The fields of a keydict, as its generated `KeySetLink` table lists them.
/// The table ends with a null name.
unsafe fn keyset_fields(table: *const KeySetLink) -> impl Iterator<Item = &'static KeySetLink> {
    // SAFETY: `table` is one of the generated tables, which are
    // null-terminated by construction.
    let len = unsafe {
        let mut n = 0;
        while !(*table.add(n)).str.is_null() {
            n += 1;
        }
        n
    };
    // SAFETY: as above -- the rows are `static`s in the binary, so the
    // borrow the caller gets outlives any use of it.
    (0..len).map(move |i| unsafe { &*table.add(i) })
}

/// Fill the keydict `retval` from `dict`, type-checking each value against
/// what the field it names holds. False, with `err` set, on the first
/// unknown key or wrong type.
///
/// `retval` is untyped because there is one such struct per API function;
/// `hashy` is that struct's generated perfect-hash lookup and the
/// `KeySetLink` it returns is what says where and what the field is.
pub(crate) unsafe fn api_dict_to_keydict(
    retval: *mut c_void,
    hashy: FieldHashfn,
    dict: Dict,
    err: *mut Error,
) -> bool {
    for i in 0..dict.size {
        // SAFETY: `i` is below `size`, so the pair is inside `items`.
        let (key, given) = unsafe {
            let pair = dict.items.add(i);
            ((*pair).key, (*pair).value)
        };
        // SAFETY: `hashy` is the generated lookup for `retval`'s type, and
        // `key` names its own bytes.
        let field = unsafe { hashy.expect("non-null function pointer")(key.data(), key.len()) };
        if field.is_null() {
            // SAFETY: `key` names its own bytes.
            let key = unsafe { c_str_len(key.data(), key.len()) };
            // SAFETY: `err` is the caller's slot.
            unsafe { *err = api_error!(kErrorTypeValidation, "Invalid key: '{key}'") };
            return false;
        }
        // SAFETY: the lookup answered a row of the generated table, which is
        // a `static` in the binary.
        let field = unsafe { &*field };

        // Optional fields record that they were given, so that the API
        // function can tell "absent" from "set to the default".
        if field.opt_index >= 0 {
            let ks = retval.cast::<OptKeySet>();
            // SAFETY: every keyset with an optional field starts with the
            // mask those fields index.
            unsafe { (*ks).is_set_ |= (1 as OptionalKeys) << field.opt_index };
        }

        // SAFETY: the row's offset names a field of `retval`.
        let mem = unsafe { retval.cast::<c_char>().add(field.ptr_off) };
        let expected = field.type_0 as ObjectType;
        // A mismatch reports the field's name, not the key's: they are
        // the same string.
        let wrong_type = |want: ObjectType| {
            let (want, got) = (api_typename(want), api_typename(given.type_0));
            // SAFETY: the caller's error slot.
            unsafe { *err = err_expected_ptr(field.str, want, Some(got)) };
        };

        match expected {
            // A nil-typed field takes the object as it stands.
            // SAFETY: the row says an `Object` lives at `mem`.
            kObjectTypeNil => unsafe { *mem.cast::<Object>() = given },
            kObjectTypeInteger if field.is_hlgroup => {
                let mut hl_id = 0;
                if given.type_0 != kObjectTypeNil {
                    // SAFETY: `given` is live and `err` the caller's slot.
                    hl_id = unsafe { object_to_hl_id(given, key.data(), err) };
                    // SAFETY: `err` is the caller's slot.
                    if unsafe { (*err).kind() } != kErrorTypeNone {
                        return false;
                    }
                }
                // SAFETY: the row says an `Integer` lives at `mem`.
                unsafe { *mem.cast::<Integer>() = hl_id as Integer };
            }
            kObjectTypeInteger => {
                let Some(number) = given.as_integer() else {
                    wrong_type(kObjectTypeInteger);
                    return false;
                };
                // SAFETY: the row says an `Integer` lives at `mem`.
                unsafe { *mem.cast::<Integer>() = number };
            }
            // A float field takes an integer too.
            kObjectTypeFloat => {
                let widened = given.as_integer().map(|n| n as Float);
                let Some(float) = given.as_float().or(widened) else {
                    wrong_type(kObjectTypeFloat);
                    return false;
                };
                // SAFETY: the row says a `Float` lives at `mem`.
                unsafe { *mem.cast::<Float>() = float };
            }
            kObjectTypeBoolean => {
                // SAFETY: `given` is live, and `field.str`/`err` are the
                // table's name and the caller's slot.
                let on = unsafe { api_object_to_bool(given, field.str, false, err) };
                // SAFETY: the row says a `Boolean` lives at `mem`.
                unsafe { *mem.cast::<Boolean>() = on };
                // SAFETY: `err` is the caller's slot.
                if unsafe { (*err).kind() } != kErrorTypeNone {
                    return false;
                }
            }
            kObjectTypeString => {
                let Some(str) = given.as_string() else {
                    wrong_type(kObjectTypeString);
                    return false;
                };
                // SAFETY: the row says a `String` lives at `mem`.
                unsafe { *mem.cast::<String_0>() = str };
            }
            kObjectTypeArray => {
                let Some(array) = given.as_array() else {
                    wrong_type(kObjectTypeArray);
                    return false;
                };
                // SAFETY: the row says an `Array` lives at `mem`.
                unsafe { *mem.cast::<Array>() = array };
            }
            kObjectTypeDict => {
                // An empty array is how msgpack spells an empty map.
                let empty = given.as_array().is_some_and(|array| array.size == 0);
                let pairs = if empty {
                    Some(EMPTY_DICT)
                } else {
                    given.as_dict()
                };
                let Some(pairs) = pairs else {
                    wrong_type(kObjectTypeDict);
                    return false;
                };
                // SAFETY: the row says a `Dict` lives at `mem`.
                unsafe { *mem.cast::<Dict>() = pairs };
            }
            kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
                // A handle arrives either under its own tag or as a plain
                // integer, and both carry it in the same arm of the union.
                if given.type_0 != kObjectTypeInteger && given.type_0 != expected {
                    wrong_type(expected);
                    return false;
                }
                // SAFETY: as above, and the row says a handle lives at `mem`.
                unsafe { *mem.cast::<handle_T>() = given.data.integer as handle_T };
            }
            kObjectTypeLuaRef => {
                // SAFETY: `key` names its own bytes.
                let key = unsafe { c_str_len(key.data(), key.len()) };
                let e = api_error!(
                    kErrorTypeValidation,
                    "Invalid key: '{key}' is only allowed from Lua"
                );
                // SAFETY: `err` is the caller's slot.
                unsafe { *err = e };
                return false;
            }
            // SAFETY: the generated tables name no other type.
            _ => unsafe { abort() },
        }
    }
    true
}

/// The reverse of [`api_dict_to_keydict`]: the keydict `value` as a plain
/// dictionary, holding only the fields that were set. Lua references are
/// skipped — they mean nothing outside the Lua state.
pub(crate) unsafe fn api_keydict_to_dict(
    value: *mut c_void,
    table: *const KeySetLink,
    max_size: size_t,
    arena: *mut Arena,
) -> Dict {
    let mut rv = arena_dict(arena, max_size);
    // SAFETY: as `api_dict_to_keydict`; `max_size` is the table's length.
    for field in unsafe { keyset_fields(table) } {
        if field.opt_index >= 0 {
            let ks = value.cast::<OptKeySet>();
            // SAFETY: every keyset with an optional field starts with the
            // mask those fields index.
            let is_set = unsafe { (*ks).is_set_ };
            if is_set & ((1 as OptionalKeys) << field.opt_index) == 0 {
                continue;
            }
        }
        // SAFETY: the row's offset names a field of `value`, and its type
        // says what lives there. A Lua reference is still counted as a key,
        // with a nil value, because it means nothing outside the Lua state.
        let val = unsafe {
            let mem = value.cast::<c_char>().add(field.ptr_off);
            match field.type_0 as ObjectType {
                kObjectTypeNil => *mem.cast::<Object>(),
                kObjectTypeInteger => Object::integer(*mem.cast::<Integer>()),
                kObjectTypeFloat => Object::float(*mem.cast::<Float>()),
                kObjectTypeBoolean => Object::boolean(*mem.cast::<Boolean>()),
                kObjectTypeString => Object::string(*mem.cast::<String_0>()),
                kObjectTypeArray => Object::array(*mem.cast::<Array>()),
                kObjectTypeDict => Object::dict(*mem.cast::<Dict>()),
                kObjectTypeBuffer => Object::buffer(*mem.cast::<handle_T>()),
                kObjectTypeWindow => Object::window(*mem.cast::<handle_T>()),
                kObjectTypeTabpage => Object::tabpage(*mem.cast::<handle_T>()),
                kObjectTypeLuaRef => NIL,
                _ => abort(),
            }
        };
        // SAFETY: `rv` was sized for the whole table, and the row's name is
        // a static C string.
        unsafe {
            *rv.items.add(rv.size) = key_value_pair {
                key: cstr_as_string(field.str),
                value: val,
            };
        }
        rv.size += 1;
    }
    rv
}
