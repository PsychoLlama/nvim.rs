//! What the rest of the editor calls the evaluator through.
//!
//! Every entry point here brackets one evaluation: it sets up an `evalarg_T`,
//! runs `eval0`, converts the result to whatever the caller wanted, and
//! clears the typval on both the success and the error path.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_v_event(mut sve: *mut save_v_event_T) -> *mut dict_T {
    let mut v_event: *mut dict_T = get_vim_var_dict(VV_EVENT);
    if (*v_event).dv_hashtab.ht_used > 0 as size_t {
        (*sve).sve_did_save = true_0 != 0;
        (*sve).sve_hashtab = (*v_event).dv_hashtab;
        hash_init(&raw mut (*v_event).dv_hashtab);
    } else {
        (*sve).sve_did_save = false_0 != 0;
    }
    return v_event;
}

pub unsafe extern "C" fn restore_v_event(mut v_event: *mut dict_T, mut sve: *mut save_v_event_T) {
    tv_dict_free_contents(v_event);
    if (*sve).sve_did_save {
        (*v_event).dv_hashtab = (*sve).sve_hashtab;
    } else {
        hash_init(&raw mut (*v_event).dv_hashtab);
    };
}

pub unsafe extern "C" fn eval_init() {
    evalvars_init();
    func_init();
}

pub unsafe extern "C" fn fill_evalarg_from_eap(
    mut evalarg: *mut evalarg_T,
    mut eap: *mut exarg_T,
    mut skip: bool,
) {
    *evalarg = evalarg_T {
        eval_flags: if skip as c_int != 0 {
            0 as c_int
        } else {
            EVAL_EVALUATE as c_int
        },
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    if eap.is_null() {
        return;
    }
    if sourcing_a_script(eap) != 0 {
        (*evalarg).eval_getline = (*eap).ea_getline;
        (*evalarg).eval_cookie = (*eap).cookie;
    }
}

pub unsafe extern "C" fn eval_to_bool(
    mut arg: *mut c_char,
    mut error: *mut bool,
    mut eap: *mut exarg_T,
    skip: bool,
    use_simple_function: bool,
) -> bool {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: bool = false_0 != 0;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
    if skip {
        (*emsg_skip.ptr()) += 1;
    }
    let mut r: c_int = if use_simple_function as c_int != 0 {
        eval0_simple_funccal(arg, &raw mut tv, eap, &raw mut evalarg)
    } else {
        eval0(arg, &raw mut tv, eap, &raw mut evalarg)
    };
    if r == FAIL {
        *error = true_0 != 0;
    } else {
        *error = false_0 != 0;
        if !skip {
            retval = tv_get_number_chk(&raw mut tv, error) != 0 as varnumber_T;
            tv_clear(&raw mut tv);
        }
    }
    if skip {
        (*emsg_skip.ptr()) -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return retval;
}

pub(crate) unsafe extern "C" fn eval1_emsg(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
) -> c_int {
    let start: *const c_char = *arg;
    let did_emsg_before: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let ret: c_int = eval1(arg, rettv, &raw mut evalarg);
    if ret == FAIL {
        if !aborting()
            && did_emsg.get() == did_emsg_before
            && called_emsg.get() == called_emsg_before
        {
            semsg(gettext(&raw const e_invexpr2 as *const c_char), start);
        }
    }
    clear_evalarg(&raw mut evalarg, eap);
    return ret;
}

pub unsafe extern "C" fn eval_expr_valid_arg(tv: *const typval_T) -> bool {
    return (*tv).v_type as c_uint != VAR_UNKNOWN as c_int as c_uint
        && ((*tv).v_type as c_uint != VAR_STRING as c_int as c_uint
            || !(*tv).vval.v_string.is_null() && *(*tv).vval.v_string as c_int != NUL);
}

pub(crate) unsafe extern "C" fn eval_expr_partial(
    mut expr: *const typval_T,
    mut argv: *mut typval_T,
    mut argc: c_int,
    mut rettv: *mut typval_T,
) -> c_int {
    let partial: *mut partial_T = (*expr).vval.v_partial;
    if partial.is_null() {
        return FAIL;
    }
    let s: *const c_char = partial_name(partial);
    if s.is_null() || *s as c_int == NUL {
        return FAIL;
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_partial = partial;
    if call_func(s, -1 as c_int, rettv, argc, argv, &raw mut funcexe) == FAIL {
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval_expr_func(
    mut expr: *const typval_T,
    mut argv: *mut typval_T,
    mut argc: c_int,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut buf: [c_char; 65] = [0; 65];
    let s: *const c_char = if (*expr).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        (*expr).vval.v_string as *const c_char
    } else {
        tv_get_string_buf_chk(expr, &raw mut buf as *mut c_char)
    };
    if s.is_null() || *s as c_int == NUL {
        return FAIL;
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true_0 != 0;
    if call_func(s, -1 as c_int, rettv, argc, argv, &raw mut funcexe) == FAIL {
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn eval_expr_string(
    mut expr: *const typval_T,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut buf: [c_char; 65] = [0; 65];
    let mut s: *mut c_char =
        tv_get_string_buf_chk(expr, &raw mut buf as *mut c_char) as *mut c_char;
    if s.is_null() {
        return FAIL;
    }
    s = skipwhite(s);
    if eval1_emsg(&raw mut s, rettv, ::core::ptr::null_mut::<exarg_T>()) == FAIL {
        return FAIL;
    }
    if *skipwhite(s) as c_int != NUL {
        tv_clear(rettv);
        semsg(gettext(&raw const e_invexpr2 as *const c_char), s);
        return FAIL;
    }
    return OK;
}

pub unsafe extern "C" fn eval_expr_typval(
    mut expr: *const typval_T,
    mut want_func: bool,
    mut argv: *mut typval_T,
    mut argc: c_int,
    mut rettv: *mut typval_T,
) -> c_int {
    if (*expr).v_type as c_uint == VAR_PARTIAL as c_int as c_uint {
        return eval_expr_partial(expr, argv, argc, rettv);
    }
    if (*expr).v_type as c_uint == VAR_FUNC as c_int as c_uint || want_func as c_int != 0 {
        return eval_expr_func(expr, argv, argc, rettv);
    }
    return eval_expr_string(expr, rettv);
}

pub unsafe extern "C" fn eval_expr_to_bool(
    mut expr: *const typval_T,
    mut error: *mut bool,
) -> bool {
    let mut argv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if eval_expr_typval(
        expr,
        false_0 != 0,
        &raw mut argv,
        0 as c_int,
        &raw mut rettv,
    ) == FAIL
    {
        *error = true_0 != 0;
        return false_0 != 0;
    }
    let res: bool = tv_get_number_chk(&raw mut rettv, error) != 0 as varnumber_T;
    tv_clear(&raw mut rettv);
    return res;
}

pub unsafe extern "C" fn eval_to_string_skip(
    mut arg: *mut c_char,
    mut eap: *mut exarg_T,
    skip: bool,
) -> *mut c_char {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
    if skip {
        (*emsg_skip.ptr()) += 1;
    }
    if eval0(arg, &raw mut tv, eap, &raw mut evalarg) == FAIL || skip as c_int != 0 {
        retval = ::core::ptr::null_mut::<c_char>();
    } else {
        retval = xstrdup(tv_get_string(&raw mut tv));
        tv_clear(&raw mut tv);
    }
    if skip {
        (*emsg_skip.ptr()) -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return retval;
}

pub unsafe extern "C" fn skip_expr(mut pp: *mut *mut c_char, evalarg: *mut evalarg_T) -> c_int {
    let save_flags: c_int = if evalarg.is_null() {
        0 as c_int
    } else {
        (*evalarg).eval_flags
    };
    if !evalarg.is_null() {
        (*evalarg).eval_flags &= !(EVAL_EVALUATE as c_int);
    }
    *pp = skipwhite(*pp);
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut res: c_int = eval1(pp, &raw mut rettv, ::core::ptr::null_mut::<evalarg_T>());
    if !evalarg.is_null() {
        (*evalarg).eval_flags = save_flags;
    }
    return res;
}

pub(crate) unsafe extern "C" fn typval2string(
    mut tv: *mut typval_T,
    mut join_list: bool,
) -> *mut c_char {
    if join_list as c_int != 0 && (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<c_char>() as c_int,
            80 as c_int,
        );
        if !(*tv).vval.v_list.is_null() {
            tv_list_join(
                &raw mut ga,
                (*tv).vval.v_list,
                b"\n\0".as_ptr() as *const c_char,
            );
            if tv_list_len((*tv).vval.v_list) > 0 as c_int {
                ga_append(&raw mut ga, NL as uint8_t);
            }
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        return ga.ga_data as *mut c_char;
    } else if (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint
        || (*tv).v_type as c_uint == VAR_DICT as c_int as c_uint
    {
        return encode_tv2string(tv, ::core::ptr::null_mut::<size_t>());
    }
    return xstrdup(tv_get_string(tv));
}

pub unsafe extern "C" fn eval_to_string_eap(
    mut arg: *mut c_char,
    join_list: bool,
    mut eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut c_char {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let mut r: c_int = if use_simple_function as c_int != 0 {
        eval0_simple_funccal(
            arg,
            &raw mut tv,
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut evalarg,
        )
    } else {
        eval0(
            arg,
            &raw mut tv,
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut evalarg,
        )
    };
    if r == FAIL {
        retval = ::core::ptr::null_mut::<c_char>();
    } else {
        retval = typval2string(&raw mut tv, join_list);
        tv_clear(&raw mut tv);
    }
    clear_evalarg(&raw mut evalarg, ::core::ptr::null_mut::<exarg_T>());
    return retval;
}

pub unsafe extern "C" fn eval_to_string(
    mut arg: *mut c_char,
    join_list: bool,
    use_simple_function: bool,
) -> *mut c_char {
    return eval_to_string_eap(
        arg,
        join_list,
        ::core::ptr::null_mut::<exarg_T>(),
        use_simple_function,
    );
}

pub unsafe extern "C" fn eval_to_string_safe(
    mut arg: *mut c_char,
    use_sandbox: bool,
    use_simple_function: bool,
) -> *mut c_char {
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    (*textlock.ptr()) += 1;
    retval = eval_to_string(arg, false_0 != 0, use_simple_function);
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    (*textlock.ptr()) -= 1;
    restore_funccal();
    return retval;
}

pub unsafe extern "C" fn eval_to_number(
    mut expr: *mut c_char,
    use_simple_function: bool,
) -> varnumber_T {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: varnumber_T = 0;
    let mut p: *mut c_char = skipwhite(expr);
    let mut r: c_int = NOTDONE;
    (*emsg_off.ptr()) += 1;
    if use_simple_function {
        r = may_call_simple_func(expr, &raw mut rettv);
    }
    if r == NOTDONE {
        r = eval1(&raw mut p, &raw mut rettv, EVALARG_EVALUATE.ptr());
    }
    if r == FAIL {
        retval = -1 as varnumber_T;
    } else {
        retval = tv_get_number_chk(&raw mut rettv, ::core::ptr::null_mut::<bool>());
        tv_clear(&raw mut rettv);
    }
    (*emsg_off.ptr()) -= 1;
    return retval;
}

pub unsafe extern "C" fn eval_expr(mut arg: *mut c_char, mut eap: *mut exarg_T) -> *mut typval_T {
    return eval_expr_ext(arg, eap, false_0 != 0);
}

pub unsafe extern "C" fn eval_expr_ext(
    mut arg: *mut c_char,
    mut eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut typval_T {
    let mut tv: *mut typval_T = xmalloc(::core::mem::size_of::<typval_T>()) as *mut typval_T;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let mut r: c_int = NOTDONE;
    if use_simple_function {
        r = eval0_simple_funccal(arg, tv, eap, &raw mut evalarg);
    }
    if r == NOTDONE {
        r = eval0(arg, tv, eap, &raw mut evalarg);
    }
    if r == FAIL {
        let mut ptr_: *mut *mut c_void = &raw mut tv as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return tv;
}

pub unsafe extern "C" fn call_vim_function(
    mut func: *const c_char,
    mut argc: c_int,
    mut argv: *mut typval_T,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut ret: c_int = 0;
    let mut len: c_int = strlen(func) as c_int;
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    '_fail: {
        if len >= 6 as c_int
            && memcmp(
                func as *const c_void,
                b"v:lua.\0".as_ptr() as *const c_char as *const c_void,
                6 as size_t,
            ) == 0
        {
            func = func.offset(6 as c_int as isize);
            len = check_luafunc_name(func, false_0 != 0);
            if len == 0 as c_int {
                ret = FAIL;
                break '_fail;
            } else {
                pt = get_vim_var_partial(VV_LUA);
            }
        }
        (*rettv).v_type = VAR_UNKNOWN;
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = true_0 != 0;
        funcexe.fe_partial = pt;
        ret = call_func(func, len, rettv, argc, argv, &raw mut funcexe);
    }
    if ret == FAIL {
        tv_clear(rettv);
    }
    return ret;
}

pub unsafe extern "C" fn call_func_retstr(
    func: *const c_char,
    mut argc: c_int,
    mut argv: *mut typval_T,
) -> *mut c_void {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
        return NULL_0;
    }
    let retval: *mut c_char = xstrdup(tv_get_string(&raw mut rettv));
    tv_clear(&raw mut rettv);
    return retval as *mut c_void;
}

pub unsafe extern "C" fn call_func_retlist(
    mut func: *const c_char,
    mut argc: c_int,
    mut argv: *mut typval_T,
) -> *mut c_void {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
        return NULL_0;
    }
    if rettv.v_type as c_uint != VAR_LIST as c_int as c_uint {
        tv_clear(&raw mut rettv);
        return NULL_0;
    }
    return rettv.vval.v_list as *mut c_void;
}

pub unsafe extern "C" fn eval_foldexpr(mut wp: *mut win_T, mut cp: *mut c_int) -> c_int {
    let saved_sctx: sctx_T = current_sctx.get();
    let use_sandbox: bool = was_set_insecurely(wp, kOptFoldexpr, OPT_LOCAL as c_int);
    let mut arg: *mut c_char = skipwhite((*wp).w_onebuf_opt.wo_fde);
    current_sctx.set((*wp).w_onebuf_opt.wo_script_ctx[kWinOptFoldexpr as c_int as usize]);
    (*emsg_off.ptr()) += 1;
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    (*textlock.ptr()) += 1;
    *cp = NUL;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: varnumber_T = 0;
    if eval0_simple_funccal(
        arg,
        &raw mut tv,
        ::core::ptr::null_mut::<exarg_T>(),
        EVALARG_EVALUATE.ptr(),
    ) == FAIL
    {
        retval = 0 as varnumber_T;
    } else {
        if tv.v_type as c_uint == VAR_NUMBER as c_int as c_uint {
            retval = tv.vval.v_number;
        } else if tv.v_type as c_uint != VAR_STRING as c_int as c_uint || tv.vval.v_string.is_null()
        {
            retval = 0 as varnumber_T;
        } else {
            let mut s: *mut c_char = tv.vval.v_string;
            if *s as c_int != NUL && !ascii_isdigit(*s as c_int) && *s as c_int != '-' as c_int {
                let c2rust_fresh10 = s;
                s = s.offset(1);
                *cp = *c2rust_fresh10 as uint8_t as c_int;
            }
            retval = atol(s) as varnumber_T;
        }
        tv_clear(&raw mut tv);
    }
    (*emsg_off.ptr()) -= 1;
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    (*textlock.ptr()) -= 1;
    clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
    current_sctx.set(saved_sctx);
    return retval as c_int;
}

pub unsafe extern "C" fn eval_foldtext(mut wp: *mut win_T) -> Object {
    let use_sandbox: bool = was_set_insecurely(wp, kOptFoldtext, OPT_LOCAL as c_int);
    let mut arg: *mut c_char = (*wp).w_onebuf_opt.wo_fdt;
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    (*textlock.ptr()) += 1;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: Object = Object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    };
    if eval0_simple_funccal(
        arg,
        &raw mut tv,
        ::core::ptr::null_mut::<exarg_T>(),
        EVALARG_EVALUATE.ptr(),
    ) == FAIL
    {
        retval = object {
            type_0: kObjectTypeString,
            data: object_data {
                string: String_0 {
                    data: ::core::ptr::null_mut::<c_char>(),
                    size: 0 as size_t,
                },
            },
        };
    } else {
        if tv.v_type as c_uint == VAR_LIST as c_int as c_uint {
            retval = vim_to_object(&raw mut tv, ::core::ptr::null_mut::<Arena>(), false_0 != 0);
        } else {
            retval = object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: cstr_to_string(tv_get_string(&raw mut tv)),
                },
            };
        }
        tv_clear(&raw mut tv);
    }
    clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    (*textlock.ptr()) -= 1;
    restore_funccal();
    return retval;
}

pub unsafe extern "C" fn set_argv_var(mut argv: *mut *mut c_char, mut argc: c_int) {
    let mut l: *mut list_T = tv_list_alloc(argc as ptrdiff_t);
    tv_list_set_lock(l, VAR_FIXED);
    let mut i: c_int = 0 as c_int;
    while i < argc {
        tv_list_append_string(l, *argv.offset(i as isize) as *const c_char, -1 as ssize_t);
        (*tv_list_last(l)).li_tv.v_lock = VAR_FIXED;
        i += 1;
    }
    set_vim_var_list(VV_ARGV, l);
}

pub unsafe extern "C" fn typval_tostring(mut arg: *mut typval_T, mut quotes: bool) -> *mut c_char {
    if arg.is_null() {
        return xstrdup(b"(does not exist)\0".as_ptr() as *const c_char);
    }
    if !quotes && (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint {
        return xstrdup(if (*arg).vval.v_string.is_null() {
            b"\0".as_ptr() as *const c_char
        } else {
            (*arg).vval.v_string as *const c_char
        });
    }
    return encode_tv2string(arg, ::core::ptr::null_mut::<size_t>());
}

#[inline]
pub(crate) unsafe extern "C" fn tv_init(tv: *mut typval_T) {
    if !tv.is_null() {
        memset(
            tv as *mut c_void,
            0 as c_int,
            ::core::mem::size_of::<typval_T>(),
        );
    }
}
