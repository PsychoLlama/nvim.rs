//! Highlight groups, highlight namespaces and colour names.
//!
//! `nvim_set_hl` fills an `HlAttrs` from the keyset and installs it in a
//! namespace; `nvim_get_hl` renders one (or a whole namespace) back.  The
//! `*_hl_ns` trio switches which namespace the screen is drawn with -- the
//! `_fast` spelling being the one a fast callback may call -- and the two
//! colour functions are the built-in name table.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, dict_put_str, has_key};
use crate::api::private::validate::err_invalid_ptr;
use crate::highlight::HlAttrFlags;

pub unsafe fn nvim_get_hl_id_by_name(name: String_0) -> Integer {
    unsafe { syn_check_group(name.data(), name.len()) as Integer }
}

pub unsafe fn nvim_get_hl(
    ns_id: Integer,
    opts: *mut KeyDict_get_highlight,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    unsafe { ns_get_hl_defs(ns_id as NS, opts, arena, &mut error).reported(error) }
}

pub unsafe fn nvim_set_hl(
    channel_id: uint64_t,
    ns_id: Integer,
    name: String_0,
    val: *mut KeyDict_highlight,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let mut hl_id: ::core::ffi::c_int = unsafe { syn_check_group(name.data(), name.len()) };
    if !(hl_id != 0 as ::core::ffi::c_int) {
        let (what, got) = (c"highlight name".as_ptr(), name.data());
        // SAFETY: `error` is this frame's own slot and `name` is the caller's.
        error = unsafe { err_invalid_ptr(what, got, 0, true) };
        return ().reported(error);
    }
    let mut link_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if has_key(
        unsafe { (*val).is_set__highlight_ },
        KEYSET_OPTIDX_highlight__url,
    ) {
        error = Error::validation(c"Invalid key: 'url'");
        return ().reported(error);
    }
    let mut update: bool = has_key(
        unsafe { (*val).is_set__highlight_ },
        KEYSET_OPTIDX_highlight__update,
    ) && unsafe { (*val).update } as ::core::ffi::c_int != 0;
    let mut base: Option<&HlAttrs> = None;
    let mut base_attrs: HlAttrs = HlAttrs {
        rgb_ae_attr: HlAttrFlags::NONE,
        cterm_ae_attr: HlAttrFlags::NONE,
        rgb_fg_color: 0,
        rgb_bg_color: 0,
        rgb_sp_color: 0,
        cterm_fg_color: 0,
        cterm_bg_color: 0,
        hl_blend: 0,
        url: 0,
    };
    if update as ::core::ffi::c_int != 0
        && let Some(attrs) = unsafe { hl_ns_get_attrs(ns_id as ::core::ffi::c_int, hl_id, None) }
    {
        base_attrs = attrs;
        base = Some(&base_attrs);
    }
    let mut attrs: HlAttrs =
        unsafe { dict2hlattrs(&*val, true, Some(&mut link_id), base, &mut error) };
    if !(error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        let _sctx = api_set_sctx(channel_id);
        unsafe { ns_hl_def(ns_id as NS, hl_id, attrs, link_id, Some(&*val)) };
    }
    ().reported(error)
}

pub unsafe fn nvim_get_hl_ns(opts: *mut KeyDict_get_ns) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    if has_key(
        unsafe { (*opts).is_set__get_ns_ },
        KEYSET_OPTIDX_get_ns__winid,
    ) {
        let mut win: *mut win_T = unsafe { find_window_by_handle((*opts).winid, &mut error) };
        if win.is_null() {
            return (0 as Integer).reported(error);
        }
        (unsafe { (*win).w_ns_hl } as Integer).reported(error)
    } else {
        (ns_hl_global.get() as Integer).reported(error)
    }
}

pub unsafe fn nvim_set_hl_ns(ns_id: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    if !(ns_id >= 0 as Integer) {
        let what = c"namespace".as_ptr();
        let none = ::core::ptr::null::<::core::ffi::c_char>();
        // SAFETY: `err` is this frame's own slot and `what` is a literal.
        error = unsafe { err_invalid_ptr(what, none, ns_id, false) };
        return ().reported(error);
    }
    ns_hl_global.set(ns_id as NS);
    unsafe { hl_check_ns() };
    unsafe { redraw_all_later(UPD_NOT_VALID) };
    ().reported(error)
}

pub unsafe fn nvim_set_hl_ns_fast(ns_id: Integer) {
    ns_hl_fast.set(ns_id as NS);
    unsafe { hl_check_ns() };
}

pub unsafe fn nvim_get_color_by_name(name: String_0) -> Integer {
    // An API string is NUL-terminated.
    name_to_color(unsafe { ::core::ffi::CStr::from_ptr(name.data()) }).0 as Integer
}

pub unsafe fn nvim_get_color_map(arena: *mut Arena) -> Dict {
    let mut colors: Dict = arena_dict(arena, COLOR_NAMES.len() as size_t);
    for entry in &COLOR_NAMES {
        let name = String_0::from_cstr(entry.name);
        let color = Object::integer(entry.color as Integer);
        // SAFETY: `colors` is the arena block sized for every colour name.
        unsafe { dict_put_str(&mut colors, name, color) };
    }
    colors
}
