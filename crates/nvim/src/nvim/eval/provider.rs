//! Calling out of the evaluator: provider script hosts, the job callbacks
//! they are driven by, and prompt-buffer callbacks.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn common_job_callbacks(
    mut vopts: *mut dict_T,
    mut on_stdout: *mut CallbackReader,
    mut on_stderr: *mut CallbackReader,
    mut on_exit: *mut Callback,
) -> bool {
    if tv_dict_get_callback(
        vopts,
        b"on_stdout\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as usize) as ptrdiff_t,
        &raw mut (*on_stdout).cb,
    ) as c_int
        != 0
        && tv_dict_get_callback(
            vopts,
            b"on_stderr\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as usize) as ptrdiff_t,
            &raw mut (*on_stderr).cb,
        ) as c_int
            != 0
        && tv_dict_get_callback(
            vopts,
            b"on_exit\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
            on_exit,
        ) as c_int
            != 0
    {
        (*on_stdout).buffered =
            tv_dict_get_number(vopts, b"stdout_buffered\0".as_ptr() as *const c_char) != 0;
        (*on_stderr).buffered =
            tv_dict_get_number(vopts, b"stderr_buffered\0".as_ptr() as *const c_char) != 0;
        if (*on_stdout).buffered as c_int != 0
            && (*on_stdout).cb.type_0 as c_uint == kCallbackNone as c_int as c_uint
        {
            (*on_stdout).self_0 = vopts;
        }
        if (*on_stderr).buffered as c_int != 0
            && (*on_stderr).cb.type_0 as c_uint == kCallbackNone as c_int as c_uint
        {
            (*on_stderr).self_0 = vopts;
        }
        (*vopts).dv_refcount += 1;
        return true_0 != 0;
    }
    callback_reader_free(on_stdout);
    callback_reader_free(on_stderr);
    callback_free(on_exit);
    return false_0 != 0;
}

pub unsafe extern "C" fn find_job(mut id: uint64_t, mut show_error: bool) -> *mut Channel {
    let mut data: *mut Channel = find_channel(id);
    if data.is_null()
        || (*data).streamtype as c_uint != kChannelStreamProc as c_int as c_uint
        || proc_is_stopped(&*channel_proc(data)) as c_int != 0
    {
        if show_error {
            if !data.is_null()
                && (*data).streamtype as c_uint != kChannelStreamProc as c_int as c_uint
            {
                emsg(gettext(&raw const e_invchanjob as *const c_char));
            } else {
                emsg(gettext(&raw const e_invchan as *const c_char));
            }
        }
        return ::core::ptr::null_mut::<Channel>();
    }
    return data;
}

pub unsafe extern "C" fn script_host_eval(
    mut name: *mut c_char,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as c_int as isize)).v_type as c_uint != VAR_STRING as c_int as c_uint {
        emsg(gettext(&raw const e_invarg as *const c_char));
        return;
    }
    let mut args: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
    tv_list_append_string(
        args,
        (*argvars.offset(0 as c_int as isize)).vval.v_string,
        -1 as ssize_t,
    );
    *rettv = eval_call_provider(
        name,
        b"eval\0".as_ptr() as *const c_char as *mut c_char,
        args,
        false_0 != 0,
    );
}

pub unsafe extern "C" fn eval_call_provider(
    mut provider: *mut c_char,
    mut method: *mut c_char,
    mut arguments: *mut list_T,
    mut discard: bool,
) -> typval_T {
    if !eval_has_provider(provider, false_0 != 0) {
        semsg(
            b"E319: No \"%s\" provider found. Run \":checkhealth vim.provider\"\0".as_ptr()
                as *const c_char,
            provider,
        );
        return typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: 0 as varnumber_T,
            },
        };
    }
    let mut func: [c_char; 256] = [0; 256];
    let mut name_len: c_int = snprintf(
        &raw mut func as *mut c_char,
        ::core::mem::size_of::<[c_char; 256]>(),
        b"provider#%s#Call\0".as_ptr() as *const c_char,
        provider,
    );
    let mut saved_provider_caller_scope: caller_scope = provider_caller_scope.get() as caller_scope;
    provider_caller_scope.set(caller_scope {
        script_ctx: current_sctx.get(),
        es_entry: *((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize),
        autocmd_fname: autocmd_fname.get(),
        autocmd_match: autocmd_match.get(),
        autocmd_fname_full: autocmd_fname_full.get(),
        autocmd_bufnr: autocmd_bufnr.get(),
        funccalp: get_current_funccal() as *mut c_void,
    } as caller_scope);
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    (*provider_call_nesting.ptr()) += 1;
    let mut argvars: [typval_T; 3] = [
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_string: method },
        },
        typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: arguments },
        },
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
    ];
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tv_list_ref(arguments);
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    call_func(
        &raw mut func as *mut c_char,
        name_len,
        &raw mut rettv,
        2 as c_int,
        &raw mut argvars as *mut typval_T,
        &raw mut funcexe,
    );
    tv_list_unref(arguments);
    restore_funccal();
    provider_caller_scope.set(saved_provider_caller_scope as caller_scope);
    (*provider_call_nesting.ptr()) -= 1;
    '_c2rust_label: {
        if provider_call_nesting.get() >= 0 as c_int {
        } else {
            __assert_fail(
                b"provider_call_nesting >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                6585 as c_uint,
                b"typval_T eval_call_provider(char *, char *, list_T *, _Bool)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    if discard {
        tv_clear(&raw mut rettv);
    }
    return rettv;
}

pub unsafe extern "C" fn eval_has_provider(
    mut feat: *const c_char,
    mut throw_if_fast: bool,
) -> bool {
    if !strequal(feat, b"clipboard\0".as_ptr() as *const c_char)
        && !strequal(feat, b"python3\0".as_ptr() as *const c_char)
        && !strequal(feat, b"python3_compiled\0".as_ptr() as *const c_char)
        && !strequal(feat, b"python3_dynamic\0".as_ptr() as *const c_char)
        && !strequal(feat, b"perl\0".as_ptr() as *const c_char)
        && !strequal(feat, b"ruby\0".as_ptr() as *const c_char)
        && !strequal(feat, b"node\0".as_ptr() as *const c_char)
    {
        return false_0 != 0;
    }
    if throw_if_fast as c_int != 0 && !nlua_is_deferred_safe() {
        semsg(
            &raw const e_fast_api_disabled as *const c_char,
            b"Vimscript function\0".as_ptr() as *const c_char,
        );
        return false_0 != 0;
    }
    let mut name: [c_char; 32] = [0; 32];
    snprintf(
        &raw mut name as *mut c_char,
        ::core::mem::size_of::<[c_char; 32]>(),
        b"%s\0".as_ptr() as *const c_char,
        feat,
    );
    strchrsub(&raw mut name as *mut c_char, '_' as c_char, NUL as c_char);
    let mut buf: [c_char; 256] = [0; 256];
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut len: c_int = snprintf(
        &raw mut buf as *mut c_char,
        ::core::mem::size_of::<[c_char; 256]>(),
        b"g:loaded_%s_provider\0".as_ptr() as *const c_char,
        &raw mut name as *mut c_char,
    );
    if eval_variable(
        &raw mut buf as *mut c_char,
        len,
        &raw mut tv,
        ::core::ptr::null_mut::<*mut dictitem_T>(),
        false_0 != 0,
        true_0 != 0,
    ) == FAIL
    {
        len = snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 256]>(),
            b"provider#%s#bogus\0".as_ptr() as *const c_char,
            &raw mut name as *mut c_char,
        );
        script_autoload(&raw mut buf as *mut c_char, len as size_t, false_0 != 0);
        len = snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 256]>(),
            b"g:loaded_%s_provider\0".as_ptr() as *const c_char,
            &raw mut name as *mut c_char,
        );
        if eval_variable(
            &raw mut buf as *mut c_char,
            len,
            &raw mut tv,
            ::core::ptr::null_mut::<*mut dictitem_T>(),
            false_0 != 0,
            true_0 != 0,
        ) == FAIL
        {
            snprintf(
                &raw mut buf as *mut c_char,
                ::core::mem::size_of::<[c_char; 256]>(),
                b"provider#%s#Call\0".as_ptr() as *const c_char,
                &raw mut name as *mut c_char,
            );
            if !find_func(&raw mut buf as *mut c_char).is_null() && p_lpl.get() != 0 {
                semsg(
                    b"provider: %s: missing required variable g:loaded_%s_provider\0".as_ptr()
                        as *const c_char,
                    &raw mut name as *mut c_char,
                    &raw mut name as *mut c_char,
                );
            }
            return false_0 != 0;
        }
    }
    let mut ok: bool = if tv.v_type as c_uint == VAR_NUMBER as c_int as c_uint {
        (2 as varnumber_T == tv.vval.v_number) as c_int
    } else {
        false_0
    } != 0;
    if ok {
        snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 256]>(),
            b"provider#%s#Call\0".as_ptr() as *const c_char,
            &raw mut name as *mut c_char,
        );
        if find_func(&raw mut buf as *mut c_char).is_null() {
            semsg(
                b"provider: %s: g:loaded_%s_provider=2 but %s is not defined\0".as_ptr()
                    as *const c_char,
                &raw mut name as *mut c_char,
                &raw mut name as *mut c_char,
                &raw mut buf as *mut c_char,
            );
            ok = false_0 != 0;
        }
    }
    return ok;
}

pub unsafe extern "C" fn eval_fmt_source_name_line(mut buf: *mut c_char, mut bufsize: size_t) {
    if !(*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
    .es_name
    .is_null()
    {
        snprintf(
            buf,
            bufsize,
            b"%s:%d\0".as_ptr() as *const c_char,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
            .es_name,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
            .es_lnum,
        );
    } else {
        snprintf(buf, bufsize, b"?\0".as_ptr() as *const c_char);
    };
}

pub unsafe extern "C" fn prompt_get_input(mut buf: *mut buf_T) -> *mut c_char {
    if !bt_prompt(buf) {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut lnum_start: linenr_T = (*buf).b_prompt_start.mark.lnum;
    let mut lnum_last: linenr_T = (*buf).b_ml.ml_line_count;
    let mut text: *mut c_char = ml_get_buf(buf, lnum_start);
    if strlen(text) as c_int >= (*buf).b_prompt_start.mark.col {
        text = text.offset((*buf).b_prompt_start.mark.col as isize);
    }
    let mut full_text: *mut c_char = xstrdup(text);
    let mut i: linenr_T = lnum_start + 1 as linenr_T;
    while i <= lnum_last {
        let mut half_text: *mut c_char = concat_str(full_text, b"\n\0".as_ptr() as *const c_char);
        xfree(full_text as *mut c_void);
        full_text = concat_str(half_text, ml_get_buf(buf, i));
        xfree(half_text as *mut c_void);
        i += 1;
    }
    return full_text;
}

pub unsafe extern "C" fn prompt_invoke_callback() {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut argv: [typval_T; 2] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 2];
    let mut lnum: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    let mut user_input: *mut c_char = prompt_get_input(curbuf.get());
    if user_input.is_null() {
        return;
    }
    ml_append(
        lnum,
        b"\0".as_ptr() as *const c_char as *mut c_char,
        0 as colnr_T,
        false_0 != 0,
    );
    appended_lines_mark(lnum, 1 as c_int);
    (*curwin.get()).w_cursor.lnum = lnum + 1 as linenr_T;
    (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
    (*curbuf.get()).b_prompt_start.mark.lnum = lnum + 1 as linenr_T;
    if (*curbuf.get()).b_prompt_callback.type_0 as c_uint == kCallbackNone as c_int as c_uint {
        xfree(user_input as *mut c_void);
    } else {
        argv[0 as c_int as usize].v_type = VAR_STRING;
        argv[0 as c_int as usize].vval.v_string = user_input;
        argv[1 as c_int as usize].v_type = VAR_UNKNOWN;
        callback_call(
            &raw mut (*curbuf.get()).b_prompt_callback,
            1 as c_int,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as c_int as isize));
        tv_clear(&raw mut rettv);
    }
    u_clearallandblockfree(curbuf.get());
    (*curbuf.get()).b_prompt_start.mark.lnum = (*curbuf.get()).b_ml.ml_line_count;
    (*curbuf.get()).b_prompt_append_new_line = true_0 != 0;
}

pub unsafe extern "C" fn invoke_prompt_interrupt() -> bool {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut argv: [typval_T; 1] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 1];
    if (*curbuf.get()).b_prompt_interrupt.type_0 as c_uint == kCallbackNone as c_int as c_uint {
        return false_0 != 0;
    }
    argv[0 as c_int as usize].v_type = VAR_UNKNOWN;
    got_int.set(false_0 != 0);
    let mut ret: c_int = callback_call(
        &raw mut (*curbuf.get()).b_prompt_interrupt,
        0 as c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    ) as c_int;
    tv_clear(&raw mut rettv);
    return ret != FAIL;
}
