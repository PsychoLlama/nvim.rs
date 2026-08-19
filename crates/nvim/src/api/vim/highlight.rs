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
use crate::highlight::HlAttrFlags;

pub unsafe fn nvim_get_hl_id_by_name(name: String_0) -> Integer {
    unsafe {
        return syn_check_group(name.data(), name.len()) as Integer;
    }
}

pub unsafe fn nvim_get_hl(
    ns_id: Integer,
    opts: *mut KeyDict_get_highlight,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        return ns_get_hl_defs(ns_id as NS, opts, arena, err).reported(error);
    }
}

pub unsafe fn nvim_set_hl(
    channel_id: uint64_t,
    ns_id: Integer,
    name: String_0,
    val: *mut KeyDict_highlight,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut hl_id: ::core::ffi::c_int = syn_check_group(name.data(), name.len());
        if !(hl_id != 0 as ::core::ffi::c_int) {
            api_err_invalid(
                err,
                c"highlight name".as_ptr(),
                name.data(),
                0 as int64_t,
                true,
            );
            return ().reported(error);
        }
        let mut link_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if has_key((*val).is_set__highlight_, KEYSET_OPTIDX_highlight__url) {
            api_set_error(err, kErrorTypeValidation, c"Invalid key: 'url'".as_ptr());
            return ().reported(error);
        }
        let mut update: bool = has_key((*val).is_set__highlight_, KEYSET_OPTIDX_highlight__update)
            && (*val).update as ::core::ffi::c_int != 0;
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
        if update as ::core::ffi::c_int != 0 {
            if let Some(attrs) = hl_ns_get_attrs(ns_id as ::core::ffi::c_int, hl_id, None) {
                base_attrs = attrs;
                base = Some(&base_attrs);
            }
        }
        let mut attrs: HlAttrs = dict2hlattrs(&*val, true, Some(&mut link_id), base, err);
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            let save_current_sctx: sctx_T = api_set_sctx(channel_id);
            ns_hl_def(ns_id as NS, hl_id, attrs, link_id, Some(&*val));
            current_sctx.set(save_current_sctx);
        }
    }
    ().reported(error)
}

pub unsafe fn nvim_get_hl_ns(opts: *mut KeyDict_get_ns) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        if has_key((*opts).is_set__get_ns_, KEYSET_OPTIDX_get_ns__winid) {
            let mut win: *mut win_T = find_window_by_handle((*opts).winid, err);
            if win.is_null() {
                return (0 as Integer).reported(error);
            }
            return ((*win).w_ns_hl as Integer).reported(error);
        } else {
            return (ns_hl_global.get() as Integer).reported(error);
        };
    }
}

pub unsafe fn nvim_set_hl_ns(ns_id: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        if !(ns_id >= 0 as Integer) {
            api_err_invalid(
                err,
                c"namespace".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            );
            return ().reported(error);
        }
        ns_hl_global.set(ns_id as NS);
        hl_check_ns();
        redraw_all_later(UPD_NOT_VALID);
    }
    ().reported(error)
}

pub unsafe fn nvim_set_hl_ns_fast(ns_id: Integer) {
    unsafe {
        ns_hl_fast.set(ns_id as NS);
        hl_check_ns();
    }
}

pub unsafe fn nvim_get_color_by_name(name: String_0) -> Integer {
    unsafe {
        // An API string is NUL-terminated.
        return name_to_color(::core::ffi::CStr::from_ptr(name.data())).0 as Integer;
    }
}

pub unsafe fn nvim_get_color_map(arena: *mut Arena) -> Dict {
    unsafe {
        let mut colors: Dict = arena_dict(arena, COLOR_NAMES.len() as size_t);
        for entry in &COLOR_NAMES {
            dict_put_str(
                &mut colors,
                cstr_as_string(entry.name.as_ptr()),
                Object::integer(entry.color as Integer),
            );
        }
        return colors;
    }
}
