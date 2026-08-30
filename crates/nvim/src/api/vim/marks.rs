//! Global marks.
//!
//! `nvim_get_mark` resolves an uppercase mark to (row, col, buffer, name),
//! loading the buffer's mark list from ShaDa if the buffer is not
//! currently loaded, and `nvim_del_mark` removes one.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{Reported, array_add};
use crate::api::private::validate::err_bad_value;
use crate::ascii::ascii_isdigit;
use crate::cstr;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The one character `name` spells, when it names a global mark: an
/// uppercase letter, or a digit for one of the numbered file marks.
///
/// `None` -- with `err` set -- when it is neither, or is not one character.
///
/// # Safety
/// `name` must name its own bytes.
unsafe fn global_mark_name(name: String_0, err: &mut Error) -> Option<c_char> {
    if name.len() != 1 {
        // SAFETY: the caller's promise about `name` and `err`.
        unsafe { reject(err, c"mark name (must be a single char)", name) };
        return None;
    }
    // SAFETY: the caller's promise -- `name` has the one byte read here.
    let mark = unsafe { *name.data() };
    if !((mark as u8).is_ascii_uppercase() || ascii_isdigit(mark as c_int)) {
        // SAFETY: as above.
        unsafe { reject(err, c"mark name (must be file/uppercase)", name) };
        return None;
    }
    Some(mark)
}

/// "Invalid `what`: '`name`'".
///
/// # Safety
/// `name` must be NUL-terminated.
unsafe fn reject(err: &mut Error, what: &CStr, name: String_0) {
    let (what, got) = (what.as_ptr(), name.data());
    // SAFETY: the names and values are NUL-terminated strings.
    *err = err_bad_value(unsafe { cstr::at(what) }, unsafe { cstr::at(got) });
}

/// Remove the global mark `name`.
///
/// # Safety
/// `name` must name its own bytes.
pub unsafe fn nvim_del_mark(name: String_0) -> Result<Boolean, Error> {
    let mut error = Error::none();
    // SAFETY: `name` is the caller's and `error` this frame's own slot.
    if unsafe { global_mark_name(name, &mut error) }.is_none() {
        return false.reported(error);
    }
    let no_buf = ptr::null_mut::<buf_T>();
    // SAFETY: a global mark takes no buffer, and `error` is this frame's own.
    let res = unsafe { set_mark(no_buf, name, 0, 0, &mut error) };
    res.reported(error)
}

/// The global mark `name`, as `[row, col, buffer, filename]`.
///
/// A mark that names nothing, or names a line that is gone, answers
/// `[0, 0, 0, ""]` rather than an error.
///
/// # Safety
/// `name` must name its own bytes and `arena` must be the caller's.
pub unsafe fn nvim_get_mark(
    name: String_0,
    _opts: *mut KeyDict_empty,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = Error::none();
    // SAFETY: `name` is the caller's and `error` this frame's own slot.
    let Some(mark) = (unsafe { global_mark_name(name, &mut error) }) else {
        return Array::EMPTY.reported(error);
    };
    // SAFETY: `mark_get_global` answers a live global mark for every name
    // this one accepts -- the slot exists whether or not it is set.
    let (pos, fnum, fname) = unsafe {
        let mark = mark_get_global(false, mark as c_int);
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
    if filename.is_null() || pos.lnum <= 0 as linenr_T {
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
    let mut rv = arena_array(arena, 4 as size_t);
    // SAFETY: `filename` is NUL-terminated and `arena` is the caller's, so
    // the copy outlives the answer.
    let path = unsafe { Object::string(arena_string(arena, cstr_as_string(filename))) };
    // SAFETY: `rv` is the four-slot block the arena just handed back.
    unsafe {
        array_add(&mut rv, Object::integer(row));
        array_add(&mut rv, Object::integer(col));
        array_add(&mut rv, Object::integer(Integer::from(bufnr)));
        array_add(&mut rv, path);
    }
    if allocated {
        // SAFETY: as above -- the arena has its own copy now.
        unsafe { xfree(filename.cast()) };
    }
    rv.reported(error)
}
