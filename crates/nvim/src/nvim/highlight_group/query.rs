//! Answering questions about a group from outside.
//!
//! [`ns_get_hl_defs`] is `nvim_get_hl()`, building a dictionary per group
//! via [`hlgroup2dict`]; [`highlight_has_attr`] and [`highlight_color`] are
//! what `synIDattr()` calls for one attribute at a time.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn hlgroup2dict(
    mut hl: *mut Dict,
    mut ns_id: NS,
    mut hl_id: ::core::ffi::c_int,
    mut arena: *mut Arena,
) -> bool {
    unsafe {
        let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
        let mut ns: NS = ns_id;
        let mut link: ::core::ffi::c_int = if ns_id == 0 as ::core::ffi::c_int {
            (*sgp).sg_link
        } else {
            ns_get_hl(&mut ns, hl_id, true_0 != 0, (*sgp).sg_set != 0)
        };
        if link == -1 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if ns_id == 0 as ::core::ffi::c_int
            && (*sgp).sg_cleared as ::core::ffi::c_int != 0
            && (*sgp).sg_set == 0 as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        ns = ns_id;
        let mut attr: HlAttrs = syn_attr2entry(if ns_id == 0 as ::core::ffi::c_int {
            (*sgp).sg_attr
        } else {
            ns_get_hl(&mut ns, hl_id, false_0 != 0, (*sgp).sg_set != 0)
        });
        *hl = arena_dict(
            arena,
            (HLATTRS_DICT_SIZE as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
        );
        if attr.rgb_ae_attr & HL_DEFAULT as int32_t != 0 {
            let c2rust_fresh1 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh1 as isize) = key_value_pair {
                key: cstr_as_string(b"default\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_0 { boolean: true },
                },
            };
        }
        if link > 0 as ::core::ffi::c_int {
            '_c2rust_label: {
                if 1 as ::core::ffi::c_int <= link && link <= (*highlight_ga.ptr()).ga_len {
                } else {
                    __assert_fail(
                        b"1 <= link && link <= highlight_ga.ga_len\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/highlight_group.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1661 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            let c2rust_fresh2 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh2 as isize) = key_value_pair {
                key: cstr_as_string(b"link\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_0 {
                        string: cstr_as_string(
                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                .offset((link - 1 as ::core::ffi::c_int) as isize))
                            .sg_name,
                        ),
                    },
                },
            };
        }
        let mut hl_cterm: Dict =
            arena_dict(arena, HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t);
        hlattrs2dict(&mut *hl, None, attr, true_0 != 0, true_0 != 0);
        hlattrs2dict(
            &mut *hl,
            Some(&mut hl_cterm),
            attr,
            false_0 != 0,
            true_0 != 0,
        );
        if hl_cterm.size != 0 {
            let c2rust_fresh3 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh3 as isize) = key_value_pair {
                key: cstr_as_string(b"cterm\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeDict,
                    data: C2Rust_Unnamed_0 { dict: hl_cterm },
                },
            };
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn ns_get_hl_defs(
    mut ns_id: NS,
    mut opts: *mut KeyDict_get_highlight,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut rv: Dict = Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut link: Boolean = if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__link
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).link as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
        let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__name
            != 0 as ::core::ffi::c_ulonglong
        {
            let mut create: Boolean = if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__create
                != 0 as ::core::ffi::c_ulonglong
            {
                (*opts).create as ::core::ffi::c_int
            } else {
                true_0
            } != 0;
            id = if create as ::core::ffi::c_int != 0 {
                syn_check_group((*opts).name.data, (*opts).name.size)
            } else {
                syn_name2id_len((*opts).name.data, (*opts).name.size)
            };
            if id == 0 as ::core::ffi::c_int && !create {
                let mut attrs: Dict = ARRAY_DICT_INIT;
                return attrs;
            }
        } else if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__id
            != 0 as ::core::ffi::c_ulonglong
        {
            id = (*opts).id as ::core::ffi::c_int;
        }
        if id != -1 as ::core::ffi::c_int {
            if !(1 as ::core::ffi::c_int <= id && id <= (*highlight_ga.ptr()).ga_len) {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"Highlight id out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                let mut attrs_0: Dict = ARRAY_DICT_INIT;
                hlgroup2dict(
                    &raw mut attrs_0,
                    ns_id,
                    if link as ::core::ffi::c_int != 0 {
                        id
                    } else {
                        syn_get_final_id(id)
                    },
                    arena,
                );
                return attrs_0;
            }
        } else if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            rv = arena_dict(arena, (*highlight_ga.ptr()).ga_len as size_t);
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while i <= (*highlight_ga.ptr()).ga_len {
                let mut attrs_1: Dict = ARRAY_DICT_INIT;
                if hlgroup2dict(&raw mut attrs_1, ns_id, i, arena) {
                    let c2rust_fresh0 = rv.size;
                    rv.size = rv.size.wrapping_add(1);
                    *rv.items.offset(c2rust_fresh0 as isize) = key_value_pair {
                        key: cstr_as_string(
                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(
                                ((if link as ::core::ffi::c_int != 0 {
                                    i
                                } else {
                                    syn_get_final_id(i)
                                }) - 1 as ::core::ffi::c_int)
                                    as isize,
                            ))
                            .sg_name,
                        ),
                        value: object {
                            type_0: kObjectTypeDict,
                            data: C2Rust_Unnamed_0 { dict: attrs_1 },
                        },
                    };
                }
                i += 1;
            }
            return rv;
        }
        return ARRAY_DICT_INIT;
    }
}

pub unsafe extern "C" fn highlight_has_attr(
    id: ::core::ffi::c_int,
    flag: ::core::ffi::c_int,
    modec: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe {
        if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        let mut attr: ::core::ffi::c_int = 0;
        if modec == 'g' as ::core::ffi::c_int {
            attr = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_gui;
        } else {
            attr = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_cterm;
        }
        if flag & HL_UNDERLINE_MASK != 0 {
            let mut ul: ::core::ffi::c_int = attr & HL_UNDERLINE_MASK;
            return if ul == flag {
                b"1\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                ::core::ptr::null::<::core::ffi::c_char>()
            };
        } else {
            return if attr & flag != 0 {
                b"1\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                ::core::ptr::null::<::core::ffi::c_char>()
            };
        };
    }
}

pub unsafe extern "C" fn highlight_color(
    id: ::core::ffi::c_int,
    what: *const ::core::ffi::c_char,
    modec: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe {
        static name: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new([0; 20]);
        let mut fg: bool = false_0 != 0;
        let mut sp: bool = false_0 != 0;
        let mut font: bool = false_0 != 0;
        if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'f' as ::core::ffi::c_int
            && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'g' as ::core::ffi::c_int
        {
            fg = true_0 != 0;
        } else if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'f' as ::core::ffi::c_int
            && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'o' as ::core::ffi::c_int
            && (if (*what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'n' as ::core::ffi::c_int
            && (if (*what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 't' as ::core::ffi::c_int
        {
            font = true_0 != 0;
        } else if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 's' as ::core::ffi::c_int
            && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'p' as ::core::ffi::c_int
        {
            sp = true_0 != 0;
        } else if !((if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'b' as ::core::ffi::c_int
            && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'g' as ::core::ffi::c_int)
        {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        let mut n: ::core::ffi::c_int = 0;
        if modec == 'g' as ::core::ffi::c_int {
            if *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '#' as ::core::ffi::c_int
                && ui_rgb_attached() as ::core::ffi::c_int != 0
            {
                if fg {
                    n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_fg as ::core::ffi::c_int;
                } else if sp {
                    n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_sp as ::core::ffi::c_int;
                } else {
                    n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_bg as ::core::ffi::c_int;
                }
                if n < 0 as ::core::ffi::c_int || n > 0xffffff as ::core::ffi::c_int {
                    return ::core::ptr::null::<::core::ffi::c_char>();
                }
                snprintf(
                    name.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                    b"#%06x\0".as_ptr() as *const ::core::ffi::c_char,
                    n,
                );
                return name.ptr() as *mut ::core::ffi::c_char;
            }
            if fg {
                return coloridx_to_name(
                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_fg_idx,
                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_fg as ::core::ffi::c_int,
                    name.ptr() as *mut ::core::ffi::c_char,
                );
            } else if sp {
                return coloridx_to_name(
                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_sp_idx,
                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_sp as ::core::ffi::c_int,
                    name.ptr() as *mut ::core::ffi::c_char,
                );
            } else {
                return coloridx_to_name(
                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_bg_idx,
                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_rgb_bg as ::core::ffi::c_int,
                    name.ptr() as *mut ::core::ffi::c_char,
                );
            }
        }
        if font as ::core::ffi::c_int != 0 || sp as ::core::ffi::c_int != 0 {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if modec == 'c' as ::core::ffi::c_int {
            if fg {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_cterm_fg
                    - 1 as ::core::ffi::c_int;
            } else {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_cterm_bg
                    - 1 as ::core::ffi::c_int;
            }
            if n < 0 as ::core::ffi::c_int {
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                n,
            );
            return name.ptr() as *mut ::core::ffi::c_char;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}
