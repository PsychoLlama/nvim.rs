//! Buffer-local marks.
//!
//! `nvim_buf_set_mark` and `nvim_buf_del_mark` take the lowercase marks a
//! buffer owns; `nvim_buf_get_mark` also answers for the marks the *global*
//! mark list holds against this buffer, which is why it is the long one.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add};
use core::ffi::{CStr, c_char};

/// "Invalid `name`: '`val`'", for a value the caller spelled wrong.
///
/// # Safety
/// `err` must be the caller's error slot and `val` null or a C string.
unsafe fn err_bad_value(err: *mut Error, name: &CStr, val: *const c_char) {
    // SAFETY: the caller's promise; `name` is a C string too.
    unsafe { api_err_invalid(err, name.as_ptr(), val, 0, true) };
}

pub unsafe fn nvim_buf_del_mark(buf: Buffer, name: String_0) -> Result<Boolean, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // The record `mark_get` answers into; see `mark_get`.
    let mut slot = fmark_T::UNSET;
    let mut res: bool = false;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return (res as Boolean).reported(error);
    }
    if !(name.len() == 1 as size_t) {
        // SAFETY: `err` is this call's own error slot.
        unsafe { err_bad_value(err, c"mark name (must be a single char)", name.data()) };
        return (res as Boolean).reported(error);
    }
    let mut fm: *mut fmark_T = unsafe {
        mark_get(
            b,
            curwin.get(),
            &raw mut slot,
            kMarkAllNoResolve,
            *name.data() as ::core::ffi::c_int,
        )
    };
    if fm.is_null() {
        unsafe { api_err_invalid(err, c"mark name".as_ptr(), name.data(), 0 as int64_t, true) };
        return (res as Boolean).reported(error);
    }
    if unsafe { (*fm).mark.lnum } != 0 as linenr_T
        && unsafe { (*fm).fnum } == unsafe { (*b).handle }
    {
        res = unsafe { set_mark(b, name, 0 as Integer, 0 as Integer, err) };
    }
    (res as Boolean).reported(error)
}

pub unsafe fn nvim_buf_set_mark(
    buf: Buffer,
    name: String_0,
    line: Integer,
    col: Integer,
    _opts: *mut KeyDict_empty,
) -> Result<Boolean, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut res: bool = false;
    let mut b: *mut buf_T = unsafe { api_buf_ensure_loaded(buf, err) };
    if b.is_null() {
        return (res as Boolean).reported(error);
    }
    if !(name.len() == 1 as size_t) {
        // SAFETY: `err` is this call's own error slot.
        unsafe { err_bad_value(err, c"mark name (must be a single char)", name.data()) };
        return (res as Boolean).reported(error);
    }
    res = unsafe { set_mark(b, name, line, col, err) };
    (res as Boolean).reported(error)
}

pub unsafe fn nvim_buf_get_mark(
    buf: Buffer,
    name: String_0,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // The record `mark_get` answers into; see `mark_get`.
    let mut slot = fmark_T::UNSET;
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return rv.reported(error);
    }
    if !(name.len() == 1 as size_t) {
        // SAFETY: `err` is this call's own error slot.
        unsafe { err_bad_value(err, c"mark name (must be a single char)", name.data()) };
        return rv.reported(error);
    }
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut mark: ::core::ffi::c_char = unsafe { *name.data() };
    fm = unsafe {
        mark_get(
            b,
            curwin.get(),
            &raw mut slot,
            kMarkAllNoResolve,
            mark as ::core::ffi::c_int,
        )
    };
    if fm.is_null() {
        unsafe { api_err_invalid(err, c"mark name".as_ptr(), name.data(), 0 as int64_t, true) };
        return rv.reported(error);
    }
    if unsafe { (*fm).fnum } != unsafe { (*b).handle } {
        pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
        pos.col = 0 as ::core::ffi::c_int as colnr_T;
    } else {
        pos = unsafe { (*fm).mark };
    }
    rv = arena_array(arena, 2 as size_t);
    unsafe { array_add(&mut rv, Object::integer(pos.lnum as Integer)) };
    unsafe { array_add(&mut rv, Object::integer(pos.col as Integer)) };
    rv.reported(error)
}
