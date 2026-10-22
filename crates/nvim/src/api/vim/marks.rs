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
use crate::memory::XString;
use core::ffi::{CStr, c_char, c_int};

/// The one character `name` spells, when it names a global mark: an
/// uppercase letter, or a digit for one of the numbered file marks.
///
/// Refuses when it is neither, or is not one character.
fn global_mark_name(name: &String_0) -> Result<c_char, Error> {
    if name.len() != 1 {
        return Err(reject(c"mark name (must be a single char)", name.clone()));
    }
    // SAFETY: the caller's promise -- `name` has the one byte read here.
    let mark = unsafe { *name.data() };
    if !(mark.cast_unsigned().is_ascii_uppercase() || ascii_isdigit(c_int::from(mark))) {
        return Err(reject(c"mark name (must be file/uppercase)", name.clone()));
    }
    Ok(mark)
}

/// "Invalid `what`: '`name`'".
fn reject(what: &CStr, name: String_0) -> Error {
    let (what, got) = (what.as_ptr(), name.data());
    // SAFETY: the names and values are NUL-terminated strings.
    err_bad_value(unsafe { cstr::at(what) }, unsafe { cstr::at(got) })
}

/// Remove the global mark `name`.
pub fn nvim_del_mark(name: String_0) -> Result<Boolean, Error> {
    global_mark_name(&name)?;
    set_mark(None, name, 0, 0)?;
    Ok(true)
}

/// The global mark `name`, as `[row, col, buffer, filename]`.
///
/// A mark that names nothing, or names a line that is gone, answers
/// `[0, 0, 0, ""]` rather than an error.
pub fn nvim_get_mark(name: String_0, _opts: &mut KeyDict_empty) -> Result<Array, Error> {
    let mark = global_mark_name(&name)?;
    // A copy of the slot, which exists whether or not it is set.
    let mark = mark_get_global(false, c_int::from(mark));
    let (pos, fnum) = (mark.fmark.mark, mark.fmark.fnum);
    // A mark in a buffer names the buffer; one restored from ShaDa names
    // only the file it was in.
    let mut bufnr = fnum;
    let filename = if fnum != 0 {
        buflist_nr2name(bufnr, 1, 1)
    } else {
        mark.file_name().map(XString::from_cstr)
    };
    let mut row: Integer = 0;
    let mut col: Integer = 0;
    let filename = match filename {
        Some(name) if pos.lnum > 0 as LineNr => {
            row = Integer::from(pos.lnum);
            col = Integer::from(pos.col);
            name
        }
        _ => {
            bufnr = 0;
            XString::new()
        }
    };
    let mut rv = Array::with_capacity(4 as size_t);
    let path = Object::string(String_0::from_bytes(filename.as_cstr().to_bytes()));
    rv.push(Object::integer(row));
    rv.push(Object::integer(col));
    rv.push(Object::integer(Integer::from(bufnr)));
    rv.push(path);
    Ok(rv)
}
