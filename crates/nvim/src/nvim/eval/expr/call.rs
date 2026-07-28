//! Calling something: a function name, a method, a lambda or a partial.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn call_func_rettv(
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    rettv: *mut typval_T,
    evaluate: bool,
    selfdict: *mut dict_T,
    basetv: *mut typval_T,
    lua_funcname: *const c_char,
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
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut functv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut funcname: *const c_char = ::core::ptr::null::<c_char>();
    let mut is_lua: bool = false_0 != 0;
    let mut ret: c_int = 0;
    '_theend: {
        if evaluate {
            functv = *rettv;
            (*rettv).v_type = VAR_UNKNOWN;
            if functv.v_type as c_uint == VAR_PARTIAL as c_int as c_uint {
                pt = functv.vval.v_partial;
                is_lua = is_luafunc(pt);
                funcname = if is_lua as c_int != 0 {
                    lua_funcname
                } else {
                    partial_name(pt) as *const c_char
                };
            } else {
                funcname = functv.vval.v_string;
                if funcname.is_null() || *funcname as c_int == NUL {
                    emsg(gettext(
                        (e_empty_function_name.ptr() as *const _) as *const c_char,
                    ));
                    ret = FAIL;
                    break '_theend;
                }
            }
        } else {
            funcname = b"\0".as_ptr() as *const c_char;
        }
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = evaluate;
        funcexe.fe_partial = pt;
        funcexe.fe_selfdict = selfdict;
        funcexe.fe_basetv = basetv;
        ret = get_func_tv(
            funcname,
            if is_lua as c_int != 0 {
                (*arg).offset_from(funcname) as c_int
            } else {
                -1 as c_int
            },
            rettv,
            arg,
            evalarg,
            &raw mut funcexe,
        );
    }
    if evaluate {
        tv_clear(&raw mut functv);
    }
    return ret;
}

pub(crate) unsafe extern "C" fn eval_lambda(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    *arg = (*arg).offset(2 as c_int as isize);
    let mut base: typval_T = *rettv;
    (*rettv).v_type = VAR_UNKNOWN;
    let mut ret: c_int = get_lambda_tv(arg, rettv, evalarg);
    if ret != OK {
        return FAIL;
    } else if **arg as c_int != '(' as c_int {
        if verbose {
            if *skipwhite(*arg) as c_int == '(' as c_int {
                emsg(gettext(e_nowhitespace.get()));
            } else {
                semsg(
                    gettext(&raw const e_missingparen as *const c_char),
                    b"lambda\0".as_ptr() as *const c_char,
                );
            }
        }
        tv_clear(rettv);
        ret = FAIL;
    } else {
        ret = call_func_rettv(
            arg,
            evalarg,
            rettv,
            evaluate,
            ::core::ptr::null_mut::<dict_T>(),
            &raw mut base,
            ::core::ptr::null::<c_char>(),
        );
    }
    if evaluate {
        tv_clear(&raw mut base);
    }
    return ret;
}

pub(crate) unsafe extern "C" fn eval_method(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    *arg = (*arg).offset(2 as c_int as isize);
    let mut base: typval_T = *rettv;
    (*rettv).v_type = VAR_UNKNOWN;
    let mut len: c_int = 0;
    let mut name: *mut c_char = *arg;
    let mut lua_funcname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut alias: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if strnequal(name, b"v:lua.\0".as_ptr() as *const c_char, 6 as size_t) {
        lua_funcname = name.offset(6 as c_int as isize);
        *arg = skip_luafunc_name(lua_funcname) as *mut c_char;
        *arg = skipwhite(*arg);
        len = (*arg).offset_from(lua_funcname) as c_int;
    } else {
        len = get_name_len(
            arg as *mut *const c_char,
            &raw mut alias,
            evaluate,
            true_0 != 0,
        );
        if !alias.is_null() {
            name = alias;
        }
    }
    let mut tofree: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut ret: c_int = OK;
    if len <= 0 as c_int {
        if verbose {
            if lua_funcname.is_null() {
                emsg(gettext(
                    b"E260: Missing name after ->\0".as_ptr() as *const c_char
                ));
            } else {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), name);
            }
        }
        ret = FAIL;
    } else {
        *arg = skipwhite(*arg);
        let mut paren: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if **arg as c_int != '(' as c_int && lua_funcname.is_null() && alias.is_null() && {
            paren = vim_strchr(*arg, '(' as c_int);
            !paren.is_null()
        } {
            *arg = name;
            *paren = NUL as c_char;
            let mut ref_0: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            ref_0.v_type = VAR_UNKNOWN;
            if eval7(arg, &raw mut ref_0, evalarg, false_0 != 0) == FAIL {
                *arg = name.offset(len as isize);
                ret = FAIL;
            } else if *skipwhite(*arg) as c_int != NUL {
                if verbose {
                    semsg(gettext(&raw const e_trailing_arg as *const c_char), *arg);
                }
                ret = FAIL;
            } else if ref_0.v_type as c_uint == VAR_FUNC as c_int as c_uint
                && !ref_0.vval.v_string.is_null()
            {
                name = ref_0.vval.v_string;
                ref_0.vval.v_string = ::core::ptr::null_mut::<c_char>();
                tofree = name;
                len = strlen(name) as c_int;
            } else if ref_0.v_type as c_uint == VAR_PARTIAL as c_int as c_uint
                && !ref_0.vval.v_partial.is_null()
            {
                if (*ref_0.vval.v_partial).pt_argc > 0 as c_int
                    || !(*ref_0.vval.v_partial).pt_dict.is_null()
                {
                    if verbose {
                        emsg(gettext(
                            (e_cannot_use_partial_here.ptr() as *const _) as *const c_char,
                        ));
                    }
                    ret = FAIL;
                } else {
                    name = xstrdup(partial_name(ref_0.vval.v_partial));
                    tofree = name;
                    if name.is_null() {
                        ret = FAIL;
                        name = *arg;
                    } else {
                        len = strlen(name) as c_int;
                    }
                }
            } else {
                if verbose {
                    semsg(
                        gettext(&raw const e_not_callable_type_str as *const c_char),
                        name,
                    );
                }
                ret = FAIL;
            }
            tv_clear(&raw mut ref_0);
            *paren = '(' as c_char;
        }
        if ret == OK {
            if **arg as c_int != '(' as c_int {
                if verbose {
                    semsg(gettext(&raw const e_missingparen as *const c_char), name);
                }
                ret = FAIL;
            } else if ascii_iswhite(*(*arg).offset(-1 as c_int as isize) as c_int) {
                if verbose {
                    emsg(gettext(e_nowhitespace.get()));
                }
                ret = FAIL;
            } else if !lua_funcname.is_null() {
                if evaluate {
                    (*rettv).v_type = VAR_PARTIAL;
                    (*rettv).vval.v_partial = get_vim_var_partial(VV_LUA);
                    (*(*rettv).vval.v_partial).pt_refcount += 1;
                }
                ret = call_func_rettv(
                    arg,
                    evalarg,
                    rettv,
                    evaluate,
                    ::core::ptr::null_mut::<dict_T>(),
                    &raw mut base,
                    lua_funcname,
                );
            } else {
                ret = eval_func(
                    arg,
                    evalarg,
                    name,
                    len,
                    rettv,
                    if evaluate as c_int != 0 {
                        EVAL_EVALUATE as c_int
                    } else {
                        0 as c_int
                    },
                    &raw mut base,
                );
            }
        }
    }
    if evaluate {
        tv_clear(&raw mut base);
    }
    xfree(tofree as *mut c_void);
    if !alias.is_null() {
        xfree(alias as *mut c_void);
    }
    return ret;
}

pub unsafe extern "C" fn partial_name(mut pt: *mut partial_T) -> *mut c_char {
    if !pt.is_null() {
        if !(*pt).pt_name.is_null() {
            return (*pt).pt_name;
        }
        if !(*pt).pt_func.is_null() {
            return &raw mut (*(*pt).pt_func).uf_name as *mut c_char;
        }
    }
    return b"\0".as_ptr() as *const c_char as *mut c_char;
}

pub(crate) unsafe extern "C" fn partial_free(mut pt: *mut partial_T) {
    let mut i: c_int = 0 as c_int;
    while i < (*pt).pt_argc {
        tv_clear((*pt).pt_argv.offset(i as isize));
        i += 1;
    }
    xfree((*pt).pt_argv as *mut c_void);
    tv_dict_unref((*pt).pt_dict);
    if !(*pt).pt_name.is_null() {
        func_unref((*pt).pt_name);
        xfree((*pt).pt_name as *mut c_void);
    } else {
        func_ptr_unref((*pt).pt_func);
    }
    xfree(pt as *mut c_void);
}

pub unsafe extern "C" fn partial_unref(mut pt: *mut partial_T) {
    if pt.is_null() {
        return;
    }
    (*pt).pt_refcount -= 1;
    if (*pt).pt_refcount <= 0 as c_int {
        partial_free(pt);
    }
}
