//! The `i"`/`a"` objects, and the quote scan they are built on.
//!
//! [`find_next_quote`] and [`find_prev_quote`] walk one line looking for an
//! unescaped `quotechar`, where "escaped" is decided by 'quoteescape'.
//! [`current_quote`] is the bookkeeping around them: which of the two quotes
//! the cursor is nearest, whether the current Visual selection already sits
//! inside a quoted string, and how 'selection' shifts both ends.

use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::cursor::{dec_cursor, get_cursor_line_ptr, inc_cursor};
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_mode, curbuf, curwin, p_sel, redraw_cmdline,
};
use crate::src::nvim::mbyte::{utf_head_off, utfc_ptr2len};
use crate::src::nvim::memline::dec;
use crate::src::nvim::pos::{equalpos, lt};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{colnr_T, oparg_T, pos_T, uint8_t};

unsafe extern "C" fn find_next_quote(
    mut line: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut quotechar: ::core::ffi::c_int,
    mut escape: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    loop {
        let mut c: ::core::ffi::c_int = *line.offset(col as isize) as uint8_t as ::core::ffi::c_int;
        if c == NUL {
            return -1 as ::core::ffi::c_int;
        } else {
            if !escape.is_null() && !vim_strchr(escape, c).is_null() {
                col += 1;
                if *line.offset(col as isize) as ::core::ffi::c_int == NUL {
                    return -1 as ::core::ffi::c_int;
                }
            } else if c == quotechar {
                break;
            }
            col += utfc_ptr2len(line.offset(col as isize));
        }
    }
    return col;
}
unsafe extern "C" fn find_prev_quote(
    mut line: *mut ::core::ffi::c_char,
    mut col_start: ::core::ffi::c_int,
    mut quotechar: ::core::ffi::c_int,
    mut escape: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    while col_start > 0 as ::core::ffi::c_int {
        col_start -= 1;
        col_start -= utf_head_off(line, line.offset(col_start as isize));
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !escape.is_null() {
            while col_start - n > 0 as ::core::ffi::c_int
                && !vim_strchr(
                    escape,
                    *line.offset((col_start - n - 1 as ::core::ffi::c_int) as isize) as uint8_t
                        as ::core::ffi::c_int,
                )
                .is_null()
            {
                n += 1;
            }
        }
        if n & 1 as ::core::ffi::c_int != 0 {
            col_start -= n;
        } else if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar {
            break;
        }
    }
    return col_start;
}
pub unsafe extern "C" fn current_quote(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut quotechar: ::core::ffi::c_int,
) -> bool {
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut col_end: ::core::ffi::c_int = 0;
    let mut col_start: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    let mut inclusive: bool = false;
    let mut vis_empty: bool = true;
    let mut vis_bef_curs: bool = false;
    let mut did_exclusive_adj: bool = false;
    let mut inside_quotes: bool = false;
    let mut selected_quote: bool = false;
    let mut i: ::core::ffi::c_int = 0;
    let mut restore_vis_bef: bool = false;
    if VIsual_active.get() {
        if (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
            return false;
        }
        vis_bef_curs = lt(VIsual.get(), (*curwin.get()).w_cursor);
        vis_empty = equalpos(VIsual.get(), (*curwin.get()).w_cursor);
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            if vis_bef_curs {
                dec_cursor();
                did_exclusive_adj = true;
            } else if !vis_empty {
                dec(VIsual.ptr());
                did_exclusive_adj = true;
            }
            vis_empty = equalpos(VIsual.get(), (*curwin.get()).w_cursor);
            if !vis_bef_curs && !vis_empty {
                let mut t: pos_T = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor = VIsual.get();
                VIsual.set(t);
                vis_bef_curs = true;
                restore_vis_bef = true;
            }
        }
    }
    if !vis_empty {
        if vis_bef_curs {
            inside_quotes = (*VIsual.ptr()).col > 0 as ::core::ffi::c_int
                && *line.offset(
                    ((*VIsual.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar
                && *line.offset((*curwin.get()).w_cursor.col as isize) as ::core::ffi::c_int != NUL
                && *line.offset(
                    ((*curwin.get()).w_cursor.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar;
            i = (*VIsual.ptr()).col as ::core::ffi::c_int;
            col_end = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        } else {
            inside_quotes = (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                && *line.offset(
                    ((*curwin.get()).w_cursor.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar
                && *line.offset((*VIsual.ptr()).col as isize) as ::core::ffi::c_int != NUL
                && *line.offset(
                    ((*VIsual.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar;
            i = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            col_end = (*VIsual.ptr()).col as ::core::ffi::c_int;
        }
        while i <= col_end {
            if *line.offset(i as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            let c2rust_fresh7 = i;
            i = i + 1;
            if *line.offset(c2rust_fresh7 as isize) as uint8_t as ::core::ffi::c_int != quotechar {
                continue;
            }
            selected_quote = true;
            break;
        }
    }
    '_abort_search: {
        's_368: {
            if !vis_empty
                && *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar
            {
                if vis_bef_curs {
                    col_start = find_next_quote(
                        line,
                        col_start + 1 as ::core::ffi::c_int,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    } else {
                        col_end = find_next_quote(
                            line,
                            col_start + 1 as ::core::ffi::c_int,
                            quotechar,
                            (*curbuf.get()).b_p_qe,
                        );
                        if col_end < 0 as ::core::ffi::c_int {
                            col_end = col_start;
                            col_start = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        }
                    }
                } else {
                    col_end = find_prev_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if *line.offset(col_end as isize) as uint8_t as ::core::ffi::c_int != quotechar
                    {
                        break '_abort_search;
                    } else {
                        col_start =
                            find_prev_quote(line, col_end, quotechar, (*curbuf.get()).b_p_qe);
                        if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int
                            != quotechar
                        {
                            col_start = col_end;
                            col_end = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        }
                    }
                }
            } else if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar
                || !vis_empty
            {
                let mut first_col: ::core::ffi::c_int = col_start;
                if !vis_empty {
                    if vis_bef_curs {
                        first_col = find_next_quote(
                            line,
                            col_start,
                            quotechar,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    } else {
                        first_col = find_prev_quote(
                            line,
                            col_start,
                            quotechar,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    }
                }
                col_start = 0 as ::core::ffi::c_int;
                loop {
                    col_start = find_next_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int || col_start > first_col {
                        break '_abort_search;
                    }
                    col_end = find_next_quote(
                        line,
                        col_start + 1 as ::core::ffi::c_int,
                        quotechar,
                        (*curbuf.get()).b_p_qe,
                    );
                    if col_end < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    }
                    if col_start <= first_col && first_col <= col_end {
                        break 's_368;
                    }
                    col_start = col_end + 1 as ::core::ffi::c_int;
                }
            } else {
                col_start = find_prev_quote(line, col_start, quotechar, (*curbuf.get()).b_p_qe);
                if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int != quotechar {
                    col_start = find_next_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    }
                }
                col_end = find_next_quote(
                    line,
                    col_start + 1 as ::core::ffi::c_int,
                    quotechar,
                    (*curbuf.get()).b_p_qe,
                );
                if col_end < 0 as ::core::ffi::c_int {
                    break '_abort_search;
                }
            }
        }
        if include {
            if ascii_iswhite(
                *line.offset((col_end + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            ) {
                while ascii_iswhite(*line.offset((col_end + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int)
                {
                    col_end += 1;
                }
            } else {
                while col_start > 0 as ::core::ffi::c_int
                    && ascii_iswhite(*line.offset((col_start - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as ::core::ffi::c_int
                        != 0
                {
                    col_start -= 1;
                }
            }
        }
        if !include
            && count < 2 as ::core::ffi::c_int
            && (vis_empty as ::core::ffi::c_int != 0 || !inside_quotes)
        {
            col_start += 1;
        }
        (*curwin.get()).w_cursor.col = col_start as colnr_T;
        if VIsual_active.get() {
            if vis_empty as ::core::ffi::c_int != 0
                || vis_bef_curs as ::core::ffi::c_int != 0
                    && !selected_quote
                    && (inside_quotes as ::core::ffi::c_int != 0
                        || *line.offset((*VIsual.ptr()).col as isize) as uint8_t
                            as ::core::ffi::c_int
                            != quotechar
                            && ((*VIsual.ptr()).col == 0 as ::core::ffi::c_int
                                || *line.offset(
                                    ((*VIsual.ptr()).col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as isize,
                                ) as uint8_t
                                    as ::core::ffi::c_int
                                    != quotechar))
            {
                VIsual.set((*curwin.get()).w_cursor);
                redraw_curbuf_later(UPD_INVERTED);
            }
        } else {
            (*oap).start = (*curwin.get()).w_cursor;
            (*oap).motion_type = kMTCharWise;
        }
        (*curwin.get()).w_cursor.col = col_end as colnr_T;
        if (include as ::core::ffi::c_int != 0
            || count > 1 as ::core::ffi::c_int
            || !vis_empty && inside_quotes as ::core::ffi::c_int != 0)
            && inc_cursor() == 2 as ::core::ffi::c_int
        {
            inclusive = true;
        }
        if VIsual_active.get() {
            if vis_empty as ::core::ffi::c_int != 0 || vis_bef_curs as ::core::ffi::c_int != 0 {
                if *p_sel.get() as ::core::ffi::c_int != 'e' as ::core::ffi::c_int {
                    dec_cursor();
                }
            } else {
                if inside_quotes as ::core::ffi::c_int != 0
                    || !selected_quote
                        && *line.offset((*VIsual.ptr()).col as isize) as uint8_t
                            as ::core::ffi::c_int
                            != quotechar
                        && (*line.offset((*VIsual.ptr()).col as isize) as ::core::ffi::c_int == NUL
                            || *line.offset(
                                ((*VIsual.ptr()).col as ::core::ffi::c_int
                                    + 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as uint8_t as ::core::ffi::c_int
                                != quotechar)
                {
                    dec_cursor();
                    VIsual.set((*curwin.get()).w_cursor);
                }
                (*curwin.get()).w_cursor.col = col_start as colnr_T;
            }
            if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                VIsual_mode.set('v' as ::core::ffi::c_int);
                redraw_cmdline.set(true);
            }
        } else {
            (*oap).inclusive = inclusive;
        }
        return true;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
    {
        if did_exclusive_adj {
            inc_cursor();
        }
        if restore_vis_bef {
            let mut t_0: pos_T = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = VIsual.get();
            VIsual.set(t_0);
        }
    }
    return false;
}
