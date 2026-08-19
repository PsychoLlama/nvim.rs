#![deny(unsafe_op_in_unsafe_fn)]

//! Highlight namespaces: `nvim_set_hl(ns, …)`, `'winhighlight'`, and the
//! per-namespace `HLF_*` tables the drawing code reads through.
//!
//! A namespace is a private overlay on the global (`:highlight`) definitions.
//! Groups defined in one are looked up by `(ns_id, hl_id)` in [`NS_HLS`];
//! anything the namespace does not define falls back to the global group, and
//! a namespace whose provider has an `hl_def` callback is asked, once per
//! group, to invent one on demand.
//!
//! The drawing code cannot afford that lookup per cell, so each namespace
//! also gets a resolved table — one attribute id per `HLF_*` builtin — built
//! by [`update_ns_hl`] and cached until the provider invalidates it. Three
//! globals point into those tables: `hl_attr_active` (what `HL_ATTR` reads),
//! and each window's `w_ns_hl_attr` (what [`win_hl_attr`] reads). That is why
//! the tables are boxed and handed out as raw pointers — the pointers outlive
//! any borrow of the map, and nothing ever removes an entry.
//!
//! Which namespace is "active" is itself a small resolution ([`hl_check_ns`]):
//! a namespace forced by a fast callback wins, then the current window's,
//! then the global one.

use super::{
    HLATTRS_INIT, dict2hlattrs, get_attr_entry, hl_apply_winblend, hl_combine_attr,
    hl_get_syn_attr, kHlUI, syn_attr2entry,
};
use crate::api::private::dispatch::KeyDict_highlight_get_field;
use crate::api::private::helpers::{api_dict_to_keydict, cstr_as_string};
use crate::decoration_provider::with_decor_provider;
use crate::global_cell::GlobalCell;
use crate::highlight::HlAttrFlags;
use crate::highlight_group::{
    HLF_BORDER, HLF_COUNT, HLF_INACTIVE, HLF_NFLOAT, HLF_NONE, HLF_PNI, HLF_PST, hlf_names,
    set_hl_group, syn_check_group, syn_id2name, syn_ns_id2attr,
};
use crate::lua::executor::nlua_call_ref;
use crate::main::{
    curwin, highlight_attr, hl_attr_active, must_redraw_pum, need_highlight_changed, ns_hl_active,
    ns_hl_fast, ns_hl_global, ns_hl_win, p_pb,
};
use crate::option::check_blending;
use crate::popupmenu::pum_drawn;
use crate::types::builders::ArrayBuf;
use crate::types::{
    ColorItem, ColorKey, DecorProvider, Error, HlAttrs, HlEntry, KeyDict_highlight, KeySetLink,
    LuaRetMode, NS, Object, kErrorTypeNone, kObjectTypeDict, size_t, win_T,
};
use ::libc::strlen;
use core::ffi::{c_char, c_int};
use core::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::hash::DefaultHasher;

/// The two maps here are keyed by small integers and never iterated, so a
/// fixed-seed hasher is both enough and constructible in a `static`.
type Table<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;

/// Reads or writes one field of `ns_id`'s decoration provider, registering an
/// empty provider if there is none.
///
/// Everything in this file that touches a provider goes through here, because
/// almost everything here can run Lua — a `hl_def` callback, or the attribute
/// tables rebuilding — and Lua can register a provider, which moves the
/// provider list. A pointer held across one of those is a use-after-free;
/// upstream has exactly that shape and gets away with it only because nothing
/// has done it yet.
fn provider_field<R>(ns_id: NS, f: impl FnOnce(&mut DecorProvider) -> R) -> R {
    with_decor_provider(ns_id, true, f).expect("force registers one")
}

/// `nlua_call_ref`'s "give me the value as an Object" mode.
const kRetObject: LuaRetMode = 0;
/// The `hl_def` slot of a provider that has no callback.
const LUA_NOREF: c_int = -2;
/// `fallback`'s bit in a `Dict(highlight)`'s `is_set__highlight_` mask.
const KEY_FALLBACK: c_int = 21;

/// A group definition private to a namespace, indexed by `(ns_id, hl_id)`.
///
/// The `version` is the provider's `hl_valid` at the time it was resolved:
/// bumping that is how a provider says "ask me again".
static NS_HLS: GlobalCell<Table<ColorKey, ColorItem>> =
    GlobalCell::new(HashMap::with_hasher(BuildHasherDefault::new()));

/// The resolved `HLF_*` table per namespace. See the module docs for why the
/// entries are boxed and handed out raw.
static NS_HL_ATTR: GlobalCell<Table<c_int, NsHlTable>> =
    GlobalCell::new(HashMap::with_hasher(BuildHasherDefault::new()));

/// One namespace's builtin-group table: `HLF_COUNT` attribute ids.
///
/// Owned here, but reached through a raw pointer that escapes into
/// `hl_attr_active` and every window's `w_ns_hl_attr`. Those outlive any
/// borrow of [`NS_HL_ATTR`], which is why the storage is a separate
/// allocation rather than inline in the map. Nothing removes an entry, so the
/// address is good for the process's life — `Drop` is here for completeness
/// and never runs.
struct NsHlTable(*mut [c_int; HLF_COUNT as usize]);

impl NsHlTable {
    fn as_ptr(&self) -> *mut c_int {
        self.0.cast()
    }
}

impl Default for NsHlTable {
    fn default() -> Self {
        Self(Box::into_raw(Box::new([0; HLF_COUNT as usize])))
    }
}

impl Drop for NsHlTable {
    fn drop(&mut self) {
        // SAFETY: `self.0` came from `Box::into_raw` and is dropped once.
        drop(unsafe { Box::from_raw(self.0) });
    }
}

/// Forgets every namespace's group definitions. Only the free-all-memory
/// path reaches this; the resolved `HLF_*` tables are deliberately kept,
/// because the globals still point into them.
pub fn clear_ns_defs() {
    NS_HLS.with_mut(HashMap::clear);
}

/// Defines highlight group `hl_id` inside namespace `ns_id`.
///
/// `ns_id` 0 is not a namespace at all — it is the global `:highlight` table,
/// and `dict` (which must then be `Some`) goes straight to `set_hl_group`.
///
/// A `default` definition does not overwrite one the namespace already has,
/// which is what `nvim_set_hl`'s `default = true` means.
///
/// # Safety
/// Main thread only.
pub unsafe fn ns_hl_def(
    ns_id: NS,
    hl_id: c_int,
    attrs: HlAttrs,
    link_id: c_int,
    dict: Option<&KeyDict_highlight>,
) {
    // SAFETY: the editor's own tables.
    unsafe {
        if ns_id == 0 {
            let dict = dict.expect("the global table needs the caller's dict");
            set_hl_group(hl_id, attrs, dict, link_id);
            return;
        }
        let key = ColorKey {
            ns_id,
            syn_id: hl_id,
        };
        let is_default = attrs.rgb_ae_attr.has(HlAttrFlags::DEFAULT);
        if is_default && NS_HLS.with(|hls| hls.contains_key(&key)) {
            return;
        }
        // Registered before the lookup, as upstream does: a provider's
        // position in the list is the order its callbacks run in.
        provider_field(ns_id, |_| ());
        // A link is resolved lazily, so it has no attribute set of its own.
        let attr_id = if link_id > 0 {
            -1
        } else {
            hl_get_syn_attr(ns_id, hl_id, attrs)
        };
        let item = ColorItem {
            attr_id,
            link_id,
            // Re-resolved rather than held: `hl_get_syn_attr` can rebuild the
            // attribute tables and reach a Lua callback, which can register
            // another provider and move the list.
            version: provider_field(ns_id, |p| p.hl_valid),
            is_default,
            link_global: attrs.rgb_ae_attr.has(HlAttrFlags::GLOBAL),
        };
        NS_HLS.with_mut(|hls| hls.insert(key, item));
        provider_field(ns_id, |p| p.hl_cached = false);
    }
}

/// Resolves highlight group `hl_id` in the namespace `*ns_hl` names.
///
/// Answers -1 for "this namespace has nothing to say, use the global group".
/// Otherwise, with `link` set, the group this one links to (0 meaning "it has
/// real attributes, not a link"); without it, the attribute id.
///
/// `*ns_hl` is in/out: a negative value means "whatever is active", which is
/// filled in, and a `link_global` definition rewrites it to 0 so that the
/// caller resolves the link against the global table.
///
/// `nodefault` skips definitions the namespace only supplied as a default.
///
/// # Safety
/// Calls into Lua, which can re-enter the editor; main thread only.
pub unsafe fn ns_get_hl(ns_hl: &mut NS, hl_id: c_int, link: bool, nodefault: bool) -> c_int {
    // Guards the `hl_def` callback against asking about itself.
    static RECURSIVE: GlobalCell<c_int> = GlobalCell::new(0);

    if *ns_hl == 0 {
        // The default namespace has no provider, so stop here.
        return -1;
    }
    if *ns_hl < 0 {
        if ns_hl_active.get() <= 0 {
            return -1;
        }
        *ns_hl = ns_hl_active.get();
    }
    let ns_id = *ns_hl;
    let key = ColorKey {
        ns_id,
        syn_id: hl_id,
    };

    // SAFETY: the editor's own tables, plus a Lua callback that may re-enter.
    unsafe {
        let mut item = NS_HLS.with(|hls| hls.get(&key).copied()).unwrap_or(UNSET);
        let hl_def = provider_field(ns_id, |p| p.hl_def);
        let mut valid = item.version >= provider_field(ns_id, |p| p.hl_valid);

        if !valid && hl_def != LUA_NOREF && RECURSIVE.get() == 0 {
            let mut args = ArrayBuf::<3>::new();
            args.push(Object::integer(ns_id.into()));
            args.push(Object::string(cstr_as_string(syn_id2name(hl_id))));
            args.push(Object::boolean(link));

            let mut err = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut(),
            };
            RECURSIVE.set(RECURSIVE.get() + 1);
            let name = c"hl_def".as_ptr();
            let ret = nlua_call_ref(
                hl_def,
                name,
                args.array(),
                kRetObject,
                ::core::ptr::null_mut(),
                &raw mut err,
            );
            RECURSIVE.set(RECURSIVE.get() - 1);

            // Anything but a dict means the callback declined; fall back.
            let mut fallback = true;
            // A `fallback` key the callback set explicitly also means "this
            // answer is provisional", recorded as a version one behind the
            // provider's so the next lookup asks again.
            let mut provisional = false;
            let mut attrs = HLATTRS_INIT;
            if ret.type_0 == kObjectTypeDict {
                fallback = false;
                let mut dict = KeyDict_highlight::default();
                let field = Some(
                    KeyDict_highlight_get_field
                        as unsafe fn(*const c_char, size_t) -> *mut KeySetLink,
                );
                let target = (&raw mut dict).cast();
                if api_dict_to_keydict(target, field, ret.data.dict, &raw mut err) {
                    let link_id = &mut item.link_id;
                    attrs = dict2hlattrs(&dict, true, Some(link_id), None, &raw mut err);
                    let asked = dict.is_set__highlight_ & (1 << KEY_FALLBACK) != 0;
                    fallback = !asked || dict.fallback;
                    provisional = dict.fallback;
                    if item.link_id >= 0 {
                        fallback = true;
                    }
                }
            }

            item.attr_id = if fallback {
                -1
            } else {
                hl_get_syn_attr(ns_id, hl_id, attrs)
            };
            // The callback and `hl_get_syn_attr` both pump, so the provider
            // is resolved again rather than read through a stale pointer.
            item.version = provider_field(ns_id, |p| p.hl_valid) - c_int::from(provisional);
            item.is_default = attrs.rgb_ae_attr.has(HlAttrFlags::DEFAULT);
            item.link_global = attrs.rgb_ae_attr.has(HlAttrFlags::GLOBAL);
            NS_HLS.with_mut(|hls| hls.insert(key, item));
            valid = true;
        }

        if (item.is_default && nodefault) || !valid {
            return -1;
        }
        if !link {
            return item.attr_id;
        }
        if item.attr_id >= 0 {
            // Real attributes, so there is no link to follow.
            return 0;
        }
        if item.link_global {
            *ns_hl = 0;
        }
        item.link_id
    }
}

/// What a group the namespace has never been asked about reads as.
const UNSET: ColorItem = ColorItem {
    attr_id: -1,
    link_id: -1,
    version: -1,
    is_default: false,
    link_global: false,
};

/// Re-resolves which namespace is active and points `hl_attr_active` at its
/// table. Answers whether it changed, which is the caller's cue to redraw.
///
/// # Safety
/// Reaches the namespace tables and the decoration providers; main thread.
pub unsafe fn hl_check_ns() -> bool {
    let ns = if ns_hl_fast.get() > 0 {
        ns_hl_fast.get()
    } else if ns_hl_win.get() >= 0 {
        ns_hl_win.get()
    } else {
        ns_hl_global.get()
    };
    if ns_hl_active.get() == ns {
        return false;
    }

    ns_hl_active.set(ns);
    hl_attr_active.set(highlight_attr.ptr().cast::<c_int>());
    if ns > 0 {
        // SAFETY: the editor's own tables.
        unsafe { update_ns_hl(ns) };
        let table = NS_HL_ATTR.with(|tables| tables.get(&ns).map(NsHlTable::as_ptr));
        if let Some(table) = table {
            hl_attr_active.set(table);
        }
    }
    need_highlight_changed.set(true);
    true
}

/// [`hl_check_ns`] for the window about to be drawn, or for the global
/// elements when there is none.
///
/// # Safety
/// `wp` is null or a live window; main thread only.
pub unsafe fn win_check_ns_hl(wp: *mut win_T) -> bool {
    // SAFETY: the caller's window.
    unsafe {
        ns_hl_win.set(if wp.is_null() { -1 } else { (*wp).w_ns_hl });
        hl_check_ns()
    }
}

/// The attributes of highlight group `hl_id` as namespace `ns_id` sees it,
/// or `None` when the group has none.
///
/// `optional` is in/out and tracks whether the group was *explicitly* defined
/// in the namespace: it goes in as the caller's expectation and comes back as
/// what `syn_ns_id2attr` found.
///
/// # Safety
/// Reaches the group tables; main thread only.
pub unsafe fn hl_ns_get_attrs(
    ns_id: c_int,
    hl_id: c_int,
    optional: Option<&mut bool>,
) -> Option<HlAttrs> {
    let mut opt = optional.as_deref().copied().unwrap_or(true);
    // SAFETY: the editor's own tables.
    let syn_attr = unsafe { syn_ns_id2attr(ns_id, hl_id, &mut opt) };
    if let Some(optional) = optional {
        *optional = opt;
    }
    if syn_attr <= 0 {
        return None;
    }
    Some(syn_attr2entry(syn_attr))
}

/// The attribute id for builtin group `idx` in namespace `ns_id`.
///
/// `final_id` is the group `hi link` and `'winhighlight'` have already
/// resolved `idx` to. `optional` says an undefined group should answer 0
/// rather than an entry with no attributes — the distinction matters for
/// `NormalNC` and `NormalFloat`, where "not defined" and "defined as
/// nothing" mean different things to the drawing code.
///
/// # Safety
/// Reaches the group tables and the popup menu; main thread only.
pub unsafe fn hl_get_ui_attr(ns_id: c_int, idx: c_int, final_id: c_int, optional: bool) -> c_int {
    let mut attrs = HLATTRS_INIT;
    let mut optional = optional;
    let mut available = false;
    // SAFETY: the editor's own tables.
    unsafe {
        if final_id > 0 {
            if let Some(found) = hl_ns_get_attrs(ns_id, final_id, Some(&mut optional)) {
                attrs = found;
                available = true;
            }
        }

        // The popup menu's own groups pick up 'pumblend' unless the group
        // set a blend itself.
        if HLF_PNI <= idx && idx <= HLF_PST {
            if attrs.hl_blend == -1 && p_pb.get() > 0 {
                attrs.hl_blend = p_pb.get() as c_int;
            }
            if pum_drawn() {
                must_redraw_pum.set(true);
            }
        }

        if optional && !available {
            return 0;
        }
        get_attr_entry(HlEntry {
            attr: attrs,
            kind: kHlUI,
            id1: idx,
            id2: final_id,
        })
    }
}

/// Brings `wp`'s cached highlight state up to date: which namespace table it
/// reads through, its `Normal`/`NormalNC` attributes, and its border.
///
/// `invalid` forces the work even when the window has not asked for it.
///
/// # Safety
/// `wp` is a live window; main thread only.
pub unsafe fn update_window_hl(wp: *mut win_T, invalid: bool) {
    // SAFETY: the caller's window and the editor's own tables.
    unsafe {
        let ns_id = (*wp).w_ns_hl;
        update_ns_hl(ns_id);
        if ns_id != (*wp).w_ns_hl_active || (*wp).w_ns_hl_attr.is_null() {
            (*wp).w_ns_hl_active = ns_id;
            let table = NS_HL_ATTR.with(|tables| tables.get(&ns_id).map(NsHlTable::as_ptr));
            // No namespace table: read the global one.
            (*wp).w_ns_hl_attr = table.unwrap_or_else(|| highlight_attr.ptr().cast::<c_int>());
        }
        let hl_def = (*wp).w_ns_hl_attr;

        if (*wp).w_hl_needs_update == 0 && !invalid {
            return;
        }
        (*wp).w_hl_needs_update = 0;

        // A blending float always has a *named* normal group, because
        // `NormalFloat` always is one.
        let float_win = (*wp).w_floating && !(*wp).w_config.external;
        (*wp).w_hl_attr_normal = if float_win && *hl_def.add(HLF_NFLOAT as usize) != 0 && ns_id > 0
        {
            *hl_def.add(HLF_NFLOAT as usize)
        } else if *hl_def.add(HLF_NONE as usize) > 0 {
            *hl_def.add(HLF_NONE as usize)
        } else if float_win {
            let active = *hl_attr_active.get().add(HLF_NFLOAT as usize);
            if active > 0 {
                active
            } else {
                (*highlight_attr.ptr())[HLF_NFLOAT as usize]
            }
        } else {
            0
        };
        if (*wp).w_floating {
            let winbl = (*wp).w_onebuf_opt.wo_winbl as c_int;
            (*wp).w_hl_attr_normal = hl_apply_winblend(winbl, (*wp).w_hl_attr_normal);
        }

        (*wp).w_config.shadow = false;
        if (*wp).w_floating && (*wp).w_config.border {
            let winbl = (*wp).w_onebuf_opt.wo_winbl as c_int;
            for i in 0..8 {
                let id = (*wp).w_config.border_hl_ids[i];
                let mut attr = if id != 0 {
                    hl_get_ui_attr(ns_id, HLF_BORDER, id, false)
                } else {
                    *hl_def.add(HLF_BORDER as usize)
                };
                attr = hl_apply_winblend(winbl, attr);
                if syn_attr2entry(attr).hl_blend > 0 {
                    (*wp).w_config.shadow = true;
                }
                (*wp).w_config.border_attr[i] = attr;
            }
        }

        // A shadow is itself a reason to blend.
        check_blending(wp);

        // TODO(bfredl): this a bit ad-hoc. move it from highlight ns logic
        // to 'winhl' implementation?
        let inactive = *hl_def.add(HLF_INACTIVE as usize);
        (*wp).w_hl_attr_normalnc = if inactive == 0 {
            let global = *hl_attr_active.get().add(HLF_INACTIVE as usize);
            hl_combine_attr(global, (*wp).w_hl_attr_normal)
        } else {
            inactive
        };
        if (*wp).w_floating {
            let winbl = (*wp).w_onebuf_opt.wo_winbl as c_int;
            (*wp).w_hl_attr_normalnc = hl_apply_winblend(winbl, (*wp).w_hl_attr_normalnc);
        }
    }
}

/// Rebuilds namespace `ns_id`'s builtin-group table, unless its provider says
/// the cached one still stands.
///
/// # Safety
/// Reaches the group tables, and through `hl_get_ui_attr` the Lua callbacks;
/// main thread only.
pub unsafe fn update_ns_hl(ns_id: c_int) {
    if ns_id <= 0 {
        return;
    }
    // SAFETY: the editor's own tables.
    unsafe {
        if provider_field(ns_id, |p| p.hl_cached) {
            return;
        }

        // The pointer, not a borrow: resolving a group below can re-enter and
        // insert into the map, and the table's own allocation does not move.
        let table = NS_HL_ATTR.with_mut(|tables| tables.entry(ns_id).or_default().as_ptr());
        let names = hlf_names.ptr().cast::<*const c_char>();
        for hlf in 1..HLF_COUNT {
            let name = *names.add(hlf as usize);
            let id = syn_check_group(name, strlen(name));
            // These two are the groups where "undefined" is meaningful.
            let optional = hlf == HLF_INACTIVE || hlf == HLF_NFLOAT;
            *table.add(hlf as usize) = hl_get_ui_attr(ns_id, hlf, id, optional);
        }

        // NOOOO! You cannot just pretend that "Normal" is just like any other
        // syntax group! It needs at least 10 layers of special casing! Noooooo!
        //
        // haha, tema engine go brrr
        let normality = syn_check_group(c"Normal".as_ptr(), 6);
        *table.add(HLF_NONE as usize) = hl_get_ui_attr(ns_id, -1, normality, true);

        // hl_get_ui_attr might have invalidated the decor provider.
        provider_field(ns_id, |p| p.hl_cached = true);
    }
}

/// The attribute a window's background cells are drawn with.
///
/// # Safety
/// `wp` is a live window; main thread only.
pub unsafe fn win_bg_attr(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's window and the active namespace table.
    unsafe {
        // A fast callback's namespace overrides the window's own cache.
        if ns_hl_fast.get() < 0 {
            let local = if wp == curwin.get() {
                (*wp).w_hl_attr_normal
            } else {
                (*wp).w_hl_attr_normalnc
            };
            if local != 0 {
                return local;
            }
        }
        let inactive = *hl_attr_active.get().add(HLF_INACTIVE as usize);
        if wp == curwin.get() || inactive == 0 {
            *hl_attr_active.get().add(HLF_NONE as usize)
        } else {
            inactive
        }
    }
}

/// The attribute the window resolves builtin highlight group `hlf` to: its
/// own namespace's table when one is active, otherwise the global one.
///
/// # Safety
/// `wp` is a live window; main thread only.
#[inline]
pub unsafe fn win_hl_attr(wp: *mut win_T, hlf: c_int) -> c_int {
    // SAFETY: the caller's window. `w_ns_hl_attr` may still be null if
    // highlights are checked before the first redraw.
    unsafe {
        let table = if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast.get() < 0 {
            (*wp).w_ns_hl_attr
        } else {
            hl_attr_active.get()
        };
        *table.add(hlf as usize)
    }
}
