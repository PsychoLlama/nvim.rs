//! `:return`, `:call`, `:defer`, and the do_cmdline cookie.
//!
//! `ex_return`/`do_return` implement returning -- including the case where
//! a `:finally` is still to run -- and `get_return_cmd` renders a pending
//! return for the debugger.  `get_func_line` and the small accessors below
//! it are the `do_cmdline` cookie interface a function body is executed
//! through.  `:defer` records a call to make on the way out and
//! `invoke_all_defer` makes them.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct defer_T {
    pub dr_name: *mut ::core::ffi::c_char,
    pub dr_argvars: [typval_T; 21],
    pub dr_argcount: ::core::ffi::c_int,
}

pub unsafe fn ex_return(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut returning: bool = false_0 != 0;
        if (*current_funccal.ptr()).is_null() {
            emsg(gettext(
                b"E133: :return not inside a function\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
        let mut evalarg: evalarg_T = evalarg_T {
            eval_flags: if (*eap).skip != 0 {
                0 as ::core::ffi::c_int
            } else {
                EVAL_EVALUATE as ::core::ffi::c_int
            },
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) += 1;
        }
        (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if *arg as ::core::ffi::c_int != NUL
            && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
            && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            && eval0(arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL
        {
            if (*eap).skip == 0 {
                returning = do_return(
                    eap,
                    false_0 != 0,
                    true_0 != 0,
                    &raw mut rettv as *mut ::core::ffi::c_void,
                );
            } else {
                tv_clear(&raw mut rettv);
            }
        } else if (*eap).skip == 0 {
            update_force_abort();
            if !aborting() {
                returning = do_return(eap, false_0 != 0, true_0 != 0, NULL);
            }
        }
        if returning {
            (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else if (*eap).nextcmd.is_null() {
            (*eap).nextcmd = check_nextcmd(arg);
        }
        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) -= 1;
        }
        clear_evalarg(&raw mut evalarg, eap);
    }
}

unsafe extern "C" fn ex_call_inner(
    mut eap: *mut exarg_T,
    mut name: *mut ::core::ffi::c_char,
    mut arg: *mut *mut ::core::ffi::c_char,
    mut startarg: *mut ::core::ffi::c_char,
    funcexe_init: *const funcexe_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut doesrange: bool = false;
        let mut failed: bool = false_0 != 0;
        let mut lnum: linenr_T = (*eap).line1;
        while lnum <= (*eap).line2 {
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                if lnum > (*curbuf.get()).b_ml.ml_line_count {
                    emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
                    break;
                } else {
                    (*curwin.get()).w_cursor.lnum = lnum;
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                }
            }
            *arg = startarg;
            let mut funcexe: funcexe_T = *funcexe_init;
            funcexe.fe_doesrange = &raw mut doesrange;
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            rettv.v_type = VAR_UNKNOWN;
            if get_func_tv(
                name,
                -1 as ::core::ffi::c_int,
                &raw mut rettv,
                arg,
                evalarg,
                &raw mut funcexe,
            ) == FAIL
            {
                failed = true_0 != 0;
                break;
            } else if handle_subscript(
                arg as *mut *const ::core::ffi::c_char,
                &raw mut rettv,
                EVALARG_EVALUATE.ptr(),
                true_0 != 0,
            ) == FAIL
            {
                failed = true_0 != 0;
                break;
            } else {
                tv_clear(&raw mut rettv);
                if doesrange {
                    break;
                }
                if aborting() {
                    break;
                }
                lnum += 1;
            }
        }
        return failed as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn ex_defer_inner(
    mut name: *mut ::core::ffi::c_char,
    mut arg: *mut *mut ::core::ffi::c_char,
    partial: *const partial_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut argvars: [typval_T; 21] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 21];
        let mut partial_argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*current_funccal.ptr()).is_null() {
            semsg(
                gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
                b"defer\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return FAIL;
        }
        if !partial.is_null() {
            if !(*partial).pt_dict.is_null() {
                emsg(gettext(
                    (e_cannot_use_partial_with_dictionary_for_defer.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
            if (*partial).pt_argc > 0 as ::core::ffi::c_int {
                partial_argc = (*partial).pt_argc;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < partial_argc {
                    tv_copy(
                        (*partial).pt_argv.offset(i as isize),
                        (&raw mut argvars as *mut typval_T).offset(i as isize),
                    );
                    i += 1;
                }
            }
        }
        let mut r: ::core::ffi::c_int = get_func_arguments(
            arg,
            evalarg,
            false_0,
            (&raw mut argvars as *mut typval_T).offset(partial_argc as isize),
            &raw mut argcount,
        );
        argcount += partial_argc;
        if r == OK {
            if builtin_function(name, -1 as ::core::ffi::c_int) {
                let fdef: *const EvalFuncDef = find_internal_func(name);
                if fdef.is_null() {
                    emsg_funcname(
                        &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                        name,
                    );
                    r = FAIL;
                } else if check_internal_func(fdef, argcount) == -1 as ::core::ffi::c_int {
                    r = FAIL;
                }
            } else {
                let mut ufunc: *mut ufunc_T = find_func(name);
                if !ufunc.is_null() {
                    let mut error: ::core::ffi::c_int = check_user_func_argcount(ufunc, argcount);
                    if error != FCERR_UNKNOWN as ::core::ffi::c_int {
                        user_func_error(error, name, false_0 != 0);
                        r = FAIL;
                    }
                }
            }
        }
        if r == FAIL {
            loop {
                argcount -= 1;
                if argcount < 0 as ::core::ffi::c_int {
                    break;
                }
                tv_clear((&raw mut argvars as *mut typval_T).offset(argcount as isize));
            }
            return FAIL;
        }
        add_defer(name, argcount, &raw mut argvars as *mut typval_T);
        return OK;
    }
}

pub unsafe extern "C" fn can_add_defer() -> bool {
    unsafe {
        if get_current_funccal().is_null() {
            semsg(
                gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
                b"defer\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn add_defer(
    mut name: *mut ::core::ffi::c_char,
    mut argcount_arg: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
) {
    unsafe {
        let mut saved_name: *mut ::core::ffi::c_char = xstrdup(name);
        let mut argcount: ::core::ffi::c_int = argcount_arg;
        if (*current_funccal.get()).fc_defer.ga_itemsize == 0 as ::core::ffi::c_int {
            ga_init(
                &raw mut (*current_funccal.get()).fc_defer,
                ::core::mem::size_of::<defer_T>() as ::core::ffi::c_int,
                10 as ::core::ffi::c_int,
            );
        }
        let mut dr: *mut defer_T = ga_append_via_ptr(
            &raw mut (*current_funccal.get()).fc_defer,
            ::core::mem::size_of::<defer_T>(),
        ) as *mut defer_T;
        (*dr).dr_name = saved_name;
        (*dr).dr_argcount = argcount;
        while argcount > 0 as ::core::ffi::c_int {
            argcount -= 1;
            (*dr).dr_argvars[argcount as usize] = *argvars.offset(argcount as isize);
        }
    }
}

pub(crate) unsafe extern "C" fn handle_defer_one(mut funccal: *mut funccall_T) {
    unsafe {
        let mut idx: ::core::ffi::c_int = (*funccal).fc_defer.ga_len - 1 as ::core::ffi::c_int;
        while idx >= 0 as ::core::ffi::c_int {
            let mut dr: *mut defer_T =
                ((*funccal).fc_defer.ga_data as *mut defer_T).offset(idx as isize);
            if !(*dr).dr_name.is_null() {
                let mut funcexe: funcexe_T = funcexe_T {
                    fe_argv_func: None,
                    fe_firstline: 0,
                    fe_lastline: 0,
                    fe_doesrange: ::core::ptr::null_mut::<bool>(),
                    fe_evaluate: true_0 != 0,
                    fe_partial: ::core::ptr::null_mut::<partial_T>(),
                    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
                    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
                    fe_found_var: false,
                };
                let mut rettv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                rettv.v_type = VAR_UNKNOWN;
                let mut name: *mut ::core::ffi::c_char = (*dr).dr_name;
                (*dr).dr_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut estate: exception_state_T = exception_state_T {
                    estate_current_exception: ::core::ptr::null_mut::<except_T>(),
                    estate_did_throw: false,
                    estate_need_rethrow: false,
                    estate_trylevel: 0,
                    estate_did_emsg: 0,
                };
                exception_state_save(&raw mut estate);
                exception_state_clear();
                call_func(
                    name,
                    -1 as ::core::ffi::c_int,
                    &raw mut rettv,
                    (*dr).dr_argcount,
                    &raw mut (*dr).dr_argvars as *mut typval_T,
                    &raw mut funcexe,
                );
                exception_state_restore(&raw mut estate);
                tv_clear(&raw mut rettv);
                xfree(name as *mut ::core::ffi::c_void);
                let mut i: ::core::ffi::c_int = (*dr).dr_argcount - 1 as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    tv_clear((&raw mut (*dr).dr_argvars as *mut typval_T).offset(i as isize));
                    i -= 1;
                }
            }
            idx -= 1;
        }
        ga_clear(&raw mut (*funccal).fc_defer);
    }
}

pub unsafe extern "C" fn invoke_all_defer() {
    unsafe {
        let mut fc: *mut funccall_T = current_funccal.get();
        while !fc.is_null() {
            handle_defer_one(fc);
            fc = (*fc).fc_caller;
        }
        let mut fce: *mut funccal_entry_T = funccal_stack.get();
        while !fce.is_null() {
            let mut fc_0: *mut funccall_T = (*fce).top_funccal as *mut funccall_T;
            while !fc_0.is_null() {
                handle_defer_one(fc_0);
                fc_0 = (*fc_0).fc_caller;
            }
            fce = (*fce).next;
        }
    }
}

pub unsafe fn ex_call(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut failed: bool = false_0 != 0;
        let mut fudi: funcdict_T = funcdict_T {
            fd_dict: ::core::ptr::null_mut::<dict_T>(),
            fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fd_di: ::core::ptr::null_mut::<dictitem_T>(),
        };
        let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
        let mut evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
        if (*eap).skip != 0 {
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            (*emsg_skip.ptr()) += 1;
            if eval0((*eap).arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL {
                tv_clear(&raw mut rettv);
            }
            (*emsg_skip.ptr()) -= 1;
            clear_evalarg(&raw mut evalarg, eap);
            return;
        }
        let mut tofree: *mut ::core::ffi::c_char = trans_function_name(
            &raw mut arg,
            false_0 != 0,
            TFN_INT as ::core::ffi::c_int,
            &raw mut fudi,
            &raw mut partial,
        );
        if !fudi.fd_newkey.is_null() {
            semsg(
                gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                fudi.fd_newkey,
            );
            xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
        }
        if tofree.is_null() {
            return;
        }
        if !fudi.fd_dict.is_null() {
            (*fudi.fd_dict).dv_refcount += 1;
        }
        let mut len: ::core::ffi::c_int = strlen(tofree) as ::core::ffi::c_int;
        let mut found_var: bool = false_0 != 0;
        let mut name: *mut ::core::ffi::c_char = deref_func_name(
            tofree,
            &raw mut len,
            if !partial.is_null() {
                ::core::ptr::null_mut::<*mut partial_T>()
            } else {
                &raw mut partial
            },
            false_0 != 0,
            &raw mut found_var,
        );
        let mut startarg: *mut ::core::ffi::c_char = skipwhite(arg);
        if *startarg as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_missingparen as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        } else {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_defer as ::core::ffi::c_int {
                arg = startarg;
                failed = ex_defer_inner(name, &raw mut arg, partial, &raw mut evalarg) == FAIL;
            } else {
                let mut funcexe: funcexe_T = FUNCEXE_INIT;
                funcexe.fe_partial = partial;
                funcexe.fe_selfdict = fudi.fd_dict;
                funcexe.fe_firstline = (*eap).line1;
                funcexe.fe_lastline = (*eap).line2;
                funcexe.fe_found_var = found_var;
                funcexe.fe_evaluate = true_0 != 0;
                failed = ex_call_inner(
                    eap,
                    name,
                    &raw mut arg,
                    startarg,
                    &raw mut funcexe,
                    &raw mut evalarg,
                ) != 0;
            }
            if (!aborting() || did_throw.get() as ::core::ffi::c_int != 0)
                && (!failed || (*(*eap).cstack).cs_trylevel > 0 as ::core::ffi::c_int)
            {
                if ends_excmd(*arg as ::core::ffi::c_int) == 0 {
                    if !failed && !aborting() {
                        emsg_severe.set(true_0 != 0);
                        semsg(
                            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                            arg,
                        );
                    }
                } else {
                    (*eap).nextcmd = check_nextcmd(arg);
                }
            }
            clear_evalarg(&raw mut evalarg, eap);
        }
        tv_dict_unref(fudi.fd_dict);
        xfree(tofree as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn do_return(
    mut eap: *mut exarg_T,
    mut reanimate: bool,
    mut is_cmd: bool,
    mut rettv: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        let cstack: *mut cstack_T = (*eap).cstack;
        if reanimate {
            (*current_funccal.get()).fc_returned = false_0;
        }
        let mut idx: ::core::ffi::c_int =
            cleanup_conditionals((*eap).cstack, 0 as ::core::ffi::c_int, true_0);
        if idx >= 0 as ::core::ffi::c_int {
            (*cstack).cs_pending[idx as usize] =
                CSTP_RETURN as ::core::ffi::c_int as ::core::ffi::c_char;
            if !is_cmd && !reanimate {
                (*cstack).cs_pend.csp_rv[idx as usize] = rettv;
            } else {
                if reanimate {
                    '_c2rust_label: {
                        if !(*current_funccal.get()).fc_rettv.is_null() {
                        } else {
                            __assert_fail(
                                b"current_funccal->fc_rettv\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/userfunc.rs\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                3664 as ::core::ffi::c_uint,
                                b"_Bool do_return(exarg_T *, _Bool, _Bool, void *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    rettv = (*current_funccal.get()).fc_rettv as *mut ::core::ffi::c_void;
                }
                if !rettv.is_null() {
                    (*cstack).cs_pend.csp_rv[idx as usize] =
                        xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>());
                    *((*cstack).cs_pend.csp_rv[idx as usize] as *mut typval_T) =
                        *(rettv as *mut typval_T);
                } else {
                    (*cstack).cs_pend.csp_rv[idx as usize] = NULL;
                }
                if reanimate {
                    (*(*current_funccal.get()).fc_rettv).v_type = VAR_NUMBER;
                    (*(*current_funccal.get()).fc_rettv).vval.v_number = 0 as varnumber_T;
                }
            }
            report_make_pending(CSTP_RETURN as ::core::ffi::c_int, rettv);
        } else {
            (*current_funccal.get()).fc_returned = true_0;
            if !reanimate && !rettv.is_null() {
                tv_clear((*current_funccal.get()).fc_rettv);
                *(*current_funccal.get()).fc_rettv = *(rettv as *mut typval_T);
                if !is_cmd {
                    xfree(rettv);
                }
            }
        }
        return idx < 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn get_return_cmd(
    mut rettv: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut slen: size_t = 0 as size_t;
        if !rettv.is_null() {
            s = encode_tv2echo(rettv as *mut typval_T, ::core::ptr::null_mut::<size_t>());
            tofree = s;
        }
        if s.is_null() {
            s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            slen = strlen(s);
        }
        xstrlcpy(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            b":return \0".as_ptr() as *const ::core::ffi::c_char,
            IOSIZE as size_t,
        );
        xstrlcpy(
            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(8 as ::core::ffi::c_int as isize),
            s,
            (IOSIZE - 8 as ::core::ffi::c_int) as size_t,
        );
        let mut IObufflen: size_t = (8 as size_t).wrapping_add(slen);
        if IObufflen >= IOSIZE as size_t {
            strcpy(
                (IObuff.ptr() as *mut ::core::ffi::c_char)
                    .offset((1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b"...\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            IObufflen = (IOSIZE - 1 as ::core::ffi::c_int) as size_t;
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        return xstrnsave(IObuff.ptr() as *mut ::core::ffi::c_char, IObufflen);
    }
}

pub unsafe extern "C" fn get_func_line(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
        let mut fp: *mut ufunc_T = (*fcp).fc_func;
        let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*fcp).fc_dbg_tick != debug_tick.get() {
            (*fcp).fc_breakpoint = dbg_find_breakpoint(
                false_0 != 0,
                &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
            (*fcp).fc_dbg_tick = debug_tick.get();
        }
        if do_profiling.get() == PROF_YES {
            func_line_end(cookie);
        }
        let mut gap: *mut garray_T = &raw mut (*fp).uf_lines;
        if (*fp).uf_flags & FC_ABORT != 0 && did_emsg.get() != 0 && !aborted_in_try()
            || (*fcp).fc_returned != 0
        {
            retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            while (*fcp).fc_linenr < (*gap).ga_len
                && (*((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((*fcp).fc_linenr as isize))
                .is_null()
            {
                (*fcp).fc_linenr += 1;
            }
            if (*fcp).fc_linenr >= (*gap).ga_len {
                retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                let c2rust_fresh10 = (*fcp).fc_linenr;
                (*fcp).fc_linenr = (*fcp).fc_linenr + 1;
                retval = xstrdup(
                    *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh10 as isize),
                );
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum = (*fcp).fc_linenr as linenr_T;
                if do_profiling.get() == PROF_YES {
                    func_line_start(cookie);
                }
            }
        }
        if (*fcp).fc_breakpoint != 0 as linenr_T
            && (*fcp).fc_breakpoint
                <= (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
        {
            dbg_breakpoint(
                &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
            (*fcp).fc_breakpoint = dbg_find_breakpoint(
                false_0 != 0,
                &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
            (*fcp).fc_dbg_tick = debug_tick.get();
        }
        return retval;
    }
}

pub unsafe extern "C" fn func_has_ended(
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
        return ((*(*fcp).fc_func).uf_flags & FC_ABORT != 0
            && did_emsg.get() != 0
            && !aborted_in_try()
            || (*fcp).fc_returned != 0) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn func_has_abort(
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        return (*(*(cookie as *mut funccall_T)).fc_func).uf_flags & FC_ABORT;
    }
}

pub unsafe extern "C" fn func_name(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return &raw mut (*(*(cookie as *mut funccall_T)).fc_func).uf_name
            as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn func_breakpoint(mut cookie: *mut ::core::ffi::c_void) -> *mut linenr_T {
    unsafe {
        return &raw mut (*(cookie as *mut funccall_T)).fc_breakpoint;
    }
}

pub unsafe extern "C" fn func_dbg_tick(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_int {
    unsafe {
        return &raw mut (*(cookie as *mut funccall_T)).fc_dbg_tick;
    }
}

pub unsafe extern "C" fn func_level(mut cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    unsafe {
        return (*(cookie as *mut funccall_T)).fc_level;
    }
}

pub unsafe extern "C" fn current_func_returned() -> ::core::ffi::c_int {
    unsafe {
        return (*current_funccal.get()).fc_returned;
    }
}
