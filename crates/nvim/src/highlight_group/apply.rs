//! [`set_hl_group`], the other way to define a group.
//!
//! `nvim_set_hl()` in the global namespace arrives here with the attributes
//! already parsed out of its dictionary, so this is the same work
//! [`super::do_highlight`] does key by key, done all at once.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use crate::cursor_shape::cursor_mode_uses_syn_id;
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::highlight::{HL_DEFAULT, hl_get_syn_attr};
use crate::lua::executor::nlua_set_sctx;
use crate::main::{
    cterm_normal_bg_color, cterm_normal_fg_color, current_sctx, need_highlight_changed, normal_bg,
    normal_fg, normal_sp, updating_screen,
};
use crate::types::{HlAttrs, KeyDict_highlight, Object, kObjectTypeNil, kObjectTypeString};
use crate::ui::{ui_default_colors_set, ui_mode_info_set};

use super::command::sourcing_lnum;
use super::{
    KEYSET_OPTIDX_highlight__bg, KEYSET_OPTIDX_highlight__fg, KEYSET_OPTIDX_highlight__sp,
    KEYSET_OPTIDX_highlight__update, SG_LINK, group, highlight_attr_set_all, hl_has_settings,
    kColorIdxHex, kColorIdxNone, name_to_color, with_group,
};

/// Whether the caller set key `bit` of `Dict(highlight)`.
fn has_key(dict: &KeyDict_highlight, bit: c_int) -> bool {
    dict.is_set__highlight_ & (1u64 << bit) != 0
}

/// Applies `attrs` (and `link_id`, if any) to the group with id `id`.
///
/// `dict` is the caller's `nvim_set_hl()` argument, which is still needed for
/// three things the parsed `HlAttrs` cannot carry: `force`, `update`, and the
/// *spelling* of each colour, so that `:highlight` can print back the name it
/// was given rather than a hex value.
///
/// # Safety
/// Redraws and emits UI events; main thread only.
pub unsafe fn set_hl_group(id: c_int, attrs: HlAttrs, dict: &KeyDict_highlight, link_id: c_int) {
    let is_default = attrs.rgb_ae_attr & HL_DEFAULT != 0;

    // Return if "default" was used and the group already has settings.
    if is_default && hl_has_settings(id, true) && !dict.force {
        return;
    }

    // SAFETY: the editor's own tables and UI, on the main thread.
    unsafe {
        let old_link = group(id).link;
        with_group(id, |entry| {
            entry.cleared = false;
            if link_id > 0 {
                entry.link = link_id;
                entry.script_ctx = current_sctx.get();
                entry.script_ctx.sc_lnum += sourcing_lnum();
                nlua_set_sctx(&raw mut entry.script_ctx);
                entry.set |= SG_LINK as c_int;
                if is_default {
                    entry.deflink = link_id;
                    entry.deflink_sctx = current_sctx.get();
                    entry.deflink_sctx.sc_lnum += sourcing_lnum();
                    nlua_set_sctx(&raw mut entry.deflink_sctx);
                }
            } else {
                entry.link = 0;
            }

            entry.gui = attrs.rgb_ae_attr & !HL_DEFAULT;
            entry.rgb_fg = attrs.rgb_fg_color;
            entry.rgb_bg = attrs.rgb_bg_color;
            entry.rgb_sp = attrs.rgb_sp_color;
        });

        let update = has_key(dict, KEYSET_OPTIDX_highlight__update) && dict.update;
        let entry = group(id);
        // The colour *spellings*: what `:highlight` will print back. A name
        // becomes its table index, a number `kColorIdxHex`, and an absent key
        // either clears the index or — with `update` — inherits the linked
        // group's, so that an inherited colour still shows a name.
        let linked = (old_link > 0).then(|| group(old_link));
        let spellings = [
            (
                entry.rgb_fg,
                pick(dict, KEYSET_OPTIDX_highlight__fg, dict.fg, dict.foreground),
                linked.map(|g| g.rgb_fg_idx),
            ),
            (
                entry.rgb_bg,
                pick(dict, KEYSET_OPTIDX_highlight__bg, dict.bg, dict.background),
                linked.map(|g| g.rgb_bg_idx),
            ),
            (
                entry.rgb_sp,
                pick(dict, KEYSET_OPTIDX_highlight__sp, dict.sp, dict.special),
                linked.map(|g| g.rgb_sp_idx),
            ),
        ];
        let mut idxs = [KEEP; 3];
        for (slot, &(value, name, linked_idx)) in idxs.iter_mut().zip(&spellings) {
            *slot = if name.type_0 != kObjectTypeNil {
                if value < 0 {
                    kColorIdxNone
                } else if name.type_0 == kObjectTypeString && name.data.string.size != 0 {
                    // SAFETY: an API string is NUL-terminated.
                    name_to_color(::core::ffi::CStr::from_ptr(name.data.string.data)).1
                } else {
                    kColorIdxHex
                }
            } else if !update {
                kColorIdxNone
            } else if let Some(linked_idx) = linked_idx
                && value >= 0
            {
                if linked_idx != kColorIdxNone {
                    linked_idx
                } else {
                    kColorIdxHex
                }
            } else {
                // `update` with nothing to inherit: leave what is there.
                KEEP
            };
        }

        with_group(id, |entry| {
            for (slot, &new) in [
                &mut entry.rgb_fg_idx,
                &mut entry.rgb_bg_idx,
                &mut entry.rgb_sp_idx,
            ]
            .into_iter()
            .zip(&idxs)
            {
                if new != KEEP {
                    *slot = new;
                }
            }

            entry.cterm = attrs.cterm_ae_attr & !HL_DEFAULT;
            entry.cterm_bg = c_int::from(attrs.cterm_bg_color);
            entry.cterm_fg = c_int::from(attrs.cterm_fg_color);
            entry.cterm_bold = entry.cterm & super::HL_BOLD != 0;

            if attrs.hl_blend != -1 {
                entry.blend = attrs.hl_blend;
            } else if !update {
                entry.blend = -1;
            }

            entry.script_ctx = current_sctx.get();
            entry.script_ctx.sc_lnum += sourcing_lnum();
            nlua_set_sctx(&raw mut entry.script_ctx);
        });

        let attr = hl_get_syn_attr(0, id, attrs);
        with_group(id, |entry| entry.attr = attr);

        // 'Normal' is special.
        let entry = group(id);
        if entry.name_u == c"NORMAL" {
            cterm_normal_fg_color.set(entry.cterm_fg);
            cterm_normal_bg_color.set(entry.cterm_bg);
            let changed = normal_bg.get() != entry.rgb_bg
                || normal_fg.get() != entry.rgb_fg
                || normal_sp.get() != entry.rgb_sp;
            normal_fg.set(entry.rgb_fg);
            normal_bg.set(entry.rgb_bg);
            normal_sp.set(entry.rgb_sp);
            if changed {
                highlight_attr_set_all();
            }
            ui_default_colors_set();
        } else if cursor_mode_uses_syn_id(id) {
            // A cursor style uses this group; its attribute has changed.
            ui_mode_info_set();
        }

        if !updating_screen.get() {
            redraw_all_later(UPD_NOT_VALID);
        }
        need_highlight_changed.set(true);
    }
}

/// Not a colour index: "leave the one that is there alone".
const KEEP: c_int = c_int::MIN;

/// The long key wins only if the short one was not given, which is how
/// `fg`/`foreground` and their two siblings pair up.
fn pick(dict: &KeyDict_highlight, bit: c_int, short: Object, long: Object) -> Object {
    if has_key(dict, bit) { short } else { long }
}
