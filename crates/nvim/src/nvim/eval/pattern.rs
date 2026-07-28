//! Matching and substituting with a regexp built from an expression.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn pattern_match(
    mut pat: *const c_char,
    mut text: *const c_char,
    mut ic: bool,
) -> c_int {
    let mut matches: c_int = 0 as c_int;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<c_char>(); 10],
        endp: [::core::ptr::null_mut::<c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut save_cpo: *mut c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut c_char);
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = ic;
        matches = vim_regexec_nl(&raw mut regmatch, text, 0 as colnr_T) as c_int;
        vim_regfree(regmatch.regprog);
    }
    p_cpo.set(save_cpo);
    return matches;
}

pub unsafe extern "C" fn do_string_sub(
    mut str: *mut c_char,
    mut len: size_t,
    mut pat: *mut c_char,
    mut sub: *mut c_char,
    mut expr: *mut typval_T,
    mut flags: *const c_char,
    mut ret_len: *mut size_t,
) -> *mut c_char {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<c_char>(); 10],
        endp: [::core::ptr::null_mut::<c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    let mut save_cpo: *mut c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut c_char);
    ga_init(&raw mut ga, 1 as c_int, 200 as c_int);
    regmatch.rm_ic = p_ic.get() != 0;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        let mut tail: *mut c_char = str;
        let mut end: *mut c_char = str.offset(len as isize);
        let mut do_all: bool = *flags.offset(0 as c_int as isize) as c_int == 'g' as c_int;
        let mut sublen: c_int = 0;
        let mut zero_width: *mut c_char = ::core::ptr::null_mut::<c_char>();
        while vim_regexec_nl(&raw mut regmatch, str, tail.offset_from(str) as colnr_T) {
            if regmatch.startp[0 as c_int as usize] == regmatch.endp[0 as c_int as usize] {
                if zero_width == regmatch.startp[0 as c_int as usize] {
                    let mut i: c_int = utfc_ptr2len(tail);
                    memmove(
                        (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) as *mut c_void,
                        tail as *const c_void,
                        i as size_t,
                    );
                    ga.ga_len += i;
                    tail = tail.offset(i as isize);
                    continue;
                } else {
                    zero_width = regmatch.startp[0 as c_int as usize];
                }
            }
            sublen = vim_regsub(
                &raw mut regmatch,
                sub,
                expr,
                tail,
                0 as c_int,
                REGSUB_MAGIC as c_int,
            );
            if sublen <= 0 as c_int {
                ga_clear(&raw mut ga);
                break;
            } else {
                ga_grow(
                    &raw mut ga,
                    (end.offset_from(tail) + sublen as isize
                        - regmatch.endp[0 as c_int as usize]
                            .offset_from(regmatch.startp[0 as c_int as usize]))
                        as c_int,
                );
                let mut i_0: c_int =
                    regmatch.startp[0 as c_int as usize].offset_from(tail) as c_int;
                memmove(
                    (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) as *mut c_void,
                    tail as *const c_void,
                    i_0 as size_t,
                );
                vim_regsub(
                    &raw mut regmatch,
                    sub,
                    expr,
                    (ga.ga_data as *mut c_char)
                        .offset(ga.ga_len as isize)
                        .offset(i_0 as isize),
                    sublen,
                    REGSUB_COPY as c_int | REGSUB_MAGIC as c_int,
                );
                ga.ga_len += i_0 + sublen - 1 as c_int;
                tail = regmatch.endp[0 as c_int as usize];
                if *tail as c_int == NUL {
                    break;
                }
                if !do_all {
                    break;
                }
            }
        }
        if !ga.ga_data.is_null() {
            strcpy((ga.ga_data as *mut c_char).offset(ga.ga_len as isize), tail);
            ga.ga_len += end.offset_from(tail) as c_int;
        }
        vim_regfree(regmatch.regprog);
    }
    if !ga.ga_data.is_null() {
        str = ga.ga_data as *mut c_char;
        len = ga.ga_len as size_t;
    }
    let mut ret: *mut c_char = xstrnsave(str, len);
    ga_clear(&raw mut ga);
    if p_cpo.get() == empty_string_option.ptr() as *mut c_char {
        p_cpo.set(save_cpo);
    } else {
        if *p_cpo.get() as c_int == NUL {
            set_option_value_give_err(
                kOptCpoptions,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(save_cpo),
                    },
                },
                0 as c_int,
            );
        }
        free_string_option(save_cpo);
    }
    if !ret_len.is_null() {
        *ret_len = len;
    }
    return ret;
}
