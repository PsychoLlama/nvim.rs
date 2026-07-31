//! Naming keys.
//!
//! What termkey hands back is a [`TermKeyKey`]: a type, a codepoint or a
//! symbol, and modifier bits. What the editor reads is text — `a`, `<C-A>`,
//! `<LeftDrag><12,4>`. Turning the first into the second is all this module
//! does; the bytes it produces go to the editor unchanged, so the spellings
//! here are the ones `:help key-notation` documents.
//!
//! termkey writes most of a key itself. What it does not know about is the
//! kitty keyboard protocol's private-use codepoints and the super/meta
//! modifiers, which is why those two are spelled out here.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::tui::termkey::driver_csi::termkey_interpret_mouse;
use crate::src::nvim::tui::termkey::termkey::{
    TERMKEY_FORMAT_ALTISMETA, TERMKEY_FORMAT_WRAPBRACKET, TERMKEY_KEYMOD_ALT, TERMKEY_KEYMOD_CTRL,
    TERMKEY_KEYMOD_SHIFT, TERMKEY_MOUSE_DRAG, TERMKEY_MOUSE_PRESS, TERMKEY_MOUSE_RELEASE,
    TERMKEY_MOUSE_UNKNOWN, TERMKEY_SYM_SUSPEND, TERMKEY_TYPE_KEYSYM, TERMKEY_TYPE_UNICODE,
    termkey_strfkey,
};
use crate::src::nvim::types::{TermKey, TermKeyKey, TermKeyMouseEvent};
use core::ffi::{c_char, c_int};
use core::fmt::Write;

/// The modifiers termkey reports, as the bits a key carries them in.
const MOD_SHIFT: c_int = TERMKEY_KEYMOD_SHIFT as c_int;
const MOD_ALT: c_int = TERMKEY_KEYMOD_ALT as c_int;
const MOD_CTRL: c_int = TERMKEY_KEYMOD_CTRL as c_int;

/// The modifiers termkey does not report, in the same bits the kitty
/// keyboard protocol sends them.
const MOD_SUPER: c_int = 8;
const MOD_META: c_int = 32;

/// Every modifier this layer can name. A key carrying none of them is
/// ordinary text and needs no `<>` around it.
pub(super) const KEYMOD_RECOGNIZED: c_int = MOD_SHIFT | MOD_ALT | MOD_CTRL | MOD_SUPER | MOD_META;

/// The private-use area, where the kitty keyboard protocol puts the keys
/// Unicode has no codepoint for.
const PRIVATE_USE: core::ops::RangeInclusive<c_int> = 0xe000..=0xf8ff;

/// How termkey is asked to spell a key: alt as meta, and the whole thing
/// wrapped in `<>`.
const KEY_FORMAT: core::ffi::c_uint = TERMKEY_FORMAT_ALTISMETA | TERMKEY_FORMAT_WRAPBRACKET;

// ------------------------------------------------------------------- buffer

/// How long a named key can get. The longest thing written here is a mouse
/// event with every modifier and a four-digit position, at forty-odd bytes.
const KEY_TEXT_MAX: usize = 64;

/// A key in the editor's notation, built up in place.
///
/// Overflowing it is a bug in this module rather than something a terminal
/// can provoke — no sequence names a key longer than [`KEY_TEXT_MAX`] — so
/// it panics rather than truncating, as the C asserted.
pub(super) struct KeyText {
    buf: [u8; KEY_TEXT_MAX],
    len: usize,
}

impl KeyText {
    pub(super) fn new() -> Self {
        Self {
            buf: [0; KEY_TEXT_MAX],
            len: 0,
        }
    }

    pub(super) fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    fn push(&mut self, byte: u8) {
        assert!(self.len < KEY_TEXT_MAX, "key name too long");
        self.buf[self.len] = byte;
        self.len += 1;
    }

    fn push_str(&mut self, text: &str) {
        let end = self.len + text.len();
        assert!(end < KEY_TEXT_MAX, "key name too long");
        self.buf[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
    }

    /// Insert `text` at `at`, shifting what follows along. This is how the
    /// modifiers termkey does not know about get in after the opening `<`
    /// of a key it has already spelled.
    fn insert_str(&mut self, at: usize, text: &str) {
        let end = self.len + text.len();
        assert!(at <= self.len, "insert past the end of a key name");
        assert!(end < KEY_TEXT_MAX, "key name too long");
        self.buf.copy_within(at..self.len, at + text.len());
        self.buf[at..at + text.len()].copy_from_slice(text.as_bytes());
        self.len = end;
    }
}

impl Write for KeyText {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        self.push_str(text);
        Ok(())
    }
}

// ---------------------------------------------------------------- modifiers

/// The modifiers termkey reports, in the order the editor spells them.
fn write_modifiers(key: &TermKeyKey, text: &mut KeyText) {
    for (bit, name) in [(MOD_SHIFT, "S-"), (MOD_ALT, "A-"), (MOD_CTRL, "C-")] {
        if key.modifiers & bit != 0 {
            text.push_str(name);
        }
    }
}

/// The modifiers only the kitty keyboard protocol reports, which termkey
/// passes through as bits without naming.
fn more_modifiers(key: &TermKeyKey) -> &'static str {
    match (
        key.modifiers & MOD_SUPER != 0,
        key.modifiers & MOD_META != 0,
    ) {
        (true, true) => "D-T-",
        (true, false) => "D-",
        (false, true) => "T-",
        (false, false) => "",
    }
}

// -------------------------------------------------------- the kitty protocol

/// The kitty keyboard protocol's private-use codepoints and what the editor
/// calls them. Sorted by codepoint: [`kitty_key_name`] searches it.
static KITTY_KEYS: [(c_int, &str); 77] = [
    (57344, "Esc"),
    (57345, "CR"),
    (57346, "Tab"),
    (57347, "BS"),
    (57348, "Insert"),
    (57349, "Del"),
    (57350, "Left"),
    (57351, "Right"),
    (57352, "Up"),
    (57353, "Down"),
    (57354, "PageUp"),
    (57355, "PageDown"),
    (57356, "Home"),
    (57357, "End"),
    (57364, "F1"),
    (57365, "F2"),
    (57366, "F3"),
    (57367, "F4"),
    (57368, "F5"),
    (57369, "F6"),
    (57370, "F7"),
    (57371, "F8"),
    (57372, "F9"),
    (57373, "F10"),
    (57374, "F11"),
    (57375, "F12"),
    (57376, "F13"),
    (57377, "F14"),
    (57378, "F15"),
    (57379, "F16"),
    (57380, "F17"),
    (57381, "F18"),
    (57382, "F19"),
    (57383, "F20"),
    (57384, "F21"),
    (57385, "F22"),
    (57386, "F23"),
    (57387, "F24"),
    (57388, "F25"),
    (57389, "F26"),
    (57390, "F27"),
    (57391, "F28"),
    (57392, "F29"),
    (57393, "F30"),
    (57394, "F31"),
    (57395, "F32"),
    (57396, "F33"),
    (57397, "F34"),
    (57398, "F35"),
    (57399, "k0"),
    (57400, "k1"),
    (57401, "k2"),
    (57402, "k3"),
    (57403, "k4"),
    (57404, "k5"),
    (57405, "k6"),
    (57406, "k7"),
    (57407, "k8"),
    (57408, "k9"),
    (57409, "kPoint"),
    (57410, "kDivide"),
    (57411, "kMultiply"),
    (57412, "kMinus"),
    (57413, "kPlus"),
    (57414, "kEnter"),
    (57415, "kEqual"),
    (57417, "kLeft"),
    (57418, "kRight"),
    (57419, "kUp"),
    (57420, "kDown"),
    (57421, "kPageUp"),
    (57422, "kPageDown"),
    (57423, "kHome"),
    (57424, "kEnd"),
    (57425, "kInsert"),
    (57426, "kDel"),
    (57427, "kOrigin"),
];

/// What the editor calls the kitty key at `codepoint`, if it is one.
fn kitty_key_name(codepoint: c_int) -> Option<&'static str> {
    KITTY_KEYS
        .binary_search_by_key(&codepoint, |&(code, _)| code)
        .ok()
        .map(|i| KITTY_KEYS[i].1)
}

/// Name a key the kitty keyboard protocol sent, if that is what this is.
///
/// Keys outside the private-use area, and codepoints in it that the protocol
/// does not define, are not this module's to name.
fn kitty_protocol(key: &TermKeyKey) -> Option<KeyText> {
    // SAFETY: `code` is a union tagged by `type_0`; every caller here has
    // already established this is a unicode key, whose arm is `codepoint`.
    let codepoint = unsafe { key.code.codepoint };
    if !PRIVATE_USE.contains(&codepoint) {
        return None;
    }
    let name = kitty_key_name(codepoint)?;
    let mut text = KeyText::new();
    text.push(b'<');
    write_modifiers(key, &mut text);
    text.push_str(more_modifiers(key));
    text.push_str(name);
    text.push(b'>');
    Some(text)
}

// --------------------------------------------------------------- whole keys

/// Name a key carrying no modifiers: its own UTF-8, with the one character
/// the editor cannot read literally spelled out.
pub(super) fn simple_utf8(key: &TermKeyKey) -> KeyText {
    if let Some(text) = kitty_protocol(key) {
        return text;
    }
    let mut text = KeyText::new();
    for &byte in &key.utf8 {
        match byte as u8 {
            0 => break,
            b'<' => text.push_str("<lt>"),
            byte => text.push(byte),
        }
    }
    text
}

/// Name a key carrying modifiers, or one termkey has a name for.
///
/// # Safety
/// `tk` must be the termkey instance `key` came from.
pub(super) unsafe fn modified_utf8(tk: *mut TermKey, key: &TermKeyKey) -> KeyText {
    // SAFETY: `code` is tagged by `type_0`, and the caller guarantees `tk`.
    let mut text = unsafe {
        if key.type_0 == TERMKEY_TYPE_KEYSYM && key.code.sym == TERMKEY_SYM_SUSPEND {
            // The editor's own suspend, rather than a key named `Suspend`.
            let mut text = KeyText::new();
            text.push_str("<C-Z>");
            text
        } else if key.type_0 != TERMKEY_TYPE_UNICODE {
            strfkey(tk, key)
        } else {
            debug_assert!(key.modifiers != 0, "an unmodified key is not this one");
            if let Some(text) = kitty_protocol(key) {
                return text;
            }
            let mut text = strfkey(tk, key);
            // termkey spells a control key in its uppercase form, so the
            // shift that tells `<C-S-A>` from `<C-A>` has to be put back.
            let codepoint = key.code.codepoint;
            let shifted_control = key.modifiers & MOD_CTRL != 0
                && key.modifiers & MOD_SHIFT == 0
                && (c_int::from(b'A')..=c_int::from(b'Z')).contains(&codepoint);
            if shifted_control {
                text.insert_str(1, "S-");
            }
            text
        }
    };
    text.insert_str(1, more_modifiers(key));
    text
}

/// Let termkey spell `key` itself.
///
/// # Safety
/// `tk` must be the termkey instance `key` came from.
unsafe fn strfkey(tk: *mut TermKey, key: &TermKeyKey) -> KeyText {
    let mut text = KeyText::new();
    // SAFETY: the caller guarantees `tk`; the buffer is this frame's, and
    // termkey writes at most as many bytes as it is given. `key` is read,
    // not written, but termkey's signature does not say so.
    let len = unsafe {
        termkey_strfkey(
            tk,
            text.buf.as_mut_ptr().cast::<c_char>(),
            KEY_TEXT_MAX,
            core::ptr::from_ref(key).cast_mut(),
            KEY_FORMAT,
        )
    };
    assert!(len < KEY_TEXT_MAX, "key name too long");
    text.len = len;
    text
}

// -------------------------------------------------------------------- mouse

/// Which button is being held, so a drag or a release can say which one it
/// is: terminals report the button on the press and leave it out afterwards.
static LAST_PRESSED_BUTTON: GlobalCell<c_int> = GlobalCell::new(0);

/// Name a mouse event: what happened, to which button, and where.
///
/// Returns `None` for the events the editor has no notation for — anything
/// that is not a press, a drag or a release.
///
/// # Safety
/// `tk` must be the termkey instance `key` came from.
pub(super) unsafe fn mouse_event(tk: *mut TermKey, key: &TermKeyKey) -> Option<KeyText> {
    let mut event: TermKeyMouseEvent = TERMKEY_MOUSE_UNKNOWN;
    let (mut button, mut row, mut col) = (0, 0, 0);
    // SAFETY: the caller guarantees `tk`; the out-parameters are this
    // frame's. `key` is read, not written.
    unsafe {
        termkey_interpret_mouse(
            tk,
            core::ptr::from_ref(key),
            &raw mut event,
            &raw mut button,
            &raw mut row,
            &raw mut col,
        );
    }

    // A drag or a release names no button; it is whichever one went down.
    if (event == TERMKEY_MOUSE_RELEASE || event == TERMKEY_MOUSE_DRAG) && button == 0 {
        button = LAST_PRESSED_BUTTON.get();
    }
    if button == 0 && event != TERMKEY_MOUSE_RELEASE
        || event != TERMKEY_MOUSE_PRESS
            && event != TERMKEY_MOUSE_DRAG
            && event != TERMKEY_MOUSE_RELEASE
    {
        return None;
    }

    let mut text = KeyText::new();
    text.push(b'<');
    write_modifiers(key, &mut text);
    text.push_str(match button {
        1 => "Left",
        2 => "Middle",
        3 => "Right",
        8 => "X1",
        9 => "X2",
        _ => "",
    });
    match event {
        TERMKEY_MOUSE_PRESS => match button {
            // The wheel arrives as a press of a button that cannot be held.
            4 => text.push_str("ScrollWheelUp"),
            5 => text.push_str("ScrollWheelDown"),
            6 => text.push_str("ScrollWheelLeft"),
            7 => text.push_str("ScrollWheelRight"),
            _ => {
                text.push_str("Mouse");
                LAST_PRESSED_BUTTON.set(button);
            }
        },
        TERMKEY_MOUSE_DRAG => text.push_str("Drag"),
        TERMKEY_MOUSE_RELEASE => {
            // A release with nothing held is the pointer having moved.
            text.push_str(if button != 0 { "Release" } else { "MouseMove" });
            LAST_PRESSED_BUTTON.set(0);
        }
        _ => unreachable!("mouse event {event} is not one of the three"),
    }
    // The terminal counts from one, the editor from zero.
    let _ = write!(text, "><{},{}>", col - 1, row - 1);
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kitty_table_is_searchable() {
        assert!(
            KITTY_KEYS.windows(2).all(|w| w[0].0 < w[1].0),
            "the table must be sorted for the search to find anything"
        );
        assert_eq!(kitty_key_name(57344), Some("Esc"));
        assert_eq!(kitty_key_name(57427), Some("kOrigin"));
        assert_eq!(kitty_key_name(57416), None);
        assert_eq!(kitty_key_name(0), None);
    }

    #[test]
    fn text_inserts_in_the_middle() {
        let mut text = KeyText::new();
        text.push(b'<');
        text.push_str("C-A>");
        text.insert_str(1, "D-");
        assert_eq!(text.as_bytes(), b"<D-C-A>");
    }
}
