//! Highlight groups, highlight namespaces and colour names.
//!
//! `nvim_set_hl` fills an `HlAttrs` from the keyset and installs it in a
//! namespace; `nvim_get_hl` renders one (or a whole namespace) back.  The
//! `*_hl_ns` trio switches which namespace the screen is drawn with -- the
//! `_fast` spelling being the one a fast callback may call -- and the two
//! colour functions are the built-in name table.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_bad_number, err_bad_value};

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_get_hl_id_by_name(name: String_0) -> Integer {
    unsafe { syn_check_group(name.data(), name.len()) as Integer }
}

/// # Safety
///
/// `opts` must point at the `KeyDict_get_highlight` the dispatcher filled in,
/// live for the call. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_get_hl(
    ns_id: Integer,
    opts: *mut KeyDict_get_highlight,
) -> Result<ApiDict, Error> {
    unsafe { ns_get_hl_defs(ns_id as NS, opts) }
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `val` must point at the `KeyDict_highlight` the
/// dispatcher filled in, live for the call.
pub unsafe fn nvim_set_hl(
    channel_id: uint64_t,
    ns_id: Integer,
    name: String_0,
    val: *mut KeyDict_highlight,
) -> Result<(), Error> {
    let hl_id: ::core::ffi::c_int = unsafe { syn_check_group(name.data(), name.len()) };
    if !(hl_id != 0 as ::core::ffi::c_int) {
        // SAFETY: the caller's highlight name is NUL-terminated.
        return Err(err_bad_value(c"highlight name", name.as_cstr()));
    }
    let mut link_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if unsafe { (*val).url.as_ref() }.is_some() {
        return Err(Error::validation(c"Invalid key: 'url'"));
    }
    let update: bool = unsafe { (*val).update }.unwrap_or(false);
    let mut base: Option<&HlAttrs> = None;
    let base_attrs: HlAttrs;
    if update
        && let Some(attrs) = unsafe { hl_ns_get_attrs(ns_id as ::core::ffi::c_int, hl_id, None) }
    {
        base_attrs = attrs;
        base = Some(&base_attrs);
    }
    let attrs: HlAttrs = unsafe { dict2hlattrs(&*val, true, Some(&mut link_id), base) }?;
    let _sctx = api_set_sctx(channel_id);
    unsafe { ns_hl_def(ns_id as NS, hl_id, attrs, link_id, Some(&*val)) };
    Ok(())
}

/// # Safety
///
/// `opts` must point at the `KeyDict_get_ns` the dispatcher filled in, live
/// for the call.
pub unsafe fn nvim_get_hl_ns(opts: *mut KeyDict_get_ns) -> Result<Integer, Error> {
    let winid = unsafe { (*opts).winid };
    let Some(winid) = winid else {
        return Ok(ns_hl_global.get() as Integer);
    };
    let Some(win) = find_window_by_handle(winid)? else {
        return Ok(0 as Integer);
    };
    Ok(win.w_ns_hl as Integer)
}

pub fn nvim_set_hl_ns(ns_id: Integer) -> Result<(), Error> {
    let mut error = Error::none();
    if !(ns_id >= 0 as Integer) {
        error = err_bad_number(c"namespace", ns_id);
        return ().reported(error);
    }
    ns_hl_global.set(ns_id as NS);
    unsafe { hl_check_ns() };
    redraw_all_later(UPD_NOT_VALID);
    ().reported(error)
}

pub fn nvim_set_hl_ns_fast(ns_id: Integer) {
    ns_hl_fast.set(ns_id as NS);
    unsafe { hl_check_ns() };
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_get_color_by_name(name: String_0) -> Integer {
    // An API string is NUL-terminated.
    name_to_color(unsafe { ::core::ffi::CStr::from_ptr(name.data()) }).0 as Integer
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
pub unsafe fn nvim_get_color_map() -> ApiDict {
    let mut colors: ApiDict = ApiDict::with_capacity(COLOR_NAMES.len() as size_t);
    for entry in &COLOR_NAMES {
        let color = Object::integer(entry.color as Integer);
        colors.insert(entry.name, color);
    }
    colors
}
