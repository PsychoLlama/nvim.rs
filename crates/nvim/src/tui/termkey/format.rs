#![forbid(unsafe_code)]

//! Rendering a key as text, for `termkey_strfkey`.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

use crate::tui::termkey::keynames;
use crate::tui::termkey::termkey::{
    TERMKEY_FORMAT_ALTISMETA, TERMKEY_FORMAT_CARETCTRL, TERMKEY_FORMAT_LONGMOD,
    TERMKEY_FORMAT_LOWERMOD, TERMKEY_FORMAT_LOWERSPACE, TERMKEY_FORMAT_MOUSE_POS,
    TERMKEY_FORMAT_SPACEMOD, TERMKEY_FORMAT_WRAPBRACKET, TERMKEY_KEYMOD_ALT, TERMKEY_KEYMOD_CTRL,
    TERMKEY_KEYMOD_SHIFT,
};
use crate::types::{TermKeyFormat, TermKeyMouseEvent, TermKeySym};
use core::ffi::c_int;
use core::fmt::{self, Write};

/// A byte buffer that `write!` can target. The rendered text is not always
/// valid UTF-8: a key's `utf8` field holds the editor's six-byte encoding, and
/// codepoints past U+10FFFF have no valid spelling.
#[derive(Default)]
struct Out(Vec<u8>);

impl Write for Out {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0.extend_from_slice(text.as_bytes());
        Ok(())
    }
}

/// What a key is, with everything the union and the driver would have to be
/// consulted for already resolved.
pub enum KeyBody<'a> {
    /// A character, and its UTF-8 spelling.
    Unicode {
        codepoint: c_int,
        utf8: &'a [u8],
    },
    Sym(TermKeySym),
    /// A numbered function key.
    Function(c_int),
    Mouse {
        event: TermKeyMouseEvent,
        button: c_int,
        line: c_int,
        col: c_int,
    },
    Position,
    Mode {
        initial: c_int,
        mode: c_int,
        value: c_int,
    },
    Dcs,
    Osc,
    Apc,
    /// A control sequence nothing recognised, identified by its final byte.
    UnknownCsi(c_int),
    /// A key of no type this build knows. Upstream wrote nothing for it and
    /// then added the length of whatever it wrote last.
    Unrecognised,
}

/// How the three modifiers are spelled, chosen by the LONGMOD, ALTISMETA and
/// LOWERMOD format bits.
struct ModNames {
    shift: &'static str,
    alt: &'static str,
    ctrl: &'static str,
}

const fn mods(shift: &'static str, alt: &'static str, ctrl: &'static str) -> ModNames {
    ModNames { shift, alt, ctrl }
}

static MOD_NAMES: [ModNames; 8] = [
    mods("S", "A", "C"),
    mods("Shift", "Alt", "Ctrl"),
    mods("S", "M", "C"),
    mods("Shift", "Meta", "Ctrl"),
    mods("s", "a", "c"),
    mods("shift", "alt", "ctrl"),
    mods("s", "m", "c"),
    mods("shift", "meta", "ctrl"),
];

static MOUSE_EVENT_NAMES: [&str; 4] = ["Unknown", "Press", "Drag", "Release"];

fn has(format: TermKeyFormat, bit: TermKeyFormat) -> bool {
    format & bit != 0
}

/// Render `body` with `modifiers` as `format` asks for.
pub fn render(body: &KeyBody, modifiers: c_int, format: TermKeyFormat) -> Vec<u8> {
    let names = &MOD_NAMES[usize::from(has(format, TERMKEY_FORMAT_LONGMOD))
        + usize::from(has(format, TERMKEY_FORMAT_ALTISMETA)) * 2
        + usize::from(has(format, TERMKEY_FORMAT_LOWERMOD)) * 4];
    // Brackets go round anything but a bare character.
    let wrapbracket = has(format, TERMKEY_FORMAT_WRAPBRACKET)
        && (!matches!(body, KeyBody::Unicode { .. }) || modifiers != 0);
    let separator = if has(format, TERMKEY_FORMAT_SPACEMOD) {
        ' '
    } else {
        '-'
    };

    if has(format, TERMKEY_FORMAT_CARETCTRL)
        && let KeyBody::Unicode { codepoint, .. } = body
        && modifiers == TERMKEY_KEYMOD_CTRL as c_int
        && let Some(caret) = caret_notation(*codepoint)
    {
        // Caret notation carries the control modifier itself, so nothing else
        // is emitted — not even the other modifiers, had there been any.
        return if wrapbracket {
            format!("<^{caret}>").into_bytes()
        } else {
            format!("^{caret}").into_bytes()
        };
    }

    let mut out = Out::default();
    if wrapbracket {
        out.0.push(b'<');
    }
    for (bit, name) in [
        (TERMKEY_KEYMOD_ALT, names.alt),
        (TERMKEY_KEYMOD_CTRL, names.ctrl),
        (TERMKEY_KEYMOD_SHIFT, names.shift),
    ] {
        if modifiers & bit as c_int != 0 {
            let _ = write!(out, "{name}{separator}");
        }
    }
    write_body(&mut out, body, format);
    if wrapbracket {
        out.0.push(b'>');
    }
    out.0
}

/// The letter a control character is `^`-notated as, if it has one.
fn caret_notation(codepoint: c_int) -> Option<char> {
    let byte = codepoint as u8;
    match byte {
        b'a'..=b'z' => Some((byte - 0x20) as char),
        b'@' | b'['..=b'_' => Some(byte as char),
        _ => None,
    }
}

fn write_body(out: &mut Out, body: &KeyBody, format: TermKeyFormat) {
    let lowerspace = has(format, TERMKEY_FORMAT_LOWERSPACE);
    match *body {
        KeyBody::Unicode { utf8, .. } => out.0.extend_from_slice(utf8),
        KeyBody::Sym(sym) => {
            let name = keynames::name(sym);
            if lowerspace {
                let _ = write!(out, "{}", keynames::spaced_lowercase(name));
            } else {
                let _ = write!(out, "{}", keynames::text(name));
            }
        }
        KeyBody::Function(number) => {
            let prefix = if lowerspace { 'f' } else { 'F' };
            let _ = write!(out, "{prefix}{number}");
        }
        KeyBody::Mouse {
            event,
            button,
            line,
            col,
        } => {
            let name = MOUSE_EVENT_NAMES
                .get(event as usize)
                .copied()
                .unwrap_or("Unknown");
            let _ = write!(out, "Mouse{name}({button})");
            if has(format, TERMKEY_FORMAT_MOUSE_POS) {
                let _ = write!(out, " @ ({col},{line})");
            }
        }
        KeyBody::Position => out.0.extend_from_slice(b"Position"),
        KeyBody::Mode {
            initial,
            mode,
            value,
        } => {
            if initial != 0 {
                let _ = write!(out, "Mode({}{mode}={value})", initial as u8 as char);
            } else {
                let _ = write!(out, "Mode({mode}={value})");
            }
        }
        KeyBody::Dcs => out.0.extend_from_slice(b"DCS"),
        KeyBody::Osc => out.0.extend_from_slice(b"OSC"),
        KeyBody::Apc => out.0.extend_from_slice(b"APC"),
        KeyBody::UnknownCsi(command) => {
            let _ = write!(out, "CSI {}", (command & 0xff) as u8 as char);
        }
        KeyBody::Unrecognised => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTRL: c_int = TERMKEY_KEYMOD_CTRL as c_int;
    const ALT: c_int = TERMKEY_KEYMOD_ALT as c_int;
    const SHIFT: c_int = TERMKEY_KEYMOD_SHIFT as c_int;

    fn unicode(codepoint: c_int, utf8: &str) -> KeyBody<'_> {
        KeyBody::Unicode {
            codepoint,
            utf8: utf8.as_bytes(),
        }
    }

    fn text(body: &KeyBody, modifiers: c_int, format: TermKeyFormat) -> String {
        String::from_utf8(render(body, modifiers, format)).expect("test bodies are all ASCII")
    }

    #[test]
    fn a_plain_character_is_itself_even_in_brackets() {
        let key = unicode(b'A' as c_int, "A");
        assert_eq!(text(&key, 0, 0), "A");
        assert_eq!(text(&key, 0, TERMKEY_FORMAT_WRAPBRACKET), "A");
    }

    #[test]
    fn modifiers_spell_out_and_lower_case_on_request() {
        let key = unicode(b'b' as c_int, "b");
        assert_eq!(text(&key, CTRL, 0), "C-b");
        assert_eq!(text(&key, CTRL, TERMKEY_FORMAT_LONGMOD), "Ctrl-b");
        assert_eq!(
            text(&key, CTRL, TERMKEY_FORMAT_LONGMOD | TERMKEY_FORMAT_SPACEMOD),
            "Ctrl b"
        );
        assert_eq!(
            text(&key, CTRL, TERMKEY_FORMAT_LONGMOD | TERMKEY_FORMAT_LOWERMOD),
            "ctrl-b"
        );
        assert_eq!(text(&key, CTRL, TERMKEY_FORMAT_WRAPBRACKET), "<C-b>");
    }

    #[test]
    fn alt_becomes_meta_on_request() {
        let key = unicode(b'c' as c_int, "c");
        assert_eq!(text(&key, ALT, 0), "A-c");
        assert_eq!(text(&key, ALT, TERMKEY_FORMAT_LONGMOD), "Alt-c");
        assert_eq!(text(&key, ALT, TERMKEY_FORMAT_ALTISMETA), "M-c");
        assert_eq!(
            text(&key, ALT, TERMKEY_FORMAT_LONGMOD | TERMKEY_FORMAT_ALTISMETA),
            "Meta-c"
        );
    }

    #[test]
    fn modifiers_are_emitted_alt_then_ctrl_then_shift() {
        let key = unicode(b'x' as c_int, "x");
        assert_eq!(text(&key, ALT | CTRL | SHIFT, 0), "A-C-S-x");
    }

    #[test]
    fn caret_notation_replaces_the_control_modifier() {
        assert_eq!(
            text(&unicode(b'b' as c_int, "b"), CTRL, TERMKEY_FORMAT_CARETCTRL),
            "^B"
        );
        assert_eq!(
            text(
                &unicode(b'b' as c_int, "b"),
                CTRL,
                TERMKEY_FORMAT_CARETCTRL | TERMKEY_FORMAT_WRAPBRACKET
            ),
            "<^B>"
        );
        // '@' and '[' through '_' are their own caret letters.
        assert_eq!(
            text(&unicode(b'@' as c_int, "@"), CTRL, TERMKEY_FORMAT_CARETCTRL),
            "^@"
        );
        // Anything else falls back to the modifier prefix.
        assert_eq!(
            text(&unicode(b'1' as c_int, "1"), CTRL, TERMKEY_FORMAT_CARETCTRL),
            "C-1"
        );
        // As does a key carrying more than just control.
        assert_eq!(
            text(
                &unicode(b'b' as c_int, "b"),
                CTRL | ALT,
                TERMKEY_FORMAT_CARETCTRL
            ),
            "A-C-b"
        );
    }

    #[test]
    fn symbols_and_function_keys() {
        assert_eq!(text(&KeyBody::Sym(7), 0, 0), "Up");
        assert_eq!(
            text(&KeyBody::Sym(7), 0, TERMKEY_FORMAT_WRAPBRACKET),
            "<Up>"
        );
        assert_eq!(text(&KeyBody::Sym(16), 0, 0), "PageUp");
        assert_eq!(
            text(&KeyBody::Sym(16), 0, TERMKEY_FORMAT_LOWERSPACE),
            "page up"
        );
        assert_eq!(text(&KeyBody::Function(5), 0, 0), "F5");
        assert_eq!(
            text(&KeyBody::Function(5), 0, TERMKEY_FORMAT_WRAPBRACKET),
            "<F5>"
        );
        assert_eq!(
            text(&KeyBody::Function(5), 0, TERMKEY_FORMAT_LOWERSPACE),
            "f5"
        );
    }

    #[test]
    fn reports_render_their_payload() {
        let mouse = KeyBody::Mouse {
            event: 1,
            button: 1,
            line: 1,
            col: 1,
        };
        assert_eq!(text(&mouse, 0, 0), "MousePress(1)");
        assert_eq!(
            text(&mouse, 0, TERMKEY_FORMAT_MOUSE_POS),
            "MousePress(1) @ (1,1)"
        );
        assert_eq!(text(&mouse, CTRL, 0), "C-MousePress(1)");
        assert_eq!(text(&KeyBody::Position, 0, 0), "Position");
        assert_eq!(
            text(
                &KeyBody::Mode {
                    initial: b'?' as c_int,
                    mode: 1,
                    value: 2
                },
                0,
                0
            ),
            "Mode(?1=2)"
        );
        assert_eq!(
            text(
                &KeyBody::Mode {
                    initial: 0,
                    mode: 4,
                    value: 1
                },
                0,
                0
            ),
            "Mode(4=1)"
        );
        assert_eq!(text(&KeyBody::Dcs, 0, 0), "DCS");
        assert_eq!(text(&KeyBody::UnknownCsi(b'v' as c_int), 0, 0), "CSI v");
    }
}
