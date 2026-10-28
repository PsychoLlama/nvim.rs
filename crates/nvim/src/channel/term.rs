//! The bridge between a pty job and the terminal emulator drawing it.
//!
//! `terminal/` owns the emulator and calls back here for everything that
//! touches the child: bytes typed into the window, window resizes, and the
//! teardown that has to wait for the job's writes to drain. Each callback is
//! told the channel's id.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::multiqueue::put_id_event;
use crate::log::{LOGLVL_INF, logmsg};
use crate::terminal::{terminal_alloc, terminal_destroy};
use crate::types::{ChannelStream, JobProc, TerminalOptions, uint16_t, uint64_t};
use crate::winlayer::Buf;

use super::{ChannelRef, channel_decref, with_channel};

/// Run `f` on job channel `id`'s child, if it is still a registered job.
fn with_job<R>(id: uint64_t, f: impl FnOnce(&mut JobProc) -> R) -> Option<R> {
    with_channel(id, |channel| match &mut channel.stream {
        ChannelStream::Job(job) => Some(f(job)),
        _ => None,
    })
    .flatten()
}

/// Gives `buffer` a terminal driven by pty job channel `id`.
pub fn channel_terminal_alloc(mut buffer: Buf, id: uint64_t) {
    let Some((width, height)) = with_job(id, |job| {
        job.pty().map_or((0, 0), |pty| (pty.width, pty.height))
    }) else {
        return;
    };
    let topts = TerminalOptions {
        owner: id,
        width,
        height,
        read_pause_cb: Some(term_read_pause),
        write_cb: Some(term_write),
        resize_cb: Some(term_resize),
        resume_cb: Some(term_resume),
        close_cb: Some(term_close),
        force_crlf: false,
    };
    buffer.b_p_channel = id.cast_signed();
    // The terminal's own reference, dropped by `term_delayed_free`.
    let Some(held) = ChannelRef::new(id) else {
        return;
    };
    let _ = held.into_raw();
    let term = terminal_alloc(buffer, topts);
    with_channel(id, |channel| channel.term = Some(term));
}

/// Back-pressure from the terminal: stop reading while it catches up.
fn term_read_pause(pause: bool, id: uint64_t) {
    with_job(id, |job| job.pause_output(pause));
}

/// The user typed into the terminal; forward it to the child.
fn term_write(bytes: &[u8], id: uint64_t) {
    let written = with_job(id, |job| {
        let stdin = &mut job.proc_mut().in_0;
        if stdin.closed {
            return false;
        }
        let _ = stdin.write_bytes(bytes);
        true
    });
    if written == Some(false) {
        logmsg!(
            LOGLVL_INF,
            c"term_write",
            918,
            "write failed: stream is closed"
        );
    }
}

fn term_resize(width: uint16_t, height: uint16_t, id: uint64_t) {
    with_job(id, |job| job.resize(width, height));
}

fn term_resume(id: uint64_t) {
    with_job(id, JobProc::resume);
}

/// The terminal window went away: stop the child and wait for its streams.
fn term_close(id: uint64_t) {
    with_channel(id, |channel| {
        if let ChannelStream::Job(job) = &mut channel.stream {
            job.stop();
        }
        put_id_event(&channel.events, term_delayed_free, id, 0);
    });
}

/// Frees the terminal once nothing is still writing through it.
///
/// Re-queues itself while either stream has a request outstanding, because
/// those requests hold buffers the terminal owns.
fn term_delayed_free(id: uint64_t, _: u64) {
    let Some(term) = with_channel(id, |channel| {
        if let ChannelStream::Job(job) = &channel.stream
            && job.has_pending_requests()
        {
            put_id_event(&channel.events, term_delayed_free, id, 0);
            return Err(());
        }
        Ok(channel.term)
    }) else {
        return;
    };
    let Ok(term) = term else {
        return;
    };
    if let Some(term) = term
        && terminal_destroy(term)
    {
        with_channel(id, |channel| channel.term = None);
    }
    channel_decref(id);
}
