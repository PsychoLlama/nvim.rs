//! Buffer-local marks.
//!
//! `nvim_buf_set_mark` and `nvim_buf_del_mark` take the lowercase marks a
//! buffer owns; `nvim_buf_get_mark` also answers for the marks the *global*
//! mark list holds against this buffer, which is why it is the long one.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{Reported, array_add};
use crate::api::private::validate::err_bad_value;
use crate::winlayer::Win;

pub unsafe fn nvim_buf_del_mark(buf: BufferHandle, name: String_0) -> Result<Boolean, Error> {
    let mut error = Error::none();
    // The record `mark_get` answers into; see `mark_get`.
    let mut slot = FileMark::UNSET;
    let mut res: bool = false;
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return (res as Boolean).reported(error);
    };
    if !(name.len() == 1 as size_t) {
        // SAFETY: the value the keyset carried, live for this call.
        // SAFETY: the caller's mark name is NUL-terminated.
        let name = unsafe { name.as_cstr() };
        error = err_bad_value(c"mark name (must be a single char)", name);
        return (res as Boolean).reported(error);
    }
    let fm: *mut FileMark = unsafe {
        mark_get(
            b,
            Win::current(),
            &raw mut slot,
            kMarkAllNoResolve,
            *name.data() as ::core::ffi::c_int,
        )
    };
    if fm.is_null() {
        error = err_bad_value(c"mark name", unsafe { name.as_cstr() });
        return (res as Boolean).reported(error);
    }
    if unsafe { (*fm).mark.lnum } != 0 as LineNr && unsafe { (*fm).fnum } == b.handle {
        res = unsafe { set_mark(b.raw(), name, 0 as Integer, 0 as Integer, &mut error) };
    }
    (res as Boolean).reported(error)
}

pub unsafe fn nvim_buf_set_mark(
    buf: BufferHandle,
    name: String_0,
    line: Integer,
    col: Integer,
    _opts: *mut KeyDict_empty,
) -> Result<Boolean, Error> {
    let mut error = Error::none();
    let mut res: bool = false;
    let Some(b) = api_buf_ensure_loaded(buf, &mut error) else {
        return (res as Boolean).reported(error);
    };
    if !(name.len() == 1 as size_t) {
        // SAFETY: the value the keyset carried, live for this call.
        // SAFETY: the caller's mark name is NUL-terminated.
        let name = unsafe { name.as_cstr() };
        error = err_bad_value(c"mark name (must be a single char)", name);
        return (res as Boolean).reported(error);
    }
    res = unsafe { set_mark(b.raw(), name, line, col, &mut error) };
    (res as Boolean).reported(error)
}

pub unsafe fn nvim_buf_get_mark(
    buf: BufferHandle,
    name: String_0,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = Error::none();
    // The record `mark_get` answers into; see `mark_get`.
    let mut slot = FileMark::UNSET;
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return rv.reported(error);
    };
    if !(name.len() == 1 as size_t) {
        // SAFETY: the value the keyset carried, live for this call.
        // SAFETY: the caller's mark name is NUL-terminated.
        let name = unsafe { name.as_cstr() };
        error = err_bad_value(c"mark name (must be a single char)", name);
        return rv.reported(error);
    }
    let mut pos: Pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mark: ::core::ffi::c_char = unsafe { *name.data() };
    let fm: *mut FileMark = unsafe {
        mark_get(
            b,
            Win::current(),
            &raw mut slot,
            kMarkAllNoResolve,
            mark as ::core::ffi::c_int,
        )
    };
    if fm.is_null() {
        error = err_bad_value(c"mark name", unsafe { name.as_cstr() });
        return rv.reported(error);
    }
    if unsafe { (*fm).fnum } != b.handle {
        pos.lnum = 0 as ::core::ffi::c_int as LineNr;
        pos.col = 0 as ::core::ffi::c_int as ColNr;
    } else {
        pos = unsafe { (*fm).mark };
    }
    rv = arena_array(arena, 2 as size_t);
    unsafe { array_add(&mut rv, Object::integer(pos.lnum as Integer)) };
    unsafe { array_add(&mut rv, Object::integer(pos.col as Integer)) };
    rv.reported(error)
}
