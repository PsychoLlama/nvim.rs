//! Global and `v:` variables, and the current line.
//!
//! The `nvim_{get,set,del}_var` trio over the global dictionary and the
//! `nvim_{get,set}_vvar` pair over `v:`, plus the three current-line
//! accessors, which are the same shape: one lookup and one conversion
//! through the api's Object bridge.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported};

/// The current buffer's handle and the cursor's line, as the deprecated
/// `buffer_*_line` trio take them -- a zero-based index.
fn cursor_line() -> (Buffer, Integer) {
    // SAFETY: `curbuf` and `curwin` name live objects for the editor's whole
    // run.
    let (buf, lnum) = unsafe { ((*curbuf.get()).handle, (*curwin.get()).w_cursor.lnum) };
    (buf, Integer::from(lnum - 1))
}

/// The line the cursor is on.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_get_current_line(arena: *mut Arena) -> Result<String_0, Error> {
    let (buf, lnum) = cursor_line();
    // SAFETY: `arena` is the caller's, and the pair names the cursor's line.
    unsafe { buffer_get_line(buf, lnum, arena) }
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
/// `name` must name its own bytes and `arena` must be the caller's.
pub unsafe fn nvim_get_var(name: String_0, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: the caller's promise about `name`.
    let mut di = unsafe { find_globvar(name) };
    if di.is_null() {
        // SAFETY: as above.
        let loaded = unsafe { script_autoload(name.data(), name.len(), false) };
        if !loaded || aborting() {
            // SAFETY: `err` is this frame's own slot, and the format takes the
            // one C string it is given.
            unsafe { key_not_found(err, name) };
            return NIL.reported(error);
        }
        // SAFETY: as above.
        di = unsafe { find_globvar(name) };
    }
    if di.is_null() {
        // SAFETY: as above.
        unsafe { key_not_found(err, name) };
        return NIL.reported(error);
    }
    // SAFETY: `di` is the live dictionary item just found, and `arena` is the
    // caller's.
    unsafe { vim_to_object(&raw mut (*di).di_tv, arena, true) }.reported(error)
}

/// `g:name`'s dictionary item, or null.
///
/// # Safety
/// `name` must name its own bytes.
unsafe fn find_globvar(name: String_0) -> *mut dictitem_T {
    // SAFETY: the caller's promise; the global dictionary is live from
    // startup to exit.
    unsafe { tv_dict_find(get_globvar_dict(), name.data(), name.len() as ptrdiff_t) }
}

/// "Key not found: `name`".
///
/// # Safety
/// `err` must be the caller's error slot and `name` must be NUL-terminated.
unsafe fn key_not_found(err: *mut Error, name: String_0) {
    let fmt = c"Key not found: %s".as_ptr();
    // SAFETY: the caller's promise; the format takes the one C string given.
    unsafe { api_set_error(err, kErrorTypeValidation, fmt, name.data()) };
}

/// Set the global variable `name` to `value`.
///
/// # Safety
/// `name` and `value` must name their own contents.
pub unsafe fn nvim_set_var(name: String_0, value: Object) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let dict = get_globvar_dict();
    // SAFETY: the caller's promise, and `err` is this frame's own slot. The
    // null arena means the value is copied rather than borrowed.
    unsafe { dict_set_var(dict, name, value, false, false, NO_ARENA, err) };
    ().reported(error)
}

/// Remove the global variable `name`.
///
/// # Safety
/// `name` must name its own bytes.
pub unsafe fn nvim_del_var(name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let dict = get_globvar_dict();
    // SAFETY: as [`nvim_set_var`]; `del` says to remove rather than assign.
    unsafe { dict_set_var(dict, name, NIL, true, false, NO_ARENA, err) };
    ().reported(error)
}

/// The `v:` variable `name`.
///
/// # Safety
/// `name` must name its own bytes and `arena` must be the caller's.
pub unsafe fn nvim_get_vvar(name: String_0, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: the caller's promise; `v:` is live from startup to exit and
    // `err` is this frame's own slot.
    unsafe { dict_get_value(get_vimvar_dict(), name, arena, err) }.reported(error)
}

/// Set the `v:` variable `name` to `value`.
///
/// # Safety
/// `name` and `value` must name their own contents.
pub unsafe fn nvim_set_vvar(name: String_0, value: Object) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let dict = get_vimvar_dict();
    // SAFETY: as [`nvim_set_var`], over `v:` rather than the globals.
    unsafe { dict_set_var(dict, name, value, false, false, NO_ARENA, err) };
    ().reported(error)
}

/// No arena: `dict_set_var` copies the value rather than borrowing it.
const NO_ARENA: *mut Arena = ::core::ptr::null_mut();
