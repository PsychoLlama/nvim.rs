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
use crate::types::{Array, Error, int64_t, kErrorTypeValidation, kObjectTypeString};
use core::ffi::CStr;

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
        let Some(l) = item.as_string() else {
            let want = msg_cstr(api_typename(kObjectTypeString));
            let got = msg_cstr(api_typename(item.kind()));
            return Err(api_error!(
                kErrorTypeValidation,
                "Invalid '{name}' item: expected {want}, got {got}"
            ));
        };
        // SAFETY: the string is the caller's, live for its own length.
        if disallow_nl && unsafe { l.as_bytes() }.contains(&b'\n') {
            return Err(api_error!(
                kErrorTypeValidation,
                "'{name}' item contains newlines"
            ));
        }
    }
    Ok(())
}
