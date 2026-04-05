//! Server connection and OS exit
//!
//! Implements `rs_server_connect` and `rs_os_exit` replacing static C
//! functions in main.c.

use std::ffi::{c_char, c_int};

// Standard file descriptor numbers
const STDIN_FILENO: c_int = 0;

unsafe extern "C" {
    fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;

    // C helper: wraps channel_connect with CALLBACK_READER_INIT
    fn nvim_channel_connect(
        is_tcp: bool,
        server_addr: *const c_char,
        error: *mut *const c_char,
    ) -> u64;

    // C helper: exposes static event_teardown()
    fn nvim_event_teardown() -> bool;

    // C helper: calls free_all_mem() only when EXITFREE is defined
    fn nvim_free_all_mem_if_exitfree();

    fn ui_client_stop();
    fn ui_flush();
    fn ui_call_stop();
    fn ml_close_all(del_file: bool);
    fn stream_set_blocking(fd: c_int, blocking: bool);
    fn exit(r: c_int) -> !;

    static mut exiting: bool;
    static mut ui_client_channel_id: u64;
    static mut ui_client_exit_status: c_int;
    static mut used_stdin: bool;
}

/// Connect to a remote Nvim server.
///
/// Returns the channel id on success, 0 on failure (sets *errmsg).
///
/// # Safety
/// `server_addr` must be a valid C string or null.
/// `errmsg` must be a valid non-null pointer to a `*const c_char`.
#[no_mangle]
pub unsafe extern "C" fn rs_server_connect(
    server_addr: *mut c_char,
    errmsg: *mut *const c_char,
) -> u64 {
    if server_addr.is_null() {
        *errmsg = c"no address specified".as_ptr();
        return 0;
    }

    // Detect TCP: address contains ':'
    let is_tcp = !strrchr(server_addr, b':' as c_int).is_null();

    let mut error: *const c_char = std::ptr::null();
    let chan = nvim_channel_connect(is_tcp, server_addr, &mut error);
    if !error.is_null() {
        *errmsg = error;
        return 0;
    }
    chan
}

/// Exit Nvim with UI teardown, memfile close, and stream normalization.
///
/// # Safety
/// Must only be called from the main thread during shutdown.
#[no_mangle]
pub unsafe extern "C" fn rs_os_exit(mut r: c_int) -> ! {
    exiting = true;

    if ui_client_channel_id != 0 {
        ui_client_stop();
        if r == 0 {
            r = ui_client_exit_status;
        }
    } else {
        ui_flush();
        ui_call_stop();
    }

    if !nvim_event_teardown() && r == 0 {
        r = 1; // Exit with error if main_loop did not teardown gracefully.
    }
    if ui_client_channel_id == 0 {
        ml_close_all(true); // remove all memfiles
    }
    if used_stdin {
        stream_set_blocking(STDIN_FILENO, true); // normalize stream (#2598)
    }

    // ILOG omitted (C macro, not worth bridging for teardown)

    nvim_free_all_mem_if_exitfree();

    exit(r)
}
