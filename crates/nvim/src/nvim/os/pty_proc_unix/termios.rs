//! The line discipline a `:terminal` child is handed.
//!
//! Upstream took these settings from pangoterm. They describe a plain
//! cooked-mode terminal: CR translated to NL on input, output post-processing
//! on, canonical input with echo and signal generation, and the usual control
//! characters. The child is free to change all of it (a shell running a
//! full-screen program will), so this is only the state it starts in.
#![forbid(unsafe_code)]

use crate::src::nvim::types::{cc_t, tcflag_t, termios};

// Input modes.
pub const ICRNL: tcflag_t = 0o400;
pub const IXON: tcflag_t = 0o2000;
pub const IUTF8: tcflag_t = 0o40000;

// Output modes.
pub const OPOST: tcflag_t = 0o1;
pub const ONLCR: tcflag_t = 0o4;

// Control modes.
pub const CS8: tcflag_t = 0o60;
pub const CREAD: tcflag_t = 0o200;

// Local modes.
pub const ISIG: tcflag_t = 0o1;
pub const ICANON: tcflag_t = 0o2;
pub const ECHO: tcflag_t = 0o10;
pub const ECHOE: tcflag_t = 0o20;
pub const ECHOK: tcflag_t = 0o40;
pub const ECHOCTL: tcflag_t = 0o1000;
pub const ECHOKE: tcflag_t = 0o4000;
pub const IEXTEN: tcflag_t = 0o100000;

// Indices into `c_cc`.
pub const VINTR: usize = 0;
pub const VQUIT: usize = 1;
pub const VERASE: usize = 2;
pub const VKILL: usize = 3;
pub const VEOF: usize = 4;
pub const VTIME: usize = 5;
pub const VMIN: usize = 6;
pub const VSTART: usize = 8;
pub const VSTOP: usize = 9;
pub const VSUSP: usize = 10;
pub const VEOL: usize = 11;
pub const VREPRINT: usize = 12;
pub const VWERASE: usize = 14;
pub const VLNEXT: usize = 15;
pub const VEOL2: usize = 16;

/// The control character `ch` names, e.g. `ctrl(b'C')` for `^C`.
const fn ctrl(ch: u8) -> cc_t {
    (0x1f & ch) as cc_t
}

/// The settings a freshly forked pty child starts with.
///
/// The speeds are *not* set here. Upstream calls `cfsetispeed`/`cfsetospeed`
/// after this point; see `pty_proc_spawn` for why those are no-ops.
///
/// Upstream also ORs `TAB0`, `NL0`, `CR0`, `BS0`, `VT0` and `FF0` into
/// `c_oflag`; every one of them is the zero member of its own mask, so they
/// are named for intent and change nothing.
pub fn default_termios() -> termios {
    /// The value that disables a control character.
    const POSIX_VDISABLE: cc_t = 0;

    let mut cc = [0 as cc_t; 32];
    cc[VINTR] = ctrl(b'C');
    cc[VQUIT] = ctrl(b'\\');
    cc[VERASE] = 0x7f;
    cc[VKILL] = ctrl(b'U');
    cc[VEOF] = ctrl(b'D');
    cc[VEOL] = POSIX_VDISABLE;
    cc[VEOL2] = POSIX_VDISABLE;
    cc[VSTART] = ctrl(b'Q');
    cc[VSTOP] = ctrl(b'S');
    cc[VSUSP] = ctrl(b'Z');
    cc[VREPRINT] = ctrl(b'R');
    cc[VWERASE] = ctrl(b'W');
    cc[VLNEXT] = ctrl(b'V');
    cc[VMIN] = 1;
    cc[VTIME] = 0;

    termios {
        c_iflag: ICRNL | IXON | IUTF8,
        c_oflag: OPOST | ONLCR,
        c_cflag: CS8 | CREAD,
        c_lflag: ISIG | ICANON | IEXTEN | ECHO | ECHOE | ECHOK | ECHOCTL | ECHOKE,
        c_line: 0,
        c_cc: cc,
        c_ispeed: 0,
        c_ospeed: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_control_characters_are_the_usual_ones() {
        let t = default_termios();
        assert_eq!(t.c_cc[VINTR], 0x03);
        assert_eq!(t.c_cc[VQUIT], 0x1c);
        assert_eq!(t.c_cc[VERASE], 0x7f);
        assert_eq!(t.c_cc[VKILL], 0x15);
        assert_eq!(t.c_cc[VEOF], 0x04);
        assert_eq!(t.c_cc[VSTART], 0x11);
        assert_eq!(t.c_cc[VSTOP], 0x13);
        assert_eq!(t.c_cc[VSUSP], 0x1a);
        assert_eq!(t.c_cc[VREPRINT], 0x12);
        assert_eq!(t.c_cc[VWERASE], 0x17);
        assert_eq!(t.c_cc[VLNEXT], 0x16);
    }

    #[test]
    fn end_of_line_is_disabled_and_reads_are_unbuffered() {
        let t = default_termios();
        assert_eq!(t.c_cc[VEOL], 0);
        assert_eq!(t.c_cc[VEOL2], 0);
        assert_eq!(t.c_cc[VMIN], 1);
        assert_eq!(t.c_cc[VTIME], 0);
    }

    #[test]
    fn the_line_is_cooked_and_echoing() {
        let t = default_termios();
        assert_eq!(t.c_iflag, 0o40000 | 0o2000 | 0o400);
        assert_eq!(t.c_oflag, 0o5);
        assert_eq!(t.c_cflag, 0o260);
        assert_eq!(t.c_lflag, 0o105073);
    }

    #[test]
    fn nothing_outside_the_named_slots_is_set() {
        let t = default_termios();
        let named = [
            VINTR, VQUIT, VERASE, VKILL, VEOF, VTIME, VMIN, VSTART, VSTOP, VSUSP, VEOL, VREPRINT,
            VWERASE, VLNEXT, VEOL2,
        ];
        for (i, &value) in t.c_cc.iter().enumerate() {
            if !named.contains(&i) {
                assert_eq!(value, 0, "c_cc[{i}]");
            }
        }
    }
}
