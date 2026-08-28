//! `nvim_win_get_config()`: rendering a window's config back.
//!
//! The inverse of the parse: every field the config keyset can carry is read
//! off the `WinConfig` and packed into a Dict, including the border and its
//! title/footer -- which `config_put_bordertext` renders back as the
//! `[[text, hl], ..]` chunk arrays they were given as.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add, set_key, window_by_handle};
use crate::winlayer::Live;
use core::ffi::{CStr, c_char, c_int};

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

/// Mark `key` present in `config`'s optional-key set.
fn set(config: &mut KeyDict_win_config, key: c_int) {
    config.is_set__win_config_ = set_key(config.is_set__win_config_, key);
}

/// Put one of the two border texts -- its chunks and its position -- into
/// `config`, as the keys `nvim_win_set_config` would take back.
fn config_put_bordertext(
    config: &mut KeyDict_win_config,
    fconfig: WinCfg,
    bordertext_type: BorderTextType,
    arena: *mut Arena,
) {
    let footer = bordertext_type == kBorderTextFooter;
    let (vt, align) = if footer {
        (fconfig.footer_chunks, fconfig.footer_pos)
    } else {
        (fconfig.title_chunks, fconfig.title_pos)
    };
    // SAFETY: the chunks are the window's own, and `arena` is the caller's.
    let bordertext = Object::array(unsafe { virt_text_to_array(vt, true, arena) });
    let pos = String_0::from_cstr(ALIGN_TEXT_STR[align as usize]);
    let (text_key, pos_key) = if footer {
        (
            KEYSET_OPTIDX_win_config__footer,
            KEYSET_OPTIDX_win_config__footer_pos,
        )
    } else {
        (
            KEYSET_OPTIDX_win_config__title,
            KEYSET_OPTIDX_win_config__title_pos,
        )
    };
    config.is_set__win_config_ = set_key(config.is_set__win_config_, text_key);
    config.is_set__win_config_ = set_key(config.is_set__win_config_, pos_key);
    if footer {
        config.footer = bordertext;
        config.footer_pos = pos;
    } else {
        config.title = bordertext;
        config.title_pos = pos;
    }
}

/// The eight border cells as the `border` key takes them: a bare string per
/// cell, or a `[char, highlight]` pair for a cell that carries one.
///
/// # Safety
/// `arena` must be the caller's, and outlive the answer along with `fconfig`.
unsafe fn border_array(fconfig: WinCfg, arena: *mut Arena) -> Array {
    let mut border = arena_array(arena, 8);
    for i in 0..8 {
        // SAFETY: the cell is one of the config's own eight, and holds at
        // most `MAX_SCHAR_SIZE` bytes; taking its address off the raw pointer
        // rather than off a `Deref` is what keeps `fconfig` usable after.
        let cell = unsafe {
            let chars = (&raw mut (*fconfig.raw()).border_chars).cast::<c_char>();
            cstrn_as_string(
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
                let mut tuple = arena_array(arena, 2);
                array_add(&mut tuple, Object::string(cell));
                array_add(&mut tuple, Object::string(cstr_as_string(name)));
                array_add(&mut border, Object::array(tuple));
            } else {
                array_add(&mut border, Object::string(cell));
            }
        }
    }
    border
}

/// `win`'s configuration, as the dictionary `nvim_open_win` would take.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_win_get_config(
    win: Window,
    arena: *mut Arena,
) -> Result<KeyDict_win_config, Error> {
    let mut error = ERROR_INIT;
    let mut rv: KeyDict_win_config = KEYDICT_INIT;
    let Some(wp) = window_by_handle(win, &mut error) else {
        return rv.reported(error);
    };
    // SAFETY: `wp` names a live window, so its own config field is live with
    // it. The address comes off the raw pointer rather than off a `Deref`,
    // which is what lets both stay usable.
    let config: WinCfg = unsafe { Live::new(&raw mut (*wp.raw()).w_config) };

    set(&mut rv, KEYSET_OPTIDX_win_config__focusable);
    rv.focusable = config.focusable;
    set(&mut rv, KEYSET_OPTIDX_win_config__external);
    rv.external = config.external;
    set(&mut rv, KEYSET_OPTIDX_win_config__hide);
    rv.hide = config.hide;
    set(&mut rv, KEYSET_OPTIDX_win_config__mouse);
    rv.mouse = config.mouse;
    set(&mut rv, KEYSET_OPTIDX_win_config__style);
    rv.style = String_0::from_cstr(WIN_STYLE_STR[config.style as usize]);

    if wp.w_floating {
        set(&mut rv, KEYSET_OPTIDX_win_config__width);
        rv.width = Integer::from(config.width);
        set(&mut rv, KEYSET_OPTIDX_win_config__height);
        rv.height = Integer::from(config.height);
        if !config.external {
            if config.relative == kFloatRelativeWindow {
                set(&mut rv, KEYSET_OPTIDX_win_config__win);
                rv.win = config.window;
                if config.bufpos.lnum >= 0 {
                    let mut pos = arena_array(arena, 2);
                    let (lnum, col) = (config.bufpos.lnum, config.bufpos.col);
                    // SAFETY: `pos` is the two-slot block `arena` just handed
                    // back.
                    unsafe {
                        array_add(&mut pos, Object::integer(Integer::from(lnum)));
                        array_add(&mut pos, Object::integer(Integer::from(col)));
                    }
                    set(&mut rv, KEYSET_OPTIDX_win_config__bufpos);
                    rv.bufpos = pos;
                }
            }
            set(&mut rv, KEYSET_OPTIDX_win_config__anchor);
            rv.anchor = String_0::from_cstr(FLOAT_ANCHOR_STR[config.anchor as usize]);
            set(&mut rv, KEYSET_OPTIDX_win_config__row);
            rv.row = config.row;
            set(&mut rv, KEYSET_OPTIDX_win_config__col);
            rv.col = config.col;
            set(&mut rv, KEYSET_OPTIDX_win_config__zindex);
            rv.zindex = Integer::from(config.zindex);
        }
        set(&mut rv, KEYSET_OPTIDX_win_config__border);
        if config.border {
            // SAFETY: `arena` is the caller's, and outlives the answer along
            // with the window's config.
            rv.border = Object::array(unsafe { border_array(config, arena) });
            if config.title {
                config_put_bordertext(&mut rv, config, kBorderTextTitle, arena);
            }
            if config.footer {
                config_put_bordertext(&mut rv, config, kBorderTextFooter, arena);
            }
        } else {
            rv.border = Object::string(String_0::from_cstr(c"none"));
        }
    } else if !config.external {
        set(&mut rv, KEYSET_OPTIDX_win_config__width);
        rv.width = Integer::from(wp.w_width);
        set(&mut rv, KEYSET_OPTIDX_win_config__height);
        rv.height = Integer::from(wp.w_height);
        let split = win_split_dir(wp);
        set(&mut rv, KEYSET_OPTIDX_win_config__split);
        rv.split = String_0::from_cstr(WIN_SPLIT_STR[split as usize]);
    }

    let rel = if wp.w_floating && !config.external {
        FLOAT_RELATIVE_STR[config.relative as usize]
    } else {
        c""
    };
    set(&mut rv, KEYSET_OPTIDX_win_config__relative);
    rv.relative = String_0::from_cstr(rel);
    if config._cmdline_offset < INT_MAX {
        set(&mut rv, KEYSET_OPTIDX_win_config___cmdline_offset);
        rv._cmdline_offset = Integer::from(config._cmdline_offset);
    }
    rv.reported(error)
}
