//! `nvim_get_hl_by_id()` and `nvim_get_hl_by_name()`.
//!
//! Both are `nvim_get_hl` with the namespace fixed to the global one and the
//! result rendered in the old `rgb`/`cterm` shape.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};

pub unsafe fn nvim_get_hl_by_id(
    hl_id: Integer,
    rgb: Boolean,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut dic: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        if !(syn_get_final_id(hl_id as ::core::ffi::c_int) != 0 as ::core::ffi::c_int) {
            api_err_invalid(
                err,
                c"highlight id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                hl_id as int64_t,
                false,
            );
            return dic.reported(error);
        }
        let mut attrcode: ::core::ffi::c_int = syn_id2attr(hl_id as ::core::ffi::c_int);
        return hl_get_attr_by_id(attrcode as Integer, rgb, arena, err).reported(error);
    }
}

pub unsafe fn nvim_get_hl_by_name(
    name: String_0,
    rgb: Boolean,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut result: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut id: ::core::ffi::c_int = syn_name2id(name.data);
        if !(id != 0 as ::core::ffi::c_int) {
            api_err_invalid(
                err,
                c"highlight name".as_ptr(),
                name.data,
                0 as int64_t,
                true,
            );
            return result.reported(error);
        }
        nvim_get_hl_by_id(id as Integer, rgb, arena)
    }
}
