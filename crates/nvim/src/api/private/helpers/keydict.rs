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
    api_set_error, api_typename, arena_dict, cstr_as_string, object_to_hl_id,
};
use crate::api::private::validate::api_err_exp;
use crate::lua::executor::api_free_luaref;
use crate::os::libc::abort;
use crate::types::{
    Arena, Array, Boolean, Dict, Error, FieldHashfn, Float, Integer, KeySetLink, LuaRef, Object,
    ObjectType, OptKeySet, OptionalKeys, String_0, handle_T, kErrorTypeNone, kErrorTypeValidation,
    kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage,
    kObjectTypeWindow, key_value_pair, object, object_data, size_t,
};
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
pub(crate) unsafe fn api_luarefs_free_keydict(dict: *mut c_void, table: *mut KeySetLink) {
    // SAFETY: `table` is the generated table for `dict`'s type, so its
    // offsets and types describe `dict`'s fields; it ends with a null name.
    unsafe {
        for field in keyset_fields(table) {
            let mem = (dict as *mut c_char).add((*field).ptr_off);
            match (*field).type_0 as ObjectType {
                kObjectTypeNil => api_luarefs_free_object(*(mem as *mut Object)),
                kObjectTypeLuaRef => api_free_luaref(*(mem as *mut LuaRef)),
                kObjectTypeDict => api_luarefs_free_dict(*(mem as *mut Dict)),
                _ => {}
            }
        }
    }
}

// -- Keydicts --------------------------------------------------------------

/// The fields of a keydict, as its generated `KeySetLink` table lists them.
/// The table ends with a null name.
unsafe fn keyset_fields(table: *mut KeySetLink) -> impl Iterator<Item = *mut KeySetLink> {
    // SAFETY: `table` is one of the generated tables, which are
    // null-terminated by construction.
    let len = unsafe {
        let mut n = 0;
        while !(*table.add(n)).str.is_null() {
            n += 1;
        }
        n
    };
    (0..len).map(move |i| unsafe { table.add(i) })
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
    // SAFETY: `hashy` is the generated lookup for `retval`'s type, so the
    // offsets it hands back are inside `retval`.
    unsafe {
        for i in 0..dict.size {
            let k = (*dict.items.add(i)).key;
            let field = hashy.expect("non-null function pointer")(k.data, k.size);
            if field.is_null() {
                let fmt = c"Invalid key: '%.*s'".as_ptr();
                api_set_error(err, kErrorTypeValidation, fmt, k.size as c_int, k.data);
                return false;
            }
            // Optional fields record that they were given, so that the API
            // function can tell "absent" from "set to the default".
            if (*field).opt_index >= 0 {
                let ks = retval as *mut OptKeySet;
                (*ks).is_set_ |= (1 as OptionalKeys) << (*field).opt_index;
            }

            let mem = (retval as *mut c_char).add((*field).ptr_off);
            let value = &raw mut (*dict.items.add(i)).value;
            let expected = (*field).type_0 as ObjectType;
            // A mismatch reports the field's name, not the key's: they are
            // the same string.
            let mut wrong_type = |want: ObjectType| {
                api_err_exp(
                    err,
                    (*field).str,
                    api_typename(want),
                    api_typename((*value).type_0),
                );
            };

            match expected {
                // A nil-typed field takes the object as it stands.
                kObjectTypeNil => *(mem as *mut Object) = *value,
                kObjectTypeInteger if (*field).is_hlgroup => {
                    let mut hl_id = 0;
                    if (*value).type_0 != kObjectTypeNil {
                        hl_id = object_to_hl_id(*value, k.data, err);
                        if (*err).type_0 != kErrorTypeNone {
                            return false;
                        }
                    }
                    *(mem as *mut Integer) = hl_id as Integer;
                }
                kObjectTypeInteger => {
                    if (*value).type_0 != kObjectTypeInteger {
                        wrong_type(kObjectTypeInteger);
                        return false;
                    }
                    *(mem as *mut Integer) = (*value).data.integer;
                }
                // A float field takes an integer too.
                kObjectTypeFloat => match (*value).type_0 {
                    kObjectTypeInteger => *(mem as *mut Float) = (*value).data.integer as Float,
                    kObjectTypeFloat => *(mem as *mut Float) = (*value).data.floating,
                    _ => {
                        wrong_type(kObjectTypeFloat);
                        return false;
                    }
                },
                kObjectTypeBoolean => {
                    *(mem as *mut Boolean) = api_object_to_bool(*value, (*field).str, false, err);
                    if (*err).type_0 != kErrorTypeNone {
                        return false;
                    }
                }
                kObjectTypeString => {
                    if (*value).type_0 != kObjectTypeString {
                        wrong_type(kObjectTypeString);
                        return false;
                    }
                    *(mem as *mut String_0) = (*value).data.string;
                }
                kObjectTypeArray => {
                    if (*value).type_0 != kObjectTypeArray {
                        wrong_type(kObjectTypeArray);
                        return false;
                    }
                    *(mem as *mut Array) = (*value).data.array;
                }
                // An empty array is how msgpack spells an empty map.
                kObjectTypeDict => match (*value).type_0 {
                    kObjectTypeArray if (*value).data.array.size == 0 => {
                        *(mem as *mut Dict) = EMPTY_DICT;
                    }
                    kObjectTypeDict => *(mem as *mut Dict) = (*value).data.dict,
                    _ => {
                        wrong_type(kObjectTypeDict);
                        return false;
                    }
                },
                kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
                    if (*value).type_0 != kObjectTypeInteger && (*value).type_0 != expected {
                        wrong_type(expected);
                        return false;
                    }
                    *(mem as *mut handle_T) = (*value).data.integer as handle_T;
                }
                kObjectTypeLuaRef => {
                    let fmt = c"Invalid key: '%.*s' is only allowed from Lua".as_ptr();
                    api_set_error(err, kErrorTypeValidation, fmt, k.size as c_int, k.data);
                    return false;
                }
                _ => abort(),
            }
        }
        true
    }
}

/// The reverse of [`api_dict_to_keydict`]: the keydict `value` as a plain
/// dictionary, holding only the fields that were set. Lua references are
/// skipped — they mean nothing outside the Lua state.
pub(crate) unsafe fn api_keydict_to_dict(
    value: *mut c_void,
    table: *mut KeySetLink,
    max_size: size_t,
    arena: *mut Arena,
) -> Dict {
    // SAFETY: as `api_dict_to_keydict`; `max_size` is the table's length.
    unsafe {
        let mut rv = arena_dict(arena, max_size);
        for field in keyset_fields(table) {
            if (*field).opt_index >= 0 {
                let ks = value as *mut OptKeySet;
                if (*ks).is_set_ & ((1 as OptionalKeys) << (*field).opt_index) == 0 {
                    continue;
                }
            }
            let mem = (value as *mut c_char).add((*field).ptr_off);
            let mut val = NIL;
            match (*field).type_0 as ObjectType {
                kObjectTypeNil => val = *(mem as *mut Object),
                kObjectTypeInteger => {
                    val = object {
                        type_0: kObjectTypeInteger,
                        data: object_data {
                            integer: *(mem as *mut Integer),
                        },
                    };
                }
                kObjectTypeFloat => {
                    val = object {
                        type_0: kObjectTypeFloat,
                        data: object_data {
                            floating: *(mem as *mut Float),
                        },
                    };
                }
                kObjectTypeBoolean => {
                    val = object {
                        type_0: kObjectTypeBoolean,
                        data: object_data {
                            boolean: *(mem as *mut Boolean),
                        },
                    };
                }
                kObjectTypeString => {
                    val = object {
                        type_0: kObjectTypeString,
                        data: object_data {
                            string: *(mem as *mut String_0),
                        },
                    };
                }
                kObjectTypeArray => {
                    val = object {
                        type_0: kObjectTypeArray,
                        data: object_data {
                            array: *(mem as *mut Array),
                        },
                    };
                }
                kObjectTypeDict => {
                    val = object {
                        type_0: kObjectTypeDict,
                        data: object_data {
                            dict: *(mem as *mut Dict),
                        },
                    };
                }
                handle @ (kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage) => {
                    val.data.integer = *(mem as *mut handle_T) as Integer;
                    val.type_0 = handle;
                }
                // A Lua reference is still counted as a key, with a nil value.
                kObjectTypeLuaRef => {}
                _ => abort(),
            }
            *rv.items.add(rv.size) = key_value_pair {
                key: cstr_as_string((*field).str),
                value: val,
            };
            rv.size += 1;
        }
        rv
    }
}
