//! The `--startuptime` log: one line per startup event, with the elapsed
//! and (for sourced scripts) self+sourced columns.
//!
//! This half stays on C stdio. The file is potentially appended to by
//! several nvim processes at once, so the whole report accumulates in a
//! full ("controlled") `setvbuf` buffer and reaches the disk exactly once,
//! at [`time_finish`] — which is what keeps two concurrent processes'
//! reports from interleaving line by line.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{profile_start, profile_sub};
use crate::event::libuv::uv_err_name;
use crate::global_cell::GlobalCell;
use crate::main::{e_notopen, time_fd};
use crate::memory::{xfree, xmalloc};
use crate::os::cshim::{gettext, stderr};
use crate::types::proftime_T;
use ::libc::{fclose, fopen, fprintf, setvbuf};
use core::ffi::{CStr, c_char, c_int, c_void};
use std::ffi::CString;

// ---------------------------------------------------------------------------
// --startuptime.

/// When `time_start()` was called.
static G_START_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);
/// Time of the previous event line, for the "elapsed" column.
static G_PREV_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);
/// The setvbuf buffer handed to `time_fd`; freed at [`time_finish`].
static STARTUPTIME_BUF: GlobalCell<*mut c_char> = GlobalCell::new(core::ptr::null_mut());

/// Save the previous time before doing something that could nest (sourcing
/// a script from a script). Returns `(rel, start)`: the time elapsed so far
/// (to hand to [`time_pop`]) and the current time.
pub fn time_push() -> (proftime_T, proftime_T) {
    let now = profile_start();
    let rel = profile_sub(now, G_PREV_TIME.get());
    G_PREV_TIME.set(now);
    (rel, now)
}

/// Subtract the nested duration `tp` (from [`time_push`]) from the
/// previous-event time.
pub fn time_pop(tp: proftime_T) {
    G_PREV_TIME.set(G_PREV_TIME.get().wrapping_sub(tp));
}

/// `"%07.3lf"` milliseconds between `then` and `now`.
fn time_diff_str(then: proftime_T, now: proftime_T) -> String {
    format!("{:07.3}", profile_sub(now, then) as f64 / 1e6)
}

/// Append raw bytes to the startuptime log. No-op when `--startuptime` is
/// off or the bytes contain a NUL.
fn write_startup(bytes: &[u8]) {
    let fd = time_fd.get();
    if fd.is_null() {
        return;
    }
    if let Ok(line) = CString::new(bytes) {
        // SAFETY: fd is the open startuptime stream; "%s" consumes the one
        // string argument.
        unsafe { fprintf(fd, c"%s".as_ptr(), line.as_ptr()) };
    }
}

/// Write the startuptime report header and the first message. Must be
/// called once before [`time_msg`].
///
/// # Safety
/// `message` is NUL-terminated.
pub unsafe fn time_start(message: *const c_char) {
    if time_fd.get().is_null() {
        return;
    }
    let now = profile_start();
    G_START_TIME.set(now);
    G_PREV_TIME.set(now);
    write_startup(
        b"\ntimes in msec\n clock   self+sourced   self:  sourced script\n clock   elapsed:              other lines\n\n",
    );
    // SAFETY: the caller's message.
    unsafe { time_msg(message, core::ptr::null()) };
}

/// One startuptime line: clock, optional self+sourced (when `start` is
/// non-null, only for sourcing), elapsed, and the message.
///
/// # Safety
/// `mesg` is NUL-terminated; `start` is null or points at a readable
/// `proftime_T`.
pub unsafe fn time_msg(mesg: *const c_char, start: *const proftime_T) {
    if time_fd.get().is_null() {
        return;
    }
    let now = profile_start();
    let mut line = time_diff_str(G_START_TIME.get(), now);
    if !start.is_null() {
        line.push_str("  ");
        // SAFETY: non-null, so the caller's contract makes it readable.
        line.push_str(&time_diff_str(unsafe { *start }, now));
    }
    line.push_str("  ");
    line.push_str(&time_diff_str(G_PREV_TIME.get(), now));
    G_PREV_TIME.set(now);
    line.push_str(": ");
    let mut bytes = line.into_bytes();
    // SAFETY: the caller's NUL-terminated message.
    bytes.extend_from_slice(unsafe { CStr::from_ptr(mesg) }.to_bytes());
    bytes.push(b'\n');
    write_startup(&bytes);
}

/// Open the `--startuptime` stream. The file is (potentially) written by
/// multiple nvim processes concurrently, so the report accumulates in a
/// full ("controlled") setvbuf buffer and is flushed to disk exactly once,
/// by [`time_finish`].
///
/// # Safety
/// `fname` and `proc_name` are NUL-terminated.
pub unsafe fn time_init(fname: *const c_char, proc_name: *const c_char) {
    const BUFSIZE: usize = 8192; // Big enough for the entire report.
    const _IOFBF: c_int = 0;
    // SAFETY: the caller's path; the handle is stored in `time_fd`, which is
    // what closes it.
    time_fd.set(unsafe { fopen(fname, c"a".as_ptr()) });
    if time_fd.get().is_null() {
        // SAFETY: the message is a NUL-terminated global with one %s.
        unsafe { fprintf(stderr, gettext(e_notopen.as_ptr()), fname) };
        return;
    }
    // SAFETY: `xmalloc` returns `BUFSIZE + 1` owned bytes, which is exactly
    // the size handed to `setvbuf`; the buffer outlives the stream because
    // `time_finish` frees it after `fclose`.
    let r = unsafe {
        STARTUPTIME_BUF.set(xmalloc(BUFSIZE + 1) as *mut c_char);
        setvbuf(time_fd.get(), STARTUPTIME_BUF.get(), _IOFBF, BUFSIZE + 1)
    };
    if r != 0 {
        // SAFETY: the buffer and stream just set up, released here and
        // cleared so nothing reaches them again.
        unsafe { xfree(STARTUPTIME_BUF.replace(core::ptr::null_mut()) as *mut c_void) };
        unsafe { fclose(time_fd.get()) };
        time_fd.set(core::ptr::null_mut());
        let fmt = c"time_init: setvbuf failed: %d %s".as_ptr();
        let why = unsafe { uv_err_name(r) };
        unsafe { fprintf(stderr, fmt, r, why) };
        return;
    }
    let mut header = b"--- Startup times for process: ".to_vec();
    // SAFETY: the caller's NUL-terminated process name.
    header.extend_from_slice(unsafe { CStr::from_ptr(proc_name) }.to_bytes());
    header.extend_from_slice(b" ---\n");
    write_startup(&header);
}

/// Flush the startuptime report to disk and close the stream.
pub fn time_finish() {
    if time_fd.get().is_null() {
        return;
    }
    debug_assert!(!STARTUPTIME_BUF.get().is_null());
    // SAFETY: the stream and its buffer were set up by time_init; nothing
    // touches them after the fd is cleared.
    unsafe { time_msg(c"--- NVIM STARTED ---\n".as_ptr(), core::ptr::null()) };
    unsafe { fclose(time_fd.get()) };
    time_fd.set(core::ptr::null_mut());
    unsafe { xfree(STARTUPTIME_BUF.replace(core::ptr::null_mut()) as *mut c_void) };
}
