#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

//! Highlight attributes as API values.
//!
//! Two conversions, roughly inverse. [`hlattrs2dict`] writes an [`HlAttrs`]
//! out as the `{bold = true, fg = 0xff0000, …}` shape `nvim_get_hl` and the
//! `hl_attr_define` UI event carry; [`dict2hlattrs`] reads that shape back
//! from a parsed `Dict(highlight)` keyset.
//!
//! The asymmetry between them is upstream's and deliberate. Writing splits by
//! *storage*: one `HlAttrs` holds both a `gui` and a `cterm` definition, and
//! `use_rgb` picks which one is being described. Reading splits by *key*:
//! `fg` and `ctermfg` are separate keys of one dict, a `cterm` sub-dict may
//! override the attribute bits wholesale, and "the caller said `bold =
//! false`" has to be told from "the caller said nothing" — which is the
//! whole reason a keyset's fields are `Option`s.

use super::{HLATTRS_INIT, attr_entry_count, syn_attr2entry};
use crate::api::private::dispatch::key_dict_highlight_cterm_get_field;
use crate::api::private::helpers::api_dict_to_keydict;
use crate::api::private::validate::{err_bad_value, err_expected, err_out_of_range};
use crate::api_error;
use crate::highlight::HlAttrFlags;
use crate::highlight_group::{name_to_color, name_to_ctermcolor};
use crate::message_fmt::msg_cstr;
use crate::types::String_0;
use crate::types::{
    ApiDict, Arena, Boolean, Error, FieldHashfn, HlAttrs, Integer, KeyDict_highlight,
    KeyDict_highlight_cterm, Object, int16_t, int32_t, kErrorTypeException, kErrorTypeValidation,
    size_t,
};
use ::libc::strcasecmp;
use core::ffi::{CStr, c_int};

/// Most entries [`hlattrs2dict`] can write, and so the capacity every caller
/// must hand it. Fourteen attribute bits, three RGB colours or two cterm
/// ones, the two `*_indexed` flags and `blend`.
pub const HLATTRS_DICT_SIZE: size_t = 24;

/// Appends `key: value`, with the key copied out of the literal.
///
/// Every caller reserves what it is going to write up front -- at least
/// [`HLATTRS_DICT_SIZE`], and more where it also writes keys of its own --
/// so a push that has to grow means a reservation is wrong. Growing is
/// correct rather than the heap overwrite upstream's `PUT_C` performed, so
/// that is a debug check rather than a panic.
pub(crate) fn put(dict: &mut ApiDict, key: &'static CStr, value: Object) {
    debug_assert!(dict.len() < dict.capacity(), "highlight dict overflow");
    dict.insert(String_0::from_cstr(key), value);
}

/// Gets the highlight description of attribute id `attr_id` as a dict.
///
/// Answers an empty dict for id 0 (which is "no attributes at all"), and sets
/// Refuses an id no [`get_attr_entry`](super::get_attr_entry) ever handed
/// out.
///
/// # Safety
/// `_arena` is null or a live arena. Nothing is taken from it any more: the
/// answer owns its entries.
pub unsafe fn hl_get_attr_by_id(
    attr_id: Integer,
    rgb: Boolean,
    _arena: *mut Arena,
) -> Result<ApiDict, Error> {
    let empty = ApiDict::EMPTY;
    if attr_id == 0 {
        return Ok(empty);
    }
    if attr_id < 0 || attr_id >= Integer::from(attr_entry_count()) {
        let why = api_error!(kErrorTypeException, "Invalid attribute id: {attr_id}");
        return Err(why);
    }
    let mut retval = ApiDict::with_capacity(HLATTRS_DICT_SIZE);
    let attrs = syn_attr2entry(attr_id as c_int);
    // SAFETY: the dict was just reserved for every key `hlattrs2dict` writes.
    unsafe { hlattrs2dict(&mut retval, None, attrs, rgb, false) };
    Ok(retval)
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
    hl: &mut ApiDict,
    hl_attrs: Option<&mut ApiDict>,
    ae: HlAttrs,
    use_rgb: bool,
    short_keys: bool,
) {
    assert!(
        hl.capacity() >= HLATTRS_DICT_SIZE,
        "hlattrs2dict: hl too small"
    );
    let mask = if use_rgb {
        ae.rgb_ae_attr
    } else {
        ae.cterm_ae_attr
    };
    match hl_attrs {
        Some(attrs) => {
            assert!(
                attrs.capacity() >= HLATTRS_DICT_SIZE,
                "hlattrs2dict: hl_attrs too small"
            );
            put_flags(attrs, mask);
        }
        None => put_flags(hl, mask),
    }
    put_colors(hl, ae, mask, use_rgb, short_keys);
}

/// The attribute bits of `mask`, one boolean key each.
fn put_flags(hl: &mut ApiDict, mask: HlAttrFlags) {
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

/// The colours of `ae`, plus `blend`.
fn put_colors(hl: &mut ApiDict, ae: HlAttrs, mask: HlAttrFlags, use_rgb: bool, short_keys: bool) {
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

/// Upstream's `CHECK_FLAG`, for the `cterm` sub-dict: it replaces the cterm
/// bits outright, so an absent key is simply false and only the set
/// direction is meaningful.
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
/// Refuses at the first bad value.
///
/// # Safety
/// The `Object` fields of `dict` must carry values matching their tags.
pub unsafe fn dict2hlattrs(
    dict: &KeyDict_highlight,
    use_rgb: bool,
    link_id: Option<&mut c_int>,
    base: Option<&HlAttrs>,
) -> Result<HlAttrs, Error> {
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

    // A key the caller did not name leaves its bit alone; one that is there
    // sets or clears it.
    let flag = |on: Option<bool>, bit: HlAttrFlags, mask: &mut HlAttrFlags| {
        if let Some(on) = on {
            apply_flag(mask, on, bit);
        }
    };
    let attributes = [
        (dict.reverse, HlAttrFlags::INVERSE),
        (dict.bold, HlAttrFlags::BOLD),
        (dict.italic, HlAttrFlags::ITALIC),
        (dict.underline, HlAttrFlags::UNDERLINE),
        (dict.undercurl, HlAttrFlags::UNDERCURL),
        (dict.underdouble, HlAttrFlags::UNDERDOUBLE),
        (dict.underdotted, HlAttrFlags::UNDERDOTTED),
        (dict.underdashed, HlAttrFlags::UNDERDASHED),
        (dict.standout, HlAttrFlags::STANDOUT),
        (dict.strikethrough, HlAttrFlags::STRIKETHROUGH),
        (dict.altfont, HlAttrFlags::ALTFONT),
        (dict.dim, HlAttrFlags::DIM),
        (dict.blink, HlAttrFlags::BLINK),
        (dict.conceal, HlAttrFlags::CONCEALED),
        (dict.overline, HlAttrFlags::OVERLINE),
        (dict.nocombine, HlAttrFlags::NOCOMBINE),
        (dict.default_, HlAttrFlags::DEFAULT),
    ];
    for (on, bit) in attributes {
        flag(on, bit, &mut mask);
    }
    // Only a gui definition can say which colours came from the palette.
    if use_rgb {
        flag(dict.fg_indexed, HlAttrFlags::FG_INDEXED, &mut mask);
        flag(dict.bg_indexed, HlAttrFlags::BG_INDEXED, &mut mask);
    }

    // The long spelling is the fallback for the short one, never both.
    if let Some(given) = &dict.fg {
        fg = object_to_color(given, c"fg", use_rgb)?;
    } else if let Some(given) = &dict.foreground {
        fg = object_to_color(given, c"foreground", use_rgb)?;
    }
    if let Some(given) = &dict.bg {
        bg = object_to_color(given, c"bg", use_rgb)?;
    } else if let Some(given) = &dict.background {
        bg = object_to_color(given, c"background", use_rgb)?;
    }
    // A special colour is always an RGB one: cterm has no such thing.
    if let Some(given) = &dict.sp {
        sp = object_to_color(given, c"sp", true)?;
    } else if let Some(given) = &dict.special {
        sp = object_to_color(given, c"special", true)?;
    }

    if let Some(given) = dict.blend {
        if !(0..=100).contains(&given) {
            return Err(err_out_of_range(c"blend"));
        }
        blend = given as int32_t;
    }

    if dict.link.is_some() || dict.link_global.is_some() {
        let global = dict.link_global;
        let Some(link_id) = link_id else {
            let name = if global.is_some() {
                c"link_global"
            } else {
                c"link"
            };
            let name = msg_cstr(name);
            return Err(api_error!(kErrorTypeValidation, "Invalid Key: '{name}'"));
        };
        match global {
            Some(id) => {
                *link_id = id as c_int;
                mask |= HlAttrFlags::GLOBAL;
            }
            None => *link_id = dict.link.unwrap_or(0) as c_int,
        }
    }

    // A `cterm` sub-dict replaces the cterm bits outright rather than
    // amending them: what it does not name is off.
    if let Some(given) = &dict.cterm {
        let mut cterm = KeyDict_highlight_cterm::default();
        let field: FieldHashfn = Some(key_dict_highlight_cterm_get_field);
        let target = (&raw mut cterm).cast();
        // The sub-dict is copied: `dict` is the caller's and goes on holding
        // it. It is a handful of booleans.
        // SAFETY: `field` is `KeyDict_highlight_cterm`'s own lookup, and
        // `target` is that keydict.
        unsafe { api_dict_to_keydict(target, field, given.clone()) }?;
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
            set_flag(&mut cterm_mask, on.unwrap_or(false), bit);
        }
    }

    if let Some(given) = &dict.ctermfg {
        ctermfg = object_to_color(given, c"ctermfg", false)?;
    }
    if let Some(given) = &dict.ctermbg {
        ctermbg = object_to_color(given, c"ctermbg", false)?;
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
    Ok(hlattrs)
}

/// A colour key's value as a colour number: an integer verbatim, a name
/// looked up, `""`/`"NONE"` as -1 (unset).
///
/// `rgb` picks the palette the name is resolved against. `key` names the key
/// in the error message for a value that is neither a string nor an integer.
fn object_to_color(val: &Object, key: &CStr, rgb: bool) -> Result<int32_t, Error> {
    if let Some(n) = val.as_integer() {
        return Ok(n as int32_t);
    }
    let Some(str) = val.as_string() else {
        let expected = c"String or Integer";
        return Err(err_expected(key, expected, None));
    };
    // SAFETY: an API string is NUL-terminated, so it is a C string too.
    if str.is_empty() || unsafe { strcasecmp(str.data(), c"NONE".as_ptr()) } == 0 {
        return Ok(-1);
    }
    let name = str.as_cstr();
    let color = if rgb {
        name_to_color(name).0 as int32_t
    } else {
        name_to_ctermcolor(name)
    };
    if color < 0 {
        return Err(err_bad_value(c"highlight color", name));
    }
    Ok(color)
}
