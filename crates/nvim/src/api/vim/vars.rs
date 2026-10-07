//! Global and `v:` variables, and the current line.
//!
//! The `nvim_{get,set,del}_var` trio over the global dictionary and the
//! `nvim_{get,set}_vvar` pair over `v:`, plus the three current-line
//! accessors, which are the same shape: one lookup and one conversion
//! through the api's Object bridge.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::message_fmt::msg_cstr;
use crate::winlayer::{Buf, Win};

/// The current buffer's handle and the cursor's line, as the deprecated
/// `buffer_*_line` trio take them -- a zero-based index.
fn cursor_line() -> (BufferHandle, Integer) {
    let (buf, lnum) = (Buf::current().handle, Win::current().w_cursor.lnum);
    (buf, Integer::from(lnum - 1))
}

/// The line the cursor is on.
pub fn nvim_get_current_line() -> Result<String_0, Error> {
    let (buf, lnum) = cursor_line();
    buffer_get_line(buf, lnum)
}

/// Replace the line the cursor is on with `line`.
pub fn nvim_set_current_line(line: String_0) -> Result<(), Error> {
    let (buf, lnum) = cursor_line();
    buffer_set_line(buf, lnum, line)
}

/// Delete the line the cursor is on.
pub fn nvim_del_current_line() -> Result<(), Error> {
    let (buf, lnum) = cursor_line();
    buffer_del_line(buf, lnum)
}

/// The global variable `name`, autoloading the script that defines it if it
/// is not there yet.
pub fn nvim_get_var(name: String_0) -> Result<Object, Error> {
    let mut value = globvar_object(&name);
    if value.is_none() {
        // SAFETY: the API string names its own `len` bytes.
        let loaded = unsafe { script_autoload(name.data(), name.len(), false) };
        if !loaded || aborting() {
            return Object::Nil.reported(key_not_found(&name));
        }
        value = globvar_object(&name);
    }
    match value {
        Some(value) => value.reported(Error::none()),
        None => Object::Nil.reported(key_not_found(&name)),
    }
}

/// `g:name`'s value, converted -- `None` when there is no such variable.
fn globvar_object(name: &String_0) -> Option<Object> {
    // The borrow ends with the conversion, which runs no user code.
    globvar_dict()
        .find(name.as_bytes())
        .map(|item| Object::from(&item.di_tv))
}

/// "Key not found: `name`".
fn key_not_found(name: &String_0) -> Error {
    let name = msg_cstr(name.as_cstr());
    api_error!(kErrorTypeValidation, "Key not found: {name}")
}

/// Set the global variable `name` to `value`.
pub fn nvim_set_var(name: String_0, value: Object) -> Result<(), Error> {
    let dict = globvar_dict();
    // SAFETY: the handle keeps the dictionary live through the call; the
    // value is copied into it.
    unsafe { dict_set_var(dict.as_ptr(), &name, value, false, false) }.map(|_| ())
}

/// Remove the global variable `name`.
pub fn nvim_del_var(name: String_0) -> Result<(), Error> {
    let dict = globvar_dict();
    // SAFETY: as [`nvim_set_var`]; `del` says to remove rather than assign.
    unsafe { dict_set_var(dict.as_ptr(), &name, Object::Nil, true, false) }.map(|_| ())
}

/// The `v:` variable `name`.
pub fn nvim_get_vvar(name: String_0) -> Result<Object, Error> {
    let dict = vimvar_dict();
    // SAFETY: the handle keeps `v:` live through the call.
    unsafe { dict_get_value(dict.as_ptr(), &name) }
}

/// Set the `v:` variable `name` to `value`.
pub fn nvim_set_vvar(name: String_0, value: Object) -> Result<(), Error> {
    let dict = vimvar_dict();
    // SAFETY: as [`nvim_set_var`], over `v:` rather than the globals.
    unsafe { dict_set_var(dict.as_ptr(), &name, value, false, false) }.map(|_| ())
}
