//! Choosing a topline for a cursor that has moved -- the `scroll_cursor_*`
//! family and `cursor_correct()`.
//!
//! [`scroll_cursor_top`], [`scroll_cursor_bot`] and
//! [`scroll_cursor_halfway`] are the three answers `update_topline` picks
//! between when the cursor has left the visible range: put its line at the top
//! (honouring `'scrolloff'`), at the bottom (`scroll_cursor_bot` also decides
//! whether scrolling or redrawing is cheaper), or in the middle.
//! [`cursor_correct`] is the reverse -- the window stays put and the cursor
//! moves to satisfy `'scrolloff'`.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::win_lines_concealed;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::main::{curbuf, curwin, mouse_dragging};
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::os::libc::labs;
use crate::src::nvim::plines::{linetabsize_eol, plines_win_full, plines_win_nofill, win_get_fill};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::types::{Direction, colnr_T, int64_t, linenr_T, win_T};

pub unsafe extern "C" fn scroll_cursor_top(
    mut wp: *mut win_T,
    mut min_scroll: ::core::ffi::c_int,
    mut always: ::core::ffi::c_int,
) {
    unsafe {
        let mut old_topline: linenr_T = (*wp).w_topline;
        let mut old_skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int;
        let mut old_topfill: linenr_T = (*wp).w_topfill as linenr_T;
        let mut off: int64_t = get_scrolloff_value(wp);
        if mouse_dragging.get() > 0 as ::core::ffi::c_int {
            off = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t;
        }
        validate_cheight(wp);
        let mut scrolled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut used: ::core::ffi::c_int = (*wp).w_cline_height;
        if (*wp).w_cursor.lnum < (*wp).w_topline {
            scrolled = used;
        }
        let mut top: linenr_T = 0;
        let mut bot: linenr_T = 0;
        if hasFolding(wp, (*wp).w_cursor.lnum, &raw mut top, &raw mut bot) {
            top -= 1;
            bot += 1;
        } else {
            top = (*wp).w_cursor.lnum - 1 as linenr_T;
            bot = (*wp).w_cursor.lnum + 1 as linenr_T;
        }
        let mut new_topline: linenr_T = top + 1 as linenr_T;
        let mut extra: ::core::ffi::c_int = win_get_fill(wp, (*wp).w_cursor.lnum);
        while top > 0 as linenr_T {
            let mut i: ::core::ffi::c_int = plines_win_nofill(wp, top, true_0 != 0);
            hasFolding(wp, top, &raw mut top, ::core::ptr::null_mut::<linenr_T>());
            if top < (*wp).w_topline {
                scrolled += i;
            }
            if (new_topline >= (*wp).w_topline || scrolled > min_scroll) && extra as int64_t >= off
            {
                break;
            }
            used += i;
            if (extra + i) as int64_t <= off && bot < (*(*wp).w_buffer).b_ml.ml_line_count {
                used += plines_win_full(
                    wp,
                    bot,
                    &raw mut bot,
                    ::core::ptr::null_mut::<bool>(),
                    true_0 != 0,
                    true_0 != 0,
                );
            }
            if used > (*wp).w_view_height {
                break;
            }
            extra += i;
            new_topline = top;
            top -= 1;
            bot += 1;
        }
        if used > (*wp).w_view_height {
            scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
        } else {
            if new_topline < (*wp).w_topline || always != 0 {
                (*wp).w_topline = new_topline;
            }
            (*wp).w_topline = if (*wp).w_topline < (*wp).w_cursor.lnum {
                (*wp).w_topline
            } else {
                (*wp).w_cursor.lnum
            };
            (*wp).w_topfill = win_get_fill(wp, (*wp).w_topline);
            if (*wp).w_topfill > 0 as ::core::ffi::c_int && extra as int64_t > off {
                (*wp).w_topfill -= extra - off as ::core::ffi::c_int;
                (*wp).w_topfill = if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                    (*wp).w_topfill
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            check_topfill(wp, false_0 != 0);
            if (*wp).w_topline != old_topline {
                reset_skipcol(wp);
            } else if (*wp).w_topline == (*wp).w_cursor.lnum {
                validate_virtcol(wp);
                if (*wp).w_skipcol >= (*wp).w_virtcol {
                    reset_skipcol(wp);
                }
            }
            if (*wp).w_topline != old_topline
                || (*wp).w_skipcol != old_skipcol
                || (*wp).w_topfill as linenr_T != old_topfill
            {
                (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
            }
            (*wp).w_valid |= VALID_TOPLINE;
            (*wp).w_viewport_invalid = true_0 != 0;
        };
    }
}

pub unsafe extern "C" fn set_empty_rows(mut wp: *mut win_T, mut used: ::core::ffi::c_int) {
    unsafe {
        (*wp).w_filler_rows = 0 as ::core::ffi::c_int;
        if used == 0 as ::core::ffi::c_int {
            (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
        } else {
            (*wp).w_empty_rows = (*wp).w_view_height - used;
            if (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
                (*wp).w_filler_rows = win_get_fill(wp, (*wp).w_botline);
                if (*wp).w_empty_rows > (*wp).w_filler_rows {
                    (*wp).w_empty_rows -= (*wp).w_filler_rows;
                } else {
                    (*wp).w_filler_rows = (*wp).w_empty_rows;
                    (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
                }
            }
        };
    }
}

pub unsafe extern "C" fn scroll_cursor_bot(
    mut wp: *mut win_T,
    mut min_scroll: ::core::ffi::c_int,
    mut set_topbot: bool,
) {
    unsafe {
        let mut loff: lineoff_T = lineoff_T {
            lnum: 0,
            fill: 0,
            height: 0,
        };
        let mut old_topline: linenr_T = (*wp).w_topline;
        let mut old_skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int;
        let mut old_topfill: ::core::ffi::c_int = (*wp).w_topfill;
        let mut old_botline: linenr_T = (*wp).w_botline;
        let mut old_valid: ::core::ffi::c_int = (*wp).w_valid;
        let mut old_empty_rows: ::core::ffi::c_int = (*wp).w_empty_rows;
        let mut cln: linenr_T = (*wp).w_cursor.lnum;
        let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
        if set_topbot {
            let mut used: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut cln_last: linenr_T = cln;
            hasFolding(
                wp,
                cln,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut cln_last,
            );
            (*wp).w_botline = cln_last + 1 as linenr_T;
            loff.lnum = cln_last + 1 as linenr_T;
            loff.fill = 0 as ::core::ffi::c_int;
            loop {
                topline_back_winheight(wp, &raw mut loff, false_0);
                if loff.height == MAXCOL as ::core::ffi::c_int {
                    break;
                }
                if used + loff.height > (*wp).w_view_height {
                    if do_sms {
                        if used < (*wp).w_view_height {
                            let mut plines_offset: ::core::ffi::c_int =
                                used + loff.height - (*wp).w_view_height;
                            used = (*wp).w_view_height;
                            (*wp).w_topfill = loff.fill;
                            (*wp).w_topline = loff.lnum;
                            (*wp).w_skipcol = skipcol_from_plines(wp, plines_offset) as colnr_T;
                        }
                    }
                    break;
                } else {
                    (*wp).w_topfill = loff.fill;
                    (*wp).w_topline = loff.lnum;
                    used += loff.height;
                }
            }
            set_empty_rows(wp, used);
            (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
            if (*wp).w_topline != old_topline
                || (*wp).w_topfill != old_topfill
                || (*wp).w_skipcol != old_skipcol
                || (*wp).w_skipcol != 0 as ::core::ffi::c_int
            {
                (*wp).w_valid &= !(VALID_WROW | VALID_CROW);
                if (*wp).w_skipcol != old_skipcol {
                    redraw_later(wp, UPD_NOT_VALID);
                } else {
                    reset_skipcol(wp);
                }
            }
        } else {
            validate_botline_win(wp);
        }
        let mut used_0: ::core::ffi::c_int = plines_win_nofill(wp, cln, true_0 != 0);
        let mut scrolled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if cln >= (*wp).w_botline {
            scrolled = used_0;
            if cln == (*wp).w_botline {
                scrolled -= (*wp).w_empty_rows;
            }
            if do_sms {
                let mut top_plines: ::core::ffi::c_int =
                    plines_win_nofill(wp, (*wp).w_topline, false_0 != 0);
                let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
                if width1 > 0 as ::core::ffi::c_int {
                    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
                    let mut skip_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if (*wp).w_skipcol > width1 {
                        skip_lines += ((*wp).w_skipcol as ::core::ffi::c_int - width1) / width2
                            + 1 as ::core::ffi::c_int;
                    } else if (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                        skip_lines = 1 as ::core::ffi::c_int;
                    }
                    top_plines -= skip_lines;
                    if top_plines > (*wp).w_view_height {
                        scrolled += top_plines - (*wp).w_view_height;
                    }
                }
            }
        }
        let mut boff: lineoff_T = lineoff_T {
            lnum: 0,
            fill: 0,
            height: 0,
        };
        if !hasFolding(
            wp,
            (*wp).w_cursor.lnum,
            &raw mut loff.lnum,
            &raw mut boff.lnum,
        ) {
            loff.lnum = cln;
            boff.lnum = cln;
        }
        loff.fill = 0 as ::core::ffi::c_int;
        boff.fill = 0 as ::core::ffi::c_int;
        let mut fill_below_window: ::core::ffi::c_int =
            win_get_fill(wp, (*wp).w_botline) - (*wp).w_filler_rows;
        let mut extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut so: int64_t = get_scrolloff_value(wp);
        while loff.lnum > 1 as linenr_T {
            if ((scrolled <= 0 as ::core::ffi::c_int || scrolled >= min_scroll)
                && extra as int64_t
                    >= (if mouse_dragging.get() > 0 as ::core::ffi::c_int {
                        (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t
                    } else {
                        so
                    })
                || boff.lnum + 1 as linenr_T > (*(*wp).w_buffer).b_ml.ml_line_count)
                && loff.lnum <= (*wp).w_botline
                && (loff.lnum < (*wp).w_botline || loff.fill >= fill_below_window)
            {
                break;
            }
            topline_back(wp, &raw mut loff);
            if loff.height == MAXCOL as ::core::ffi::c_int {
                used_0 = MAXCOL as ::core::ffi::c_int;
            } else {
                used_0 += loff.height;
            }
            if used_0 > (*wp).w_view_height {
                break;
            }
            if loff.lnum >= (*wp).w_botline
                && (loff.lnum > (*wp).w_botline || loff.fill <= fill_below_window)
            {
                scrolled += loff.height;
                if loff.lnum == (*wp).w_botline && loff.fill == 0 as ::core::ffi::c_int {
                    scrolled -= (*wp).w_empty_rows;
                }
            }
            if boff.lnum >= (*(*wp).w_buffer).b_ml.ml_line_count {
                continue;
            }
            botline_forw(wp, &raw mut boff);
            debug_assert!(
                boff.height != MAXCOL as ::core::ffi::c_int,
                "boff.height != MAXCOL"
            );
            used_0 += boff.height;
            if used_0 > (*wp).w_view_height {
                break;
            }
            if (extra as int64_t)
                < (if mouse_dragging.get() > 0 as ::core::ffi::c_int {
                    (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t
                } else {
                    so
                })
                || scrolled < min_scroll
            {
                extra += boff.height;
                if boff.lnum >= (*wp).w_botline
                    || boff.lnum + 1 as linenr_T == (*wp).w_botline
                        && boff.fill > (*wp).w_filler_rows
                {
                    scrolled += boff.height;
                    if boff.lnum == (*wp).w_botline && boff.fill == 0 as ::core::ffi::c_int {
                        scrolled -= (*wp).w_empty_rows;
                    }
                }
            }
        }
        let mut line_count: linenr_T = 0;
        if scrolled <= 0 as ::core::ffi::c_int {
            line_count = 0 as ::core::ffi::c_int as linenr_T;
        } else if used_0 > (*wp).w_view_height {
            line_count = used_0 as linenr_T;
        } else {
            line_count = 0 as ::core::ffi::c_int as linenr_T;
            boff.fill = (*wp).w_topfill;
            boff.lnum = (*wp).w_topline - 1 as linenr_T;
            let mut i: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            while i < scrolled && boff.lnum < (*wp).w_botline {
                botline_forw(wp, &raw mut boff);
                i += boff.height;
                line_count += 1;
            }
            if i < scrolled {
                line_count = 9999 as ::core::ffi::c_int as linenr_T;
            }
        }
        if line_count >= (*wp).w_view_height as linenr_T && line_count > min_scroll as linenr_T {
            scroll_cursor_halfway(wp, false_0 != 0, true_0 != 0);
        } else if line_count > 0 as linenr_T {
            if do_sms {
                scrollup(wp, scrolled as linenr_T, true_0 != 0);
            } else {
                scrollup(wp, line_count, true_0 != 0);
            }
        }
        if (*wp).w_topline == old_topline
            && (*wp).w_skipcol == old_skipcol
            && set_topbot as ::core::ffi::c_int != 0
        {
            (*wp).w_botline = old_botline;
            (*wp).w_empty_rows = old_empty_rows;
            (*wp).w_valid = old_valid;
        }
        (*wp).w_valid |= VALID_TOPLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
        if set_topbot {
            cursor_correct_sms(wp);
        }
    }
}

pub unsafe extern "C" fn scroll_cursor_halfway(
    mut wp: *mut win_T,
    mut atend: bool,
    mut prefer_above: bool,
) {
    unsafe {
        let mut old_topline: linenr_T = (*wp).w_topline;
        let mut loff: lineoff_T = lineoff_T {
            lnum: (*wp).w_cursor.lnum,
            fill: 0,
            height: 0,
        };
        let mut boff: lineoff_T = lineoff_T {
            lnum: (*wp).w_cursor.lnum,
            fill: 0,
            height: 0,
        };
        hasFolding(wp, loff.lnum, &raw mut loff.lnum, &raw mut boff.lnum);
        let mut used: ::core::ffi::c_int = plines_win_nofill(wp, loff.lnum, true_0 != 0);
        loff.fill = 0 as ::core::ffi::c_int;
        boff.fill = 0 as ::core::ffi::c_int;
        let mut topline: linenr_T = loff.lnum;
        let mut skipcol: colnr_T = 0 as colnr_T;
        let mut want_height: ::core::ffi::c_int = 0;
        let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
        if do_sms {
            if atend {
                want_height = ((*wp).w_view_height - used) / 2 as ::core::ffi::c_int;
                used = 0 as ::core::ffi::c_int;
            } else {
                want_height = (*wp).w_view_height;
            }
        }
        let mut topfill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while topline > 1 as linenr_T {
            if do_sms {
                topline_back_winheight(wp, &raw mut loff, false_0);
                if loff.height == MAXCOL as ::core::ffi::c_int {
                    break;
                }
                used += loff.height;
                if !atend && boff.lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                    botline_forw(wp, &raw mut boff);
                    used += boff.height;
                }
                if used > want_height {
                    if used - loff.height < want_height {
                        topline = loff.lnum;
                        topfill = loff.fill;
                        skipcol = skipcol_from_plines(wp, used - want_height) as colnr_T;
                    }
                    break;
                } else {
                    topline = loff.lnum;
                    topfill = loff.fill;
                }
            } else {
                let mut done: bool = false_0 != 0;
                let mut above: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut below: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while round <= 2 as ::core::ffi::c_int {
                    if if prefer_above as ::core::ffi::c_int != 0 {
                        (round == 2 as ::core::ffi::c_int && below < above) as ::core::ffi::c_int
                    } else {
                        (round == 1 as ::core::ffi::c_int && below <= above) as ::core::ffi::c_int
                    } != 0
                    {
                        if boff.lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                            botline_forw(wp, &raw mut boff);
                            used += boff.height;
                            if used > (*wp).w_view_height {
                                done = true_0 != 0;
                                break;
                            } else {
                                below += boff.height;
                            }
                        } else {
                            below += 1;
                            if atend {
                                used += 1;
                            }
                        }
                    }
                    if if prefer_above as ::core::ffi::c_int != 0 {
                        (round == 1 as ::core::ffi::c_int && below >= above) as ::core::ffi::c_int
                    } else {
                        (round == 1 as ::core::ffi::c_int && below > above) as ::core::ffi::c_int
                    } != 0
                    {
                        topline_back(wp, &raw mut loff);
                        if loff.height == MAXCOL as ::core::ffi::c_int {
                            used = MAXCOL as ::core::ffi::c_int;
                        } else {
                            used += loff.height;
                        }
                        if used > (*wp).w_view_height {
                            done = true_0 != 0;
                            break;
                        } else {
                            above += loff.height;
                            topline = loff.lnum;
                            topfill = loff.fill;
                        }
                    }
                    round += 1;
                }
                if done {
                    break;
                }
            }
        }
        if !hasFolding(
            wp,
            topline,
            &raw mut (*wp).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        ) && ((*wp).w_topline != topline
            || skipcol != 0 as ::core::ffi::c_int
            || (*wp).w_skipcol != 0 as ::core::ffi::c_int)
        {
            (*wp).w_topline = topline;
            if skipcol != 0 as ::core::ffi::c_int {
                (*wp).w_skipcol = skipcol;
                redraw_later(wp, UPD_NOT_VALID);
            } else if do_sms {
                reset_skipcol(wp);
            }
        }
        (*wp).w_topfill = topfill;
        if old_topline > (*wp).w_topline + (*wp).w_view_height as linenr_T {
            (*wp).w_botfill = false_0 != 0;
        }
        check_topfill(wp, false_0 != 0);
        (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
        (*wp).w_valid |= VALID_TOPLINE;
    }
}

pub unsafe extern "C" fn cursor_correct(mut wp: *mut win_T) {
    unsafe {
        let mut above_wanted: int64_t = get_scrolloff_value(wp);
        let mut below_wanted: int64_t = get_scrolloff_value(wp);
        if mouse_dragging.get() > 0 as ::core::ffi::c_int {
            above_wanted = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t;
            below_wanted = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t;
        }
        if (*wp).w_topline == 1 as linenr_T {
            above_wanted = 0 as int64_t;
            let mut max_off: ::core::ffi::c_int = (*wp).w_view_height / 2 as ::core::ffi::c_int;
            below_wanted = if below_wanted < max_off as int64_t {
                below_wanted
            } else {
                max_off as int64_t
            };
        }
        validate_botline_win(wp);
        if (*wp).w_botline == (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T
            && mouse_dragging.get() == 0 as ::core::ffi::c_int
        {
            below_wanted = 0 as int64_t;
            let mut max_off_0: ::core::ffi::c_int =
                ((*wp).w_view_height - 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
            above_wanted = if above_wanted < max_off_0 as int64_t {
                above_wanted
            } else {
                max_off_0 as int64_t
            };
        }
        let mut cln: linenr_T = (*wp).w_cursor.lnum;
        if cln as int64_t >= (*wp).w_topline as int64_t + above_wanted
            && (cln as int64_t) < (*wp).w_botline as int64_t - below_wanted
            && !win_lines_concealed(wp)
        {
            return;
        }
        if (*wp).w_onebuf_opt.wo_sms != 0 && (*wp).w_onebuf_opt.wo_wrap == 0 {
            if (*wp).w_cline_height == (*wp).w_view_height {
                reset_skipcol(wp);
                return;
            }
        }
        let mut topline: linenr_T = (*wp).w_topline;
        let mut botline: linenr_T = (*wp).w_botline - 1 as linenr_T;
        let mut above: ::core::ffi::c_int = (*wp).w_topfill;
        let mut below: ::core::ffi::c_int = (*wp).w_filler_rows;
        while ((above as int64_t) < above_wanted || (below as int64_t) < below_wanted)
            && topline < botline
        {
            if (below as int64_t) < below_wanted
                && (below <= above || above as int64_t >= above_wanted)
            {
                below += plines_win_full(
                    wp,
                    botline,
                    ::core::ptr::null_mut::<linenr_T>(),
                    ::core::ptr::null_mut::<bool>(),
                    true_0 != 0,
                    true_0 != 0,
                );
                hasFolding(
                    wp,
                    botline,
                    &raw mut botline,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                botline -= 1;
            }
            if (above as int64_t) < above_wanted
                && (above < below || below as int64_t >= below_wanted)
            {
                above += plines_win_nofill(wp, topline, true_0 != 0);
                hasFolding(
                    wp,
                    topline,
                    ::core::ptr::null_mut::<linenr_T>(),
                    &raw mut topline,
                );
                if topline < botline {
                    above += win_get_fill(wp, topline + 1 as linenr_T);
                }
                topline += 1;
            }
        }
        if topline == botline || botline == 0 as linenr_T {
            (*wp).w_cursor.lnum = topline;
        } else if topline > botline {
            (*wp).w_cursor.lnum = botline;
        } else {
            if cln < topline && (*wp).w_topline > 1 as linenr_T {
                (*wp).w_cursor.lnum = topline;
                (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
            }
            if cln > botline && (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
                (*wp).w_cursor.lnum = botline;
                (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
            }
        }
        check_cursor_moved(wp);
        (*wp).w_valid |= VALID_TOPLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn get_scroll_overlap(mut dir: Direction) -> ::core::ffi::c_int {
    unsafe {
        let mut loff: lineoff_T = lineoff_T {
            lnum: 0,
            fill: 0,
            height: 0,
        };
        let mut min_height: ::core::ffi::c_int =
            (*curwin.get()).w_view_height - 2 as ::core::ffi::c_int;
        validate_botline_win(curwin.get());
        if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
            && (*curwin.get()).w_topline == 1 as linenr_T
            || dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                && (*curwin.get()).w_botline > (*curbuf.get()).b_ml.ml_line_count
        {
            return min_height + 2 as ::core::ffi::c_int;
        }
        loff.lnum = if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            (*curwin.get()).w_botline
        } else {
            (*curwin.get()).w_topline - 1 as linenr_T
        };
        loff.fill = win_get_fill(
            curwin.get(),
            loff.lnum
                + (dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int)
                    as ::core::ffi::c_int,
        ) - (if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            (*curwin.get()).w_filler_rows
        } else {
            (*curwin.get()).w_topfill
        });
        loff.height = if loff.fill > 0 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            plines_win_nofill(curwin.get(), loff.lnum, true_0 != 0)
        };
        let mut h1: ::core::ffi::c_int = loff.height;
        if h1 > min_height {
            return min_height + 2 as ::core::ffi::c_int;
        }
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            topline_back(curwin.get(), &raw mut loff);
        } else {
            botline_forw(curwin.get(), &raw mut loff);
        }
        let mut h2: ::core::ffi::c_int = loff.height;
        if h2 == MAXCOL as ::core::ffi::c_int || h2 + h1 > min_height {
            return min_height + 2 as ::core::ffi::c_int;
        }
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            topline_back(curwin.get(), &raw mut loff);
        } else {
            botline_forw(curwin.get(), &raw mut loff);
        }
        let mut h3: ::core::ffi::c_int = loff.height;
        if h3 == MAXCOL as ::core::ffi::c_int || h3 + h2 > min_height {
            return min_height + 2 as ::core::ffi::c_int;
        }
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            topline_back(curwin.get(), &raw mut loff);
        } else {
            botline_forw(curwin.get(), &raw mut loff);
        }
        let mut h4: ::core::ffi::c_int = loff.height;
        if h4 == MAXCOL as ::core::ffi::c_int
            || h4 + h3 + h2 > min_height
            || h3 + h2 + h1 > min_height
        {
            return min_height + 1 as ::core::ffi::c_int;
        } else {
            return min_height;
        };
    }
}

pub(crate) unsafe extern "C" fn scroll_with_sms(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
    mut curscount: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut prev_sms: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_sms;
        let mut prev_skipcol: colnr_T = (*curwin.get()).w_skipcol;
        let mut prev_topline: linenr_T = (*curwin.get()).w_topline;
        let mut prev_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
        (*curwin.get()).w_onebuf_opt.wo_sms = true_0;
        scroll_redraw(
            (dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
            count as linenr_T,
        );
        if prev_sms == 0 && (*curwin.get()).w_skipcol > 0 as ::core::ffi::c_int {
            let mut fixdir: ::core::ffi::c_int = dir as ::core::ffi::c_int;
            if labs(((*curwin.get()).w_topline - prev_topline) as ::core::ffi::c_long)
                > (dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int)
                    as ::core::ffi::c_int as ::core::ffi::c_long
            {
                fixdir = dir as ::core::ffi::c_int * -1 as ::core::ffi::c_int;
            }
            let mut width1: ::core::ffi::c_int =
                (*curwin.get()).w_view_width - win_col_off(curwin.get());
            let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin.get());
            count = 1 as ::core::ffi::c_int
                + ((*curwin.get()).w_skipcol as ::core::ffi::c_int
                    - width1
                    - 1 as ::core::ffi::c_int)
                    / width2;
            if fixdir == FORWARD as ::core::ffi::c_int {
                count = 1 as ::core::ffi::c_int
                    + (linetabsize_eol(curwin.get(), (*curwin.get()).w_topline)
                        - (*curwin.get()).w_skipcol as ::core::ffi::c_int
                        - width1
                        + width2
                        - 1 as ::core::ffi::c_int)
                        / width2;
            }
            scroll_redraw(
                (fixdir == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
                count as linenr_T,
            );
            *curscount += count
                * (if fixdir == dir as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    -1 as ::core::ffi::c_int
                });
        }
        (*curwin.get()).w_onebuf_opt.wo_sms = prev_sms;
        return (*curwin.get()).w_topline != prev_topline
            || (*curwin.get()).w_topfill != prev_topfill
            || (*curwin.get()).w_skipcol != prev_skipcol;
    }
}
