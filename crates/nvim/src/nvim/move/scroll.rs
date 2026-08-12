//! Scrolling the window by a count -- `scrolldown()`, `scrollup()` and the
//! clamped forms.
//!
//! These move `w_topline` (and, under `'smoothscroll'`, `w_skipcol`) by a given
//! number of lines without regard to where the cursor is, leaving the cursor
//! correction to the caller.  The `_clamp` pair stops before the cursor would
//! leave the window at all, which is what CTRL-E/CTRL-Y need; `topline_back`
//! and `botline_forw` step one `lineoff_T` at a time over folds and diff
//! filler, and are the shared primitive the `scroll_cursor_*` family walks
//! with.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::cursor::coladvance;
use crate::src::nvim::decoration::{decor_conceal_line, win_lines_concealed};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later};
use crate::src::nvim::edit::{cursor_down, cursor_up};
use crate::src::nvim::fold::{foldAdjustCursor, hasFolding};
use crate::src::nvim::main::{curbuf, curwin};
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::plines::{
    linetabsize_eol, plines_win, plines_win_nofill, win_get_fill, win_may_fill,
};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::types::{colnr_T, int64_t, linenr_T, win_T};
use crate::src::nvim::winfloat::win_check_anchored_floats;

pub(crate) unsafe extern "C" fn cursor_correct_sms(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_onebuf_opt.wo_sms == 0
            || (*wp).w_onebuf_opt.wo_wrap == 0
            || (*wp).w_cursor.lnum != (*wp).w_topline
        {
            return;
        }
        let mut so: int64_t = get_scrolloff_value(wp);
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
        let mut so_cols: int64_t = if so == 0 as int64_t {
            0 as int64_t
        } else {
            width1 as int64_t + (so - 1 as int64_t) * width2 as int64_t
        };
        let mut space_cols: ::core::ffi::c_int =
            ((*wp).w_view_height - 1 as ::core::ffi::c_int) * width2;
        let mut size: ::core::ffi::c_int = if so == 0 as int64_t {
            0 as ::core::ffi::c_int
        } else {
            linetabsize_eol(wp, (*wp).w_topline)
        };
        if (*wp).w_topline == 1 as linenr_T && (*wp).w_skipcol == 0 as ::core::ffi::c_int {
            so_cols = 0 as int64_t;
        } else if so_cols > (space_cols / 2 as ::core::ffi::c_int) as int64_t {
            so_cols = (space_cols / 2 as ::core::ffi::c_int) as int64_t;
        }
        while so_cols > size as int64_t
            && so_cols - width2 as int64_t >= width1 as int64_t
            && width1 > 0 as ::core::ffi::c_int
        {
            so_cols -= width2 as int64_t;
        }
        if so_cols >= width1 as int64_t && so_cols > size as int64_t {
            so_cols -= width1 as int64_t;
        }
        let mut overlap: ::core::ffi::c_int = if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            sms_marker_overlap(wp, (*wp).w_view_width - width2)
        };
        let mut top: int64_t = (*wp).w_skipcol as int64_t
            + (if so_cols != 0 as int64_t {
                so_cols
            } else {
                overlap as int64_t
            });
        let mut bot: int64_t = ((*wp).w_skipcol as ::core::ffi::c_int
            + width1
            + ((*wp).w_view_height - 1 as ::core::ffi::c_int) * width2)
            as int64_t
            - so_cols;
        validate_virtcol(wp);
        let mut col: colnr_T = (*wp).w_virtcol;
        if (col as int64_t) < top {
            if col < width1 {
                col += width1;
            }
            while width2 > 0 as ::core::ffi::c_int && (col as int64_t) < top {
                col += width2;
            }
        } else {
            while width2 > 0 as ::core::ffi::c_int && col as int64_t >= bot {
                col -= width2;
            }
        }
        if col != (*wp).w_virtcol {
            (*wp).w_curswant = col;
            let reached = coladvance(wp, (*wp).w_curswant);
            (*wp).w_valid &=
                !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
            if !reached
                && (*wp).w_skipcol > 0 as ::core::ffi::c_int
                && (*wp).w_cursor.lnum < (*(*wp).w_buffer).b_ml.ml_line_count
            {
                validate_virtcol(wp);
                if (*wp).w_virtcol < (*wp).w_skipcol as ::core::ffi::c_int + overlap {
                    (*wp).w_cursor.lnum += 1;
                    (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    (*wp).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    (*wp).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
                    (*wp).w_valid &= !VALID_VIRTCOL;
                }
            }
        }
    }
}

pub unsafe extern "C" fn scroll_redraw(mut up: ::core::ffi::c_int, mut count: linenr_T) {
    unsafe {
        let mut prev_topline: linenr_T = (*curwin.get()).w_topline;
        let mut prev_skipcol: ::core::ffi::c_int = (*curwin.get()).w_skipcol as ::core::ffi::c_int;
        let mut prev_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
        let mut prev_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut moved: bool = if up != 0 {
            scrollup(curwin.get(), count, true_0 != 0) as ::core::ffi::c_int
        } else {
            scrolldown(curwin.get(), count, true_0) as ::core::ffi::c_int
        } != 0;
        if get_scrolloff_value(curwin.get()) > 0 as int64_t {
            cursor_correct(curwin.get());
            check_cursor_moved(curwin.get());
            (*curwin.get()).w_valid |= VALID_TOPLINE;
            while (*curwin.get()).w_topline == prev_topline
                && (*curwin.get()).w_skipcol == prev_skipcol
                && (*curwin.get()).w_topfill == prev_topfill
            {
                if up != 0 {
                    if (*curwin.get()).w_cursor.lnum > prev_lnum
                        || cursor_down(1 as ::core::ffi::c_int, false_0 != 0) == FAIL
                    {
                        break;
                    }
                } else if (*curwin.get()).w_cursor.lnum < prev_lnum
                    || prev_topline as ::core::ffi::c_long == 1 as ::core::ffi::c_long
                    || cursor_up(1 as linenr_T, false_0 != 0) == FAIL
                {
                    break;
                }
                check_cursor_moved(curwin.get());
                (*curwin.get()).w_valid |= VALID_TOPLINE;
            }
        }
        if moved {
            (*curwin.get()).w_viewport_invalid = true_0 != 0;
        }
        cursor_correct_sms(curwin.get());
        if (*curwin.get()).w_cursor.lnum != prev_lnum {
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        }
        redraw_later(curwin.get(), UPD_VALID);
    }
}

pub unsafe extern "C" fn scrolldown(
    mut wp: *mut win_T,
    mut line_count: linenr_T,
    mut byfold: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut width1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut width2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
        if do_sms {
            width1 = (*wp).w_view_width - win_col_off(wp);
            width2 = width1 + win_col_off2(wp);
        }
        hasFolding(
            wp,
            (*wp).w_topline,
            &raw mut (*wp).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        );
        validate_cursor(wp);
        let mut todo: ::core::ffi::c_int = line_count as ::core::ffi::c_int;
        while todo > 0 as ::core::ffi::c_int {
            let mut can_fill: bool = (*wp).w_topfill
                < (*wp).w_view_height - 1 as ::core::ffi::c_int
                && (*wp).w_topfill < win_get_fill(wp, (*wp).w_topline);
            if (*wp).w_topline == 1 as linenr_T
                && !can_fill
                && (!do_sms || (*wp).w_skipcol < width1)
            {
                break;
            }
            if do_sms as ::core::ffi::c_int != 0 && (*wp).w_skipcol >= width1 {
                if (*wp).w_skipcol >= width1 + width2 {
                    (*wp).w_skipcol -= width2;
                } else {
                    (*wp).w_skipcol -= width1;
                }
                redraw_later(wp, UPD_NOT_VALID);
                done += 1;
            } else if can_fill {
                (*wp).w_topfill += 1;
                done += 1;
            } else {
                (*wp).w_topline -= 1;
                (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                (*wp).w_topfill = 0 as ::core::ffi::c_int;
                let mut first: linenr_T = 0;
                if hasFolding(
                    wp,
                    (*wp).w_topline,
                    &raw mut first,
                    ::core::ptr::null_mut::<linenr_T>(),
                ) {
                    done += !decor_conceal_line(
                        wp,
                        first as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        false_0 != 0,
                    ) as ::core::ffi::c_int;
                    if byfold == 0 {
                        todo -= ((*wp).w_topline - first - 1 as linenr_T) as ::core::ffi::c_int;
                    }
                    (*wp).w_botline -= (*wp).w_topline - first;
                    (*wp).w_topline = first;
                } else if decor_conceal_line(
                    wp,
                    (*wp).w_topline as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                ) {
                    todo += 1;
                } else if do_sms {
                    let mut size: ::core::ffi::c_int = linetabsize_eol(wp, (*wp).w_topline);
                    if size > width1 {
                        (*wp).w_skipcol = width1 as colnr_T;
                        size -= width1;
                        redraw_later(wp, UPD_NOT_VALID);
                    }
                    while size > width2 {
                        (*wp).w_skipcol += width2;
                        size -= width2;
                    }
                    done += 1;
                } else {
                    done += plines_win_nofill(wp, (*wp).w_topline, true_0 != 0);
                }
            }
            (*wp).w_botline -= 1;
            invalidate_botline_win(wp);
            todo -= 1;
        }
        while (*wp).w_topline > 1 as linenr_T
            && decor_conceal_line(
                wp,
                (*wp).w_topline as ::core::ffi::c_int - 2 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int
                != 0
        {
            (*wp).w_topline -= 1;
            hasFolding(
                wp,
                (*wp).w_topline,
                &raw mut (*wp).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
            );
        }
        (*wp).w_wrow += done;
        (*wp).w_cline_row += done;
        if (*wp).w_cursor.lnum == (*wp).w_topline {
            (*wp).w_cline_row = 0 as ::core::ffi::c_int;
        }
        check_topfill(wp, true_0 != 0);
        let mut wrow: ::core::ffi::c_int = (*wp).w_wrow;
        if (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_view_width != 0 as ::core::ffi::c_int {
            validate_virtcol(wp);
            validate_cheight(wp);
            wrow += (*wp).w_cline_height
                - 1 as ::core::ffi::c_int
                - (*wp).w_virtcol as ::core::ffi::c_int / (*wp).w_view_width;
        }
        let mut moved: bool = false_0 != 0;
        while wrow >= (*wp).w_view_height && (*wp).w_cursor.lnum > 1 as linenr_T {
            let mut first_0: linenr_T = 0;
            if hasFolding(
                wp,
                (*wp).w_cursor.lnum,
                &raw mut first_0,
                ::core::ptr::null_mut::<linenr_T>(),
            ) {
                wrow -= !decor_conceal_line(
                    wp,
                    (*wp).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                ) as ::core::ffi::c_int;
                (*wp).w_cursor.lnum = if first_0 - 1 as linenr_T > 1 as linenr_T {
                    first_0 - 1 as linenr_T
                } else {
                    1 as linenr_T
                };
            } else {
                let c2rust_fresh0 = (*wp).w_cursor.lnum;
                (*wp).w_cursor.lnum = (*wp).w_cursor.lnum - 1;
                wrow -= plines_win(wp, c2rust_fresh0, true_0 != 0);
            }
            (*wp).w_valid &=
                !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
            moved = true_0 != 0;
        }
        if moved {
            foldAdjustCursor(wp);
            coladvance(wp, (*wp).w_curswant);
        }
        (*wp).w_cursor.lnum = if (*wp).w_cursor.lnum > (*wp).w_topline {
            (*wp).w_cursor.lnum
        } else {
            (*wp).w_topline
        };
        return moved;
    }
}

pub unsafe extern "C" fn scrollup(
    mut wp: *mut win_T,
    mut line_count: linenr_T,
    mut byfold: bool,
) -> bool {
    unsafe {
        let mut topline: linenr_T = (*wp).w_topline;
        let mut botline: linenr_T = (*wp).w_botline;
        let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
        if do_sms as ::core::ffi::c_int != 0
            || byfold as ::core::ffi::c_int != 0
                && win_lines_concealed(wp) as ::core::ffi::c_int != 0
            || win_may_fill(wp) as ::core::ffi::c_int != 0
        {
            let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
            let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
            let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let prev_skipcol: colnr_T = (*wp).w_skipcol;
            if do_sms {
                size = linetabsize_eol(wp, (*wp).w_topline);
            }
            let mut todo: ::core::ffi::c_int = line_count as ::core::ffi::c_int;
            while todo > 0 as ::core::ffi::c_int {
                todo += decor_conceal_line(
                    wp,
                    (*wp).w_topline as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                ) as ::core::ffi::c_int;
                if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                    (*wp).w_topfill -= 1;
                } else {
                    let mut lnum: linenr_T = (*wp).w_topline;
                    if byfold {
                        hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                    }
                    if lnum == (*wp).w_topline && do_sms as ::core::ffi::c_int != 0 {
                        let mut add: ::core::ffi::c_int =
                            if (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                                width2
                            } else {
                                width1
                            };
                        (*wp).w_skipcol += add;
                        if (*wp).w_skipcol >= size {
                            if lnum == (*(*wp).w_buffer).b_ml.ml_line_count {
                                (*wp).w_skipcol -= add;
                                break;
                            } else {
                                lnum += 1;
                            }
                        }
                    } else {
                        if lnum >= (*(*wp).w_buffer).b_ml.ml_line_count {
                            break;
                        }
                        lnum += 1;
                    }
                    if lnum > (*wp).w_topline {
                        (*wp).w_botline += lnum - (*wp).w_topline;
                        (*wp).w_topline = lnum;
                        (*wp).w_topfill = win_get_fill(wp, lnum);
                        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                        if todo > 1 as ::core::ffi::c_int && do_sms as ::core::ffi::c_int != 0 {
                            size = linetabsize_eol(wp, (*wp).w_topline);
                        }
                    }
                }
                todo -= 1;
            }
            if prev_skipcol > 0 as ::core::ffi::c_int || (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                redraw_later(wp, UPD_NOT_VALID);
            }
        } else {
            (*wp).w_topline += line_count;
            (*wp).w_botline += line_count;
        }
        (*wp).w_topline = if (*wp).w_topline < (*(*wp).w_buffer).b_ml.ml_line_count {
            (*wp).w_topline
        } else {
            (*(*wp).w_buffer).b_ml.ml_line_count
        };
        (*wp).w_botline = if (*wp).w_botline < (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T
        {
            (*wp).w_botline
        } else {
            (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T
        };
        check_topfill(wp, false_0 != 0);
        hasFolding(
            wp,
            (*wp).w_topline,
            &raw mut (*wp).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        );
        (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
        if (*wp).w_cursor.lnum < (*wp).w_topline {
            (*wp).w_cursor.lnum = (*wp).w_topline;
            (*wp).w_valid &=
                !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
            coladvance(wp, (*wp).w_curswant);
        }
        let mut moved: bool = topline != (*wp).w_topline || botline != (*wp).w_botline;
        return moved;
    }
}

pub unsafe extern "C" fn adjust_skipcol() {
    unsafe {
        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0
            || (*curwin.get()).w_onebuf_opt.wo_sms == 0
            || (*curwin.get()).w_cursor.lnum != (*curwin.get()).w_topline
        {
            return;
        }
        let mut width1: ::core::ffi::c_int =
            (*curwin.get()).w_view_width - win_col_off(curwin.get());
        if width1 <= 0 as ::core::ffi::c_int {
            return;
        }
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin.get());
        let mut so: int64_t = get_scrolloff_value(curwin.get());
        let mut scrolloff_cols: int64_t = if so == 0 as int64_t {
            0 as int64_t
        } else {
            width1 as int64_t + (so - 1 as int64_t) * width2 as int64_t
        };
        let mut scrolled: bool = false_0 != 0;
        validate_cheight(curwin.get());
        if (*curwin.get()).w_cline_height == (*curwin.get()).w_view_height
            && plines_win(curwin.get(), (*curwin.get()).w_cursor.lnum, false_0 != 0)
                <= (*curwin.get()).w_view_height
        {
            reset_skipcol(curwin.get());
            return;
        }
        validate_virtcol(curwin.get());
        let mut overlap: ::core::ffi::c_int =
            sms_marker_overlap(curwin.get(), (*curwin.get()).w_view_width - width2);
        while (*curwin.get()).w_skipcol > 0 as ::core::ffi::c_int
            && ((*curwin.get()).w_virtcol as int64_t)
                < ((*curwin.get()).w_skipcol as ::core::ffi::c_int + overlap) as int64_t
                    + scrolloff_cols
        {
            if (*curwin.get()).w_skipcol >= width1 + width2 {
                (*curwin.get()).w_skipcol -= width2;
            } else {
                (*curwin.get()).w_skipcol -= width1;
            }
            scrolled = true_0 != 0;
        }
        if scrolled {
            validate_virtcol(curwin.get());
            redraw_later(curwin.get(), UPD_NOT_VALID);
            return;
        }
        let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut col: int64_t = (*curwin.get()).w_virtcol as int64_t + scrolloff_cols;
        if scrolloff_cols > 0 as int64_t {
            let mut size: ::core::ffi::c_int =
                linetabsize_eol(curwin.get(), (*curwin.get()).w_topline);
            size = width1 + width2 * ((size - width1 + width2 - 1 as ::core::ffi::c_int) / width2);
            while col > size as int64_t {
                col -= width2 as int64_t;
            }
        }
        col -= (*curwin.get()).w_skipcol as int64_t;
        if col >= width1 as int64_t {
            col -= width1 as int64_t;
            row += 1;
        }
        if col > width2 as int64_t {
            row += (col / width2 as int64_t) as ::core::ffi::c_int;
        }
        if row >= (*curwin.get()).w_view_height {
            if (*curwin.get()).w_skipcol == 0 as ::core::ffi::c_int {
                (*curwin.get()).w_skipcol += width1;
                row -= 1;
            }
            if row >= (*curwin.get()).w_view_height {
                (*curwin.get()).w_skipcol += (row - (*curwin.get()).w_view_height) * width2;
            }
            redraw_later(curwin.get(), UPD_NOT_VALID);
        }
    }
}

pub unsafe extern "C" fn check_topfill(mut wp: *mut win_T, mut down: bool) {
    unsafe {
        if (*wp).w_topfill > 0 as ::core::ffi::c_int {
            let mut n: ::core::ffi::c_int = plines_win_nofill(wp, (*wp).w_topline, true_0 != 0);
            if (*wp).w_topfill + n > (*wp).w_view_height {
                if down as ::core::ffi::c_int != 0 && (*wp).w_topline > 1 as linenr_T {
                    (*wp).w_topline -= 1;
                    (*wp).w_topfill = 0 as ::core::ffi::c_int;
                } else {
                    (*wp).w_topfill = (*wp).w_view_height - n;
                    (*wp).w_topfill = if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                        (*wp).w_topfill
                    } else {
                        0 as ::core::ffi::c_int
                    };
                }
            }
        }
        win_check_anchored_floats(wp);
    }
}

pub unsafe extern "C" fn scrolldown_clamp() {
    unsafe {
        let mut can_fill: bool =
            (*curwin.get()).w_topfill < win_get_fill(curwin.get(), (*curwin.get()).w_topline);
        if (*curwin.get()).w_topline <= 1 as linenr_T && !can_fill {
            return;
        }
        validate_cursor(curwin.get());
        let mut end_row: ::core::ffi::c_int = (*curwin.get()).w_wrow;
        if can_fill {
            end_row += 1;
        } else {
            end_row += plines_win_nofill(
                curwin.get(),
                (*curwin.get()).w_topline - 1 as linenr_T,
                true_0 != 0,
            );
        }
        if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            && (*curwin.get()).w_view_width != 0 as ::core::ffi::c_int
        {
            validate_cheight(curwin.get());
            validate_virtcol(curwin.get());
            end_row += (*curwin.get()).w_cline_height
                - 1 as ::core::ffi::c_int
                - (*curwin.get()).w_virtcol as ::core::ffi::c_int / (*curwin.get()).w_view_width;
        }
        if (end_row as int64_t)
            < (*curwin.get()).w_view_height as int64_t - get_scrolloff_value(curwin.get())
        {
            if can_fill {
                (*curwin.get()).w_topfill += 1;
                check_topfill(curwin.get(), true_0 != 0);
            } else {
                (*curwin.get()).w_topline -= 1;
                (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
            }
            hasFolding(
                curwin.get(),
                (*curwin.get()).w_topline,
                &raw mut (*curwin.get()).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            (*curwin.get()).w_botline -= 1;
            (*curwin.get()).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
        }
    }
}

pub unsafe extern "C" fn scrollup_clamp() {
    unsafe {
        if (*curwin.get()).w_topline == (*curbuf.get()).b_ml.ml_line_count
            && (*curwin.get()).w_topfill == 0 as ::core::ffi::c_int
        {
            return;
        }
        validate_cursor(curwin.get());
        let mut start_row: ::core::ffi::c_int = (*curwin.get()).w_wrow
            - plines_win_nofill(curwin.get(), (*curwin.get()).w_topline, true_0 != 0)
            - (*curwin.get()).w_topfill;
        if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            && (*curwin.get()).w_view_width != 0 as ::core::ffi::c_int
        {
            validate_virtcol(curwin.get());
            start_row -=
                (*curwin.get()).w_virtcol as ::core::ffi::c_int / (*curwin.get()).w_view_width;
        }
        if start_row as int64_t >= get_scrolloff_value(curwin.get()) {
            if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
                (*curwin.get()).w_topfill -= 1;
            } else {
                hasFolding(
                    curwin.get(),
                    (*curwin.get()).w_topline,
                    ::core::ptr::null_mut::<linenr_T>(),
                    &raw mut (*curwin.get()).w_topline,
                );
                (*curwin.get()).w_topline += 1;
            }
            (*curwin.get()).w_botline += 1;
            (*curwin.get()).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
        }
    }
}

pub(crate) unsafe extern "C" fn topline_back_winheight(
    mut wp: *mut win_T,
    mut lp: *mut lineoff_T,
    mut winheight: ::core::ffi::c_int,
) {
    unsafe {
        if (*lp).fill < win_get_fill(wp, (*lp).lnum) {
            (*lp).fill += 1;
            (*lp).height = 1 as ::core::ffi::c_int;
        } else {
            (*lp).lnum -= 1;
            (*lp).fill = 0 as ::core::ffi::c_int;
            if (*lp).lnum < 1 as linenr_T {
                (*lp).height = MAXCOL as ::core::ffi::c_int;
            } else if hasFolding(
                wp,
                (*lp).lnum,
                &raw mut (*lp).lnum,
                ::core::ptr::null_mut::<linenr_T>(),
            ) {
                (*lp).height = !decor_conceal_line(
                    wp,
                    (*lp).lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                ) as ::core::ffi::c_int;
            } else {
                (*lp).height = plines_win_nofill(wp, (*lp).lnum, winheight != 0);
            }
        };
    }
}

pub(crate) unsafe extern "C" fn topline_back(mut wp: *mut win_T, mut lp: *mut lineoff_T) {
    unsafe {
        topline_back_winheight(wp, lp, true_0);
    }
}

pub(crate) unsafe extern "C" fn botline_forw(mut wp: *mut win_T, mut lp: *mut lineoff_T) {
    unsafe {
        if (*lp).fill < win_get_fill(wp, (*lp).lnum + 1 as linenr_T) {
            (*lp).fill += 1;
            (*lp).height = 1 as ::core::ffi::c_int;
        } else {
            (*lp).lnum += 1;
            (*lp).fill = 0 as ::core::ffi::c_int;
            debug_assert!(!(*wp).w_buffer.is_null(), "wp->w_buffer != 0");
            if (*lp).lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
                (*lp).height = MAXCOL as ::core::ffi::c_int;
            } else if hasFolding(
                wp,
                (*lp).lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*lp).lnum,
            ) {
                (*lp).height = !decor_conceal_line(
                    wp,
                    (*lp).lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                ) as ::core::ffi::c_int;
            } else {
                (*lp).height = plines_win_nofill(wp, (*lp).lnum, true_0 != 0);
            }
        };
    }
}
