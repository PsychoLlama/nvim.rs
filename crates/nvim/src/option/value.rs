//! The `OptVal` tagged union: freeing, copying, comparing, and converting to
//! and from the option variable, an API `Object` and a C string.
//!
//! The union is `repr(C)` and stays that way: the generated table stores an
//! option's default as one, and the API hands one across the RPC boundary.
//! Reading the payload is only sound after the tag has been tested, so every
//! read here sits behind a `match` on `type_0` — that match *is* the
//! soundness argument, and the `unreachable!()` arms are the tag values the
//! union has no payload for.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::api::private::helpers::{api_free_string, copy_string, cstr_as_string};
use crate::main::{curbuf, empty_string_option};
use crate::memory::{strnequal, xmalloc, xstrdup};
use crate::options::options;
use crate::os::cshim::snprintf;
use crate::types::{
    Arena, Object, OptIndex, OptInt, OptVal, OptValData, OptValType, kObjectTypeBoolean,
    kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, object, object_data, size_t,
};
use crate::undo::curbuf_is_changed;

use super::{
    NUMBUFLEN, is_option_hidden, kOptValTypeBoolean, kOptValTypeNil, kOptValTypeNumber,
    kOptValTypeString, option_has_type,
};

/// An `OptVal` holding nothing: an unknown option, or one the caller only
/// wants the *absence* of.  The union payload is a don't-care and the tag is
/// what every reader tests.
pub(crate) const NIL_OPTVAL: OptVal = OptVal {
    type_0: kOptValTypeNil,
    data: OptValData { boolean: 0 },
};

/// The `OptVal` for a boolean option's value.
///
/// `None` is upstream's `kNone`: a global-local option with no value of its
/// own in this scope.  It is not "false", and `optval_as_object` reports it
/// as nil rather than as `false`.
pub(crate) const fn boolean_optval(value: Option<bool>) -> OptVal {
    OptVal {
        type_0: kOptValTypeBoolean,
        data: OptValData {
            boolean: match value {
                Some(true) => 1,
                Some(false) => 0,
                None => -1,
            },
        },
    }
}

/// A boolean option's value, `None` for the unset global-local marker.
///
/// # Safety
/// The `OptVal` this payload came from must be tagged `kOptValTypeBoolean`.
pub(crate) unsafe fn optval_boolean(data: OptValData) -> Option<bool> {
    // SAFETY: the caller's promise about the tag.
    match unsafe { data.boolean } {
        0 => Some(false),
        1.. => Some(true),
        _ => None,
    }
}

/// What `:set` and `nvim_get_option_info` call each value type.
pub(crate) fn optval_type_name(type_0: OptValType) -> &'static CStr {
    match type_0 {
        kOptValTypeNil => c"nil",
        kOptValTypeBoolean => c"boolean",
        kOptValTypeNumber => c"number",
        kOptValTypeString => c"string",
        _ => unreachable!("option value type {type_0}"),
    }
}

/// Release what a value owns. Only a string owns anything, and the shared
/// empty string every unset string option points at is not ours to free.
pub fn optval_free(value: OptVal) {
    if value.type_0 != kOptValTypeString {
        return;
    }
    // SAFETY: the tag says the payload is the string field.
    let string = unsafe { value.data.string };
    if string.data() != empty_string_option.ptr().cast::<c_char>() {
        // SAFETY: a string value owns its bytes, and it is not the shared
        // empty string.
        unsafe { api_free_string(string) };
    }
}

/// A value that owns its own copy of whatever the original owned.
pub fn optval_copy(value: OptVal) -> OptVal {
    match value.type_0 {
        kOptValTypeNil | kOptValTypeBoolean | kOptValTypeNumber => value,
        kOptValTypeString => OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                // SAFETY: the tag says the payload is the string field.
                string: unsafe { copy_string(value.data.string, ptr::null_mut::<Arena>()) },
            },
        },
        _ => unreachable!("option value type {}", value.type_0),
    }
}

/// Whether two values are the same. Two strings of the same length are
/// compared byte-wise, so an option holding a NUL still compares correctly.
pub fn optval_equal(o1: OptVal, o2: OptVal) -> bool {
    if o1.type_0 != o2.type_0 {
        return false;
    }
    // SAFETY: the tags agree, so both payloads are the field being read.
    unsafe {
        match o1.type_0 {
            kOptValTypeNil => true,
            kOptValTypeBoolean => o1.data.boolean == o2.data.boolean,
            kOptValTypeNumber => o1.data.number == o2.data.number,
            kOptValTypeString => {
                o1.data.string.len() == o2.data.string.len()
                    && (o1.data.string.data() == o2.data.string.data()
                        || strnequal(
                            o1.data.string.data(),
                            o2.data.string.data(),
                            o1.data.string.len(),
                        ))
            }
            _ => unreachable!("option value type {}", o1.type_0),
        }
    }
}

/// The type the table declares for an option.
pub(crate) fn option_get_type(opt_idx: OptIndex) -> OptValType {
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { (*options.ptr())[opt_idx as usize].type_0 }
}

/// Read the option variable `varp` points at as a value of `opt_idx`'s type.
///
/// # Safety
///
/// `varp` must be the variable the table names for `opt_idx`, in some scope
/// — what `get_varp`/`get_varp_scope` hand out.
pub unsafe fn optval_from_varp(opt_idx: OptIndex, varp: *mut c_void) -> OptVal {
    // 'modified' has no variable of its own worth reading: `b_changed` alone
    // misses a buffer whose undo state says it is unchanged after all.
    // SAFETY: `curbuf` is a live buffer for as long as the editor is running.
    if varp.cast::<c_int>() == unsafe { &raw mut (*curbuf.get()).b_changed } {
        // SAFETY: reading the current buffer's change state.
        return boolean_optval(Some(unsafe { curbuf_is_changed() }));
    }
    let type_0 = option_get_type(opt_idx);
    let data = match type_0 {
        kOptValTypeNil => NIL_OPTVAL.data,
        // SAFETY (all three): the caller's `varp` is the variable for this
        // option, and the table's type says which of the three it is.
        kOptValTypeBoolean => OptValData {
            boolean: match unsafe { *varp.cast::<c_int>() } {
                0 => 0,
                1.. => 1,
                _ => -1,
            },
        },
        kOptValTypeNumber => OptValData {
            number: unsafe { *varp.cast::<OptInt>() },
        },
        kOptValTypeString => OptValData {
            string: unsafe { cstr_as_string(*varp.cast::<*mut c_char>()) },
        },
        _ => unreachable!("option value type {type_0}"),
    };
    OptVal { type_0, data }
}

/// Write `value` into the option variable `varp` points at, taking ownership
/// of whatever it holds. With `free_oldval` the value already there is freed
/// first; without it the caller has kept a copy and will free it later.
///
/// # Safety
///
/// `varp` must be the variable the table names for `opt_idx`, in some scope.
pub(crate) unsafe fn set_option_varp(
    opt_idx: OptIndex,
    varp: *mut c_void,
    value: OptVal,
    free_oldval: bool,
) {
    debug_assert!(option_has_type(opt_idx, value.type_0));
    if free_oldval {
        // SAFETY: the caller's `varp` is this option's variable.
        optval_free(unsafe { optval_from_varp(opt_idx, varp) });
    }
    // SAFETY: the assertion above ties the tag to the variable's type, so
    // both the union read and the write through `varp` are of that type.
    unsafe {
        match value.type_0 {
            kOptValTypeBoolean => *varp.cast::<c_int>() = value.data.boolean,
            kOptValTypeNumber => *varp.cast::<OptInt>() = value.data.number,
            kOptValTypeString => *varp.cast::<*mut c_char>() = value.data.string.data(),
            _ => unreachable!("a nil value has no variable to write it to"),
        }
    }
}

/// The value as a freshly allocated C string, the way `:set` reports it in a
/// message: a string is quoted, a boolean spelled out. The caller owns it.
pub(crate) fn optval_to_cstr(value: OptVal) -> *mut c_char {
    // SAFETY: each arm reads the payload its own tag selected.
    unsafe {
        match value.type_0 {
            kOptValTypeNil => xstrdup(c"".as_ptr()),
            kOptValTypeBoolean => xstrdup(if value.data.boolean != 0 {
                c"true".as_ptr()
            } else {
                c"false".as_ptr()
            }),
            kOptValTypeNumber => {
                let len = NUMBUFLEN as size_t;
                let buf = xmalloc(len).cast::<c_char>();
                snprintf(buf, len, c"%ld".as_ptr(), value.data.number);
                buf
            }
            kOptValTypeString => {
                // Two quotes and the terminator.
                let len = value.data.string.len().wrapping_add(3);
                let buf = xmalloc(len).cast::<c_char>();
                snprintf(buf, len, c"\"%s\"".as_ptr(), value.data.string.data());
                buf
            }
            _ => unreachable!("option value type {}", value.type_0),
        }
    }
}

/// The value as the API reports it. A tri-state boolean that is neither true
/// nor false comes back as nil.
pub fn optval_as_object(value: OptVal) -> Object {
    let nil = object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    };
    // SAFETY: each arm reads the payload its own tag selected.
    unsafe {
        match value.type_0 {
            kOptValTypeNil => nil,
            kOptValTypeBoolean => match optval_boolean(value.data) {
                Some(boolean) => object {
                    type_0: kObjectTypeBoolean,
                    data: object_data { boolean },
                },
                // A global-local option with no local value has no API
                // spelling but nil.
                None => nil,
            },
            kOptValTypeNumber => object {
                type_0: kObjectTypeInteger,
                data: object_data {
                    integer: value.data.number,
                },
            },
            kOptValTypeString => object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: value.data.string,
                },
            },
            _ => unreachable!("option value type {}", value.type_0),
        }
    }
}

/// The API value as an option value, or `None` for an `Object` no option can
/// hold. The result borrows the object's string rather than copying it.
pub fn object_as_optval(o: Object) -> Option<OptVal> {
    // SAFETY: each arm reads the payload the object's own tag selected.
    unsafe {
        Some(match o.type_0 {
            kObjectTypeNil => NIL_OPTVAL,
            kObjectTypeBoolean => boolean_optval(Some(o.data.boolean)),
            kObjectTypeInteger => OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData {
                    number: o.data.integer,
                },
            },
            kObjectTypeString => OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: o.data.string,
                },
            },
            _ => return None,
        })
    }
}

/// Whether the option still holds the default the table gives it. A hidden
/// option counts as default whatever its variable says.
///
/// # Safety
///
/// `varp` must be the variable the table names for `opt_idx`, in some scope.
pub(crate) unsafe fn optval_is_default(opt_idx: OptIndex, varp: *mut c_void) -> bool {
    if is_option_hidden(opt_idx) {
        return true;
    }
    // SAFETY: the caller's `varp` is this option's variable.
    let current = unsafe { optval_from_varp(opt_idx, varp) };
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    let default = unsafe { (*options.ptr())[opt_idx as usize].def_val };
    optval_equal(current, default)
}
