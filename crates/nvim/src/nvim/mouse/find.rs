//! Which window, row and column a screen position names --
//! `mouse_find_win()` and `mouse_comp_pos()`.
//!
//! [`mouse_comp_pos`] converts a window-relative row into a buffer line,
//! walking wrapped lines, folds and diff filler; [`mouse_find_win_inner`] and
//! [`mouse_find_win_outer`] walk the frame tree for the window containing a
//! screen position (the outer form counts the status line and separator as
//! belonging to the window above/left of them);
//! [`mouse_find_grid_win`] is the `ext_multigrid` entry point that maps a grid
//! handle plus coordinates onto both.  [`vcol2col`] is the column half.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::decor_conceal_line;
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::grid::get_win_by_grid_handle;
use crate::src::nvim::main::{curtab, firstwin, msg_grid, msg_grid_pos, pum_grid, topframe};
use crate::src::nvim::mbyte::{utf_ptr2StrCharInfo, utfc_next};
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::r#move::{win_col_off, win_col_off2};
use crate::src::nvim::plines::{
    init_charsize_arg, plines_win, plines_win_nofill, win_charsize, win_get_fill, win_may_fill,
};
use crate::src::nvim::types::{
    CharsizeArg, CharsizeKind, ScreenGrid, StrCharInfo, colnr_T, frame_T, handle_T, linenr_T, win_T,
};
use crate::src::nvim::ui_compositor::ui_comp_mouse_focus;

pub unsafe extern "C" fn mouse_comp_pos(
    mut win: *mut win_T,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
    mut lnump: *mut linenr_T,
) -> bool {
    unsafe {
        let mut col: ::core::ffi::c_int = *colp;
        let mut row: ::core::ffi::c_int = *rowp;
        let mut retval: bool = false_0 != 0;
        let mut count: ::core::ffi::c_int = 0;
        if (*win).w_onebuf_opt.wo_rl != 0 {
            col = (*win).w_view_width - 1 as ::core::ffi::c_int - col;
        }
        let mut lnum: linenr_T = (*win).w_topline;
        while row > 0 as ::core::ffi::c_int {
            if win_may_fill(win) {
                row -= if lnum == (*win).w_topline {
                    (*win).w_topfill
                } else {
                    win_get_fill(win, lnum)
                };
                count = plines_win_nofill(win, lnum, false_0 != 0);
            } else {
                count = plines_win(win, lnum, false_0 != 0);
            }
            if (*win).w_skipcol > 0 as ::core::ffi::c_int && lnum == (*win).w_topline {
                let mut width1: ::core::ffi::c_int = (*win).w_view_width - win_col_off(win);
                if width1 > 0 as ::core::ffi::c_int {
                    let mut skip_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if (*win).w_skipcol > width1 {
                        skip_lines = ((*win).w_skipcol as ::core::ffi::c_int - width1)
                            / (width1 + win_col_off2(win))
                            + 1 as ::core::ffi::c_int;
                    } else if (*win).w_skipcol > 0 as ::core::ffi::c_int {
                        skip_lines = 1 as ::core::ffi::c_int;
                    }
                    count -= skip_lines;
                }
            }
            if count > row {
                break;
            }
            hasFolding(
                win,
                lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut lnum,
            );
            if lnum == (*(*win).w_buffer).b_ml.ml_line_count {
                retval = true_0 != 0;
                break;
            } else {
                row -= count;
                lnum += 1;
            }
        }
        while lnum < (*(*win).w_buffer).b_ml.ml_line_count
            && decor_conceal_line(
                win,
                lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int
                != 0
        {
            lnum += 1;
            hasFolding(
                win,
                lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut lnum,
            );
        }
        if !retval {
            let mut off: ::core::ffi::c_int = win_col_off(win) - win_col_off2(win);
            col = if col > off { col } else { off };
            col += row * ((*win).w_view_width - off);
            if lnum == (*win).w_topline {
                col += (*win).w_skipcol as ::core::ffi::c_int;
            }
        }
        if (*win).w_onebuf_opt.wo_wrap == 0 {
            col += (*win).w_leftcol as ::core::ffi::c_int;
        }
        col -= win_col_off(win);
        col = if col > 0 as ::core::ffi::c_int {
            col
        } else {
            0 as ::core::ffi::c_int
        };
        *colp = col;
        *rowp = row;
        *lnump = lnum;
        return retval;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mouse_find_win_inner(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    unsafe {
        let mut wp_grid: *mut win_T = mouse_find_grid_win(gridp, rowp, colp);
        if !wp_grid.is_null() {
            return wp_grid;
        } else if *gridp > 1 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<win_T>();
        }
        let mut fp: *mut frame_T = topframe.get();
        *rowp -= (*firstwin.get()).w_winrow;
        while (*fp).fr_layout as ::core::ffi::c_int != FR_LEAF {
            if (*fp).fr_layout as ::core::ffi::c_int == FR_ROW {
                fp = (*fp).fr_child;
                while !(*fp).fr_next.is_null() {
                    if *colp < (*fp).fr_width {
                        break;
                    }
                    *colp -= (*fp).fr_width;
                    fp = (*fp).fr_next;
                }
            } else {
                fp = (*fp).fr_child;
                while !(*fp).fr_next.is_null() {
                    if *rowp < (*fp).fr_height {
                        break;
                    }
                    *rowp -= (*fp).fr_height;
                    fp = (*fp).fr_next;
                }
            }
        }
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if wp == (*fp).fr_win {
                *rowp -= (*wp).w_winbar_height;
                return wp;
            }
            wp = (*wp).w_next;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}

pub unsafe extern "C" fn mouse_find_win_outer(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = mouse_find_win_inner(gridp, rowp, colp);
        if !wp.is_null() {
            *rowp += (*wp).w_winrow_off;
            *colp += (*wp).w_wincol_off;
        }
        return wp;
    }
}

unsafe extern "C" fn mouse_find_grid_win(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    unsafe {
        if *gridp == (*msg_grid.ptr()).handle {
            *rowp += msg_grid_pos.get();
            *gridp = DEFAULT_GRID_HANDLE;
        } else if *gridp > 1 as ::core::ffi::c_int {
            let mut wp: *mut win_T = get_win_by_grid_handle(*gridp as handle_T);
            if !wp.is_null()
                && !(*wp).w_grid_alloc.chars.is_null()
                && !((*wp).w_floating as ::core::ffi::c_int != 0 && !(*wp).w_config.mouse)
            {
                *rowp = if *rowp - (*wp).w_grid.row_offset
                    < (*wp).w_view_height - 1 as ::core::ffi::c_int
                {
                    *rowp - (*wp).w_grid.row_offset
                } else {
                    (*wp).w_view_height - 1 as ::core::ffi::c_int
                };
                *colp = if *colp - (*wp).w_grid.col_offset
                    < (*wp).w_view_width - 1 as ::core::ffi::c_int
                {
                    *colp - (*wp).w_grid.col_offset
                } else {
                    (*wp).w_view_width - 1 as ::core::ffi::c_int
                };
                return wp;
            }
        } else if *gridp == 0 as ::core::ffi::c_int {
            let mut grid: *mut ScreenGrid = ui_comp_mouse_focus(*rowp, *colp);
            if grid == pum_grid.ptr() {
                *gridp = (*grid).handle as ::core::ffi::c_int;
                *rowp -= (*grid).comp_row;
                *colp -= (*grid).comp_col;
                return ::core::ptr::null_mut::<win_T>();
            } else {
                let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
                    firstwin.get()
                } else {
                    (*curtab.get()).tp_firstwin
                };
                while !wp_0.is_null() {
                    if &raw mut (*wp_0).w_grid_alloc != grid {
                        wp_0 = (*wp_0).w_next;
                    } else {
                        *gridp = (*grid).handle as ::core::ffi::c_int;
                        *rowp -= (*wp_0).w_winrow + (*wp_0).w_grid.row_offset;
                        *colp -= (*wp_0).w_wincol + (*wp_0).w_grid.col_offset;
                        return wp_0;
                    }
                }
            }
            *gridp = DEFAULT_GRID_HANDLE;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}

pub unsafe extern "C" fn vcol2col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut vcol: colnr_T,
    mut coladdp: *mut colnr_T,
) -> colnr_T {
    unsafe {
        let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
        let mut csarg: CharsizeArg = CharsizeArg::default();
        let mut cstype: CharsizeKind = init_charsize_arg(&mut csarg, wp, lnum, line);
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        let mut cur_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while cur_vcol < vcol && *ci.ptr as ::core::ffi::c_int != NUL {
            let mut next_vcol: ::core::ffi::c_int =
                cur_vcol + win_charsize(cstype, cur_vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            if next_vcol > vcol {
                break;
            }
            cur_vcol = next_vcol;
            ci = utfc_next(ci);
        }
        if !coladdp.is_null() {
            *coladdp = (vcol as ::core::ffi::c_int - cur_vcol) as colnr_T;
        }
        return ci.ptr.offset_from(line) as colnr_T;
    }
}
