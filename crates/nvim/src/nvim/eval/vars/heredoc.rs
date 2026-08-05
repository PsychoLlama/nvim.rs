//! `=<< MARKER` -- the here-document form of an assignment.
//!
//! `heredoc_get` collects the lines and applies `trim`'s indent rules; the
//! two `eval_*_expr_in_str` implement `eval`'s `{expr}` interpolation, which
//! is the only thing in the file that evaluates its own input.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn eval_one_expr_in_str(
    mut p: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
    mut evaluate: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut block_start: *mut ::core::ffi::c_char =
            skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
        let mut block_end: *mut ::core::ffi::c_char = block_start;
        if *block_start == NUL {
            semsg(
                gettext(&raw const e_missing_close_curly_str as *const ::core::ffi::c_char),
                p,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if skip_expr(&raw mut block_end, ::core::ptr::null_mut::<evalarg_T>()) == FAIL {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        block_end = skipwhite(block_end);
        if *block_end as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_missing_close_curly_str as *const ::core::ffi::c_char),
                p,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if evaluate {
            *block_end = NUL;
            let mut expr_val: *mut ::core::ffi::c_char =
                eval_to_string(block_start, false_0 != 0, false_0 != 0);
            *block_end = '}' as ::core::ffi::c_char;
            if expr_val.is_null() {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            ga_concat(gap, expr_val);
            xfree(expr_val as *mut ::core::ffi::c_void);
        }
        return block_end.offset(1 as ::core::ffi::c_int as isize);
    }
}

unsafe extern "C" fn eval_all_expr_in_str(
    mut str: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        let mut p: *mut ::core::ffi::c_char = str;
        while *p != NUL {
            let mut escaped_brace: bool = false_0 != 0;
            let mut lit_start: *mut ::core::ffi::c_char = p;
            while *p as ::core::ffi::c_int != '{' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                && *p != NUL
            {
                p = p.offset(1);
            }
            if *p != NUL
                && *p as ::core::ffi::c_int
                    == *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            {
                p = p.offset(1);
                escaped_brace = true_0 != 0;
            } else if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                semsg(
                    gettext(&raw const e_stray_closing_curly_str as *const ::core::ffi::c_char),
                    str,
                );
                ga_clear(&raw mut ga);
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            ga_concat_len(&raw mut ga, lit_start, p.offset_from(lit_start) as size_t);
            if *p == NUL {
                break;
            }
            if escaped_brace {
                p = p.offset(1);
            } else {
                p = eval_one_expr_in_str(p, &raw mut ga, true_0 != 0);
                if p.is_null() {
                    ga_clear(&raw mut ga);
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn heredoc_get(
    mut eap: *mut exarg_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut script_get: bool,
) -> *mut list_T {
    unsafe {
        let mut marker: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut marker_indent_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut text_indent_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut text_indent: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dot: [::core::ffi::c_char; 2] =
            ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b".\0");
        let mut heredoc_in_string: bool = false_0 != 0;
        let mut line_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut nl_ptr: *mut ::core::ffi::c_char = vim_strchr(cmd, '\n' as ::core::ffi::c_int);
        if !nl_ptr.is_null() {
            heredoc_in_string = true_0 != 0;
            line_arg = nl_ptr.offset(1 as ::core::ffi::c_int as isize);
            *nl_ptr = NUL;
        } else if (*eap).ea_getline.is_none() {
            emsg(gettext(e_cannot_use_heredoc_here.as_ptr()));
            return ::core::ptr::null_mut::<list_T>();
        }
        cmd = skipwhite(cmd);
        let mut evalstr: bool = false_0 != 0;
        let mut eval_failed: bool = false_0 != 0;
        loop {
            if strncmp(
                cmd,
                b"trim\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
                && (*cmd.offset(4 as ::core::ffi::c_int as isize) == NUL
                    || ascii_iswhite(
                        *cmd.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0)
            {
                cmd = skipwhite(cmd.offset(4 as ::core::ffi::c_int as isize));
                let mut p: *mut ::core::ffi::c_char = *(*eap).cmdlinep;
                while ascii_iswhite(*p as ::core::ffi::c_int) {
                    p = p.offset(1);
                    marker_indent_len += 1;
                }
                text_indent_len = -1 as ::core::ffi::c_int;
            } else {
                if !(strncmp(
                    cmd,
                    b"eval\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && (*cmd.offset(4 as ::core::ffi::c_int as isize) == NUL
                        || ascii_iswhite(
                            *cmd.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0))
                {
                    break;
                }
                cmd = skipwhite(cmd.offset(4 as ::core::ffi::c_int as isize));
                evalstr = true_0 != 0;
            }
        }
        let comment_char: ::core::ffi::c_char = '"' as ::core::ffi::c_char;
        if *cmd != NUL && *cmd as ::core::ffi::c_int != comment_char as ::core::ffi::c_int {
            marker = skipwhite(cmd);
            let mut p_0: *mut ::core::ffi::c_char = skiptowhite(marker);
            if *skipwhite(p_0) != NUL
                && *skipwhite(p_0) as ::core::ffi::c_int != comment_char as ::core::ffi::c_int
            {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    p_0,
                );
                return ::core::ptr::null_mut::<list_T>();
            }
            *p_0 = NUL;
            if !script_get
                && *(*__ctype_b_loc()).offset(*marker as uint8_t as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISlower as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                emsg(gettext(
                    b"E221: Marker cannot start with lower case letter\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return ::core::ptr::null_mut::<list_T>();
            }
        } else if script_get {
            marker = &raw mut dot as *mut ::core::ffi::c_char;
        } else {
            emsg(gettext(
                b"E172: Missing marker\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return ::core::ptr::null_mut::<list_T>();
        }
        let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut l: *mut list_T = tv_list_alloc(0 as ptrdiff_t);
        loop {
            let mut mi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut ti: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if heredoc_in_string {
                if *line_arg == NUL {
                    if !script_get {
                        semsg(gettext(e_missing_end_marker_str.as_ptr()), marker);
                    }
                    break;
                } else {
                    theline = line_arg;
                    let mut next_line: *mut ::core::ffi::c_char =
                        vim_strchr(theline, '\n' as ::core::ffi::c_int);
                    if next_line.is_null() {
                        line_arg = line_arg.offset(strlen(line_arg) as isize);
                    } else {
                        *next_line = NUL;
                        line_arg = next_line.offset(1 as ::core::ffi::c_int as isize);
                    }
                }
            } else {
                xfree(theline as *mut ::core::ffi::c_void);
                theline = (*eap).ea_getline.expect("non-null function pointer")(
                    NUL as ::core::ffi::c_int,
                    (*eap).cookie,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                if theline.is_null() {
                    if !script_get {
                        semsg(gettext(e_missing_end_marker_str.as_ptr()), marker);
                    }
                    break;
                }
            }
            if marker_indent_len > 0 as ::core::ffi::c_int
                && strncmp(theline, *(*eap).cmdlinep, marker_indent_len as size_t)
                    == 0 as ::core::ffi::c_int
            {
                mi = marker_indent_len;
            }
            if strcmp(marker, theline.offset(mi as isize)) == 0 as ::core::ffi::c_int {
                break;
            }
            if eval_failed {
                continue;
            }
            if text_indent_len == -1 as ::core::ffi::c_int && *theline != NUL {
                let mut p_1: *mut ::core::ffi::c_char = theline;
                text_indent_len = 0 as ::core::ffi::c_int;
                while ascii_iswhite(*p_1 as ::core::ffi::c_int) {
                    p_1 = p_1.offset(1);
                    text_indent_len += 1;
                }
                text_indent = xmemdupz(
                    theline as *const ::core::ffi::c_void,
                    text_indent_len as size_t,
                ) as *mut ::core::ffi::c_char;
            }
            if !text_indent.is_null() {
                ti = 0 as ::core::ffi::c_int;
                while ti < text_indent_len {
                    if *theline.offset(ti as isize) as ::core::ffi::c_int
                        != *text_indent.offset(ti as isize) as ::core::ffi::c_int
                    {
                        break;
                    }
                    ti += 1;
                }
            }
            let mut str: *mut ::core::ffi::c_char = theline.offset(ti as isize);
            if evalstr as ::core::ffi::c_int != 0 && (*eap).skip == 0 {
                str = eval_all_expr_in_str(str);
                if str.is_null() {
                    eval_failed = true_0 != 0;
                } else {
                    tv_list_append_allocated_string(l, str);
                }
            } else {
                tv_list_append_string(l, str, -1 as ssize_t);
            }
        }
        if heredoc_in_string {
            (*eap).nextcmd = line_arg;
        } else {
            xfree(theline as *mut ::core::ffi::c_void);
        }
        xfree(text_indent as *mut ::core::ffi::c_void);
        if eval_failed {
            tv_list_free(l);
            return ::core::ptr::null_mut::<list_T>();
        }
        return l;
    }
}
