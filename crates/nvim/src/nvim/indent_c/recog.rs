//! What kind of statement a line is.
//!
//! The `cin_is*` predicates `get_c_indent`'s backwards scan asks of each line it
//! walks past: is it a `case`/`default` label, a scope declaration
//! (`private:`, and whatever else 'cinscopedecls' names), a `break`, one of the
//! 'cinwords' keywords, an `if`/`else`/`do`, the `while` belonging to a `do`.
//! `cin_isterminated` is the one the whole state machine turns on -- it answers
//! the *character* a statement ended with (`;`, `,`, `{`, or 0 for "did not
//! end"), which is what tells a continuation line from a finished one.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn cin_is_cinword(mut line: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut retval: bool = false;
        let mut cinw_len: size_t = strlen((*curbuf.get()).b_p_cinw).wrapping_add(1 as size_t);
        let mut cinw_buf: *mut ::core::ffi::c_char = xmalloc(cinw_len) as *mut ::core::ffi::c_char;
        line = skipwhite(line);
        let mut cinw: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cinw;
        while *cinw != 0 {
            let mut len: size_t = copy_option_part(
                &raw mut cinw,
                cinw_buf,
                cinw_len,
                c",".as_ptr() as *mut ::core::ffi::c_char,
            );
            if !(strncmp(line, cinw_buf, len) == 0 as ::core::ffi::c_int
                && (!vim_iswordc(*line.offset(len as isize) as uint8_t as ::core::ffi::c_int)
                    || !vim_iswordc(
                        *line.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )))
            {
                continue;
            }
            retval = true;
            break;
        }
        xfree(cinw_buf as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub(crate) unsafe extern "C" fn cin_has_js_key(mut text: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut s: *const ::core::ffi::c_char = skipwhite(text);
        let mut quote: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
        if *s as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int
        {
            quote = *s;
            s = s.offset(1);
        }
        if !vim_isIDc(*s as uint8_t as ::core::ffi::c_int) {
            return false;
        }
        while vim_isIDc(*s as uint8_t as ::core::ffi::c_int) {
            s = s.offset(1);
        }
        if *s as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == quote as ::core::ffi::c_int
        {
            s = s.offset(1);
        }
        s = cin_skipcomment(s);
        return *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ':' as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_iscase(
    mut s: *const ::core::ffi::c_char,
    mut strict: bool,
) -> bool {
    unsafe {
        s = cin_skipcomment(s);
        if cin_starts_with(s, c"case".as_ptr()) != 0 {
            s = s.offset(4 as ::core::ffi::c_int as isize);
            while *s != 0 {
                s = cin_skipcomment(s);
                if *s as ::core::ffi::c_int == NUL {
                    break;
                }
                if *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                    if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    {
                        s = s.offset(1);
                    } else {
                        return true;
                    }
                }
                if *s as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                    && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\'' as ::core::ffi::c_int
                {
                    s = s.offset(2 as ::core::ffi::c_int as isize);
                } else if *s as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    && (*s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int)
                {
                    return false;
                } else if *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                    if strict {
                        return false;
                    }
                    return true;
                }
                s = s.offset(1);
            }
            return false;
        }
        if cin_isdefault(s) != 0 {
            return true;
        }
        return false;
    }
}

pub(crate) unsafe extern "C" fn cin_isdefault(
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return (strncmp(s, c"default".as_ptr(), 7 as size_t) == 0 as ::core::ffi::c_int
            && {
                s = cin_skipcomment(s.offset(7 as ::core::ffi::c_int as isize));
                *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            }
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ':' as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_isscopedecl(mut p: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut s: *const ::core::ffi::c_char = cin_skipcomment(p);
        let cinsd_len: size_t = strlen((*curbuf.get()).b_p_cinsd).wrapping_add(1 as size_t);
        let mut cinsd_buf: *mut ::core::ffi::c_char =
            xmalloc(cinsd_len) as *mut ::core::ffi::c_char;
        let mut found: bool = false;
        let mut cinsd: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cinsd;
        while *cinsd != 0 {
            let len: size_t = copy_option_part(
                &raw mut cinsd,
                cinsd_buf,
                cinsd_len,
                c",".as_ptr() as *mut ::core::ffi::c_char,
            );
            if strncmp(s, cinsd_buf, len) != 0 as ::core::ffi::c_int {
                continue;
            }
            let mut skip: *const ::core::ffi::c_char = cin_skipcomment(s.offset(len as isize));
            if !(*skip as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && *skip.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ':' as ::core::ffi::c_int)
            {
                continue;
            }
            found = true;
            break;
        }
        xfree(cinsd_buf as *mut ::core::ffi::c_void);
        return found;
    }
}

pub(crate) unsafe extern "C" fn cin_isterminated(
    mut s: *const ::core::ffi::c_char,
    mut incl_open: ::core::ffi::c_int,
    mut incl_comma: ::core::ffi::c_int,
) -> ::core::ffi::c_char {
    unsafe {
        let mut found_start: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
        let mut n_open: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        let mut is_else: ::core::ffi::c_int = false_0;
        s = cin_skipcomment(s);
        if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int && cin_iselse(s) == 0
        {
            found_start = *s;
        }
        if found_start == 0 {
            is_else = cin_iselse(s);
        }
        while *s != 0 {
            s = skip_string(cin_skipcomment(s));
            if *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                && n_open > 0 as ::core::ffi::c_uint
            {
                n_open = n_open.wrapping_sub(1);
            }
            if (is_else == 0 || n_open == 0 as ::core::ffi::c_uint)
                && (*s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                    || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                    || incl_comma != 0 && *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize))
            {
                return *s;
            } else if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                if incl_open != 0 && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) {
                    return *s;
                } else {
                    n_open = n_open.wrapping_add(1);
                }
            }
            if *s != 0 {
                s = s.offset(1);
            }
        }
        return found_start;
    }
}

pub(crate) unsafe extern "C" fn cin_isif(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        return (strncmp(p, c"if".as_ptr(), 2 as size_t) == 0 as ::core::ffi::c_int
            && !vim_isIDc(
                *p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            )) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_iselse(
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
            p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
        }
        return (strncmp(p, c"else".as_ptr(), 4 as size_t) == 0 as ::core::ffi::c_int
            && !vim_isIDc(
                *p.offset(4 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            )) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_isdo(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        return (strncmp(p, c"do".as_ptr(), 2 as size_t) == 0 as ::core::ffi::c_int
            && !vim_isIDc(
                *p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            )) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_iswhileofdo(
    mut p: *const ::core::ffi::c_char,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut cursor_save: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut retval: ::core::ffi::c_int = false_0;
        p = cin_skipcomment(p);
        if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
            p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
        }
        if cin_starts_with(p, c"while".as_ptr()) != 0 {
            cursor_save = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor.lnum = lnum;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            p = get_cursor_line_ptr();
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != 'w' as ::core::ffi::c_int
            {
                p = p.offset(1);
                (*curwin.get()).w_cursor.col += 1;
            }
            trypos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (*curbuf.get()).b_ind_maxparen as int64_t,
            );
            if !trypos.is_null()
                && *cin_skipcomment(ml_get_pos(trypos).offset(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int
                    == ';' as ::core::ffi::c_int
            {
                retval = true_0;
            }
            (*curwin.get()).w_cursor = cursor_save;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn cin_is_if_for_while_before_offset(
    mut line: *const ::core::ffi::c_char,
    mut poffset: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut offset: ::core::ffi::c_int = *poffset;
        let c2rust_fresh3 = offset;
        offset = offset - 1;
        if c2rust_fresh3 < 2 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        while offset > 2 as ::core::ffi::c_int
            && ascii_iswhite(*line.offset(offset as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            offset -= 1;
        }
        offset -= 1 as ::core::ffi::c_int;
        '_probablyFound: {
            if strncmp(line.offset(offset as isize), c"if".as_ptr(), 2 as size_t) != 0 {
                if offset >= 1 as ::core::ffi::c_int {
                    offset -= 1 as ::core::ffi::c_int;
                    if strncmp(line.offset(offset as isize), c"for".as_ptr(), 3 as size_t) == 0 {
                        break '_probablyFound;
                    } else if offset >= 2 as ::core::ffi::c_int {
                        offset -= 2 as ::core::ffi::c_int;
                        if strncmp(line.offset(offset as isize), c"while".as_ptr(), 5 as size_t)
                            == 0
                        {
                            break '_probablyFound;
                        }
                    }
                }
                return 0 as ::core::ffi::c_int;
            }
        }
        if offset == 0
            || !vim_isIDc(
                *line.offset((offset - 1 as ::core::ffi::c_int) as isize) as uint8_t
                    as ::core::ffi::c_int,
            )
        {
            *poffset = offset;
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_iswhileofdo_end(
    mut terminated: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut i: ::core::ffi::c_int = 0;
        if terminated != ';' as ::core::ffi::c_int {
            return false_0;
        }
        line = get_cursor_line_ptr();
        p = line;
        while *p as ::core::ffi::c_int != NUL {
            p = cin_skipcomment(p);
            if *p as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
                s = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                if *s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                    && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize))
                {
                    i = p.offset_from(line) as ::core::ffi::c_int;
                    (*curwin.get()).w_cursor.col = i as colnr_T;
                    trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                    if !trypos.is_null() {
                        s = cin_skipcomment(ml_get((*trypos).lnum));
                        if *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                            s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
                        }
                        if cin_starts_with(s, c"while".as_ptr()) != 0 {
                            (*curwin.get()).w_cursor.lnum = (*trypos).lnum;
                            return true_0;
                        }
                    }
                    line = get_cursor_line_ptr();
                    p = line.offset(i as isize);
                }
            }
            if *p as ::core::ffi::c_int != NUL {
                p = p.offset(1);
            }
        }
        return false_0;
    }
}

pub(crate) unsafe extern "C" fn cin_isbreak(
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return (strncmp(p, c"break".as_ptr(), 5 as size_t) == 0 as ::core::ffi::c_int
            && !vim_isIDc(
                *p.offset(5 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            )) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_ends_in(
    mut s: *const ::core::ffi::c_char,
    mut find: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = s;
        let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = strlen(find) as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != NUL {
            p = cin_skipcomment(p);
            if strncmp(p, find, len as size_t) == 0 as ::core::ffi::c_int {
                r = skipwhite(p.offset(len as isize));
                if cin_nocode(r) {
                    return true_0;
                }
            }
            if *p as ::core::ffi::c_int != NUL {
                p = p.offset(1);
            }
        }
        return false_0;
    }
}

pub(crate) unsafe extern "C" fn cin_starts_with(
    mut s: *const ::core::ffi::c_char,
    mut word: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut l: size_t = strlen(word);
        return (strncmp(s, word, l) == 0 as ::core::ffi::c_int
            && !vim_isIDc(*s.offset(l as isize) as uint8_t as ::core::ffi::c_int))
            as ::core::ffi::c_int;
    }
}
