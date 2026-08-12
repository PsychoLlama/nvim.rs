//! The mouse: which window and character a screen position names, and what a
//! button does there.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`click`] | `%@Func@` click definitions and the popup menu |
//! | [`domouse`] | `do_mouse()`, the Normal/Visual-mode mouse command |
//! | [`scroll`] | the wheel, and the mouse in Insert mode |
//! | [`jump`] | `jump_to_mouse()`, screen position to buffer position |
//! | [`find`] | which window, line and column a screen position names |
//!
//! What stays here is the flag alphabet the five share (`MOUSE_*`, `IN_*`,
//! `MOD_MASK_*`), the word-boundary helpers a double click uses, the tab-page
//! click actions, `setmouse()`, `getmousepos()` and the "longest line" scan
//! `'mousescroll'` needs.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::charset::vim_iswordc;
use crate::src::nvim::cursor::set_leftcol;
use crate::src::nvim::eval::typval::{tv_dict_add_nr, tv_dict_alloc_ret};
use crate::src::nvim::ex_docmd::{tabpage_close, tabpage_close_other};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::grid_adjust;
use crate::src::nvim::main::{
    curbuf, curtab, curwin, first_tabpage, mouse_col, mouse_grid, mouse_row, p_sel,
    tab_page_click_defs,
};
use crate::src::nvim::mbyte::{mb_get_class, utf_head_off, utf8len_tab, utfc_ptr2len};
use crate::src::nvim::memline::ml_get;
use crate::src::nvim::os::libc::abs;
use crate::src::nvim::plines::win_chartabsize;
use crate::src::nvim::search::BACKWARD;
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    EvalFuncData, MotionType, ScreenGrid, cmdarg_T, colnr_T, dict_T, linenr_T, pos_T, size_t,
    tabpage_T, typval_T, uint8_t, varnumber_T, win_T,
};
use crate::src::nvim::ui::{ui_check_mouse, ui_cursor_shape};
use crate::src::nvim::window::{find_tabpage, tabpage_index, tabpage_move};

// The carve of the transpiled module; see each child's docs.
mod click;
mod domouse;
mod find;
mod jump;
mod scroll;

pub(crate) use self::click::*;
pub use self::domouse::*;
pub use self::find::*;
pub use self::jump::*;
pub use self::scroll::*;

pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MOUSE_STATUSCOL: C2Rust_Unnamed_16 = 4096;
pub const MOUSE_WINBAR: C2Rust_Unnamed_16 = 2048;
pub const MOUSE_FOLD_OPEN: C2Rust_Unnamed_16 = 1024;
pub const MOUSE_FOLD_CLOSE: C2Rust_Unnamed_16 = 512;
pub const CURSOR_MOVED: C2Rust_Unnamed_16 = 256;
pub const IN_OTHER_WIN: C2Rust_Unnamed_16 = 8;
pub const IN_SEP_LINE: C2Rust_Unnamed_16 = 4;
pub const IN_STATUS_LINE: C2Rust_Unnamed_16 = 2;
pub const IN_BUFFER: C2Rust_Unnamed_16 = 1;
pub const IN_UNKNOWN: C2Rust_Unnamed_16 = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MOUSE_RELEASED: C2Rust_Unnamed_17 = 32;
pub const MOUSE_MAY_STOP_VIS: C2Rust_Unnamed_17 = 16;
pub const MOUSE_SETPOS: C2Rust_Unnamed_17 = 8;
pub const MOUSE_DID_MOVE: C2Rust_Unnamed_17 = 4;
pub const MOUSE_MAY_VIS: C2Rust_Unnamed_17 = 2;
pub const MOUSE_FOCUS: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const MOUSE_X2: C2Rust_Unnamed_18 = 1024;
pub const MOUSE_X1: C2Rust_Unnamed_18 = 768;
pub const MOUSE_RIGHT: C2Rust_Unnamed_18 = 2;
pub const MOUSE_MIDDLE: C2Rust_Unnamed_18 = 1;
pub const MOUSE_LEFT: C2Rust_Unnamed_18 = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const MSCR_RIGHT: C2Rust_Unnamed_19 = -2;
pub const MSCR_UP: C2Rust_Unnamed_19 = 1;
pub const MSCR_DOWN: C2Rust_Unnamed_19 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_BOTLINE_AP: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MOUSE_VISUAL: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MOD_MASK_META: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MOD_MASK_2CLICK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MOD_MASK_3CLICK: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MOD_MASK_4CLICK: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const MOD_MASK_MULTI_CLICK: ::core::ffi::c_int =
    MOD_MASK_2CLICK | MOD_MASK_3CLICK | MOD_MASK_4CLICK;
static orig_topline: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static orig_topfill: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
unsafe extern "C" fn get_mouse_class(mut p: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if utf8len_tab[*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as usize]
            as ::core::ffi::c_int
            > 1 as ::core::ffi::c_int
        {
            return mb_get_class(p);
        }
        let c: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
        if c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if vim_iswordc(c) {
            return 2 as ::core::ffi::c_int;
        }
        if c != NUL && !vim_strchr(c"-+*/%<>&|^!=".as_ptr(), c).is_null() {
            return 1 as ::core::ffi::c_int;
        }
        return c;
    }
}
unsafe extern "C" fn find_start_of_word(mut pos: *mut pos_T) {
    unsafe {
        let mut line: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
        let mut cclass: ::core::ffi::c_int = get_mouse_class(line.offset((*pos).col as isize));
        while (*pos).col > 0 as ::core::ffi::c_int {
            let mut col: ::core::ffi::c_int =
                (*pos).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            col -= utf_head_off(line, line.offset(col as isize));
            if get_mouse_class(line.offset(col as isize)) != cclass {
                break;
            }
            (*pos).col = col as colnr_T;
        }
    }
}
unsafe extern "C" fn find_end_of_word(mut pos: *mut pos_T) {
    unsafe {
        let mut line: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            && (*pos).col > 0 as ::core::ffi::c_int
        {
            (*pos).col -= 1;
            (*pos).col -= utf_head_off(line, line.offset((*pos).col as isize));
        }
        let mut cclass: ::core::ffi::c_int = get_mouse_class(line.offset((*pos).col as isize));
        while *line.offset((*pos).col as isize) as ::core::ffi::c_int != NUL {
            let mut col: ::core::ffi::c_int =
                (*pos).col as ::core::ffi::c_int + utfc_ptr2len(line.offset((*pos).col as isize));
            if get_mouse_class(line.offset(col as isize)) != cclass {
                if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                    (*pos).col = col as colnr_T;
                }
                break;
            } else {
                (*pos).col = col as colnr_T;
            }
        }
    }
}
unsafe extern "C" fn move_tab_to_mouse() {
    unsafe {
        let mut tabnr: ::core::ffi::c_int =
            (*(*tab_page_click_defs.ptr()).offset(mouse_col.get() as isize)).tabnr;
        if tabnr <= 0 as ::core::ffi::c_int {
            tabpage_move(9999 as ::core::ffi::c_int);
        } else if tabnr < tabpage_index(curtab.get()) {
            tabpage_move(tabnr - 1 as ::core::ffi::c_int);
        } else {
            tabpage_move(tabnr);
        };
    }
}
unsafe extern "C" fn mouse_tab_close(mut c1: ::core::ffi::c_int) {
    unsafe {
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if c1 == 999 as ::core::ffi::c_int {
            tp = curtab.get();
        } else {
            tp = find_tabpage(c1);
        }
        if tp == curtab.get() {
            if !(*first_tabpage.get()).tp_next.is_null() {
                tabpage_close(false_0);
            }
        } else if !tp.is_null() {
            tabpage_close_other(tp, false_0);
        }
    }
}
static got_click: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn do_mousescroll_horiz(mut leftcol: colnr_T) -> bool {
    unsafe {
        if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 {
            return false_0 != 0;
        }
        if (*curwin.get()).w_leftcol == leftcol {
            return false_0 != 0;
        }
        if !virtual_active(curwin.get()) && leftcol > scroll_line_len((*curwin.get()).w_cursor.lnum)
        {
            (*curwin.get()).w_cursor.lnum = find_longest_lnum();
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        return set_leftcol(leftcol);
    }
}
pub unsafe fn nv_mousescroll(mut cap: *mut cmdarg_T) {
    unsafe {
        let old_curwin: *mut win_T = curwin.get();
        if mouse_row.get() >= 0 as ::core::ffi::c_int && mouse_col.get() >= 0 as ::core::ffi::c_int
        {
            let mut grid: ::core::ffi::c_int = mouse_grid.get();
            let mut row: ::core::ffi::c_int = mouse_row.get();
            let mut col: ::core::ffi::c_int = mouse_col.get();
            curwin.set(mouse_find_win_inner(
                &raw mut grid,
                &raw mut row,
                &raw mut col,
            ));
            if (*curwin.ptr()).is_null() {
                curwin.set(old_curwin);
                return;
            }
            curbuf.set((*curwin.get()).w_buffer);
        }
        do_mousescroll(cap);
        (*curwin.get()).w_redr_status = true_0 != 0;
        curwin.set(old_curwin);
        curbuf.set((*curwin.get()).w_buffer);
    }
}
pub unsafe fn nv_mouse(mut cap: *mut cmdarg_T) {
    unsafe {
        do_mouse(
            (*cap).oap,
            (*cap).cmdchar,
            BACKWARD as ::core::ffi::c_int,
            (*cap).count1,
            false,
        );
    }
}
pub unsafe extern "C" fn setmouse() {
    unsafe {
        ui_cursor_shape();
        ui_check_mouse();
    }
}
unsafe extern "C" fn set_mouse_topline(mut wp: *mut win_T) {
    unsafe {
        orig_topline.set((*wp).w_topline);
        orig_topfill.set((*wp).w_topfill);
    }
}
unsafe extern "C" fn scroll_line_len(mut lnum: linenr_T) -> colnr_T {
    unsafe {
        let mut col: colnr_T = 0 as colnr_T;
        let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
        if *line as ::core::ffi::c_int != NUL {
            loop {
                let mut numchar: ::core::ffi::c_int = win_chartabsize(curwin.get(), line, col);
                line = line.offset(utfc_ptr2len(line) as isize);
                if *line as ::core::ffi::c_int == NUL {
                    break;
                }
                col += numchar;
            }
        }
        return col;
    }
}
unsafe extern "C" fn find_longest_lnum() -> linenr_T {
    unsafe {
        let mut ret: linenr_T = 0 as linenr_T;
        if (*curwin.get()).w_topline <= (*curwin.get()).w_cursor.lnum
            && (*curwin.get()).w_botline > (*curwin.get()).w_cursor.lnum
            && (*curwin.get()).w_botline <= (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T
        {
            let mut max: colnr_T = 0 as colnr_T;
            let mut lnum: linenr_T = (*curwin.get()).w_topline;
            while lnum < (*curwin.get()).w_botline {
                let mut len: colnr_T = scroll_line_len(lnum);
                if len > max {
                    max = len;
                    ret = lnum;
                } else if len == max
                    && abs(lnum as ::core::ffi::c_int
                        - (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int)
                        < abs(ret as ::core::ffi::c_int
                            - (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int)
                {
                    ret = lnum;
                }
                lnum += 1;
            }
        } else {
            ret = (*curwin.get()).w_cursor.lnum;
        }
        return ret;
    }
}
unsafe extern "C" fn mouse_check_grid(
    mut vcolp: *mut colnr_T,
    mut flagsp: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut click_grid: ::core::ffi::c_int = mouse_grid.get();
        let mut click_row: ::core::ffi::c_int = mouse_row.get();
        let mut click_col: ::core::ffi::c_int = mouse_col.get();
        if mouse_find_win_inner(&raw mut click_grid, &raw mut click_row, &raw mut click_col)
            != curwin.get()
            || (*curwin.get()).w_redr_type != 0 as ::core::ffi::c_int
        {
            return;
        }
        let mut start_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut start_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut gp: *mut ScreenGrid = grid_adjust(
            &raw mut (*curwin.get()).w_grid,
            &raw mut start_row,
            &raw mut start_col,
        );
        if (*gp).handle != click_grid || (*gp).chars.is_null() {
            return;
        }
        click_row += start_row;
        click_col += start_col;
        if click_row < 0 as ::core::ffi::c_int
            || click_row >= (*gp).rows
            || click_col < 0 as ::core::ffi::c_int
            || click_col >= (*gp).cols
        {
            return;
        }
        let off: size_t =
            (*(*gp).line_offset.offset(click_row as isize)).wrapping_add(click_col as size_t);
        let mut col_from_screen: colnr_T = *(*gp).vcols.add(off);
        if col_from_screen >= 0 as ::core::ffi::c_int {
            *vcolp = col_from_screen;
        }
        if col_from_screen == -2 as ::core::ffi::c_int {
            *flagsp |= MOUSE_FOLD_OPEN as ::core::ffi::c_int;
        } else if col_from_screen == -3 as ::core::ffi::c_int {
            *flagsp |= MOUSE_FOLD_CLOSE as ::core::ffi::c_int;
        }
    }
}
pub unsafe extern "C" fn f_getmousepos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut winid: varnumber_T = 0 as varnumber_T;
        let mut winrow: varnumber_T = 0 as varnumber_T;
        let mut wincol: varnumber_T = 0 as varnumber_T;
        let mut lnum: linenr_T = 0 as linenr_T;
        let mut column: varnumber_T = 0 as varnumber_T;
        let mut coladd: colnr_T = 0 as colnr_T;
        tv_dict_alloc_ret(rettv);
        let mut d: *mut dict_T = (*rettv).vval.v_dict;
        tv_dict_add_nr(
            d,
            c"screenrow".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            mouse_row.get() as varnumber_T + 1 as varnumber_T,
        );
        tv_dict_add_nr(
            d,
            c"screencol".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            mouse_col.get() as varnumber_T + 1 as varnumber_T,
        );
        let mut wp: *mut win_T = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
        if !wp.is_null() {
            let mut height: ::core::ffi::c_int =
                (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height;
            if row < height + (*wp).w_border_adj[2 as ::core::ffi::c_int as usize] {
                winid = (*wp).handle as varnumber_T;
                winrow = (row + 1 as ::core::ffi::c_int + (*wp).w_winrow_off) as varnumber_T;
                wincol = (col + 1 as ::core::ffi::c_int + (*wp).w_wincol_off) as varnumber_T;
                if row >= 0 as ::core::ffi::c_int
                    && row < (*wp).w_height
                    && col >= 0 as ::core::ffi::c_int
                    && col < (*wp).w_width
                {
                    mouse_comp_pos(wp, &raw mut row, &raw mut col, &raw mut lnum);
                    col = vcol2col(wp, lnum, col as colnr_T, &raw mut coladd) as ::core::ffi::c_int;
                    column = (col + 1 as ::core::ffi::c_int) as varnumber_T;
                }
            }
        }
        tv_dict_add_nr(
            d,
            c"winid".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            winid,
        );
        tv_dict_add_nr(
            d,
            c"winrow".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            winrow,
        );
        tv_dict_add_nr(
            d,
            c"wincol".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            wincol,
        );
        tv_dict_add_nr(
            d,
            c"line".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            lnum as varnumber_T,
        );
        tv_dict_add_nr(
            d,
            c"column".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            column,
        );
        tv_dict_add_nr(
            d,
            c"coladd".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            coladd as varnumber_T,
        );
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
