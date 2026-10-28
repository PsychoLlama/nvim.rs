//! The four ways a channel comes into being.
//!
//! A job (a spawned child, optionally behind a pty), a socket this process
//! dialled, a socket it accepted, or this process's own standard streams.
//! Each ends by registering the channel and announcing it; the failure paths
//! before that point unwind the half-built channel with
//! [`channel_destroy_early`], which gives its id back.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::eval::typval::DictRef;
use crate::event::channel::{
    JobSpawn, Reads, accept_socket, attach_socket, connect_socket, init_job_streams, open_stdio,
    owner_data, spawn_job, start_reads, unopened_rstream, unopened_stdio,
};
use crate::event::libuv_proc::libuv_proc_init;
use crate::event::r#loop::main_loop_raw;
use crate::event::proc::exit_on_closed_chan;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::message_fmt::msg_cstr;
use crate::msgpack_rpc::channel::rpc_start;
use crate::msgpack_rpc::server::server_owns_pipe_address;
use crate::os::cshim::gettext;
use crate::os::fs::os_strerror;
use crate::os::proc::{os_dup_above, os_dup2};
use crate::os::pty_proc_unix::pty_proc_init;
use crate::os::shell::Argv;
use crate::semsg;
use crate::startup::{embedded_mode, exiting, headless_mode, ui_client_channel_id};
use crate::terminal::{terminal_close, terminal_set_state};
use crate::types::channel::kChannelStdinPipe;
use crate::types::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::types::{
    Callback, CallbackReader, ChannelStdinMode, ChannelStream, InternalState, JobProc, LuaRef,
    SocketWatcher, VarNumber, uint16_t, uint64_t,
};
use crate::ui_client::ui_client_attach_to_restarted_server;

use super::known::*;
use super::reader::{callback_reader_start, schedule_channel_event};
use super::{
    alloc_channel, channel_create_event, channel_decref, channel_destroy_early, with_channel,
};

/// Whether stdio has already been claimed. Only one channel may own it.
static did_stdio: GlobalCell<bool> = GlobalCell::new(false);

/// What a job is started with: [`channel_job_start`]'s arguments.
pub struct JobOptions<'a> {
    /// The command line, the program first.
    pub argv: Argv,
    /// The program to run, when not `argv[0]` looked up in `$PATH`.
    pub exepath: Option<&'a CStr>,
    pub on_stdout: CallbackReader,
    pub on_stderr: CallbackReader,
    pub on_exit: Callback,
    pub pty: bool,
    pub rpc: bool,
    pub overlapped: bool,
    pub detach: bool,
    pub stdin_mode: ChannelStdinMode,
    pub cwd: Option<&'a CStr>,
    /// A pty's size; 0 keeps the default.
    pub pty_width: uint16_t,
    pub pty_height: uint16_t,
    /// The environment; `None` inherits this process's.
    pub env: Option<DictRef>,
}

impl<'a> JobOptions<'a> {
    /// A plain job running `argv`: pipes, no callbacks, no pty.
    pub fn new(argv: Argv) -> Self {
        JobOptions {
            argv,
            exepath: None,
            on_stdout: CallbackReader::none(),
            on_stderr: CallbackReader::none(),
            on_exit: Callback::None,
            pty: false,
            rpc: false,
            overlapped: false,
            detach: false,
            stdin_mode: kChannelStdinPipe,
            cwd: None,
            pty_width: 0,
            pty_height: 0,
            env: None,
        }
    }
}

/// `rpcstart()`'s spawn: `argv` (the program first) started as a job that
/// speaks RPC over its pipes, its channel announced. Answers the channel id,
/// or why the spawn failed, as [`channel_job_start`]'s status does.
pub(crate) fn rpc_job_start(argv: &[&CStr]) -> VarNumber {
    let words = argv
        .iter()
        .map(|arg| crate::memory::ThinCString::from_cstr(arg))
        .collect();
    let mut opts = JobOptions::new(Argv::from_words(words));
    opts.rpc = true;
    match channel_job_start(opts) {
        Ok(id) => {
            channel_create_event(id, None);
            id.cast_signed()
        }
        Err(status) => status,
    }
}

/// Spawns a child process and wires a channel to it, answering its id.
///
/// The error is the status `jobstart()` reports: 0 when the arguments were
/// rejected, otherwise the spawn's (negative) libuv error.
pub fn channel_job_start(opts: JobOptions<'_>) -> Result<uint64_t, VarNumber> {
    /// A detached child has no controlling terminal to hand a pty to.
    const PTY_DETACHED: &CStr = c"terminal/pty job cannot be detached";

    let JobOptions {
        argv,
        exepath,
        on_stdout,
        on_stderr,
        on_exit,
        pty,
        rpc,
        overlapped,
        detach,
        stdin_mode,
        cwd,
        pty_width,
        pty_height,
        env,
    } = opts;

    // The channel is this function's alone until the spawn succeeds, so the
    // unwind paths below may destroy it outright.
    let id = alloc_channel(None, |id| {
        let data = owner_data(id);
        ChannelStream::Job(if pty {
            let mut pty = pty_proc_init(main_loop_raw(), data);
            if pty_width > 0 {
                pty.width = pty_width;
            }
            if pty_height > 0 {
                pty.height = pty_height;
            }
            JobProc::Pty(pty)
        } else {
            JobProc::Uv(libuv_proc_init(main_loop_raw(), data))
        })
    });
    let (has_out, has_err) = with_channel(id, |channel| {
        channel.on_data = on_stdout;
        channel.on_stderr = on_stderr;
        channel.on_exit = on_exit;
        // A pty multiplexes both directions onto the master, so it always
        // reads and never has a separate stderr.
        if pty {
            (true, false)
        } else {
            (rpc || channel.on_data.is_set(), channel.on_stderr.is_set())
        }
    })
    .expect("the channel just made");

    if pty && detach {
        let why = PTY_DETACHED.to_string_lossy();
        semsg!("E475: Invalid argument: {why}");
        drop((argv, env));
        channel_destroy_early(id);
        return Err(0);
    }
    let has_in = stdin_mode == kChannelStdinPipe;

    // The name is copied because the failure message is formatted after
    // the spawn has taken ownership of `argv`.
    let cmd: CString = exepath.unwrap_or_else(|| argv.program()).to_owned();
    let spec = JobSpawn {
        argv,
        exepath,
        cwd,
        env,
        detach,
        overlapped,
        // A pty has no separate stderr to forward.
        fwd_err: !pty && with_channel(id, |channel| channel.on_stderr.fwd_err).unwrap_or(false),
        has_in,
        has_out,
        has_err,
    };
    if let Err(status) = spawn_job(id, spec) {
        let (why, cmd) = (msg_cstr(os_strerror(status)), msg_cstr(&cmd));
        semsg!("E903: Process failed to start: {why}: \"{cmd}\"");
        let status = with_channel(id, |channel| match &channel.stream {
            ChannelStream::Job(job) => VarNumber::from(job.proc().status),
            _ => VarNumber::from(status),
        })
        .unwrap_or(VarNumber::from(status));
        channel_destroy_early(id);
        return Err(status);
    }

    init_job_streams(id, has_in, has_out);
    if rpc {
        rpc_start(id);
    } else if has_out {
        with_channel(id, |channel| {
            callback_reader_start(&mut channel.on_data, c"stdout");
        });
        start_reads(id, Reads::Output);
    }
    if has_err {
        with_channel(id, |channel| {
            callback_reader_start(&mut channel.on_stderr, c"stderr");
        });
        start_reads(id, Reads::Stderr);
    }
    Ok(id)
}

/// Connects to a socket (or, for an address this process is itself listening
/// on, to itself) and answers the channel id, or why it could not.
///
/// The event loop must be up, which it is from `main()` on.
pub fn channel_connect(
    tcp: bool,
    address: &CStr,
    rpc: bool,
    on_output: CallbackReader,
    timeout: c_int,
) -> Result<uint64_t, &'static CStr> {
    // Talking to our own listening socket would deadlock: the reply cannot
    // be read while this process is blocked writing the request. An
    // internal channel short-circuits the transport instead.
    if !tcp && rpc && server_owns_pipe_address(address) {
        let id = alloc_channel(None, |_| {
            ChannelStream::Internal(InternalState {
                cb: LUA_NOREF as LuaRef,
                closed: false,
            })
        });
        rpc_start(id);
        channel_create_event(id, Some(address));
        return Ok(id);
    }

    let id = alloc_channel(None, |_| ChannelStream::Socket(unopened_rstream()));
    if let Err(why) = connect_socket(id, tcp, address, timeout) {
        channel_decref(id);
        return Err(why);
    }
    attach_socket(id);
    if rpc {
        rpc_start(id);
    } else {
        with_channel(id, |channel| {
            channel.on_data = on_output;
            callback_reader_start(&mut channel.on_data, c"data");
        });
        start_reads(id, Reads::Output);
    }
    channel_create_event(id, Some(address));
    Ok(id)
}

/// Takes over an accepted connection from a listening socket. Always RPC.
pub fn channel_from_connection(watcher: &mut SocketWatcher) {
    let id = alloc_channel(None, |_| ChannelStream::Socket(unopened_rstream()));
    if let Err(result) = accept_socket(id, watcher) {
        // The client handle is already closed; the channel never got going.
        logmsg!(
            LOGLVL_ERR,
            c"channel_from_connection",
            529,
            "Failed to accept connection: {}",
            os_strerror(result).to_string_lossy()
        );
        channel_destroy_early(id);
        return;
    }
    attach_socket(id);
    rpc_start(id);
    let address = crate::cstr::in_chars(&watcher.addr).to_owned();
    channel_create_event(id, Some(&address));
}

/// Wires a channel to this process's own stdin and stdout.
///
/// Only one may exist, and only when nothing else is using the terminal;
/// answers the channel id, or why not. The event loop must be up, which it
/// is from `main()` on.
pub fn channel_from_stdio(rpc: bool, on_output: CallbackReader) -> Result<uint64_t, &'static CStr> {
    let refusal = if !headless_mode.get() && !embedded_mode.get() {
        Some(c"can only be opened in headless mode")
    } else if did_stdio.get() {
        Some(c"channel was already open")
    } else {
        None
    };
    if let Some(msg) = refusal {
        return Err(gettext(msg));
    }
    did_stdio.set(true);

    let id = alloc_channel(Some(CHAN_STDIO), |_| ChannelStream::Stdio(unopened_stdio()));
    let (stdin_fd, stdout_fd) = if embedded_mode.get() {
        // The embedder owns fds 0 and 1; move them aside and point what is
        // left at stderr, so a stray `print` cannot corrupt the protocol.
        let stdin_fd = os_dup_above(STDIN_FILENO, STDERR_FILENO + 1);
        let stdout_fd = os_dup_above(STDOUT_FILENO, STDERR_FILENO + 1);
        os_dup2(STDERR_FILENO, STDOUT_FILENO);
        os_dup2(STDERR_FILENO, STDIN_FILENO);
        (stdin_fd, stdout_fd)
    } else {
        (STDIN_FILENO, STDOUT_FILENO)
    };
    open_stdio(id, stdin_fd, stdout_fd);
    if rpc {
        rpc_start(id);
    } else {
        with_channel(id, |channel| {
            channel.on_data = on_output;
            callback_reader_start(&mut channel.on_data, c"stdin");
        });
        start_reads(id, Reads::Output);
    }
    Ok(id)
}

// -------------------------------------------------------------------------
// Process callbacks
// -------------------------------------------------------------------------

/// Job channel `id`'s child exited, or its handles closed. `status` is
/// negative for the latter, which is what "closed without an exit status"
/// means here.
pub(crate) fn job_exited(id: uint64_t, status: c_int) {
    // `TermClose` runs from here, with nothing of the channel borrowed.
    if let Some(term) = with_channel(id, |channel| channel.term).flatten() {
        terminal_close(term, status);
    }
    // A UI client whose server died: try to reconnect before following it.
    if !exiting.get() && ui_client_channel_id.get() == id {
        ui_client_attach_to_restarted_server();
        if ui_client_channel_id.get() == id {
            exit_on_closed_chan(status);
        }
    }
    let exited = status >= 0;
    let wants_exit = with_channel(id, |channel| channel.on_exit.is_set()).unwrap_or(false);
    if exited && wants_exit {
        schedule_channel_event(id);
    }
    if exited {
        with_channel(id, |channel| channel.exit_status = status);
    }
    // The child's own reference.
    channel_decref(id);
}

/// Job channel `id`'s child was stopped or continued; only a terminal cares.
pub(crate) fn job_state_changed(id: uint64_t, suspended: bool) {
    if let Some(term) = with_channel(id, |channel| channel.term).flatten() {
        terminal_set_state(term, suspended);
    }
}
