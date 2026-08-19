//! Global and `v:` variables, and the current line.
//!
//! The `nvim_{get,set,del}_var` trio over the global dictionary and the
//! `nvim_{get,set}_vvar` pair over `v:`, plus the three current-line
//! accessors, which are the same shape: one lookup and one conversion
//! through the api's Object bridge.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported};

pub unsafe fn nvim_get_current_line(arena: *mut Arena) -> Result<String_0, Error> {
    unsafe {
        buffer_get_line(
            (*curbuf.get()).handle as Buffer,
            ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
            arena,
        )
    }
}

pub unsafe fn nvim_set_current_line(line: String_0, arena: *mut Arena) -> Result<(), Error> {
    unsafe {
        buffer_set_line(
            (*curbuf.get()).handle as Buffer,
            ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
            line,
            arena,
        )
    }
}

pub unsafe fn nvim_del_current_line(arena: *mut Arena) -> Result<(), Error> {
    unsafe {
        buffer_del_line(
            (*curbuf.get()).handle as Buffer,
            ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
            arena,
        )
    }
}

pub unsafe fn nvim_get_var(name: String_0, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut di: *mut dictitem_T =
            tv_dict_find(get_globvar_dict(), name.data, name.size as ptrdiff_t);
        if di.is_null() {
            let mut found: bool =
                script_autoload(name.data, name.size, false) as ::core::ffi::c_int != 0
                    && !aborting();
            if !found {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Key not found: %s".as_ptr(),
                    name.data,
                );
                return NIL.reported(error);
            }
            di = tv_dict_find(get_globvar_dict(), name.data, name.size as ptrdiff_t);
        }
        if di.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Key not found: %s".as_ptr(),
                name.data,
            );
            return NIL.reported(error);
        }
        return vim_to_object(&raw mut (*di).di_tv, arena, true).reported(error);
    }
}

pub unsafe fn nvim_set_var(name: String_0, value: Object) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        dict_set_var(
            get_globvar_dict(),
            name,
            value,
            false,
            false,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
    ().reported(error)
}

pub unsafe fn nvim_del_var(name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        dict_set_var(
            get_globvar_dict(),
            name,
            NIL,
            true,
            false,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
    ().reported(error)
}

pub unsafe fn nvim_get_vvar(name: String_0, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        return dict_get_value(get_vimvar_dict(), name, arena, err).reported(error);
    }
}

pub unsafe fn nvim_set_vvar(name: String_0, value: Object) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        dict_set_var(
            get_vimvar_dict(),
            name,
            value,
            false,
            false,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
    ().reported(error)
}
