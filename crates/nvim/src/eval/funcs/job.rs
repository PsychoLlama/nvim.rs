//! Child processes: the `job*()` family and the environment it hands them.
#![forbid(unsafe_code)]

use super::{f_environ, kChannelPartRpc};
use crate::api::private::helpers::{dict_set_var, scope_vars};
use crate::autocmd::apply_autocmds;
use crate::buffer::{buf_close_terminal, setfname};
use crate::channel::open::JobOptions;
use crate::channel::{
    ChannelRef, channel_close, channel_create_event, channel_decref, channel_exists,
    channel_job_start, channel_process_events, channel_terminal_alloc, with_channel,
};
use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    DictRef, NumBuf, dict_extend, dict_find, dict_get_number, list_iter, tv_dict_alloc,
    tv_list_alloc,
};
use crate::eval::vars::vim_var_bytes;
use crate::eval::{ArgvRefusal, common_job_callbacks, find_job, tv_to_argv};
use crate::event::channel::{park_channel_events, wait_job};
use crate::event::multiqueue::ParkingQueue;
use crate::event::proc::proc_is_stopped;
use crate::ex_cmds::check_secure;
use crate::ex_getln::{text_locked, text_locked_msg};
use crate::memline::ml_open;
use crate::memory::XString;
use crate::message::emsg;
use crate::message::{e_channotpty, e_invarg};
use crate::message_fmt::msg_cstr;
use crate::option::vars::p_tgc;
use crate::os::cshim::gettext;
use crate::os::env::{ENV_NVIM, getenv_owned, home_replace_in};
use crate::os::fs::dir_exists;
use crate::os::time::os_hrtime;
use crate::path::vim_full_name;
use crate::semsg;
use crate::terminal::{Term, buf_terminal_running, terminal_buf, terminal_open};
use crate::types::AutoEvent;
use crate::types::channel::{kChannelStdinNull, kChannelStdinPipe};
use crate::types::{
    Callback, CallbackReader, ChannelStdinMode, ChannelStream, Dict, EvalFuncData, IOSIZE, Integer,
    JobProc, MAXPATHL, Object, String_0, TypVal, VAR_BOOL, VAR_DICT, VAR_LIST, VAR_NUMBER,
    VAR_STRING, VarNumber, Vv, uint16_t, uint64_t,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_flush};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

/// A cleared `CallbackReader`, which the option parser fills in.
const NO_READER: CallbackReader = CallbackReader::none();

/// A cleared `Callback`.
const NO_CALLBACK: Callback = Callback::None;

/// The job id a `job*()` builtin was handed, or `None` when the argument
/// was not a Number at all -- in which case the error is already out.
fn job_id(arg: &TypVal) -> Option<uint64_t> {
    if arg.v_type() != VAR_NUMBER {
        emsg(gettext(e_invarg));
        return None;
    }
    Some(arg.number_or_zero() as uint64_t)
}

/// `jobpid({job})`
pub fn f_jobpid(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    if check_secure() {
        return;
    }
    let Some(id) = job_id(&args[0]) else {
        return;
    };
    if !find_job(id, true) {
        return;
    }
    if let Some(pid) = with_job(id, |job| job.proc().pid) {
        result.write_number(VarNumber::from(pid));
    }
}

/// Run `f` on job channel `id`'s child, if it is still a registered job.
fn with_job<R>(id: uint64_t, f: impl FnOnce(&mut JobProc) -> R) -> Option<R> {
    with_channel(id, |channel| match &mut channel.stream {
        ChannelStream::Job(job) => Some(f(job)),
        _ => None,
    })
    .flatten()
}

/// `jobresize({job}, {width}, {height})` — only for a pty job.
pub fn f_jobresize(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    if check_secure() {
        return;
    }
    // All three arguments are checked together, so a bad width reports
    // the same message a bad job id does.
    if args[0].v_type() != VAR_NUMBER
        || args[1].v_type() != VAR_NUMBER
        || args[2].v_type() != VAR_NUMBER
    {
        emsg(gettext(e_invarg));
        return;
    }
    let id = args[0].number_or_zero() as uint64_t;
    if !find_job(id, true) {
        return;
    }
    let width = args[1].number_or_zero() as uint16_t;
    let height = args[2].number_or_zero() as uint16_t;
    let resized = with_job(id, |job| {
        let is_pty = job.pty().is_some();
        job.resize(width, height);
        is_pty
    });
    if resized != Some(true) {
        emsg(gettext(e_channotpty));
        return;
    }
    result.write_number(1);
}

/// `jobstop({job})`
pub fn f_jobstop(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    if check_secure() {
        return;
    }
    let Some(id) = job_id(&args[0]) else {
        return;
    };
    // `false`: a job that has already gone is not an error here.
    if !find_job(id, false) {
        return;
    }
    let is_rpc = with_channel(id, |channel| channel.is_rpc).unwrap_or(false);
    let refused = if is_rpc {
        channel_close(id, kChannelPartRpc).err()
    } else {
        None
    };
    with_job(id, JobProc::stop);
    // Reported as a success even when closing the RPC half complained.
    result.write_number(1);
    if let Some(why) = refused {
        emsg(why);
    }
}

/// `jobwait({jobs} [, {timeout}])`
pub fn f_jobwait(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_LIST
        || (!args.get(1).is_some_and(|arg| arg.v_type() == VAR_NUMBER) && args.len() > 1)
    {
        emsg(gettext(e_invarg));
        return;
    }

    // The waiting jobs' events are parked on a queue of our own so that
    // they do not run while we block.
    let waiting_jobs = ParkingQueue::on_main_loop();

    // One slot per item: the job, with a reference held for the wait, or
    // `None` for an item that is not a running job (reported as -3).
    let mut jobs: Vec<Option<ChannelRef>> = Vec::new();
    for arg in list_iter(args[0].list_ref()) {
        let job = if arg.li_tv.v_type() == VAR_NUMBER {
            let id = arg.li_tv.number_or_zero() as uint64_t;
            with_job(id, |job| {
                (id, proc_is_stopped(job.proc()), job.proc().status)
            })
        } else {
            None
        };
        jobs.push(match job {
            None => None,
            Some((id, true, _)) => {
                // Already gone; reap it and report -3 as well.
                let _held = ChannelRef::new(id);
                wait_job(id, -1, None);
                None
            }
            Some((id, false, status)) => {
                let held = ChannelRef::new(id);
                if status < 0 {
                    channel_process_events(id);
                    if channel_exists(id) {
                        park_channel_events(id, Some(&waiting_jobs));
                    }
                }
                held
            }
        });
    }

    // A negative or absent timeout means "no limit".
    let mut remaining = -1;
    let mut before = 0u64;
    if args.get(1).is_some_and(|arg| arg.v_type() == VAR_NUMBER) && args[1].number_or_zero() >= 0 {
        remaining = args[1].number_or_zero() as c_int;
        before = os_hrtime();
    }
    // Only mark the UI busy when this actually blocks.
    let busy = remaining != 0;
    if busy {
        ui_busy_start();
        ui_flush();
    }

    for job in jobs.iter().flatten() {
        if remaining == 0 {
            break;
        }
        let status = wait_job(job.id(), remaining, Some(&waiting_jobs));
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

    let statuses = tv_list_alloc(jobs.len() as isize);
    for job in jobs {
        let Some(job) = job else {
            statuses.edit().push_number(-3);
            continue;
        };
        // Hand the parked events back before reporting.
        let id = job.id();
        channel_process_events(id);
        park_channel_events(id, None);
        let status = with_job(id, |job| job.proc().status).unwrap_or(-1);
        statuses.edit().push_number(VarNumber::from(status));
    }

    drop(waiting_jobs);
    if busy {
        ui_busy_stop();
    }
    result.write_list(Some(statuses));
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

/// Build the environment for a child process: ours (unless `clear_env`),
/// adjusted for a pty, then the job's own `job_env` on top.
///
/// `set_nvim_addr` points the child's `$NVIM` at this instance's server
/// address, so a job can talk back; `:restart`'s replacement server is not
/// a job of this one and leaves it alone. `pty_term_name` is the child's
/// `$TERM`, and present whenever `pty` is set.
pub(crate) fn create_environment(
    job_env: Option<&DictRef>,
    clear_env: bool,
    pty: bool,
    set_nvim_addr: bool,
    pty_term_name: Option<&CStr>,
) -> DictRef {
    let mut env = tv_dict_alloc();

    if !clear_env {
        // Start from our own environment. `f_environ` is the builtin,
        // called directly because it is the only thing that knows how
        // to turn `environ` into a Dict.
        let mut inherited = TV_INITIAL_VALUE;
        f_environ(&[], &mut inherited, EvalFuncData::None);
        if let Some(inherited) = inherited.dict_shared() {
            dict_extend(&env, inherited, b'f');
        }
        // The one reference, given back with the handle.
        drop(inherited.take_dict());

        if pty {
            for name in PTY_IGNORED_ENV {
                drop(env.remove_key(name.to_bytes()));
            }
            // COLORTERM was just removed; put ours back when we know
            // the child can use it.
            if p_tgc() {
                let _ = env.add_str(b"COLORTERM", Some(c"truecolor"));
            }
        }
    }

    if pty {
        drop(env.remove_key(b"TERM"));
        let _ = env.add_str(b"TERM", pty_term_name);
    }

    // $NVIM points the child at this instance's server address, when
    // there is one. #3118
    if set_nvim_addr {
        let nvim_addr = vim_var_bytes(Vv::Servername);
        if !nvim_addr.is_empty() {
            drop(env.remove_key(ENV_NVIM.to_bytes()));
            let _ = env.add_str_len(ENV_NVIM.to_bytes(), Some(&nvim_addr));
        }
    }

    // The job's own `env` wins over everything above.
    if let Some(job_env) = job_env {
        dict_extend(&env, job_env, b'f');
    }

    if pty {
        for name in REQUIRED_ENV {
            if env.has_key(name.to_bytes()) {
                continue;
            }
            if let Some(value) = getenv_owned(name) {
                let _ = env.add_str(name.to_bytes(), Some(value.as_cstr()));
            }
        }
    }

    env
}

/// `jobstart({cmd} [, {opts}])`
pub fn f_jobstart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut cmdbuf = NumBuf::new();
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    result.write_number(0);
    // SAFETY throughout: the frame is live.
    if check_secure() {
        return;
    }

    let argv = match tv_to_argv(&args[0]) {
        Ok(argv) => argv,
        // A malformed command answers 0; a command that is simply not
        // executable answers -1.
        Err(refusal) => {
            result.write_number(if refusal == ArgvRefusal::NotExecutable {
                -1
            } else {
                0
            });
            return;
        }
    };
    // What a terminal buffer is named after: the command line as given, or
    // the resolved program of a List.
    let cmd: CString = if args[0].v_type() == VAR_STRING {
        cmdbuf.string(&args[0]).to_owned()
    } else {
        argv.program().to_owned()
    };
    // Every early exit below releases `argv` by dropping it.
    if !args.get(1).is_some_and(|arg| arg.v_type() == VAR_DICT) && args.len() > 1 {
        let arg0 = "expected dictionary";
        semsg!("E475: Invalid argument: {arg0}");
        return;
    }

    let mut job_opts: Option<&Dict> = None;
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
    let mut cwd: Option<&CStr> = None;
    // A reference to the env dictionary itself, not the item holding it:
    // the borrow the lookup answers does not survive `common_job_callbacks`,
    // which takes a reference to the options dictionary it was found in.
    let mut job_env: Option<DictRef> = None;

    if args.get(1).is_some_and(|arg| arg.v_type() == VAR_DICT) {
        job_opts = args[1].dict_ref();
        detach = dict_get_number(job_opts, b"detach") != 0;
        rpc = dict_get_number(job_opts, b"rpc") != 0;
        term = dict_get_number(job_opts, b"term") != 0;
        pty = term || dict_get_number(job_opts, b"pty") != 0;
        clear_env = dict_get_number(job_opts, b"clear_env") != 0;
        overlapped = dict_get_number(job_opts, b"overlapped") != 0;

        // An unrecognised `stdin` is a warning, not a failure.
        // `strncmp` over NUMBUFLEN bytes against a shorter literal is an
        // exact comparison.
        if let Some(s) = numbuf.dict_string(job_opts, b"stdin") {
            if s == c"null" {
                stdin_mode = kChannelStdinNull;
            } else if s != c"pipe" {
                let (arg0, s) = (msg_cstr(c"stdin"), msg_cstr(s));
                semsg!("E475: Invalid value for argument {arg0}: {s}");
            }
        }

        // `term` is the one option whose *type* is checked, because a
        // truthy string used to mean something else.
        let job_term = dict_find(job_opts, b"term");
        if job_term.is_some_and(|di| di.di_tv.v_type() != VAR_BOOL) {
            semsg!("E475: Invalid argument: 'term' must be Boolean");
            return;
        }
        if pty && rpc {
            semsg!("E475: Invalid argument: job cannot have both 'pty' and 'rpc' options set");
            return;
        }

        if let Some(new_cwd) = numbuf2.dict_string(job_opts, b"cwd")
            && !new_cwd.is_empty()
        {
            cwd = Some(new_cwd);
            if !dir_exists(new_cwd) {
                semsg!("E475: Invalid argument: expected valid directory");
                return;
            }
        }

        if let Some(di) = dict_find(job_opts, b"env") {
            if di.di_tv.v_type() != VAR_DICT {
                let arg0 = "env";
                semsg!("E475: Invalid argument: {arg0}");
                return;
            }
            job_env = di.di_tv.dict_handle();
        }

        let options = args[1].dict_shared();
        if !common_job_callbacks(options, &mut on_stdout, &mut on_stderr, &mut on_exit) {
            return;
        }
        // The call above took a reference to the options dictionary, which
        // is a write through it: the borrow taken before that is spent, so
        // the reads after it start from the argument again.
        job_opts = args[1].dict_ref();
    }

    // `dict_get_number` accepts a null dict, so these two are read
    // whether or not there were options at all.
    let mut width = dict_get_number(job_opts, b"width") as uint16_t;
    let mut height = dict_get_number(job_opts, b"height") as uint16_t;
    let mut term_name: Option<&CStr> = None;

    if term {
        if text_locked() {
            text_locked_msg();
            return;
        }
        if Buf::current().b_changed != 0 {
            let msg = c"jobstart(...,{term=true}) requires unmodified buffer";
            emsg(gettext(msg));
            return;
        }
        if !Buf::current().terminal.is_null() {
            if buf_terminal_running(Buf::current()) {
                let handle = Buf::current().handle;
                semsg!("Terminal already connected to buffer {}", handle);
                return;
            }
            buf_close_terminal(Buf::current());
        }
        // `pty && rpc` was refused above and `term` implies `pty`.
        debug_assert!(!rpc);

        term_name = Some(c"xterm-256color");
        if cwd.is_none() {
            cwd = Some(c".");
        }
        overlapped = false;
        detach = false;
        stdin_mode = kChannelStdinPipe;
        if width == 0 {
            width = (Win::current().w_view_width - Win::current().col_off()).max(0) as uint16_t;
        }
        if height == 0 {
            height = Win::current().w_view_height as uint16_t;
        }
    }
    if pty && term_name.is_none() {
        term_name = Some(numbuf3.dict_string(job_opts, b"TERM").unwrap_or(c"ansi"));
    }

    let env = create_environment(job_env.as_ref(), clear_env, pty, true, term_name);
    let options = JobOptions {
        on_stdout,
        on_stderr,
        on_exit,
        pty,
        rpc,
        overlapped,
        detach,
        stdin_mode,
        cwd,
        pty_width: width,
        pty_height: height,
        env: Some(env),
        ..JobOptions::new(argv)
    };
    // The id, or why the start failed, lands in the return value.
    let id = match channel_job_start(options) {
        Ok(id) => id,
        Err(status) => {
            result.write_number(status);
            return;
        }
    };
    result.write_number(id.cast_signed());
    if !term {
        channel_create_event(id, None);
        return;
    }
    attach_terminal(id, cwd.unwrap_or(c"."), &cmd);
}

/// Give `{term: v:true}` job channel `id` the current buffer.
fn attach_terminal(id: uint64_t, cwd: &CStr, cmd: &CStr) {
    let pid = with_job(id, |job| job.proc().pid).unwrap_or(0);
    let mut buf = Buf::current();
    buf.b_p_swf = 0;
    if buf.b_ml.ml_mfp.is_none() && ml_open(buf).is_err() {
        with_job(id, JobProc::stop);
        channel_decref(id);
        return;
    }
    // Held for the autocommands below, which may close the terminal.
    let held = ChannelRef::new(id);
    channel_terminal_alloc(buf, id);
    apply_autocmds(AutoEvent::BufFilePre, None, None, false, Some(buf));

    // The autocommand may have closed the terminal out from under us,
    // which is what each of these three re-tests is for.
    if terminal_live(id) {
        // Name the buffer `term://{cwd}//{pid}:{cmd}`.
        let full = vim_full_name(cwd, MAXPATHL as usize, false).unwrap_or_else(|name| name);
        let full = cstr::owned(&full);
        let mut dir = home_replace_in(None, &full, IOSIZE as usize, true)
            .as_cstr()
            .to_bytes()
            .to_vec();
        // Drop a trailing separator, but keep `/` itself meaningful by
        // spelling it `/.`.
        let len = dir.len();
        if len != 1 && matches!(dir.last(), Some(b'\\' | b'/')) {
            dir.pop();
        }
        if len == 1 && dir[0] == b'/' {
            dir.push(b'.');
        }
        let mut name = b"term://".to_vec();
        name.extend_from_slice(&dir);
        name.extend_from_slice(format!("//{pid}:").as_bytes());
        name.extend_from_slice(cmd.to_bytes());
        // `snprintf` into a `MAXPATHL` buffer.
        name.truncate(MAXPATHL as usize - 1);
        let name = XString::from_bytes(&name);
        let _ = setfname(buf, Some(name.as_cstr()), None, true);
        apply_autocmds(AutoEvent::BufFilePost, None, None, false, Some(buf));

        if terminal_live(id) {
            // Locked so that the two variables cannot be swapped out
            // from under the terminal by a BufFilePost autocommand.
            buf.b_locked += 1;
            set_buf_var(buf, c"terminal_job_id", id.cast_signed());
            set_buf_var(buf, c"terminal_job_pid", Integer::from(pid));
            buf.b_locked -= 1;

            if let Some(term) = live_terminal(id) {
                terminal_open(term, buf, &|| terminal_live(id));
            }
        }
    }

    channel_create_event(id, None);
    drop(held);
}

/// Channel `id`'s terminal, while it is still attached to a real buffer.
fn live_terminal(id: uint64_t) -> Option<Term> {
    with_channel(id, |channel| channel.term)
        .flatten()
        .filter(|&term| terminal_buf(term) != 0)
}

/// Whether channel `id` still has a terminal attached to a real buffer.
fn terminal_live(id: uint64_t) -> bool {
    live_terminal(id).is_some()
}

/// Set one buffer-local variable to an Integer, discarding any refusal.
fn set_buf_var(buffer: Buf, name: &CStr, value: Integer) {
    let value = Object::Integer(value);
    let vars = scope_vars(&buffer.b_bufvar);
    let name = String_0::from_cstr(name);
    drop(dict_set_var(&vars, &name, value, false, false));
}
