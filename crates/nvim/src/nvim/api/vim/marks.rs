//! Global marks.
//!
//! `nvim_get_mark` resolves an uppercase mark to (row, col, buffer, name),
//! loading the buffer's mark list from ShaDa if the buffer is not
//! currently loaded, and `nvim_del_mark` removes one.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::array_add;

pub unsafe extern "C" fn nvim_del_mark(mut name: String_0, mut err: *mut Error) -> Boolean {
    unsafe {
        let mut res: bool = false;
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                c"mark name (must be a single char)".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return res as Boolean;
        }
        if !(*name.data as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *name.data as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || ascii_isdigit(*name.data as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
        {
            api_err_invalid(
                err,
                c"mark name (must be file/uppercase)".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return res as Boolean;
        }
        res = set_mark(
            ::core::ptr::null_mut::<buf_T>(),
            name,
            0 as Integer,
            0 as Integer,
            err,
        );
        return res as Boolean;
    }
}

pub unsafe extern "C" fn nvim_get_mark(
    mut name: String_0,
    mut _opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let mut rv: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        if !(name.size == 1 as size_t) {
            api_err_invalid(
                err,
                c"mark name (must be a single char)".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return rv;
        }
        if !(*name.data as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *name.data as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || ascii_isdigit(*name.data as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
        {
            api_err_invalid(
                err,
                c"mark name (must be file/uppercase)".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return rv;
        }
        let mut mark: *mut xfmark_T = mark_get_global(false, *name.data as ::core::ffi::c_int);
        let mut pos: pos_T = (*mark).fmark.mark;
        let mut allocated: bool = false;
        let mut bufnr: ::core::ffi::c_int = 0;
        let mut filename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*mark).fmark.fnum != 0 as ::core::ffi::c_int {
            bufnr = (*mark).fmark.fnum;
            filename = buflist_nr2name(bufnr, true_0, true_0);
            allocated = true;
        } else {
            filename = (*mark).fname;
            bufnr = 0 as ::core::ffi::c_int;
        }
        let mut exists: bool = !filename.is_null();
        let mut row: Integer = 0;
        let mut col: Integer = 0;
        if !exists || pos.lnum <= 0 as linenr_T {
            if allocated {
                xfree(filename as *mut ::core::ffi::c_void);
                allocated = false;
            }
            filename = c"".as_ptr() as *mut ::core::ffi::c_char;
            bufnr = 0 as ::core::ffi::c_int;
            row = 0 as Integer;
            col = 0 as Integer;
        } else {
            row = pos.lnum as Integer;
            col = pos.col as Integer;
        }
        rv = arena_array(arena, 4 as size_t);
        array_add(&mut rv, Object::integer(row));
        array_add(&mut rv, Object::integer(col));
        array_add(&mut rv, Object::integer(bufnr as Integer));
        array_add(
            &mut rv,
            Object::string(arena_string(arena, cstr_as_string(filename))),
        );
        if allocated {
            xfree(filename as *mut ::core::ffi::c_void);
        }
        return rv;
    }
}
