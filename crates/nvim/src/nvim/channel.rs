//! Channels: the editor's connections to child processes, sockets, its own
//! standard streams and its own terminal emulator.
//!
//! A channel is a numbered handle over one of five transports, plus optional
//! Vimscript callbacks and an optional msgpack-rpc layer
//! ([`msgpack_rpc::channel`]). The transports live in a union chosen by
//! `Channel::streamtype`; see [`channel_proc`] for why.
//!
//! Lifetime is by reference count, and every reference is dropped through an
//! event rather than inline: a channel is reachable from libuv handles, from
//! queued events and from Vimscript callbacks that may be running when it
//! closes.
//!
//! [`msgpack_rpc::channel`]: crate::src::nvim::msgpack_rpc::channel

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};

use crate::src::nvim::eval::typval::{callback_free, tv_dict_free};
use crate::src::nvim::event::libuv_proc::libuv_proc_init;
use crate::src::nvim::event::r#loop::one_arg_event;
use crate::src::nvim::event::multiqueue::{
    multiqueue_free, multiqueue_new_child, multiqueue_put_event,
};
use crate::src::nvim::event::proc::{
    exit_on_closed_chan, proc_free, proc_get_exepath, proc_spawn, proc_stop,
};
use crate::src::nvim::event::rstream::{
    rstream_init, rstream_init_fd, rstream_may_close, rstream_start, rstream_start_inner,
    rstream_stop_inner,
};
use crate::src::nvim::event::socket::{socket_connect, socket_watcher_accept};
use crate::src::nvim::event::stream::stream_may_close;
use crate::src::nvim::event::wstream::{
    wstream_init, wstream_init_fd, wstream_new_buffer, wstream_write,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::api_free_luaref;
use crate::src::nvim::main::{
    channels, e_invarg2, e_invchan, e_invstream, e_invstreamrpc, e_jobspawn, embedded_mode,
    exiting, headless_mode, main_loop, ui_client_channel_id,
};
use crate::src::nvim::map::{map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::src::nvim::memory::{xfree, xmemdup, xstrdup};
use crate::src::nvim::message::semsg;
use crate::src::nvim::msgpack_rpc::channel::call_stack::CallStack;
use crate::src::nvim::msgpack_rpc::channel::{rpc_close, rpc_free, rpc_init, rpc_start};
use crate::src::nvim::msgpack_rpc::server::server_owns_pipe_address;
use crate::src::nvim::os::fs::os_write;
use crate::src::nvim::os::libc::{dup2, fcntl, freopen, gettext, stderr};
use crate::src::nvim::os::pty_proc_unix::{
    pty_proc_close_master, pty_proc_init, pty_proc_resize, pty_proc_resume,
};
use crate::src::nvim::os::shell::shell_free_argv;
use crate::src::nvim::terminal::{
    terminal_alloc, terminal_close, terminal_destroy, terminal_receive, terminal_set_state,
};
use crate::src::nvim::types::{
    Callback, CallbackReader, Channel, ChannelPart, ChannelStdinMode, ChannelStreamType, Dict,
    InternalState, LuaRef, OptInt, Proc, PtyProc, RStream, RpcState, SocketWatcher, Stream,
    TerminalOptions, auto_event, buf_T, dict_T, ptr_t, size_t, uint16_t, uint64_t, varnumber_T,
};

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {

    use super::{ChannelPart, ChannelStdinMode, ChannelStreamType, auto_event};
    use core::ffi::c_int;

    pub const kChannelStreamProc: ChannelStreamType = 0;
    pub const kChannelStreamSocket: ChannelStreamType = 1;
    pub const kChannelStreamStdio: ChannelStreamType = 2;
    pub const kChannelStreamStderr: ChannelStreamType = 3;
    pub const kChannelStreamInternal: ChannelStreamType = 4;

    pub const kChannelPartStdin: ChannelPart = 0;
    pub const kChannelPartStdout: ChannelPart = 1;
    pub const kChannelPartStderr: ChannelPart = 2;
    pub const kChannelPartRpc: ChannelPart = 3;
    pub const kChannelPartAll: ChannelPart = 4;

    pub const kChannelStdinPipe: ChannelStdinMode = 0;

    pub const kProcTypePty: c_int = 1;
    pub const kCallbackNone: c_int = 0;
    pub const kListLenMayKnow: c_int = -3;
    pub const LUA_NOREF: c_int = -2;
    pub const LOGLVL_INF: c_int = 2;

    pub const VAR_UNKNOWN: c_int = 0;
    pub const VAR_NUMBER: c_int = 1;
    pub const VAR_STRING: c_int = 2;
    pub const VAR_LIST: c_int = 4;
    pub const VAR_DICT: c_int = 5;
    pub const VAR_UNLOCKED: c_int = 0;

    pub const EVENT_CHANINFO: auto_event = 23;
    pub const EVENT_CHANOPEN: auto_event = 24;

    pub const STDIN_FILENO: c_int = 0;
    pub const STDOUT_FILENO: c_int = 1;
    pub const STDERR_FILENO: c_int = 2;
    /// `fcntl` command: dup to the lowest free descriptor at or above the
    /// third argument, with close-on-exec set.
    pub const F_DUPFD_CLOEXEC: c_int = 1030;

    /// The id of the channel over this process's own stdin/stdout.
    pub const CHAN_STDIO: c_int = 1;
    /// The id of the channel over this process's own stderr.
    pub const CHAN_STDERR: c_int = 2;
}

use known::*;

pub mod info;
pub mod reader;

pub use info::{
    channel_all_info, channel_create_event, channel_info, channel_info_changed, channel_job_running,
};
pub use reader::{
    callback_reader_free, callback_reader_start, channel_reader_callbacks, on_channel_data,
    on_job_stderr,
};
use reader::{callback_reader_set, schedule_channel_event};

/// Whether stdio has already been claimed. Only one channel may own it.
static did_stdio: GlobalCell<bool> = GlobalCell::new(false);
/// The next dynamically allocated channel id. The first two are reserved.
static next_chan_id: GlobalCell<uint64_t> = GlobalCell::new((CHAN_STDERR + 1) as uint64_t);

// ---------------------------------------------------------------------------
// Transports
// ---------------------------------------------------------------------------
//
// `Channel::stream` is a union discriminated by `Channel::streamtype`. It stays
// a union rather than becoming a Rust enum because libuv and the stream layer
// hold the *addresses* of its members for the channel's whole life — a
// `uv_process_t`'s `data`, every `Stream::internal_data` — and Rust has no way
// to project into an enum variant without going through a reference, which
// would invalidate every pointer previously derived from the same storage. All
// the accessors below are raw projections from the caller's `*mut Channel`, so
// there is one provenance chain per channel and no aliasing between them.

/// The child process behind a job channel.
///
/// Valid for both kinds of job: `LibuvProc` and `PtyProc` both begin with the
/// `Proc` this returns, and the two are told apart by `Proc::type_0`.
pub unsafe fn channel_proc(chan: *mut Channel) -> *mut Proc {
    debug_assert!((*chan).streamtype == kChannelStreamProc);
    &raw mut (*chan).stream.proc
}

/// The pty master behind a pty job.
pub unsafe fn channel_pty(chan: *mut Channel) -> *mut PtyProc {
    debug_assert!((*chan).streamtype == kChannelStreamProc);
    debug_assert!((*chan).stream.proc.type_0 as c_int == kProcTypePty);
    &raw mut (*chan).stream.pty
}

/// The in-process endpoint behind an internal channel.
pub unsafe fn channel_internal(chan: *mut Channel) -> *mut InternalState {
    debug_assert!((*chan).streamtype == kChannelStreamInternal);
    &raw mut (*chan).stream.internal
}

/// The stream this channel writes to.
///
/// Only the three transports with a real handle have one; the internal and
/// stderr channels are written to by other means and never reach here.
pub unsafe fn channel_instream(chan: *mut Channel) -> *mut Stream {
    match (*chan).streamtype {
        kChannelStreamProc => &raw mut (*chan).stream.proc.in_0,
        kChannelStreamSocket => &raw mut (*chan).stream.socket.s,
        kChannelStreamStdio => &raw mut (*chan).stream.stdio.out,
        other => unreachable!("channel stream type {other} has no write stream"),
    }
}

/// The stream this channel reads from. See [`channel_instream`].
pub unsafe fn channel_outstream(chan: *mut Channel) -> *mut RStream {
    match (*chan).streamtype {
        kChannelStreamProc => &raw mut (*chan).stream.proc.out,
        kChannelStreamSocket => &raw mut (*chan).stream.socket,
        kChannelStreamStdio => &raw mut (*chan).stream.stdio.in_0,
        other => unreachable!("channel stream type {other} has no read stream"),
    }
}

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

/// The channel registered under `id`, or null.
pub unsafe fn find_channel(id: uint64_t) -> *mut Channel {
    let map = channels.ptr();
    let slot = mh_get_uint64_t(&raw mut (*map).set, id);
    // The hash's "absent" slot index.
    if slot == u32::MAX {
        return ptr::null_mut();
    }
    *(*map).values.add(slot as usize) as *mut Channel
}

/// Opens the stderr channel and the RPC event queue.
pub unsafe fn channel_init() {
    channel_alloc(kChannelStreamStderr);
    rpc_init();
}

/// Registers a new channel of `type_0` and hands out the caller's reference.
///
/// The stdio and stderr channels have fixed ids; everything else takes the
/// next one. `Channel` owns heap state (the RPC call stack), so it is a real
/// allocation rather than a zeroed block — the transport union is still
/// zeroed, because each transport's setup writes it whole.
pub unsafe fn channel_alloc(type_0: ChannelStreamType) -> *mut Channel {
    let id = match type_0 {
        kChannelStreamStdio => CHAN_STDIO as uint64_t,
        kChannelStreamStderr => CHAN_STDERR as uint64_t,
        _ => {
            let id = next_chan_id.get();
            next_chan_id.set(id.wrapping_add(1));
            id
        }
    };
    // Channel ids are handed to Vimscript as numbers.
    assert!(id <= i64::MAX as uint64_t);
    let chan = Box::into_raw(Box::new(Channel {
        id,
        refcount: 1,
        events: multiqueue_new_child((*main_loop.ptr()).events),
        streamtype: type_0,
        stream: mem::zeroed(),
        is_rpc: false,
        detach: false,
        rpc: RpcState {
            closed: false,
            unpacker: ptr::null_mut(),
            ui: ptr::null_mut(),
            next_request_id: 0,
            call_stack: CallStack::new(),
            info: empty_dict(),
            client_type: 0,
        },
        term: ptr::null_mut(),
        on_data: mem::zeroed(),
        on_stderr: mem::zeroed(),
        on_exit: mem::zeroed(),
        exit_status: -1,
        callback_busy: false,
        callback_scheduled: false,
    }));
    let slot = map_put_ref_uint64_t_ptr_t(channels.ptr(), id, ptr::null_mut(), ptr::null_mut());
    *slot = chan as ptr_t;
    chan
}

/// Closes every channel. Called on exit.
pub unsafe fn channel_teardown() {
    let map = channels.ptr();
    for i in 0..(*map).set.h.n_keys {
        let chan = *(*map).values.add(i as usize) as *mut Channel;
        channel_close((*chan).id, kChannelPartAll, ptr::null_mut());
    }
}

pub unsafe fn channel_incref(chan: *mut Channel) {
    (*chan).refcount += 1;
}

/// Drops a reference, scheduling the free once the last one goes.
///
/// The free is deferred to an event because the caller is very often a libuv
/// callback still standing on the channel's own memory.
pub unsafe fn channel_decref(chan: *mut Channel) {
    (*chan).refcount -= 1;
    if (*chan).refcount == 0 {
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            one_arg_event(Some(free_channel_event), chan as *mut c_void),
        );
    }
}

unsafe extern "C" fn free_channel_event(argv: *mut *mut c_void) {
    let chan = *argv as *mut Channel;
    map_del_uint64_t_ptr_t(channels.ptr(), (*chan).id, ptr::null_mut());
    channel_destroy(chan);
}

unsafe fn channel_destroy(chan: *mut Channel) {
    if (*chan).is_rpc {
        rpc_free(chan);
    }
    if (*chan).streamtype == kChannelStreamProc {
        proc_free(channel_proc(chan));
    }
    callback_reader_free(&raw mut (*chan).on_data);
    callback_reader_free(&raw mut (*chan).on_stderr);
    callback_free(&raw mut (*chan).on_exit);
    multiqueue_free((*chan).events);
    drop(Box::from_raw(chan));
}

/// Unwinds a channel that never got off the ground.
///
/// Only valid for the channel that took the most recent id, and only before
/// anything else has taken a reference — which is what the two assertions say.
/// Giving the id back keeps `job_spec`'s expectations about consecutive ids.
unsafe fn channel_destroy_early(chan: *mut Channel) {
    next_chan_id.set(next_chan_id.get().wrapping_sub(1));
    assert!(
        (*chan).id == next_chan_id.get(),
        "channel id was not the last"
    );
    map_del_uint64_t_ptr_t(channels.ptr(), (*chan).id, ptr::null_mut());
    (*chan).id = 0;
    (*chan).refcount -= 1;
    assert!((*chan).refcount == 0, "channel was already referenced");
    multiqueue_put_event(
        (*main_loop.ptr()).events,
        one_arg_event(Some(free_channel_event), chan as *mut c_void),
    );
}

/// The empty `Dict`, which is what a channel starts with and what
/// [`channel_info`] answers for an id that is not registered.
fn empty_dict() -> Dict {
    Dict {
        size: 0,
        capacity: 0,
        items: ptr::null_mut(),
    }
}

/// The stream layer's close callback: the channel is what owns the stream.
unsafe extern "C" fn close_cb(_stream: *mut Stream, data: *mut c_void) {
    channel_decref(data as *mut Channel);
}

// ---------------------------------------------------------------------------
// Closing
// ---------------------------------------------------------------------------

/// Closes `part` of channel `id`, reporting why not through `error`.
///
/// Which parts exist depends on the transport, and an RPC channel only accepts
/// `kChannelPartRpc`/`kChannelPartAll` — its stdin and stdout carry the
/// protocol and cannot be closed separately.
pub unsafe fn channel_close(id: uint64_t, part: ChannelPart, error: *mut *const c_char) -> bool {
    let mut dummy: *const c_char = ptr::null();
    let error = if error.is_null() {
        &raw mut dummy
    } else {
        error
    };

    let chan = find_channel(id);
    if chan.is_null() {
        // An id below the watermark named a channel that has already gone,
        // which is not an error: closing twice is allowed.
        if id < next_chan_id.get() {
            return true;
        }
        *error = e_invchan.ptr() as *const c_char;
        return false;
    }

    let close_main = part == kChannelPartRpc || part == kChannelPartAll;
    if close_main {
        if (*chan).is_rpc {
            rpc_close(chan);
        } else if part == kChannelPartRpc {
            *error = e_invstream.ptr() as *const c_char;
            return false;
        }
    } else if (part == kChannelPartStdin || part == kChannelPartStdout) && (*chan).is_rpc {
        *error = e_invstreamrpc.ptr() as *const c_char;
        return false;
    }

    match (*chan).streamtype {
        kChannelStreamSocket => {
            if !close_main {
                *error = e_invstream.ptr() as *const c_char;
                return false;
            }
            rstream_may_close(&raw mut (*chan).stream.socket);
        }
        kChannelStreamProc => {
            let proc = channel_proc(chan);
            if part == kChannelPartStdin || close_main {
                stream_may_close(&raw mut (*proc).in_0);
            }
            if part == kChannelPartStdout || close_main {
                rstream_may_close(&raw mut (*proc).out);
            }
            if part == kChannelPartStderr || part == kChannelPartAll {
                rstream_may_close(&raw mut (*proc).err);
            }
            if (*proc).type_0 as c_int == kProcTypePty && part == kChannelPartAll {
                pty_proc_close_master(channel_pty(chan));
            }
        }
        kChannelStreamStdio => {
            if part == kChannelPartStdin || close_main {
                rstream_may_close(&raw mut (*chan).stream.stdio.in_0);
            }
            if part == kChannelPartStdout || close_main {
                stream_may_close(&raw mut (*chan).stream.stdio.out);
            }
            // This process's stderr belongs to the stderr channel, not here.
            if part == kChannelPartStderr {
                *error = e_invstream.ptr() as *const c_char;
                return false;
            }
        }
        kChannelStreamStderr => {
            if part != kChannelPartAll && part != kChannelPartStderr {
                *error = e_invstream.ptr() as *const c_char;
                return false;
            }
            if !(*chan).stream.err.closed {
                (*chan).stream.err.closed = true;
                // On the way out the descriptor is about to go anyway, and
                // reopening it would swallow anything still being written.
                if !exiting.get() {
                    freopen(c"/dev/null".as_ptr(), c"w".as_ptr(), stderr);
                }
                channel_decref(chan);
            }
        }
        kChannelStreamInternal => {
            if !close_main {
                *error = e_invstream.ptr() as *const c_char;
                return false;
            }
            // An internal channel with a terminal is closed by closing the
            // terminal; without one there is nothing but the reference.
            if (*chan).term.is_null() {
                channel_decref(chan);
            } else {
                let internal = channel_internal(chan);
                api_free_luaref((*internal).cb);
                (*internal).cb = LUA_NOREF as LuaRef;
                (*internal).closed = true;
                terminal_close(&raw mut (*chan).term, 0);
                (*chan).exit_status = 0;
            }
        }
        _ => {}
    }
    true
}

// ---------------------------------------------------------------------------
// Opening
// ---------------------------------------------------------------------------

/// Spawns a child process and wires a channel to it.
///
/// Returns null and writes `status_out` on failure: 0 when the arguments were
/// rejected, otherwise the spawn's libuv error.
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
    let chan = channel_alloc(kChannelStreamProc);
    (*chan).on_data = on_stdout;
    (*chan).on_stderr = on_stderr;
    (*chan).on_exit = on_exit;

    if pty {
        // A detached child has no controlling terminal to hand a pty to.
        if detach {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"terminal/pty job cannot be detached".as_ptr(),
            );
            shell_free_argv(argv);
            if !env.is_null() {
                tv_dict_free(env);
            }
            channel_destroy_early(chan);
            *status_out = 0;
            return ptr::null_mut();
        }
        (*chan).stream.pty = pty_proc_init(main_loop.ptr(), chan as *mut c_void);
        if pty_width > 0 {
            (*chan).stream.pty.width = pty_width;
        }
        if pty_height > 0 {
            (*chan).stream.pty.height = pty_height;
        }
    } else {
        (*chan).stream.uv = libuv_proc_init(main_loop.ptr(), chan as *mut c_void);
    }

    let proc = channel_proc(chan);
    (*proc).argv = argv;
    (*proc).exepath = exepath;
    (*proc).cb = Some(channel_proc_exit_cb);
    (*proc).state_cb = Some(channel_proc_state_cb);
    (*proc).events = (*chan).events;
    (*proc).detach = detach;
    (*proc).cwd = cwd;
    (*proc).env = env;
    (*proc).overlapped = overlapped;

    // A pty multiplexes both directions onto the master, so it always reads
    // and never has a separate stderr.
    let (has_out, has_err) = if (*proc).type_0 as c_int == kProcTypePty {
        (true, false)
    } else {
        (*proc).fwd_err = (*chan).on_stderr.fwd_err;
        (
            rpc || callback_reader_set((*chan).on_data),
            callback_reader_set((*chan).on_stderr),
        )
    };
    let has_in = stdin_mode == kChannelStdinPipe;

    // The name is copied because the failure message is formatted after the
    // spawn has taken ownership of `argv`.
    let cmd = xstrdup(proc_get_exepath(proc));
    let status = proc_spawn(proc, has_in, has_out, has_err);
    if status != 0 {
        semsg(
            gettext(e_jobspawn.ptr() as *const c_char),
            crate::src::nvim::event::libuv::uv_strerror(status),
            cmd,
        );
        xfree(cmd as *mut c_void);
        if !(*proc).env.is_null() {
            tv_dict_free((*proc).env);
        }
        channel_destroy_early(chan);
        *status_out = (*proc).status as varnumber_T;
        return ptr::null_mut();
    }
    xfree(cmd as *mut c_void);
    if !(*proc).env.is_null() {
        tv_dict_free((*proc).env);
    }

    if has_in {
        wstream_init(&raw mut (*proc).in_0, 0);
    }
    if has_out {
        rstream_init(&raw mut (*proc).out);
    }
    if rpc {
        rpc_start(chan);
    } else if has_out {
        callback_reader_start(&raw mut (*chan).on_data, c"stdout".as_ptr());
        rstream_start(
            &raw mut (*proc).out,
            Some(on_channel_data),
            chan as *mut c_void,
        );
    }
    if has_err {
        callback_reader_start(&raw mut (*chan).on_stderr, c"stderr".as_ptr());
        rstream_init(&raw mut (*proc).err);
        rstream_start(
            &raw mut (*proc).err,
            Some(on_job_stderr),
            chan as *mut c_void,
        );
    }
    *status_out = (*chan).id as varnumber_T;
    chan
}

/// Connects to a socket (or, for an address this process is itself listening
/// on, to itself) and returns the channel id, or 0 on failure.
pub unsafe fn channel_connect(
    tcp: bool,
    address: *const c_char,
    rpc: bool,
    on_output: CallbackReader,
    timeout: c_int,
    error: *mut *const c_char,
) -> uint64_t {
    // Talking to our own listening socket would deadlock: the reply cannot be
    // read while this process is blocked writing the request. An internal
    // channel short-circuits the transport instead.
    if !tcp && rpc && server_owns_pipe_address(address) {
        let channel = channel_alloc(kChannelStreamInternal);
        (*channel_internal(channel)).cb = LUA_NOREF as LuaRef;
        rpc_start(channel);
        channel_create_event(channel, address);
        return (*channel).id;
    }

    let channel = channel_alloc(kChannelStreamSocket);
    if !socket_connect(
        main_loop.ptr(),
        &raw mut (*channel).stream.socket,
        tcp,
        address,
        timeout,
        error,
    ) {
        channel_decref(channel);
        return 0;
    }
    attach_socket(channel);
    if rpc {
        rpc_start(channel);
    } else {
        (*channel).on_data = on_output;
        callback_reader_start(&raw mut (*channel).on_data, c"data".as_ptr());
        rstream_start(
            &raw mut (*channel).stream.socket,
            Some(on_channel_data),
            channel as *mut c_void,
        );
    }
    channel_create_event(channel, address);
    (*channel).id
}

/// Takes over an accepted connection from a listening socket. Always RPC.
pub unsafe fn channel_from_connection(watcher: *mut SocketWatcher) {
    let channel = channel_alloc(kChannelStreamSocket);
    socket_watcher_accept(watcher, &raw mut (*channel).stream.socket);
    attach_socket(channel);
    rpc_start(channel);
    channel_create_event(channel, (&raw mut (*watcher).addr) as *mut c_char);
}

/// Gives a connected socket its owner and its buffers.
unsafe fn attach_socket(channel: *mut Channel) {
    let socket = &raw mut (*channel).stream.socket;
    (*socket).s.internal_close_cb = Some(close_cb);
    (*socket).s.internal_data = channel as *mut c_void;
    wstream_init(&raw mut (*socket).s, 0);
    rstream_init(socket);
}

/// Wires a channel to this process's own stdin and stdout.
///
/// Only one may exist, and only when nothing else is using the terminal.
pub unsafe fn channel_from_stdio(
    rpc: bool,
    on_output: CallbackReader,
    error: *mut *const c_char,
) -> uint64_t {
    if !headless_mode.get() && !embedded_mode.get() {
        *error = gettext(c"can only be opened in headless mode".as_ptr());
        return 0;
    }
    if did_stdio.get() {
        *error = gettext(c"channel was already open".as_ptr());
        return 0;
    }
    did_stdio.set(true);

    let channel = channel_alloc(kChannelStreamStdio);
    let mut stdin_dup_fd = STDIN_FILENO;
    let mut stdout_dup_fd = STDOUT_FILENO;
    if embedded_mode.get() {
        // The embedder owns fds 0 and 1; move them aside and point what is
        // left at stderr, so a stray `print` cannot corrupt the protocol.
        stdin_dup_fd = fcntl(STDIN_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1);
        stdout_dup_fd = fcntl(STDOUT_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1);
        dup2(STDERR_FILENO, STDOUT_FILENO);
        dup2(STDERR_FILENO, STDIN_FILENO);
    }
    rstream_init_fd(
        main_loop.ptr(),
        &raw mut (*channel).stream.stdio.in_0,
        stdin_dup_fd,
    );
    wstream_init_fd(
        main_loop.ptr(),
        &raw mut (*channel).stream.stdio.out,
        stdout_dup_fd,
        0,
    );
    if rpc {
        rpc_start(channel);
    } else {
        (*channel).on_data = on_output;
        callback_reader_start(&raw mut (*channel).on_data, c"stdin".as_ptr());
        rstream_start(
            &raw mut (*channel).stream.stdio.in_0,
            Some(on_channel_data),
            channel as *mut c_void,
        );
    }
    (*channel).id
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// Writes raw bytes to a channel, returning how many were accepted.
///
/// `data_owned` hands this function the buffer: it is either passed straight
/// to the write queue or freed here.
pub unsafe fn channel_send(
    id: uint64_t,
    data: *mut c_char,
    len: size_t,
    data_owned: bool,
    error: *mut *const c_char,
) -> size_t {
    const CLOSED: &CStr = c"Can't send data to closed stream";
    const RAW_TO_RPC: &CStr = c"Can't send raw data to rpc channel";

    let mut written = 0;
    let chan = find_channel(id);
    if chan.is_null() {
        *error = gettext(e_invchan.ptr() as *const c_char);
    } else if (*chan).streamtype == kChannelStreamStderr {
        if (*chan).stream.err.closed {
            *error = gettext(CLOSED.as_ptr());
        } else {
            // stderr is not on the event loop; it is written synchronously and
            // a short write is reported as such.
            let wres = os_write(STDERR_FILENO, data, len, false);
            if wres >= 0 {
                written = wres as size_t;
            }
        }
    } else if (*chan).streamtype == kChannelStreamInternal {
        if (*chan).is_rpc {
            *error = gettext(RAW_TO_RPC.as_ptr());
        } else if (*chan).term.is_null() || (*channel_internal(chan)).closed {
            *error = gettext(CLOSED.as_ptr());
        } else {
            terminal_receive((*chan).term, data, len);
            written = len;
        }
    } else {
        let in_0 = channel_instream(chan);
        if (*in_0).closed {
            *error = gettext(CLOSED.as_ptr());
        } else if (*chan).is_rpc {
            *error = gettext(RAW_TO_RPC.as_ptr());
        } else {
            let owned = if data_owned {
                data as *mut c_void
            } else {
                xmemdup(data as *const c_void, len)
            };
            let buf = wstream_new_buffer(owned as *mut c_char, len, 1, Some(xfree));
            // The write queue owns the buffer either way, so this returns
            // without the free below.
            return if wstream_write(in_0, buf) == 0 {
                len
            } else {
                0
            };
        }
    }
    if data_owned {
        xfree(data as *mut c_void);
    }
    written
}

// ---------------------------------------------------------------------------
// Process callbacks
// ---------------------------------------------------------------------------

/// The child exited, or its handles closed. `status` is negative for the
/// latter, which is what "closed without an exit status" means here.
unsafe extern "C" fn channel_proc_exit_cb(_proc: *mut Proc, status: c_int, data: *mut c_void) {
    let chan = data as *mut Channel;
    if !(*chan).term.is_null() {
        terminal_close(&raw mut (*chan).term, status);
    }
    // A UI client whose server died: try to reconnect before following it.
    if !exiting.get() && ui_client_channel_id.get() == (*chan).id {
        ui_client_attach_to_restarted_server();
        if ui_client_channel_id.get() == (*chan).id {
            exit_on_closed_chan(status);
        }
    }
    let exited = status >= 0;
    if exited && (*chan).on_exit.type_0 as c_int != kCallbackNone {
        schedule_channel_event(chan);
    }
    if exited {
        (*chan).exit_status = status;
    }
    channel_decref(chan);
}

/// The child was stopped or continued; only a terminal cares.
unsafe extern "C" fn channel_proc_state_cb(_proc: *mut Proc, suspended: bool, data: *mut c_void) {
    let chan = data as *mut Channel;
    if !(*chan).term.is_null() {
        terminal_set_state((*chan).term, suspended);
    }
}

use crate::src::nvim::ui_client::ui_client_attach_to_restarted_server;

// ---------------------------------------------------------------------------
// The terminal bridge
// ---------------------------------------------------------------------------

/// Gives `buf` a terminal driven by this channel's pty.
pub unsafe fn channel_terminal_alloc(buf: *mut buf_T, chan: *mut Channel) {
    let pty = channel_pty(chan);
    let topts = TerminalOptions {
        data: chan as *mut c_void,
        width: (*pty).width,
        height: (*pty).height,
        read_pause_cb: Some(term_read_pause),
        write_cb: Some(term_write),
        resize_cb: Some(term_resize),
        resume_cb: Some(term_resume),
        close_cb: Some(term_close),
        force_crlf: false,
    };
    (*buf).b_p_channel = (*chan).id as OptInt;
    channel_incref(chan);
    (*chan).term = terminal_alloc(buf, topts);
}

/// Back-pressure from the terminal: stop reading while it catches up.
unsafe extern "C" fn term_read_pause(pause: bool, data: *mut c_void) {
    let chan = data as *mut Channel;
    let out = &raw mut (*chan).stream.proc.out;
    if (*out).s.closed {
        return;
    }
    if pause {
        rstream_stop_inner(out);
    } else {
        rstream_start_inner(out);
    }
}

/// The user typed into the terminal; forward it to the child.
unsafe extern "C" fn term_write(buf: *const c_char, size: size_t, data: *mut c_void) {
    let chan = data as *mut Channel;
    let in_0 = &raw mut (*chan).stream.proc.in_0;
    if (*in_0).closed {
        logmsg(
            LOGLVL_INF,
            ptr::null(),
            c"term_write".as_ptr(),
            918,
            true,
            c"write failed: stream is closed".as_ptr(),
        );
        return;
    }
    let wbuf = wstream_new_buffer(
        xmemdup(buf as *const c_void, size) as *mut c_char,
        size,
        1,
        Some(xfree),
    );
    wstream_write(in_0, wbuf);
}

unsafe extern "C" fn term_resize(width: uint16_t, height: uint16_t, data: *mut c_void) {
    let chan = data as *mut Channel;
    pty_proc_resize(channel_pty(chan), width, height);
}

unsafe extern "C" fn term_resume(data: *mut c_void) {
    let chan = data as *mut Channel;
    pty_proc_resume(channel_pty(chan));
}

/// The terminal window went away: stop the child and wait for its streams.
unsafe extern "C" fn term_close(data: *mut c_void) {
    let chan = data as *mut Channel;
    proc_stop(channel_proc(chan));
    multiqueue_put_event((*chan).events, one_arg_event(Some(term_delayed_free), data));
}

/// Frees the terminal once nothing is still writing through it.
///
/// Re-queues itself while either stream has a request outstanding, because
/// those requests hold buffers the terminal owns.
unsafe extern "C" fn term_delayed_free(argv: *mut *mut c_void) {
    let chan = *argv as *mut Channel;
    let proc = &raw mut (*chan).stream.proc;
    if (*proc).in_0.pending_reqs != 0 || (*proc).out.s.pending_reqs != 0 {
        multiqueue_put_event(
            (*chan).events,
            one_arg_event(Some(term_delayed_free), chan as *mut c_void),
        );
        return;
    }
    if !(*chan).term.is_null() {
        terminal_destroy(&raw mut (*chan).term);
    }
    channel_decref(chan);
}
