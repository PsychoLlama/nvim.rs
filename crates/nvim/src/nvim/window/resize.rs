//! Applying a new size to a window, and the rows that are not text.
//!
//! [`win_set_inner_size`] is where a height or width finally lands on a
//! `win_T`: it resizes the grid, re-wraps the text, and keeps the view stable
//! by remembering the cursor's [`set_fraction`] of the window and restoring it
//! with [`scroll_to_fraction`] ([`win_fix_scroll`] and [`win_fix_cursor`] are
//! the `'splitkeep'` half of the same problem).  The rest is the non-text
//! bookkeeping: [`command_height`] for `'cmdheight'`, [`last_status`] and
//! [`last_status_rec`] for `'laststatus'`, [`set_winbar`] for `'winbar'`,
//! [`tabline_height`] for `'showtabline'`, and [`min_rows`], which says how
//! few rows the layout can survive in.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::buffer::bt_help;
use crate::src::nvim::decoration::decor_conceal_line;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, comp_col, redraw_later};
use crate::src::nvim::edit::{cursor_down_inner, cursor_up_inner};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::grid::grid_clear;
use crate::src::nvim::main::{
    Columns, Rows, cmdline_row, curbuf, curtab, curwin, default_gridview, e_noroom, exiting,
    first_tabpage, firstwin, full_screen, msg_grid_adj, msg_row, msg_scrolled, p_ch, p_ls, p_spk,
    p_stal, p_wbr, redraw_cmdline, skip_update_topline, skip_win_fix_cursor, topframe,
};
use crate::src::nvim::mark::setmark;
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::{emsg, msg_grid_validate};
use crate::src::nvim::r#move::{
    changed_line_abv_curs_win, curs_columns, invalidate_botline_win, set_topline,
    validate_botline_win, validate_cursor, win_col_off, win_col_off2,
};
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::os::libc::{abs, gettext};
use crate::src::nvim::plines::{plines_win, plines_win_col, plines_win_nofill};
use crate::src::nvim::state::{MODE_CMDLINE, MODE_NORMAL, MODE_TERMINAL, get_real_state};
use crate::src::nvim::statusline::stl_clear_click_defs;
use crate::src::nvim::terminal::terminal_check_size;
use crate::src::nvim::types::ui::{kUIMessages, kUIMultigrid, kUITabline};
use crate::src::nvim::types::{
    GridView, Integer, OptInt, StlClickDefinition, Window, colnr_T, frame_T, int64_t, linenr_T,
    pos_T, scid_T, size_t, tabpage_T, win_T,
};
use crate::src::nvim::ui::{ui_call_win_viewport_margins, ui_has};
use crate::src::nvim::winfloat::{
    win_border_height, win_border_width, win_float_anchor_laststatus,
};

pub unsafe extern "C" fn set_fraction(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_view_height > 1 as ::core::ffi::c_int {
            (*wp).w_fraction = ((*wp).w_wrow * FRACTION_MULT
                + FRACTION_MULT / 2 as ::core::ffi::c_int)
                / (*wp).w_view_height;
        }
    }
}

pub unsafe extern "C" fn win_fix_scroll(mut resize: bool) {
    unsafe {
        if *p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
            return;
        }
        skip_update_topline.set(true_0 != 0);
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if !(*wp).w_floating && (*wp).w_height != (*wp).w_prev_height {
                (*wp).w_do_win_fix_cursor = true_0 != 0;
                if *p_spk.get() as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                    && (*wp).w_winrow != (*wp).w_prev_winrow
                    && (*wp).w_botline - 1 as linenr_T <= (*(*wp).w_buffer).b_ml.ml_line_count
                {
                    let mut diff: ::core::ffi::c_int = (*wp).w_winrow - (*wp).w_prev_winrow
                        + ((*wp).w_height - (*wp).w_prev_height);
                    let mut cursor: pos_T = (*wp).w_cursor;
                    (*wp).w_cursor.lnum = (*wp).w_botline - 1 as linenr_T;
                    if diff > 0 as ::core::ffi::c_int {
                        cursor_down_inner(wp, diff, false_0 != 0);
                    } else {
                        cursor_up_inner(wp, -(diff as linenr_T), false_0 != 0);
                    }
                    (*wp).w_fraction = FRACTION_MULT;
                    scroll_to_fraction(wp, (*wp).w_prev_height);
                    (*wp).w_cursor = cursor;
                    (*wp).w_valid &= !VALID_WCOL;
                } else if wp == curwin.get() {
                    (*wp).w_valid &= !VALID_CROW;
                }
                invalidate_botline_win(wp);
                validate_botline_win(wp);
            }
            (*wp).w_prev_height = (*wp).w_height;
            (*wp).w_prev_winrow = (*wp).w_winrow;
            wp = (*wp).w_next;
        }
        skip_update_topline.set(false_0 != 0);
        if get_real_state() & (MODE_NORMAL | MODE_CMDLINE | MODE_TERMINAL) == 0 {
            win_fix_cursor(false_0 != 0);
        } else if resize {
            win_fix_cursor(true_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn win_fix_cursor(mut normal: bool) {
    unsafe {
        let mut wp: *mut win_T = curwin.get();
        if skip_win_fix_cursor.get() as ::core::ffi::c_int != 0
            || !(*wp).w_do_win_fix_cursor
            || (*(*wp).w_buffer).b_ml.ml_line_count < (*wp).w_view_height as linenr_T
        {
            return;
        }
        (*wp).w_do_win_fix_cursor = false_0 != 0;
        let mut so: ::core::ffi::c_int = (if (((*wp).w_view_height / 2 as ::core::ffi::c_int)
            as int64_t)
            < get_scrolloff_value(wp)
        {
            ((*wp).w_view_height / 2 as ::core::ffi::c_int) as int64_t
        } else {
            get_scrolloff_value(wp)
        }) as ::core::ffi::c_int;
        let mut lnum: linenr_T = (*wp).w_cursor.lnum;
        (*wp).w_cursor.lnum = (*wp).w_topline;
        cursor_down_inner(wp, so, false_0 != 0);
        let mut top: linenr_T = (*wp).w_cursor.lnum;
        (*wp).w_cursor.lnum = (*wp).w_botline - 1 as linenr_T;
        cursor_up_inner(wp, so as linenr_T, false_0 != 0);
        let mut bot: linenr_T = (*wp).w_cursor.lnum;
        (*wp).w_cursor.lnum = lnum;
        let mut nlnum: linenr_T = 0 as linenr_T;
        if lnum > bot && (*wp).w_botline - (*(*wp).w_buffer).b_ml.ml_line_count != 1 as linenr_T {
            nlnum = bot;
        } else if lnum < top && (*wp).w_topline != 1 as linenr_T {
            nlnum = if so == (*wp).w_view_height / 2 as ::core::ffi::c_int {
                bot
            } else {
                top
            };
        }
        if nlnum != 0 as linenr_T {
            if normal {
                setmark('\'' as ::core::ffi::c_int);
                (*wp).w_cursor.lnum = nlnum;
            } else {
                (*wp).w_fraction = if nlnum == bot {
                    FRACTION_MULT
                } else {
                    0 as ::core::ffi::c_int
                };
                scroll_to_fraction(wp, (*wp).w_prev_height);
                validate_botline_win(curwin.get());
            }
        }
    }
}

pub unsafe extern "C" fn win_new_height(mut wp: *mut win_T, mut height: ::core::ffi::c_int) {
    unsafe {
        height = if height > 0 as ::core::ffi::c_int {
            height
        } else {
            0 as ::core::ffi::c_int
        };
        if (*wp).w_height == height {
            return;
        }
        (*wp).w_height = height;
        (*wp).w_pos_changed = true_0 != 0;
        win_set_inner_size(wp, true_0 != 0);
    }
}

pub unsafe extern "C" fn scroll_to_fraction(
    mut wp: *mut win_T,
    mut prev_height: ::core::ffi::c_int,
) {
    unsafe {
        let mut height: ::core::ffi::c_int = (*wp).w_view_height;
        if height > 0 as ::core::ffi::c_int
            && ((*wp).w_onebuf_opt.wo_scb == 0 || wp == curwin.get())
            && ((height as linenr_T) < (*(*wp).w_buffer).b_ml.ml_line_count
                || (*wp).w_topline > 1 as linenr_T)
        {
            let mut lnum: linenr_T = (*wp).w_cursor.lnum;
            lnum = if lnum > 1 as linenr_T {
                lnum
            } else {
                1 as linenr_T
            };
            (*wp).w_wrow = ((*wp).w_fraction * height - 1 as ::core::ffi::c_int) / FRACTION_MULT;
            let mut line_size: ::core::ffi::c_int =
                plines_win_col(wp, lnum, (*wp).w_cursor.col as ::core::ffi::c_long)
                    - 1 as ::core::ffi::c_int;
            let mut sline: ::core::ffi::c_int = (*wp).w_wrow - line_size;
            if sline >= 0 as ::core::ffi::c_int {
                let rows: ::core::ffi::c_int = plines_win(wp, lnum, false_0 != 0);
                if sline > (*wp).w_view_height - rows {
                    sline = (*wp).w_view_height - rows;
                    (*wp).w_wrow -= rows - line_size;
                }
            }
            if sline < 0 as ::core::ffi::c_int {
                (*wp).w_wrow = line_size;
                if (*wp).w_wrow >= (*wp).w_view_height
                    && (*wp).w_view_width - win_col_off(wp) > 0 as ::core::ffi::c_int
                {
                    (*wp).w_skipcol += (*wp).w_view_width - win_col_off(wp);
                    (*wp).w_wrow -= 1;
                    while (*wp).w_wrow >= (*wp).w_view_height {
                        (*wp).w_skipcol += (*wp).w_view_width - win_col_off(wp) + win_col_off2(wp);
                        (*wp).w_wrow -= 1;
                    }
                }
            } else if sline > 0 as ::core::ffi::c_int {
                while sline > 0 as ::core::ffi::c_int && lnum > 1 as linenr_T {
                    hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
                    if lnum == 1 as linenr_T {
                        line_size = !decor_conceal_line(
                            wp,
                            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            false_0 != 0,
                        ) as ::core::ffi::c_int;
                        sline -= 1;
                        break;
                    } else {
                        lnum -= 1;
                        if lnum == (*wp).w_topline {
                            line_size = plines_win_nofill(wp, lnum, true_0 != 0) + (*wp).w_topfill;
                        } else {
                            line_size = plines_win(wp, lnum, true_0 != 0);
                        }
                        sline -= line_size;
                    }
                }
                if sline < 0 as ::core::ffi::c_int {
                    hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                    lnum += 1;
                    (*wp).w_wrow -= line_size + sline;
                } else if sline > 0 as ::core::ffi::c_int {
                    lnum = 1 as ::core::ffi::c_int as linenr_T;
                    (*wp).w_wrow -= sline;
                }
            }
            set_topline(wp, lnum);
        }
        if wp == curwin.get() {
            curs_columns(wp, false_0);
        }
        if prev_height > 0 as ::core::ffi::c_int {
            (*wp).w_prev_fraction_row = (*wp).w_wrow;
        }
        redraw_later(wp, UPD_SOME_VALID);
        invalidate_botline_win(wp);
    }
}

pub unsafe extern "C" fn win_set_inner_size(mut wp: *mut win_T, mut valid_cursor: bool) {
    unsafe {
        let mut width: ::core::ffi::c_int = (*wp).w_width_request;
        if width == 0 as ::core::ffi::c_int {
            width = (*wp).w_width;
        }
        let mut prev_height: ::core::ffi::c_int = (*wp).w_view_height;
        let mut height: ::core::ffi::c_int = (*wp).w_height_request;
        if height == 0 as ::core::ffi::c_int {
            height = if 0 as ::core::ffi::c_int > (*wp).w_height - (*wp).w_winbar_height {
                0 as ::core::ffi::c_int
            } else {
                (*wp).w_height - (*wp).w_winbar_height
            };
        }
        if height != prev_height {
            if height > 0 as ::core::ffi::c_int && valid_cursor as ::core::ffi::c_int != 0 {
                if wp == curwin.get()
                    && (*p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int
                        || (*wp).w_floating as ::core::ffi::c_int != 0)
                {
                    validate_cursor(curwin.get());
                }
                if (*wp).w_view_height != prev_height {
                    return;
                }
                if (*wp).w_wrow != (*wp).w_prev_fraction_row {
                    set_fraction(wp);
                }
            }
            (*wp).w_view_height = height;
            win_comp_scroll(wp);
            if valid_cursor as ::core::ffi::c_int != 0
                && !exiting.get()
                && (*p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int
                    || (*wp).w_floating as ::core::ffi::c_int != 0)
            {
                (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                scroll_to_fraction(wp, prev_height);
            }
            redraw_later(wp, UPD_SOME_VALID);
        }
        if width != (*wp).w_view_width {
            (*wp).w_view_width = width;
            (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
            if valid_cursor {
                changed_line_abv_curs_win(wp);
                invalidate_botline_win(wp);
                if wp == curwin.get()
                    && (*p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int
                        || (*wp).w_floating as ::core::ffi::c_int != 0)
                {
                    curs_columns(wp, true_0);
                }
            }
            redraw_later(wp, UPD_NOT_VALID);
        }
        if !(*(*wp).w_buffer).terminal.is_null() {
            terminal_check_size((*(*wp).w_buffer).terminal);
        }
        let mut float_stl_height: ::core::ffi::c_int =
            if (*wp).w_floating as ::core::ffi::c_int != 0 && (*wp).w_status_height != 0 {
                STATUS_HEIGHT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        (*wp).w_height_outer =
            (*wp).w_view_height + win_border_height(wp) + (*wp).w_winbar_height + float_stl_height;
        (*wp).w_width_outer = (*wp).w_view_width + win_border_width(wp);
        (*wp).w_winrow_off =
            (*wp).w_border_adj[0 as ::core::ffi::c_int as usize] + (*wp).w_winbar_height;
        (*wp).w_wincol_off = (*wp).w_border_adj[3 as ::core::ffi::c_int as usize];
        if ui_has(kUIMultigrid) {
            ui_call_win_viewport_margins(
                (*wp).w_grid_alloc.handle as Integer,
                (*wp).handle as Window,
                (*wp).w_winrow_off as Integer,
                (*wp).w_border_adj[2 as ::core::ffi::c_int as usize] as Integer,
                (*wp).w_wincol_off as Integer,
                (*wp).w_border_adj[1 as ::core::ffi::c_int as usize] as Integer,
            );
        }
        (*wp).w_redr_status = true_0 != 0;
    }
}

pub unsafe extern "C" fn win_new_width(mut wp: *mut win_T, mut width: ::core::ffi::c_int) {
    unsafe {
        (*wp).w_width = if width < 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            width
        };
        (*wp).w_pos_changed = true_0 != 0;
        win_set_inner_size(wp, true_0 != 0);
    }
}

pub unsafe extern "C" fn win_default_scroll(mut wp: *mut win_T) -> OptInt {
    unsafe {
        return (if (*wp).w_view_height / 2 as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
            (*wp).w_view_height / 2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as OptInt;
    }
}

pub unsafe extern "C" fn win_comp_scroll(mut wp: *mut win_T) {
    unsafe {
        let old_w_p_scr: OptInt = (*wp).w_onebuf_opt.wo_scr;
        (*wp).w_onebuf_opt.wo_scr = win_default_scroll(wp);
        if (*wp).w_onebuf_opt.wo_scr != old_w_p_scr {
            (*wp).w_onebuf_opt.wo_script_ctx[kWinOptScroll as ::core::ffi::c_int as usize].sc_sid =
                SID_WINLAYOUT as scid_T;
            (*wp).w_onebuf_opt.wo_script_ctx[kWinOptScroll as ::core::ffi::c_int as usize]
                .sc_lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
    }
}

pub unsafe extern "C" fn command_height() {
    unsafe {
        let mut old_p_ch: ::core::ffi::c_int = (*curtab.get()).tp_ch_used as ::core::ffi::c_int;
        let mut frp: *mut frame_T =
            (*lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>())).w_frame;
        while (*frp).fr_width != Columns.get() && !(*frp).fr_parent.is_null() {
            frp = (*frp).fr_parent;
        }
        while !(*frp).fr_prev.is_null()
            && (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF
            && (*(*frp).fr_win).w_onebuf_opt.wo_wfh != 0
        {
            frp = (*frp).fr_prev;
        }
        while p_ch.get() > old_p_ch as OptInt
            && command_frame_height.get() as ::core::ffi::c_int != 0
        {
            if frp.is_null() {
                emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                p_ch.set(old_p_ch as OptInt);
                break;
            } else {
                let mut h: ::core::ffi::c_int = if ((p_ch.get() - old_p_ch as OptInt)
                    as ::core::ffi::c_int)
                    < (*frp).fr_height - frame_minheight(frp, ::core::ptr::null_mut::<win_T>())
                {
                    (p_ch.get() - old_p_ch as OptInt) as ::core::ffi::c_int
                } else {
                    (*frp).fr_height - frame_minheight(frp, ::core::ptr::null_mut::<win_T>())
                };
                frame_add_height(frp, -h);
                old_p_ch += h;
                frp = (*frp).fr_prev;
            }
        }
        if p_ch.get() < old_p_ch as OptInt
            && command_frame_height.get() as ::core::ffi::c_int != 0
            && !frp.is_null()
        {
            frame_add_height(frp, (old_p_ch as OptInt - p_ch.get()) as ::core::ffi::c_int);
        }
        win_comp_pos();
        cmdline_row.set(Rows.get() - p_ch.get() as ::core::ffi::c_int);
        redraw_cmdline.set(true_0 != 0);
        if msg_scrolled.get() == 0 as ::core::ffi::c_int
            && full_screen.get() as ::core::ffi::c_int != 0
        {
            let mut grid: *mut GridView = default_gridview.ptr();
            if !ui_has(kUIMessages) {
                msg_grid_validate();
                grid = msg_grid_adj.ptr();
            }
            grid_clear(
                grid,
                cmdline_row.get(),
                Rows.get(),
                0 as ::core::ffi::c_int,
                Columns.get(),
                0 as ::core::ffi::c_int,
            );
            msg_row.set(cmdline_row.get());
        }
        (*curtab.get()).tp_ch_used = p_ch.get();
        min_set_ch.set(p_ch.get());
    }
}

unsafe extern "C" fn frame_add_height(mut frp: *mut frame_T, mut n: ::core::ffi::c_int) {
    unsafe {
        frame_new_height(
            frp,
            (*frp).fr_height + n,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        loop {
            frp = (*frp).fr_parent;
            if frp.is_null() {
                break;
            }
            (*frp).fr_height += n;
        }
    }
}

pub unsafe extern "C" fn last_status(mut morewin: bool) {
    unsafe {
        last_status_rec(
            topframe.get(),
            last_stl_height(morewin) > 0 as ::core::ffi::c_int,
            global_stl_height() > 0 as ::core::ffi::c_int,
        );
        win_float_anchor_laststatus();
    }
}

pub unsafe extern "C" fn win_remove_status_line(mut wp: *mut win_T, mut add_hsep: bool) {
    unsafe {
        (*wp).w_status_height = 0 as ::core::ffi::c_int;
        if add_hsep {
            (*wp).w_hsep_height = 1 as ::core::ffi::c_int;
        } else {
            win_new_height(
                wp,
                (if (*wp).w_floating as ::core::ffi::c_int != 0 {
                    (*wp).w_view_height
                } else {
                    (*wp).w_height
                }) + STATUS_HEIGHT as ::core::ffi::c_int,
            );
        }
        comp_col();
        stl_clear_click_defs((*wp).w_status_click_defs, (*wp).w_status_click_defs_size);
        xfree((*wp).w_status_click_defs as *mut ::core::ffi::c_void);
        (*wp).w_status_click_defs_size = 0 as size_t;
        (*wp).w_status_click_defs = ::core::ptr::null_mut::<StlClickDefinition>();
    }
}

unsafe extern "C" fn find_horizontally_resizable_frame(mut fr: *mut frame_T) -> *mut frame_T {
    unsafe {
        let mut fp: *mut frame_T = fr;
        while (*fp).fr_height <= frame_minheight(fp, ::core::ptr::null_mut::<win_T>()) {
            if fp == topframe.get() {
                return ::core::ptr::null_mut::<frame_T>();
            }
            if (*(*fp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
                && !(*fp).fr_prev.is_null()
            {
                fp = (*fp).fr_prev;
            } else {
                fp = (*fp).fr_parent;
            }
        }
        return fp;
    }
}

unsafe extern "C" fn resize_frame_for_status(mut fr: *mut frame_T) -> bool {
    unsafe {
        let mut wp: *mut win_T = (*fr).fr_win;
        let mut fp: *mut frame_T = find_horizontally_resizable_frame(fr);
        if fp.is_null() {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            return false_0 != 0;
        } else if fp != fr {
            frame_new_height(
                fp,
                (*fp).fr_height - 1 as ::core::ffi::c_int,
                false_0 != 0,
                false_0 != 0,
                false_0 != 0,
            );
            frame_fix_height(wp);
            win_comp_pos();
        } else {
            win_new_height(wp, (*wp).w_height - 1 as ::core::ffi::c_int);
        }
        return true_0 != 0;
    }
}

unsafe extern "C" fn resize_frame_for_winbar(mut fr: *mut frame_T) -> bool {
    unsafe {
        let mut wp: *mut win_T = (*fr).fr_win;
        let mut fp: *mut frame_T = find_horizontally_resizable_frame(fr);
        if fp.is_null() || fp == fr {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            return false_0 != 0;
        }
        frame_new_height(
            fp,
            (*fp).fr_height - 1 as ::core::ffi::c_int,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        win_new_height(wp, (*wp).w_height + 1 as ::core::ffi::c_int);
        frame_fix_height(wp);
        win_comp_pos();
        return true_0 != 0;
    }
}

unsafe extern "C" fn last_status_rec(
    mut fr: *mut frame_T,
    mut statusline: bool,
    mut is_stl_global: bool,
) {
    unsafe {
        if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
            let mut wp: *mut win_T = (*fr).fr_win;
            let mut is_last: bool = is_bottom_win(wp);
            if is_last {
                if (*wp).w_status_height != 0 as ::core::ffi::c_int
                    && (!statusline || is_stl_global as ::core::ffi::c_int != 0)
                {
                    win_remove_status_line(wp, false_0 != 0);
                } else if (*wp).w_status_height == 0 as ::core::ffi::c_int
                    && !is_stl_global
                    && statusline as ::core::ffi::c_int != 0
                {
                    (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
                    if !resize_frame_for_status(fr) {
                        return;
                    }
                    comp_col();
                }
                if abs((*wp).w_height - (*wp).w_prev_height) == 1 as ::core::ffi::c_int {
                    (*wp).w_prev_height = (*wp).w_height;
                }
            } else if (*wp).w_status_height != 0 as ::core::ffi::c_int
                && is_stl_global as ::core::ffi::c_int != 0
            {
                win_remove_status_line(wp, true_0 != 0);
            } else if (*wp).w_status_height == 0 as ::core::ffi::c_int && !is_stl_global {
                (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
                (*wp).w_hsep_height = 0 as ::core::ffi::c_int;
                comp_col();
            }
        } else {
            let mut fp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            fp = (*fr).fr_child;
            while !fp.is_null() {
                last_status_rec(fp, statusline, is_stl_global);
                fp = (*fp).fr_next;
            }
        };
    }
}

pub unsafe extern "C" fn set_winbar_win(
    mut wp: *mut win_T,
    mut make_room: bool,
    mut valid_cursor: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut winbar_height: ::core::ffi::c_int = if (*wp).w_floating as ::core::ffi::c_int != 0 {
            if *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }
        } else if *p_wbr.get() as ::core::ffi::c_int != NUL
            || *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
        {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        if (*wp).w_winbar_height != winbar_height {
            if winbar_height == 1 as ::core::ffi::c_int
                && (*wp).w_view_height <= 1 as ::core::ffi::c_int
            {
                if (*wp).w_floating {
                    emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                    return NOTDONE;
                } else if !make_room || !resize_frame_for_winbar((*wp).w_frame) {
                    return FAIL;
                }
            }
            (*wp).w_winbar_height = winbar_height;
            win_set_inner_size(wp, valid_cursor);
            if winbar_height == 0 as ::core::ffi::c_int {
                stl_clear_click_defs((*wp).w_winbar_click_defs, (*wp).w_winbar_click_defs_size);
                xfree((*wp).w_winbar_click_defs as *mut ::core::ffi::c_void);
                (*wp).w_winbar_click_defs_size = 0 as size_t;
                (*wp).w_winbar_click_defs = ::core::ptr::null_mut::<StlClickDefinition>();
            }
        }
        return OK;
    }
}

pub unsafe extern "C" fn set_winbar(mut make_room: bool) {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if set_winbar_win(wp, make_room, true_0 != 0) == FAIL {
                break;
            }
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn tabline_height() -> ::core::ffi::c_int {
    unsafe {
        if ui_has(kUITabline) {
            return 0 as ::core::ffi::c_int;
        }
        debug_assert!(!(*first_tabpage.ptr()).is_null(), "first_tabpage");
        match p_stal.get() {
            0 => return 0 as ::core::ffi::c_int,
            1 => {
                return if (*first_tabpage.get()).tp_next.is_null() {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                };
            }
            _ => {}
        }
        return 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn global_winbar_height() -> ::core::ffi::c_int {
    unsafe {
        return if *p_wbr.get() as ::core::ffi::c_int != NUL {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn global_stl_height() -> ::core::ffi::c_int {
    return if p_ls.get() == 3 as OptInt {
        STATUS_HEIGHT as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}

pub unsafe extern "C" fn last_stl_height(mut morewin: bool) -> ::core::ffi::c_int {
    unsafe {
        return if p_ls.get() > 1 as OptInt
            || p_ls.get() == 1 as OptInt
                && (morewin as ::core::ffi::c_int != 0
                    || !one_window(firstwin.get(), ::core::ptr::null_mut::<tabpage_T>()))
        {
            STATUS_HEIGHT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn min_rows(mut tp: *mut tabpage_T) -> ::core::ffi::c_int {
    unsafe {
        if (*firstwin.ptr()).is_null() {
            return MIN_LINES as ::core::ffi::c_int;
        }
        let mut total: ::core::ffi::c_int =
            frame_minheight((*tp).tp_topframe, ::core::ptr::null_mut::<win_T>());
        total += tabline_height() + global_stl_height();
        if (if tp == curtab.get() {
            p_ch.get()
        } else {
            (*tp).tp_ch_used
        }) > 0 as OptInt
        {
            total += 1;
        }
        return total;
    }
}

pub unsafe extern "C" fn min_rows_for_all_tabpages() -> ::core::ffi::c_int {
    unsafe {
        if (*firstwin.ptr()).is_null() {
            return MIN_LINES as ::core::ffi::c_int;
        }
        let mut total: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut n: ::core::ffi::c_int =
                frame_minheight((*tp).tp_topframe, ::core::ptr::null_mut::<win_T>());
            if (if tp == curtab.get() {
                p_ch.get()
            } else {
                (*tp).tp_ch_used
            }) > 0 as OptInt
            {
                n += 1;
            }
            total = if total > n { total } else { n };
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        total += tabline_height() + global_stl_height();
        return total;
    }
}

pub unsafe extern "C" fn only_one_window() -> bool {
    unsafe {
        if !(*first_tabpage.get()).tp_next.is_null() {
            return false_0 != 0;
        }
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if !(*wp).w_buffer.is_null()
                && (!(bt_help((*wp).w_buffer) as ::core::ffi::c_int != 0 && !bt_help(curbuf.get())
                    || (*wp).w_floating as ::core::ffi::c_int != 0
                    || (*wp).w_onebuf_opt.wo_pvw != 0)
                    || wp == curwin.get())
                && !is_aucmd_win(wp)
            {
                count += 1;
            }
            wp = (*wp).w_next;
        }
        return count <= 1 as ::core::ffi::c_int;
    }
}
