//! Child processes: the `job*()` family and the environment it hands
//! them.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_jobpid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut data: *mut Channel = find_job(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        true_0 != 0,
    );
    if data.is_null() {
        return;
    }
    let mut proc: *mut Proc = channel_proc(data);
    (*rettv).vval.v_number = (*proc).pid as varnumber_T;
}
pub unsafe extern "C" fn f_jobresize(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut data: *mut Channel = find_job(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        true_0 != 0,
    );
    if data.is_null() {
        return;
    }
    if (*channel_proc(data)).type_0 as ::core::ffi::c_uint
        != kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            &raw const e_channotpty as *const ::core::ffi::c_char,
        ));
        return;
    }
    pty_proc_resize(
        channel_pty(data),
        (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint16_t,
        (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint16_t,
    );
    (*rettv).vval.v_number = 1 as varnumber_T;
}
static pty_ignored_env_vars: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"COLUMNS\0".as_ptr() as *const ::core::ffi::c_char,
    b"LINES\0".as_ptr() as *const ::core::ffi::c_char,
    b"TERMCAP\0".as_ptr() as *const ::core::ffi::c_char,
    b"COLORFGBG\0".as_ptr() as *const ::core::ffi::c_char,
    b"COLORTERM\0".as_ptr() as *const ::core::ffi::c_char,
    b"VIM\0".as_ptr() as *const ::core::ffi::c_char,
    b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
static required_env_vars: GlobalCell<[*const ::core::ffi::c_char; 1]> =
    GlobalCell::new([::core::ptr::null::<::core::ffi::c_char>()]);
pub unsafe extern "C" fn create_environment(
    mut job_env: *const dictitem_T,
    clear_env: bool,
    pty: bool,
    pty_term_name: *const ::core::ffi::c_char,
) -> *mut dict_T {
    let mut env: *mut dict_T = tv_dict_alloc();
    if !clear_env {
        let mut temp_env: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        f_environ(
            ::core::ptr::null_mut::<typval_T>(),
            &raw mut temp_env,
            EvalFuncData { null: NULL_0 },
        );
        tv_dict_extend(
            env,
            temp_env.vval.v_dict,
            b"force\0".as_ptr() as *const ::core::ffi::c_char,
        );
        tv_dict_free(temp_env.vval.v_dict);
        if pty {
            let mut i: size_t = 0 as size_t;
            while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
                        .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                )
                && !(*pty_ignored_env_vars.ptr())[i as usize].is_null()
            {
                let mut dv: *mut dictitem_T = tv_dict_find(
                    env,
                    (*pty_ignored_env_vars.ptr())[i as usize],
                    -1 as ptrdiff_t,
                );
                if !dv.is_null() {
                    tv_dict_item_remove(env, dv);
                }
                i = i.wrapping_add(1);
            }
            if p_tgc.get() != 0 {
                tv_dict_add_str(
                    env,
                    b"COLORTERM\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                    b"truecolor\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
    }
    if pty {
        let mut dv_0: *mut dictitem_T = tv_dict_find(
            env,
            b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !dv_0.is_null() {
            tv_dict_item_remove(env, dv_0);
        }
        tv_dict_add_str(
            env,
            b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            pty_term_name,
        );
    }
    let mut nvim_addr: *mut ::core::ffi::c_char = get_vim_var_str(VV_SEND_SERVER);
    if *nvim_addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        let mut dv_1: *mut dictitem_T = tv_dict_find(
            env,
            b"NVIM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !dv_1.is_null() {
            tv_dict_item_remove(env, dv_1);
        }
        tv_dict_add_str(
            env,
            b"NVIM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            nvim_addr,
        );
    }
    if !job_env.is_null() {
        tv_dict_extend(
            env,
            (*job_env).di_tv.vval.v_dict,
            b"force\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if pty {
        let mut i_0: size_t = 0 as size_t;
        while i_0
            < ::core::mem::size_of::<[*const ::core::ffi::c_char; 1]>()
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*const ::core::ffi::c_char; 1]>()
                        .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                )
            && !(*required_env_vars.ptr())[i_0 as usize].is_null()
        {
            let mut len: size_t = strlen((*required_env_vars.ptr())[i_0 as usize]);
            let mut dv_2: *mut dictitem_T = tv_dict_find(
                env,
                (*required_env_vars.ptr())[i_0 as usize],
                len as ptrdiff_t,
            );
            if dv_2.is_null() {
                let mut env_var: *mut ::core::ffi::c_char =
                    os_getenv((*required_env_vars.ptr())[i_0 as usize]);
                if !env_var.is_null() {
                    tv_dict_add_allocated_str(
                        env,
                        (*required_env_vars.ptr())[i_0 as usize],
                        len,
                        env_var,
                    );
                }
            }
            i_0 = i_0.wrapping_add(1);
        }
    }
    return env;
}
pub unsafe extern "C" fn f_jobstart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut len: size_t = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeException,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut executable: bool = true_0 != 0;
    let mut argv: *mut *mut ::core::ffi::c_char = tv_to_argv(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut cmd,
        &raw mut executable,
    );
    if argv.is_null() {
        (*rettv).vval.v_number = (if executable as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        }) as varnumber_T;
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected dictionary\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shell_free_argv(argv);
        return;
    }
    let mut job_opts: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut detach: bool = false_0 != 0;
    let mut rpc: bool = false_0 != 0;
    let mut pty: bool = false_0 != 0;
    let mut term: bool = false_0 != 0;
    let mut clear_env: bool = false_0 != 0;
    let mut overlapped: bool = false_0 != 0;
    let mut stdin_mode: ChannelStdinMode = kChannelStdinPipe;
    let mut on_stdout: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut on_stderr: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut on_exit: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut job_env: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        job_opts = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        detach = tv_dict_get_number(job_opts, b"detach\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        rpc = tv_dict_get_number(job_opts, b"rpc\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        term = tv_dict_get_number(job_opts, b"term\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        pty = term as ::core::ffi::c_int != 0
            || tv_dict_get_number(job_opts, b"pty\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T;
        clear_env = tv_dict_get_number(
            job_opts,
            b"clear_env\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as varnumber_T;
        overlapped = tv_dict_get_number(
            job_opts,
            b"overlapped\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as varnumber_T;
        let mut s: *mut ::core::ffi::c_char = tv_dict_get_string(
            job_opts,
            b"stdin\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !s.is_null() {
            if strncmp(
                s,
                b"null\0".as_ptr() as *const ::core::ffi::c_char,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
            ) == 0
            {
                stdin_mode = kChannelStdinNull;
            } else if strncmp(
                s,
                b"pipe\0".as_ptr() as *const ::core::ffi::c_char,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
            ) != 0
            {
                semsg(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    b"stdin\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
            }
        }
        let job_term: *mut dictitem_T = tv_dict_find(
            job_opts,
            b"term\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !job_term.is_null()
            && VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*job_term).di_tv.v_type as ::core::ffi::c_uint
        {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"'term' must be Boolean\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            return;
        }
        if pty as ::core::ffi::c_int != 0 && rpc as ::core::ffi::c_int != 0 {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"job cannot have both 'pty' and 'rpc' options set\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            return;
        }
        let mut new_cwd: *mut ::core::ffi::c_char = tv_dict_get_string(
            job_opts,
            b"cwd\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !new_cwd.is_null() && *new_cwd as ::core::ffi::c_int != NUL {
            cwd = new_cwd;
            if !os_isdir(cwd) {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    b"expected valid directory\0".as_ptr() as *const ::core::ffi::c_char,
                );
                shell_free_argv(argv);
                return;
            }
        }
        job_env = tv_dict_find(
            job_opts,
            b"env\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !job_env.is_null()
            && (*job_env).di_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"env\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            return;
        }
        if !common_job_callbacks(
            job_opts,
            &raw mut on_stdout,
            &raw mut on_stderr,
            &raw mut on_exit,
        ) {
            shell_free_argv(argv);
            return;
        }
    }
    let mut width: uint16_t =
        tv_dict_get_number(job_opts, b"width\0".as_ptr() as *const ::core::ffi::c_char) as uint16_t;
    let mut height: uint16_t =
        tv_dict_get_number(job_opts, b"height\0".as_ptr() as *const ::core::ffi::c_char)
            as uint16_t;
    let mut term_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if term {
        if text_locked() {
            text_locked_msg();
            shell_free_argv(argv);
            return;
        }
        if (*curbuf.get()).b_changed != 0 {
            emsg(gettext(
                b"jobstart(...,{term=true}) requires unmodified buffer\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            shell_free_argv(argv);
            return;
        }
        if !(*curbuf.get()).terminal.is_null() {
            if terminal_running((*curbuf.get()).terminal) {
                semsg(
                    gettext(b"Terminal already connected to buffer %d\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    (*curbuf.get()).handle,
                );
                shell_free_argv(argv);
                return;
            }
            buf_close_terminal(curbuf.get());
        }
        '_c2rust_label: {
            if !rpc {
            } else {
                __assert_fail(
                    b"!rpc\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3606 as ::core::ffi::c_uint,
                    b"void f_jobstart(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        term_name =
            b"xterm-256color\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        cwd = (if !cwd.is_null() {
            cwd as *const ::core::ffi::c_char
        } else {
            b".\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        overlapped = false_0 != 0;
        detach = false_0 != 0;
        stdin_mode = kChannelStdinPipe;
        width = (if width as ::core::ffi::c_int != 0 {
            width as ::core::ffi::c_int
        } else {
            (if 0 as ::core::ffi::c_int > (*curwin.get()).w_view_width - win_col_off(curwin.get()) {
                0 as ::core::ffi::c_int
            } else {
                (*curwin.get()).w_view_width - win_col_off(curwin.get())
            }) as uint16_t as ::core::ffi::c_int
        }) as uint16_t;
        height = (if height as ::core::ffi::c_int != 0 {
            height as ::core::ffi::c_int
        } else {
            (*curwin.get()).w_view_height as uint16_t as ::core::ffi::c_int
        }) as uint16_t;
    }
    if pty {
        term_name = if !term_name.is_null() {
            term_name
        } else {
            tv_dict_get_string(
                job_opts,
                b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            )
        };
        term_name = (if !term_name.is_null() {
            term_name as *const ::core::ffi::c_char
        } else {
            b"ansi\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
    }
    let mut env: *mut dict_T = create_environment(job_env, clear_env, pty, term_name);
    let mut chan: *mut Channel = channel_job_start(
        argv,
        ::core::ptr::null::<::core::ffi::c_char>(),
        on_stdout,
        on_stderr,
        on_exit,
        pty,
        rpc,
        overlapped,
        detach,
        stdin_mode,
        cwd,
        width,
        height,
        env,
        &raw mut (*rettv).vval.v_number,
    );
    if chan.is_null() {
        return;
    } else {
        if !term {
            channel_create_event(chan, ::core::ptr::null::<::core::ffi::c_char>());
        } else {
            if (*rettv).vval.v_number <= 0 as varnumber_T {
                return;
            }
            let pid: ::core::ffi::c_int = (*channel_proc(chan)).pid;
            let buf: *mut buf_T = curbuf.get();
            (*buf).b_p_swf = false_0;
            if (*buf).b_ml.ml_mfp.is_null() && ml_open(buf) == FAIL {
                proc_stop(channel_proc(chan));
                channel_decref(chan);
                return;
            }
            channel_incref(chan);
            channel_terminal_alloc(buf, chan);
            apply_autocmds(
                EVENT_BUFFILEPRE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                buf,
            );
            if !((*chan).term.is_null() || terminal_buf((*chan).term) == 0 as ::core::ffi::c_int) {
                vim_FullName(
                    cwd,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                    false_0 != 0,
                );
                len = home_replace(
                    ::core::ptr::null::<buf_T>(),
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                    true_0 != 0,
                );
                if len != 1 as size_t
                    && ((*IObuff.ptr())[len.wrapping_sub(1 as size_t) as usize]
                        as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        || (*IObuff.ptr())[len.wrapping_sub(1 as size_t) as usize]
                            as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int)
                {
                    (*IObuff.ptr())[len.wrapping_sub(1 as size_t) as usize] =
                        NUL as ::core::ffi::c_char;
                }
                if len == 1 as size_t
                    && (*IObuff.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                {
                    (*IObuff.ptr())[1 as ::core::ffi::c_int as usize] = '.' as ::core::ffi::c_char;
                    (*IObuff.ptr())[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                }
                snprintf(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                    b"term://%s//%d:%s\0".as_ptr() as *const ::core::ffi::c_char,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    pid,
                    cmd,
                );
                setfname(
                    buf,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    true_0 != 0,
                );
                apply_autocmds(
                    EVENT_BUFFILEPOST,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    buf,
                );
                if !((*chan).term.is_null()
                    || terminal_buf((*chan).term) == 0 as ::core::ffi::c_int)
                {
                    err = Error {
                        type_0: kErrorTypeNone,
                        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    };
                    (*buf).b_locked += 1;
                    dict_set_var(
                        (*buf).b_vars,
                        cstr_as_string(b"terminal_job_id\0".as_ptr() as *const ::core::ffi::c_char),
                        object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed_16 {
                                integer: (*chan).id as Integer,
                            },
                        },
                        false_0 != 0,
                        false_0 != 0,
                        ::core::ptr::null_mut::<Arena>(),
                        &raw mut err,
                    );
                    api_clear_error(&raw mut err);
                    dict_set_var(
                        (*buf).b_vars,
                        cstr_as_string(b"terminal_job_pid\0".as_ptr() as *const ::core::ffi::c_char),
                        object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed_16 {
                                integer: pid as Integer,
                            },
                        },
                        false_0 != 0,
                        false_0 != 0,
                        ::core::ptr::null_mut::<Arena>(),
                        &raw mut err,
                    );
                    api_clear_error(&raw mut err);
                    (*buf).b_locked -= 1;
                    if !((*chan).term.is_null()
                        || terminal_buf((*chan).term) == 0 as ::core::ffi::c_int)
                    {
                        terminal_open(&raw mut (*chan).term, buf);
                    }
                }
            }
            channel_create_event(chan, ::core::ptr::null::<::core::ffi::c_char>());
            channel_decref(chan);
        }
        return;
    };
}
pub unsafe extern "C" fn f_jobstop(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut data: *mut Channel = find_job(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        false_0 != 0,
    );
    if data.is_null() {
        return;
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*data).is_rpc {
        channel_close((*data).id, kChannelPartRpc, &raw mut error);
    }
    proc_stop(channel_proc(data));
    (*rettv).vval.v_number = 1 as varnumber_T;
    if !error.is_null() {
        emsg(error);
    }
}
pub unsafe extern "C" fn f_jobwait(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut args: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut jobs: *mut *mut Channel = xcalloc(
        tv_list_len(args) as size_t,
        ::core::mem::size_of::<*mut Channel>(),
    ) as *mut *mut Channel;
    let mut waiting_jobs: *mut MultiQueue = multiqueue_new(
        Some(loop_on_put as unsafe extern "C" fn(*mut MultiQueue, *mut ::core::ffi::c_void) -> ()),
        main_loop.ptr() as *mut ::core::ffi::c_void,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = args;
    if !l_.is_null() {
        let mut arg: *const listitem_T = (*l_).lv_first;
        while !arg.is_null() {
            let mut chan: *mut Channel = ::core::ptr::null_mut::<Channel>();
            if (*arg).li_tv.v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    chan = find_channel((*arg).li_tv.vval.v_number as uint64_t);
                    chan.is_null()
                }
                || (*chan).streamtype as ::core::ffi::c_uint
                    != kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *jobs.offset(i as isize) = ::core::ptr::null_mut::<Channel>();
            } else if proc_is_stopped(&*channel_proc(chan)) {
                proc_wait(
                    channel_proc(chan),
                    -1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<MultiQueue>(),
                );
                *jobs.offset(i as isize) = ::core::ptr::null_mut::<Channel>();
            } else {
                *jobs.offset(i as isize) = chan;
                channel_incref(chan);
                if (*channel_proc(chan)).status < 0 as ::core::ffi::c_int {
                    multiqueue_process_events((*chan).events);
                    multiqueue_replace_parent((*chan).events, waiting_jobs);
                }
            }
            i += 1;
            arg = (*arg).li_next;
        }
    }
    let mut remaining: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut before: uint64_t = 0 as uint64_t;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number
            >= 0 as varnumber_T
    {
        remaining = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number as ::core::ffi::c_int;
        before = os_hrtime();
    }
    let busy: bool = remaining != 0 as ::core::ffi::c_int;
    if busy {
        ui_busy_start();
        ui_flush();
    }
    i = 0 as ::core::ffi::c_int;
    while i < tv_list_len(args) {
        if remaining == 0 as ::core::ffi::c_int {
            break;
        }
        if !(*jobs.offset(i as isize)).is_null() {
            let mut status: ::core::ffi::c_int = proc_wait(
                channel_proc(*jobs.offset(i as isize)),
                remaining,
                waiting_jobs,
            );
            if status < 0 as ::core::ffi::c_int {
                break;
            }
            if remaining > 0 as ::core::ffi::c_int {
                let mut now: uint64_t = os_hrtime();
                remaining = if (0 as ::core::ffi::c_int)
                    < remaining
                        - now.wrapping_sub(before).wrapping_div(1000000 as uint64_t)
                            as ::core::ffi::c_int
                {
                    0 as ::core::ffi::c_int
                } else {
                    remaining
                        - now.wrapping_sub(before).wrapping_div(1000000 as uint64_t)
                            as ::core::ffi::c_int
                };
                before = now;
            }
        }
        i += 1;
    }
    let rv: *mut list_T = tv_list_alloc(tv_list_len(args) as ptrdiff_t);
    i = 0 as ::core::ffi::c_int;
    while i < tv_list_len(args) {
        if (*jobs.offset(i as isize)).is_null() {
            tv_list_append_number(rv, -3 as varnumber_T);
        } else {
            multiqueue_process_events((**jobs.offset(i as isize)).events);
            multiqueue_replace_parent(
                (**jobs.offset(i as isize)).events,
                (*main_loop.ptr()).events,
            );
            tv_list_append_number(
                rv,
                (*channel_proc(*jobs.offset(i as isize))).status as varnumber_T,
            );
            channel_decref(*jobs.offset(i as isize));
        }
        i += 1;
    }
    multiqueue_free(waiting_jobs);
    xfree(jobs as *mut ::core::ffi::c_void);
    if busy {
        ui_busy_stop();
    }
    tv_list_ref(rv);
    (*rettv).v_type = VAR_LIST;
    (*rettv).vval.v_list = rv;
}
