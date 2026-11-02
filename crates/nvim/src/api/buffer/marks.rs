//! Buffer-local marks.
//!
//! `nvim_buf_set_mark` and `nvim_buf_del_mark` take the lowercase marks a
//! buffer owns; `nvim_buf_get_mark` also answers for the marks the *global*
//! mark list holds against this buffer, which is why it is the long one.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::err_bad_value;
use crate::winlayer::Win;

pub fn nvim_buf_del_mark(buf: BufferHandle, name: ApiString) -> Result<Boolean, Error> {
    let mut res: bool = false;
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(res as Boolean);
    };
    if !(name.len() == 1 as size_t) {
        let name = name.as_cstr();
        return Err(err_bad_value(c"mark name (must be a single char)", name));
    }
    // As C read it: a `char`, so a byte past ASCII is negative.
    let mark = ::core::ffi::c_int::from(::core::ffi::c_char::from_ne_bytes([name.as_bytes()[0]]));
    let Some(fm) = mark_get(b, Win::current(), kMarkAllNoResolve, mark) else {
        return Err(err_bad_value(c"mark name", name.as_cstr()));
    };
    if fm.mark.lnum != LineNr::ZERO && fm.fnum == b.handle.get() {
        set_mark(Some(b), name, 0 as Integer, 0 as Integer)?;
        res = true;
    }
    Ok(res as Boolean)
}

pub fn nvim_buf_set_mark(
    buf: BufferHandle,
    name: ApiString,
    line: Integer,
    col: Integer,
    _opts: &mut KeyDict_empty,
) -> Result<Boolean, Error> {
    let mut res: bool = false;
    let Some(b) = api_buf_ensure_loaded(buf)? else {
        return Ok(res as Boolean);
    };
    if !(name.len() == 1 as size_t) {
        let name = name.as_cstr();
        return Err(err_bad_value(c"mark name (must be a single char)", name));
    }
    set_mark(Some(b), name, line, col)?;
    res = true;
    Ok(res as Boolean)
}

pub fn nvim_buf_get_mark(buf: BufferHandle, name: ApiString) -> Result<Array, Error> {
    let mut rv = Vec::new();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv.into());
    };
    if !(name.len() == 1 as size_t) {
        let name = name.as_cstr();
        return Err(err_bad_value(c"mark name (must be a single char)", name));
    }
    let mut pos: Pos = Pos {
        lnum: LineNr::ZERO,
        col: 0,
        coladd: 0,
    };
    let mark = ::core::ffi::c_char::from_ne_bytes([name.as_bytes()[0]]);
    let Some(fm) = mark_get(
        b,
        Win::current(),
        kMarkAllNoResolve,
        ::core::ffi::c_int::from(mark),
    ) else {
        return Err(err_bad_value(c"mark name", name.as_cstr()));
    };
    if fm.fnum != b.handle.get() {
        pos.lnum = LineNr::ZERO;
        pos.col = 0 as ::core::ffi::c_int as ColNr;
    } else {
        pos = fm.mark;
    }
    rv = Vec::with_capacity(2 as size_t);
    rv.push(Object::integer(Integer::from(pos.lnum)));
    rv.push(Object::integer(Integer::from(pos.col)));
    Ok(rv.into())
}
