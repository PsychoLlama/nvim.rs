//! Process management functions

use std::ffi::c_int;

/// Check if a process is running.
///
/// Uses kill(pid, 0) to check if a process exists and can receive signals.
/// Returns true if the process is running (or we get EPERM, meaning we don't
/// have permission but the process exists).
#[unsafe(export_name = "os_proc_running")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_proc_running_current_process() {
        // Current process should be running
        let pid = unsafe { libc::getpid() };
        assert!(rs_os_proc_running(pid));
    }

    #[test]
    #[cfg(unix)]
    fn test_proc_running_init_process() {
        // PID 1 (init/systemd) should always be running
        assert!(rs_os_proc_running(1));
    }

    #[test]
    #[cfg(unix)]
    fn test_proc_running_nonexistent_pid() {
        // A very high negative PID should not exist
        // Note: PID -1 is special (broadcasts to all processes), so we use a different value
        // PID below -1 should return ESRCH (no such process)
        assert!(!rs_os_proc_running(-12345));
    }

    #[test]
    #[cfg(unix)]
    fn test_proc_running_unlikely_pid() {
        // A very high PID is unlikely to exist
        // Using a PID that's likely above the max (usually 32768 or 4194304)
        assert!(!rs_os_proc_running(999_999_999));
    }
}
