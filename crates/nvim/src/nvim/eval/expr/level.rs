//! The recursive-descent evaluator, one function per precedence level.
//!
//! `eval0` is the entry; each level parses its own operators and hands the
//! rest down, so `eval1` is `? :`, `eval2` is `||`, `eval3` is `&&`, `eval4`
//! the comparisons, `eval5` `+`/`-`/`..`, `eval6` `*`/`/`/`%` and `eval7` an
//! operand with its subscripts.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn eval_func(
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    name: *mut c_char,
    name_len: c_int,
    rettv: *mut typval_T,
    flags: c_int,
    basetv: *mut typval_T,
) -> c_int {
    let evaluate: bool = flags & EVAL_EVALUATE as c_int != 0;
    let mut s: *mut c_char = name;
    let mut len: c_int = name_len;
    let mut found_var: bool = false_0 != 0;
    if !evaluate {
        check_vars(s, len as size_t);
    }
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    s = deref_func_name(
        s,
        &raw mut len,
        &raw mut partial,
        !evaluate,
        &raw mut found_var,
    );
    s = xmemdupz(s as *const c_void, len as size_t) as *mut c_char;
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = evaluate;
    funcexe.fe_partial = partial;
    funcexe.fe_basetv = basetv;
    funcexe.fe_found_var = found_var;
    let mut ret: c_int = get_func_tv(s, len, rettv, arg, evalarg, &raw mut funcexe);
    xfree(s as *mut c_void);
    if (*rettv).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint
        && !evaluate
        && **arg as c_int == '(' as c_int
    {
        (*rettv).vval.v_string = tv_empty_string.get() as *mut c_char;
        (*rettv).v_type = VAR_FUNC;
    }
    if evaluate as c_int != 0 && aborting() as c_int != 0 {
        if ret == OK {
            tv_clear(rettv);
        }
        ret = FAIL;
    }
    return ret;
}

pub unsafe extern "C" fn clear_evalarg(mut evalarg: *mut evalarg_T, mut eap: *mut exarg_T) {
    if evalarg.is_null() {
        return;
    }
    if !(*evalarg).eval_tofree.is_null() {
        if !eap.is_null() {
            xfree((*eap).cmdline_tofree as *mut c_void);
            (*eap).cmdline_tofree = *(*eap).cmdlinep;
            *(*eap).cmdlinep = (*evalarg).eval_tofree;
        } else {
            xfree((*evalarg).eval_tofree as *mut c_void);
        }
        (*evalarg).eval_tofree = ::core::ptr::null_mut::<c_char>();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn eval0(
    mut arg: *mut c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let did_emsg_before: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let mut end_error: bool = false_0 != 0;
    let mut p: *mut c_char = skipwhite(arg);
    let mut ret: c_int = eval1(&raw mut p, rettv, evalarg);
    if ret != FAIL {
        end_error = ends_excmd(*p as c_int) == 0;
    }
    if ret == FAIL || end_error as c_int != 0 {
        if ret != FAIL {
            tv_clear(rettv);
        }
        if !aborting()
            && did_emsg.get() == did_emsg_before
            && called_emsg.get() == called_emsg_before
        {
            if end_error {
                semsg(gettext(&raw const e_trailing_arg as *const c_char), p);
            } else {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), arg);
            }
        }
        if !eap.is_null() && !p.is_null() {
            let mut nextcmd: *mut c_char = check_nextcmd(p);
            if !nextcmd.is_null() && *nextcmd as c_int != '|' as c_int {
                (*eap).nextcmd = nextcmd;
            }
        }
        return FAIL;
    }
    if !eap.is_null() {
        (*eap).nextcmd = check_nextcmd(p);
    }
    return ret;
}

pub unsafe extern "C" fn may_call_simple_func(
    mut arg: *const c_char,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut parens: *const c_char = strstr(arg, b"()\0".as_ptr() as *const c_char);
    let mut r: c_int = NOTDONE;
    if !parens.is_null() && *skipwhite(parens.offset(2 as c_int as isize)) as c_int == NUL {
        if strnequal(arg, b"v:lua.\0".as_ptr() as *const c_char, 6 as size_t) {
            let mut p: *const c_char = arg.offset(6 as c_int as isize);
            if p != parens && skip_luafunc_name(p) == parens {
                r = call_simple_luafunc(p, parens.offset_from(p) as size_t, rettv);
            }
        } else {
            let mut p_0: *const c_char =
                if strncmp(arg, b"<SNR>\0".as_ptr() as *const c_char, 5 as size_t) == 0 as c_int {
                    skipdigits(arg.offset(5 as c_int as isize)) as *const c_char
                } else {
                    arg
                };
            if to_name_end(p_0, true_0 != 0) == parens {
                r = call_simple_func(arg, parens.offset_from(arg) as size_t, rettv);
            }
        }
    }
    return r;
}

pub(crate) unsafe extern "C" fn eval0_simple_funccal(
    mut arg: *mut c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let mut r: c_int = may_call_simple_func(arg, rettv);
    if r == NOTDONE {
        r = eval0(arg, rettv, eap, evalarg);
    }
    return r;
}

pub unsafe extern "C" fn eval1(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    memset(
        rettv as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<typval_T>(),
    );
    if eval2(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    if *p as c_int == '?' as c_int {
        let op_falsy: bool = *p.offset(1 as c_int as isize) as c_int == '?' as c_int;
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<c_void>(),
            eval_tofree: ::core::ptr::null_mut::<c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<c_void>(),
                eval_tofree: ::core::ptr::null_mut::<c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as c_int != 0;
        let mut result: bool = false_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if op_falsy {
                result = tv2bool(rettv);
            } else if tv_get_number_chk(rettv, &raw mut error) != 0 as varnumber_T {
                result = true_0 != 0;
            }
            if error as c_int != 0 || !op_falsy || !result {
                tv_clear(rettv);
            }
            if error {
                return FAIL;
            }
        }
        if op_falsy {
            *arg = (*arg).offset(1);
        }
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        (*evalarg_used).eval_flags = if if op_falsy as c_int != 0 {
            !result as c_int
        } else {
            result as c_int
        } != 0
        {
            orig_flags
        } else {
            orig_flags & !(EVAL_EVALUATE as c_int)
        };
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval1(arg, &raw mut var2, evalarg_used) == FAIL {
            (*evalarg_used).eval_flags = orig_flags;
            return FAIL;
        }
        if !op_falsy || !result {
            *rettv = var2;
        }
        if !op_falsy {
            p = *arg;
            if *p as c_int != ':' as c_int {
                emsg(gettext(
                    b"E109: Missing ':' after '?'\0".as_ptr() as *const c_char
                ));
                if evaluate as c_int != 0 && result as c_int != 0 {
                    tv_clear(rettv);
                }
                (*evalarg_used).eval_flags = orig_flags;
                return FAIL;
            }
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            (*evalarg_used).eval_flags = if !result {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as c_int)
            };
            if eval1(arg, &raw mut var2, evalarg_used) == FAIL {
                if evaluate as c_int != 0 && result as c_int != 0 {
                    tv_clear(rettv);
                }
                (*evalarg_used).eval_flags = orig_flags;
                return FAIL;
            }
            if evaluate as c_int != 0 && !result {
                *rettv = var2;
            }
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval2(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    if eval3(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    if *p.offset(0 as c_int as isize) as c_int == '|' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '|' as c_int
    {
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<c_void>(),
            eval_tofree: ::core::ptr::null_mut::<c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<c_void>(),
                eval_tofree: ::core::ptr::null_mut::<c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as c_int != 0;
        let mut result: bool = false_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if tv_get_number_chk(rettv, &raw mut error) != 0 as varnumber_T {
                result = true_0 != 0;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }
        while *p.offset(0 as c_int as isize) as c_int == '|' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '|' as c_int
        {
            *arg = skipwhite((*arg).offset(2 as c_int as isize));
            (*evalarg_used).eval_flags = if !result {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as c_int)
            };
            let mut var2: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval3(arg, &raw mut var2, evalarg_used) == FAIL {
                return FAIL;
            }
            if evaluate as c_int != 0 && !result {
                let mut error_0: bool = false_0 != 0;
                if tv_get_number_chk(&raw mut var2, &raw mut error_0) != 0 as varnumber_T {
                    result = true_0 != 0;
                }
                tv_clear(&raw mut var2);
                if error_0 {
                    return FAIL;
                }
            }
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = result as varnumber_T;
            }
            p = *arg;
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval3(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    if eval4(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    if *p.offset(0 as c_int as isize) as c_int == '&' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '&' as c_int
    {
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<c_void>(),
            eval_tofree: ::core::ptr::null_mut::<c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<c_void>(),
                eval_tofree: ::core::ptr::null_mut::<c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as c_int != 0;
        let mut result: bool = true_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if tv_get_number_chk(rettv, &raw mut error) == 0 as varnumber_T {
                result = false_0 != 0;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }
        while *p.offset(0 as c_int as isize) as c_int == '&' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '&' as c_int
        {
            *arg = skipwhite((*arg).offset(2 as c_int as isize));
            (*evalarg_used).eval_flags = if result as c_int != 0 {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as c_int)
            };
            let mut var2: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval4(arg, &raw mut var2, evalarg_used) == FAIL {
                return FAIL;
            }
            if evaluate as c_int != 0 && result as c_int != 0 {
                let mut error_0: bool = false_0 != 0;
                if tv_get_number_chk(&raw mut var2, &raw mut error_0) == 0 as varnumber_T {
                    result = false_0 != 0;
                }
                tv_clear(&raw mut var2);
                if error_0 {
                    return FAIL;
                }
            }
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = result as varnumber_T;
            }
            p = *arg;
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval4(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut type_0: exprtype_T = EXPR_UNKNOWN;
    let mut len: c_int = 2 as c_int;
    if eval5(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    match *p.offset(0 as c_int as isize) as c_int {
        61 => {
            if *p.offset(1 as c_int as isize) as c_int == '=' as c_int {
                type_0 = EXPR_EQUAL;
            } else if *p.offset(1 as c_int as isize) as c_int == '~' as c_int {
                type_0 = EXPR_MATCH;
            }
        }
        33 => {
            if *p.offset(1 as c_int as isize) as c_int == '=' as c_int {
                type_0 = EXPR_NEQUAL;
            } else if *p.offset(1 as c_int as isize) as c_int == '~' as c_int {
                type_0 = EXPR_NOMATCH;
            }
        }
        62 => {
            if *p.offset(1 as c_int as isize) as c_int != '=' as c_int {
                type_0 = EXPR_GREATER;
                len = 1 as c_int;
            } else {
                type_0 = EXPR_GEQUAL;
            }
        }
        60 => {
            if *p.offset(1 as c_int as isize) as c_int != '=' as c_int {
                type_0 = EXPR_SMALLER;
                len = 1 as c_int;
            } else {
                type_0 = EXPR_SEQUAL;
            }
        }
        105 => {
            if *p.offset(1 as c_int as isize) as c_int == 's' as c_int {
                if *p.offset(2 as c_int as isize) as c_int == 'n' as c_int
                    && *p.offset(3 as c_int as isize) as c_int == 'o' as c_int
                    && *p.offset(4 as c_int as isize) as c_int == 't' as c_int
                {
                    len = 5 as c_int;
                }
                if *(*__ctype_b_loc()).offset(*p.offset(len as isize) as uint8_t as c_int as isize)
                    as c_int
                    & _ISalnum as c_int as c_ushort as c_int
                    == 0
                    && *p.offset(len as isize) as c_int != '_' as c_int
                {
                    type_0 = (if len == 2 as c_int {
                        EXPR_IS as c_int
                    } else {
                        EXPR_ISNOT as c_int
                    }) as exprtype_T;
                }
            }
        }
        _ => {}
    }
    if type_0 as c_uint != EXPR_UNKNOWN as c_int as c_uint {
        let mut ic: bool = false;
        if *p.offset(len as isize) as c_int == '?' as c_int {
            ic = true_0 != 0;
            len += 1;
        } else if *p.offset(len as isize) as c_int == '#' as c_int {
            ic = false_0 != 0;
            len += 1;
        } else {
            ic = p_ic.get() != 0;
        }
        *arg = skipwhite(p.offset(len as isize));
        if eval5(arg, &raw mut var2, evalarg) == FAIL {
            tv_clear(rettv);
            return FAIL;
        }
        if !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0 {
            let ret: c_int = typval_compare(rettv, &raw mut var2, type_0, ic);
            tv_clear(&raw mut var2);
            return ret;
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval5(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    if eval6(arg, rettv, evalarg, false_0 != 0) == FAIL {
        return FAIL;
    }
    loop {
        let mut op: c_int = **arg as uint8_t as c_int;
        let mut concat: bool = op == '.' as c_int;
        if op != '+' as c_int && op != '-' as c_int && !concat {
            break;
        }
        let evaluate: bool = if evalarg.is_null() {
            0 as c_int
        } else {
            (*evalarg).eval_flags & EVAL_EVALUATE as c_int
        } != 0;
        if (op != '+' as c_int
            || (*rettv).v_type as c_uint != VAR_LIST as c_int as c_uint
                && (*rettv).v_type as c_uint != VAR_BLOB as c_int as c_uint)
            && (op == '.' as c_int || (*rettv).v_type as c_uint != VAR_FLOAT as c_int as c_uint)
            && evaluate as c_int != 0
        {
            if op == '.' as c_int && !tv_check_str(rettv)
                || op != '.' as c_int && !tv_check_num(rettv)
            {
                tv_clear(rettv);
                return FAIL;
            }
        }
        if op == '.' as c_int && *(*arg).offset(1 as c_int as isize) as c_int == '.' as c_int {
            *arg = (*arg).offset(1);
        }
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval6(arg, &raw mut var2, evalarg, op == '.' as c_int) == FAIL {
            tv_clear(rettv);
            return FAIL;
        }
        if evaluate {
            if op == '.' as c_int {
                if eval_concat_str(rettv, &raw mut var2) == FAIL {
                    return FAIL;
                }
            } else if op == '+' as c_int
                && (*rettv).v_type as c_uint == VAR_BLOB as c_int as c_uint
                && var2.v_type as c_uint == VAR_BLOB as c_int as c_uint
            {
                eval_addblob(rettv, &raw mut var2);
            } else if op == '+' as c_int
                && (*rettv).v_type as c_uint == VAR_LIST as c_int as c_uint
                && var2.v_type as c_uint == VAR_LIST as c_int as c_uint
            {
                if eval_addlist(rettv, &raw mut var2) == FAIL {
                    return FAIL;
                }
            } else if eval_addsub_number(rettv, &raw mut var2, op) == FAIL {
                return FAIL;
            }
            tv_clear(&raw mut var2);
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval6(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut want_string: bool,
) -> c_int {
    if eval7(arg, rettv, evalarg, want_string) == FAIL {
        return FAIL;
    }
    loop {
        let mut op: c_int = **arg as uint8_t as c_int;
        if op != '*' as c_int && op != '/' as c_int && op != '%' as c_int {
            break;
        }
        let evaluate: bool = if evalarg.is_null() {
            0 as c_int
        } else {
            (*evalarg).eval_flags & EVAL_EVALUATE as c_int
        } != 0;
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval7(arg, &raw mut var2, evalarg, false_0 != 0) == FAIL {
            return FAIL;
        }
        if evaluate {
            if eval_multdiv_number(rettv, &raw mut var2, op) == FAIL {
                return FAIL;
            }
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval7(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut want_string: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    let mut ret: c_int = OK;
    static recurse: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    (*rettv).v_type = VAR_UNKNOWN;
    let mut start_leader: *const c_char = *arg;
    while **arg as c_int == '!' as c_int
        || **arg as c_int == '-' as c_int
        || **arg as c_int == '+' as c_int
    {
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
    }
    let mut end_leader: *const c_char = *arg;
    if recurse.get() == 1000 as c_int {
        semsg(
            gettext((e_expression_too_recursive_str.ptr() as *const _) as *const c_char),
            *arg,
        );
        return FAIL;
    }
    (*recurse.ptr()) += 1;
    match **arg as c_int {
        48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
            ret = eval_number(arg, rettv, evaluate, want_string);
            if ret == OK && evaluate as c_int != 0 && end_leader > start_leader {
                ret = eval7_leader(rettv, true_0 != 0, start_leader, &raw mut end_leader);
            }
        }
        34 => {
            ret = eval_string(arg, rettv, evaluate, false_0 != 0);
        }
        39 => {
            ret = eval_lit_string(arg, rettv, evaluate, false_0 != 0);
        }
        91 => {
            ret = eval_list(arg, rettv, evalarg);
        }
        35 => {
            ret = eval_lit_dict(arg, rettv, evalarg);
        }
        123 => {
            ret = get_lambda_tv(arg, rettv, evalarg);
            if ret == NOTDONE {
                ret = eval_dict(arg, rettv, evalarg, false_0 != 0);
            }
        }
        38 => {
            ret = eval_option(arg as *mut *const c_char, rettv, evaluate);
        }
        36 => {
            if *(*arg).offset(1 as c_int as isize) as c_int == '"' as c_int
                || *(*arg).offset(1 as c_int as isize) as c_int == '\'' as c_int
            {
                ret = eval_interp_string(arg, rettv, evaluate);
            } else {
                ret = eval_env_var(arg, rettv, evaluate as c_int);
            }
        }
        64 => {
            *arg = (*arg).offset(1);
            if evaluate {
                (*rettv).v_type = VAR_STRING;
                (*rettv).vval.v_string =
                    get_reg_contents(**arg as c_int, kGRegExprSrc as c_int) as *mut c_char;
            }
            if **arg as c_int != NUL {
                *arg = (*arg).offset(1);
            }
        }
        40 => {
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            ret = eval1(arg, rettv, evalarg);
            if **arg as c_int == ')' as c_int {
                *arg = (*arg).offset(1);
            } else if ret == OK {
                emsg(gettext(b"E110: Missing ')'\0".as_ptr() as *const c_char));
                tv_clear(rettv);
                ret = FAIL;
            }
        }
        _ => {
            ret = NOTDONE;
        }
    }
    if ret == NOTDONE {
        let mut s: *mut c_char = *arg;
        let mut alias: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut len: c_int = get_name_len(
            arg as *mut *const c_char,
            &raw mut alias,
            evaluate,
            true_0 != 0,
        );
        if !alias.is_null() {
            s = alias;
        }
        if len <= 0 as c_int {
            ret = FAIL;
        } else {
            let flags: c_int = if evalarg.is_null() {
                0 as c_int
            } else {
                (*evalarg).eval_flags
            };
            if *skipwhite(*arg) as c_int == '(' as c_int {
                *arg = skipwhite(*arg);
                ret = eval_func(
                    arg,
                    evalarg,
                    s,
                    len,
                    rettv,
                    flags,
                    ::core::ptr::null_mut::<typval_T>(),
                );
            } else if evaluate {
                ret = eval_variable(
                    s,
                    len,
                    rettv,
                    ::core::ptr::null_mut::<*mut dictitem_T>(),
                    true_0 != 0,
                    false_0 != 0,
                );
            } else {
                check_vars(s, len as size_t);
                if (*rettv).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint
                    && !evaluate
                    && strnequal(s, b"v:lua.\0".as_ptr() as *const c_char, 6 as size_t) as c_int
                        != 0
                {
                    (*rettv).v_type = VAR_PARTIAL;
                    (*rettv).vval.v_partial = get_vim_var_partial(VV_LUA);
                    (*(*rettv).vval.v_partial).pt_refcount += 1;
                }
                ret = OK;
            }
        }
        xfree(alias as *mut c_void);
    }
    *arg = skipwhite(*arg);
    if ret == OK {
        ret = handle_subscript(arg as *mut *const c_char, rettv, evalarg, true_0 != 0);
    }
    if ret == OK && evaluate as c_int != 0 && end_leader > start_leader {
        ret = eval7_leader(rettv, false_0 != 0, start_leader, &raw mut end_leader);
    }
    (*recurse.ptr()) -= 1;
    return ret;
}

pub(crate) unsafe extern "C" fn eval7_leader(
    rettv: *mut typval_T,
    numeric_only: bool,
    start_leader: *const c_char,
    end_leaderp: *mut *const c_char,
) -> c_int {
    let mut end_leader: *const c_char = *end_leaderp;
    let mut ret: c_int = OK;
    let mut error: bool = false_0 != 0;
    let mut val: varnumber_T = 0 as varnumber_T;
    let mut f: float_T = 0.0f64;
    if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f = (*rettv).vval.v_float;
    } else {
        val = tv_get_number_chk(rettv, &raw mut error);
    }
    if error {
        tv_clear(rettv);
        ret = FAIL;
    } else {
        while end_leader > start_leader {
            end_leader = end_leader.offset(-1);
            if *end_leader as c_int == '!' as c_int {
                if numeric_only {
                    end_leader = end_leader.offset(1);
                    break;
                } else if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
                    (*rettv).v_type = VAR_BOOL;
                    val = (if f == 0.0f64 {
                        kBoolVarTrue as c_int
                    } else {
                        kBoolVarFalse as c_int
                    }) as varnumber_T;
                } else {
                    val = (val == 0) as c_int as varnumber_T;
                }
            } else if *end_leader as c_int == '-' as c_int {
                if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
                    f = -f;
                } else {
                    val = -val;
                }
            }
        }
        if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
            tv_clear(rettv);
            (*rettv).vval.v_float = f;
        } else {
            tv_clear(rettv);
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = val;
        }
    }
    *end_leaderp = end_leader;
    return ret;
}
