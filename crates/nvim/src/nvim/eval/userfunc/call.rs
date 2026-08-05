//! Calling a user function: the funccall_T's whole life.
//!
//! `call_user_func` builds the `a:` and `l:` scopes in the funccall's
//! embedded storage, evaluates the default arguments in order, runs the
//! body through `do_cmdline` and tears the scopes down again.
//! `call_user_func_check` is the guard in front of it ('maxfuncdepth',
//! the `dict` attribute, deleted functions) and `user_func_error` turns an
//! `FCERR_*` code into the message the user sees.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn call_user_func(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut firstline: linenr_T,
    mut lastline: linenr_T,
    mut selfdict: *mut dict_T,
) {
    unsafe {
        let mut using_sandbox: bool = false_0 != 0;
        static depth: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut fixvar_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut islambda: bool = false_0 != 0;
        let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut namelen: size_t = 0;
        let mut tv_to_free: [*mut typval_T; 20] = [::core::ptr::null_mut::<typval_T>(); 20];
        let mut tv_to_free_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wait_start: proftime_T = 0;
        let mut call_start: proftime_T = 0;
        let mut started_profiling: bool = false_0 != 0;
        let mut did_save_redo: bool = false_0 != 0;
        let mut save_redo: save_redo_T = save_redo_T {
            sr_redobuff: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            sr_old_redobuff: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
        };
        if depth.get() as OptInt >= p_mfd.get() {
            emsg(gettext(
                b"E132: Function call depth is higher than 'maxfuncdepth'\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = -1 as varnumber_T;
            return;
        }
        (*depth.ptr()) += 1;
        save_search_patterns();
        if !ins_compl_active() {
            saveRedobuff(&raw mut save_redo);
            did_save_redo = true_0 != 0;
        }
        (*fp).uf_calls += 1;
        line_breakcheck();
        let mut fc: *mut funccall_T = create_funccal(fp, rettv);
        (*fc).fc_level = ex_nesting_level.get();
        (*fc).fc_breakpoint = dbg_find_breakpoint(
            false_0 != 0,
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            0 as linenr_T,
        );
        (*fc).fc_dbg_tick = debug_tick.get();
        ga_init(
            &raw mut (*fc).fc_ufuncs,
            ::core::mem::size_of::<*mut ufunc_T>() as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        if strncmp(
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            islambda = true_0 != 0;
        }
        init_var_dict(
            &raw mut (*fc).fc_l_vars,
            &raw mut (*fc).fc_l_vars_var,
            VAR_DEF_SCOPE,
        );
        if !selfdict.is_null() {
            let c2rust_fresh3 = fixvar_idx;
            fixvar_idx = fixvar_idx + 1;
            v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh3 as isize)
                as *mut dictitem_T;
            name = &raw mut (*v).di_key as *mut ::core::ffi::c_char;
            strcpy(
                name,
                b"self\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            (*v).di_flags =
                (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
            hash_add(
                &raw mut (*fc).fc_l_vars.dv_hashtab,
                &raw mut (*v).di_key as *mut ::core::ffi::c_char,
            );
            (*v).di_tv.v_type = VAR_DICT;
            (*v).di_tv.v_lock = VAR_UNLOCKED;
            (*v).di_tv.vval.v_dict = selfdict;
            (*selfdict).dv_refcount += 1;
        }
        init_var_dict(
            &raw mut (*fc).fc_l_avars,
            &raw mut (*fc).fc_l_avars_var,
            VAR_SCOPE,
        );
        if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
            let c2rust_fresh4 = fixvar_idx;
            fixvar_idx = fixvar_idx + 1;
            add_nr_var(
                &raw mut (*fc).fc_l_avars,
                (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh4 as isize)
                    as *mut dictitem_T,
                b"0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                (if argcount >= (*fp).uf_args.ga_len {
                    argcount - (*fp).uf_args.ga_len
                } else {
                    0 as ::core::ffi::c_int
                }) as varnumber_T,
            );
        }
        (*fc).fc_l_avars.dv_lock = VAR_FIXED;
        if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
            let c2rust_fresh5 = fixvar_idx;
            fixvar_idx = fixvar_idx + 1;
            v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh5 as isize)
                as *mut dictitem_T;
            name = &raw mut (*v).di_key as *mut ::core::ffi::c_char;
            strcpy(
                name,
                b"000\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            (*v).di_flags =
                (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
            hash_add(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                &raw mut (*v).di_key as *mut ::core::ffi::c_char,
            );
            (*v).di_tv.v_type = VAR_LIST;
            (*v).di_tv.v_lock = VAR_FIXED;
            (*v).di_tv.vval.v_list = &raw mut (*fc).fc_l_varlist;
        }
        tv_list_init_static(&raw mut (*fc).fc_l_varlist);
        tv_list_set_lock(&raw mut (*fc).fc_l_varlist, VAR_FIXED);
        if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
            let c2rust_fresh6 = fixvar_idx;
            fixvar_idx = fixvar_idx + 1;
            add_nr_var(
                &raw mut (*fc).fc_l_avars,
                (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh6 as isize)
                    as *mut dictitem_T,
                b"firstline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                firstline as varnumber_T,
            );
            let c2rust_fresh7 = fixvar_idx;
            fixvar_idx = fixvar_idx + 1;
            add_nr_var(
                &raw mut (*fc).fc_l_avars,
                (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh7 as isize)
                    as *mut dictitem_T,
                b"lastline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                lastline as varnumber_T,
            );
        }
        let mut default_arg_err: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < argcount || i < (*fp).uf_args.ga_len {
            let mut addlocal: bool = false_0 != 0;
            let mut isdefault: bool = false_0 != 0;
            let mut def_rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut ai: ::core::ffi::c_int = i - (*fp).uf_args.ga_len;
            if ai < 0 as ::core::ffi::c_int {
                name = *((*fp).uf_args.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
                if islambda {
                    addlocal = true_0 != 0;
                }
                isdefault =
                    ai + (*fp).uf_def_args.ga_len >= 0 as ::core::ffi::c_int && i >= argcount;
                if isdefault {
                    let mut default_expr: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    def_rettv.v_type = VAR_NUMBER;
                    def_rettv.vval.v_number = -1 as varnumber_T;
                    default_expr = *((*fp).uf_def_args.ga_data as *mut *mut ::core::ffi::c_char)
                        .offset((ai + (*fp).uf_def_args.ga_len) as isize);
                    if eval1(
                        &raw mut default_expr,
                        &raw mut def_rettv,
                        EVALARG_EVALUATE.ptr(),
                    ) == FAIL
                    {
                        default_arg_err = true_0 != 0;
                        break;
                    }
                }
                namelen = strlen(name);
            } else {
                if (*fp).uf_flags & FC_NOARGS != 0 as ::core::ffi::c_int {
                    break;
                }
                namelen = snprintf(
                    &raw mut numbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    ai + 1 as ::core::ffi::c_int,
                ) as size_t;
                name = &raw mut numbuf as *mut ::core::ffi::c_char;
            }
            if fixvar_idx < FIXVAR_CNT as ::core::ffi::c_int
                && namelen <= VAR_SHORT_LEN as ::core::ffi::c_int as size_t
            {
                let c2rust_fresh8 = fixvar_idx;
                fixvar_idx = fixvar_idx + 1;
                v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7)
                    .offset(c2rust_fresh8 as isize) as *mut dictitem_T;
                (*v).di_flags = (DI_FLAGS_RO as ::core::ffi::c_int
                    | DI_FLAGS_FIX as ::core::ffi::c_int)
                    as uint8_t;
                strcpy(&raw mut (*v).di_key as *mut ::core::ffi::c_char, name);
            } else {
                v = tv_dict_item_alloc_len(name, namelen);
                (*v).di_flags = ((*v).di_flags as ::core::ffi::c_int
                    | (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int))
                    as uint8_t;
            }
            (*v).di_tv = if isdefault as ::core::ffi::c_int != 0 {
                def_rettv
            } else {
                *argvars.offset(i as isize)
            };
            (*v).di_tv.v_lock = VAR_FIXED;
            if isdefault {
                let c2rust_fresh9 = tv_to_free_len;
                tv_to_free_len = tv_to_free_len + 1;
                let c2rust_lvalue_ptr = &raw mut tv_to_free[c2rust_fresh9 as usize];
                *c2rust_lvalue_ptr = &raw mut (*v).di_tv;
            }
            if addlocal {
                tv_copy(&raw mut (*v).di_tv, &raw mut (*v).di_tv);
                hash_add(
                    &raw mut (*fc).fc_l_vars.dv_hashtab,
                    &raw mut (*v).di_key as *mut ::core::ffi::c_char,
                );
            } else {
                hash_add(
                    &raw mut (*fc).fc_l_avars.dv_hashtab,
                    &raw mut (*v).di_key as *mut ::core::ffi::c_char,
                );
            }
            if ai >= 0 as ::core::ffi::c_int && ai < MAX_FUNC_ARGS as ::core::ffi::c_int {
                let mut li: *mut listitem_T =
                    (&raw mut (*fc).fc_l_listitems as *mut listitem_T).offset(ai as isize);
                (*li).li_tv = *argvars.offset(i as isize);
                (*li).li_tv.v_lock = VAR_FIXED;
                tv_list_append(&raw mut (*fc).fc_l_varlist, li);
            }
            i += 1;
        }
        (*RedrawingDisabled.ptr()) += 1;
        if (*fp).uf_flags & FC_SANDBOX != 0 {
            using_sandbox = true_0 != 0;
            (*sandbox.ptr()) += 1;
        }
        estack_push_ufunc(fp, 1 as linenr_T);
        if p_verbose.get() >= 12 as OptInt {
            (*no_wait_return.ptr()) += 1;
            verbose_enter_scroll();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"calling %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
            );
            if p_verbose.get() >= 14 as OptInt {
                msg_puts(b"(\0".as_ptr() as *const ::core::ffi::c_char);
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < argcount {
                    if i_0 > 0 as ::core::ffi::c_int {
                        msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
                    }
                    if (*argvars.offset(i_0 as isize)).v_type as ::core::ffi::c_uint
                        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        msg_outnum(
                            (*argvars.offset(i_0 as isize)).vval.v_number as ::core::ffi::c_int,
                        );
                    } else {
                        (*emsg_off.ptr()) += 1;
                        let mut tofree: *mut ::core::ffi::c_char = encode_tv2string(
                            argvars.offset(i_0 as isize),
                            ::core::ptr::null_mut::<size_t>(),
                        );
                        (*emsg_off.ptr()) -= 1;
                        if !tofree.is_null() {
                            let mut s: *mut ::core::ffi::c_char = tofree;
                            let mut buf: [::core::ffi::c_char; 480] = [0; 480];
                            if vim_strsize(s) > MSG_BUF_CLEN {
                                trunc_string(
                                    s,
                                    &raw mut buf as *mut ::core::ffi::c_char,
                                    MSG_BUF_CLEN,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 480]>()
                                        as ::core::ffi::c_int,
                                );
                                s = &raw mut buf as *mut ::core::ffi::c_char;
                            }
                            msg_puts(s);
                            xfree(tofree as *mut ::core::ffi::c_void);
                        }
                    }
                    i_0 += 1;
                }
                msg_puts(b")\0".as_ptr() as *const ::core::ffi::c_char);
            }
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            verbose_leave_scroll();
            (*no_wait_return.ptr()) -= 1;
        }
        let do_profiling_yes: bool = do_profiling.get() == PROF_YES;
        let mut func_not_yet_profiling_but_should: bool = do_profiling_yes as ::core::ffi::c_int
            != 0
            && (*fp).uf_profiling == 0
            && has_profiling(
                false_0 != 0,
                &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<bool>(),
            ) as ::core::ffi::c_int
                != 0;
        if func_not_yet_profiling_but_should {
            started_profiling = true_0 != 0;
            func_do_profile(fp);
        }
        let mut func_or_func_caller_profiling: bool = do_profiling_yes as ::core::ffi::c_int != 0
            && ((*fp).uf_profiling != 0
                || !(*fc).fc_caller.is_null() && (*(*(*fc).fc_caller).fc_func).uf_profiling != 0);
        if func_or_func_caller_profiling {
            (*fp).uf_tm_count += 1;
            call_start = profile_start();
            (*fp).uf_tm_children = profile_zero();
        }
        if do_profiling_yes {
            wait_start = script_prof_save();
        }
        let save_current_sctx: sctx_T = current_sctx.get();
        current_sctx.set((*fp).uf_script_ctx);
        let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
        did_emsg.set(false_0);
        if default_arg_err as ::core::ffi::c_int != 0
            && ((*fp).uf_flags & FC_ABORT != 0 || trylevel.get() > 0 as ::core::ffi::c_int)
        {
            did_emsg.set(true_0);
        } else if islambda {
            let mut p: *mut ::core::ffi::c_char = (*((*fp).uf_lines.ga_data
                as *mut *mut ::core::ffi::c_char))
                .offset(7 as ::core::ffi::c_int as isize);
            (*ex_nesting_level.ptr()) += 1;
            eval1(&raw mut p, rettv, EVALARG_EVALUATE.ptr());
            (*ex_nesting_level.ptr()) -= 1;
        } else {
            do_cmdline(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                Some(
                    get_func_line
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                fc as *mut ::core::ffi::c_void,
                DOCMD_NOWAIT as ::core::ffi::c_int
                    | DOCMD_VERBOSE as ::core::ffi::c_int
                    | DOCMD_REPEAT as ::core::ffi::c_int,
            );
        }
        handle_defer_one(current_funccal.get());
        (*RedrawingDisabled.ptr()) -= 1;
        if did_emsg.get() != 0 && (*fp).uf_flags & FC_ABORT != 0
            || (*rettv).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_clear(rettv);
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = -1 as varnumber_T;
        }
        if func_or_func_caller_profiling {
            call_start = profile_end(call_start);
            call_start = profile_sub_wait(wait_start, call_start);
            (*fp).uf_tm_total = profile_add((*fp).uf_tm_total, call_start);
            (*fp).uf_tm_self = profile_self((*fp).uf_tm_self, call_start, (*fp).uf_tm_children);
            if !(*fc).fc_caller.is_null() && (*(*(*fc).fc_caller).fc_func).uf_profiling != 0 {
                (*(*(*fc).fc_caller).fc_func).uf_tm_children =
                    profile_add((*(*(*fc).fc_caller).fc_func).uf_tm_children, call_start);
                (*(*(*fc).fc_caller).fc_func).uf_tml_children =
                    profile_add((*(*(*fc).fc_caller).fc_func).uf_tml_children, call_start);
            }
            if started_profiling {
                (*fp).uf_profiling = false_0;
            }
        }
        if p_verbose.get() >= 12 as OptInt {
            (*no_wait_return.ptr()) += 1;
            verbose_enter_scroll();
            if aborting() {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"%s aborted\0".as_ptr() as *const ::core::ffi::c_char),
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                );
            } else if (*(*fc).fc_rettv).v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"%s returning #%ld\0".as_ptr() as *const ::core::ffi::c_char),
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                    (*(*fc).fc_rettv).vval.v_number,
                );
            } else {
                let mut buf_0: [::core::ffi::c_char; 480] = [0; 480];
                (*emsg_off.ptr()) += 1;
                let mut s_0: *mut ::core::ffi::c_char =
                    encode_tv2string((*fc).fc_rettv, ::core::ptr::null_mut::<size_t>());
                let mut tofree_0: *mut ::core::ffi::c_char = s_0;
                (*emsg_off.ptr()) -= 1;
                if !s_0.is_null() {
                    if vim_strsize(s_0) > MSG_BUF_CLEN {
                        trunc_string(
                            s_0,
                            &raw mut buf_0 as *mut ::core::ffi::c_char,
                            MSG_BUF_CLEN,
                            MSG_BUF_LEN,
                        );
                        s_0 = &raw mut buf_0 as *mut ::core::ffi::c_char;
                    }
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"%s returning %s\0".as_ptr() as *const ::core::ffi::c_char),
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_name,
                        s_0,
                    );
                    xfree(tofree_0 as *mut ::core::ffi::c_void);
                }
            }
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            verbose_leave_scroll();
            (*no_wait_return.ptr()) -= 1;
        }
        estack_pop();
        current_sctx.set(save_current_sctx);
        if do_profiling_yes {
            script_prof_restore(wait_start);
        }
        if using_sandbox {
            (*sandbox.ptr()) -= 1;
        }
        if p_verbose.get() >= 12 as OptInt
            && !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
        {
            (*no_wait_return.ptr()) += 1;
            verbose_enter_scroll();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"continuing in %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
            );
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            verbose_leave_scroll();
            (*no_wait_return.ptr()) -= 1;
        }
        (*did_emsg.ptr()) |= save_did_emsg;
        (*depth.ptr()) -= 1;
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < tv_to_free_len {
            tv_clear(tv_to_free[i_1 as usize]);
            i_1 += 1;
        }
        cleanup_function_call(fc);
        (*fp).uf_calls -= 1;
        if (*fp).uf_calls <= 0 as ::core::ffi::c_int && (*fp).uf_refcount <= 0 as ::core::ffi::c_int
        {
            func_clear_free(fp, false_0 != 0);
        }
        if did_save_redo {
            restoreRedobuff(&raw mut save_redo);
        }
        restore_search_patterns();
    }
}

pub(crate) unsafe extern "C" fn call_user_func_check(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut funcexe: *mut funcexe_T,
    mut selfdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*fp).uf_flags & FC_LUAREF != 0 {
            return typval_exec_lua_callable((*fp).uf_luaref, argcount, argvars, rettv);
        }
        if (*fp).uf_flags & FC_RANGE != 0 && !(*funcexe).fe_doesrange.is_null() {
            *(*funcexe).fe_doesrange = true_0 != 0;
        }
        let mut error: ::core::ffi::c_int = check_user_func_argcount(fp, argcount);
        if error != FCERR_UNKNOWN as ::core::ffi::c_int {
            return error;
        }
        if (*fp).uf_flags & FC_DICT != 0 && selfdict.is_null() {
            error = FCERR_DICT as ::core::ffi::c_int;
        } else {
            call_user_func(
                fp,
                argcount,
                argvars,
                rettv,
                (*funcexe).fe_firstline,
                (*funcexe).fe_lastline,
                if (*fp).uf_flags & FC_DICT != 0 {
                    selfdict
                } else {
                    ::core::ptr::null_mut::<dict_T>()
                },
            );
            error = FCERR_NONE as ::core::ffi::c_int;
        }
        return error;
    }
}

pub(crate) unsafe extern "C" fn user_func_error(
    mut error: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut found_var: bool,
) {
    unsafe {
        match error {
            0 => {
                if found_var {
                    semsg(
                        gettext(&raw const e_not_callable_type_str as *const ::core::ffi::c_char),
                        name,
                    );
                } else {
                    emsg_funcname(
                        &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                        name,
                    );
                }
            }
            8 => {
                emsg_funcname(
                    b"E276: Cannot use function as a method: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    name,
                );
            }
            7 => {
                emsg_funcname(
                    b"E933: Function was deleted: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                );
            }
            1 => {
                emsg_funcname(
                    gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
                    name,
                );
            }
            2 => {
                emsg_funcname(
                    gettext(&raw const e_toofewarg as *const ::core::ffi::c_char),
                    name,
                );
            }
            3 => {
                emsg_funcname(
                    b"E120: Using <SID> not in a script context: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    name,
                );
            }
            4 => {
                emsg_funcname(
                    b"E725: Calling dict function without Dictionary: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    name,
                );
            }
            _ => {}
        };
    }
}

pub unsafe extern "C" fn call_simple_luafunc(
    mut funcname: *const ::core::ffi::c_char,
    mut len: size_t,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = 0 as varnumber_T;
        let mut argvars: [typval_T; 1] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 1];
        argvars[0 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        nlua_typval_call(
            funcname,
            len,
            &raw mut argvars as *mut typval_T,
            0 as ::core::ffi::c_int,
            rettv,
        );
        return OK;
    }
}

pub unsafe extern "C" fn call_simple_func(
    mut funcname: *const ::core::ffi::c_char,
    mut len: size_t,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ret: ::core::ffi::c_int = FAIL;
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = 0 as varnumber_T;
        let mut name: *mut ::core::ffi::c_char = xstrnsave(funcname, len);
        let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
        let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
            name,
            &raw mut fname_buf as *mut ::core::ffi::c_char,
            &raw mut tofree,
            &raw mut error,
        );
        let mut is_global: bool = *fname.offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 'g' as ::core::ffi::c_int
            && *fname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int;
        let mut rfname: *mut ::core::ffi::c_char = if is_global as ::core::ffi::c_int != 0 {
            fname.offset(2 as ::core::ffi::c_int as isize)
        } else {
            fname
        };
        let mut fp: *mut ufunc_T = find_func(rfname);
        if fp.is_null() {
            ret = NOTDONE;
        } else if !fp.is_null() && (*fp).uf_flags & FC_DELETED != 0 {
            error = FCERR_DELETED as ::core::ffi::c_int;
        } else if !fp.is_null() {
            let mut argvars: [typval_T; 1] = [typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            }; 1];
            argvars[0 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
            let mut funcexe: funcexe_T = FUNCEXE_INIT;
            funcexe.fe_evaluate = true_0 != 0;
            error = call_user_func_check(
                fp,
                0 as ::core::ffi::c_int,
                &raw mut argvars as *mut typval_T,
                rettv,
                &raw mut funcexe,
                ::core::ptr::null_mut::<dict_T>(),
            );
            if error == FCERR_NONE as ::core::ffi::c_int {
                ret = OK;
            }
        }
        user_func_error(error, name, false_0 != 0);
        xfree(tofree as *mut ::core::ffi::c_void);
        xfree(name as *mut ::core::ffi::c_void);
        return ret;
    }
}
