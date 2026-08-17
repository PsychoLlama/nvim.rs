//! Answering questions about a group from outside.
//!
//! [`ns_get_hl_defs`] is `nvim_get_hl()`, building a dictionary per group via
//! [`hlgroup2dict`]; [`highlight_has_attr`] and [`highlight_color`] are what
//! `synIDattr()` calls for one attribute at a time.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use crate::api::private::helpers::{api_set_error, arena_dict, cstr_as_string};
use crate::global_cell::GlobalCell;
use crate::highlight::dict::put;
use crate::highlight::{HL_DEFAULT, HLATTRS_DICT_SIZE, hlattrs2dict, ns_get_hl, syn_attr2entry};
use crate::types::{
    Arena, Dict, Error, KeyDict_get_highlight, KeyValuePair, NS, Object, kErrorTypeNone,
    kErrorTypeValidation, size_t,
};
use crate::ui::ui_rgb_attached;

use super::{
    HL_UNDERLINE_MASK, HexBuf, KEYSET_OPTIDX_get_highlight__create,
    KEYSET_OPTIDX_get_highlight__id, KEYSET_OPTIDX_get_highlight__link,
    KEYSET_OPTIDX_get_highlight__name, coloridx_to_name, group, highlight_num_groups,
    syn_check_group, syn_get_final_id, syn_name2id_len,
};

/// The empty dict every "nothing to say" path answers.
const NO_DICT: Dict = Dict {
    size: 0,
    capacity: 0,
    items: core::ptr::null_mut::<KeyValuePair>(),
};

/// Whether the caller set key `bit` of `Dict(get_highlight)`.
///
/// # Safety
/// `opts` is a live keydict.
unsafe fn has_key(opts: *mut KeyDict_get_highlight, bit: c_int) -> bool {
    // SAFETY: the caller's keydict.
    unsafe { (*opts).is_set__get_highlight_ & (1u64 << bit) != 0 }
}

/// Describes the group with id `hl_id`, as namespace `ns_id` sees it.
///
/// Answers false — leaving `hl` alone — for a group the namespace says
/// nothing about, and for a table entry that was created by a lookup and
/// never given settings.
///
/// # Safety
/// Reaches the group and namespace tables; `arena` is live; main thread only.
unsafe fn hlgroup2dict(hl: &mut Dict, ns_id: NS, hl_id: c_int, arena: *mut Arena) -> bool {
    let entry = group(hl_id);
    let mut ns = ns_id;
    // SAFETY: the editor's own tables.
    let link = if ns_id == 0 {
        entry.link
    } else {
        unsafe { ns_get_hl(&mut ns, hl_id, true, entry.set != 0) }
    };
    if link == -1 {
        return false;
    }
    if ns_id == 0 && entry.cleared && entry.set == 0 {
        // The table entry was created but never set.
        return false;
    }

    ns = ns_id;
    // SAFETY: as above.
    let attr = unsafe {
        syn_attr2entry(if ns_id == 0 {
            entry.attr
        } else {
            ns_get_hl(&mut ns, hl_id, false, entry.set != 0)
        })
    };

    // SAFETY: the arena hands out `HLATTRS_DICT_SIZE + 1` writable entries.
    unsafe {
        *hl = arena_dict(arena, HLATTRS_DICT_SIZE + 1);
        if attr.rgb_ae_attr & HL_DEFAULT != 0 {
            put(hl, c"default", Object::boolean(true));
        }
        if link > 0 {
            assert!(link <= highlight_num_groups(), "link out of bounds");
            put(
                hl,
                c"link",
                Object::string(cstr_as_string(group(link).name.as_ptr())),
            );
        }
        let mut cterm = arena_dict(arena, HLATTRS_DICT_SIZE);
        hlattrs2dict(hl, None, attr, true, true);
        hlattrs2dict(hl, Some(&mut cterm), attr, false, true);
        if cterm.size != 0 {
            put(hl, c"cterm", Object::dict(cterm));
        }
    }
    true
}

/// `nvim_get_hl()`: one group's definition, or every group's.
///
/// `name`/`id` pick a single group — `name` with `create` set adds it if it
/// does not exist — and `link` chooses between reporting the link and
/// following it.
///
/// # Safety
/// `opts`, `arena` and `err` are live; main thread only.
pub unsafe fn ns_get_hl_defs(
    ns_id: NS,
    opts: *mut KeyDict_get_highlight,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    // SAFETY: the caller's keydict, arena and error slot.
    unsafe {
        let link = !has_key(opts, KEYSET_OPTIDX_get_highlight__link) || (*opts).link;

        let mut id = -1;
        if has_key(opts, KEYSET_OPTIDX_get_highlight__name) {
            let create = !has_key(opts, KEYSET_OPTIDX_get_highlight__create) || (*opts).create;
            let (name, len) = ((*opts).name.data, (*opts).name.size);
            id = if create {
                syn_check_group(name, len)
            } else {
                syn_name2id_len(name, len)
            };
            if id == 0 && !create {
                return NO_DICT;
            }
        } else if has_key(opts, KEYSET_OPTIDX_get_highlight__id) {
            id = (*opts).id as c_int;
        }

        if id != -1 {
            if id < 1 || id > highlight_num_groups() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"%s".as_ptr(),
                    c"Highlight id out of bounds".as_ptr(),
                );
                return NO_DICT;
            }
            let mut attrs = NO_DICT;
            hlgroup2dict(
                &mut attrs,
                ns_id,
                if link { id } else { syn_get_final_id(id) },
                arena,
            );
            return attrs;
        }

        if (*err).type_0 != kErrorTypeNone {
            return NO_DICT;
        }

        let mut rv = arena_dict(arena, highlight_num_groups() as size_t);
        for id in 1..=highlight_num_groups() {
            let mut attrs = NO_DICT;
            if !hlgroup2dict(&mut attrs, ns_id, id, arena) {
                continue;
            }
            let named = if link { id } else { syn_get_final_id(id) };
            assert!(rv.size < rv.capacity, "highlight dict overflow");
            *rv.items.add(rv.size) = KeyValuePair {
                key: cstr_as_string(group(named).name.as_ptr()),
                value: Object::dict(attrs),
            };
            rv.size += 1;
        }
        rv
    }
}

/// `synIDattr({id}, {flag})` for a boolean attribute: `"1"` if the group has
/// it, NULL if not.
///
/// `modec` is `'g'` for `gui=` and anything else for `cterm=`.
pub fn highlight_has_attr(id: c_int, flag: c_int, modec: c_int) -> *const c_char {
    if id <= 0 || id > highlight_num_groups() {
        return core::ptr::null();
    }
    let entry = group(id);
    let attr = if modec == 'g' as c_int {
        entry.gui
    } else {
        entry.cterm
    };
    // The underline styles share a field, so the answer is an equality test
    // rather than a mask test.
    let has = if flag & HL_UNDERLINE_MASK != 0 {
        attr & HL_UNDERLINE_MASK == flag
    } else {
        attr & flag != 0
    };
    if has {
        c"1".as_ptr()
    } else {
        core::ptr::null()
    }
}

/// Where a `#rrggbb` or decimal answer from [`highlight_color`] is formatted.
///
/// Static because the answer is a borrowed pointer, overwritten on the next
/// call — which is what upstream documents for this function.
static ANSWER: GlobalCell<[u8; 20]> = GlobalCell::new([0; 20]);

/// `synIDattr({id}, {what})` for a colour: `"fg"`, `"bg"`, `"sp"`, any of
/// those with a `#` suffix, or `"font"`.
///
/// `modec` is `'g'` for `gui=`, `'c'` for `cterm=` and `'t'` for `term=`,
/// which has no colours at all. NULL means "nothing to report".
///
/// # Safety
/// `what` is NUL-terminated and at least four bytes readable — upstream reads
/// `what[3]` unconditionally, which its own callers guarantee. Main thread
/// only.
pub unsafe fn highlight_color(id: c_int, what: *const c_char, modec: c_int) -> *const c_char {
    if id <= 0 || id > highlight_num_groups() {
        return core::ptr::null();
    }
    // SAFETY: the caller's NUL-terminated selector.
    let what = unsafe { CStr::from_ptr(what) }.to_bytes();
    let lower = |i: usize| what.get(i).copied().unwrap_or(0).to_ascii_lowercase();

    let (mut fg, mut sp, mut font) = (false, false, false);
    if lower(0) == b'f' && lower(1) == b'g' {
        fg = true;
    } else if lower(0) == b'f' && lower(1) == b'o' && lower(2) == b'n' && lower(3) == b't' {
        font = true;
    } else if lower(0) == b's' && lower(1) == b'p' {
        sp = true;
    } else if !(lower(0) == b'b' && lower(1) == b'g') {
        return core::ptr::null();
    }

    let entry = group(id);
    if modec == 'g' as c_int {
        if what.get(2) == Some(&b'#') && ui_rgb_attached() {
            let n = if fg {
                entry.rgb_fg
            } else if sp {
                entry.rgb_sp
            } else {
                entry.rgb_bg
            };
            if !(0..=0xffffff).contains(&n) {
                return core::ptr::null();
            }
            return answer(format_args!("#{n:06x}"));
        }
        let (idx, value) = if fg {
            (entry.rgb_fg_idx, entry.rgb_fg)
        } else if sp {
            (entry.rgb_sp_idx, entry.rgb_sp)
        } else {
            (entry.rgb_bg_idx, entry.rgb_bg)
        };
        // The name goes in the same buffer as the formatted answers, so that
        // every answer this function gives has the same lifetime.
        let mut hexbuf: HexBuf = [0; 8];
        return match coloridx_to_name(idx, value, &mut hexbuf) {
            Some(name) => answer(format_args!("{}", name.to_string_lossy())),
            None => core::ptr::null(),
        };
    }

    if font || sp {
        return core::ptr::null();
    }
    if modec == 'c' as c_int {
        let n = if fg { entry.cterm_fg } else { entry.cterm_bg } - 1;
        if n < 0 {
            return core::ptr::null();
        }
        return answer(format_args!("{n}"));
    }
    // `term` has no colours.
    core::ptr::null()
}

/// Parks `text` in [`ANSWER`] and hands back a pointer to it, truncating at
/// 19 bytes as upstream's `char[20]` did.
fn answer(text: core::fmt::Arguments) -> *const c_char {
    let text = text.to_string();
    let bytes = text.as_bytes();
    ANSWER.with_mut(|buf| {
        let len = bytes.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&bytes[..len]);
        buf[len] = 0;
    });
    ANSWER.as_raw().cast()
}
