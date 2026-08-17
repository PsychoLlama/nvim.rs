//! Child processes: the `job*()` family and the environment it hands them.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{
    C2Rust_Unnamed_16, C2Rust_Unnamed_22, FAIL, GA_EMPTY_INIT_VALUE, NUL, NUMBUFLEN, f_environ,
    false_0, kChannelPartRpc, kChannelStreamProc, kProcTypePty,
};
use crate::api::private::helpers::{api_clear_error, cstr_as_string, dict_set_var};
use crate::autocmd::{EVENT_BUFFILEPOST, EVENT_BUFFILEPRE, apply_autocmds};
use crate::buffer::{buf_close_terminal, setfname};
use crate::channel::{
    channel_close, channel_create_event, channel_decref, channel_incref, channel_job_start,
    channel_proc, channel_pty, channel_terminal_alloc, find_channel,
};
use crate::eval::typval::{
    kCallbackNone, tv_dict_add_allocated_str, tv_dict_add_str, tv_dict_alloc, tv_dict_extend,
    tv_dict_find, tv_dict_free, tv_dict_get_number, tv_dict_get_string, tv_dict_item_remove,
    tv_list_alloc, tv_list_append_number, tv_list_len, tv_list_ref,
};
use crate::eval::vars::get_vim_var_str;
use crate::eval::{common_job_callbacks, find_job, tv_to_argv};
use crate::event::r#loop::loop_on_put;
use crate::event::multiqueue::{
    multiqueue_free, multiqueue_new, multiqueue_process_events, multiqueue_replace_parent,
};
use crate::event::proc::{proc_is_stopped, proc_stop, proc_wait};
use crate::ex_cmds::check_secure;
use crate::ex_getln::{text_locked, text_locked_msg};
use crate::main::{
    IObuff, NameBuff, curbuf, curwin, e_channotpty, e_invarg, e_invarg2, e_invargNval, main_loop,
    p_tgc,
};
use crate::memline::ml_open;
use crate::memory::{xcalloc, xfree};
use crate::message::emsg;
use crate::r#move::win_col_off;
use crate::os::env::{home_replace, os_getenv};
use crate::os::fs::os_isdir;
use crate::os::libc::{gettext, snprintf, strlen, strncmp};
use crate::os::pty_proc_unix::pty_proc_resize;
use crate::os::shell::shell_free_argv;
use crate::os::time::os_hrtime;
use crate::path::vim_FullName;
use crate::semsg_c;
use crate::terminal::{terminal_buf, terminal_open, terminal_running};
use crate::types::channel::{kChannelStdinNull, kChannelStdinPipe};
use crate::types::{
    Arena, Callback, CallbackReader, Channel, ChannelStdinMode, Error, EvalFuncData, Integer,
    VAR_BOOL, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_UNKNOWN, VAR_UNLOCKED, VV_SEND_SERVER, buf_T,
    dict_T, dictitem_T, kErrorTypeNone, kObjectTypeInteger, list_T, listitem_T, object, typval_T,
    typval_vval_union, uint16_t, uint64_t, varnumber_T,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_flush};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// A cleared `CallbackReader`, which the option parser fills in.
const NO_READER: CallbackReader = CallbackReader {
    cb: NO_CALLBACK,
    self_0: ptr::null_mut::<dict_T>(),
    buffer: GA_EMPTY_INIT_VALUE,
    eof: false,
    buffered: false,
    fwd_err: false,
    type_0: ptr::null::<c_char>(),
};

/// A cleared `Callback`.
const NO_CALLBACK: Callback = Callback {
    data: C2Rust_Unnamed_22 {
        funcref: ptr::null_mut::<c_char>(),
    },
    type_0: kCallbackNone,
};

/// The job id a `job*()` builtin was handed, or `None` when the argument
/// was not a Number at all -- in which case the error is already out.
///
/// # Safety
/// `arg` is a live typval.
unsafe fn job_id(arg: &typval_T) -> Option<uint64_t> {
    if arg.v_type != VAR_NUMBER {
        // SAFETY: `e_invarg` is a live NUL-terminated buffer.
        unsafe { emsg(gettext(e_invarg.as_ptr())) };
        return None;
    }
    // SAFETY: the type tag names the union member.
    Some(unsafe { arg.vval.v_number } as uint64_t)
}

/// `jobpid({job})`
pub unsafe extern "C" fn f_jobpid(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `find_job` answers with a live channel or
    // null.
    unsafe {
        if check_secure() {
            return;
        }
        let Some(id) = job_id(args.get(0)) else {
            return;
        };
        let data = find_job(id, true);
        if data.is_null() {
            return;
        }
        rettv.vval.v_number = (*channel_proc(data)).pid as varnumber_T;
    }
}

/// `jobresize({job}, {width}, {height})` — only for a pty job.
pub unsafe extern "C" fn f_jobresize(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `find_job` answers with a live channel or
    // null.
    unsafe {
        if check_secure() {
            return;
        }
        // All three arguments are checked together, so a bad width reports
        // the same message a bad job id does.
        if args.ty(0) != VAR_NUMBER || args.ty(1) != VAR_NUMBER || args.ty(2) != VAR_NUMBER {
            emsg(gettext(e_invarg.as_ptr()));
            return;
        }
        let data = find_job(args.get(0).vval.v_number as uint64_t, true);
        if data.is_null() {
            return;
        }
        if (*channel_proc(data)).type_0 != kProcTypePty {
            emsg(gettext(e_channotpty.as_ptr()));
            return;
        }
        pty_proc_resize(
            channel_pty(data),
            args.get(1).vval.v_number as uint16_t,
            args.get(2).vval.v_number as uint16_t,
        );
        rettv.vval.v_number = 1;
    }
}

/// `jobstop({job})`
pub unsafe extern "C" fn f_jobstop(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `find_job` answers with a live channel or
    // null, and `error` is a borrowed static message.
    unsafe {
        if check_secure() {
            return;
        }
        let Some(id) = job_id(args.get(0)) else {
            return;
        };
        // `false`: a job that has already gone is not an error here.
        let data = find_job(id, false);
        if data.is_null() {
            return;
        }
        let mut error = ptr::null::<c_char>();
        if (*data).is_rpc {
            channel_close((*data).id, kChannelPartRpc, &raw mut error);
        }
        proc_stop(channel_proc(data));
        // Reported as a success even when closing the RPC half complained.
        rettv.vval.v_number = 1;
        if !error.is_null() {
            emsg(error);
        }
    }
}

/// `jobwait({jobs} [, {timeout}])`
pub unsafe extern "C" fn f_jobwait(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `jobs` is an allocation this body owns for
    // its whole length, and every channel in it holds a reference.
    unsafe {
        if check_secure() {
            return;
        }
        if args.ty(0) != VAR_LIST || (args.ty(1) != VAR_NUMBER && args.has(1)) {
            emsg(gettext(e_invarg.as_ptr()));
            return;
        }

        let list: *mut list_T = args.get(0).vval.v_list;
        let count = tv_list_len(list);
        let jobs = xcalloc(count as usize, size_of::<*mut Channel>()) as *mut *mut Channel;
        // The waiting jobs' events are parked on a queue of our own so that
        // they do not run while we block.
        let waiting_jobs = multiqueue_new(Some(loop_on_put), main_loop.ptr() as *mut c_void);

        let mut i = 0;
        if !list.is_null() {
            let mut arg: *const listitem_T = (*list).lv_first;
            while !arg.is_null() {
                let mut chan = ptr::null_mut::<Channel>();
                if (*arg).li_tv.v_type != VAR_NUMBER
                    || {
                        chan = find_channel((*arg).li_tv.vval.v_number as uint64_t);
                        chan.is_null()
                    }
                    || (*chan).streamtype != kChannelStreamProc
                {
                    // Not a job: reported as -3 below.
                    *jobs.add(i as usize) = ptr::null_mut();
                } else if proc_is_stopped(&*channel_proc(chan)) {
                    // Already gone; reap it and report -3 as well.
                    proc_wait(channel_proc(chan), -1, ptr::null_mut());
                    *jobs.add(i as usize) = ptr::null_mut();
                } else {
                    *jobs.add(i as usize) = chan;
                    channel_incref(chan);
                    if (*channel_proc(chan)).status < 0 {
                        multiqueue_process_events((*chan).events);
                        multiqueue_replace_parent((*chan).events, waiting_jobs);
                    }
                }
                i += 1;
                arg = (*arg).li_next;
            }
        }

        // A negative or absent timeout means "no limit".
        let mut remaining = -1;
        let mut before = 0u64;
        if args.ty(1) == VAR_NUMBER && args.get(1).vval.v_number >= 0 {
            remaining = args.get(1).vval.v_number as c_int;
            before = os_hrtime();
        }
        // Only mark the UI busy when this actually blocks.
        let busy = remaining != 0;
        if busy {
            ui_busy_start();
            ui_flush();
        }

        for i in 0..count {
            if remaining == 0 {
                break;
            }
            if (*jobs.add(i as usize)).is_null() {
                continue;
            }
            let status = proc_wait(channel_proc(*jobs.add(i as usize)), remaining, waiting_jobs);
            if status < 0 {
                // Interrupted or timed out; the rest report -1.
                break;
            }
            if remaining > 0 {
                let now = os_hrtime();
                let elapsed = now.wrapping_sub(before).wrapping_div(1_000_000) as c_int;
                // Upstream writes MIN here, not MAX, so any positive
                // timeout collapses to 0 after the first job and the loop
                // stops. Preserved; see the commit that rewrote this file.
                remaining = (remaining - elapsed).min(0);
                before = now;
            }
        }

        let rv = tv_list_alloc(count as isize);
        for i in 0..count {
            let chan = *jobs.add(i as usize);
            if chan.is_null() {
                tv_list_append_number(rv, -3);
                continue;
            }
            // Hand the parked events back before reporting.
            multiqueue_process_events((*chan).events);
            multiqueue_replace_parent((*chan).events, (*main_loop.ptr()).events);
            tv_list_append_number(rv, (*channel_proc(chan)).status as varnumber_T);
            channel_decref(chan);
        }

        multiqueue_free(waiting_jobs);
        xfree(jobs as *mut c_void);
        if busy {
            ui_busy_stop();
        }
        tv_list_ref(rv);
        rettv.v_type = VAR_LIST;
        rettv.vval.v_list = rv;
    }
}

/// Variables a pty job must not inherit: they describe *our* terminal, and
/// the child gets its own.
const PTY_IGNORED_ENV: [&CStr; 7] = [
    c"COLUMNS",
    c"LINES",
    c"TERMCAP",
    c"COLORFGBG",
    c"COLORTERM",
    c"VIM",
    c"VIMRUNTIME",
];

/// Variables a pty job must inherit from *our* environment even when the
/// job's own `env` does not mention them.
///
/// Empty, and upstream's is too -- the C array holds nothing but its NULL
/// terminator. The loop below is kept because the list is the thing that
/// would change.
const REQUIRED_ENV: [&CStr; 0] = [];

/// Build the environment for a child process.
///
/// # Safety
/// `job_env` is null or a live dict item holding a Dict; `pty_term_name` is
/// null or a NUL-terminated string, and non-null whenever `pty` is set.
unsafe fn create_environment(
    job_env: *const dictitem_T,
    clear_env: bool,
    pty: bool,
    pty_term_name: *const c_char,
) -> *mut dict_T {
    // SAFETY: the caller's obligation; every key below is a `'static`
    // NUL-terminated string and the dict owns what it is given.
    unsafe {
        let env = tv_dict_alloc();

        if !clear_env {
            // Start from our own environment. `f_environ` is the builtin,
            // called directly because it is the only thing that knows how
            // to turn `environ` into a Dict.
            let mut inherited = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            f_environ(
                ptr::null_mut(),
                &raw mut inherited,
                EvalFuncData {
                    null: ptr::null_mut(),
                },
            );
            tv_dict_extend(env, inherited.vval.v_dict, c"force".as_ptr());
            tv_dict_free(inherited.vval.v_dict);

            if pty {
                for name in PTY_IGNORED_ENV {
                    let dv = tv_dict_find(env, name.as_ptr(), -1);
                    if !dv.is_null() {
                        tv_dict_item_remove(env, dv);
                    }
                }
                // COLORTERM was just removed; put ours back when we know
                // the child can use it.
                if p_tgc.get() != 0 {
                    tv_dict_add_str(env, c"COLORTERM".as_ptr(), 9, c"truecolor".as_ptr());
                }
            }
        }

        if pty {
            let dv = tv_dict_find(env, c"TERM".as_ptr(), 4);
            if !dv.is_null() {
                tv_dict_item_remove(env, dv);
            }
            tv_dict_add_str(env, c"TERM".as_ptr(), 4, pty_term_name);
        }

        // $NVIM points the child at this instance's server address, when
        // there is one.
        let nvim_addr = get_vim_var_str(VV_SEND_SERVER);
        if *nvim_addr as c_int != NUL {
            let dv = tv_dict_find(env, c"NVIM".as_ptr(), 4);
            if !dv.is_null() {
                tv_dict_item_remove(env, dv);
            }
            tv_dict_add_str(env, c"NVIM".as_ptr(), 4, nvim_addr);
        }

        // The job's own `env` wins over everything above.
        if !job_env.is_null() {
            tv_dict_extend(env, (*job_env).di_tv.vval.v_dict, c"force".as_ptr());
        }

        if pty {
            for name in REQUIRED_ENV {
                let len = strlen(name.as_ptr());
                if tv_dict_find(env, name.as_ptr(), len as isize).is_null() {
                    let value = os_getenv(name.as_ptr());
                    if !value.is_null() {
                        tv_dict_add_allocated_str(env, name.as_ptr(), len, value);
                    }
                }
            }
        }

        env
    }
}

/// `jobstart({cmd} [, {opts}])`
pub unsafe extern "C" fn f_jobstart(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `argv` is released on every path that does
    // not hand it to `channel_job_start`, which adopts it.
    unsafe {
        if check_secure() {
            return;
        }

        let mut cmd = ptr::null::<c_char>();
        let mut executable = true;
        let argv = tv_to_argv(args.ptr(0), &raw mut cmd, &raw mut executable);
        if argv.is_null() {
            // A malformed command answers 0; a command that is simply not
            // executable answers -1.
            rettv.vval.v_number = if executable { 0 } else { -1 };
            return;
        }
        // From here on every early exit must release `argv`.
        macro_rules! bail {
            () => {{
                shell_free_argv(argv);
                return;
            }};
        }

        if args.ty(1) != VAR_DICT && args.has(1) {
            semsg_c!(gettext(e_invarg2.as_ptr()), c"expected dictionary".as_ptr(),);
            bail!();
        }

        let mut job_opts = ptr::null_mut::<dict_T>();
        let mut detach = false;
        let mut rpc = false;
        let mut pty = false;
        let mut term = false;
        let mut clear_env = false;
        let mut overlapped = false;
        let mut stdin_mode: ChannelStdinMode = kChannelStdinPipe;
        let mut on_stdout = NO_READER;
        let mut on_stderr = NO_READER;
        let mut on_exit = NO_CALLBACK;
        let mut cwd = ptr::null_mut::<c_char>();
        let mut job_env = ptr::null_mut::<dictitem_T>();

        if args.ty(1) == VAR_DICT {
            job_opts = args.get(1).vval.v_dict;
            detach = tv_dict_get_number(job_opts, c"detach".as_ptr()) != 0;
            rpc = tv_dict_get_number(job_opts, c"rpc".as_ptr()) != 0;
            term = tv_dict_get_number(job_opts, c"term".as_ptr()) != 0;
            pty = term || tv_dict_get_number(job_opts, c"pty".as_ptr()) != 0;
            clear_env = tv_dict_get_number(job_opts, c"clear_env".as_ptr()) != 0;
            overlapped = tv_dict_get_number(job_opts, c"overlapped".as_ptr()) != 0;

            // An unrecognised `stdin` is a warning, not a failure.
            let s = tv_dict_get_string(job_opts, c"stdin".as_ptr(), false);
            if !s.is_null() {
                if strncmp(s, c"null".as_ptr(), NUMBUFLEN as usize) == 0 {
                    stdin_mode = kChannelStdinNull;
                } else if strncmp(s, c"pipe".as_ptr(), NUMBUFLEN as usize) != 0 {
                    semsg_c!(gettext(e_invargNval.as_ptr()), c"stdin".as_ptr(), s,);
                }
            }

            // `term` is the one option whose *type* is checked, because a
            // truthy string used to mean something else.
            let job_term = tv_dict_find(job_opts, c"term".as_ptr(), 4);
            if !job_term.is_null() && (*job_term).di_tv.v_type != VAR_BOOL {
                semsg_c!(
                    gettext(e_invarg2.as_ptr()),
                    c"'term' must be Boolean".as_ptr(),
                );
                bail!();
            }
            if pty && rpc {
                semsg_c!(
                    gettext(e_invarg2.as_ptr()),
                    c"job cannot have both 'pty' and 'rpc' options set".as_ptr(),
                );
                bail!();
            }

            let new_cwd = tv_dict_get_string(job_opts, c"cwd".as_ptr(), false);
            if !new_cwd.is_null() && *new_cwd as c_int != NUL {
                cwd = new_cwd;
                if !os_isdir(cwd) {
                    semsg_c!(
                        gettext(e_invarg2.as_ptr()),
                        c"expected valid directory".as_ptr(),
                    );
                    bail!();
                }
            }

            job_env = tv_dict_find(job_opts, c"env".as_ptr(), 3);
            if !job_env.is_null() && (*job_env).di_tv.v_type != VAR_DICT {
                semsg_c!(gettext(e_invarg2.as_ptr()), c"env".as_ptr());
                bail!();
            }

            if !common_job_callbacks(
                job_opts,
                &raw mut on_stdout,
                &raw mut on_stderr,
                &raw mut on_exit,
            ) {
                bail!();
            }
        }

        // `tv_dict_get_number` accepts a null dict, so these two are read
        // whether or not there were options at all.
        let mut width = tv_dict_get_number(job_opts, c"width".as_ptr()) as uint16_t;
        let mut height = tv_dict_get_number(job_opts, c"height".as_ptr()) as uint16_t;
        let mut term_name = ptr::null_mut::<c_char>();

        if term {
            if text_locked() {
                text_locked_msg();
                bail!();
            }
            if (*curbuf.get()).b_changed != 0 {
                emsg(gettext(
                    c"jobstart(...,{term=true}) requires unmodified buffer".as_ptr(),
                ));
                bail!();
            }
            if !(*curbuf.get()).terminal.is_null() {
                if terminal_running((*curbuf.get()).terminal) {
                    semsg_c!(
                        gettext(c"Terminal already connected to buffer %d".as_ptr()),
                        (*curbuf.get()).handle,
                    );
                    bail!();
                }
                buf_close_terminal(curbuf.get());
            }
            // `pty && rpc` was refused above and `term` implies `pty`.
            debug_assert!(!rpc);

            term_name = c"xterm-256color".as_ptr() as *mut c_char;
            if cwd.is_null() {
                cwd = c".".as_ptr() as *mut c_char;
            }
            overlapped = false;
            detach = false;
            stdin_mode = kChannelStdinPipe;
            if width == 0 {
                width =
                    ((*curwin.get()).w_view_width - win_col_off(curwin.get())).max(0) as uint16_t;
            }
            if height == 0 {
                height = (*curwin.get()).w_view_height as uint16_t;
            }
        }
        if pty && term_name.is_null() {
            term_name = tv_dict_get_string(job_opts, c"TERM".as_ptr(), false);
            if term_name.is_null() {
                term_name = c"ansi".as_ptr() as *mut c_char;
            }
        }

        let env = create_environment(job_env, clear_env, pty, term_name);
        let chan = channel_job_start(
            argv,
            ptr::null(),
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
            &raw mut rettv.vval.v_number,
        );
        if chan.is_null() {
            return;
        }
        if !term {
            channel_create_event(chan, ptr::null());
            return;
        }
        if rettv.vval.v_number <= 0 {
            return;
        }
        attach_terminal(chan, cwd, cmd);
    }
}

/// Give a `{term: v:true}` job the current buffer.
///
/// # Safety
/// `chan` is a live channel with a running process, `cwd` and `cmd` are
/// NUL-terminated strings.
unsafe fn attach_terminal(chan: *mut Channel, cwd: *const c_char, cmd: *const c_char) {
    // SAFETY: the caller's obligation; `NameBuff` and `IObuff` are the
    // shared scratch buffers and are only used within this body.
    unsafe {
        let pid = (*channel_proc(chan)).pid;
        let buf = curbuf.get();
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
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            buf,
        );

        // The autocommand may have closed the terminal out from under us,
        // which is what each of these three re-tests is for.
        if terminal_live(chan) {
            // Name the buffer `term://{cwd}//{pid}:{cmd}`.
            vim_FullName(cwd, NameBuff.ptr() as *mut c_char, 4096, false);
            let len = home_replace(
                ptr::null(),
                NameBuff.ptr() as *mut c_char,
                IObuff.ptr() as *mut c_char,
                1025,
                true,
            );
            // Drop a trailing separator, but keep `/` itself meaningful by
            // spelling it `/.`.
            let io = IObuff.ptr();
            if len != 1 && matches!((*io)[len - 1] as u8, b'\\' | b'/') {
                (*io)[len - 1] = NUL as c_char;
            }
            if len == 1 && (*io)[0] as u8 == b'/' {
                (*io)[1] = b'.' as c_char;
                (*io)[2] = NUL as c_char;
            }
            snprintf(
                NameBuff.ptr() as *mut c_char,
                4096,
                c"term://%s//%d:%s".as_ptr(),
                IObuff.ptr() as *mut c_char,
                pid,
                cmd,
            );
            setfname(buf, NameBuff.ptr() as *mut c_char, ptr::null_mut(), true);
            apply_autocmds(
                EVENT_BUFFILEPOST,
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                buf,
            );

            if terminal_live(chan) {
                let mut err = Error {
                    type_0: kErrorTypeNone,
                    msg: ptr::null_mut(),
                };
                // Locked so that the two variables cannot be swapped out
                // from under the terminal by a BufFilePost autocommand.
                (*buf).b_locked += 1;
                set_buf_var(buf, c"terminal_job_id", (*chan).id as Integer, &raw mut err);
                set_buf_var(buf, c"terminal_job_pid", pid as Integer, &raw mut err);
                (*buf).b_locked -= 1;

                if terminal_live(chan) {
                    terminal_open(&raw mut (*chan).term, buf);
                }
            }
        }

        channel_create_event(chan, ptr::null());
        channel_decref(chan);
    }
}

/// Whether the channel still has a terminal attached to a real buffer.
///
/// # Safety
/// `chan` is a live channel.
unsafe fn terminal_live(chan: *mut Channel) -> bool {
    // SAFETY: the caller's obligation.
    unsafe { !(*chan).term.is_null() && terminal_buf((*chan).term) != 0 }
}

/// Set one buffer-local variable to an Integer, discarding any error.
///
/// # Safety
/// `buf` is a live buffer and `err` a live out-parameter.
unsafe fn set_buf_var(buf: *mut buf_T, name: &CStr, value: Integer, err: *mut Error) {
    // SAFETY: the caller's obligation; the name is `'static`.
    unsafe {
        dict_set_var(
            (*buf).b_vars,
            cstr_as_string(name.as_ptr()),
            object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_16 { integer: value },
            },
            false,
            false,
            ptr::null_mut::<Arena>(),
            err,
        );
        api_clear_error(err);
    }
}
