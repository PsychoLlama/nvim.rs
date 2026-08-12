//! Where a position is on the screen -- `curs_columns()` and the
//! `screenpos()`/`virtcol2col()` builtins.
//!
//! [`curs_columns`] is the horizontal half of the viewport: it computes the
//! cursor's screen column from its virtual column, applies `'sidescroll'` and
//! `'sidescrolloff'` to `w_leftcol`, and picks `w_skipcol` when the line wraps.
//! `textpos2screenpos` answers the same question for an arbitrary position on
//! behalf of `screenpos()`, and `virtcol2col` inverts it.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, redraw_later, win_scroll_lines};
use crate::src::nvim::eval::typval::{
    tv_check_for_number_arg, tv_dict_add_nr, tv_dict_alloc_ret, tv_get_number, tv_get_number_chk,
};
use crate::src::nvim::eval::window::find_win_by_nr_or_id;
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::main::{dollar_vcol, e_invalid_line_number_nr, p_ss};
use crate::src::nvim::mbyte::utf_head_off;
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::mouse::vcol2col;
use crate::src::nvim::option::{get_scrolloff_value, get_sidescrolloff_value};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::plines::{
    getvcol, getvvcol, plines_m_win, plines_win, plines_win_nofill, win_get_fill,
};
use crate::src::nvim::types::{
    EvalFuncData, OptInt, colnr_T, dict_T, int64_t, linenr_T, pos_T, size_t, typval_T, varnumber_T,
    win_T,
};
use crate::src::nvim::winfloat::win_check_anchored_floats;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn curs_columns(mut wp: *mut win_T, mut may_scroll: ::core::ffi::c_int) {
    unsafe {
        let mut startcol: colnr_T = 0;
        let mut endcol: colnr_T = 0;
        update_topline(wp);
        if (*wp).w_valid & VALID_CROW == 0 {
            curs_rows(wp);
        }
        if (*wp).w_cline_folded {
            endcol = (*wp).w_leftcol;
            (*wp).w_virtcol = endcol;
            startcol = (*wp).w_virtcol;
        } else {
            getvvcol(
                wp,
                &raw mut (*wp).w_cursor,
                &raw mut startcol,
                &raw mut (*wp).w_virtcol,
                &raw mut endcol,
            );
        }
        if startcol > dollar_vcol.get() {
            dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
        }
        let mut extra: ::core::ffi::c_int = win_col_off(wp);
        (*wp).w_wcol = (*wp).w_virtcol as ::core::ffi::c_int + extra;
        endcol += extra;
        (*wp).w_wrow = (*wp).w_cline_row;
        let mut n: ::core::ffi::c_int = 0;
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - extra;
        let mut width2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut did_sub_skipcol: bool = false_0 != 0;
        if width1 <= 0 as ::core::ffi::c_int {
            (*wp).w_wcol = (*wp).w_view_width - 1 as ::core::ffi::c_int;
            if (*wp).w_onebuf_opt.wo_wrap != 0 {
                (*wp).w_wrow = (*wp).w_view_height - 1 as ::core::ffi::c_int;
            } else {
                (*wp).w_wrow = (*wp).w_view_height - 1 as ::core::ffi::c_int - (*wp).w_empty_rows;
            }
        } else if (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_view_width != 0 as ::core::ffi::c_int {
            width2 = width1 + win_col_off2(wp);
            if (*wp).w_cursor.lnum == (*wp).w_topline
                && (*wp).w_skipcol > 0 as ::core::ffi::c_int
                && (*wp).w_wcol >= (*wp).w_skipcol
            {
                if (*wp).w_skipcol <= width1 {
                    (*wp).w_wcol -= width2;
                } else {
                    (*wp).w_wcol -= width2
                        * (((*wp).w_skipcol as ::core::ffi::c_int - width1) / width2
                            + 1 as ::core::ffi::c_int);
                }
                did_sub_skipcol = true_0 != 0;
            }
            if (*wp).w_wcol >= (*wp).w_view_width {
                n = ((*wp).w_wcol - (*wp).w_view_width) / width2 + 1 as ::core::ffi::c_int;
                (*wp).w_wcol -= n * width2;
                (*wp).w_wrow += n;
            }
        } else if may_scroll != 0 && !(*wp).w_cline_folded {
            let mut siso: int64_t = get_sidescrolloff_value(wp);
            let mut off_left: int64_t = (startcol - (*wp).w_leftcol) as int64_t - siso;
            let mut off_right: int64_t = (endcol - (*wp).w_leftcol) as int64_t
                - ((*wp).w_view_width as int64_t - siso)
                + 1 as int64_t;
            if off_left < 0 as int64_t || off_right > 0 as int64_t {
                let mut diff: int64_t = if off_left < 0 as int64_t {
                    -off_left
                } else {
                    off_right
                };
                let mut new_leftcol: ::core::ffi::c_int = 0;
                if p_ss.get() == 0 as OptInt
                    || diff >= (width1 / 2 as ::core::ffi::c_int) as int64_t
                    || off_right >= off_left
                {
                    new_leftcol = (*wp).w_wcol - extra - width1 / 2 as ::core::ffi::c_int;
                } else {
                    if diff < p_ss.get() {
                        debug_assert!(p_ss.get() <= 2147483647 as OptInt, "p_ss <= INT_MAX");
                        diff = p_ss.get() as int64_t;
                    }
                    if off_left < 0 as int64_t {
                        new_leftcol =
                            (*wp).w_leftcol as ::core::ffi::c_int - diff as ::core::ffi::c_int;
                    } else {
                        new_leftcol =
                            (*wp).w_leftcol as ::core::ffi::c_int + diff as ::core::ffi::c_int;
                    }
                }
                new_leftcol = if new_leftcol > 0 as ::core::ffi::c_int {
                    new_leftcol
                } else {
                    0 as ::core::ffi::c_int
                };
                if new_leftcol != (*wp).w_leftcol {
                    (*wp).w_leftcol = new_leftcol as colnr_T;
                    win_check_anchored_floats(wp);
                    redraw_later(wp, UPD_NOT_VALID);
                }
            }
            (*wp).w_wcol -= (*wp).w_leftcol as ::core::ffi::c_int;
        } else if (*wp).w_wcol > (*wp).w_leftcol {
            (*wp).w_wcol -= (*wp).w_leftcol as ::core::ffi::c_int;
        } else {
            (*wp).w_wcol = 0 as ::core::ffi::c_int;
        }
        if (*wp).w_cursor.lnum == (*wp).w_topline {
            (*wp).w_wrow += (*wp).w_topfill;
        } else {
            (*wp).w_wrow += win_get_fill(wp, (*wp).w_cursor.lnum);
        }
        let mut plines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut so: int64_t = get_scrolloff_value(wp);
        let mut prev_skipcol: colnr_T = (*wp).w_skipcol;
        if ((*wp).w_wrow >= (*wp).w_view_height
            || (prev_skipcol > 0 as ::core::ffi::c_int
                || (*wp).w_wrow as int64_t + so >= (*wp).w_view_height as int64_t)
                && {
                    plines = plines_win_nofill(wp, (*wp).w_cursor.lnum, false_0 != 0);
                    plines - 1 as ::core::ffi::c_int >= (*wp).w_view_height
                })
            && (*wp).w_view_height != 0 as ::core::ffi::c_int
            && (*wp).w_cursor.lnum == (*wp).w_topline
            && width2 > 0 as ::core::ffi::c_int
            && (*wp).w_view_width != 0 as ::core::ffi::c_int
        {
            extra = 0 as ::core::ffi::c_int;
            if (*wp).w_skipcol as int64_t + so * width2 as int64_t > (*wp).w_virtcol as int64_t {
                extra = 1 as ::core::ffi::c_int;
            }
            if plines == 0 as ::core::ffi::c_int {
                plines = plines_win(wp, (*wp).w_cursor.lnum, false_0 != 0);
            }
            plines -= 1;
            if plines as int64_t > (*wp).w_wrow as int64_t + so {
                debug_assert!(
                    (*wp).w_wrow as int64_t + so <= 2147483647 as int64_t,
                    "wp->w_wrow + so <= INT_MAX"
                );
                n = ((*wp).w_wrow as int64_t + so) as ::core::ffi::c_int;
            } else {
                n = plines;
            }
            if n as int64_t
                >= ((*wp).w_view_height + (*wp).w_skipcol as ::core::ffi::c_int / width2) as int64_t
                    - so
            {
                extra += 2 as ::core::ffi::c_int;
            }
            if extra == 3 as ::core::ffi::c_int
                || (*wp).w_view_height as int64_t <= so * 2 as int64_t
            {
                n = (*wp).w_virtcol as ::core::ffi::c_int / width2;
                if n > (*wp).w_view_height / 2 as ::core::ffi::c_int {
                    n -= (*wp).w_view_height / 2 as ::core::ffi::c_int;
                } else {
                    n = 0 as ::core::ffi::c_int;
                }
                if n > plines - (*wp).w_view_height + 1 as ::core::ffi::c_int {
                    n = plines - (*wp).w_view_height + 1 as ::core::ffi::c_int;
                }
                (*wp).w_skipcol = (if n > 0 as ::core::ffi::c_int {
                    width1 + (n - 1 as ::core::ffi::c_int) * width2
                } else {
                    0 as ::core::ffi::c_int
                }) as colnr_T;
            } else if extra == 1 as ::core::ffi::c_int {
                debug_assert!(so <= 2147483647 as int64_t, "so <= INT_MAX");
                extra = (((*wp).w_skipcol as int64_t + so * width2 as int64_t
                    - (*wp).w_virtcol as int64_t
                    + width2 as int64_t
                    - 1 as int64_t)
                    / width2 as int64_t) as ::core::ffi::c_int;
                if extra > 0 as ::core::ffi::c_int {
                    if extra * width2 > (*wp).w_skipcol {
                        extra = (*wp).w_skipcol as ::core::ffi::c_int / width2;
                    }
                    (*wp).w_skipcol -= extra * width2;
                }
            } else if extra == 2 as ::core::ffi::c_int {
                endcol = ((n - (*wp).w_view_height + 1 as ::core::ffi::c_int) * width2) as colnr_T;
                while endcol > (*wp).w_virtcol {
                    endcol -= width2;
                }
                (*wp).w_skipcol = if (*wp).w_skipcol > endcol {
                    (*wp).w_skipcol
                } else {
                    endcol
                };
            }
            if did_sub_skipcol {
                (*wp).w_wrow -= ((*wp).w_skipcol as ::core::ffi::c_int
                    - prev_skipcol as ::core::ffi::c_int)
                    / width2;
            } else {
                (*wp).w_wrow -= (*wp).w_skipcol as ::core::ffi::c_int / width2;
            }
            if (*wp).w_wrow >= (*wp).w_view_height {
                extra = (*wp).w_wrow - (*wp).w_view_height + 1 as ::core::ffi::c_int;
                (*wp).w_skipcol += extra * width2;
                (*wp).w_wrow -= extra;
            }
            extra = (prev_skipcol as ::core::ffi::c_int - (*wp).w_skipcol as ::core::ffi::c_int)
                / width2;
            if !(*wp).w_grid.target.is_null() {
                win_scroll_lines(wp, 0 as ::core::ffi::c_int, extra);
            }
        } else if (*wp).w_onebuf_opt.wo_sms == 0 {
            (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
        }
        if prev_skipcol != (*wp).w_skipcol {
            redraw_later(wp, UPD_SOME_VALID);
        }
        redraw_for_cursorcolumn(wp);
        (*wp).w_valid_leftcol = (*wp).w_leftcol;
        (*wp).w_valid_skipcol = (*wp).w_skipcol;
        (*wp).w_valid |= VALID_WCOL | VALID_WROW | VALID_VIRTCOL;
    }
}

pub unsafe extern "C" fn textpos2screenpos(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut rowp: *mut ::core::ffi::c_int,
    mut scolp: *mut ::core::ffi::c_int,
    mut ccolp: *mut ::core::ffi::c_int,
    mut ecolp: *mut ::core::ffi::c_int,
    mut local: bool,
) {
    unsafe {
        let mut scol: colnr_T = 0 as colnr_T;
        let mut ccol: colnr_T = 0 as colnr_T;
        let mut ecol: colnr_T = 0 as colnr_T;
        let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut coloff: colnr_T = 0 as colnr_T;
        let mut visible_row: bool = false_0 != 0;
        let mut is_folded: bool = false_0 != 0;
        let mut lnum: linenr_T = (*pos).lnum;
        if lnum >= (*wp).w_topline && lnum <= (*wp).w_botline {
            is_folded = hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
            row = plines_m_win(wp, (*wp).w_topline, lnum - 1 as linenr_T, INT_MAX);
            row -= adjust_plines_for_skipcol(wp);
            row += if lnum == (*wp).w_topline {
                (*wp).w_topfill
            } else {
                win_get_fill(wp, lnum)
            };
            visible_row = true_0 != 0;
        } else if !local || lnum < (*wp).w_topline {
            row = 0 as ::core::ffi::c_int;
        } else {
            row = (*wp).w_view_height - 1 as ::core::ffi::c_int;
        }
        let mut existing_row: bool =
            lnum > 0 as linenr_T && lnum <= (*(*wp).w_buffer).b_ml.ml_line_count;
        if (local as ::core::ffi::c_int != 0 || visible_row as ::core::ffi::c_int != 0)
            && existing_row as ::core::ffi::c_int != 0
        {
            let off: colnr_T = win_col_off(wp);
            if is_folded {
                row += (if local as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    (*wp).w_winrow + (*wp).w_winrow_off
                }) + 1 as ::core::ffi::c_int;
                coloff = (if local as ::core::ffi::c_int != 0 {
                    0 as colnr_T
                } else {
                    (*wp).w_wincol as colnr_T + (*wp).w_wincol_off as colnr_T
                }) + 1 as colnr_T
                    + off;
            } else {
                debug_assert!(lnum == (*pos).lnum, "lnum == pos->lnum");
                getvcol(wp, pos, &raw mut scol, &raw mut ccol, &raw mut ecol);
                let mut col: colnr_T = scol;
                col += off;
                let mut width: ::core::ffi::c_int =
                    (*wp).w_view_width - off as ::core::ffi::c_int + win_col_off2(wp);
                if (*wp).w_onebuf_opt.wo_wrap != 0
                    && col >= (*wp).w_view_width
                    && width > 0 as ::core::ffi::c_int
                {
                    let mut rowoff: ::core::ffi::c_int = if visible_row as ::core::ffi::c_int != 0 {
                        (col as ::core::ffi::c_int - (*wp).w_view_width) / width
                            + 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    col -= rowoff * width;
                    row += rowoff;
                }
                col -= (*wp).w_leftcol;
                if col >= 0 as ::core::ffi::c_int
                    && col < (*wp).w_view_width
                    && row >= 0 as ::core::ffi::c_int
                    && row < (*wp).w_view_height
                {
                    coloff = (col as ::core::ffi::c_int - scol as ::core::ffi::c_int
                        + (if local as ::core::ffi::c_int != 0 {
                            0 as ::core::ffi::c_int
                        } else {
                            (*wp).w_wincol + (*wp).w_wincol_off
                        })
                        + 1 as ::core::ffi::c_int) as colnr_T;
                    row += (if local as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        (*wp).w_winrow + (*wp).w_winrow_off
                    }) + 1 as ::core::ffi::c_int;
                } else {
                    ecol = 0 as ::core::ffi::c_int as colnr_T;
                    ccol = ecol;
                    scol = ccol;
                    if local {
                        coloff = (if col < 0 as ::core::ffi::c_int {
                            -1 as ::core::ffi::c_int
                        } else {
                            (*wp).w_view_width + 1 as ::core::ffi::c_int
                        }) as colnr_T;
                    } else {
                        row = 0 as ::core::ffi::c_int;
                    }
                }
            }
        }
        *rowp = row;
        *scolp = (scol + coloff) as ::core::ffi::c_int;
        *ccolp = (ccol + coloff) as ::core::ffi::c_int;
        *ecolp = (ecol + coloff) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn f_screenpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_dict_alloc_ret(rettv);
        let mut dict: *mut dict_T = (*rettv).vval.v_dict;
        let mut wp: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if wp.is_null() {
            return;
        }
        let mut pos: pos_T = pos_T {
            lnum: tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as linenr_T,
            col: tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as colnr_T
                - 1 as colnr_T,
            coladd: 0 as colnr_T,
        };
        if pos.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
            semsg_c!(
                gettext(&raw const e_invalid_line_number_nr as *const ::core::ffi::c_char),
                pos.lnum,
            );
            return;
        }
        pos.col = (if pos.col > 0 as ::core::ffi::c_int {
            pos.col as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as colnr_T;
        let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut scol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ccol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ecol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        textpos2screenpos(
            wp,
            &raw mut pos,
            &raw mut row,
            &raw mut scol,
            &raw mut ccol,
            &raw mut ecol,
            false_0 != 0,
        );
        tv_dict_add_nr(
            dict,
            c"row".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            row as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            c"col".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            scol as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            c"curscol".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            ccol as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            c"endcol".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            ecol as varnumber_T,
        );
    }
}

unsafe extern "C" fn virtcol2col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut offset: ::core::ffi::c_int = vcol2col(
            wp,
            lnum,
            vcol as colnr_T - 1 as colnr_T,
            ::core::ptr::null_mut::<colnr_T>(),
        );
        let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
        let mut p: *mut ::core::ffi::c_char = line.offset(offset as isize);
        if *p as ::core::ffi::c_int == NUL {
            if p == line {
                return 0 as ::core::ffi::c_int;
            }
            p = p.offset(
                -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
        }
        return (p.offset_from(line) + 1_isize) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn f_virtcol2col(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        if tv_check_for_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || tv_check_for_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
            || tv_check_for_number_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        let mut wp: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if wp.is_null() {
            return;
        }
        let mut error: bool = false_0 != 0;
        let mut lnum: linenr_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as linenr_T;
        if error as ::core::ffi::c_int != 0
            || lnum < 0 as linenr_T
            || lnum > (*(*wp).w_buffer).b_ml.ml_line_count
        {
            return;
        }
        let mut screencol: ::core::ffi::c_int = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error as ::core::ffi::c_int != 0 || screencol < 0 as ::core::ffi::c_int {
            return;
        }
        (*rettv).vval.v_number = virtcol2col(wp, lnum, screencol) as varnumber_T;
    }
}
