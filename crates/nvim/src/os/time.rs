//! Clock queries and the editor's sleep/delay primitives.
//!
//! libuv owns the clocks and the poll loop, so every function here is a thin
//! layer over a `uv_*` or libc call. The layer itself is ordinary safe Rust:
//! the raw calls are confined to one block per function and the surface takes
//! and returns plain values.
#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

#[cfg(not(miri))]
use crate::event::libuv::uv_hrtime;
use crate::event::libuv::{uv_now, uv_sleep};
use crate::event::r#loop::process_events_until;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_DBG, logmsg};
use crate::os::cshim::{gettext, tzset};
use crate::os::env::env_get_bounded;
use crate::os::input::{InputEvents, input_ready};
use crate::startup::main_loop;
use crate::types::{Timestamp, time_t, tm};
use ::libc::{localtime_r, strftime, strptime};
use core::ffi::{CStr, c_int};
use core::ptr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A `struct tm` with every field zeroed, for callers about to fill it in.
///
/// The transpiled tree spelled this out field by field at every call site;
/// `tm` has no `Default` because it is a `repr(C)` libc type.
pub const fn tm_zeroed() -> tm {
    tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ptr::null(),
    }
}

/// A high-resolution (nanosecond), monotonically-increasing time relative to
/// an arbitrary point in the past.
///
/// Unrelated to the time of day, and so not subject to clock drift.
pub fn os_hrtime() -> u64 {
    // Miri cannot call into libuv. uv_hrtime on Linux is just
    // clock_gettime(CLOCK_MONOTONIC); a process-relative Instant reading
    // preserves the properties callers rely on (monotonic, ns resolution).
    #[cfg(miri)]
    {
        use std::sync::OnceLock;
        use std::time::Instant;
        static EPOCH: OnceLock<Instant> = OnceLock::new();
        EPOCH.get_or_init(Instant::now).elapsed().as_nanos() as u64
    }
    // SAFETY: uv_hrtime has no preconditions.
    #[cfg(not(miri))]
    unsafe {
        uv_hrtime()
    }
}

/// The current system time from a high-resolution real-time clock, in
/// nanoseconds since the UNIX epoch, or 0 if the clock could not be read.
///
/// The real-time clock is subject to time adjustments and can jump backwards.
pub fn os_realtime() -> i64 {
    // `uv_clock_gettime(UV_CLOCK_REALTIME)` is `clock_gettime(CLOCK_REALTIME)`,
    // which is what `SystemTime::now` reads; the error libuv could log is a
    // clock that does not exist.
    let nanos = |d: Duration| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX);
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(after) => nanos(after),
        Err(before) => -nanos(before.duration()),
    }
}

/// A millisecond-resolution, monotonically-increasing time relative to an
/// arbitrary point in the past.
///
/// The loop caches this: it does not change until the next loop tick.
pub fn os_now() -> u64 {
    // SAFETY: the main loop's uv handle is live for the editor's lifetime.
    unsafe { uv_now(&raw mut (*main_loop.ptr()).uv) }
}

/// Sleep for `ms` milliseconds, polling the loop meanwhile.
///
/// With `ignoreinput` only SIGINT (CTRL-C) cuts the delay short; otherwise
/// any available input does.
pub fn os_delay(ms: u64, ignoreinput: bool) {
    // Upstream reaches this through LOOP_PROCESS_EVENTS_UNTIL with a NULL
    // queue, so the macro's "drain this queue instead of polling" branch is
    // dead here and is not reproduced.
    logmsg!(LOGLVL_DBG, c"os_delay", 76, "{} ms", ms);
    let ms = ms.min(u64::from(c_int::MAX.cast_unsigned())).cast_signed();
    let done = || {
        if ignoreinput {
            got_int.get()
        } else {
            input_ready(InputEvents::None)
        }
    };
    // SAFETY: the process's event loop, always live, and no queue.
    unsafe { process_events_until(main_loop.ptr(), ptr::null_mut(), ms, done) };
}

/// Sleep for `ms` milliseconds serving the main loop's own queue, until an
/// interrupt -- what `:sleep` does.
pub(crate) fn sleep_serving_events(ms: i64) {
    // SAFETY: the main loop and its queue are live for the editor's
    // lifetime.
    unsafe {
        let uv_loop = main_loop.ptr();
        process_events_until(uv_loop, (*uv_loop).events, ms, || got_int.get());
    }
}

/// Sleep for `ms` milliseconds without checking for events or interrupts.
///
/// This blocks even "fast" events, which is disruptive; prefer [`os_delay`].
pub fn os_sleep(ms: u64) {
    let ms = u32::try_from(ms.min(u64::from(u32::MAX))).expect("clamped to a u32 above");
    // SAFETY: uv_sleep has no preconditions.
    unsafe { uv_sleep(ms) }
}

/// The TZ value `tzset` was last called for. POSIX does not require
/// `localtime_r` to re-read the zone the way `localtime` does, and calling
/// `tzset` on every conversion is too expensive, so the value is cached and
/// the zone is only refreshed when it changes. 63 octets plus terminator.
static TZ_CACHE: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// What fits in [`TZ_CACHE`], terminator included.
const TZ_LEN: usize = 64;

/// Thread-safe local-time conversion. Returns false if `clock` could not be
/// converted, leaving `result` untouched.
pub fn os_localtime_r(clock: time_t, result: &mut tm) -> bool {
    let tz = env_get_bounded(c"TZ");
    // Upstream compares only what the cache can hold, so a `$TZ` longer than
    // that does not re-`tzset` on every conversion.
    let head = &tz[..tz.len().min(TZ_LEN - 1)];
    if TZ_CACHE.with(|cache| cache.as_slice() != head) {
        // SAFETY: re-reads the zone for the value about to be cached.
        unsafe { tzset() };
        TZ_CACHE.set(head.to_vec());
    }
    // SAFETY: `localtime_r` fills `result`, which is live for the call.
    unsafe { !localtime_r(&raw const clock, result).is_null() }
}

/// [`os_localtime_r`] for the current time.
pub fn os_localtime(result: &mut tm) -> bool {
    os_localtime_r(os_time_raw(), result)
}

/// `clock` as local time, as `ctime_r` would render it into a buffer of
/// `room` bytes (at least two), optionally with a trailing newline: "(Invalid)"
/// when the time cannot be rendered.
pub(crate) fn ctime_text(clock: time_t, room: usize, add_newline: bool) -> Vec<u8> {
    let room = room.max(2);
    let mut local = tm_zeroed();
    // Upstream gives `strftime` `room - 1` bytes and `xstrlcpy` the same, so
    // the text is at most `room - 2` long and a newline always fits after.
    let mut text = if os_localtime_r(clock, &mut local) {
        strftime_within(gettext(c"%a %b %d %H:%M:%S %Y"), &local, room - 1)
    } else {
        Vec::new()
    };
    if text.is_empty() {
        text = gettext(c"(Invalid)").to_bytes().to_vec();
        text.truncate(room - 2);
    }
    if add_newline {
        text.push(b'\n');
    }
    text
}

/// How long the machine has been up, in seconds, or `None` when the system
/// will not say.
pub(crate) fn uptime() -> Option<f64> {
    let mut uptime: f64 = 0.0;
    // SAFETY: an out-parameter of this frame.
    (unsafe { crate::event::libuv::uv_uptime(&raw mut uptime) } == 0).then_some(uptime)
}

/// Parse `text` according to `format` into `tm`: whether the format
/// matched.
pub fn os_strptime(text: &CStr, format: &CStr, tm: &mut tm) -> bool {
    // SAFETY: both strings are NUL-terminated and `tm` is live for the call;
    // the answer (where the parse stopped) is only tested for NULL.
    !unsafe { strptime(text.as_ptr(), format.as_ptr(), tm) }.is_null()
}

/// `time` formatted by `strftime`'s `format`, at most 255 bytes of it.
/// Empty when the formatted text would not fit, as `strftime` answers it.
pub(crate) fn os_strftime(format: &CStr, time: &tm) -> Vec<u8> {
    strftime_within(format, time, 256)
}

/// `strftime` into a buffer of `room` bytes: the text, or empty when it
/// (and its NUL) would not fit.
fn strftime_within(format: &CStr, time: &tm, room: usize) -> Vec<u8> {
    let mut out = vec![0u8; room];
    // SAFETY: a terminated format, a filled-in time, and `room` bytes of
    // this call's own to write into.
    let len = unsafe { strftime(out.as_mut_ptr().cast(), room, format.as_ptr(), time) };
    out.truncate(len);
    out
}

/// `mktime` of `time`, which it normalises in place: seconds since the
/// epoch, or -1 when the time cannot be represented.
pub(crate) fn os_mktime(time: &mut tm) -> time_t {
    // SAFETY: a live time of the caller's own.
    unsafe { ::libc::mktime(time) }
}

/// Seconds since the UNIX epoch.
pub fn os_time() -> Timestamp {
    os_time_raw().cast_unsigned()
}

/// Seconds since the UNIX epoch, as the C library counts them.
#[cfg(not(miri))]
pub(crate) fn os_time_raw() -> time_t {
    // SAFETY: `time` accepts a null out-parameter.
    unsafe { ::libc::time(ptr::null_mut()) }
}

/// [`os_time_raw`] where the C library cannot be called (Miri): whole
/// seconds of the real-time clock, floored as `time(NULL)` floors them.
/// The C call stays the shipped one: it is a vDSO read, and this is several
/// times its cost on the undo path, which stamps every change.
#[cfg(miri)]
pub(crate) fn os_time_raw() -> time_t {
    let seconds = |d: Duration| time_t::try_from(d.as_secs()).unwrap_or(time_t::MAX);
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(after) => seconds(after),
        Err(before) => {
            let before = before.duration();
            -seconds(before) - time_t::from(before.subsec_nanos() > 0)
        }
    }
}
