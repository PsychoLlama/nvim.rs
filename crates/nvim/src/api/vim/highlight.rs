//! Highlight groups, highlight namespaces and colour names.
//!
//! `nvim_set_hl` fills an `HlAttrs` from the keyset and installs it in a
//! namespace; `nvim_get_hl` renders one (or a whole namespace) back.  The
//! `*_hl_ns` trio switches which namespace the screen is drawn with -- the
//! `_fast` spelling being the one a fast callback may call -- and the two
//! colour functions are the built-in name table.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::{err_bad_number, err_bad_value};
use crate::narrow::number_as_int;

pub fn nvim_get_hl_id_by_name(name: ApiString) -> Integer {
    Integer::from(syn_check_group(name.as_bytes()))
}

pub fn nvim_get_hl(ns_id: Integer, opts: &mut KeyDict_get_highlight) -> Result<ApiDict, Error> {
    ns_get_hl_defs(Handle::new(number_as_int(ns_id)), opts)
}

pub fn nvim_set_hl(
    channel_id: uint64_t,
    ns_id: Integer,
    name: ApiString,
    val: &mut KeyDict_highlight,
) -> Result<(), Error> {
    let hl_id: ::core::ffi::c_int = syn_check_group(name.as_bytes());
    if !(hl_id != 0 as ::core::ffi::c_int) {
        return Err(err_bad_value(c"highlight name", name.as_cstr()));
    }
    let mut link_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if val.url.as_ref().is_some() {
        return Err(Error::validation(c"Invalid key: 'url'"));
    }
    let update: bool = val.update.unwrap_or(false);
    let mut base: Option<&HlAttrs> = None;
    let base_attrs: HlAttrs;
    if update && let Some(attrs) = hl_ns_get_attrs(number_as_int(ns_id), hl_id, None) {
        base_attrs = attrs;
        base = Some(&base_attrs);
    }
    let attrs: HlAttrs = dict2hlattrs(&*val, true, Some(&mut link_id), base)?;
    let _sctx = api_set_sctx(channel_id);
    ns_hl_def(
        Handle::new(number_as_int(ns_id)),
        hl_id,
        attrs,
        link_id,
        Some(&*val),
    );
    Ok(())
}

pub fn nvim_get_hl_ns(opts: &mut KeyDict_get_ns) -> Result<Integer, Error> {
    let winid = opts.winid;
    let Some(winid) = winid else {
        return Ok(Integer::from(ns_hl_global.get()));
    };
    let Some(win) = find_window_by_handle(winid)? else {
        return Ok(0 as Integer);
    };
    Ok(Integer::from(win.w_ns_hl))
}

pub fn nvim_set_hl_ns(ns_id: Integer) -> Result<(), Error> {
    if !(ns_id >= 0 as Integer) {
        return Err(err_bad_number(c"namespace", ns_id));
    }
    ns_hl_global.set(Handle::new(number_as_int(ns_id)));
    hl_check_ns();
    redraw_all_later(UPD_NOT_VALID);
    Ok(())
}

pub fn nvim_set_hl_ns_fast(ns_id: Integer) {
    ns_hl_fast.set(Handle::new(number_as_int(ns_id)));
    hl_check_ns();
}

pub fn nvim_get_color_by_name(name: ApiString) -> Integer {
    // An API string is NUL-terminated.
    Integer::from(name_to_color(name.as_cstr()).0)
}

pub fn nvim_get_color_map() -> ApiDict {
    let mut colors: ApiDict = ApiDict::with_capacity(COLOR_NAMES.len() as size_t);
    for entry in &COLOR_NAMES {
        let color = Object::integer(Integer::from(entry.color));
        colors.insert(entry.name, color);
    }
    colors
}
