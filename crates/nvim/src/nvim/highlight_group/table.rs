//! The group table itself: names, ids, and the links between them.
//!
//! Every highlight group is an entry in one array, and its id is that
//! entry's index plus one. [`syn_check_group`] is the way in — it interns a
//! name, adding a group if it is new — and [`syn_name2id`]/[`syn_id2name`]
//! are the two directions of the lookup. [`syn_ns_get_final_id`] follows
//! `:highlight link` chains (and namespace overrides) to the group that
//! actually carries the attributes, which [`syn_id2attr`] then resolves.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) static value_init_int: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);

pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};

pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};

pub const MAP_INIT: Map_cstr_t_int = Map_cstr_t_int {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};

pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;

#[inline]
pub(crate) unsafe extern "C" fn map_get_cstr_t_int(
    mut map: *mut Map_cstr_t_int,
    mut key: cstr_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_int.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}

#[inline]
pub(crate) unsafe extern "C" fn map_put_cstr_t_int(
    mut map: *mut Map_cstr_t_int,
    mut key: cstr_t,
    mut value: ::core::ffi::c_int,
) {
    unsafe {
        let mut val: *mut ::core::ffi::c_int = map_put_ref_cstr_t_int(
            map,
            key,
            ::core::ptr::null_mut::<*mut cstr_t>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}

pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};

pub(crate) static highlight_ga: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);

pub static highlight_arena: GlobalCell<Arena> = GlobalCell::new(ARENA_EMPTY);

pub static highlight_unames: GlobalCell<Map_cstr_t_int> = GlobalCell::new(MAP_INIT);

pub unsafe extern "C" fn highlight_num_groups() -> ::core::ffi::c_int {
    unsafe {
        return (*highlight_ga.ptr()).ga_len;
    }
}

pub unsafe extern "C" fn highlight_group_name(
    mut id: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(id as isize)).sg_name;
    }
}

pub unsafe extern "C" fn highlight_link_id(mut id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(id as isize)).sg_link;
    }
}

pub(crate) unsafe extern "C" fn hl_has_settings(
    mut idx: ::core::ffi::c_int,
    mut check_link: bool,
) -> bool {
    unsafe {
        return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared
            as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            && ((*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_attr
                != 0 as ::core::ffi::c_int
                || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                    .sg_cterm_fg
                    != 0 as ::core::ffi::c_int
                || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                    .sg_cterm_bg
                    != 0 as ::core::ffi::c_int
                || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                    .sg_rgb_fg_idx
                    != kColorIdxNone as ::core::ffi::c_int
                || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                    .sg_rgb_bg_idx
                    != kColorIdxNone as ::core::ffi::c_int
                || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                    .sg_rgb_sp_idx
                    != kColorIdxNone as ::core::ffi::c_int
                || check_link as ::core::ffi::c_int != 0
                    && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                        .sg_set
                        & SG_LINK as ::core::ffi::c_int
                        != 0);
    }
}

pub(crate) unsafe extern "C" fn highlight_clear(mut idx: ::core::ffi::c_int) {
    unsafe {
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared =
            true_0 != 0;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_attr =
            0 as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm =
            0 as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_bold =
            false_0 != 0;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_fg =
            0 as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_bg =
            0 as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_gui =
            0 as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_fg =
            -1 as ::core::ffi::c_int as RgbValue;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_bg =
            -1 as ::core::ffi::c_int as RgbValue;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_sp =
            -1 as ::core::ffi::c_int as RgbValue;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_fg_idx =
            kColorIdxNone as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_bg_idx =
            kColorIdxNone as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_sp_idx =
            kColorIdxNone as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_blend =
            -1 as ::core::ffi::c_int;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_link =
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_deflink;
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_script_ctx =
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_deflink_sctx;
    }
}

pub(crate) unsafe extern "C" fn set_hl_attr(mut idx: ::core::ffi::c_int) {
    unsafe {
        let mut at_en: HlAttrs = HLATTRS_INIT;
        let mut sgp: *mut HlGroup =
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
        at_en.cterm_ae_attr = (*sgp).sg_cterm as int32_t;
        at_en.cterm_fg_color = (*sgp).sg_cterm_fg as int16_t;
        at_en.cterm_bg_color = (*sgp).sg_cterm_bg as int16_t;
        at_en.rgb_ae_attr = (*sgp).sg_gui as int32_t;
        at_en.rgb_fg_color = if (*sgp).sg_rgb_fg_idx != kColorIdxNone as ::core::ffi::c_int {
            (*sgp).sg_rgb_fg
        } else {
            -1 as RgbValue
        };
        at_en.rgb_bg_color = if (*sgp).sg_rgb_bg_idx != kColorIdxNone as ::core::ffi::c_int {
            (*sgp).sg_rgb_bg
        } else {
            -1 as RgbValue
        };
        at_en.rgb_sp_color = if (*sgp).sg_rgb_sp_idx != kColorIdxNone as ::core::ffi::c_int {
            (*sgp).sg_rgb_sp
        } else {
            -1 as RgbValue
        };
        at_en.hl_blend = (*sgp).sg_blend as int32_t;
        (*sgp).sg_attr = hl_get_syn_attr(
            0 as ::core::ffi::c_int,
            idx + 1 as ::core::ffi::c_int,
            at_en,
        );
        if cursor_mode_uses_syn_id(idx + 1 as ::core::ffi::c_int) {
            ui_mode_info_set();
        }
    }
}

pub unsafe extern "C" fn syn_name2id(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '@' as ::core::ffi::c_int
        {
            return syn_check_group(name, strlen(name));
        }
        return syn_name2id_len(name, strlen(name));
    }
}

pub unsafe extern "C" fn syn_name2id_len(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut name_u: [::core::ffi::c_char; 201] = [0; 201];
        if len == 0 as size_t || len > MAX_SYN_NAME as size_t {
            return 0 as ::core::ffi::c_int;
        }
        vim_memcpy_up(&raw mut name_u as *mut ::core::ffi::c_char, name, len);
        name_u[len as usize] = NUL as ::core::ffi::c_char;
        return map_get_cstr_t_int(
            highlight_unames.ptr(),
            &raw mut name_u as *mut ::core::ffi::c_char as cstr_t,
        );
    }
}

pub unsafe extern "C" fn syn_name2attr(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut id: ::core::ffi::c_int = syn_name2id(name);
        if id != 0 as ::core::ffi::c_int {
            return syn_id2attr(id);
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn highlight_exists(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return (syn_name2id(name) > 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn syn_id2name(mut id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    unsafe {
        if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((id - 1 as ::core::ffi::c_int) as isize))
        .sg_name;
    }
}

pub unsafe extern "C" fn syn_check_group(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        if len > MAX_SYN_NAME as size_t {
            emsg(gettext(
                &raw const e_highlight_group_name_too_long as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        }
        let mut id: ::core::ffi::c_int = syn_name2id_len(name, len);
        if id == 0 as ::core::ffi::c_int {
            return syn_add_group(name, len);
        }
        return id;
    }
}

pub(crate) unsafe extern "C" fn syn_add_group(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < len {
            let mut c: ::core::ffi::c_int =
                *name.offset(i as isize) as uint8_t as ::core::ffi::c_int;
            if !vim_isprintc(c) {
                emsg(gettext(
                    b"E669: Unprintable character in group name\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return 0 as ::core::ffi::c_int;
            } else if !(c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(c) as ::core::ffi::c_int != 0)
                && c != '_' as ::core::ffi::c_int
                && c != '.' as ::core::ffi::c_int
                && c != '@' as ::core::ffi::c_int
                && c != '-' as ::core::ffi::c_int
            {
                msg_source(HLF_W);
                emsg(gettext(
                    &raw const e_highlight_group_name_invalid_char as *const ::core::ffi::c_char,
                ));
                return 0 as ::core::ffi::c_int;
            }
            i = i.wrapping_add(1);
        }
        let mut scoped_parent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if len > 1 as size_t
            && *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '@' as ::core::ffi::c_int
        {
            let mut delim: *mut ::core::ffi::c_char =
                xmemrchr(name as *const ::core::ffi::c_void, '.' as uint8_t, len)
                    as *mut ::core::ffi::c_char;
            if !delim.is_null() {
                scoped_parent = syn_check_group(name, delim.offset_from(name) as size_t);
            }
        }
        if (*highlight_ga.ptr()).ga_data.is_null() {
            (*highlight_ga.ptr()).ga_itemsize =
                ::core::mem::size_of::<HlGroup>() as ::core::ffi::c_int;
            ga_set_growsize(highlight_ga.ptr(), 10 as ::core::ffi::c_int);
            ga_grow(highlight_ga.ptr(), 300 as ::core::ffi::c_int);
        }
        if (*highlight_ga.ptr()).ga_len >= MAX_HL_ID as ::core::ffi::c_int {
            emsg(gettext(
                b"E849: Too many highlight and syntax groups\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        }
        let mut hlgp: *mut HlGroup =
            ga_append_via_ptr(highlight_ga.ptr(), ::core::mem::size_of::<HlGroup>())
                as *mut HlGroup;
        memset(
            hlgp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<HlGroup>(),
        );
        (*hlgp).sg_name = arena_memdupz(highlight_arena.ptr(), name, len);
        (*hlgp).sg_rgb_bg = -1 as ::core::ffi::c_int as RgbValue;
        (*hlgp).sg_rgb_fg = -1 as ::core::ffi::c_int as RgbValue;
        (*hlgp).sg_rgb_sp = -1 as ::core::ffi::c_int as RgbValue;
        (*hlgp).sg_rgb_bg_idx = kColorIdxNone as ::core::ffi::c_int;
        (*hlgp).sg_rgb_fg_idx = kColorIdxNone as ::core::ffi::c_int;
        (*hlgp).sg_rgb_sp_idx = kColorIdxNone as ::core::ffi::c_int;
        (*hlgp).sg_blend = -1 as ::core::ffi::c_int;
        (*hlgp).sg_name_u = arena_memdupz(highlight_arena.ptr(), name, len);
        (*hlgp).sg_parent = scoped_parent;
        (*hlgp).sg_cleared = true_0 != 0;
        vim_strup((*hlgp).sg_name_u);
        let mut id: ::core::ffi::c_int = (*highlight_ga.ptr()).ga_len;
        map_put_cstr_t_int(highlight_unames.ptr(), (*hlgp).sg_name_u as cstr_t, id);
        return id;
    }
}

pub unsafe extern "C" fn syn_id2attr(mut hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut optional: bool = false_0 != 0;
        return syn_ns_id2attr(-1 as ::core::ffi::c_int, hl_id, &raw mut optional);
    }
}

pub unsafe extern "C" fn syn_ns_id2attr(
    mut ns_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut optional: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        if syn_ns_get_final_id(&raw mut ns_id, &raw mut hl_id) {
            *optional = false_0 != 0;
        }
        let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
        let mut attr: ::core::ffi::c_int =
            ns_get_hl(&mut ns_id, hl_id, false_0 != 0, (*sgp).sg_set != 0);
        if attr >= 0 as ::core::ffi::c_int
            || *optional as ::core::ffi::c_int != 0 && ns_id > 0 as ::core::ffi::c_int
        {
            return attr;
        }
        return (*sgp).sg_attr;
    }
}

pub unsafe extern "C" fn syn_get_final_id(mut hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut ns_id: ::core::ffi::c_int = (*curwin.get()).w_ns_hl_active;
        syn_ns_get_final_id(&raw mut ns_id, &raw mut hl_id);
        return hl_id;
    }
}

pub unsafe extern "C" fn syn_ns_get_final_id(
    mut ns_id: *mut ::core::ffi::c_int,
    mut hl_idp: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut hl_id: ::core::ffi::c_int = *hl_idp;
        let mut used: bool = false_0 != 0;
        if hl_id > (*highlight_ga.ptr()).ga_len || hl_id < 1 as ::core::ffi::c_int {
            *hl_idp = 0 as ::core::ffi::c_int;
            return false_0 != 0;
        }
        let mut count: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
        loop {
            count -= 1;
            if count < 0 as ::core::ffi::c_int {
                break;
            }
            let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
            let mut check: ::core::ffi::c_int =
                ns_get_hl(&mut *ns_id, hl_id, true_0 != 0, (*sgp).sg_set != 0);
            if check == 0 as ::core::ffi::c_int {
                *hl_idp = hl_id;
                return true_0 != 0;
            } else if check > 0 as ::core::ffi::c_int {
                used = true_0 != 0;
                hl_id = check;
            } else if (*sgp).sg_link > 0 as ::core::ffi::c_int
                && (*sgp).sg_link <= (*highlight_ga.ptr()).ga_len
            {
                hl_id = (*sgp).sg_link;
            } else {
                if !((*sgp).sg_cleared as ::core::ffi::c_int != 0
                    && (*sgp).sg_parent > 0 as ::core::ffi::c_int)
                {
                    break;
                }
                hl_id = (*sgp).sg_parent;
            }
        }
        *hl_idp = hl_id;
        return used;
    }
}

pub unsafe extern "C" fn highlight_attr_set_all() {
    unsafe {
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*highlight_ga.ptr()).ga_len {
            let mut sgp: *mut HlGroup =
                ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
            if (*sgp).sg_rgb_bg_idx == kColorIdxFg as ::core::ffi::c_int {
                (*sgp).sg_rgb_bg = normal_fg.get();
            } else if (*sgp).sg_rgb_bg_idx == kColorIdxBg as ::core::ffi::c_int {
                (*sgp).sg_rgb_bg = normal_bg.get();
            }
            if (*sgp).sg_rgb_fg_idx == kColorIdxFg as ::core::ffi::c_int {
                (*sgp).sg_rgb_fg = normal_fg.get();
            } else if (*sgp).sg_rgb_fg_idx == kColorIdxBg as ::core::ffi::c_int {
                (*sgp).sg_rgb_fg = normal_bg.get();
            }
            if (*sgp).sg_rgb_sp_idx == kColorIdxFg as ::core::ffi::c_int {
                (*sgp).sg_rgb_sp = normal_fg.get();
            } else if (*sgp).sg_rgb_sp_idx == kColorIdxBg as ::core::ffi::c_int {
                (*sgp).sg_rgb_sp = normal_bg.get();
            }
            set_hl_attr(idx);
            idx += 1;
        }
    }
}
