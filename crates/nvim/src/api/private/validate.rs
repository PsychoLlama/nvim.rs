use crate::api::private::helpers::{api_set_error, api_typename};

use crate::main::IObuff;
use crate::os::cshim::{snprintf, strchr};
use crate::types::{
    Array, Error, ErrorType, NUL, String_0, int64_t, kErrorTypeValidation, kObjectTypeString,
    size_t,
};
use ::libc::memchr;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub unsafe fn api_err_invalid(
    mut err: *mut Error,
    mut name: *const ::core::ffi::c_char,
    mut val_s: *const ::core::ffi::c_char,
    mut val_n: int64_t,
    mut quote_val: bool,
) {
    let mut errtype: ErrorType = kErrorTypeValidation;
    let mut has_space: *const ::core::ffi::c_char = strchr(name, ' ' as ::core::ffi::c_int);
    if !val_s.is_null()
        && *val_s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        api_set_error(
            err,
            errtype,
            if !has_space.is_null() {
                c"Invalid %s".as_ptr()
            } else {
                c"Invalid '%s'".as_ptr()
            },
            name,
        );
        return;
    }
    if val_s.is_null() {
        api_set_error(
            err,
            errtype,
            if !has_space.is_null() {
                c"Invalid %s: %ld".as_ptr()
            } else {
                c"Invalid '%s': %ld".as_ptr()
            },
            name,
            val_n,
        );
        return;
    }
    if !has_space.is_null() {
        api_set_error(
            err,
            errtype,
            if quote_val as ::core::ffi::c_int != 0 {
                c"Invalid %s: '%s'".as_ptr()
            } else {
                c"Invalid %s: %s".as_ptr()
            },
            name,
            val_s,
        );
    } else {
        api_set_error(
            err,
            errtype,
            if quote_val as ::core::ffi::c_int != 0 {
                c"Invalid '%s': '%s'".as_ptr()
            } else {
                c"Invalid '%s': %s".as_ptr()
            },
            name,
            val_s,
        );
    };
}
pub unsafe fn api_err_exp(
    mut err: *mut Error,
    mut name: *const ::core::ffi::c_char,
    mut expected: *const ::core::ffi::c_char,
    mut actual: *const ::core::ffi::c_char,
) {
    let mut errtype: ErrorType = kErrorTypeValidation;
    let mut has_space: *const ::core::ffi::c_char = strchr(name, ' ' as ::core::ffi::c_int);
    if actual.is_null() {
        api_set_error(
            err,
            errtype,
            if !has_space.is_null() {
                c"Invalid %s: expected %s".as_ptr()
            } else {
                c"Invalid '%s': expected %s".as_ptr()
            },
            name,
            expected,
        );
        return;
    }
    api_set_error(
        err,
        errtype,
        if !has_space.is_null() {
            c"Invalid %s: expected %s, got %s".as_ptr()
        } else {
            c"Invalid '%s': expected %s, got %s".as_ptr()
        },
        name,
        expected,
        actual,
    );
}
pub unsafe fn api_err_required(mut err: *mut Error, mut name: *const ::core::ffi::c_char) {
    let mut errtype: ErrorType = kErrorTypeValidation;
    let mut has_space: *const ::core::ffi::c_char = strchr(name, ' ' as ::core::ffi::c_int);
    api_set_error(
        err,
        errtype,
        if !has_space.is_null() {
            c"Required: %s".as_ptr()
        } else {
            c"Required: '%s'".as_ptr()
        },
        name,
    );
}
pub unsafe fn api_err_conflict(
    mut err: *mut Error,
    mut name: *const ::core::ffi::c_char,
    mut name2: *const ::core::ffi::c_char,
) {
    let mut errtype: ErrorType = kErrorTypeValidation;
    let mut has_space2: *const ::core::ffi::c_char = strchr(name2, ' ' as ::core::ffi::c_int);
    api_set_error(
        err,
        errtype,
        if !has_space2.is_null() {
            c"Conflict: '%s' not allowed with %s".as_ptr()
        } else {
            c"Conflict: '%s' not allowed with '%s'".as_ptr()
        },
        name,
        name2,
    );
}
pub unsafe fn check_string_array(
    mut arr: Array,
    mut name: *mut ::core::ffi::c_char,
    mut disallow_nl: bool,
    mut err: *mut Error,
) -> bool {
    snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        c"'%s' item".as_ptr(),
        name,
    );
    let mut i: size_t = 0 as size_t;
    while i < arr.size {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != (*arr.items.add(i)).type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                api_typename(kObjectTypeString),
                api_typename((*arr.items.add(i)).type_0),
            );
            return false;
        }
        if disallow_nl {
            let l: String_0 = (*arr.items.add(i)).data.string;
            if !memchr(
                l.data() as *const ::core::ffi::c_void,
                '\n' as ::core::ffi::c_int,
                l.len(),
            )
            .is_null()
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"'%s' item contains newlines".as_ptr(),
                    name,
                );
                return false;
            }
        }
        i = i.wrapping_add(1);
    }
    return true;
}
