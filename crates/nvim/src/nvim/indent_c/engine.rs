//! `get_c_indent` -- the C indent itself.
//!
//! Still one 1,866-line function here, and still over the file cap: a carve
//! cannot split a single over-cap item, so this file is a holding pen until the
//! rewrite decomposes it by 'cinoptions' concern.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_c_indent() -> ::core::ffi::c_int {
    unsafe {
        let mut cur_curpos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut amount: ::core::ffi::c_int = 0;
        let mut scope_amount: ::core::ffi::c_int = 0;
        let mut cur_amount: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
        let mut col: colnr_T = 0;
        let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut linecopy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut comment_pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut tryposBrace: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut tryposCopy: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut our_paren_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut start_brace: ::core::ffi::c_int = 0;
        let mut ourscope: linenr_T = 0;
        let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut look: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut terminated: u8 = 0;
        let mut lookfor: ::core::ffi::c_int = 0;
        let mut whilelevel: ::core::ffi::c_int = 0;
        let mut lnum: linenr_T = 0;
        let mut n: ::core::ffi::c_int = 0;
        let mut lookfor_break: ::core::ffi::c_int = 0;
        let mut lookfor_cpp_namespace: bool = false;
        let mut cont_amount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut original_line_islabel: ::core::ffi::c_int = 0;
        let mut added_to_amount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut raw_string_start: linenr_T = 0 as linenr_T;
        let mut cache_cpp_baseclass: cpp_baseclass_cache_T = cpp_baseclass_cache_T {
            found: false_0,
            lpos: lpos_T {
                lnum: MAXLNUM as ::core::ffi::c_int as linenr_T,
                col: 0 as colnr_T,
            },
        };
        let mut ind_continuation: ::core::ffi::c_int = (*curbuf.get()).b_ind_continuation;
        cur_curpos = (*curwin.get()).w_cursor;
        if cur_curpos.lnum == 1 as linenr_T {
            return 0 as ::core::ffi::c_int;
        }
        linecopy = xstrdup(ml_get(cur_curpos.lnum));
        if State.get() & MODE_INSERT != 0
            && (*curwin.get()).w_cursor.col < strlen(linecopy) as colnr_T
            && *linecopy.offset((*curwin.get()).w_cursor.col as isize) as ::core::ffi::c_int
                == ')' as ::core::ffi::c_int
        {
            *linecopy.offset((*curwin.get()).w_cursor.col as isize) = NUL as ::core::ffi::c_char;
        }
        theline = skipwhite(linecopy);
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        original_line_islabel = cin_islabel() as ::core::ffi::c_int;
        comment_pos = ind_find_start_comment();
        if !comment_pos.is_null() {
            tryposCopy = *comment_pos;
            comment_pos = &raw mut tryposCopy;
        }
        trypos = find_start_rawstring((*curbuf.get()).b_ind_maxcomment);
        if !trypos.is_null()
            && (comment_pos.is_null() || lt(*trypos, *comment_pos) as ::core::ffi::c_int != 0)
        {
            amount = -1 as ::core::ffi::c_int;
        } else {
            '_theend: {
                if *theline as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                    && (*linecopy as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                        || in_cinkeys('#' as ::core::ffi::c_int, ' ' as ::core::ffi::c_int, true)
                            as ::core::ffi::c_int
                            != 0)
                {
                    let directive: *const ::core::ffi::c_char =
                        skipwhite(theline.offset(1 as ::core::ffi::c_int as isize));
                    if (*curbuf.get()).b_ind_pragma == 0 as ::core::ffi::c_int
                        || strncmp(directive, c"pragma".as_ptr(), 6 as size_t)
                            != 0 as ::core::ffi::c_int
                    {
                        amount = (*curbuf.get()).b_ind_hash_comment;
                        break '_theend;
                    }
                }
                if original_line_islabel != 0
                    && (*curbuf.get()).b_ind_js == 0
                    && (*curbuf.get()).b_ind_jump_label < 0 as ::core::ffi::c_int
                {
                    amount = 0 as ::core::ffi::c_int;
                } else {
                    if cin_islinecomment(theline) {
                        let mut linecomment_pos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        trypos = find_line_comment();
                        if trypos.is_null() && (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                            linecomment_pos.col = check_linecomment(ml_get(
                                (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                            )) as colnr_T;
                            if linecomment_pos.col != MAXCOL as ::core::ffi::c_int {
                                trypos = &raw mut linecomment_pos;
                                (*trypos).lnum = (*curwin.get()).w_cursor.lnum - 1 as linenr_T;
                            }
                        }
                        if !trypos.is_null() {
                            getvcol(
                                curwin.get(),
                                trypos,
                                &raw mut col,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                            );
                            amount = col as ::core::ffi::c_int;
                            break '_theend;
                        }
                    }
                    if !cin_iscomment(theline) && !comment_pos.is_null() {
                        let mut lead_start_len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
                        let mut lead_middle_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        let mut lead_start: [::core::ffi::c_char; 50] = [0; 50];
                        let mut lead_middle: [::core::ffi::c_char; 50] = [0; 50];
                        let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
                        let mut lead_end_len: ::core::ffi::c_int = 0;
                        let mut p: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        let mut start_align: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut start_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut done: ::core::ffi::c_int = false_0;
                        getvcol(
                            curwin.get(),
                            comment_pos,
                            &raw mut col,
                            ::core::ptr::null_mut::<colnr_T>(),
                            ::core::ptr::null_mut::<colnr_T>(),
                        );
                        amount = col as ::core::ffi::c_int;
                        *(&raw mut lead_start as *mut ::core::ffi::c_char) =
                            NUL as ::core::ffi::c_char;
                        *(&raw mut lead_middle as *mut ::core::ffi::c_char) =
                            NUL as ::core::ffi::c_char;
                        p = (*curbuf.get()).b_p_com;
                        while *p as ::core::ffi::c_int != NUL {
                            let mut align: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while *p as ::core::ffi::c_int != NUL
                                && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                            {
                                if *p as ::core::ffi::c_int == COM_START
                                    || *p as ::core::ffi::c_int == COM_END
                                    || *p as ::core::ffi::c_int == COM_MIDDLE
                                {
                                    let c2rust_fresh1 = p;
                                    p = p.offset(1);
                                    what = *c2rust_fresh1 as ::core::ffi::c_uchar
                                        as ::core::ffi::c_int;
                                } else if *p as ::core::ffi::c_int == COM_LEFT
                                    || *p as ::core::ffi::c_int == COM_RIGHT
                                {
                                    let c2rust_fresh2 = p;
                                    p = p.offset(1);
                                    align = *c2rust_fresh2 as ::core::ffi::c_uchar
                                        as ::core::ffi::c_int;
                                } else if ascii_isdigit(*p as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                                    || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                                {
                                    off = getdigits_int(&raw mut p, true, 0 as ::core::ffi::c_int);
                                } else {
                                    p = p.offset(1);
                                }
                            }
                            if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                                p = p.offset(1);
                            }
                            lead_end_len = copy_option_part(
                                &raw mut p,
                                &raw mut lead_end as *mut ::core::ffi::c_char,
                                COM_MAX_LEN as size_t,
                                c",".as_ptr() as *mut ::core::ffi::c_char,
                            ) as ::core::ffi::c_int;
                            if what == COM_START {
                                strcpy(
                                    &raw mut lead_start as *mut ::core::ffi::c_char,
                                    &raw mut lead_end as *mut ::core::ffi::c_char,
                                );
                                lead_start_len = lead_end_len;
                                start_off = off;
                                start_align = align;
                            } else if what == COM_MIDDLE {
                                strcpy(
                                    &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    &raw mut lead_end as *mut ::core::ffi::c_char,
                                );
                                lead_middle_len = lead_end_len;
                            } else {
                                if what != COM_END {
                                    continue;
                                }
                                if strncmp(
                                    theline,
                                    &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    lead_middle_len as size_t,
                                ) == 0 as ::core::ffi::c_int
                                    && strncmp(
                                        theline,
                                        &raw mut lead_end as *mut ::core::ffi::c_char,
                                        lead_end_len as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                {
                                    done = true_0;
                                    if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                                        look = skipwhite(ml_get(
                                            (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                        ));
                                        if strncmp(
                                            look,
                                            &raw mut lead_start as *mut ::core::ffi::c_char,
                                            lead_start_len as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            amount = get_indent_lnum(
                                                (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                            );
                                        } else if strncmp(
                                            look,
                                            &raw mut lead_middle as *mut ::core::ffi::c_char,
                                            lead_middle_len as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            amount = get_indent_lnum(
                                                (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                            );
                                            break;
                                        } else if strncmp(
                                            ml_get((*comment_pos).lnum)
                                                .offset((*comment_pos).col as isize),
                                            &raw mut lead_start as *mut ::core::ffi::c_char,
                                            lead_start_len as size_t,
                                        ) != 0 as ::core::ffi::c_int
                                        {
                                            continue;
                                        }
                                    }
                                    if start_off != 0 as ::core::ffi::c_int {
                                        amount += start_off;
                                    } else if start_align == COM_RIGHT {
                                        amount += vim_strsize(
                                            &raw mut lead_start as *mut ::core::ffi::c_char,
                                        ) - vim_strsize(
                                            &raw mut lead_middle as *mut ::core::ffi::c_char,
                                        );
                                    }
                                    break;
                                } else {
                                    if !(strncmp(
                                        theline,
                                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                                        lead_middle_len as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                        && strncmp(
                                            theline,
                                            &raw mut lead_end as *mut ::core::ffi::c_char,
                                            lead_end_len as size_t,
                                        ) == 0 as ::core::ffi::c_int)
                                    {
                                        continue;
                                    }
                                    amount = get_indent_lnum(
                                        (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                    );
                                    if off != 0 as ::core::ffi::c_int {
                                        amount += off;
                                    } else if align == COM_RIGHT {
                                        amount += vim_strsize(
                                            &raw mut lead_start as *mut ::core::ffi::c_char,
                                        ) - vim_strsize(
                                            &raw mut lead_middle as *mut ::core::ffi::c_char,
                                        );
                                    }
                                    done = true_0;
                                    break;
                                }
                            }
                        }
                        if done == 0 {
                            if *theline.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '*' as ::core::ffi::c_int
                            {
                                amount += 1 as ::core::ffi::c_int;
                            } else {
                                amount = -1 as ::core::ffi::c_int;
                                lnum = cur_curpos.lnum - 1 as linenr_T;
                                while lnum > (*comment_pos).lnum {
                                    if linewhite(lnum) {
                                        lnum -= 1;
                                    } else {
                                        amount = get_indent_lnum(lnum);
                                        break;
                                    }
                                }
                                if amount == -1 as ::core::ffi::c_int {
                                    if (*curbuf.get()).b_ind_in_comment2 == 0 {
                                        start = ml_get((*comment_pos).lnum);
                                        look = start
                                            .offset((*comment_pos).col as isize)
                                            .offset(2 as ::core::ffi::c_int as isize);
                                        if *look as ::core::ffi::c_int != NUL {
                                            (*comment_pos).col =
                                                skipwhite(look).offset_from(start) as colnr_T;
                                        }
                                    }
                                    getvcol(
                                        curwin.get(),
                                        comment_pos,
                                        &raw mut col,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        ::core::ptr::null_mut::<colnr_T>(),
                                    );
                                    amount = col as ::core::ffi::c_int;
                                    if (*curbuf.get()).b_ind_in_comment2 != 0
                                        || *look as ::core::ffi::c_int == NUL
                                    {
                                        amount += (*curbuf.get()).b_ind_in_comment;
                                    }
                                }
                            }
                        }
                    } else if *skipwhite(theline) as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                        && {
                            trypos = find_match_char(b'[', (*curbuf.get()).b_ind_maxparen);
                            !trypos.is_null()
                        }
                    {
                        amount = get_indent_lnum((*trypos).lnum);
                    } else {
                        trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                        if !trypos.is_null()
                            && (*curbuf.get()).b_ind_java == 0 as ::core::ffi::c_int
                            || {
                                tryposBrace = find_start_brace();
                                !tryposBrace.is_null()
                            }
                            || !trypos.is_null()
                        {
                            if !trypos.is_null() && !tryposBrace.is_null() {
                                if if (*trypos).lnum != (*tryposBrace).lnum {
                                    ((*trypos).lnum < (*tryposBrace).lnum) as ::core::ffi::c_int
                                } else {
                                    ((*trypos).col < (*tryposBrace).col) as ::core::ffi::c_int
                                } != 0
                                {
                                    trypos = ::core::ptr::null_mut::<pos_T>();
                                } else {
                                    tryposBrace = ::core::ptr::null_mut::<pos_T>();
                                }
                            }
                            if !trypos.is_null() {
                                our_paren_pos = *trypos;
                                if *theline.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ')' as ::core::ffi::c_int
                                    && (*curbuf.get()).b_ind_paren_prev != 0
                                {
                                    amount = get_indent_lnum(
                                        (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                    );
                                } else {
                                    amount = -1 as ::core::ffi::c_int;
                                    lnum = cur_curpos.lnum - 1 as linenr_T;
                                    while lnum > our_paren_pos.lnum {
                                        l = skipwhite(ml_get(lnum));
                                        if !cin_nocode(l) {
                                            if !cin_ispreproc_cont(&mut l, &mut lnum, &mut amount) {
                                                (*curwin.get()).w_cursor.lnum = lnum;
                                                trypos = ind_find_start_CORS(None);
                                                if !trypos.is_null() {
                                                    lnum = (*trypos).lnum + 1 as linenr_T;
                                                } else {
                                                    trypos = find_match_paren(corr_ind_maxparen(
                                                        &raw mut cur_curpos,
                                                    ));
                                                    if !trypos.is_null()
                                                        && (*trypos).lnum == our_paren_pos.lnum
                                                        && (*trypos).col == our_paren_pos.col
                                                    {
                                                        amount = get_indent_lnum(lnum);
                                                        if *theline.offset(
                                                            0 as ::core::ffi::c_int as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            == ')' as ::core::ffi::c_int
                                                        {
                                                            if our_paren_pos.lnum != lnum
                                                                && cur_amount > amount
                                                            {
                                                                cur_amount = amount;
                                                            }
                                                            amount = -1 as ::core::ffi::c_int;
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        lnum -= 1;
                                    }
                                }
                                if amount == -1 as ::core::ffi::c_int {
                                    let mut ignore_paren_col: ::core::ffi::c_int =
                                        0 as ::core::ffi::c_int;
                                    let mut is_if_for_while: ::core::ffi::c_int =
                                        0 as ::core::ffi::c_int;
                                    if (*curbuf.get()).b_ind_if_for_while != 0 {
                                        let mut cursor_save: pos_T = (*curwin.get()).w_cursor;
                                        let mut outermost: pos_T = pos_T {
                                            lnum: 0,
                                            col: 0,
                                            coladd: 0,
                                        };
                                        let mut line: *mut ::core::ffi::c_char =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        trypos = &raw mut our_paren_pos;
                                        loop {
                                            outermost = *trypos;
                                            (*curwin.get()).w_cursor.lnum = outermost.lnum;
                                            (*curwin.get()).w_cursor.col = outermost.col;
                                            trypos =
                                                find_match_paren((*curbuf.get()).b_ind_maxparen);
                                            if !(!trypos.is_null()
                                                && (*trypos).lnum == outermost.lnum)
                                            {
                                                break;
                                            }
                                        }
                                        (*curwin.get()).w_cursor = cursor_save;
                                        line = ml_get(outermost.lnum);
                                        is_if_for_while = cin_is_if_for_while_before_offset(
                                            line,
                                            &mut outermost.col,
                                        )
                                            as ::core::ffi::c_int;
                                    }
                                    amount = skip_label(our_paren_pos.lnum, &raw mut look);
                                    look = skipwhite(look);
                                    if *look as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                                        let mut save_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
                                        let mut line_0: *mut ::core::ffi::c_char =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        let mut look_col: ::core::ffi::c_int = 0;
                                        (*curwin.get()).w_cursor.lnum = our_paren_pos.lnum;
                                        line_0 = get_cursor_line_ptr();
                                        look_col = look.offset_from(line_0) as ::core::ffi::c_int;
                                        (*curwin.get()).w_cursor.col =
                                            (look_col + 1 as ::core::ffi::c_int) as colnr_T;
                                        trypos = findmatchlimit(
                                            ::core::ptr::null_mut::<oparg_T>(),
                                            ')' as ::core::ffi::c_int,
                                            0 as ::core::ffi::c_int,
                                            (*curbuf.get()).b_ind_maxparen as int64_t,
                                        );
                                        if !trypos.is_null()
                                            && (*trypos).lnum == our_paren_pos.lnum
                                            && (*trypos).col < our_paren_pos.col
                                        {
                                            ignore_paren_col = (*trypos).col as ::core::ffi::c_int
                                                + 1 as ::core::ffi::c_int;
                                        }
                                        (*curwin.get()).w_cursor.lnum = save_lnum;
                                        look = ml_get(our_paren_pos.lnum).offset(look_col as isize);
                                    }
                                    if *theline.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == ')' as ::core::ffi::c_int
                                        || (*curbuf.get()).b_ind_unclosed == 0 as ::core::ffi::c_int
                                            && is_if_for_while == 0 as ::core::ffi::c_int
                                        || (*curbuf.get()).b_ind_unclosed_noignore == 0
                                            && *look as ::core::ffi::c_int
                                                == '(' as ::core::ffi::c_int
                                            && ignore_paren_col == 0 as ::core::ffi::c_int
                                    {
                                        if *theline.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            != ')' as ::core::ffi::c_int
                                        {
                                            cur_amount = MAXCOL as ::core::ffi::c_int;
                                            l = ml_get(our_paren_pos.lnum);
                                            if (*curbuf.get()).b_ind_unclosed_wrapped != 0
                                                && cin_ends_in(l, b"(")
                                            {
                                                n = 1 as ::core::ffi::c_int;
                                                col = 0 as ::core::ffi::c_int as colnr_T;
                                                while col < our_paren_pos.col {
                                                    match *l.offset(col as isize)
                                                        as ::core::ffi::c_int
                                                    {
                                                        40 | 123 => {
                                                            n += 1;
                                                        }
                                                        41 | 125 => {
                                                            if n > 1 as ::core::ffi::c_int {
                                                                n -= 1;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                    col += 1;
                                                }
                                                our_paren_pos.col =
                                                    0 as ::core::ffi::c_int as colnr_T;
                                                amount +=
                                                    n * (*curbuf.get()).b_ind_unclosed_wrapped;
                                            } else if (*curbuf.get()).b_ind_unclosed_whiteok != 0 {
                                                our_paren_pos.col += 1;
                                            } else {
                                                col = (our_paren_pos.col as ::core::ffi::c_int
                                                    + 1 as ::core::ffi::c_int)
                                                    as colnr_T;
                                                while ascii_iswhite(
                                                    *l.offset(col as isize) as ::core::ffi::c_int
                                                ) {
                                                    col += 1;
                                                }
                                                if *l.offset(col as isize) as ::core::ffi::c_int
                                                    != NUL
                                                {
                                                    our_paren_pos.col = col;
                                                } else {
                                                    our_paren_pos.col += 1;
                                                }
                                            }
                                        }
                                        if our_paren_pos.col > 0 as ::core::ffi::c_int {
                                            getvcol(
                                                curwin.get(),
                                                &raw mut our_paren_pos,
                                                &raw mut col,
                                                ::core::ptr::null_mut::<colnr_T>(),
                                                ::core::ptr::null_mut::<colnr_T>(),
                                            );
                                            if cur_amount > col {
                                                cur_amount = col as ::core::ffi::c_int;
                                            }
                                        }
                                    }
                                    if !(*theline.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == ')' as ::core::ffi::c_int
                                        && (*curbuf.get()).b_ind_matching_paren != 0)
                                    {
                                        if (*curbuf.get()).b_ind_unclosed == 0 as ::core::ffi::c_int
                                            && is_if_for_while == 0 as ::core::ffi::c_int
                                            || (*curbuf.get()).b_ind_unclosed_noignore == 0
                                                && *look as ::core::ffi::c_int
                                                    == '(' as ::core::ffi::c_int
                                                && ignore_paren_col == 0 as ::core::ffi::c_int
                                        {
                                            if cur_amount != MAXCOL as ::core::ffi::c_int {
                                                amount = cur_amount;
                                            }
                                        } else {
                                            col = our_paren_pos.col;
                                            while our_paren_pos.col > ignore_paren_col {
                                                our_paren_pos.col -= 1;
                                                match *ml_get_pos(&raw mut our_paren_pos)
                                                    as ::core::ffi::c_int
                                                {
                                                    40 => {
                                                        amount += (*curbuf.get()).b_ind_unclosed2;
                                                        col = our_paren_pos.col;
                                                    }
                                                    41 => {
                                                        amount -= (*curbuf.get()).b_ind_unclosed2;
                                                        col =
                                                            MAXCOL as ::core::ffi::c_int as colnr_T;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            if col == MAXCOL as ::core::ffi::c_int {
                                                amount += (*curbuf.get()).b_ind_unclosed;
                                            } else {
                                                (*curwin.get()).w_cursor.lnum = our_paren_pos.lnum;
                                                (*curwin.get()).w_cursor.col = col;
                                                if !find_match_paren_after_brace(
                                                    (*curbuf.get()).b_ind_maxparen,
                                                )
                                                .is_null()
                                                {
                                                    amount += (*curbuf.get()).b_ind_unclosed2;
                                                } else if is_if_for_while != 0 {
                                                    amount += (*curbuf.get()).b_ind_if_for_while;
                                                } else {
                                                    amount += (*curbuf.get()).b_ind_unclosed;
                                                }
                                            }
                                            if cur_amount < amount {
                                                amount = cur_amount;
                                            }
                                        }
                                    }
                                }
                                if cin_iscomment(theline) {
                                    amount += (*curbuf.get()).b_ind_comment;
                                }
                            } else {
                                tryposCopy = *tryposBrace;
                                tryposBrace = &raw mut tryposCopy;
                                trypos = tryposBrace;
                                ourscope = (*trypos).lnum;
                                start = ml_get(ourscope);
                                look = skipwhite(start);
                                if *look as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                                    getvcol(
                                        curwin.get(),
                                        trypos,
                                        &raw mut col,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        ::core::ptr::null_mut::<colnr_T>(),
                                    );
                                    amount = col as ::core::ffi::c_int;
                                    if *start as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                                        start_brace = BRACE_IN_COL0;
                                    } else {
                                        start_brace = BRACE_AT_START;
                                    }
                                } else {
                                    (*curwin.get()).w_cursor.lnum = ourscope;
                                    lnum = ourscope;
                                    if find_last_paren(start, b'(', b')') && {
                                        trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                                        !trypos.is_null()
                                    } {
                                        lnum = (*trypos).lnum;
                                    }
                                    if ((*curbuf.get()).b_ind_js != 0
                                        || (*curbuf.get()).b_ind_keep_case_label != 0)
                                        && cin_iscase(skipwhite(get_cursor_line_ptr()), false)
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        amount = get_indent();
                                    } else if (*curbuf.get()).b_ind_js != 0 {
                                        amount = get_indent_lnum(lnum);
                                    } else {
                                        amount = skip_label(lnum, &raw mut l);
                                    }
                                    start_brace = BRACE_AT_END;
                                }
                                let mut js_cur_has_key: bool = if (*curbuf.get()).b_ind_js != 0 {
                                    cin_has_js_key(theline) as ::core::ffi::c_int
                                } else {
                                    false_0
                                } != 0;
                                if *theline.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '}' as ::core::ffi::c_int
                                {
                                    amount += (*curbuf.get()).b_ind_close_extra;
                                } else {
                                    lookfor = LOOKFOR_INITIAL;
                                    if cin_iselse(theline) {
                                        lookfor = LOOKFOR_IF;
                                    } else if cin_iswhileofdo(theline, cur_curpos.lnum) {
                                        lookfor = LOOKFOR_DO;
                                    }
                                    if lookfor != LOOKFOR_INITIAL {
                                        (*curwin.get()).w_cursor.lnum = cur_curpos.lnum;
                                        if find_match(lookfor, ourscope) {
                                            amount = get_indent();
                                            break '_theend;
                                        }
                                    }
                                    if start_brace == BRACE_IN_COL0 {
                                        amount = (*curbuf.get()).b_ind_open_left_imag;
                                        lookfor_cpp_namespace = true;
                                    } else if start_brace == BRACE_AT_START
                                        && lookfor_cpp_namespace as ::core::ffi::c_int != 0
                                    {
                                        lookfor_cpp_namespace = true;
                                    } else if start_brace == BRACE_AT_END {
                                        amount += (*curbuf.get()).b_ind_open_imag;
                                        l = skipwhite(get_cursor_line_ptr());
                                        if cin_is_cpp_namespace(l) {
                                            amount += (*curbuf.get()).b_ind_cpp_namespace;
                                        } else if cin_is_cpp_extern_c(l) != 0 {
                                            amount += (*curbuf.get()).b_ind_cpp_extern_c;
                                        }
                                    } else {
                                        amount -= (*curbuf.get()).b_ind_open_extra;
                                        if amount < 0 as ::core::ffi::c_int {
                                            amount = 0 as ::core::ffi::c_int;
                                        }
                                    }
                                    lookfor_break = false_0;
                                    if cin_iscase(theline, false) {
                                        lookfor = LOOKFOR_CASE;
                                        amount += (*curbuf.get()).b_ind_case;
                                    } else if cin_isscopedecl(theline) {
                                        lookfor = LOOKFOR_SCOPEDECL;
                                        amount += (*curbuf.get()).b_ind_scopedecl;
                                    } else {
                                        if (*curbuf.get()).b_ind_case_break != 0
                                            && cin_isbreak(theline)
                                        {
                                            lookfor_break = true_0;
                                        }
                                        lookfor = LOOKFOR_INITIAL;
                                        amount += (*curbuf.get()).b_ind_level;
                                    }
                                    scope_amount = amount;
                                    whilelevel = 0 as ::core::ffi::c_int;
                                    (*curwin.get()).w_cursor = cur_curpos;
                                    's_2927: loop {
                                        (*curwin.get()).w_cursor.lnum -= 1;
                                        (*curwin.get()).w_cursor.col =
                                            0 as ::core::ffi::c_int as colnr_T;
                                        if (*curwin.get()).w_cursor.lnum <= ourscope {
                                            if lookfor == LOOKFOR_ENUM_OR_INIT {
                                                if (*curwin.get()).w_cursor.lnum == 0 as linenr_T
                                                    || (*curwin.get()).w_cursor.lnum
                                                        < ourscope
                                                            - (*curbuf.get()).b_ind_maxparen
                                                                as linenr_T
                                                {
                                                    if cont_amount > 0 as ::core::ffi::c_int {
                                                        amount = cont_amount;
                                                    } else if (*curbuf.get()).b_ind_js == 0 {
                                                        amount += ind_continuation;
                                                    }
                                                    break;
                                                } else {
                                                    trypos = ind_find_start_CORS(None);
                                                    if !trypos.is_null() {
                                                        (*curwin.get()).w_cursor.lnum =
                                                            (*trypos).lnum + 1 as linenr_T;
                                                        (*curwin.get()).w_cursor.col =
                                                            0 as ::core::ffi::c_int as colnr_T;
                                                    } else {
                                                        l = get_cursor_line_ptr();
                                                        if cin_ispreproc_cont(
                                                            &mut l,
                                                            &mut (*curwin.get()).w_cursor.lnum,
                                                            &mut amount,
                                                        ) {
                                                            continue;
                                                        }
                                                        if cin_nocode(l) {
                                                            continue;
                                                        }
                                                        terminated =
                                                            cin_isterminated(l, false, true);
                                                        if start_brace != BRACE_IN_COL0
                                                            || !cin_isfuncdecl(
                                                                Some(&mut l),
                                                                (*curwin.get()).w_cursor.lnum,
                                                                0 as linenr_T,
                                                            )
                                                        {
                                                            if terminated as ::core::ffi::c_int
                                                                == ',' as ::core::ffi::c_int
                                                            {
                                                                break;
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                != ';' as ::core::ffi::c_int
                                                                && cin_isinit()
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                            {
                                                                break;
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                == 0 as ::core::ffi::c_int
                                                                || terminated as ::core::ffi::c_int
                                                                    == '{' as ::core::ffi::c_int
                                                            {
                                                                continue;
                                                            }
                                                        }
                                                        if terminated as ::core::ffi::c_int
                                                            != ';' as ::core::ffi::c_int
                                                        {
                                                            trypos =
                                                                ::core::ptr::null_mut::<pos_T>();
                                                            if find_last_paren(l, b'(', b')') {
                                                                trypos = find_match_paren(
                                                                    (*curbuf.get()).b_ind_maxparen,
                                                                );
                                                            }
                                                            if trypos.is_null()
                                                                && find_last_paren(l, b'{', b'}')
                                                            {
                                                                trypos = find_start_brace();
                                                            }
                                                            if !trypos.is_null() {
                                                                (*curwin.get()).w_cursor.lnum =
                                                                    (*trypos).lnum + 1 as linenr_T;
                                                                (*curwin.get()).w_cursor.col = 0
                                                                    as ::core::ffi::c_int
                                                                    as colnr_T;
                                                                continue;
                                                            }
                                                        }
                                                        if cont_amount > 0 as ::core::ffi::c_int {
                                                            amount = cont_amount;
                                                        } else {
                                                            amount += ind_continuation;
                                                        }
                                                        break;
                                                    }
                                                }
                                            } else if lookfor == LOOKFOR_UNTERM {
                                                if cont_amount > 0 as ::core::ffi::c_int {
                                                    amount = cont_amount;
                                                } else {
                                                    amount += ind_continuation;
                                                }
                                                break;
                                            } else {
                                                if lookfor != LOOKFOR_TERM
                                                    && lookfor != LOOKFOR_CPP_BASECLASS
                                                    && lookfor != LOOKFOR_COMMA
                                                {
                                                    amount = scope_amount;
                                                    if *theline
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == '{' as ::core::ffi::c_int
                                                    {
                                                        amount += (*curbuf.get()).b_ind_open_extra;
                                                        added_to_amount =
                                                            (*curbuf.get()).b_ind_open_extra;
                                                    }
                                                }
                                                if !lookfor_cpp_namespace {
                                                    break;
                                                }
                                                if (*curwin.get()).w_cursor.lnum == ourscope {
                                                    continue;
                                                }
                                                if (*curwin.get()).w_cursor.lnum == 0 as linenr_T
                                                    || (*curwin.get()).w_cursor.lnum
                                                        < ourscope - FIND_NAMESPACE_LIM as linenr_T
                                                {
                                                    break;
                                                }
                                                trypos = ind_find_start_CORS(None);
                                                if !trypos.is_null() {
                                                    (*curwin.get()).w_cursor.lnum =
                                                        (*trypos).lnum + 1 as linenr_T;
                                                    (*curwin.get()).w_cursor.col =
                                                        0 as ::core::ffi::c_int as colnr_T;
                                                } else {
                                                    l = get_cursor_line_ptr();
                                                    if cin_ispreproc_cont(
                                                        &mut l,
                                                        &mut (*curwin.get()).w_cursor.lnum,
                                                        &mut amount,
                                                    ) {
                                                        continue;
                                                    }
                                                    if cin_is_cpp_namespace(l) {
                                                        amount += (*curbuf.get())
                                                            .b_ind_cpp_namespace
                                                            - added_to_amount;
                                                        break;
                                                    } else if cin_is_cpp_extern_c(l) != 0 {
                                                        amount += (*curbuf.get())
                                                            .b_ind_cpp_extern_c
                                                            - added_to_amount;
                                                        break;
                                                    } else if !cin_nocode(l) {
                                                        break;
                                                    }
                                                }
                                            }
                                        } else {
                                            trypos =
                                                ind_find_start_CORS(Some(&mut raw_string_start));
                                            if !trypos.is_null() {
                                                (*curwin.get()).w_cursor.lnum =
                                                    (*trypos).lnum + 1 as linenr_T;
                                                (*curwin.get()).w_cursor.col =
                                                    0 as ::core::ffi::c_int as colnr_T;
                                            } else {
                                                l = get_cursor_line_ptr();
                                                let mut iscase: bool = cin_iscase(l, false);
                                                if iscase as ::core::ffi::c_int != 0
                                                    || cin_isscopedecl(l)
                                                {
                                                    if lookfor == LOOKFOR_CPP_BASECLASS {
                                                        break;
                                                    }
                                                    if whilelevel > 0 as ::core::ffi::c_int {
                                                        continue;
                                                    }
                                                    if lookfor == LOOKFOR_UNTERM
                                                        || lookfor == LOOKFOR_ENUM_OR_INIT
                                                    {
                                                        if cont_amount > 0 as ::core::ffi::c_int {
                                                            amount = cont_amount;
                                                        } else {
                                                            amount += ind_continuation;
                                                        }
                                                        break;
                                                    } else if iscase as ::core::ffi::c_int != 0
                                                        && lookfor == LOOKFOR_CASE
                                                        || iscase as ::core::ffi::c_int != 0
                                                            && lookfor_break != 0
                                                        || !iscase && lookfor == LOOKFOR_SCOPEDECL
                                                    {
                                                        trypos = find_start_brace();
                                                        if !(trypos.is_null()
                                                            || (*trypos).lnum == ourscope)
                                                        {
                                                            continue;
                                                        }
                                                        amount = get_indent();
                                                        break;
                                                    } else {
                                                        n = get_indent_nolabel(
                                                            (*curwin.get()).w_cursor.lnum,
                                                        );
                                                        if lookfor == LOOKFOR_TERM {
                                                            if n != 0 {
                                                                amount = n;
                                                            }
                                                            if lookfor_break == 0 {
                                                                break;
                                                            }
                                                        }
                                                        if n != 0 {
                                                            amount = n;
                                                            l = after_label(get_cursor_line_ptr());
                                                            if !l.is_null()
                                                                && cin_is_cinword(l)
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                            {
                                                                if *theline.offset(
                                                                    0 as ::core::ffi::c_int
                                                                        as isize,
                                                                )
                                                                    as ::core::ffi::c_int
                                                                    == '{' as ::core::ffi::c_int
                                                                {
                                                                    amount += (*curbuf.get())
                                                                        .b_ind_open_extra;
                                                                } else {
                                                                    amount += (*curbuf.get())
                                                                        .b_ind_level
                                                                        + (*curbuf.get())
                                                                            .b_ind_no_brace;
                                                                }
                                                            }
                                                            break;
                                                        } else {
                                                            scope_amount = get_indent()
                                                                + (if iscase as ::core::ffi::c_int
                                                                    != 0
                                                                {
                                                                    (*curbuf.get()).b_ind_case_code
                                                                } else {
                                                                    (*curbuf.get())
                                                                        .b_ind_scopedecl_code
                                                                });
                                                            lookfor = if (*curbuf.get())
                                                                .b_ind_case_break
                                                                != 0
                                                            {
                                                                LOOKFOR_NOBREAK
                                                            } else {
                                                                LOOKFOR_ANY
                                                            };
                                                        }
                                                    }
                                                } else if lookfor == LOOKFOR_CASE
                                                    || lookfor == LOOKFOR_SCOPEDECL
                                                {
                                                    if find_last_paren(l, b'{', b'}') && {
                                                        trypos = find_start_brace();
                                                        !trypos.is_null()
                                                    } {
                                                        (*curwin.get()).w_cursor.lnum =
                                                            (*trypos).lnum + 1 as linenr_T;
                                                        (*curwin.get()).w_cursor.col =
                                                            0 as ::core::ffi::c_int as colnr_T;
                                                    }
                                                } else {
                                                    if (*curbuf.get()).b_ind_js == 0
                                                        && cin_islabel()
                                                    {
                                                        l = after_label(get_cursor_line_ptr());
                                                        if l.is_null() || cin_nocode(l) {
                                                            continue;
                                                        }
                                                    }
                                                    l = get_cursor_line_ptr();
                                                    if cin_ispreproc_cont(
                                                        &mut l,
                                                        &mut (*curwin.get()).w_cursor.lnum,
                                                        &mut amount,
                                                    ) || cin_nocode(l)
                                                    {
                                                        continue;
                                                    }
                                                    n = 0 as ::core::ffi::c_int;
                                                    if lookfor != LOOKFOR_TERM
                                                        && (*curbuf.get()).b_ind_cpp_baseclass
                                                            > 0 as ::core::ffi::c_int
                                                    {
                                                        n = cin_is_cpp_baseclass(
                                                            &raw mut cache_cpp_baseclass,
                                                        );
                                                        l = get_cursor_line_ptr();
                                                    }
                                                    if n != 0 {
                                                        if lookfor == LOOKFOR_UNTERM {
                                                            if cont_amount > 0 as ::core::ffi::c_int
                                                            {
                                                                amount = cont_amount;
                                                            } else {
                                                                amount += ind_continuation;
                                                            }
                                                            break;
                                                        } else if *theline.offset(
                                                            0 as ::core::ffi::c_int as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            == '{' as ::core::ffi::c_int
                                                        {
                                                            lookfor = LOOKFOR_UNTERM;
                                                            ind_continuation =
                                                                0 as ::core::ffi::c_int;
                                                        } else {
                                                            amount = get_baseclass_amount(
                                                                cache_cpp_baseclass.lpos.col
                                                                    as ::core::ffi::c_int,
                                                            );
                                                            break;
                                                        }
                                                    } else if lookfor == LOOKFOR_CPP_BASECLASS {
                                                        if cin_isterminated(l, true, false) != 0 {
                                                            break;
                                                        }
                                                    } else {
                                                        terminated =
                                                            cin_isterminated(l, false, true);
                                                        if js_cur_has_key {
                                                            js_cur_has_key = false;
                                                            if (*curbuf.get()).b_ind_js != 0
                                                                && terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                            {
                                                                lookfor = LOOKFOR_JS_KEY;
                                                            }
                                                        }
                                                        if lookfor == LOOKFOR_JS_KEY
                                                            && cin_has_js_key(l)
                                                                as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            amount = get_indent();
                                                            break;
                                                        } else {
                                                            if lookfor == LOOKFOR_COMMA {
                                                                if !tryposBrace.is_null()
                                                                    && (*tryposBrace).lnum
                                                                        >= (*curwin.get())
                                                                            .w_cursor
                                                                            .lnum
                                                                {
                                                                    break;
                                                                }
                                                                if terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                                {
                                                                    break;
                                                                } else {
                                                                    amount = get_indent();
                                                                    if (*curwin.get()).w_cursor.lnum
                                                                        - 1 as linenr_T
                                                                        == ourscope
                                                                    {
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                == 0 as ::core::ffi::c_int
                                                                || lookfor != LOOKFOR_UNTERM
                                                                    && terminated
                                                                        as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                            {
                                                                if lookfor != LOOKFOR_ENUM_OR_INIT
                                                                && (*skipwhite(l)
                                                                    as ::core::ffi::c_int
                                                                    == '[' as ::core::ffi::c_int
                                                                    || *l.offset(
                                                                        strlen(l).wrapping_sub(
                                                                            1 as size_t,
                                                                        )
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '['
                                                                            as ::core::ffi::c_int)
                                                            {
                                                                amount += ind_continuation;
                                                            }
                                                                find_last_paren(l, b'(', b')');
                                                                trypos = find_match_paren(
                                                                    corr_ind_maxparen(
                                                                        &raw mut cur_curpos,
                                                                    ),
                                                                );
                                                                if !trypos.is_null()
                                                                    && ((*trypos).lnum
                                                                        < (*tryposBrace).lnum
                                                                        || (*trypos).lnum
                                                                            == (*tryposBrace).lnum
                                                                            && (*trypos).col
                                                                                < (*tryposBrace)
                                                                                    .col)
                                                                {
                                                                    trypos = ::core::ptr::null_mut::<
                                                                        pos_T,
                                                                    >(
                                                                    );
                                                                }
                                                                l = get_cursor_line_ptr();
                                                                if trypos.is_null()
                                                                    && terminated
                                                                        as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                {
                                                                    if find_last_paren(
                                                                        l, b'{', b'}',
                                                                    ) {
                                                                        trypos = find_start_brace();
                                                                    }
                                                                    l = get_cursor_line_ptr();
                                                                }
                                                                if !trypos.is_null() {
                                                                    (*curwin.get()).w_cursor =
                                                                        *trypos;
                                                                    l = get_cursor_line_ptr();
                                                                    if cin_iscase(l, false)
                                                                        as ::core::ffi::c_int
                                                                        != 0
                                                                        || cin_isscopedecl(l)
                                                                            as ::core::ffi::c_int
                                                                            != 0
                                                                    {
                                                                        (*curwin.get())
                                                                            .w_cursor
                                                                            .lnum += 1;
                                                                        (*curwin.get())
                                                                            .w_cursor
                                                                            .col = 0
                                                                            as ::core::ffi::c_int
                                                                            as colnr_T;
                                                                        continue;
                                                                    }
                                                                }
                                                                if terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                                {
                                                                    while (*curwin.get())
                                                                        .w_cursor
                                                                        .lnum
                                                                        > 1 as linenr_T
                                                                    {
                                                                        l = ml_get(
                                                                            (*curwin.get())
                                                                                .w_cursor
                                                                                .lnum
                                                                                - 1 as linenr_T,
                                                                        );
                                                                        if *l as ::core::ffi::c_int == NUL
                                                                        || *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                                                            as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
                                                                    {
                                                                        break;
                                                                    }
                                                                        (*curwin.get())
                                                                            .w_cursor
                                                                            .lnum -= 1;
                                                                        (*curwin.get())
                                                                            .w_cursor
                                                                            .col = 0
                                                                            as ::core::ffi::c_int
                                                                            as colnr_T;
                                                                    }
                                                                    l = get_cursor_line_ptr();
                                                                }
                                                                if (*curbuf.get()).b_ind_js != 0 {
                                                                    cur_amount = get_indent();
                                                                } else {
                                                                    cur_amount = skip_label(
                                                                        (*curwin.get())
                                                                            .w_cursor
                                                                            .lnum,
                                                                        &raw mut l,
                                                                    );
                                                                }
                                                                if terminated as ::core::ffi::c_int
                                                                    != ',' as ::core::ffi::c_int
                                                                    && lookfor != LOOKFOR_TERM
                                                                    && *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                {
                                                                    amount = cur_amount;
                                                                    if *skipwhite(l)
                                                                        as ::core::ffi::c_int
                                                                        != '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    if !((*curbuf.get())
                                                                        .b_ind_cpp_baseclass
                                                                        != 0
                                                                        && (*curbuf.get()).b_ind_js
                                                                            == 0)
                                                                    {
                                                                        break;
                                                                    }
                                                                    lookfor = LOOKFOR_CPP_BASECLASS;
                                                                } else if cin_is_cinword(l)
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                    || cin_iselse(skipwhite(l))
                                                                {
                                                                    if lookfor == LOOKFOR_UNTERM
                                                                        || lookfor
                                                                            == LOOKFOR_ENUM_OR_INIT
                                                                    {
                                                                        if cont_amount
                                                                        > 0 as ::core::ffi::c_int
                                                                    {
                                                                        amount = cont_amount;
                                                                    } else {
                                                                        amount += ind_continuation;
                                                                    }
                                                                        break;
                                                                    } else {
                                                                        amount = cur_amount;
                                                                        if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                        if lookfor != LOOKFOR_TERM {
                                                                            amount += (*curbuf
                                                                                .get())
                                                                            .b_ind_level
                                                                                + (*curbuf.get())
                                                                                    .b_ind_no_brace;
                                                                            break;
                                                                        } else {
                                                                            l = skipwhite(
                                                                                get_cursor_line_ptr(
                                                                                ),
                                                                            );
                                                                            if cin_isdo(l) {
                                                                                if whilelevel == 0 as ::core::ffi::c_int {
                                                                                break;
                                                                            }
                                                                                whilelevel -= 1;
                                                                            }
                                                                            if !(cin_iselse(l)
                                                                            && whilelevel == 0 as ::core::ffi::c_int)
                                                                        {
                                                                            continue;
                                                                        }
                                                                            if *l as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                                                                            (*curwin.get()).w_cursor.col = (l
                                                                                .offset_from(get_cursor_line_ptr()) as ::core::ffi::c_int
                                                                                + 1 as ::core::ffi::c_int) as colnr_T;
                                                                        }
                                                                            trypos =
                                                                                find_start_brace();
                                                                            if trypos.is_null()
                                                                                || !find_match(
                                                                                    LOOKFOR_IF,
                                                                                    (*trypos).lnum,
                                                                                )
                                                                            {
                                                                                break;
                                                                            }
                                                                        }
                                                                    }
                                                                } else if lookfor == LOOKFOR_UNTERM
                                                                {
                                                                    if terminated
                                                                        as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                    {
                                                                        amount += ind_continuation;
                                                                    }
                                                                    break;
                                                                } else if lookfor
                                                                    == LOOKFOR_ENUM_OR_INIT
                                                                {
                                                                    if terminated
                                                                        as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                    {
                                                                        if (*curbuf.get())
                                                                        .b_ind_cpp_baseclass
                                                                        == 0 as ::core::ffi::c_int
                                                                    {
                                                                        break;
                                                                    }
                                                                        lookfor =
                                                                            LOOKFOR_CPP_BASECLASS;
                                                                    } else if amount > cur_amount {
                                                                        amount = cur_amount;
                                                                    }
                                                                } else {
                                                                    l = get_cursor_line_ptr();
                                                                    amount = cur_amount;
                                                                    n = strlen(l)
                                                                        as ::core::ffi::c_int;
                                                                    if (*curbuf.get()).b_ind_js != 0
                                                                    && terminated as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                    && (*skipwhite(l) as ::core::ffi::c_int
                                                                        == ']' as ::core::ffi::c_int
                                                                        || n >= 2 as ::core::ffi::c_int
                                                                            && *l.offset((n - 2 as ::core::ffi::c_int) as isize)
                                                                                as ::core::ffi::c_int == ']' as ::core::ffi::c_int)
                                                                {
                                                                    break;
                                                                }
                                                                    if lookfor == LOOKFOR_INITIAL
                                                                    && terminated
                                                                        as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                {
                                                                    if (*curbuf.get()).b_ind_js != 0
                                                                    {
                                                                        if cin_iscomment(skipwhite(
                                                                            l,
                                                                        ))
                                                                        {
                                                                            break;
                                                                        }
                                                                        lookfor = LOOKFOR_COMMA;
                                                                        trypos = find_match_char(b'[',
                                                                            (*curbuf.get()).b_ind_maxparen,
                                                                        );
                                                                        if trypos.is_null() {
                                                                            continue;
                                                                        }
                                                                        if (*trypos).lnum
                                                                            == (*curwin.get())
                                                                                .w_cursor
                                                                                .lnum
                                                                                - 1 as linenr_T
                                                                        {
                                                                            break;
                                                                        }
                                                                        ourscope = (*trypos).lnum;
                                                                    } else {
                                                                        lookfor =
                                                                            LOOKFOR_ENUM_OR_INIT;
                                                                        cont_amount =
                                                                            cin_first_id_amount();
                                                                    }
                                                                } else {
                                                                    if lookfor == LOOKFOR_INITIAL
                                                                        && *l as ::core::ffi::c_int != NUL
                                                                        && *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                                                            as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                                                    {
                                                                        cont_amount = cin_get_equal_amount((*curwin.get()).w_cursor.lnum);
                                                                    }
                                                                    if lookfor != LOOKFOR_TERM
                                                                        && lookfor != LOOKFOR_JS_KEY
                                                                        && lookfor != LOOKFOR_COMMA
                                                                        && raw_string_start
                                                                            != (*curwin.get())
                                                                                .w_cursor
                                                                                .lnum
                                                                    {
                                                                        lookfor = LOOKFOR_UNTERM;
                                                                    }
                                                                }
                                                                }
                                                            } else if cin_iswhileofdo_end(
                                                                terminated,
                                                            ) {
                                                                if lookfor == LOOKFOR_UNTERM
                                                                    || lookfor
                                                                        == LOOKFOR_ENUM_OR_INIT
                                                                {
                                                                    if cont_amount
                                                                        > 0 as ::core::ffi::c_int
                                                                    {
                                                                        amount = cont_amount;
                                                                    } else {
                                                                        amount += ind_continuation;
                                                                    }
                                                                    break;
                                                                } else {
                                                                    if whilelevel
                                                                        == 0 as ::core::ffi::c_int
                                                                    {
                                                                        lookfor = LOOKFOR_TERM;
                                                                        amount = get_indent();
                                                                        if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    }
                                                                    whilelevel += 1;
                                                                }
                                                            } else if lookfor == LOOKFOR_NOBREAK
                                                                && cin_isbreak(skipwhite(
                                                                    get_cursor_line_ptr(),
                                                                ))
                                                            {
                                                                lookfor = LOOKFOR_ANY;
                                                            } else {
                                                                if whilelevel
                                                                    > 0 as ::core::ffi::c_int
                                                                {
                                                                    l = cin_skipcomment(
                                                                        get_cursor_line_ptr(),
                                                                    );
                                                                    if cin_isdo(l) {
                                                                        amount = get_indent();
                                                                        whilelevel -= 1;
                                                                        continue;
                                                                    }
                                                                }
                                                                if lookfor == LOOKFOR_UNTERM
                                                                    || lookfor
                                                                        == LOOKFOR_ENUM_OR_INIT
                                                                {
                                                                    if cont_amount
                                                                        > 0 as ::core::ffi::c_int
                                                                    {
                                                                        amount = cont_amount;
                                                                    } else {
                                                                        amount += ind_continuation;
                                                                    }
                                                                    break;
                                                                } else if lookfor == LOOKFOR_TERM {
                                                                    if lookfor_break == 0
                                                                    && whilelevel
                                                                        == 0 as ::core::ffi::c_int
                                                                {
                                                                    break;
                                                                }
                                                                } else {
                                                                    loop {
                                                                        l = get_cursor_line_ptr();
                                                                        if find_last_paren(
                                                                            l, b'(', b')',
                                                                        ) && {
                                                                            trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                                                                            !trypos.is_null()
                                                                        } {
                                                                            (*curwin.get())
                                                                                .w_cursor = *trypos;
                                                                            l = get_cursor_line_ptr(
                                                                            );
                                                                            if cin_iscase(l, false)
                                                                                || cin_isscopedecl(
                                                                                    l,
                                                                                )
                                                                            {
                                                                                (*curwin.get())
                                                                                    .w_cursor
                                                                                    .lnum += 1;
                                                                                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                                                                break;
                                                                            }
                                                                        }
                                                                        iscase = (*curbuf.get())
                                                                        .b_ind_keep_case_label
                                                                        != 0
                                                                        && cin_iscase(
                                                                            l,
                                                                            false,
                                                                        )
                                                                            as ::core::ffi::c_int
                                                                            != 0;
                                                                        amount = skip_label(
                                                                            (*curwin.get())
                                                                                .w_cursor
                                                                                .lnum,
                                                                            &raw mut l,
                                                                        );
                                                                        if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                        l = skipwhite(l);
                                                                        if *l as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount -= (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                        lookfor = if iscase
                                                                            as ::core::ffi::c_int
                                                                            != 0
                                                                        {
                                                                            LOOKFOR_ANY
                                                                        } else {
                                                                            LOOKFOR_TERM
                                                                        };
                                                                        if lookfor == LOOKFOR_TERM
                                                                        && *l as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                                                                        && cin_iselse(l)
                                                                        && whilelevel == 0 as ::core::ffi::c_int
                                                                    {
                                                                        trypos = find_start_brace();
                                                                        if trypos.is_null()
                                                                            || !find_match(LOOKFOR_IF, (*trypos).lnum)
                                                                        {
                                                                            break 's_2927;
                                                                        } else {
                                                                            break;
                                                                        }
                                                                    } else {
                                                                        l = get_cursor_line_ptr();
                                                                        if !(find_last_paren(
                                                                            l,
                                                                            b'{',
                                                                            b'}',
                                                                        )
                                                                            && {
                                                                                trypos = find_start_brace();
                                                                                !trypos.is_null()
                                                                            })
                                                                        {
                                                                            break;
                                                                        }
                                                                        (*curwin.get()).w_cursor = *trypos;
                                                                        l = cin_skipcomment(get_cursor_line_ptr());
                                                                        if *l as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                                                                            || !cin_iselse(l)
                                                                        {
                                                                            continue;
                                                                        }
                                                                        (*curwin.get()).w_cursor.lnum += 1;
                                                                        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                                                        break;
                                                                    }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if cin_iscomment(theline) {
                                amount += (*curbuf.get()).b_ind_comment;
                            }
                            if (*curbuf.get()).b_ind_jump_label > 0 as ::core::ffi::c_int
                                && original_line_islabel != 0
                            {
                                amount -= (*curbuf.get()).b_ind_jump_label;
                            }
                        } else if *theline.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == '{' as ::core::ffi::c_int
                        {
                            amount = (*curbuf.get()).b_ind_first_open;
                        } else if cur_curpos.lnum < (*curbuf.get()).b_ml.ml_line_count
                            && !cin_nocode(theline)
                            && vim_strchr(theline, '{' as ::core::ffi::c_int).is_null()
                            && vim_strchr(theline, '}' as ::core::ffi::c_int).is_null()
                            && !cin_ends_in(theline, b":")
                            && !cin_ends_in(theline, b",")
                            && cin_isfuncdecl(
                                None,
                                cur_curpos.lnum + 1 as linenr_T,
                                cur_curpos.lnum + 1 as linenr_T,
                            )
                            && cin_isterminated(theline, false, true) == 0
                        {
                            amount = (*curbuf.get()).b_ind_func_type;
                        } else {
                            amount = 0 as ::core::ffi::c_int;
                            (*curwin.get()).w_cursor = cur_curpos;
                            while (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                                (*curwin.get()).w_cursor.lnum -= 1;
                                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                l = get_cursor_line_ptr();
                                trypos = ind_find_start_CORS(None);
                                if !trypos.is_null() {
                                    (*curwin.get()).w_cursor.lnum = (*trypos).lnum + 1 as linenr_T;
                                    (*curwin.get()).w_cursor.col =
                                        0 as ::core::ffi::c_int as colnr_T;
                                } else {
                                    n = 0 as ::core::ffi::c_int;
                                    if (*curbuf.get()).b_ind_cpp_baseclass
                                        != 0 as ::core::ffi::c_int
                                    {
                                        n = cin_is_cpp_baseclass(&raw mut cache_cpp_baseclass);
                                        l = get_cursor_line_ptr();
                                    }
                                    if n != 0 {
                                        amount = get_baseclass_amount(
                                            cache_cpp_baseclass.lpos.col as ::core::ffi::c_int,
                                        );
                                        break;
                                    } else {
                                        if cin_ispreproc_cont(
                                            &mut l,
                                            &mut (*curwin.get()).w_cursor.lnum,
                                            &mut amount,
                                        ) {
                                            continue;
                                        }
                                        if cin_nocode(l) {
                                            continue;
                                        }
                                        if cin_ends_in(l, b",")
                                            || *l as ::core::ffi::c_int != NUL && {
                                                n = *l
                                                    .offset(strlen(l).wrapping_sub(1 as size_t)
                                                        as isize)
                                                    as uint8_t
                                                    as ::core::ffi::c_int;
                                                n == '\\' as ::core::ffi::c_int
                                            }
                                        {
                                            if find_last_paren(l, b'(', b')') && {
                                                trypos = find_match_paren(
                                                    (*curbuf.get()).b_ind_maxparen,
                                                );
                                                !trypos.is_null()
                                            } {
                                                (*curwin.get()).w_cursor = *trypos;
                                            }
                                            while n == 0 as ::core::ffi::c_int
                                                && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                                            {
                                                l = ml_get(
                                                    (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                                );
                                                if *l as ::core::ffi::c_int == NUL
                                                    || *l
                                                        .offset(strlen(l).wrapping_sub(1 as size_t)
                                                            as isize)
                                                        as ::core::ffi::c_int
                                                        != '\\' as ::core::ffi::c_int
                                                {
                                                    break;
                                                }
                                                (*curwin.get()).w_cursor.lnum -= 1;
                                                (*curwin.get()).w_cursor.col =
                                                    0 as ::core::ffi::c_int as colnr_T;
                                            }
                                            amount = get_indent();
                                            if amount == 0 as ::core::ffi::c_int {
                                                amount = cin_first_id_amount();
                                            }
                                            if amount == 0 as ::core::ffi::c_int {
                                                amount = ind_continuation;
                                            }
                                            break;
                                        } else {
                                            if cin_isfuncdecl(None, cur_curpos.lnum, 0 as linenr_T)
                                            {
                                                break;
                                            }
                                            l = get_cursor_line_ptr();
                                            if *skipwhite(l) as ::core::ffi::c_int
                                                == '}' as ::core::ffi::c_int
                                            {
                                                break;
                                            }
                                            if cin_ends_in(l, b"};") {
                                                break;
                                            }
                                            if cin_ends_in(l, b"[") {
                                                amount = get_indent() + ind_continuation;
                                                break;
                                            } else {
                                                look = skipwhite(l);
                                                if *look as ::core::ffi::c_int
                                                    == ';' as ::core::ffi::c_int
                                                    && cin_nocode(
                                                        look.offset(
                                                            1 as ::core::ffi::c_int as isize,
                                                        ),
                                                    )
                                                {
                                                    let mut curpos_save: pos_T =
                                                        (*curwin.get()).w_cursor;
                                                    while (*curwin.get()).w_cursor.lnum
                                                        > 1 as linenr_T
                                                    {
                                                        (*curwin.get()).w_cursor.lnum -= 1;
                                                        look =
                                                            ml_get((*curwin.get()).w_cursor.lnum);
                                                        if !(cin_nocode(look)
                                                            || cin_ispreproc_cont(
                                                                &mut look,
                                                                &mut (*curwin.get()).w_cursor.lnum,
                                                                &mut amount,
                                                            ))
                                                        {
                                                            break;
                                                        }
                                                    }
                                                    if (*curwin.get()).w_cursor.lnum > 0 as linenr_T
                                                        && cin_ends_in(look, b"}")
                                                    {
                                                        break;
                                                    }
                                                    (*curwin.get()).w_cursor = curpos_save;
                                                }
                                                if cin_isfuncdecl(
                                                    Some(&mut l),
                                                    (*curwin.get()).w_cursor.lnum,
                                                    0 as linenr_T,
                                                ) {
                                                    amount = (*curbuf.get()).b_ind_param;
                                                    break;
                                                } else {
                                                    if cin_ends_in(l, b";") {
                                                        l = ml_get(
                                                            (*curwin.get()).w_cursor.lnum
                                                                - 1 as linenr_T,
                                                        );
                                                        if cin_ends_in(l, b",")
                                                            || *l as ::core::ffi::c_int != NUL
                                                                && *l.offset(
                                                                    strlen(l)
                                                                        .wrapping_sub(1 as size_t)
                                                                        as isize,
                                                                )
                                                                    as ::core::ffi::c_int
                                                                    == '\\' as ::core::ffi::c_int
                                                        {
                                                            break;
                                                        }
                                                        l = get_cursor_line_ptr();
                                                    }
                                                    find_last_paren(l, b'(', b')');
                                                    trypos = find_match_paren(
                                                        (*curbuf.get()).b_ind_maxparen,
                                                    );
                                                    if !trypos.is_null() {
                                                        (*curwin.get()).w_cursor = *trypos;
                                                    }
                                                    amount = get_indent();
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if cin_iscomment(theline) {
                                amount += (*curbuf.get()).b_ind_comment;
                            }
                            if cur_curpos.lnum > 1 as linenr_T {
                                l = ml_get(cur_curpos.lnum - 1 as linenr_T);
                                if *l as ::core::ffi::c_int != NUL
                                    && *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                        as ::core::ffi::c_int
                                        == '\\' as ::core::ffi::c_int
                                {
                                    cur_amount =
                                        cin_get_equal_amount(cur_curpos.lnum - 1 as linenr_T);
                                    if cur_amount > 0 as ::core::ffi::c_int {
                                        amount = cur_amount;
                                    } else if cur_amount == 0 as ::core::ffi::c_int {
                                        amount += ind_continuation;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if amount < 0 as ::core::ffi::c_int {
                amount = 0 as ::core::ffi::c_int;
            }
        }
        (*curwin.get()).w_cursor = cur_curpos;
        xfree(linecopy as *mut ::core::ffi::c_void);
        return amount;
    }
}
