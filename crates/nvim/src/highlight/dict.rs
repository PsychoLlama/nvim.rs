#![deny(unsafe_op_in_unsafe_fn)]

//! Highlight attributes as API values.
//!
//! Two conversions, roughly inverse. [`hlattrs2dict`] writes an [`HlAttrs`]
//! out as the `{ bold = true, fg = 0xff0000, … }` shape `nvim_get_hl` and the
//! `hl_attr_define` UI event carry; [`dict2hlattrs`] reads that shape back
//! from a parsed `Dict(highlight)` keyset.
//!
//! The asymmetry between them is upstream's and deliberate. Writing splits by
//! *storage*: one `HlAttrs` holds both a `gui` and a `cterm` definition, and
//! `use_rgb` picks which one is being described. Reading splits by *key*:
//! `fg` and `ctermfg` are separate keys of one dict, a `cterm` sub-dict may
//! override the attribute bits wholesale, and "the caller said `bold =
//! false`" has to be told from "the caller said nothing" — which is what the
//! keyset's `is_set__highlight_` mask is for.

use super::{HLATTRS_INIT, attr_entry_count, syn_attr2entry};
use crate::api::private::dispatch::KeyDict_highlight_cterm_get_field;
use crate::api::private::helpers::{api_dict_to_keydict, api_set_error, arena_dict};
use crate::api::private::validate::{api_err_exp, api_err_invalid};
use crate::highlight::HlAttrFlags;
use crate::highlight_group::{name_to_color, name_to_ctermcolor};
use crate::types::builders::static_cstring;
use crate::types::{
    Arena, Boolean, Dict, Error, HlAttrs, Integer, KeyDict_highlight, KeyDict_highlight_cterm,
    KeySetLink, KeyValuePair, Object, int16_t, int32_t, kErrorTypeException, kErrorTypeNone,
    kErrorTypeValidation, kObjectTypeInteger, kObjectTypeString, size_t,
};
use ::libc::strcasecmp;
use core::ffi::{CStr, c_int};

/// Most entries [`hlattrs2dict`] can write, and so the capacity every caller
/// must hand it. Fourteen attribute bits, three RGB colours or two cterm
/// ones, the two `*_indexed` flags and `blend`.
pub const HLATTRS_DICT_SIZE: size_t = 24;

/// Bit positions in `KeyDict_highlight::is_set__highlight_`, which apigen
/// numbers from the field order in `types::keysets`.
mod key {
    use core::ffi::c_int;

    pub const BG: c_int = 1;
    pub const FG: c_int = 2;
    pub const SP: c_int = 3;
    pub const DIM: c_int = 4;
    pub const BOLD: c_int = 6;
    pub const LINK: c_int = 7;
    pub const BLEND: c_int = 8;
    pub const BLINK: c_int = 10;
    pub const CTERM: c_int = 11;
    pub const ITALIC: c_int = 12;
    pub const REVERSE: c_int = 14;
    pub const DEFAULT: c_int = 15;
    pub const ALTFONT: c_int = 16;
    pub const CONCEAL: c_int = 17;
    pub const SPECIAL: c_int = 18;
    pub const CTERMFG: c_int = 19;
    pub const CTERMBG: c_int = 20;
    pub const OVERLINE: c_int = 22;
    pub const STANDOUT: c_int = 23;
    pub const NOCOMBINE: c_int = 24;
    pub const UNDERCURL: c_int = 25;
    pub const UNDERLINE: c_int = 26;
    pub const BACKGROUND: c_int = 27;
    pub const BG_INDEXED: c_int = 28;
    pub const FOREGROUND: c_int = 29;
    pub const FG_INDEXED: c_int = 30;
    pub const LINK_GLOBAL: c_int = 31;
    pub const UNDERDASHED: c_int = 32;
    pub const UNDERDOTTED: c_int = 33;
    pub const UNDERDOUBLE: c_int = 34;
    pub const STRIKETHROUGH: c_int = 35;
}

/// Did the caller name this key? Optional keyset fields record that, which is
/// the only way to tell `bold = false` from an absent `bold`.
fn is_set(dict: &KeyDict_highlight, opt_index: c_int) -> bool {
    dict.is_set__highlight_ & (1 << opt_index) != 0
}

/// Appends `key: value` to a dict built in storage the caller allocated.
///
/// Upstream's `PUT_C` writes past the end rather than checking; the callers
/// here all size their storage at [`HLATTRS_DICT_SIZE`], so a full dict is a
/// bug in this file and worth a panic instead of a heap overwrite.
///
/// # Safety
/// `dict.items` must point at `dict.capacity` writable entries.
pub(crate) unsafe fn put(dict: &mut Dict, key: &'static CStr, value: Object) {
    assert!(dict.size < dict.capacity, "highlight dict overflow");
    // SAFETY: the assert above kept the index inside the caller's storage.
    unsafe {
        *dict.items.add(dict.size) = KeyValuePair {
            key: static_cstring(key),
            value,
        };
    }
    dict.size += 1;
}

/// Gets the highlight description of attribute id `attr_id` as a dict.
///
/// Answers an empty dict for id 0 (which is "no attributes at all"), and sets
/// `err` for an id no [`get_attr_entry`](super::get_attr_entry) ever handed
/// out.
///
/// # Safety
/// `arena` is null or a live arena; `err` points at a live [`Error`].
pub unsafe fn hl_get_attr_by_id(
    attr_id: Integer,
    rgb: Boolean,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    let empty = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
    };
    if attr_id == 0 {
        return empty;
    }
    // SAFETY: the caller's arena and error slot.
    unsafe {
        if attr_id < 0 || attr_id >= Integer::from(attr_entry_count()) {
            let fmt = c"Invalid attribute id: %ld".as_ptr();
            api_set_error(err, kErrorTypeException, fmt, attr_id);
            return empty;
        }
        let mut retval = arena_dict(arena, HLATTRS_DICT_SIZE);
        let attrs = syn_attr2entry(attr_id as c_int);
        hlattrs2dict(&mut retval, None, attrs, rgb, false);
        retval
    }
}

/// Writes `ae` out as a dict.
///
/// The attribute *bits* go to `hl_attrs` when one is given and to `hl`
/// otherwise — `nvim_get_hl` passes a second dict so that the cterm bits land
/// in a `cterm = {…}` sub-dict while the cterm colours stay at the top level.
///
/// `use_rgb` picks which half of `ae` is being described. `short_keys` picks
/// the `nvim_get_hl` spelling (`fg`/`bg`/`sp`, and `ctermfg`/`ctermbg` for the
/// cterm half) over the UI event's (`foreground`/`background`/`special`).
///
/// # Safety
/// Both dicts must have room for [`HLATTRS_DICT_SIZE`] entries.
pub unsafe fn hlattrs2dict(
    hl: &mut Dict,
    hl_attrs: Option<&mut Dict>,
    ae: HlAttrs,
    use_rgb: bool,
    short_keys: bool,
) {
    assert!(
        hl.capacity >= HLATTRS_DICT_SIZE,
        "hlattrs2dict: hl too small"
    );
    let mask = if use_rgb {
        ae.rgb_ae_attr
    } else {
        ae.cterm_ae_attr
    };
    // SAFETY: both dicts have the capacity the caller promised.
    unsafe {
        match hl_attrs {
            Some(attrs) => {
                assert!(
                    attrs.capacity >= HLATTRS_DICT_SIZE,
                    "hlattrs2dict: hl_attrs too small"
                );
                put_flags(attrs, mask);
            }
            None => put_flags(hl, mask),
        }
        put_colors(hl, ae, mask, use_rgb, short_keys);
    }
}

/// The attribute bits of `mask`, one boolean key each.
///
/// # Safety
/// As [`put`].
unsafe fn put_flags(hl: &mut Dict, mask: HlAttrFlags) {
    // SAFETY: the caller's storage, sized for every key below.
    unsafe {
        let flag = |bit: HlAttrFlags| mask.has(bit);
        if flag(HlAttrFlags::INVERSE) {
            put(hl, c"reverse", Object::boolean(true));
        }
        if flag(HlAttrFlags::BOLD) {
            put(hl, c"bold", Object::boolean(true));
        }
        if flag(HlAttrFlags::ITALIC) {
            put(hl, c"italic", Object::boolean(true));
        }
        // The underline styles share one field, so at most one is reported.
        match mask.masked(HlAttrFlags::UNDERLINE_MASK) {
            HlAttrFlags::UNDERLINE => put(hl, c"underline", Object::boolean(true)),
            HlAttrFlags::UNDERCURL => put(hl, c"undercurl", Object::boolean(true)),
            HlAttrFlags::UNDERDOUBLE => put(hl, c"underdouble", Object::boolean(true)),
            HlAttrFlags::UNDERDOTTED => put(hl, c"underdotted", Object::boolean(true)),
            HlAttrFlags::UNDERDASHED => put(hl, c"underdashed", Object::boolean(true)),
            _ => {}
        }
        if flag(HlAttrFlags::STANDOUT) {
            put(hl, c"standout", Object::boolean(true));
        }
        if flag(HlAttrFlags::STRIKETHROUGH) {
            put(hl, c"strikethrough", Object::boolean(true));
        }
        if flag(HlAttrFlags::ALTFONT) {
            put(hl, c"altfont", Object::boolean(true));
        }
        if flag(HlAttrFlags::DIM) {
            put(hl, c"dim", Object::boolean(true));
        }
        if flag(HlAttrFlags::BLINK) {
            put(hl, c"blink", Object::boolean(true));
        }
        if flag(HlAttrFlags::CONCEALED) {
            put(hl, c"conceal", Object::boolean(true));
        }
        if flag(HlAttrFlags::OVERLINE) {
            put(hl, c"overline", Object::boolean(true));
        }
        if flag(HlAttrFlags::NOCOMBINE) {
            put(hl, c"nocombine", Object::boolean(true));
        }
    }
}

/// The colours of `ae`, plus `blend`.
///
/// # Safety
/// As [`put`].
unsafe fn put_colors(
    hl: &mut Dict,
    ae: HlAttrs,
    mask: HlAttrFlags,
    use_rgb: bool,
    short_keys: bool,
) {
    // SAFETY: the caller's storage, sized for every key below.
    unsafe {
        if use_rgb {
            if ae.rgb_fg_color != -1 {
                let key = if short_keys { c"fg" } else { c"foreground" };
                put(hl, key, Object::integer(Integer::from(ae.rgb_fg_color)));
            }
            if ae.rgb_bg_color != -1 {
                let key = if short_keys { c"bg" } else { c"background" };
                put(hl, key, Object::integer(Integer::from(ae.rgb_bg_color)));
            }
            if ae.rgb_sp_color != -1 {
                let key = if short_keys { c"sp" } else { c"special" };
                put(hl, key, Object::integer(Integer::from(ae.rgb_sp_color)));
            }
            if mask.has(HlAttrFlags::FG_INDEXED) {
                put(hl, c"fg_indexed", Object::boolean(true));
            }
            if mask.has(HlAttrFlags::BG_INDEXED) {
                put(hl, c"bg_indexed", Object::boolean(true));
            }
        } else {
            // Cterm colours are stored biased by one so that 0 means unset.
            if ae.cterm_fg_color != 0 {
                let key = if short_keys {
                    c"ctermfg"
                } else {
                    c"foreground"
                };
                put(
                    hl,
                    key,
                    Object::integer(Integer::from(ae.cterm_fg_color - 1)),
                );
            }
            if ae.cterm_bg_color != 0 {
                let key = if short_keys {
                    c"ctermbg"
                } else {
                    c"background"
                };
                put(
                    hl,
                    key,
                    Object::integer(Integer::from(ae.cterm_bg_color - 1)),
                );
            }
        }
        // `nvim_get_hl` reports blend once, with the gui half.
        if ae.hl_blend > -1 && (use_rgb || !short_keys) {
            put(hl, c"blend", Object::integer(Integer::from(ae.hl_blend)));
        }
    }
}

/// Upstream's `CHECK_FLAG_WITH_KEY`: set `flag` when the key is on, clear it
/// when the key is off *and* currently reads as exactly `flag`.
///
/// The underline styles share the three bits of
/// [`HlAttrFlags::UNDERLINE_MASK`], so setting one of them displaces the
/// others and clearing one clears the field — which is why the bits cleared
/// are not always the bits set.
fn apply_flag(mask: &mut HlAttrFlags, on: bool, flag: HlAttrFlags) {
    let field = if flag.has(HlAttrFlags::UNDERLINE_MASK) {
        HlAttrFlags::UNDERLINE_MASK
    } else {
        flag
    };
    if on {
        *mask = mask.without(field) | flag;
    } else if mask.masked(field) == flag {
        mask.clear(field);
    }
}

/// Upstream's `CHECK_FLAG`, for the `cterm` sub-dict: it has no "was it
/// given" mask, so an absent key is simply false and only the set direction
/// is meaningful.
fn set_flag(mask: &mut HlAttrFlags, on: bool, flag: HlAttrFlags) {
    if !on {
        return;
    }
    if flag.has(HlAttrFlags::UNDERLINE_MASK) {
        mask.clear(HlAttrFlags::UNDERLINE_MASK);
    }
    *mask |= flag;
}

/// Reads a `Dict(highlight)` back into an [`HlAttrs`].
///
/// `use_rgb` says whether this is a `gui` definition (`fg`/`bg`/`sp` name RGB
/// colours and the cterm half is filled in separately) or a cterm one
/// (`fg`/`bg` name colour numbers and there is no gui half).
///
/// `base` is the definition being amended, for `nvim_set_hl`'s partial
/// updates; without one every unnamed key reads as unset. `link_id` is where
/// a `link`/`link_global` key is reported; passing `None` makes those keys an
/// error, which is how the UI-side caller rejects them.
///
/// Answers `HLATTRS_INIT` with `err` set on the first bad value.
///
/// # Safety
/// `err` points at a live [`Error`]; the `Object` fields of `dict` must carry
/// values matching their tags.
pub unsafe fn dict2hlattrs(
    dict: &KeyDict_highlight,
    use_rgb: bool,
    link_id: Option<&mut c_int>,
    base: Option<&HlAttrs>,
    err: *mut Error,
) -> HlAttrs {
    let mut fg = base.map_or(-1, |b| b.rgb_fg_color);
    let mut bg = base.map_or(-1, |b| b.rgb_bg_color);
    let mut sp = base.map_or(-1, |b| b.rgb_sp_color);
    // The cterm colours are stored biased by one; unbias them, and let 0
    // (unset) come back as -1 like a missing key.
    let unbias = |c: int16_t| if c == 0 { -1 } else { int32_t::from(c) - 1 };
    let mut ctermfg = base.map_or(-1, |b| unbias(b.cterm_fg_color));
    let mut ctermbg = base.map_or(-1, |b| unbias(b.cterm_bg_color));
    let mut blend = base.map_or(-1, |b| b.hl_blend);
    let mut mask = base.map_or(HlAttrFlags::NONE, |b| b.rgb_ae_attr);
    let mut cterm_mask = base.map_or(HlAttrFlags::NONE, |b| b.cterm_ae_attr);
    let mut cterm_mask_provided = false;

    let mut flag = |set: bool, on: bool, bit: HlAttrFlags, mask: &mut HlAttrFlags| {
        if set {
            apply_flag(mask, on, bit);
        }
    };
    flag(
        is_set(dict, key::REVERSE),
        dict.reverse,
        HlAttrFlags::INVERSE,
        &mut mask,
    );
    flag(
        is_set(dict, key::BOLD),
        dict.bold,
        HlAttrFlags::BOLD,
        &mut mask,
    );
    flag(
        is_set(dict, key::ITALIC),
        dict.italic,
        HlAttrFlags::ITALIC,
        &mut mask,
    );
    let underlines = [
        (key::UNDERLINE, dict.underline, HlAttrFlags::UNDERLINE),
        (key::UNDERCURL, dict.undercurl, HlAttrFlags::UNDERCURL),
        (key::UNDERDOUBLE, dict.underdouble, HlAttrFlags::UNDERDOUBLE),
        (key::UNDERDOTTED, dict.underdotted, HlAttrFlags::UNDERDOTTED),
        (key::UNDERDASHED, dict.underdashed, HlAttrFlags::UNDERDASHED),
    ];
    for (opt, on, bit) in underlines {
        flag(is_set(dict, opt), on, bit, &mut mask);
    }
    flag(
        is_set(dict, key::STANDOUT),
        dict.standout,
        HlAttrFlags::STANDOUT,
        &mut mask,
    );
    let strike = is_set(dict, key::STRIKETHROUGH);
    flag(
        strike,
        dict.strikethrough,
        HlAttrFlags::STRIKETHROUGH,
        &mut mask,
    );
    flag(
        is_set(dict, key::ALTFONT),
        dict.altfont,
        HlAttrFlags::ALTFONT,
        &mut mask,
    );
    flag(
        is_set(dict, key::DIM),
        dict.dim,
        HlAttrFlags::DIM,
        &mut mask,
    );
    flag(
        is_set(dict, key::BLINK),
        dict.blink,
        HlAttrFlags::BLINK,
        &mut mask,
    );
    flag(
        is_set(dict, key::CONCEAL),
        dict.conceal,
        HlAttrFlags::CONCEALED,
        &mut mask,
    );
    flag(
        is_set(dict, key::OVERLINE),
        dict.overline,
        HlAttrFlags::OVERLINE,
        &mut mask,
    );
    // Only a gui definition can say which colours came from the palette.
    if use_rgb {
        let indexed = is_set(dict, key::FG_INDEXED);
        flag(indexed, dict.fg_indexed, HlAttrFlags::FG_INDEXED, &mut mask);
        let indexed = is_set(dict, key::BG_INDEXED);
        flag(indexed, dict.bg_indexed, HlAttrFlags::BG_INDEXED, &mut mask);
    }
    let nocombine = is_set(dict, key::NOCOMBINE);
    flag(nocombine, dict.nocombine, HlAttrFlags::NOCOMBINE, &mut mask);
    flag(
        is_set(dict, key::DEFAULT),
        dict.default_,
        HlAttrFlags::DEFAULT,
        &mut mask,
    );

    // SAFETY: the caller's error slot; each `Object` carries its own tag.
    unsafe {
        // The long spelling is the fallback for the short one, never both.
        if is_set(dict, key::FG) {
            fg = object_to_color(dict.fg, c"fg", use_rgb, err);
        } else if is_set(dict, key::FOREGROUND) {
            fg = object_to_color(dict.foreground, c"foreground", use_rgb, err);
        }
        if error_set(err) {
            return HLATTRS_INIT;
        }
        if is_set(dict, key::BG) {
            bg = object_to_color(dict.bg, c"bg", use_rgb, err);
        } else if is_set(dict, key::BACKGROUND) {
            bg = object_to_color(dict.background, c"background", use_rgb, err);
        }
        if error_set(err) {
            return HLATTRS_INIT;
        }
        // A special colour is always an RGB one: cterm has no such thing.
        if is_set(dict, key::SP) {
            sp = object_to_color(dict.sp, c"sp", true, err);
        } else if is_set(dict, key::SPECIAL) {
            sp = object_to_color(dict.special, c"special", true, err);
        }
        if error_set(err) {
            return HLATTRS_INIT;
        }

        if is_set(dict, key::BLEND) {
            let given = dict.blend;
            if !(0..=100).contains(&given) {
                api_err_invalid(err, c"blend".as_ptr(), c"out of range".as_ptr(), 0, false);
                return HLATTRS_INIT;
            }
            blend = given as int32_t;
        }

        if is_set(dict, key::LINK) || is_set(dict, key::LINK_GLOBAL) {
            let global = is_set(dict, key::LINK_GLOBAL);
            let Some(link_id) = link_id else {
                let name = if global { c"link_global" } else { c"link" };
                let fmt = c"Invalid Key: '%s'".as_ptr();
                api_set_error(err, kErrorTypeValidation, fmt, name.as_ptr());
                return HLATTRS_INIT;
            };
            if global {
                *link_id = dict.link_global as c_int;
                mask |= HlAttrFlags::GLOBAL;
            } else {
                *link_id = dict.link as c_int;
            }
        }

        // A `cterm` sub-dict replaces the cterm bits outright rather than
        // amending them: what it does not name is off.
        if is_set(dict, key::CTERM) {
            let mut cterm = KeyDict_highlight_cterm::default();
            let field = Some(
                KeyDict_highlight_cterm_get_field
                    as unsafe fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            );
            let target = (&raw mut cterm).cast();
            if !api_dict_to_keydict(target, field, dict.cterm, err) {
                return HLATTRS_INIT;
            }
            cterm_mask_provided = true;
            cterm_mask = HlAttrFlags::NONE;
            let bits = [
                (cterm.reverse, HlAttrFlags::INVERSE),
                (cterm.bold, HlAttrFlags::BOLD),
                (cterm.italic, HlAttrFlags::ITALIC),
                (cterm.underline, HlAttrFlags::UNDERLINE),
                (cterm.undercurl, HlAttrFlags::UNDERCURL),
                (cterm.underdouble, HlAttrFlags::UNDERDOUBLE),
                (cterm.underdotted, HlAttrFlags::UNDERDOTTED),
                (cterm.underdashed, HlAttrFlags::UNDERDASHED),
                (cterm.standout, HlAttrFlags::STANDOUT),
                (cterm.strikethrough, HlAttrFlags::STRIKETHROUGH),
                (cterm.altfont, HlAttrFlags::ALTFONT),
                (cterm.dim, HlAttrFlags::DIM),
                (cterm.blink, HlAttrFlags::BLINK),
                (cterm.conceal, HlAttrFlags::CONCEALED),
                (cterm.overline, HlAttrFlags::OVERLINE),
                (cterm.nocombine, HlAttrFlags::NOCOMBINE),
            ];
            for (on, bit) in bits {
                set_flag(&mut cterm_mask, on, bit);
            }
        }

        if is_set(dict, key::CTERMFG) {
            ctermfg = object_to_color(dict.ctermfg, c"ctermfg", false, err);
            if error_set(err) {
                return HLATTRS_INIT;
            }
        }
        if is_set(dict, key::CTERMBG) {
            ctermbg = object_to_color(dict.ctermbg, c"ctermbg", false, err);
            if error_set(err) {
                return HLATTRS_INIT;
            }
        }
    }

    // Re-bias a colour number for storage: 0 is "unset", so every real
    // number sits one higher.
    let bias = |c: int32_t| if c == -1 { 0 } else { (c + 1) as int16_t };
    let mut hlattrs = HLATTRS_INIT;
    if use_rgb {
        // The gui bits stand in for the cterm ones unless a `cterm` key said
        // otherwise.
        hlattrs.rgb_ae_attr = mask;
        hlattrs.rgb_bg_color = bg;
        hlattrs.rgb_fg_color = fg;
        hlattrs.rgb_sp_color = sp;
        hlattrs.hl_blend = blend;
        hlattrs.cterm_bg_color = bias(ctermbg);
        hlattrs.cterm_fg_color = bias(ctermfg);
        hlattrs.cterm_ae_attr = if cterm_mask_provided {
            cterm_mask
        } else {
            mask
        };
    } else {
        hlattrs.cterm_bg_color = bias(bg);
        hlattrs.cterm_fg_color = bias(fg);
        hlattrs.cterm_ae_attr = mask;
    }
    hlattrs
}

/// Has something already failed?
///
/// # Safety
/// `err` points at a live [`Error`].
unsafe fn error_set(err: *mut Error) -> bool {
    // SAFETY: the caller's error slot.
    unsafe { (*err).type_0 != kErrorTypeNone }
}

/// A colour key's value as a colour number: an integer verbatim, a name
/// looked up, `""`/`"NONE"` as -1 (unset).
///
/// `rgb` picks the palette the name is resolved against. `key` names the key
/// in the error message for a value that is neither a string nor an integer.
///
/// # Safety
/// `val` must carry a value matching its tag, and a string value must be
/// NUL-terminated. `err` points at a live [`Error`].
unsafe fn object_to_color(val: Object, key: &CStr, rgb: bool, err: *mut Error) -> int32_t {
    // SAFETY: the tag says which union arm is live.
    unsafe {
        if val.type_0 == kObjectTypeInteger {
            return val.data.integer as int32_t;
        }
        if val.type_0 != kObjectTypeString {
            api_err_exp(
                err,
                key.as_ptr(),
                c"String or Integer".as_ptr(),
                ::core::ptr::null(),
            );
            return 0;
        }
        let str = val.data.string;
        if str.is_empty() || strcasecmp(str.data(), c"NONE".as_ptr()) == 0 {
            return -1;
        }
        let name = CStr::from_ptr(str.data());
        let color = if rgb {
            name_to_color(name).0 as int32_t
        } else {
            name_to_ctermcolor(name)
        };
        if color < 0 {
            api_err_invalid(err, c"highlight color".as_ptr(), str.data(), 0, true);
        }
        color
    }
}
