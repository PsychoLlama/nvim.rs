//! Process management functions

use std::ffi::c_int;

/// Check if a process is running.
///
/// Uses kill(pid, 0) to check if a process exists and can receive signals.
/// Returns true if the process is running (or we get EPERM, meaning we don't
/// have permission but the process exists).
#[no_mangle]
pub extern "C" fn rs_os_proc_running(pid: c_int) -> bool {
    #[cfg(unix)]
    {
        // kill(pid, 0) checks if we can send a signal to the process
        // - Returns 0 if the process exists and we have permission
        // - Returns -1 with ESRCH if the process doesn't exist
        // - Returns -1 with EPERM if process exists but we don't have permission
        let result = unsafe { libc::kill(pid, 0) };
        if result == 0 {
            return true;
        }
        let errno = unsafe { *libc::__errno_location() };
        // ESRCH (3) means the process doesn't exist
        // EPERM (1) means process exists but we don't have permission
        // For any other error, assume the process is running
        errno != libc::ESRCH
    }

    #[cfg(not(unix))]
    {
        // On Windows, this would need to use OpenProcess/GetExitCodeProcess
        // For now, assume running
        let _ = pid;
        true
    }
}
