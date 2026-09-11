//! Keydicts: the generated structs the typed `nvim_*` signatures take their
//! options as.
//!
//! There is one such struct per function, so the code here is untyped and
//! works off the generated `KeySetLink` table instead: each entry names a
//! key, the offset of its field and the `ObjectType` that field holds. Every
//! field is an `Option`, so the row's type says `Option<T>` and `None` is
//! the key the caller did not name.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::{
    EMPTY_DICT, api_luarefs_free_dict, api_luarefs_free_object, api_object_to_bool, api_typename,
    arena_dict, cstr_as_string, object_to_hl_id,
};
use crate::api::private::validate::err_expected;
use crate::api_error;
use crate::cstr;
use crate::lua::executor::api_free_luaref;
use crate::message_fmt::c_str_len;
use crate::msgpack_rpc::unpacker::kUnpackTypeStringArray;
use crate::narrow::number_as_int;
use crate::types::{
    ApiDict, Arena, Array, Boolean, Error, FieldHashfn, Float, Handle, Integer, KeySetLink, LuaRef,
    Object, ObjectType, String_0, StringArray, kErrorTypeValidation, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat, kObjectTypeInteger,
    kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow,
    key_value_pair, size_t,
};
use ::libc::abort;
use core::ffi::{c_char, c_int, c_void};

/// Whether the keyset field at `mem` has been filled in.
///
/// Every keyset field is an `Option`, so "did the caller name this key?" is
/// one question -- but not one question the *bytes* can answer: an
/// `Option<Integer>` and an `Option<Boolean>` carry their `None` in
/// different places and at different widths. So the row's type picks the
/// arm, exactly as the walks that read and write the field do.
///
/// `kUnpackTypeStringArray` is ShaDa's own tag, which is why this takes the
/// row's `c_int` rather than an [`ObjectType`].
///
/// # Safety
///
/// `mem` must point at a keyset field of the kind `type_0` names.
pub(crate) unsafe fn keyset_field_is_set(mem: *const c_void, type_0: c_int) -> bool {
    macro_rules! set {
        ($ty:ty) => {
            // SAFETY: the caller's promise: a field of the kind the row names.
            unsafe { (*mem.cast::<Option<$ty>>()).is_some() }
        };
    }
    match type_0.cast_unsigned() {
        kObjectTypeNil => set!(Object),
        kObjectTypeBoolean => set!(Boolean),
        kObjectTypeInteger => set!(Integer),
        kObjectTypeFloat => set!(Float),
        kObjectTypeString => set!(String_0),
        kObjectTypeArray => set!(Array),
        kObjectTypeDict => set!(ApiDict),
        kObjectTypeLuaRef => set!(LuaRef),
        kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => set!(Handle),
        _ if type_0 == kUnpackTypeStringArray => set!(StringArray),
        // SAFETY: the generated tables name no other type.
        _ => unsafe { abort() },
    }
}

/// [`api_luarefs_free_object`] over a keydict, walking `table` to find which
/// of its fields can hold a reference.
///
/// # Safety
///
/// `dict` must point at the keydict `table` describes, and `table` at the
/// generated `KeySetLink` table for that keydict's type -- the offsets and
/// field types are read from it and applied to `dict`, and the walk stops at
/// the row with a null name.
pub(crate) unsafe fn api_luarefs_free_keydict(dict: *mut c_void, table: *const KeySetLink) {
    // SAFETY: `table` is the generated table for `dict`'s type, so its
    // offsets and types describe `dict`'s fields; it ends with a null name.
    for field in unsafe { keyset_fields(table) } {
        // SAFETY: as above -- the offset is inside `dict` and the row says
        // which type lives there.
        unsafe {
            let mem = dict.cast::<c_char>().add(field.ptr_off);
            match field.type_0.cast_unsigned() {
                kObjectTypeNil => {
                    if let Some(object) = *mem.cast::<Option<Object>>() {
                        api_luarefs_free_object(object);
                    }
                }
                kObjectTypeLuaRef => {
                    if let Some(luaref) = *mem.cast::<Option<LuaRef>>() {
                        api_free_luaref(luaref);
                    }
                }
                kObjectTypeDict => {
                    if let Some(dict) = *mem.cast::<Option<ApiDict>>() {
                        api_luarefs_free_dict(dict);
                    }
                }
                _ => {}
            }
        }
    }
}

// -- Keydicts --------------------------------------------------------------

/// The fields of a keydict, as its generated `KeySetLink` table lists them.
/// The table ends with a null name.
///
/// # Safety
///
/// `table` must point at one of the generated `KeySetLink` tables, which end
/// with a row whose name is null; the iterator borrows it for `'static`, so
/// it must be one of those statics and not a temporary.
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
/// what the field it names holds. Refuses at the first unknown key or wrong
/// type.
///
/// `retval` is untyped because there is one such struct per API function;
/// `hashy` is that struct's generated perfect-hash lookup and the
/// `KeySetLink` it returns is what says where and what the field is.
///
/// # Safety
///
/// `retval` must point at the keydict struct `hashy` was generated for, live
/// and unaliased for the call: every field this writes is found by offset
/// from it.
pub(crate) unsafe fn api_dict_to_keydict(
    retval: *mut c_void,
    hashy: FieldHashfn,
    dict: ApiDict,
) -> Result<(), Error> {
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
            let key = unsafe { c_str_len(key.data(), key.len()) }.null_as_empty();
            return Err(api_error!(kErrorTypeValidation, "Invalid key: '{key}'"));
        }
        // SAFETY: the lookup answered a row of the generated table, which is
        // a `static` in the binary.
        let field = unsafe { &*field };

        // SAFETY: the row's offset names a field of `retval`. Writing
        // `Some` there is what records that the caller named the key: a
        // field nobody wrote is still the `None` `Default` left.
        let mem = unsafe { retval.cast::<c_char>().add(field.ptr_off) };
        let expected: ObjectType = field.type_0.cast_unsigned();
        // A mismatch reports the field's name, not the key's: they are
        // the same string.
        let wrong_type = |want: ObjectType| {
            let (want, got) = (api_typename(want), api_typename(given.kind()));
            // SAFETY: a keyset field's name is a static NUL-terminated string.
            let name = unsafe { cstr::at(field.str) };
            err_expected(name, want, Some(got))
        };

        match expected {
            // A nil-typed field takes the object as it stands.
            // SAFETY: the row says an `Object` lives at `mem`.
            kObjectTypeNil => unsafe { *mem.cast::<Option<Object>>() = Some(given) },
            kObjectTypeInteger if field.is_hlgroup => {
                let mut hl_id = 0;
                if !given.is_nil() {
                    // SAFETY: `given` is live.
                    hl_id = unsafe { object_to_hl_id(given, key.data()) }?;
                }
                // SAFETY: the row says an `Option<Integer>` lives at `mem`.
                unsafe { *mem.cast::<Option<Integer>>() = Some(Integer::from(hl_id)) };
            }
            kObjectTypeInteger => {
                let Some(number) = given.as_integer() else {
                    return Err(wrong_type(kObjectTypeInteger));
                };
                // SAFETY: the row says an `Option<Integer>` lives at `mem`.
                unsafe { *mem.cast::<Option<Integer>>() = Some(number) };
            }
            // A float field takes an integer too.
            kObjectTypeFloat => {
                let widened = given.as_integer().map(|n| n as Float);
                let Some(float) = given.as_float().or(widened) else {
                    return Err(wrong_type(kObjectTypeFloat));
                };
                // SAFETY: the row says an `Option<Float>` lives at `mem`.
                unsafe { *mem.cast::<Option<Float>>() = Some(float) };
            }
            kObjectTypeBoolean => {
                // SAFETY: `given` is live and `field.str` is the table's name.
                let on = unsafe { api_object_to_bool(given, field.str, false) }?;
                // SAFETY: the row says an `Option<Boolean>` lives at `mem`.
                unsafe { *mem.cast::<Option<Boolean>>() = Some(on) };
            }
            kObjectTypeString => {
                let Some(str) = given.as_string() else {
                    return Err(wrong_type(kObjectTypeString));
                };
                // SAFETY: the row says an `Option<String>` lives at `mem`.
                unsafe { *mem.cast::<Option<String_0>>() = Some(str) };
            }
            kObjectTypeArray => {
                let Some(array) = given.as_array() else {
                    return Err(wrong_type(kObjectTypeArray));
                };
                // SAFETY: the row says an `Option<Array>` lives at `mem`.
                unsafe { *mem.cast::<Option<Array>>() = Some(array) };
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
                    return Err(wrong_type(kObjectTypeDict));
                };
                // SAFETY: the row says an `Option<ApiDict>` lives at `mem`.
                unsafe { *mem.cast::<Option<ApiDict>>() = Some(pairs) };
            }
            kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
                // A handle arrives either under its own variant or as a plain
                // integer, and both carry it as an `Integer`.
                let handle = match given {
                    Object::Integer(n) => n,
                    _ if given.kind() == expected => given.as_handle().unwrap_or(0),
                    _ => return Err(wrong_type(expected)),
                };
                // SAFETY: the row says an `Option<Handle>` lives at `mem`.
                unsafe { *mem.cast::<Option<Handle>>() = Some(number_as_int(handle)) };
            }
            kObjectTypeLuaRef => {
                // SAFETY: `key` names its own bytes.
                let key = unsafe { c_str_len(key.data(), key.len()) }.null_as_empty();
                return Err(api_error!(
                    kErrorTypeValidation,
                    "Invalid key: '{key}' is only allowed from Lua"
                ));
            }
            // SAFETY: the generated tables name no other type.
            _ => unsafe { abort() },
        }
    }
    Ok(())
}

/// The reverse of [`api_dict_to_keydict`]: the keydict `value` as a plain
/// dictionary, holding only the fields that were set. Lua references are
/// skipped — they mean nothing outside the Lua state.
///
/// # Safety
///
/// `value` must point at the keydict `table` describes, and `table` at the
/// generated `KeySetLink` table for that keydict's type -- the offsets and
/// field types are read from it and applied to `value`, and the walk stops at
/// the row with a null name. `max_size` must be that table's length, and
/// `arena` must point at a live arena the answer is allocated in.
pub(crate) unsafe fn api_keydict_to_dict(
    value: *mut c_void,
    table: *const KeySetLink,
    max_size: size_t,
    arena: *mut Arena,
) -> ApiDict {
    let mut rv = arena_dict(arena, max_size);
    // SAFETY: as `api_dict_to_keydict`; `max_size` is the table's length.
    for field in unsafe { keyset_fields(table) } {
        // SAFETY: the row's offset names a field of `value`, and its type
        // says what lives there. A field the caller never named is `None`
        // and is not a key of the answer at all. A Lua reference *is* still
        // counted as a key, with a nil value, because it means nothing
        // outside the Lua state.
        let mem = unsafe { value.cast::<c_char>().add(field.ptr_off) };
        // SAFETY: as above.
        if !unsafe { keyset_field_is_set(mem.cast(), field.type_0) } {
            continue;
        }
        // The field, which the test above says is `Some`; the default is
        // there to spell the arm's type, not because it can be reached.
        macro_rules! field {
            ($ty:ty, $absent:expr) => {
                // SAFETY: as above -- the row's type says what lives at `mem`.
                unsafe { (*mem.cast::<Option<$ty>>()).unwrap_or($absent) }
            };
        }
        let val = match field.type_0.cast_unsigned() {
            kObjectTypeNil => field!(Object, Object::Nil),
            kObjectTypeInteger => Object::integer(field!(Integer, 0)),
            kObjectTypeFloat => Object::float(field!(Float, 0.0)),
            kObjectTypeBoolean => Object::boolean(field!(Boolean, false)),
            kObjectTypeString => Object::string(field!(String_0, String_0::NULL)),
            kObjectTypeArray => Object::array(field!(Array, Array::EMPTY)),
            kObjectTypeDict => Object::dict(field!(ApiDict, ApiDict::EMPTY)),
            kObjectTypeBuffer => Object::buffer(field!(Handle, 0)),
            kObjectTypeWindow => Object::window(field!(Handle, 0)),
            kObjectTypeTabpage => Object::tabpage(field!(Handle, 0)),
            kObjectTypeLuaRef => Object::Nil,
            // SAFETY: the generated tables name no other type.
            _ => unsafe { abort() },
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
