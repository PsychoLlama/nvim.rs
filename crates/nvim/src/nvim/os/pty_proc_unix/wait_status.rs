//! Decoding the status word `waitpid` fills in.
//!
//! `<sys/wait.h>` exposes this only as macros, so the bit layout is spelled
//! out here: the low seven bits carry the terminating signal (zero for a
//! normal exit), bit 7 says a core was dumped, and the next byte carries the
//! exit code. A stopped child is reported as `0x7f` in the low byte with the
//! stopping signal above it, and a continued one as the reserved `0xffff`.
#![forbid(unsafe_code)]

use core::ffi::c_int;

/// What one `waitpid` report says happened to the child.
#[derive(Debug, PartialEq, Eq)]
pub enum ChildState {
    /// Suspended, by SIGSTOP/SIGTSTP or the like.
    Stopped,
    /// Resumed by SIGCONT.
    Continued,
    /// Gone. `status` is the exit code, or 128 plus the signal that killed
    /// it — the encoding a shell reports. It is `None` for a word that
    /// claims neither, in which case upstream leaves the child's recorded
    /// status untouched and still reports the exit.
    Exited { status: Option<c_int> },
}

pub fn decode(stat: c_int) -> ChildState {
    if stat & 0xff == 0x7f {
        return ChildState::Stopped;
    }
    if stat == 0xffff {
        return ChildState::Continued;
    }
    let signal = stat & 0x7f;
    let status = if signal == 0 {
        Some((stat & 0xff00) >> 8)
    } else if is_signalled(signal) {
        Some(128 + signal)
    } else {
        None
    };
    ChildState::Exited { status }
}

/// glibc's `WIFSIGNALED`: true for a low byte that is neither zero (a normal
/// exit) nor `0x7f` (stopped). Written the way the C macro is, because the
/// signed-byte truncation is what excludes `0x7f`.
fn is_signalled(signal: c_int) -> bool {
    (((signal + 1) as i8) as c_int) >> 1 > 0
}

#[cfg(test)]
mod tests {
    use super::{ChildState, decode, is_signalled};
    use core::ffi::c_int;

    fn exited(status: c_int) -> ChildState {
        ChildState::Exited {
            status: Some(status),
        }
    }

    #[test]
    fn a_normal_exit_carries_its_code_in_the_second_byte() {
        assert_eq!(decode(0x0000), exited(0));
        assert_eq!(decode(0x0100), exited(1));
        assert_eq!(decode(0x7a00), exited(122));
        assert_eq!(decode(0xff00), exited(255));
    }

    #[test]
    fn a_signalled_death_reports_128_plus_the_signal() {
        assert_eq!(decode(9), exited(137));
        assert_eq!(decode(15), exited(143));
        // The core-dumped bit does not change the answer.
        assert_eq!(decode(0x80 | 11), exited(139));
    }

    #[test]
    fn a_stopped_child_is_not_an_exit() {
        // SIGTSTP (20) and SIGSTOP (19) delivered as stops.
        assert_eq!(decode((20 << 8) | 0x7f), ChildState::Stopped);
        assert_eq!(decode(0x137f), ChildState::Stopped);
    }

    #[test]
    fn a_continued_child_has_its_own_reserved_word() {
        assert_eq!(decode(0xffff), ChildState::Continued);
    }

    #[test]
    fn the_signalled_test_excludes_both_ends_of_the_range() {
        // Zero is a normal exit and 0x7f is the stop marker; the signed-byte
        // truncation is what keeps the latter out.
        assert!(!is_signalled(0));
        assert!(!is_signalled(0x7f));
        assert!(is_signalled(1));
        assert!(is_signalled(0x7e));
    }
}
