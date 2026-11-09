//! Channels: the editor's connections to child processes, sockets, its own
//! standard streams and its own terminal emulator.
//!
//! A channel is a numbered record over one of five transports
//! ([`ChannelStream`]), plus optional Vimscript callbacks and an optional
//! msgpack-rpc layer ([`msgpack_rpc::channel`]).
//!
//! # Ownership
//!
//! The channel table ([`CHANNELS`]) owns every channel, boxed, by its id.
//! Editor code names a channel by id and borrows it from the table for one
//! step at a time ([`with_channel`]); nothing holds that borrow across a
//! call that can run user code, because a channel's callbacks run Vimscript
//! and Lua, which can open and close channels. A step that has to run user
//! code copies out what it needs first and looks the channel up again after.
//!
//! Lifetime is by reference count: a channel stays in the table while its
//! transport's handles, a queued event, a terminal or a waiting frame
//! stands on it, and every reference is dropped through an event rather than
//! inline -- the last one queues the free. Channel ids are never reused
//! (bar the one a failed start gives back before anything saw it), so an id
//! kept past its channel finds nothing rather than the next tenant.
//!
//! libuv keeps the addresses of the transport's handles; the stream and
//! process callbacks are told the channel's id. That half lives in
//! [`crate::event::channel`], which is the only code that reaches a channel
//! by address.
//!
//! [`msgpack_rpc::channel`]: crate::msgpack_rpc::channel

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::lua::ffi::LUA_NOREF;
use core::ffi::{CStr, c_int};

use crate::channel::defs::{
    CHANNEL_STREAM_INTERNAL, CHANNEL_STREAM_PROC, CHANNEL_STREAM_SOCKET, CHANNEL_STREAM_STDERR,
    CHANNEL_STREAM_STDIO,
};
use crate::channel::defs::{Channel, ChannelPart, ChannelStream, StderrState};
use crate::eval::typval::callback_free;
use crate::event::channel::TransportMut;
use crate::event::channel::{process_channel_events, wait_job};
use crate::event_queue::{ChildQueue, put_job_on_main_loop};
use crate::global_cell::GlobalCell;
use crate::lua::executor::release_luaref;
use crate::message::{E_INVCHAN, E_INVSTREAM, E_INVSTREAMRPC};
use crate::msgpack_rpc::channel::{rpc_close, rpc_free};
use crate::os::cshim::gettext;
use crate::os::fs::write_bytes;
use crate::os::libc::uint64_t;
use crate::os::proc::os_stderr_to_null;
use crate::registry::SlotTable;
use crate::startup::EXITING;
use crate::terminal::{terminal_close, terminal_receive};
use crate::types::{Failed, LuaRef};
use std::os::fd::AsFd as _;

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use super::ChannelPart;

    pub(super) const CHANNEL_PART_STDIN: ChannelPart = 0;
    pub(super) const CHANNEL_PART_STDOUT: ChannelPart = 1;
    pub(super) const CHANNEL_PART_STDERR: ChannelPart = 2;
    pub(super) const CHANNEL_PART_RPC: ChannelPart = 3;
    pub(super) const CHANNEL_PART_ALL: ChannelPart = 4;
}

use known::*;

/// The id of the channel over this process's own stdin/stdout.
pub(crate) const CHAN_STDIO: u64 = 1;
/// The id of the channel over this process's own stderr.
pub(crate) const CHAN_STDERR: u64 = 2;

pub mod defs;
pub mod info;
pub mod open;
pub mod reader;
pub mod term;

pub(crate) use info::channel_stream_type;
pub use info::{
    channel_all_info, channel_create_event, channel_info, channel_info_changed, channel_job_running,
};
pub(crate) use open::rpc_job_start;
pub use open::{channel_connect, channel_from_connection, channel_from_stdio, channel_job_start};
pub use reader::{callback_reader_free, callback_reader_start};
pub use term::channel_terminal_alloc;

/// Every open channel, by id, boxed: libuv keeps the addresses of the
/// handles inside one, so a channel never moves while it is registered.
/// See [`crate::registry`] for the order this keeps.
pub(crate) static CHANNELS: GlobalCell<SlotTable<uint64_t, Box<Channel>>> =
    GlobalCell::new(SlotTable::new());

/// The next dynamically allocated channel id. The first two are reserved.
static NEXT_CHAN_ID: GlobalCell<uint64_t> = GlobalCell::new(CHAN_STDERR + 1);

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

/// Run `f` on channel `id`, borrowed from the table, if it is registered.
///
/// The borrow ends with `f`, which must not run user code -- no callback,
/// no autocommand, no event loop, no RPC -- and must not reach the table
/// again.
pub(crate) fn with_channel<R>(id: uint64_t, f: impl FnOnce(&mut Channel) -> R) -> Option<R> {
    CHANNELS.with_mut(|table| table.get_mut(&id).map(|channel| f(channel)))
}

/// Whether channel `id` is registered.
pub(crate) fn channel_exists(id: uint64_t) -> bool {
    with_channel(id, |_| ()).is_some()
}

/// Opens the stderr channel and the RPC event queue.
pub fn channel_init() {
    let _ = alloc_channel(Some(CHAN_STDERR), |_| {
        ChannelStream::Stderr(StderrState { closed: false })
    });
    crate::msgpack_rpc::channel::rpc_init();
}

/// Registers a new channel over the transport `stream` builds -- told the
/// channel's id -- and hands out the caller's reference. `fixed` names the
/// stdio and stderr channels' reserved ids; everything else takes the next.
pub(crate) fn alloc_channel(
    fixed: Option<uint64_t>,
    stream: impl FnOnce(uint64_t) -> ChannelStream,
) -> uint64_t {
    let id = fixed.unwrap_or_else(|| {
        let id = NEXT_CHAN_ID.get();
        NEXT_CHAN_ID.set(id.wrapping_add(1));
        id
    });
    // Channel ids are handed to Vimscript as Numbers.
    debug_assert!(i64::try_from(id).is_ok());
    let channel = Box::new(Channel::new(id, ChildQueue::of_main_loop(), stream(id)));
    CHANNELS.with_mut(|table| table.insert(id, channel));
    id
}

/// Closes every channel. Called on exit.
pub fn channel_teardown() {
    // The ids are snapshotted rather than walked live because closing a
    // channel queues a free against the same table.
    for id in CHANNELS.with(SlotTable::snapshot_keys) {
        let _ = channel_close(id, CHANNEL_PART_ALL);
    }
}

/// A reference to a channel, held for as long as this value is: the
/// channel stays in the table, at its address, until it is dropped.
#[derive(Debug)]
pub(crate) struct ChannelRef(uint64_t);

impl ChannelRef {
    /// A reference to channel `id`, if it is registered.
    pub(crate) fn new(id: uint64_t) -> Option<Self> {
        with_channel(id, |channel| channel.refcount.retain())?;
        Some(ChannelRef(id))
    }

    /// The channel's id.
    pub(crate) fn id(&self) -> uint64_t {
        self.0
    }

    /// Give the reference up to whoever will drop it with [`channel_decref`]
    /// -- an event, a handle.
    pub(crate) fn into_raw(self) -> uint64_t {
        core::mem::ManuallyDrop::new(self).0
    }
}

impl Drop for ChannelRef {
    fn drop(&mut self) {
        channel_decref(self.0);
    }
}

/// Take a reference to channel `id`, for a holder that is not a
/// [`ChannelRef`] (`msgpack_rpc/`'s requests and calls).
pub fn channel_incref(id: uint64_t) {
    with_channel(id, |channel| channel.refcount.retain()).expect("a reference to a live channel");
}

/// Drops a reference to channel `id`, scheduling the free once the last one
/// goes.
///
/// The free is deferred to an event because the caller is very often a
/// libuv callback still standing on the channel's own memory.
pub fn channel_decref(id: uint64_t) {
    let last = with_channel(id, |channel| channel.refcount.release() == 0)
        .expect("a reference to a live channel");
    if last {
        channel_free_later(id);
    }
}

/// Queue the free of channel `id`, whose last reference has gone: for the
/// holders that count references on the channel themselves
/// (`msgpack_rpc/`'s `Chan`).
pub(crate) fn channel_free_later(id: uint64_t) {
    put_job_on_main_loop(Box::new(move || {
        if let Some(channel) = CHANNELS.with_mut(|table| table.remove(&id)) {
            channel_destroy(channel);
        }
    }));
}

/// Frees a channel and everything hanging off it. It is out of the table
/// and nothing references it.
fn channel_destroy(mut channel: Box<Channel>) {
    if channel.is_rpc {
        rpc_free(&mut channel);
    }
    if let Some(job) = channel.stream_mut().job() {
        job.free_argv();
    }
    callback_reader_free(&mut channel.on_data);
    callback_reader_free(&mut channel.on_stderr);
    callback_free(&mut channel.on_exit);
    // The box and its queue go with it.
}

/// Unwinds a channel that never got off the ground.
///
/// Only valid for the channel that took the most recent id, and only before
/// anything else has taken a reference — which is what the two assertions
/// say. Giving the id back keeps `job_spec`'s expectations about
/// consecutive ids.
pub(super) fn channel_destroy_early(id: uint64_t) {
    NEXT_CHAN_ID.set(NEXT_CHAN_ID.get().wrapping_sub(1));
    assert!(id == NEXT_CHAN_ID.get(), "channel id was not the last");
    let mut channel = CHANNELS
        .with_mut(|table| table.remove(&id))
        .expect("the channel being unwound is registered");
    channel.id = 0;
    assert!(
        channel.refcount.release() == 0,
        "channel was already referenced"
    );
    // Freed from the loop, not here: a handle the failed start closed may
    // still be finishing its close.
    put_job_on_main_loop(Box::new(move || channel_destroy(channel)));
}

// ---------------------------------------------------------------------------
// Closing
// ---------------------------------------------------------------------------

/// Why a channel, or a part of one, could not be closed: one of the three
/// shared `e_*` messages the C wrote through its `char **error`.
pub(crate) type CloseError = &'static CStr;

/// Closes `part` of channel `id`, or says why not.
///
/// Which parts exist depends on the transport, and an RPC channel only
/// accepts `CHANNEL_PART_RPC`/`CHANNEL_PART_ALL` — its stdin and stdout carry
/// the protocol and cannot be closed separately. Closing a channel that has
/// already gone is not an error: closing twice is allowed.
pub fn channel_close(id: uint64_t, part: ChannelPart) -> Result<(), CloseError> {
    let Some((kind, is_rpc)) = with_channel(id, |channel| (channel.streamtype(), channel.is_rpc))
    else {
        // An id below the watermark named a channel that has already gone.
        return if id < NEXT_CHAN_ID.get() {
            Ok(())
        } else {
            Err(E_INVCHAN)
        };
    };

    let close_main = part == CHANNEL_PART_RPC || part == CHANNEL_PART_ALL;
    if close_main {
        if is_rpc {
            rpc_close(id);
        } else if part == CHANNEL_PART_RPC {
            return Err(E_INVSTREAM);
        }
    } else if (part == CHANNEL_PART_STDIN || part == CHANNEL_PART_STDOUT) && is_rpc {
        return Err(E_INVSTREAMRPC);
    }

    match kind {
        CHANNEL_STREAM_SOCKET => {
            if !close_main {
                return Err(E_INVSTREAM);
            }
            with_channel(id, |channel| {
                if let TransportMut::Socket(socket) = channel.stream_mut().project() {
                    socket.close();
                }
            });
        }
        CHANNEL_STREAM_PROC => close_job_parts(id, part, close_main),
        CHANNEL_STREAM_STDIO => close_stdio_parts(id, part, close_main)?,
        CHANNEL_STREAM_STDERR => close_stderr(id, part)?,
        CHANNEL_STREAM_INTERNAL => {
            if !close_main {
                return Err(E_INVSTREAM);
            }
            close_internal(id);
        }
        _ => {}
    }
    Ok(())
}

/// [`channel_close`], reporting why a refused close was refused.
pub(crate) fn channel_close_or_report(id: uint64_t, part: ChannelPart) -> Result<(), Failed> {
    channel_close(id, part).map_err(|why| {
        crate::message::emsg(why);
        Failed
    })
}

/// Closes the requested halves of a job's pipes, and its pty master last.
fn close_job_parts(id: uint64_t, part: ChannelPart, close_main: bool) {
    with_channel(id, |channel| {
        if let Some(mut job) = channel.stream_mut().job() {
            job.as_mut().close_streams(
                part == CHANNEL_PART_STDIN || close_main,
                part == CHANNEL_PART_STDOUT || close_main,
                part == CHANNEL_PART_STDERR || part == CHANNEL_PART_ALL,
            );
            if part == CHANNEL_PART_ALL {
                job.close_master();
            }
        }
    });
}

/// Closes this process's own stdin/stdout.
fn close_stdio_parts(id: uint64_t, part: ChannelPart, close_main: bool) -> Result<(), CloseError> {
    // This process's stderr belongs to the stderr channel, not here.
    if part == CHANNEL_PART_STDERR {
        return Err(E_INVSTREAM);
    }
    with_channel(id, |channel| {
        if let TransportMut::Stdio(stdin, stdout) = channel.stream_mut().project() {
            if part == CHANNEL_PART_STDIN || close_main {
                stdin.close();
            }
            if part == CHANNEL_PART_STDOUT || close_main {
                stdout.close();
            }
        }
    });
    Ok(())
}

/// Closes the stderr channel, which is a descriptor rather than a stream.
fn close_stderr(id: uint64_t, part: ChannelPart) -> Result<(), CloseError> {
    if part != CHANNEL_PART_ALL && part != CHANNEL_PART_STDERR {
        return Err(E_INVSTREAM);
    }
    let was_open = with_channel(id, |channel| match channel.stream_mut().project() {
        TransportMut::Stderr(err) => !core::mem::replace(&mut err.closed, true),
        _ => false,
    })
    .unwrap_or(false);
    if !was_open {
        return Ok(());
    }
    // On the way out the descriptor is about to go anyway, and reopening
    // it would swallow anything still being written.
    if !EXITING.get() {
        os_stderr_to_null();
    }
    channel_decref(id);
    Ok(())
}

/// Closes an internal channel.
///
/// One with a terminal is closed by closing the terminal; without one there
/// is nothing to release but the reference.
fn close_internal(id: uint64_t) {
    let Some(term) = with_channel(id, |channel| channel.term).flatten() else {
        channel_decref(id);
        return;
    };
    let cb = with_channel(id, |channel| {
        channel.exit_status = 0;
        match channel.stream_mut().internal() {
            Some(internal) => {
                internal.closed = true;
                core::mem::replace(&mut internal.cb, LUA_NOREF as LuaRef)
            }
            None => LUA_NOREF as LuaRef,
        }
    });
    if let Some(cb) = cb {
        release_luaref(cb);
    }
    // `TermClose` runs from here, with nothing of the channel borrowed.
    terminal_close(term, 0);
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// Writes `data` to channel `id`, which takes it over, answering how many
/// bytes were accepted, or why none were (a translated message).
pub fn channel_send(id: uint64_t, data: Vec<u8>) -> Result<usize, &'static CStr> {
    const CLOSED: &CStr = c"Can't send data to closed stream";
    const RAW_TO_RPC: &CStr = c"Can't send raw data to rpc channel";

    #[derive(Debug)]
    enum Route {
        Stderr,
        Terminal(crate::terminal::Term),
        Written(bool, usize),
    }
    let mut data = data;
    let route = with_channel(id, |channel| {
        let is_rpc = channel.is_rpc;
        let term = channel.term;
        match channel.stream() {
            ChannelStream::Stderr(err) => {
                if err.closed {
                    Err(CLOSED)
                } else {
                    Ok(Route::Stderr)
                }
            }
            ChannelStream::Internal(internal) => {
                if is_rpc {
                    Err(RAW_TO_RPC)
                } else if internal.closed {
                    Err(CLOSED)
                } else {
                    term.map(Route::Terminal).ok_or(CLOSED)
                }
            }
            _ => {
                let out = channel.stream_mut().write_stream();
                let out = out.expect("every other transport writes");
                if out.closed {
                    Err(CLOSED)
                } else if is_rpc {
                    Err(RAW_TO_RPC)
                } else {
                    let len = data.len();
                    Ok(Route::Written(
                        out.write_owned(core::mem::take(&mut data)),
                        len,
                    ))
                }
            }
        }
    });
    match route {
        None => Err(gettext(E_INVCHAN)),
        Some(Err(msg)) => Err(gettext(msg)),
        // stderr is not on the event loop; it is written synchronously and
        // a short write is reported as such.
        Some(Ok(Route::Stderr)) => {
            Ok(usize::try_from(write_bytes(std::io::stderr().as_fd(), &data, false)).unwrap_or(0))
        }
        Some(Ok(Route::Terminal(term))) => {
            terminal_receive(term, &data);
            Ok(data.len())
        }
        Some(Ok(Route::Written(queued, len))) => Ok(if queued { len } else { 0 }),
    }
}

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

/// Stop the job behind channel `id` and wait `ms` for it to go: the wait's
/// answer, or `None` when there is no such job any more.
pub(crate) fn channel_job_stop_wait(id: uint64_t, ms: c_int) -> Option<c_int> {
    let held = ChannelRef::new(id)?;
    let is_job = with_channel(id, |channel| match channel.stream_mut().job() {
        Some(job) => {
            job.stop();
            true
        }
        None => false,
    })?;
    let status = is_job.then(|| wait_job(held.id(), ms, None));
    drop(held);
    status
}

/// Run what channel `id` has queued, if it is still there.
pub(crate) fn channel_process_events(id: uint64_t) {
    if channel_exists(id) {
        process_channel_events(id);
    }
}
