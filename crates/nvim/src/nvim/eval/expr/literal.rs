//! Operands that are written out: numbers, the three string forms,
//! `&option` and `$ENV`.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn eval_option(
    arg: *mut *const c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    let working: bool = **arg as c_int == '+' as c_int;
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: c_int = 0;
    let option_end: *mut c_char =
        find_option_var_end(arg, &raw mut opt_idx, &raw mut opt_flags) as *mut c_char;
    if option_end.is_null() {
        if !rettv.is_null() {
            semsg(
                gettext(b"E112: Option name missing: %s\0".as_ptr() as *const c_char),
                *arg,
            );
        }
        return FAIL;
    }
    if !evaluate {
        *arg = option_end;
        return OK;
    }
    let mut c: c_char = *option_end;
    *option_end = NUL as c_char;
    let mut ret: c_int = OK;
    let mut is_tty_opt: bool = is_tty_option(*arg);
    if opt_idx as c_int == kOptInvalid as c_int && !is_tty_opt {
        if !rettv.is_null() {
            semsg(
                gettext(b"E113: Unknown option: %s\0".as_ptr() as *const c_char),
                *arg,
            );
        }
        ret = FAIL;
    } else if !rettv.is_null() {
        let mut value: OptVal = if is_tty_opt as c_int != 0 {
            get_tty_option(*arg)
        } else {
            get_option_value(opt_idx, opt_flags)
        };
        '_c2rust_label: {
            if value.type_0 as c_int != kOptValTypeNil as c_int {
            } else {
                __assert_fail(
                    b"value.type != kOptValTypeNil\0".as_ptr() as *const c_char,
                    b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                    3409 as c_uint,
                    b"int eval_option(const char **const, typval_T *const, const _Bool)\0".as_ptr()
                        as *const c_char,
                );
            }
        };
        *rettv = optval_as_tv(value, true_0 != 0);
    } else if working as c_int != 0 && !is_tty_opt && is_option_hidden(opt_idx) as c_int != 0 {
        ret = FAIL;
    }
    *option_end = c;
    *arg = option_end;
    return ret;
}

pub(crate) unsafe extern "C" fn eval_number(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut want_string: bool,
) -> c_int {
    let mut p: *mut c_char = skipdigits((*arg).offset(1 as c_int as isize));
    let mut get_float: bool = false_0 != 0;
    if !want_string
        && *p.offset(0 as c_int as isize) as c_int == '.' as c_int
        && ascii_isdigit(*p.offset(1 as c_int as isize) as c_int) as c_int != 0
    {
        get_float = true_0 != 0;
        p = skipdigits(p.offset(2 as c_int as isize));
        if *p as c_int == 'e' as c_int || *p as c_int == 'E' as c_int {
            p = p.offset(1);
            if *p as c_int == '-' as c_int || *p as c_int == '+' as c_int {
                p = p.offset(1);
            }
            if !ascii_isdigit(*p as c_int) {
                get_float = false_0 != 0;
            } else {
                p = skipdigits(p.offset(1 as c_int as isize));
            }
        }
        if *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
            || *p as c_int == '.' as c_int
        {
            get_float = false_0 != 0;
        }
    }
    if get_float {
        let mut f: float_T = 0.;
        *arg = (*arg).offset(string2float(*arg, &raw mut f) as isize);
        if evaluate {
            (*rettv).v_type = VAR_FLOAT;
            (*rettv).vval.v_float = f;
        }
    } else if **arg as c_int == '0' as c_int
        && (*(*arg).offset(1 as c_int as isize) as c_int == 'z' as c_int
            || *(*arg).offset(1 as c_int as isize) as c_int == 'Z' as c_int)
    {
        let mut blob: *mut blob_T = ::core::ptr::null_mut::<blob_T>();
        if evaluate {
            blob = tv_blob_alloc();
        }
        let mut bp: *mut c_char = ::core::ptr::null_mut::<c_char>();
        bp = (*arg).offset(2 as c_int as isize);
        while ascii_isxdigit(*bp.offset(0 as c_int as isize) as c_int) {
            if !ascii_isxdigit(*bp.offset(1 as c_int as isize) as c_int) {
                if !blob.is_null() {
                    emsg(gettext(
                        b"E973: Blob literal should have an even number of hex characters\0"
                            .as_ptr() as *const c_char,
                    ));
                    ga_clear(&raw mut (*blob).bv_ga);
                    let mut ptr_: *mut *mut c_void = &raw mut blob as *mut *mut c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL_0;
                    let _ = *ptr_;
                }
                return FAIL;
            }
            if !blob.is_null() {
                ga_append(
                    &raw mut (*blob).bv_ga,
                    ((hex2nr(*bp as c_int) << 4 as c_int)
                        + hex2nr(*bp.offset(1 as c_int as isize) as c_int))
                        as uint8_t,
                );
            }
            if *bp.offset(2 as c_int as isize) as c_int == '.' as c_int
                && ascii_isxdigit(*bp.offset(3 as c_int as isize) as c_int) as c_int != 0
            {
                bp = bp.offset(1);
            }
            bp = bp.offset(2 as c_int as isize);
        }
        if !blob.is_null() {
            tv_blob_set_ret(rettv, blob);
        }
        *arg = bp;
    } else {
        let mut len: c_int = 0;
        let mut n: varnumber_T = 0;
        vim_str2nr(
            *arg,
            ::core::ptr::null_mut::<c_int>(),
            &raw mut len,
            STR2NR_ALL as c_int,
            &raw mut n,
            ::core::ptr::null_mut::<uvarnumber_T>(),
            0 as c_int,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if len == 0 as c_int {
            if evaluate {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), *arg);
            }
            return FAIL;
        }
        *arg = (*arg).offset(len as isize);
        if evaluate {
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = n;
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval_string(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut interpolate: bool,
) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let arg_end: *const c_char = (*arg).offset(strlen(*arg) as isize);
    let mut extra: c_uint = (if interpolate as c_int != 0 {
        1 as c_int
    } else {
        0 as c_int
    }) as c_uint;
    let off: c_int = if interpolate as c_int != 0 {
        0 as c_int
    } else {
        1 as c_int
    };
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL && *p as c_int != '"' as c_int {
        if *p as c_int == '\\' as c_int && *p.offset(1 as c_int as isize) as c_int != NUL {
            p = p.offset(1);
            if *p as c_int == '<' as c_int {
                let mut modifiers: c_int = 0 as c_int;
                let mut flags: c_int = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                extra = extra.wrapping_add(5 as c_uint);
                if *p.offset(1 as c_int as isize) as c_int != '*' as c_int {
                    flags |= FSK_SIMPLIFY as c_int;
                }
                if find_special_key(
                    &raw mut p as *mut *const c_char,
                    arg_end.offset_from(p) as size_t,
                    &raw mut modifiers,
                    flags,
                    ::core::ptr::null_mut::<bool>(),
                ) != 0 as c_int
                {
                    p = p.offset(-1);
                }
            }
        } else if interpolate as c_int != 0
            && (*p as c_int == '{' as c_int || *p as c_int == '}' as c_int)
        {
            if *p as c_int == '{' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '{' as c_int
            {
                break;
            }
            p = p.offset(1);
            if *p.offset(-1 as c_int as isize) as c_int == '}' as c_int
                && *p as c_int != '}' as c_int
            {
                semsg(
                    gettext(&raw const e_stray_closing_curly_str as *const c_char),
                    *arg,
                );
                return FAIL;
            }
            extra = extra.wrapping_sub(1);
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p as c_int != '"' as c_int && !(interpolate as c_int != 0 && *p as c_int == '{' as c_int) {
        semsg(
            gettext(b"E114: Missing quote: %s\0".as_ptr() as *const c_char),
            *arg,
        );
        return FAIL;
    }
    if !evaluate {
        *arg = p.offset(off as isize);
        return OK;
    }
    (*rettv).v_type = VAR_STRING;
    let len: c_int = (p.offset_from(*arg) + extra as isize) as c_int;
    (*rettv).vval.v_string = xmalloc(len as size_t) as *mut c_char;
    let mut end: *mut c_char = (*rettv).vval.v_string;
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL && *p as c_int != '"' as c_int {
        if *p as c_int == '\\' as c_int {
            's_424: {
                p = p.offset(1);
                match *p as c_int {
                    98 => {
                        let c2rust_fresh0 = end;
                        end = end.offset(1);
                        *c2rust_fresh0 = BS as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    101 => {
                        let c2rust_fresh1 = end;
                        end = end.offset(1);
                        *c2rust_fresh1 = ESC as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    102 => {
                        let c2rust_fresh2 = end;
                        end = end.offset(1);
                        *c2rust_fresh2 = FF as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    110 => {
                        let c2rust_fresh3 = end;
                        end = end.offset(1);
                        *c2rust_fresh3 = NL as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    114 => {
                        let c2rust_fresh4 = end;
                        end = end.offset(1);
                        *c2rust_fresh4 = CAR as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    116 => {
                        let c2rust_fresh5 = end;
                        end = end.offset(1);
                        *c2rust_fresh5 = TAB as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    88 | 120 | 117 | 85 => {
                        if ascii_isxdigit(*p.offset(1 as c_int as isize) as c_int) {
                            let mut n: c_int = 0;
                            let mut nr: c_int = 0;
                            let mut c: c_int = toupper(*p as uint8_t as c_int);
                            if c == 'X' as c_int {
                                n = 2 as c_int;
                            } else if *p as c_int == 'u' as c_int {
                                n = 4 as c_int;
                            } else {
                                n = 8 as c_int;
                            }
                            nr = 0 as c_int;
                            loop {
                                n -= 1;
                                if !(n >= 0 as c_int
                                    && ascii_isxdigit(*p.offset(1 as c_int as isize) as c_int)
                                        as c_int
                                        != 0)
                                {
                                    break;
                                }
                                p = p.offset(1);
                                nr = (nr << 4 as c_int) + hex2nr(*p as c_int);
                            }
                            p = p.offset(1);
                            if c != 'X' as c_int {
                                end = end.offset(utf_char2bytes(nr, end) as isize);
                            } else {
                                let c2rust_fresh6 = end;
                                end = end.offset(1);
                                *c2rust_fresh6 = nr as c_char;
                            }
                        }
                        break 's_424;
                    }
                    48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                        let c2rust_fresh7 = p;
                        p = p.offset(1);
                        *end = (*c2rust_fresh7 as c_int - '0' as c_int) as c_char;
                        if *p as c_int >= '0' as c_int && *p as c_int <= '7' as c_int {
                            let c2rust_fresh8 = p;
                            p = p.offset(1);
                            *end = (((*end as c_int) << 3 as c_int) + *c2rust_fresh8 as c_int
                                - '0' as c_int) as c_char;
                            if *p as c_int >= '0' as c_int && *p as c_int <= '7' as c_int {
                                let c2rust_fresh9 = p;
                                p = p.offset(1);
                                *end = (((*end as c_int) << 3 as c_int) + *c2rust_fresh9 as c_int
                                    - '0' as c_int)
                                    as c_char;
                            }
                        }
                        end = end.offset(1);
                        break 's_424;
                    }
                    60 => {
                        let mut flags_0: c_int = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                        if *p.offset(1 as c_int as isize) as c_int != '*' as c_int {
                            flags_0 |= FSK_SIMPLIFY as c_int;
                        }
                        extra = trans_special(
                            &raw mut p as *mut *const c_char,
                            arg_end.offset_from(p) as size_t,
                            end,
                            flags_0,
                            false_0 != 0,
                            ::core::ptr::null_mut::<bool>(),
                        );
                        if extra != 0 as c_uint {
                            end = end.offset(extra as isize);
                            if end >= (*rettv).vval.v_string.offset(len as isize) {
                                iemsg(b"eval_string() used more space than allocated\0".as_ptr()
                                    as *const c_char);
                            }
                            break 's_424;
                        }
                    }
                    _ => {}
                }
                mb_copy_char(&raw mut p as *mut *const c_char, &raw mut end);
            }
        } else {
            if interpolate as c_int != 0
                && (*p as c_int == '{' as c_int || *p as c_int == '}' as c_int)
            {
                if *p as c_int == '{' as c_int
                    && *p.offset(1 as c_int as isize) as c_int != '{' as c_int
                {
                    break;
                }
                p = p.offset(1);
            }
            mb_copy_char(&raw mut p as *mut *const c_char, &raw mut end);
        }
    }
    *end = NUL as c_char;
    if *p as c_int == '"' as c_int && !interpolate {
        p = p.offset(1);
    }
    *arg = p;
    return OK;
}

pub(crate) unsafe extern "C" fn eval_lit_string(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut interpolate: bool,
) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut reduce: c_int = if interpolate as c_int != 0 {
        -1 as c_int
    } else {
        0 as c_int
    };
    let off: c_int = if interpolate as c_int != 0 {
        0 as c_int
    } else {
        1 as c_int
    };
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL {
        if *p as c_int == '\'' as c_int {
            if *p.offset(1 as c_int as isize) as c_int != '\'' as c_int {
                break;
            }
            reduce += 1;
            p = p.offset(1);
        } else if interpolate {
            if *p as c_int == '{' as c_int {
                if *p.offset(1 as c_int as isize) as c_int != '{' as c_int {
                    break;
                }
                p = p.offset(1);
                reduce += 1;
            } else if *p as c_int == '}' as c_int {
                p = p.offset(1);
                if *p as c_int != '}' as c_int {
                    semsg(
                        gettext(&raw const e_stray_closing_curly_str as *const c_char),
                        *arg,
                    );
                    return FAIL;
                }
                reduce += 1;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p as c_int != '\'' as c_int && !(interpolate as c_int != 0 && *p as c_int == '{' as c_int) {
        semsg(
            gettext(b"E115: Missing quote: %s\0".as_ptr() as *const c_char),
            *arg,
        );
        return FAIL;
    }
    if !evaluate {
        *arg = p.offset(off as isize);
        return OK;
    }
    let mut str: *mut c_char =
        xmalloc((p.offset_from(*arg) - reduce as isize) as size_t) as *mut c_char;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = str;
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL {
        if *p as c_int == '\'' as c_int {
            if *p.offset(1 as c_int as isize) as c_int != '\'' as c_int {
                break;
            }
            p = p.offset(1);
        } else if interpolate as c_int != 0
            && (*p as c_int == '{' as c_int || *p as c_int == '}' as c_int)
        {
            if *p as c_int == '{' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '{' as c_int
            {
                break;
            }
            p = p.offset(1);
        }
        mb_copy_char(&raw mut p as *mut *const c_char, &raw mut str);
    }
    *str = NUL as c_char;
    *arg = p.offset(off as isize);
    return OK;
}

pub unsafe extern "C" fn eval_interp_string(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
) -> c_int {
    let mut ret: c_int = OK;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    ga_init(&raw mut ga, 1 as c_int, 80 as c_int);
    *arg = (*arg).offset(1);
    let quote: c_int = **arg as uint8_t as c_int;
    *arg = (*arg).offset(1);
    loop {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if quote == '"' as c_int {
            ret = eval_string(arg, &raw mut tv, evaluate, true_0 != 0);
        } else {
            ret = eval_lit_string(arg, &raw mut tv, evaluate, true_0 != 0);
        }
        if ret == FAIL {
            break;
        }
        if evaluate {
            ga_concat(&raw mut ga, tv.vval.v_string);
            tv_clear(&raw mut tv);
        }
        if **arg as c_int != '{' as c_int {
            *arg = (*arg).offset(1);
            break;
        } else {
            let mut p: *mut c_char = eval_one_expr_in_str(*arg, &raw mut ga, evaluate);
            if p.is_null() {
                ret = FAIL;
                break;
            } else {
                *arg = p;
            }
        }
    }
    (*rettv).v_type = VAR_STRING;
    if ret != FAIL && evaluate as c_int != 0 {
        ga_append(&raw mut ga, NUL as uint8_t);
    }
    (*rettv).vval.v_string = ga.ga_data as *mut c_char;
    return OK;
}

pub unsafe extern "C" fn string2float(text: *const c_char, ret_value: *mut float_T) -> size_t {
    if strncasecmp(
        text as *mut c_char,
        b"inf\0".as_ptr() as *const c_char as *mut c_char,
        3 as c_int as size_t,
    ) == 0 as c_int
    {
        *ret_value = ::core::f32::INFINITY as float_T;
        return 3 as size_t;
    }
    if strncasecmp(
        text as *mut c_char,
        b"-inf\0".as_ptr() as *const c_char as *mut c_char,
        4 as c_int as size_t,
    ) == 0 as c_int
    {
        *ret_value = -::core::f32::INFINITY as float_T;
        return 4 as size_t;
    }
    if strncasecmp(
        text as *mut c_char,
        b"nan\0".as_ptr() as *const c_char as *mut c_char,
        3 as c_int as size_t,
    ) == 0 as c_int
    {
        *ret_value = ::core::f32::NAN as float_T;
        return 3 as size_t;
    }
    let mut s: *mut c_char = ::core::ptr::null_mut::<c_char>();
    *ret_value = strtod(text, &raw mut s) as float_T;
    return s.offset_from(text) as size_t;
}

pub(crate) unsafe extern "C" fn eval_env_var(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: c_int,
) -> c_int {
    *arg = (*arg).offset(1);
    let mut name: *mut c_char = *arg;
    let mut len: c_int = get_env_len(arg as *mut *const c_char);
    if evaluate != 0 {
        if len == 0 as c_int {
            return FAIL;
        }
        let mut cc: c_int = *name.offset(len as isize) as c_int;
        *name.offset(len as isize) = NUL as c_char;
        let mut string: *mut c_char = vim_getenv(name);
        if string.is_null() || *string as c_int == NUL {
            xfree(string as *mut c_void);
            string = expand_env_save(name.offset(-(1 as c_int as isize)));
            if !string.is_null() && *string as c_int == '$' as c_int {
                let mut ptr_: *mut *mut c_void = &raw mut string as *mut *mut c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
            }
        }
        *name.offset(len as isize) = cc as c_char;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = string;
        (*rettv).v_lock = VAR_UNLOCKED;
    }
    return OK;
}
