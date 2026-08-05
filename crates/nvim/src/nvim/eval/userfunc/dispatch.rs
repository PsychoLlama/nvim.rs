//! Choosing what a call *is*, before any of it happens.
//!
//! `call_func` is the one entry point every caller of anything callable
//! reaches: a partial, a `v:lua` reference, a user function, an autoloaded
//! one, or a builtin.  `get_func_tv` is the expression-level wrapper that
//! parses the argument list first, and `func_call` the one that takes the
//! arguments already built as a list.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_func_tv(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    mut funcexe: *mut funcexe_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut argvars: [typval_T; 21] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 21];
        let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let evaluate: bool = if evalarg.is_null() {
            false_0
        } else {
            (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int
        } != 0;
        let mut argp: *mut ::core::ffi::c_char = *arg;
        let mut ret: ::core::ffi::c_int = get_func_arguments(
            &raw mut argp,
            evalarg,
            if (*funcexe).fe_partial.is_null() {
                0 as ::core::ffi::c_int
            } else {
                (*(*funcexe).fe_partial).pt_argc
            },
            &raw mut argvars as *mut typval_T,
            &raw mut argcount,
        );
        '_c2rust_label: {
            if ret == 1 as ::core::ffi::c_int || ret == 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                b"ret == OK || ret == FAIL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/userfunc.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                565 as ::core::ffi::c_uint,
                b"int get_func_tv(const char *, int, typval_T *, char **, evalarg_T *const, funcexe_T *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        if ret == OK {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if get_vim_var_nr(VV_TESTING) != 0 {
                if (*funcargs.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
                    ga_init(
                        funcargs.ptr(),
                        ::core::mem::size_of::<*mut typval_T>() as ::core::ffi::c_int,
                        50 as ::core::ffi::c_int,
                    );
                }
                i = 0 as ::core::ffi::c_int;
                while i < argcount {
                    ga_grow(funcargs.ptr(), 1 as ::core::ffi::c_int);
                    let c2rust_fresh2 = (*funcargs.ptr()).ga_len;
                    (*funcargs.ptr()).ga_len = (*funcargs.ptr()).ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *((*funcargs.ptr()).ga_data
                        as *mut *mut typval_T)
                        .offset(c2rust_fresh2 as isize);
                    *c2rust_lvalue_ptr = (&raw mut argvars as *mut typval_T).offset(i as isize);
                    i += 1;
                }
            }
            ret = call_func(
                name,
                len,
                rettv,
                argcount,
                &raw mut argvars as *mut typval_T,
                funcexe,
            );
            (*funcargs.ptr()).ga_len -= i;
        } else if !aborting() && evaluate as ::core::ffi::c_int != 0 {
            if argcount == MAX_FUNC_ARGS as ::core::ffi::c_int {
                emsg_funcname(
                    b"E740: Too many arguments for function %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    name,
                );
            } else {
                emsg_funcname(
                    b"E116: Invalid arguments for function %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    name,
                );
            }
        }
        loop {
            argcount -= 1;
            if argcount < 0 as ::core::ffi::c_int {
                break;
            }
            tv_clear((&raw mut argvars as *mut typval_T).offset(argcount as isize));
        }
        *arg = skipwhite(argp);
        return ret;
    }
}

pub unsafe extern "C" fn func_call(
    mut name: *mut ::core::ffi::c_char,
    mut args: *mut typval_T,
    mut partial: *mut partial_T,
    mut selfdict: *mut dict_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
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
        let mut argv: [typval_T; 21] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 21];
        let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *mut list_T = (*args).vval.v_list;
        '_func_call_skip_call: {
            's_51: {
                if !l_.is_null() {
                    let mut item: *mut listitem_T = (*l_).lv_first;
                    loop {
                        if item.is_null() {
                            break 's_51;
                        }
                        if argc
                            == MAX_FUNC_ARGS as ::core::ffi::c_int
                                - (if partial.is_null() {
                                    0 as ::core::ffi::c_int
                                } else {
                                    (*partial).pt_argc
                                })
                        {
                            emsg(gettext(b"E699: Too many arguments\0".as_ptr()
                                as *const ::core::ffi::c_char));
                            break '_func_call_skip_call;
                        } else {
                            let c2rust_fresh11 = argc;
                            argc = argc + 1;
                            tv_copy(
                                &raw mut (*item).li_tv,
                                (&raw mut argv as *mut typval_T).offset(c2rust_fresh11 as isize),
                            );
                            item = (*item).li_next;
                        }
                    }
                }
            }
            funcexe = FUNCEXE_INIT;
            funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_evaluate = true_0 != 0;
            funcexe.fe_partial = partial;
            funcexe.fe_selfdict = selfdict;
            r = call_func(
                name,
                -1 as ::core::ffi::c_int,
                rettv,
                argc,
                &raw mut argv as *mut typval_T,
                &raw mut funcexe,
            );
        }
        while argc > 0 as ::core::ffi::c_int {
            argc -= 1;
            tv_clear((&raw mut argv as *mut typval_T).offset(argc as isize));
        }
        return r;
    }
}

pub unsafe extern "C" fn callback_call_retnr(
    mut callback: *mut Callback,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
) -> varnumber_T {
    unsafe {
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if !callback_call(callback, argcount, argvars, &raw mut rettv) {
            return -2 as varnumber_T;
        }
        let mut retval: varnumber_T =
            tv_get_number_chk(&raw mut rettv, ::core::ptr::null_mut::<bool>());
        tv_clear(&raw mut rettv);
        return retval;
    }
}

pub unsafe extern "C" fn call_func(
    mut funcname: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut argcount_in: ::core::ffi::c_int,
    mut argvars_in: *mut typval_T,
    mut funcexe: *mut funcexe_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ret: ::core::ffi::c_int = FAIL;
        let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
        let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
        let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut argcount: ::core::ffi::c_int = argcount_in;
        let mut argvars: *mut typval_T = argvars_in;
        let mut selfdict: *mut dict_T = (*funcexe).fe_selfdict;
        let mut argv: [typval_T; 21] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 21];
        let mut argv_clear: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut argv_base: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut partial: *mut partial_T = (*funcexe).fe_partial;
        (*rettv).v_type = VAR_UNKNOWN;
        if len <= 0 as ::core::ffi::c_int {
            len = strlen(funcname) as ::core::ffi::c_int;
        }
        if !partial.is_null() {
            fp = (*partial).pt_func;
        }
        if fp.is_null() {
            name = xmemdupz(funcname as *const ::core::ffi::c_void, len as size_t)
                as *mut ::core::ffi::c_char;
            fname = fname_trans_sid(
                name,
                &raw mut fname_buf as *mut ::core::ffi::c_char,
                &raw mut tofree,
                &raw mut error,
            );
        }
        if !(*funcexe).fe_doesrange.is_null() {
            *(*funcexe).fe_doesrange = false_0 != 0;
        }
        '_theend: {
            if !partial.is_null() {
                if !(*partial).pt_dict.is_null() && (selfdict.is_null() || !(*partial).pt_auto) {
                    selfdict = (*partial).pt_dict;
                }
                if error == FCERR_NONE as ::core::ffi::c_int
                    && (*partial).pt_argc > 0 as ::core::ffi::c_int
                {
                    argv_clear = 0 as ::core::ffi::c_int;
                    while argv_clear < (*partial).pt_argc {
                        if argv_clear + argcount_in >= MAX_FUNC_ARGS as ::core::ffi::c_int {
                            error = FCERR_TOOMANY as ::core::ffi::c_int;
                            break '_theend;
                        } else {
                            tv_copy(
                                (*partial).pt_argv.offset(argv_clear as isize),
                                (&raw mut argv as *mut typval_T).offset(argv_clear as isize),
                            );
                            argv_clear += 1;
                        }
                    }
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < argcount_in {
                        argv[(i + argv_clear) as usize] = *argvars_in.offset(i as isize);
                        i += 1;
                    }
                    argvars = &raw mut argv as *mut typval_T;
                    argcount = (*partial).pt_argc + argcount_in;
                }
            }
            if error == FCERR_NONE as ::core::ffi::c_int
                && (*funcexe).fe_evaluate as ::core::ffi::c_int != 0
            {
                let mut is_global: bool = fp.is_null()
                    && *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'g' as ::core::ffi::c_int
                    && *fname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int;
                let mut rfname: *mut ::core::ffi::c_char = if is_global as ::core::ffi::c_int != 0 {
                    fname.offset(2 as ::core::ffi::c_int as isize)
                } else {
                    fname
                };
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = 0 as varnumber_T;
                error = FCERR_UNKNOWN as ::core::ffi::c_int;
                if is_luafunc(partial) {
                    if len > 0 as ::core::ffi::c_int {
                        error = FCERR_NONE as ::core::ffi::c_int;
                        argv_add_base(
                            (*funcexe).fe_basetv,
                            &raw mut argvars,
                            &raw mut argcount,
                            &raw mut argv as *mut typval_T,
                            &raw mut argv_base,
                        );
                        nlua_typval_call(funcname, len as size_t, argvars, argcount, rettv);
                    } else {
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut name as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL;
                        let _ = *ptr_;
                        funcname = b"v:lua\0".as_ptr() as *const ::core::ffi::c_char;
                    }
                } else if !fp.is_null() || !builtin_function(rfname, -1 as ::core::ffi::c_int) {
                    if fp.is_null() {
                        fp = find_func(rfname);
                    }
                    if fp.is_null()
                        && apply_autocmds(
                            EVENT_FUNCUNDEFINED,
                            rfname,
                            rfname,
                            true_0 != 0,
                            ::core::ptr::null_mut::<buf_T>(),
                        ) as ::core::ffi::c_int
                            != 0
                        && !aborting()
                    {
                        fp = find_func(rfname);
                    }
                    if fp.is_null()
                        && script_autoload(rfname, strlen(rfname), true_0 != 0)
                            as ::core::ffi::c_int
                            != 0
                        && !aborting()
                    {
                        fp = find_func(rfname);
                    }
                    if !fp.is_null() && (*fp).uf_flags & FC_DELETED != 0 {
                        error = FCERR_DELETED as ::core::ffi::c_int;
                    } else if !fp.is_null() {
                        if (*funcexe).fe_argv_func.is_some() {
                            argcount = (*funcexe).fe_argv_func.expect("non-null function pointer")(
                                argcount, argvars, argv_clear, fp,
                            );
                        }
                        argv_add_base(
                            (*funcexe).fe_basetv,
                            &raw mut argvars,
                            &raw mut argcount,
                            &raw mut argv as *mut typval_T,
                            &raw mut argv_base,
                        );
                        error =
                            call_user_func_check(fp, argcount, argvars, rettv, funcexe, selfdict);
                    }
                } else if !(*funcexe).fe_basetv.is_null() {
                    error =
                        call_internal_method(fname, argcount, argvars, rettv, (*funcexe).fe_basetv);
                } else {
                    error = call_internal_func(fname, argcount, argvars, rettv);
                }
                update_force_abort();
            }
            if error == FCERR_NONE as ::core::ffi::c_int {
                ret = OK;
            }
        }
        if !aborting() {
            user_func_error(
                error,
                if !name.is_null() {
                    name as *const ::core::ffi::c_char
                } else {
                    funcname
                },
                (*funcexe).fe_found_var,
            );
        }
        while argv_clear > 0 as ::core::ffi::c_int {
            argv_clear -= 1;
            tv_clear((&raw mut argv as *mut typval_T).offset((argv_clear + argv_base) as isize));
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        xfree(name as *mut ::core::ffi::c_void);
        return ret;
    }
}
