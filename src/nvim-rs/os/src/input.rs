//! Input/terminal related functions

use std::ffi::c_int;

extern "C" {
    fn nvim_get_input_blocking() -> c_int;
}

/// Check if the main loop is blocked and waiting for input.
///
/// # Safety
/// Calls C accessor function for `blocking` static.
#[no_mangle]
pub unsafe extern "C" fn rs_input_blocking() -> c_int {
    nvim_get_input_blocking()
}

/// Check if a file descriptor refers to a terminal.
///
/// Uses libc `isatty()` which returns 1 if fd is a terminal, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_os_isatty(fd: c_int) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::isatty(fd) != 0 }
    }

    #[cfg(not(unix))]
    {
        // On Windows, this would need _isatty() from msvcrt
        let _ = fd;
        false
    }
}
