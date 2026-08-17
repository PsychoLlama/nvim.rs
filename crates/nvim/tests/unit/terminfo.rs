//! Built-in terminal descriptions and the parameterised-string interpreter.
//!
//! The descriptions are contract: they decide what nvim writes to a terminal
//! the system's terminfo database cannot describe, and they were generated
//! from Dickey's terminfo.src. The checksum below was computed from
//! `v0.12.4`'s `terminfo_builtin.h`, so it locks the port to the C source
//! capability for capability rather than merely to itself.

use c2rust_neovim::tui::terminfo::builtin::{DESCRIPTIONS, Description, from_term};
use c2rust_neovim::tui::terminfo::caps::{
    EXT_CAPS, KEYS, MAX_FUNCTION_KEY, STRING_CAPS, kTermCount, key_slot,
};
use c2rust_neovim::tui::terminfo::param::{Out, Param, expand};
use std::ffi::CStr;

/// Expand a description's sparse capability lists into the dense slot arrays
/// the entry it produces has.
fn dense<T: Copy>(len: usize, filled: impl Iterator<Item = (usize, T)>) -> Vec<Option<T>> {
    let mut slots = vec![None; len];
    for (slot, value) in filled {
        slots[slot] = Some(value);
    }
    slots
}

struct Fnv(u64);

impl Fnv {
    fn update(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
        }
    }

    /// A present capability hashes as its bytes and a NUL; an absent one as a
    /// byte that cannot start a sequence.
    fn capability(&mut self, seq: Option<&CStr>) {
        match seq {
            Some(seq) => {
                self.update(seq.to_bytes());
                self.update(&[0]);
            }
            None => self.update(&[0xff]),
        }
    }
}

#[test]
fn builtin_descriptions_match_the_generated_table() {
    let mut fnv = Fnv(0xcbf2_9ce4_8422_2325);
    for (name, description) in DESCRIPTIONS {
        fnv.update(name.as_bytes());
        fnv.update(&[0]);
        fnv.update(&[
            u8::from(description.bce),
            u8::from(description.has_tc_or_rgb),
            u8::from(description.su),
        ]);
        for number in [
            description.max_colors,
            description.lines,
            description.columns,
        ] {
            fnv.update(&(number as u32).to_le_bytes());
        }

        let defs = dense(
            kTermCount as usize,
            description
                .defs
                .iter()
                .map(|&(slot, seq)| (slot as usize, seq)),
        );
        for (slot, seq) in defs.iter().enumerate() {
            fnv.update(&[slot as u8]);
            fnv.capability(*seq);
        }

        let keys = dense(
            KEYS.len(),
            description
                .keys
                .iter()
                .map(|&(slot, seq, shifted)| (slot, (seq, shifted))),
        );
        for (slot, pair) in keys.iter().enumerate() {
            fnv.update(&[slot as u8]);
            fnv.capability(pair.map(|(seq, _)| seq));
            fnv.capability(pair.and_then(|(_, shifted)| shifted));
        }

        for slot in 0..MAX_FUNCTION_KEY {
            fnv.update(&[slot as u8]);
            fnv.capability(description.f_keys.get(slot).copied().flatten());
        }
    }
    assert_eq!(fnv.0, 0x4ec1_0bcc_c4de_7dbb);
}

/// Spot checks in terms the capability names make readable, so a checksum
/// failure has something to be diffed against.
#[test]
fn well_known_capabilities_are_where_they_should_be() {
    let cap = |term: &CStr, name: &str| -> Option<&'static CStr> {
        let slot = STRING_CAPS
            .iter()
            .position(|cap| cap.name == name)
            .or_else(|| {
                EXT_CAPS
                    .iter()
                    .position(|cap| cap.name == name)
                    .map(|i| STRING_CAPS.len() + i)
            })
            .unwrap_or_else(|| panic!("no capability {name}"));
        from_term(Some(term))
            .1
            .defs
            .iter()
            .find(|&&(s, _)| s as usize == slot)
            .map(|&(_, seq)| seq)
    };

    assert_eq!(
        cap(c"xterm-256color", "cursor_address").unwrap(),
        c"\x1b[%i%p1%d;%p2%dH"
    );
    assert_eq!(
        cap(c"xterm-256color", "set_cursor_style").unwrap(),
        c"\x1b[%p1%d q"
    );
    assert_eq!(cap(c"linux", "clear_screen").unwrap(), c"\x1b[H\x1b[J");
    // The `ansi` fallback has no alternate screen and no italics.
    assert_eq!(cap(c"nothing-like-this", "enter_ca_mode"), None);
    assert_eq!(cap(c"nothing-like-this", "enter_italics_mode"), None);
    assert_eq!(cap(c"nothing-like-this", "carriage_return").unwrap(), c"\r");
}

#[test]
fn descriptions_agree_with_their_key_slots() {
    let (_, xterm) = from_term(Some(c"xterm"));
    let key = |slot: usize| {
        xterm
            .keys
            .iter()
            .find(|&&(s, _, _)| s == slot)
            .map(|&(_, seq, shifted)| (seq, shifted))
    };
    assert_eq!(key(key_slot::LEFT), Some((c"\x1bOD", Some(c"\x1b[1;2D"))));
    assert_eq!(key(key_slot::RIGHT), Some((c"\x1bOC", Some(c"\x1b[1;2C"))));
    assert_eq!(key(key_slot::BACKSPACE), Some((c"\x08", None)));
    // F1 is the first function key, not the zeroth.
    assert_eq!(xterm.f_keys[0], Some(c"\x1bOP"));
}

/// Every description has to survive being used: no slot out of range, no
/// capability that the interpreter refuses.
#[test]
fn every_capability_expands() {
    for (name, description) in DESCRIPTIONS {
        for &(slot, seq) in description.defs {
            let mut params = [Param::default(); 9];
            for (i, param) in params.iter_mut().enumerate() {
                param.num = i as i64 + 1;
            }
            let mut buf = [0u8; 1024];
            let mut out = Out::new(&mut buf);
            assert!(
                expand(seq.to_bytes(), &mut params, &mut out),
                "{name}: slot {slot} would not expand"
            );
        }
    }
}

/// A handful of the descriptions' own capabilities, expanded.
#[test]
fn expansion_of_real_capabilities() {
    let run = |cap: &CStr, nums: &[i64]| {
        let mut params = [Param::default(); 9];
        for (param, &num) in params.iter_mut().zip(nums) {
            param.num = num;
        }
        let mut buf = [0u8; 256];
        let mut out = Out::new(&mut buf);
        assert!(expand(cap.to_bytes(), &mut params, &mut out));
        let len = out.len();
        String::from_utf8(buf[..len].to_vec()).unwrap()
    };

    assert_eq!(run(c"\x1b[%i%p1%d;%p2%dH", &[0, 0]), "\x1b[1;1H");
    assert_eq!(run(c"\x1b[%i%p1%d;%p2%dH", &[23, 79]), "\x1b[24;80H");
    assert_eq!(run(c"\x1b[%p1%dX", &[5]), "\x1b[5X");
    assert_eq!(run(c"\x1b[%p1%d q", &[2]), "\x1b[2 q");
    // xterm's `set_a_foreground`, which picks one of three encodings.
    let setaf = c"\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m";
    assert_eq!(run(setaf, &[3]), "\x1b[33m");
    assert_eq!(run(setaf, &[9]), "\x1b[91m");
    assert_eq!(run(setaf, &[200]), "\x1b[38;5;200m");
}

/// The interpreter refuses rather than truncating, which is what lets the TUI
/// flush and retry into an empty buffer.
#[test]
fn expansion_that_does_not_fit_fails() {
    let mut params = [Param::default(); 9];
    params[0].num = 10;
    let mut buf = [0u8; 8];
    let mut out = Out::new(&mut buf);
    assert!(!expand(b"\x1b[%i%p1%d;%p2%dH", &mut params, &mut out));
}

/// Descriptions are static data; asking twice hands back the same one.
#[test]
fn lookup_is_stable() {
    let first: &Description = from_term(Some(c"xterm-256color")).1;
    let second: &Description = from_term(Some(c"xterm-kitty")).1;
    assert!(std::ptr::eq(first, second));
}
