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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::Ui;
use super::events::{count, linegrid, send};
use super::packer::push_call;
use crate::api::private::helpers::cstr_to_string;
use crate::highlight::{HLATTRS_DICT_SIZE, hl_get_url, hlattrs2dict, syn_attr2entry};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::narrow::number_as_int;
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::ui::{kUIHlState, kUIPopupmenu, kUIWildmenu};
use crate::types::{ApiDict, Arena, Array, HlAttrs, Integer, Object, RemoteUI};
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
    let mut rgb_buf = DictBuf::<{ HLATTRS_DICT_SIZE + 1 }>::new();
    let mut cterm_buf = DictBuf::<HLATTRS_DICT_SIZE>::new();
    let mut rgb = rgb_buf.dict();
    let mut cterm = cterm_buf.dict();
    unsafe {
        hlattrs2dict(&mut rgb, None, rgb_attrs, true, false);
        hlattrs2dict(&mut cterm, None, rgb_attrs, false, false);
        if rgb_attrs.url >= 0 {
            let url = hl_get_url(rgb_attrs.url.cast_unsigned());
            rgb.insert(c"url", Object::string(cstr_to_string(url)));
        }
    }

    // `info` says which highlight groups produced the attributes, which is
    // only meaningful to a UI that asked to track highlight state.
    // SAFETY: `ui` is live, per this function's contract.
    let hl_state = unsafe { Ui::new(ui) }.ui_ext[kUIHlState as usize];
    let info = if hl_state { info } else { Array::EMPTY };
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
    unsafe { translate(ui, name, args) };
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
}

/// [`remote_ui_event`]'s body.
///
/// # Safety
///
/// As [`remote_ui_event`].
unsafe fn translate(ui: *mut RemoteUI, name: &'static CStr, args: Array) {
    // SAFETY: the caller's promise -- `ui` is live.
    let mut live = unsafe { Ui::new(ui) };
    // SAFETY: as above.
    if !unsafe { linegrid(ui) } {
        // The cmdline events carry `[[attr, text], ...]` chunks, and a
        // UI on the legacy protocol cannot look an attr id up.
        match name.to_bytes() {
            b"cmdline_show" | b"cmdline_block_append" => {
                // SAFETY: `args` and `ui` are the caller's.
                unsafe {
                    let new_args = translate_firstarg(ui, &args);
                    push_call(ui, name, new_args);
                }
                return;
            }
            b"cmdline_block_show" => {
                // The editor built `args` for this event, so its one element
                // is the block of lines.
                // SAFETY: a `cmdline_block_show` has that one element.
                let block = args[0]
                    .as_array()
                    .expect("cmdline_block_show carries a block of lines");
                let mut new_block = Array::with_capacity(block.len());
                for item in block {
                    let line = item
                        .as_array()
                        .expect("a cmdline block's element is one line's chunks");
                    // SAFETY: `ui` is live, per this function's contract.
                    let translated = unsafe { translate_contents(ui, line) };
                    new_block.push(Object::array(translated));
                }
                let mut new_args = ArrayBuf::<1>::new();
                new_args.push(Object::array(new_block));
                // SAFETY: `new_args` borrows this frame's buffer.
                unsafe { push_call(ui, name, new_args.array()) };
                return;
            }
            _ => {}
        }
    }

    // `ext_wildmenu` is the pre-`ext_popupmenu` spelling: same event,
    // fewer arguments, different name. A UI that asked for both gets
    // the popupmenu events, and only completion in the cmdline (which
    // the popupmenu reports as row -1) falls back to the wildmenu.
    if live.ui_ext[kUIWildmenu as usize] {
        match name.to_bytes() {
            b"popupmenu_show" => {
                // SAFETY: the editor built `args` for this event, which has
                // five elements.
                let row = args[4]
                    .as_integer()
                    .expect("popupmenu_show carries an anchor row");
                live.wildmenu_active = row == -1 || !live.ui_ext[kUIPopupmenu as usize];
                if live.wildmenu_active {
                    // The wildmenu shows only the completion word, so
                    // each item's remaining fields are dropped.
                    // SAFETY: the event has a first element.
                    let items = args[0]
                        .as_array()
                        .expect("popupmenu_show carries an item list");
                    let mut new_items = Array::with_capacity(items.len());
                    for item in items {
                        let fields = item.as_array().expect("a popupmenu item is a field list");
                        new_items.push(fields[0].clone());
                    }
                    let mut new_args = ArrayBuf::<1>::new();
                    new_args.push(Object::array(new_items));
                    // SAFETY: `new_args` borrows this frame's buffer.
                    unsafe { push_call(ui, c"wildmenu_show", new_args.array()) };

                    // A popupmenu carries the selected index with the
                    // items; the wildmenu has a separate event for it.
                    // The second element is that index.
                    let selected = &args[1];
                    if selected.as_integer() != Some(-1) {
                        let mut new_args = ArrayBuf::<1>::new();
                        new_args.push(selected.clone());
                        // SAFETY: as above.
                        unsafe { push_call(ui, c"wildmenu_select", new_args.array()) };
                    }
                    return;
                }
            }
            b"popupmenu_select" if live.wildmenu_active => {
                // SAFETY: `args` is the caller's, unchanged.
                unsafe { push_call(ui, c"wildmenu_select", args) };
                return;
            }
            b"popupmenu_hide" if live.wildmenu_active => {
                // SAFETY: as above.
                unsafe { push_call(ui, c"wildmenu_hide", args) };
                return;
            }
            _ => {}
        }
    }

    // SAFETY: `args` is the caller's, forwarded unchanged.
    unsafe { push_call(ui, name, args) };
}

/// [`translate_contents`] applied to the first argument, the rest copied.
///
/// # Safety
///
/// `ui` must be live and `args` must have a leading array element.
unsafe fn translate_firstarg(ui: *mut RemoteUI, args: &Array) -> Array {
    let mut new_args = Array::with_capacity(args.len());
    let contents = args[0]
        .as_array()
        .expect("the first argument is a chunk list");
    // SAFETY: the caller's promise -- `ui` is live.
    let translated = unsafe { translate_contents(ui, contents) };
    new_args.push(Object::array(translated));
    for item in args.iter().skip(1) {
        new_args.push(item.clone());
    }
    new_args
}

/// Expands the highlight ids in a chunk list into attribute dicts.
///
/// # Safety
///
/// `ui` must be live and `contents` must be a list of `[attr, text]` pairs.
unsafe fn translate_contents(ui: *mut RemoteUI, contents: &Array) -> Array {
    // SAFETY: the caller's promise -- `ui` is live.
    let live = unsafe { Ui::new(ui) };
    let rgb = live.rgb;
    let mut new_contents = Array::with_capacity(contents.len());
    for chunk in contents {
        let item = chunk.as_array().expect("a chunk is an [attr, text] pair");
        let attr = number_as_int(
            item[0]
                .as_integer()
                .expect("a chunk's first element is its attribute id"),
        );
        let mut new_item = Array::with_capacity(2);
        let attrs = if attr != 0 {
            let mut dict = ApiDict::with_capacity(HLATTRS_DICT_SIZE);
            // SAFETY: `attr` is a resolved attribute id.
            unsafe { hlattrs2dict(&mut dict, None, syn_attr2entry(attr), rgb, false) };
            dict
        } else {
            ApiDict::EMPTY
        };
        new_item.push(Object::dict(attrs));
        new_item.push(item[1].clone());
        new_contents.push(Object::array(new_item));
    }
    new_contents
}
