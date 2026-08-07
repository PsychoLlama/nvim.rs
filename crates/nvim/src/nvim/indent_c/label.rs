//! Indent amounts read off a line rather than computed.
//!
//! `skip_label` and `get_indent_nolabel` answer "how indented is this line, not
//! counting a jump label in front of it"; `after_label` is the text past one.
//! `cin_first_id_amount` is 'cinoptions' `+`'s continuation base -- the column of
//! the first identifier after a type -- and `cin_get_equal_amount` the column
//! after a trailing `=`, which is what a `\`-continued assignment lines up
//! with.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn after_label(
    mut l: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        while *l != 0 {
            if *l as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                if *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                {
                    l = l.offset(1);
                } else if !cin_iscase(l.offset(1 as ::core::ffi::c_int as isize), false_0 != 0) {
                    break;
                }
            } else if *l as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                && *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
                && *l.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\'' as ::core::ffi::c_int
            {
                l = l.offset(2 as ::core::ffi::c_int as isize);
            }
            l = l.offset(1);
        }
        if *l as ::core::ffi::c_int == NUL {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        l = cin_skipcomment(l.offset(1 as ::core::ffi::c_int as isize));
        if *l as ::core::ffi::c_int == NUL {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return l;
    }
}

pub(crate) unsafe extern "C" fn get_indent_nolabel(mut lnum: linenr_T) -> ::core::ffi::c_int {
    unsafe {
        let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut fp: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut col: colnr_T = 0;
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        l = ml_get(lnum);
        p = after_label(l);
        if p.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        fp.col = p.offset_from(l) as colnr_T;
        fp.lnum = lnum;
        getvcol(
            curwin.get(),
            &raw mut fp,
            &raw mut col,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        return col;
    }
}

pub(crate) unsafe extern "C" fn skip_label(
    mut lnum: linenr_T,
    mut pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut amount: ::core::ffi::c_int = 0;
        let mut cursor_save: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        cursor_save = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum = lnum;
        l = get_cursor_line_ptr();
        if cin_iscase(l, false_0 != 0) as ::core::ffi::c_int != 0
            || cin_isscopedecl(l) as ::core::ffi::c_int != 0
            || cin_islabel() as ::core::ffi::c_int != 0
        {
            amount = get_indent_nolabel(lnum);
            l = after_label(get_cursor_line_ptr());
            if l.is_null() {
                l = get_cursor_line_ptr();
            }
        } else {
            amount = get_indent();
            l = get_cursor_line_ptr();
        }
        *pp = l;
        (*curwin.get()).w_cursor = cursor_save;
        return amount;
    }
}

pub(crate) unsafe extern "C" fn cin_first_id_amount() -> ::core::ffi::c_int {
    unsafe {
        let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = 0;
        let mut fp: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut col: colnr_T = 0;
        line = get_cursor_line_ptr();
        p = skipwhite(line);
        len = skiptowhite(p).offset_from(p) as ::core::ffi::c_int;
        if len == 6 as ::core::ffi::c_int
            && strncmp(
                p,
                b"static\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
            len = skiptowhite(p).offset_from(p) as ::core::ffi::c_int;
        }
        if len == 6 as ::core::ffi::c_int
            && strncmp(
                p,
                b"struct\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
        } else if len == 4 as ::core::ffi::c_int
            && strncmp(
                p,
                b"enum\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
        } else if len == 8 as ::core::ffi::c_int
            && strncmp(
                p,
                b"unsigned\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            || len == 6 as ::core::ffi::c_int
                && strncmp(
                    p,
                    b"signed\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) == 0 as ::core::ffi::c_int
        {
            s = skipwhite(p.offset(len as isize));
            if strncmp(
                s,
                b"int\0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_iswhite(*s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                || strncmp(
                    s,
                    b"long\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && ascii_iswhite(
                        *s.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                || strncmp(
                    s,
                    b"short\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && ascii_iswhite(
                        *s.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                || strncmp(
                    s,
                    b"char\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && ascii_iswhite(
                        *s.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
            {
                p = s;
            }
        }
        len = 0 as ::core::ffi::c_int;
        while vim_isIDc(*p.offset(len as isize) as uint8_t as ::core::ffi::c_int) {
            len += 1;
        }
        if len == 0 as ::core::ffi::c_int
            || !ascii_iswhite(*p.offset(len as isize) as ::core::ffi::c_int)
            || cin_nocode(p) != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        p = skipwhite(p.offset(len as isize));
        fp.lnum = (*curwin.get()).w_cursor.lnum;
        fp.col = p.offset_from(line) as colnr_T;
        getvcol(
            curwin.get(),
            &raw mut fp,
            &raw mut col,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        return col;
    }
}

pub(crate) unsafe extern "C" fn cin_get_equal_amount(mut lnum: linenr_T) -> ::core::ffi::c_int {
    unsafe {
        let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut col: colnr_T = 0;
        let mut fp: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        if lnum > 1 as linenr_T {
            line = ml_get(lnum - 1 as linenr_T);
            if *line as ::core::ffi::c_int != NUL
                && *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize)
                    as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                return -1 as ::core::ffi::c_int;
            }
        }
        s = ml_get(lnum);
        line = s;
        while *s as ::core::ffi::c_int != NUL
            && vim_strchr(
                b"=;{}\"'\0".as_ptr() as *const ::core::ffi::c_char,
                *s as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            if cin_iscomment(s) != 0 {
                s = cin_skipcomment(s);
            } else {
                s = s.offset(1);
            }
        }
        if *s as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        s = skipwhite(s.offset(1 as ::core::ffi::c_int as isize));
        if cin_nocode(s) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            s = s.offset(1);
        }
        fp.lnum = lnum;
        fp.col = s.offset_from(line) as colnr_T;
        getvcol(
            curwin.get(),
            &raw mut fp,
            &raw mut col,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        return col;
    }
}
