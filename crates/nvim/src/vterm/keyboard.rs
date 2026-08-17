//! Keyboard input: turning key presses into the bytes the host expects.
//!
//! Three things decide the spelling of a key. The terminal modes DECCKM,
//! DECKPAM and LNM pick between the legacy cursor/keypad/Enter forms; whether
//! the host accepts 8-bit C1 controls picks between `CSI` and `ESC [`; and the
//! Kitty keyboard protocol's disambiguation level, when the host has pushed it
//! onto the key-encoding stack, replaces almost everything with `CSI <code>;<mod>u`.
//!
//! The four entry points keep their C ABI — the unit specs call them through
//! LuaJIT's FFI — but the encoding itself is pure and pointer-free.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::types::{VTerm, VTermKey, VTermKeyEncodingFlags, VTermModifier, VTermState, uint32_t};
use crate::vterm::output::EscapeSeq;
use crate::vterm::vterm::{
    VTERM_KEY_BACKSPACE, VTERM_KEY_ENTER, VTERM_KEY_FUNCTION_0, VTERM_KEY_FUNCTION_MAX,
    VTERM_KEY_KP_0, VTERM_KEY_NONE, VTERM_KEY_TAB, VTERM_MOD_ALT, VTERM_MOD_CTRL, VTERM_MOD_NONE,
    VTERM_MOD_SHIFT, vterm_push_output_bytes,
};
use core::ffi::{c_char, c_int};
use core::fmt::Write;

/// How a special key is spelled on the wire.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum KeyForm {
    /// No key at this slot.
    Absent,
    /// The literal byte in `literal`.
    Literal,
    /// Literal, except that Shift-Tab has a CSI form of its own.
    Tab,
    /// Literal, unless LNM asked for CR LF.
    Enter,
    /// `SS3 <literal>`, or the CSI form once modifiers are involved.
    Ss3,
    /// `CSI <literal>`.
    Csi,
    /// SS3 or CSI, depending on DECCKM.
    CsiCursor,
    /// `CSI <csinum> <literal>`, where `literal` is the final `~`.
    CsiNum,
    /// Literal, or the application-keypad form, depending on DECKPAM.
    Keypad,
}

#[derive(Clone, Copy)]
struct Keycode {
    form: KeyForm,
    literal: c_int,
    csinum: c_int,
}

const fn key(form: KeyForm, literal: u8, csinum: c_int) -> Keycode {
    Keycode {
        form,
        literal: literal as c_int,
        csinum,
    }
}

/// Keys below `VTERM_KEY_FUNCTION_0`, indexed by their `VTermKey`.
const KEYCODES: [Keycode; 15] = [
    key(KeyForm::Absent, 0, 0),
    key(KeyForm::Enter, b'\r', 0),
    key(KeyForm::Tab, b'\t', 0),
    key(KeyForm::Literal, 0x7f, 0),
    key(KeyForm::Literal, 0x1b, 0),
    key(KeyForm::CsiCursor, b'A', 0),
    key(KeyForm::CsiCursor, b'B', 0),
    key(KeyForm::CsiCursor, b'D', 0),
    key(KeyForm::CsiCursor, b'C', 0),
    key(KeyForm::CsiNum, b'~', 2),
    key(KeyForm::CsiNum, b'~', 3),
    key(KeyForm::CsiCursor, b'H', 0),
    key(KeyForm::CsiCursor, b'F', 0),
    key(KeyForm::CsiNum, b'~', 5),
    key(KeyForm::CsiNum, b'~', 6),
];

/// Function keys, indexed by `VTermKey - VTERM_KEY_FUNCTION_0`. F1-F4 keep
/// the VT100 SS3 forms; the rest are numbered, with gaps where DEC left them.
const FUNCTION_KEYCODES: [Keycode; 13] = [
    key(KeyForm::Absent, 0, 0),
    key(KeyForm::Ss3, b'P', 0),
    key(KeyForm::Ss3, b'Q', 0),
    key(KeyForm::Ss3, b'R', 0),
    key(KeyForm::Ss3, b'S', 0),
    key(KeyForm::CsiNum, b'~', 15),
    key(KeyForm::CsiNum, b'~', 17),
    key(KeyForm::CsiNum, b'~', 18),
    key(KeyForm::CsiNum, b'~', 19),
    key(KeyForm::CsiNum, b'~', 20),
    key(KeyForm::CsiNum, b'~', 21),
    key(KeyForm::CsiNum, b'~', 23),
    key(KeyForm::CsiNum, b'~', 24),
];

/// Keypad keys, indexed by `VTermKey - VTERM_KEY_KP_0`: the byte the numeric
/// keypad sends, the private-use codepoint the disambiguating protocol reports
/// instead, and the final byte of the application-keypad form.
const KEYPAD: [(u8, c_int, u8); 18] = [
    (b'0', 57399, b'p'),
    (b'1', 57400, b'q'),
    (b'2', 57401, b'r'),
    (b'3', 57402, b's'),
    (b'4', 57403, b't'),
    (b'5', 57404, b'u'),
    (b'6', 57405, b'v'),
    (b'7', 57406, b'w'),
    (b'8', 57407, b'x'),
    (b'9', 57408, b'y'),
    (b'*', 57411, b'j'),
    (b'+', 57413, b'k'),
    (b',', 57416, b'l'),
    (b'-', 57412, b'm'),
    (b'.', 57409, b'n'),
    (b'/', 57410, b'o'),
    (b'\n', 57414, b'M'),
    (b'=', 57415, b'X'),
];

/// Everything about the terminal's current modes that changes how a key is
/// spelled.
#[derive(Clone, Copy)]
struct KeyModes {
    /// DECCKM: cursor keys send SS3 rather than CSI.
    cursor: bool,
    /// DECKPAM: the keypad sends its application-mode codes.
    keypad: bool,
    /// LNM: Enter sends CR LF.
    newline: bool,
    /// The host accepts 8-bit C1 controls.
    ctrl8bit: bool,
    /// The Kitty keyboard protocol's disambiguation level is in effect.
    disambiguate: bool,
}

impl KeyModes {
    fn read(vt: &VTerm, state: &VTermState) -> Self {
        KeyModes {
            cursor: state.mode.cursor() != 0,
            keypad: state.mode.keypad() != 0,
            newline: state.mode.newline() != 0,
            ctrl8bit: vt.mode.ctrl8bit() != 0,
            disambiguate: key_encoding_flags(state).disambiguate(),
        }
    }
}

/// The key-encoding flags on top of the stack for the screen in use.
///
/// Each screen keeps its own stack so that a full-screen program's push is
/// undone when it switches back, and each stack always has a base entry.
fn key_encoding_flags(state: &VTermState) -> VTermKeyEncodingFlags {
    let stack = &state.key_encoding_stacks[state.mode.alt_screen() as usize];
    debug_assert!(stack.size > 0, "key-encoding stack lost its base entry");
    stack.items[usize::from(stack.size) - 1]
}

/// The keycode table entry for `key`, or `None` if it falls outside every
/// table.
fn lookup(key: VTermKey, disambiguate: bool) -> Option<Keycode> {
    if key < VTERM_KEY_FUNCTION_0 {
        KEYCODES.get(key as usize).copied()
    } else if key <= VTERM_KEY_FUNCTION_MAX {
        FUNCTION_KEYCODES
            .get((key - VTERM_KEY_FUNCTION_0) as usize)
            .copied()
    } else {
        KEYPAD
            .get((key - VTERM_KEY_KP_0) as usize)
            .map(|&(legacy, disambiguated, csinum)| Keycode {
                form: KeyForm::Keypad,
                literal: if disambiguate {
                    disambiguated
                } else {
                    c_int::from(legacy)
                },
                csinum: c_int::from(csinum),
            })
    }
}

/// `CSI <params> <literal>`, the fallback once a key carries modifiers.
fn csi_form(k: Keycode, mod_0: VTermModifier, ctrl8bit: bool) -> EscapeSeq {
    let mut seq = EscapeSeq::csi(ctrl8bit);
    if mod_0 != 0 {
        let _ = write!(seq, "1;{}", mod_0 + 1);
    }
    seq.push(k.literal as u8);
    seq
}

/// The literal spelling: the key's own byte, prefixed with ESC for Alt, or
/// the disambiguating protocol's `CSI <code>;<mod>u`.
fn literal_form(key: VTermKey, k: Keycode, mod_0: VTermModifier, modes: KeyModes) -> EscapeSeq {
    // Enter, Tab and Backspace keep their legacy bytes when unmodified even
    // under the disambiguating protocol, so that a bare Return still ends a
    // line for programs that never asked for the new encoding.
    let disambiguate = modes.disambiguate
        && (!matches!(key, VTERM_KEY_ENTER | VTERM_KEY_TAB | VTERM_KEY_BACKSPACE)
            || mod_0 != VTERM_MOD_NONE);

    if disambiguate {
        let mut seq = EscapeSeq::csi(modes.ctrl8bit);
        let _ = write!(seq, "{};{}u", k.literal, mod_0 + 1);
        seq
    } else {
        let mut seq = EscapeSeq::new();
        if mod_0 & VTERM_MOD_ALT != 0 {
            seq.push(0x1b);
        }
        seq.push(k.literal as u8);
        seq
    }
}

/// Spell one special key, or `None` when there is nothing to send.
fn encode_key(key: VTermKey, mod_0: VTermModifier, modes: KeyModes) -> Option<EscapeSeq> {
    let mut k = lookup(key, modes.disambiguate)?;

    // Resolve the mode-dependent forms down to Literal, Csi or Ss3.
    let form = match k.form {
        KeyForm::Absent => return None,
        KeyForm::Tab if !modes.disambiguate && mod_0 & VTERM_MOD_SHIFT != 0 => {
            // Shift-Tab is CSI Z, which takes its modifiers in the leading
            // parameter rather than the usual trailing one.
            let mut seq = EscapeSeq::csi(modes.ctrl8bit);
            if mod_0 != VTERM_MOD_SHIFT {
                let _ = write!(seq, "1;{}", mod_0 + 1);
            }
            seq.push(b'Z');
            return Some(seq);
        }
        KeyForm::Enter if modes.newline => {
            let mut seq = EscapeSeq::new();
            seq.extend(b"\r\n");
            return Some(seq);
        }
        KeyForm::CsiNum => {
            let mut seq = EscapeSeq::csi(modes.ctrl8bit);
            if mod_0 == 0 {
                let _ = write!(seq, "{}", k.csinum);
            } else {
                let _ = write!(seq, "{};{}", k.csinum, mod_0 + 1);
            }
            seq.push(k.literal as u8);
            return Some(seq);
        }
        KeyForm::Tab | KeyForm::Enter | KeyForm::Literal => KeyForm::Literal,
        KeyForm::Ss3 => KeyForm::Ss3,
        KeyForm::Csi => KeyForm::Csi,
        KeyForm::CsiCursor if modes.cursor => KeyForm::Ss3,
        KeyForm::CsiCursor => KeyForm::Csi,
        KeyForm::Keypad if modes.keypad => {
            k.literal = k.csinum;
            KeyForm::Ss3
        }
        KeyForm::Keypad => KeyForm::Literal,
    };

    Some(match form {
        // SS3 has no room for parameters, so a modified key falls back to CSI.
        KeyForm::Ss3 if mod_0 == 0 => {
            let mut seq = EscapeSeq::ss3(modes.ctrl8bit);
            seq.push(k.literal as u8);
            seq
        }
        KeyForm::Ss3 | KeyForm::Csi => csi_form(k, mod_0, modes.ctrl8bit),
        _ => literal_form(key, k, mod_0, modes),
    })
}

/// The byte Ctrl+`c` sends, for the digits and punctuation the DEC keyboard
/// gave control codes to.
fn ctrl_fold(c: u32) -> u32 {
    match c {
        // Ctrl-Space and Ctrl-2 are both NUL.
        0x20 | 0x32 => 0,
        // Ctrl-3 through Ctrl-7 walk ESC, FS, GS, RS, US.
        0x33..=0x37 => 0x1b + c - 0x33,
        // Ctrl-8 is DEL, Ctrl-/ is US.
        0x38 => 0x7f,
        0x2f => 0x1f,
        // Everything from '@' up loses its top two bits, the usual rule.
        0x40..=0x7f => c & 0x1f,
        _ => c,
    }
}

/// Spell one ordinary character keypress.
fn encode_unichar(c: u32, mod_0: VTermModifier, modes: KeyModes) -> Option<EscapeSeq> {
    // A character with no modifier beyond Shift is already the character the
    // host wants. Space is the exception: Shift-Space has to stay
    // distinguishable from Ctrl-Space, which is NUL.
    let passthru = if c == u32::from(b' ') {
        mod_0 == VTERM_MOD_NONE
    } else {
        mod_0 & !VTERM_MOD_SHIFT == 0
    };
    if passthru {
        let mut seq = EscapeSeq::new();
        seq.push_utf8(c as i32);
        return Some(seq);
    }

    let mut c = c;
    let mut mod_0 = mod_0;
    if modes.disambiguate {
        if (u32::from(b'A')..=u32::from(b'Z')).contains(&c) {
            // CSI-u reports the unshifted key plus an explicit Shift.
            c += u32::from(b'a' - b'A');
            mod_0 |= VTERM_MOD_SHIFT;
        }
        let mut seq = EscapeSeq::csi(modes.ctrl8bit);
        let _ = write!(seq, "{};{}u", c, mod_0 + 1);
        return Some(seq);
    }

    if mod_0 & VTERM_MOD_CTRL != 0 {
        c = ctrl_fold(c);
    }
    // The legacy encoding has one byte for the character, so anything above
    // U+00FF is truncated exactly as the `%c` conversion used to truncate it.
    let mut seq = EscapeSeq::new();
    if mod_0 & VTERM_MOD_ALT != 0 {
        seq.push(0x1b);
    }
    seq.push(c as u8);
    Some(seq)
}

/// Writes a spelled key back to the host, if it spelled to anything.
///
/// # Safety
///
/// `vt` must point at a live terminal.
unsafe fn send(vt: *mut VTerm, report: Option<EscapeSeq>) {
    let Some(bytes) = report.as_ref().and_then(EscapeSeq::finish) else {
        return;
    };
    // SAFETY: forwarded to this function's own caller; `bytes` outlives the
    // call, which copies out of it.
    unsafe { vterm_push_output_bytes(vt, bytes.as_ptr().cast::<c_char>(), bytes.len()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_unichar(vt: *mut VTerm, c: uint32_t, mod_0: VTermModifier) {
    // SAFETY: the caller hands over a live terminal that has a state.
    let modes = unsafe { KeyModes::read(&*vt, &*(*vt).state) };
    // SAFETY: `vt` is that same live terminal.
    unsafe { send(vt, encode_unichar(c, mod_0, modes)) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_key(vt: *mut VTerm, key: VTermKey, mod_0: VTermModifier) {
    if key == VTERM_KEY_NONE {
        return;
    }
    // SAFETY: the caller hands over a live terminal that has a state.
    let modes = unsafe { KeyModes::read(&*vt, &*(*vt).state) };
    // SAFETY: `vt` is that same live terminal.
    unsafe { send(vt, encode_key(key, mod_0, modes)) };
}

/// The bracketed-paste brackets, sent only when the host turned the mode on.
fn paste_marker(state: &VTermState, ctrl8bit: bool, body: &[u8]) -> Option<EscapeSeq> {
    if state.mode.bracketpaste() == 0 {
        return None;
    }
    let mut seq = EscapeSeq::csi(ctrl8bit);
    seq.extend(body);
    Some(seq)
}

/// Writes one bracketed-paste marker back to the host.
///
/// # Safety
///
/// `vt` must point at a live terminal that has a state.
unsafe fn push_paste_marker(vt: *mut VTerm, body: &[u8]) {
    // SAFETY: forwarded to this function's own caller.
    let report = unsafe { paste_marker(&*(*vt).state, (*vt).mode.ctrl8bit() != 0, body) };
    // SAFETY: `vt` is that same live terminal.
    unsafe { send(vt, report) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_start_paste(vt: *mut VTerm) {
    // SAFETY: forwarded to this function's own caller.
    unsafe { push_paste_marker(vt, b"200~") };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_keyboard_end_paste(vt: *mut VTerm) {
    // SAFETY: forwarded to this function's own caller.
    unsafe { push_paste_marker(vt, b"201~") };
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY: KeyModes = KeyModes {
        cursor: false,
        keypad: false,
        newline: false,
        ctrl8bit: false,
        disambiguate: false,
    };

    fn key_bytes(key: VTermKey, mod_0: VTermModifier, modes: KeyModes) -> Vec<u8> {
        encode_key(key, mod_0, modes)
            .and_then(|seq| seq.finish().map(<[u8]>::to_vec))
            .unwrap_or_default()
    }

    fn char_bytes(c: u32, mod_0: VTermModifier, modes: KeyModes) -> Vec<u8> {
        encode_unichar(c, mod_0, modes)
            .and_then(|seq| seq.finish().map(<[u8]>::to_vec))
            .unwrap_or_default()
    }

    #[test]
    fn unknown_keys_send_nothing() {
        // The gap between the named keys and the function keys.
        assert!(encode_key(15, 0, LEGACY).is_none());
        assert!(encode_key(255, 0, LEGACY).is_none());
        // The function-key and keypad table bases are placeholders.
        assert!(encode_key(VTERM_KEY_FUNCTION_0, 0, LEGACY).is_none());
        assert!(encode_key(VTERM_KEY_KP_0 + 18, 0, LEGACY).is_none());
    }

    #[test]
    fn cursor_keys_follow_deccjm() {
        let app = KeyModes {
            cursor: true,
            ..LEGACY
        };
        assert_eq!(key_bytes(5, 0, LEGACY), b"\x1b[A");
        assert_eq!(key_bytes(5, 0, app), b"\x1bOA");
        // Modifiers have no room in SS3, so both modes fall back to CSI.
        assert_eq!(key_bytes(5, VTERM_MOD_SHIFT, app), b"\x1b[1;2A");
        assert_eq!(key_bytes(5, VTERM_MOD_CTRL, LEGACY), b"\x1b[1;5A");
    }

    #[test]
    fn numbered_keys_carry_their_modifier_after_the_number() {
        // Insert, Delete, PageUp, PageDown.
        assert_eq!(key_bytes(9, 0, LEGACY), b"\x1b[2~");
        assert_eq!(key_bytes(10, VTERM_MOD_SHIFT, LEGACY), b"\x1b[3;2~");
        assert_eq!(key_bytes(13, 0, LEGACY), b"\x1b[5~");
        // F1 is SS3, F5 is numbered.
        assert_eq!(key_bytes(VTERM_KEY_FUNCTION_0 + 1, 0, LEGACY), b"\x1bOP");
        assert_eq!(key_bytes(VTERM_KEY_FUNCTION_0 + 5, 0, LEGACY), b"\x1b[15~");
    }

    #[test]
    fn shift_tab_has_its_own_final_byte() {
        assert_eq!(key_bytes(VTERM_KEY_TAB, 0, LEGACY), b"\t");
        assert_eq!(key_bytes(VTERM_KEY_TAB, VTERM_MOD_SHIFT, LEGACY), b"\x1b[Z");
        assert_eq!(
            key_bytes(VTERM_KEY_TAB, VTERM_MOD_SHIFT | VTERM_MOD_ALT, LEGACY),
            b"\x1b[1;4Z"
        );
        // Alt alone still takes the literal path.
        assert_eq!(key_bytes(VTERM_KEY_TAB, VTERM_MOD_ALT, LEGACY), b"\x1b\t");
    }

    #[test]
    fn enter_follows_lnm() {
        assert_eq!(key_bytes(VTERM_KEY_ENTER, 0, LEGACY), b"\r");
        let lnm = KeyModes {
            newline: true,
            ..LEGACY
        };
        assert_eq!(key_bytes(VTERM_KEY_ENTER, 0, lnm), b"\r\n");
    }

    #[test]
    fn the_keypad_switches_between_digits_and_application_codes() {
        assert_eq!(key_bytes(VTERM_KEY_KP_0, 0, LEGACY), b"0");
        let app = KeyModes {
            keypad: true,
            ..LEGACY
        };
        assert_eq!(key_bytes(VTERM_KEY_KP_0, 0, app), b"\x1bOp");
        assert_eq!(
            key_bytes(VTERM_KEY_KP_0, VTERM_MOD_SHIFT, app),
            b"\x1b[1;2p"
        );
    }

    #[test]
    fn disambiguation_reports_keypad_codepoints() {
        let csiu = KeyModes {
            disambiguate: true,
            ..LEGACY
        };
        assert_eq!(key_bytes(VTERM_KEY_KP_0, 0, csiu), b"\x1b[57399;1u");
        // With DECKPAM the application code still wins over the codepoint.
        let both = KeyModes {
            keypad: true,
            ..csiu
        };
        assert_eq!(key_bytes(VTERM_KEY_KP_0, 0, both), b"\x1bOp");
    }

    #[test]
    fn disambiguation_leaves_the_three_legacy_keys_alone_when_unmodified() {
        let csiu = KeyModes {
            disambiguate: true,
            ..LEGACY
        };
        assert_eq!(key_bytes(VTERM_KEY_ENTER, 0, csiu), b"\r");
        assert_eq!(key_bytes(VTERM_KEY_TAB, 0, csiu), b"\t");
        assert_eq!(key_bytes(VTERM_KEY_BACKSPACE, 0, csiu), b"\x7f");
        assert_eq!(
            key_bytes(VTERM_KEY_ENTER, VTERM_MOD_CTRL, csiu),
            b"\x1b[13;5u"
        );
        // Escape has no such exemption.
        assert_eq!(key_bytes(4, 0, csiu), b"\x1b[27;1u");
    }

    #[test]
    fn eight_bit_hosts_get_bare_c1_controls() {
        let c1 = KeyModes {
            ctrl8bit: true,
            ..LEGACY
        };
        assert_eq!(key_bytes(5, 0, c1), b"\x9bA");
        assert_eq!(key_bytes(9, 0, c1), b"\x9b2~");
        let app = KeyModes { cursor: true, ..c1 };
        assert_eq!(key_bytes(5, 0, app), b"\x8fA");
    }

    #[test]
    fn characters_pass_through_unless_they_carry_a_modifier() {
        assert_eq!(char_bytes(u32::from(b'a'), 0, LEGACY), b"a");
        assert_eq!(char_bytes(u32::from(b'A'), VTERM_MOD_SHIFT, LEGACY), b"A");
        assert_eq!(char_bytes(0x20ac, 0, LEGACY), "€".as_bytes());
        assert_eq!(char_bytes(u32::from(b' '), 0, LEGACY), b" ");
        // Space is the one character whose Shift form is not a pass-through,
        // so that Ctrl-Space can reach the folding path below and become NUL.
        // It still ends up as a plain space.
        assert_eq!(char_bytes(u32::from(b' '), VTERM_MOD_SHIFT, LEGACY), b" ");
    }

    #[test]
    fn control_folds_the_dec_keyboard_way() {
        let ctrl = |c: u8| char_bytes(u32::from(c), VTERM_MOD_CTRL, LEGACY);
        assert_eq!(ctrl(b'a'), b"\x01");
        assert_eq!(ctrl(b'A'), b"\x01");
        assert_eq!(ctrl(b'2'), b"\0");
        assert_eq!(ctrl(b' '), b"\0");
        assert_eq!(ctrl(b'3'), b"\x1b");
        assert_eq!(ctrl(b'7'), b"\x1f");
        assert_eq!(ctrl(b'8'), b"\x7f");
        assert_eq!(ctrl(b'/'), b"\x1f");
        // A digit with no control code of its own is sent unchanged.
        assert_eq!(ctrl(b'1'), b"1");
    }

    #[test]
    fn alt_prefixes_escape() {
        assert_eq!(char_bytes(u32::from(b'a'), VTERM_MOD_ALT, LEGACY), b"\x1ba");
        assert_eq!(
            char_bytes(u32::from(b'a'), VTERM_MOD_ALT | VTERM_MOD_CTRL, LEGACY),
            b"\x1b\x01"
        );
    }

    #[test]
    fn disambiguation_reports_shift_explicitly() {
        let csiu = KeyModes {
            disambiguate: true,
            ..LEGACY
        };
        // An uppercase letter is reported as its key plus an explicit Shift.
        assert_eq!(
            char_bytes(u32::from(b'A'), VTERM_MOD_CTRL, csiu),
            b"\x1b[97;6u"
        );
        assert_eq!(
            char_bytes(u32::from(b'a'), VTERM_MOD_CTRL, csiu),
            b"\x1b[97;5u"
        );
        // Characters carrying nothing but Shift still pass through untouched,
        // so the protocol never sees them.
        assert_eq!(char_bytes(u32::from(b'a'), 0, csiu), b"a");
        assert_eq!(char_bytes(u32::from(b'A'), VTERM_MOD_SHIFT, csiu), b"A");
    }
}
