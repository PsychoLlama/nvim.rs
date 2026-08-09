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

use crate::src::nvim::event::libuv::{
    uv_close, uv_guess_handle, uv_idle_init, uv_is_closing, uv_loop_close, uv_loop_init,
    uv_pipe_init, uv_pipe_open, uv_run, uv_stream_get_write_queue_size, uv_stream_set_blocking,
};
use crate::src::nvim::log::{LOGLVL_DBG, LOGLVL_WRN, logmsg_c};
use crate::src::nvim::types::{
    Loop, Stream, uv_file, uv_handle_t, uv_handle_type, uv_loop_t, uv_pipe_t, uv_run_mode,
    uv_stream_t,
};
use core::ffi::{c_int, c_void};
use core::ptr;

const UV_NAMED_PIPE: uv_handle_type = 7;
const UV_TTY: uv_handle_type = 14;
const UV_FILE: uv_handle_type = 17;
const UV_RUN_NOWAIT: uv_run_mode = 2;

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
        uv_pipe_open(&raw mut stream, fd as uv_file);
        let retval =
            uv_stream_set_blocking(&raw mut stream as *mut uv_stream_t, c_int::from(blocking));
        uv_close(&raw mut stream as *mut uv_handle_t, None);
        uv_run(&raw mut uv_loop, UV_RUN_NOWAIT);
        uv_loop_close(&raw mut uv_loop);
        retval
    }
}

/// Attach `stream` to a descriptor or to an existing `uv_stream_t`.
///
/// Exactly one of `fd` and `uvstream` is given: a descriptor needs a loop to
/// register a handle with, an existing handle brings its own.
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
    (*stream).uvstream = uvstream;
    if fd >= 0 {
        (*stream).fd = fd as uv_file;
        let handle_type = uv_guess_handle(fd as uv_file);
        if handle_type == UV_FILE {
            // A regular file has no readiness to wait on: an idle handle
            // drives the synchronous reads instead.
            uv_idle_init(&raw mut (*uv_loop).uv, &raw mut (*stream).uv.idle);
            (*stream).uv.idle.data = stream as *mut c_void;
        } else {
            debug_assert!(handle_type == UV_NAMED_PIPE || handle_type == UV_TTY);
            uv_pipe_init(&raw mut (*uv_loop).uv, &raw mut (*stream).uv.pipe, 0);
            uv_pipe_open(&raw mut (*stream).uv.pipe, fd as uv_file);
            (*stream).uvstream = &raw mut (*stream).uv.pipe as *mut uv_stream_t;
        }
    }
    if !(*stream).uvstream.is_null() {
        (*(*stream).uvstream).data = stream as *mut c_void;
    }
    (*stream).fpos = 0;
    (*stream).internal_data = ptr::null_mut();
    (*stream).curmem = 0;
    (*stream).maxmem = 0;
    (*stream).pending_reqs = 0;
    (*stream).write_cb = None;
    (*stream).close_cb = None;
    (*stream).internal_close_cb = None;
    (*stream).closed = false;
    (*stream).events = ptr::null_mut();
}

/// Mark `stream` closed, and close its handle once nothing is outstanding.
///
/// Requests still in flight keep the handle alive; whichever of them finishes
/// last calls [`stream_close_handle`] itself.
pub unsafe fn stream_may_close(stream: *mut Stream) {
    if (*stream).closed {
        return;
    }
    logmsg_c!(
        LOGLVL_DBG,
        ptr::null(),
        c"stream_may_close".as_ptr(),
        101,
        true,
        c"closing Stream: %p".as_ptr(),
        stream as *mut c_void,
    );
    (*stream).closed = true;
    if (*stream).pending_reqs == 0 {
        stream_close_handle(stream);
    }
}

pub unsafe fn stream_close_handle(stream: *mut Stream) {
    let handle = if (*stream).uvstream.is_null() {
        &raw mut (*stream).uv.idle as *mut uv_handle_t
    } else {
        let unwritten = uv_stream_get_write_queue_size((*stream).uvstream);
        if unwritten > 0 {
            logmsg_c!(
                LOGLVL_WRN,
                ptr::null(),
                c"stream_close_handle".as_ptr(),
                124,
                true,
                c"closed Stream (%p) with %zu unwritten bytes".as_ptr(),
                stream as *mut c_void,
                unwritten,
            );
        }
        (*stream).uvstream as *mut uv_handle_t
    };
    // The before-close hook may itself start work, so it is counted as a
    // pending request for its own duration.
    if let Some(before_close) = (*stream).before_close_cb {
        (*stream).pending_reqs += 1;
        before_close(stream, (*stream).close_cb_data);
        (*stream).pending_reqs -= 1;
    }
    if uv_is_closing(handle) == 0 {
        uv_close(handle, Some(close_cb));
    }
}

/// libuv's close callback: hand the stream to whoever asked to hear about it.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    let stream = (*handle).data as *mut Stream;
    if stream.is_null() {
        return;
    }
    if let Some(notify) = (*stream).close_cb {
        notify(stream, (*stream).close_cb_data);
    }
    if let Some(notify_internal) = (*stream).internal_close_cb {
        notify_internal(stream, (*stream).internal_data);
    }
}
