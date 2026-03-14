//! Input/terminal related functions

use std::ffi::c_int;

extern "C" {
    fn nvim_get_input_blocking() -> c_int;
}

/// Check if the main loop is blocked and waiting for input.
///
/// # Safety
/// Calls C accessor function for `blocking` static.
#[unsafe(export_name = "input_blocking")]
pub unsafe extern "C" fn rs_input_blocking() -> bool {
    nvim_get_input_blocking() != 0
}

/// Check if a file descriptor refers to a terminal.
///
/// Uses libc `isatty()` which returns 1 if fd is a terminal, 0 otherwise.
#[unsafe(export_name = "os_isatty")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isatty_invalid_fd() {
        // Negative file descriptors are never terminals
        assert!(!rs_os_isatty(-1));
        assert!(!rs_os_isatty(-100));
    }

    #[test]
    fn test_isatty_high_fd() {
        // Very high file descriptors are unlikely to be open/terminals
        assert!(!rs_os_isatty(999));
        assert!(!rs_os_isatty(10000));
    }

    #[test]
    #[cfg(unix)]
    fn test_isatty_closed_fd() {
        // File descriptors that are not open are not terminals
        // We use a likely-closed FD (e.g., 100)
        assert!(!rs_os_isatty(100));
    }
}
