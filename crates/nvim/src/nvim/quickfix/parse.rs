//! Matching one line of output against the compiled formats.
//!
//! [`qf_parse_line`] runs the `efm_T` list over a line and, on a match,
//! [`qf_parse_match`] and [`qf_parse_get_fields`] pull out the fields the
//! format named. There is one `qf_parse_fmt_*` handler per conversion, and
//! the multiline prefixes (`%A`/`%C`/`%Z` and friends) are handled by
//! [`qf_parse_multiline_pfx`], which folds a continuation line into the
//! entry the previous line started.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn qf_parse_line(
    mut qfl: *mut qf_list_T,
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fmt_first: *mut efm_T,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut fmt_ptr: *mut efm_T = ::core::ptr::null_mut::<efm_T>();
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut status: ::core::ffi::c_int = 0;
        's_240: {
            loop {
                if (*fmt_start.ptr()).is_null() {
                    fmt_ptr = fmt_first;
                } else {
                    fmt_ptr = fmt_start.get();
                    fmt_start.set(::core::ptr::null_mut::<efm_T>());
                }
                (*fields).valid = true_0 != 0;
                while !fmt_ptr.is_null() {
                    idx = (*fmt_ptr).prefix as uint8_t as ::core::ffi::c_int;
                    status = qf_parse_get_fields(
                        linebuf,
                        linelen,
                        fmt_ptr,
                        fields,
                        (*qfl).qf_multiline as ::core::ffi::c_int,
                        (*qfl).qf_multiscan as ::core::ffi::c_int,
                        &raw mut tail,
                    );
                    if status == QF_NOMEM as ::core::ffi::c_int {
                        return status;
                    }
                    if status == QF_OK as ::core::ffi::c_int {
                        break;
                    }
                    fmt_ptr = (*fmt_ptr).next;
                }
                (*qfl).qf_multiscan = false_0 != 0;
                if fmt_ptr.is_null()
                    || idx == 'D' as ::core::ffi::c_int
                    || idx == 'X' as ::core::ffi::c_int
                {
                    if !fmt_ptr.is_null() {
                        status = qf_parse_dir_pfx(idx, fields, qfl);
                        if status != QF_OK as ::core::ffi::c_int {
                            return status;
                        }
                    }
                    status = qf_parse_line_nomatch(linebuf, linelen, fields);
                    if status != QF_OK as ::core::ffi::c_int {
                        return status;
                    }
                    if fmt_ptr.is_null() {
                        (*qfl).qf_multiignore = false_0 != 0;
                        (*qfl).qf_multiline = (*qfl).qf_multiignore;
                    }
                    break 's_240;
                } else {
                    if (*fmt_ptr).conthere != 0 {
                        fmt_start.set(fmt_ptr);
                    }
                    if !vim_strchr(b"AEWIN\0".as_ptr() as *const ::core::ffi::c_char, idx).is_null()
                    {
                        (*qfl).qf_multiline = true_0 != 0;
                        (*qfl).qf_multiignore = false_0 != 0;
                        break;
                    } else if !vim_strchr(b"CZ\0".as_ptr() as *const ::core::ffi::c_char, idx)
                        .is_null()
                    {
                        status = qf_parse_multiline_pfx(idx, qfl, fields);
                        if status != QF_OK as ::core::ffi::c_int {
                            return status;
                        }
                        break;
                    } else {
                        if vim_strchr(b"OPQ\0".as_ptr() as *const ::core::ffi::c_char, idx)
                            .is_null()
                        {
                            break;
                        }
                        status = qf_parse_file_pfx(idx, fields, qfl, tail);
                        if status != QF_MULTISCAN as ::core::ffi::c_int {
                            break;
                        }
                        let mut s: *mut ::core::ffi::c_char = skipwhite(tail);
                        let mut new_linelen: size_t = strlen(s);
                        if new_linelen >= linelen {
                            return QF_IGNORE_LINE as ::core::ffi::c_int;
                        }
                        linebuf = s;
                        linelen = new_linelen;
                    }
                }
            }
            if (*fmt_ptr).flags as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                if (*qfl).qf_multiline {
                    (*qfl).qf_multiignore = true_0 != 0;
                }
                return QF_IGNORE_LINE as ::core::ffi::c_int;
            }
        }
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_alloc_fields(mut pfields: *mut qffields_T) {
    unsafe {
        (*pfields).namebuf =
            xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
        (*pfields).module =
            xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
        (*pfields).errmsglen = (CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t;
        (*pfields).errmsg = xmalloc((*pfields).errmsglen) as *mut ::core::ffi::c_char;
        (*pfields).pattern =
            xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn qf_free_fields(mut pfields: *mut qffields_T) {
    unsafe {
        xfree((*pfields).namebuf as *mut ::core::ffi::c_void);
        xfree((*pfields).module as *mut ::core::ffi::c_void);
        xfree((*pfields).errmsg as *mut ::core::ffi::c_void);
        xfree((*pfields).pattern as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_f(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
    mut prefix: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        let mut c: ::core::ffi::c_char = *(*rmp).endp[midx as usize];
        *(*rmp).endp[midx as usize] = NUL as ::core::ffi::c_char;
        expand_env((*rmp).startp[midx as usize], (*fields).namebuf, CMDBUFFSIZE);
        *(*rmp).endp[midx as usize] = c;
        if !vim_strchr(b"OPQ\0".as_ptr() as *const ::core::ffi::c_char, prefix).is_null()
            && !os_path_exists((*fields).namebuf)
        {
            return QF_FAIL as ::core::ffi::c_int;
        }
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_b(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        let mut bnr: ::core::ffi::c_int = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
        if buflist_findnr(bnr).is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).bnr = bnr;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_n(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).enr = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_l(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).lnum = atol((*rmp).startp[midx as usize]) as linenr_T;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_e(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).end_lnum = atol((*rmp).startp[midx as usize]) as linenr_T;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_c(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).col = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_k(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).end_col = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_t(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).type_0 = *(*rmp).startp[midx as usize];
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn copy_nonerror_line(
    mut linebuf: *const ::core::ffi::c_char,
    mut linelen: size_t,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if linelen >= (*fields).errmsglen {
            (*fields).errmsg = xrealloc(
                (*fields).errmsg as *mut ::core::ffi::c_void,
                linelen.wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            (*fields).errmsglen = linelen.wrapping_add(1 as size_t);
        }
        xstrlcpy((*fields).errmsg, linebuf, linelen.wrapping_add(1 as size_t));
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_m(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        let mut len: size_t =
            (*rmp).endp[midx as usize].offset_from((*rmp).startp[midx as usize]) as size_t;
        if len >= (*fields).errmsglen {
            (*fields).errmsg = xrealloc(
                (*fields).errmsg as *mut ::core::ffi::c_void,
                len.wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            (*fields).errmsglen = len.wrapping_add(1 as size_t);
        }
        xstrlcpy(
            (*fields).errmsg,
            (*rmp).startp[midx as usize],
            len.wrapping_add(1 as size_t),
        );
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_r(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut tail: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        *tail = (*rmp).startp[midx as usize];
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_p(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).col = 0 as ::core::ffi::c_int;
        let mut match_ptr: *mut ::core::ffi::c_char = (*rmp).startp[midx as usize];
        while match_ptr != (*rmp).endp[midx as usize] {
            (*fields).col += 1;
            if *match_ptr as ::core::ffi::c_int == TAB {
                (*fields).col += 7 as ::core::ffi::c_int;
                (*fields).col -= (*fields).col % 8 as ::core::ffi::c_int;
            }
            match_ptr = match_ptr.offset(1);
        }
        (*fields).col += 1;
        (*fields).use_viscol = true_0 != 0;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_v(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        (*fields).col = atol((*rmp).startp[midx as usize]) as ::core::ffi::c_int;
        (*fields).use_viscol = true_0 != 0;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_s(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        let mut len: size_t =
            (*rmp).endp[midx as usize].offset_from((*rmp).startp[midx as usize]) as size_t;
        len = if len < (1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as size_t {
            len
        } else {
            (1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as size_t
        };
        strcpy(
            (*fields).pattern,
            b"^\\V\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        xstrlcat(
            (*fields).pattern,
            (*rmp).startp[midx as usize],
            len.wrapping_add(4 as size_t),
        );
        *(*fields)
            .pattern
            .offset(len.wrapping_add(3 as size_t) as isize) = '\\' as ::core::ffi::c_char;
        *(*fields)
            .pattern
            .offset(len.wrapping_add(4 as size_t) as isize) = '$' as ::core::ffi::c_char;
        *(*fields)
            .pattern
            .offset(len.wrapping_add(5 as size_t) as isize) = NUL as ::core::ffi::c_char;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_fmt_o(
    mut rmp: *mut regmatch_T,
    mut midx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*rmp).startp[midx as usize].is_null() || (*rmp).endp[midx as usize].is_null() {
            return QF_FAIL as ::core::ffi::c_int;
        }
        let mut len: size_t =
            (*rmp).endp[midx as usize].offset_from((*rmp).startp[midx as usize]) as size_t;
        let mut dsize: size_t = strlen((*fields).module)
            .wrapping_add(len)
            .wrapping_add(1 as size_t);
        dsize = if dsize < 1024 as size_t {
            dsize
        } else {
            1024 as size_t
        };
        xstrlcat((*fields).module, (*rmp).startp[midx as usize], dsize);
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_match(
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fmt_ptr: *mut efm_T,
    mut regmatch: *mut regmatch_T,
    mut fields: *mut qffields_T,
    mut qf_multiline: ::core::ffi::c_int,
    mut qf_multiscan: ::core::ffi::c_int,
    mut tail: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut idx: ::core::ffi::c_char = (*fmt_ptr).prefix;
        if (idx as ::core::ffi::c_int == 'C' as ::core::ffi::c_int
            || idx as ::core::ffi::c_int == 'Z' as ::core::ffi::c_int)
            && qf_multiline == 0
        {
            return QF_FAIL as ::core::ffi::c_int;
        }
        if !vim_strchr(
            b"EWIN\0".as_ptr() as *const ::core::ffi::c_char,
            idx as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            (*fields).type_0 = idx;
        } else {
            (*fields).type_0 = 0 as ::core::ffi::c_char;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < FMT_PATTERNS {
            let mut status: ::core::ffi::c_int = QF_OK as ::core::ffi::c_int;
            let mut midx: ::core::ffi::c_int = (*fmt_ptr).addr[i as usize] as ::core::ffi::c_int;
            if i == 0 as ::core::ffi::c_int && midx > 0 as ::core::ffi::c_int {
                status = qf_parse_fmt_f(regmatch, midx, fields, idx as ::core::ffi::c_int);
            } else if i == FMT_PATTERN_M {
                if (*fmt_ptr).flags as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                    && qf_multiscan == 0
                {
                    status = copy_nonerror_line(linebuf, linelen, fields);
                } else if midx > 0 as ::core::ffi::c_int {
                    status = qf_parse_fmt_m(regmatch, midx, fields);
                }
            } else if i == FMT_PATTERN_R && midx > 0 as ::core::ffi::c_int {
                status = qf_parse_fmt_r(regmatch, midx, tail);
            } else if midx > 0 as ::core::ffi::c_int {
                status = (*qf_parse_fmt.ptr())[i as usize].expect("non-null function pointer")(
                    regmatch, midx, fields,
                );
            }
            if status != QF_OK as ::core::ffi::c_int {
                return status;
            }
            i += 1;
        }
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_get_fields(
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fmt_ptr: *mut efm_T,
    mut fields: *mut qffields_T,
    mut qf_multiline: ::core::ffi::c_int,
    mut qf_multiscan: ::core::ffi::c_int,
    mut tail: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if qf_multiscan != 0
            && vim_strchr(
                b"OPQ\0".as_ptr() as *const ::core::ffi::c_char,
                (*fmt_ptr).prefix as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            return QF_FAIL as ::core::ffi::c_int;
        }
        *(*fields).namebuf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        (*fields).bnr = 0 as ::core::ffi::c_int;
        *(*fields).module.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        *(*fields).pattern.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        if qf_multiscan == 0 {
            *(*fields).errmsg.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        (*fields).lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*fields).end_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*fields).col = 0 as ::core::ffi::c_int;
        (*fields).end_col = 0 as ::core::ffi::c_int;
        (*fields).use_viscol = false_0 != 0;
        (*fields).enr = -1 as ::core::ffi::c_int;
        (*fields).type_0 = 0 as ::core::ffi::c_char;
        *tail = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        regmatch.rm_ic = true_0 != 0;
        regmatch.regprog = (*fmt_ptr).prog;
        let mut r: bool = vim_regexec(&raw mut regmatch, linebuf, 0 as colnr_T);
        (*fmt_ptr).prog = regmatch.regprog;
        let mut status: ::core::ffi::c_int = QF_FAIL as ::core::ffi::c_int;
        if r {
            status = qf_parse_match(
                linebuf,
                linelen,
                fmt_ptr,
                &raw mut regmatch,
                fields,
                qf_multiline,
                qf_multiscan,
                tail,
            );
        }
        return status;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_dir_pfx(
    mut idx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
    mut qfl: *mut qf_list_T,
) -> ::core::ffi::c_int {
    unsafe {
        if idx == 'D' as ::core::ffi::c_int {
            if *(*fields).namebuf as ::core::ffi::c_int == NUL {
                emsg(gettext(b"E379: Missing or empty directory name\0".as_ptr()
                    as *const ::core::ffi::c_char));
                return QF_FAIL as ::core::ffi::c_int;
            }
            (*qfl).qf_directory = qf_push_dir(
                (*fields).namebuf,
                &raw mut (*qfl).qf_dir_stack,
                false_0 != 0,
            );
            if (*qfl).qf_directory.is_null() {
                return QF_FAIL as ::core::ffi::c_int;
            }
        } else if idx == 'X' as ::core::ffi::c_int {
            (*qfl).qf_directory = qf_pop_dir(&raw mut (*qfl).qf_dir_stack);
        }
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_file_pfx(
    mut idx: ::core::ffi::c_int,
    mut fields: *mut qffields_T,
    mut qfl: *mut qf_list_T,
    mut tail: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        (*fields).valid = false_0 != 0;
        if *(*fields).namebuf as ::core::ffi::c_int == NUL
            || os_path_exists((*fields).namebuf) as ::core::ffi::c_int != 0
        {
            if *(*fields).namebuf as ::core::ffi::c_int != 0 && idx == 'P' as ::core::ffi::c_int {
                (*qfl).qf_currfile = qf_push_dir(
                    (*fields).namebuf,
                    &raw mut (*qfl).qf_file_stack,
                    true_0 != 0,
                );
            } else if idx == 'Q' as ::core::ffi::c_int {
                (*qfl).qf_currfile = qf_pop_dir(&raw mut (*qfl).qf_file_stack);
            }
            *(*fields).namebuf = NUL as ::core::ffi::c_char;
            if !tail.is_null() && *tail as ::core::ffi::c_int != 0 {
                (*qfl).qf_multiscan = true_0 != 0;
                return QF_MULTISCAN as ::core::ffi::c_int;
            }
        }
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_parse_line_nomatch(
    mut linebuf: *mut ::core::ffi::c_char,
    mut linelen: size_t,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        *(*fields).namebuf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        (*fields).lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*fields).valid = false_0 != 0;
        return copy_nonerror_line(linebuf, linelen, fields);
    }
}

pub(crate) unsafe extern "C" fn qf_parse_multiline_pfx(
    mut idx: ::core::ffi::c_int,
    mut qfl: *mut qf_list_T,
    mut fields: *mut qffields_T,
) -> ::core::ffi::c_int {
    unsafe {
        if !(*qfl).qf_multiignore {
            let mut qfprev: *mut qfline_T = (*qfl).qf_last;
            if qfprev.is_null() {
                return QF_FAIL as ::core::ffi::c_int;
            }
            if *(*fields).errmsg != 0 {
                let mut textlen: size_t = strlen((*qfprev).qf_text);
                let mut errlen: size_t = strlen((*fields).errmsg);
                (*qfprev).qf_text = xrealloc(
                    (*qfprev).qf_text as *mut ::core::ffi::c_void,
                    textlen.wrapping_add(errlen).wrapping_add(2 as size_t),
                ) as *mut ::core::ffi::c_char;
                *(*qfprev).qf_text.offset(textlen as isize) = '\n' as ::core::ffi::c_char;
                strcpy(
                    (*qfprev)
                        .qf_text
                        .offset(textlen as isize)
                        .offset(1 as ::core::ffi::c_int as isize),
                    (*fields).errmsg,
                );
            }
            if (*qfprev).qf_nr == -1 as ::core::ffi::c_int {
                (*qfprev).qf_nr = (*fields).enr;
            }
            if vim_isprintc((*fields).type_0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && (*qfprev).qf_type == 0
            {
                (*qfprev).qf_type = (*fields).type_0;
            }
            if (*qfprev).qf_lnum == 0 {
                (*qfprev).qf_lnum = (*fields).lnum;
            }
            if (*qfprev).qf_end_lnum == 0 {
                (*qfprev).qf_end_lnum = (*fields).end_lnum;
            }
            if (*qfprev).qf_col == 0 {
                (*qfprev).qf_col = (*fields).col;
                (*qfprev).qf_viscol = (*fields).use_viscol as ::core::ffi::c_char;
            }
            if (*qfprev).qf_end_col == 0 {
                (*qfprev).qf_end_col = (*fields).end_col;
            }
            if (*qfprev).qf_fnum == 0 {
                (*qfprev).qf_fnum = qf_get_fnum(
                    qfl,
                    (*qfl).qf_directory,
                    if *(*fields).namebuf as ::core::ffi::c_int != 0
                        || !(*qfl).qf_directory.is_null()
                    {
                        (*fields).namebuf
                    } else if !(*qfl).qf_currfile.is_null()
                        && (*fields).valid as ::core::ffi::c_int != 0
                    {
                        (*qfl).qf_currfile
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    },
                );
            }
        }
        if idx == 'Z' as ::core::ffi::c_int {
            (*qfl).qf_multiignore = false_0 != 0;
            (*qfl).qf_multiline = (*qfl).qf_multiignore;
        }
        line_breakcheck();
        return QF_IGNORE_LINE as ::core::ffi::c_int;
    }
}
