//! `nvim_win_get_config()`: rendering a window's config back.
//!
//! The inverse of the parse: every field the config keyset can carry is read
//! off the `WinConfig` and packed into a Dict, including the border and its
//! title/footer -- which `config_put_bordertext` renders back as the
//! `[[text, hl], ..]` chunk arrays they were given as.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn config_put_bordertext(
    mut config: *mut KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut bordertext_type: BorderTextType,
    mut arena: *mut Arena,
) {
    unsafe {
        let mut vt: VirtText = VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        };
        let mut align: AlignTextPos = kAlignLeft;
        match bordertext_type as ::core::ffi::c_uint {
            0 => {
                vt = (*fconfig).title_chunks;
                align = (*fconfig).title_pos;
            }
            1 => {
                vt = (*fconfig).footer_chunks;
                align = (*fconfig).footer_pos;
            }
            _ => {}
        }
        let mut bordertext: Array = virt_text_to_array(vt, true, arena);
        let mut pos: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match align as ::core::ffi::c_uint {
            0 => {
                pos = c"left".as_ptr() as *mut ::core::ffi::c_char;
            }
            1 => {
                pos = c"center".as_ptr() as *mut ::core::ffi::c_char;
            }
            2 => {
                pos = c"right".as_ptr() as *mut ::core::ffi::c_char;
            }
            _ => {}
        }
        match bordertext_type as ::core::ffi::c_uint {
            0 => {
                (*config).is_set__win_config_ = ((*config).is_set__win_config_
                    as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title)
                    as OptionalKeys;
                (*config).title = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: bordertext },
                };
                (*config).is_set__win_config_ = ((*config).is_set__win_config_
                    as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title_pos)
                    as OptionalKeys;
                (*config).title_pos = cstr_as_string(pos);
            }
            1 => {
                (*config).is_set__win_config_ = ((*config).is_set__win_config_
                    as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer)
                    as OptionalKeys;
                (*config).footer = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: bordertext },
                };
                (*config).is_set__win_config_ = ((*config).is_set__win_config_
                    as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer_pos)
                    as OptionalKeys;
                (*config).footer_pos = cstr_as_string(pos);
            }
            _ => {}
        };
    }
}

pub unsafe extern "C" fn nvim_win_get_config(
    mut win: Window,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_win_config {
    unsafe {
        static float_relative_str: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
            c"editor".as_ptr(),
            c"win".as_ptr(),
            c"cursor".as_ptr(),
            c"mouse".as_ptr(),
            c"tabline".as_ptr(),
            c"laststatus".as_ptr(),
        ]);
        static win_split_str: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
            c"left".as_ptr(),
            c"right".as_ptr(),
            c"above".as_ptr(),
            c"below".as_ptr(),
        ]);
        static win_style_str: GlobalCell<[*const ::core::ffi::c_char; 2]> =
            GlobalCell::new([c"".as_ptr(), c"minimal".as_ptr()]);
        let mut rv: KeyDict_win_config = KEYDICT_INIT;
        let mut wp: *mut win_T = find_window_by_handle(win, err);
        if wp.is_null() {
            return rv;
        }
        let mut config: *mut WinConfig = &raw mut (*wp).w_config;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__focusable)
            as OptionalKeys;
        rv.focusable = (*config).focusable as Boolean;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__external)
            as OptionalKeys;
        rv.external = (*config).external as Boolean;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__hide)
            as OptionalKeys;
        rv.hide = (*config).hide as Boolean;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__mouse)
            as OptionalKeys;
        rv.mouse = (*config).mouse as Boolean;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__style)
            as OptionalKeys;
        rv.style = cstr_as_string((*win_style_str.ptr())[(*config).style as usize]);
        if (*wp).w_floating {
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width)
                as OptionalKeys;
            rv.width = (*config).width as Integer;
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height)
                as OptionalKeys;
            rv.height = (*config).height as Integer;
            if !(*config).external {
                if (*config).relative as ::core::ffi::c_uint
                    == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win)
                        as OptionalKeys;
                    rv.win = (*config).window;
                    if (*config).bufpos.lnum >= 0 as linenr_T {
                        let mut pos: Array = arena_array(arena, 2 as size_t);
                        let c2rust_fresh2 = pos.size;
                        pos.size = pos.size.wrapping_add(1);
                        *pos.items.offset(c2rust_fresh2 as isize) = object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed {
                                integer: (*config).bufpos.lnum as Integer,
                            },
                        };
                        let c2rust_fresh3 = pos.size;
                        pos.size = pos.size.wrapping_add(1);
                        *pos.items.offset(c2rust_fresh3 as isize) = object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed {
                                integer: (*config).bufpos.col as Integer,
                            },
                        };
                        rv.is_set__win_config_ = (rv.is_set__win_config_
                            as ::core::ffi::c_ulonglong
                            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__bufpos)
                            as OptionalKeys;
                        rv.bufpos = pos;
                    }
                }
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__anchor)
                    as OptionalKeys;
                rv.anchor = cstr_as_string(
                    *(&raw const float_anchor_str as *const *const ::core::ffi::c_char)
                        .offset((*config).anchor as isize),
                );
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__row)
                    as OptionalKeys;
                rv.row = (*config).row as Float;
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__col)
                    as OptionalKeys;
                rv.col = (*config).col as Float;
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__zindex)
                    as OptionalKeys;
                rv.zindex = (*config).zindex as Integer;
            }
            if (*config).border {
                let mut border: Array = arena_array(arena, 8 as size_t);
                let mut i: size_t = 0 as size_t;
                while i < 8 as size_t {
                    let mut s: String_0 = cstrn_as_string(
                        &raw mut *(&raw mut (*config).border_chars
                            as *mut [::core::ffi::c_char; 32])
                            .offset(i as isize) as *mut ::core::ffi::c_char,
                        MAX_SCHAR_SIZE as size_t,
                    );
                    let mut hi_id: ::core::ffi::c_int = (*config).border_hl_ids[i as usize];
                    let mut hi_name: *mut ::core::ffi::c_char = syn_id2name(hi_id);
                    if *hi_name.offset(0 as ::core::ffi::c_int as isize) != 0 {
                        let mut tuple: Array = arena_array(arena, 2 as size_t);
                        let c2rust_fresh4 = tuple.size;
                        tuple.size = tuple.size.wrapping_add(1);
                        *tuple.items.offset(c2rust_fresh4 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed { string: s },
                        };
                        let c2rust_fresh5 = tuple.size;
                        tuple.size = tuple.size.wrapping_add(1);
                        *tuple.items.offset(c2rust_fresh5 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: cstr_as_string(hi_name),
                            },
                        };
                        let c2rust_fresh6 = border.size;
                        border.size = border.size.wrapping_add(1);
                        *border.items.offset(c2rust_fresh6 as isize) = object {
                            type_0: kObjectTypeArray,
                            data: C2Rust_Unnamed { array: tuple },
                        };
                    } else {
                        let c2rust_fresh7 = border.size;
                        border.size = border.size.wrapping_add(1);
                        *border.items.offset(c2rust_fresh7 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed { string: s },
                        };
                    }
                    i = i.wrapping_add(1);
                }
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border)
                    as OptionalKeys;
                rv.border = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: border },
                };
                if (*config).title {
                    config_put_bordertext(&raw mut rv, config, kBorderTextTitle, arena);
                }
                if (*config).footer {
                    config_put_bordertext(&raw mut rv, config, kBorderTextFooter, arena);
                }
            } else {
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border)
                    as OptionalKeys;
                rv.border = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(c"none".as_ptr()),
                    },
                };
            }
        } else if !(*config).external {
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width)
                as OptionalKeys;
            rv.width = (*wp).w_width as Integer;
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height)
                as OptionalKeys;
            rv.height = (*wp).w_height as Integer;
            let mut split: WinSplit = win_split_dir(wp);
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split)
                as OptionalKeys;
            rv.split = cstr_as_string((*win_split_str.ptr())[split as usize]);
        }
        let mut rel: *const ::core::ffi::c_char =
            if (*wp).w_floating as ::core::ffi::c_int != 0 && !(*config).external {
                (*float_relative_str.ptr())[(*config).relative as usize]
            } else {
                c"".as_ptr()
            };
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__relative)
            as OptionalKeys;
        rv.relative = cstr_as_string(rel);
        if (*config)._cmdline_offset < INT_MAX {
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config___cmdline_offset)
                as OptionalKeys;
            rv._cmdline_offset = (*config)._cmdline_offset as Integer;
        }
        return rv;
    }
}
