#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// libuv's callback signatures, and the loop objects `event/` hands the
// library by address.
#![allow(non_camel_case_types)]

//! The event loop's objects: the loop itself and the streams, processes and
//! watchers `event/` registers with libuv by address, plus the queued
//! [`Event`].

use crate::event::uv::{
    uv_async_t, uv_file, uv_idle_t, uv_loop_t, uv_pipe_t, uv_signal_t, uv_stream_t, uv_tcp_t,
    uv_timer_t,
};
use crate::memory::cstr_vec::CStrVec;
use crate::os::libc::{int64_t, size_t, uint8_t, uint64_t};
use crate::shell::Argv;
use crate::types::Refcount;
use core::ffi::CStr;
use std::ffi::CString;

/// One queued event, run once when its queue reaches it.
///
/// Not `Clone`: an event owns what it runs with, so it is run -- and
/// therefore delivered -- exactly once.
pub enum Event {
    /// A C-shaped handler and the argument words it is run with. Building
    /// one is the unsafe step ([`RawEvent::event`]); running one is not.
    ///
    /// [`RawEvent::event`]: crate::event::loop::RawEvent::event
    Raw(crate::event::r#loop::RawEvent),
    /// Work that owns what it needs.
    Job(Box<dyn FnOnce()>),
    /// `handler(a, b)`: work whose state is two ids, which finds its record
    /// again by them.
    Ids(fn(u64, u64), u64, u64),
}
// By hand: a job is a closure, which has no `Debug`.
impl ::core::fmt::Debug for Event {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::Raw(raw) => f.debug_tuple("Raw").field(raw).finish(),
            Self::Job(_) => f
                .debug_tuple("Job")
                .field(&format_args!("<closure>"))
                .finish(),
            Self::Ids(handler, a, b) => f
                .debug_tuple("Ids")
                .field(handler)
                .field(a)
                .field(b)
                .finish(),
        }
    }
}
pub type Loop = loop_0;
pub type Proc = proc;
pub type ProcType = ::core::ffi::c_uint;
pub type RStream = rstream;
pub type Stream = stream;
pub type internal_proc_cb = Option<unsafe fn(*mut Proc) -> ()>;
#[derive(Debug)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    /// The job-control children. Owned: a `Box<Vec<*mut Proc>>` that
    /// `loop_init` leaves here and `loop_close` takes back.
    pub children: *mut Vec<*mut Proc>,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
}
#[repr(C)]
#[derive(Debug)]
pub struct proc {
    pub type_0: ProcType,
    pub loop_0: *mut Loop,
    pub data: *mut ::core::ffi::c_void,
    pub pid: ::core::ffi::c_int,
    pub status: ::core::ffi::c_int,
    pub refcount: Refcount,
    pub exit_signal: uint8_t,
    pub stopped_time: uint64_t,
    /// What the child runs. Boxed so that an all-zero `Proc` (`proc_init`)
    /// is one without.
    pub(crate) command: Option<Box<Command>>,
    pub in_0: Stream,
    pub out: RStream,
    pub err: RStream,
    pub cb: proc_exit_cb,
    pub state_cb: proc_state_cb,
    pub internal_exit_cb: internal_proc_cb,
    pub internal_close_cb: internal_proc_cb,
    pub closed: bool,
    pub detach: bool,
    pub overlapped: bool,
    pub fwd_err: bool,
    pub stdio_noinherit: bool,
    pub events: Option<crate::event_queue::QueueId>,
}
/// What a child runs: its command line, and where and with which
/// environment it starts.
#[derive(Debug)]
pub(crate) struct Command {
    pub argv: CStrVec,
    /// The program, when the caller resolved it; `argv[0]` otherwise.
    pub exepath: Option<CString>,
    /// The directory it starts in; the editor's own otherwise.
    pub cwd: Option<CString>,
    /// Its whole environment; the editor's own otherwise.
    pub env: Option<CStrVec>,
}

impl Command {
    /// A command line run as it is, in the editor's directory and
    /// environment.
    pub(crate) fn new(argv: Argv) -> Self {
        Self {
            argv: CStrVec::new(argv.into_words()),
            exepath: None,
            cwd: None,
            env: None,
        }
    }

    /// The program to start: `exepath`, or else `argv[0]`.
    pub(crate) fn program(&self) -> &CStr {
        self.exepath
            .as_deref()
            .or_else(|| self.argv.first())
            .unwrap_or(c"")
    }

    /// Drop what only the spawn reads: everything but the command line.
    pub(crate) fn spawned(&mut self) {
        self.exepath = None;
        self.cwd = None;
        self.env = None;
    }
}

pub type proc_exit_cb =
    Option<unsafe fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type proc_state_cb = Option<unsafe fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> ()>;
/// A stream's reading half.
///
/// Not `Copy` and not `Unpin` (its [`stream`] is neither): libuv keeps the
/// address of the handle inside, so a registered one must not move or be
/// duplicated.
#[repr(C)]
#[derive(Debug)]
pub struct rstream {
    pub s: Stream,
    pub did_eof: bool,
    pub want_read: bool,
    pub pending_read: bool,
    pub paused_full: bool,
    pub(crate) buf: crate::event::rstream::ReadBuffer,
    pub(crate) read_cb: stream_read_cb,
    pub num_bytes: size_t,
}
/// A stream: a libuv pipe, TTY, TCP or idle handle, and what reads and
/// writes it.
///
/// Not `Copy`, and not `Unpin`: libuv keeps the address of `uv` from the
/// moment the stream is opened until its close callback, so a registered
/// stream must not move or be duplicated. The transports that hold one are
/// reached through `Pin` everywhere outside `event/`.
#[repr(C)]
#[derive(Debug)]
pub struct stream {
    pub closed: bool,
    pub uv: stream_uv,
    pub uvstream: *mut uv_stream_t,
    pub fd: uv_file,
    pub fpos: int64_t,
    pub cb_data: *mut ::core::ffi::c_void,
    pub before_close_cb: stream_close_cb,
    pub close_cb: stream_close_cb,
    pub internal_close_cb: stream_close_cb,
    pub close_cb_data: *mut ::core::ffi::c_void,
    pub internal_data: *mut ::core::ffi::c_void,
    pub pending_reqs: size_t,
    pub write_cb: stream_write_cb,
    pub curmem: size_t,
    pub maxmem: size_t,
    pub _pinned: ::core::marker::PhantomPinned,
}
pub type stream_close_cb = Option<unsafe fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
/// A read stream's consumer: handed what arrived, it answers how many of
/// those bytes it took.
pub(crate) type stream_read_cb = Option<fn(&crate::event::rstream::Arrived<'_>) -> usize>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union stream_uv {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type stream_write_cb =
    Option<unsafe fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
crate::untagged_union_debug!(stream_uv);
