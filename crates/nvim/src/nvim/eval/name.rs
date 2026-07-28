//! Scanning a variable, function or option name out of an expression.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_env_len(mut arg: *mut *const c_char) -> c_int {
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = *arg;
    while vim_isIDc(*p as uint8_t as c_int) {
        p = p.offset(1);
    }
    if p == *arg {
        return 0 as c_int;
    }
    let mut len: c_int = p.offset_from(*arg) as c_int;
    *arg = p;
    return len;
}

pub unsafe extern "C" fn get_id_len(arg: *mut *const c_char) -> c_int {
    let mut len: c_int = 0;
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = *arg;
    while eval_isnamec(*p as c_int) {
        if *p as c_int == ':' as c_int {
            len = p.offset_from(*arg) as c_int;
            if len > 1 as c_int
                || len == 1 as c_int
                    && vim_strchr(namespace_char.get(), **arg as uint8_t as c_int).is_null()
            {
                break;
            }
        }
        p = p.offset(1);
    }
    if p == *arg {
        return 0 as c_int;
    }
    len = p.offset_from(*arg) as c_int;
    *arg = skipwhite(p);
    return len;
}

pub unsafe extern "C" fn get_name_len(
    arg: *mut *const c_char,
    mut alias: *mut *mut c_char,
    mut evaluate: bool,
    mut verbose: bool,
) -> c_int {
    *alias = ::core::ptr::null_mut::<c_char>();
    if *(*arg).offset(0 as c_int as isize) as c_int == K_SPECIAL as c_char as c_int
        && *(*arg).offset(1 as c_int as isize) as c_int == KS_EXTRA as c_char as c_int
        && *(*arg).offset(2 as c_int as isize) as c_int == KE_SNR as c_int as c_char as c_int
    {
        *arg = (*arg).offset(3 as c_int as isize);
        return get_id_len(arg) + 3 as c_int;
    }
    let mut len: c_int = eval_fname_script(*arg);
    if len > 0 as c_int {
        *arg = (*arg).offset(len as isize);
    }
    let mut expr_start: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut expr_end: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *const c_char = find_name_end(
        *arg,
        &raw mut expr_start as *mut *const c_char,
        &raw mut expr_end as *mut *const c_char,
        if len > 0 as c_int {
            0 as c_int
        } else {
            FNE_CHECK_START
        },
    );
    if !expr_start.is_null() {
        if !evaluate {
            len += p.offset_from(*arg) as c_int;
            *arg = skipwhite(p);
            return len;
        }
        let mut temp_string: *mut c_char = make_expanded_name(
            (*arg).offset(-(len as isize)),
            expr_start,
            expr_end,
            p as *mut c_char,
        );
        if temp_string.is_null() {
            return -1 as c_int;
        }
        *alias = temp_string;
        *arg = skipwhite(p);
        return strlen(temp_string) as c_int;
    }
    len += get_id_len(arg);
    if len == 0 as c_int && verbose as c_int != 0 && **arg as c_int != NUL {
        semsg(gettext(&raw const e_invexpr2 as *const c_char), *arg);
    }
    return len;
}

pub unsafe extern "C" fn find_name_end(
    mut arg: *const c_char,
    mut expr_start: *mut *const c_char,
    mut expr_end: *mut *const c_char,
    mut flags: c_int,
) -> *const c_char {
    if !expr_start.is_null() {
        *expr_start = ::core::ptr::null::<c_char>();
        *expr_end = ::core::ptr::null::<c_char>();
    }
    if flags & FNE_CHECK_START != 0
        && !eval_isnamec1(*arg as c_int)
        && *arg as c_int != '{' as c_int
    {
        return arg;
    }
    let mut mb_nest: c_int = 0 as c_int;
    let mut br_nest: c_int = 0 as c_int;
    let mut len: c_int = 0;
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = arg;
    while *p as c_int != NUL
        && (eval_isnamec(*p as c_int) as c_int != 0
            || *p as c_int == '{' as c_int
            || flags & FNE_INCL_BR != 0
                && (*p as c_int == '[' as c_int
                    || *p as c_int == '.' as c_int
                        && eval_isdictc(*p.offset(1 as c_int as isize) as c_int) as c_int != 0)
            || mb_nest != 0 as c_int
            || br_nest != 0 as c_int)
    {
        if *p as c_int == '\'' as c_int {
            p = p.offset(1 as c_int as isize);
            while *p as c_int != NUL && *p as c_int != '\'' as c_int {
                p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
            }
            if *p as c_int == NUL {
                break;
            }
        } else if *p as c_int == '"' as c_int {
            p = p.offset(1 as c_int as isize);
            while *p as c_int != NUL && *p as c_int != '"' as c_int {
                if *p as c_int == '\\' as c_int && *p.offset(1 as c_int as isize) as c_int != NUL {
                    p = p.offset(1);
                }
                p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
            }
            if *p as c_int == NUL {
                break;
            }
        } else if br_nest == 0 as c_int && mb_nest == 0 as c_int && *p as c_int == ':' as c_int {
            len = p.offset_from(arg) as c_int;
            if len > 1 as c_int && *p.offset(-1 as c_int as isize) as c_int != '}' as c_int
                || len == 1 as c_int
                    && vim_strchr(namespace_char.get(), *arg as uint8_t as c_int).is_null()
            {
                break;
            }
        }
        if mb_nest == 0 as c_int {
            if *p as c_int == '[' as c_int {
                br_nest += 1;
            } else if *p as c_int == ']' as c_int {
                br_nest -= 1;
            }
        }
        if br_nest == 0 as c_int {
            if *p as c_int == '{' as c_int {
                mb_nest += 1;
                if !expr_start.is_null() && (*expr_start).is_null() {
                    *expr_start = p;
                }
            } else if *p as c_int == '}' as c_int {
                mb_nest -= 1;
                if !expr_start.is_null() && mb_nest == 0 as c_int && (*expr_end).is_null() {
                    *expr_end = p;
                }
            }
        }
        p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
    }
    return p;
}

pub(crate) unsafe extern "C" fn make_expanded_name(
    mut in_start: *const c_char,
    mut expr_start: *mut c_char,
    mut expr_end: *mut c_char,
    mut in_end: *mut c_char,
) -> *mut c_char {
    if expr_end.is_null() || in_end.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    *expr_start = NUL as c_char;
    *expr_end = NUL as c_char;
    let mut c1: c_char = *in_end;
    *in_end = NUL as c_char;
    let mut temp_result: *mut c_char = eval_to_string(
        expr_start.offset(1 as c_int as isize),
        false_0 != 0,
        false_0 != 0,
    );
    if !temp_result.is_null() {
        let mut retvalsize: size_t = (expr_start.offset_from(in_start) as size_t)
            .wrapping_add(strlen(temp_result))
            .wrapping_add(in_end.offset_from(expr_end) as size_t)
            .wrapping_add(1 as size_t);
        retval = xmalloc(retvalsize) as *mut c_char;
        vim_snprintf(
            retval,
            retvalsize,
            b"%s%s%s\0".as_ptr() as *const c_char,
            in_start,
            temp_result,
            expr_end.offset(1 as c_int as isize),
        );
    }
    xfree(temp_result as *mut c_void);
    *in_end = c1;
    *expr_start = '{' as c_char;
    *expr_end = '}' as c_char;
    if !retval.is_null() {
        temp_result = find_name_end(
            retval,
            &raw mut expr_start as *mut *const c_char,
            &raw mut expr_end as *mut *const c_char,
            0 as c_int,
        ) as *mut c_char;
        if !expr_start.is_null() {
            temp_result = make_expanded_name(retval, expr_start, expr_end, temp_result);
            xfree(retval as *mut c_void);
            retval = temp_result;
        }
    }
    return retval;
}

pub unsafe extern "C" fn eval_isnamec(mut c: c_int) -> bool {
    return c as c_uint >= 'A' as c_uint && c as c_uint <= 'Z' as c_uint
        || c as c_uint >= 'a' as c_uint && c as c_uint <= 'z' as c_uint
        || ascii_isdigit(c) as c_int != 0
        || c == '_' as c_int
        || c == ':' as c_int
        || c == AUTOLOAD_CHAR;
}

pub unsafe extern "C" fn eval_isnamec1(mut c: c_int) -> bool {
    return c as c_uint >= 'A' as c_uint && c as c_uint <= 'Z' as c_uint
        || c as c_uint >= 'a' as c_uint && c as c_uint <= 'z' as c_uint
        || c == '_' as c_int;
}

pub unsafe extern "C" fn eval_isdictc(mut c: c_int) -> bool {
    return c as c_uint >= 'A' as c_uint && c as c_uint <= 'Z' as c_uint
        || c as c_uint >= 'a' as c_uint && c as c_uint <= 'z' as c_uint
        || ascii_isdigit(c) as c_int != 0
        || c == '_' as c_int;
}

pub unsafe extern "C" fn is_luafunc(mut partial: *mut partial_T) -> bool {
    return partial == get_vim_var_partial(VV_LUA);
}

pub(crate) unsafe extern "C" fn tv_is_luafunc(mut tv: *mut typval_T) -> bool {
    return (*tv).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
        && is_luafunc((*tv).vval.v_partial) as c_int != 0;
}

pub unsafe extern "C" fn skip_luafunc_name(mut p: *const c_char) -> *const c_char {
    while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
        || ascii_isdigit(*p as c_int) as c_int != 0
        || *p as c_int == '_' as c_int
        || *p as c_int == '-' as c_int
        || *p as c_int == '.' as c_int
        || *p as c_int == '\'' as c_int
    {
        p = p.offset(1);
    }
    return p;
}

pub unsafe extern "C" fn check_luafunc_name(str: *const c_char, paren: bool) -> c_int {
    let p: *const c_char = skip_luafunc_name(str);
    if *p as c_int
        != (if paren as c_int != 0 {
            '(' as c_int
        } else {
            NUL
        })
    {
        return 0 as c_int;
    }
    return p.offset_from(str) as c_int;
}

pub unsafe extern "C" fn find_option_var_end(
    arg: *mut *const c_char,
    opt_idxp: *mut OptIndex,
    opt_flags: *mut c_int,
) -> *const c_char {
    let mut p: *const c_char = *arg;
    p = p.offset(1);
    if *p as c_int == 'g' as c_int && *p.offset(1 as c_int as isize) as c_int == ':' as c_int {
        *opt_flags = OPT_GLOBAL as c_int;
        p = p.offset(2 as c_int as isize);
    } else if *p as c_int == 'l' as c_int && *p.offset(1 as c_int as isize) as c_int == ':' as c_int
    {
        *opt_flags = OPT_LOCAL as c_int;
        p = p.offset(2 as c_int as isize);
    } else {
        *opt_flags = 0 as c_int;
    }
    let mut end: *const c_char = find_option_end(p, opt_idxp);
    *arg = if end.is_null() { *arg } else { p };
    return end;
}
