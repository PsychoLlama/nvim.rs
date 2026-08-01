//! [`set_hl_group`], the other way to define a group.
//!
//! `nvim_set_hl()` in the global namespace arrives here with the attributes
//! already parsed out of its dictionary, so this is the same work
//! [`do_highlight`] does key by key, done all at once.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn set_hl_group(
    mut id: ::core::ffi::c_int,
    mut attrs: HlAttrs,
    mut dict: *mut KeyDict_highlight,
    mut link_id: ::core::ffi::c_int,
) {
    unsafe {
        let mut idx: ::core::ffi::c_int = id - 1 as ::core::ffi::c_int;
        let mut is_default: bool = attrs.rgb_ae_attr & HL_DEFAULT as int32_t != 0;
        if is_default as ::core::ffi::c_int != 0
            && hl_has_settings(idx, true_0 != 0) as ::core::ffi::c_int != 0
            && !(*dict).force
        {
            return;
        }
        let mut g: *mut HlGroup =
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
        (*g).sg_cleared = false_0 != 0;
        let mut old_link: ::core::ffi::c_int = (*g).sg_link;
        if link_id > 0 as ::core::ffi::c_int {
            (*g).sg_link = link_id;
            (*g).sg_script_ctx = current_sctx.get();
            (*g).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum;
            nlua_set_sctx(&raw mut (*g).sg_script_ctx);
            (*g).sg_set |= SG_LINK as ::core::ffi::c_int;
            if is_default {
                (*g).sg_deflink = link_id;
                (*g).sg_deflink_sctx = current_sctx.get();
                (*g).sg_deflink_sctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
                nlua_set_sctx(&raw mut (*g).sg_deflink_sctx);
            }
        } else {
            (*g).sg_link = 0 as ::core::ffi::c_int;
        }
        let mut update: bool = (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__update
            != 0 as ::core::ffi::c_ulonglong
            && (*dict).update as ::core::ffi::c_int != 0;
        (*g).sg_gui = (attrs.rgb_ae_attr & !(HL_DEFAULT as int32_t)) as ::core::ffi::c_int;
        (*g).sg_rgb_fg = attrs.rgb_fg_color;
        (*g).sg_rgb_bg = attrs.rgb_bg_color;
        (*g).sg_rgb_sp = attrs.rgb_sp_color;
        let mut cattrs: [C2Rust_Unnamed_21; 4] = [
            C2Rust_Unnamed_21 {
                dest: &raw mut (*g).sg_rgb_fg_idx,
                val: (*g).sg_rgb_fg,
                name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__fg
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*dict).fg
                } else {
                    (*dict).foreground
                },
            },
            C2Rust_Unnamed_21 {
                dest: &raw mut (*g).sg_rgb_bg_idx,
                val: (*g).sg_rgb_bg,
                name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__bg
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*dict).bg
                } else {
                    (*dict).background
                },
            },
            C2Rust_Unnamed_21 {
                dest: &raw mut (*g).sg_rgb_sp_idx,
                val: (*g).sg_rgb_sp,
                name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__sp
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*dict).sp
                } else {
                    (*dict).special
                },
            },
            C2Rust_Unnamed_21 {
                dest: ::core::ptr::null_mut::<::core::ffi::c_int>(),
                val: -1 as RgbValue,
                name: object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed_0 { boolean: false },
                },
            },
        ];
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !cattrs[j as usize].dest.is_null() {
            if cattrs[j as usize].name.type_0 as ::core::ffi::c_uint
                != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if cattrs[j as usize].val < 0 as RgbValue {
                    *cattrs[j as usize].dest = kColorIdxNone as ::core::ffi::c_int;
                } else if cattrs[j as usize].name.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    && cattrs[j as usize].name.data.string.size != 0
                {
                    *cattrs[j as usize].dest = name_to_color(::core::ffi::CStr::from_ptr(
                        cattrs[j as usize].name.data.string.data,
                    ))
                    .1;
                } else {
                    *cattrs[j as usize].dest = kColorIdxHex as ::core::ffi::c_int;
                }
            } else if !update {
                *cattrs[j as usize].dest = kColorIdxNone as ::core::ffi::c_int;
            } else if old_link > 0 as ::core::ffi::c_int && cattrs[j as usize].val >= 0 as RgbValue
            {
                let mut linked: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((old_link - 1 as ::core::ffi::c_int) as isize);
                let mut linked_idx: ::core::ffi::c_int = if j == 0 as ::core::ffi::c_int {
                    (*linked).sg_rgb_fg_idx
                } else if j == 1 as ::core::ffi::c_int {
                    (*linked).sg_rgb_bg_idx
                } else {
                    (*linked).sg_rgb_sp_idx
                };
                *cattrs[j as usize].dest = if linked_idx != kColorIdxNone as ::core::ffi::c_int {
                    linked_idx
                } else {
                    kColorIdxHex as ::core::ffi::c_int
                };
            }
            j += 1;
        }
        (*g).sg_cterm = (attrs.cterm_ae_attr & !(HL_DEFAULT as int32_t)) as ::core::ffi::c_int;
        (*g).sg_cterm_bg = attrs.cterm_bg_color as ::core::ffi::c_int;
        (*g).sg_cterm_fg = attrs.cterm_fg_color as ::core::ffi::c_int;
        (*g).sg_cterm_bold = (*g).sg_cterm & HL_BOLD != 0;
        if attrs.hl_blend != -1 as int32_t {
            (*g).sg_blend = attrs.hl_blend as ::core::ffi::c_int;
        } else if !update {
            (*g).sg_blend = -1 as ::core::ffi::c_int;
        }
        (*g).sg_script_ctx = current_sctx.get();
        (*g).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*g).sg_script_ctx);
        (*g).sg_attr = hl_get_syn_attr(0 as ::core::ffi::c_int, id, attrs);
        if strcmp(
            (*g).sg_name_u,
            b"NORMAL\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            cterm_normal_fg_color.set((*g).sg_cterm_fg);
            cterm_normal_bg_color.set((*g).sg_cterm_bg);
            let mut did_changed: bool = false_0 != 0;
            if normal_bg.get() != (*g).sg_rgb_bg
                || normal_fg.get() != (*g).sg_rgb_fg
                || normal_sp.get() != (*g).sg_rgb_sp
            {
                did_changed = true_0 != 0;
            }
            normal_fg.set((*g).sg_rgb_fg);
            normal_bg.set((*g).sg_rgb_bg);
            normal_sp.set((*g).sg_rgb_sp);
            if did_changed {
                highlight_attr_set_all();
            }
            ui_default_colors_set();
        } else if cursor_mode_uses_syn_id(id) {
            ui_mode_info_set();
        }
        if !updating_screen.get() {
            redraw_all_later(UPD_NOT_VALID);
        }
        need_highlight_changed.set(true_0 != 0);
    }
}
