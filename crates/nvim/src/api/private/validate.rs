//! The four "that argument is wrong" messages the API family shares, and the
//! one check that needs more than a line to make.
//!
//! Each message comes in two spellings, chosen by whether the thing being
//! named is one word or several: `Invalid 'buffer id'` reads right for a
//! single name and `Invalid buffer id: 3` for a phrase, so the quoting
//! follows the space. That rule is upstream's, and every caller depends on
//! the exact text -- the functional suite asserts these strings.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{api_set_error, api_typename};

use crate::os::cshim::snprintf;
use crate::types::{
    Array, Error, IOSIZE, String_0, int64_t, kErrorTypeValidation, kObjectTypeString, size_t,
};
use core::ffi::{CStr, c_char};

/// Whether `name` is a phrase rather than a single name, which is what
/// decides between the quoted and unquoted spelling of every message here.
///
/// # Safety
/// `name` must be a C string.
unsafe fn is_phrase(name: *const c_char) -> bool {
    // SAFETY: the caller's promise.
    unsafe { CStr::from_ptr(name) }.to_bytes().contains(&b' ')
}

/// "Invalid `name`", optionally saying what the offending value was: `val_s`
/// when it is a non-empty string, `val_n` when `val_s` is null, and nothing
/// at all when `val_s` is the empty string.
///
/// # Safety
/// `err` must be the caller's error slot, `name` a C string, and `val_s` null
/// or a C string.
pub unsafe fn api_err_invalid(
    err: *mut Error,
    name: *const c_char,
    val_s: *const c_char,
    val_n: int64_t,
    quote_val: bool,
) {
    // SAFETY: the caller's promise about `name` and `val_s`.
    let phrase = unsafe { is_phrase(name) };
    // SAFETY: as above -- `val_s` is null or the caller's C string.
    let val = (!val_s.is_null()).then(|| unsafe { CStr::from_ptr(val_s) });
    match val {
        Some(val) if val.is_empty() => {
            let fmt = if phrase {
                c"Invalid %s"
            } else {
                c"Invalid '%s'"
            };
            // SAFETY: the caller's promise about `err`; the format takes the
            // one C string it is given.
            unsafe { api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), name) };
        }
        Some(val) => {
            let fmt = match (phrase, quote_val) {
                (true, true) => c"Invalid %s: '%s'",
                (true, false) => c"Invalid %s: %s",
                (false, true) => c"Invalid '%s': '%s'",
                (false, false) => c"Invalid '%s': %s",
            };
            // SAFETY: as above, with two C strings.
            unsafe { api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), name, val.as_ptr()) };
        }
        None => {
            let fmt = if phrase {
                c"Invalid %s: %ld"
            } else {
                c"Invalid '%s': %ld"
            };
            // SAFETY: as above, with a C string and an `int64_t`.
            unsafe { api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), name, val_n) };
        }
    }
}

/// "Invalid `name`: expected `expected`", naming what arrived when `actual`
/// says.
///
/// # Safety
/// `err` must be the caller's error slot, `name` and `expected` C strings,
/// and `actual` null or a C string.
pub unsafe fn api_err_exp(
    err: *mut Error,
    name: *const c_char,
    expected: *const c_char,
    actual: *const c_char,
) {
    // SAFETY: the caller's promise about `name`.
    let phrase = unsafe { is_phrase(name) };
    if actual.is_null() {
        let fmt = if phrase {
            c"Invalid %s: expected %s"
        } else {
            c"Invalid '%s': expected %s"
        };
        // SAFETY: the caller's promise about `err`; two C strings.
        unsafe { api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), name, expected) };
        return;
    }
    let fmt = if phrase {
        c"Invalid %s: expected %s, got %s"
    } else {
        c"Invalid '%s': expected %s, got %s"
    };
    // SAFETY: as above, with three C strings.
    unsafe {
        api_set_error(
            err,
            kErrorTypeValidation,
            fmt.as_ptr(),
            name,
            expected,
            actual,
        )
    };
}

/// "Required: `name`", for an option the caller left out.
///
/// # Safety
/// `err` must be the caller's error slot and `name` a C string.
pub unsafe fn api_err_required(err: *mut Error, name: *const c_char) {
    // SAFETY: the caller's promise about `name`.
    let fmt = if unsafe { is_phrase(name) } {
        c"Required: %s"
    } else {
        c"Required: '%s'"
    };
    // SAFETY: the caller's promise about `err`; the format takes one C string.
    unsafe { api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), name) };
}

/// "Conflict: `name` not allowed with `name2`", for two options that exclude
/// each other. `name` is always quoted; only `name2`'s spelling follows the
/// space rule, as upstream's does.
///
/// # Safety
/// `err` must be the caller's error slot and both names C strings.
pub unsafe fn api_err_conflict(err: *mut Error, name: *const c_char, name2: *const c_char) {
    // SAFETY: the caller's promise about `name2`.
    let fmt = if unsafe { is_phrase(name2) } {
        c"Conflict: '%s' not allowed with %s"
    } else {
        c"Conflict: '%s' not allowed with '%s'"
    };
    // SAFETY: the caller's promise about `err`; the format takes two C
    // strings.
    unsafe { api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), name, name2) };
}

/// Whether every element of `arr` is a String, and -- when `disallow_nl` --
/// one without a newline in it. `name` names the array in whatever message
/// this leaves in `err`.
///
/// # Safety
/// `arr` must point at its own elements, `name` must be a C string, and `err`
/// must be the caller's error slot.
pub unsafe fn check_string_array(
    arr: Array,
    name: *mut c_char,
    disallow_nl: bool,
    err: *mut Error,
) -> bool {
    // The item name is built once and then handed to whichever message
    // fires. Upstream builds it in the shared scratch buffer, which the
    // message machinery writes again.
    let mut item = [0 as c_char; IOSIZE as usize];
    // SAFETY: `name` is the caller's C string and `item` is `IOSIZE` bytes.
    let item_name = unsafe {
        let buf = item.as_mut_ptr();
        snprintf(buf, IOSIZE as size_t, c"'%s' item".as_ptr(), name);
        buf
    };
    // SAFETY: `arr` is the caller's array, per this function's contract; an
    // empty one may carry a null `items`, which no slice may.
    let items = match arr.size {
        0 => &[][..],
        size => unsafe { core::slice::from_raw_parts(arr.items, size) },
    };
    for item in items {
        if item.type_0 != kObjectTypeString {
            // SAFETY: the caller's promise about `err`; the type names are
            // static and `item_name` is the scratch buffer above.
            unsafe {
                let got = api_typename(item.type_0);
                api_err_exp(err, item_name, api_typename(kObjectTypeString), got);
            }
            return false;
        }
        // SAFETY: the tag says the payload is the string, and the string is
        // the caller's, live for its own length.
        let l: String_0 = unsafe { item.data.string };
        if disallow_nl && unsafe { l.as_bytes() }.contains(&b'\n') {
            // SAFETY: as above; the format takes the one C string.
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"'%s' item contains newlines".as_ptr(),
                    name,
                );
            }
            return false;
        }
    }
    true
}
