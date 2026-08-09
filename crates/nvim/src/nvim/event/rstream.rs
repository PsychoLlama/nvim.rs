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

use crate::src::nvim::event::libuv::{
    uv_err_name, uv_fs_read, uv_fs_req_cleanup, uv_idle_start, uv_idle_stop, uv_read_start,
    uv_read_stop, uv_strerror,
};
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::event::stream::{stream_close_handle, stream_init, stream_may_close};
use crate::src::nvim::log::{LOGLVL_DBG, logmsg_c};
use crate::src::nvim::memory::{alloc_block, free_block};
use crate::src::nvim::os::libc::memmove;
use crate::src::nvim::types::{
    Event, Loop, RStream, Stream, size_t, ssize_t, stream_read_cb, uv_buf_t, uv_fs_t, uv_handle_t,
    uv_handle_type, uv_idle_t, uv_stream_t,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

const UV_TTY: uv_handle_type = 14;
const UV_ENOBUFS: c_int = -105;
const UV_EOF: c_int = -4095;

/// The size of the arena block an `RStream` reads into.
const ARENA_BLOCK_SIZE: usize = 4096;

pub unsafe fn rstream_init_fd(uv_loop: *mut Loop, stream: *mut RStream, fd: c_int) {
    stream_init(uv_loop, &raw mut (*stream).s, fd, ptr::null_mut());
    rstream_init(stream);
}

/// Give `stream` its arena block. The stream must already have been through
/// [`stream_init`].
pub unsafe fn rstream_init(stream: *mut RStream) {
    (*stream).read_cb = None;
    (*stream).num_bytes = 0;
    (*stream).buffer = alloc_block() as *mut c_char;
    (*stream).write_pos = (*stream).buffer;
    (*stream).read_pos = (*stream).write_pos;
    (*stream).s.close_cb = Some(rstream_close_cb);
    (*stream).s.close_cb_data = stream as *mut c_void;
}

/// Ask libuv for reads, without touching `want_read`. Used to resume after
/// back-pressure, and by consumers that pause a stream temporarily.
pub unsafe fn rstream_start_inner(stream: *mut RStream) {
    if (*stream).s.uvstream.is_null() {
        uv_idle_start(&raw mut (*stream).s.uv.idle, Some(fread_idle_cb));
    } else {
        uv_read_start((*stream).s.uvstream, Some(alloc_cb), Some(read_cb));
    }
}

/// Deliver reads to `cb`. Does not start libuv reading while the buffer is
/// still full; [`rstream_consume`] resumes them once there is room.
pub unsafe fn rstream_start(stream: *mut RStream, cb: stream_read_cb, data: *mut c_void) {
    (*stream).read_cb = cb;
    (*stream).s.cb_data = data;
    (*stream).want_read = true;
    if !(*stream).paused_full {
        rstream_start_inner(stream);
    }
}

pub unsafe fn rstream_stop_inner(stream: *mut RStream) {
    if (*stream).s.uvstream.is_null() {
        uv_idle_stop(&raw mut (*stream).s.uv.idle);
    } else {
        uv_read_stop((*stream).s.uvstream);
    }
}

pub unsafe fn rstream_stop(stream: *mut RStream) {
    rstream_stop_inner(stream);
    (*stream).want_read = false;
}

/// Bytes of the block still free after `write_pos`.
unsafe fn rstream_space(stream: *mut RStream) -> size_t {
    (*stream).buffer.addr() + ARENA_BLOCK_SIZE - (*stream).write_pos.addr()
}

/// Bytes read but not yet consumed.
pub unsafe fn rstream_available(stream: *mut RStream) -> size_t {
    (*stream).write_pos.addr() - (*stream).read_pos.addr()
}

/// libuv's allocation callback: offer it the free tail of the block.
unsafe extern "C" fn alloc_cb(handle: *mut uv_handle_t, _suggested: size_t, buf: *mut uv_buf_t) {
    let stream = (*handle).data as *mut RStream;
    (*buf).base = (*stream).write_pos;
    (*buf).len = rstream_space(stream);
}

/// libuv's read callback for a pipe, TTY or socket.
unsafe extern "C" fn read_cb(uvstream: *mut uv_stream_t, cnt: ssize_t, _buf: *const uv_buf_t) {
    let stream = (*uvstream).data as *mut RStream;
    if cnt > 0 {
        (*stream).num_bytes += cnt as size_t;
        (*stream).write_pos = (*stream).write_pos.offset(cnt);
        invoke_read_cb(stream, false);
        return;
    }
    if cnt == UV_ENOBUFS as ssize_t || cnt == 0 {
        // Nothing was read and nothing is wrong: the buffer was full, or
        // libuv had a spurious wakeup.
        return;
    }
    if cnt == UV_EOF as ssize_t && (*uvstream).type_0 == UV_TTY {
        // A TTY that reports EOF is not necessarily done — CTRL-D on a
        // terminal reads as EOF but the descriptor stays open — so reading
        // continues.
        invoke_read_cb(stream, true);
        return;
    }
    logmsg_c!(
        LOGLVL_DBG,
        ptr::null(),
        c"read_cb".as_ptr(),
        122,
        true,
        c"closing Stream (%p): %s (%s)".as_ptr(),
        stream as *mut c_void,
        uv_err_name(cnt as c_int),
        uv_strerror(cnt as c_int),
    );
    uv_read_stop(uvstream);
    invoke_read_cb(stream, true);
}

/// The idle callback standing in for readiness on a regular file: one
/// synchronous read per loop iteration.
unsafe extern "C" fn fread_idle_cb(handle: *mut uv_idle_t) {
    let stream = (*handle).data as *mut RStream;
    let mut req: uv_fs_t = core::mem::zeroed();
    (*stream).uvbuf.base = (*stream).write_pos;
    (*stream).uvbuf.len = rstream_space(stream);
    uv_fs_read(
        (*handle).loop_0,
        &raw mut req,
        (*stream).s.fd,
        &raw const (*stream).uvbuf,
        1,
        (*stream).s.fpos,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    if req.result <= 0 {
        uv_idle_stop(&raw mut (*stream).s.uv.idle);
        invoke_read_cb(stream, true);
        return;
    }
    (*stream).write_pos = (*stream).write_pos.offset(req.result as isize);
    (*stream).s.fpos += req.result as i64;
    invoke_read_cb(stream, false);
}

/// Post a read event, or run it inline when the stream has no queue.
///
/// Filling the block stops the reads: [`rstream_consume`] restarts them.
unsafe fn invoke_read_cb(stream: *mut RStream, eof: bool) {
    (*stream).did_eof |= eof;
    if rstream_space(stream) == 0 {
        rstream_stop_inner(stream);
        (*stream).paused_full = true;
    }
    if (*stream).pending_read {
        return;
    }
    (*stream).s.pending_reqs += 1;
    (*stream).pending_read = true;
    if (*stream).s.events.is_null() {
        let mut argv = [stream as *mut c_void];
        read_event(&raw mut argv as *mut *mut c_void);
    } else {
        let mut argv = [ptr::null_mut::<c_void>(); 10];
        argv[0] = stream as *mut c_void;
        multiqueue_put_event(
            (*stream).s.events,
            Event {
                handler: Some(read_event),
                argv,
            },
        );
    }
}

/// Hand the filled span to the consumer and take back what it consumed.
unsafe extern "C" fn read_event(argv: *mut *mut c_void) {
    let stream = (*argv.offset(0)) as *mut RStream;
    (*stream).pending_read = false;
    if let Some(consume) = (*stream).read_cb {
        let available = rstream_available(stream);
        let consumed = consume(
            stream,
            (*stream).read_pos,
            available,
            (*stream).s.cb_data,
            (*stream).did_eof,
        );
        debug_assert!(consumed <= available);
        rstream_consume(stream, consumed);
    }
    (*stream).s.pending_reqs -= 1;
    if (*stream).s.closed && (*stream).s.pending_reqs == 0 {
        stream_close_handle(&raw mut (*stream).s);
    }
}

/// Drop the first `consumed` unread bytes, compacting what is left to the
/// front of the block and resuming reads if that made room.
pub unsafe fn rstream_consume(stream: *mut RStream, consumed: size_t) {
    (*stream).read_pos = (*stream).read_pos.add(consumed);
    let remaining = rstream_available(stream);
    if remaining > 0 && (*stream).read_pos > (*stream).buffer {
        memmove(
            (*stream).buffer as *mut c_void,
            (*stream).read_pos as *const c_void,
            remaining,
        );
        (*stream).read_pos = (*stream).buffer;
        (*stream).write_pos = (*stream).buffer.add(remaining);
    } else if remaining == 0 {
        (*stream).write_pos = (*stream).buffer;
        (*stream).read_pos = (*stream).write_pos;
    }
    if (*stream).want_read && (*stream).paused_full && rstream_space(stream) != 0 {
        debug_assert!((*stream).read_cb.is_some());
        (*stream).paused_full = false;
        rstream_start_inner(stream);
    }
}

/// The stream's close callback: give the arena block back.
unsafe extern "C" fn rstream_close_cb(s: *mut Stream, data: *mut c_void) {
    let stream = data as *mut RStream;
    debug_assert!(!stream.is_null() && s == &raw mut (*stream).s);
    if !(*stream).buffer.is_null() {
        free_block((*stream).buffer as *mut c_void);
    }
}

pub unsafe fn rstream_may_close(stream: *mut RStream) {
    stream_may_close(&raw mut (*stream).s);
}
