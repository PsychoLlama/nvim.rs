//! Matching brackets.
//!
//! [`findmatchlimit`] is the walk `%` and its neighbours share: from a
//! position it looks for the other half of a `'matchpairs'` pair, a `#if`
//! /`#endif` triple, or the end of a comment or string, skipping anything
//! that a comment, a string or a backslash makes not count
//! ([`check_linecomment`], [`find_rawstring_end`], [`check_prevcol`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn findmatch(
    mut oap: *mut oparg_T,
    mut initc: ::core::ffi::c_int,
) -> *mut pos_T {
    unsafe {
        return findmatchlimit(oap, initc, 0 as ::core::ffi::c_int, 0 as int64_t);
    }
}

pub(crate) unsafe extern "C" fn check_prevcol(
    mut linep: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut ch: ::core::ffi::c_int,
    mut prevcol: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        col -= 1;
        if col > 0 as ::core::ffi::c_int {
            col -= utf_head_off(linep, linep.offset(col as isize));
        }
        if !prevcol.is_null() {
            *prevcol = col;
        }
        return col >= 0 as ::core::ffi::c_int
            && *linep.offset(col as isize) as uint8_t as ::core::ffi::c_int == ch;
    }
}

pub(crate) unsafe extern "C" fn find_rawstring_end(
    mut linep: *mut ::core::ffi::c_char,
    mut startpos: *mut pos_T,
    mut endpos: *mut pos_T,
) -> bool {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut lnum: linenr_T = 0;
        p = linep
            .offset((*startpos).col as isize)
            .offset(1 as ::core::ffi::c_int as isize);
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '(' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        let mut delim_len: size_t =
            (p.offset_from(linep) - (*startpos).col as isize - 1 as isize) as size_t;
        let mut delim_copy: *mut ::core::ffi::c_char = xmemdupz(
            linep
                .offset((*startpos).col as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            delim_len,
        ) as *mut ::core::ffi::c_char;
        let mut found: bool = false_0 != 0;
        lnum = (*startpos).lnum;
        while lnum <= (*endpos).lnum {
            let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
            p = line.offset(
                (if lnum == (*startpos).lnum {
                    (*startpos).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as isize,
            );
            while *p != 0 {
                if lnum == (*endpos).lnum && p.offset_from(line) as colnr_T >= (*endpos).col {
                    break;
                }
                if *p as ::core::ffi::c_int == ')' as ::core::ffi::c_int
                    && strncmp(
                        delim_copy,
                        p.offset(1 as ::core::ffi::c_int as isize),
                        delim_len,
                    ) == 0 as ::core::ffi::c_int
                    && *p.offset(delim_len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    found = true_0 != 0;
                    break;
                } else {
                    p = p.offset(1);
                }
            }
            if found {
                break;
            }
            lnum += 1;
        }
        xfree(delim_copy as *mut ::core::ffi::c_void);
        return found;
    }
}

pub(crate) unsafe extern "C" fn find_mps_values(
    mut initc: *mut ::core::ffi::c_int,
    mut findc: *mut ::core::ffi::c_int,
    mut backwards: *mut bool,
    mut switchit: bool,
) {
    unsafe {
        let mut ptr: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_mps;
        while *ptr as ::core::ffi::c_int != NUL {
            if utf_ptr2char(ptr) == *initc {
                if switchit {
                    *findc = *initc;
                    *initc = utf_ptr2char(
                        ptr.offset(utfc_ptr2len(ptr) as isize)
                            .offset(1 as ::core::ffi::c_int as isize),
                    );
                    *backwards = true_0 != 0;
                } else {
                    *findc = utf_ptr2char(
                        ptr.offset(utfc_ptr2len(ptr) as isize)
                            .offset(1 as ::core::ffi::c_int as isize),
                    );
                    *backwards = false_0 != 0;
                }
                return;
            }
            let mut prev: *mut ::core::ffi::c_char = ptr;
            ptr = ptr.offset((utfc_ptr2len(ptr) + 1 as ::core::ffi::c_int) as isize);
            if utf_ptr2char(ptr) == *initc {
                if switchit {
                    *findc = *initc;
                    *initc = utf_ptr2char(prev);
                    *backwards = false_0 != 0;
                } else {
                    *findc = utf_ptr2char(prev);
                    *backwards = true_0 != 0;
                }
                return;
            }
            ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
            if *ptr as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                ptr = ptr.offset(1);
            }
        }
    }
}

pub unsafe extern "C" fn findmatchlimit(
    mut oap: *mut oparg_T,
    mut initc: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut maxtravel: int64_t,
) -> *mut pos_T {
    unsafe {
        static pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        });
        let mut findc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut backwards: bool = false_0 != 0;
        let mut raw_string: bool = false_0 != 0;
        let mut inquote: bool = false_0 != 0;
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut hash_dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut comment_dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut traveled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ignore_cend: bool = false_0 != 0;
        let mut match_escaped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut dir: ::core::ffi::c_int = 0;
        let mut comment_col: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
        let mut lispcomm: bool = false_0 != 0;
        let mut lisp: bool = (*curbuf.get()).b_p_lisp != 0;
        pos.set((*curwin.get()).w_cursor);
        (*pos.ptr()).coladd = 0 as ::core::ffi::c_int as colnr_T;
        let mut linep: *mut ::core::ffi::c_char = ml_get((*pos.ptr()).lnum);
        let mut cpo_match: bool = !vim_strchr(p_cpo.get(), CPO_MATCH).is_null();
        let mut cpo_bsl: bool = !vim_strchr(p_cpo.get(), CPO_MATCHBSL).is_null();
        if flags & FM_BACKWARD as ::core::ffi::c_int != 0 {
            dir = BACKWARD as ::core::ffi::c_int;
        } else if flags & FM_FORWARD as ::core::ffi::c_int != 0 {
            dir = FORWARD as ::core::ffi::c_int;
        } else {
            dir = 0 as ::core::ffi::c_int;
        }
        if initc == '/' as ::core::ffi::c_int
            || initc == '*' as ::core::ffi::c_int
            || initc == 'R' as ::core::ffi::c_int
        {
            comment_dir = dir;
            if initc == '/' as ::core::ffi::c_int {
                ignore_cend = true_0 != 0;
            }
            backwards = if dir == FORWARD as ::core::ffi::c_int {
                false_0
            } else {
                true_0
            } != 0;
            raw_string = initc == 'R' as ::core::ffi::c_int;
            initc = NUL;
        } else if initc != '#' as ::core::ffi::c_int && initc != NUL {
            find_mps_values(
                &raw mut initc,
                &raw mut findc,
                &raw mut backwards,
                true_0 != 0,
            );
            if dir != 0 {
                backwards = if dir == FORWARD as ::core::ffi::c_int {
                    false_0
                } else {
                    true_0
                } != 0;
            }
            if findc == NUL {
                return ::core::ptr::null_mut::<pos_T>();
            }
        } else {
            if initc == '#' as ::core::ffi::c_int {
                hash_dir = dir;
            } else {
                if !cpo_match {
                    ptr = skipwhite(linep);
                    if *ptr as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                        && (*pos.ptr()).col <= ptr.offset_from(linep) as colnr_T
                    {
                        ptr = skipwhite(ptr.offset(1 as ::core::ffi::c_int as isize));
                        if strncmp(
                            ptr,
                            b"if\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                ptr,
                                b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                ptr,
                                b"el\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            hash_dir = 1 as ::core::ffi::c_int;
                        }
                    } else if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                    {
                        if *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                        {
                            comment_dir = FORWARD as ::core::ffi::c_int;
                            backwards = false_0 != 0;
                            (*pos.ptr()).col += 1;
                        } else if (*pos.ptr()).col > 0 as ::core::ffi::c_int
                            && *linep.offset(
                                ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as ::core::ffi::c_int
                                == '*' as ::core::ffi::c_int
                        {
                            comment_dir = BACKWARD as ::core::ffi::c_int;
                            backwards = true_0 != 0;
                            (*pos.ptr()).col -= 1;
                        }
                    } else if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                    {
                        if *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                        {
                            comment_dir = BACKWARD as ::core::ffi::c_int;
                            backwards = true_0 != 0;
                        } else if (*pos.ptr()).col > 0 as ::core::ffi::c_int
                            && *linep.offset(
                                ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as ::core::ffi::c_int
                                == '/' as ::core::ffi::c_int
                        {
                            comment_dir = FORWARD as ::core::ffi::c_int;
                            backwards = false_0 != 0;
                        }
                    }
                }
                if hash_dir == 0 && comment_dir == 0 {
                    if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int == NUL
                        && (*pos.ptr()).col != 0
                    {
                        (*pos.ptr()).col -= 1;
                    }
                    loop {
                        initc = utf_ptr2char(linep.offset((*pos.ptr()).col as isize));
                        if initc == NUL {
                            break;
                        }
                        find_mps_values(
                            &raw mut initc,
                            &raw mut findc,
                            &raw mut backwards,
                            false_0 != 0,
                        );
                        if findc != 0 {
                            break;
                        }
                        (*pos.ptr()).col += utfc_ptr2len(linep.offset((*pos.ptr()).col as isize));
                    }
                    if findc == 0 {
                        if !cpo_match
                            && *skipwhite(linep) as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                        {
                            hash_dir = 1 as ::core::ffi::c_int;
                        } else {
                            return ::core::ptr::null_mut::<pos_T>();
                        }
                    } else if !cpo_bsl {
                        let mut bslcnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut col: ::core::ffi::c_int = (*pos.ptr()).col as ::core::ffi::c_int;
                        while check_prevcol(linep, col, '\\' as ::core::ffi::c_int, &raw mut col) {
                            bslcnt += 1;
                        }
                        match_escaped = bslcnt & 1 as ::core::ffi::c_int;
                    }
                }
            }
            if hash_dir != 0 {
                if !oap.is_null() {
                    (*oap).motion_type = kMTLineWise;
                }
                if initc != '#' as ::core::ffi::c_int {
                    ptr = skipwhite(skipwhite(linep).offset(1 as ::core::ffi::c_int as isize));
                    if strncmp(
                        ptr,
                        b"if\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                        || strncmp(
                            ptr,
                            b"el\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        hash_dir = 1 as ::core::ffi::c_int;
                    } else if strncmp(
                        ptr,
                        b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        hash_dir = -1 as ::core::ffi::c_int;
                    } else {
                        return ::core::ptr::null_mut::<pos_T>();
                    }
                }
                (*pos.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
                while !got_int.get() {
                    if hash_dir > 0 as ::core::ffi::c_int {
                        if (*pos.ptr()).lnum == (*curbuf.get()).b_ml.ml_line_count {
                            break;
                        }
                    } else if (*pos.ptr()).lnum == 1 as linenr_T {
                        break;
                    }
                    (*pos.ptr()).lnum =
                        ((*pos.ptr()).lnum as ::core::ffi::c_int + hash_dir) as linenr_T;
                    linep = ml_get((*pos.ptr()).lnum);
                    line_breakcheck();
                    ptr = skipwhite(linep);
                    if *ptr as ::core::ffi::c_int != '#' as ::core::ffi::c_int {
                        continue;
                    }
                    (*pos.ptr()).col = ptr.offset_from(linep) as colnr_T;
                    ptr = skipwhite(ptr.offset(1 as ::core::ffi::c_int as isize));
                    if hash_dir > 0 as ::core::ffi::c_int {
                        if strncmp(
                            ptr,
                            b"if\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        {
                            count += 1;
                        } else if strncmp(
                            ptr,
                            b"el\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        {
                            if count == 0 as ::core::ffi::c_int {
                                return pos.ptr();
                            }
                        } else if strncmp(
                            ptr,
                            b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        {
                            if count == 0 as ::core::ffi::c_int {
                                return pos.ptr();
                            }
                            count -= 1;
                        }
                    } else if strncmp(
                        ptr,
                        b"if\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        if count == 0 as ::core::ffi::c_int {
                            return pos.ptr();
                        }
                        count -= 1;
                    } else if initc == '#' as ::core::ffi::c_int
                        && strncmp(
                            ptr,
                            b"el\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        if count == 0 as ::core::ffi::c_int {
                            return pos.ptr();
                        }
                    } else if strncmp(
                        ptr,
                        b"endif\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        count += 1;
                    }
                }
                return ::core::ptr::null_mut::<pos_T>();
            }
        }
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && !vim_strchr(b"()[]{}<>\0".as_ptr() as *const ::core::ffi::c_char, initc).is_null()
        {
            backwards = !backwards;
        }
        let mut do_quotes: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut at_start: ::core::ffi::c_int = 0;
        let mut start_in_quotes: TriState = kNone;
        let mut match_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        clearpos(&mut match_pos);
        if backwards as ::core::ffi::c_int != 0 && comment_dir != 0
            || lisp as ::core::ffi::c_int != 0
        {
            comment_col = check_linecomment(linep);
        }
        if lisp as ::core::ffi::c_int != 0
            && comment_col != MAXCOL as ::core::ffi::c_int
            && (*pos.ptr()).col > comment_col
        {
            lispcomm = true_0 != 0;
        }
        while !got_int.get() {
            if backwards {
                if lispcomm as ::core::ffi::c_int != 0 && (*pos.ptr()).col < comment_col {
                    break;
                }
                if (*pos.ptr()).col == 0 as ::core::ffi::c_int {
                    if (*pos.ptr()).lnum == 1 as linenr_T {
                        break;
                    }
                    (*pos.ptr()).lnum -= 1;
                    if maxtravel > 0 as int64_t && {
                        traveled += 1;
                        traveled as int64_t > maxtravel
                    } {
                        break;
                    }
                    linep = ml_get((*pos.ptr()).lnum);
                    (*pos.ptr()).col = ml_get_len((*pos.ptr()).lnum);
                    do_quotes = -1 as ::core::ffi::c_int;
                    line_breakcheck();
                    if comment_dir != 0 || lisp as ::core::ffi::c_int != 0 {
                        comment_col = check_linecomment(linep);
                    }
                    if lisp as ::core::ffi::c_int != 0
                        && comment_col != MAXCOL as ::core::ffi::c_int
                    {
                        (*pos.ptr()).col = comment_col as colnr_T;
                    }
                } else {
                    (*pos.ptr()).col -= 1;
                    (*pos.ptr()).col -=
                        utf_head_off(linep, linep.offset((*pos.ptr()).col as isize));
                }
            } else if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int == NUL
                || lisp as ::core::ffi::c_int != 0
                    && comment_col != MAXCOL as ::core::ffi::c_int
                    && (*pos.ptr()).col == comment_col
            {
                if (*pos.ptr()).lnum == (*curbuf.get()).b_ml.ml_line_count
                    || lispcomm as ::core::ffi::c_int != 0
                {
                    break;
                }
                (*pos.ptr()).lnum += 1;
                if maxtravel != 0 && {
                    let c2rust_fresh8 = traveled;
                    traveled = traveled + 1;
                    c2rust_fresh8 as int64_t > maxtravel
                } {
                    break;
                }
                linep = ml_get((*pos.ptr()).lnum);
                (*pos.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
                do_quotes = -1 as ::core::ffi::c_int;
                line_breakcheck();
                if lisp {
                    comment_col = check_linecomment(linep);
                }
            } else {
                (*pos.ptr()).col += utfc_ptr2len(linep.offset((*pos.ptr()).col as isize));
            }
            if (*pos.ptr()).col == 0 as ::core::ffi::c_int
                && flags & FM_BLOCKSTOP as ::core::ffi::c_int != 0
                && (*linep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '{' as ::core::ffi::c_int
                    || *linep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '}' as ::core::ffi::c_int)
            {
                if *linep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == findc
                    && count == 0 as ::core::ffi::c_int
                {
                    return pos.ptr();
                }
                break;
            } else if comment_dir != 0 {
                if comment_dir == FORWARD as ::core::ffi::c_int {
                    if *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                        && *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                    {
                        (*pos.ptr()).col += 1;
                        return pos.ptr();
                    }
                } else {
                    if (*pos.ptr()).col == 0 as ::core::ffi::c_int {
                        continue;
                    }
                    if raw_string {
                        if *linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == 'R' as ::core::ffi::c_int
                            && *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                                == '"' as ::core::ffi::c_int
                            && !vim_strchr(
                                linep
                                    .offset((*pos.ptr()).col as isize)
                                    .offset(1 as ::core::ffi::c_int as isize),
                                '(' as ::core::ffi::c_int,
                            )
                            .is_null()
                        {
                            if !find_rawstring_end(
                                linep,
                                pos.ptr(),
                                if count > 0 as ::core::ffi::c_int {
                                    &raw mut match_pos
                                } else {
                                    &raw mut (*curwin.get()).w_cursor
                                },
                            ) {
                                count += 1;
                                match_pos = pos.get();
                                match_pos.col -= 1;
                            }
                            linep = ml_get((*pos.ptr()).lnum);
                        }
                    } else if *linep.offset(
                        ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                        && *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                        && ((*pos.ptr()).col == 1 as ::core::ffi::c_int
                            || *linep.offset(
                                ((*pos.ptr()).col as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
                                    as isize,
                            ) as ::core::ffi::c_int
                                != '*' as ::core::ffi::c_int)
                        && (*pos.ptr()).col < comment_col
                    {
                        count += 1;
                        match_pos = pos.get();
                        match_pos.col -= 1;
                    } else {
                        if !(*linep.offset(
                            ((*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as isize,
                        ) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                            && *linep.offset((*pos.ptr()).col as isize) as ::core::ffi::c_int
                                == '/' as ::core::ffi::c_int)
                        {
                            continue;
                        }
                        if count > 0 as ::core::ffi::c_int {
                            pos.set(match_pos);
                        } else if (*pos.ptr()).col > 1 as ::core::ffi::c_int
                            && *linep.offset(
                                ((*pos.ptr()).col as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
                                    as isize,
                            ) as ::core::ffi::c_int
                                == '/' as ::core::ffi::c_int
                            && (*pos.ptr()).col <= comment_col
                        {
                            (*pos.ptr()).col -= 2 as ::core::ffi::c_int;
                        } else {
                            if ignore_cend {
                                continue;
                            }
                            return ::core::ptr::null_mut::<pos_T>();
                        }
                        return pos.ptr();
                    }
                }
            } else {
                if cpo_match {
                    do_quotes = 0 as ::core::ffi::c_int;
                } else if do_quotes == -1 as ::core::ffi::c_int {
                    at_start = do_quotes;
                    ptr = linep;
                    while *ptr != 0 {
                        if ptr
                            == linep
                                .offset((*pos.ptr()).col as isize)
                                .offset(backwards as ::core::ffi::c_int as isize)
                        {
                            at_start = do_quotes & 1 as ::core::ffi::c_int;
                        }
                        if *ptr as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                            && (ptr == linep
                                || *ptr.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != '\'' as ::core::ffi::c_int
                                || *ptr.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != '\'' as ::core::ffi::c_int)
                        {
                            do_quotes += 1;
                        }
                        if *ptr as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != NUL
                        {
                            ptr = ptr.offset(1);
                        }
                        ptr = ptr.offset(1);
                    }
                    do_quotes &= 1 as ::core::ffi::c_int;
                    if do_quotes == 0 {
                        inquote = false_0 != 0;
                        if *ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        {
                            do_quotes = 1 as ::core::ffi::c_int;
                            if start_in_quotes as ::core::ffi::c_int == kNone as ::core::ffi::c_int
                            {
                                inquote = true_0 != 0;
                                start_in_quotes = kTrue;
                            } else if backwards {
                                inquote = true_0 != 0;
                            }
                        }
                        if (*pos.ptr()).lnum > 1 as linenr_T {
                            ptr = ml_get((*pos.ptr()).lnum - 1 as linenr_T);
                            if *ptr as ::core::ffi::c_int != 0
                                && *ptr
                                    .offset(ml_get_len((*pos.ptr()).lnum - 1 as linenr_T) as isize)
                                    .offset(-(1 as ::core::ffi::c_int as isize))
                                    as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                            {
                                do_quotes = 1 as ::core::ffi::c_int;
                                if start_in_quotes as ::core::ffi::c_int
                                    == kNone as ::core::ffi::c_int
                                {
                                    inquote = at_start != 0;
                                    if inquote {
                                        start_in_quotes = kTrue;
                                    }
                                } else if !backwards {
                                    inquote = true_0 != 0;
                                }
                            }
                            linep = ml_get((*pos.ptr()).lnum);
                        }
                    }
                }
                if start_in_quotes as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
                    start_in_quotes = kFalse;
                }
                let c: ::core::ffi::c_int = utf_ptr2char(linep.offset((*pos.ptr()).col as isize));
                's_1456: {
                    match c {
                        NUL => {
                            if (*pos.ptr()).col == 0 as ::core::ffi::c_int
                                || *linep.offset(
                                    ((*pos.ptr()).col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as isize,
                                ) as ::core::ffi::c_int
                                    != '\\' as ::core::ffi::c_int
                            {
                                inquote = false_0 != 0;
                                start_in_quotes = kFalse;
                            }
                            break 's_1456;
                        }
                        34 => {
                            if do_quotes != 0 {
                                let mut col_0: ::core::ffi::c_int = 0;
                                col_0 = (*pos.ptr()).col as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int;
                                while col_0 >= 0 as ::core::ffi::c_int {
                                    if *linep.offset(col_0 as isize) as ::core::ffi::c_int
                                        != '\\' as ::core::ffi::c_int
                                    {
                                        break;
                                    }
                                    col_0 -= 1;
                                }
                                if (*pos.ptr()).col - 1 as ::core::ffi::c_int - col_0
                                    & 1 as ::core::ffi::c_int
                                    == 0 as ::core::ffi::c_int
                                {
                                    inquote = !inquote;
                                    start_in_quotes = kFalse;
                                }
                            }
                            break 's_1456;
                        }
                        39 => {
                            if !cpo_match
                                && initc != '\'' as ::core::ffi::c_int
                                && findc != '\'' as ::core::ffi::c_int
                            {
                                if backwards {
                                    if (*pos.ptr()).col > 1 as ::core::ffi::c_int {
                                        if *linep.offset(
                                            ((*pos.ptr()).col as ::core::ffi::c_int
                                                - 2 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                            as ::core::ffi::c_int
                                            == '\'' as ::core::ffi::c_int
                                        {
                                            (*pos.ptr()).col -= 2 as ::core::ffi::c_int;
                                            break 's_1456;
                                        } else if *linep.offset(
                                            ((*pos.ptr()).col as ::core::ffi::c_int
                                                - 2 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                            as ::core::ffi::c_int
                                            == '\\' as ::core::ffi::c_int
                                            && (*pos.ptr()).col > 2 as ::core::ffi::c_int
                                            && *linep.offset(
                                                ((*pos.ptr()).col as ::core::ffi::c_int
                                                    - 3 as ::core::ffi::c_int)
                                                    as isize,
                                            )
                                                as ::core::ffi::c_int
                                                == '\'' as ::core::ffi::c_int
                                        {
                                            (*pos.ptr()).col -= 3 as ::core::ffi::c_int;
                                            break 's_1456;
                                        }
                                    }
                                } else if *linep.offset(
                                    ((*pos.ptr()).col as ::core::ffi::c_int
                                        + 1 as ::core::ffi::c_int)
                                        as isize,
                                ) != 0
                                {
                                    if *linep.offset(
                                        ((*pos.ptr()).col as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int)
                                            as isize,
                                    ) as ::core::ffi::c_int
                                        == '\\' as ::core::ffi::c_int
                                        && *linep.offset(
                                            ((*pos.ptr()).col as ::core::ffi::c_int
                                                + 2 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                        && *linep.offset(
                                            ((*pos.ptr()).col as ::core::ffi::c_int
                                                + 3 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                            as ::core::ffi::c_int
                                            == '\'' as ::core::ffi::c_int
                                    {
                                        (*pos.ptr()).col += 3 as ::core::ffi::c_int;
                                        break 's_1456;
                                    } else if *linep.offset(
                                        ((*pos.ptr()).col as ::core::ffi::c_int
                                            + 2 as ::core::ffi::c_int)
                                            as isize,
                                    )
                                        as ::core::ffi::c_int
                                        == '\'' as ::core::ffi::c_int
                                    {
                                        (*pos.ptr()).col += 2 as ::core::ffi::c_int;
                                        break 's_1456;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    if !((*curbuf.get()).b_p_lisp != 0
                        && !vim_strchr(b"(){}[]\0".as_ptr() as *const ::core::ffi::c_char, c)
                            .is_null()
                        && (*pos.ptr()).col > 1 as ::core::ffi::c_int
                        && check_prevcol(
                            linep,
                            (*pos.ptr()).col as ::core::ffi::c_int,
                            '\\' as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ) as ::core::ffi::c_int
                            != 0
                        && check_prevcol(
                            linep,
                            (*pos.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            '#' as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ) as ::core::ffi::c_int
                            != 0)
                    {
                        if (!inquote
                            || start_in_quotes as ::core::ffi::c_int == kTrue as ::core::ffi::c_int)
                            && (c == initc || c == findc)
                        {
                            let mut bslcnt_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if !cpo_bsl {
                                let mut col_1: ::core::ffi::c_int =
                                    (*pos.ptr()).col as ::core::ffi::c_int;
                                while check_prevcol(
                                    linep,
                                    col_1,
                                    '\\' as ::core::ffi::c_int,
                                    &raw mut col_1,
                                ) {
                                    bslcnt_0 += 1;
                                }
                            }
                            if cpo_bsl as ::core::ffi::c_int != 0
                                || bslcnt_0 & 1 as ::core::ffi::c_int == match_escaped
                            {
                                if c == initc {
                                    count += 1;
                                } else {
                                    if count == 0 as ::core::ffi::c_int {
                                        return pos.ptr();
                                    }
                                    count -= 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        if comment_dir == BACKWARD as ::core::ffi::c_int && count > 0 as ::core::ffi::c_int {
            pos.set(match_pos);
            return pos.ptr();
        }
        return NULL_0 as *mut pos_T;
    }
}

pub unsafe extern "C" fn check_linecomment(
    mut line: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = line;
        if (*curbuf.get()).b_p_lisp != 0 {
            if !vim_strchr(p, ';' as ::core::ffi::c_int).is_null() {
                let mut in_str: bool = false_0 != 0;
                loop {
                    p = strpbrk(p, b"\";\0".as_ptr() as *const ::core::ffi::c_char);
                    if p.is_null() {
                        break;
                    }
                    if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                        if in_str {
                            if *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                                != '\\' as ::core::ffi::c_int
                            {
                                in_str = false_0 != 0;
                            }
                        } else if p == line
                            || p.offset_from(line) >= 2 as isize
                                && *p.offset(-(1 as ::core::ffi::c_int as isize))
                                    as ::core::ffi::c_int
                                    != '\\' as ::core::ffi::c_int
                                && *p.offset(-(2 as ::core::ffi::c_int as isize))
                                    as ::core::ffi::c_int
                                    != '#' as ::core::ffi::c_int
                        {
                            in_str = true_0 != 0;
                        }
                    } else if !in_str
                        && (p.offset_from(line) < 2 as isize
                            || *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                                != '\\' as ::core::ffi::c_int
                                && *p.offset(-(2 as ::core::ffi::c_int as isize))
                                    as ::core::ffi::c_int
                                    != '#' as ::core::ffi::c_int)
                        && is_pos_in_string(line, p.offset_from(line) as colnr_T) == 0
                    {
                        break;
                    }
                    p = p.offset(1);
                }
            } else {
                p = ::core::ptr::null::<::core::ffi::c_char>();
            }
        } else {
            loop {
                p = vim_strchr(p, '/' as ::core::ffi::c_int);
                if p.is_null() {
                    break;
                }
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
                    && (p == line
                        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != '*' as ::core::ffi::c_int
                        || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != '*' as ::core::ffi::c_int)
                    && is_pos_in_string(line, p.offset_from(line) as colnr_T) == 0
                {
                    break;
                }
                p = p.offset(1);
            }
        }
        if p.is_null() {
            return MAXCOL as ::core::ffi::c_int;
        }
        return p.offset_from(line) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn linewhite(mut lnum: linenr_T) -> bool {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = skipwhite(ml_get(lnum));
        return *p as ::core::ffi::c_int == NUL;
    }
}
