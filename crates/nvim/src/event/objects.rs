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
    addrinfo, uv_async_t, uv_buf_t, uv_file, uv_idle_t, uv_loop_t, uv_pipe_t, uv_signal_t,
    uv_stream_t, uv_tcp_t, uv_timer_t,
};
use crate::os::libc::{int64_t, size_t, uint8_t, uint64_t};
use crate::types::{Dict, Refcount};

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
pub type SignalWatcher = signal_watcher;
pub type SocketWatcher = socket_watcher;
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
    pub cwd: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub exepath: *const ::core::ffi::c_char,
    pub env: *mut Dict,
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
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub uvbuf: uv_buf_t,
    pub read_cb: stream_read_cb,
    pub num_bytes: size_t,
}
pub type signal_cb =
    Option<unsafe fn(*mut SignalWatcher, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type signal_close_cb = Option<unsafe fn(*mut SignalWatcher, *mut ::core::ffi::c_void) -> ()>;
#[repr(C)]
#[derive(Debug)]
pub struct signal_watcher {
    pub uv: uv_signal_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: signal_cb,
    pub close_cb: signal_close_cb,
    pub events: Option<crate::event_queue::QueueId>,
}
/// A listening socket's connection callback: the watcher, and libuv's
/// status for the connection.
pub type socket_cb = Option<fn(&mut SocketWatcher, ::core::ffi::c_int)>;
pub type socket_close_cb = Option<unsafe fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> ()>;
#[derive(Debug)]
pub struct socket_watcher {
    pub addr: [::core::ffi::c_char; 256],
    pub uv: socket_watcher_uv,
    pub stream: *mut uv_stream_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: socket_cb,
    pub close_cb: socket_close_cb,
}
#[repr(C)]
pub union socket_watcher_uv {
    pub tcp: socket_watcher_uv_tcp,
    pub pipe: socket_watcher_uv_pipe,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct socket_watcher_uv_pipe {
    pub handle: uv_pipe_t,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct socket_watcher_uv_tcp {
    pub handle: uv_tcp_t,
    pub addrinfo: *mut addrinfo,
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
pub type stream_read_cb = Option<
    unsafe fn(
        *mut RStream,
        *const ::core::ffi::c_char,
        size_t,
        *mut ::core::ffi::c_void,
        bool,
    ) -> size_t,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union stream_uv {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type stream_write_cb =
    Option<unsafe fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
crate::untagged_union_debug!(socket_watcher_uv, stream_uv);
