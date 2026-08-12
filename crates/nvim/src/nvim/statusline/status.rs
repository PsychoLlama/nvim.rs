//! The plain status line, and the click-definition arenas.
//!
//! [`win_redr_status`] is the default status line -- the one drawn when
//! `'statusline'` is empty: the buffer name (shortened to fit by
//! [`get_trans_bufname`]), the `[+]`/`[RO]`/`[Help]` flags and the ruler.
//! [`stl_connected`] answers whether a window's status line runs all the way
//! to the screen edge, which decides its fill character.  The `stl_*_click_defs`
//! trio owns the per-window arena of `%@Func@` click records: allocate it to
//! the window's width, fill it from the parsed items, and free the strings
//! it holds.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::buffer::buf_spname;
use crate::src::nvim::charset::{trans_characters, vim_strnsize};
use crate::src::nvim::drawscreen::redrawing;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{grid_line_flush, grid_line_put_schar, grid_line_start};
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight_group::HLF_C;
use crate::src::nvim::main::{
    NameBuff, curwin, default_gridview, redraw_cmdline, wild_menu_showing,
};
use crate::src::nvim::memory::{xcalloc, xfree, xstrlcpy};
use crate::src::nvim::os::env::home_replace;
use crate::src::nvim::os::libc::memset;
use crate::src::nvim::types::ui::kUIWildmenu;
use crate::src::nvim::types::{
    StlClickDefinition, StlClickRecord, buf_T, frame_T, hlf_T, schar_T, size_t, win_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::window::global_stl_height;

pub unsafe extern "C" fn win_redr_status(mut wp: *mut win_T) {
    unsafe {
        let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
        static busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if busy.get() as ::core::ffi::c_int != 0
            || wild_menu_showing.get() != 0 as ::core::ffi::c_int && !ui_has(kUIWildmenu)
        {
            return;
        }
        busy.set(true_0 != 0);
        (*wp).w_redr_status = false_0 != 0;
        if (*wp).w_status_height == 0 as ::core::ffi::c_int
            && !(is_stl_global as ::core::ffi::c_int != 0 && wp == curwin.get())
        {
            redraw_cmdline.set(true_0 != 0);
        } else if !redrawing() {
            (*wp).w_redr_status = true_0 != 0;
        } else if *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
            || !(*wp).w_floating
            || is_stl_global as ::core::ffi::c_int != 0 && wp == curwin.get()
        {
            redraw_custom_statusline(wp);
        }
        let mut group: hlf_T = HLF_C;
        if (*wp).w_vsep_width != 0 as ::core::ffi::c_int
            && (*wp).w_status_height != 0 as ::core::ffi::c_int
            && redrawing() as ::core::ffi::c_int != 0
        {
            let mut fillchar: schar_T = 0;
            if stl_connected(wp) {
                fillchar = fillchar_status(&raw mut group, wp);
            } else {
                fillchar = (*wp).w_p_fcs_chars.vert;
            }
            let mut attr: ::core::ffi::c_int = win_hl_attr(wp, group as ::core::ffi::c_int);
            grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
            grid_line_put_schar((*wp).w_wincol + (*wp).w_width, fillchar, attr);
            grid_line_flush();
        }
        busy.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn get_trans_bufname(mut buf: *mut buf_T) {
    unsafe {
        if !buf_spname(buf).is_null() {
            xstrlcpy(
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                buf_spname(buf),
                MAXPATHL as size_t,
            );
        } else {
            home_replace(
                buf,
                (*buf).b_fname,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
        }
        trans_characters(NameBuff.ptr() as *mut ::core::ffi::c_char, MAXPATHL);
    }
}

pub unsafe extern "C" fn stl_connected(mut wp: *mut win_T) -> bool {
    unsafe {
        let mut fr: *mut frame_T = (*wp).w_frame;
        while !(*fr).fr_parent.is_null() {
            if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL {
                if !(*fr).fr_next.is_null() {
                    break;
                }
            } else if !(*fr).fr_next.is_null() {
                return true_0 != 0;
            }
            fr = (*fr).fr_parent;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn stl_clear_click_defs(
    click_defs: *mut StlClickDefinition,
    click_defs_size: size_t,
) {
    unsafe {
        if !click_defs.is_null() {
            let mut i: size_t = 0 as size_t;
            while i < click_defs_size {
                if i == 0 as size_t
                    || (*click_defs.add(i)).func
                        != (*click_defs.add(i.wrapping_sub(1 as size_t))).func
                {
                    xfree((*click_defs.add(i)).func as *mut ::core::ffi::c_void);
                }
                i = i.wrapping_add(1);
            }
            memset(
                click_defs as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                click_defs_size.wrapping_mul(::core::mem::size_of::<StlClickDefinition>()),
            );
        }
    }
}

pub unsafe extern "C" fn stl_alloc_click_defs(
    mut cdp: *mut StlClickDefinition,
    mut width: ::core::ffi::c_int,
    mut size: *mut size_t,
) -> *mut StlClickDefinition {
    unsafe {
        if *size < width as size_t {
            xfree(cdp as *mut ::core::ffi::c_void);
            *size = width as size_t;
            cdp = xcalloc(*size, ::core::mem::size_of::<StlClickDefinition>())
                as *mut StlClickDefinition;
        }
        return cdp;
    }
}

pub unsafe extern "C" fn stl_fill_click_defs(
    mut click_defs: *mut StlClickDefinition,
    mut click_recs: *mut StlClickRecord,
    mut buf: *const ::core::ffi::c_char,
    mut width: ::core::ffi::c_int,
    mut tabline: bool,
) {
    unsafe {
        if click_defs.is_null() {
            return;
        }
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cur_click_def: StlClickDefinition = StlClickDefinition {
            type_0: kStlClickDisabled,
            tabnr: 0,
            func: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*click_recs.offset(i as isize)).start.is_null() {
            len += vim_strnsize(
                buf,
                (*click_recs.offset(i as isize)).start.offset_from(buf) as ::core::ffi::c_int,
            );
            debug_assert!(len <= width, "len <= width");
            if col < len {
                while col < len {
                    let c2rust_fresh5 = col;
                    col = col + 1;
                    *click_defs.offset(c2rust_fresh5 as isize) = cur_click_def;
                }
            } else {
                xfree(cur_click_def.func as *mut ::core::ffi::c_void);
            }
            buf = (*click_recs.offset(i as isize)).start;
            cur_click_def = (*click_recs.offset(i as isize)).def;
            if !tabline
                && !(cur_click_def.type_0 as ::core::ffi::c_uint
                    == kStlClickDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
                    || cur_click_def.type_0 as ::core::ffi::c_uint
                        == kStlClickFuncRun as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                cur_click_def.type_0 = kStlClickDisabled;
            }
            i += 1;
        }
        if col < width {
            while col < width {
                let c2rust_fresh6 = col;
                col = col + 1;
                *click_defs.offset(c2rust_fresh6 as isize) = cur_click_def;
            }
        } else {
            xfree(cur_click_def.func as *mut ::core::ffi::c_void);
        };
    }
}
