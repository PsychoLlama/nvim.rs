//! `nvim_win_get_config()`: rendering a window's config back.
//!
//! The inverse of the parse: every field the config keyset can carry is read
//! off the `WinConfig` and packed into a Dict, including the border and its
//! title/footer -- which `config_put_bordertext` renders back as the
//! `[[text, hl], ..]` chunk arrays they were given as.

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
use crate::api::private::helpers::find_window_by_handle;
use crate::winlayer::Live;
use core::ffi::{CStr, c_char};

// The enumerated keys' spellings, each indexed by the value it names. They
// are the same literals `parse.rs` matches on the way in; upstream keeps
// them as `static const char *` tables and reads them through a pointer,
// which is what made this file's answer an unchecked one.

/// [`FloatRelative`]'s names, indexed by the value.
const FLOAT_RELATIVE_STR: [&CStr; 6] = [
    c"editor",
    c"win",
    c"cursor",
    c"mouse",
    c"tabline",
    c"laststatus",
];

/// [`WinSplit`]'s names, indexed by the value.
const WIN_SPLIT_STR: [&CStr; 4] = [c"left", c"right", c"above", c"below"];

/// [`WinStyle`]'s names, indexed by the value.
const WIN_STYLE_STR: [&CStr; 2] = [c"", c"minimal"];

/// [`FloatAnchor`]'s names, indexed by its two bits.
const FLOAT_ANCHOR_STR: [&CStr; 4] = [c"NW", c"NE", c"SW", c"SE"];

/// [`AlignTextPos`]'s names, indexed by the value.
const ALIGN_TEXT_STR: [&CStr; 3] = [c"left", c"center", c"right"];

/// Put one of the two border texts -- its chunks and its position -- into
/// `config`, as the keys `nvim_win_set_config` would take back.
fn config_put_bordertext(
    config: &mut KeyDict_win_config,
    fconfig: WinCfg,
    bordertext_type: BorderTextType,
) {
    let footer = bordertext_type == kBorderTextFooter;
    let (vt, align) = if footer {
        (fconfig.footer_chunks, fconfig.footer_pos)
    } else {
        (fconfig.title_chunks, fconfig.title_pos)
    };
    // SAFETY: the chunks are the window's own, and `arena` is the caller's.
    let bordertext = Object::array(unsafe { virt_text_to_array(vt, true) });
    let pos = String_0::from_cstr(ALIGN_TEXT_STR[align as usize]);
    if footer {
        config.footer = Some(bordertext);
        config.footer_pos = Some(pos);
    } else {
        config.title = Some(bordertext);
        config.title_pos = Some(pos);
    }
}

/// The eight border cells as the `border` key takes them: a bare string per
/// cell, or a `[char, highlight]` pair for a cell that carries one.
///
/// # Safety
/// `arena` must be the caller's, and outlive the answer along with `fconfig`.
unsafe fn border_array(fconfig: WinCfg) -> Array {
    let mut border = Array::with_capacity(8);
    for i in 0..8 {
        // SAFETY: the cell is one of the config's own eight, and holds at
        // most `MAX_SCHAR_SIZE` bytes; taking its address off the raw pointer
        // rather than off a `Deref` is what keeps `fconfig` usable after.
        let cell = unsafe {
            let chars = (&raw mut (*fconfig.raw()).border_chars).cast::<c_char>();
            cstrn_to_string(
                chars.add(i * MAX_SCHAR_SIZE as usize),
                MAX_SCHAR_SIZE as size_t,
            )
        };
        let name = syn_id2name(fconfig.border_hl_ids[i]);
        // SAFETY: `syn_id2name` answers a NUL-terminated name, empty for an
        // id with no group.
        let highlighted = unsafe { *name } != 0;
        // SAFETY: `arena` is the caller's, and both strings live as long as
        // it does.
        unsafe {
            if highlighted {
                let mut tuple = Array::with_capacity(2);
                tuple.push(Object::string(cell));
                tuple.push(Object::string(cstr_to_string(name)));
                border.push(Object::array(tuple));
            } else {
                border.push(Object::string(cell));
            }
        }
    }
    border
}

/// `win`'s configuration, as the dictionary `nvim_open_win` would take.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_win_get_config(win: WindowHandle) -> Result<KeyDict_win_config, Error> {
    let mut rv = KeyDict_win_config::default();
    let Some(wp) = find_window_by_handle(win)? else {
        return Ok(rv);
    };
    // SAFETY: `wp` names a live window, so its own config field is live with
    // it. The address comes off the raw pointer rather than off a `Deref`,
    // which is what lets both stay usable.
    let config: WinCfg = unsafe { Live::new(&raw mut (*wp.raw()).w_config) };

    rv.focusable = Some(config.focusable);
    rv.external = Some(config.external);
    rv.hide = Some(config.hide);
    rv.mouse = Some(config.mouse);
    rv.style = Some(String_0::from_cstr(WIN_STYLE_STR[config.style as usize]));

    if wp.w_floating {
        rv.width = Some(Integer::from(config.width));
        rv.height = Some(Integer::from(config.height));
        if !config.external {
            if config.relative == kFloatRelativeWindow {
                rv.win = Some(config.window);
                if config.bufpos.lnum >= 0 {
                    let mut pos = Array::with_capacity(2);
                    let (lnum, col) = (config.bufpos.lnum, config.bufpos.col);
                    // SAFETY: `pos` is the two-slot block `arena` just handed
                    // back.
                    pos.push(Object::integer(Integer::from(lnum)));
                    pos.push(Object::integer(Integer::from(col)));
                    rv.bufpos = Some(pos);
                }
            }
            let anchor = usize::try_from(config.anchor).expect("an anchor is one of four");
            rv.anchor = Some(String_0::from_cstr(FLOAT_ANCHOR_STR[anchor]));
            rv.row = Some(config.row);
            rv.col = Some(config.col);
            rv.zindex = Some(Integer::from(config.zindex));
        }
        if config.border {
            // SAFETY: `arena` is the caller's, and outlives the answer along
            // with the window's config.
            rv.border = Some(Object::array(unsafe { border_array(config) }));
            if config.title {
                config_put_bordertext(&mut rv, config, kBorderTextTitle);
            }
            if config.footer {
                config_put_bordertext(&mut rv, config, kBorderTextFooter);
            }
        } else {
            rv.border = Some(Object::string(String_0::from_cstr(c"none")));
        }
    } else if !config.external {
        rv.width = Some(Integer::from(wp.w_width));
        rv.height = Some(Integer::from(wp.w_height));
        let split = win_split_dir(wp);
        rv.split = Some(String_0::from_cstr(WIN_SPLIT_STR[split as usize]));
    }

    let rel = if wp.w_floating && !config.external {
        FLOAT_RELATIVE_STR[config.relative as usize]
    } else {
        c""
    };
    rv.relative = Some(String_0::from_cstr(rel));
    if config._cmdline_offset < INT_MAX {
        rv._cmdline_offset = Some(Integer::from(config._cmdline_offset));
    }
    Ok(rv)
}
