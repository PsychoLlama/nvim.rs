//! The remaining Vimscript string builtins.
//!
//! Measurement (`strlen`, `strchars`, `strcharlen`, `strwidth`,
//! `strdisplaywidth`), search (`stridx`, `strridx`), conversion (`str2nr`,
//! `str2list`, `string`, `strtrans`) and transformation (`tolower`, `toupper`,
//! `tr`, `trim`).  `strchar_common` is the character count `strchars()` and
//! `strcharlen()` share, differing only in whether composing characters count
//! separately.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::charset::{
    STR2NR_BIN, STR2NR_FORCE, STR2NR_HEX, STR2NR_OCT, STR2NR_OOCT, STR2NR_QUOTE,
};

pub unsafe extern "C" fn f_str2list(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        let mut p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        while *p as ::core::ffi::c_int != NUL {
            tv_list_append_number((*rettv).vval.v_list, utf_ptr2char(p) as varnumber_T);
            p = p.offset(utf_ptr2len(p) as isize);
        }
    }
}

pub unsafe extern "C" fn f_str2nr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut base: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
        let mut what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            base = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize))
                as ::core::ffi::c_int;
            if base != 2 as ::core::ffi::c_int
                && base != 8 as ::core::ffi::c_int
                && base != 10 as ::core::ffi::c_int
                && base != 16 as ::core::ffi::c_int
            {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_get_bool(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0
            {
                what |= STR2NR_QUOTE;
            }
        }
        let mut p: *mut ::core::ffi::c_char = skipwhite(tv_get_string(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        ));
        let mut isneg: bool = *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
        if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        {
            p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
        }
        match base {
            2 => {
                what |= STR2NR_BIN | STR2NR_FORCE;
            }
            8 => {
                what |= STR2NR_OCT | STR2NR_OOCT | STR2NR_FORCE;
            }
            16 => {
                what |= STR2NR_HEX | STR2NR_FORCE;
            }
            _ => {}
        }
        let mut n: varnumber_T = 0;
        vim_str2nr(
            p,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            what,
            &raw mut n,
            ::core::ptr::null_mut::<uvarnumber_T>(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if isneg {
            (*rettv).vval.v_number = -n;
        } else {
            (*rettv).vval.v_number = n;
        };
    }
}

pub unsafe extern "C" fn f_stridx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let needle: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
        let mut haystack: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        let haystack_start: *const ::core::ffi::c_char = haystack;
        if needle.is_null() || haystack.is_null() {
            return;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut error: bool = false_0 != 0;
            let start_idx: ptrdiff_t = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ptrdiff_t;
            if error as ::core::ffi::c_int != 0 || start_idx >= strlen(haystack) as ptrdiff_t {
                return;
            }
            if start_idx >= 0 as ptrdiff_t {
                haystack = haystack.offset(start_idx as isize);
            }
        }
        let mut pos: *const ::core::ffi::c_char = strstr(haystack, needle);
        if !pos.is_null() {
            (*rettv).vval.v_number = pos.offset_from(haystack_start) as varnumber_T;
        }
    }
}

pub unsafe extern "C" fn f_string(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = encode_tv2string(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<size_t>(),
        );
    }
}

pub unsafe extern "C" fn f_strlen(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = strlen(tv_get_string(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        )) as varnumber_T;
    }
}

unsafe extern "C" fn strchar_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut skipcc: bool,
) {
    unsafe {
        let mut s: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut len: varnumber_T = 0 as varnumber_T;
        let mut func_mb_ptr2char_adv: Option<
            unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
        > = None;
        func_mb_ptr2char_adv = (if skipcc as ::core::ffi::c_int != 0 {
            Some(
                mb_ptr2char_adv
                    as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
        } else {
            Some(
                mb_cptr2char_adv
                    as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
        })
            as Option<unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        while *s as ::core::ffi::c_int != NUL {
            func_mb_ptr2char_adv.expect("non-null function pointer")(&raw mut s);
            len += 1;
        }
        (*rettv).vval.v_number = len;
    }
}

pub unsafe extern "C" fn f_strcharlen(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        strchar_common(argvars, rettv, true_0 != 0);
    }
}

pub unsafe extern "C" fn f_strchars(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut skipcc: varnumber_T = false_0 as varnumber_T;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut error: bool = false_0 != 0;
            skipcc = tv_get_bool_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
            if error {
                return;
            }
            if skipcc < 0 as varnumber_T || skipcc > 1 as varnumber_T {
                semsg(
                    gettext(&raw const e_using_number_as_bool_nr as *const ::core::ffi::c_char),
                    skipcc,
                );
                return;
            }
        }
        strchar_common(argvars, rettv, skipcc != 0);
    }
}

pub unsafe extern "C" fn f_strdisplaywidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let s: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            col = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize))
                as ::core::ffi::c_int;
        }
        (*rettv).vval.v_number =
            (linetabsize_col(col, s as *mut ::core::ffi::c_char) - col) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_strwidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let s: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        (*rettv).vval.v_number = mb_string2cells(s) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_strridx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let needle: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
        let haystack: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        (*rettv).vval.v_number = -1 as varnumber_T;
        if needle.is_null() || haystack.is_null() {
            return;
        }
        let haystack_len: size_t = strlen(haystack);
        let mut end_idx: ptrdiff_t = 0;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            end_idx = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as ptrdiff_t;
            if end_idx < 0 as ptrdiff_t {
                return;
            }
        } else {
            end_idx = haystack_len as ptrdiff_t;
        }
        let mut lastmatch: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if *needle as ::core::ffi::c_int == NUL {
            lastmatch = haystack.offset(end_idx as isize);
        } else {
            let mut rest: *const ::core::ffi::c_char = haystack;
            while *rest as ::core::ffi::c_int != NUL {
                rest = strstr(rest, needle);
                if rest.is_null() || rest > haystack.offset(end_idx as isize) {
                    break;
                }
                lastmatch = rest;
                rest = rest.offset(1);
            }
        }
        if !lastmatch.is_null() {
            (*rettv).vval.v_number = lastmatch.offset_from(haystack) as varnumber_T;
        }
    }
}

pub unsafe extern "C" fn f_strtrans(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = transstr(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            true_0 != 0,
        );
    }
}

pub unsafe extern "C" fn f_tolower(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = strcase_save(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            false_0 != 0,
        );
    }
}

pub unsafe extern "C" fn f_toupper(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = strcase_save(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            true_0 != 0,
        );
    }
}

pub unsafe extern "C" fn f_tr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
        let mut in_str: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut fromstr: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        let mut tostr: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut buf2 as *mut ::core::ffi::c_char,
        );
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if fromstr.is_null() || tostr.is_null() {
            return;
        }
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        let mut first: bool = true_0 != 0;
        '_error: {
            while *in_str as ::core::ffi::c_int != NUL {
                let mut cpstr: *const ::core::ffi::c_char = in_str;
                let inlen: ::core::ffi::c_int = utfc_ptr2len(in_str);
                let mut cplen: ::core::ffi::c_int = inlen;
                let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut fromlen: ::core::ffi::c_int = 0;
                let mut p: *const ::core::ffi::c_char = fromstr;
                while *p as ::core::ffi::c_int != NUL {
                    fromlen = utfc_ptr2len(p);
                    if fromlen == inlen
                        && strncmp(in_str, p, inlen as size_t) == 0 as ::core::ffi::c_int
                    {
                        let mut tolen: ::core::ffi::c_int = 0;
                        p = tostr;
                        while *p as ::core::ffi::c_int != NUL {
                            tolen = utfc_ptr2len(p);
                            let c2rust_fresh32 = idx;
                            idx = idx - 1;
                            if c2rust_fresh32 == 0 as ::core::ffi::c_int {
                                cplen = tolen;
                                cpstr = p;
                                break;
                            } else {
                                p = p.offset(tolen as isize);
                            }
                        }
                        if *p as ::core::ffi::c_int == NUL {
                            break '_error;
                        } else {
                            break;
                        }
                    } else {
                        idx += 1;
                        p = p.offset(fromlen as isize);
                    }
                }
                if first as ::core::ffi::c_int != 0 && cpstr == in_str {
                    first = false_0 != 0;
                    let mut tolen_0: ::core::ffi::c_int = 0;
                    let mut p_0: *const ::core::ffi::c_char = tostr;
                    while *p_0 as ::core::ffi::c_int != NUL {
                        tolen_0 = utfc_ptr2len(p_0);
                        idx -= 1;
                        p_0 = p_0.offset(tolen_0 as isize);
                    }
                    if idx != 0 as ::core::ffi::c_int {
                        break '_error;
                    }
                }
                ga_grow(&raw mut ga, cplen);
                memmove(
                    (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize)
                        as *mut ::core::ffi::c_void,
                    cpstr as *const ::core::ffi::c_void,
                    cplen as size_t,
                );
                ga.ga_len += cplen;
                in_str = in_str.offset(inlen as isize);
            }
            ga_append(&raw mut ga, NUL as uint8_t);
            (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
            return;
        }
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            fromstr,
        );
        ga_clear(&raw mut ga);
    }
}

pub unsafe extern "C" fn f_trim(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
        let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
        let mut head: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut buf1 as *mut ::core::ffi::c_char,
        );
        let mut mask: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut prev: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if head.is_null() {
            return;
        }
        if tv_check_for_opt_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
            return;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            mask = tv_get_string_buf_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut buf2 as *mut ::core::ffi::c_char,
            );
            if *mask as ::core::ffi::c_int == NUL {
                mask = ::core::ptr::null::<::core::ffi::c_char>();
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut error: bool = false_0 != 0;
                dir = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as ::core::ffi::c_int;
                if error {
                    return;
                }
                if dir < 0 as ::core::ffi::c_int || dir > 2 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        tv_get_string(argvars.offset(2 as ::core::ffi::c_int as isize)),
                    );
                    return;
                }
            }
        }
        if dir == 0 as ::core::ffi::c_int || dir == 1 as ::core::ffi::c_int {
            while *head as ::core::ffi::c_int != NUL {
                let mut c1: ::core::ffi::c_int = utf_ptr2char(head);
                if mask.is_null() {
                    if c1 > ' ' as ::core::ffi::c_int && c1 != 0xa0 as ::core::ffi::c_int {
                        break;
                    }
                } else {
                    p = mask;
                    while *p as ::core::ffi::c_int != NUL {
                        if c1 == utf_ptr2char(p) {
                            break;
                        }
                        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                    }
                    if *p as ::core::ffi::c_int == NUL {
                        break;
                    }
                }
                head = head.offset(utfc_ptr2len(head as *mut ::core::ffi::c_char) as isize);
            }
        }
        let mut tail: *const ::core::ffi::c_char = head.offset(strlen(head) as isize);
        if dir == 0 as ::core::ffi::c_int || dir == 2 as ::core::ffi::c_int {
            while tail > head {
                prev = tail;
                prev = prev.offset(
                    -((utf_head_off(
                        head as *mut ::core::ffi::c_char,
                        (prev as *mut ::core::ffi::c_char)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) + 1 as ::core::ffi::c_int) as isize),
                );
                let mut c1_0: ::core::ffi::c_int = utf_ptr2char(prev);
                if mask.is_null() {
                    if c1_0 > ' ' as ::core::ffi::c_int && c1_0 != 0xa0 as ::core::ffi::c_int {
                        break;
                    }
                } else {
                    p = mask;
                    while *p as ::core::ffi::c_int != NUL {
                        if c1_0 == utf_ptr2char(p) {
                            break;
                        }
                        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                    }
                    if *p as ::core::ffi::c_int == NUL {
                        break;
                    }
                }
                tail = prev;
            }
        }
        (*rettv).vval.v_string = xstrnsave(head, tail.offset_from(head) as size_t);
    }
}
