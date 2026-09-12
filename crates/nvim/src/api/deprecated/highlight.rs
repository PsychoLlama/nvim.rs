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

/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_get_hl_by_id(hl_id: Integer, rgb: Boolean) -> Result<ApiDict, Error> {
    let known = syn_get_final_id(number_as_int(hl_id)) != 0;
    if !known {
        return Err(err_bad_number(c"highlight id", hl_id));
    }
    let attrcode = syn_id2attr(number_as_int(hl_id));
    hl_get_attr_by_id(Integer::from(attrcode), rgb)
}

/// # Safety
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_get_hl_by_name(name: String_0, rgb: Boolean) -> Result<ApiDict, Error> {
    let error;
    // SAFETY: `name` is the caller's NUL-terminated group name.
    let id = unsafe { syn_name2id(name.data()) };
    if id == 0 {
        // SAFETY: the caller's highlight name is NUL-terminated.
        error = err_bad_value(c"highlight name", name.as_cstr());
        return ApiDict::EMPTY.reported(error);
    }
    // SAFETY: `arena` is the caller's.
    unsafe { nvim_get_hl_by_id(Integer::from(id), rgb) }
}
