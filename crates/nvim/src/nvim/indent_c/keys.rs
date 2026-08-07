//! When to reindent: 'cinkeys', and the front doors.
//!
//! `in_cinkeys` answers whether a typed character should trigger a reindent,
//! which is the whole of 'cinkeys': a comma-separated list of keys, each
//! optionally prefixed by `*` (reindent *before* inserting), `!` (never insert,
//! just reindent) or `0` (only when it is the first thing on the line), plus the
//! `o`/`O`/`e`/`=` word forms.  `cindent_on` is the "is C indenting active at
//! all" test 'cindent'/'indentexpr' share, and `f_cindent` is `cindent()`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn cindent_on() -> bool {
    unsafe {
        return p_paste.get() == 0
            && ((*curbuf.get()).b_p_cin != 0
                || *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL);
    }
}

pub unsafe extern "C" fn in_cinkeys(
    mut keytyped: ::core::ffi::c_int,
    mut when: ::core::ffi::c_int,
    mut line_is_empty: bool,
) -> bool {
    unsafe {
        let mut look: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut try_match: bool = false;
        let mut try_match_word: bool = false;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut icase: bool = false;
        if keytyped == NUL {
            return false;
        }
        if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
            look = (*curbuf.get()).b_p_indk;
        } else {
            look = (*curbuf.get()).b_p_cink;
        }
        while *look != 0 {
            match when {
                42 => {
                    try_match = *look as ::core::ffi::c_int == '*' as ::core::ffi::c_int;
                }
                33 => {
                    try_match = *look as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
                }
                _ => {
                    try_match = *look as ::core::ffi::c_int != '*' as ::core::ffi::c_int;
                }
            }
            if *look as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                || *look as ::core::ffi::c_int == '!' as ::core::ffi::c_int
            {
                look = look.offset(1);
            }
            if *look as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
                try_match_word = try_match;
                if !line_is_empty {
                    try_match = false;
                }
                look = look.offset(1);
            } else {
                try_match_word = false;
            }
            if *look as ::core::ffi::c_int == '^' as ::core::ffi::c_int
                && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= '?' as ::core::ffi::c_int
                && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    <= '_' as ::core::ffi::c_int
            {
                if try_match as ::core::ffi::c_int != 0
                    && keytyped
                        == (if (*look.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int)
                            < 'a' as ::core::ffi::c_int
                            || *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'z' as ::core::ffi::c_int
                        {
                            *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }) ^ 0x40 as ::core::ffi::c_int
                {
                    return true;
                }
                look = look.offset(2 as ::core::ffi::c_int as isize);
            } else if *look as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
                if try_match as ::core::ffi::c_int != 0 && keytyped == KEY_OPEN_FORW {
                    return true;
                }
                look = look.offset(1);
            } else if *look as ::core::ffi::c_int == 'O' as ::core::ffi::c_int {
                if try_match as ::core::ffi::c_int != 0 && keytyped == KEY_OPEN_BACK {
                    return true;
                }
                look = look.offset(1);
            } else if *look as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                if try_match as ::core::ffi::c_int != 0
                    && keytyped == 'e' as ::core::ffi::c_int
                    && (*curwin.get()).w_cursor.col >= 4 as ::core::ffi::c_int
                {
                    p = get_cursor_line_ptr();
                    if skipwhite(p)
                        == p.offset((*curwin.get()).w_cursor.col as isize)
                            .offset(-(4 as ::core::ffi::c_int as isize))
                        && strncmp(
                            p.offset((*curwin.get()).w_cursor.col as isize)
                                .offset(-(4 as ::core::ffi::c_int as isize)),
                            c"else".as_ptr(),
                            4 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        return true;
                    }
                }
                look = look.offset(1);
            } else if *look as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                if try_match as ::core::ffi::c_int != 0 && keytyped == ':' as ::core::ffi::c_int {
                    p = get_cursor_line_ptr();
                    if cin_iscase(p, false) || cin_isscopedecl(p) || cin_islabel() {
                        return true;
                    }
                    p = get_cursor_line_ptr();
                    if (*curwin.get()).w_cursor.col > 2 as ::core::ffi::c_int
                        && *p.offset(
                            ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int
                            == ':' as ::core::ffi::c_int
                        && *p.offset(
                            ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                - 2 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int
                            == ':' as ::core::ffi::c_int
                    {
                        *p.offset(
                            ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int) as isize,
                        ) = ' ' as ::core::ffi::c_char;
                        let i: bool = cin_iscase(p, false) || cin_isscopedecl(p) || cin_islabel();
                        p = get_cursor_line_ptr();
                        *p.offset(
                            ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int) as isize,
                        ) = ':' as ::core::ffi::c_char;
                        if i {
                            return true;
                        }
                    }
                }
                look = look.offset(1);
            } else if *look as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
                if try_match {
                    if !vim_strchr(
                        c"<>!*oOe0:".as_ptr(),
                        *look.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null()
                        && keytyped
                            == *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    {
                        return true;
                    }
                    if keytyped
                        == get_special_key_code(look.offset(1 as ::core::ffi::c_int as isize))
                    {
                        return true;
                    }
                }
                while *look as ::core::ffi::c_int != 0
                    && *look as ::core::ffi::c_int != '>' as ::core::ffi::c_int
                {
                    look = look.offset(1);
                }
                while *look as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                    look = look.offset(1);
                }
            } else if *look as ::core::ffi::c_int == '=' as ::core::ffi::c_int
                && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ',' as ::core::ffi::c_int
                && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                look = look.offset(1);
                if *look as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
                    icase = true;
                    look = look.offset(1);
                } else {
                    icase = false;
                }
                p = vim_strchr(look, ',' as ::core::ffi::c_int);
                if p.is_null() {
                    p = look.offset(strlen(look) as isize);
                }
                if (try_match as ::core::ffi::c_int != 0
                    || try_match_word as ::core::ffi::c_int != 0)
                    && (*curwin.get()).w_cursor.col >= p.offset_from(look) as colnr_T
                {
                    let mut match_0: bool = false;
                    if keytyped == KEY_COMPLETE {
                        let mut n: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        let mut s: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                        s = line.offset((*curwin.get()).w_cursor.col as isize);
                        while s > line {
                            n = mb_prevptr(line, s);
                            if !vim_iswordp(n) {
                                break;
                            }
                            s = n;
                        }
                        '_c2rust_label: {
                            if p >= look
                                && p.offset_from(look) as uintmax_t
                                    <= 18446744073709551615 as uintmax_t
                            {
                            } else {
                                __assert_fail(
                                    c"p >= look && (uintmax_t)(p - look) <= SIZE_MAX".as_ptr(),
                                    c"src/nvim/indent_c.rs".as_ptr(),
                                    3933 as ::core::ffi::c_uint,
                                    __ASSERT_FUNCTION.as_ptr(),
                                );
                            }
                        };
                        if s.offset(p.offset_from(look) as isize)
                            <= line.offset((*curwin.get()).w_cursor.col as isize)
                            && (if icase as ::core::ffi::c_int != 0 {
                                mb_strnicmp(s, look, p.offset_from(look) as size_t)
                            } else {
                                strncmp(s, look, p.offset_from(look) as size_t)
                            }) == 0 as ::core::ffi::c_int
                        {
                            match_0 = true;
                        }
                    } else if keytyped
                        == *p.offset(-1 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                        || icase as ::core::ffi::c_int != 0
                            && keytyped < 256 as ::core::ffi::c_int
                            && keytyped >= 0 as ::core::ffi::c_int
                            && tolower(keytyped)
                                == tolower(*p.offset(-1 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int)
                    {
                        let mut line_0: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        '_c2rust_label_0: {
                            if p >= look
                                && p.offset_from(look) as uintmax_t
                                    <= 18446744073709551615 as uintmax_t
                            {
                            } else {
                                __assert_fail(
                                    c"p >= look && (uintmax_t)(p - look) <= SIZE_MAX".as_ptr(),
                                    c"src/nvim/indent_c.rs".as_ptr(),
                                    3946 as ::core::ffi::c_uint,
                                    __ASSERT_FUNCTION.as_ptr(),
                                );
                            }
                        };
                        if ((*curwin.get()).w_cursor.col == p.offset_from(look) as colnr_T
                            || !vim_iswordc(
                                *line_0.offset((-p.offset_from(look) - 1 as isize) as isize)
                                    as uint8_t
                                    as ::core::ffi::c_int,
                            ))
                            && (if icase as ::core::ffi::c_int != 0 {
                                mb_strnicmp(
                                    line_0.offset(-(p.offset_from(look) as isize)),
                                    look,
                                    p.offset_from(look) as size_t,
                                )
                            } else {
                                strncmp(
                                    line_0.offset(-(p.offset_from(look) as isize)),
                                    look,
                                    p.offset_from(look) as size_t,
                                )
                            }) == 0 as ::core::ffi::c_int
                        {
                            match_0 = true;
                        }
                    }
                    if match_0 as ::core::ffi::c_int != 0
                        && try_match_word as ::core::ffi::c_int != 0
                        && !try_match
                    {
                        if getwhitecols_curline()
                            != ((*curwin.get()).w_cursor.col as isize - p.offset_from(look))
                                as ::core::ffi::c_int as intptr_t
                        {
                            match_0 = false;
                        }
                    }
                    if match_0 {
                        return true;
                    }
                }
                look = p;
            } else {
                if try_match as ::core::ffi::c_int != 0
                    && *look as uint8_t as ::core::ffi::c_int == keytyped
                {
                    return true;
                }
                if *look as ::core::ffi::c_int != NUL {
                    look = look.offset(1);
                }
            }
            look = skip_to_option_part(look);
        }
        return false;
    }
}

pub unsafe extern "C" fn do_c_expr_indent() {
    unsafe {
        if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
            fixthisline(Some(
                get_expr_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
            ));
        } else {
            fixthisline(Some(
                get_c_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
            ));
        };
    }
}

pub unsafe extern "C" fn f_cindent(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        let mut lnum: linenr_T = tv_get_lnum(argvars);
        if lnum >= 1 as linenr_T && lnum <= (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = lnum;
            (*rettv).vval.v_number = get_c_indent() as varnumber_T;
            (*curwin.get()).w_cursor = pos;
        } else {
            (*rettv).vval.v_number = -1 as varnumber_T;
        };
    }
}
