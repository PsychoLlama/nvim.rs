//! What the user types, on its way to the child program.
//!
//! The editor and the emulator disagree about what a key is. The editor
//! delivers one integer per key, with modifiers in a separate global and
//! anything beyond ASCII spelled as a negative "extra" code; vterm wants a
//! [`VTermKey`] or a Unicode codepoint, plus an explicit modifier set.
//! [`convert_key`] and [`convert_modifiers`] do that translation, and
//! [`terminal_send_key`] hands the result over.
//!
//! Mouse events take a different route. [`send_mouse_event`] decides
//! whether the child even wants them — only if it enabled mouse reporting,
//! and only inside the terminal's own window — and otherwise gives the
//! event back to the editor so that scrolling a terminal window from
//! outside it still scrolls the buffer.
//!
//! Pastes ([`terminal_paste`]) bracket the text and drop the control
//! characters `'termpastefilter'` names, so that pasting a file containing
//! an escape sequence cannot drive the emulator.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::drawscreen::UPD_NOT_VALID;
use crate::getchar::state::{KEY_TYPED, MOD_MASK, VGETC_CHAR, VGETC_MOD_MASK};
use crate::getchar::{ins_char_typebuf, ungetchars};
use crate::keycodes::SpecialKey;
use crate::keycodes::{ModMask, unsimplify_key};
use crate::mbyte::{char_at, char_len};
use crate::memory::XString;
use crate::mouse::{
    MSCR_DOWN, MSCR_LEFT, MSCR_RIGHT, MSCR_UP, MousePos, do_mousescroll, find_win_inner,
};
use crate::normal::with_scratch_cmd;
use crate::option::vars::TPF_FLAGS;
use crate::options::{
    OPT_TPF_FLAG_BS, OPT_TPF_FLAG_C0, OPT_TPF_FLAG_C1, OPT_TPF_FLAG_DEL, OPT_TPF_FLAG_ESC,
    OPT_TPF_FLAG_FF, OPT_TPF_FLAG_HT,
};
use crate::types::{Key, KeyKind};

use crate::vterm::keycodes::{VTermKey, VTermModifier};
use crate::vterm::types::VTerm;
use crate::winlayer::graph::switch_to;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::refresh::invalidate_terminal;
use super::{Term, terminal_send};
// vterm's key names, from libvterm's `VTermKey`. Function keys are not
// named individually: they are `FUNCTION_0 + n`, see [`function_key`].
use crate::vterm::vterm::{
    VTERM_KEY_BACKSPACE, VTERM_KEY_DEL, VTERM_KEY_DOWN, VTERM_KEY_END, VTERM_KEY_ENTER,
    VTERM_KEY_ESCAPE, VTERM_KEY_FUNCTION_0, VTERM_KEY_HOME, VTERM_KEY_INS, VTERM_KEY_KP_0,
    VTERM_KEY_KP_1, VTERM_KEY_KP_2, VTERM_KEY_KP_3, VTERM_KEY_KP_4, VTERM_KEY_KP_5, VTERM_KEY_KP_6,
    VTERM_KEY_KP_7, VTERM_KEY_KP_8, VTERM_KEY_KP_9, VTERM_KEY_KP_DIVIDE, VTERM_KEY_KP_ENTER,
    VTERM_KEY_KP_MINUS, VTERM_KEY_KP_MULT, VTERM_KEY_KP_PERIOD, VTERM_KEY_KP_PLUS, VTERM_KEY_LEFT,
    VTERM_KEY_NONE, VTERM_KEY_PAGEDOWN, VTERM_KEY_PAGEUP, VTERM_KEY_RIGHT, VTERM_KEY_TAB,
    VTERM_KEY_UP, VTERM_MOD_ALT, VTERM_MOD_CTRL, VTERM_MOD_NONE, VTERM_MOD_SHIFT,
};

/// vterm's name for function key `n`.
const fn function_key(n: c_int) -> VTermKey {
    VTERM_KEY_FUNCTION_0 + n.cast_unsigned()
}

/// vterm's button numbers: 1-3 are the real buttons, 4-7 are the wheel's
/// four directions, and 8-9 are the side buttons.
const BUTTON_LEFT: c_int = 1;
const BUTTON_MIDDLE: c_int = 2;
const BUTTON_RIGHT: c_int = 3;
const BUTTON_WHEEL_UP: c_int = 4;
const BUTTON_WHEEL_DOWN: c_int = 5;
const BUTTON_WHEEL_RIGHT: c_int = 6;
const BUTTON_WHEEL_LEFT: c_int = 7;
const BUTTON_X1: c_int = 8;
const BUTTON_X2: c_int = 9;

/// Move the editor's modifiers `mods` into vterm's modifier state.
///
/// Control-uppercase is lowercased first: the editor reports `<C-A>` as
/// Ctrl plus `A`, but a terminal's control codes are computed from the
/// lowercase letter.
fn convert_modifiers(key: &mut Key, mods: ModMask, state: &mut VTermModifier) {
    if mods.has(ModMask::SHIFT) {
        *state |= VTERM_MOD_SHIFT;
    }
    if mods.has(ModMask::CTRL) {
        *state |= VTERM_MOD_CTRL;
        if !mods.has(ModMask::SHIFT)
            && let Some(upper) = key.as_byte().filter(u8::is_ascii_uppercase)
        {
            *key = Key::byte(upper.to_ascii_lowercase());
        }
    }
    if mods.has(ModMask::ALT) {
        *state |= VTERM_MOD_ALT;
    }
}

/// Apply the editor's modifiers `mods` to `key`, and name the vterm key it
/// stands for — [`VTERM_KEY_NONE`] if it is an ordinary character rather
/// than a named key.
///
/// A key that is itself a modified code (`<S-Del>`, `<C-Left>`) is split
/// back into the plain key and its modifier first, so every such code the
/// editor has reaches the child, not only a hand-picked few.
fn convert_key(key: &mut Key, mods: ModMask, state: &mut VTermModifier) -> VTermKey {
    let (unsimplified, mods) = unsimplify_key(*key, mods);
    *key = unsimplified;
    convert_modifiers(key, mods, state);
    match key.kind() {
        KeyKind::Special(SpecialKey::Bs) => VTERM_KEY_BACKSPACE,
        KeyKind::Special(SpecialKey::Tab) | KeyKind::Char(Key::TAB) => VTERM_KEY_TAB,
        KeyKind::Char(Key::CAR) => VTERM_KEY_ENTER,
        KeyKind::Char(Key::ESC) => VTERM_KEY_ESCAPE,
        KeyKind::Special(SpecialKey::Up) => VTERM_KEY_UP,
        KeyKind::Special(SpecialKey::Down) => VTERM_KEY_DOWN,
        KeyKind::Special(SpecialKey::Left) => VTERM_KEY_LEFT,
        KeyKind::Special(SpecialKey::Right) => VTERM_KEY_RIGHT,
        KeyKind::Special(SpecialKey::Ins) => VTERM_KEY_INS,
        KeyKind::Special(SpecialKey::Del) => VTERM_KEY_DEL,
        KeyKind::Special(SpecialKey::Home) => VTERM_KEY_HOME,
        KeyKind::Special(SpecialKey::End) => VTERM_KEY_END,
        KeyKind::Special(SpecialKey::Pageup) => VTERM_KEY_PAGEUP,
        KeyKind::Special(SpecialKey::Pagedown) => VTERM_KEY_PAGEDOWN,
        // The keypad's keys double as cursor keys when NumLock is off, and
        // the editor reports which face was used; vterm wants the keypad.
        KeyKind::Special(SpecialKey::K0 | SpecialKey::Kins) => VTERM_KEY_KP_0,
        KeyKind::Special(SpecialKey::K1 | SpecialKey::Kend) => VTERM_KEY_KP_1,
        KeyKind::Special(SpecialKey::K2 | SpecialKey::Kdown) => VTERM_KEY_KP_2,
        KeyKind::Special(SpecialKey::K3 | SpecialKey::Kpagedown) => VTERM_KEY_KP_3,
        KeyKind::Special(SpecialKey::K4 | SpecialKey::Kleft) => VTERM_KEY_KP_4,
        KeyKind::Special(SpecialKey::K5 | SpecialKey::Korigin) => VTERM_KEY_KP_5,
        KeyKind::Special(SpecialKey::K6 | SpecialKey::Kright) => VTERM_KEY_KP_6,
        KeyKind::Special(SpecialKey::K7 | SpecialKey::Khome) => VTERM_KEY_KP_7,
        KeyKind::Special(SpecialKey::K8 | SpecialKey::Kup) => VTERM_KEY_KP_8,
        KeyKind::Special(SpecialKey::K9 | SpecialKey::Kpageup) => VTERM_KEY_KP_9,
        KeyKind::Special(SpecialKey::Kdel | SpecialKey::Kpoint) => VTERM_KEY_KP_PERIOD,
        KeyKind::Special(SpecialKey::Kenter) => VTERM_KEY_KP_ENTER,
        KeyKind::Special(SpecialKey::Kplus) => VTERM_KEY_KP_PLUS,
        KeyKind::Special(SpecialKey::Kminus) => VTERM_KEY_KP_MINUS,
        KeyKind::Special(SpecialKey::Kmultiply) => VTERM_KEY_KP_MULT,
        KeyKind::Special(SpecialKey::Kdivide) => VTERM_KEY_KP_DIVIDE,
        KeyKind::Special(SpecialKey::F1) => function_key(1),
        KeyKind::Special(SpecialKey::F2) => function_key(2),
        KeyKind::Special(SpecialKey::F3) => function_key(3),
        KeyKind::Special(SpecialKey::F4) => function_key(4),
        KeyKind::Special(SpecialKey::F5) => function_key(5),
        KeyKind::Special(SpecialKey::F6) => function_key(6),
        KeyKind::Special(SpecialKey::F7) => function_key(7),
        KeyKind::Special(SpecialKey::F8) => function_key(8),
        KeyKind::Special(SpecialKey::F9) => function_key(9),
        KeyKind::Special(SpecialKey::F10) => function_key(10),
        KeyKind::Special(SpecialKey::F11) => function_key(11),
        KeyKind::Special(SpecialKey::F12) => function_key(12),
        KeyKind::Special(SpecialKey::F13) => function_key(13),
        KeyKind::Special(SpecialKey::F14) => function_key(14),
        KeyKind::Special(SpecialKey::F15) => function_key(15),
        KeyKind::Special(SpecialKey::F16) => function_key(16),
        KeyKind::Special(SpecialKey::F17) => function_key(17),
        KeyKind::Special(SpecialKey::F18) => function_key(18),
        KeyKind::Special(SpecialKey::F19) => function_key(19),
        KeyKind::Special(SpecialKey::F20) => function_key(20),
        KeyKind::Special(SpecialKey::F21) => function_key(21),
        KeyKind::Special(SpecialKey::F22) => function_key(22),
        KeyKind::Special(SpecialKey::F23) => function_key(23),
        KeyKind::Special(SpecialKey::F24) => function_key(24),
        KeyKind::Special(SpecialKey::F25) => function_key(25),
        KeyKind::Special(SpecialKey::F26) => function_key(26),
        KeyKind::Special(SpecialKey::F27) => function_key(27),
        KeyKind::Special(SpecialKey::F28) => function_key(28),
        KeyKind::Special(SpecialKey::F29) => function_key(29),
        KeyKind::Special(SpecialKey::F30) => function_key(30),
        KeyKind::Special(SpecialKey::F31) => function_key(31),
        KeyKind::Special(SpecialKey::F32) => function_key(32),
        KeyKind::Special(SpecialKey::F33) => function_key(33),
        KeyKind::Special(SpecialKey::F34) => function_key(34),
        KeyKind::Special(SpecialKey::F35) => function_key(35),
        KeyKind::Special(SpecialKey::F36) => function_key(36),
        KeyKind::Special(SpecialKey::F37) => function_key(37),
        KeyKind::Special(SpecialKey::F38) => function_key(38),
        KeyKind::Special(SpecialKey::F39) => function_key(39),
        KeyKind::Special(SpecialKey::F40) => function_key(40),
        KeyKind::Special(SpecialKey::F41) => function_key(41),
        KeyKind::Special(SpecialKey::F42) => function_key(42),
        KeyKind::Special(SpecialKey::F43) => function_key(43),
        KeyKind::Special(SpecialKey::F44) => function_key(44),
        KeyKind::Special(SpecialKey::F45) => function_key(45),
        KeyKind::Special(SpecialKey::F46) => function_key(46),
        KeyKind::Special(SpecialKey::F47) => function_key(47),
        KeyKind::Special(SpecialKey::F48) => function_key(48),
        KeyKind::Special(SpecialKey::F49) => function_key(49),
        KeyKind::Special(SpecialKey::F50) => function_key(50),
        KeyKind::Special(SpecialKey::F51) => function_key(51),
        KeyKind::Special(SpecialKey::F52) => function_key(52),
        KeyKind::Special(SpecialKey::F53) => function_key(53),
        KeyKind::Special(SpecialKey::F54) => function_key(54),
        KeyKind::Special(SpecialKey::F55) => function_key(55),
        KeyKind::Special(SpecialKey::F56) => function_key(56),
        KeyKind::Special(SpecialKey::F57) => function_key(57),
        KeyKind::Special(SpecialKey::F58) => function_key(58),
        KeyKind::Special(SpecialKey::F59) => function_key(59),
        KeyKind::Special(SpecialKey::F60) => function_key(60),
        KeyKind::Special(SpecialKey::F61) => function_key(61),
        KeyKind::Special(SpecialKey::F62) => function_key(62),
        KeyKind::Special(SpecialKey::F63) => function_key(63),
        _ => VTERM_KEY_NONE,
    }
}

/// Send one key the editor read to the child.
///
/// A keycode with no vterm name is a codepoint, unless it is negative — one
/// of the extras vterm has no equivalent for, which is simply dropped.
pub(super) fn terminal_send_key(mut term: Term, c: Key) {
    send_key(&mut term.vt, c, MOD_MASK.get());
}

/// [`terminal_send_key`] with the emulator and the editor's modifiers named,
/// so the key-to-bytes mapping can be driven without a live terminal.
fn send_key(vt: &mut VTerm, c: Key, mods: ModMask) {
    let mut state = VTERM_MOD_NONE;
    // The editor spells NUL as an extra so it survives being a C string;
    // vterm wants the control character back.
    let mut key = if c == SpecialKey::Zero { Key::NUL } else { c };
    let named = convert_key(&mut key, mods, &mut state);
    if named != VTERM_KEY_NONE {
        vt.keyboard_key(named, state);
    } else if !key.is_special() {
        vt.keyboard_unichar(key.code().cast_unsigned(), state);
    }
}

/// Whether `'termpastefilter'` says to drop `c` from a paste.
fn is_filter_char(c: c_int) -> bool {
    let flag = match c {
        0x08 => OPT_TPF_FLAG_BS as ::core::ffi::c_uint,
        0x09 => OPT_TPF_FLAG_HT as ::core::ffi::c_uint,
        // Newlines are the one thing a paste is guaranteed to want.
        0x0a | 0x0d => return false,
        0x0c => OPT_TPF_FLAG_FF as ::core::ffi::c_uint,
        0x1b => OPT_TPF_FLAG_ESC as ::core::ffi::c_uint,
        0x7f => OPT_TPF_FLAG_DEL as ::core::ffi::c_uint,
        _ if c < 0x20 => OPT_TPF_FLAG_C0 as ::core::ffi::c_uint,
        _ if (0x80..=0x9f).contains(&c) => OPT_TPF_FLAG_C1 as ::core::ffi::c_uint,
        _ => return false,
    };
    TPF_FLAGS.get() & flag != 0
}

/// Open or close a bracketed paste that spans more than one `nvim_paste`
/// call, so the child of `buffer`'s terminal, if it has one, sees one paste
/// rather than several.
pub(crate) fn buf_terminal_set_streamed_paste(buffer: Buf, streamed: bool) {
    let Some(mut term) = buffer.terminal_handle() else {
        return;
    };
    if term.streamed_paste != streamed {
        if streamed {
            term.vt.keyboard_start_paste();
        } else {
            term.vt.keyboard_end_paste();
        }
    }
    term.streamed_paste = streamed;
}

/// Paste `lines` into the current terminal `count` times.
///
/// Bracketed unless the paste is already part of a stream, and filtered
/// through `'termpastefilter'` a character at a time.
pub(crate) fn terminal_paste(lines: &[XString], count: c_int) {
    if lines.is_empty() {
        return;
    }
    // A paste only reaches here while the current buffer has a terminal.
    let mut term = Buf::current()
        .terminal_handle()
        .expect("a paste into a terminal buffer");
    let bracket = !term.streamed_paste;
    if bracket {
        term.vt.keyboard_start_paste();
    }
    let mut filtered: Vec<u8> = Vec::new();
    for _ in 0..count {
        for (index, line) in lines.iter().enumerate() {
            if index != 0 {
                terminal_send(term, b"\n");
            }
            filtered.clear();
            // As the C string the line is, to its first NUL.
            let bytes = line.as_cstr().to_bytes();
            let mut at = 0;
            while at < bytes.len() {
                let (len, c) = (char_len(&bytes[at..]), char_at(&bytes[at..]));
                if !is_filter_char(c.get()) {
                    filtered.extend_from_slice(&bytes[at..at + len]);
                }
                at += len;
            }
            terminal_send(term, &filtered);
        }
    }
    if bracket {
        term.vt.keyboard_end_paste();
    }
}

/// The vterm button a mouse keycode stands for, and whether it is a press.
///
/// Drags count as presses: vterm tracks the button down and turns a move
/// with a button held into a drag report itself.
fn mouse_button(c: Key) -> Option<(c_int, bool)> {
    match c.special() {
        Some(SpecialKey::Leftmouse | SpecialKey::Leftdrag) => Some((BUTTON_LEFT, true)),
        Some(SpecialKey::Leftrelease) => Some((BUTTON_LEFT, false)),
        Some(SpecialKey::Middlemouse | SpecialKey::Middledrag) => Some((BUTTON_MIDDLE, true)),
        Some(SpecialKey::Middlerelease) => Some((BUTTON_MIDDLE, false)),
        Some(SpecialKey::Rightmouse | SpecialKey::Rightdrag) => Some((BUTTON_RIGHT, true)),
        Some(SpecialKey::Rightrelease) => Some((BUTTON_RIGHT, false)),
        Some(SpecialKey::X1mouse | SpecialKey::X1drag) => Some((BUTTON_X1, true)),
        Some(SpecialKey::X1release) => Some((BUTTON_X1, false)),
        Some(SpecialKey::X2mouse | SpecialKey::X2drag) => Some((BUTTON_X2, true)),
        Some(SpecialKey::X2release) => Some((BUTTON_X2, false)),
        Some(SpecialKey::Mousedown) => Some((BUTTON_WHEEL_UP, true)),
        Some(SpecialKey::Mouseup) => Some((BUTTON_WHEEL_DOWN, true)),
        Some(SpecialKey::Mouseleft) => Some((BUTTON_WHEEL_LEFT, true)),
        Some(SpecialKey::Mouseright) => Some((BUTTON_WHEEL_RIGHT, true)),
        // A bare move: position only, no button.
        Some(SpecialKey::Mousemove) => Some((0, false)),
        _ => None,
    }
}

/// Whether `c` is a mouse event, and so [`send_mouse_event`]'s to deal
/// with rather than the child's.
pub(super) fn is_mouse_key(c: Key) -> bool {
    mouse_button(c).is_some()
}

/// The scroll direction a wheel keycode asks for, for the editor's own
/// scrolling when the child is not taking mouse events.
fn scroll_direction(c: Key) -> Option<c_int> {
    match c.special() {
        Some(SpecialKey::Mouseup) => Some(MSCR_UP),
        Some(SpecialKey::Mousedown) => Some(MSCR_DOWN),
        Some(SpecialKey::Mouseleft) => Some(MSCR_LEFT),
        Some(SpecialKey::Mouseright) => Some(MSCR_RIGHT),
        _ => None,
    }
}

/// Scroll `mouse_win` as the editor would if the mouse were over an
/// ordinary buffer.
fn scroll_window(mouse_win: Win, key: Key, direction: c_int) {
    let saved = switch_to(mouse_win);

    with_scratch_cmd(|cap| {
        cap.cmdchar = key;
        cap.arg = direction;
        do_mousescroll(cap);
    });

    // Whatever the scroll left as the current window, which need not be the
    // one it started in.
    //
    let mut scrolled = Win::current();
    scrolled.w_redr_status = true;
    saved.restore();
}

/// Deal with a mouse event while a terminal has focus.
///
/// Returns whether the editor should handle the key itself. Three outcomes:
/// the child gets the event, the editor scrolls the window under the
/// pointer, or the event is pushed back for normal-mode processing (which
/// is what makes clicking out of a terminal work).
pub(super) fn send_mouse_event(mut term: Term, c: Key) -> bool {
    let mut pos = MousePos::current();
    let mouse_win = find_win_inner(&mut pos);
    let MousePos { grid, row, col } = pos;

    if let Some(mouse_win) = mouse_win {
        // An external grid is exactly the terminal's window, so the height
        // and width checks below only apply to the shared one.
        //
        let offset = mouse_win.col_off();
        let inside = row >= 0
            && (grid > 1 || row + mouse_win.w_winbar_height < mouse_win.w_height)
            && col >= offset
            && (grid > 1 || col < mouse_win.w_width);
        let showing_term = mouse_win.buffer().terminal == term.raw();
        let forwarding = !term.suspended && !term.closed && term.forward_mouse && showing_term;

        if forwarding && inside {
            let Some((button, pressed)) = mouse_button(c) else {
                return false;
            };
            let mut state = VTERM_MOD_NONE;
            convert_modifiers(&mut { c }, MOD_MASK.get(), &mut state);
            term.vt.mouse_move(row, col - offset, state);
            if button != 0 {
                term.vt.mouse_button(button, pressed, state);
            }
            return false;
        }

        if let Some(direction) = scroll_direction(c) {
            scroll_window(mouse_win, c, direction);
            mouse_win.redraw_later(UPD_NOT_VALID);
            // The terminal's own window may have scrolled under it.
            invalidate_terminal(term, None);
            // False when the user scrolled a different window, so that the
            // editor gets a chance to leave terminal mode.
            return mouse_win.is_current();
        }

        // A release inside the terminal's own window is dropped: it means
        // nothing to the editor here.
        if c == SpecialKey::Leftrelease && showing_term {
            return false;
        }
    }

    // A bare move is dropped for the same reason.
    if c == SpecialKey::Mousemove {
        return false;
    }

    // Hand the event back for normal mode, then rewind the typeahead so
    // that terminal mode ends before it is read.
    let (c, mods) = (VGETC_CHAR.get(), VGETC_MOD_MASK.get());
    let len = ins_char_typebuf(c, mods, true);
    if KEY_TYPED.get() {
        ungetchars(len); // rewind over the key just pushed
    }
    true
}

/// What a key the editor read becomes on the way to the child: the vterm
/// key and modifier [`convert_key`] names, and the bytes [`send_key`] has the
/// emulator write for it in its power-on modes. The oracle for retyping the
/// editor's key codes.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vterm::vterm::OutputSink;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// `convert_key`'s three answers: the vterm key, the modifier state and
    /// the key code it left behind.
    fn converted(key: Key, mods: ModMask) -> (VTermKey, VTermModifier, Key) {
        let (mut key, mut state) = (key, VTERM_MOD_NONE);
        let named = convert_key(&mut key, mods, &mut state);
        (named, state, key)
    }

    /// The bytes the child is sent for `key` with `mods`.
    fn bytes(key: Key, mods: ModMask) -> Vec<u8> {
        let sent = Rc::new(RefCell::new(Vec::new()));
        let mut vt = VTerm::new(24, 80);
        vt.set_utf8(true);
        vt.obtain_state();
        let sink = Rc::clone(&sent);
        vt.set_output(OutputSink::Rust(Box::new(move |out: &[u8]| {
            sink.borrow_mut().extend_from_slice(out);
        })));
        send_key(&mut vt, key, mods);
        sent.take()
    }

    const NONE: ModMask = ModMask::NONE;
    const A: Key = Key::ascii(b'A');
    const LOWER_A: Key = Key::ascii(b'a');
    const E_ACUTE: Key = Key::from_char('\u{e9}');

    #[test]
    fn an_ordinary_character_is_no_named_key() {
        assert_eq!(
            converted(LOWER_A, NONE),
            (VTERM_KEY_NONE, VTERM_MOD_NONE, LOWER_A)
        );
        assert_eq!(
            converted(E_ACUTE, NONE),
            (VTERM_KEY_NONE, VTERM_MOD_NONE, E_ACUTE)
        );
    }

    #[test]
    fn control_lowercases_a_letter_unless_shift_is_held() {
        assert_eq!(
            converted(A, ModMask::CTRL),
            (VTERM_KEY_NONE, VTERM_MOD_CTRL, LOWER_A)
        );
        assert_eq!(
            converted(A, ModMask::CTRL | ModMask::SHIFT),
            (VTERM_KEY_NONE, VTERM_MOD_CTRL | VTERM_MOD_SHIFT, A)
        );
        assert_eq!(
            converted(LOWER_A, ModMask::ALT),
            (VTERM_KEY_NONE, VTERM_MOD_ALT, LOWER_A)
        );
    }

    #[test]
    fn the_raw_control_bytes_are_named_keys() {
        assert_eq!(converted(Key::TAB, NONE).0, VTERM_KEY_TAB);
        assert_eq!(converted(Key::CAR, NONE).0, VTERM_KEY_ENTER);
        assert_eq!(converted(Key::ESC, NONE).0, VTERM_KEY_ESCAPE);
        assert_eq!(
            converted(Key::from(SpecialKey::Bs), NONE).0,
            VTERM_KEY_BACKSPACE
        );
        assert_eq!(converted(Key::from(SpecialKey::Tab), NONE).0, VTERM_KEY_TAB);
    }

    #[test]
    fn a_shifted_key_code_is_split_into_the_key_and_shift() {
        assert_eq!(
            converted(Key::from(SpecialKey::SUp), NONE),
            (VTERM_KEY_UP, VTERM_MOD_SHIFT, Key::from(SpecialKey::Up))
        );
        assert_eq!(
            converted(Key::from(SpecialKey::CLeft), NONE),
            (VTERM_KEY_LEFT, VTERM_MOD_CTRL, Key::from(SpecialKey::Left))
        );
        assert_eq!(
            converted(Key::from(SpecialKey::STab), NONE),
            (VTERM_KEY_TAB, VTERM_MOD_SHIFT, Key::from(SpecialKey::Tab))
        );
    }

    #[test]
    fn both_faces_of_a_keypad_key_reach_the_keypad() {
        for (number, cursor, want) in [
            (SpecialKey::K0, SpecialKey::Kins, VTERM_KEY_KP_0),
            (SpecialKey::K1, SpecialKey::Kend, VTERM_KEY_KP_1),
            (SpecialKey::K5, SpecialKey::Korigin, VTERM_KEY_KP_5),
            (SpecialKey::K9, SpecialKey::Kpageup, VTERM_KEY_KP_9),
            (SpecialKey::Kpoint, SpecialKey::Kdel, VTERM_KEY_KP_PERIOD),
        ] {
            assert_eq!(converted(Key::from(number), NONE).0, want);
            assert_eq!(converted(Key::from(cursor), NONE).0, want);
        }
        assert_eq!(
            converted(Key::from(SpecialKey::Kenter), NONE).0,
            VTERM_KEY_KP_ENTER
        );
    }

    #[test]
    fn function_keys_count_up_from_function_zero() {
        assert_eq!(
            converted(Key::from(SpecialKey::F1), NONE).0,
            function_key(1)
        );
        assert_eq!(
            converted(Key::from(SpecialKey::F12), NONE).0,
            function_key(12)
        );
        assert_eq!(
            converted(Key::from(SpecialKey::F63), NONE).0,
            function_key(63)
        );
    }

    #[test]
    fn keys_become_the_bytes_a_terminal_sends() {
        let rows: &[(Key, ModMask, &[u8])] = &[
            (LOWER_A, NONE, b"a"),
            (E_ACUTE, NONE, "\u{e9}".as_bytes()),
            (Key::CTRL_A, NONE, b"\x01"),
            (LOWER_A, ModMask::ALT, b"\x1ba"),
            (A, ModMask::CTRL, b"\x01"),
            (Key::from(SpecialKey::Up), NONE, b"\x1b[A"),
            (Key::from(SpecialKey::SUp), NONE, b"\x1b[1;2A"),
            (Key::from(SpecialKey::Up), ModMask::CTRL, b"\x1b[1;5A"),
            (Key::from(SpecialKey::Del), NONE, b"\x1b[3~"),
            (Key::from(SpecialKey::F1), NONE, b"\x1bOP"),
            (Key::from(SpecialKey::F5), NONE, b"\x1b[15~"),
            (Key::from(SpecialKey::STab), NONE, b"\x1b[Z"),
            (Key::from(SpecialKey::Bs), NONE, b"\x7f"),
            (Key::CAR, NONE, b"\r"),
            (Key::ESC, NONE, b"\x1b"),
        ];
        for &(key, mods, want) in rows {
            assert_eq!(
                bytes(key, mods),
                want,
                "key {key:?} mods {:#x}",
                mods.byte()
            );
        }
    }

    #[test]
    fn the_nul_extra_becomes_nul_and_other_extras_are_dropped() {
        assert_eq!(bytes(Key::from(SpecialKey::Zero), NONE), b"\0");
        assert_eq!(bytes(Key::from(SpecialKey::Ignore), NONE), b"");
    }
}
