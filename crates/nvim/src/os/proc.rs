//! OS process inspection and termination.
//!
//! # Boundary
//!
//! `uv_kill` is libuv's `kill(2)`, returning a negated errno rather than
//! setting one. Everything else is `/proc`, read through `std::fs`.
//!
//! The descriptor shuffles a channel over this process's own stdio needs
//! (`os_dup_above`, `os_dup2`, `os_stderr_to_null`) live here too.
//!
//! This is the Linux build of upstream's `os/proc.c`; the Windows
//! (toolhelp32) and BSD/macOS (`sysctl KERN_PROC`) implementations of
//! [`os_proc_children`] were not transpiled, and with them went the
//! "process not found" outcome that only those platforms could report.

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

use crate::event::libuv::uv_kill;
use crate::log::{LOGLVL_INF, logmsg};
use crate::message_fmt::msg_cstr;
use crate::os::uv_error::UV_ESRCH;
use core::ffi::c_int;
use std::ffi::CString;

const SIGKILL: c_int = 9;
const SIGTERM: c_int = 15;

/// Kill the process group led by `pid`, which is what nvim's spawned jobs
/// are set up as. `sig` must be SIGTERM or SIGKILL.
///
/// Returns whether the signal was delivered.
pub fn os_proc_tree_kill(pid: c_int, sig: c_int) -> bool {
    debug_assert!(sig == SIGTERM || sig == SIGKILL);
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
        logmsg!(LOGLVL_INF, c"os_proc_tree_kill", 103, "{}", msg_cstr(&text));
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
/// Send `sig` to process `pid`; whether it was sent.
pub(crate) fn os_kill(pid: c_int, sig: c_int) -> bool {
    // SAFETY: `uv_kill` takes no pointers.
    unsafe { uv_kill(pid, sig) == 0 }
}

/// A process owned by another user answers EPERM rather than ESRCH; only a
/// definite ESRCH counts as "gone".
pub fn os_proc_running(pid: c_int) -> bool {
    // SAFETY: `uv_kill` takes no pointers. Signal 0 delivers nothing and
    // only probes for the process's existence.
    unsafe { uv_kill(pid, 0) != UV_ESRCH }
}

/// Moves `fd` aside: a duplicate at the lowest free descriptor at or above
/// `min`, close-on-exec. Answers it, or -1.
pub(crate) fn os_dup_above(fd: c_int, min: c_int) -> c_int {
    /// `fcntl` command: dup to the lowest free descriptor at or above the
    /// third argument, with close-on-exec set.
    const F_DUPFD_CLOEXEC: c_int = 1030;
    // SAFETY: `fcntl` on a descriptor number; a closed one answers -1.
    unsafe { ::libc::fcntl(fd, F_DUPFD_CLOEXEC, min) }
}

/// Points descriptor `to` at whatever `fd` is open on, when it can.
pub(crate) fn os_dup2(fd: c_int, to: c_int) {
    // SAFETY: `dup2` on two descriptor numbers; a closed one fails, which
    // leaves `to` as it was.
    unsafe { ::libc::dup2(fd, to) };
}

/// Reopens this process's stderr on `/dev/null`, so that whatever is still
/// written to it goes nowhere.
pub(crate) fn os_stderr_to_null() {
    // SAFETY: two NUL-terminated literals and the C library's own `stderr`
    // stream.
    unsafe {
        ::libc::freopen(
            c"/dev/null".as_ptr(),
            c"w".as_ptr(),
            crate::os::cshim::stderr,
        )
    };
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
