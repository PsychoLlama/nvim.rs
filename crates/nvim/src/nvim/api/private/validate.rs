use crate::src::nvim::api::private::helpers::{api_set_error, api_typename};

use crate::src::nvim::main::IObuff;
use crate::src::nvim::os::libc::{memchr, snprintf, strchr};
use crate::src::nvim::types::api::kErrorTypeValidation;
pub use crate::src::nvim::types::{
    Array, Boolean, Dict, Error, ErrorType, Float, Integer, KeyValuePair, LuaRef, Object,
    ObjectType, String_0, int64_t, kObjectTypeString, key_value_pair, object,
    object_data as C2Rust_Unnamed, size_t,
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub unsafe extern "C" fn api_err_invalid(
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
                b"Invalid %s\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"Invalid '%s'\0".as_ptr() as *const ::core::ffi::c_char
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
                b"Invalid %s: %ld\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"Invalid '%s': %ld\0".as_ptr() as *const ::core::ffi::c_char
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
                b"Invalid %s: '%s'\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"Invalid %s: %s\0".as_ptr() as *const ::core::ffi::c_char
            },
            name,
            val_s,
        );
    } else {
        api_set_error(
            err,
            errtype,
            if quote_val as ::core::ffi::c_int != 0 {
                b"Invalid '%s': '%s'\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"Invalid '%s': %s\0".as_ptr() as *const ::core::ffi::c_char
            },
            name,
            val_s,
        );
    };
}
pub unsafe extern "C" fn api_err_exp(
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
                b"Invalid %s: expected %s\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"Invalid '%s': expected %s\0".as_ptr() as *const ::core::ffi::c_char
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
            b"Invalid %s: expected %s, got %s\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"Invalid '%s': expected %s, got %s\0".as_ptr() as *const ::core::ffi::c_char
        },
        name,
        expected,
        actual,
    );
}
pub unsafe extern "C" fn api_err_required(
    mut err: *mut Error,
    mut name: *const ::core::ffi::c_char,
) {
    let mut errtype: ErrorType = kErrorTypeValidation;
    let mut has_space: *const ::core::ffi::c_char = strchr(name, ' ' as ::core::ffi::c_int);
    api_set_error(
        err,
        errtype,
        if !has_space.is_null() {
            b"Required: %s\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"Required: '%s'\0".as_ptr() as *const ::core::ffi::c_char
        },
        name,
    );
}
pub unsafe extern "C" fn api_err_conflict(
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
            b"Conflict: '%s' not allowed with %s\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"Conflict: '%s' not allowed with '%s'\0".as_ptr() as *const ::core::ffi::c_char
        },
        name,
        name2,
    );
}
pub unsafe extern "C" fn check_string_array(
    mut arr: Array,
    mut name: *mut ::core::ffi::c_char,
    mut disallow_nl: bool,
    mut err: *mut Error,
) -> bool {
    snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        b"'%s' item\0".as_ptr() as *const ::core::ffi::c_char,
        name,
    );
    let mut i: size_t = 0 as size_t;
    while i < arr.size {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != (*arr.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                api_typename(kObjectTypeString),
                api_typename((*arr.items.offset(i as isize)).type_0),
            );
            return false;
        }
        if disallow_nl {
            let l: String_0 = (*arr.items.offset(i as isize)).data.string;
            if !memchr(
                l.data as *const ::core::ffi::c_void,
                '\n' as ::core::ffi::c_int,
                l.size,
            )
            .is_null()
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"'%s' item contains newlines\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                );
                return false;
            }
        }
        i = i.wrapping_add(1);
    }
    return true_0 != 0;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
