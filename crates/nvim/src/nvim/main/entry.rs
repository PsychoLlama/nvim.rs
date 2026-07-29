//! Process entry, and the order the startup does things in.
//!
//! `main_0` is the sequence itself: it reads as a list of phases because that is
//! what it is, and the ordering between them is load-bearing -- options before
//! buffers, buffers before windows, windows before the UI, the UI before
//! `VimEnter`.

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn event_init() {
    loop_init(main_loop.ptr());
    env_init();
    resize_events.set(multiqueue_new_child((*main_loop.ptr()).events));
    autocmd_init();
    signal_init();
    channel_init();
    terminal_init();
    ui_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"event init\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}

pub(crate) unsafe extern "C" fn event_teardown() -> bool {
    if (*main_loop.ptr()).events.is_null() {
        input_stop();
        return true_0 != 0;
    }
    multiqueue_process_events((*main_loop.ptr()).events);
    loop_poll_events(main_loop.ptr(), 0 as int64_t);
    input_stop();
    server_teardown();
    channel_teardown();
    proc_teardown(main_loop.ptr());
    timer_teardown();
    signal_teardown();
    terminal_teardown();
    return loop_close(main_loop.ptr(), true_0 != 0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn early_init(mut paramp: *mut mparm_T) {
    os_hint_priority();
    estack_init();
    cmdline_init();
    eval_init();
    set_vim_var_nr(VV_STARTTIME, os_realtime());
    init_path(if !(*argv0.ptr()).is_null() {
        argv0.get() as *const c_char
    } else {
        b"nvim\0".as_ptr() as *const c_char
    });
    init_normal_cmds();
    runtime_init();
    highlight_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"early init\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    init_locale();
    set_init_tablocal();
    win_alloc_first();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init first window\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    alist_init(global_alist.ptr());
    (*global_alist.ptr()).id = 0 as c_int;
    init_homedir();
    set_init_1(
        if !paramp.is_null() {
            (*paramp).clean as c_int
        } else {
            false_0
        } != 0,
    );
    log_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 1\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_lang_var();
    qf_init_stack();
}

pub(crate) unsafe fn main_0(mut argc: c_int, mut argv: *mut *mut c_char) -> c_int {
    argv0.set(*argv.offset(0 as c_int as isize));
    if !appname_is_valid() {
        fprintf(
            stderr,
            b"$NVIM_APPNAME must be a name or relative path.\n\0".as_ptr() as *const c_char,
        );
        exit(1 as c_int);
    }
    if argc > 1 as c_int
        && strcasecmp(
            *argv.offset(1 as c_int as isize),
            b"-ll\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
    {
        if argc == 2 as c_int {
            print_mainerr(
                err_arg_missing.get(),
                *argv.offset(1 as c_int as isize),
                ::core::ptr::null::<c_char>(),
            );
            exit(1 as c_int);
        }
        nlua_run_script(argv, argc, 3 as c_int);
    }
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut params: mparm_T = mparm_T {
        argc: 0,
        argv: ::core::ptr::null_mut::<*mut c_char>(),
        use_vimrc: ::core::ptr::null_mut::<c_char>(),
        clean: false,
        n_commands: 0,
        commands: [::core::ptr::null_mut::<c_char>(); 10],
        cmds_tofree: [0; 10],
        n_pre_commands: 0,
        pre_commands: [::core::ptr::null_mut::<c_char>(); 10],
        luaf: ::core::ptr::null_mut::<c_char>(),
        lua_arg0: 0,
        edit_type: 0,
        tagname: ::core::ptr::null_mut::<c_char>(),
        use_ef: ::core::ptr::null_mut::<c_char>(),
        input_istext: false,
        no_swap_file: 0,
        use_debug_break_level: 0,
        window_count: 0,
        window_layout: 0,
        diff_mode: 0,
        listen_addr: ::core::ptr::null_mut::<c_char>(),
        remote: 0,
        server_addr: ::core::ptr::null_mut::<c_char>(),
        scriptin: ::core::ptr::null_mut::<c_char>(),
        scriptout: ::core::ptr::null_mut::<c_char>(),
        scriptout_append: false,
        had_stdin_file: false,
    };
    init_params(&raw mut params, argc, argv);
    init_startuptime(&raw mut params);
    let mut i: c_int = 1 as c_int;
    while i < params.argc {
        if strcasecmp(
            *params.argv.offset(i as isize),
            b"--clean\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
        {
            params.clean = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    event_init();
    early_init(&raw mut params);
    set_argv_var(argv, argc);
    check_and_set_isatty(&raw mut params);
    command_line_scan(&raw mut params);
    set_argf_var();
    nlua_init(argv, argc, params.lua_arg0);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init lua interpreter\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if embedded_mode.get() {
        let mut err: *const c_char = ::core::ptr::null::<c_char>();
        if channel_from_stdio(
            true_0 != 0,
            CallbackReader {
                cb: Callback {
                    data: Callback_data {
                        funcref: ::core::ptr::null_mut::<c_char>(),
                    },
                    type_0: kCallbackNone,
                },
                self_0: ::core::ptr::null_mut::<dict_T>(),
                buffer: GA_EMPTY_INIT_VALUE,
                eof: false,
                buffered: false_0 != 0,
                fwd_err: false_0 != 0,
                type_0: ::core::ptr::null::<c_char>(),
            },
            &raw mut err,
        ) == 0
        {
            abort();
        }
    }
    if (*global_alist.ptr()).al_ga.ga_len > 0 as c_int {
        fname = get_fname(&raw mut params);
    }
    if recoverymode.get() as c_int != 0 && fname.is_null() {
        headless_mode.set(true_0 != 0);
    }
    let mut has_term: bool = stdin_isatty.get() as c_int != 0
        || stdout_isatty.get() as c_int != 0
        || stderr_isatty.get() as c_int != 0;
    let mut use_builtin_ui: bool = has_term as c_int != 0
        && !headless_mode.get()
        && !embedded_mode.get()
        && !silent_mode.get();
    if params.remote != 0 {
        remote_request(
            &raw mut params,
            params.remote,
            params.server_addr,
            argc,
            argv,
            use_builtin_ui,
        );
    }
    let mut remote_ui: bool = ui_client_channel_id.get() != 0 as uint64_t;
    if use_builtin_ui as c_int != 0 && !remote_ui {
        ui_client_forward_stdin.set(!stdin_isatty.get());
        let mut rv: uint64_t = ui_client_start_server(
            get_vim_var_str(VV_PROGPATH),
            params.argc as size_t,
            params.argv,
        );
        if rv == 0 {
            fprintf(
                stderr,
                b"Failed to start Nvim server!\n\0".as_ptr() as *const c_char,
            );
            os_exit(1 as c_int);
        }
        ui_client_channel_id.set(rv);
    }
    if ui_client_channel_id.get() != 0 {
        ui_client_run();
    }
    '_c2rust_label: {
        if ui_client_channel_id.get() == 0 && !use_builtin_ui {
        } else {
            __assert_fail(
                b"!ui_client_channel_id && !use_builtin_ui\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                369 as c_uint,
                b"int main(int, char **)\0".as_ptr() as *const c_char,
            );
        }
    };
    if !server_init(params.listen_addr) {
        mainerr(
            IObuff.ptr() as *mut c_char,
            ::core::ptr::null::<c_char>(),
            ::core::ptr::null::<c_char>(),
        );
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"expanding arguments\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.diff_mode != 0 && params.window_count == -1 as c_int {
        params.window_count = 0 as c_int;
    }
    (*RedrawingDisabled.ptr()) += 1;
    setbuf(stdout, ::core::ptr::null_mut::<c_char>());
    full_screen.set(!silent_mode.get());
    win_init_size();
    if params.diff_mode != 0 {
        diff_win_options(firstwin.get(), false_0 != 0);
    }
    '_c2rust_label_0: {
        if p_ch.get() >= 0 as OptInt
            && Rows.get() as OptInt >= p_ch.get()
            && Rows.get() as OptInt - p_ch.get() <= 2147483647 as OptInt
        {
        } else {
            __assert_fail(
                b"p_ch >= 0 && Rows >= p_ch && Rows - p_ch <= INT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                414 as c_uint,
                b"int main(int, char **)\0".as_ptr() as *const c_char,
            );
        }
    };
    cmdline_row.set(Rows.get() - p_ch.get() as c_int);
    msg_row.set(cmdline_row.get());
    default_grid_alloc();
    set_init_2(headless_mode.get());
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 2\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    msg_scroll.set(true_0);
    no_wait_return.set(true_0);
    init_highlight(true_0 != 0, false_0 != 0);
    ui_comp_syn_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init highlight\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    debug_break_level.set(params.use_debug_break_level);
    if !stdin_isatty.get()
        && !params.input_istext
        && silent_mode.get() as c_int != 0
        && exmode_active.get() as c_int != 0
    {
        input_start();
    }
    let mut use_remote_ui: bool = embedded_mode.get() as c_int != 0 && !headless_mode.get();
    if use_remote_ui {
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"waiting for UI\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        remote_ui_wait_for_attach();
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"done waiting for UI\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        (*firstwin.get()).w_prev_height = (*firstwin.get()).w_height;
    }
    starting.set(NO_BUFFERS);
    screenclear();
    win_new_screensize();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"clear screen\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if edit_stdin(&raw mut params) {
        params.edit_type = EDIT_STDIN as c_int;
    }
    if !params.scriptin.is_null() {
        if !open_scriptin(params.scriptin) {
            os_exit(2 as c_int);
        }
    }
    if !params.scriptout.is_null() {
        scriptout.set(os_fopen(
            params.scriptout,
            if params.scriptout_append as c_int != 0 {
                APPENDBIN.as_ptr()
            } else {
                WRITEBIN.as_ptr()
            },
        ));
        if (*scriptout.ptr()).is_null() {
            fprintf(
                stderr,
                gettext(b"Cannot open for script output: \"\0".as_ptr() as *const c_char),
            );
            fprintf(
                stderr,
                b"%s\"\n\0".as_ptr() as *const c_char,
                params.scriptout,
            );
            os_exit(2 as c_int);
        }
    }
    nlua_init_defaults();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init default mappings & autocommands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    let mut vimrc_none: bool = strequal(params.use_vimrc, b"NONE\0".as_ptr() as *const c_char);
    p_lpl.set(if vimrc_none as c_int != 0 {
        params.clean as c_int
    } else {
        p_lpl.get()
    });
    exe_pre_commands(&raw mut params);
    if !vimrc_none || params.clean as c_int != 0 {
        filetype_plugin_enable();
    }
    source_startup_scripts(&raw mut params);
    if !vimrc_none || params.clean as c_int != 0 {
        filetype_maybe_enable();
        syn_maybe_enable();
    }
    set_vim_var_nr(VV_VIM_DID_INIT, 1 as varnumber_T);
    load_plugins();
    set_window_layout(&raw mut params);
    if recoverymode.get() as c_int != 0 && fname.is_null() {
        recover_names(
            ::core::ptr::null_mut::<c_char>(),
            true_0 != 0,
            ::core::ptr::null_mut::<list_T>(),
            0 as c_int,
            ::core::ptr::null_mut::<*mut c_char>(),
        );
        os_exit(0 as c_int);
    }
    set_init_3();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 3\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.no_swap_file != 0 {
        p_uc.set(0 as OptInt);
    }
    if silent_mode.get() {
        p_ut.set(1 as OptInt);
    }
    if *p_shada.get() as c_int != NUL {
        shada_read_everything(::core::ptr::null::<c_char>(), false_0 != 0, true_0 != 0);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"reading ShaDa\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    if get_vim_var_list(VV_OLDFILES).is_null() {
        set_vim_var_list(VV_OLDFILES, tv_list_alloc(0 as ptrdiff_t));
    }
    handle_quickfix(&raw mut params);
    starting.set(NO_BUFFERS);
    no_wait_return.set(false_0);
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    if params.edit_type == EDIT_STDIN as c_int && !recoverymode.get() {
        read_stdin();
    }
    setmouse();
    redraw_later(curwin.get(), UPD_VALID as c_int);
    no_wait_return.set(true_0);
    create_windows(&raw mut params);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"opening buffers\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_vim_var_string(
        VV_SWAPCOMMAND,
        ::core::ptr::null::<c_char>(),
        -1 as ptrdiff_t,
    );
    if exmode_active.get() {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
    }
    apply_autocmds(
        EVENT_BUFENTER,
        ::core::ptr::null_mut::<c_char>(),
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"BufEnter autocommands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    setpcmark();
    if params.edit_type == EDIT_QF as c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as c_int,
            0 as c_int,
            false_0,
        );
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"jump to first error\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    edit_buffers(&raw mut params);
    if params.diff_mode != 0 {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_arg_idx_invalid == 0 {
                diff_win_options(wp, true_0 != 0);
            }
            wp = (*wp).w_next;
        }
    }
    shorten_fnames(false_0);
    handle_tag(params.tagname);
    if params.n_commands > 0 as c_int {
        exe_commands(&raw mut params);
    }
    starting.set(0 as c_int);
    RedrawingDisabled.set(0 as c_int);
    redraw_all_later(UPD_NOT_VALID as c_int);
    no_wait_return.set(false_0);
    do_autochdir();
    set_vim_var_nr(VV_VIM_DID_ENTER, 1 as varnumber_T);
    apply_autocmds(
        EVENT_VIMENTER,
        ::core::ptr::null_mut::<c_char>(),
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"VimEnter autocommands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if use_remote_ui {
        do_autocmd_uienter_all();
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"UIEnter autocommands\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    set_reg_var(get_default_register_name());
    if (*curwin.get()).w_onebuf_opt.wo_diff != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        update_topline(curwin.get());
        check_scrollbind(0 as linenr_T, 0 as c_int);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"diff scrollbinding\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    if restart_edit.get() != 0 as c_int {
        stuffcharReadbuff(-(253 as c_int + ((KE_NOP as c_int) << 8 as c_int)));
    }
    if cb_flags.get() & (kOptCbFlagUnnamed as c_int | kOptCbFlagUnnamedplus as c_int) as c_uint != 0
    {
        eval_has_provider(b"clipboard\0".as_ptr() as *const c_char, false_0 != 0);
    }
    if !params.luaf.is_null() {
        msg_scroll.set(true_0);
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<c_char>(),
            b"main\0".as_ptr() as *const c_char,
            678 as c_int,
            true_0 != 0,
            b"executing Lua -l script\0".as_ptr() as *const c_char,
        );
        let mut lua_ok: bool = nlua_exec_file(params.luaf);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"executing Lua -l script\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if msg_didout.get() {
            msg_putchar('\n' as c_int);
            msg_didout.set(false_0 != 0);
        }
        getout(if lua_ok as c_int != 0 {
            0 as c_int
        } else {
            1 as c_int
        });
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"before starting main loop\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<c_char>(),
        b"main\0".as_ptr() as *const c_char,
        689 as c_int,
        true_0 != 0,
        b"starting main loop\0".as_ptr() as *const c_char,
    );
    normal_enter(false_0 != 0, false_0 != 0);
    return 0 as c_int;
}

pub fn main() {
    let mut args_strings: Vec<Vec<u8>> = ::std::env::args()
        .map(|arg| {
            ::std::ffi::CString::new(arg)
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    let mut args_ptrs: Vec<*mut c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut c_char)
        .chain(::core::iter::once(::core::ptr::null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(main_0(
            (args_ptrs.len() - 1) as c_int,
            args_ptrs.as_mut_ptr() as *mut *mut c_char,
        ) as i32)
    }
}
