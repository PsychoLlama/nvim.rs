//! OS process inspection and termination.
//!
//! # Boundary
//!
//! `uv_kill` is libuv's `kill(2)`, returning a negated errno rather than
//! setting one. Everything else is `/proc`, read through `std::fs`.
//!
//! This is the Linux build of upstream's `os/proc.c`; the Windows
//! (toolhelp32) and BSD/macOS (`sysctl KERN_PROC`) implementations of
//! [`os_proc_children`] were not transpiled, and with them went the
//! "process not found" outcome that only those platforms could report.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::event::libuv::uv_kill;
use crate::src::nvim::log::logmsg;
use core::ffi::c_int;
use core::ptr;
use std::ffi::CString;

const LOGLVL_INF: c_int = 2;
const SIGKILL: c_int = 9;
const SIGTERM: c_int = 15;
/// `uv_kill` reporting ESRCH: no such process.
const UV_ESRCH: c_int = -3;

/// Kill the process group led by `pid`, which is what nvim's spawned jobs
/// are set up as. `sig` must be SIGTERM or SIGKILL.
///
/// Returns whether the signal was delivered.
pub fn os_proc_tree_kill(pid: c_int, sig: c_int) -> bool {
    assert!(sig == SIGTERM || sig == SIGKILL);
    if pid == 0 {
        // Never kill self: `kill(0, ...)` signals our own process group.
        return false;
    }
    let name = if sig == SIGTERM { "SIGTERM" } else { "SIGKILL" };
    let text = CString::new(format!("sending {name} to PID {}", -pid)).expect("no interior NUL");
    // SAFETY: `logmsg` is variadic and printf-shaped, so the message goes
    // through `%s` rather than becoming the format string itself; both
    // pointers outlive the call. `uv_kill` takes no pointers.
    unsafe {
        logmsg(
            LOGLVL_INF,
            ptr::null(),
            c"os_proc_tree_kill".as_ptr(),
            103,
            true,
            c"%s".as_ptr(),
            text.as_ptr(),
        );
        uv_kill(-pid, sig) == 0
    }
}

/// The pids of the immediate children of `ppid`, or `None` when the process
/// could not be inspected — the caller is expected to fall back to
/// `vim._os_proc_children()`.
///
/// Children are read from the *thread* of the same id as the process, which
/// is where Linux records them.
pub fn os_proc_children(ppid: c_int) -> Option<Vec<c_int>> {
    if ppid < 0 {
        return None;
    }
    let children = std::fs::read_to_string(format!("/proc/{ppid}/task/{ppid}/children")).ok()?;
    Some(parse_pids(&children))
}

/// Leading whitespace-separated integers, stopping at the first token that
/// is not one — `fscanf("%d")` in a loop, which is what the C did.
fn parse_pids(text: &str) -> Vec<c_int> {
    text.split_ascii_whitespace()
        .map_while(|token| token.parse().ok())
        .collect()
}

/// Whether process `pid` is running.
///
/// A process owned by another user answers EPERM rather than ESRCH; only a
/// definite ESRCH counts as "gone".
pub fn os_proc_running(pid: c_int) -> bool {
    // SAFETY: `uv_kill` takes no pointers. Signal 0 delivers nothing and
    // only probes for the process's existence.
    unsafe { uv_kill(pid, 0) != UV_ESRCH }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pids_parse_until_the_first_non_integer() {
        assert_eq!(parse_pids("123 456\n"), [123, 456]);
        assert_eq!(parse_pids("7 x 9"), [7]);
        assert_eq!(parse_pids(""), []);
        assert_eq!(parse_pids("  \n "), []);
    }
}
