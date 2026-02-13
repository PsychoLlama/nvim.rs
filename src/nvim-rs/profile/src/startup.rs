//! Startup timing.
//!
//! Implements the `--startuptime` report functionality. Tracks elapsed time
//! from startup and writes timing data to a file.

use std::os::raw::{c_char, c_int};

use crate::types::FileHandle;
use crate::Proftime;

extern "C" {
    fn nvim_profile_get_time_fd() -> FileHandle;
    fn nvim_profile_set_time_fd(fd: FileHandle);
    fn nvim_profile_fopen(name: *const c_char, mode: *const c_char) -> FileHandle;
    fn nvim_profile_fclose(fd: FileHandle);
    fn nvim_profile_fputs(s: *const c_char, fd: FileHandle);
    fn nvim_profile_xmalloc(size: usize) -> *mut c_char;
    fn nvim_profile_xfree(ptr: *mut c_char);
    fn nvim_profile_setvbuf(fd: FileHandle, buf: *mut c_char, size: usize) -> c_int;
    fn nvim_profile_fprintf_stderr(s: *const c_char);
    fn nvim_profile_get_startuptime_buf() -> *mut c_char;
    fn nvim_profile_set_startuptime_buf(buf: *mut c_char);
    fn nvim_profile_gettext_e_notopen() -> *const c_char;
    fn nvim_profile_uv_err_name(r: c_int) -> *const c_char;
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
}

/// Global start time for startuptime report.
static mut G_START_TIME: Proftime = 0;

/// Previous time for startuptime report (used for elapsed deltas).
static mut G_PREV_TIME: Proftime = 0;

/// Write a time difference to the time_fd in "msec.usec" format.
unsafe fn time_diff(then: Proftime, now: Proftime, fd: FileHandle) {
    let diff = crate::rs_profile_sub(now, then);
    let mut buf = [0u8; 32];
    snprintf(
        buf.as_mut_ptr().cast::<c_char>(),
        buf.len(),
        c"%07.3lf".as_ptr(),
        diff as f64 / 1.0e6,
    );
    nvim_profile_fputs(buf.as_ptr().cast::<c_char>(), fd);
}

/// Saves the previous time before doing something that could nest.
///
/// # Safety
///
/// `rel` and `start` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_time_push(rel: *mut Proftime, start: *mut Proftime) {
    let now = crate::timing::rs_profile_start();
    let prev = std::ptr::addr_of!(G_PREV_TIME).read();

    *rel = crate::rs_profile_sub(now, prev);
    *start = now;

    std::ptr::addr_of_mut!(G_PREV_TIME).write(now);
}

/// Computes the prev time after doing something that could nest.
///
/// # Safety
///
/// Accesses mutable static.
#[no_mangle]
pub unsafe extern "C" fn rs_time_pop(tp: Proftime) {
    let ptr = std::ptr::addr_of_mut!(G_PREV_TIME);
    ptr.write(ptr.read().wrapping_sub(tp));
}

/// Initializes the startuptime code.
///
/// # Safety
///
/// `message` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_time_start(message: *const c_char) {
    let fd = nvim_profile_get_time_fd();
    if fd.is_null() {
        return;
    }

    let now = crate::timing::rs_profile_start();
    std::ptr::addr_of_mut!(G_PREV_TIME).write(now);
    std::ptr::addr_of_mut!(G_START_TIME).write(now);

    nvim_profile_fputs(c"\ntimes in msec\n".as_ptr(), fd);
    nvim_profile_fputs(
        c" clock   self+sourced   self:  sourced script\n".as_ptr(),
        fd,
    );
    nvim_profile_fputs(
        c" clock   elapsed:              other lines\n\n".as_ptr(),
        fd,
    );

    rs_time_msg(message, std::ptr::null());
}

/// Prints out timing info.
///
/// # Safety
///
/// `mesg` must be a valid C string. `start` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_time_msg(mesg: *const c_char, start: *const Proftime) {
    let fd = nvim_profile_get_time_fd();
    if fd.is_null() {
        return;
    }

    let now = crate::timing::rs_profile_start();
    let g_start = std::ptr::addr_of!(G_START_TIME).read();
    let g_prev = std::ptr::addr_of!(G_PREV_TIME).read();

    time_diff(g_start, now, fd);

    if !start.is_null() {
        nvim_profile_fputs(c"  ".as_ptr(), fd);
        time_diff(*start, now, fd);
    }

    nvim_profile_fputs(c"  ".as_ptr(), fd);
    time_diff(g_prev, now, fd);

    std::ptr::addr_of_mut!(G_PREV_TIME).write(now);

    nvim_profile_fputs(c": ".as_ptr(), fd);
    nvim_profile_fputs(mesg, fd);
    nvim_profile_fputs(c"\n".as_ptr(), fd);
}

/// Initializes the `time_fd` stream for the --startuptime report.
///
/// # Safety
///
/// `fname` and `proc_name` must be valid C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_time_init(fname: *const c_char, proc_name: *const c_char) {
    let bufsize: usize = 8192;
    let fd = nvim_profile_fopen(fname, c"a".as_ptr());
    if fd.is_null() {
        // Format error message: e_notopen contains "%s" format
        let fmt = nvim_profile_gettext_e_notopen();
        let mut errbuf = [0u8; 512];
        snprintf(
            errbuf.as_mut_ptr().cast::<c_char>(),
            errbuf.len(),
            fmt,
            fname,
        );
        nvim_profile_fprintf_stderr(errbuf.as_ptr().cast::<c_char>());
        return;
    }
    nvim_profile_set_time_fd(fd);

    let stbuf = nvim_profile_xmalloc(bufsize + 1);
    nvim_profile_set_startuptime_buf(stbuf);

    let r = nvim_profile_setvbuf(fd, stbuf, bufsize + 1);
    if r != 0 {
        nvim_profile_set_startuptime_buf(std::ptr::null_mut());
        nvim_profile_xfree(stbuf);
        nvim_profile_fclose(fd);
        nvim_profile_set_time_fd(std::ptr::null_mut());

        // Format: "time_init: setvbuf failed: %d %s"
        let errname = nvim_profile_uv_err_name(r);
        let mut errbuf = [0u8; 256];
        snprintf(
            errbuf.as_mut_ptr().cast::<c_char>(),
            errbuf.len(),
            c"time_init: setvbuf failed: %d %s".as_ptr(),
            r,
            errname,
        );
        nvim_profile_fprintf_stderr(errbuf.as_ptr().cast::<c_char>());
        return;
    }

    // Write header
    let mut hdr = [0u8; 256];
    snprintf(
        hdr.as_mut_ptr().cast::<c_char>(),
        hdr.len(),
        c"--- Startup times for process: %s ---\n".as_ptr(),
        proc_name,
    );
    nvim_profile_fputs(hdr.as_ptr().cast::<c_char>(), fd);
}

/// Flushes the startuptimes to disk for the current process.
///
/// # Safety
///
/// Accesses global time_fd and startuptime_buf.
#[no_mangle]
pub unsafe extern "C" fn rs_time_finish() {
    let fd = nvim_profile_get_time_fd();
    if fd.is_null() {
        return;
    }

    // TIME_MSG equivalent: call time_msg if time_fd != NULL
    rs_time_msg(c"--- NVIM STARTED ---\n".as_ptr(), std::ptr::null());

    // Flush buffer to disk
    nvim_profile_fclose(fd);
    nvim_profile_set_time_fd(std::ptr::null_mut());

    let stbuf = nvim_profile_get_startuptime_buf();
    if !stbuf.is_null() {
        nvim_profile_xfree(stbuf);
        nvim_profile_set_startuptime_buf(std::ptr::null_mut());
    }
}
