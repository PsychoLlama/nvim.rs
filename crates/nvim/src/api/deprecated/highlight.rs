//! `nvim_get_hl_by_id()` and `nvim_get_hl_by_name()`.
//!
//! Both are `nvim_get_hl` with the namespace fixed to the global one and the
//! result rendered in the old `rgb`/`cterm` shape.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_bad_number, err_bad_value};
use crate::narrow::number_as_int;

pub unsafe fn nvim_get_hl_by_id(
    hl_id: Integer,
    rgb: Boolean,
    arena: *mut Arena,
) -> Result<ApiDict, Error> {
    let mut error = Error::none();
    // SAFETY: these take a highlight-group id rather than a pointer.
    let known = unsafe { syn_get_final_id(number_as_int(hl_id)) } != 0;
    if !known {
        error = err_bad_number(c"highlight id", hl_id);
        return ApiDict::EMPTY.reported(error);
    }
    // SAFETY: as above.
    let attrcode = unsafe { syn_id2attr(number_as_int(hl_id)) };
    // SAFETY: `arena` is the caller's and `error` this frame's slot.
    unsafe { hl_get_attr_by_id(Integer::from(attrcode), rgb, arena, &mut error) }.reported(error)
}

pub unsafe fn nvim_get_hl_by_name(
    name: String_0,
    rgb: Boolean,
    arena: *mut Arena,
) -> Result<ApiDict, Error> {
    let error;
    // SAFETY: `name` is the caller's NUL-terminated group name.
    let id = unsafe { syn_name2id(name.data()) };
    if id == 0 {
        // SAFETY: the caller's highlight name is NUL-terminated.
        error = err_bad_value(c"highlight name", unsafe { name.as_cstr() });
        return ApiDict::EMPTY.reported(error);
    }
    // SAFETY: `arena` is the caller's.
    unsafe { nvim_get_hl_by_id(Integer::from(id), rgb, arena) }
}
