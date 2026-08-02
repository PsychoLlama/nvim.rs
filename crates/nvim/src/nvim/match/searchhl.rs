//! 'hlsearch' and match highlighting, one window line at a time.
//!
//! The drawing code calls these in a fixed order: [`init_search_hl`] once
//! per window redraw, [`prepare_search_hl`] once per line to advance every
//! pattern to that line, [`prepare_search_hl_line`] to find what is already
//! highlighted at the left edge, then [`update_search_hl`] per column.
//! `search_hl` (the `'hlsearch'` pattern) and the window's match list are
//! walked together, ordered by priority, with SEARCH_HL_PRIORITY (0) as
//! `'hlsearch'`'s own place in that order.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn init_search_hl(mut wp: *mut win_T, mut search_hl: *mut match_T) {
    unsafe {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        while !cur.is_null() {
            (*cur).mit_hl.rm = (*cur).mit_match;
            if (*cur).mit_hlg_id == 0 as ::core::ffi::c_int {
                (*cur).mit_hl.attr = 0 as ::core::ffi::c_int;
            } else {
                (*cur).mit_hl.attr = syn_id2attr((*cur).mit_hlg_id);
            }
            (*cur).mit_hl.buf = (*wp).w_buffer;
            (*cur).mit_hl.lnum = 0 as ::core::ffi::c_int as linenr_T;
            (*cur).mit_hl.first_lnum = 0 as ::core::ffi::c_int as linenr_T;
            (*cur).mit_hl.tm = profile_setlimit(p_rdt.get() as int64_t);
            cur = (*cur).mit_next;
        }
        (*search_hl).buf = (*wp).w_buffer;
        (*search_hl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*search_hl).first_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*search_hl).attr = win_hl_attr(wp, HLF_L);
    }
}

pub(crate) unsafe extern "C" fn next_search_hl_pos(
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut match_0: *mut matchitem_T,
    mut mincol: colnr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut found: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        let mut i: ::core::ffi::c_int = (*match_0).mit_pos_cur;
        while i < (*match_0).mit_pos_count {
            let mut pos: *mut llpos_T = (*match_0).mit_pos_array.offset(i as isize);
            if (*pos).lnum == 0 as linenr_T {
                break;
            }
            if !((*pos).len == 0 as ::core::ffi::c_int && (*pos).col < mincol) {
                if (*pos).lnum == lnum {
                    if found >= 0 as ::core::ffi::c_int {
                        if (*pos).col < (*(*match_0).mit_pos_array.offset(found as isize)).col {
                            let mut tmp: llpos_T = *pos;
                            *pos = *(*match_0).mit_pos_array.offset(found as isize);
                            *(*match_0).mit_pos_array.offset(found as isize) = tmp;
                        }
                    } else {
                        found = i;
                    }
                }
            }
            i += 1;
        }
        (*match_0).mit_pos_cur = 0 as ::core::ffi::c_int;
        if found >= 0 as ::core::ffi::c_int {
            let mut start: colnr_T = if (*(*match_0).mit_pos_array.offset(found as isize)).col
                == 0 as ::core::ffi::c_int
            {
                0 as colnr_T
            } else {
                (*(*match_0).mit_pos_array.offset(found as isize)).col - 1 as colnr_T
            };
            let mut end: colnr_T = if (*(*match_0).mit_pos_array.offset(found as isize)).col
                == 0 as ::core::ffi::c_int
            {
                MAXCOL as ::core::ffi::c_int
            } else {
                start + (*(*match_0).mit_pos_array.offset(found as isize)).len as colnr_T
            };
            (*shl).lnum = lnum;
            (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum =
                0 as ::core::ffi::c_int as linenr_T;
            (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col = start;
            (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum =
                0 as ::core::ffi::c_int as linenr_T;
            (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col = end;
            (*shl).is_addpos = true_0 != 0;
            (*shl).has_cursor = false_0 != 0;
            (*match_0).mit_pos_cur = found + 1 as ::core::ffi::c_int;
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn next_search_hl(
    mut win: *mut win_T,
    mut search_hl: *mut match_T,
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut cur: *mut matchitem_T,
) {
    unsafe {
        let mut matchcol: colnr_T = 0;
        let mut nmatched: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
        if (lnum < search_first_line.get() || lnum > search_last_line.get()) && cur.is_null() {
            (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
            return;
        }
        if (*shl).lnum != 0 as linenr_T {
            let mut l: linenr_T = (*shl).lnum
                + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
            if lnum > l {
                (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
            } else if lnum < l || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol {
                return;
            }
        }
        loop {
            if profile_passed_limit((*shl).tm) {
                (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                break;
            } else {
                if (*shl).lnum == 0 as linenr_T {
                    matchcol = 0 as ::core::ffi::c_int as colnr_T;
                } else if vim_strchr(p_cpo.get(), CPO_SEARCH).is_null()
                    || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T
                        && (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                            <= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col
                {
                    matchcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                    let mut ml: *mut ::core::ffi::c_char =
                        ml_get_buf((*shl).buf, lnum).offset(matchcol as isize);
                    if *ml as ::core::ffi::c_int == NUL {
                        matchcol += 1;
                        (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                        break;
                    } else {
                        matchcol += utfc_ptr2len(ml);
                    }
                } else {
                    matchcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
                }
                (*shl).lnum = lnum;
                if !(*shl).rm.regprog.is_null() {
                    let mut regprog_is_copy: bool = shl != search_hl
                        && !cur.is_null()
                        && shl == &raw mut (*cur).mit_hl
                        && ::core::ptr::addr_eq((*cur).mit_match.regprog, (*cur).mit_hl.rm.regprog);
                    let mut timed_out: ::core::ffi::c_int = false_0;
                    nmatched = vim_regexec_multi(
                        &raw mut (*shl).rm,
                        win,
                        (*shl).buf,
                        lnum,
                        matchcol,
                        &raw mut (*shl).tm,
                        &raw mut timed_out,
                    );
                    if regprog_is_copy {
                        (*cur).mit_match.regprog = (*cur).mit_hl.rm.regprog;
                    }
                    if called_emsg.get() > called_emsg_before
                        || got_int.get() as ::core::ffi::c_int != 0
                        || timed_out != 0
                    {
                        if shl == search_hl {
                            vim_regfree((*shl).rm.regprog);
                            set_no_hlsearch(true_0 != 0);
                        }
                        (*shl).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
                        (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                        got_int.set(false_0 != 0);
                        break;
                    }
                } else if !cur.is_null() {
                    nmatched = next_search_hl_pos(shl, lnum, cur, matchcol);
                }
                if nmatched == 0 as ::core::ffi::c_int {
                    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                    break;
                } else {
                    if !((*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T
                        || (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col >= mincol
                        || nmatched > 1 as ::core::ffi::c_int
                        || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol)
                    {
                        continue;
                    }
                    (*shl).lnum += (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    break;
                }
            }
        }
    }
}

pub unsafe extern "C" fn prepare_search_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut lnum: linenr_T,
) {
    unsafe {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
        let mut shl_flag: bool = false_0 != 0;
        while !cur.is_null() || shl_flag as ::core::ffi::c_int == false_0 {
            if shl_flag as ::core::ffi::c_int == false_0 {
                shl = search_hl;
                shl_flag = true_0 != 0;
            } else {
                shl = &raw mut (*cur).mit_hl;
            }
            if !(*shl).rm.regprog.is_null()
                && (*shl).lnum == 0 as linenr_T
                && re_multiline((*shl).rm.regprog) != 0
            {
                if (*shl).first_lnum == 0 as linenr_T {
                    (*shl).first_lnum = lnum;
                    while (*shl).first_lnum > (*wp).w_topline {
                        if hasFolding(
                            wp,
                            (*shl).first_lnum - 1 as linenr_T,
                            ::core::ptr::null_mut::<linenr_T>(),
                            ::core::ptr::null_mut::<linenr_T>(),
                        ) {
                            break;
                        }
                        (*shl).first_lnum -= 1;
                    }
                }
                if !cur.is_null() {
                    (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
                }
                let mut pos_inprogress: bool = true_0 != 0;
                let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (*shl).first_lnum < lnum
                    && (!(*shl).rm.regprog.is_null()
                        || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0)
                {
                    next_search_hl(
                        wp,
                        search_hl,
                        shl,
                        (*shl).first_lnum,
                        n,
                        if shl == search_hl {
                            ::core::ptr::null_mut::<matchitem_T>()
                        } else {
                            cur
                        },
                    );
                    pos_inprogress =
                        !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                    if (*shl).lnum != 0 as linenr_T {
                        (*shl).first_lnum = (*shl).lnum
                            + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                            - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                        n = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                            as ::core::ffi::c_int;
                    } else {
                        (*shl).first_lnum += 1;
                        n = 0 as ::core::ffi::c_int;
                    }
                }
            }
            if shl != search_hl && !cur.is_null() {
                cur = (*cur).mit_next;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn check_cur_search_hl(mut wp: *mut win_T, mut shl: *mut match_T) {
    unsafe {
        let mut linecount: linenr_T = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
            - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
        if (*wp).w_cursor.lnum >= (*shl).lnum
            && (*wp).w_cursor.lnum <= (*shl).lnum + linecount
            && ((*wp).w_cursor.lnum > (*shl).lnum
                || (*wp).w_cursor.col >= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col)
            && ((*wp).w_cursor.lnum < (*shl).lnum + linecount
                || (*wp).w_cursor.col < (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col)
        {
            (*shl).has_cursor = true_0 != 0;
        } else {
            (*shl).has_cursor = false_0 != 0;
        };
    }
}

pub unsafe extern "C" fn prepare_search_hl_line(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut search_attr: *mut ::core::ffi::c_int,
    mut search_attr_from_match: *mut bool,
) -> bool {
    unsafe {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
        let mut shl_flag: bool = false_0 != 0;
        let mut area_highlighting: bool = false_0 != 0;
        while !cur.is_null() || !shl_flag {
            if !shl_flag {
                shl = search_hl;
                shl_flag = true_0 != 0;
            } else {
                shl = &raw mut (*cur).mit_hl;
            }
            (*shl).startcol = MAXCOL as ::core::ffi::c_int as colnr_T;
            (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
            (*shl).attr_cur = 0 as ::core::ffi::c_int;
            (*shl).is_addpos = false_0 != 0;
            (*shl).has_cursor = false_0 != 0;
            if !cur.is_null() {
                (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
            }
            next_search_hl(
                wp,
                search_hl,
                shl,
                lnum,
                mincol,
                if shl == search_hl {
                    ::core::ptr::null_mut::<matchitem_T>()
                } else {
                    cur
                },
            );
            *line = ml_get_buf((*wp).w_buffer, lnum);
            if (*shl).lnum != 0 as linenr_T && (*shl).lnum <= lnum {
                if (*shl).lnum == lnum {
                    (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                } else {
                    (*shl).startcol = 0 as ::core::ffi::c_int as colnr_T;
                }
                if lnum
                    == (*shl).lnum + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                        - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum
                {
                    (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
                } else {
                    (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
                }
                if shl == search_hl {
                    check_cur_search_hl(wp, shl);
                }
                if (*shl).startcol == (*shl).endcol {
                    if *(*line).offset((*shl).endcol as isize) as ::core::ffi::c_int != NUL {
                        (*shl).endcol += utfc_ptr2len((*line).offset((*shl).endcol as isize));
                    } else {
                        (*shl).endcol += 1;
                    }
                }
                if (*shl).startcol < mincol {
                    (*shl).attr_cur = (*shl).attr;
                    *search_attr = (*shl).attr;
                    *search_attr_from_match = shl != search_hl;
                }
                area_highlighting = true_0 != 0;
            }
            if shl != search_hl && !cur.is_null() {
                cur = (*cur).mit_next;
            }
        }
        return area_highlighting;
    }
}

pub unsafe extern "C" fn update_search_hl(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut has_match_conc: *mut ::core::ffi::c_int,
    mut match_conc: *mut ::core::ffi::c_int,
    mut lcs_eol_todo: bool,
    mut on_last_col: *mut bool,
    mut search_attr_from_match: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
        let mut shl_flag: bool = false_0 != 0;
        let mut search_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !cur.is_null() || !shl_flag {
            if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
                shl = search_hl;
                shl_flag = true_0 != 0;
            } else {
                shl = &raw mut (*cur).mit_hl;
            }
            if !cur.is_null() {
                (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
            }
            let mut pos_inprogress: bool = true_0 != 0;
            while !(*shl).rm.regprog.is_null()
                || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0
            {
                if (*shl).startcol != MAXCOL as ::core::ffi::c_int
                    && col >= (*shl).startcol
                    && col < (*shl).endcol
                {
                    let mut next_col: ::core::ffi::c_int =
                        col as ::core::ffi::c_int + utfc_ptr2len((*line).offset(col as isize));
                    if (*shl).endcol < next_col {
                        (*shl).endcol = next_col as colnr_T;
                    }
                    if shl == search_hl && (*shl).has_cursor as ::core::ffi::c_int != 0 {
                        (*shl).attr_cur = win_hl_attr(wp, HLF_LC);
                        if (*shl).attr_cur != (*shl).attr {
                            search_hl_has_cursor_lnum.set(lnum);
                        }
                    } else {
                        (*shl).attr_cur = (*shl).attr;
                    }
                    if !cur.is_null()
                        && shl != search_hl
                        && syn_name2id(b"Conceal\0".as_ptr() as *const ::core::ffi::c_char)
                            == (*cur).mit_hlg_id
                    {
                        *has_match_conc = if col == (*shl).startcol {
                            2 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        };
                        *match_conc = (*cur).mit_conceal_char;
                    } else {
                        *has_match_conc = 0 as ::core::ffi::c_int;
                    }
                    break;
                } else {
                    if col != (*shl).endcol {
                        break;
                    }
                    (*shl).attr_cur = 0 as ::core::ffi::c_int;
                    next_search_hl(
                        wp,
                        search_hl,
                        shl,
                        lnum,
                        col,
                        if shl == search_hl {
                            ::core::ptr::null_mut::<matchitem_T>()
                        } else {
                            cur
                        },
                    );
                    pos_inprogress =
                        !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                    *line = ml_get_buf((*wp).w_buffer, lnum);
                    if (*shl).lnum != lnum {
                        break;
                    }
                    (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                    if (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T {
                        (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
                    } else {
                        (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
                    }
                    if shl == search_hl {
                        check_cur_search_hl(wp, shl);
                    }
                    if (*shl).startcol == (*shl).endcol {
                        let mut p: *mut ::core::ffi::c_char =
                            (*line).offset((*shl).endcol as isize);
                        if *p as ::core::ffi::c_int == NUL {
                            (*shl).endcol += 1;
                        } else {
                            (*shl).endcol += utfc_ptr2len(p);
                        }
                    }
                }
            }
            if shl != search_hl && !cur.is_null() {
                cur = (*cur).mit_next;
            }
        }
        *search_attr_from_match = false_0 != 0;
        search_attr = (*search_hl).attr_cur;
        cur = (*wp).w_match_head;
        shl_flag = false_0 != 0;
        while !cur.is_null() || !shl_flag {
            if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
                shl = search_hl;
                shl_flag = true_0 != 0;
            } else {
                shl = &raw mut (*cur).mit_hl;
            }
            if (*shl).attr_cur != 0 as ::core::ffi::c_int {
                search_attr = (*shl).attr_cur;
                *on_last_col = col as ::core::ffi::c_int + 1 as ::core::ffi::c_int >= (*shl).endcol;
                *search_attr_from_match = shl != search_hl;
            }
            if shl != search_hl && !cur.is_null() {
                cur = (*cur).mit_next;
            }
        }
        if *(*line).offset(col as isize) as ::core::ffi::c_int == NUL
            && ((*wp).w_onebuf_opt.wo_list != 0 && !lcs_eol_todo)
        {
            search_attr = 0 as ::core::ffi::c_int;
        }
        return search_attr;
    }
}

pub unsafe extern "C" fn get_prevcol_hl_flag(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut curcol: colnr_T,
) -> bool {
    unsafe {
        let mut prevcol: colnr_T = curcol;
        if (if (*wp).w_onebuf_opt.wo_wrap != 0 {
            (*wp).w_skipcol
        } else {
            (*wp).w_leftcol
        }) > prevcol
        {
            prevcol += 1;
        }
        if !(*search_hl).is_addpos
            && (prevcol == (*search_hl).startcol
                || prevcol > (*search_hl).startcol
                    && (*search_hl).endcol == MAXCOL as ::core::ffi::c_int)
        {
            return true_0 != 0;
        }
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        while !cur.is_null() {
            if !(*cur).mit_hl.is_addpos
                && (prevcol == (*cur).mit_hl.startcol
                    || prevcol > (*cur).mit_hl.startcol
                        && (*cur).mit_hl.endcol == MAXCOL as ::core::ffi::c_int)
            {
                return true_0 != 0;
            }
            cur = (*cur).mit_next;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn get_search_match_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut col: colnr_T,
    mut char_attr: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
        let mut shl_flag: bool = false_0 != 0;
        while !cur.is_null() || !shl_flag {
            if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
                shl = search_hl;
                shl_flag = true_0 != 0;
            } else {
                shl = &raw mut (*cur).mit_hl;
            }
            if col as ::core::ffi::c_int - 1 as ::core::ffi::c_int == (*shl).startcol
                && (shl == search_hl || !(*shl).is_addpos)
            {
                *char_attr = (*shl).attr;
            }
            if shl != search_hl && !cur.is_null() {
                cur = (*cur).mit_next;
            }
        }
    }
}
