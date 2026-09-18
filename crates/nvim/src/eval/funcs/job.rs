//! Child processes: the `job*()` family and the environment it hands them.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::{NUMBUFLEN, f_environ, kChannelPartRpc, kChannelStreamProc, kProcTypePty};
use crate::api::private::helpers::{cstr_to_string, dict_set_var};
use crate::autocmd::apply_autocmds;
use crate::buffer::{buf_close_terminal, setfname};
use crate::channel::{
    channel_close, channel_create_event, channel_decref, channel_incref, channel_job_start,
    channel_proc, channel_pty, channel_terminal_alloc, find_channel,
};
use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    NumBuf, dict_extend, dict_find, dict_get_number, list_iter, list_len, tv_dict_alloc,
    tv_dict_free, tv_dict_item_remove, tv_list_alloc,
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
use crate::memline::ml_open;
use crate::memory::{xcalloc, xfree};
use crate::message::{e_channotpty, e_invarg};
use crate::message::{emsg, emsg_ptr};
use crate::message_fmt::c_str;
use crate::option::vars::p_tgc;
use crate::os::cshim::{gettext, snprintf};
use crate::os::env::{home_replace, os_getenv};
use crate::os::fs::os_isdir;
use crate::os::pty_proc_unix::pty_proc_resize;
use crate::os::shell::shell_free_argv;
use crate::os::time::os_hrtime;
use crate::path::vim_full_name;
use crate::semsg;
use crate::startup::main_loop;
use crate::terminal::{terminal_buf, terminal_open, terminal_running};
use crate::types::AutoEvent;
use crate::types::channel::{kChannelStdinNull, kChannelStdinPipe};
use crate::types::{
    Callback, CallbackReader, Channel, ChannelStdinMode, Dict, EvalFuncData, IOSIZE, Integer, List,
    MAXPATHL, NUL, Object, TypVal, VAR_BOOL, VAR_DICT, VAR_LIST, VAR_NUMBER, VarNumber, Vv,
    uint16_t, uint64_t,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_flush};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

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
    // SAFETY throughout: the frame is live; `find_job` answers with a live channel or
    // null.
    if check_secure() {
        return;
    }
    let Some(id) = job_id(&args[0]) else {
        return;
    };
    let data = unsafe { find_job(id, true) };
    if data.is_null() {
        return;
    }
    result.write_number(unsafe { (*channel_proc(data)).pid } as VarNumber);
}

/// `jobresize({job}, {width}, {height})` — only for a pty job.
pub fn f_jobresize(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    // SAFETY throughout: the frame is live; `find_job` answers with a live channel or
    // null.
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
    let data = unsafe { find_job(args[0].number_or_zero() as uint64_t, true) };
    if data.is_null() {
        return;
    }
    if unsafe { (*channel_proc(data)).type_0 } != kProcTypePty {
        emsg(gettext(e_channotpty));
        return;
    }
    // SAFETY: the tags checked above say both arguments are Numbers, and
    // `data` is the live channel the id resolved to.
    let width = args[1].number_or_zero() as uint16_t;
    let height = args[2].number_or_zero() as uint16_t;
    let pty = unsafe { channel_pty(data) };
    unsafe { pty_proc_resize(pty, width, height) };
    result.write_number(1);
}

/// `jobstop({job})`
pub fn f_jobstop(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    // SAFETY throughout: the frame is live; `find_job` answers with a live channel or
    // null, and `error` is a borrowed static message.
    if check_secure() {
        return;
    }
    let Some(id) = job_id(&args[0]) else {
        return;
    };
    // `false`: a job that has already gone is not an error here.
    let data = unsafe { find_job(id, false) };
    if data.is_null() {
        return;
    }
    let mut error = ptr::null::<c_char>();
    if unsafe { (*data).is_rpc } {
        unsafe { channel_close((*data).id, kChannelPartRpc, &raw mut error) };
    }
    unsafe { proc_stop(channel_proc(data)) };
    // Reported as a success even when closing the RPC half complained.
    result.write_number(1);
    if !error.is_null() {
        unsafe { emsg_ptr(error) };
    }
}

/// `jobwait({jobs} [, {timeout}])`
pub fn f_jobwait(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    // SAFETY throughout: the frame is live; `jobs` is an allocation this body owns for
    // its whole length, and every channel in it holds a reference.
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_LIST
        || (!args.get(1).is_some_and(|arg| arg.v_type() == VAR_NUMBER) && args.len() > 1)
    {
        emsg(gettext(e_invarg));
        return;
    }

    let list: *mut List = args[0].list_or_null();
    let count = list_len(unsafe { list.as_ref() });
    let jobs = unsafe { xcalloc(count as usize, size_of::<*mut Channel>()) } as *mut *mut Channel;
    // The waiting jobs' events are parked on a queue of our own so that
    // they do not run while we block.
    let waiting_jobs = unsafe { multiqueue_new(Some(loop_on_put), main_loop.ptr() as *mut c_void) };

    let mut i = 0;
    {
        for arg in list_iter(unsafe { list.as_ref() }) {
            let chan;
            if arg.li_tv.v_type() != VAR_NUMBER
                || {
                    chan = find_channel(arg.li_tv.number_or_zero() as uint64_t);
                    chan.is_null()
                }
                || unsafe { (*chan).streamtype } != kChannelStreamProc
            {
                // Not a job: reported as -3 below.
                unsafe { *jobs.add(i as usize) = ptr::null_mut() };
            } else if proc_is_stopped(unsafe { &*channel_proc(chan) }) {
                // Already gone; reap it and report -3 as well.
                unsafe { proc_wait(channel_proc(chan), -1, ptr::null_mut()) };
                unsafe { *jobs.add(i as usize) = ptr::null_mut() };
            } else {
                unsafe { *jobs.add(i as usize) = chan };
                unsafe { channel_incref(chan) };
                if unsafe { (*channel_proc(chan)).status } < 0 {
                    unsafe { multiqueue_process_events((*chan).events) };
                    unsafe { multiqueue_replace_parent((*chan).events, waiting_jobs) };
                }
            }
            i += 1;
        }
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

    for i in 0..count {
        if remaining == 0 {
            break;
        }
        if unsafe { *jobs.add(i as usize) }.is_null() {
            continue;
        }
        let status =
            unsafe { proc_wait(channel_proc(*jobs.add(i as usize)), remaining, waiting_jobs) };
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

    let held = tv_list_alloc(count as isize);
    let rv = held.as_ptr();
    for i in 0..count {
        let chan = unsafe { *jobs.add(i as usize) };
        if chan.is_null() {
            unsafe { (*rv).push_number(-3) };
            continue;
        }
        // Hand the parked events back before reporting.
        unsafe { multiqueue_process_events((*chan).events) };
        unsafe { multiqueue_replace_parent((*chan).events, (*main_loop.ptr()).events) };
        unsafe { (*rv).push_number((*channel_proc(chan)).status as VarNumber) };
        unsafe { channel_decref(chan) };
    }

    unsafe { multiqueue_free(waiting_jobs) };
    unsafe { xfree(jobs as *mut c_void) };
    if busy {
        ui_busy_stop();
    }
    result.write_list(Some(held));
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
    job_env: *mut Dict,
    clear_env: bool,
    pty: bool,
    pty_term_name: *const c_char,
) -> *mut Dict {
    // SAFETY: the caller's obligation; every key below is a `'static`
    // NUL-terminated string and the dict owns what it is given.
    let mut env_held = tv_dict_alloc();
    let env = env_held.as_ptr();

    if !clear_env {
        // Start from our own environment. `f_environ` is the builtin,
        // called directly because it is the only thing that knows how
        // to turn `environ` into a Dict.
        let mut inherited = TV_INITIAL_VALUE;
        let out = &raw mut inherited;
        let row = EvalFuncData::None;
        // SAFETY: `out` is this frame's own value.
        f_environ(&[], unsafe { &mut *out }, row);
        unsafe { dict_extend(env, inherited.dict_or_null(), b'f') };
        unsafe { tv_dict_free(inherited.dict_or_null()) };
        // Freed outright rather than released, so the value that named it
        // must give it up without a second release.
        inherited.disown();

        if pty {
            for name in PTY_IGNORED_ENV {
                if let Some(dv) = env_held.find(name.to_bytes()) {
                    // SAFETY: the dictionary this call owns, and its own item.
                    unsafe { tv_dict_item_remove(env, ::core::ptr::from_ref(dv).cast_mut()) };
                }
            }
            // COLORTERM was just removed; put ours back when we know
            // the child can use it.
            if p_tgc() {
                let truecolor = c"truecolor".as_ptr();
                let _ = unsafe { (*env).add_str(b"COLORTERM", truecolor) };
            }
        }
    }

    if pty {
        if let Some(dv) = env_held.find(b"TERM") {
            // SAFETY: the dictionary this call owns, and its own item.
            unsafe { tv_dict_item_remove(env, ::core::ptr::from_ref(dv).cast_mut()) };
        }
        let _ = unsafe { (*env).add_str(b"TERM", pty_term_name) };
    }

    // $NVIM points the child at this instance's server address, when
    // there is one.
    let nvim_addr = get_vim_var_str(Vv::Servername);
    if unsafe { *nvim_addr } as c_int != NUL {
        if let Some(dv) = env_held.find(b"NVIM") {
            // SAFETY: the dictionary this call owns, and its own item.
            unsafe { tv_dict_item_remove(env, ::core::ptr::from_ref(dv).cast_mut()) };
        }
        let _ = unsafe { (*env).add_str(b"NVIM", nvim_addr) };
    }

    // The job's own `env` wins over everything above.
    if !job_env.is_null() {
        // SAFETY: the dictionary this call owns, and the job's own.
        unsafe { dict_extend(env, job_env, b'f') };
    }

    if pty {
        for name in REQUIRED_ENV {
            if env_held.has_key(name.to_bytes()) {
                continue;
            }
            // SAFETY: a NUL-terminated variable name.
            let value = unsafe { os_getenv(name.as_ptr()) };
            if !value.is_null() {
                // SAFETY: `os_getenv` answers an allocation this takes over.
                let _ = unsafe { env_held.add_allocated_str(name.to_bytes(), value) };
            }
        }
    }

    // The caller takes the reference over, and frees it with the process.
    env_held.into_raw()
}

/// `jobstart({cmd} [, {opts}])`
pub fn f_jobstart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut cmdbuf = NumBuf::new();
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    result.write_number(0);
    // SAFETY throughout: the frame is live; `argv` is released on every path that does
    // not hand it to `channel_job_start`, which adopts it.
    if check_secure() {
        return;
    }

    let mut cmd = ptr::null::<c_char>();
    let mut executable = true;
    let argv = unsafe { tv_to_argv(&args[0], &raw mut cmd, &raw mut executable, &mut cmdbuf) };
    if argv.is_null() {
        // A malformed command answers 0; a command that is simply not
        // executable answers -1.
        result.write_number(if executable { 0 } else { -1 });
        return;
    }
    // From here on every early exit must release `argv`.
    macro_rules! bail {
        () => {{
            unsafe { shell_free_argv(argv) };
            return;
        }};
    }

    if !args.get(1).is_some_and(|arg| arg.v_type() == VAR_DICT) && args.len() > 1 {
        let arg0 = "expected dictionary";
        semsg!("E475: Invalid argument: {arg0}");
        bail!();
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
    let mut cwd = ptr::null::<c_char>();
    // The env dictionary itself, not the item holding it: the borrow the
    // lookup answers does not survive `common_job_callbacks`, which takes a
    // reference to the options dictionary it was found in.
    let mut job_env = ptr::null_mut::<Dict>();

    if args.get(1).is_some_and(|arg| arg.v_type() == VAR_DICT) {
        job_opts = args[1].dict_ref();
        detach = dict_get_number(job_opts, b"detach") != 0;
        rpc = dict_get_number(job_opts, b"rpc") != 0;
        term = dict_get_number(job_opts, b"term") != 0;
        pty = term || dict_get_number(job_opts, b"pty") != 0;
        clear_env = dict_get_number(job_opts, b"clear_env") != 0;
        overlapped = dict_get_number(job_opts, b"overlapped") != 0;

        // An unrecognised `stdin` is a warning, not a failure.
        let s = numbuf.dict_string(job_opts, b"stdin");
        if !s.is_null() {
            if unsafe { cstr::prefix_eq(s, c"null".as_ptr(), NUMBUFLEN as usize) } {
                stdin_mode = kChannelStdinNull;
            } else if !unsafe { cstr::prefix_eq(s, c"pipe".as_ptr(), NUMBUFLEN as usize) } {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (arg0, s) = unsafe { (c_str(c"stdin".as_ptr()), c_str(s)) };
                semsg!("E475: Invalid value for argument {arg0}: {s}");
            }
        }

        // `term` is the one option whose *type* is checked, because a
        // truthy string used to mean something else.
        let job_term = dict_find(job_opts, b"term");
        if job_term.is_some_and(|di| di.di_tv.v_type() != VAR_BOOL) {
            let what = c"'term' must be Boolean".as_ptr();
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E475: Invalid argument: {what}");
            bail!();
        }
        if pty && rpc {
            let what = c"job cannot have both 'pty' and 'rpc' options set".as_ptr();
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E475: Invalid argument: {what}");
            bail!();
        }

        let new_cwd = numbuf2.dict_string(job_opts, b"cwd");
        if !new_cwd.is_null() && unsafe { *new_cwd } as c_int != NUL {
            cwd = new_cwd;
            if !unsafe { os_isdir(cwd) } {
                let what = c"expected valid directory".as_ptr();
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let what = unsafe { c_str(what) };
                semsg!("E475: Invalid argument: {what}");
                bail!();
            }
        }

        if let Some(di) = dict_find(job_opts, b"env") {
            if di.di_tv.v_type() != VAR_DICT {
                let arg0 = "env";
                semsg!("E475: Invalid argument: {arg0}");
                bail!();
            }
            job_env = di.di_tv.dict_or_null();
        }

        let out = &raw mut on_stdout;
        let err = &raw mut on_stderr;
        let exit = &raw mut on_exit;
        // SAFETY: `job_opts` is null or a live Dict; the three are locals.
        if !unsafe { common_job_callbacks(args[1].dict_or_null(), out, err, exit) } {
            bail!();
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
    let mut term_name = ptr::null::<c_char>();

    if term {
        if text_locked() {
            text_locked_msg();
            bail!();
        }
        if Buf::current().b_changed != 0 {
            let msg = c"jobstart(...,{term=true}) requires unmodified buffer";
            emsg(gettext(msg));
            bail!();
        }
        if !Buf::current().terminal.is_null() {
            if unsafe { terminal_running(Buf::current().terminal) } {
                let handle = Buf::current().handle;
                semsg!("Terminal already connected to buffer {}", handle);
                bail!();
            }
            buf_close_terminal(Buf::current());
        }
        // `pty && rpc` was refused above and `term` implies `pty`.
        debug_assert!(!rpc);

        term_name = c"xterm-256color".as_ptr();
        if cwd.is_null() {
            cwd = c".".as_ptr();
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
    if pty && term_name.is_null() {
        term_name = numbuf3.dict_string(job_opts, b"TERM");
        if term_name.is_null() {
            term_name = c"ansi".as_ptr();
        }
    }

    let env = unsafe { create_environment(job_env, clear_env, pty, term_name) };
    // `channel_job_start` answers the channel id, or the reason it failed,
    // through this slot on every path; it lands in the return value below.
    let mut status: VarNumber = 0;
    let status_out = &raw mut status;
    // SAFETY: `argv` is a NUL-terminated vector this frame owns, `env` the
    // environment built above, and `status_out` a local.
    // The fifteen arguments are what upstream's `channel_job_start` takes;
    // there is no shorter way to write the call.
    let chan = unsafe {
        channel_job_start(
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
            status_out,
        )
    };
    result.write_number(status);
    if chan.is_null() {
        return;
    }
    if !term {
        unsafe { channel_create_event(chan, ptr::null()) };
        return;
    }
    if result.number_or_zero() <= 0 {
        return;
    }
    unsafe { attach_terminal(chan, cwd, cmd) };
}

/// Give a `{term: v:true}` job the current buffer.
///
/// # Safety
/// `chan` is a live channel with a running process, `cwd` and `cmd` are
/// NUL-terminated strings.
unsafe fn attach_terminal(chan: *mut Channel, cwd: *const c_char, cmd: *const c_char) {
    // The autocommands below run while the name is half-built, which is why
    // neither of these is the shared scratch buffer upstream uses.
    let mut name = [0 as c_char; MAXPATHL as usize];
    let mut shortened = [0 as c_char; IOSIZE as usize];
    // SAFETY: the caller's obligation; both buffers outlive every call they
    // are handed to below.
    let pid = unsafe { (*channel_proc(chan)).pid };
    let mut buf = Buf::current();
    buf.b_p_swf = 0;
    if buf.b_ml.ml_mfp.is_null() && ml_open(buf).is_err() {
        unsafe { proc_stop(channel_proc(chan)) };
        unsafe { channel_decref(chan) };
        return;
    }
    unsafe { channel_incref(chan) };
    unsafe { channel_terminal_alloc(buf, chan) };
    let noname = ptr::null_mut::<c_char>();
    unsafe { apply_autocmds(AutoEvent::BufFilePre, noname, noname, false, Some(buf)) };

    // The autocommand may have closed the terminal out from under us,
    // which is what each of these three re-tests is for.
    if unsafe { terminal_live(chan) } {
        // Name the buffer `term://{cwd}//{pid}:{cmd}`.
        let _ = unsafe { vim_full_name(cwd, name.as_mut_ptr(), MAXPATHL as usize, false) };
        let (src, dst) = (name.as_mut_ptr(), shortened.as_mut_ptr());
        let len = unsafe { home_replace(None, src, dst, IOSIZE as usize, true) };
        // Drop a trailing separator, but keep `/` itself meaningful by
        // spelling it `/.`.
        if len != 1 && matches!(shortened[len - 1] as u8, b'\\' | b'/') {
            shortened[len - 1] = NUL as c_char;
        }
        if len == 1 && shortened[0] as u8 == b'/' {
            shortened[1] = b'.' as c_char;
            shortened[2] = NUL as c_char;
        }
        let out = name.as_mut_ptr();
        let fmt = c"term://%s//%d:%s".as_ptr();
        let dir = shortened.as_ptr();
        unsafe { snprintf(out, MAXPATHL as usize, fmt, dir, pid, cmd) };
        let _ = setfname(buf, Some(cstr::in_chars(&name)), None, true);
        unsafe { apply_autocmds(AutoEvent::BufFilePost, noname, noname, false, Some(buf)) };

        if unsafe { terminal_live(chan) } {
            // Locked so that the two variables cannot be swapped out
            // from under the terminal by a BufFilePost autocommand.
            buf.b_locked += 1;
            unsafe { set_buf_var(buf, c"terminal_job_id", (*chan).id as Integer) };
            set_buf_var(buf, c"terminal_job_pid", pid as Integer);
            buf.b_locked -= 1;

            if unsafe { terminal_live(chan) } {
                unsafe { terminal_open(&raw mut (*chan).term, buf) };
            }
        }
    }

    unsafe { channel_create_event(chan, ptr::null()) };
    unsafe { channel_decref(chan) };
}

/// Whether the channel still has a terminal attached to a real buffer.
///
/// # Safety
/// `chan` is a live channel.
unsafe fn terminal_live(chan: *mut Channel) -> bool {
    // SAFETY: the caller's obligation.
    unsafe { !(*chan).term.is_null() && terminal_buf((*chan).term) != 0 }
}

/// Set one buffer-local variable to an Integer, discarding any refusal.
fn set_buf_var(buffer: Buf, name: &CStr, value: Integer) {
    let value = Object::Integer(value);
    // SAFETY: the caller's obligation; the name is `'static`.
    let vars = buffer.b_vars;
    let name = unsafe { cstr_to_string(name.as_ptr()) };
    drop(unsafe { dict_set_var(vars, &name, value, false, false) });
}
