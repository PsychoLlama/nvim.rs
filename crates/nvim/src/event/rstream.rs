//! The reading half of a stream.
//!
//! Each `RStream` owns one pool block ([`ReadBuffer`]). libuv reads into the
//! free tail of that block, and the consumer is handed the filled span as a
//! slice ([`Arrived`]) and reports how much of it it took; whatever is left
//! is moved back to the front. When the
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
//! [`ReadStream`] is a stream as its owner holds it (stdin, the TUI, a
//! socket or stdio channel): in place, driven through `&self`, opened by the
//! one unsafe call. [`Reader`] is the `*mut RStream` libuv threads through
//! its handles' `data`, wrapped once. `conn()` answers the [`Conn`] for the
//! common half, which is what keeps `(*stream).s.<field>` from appearing at
//! every site.

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

use crate::event::libuv::{
    uv_fs_read, uv_fs_req_cleanup, uv_idle_start, uv_idle_stop, uv_read_start, uv_read_stop,
};
use crate::event::r#loop::main_loop_raw;
use crate::event::objects::Loop;
use crate::event::objects::{RStream, Stream, stream_read_cb};
use crate::event::stream::{Conn, close_handle, may_close, stream_init};
use crate::event::uv::UV_TTY;
use crate::event::uv::{uv_buf_t, uv_fs_t, uv_handle_t, uv_idle_t, uv_stream_t};
use crate::log::{LOGLVL_DBG, logmsg};
use crate::memory::{BLOCK_SIZE, block_vec, recycle_block};
use crate::message_fmt::{msg_addr, msg_cstr};
use crate::os::fs::{os_err_name, os_strerror};
use crate::os::libc::{size_t, ssize_t};
use crate::os::uv_error::{UV_ENOBUFS, UV_EOF};
use core::cell::UnsafeCell;
use core::ffi::{c_int, c_void};
use core::ops::Deref;
use core::pin::Pin;
use core::ptr;

/// The size of the pool block an `RStream` reads into.
pub(crate) const ARENA_BLOCK_SIZE: usize = BLOCK_SIZE;

/// What a read stream hands its consumer.
#[derive(Debug)]
pub(crate) struct Arrived<'a> {
    /// The unread bytes, from the front of the stream's block.
    pub(crate) bytes: &'a [u8],
    /// Whether the stream has reached the end of its input.
    pub(crate) eof: bool,
    /// The word the consumer was started with.
    pub(crate) data: *mut c_void,
    /// The descriptor the stream reads, or -1 for a handle it was given.
    pub(crate) fd: c_int,
    /// The stream's address, for a log line; never dereferenced.
    pub(crate) stream: *const c_void,
}

/// The block a stream reads into, and how far it is read and written:
/// `block[read..write]` is read but not consumed, `block[write..]` is the
/// free tail libuv fills.
///
/// No block until [`ReadBuffer::acquire`]; all-zero is that empty state (a
/// `None` box and two zero offsets), which is how the C-shaped owners
/// zero-fill a stream before opening it.
#[derive(Debug, Default)]
pub(crate) struct ReadBuffer {
    block: Option<Box<[u8; ARENA_BLOCK_SIZE]>>,
    read: usize,
    write: usize,
}

impl ReadBuffer {
    /// Take a block from the pool, empty.
    fn acquire(&mut self) {
        let mut block = block_vec();
        block.resize(ARENA_BLOCK_SIZE, 0);
        self.block = Some(
            block
                .into_boxed_slice()
                .try_into()
                .expect("a pool block is a block long"),
        );
        self.read = 0;
        self.write = 0;
    }

    /// Give the block back to the pool.
    fn release(&mut self) {
        if let Some(block) = self.block.take() {
            recycle_block((block as Box<[u8]>).into_vec());
        }
        self.read = 0;
        self.write = 0;
    }

    /// Bytes of the block still free after what was read.
    fn space(&self) -> usize {
        if self.block.is_some() {
            ARENA_BLOCK_SIZE - self.write
        } else {
            0
        }
    }

    /// Bytes read but not yet consumed.
    fn available(&self) -> usize {
        self.write - self.read
    }

    /// The bytes read but not yet consumed.
    fn unread(&self) -> &[u8] {
        self.block
            .as_deref()
            .map_or(&[], |block| &block[self.read..self.write])
    }

    /// The free tail, as the descriptor libuv reads into.
    ///
    /// The pointer is taken without forming a reference to the block, so it
    /// leaves alone a consumer's borrow of the unread head, which it does
    /// not overlap: libuv may read while a consumer runs the loop.
    fn tail(&mut self) -> uv_buf_t {
        let write = self.write;
        let space = self.space();
        let base = self.block.as_mut().map_or(ptr::null_mut(), |block| {
            Box::as_mut_ptr(block).cast::<u8>().wrapping_add(write)
        });
        uv_buf_t {
            base: base.cast(),
            len: space,
        }
    }

    /// `count` more bytes of the free tail were filled.
    fn filled(&mut self, count: usize) {
        assert!(count <= self.space(), "a read fits the tail it was given");
        self.write += count;
    }

    /// Drop the first `consumed` unread bytes, moving what is left to the
    /// front of the block.
    fn consume(&mut self, consumed: usize) {
        assert!(
            consumed <= self.available(),
            "a consumer takes no more than it was given"
        );
        self.read += consumed;
        if self.read == self.write {
            self.read = 0;
            self.write = 0;
        } else if self.read > 0
            && let Some(block) = self.block.as_deref_mut()
        {
            block.copy_within(self.read..self.write, 0);
            self.write -= self.read;
            self.read = 0;
        }
    }
}

/// A reading stream, reached through the raw pointer libuv keeps in its
/// handle's `data` field.
///
/// `RStream` embeds a `Stream` as its first field, so a reading stream is
/// also a [`Conn`] — but the two are kept apart on purpose: only the reading
/// half knows about the block, and only the common half knows about closing.
#[derive(Copy, Clone, Debug)]
pub(super) struct Reader(*mut RStream);

impl Reader {
    /// # Safety
    /// `stream` is live, does not move while it is open, and outlives every
    /// use of this handle.
    pub(super) unsafe fn new(stream: *mut RStream) -> Self {
        debug_assert!(!stream.is_null());
        Reader(stream)
    }

    /// The pointer back, for the consumer's callback, which still takes one.
    pub(super) fn as_ptr(self) -> *mut RStream {
        self.0
    }

    /// The common half: the libuv handle, and closing.
    pub(super) fn conn(self) -> Conn {
        // SAFETY: a field of the live stream, so it inherits this handle's
        // promise.
        unsafe { Conn::new(&raw mut (*self.0).s) }
    }

    /// The stream's block, for one step that runs nothing else.
    fn with_buffer<R>(self, f: impl FnOnce(&mut ReadBuffer) -> R) -> R {
        // SAFETY: a field of the live stream (the promise made at
        // construction), borrowed for a leaf step: nothing in `f` reaches
        // the stream again. libuv never writes the field, only the block
        // it points at, through the tail it was handed.
        f(unsafe { &mut (*self.0).buf })
    }

    /// Bytes of the block still free after what was read.
    fn space(self) -> size_t {
        self.buf.space()
    }
}

impl Deref for Reader {
    type Target = RStream;

    fn deref(&self) -> &RStream {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

/// All-zero: every handle unregistered, every pointer null, no block.
pub(super) const fn zeroed_rstream() -> RStream {
    // SAFETY: a stream is plain C data -- pointers, integers, flags,
    // `Option`s of function pointers and libuv handles -- for all of which
    // all-zero is the unset value, and its block is an `Option<Box>`, for
    // which all-zero is `None`.
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
}

/// A read stream as its owner holds it: in place, at the address libuv is
/// given when it is opened.
///
/// Every operation takes `&self`. libuv writes the handle inside behind any
/// reference to it, and the `UnsafeCell` is what lets a shared one coexist
/// with that; no `&mut` to a registered stream is ever formed here. Opening
/// is the one unsafe step, because from then on the owner promises the
/// stream stays put until its close callback has run. The other operations
/// do nothing on a stream that was never opened or has finished closing:
/// its block is what says it is open.
///
/// The owners: stdin's (`os/input.rs`), the TUI's (`tui::tui::Handles`), a
/// socket channel's and a stdio channel's (`ChannelStream`). A child's two
/// output streams are still plain [`RStream`] fields of its `Proc`.
#[repr(transparent)]
pub struct ReadStream(UnsafeCell<RStream>);

impl core::fmt::Debug for ReadStream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ReadStream").field(&self.as_ptr()).finish()
    }
}

impl ReadStream {
    /// A stream before it is opened: every handle unregistered, every
    /// pointer null, no block.
    pub(crate) const fn unopened() -> Self {
        Self(UnsafeCell::new(zeroed_rstream()))
    }

    /// [`ReadStream::unopened`], marked closed: what `os/input.rs` asks
    /// before it opens stdin.
    pub(crate) const fn closed() -> Self {
        let mut stream = zeroed_rstream();
        stream.s.closed = true;
        Self(UnsafeCell::new(stream))
    }

    /// The stream's address: libuv's view of it, and a log line's.
    pub(crate) fn as_ptr(&self) -> *mut RStream {
        self.0.get()
    }

    /// The handle `rstream`'s functions work through, valid for this
    /// borrow.
    fn reader(&self) -> Reader {
        // SAFETY: live for the borrow, and once opened it does not move
        // until its close callback (the promise `open` takes); before that
        // nothing registered holds the address.
        unsafe { Reader::new(self.as_ptr()) }
    }

    /// The handle, when the stream is open: it has its block from
    /// `rstream_init` until the close callback gives it back.
    fn opened(&self) -> Option<Reader> {
        let reader = self.reader();
        reader.buf.block.is_some().then_some(reader)
    }

    /// Open the stream over `fd` on the editor's loop, and give it its
    /// block.
    ///
    /// # Safety
    /// `fd` is open. The stream stays at this address, alive, until its
    /// close callback has run (after [`ReadStream::close`] and the loop
    /// turn that finishes it).
    pub(crate) unsafe fn open_fd(&self, fd: c_int) {
        // SAFETY: the editor's loop, live for the process, and the caller's
        // promise about the stream and the descriptor.
        unsafe { rstream_init_fd(main_loop_raw(), self.as_ptr(), fd) };
    }

    /// Give the stream its block, once its common half is connected (a
    /// socket the stream layer opened by connecting or accepting).
    ///
    /// # Safety
    /// The common half has been through `stream_init`; and as
    /// [`ReadStream::open_fd`].
    pub(crate) unsafe fn open(&self) {
        // SAFETY: the caller's promise.
        unsafe { rstream_init(self.as_ptr()) };
    }

    /// Deliver reads to `consume`, told `data` each time.
    pub(crate) fn start(&self, consume: fn(&Arrived<'_>) -> usize, data: *mut c_void) {
        let stream = self
            .opened()
            .expect("a read stream is opened before it is started");
        start_with(stream, Some(consume), data);
    }

    /// Stop delivering reads.
    pub(crate) fn stop(&self) {
        if let Some(stream) = self.opened() {
            stop_reading(stream);
            stream.set_want_read(false);
        }
    }

    /// Mark the stream closed; its handle closes once nothing is in flight,
    /// and the close callback gives its block back.
    pub(crate) fn close(&self) {
        if let Some(stream) = self.opened() {
            may_close(stream.conn());
        }
    }

    /// Has the stream been marked closed (or never opened, for one built
    /// [`ReadStream::closed`])?
    pub(crate) fn is_closed(&self) -> bool {
        self.reader().s.closed
    }

    /// Has the stream seen the end of its input?
    pub(crate) fn at_eof(&self) -> bool {
        self.reader().did_eof
    }

    /// A copy of the bytes read but not consumed, for a consumer outside a
    /// delivery (a timer giving up on the rest of a sequence) whose owner
    /// also holds the stream.
    pub(crate) fn unread(&self) -> Vec<u8> {
        self.reader().buf.unread().to_vec()
    }

    /// Drop the first `consumed` unread bytes, and resume reads if that made
    /// room.
    pub(crate) fn consume(&self, consumed: usize) {
        if let Some(stream) = self.opened() {
            consume_bytes(stream, consumed);
        }
    }

    /// The common half, pinned with the stream: what a socket writes
    /// through.
    pub(super) fn common(self: Pin<&mut Self>) -> Pin<&mut Stream> {
        // SAFETY: structural pinning: a field of the pinned stream, lent
        // pinned, and the exclusive borrow of the whole is what makes the
        // `&mut` to the part unique.
        unsafe { self.map_unchecked_mut(|stream| &mut stream.0.get_mut().s) }
    }

    /// The common half's handle, for the socket layer, which connects and
    /// accepts through it before the stream is opened.
    pub(super) fn conn(&self) -> Conn {
        self.reader().conn()
    }

    /// Who hears that the common half finished closing: a socket channel's
    /// reference.
    pub(crate) fn set_internal_close_cb(
        &self,
        cb: crate::event::objects::stream_close_cb,
        data: *mut c_void,
    ) {
        let conn = self.reader().conn();
        conn.set_internal_close_cb(cb);
        conn.set_internal_data(data);
    }
}

crate::event::stream::field_writers!(Reader {
    fn set_did_eof => did_eof: bool;
    fn set_want_read => want_read: bool;
    fn set_pending_read => pending_read: bool;
    fn set_paused_full => paused_full: bool;
    fn set_read_cb => read_cb: stream_read_cb;
    fn set_num_bytes => num_bytes: size_t;
});

/// # Safety
/// `uv_loop` is live, `stream` points at storage that does not move
/// afterwards, and `fd` is open.
unsafe fn rstream_init_fd(uv_loop: *mut Loop, stream: *mut RStream, fd: c_int) {
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
pub(super) unsafe fn rstream_init(stream: *mut RStream) {
    // SAFETY: the caller's stream.
    let stream = unsafe { Reader::new(stream) };
    stream.set_read_cb(None);
    stream.set_num_bytes(0);
    stream.with_buffer(ReadBuffer::acquire);
    stream.conn().set_close_cb(Some(rstream_close_cb));
    stream.conn().set_close_cb_data(stream.as_ptr().cast());
}

/// Ask libuv for reads, without touching `want_read`. Used to resume after
/// back-pressure, and by consumers that pause a stream temporarily.
///
/// # Safety
/// `stream` has been through [`rstream_init`].
pub(super) unsafe fn rstream_start_inner(stream: *mut RStream) {
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
pub(super) unsafe fn rstream_start(stream: *mut RStream, cb: stream_read_cb, data: *mut c_void) {
    // SAFETY: the caller's stream.
    start_with(unsafe { Reader::new(stream) }, cb, data);
}

/// [`rstream_start`] with the handle already built.
fn start_with(stream: Reader, cb: stream_read_cb, data: *mut c_void) {
    stream.set_read_cb(cb);
    stream.conn().set_cb_data(data);
    stream.set_want_read(true);
    if !stream.paused_full {
        start_reading(stream);
    }
}

/// # Safety
/// `stream` has been through [`rstream_init`].
pub(super) unsafe fn rstream_stop_inner(stream: *mut RStream) {
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

/// libuv's allocation callback: offer it the free tail of the block.
///
/// # Safety
/// libuv's alloc callback: `handle` is a reading stream's own.
unsafe extern "C" fn alloc_cb(handle: *mut uv_handle_t, _suggested: size_t, buf: *mut uv_buf_t) {
    // SAFETY: `rstream_init_fd` put the stream's address in `data`.
    let stream = unsafe { Reader::new((*handle).data.cast()) };
    let tail = stream.with_buffer(ReadBuffer::tail);
    // SAFETY: libuv's own buffer descriptor, to fill in.
    unsafe { *buf = tail };
}

/// libuv's read callback for a pipe, TTY or socket.
///
/// # Safety
/// libuv's read callback: `uvstream` is a reading stream's own, and `cnt`
/// bytes were written into the span [`alloc_cb`] handed out.
unsafe extern "C" fn read_cb(uvstream: *mut uv_stream_t, cnt: ssize_t, _buf: *const uv_buf_t) {
    // SAFETY: `stream_init` put the stream's address in `data`.
    let stream = unsafe { Reader::new((*uvstream).data.cast()) };
    if cnt > 0 {
        let cnt = cnt.cast_unsigned();
        stream.set_num_bytes(stream.num_bytes + cnt);
        // libuv wrote `cnt` bytes into the free tail `alloc_cb` handed out.
        stream.with_buffer(|buffer| buffer.filled(cnt));
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
            "closing Stream ({}): {} ({})",
            msg_addr(stream.as_ptr().cast::<c_void>()),
            msg_cstr(os_err_name(status)),
            msg_cstr(os_strerror(status))
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
    let stream = unsafe { Reader::new((*handle).data.cast()) };
    let tail = stream.with_buffer(ReadBuffer::tail);
    // SAFETY: the descriptor the stream was built from, and a request and a
    // buffer descriptor that both live as long as the synchronous call.
    let result = unsafe {
        let mut req: uv_fs_t = core::mem::zeroed();
        uv_fs_read(
            (*handle).loop_0,
            &raw mut req,
            stream.s.fd,
            &raw const tail,
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
    let filled = usize::try_from(result).expect("a positive read count fits a usize");
    // The read filled `filled` bytes of the free tail.
    stream.with_buffer(|buffer| buffer.filled(filled));
    stream
        .conn()
        .set_fpos(stream.s.fpos + i64::try_from(result).expect("a read fits in a file offset"));
    invoke_read_cb(stream, false);
}

/// Tell the consumer the stream is at its end with nothing more read: a
/// child that keeps its output open would otherwise deny it the end.
pub(super) fn deliver_eof(stream: Reader) {
    if let Some(consume) = stream.read_cb {
        consume(&Arrived {
            bytes: &[],
            eof: true,
            data: stream.s.cb_data,
            fd: stream.s.fd,
            stream: stream.as_ptr().cast_const().cast(),
        });
    }
}

/// Hand what was read to the consumer, unless a delivery is already under
/// way further up the stack (the consumer can poll the loop).
///
/// Filling the block stops the reads: [`rstream_consume`] restarts them.
fn invoke_read_cb(stream: Reader, eof: bool) {
    stream.set_did_eof(stream.did_eof | eof);
    if stream.space() == 0 {
        stop_reading(stream);
        stream.set_paused_full(true);
    }
    if stream.pending_read {
        return;
    }
    stream.conn().set_pending_reqs(stream.s.pending_reqs + 1);
    stream.set_pending_read(true);
    deliver_reads(stream);
}

/// Hand the filled span to the consumer and take back what it consumed.
fn deliver_reads(stream: Reader) {
    stream.set_pending_read(false);
    if let Some(consume) = stream.read_cb {
        // The unread head stays put while the consumer runs: a read it lets
        // run fills only the tail, and nothing compacts the block before
        // `consume_bytes` below (`pending_read` holds off a nested delivery).
        let arrived = Arrived {
            bytes: stream.buf.unread(),
            eof: stream.did_eof,
            data: stream.s.cb_data,
            fd: stream.s.fd,
            stream: stream.as_ptr().cast_const().cast(),
        };
        let consumed = consume(&arrived);
        consume_bytes(stream, consumed);
    }
    stream.conn().set_pending_reqs(stream.s.pending_reqs - 1);
    if stream.s.closed && stream.s.pending_reqs == 0 {
        close_handle(stream.conn());
    }
}

/// Drop the first `consumed` unread bytes, compacting what is left to the
/// front of the block and resuming reads if that made room.
fn consume_bytes(stream: Reader, consumed: size_t) {
    stream.with_buffer(|buffer| buffer.consume(consumed));
    if stream.want_read && stream.paused_full && stream.space() != 0 {
        debug_assert!(stream.read_cb.is_some());
        stream.set_paused_full(false);
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
    stream.with_buffer(ReadBuffer::release);
}

/// # Safety
/// `stream` has been through [`rstream_init`].
pub(super) unsafe fn rstream_may_close(stream: *mut RStream) {
    // SAFETY: the caller's stream.
    may_close(unsafe { Reader::new(stream) }.conn());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    /// Fill the tail as libuv would, at the address `tail` hands out.
    fn read_in(buffer: &mut ReadBuffer, bytes: &[u8]) {
        let tail = buffer.tail();
        assert!(bytes.len() <= tail.len);
        let write = buffer.write;
        let block = buffer.block.as_deref_mut().expect("an opened buffer");
        assert_eq!(block[write..].as_ptr(), tail.base.cast::<u8>().cast_const());
        block[write..write + bytes.len()].copy_from_slice(bytes);
        buffer.filled(bytes.len());
    }

    /// Without a block there is nothing to read into and nothing unread.
    #[test]
    fn an_unopened_buffer_is_empty() {
        let mut buffer = ReadBuffer::default();
        assert_eq!(buffer.space(), 0);
        assert_eq!(buffer.available(), 0);
        assert!(buffer.unread().is_empty());
        assert!(buffer.tail().base.is_null());
        buffer.consume(0);
        buffer.release();
    }

    /// What is read is unread until consumed; a partial take moves the rest
    /// to the front, a whole one empties the block.
    #[test]
    fn consuming_compacts_the_rest_to_the_front() {
        let _held = editor_state_lock();
        let mut buffer = ReadBuffer::default();
        buffer.acquire();
        assert_eq!(buffer.space(), ARENA_BLOCK_SIZE);

        read_in(&mut buffer, b"hello, ");
        read_in(&mut buffer, b"world");
        assert_eq!(buffer.unread(), b"hello, world");
        assert_eq!(buffer.space(), ARENA_BLOCK_SIZE - 12);

        buffer.consume(7);
        assert_eq!(buffer.unread(), b"world");
        assert_eq!((buffer.read, buffer.write), (0, 5), "moved to the front");
        assert_eq!(buffer.space(), ARENA_BLOCK_SIZE - 5);

        buffer.consume(5);
        assert!(buffer.unread().is_empty());
        assert_eq!(buffer.space(), ARENA_BLOCK_SIZE);
        buffer.release();
        assert_eq!(buffer.space(), 0);
    }

    /// A full block has no tail left, and taking any of it makes room.
    #[test]
    fn a_full_block_has_no_tail() {
        let _held = editor_state_lock();
        let mut buffer = ReadBuffer::default();
        buffer.acquire();
        read_in(&mut buffer, &[b'x'; ARENA_BLOCK_SIZE]);
        assert_eq!(buffer.space(), 0);
        assert_eq!(buffer.tail().len, 0);
        buffer.consume(1);
        assert_eq!(buffer.space(), 1);
        assert_eq!(buffer.available(), ARENA_BLOCK_SIZE - 1);
        buffer.release();
    }

    /// A released block goes back to the pool, and the next stream's block
    /// is that one.
    #[test]
    fn a_released_block_is_the_next_one_acquired() {
        let _held = editor_state_lock();
        crate::memory::free_reuse_blks();
        let mut first = ReadBuffer::default();
        first.acquire();
        let address = first.tail().base;
        first.release();
        let mut second = ReadBuffer::default();
        second.acquire();
        assert_eq!(second.tail().base, address);
        second.release();
    }

    /// Taking more than was read is a broken consumer, not a short read.
    #[test]
    #[should_panic(expected = "a consumer takes no more than it was given")]
    fn consuming_more_than_was_read_panics() {
        let mut buffer = ReadBuffer::default();
        buffer.consume(1);
    }
}
