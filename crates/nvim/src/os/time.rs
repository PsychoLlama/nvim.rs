//! Clock queries and the editor's sleep/delay primitives.
//!
//! libuv owns the clocks and the poll loop, so every function here is a thin
//! layer over a `uv_*` or libc call. The layer itself is ordinary safe Rust:
//! the raw calls are confined to one block per function and the surface takes
//! and returns plain values.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
#[cfg(not(miri))]
use crate::event::libuv::uv_hrtime;
use crate::event::libuv::{uv_clock_gettime, uv_err_name, uv_now, uv_sleep};
use crate::event::r#loop::process_events_until;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_DBG, LOGLVL_ERR, logmsg};
use crate::main::{got_int, main_loop};
use crate::memory::{xstrlcat, xstrlcpy};
use crate::message_fmt::c_str;
use crate::os::cshim::{gettext, tzset};
use crate::os::env::{env_buf, os_getenv_into};
use crate::os::input::os_input_ready;
use crate::types::{Timestamp, UV_CLOCK_REALTIME, time_t, tm, uv_timespec64_t};
use ::libc::{localtime_r, strftime, strptime, time};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

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
    let mut ts = uv_timespec64_t {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `ts` is a live, correctly typed out-parameter; the log call's
    // format string matches its two arguments.
    let error_number = unsafe {
        let error_number = uv_clock_gettime(UV_CLOCK_REALTIME, &raw mut ts);
        if error_number != 0 {
            logmsg!(
                LOGLVL_ERR,
                c"os_realtime",
                48,
                "uv_clock_gettime failed: {} {}",
                error_number,
                c_str(uv_err_name(error_number))
            );
        }
        error_number
    };
    if error_number != 0 {
        return 0;
    }
    ts.tv_sec * 1_000_000_000 + i64::from(ts.tv_nsec)
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
    //
    // SAFETY: the main loop is live; the log call's format matches its one
    // argument; os_input_ready accepts a null queue.
    unsafe {
        logmsg!(LOGLVL_DBG, c"os_delay", 76, "{} ms", ms);
        let ms = ms.min(c_int::MAX as u64) as i64;
        process_events_until(main_loop.ptr(), ptr::null_mut(), ms, || {
            if ignoreinput {
                got_int.get()
            } else {
                os_input_ready(ptr::null_mut())
            }
        });
    }
}

/// Sleep for `ms` milliseconds without checking for events or interrupts.
///
/// This blocks even "fast" events, which is disruptive; prefer [`os_delay`].
pub fn os_sleep(ms: u64) {
    // SAFETY: uv_sleep has no preconditions.
    unsafe { uv_sleep(ms.min(u32::MAX as u64) as u32) }
}

/// The TZ value `tzset` was last called for. POSIX does not require
/// `localtime_r` to re-read the zone the way `localtime` does, and calling
/// `tzset` on every conversion is too expensive, so the value is cached and
/// the zone is only refreshed when it changes. 63 octets plus terminator.
static TZ_CACHE: GlobalCell<[c_char; TZ_LEN]> = GlobalCell::new([0; TZ_LEN]);

/// What fits in [`TZ_CACHE`], terminator included.
const TZ_LEN: usize = 64;

/// Thread-safe local-time conversion. Returns false if `clock` could not be
/// converted, leaving `result` untouched.
pub fn os_localtime_r(clock: time_t, result: &mut tm) -> bool {
    let mut env = env_buf();
    // SAFETY: `$TZ` is NUL-terminated where it is not NULL, and it lands in
    // `env`, which outlives the borrow.
    let tz = unsafe {
        let value = os_getenv_into(c"TZ".as_ptr(), &mut env);
        if value.is_null() {
            c""
        } else {
            CStr::from_ptr(value)
        }
    };
    // Upstream compares only what the cache can hold, so a `$TZ` longer than
    // that does not re-`tzset` on every conversion.
    let head = &tz.to_bytes()[..tz.to_bytes().len().min(TZ_LEN - 1)];
    if TZ_CACHE.with(|cache| cstr::in_chars(cache).to_bytes() != head) {
        let mut cache = [0 as c_char; TZ_LEN];
        for (slot, &b) in cache.iter_mut().zip(head) {
            *slot = b.cast_signed();
        }
        // SAFETY: re-reads the zone for the value just cached.
        unsafe { tzset() };
        TZ_CACHE.set(cache);
    }
    // SAFETY: `localtime_r` fills `result`, which is live for the call.
    unsafe { !localtime_r(&raw const clock, result).is_null() }
}

/// [`os_localtime_r`] for the current time.
pub fn os_localtime(result: &mut tm) -> bool {
    os_localtime_r(os_time_raw(), result)
}

/// Render `clock` as local time into `result`, as `ctime_r` would, optionally
/// with a trailing newline. Yields "(Invalid)" when the time cannot be
/// rendered. Returns `result`'s base pointer for the C-shaped callers.
pub fn os_ctime_r(clock: time_t, result: &mut [c_char], add_newline: bool) -> *mut c_char {
    let mut local = tm_zeroed();
    let filled = os_localtime_r(clock, &mut local);
    let len = result.len();
    let out = result.as_mut_ptr();
    // SAFETY: `out` addresses `len` writable chars; strftime and the xstrlc*
    // pair are each given a bound within that. `local` is live and filled
    // whenever it is read.
    unsafe {
        if !filled
            || strftime(
                out,
                len - 1,
                gettext(c"%a %b %d %H:%M:%S %Y").as_ptr(),
                &raw mut local,
            ) == 0
        {
            xstrlcpy(out, gettext(c"(Invalid)").as_ptr(), len - 1);
        }
        if add_newline {
            xstrlcat(out, c"\n".as_ptr(), len);
        }
    }
    out
}

/// Parse `str` according to `format` into `tm`. Returns a pointer into `str`
/// just past the parsed text, or null if the input did not match.
pub fn os_strptime(str: &CStr, format: &CStr, tm: &mut tm) -> *mut c_char {
    // SAFETY: both strings are NUL-terminated by construction and `tm` is
    // live for the call.
    unsafe { strptime(str.as_ptr(), format.as_ptr(), tm) }
}

/// Seconds since the UNIX epoch.
pub fn os_time() -> Timestamp {
    os_time_raw() as Timestamp
}

fn os_time_raw() -> time_t {
    // SAFETY: `time` accepts a null out-parameter.
    unsafe { time(ptr::null_mut()) }
}
