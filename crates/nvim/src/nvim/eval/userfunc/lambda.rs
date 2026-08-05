//! Lambdas, closures and partials -- the anonymous half.
//!
//! `get_lambda_tv` parses `{x -> expr}` into a real `ufunc_T` with a
//! generated `<lambda>N` name and, if it captured anything, a reference to
//! the funccall it was made in (`register_closure`).  `make_partial` is the
//! other way a callable carries state: a bound dictionary, bound arguments,
//! or both.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn register_closure(mut fp: *mut ufunc_T) {
    unsafe {
        if (*fp).uf_scoped == current_funccal.get() {
            return;
        }
        funccal_unref((*fp).uf_scoped, fp, false_0 != 0);
        (*fp).uf_scoped = current_funccal.get();
        (*current_funccal.get()).fc_refcount += 1;
        ga_grow(
            &raw mut (*current_funccal.get()).fc_ufuncs,
            1 as ::core::ffi::c_int,
        );
        let c2rust_fresh1 = (*current_funccal.get()).fc_ufuncs.ga_len;
        (*current_funccal.get()).fc_ufuncs.ga_len = (*current_funccal.get()).fc_ufuncs.ga_len + 1;
        let c2rust_lvalue_ptr = &raw mut *((*current_funccal.get()).fc_ufuncs.ga_data
            as *mut *mut ufunc_T)
            .offset(c2rust_fresh1 as isize);
        *c2rust_lvalue_ptr = fp;
    }
}

static lambda_name: GlobalCell<[::core::ffi::c_char; 73]> = GlobalCell::new([0; 73]);

unsafe extern "C" fn get_lambda_name() -> String_0 {
    unsafe {
        static lambda_no: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        (*lambda_no.ptr()) += 1;
        let mut n: ::core::ffi::c_int = snprintf(
            lambda_name.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 73]>(),
            b"<lambda>%d\0".as_ptr() as *const ::core::ffi::c_char,
            lambda_no.get(),
        );
        return String_0 {
            data: lambda_name.ptr() as *mut ::core::ffi::c_char,
            size: if n < 1 as ::core::ffi::c_int {
                0 as size_t
            } else {
                (if n < ::core::mem::size_of::<[::core::ffi::c_char; 73]>() as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                {
                    n
                } else {
                    ::core::mem::size_of::<[::core::ffi::c_char; 73]>() as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                }) as size_t
            },
        };
    }
}

pub(crate) unsafe extern "C" fn alloc_ufunc(
    mut name: *const ::core::ffi::c_char,
    mut namelen: size_t,
) -> *mut ufunc_T {
    unsafe {
        let mut len: size_t = (240 as size_t)
            .wrapping_add(namelen)
            .wrapping_add(1 as size_t);
        let mut fp: *mut ufunc_T = xcalloc(1 as size_t, len) as *mut ufunc_T;
        xmemcpyz(
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            name as *const ::core::ffi::c_void,
            namelen,
        );
        (*fp).uf_namelen = namelen;
        if *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == K_SPECIAL
        {
            len = namelen.wrapping_add(3 as size_t);
            (*fp).uf_name_exp = xmalloc(len) as *mut ::core::ffi::c_char;
            snprintf(
                (*fp).uf_name_exp,
                len,
                b"<SNR>%s\0".as_ptr() as *const ::core::ffi::c_char,
                (&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                    .offset(3 as ::core::ffi::c_int as isize),
            );
        }
        return fp;
    }
}

pub unsafe extern "C" fn get_lambda_tv(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let evaluate: bool =
            !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
        let mut newargs: garray_T = GA_EMPTY_INIT_VALUE;
        let mut pnewargs: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
        let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
        let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
        let mut varargs: ::core::ffi::c_int = 0;
        let mut old_eval_lavars: *mut bool = eval_lavars_used.get();
        let mut eval_lavars: bool = false_0 != 0;
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut s: *mut ::core::ffi::c_char =
            skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        let mut ret: ::core::ffi::c_int = get_function_args(
            &raw mut s,
            '-' as ::core::ffi::c_char,
            ::core::ptr::null_mut::<garray_T>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<garray_T>(),
            true_0 != 0,
        );
        if ret == FAIL || *s as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
            return NOTDONE;
        }
        if evaluate {
            pnewargs = &raw mut newargs;
        } else {
            pnewargs = ::core::ptr::null_mut::<garray_T>();
        }
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        ret = get_function_args(
            arg,
            '-' as ::core::ffi::c_char,
            pnewargs,
            &raw mut varargs,
            ::core::ptr::null_mut::<garray_T>(),
            false_0 != 0,
        );
        if !(ret == FAIL || **arg as ::core::ffi::c_int != '>' as ::core::ffi::c_int) {
            if evaluate {
                eval_lavars_used.set(&raw mut eval_lavars);
            }
            *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
            start = *arg;
            ret = skip_expr(arg, evalarg);
            end = *arg;
            if ret != FAIL {
                if !evalarg.is_null() {
                    tofree = (*evalarg).eval_tofree;
                    (*evalarg).eval_tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                *arg = skipwhite(*arg);
                if **arg as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E451: Expected }: %s\0".as_ptr() as *const ::core::ffi::c_char),
                        *arg,
                    );
                } else {
                    *arg = (*arg).offset(1);
                    if evaluate {
                        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut newlines: garray_T = garray_T {
                            ga_len: 0,
                            ga_maxlen: 0,
                            ga_itemsize: 0,
                            ga_growsize: 0,
                            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        };
                        let mut name: String_0 = get_lambda_name();
                        fp = alloc_ufunc(name.data, name.size);
                        pt = xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>())
                            as *mut partial_T;
                        ga_init(
                            &raw mut newlines,
                            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                        ga_grow(&raw mut newlines, 1 as ::core::ffi::c_int);
                        let mut len: size_t = (end
                            .offset(7 as ::core::ffi::c_int as isize)
                            .offset_from(start)
                            + 1 as isize) as size_t;
                        let mut p: *mut ::core::ffi::c_char =
                            xmalloc(len) as *mut ::core::ffi::c_char;
                        let c2rust_fresh0 = newlines.ga_len;
                        newlines.ga_len = newlines.ga_len + 1;
                        let c2rust_lvalue_ptr = &raw mut *(newlines.ga_data
                            as *mut *mut ::core::ffi::c_char)
                            .offset(c2rust_fresh0 as isize);
                        *c2rust_lvalue_ptr = p;
                        strcpy(
                            p,
                            b"return \0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                        xmemcpyz(
                            p.offset(7 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                            start as *const ::core::ffi::c_void,
                            end.offset_from(start) as size_t,
                        );
                        if strstr(
                            p.offset(7 as ::core::ffi::c_int as isize),
                            b"a:\0".as_ptr() as *const ::core::ffi::c_char,
                        )
                        .is_null()
                        {
                            flags |= FC_NOARGS;
                        }
                        (*fp).uf_refcount = 1 as ::core::ffi::c_int;
                        hash_add(
                            func_hashtab.ptr(),
                            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                        );
                        (*fp).uf_args = newargs;
                        ga_init(
                            &raw mut (*fp).uf_def_args,
                            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                        (*fp).uf_lines = newlines;
                        if !(*current_funccal.ptr()).is_null()
                            && eval_lavars as ::core::ffi::c_int != 0
                        {
                            flags |= FC_CLOSURE;
                            register_closure(fp);
                        } else {
                            (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
                        }
                        if prof_def_func() {
                            func_do_profile(fp);
                        }
                        if sandbox.get() != 0 {
                            flags |= FC_SANDBOX;
                        }
                        (*fp).uf_varargs = true_0;
                        (*fp).uf_flags = flags;
                        (*fp).uf_calls = 0 as ::core::ffi::c_int;
                        (*fp).uf_script_ctx = current_sctx.get();
                        (*fp).uf_script_ctx.sc_lnum =
                            ((*fp).uf_script_ctx.sc_lnum as ::core::ffi::c_int
                                + ((*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                    ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                                ))
                                .es_lnum
                                    - newlines.ga_len as linenr_T)
                                    as ::core::ffi::c_int) as linenr_T;
                        (*pt).pt_func = fp;
                        (*pt).pt_refcount = 1 as ::core::ffi::c_int;
                        (*rettv).vval.v_partial = pt;
                        (*rettv).v_type = VAR_PARTIAL;
                    }
                    eval_lavars_used.set(old_eval_lavars);
                    if !evalarg.is_null() && (*evalarg).eval_tofree.is_null() {
                        (*evalarg).eval_tofree = tofree;
                    } else {
                        xfree(tofree as *mut ::core::ffi::c_void);
                    }
                    return OK;
                }
            }
        }
        ga_clear_strings(&raw mut newargs);
        '_c2rust_label: {
            if fp.is_null() {
            } else {
                __assert_fail(
                    b"fp == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    418 as ::core::ffi::c_uint,
                    b"int get_lambda_tv(char **, typval_T *, evalarg_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        xfree(pt as *mut ::core::ffi::c_void);
        if !evalarg.is_null() && (*evalarg).eval_tofree.is_null() {
            (*evalarg).eval_tofree = tofree;
        } else {
            xfree(tofree as *mut ::core::ffi::c_void);
        }
        eval_lavars_used.set(old_eval_lavars);
        return FAIL;
    }
}

pub unsafe extern "C" fn make_partial(selfdict: *mut dict_T, rettv: *mut typval_T) {
    unsafe {
        let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
        let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
        let mut error: ::core::ffi::c_int = 0;
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*rettv).vval.v_partial.is_null()
            && !(*(*rettv).vval.v_partial).pt_func.is_null()
        {
            fp = (*(*rettv).vval.v_partial).pt_func;
        } else {
            let mut fname: *mut ::core::ffi::c_char = if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*rettv).vval.v_string
            } else if (*rettv).vval.v_partial.is_null() {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                (*(*rettv).vval.v_partial).pt_name
            };
            if fname.is_null() {
                (*rettv).v_type = VAR_FUNC;
                (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                let mut tofree: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                fname = fname_trans_sid(
                    fname,
                    &raw mut fname_buf as *mut ::core::ffi::c_char,
                    &raw mut tofree,
                    &raw mut error,
                );
                fp = find_func(fname);
                xfree(tofree as *mut ::core::ffi::c_void);
            }
        }
        if !fp.is_null() && (*fp).uf_flags & FC_DICT != 0 {
            let mut pt: *mut partial_T =
                xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
            (*pt).pt_refcount = 1 as ::core::ffi::c_int;
            (*pt).pt_dict = selfdict;
            (*selfdict).dv_refcount += 1;
            (*pt).pt_auto = true_0 != 0;
            if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*pt).pt_name = (*rettv).vval.v_string;
            } else {
                let mut ret_pt: *mut partial_T = (*rettv).vval.v_partial;
                if !(*ret_pt).pt_name.is_null() {
                    (*pt).pt_name = xstrdup((*ret_pt).pt_name);
                    func_ref((*pt).pt_name);
                } else {
                    (*pt).pt_func = (*ret_pt).pt_func;
                    func_ptr_ref((*pt).pt_func);
                }
                if (*ret_pt).pt_argc > 0 as ::core::ffi::c_int {
                    let mut arg_size: size_t = ::core::mem::size_of::<typval_T>()
                        .wrapping_mul((*ret_pt).pt_argc as size_t);
                    (*pt).pt_argv = xmalloc(arg_size) as *mut typval_T;
                    (*pt).pt_argc = (*ret_pt).pt_argc;
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < (*pt).pt_argc {
                        tv_copy(
                            (*ret_pt).pt_argv.offset(i as isize),
                            (*pt).pt_argv.offset(i as isize),
                        );
                        i += 1;
                    }
                }
                partial_unref(ret_pt);
            }
            (*rettv).v_type = VAR_PARTIAL;
            (*rettv).vval.v_partial = pt;
        }
    }
}

pub unsafe extern "C" fn register_luafunc(mut ref_0: LuaRef) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut name: String_0 = get_lambda_name();
        let mut fp: *mut ufunc_T = alloc_ufunc(name.data, name.size);
        (*fp).uf_refcount = 1 as ::core::ffi::c_int;
        (*fp).uf_varargs = true_0;
        (*fp).uf_flags = FC_LUAREF;
        (*fp).uf_calls = 0 as ::core::ffi::c_int;
        (*fp).uf_script_ctx = current_sctx.get();
        (*fp).uf_luaref = ref_0;
        hash_add(
            func_hashtab.ptr(),
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
        );
        return &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
    }
}
