//! The events that are not sent by a sink of their own.
//!
//! Most of what a UI is told has a dedicated serializer in
//! [`events`](super::events). The rest — cmdline, popupmenu, messages,
//! tabline — reaches a UI through [`remote_ui_event`], which takes a name
//! and an already-built argument array and forwards them unchanged. Those
//! events are defined by their external `ext_` option, so the editor builds
//! the arguments once and every UI gets the same ones.
//!
//! "Unchanged" has two exceptions, both of them the old protocol showing
//! through, and both handled here:
//!
//! - a UI without `ext_linegrid` cannot resolve a highlight id, so the
//!   cmdline events' attribute ids are expanded into full attribute dicts;
//! - a UI with `ext_wildmenu` but not `ext_popupmenu` gets the popupmenu
//!   events renamed and cut down to the wildmenu shape.
//!
//! [`remote_ui_hl_attr_define`] lives here too: it is the event that
//! *builds* the attribute table those exceptions exist to work around.

#![deny(unsafe_op_in_unsafe_fn)]

use super::events::{count, linegrid, send};
use super::packer::push_call;
use crate::src::nvim::api::private::helpers::{arena_array, arena_dict, cstr_as_string};
use crate::src::nvim::highlight::{HLATTRS_DICT_SIZE, hl_get_url, hlattrs2dict, syn_attr2entry};
use crate::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::src::nvim::types::builders::{ArrayBuf, DictBuf, static_cstring};
use crate::src::nvim::types::{
    Arena, Array, Dict, HlAttrs, Integer, KeyValuePair, Object, RemoteUI,
};
use crate::src::nvim::ui::{kUIHlState, kUIPopupmenu, kUIWildmenu};
use core::ffi::CStr;

/// Announces a highlight attribute id and what it resolves to.
///
/// Both dicts are built from `rgb_attrs`; `cterm_attrs` is accepted and
/// ignored, matching upstream. The difference between them is the `use_rgb`
/// flag, which picks which half of the attribute entry is read.
///
/// # Safety
///
/// `ui` must be live and `info` valid for the duration of the call.
pub unsafe fn remote_ui_hl_attr_define(
    ui: *mut RemoteUI,
    id: Integer,
    rgb_attrs: HlAttrs,
    _cterm_attrs: HlAttrs,
    info: Array,
) {
    // The legacy protocol has no attribute table; those UIs get the whole
    // attribute set with every cell instead.
    if !unsafe { linegrid(ui) } {
        return;
    }

    // One spare entry past what `hlattrs2dict` fills, for the URL below.
    let mut rgb_buf = DictBuf::<{ HLATTRS_DICT_SIZE as usize + 1 }>::new();
    let mut cterm_buf = DictBuf::<{ HLATTRS_DICT_SIZE as usize }>::new();
    let mut rgb = rgb_buf.dict();
    let mut cterm = cterm_buf.dict();
    unsafe {
        hlattrs2dict(&raw mut rgb, core::ptr::null_mut(), rgb_attrs, true, false);
        hlattrs2dict(
            &raw mut cterm,
            core::ptr::null_mut(),
            rgb_attrs,
            false,
            false,
        );
        if rgb_attrs.url >= 0 {
            let url = hl_get_url(rgb_attrs.url as u32);
            *rgb.items.add(rgb.size) = KeyValuePair {
                key: static_cstring(c"url"),
                value: Object::string(cstr_as_string(url)),
            };
            rgb.size += 1;
        }
    }

    // `info` says which highlight groups produced the attributes, which is
    // only meaningful to a UI that asked to track highlight state.
    let info = if unsafe { (*ui).ui_ext[kUIHlState as usize] } {
        info
    } else {
        Array {
            size: 0,
            capacity: 0,
            items: core::ptr::null_mut(),
        }
    };
    send!(
        ui,
        c"hl_attr_define",
        Object::integer(id),
        Object::dict(rgb),
        Object::dict(cterm),
        Object::array(info),
    );
}

/// Forwards an externalised UI event, translating it for an old UI first.
///
/// # Safety
///
/// `ui` must be live and `args` valid for the duration of the call.
pub unsafe fn remote_ui_event(ui: *mut RemoteUI, name: &'static CStr, args: Array) {
    // Only the translations allocate, and all of them are done by the time
    // this returns, so one arena covers the whole call.
    let mut arena: Arena = ARENA_EMPTY;
    let translated = unsafe { translate(ui, name, args, &raw mut arena) };
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
    translated
}

/// [`remote_ui_event`]'s body, with the arena freed by its caller whichever
/// way this returns.
///
/// # Safety
///
/// As [`remote_ui_event`], plus `arena` must be usable.
unsafe fn translate(ui: *mut RemoteUI, name: &'static CStr, args: Array, arena: *mut Arena) {
    unsafe {
        if !linegrid(ui) {
            // The cmdline events carry `[[attr, text], ...]` chunks, and a
            // UI on the legacy protocol cannot look an attr id up.
            match name.to_bytes() {
                b"cmdline_show" | b"cmdline_block_append" => {
                    let new_args = translate_firstarg(ui, args, arena);
                    push_call(ui, name, new_args);
                    return;
                }
                b"cmdline_block_show" => {
                    let block = (*args.items).data.array;
                    let mut new_block = arena_array(arena, block.size);
                    for i in 0..block.size {
                        let line = (*block.items.add(i)).data.array;
                        push(
                            &mut new_block,
                            Object::array(translate_contents(ui, line, arena)),
                        );
                    }
                    let mut new_args = ArrayBuf::<1>::new();
                    new_args.push(Object::array(new_block));
                    push_call(ui, name, new_args.array());
                    return;
                }
                _ => {}
            }
        }

        // `ext_wildmenu` is the pre-`ext_popupmenu` spelling: same event,
        // fewer arguments, different name. A UI that asked for both gets
        // the popupmenu events, and only completion in the cmdline (which
        // the popupmenu reports as row -1) falls back to the wildmenu.
        if (*ui).ui_ext[kUIWildmenu as usize] {
            match name.to_bytes() {
                b"popupmenu_show" => {
                    (*ui).wildmenu_active = (*args.items.add(4)).data.integer == -1
                        || !(*ui).ui_ext[kUIPopupmenu as usize];
                    if (*ui).wildmenu_active {
                        // The wildmenu shows only the completion word, so
                        // each item's remaining fields are dropped.
                        let items = (*args.items).data.array;
                        let mut new_items = arena_array(arena, items.size);
                        for i in 0..items.size {
                            let item = (*items.items.add(i)).data.array;
                            push(&mut new_items, *item.items);
                        }
                        let mut new_args = ArrayBuf::<1>::new();
                        new_args.push(Object::array(new_items));
                        push_call(ui, c"wildmenu_show", new_args.array());

                        // A popupmenu carries the selected index with the
                        // items; the wildmenu has a separate event for it.
                        let selected = *args.items.add(1);
                        if selected.data.integer != -1 {
                            let mut new_args = ArrayBuf::<1>::new();
                            new_args.push(selected);
                            push_call(ui, c"wildmenu_select", new_args.array());
                        }
                        return;
                    }
                }
                b"popupmenu_select" if (*ui).wildmenu_active => {
                    push_call(ui, c"wildmenu_select", args);
                    return;
                }
                b"popupmenu_hide" if (*ui).wildmenu_active => {
                    push_call(ui, c"wildmenu_hide", args);
                    return;
                }
                _ => {}
            }
        }

        push_call(ui, name, args);
    }
}

/// Appends `value` to an arena-allocated array, which is sized up front and
/// filled in afterwards.
///
/// # Safety
///
/// `array` must have room for one more element.
unsafe fn push(array: &mut Array, value: Object) {
    unsafe { *array.items.add(array.size) = value };
    array.size += 1;
}

/// [`translate_contents`] applied to the first argument, the rest copied.
///
/// # Safety
///
/// `ui` must be live, `args` must have a leading array element, and `arena`
/// must be usable.
unsafe fn translate_firstarg(ui: *mut RemoteUI, args: Array, arena: *mut Arena) -> Array {
    unsafe {
        let mut new_args = arena_array(arena, args.size);
        let contents = (*args.items).data.array;
        push(
            &mut new_args,
            Object::array(translate_contents(ui, contents, arena)),
        );
        for i in 1..args.size {
            push(&mut new_args, *args.items.add(i));
        }
        new_args
    }
}

/// Expands the highlight ids in a chunk list into attribute dicts.
///
/// # Safety
///
/// `ui` must be live, `contents` must be a list of `[attr, text]` pairs,
/// and `arena` must be usable.
unsafe fn translate_contents(ui: *mut RemoteUI, contents: Array, arena: *mut Arena) -> Array {
    unsafe {
        let mut new_contents = arena_array(arena, contents.size);
        for i in 0..contents.size {
            let item = (*contents.items.add(i)).data.array;
            let mut new_item = arena_array(arena, 2);
            let attr = (*item.items).data.integer as core::ffi::c_int;
            let attrs = if attr != 0 {
                let mut dict = arena_dict(arena, HLATTRS_DICT_SIZE as usize);
                hlattrs2dict(
                    &raw mut dict,
                    core::ptr::null_mut(),
                    syn_attr2entry(attr),
                    (*ui).rgb,
                    false,
                );
                dict
            } else {
                Dict {
                    size: 0,
                    capacity: 0,
                    items: core::ptr::null_mut(),
                }
            };
            push(&mut new_item, Object::dict(attrs));
            push(&mut new_item, *item.items.add(1));
            push(&mut new_contents, Object::array(new_item));
        }
        new_contents
    }
}
