//! The C++ shapes: `namespace`, `extern "C"` and a base-class list.
//!
//! Three 'cinoptions' letters live here.  `N` (`b_ind_cpp_namespace`) and `E`
//! (`b_ind_cpp_extern_c`) are the two block openers whose contents upstream does
//! not want indented, and both are recognised from the *opening* line.  `k`
//! (`b_ind_cpp_baseclass`) is the harder one: `cin_is_cpp_baseclass` decides
//! whether a line is inside a constructor's initialiser list or a class's base
//! clause, which needs a scan back to the `class`/`:` that started it -- so it
//! caches its answer in the `cpp_baseclass_cache_T` its caller owns.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn cin_is_cpp_namespace(mut s: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut has_name: bool = false;
        let mut has_name_start: bool = false;
        s = cin_skipcomment(s);
        while (strncmp(s, c"inline".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int
            || strncmp(s, c"export".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int)
            && (*s.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || !vim_iswordc(
                    *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                ))
        {
            s = cin_skipcomment(skipwhite(s.offset(6 as ::core::ffi::c_int as isize)));
        }
        if strncmp(s, c"namespace".as_ptr(), 9 as size_t) == 0 as ::core::ffi::c_int
            && (*s.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || !vim_iswordc(
                    *s.offset(9 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                ))
        {
            p = cin_skipcomment(skipwhite(s.offset(9 as ::core::ffi::c_int as isize)));
            while *p as ::core::ffi::c_int != NUL {
                if ascii_iswhite(*p as ::core::ffi::c_int) {
                    has_name = true;
                    p = cin_skipcomment(skipwhite(p));
                } else {
                    if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                        break;
                    }
                    if vim_iswordc(*p as uint8_t as ::core::ffi::c_int) {
                        has_name_start = true;
                        if has_name {
                            return false;
                        }
                        p = p.offset(1);
                    } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ':' as ::core::ffi::c_int
                        && vim_iswordc(*p.offset(2 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int) as ::core::ffi::c_int
                            != 0
                    {
                        if !has_name_start || has_name as ::core::ffi::c_int != 0 {
                            return false;
                        }
                        p = p.offset(3 as ::core::ffi::c_int as isize);
                    } else {
                        return false;
                    }
                }
            }
            return true;
        }
        return false;
    }
}

pub(crate) unsafe extern "C" fn cin_is_cpp_baseclass(
    mut cached: *mut cpp_baseclass_cache_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut pos: *mut lpos_T = &raw mut (*cached).lpos;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut class_or_struct: ::core::ffi::c_int = 0;
        let mut lookfor_ctor_init: ::core::ffi::c_int = 0;
        let mut cpp_base_class: ::core::ffi::c_int = 0;
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut line: *const ::core::ffi::c_char = get_cursor_line_ptr();
        if (*pos).lnum <= lnum {
            return (*cached).found;
        }
        (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
        s = skipwhite(line);
        if *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
            return false_0;
        }
        s = cin_skipcomment(s);
        if *s as ::core::ffi::c_int == NUL {
            return false_0;
        }
        class_or_struct = false_0;
        lookfor_ctor_init = class_or_struct;
        cpp_base_class = lookfor_ctor_init;
        while lnum > 1 as linenr_T {
            line = ml_get(lnum - 1 as linenr_T);
            s = skipwhite(line);
            if *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == NUL
            {
                break;
            }
            while *s as ::core::ffi::c_int != NUL {
                s = cin_skipcomment(s);
                if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                    || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                    || *s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                        && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize))
                {
                    break;
                }
                if *s as ::core::ffi::c_int != NUL {
                    s = s.offset(1);
                }
            }
            if *s as ::core::ffi::c_int != NUL {
                break;
            }
            lnum -= 1;
        }
        (*pos).lnum = lnum;
        line = ml_get(lnum);
        s = line;
        loop {
            if *s as ::core::ffi::c_int == NUL {
                if lnum == (*curwin.get()).w_cursor.lnum {
                    break;
                }
                lnum += 1;
                line = ml_get(lnum);
                s = line;
            }
            if s == line {
                if cin_iscase(s, false) {
                    break;
                }
                s = cin_skipcomment(line);
                if *s as ::core::ffi::c_int == NUL {
                    continue;
                }
            }
            if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '"' as ::core::ffi::c_int
                || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'R' as ::core::ffi::c_int
                    && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
            {
                s = skip_string(s).offset(1 as ::core::ffi::c_int as isize);
            } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                {
                    lookfor_ctor_init = false_0;
                    s = cin_skipcomment(s.offset(2 as ::core::ffi::c_int as isize));
                } else if lookfor_ctor_init != 0 || class_or_struct != 0 {
                    cpp_base_class = true_0;
                    class_or_struct = false_0;
                    lookfor_ctor_init = class_or_struct;
                    (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
                    s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
                } else {
                    s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
                }
            } else if strncmp(s, c"class".as_ptr(), 5 as size_t) == 0 as ::core::ffi::c_int
                && !vim_isIDc(
                    *s.offset(5 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                )
                || strncmp(s, c"struct".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int
                    && !vim_isIDc(*s.offset(6 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int)
            {
                class_or_struct = true_0;
                lookfor_ctor_init = false_0;
                if *s as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
                    s = cin_skipcomment(s.offset(5 as ::core::ffi::c_int as isize));
                } else {
                    s = cin_skipcomment(s.offset(6 as ::core::ffi::c_int as isize));
                }
            } else {
                if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '{' as ::core::ffi::c_int
                    || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '}' as ::core::ffi::c_int
                    || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ';' as ::core::ffi::c_int
                {
                    class_or_struct = false_0;
                    lookfor_ctor_init = class_or_struct;
                    cpp_base_class = lookfor_ctor_init;
                } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ')' as ::core::ffi::c_int
                {
                    class_or_struct = false_0;
                    lookfor_ctor_init = true_0;
                } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '?' as ::core::ffi::c_int
                {
                    return false_0;
                } else if !vim_isIDc(
                    *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                ) {
                    class_or_struct = false_0;
                    lookfor_ctor_init = false_0;
                } else if (*pos).col == 0 as ::core::ffi::c_int {
                    lookfor_ctor_init = false_0;
                    if cpp_base_class != 0 {
                        (*pos).col = s.offset_from(line) as colnr_T;
                    }
                }
                if lnum == (*curwin.get()).w_cursor.lnum
                    && *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                    && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize))
                {
                    (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
                }
                s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
            }
        }
        (*cached).found = cpp_base_class;
        if cpp_base_class != 0 {
            (*pos).lnum = lnum;
        }
        return cpp_base_class;
    }
}

pub(crate) unsafe extern "C" fn get_baseclass_amount(
    mut col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut amount: ::core::ffi::c_int = 0;
        let mut vcol: colnr_T = 0;
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        if col == 0 as ::core::ffi::c_int {
            amount = get_indent();
            if find_last_paren(
                get_cursor_line_ptr(),
                '(' as ::core::ffi::c_char,
                ')' as ::core::ffi::c_char,
            ) != 0
                && {
                    trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                    !trypos.is_null()
                }
            {
                amount = get_indent_lnum((*trypos).lnum);
            }
            if cin_ends_in(get_cursor_line_ptr(), c",".as_ptr()) == 0 {
                amount += (*curbuf.get()).b_ind_cpp_baseclass;
            }
        } else {
            (*curwin.get()).w_cursor.col = col as colnr_T;
            getvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                &raw mut vcol,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
            );
            amount = vcol;
        }
        if amount < (*curbuf.get()).b_ind_cpp_baseclass {
            amount = (*curbuf.get()).b_ind_cpp_baseclass;
        }
        return amount;
    }
}

pub(crate) unsafe extern "C" fn cin_is_cpp_extern_c(
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut has_string_literal: ::core::ffi::c_int = false_0;
        s = cin_skipcomment(s);
        if strncmp(s, c"extern".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int
            && (*s.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || !vim_iswordc(
                    *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                ))
        {
            p = cin_skipcomment(skipwhite(s.offset(6 as ::core::ffi::c_int as isize)));
            while *p as ::core::ffi::c_int != NUL {
                if ascii_iswhite(*p as ::core::ffi::c_int) {
                    p = cin_skipcomment(skipwhite(p));
                } else {
                    if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                        break;
                    }
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'C' as ::core::ffi::c_int
                        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '"' as ::core::ffi::c_int
                    {
                        if has_string_literal != 0 {
                            return false_0;
                        }
                        has_string_literal = true_0;
                        p = p.offset(3 as ::core::ffi::c_int as isize);
                    } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'C' as ::core::ffi::c_int
                        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '+' as ::core::ffi::c_int
                        && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '+' as ::core::ffi::c_int
                        && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '"' as ::core::ffi::c_int
                    {
                        if has_string_literal != 0 {
                            return false_0;
                        }
                        has_string_literal = true_0;
                        p = p.offset(5 as ::core::ffi::c_int as isize);
                    } else {
                        return false_0;
                    }
                }
            }
            return if has_string_literal != 0 {
                true_0
            } else {
                false_0
            };
        }
        return false_0;
    }
}
