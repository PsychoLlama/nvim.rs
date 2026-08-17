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
//! # Why this module stays on raw pointers
//!
//! Nothing here turns a `*mut Channel` into a `&mut Channel` that outlives a
//! call. A channel's address is handed to libuv, to the stream layer and to
//! the terminal as an opaque `data` word at setup time, so almost every
//! function in this file can be reentered *through a pointer this module did
//! not derive* — `close_cb`, `on_channel_data`, `term_close` all arrive that
//! way. A long-lived `&mut` would make each of those a distinct, conflicting
//! borrow. The narrow `unsafe` blocks below are therefore per-access, and the
//! module's globals are reached through the two named raw accessors
//! ([`channel_map`] and [`main_events`]) rather than `GlobalCell::with_mut`,
//! whose borrow tracking cannot be held across an event-loop turn.
//!
//! [`msgpack_rpc::channel`]: crate::msgpack_rpc::channel

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};

use crate::eval::typval::callback_free;
use crate::event::r#loop::one_arg_event;
use crate::event::multiqueue::{multiqueue_free, multiqueue_new_child, multiqueue_put_event};
use crate::event::proc::proc_free;
use crate::event::rstream::rstream_may_close;
use crate::event::stream::stream_may_close;
use crate::event::wstream::{wstream_new_buffer, wstream_write};
use crate::global_cell::GlobalCell;
use crate::lua::executor::api_free_luaref;
use crate::main::{channels, e_invchan, e_invstream, e_invstreamrpc, exiting, main_loop};
use crate::map::{map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::memory::{xfree, xmemdup};
use crate::msgpack_rpc::channel::call_stack::CallStack;
use crate::msgpack_rpc::channel::{rpc_close, rpc_free, rpc_init};
use crate::os::fs::os_write;
use crate::os::libc::{freopen, gettext, stderr};
use crate::os::pty_proc_unix::pty_proc_close_master;
use crate::terminal::{terminal_close, terminal_receive};
use crate::types::libc::STDERR_FILENO;
use crate::types::{
    Channel, ChannelPart, ChannelStreamType, Dict, InternalState, Loop, LuaRef, Map_uint64_t_ptr_t,
    MultiQueue, Proc, PtyProc, RStream, RpcState, Stream, ptr_t, size_t, uint64_t,
};

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {

    use super::{ChannelPart, ChannelStreamType};
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

    pub const kProcTypePty: c_int = 1;
    pub const LUA_NOREF: c_int = -2;

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
pub mod open;
pub mod reader;
pub mod term;

pub use info::{
    channel_all_info, channel_create_event, channel_info, channel_info_changed, channel_job_running,
};
pub use open::{channel_connect, channel_from_connection, channel_from_stdio, channel_job_start};
pub use reader::{
    callback_reader_free, callback_reader_start, channel_reader_callbacks, on_channel_data,
    on_job_stderr,
};
pub use term::channel_terminal_alloc;

/// The next dynamically allocated channel id. The first two are reserved.
static next_chan_id: GlobalCell<uint64_t> = GlobalCell::new((CHAN_STDERR + 1) as uint64_t);

// ---------------------------------------------------------------------------
// Globals, as this module reaches them
// ---------------------------------------------------------------------------

/// The channel registry: id → `*mut Channel`.
///
/// The one raw escape from `channels`. A `with_mut` borrow cannot be used
/// here: the map is walked while closing channels, and closing one queues an
/// event whose handler deletes from the same map.
fn channel_map() -> *mut Map_uint64_t_ptr_t {
    channels.ptr()
}

/// The event loop, as libuv and the stream layer take it.
pub(super) fn main_loop_ptr() -> *mut Loop {
    main_loop.ptr()
}

/// The main event queue, which is where deferred frees, autocommands and
/// anything else the caller must not run inline are put.
///
/// The tree's one reach for it, hence the visibility: `main_loop` itself
/// lives in the transpiled globals header, which the line cap has frozen.
pub(super) fn main_loop_events() -> *mut MultiQueue {
    // SAFETY: `main_loop` is a live, fully initialised `Loop` from
    // `event_init` until the process exits.
    unsafe { (*main_loop_ptr()).events }
}

/// The address of one of the shared `e_*` message strings.
///
/// They are plain `static`s -- `const char e_x[]` upstream, never written --
/// so a shared pointer is all any caller wants.
fn message<const N: usize>(msg: &'static [c_char; N]) -> *const c_char {
    msg.as_ptr()
}

/// The translated text of one of them.
pub(super) fn translated<const N: usize>(msg: &'static [c_char; N]) -> *const c_char {
    // SAFETY: gettext answers either its argument or a pointer into the loaded
    // message catalog; both outlive the call.
    unsafe { gettext(message(msg)) }
}

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
///
/// # Safety
/// `chan` points at a live channel whose transport is a job.
pub unsafe fn channel_proc(chan: *mut Channel) -> *mut Proc {
    debug_assert!(unsafe { (*chan).streamtype } == kChannelStreamProc);
    // SAFETY: the caller's live channel; `Channel_stream` is `repr(C)` and
    // `LibuvProc`/`PtyProc` both start with their `Proc`, so all three spell
    // the same address.
    unsafe { &raw mut (*chan).stream.proc }
}

/// The pty master behind a pty job.
///
/// # Safety
/// `chan` points at a live channel whose transport is a *pty* job.
pub unsafe fn channel_pty(chan: *mut Channel) -> *mut PtyProc {
    debug_assert!(unsafe { (*chan).streamtype } == kChannelStreamProc);
    debug_assert!(unsafe { (*chan).stream.proc.type_0 } as c_int == kProcTypePty);
    // SAFETY: as `channel_proc`, with the caller's stronger promise.
    unsafe { &raw mut (*chan).stream.pty }
}

/// The in-process endpoint behind an internal channel.
///
/// # Safety
/// `chan` points at a live channel whose transport is internal.
pub unsafe fn channel_internal(chan: *mut Channel) -> *mut InternalState {
    debug_assert!(unsafe { (*chan).streamtype } == kChannelStreamInternal);
    // SAFETY: the caller's live channel, projected to the active member.
    unsafe { &raw mut (*chan).stream.internal }
}

/// The stream this channel writes to.
///
/// Only the three transports with a real handle have one; the internal and
/// stderr channels are written to by other means and never reach here.
///
/// # Safety
/// `chan` points at a live channel with a write stream.
pub unsafe fn channel_instream(chan: *mut Channel) -> *mut Stream {
    // SAFETY: the caller's live channel, projected to the member its
    // `streamtype` says is active.
    unsafe {
        match (*chan).streamtype {
            kChannelStreamProc => &raw mut (*chan).stream.proc.in_0,
            kChannelStreamSocket => &raw mut (*chan).stream.socket.s,
            kChannelStreamStdio => &raw mut (*chan).stream.stdio.out,
            other => unreachable!("channel stream type {other} has no write stream"),
        }
    }
}

/// The stream this channel reads from. See [`channel_instream`].
///
/// # Safety
/// `chan` points at a live channel with a read stream.
pub unsafe fn channel_outstream(chan: *mut Channel) -> *mut RStream {
    // SAFETY: as `channel_instream`.
    unsafe {
        match (*chan).streamtype {
            kChannelStreamProc => &raw mut (*chan).stream.proc.out,
            kChannelStreamSocket => &raw mut (*chan).stream.socket,
            kChannelStreamStdio => &raw mut (*chan).stream.stdio.in_0,
            other => unreachable!("channel stream type {other} has no read stream"),
        }
    }
}

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

/// The channel registered under `id`, or null.
///
/// # Safety
/// The answer is only live until the next event-loop turn frees it.
pub unsafe fn find_channel(id: uint64_t) -> *mut Channel {
    let map = channel_map();
    // SAFETY: the registry is a live map for the process's lifetime.
    let slot = unsafe { mh_get_uint64_t(&raw mut (*map).set, id) };
    // The hash's "absent" slot index.
    if slot == u32::MAX {
        return ptr::null_mut();
    }
    // SAFETY: an occupied slot index is in range of the value array, and every
    // value in it was registered by `channel_alloc`.
    unsafe { *(*map).values.add(slot as usize) as *mut Channel }
}

/// Opens the stderr channel and the RPC event queue.
///
/// # Safety
/// Called once, from startup, after the event loop exists.
pub unsafe fn channel_init() {
    // SAFETY: the caller's promise.
    unsafe {
        channel_alloc(kChannelStreamStderr);
        rpc_init();
    }
}

/// A channel with no transport and no callbacks yet.
///
/// `Channel` owns heap state (the RPC call stack), so it is a real value
/// rather than a zeroed block — but the transport union and the three callback
/// slots stay zeroed, which is what their readers test for.
fn blank_channel(id: uint64_t, type_0: ChannelStreamType, events: *mut MultiQueue) -> Channel {
    Channel {
        id,
        refcount: 1,
        events,
        streamtype: type_0,
        // SAFETY: `Channel_stream` is a union of plain data; each transport's
        // setup writes its member whole before anything reads it, and the
        // stderr transport's one `bool` means "open" as zero.
        stream: unsafe { mem::zeroed() },
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
        // SAFETY: a zeroed `CallbackReader`/`Callback` is the "none" state —
        // `kCallbackNone` is 0 and every pointer in them is nullable.
        on_data: unsafe { mem::zeroed() },
        on_stderr: unsafe { mem::zeroed() },
        on_exit: unsafe { mem::zeroed() },
        exit_status: -1,
        callback_busy: false,
        callback_scheduled: false,
    }
}

/// Registers a new channel of `type_0` and hands out the caller's reference.
///
/// The stdio and stderr channels have fixed ids; everything else takes the
/// next one.
///
/// # Safety
/// The event loop exists; the answer is one owned reference.
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
    debug_assert!(id <= i64::MAX as uint64_t);
    // SAFETY: the main queue is live, and `multiqueue_new_child` hands back a
    // queue this channel owns until `channel_destroy` frees it.
    let events = unsafe { multiqueue_new_child(main_loop_events()) };
    let chan = Box::into_raw(Box::new(blank_channel(id, type_0, events)));
    // SAFETY: the registry is live, and `id` is fresh, so the slot the map
    // hands back is this channel's alone.
    unsafe {
        *map_put_ref_uint64_t_ptr_t(channel_map(), id, ptr::null_mut(), ptr::null_mut()) =
            chan as ptr_t
    };
    chan
}

/// Closes every channel. Called on exit.
///
/// # Safety
/// Called from the main thread with the registry live.
pub unsafe fn channel_teardown() {
    let map = channel_map();
    // SAFETY: the registry's key and value arrays are `n_keys` long, and every
    // value is a live channel. The ids are snapshotted rather than walked live
    // because closing a channel queues a free event against the same map.
    let ids: Vec<uint64_t> = unsafe {
        (0..(*map).set.h.n_keys as usize)
            .map(|i| (*(*(*map).values.add(i)).cast::<Channel>()).id)
            .collect()
    };
    for id in ids {
        // SAFETY: each id named a live channel a moment ago, and
        // `channel_close` tolerates one that has since gone.
        unsafe { channel_close(id, kChannelPartAll, ptr::null_mut()) };
    }
}

/// # Safety
/// `chan` is a live channel the caller already holds a reference to.
pub unsafe fn channel_incref(chan: *mut Channel) {
    // SAFETY: the caller's live channel.
    unsafe { (*chan).refcount += 1 };
}

/// Drops a reference, scheduling the free once the last one goes.
///
/// The free is deferred to an event because the caller is very often a libuv
/// callback still standing on the channel's own memory.
///
/// # Safety
/// `chan` is live and the caller owns the reference it is dropping.
pub unsafe fn channel_decref(chan: *mut Channel) {
    // SAFETY: the caller's live, owned reference.
    let last = unsafe {
        (*chan).refcount -= 1;
        (*chan).refcount == 0
    };
    if last {
        // SAFETY: the main queue is live and takes ownership of the event.
        unsafe {
            multiqueue_put_event(
                main_loop_events(),
                one_arg_event(Some(free_channel_event), chan.cast()),
            )
        };
    }
}

unsafe extern "C" fn free_channel_event(argv: *mut *mut c_void) {
    // SAFETY: the event carries the one remaining reference to a channel that
    // `channel_decref` dropped to zero.
    unsafe {
        let chan = (*argv).cast::<Channel>();
        map_del_uint64_t_ptr_t(channel_map(), (*chan).id, ptr::null_mut());
        channel_destroy(chan);
    }
}

/// Frees a channel and everything hanging off it.
///
/// # Safety
/// `chan` has no references left and is no longer in the registry.
unsafe fn channel_destroy(chan: *mut Channel) {
    // SAFETY: the caller's unreferenced channel; each `free` below matches the
    // setup that ran when the corresponding feature was turned on.
    unsafe {
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
}

/// Unwinds a channel that never got off the ground.
///
/// Only valid for the channel that took the most recent id, and only before
/// anything else has taken a reference — which is what the two assertions say.
/// Giving the id back keeps `job_spec`'s expectations about consecutive ids.
///
/// # Safety
/// `chan` is the most recently allocated channel and is unreferenced.
pub(super) unsafe fn channel_destroy_early(chan: *mut Channel) {
    next_chan_id.set(next_chan_id.get().wrapping_sub(1));
    // SAFETY: the caller's channel, still registered and still theirs alone.
    unsafe {
        assert!(
            (*chan).id == next_chan_id.get(),
            "channel id was not the last"
        );
        map_del_uint64_t_ptr_t(channel_map(), (*chan).id, ptr::null_mut());
        (*chan).id = 0;
        (*chan).refcount -= 1;
        assert!((*chan).refcount == 0, "channel was already referenced");
        multiqueue_put_event(
            main_loop_events(),
            one_arg_event(Some(free_channel_event), chan.cast()),
        );
    }
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
pub(super) unsafe extern "C" fn close_cb(_stream: *mut Stream, data: *mut c_void) {
    // SAFETY: `data` is the channel the stream was set up with, and the stream
    // held one reference to it.
    unsafe { channel_decref(data.cast()) };
}

// ---------------------------------------------------------------------------
// Closing
// ---------------------------------------------------------------------------

/// Closes `part` of channel `id`, reporting why not through `error`.
///
/// Which parts exist depends on the transport, and an RPC channel only accepts
/// `kChannelPartRpc`/`kChannelPartAll` — its stdin and stdout carry the
/// protocol and cannot be closed separately.
///
/// # Safety
/// Called from the main thread; `error`, if given, is writable.
pub unsafe fn channel_close(id: uint64_t, part: ChannelPart, error: *mut *const c_char) -> bool {
    // SAFETY: the caller's promise.
    match unsafe { close_channel_part(id, part) } {
        Ok(()) => true,
        Err(msg) => {
            if !error.is_null() {
                // SAFETY: the caller's writable out-parameter.
                unsafe { *error = msg };
            }
            false
        }
    }
}

/// [`channel_close`] with the out-parameter turned back into a result.
///
/// # Safety
/// Called from the main thread with the registry live.
unsafe fn close_channel_part(id: uint64_t, part: ChannelPart) -> Result<(), *const c_char> {
    // SAFETY: the caller's promise; the answer is used before the next turn.
    let chan = unsafe { find_channel(id) };
    if chan.is_null() {
        // An id below the watermark named a channel that has already gone,
        // which is not an error: closing twice is allowed.
        return if id < next_chan_id.get() {
            Ok(())
        } else {
            Err(message(&e_invchan))
        };
    }
    // SAFETY: a live registry entry.
    let (streamtype, is_rpc) = unsafe { ((*chan).streamtype, (*chan).is_rpc) };

    let close_main = part == kChannelPartRpc || part == kChannelPartAll;
    if close_main {
        if is_rpc {
            // SAFETY: an RPC channel's protocol state is live while it is.
            unsafe { rpc_close(chan) };
        } else if part == kChannelPartRpc {
            return Err(message(&e_invstream));
        }
    } else if (part == kChannelPartStdin || part == kChannelPartStdout) && is_rpc {
        return Err(message(&e_invstreamrpc));
    }

    // SAFETY: `chan` is live and each arm touches only the transport its
    // `streamtype` selected.
    unsafe {
        match streamtype {
            kChannelStreamSocket => {
                if !close_main {
                    return Err(message(&e_invstream));
                }
                rstream_may_close(&raw mut (*chan).stream.socket);
            }
            kChannelStreamProc => close_job_parts(chan, part, close_main),
            kChannelStreamStdio => close_stdio_parts(chan, part, close_main)?,
            kChannelStreamStderr => close_stderr(chan, part)?,
            kChannelStreamInternal => {
                if !close_main {
                    return Err(message(&e_invstream));
                }
                close_internal(chan);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Closes the requested halves of a job's pipes, and its pty master last.
///
/// # Safety
/// `chan` is a live job channel.
unsafe fn close_job_parts(chan: *mut Channel, part: ChannelPart, close_main: bool) {
    // SAFETY: the caller's live job channel.
    unsafe {
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
}

/// Closes this process's own stdin/stdout.
///
/// # Safety
/// `chan` is a live stdio channel.
unsafe fn close_stdio_parts(
    chan: *mut Channel,
    part: ChannelPart,
    close_main: bool,
) -> Result<(), *const c_char> {
    // This process's stderr belongs to the stderr channel, not here.
    if part == kChannelPartStderr {
        return Err(message(&e_invstream));
    }
    // SAFETY: the caller's live stdio channel.
    unsafe {
        if part == kChannelPartStdin || close_main {
            rstream_may_close(&raw mut (*chan).stream.stdio.in_0);
        }
        if part == kChannelPartStdout || close_main {
            stream_may_close(&raw mut (*chan).stream.stdio.out);
        }
    }
    Ok(())
}

/// Closes the stderr channel, which is a descriptor rather than a stream.
///
/// # Safety
/// `chan` is the live stderr channel.
unsafe fn close_stderr(chan: *mut Channel, part: ChannelPart) -> Result<(), *const c_char> {
    if part != kChannelPartAll && part != kChannelPartStderr {
        return Err(message(&e_invstream));
    }
    // SAFETY: the caller's live stderr channel, whose transport is one flag.
    unsafe {
        if (*chan).stream.err.closed {
            return Ok(());
        }
        (*chan).stream.err.closed = true;
        // On the way out the descriptor is about to go anyway, and reopening
        // it would swallow anything still being written.
        if !exiting.get() {
            freopen(c"/dev/null".as_ptr(), c"w".as_ptr(), stderr);
        }
        channel_decref(chan);
    }
    Ok(())
}

/// Closes an internal channel.
///
/// One with a terminal is closed by closing the terminal; without one there is
/// nothing to release but the reference.
///
/// # Safety
/// `chan` is a live internal channel.
unsafe fn close_internal(chan: *mut Channel) {
    // SAFETY: the caller's live internal channel.
    unsafe {
        if (*chan).term.is_null() {
            channel_decref(chan);
            return;
        }
        let internal = channel_internal(chan);
        api_free_luaref((*internal).cb);
        (*internal).cb = LUA_NOREF as LuaRef;
        (*internal).closed = true;
        terminal_close(&raw mut (*chan).term, 0);
        (*chan).exit_status = 0;
    }
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// Writes raw bytes to a channel, returning how many were accepted.
///
/// `data_owned` hands this function the buffer: it is either passed straight
/// to the write queue or freed here.
///
/// # Safety
/// `data` is `len` readable bytes, owned by this call when `data_owned`;
/// `error` is writable.
pub unsafe fn channel_send(
    id: uint64_t,
    data: *mut c_char,
    len: size_t,
    data_owned: bool,
    error: *mut *const c_char,
) -> size_t {
    // SAFETY: the caller's promise. Every arm either hands `data` to the write
    // queue and returns, or falls through to the free below.
    unsafe {
        let chan = find_channel(id);
        if chan.is_null() {
            *error = translated(&e_invchan);
        } else {
            match send_to_channel(chan, data, len, data_owned) {
                Ok(Sent::Bytes(n)) => {
                    if data_owned {
                        xfree(data.cast());
                    }
                    return n;
                }
                Ok(Sent::Queued(n)) => return n,
                Err(msg) => *error = gettext(msg.as_ptr()),
            }
        }
        if data_owned {
            xfree(data.cast());
        }
        0
    }
}

/// How much of a [`channel_send`] got through, and who owns the buffer now.
enum Sent {
    /// Written here; the caller still owns the buffer.
    Bytes(size_t),
    /// Handed to the write queue, which owns the buffer.
    Queued(size_t),
}

/// The transport-specific half of [`channel_send`].
///
/// # Safety
/// As [`channel_send`], with `chan` live.
unsafe fn send_to_channel(
    chan: *mut Channel,
    data: *mut c_char,
    len: size_t,
    data_owned: bool,
) -> Result<Sent, &'static CStr> {
    const CLOSED: &CStr = c"Can't send data to closed stream";
    const RAW_TO_RPC: &CStr = c"Can't send raw data to rpc channel";

    // SAFETY: the caller's live channel and readable buffer.
    unsafe {
        match (*chan).streamtype {
            kChannelStreamStderr => {
                if (*chan).stream.err.closed {
                    return Err(CLOSED);
                }
                // stderr is not on the event loop; it is written synchronously
                // and a short write is reported as such.
                let wres = os_write(STDERR_FILENO, data, len, false);
                Ok(Sent::Bytes(if wres >= 0 { wres as size_t } else { 0 }))
            }
            kChannelStreamInternal => {
                if (*chan).is_rpc {
                    return Err(RAW_TO_RPC);
                }
                if (*chan).term.is_null() || (*channel_internal(chan)).closed {
                    return Err(CLOSED);
                }
                terminal_receive((*chan).term, data, len);
                Ok(Sent::Bytes(len))
            }
            _ => {
                let in_0 = channel_instream(chan);
                if (*in_0).closed {
                    return Err(CLOSED);
                }
                if (*chan).is_rpc {
                    return Err(RAW_TO_RPC);
                }
                let owned = if data_owned {
                    data.cast::<c_void>()
                } else {
                    xmemdup(data.cast(), len)
                };
                let buf = wstream_new_buffer(owned.cast(), len, 1, Some(xfree));
                // The write queue owns the buffer from here either way.
                Ok(Sent::Queued(if wstream_write(in_0, buf) == 0 {
                    len
                } else {
                    0
                }))
            }
        }
    }
}
