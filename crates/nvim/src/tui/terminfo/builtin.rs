//! The built-in terminal descriptions.
//!
//! nvim carries its own copy of a handful of terminfo entries, for the
//! systems that have no terminfo database, for a `$TERM` the database does
//! not know, and for the terminals whose shipped description is worse than
//! the one ncurses has. [`from_term`] is the whole interface: give it
//! `$TERM`, get back the name nvim will call the terminal and a description
//! to work from.
//!
//! The descriptions themselves live in the submodules, written the way a
//! terminal description is written -- only the capabilities the terminal
//! actually has, each one named. [`Description::entry`] expands one into the
//! dense [`TerminfoEntry`] the rest of the TUI indexes by slot.

#![forbid(unsafe_code)]

mod consoles;
mod emulators;
mod multiplexers;
mod windows;

use crate::tui::terminfo::caps::{MAX_FUNCTION_KEY, TerminfoDef, kTermCount};
use crate::tui::terminfo::is_term_family;
use crate::types::TerminfoEntry;
use core::ffi::{CStr, c_int};

/// A built-in terminal description: what the terminal can do, in the sparse
/// form it is written in.
pub struct Description {
    pub bce: bool,
    /// Advertises truecolour, through either the `Tc` or the `RGB` extension.
    pub has_tc_or_rgb: bool,
    /// Advertises the `Su` extension: underline styles beyond the plain one.
    pub su: bool,
    pub max_colors: c_int,
    pub lines: c_int,
    pub columns: c_int,
    /// The string capabilities the terminal has, by `defs` slot.
    pub defs: &'static [(TerminfoDef, &'static CStr)],
    /// The special keys the terminal sends, by `keys` slot: the sequence and
    /// its shifted variant.
    pub keys: &'static [(usize, &'static CStr, Option<&'static CStr>)],
    /// `key_f1` upward. Gaps are real -- one description skips a number.
    pub f_keys: &'static [Option<&'static CStr>],
}

impl Description {
    /// Expand to the dense entry the TUI reads.
    ///
    /// The sequences are pointers into this binary's read-only data, so the
    /// entry borrows nothing that can go away; the TUI copies it into its own
    /// state and may then patch individual slots.
    pub fn entry(&self) -> TerminfoEntry {
        let mut entry = TerminfoEntry {
            bce: self.bce,
            has_Tc_or_RGB: self.has_tc_or_rgb,
            Su: self.su,
            max_colors: self.max_colors,
            lines: self.lines,
            columns: self.columns,
            defs: [core::ptr::null(); kTermCount as usize],
            keys: [[core::ptr::null(); 2]; 16],
            f_keys: [core::ptr::null(); MAX_FUNCTION_KEY],
        };
        for &(slot, seq) in self.defs {
            entry.defs[slot as usize] = seq.as_ptr();
        }
        for &(slot, seq, shifted) in self.keys {
            entry.keys[slot][0] = seq.as_ptr();
            entry.keys[slot][1] = shifted.map_or(core::ptr::null(), CStr::as_ptr);
        }
        for (i, seq) in self.f_keys.iter().enumerate() {
            entry.f_keys[i] = seq.map_or(core::ptr::null(), CStr::as_ptr);
        }
        entry
    }
}

/// Pick a built-in description for `$TERM` (absent when nvim has no `$TERM`
/// at all), returning the name nvim will report for the terminal alongside
/// it.
///
/// The order is upstream's, and it matters: `xterm` is tested before the
/// terminals whose `$TERM` merely starts with something xterm-like would
/// match, and everything unrecognised lands on `ansi`.
pub fn from_term(term: Option<&CStr>) -> (&'static CStr, &'static Description) {
    let name = term.map(CStr::to_bytes).unwrap_or(b"");
    let family = |f: &[u8]| is_term_family(name, f);
    if name == b"ghostty" || name == b"xterm-ghostty" {
        (c"ghostty", &emulators::GHOSTTY)
    } else if family(b"xterm") {
        (c"xterm", &emulators::XTERM_256COLOUR)
    } else if family(b"screen") {
        (c"screen", &multiplexers::SCREEN_256COLOUR)
    } else if family(b"tmux") {
        (c"tmux", &multiplexers::TMUX_256COLOUR)
    } else if family(b"rxvt") {
        (c"rxvt", &emulators::RXVT_256COLOUR)
    } else if family(b"putty") {
        (c"putty", &emulators::PUTTY_256COLOUR)
    } else if family(b"linux") {
        (c"linux", &consoles::LINUX_16COLOUR)
    } else if family(b"interix") {
        (c"interix", &consoles::INTERIX_8COLOUR)
    } else if family(b"iterm") || family(b"iterm2") || family(b"iTerm.app") || family(b"iTerm2.app")
    {
        (c"iterm", &emulators::ITERM_256COLOUR)
    } else if family(b"st") {
        (c"st", &emulators::ST_256COLOUR)
    } else if family(b"gnome") || family(b"vte") {
        (c"vte", &emulators::VTE_256COLOUR)
    } else if family(b"cygwin") {
        (c"cygwin", &windows::CYGWIN)
    } else if family(b"win32con") {
        (c"win32con", &windows::WIN32CON)
    } else if family(b"conemu") {
        (c"conemu", &windows::CONEMU)
    } else if family(b"vtpcon") {
        (c"vtpcon", &windows::VTPCON)
    } else {
        (c"ansi", &consoles::ANSI)
    }
}

/// Every built-in description, in the order the generator emits them, under
/// the name of the terminfo entry it was compiled from. Ordered and named for
/// `tests/unit/terminfo.rs`, which checksums the lot.
pub const DESCRIPTIONS: [(&str, &Description); 16] = [
    ("ansi", &consoles::ANSI),
    ("ghostty", &emulators::GHOSTTY),
    ("interix_8colour", &consoles::INTERIX_8COLOUR),
    ("iterm_256colour", &emulators::ITERM_256COLOUR),
    ("linux_16colour", &consoles::LINUX_16COLOUR),
    ("putty_256colour", &emulators::PUTTY_256COLOUR),
    ("rxvt_256colour", &emulators::RXVT_256COLOUR),
    ("screen_256colour", &multiplexers::SCREEN_256COLOUR),
    ("st_256colour", &emulators::ST_256COLOUR),
    ("tmux_256colour", &multiplexers::TMUX_256COLOUR),
    ("vte_256colour", &emulators::VTE_256COLOUR),
    ("xterm_256colour", &emulators::XTERM_256COLOUR),
    ("cygwin", &windows::CYGWIN),
    ("win32con", &windows::WIN32CON),
    ("conemu", &windows::CONEMU),
    ("vtpcon", &windows::VTPCON),
];

/// A slot's expansion, identified by the literal it must have come from --
/// the entry holds bare pointers, and this module may not dereference them.
#[cfg(test)]
fn holds(slot: *const core::ffi::c_char, want: Option<&'static CStr>) -> bool {
    slot == want.map_or(core::ptr::null(), CStr::as_ptr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::terminfo::caps::{KEYS, STRING_CAPS, key_slot};

    #[test]
    fn descriptions_stay_inside_their_arrays() {
        for (name, description) in DESCRIPTIONS {
            for &(slot, _) in description.defs {
                assert!(slot < kTermCount, "{name}: def slot {slot}");
            }
            for &(slot, _, _) in description.keys {
                assert!(slot < KEYS.len(), "{name}: key slot {slot}");
            }
            assert!(
                description.f_keys.len() <= MAX_FUNCTION_KEY,
                "{name}: {} function keys",
                description.f_keys.len()
            );
        }
    }

    /// Each description's slots are written in ascending order and never
    /// repeat, which is how a duplicated or misplaced capability shows up.
    #[test]
    fn capabilities_are_written_in_slot_order() {
        for (name, description) in DESCRIPTIONS {
            assert!(
                description.defs.windows(2).all(|w| w[0].0 < w[1].0),
                "{name}: defs out of order"
            );
            assert!(
                description.keys.windows(2).all(|w| w[0].0 < w[1].0),
                "{name}: keys out of order"
            );
        }
    }

    #[test]
    fn expansion_puts_sequences_in_their_named_slots() {
        let xterm = from_term(Some(c"xterm-256color"));
        assert_eq!(xterm.0, c"xterm");
        let entry = xterm.1.entry();
        assert!(
            holds(entry.defs[STRING_CAPS.len() - 1], None),
            "to_status_line"
        );
        assert!(holds(entry.defs[0], Some(c"\r")), "carriage_return");
        assert!(holds(entry.keys[key_slot::LEFT][0], Some(c"\x1bOD")));
        assert!(holds(entry.keys[key_slot::LEFT][1], Some(c"\x1b[1;2D")));
        assert!(holds(entry.f_keys[0], Some(c"\x1bOP")));
        assert!(entry.bce);
        assert_eq!(entry.max_colors, 256);
    }

    /// The `$TERM` matching upstream does, including the cases that are not
    /// simple family prefixes.
    #[test]
    fn term_names_pick_the_expected_description() {
        for (term, want) in [
            ("ghostty", "ghostty"),
            ("xterm-ghostty", "ghostty"),
            ("xterm", "xterm"),
            ("xterm-256color", "xterm"),
            ("screen.xterm-256color", "screen"),
            ("tmux-256color", "tmux"),
            ("rxvt-unicode-256color", "rxvt"),
            ("putty-256color", "putty"),
            ("linux", "linux"),
            ("interix", "interix"),
            ("iTerm2.app", "iterm"),
            ("st-256color", "st"),
            ("gnome-256color", "vte"),
            ("vte", "vte"),
            ("cygwin", "cygwin"),
            ("vtpcon", "vtpcon"),
            ("wezterm", "ansi"),
            ("", "ansi"),
        ] {
            let name = from_term(Some(&std::ffi::CString::new(term).unwrap())).0;
            assert_eq!(name.to_str().unwrap(), want, "$TERM={term}");
        }
        assert_eq!(from_term(None).0, c"ansi");
    }

    /// A description with no `$TERM` at all still expands, and `ansi` is what
    /// nvim falls back to.
    #[test]
    fn the_fallback_is_usable() {
        let entry = from_term(None).1.entry();
        assert!(holds(entry.defs[0], Some(c"\r")));
        assert!(holds(entry.defs[kTermCount as usize - 1], None));
        assert_eq!(entry.max_colors, 8);
    }
}
