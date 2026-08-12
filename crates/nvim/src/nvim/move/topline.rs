//! Deciding which line the window starts at -- `update_topline()` and the
//! validity bookkeeping around it.
//!
//! [`update_topline`] is the entry point every redraw goes through: it decides
//! whether the cursor has left the visible range and, if so, hands off to the
//! `scroll_cursor_*` family to pick a new `w_topline`.  Around it sit
//! `'scrolljump'`, the `'scrolloff'` margin test, the "did the cursor move?"
//! memo that lets a redraw skip the work entirely, and `set_topline`, the
//! explicit form used when something else has already chosen the line.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::buffer::buf_is_empty;
use crate::src::nvim::cursor::check_cursor_lnum;
use crate::src::nvim::decoration::{decor_conceal_line, win_lines_concealed};
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, conceal_cursor_line, redraw_later,
};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::main::{
    curtab, curwin, default_grid, dollar_vcol, first_tabpage, firstwin, mouse_dragging, p_sj, p_so,
    skip_update_topline,
};
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::plines::{getvvcol, win_get_fill};
use crate::src::nvim::types::{OptInt, colnr_T, int64_t, linenr_T, tabpage_T, win_T};
use crate::src::nvim::winfloat::win_check_anchored_floats;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn update_topline(mut wp: *mut win_T) {
    unsafe {
        let mut check_botline: bool = false_0 != 0;
        let mut so_ptr: *mut OptInt = if (*wp).w_onebuf_opt.wo_so >= 0 as OptInt {
            &raw mut (*wp).w_onebuf_opt.wo_so
        } else {
            p_so.ptr()
        };
        let mut save_so: OptInt = *so_ptr;
        if skip_update_topline.get() {
            return;
        }
        if (*default_grid.ptr()).chars.is_null() || (*wp).w_view_height == 0 as ::core::ffi::c_int {
            check_cursor_lnum(wp);
            (*wp).w_topline = (*wp).w_cursor.lnum;
            (*wp).w_botline = (*wp).w_topline;
            (*wp).w_viewport_invalid = true_0 != 0;
            (*wp).w_scbind_pos = 1 as ::core::ffi::c_int;
            return;
        }
        check_cursor_moved(wp);
        if (*wp).w_valid & VALID_TOPLINE != 0 {
            return;
        }
        if mouse_dragging.get() > 0 as ::core::ffi::c_int {
            *so_ptr = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as OptInt;
        }
        let mut old_topline: linenr_T = (*wp).w_topline;
        let mut old_topfill: ::core::ffi::c_int = (*wp).w_topfill;
        if buf_is_empty((*wp).w_buffer) {
            if (*wp).w_topline != 1 as linenr_T {
                redraw_later(wp, UPD_NOT_VALID);
            }
            (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
            (*wp).w_botline = 2 as ::core::ffi::c_int as linenr_T;
            (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
            (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
            (*wp).w_viewport_invalid = true_0 != 0;
            (*wp).w_scbind_pos = 1 as ::core::ffi::c_int;
        } else {
            let mut check_topline: bool = false_0 != 0;
            if (*wp).w_topline > 1 as linenr_T || (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                if (*wp).w_cursor.lnum < (*wp).w_topline {
                    check_topline = true_0 != 0;
                } else if check_top_offset(wp) {
                    check_topline = true_0 != 0;
                } else if (*wp).w_skipcol > 0 as ::core::ffi::c_int
                    && (*wp).w_cursor.lnum == (*wp).w_topline
                {
                    let mut vcol: colnr_T = 0;
                    getvvcol(
                        wp,
                        &raw mut (*wp).w_cursor,
                        &raw mut vcol,
                        ::core::ptr::null_mut::<colnr_T>(),
                        ::core::ptr::null_mut::<colnr_T>(),
                    );
                    let mut overlap: ::core::ffi::c_int =
                        sms_marker_overlap(wp, -1 as ::core::ffi::c_int);
                    if (*wp).w_skipcol as ::core::ffi::c_int + overlap > vcol {
                        check_topline = true_0 != 0;
                    }
                }
            }
            if !check_topline && (*wp).w_topfill > win_get_fill(wp, (*wp).w_topline) {
                check_topline = true_0 != 0;
            }
            if check_topline {
                let mut halfheight: ::core::ffi::c_int =
                    (*wp).w_view_height / 2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                if halfheight < 2 as ::core::ffi::c_int {
                    halfheight = 2 as ::core::ffi::c_int;
                }
                let mut n: int64_t = 0;
                if win_lines_concealed(wp) {
                    n = 0 as int64_t;
                    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
                    while (lnum as OptInt) < (*wp).w_topline as OptInt + *so_ptr {
                        debug_assert!(!(*wp).w_buffer.is_null(), "wp->w_buffer != 0");
                        if lnum >= (*(*wp).w_buffer).b_ml.ml_line_count || {
                            n += !decor_conceal_line(wp, lnum as ::core::ffi::c_int, false_0 != 0)
                                as ::core::ffi::c_int as int64_t;
                            n >= halfheight as int64_t
                        } {
                            break;
                        }
                        hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                        lnum += 1;
                    }
                } else {
                    n = ((*wp).w_topline as OptInt + *so_ptr - (*wp).w_cursor.lnum as OptInt)
                        as int64_t;
                }
                if n >= halfheight as int64_t {
                    scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
                } else {
                    scroll_cursor_top(wp, scrolljump_value(wp), false_0);
                    check_botline = true_0 != 0;
                }
            } else {
                hasFolding(
                    wp,
                    (*wp).w_topline,
                    &raw mut (*wp).w_topline,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                check_botline = true_0 != 0;
            }
        }
        if check_botline {
            if (*wp).w_valid & VALID_BOTLINE_AP == 0 {
                validate_botline_win(wp);
            }
            debug_assert!(!(*wp).w_buffer.is_null(), "wp->w_buffer != 0");
            if (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
                if (*wp).w_cursor.lnum < (*wp).w_botline {
                    if (*wp).w_cursor.lnum as OptInt >= (*wp).w_botline as OptInt - *so_ptr
                        || win_lines_concealed(wp) as ::core::ffi::c_int != 0
                    {
                        let mut loff: lineoff_T = lineoff_T {
                            lnum: 0,
                            fill: 0,
                            height: 0,
                        };
                        let mut n_0: ::core::ffi::c_int = (*wp).w_empty_rows;
                        loff.lnum = (*wp).w_cursor.lnum;
                        hasFolding(
                            wp,
                            loff.lnum,
                            ::core::ptr::null_mut::<linenr_T>(),
                            &raw mut loff.lnum,
                        );
                        loff.fill = 0 as ::core::ffi::c_int;
                        n_0 += (*wp).w_filler_rows;
                        loff.height = 0 as ::core::ffi::c_int;
                        while loff.lnum < (*wp).w_botline
                            && ((loff.lnum + 1 as linenr_T) < (*wp).w_botline
                                || loff.fill == 0 as ::core::ffi::c_int)
                        {
                            n_0 += loff.height;
                            if n_0 as OptInt >= *so_ptr {
                                break;
                            }
                            botline_forw(wp, &raw mut loff);
                        }
                        if n_0 as OptInt >= *so_ptr {
                            check_botline = false_0 != 0;
                        }
                    } else {
                        check_botline = false_0 != 0;
                    }
                }
                if check_botline {
                    let mut n_1: int64_t = 0 as int64_t;
                    if win_lines_concealed(wp) {
                        let mut lnum_0: linenr_T = (*wp).w_cursor.lnum;
                        while (lnum_0 as OptInt) >= (*wp).w_botline as OptInt - *so_ptr {
                            if lnum_0 <= 0 as linenr_T || {
                                n_1 += !decor_conceal_line(
                                    wp,
                                    lnum_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    false_0 != 0,
                                ) as ::core::ffi::c_int
                                    as int64_t;
                                n_1 > ((*wp).w_view_height + 1 as ::core::ffi::c_int) as int64_t
                            } {
                                break;
                            }
                            hasFolding(
                                wp,
                                lnum_0,
                                &raw mut lnum_0,
                                ::core::ptr::null_mut::<linenr_T>(),
                            );
                            lnum_0 -= 1;
                        }
                    } else {
                        n_1 = (((*wp).w_cursor.lnum - (*wp).w_botline + 1 as linenr_T) as OptInt
                            + *so_ptr) as int64_t;
                    }
                    if n_1 <= ((*wp).w_view_height + 1 as ::core::ffi::c_int) as int64_t {
                        scroll_cursor_bot(wp, scrolljump_value(wp), false_0 != 0);
                    } else {
                        scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
                    }
                }
            }
        }
        (*wp).w_valid |= VALID_TOPLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
        win_check_anchored_floats(wp);
        if (*wp).w_topline != old_topline || (*wp).w_topfill != old_topfill {
            dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
            redraw_later(wp, UPD_VALID);
            if (*wp).w_onebuf_opt.wo_sms == 0 {
                reset_skipcol(wp);
            } else if (*wp).w_skipcol != 0 as ::core::ffi::c_int {
                redraw_later(wp, UPD_SOME_VALID);
            }
            if (*wp).w_cursor.lnum == (*wp).w_topline {
                validate_cursor(wp);
            }
        }
        *so_ptr = save_so;
    }
}

unsafe extern "C" fn scrolljump_value(mut wp: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        let mut result: ::core::ffi::c_int = if p_sj.get() >= 0 as OptInt {
            p_sj.get() as ::core::ffi::c_int
        } else {
            (*wp).w_view_height * -p_sj.get() as ::core::ffi::c_int / 100 as ::core::ffi::c_int
        };
        return result;
    }
}

unsafe extern "C" fn check_top_offset(mut wp: *mut win_T) -> bool {
    unsafe {
        let mut so: int64_t = get_scrolloff_value(wp);
        if ((*wp).w_cursor.lnum as int64_t) < (*wp).w_topline as int64_t + so
            || win_lines_concealed(wp) as ::core::ffi::c_int != 0
        {
            let mut loff: lineoff_T = lineoff_T {
                lnum: 0,
                fill: 0,
                height: 0,
            };
            loff.lnum = (*wp).w_cursor.lnum;
            loff.fill = 0 as ::core::ffi::c_int;
            let mut n: ::core::ffi::c_int = (*wp).w_topfill;
            while (n as int64_t) < so {
                topline_back(wp, &raw mut loff);
                if loff.lnum < (*wp).w_topline
                    || loff.lnum == (*wp).w_topline && loff.fill > 0 as ::core::ffi::c_int
                {
                    break;
                }
                n += loff.height;
            }
            if (n as int64_t) < so {
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn update_curswant_force() {
    unsafe {
        validate_virtcol(curwin.get());
        (*curwin.get()).w_curswant = (*curwin.get()).w_virtcol;
        (*curwin.get()).w_set_curswant = false_0;
    }
}

pub unsafe extern "C" fn update_curswant() {
    unsafe {
        if (*curwin.get()).w_set_curswant != 0 {
            update_curswant_force();
        }
    }
}

pub unsafe extern "C" fn check_cursor_moved(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_cursor.lnum != (*wp).w_valid_cursor.lnum {
            (*wp).w_valid &= !(VALID_WROW
                | VALID_WCOL
                | VALID_VIRTCOL
                | VALID_CHEIGHT
                | VALID_CROW
                | VALID_TOPLINE);
            if wp == curwin.get()
                && (*wp).w_valid_cursor.lnum > 0 as linenr_T
                && (*wp).w_onebuf_opt.wo_cole >= 2 as OptInt
                && !conceal_cursor_line(wp)
                && (decor_conceal_line(
                    wp,
                    (*wp).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    true_0 != 0,
                ) as ::core::ffi::c_int
                    != 0
                    || decor_conceal_line(
                        wp,
                        (*wp).w_valid_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        true_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0)
            {
                changed_window_setting(wp);
            }
            (*wp).w_valid_cursor = (*wp).w_cursor;
            (*wp).w_valid_leftcol = (*wp).w_leftcol;
            (*wp).w_valid_skipcol = (*wp).w_skipcol;
            (*wp).w_viewport_invalid = true_0 != 0;
        } else if (*wp).w_skipcol != (*wp).w_valid_skipcol {
            (*wp).w_valid &= !(VALID_WROW
                | VALID_WCOL
                | VALID_VIRTCOL
                | VALID_CHEIGHT
                | VALID_CROW
                | VALID_BOTLINE
                | VALID_BOTLINE_AP);
            (*wp).w_valid_cursor = (*wp).w_cursor;
            (*wp).w_valid_leftcol = (*wp).w_leftcol;
            (*wp).w_valid_skipcol = (*wp).w_skipcol;
        } else if (*wp).w_cursor.col != (*wp).w_valid_cursor.col
            || (*wp).w_leftcol != (*wp).w_valid_leftcol
            || (*wp).w_cursor.coladd != (*wp).w_valid_cursor.coladd
        {
            (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
            (*wp).w_valid_cursor.col = (*wp).w_cursor.col;
            (*wp).w_valid_leftcol = (*wp).w_leftcol;
            (*wp).w_valid_cursor.coladd = (*wp).w_cursor.coladd;
            (*wp).w_viewport_invalid = true_0 != 0;
        }
    }
}

pub unsafe extern "C" fn changed_window_setting(mut wp: *mut win_T) {
    unsafe {
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
        changed_line_abv_curs_win(wp);
        (*wp).w_valid &= !(VALID_BOTLINE | VALID_BOTLINE_AP | VALID_TOPLINE);
        redraw_later(wp, UPD_NOT_VALID);
    }
}

pub unsafe extern "C" fn changed_window_setting_all() {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                changed_window_setting(wp);
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_topline(mut wp: *mut win_T, mut lnum: linenr_T) {
    unsafe {
        let mut prev_topline: linenr_T = (*wp).w_topline;
        hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
        (*wp).w_botline += lnum - (*wp).w_topline;
        if (*wp).w_botline > (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T {
            (*wp).w_botline = (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T;
        }
        (*wp).w_topline = lnum;
        (*wp).w_topline_was_set = true_0 as ::core::ffi::c_char;
        if lnum != prev_topline {
            (*wp).w_topfill = 0 as ::core::ffi::c_int;
        }
        (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_TOPLINE);
        redraw_later(wp, UPD_VALID);
    }
}
