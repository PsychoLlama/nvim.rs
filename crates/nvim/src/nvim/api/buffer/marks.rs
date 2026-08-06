//! Buffer-local marks.
//!
//! `nvim_buf_set_mark` and `nvim_buf_del_mark` take the lowercase marks a
//! buffer owns; `nvim_buf_get_mark` also answers for the marks the *global*
//! mark list holds against this buffer, which is why it is the long one.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_buf_del_mark(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) -> Boolean {
    unsafe {
        let mut res: bool = false_0 != 0;
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return res as Boolean;
        }
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
            return res as Boolean;
        }
        let mut fm: *mut fmark_T = mark_get(
            b,
            curwin.get(),
            ::core::ptr::null_mut::<fmark_T>(),
            kMarkAllNoResolve,
            *name.data as ::core::ffi::c_int,
        );
        if fm.is_null() {
            api_err_invalid(
                err,
                b"mark name\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
            return res as Boolean;
        }
        if (*fm).mark.lnum != 0 as linenr_T && (*fm).fnum == (*b).handle {
            res = set_mark(b, name, 0 as Integer, 0 as Integer, err);
        }
        return res as Boolean;
    }
}

pub unsafe extern "C" fn nvim_buf_set_mark(
    mut buf: Buffer,
    mut name: String_0,
    mut line: Integer,
    mut col: Integer,
    mut _opts: *mut KeyDict_empty,
    mut err: *mut Error,
) -> Boolean {
    unsafe {
        let mut res: bool = false_0 != 0;
        let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
        if b.is_null() {
            return res as Boolean;
        }
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
            return res as Boolean;
        }
        res = set_mark(b, name, line, col, err);
        return res as Boolean;
    }
}

pub unsafe extern "C" fn nvim_buf_get_mark(
    mut buf: Buffer,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let mut rv: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return rv;
        }
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
            return rv;
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
            api_err_invalid(
                err,
                b"mark name\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
            return rv;
        }
        if (*fm).fnum != (*b).handle {
            pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
            pos.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            pos = (*fm).mark;
        }
        rv = arena_array(arena, 2 as size_t);
        let c2rust_fresh2 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: pos.lnum as Integer,
            },
        };
        let c2rust_fresh3 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: pos.col as Integer,
            },
        };
        return rv;
    }
}
