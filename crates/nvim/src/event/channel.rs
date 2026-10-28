//! A channel's transport on the event loop.
//!
//! A channel ([`Channel`]) embeds its transport -- a child process, a
//! socket, this process's own stdio -- and libuv keeps the addresses of the
//! handles inside it. The channel table boxes every channel and frees one
//! only once each of those handles has closed, so the transport's address
//! is fixed for exactly as long as libuv may use it. This module is the one
//! place that hands those addresses out: every function that *registers* a
//! handle (spawning, connecting, accepting, starting reads) takes the
//! channel's **id**, finds the channel in the table, and does the
//! registration here.
//!
//! The other direction is the same: a handle's callbacks carry the
//! channel's id in their `data` word, never its address, and the shims
//! below decode it and call the channel layer's safe functions with it. The
//! channel layer looks the channel up again itself, and stops cleanly if it
//! has gone.
//!
//! What only *uses* a handle already registered -- closing it, writing to
//! it, stopping or resizing the child -- is a method on the transport,
//! called with the channel borrowed from the table for that one step.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `event/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::channel::with_channel;
use crate::eval::typval::DictRef;
use crate::event::r#loop::{Drain, main_loop_raw};
use crate::event::multiqueue::{
    ParkingQueue, multiqueue_process_events, multiqueue_replace_parent,
};
use crate::event::proc::{proc_free, proc_spawn, proc_stop, proc_wait};
use crate::event::rstream::{
    rstream_init, rstream_init_fd, rstream_may_close, rstream_start, rstream_start_inner,
    rstream_stop_inner,
};
use crate::event::socket::{socket_connect, socket_watcher_accept};
use crate::event::stream::stream_may_close;
use crate::event::wstream::{wstream_init, wstream_init_fd, wstream_new_buffer, wstream_write};
use crate::memory::{xfree, xmemdup};
use crate::os::pty_proc_unix::{
    pty_proc_close_master, pty_proc_resize, pty_proc_resume, pty_proc_tty_name,
};
use crate::os::shell::Argv;
use crate::types::{Channel, ChannelStream, JobProc, Proc, RStream, StdioPair, Stream, size_t};

/// The `data` word a channel's handles carry: its id, not its address.
pub(crate) fn owner_data(id: u64) -> *mut c_void {
    ptr::without_provenance_mut(usize::try_from(id).expect("a channel id fits a pointer"))
}

/// The channel id a `data` word carries.
fn owner_id(data: *mut c_void) -> u64 {
    data.addr() as u64
}

/// Run `f` on channel `id`, borrowed from the table for a step that runs no
/// user code -- registering a handle, closing one.
///
/// # Panics
/// When there is no such channel: every caller is reached for a channel
/// that holds a reference to itself (a handle, an event, a waiting frame).
fn with<R>(id: u64, f: impl FnOnce(&mut Channel) -> R) -> R {
    with_channel(id, f).unwrap_or_else(|| panic!("channel {id} is gone"))
}

/// The child behind job channel `id`, at its fixed address, with the
/// provenance of the whole `LibuvProc`/`PtyProc` the process layer casts it
/// back to.
fn job_proc(id: u64) -> *mut Proc {
    with(id, |channel| match &mut channel.stream {
        ChannelStream::Job(job) => job.proc_ptr(),
        _ => unreachable!("channel {id} is not a job"),
    })
}

// ---------------------------------------------------------------------------
// The transport, borrowed
// ---------------------------------------------------------------------------

impl JobProc {
    /// The child as the process layer takes it, derived from the whole
    /// variant so that a cast back to `LibuvProc`/`PtyProc` stays in bounds.
    fn proc_ptr(&mut self) -> *mut Proc {
        match self {
            JobProc::Uv(uv) => (&raw mut *uv).cast(),
            JobProc::Pty(pty) => (&raw mut *pty).cast(),
        }
    }

    /// The child's `Proc`, mutably.
    pub(crate) fn proc_mut(&mut self) -> &mut Proc {
        match self {
            JobProc::Uv(uv) => &mut uv.proc,
            JobProc::Pty(pty) => &mut pty.proc,
        }
    }

    /// Ask the child to terminate, and arm the kill timer.
    pub(crate) fn stop(&mut self) {
        // SAFETY: a spawned child at its fixed address; stopping registers
        // nothing new.
        unsafe { proc_stop(self.proc_ptr()) };
    }

    /// Release the argument vector the child was started with.
    pub(crate) fn free_argv(&mut self) {
        // SAFETY: as above; the vector is the child's own.
        unsafe { proc_free(self.proc_ptr()) };
    }

    /// The child's command line, as it was started.
    pub(crate) fn argv(&self) -> Vec<&CStr> {
        let mut words = Vec::new();
        let mut at = self.proc().argv;
        if at.is_null() {
            return words;
        }
        // SAFETY: a spawned child owns a NULL-terminated vector of
        // NUL-terminated words until `free_argv`; the walk stops at the
        // null, and each word is borrowed for as long as `self` is.
        unsafe {
            while !(*at).is_null() {
                words.push(CStr::from_ptr(*at));
                at = at.add(1);
            }
        }
        words
    }

    /// Close the requested halves of the child's pipes.
    pub(crate) fn close_streams(&mut self, stdin: bool, stdout: bool, stderr: bool) {
        let proc = self.proc_mut();
        if stdin {
            proc.in_0.close();
        }
        if stdout {
            proc.out.close();
        }
        if stderr {
            proc.err.close();
        }
    }

    /// Close a pty child's master, which sends it SIGHUP.
    pub(crate) fn close_master(&mut self) {
        if let JobProc::Pty(pty) = self {
            // SAFETY: the child's own pty.
            unsafe { pty_proc_close_master(pty) };
        }
    }

    /// Tell a pty child its terminal changed size.
    pub(crate) fn resize(&mut self, width: u16, height: u16) {
        if let JobProc::Pty(pty) = self {
            // SAFETY: the child's own pty.
            unsafe { pty_proc_resize(pty, width, height) };
        }
    }

    /// Resume a suspended pty child.
    pub(crate) fn resume(&mut self) {
        if let JobProc::Pty(pty) = self {
            // SAFETY: the child's own pty.
            unsafe { pty_proc_resume(pty) };
        }
    }

    /// The path of a pty child's side of the pty, copied out of libc's
    /// buffer.
    pub(crate) fn tty_name(&mut self) -> Option<std::ffi::CString> {
        let JobProc::Pty(pty) = self else {
            return None;
        };
        // SAFETY: the child's own pty; `ptsname` answers null or libc's
        // NUL-terminated buffer, copied before anything else can call it.
        let name = unsafe { pty_proc_tty_name(pty) };
        (!name.is_null()).then(|| unsafe { CStr::from_ptr(name) }.to_owned())
    }

    /// Stop or restart reading the child's output, without changing whether
    /// it wants reads: the terminal's back-pressure.
    pub(crate) fn pause_output(&mut self, pause: bool) {
        let out = &mut self.proc_mut().out;
        if out.s.closed {
            return;
        }
        let action = if pause {
            rstream_stop_inner
        } else {
            rstream_start_inner
        };
        // SAFETY: the child's own output stream, started by the spawn.
        unsafe { action(out) };
    }

    /// Whether a write or a read is still in flight on stdin or stdout.
    pub(crate) fn has_pending_requests(&self) -> bool {
        let proc = self.proc();
        proc.in_0.pending_reqs != 0 || proc.out.s.pending_reqs != 0
    }
}

impl RStream {
    /// Mark the stream closed; its handle closes once nothing is in flight.
    pub(crate) fn close(&mut self) {
        // SAFETY: a stream this module initialised, at its fixed address.
        unsafe { rstream_may_close(self) };
    }
}

impl Stream {
    /// Mark the stream closed; its handle closes once nothing is in flight.
    pub(crate) fn close(&mut self) {
        // SAFETY: a stream this module initialised, at its fixed address.
        unsafe { stream_may_close(self) };
    }

    /// Queue `bytes` for writing, handing their block to the write queue;
    /// whether libuv took it.
    pub(crate) fn write_owned(&mut self, bytes: Vec<u8>) -> bool {
        if bytes.capacity() == 0 {
            // No block to hand over (a dangling `Vec` pointer is not one).
            return self.write_bytes(&bytes);
        }
        let len = bytes.len();
        let block = core::mem::ManuallyDrop::new(bytes).as_mut_ptr();
        // The global allocator is `malloc`, so the vector's block is one the
        // buffer's `xfree` finalizer releases.
        let buffer = wstream_new_buffer(block.cast::<c_char>(), len, 1, Some(xfree));
        // SAFETY: a write stream this module initialised, and a buffer whose
        // one reference the write takes over.
        unsafe { wstream_write(self, buffer) == 0 }
    }

    /// Queue a copy of `bytes` for writing; whether libuv took it.
    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) -> bool {
        // SAFETY: the caller's readable bytes, copied into a block the
        // buffer's finalizer frees.
        let copy = unsafe { xmemdup(bytes.as_ptr().cast(), bytes.len()) };
        let buffer = wstream_new_buffer(copy.cast::<c_char>(), bytes.len(), 1, Some(xfree));
        // SAFETY: a write stream this module initialised, and a buffer whose
        // one reference the write takes over.
        unsafe { wstream_write(self, buffer) == 0 }
    }
}

// ---------------------------------------------------------------------------
// Registering, by id
// ---------------------------------------------------------------------------

/// What a job is started with.
pub(crate) struct JobSpawn<'a> {
    pub argv: Argv,
    pub exepath: Option<&'a CStr>,
    pub cwd: Option<&'a CStr>,
    pub env: Option<DictRef>,
    pub detach: bool,
    pub overlapped: bool,
    pub fwd_err: bool,
    pub has_in: bool,
    pub has_out: bool,
    pub has_err: bool,
}

/// Spawn job channel `id`'s child, or answer the negative libuv error, in
/// which case the child's handles are already closed and its argv released.
pub(crate) fn spawn_job(id: u64, spec: JobSpawn<'_>) -> Result<(), c_int> {
    let proc = with(id, |channel| {
        let events = channel.events.as_ptr();
        let ChannelStream::Job(job) = &mut channel.stream else {
            unreachable!("channel {id} is not a job")
        };
        let proc = job.proc_mut();
        proc.argv = spec.argv.into_raw();
        proc.exepath = spec.exepath.map_or(ptr::null(), CStr::as_ptr);
        proc.cwd = spec.cwd.map_or(ptr::null(), CStr::as_ptr);
        proc.env = spec.env.map_or(ptr::null_mut(), DictRef::into_raw);
        proc.cb = Some(job_exit_shim);
        proc.state_cb = Some(job_state_shim);
        proc.events = events;
        proc.detach = spec.detach;
        proc.overlapped = spec.overlapped;
        proc.fwd_err = spec.fwd_err;
        job.proc_ptr()
    });
    // SAFETY: the job's child, at its fixed address and unspawned; the
    // strings outlive the spawn, which is the only reader of `exepath` and
    // `cwd`.
    let status = unsafe { proc_spawn(proc, spec.has_in, spec.has_out, spec.has_err) };
    let env = with(id, |channel| {
        let ChannelStream::Job(job) = &mut channel.stream else {
            unreachable!("channel {id} is not a job")
        };
        let proc = job.proc_mut();
        proc.exepath = ptr::null();
        proc.cwd = ptr::null();
        core::mem::replace(&mut proc.env, ptr::null_mut())
    });
    // The environment was only for the spawn.
    // SAFETY: the reference `into_raw` gave the child, taken back once.
    drop(unsafe { DictRef::owning(env) });
    if status == 0 { Ok(()) } else { Err(status) }
}

/// Which consumer a channel's reads go to.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Reads {
    /// A job's stdout, a socket's or stdio's data: `on_stdout`/`on_data`.
    Output,
    /// A job's stderr: `on_stderr`.
    Stderr,
    /// The msgpack-rpc decoder.
    Rpc,
}

/// Give job channel `id`'s spawned pipes their buffers. Stderr is set up by
/// [`start_reads`] when it is read at all.
pub(crate) fn init_job_streams(id: u64, has_in: bool, has_out: bool) {
    let proc = job_proc(id);
    if has_in {
        // SAFETY: the job channel's stdin, at its fixed address, which the
        // spawn has just connected.
        unsafe { wstream_init(&raw mut (*proc).in_0, 0) };
    }
    if has_out {
        // SAFETY: as above, for stdout.
        unsafe { rstream_init(&raw mut (*proc).out) };
    }
}

/// Start delivering channel `id`'s reads to `reads`.
pub(crate) fn start_reads(id: u64, reads: Reads) {
    let (stream, init) = with(id, |channel| match (&mut channel.stream, reads) {
        (ChannelStream::Job(job), Reads::Stderr) => (&raw mut job.proc_mut().err, true),
        (ChannelStream::Job(job), _) => (&raw mut job.proc_mut().out, false),
        (ChannelStream::Socket(socket), _) => (&raw mut *socket, false),
        (ChannelStream::Stdio(stdio), _) => (&raw mut stdio.in_0, false),
        _ => unreachable!("channel {id} has nothing to read"),
    });
    let shim: unsafe fn(*mut RStream, *const c_char, size_t, *mut c_void, bool) -> size_t =
        match reads {
            Reads::Output => output_shim,
            Reads::Stderr => stderr_shim,
            Reads::Rpc => crate::msgpack_rpc::channel::receive::receive_msgpack,
        };
    if init {
        // SAFETY: the job's stderr, at its fixed address, connected by the
        // spawn and given its buffer here.
        unsafe { rstream_init(stream) };
    }
    // SAFETY: the channel's read stream, at its fixed address and
    // initialised; the callback is told the id.
    unsafe { rstream_start(stream, Some(shim), owner_data(id)) };
}

/// Connect socket channel `id` to `address`, driving the loop until it
/// settles; why not, as a message the caller may show.
pub(crate) fn connect_socket(
    id: u64,
    tcp: bool,
    address: &CStr,
    timeout: c_int,
) -> Result<(), &'static CStr> {
    let socket = socket_stream(id);
    let mut error: *const c_char = ptr::null();
    let (address, at_error) = (address.as_ptr(), &raw mut error);
    // SAFETY: the socket channel's stream, at its fixed address; the loop
    // runs only libuv's callbacks here, and the channel holds its own
    // reference until this returns.
    let connected =
        unsafe { socket_connect(main_loop_raw(), socket, tcp, address, timeout, at_error) };
    if connected {
        Ok(())
    } else {
        // SAFETY: `socket_connect` names every failure with a static string.
        Err(unsafe { CStr::from_ptr(error) })
    }
}

/// Take over a pending connection as socket channel `id`'s stream, or
/// answer libuv's error.
pub(crate) fn accept_socket(
    id: u64,
    watcher: &mut crate::types::SocketWatcher,
) -> Result<(), c_int> {
    let socket = socket_stream(id);
    // SAFETY: the socket channel's stream, at its fixed address, and the
    // caller's listening watcher.
    let result = unsafe { socket_watcher_accept(watcher, socket) };
    if result == 0 { Ok(()) } else { Err(result) }
}

/// Channel `id`'s write and read streams' addresses, for the log line
/// that names them; never dereferenced.
pub(crate) fn stream_addresses(id: u64) -> (*mut c_void, *mut c_void) {
    with(id, |channel| match &mut channel.stream {
        ChannelStream::Job(job) => {
            let proc = job.proc_mut();
            ((&raw mut proc.in_0).cast(), (&raw mut proc.out).cast())
        }
        ChannelStream::Socket(socket) => ((&raw mut socket.s).cast(), (&raw mut *socket).cast()),
        ChannelStream::Stdio(stdio) => ((&raw mut stdio.out).cast(), (&raw mut stdio.in_0).cast()),
        _ => (ptr::null_mut(), ptr::null_mut()),
    })
}

/// Socket channel `id`'s stream, at its fixed address.
fn socket_stream(id: u64) -> *mut RStream {
    with(id, |channel| match &mut channel.stream {
        ChannelStream::Socket(socket) => &raw mut *socket,
        _ => unreachable!("channel {id} is not a socket"),
    })
}

/// Make socket channel `id` the owner of its connected stream: closing the
/// stream drops the channel's reference, and both halves get their buffers.
pub(crate) fn attach_socket(id: u64) {
    with(id, |channel| {
        if let ChannelStream::Socket(socket) = &mut channel.stream {
            socket.s.internal_close_cb = Some(stream_close_shim);
            socket.s.internal_data = owner_data(id);
        }
    });
    let socket = socket_stream(id);
    // SAFETY: the socket channel's connected stream, at its fixed address.
    unsafe { wstream_init(&raw mut (*socket).s, 0) };
    // SAFETY: as above.
    unsafe { rstream_init(socket) };
}

/// Open stdio channel `id`'s two streams over `stdin_fd` and `stdout_fd`.
pub(crate) fn open_stdio(id: u64, stdin_fd: c_int, stdout_fd: c_int) {
    let (stdin, stdout) = with(id, |channel| match &mut channel.stream {
        ChannelStream::Stdio(stdio) => (&raw mut stdio.in_0, &raw mut stdio.out),
        _ => unreachable!("channel {id} is not stdio"),
    });
    // SAFETY: the stdio channel's streams, at their fixed address, over two
    // descriptors this process owns.
    unsafe { rstream_init_fd(main_loop_raw(), stdin, stdin_fd) };
    // SAFETY: as above.
    unsafe { wstream_init_fd(main_loop_raw(), stdout, stdout_fd, 0) };
}

/// A transport before it is opened: every handle unregistered, every
/// pointer null, which is what the setup functions above expect to fill.
pub(crate) fn unopened_rstream() -> RStream {
    // SAFETY: a stream is plain C data -- pointers, integers, flags and
    // libuv handles -- for all of which all-zero is the unset value.
    unsafe { core::mem::zeroed() }
}

/// [`unopened_rstream`], for a stdio pair.
pub(crate) fn unopened_stdio() -> StdioPair {
    // SAFETY: as above.
    unsafe { core::mem::zeroed() }
}

/// Queue `buffer` on `channel`'s write stream. 0, or the libuv error.
///
/// By address rather than by id: the RPC layer, its one caller, holds the
/// channel already, and a lookup per message is its hot path's cost.
///
/// # Safety
/// `channel` is a live channel with a write stream, and `buffer` is a
/// reference the caller owns and gives up here.
pub(crate) unsafe fn write_wbuffer(
    channel: *mut Channel,
    buffer: *mut crate::types::WBuffer,
) -> c_int {
    // SAFETY: the caller's live channel; the borrow ends with the match.
    let stream = match unsafe { &mut (*channel).stream } {
        ChannelStream::Job(job) => &raw mut job.proc_mut().in_0,
        ChannelStream::Socket(socket) => &raw mut socket.s,
        ChannelStream::Stdio(stdio) => &raw mut stdio.out,
        _ => unreachable!("the channel has no write stream"),
    };
    // SAFETY: the channel's write stream, at its fixed address and
    // initialised, and the caller's reference to the buffer.
    unsafe { wstream_write(stream, buffer) }
}

// ---------------------------------------------------------------------------
// Waiting, by id
// ---------------------------------------------------------------------------

/// Wait at most `ms` for job channel `id`'s child to exit, draining `drain`
/// (the child's own queue when `None`): its exit status, -1 on timeout, -2
/// when interrupted.
///
/// The caller holds a reference to the channel, which keeps the child at
/// its address through every event the wait lets run.
pub(crate) fn wait_job(id: u64, ms: c_int, drain: Option<&ParkingQueue>) -> c_int {
    let proc = job_proc(id);
    let events = drain.map_or(ptr::null_mut(), |queue| Drain::Parked(queue).queue());
    // SAFETY: the job channel's child, at its fixed address; the wait holds
    // no borrow of it across the events it runs.
    unsafe { proc_wait(proc, ms, events) }
}

/// Run whatever channel `id` has queued.
pub(crate) fn process_channel_events(id: u64) {
    let events = with(id, |channel| channel.events.as_ptr());
    // SAFETY: the channel's own queue, which lives as long as the channel;
    // nothing of the channel is borrowed while its events run.
    unsafe { multiqueue_process_events(events) };
}

/// Re-parent channel `id`'s queue: onto `parking` while a frame waits on
/// the job, back onto the main loop's queue with `None`.
pub(crate) fn park_channel_events(id: u64, parking: Option<&ParkingQueue>) {
    let events = with(id, |channel| channel.events.as_ptr());
    let parent = parking.map_or_else(crate::event::r#loop::main_loop_events, ParkingQueue::as_ptr);
    // SAFETY: the channel's own queue, a child, and a parent that outlives
    // the parking.
    unsafe { multiqueue_replace_parent(events, parent) };
}

// ---------------------------------------------------------------------------
// Callbacks: the id in, the channel layer out
// ---------------------------------------------------------------------------

/// A job's stdout, a socket's or stdio's data arrived.
///
/// # Safety
/// `RStream`'s read callback: `buf` is `count` readable bytes.
unsafe fn output_shim(
    _stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    // SAFETY: the caller's readable span, borrowed for the call.
    let bytes = unsafe { core::slice::from_raw_parts(buf.cast::<u8>(), count) };
    crate::channel::reader::on_channel_output(owner_id(data), false, bytes, eof)
}

/// A job's stderr arrived.
///
/// # Safety
/// As [`output_shim`].
unsafe fn stderr_shim(
    _stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    // SAFETY: as above.
    let bytes = unsafe { core::slice::from_raw_parts(buf.cast::<u8>(), count) };
    crate::channel::reader::on_channel_output(owner_id(data), true, bytes, eof)
}

/// A socket channel's stream finished closing: it held a reference.
fn stream_close_shim(_stream: *mut Stream, data: *mut c_void) {
    crate::channel::channel_decref(owner_id(data));
}

/// A job's child exited, or its handles closed.
fn job_exit_shim(_proc: *mut Proc, status: c_int, data: *mut c_void) {
    crate::channel::open::job_exited(owner_id(data), status);
}

/// A job's child was stopped or continued.
fn job_state_shim(_proc: *mut Proc, suspended: bool, data: *mut c_void) {
    crate::channel::open::job_state_changed(owner_id(data), suspended);
}
