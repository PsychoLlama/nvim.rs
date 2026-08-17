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

use crate::event::libuv::{uv_fs_req_cleanup, uv_fs_write, uv_write};
use crate::event::stream::{stream_close_handle, stream_init};
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

pub unsafe fn wstream_init_fd(uv_loop: *mut Loop, stream: *mut Stream, fd: c_int, maxmem: size_t) {
    stream_init(uv_loop, stream, fd, ptr::null_mut());
    wstream_init(stream, maxmem);
}

/// Cap `stream`'s unwritten payload at `maxmem`, or at [`DEFAULT_MAXMEM`]
/// when it is zero.
pub unsafe fn wstream_init(stream: *mut Stream, maxmem: size_t) {
    (*stream).maxmem = if maxmem != 0 { maxmem } else { DEFAULT_MAXMEM };
}

pub unsafe fn wstream_set_write_cb(stream: *mut Stream, cb: stream_write_cb, data: *mut c_void) {
    (*stream).write_cb = cb;
    (*stream).cb_data = data;
}

/// Queue `buffer` for writing to `stream`, taking over the caller's
/// reference. Returns 0 when the write was queued — or, for a regular file,
/// completed — and a libuv error code otherwise.
pub unsafe fn wstream_write(stream: *mut Stream, buffer: *mut WBuffer) -> c_int {
    debug_assert!((*stream).maxmem != 0, "wstream_init was not called");
    debug_assert!(!(*stream).closed);

    let uvbuf = uv_buf_t {
        base: (*buffer).data,
        len: (*buffer).size,
    };

    if (*stream).uvstream.is_null() {
        return write_file(stream, buffer, &raw const uvbuf);
    }

    let err = if (*stream).curmem > (*stream).maxmem {
        UV_ENOMEM
    } else {
        (*stream).curmem += (*buffer).size;
        let request = Box::into_raw(Box::new(WRequest {
            stream,
            buffer,
            uv_req: core::mem::zeroed(),
        }));
        (*request).uv_req.data = request as *mut c_void;
        let err = uv_write(
            &raw mut (*request).uv_req,
            (*stream).uvstream,
            &raw const uvbuf,
            1,
            Some(write_cb),
        );
        if err == 0 {
            (*stream).pending_reqs += 1;
            return 0;
        }
        drop(Box::from_raw(request));
        err
    };

    wstream_release_wbuffer(buffer);
    debug_assert!(err != 0);
    err
}

/// The regular-file path: one synchronous `uv_fs_write` at the stream's
/// current offset. Such a stream never has a write callback, so the outcome
/// is reported by the return value alone.
unsafe fn write_file(stream: *mut Stream, buffer: *mut WBuffer, uvbuf: *const uv_buf_t) -> c_int {
    let mut req: uv_fs_t = core::mem::zeroed();
    let err = uv_fs_write(
        (*stream).uv.idle.loop_0,
        &raw mut req,
        (*stream).fd,
        uvbuf,
        1,
        (*stream).fpos,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    wstream_release_wbuffer(buffer);
    debug_assert!((*stream).write_cb.is_none());
    (*stream).fpos += req.result.max(0) as i64;
    if req.result > 0 {
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
unsafe extern "C" fn write_cb(req: *mut uv_write_t, status: c_int) {
    let request = Box::from_raw((*req).data as *mut WRequest);
    let stream = request.stream;
    (*stream).curmem -= (*request.buffer).size;
    wstream_release_wbuffer(request.buffer);
    if let Some(report) = (*stream).write_cb {
        report(stream, (*stream).cb_data, status);
    }
    (*stream).pending_reqs -= 1;
    if (*stream).closed && (*stream).pending_reqs == 0 {
        stream_close_handle(stream);
    }
}

/// Drop one reference to `buffer`, releasing it when the last one goes.
pub unsafe fn wstream_release_wbuffer(buffer: *mut WBuffer) {
    (*buffer).refcount -= 1;
    if (*buffer).refcount == 0 {
        let buffer = Box::from_raw(buffer);
        if let Some(cb) = buffer.cb {
            cb(buffer.data as *mut c_void);
        }
    }
}
