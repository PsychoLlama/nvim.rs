//! The tab line -- `draw_tabline()` and its `ext_tabline` form.
//!
//! [`draw_tabline`] is the default (non-`'tabline'`) rendering: one label per
//! tab page, each showing the modified marker, the window count and the
//! shortened buffer name of its current window, truncated to share the width
//! evenly, with the tab-page click definitions recorded as it goes.
//! [`ui_ext_tabline_update`] is the same information as data, pushed to a UI
//! that has taken over the tab line.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{
    arena_array, arena_dict, arena_string, cstr_as_string,
};
use crate::src::nvim::charset::{ptr2cells, vim_strsize};
use crate::src::nvim::grid::{
    grid_line_fill, grid_line_flush, grid_line_put_schar, grid_line_puts, grid_line_start,
};
use crate::src::nvim::highlight::hl_combine_attr;
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight_group::{HLF_T, HLF_TP, HLF_TPF, HLF_TPS};
use crate::src::nvim::main::{
    Columns, NameBuff, curbuf, curtab, curwin, default_grid, default_gridview, first_tabpage,
    firstbuf, firstwin, hl_attr_active, p_sc, p_sloc, p_tal, redraw_tabline, showcmd_buf, t_colors,
    tab_page_click_defs, tab_page_click_defs_size, topframe,
};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::src::nvim::path::shorten_dir;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::ui::kUITabline;
use crate::src::nvim::types::{
    Arena, Array, Buffer, Dict, Integer, StlClickDefinition, Tabpage, buf_T, kObjectTypeBuffer,
    kObjectTypeDict, kObjectTypeString, kObjectTypeTabpage, key_value_pair, object,
    object_data as C2Rust_Unnamed, schar_T, size_t, tabpage_T, win_T,
};
use crate::src::nvim::ui::{ui_call_tabline_update, ui_has};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::window::tabline_height;

unsafe extern "C" fn ui_ext_tabline_update() {
    unsafe {
        let mut arena: Arena = ARENA_EMPTY;
        let mut n_tabs: size_t = 0 as size_t;
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            n_tabs = n_tabs.wrapping_add(1);
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        let mut tabs: Array = arena_array(&raw mut arena, n_tabs);
        let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_0.is_null() {
            let mut tab_info: Dict = arena_dict(&raw mut arena, 2 as size_t);
            let c2rust_fresh45 = tab_info.size;
            tab_info.size = tab_info.size.wrapping_add(1);
            *tab_info.items.add(c2rust_fresh45) = key_value_pair {
                key: cstr_as_string(c"tab".as_ptr()),
                value: object {
                    type_0: kObjectTypeTabpage,
                    data: C2Rust_Unnamed {
                        integer: (*tp_0).handle as Integer,
                    },
                },
            };
            let mut cwp: *mut win_T = if tp_0 == curtab.get() {
                curwin.get()
            } else {
                (*tp_0).tp_curwin
            };
            get_trans_bufname((*cwp).w_buffer);
            let c2rust_fresh46 = tab_info.size;
            tab_info.size = tab_info.size.wrapping_add(1);
            *tab_info.items.add(c2rust_fresh46) = key_value_pair {
                key: cstr_as_string(c"name".as_ptr()),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_string(
                            &raw mut arena,
                            cstr_as_string(NameBuff.ptr() as *mut ::core::ffi::c_char),
                        ),
                    },
                },
            };
            let c2rust_fresh47 = tabs.size;
            tabs.size = tabs.size.wrapping_add(1);
            *tabs.items.add(c2rust_fresh47) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: tab_info },
            };
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
        let mut n_buffers: size_t = 0 as size_t;
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            n_buffers = n_buffers.wrapping_add(
                (if (*buf).b_p_bl != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as size_t,
            );
            buf = (*buf).b_next;
        }
        let mut buffers: Array = arena_array(&raw mut arena, n_buffers);
        let mut buf_0: *mut buf_T = firstbuf.get();
        while !buf_0.is_null() {
            if (*buf_0).b_p_bl != 0 {
                let mut buffer_info: Dict = arena_dict(&raw mut arena, 2 as size_t);
                let c2rust_fresh48 = buffer_info.size;
                buffer_info.size = buffer_info.size.wrapping_add(1);
                *buffer_info.items.add(c2rust_fresh48) = key_value_pair {
                    key: cstr_as_string(c"buffer".as_ptr()),
                    value: object {
                        type_0: kObjectTypeBuffer,
                        data: C2Rust_Unnamed {
                            integer: (*buf_0).handle as Integer,
                        },
                    },
                };
                get_trans_bufname(buf_0);
                let c2rust_fresh49 = buffer_info.size;
                buffer_info.size = buffer_info.size.wrapping_add(1);
                *buffer_info.items.add(c2rust_fresh49) = key_value_pair {
                    key: cstr_as_string(c"name".as_ptr()),
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: arena_string(
                                &raw mut arena,
                                cstr_as_string(NameBuff.ptr() as *mut ::core::ffi::c_char),
                            ),
                        },
                    },
                };
                let c2rust_fresh50 = buffers.size;
                buffers.size = buffers.size.wrapping_add(1);
                *buffers.items.add(c2rust_fresh50) = object {
                    type_0: kObjectTypeDict,
                    data: C2Rust_Unnamed { dict: buffer_info },
                };
            }
            buf_0 = (*buf_0).b_next;
        }
        ui_call_tabline_update(
            (*curtab.get()).handle as Tabpage,
            tabs,
            (*curbuf.get()).handle as Buffer,
            buffers,
        );
        arena_mem_free(arena_finish(&raw mut arena));
    }
}

pub unsafe extern "C" fn draw_tabline() {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut attr_nosel: ::core::ffi::c_int = *(*hl_attr_active.ptr()).offset(HLF_TP as isize);
        let mut attr_fill: ::core::ffi::c_int = *(*hl_attr_active.ptr()).offset(HLF_TPF as isize);
        let mut use_sep_chars: bool = t_colors.get() < 8 as ::core::ffi::c_int;
        if (*default_grid.ptr()).chars.is_null() {
            return;
        }
        redraw_tabline.set(false_0 != 0);
        if ui_has(kUITabline) {
            ui_ext_tabline_update();
            return;
        }
        if tabline_height() < 1 as ::core::ffi::c_int {
            return;
        }
        debug_assert!(
            tab_page_click_defs_size.get() >= Columns.get() as size_t,
            "tab_page_click_defs_size >= (size_t)Columns"
        );
        stl_clear_click_defs(tab_page_click_defs.get(), tab_page_click_defs_size.get());
        if *p_tal.get() as ::core::ffi::c_int != NUL {
            win_redr_custom(
                ::core::ptr::null_mut::<win_T>(),
                false_0 != 0,
                false_0 != 0,
                false_0 != 0,
            );
        } else {
            let mut tabcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut cwp: *mut win_T = ::core::ptr::null_mut::<win_T>();
            let mut wincount: ::core::ffi::c_int = 0;
            grid_line_start(default_gridview.ptr(), 0 as ::core::ffi::c_int);
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                tabcount += 1;
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            let mut tabwidth: ::core::ffi::c_int = if (if tabcount > 0 as ::core::ffi::c_int {
                (Columns.get() - 1 as ::core::ffi::c_int + tabcount / 2 as ::core::ffi::c_int)
                    / tabcount
            } else {
                0 as ::core::ffi::c_int
            }) > 6 as ::core::ffi::c_int
            {
                if tabcount > 0 as ::core::ffi::c_int {
                    (Columns.get() - 1 as ::core::ffi::c_int + tabcount / 2 as ::core::ffi::c_int)
                        / tabcount
                } else {
                    0 as ::core::ffi::c_int
                }
            } else {
                6 as ::core::ffi::c_int
            };
            let mut attr: ::core::ffi::c_int = attr_nosel;
            tabcount = 0 as ::core::ffi::c_int;
            let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp_0.is_null() {
                if col >= Columns.get() - 4 as ::core::ffi::c_int {
                    break;
                }
                let mut scol: ::core::ffi::c_int = col;
                if tp_0 == curtab.get() {
                    cwp = curwin.get();
                    wp = firstwin.get();
                } else {
                    cwp = (*tp_0).tp_curwin;
                    wp = (*tp_0).tp_firstwin;
                }
                if (*tp_0).tp_topframe == topframe.get() {
                    attr = win_hl_attr(cwp, HLF_TPS);
                }
                if use_sep_chars as ::core::ffi::c_int != 0 && col > 0 as ::core::ffi::c_int {
                    let c2rust_fresh39 = col;
                    col = col + 1;
                    grid_line_put_schar(c2rust_fresh39, '|' as ::core::ffi::c_int as schar_T, attr);
                }
                if (*tp_0).tp_topframe != topframe.get() {
                    attr = win_hl_attr(cwp, HLF_TP);
                }
                let c2rust_fresh40 = col;
                col = col + 1;
                grid_line_put_schar(c2rust_fresh40, ' ' as ::core::ffi::c_int as schar_T, attr);
                let mut modified: bool = false_0 != 0;
                wincount = 0 as ::core::ffi::c_int;
                while !wp.is_null() {
                    if !(*wp).w_config.focusable || (*wp).w_config.hide as ::core::ffi::c_int != 0 {
                        wincount -= 1;
                    } else if bufIsChanged((*wp).w_buffer) {
                        modified = true_0 != 0;
                    }
                    wp = (*wp).w_next;
                    wincount += 1;
                }
                if modified as ::core::ffi::c_int != 0 || wincount > 1 as ::core::ffi::c_int {
                    if wincount > 1 as ::core::ffi::c_int {
                        let mut len: ::core::ffi::c_int = vim_snprintf(
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            c"%d".as_ptr(),
                            wincount,
                        );
                        if col + len >= Columns.get() - 3 as ::core::ffi::c_int {
                            break;
                        }
                        grid_line_puts(
                            col,
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                            len,
                            hl_combine_attr(attr, win_hl_attr(cwp, HLF_T)),
                        );
                        col += len;
                    }
                    if modified {
                        let c2rust_fresh41 = col;
                        col = col + 1;
                        grid_line_put_schar(
                            c2rust_fresh41,
                            '+' as ::core::ffi::c_int as schar_T,
                            attr,
                        );
                    }
                    let c2rust_fresh42 = col;
                    col = col + 1;
                    grid_line_put_schar(c2rust_fresh42, ' ' as ::core::ffi::c_int as schar_T, attr);
                }
                let mut room: ::core::ffi::c_int = scol - col + tabwidth - 1 as ::core::ffi::c_int;
                if room > 0 as ::core::ffi::c_int {
                    get_trans_bufname((*cwp).w_buffer);
                    shorten_dir(NameBuff.ptr() as *mut ::core::ffi::c_char);
                    let mut len_0: ::core::ffi::c_int =
                        vim_strsize(NameBuff.ptr() as *mut ::core::ffi::c_char);
                    let mut p: *mut ::core::ffi::c_char =
                        NameBuff.ptr() as *mut ::core::ffi::c_char;
                    while len_0 > room {
                        len_0 -= ptr2cells(p);
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    let mut n: ::core::ffi::c_int = Columns.get() - col - 1 as ::core::ffi::c_int;
                    len_0 = if len_0 < n { len_0 } else { n };
                    grid_line_puts(col, p, -1 as ::core::ffi::c_int, attr);
                    col += len_0;
                }
                let c2rust_fresh43 = col;
                col = col + 1;
                grid_line_put_schar(c2rust_fresh43, ' ' as ::core::ffi::c_int as schar_T, attr);
                tabcount += 1;
                while scol < col {
                    let c2rust_fresh44 = scol;
                    scol = scol + 1;
                    *(*tab_page_click_defs.ptr()).offset(c2rust_fresh44 as isize) =
                        StlClickDefinition {
                            type_0: kStlClickTabSwitch,
                            tabnr: tabcount,
                            func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        };
                }
                tp_0 = (*tp_0).tp_next as *mut tabpage_T;
            }
            let mut scol_0: ::core::ffi::c_int = col;
            while scol_0 < Columns.get() {
                *(*tab_page_click_defs.ptr()).offset(scol_0 as isize) = StlClickDefinition {
                    type_0: kStlClickTabSwitch,
                    tabnr: 0 as ::core::ffi::c_int,
                    func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
                scol_0 += 1;
            }
            let mut c: ::core::ffi::c_char = (if use_sep_chars as ::core::ffi::c_int != 0 {
                '_' as ::core::ffi::c_int
            } else {
                ' ' as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
            grid_line_fill(col, Columns.get(), c as schar_T, attr_fill);
            if p_sc.get() != 0 && *p_sloc.get() as ::core::ffi::c_int == 't' as ::core::ffi::c_int {
                let mut n_0: ::core::ffi::c_int = Columns.get()
                    - col
                    - (tabcount > 1 as ::core::ffi::c_int) as ::core::ffi::c_int
                        * 3 as ::core::ffi::c_int;
                let sc_width: ::core::ffi::c_int = if (10 as ::core::ffi::c_int) < n_0 {
                    10 as ::core::ffi::c_int
                } else {
                    n_0
                };
                if sc_width > 0 as ::core::ffi::c_int {
                    grid_line_puts(
                        Columns.get()
                            - sc_width
                            - (tabcount > 1 as ::core::ffi::c_int) as ::core::ffi::c_int
                                * 2 as ::core::ffi::c_int,
                        showcmd_buf.ptr() as *mut ::core::ffi::c_char,
                        sc_width,
                        attr_nosel,
                    );
                }
            }
            if tabcount > 1 as ::core::ffi::c_int {
                grid_line_put_schar(
                    Columns.get() - 1 as ::core::ffi::c_int,
                    'X' as ::core::ffi::c_int as schar_T,
                    attr_nosel,
                );
                *(*tab_page_click_defs.ptr())
                    .offset((Columns.get() - 1 as ::core::ffi::c_int) as isize) =
                    StlClickDefinition {
                        type_0: kStlClickTabClose,
                        tabnr: 999 as ::core::ffi::c_int,
                        func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    };
            }
            grid_line_flush();
        }
        redraw_tabline.set(false_0 != 0);
    }
}
