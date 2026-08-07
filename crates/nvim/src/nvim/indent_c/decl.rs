//! Labels, declarations and the preprocessor.
//!
//! `cin_islabel` decides whether the current line is a jump label -- which
//! 'cinoptions' `L` moves to the left margin -- and has to look *backwards* to
//! do it, because `foo:` is only a label if the statement before it ended.
//! `cin_isfuncdecl` is the K&R-parameter test, `cin_isinit` the
//! `= {`/`enum` one, and `cin_ispreproc_cont` walks a `\`-continued `#define`
//! back to its first line so that the scan does not stop inside one.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn cin_islabel_skip(mut s: *mut *const ::core::ffi::c_char) -> bool {
    unsafe {
        if !vim_isIDc(**s as uint8_t as ::core::ffi::c_int) {
            return false;
        }
        while vim_isIDc(**s as uint8_t as ::core::ffi::c_int) {
            *s = (*s).offset(utfc_ptr2len(*s) as isize);
        }
        *s = cin_skipcomment(*s);
        return **s as ::core::ffi::c_int == ':' as ::core::ffi::c_int && {
            *s = (*s).offset(1);
            **s as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        };
    }
}

pub(crate) unsafe extern "C" fn cin_islabel() -> bool {
    unsafe {
        let mut s: *const ::core::ffi::c_char = cin_skipcomment(get_cursor_line_ptr());
        if cin_isdefault(s) != 0 {
            return false;
        }
        if cin_isscopedecl(s) {
            return false;
        }
        if !cin_islabel_skip(&raw mut s) {
            return false;
        }
        if !ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>()).is_null() {
            return false;
        }
        let mut cursor_save: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        cursor_save = (*curwin.get()).w_cursor;
        while (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
            (*curwin.get()).w_cursor.lnum -= 1;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            trypos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
            if !trypos.is_null() {
                (*curwin.get()).w_cursor = *trypos;
            }
            line = get_cursor_line_ptr();
            if cin_ispreproc(line) != 0 {
                continue;
            }
            line = cin_skipcomment(line);
            if *line as ::core::ffi::c_int == NUL {
                continue;
            }
            (*curwin.get()).w_cursor = cursor_save;
            if cin_isterminated(line, true_0, false_0) as ::core::ffi::c_int != 0
                || cin_isscopedecl(line) as ::core::ffi::c_int != 0
                || cin_iscase(line, true) as ::core::ffi::c_int != 0
                || cin_islabel_skip(&raw mut line) as ::core::ffi::c_int != 0
                    && cin_nocode(line) != 0
            {
                return true;
            }
            return false;
        }
        (*curwin.get()).w_cursor = cursor_save;
        return true;
    }
}

unsafe extern "C" fn cin_is_compound_init(mut s: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut p: *const ::core::ffi::c_char = s;
        let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        while *p != 0 {
            if *p as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                r = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
                p = r;
            } else if strncmp(p, c"return".as_ptr(), 6 as size_t) == 0
                && !vim_isIDc(*p.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                && (p == s
                    || p > s
                        && !vim_isIDc(
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ))
            {
                r = cin_skipcomment(p.offset(6 as ::core::ffi::c_int as isize));
                p = r;
            } else {
                p = cin_skip_comment_and_string(p.offset(1 as ::core::ffi::c_int as isize));
            }
        }
        if r.is_null() {
            return false;
        }
        p = r;
        if cin_nocode(p) != 0 {
            return true;
        }
        if *p as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
        }
        if *p as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
            let mut open_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            loop {
                p = cin_skip_comment_and_string(p.offset(1 as ::core::ffi::c_int as isize));
                if cin_nocode(p) != 0 {
                    return true;
                }
                open_count += (*p as ::core::ffi::c_int == '(' as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    - (*p as ::core::ffi::c_int == ')' as ::core::ffi::c_int) as ::core::ffi::c_int;
                if open_count == 0 {
                    break;
                }
            }
            p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
            if cin_nocode(p) != 0 {
                return true;
            }
        }
        while *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
            p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
        }
        return cin_nocode(p) != 0;
    }
}

pub(crate) unsafe extern "C" fn cin_isinit() -> bool {
    unsafe {
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        static skip: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
            c"static".as_ptr() as *mut ::core::ffi::c_char,
            c"public".as_ptr() as *mut ::core::ffi::c_char,
            c"protected".as_ptr() as *mut ::core::ffi::c_char,
            c"private".as_ptr() as *mut ::core::ffi::c_char,
        ]);
        s = cin_skipcomment(get_cursor_line_ptr());
        if cin_starts_with(s, c"typedef".as_ptr()) != 0 {
            s = cin_skipcomment(s.offset(7 as ::core::ffi::c_int as isize));
        }
        loop {
            let mut i: ::core::ffi::c_int = 0;
            let mut l: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            while i < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
                .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
                        .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int
            {
                l = strlen((*skip.ptr())[i as usize]) as ::core::ffi::c_int;
                if cin_starts_with(s, (*skip.ptr())[i as usize]) != 0 {
                    s = cin_skipcomment(s.offset(l as isize));
                    l = 0 as ::core::ffi::c_int;
                    break;
                } else {
                    i += 1;
                }
            }
            if l != 0 as ::core::ffi::c_int {
                break;
            }
        }
        if cin_starts_with(s, c"enum".as_ptr()) != 0 {
            return true;
        }
        return cin_is_compound_init(s);
    }
}

unsafe extern "C" fn cin_ispreproc(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if *skipwhite(s) as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
            return true_0;
        }
        return false_0;
    }
}

pub(crate) unsafe extern "C" fn cin_ispreproc_cont(
    mut pp: *mut *const ::core::ffi::c_char,
    mut lnump: *mut linenr_T,
    mut amount: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut line: *const ::core::ffi::c_char = *pp;
        let mut lnum: linenr_T = *lnump;
        let mut retval: ::core::ffi::c_int = false_0;
        let mut candidate_amount: ::core::ffi::c_int = *amount;
        if *line as ::core::ffi::c_int != NUL
            && *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
        {
            candidate_amount = get_indent_lnum(lnum);
        }
        loop {
            if cin_ispreproc(line) != 0 {
                retval = true_0;
                *lnump = lnum;
                break;
            } else {
                if lnum == 1 as linenr_T {
                    break;
                }
                lnum -= 1;
                line = ml_get(lnum);
                if *line as ::core::ffi::c_int == NUL
                    || *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize)
                        as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                {
                    break;
                }
            }
        }
        if lnum != *lnump {
            *pp = ml_get(*lnump);
        }
        if retval != 0 {
            *amount = candidate_amount;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn cin_isfuncdecl(
    mut sp: *mut *const ::core::ffi::c_char,
    mut first_lnum: linenr_T,
    mut min_lnum: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut lnum: linenr_T = first_lnum;
        let mut save_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut retval: ::core::ffi::c_int = false_0;
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut just_started: ::core::ffi::c_int = true_0;
        if sp.is_null() {
            s = ml_get(lnum);
        } else {
            s = *sp;
        }
        (*curwin.get()).w_cursor.lnum = lnum;
        if find_last_paren(s, '(' as ::core::ffi::c_char, ')' as ::core::ffi::c_char) != 0 && {
            trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
            !trypos.is_null()
        } {
            lnum = (*trypos).lnum;
            if lnum < min_lnum {
                (*curwin.get()).w_cursor.lnum = save_lnum;
                return false_0;
            }
            s = ml_get(lnum);
        }
        (*curwin.get()).w_cursor.lnum = save_lnum;
        if cin_ispreproc(s) != 0 {
            return false_0;
        }
        while *s as ::core::ffi::c_int != 0
            && *s as ::core::ffi::c_int != '(' as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != ';' as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != '"' as ::core::ffi::c_int
        {
            if cin_iscomment(s) != 0 {
                s = cin_skipcomment(s);
            } else if *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                {
                    s = s.offset(2 as ::core::ffi::c_int as isize);
                } else {
                    return false_0;
                }
            } else {
                s = s.offset(1);
            }
        }
        if *s as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
            return false_0;
        }
        while *s as ::core::ffi::c_int != 0
            && *s as ::core::ffi::c_int != ';' as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != '"' as ::core::ffi::c_int
        {
            if *s as ::core::ffi::c_int == ')' as ::core::ffi::c_int
                && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                lnum = first_lnum - 1 as linenr_T;
                s = ml_get(lnum);
                if *s as ::core::ffi::c_int == NUL
                    || *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                {
                    retval = true_0;
                }
                break;
            } else if *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
                || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || cin_nocode(s) != 0
            {
                let mut comma: ::core::ffi::c_int =
                    (*s as ::core::ffi::c_int == ',' as ::core::ffi::c_int) as ::core::ffi::c_int;
                while lnum < (*curbuf.get()).b_ml.ml_line_count {
                    lnum += 1;
                    s = ml_get(lnum);
                    if cin_ispreproc(s) == 0 {
                        break;
                    }
                }
                if lnum >= (*curbuf.get()).b_ml.ml_line_count {
                    break;
                }
                s = skipwhite(s);
                if just_started == 0
                    && (comma == 0
                        && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                        && *s as ::core::ffi::c_int != ')' as ::core::ffi::c_int)
                {
                    break;
                }
                just_started = false_0;
            } else if cin_iscomment(s) != 0 {
                s = cin_skipcomment(s);
            } else {
                s = s.offset(1);
                just_started = false_0;
            }
        }
        if lnum != first_lnum && !sp.is_null() {
            *sp = ml_get(first_lnum);
        }
        return retval;
    }
}
