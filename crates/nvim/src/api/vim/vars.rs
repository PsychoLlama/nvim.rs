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
use crate::message_fmt::c_str;
use crate::winlayer::{Buf, Win};

/// The current buffer's handle and the cursor's line, as the deprecated
/// `buffer_*_line` trio take them -- a zero-based index.
fn cursor_line() -> (BufferHandle, Integer) {
    let (buf, lnum) = (Buf::current().handle, Win::current().w_cursor.lnum);
    (buf, Integer::from(lnum - 1))
}

/// The line the cursor is on.
///
/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_get_current_line() -> Result<String_0, Error> {
    let (buf, lnum) = cursor_line();
    // SAFETY: `arena` is the caller's, and the pair names the cursor's line.
    unsafe { buffer_get_line(buf, lnum) }
}

/// Replace the line the cursor is on with `line`.
///
/// # Safety
/// `line` must name its own bytes and `arena` must be the caller's.
pub unsafe fn nvim_set_current_line(line: String_0, arena: *mut Arena) -> Result<(), Error> {
    let (buf, lnum) = cursor_line();
    // SAFETY: the caller's promise, and the pair names the cursor's line.
    unsafe { buffer_set_line(buf, lnum, line, arena) }
}

/// Delete the line the cursor is on.
///
/// # Safety
/// `arena` must be the caller's.
pub unsafe fn nvim_del_current_line(arena: *mut Arena) -> Result<(), Error> {
    let (buf, lnum) = cursor_line();
    // SAFETY: `arena` is the caller's, and the pair names the cursor's line.
    unsafe { buffer_del_line(buf, lnum, arena) }
}

/// The global variable `name`, autoloading the script that defines it if it
/// is not there yet.
///
/// # Safety
/// `name` must name its own bytes.
pub unsafe fn nvim_get_var(name: String_0) -> Result<Object, Error> {
    let mut error = Error::none();
    // SAFETY: the caller's promise about `name`.
    let mut di = unsafe { find_globvar(&name) };
    if di.is_null() {
        // SAFETY: as above.
        let loaded = unsafe { script_autoload(name.data(), name.len(), false) };
        if !loaded || aborting() {
            // SAFETY: `name` names its own NUL-terminated bytes.
            error = unsafe { key_not_found(&name) };
            return Object::Nil.reported(error);
        }
        // SAFETY: as above.
        di = unsafe { find_globvar(&name) };
    }
    if di.is_null() {
        // SAFETY: as above.
        error = unsafe { key_not_found(&name) };
        return Object::Nil.reported(error);
    }
    // SAFETY: `di` is the live dictionary item just found, and `arena` is the
    // caller's.
    // SAFETY: `di` is the live item the lookup found.
    Object::from(unsafe { &(*di).di_tv }).reported(error)
}

/// `g:name`'s dictionary item, or null.
///
/// # Safety
/// `name` must name its own bytes.
unsafe fn find_globvar(name: &String_0) -> *mut DictItem {
    // SAFETY: the caller's promise; the global dictionary is live from
    // startup to exit.
    let len: ptrdiff_t = name.len().cast_signed();
    unsafe { tv_dict_find(get_globvar_dict(), name.data(), len) }
}

/// "Key not found: `name`".
///
/// # Safety
/// `name` must be NUL-terminated.
unsafe fn key_not_found(name: &String_0) -> Error {
    // SAFETY: the caller's promise.
    let name = unsafe { c_str(name.data()) };
    api_error!(kErrorTypeValidation, "Key not found: {name}")
}

/// Set the global variable `name` to `value`.
///
/// # Safety
/// `name` and `value` must name their own contents.
pub unsafe fn nvim_set_var(name: String_0, value: Object) -> Result<(), Error> {
    let dict = get_globvar_dict();
    // SAFETY: the caller's promise, and `error` is this frame's own slot. The
    // null arena means the value is copied rather than borrowed.
    unsafe { dict_set_var(dict, &name, value, false, false) }.map(|_| ())
}

/// Remove the global variable `name`.
///
/// # Safety
/// `name` must name its own bytes.
pub unsafe fn nvim_del_var(name: String_0) -> Result<(), Error> {
    let dict = get_globvar_dict();
    // SAFETY: as [`nvim_set_var`]; `del` says to remove rather than assign.
    unsafe { dict_set_var(dict, &name, Object::Nil, true, false) }.map(|_| ())
}

/// The `v:` variable `name`.
///
/// # Safety
/// `name` must name its own bytes.
pub unsafe fn nvim_get_vvar(name: String_0) -> Result<Object, Error> {
    // SAFETY: the caller's promise; `v:` is live from startup to exit and
    // `error` is this frame's own slot.
    unsafe { dict_get_value(get_vimvar_dict(), &name) }
}

/// Set the `v:` variable `name` to `value`.
///
/// # Safety
/// `name` and `value` must name their own contents.
pub unsafe fn nvim_set_vvar(name: String_0, value: Object) -> Result<(), Error> {
    let dict = get_vimvar_dict();
    // SAFETY: as [`nvim_set_var`], over `v:` rather than the globals.
    unsafe { dict_set_var(dict, &name, value, false, false) }.map(|_| ())
}
