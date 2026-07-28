//! Calling things: `call()`, `function()`, `eval()`, `execute()` and the
//! bridges to the script hosts.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_call(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_list_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_list
        .is_null()
    {
        return;
    }
    let mut owned: bool = false_0 != 0;
    let mut func: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        func = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        partial = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_partial;
        func = partial_name(partial);
    } else if nlua_is_table_from_lua(argvars.offset(0 as ::core::ffi::c_int as isize)) {
        func = nlua_register_table_as_callable(argvars.offset(0 as ::core::ffi::c_int as isize));
        owned = true_0 != 0;
    } else {
        func = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
    }
    if func.is_null() || *func as ::core::ffi::c_int == NUL {
        return;
    }
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut p: *mut ::core::ffi::c_char = func;
        tofree = trans_function_name(
            &raw mut p,
            false_0 != 0,
            TFN_INT as ::core::ffi::c_int | TFN_QUIET as ::core::ffi::c_int,
            ::core::ptr::null_mut::<funcdict_T>(),
            ::core::ptr::null_mut::<*mut partial_T>(),
        );
        if tofree.is_null() {
            emsg_funcname(
                &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                func,
            );
            return;
        }
        func = tofree;
    }
    let mut selfdict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    '_done: {
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_check_for_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
                break '_done;
            } else {
                selfdict = (*argvars.offset(2 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict;
            }
        }
        func_call(
            func,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            partial,
            selfdict,
            rettv,
        );
    }
    if owned {
        func_unref(func);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_eval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut s: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if !s.is_null() {
        s = skipwhite(s);
    }
    let expr_start: *const ::core::ffi::c_char = s;
    if s.is_null()
        || eval1(
            &raw mut s as *mut *mut ::core::ffi::c_char,
            rettv,
            EVALARG_EVALUATE.ptr(),
        ) == FAIL
    {
        if !expr_start.is_null() && !aborting() {
            semsg(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                expr_start,
            );
        }
        need_clr_eos.set(false_0 != 0);
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = 0 as varnumber_T;
    } else if *s as ::core::ffi::c_int != NUL {
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            s,
        );
    }
}
unsafe extern "C" fn get_list_line(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let p: *mut GetListLineCookie = cookie as *mut GetListLineCookie;
    let item: *const listitem_T = (*p).li;
    if item.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let s: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        &raw const (*item).li_tv,
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    (*p).li = (*item).li_next;
    return if s.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(s)
    };
}
pub unsafe extern "C" fn execute_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_off: ::core::ffi::c_int,
) {
    let save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    let save_emsg_silent: ::core::ffi::c_int = emsg_silent.get();
    let save_emsg_noredir: bool = emsg_noredir.get();
    let save_redir_off: bool = redir_off.get();
    let save_capture_ga: *mut garray_T = capture_ga.get();
    let save_msg_col: ::core::ffi::c_int = msg_col.get();
    let mut echo_output: bool = false_0 != 0;
    if check_secure() {
        return;
    }
    if (*argvars.offset((arg_off + 1 as ::core::ffi::c_int) as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let s: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset((arg_off + 1 as ::core::ffi::c_int) as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if s.is_null() {
            return;
        }
        if *s as ::core::ffi::c_int == NUL {
            echo_output = true_0 != 0;
        }
        if strncmp(
            s,
            b"silent\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            (*msg_silent.ptr()) += 1;
        }
        if strcmp(s, b"silent!\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        {
            emsg_silent.set(true_0);
            emsg_noredir.set(true_0 != 0);
        }
    } else {
        (*msg_silent.ptr()) += 1;
    }
    let mut capture_local: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut capture_local,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    capture_ga.set(&raw mut capture_local);
    redir_off.set(false_0 != 0);
    if !echo_output {
        msg_col.set(0 as ::core::ffi::c_int);
    }
    if (*argvars.offset(arg_off as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        do_cmdline_cmd(tv_get_string(argvars.offset(arg_off as isize)));
    } else if !(*argvars.offset(arg_off as isize)).vval.v_list.is_null() {
        let list: *mut list_T = (*argvars.offset(arg_off as isize)).vval.v_list;
        tv_list_ref(list);
        let mut cookie: GetListLineCookie = GetListLineCookie {
            l: list,
            li: tv_list_first(list),
        };
        do_cmdline(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            Some(
                get_list_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
            &raw mut cookie as *mut ::core::ffi::c_void,
            DOCMD_NOWAIT as ::core::ffi::c_int
                | DOCMD_VERBOSE as ::core::ffi::c_int
                | DOCMD_REPEAT as ::core::ffi::c_int
                | DOCMD_KEYTYPED as ::core::ffi::c_int,
        );
        tv_list_unref(list);
    }
    msg_silent.set(save_msg_silent);
    emsg_silent.set(save_emsg_silent);
    emsg_noredir.set(save_emsg_noredir);
    redir_off.set(save_redir_off);
    if echo_output {
        msg_col.set(0 as ::core::ffi::c_int);
    } else {
        msg_col.set(save_msg_col);
    }
    ga_append(capture_ga.get(), NUL as uint8_t);
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = (*capture_ga.get()).ga_data as *mut ::core::ffi::c_char;
    capture_ga.set(save_capture_ga);
}
pub unsafe extern "C" fn f_execute(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    execute_common(argvars, rettv, 0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_exists(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: ::core::ffi::c_int = false_0;
    let mut p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
        if os_env_exists(p.offset(1 as ::core::ffi::c_int as isize), false_0 != 0) {
            n = true_0;
        } else {
            let exp_0: *mut ::core::ffi::c_char = expand_env_save(p as *mut ::core::ffi::c_char);
            if !exp_0.is_null() && *exp_0 as ::core::ffi::c_int != '$' as ::core::ffi::c_int {
                n = true_0;
            }
            xfree(exp_0 as *mut ::core::ffi::c_void);
        }
    } else if *p as ::core::ffi::c_int == '&' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        n = (eval_option(&raw mut p, ::core::ptr::null_mut::<typval_T>(), true_0 != 0) == OK)
            as ::core::ffi::c_int;
        if *skipwhite(p) as ::core::ffi::c_int != NUL {
            n = false_0;
        }
    } else if *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        n = if strnequal(
            p,
            b"*v:lua.\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) as ::core::ffi::c_int
            != 0
        {
            nlua_func_exists(p.offset(7 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
        } else {
            function_exists(p.offset(1 as ::core::ffi::c_int as isize), false_0 != 0)
                as ::core::ffi::c_int
        };
    } else if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
        n = cmd_exists(p.offset(1 as ::core::ffi::c_int as isize));
    } else if *p as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
        {
            n = autocmd_supported(p.offset(2 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        } else {
            n = au_exists(p.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        }
    } else {
        n = var_exists(p) as ::core::ffi::c_int;
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
unsafe extern "C" fn common_function(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut is_funcref: bool,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_string: bool = false_0 != 0;
    let mut arg_pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut trans_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        s = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_partial
            .is_null()
    {
        arg_pt = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_partial;
        s = partial_name(arg_pt);
    } else {
        s = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
        use_string = true_0 != 0;
    }
    if use_string as ::core::ffi::c_int != 0 && vim_strchr(s, AUTOLOAD_CHAR).is_null()
        || is_funcref as ::core::ffi::c_int != 0
    {
        name = s;
        trans_name = save_function_name(
            &raw mut name,
            false_0 != 0,
            TFN_INT as ::core::ffi::c_int
                | TFN_QUIET as ::core::ffi::c_int
                | TFN_NO_AUTOLOAD as ::core::ffi::c_int
                | TFN_NO_DEREF as ::core::ffi::c_int,
            ::core::ptr::null_mut::<funcdict_T>(),
        );
        if *name as ::core::ffi::c_int != NUL {
            s = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    '_theend: {
        if s.is_null()
            || *s as ::core::ffi::c_int == NUL
            || use_string as ::core::ffi::c_int != 0
                && ascii_isdigit(*s as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || is_funcref as ::core::ffi::c_int != 0 && trans_name.is_null()
        {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                if use_string as ::core::ffi::c_int != 0 {
                    tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                } else {
                    s as *const ::core::ffi::c_char
                },
            );
        } else if !trans_name.is_null()
            && (if is_funcref as ::core::ffi::c_int != 0 {
                find_func(trans_name).is_null() as ::core::ffi::c_int
            } else {
                !translated_function_exists(trans_name) as ::core::ffi::c_int
            }) != 0
        {
            semsg(
                gettext(b"E700: Unknown function: %s\0".as_ptr() as *const ::core::ffi::c_char),
                s,
            );
        } else {
            let mut dict_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut list: *mut list_T = ::core::ptr::null_mut::<list_T>();
            if strncmp(
                s,
                b"s:\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
                || strncmp(
                    s,
                    b"<SID>\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                name = get_scriptlocal_funcname(s);
            } else {
                name = xstrdup(s);
            }
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    arg_idx = 1 as ::core::ffi::c_int;
                    dict_idx = 2 as ::core::ffi::c_int;
                } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
                    as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    dict_idx = 1 as ::core::ffi::c_int;
                } else {
                    arg_idx = 1 as ::core::ffi::c_int;
                }
                if dict_idx > 0 as ::core::ffi::c_int {
                    if tv_check_for_dict_arg(argvars, dict_idx) == FAIL {
                        xfree(name as *mut ::core::ffi::c_void);
                        break '_theend;
                    } else if (*argvars.offset(dict_idx as isize)).vval.v_dict.is_null() {
                        dict_idx = 0 as ::core::ffi::c_int;
                    }
                }
                if arg_idx > 0 as ::core::ffi::c_int {
                    if (*argvars.offset(arg_idx as isize)).v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        emsg(gettext(
                            b"E923: Second argument of function() must be a list or a dict\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ));
                        xfree(name as *mut ::core::ffi::c_void);
                        break '_theend;
                    } else {
                        list = (*argvars.offset(arg_idx as isize)).vval.v_list;
                        if tv_list_len(list) == 0 as ::core::ffi::c_int {
                            arg_idx = 0 as ::core::ffi::c_int;
                        } else if tv_list_len(list) > MAX_FUNC_ARGS as ::core::ffi::c_int {
                            emsg_funcname(&raw const e_toomanyarg as *const ::core::ffi::c_char, s);
                            xfree(name as *mut ::core::ffi::c_void);
                            break '_theend;
                        }
                    }
                }
            }
            if dict_idx > 0 as ::core::ffi::c_int
                || arg_idx > 0 as ::core::ffi::c_int
                || !arg_pt.is_null()
                || is_funcref as ::core::ffi::c_int != 0
            {
                let pt: *mut partial_T =
                    xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
                if arg_idx > 0 as ::core::ffi::c_int
                    || !arg_pt.is_null() && (*arg_pt).pt_argc > 0 as ::core::ffi::c_int
                {
                    let arg_len: ::core::ffi::c_int = if arg_pt.is_null() {
                        0 as ::core::ffi::c_int
                    } else {
                        (*arg_pt).pt_argc
                    };
                    let lv_len: ::core::ffi::c_int = tv_list_len(list);
                    (*pt).pt_argc = arg_len + lv_len;
                    (*pt).pt_argv = xmalloc(
                        ::core::mem::size_of::<typval_T>().wrapping_mul((*pt).pt_argc as size_t),
                    ) as *mut typval_T;
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < arg_len {
                        tv_copy(
                            (*arg_pt).pt_argv.offset(i as isize),
                            (*pt).pt_argv.offset(i as isize),
                        );
                        i += 1;
                    }
                    if lv_len > 0 as ::core::ffi::c_int {
                        let l_: *mut list_T = list;
                        if !l_.is_null() {
                            let mut li: *mut listitem_T = (*l_).lv_first;
                            while !li.is_null() {
                                let c2rust_fresh7 = i;
                                i = i + 1;
                                tv_copy(
                                    &raw mut (*li).li_tv,
                                    (*pt).pt_argv.offset(c2rust_fresh7 as isize),
                                );
                                li = (*li).li_next;
                            }
                        }
                    }
                }
                if dict_idx > 0 as ::core::ffi::c_int {
                    (*pt).pt_dict = (*argvars.offset(dict_idx as isize)).vval.v_dict;
                    (*(*pt).pt_dict).dv_refcount += 1;
                } else if !arg_pt.is_null() {
                    (*pt).pt_dict = (*arg_pt).pt_dict;
                    (*pt).pt_auto = (*arg_pt).pt_auto;
                    if !(*pt).pt_dict.is_null() {
                        (*(*pt).pt_dict).dv_refcount += 1;
                    }
                }
                (*pt).pt_refcount = 1 as ::core::ffi::c_int;
                if !arg_pt.is_null() && !(*arg_pt).pt_func.is_null() {
                    (*pt).pt_func = (*arg_pt).pt_func;
                    func_ptr_ref((*pt).pt_func);
                    xfree(name as *mut ::core::ffi::c_void);
                } else if is_funcref {
                    (*pt).pt_func = find_func(trans_name);
                    func_ptr_ref((*pt).pt_func);
                    xfree(name as *mut ::core::ffi::c_void);
                } else {
                    (*pt).pt_name = name;
                    func_ref(name);
                }
                (*rettv).v_type = VAR_PARTIAL;
                (*rettv).vval.v_partial = pt;
            } else {
                (*rettv).v_type = VAR_FUNC;
                (*rettv).vval.v_string = name;
                func_ref(name);
            }
        }
    }
    xfree(trans_name as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_funcref(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    common_function(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_function(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    common_function(argvars, rettv, false_0 != 0);
}
pub unsafe extern "C" fn f_garbagecollect(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    want_garbage_collect.set(true_0 != 0);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) == 1 as varnumber_T
    {
        garbage_collect_at_exit.set(true_0 != 0);
    }
}
unsafe extern "C" fn libcall_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut out_type: ::core::ffi::c_int,
) {
    (*rettv).v_type = out_type as VarType;
    if out_type != VAR_NUMBER as ::core::ffi::c_int {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if check_secure() {
        return;
    }
    if (*argvars).v_type != VAR_STRING || (*argvars.offset(1)).v_type != VAR_STRING {
        return;
    }
    let libname: *const ::core::ffi::c_char = (*argvars).vval.v_string;
    let funcname: *const ::core::ffi::c_char = (*argvars.offset(1)).vval.v_string;
    let arg3: *mut typval_T = argvars.offset(2);
    let str_in: *mut ::core::ffi::c_char = if (*arg3).v_type == VAR_STRING {
        (*arg3).vval.v_string
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    // A VAR_STRING third argument with a NULL v_string falls through to the
    // int-taking prototype, reading the same union as a number. Upstream
    // quirk, preserved.
    let arg = if str_in.is_null() {
        LibcallArg::Int((*arg3).vval.v_number as ::core::ffi::c_int)
    } else {
        LibcallArg::Str(CStr::from_ptr(str_in))
    };
    let want = if out_type == VAR_STRING as ::core::ffi::c_int {
        LibcallReturn::Str
    } else {
        LibcallReturn::Int
    };
    let result = if libname.is_null() || funcname.is_null() {
        None
    } else {
        os_libcall(CStr::from_ptr(libname), CStr::from_ptr(funcname), arg, want)
    };
    match result {
        None => {
            semsg(
                gettext(&raw const e_libcall as *const ::core::ffi::c_char),
                funcname,
            );
        }
        Some(LibcallResult::Str(s)) => {
            (*rettv).vval.v_string = s.map_or(::core::ptr::null_mut(), CString::into_raw);
        }
        Some(LibcallResult::Int(n)) => (*rettv).vval.v_number = n as varnumber_T,
    }
}
pub unsafe extern "C" fn f_libcall(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    libcall_common(argvars, rettv, VAR_STRING as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_libcallnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    libcall_common(argvars, rettv, VAR_NUMBER as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_luaeval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if str.is_null() {
        return;
    }
    nlua_typval_eval(
        cstr_as_string(str),
        argvars.offset(1 as ::core::ffi::c_int as isize),
        rettv,
    );
}
pub unsafe extern "C" fn f_py3eval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    script_host_eval(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        argvars,
        rettv,
    );
}
pub unsafe extern "C" fn f_perleval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    script_host_eval(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        argvars,
        rettv,
    );
}
pub unsafe extern "C" fn f_rubyeval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    script_host_eval(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        argvars,
        rettv,
    );
}
