use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_set_error, find_buffer_by_handle, find_window_by_handle,
};
use crate::src::nvim::api::vim::nvim_create_buf;
use crate::src::nvim::autocmd::{block_autocmds, unblock_autocmds};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later, set_must_redraw};
use crate::src::nvim::options::kOptBufhidden;

use crate::src::nvim::grid::grid_adjust;
use crate::src::nvim::main::{
    Columns, Rows, cmdwin_win, curtab, curwin, e_cmdwin, empty_string_option, firstwin, lastwin,
    mouse_col, mouse_grid, mouse_row, p_ch, p_ls, prevwin,
};
use crate::src::nvim::memory::{xfree, xrealloc, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::mouse::mouse_find_win_inner;
use crate::src::nvim::r#move::textpos2screenpos;
use crate::src::nvim::option::{parse_winhl_opt, set_option_direct_for};
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::libc::{memcmp, qsort, strlen};
use crate::src::nvim::strings::concat_str;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    AlignTextPos, Buffer, Error, FloatAnchor, OptInt, OptScope, OptVal, OptValData, OptValType,
    String_0, VirtText, VirtTextChunk, WinConfig, WinSplit, WinStyle, Window, buf_T, colnr_T,
    frame_T, kErrorTypeException, kErrorTypeNone, kFloatRelativeCursor, kFloatRelativeEditor,
    kFloatRelativeLaststatus, kFloatRelativeMouse, kFloatRelativeWindow, linenr_T, lpos_T, pos_T,
    schar_T, scid_T, size_t, tabpage_T, win_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::window::{
    last_status, lastwin_nofloating, merge_win_config, tabpage_win_valid, win_alloc, win_append,
    win_close, win_comp_pos, win_enter, win_find_tabpage, win_free, win_init, win_remove,
    win_remove_status_line, win_set_buf, win_set_inner_size, win_valid, winframe_remove,
};
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const kZIndexMessages: C2Rust_Unnamed_12 = 200;
pub const kZIndexFloatDefault: C2Rust_Unnamed_12 = 50;
pub const kOptValTypeString: OptValType = 2;
pub const kOptScopeBuf: OptScope = 2;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_14 = 2;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const STATUS_HEIGHT: C2Rust_Unnamed_15 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_16 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut win_T,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: C2Rust_Unnamed_16 = C2Rust_Unnamed_16 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<*mut win_T>(),
};
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub unsafe extern "C" fn win_new_float(
    mut wp: *mut win_T,
    mut last: bool,
    mut fconfig: WinConfig,
    mut err: *mut Error,
) -> *mut win_T {
    if wp.is_null() {
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        let mut tp_last: *mut win_T = if last as ::core::ffi::c_int != 0 {
            lastwin.get()
        } else {
            lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>())
        };
        if fconfig.window != 0 as ::core::ffi::c_int {
            debug_assert!(!last, "!last");
            let mut parent_wp: *mut win_T = find_window_by_handle(fconfig.window, err);
            if parent_wp.is_null() {
                return ::core::ptr::null_mut::<win_T>();
            }
            tp = win_find_tabpage(parent_wp);
            if tp.is_null() {
                return ::core::ptr::null_mut::<win_T>();
            }
            tp_last = lastwin_nofloating(if tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                tp
            });
        }
        wp = win_alloc(tp_last, false_0 != 0);
        win_init(wp, curwin.get(), 0 as ::core::ffi::c_int);
        if !(*wp).w_onebuf_opt.wo_wbr.is_null() && fconfig.height == 1 as ::core::ffi::c_int {
            if (*wp).w_onebuf_opt.wo_wbr != empty_string_option.ptr() as *mut ::core::ffi::c_char {
                free_string_option((*wp).w_onebuf_opt.wo_wbr);
            }
            (*wp).w_onebuf_opt.wo_wbr = empty_string_option.ptr() as *mut ::core::ffi::c_char;
        }
        if !(*wp).w_onebuf_opt.wo_stl.is_null()
            && (*wp).w_onebuf_opt.wo_stl != empty_string_option.ptr() as *mut ::core::ffi::c_char
        {
            free_string_option((*wp).w_onebuf_opt.wo_stl);
            (*wp).w_onebuf_opt.wo_stl = empty_string_option.ptr() as *mut ::core::ffi::c_char;
        }
    } else {
        debug_assert!(!last, "!last");
        debug_assert!(!(*wp).w_floating, "!wp->w_floating");
        let mut win_tp: *mut tabpage_T = win_find_tabpage(wp);
        debug_assert!(!win_tp.is_null(), "win_tp");
        if win_tp == curtab.get()
            && firstwin.get() == wp
            && lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()) == wp
            || win_tp != curtab.get()
                && (*win_tp).tp_firstwin == wp
                && lastwin_nofloating(win_tp) == wp
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"Cannot change last window into float\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<win_T>();
        } else if !(*cmdwin_win.ptr()).is_null() && !(*cmdwin_win.get()).w_floating {
            let mut other_nonfloat: bool = false_0 != 0;
            let mut wp2: *mut win_T = if win_tp == curtab.get() {
                firstwin.get()
            } else {
                (*win_tp).tp_firstwin
            };
            while !wp2.is_null() {
                if (*wp2).w_floating {
                    break;
                }
                if wp2 != wp && wp2 != cmdwin_win.get() {
                    other_nonfloat = true_0 != 0;
                    break;
                } else {
                    wp2 = (*wp2).w_next;
                }
            }
            if !other_nonfloat {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw const e_cmdwin as *const ::core::ffi::c_char,
                );
                return ::core::ptr::null_mut::<win_T>();
            }
        }
        let mut tp_0: *mut tabpage_T = if win_tp == curtab.get() {
            ::core::ptr::null_mut::<tabpage_T>()
        } else {
            win_tp
        };
        let mut dir: ::core::ffi::c_int = 0;
        winframe_remove(
            wp,
            &raw mut dir,
            tp_0,
            ::core::ptr::null_mut::<*mut frame_T>(),
        );
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*wp).w_frame as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        win_remove(wp, tp_0);
        if win_tp == curtab.get() {
            last_status(false_0 != 0);
            win_comp_pos();
        }
        win_append(lastwin_nofloating(tp_0), wp, tp_0);
    }
    (*wp).w_floating = true_0 != 0;
    (*wp).w_status_height = if !(*wp).w_onebuf_opt.wo_stl.is_null()
        && *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
        && (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt)
    {
        STATUS_HEIGHT as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    (*wp).w_winbar_height = 0 as ::core::ffi::c_int;
    (*wp).w_hsep_height = 0 as ::core::ffi::c_int;
    (*wp).w_vsep_width = 0 as ::core::ffi::c_int;
    win_config_float(wp, fconfig);
    redraw_later(wp, UPD_VALID);
    return wp;
}
pub unsafe extern "C" fn win_set_minimal_style(mut wp: *mut win_T) {
    (*wp).w_onebuf_opt.wo_nu = false_0;
    (*wp).w_onebuf_opt.wo_rnu = false_0;
    (*wp).w_onebuf_opt.wo_cul = false_0;
    (*wp).w_onebuf_opt.wo_cuc = false_0;
    (*wp).w_onebuf_opt.wo_spell = false_0;
    (*wp).w_onebuf_opt.wo_list = false_0;
    if (*wp).w_p_fcs_chars.eob != ' ' as schar_T {
        let mut old: *mut ::core::ffi::c_char = (*wp).w_onebuf_opt.wo_fcs;
        (*wp).w_onebuf_opt.wo_fcs = if *old as ::core::ffi::c_int == NUL {
            xstrdup(b"eob: \0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            concat_str(old, b",eob: \0".as_ptr() as *const ::core::ffi::c_char)
        };
        free_string_option(old);
    }
    let mut old_0: *mut ::core::ffi::c_char = (*wp).w_onebuf_opt.wo_winhl;
    (*wp).w_onebuf_opt.wo_winhl = if *old_0 as ::core::ffi::c_int == NUL {
        xstrdup(b"EndOfBuffer:\0".as_ptr() as *const ::core::ffi::c_char)
    } else {
        concat_str(
            old_0,
            b",EndOfBuffer:\0".as_ptr() as *const ::core::ffi::c_char,
        )
    };
    free_string_option(old_0);
    parse_winhl_opt(::core::ptr::null::<::core::ffi::c_char>(), wp);
    if *(*wp)
        .w_onebuf_opt
        .wo_scl
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 'a' as ::core::ffi::c_int
        || strlen((*wp).w_onebuf_opt.wo_scl) >= 8 as size_t
    {
        free_string_option((*wp).w_onebuf_opt.wo_scl);
        (*wp).w_onebuf_opt.wo_scl = xstrdup(b"auto\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if *(*wp)
        .w_onebuf_opt
        .wo_fdc
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != '0' as ::core::ffi::c_int
    {
        free_string_option((*wp).w_onebuf_opt.wo_fdc);
        (*wp).w_onebuf_opt.wo_fdc = xstrdup(b"0\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if !(*wp).w_onebuf_opt.wo_cc.is_null() && *(*wp).w_onebuf_opt.wo_cc as ::core::ffi::c_int != NUL
    {
        free_string_option((*wp).w_onebuf_opt.wo_cc);
        (*wp).w_onebuf_opt.wo_cc = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if !(*wp).w_onebuf_opt.wo_stc.is_null()
        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
    {
        free_string_option((*wp).w_onebuf_opt.wo_stc);
        (*wp).w_onebuf_opt.wo_stc = empty_string_option.ptr() as *mut ::core::ffi::c_char;
    }
    if (*wp).w_floating as ::core::ffi::c_int != 0
        && !(*wp).w_onebuf_opt.wo_stl.is_null()
        && *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
    {
        free_string_option((*wp).w_onebuf_opt.wo_stl);
        (*wp).w_onebuf_opt.wo_stl = empty_string_option.ptr() as *mut ::core::ffi::c_char;
        if (*wp).w_status_height > 0 as ::core::ffi::c_int {
            win_config_float(wp, (*wp).w_config);
        }
    }
}
pub unsafe extern "C" fn win_border_height(mut wp: *mut win_T) -> ::core::ffi::c_int {
    return (*wp).w_border_adj[0 as ::core::ffi::c_int as usize]
        + (*wp).w_border_adj[2 as ::core::ffi::c_int as usize];
}
pub unsafe extern "C" fn win_border_width(mut wp: *mut win_T) -> ::core::ffi::c_int {
    return (*wp).w_border_adj[1 as ::core::ffi::c_int as usize]
        + (*wp).w_border_adj[3 as ::core::ffi::c_int as usize];
}
pub unsafe extern "C" fn win_config_float(mut wp: *mut win_T, mut fconfig: WinConfig) {
    let mut show_stl: bool = *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
        && (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt);
    if (*wp).w_status_height != 0 && !show_stl {
        win_remove_status_line(wp, false_0 != 0);
    } else if (*wp).w_status_height == 0 as ::core::ffi::c_int
        && show_stl as ::core::ffi::c_int != 0
    {
        (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
    }
    (*wp).w_width = if fconfig.width > 1 as ::core::ffi::c_int {
        fconfig.width
    } else {
        1 as ::core::ffi::c_int
    };
    (*wp).w_height = if fconfig.height > 1 as ::core::ffi::c_int {
        fconfig.height
    } else {
        1 as ::core::ffi::c_int
    };
    if fconfig.relative as ::core::ffi::c_uint
        == kFloatRelativeCursor as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fconfig.relative = kFloatRelativeWindow;
        fconfig.row += (*curwin.get()).w_wrow as ::core::ffi::c_double;
        fconfig.col += (*curwin.get()).w_wcol as ::core::ffi::c_double;
        fconfig.window = (*curwin.get()).handle as Window;
    } else if fconfig.relative as ::core::ffi::c_uint
        == kFloatRelativeMouse as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut mouse_win: *mut win_T =
            mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
        if !mouse_win.is_null() {
            fconfig.relative = kFloatRelativeWindow;
            fconfig.row += row as ::core::ffi::c_double;
            fconfig.col += col as ::core::ffi::c_double;
            fconfig.window = (*mouse_win).handle as Window;
        }
    }
    let mut change_external: bool =
        fconfig.external as ::core::ffi::c_int != (*wp).w_config.external as ::core::ffi::c_int;
    let mut change_border: bool = fconfig.border as ::core::ffi::c_int
        != (*wp).w_config.border as ::core::ffi::c_int
        || memcmp(
            &raw mut fconfig.border_hl_ids as *mut ::core::ffi::c_int as *const ::core::ffi::c_void,
            &raw mut (*wp).w_config.border_hl_ids as *mut ::core::ffi::c_int
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_int; 8]>(),
        ) != 0 as ::core::ffi::c_int;
    merge_win_config(&raw mut (*wp).w_config, fconfig);
    let mut has_border: bool = (*wp).w_floating as ::core::ffi::c_int != 0
        && (*wp).w_config.border as ::core::ffi::c_int != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        let mut new_adj: ::core::ffi::c_int = (has_border as ::core::ffi::c_int != 0
            && (*wp).w_config.border_chars
                [(2 as ::core::ffi::c_int * i + 1 as ::core::ffi::c_int) as usize]
                [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int;
        if new_adj != (*wp).w_border_adj[i as usize] {
            change_border = true_0 != 0;
            (*wp).w_border_adj[i as usize] = new_adj;
        }
        i += 1;
    }
    if !ui_has(kUIMultigrid) {
        let mut above_ch: ::core::ffi::c_int =
            if (*wp).w_config.zindex < kZIndexMessages as ::core::ffi::c_int {
                p_ch.get() as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        (*wp).w_height = if (*wp).w_height < Rows.get() - win_border_height(wp) - above_ch {
            (*wp).w_height
        } else {
            Rows.get() - win_border_height(wp) - above_ch
        };
        (*wp).w_width = if (*wp).w_width < Columns.get() - win_border_width(wp) {
            (*wp).w_width
        } else {
            Columns.get() - win_border_width(wp)
        };
    }
    win_set_inner_size(wp, true_0 != 0);
    set_must_redraw(UPD_VALID);
    (*wp).w_redr_status = (*wp).w_status_height != 0;
    (*wp).w_pos_changed = true_0 != 0;
    if change_external as ::core::ffi::c_int != 0 || change_border as ::core::ffi::c_int != 0 {
        (*wp).w_hl_needs_update = true_0;
        redraw_later(wp, UPD_NOT_VALID);
    }
    if (*wp).w_config.relative as ::core::ffi::c_uint
        == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut row_0: ::core::ffi::c_int = (*wp).w_config.row as ::core::ffi::c_int;
        let mut col_0: ::core::ffi::c_int = (*wp).w_config.col as ::core::ffi::c_int;
        let mut dummy: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut parent: *mut win_T = find_window_by_handle((*wp).w_config.window, &raw mut dummy);
        if !parent.is_null() {
            row_0 += (*parent).w_winrow;
            col_0 += (*parent).w_wincol;
            grid_adjust(&raw mut (*parent).w_grid, &raw mut row_0, &raw mut col_0);
            if (*wp).w_config.bufpos.lnum >= 0 as linenr_T {
                let mut pos: pos_T = pos_T {
                    lnum: if ((*wp).w_config.bufpos.lnum + 1 as linenr_T)
                        < (*(*parent).w_buffer).b_ml.ml_line_count
                    {
                        (*wp).w_config.bufpos.lnum + 1 as linenr_T
                    } else {
                        (*(*parent).w_buffer).b_ml.ml_line_count
                    },
                    col: (*wp).w_config.bufpos.col,
                    coladd: 0 as colnr_T,
                };
                let mut trow: ::core::ffi::c_int = 0;
                let mut tcol: ::core::ffi::c_int = 0;
                let mut tcolc: ::core::ffi::c_int = 0;
                let mut tcole: ::core::ffi::c_int = 0;
                textpos2screenpos(
                    parent,
                    &raw mut pos,
                    &raw mut trow,
                    &raw mut tcol,
                    &raw mut tcolc,
                    &raw mut tcole,
                    true_0 != 0,
                );
                row_0 += trow - 1 as ::core::ffi::c_int;
                col_0 += tcol - 1 as ::core::ffi::c_int;
            }
        }
        api_clear_error(&raw mut dummy);
        (*wp).w_winrow = row_0;
        (*wp).w_wincol = col_0;
    } else {
        (*wp).w_winrow = fconfig.row as ::core::ffi::c_int;
        (*wp).w_wincol = fconfig.col as ::core::ffi::c_int;
    }
    if fconfig.border {
        (*wp).w_redr_border = true_0 != 0;
        redraw_later(wp, UPD_VALID);
    }
}
unsafe extern "C" fn float_zindex_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut za: ::core::ffi::c_int = (**(a as *mut *mut win_T)).w_config.zindex;
    let mut zb: ::core::ffi::c_int = (**(b as *mut *mut win_T)).w_config.zindex;
    return if za == zb {
        0 as ::core::ffi::c_int
    } else if za < zb {
        1 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn win_float_remove(mut bang: bool, mut count: ::core::ffi::c_int) {
    let mut float_win_arr: C2Rust_Unnamed_16 = KV_INITIAL_VALUE;
    let mut wp: *mut win_T = lastwin.get();
    while !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        if float_win_arr.size == float_win_arr.capacity {
            float_win_arr.capacity = if float_win_arr.capacity != 0 {
                float_win_arr.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            float_win_arr.items = xrealloc(
                float_win_arr.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<*mut win_T>().wrapping_mul(float_win_arr.capacity),
            ) as *mut *mut win_T;
        } else {
        };
        let c2rust_fresh0 = float_win_arr.size;
        float_win_arr.size = float_win_arr.size.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut *float_win_arr.items.offset(c2rust_fresh0 as isize);
        *c2rust_lvalue_ptr = wp;
        wp = (*wp).w_prev;
    }
    if float_win_arr.size > 0 as size_t {
        qsort(
            float_win_arr.items as *mut ::core::ffi::c_void,
            float_win_arr.size,
            ::core::mem::size_of::<*mut win_T>(),
            Some(
                float_zindex_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
    let mut i: size_t = 0 as size_t;
    while i < float_win_arr.size {
        let mut wp_0: *mut win_T = *float_win_arr.items.offset(i as isize);
        if win_valid(wp_0) as ::core::ffi::c_int != 0
            && win_close(wp_0, false_0 != 0, false_0 != 0) == FAIL
        {
            break;
        }
        if !bang {
            count -= 1;
            if count == 0 as ::core::ffi::c_int {
                break;
            }
        }
        i = i.wrapping_add(1);
    }
    xfree(float_win_arr.items as *mut ::core::ffi::c_void);
    float_win_arr.capacity = 0 as size_t;
    float_win_arr.size = float_win_arr.capacity;
    float_win_arr.items = ::core::ptr::null_mut::<*mut win_T>();
}
pub unsafe extern "C" fn win_check_anchored_floats(mut win: *mut win_T) {
    let mut wp: *mut win_T = lastwin.get();
    while !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        if (*wp).w_config.relative as ::core::ffi::c_uint
            == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*wp).w_config.window == (*win).handle
        {
            (*wp).w_pos_changed = true_0 != 0;
        }
        wp = (*wp).w_prev;
    }
}
pub unsafe extern "C" fn win_float_update_statusline() {
    let mut wp: *mut win_T = lastwin.get();
    while !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        let mut has_status: bool = (*wp).w_status_height > 0 as ::core::ffi::c_int;
        let mut should_show: bool = *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
            && (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt);
        if should_show as ::core::ffi::c_int != has_status as ::core::ffi::c_int {
            win_config_float(wp, (*wp).w_config);
        }
        wp = (*wp).w_prev;
    }
}
pub unsafe extern "C" fn win_float_anchor_laststatus() {
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_config.relative as ::core::ffi::c_uint
            == kFloatRelativeLaststatus as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*win).w_pos_changed = true_0 != 0;
        }
        win = (*win).w_next;
    }
}
pub unsafe extern "C" fn win_reconfig_floats() {
    let mut wp: *mut win_T = lastwin.get();
    while !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        win_config_float(wp, (*wp).w_config);
        wp = (*wp).w_prev;
    }
}
pub unsafe extern "C" fn win_float_find_preview() -> *mut win_T {
    let mut wp: *mut win_T = lastwin.get();
    while !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        if (*wp).w_float_is_info {
            return wp;
        }
        wp = (*wp).w_prev;
    }
    return ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn win_float_find_altwin(
    mut win: *const win_T,
    mut tp: *const tabpage_T,
) -> *mut win_T {
    let mut wp: *mut win_T = prevwin.get();
    if tp.is_null() {
        return if win_valid(wp) as ::core::ffi::c_int != 0
            && wp != win as *mut win_T
            && (*wp).w_config.focusable as ::core::ffi::c_int != 0
            && !(*wp).w_config.hide
        {
            wp
        } else {
            firstwin.get()
        };
    }
    debug_assert!(tp != curtab.get() as *const tabpage_T, "tp != curtab");
    wp = if tabpage_win_valid(tp, (*tp).tp_prevwin) as ::core::ffi::c_int != 0 {
        (*tp).tp_prevwin
    } else {
        (*tp).tp_firstwin
    };
    return if (*wp).w_config.focusable as ::core::ffi::c_int != 0 && !(*wp).w_config.hide {
        wp
    } else {
        (*tp).tp_firstwin
    };
}
#[inline]
unsafe extern "C" fn handle_error_and_cleanup(
    mut wp: *mut win_T,
    mut err: *mut Error,
) -> *mut win_T {
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg((*err).msg);
        api_clear_error(err);
    }
    if !wp.is_null() {
        win_remove(wp, ::core::ptr::null_mut::<tabpage_T>());
        win_free(wp, ::core::ptr::null_mut::<tabpage_T>());
    }
    unblock_autocmds();
    return ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn win_float_create_preview(
    mut enter: bool,
    mut new_buf: bool,
) -> *mut win_T {
    let mut config: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    config.col = (*curwin.get()).w_wcol as ::core::ffi::c_double;
    config.row = (*curwin.get()).w_wrow as ::core::ffi::c_double;
    config.relative = kFloatRelativeEditor;
    config.focusable = false_0 != 0;
    config.mouse = true_0 != 0;
    config.anchor = 0 as ::core::ffi::c_int as FloatAnchor;
    config.noautocmd = true_0 != 0;
    config.hide = true_0 != 0;
    config.style = kWinStyleMinimal;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    block_autocmds();
    let mut wp: *mut win_T = win_new_float(
        ::core::ptr::null_mut::<win_T>(),
        false_0 != 0,
        config,
        &raw mut err,
    );
    if wp.is_null() {
        return handle_error_and_cleanup(wp, &raw mut err);
    }
    if new_buf {
        let mut b: Buffer = nvim_create_buf(false_0 != 0, true_0 != 0, &raw mut err);
        if b == 0 {
            return handle_error_and_cleanup(wp, &raw mut err);
        }
        let mut buf: *mut buf_T = find_buffer_by_handle(b, &raw mut err);
        if buf.is_null() {
            return handle_error_and_cleanup(wp, &raw mut err);
        }
        (*buf).b_p_bl = false_0;
        set_option_direct_for(
            kOptBufhidden,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"wipe\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
            0 as scid_T,
            kOptScopeBuf,
            buf as *mut ::core::ffi::c_void,
        );
        win_set_buf(wp, buf, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return handle_error_and_cleanup(wp, &raw mut err);
        }
    }
    unblock_autocmds();
    (*wp).w_onebuf_opt.wo_diff = false_0;
    (*wp).w_float_is_info = true_0 != 0;
    (*wp).w_onebuf_opt.wo_wrap = true_0;
    (*wp).w_onebuf_opt.wo_so = 0 as OptInt;
    if enter {
        win_enter(wp, false_0 != 0);
    }
    return wp;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
