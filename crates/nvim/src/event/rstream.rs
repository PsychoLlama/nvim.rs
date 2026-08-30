//! The reading half of a stream.
//!
//! Each `RStream` owns one arena block. libuv reads into the free tail of
//! that block, and the consumer is handed the filled span and reports how
//! much of it it took; whatever is left is moved back to the front. When the
//! block fills up the reads are stopped, and they resume as soon as consuming
//! makes room — that back-pressure is what keeps a fast writer from
//! outrunning the editor.
//!
//! The consumer is not called from the libuv callback directly. An event is
//! posted to the stream's queue instead, so reads are delivered in the same
//! order as everything else the loop is doing; a stream with no queue is
//! called back inline. Only one such event is outstanding at a time
//! (`pending_read`), and it counts as a pending request so the handle is not
//! closed out from under it.
//!
//! [`Reader`] is the `*mut RStream` libuv threads through its handles'
//! `data`, wrapped once. `conn()` answers the [`Conn`] for the common half,
//! which is what keeps `(*stream).s.<field>` from appearing at every site.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::libuv::{
    uv_err_name, uv_fs_read, uv_fs_req_cleanup, uv_idle_start, uv_idle_stop, uv_read_start,
    uv_read_stop, uv_strerror,
};
use crate::event::r#loop::one_arg_event;
use crate::event::multiqueue::multiqueue_put_event;
use crate::event::stream::{Conn, close_handle, may_close, stream_init};
use crate::log::{LOGLVL_DBG, logmsg};
use crate::memory::{alloc_block, free_block};
use crate::os::uv_error::{UV_ENOBUFS, UV_EOF};
use crate::types::{
    Loop, RStream, Stream, size_t, ssize_t, stream_read_cb, uv_buf_t, uv_fs_t, uv_handle_t,
    uv_handle_type, uv_idle_t, uv_stream_t,
};
use core::ffi::{c_char, c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::ptr;

const UV_TTY: uv_handle_type = 14;

/// The size of the arena block an `RStream` reads into.
const ARENA_BLOCK_SIZE: usize = 4096;

/// A reading stream, reached through the raw pointer libuv keeps in its
/// handle's `data` field.
///
/// `RStream` embeds a `Stream` as its first field, so a reading stream is
/// also a [`Conn`] — but the two are kept apart on purpose: only the reading
/// half knows about the block, and only the common half knows about closing.
#[derive(Copy, Clone)]
pub struct Reader(*mut RStream);

impl Reader {
    /// # Safety
    /// `stream` is live, does not move while it is open, and outlives every
    /// use of this handle.
    pub unsafe fn new(stream: *mut RStream) -> Self {
        debug_assert!(!stream.is_null());
        Reader(stream)
    }

    /// The pointer back, for the consumer's callback, which still takes one.
    pub fn as_ptr(self) -> *mut RStream {
        self.0
    }

    /// The common half: the libuv handle, and closing.
    pub fn conn(self) -> Conn {
        // SAFETY: a field of the live stream, so it inherits this handle's
        // promise.
        unsafe { Conn::new(&raw mut (*self.0).s) }
    }

    /// Bytes of the block still free after `write_pos`.
    fn space(self) -> size_t {
        self.buffer.addr() + ARENA_BLOCK_SIZE - self.write_pos.addr()
    }

    /// Bytes read but not yet consumed.
    fn available(self) -> size_t {
        self.write_pos.addr() - self.read_pos.addr()
    }
}

impl Deref for Reader {
    type Target = RStream;

    fn deref(&self) -> &RStream {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Reader {
    fn deref_mut(&mut self) -> &mut RStream {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// # Safety
/// `uv_loop` is live, `stream` points at storage that does not move
/// afterwards, and `fd` is open.
pub unsafe fn rstream_init_fd(uv_loop: *mut Loop, stream: *mut RStream, fd: c_int) {
    // SAFETY: the caller's promise, handed straight on.
    unsafe {
        stream_init(uv_loop, &raw mut (*stream).s, fd, ptr::null_mut());
        rstream_init(stream);
    }
}

/// Give `stream` its arena block. The stream must already have been through
/// [`stream_init`].
///
/// # Safety
/// `stream` has been through [`stream_init`] and has no block yet.
pub unsafe fn rstream_init(stream: *mut RStream) {
    // SAFETY: the caller's stream.
    let mut stream = unsafe { Reader::new(stream) };
    stream.read_cb = None;
    stream.num_bytes = 0;
    // SAFETY: a block from the arena, given back by `rstream_close_cb`.
    stream.buffer = unsafe { alloc_block() }.cast::<c_char>();
    stream.write_pos = stream.buffer;
    stream.read_pos = stream.write_pos;
    stream.s.close_cb = Some(rstream_close_cb);
    stream.s.close_cb_data = stream.as_ptr().cast();
}

/// Ask libuv for reads, without touching `want_read`. Used to resume after
/// back-pressure, and by consumers that pause a stream temporarily.
///
/// # Safety
/// `stream` has been through [`rstream_init`].
pub unsafe fn rstream_start_inner(stream: *mut RStream) {
    // SAFETY: the caller's stream.
    start_reading(unsafe { Reader::new(stream) });
}

/// [`rstream_start_inner`] with the handle already built.
fn start_reading(stream: Reader) {
    // SAFETY: the stream's own handle, registered with a loop, and the two
    // callbacks below are this module's.
    unsafe {
        match stream.conn().uv_stream() {
            None => uv_idle_start(stream.conn().idle(), Some(fread_idle_cb)),
            Some(uvstream) => uv_read_start(uvstream, Some(alloc_cb), Some(read_cb)),
        };
    }
}

/// Deliver reads to `cb`. Does not start libuv reading while the buffer is
/// still full; [`rstream_consume`] resumes them once there is room.
///
/// # Safety
/// `stream` has been through [`rstream_init`], and `cb` is safe to call with
/// `data`.
pub unsafe fn rstream_start(stream: *mut RStream, cb: stream_read_cb, data: *mut c_void) {
    // SAFETY: the caller's stream.
    let mut stream = unsafe { Reader::new(stream) };
    stream.read_cb = cb;
    stream.s.cb_data = data;
    stream.want_read = true;
    if !stream.paused_full {
        start_reading(stream);
    }
}

/// # Safety
/// `stream` has been through [`rstream_init`].
pub unsafe fn rstream_stop_inner(stream: *mut RStream) {
    // SAFETY: the caller's stream.
    stop_reading(unsafe { Reader::new(stream) });
}

/// [`rstream_stop_inner`] with the handle already built.
fn stop_reading(stream: Reader) {
    // SAFETY: the stream's own handle, registered with a loop.
    unsafe {
        match stream.conn().uv_stream() {
            None => uv_idle_stop(stream.conn().idle()),
            Some(uvstream) => uv_read_stop(uvstream),
        };
    }
}

/// # Safety
/// `stream` has been through [`rstream_init`].
pub unsafe fn rstream_stop(stream: *mut RStream) {
    // SAFETY: the caller's stream.
    let mut stream = unsafe { Reader::new(stream) };
    stop_reading(stream);
    stream.want_read = false;
}

/// Bytes read but not yet consumed.
///
/// # Safety
/// `stream` has been through [`rstream_init`].
pub unsafe fn rstream_available(stream: *mut RStream) -> size_t {
    // SAFETY: the caller's stream.
    unsafe { Reader::new(stream) }.available()
}

/// libuv's allocation callback: offer it the free tail of the block.
///
/// # Safety
/// libuv's alloc callback: `handle` is a reading stream's own.
unsafe extern "C" fn alloc_cb(handle: *mut uv_handle_t, _suggested: size_t, buf: *mut uv_buf_t) {
    // SAFETY: `rstream_init_fd` put the stream's address in `data`.
    let stream = unsafe { Reader::new((*handle).data.cast()) };
    // SAFETY: libuv's own buffer descriptor, to fill in.
    unsafe {
        (*buf).base = stream.write_pos;
        (*buf).len = stream.space();
    }
}

/// libuv's read callback for a pipe, TTY or socket.
///
/// # Safety
/// libuv's read callback: `uvstream` is a reading stream's own, and `cnt`
/// bytes were written into the span [`alloc_cb`] handed out.
unsafe extern "C" fn read_cb(uvstream: *mut uv_stream_t, cnt: ssize_t, _buf: *const uv_buf_t) {
    // SAFETY: `stream_init` put the stream's address in `data`.
    let mut stream = unsafe { Reader::new((*uvstream).data.cast()) };
    if cnt > 0 {
        stream.num_bytes += cnt.cast_unsigned();
        // SAFETY: libuv wrote `cnt` bytes into the free tail, which is where
        // `write_pos` pointed and which is at least that long.
        stream.write_pos = unsafe { stream.write_pos.offset(cnt) };
        invoke_read_cb(stream, false);
        return;
    }
    // Past this point the count is a libuv status rather than a length, and
    // every status libuv reports is an `int`.
    let status = c_int::try_from(cnt).expect("a libuv read status fits in an int");
    if status == UV_ENOBUFS || status == 0 {
        // Nothing was read and nothing is wrong: the buffer was full, or
        // libuv had a spurious wakeup.
        return;
    }
    // SAFETY: the caller's handle, whose type is a plain field read.
    if status == UV_EOF && unsafe { (*uvstream).type_0 } == UV_TTY {
        // A TTY that reports EOF is not necessarily done — CTRL-D on a
        // terminal reads as EOF but the descriptor stays open — so reading
        // continues.
        invoke_read_cb(stream, true);
        return;
    }
    // SAFETY: the log takes its own lock, and libuv's two name lookups take
    // any status.
    unsafe {
        logmsg!(
            LOGLVL_DBG,
            c"read_cb",
            122,
            c"closing Stream (%p): %s (%s)",
            stream.as_ptr().cast::<c_void>(),
            uv_err_name(status),
            uv_strerror(status),
        );
        uv_read_stop(uvstream);
    }
    invoke_read_cb(stream, true);
}

/// The idle callback standing in for readiness on a regular file: one
/// synchronous read per loop iteration.
///
/// # Safety
/// libuv's idle callback: `handle` is a file-backed reading stream's own.
unsafe extern "C" fn fread_idle_cb(handle: *mut uv_idle_t) {
    // SAFETY: `stream_init` put the stream's address in `data`.
    let mut stream = unsafe { Reader::new((*handle).data.cast()) };
    stream.uvbuf.base = stream.write_pos;
    stream.uvbuf.len = stream.space();
    // SAFETY: the descriptor the stream was built from, and a request and a
    // buffer descriptor that both live as long as the synchronous call.
    let result = unsafe {
        let mut req: uv_fs_t = core::mem::zeroed();
        uv_fs_read(
            (*handle).loop_0,
            &raw mut req,
            stream.s.fd,
            &raw const (*stream.as_ptr()).uvbuf,
            1,
            stream.s.fpos,
            None,
        );
        uv_fs_req_cleanup(&raw mut req);
        req.result
    };
    if result <= 0 {
        // SAFETY: the stream's own idle handle.
        unsafe { uv_idle_stop(stream.conn().idle()) };
        invoke_read_cb(stream, true);
        return;
    }
    // SAFETY: the read filled `result` bytes of the free tail.
    stream.write_pos = unsafe { stream.write_pos.offset(result) };
    stream.s.fpos += i64::try_from(result).expect("a read fits in a file offset");
    invoke_read_cb(stream, false);
}

/// Post a read event, or run it inline when the stream has no queue.
///
/// Filling the block stops the reads: [`rstream_consume`] restarts them.
fn invoke_read_cb(mut stream: Reader, eof: bool) {
    stream.did_eof |= eof;
    if stream.space() == 0 {
        stop_reading(stream);
        stream.paused_full = true;
    }
    if stream.pending_read {
        return;
    }
    stream.s.pending_reqs += 1;
    stream.pending_read = true;
    let arg = stream.as_ptr().cast::<c_void>();
    match stream.conn().events() {
        // SAFETY: the stream outlives the event, which carries nothing but
        // the stream itself.
        Some(events) => unsafe {
            multiqueue_put_event(events, one_arg_event(Some(read_event), arg));
        },
        None => {
            let mut argv = [arg];
            // SAFETY: as above; the argv is this frame's.
            unsafe { read_event(argv.as_mut_ptr()) };
        }
    }
}

/// Hand the filled span to the consumer and take back what it consumed.
///
/// # Safety
/// `argv` slot 0 is the stream, as [`invoke_read_cb`] packed it.
unsafe extern "C" fn read_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise.
    let mut stream = unsafe { Reader::new((*argv).cast()) };
    stream.pending_read = false;
    if let Some(consume) = stream.read_cb {
        let available = stream.available();
        let (read_pos, cb_data, did_eof) = (stream.read_pos, stream.s.cb_data, stream.did_eof);
        // SAFETY: the consumer and its data were installed together by
        // `rstream_start`, and the span is the block's unread head.
        let consumed = unsafe { consume(stream.as_ptr(), read_pos, available, cb_data, did_eof) };
        debug_assert!(consumed <= available);
        consume_bytes(stream, consumed);
    }
    stream.s.pending_reqs -= 1;
    if stream.s.closed && stream.s.pending_reqs == 0 {
        close_handle(stream.conn());
    }
}

/// Drop the first `consumed` unread bytes, compacting what is left to the
/// front of the block and resuming reads if that made room.
///
/// # Safety
/// `stream` has been through [`rstream_init`], and `consumed` is no more than
/// [`rstream_available`] answered.
pub unsafe fn rstream_consume(stream: *mut RStream, consumed: size_t) {
    // SAFETY: the caller's stream.
    consume_bytes(unsafe { Reader::new(stream) }, consumed);
}

/// [`rstream_consume`] with the handle already built.
fn consume_bytes(mut stream: Reader, consumed: size_t) {
    // SAFETY: the caller's promise: `consumed` bytes of the unread head.
    stream.read_pos = unsafe { stream.read_pos.add(consumed) };
    let remaining = stream.available();
    if remaining > 0 && stream.read_pos > stream.buffer {
        // SAFETY: both spans are inside the one block, and `memmove` is
        // defined for overlapping ones.
        unsafe {
            stream
                .buffer
                .cast::<u8>()
                .copy_from(stream.read_pos.cast(), remaining);
        }
        stream.read_pos = stream.buffer;
        // SAFETY: `remaining` bytes of the block, just moved to its front.
        stream.write_pos = unsafe { stream.buffer.add(remaining) };
    } else if remaining == 0 {
        stream.write_pos = stream.buffer;
        stream.read_pos = stream.write_pos;
    }
    if stream.want_read && stream.paused_full && stream.space() != 0 {
        debug_assert!(stream.read_cb.is_some());
        stream.paused_full = false;
        start_reading(stream);
    }
}

/// The stream's close callback: give the arena block back.
///
/// # Safety
/// `rstream_init` installed this with the stream as its data, and it runs
/// once.
unsafe fn rstream_close_cb(s: *mut Stream, data: *mut c_void) {
    // SAFETY: the caller's promise.
    let stream = unsafe { Reader::new(data.cast()) };
    debug_assert!(s == stream.conn().as_ptr());
    if !stream.buffer.is_null() {
        // SAFETY: the block `rstream_init` took, given back exactly once.
        unsafe { free_block(stream.buffer.cast::<c_void>()) };
    }
}

/// # Safety
/// `stream` has been through [`rstream_init`].
pub unsafe fn rstream_may_close(stream: *mut RStream) {
    // SAFETY: the caller's stream.
    may_close(unsafe { Reader::new(stream) }.conn());
}
