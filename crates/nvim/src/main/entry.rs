//! Process entry, and the order the startup does things in.
//!
//! [`main_0`] is the sequence itself. It reads as a list of phases because
//! that is what it is, and the ordering between them is load-bearing:
//! options before buffers, buffers before windows, windows before the UI,
//! the UI before `VimEnter`. The `time_msg_at` calls between the phases are
//! the seams `--startuptime` reports.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

use crate::api::ui::remote_ui_wait_for_attach;
use crate::arglist::alist_init;
use crate::arglist::global_arglist;
use crate::autocmd::{EVENT_BUFENTER, EVENT_VIMENTER, apply_autocmds, autocmd_init};
use crate::buffer::do_autochdir;
use crate::channel::{channel_from_stdio, channel_init, channel_teardown};
use crate::diff::diff_win_options;
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, default_grid_alloc, redraw_all_later, redraw_later, screenclear,
};
use crate::eval::typval::{kCallbackNone, tv_list_alloc};
use crate::eval::vars::{
    get_vim_var_list, get_vim_var_str, set_reg_var, set_vim_var_list, set_vim_var_nr,
    set_vim_var_string,
};
use crate::eval::{eval_has_provider, eval_init, set_argv_var, timer_teardown};
use crate::event::r#loop::{loop_close, loop_init, loop_poll_events};
use crate::event::multiqueue::{multiqueue_new_child, multiqueue_process_events};
use crate::event::proc::proc_teardown;
use crate::ex_docmd::{filetype_maybe_enable, filetype_plugin_enable};
use crate::ex_getln::cmdline_init;
use crate::fileio::shorten_fnames;
use crate::getchar::{open_scriptin, stuff_readbuf_char};
use crate::highlight::highlight_init;
use crate::highlight_group::init_highlight;
use crate::keycodes::KE_NOP;
use crate::log::{LOGLVL_DBG, LOGLVL_INF, log_init, logmsg_c};
use crate::lua::executor::{nlua_exec_file, nlua_init, nlua_init_defaults, nlua_run_script};
use crate::main::args::{
    check_and_set_isatty, command_line_scan, edit_stdin, init_params, init_path, init_startuptime,
    set_window_layout,
};
use crate::main::buffers::{
    create_windows, edit_buffers, get_fname, handle_quickfix, handle_tag, read_stdin, set_argf_var,
};
use crate::main::config::{exe_commands, exe_pre_commands, source_startup_scripts};
use crate::main::exit::{getout, os_exit};
use crate::main::remote::remote_request;
use crate::main::usage::{mainerr, print_mainerr};
use crate::main::{
    APPENDBIN, EDIT_QF, EDIT_STDIN, GA_EMPTY_INIT_VALUE, NO_BUFFERS, RedrawingDisabled, Rows,
    WRITEBIN, argv0, cb_flags, cmdline_row, curbuf, curwin, debug_break_level, embedded_mode,
    err_arg_missing, exmode_active, firstwin, full_screen, headless_mode, kOptCbFlagUnnamed,
    kOptCbFlagUnnamedplus, main_loop, mparm_T, msg_didout, msg_row, msg_scroll, no_wait_return,
    p_ch, p_lpl, p_shada, p_uc, p_ut, recoverymode, resize_events, restart_edit, scriptout,
    silent_mode, starting, stderr_isatty, stdin_isatty, stdout_isatty, time_msg_at,
    ui_client_channel_id, ui_client_forward_stdin,
};
use crate::mark::setpcmark;
use crate::memline::recover_names;
use crate::memory::strequal;
use crate::message::msg_putchar;
use crate::mouse::setmouse;
use crate::r#move::update_topline;
use crate::msgpack_rpc::server::{server_init, server_teardown};
use crate::normal::{check_scrollbind, normal_enter};
use crate::option::{set_init_1, set_init_2, set_init_3, set_init_tablocal};
use crate::os::cshim::{gettext, stderr, stdout};
use crate::os::env::{env_init, init_homedir, os_hint_priority};
use crate::os::fs::os_fopen;
use crate::os::input::{input_start, input_stop};
use crate::os::lang::{init_locale, set_lang_var};
use crate::os::signal::{signal_init, signal_teardown};
use crate::os::stdpaths::appname_is_valid;
use crate::os::time::os_realtime;
use crate::quickfix::{qf_init_stack, qf_jump};
use crate::register::get_default_register_name;
use crate::runtime::{estack_init, load_plugins, runtime_init};
use crate::shada::shada_read_everything;
use crate::syntax::syn_maybe_enable;
use crate::terminal::{terminal_init, terminal_teardown};
use crate::types::{
    Callback, Callback_data, CallbackReader, IOSIZE, NUL, OptInt, Vv, dict_T, int64_t, linenr_T,
    list_T, qf_info_T, varnumber_T,
};
use crate::ui::{do_autocmd_uienter_all, ui_init};
use crate::ui_client::{ui_client_run, ui_client_start_server};
use crate::ui_compositor::ui_comp_syn_init;
use crate::window::{win_alloc_first, win_init_size, win_new_screensize};
use crate::winlayer::{Win, windows};
use ::libc::{abort, exit, fprintf, setbuf, strcasecmp};

/// Bring up the event loop and everything that hangs off it.
///
/// Exported: the unit tests build an editor without a `main`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn event_init() {
    // SAFETY: initialises the singleton main loop and its subsystems, once.
    unsafe {
        loop_init(main_loop.ptr());
        env_init();
        resize_events.set(multiqueue_new_child((*main_loop.ptr()).events));
        autocmd_init();
        signal_init();
        channel_init();
        terminal_init();
        ui_init();
        time_msg_at(c"event init");
    }
}

/// Take the event loop back down, in the reverse order.
///
/// Answers whether it came down cleanly; [`os_exit`] turns a `false` into a
/// non-zero exit status.
pub(crate) unsafe fn event_teardown() -> bool {
    // SAFETY: shuts down the singleton main loop and its subsystems.
    unsafe {
        if (*main_loop.ptr()).events.is_null() {
            // Never came up; there is nothing to drain.
            input_stop();
            return true;
        }
        // Drain what is already queued before pulling the loop out.
        multiqueue_process_events((*main_loop.ptr()).events);
        loop_poll_events(main_loop.ptr(), 0 as int64_t);
        input_stop();
        server_teardown();
        channel_teardown();
        proc_teardown(main_loop.ptr());
        timer_teardown();
        signal_teardown();
        terminal_teardown();
        loop_close(main_loop.ptr(), true)
    }
}

/// The initialisation that has to happen before the command line is even
/// looked at: the option defaults, the first window, the runtime paths.
///
/// Exported: the unit tests build an editor without a `main`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn early_init(paramp: *mut mparm_T) {
    // SAFETY: `paramp` is null when the unit tests call this; every use of
    // it below is guarded.
    unsafe {
        os_hint_priority();
        estack_init();
        cmdline_init();
        eval_init();
        set_vim_var_nr(Vv::Starttime, os_realtime());

        init_path(if !argv0.get().is_null() {
            argv0.get() as *const c_char
        } else {
            c"nvim".as_ptr()
        });
        runtime_init();
        highlight_init();
        time_msg_at(c"early init");

        init_locale();
        set_init_tablocal();
        win_alloc_first();
        time_msg_at(c"init first window");

        alist_init(global_arglist());
        (*global_arglist()).id = 0;
        init_homedir();
        set_init_1(!paramp.is_null() && (*paramp).clean);
        log_init();
        time_msg_at(c"inits 1");

        set_lang_var();
        qf_init_stack();
    }
}

/// The startup, in order.
///
/// Never returns: it ends in `normal_enter`, which is the editor's main
/// loop, or in one of the exits along the way.
pub(crate) unsafe fn main_0(argc: c_int, argv: *mut *mut c_char) -> c_int {
    // Why `server_init` gave up, when it does. Upstream leaves it in the
    // shared `IObuff`.
    let mut reason = [0 as c_char; IOSIZE as usize];
    // SAFETY: `argv[0..argc]` are the process arguments and live for the
    // whole process; `params` lives for the whole of this function, which
    // never returns while anything still holds a pointer into it.
    unsafe {
        argv0.set(*argv);
        if !appname_is_valid() {
            fprintf(
                stderr,
                c"$NVIM_APPNAME must be a name or relative path.\n".as_ptr(),
            );
            exit(1);
        }

        // `-ll` is handled before anything else: it is a bare Lua
        // interpreter with none of the editor behind it.
        if argc > 1 && strcasecmp(*argv.offset(1), c"-ll".as_ptr()) == 0 {
            if argc == 2 {
                print_mainerr(err_arg_missing.get(), *argv.offset(1), ptr::null());
                exit(1);
            }
            nlua_run_script(argv, argc, 3);
        }

        // All-zero is the "nothing given" state for everything except the
        // handful of fields `init_params` sets.
        let mut params: mparm_T = core::mem::zeroed();
        init_params(&raw mut params, argc, argv);
        init_startuptime(&raw mut params);

        // `--clean` has to be known before `early_init` reads any config.
        for i in 1..params.argc {
            if strcasecmp(*params.argv.offset(i as isize), c"--clean".as_ptr()) == 0 {
                params.clean = true;
                break;
            }
        }

        event_init();
        early_init(&raw mut params);
        set_argv_var(argv, argc);
        check_and_set_isatty(&raw mut params);
        command_line_scan(&raw mut params);
        set_argf_var();

        nlua_init(argv, argc, params.lua_arg0);
        time_msg_at(c"init lua interpreter");

        if embedded_mode.get() {
            // stdin/stdout become the RPC channel.
            let mut err: *const c_char = ptr::null();
            let reader = CallbackReader {
                cb: Callback {
                    data: Callback_data {
                        funcref: ptr::null_mut(),
                    },
                    type_0: kCallbackNone,
                },
                self_0: ptr::null_mut::<dict_T>(),
                buffer: GA_EMPTY_INIT_VALUE,
                eof: false,
                buffered: false,
                fwd_err: false,
                type_0: ptr::null(),
            };
            if channel_from_stdio(true, reader, &raw mut err) == 0 {
                abort();
            }
        }

        let mut fname: *mut c_char = ptr::null_mut();
        if (*global_arglist()).al_ga.ga_len > 0 {
            fname = get_fname(&raw mut params);
        }
        if recoverymode.get() && fname.is_null() {
            // `-r` with no file only lists the swap files.
            headless_mode.set(true);
        }

        let has_term = stdin_isatty.get() || stdout_isatty.get() || stderr_isatty.get();
        let use_builtin_ui =
            has_term && !headless_mode.get() && !embedded_mode.get() && !silent_mode.get();

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

        // A `--remote-ui` handled above already has a channel; otherwise the
        // built-in UI starts a server of its own and re-executes into it.
        let remote_ui = ui_client_channel_id.get() != 0;
        if use_builtin_ui && !remote_ui {
            ui_client_forward_stdin.set(!stdin_isatty.get());
            let chan = ui_client_start_server(
                get_vim_var_str(Vv::Progpath),
                params.argc as usize,
                params.argv,
            );
            if chan == 0 {
                fprintf(stderr, c"Failed to start Nvim server!\n".as_ptr());
                os_exit(1);
            }
            ui_client_channel_id.set(chan);
        }
        if ui_client_channel_id.get() != 0 {
            // This process is now a UI, not an editor; it does not return.
            ui_client_run();
        }
        debug_assert!(
            ui_client_channel_id.get() == 0 && !use_builtin_ui,
            "a UI client reached the editor startup"
        );

        if !server_init(params.listen_addr, &mut reason) {
            mainerr(reason.as_mut_ptr(), ptr::null(), ptr::null());
        }
        time_msg_at(c"expanding arguments");

        if params.diff_mode != 0 && params.window_count == -1 {
            // Diff mode wants one window per file.
            params.window_count = 0;
        }

        RedrawingDisabled.set(RedrawingDisabled.get() + 1);
        setbuf(stdout, ptr::null_mut());
        full_screen.set(!silent_mode.get());

        win_init_size();
        if params.diff_mode != 0 {
            diff_win_options(Win::new(firstwin.get()), false);
        }

        debug_assert!(
            p_ch.get() >= 0
                && Rows.get() as OptInt >= p_ch.get()
                && Rows.get() as OptInt - p_ch.get() <= c_int::MAX as OptInt,
            "'cmdheight' does not fit in the screen"
        );
        cmdline_row.set(Rows.get() - p_ch.get() as c_int);
        msg_row.set(cmdline_row.get());
        default_grid_alloc();

        set_init_2(headless_mode.get());
        time_msg_at(c"inits 2");

        msg_scroll.set(1);
        no_wait_return.set(1);
        init_highlight(true, false);
        ui_comp_syn_init();
        time_msg_at(c"init highlight");

        debug_break_level.set(params.use_debug_break_level);

        // `-es` with a pipe: start reading it now, so the Ex commands are
        // there when the loop asks.
        if !stdin_isatty.get() && !params.input_istext && silent_mode.get() && exmode_active.get() {
            input_start();
        }

        // `--embed` without `--headless`: the client attaches a UI, and the
        // startup waits for it so the first redraw has somewhere to go.
        let use_remote_ui = embedded_mode.get() && !headless_mode.get();
        if use_remote_ui {
            time_msg_at(c"waiting for UI");
            remote_ui_wait_for_attach();
            time_msg_at(c"done waiting for UI");
            (*firstwin.get()).w_prev_height = (*firstwin.get()).w_height;
        }

        starting.set(NO_BUFFERS);
        screenclear();
        win_new_screensize();
        time_msg_at(c"clear screen");

        if edit_stdin(&raw mut params) {
            params.edit_type = EDIT_STDIN as c_int;
        }
        if !params.scriptin.is_null() && !open_scriptin(params.scriptin) {
            os_exit(2);
        }
        if !params.scriptout.is_null() {
            scriptout.set(os_fopen(
                params.scriptout,
                if params.scriptout_append {
                    APPENDBIN.as_ptr()
                } else {
                    WRITEBIN.as_ptr()
                },
            ));
            if scriptout.get().is_null() {
                fprintf(
                    stderr,
                    gettext(c"Cannot open for script output: \"".as_ptr()),
                );
                fprintf(stderr, c"%s\"\n".as_ptr(), params.scriptout);
                os_exit(2);
            }
        }

        nlua_init_defaults();
        time_msg_at(c"init default mappings & autocommands");

        // `-u NONE` also turns the plugins off, unless `--clean` asked for
        // the defaults.
        let vimrc_none = strequal(params.use_vimrc, c"NONE".as_ptr());
        if vimrc_none {
            p_lpl.set(params.clean as c_int);
        }

        exe_pre_commands(&raw mut params);
        if !vimrc_none || params.clean {
            filetype_plugin_enable();
        }
        source_startup_scripts(&raw mut params);
        if !vimrc_none || params.clean {
            filetype_maybe_enable();
            syn_maybe_enable();
        }

        set_vim_var_nr(Vv::VimDidInit, 1 as varnumber_T);
        load_plugins();
        set_window_layout(&raw mut params);

        if recoverymode.get() && fname.is_null() {
            // `-r` with no file: list the swap files and leave.
            recover_names(
                ptr::null_mut(),
                true,
                ptr::null_mut::<list_T>(),
                0,
                ptr::null_mut(),
            );
            os_exit(0);
        }

        set_init_3();
        time_msg_at(c"inits 3");

        if params.no_swap_file != 0 {
            p_uc.set(0 as OptInt);
        }
        if silent_mode.get() {
            p_ut.set(1 as OptInt);
        }

        if *p_shada.get() as c_int != NUL {
            shada_read_everything(ptr::null(), false, true);
            time_msg_at(c"reading ShaDa");
        }
        if get_vim_var_list(Vv::Oldfiles).is_null() {
            set_vim_var_list(Vv::Oldfiles, tv_list_alloc(0));
        }

        handle_quickfix(&raw mut params);

        starting.set(NO_BUFFERS);
        no_wait_return.set(0);
        if !exmode_active.get() {
            msg_scroll.set(0);
        }

        if params.edit_type == EDIT_STDIN as c_int && !recoverymode.get() {
            read_stdin();
        }

        setmouse();
        redraw_later(curwin.get(), UPD_VALID);
        no_wait_return.set(1);

        create_windows(&raw mut params);
        time_msg_at(c"opening buffers");

        // The swap command has served its purpose; the ATTENTION prompts
        // from here on are the user's own doing.
        set_vim_var_string(Vv::Swapcommand, ptr::null(), -1);

        if exmode_active.get() {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        }
        apply_autocmds(
            EVENT_BUFENTER,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        time_msg_at(c"BufEnter autocommands");
        setpcmark();

        if params.edit_type == EDIT_QF as c_int {
            qf_jump(ptr::null_mut::<qf_info_T>(), 0, 0, 0);
            time_msg_at(c"jump to first error");
        }

        edit_buffers(&raw mut params);

        if params.diff_mode != 0 {
            for wp in windows() {
                if !wp.w_arg_idx_invalid {
                    diff_win_options(wp, true);
                }
            }
        }

        shorten_fnames(0);
        handle_tag(params.tagname);
        if params.n_commands > 0 {
            exe_commands(&raw mut params);
        }

        starting.set(0);
        RedrawingDisabled.set(0);
        redraw_all_later(UPD_NOT_VALID);
        no_wait_return.set(0);
        do_autochdir();

        set_vim_var_nr(Vv::VimDidEnter, 1 as varnumber_T);
        apply_autocmds(
            EVENT_VIMENTER,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        time_msg_at(c"VimEnter autocommands");
        if use_remote_ui {
            do_autocmd_uienter_all();
            time_msg_at(c"UIEnter autocommands");
        }

        set_reg_var(get_default_register_name());

        if (*curwin.get()).w_onebuf_opt.wo_diff != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
            update_topline(Win::current());
            check_scrollbind(0 as linenr_T, 0);
            time_msg_at(c"diff scrollbinding");
        }

        if restart_edit.get() != 0 {
            // A `-c startinsert` and friends: push the key that gets there.
            stuff_readbuf_char(-(253 + ((KE_NOP as c_int) << 8)));
        }

        if cb_flags.get() & (kOptCbFlagUnnamed as c_int | kOptCbFlagUnnamedplus as c_int) as c_uint
            != 0
        {
            // Warm the clipboard provider so the first yank is not slow.
            eval_has_provider(c"clipboard".as_ptr(), false);
        }

        if !params.luaf.is_null() {
            // `-l`: run the script and leave, with its status.
            msg_scroll.set(1);
            logmsg_c!(
                LOGLVL_DBG,
                ptr::null(),
                c"main".as_ptr(),
                678,
                true,
                c"executing Lua -l script".as_ptr(),
            );
            let lua_ok = nlua_exec_file(params.luaf);
            time_msg_at(c"executing Lua -l script");
            if msg_didout.get() {
                msg_putchar('\n' as c_int);
                msg_didout.set(false);
            }
            getout(if lua_ok { 0 } else { 1 });
        }

        time_msg_at(c"before starting main loop");
        logmsg_c!(
            LOGLVL_INF,
            ptr::null(),
            c"main".as_ptr(),
            689,
            true,
            c"starting main loop".as_ptr(),
        );

        // Never returns.
        normal_enter(false, false);
        0
    }
}

/// The process entry point.
///
/// Turns Rust's `args()` into the `argc`/`argv` pair the startup expects and
/// hands over to [`main_0`], which does not return.
pub fn main() {
    let mut args: Vec<Vec<u8>> = ::std::env::args()
        .map(|arg| {
            ::std::ffi::CString::new(arg)
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    // argv is NUL-terminated, as C requires; argc does not count that entry.
    let mut argv: Vec<*mut c_char> = args
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut c_char)
        .chain(::core::iter::once(ptr::null_mut()))
        .collect();

    // SAFETY: `args` outlives the call, and `main_0` never returns.
    unsafe { ::std::process::exit(main_0((argv.len() - 1) as c_int, argv.as_mut_ptr()) as i32) }
}
