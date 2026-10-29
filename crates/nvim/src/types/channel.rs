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

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use core::ffi::CStr;

pub use crate::msgpack_rpc::channel::call_stack::CallStack;

use super::*;
use crate::eval::typval::DictRef;

pub use crate::msgpack_rpc::unpacker::Unpacker;

/// Where a channel's `on_stdout`/`on_stderr`/`on_data` output goes.
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
/// the channel table (`channel::channels`) at a fixed address.
///
/// The transport is registered with libuv *by address* -- its handles'
/// `data` fields point into it -- which is why the table boxes every
/// channel and nothing moves one out of its box: `event/channel.rs` is the
/// one module that hands those addresses to libuv, and it reaches the
/// transport by the channel's id.
pub struct Channel {
    pub id: uint64_t,
    /// Who still stands on the channel: its transport's open handles, queued
    /// events, a terminal, a waiting frame. The last release queues the free.
    pub refcount: RefcountSize,
    /// The channel's own event queue, a child of the main loop's.
    pub(crate) events: crate::event::multiqueue::ChildQueue,
    pub stream: ChannelStream,
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
    /// Which of the five transports this is, as the C enum numbers them.
    pub fn streamtype(&self) -> ChannelStreamType {
        self.stream.kind()
    }
}

/// A channel's transport.
///
/// Inline, as upstream's union was: every channel is boxed by the table, so
/// a socket or stdio channel carrying a job's size costs what it always did.
#[allow(clippy::large_enum_variant)]
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
            ChannelStream::Job(_) => kChannelStreamProc,
            ChannelStream::Socket(_) => kChannelStreamSocket,
            ChannelStream::Stdio(_) => kChannelStreamStdio,
            ChannelStream::Stderr(_) => kChannelStreamStderr,
            ChannelStream::Internal(_) => kChannelStreamInternal,
        }
    }
}

/// A job's child, started through libuv or behind a pty. Both begin with
/// the `Proc` the process layer works on.
#[allow(clippy::large_enum_variant)]
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
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamInternal: ChannelStreamType = 4;

pub struct ChannelCallFrame {
    pub request_id: uint32_t,
    pub returned: bool,
    pub errored: bool,
    pub result: Object,
}
pub type ChannelPart = ::core::ffi::c_uint;
pub type ChannelStdinMode = ::core::ffi::c_uint;
/// What a channel does with the job's stdin.
pub const kChannelStdinPipe: ChannelStdinMode = 0;
pub const kChannelStdinNull: ChannelStdinMode = 1;
pub type ChannelStreamType = ::core::ffi::c_uint;
pub type ClientType = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
}
pub struct RpcState {
    pub closed: bool,
    pub unpacker: Option<Box<Unpacker>>,
    pub next_request_id: uint32_t,
    /// Requests this editor has sent and is still waiting on.
    pub call_stack: CallStack,
    pub info: ApiDict,
    pub client_type: ClientType,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
}
