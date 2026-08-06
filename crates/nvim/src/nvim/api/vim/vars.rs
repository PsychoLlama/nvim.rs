//! Global and `v:` variables, and the current line.
//!
//! The `nvim_{get,set,del}_var` trio over the global dictionary and the
//! `nvim_{get,set}_vvar` pair over `v:`, plus the three current-line
//! accessors, which are the same shape: one lookup and one conversion
//! through the api's Object bridge.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_get_current_line(
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    unsafe {
        return buffer_get_line(
            (*curbuf.get()).handle as Buffer,
            ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_set_current_line(
    mut line: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        buffer_set_line(
            (*curbuf.get()).handle as Buffer,
            ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
            line,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_del_current_line(mut arena: *mut Arena, mut err: *mut Error) {
    unsafe {
        buffer_del_line(
            (*curbuf.get()).handle as Buffer,
            ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_get_var(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
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
                return object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                };
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
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return vim_to_object(&raw mut (*di).di_tv, arena, true);
    }
}

pub unsafe extern "C" fn nvim_set_var(mut name: String_0, mut value: Object, mut err: *mut Error) {
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
}

pub unsafe extern "C" fn nvim_del_var(mut name: String_0, mut err: *mut Error) {
    unsafe {
        dict_set_var(
            get_globvar_dict(),
            name,
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            true,
            false,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_get_vvar(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return dict_get_value(get_vimvar_dict(), name, arena, err);
    }
}

pub unsafe extern "C" fn nvim_set_vvar(mut name: String_0, mut value: Object, mut err: *mut Error) {
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
}
