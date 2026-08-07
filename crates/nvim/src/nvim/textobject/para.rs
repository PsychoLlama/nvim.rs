//! Paragraphs and sections: the `{`/`}`/`[[`/`]]` motions and `ip`/`ap`.
//!
//! A paragraph boundary is an empty line, a form feed, or a line matching
//! one of the two-letter nroff macro lists in 'paragraphs'/'sections'.
//! [`startPS`] is that test -- the rest of the tree asks it too -- and
//! [`findpar`] and [`current_par`] are the two shapes built on it.

use super::*;
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_mode, curbuf, curwin, p_para, p_sections,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::utf_head_off;
use crate::src::nvim::memline::{ml_get, ml_get_len};
use crate::src::nvim::search::{BACKWARD, FORWARD, linewhite};
use crate::src::nvim::types::{colnr_T, linenr_T, oparg_T, uint8_t};

pub unsafe extern "C" fn findpar(
    mut pincl: *mut bool,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut what: ::core::ffi::c_int,
    mut both: bool,
) -> bool {
    let mut first: bool = false;
    let mut fold_first: linenr_T = 0;
    let mut fold_last: linenr_T = 0;
    let mut fold_skipped: bool = false;
    let mut curr: linenr_T = (*curwin.get()).w_cursor.lnum;
    loop {
        let c2rust_fresh1 = count;
        count = count - 1;
        if c2rust_fresh1 == 0 {
            break;
        }
        let mut did_skip: bool = false;
        first = true;
        loop {
            if *ml_get(curr) as ::core::ffi::c_int != NUL {
                did_skip = true;
            }
            fold_skipped = false;
            if first as ::core::ffi::c_int != 0
                && hasFolding(curwin.get(), curr, &raw mut fold_first, &raw mut fold_last)
                    as ::core::ffi::c_int
                    != 0
            {
                curr = (if dir > 0 as ::core::ffi::c_int {
                    fold_last
                } else {
                    fold_first
                }) + dir as linenr_T;
                fold_skipped = true;
            }
            if !first
                && did_skip as ::core::ffi::c_int != 0
                && startPS(curr, what, both) as ::core::ffi::c_int != 0
            {
                break;
            }
            if fold_skipped {
                curr = (curr as ::core::ffi::c_int - dir) as linenr_T;
            }
            curr = (curr as ::core::ffi::c_int + dir) as linenr_T;
            if curr < 1 as linenr_T || curr > (*curbuf.get()).b_ml.ml_line_count {
                if count != 0 {
                    return false;
                }
                curr = (curr as ::core::ffi::c_int - dir) as linenr_T;
                break;
            } else {
                first = false;
            }
        }
    }
    setpcmark();
    if both as ::core::ffi::c_int != 0
        && *ml_get(curr) as ::core::ffi::c_int == '}' as ::core::ffi::c_int
    {
        curr += 1;
    }
    (*curwin.get()).w_cursor.lnum = curr;
    if curr == (*curbuf.get()).b_ml.ml_line_count
        && what != '}' as ::core::ffi::c_int
        && dir == FORWARD as ::core::ffi::c_int
    {
        let mut line: *mut ::core::ffi::c_char = ml_get(curr);
        (*curwin.get()).w_cursor.col = ml_get_len(curr);
        if (*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col -= 1;
            (*curwin.get()).w_cursor.col -=
                utf_head_off(line, line.offset((*curwin.get()).w_cursor.col as isize));
            *pincl = true;
        }
    } else {
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    return true;
}
unsafe extern "C" fn inmacro(
    mut opt: *mut ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
) -> bool {
    let mut macro_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    macro_0 = opt;
    while *macro_0.offset(0 as ::core::ffi::c_int as isize) != 0 {
        if (*macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            || *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
                && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int))
            && (*macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                || (*macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int)
                    && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int))
        {
            break;
        }
        macro_0 = macro_0.offset(1);
        if *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        macro_0 = macro_0.offset(1);
    }
    return *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
}
pub unsafe extern "C" fn startPS(
    mut lnum: linenr_T,
    mut para: ::core::ffi::c_int,
    mut both: bool,
) -> bool {
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    if *s as uint8_t as ::core::ffi::c_int == para
        || *s as ::core::ffi::c_int == '\u{c}' as ::core::ffi::c_int
        || both as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
    {
        return true;
    }
    if *s as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        && (inmacro(p_sections.get(), s.offset(1 as ::core::ffi::c_int as isize))
            as ::core::ffi::c_int
            != 0
            || para == 0
                && inmacro(p_para.get(), s.offset(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int
                    != 0)
    {
        return true;
    }
    return false;
}
pub unsafe extern "C" fn current_par(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut type_0: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = OK;
    let mut do_white: ::core::ffi::c_int = false_0;
    if type_0 == 'S' as ::core::ffi::c_int {
        return FAIL;
    }
    let mut start_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    '_extend: {
        if !(VIsual_active.get() as ::core::ffi::c_int != 0 && start_lnum != (*VIsual.ptr()).lnum) {
            let mut white_in_front: bool = linewhite(start_lnum);
            while start_lnum > 1 as linenr_T {
                if white_in_front {
                    if !linewhite(start_lnum - 1 as linenr_T) {
                        break;
                    }
                } else if linewhite(start_lnum - 1 as linenr_T) as ::core::ffi::c_int != 0
                    || startPS(start_lnum, 0 as ::core::ffi::c_int, false) as ::core::ffi::c_int
                        != 0
                {
                    break;
                }
                start_lnum -= 1;
            }
            let mut end_lnum: linenr_T = start_lnum;
            while end_lnum <= (*curbuf.get()).b_ml.ml_line_count
                && linewhite(end_lnum) as ::core::ffi::c_int != 0
            {
                end_lnum += 1;
            }
            end_lnum -= 1;
            let mut i_0: ::core::ffi::c_int = count;
            if !include && white_in_front as ::core::ffi::c_int != 0 {
                i_0 -= 1;
            }
            loop {
                let c2rust_fresh6 = i_0;
                i_0 = i_0 - 1;
                if c2rust_fresh6 == 0 {
                    break;
                }
                if end_lnum == (*curbuf.get()).b_ml.ml_line_count {
                    return FAIL;
                }
                if !include {
                    do_white = linewhite(end_lnum + 1 as linenr_T) as ::core::ffi::c_int;
                }
                if include as ::core::ffi::c_int != 0 || do_white == 0 {
                    end_lnum += 1;
                    while end_lnum < (*curbuf.get()).b_ml.ml_line_count
                        && !linewhite(end_lnum + 1 as linenr_T)
                        && !startPS(end_lnum + 1 as linenr_T, 0 as ::core::ffi::c_int, false)
                    {
                        end_lnum += 1;
                    }
                }
                if i_0 == 0 as ::core::ffi::c_int
                    && white_in_front as ::core::ffi::c_int != 0
                    && include as ::core::ffi::c_int != 0
                {
                    break;
                }
                if include as ::core::ffi::c_int != 0 || do_white != 0 {
                    while end_lnum < (*curbuf.get()).b_ml.ml_line_count
                        && linewhite(end_lnum + 1 as linenr_T) as ::core::ffi::c_int != 0
                    {
                        end_lnum += 1;
                    }
                }
            }
            if !white_in_front && !linewhite(end_lnum) && include as ::core::ffi::c_int != 0 {
                while start_lnum > 1 as linenr_T
                    && linewhite(start_lnum - 1 as linenr_T) as ::core::ffi::c_int != 0
                {
                    start_lnum -= 1;
                }
            }
            if VIsual_active.get() {
                if VIsual_mode.get() == 'V' as ::core::ffi::c_int
                    && start_lnum == (*curwin.get()).w_cursor.lnum
                {
                    break '_extend;
                } else {
                    if (*VIsual.ptr()).lnum != start_lnum {
                        (*VIsual.ptr()).lnum = start_lnum;
                        (*VIsual.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    VIsual_mode.set('V' as ::core::ffi::c_int);
                    redraw_curbuf_later(UPD_INVERTED);
                    showmode();
                }
            } else {
                (*oap).start.lnum = start_lnum;
                (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                (*oap).motion_type = kMTLineWise;
            }
            (*curwin.get()).w_cursor.lnum = end_lnum;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            return OK;
        }
    }
    dir = if start_lnum < (*VIsual.ptr()).lnum {
        BACKWARD as ::core::ffi::c_int
    } else {
        FORWARD as ::core::ffi::c_int
    };
    let mut i: ::core::ffi::c_int = count;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if start_lnum
            == (if dir == BACKWARD as ::core::ffi::c_int {
                1 as linenr_T
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            })
        {
            retval = FAIL;
            break;
        } else {
            let mut prev_start_is_white: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut t: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while t < 2 as ::core::ffi::c_int {
                start_lnum = (start_lnum as ::core::ffi::c_int + dir) as linenr_T;
                let mut start_is_white: ::core::ffi::c_int =
                    linewhite(start_lnum) as ::core::ffi::c_int;
                if prev_start_is_white == start_is_white {
                    start_lnum = (start_lnum as ::core::ffi::c_int - dir) as linenr_T;
                    break;
                } else {
                    while start_lnum
                        != (if dir == BACKWARD as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            (*curbuf.get()).b_ml.ml_line_count
                        })
                    {
                        if start_is_white
                            != linewhite(start_lnum + dir as linenr_T) as ::core::ffi::c_int
                            || start_is_white == 0
                                && startPS(
                                    start_lnum
                                        + (if dir > 0 as ::core::ffi::c_int {
                                            1 as linenr_T
                                        } else {
                                            0 as linenr_T
                                        }),
                                    0 as ::core::ffi::c_int,
                                    false,
                                ) as ::core::ffi::c_int
                                    != 0
                        {
                            break;
                        }
                        start_lnum = (start_lnum as ::core::ffi::c_int + dir) as linenr_T;
                    }
                    if !include {
                        break;
                    }
                    if start_lnum
                        == (if dir == BACKWARD as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            (*curbuf.get()).b_ml.ml_line_count
                        })
                    {
                        break;
                    }
                    prev_start_is_white = start_is_white;
                    t += 1;
                }
            }
        }
    }
    (*curwin.get()).w_cursor.lnum = start_lnum;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    return retval;
}
