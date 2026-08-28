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
    // SAFETY: these take a highlight-group id rather than a pointer.
    let known = unsafe { syn_get_final_id(hl_id as ::core::ffi::c_int) } != 0;
    if !known {
        let null = ::core::ptr::null::<::core::ffi::c_char>();
        // SAFETY: `err` is this frame's slot; a null value string asks for
        // the numeric spelling.
        unsafe { api_err_invalid(err, c"highlight id".as_ptr(), null, hl_id, false) };
        return Dict::EMPTY.reported(error);
    }
    // SAFETY: as above.
    let attrcode = unsafe { syn_id2attr(hl_id as ::core::ffi::c_int) };
    // SAFETY: `arena` is the caller's and `err` this frame's slot.
    unsafe { hl_get_attr_by_id(attrcode as Integer, rgb, arena, err) }.reported(error)
}

pub unsafe fn nvim_get_hl_by_name(
    name: String_0,
    rgb: Boolean,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: `name` is the caller's NUL-terminated group name.
    let id = unsafe { syn_name2id(name.data()) };
    if id == 0 {
        // SAFETY: `err` is this frame's slot and `name` a C string.
        unsafe { api_err_invalid(err, c"highlight name".as_ptr(), name.data(), 0, true) };
        return Dict::EMPTY.reported(error);
    }
    // SAFETY: `arena` is the caller's.
    unsafe { nvim_get_hl_by_id(id as Integer, rgb, arena) }
}
