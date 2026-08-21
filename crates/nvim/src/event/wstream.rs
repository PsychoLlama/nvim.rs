//! The writing half of a stream.
//!
//! A write takes a reference-counted [`WBuffer`]: the caller hands over one
//! reference and the buffer is released when the write completes, so the same
//! payload can be broadcast to several streams without copying. Writes to a
//! `uv_stream_t` are queued with libuv and reported through the stream's
//! write callback; writes to a regular file happen synchronously, because
//! there is nothing to wait on.
//!
//! `curmem`/`maxmem` cap how much unwritten payload a stream may be holding.
//! Past the cap a write is refused with `UV_ENOMEM` rather than queued.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::libuv::{uv_fs_req_cleanup, uv_fs_write, uv_write};
use crate::event::stream::{Conn, close_handle, stream_init};
use crate::os::uv_error::{UV_ENOMEM, UV_UNKNOWN};
use crate::types::{
    Loop, Stream, WBuffer, size_t, stream_write_cb, uv_buf_t, uv_fs_t, uv_write_t,
    wbuffer_data_finalizer,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// How much unwritten payload a stream may hold before writes are refused.
const DEFAULT_MAXMEM: size_t = 1024 * 1024 * 2000;

/// One queued `uv_write`. libuv is handed `&uv_req` and finds its way back
/// through that request's `data` field, which points at the whole `WRequest`.
#[derive(Copy, Clone)]
pub struct WRequest {
    pub stream: *mut Stream,
    pub buffer: *mut WBuffer,
    pub uv_req: uv_write_t,
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

/// Queue `buffer` for writing to `stream`, taking over the caller's
/// reference. Returns 0 when the write was queued — or, for a regular file,
/// completed — and a libuv error code otherwise.
///
/// # Safety
/// `stream` has been through [`wstream_init`], and `buffer` is a reference
/// the caller owns.
pub unsafe fn wstream_write(stream: *mut Stream, buffer: *mut WBuffer) -> c_int {
    // SAFETY: the caller's stream and buffer.
    let mut stream = unsafe { Conn::new(stream) };
    debug_assert!(stream.maxmem != 0, "wstream_init was not called");
    debug_assert!(!stream.closed);

    // SAFETY: the caller's buffer, still holding its reference.
    let uvbuf = unsafe {
        uv_buf_t {
            base: (*buffer).data,
            len: (*buffer).size,
        }
    };

    let Some(uvstream) = stream.uv_stream() else {
        return write_file(stream, buffer, &raw const uvbuf);
    };

    let err = if stream.curmem > stream.maxmem {
        UV_ENOMEM
    } else {
        stream.curmem += uvbuf.len;
        // SAFETY: the request is boxed here and taken back by `write_cb`,
        // which libuv reaches through the `data` field set just below.
        let (request, err) = unsafe {
            let request = Box::into_raw(Box::new(WRequest {
                stream: stream.as_ptr(),
                buffer,
                uv_req: core::mem::zeroed(),
            }));
            (*request).uv_req.data = request.cast();
            let err = uv_write(
                &raw mut (*request).uv_req,
                uvstream,
                &raw const uvbuf,
                1,
                Some(write_cb),
            );
            (request, err)
        };
        if err == 0 {
            stream.pending_reqs += 1;
            return 0;
        }
        // SAFETY: libuv refused the write, so nothing else holds the request.
        drop(unsafe { Box::from_raw(request) });
        err
    };

    // SAFETY: the caller's reference, given up here because the write did
    // not take it.
    unsafe { wstream_release_wbuffer(buffer) };
    debug_assert!(err != 0);
    err
}

/// The regular-file path: one synchronous `uv_fs_write` at the stream's
/// current offset. Such a stream never has a write callback, so the outcome
/// is reported by the return value alone.
fn write_file(mut stream: Conn, buffer: *mut WBuffer, uvbuf: *const uv_buf_t) -> c_int {
    // SAFETY: the descriptor the stream was built from, and a request that
    // lives as long as the synchronous call.
    let (err, result) = unsafe {
        let mut req: uv_fs_t = core::mem::zeroed();
        let err = uv_fs_write(
            stream.uv_loop(),
            &raw mut req,
            stream.fd,
            uvbuf,
            1,
            stream.fpos,
            None,
        );
        uv_fs_req_cleanup(&raw mut req);
        (err, req.result)
    };
    // SAFETY: the caller's reference, given up now that the write is done.
    unsafe { wstream_release_wbuffer(buffer) };
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

/// A buffer of `size` bytes at `data`, held by `refcount` writers. `cb`, when
/// given, releases `data` once the last of them is done with it.
pub fn wstream_new_buffer(
    data: *mut c_char,
    size: size_t,
    refcount: size_t,
    cb: wbuffer_data_finalizer,
) -> *mut WBuffer {
    Box::into_raw(Box::new(WBuffer {
        size,
        refcount,
        data,
        cb,
    }))
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
    // SAFETY: the reference the write took over.
    unsafe {
        stream.curmem -= (*request.buffer).size;
        wstream_release_wbuffer(request.buffer);
    }
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

/// Drop one reference to `buffer`, releasing it when the last one goes.
///
/// # Safety
/// `buffer` is a reference the caller owns, and gives up here.
pub unsafe fn wstream_release_wbuffer(buffer: *mut WBuffer) {
    // SAFETY: the caller's reference.
    let last = unsafe {
        (*buffer).refcount -= 1;
        (*buffer).refcount == 0
    };
    if last {
        // SAFETY: nothing else holds a reference now, and
        // `wstream_new_buffer` boxed it.
        let buffer = unsafe { Box::from_raw(buffer) };
        if let Some(cb) = buffer.cb {
            // SAFETY: the finalizer the buffer was made with, given the
            // payload it was made for.
            unsafe { cb(buffer.data.cast::<c_void>()) };
        }
    }
}
