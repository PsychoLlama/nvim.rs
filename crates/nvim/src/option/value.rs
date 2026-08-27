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

use core::ffi::{CStr, c_char};
use core::ptr;

use crate::api::private::helpers::{api_free_string, copy_string, cstr_as_string};
use crate::main::curbuf;
use crate::memory::{strnequal, xmalloc, xstrdup};
use crate::optionstr::is_empty_option;
use crate::os::cshim::snprintf;
use crate::types::{
    Arena, Object, OptIndex, OptVal, OptValData, OptValType, kObjectTypeBoolean,
    kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, object, object_data, size_t,
};
use crate::undo::curbuf_is_changed;

use super::{
    NUMBUFLEN, OptSlot, get_option, is_option_hidden, kOptValTypeBoolean, kOptValTypeNil,
    kOptValTypeNumber, kOptValTypeString, option_default, option_has_type,
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
pub(crate) fn optval_free(value: OptVal) {
    if value.type_0 != kOptValTypeString {
        return;
    }
    // SAFETY: the tag says the payload is the string field.
    let string = unsafe { value.data.string };
    if !is_empty_option(string.data()) {
        // SAFETY: a string value owns its bytes, and it is not the shared
        // empty string.
        unsafe { api_free_string(string) };
    }
}

/// A value that owns its own copy of whatever the original owned.
pub(crate) fn optval_copy(value: OptVal) -> OptVal {
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
pub(crate) fn optval_equal(o1: OptVal, o2: OptVal) -> bool {
    if o1.type_0 != o2.type_0 {
        return false;
    }
    // SAFETY: the tags agree, so both payloads are the field being read.
    match o1.type_0 {
        kOptValTypeNil => true,
        kOptValTypeBoolean => (unsafe { o1.data.boolean }) == unsafe { o2.data.boolean },
        kOptValTypeNumber => (unsafe { o1.data.number }) == unsafe { o2.data.number },
        kOptValTypeString => {
            unsafe { o1.data.string }.len() == unsafe { o2.data.string }.len()
                && (unsafe { o1.data.string }.data() == unsafe { o2.data.string }.data()
                    || unsafe {
                        strnequal(
                            o1.data.string.data(),
                            o2.data.string.data(),
                            o1.data.string.len(),
                        )
                    })
        }
        _ => unreachable!("option value type {}", o1.type_0),
    }
}

/// The type the table declares for an option.
pub(crate) fn option_get_type(opt_idx: OptIndex) -> OptValType {
    get_option(opt_idx).type_0
}

/// Read the option variable `slot` names as a value of `opt_idx`'s type.
///
/// # Safety
///
/// `slot` must be the variable the table names for `opt_idx`, in some scope
/// — what `get_varp`/`get_varp_scope` hand out.
pub(crate) unsafe fn optval_from_varp(opt_idx: OptIndex, slot: OptSlot) -> OptVal {
    // 'modified' has no variable of its own worth reading: `b_changed` alone
    // misses a buffer whose undo state says it is unchanged after all.
    // SAFETY: `curbuf` is a live buffer for as long as the editor is running.
    if slot == OptSlot::Boolean(unsafe { &raw mut (*curbuf.get()).b_changed }) {
        // SAFETY: reading the current buffer's change state.
        return boolean_optval(Some(curbuf_is_changed()));
    }
    let data = match slot {
        OptSlot::None => return NIL_OPTVAL,
        // SAFETY (all three): the slot names this option's variable and its
        // arm is the type that variable holds. A boolean's word is the
        // option's own tri-state, so anything above 1 reads as true.
        OptSlot::Boolean(var) => OptValData {
            boolean: unsafe { *var }.clamp(-1, 1),
        },
        OptSlot::Number(var) => OptValData {
            number: unsafe { *var },
        },
        OptSlot::String(var) => OptValData {
            string: unsafe { cstr_as_string(*var) },
        },
    };
    OptVal {
        type_0: option_get_type(opt_idx),
        data,
    }
}

/// Write `value` into the option variable `slot` names, taking ownership
/// of whatever it holds. With `free_oldval` the value already there is freed
/// first; without it the caller has kept a copy and will free it later.
///
/// # Safety
///
/// `slot` must be the variable the table names for `opt_idx`, in some scope.
pub(crate) unsafe fn set_option_varp(
    opt_idx: OptIndex,
    slot: OptSlot,
    value: OptVal,
    free_oldval: bool,
) {
    debug_assert!(option_has_type(opt_idx, value.type_0));
    if free_oldval {
        // SAFETY: the caller's slot is this option's variable.
        optval_free(unsafe { optval_from_varp(opt_idx, slot) });
    }
    // SAFETY: the slot's arm and the value's tag are the same type — the
    // table asserts it for every row at compile time, and the assertion
    // above ties this value to the same row.
    match (slot, value.type_0) {
        (OptSlot::Boolean(var), kOptValTypeBoolean) => unsafe { *var = value.data.boolean },
        (OptSlot::Number(var), kOptValTypeNumber) => unsafe { *var = value.data.number },
        (OptSlot::String(var), kOptValTypeString) => unsafe { *var = value.data.string.data() },
        _ => unreachable!("an option's slot is not the type its value is"),
    }
}

/// The value as a freshly allocated C string, the way `:set` reports it in a
/// message: a string is quoted, a boolean spelled out. The caller owns it.
pub(crate) fn optval_to_cstr(value: OptVal) -> *mut c_char {
    // SAFETY: each arm reads the payload its own tag selected.
    match value.type_0 {
        kOptValTypeNil => unsafe { xstrdup(c"".as_ptr()) },
        kOptValTypeBoolean => unsafe {
            xstrdup(if value.data.boolean != 0 {
                c"true".as_ptr()
            } else {
                c"false".as_ptr()
            })
        },
        kOptValTypeNumber => {
            let len = NUMBUFLEN as size_t;
            let buf = unsafe { xmalloc(len) }.cast::<c_char>();
            unsafe { snprintf(buf, len, c"%ld".as_ptr(), value.data.number) };
            buf
        }
        kOptValTypeString => {
            // Two quotes and the terminator.
            let len = unsafe { value.data.string }.len().wrapping_add(3);
            let buf = unsafe { xmalloc(len) }.cast::<c_char>();
            unsafe { snprintf(buf, len, c"\"%s\"".as_ptr(), value.data.string.data()) };
            buf
        }
        _ => unreachable!("option value type {}", value.type_0),
    }
}

/// The value as the API reports it. A tri-state boolean that is neither true
/// nor false comes back as nil.
pub(crate) fn optval_as_object(value: OptVal) -> Object {
    let nil = object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    };
    // SAFETY: each arm reads the payload its own tag selected.
    match value.type_0 {
        kOptValTypeNil => nil,
        kOptValTypeBoolean => match unsafe { optval_boolean(value.data) } {
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
                integer: unsafe { value.data.number },
            },
        },
        kOptValTypeString => object {
            type_0: kObjectTypeString,
            data: object_data {
                string: unsafe { value.data.string },
            },
        },
        _ => unreachable!("option value type {}", value.type_0),
    }
}

/// The API value as an option value, or `None` for an `Object` no option can
/// hold. The result borrows the object's string rather than copying it.
pub(crate) fn object_as_optval(o: Object) -> Option<OptVal> {
    // SAFETY: each arm reads the payload the object's own tag selected.
    Some(match o.type_0 {
        kObjectTypeNil => NIL_OPTVAL,
        kObjectTypeBoolean => boolean_optval(Some(unsafe { o.data.boolean })),
        kObjectTypeInteger => OptVal {
            type_0: kOptValTypeNumber,
            data: OptValData {
                number: unsafe { o.data.integer },
            },
        },
        kObjectTypeString => OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: unsafe { o.data.string },
            },
        },
        _ => return None,
    })
}

/// Whether the option still holds the default the table gives it. A hidden
/// option counts as default whatever its variable says.
///
/// # Safety
///
/// `slot` must be the variable the table names for `opt_idx`, in some scope.
pub(crate) unsafe fn optval_is_default(opt_idx: OptIndex, slot: OptSlot) -> bool {
    if is_option_hidden(opt_idx) {
        return true;
    }
    // SAFETY: the caller's slot is this option's variable.
    let current = unsafe { optval_from_varp(opt_idx, slot) };
    optval_equal(current, option_default(opt_idx))
}
