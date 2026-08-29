//! Child processes: the `job*()` family and the environment it hands them.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{
    Callback_data, GA_EMPTY_INIT_VALUE, NUMBUFLEN, f_environ, kChannelPartRpc, kChannelStreamProc,
    kProcTypePty, object_data,
};
use crate::api::private::helpers::{api_clear_error, cstr_as_string, dict_set_var};
use crate::autocmd::{EVENT_BUFFILEPOST, EVENT_BUFFILEPRE, apply_autocmds};
use crate::buffer::{buf_close_terminal, setfname};
use crate::channel::{
    channel_close, channel_create_event, channel_decref, channel_incref, channel_job_start,
    channel_proc, channel_pty, channel_terminal_alloc, find_channel,
};
use crate::eval::typval::{
    NumBuf, kCallbackNone, tv_dict_add_allocated_str, tv_dict_add_str, tv_dict_alloc,
    tv_dict_extend, tv_dict_find, tv_dict_free, tv_dict_get_number, tv_dict_item_remove,
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
use crate::main::{curbuf, curwin, e_channotpty, e_invarg, main_loop, p_tgc};
use crate::memline::ml_open;
use crate::memory::{xcalloc, xfree};
use crate::message::{emsg, emsg_ptr};
use crate::message_fmt::c_str;
use crate::r#move::win_col_off;
use crate::os::cshim::{gettext, gettext_ptr, snprintf, strncmp};
use crate::os::env::{home_replace, os_getenv};
use crate::os::fs::os_isdir;
use crate::os::pty_proc_unix::pty_proc_resize;
use crate::os::shell::shell_free_argv;
use crate::os::time::os_hrtime;
use crate::path::vim_full_name;
use crate::semsg;
use crate::semsg_c;
use crate::terminal::{terminal_buf, terminal_open, terminal_running};
use crate::types::channel::{kChannelStdinNull, kChannelStdinPipe};
use crate::types::{
    Arena, Callback, CallbackReader, Channel, ChannelStdinMode, Error, EvalFuncData, FAIL, IOSIZE,
    Integer, MAXPATHL, NUL, VAR_BOOL, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_UNKNOWN, VarLock, Vv,
    buf_T, dict_T, dictitem_T, kErrorTypeNone, kObjectTypeInteger, list_T, listitem_T, object,
    typval_T, typval_vval_union, uint16_t, uint64_t, varnumber_T,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_flush};
use crate::winlayer::Buf;
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
    data: Callback_data {
        funcref: ptr::null_mut::<c_char>(),
    },
    type_0: kCallbackNone,
};

/// The job id a `job*()` builtin was handed, or `None` when the argument
/// was not a Number at all -- in which case the error is already out.
fn job_id(arg: &typval_T) -> Option<uint64_t> {
    if arg.v_type != VAR_NUMBER {
        emsg(gettext(e_invarg));
        return None;
    }
    // SAFETY: the type tag names the union member.
    Some(unsafe { arg.vval.v_number } as uint64_t)
}

/// `jobpid({job})`
pub unsafe fn f_jobpid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY throughout: the frame is live; `find_job` answers with a live channel or
    // null.
    if check_secure() {
        return;
    }
    let Some(id) = job_id(args.get(0)) else {
        return;
    };
    let data = unsafe { find_job(id, true) };
    if data.is_null() {
        return;
    }
    rettv.vval.v_number = unsafe { (*channel_proc(data)).pid } as varnumber_T;
}

/// `jobresize({job}, {width}, {height})` — only for a pty job.
pub unsafe fn f_jobresize(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY throughout: the frame is live; `find_job` answers with a live channel or
    // null.
    if check_secure() {
        return;
    }
    // All three arguments are checked together, so a bad width reports
    // the same message a bad job id does.
    if args.ty(0) != VAR_NUMBER || args.ty(1) != VAR_NUMBER || args.ty(2) != VAR_NUMBER {
        emsg(gettext(e_invarg));
        return;
    }
    let data = unsafe { find_job(args.get(0).vval.v_number as uint64_t, true) };
    if data.is_null() {
        return;
    }
    if unsafe { (*channel_proc(data)).type_0 } != kProcTypePty {
        emsg(gettext(e_channotpty));
        return;
    }
    // SAFETY: the tags checked above say both arguments are Numbers, and
    // `data` is the live channel the id resolved to.
    let width = unsafe { args.get(1).vval.v_number } as uint16_t;
    let height = unsafe { args.get(2).vval.v_number } as uint16_t;
    let pty = unsafe { channel_pty(data) };
    unsafe { pty_proc_resize(pty, width, height) };
    rettv.vval.v_number = 1;
}

/// `jobstop({job})`
pub unsafe fn f_jobstop(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY throughout: the frame is live; `find_job` answers with a live channel or
    // null, and `error` is a borrowed static message.
    if check_secure() {
        return;
    }
    let Some(id) = job_id(args.get(0)) else {
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
    rettv.vval.v_number = 1;
    if !error.is_null() {
        unsafe { emsg_ptr(error) };
    }
}

/// `jobwait({jobs} [, {timeout}])`
pub unsafe fn f_jobwait(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY throughout: the frame is live; `jobs` is an allocation this body owns for
    // its whole length, and every channel in it holds a reference.
    if check_secure() {
        return;
    }
    if args.ty(0) != VAR_LIST || (args.ty(1) != VAR_NUMBER && args.has(1)) {
        emsg(gettext(e_invarg));
        return;
    }

    let list: *mut list_T = unsafe { args.get(0).vval.v_list };
    let count = unsafe { tv_list_len(list) };
    let jobs = unsafe { xcalloc(count as usize, size_of::<*mut Channel>()) } as *mut *mut Channel;
    // The waiting jobs' events are parked on a queue of our own so that
    // they do not run while we block.
    let waiting_jobs = unsafe { multiqueue_new(Some(loop_on_put), main_loop.ptr() as *mut c_void) };

    let mut i = 0;
    if !list.is_null() {
        let mut arg: *const listitem_T = unsafe { (*list).lv_first };
        while !arg.is_null() {
            let mut chan = ptr::null_mut::<Channel>();
            if unsafe { (*arg).li_tv.v_type } != VAR_NUMBER
                || {
                    chan = find_channel(unsafe { (*arg).li_tv.vval.v_number } as uint64_t);
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
            arg = unsafe { (*arg).li_next };
        }
    }

    // A negative or absent timeout means "no limit".
    let mut remaining = -1;
    let mut before = 0u64;
    if args.ty(1) == VAR_NUMBER && unsafe { args.get(1).vval.v_number } >= 0 {
        remaining = unsafe { args.get(1).vval.v_number } as c_int;
        before = os_hrtime();
    }
    // Only mark the UI busy when this actually blocks.
    let busy = remaining != 0;
    if busy {
        ui_busy_start();
        unsafe { ui_flush() };
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

    let rv = unsafe { tv_list_alloc(count as isize) };
    for i in 0..count {
        let chan = unsafe { *jobs.add(i as usize) };
        if chan.is_null() {
            unsafe { tv_list_append_number(rv, -3) };
            continue;
        }
        // Hand the parked events back before reporting.
        unsafe { multiqueue_process_events((*chan).events) };
        unsafe { multiqueue_replace_parent((*chan).events, (*main_loop.ptr()).events) };
        unsafe { tv_list_append_number(rv, (*channel_proc(chan)).status as varnumber_T) };
        unsafe { channel_decref(chan) };
    }

    unsafe { multiqueue_free(waiting_jobs) };
    unsafe { xfree(jobs as *mut c_void) };
    if busy {
        ui_busy_stop();
    }
    unsafe { tv_list_ref(rv) };
    rettv.v_type = VAR_LIST;
    rettv.vval.v_list = rv;
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
    let env = unsafe { tv_dict_alloc() };

    if !clear_env {
        // Start from our own environment. `f_environ` is the builtin,
        // called directly because it is the only thing that knows how
        // to turn `environ` into a Dict.
        let mut inherited = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_number: 0 },
        };
        let out = &raw mut inherited;
        let row = EvalFuncData {
            null: ptr::null_mut(),
        };
        // SAFETY: `f_environ` reads no arguments and fills `inherited`.
        unsafe { f_environ(ptr::null_mut(), out, row) };
        unsafe { tv_dict_extend(env, inherited.vval.v_dict, c"force".as_ptr()) };
        unsafe { tv_dict_free(inherited.vval.v_dict) };

        if pty {
            for name in PTY_IGNORED_ENV {
                let dv = unsafe { tv_dict_find(env, name.as_ptr(), -1) };
                if !dv.is_null() {
                    unsafe { tv_dict_item_remove(env, dv) };
                }
            }
            // COLORTERM was just removed; put ours back when we know
            // the child can use it.
            if p_tgc.get() != 0 {
                unsafe { tv_dict_add_str(env, c"COLORTERM".as_ptr(), 9, c"truecolor".as_ptr()) };
            }
        }
    }

    if pty {
        let dv = unsafe { tv_dict_find(env, c"TERM".as_ptr(), 4) };
        if !dv.is_null() {
            unsafe { tv_dict_item_remove(env, dv) };
        }
        unsafe { tv_dict_add_str(env, c"TERM".as_ptr(), 4, pty_term_name) };
    }

    // $NVIM points the child at this instance's server address, when
    // there is one.
    let nvim_addr = unsafe { get_vim_var_str(Vv::Servername) };
    if unsafe { *nvim_addr } as c_int != NUL {
        let dv = unsafe { tv_dict_find(env, c"NVIM".as_ptr(), 4) };
        if !dv.is_null() {
            unsafe { tv_dict_item_remove(env, dv) };
        }
        unsafe { tv_dict_add_str(env, c"NVIM".as_ptr(), 4, nvim_addr) };
    }

    // The job's own `env` wins over everything above.
    if !job_env.is_null() {
        unsafe { tv_dict_extend(env, (*job_env).di_tv.vval.v_dict, c"force".as_ptr()) };
    }

    if pty {
        for name in REQUIRED_ENV {
            let len = name.count_bytes();
            if unsafe { tv_dict_find(env, name.as_ptr(), len as isize) }.is_null() {
                let value = unsafe { os_getenv(name.as_ptr()) };
                if !value.is_null() {
                    unsafe { tv_dict_add_allocated_str(env, name.as_ptr(), len, value) };
                }
            }
        }
    }

    env
}

/// `jobstart({cmd} [, {opts}])`
pub unsafe fn f_jobstart(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut cmdbuf = NumBuf::new();
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY throughout: the frame is live; `argv` is released on every path that does
    // not hand it to `channel_job_start`, which adopts it.
    if check_secure() {
        return;
    }

    let mut cmd = ptr::null::<c_char>();
    let mut executable = true;
    let argv = unsafe { tv_to_argv(args.ptr(0), &raw mut cmd, &raw mut executable, &mut cmdbuf) };
    if argv.is_null() {
        // A malformed command answers 0; a command that is simply not
        // executable answers -1.
        rettv.vval.v_number = if executable { 0 } else { -1 };
        return;
    }
    // From here on every early exit must release `argv`.
    macro_rules! bail {
        () => {{
            unsafe { shell_free_argv(argv) };
            return;
        }};
    }

    if args.ty(1) != VAR_DICT && args.has(1) {
        let arg0 = "expected dictionary";
        semsg!("E475: Invalid argument: {arg0}");
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
    let mut cwd = ptr::null::<c_char>();
    let mut job_env = ptr::null_mut::<dictitem_T>();

    if args.ty(1) == VAR_DICT {
        job_opts = unsafe { args.get(1).vval.v_dict };
        detach = unsafe { tv_dict_get_number(job_opts, c"detach".as_ptr()) } != 0;
        rpc = unsafe { tv_dict_get_number(job_opts, c"rpc".as_ptr()) } != 0;
        term = unsafe { tv_dict_get_number(job_opts, c"term".as_ptr()) } != 0;
        pty = term || unsafe { tv_dict_get_number(job_opts, c"pty".as_ptr()) } != 0;
        clear_env = unsafe { tv_dict_get_number(job_opts, c"clear_env".as_ptr()) } != 0;
        overlapped = unsafe { tv_dict_get_number(job_opts, c"overlapped".as_ptr()) } != 0;

        // An unrecognised `stdin` is a warning, not a failure.
        let s = unsafe { numbuf.dict_string(job_opts, c"stdin".as_ptr()) };
        if !s.is_null() {
            if unsafe { strncmp(s, c"null".as_ptr(), NUMBUFLEN as usize) } == 0 {
                stdin_mode = kChannelStdinNull;
            } else if unsafe { strncmp(s, c"pipe".as_ptr(), NUMBUFLEN as usize) } != 0 {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (arg0, s) = unsafe { (c_str(c"stdin".as_ptr()), c_str(s)) };
                semsg!("E475: Invalid value for argument {arg0}: {s}");
            }
        }

        // `term` is the one option whose *type* is checked, because a
        // truthy string used to mean something else.
        let job_term = unsafe { tv_dict_find(job_opts, c"term".as_ptr(), 4) };
        if !job_term.is_null() && unsafe { (*job_term).di_tv.v_type } != VAR_BOOL {
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

        let new_cwd = unsafe { numbuf2.dict_string(job_opts, c"cwd".as_ptr()) };
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

        job_env = unsafe { tv_dict_find(job_opts, c"env".as_ptr(), 3) };
        if !job_env.is_null() && unsafe { (*job_env).di_tv.v_type } != VAR_DICT {
            let arg0 = "env";
            semsg!("E475: Invalid argument: {arg0}");
            bail!();
        }

        let out = &raw mut on_stdout;
        let err = &raw mut on_stderr;
        let exit = &raw mut on_exit;
        // SAFETY: `job_opts` is null or a live Dict; the three are locals.
        if !unsafe { common_job_callbacks(job_opts, out, err, exit) } {
            bail!();
        }
    }

    // `tv_dict_get_number` accepts a null dict, so these two are read
    // whether or not there were options at all.
    let mut width = unsafe { tv_dict_get_number(job_opts, c"width".as_ptr()) } as uint16_t;
    let mut height = unsafe { tv_dict_get_number(job_opts, c"height".as_ptr()) } as uint16_t;
    let mut term_name = ptr::null::<c_char>();

    if term {
        if unsafe { text_locked() } {
            unsafe { text_locked_msg() };
            bail!();
        }
        if unsafe { (*curbuf.get()).b_changed } != 0 {
            let msg = c"jobstart(...,{term=true}) requires unmodified buffer";
            emsg(gettext(msg));
            bail!();
        }
        if !unsafe { (*curbuf.get()).terminal }.is_null() {
            if unsafe { terminal_running((*curbuf.get()).terminal) } {
                let fmt = c"Terminal already connected to buffer %d".as_ptr();
                let handle = unsafe { (*curbuf.get()).handle };
                unsafe { semsg_c!(gettext_ptr(fmt), handle) };
                bail!();
            }
            buf_close_terminal(unsafe { Buf::current() });
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
            width = (unsafe { (*curwin.get()).w_view_width } - unsafe { win_col_off(curwin.get()) })
                .max(0) as uint16_t;
        }
        if height == 0 {
            height = unsafe { (*curwin.get()).w_view_height } as uint16_t;
        }
    }
    if pty && term_name.is_null() {
        term_name = unsafe { numbuf3.dict_string(job_opts, c"TERM".as_ptr()) };
        if term_name.is_null() {
            term_name = c"ansi".as_ptr();
        }
    }

    let env = unsafe { create_environment(job_env, clear_env, pty, term_name) };
    let pid_out = &raw mut rettv.vval.v_number;
    // SAFETY: `argv` is a NUL-terminated vector this frame owns, `env` the
    // environment built above, and `pid_out` the return value's own slot.
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
            pid_out,
        )
    };
    if chan.is_null() {
        return;
    }
    if !term {
        unsafe { channel_create_event(chan, ptr::null()) };
        return;
    }
    if unsafe { rettv.vval.v_number } <= 0 {
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
    let buf = curbuf.get();
    unsafe { (*buf).b_p_swf = 0 };
    if unsafe { (*buf).b_ml.ml_mfp }.is_null() && unsafe { ml_open(buf) } == FAIL {
        unsafe { proc_stop(channel_proc(chan)) };
        unsafe { channel_decref(chan) };
        return;
    }
    unsafe { channel_incref(chan) };
    unsafe { channel_terminal_alloc(buf, chan) };
    let noname = ptr::null_mut::<c_char>();
    unsafe { apply_autocmds(EVENT_BUFFILEPRE, noname, noname, false, buf) };

    // The autocommand may have closed the terminal out from under us,
    // which is what each of these three re-tests is for.
    if unsafe { terminal_live(chan) } {
        // Name the buffer `term://{cwd}//{pid}:{cmd}`.
        unsafe { vim_full_name(cwd, name.as_mut_ptr(), MAXPATHL as usize, false) };
        let (src, dst) = (name.as_mut_ptr(), shortened.as_mut_ptr());
        let len = unsafe { home_replace(ptr::null(), src, dst, IOSIZE as usize, true) };
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
        unsafe { setfname(Buf::new(buf), name.as_mut_ptr(), ptr::null_mut(), true) };
        unsafe { apply_autocmds(EVENT_BUFFILEPOST, noname, noname, false, buf) };

        if unsafe { terminal_live(chan) } {
            let mut err = Error {
                type_0: kErrorTypeNone,
                msg: ptr::null_mut(),
            };
            // Locked so that the two variables cannot be swapped out
            // from under the terminal by a BufFilePost autocommand.
            unsafe { (*buf).b_locked += 1 };
            unsafe { set_buf_var(buf, c"terminal_job_id", (*chan).id as Integer, &raw mut err) };
            unsafe { set_buf_var(buf, c"terminal_job_pid", pid as Integer, &raw mut err) };
            unsafe { (*buf).b_locked -= 1 };

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

/// Set one buffer-local variable to an Integer, discarding any error.
///
/// # Safety
/// `buf` is a live buffer and `err` a live out-parameter.
unsafe fn set_buf_var(buf: *mut buf_T, name: &CStr, value: Integer, err: *mut Error) {
    let value = object {
        type_0: kObjectTypeInteger,
        data: object_data { integer: value },
    };
    let arena = ptr::null_mut::<Arena>();
    // SAFETY: the caller's obligation; the name is `'static`.
    let vars = unsafe { (*buf).b_vars };
    let name = unsafe { cstr_as_string(name.as_ptr()) };
    unsafe { dict_set_var(vars, name, value, false, false, arena, err) };
    unsafe { api_clear_error(err) };
}
