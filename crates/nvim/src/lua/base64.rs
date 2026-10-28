#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! `vim.base64`: the Lua bindings over the codec in [`crate::base64`].
//!
//! Both bindings share one body that does all the stack work; the codec
//! above it lives elsewhere.

use core::ffi::c_int;

use crate::base64::{decode, encode};
use crate::lua::stack::{At, Kind, Raise, Stack};
use crate::lua_functions;

lua_functions! {
    /// `vim.base64`.
    enum Base64 in BASE64_FUNCTIONS {
        0 => c"encode": base64_encode,
        1 => c"decode": base64_decode,
    }
}

/// Body of both bindings: take the single string argument, run it through
/// `convert`, and push the result. `None` from `convert` is a Lua error.
fn transcode(stack: &mut Stack, convert: fn(&[u8]) -> Option<Vec<u8>>) -> Result<c_int, Raise> {
    if stack.top().index() < 1 {
        return Err(Raise::message("Expected 1 argument"));
    }
    if stack.kind(At::arg(1)) != Kind::String {
        return Err(Raise::argument(1, "expected string"));
    }
    let Some(out) = convert(stack.string(At::arg(1)).unwrap_or_default()) else {
        return Err(Raise::message("Invalid input"));
    };
    stack.push_bytes(&out);
    Ok(1)
}

/// `vim.base64.encode(str)`.
fn base64_encode(stack: &mut Stack) -> Result<c_int, Raise> {
    transcode(stack, |src| Some(encode(src).into_bytes()))
}

/// `vim.base64.decode(str)`.
fn base64_decode(stack: &mut Stack) -> Result<c_int, Raise> {
    transcode(stack, decode)
}

/// Build the `vim.base64` table and leave it on the stack.
pub(crate) fn push_base64(stack: &mut Stack) {
    stack.new_table(0, 0);
    stack.register_functions(BASE64_FUNCTIONS);
}
