//! Where the cursor is on the screen, and which part of the buffer the window
//! shows.
//!
//! Carved by the question each part answers:
//!
//! | child | what |
//! | --- | --- |
//! | [`topline`] | `update_topline()` -- has the cursor left the visible range? |
//! | [`columns`] | `curs_columns()`, `screenpos()`, `virtcol2col()` -- the horizontal half |
//! | [`scroll`] | `scrolldown()`/`scrollup()` and the clamped forms |
//! | [`scrollcur`] | the `scroll_cursor_*` family and `cursor_correct()` |
//! | [`page`] | `pagescroll()` and `'cursorbind'` |
//!
//! What stays here is the `w_valid` flag alphabet the five share, the
//! `lineoff_T` cursor those flags guard, the small predicates that read or
//! invalidate them (`validate_cursor`, `validate_virtcol`,
//! `validate_cursor_col`, `changed_cline_bef_curs` and friends), `curs_rows`,
//! and the two `win_col_off` helpers that say how much of a window is not
//! text.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::cursor::check_cursor_lnum;
use crate::src::nvim::decoration::{SIGN_WIDTH, decor_conceal_line};
use crate::src::nvim::drawscreen::{
    UPD_INVERTED, UPD_SOME_VALID, UPD_VALID, conceal_cursor_line, number_width, redraw_buf_later,
    redraw_later, redrawWinline, redrawing, win_cursorline_standout,
};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::main::{VIsual_active, cmdwin_win, curbuf, curwin, p_cpo};
use crate::src::nvim::option::get_showbreak_value;
use crate::src::nvim::options::kOptCuloptFlagScreenline;
use crate::src::nvim::plines::{getvvcol, plines_win_full, win_may_fill};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{MotionType, OptInt, colnr_T, linenr_T, win_T};
use crate::src::nvim::window::win_fdccol_count;
use crate::src::nvim::winfloat::win_check_anchored_floats;

// The carve of the transpiled module; see each child's docs.
mod columns;
mod page;
mod scroll;
mod scrollcur;
mod topline;

pub use self::columns::*;
pub use self::page::*;
pub use self::scroll::*;
pub use self::scrollcur::*;
pub use self::topline::*;

pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_15 = 4;
pub const BL_SOL: C2Rust_Unnamed_15 = 2;
pub const kMTCharWise: MotionType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lineoff_T {
    pub lnum: linenr_T,
    pub fill: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VALID_CHEIGHT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_BOTLINE_AP: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn adjust_plines_for_skipcol(mut wp: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut w2: ::core::ffi::c_int = width + win_col_off2(wp);
        if (*wp).w_skipcol >= width && w2 > 0 as ::core::ffi::c_int {
            return ((*wp).w_skipcol as ::core::ffi::c_int - width) / w2 + 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}
pub unsafe extern "C" fn plines_correct_topline(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut nextp: *mut linenr_T,
    mut limit_winheight: bool,
    mut foldedp: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut n: ::core::ffi::c_int =
            plines_win_full(wp, lnum, nextp, foldedp, true_0 != 0, false_0 != 0);
        if lnum == (*wp).w_topline {
            n -= adjust_plines_for_skipcol(wp);
        }
        if limit_winheight as ::core::ffi::c_int != 0 && n > (*wp).w_view_height {
            return (*wp).w_view_height;
        }
        return n;
    }
}
unsafe extern "C" fn comp_botline(mut wp: *mut win_T) {
    unsafe {
        let mut lnum: linenr_T = 0;
        let mut done: ::core::ffi::c_int = 0;
        check_cursor_moved(wp);
        if (*wp).w_valid & VALID_CROW != 0 {
            lnum = (*wp).w_cursor.lnum;
            done = (*wp).w_cline_row;
        } else {
            lnum = (*wp).w_topline;
            done = 0 as ::core::ffi::c_int;
        }
        while lnum <= (*(*wp).w_buffer).b_ml.ml_line_count {
            let mut last: linenr_T = lnum;
            let mut folded: bool = false;
            let mut n: ::core::ffi::c_int =
                plines_correct_topline(wp, lnum, &raw mut last, true_0 != 0, &raw mut folded);
            if lnum <= (*wp).w_cursor.lnum && last >= (*wp).w_cursor.lnum {
                (*wp).w_cline_row = done;
                (*wp).w_cline_height = n;
                (*wp).w_cline_folded = folded;
                redraw_for_cursorline(wp);
                (*wp).w_valid |= VALID_CROW | VALID_CHEIGHT;
            }
            if done + n > (*wp).w_view_height {
                break;
            }
            done += n;
            lnum = last;
            lnum += 1;
        }
        (*wp).w_botline = lnum;
        (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
        (*wp).w_viewport_invalid = true_0 != 0;
        set_empty_rows(wp, done);
        win_check_anchored_floats(wp);
    }
}
unsafe extern "C" fn redraw_for_cursorline(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_valid & VALID_CROW != 0 {
            return;
        }
        if (*wp).w_onebuf_opt.wo_rnu != 0 || win_cursorline_standout(wp) as ::core::ffi::c_int != 0
        {
            redraw_later(wp, UPD_VALID);
        }
    }
}
unsafe extern "C" fn redraw_for_cursorcolumn(mut wp: *mut win_T) {
    unsafe {
        if wp == curwin.get()
            && (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
            && conceal_cursor_line(wp) as ::core::ffi::c_int != 0
        {
            redrawWinline(wp, (*wp).w_cursor.lnum);
        }
        if (*wp).w_valid & VALID_VIRTCOL != 0 {
            return;
        }
        if (*wp).w_onebuf_opt.wo_cuc != 0 {
            redraw_later(wp, UPD_SOME_VALID);
        } else if (*wp).w_onebuf_opt.wo_cul != 0
            && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                & kOptCuloptFlagScreenline as ::core::ffi::c_int
                != 0
        {
            redraw_later(wp, UPD_VALID);
        }
        if VIsual_active.get() as ::core::ffi::c_int != 0 && (*wp).w_buffer == curbuf.get() {
            redraw_buf_later(curbuf.get(), UPD_INVERTED);
        }
    }
}
pub unsafe extern "C" fn set_valid_virtcol(mut wp: *mut win_T, mut vcol: colnr_T) {
    unsafe {
        (*wp).w_virtcol = vcol;
        redraw_for_cursorcolumn(wp);
        (*wp).w_valid |= VALID_VIRTCOL;
    }
}
pub unsafe extern "C" fn sms_marker_overlap(
    mut wp: *mut win_T,
    mut extra2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if extra2 == -1 as ::core::ffi::c_int {
            extra2 = win_col_off(wp) - win_col_off2(wp);
        }
        if *get_showbreak_value(wp) as ::core::ffi::c_int != NUL {
            return 0 as ::core::ffi::c_int;
        }
        if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.prec != 0 {
            return 1 as ::core::ffi::c_int;
        }
        return if extra2 > 3 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            3 as ::core::ffi::c_int - extra2
        };
    }
}
unsafe extern "C" fn skipcol_from_plines(
    mut wp: *mut win_T,
    mut plines_off: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut skipcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if plines_off > 0 as ::core::ffi::c_int {
            skipcol += width1;
        }
        if plines_off > 1 as ::core::ffi::c_int {
            skipcol += (width1 + win_col_off2(wp)) * (plines_off - 1 as ::core::ffi::c_int);
        }
        return skipcol;
    }
}
unsafe extern "C" fn reset_skipcol(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
            return;
        }
        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
        redraw_later(wp, UPD_SOME_VALID);
    }
}
pub unsafe extern "C" fn changed_cline_bef_curs(mut wp: *mut win_T) {
    unsafe {
        (*wp).w_valid &=
            !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
    }
}
pub unsafe extern "C" fn changed_line_abv_curs() {
    unsafe {
        (*curwin.get()).w_valid &=
            !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
    }
}
pub unsafe extern "C" fn changed_line_abv_curs_win(mut wp: *mut win_T) {
    unsafe {
        (*wp).w_valid &=
            !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
    }
}
pub unsafe extern "C" fn validate_botline_win(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_valid & VALID_BOTLINE == 0 {
            comp_botline(wp);
        }
    }
}
pub unsafe extern "C" fn invalidate_botline_win(mut wp: *mut win_T) {
    unsafe {
        (*wp).w_valid &= !(VALID_BOTLINE | VALID_BOTLINE_AP);
    }
}
pub unsafe extern "C" fn approximate_botline_win(mut wp: *mut win_T) {
    unsafe {
        (*wp).w_valid &= !VALID_BOTLINE;
    }
}
pub unsafe extern "C" fn cursor_valid(mut wp: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        check_cursor_moved(wp);
        return ((*wp).w_valid & (VALID_WROW | VALID_WCOL) == VALID_WROW | VALID_WCOL)
            as ::core::ffi::c_int;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn validate_cursor(mut wp: *mut win_T) {
    unsafe {
        check_cursor_lnum(wp);
        check_cursor_moved(wp);
        if (*wp).w_valid & (VALID_WCOL | VALID_WROW) != VALID_WCOL | VALID_WROW {
            curs_columns(wp, true_0);
        }
    }
}
unsafe extern "C" fn curs_rows(mut wp: *mut win_T) {
    unsafe {
        let mut all_invalid: bool = !redrawing()
            || (*wp).w_lines_valid == 0 as ::core::ffi::c_int
            || (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum > (*wp).w_topline;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        (*wp).w_cline_row = 0 as ::core::ffi::c_int;
        let mut lnum: linenr_T = (*wp).w_topline;
        's_111: while lnum < (*wp).w_cursor.lnum {
            let mut valid: bool = false_0 != 0;
            's_11: {
                if !all_invalid && i < (*wp).w_lines_valid {
                    if (*(*wp).w_lines.offset(i as isize)).wl_lnum < lnum
                        || !(*(*wp).w_lines.offset(i as isize)).wl_valid
                    {
                        break 's_11;
                    } else if (*(*wp).w_lines.offset(i as isize)).wl_lnum == lnum {
                        if !(*(*wp).w_buffer).b_mod_set
                            || (*(*wp).w_lines.offset(i as isize)).wl_lastlnum < (*wp).w_cursor.lnum
                            || (*(*wp).w_buffer).b_mod_top
                                > (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T
                        {
                            valid = true_0 != 0;
                        }
                    } else if (*(*wp).w_lines.offset(i as isize)).wl_lnum > lnum {
                        i -= 1;
                    }
                }
                if valid as ::core::ffi::c_int != 0
                    && (lnum != (*wp).w_topline
                        || (*wp).w_skipcol == 0 as ::core::ffi::c_int && !win_may_fill(wp))
                {
                    lnum = (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T;
                    if lnum > (*wp).w_cursor.lnum {
                        break 's_111;
                    }
                    (*wp).w_cline_row +=
                        (*(*wp).w_lines.offset(i as isize)).wl_size as ::core::ffi::c_int;
                } else {
                    let mut last: linenr_T = lnum;
                    let mut folded: bool = false;
                    let mut n: ::core::ffi::c_int = plines_correct_topline(
                        wp,
                        lnum,
                        &raw mut last,
                        true_0 != 0,
                        &raw mut folded,
                    );
                    lnum = last + 1 as linenr_T;
                    if lnum
                        + decor_conceal_line(
                            wp,
                            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            false_0 != 0,
                        ) as linenr_T
                        > (*wp).w_cursor.lnum
                    {
                        break 's_111;
                    }
                    (*wp).w_cline_row += n;
                }
            }
            i += 1;
        }
        check_cursor_moved(wp);
        if (*wp).w_valid & VALID_CHEIGHT == 0 {
            if all_invalid as ::core::ffi::c_int != 0
                || i == (*wp).w_lines_valid
                || i < (*wp).w_lines_valid
                    && (!(*(*wp).w_lines.offset(i as isize)).wl_valid
                        || (*(*wp).w_lines.offset(i as isize)).wl_lnum != (*wp).w_cursor.lnum)
            {
                (*wp).w_cline_height = plines_win_full(
                    wp,
                    (*wp).w_cursor.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                    &raw mut (*wp).w_cline_folded,
                    true_0 != 0,
                    true_0 != 0,
                );
            } else if i > (*wp).w_lines_valid {
                (*wp).w_cline_height = 0 as ::core::ffi::c_int;
                (*wp).w_cline_folded = hasFolding(
                    wp,
                    (*wp).w_cursor.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                    ::core::ptr::null_mut::<linenr_T>(),
                );
            } else {
                (*wp).w_cline_height =
                    (*(*wp).w_lines.offset(i as isize)).wl_size as ::core::ffi::c_int;
                (*wp).w_cline_folded = (*(*wp).w_lines.offset(i as isize)).wl_folded;
            }
        }
        redraw_for_cursorline(wp);
        (*wp).w_valid |= VALID_CROW | VALID_CHEIGHT;
    }
}
pub unsafe extern "C" fn validate_virtcol(mut wp: *mut win_T) {
    unsafe {
        check_cursor_moved(wp);
        if (*wp).w_valid & VALID_VIRTCOL != 0 {
            return;
        }
        getvvcol(
            wp,
            &raw mut (*wp).w_cursor,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut (*wp).w_virtcol,
            ::core::ptr::null_mut::<colnr_T>(),
        );
        redraw_for_cursorcolumn(wp);
        (*wp).w_valid |= VALID_VIRTCOL;
    }
}
pub unsafe extern "C" fn validate_cheight(mut wp: *mut win_T) {
    unsafe {
        check_cursor_moved(wp);
        if (*wp).w_valid & VALID_CHEIGHT != 0 {
            return;
        }
        (*wp).w_cline_height = plines_win_full(
            wp,
            (*wp).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*wp).w_cline_folded,
            true_0 != 0,
            true_0 != 0,
        );
        (*wp).w_valid |= VALID_CHEIGHT;
    }
}
pub unsafe extern "C" fn validate_cursor_col(mut wp: *mut win_T) {
    unsafe {
        validate_virtcol(wp);
        if (*wp).w_valid & VALID_WCOL != 0 {
            return;
        }
        let mut col: colnr_T = (*wp).w_virtcol;
        let mut off: colnr_T = win_col_off(wp);
        col += off;
        let mut width: ::core::ffi::c_int =
            (*wp).w_view_width - off as ::core::ffi::c_int + win_col_off2(wp);
        if (*wp).w_onebuf_opt.wo_wrap != 0
            && col >= (*wp).w_view_width
            && width > 0 as ::core::ffi::c_int
        {
            col -= ((col as ::core::ffi::c_int - (*wp).w_view_width) / width
                + 1 as ::core::ffi::c_int)
                * width;
        }
        if col > (*wp).w_leftcol {
            col -= (*wp).w_leftcol;
        } else {
            col = 0 as ::core::ffi::c_int as colnr_T;
        }
        (*wp).w_wcol = col as ::core::ffi::c_int;
        (*wp).w_valid |= VALID_WCOL;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn win_col_off(mut wp: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        return (if (*wp).w_onebuf_opt.wo_nu != 0
            || (*wp).w_onebuf_opt.wo_rnu != 0
            || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
        {
            number_width(wp)
                + (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) + (if wp != cmdwin_win.get() {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) + win_fdccol_count(wp)
            + (*wp).w_scwidth * SIGN_WIDTH as ::core::ffi::c_int;
    }
}
pub unsafe extern "C" fn win_col_off2(mut wp: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        if ((*wp).w_onebuf_opt.wo_nu != 0
            || (*wp).w_onebuf_opt.wo_rnu != 0
            || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL)
            && !vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
        {
            return number_width(wp)
                + (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
