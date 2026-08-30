//! The "that argument is wrong" messages the whole API family shares.
//!
//! Every function here *answers* an [`Error`] rather than writing through an
//! out-parameter, so a caller writes `return Err(err_required(c"buffer"))` or
//! hands the value straight to `?`.
//!
//! Each message comes in two spellings, chosen by whether the thing being
//! named is one word or several: `Invalid 'buffer id'` reads right for a
//! single name and `Invalid buffer id: 3` for a phrase, so the quoting
//! follows the space. That rule is upstream's, and every caller depends on
//! the exact text -- the functional suite asserts these strings.
//!
//! Before this was the family's home, ten modules under `api/` each carried
//! their own `err_exception`/`err_validation`/`err_bad_value`/
//! `err_bad_number`/`err_expected`/`err_out_of_range` wrapping the same four
//! `api_set_error` calls behind the same four doc comments. They are all this
//! file now.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::api_typename;
use crate::api_error;
use crate::message_fmt::msg_cstr;
use crate::types::{
    Array, Error, ErrorType, String_0, int64_t, kErrorTypeException, kErrorTypeValidation,
    kObjectTypeString,
};
use core::ffi::{CStr, c_char};

/// Whether `name` is a phrase rather than a single name, which is what
/// decides between the quoted and unquoted spelling of every message here.
fn is_phrase(name: &CStr) -> bool {
    name.to_bytes().contains(&b' ')
}

/// What an [`err_invalid`] message names as the offending value.
pub(crate) enum Bad<'a> {
    /// Nothing: "Invalid 'name'". An empty [`Quoted`](Self::Quoted) or
    /// [`Bare`](Self::Bare) reads the same way, as upstream's did.
    Unsaid,
    /// A string, quoted: "Invalid 'name': 'value'".
    Quoted(&'a CStr),
    /// A string, unquoted -- a reason rather than a value: "Invalid 'name':
    /// out of range".
    Bare(&'a CStr),
    /// A number: "Invalid 'name': 3".
    Number(int64_t),
}

/// "Invalid `name`", saying what the offending value was when `bad` names
/// one.
pub(crate) fn err_invalid(name: &CStr, bad: Bad<'_>) -> Error {
    let phrase = is_phrase(name);
    let bad = match bad {
        Bad::Quoted(v) | Bad::Bare(v) if v.is_empty() => Bad::Unsaid,
        bad => bad,
    };
    let name = msg_cstr(name);
    match bad {
        Bad::Unsaid if phrase => api_error!(kErrorTypeValidation, "Invalid {name}"),
        Bad::Unsaid => api_error!(kErrorTypeValidation, "Invalid '{name}'"),
        Bad::Quoted(v) => {
            let v = msg_cstr(v);
            if phrase {
                api_error!(kErrorTypeValidation, "Invalid {name}: '{v}'")
            } else {
                api_error!(kErrorTypeValidation, "Invalid '{name}': '{v}'")
            }
        }
        Bad::Bare(v) => {
            let v = msg_cstr(v);
            if phrase {
                api_error!(kErrorTypeValidation, "Invalid {name}: {v}")
            } else {
                api_error!(kErrorTypeValidation, "Invalid '{name}': {v}")
            }
        }
        Bad::Number(n) if phrase => api_error!(kErrorTypeValidation, "Invalid {name}: {n}"),
        Bad::Number(n) => api_error!(kErrorTypeValidation, "Invalid '{name}': {n}"),
    }
}

/// "Invalid `name`: '`val`'", the commonest of them: a keyset string the
/// caller spelled wrong.
pub(crate) fn err_bad_value(name: &CStr, val: &CStr) -> Error {
    err_invalid(name, Bad::Quoted(val))
}

/// [`err_bad_value`] for a value still held as a raw pointer.
///
/// A null `val` keeps upstream's answer for one: the number 0, which is what
/// `api_err_invalid`'s `val_n` defaulted to at every such call site.
///
/// # Safety
/// `val` must be null or a C string that outlives the call.
pub(crate) unsafe fn err_bad_value_ptr(name: &CStr, val: *const c_char) -> Error {
    // SAFETY: the caller's promise.
    unsafe { err_invalid_ptr(name.as_ptr(), val, 0, true) }
}

/// "Invalid `name`: `n`", for a number outside what the key accepts.
pub(crate) fn err_bad_number(name: &CStr, n: int64_t) -> Error {
    err_invalid(name, Bad::Number(n))
}

/// "Invalid `name`: out of range".
pub(crate) fn err_out_of_range(name: &CStr) -> Error {
    err_invalid(name, Bad::Bare(c"out of range"))
}

/// "Invalid `name`: expected `expected`", naming what arrived when `actual`
/// says.
pub(crate) fn err_expected(name: &CStr, expected: &CStr, actual: Option<&CStr>) -> Error {
    let phrase = is_phrase(name);
    let (name, expected) = (msg_cstr(name), msg_cstr(expected));
    match actual {
        None if phrase => api_error!(kErrorTypeValidation, "Invalid {name}: expected {expected}"),
        None => api_error!(
            kErrorTypeValidation,
            "Invalid '{name}': expected {expected}"
        ),
        Some(actual) => {
            let actual = msg_cstr(actual);
            if phrase {
                api_error!(
                    kErrorTypeValidation,
                    "Invalid {name}: expected {expected}, got {actual}"
                )
            } else {
                api_error!(
                    kErrorTypeValidation,
                    "Invalid '{name}': expected {expected}, got {actual}"
                )
            }
        }
    }
}

/// "Required: `name`", for an option the caller left out.
pub(crate) fn err_required(name: &CStr) -> Error {
    let phrase = is_phrase(name);
    let name = msg_cstr(name);
    if phrase {
        api_error!(kErrorTypeValidation, "Required: {name}")
    } else {
        api_error!(kErrorTypeValidation, "Required: '{name}'")
    }
}

/// "Conflict: `name` not allowed with `name2`", for two options that exclude
/// each other. `name` is always quoted; only `name2`'s spelling follows the
/// space rule, as upstream's does.
pub(crate) fn err_conflict(name: &CStr, name2: &CStr) -> Error {
    let phrase = is_phrase(name2);
    let (name, name2) = (msg_cstr(name), msg_cstr(name2));
    if phrase {
        api_error!(
            kErrorTypeValidation,
            "Conflict: '{name}' not allowed with {name2}"
        )
    } else {
        api_error!(
            kErrorTypeValidation,
            "Conflict: '{name}' not allowed with '{name2}'"
        )
    }
}

// ---------------------------------------------------------------------------
// The pointer-shaped edge
//
// `api_err_invalid`/`api_err_exp`/`api_err_required`/`api_err_conflict` took
// every name and value as a `*const c_char`, and a hundred call sites still
// hold theirs that way. These are those four signatures minus the
// out-parameter; they narrow to the `&CStr` forms above as phase 24's string
// work reaches each caller.

/// [`err_invalid`] over raw pointers: a null `val` selects `val_n`, and
/// `quote_val` chooses between `Bad::Quoted` and `Bad::Bare`.
///
/// # Safety
/// `name` is a C string and `val` is null or a C string.
pub(crate) unsafe fn err_invalid_ptr(
    name: *const c_char,
    val: *const c_char,
    val_n: int64_t,
    quote_val: bool,
) -> Error {
    // SAFETY: the caller's promise.
    let (name, val) = unsafe { (CStr::from_ptr(name), crate::cstr::at_opt(val)) };
    match val {
        Some(val) if quote_val => err_invalid(name, Bad::Quoted(val)),
        Some(val) => err_invalid(name, Bad::Bare(val)),
        None => err_invalid(name, Bad::Number(val_n)),
    }
}

/// [`err_expected`] for a caller whose *name* is still a pointer -- a keyset
/// field's `str`, or an option key read out of a table.
///
/// # Safety
/// `name` is a C string.
pub(crate) unsafe fn err_expected_ptr(
    name: *const c_char,
    expected: &CStr,
    actual: Option<&CStr>,
) -> Error {
    // SAFETY: the caller's promise.
    err_expected(unsafe { CStr::from_ptr(name) }, expected, actual)
}

/// [`err_required`] over a raw pointer.
///
/// # Safety
/// `name` is a C string.
pub(crate) unsafe fn err_required_ptr(name: *const c_char) -> Error {
    // SAFETY: the caller's promise.
    err_required(unsafe { CStr::from_ptr(name) })
}

/// [`err_conflict`] over raw pointers.
///
/// # Safety
/// Both names are C strings.
pub(crate) unsafe fn err_conflict_ptr(name: *const c_char, name2: *const c_char) -> Error {
    // SAFETY: the caller's promise.
    let (name, name2) = unsafe { (CStr::from_ptr(name), CStr::from_ptr(name2)) };
    err_conflict(name, name2)
}

/// A failure of kind `kind` whose whole message is the string at `msg`:
/// upstream's `api_set_error(err, kind, "%s", msg)`, and its bare
/// `api_set_error(err, kind, msg)` too.
///
/// The bare form passed the message *as the format*, so a `%` in it printed
/// as a conversion. Nothing in the tree ever put one there, and this spelling
/// cannot.
///
/// # Safety
/// `msg` is a C string.
pub(crate) unsafe fn err_msg_ptr(kind: ErrorType, msg: *const c_char) -> Error {
    // SAFETY: the caller's promise.
    Error::from_message(kind, unsafe { CStr::from_ptr(msg) })
}

/// A validation failure whose whole message is `why`.
pub(crate) fn err_validation(why: &CStr) -> Error {
    Error::from_message(kErrorTypeValidation, why)
}

/// An exception whose whole message is `why`.
pub(crate) fn err_exception(why: &CStr) -> Error {
    Error::from_message(kErrorTypeException, why)
}

/// Whether every element of `arr` is a String, and -- when `disallow_nl` --
/// one without a newline in it. `name` names the array in whatever message
/// this answers with.
///
/// The name upstream builds -- `'<name>' item` -- always holds a space, so
/// the message always takes [`err_expected`]'s phrase spelling; it is written
/// out here rather than assembled in the shared scratch buffer first.
///
/// # Safety
/// `arr` must point at its own elements.
pub(crate) unsafe fn check_string_array(
    arr: Array,
    name: &CStr,
    disallow_nl: bool,
) -> Result<(), Error> {
    // SAFETY: `arr` is the caller's array, per this function's contract; an
    // empty one may carry a null `items`, which no slice may.
    let items = match arr.size {
        0 => &[][..],
        size => unsafe { core::slice::from_raw_parts(arr.items, size) },
    };
    let name = msg_cstr(name);
    for item in items {
        if item.type_0 != kObjectTypeString {
            let want = msg_cstr(api_typename(kObjectTypeString));
            let got = msg_cstr(api_typename(item.type_0));
            return Err(api_error!(
                kErrorTypeValidation,
                "Invalid '{name}' item: expected {want}, got {got}"
            ));
        }
        // SAFETY: the tag says the payload is the string, and the string is
        // the caller's, live for its own length.
        let l: String_0 = unsafe { item.data.string };
        if disallow_nl && unsafe { l.as_bytes() }.contains(&b'\n') {
            return Err(api_error!(
                kErrorTypeValidation,
                "'{name}' item contains newlines"
            ));
        }
    }
    Ok(())
}
