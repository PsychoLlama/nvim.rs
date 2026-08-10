//! The bridge between a pty job and the terminal emulator drawing it.
//!
//! `terminal.rs` owns the emulator and calls back here for everything that
//! touches the child: bytes typed into the window, window resizes, and the
//! teardown that has to wait for the job's writes to drain.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_void};
use core::ptr;

use crate::src::nvim::event::r#loop::one_arg_event;
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::event::proc::proc_stop;
use crate::src::nvim::event::rstream::{rstream_start_inner, rstream_stop_inner};
use crate::src::nvim::event::wstream::{wstream_new_buffer, wstream_write};
use crate::src::nvim::log::{LOGLVL_INF, logmsg_c};
use crate::src::nvim::memory::{xfree, xmemdup};
use crate::src::nvim::os::pty_proc_unix::{pty_proc_resize, pty_proc_resume};
use crate::src::nvim::terminal::{terminal_alloc, terminal_destroy};
use crate::src::nvim::types::{Channel, OptInt, TerminalOptions, buf_T, size_t, uint16_t};

use super::{channel_decref, channel_incref, channel_proc, channel_pty};

/// Gives `buf` a terminal driven by this channel's pty.
///
/// # Safety
/// `buf` is a live buffer and `chan` a live pty job channel.
pub unsafe fn channel_terminal_alloc(buf: *mut buf_T, chan: *mut Channel) {
    // SAFETY: the caller's live buffer and pty job.
    unsafe {
        let pty = channel_pty(chan);
        let topts = TerminalOptions {
            data: chan.cast(),
            width: (*pty).width,
            height: (*pty).height,
            read_pause_cb: Some(term_read_pause),
            write_cb: Some(term_write),
            resize_cb: Some(term_resize),
            resume_cb: Some(term_resume),
            close_cb: Some(term_close),
            force_crlf: false,
        };
        (*buf).b_p_channel = (*chan).id as OptInt;
        channel_incref(chan);
        (*chan).term = terminal_alloc(buf, topts);
    }
}

/// Back-pressure from the terminal: stop reading while it catches up.
unsafe extern "C" fn term_read_pause(pause: bool, data: *mut c_void) {
    // SAFETY: `data` is the pty job channel the terminal was built on.
    unsafe {
        let out = &raw mut (*data.cast::<Channel>()).stream.proc.out;
        if (*out).s.closed {
            return;
        }
        if pause {
            rstream_stop_inner(out);
        } else {
            rstream_start_inner(out);
        }
    }
}

/// The user typed into the terminal; forward it to the child.
unsafe extern "C" fn term_write(buf: *const c_char, size: size_t, data: *mut c_void) {
    // SAFETY: `data` is the pty job channel the terminal was built on, and
    // `buf` is `size` readable bytes for the duration of the call.
    unsafe {
        let in_0 = &raw mut (*data.cast::<Channel>()).stream.proc.in_0;
        if (*in_0).closed {
            logmsg_c!(
                LOGLVL_INF,
                ptr::null(),
                c"term_write".as_ptr(),
                918,
                true,
                c"write failed: stream is closed".as_ptr(),
            );
            return;
        }
        let wbuf = wstream_new_buffer(xmemdup(buf.cast(), size).cast(), size, 1, Some(xfree));
        wstream_write(in_0, wbuf);
    }
}

unsafe extern "C" fn term_resize(width: uint16_t, height: uint16_t, data: *mut c_void) {
    // SAFETY: `data` is the pty job channel the terminal was built on.
    unsafe { pty_proc_resize(channel_pty(data.cast()), width, height) };
}

unsafe extern "C" fn term_resume(data: *mut c_void) {
    // SAFETY: `data` is the pty job channel the terminal was built on.
    unsafe { pty_proc_resume(channel_pty(data.cast())) };
}

/// The terminal window went away: stop the child and wait for its streams.
unsafe extern "C" fn term_close(data: *mut c_void) {
    // SAFETY: `data` is the pty job channel the terminal was built on; its
    // queue outlives the event this puts on it.
    unsafe {
        let chan = data.cast::<Channel>();
        proc_stop(channel_proc(chan));
        multiqueue_put_event((*chan).events, one_arg_event(Some(term_delayed_free), data));
    }
}

/// Frees the terminal once nothing is still writing through it.
///
/// Re-queues itself while either stream has a request outstanding, because
/// those requests hold buffers the terminal owns.
unsafe extern "C" fn term_delayed_free(argv: *mut *mut c_void) {
    // SAFETY: the event carries the channel `term_close` queued it for, which
    // holds a reference for exactly this.
    unsafe {
        let chan = (*argv).cast::<Channel>();
        let proc = &raw mut (*chan).stream.proc;
        if (*proc).in_0.pending_reqs != 0 || (*proc).out.s.pending_reqs != 0 {
            multiqueue_put_event(
                (*chan).events,
                one_arg_event(Some(term_delayed_free), chan.cast()),
            );
            return;
        }
        if !(*chan).term.is_null() {
            terminal_destroy(&raw mut (*chan).term);
        }
        channel_decref(chan);
    }
}
