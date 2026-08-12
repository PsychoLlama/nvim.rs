//! Drawing a user format -- `'statusline'`, `'winbar'` and the ruler.
//!
//! [`win_redr_custom`] is the shared renderer: it picks the format
//! (`'statusline'`, `'tabline'` or `'winbar'`), decides the width and the
//! fill character, calls `build_stl_str_hl` to expand it, then paints the
//! result with its highlight runs and records the click definitions.
//! [`win_redr_winbar`] is the winbar entry point, and [`redraw_ruler`] the
//! `'ruler'`/`'rulerformat'` one -- which draws into the command line rather
//! than a status line when the window has none, and forwards to the UI as
//! `msg_ruler` when `ext_messages` is on.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{api_free_array, cstr_as_string};
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::buffer::{col_print, get_rel_pos};
use crate::src::nvim::charset::{transstr_buf, vim_strsize};
use crate::src::nvim::drawscreen::redrawing;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    grid_adjust, grid_line_fill, grid_line_flush, grid_line_puts, grid_line_start, schar_get,
    screengrid_line_start,
};
use crate::src::nvim::highlight::hl_combine_attr;
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight_group::{
    HLF_MSG, HLF_NONE, HLF_TPF, HLF_WBR, HLF_WBRNC, syn_id2attr, syn_name2id_len,
};
use crate::src::nvim::main::{
    Columns, Rows, State, curwin, default_grid, edit_submode, highlight_stlnc, highlight_user,
    hl_attr_active, msg_col, msg_grid_adj, msg_row, p_ch, p_ru, p_ruf, p_stl, p_tal, p_wbr, ru_col,
    tab_page_click_defs,
};
use crate::src::nvim::mbyte::{utf_ptr2cells, utfc_ptr2len};
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::memory::{xfree, xmemdupz, xrealloc, xstrdup};
use crate::src::nvim::message::msg_clr_eos;
use crate::src::nvim::options::{
    kOptInvalid, kOptRulerformat, kOptStatusline, kOptTabline, kOptWinbar,
};
use crate::src::nvim::os::libc::{atoi, gettext, strlen};
use crate::src::nvim::plines::getvvcol;
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    Array, Integer, Object, OptIndex, OptInt, ScreenGrid, StlClickDefinition, StlClickRecord,
    String_0, colnr_T, hlf_T, int64_t, kObjectTypeArray, kObjectTypeInteger, kObjectTypeNil,
    kObjectTypeString, object, object_data as C2Rust_Unnamed, schar_T, size_t, ssize_t,
    statuscol_T, stl_hlrec_t, tabpage_T, win_T,
};
use crate::src::nvim::ui::{ui_call_msg_ruler, ui_has};
use crate::src::nvim::window::{global_stl_height, lastwin_nofloating};

static did_show_ext_ruler: GlobalCell<bool> = GlobalCell::new(false_0 != 0);

pub(crate) unsafe extern "C" fn win_redr_custom(
    mut wp: *mut win_T,
    mut draw_winbar: bool,
    mut draw_ruler: bool,
    mut ui_event: bool,
) {
    unsafe {
        let mut ewp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut p_crb_save: ::core::ffi::c_int = 0;
        let mut len: ::core::ffi::c_int = 0;
        let mut start_col: ::core::ffi::c_int = 0;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut curattr: ::core::ffi::c_int = 0;
        let mut curgroup: ::core::ffi::c_int = 0;
        let mut content: Array = Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut maxcol: ::core::ffi::c_int = 0;
        let mut click_defs: *mut StlClickDefinition = ::core::ptr::null_mut::<StlClickDefinition>();
        static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut attr: ::core::ffi::c_int = 0;
        let mut row: ::core::ffi::c_int = 0;
        let mut maxwidth: ::core::ffi::c_int = 0;
        let mut group: hlf_T = HLF_NONE;
        let mut fillchar: schar_T = 0;
        let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut transbuf: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut stl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut opt_idx: OptIndex = kOptInvalid;
        let mut opt_scope: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut hltab: *mut stl_hlrec_t = ::core::ptr::null_mut::<stl_hlrec_t>();
        let mut tabtab: *mut StlClickRecord = ::core::ptr::null_mut::<StlClickRecord>();
        let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
        let mut grid: *mut ScreenGrid =
            if !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 && !is_stl_global {
                &raw mut (*wp).w_grid_alloc
            } else {
                default_grid.ptr()
            };
        if entered.get() {
            return;
        }
        entered.set(true_0 != 0);
        '_theend: {
            if wp.is_null() {
                stl = p_tal.get();
                row = 0 as ::core::ffi::c_int;
                fillchar = ' ' as ::core::ffi::c_int as schar_T;
                group = HLF_TPF;
                attr = *(*hl_attr_active.ptr()).offset(group as ::core::ffi::c_int as isize);
                maxwidth = Columns.get();
                opt_idx = kOptTabline;
            } else if draw_winbar {
                opt_idx = kOptWinbar;
                stl = if *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL {
                    (*wp).w_onebuf_opt.wo_wbr
                } else {
                    p_wbr.get()
                };
                opt_scope = if *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL {
                    OPT_LOCAL as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                row = -1 as ::core::ffi::c_int;
                col = 0 as ::core::ffi::c_int;
                grid = grid_adjust(&raw mut (*wp).w_grid, &raw mut row, &raw mut col);
                if row < 0 as ::core::ffi::c_int {
                    break '_theend;
                } else {
                    fillchar = (*wp).w_p_fcs_chars.wbr;
                    group = (if wp == curwin.get() {
                        HLF_WBR
                    } else {
                        HLF_WBRNC
                    }) as hlf_T;
                    attr = win_hl_attr(wp, group as ::core::ffi::c_int);
                    maxwidth = (*wp).w_view_width;
                    stl_clear_click_defs((*wp).w_winbar_click_defs, (*wp).w_winbar_click_defs_size);
                    (*wp).w_winbar_click_defs = stl_alloc_click_defs(
                        (*wp).w_winbar_click_defs,
                        maxwidth,
                        &raw mut (*wp).w_winbar_click_defs_size,
                    );
                }
            } else {
                let in_status_line: bool = (*wp).w_status_height != 0 as ::core::ffi::c_int
                    || is_stl_global as ::core::ffi::c_int != 0;
                if (*wp).w_floating as ::core::ffi::c_int != 0 && !is_stl_global && !draw_ruler {
                    row = (*wp).w_winrow_off + (*wp).w_view_height;
                    col = (*wp).w_wincol_off;
                    maxwidth = (*wp).w_view_width;
                } else {
                    row = if is_stl_global as ::core::ffi::c_int != 0 {
                        Rows.get() - p_ch.get() as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    } else {
                        (*wp).w_winrow + (*wp).w_height
                    };
                    maxwidth = if in_status_line as ::core::ffi::c_int != 0 && !is_stl_global {
                        (*wp).w_width
                    } else {
                        Columns.get()
                    };
                }
                fillchar = fillchar_status(&raw mut group, wp);
                stl_clear_click_defs((*wp).w_status_click_defs, (*wp).w_status_click_defs_size);
                (*wp).w_status_click_defs = stl_alloc_click_defs(
                    (*wp).w_status_click_defs,
                    maxwidth,
                    &raw mut (*wp).w_status_click_defs_size,
                );
                if draw_ruler {
                    stl = p_ruf.get();
                    opt_idx = kOptRulerformat;
                    if *stl as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                        stl = stl.offset(1);
                        if *stl as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                            stl = stl.offset(1);
                        }
                        if atoi(stl) != 0 {
                            while ascii_isdigit(*stl as ::core::ffi::c_int) {
                                stl = stl.offset(1);
                            }
                        }
                        let c2rust_fresh0 = stl;
                        stl = stl.offset(1);
                        if *c2rust_fresh0 as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
                            stl = p_ruf.get();
                        }
                    }
                    col = if ru_col.get() - (Columns.get() - maxwidth)
                        > (maxwidth + 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int
                    {
                        ru_col.get() - (Columns.get() - maxwidth)
                    } else {
                        (maxwidth + 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int
                    };
                    maxwidth -= col;
                    if !in_status_line {
                        row = Rows.get() - 1 as ::core::ffi::c_int;
                        grid = grid_adjust(msg_grid_adj.ptr(), &raw mut row, &raw mut col);
                        maxwidth -= 1;
                        fillchar = ' ' as ::core::ffi::c_int as schar_T;
                        group = HLF_MSG;
                    }
                } else {
                    opt_idx = kOptStatusline;
                    stl = if *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL {
                        (*wp).w_onebuf_opt.wo_stl
                    } else {
                        p_stl.get()
                    };
                    opt_scope = if *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL {
                        OPT_LOCAL as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                }
                attr = win_hl_attr(wp, group as ::core::ffi::c_int);
                if !(*wp).w_floating && in_status_line as ::core::ffi::c_int != 0 && !is_stl_global
                {
                    col += (*wp).w_wincol;
                }
            }
            if maxwidth > 0 as ::core::ffi::c_int {
                ewp = if wp.is_null() { curwin.get() } else { wp };
                p_crb_save = (*ewp).w_onebuf_opt.wo_crb;
                (*ewp).w_onebuf_opt.wo_crb = false_0;
                stl = xstrdup(stl);
                build_stl_str_hl(
                    ewp,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                    stl,
                    opt_idx,
                    opt_scope,
                    fillchar,
                    maxwidth,
                    &raw mut hltab,
                    ::core::ptr::null_mut::<size_t>(),
                    &raw mut tabtab,
                    ::core::ptr::null_mut::<statuscol_T>(),
                );
                xfree(stl as *mut ::core::ffi::c_void);
                (*ewp).w_onebuf_opt.wo_crb = p_crb_save;
                len = strlen(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
                start_col = col;
                if !ui_event {
                    screengrid_line_start(grid, row, 0 as ::core::ffi::c_int);
                }
                p = &raw mut buf as *mut ::core::ffi::c_char;
                curattr = attr;
                curgroup = group as ::core::ffi::c_int;
                content = ARRAY_DICT_INIT;
                let mut sp: *mut stl_hlrec_t = hltab;
                loop {
                    let mut textlen: ::core::ffi::c_int = (if !(*sp).start.is_null() {
                        (*sp).start.offset_from(p)
                    } else {
                        (&raw mut buf as *mut ::core::ffi::c_char)
                            .offset(len as isize)
                            .offset_from(p)
                    })
                        as ::core::ffi::c_int;
                    let mut tsize: size_t = transstr_buf(
                        if p >= (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize) {
                            c"".as_ptr()
                        } else {
                            p as *const ::core::ffi::c_char
                        },
                        textlen as ssize_t,
                        &raw mut transbuf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                        true_0 != 0,
                    );
                    if !ui_event {
                        col += grid_line_puts(
                            col,
                            &raw mut transbuf as *mut ::core::ffi::c_char,
                            tsize as ::core::ffi::c_int,
                            curattr,
                        );
                    } else {
                        let mut chunk: Array = ARRAY_DICT_INIT;
                        if chunk.size == chunk.capacity {
                            chunk.capacity = if chunk.capacity != 0 {
                                chunk.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            chunk.items = xrealloc(
                                chunk.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh1 = chunk.size;
                        chunk.size = chunk.size.wrapping_add(1);
                        *chunk.items.add(c2rust_fresh1) = object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed {
                                integer: curattr as Integer,
                            },
                        };
                        if chunk.size == chunk.capacity {
                            chunk.capacity = if chunk.capacity != 0 {
                                chunk.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            chunk.items = xrealloc(
                                chunk.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh2 = chunk.size;
                        chunk.size = chunk.size.wrapping_add(1);
                        *chunk.items.add(c2rust_fresh2) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: String_0 {
                                    data: xmemdupz(
                                        &raw mut transbuf as *mut ::core::ffi::c_char
                                            as *const ::core::ffi::c_void,
                                        tsize,
                                    )
                                        as *mut ::core::ffi::c_char,
                                    size: tsize,
                                },
                            },
                        };
                        if chunk.size == chunk.capacity {
                            chunk.capacity = if chunk.capacity != 0 {
                                chunk.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            chunk.items = xrealloc(
                                chunk.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh3 = chunk.size;
                        chunk.size = chunk.size.wrapping_add(1);
                        *chunk.items.add(c2rust_fresh3) = object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed {
                                integer: curgroup as Integer,
                            },
                        };
                        if content.size == content.capacity {
                            content.capacity = if content.capacity != 0 {
                                content.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            content.items = xrealloc(
                                content.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>().wrapping_mul(content.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh4 = content.size;
                        content.size = content.size.wrapping_add(1);
                        *content.items.add(c2rust_fresh4) = object {
                            type_0: kObjectTypeArray,
                            data: C2Rust_Unnamed { array: chunk },
                        };
                    }
                    p = (*sp).start;
                    if p.is_null() {
                        break;
                    }
                    if (*sp).userhl == 0 as ::core::ffi::c_int {
                        curattr = attr;
                        curgroup = group as ::core::ffi::c_int;
                    } else if (*sp).userhl < 0 as ::core::ffi::c_int {
                        let mut new_attr: ::core::ffi::c_int = syn_id2attr(-(*sp).userhl);
                        if (*sp).item as ::core::ffi::c_uint
                            == STL_HIGHLIGHT_COMB as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            curattr = hl_combine_attr(curattr, new_attr);
                        } else {
                            curattr = new_attr;
                        }
                        curgroup = -(*sp).userhl;
                    } else {
                        let mut userhl: *mut ::core::ffi::c_int = if !wp.is_null()
                            && wp != curwin.get()
                            && (*wp).w_status_height != 0 as ::core::ffi::c_int
                        {
                            highlight_stlnc.ptr() as *mut ::core::ffi::c_int
                        } else {
                            highlight_user.ptr() as *mut ::core::ffi::c_int
                        };
                        let mut userbuf: [::core::ffi::c_char; 5] =
                            ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"User\0");
                        userbuf[4 as ::core::ffi::c_int as usize] =
                            ((*sp).userhl as ::core::ffi::c_char as ::core::ffi::c_int
                                + '0' as ::core::ffi::c_int)
                                as ::core::ffi::c_char;
                        curattr = *userhl.offset(((*sp).userhl - 1 as ::core::ffi::c_int) as isize);
                        curgroup = syn_name2id_len(
                            &raw mut userbuf as *mut ::core::ffi::c_char,
                            5 as size_t,
                        );
                    }
                    if curattr != attr {
                        curattr = hl_combine_attr(attr, curattr);
                    }
                    sp = sp.offset(1);
                }
                if ui_event {
                    ui_call_msg_ruler(content);
                    did_show_ext_ruler.set(true_0 != 0);
                    api_free_array(content);
                } else {
                    maxcol = start_col + maxwidth;
                    grid_line_fill(col, maxcol, fillchar, curattr);
                    grid_line_flush();
                    click_defs = if wp.is_null() {
                        tab_page_click_defs.get()
                    } else if draw_winbar as ::core::ffi::c_int != 0 {
                        (*wp).w_winbar_click_defs
                    } else {
                        (*wp).w_status_click_defs
                    };
                    stl_fill_click_defs(
                        click_defs,
                        tabtab,
                        &raw mut buf as *mut ::core::ffi::c_char,
                        maxwidth,
                        wp.is_null(),
                    );
                }
            }
        }
        entered.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn win_redr_winbar(mut wp: *mut win_T) {
    unsafe {
        static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if entered.get() {
            return;
        }
        entered.set(true_0 != 0);
        if !((*wp).w_winbar_height == 0 as ::core::ffi::c_int || !redrawing()) {
            if *p_wbr.get() as ::core::ffi::c_int != NUL
                || *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
            {
                win_redr_custom(wp, true_0 != 0, false_0 != 0, false_0 != 0);
            }
        }
        entered.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn redraw_ruler() {
    unsafe {
        static did_ruler_col: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(-1 as ::core::ffi::c_int);
        let mut wp: *mut win_T = if !is_aucmd_win(curwin.get())
            && (*curwin.get()).w_status_height == 0 as ::core::ffi::c_int
        {
            curwin.get()
        } else {
            lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>())
        };
        let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
        if p_ru.get() == 0
            || (*wp).w_status_height > 0 as ::core::ffi::c_int
            || is_stl_global as ::core::ffi::c_int != 0
            || p_ch.get() == 0 as OptInt && !ui_has(kUIMessages)
        {
            if did_show_ext_ruler.get() as ::core::ffi::c_int != 0
                && ui_has(kUIMessages) as ::core::ffi::c_int != 0
            {
                ui_call_msg_ruler(ARRAY_DICT_INIT);
                did_show_ext_ruler.set(false_0 != 0);
            } else if did_ruler_col.get() > 0 as ::core::ffi::c_int {
                msg_col.set(did_ruler_col.get());
                msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
                msg_clr_eos();
            }
            did_ruler_col.set(-1 as ::core::ffi::c_int);
            return;
        }
        if (*wp).w_cursor.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
            return;
        }
        if (*wp).w_status_height == 0 as ::core::ffi::c_int
            && !is_stl_global
            && !(*edit_submode.ptr()).is_null()
        {
            return;
        }
        let mut part_of_status: bool =
            (*wp).w_status_height != 0 || is_stl_global as ::core::ffi::c_int != 0;
        if *p_ruf.get() as ::core::ffi::c_int != 0
            && (p_ch.get() > 0 as OptInt
                || ui_has(kUIMessages) as ::core::ffi::c_int != 0 && !part_of_status)
        {
            win_redr_custom(wp, false_0 != 0, true_0 != 0, ui_has(kUIMessages));
            return;
        }
        let mut group: hlf_T = HLF_MSG;
        let mut off: ::core::ffi::c_int = if (*wp).w_status_height != 0 {
            (*wp).w_wincol
        } else {
            0 as ::core::ffi::c_int
        };
        let mut width: ::core::ffi::c_int = if (*wp).w_status_height != 0 {
            (*wp).w_width
        } else {
            Columns.get()
        };
        let mut fillchar: schar_T = if part_of_status as ::core::ffi::c_int != 0 {
            fillchar_status(&raw mut group, wp)
        } else {
            ' ' as ::core::ffi::c_int as schar_T
        };
        let mut attr: ::core::ffi::c_int = if part_of_status as ::core::ffi::c_int != 0 {
            win_hl_attr(wp, group as ::core::ffi::c_int)
        } else {
            *(*hl_attr_active.ptr()).offset(group as ::core::ffi::c_int as isize)
        };
        let mut virtcol: colnr_T = (*wp).w_virtcol;
        if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.tab1 == NUL as schar_T {
            (*wp).w_onebuf_opt.wo_list = false_0;
            getvvcol(
                wp,
                &raw mut (*wp).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut virtcol,
                ::core::ptr::null_mut::<colnr_T>(),
            );
            (*wp).w_onebuf_opt.wo_list = true_0;
        }
        let mut empty_line: ::core::ffi::c_int = (State.get() & MODE_INSERT
            == 0 as ::core::ffi::c_int
            && *ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum) as ::core::ffi::c_int == NUL)
            as ::core::ffi::c_int;
        let mut buffer: [::core::ffi::c_char; 70] = [0; 70];
        let mut bufferlen: ::core::ffi::c_int = vim_snprintf(
            &raw mut buffer as *mut ::core::ffi::c_char,
            RULER_BUF_LEN as size_t,
            gettext(c"%ld,".as_ptr()),
            if (*(*wp).w_buffer).b_ml.ml_flags & ML_EMPTY != 0 {
                0 as int64_t
            } else {
                (*wp).w_cursor.lnum as int64_t
            },
        );
        bufferlen += col_print(
            (&raw mut buffer as *mut ::core::ffi::c_char).offset(bufferlen as isize),
            (RULER_BUF_LEN as size_t).wrapping_sub(bufferlen as size_t),
            if empty_line != 0 {
                0 as ::core::ffi::c_int
            } else {
                (*wp).w_cursor.col + 1 as ::core::ffi::c_int
            },
            virtcol + 1 as ::core::ffi::c_int,
        );
        let mut rel_pos: [::core::ffi::c_char; 70] = [0; 70];
        let mut rel_poslen: ::core::ffi::c_int = get_rel_pos(
            wp,
            &raw mut rel_pos as *mut ::core::ffi::c_char,
            RULER_BUF_LEN,
        );
        let mut n1: ::core::ffi::c_int =
            bufferlen + vim_strsize(&raw mut rel_pos as *mut ::core::ffi::c_char);
        if (*wp).w_status_height == 0 as ::core::ffi::c_int && !is_stl_global {
            n1 += 1;
        }
        let mut this_ru_col: ::core::ffi::c_int = ru_col.get() - (Columns.get() - width);
        let mut n2: ::core::ffi::c_int =
            (width + 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
        this_ru_col = if this_ru_col > n2 { this_ru_col } else { n2 };
        if this_ru_col + n1 < width {
            while this_ru_col + n1 < width
                && RULER_BUF_LEN > bufferlen + rel_poslen + 1 as ::core::ffi::c_int
            {
                bufferlen += schar_get(
                    (&raw mut buffer as *mut ::core::ffi::c_char).offset(bufferlen as isize),
                    fillchar,
                ) as ::core::ffi::c_int;
                n1 += 1;
            }
            bufferlen += vim_snprintf(
                (&raw mut buffer as *mut ::core::ffi::c_char).offset(bufferlen as isize),
                (RULER_BUF_LEN as size_t).wrapping_sub(bufferlen as size_t),
                c"%s".as_ptr(),
                &raw mut rel_pos as *mut ::core::ffi::c_char,
            );
        }
        if ui_has(kUIMessages) as ::core::ffi::c_int != 0 && !part_of_status {
            let mut content: Array = ARRAY_DICT_INIT;
            let mut content__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 1];
            content.capacity = 1 as size_t;
            content.items = &raw mut content__items as *mut Object;
            let mut chunk: Array = ARRAY_DICT_INIT;
            let mut chunk__items: [Object; 3] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 3];
            chunk.capacity = 3 as size_t;
            chunk.items = &raw mut chunk__items as *mut Object;
            let c2rust_fresh35 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.add(c2rust_fresh35) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: attr as Integer,
                },
            };
            let c2rust_fresh36 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.add(c2rust_fresh36) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(&raw mut buffer as *mut ::core::ffi::c_char),
                },
            };
            let c2rust_fresh37 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.add(c2rust_fresh37) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: HLF_MSG as Integer,
                },
            };
            debug_assert!(
                attr == *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                "attr == HL_ATTR(HLF_MSG)"
            );
            let c2rust_fresh38 = content.size;
            content.size = content.size.wrapping_add(1);
            *content.items.add(c2rust_fresh38) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: chunk },
            };
            ui_call_msg_ruler(content);
            did_show_ext_ruler.set(true_0 != 0);
            did_ruler_col.set(1 as ::core::ffi::c_int);
        } else {
            if did_show_ext_ruler.get() {
                ui_call_msg_ruler(ARRAY_DICT_INIT);
                did_show_ext_ruler.set(false_0 != 0);
            }
            n1 = 0 as ::core::ffi::c_int;
            n2 = 0 as ::core::ffi::c_int;
            while buffer[n1 as usize] as ::core::ffi::c_int != NUL {
                n2 += utf_ptr2cells(
                    (&raw mut buffer as *mut ::core::ffi::c_char).offset(n1 as isize),
                );
                if this_ru_col + n2 > width {
                    bufferlen = n1;
                    buffer[bufferlen as usize] = NUL as ::core::ffi::c_char;
                    break;
                } else {
                    n1 += utfc_ptr2len(
                        (&raw mut buffer as *mut ::core::ffi::c_char).offset(n1 as isize),
                    );
                }
            }
            grid_line_start(msg_grid_adj.ptr(), Rows.get() - 1 as ::core::ffi::c_int);
            did_ruler_col.set(off + this_ru_col);
            let mut w: ::core::ffi::c_int = grid_line_puts(
                did_ruler_col.get(),
                &raw mut buffer as *mut ::core::ffi::c_char,
                -1 as ::core::ffi::c_int,
                attr,
            );
            grid_line_fill(did_ruler_col.get() + w, off + width, fillchar, attr);
            grid_line_flush();
        };
    }
}
