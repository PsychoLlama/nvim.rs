//! Whole-page movement and `'cursorbind'` -- `pagescroll()` and
//! `do_check_cursorbind()`.
//!
//! [`pagescroll`] is CTRL-F/CTRL-B and the `'smoothscroll'`-aware half-page
//! forms: a page is a window's worth of *screen* lines, so it walks folds and
//! wrapped lines rather than counting buffer lines.
//! [`do_check_cursorbind`] propagates the cursor to every other
//! `'cursorbind'` window, which is a movement decision made once per command
//! rather than per window.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::decoration::win_lines_concealed;
use crate::src::nvim::diff::diff_get_corresponding_line;
use crate::src::nvim::drawscreen::{UPD_VALID, redraw_later};
use crate::src::nvim::edit::{beginline, cursor_down_inner, cursor_up_inner};
use crate::src::nvim::fold::foldAdjustCursor;
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    Rows, VIsual_active, VIsual_select, curbuf, curtab, curwin, firstwin, lastwin, p_sol, p_window,
    restart_edit,
};
use crate::src::nvim::mbyte::mb_adjust_cursor;
use crate::src::nvim::normal::{nv_g_home_m_cmd, nv_screengo};
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::plines::plines_m_win;
use crate::src::nvim::pos::equalpos;
use crate::src::nvim::search::FORWARD;
use crate::src::nvim::types::{
    Direction, OptInt, buf_T, cmdarg_T, colnr_T, int64_t, linenr_T, oparg_T, pos_T, win_T,
};

pub unsafe extern "C" fn pagescroll(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
    mut half: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut did_move: bool = false_0 != 0;
        let mut buflen: ::core::ffi::c_int =
            (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int;
        let mut prev_col: colnr_T = (*curwin.get()).w_cursor.col;
        let mut prev_curswant: colnr_T = (*curwin.get()).w_curswant;
        let mut prev_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut oa: oparg_T = oparg_T {
            op_type: 0 as ::core::ffi::c_int,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        };
        let mut ca: cmdarg_T = cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        ca.oap = &raw mut oa;
        if half {
            if count != 0 {
                (*curwin.get()).w_onebuf_opt.wo_scr = (if (*curwin.get()).w_view_height < count {
                    (*curwin.get()).w_view_height
                } else {
                    count
                }) as OptInt;
            }
            count = if (*curwin.get()).w_view_height
                < (*curwin.get()).w_onebuf_opt.wo_scr as ::core::ffi::c_int
            {
                (*curwin.get()).w_view_height
            } else {
                (*curwin.get()).w_onebuf_opt.wo_scr as ::core::ffi::c_int
            };
            let mut curscount: ::core::ffi::c_int = count;
            if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                && ((*curwin.get()).w_topline
                    + (*curwin.get()).w_view_height as linenr_T
                    + count as linenr_T
                    > buflen as linenr_T
                    || win_lines_concealed(curwin.get()) as ::core::ffi::c_int != 0)
            {
                let mut n: ::core::ffi::c_int = plines_correct_topline(
                    curwin.get(),
                    (*curwin.get()).w_topline,
                    ::core::ptr::null_mut::<linenr_T>(),
                    false_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if n - count < (*curwin.get()).w_view_height
                    && (*curwin.get()).w_topline < buflen as linenr_T
                {
                    n += plines_m_win(
                        curwin.get(),
                        (*curwin.get()).w_topline + 1 as linenr_T,
                        buflen as linenr_T,
                        (*curwin.get()).w_view_height + count,
                    );
                }
                if n < (*curwin.get()).w_view_height + count {
                    count = n - (*curwin.get()).w_view_height;
                }
            }
            if count > 0 as ::core::ffi::c_int {
                did_move = scroll_with_sms(dir, count, &raw mut curscount);
                (*curwin.get()).w_cursor.lnum = prev_lnum;
                (*curwin.get()).w_cursor.col = prev_col;
                (*curwin.get()).w_curswant = prev_curswant;
            }
            if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 {
                nv_screengo(
                    &raw mut oa,
                    dir as ::core::ffi::c_int,
                    curscount,
                    true_0 != 0,
                );
            } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                cursor_down_inner(curwin.get(), curscount, true_0 != 0);
            } else {
                cursor_up_inner(curwin.get(), curscount as linenr_T, true_0 != 0);
            }
        } else {
            count *= if firstwin.get() == lastwin.get()
                && p_window.get() > 0 as OptInt
                && p_window.get() < (Rows.get() - 1 as ::core::ffi::c_int) as OptInt
            {
                if 1 as ::core::ffi::c_int
                    > p_window.get() as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                {
                    1 as ::core::ffi::c_int
                } else {
                    p_window.get() as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                }
            } else {
                get_scroll_overlap(dir)
            };
            did_move = scroll_with_sms(dir, count, &raw mut count);
            if did_move {
                validate_botline_win(curwin.get());
                let mut lnum: linenr_T =
                    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                        (*curwin.get()).w_topline
                    } else {
                        (*curwin.get()).w_botline - 1 as linenr_T
                    };
                (*curwin.get()).w_cursor.lnum = if lnum > 1 as linenr_T {
                    lnum
                } else {
                    1 as linenr_T
                };
            }
        }
        if get_scrolloff_value(curwin.get()) > 0 as int64_t {
            cursor_correct(curwin.get());
        }
        foldAdjustCursor(curwin.get());
        did_move = did_move as ::core::ffi::c_int != 0
            || prev_col != (*curwin.get()).w_cursor.col
            || prev_lnum != (*curwin.get()).w_cursor.lnum;
        if !did_move {
            beep_flush();
        } else if (*curwin.get()).w_onebuf_opt.wo_sms == 0 {
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        } else if p_sol.get() != 0 {
            nv_g_home_m_cmd(&raw mut ca);
        }
        return if did_move as ::core::ffi::c_int != 0 {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn do_check_cursorbind() {
    unsafe {
        static prev_curwin: GlobalCell<*mut win_T> =
            GlobalCell::new(::core::ptr::null_mut::<win_T>());
        static prev_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        });
        if curwin.get() == prev_curwin.get()
            && equalpos((*curwin.get()).w_cursor, prev_cursor.get()) as ::core::ffi::c_int != 0
        {
            return;
        }
        prev_curwin.set(curwin.get());
        prev_cursor.set((*curwin.get()).w_cursor);
        let mut line: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut col: colnr_T = (*curwin.get()).w_cursor.col;
        let mut coladd: colnr_T = (*curwin.get()).w_cursor.coladd;
        let mut curswant: colnr_T = (*curwin.get()).w_curswant;
        let mut set_curswant: bool = (*curwin.get()).w_set_curswant != 0;
        let mut old_curwin: *mut win_T = curwin.get();
        let mut old_curbuf: *mut buf_T = curbuf.get();
        let mut old_VIsual_select: ::core::ffi::c_int = VIsual_select.get() as ::core::ffi::c_int;
        let mut old_VIsual_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
        VIsual_active.set(false_0 != 0);
        VIsual_select.set(VIsual_active.get());
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            curwin.set(wp);
            curbuf.set((*curwin.get()).w_buffer);
            if curwin.get() != old_curwin && (*curwin.get()).w_onebuf_opt.wo_crb != 0 {
                if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
                    (*curwin.get()).w_cursor.lnum = diff_get_corresponding_line(old_curbuf, line);
                } else {
                    (*curwin.get()).w_cursor.lnum = line;
                }
                (*curwin.get()).w_cursor.col = col;
                (*curwin.get()).w_cursor.coladd = coladd;
                (*curwin.get()).w_curswant = curswant;
                (*curwin.get()).w_set_curswant = set_curswant as ::core::ffi::c_int;
                let mut restart_edit_save: ::core::ffi::c_int = restart_edit.get();
                restart_edit.set(true_0);
                check_cursor(curwin.get());
                if (*curwin.get()).w_onebuf_opt.wo_scb == 0 {
                    validate_cursor(curwin.get());
                }
                restart_edit.set(restart_edit_save);
                mb_adjust_cursor();
                redraw_later(curwin.get(), UPD_VALID);
                if (*curwin.get()).w_onebuf_opt.wo_scb == 0 {
                    update_topline(curwin.get());
                }
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
            wp = (*wp).w_next;
        }
        VIsual_select.set(old_VIsual_select != 0);
        VIsual_active.set(old_VIsual_active != 0);
        curwin.set(old_curwin);
        curbuf.set(old_curbuf);
    }
}
