//! The four ways a channel comes into being.
//!
//! A job (a spawned child, optionally behind a pty), a socket this process
//! dialled, a socket it accepted, or this process's own standard streams.
//! Each ends by registering the channel and announcing it; the failure paths
//! before that point unwind the half-built channel with
//! [`channel_destroy_early`], which gives its id back.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::eval::typval::tv_dict_free;
use crate::event::libuv::uv_strerror;
use crate::event::libuv_proc::libuv_proc_init;
use crate::event::proc::{exit_on_closed_chan, proc_get_exepath, proc_spawn};
use crate::event::rstream::{rstream_init, rstream_init_fd, rstream_start};
use crate::event::socket::{socket_connect, socket_watcher_accept};
use crate::event::wstream::{wstream_init, wstream_init_fd};
use crate::global_cell::GlobalCell;
use crate::main::{embedded_mode, exiting, headless_mode, ui_client_channel_id};
use crate::memory::{xfree, xstrdup};
use crate::msgpack_rpc::channel::rpc_start;
use crate::msgpack_rpc::server::server_owns_pipe_address;
use crate::os::cshim::gettext;
use crate::os::pty_proc_unix::pty_proc_init;
use crate::os::shell::shell_free_argv;
use crate::terminal::{terminal_close, terminal_set_state};
use crate::types::channel::kChannelStdinPipe;
use crate::types::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::types::{
    Callback, CallbackReader, Channel, ChannelStdinMode, LuaRef, Proc, SocketWatcher, dict_T,
    uint16_t, uint64_t, varnumber_T,
};
use crate::ui_client::ui_client_attach_to_restarted_server;
use ::libc::{dup2, fcntl};

use super::known::*;
use super::reader::{
    callback_reader_set, callback_reader_start, on_channel_data, on_job_stderr,
    schedule_channel_event,
};
use super::{
    channel_alloc, channel_create_event, channel_decref, channel_destroy_early, channel_internal,
    channel_proc, close_cb, main_loop_ptr,
};

/// Whether stdio has already been claimed. Only one channel may own it.
static did_stdio: GlobalCell<bool> = GlobalCell::new(false);

/// Spawns a child process and wires a channel to it.
///
/// Returns null and writes `status_out` on failure: 0 when the arguments were
/// rejected, otherwise the spawn's libuv error.
///
/// # Safety
/// The event loop exists; `argv`, `exepath`, `cwd`, `env` and `status_out` are
/// this call's to consume.
#[allow(clippy::too_many_arguments)]
pub unsafe fn channel_job_start(
    argv: *mut *mut c_char,
    exepath: *const c_char,
    on_stdout: CallbackReader,
    on_stderr: CallbackReader,
    on_exit: Callback,
    pty: bool,
    rpc: bool,
    overlapped: bool,
    detach: bool,
    stdin_mode: ChannelStdinMode,
    cwd: *const c_char,
    pty_width: uint16_t,
    pty_height: uint16_t,
    env: *mut dict_T,
    status_out: *mut varnumber_T,
) -> *mut Channel {
    /// A detached child has no controlling terminal to hand a pty to.
    const PTY_DETACHED: &CStr = c"terminal/pty job cannot be detached";

    // SAFETY: the caller's promise. The channel is this function's alone until
    // the spawn succeeds, so the unwind paths below may destroy it outright.
    let chan = unsafe { channel_alloc(kChannelStreamProc) };
    unsafe { (*chan).on_data = on_stdout };
    unsafe { (*chan).on_stderr = on_stderr };
    unsafe { (*chan).on_exit = on_exit };

    if pty && detach {
        let why = PTY_DETACHED.to_string_lossy();
        semsg!("E475: Invalid argument: {why}");
        unsafe { shell_free_argv(argv) };
        if !env.is_null() {
            unsafe { tv_dict_free(env) };
        }
        unsafe { channel_destroy_early(chan) };
        unsafe { *status_out = 0 };
        return ptr::null_mut();
    }

    if pty {
        unsafe { (*chan).stream.pty = pty_proc_init(main_loop_ptr(), chan.cast()) };
        if pty_width > 0 {
            unsafe { (*chan).stream.pty.width = pty_width };
        }
        if pty_height > 0 {
            unsafe { (*chan).stream.pty.height = pty_height };
        }
    } else {
        unsafe { (*chan).stream.uv = libuv_proc_init(main_loop_ptr(), chan.cast()) };
    }

    let proc = unsafe { channel_proc(chan) };
    unsafe { (*proc).argv = argv };
    unsafe { (*proc).exepath = exepath };
    unsafe { (*proc).cb = Some(channel_proc_exit_cb) };
    unsafe { (*proc).state_cb = Some(channel_proc_state_cb) };
    unsafe { (*proc).events = (*chan).events };
    unsafe { (*proc).detach = detach };
    unsafe { (*proc).cwd = cwd };
    unsafe { (*proc).env = env };
    unsafe { (*proc).overlapped = overlapped };

    // A pty multiplexes both directions onto the master, so it always
    // reads and never has a separate stderr.
    let (has_out, has_err) = if unsafe { (*proc).type_0 } as c_int == kProcTypePty {
        (true, false)
    } else {
        unsafe { (*proc).fwd_err = (*chan).on_stderr.fwd_err };
        (
            rpc || callback_reader_set(unsafe { &(*chan).on_data }),
            callback_reader_set(unsafe { &(*chan).on_stderr }),
        )
    };
    let has_in = stdin_mode == kChannelStdinPipe;

    // The name is copied because the failure message is formatted after
    // the spawn has taken ownership of `argv`.
    let cmd = unsafe { xstrdup(proc_get_exepath(proc)) };
    let status = unsafe { proc_spawn(proc, has_in, has_out, has_err) };
    if status != 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
        let (arg0, cmd) = unsafe { (c_str(uv_strerror(status)), c_str(cmd)) };
        semsg!("E903: Process failed to start: {arg0}: \"{cmd}\"");
    }
    unsafe { xfree(cmd.cast()) };
    if !unsafe { (*proc).env }.is_null() {
        unsafe { tv_dict_free((*proc).env) };
    }
    if status != 0 {
        unsafe { channel_destroy_early(chan) };
        unsafe { *status_out = (*proc).status as varnumber_T };
        return ptr::null_mut();
    }

    unsafe { start_job_streams(chan, rpc, has_in, has_out, has_err) };
    unsafe { *status_out = (*chan).id as varnumber_T };
    chan
}

/// Attaches readers and writers to a job whose spawn succeeded.
///
/// # Safety
/// `chan` is a live job channel whose process has just spawned.
unsafe fn start_job_streams(
    chan: *mut Channel,
    rpc: bool,
    has_in: bool,
    has_out: bool,
    has_err: bool,
) {
    // SAFETY: the caller's freshly spawned job.
    let proc = unsafe { channel_proc(chan) };
    if has_in {
        unsafe { wstream_init(&raw mut (*proc).in_0, 0) };
    }
    if has_out {
        unsafe { rstream_init(&raw mut (*proc).out) };
    }
    if rpc {
        unsafe { rpc_start(chan) };
    } else if has_out {
        unsafe { callback_reader_start(&raw mut (*chan).on_data, c"stdout".as_ptr()) };
        unsafe { rstream_start(&raw mut (*proc).out, Some(on_channel_data), chan.cast()) };
    }
    if has_err {
        unsafe { callback_reader_start(&raw mut (*chan).on_stderr, c"stderr".as_ptr()) };
        unsafe { rstream_init(&raw mut (*proc).err) };
        unsafe { rstream_start(&raw mut (*proc).err, Some(on_job_stderr), chan.cast()) };
    }
}

/// Connects to a socket (or, for an address this process is itself listening
/// on, to itself) and returns the channel id, or 0 on failure.
///
/// # Safety
/// The event loop exists; `address` is a C string and `error` is writable.
pub unsafe fn channel_connect(
    tcp: bool,
    address: *const c_char,
    rpc: bool,
    on_output: CallbackReader,
    timeout: c_int,
    error: *mut *const c_char,
) -> uint64_t {
    // SAFETY: the caller's promise.
    // Talking to our own listening socket would deadlock: the reply cannot
    // be read while this process is blocked writing the request. An
    // internal channel short-circuits the transport instead.
    if !tcp && rpc && unsafe { server_owns_pipe_address(address) } {
        let channel = unsafe { channel_alloc(kChannelStreamInternal) };
        unsafe { (*channel_internal(channel)).cb = LUA_NOREF as LuaRef };
        unsafe { rpc_start(channel) };
        unsafe { channel_create_event(channel, address) };
        return unsafe { (*channel).id };
    }

    let channel = unsafe { channel_alloc(kChannelStreamSocket) };
    let socket = unsafe { &raw mut (*channel).stream.socket };
    if !unsafe { socket_connect(main_loop_ptr(), socket, tcp, address, timeout, error) } {
        unsafe { channel_decref(channel) };
        return 0;
    }
    unsafe { attach_socket(channel) };
    if rpc {
        unsafe { rpc_start(channel) };
    } else {
        unsafe { (*channel).on_data = on_output };
        unsafe { callback_reader_start(&raw mut (*channel).on_data, c"data".as_ptr()) };
        unsafe { rstream_start(socket, Some(on_channel_data), channel.cast()) };
    }
    unsafe { channel_create_event(channel, address) };
    unsafe { (*channel).id }
}

/// Takes over an accepted connection from a listening socket. Always RPC.
///
/// # Safety
/// `watcher` is a live listening socket with a pending connection.
pub unsafe fn channel_from_connection(watcher: *mut SocketWatcher) {
    // SAFETY: the caller's live watcher.
    let channel = unsafe { channel_alloc(kChannelStreamSocket) };
    unsafe { socket_watcher_accept(watcher, &raw mut (*channel).stream.socket) };
    unsafe { attach_socket(channel) };
    unsafe { rpc_start(channel) };
    unsafe { channel_create_event(channel, (&raw mut (*watcher).addr).cast()) };
}

/// Gives a connected socket its owner and its buffers.
///
/// # Safety
/// `channel` is a live socket channel whose handle is connected.
unsafe fn attach_socket(channel: *mut Channel) {
    // SAFETY: the caller's live socket channel.
    let socket = unsafe { &raw mut (*channel).stream.socket };
    unsafe { (*socket).s.internal_close_cb = Some(close_cb) };
    unsafe { (*socket).s.internal_data = channel.cast() };
    unsafe { wstream_init(&raw mut (*socket).s, 0) };
    unsafe { rstream_init(socket) };
}

/// Wires a channel to this process's own stdin and stdout.
///
/// Only one may exist, and only when nothing else is using the terminal.
///
/// # Safety
/// The event loop exists and `error` is writable.
pub unsafe fn channel_from_stdio(
    rpc: bool,
    on_output: CallbackReader,
    error: *mut *const c_char,
) -> uint64_t {
    let refusal = if !headless_mode.get() && !embedded_mode.get() {
        Some(c"can only be opened in headless mode")
    } else if did_stdio.get() {
        Some(c"channel was already open")
    } else {
        None
    };
    if let Some(msg) = refusal {
        // SAFETY: the caller's writable out-parameter.
        unsafe { *error = gettext(msg).as_ptr() };
        return 0;
    }
    did_stdio.set(true);

    // SAFETY: the caller's promise; the descriptors below are this process's.
    let channel = unsafe { channel_alloc(kChannelStreamStdio) };
    let (stdin_fd, stdout_fd) = if embedded_mode.get() {
        // The embedder owns fds 0 and 1; move them aside and point what is
        // left at stderr, so a stray `print` cannot corrupt the protocol.
        let stdin_fd = unsafe { fcntl(STDIN_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1) };
        let stdout_fd = unsafe { fcntl(STDOUT_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1) };
        unsafe { dup2(STDERR_FILENO, STDOUT_FILENO) };
        unsafe { dup2(STDERR_FILENO, STDIN_FILENO) };
        (stdin_fd, stdout_fd)
    } else {
        (STDIN_FILENO, STDOUT_FILENO)
    };
    let in_0 = unsafe { &raw mut (*channel).stream.stdio.in_0 };
    unsafe { rstream_init_fd(main_loop_ptr(), in_0, stdin_fd) };
    unsafe {
        wstream_init_fd(
            main_loop_ptr(),
            &raw mut (*channel).stream.stdio.out,
            stdout_fd,
            0,
        )
    };
    if rpc {
        unsafe { rpc_start(channel) };
    } else {
        unsafe { (*channel).on_data = on_output };
        unsafe { callback_reader_start(&raw mut (*channel).on_data, c"stdin".as_ptr()) };
        unsafe { rstream_start(in_0, Some(on_channel_data), channel.cast()) };
    }
    unsafe { (*channel).id }
}

// -------------------------------------------------------------------------
// Process callbacks
// -------------------------------------------------------------------------

/// The child exited, or its handles closed. `status` is negative for the
/// latter, which is what "closed without an exit status" means here.
unsafe fn channel_proc_exit_cb(_proc: *mut Proc, status: c_int, data: *mut c_void) {
    // SAFETY: `data` is the channel the process was set up with, and the
    // process held one reference to it.
    let chan = data.cast::<Channel>();
    if !unsafe { (*chan).term }.is_null() {
        unsafe { terminal_close(&raw mut (*chan).term, status) };
    }
    // A UI client whose server died: try to reconnect before following it.
    if !exiting.get() && ui_client_channel_id.get() == unsafe { (*chan).id } {
        unsafe { ui_client_attach_to_restarted_server() };
        if ui_client_channel_id.get() == unsafe { (*chan).id } {
            exit_on_closed_chan(status);
        }
    }
    let exited = status >= 0;
    if exited && unsafe { &(*chan).on_exit }.is_set() {
        unsafe { schedule_channel_event(chan) };
    }
    if exited {
        unsafe { (*chan).exit_status = status };
    }
    unsafe { channel_decref(chan) };
}

/// The child was stopped or continued; only a terminal cares.
unsafe fn channel_proc_state_cb(_proc: *mut Proc, suspended: bool, data: *mut c_void) {
    // SAFETY: `data` is the channel the process was set up with.
    let chan = data.cast::<Channel>();
    if !unsafe { (*chan).term }.is_null() {
        unsafe { terminal_set_state((*chan).term, suspended) };
    }
}
