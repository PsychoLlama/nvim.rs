//! Running a pattern over the buffer.
//!
//! [`searchit`] is the one searcher underneath `/`, `?`, `n`, `N`, `*`,
//! `gd`, `:substitute`'s address form and the tag jumps: it walks lines
//! from a starting position in one direction, wrapping at the end of the
//! buffer when `'wrapscan'` is set, and applies the search offset the
//! pattern carried. [`search_for_exact_line`] is the unrelated plain-text
//! line scanner insert-mode line completion uses.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn searchit(
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut end_pos: *mut pos_T,
    mut dir: Direction,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut count: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut pat_use: ::core::ffi::c_int,
    mut extra_arg: *mut searchit_arg_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut found: ::core::ffi::c_int = 0;
        let mut lnum: linenr_T = 0;
        let mut regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut matchcol: colnr_T = 0;
        let mut endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut matchpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut loop_0: ::core::ffi::c_int = 0;
        let mut extra_col: ::core::ffi::c_int = 0;
        let mut start_char_len: ::core::ffi::c_int = 0;
        let mut match_ok: bool = false;
        let mut nmatched: ::core::ffi::c_int = 0;
        let mut submatch: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut first_match: bool = true_0 != 0;
        let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
        let mut break_loop: bool = false_0 != 0;
        let mut stop_lnum: linenr_T = 0 as linenr_T;
        let mut tm: *mut proftime_T = ::core::ptr::null_mut::<proftime_T>();
        let mut timed_out: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
        if !extra_arg.is_null() {
            stop_lnum = (*extra_arg).sa_stop_lnum;
            tm = (*extra_arg).sa_tm;
            timed_out = &raw mut (*extra_arg).sa_timed_out;
        }
        if search_regcomp(
            pat,
            patlen,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            RE_SEARCH as ::core::ffi::c_int,
            pat_use,
            options & SEARCH_HIS as ::core::ffi::c_int + SEARCH_KEEP as ::core::ffi::c_int,
            &raw mut regmatch,
        ) == FAIL
        {
            if options & SEARCH_MSG as ::core::ffi::c_int != 0 && !rc_did_emsg.get() {
                semsg(
                    gettext(
                        b"E383: Invalid search string: %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    get_search_pat(),
                );
            }
            return FAIL;
        }
        let search_from_match_end: bool = !vim_strchr(p_cpo.get(), CPO_SEARCH).is_null();
        loop {
            if (*pos).col == MAXCOL as ::core::ffi::c_int {
                start_char_len = 0 as ::core::ffi::c_int;
            } else if (*pos).lnum >= 1 as linenr_T
                && (*pos).lnum <= (*buf).b_ml.ml_line_count
                && (*pos).col < MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int
            {
                ptr = ml_get_buf(buf, (*pos).lnum);
                if ml_get_buf_len(buf, (*pos).lnum) <= (*pos).col {
                    start_char_len = 1 as ::core::ffi::c_int;
                } else {
                    start_char_len = utfc_ptr2len(ptr.offset((*pos).col as isize));
                }
            } else {
                start_char_len = 1 as ::core::ffi::c_int;
            }
            if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                extra_col = if options & SEARCH_START as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    start_char_len
                };
            } else {
                extra_col = if options & SEARCH_START as ::core::ffi::c_int != 0 {
                    start_char_len
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            let mut start_pos: pos_T = *pos;
            found = 0 as ::core::ffi::c_int;
            let mut at_first_line: ::core::ffi::c_int = true_0;
            if (*pos).lnum == 0 as linenr_T {
                (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
                (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
                at_first_line = false_0;
            }
            if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
                && start_pos.col == 0 as ::core::ffi::c_int
                && options & SEARCH_START as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                lnum = (*pos).lnum - 1 as linenr_T;
                at_first_line = false_0;
            } else {
                lnum = (*pos).lnum;
            }
            loop_0 = 0 as ::core::ffi::c_int;
            while loop_0 <= 1 as ::core::ffi::c_int {
                's_704: while lnum > 0 as linenr_T && lnum <= (*buf).b_ml.ml_line_count {
                    if stop_lnum != 0 as linenr_T
                        && (if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                            (lnum > stop_lnum) as ::core::ffi::c_int
                        } else {
                            (lnum < stop_lnum) as ::core::ffi::c_int
                        }) != 0
                    {
                        break;
                    }
                    if !tm.is_null() && profile_passed_limit(*tm) as ::core::ffi::c_int != 0 {
                        break;
                    }
                    let mut col: colnr_T =
                        if at_first_line != 0 && options & SEARCH_COL as ::core::ffi::c_int != 0 {
                            (*pos).col
                        } else {
                            0 as colnr_T
                        };
                    nmatched =
                        vim_regexec_multi(&raw mut regmatch, win, buf, lnum, col, tm, timed_out);
                    if regmatch.regprog.is_null() {
                        break;
                    }
                    if called_emsg.get() > called_emsg_before
                        || !timed_out.is_null() && *timed_out != 0
                    {
                        break;
                    }
                    's_218: {
                        if nmatched > 0 as ::core::ffi::c_int {
                            matchpos = regmatch.startpos[0 as ::core::ffi::c_int as usize];
                            endpos = regmatch.endpos[0 as ::core::ffi::c_int as usize];
                            submatch = first_submatch(&raw mut regmatch);
                            if lnum + matchpos.lnum > (*buf).b_ml.ml_line_count {
                                ptr = b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                ptr = ml_get_buf(buf, lnum + matchpos.lnum);
                            }
                            if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                                && at_first_line != 0
                            {
                                match_ok = true_0 != 0;
                                while matchpos.lnum == 0 as linenr_T
                                    && (if options & SEARCH_END as ::core::ffi::c_int != 0
                                        && first_match as ::core::ffi::c_int != 0
                                    {
                                        (nmatched == 1 as ::core::ffi::c_int
                                            && (endpos.col - 1 as ::core::ffi::c_int)
                                                < start_pos.col + extra_col)
                                            as ::core::ffi::c_int
                                    } else {
                                        ((matchpos.col
                                            - (*ptr.offset(matchpos.col as isize)
                                                as ::core::ffi::c_int
                                                == NUL)
                                                as ::core::ffi::c_int)
                                            < start_pos.col + extra_col)
                                            as ::core::ffi::c_int
                                    }) != 0
                                {
                                    if search_from_match_end {
                                        if nmatched > 1 as ::core::ffi::c_int {
                                            match_ok = false_0 != 0;
                                            break;
                                        } else {
                                            matchcol = endpos.col;
                                            if matchcol == matchpos.col
                                                && *ptr.offset(matchcol as isize)
                                                    as ::core::ffi::c_int
                                                    != NUL
                                            {
                                                matchcol +=
                                                    utfc_ptr2len(ptr.offset(matchcol as isize));
                                            }
                                        }
                                    } else {
                                        matchcol = regmatch.rmm_matchcol;
                                        if *ptr.offset(matchcol as isize) as ::core::ffi::c_int
                                            != NUL
                                        {
                                            matchcol += utfc_ptr2len(ptr.offset(matchcol as isize));
                                        }
                                    }
                                    if matchcol == 0 as ::core::ffi::c_int
                                        && options & SEARCH_START as ::core::ffi::c_int != 0
                                    {
                                        break;
                                    }
                                    if *ptr.offset(matchcol as isize) as ::core::ffi::c_int == NUL
                                        || {
                                            nmatched = vim_regexec_multi(
                                                &raw mut regmatch,
                                                win,
                                                buf,
                                                lnum,
                                                matchcol,
                                                tm,
                                                timed_out,
                                            );
                                            nmatched == 0 as ::core::ffi::c_int
                                        }
                                    {
                                        match_ok = false_0 != 0;
                                        break;
                                    } else {
                                        if regmatch.regprog.is_null() {
                                            break;
                                        }
                                        matchpos =
                                            regmatch.startpos[0 as ::core::ffi::c_int as usize];
                                        endpos = regmatch.endpos[0 as ::core::ffi::c_int as usize];
                                        submatch = first_submatch(&raw mut regmatch);
                                        if matchpos.lnum != 0 as linenr_T {
                                            break;
                                        }
                                        ptr = ml_get_buf(buf, lnum);
                                    }
                                }
                                if !match_ok {
                                    break 's_218;
                                }
                            }
                            if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                                match_ok = false_0 != 0;
                                while loop_0 != 0
                                    || (if options & SEARCH_END as ::core::ffi::c_int != 0 {
                                        (lnum
                                            + regmatch.endpos[0 as ::core::ffi::c_int as usize]
                                                .lnum
                                            < start_pos.lnum
                                            || lnum
                                                + regmatch.endpos[0 as ::core::ffi::c_int as usize]
                                                    .lnum
                                                == start_pos.lnum
                                                && (regmatch.endpos
                                                    [0 as ::core::ffi::c_int as usize]
                                                    .col
                                                    - 1 as ::core::ffi::c_int)
                                                    < start_pos.col + extra_col)
                                            as ::core::ffi::c_int
                                    } else {
                                        (lnum
                                            + regmatch.startpos[0 as ::core::ffi::c_int as usize]
                                                .lnum
                                            < start_pos.lnum
                                            || lnum
                                                + regmatch.startpos
                                                    [0 as ::core::ffi::c_int as usize]
                                                    .lnum
                                                == start_pos.lnum
                                                && regmatch.startpos
                                                    [0 as ::core::ffi::c_int as usize]
                                                    .col
                                                    < start_pos.col + extra_col)
                                            as ::core::ffi::c_int
                                    }) != 0
                                {
                                    match_ok = true_0 != 0;
                                    matchpos = regmatch.startpos[0 as ::core::ffi::c_int as usize];
                                    endpos = regmatch.endpos[0 as ::core::ffi::c_int as usize];
                                    submatch = first_submatch(&raw mut regmatch);
                                    if search_from_match_end {
                                        if nmatched > 1 as ::core::ffi::c_int {
                                            break;
                                        }
                                        matchcol = endpos.col;
                                        if matchcol == matchpos.col
                                            && *ptr.offset(matchcol as isize) as ::core::ffi::c_int
                                                != NUL
                                        {
                                            matchcol += utfc_ptr2len(ptr.offset(matchcol as isize));
                                        }
                                    } else {
                                        if matchpos.lnum > 0 as linenr_T {
                                            break;
                                        }
                                        matchcol = matchpos.col;
                                        if *ptr.offset(matchcol as isize) as ::core::ffi::c_int
                                            != NUL
                                        {
                                            matchcol += utfc_ptr2len(ptr.offset(matchcol as isize));
                                        }
                                    }
                                    if *ptr.offset(matchcol as isize) as ::core::ffi::c_int == NUL
                                        || {
                                            nmatched = vim_regexec_multi(
                                                &raw mut regmatch,
                                                win,
                                                buf,
                                                lnum + matchpos.lnum,
                                                matchcol,
                                                tm,
                                                timed_out,
                                            );
                                            nmatched == 0 as ::core::ffi::c_int
                                        }
                                    {
                                        if !tm.is_null()
                                            && profile_passed_limit(*tm) as ::core::ffi::c_int != 0
                                        {
                                            match_ok = false_0 != 0;
                                        }
                                        break;
                                    } else {
                                        if regmatch.regprog.is_null() {
                                            break;
                                        }
                                        ptr = ml_get_buf(buf, lnum + matchpos.lnum);
                                    }
                                }
                                if !match_ok {
                                    break 's_218;
                                }
                            }
                            if options & SEARCH_END as ::core::ffi::c_int != 0
                                && options & SEARCH_NOOF as ::core::ffi::c_int == 0
                                && !(matchpos.lnum == endpos.lnum && matchpos.col == endpos.col)
                            {
                                (*pos).lnum = lnum + endpos.lnum;
                                (*pos).col = endpos.col;
                                if endpos.col == 0 as ::core::ffi::c_int {
                                    if (*pos).lnum > 1 as linenr_T {
                                        (*pos).lnum -= 1;
                                        (*pos).col = ml_get_buf_len(buf, (*pos).lnum);
                                    }
                                } else {
                                    (*pos).col -= 1;
                                    if (*pos).lnum <= (*buf).b_ml.ml_line_count {
                                        ptr = ml_get_buf(buf, (*pos).lnum);
                                        (*pos).col -=
                                            utf_head_off(ptr, ptr.offset((*pos).col as isize));
                                    }
                                }
                                if !end_pos.is_null() {
                                    (*end_pos).lnum = lnum + matchpos.lnum;
                                    (*end_pos).col = matchpos.col;
                                }
                            } else {
                                (*pos).lnum = lnum + matchpos.lnum;
                                (*pos).col = matchpos.col;
                                if !end_pos.is_null() {
                                    (*end_pos).lnum = lnum + endpos.lnum;
                                    (*end_pos).col = endpos.col;
                                }
                            }
                            (*pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
                            if !end_pos.is_null() {
                                (*end_pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
                            }
                            found = 1 as ::core::ffi::c_int;
                            first_match = false_0 != 0;
                            search_match_lines.set(endpos.lnum - matchpos.lnum);
                            search_match_endcol.set(endpos.col);
                            break 's_704;
                        } else {
                            line_breakcheck();
                            if got_int.get() {
                                break 's_704;
                            }
                            if options & SEARCH_PEEK as ::core::ffi::c_int != 0
                                && lnum - (*pos).lnum & 0x3f as linenr_T == 0 as linenr_T
                                && char_avail() as ::core::ffi::c_int != 0
                            {
                                break_loop = true_0 != 0;
                                break 's_704;
                            } else if loop_0 != 0 && lnum == start_pos.lnum {
                                break 's_704;
                            }
                        }
                    }
                    lnum = (lnum as ::core::ffi::c_int + dir as ::core::ffi::c_int) as linenr_T;
                    at_first_line = false_0;
                }
                at_first_line = false_0;
                if regmatch.regprog.is_null() {
                    break;
                }
                if p_ws.get() == 0
                    || stop_lnum != 0 as linenr_T
                    || got_int.get() as ::core::ffi::c_int != 0
                    || called_emsg.get() > called_emsg_before
                    || !timed_out.is_null() && *timed_out != 0
                    || break_loop as ::core::ffi::c_int != 0
                    || found != 0
                    || loop_0 != 0
                {
                    break;
                }
                lnum = if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                    (*buf).b_ml.ml_line_count
                } else {
                    1 as linenr_T
                };
                if !shortmess(SHM_SEARCH as ::core::ffi::c_int)
                    && shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    && options & SEARCH_MSG as ::core::ffi::c_int != 0
                {
                    give_warning(
                        gettext(
                            if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                                &raw const top_bot_msg as *const ::core::ffi::c_char
                            } else {
                                &raw const bot_top_msg as *const ::core::ffi::c_char
                            },
                        ),
                        true_0 != 0,
                        false_0 != 0,
                    );
                }
                if !extra_arg.is_null() {
                    (*extra_arg).sa_wrapped = true_0;
                }
                loop_0 += 1;
            }
            if got_int.get() as ::core::ffi::c_int != 0
                || called_emsg.get() > called_emsg_before
                || !timed_out.is_null() && *timed_out != 0
                || break_loop as ::core::ffi::c_int != 0
            {
                break;
            }
            count -= 1;
            if !(count > 0 as ::core::ffi::c_int && found != 0) {
                break;
            }
        }
        vim_regfree(regmatch.regprog);
        if found == 0 {
            if got_int.get() {
                emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
            } else if options & SEARCH_MSG as ::core::ffi::c_int == SEARCH_MSG as ::core::ffi::c_int
            {
                if p_ws.get() != 0 {
                    semsg(
                        gettext(&raw const e_patnotf2 as *const ::core::ffi::c_char),
                        get_search_pat(),
                    );
                } else if lnum == 0 as linenr_T {
                    semsg(
                        gettext(
                            (e_search_hit_top_without_match_for_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        get_search_pat(),
                    );
                } else {
                    semsg(
                        gettext(
                            (e_search_hit_bottom_without_match_for_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        get_search_pat(),
                    );
                }
            }
            return FAIL;
        }
        if (*pos).lnum > (*buf).b_ml.ml_line_count {
            (*pos).lnum = (*buf).b_ml.ml_line_count;
            (*pos).col = ml_get_buf_len(buf, (*pos).lnum);
            if (*pos).col > 0 as ::core::ffi::c_int {
                (*pos).col -= 1;
            }
        }
        return submatch + 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn first_submatch(mut rp: *mut regmmatch_T) -> ::core::ffi::c_int {
    unsafe {
        let mut submatch: ::core::ffi::c_int = 0;
        submatch = 1 as ::core::ffi::c_int;
        while (*rp).startpos[submatch as usize].lnum < 0 as linenr_T {
            if submatch == 9 as ::core::ffi::c_int {
                submatch = 0 as ::core::ffi::c_int;
                break;
            } else {
                submatch += 1;
            }
        }
        return submatch;
    }
}

pub unsafe extern "C" fn search_for_exact_line(
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut dir: Direction,
    mut pat: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut start: linenr_T = 0 as linenr_T;
        let mut compl_len: ::core::ffi::c_int = ins_compl_len();
        if (*buf).b_ml.ml_line_count == 0 as linenr_T {
            return FAIL;
        }
        loop {
            (*pos).lnum =
                ((*pos).lnum as ::core::ffi::c_int + dir as ::core::ffi::c_int) as linenr_T;
            if (*pos).lnum < 1 as linenr_T {
                if p_ws.get() != 0 {
                    (*pos).lnum = (*buf).b_ml.ml_line_count;
                    if !shortmess(SHM_SEARCH as ::core::ffi::c_int) {
                        give_warning(
                            gettext(&raw const top_bot_msg as *const ::core::ffi::c_char),
                            true_0 != 0,
                            false_0 != 0,
                        );
                    }
                } else {
                    (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
                    break;
                }
            } else if (*pos).lnum > (*buf).b_ml.ml_line_count {
                if p_ws.get() != 0 {
                    (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
                    if !shortmess(SHM_SEARCH as ::core::ffi::c_int) {
                        give_warning(
                            gettext(&raw const bot_top_msg as *const ::core::ffi::c_char),
                            true_0 != 0,
                            false_0 != 0,
                        );
                    }
                } else {
                    (*pos).lnum = 1 as ::core::ffi::c_int as linenr_T;
                    break;
                }
            }
            if (*pos).lnum == start {
                break;
            }
            if start == 0 as linenr_T {
                start = (*pos).lnum;
            }
            let mut ptr: *mut ::core::ffi::c_char = ml_get_buf(buf, (*pos).lnum);
            let mut p: *mut ::core::ffi::c_char = skipwhite(ptr);
            (*pos).col = p.offset_from(ptr) as colnr_T;
            if compl_status_adding() as ::core::ffi::c_int != 0 && !compl_status_sol() {
                if mb_strcmp_ic(p_ic.get() != 0, p, pat) == 0 as ::core::ffi::c_int {
                    return OK;
                }
            } else if *p as ::core::ffi::c_int != NUL {
                '_c2rust_label: {
                    if compl_len >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"compl_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/search.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1519 as ::core::ffi::c_uint,
                            b"int search_for_exact_line(buf_T *, pos_T *, Direction, char *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if (if p_ic.get() != 0 {
                    mb_strnicmp(p, pat, compl_len as size_t)
                } else {
                    strncmp(p, pat, compl_len as size_t)
                }) == 0 as ::core::ffi::c_int
                {
                    return OK;
                }
            }
        }
        return FAIL;
    }
}
