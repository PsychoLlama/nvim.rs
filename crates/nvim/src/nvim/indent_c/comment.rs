//! Where a comment or a string starts, and how to step over one.
//!
//! Every recogniser in this family has to answer over *code*, so each one opens
//! by calling `cin_skipcomment`, and the ones that walk a whole line call
//! `skip_string` too.  The `find_start_*` half is the other direction: given the
//! cursor, `findmatchlimit` backwards for the `/*` or the `R"delim(` that
//! encloses it, bounded by 'cinoptions' `*N` (`b_ind_maxcomment`).
//! `ind_find_start_CORS` is the pair asked at once -- Comment Or Raw String --
//! and answers whichever starts later.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ind_find_start_comment() -> *mut pos_T {
    unsafe {
        return find_start_comment((*curbuf.get()).b_ind_maxcomment);
    }
}

pub unsafe extern "C" fn find_start_comment(mut ind_maxcomment: ::core::ffi::c_int) -> *mut pos_T {
    unsafe {
        let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut cur_maxcomment: int64_t = ind_maxcomment as int64_t;
        loop {
            pos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                '*' as ::core::ffi::c_int,
                FM_BACKWARD as ::core::ffi::c_int,
                cur_maxcomment,
            );
            if pos.is_null() {
                break;
            }
            if is_pos_in_string(ml_get((*pos).lnum), (*pos).col) == 0 {
                break;
            }
            cur_maxcomment =
                ((*curwin.get()).w_cursor.lnum - (*pos).lnum - 1 as linenr_T) as int64_t;
            if cur_maxcomment > 0 as int64_t {
                continue;
            }
            pos = ::core::ptr::null_mut::<pos_T>();
            break;
        }
        return pos;
    }
}

pub(crate) unsafe extern "C" fn ind_find_start_CORS(mut is_raw: *mut linenr_T) -> *mut pos_T {
    unsafe {
        static comment_pos_copy: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        });
        let mut comment_pos: *mut pos_T = find_start_comment((*curbuf.get()).b_ind_maxcomment);
        if !comment_pos.is_null() {
            comment_pos_copy.set(*comment_pos);
            comment_pos = comment_pos_copy.ptr();
        }
        let mut rs_pos: *mut pos_T = find_start_rawstring((*curbuf.get()).b_ind_maxcomment);
        if comment_pos.is_null()
            || !rs_pos.is_null() && lt(*rs_pos, *comment_pos) as ::core::ffi::c_int != 0
        {
            if !is_raw.is_null() && !rs_pos.is_null() {
                *is_raw = (*rs_pos).lnum;
            }
            return rs_pos;
        }
        return comment_pos;
    }
}

pub(crate) unsafe extern "C" fn find_start_rawstring(
    mut ind_maxcomment: ::core::ffi::c_int,
) -> *mut pos_T {
    unsafe {
        let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut cur_maxcomment: ::core::ffi::c_int = ind_maxcomment;
        loop {
            pos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                'R' as ::core::ffi::c_int,
                FM_BACKWARD as ::core::ffi::c_int,
                cur_maxcomment as int64_t,
            );
            if pos.is_null() {
                break;
            }
            if is_pos_in_string(ml_get((*pos).lnum), (*pos).col) == 0 {
                break;
            }
            cur_maxcomment =
                ((*curwin.get()).w_cursor.lnum - (*pos).lnum - 1 as linenr_T) as ::core::ffi::c_int;
            if cur_maxcomment > 0 as ::core::ffi::c_int {
                continue;
            }
            pos = ::core::ptr::null_mut::<pos_T>();
            break;
        }
        return pos;
    }
}

pub(crate) unsafe extern "C" fn skip_string(
    mut p: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        loop {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\'' as ::core::ffi::c_int
            {
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                    break;
                }
                i = 2 as ::core::ffi::c_int;
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    i += 1;
                    while ascii_isdigit(
                        *p.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    ) {
                        i += 1;
                    }
                }
                if !(*p.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int != NUL
                    && *p.offset(i as isize) as ::core::ffi::c_int == '\'' as ::core::ffi::c_int)
                {
                    break;
                }
                p = p.offset(i as isize);
            } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '"' as ::core::ffi::c_int
            {
                p = p.offset(1);
                while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    {
                        p = p.offset(1);
                    } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                    {
                        break;
                    }
                    p = p.offset(1);
                }
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '"' as ::core::ffi::c_int
                {
                    break;
                }
            } else {
                if !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'R' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int)
                {
                    break;
                }
                let mut delim: *const ::core::ffi::c_char =
                    p.offset(2 as ::core::ffi::c_int as isize);
                let mut paren: *const ::core::ffi::c_char =
                    vim_strchr(delim, '(' as ::core::ffi::c_int);
                if paren.is_null() {
                    break;
                }
                let delim_len: ptrdiff_t = paren.offset_from(delim);
                p = p.offset(3 as ::core::ffi::c_int as isize);
                while *p != 0 {
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ')' as ::core::ffi::c_int
                        && strncmp(
                            p.offset(1 as ::core::ffi::c_int as isize),
                            delim,
                            delim_len as size_t,
                        ) == 0 as ::core::ffi::c_int
                        && *p.offset((delim_len + 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                            == '"' as ::core::ffi::c_int
                    {
                        p = p.offset((delim_len + 1 as ptrdiff_t) as isize);
                        break;
                    } else {
                        p = p.offset(1);
                    }
                }
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '"' as ::core::ffi::c_int
                {
                    break;
                }
            }
            p = p.offset(1);
        }
        if *p == 0 {
            p = p.offset(-1);
        }
        return p;
    }
}

pub unsafe extern "C" fn is_pos_in_string(
    mut line: *const ::core::ffi::c_char,
    mut col: colnr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        p = line;
        while *p as ::core::ffi::c_int != 0 && (p.offset_from(line) as colnr_T) < col {
            p = skip_string(p);
            p = p.offset(1);
        }
        return !(p.offset_from(line) as colnr_T <= col) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_skipcomment(
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        while *s != 0 {
            let mut prev_s: *const ::core::ffi::c_char = s;
            s = skipwhite(s);
            if (*curbuf.get()).b_ind_hash_comment != 0 as ::core::ffi::c_int
                && s != prev_s
                && *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int
            {
                s = s.offset(strlen(s) as isize);
                break;
            } else {
                if *s as ::core::ffi::c_int != '/' as ::core::ffi::c_int {
                    break;
                }
                s = s.offset(1);
                if *s as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                    s = s.offset(strlen(s) as isize);
                    break;
                } else {
                    if *s as ::core::ffi::c_int != '*' as ::core::ffi::c_int {
                        break;
                    }
                    s = s.offset(1);
                    while *s != 0 {
                        if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '/' as ::core::ffi::c_int
                        {
                            s = s.offset(2 as ::core::ffi::c_int as isize);
                            break;
                        } else {
                            s = s.offset(1);
                        }
                    }
                }
            }
        }
        return s;
    }
}

pub(crate) unsafe extern "C" fn cin_nocode(
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return (*cin_skipcomment(s) as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn find_line_comment() -> *mut pos_T {
    unsafe {
        static pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        });
        let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        pos.set((*curwin.get()).w_cursor);
        loop {
            (*pos.ptr()).lnum -= 1;
            if (*pos.ptr()).lnum <= 0 as linenr_T {
                break;
            }
            line = ml_get((*pos.ptr()).lnum);
            p = skipwhite(line);
            if cin_islinecomment(p) != 0 {
                (*pos.ptr()).col = p.offset_from(line) as ::core::ffi::c_int as colnr_T;
                return pos.ptr();
            }
            if *p as ::core::ffi::c_int != NUL {
                break;
            }
        }
        return ::core::ptr::null_mut::<pos_T>();
    }
}

pub(crate) unsafe extern "C" fn cin_skip_comment_and_string(
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut p: *const ::core::ffi::c_char = s;
        loop {
            r = p;
            p = cin_skipcomment(p);
            if *p != 0 {
                p = skip_string(p);
            }
            if p == r {
                break;
            }
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn cin_iscomment(
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int
            && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int)) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn cin_islinecomment(
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}
