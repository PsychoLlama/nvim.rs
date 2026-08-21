//! The common half of a stream: which libuv handle backs it, and how it is
//! closed.
//!
//! A `Stream` wraps one of three things — a pipe or TTY (a `uv_stream_t`), a
//! regular file (read and written synchronously, with a `uv_idle_t` driving
//! the reads), or a `uv_stream_t` someone else owns. Reading and writing live
//! in `rstream` and `wstream`; closing is here because both sides have to
//! agree on it: the handle is only closed once no request is outstanding.
//!
//! libuv keeps the `Stream`'s address in its handle's `data` field, which is
//! how the callbacks find their way back. Nothing here may move a `Stream`.
//!
//! [`Conn`] is that pointer wrapped once. It is the batch's handle for a
//! stream: `rstream`, `wstream`, `proc` and `socket` all build one rather
//! than dereferencing field by field, and its `uv` accessors are what keep
//! the `stream_uv` union's three arms from costing a line per access.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::libuv::{
    uv_close, uv_guess_handle, uv_idle_init, uv_is_closing, uv_loop_close, uv_loop_init,
    uv_pipe_init, uv_pipe_open, uv_run, uv_stream_get_write_queue_size, uv_stream_set_blocking,
};
use crate::event::r#loop::EventLoop;
use crate::log::{LOGLVL_DBG, LOGLVL_WRN, logmsg};
use crate::types::{
    Loop, MultiQueue, Stream, uv_handle_t, uv_handle_type, uv_idle_t, uv_loop_t, uv_pipe_t,
    uv_run_mode, uv_stream_t, uv_tcp_t,
};
use core::ffi::{c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::ptr;

const UV_NAMED_PIPE: uv_handle_type = 7;
const UV_TTY: uv_handle_type = 14;
const UV_FILE: uv_handle_type = 17;
const UV_RUN_NOWAIT: uv_run_mode = 2;

/// A stream, reached through the raw pointer libuv keeps in its handle's
/// `data` field.
///
/// A stream never moves once it is initialised, so wrapping that pointer is
/// the only unsafe step. The accessors derive their handle pointers from the
/// wrapped pointer rather than from what `deref_mut` hands back: a pointer
/// taken from a borrow would be invalidated by the next field write.
#[derive(Copy, Clone)]
pub struct Conn(*mut Stream);

impl Conn {
    /// # Safety
    /// `stream` is live, does not move while it is open, and outlives every
    /// use of this handle.
    pub unsafe fn new(stream: *mut Stream) -> Self {
        debug_assert!(!stream.is_null());
        Conn(stream)
    }

    /// The pointer back, for the callbacks and the callers that still take
    /// one.
    pub fn as_ptr(self) -> *mut Stream {
        self.0
    }

    /// The idle handle that stands in for readiness on a regular file. Only
    /// meaningful for a stream with no `uv_stream_t`.
    pub fn idle(self) -> *mut uv_idle_t {
        // SAFETY: a field of the live stream.
        unsafe { &raw mut (*self.0).uv.idle }
    }

    /// The pipe handle a descriptor-backed stream owns.
    pub fn pipe(self) -> *mut uv_pipe_t {
        // SAFETY: a field of the live stream.
        unsafe { &raw mut (*self.0).uv.pipe }
    }

    /// The TCP handle a socket-backed stream owns.
    pub fn tcp(self) -> *mut uv_tcp_t {
        // SAFETY: a field of the live stream.
        unsafe { &raw mut (*self.0).uv.tcp }
    }

    /// The libuv handle the union holds, whichever arm it is. Every arm
    /// starts with a `uv_handle_t`, and they share an address.
    pub fn uv_handle(self) -> *mut uv_handle_t {
        // SAFETY: a field of the live stream.
        unsafe { (&raw mut (*self.0).uv).cast() }
    }

    /// The loop the stream's own handle is registered with. A stream that
    /// borrowed someone else's `uv_stream_t` has none of its own.
    pub fn uv_loop(self) -> *mut uv_loop_t {
        // SAFETY: a field of the live stream; every arm of the union starts
        // with the `uv_handle_t` header this reads.
        unsafe { (*self.idle()).loop_0 }
    }

    /// The queue this stream's events go on, or `None` when it is driven
    /// synchronously.
    pub fn events(self) -> Option<*mut MultiQueue> {
        let events = self.events;
        (!events.is_null()).then_some(events)
    }

    /// The `uv_stream_t` this stream reads and writes, when it has one. A
    /// regular file does not: it is read and written synchronously.
    pub fn uv_stream(self) -> Option<*mut uv_stream_t> {
        let uvstream = self.uvstream;
        (!uvstream.is_null()).then_some(uvstream)
    }
}

impl Deref for Conn {
    type Target = Stream;

    fn deref(&self) -> &Stream {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Conn {
    fn deref_mut(&mut self) -> &mut Stream {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// Put `fd` into (or out of) blocking mode.
///
/// libuv only exposes this through a stream handle, so a throwaway loop and
/// pipe are wrapped around the descriptor for the duration of the call. The
/// handle is opened but never read from, so nothing is lost by closing it
/// immediately; the `uv_run` is what lets the close complete before the loop
/// is torn down.
pub fn stream_set_blocking(fd: c_int, blocking: bool) -> c_int {
    // SAFETY: both handles are zero-initialised exactly as the C was and then
    // filled in by the `uv_*_init` calls before anything reads them; they
    // live on this frame for the whole of the loop's short life.
    unsafe {
        let mut uv_loop: uv_loop_t = core::mem::zeroed();
        let mut stream: uv_pipe_t = core::mem::zeroed();
        uv_loop_init(&raw mut uv_loop);
        uv_pipe_init(&raw mut uv_loop, &raw mut stream, 0);
        uv_pipe_open(&raw mut stream, fd);
        let retval = uv_stream_set_blocking((&raw mut stream).cast(), c_int::from(blocking));
        uv_close((&raw mut stream).cast(), None);
        uv_run(&raw mut uv_loop, UV_RUN_NOWAIT);
        uv_loop_close(&raw mut uv_loop);
        retval
    }
}

/// Attach `stream` to a descriptor or to an existing `uv_stream_t`.
///
/// Exactly one of `fd` and `uvstream` is given: a descriptor needs a loop to
/// register a handle with, an existing handle brings its own.
///
/// # Safety
/// `stream` points at storage for a stream that does not move afterwards;
/// `uv_loop` is live when it is given, and `fd` is open when it is.
pub unsafe fn stream_init(
    uv_loop: *mut Loop,
    stream: *mut Stream,
    fd: c_int,
    uvstream: *mut uv_stream_t,
) {
    assert!(
        if uvstream.is_null() {
            fd >= 0 && !uv_loop.is_null()
        } else {
            fd < 0 && uv_loop.is_null()
        },
        "a stream is built from a descriptor or from a handle, not both"
    );
    // SAFETY: the caller's storage, filled in below before anything reads it.
    let mut stream = unsafe { Conn::new(stream) };
    stream.uvstream = uvstream;
    if fd >= 0 {
        // SAFETY: the caller's loop, given because `fd` was.
        let uv_loop = unsafe { EventLoop::new(uv_loop) }.uv();
        stream.fd = fd;
        // SAFETY: the caller's open descriptor.
        let handle_type = unsafe { uv_guess_handle(fd) };
        if handle_type == UV_FILE {
            // A regular file has no readiness to wait on: an idle handle
            // drives the synchronous reads instead.
            // SAFETY: the stream's own idle handle, on the caller's loop.
            unsafe {
                uv_idle_init(uv_loop, stream.idle());
                (*stream.idle()).data = stream.as_ptr().cast();
            }
        } else {
            debug_assert!(handle_type == UV_NAMED_PIPE || handle_type == UV_TTY);
            // SAFETY: the stream's own pipe handle, on the caller's loop,
            // taking over the caller's open descriptor.
            unsafe {
                uv_pipe_init(uv_loop, stream.pipe(), 0);
                uv_pipe_open(stream.pipe(), fd);
            }
            stream.uvstream = stream.pipe().cast();
        }
    }
    if let Some(uvstream) = stream.uv_stream() {
        // SAFETY: the handle just opened, or the caller's.
        unsafe { (*uvstream).data = stream.as_ptr().cast() };
    }
    stream.fpos = 0;
    stream.internal_data = ptr::null_mut();
    stream.curmem = 0;
    stream.maxmem = 0;
    stream.pending_reqs = 0;
    stream.write_cb = None;
    stream.close_cb = None;
    stream.internal_close_cb = None;
    stream.closed = false;
    stream.events = ptr::null_mut();
}

/// Mark `stream` closed, and close its handle once nothing is outstanding.
///
/// Requests still in flight keep the handle alive; whichever of them finishes
/// last calls [`stream_close_handle`] itself.
///
/// # Safety
/// `stream` has been through [`stream_init`].
pub unsafe fn stream_may_close(stream: *mut Stream) {
    // SAFETY: the caller's stream.
    may_close(unsafe { Conn::new(stream) });
}

/// [`stream_may_close`] with the handle already built.
pub fn may_close(mut stream: Conn) {
    if stream.closed {
        return;
    }
    // SAFETY: the log takes its own lock; the pointer is only formatted.
    unsafe {
        logmsg!(
            LOGLVL_DBG,
            c"stream_may_close",
            101,
            c"closing Stream: %p",
            stream.as_ptr().cast::<c_void>(),
        );
    }
    stream.closed = true;
    if stream.pending_reqs == 0 {
        close_handle(stream);
    }
}

/// [`stream_close_handle`] with the handle already built.
pub fn close_handle(mut stream: Conn) {
    let handle = match stream.uv_stream() {
        None => stream.idle().cast::<uv_handle_t>(),
        Some(uvstream) => {
            // SAFETY: the stream's handle, open until this call closes it.
            let unwritten = unsafe { uv_stream_get_write_queue_size(uvstream) };
            if unwritten > 0 {
                // SAFETY: the log takes its own lock.
                unsafe {
                    logmsg!(
                        LOGLVL_WRN,
                        c"stream_close_handle",
                        124,
                        c"closed Stream (%p) with %zu unwritten bytes",
                        stream.as_ptr().cast::<c_void>(),
                        unwritten,
                    );
                }
            }
            uvstream.cast()
        }
    };
    // The before-close hook may itself start work, so it is counted as a
    // pending request for its own duration.
    if let Some(before_close) = stream.before_close_cb {
        let data = stream.close_cb_data;
        stream.pending_reqs += 1;
        // SAFETY: the hook and its data were installed together by the
        // stream's owner.
        unsafe { before_close(stream.as_ptr(), data) };
        stream.pending_reqs -= 1;
    }
    // SAFETY: the handle chosen above, still registered with a loop.
    unsafe {
        if uv_is_closing(handle) == 0 {
            uv_close(handle, Some(close_cb));
        }
    }
}

/// libuv's close callback: hand the stream to whoever asked to hear about it.
///
/// # Safety
/// libuv's close callback: `handle` is one [`close_handle`] closed, and this
/// is its last callback.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: `stream_init` put the stream's address in `data`; a handle
    // nobody claimed leaves it null.
    let stream: *mut Stream = unsafe { (*handle).data.cast() };
    if stream.is_null() {
        return;
    }
    // SAFETY: the stream that registered this close.
    let stream = unsafe { Conn::new(stream) };
    if let Some(notify) = stream.close_cb {
        let data = stream.close_cb_data;
        // SAFETY: the callback and its data were installed together.
        unsafe { notify(stream.as_ptr(), data) };
    }
    // Read after the owner's callback: it may install one of its own.
    if let Some(notify_internal) = stream.internal_close_cb {
        let data = stream.internal_data;
        // SAFETY: as above.
        unsafe { notify_internal(stream.as_ptr(), data) };
    }
}
