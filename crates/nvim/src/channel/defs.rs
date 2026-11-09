#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::event::libuv_proc::LibuvProc;
use crate::event::objects::{Proc, RStream, Stream};
use crate::os::libc::{uint32_t, uint64_t};
use crate::os::pty_proc_unix::PtyProc;
use crate::types::{ApiDict, Callback, LuaRef, Object, RefcountSize};
use core::ffi::CStr;
use core::pin::Pin;

use crate::msgpack_rpc::channel::call_stack::CallStack;

use crate::eval::typval::DictRef;

use crate::msgpack_rpc::unpacker::Unpacker;

/// Where a channel's `on_stdout`/`on_stderr`/`on_data` output goes.
#[derive(Debug)]
pub struct CallbackReader {
    pub cb: Callback,
    /// The dictionary a buffered reader with no callback collects into
    /// (the job's options), holding a reference of its own.
    pub self_dict: Option<DictRef>,
    /// What the stream has produced since the last delivery.
    pub buffer: Vec<u8>,
    pub eof: bool,
    pub buffered: bool,
    pub fwd_err: bool,
    /// The stream's name in the callback and in `self_dict`: `stdout`,
    /// `stderr`, `data` or `stdin`.
    pub stream_name: &'static CStr,
}

impl CallbackReader {
    /// A reader nobody is listening to: no callback, no `self` dict, no
    /// accumulated bytes. Every reader starts here.
    pub const fn none() -> Self {
        Self {
            cb: Callback::None,
            self_dict: None,
            buffer: Vec::new(),
            eof: false,
            buffered: false,
            fwd_err: false,
            stream_name: c"",
        }
    }

    /// Whether the reader has anywhere to deliver to.
    pub fn is_set(&self) -> bool {
        self.cb.is_set() || self.self_dict.is_some()
    }
}

impl Default for CallbackReader {
    fn default() -> Self {
        Self::none()
    }
}

/// A channel: a numbered connection over one of five transports, owned by
/// the channel table (`channel::channels`).
///
/// The transport is registered with libuv *by address* -- its handles'
/// `data` fields point into it -- so it lives in a pinned box of its own,
/// private to this type: outside `event/` it is read through
/// [`Channel::stream`] and changed only through the methods
/// `event/channel.rs` defines on [`Pin<&mut ChannelStream>`](Pin), which is
/// the one module that hands those addresses to libuv. Nothing else can move
/// a transport out of its box or swap two of them.
#[derive(Debug)]
pub struct Channel {
    pub id: uint64_t,
    /// Who still stands on the channel: its transport's open handles, queued
    /// events, a terminal, a waiting frame. The last release queues the free.
    pub refcount: RefcountSize,
    /// The channel's own event queue, a child of the main loop's.
    pub(crate) events: crate::event_queue::ChildQueue,
    stream: Pin<Box<ChannelStream>>,
    pub is_rpc: bool,
    pub detach: bool,
    pub rpc: RpcState,
    /// The terminal this channel drives, if any.
    pub(crate) term: Option<crate::terminal::Term>,
    pub on_data: CallbackReader,
    pub on_stderr: CallbackReader,
    pub on_exit: Callback,
    pub exit_status: ::core::ffi::c_int,
    pub callback_busy: bool,
    pub callback_scheduled: bool,
}

impl Channel {
    /// A channel over `stream`, with one reference (the caller's), no RPC
    /// state and no callbacks.
    pub(crate) fn new(
        id: uint64_t,
        events: crate::event_queue::ChildQueue,
        stream: ChannelStream,
    ) -> Self {
        Self {
            id,
            refcount: RefcountSize::ONE,
            events,
            stream: Box::pin(stream),
            is_rpc: false,
            detach: false,
            rpc: RpcState {
                closed: false,
                unpacker: None,
                next_request_id: 0,
                call_stack: CallStack::new(),
                info: ApiDict::new(),
                client_type: 0,
            },
            term: None,
            on_data: CallbackReader::none(),
            on_stderr: CallbackReader::none(),
            on_exit: Callback::None,
            exit_status: -1,
            callback_busy: false,
            callback_scheduled: false,
        }
    }

    /// Which of the five transports this is, as the C enum numbers them.
    pub fn streamtype(&self) -> ChannelStreamType {
        self.stream.kind()
    }

    /// The transport, to read.
    pub fn stream(&self) -> &ChannelStream {
        &self.stream
    }

    /// The transport, to change through `event/channel.rs`'s methods. Pinned:
    /// libuv holds the addresses inside it.
    pub fn stream_mut(&mut self) -> Pin<&mut ChannelStream> {
        self.stream.as_mut()
    }
}

/// A channel's transport.
///
/// Not `Unpin` (every variant with a libuv handle embeds a `Stream`, which
/// is not): a channel keeps it in a pinned box, see [`Channel`].
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum ChannelStream {
    /// A child process, with up to three pipes or a pty master.
    Job(JobProc),
    /// A socket this process dialled or accepted.
    Socket(RStream),
    /// This process's own stdin and stdout.
    Stdio(StdioPair),
    /// This process's own stderr, written synchronously.
    Stderr(StderrState),
    /// No transport: an `nvim_open_term()` terminal or a connection to this
    /// process's own server.
    Internal(InternalState),
}

impl ChannelStream {
    /// The transport's kind, as the C enum numbers it.
    pub fn kind(&self) -> ChannelStreamType {
        match self {
            ChannelStream::Job(_) => CHANNEL_STREAM_PROC,
            ChannelStream::Socket(_) => CHANNEL_STREAM_SOCKET,
            ChannelStream::Stdio(_) => CHANNEL_STREAM_STDIO,
            ChannelStream::Stderr(_) => CHANNEL_STREAM_STDERR,
            ChannelStream::Internal(_) => CHANNEL_STREAM_INTERNAL,
        }
    }
}

/// A job's child, started through libuv or behind a pty. Both begin with
/// the `Proc` the process layer works on.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum JobProc {
    Uv(LibuvProc),
    Pty(PtyProc),
}

impl JobProc {
    /// The part of the child that does not depend on how it was started.
    pub fn proc(&self) -> &Proc {
        match self {
            JobProc::Uv(uv) => &uv.proc,
            JobProc::Pty(pty) => &pty.proc,
        }
    }

    /// The pty, for a job started behind one.
    pub fn pty(&self) -> Option<&PtyProc> {
        match self {
            JobProc::Pty(pty) => Some(pty),
            JobProc::Uv(_) => None,
        }
    }
}

/// The kinds of transport, as the C enum numbers them.
pub const CHANNEL_STREAM_PROC: ChannelStreamType = 0;
pub const CHANNEL_STREAM_SOCKET: ChannelStreamType = 1;
pub const CHANNEL_STREAM_STDIO: ChannelStreamType = 2;
pub const CHANNEL_STREAM_STDERR: ChannelStreamType = 3;
pub const CHANNEL_STREAM_INTERNAL: ChannelStreamType = 4;

#[derive(Debug)]
pub struct ChannelCallFrame {
    pub request_id: uint32_t,
    pub returned: bool,
    pub errored: bool,
    pub result: Object,
}
/// Which of a channel's streams a close applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelPart {
    Stdin,
    Stdout,
    Stderr,
    /// The RPC stream: stdin and stdout of an RPC channel together.
    Rpc,
    All,
}
pub type ChannelStdinMode = ::core::ffi::c_uint;
/// What a channel does with the job's stdin.
pub const CHANNEL_STDIN_PIPE: ChannelStdinMode = 0;
pub const CHANNEL_STDIN_NULL: ChannelStdinMode = 1;
pub type ChannelStreamType = ::core::ffi::c_uint;
pub type ClientType = ::core::ffi::c_int;
#[derive(Copy, Clone, Debug)]
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
}
#[derive(Debug)]
pub struct RpcState {
    pub closed: bool,
    pub unpacker: Option<Box<Unpacker>>,
    pub next_request_id: uint32_t,
    /// Requests this editor has sent and is still waiting on.
    pub call_stack: CallStack,
    pub info: ApiDict,
    pub client_type: ClientType,
}
#[derive(Copy, Clone, Debug)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Debug)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
}
