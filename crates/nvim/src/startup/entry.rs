//! Process entry, and the order the startup does things in.
//!
//! [`run`] is the sequence itself. It reads as a list of phases because
//! that is what it is, and the ordering between them is load-bearing:
//! options before buffers, buffers before windows, windows before the UI,
//! the UI before `VimEnter`. The `time_msg_at` calls between the phases are
//! the seams `--startuptime` reports.

#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`early_init`, `event_init`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]

use crate::event::multiqueue::ChildQueue;
use crate::memory::ThinCString;
use crate::types::ArgListId;
use crate::types::AutoEvent;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::CString;

use crate::api::ui::remote_ui_wait_for_attach;
use crate::arglist::alist_count;
use crate::autocmd::{autocmd_init, fire_autocmds};
use crate::buffer::do_autochdir;
use crate::channel::{channel_from_stdio, channel_init, channel_teardown};
use crate::debugger::state::debug_break_level;
use crate::diff::diff_win_options;
use crate::drawscreen::state::{RedrawingDisabled, cmdline_row};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, default_grid_alloc, redraw_all_later, redraw_later, screenclear,
};
use crate::eval::typval::tv_list_alloc;
use crate::eval::vars::{
    get_vim_var_list_handle, set_reg_var, set_vim_var_list, set_vim_var_nr, set_vim_var_string,
    vim_var_string,
};
use crate::eval::{eval_has_provider, eval_init, set_argv_var, timer_teardown};
use crate::event::r#loop::{main_loop_close, main_loop_drain, main_loop_init, main_loop_is_up};
use crate::event::proc::proc_teardown;
use crate::ex_docmd::{filetype_maybe_enable, filetype_plugin_enable};
use crate::ex_getln::cmdline_init;
use crate::fileio::shorten_fnames;
use crate::getchar::state::scriptout;
use crate::getchar::{open_scriptin, stuff_readbuf_char};
use crate::highlight::highlight_init;
use crate::highlight_group::init_highlight;
use crate::keycodes::KE_NOP;
use crate::log::{LOGLVL_DBG, LOGLVL_INF, log_init, logmsg};
use crate::lua::executor::{nlua_exec_file, nlua_init, nlua_init_defaults, nlua_run_script};
use crate::mark::setpcmark;
use crate::memline::list_swap_files;
use crate::message::msg_putchar;
use crate::message::state::{msg_didout, msg_row, msg_scroll, no_wait_return};
use crate::mouse::setmouse;
use crate::r#move::update_topline;
use crate::msgpack_rpc::server::{server_init, server_teardown};
use crate::normal::{check_scrollbind, normal_enter};
use crate::option::vars::{P_LPL, P_SHADA, P_UC, P_UT, cb_flags, p_ch};
use crate::option::{set_init_1, set_init_2, set_init_3, set_init_tablocal};
use crate::os::cshim::{gettext, unbuffer_c_stdout, write_c_stderr};
use crate::os::env::{env_init, init_homedir, os_hint_priority};
use crate::os::fs::CFile;
use crate::os::input::{input_start, input_stop};
use crate::os::lang::{init_locale, set_lang_var};
use crate::os::signal::{signal_init, signal_teardown};
use crate::os::stdpaths::appname_is_valid;
use crate::os::time::os_realtime;
use crate::profile::time_msg_at;
use crate::quickfix::{qf_init_stack, qf_jump_to_current};
use crate::register::get_default_register_name;
use crate::runtime::{estack_init, load_plugins};
use crate::shada::shada_read_everything;
use crate::startup::args::{
    check_and_set_isatty, command_line_scan, edit_stdin, init_path, init_startuptime,
    set_window_layout,
};
use crate::startup::buffers::{
    create_windows, edit_buffers, handle_quickfix, handle_tag, read_stdin, set_argf_var,
};
use crate::startup::config::{exe_commands, exe_pre_commands, source_startup_scripts};
use crate::startup::exit::{getout, os_exit};
use crate::startup::remote::remote_request;
use crate::startup::usage::{mainerr, print_mainerr};
use crate::startup::{
    APPENDBIN, EDIT_QF, EDIT_STDIN, MainParams, NO_BUFFERS, WRITEBIN, argv0, embedded_mode,
    err_arg_missing, full_screen, headless_mode, kOptCbFlagUnnamed, kOptCbFlagUnnamedplus,
    recoverymode, silent_mode, starting, stderr_isatty, stdin_isatty, stdout_isatty,
    ui_client_channel_id, ui_client_forward_stdin,
};
use crate::state::mode::{exmode_active, restart_edit};
use crate::syntax::syn_maybe_enable;
use crate::terminal::{terminal_init, terminal_teardown};
use crate::types::{CallbackReader, LineNr, OptInt, VarNumber, Vv};
use crate::ui::state::{Rows, resize_events};
use crate::ui::{do_autocmd_uienter_all, ui_init};
use crate::ui_client::{ui_client_run, ui_client_start_server};
use crate::ui_compositor::ui_comp_syn_init;
use crate::window::{win_alloc_first, win_init_size, win_new_screensize};
use crate::winlayer::{Win, first_window, windows};

/// Bring up the event loop and everything that hangs off it.
///
/// Exported: the unit tests build an editor without a `main`.
#[unsafe(no_mangle)]
pub extern "C" fn event_init() {
    main_loop_init();
    env_init();
    resize_events.set(Some(ChildQueue::of_main_loop()));
    autocmd_init();
    signal_init();
    channel_init();
    terminal_init();
    ui_init();
    time_msg_at(c"event init");
}

/// Take the event loop back down, in the reverse order.
///
/// Answers whether it came down cleanly; [`os_exit`] turns a `false` into a
/// non-zero exit status.
pub(crate) fn event_teardown() -> bool {
    if !main_loop_is_up() {
        // Never came up; there is nothing to drain.
        input_stop();
        return true;
    }
    // Drain what is already queued before pulling the loop out.
    main_loop_drain();
    input_stop();
    server_teardown();
    channel_teardown();
    proc_teardown();
    timer_teardown();
    signal_teardown();
    terminal_teardown();
    main_loop_close(true)
}

/// The initialisation that has to happen before the command line is even
/// looked at: the option defaults, the first window, the runtime paths.
/// `clean` is `--clean`, which has to be known before any config is read.
///
/// Exported: the unit tests build an editor without a `main`.
#[unsafe(no_mangle)]
pub extern "C" fn early_init(clean: bool) {
    os_hint_priority();
    estack_init();
    cmdline_init();
    eval_init();
    set_vim_var_nr(Vv::Starttime, os_realtime());

    init_path(argv0.get().unwrap_or(c"nvim"));
    highlight_init();
    time_msg_at(c"early init");

    init_locale();
    set_init_tablocal();
    win_alloc_first();
    time_msg_at(c"init first window");

    init_homedir();
    set_init_1(clean);
    log_init();
    time_msg_at(c"inits 1");

    set_lang_var();
    qf_init_stack();
}

/// The startup, in order, over the process's command line.
///
/// Never returns: it ends in `normal_enter`, which is the editor's main
/// loop, or in one of the exits along the way.
pub(crate) fn run(argv: Vec<CString>) -> c_int {
    // `argv[0]` is asked for by the error messages until the process ends.
    let name = argv
        .first()
        .map(|name| &*Box::leak(name.clone().into_boxed_c_str()));
    argv0.set(name);
    if !appname_is_valid() {
        write_c_stderr(b"$NVIM_APPNAME must be a name or relative path.\n");
        std::process::exit(1);
    }

    // `-ll` is handled before anything else: it is a bare Lua
    // interpreter with none of the editor behind it.
    if argv
        .get(1)
        .is_some_and(|arg| arg.to_bytes().eq_ignore_ascii_case(b"-ll"))
    {
        if argv.len() == 2 {
            print_mainerr(err_arg_missing, Some(&argv[1]), None);
            std::process::exit(1);
        }
        nlua_run_script(&argv, 3);
    }

    let mut params = MainParams::new(argv);
    init_startuptime(&params);

    // `--clean` has to be known before `early_init` reads any config.
    let clean = |arg: &CString| arg.to_bytes().eq_ignore_ascii_case(b"--clean");
    params.clean = params.argv.iter().skip(1).any(clean);

    event_init();
    early_init(params.clean);
    let args: Vec<&CStr> = params.argv.iter().map(CString::as_c_str).collect();
    set_argv_var(&args);
    check_and_set_isatty();
    command_line_scan(&mut params);
    set_argf_var();

    nlua_init(&params.argv, params.script_args_at);
    time_msg_at(c"init lua interpreter");

    if embedded_mode.get() && channel_from_stdio(true, CallbackReader::none()).is_err() {
        // stdin/stdout were to become the RPC channel.
        std::process::abort();
    }

    // The first file argument decides whether `-r` lists the swap files or
    // recovers one.
    let has_fname = alist_count(ArgListId::GLOBAL) > 0;
    if recoverymode.get() && !has_fname {
        // `-r` with no file only lists the swap files.
        headless_mode.set(true);
    }

    let has_term = stdin_isatty.get() || stdout_isatty.get() || stderr_isatty.get();
    let use_builtin_ui =
        has_term && !headless_mode.get() && !embedded_mode.get() && !silent_mode.get();

    if params.remote_at != 0 {
        let at = params.remote_at;
        remote_request(&mut params, at, use_builtin_ui);
    }

    // A `--remote-ui` handled above already has a channel; otherwise the
    // built-in UI starts a server of its own and re-executes into it.
    let remote_ui = ui_client_channel_id.get() != 0;
    if use_builtin_ui && !remote_ui {
        ui_client_forward_stdin.set(!stdin_isatty.get());
        let progpath = vim_var_string(Vv::Progpath).unwrap_or_else(ThinCString::empty);
        let chan = ui_client_start_server(progpath.as_cstr(), &params.argv);
        if chan == 0 {
            write_c_stderr(b"Failed to start Nvim server!\n");
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

    if let Err(reason) = server_init(params.listen_addr.as_deref()) {
        mainerr(&reason, None, None);
    }
    time_msg_at(c"expanding arguments");

    if params.diff_mode && params.window_count == -1 {
        // Diff mode wants one window per file.
        params.window_count = 0;
    }

    RedrawingDisabled.set(RedrawingDisabled.get() + 1);
    unbuffer_c_stdout();
    full_screen.set(!silent_mode.get());

    win_init_size();
    if params.diff_mode {
        diff_win_options(first_win(), false);
    }

    debug_assert!(
        p_ch() >= 0
            && Rows.get() as OptInt >= p_ch()
            && Rows.get() as OptInt - p_ch() <= c_int::MAX as OptInt,
        "'cmdheight' does not fit in the screen"
    );
    cmdline_row.set(Rows.get() - p_ch() as c_int);
    msg_row.set(cmdline_row.get());
    default_grid_alloc();

    set_init_2(headless_mode.get());
    time_msg_at(c"inits 2");

    msg_scroll.set(1);
    no_wait_return.set(1);
    init_highlight(true, false);
    ui_comp_syn_init();
    time_msg_at(c"init highlight");

    debug_break_level.set(params.debug_break_at);

    // `-es` with a pipe: start reading it now, so the Ex commands are
    // there when the loop asks.
    if !stdin_isatty.get() && !params.input_is_text && silent_mode.get() && exmode_active.get() {
        input_start();
    }

    // `--embed` without `--headless`: the client attaches a UI, and the
    // startup waits for it so the first redraw has somewhere to go.
    let use_remote_ui = embedded_mode.get() && !headless_mode.get();
    if use_remote_ui {
        time_msg_at(c"waiting for UI");
        remote_ui_wait_for_attach();
        time_msg_at(c"done waiting for UI");
        first_win().w_prev_height = first_win().w_height;
    }

    starting.set(NO_BUFFERS);
    screenclear();
    win_new_screensize();
    time_msg_at(c"clear screen");

    if edit_stdin(&params) {
        params.edit_type = EDIT_STDIN as c_int;
    }
    if let Some(scriptin) = params.scriptin.as_deref()
        && !open_scriptin(scriptin)
    {
        os_exit(2);
    }
    if let Some(name) = params.scriptout.as_deref() {
        let mode = if params.scriptout_append {
            APPENDBIN
        } else {
            WRITEBIN
        };
        scriptout.set(CFile::open(name, mode));
        if scriptout.with(Option::is_none) {
            let what = gettext(c"Cannot open for script output: \"").to_bytes();
            write_c_stderr(&[what, name.to_bytes(), b"\"\n"].concat());
            os_exit(2);
        }
    }

    nlua_init_defaults();
    time_msg_at(c"init default mappings & autocommands");

    // `-u NONE` also turns the plugins off, unless `--clean` asked for
    // the defaults.
    let vimrc_none = params.vimrc.as_deref() == Some(c"NONE");
    if vimrc_none {
        P_LPL.set(params.clean);
    }

    exe_pre_commands(&params);
    if !vimrc_none || params.clean {
        filetype_plugin_enable();
    }
    source_startup_scripts(&params);
    if !vimrc_none || params.clean {
        filetype_maybe_enable();
        syn_maybe_enable();
    }

    set_vim_var_nr(Vv::VimDidInit, 1 as VarNumber);
    load_plugins();
    set_window_layout(&mut params);

    if recoverymode.get() && !has_fname {
        // `-r` with no file: list the swap files and leave.
        list_swap_files(None);
        os_exit(0);
    }

    set_init_3();
    time_msg_at(c"inits 3");

    if params.no_swap_file {
        P_UC.set(0 as OptInt);
    }
    if silent_mode.get() {
        P_UT.set(1 as OptInt);
    }

    if P_SHADA.first_byte() != 0 {
        let _ = shada_read_everything(None, false, true);
        time_msg_at(c"reading ShaDa");
    }
    if get_vim_var_list_handle(Vv::Oldfiles).is_none() {
        set_vim_var_list(Vv::Oldfiles, Some(tv_list_alloc(0)));
    }

    handle_quickfix(&params);

    starting.set(NO_BUFFERS);
    no_wait_return.set(0);
    if !exmode_active.get() {
        msg_scroll.set(0);
    }

    if params.edit_type == EDIT_STDIN as c_int && !recoverymode.get() {
        read_stdin();
    }

    setmouse();
    redraw_later(Win::current(), UPD_VALID);
    no_wait_return.set(1);

    create_windows(&mut params);
    time_msg_at(c"opening buffers");

    // The swap command has served its purpose; the ATTENTION prompts
    // from here on are the user's own doing.
    set_vim_var_string(Vv::Swapcommand, None);

    if exmode_active.get() {
        Win::current().w_cursor.lnum = Buf::current().b_ml.ml_line_count;
    }
    fire_autocmds(AutoEvent::BufEnter, false, Buf::current_or_none());
    time_msg_at(c"BufEnter autocommands");
    setpcmark();

    if params.edit_type == EDIT_QF as c_int {
        qf_jump_to_current();
        time_msg_at(c"jump to first error");
    }

    edit_buffers(&params);

    if params.diff_mode {
        for wp in windows() {
            if !wp.w_arg_idx_invalid {
                diff_win_options(wp, true);
            }
        }
    }

    shorten_fnames(false);
    handle_tag(params.tag.as_deref());
    if !params.commands.is_empty() {
        exe_commands(&params);
    }

    starting.set(0);
    RedrawingDisabled.set(0);
    redraw_all_later(UPD_NOT_VALID);
    no_wait_return.set(0);
    do_autochdir();

    set_vim_var_nr(Vv::VimDidEnter, 1 as VarNumber);
    fire_autocmds(AutoEvent::VimEnter, false, Buf::current_or_none());
    time_msg_at(c"VimEnter autocommands");
    if use_remote_ui {
        do_autocmd_uienter_all();
        time_msg_at(c"UIEnter autocommands");
    }

    set_reg_var(get_default_register_name());

    if Win::current().w_onebuf_opt.wo_diff != 0 && Win::current().w_onebuf_opt.wo_scb != 0 {
        update_topline(Win::current());
        check_scrollbind(0 as LineNr, 0);
        time_msg_at(c"diff scrollbinding");
    }

    if restart_edit.get() != 0 {
        // A `-c startinsert` and friends: push the key that gets there.
        stuff_readbuf_char(-(253 + ((KE_NOP as c_int) << 8)));
    }

    if cb_flags.get() & (kOptCbFlagUnnamed as c_int | kOptCbFlagUnnamedplus as c_int) as c_uint != 0
    {
        // Warm the clipboard provider so the first yank is not slow.
        eval_has_provider(c"clipboard", false);
    }

    if let Some(lua_script) = params.lua_script.as_deref() {
        // `-l`: run the script and leave, with its status.
        msg_scroll.set(1);
        logmsg!(LOGLVL_DBG, c"main", 678, "executing Lua -l script");
        let lua_ok = nlua_exec_file(lua_script);
        time_msg_at(c"executing Lua -l script");
        if msg_didout.get() {
            msg_putchar('\n' as c_int);
            msg_didout.set(false);
        }
        getout(if lua_ok { 0 } else { 1 });
    }

    time_msg_at(c"before starting main loop");
    logmsg!(LOGLVL_INF, c"main", 689, "starting main loop");

    // Never returns.
    normal_enter(false, false);
    0
}

/// The process entry point: the command line, owned, handed to [`run`],
/// which does not return.
///
/// The words are the operating system's bytes, whatever their encoding: a
/// file name need not be UTF-8.
pub fn main() {
    use std::os::unix::ffi::OsStringExt;
    let argv: Vec<CString> = std::env::args_os()
        .map(|arg| CString::new(arg.into_vec()).expect("an argument holds no NUL"))
        .collect();
    std::process::exit(run(argv));
}

/// The first window of the current tab page, which exists from the moment
/// startup makes it until exit.
fn first_win() -> Win {
    first_window().expect("the editor always has a window")
}
