//! Global marks.
//!
//! `nvim_get_mark` resolves an uppercase mark to (row, col, buffer, name),
//! loading the buffer's mark list from ShaDa if the buffer is not
//! currently loaded, and `nvim_del_mark` removes one.

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
use crate::api::private::validate::err_bad_value;
use crate::ascii::ascii_isdigit;
use crate::cstr;
use core::ffi::{CStr, c_char, c_int};

/// The one character `name` spells, when it names a global mark: an
/// uppercase letter, or a digit for one of the numbered file marks.
///
/// Refuses when it is neither, or is not one character.
///
/// # Safety
/// `name` must name its own bytes.
unsafe fn global_mark_name(name: &String_0) -> Result<c_char, Error> {
    if name.len() != 1 {
        // SAFETY: the caller's promise about `name`.
        return Err(unsafe { reject(c"mark name (must be a single char)", name.clone()) });
    }
    // SAFETY: the caller's promise -- `name` has the one byte read here.
    let mark = unsafe { *name.data() };
    if !(mark.cast_unsigned().is_ascii_uppercase() || ascii_isdigit(c_int::from(mark))) {
        // SAFETY: as above.
        return Err(unsafe { reject(c"mark name (must be file/uppercase)", name.clone()) });
    }
    Ok(mark)
}

/// "Invalid `what`: '`name`'".
///
/// # Safety
/// `name` must be NUL-terminated.
unsafe fn reject(what: &CStr, name: String_0) -> Error {
    let (what, got) = (what.as_ptr(), name.data());
    // SAFETY: the names and values are NUL-terminated strings.
    err_bad_value(unsafe { cstr::at(what) }, unsafe { cstr::at(got) })
}

/// Remove the global mark `name`.
///
/// # Safety
/// `name` must name its own bytes.
pub unsafe fn nvim_del_mark(name: String_0) -> Result<Boolean, Error> {
    // SAFETY: `name` is the caller's.
    unsafe { global_mark_name(&name) }?;
    // SAFETY: a global mark takes no buffer.
    unsafe { set_mark(None, name, 0, 0) }?;
    Ok(true)
}

/// The global mark `name`, as `[row, col, buffer, filename]`.
///
/// A mark that names nothing, or names a line that is gone, answers
/// `[0, 0, 0, ""]` rather than an error.
///
/// # Safety
/// `name` must name its own bytes and `arena` must be the caller's.
pub unsafe fn nvim_get_mark(name: String_0, _opts: *mut KeyDict_empty) -> Result<Array, Error> {
    // SAFETY: `name` is the caller's.
    let mark = unsafe { global_mark_name(&name) }?;
    // SAFETY: `mark_get_global` answers a live global mark for every name
    // this one accepts -- the slot exists whether or not it is set.
    let (pos, fnum, fname) = unsafe {
        let mark = mark_get_global(false, c_int::from(mark));
        ((*mark).fmark.mark, (*mark).fmark.fnum, (*mark).fname)
    };
    // A mark in a buffer names the buffer; one restored from ShaDa names
    // only the file it was in, and `fname` is the mark's own storage.
    let mut allocated = fnum != 0;
    let mut bufnr = fnum;
    let mut filename = if allocated {
        buflist_nr2name(bufnr, 1, 1)
    } else {
        fname
    };
    let mut row: Integer = 0;
    let mut col: Integer = 0;
    if filename.is_null() || pos.lnum <= 0 as LineNr {
        if allocated {
            // SAFETY: `buflist_nr2name` handed back an allocation.
            unsafe { xfree(filename.cast()) };
            allocated = false;
        }
        filename = c"".as_ptr().cast_mut();
        bufnr = 0;
    } else {
        row = Integer::from(pos.lnum);
        col = Integer::from(pos.col);
    }
    let mut rv = Array::with_capacity(4 as size_t);
    // SAFETY: `filename` is NUL-terminated and `arena` is the caller's, so
    // the copy outlives the answer.
    let path = unsafe { Object::string(cstr_to_string(filename)) };
    // SAFETY: `rv` is the four-slot block the arena just handed back.
    rv.push(Object::integer(row));
    rv.push(Object::integer(col));
    rv.push(Object::integer(Integer::from(bufnr)));
    rv.push(path);
    if allocated {
        // SAFETY: as above -- the arena has its own copy now.
        unsafe { xfree(filename.cast()) };
    }
    Ok(rv)
}
