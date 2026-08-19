//! Buffer-local marks.
//!
//! `nvim_buf_set_mark` and `nvim_buf_del_mark` take the lowercase marks a
//! buffer owns; `nvim_buf_get_mark` also answers for the marks the *global*
//! mark list holds against this buffer, which is why it is the long one.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add};

pub unsafe fn nvim_buf_del_mark(buf: Buffer, name: String_0) -> Result<Boolean, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut res: bool = false;
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return (res as Boolean).reported(error);
        }
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                c"mark name (must be a single char)".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return (res as Boolean).reported(error);
        }
        let mut fm: *mut fmark_T = mark_get(
            b,
            curwin.get(),
            ::core::ptr::null_mut::<fmark_T>(),
            kMarkAllNoResolve,
            *name.data as ::core::ffi::c_int,
        );
        if fm.is_null() {
            api_err_invalid(err, c"mark name".as_ptr(), name.data, 0 as int64_t, true);
            return (res as Boolean).reported(error);
        }
        if (*fm).mark.lnum != 0 as linenr_T && (*fm).fnum == (*b).handle {
            res = set_mark(b, name, 0 as Integer, 0 as Integer, err);
        }
        return (res as Boolean).reported(error);
    }
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
    unsafe {
        let mut res: bool = false;
        let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
        if b.is_null() {
            return (res as Boolean).reported(error);
        }
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                c"mark name (must be a single char)".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return (res as Boolean).reported(error);
        }
        res = set_mark(b, name, line, col, err);
        return (res as Boolean).reported(error);
    }
}

pub unsafe fn nvim_buf_get_mark(
    buf: Buffer,
    name: String_0,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut rv: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return rv.reported(error);
        }
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                c"mark name (must be a single char)".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return rv.reported(error);
        }
        let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
        let mut pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut mark: ::core::ffi::c_char = *name.data;
        fm = mark_get(
            b,
            curwin.get(),
            ::core::ptr::null_mut::<fmark_T>(),
            kMarkAllNoResolve,
            mark as ::core::ffi::c_int,
        );
        if fm.is_null() {
            api_err_invalid(err, c"mark name".as_ptr(), name.data, 0 as int64_t, true);
            return rv.reported(error);
        }
        if (*fm).fnum != (*b).handle {
            pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
            pos.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            pos = (*fm).mark;
        }
        rv = arena_array(arena, 2 as size_t);
        array_add(&mut rv, Object::integer(pos.lnum as Integer));
        array_add(&mut rv, Object::integer(pos.col as Integer));
        return rv.reported(error);
    }
}
