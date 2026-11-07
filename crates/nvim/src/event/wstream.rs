//! The writing half of a stream.
//!
//! A write takes a reference-counted [`SharedBuffer`]: the caller hands over one
//! reference and the buffer is released when the write completes, so the same
//! payload can be broadcast to several streams without copying. Writes to a
//! `uv_stream_t` are queued with libuv and reported through the stream's
//! write callback; writes to a regular file happen synchronously, because
//! there is nothing to wait on.
//!
//! `curmem`/`maxmem` cap how much unwritten payload a stream may be holding.
//! Past the cap a write is refused with `UV_ENOMEM` rather than queued.

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

use crate::event::libuv::{uv_fs_req_cleanup, uv_fs_write, uv_write};
use crate::event::stream::{Conn, close_handle, stream_init};
use crate::memory::recycle_block;
use crate::os::libc::size_t;
use crate::os::uv_error::{UV_ENOMEM, UV_UNKNOWN};
use crate::types::{Loop, Stream, stream_write_cb, uv_buf_t, uv_fs_t, uv_write_t};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::rc::Rc;

/// How much unwritten payload a stream may hold before writes are refused.
const DEFAULT_MAXMEM: size_t = 1024 * 1024 * 2000;

/// One queued `uv_write`. libuv is handed `&uv_req` and finds its way back
/// through that request's `data` field, which points at the whole `WRequest`.
struct WRequest {
    stream: *mut Stream,
    /// The payload, held until libuv is done reading it.
    buffer: SharedBuffer,
    uv_req: uv_write_t,
}

/// # Safety
/// `uv_loop` is live, `stream` points at storage that does not move
/// afterwards, and `fd` is open.
pub unsafe fn wstream_init_fd(uv_loop: *mut Loop, stream: *mut Stream, fd: c_int, maxmem: size_t) {
    // SAFETY: the caller's promise, handed straight on.
    unsafe {
        stream_init(uv_loop, stream, fd, ptr::null_mut());
        wstream_init(stream, maxmem);
    }
}

/// Cap `stream`'s unwritten payload at `maxmem`, or at [`DEFAULT_MAXMEM`]
/// when it is zero.
///
/// # Safety
/// `stream` has been through [`stream_init`].
pub unsafe fn wstream_init(stream: *mut Stream, maxmem: size_t) {
    // SAFETY: the caller's stream.
    let mut stream = unsafe { Conn::new(stream) };
    stream.maxmem = if maxmem != 0 { maxmem } else { DEFAULT_MAXMEM };
}

/// Report each completed write to `cb`.
///
/// # Safety
/// `stream` has been through [`stream_init`], and `cb` is safe to call with
/// `data`.
pub unsafe fn wstream_set_write_cb(stream: *mut Stream, cb: stream_write_cb, data: *mut c_void) {
    // SAFETY: the caller's stream.
    let mut stream = unsafe { Conn::new(stream) };
    stream.write_cb = cb;
    stream.cb_data = data;
}

/// Queue `buffer` for writing to `stream`, which holds it until the write
/// completes. Returns 0 when the write was queued — or, for a regular file,
/// completed — and a libuv error code otherwise.
///
/// # Safety
/// `stream` has been through [`wstream_init`].
pub(crate) unsafe fn wstream_write(stream: *mut Stream, buffer: SharedBuffer) -> c_int {
    // SAFETY: the caller's stream.
    let mut stream = unsafe { Conn::new(stream) };
    debug_assert!(stream.maxmem != 0, "wstream_init was not called");
    debug_assert!(!stream.closed);

    let Some(uvstream) = stream.uv_stream() else {
        return write_file(stream, &buffer);
    };
    if stream.curmem > stream.maxmem {
        return UV_ENOMEM;
    }
    let uvbuf = buffer.uv_buf();
    stream.curmem += uvbuf.len;
    let request = Box::into_raw(Box::new(WRequest {
        stream: stream.as_ptr(),
        buffer,
        // SAFETY: a libuv request is plain C data whose all-zero value is
        // the unset one `uv_write` fills in.
        uv_req: unsafe { core::mem::zeroed() },
    }));
    // SAFETY: the request just boxed, taken back by `write_cb`, which libuv
    // reaches through the `data` field set here; the payload it holds is
    // what `uvbuf` points into.
    let err = unsafe {
        (*request).uv_req.data = request.cast();
        uv_write(
            &raw mut (*request).uv_req,
            uvstream,
            &raw const uvbuf,
            1,
            Some(write_cb),
        )
    };
    if err == 0 {
        stream.pending_reqs += 1;
        return 0;
    }
    stream.curmem -= uvbuf.len;
    // SAFETY: libuv refused the write, so nothing else holds the request.
    drop(unsafe { Box::from_raw(request) });
    err
}

/// The regular-file path: one synchronous `uv_fs_write` at the stream's
/// current offset. Such a stream never has a write callback, so the outcome
/// is reported by the return value alone.
fn write_file(mut stream: Conn, buffer: &SharedBuffer) -> c_int {
    let uvbuf = buffer.uv_buf();
    let mut req = uv_fs_t::default();
    // SAFETY: the descriptor the stream was built from, the payload `uvbuf`
    // points into, and a request that lives as long as the synchronous
    // call.
    let err = unsafe {
        uv_fs_write(
            stream.uv_loop(),
            &raw mut req,
            stream.fd,
            &raw const uvbuf,
            1,
            stream.fpos,
            None,
        )
    };
    let result = req.result;
    // SAFETY: the request the call just filled.
    unsafe { uv_fs_req_cleanup(&raw mut req) };
    debug_assert!(stream.write_cb.is_none());
    stream.fpos += i64::try_from(result.max(0)).expect("a write fits in a file offset");
    if result > 0 {
        0
    } else if err != 0 {
        err
    } else {
        UV_UNKNOWN
    }
}

/// libuv's write callback: release the payload, report the status, and close
/// the handle if this was the last request holding it open.
///
/// # Safety
/// libuv's write callback: `req` is one [`wstream_write`] queued, and this is
/// its only completion.
unsafe extern "C" fn write_cb(req: *mut uv_write_t, status: c_int) {
    // SAFETY: the request `wstream_write` boxed and left in `data`.
    let request = unsafe { Box::from_raw((*req).data.cast::<WRequest>()) };
    // SAFETY: the stream the request was queued on, live until its last
    // request completes — which is this one only if `pending_reqs` says so.
    let mut stream = unsafe { Conn::new(request.stream) };
    stream.curmem -= request.buffer.bytes().len();
    drop(request);
    if let Some(report) = stream.write_cb {
        let data = stream.cb_data;
        // SAFETY: the callback and its data were installed together by
        // `wstream_set_write_cb`.
        unsafe { report(stream.as_ptr(), data, status) };
    }
    stream.pending_reqs -= 1;
    if stream.closed && stream.pending_reqs == 0 {
        close_handle(stream);
    }
}

/// One reference to a write payload: what a block of packed bytes travels
/// as once it leaves its packer, so one block can go to several writers
/// (an RPC broadcast) and be released when the last is done with it.
///
/// Cloning takes a reference and dropping gives one up; the last frees the
/// bytes, back onto the block pool's reuse list when they are a block's size.
#[derive(Clone)]
pub(crate) struct SharedBuffer(Rc<Payload>);

/// The bytes a [`SharedBuffer`] shares.
struct Payload(Vec<u8>);

impl Drop for Payload {
    fn drop(&mut self) {
        recycle_block(core::mem::take(&mut self.0));
    }
}

impl SharedBuffer {
    /// `bytes` as a buffer, one reference held.
    pub(crate) fn from_vec(bytes: Vec<u8>) -> Self {
        Self(Rc::new(Payload(bytes)))
    }

    /// The bytes.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0.0
    }

    /// The bytes as libuv's buffer descriptor: a pointer into the shared
    /// payload, valid for as long as a reference is held.
    fn uv_buf(&self) -> uv_buf_t {
        uv_buf_t {
            base: self.0.0.as_ptr().cast::<c_char>().cast_mut(),
            len: self.0.0.len(),
        }
    }
}
