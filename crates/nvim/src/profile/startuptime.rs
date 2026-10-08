//! The `--startuptime` log: one line per startup event, with the elapsed
//! and (for sourced scripts) self+sourced columns.
//!
//! This half stays on C stdio. The file is potentially appended to by
//! several nvim processes at once, so the whole report accumulates in a
//! full ("controlled") `setvbuf` buffer and reaches the disk exactly once,
//! at [`time_finish`] — which is what keeps two concurrent processes'
//! reports from interleaving line by line.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{profile_start, profile_sub};
use crate::global_cell::GlobalCell;
use crate::message_fmt::{msg_cstr, to_bytes};
use crate::os::fs::{CFile, os_err_name};
use crate::profile::{startup_timing, time_fd};
use crate::tr;
use crate::types::ProfTime;
use core::ffi::CStr;
use std::io::Write;

// ---------------------------------------------------------------------------
// --startuptime.

/// When `time_start()` was called.
static G_START_TIME: GlobalCell<ProfTime> = GlobalCell::new(0);
/// Time of the previous event line, for the "elapsed" column.
static G_PREV_TIME: GlobalCell<ProfTime> = GlobalCell::new(0);

/// Save the previous time before doing something that could nest (sourcing
/// a script from a script). Returns `(rel, start)`: the time elapsed so far
/// (to hand to [`time_pop`]) and the current time.
pub fn time_push() -> (ProfTime, ProfTime) {
    let now = profile_start();
    let rel = profile_sub(now, G_PREV_TIME.get());
    G_PREV_TIME.set(now);
    (rel, now)
}

/// Subtract the nested duration `time` (from [`time_push`]) from the
/// previous-event time.
pub(crate) fn time_pop(time: ProfTime) {
    G_PREV_TIME.set(G_PREV_TIME.get().wrapping_sub(time));
}

/// `"%07.3lf"` milliseconds between `then` and `now`.
fn time_diff_str(then: ProfTime, now: ProfTime) -> String {
    format!("{:07.3}", profile_sub(now, then) as f64 / 1e6)
}

/// Append raw bytes to the startuptime log. No-op when `--startuptime` is
/// off or the bytes contain a NUL.
fn write_startup(bytes: &[u8]) {
    if bytes.contains(&0) {
        return;
    }
    time_fd.with(|log| {
        if let Some(log) = log {
            log.write(bytes);
        }
    });
}

/// Record a startup-timing message, if `--startuptime` asked for one: the
/// `TIME_MSG` macro of the C.
pub(crate) fn time_msg_at(what: &CStr) {
    time_msg(what.to_bytes(), None);
}

/// Write the startuptime report header and the first message. Must be
/// called once before [`time_msg`].
pub fn time_start(message: &CStr) {
    if !startup_timing() {
        return;
    }
    let now = profile_start();
    G_START_TIME.set(now);
    G_PREV_TIME.set(now);
    write_startup(
        b"\ntimes in msec\n clock   self+sourced   self:  sourced script\n clock   elapsed:              other lines\n\n",
    );
    time_msg_at(message);
}

/// One startuptime line: clock, the self+sourced column when `start` is
/// given (only for sourcing), elapsed, and the message.
pub fn time_msg(message: &[u8], start: Option<ProfTime>) {
    // The C formatted each label into an `IOSIZE` buffer, so a longer one (a
    // deep script path) is cut at this many bytes.
    const MESSAGE_MAX: usize = 1024;
    if !startup_timing() {
        return;
    }
    let now = profile_start();
    let mut line = time_diff_str(G_START_TIME.get(), now);
    if let Some(start) = start {
        line.push_str("  ");
        line.push_str(&time_diff_str(start, now));
    }
    line.push_str("  ");
    line.push_str(&time_diff_str(G_PREV_TIME.get(), now));
    G_PREV_TIME.set(now);
    line.push_str(": ");
    let mut bytes = line.into_bytes();
    bytes.extend_from_slice(&message[..message.len().min(MESSAGE_MAX)]);
    bytes.push(b'\n');
    write_startup(&bytes);
}

/// Report a failure to open the log on stderr, as the C's `fprintf` did: no
/// trailing newline.
fn report_to_stderr(text: &[u8]) {
    // Nothing to do when stderr itself is gone.
    let _ = std::io::stderr().write_all(text);
}

/// Open the `--startuptime` stream. The file is (potentially) written by
/// multiple nvim processes concurrently, so the report accumulates in a
/// full ("controlled") setvbuf buffer and is flushed to disk exactly once,
/// by [`time_finish`].
pub fn time_init(file_name: &CStr, process_name: &CStr) {
    const BUFSIZE: usize = 8192; // Big enough for the entire report.
    let Some(mut log) = CFile::open(file_name, c"a") else {
        report_to_stderr(&to_bytes(&tr!(
            "E484: Can't open file {}",
            msg_cstr(file_name)
        )));
        return;
    };
    if let Err(code) = log.buffer_fully(BUFSIZE + 1) {
        drop(log);
        let mut text = format!("time_init: setvbuf failed: {code} ").into_bytes();
        text.extend_from_slice(os_err_name(code).to_bytes());
        report_to_stderr(&text);
        return;
    }
    time_fd.set(Some(log));
    let mut header = b"--- Startup times for process: ".to_vec();
    header.extend_from_slice(process_name.to_bytes());
    header.extend_from_slice(b" ---\n");
    write_startup(&header);
}

/// Flush the startuptime report to disk and close the stream.
pub fn time_finish() {
    if !startup_timing() {
        return;
    }
    time_msg_at(c"--- NVIM STARTED ---\n");
    if let Some(log) = time_fd.take() {
        log.close();
    }
}
