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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::drawscreen::UPD_NOT_VALID;
use crate::getchar::{ins_char_typebuf, ungetchars};
use crate::keycodes::{Ctrl_AT, Ctrl_M, Key, NotAKey};
use crate::main::{KeyTyped, curbuf, curwin, mod_mask, tpf_flags, vgetc_char, vgetc_mod_mask};
use crate::mbyte::{utf_ptr2char, utf_ptr2len};
use crate::mouse::{MousePos, do_mousescroll, find_win_inner};
use crate::r#move::win_col_off;
use crate::ops::clear_oparg;
use crate::options::{
    kOptTpfFlagBS, kOptTpfFlagC0, kOptTpfFlagC1, kOptTpfFlagDEL, kOptTpfFlagESC, kOptTpfFlagFF,
    kOptTpfFlagHT,
};
use crate::types::{String_0, Terminal, VTermKey, VTermModifier, cmdarg_T, oparg_T, size_t};
use crate::vterm::keyboard::{
    vterm_keyboard_end_paste, vterm_keyboard_key, vterm_keyboard_start_paste,
    vterm_keyboard_unichar,
};
use crate::vterm::mouse::{vterm_mouse_button, vterm_mouse_move};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

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

const MOD_MASK_SHIFT: c_int = 0x2;
const MOD_MASK_CTRL: c_int = 0x4;
const MOD_MASK_ALT: c_int = 0x8;

const TAB: c_int = 9;
const ESC: c_int = 27;

/// vterm's name for function key `n`.
const fn function_key(n: c_int) -> VTermKey {
    VTERM_KEY_FUNCTION_0 + n as VTermKey
}

/// `do_mousescroll`'s directions.
const MSCR_DOWN: c_int = 0;
const MSCR_UP: c_int = 1;
const MSCR_LEFT: c_int = -1;
const MSCR_RIGHT: c_int = -2;

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

/// Move the editor's modifier state into vterm's, folding in the modifiers
/// that are baked into the keycode itself.
///
/// Control-uppercase is lowercased first: the editor reports `<C-A>` as
/// Ctrl plus `A`, but a terminal's control codes are computed from the
/// lowercase letter.
fn convert_modifiers(key: &mut c_int, state: &mut VTermModifier) {
    let mods = mod_mask.get();
    if mods & MOD_MASK_SHIFT != 0 {
        *state |= VTERM_MOD_SHIFT;
    }
    if mods & MOD_MASK_CTRL != 0 {
        *state |= VTERM_MOD_CTRL;
        if mods & MOD_MASK_SHIFT == 0 && (b'A' as c_int..=b'Z' as c_int).contains(key) {
            *key += b'a' as c_int - b'A' as c_int;
        }
    }
    if mods & MOD_MASK_ALT != 0 {
        *state |= VTERM_MOD_ALT;
    }
    match Key::try_from(*key) {
        Ok(
            Key::STab
            | Key::SUp
            | Key::SDown
            | Key::SLeft
            | Key::SRight
            | Key::SHome
            | Key::SEnd
            | Key::SF1
            | Key::SF2
            | Key::SF3
            | Key::SF4
            | Key::SF5
            | Key::SF6
            | Key::SF7
            | Key::SF8
            | Key::SF9
            | Key::SF10
            | Key::SF11
            | Key::SF12,
        ) => *state |= VTERM_MOD_SHIFT,
        Ok(Key::CLeft | Key::CRight | Key::CHome | Key::CEnd) => *state |= VTERM_MOD_CTRL,
        _ => {}
    }
}

/// Apply the editor's modifiers to `key`, and name the vterm key it stands
/// for — [`VTERM_KEY_NONE`] if it is an ordinary character rather than a
/// named key.
fn convert_key(key: &mut c_int, state: &mut VTermModifier) -> VTermKey {
    convert_modifiers(key, state);
    match Key::try_from(*key) {
        Ok(Key::Bs) => VTERM_KEY_BACKSPACE,
        Ok(Key::STab) | Err(NotAKey(TAB)) => VTERM_KEY_TAB,
        Err(NotAKey(Ctrl_M)) => VTERM_KEY_ENTER,
        Err(NotAKey(ESC)) => VTERM_KEY_ESCAPE,
        Ok(Key::SUp | Key::Up) => VTERM_KEY_UP,
        Ok(Key::SDown | Key::Down) => VTERM_KEY_DOWN,
        Ok(Key::SLeft | Key::CLeft | Key::Left) => VTERM_KEY_LEFT,
        Ok(Key::SRight | Key::CRight | Key::Right) => VTERM_KEY_RIGHT,
        Ok(Key::Ins) => VTERM_KEY_INS,
        Ok(Key::Del) => VTERM_KEY_DEL,
        Ok(Key::SHome | Key::CHome | Key::Home) => VTERM_KEY_HOME,
        Ok(Key::SEnd | Key::CEnd | Key::End) => VTERM_KEY_END,
        Ok(Key::Pageup) => VTERM_KEY_PAGEUP,
        Ok(Key::Pagedown) => VTERM_KEY_PAGEDOWN,
        // The keypad's keys double as cursor keys when NumLock is off, and
        // the editor reports which face was used; vterm wants the keypad.
        Ok(Key::K0 | Key::Kins) => VTERM_KEY_KP_0,
        Ok(Key::K1 | Key::Kend) => VTERM_KEY_KP_1,
        Ok(Key::K2 | Key::Kdown) => VTERM_KEY_KP_2,
        Ok(Key::K3 | Key::Kpagedown) => VTERM_KEY_KP_3,
        Ok(Key::K4 | Key::Kleft) => VTERM_KEY_KP_4,
        Ok(Key::K5 | Key::Korigin) => VTERM_KEY_KP_5,
        Ok(Key::K6 | Key::Kright) => VTERM_KEY_KP_6,
        Ok(Key::K7 | Key::Khome) => VTERM_KEY_KP_7,
        Ok(Key::K8 | Key::Kup) => VTERM_KEY_KP_8,
        Ok(Key::K9 | Key::Kpageup) => VTERM_KEY_KP_9,
        Ok(Key::Kdel | Key::Kpoint) => VTERM_KEY_KP_PERIOD,
        Ok(Key::Kenter) => VTERM_KEY_KP_ENTER,
        Ok(Key::Kplus) => VTERM_KEY_KP_PLUS,
        Ok(Key::Kminus) => VTERM_KEY_KP_MINUS,
        Ok(Key::Kmultiply) => VTERM_KEY_KP_MULT,
        Ok(Key::Kdivide) => VTERM_KEY_KP_DIVIDE,
        // Shift-F1..F12 are their own keycodes but the same vterm key; the
        // shift went into the modifier set above.
        Ok(Key::SF1 | Key::F1) => function_key(1),
        Ok(Key::SF2 | Key::F2) => function_key(2),
        Ok(Key::SF3 | Key::F3) => function_key(3),
        Ok(Key::SF4 | Key::F4) => function_key(4),
        Ok(Key::SF5 | Key::F5) => function_key(5),
        Ok(Key::SF6 | Key::F6) => function_key(6),
        Ok(Key::SF7 | Key::F7) => function_key(7),
        Ok(Key::SF8 | Key::F8) => function_key(8),
        Ok(Key::SF9 | Key::F9) => function_key(9),
        Ok(Key::SF10 | Key::F10) => function_key(10),
        Ok(Key::SF11 | Key::F11) => function_key(11),
        Ok(Key::SF12 | Key::F12) => function_key(12),
        Ok(Key::F13) => function_key(13),
        Ok(Key::F14) => function_key(14),
        Ok(Key::F15) => function_key(15),
        Ok(Key::F16) => function_key(16),
        Ok(Key::F17) => function_key(17),
        Ok(Key::F18) => function_key(18),
        Ok(Key::F19) => function_key(19),
        Ok(Key::F20) => function_key(20),
        Ok(Key::F21) => function_key(21),
        Ok(Key::F22) => function_key(22),
        Ok(Key::F23) => function_key(23),
        Ok(Key::F24) => function_key(24),
        Ok(Key::F25) => function_key(25),
        Ok(Key::F26) => function_key(26),
        Ok(Key::F27) => function_key(27),
        Ok(Key::F28) => function_key(28),
        Ok(Key::F29) => function_key(29),
        Ok(Key::F30) => function_key(30),
        Ok(Key::F31) => function_key(31),
        Ok(Key::F32) => function_key(32),
        Ok(Key::F33) => function_key(33),
        Ok(Key::F34) => function_key(34),
        Ok(Key::F35) => function_key(35),
        Ok(Key::F36) => function_key(36),
        Ok(Key::F37) => function_key(37),
        Ok(Key::F38) => function_key(38),
        Ok(Key::F39) => function_key(39),
        Ok(Key::F40) => function_key(40),
        Ok(Key::F41) => function_key(41),
        Ok(Key::F42) => function_key(42),
        Ok(Key::F43) => function_key(43),
        Ok(Key::F44) => function_key(44),
        Ok(Key::F45) => function_key(45),
        Ok(Key::F46) => function_key(46),
        Ok(Key::F47) => function_key(47),
        Ok(Key::F48) => function_key(48),
        Ok(Key::F49) => function_key(49),
        Ok(Key::F50) => function_key(50),
        Ok(Key::F51) => function_key(51),
        Ok(Key::F52) => function_key(52),
        Ok(Key::F53) => function_key(53),
        Ok(Key::F54) => function_key(54),
        Ok(Key::F55) => function_key(55),
        Ok(Key::F56) => function_key(56),
        Ok(Key::F57) => function_key(57),
        Ok(Key::F58) => function_key(58),
        Ok(Key::F59) => function_key(59),
        Ok(Key::F60) => function_key(60),
        Ok(Key::F61) => function_key(61),
        Ok(Key::F62) => function_key(62),
        Ok(Key::F63) => function_key(63),
        _ => VTERM_KEY_NONE,
    }
}

/// Send one key the editor read to the child.
///
/// A keycode with no vterm name is a codepoint, unless it is negative — one
/// of the extras vterm has no equivalent for, which is simply dropped.
pub(super) fn terminal_send_key(term: Term, c: c_int) {
    let mut state = VTERM_MOD_NONE;
    // The editor spells NUL as an extra so it survives being a C string;
    // vterm wants the control character back.
    let mut key = if c == Key::Zero.code() { Ctrl_AT } else { c };
    let named = convert_key(&mut key, &mut state);
    let vt = term.vt;
    if named != VTERM_KEY_NONE {
        // SAFETY: the terminal's own emulator.
        unsafe { vterm_keyboard_key(vt, named, state) };
    } else if key >= 0 {
        // SAFETY: as above.
        unsafe { vterm_keyboard_unichar(vt, key as u32, state) };
    }
}

/// Whether `'termpastefilter'` says to drop `c` from a paste.
fn is_filter_char(c: c_int) -> bool {
    let flag = match c {
        0x08 => kOptTpfFlagBS as ::core::ffi::c_uint,
        0x09 => kOptTpfFlagHT as ::core::ffi::c_uint,
        // Newlines are the one thing a paste is guaranteed to want.
        0x0a | 0x0d => return false,
        0x0c => kOptTpfFlagFF as ::core::ffi::c_uint,
        0x1b => kOptTpfFlagESC as ::core::ffi::c_uint,
        0x7f => kOptTpfFlagDEL as ::core::ffi::c_uint,
        _ if c < 0x20 => kOptTpfFlagC0 as ::core::ffi::c_uint,
        _ if (0x80..=0x9f).contains(&c) => kOptTpfFlagC1 as ::core::ffi::c_uint,
        _ => return false,
    };
    tpf_flags.get() & flag != 0
}

/// Open or close a bracketed paste that spans more than one `nvim_paste`
/// call, so the child sees one paste rather than several.
pub(crate) unsafe fn terminal_set_streamed_paste(term: *mut Terminal, streamed: bool) {
    // SAFETY: the caller hands over a live terminal.
    let mut term = unsafe { Term::new(term) };
    if term.streamed_paste != streamed {
        // The current buffer's terminal, not `term`, exactly as the C had
        // it; both callers pass one for the other.
        //
        // SAFETY: `curbuf` is set from startup to exit, and the paste
        // machinery only reaches this while it has a terminal.
        let vt = unsafe { Term::new(Buf::current().terminal) }.vt;
        if streamed {
            // SAFETY: a live emulator.
            unsafe { vterm_keyboard_start_paste(vt) };
        } else {
            // SAFETY: as above.
            unsafe { vterm_keyboard_end_paste(vt) };
        }
    }
    term.streamed_paste = streamed;
}

/// Paste `y_array` into the current terminal `count` times.
///
/// Bracketed unless the paste is already part of a stream, and filtered
/// through `'termpastefilter'` a character at a time.
pub(crate) unsafe fn terminal_paste(count: c_int, y_array: *mut String_0, y_size: size_t) {
    if y_size == 0 {
        return;
    }
    // SAFETY: the caller hands over `y_size` readable strings.
    let lines = unsafe { ::core::slice::from_raw_parts(y_array, y_size) };
    // SAFETY: `curbuf` is set from startup to exit, and a paste only
    // reaches here while it has a terminal.
    let term = unsafe { Term::new(Buf::current().terminal) };
    let (bracket, vt) = (!term.streamed_paste, term.vt);
    if bracket {
        // SAFETY: the terminal's own emulator.
        unsafe { vterm_keyboard_start_paste(vt) };
    }
    let mut filtered: Vec<u8> = Vec::new();
    for _ in 0..count {
        for (index, line) in lines.iter().enumerate() {
            if index != 0 {
                terminal_send(term, b"\n");
            }
            filtered.clear();
            // SAFETY: a register's line is NUL-terminated.
            let bytes = unsafe { CStr::from_ptr(line.data()) }.to_bytes();
            let mut at = 0;
            while at < bytes.len() {
                let src = bytes[at..].as_ptr().cast::<c_char>();
                // SAFETY: the tail of a NUL-terminated line, which is what
                // both of these read; neither can run past the NUL.
                let (len, c) = unsafe { (utf_ptr2len(src) as usize, utf_ptr2char(src)) };
                if !is_filter_char(c) {
                    filtered.extend_from_slice(&bytes[at..at + len]);
                }
                at += len;
            }
            terminal_send(term, &filtered);
        }
    }
    if bracket {
        // SAFETY: the terminal's own emulator.
        unsafe { vterm_keyboard_end_paste(vt) };
    }
}

/// The vterm button a mouse keycode stands for, and whether it is a press.
///
/// Drags count as presses: vterm tracks the button down and turns a move
/// with a button held into a drag report itself.
fn mouse_button(c: c_int) -> Option<(c_int, bool)> {
    match Key::try_from(c) {
        Ok(Key::Leftmouse | Key::Leftdrag) => Some((BUTTON_LEFT, true)),
        Ok(Key::Leftrelease) => Some((BUTTON_LEFT, false)),
        Ok(Key::Middlemouse | Key::Middledrag) => Some((BUTTON_MIDDLE, true)),
        Ok(Key::Middlerelease) => Some((BUTTON_MIDDLE, false)),
        Ok(Key::Rightmouse | Key::Rightdrag) => Some((BUTTON_RIGHT, true)),
        Ok(Key::Rightrelease) => Some((BUTTON_RIGHT, false)),
        Ok(Key::X1mouse | Key::X1drag) => Some((BUTTON_X1, true)),
        Ok(Key::X1release) => Some((BUTTON_X1, false)),
        Ok(Key::X2mouse | Key::X2drag) => Some((BUTTON_X2, true)),
        Ok(Key::X2release) => Some((BUTTON_X2, false)),
        Ok(Key::Mousedown) => Some((BUTTON_WHEEL_UP, true)),
        Ok(Key::Mouseup) => Some((BUTTON_WHEEL_DOWN, true)),
        Ok(Key::Mouseleft) => Some((BUTTON_WHEEL_LEFT, true)),
        Ok(Key::Mouseright) => Some((BUTTON_WHEEL_RIGHT, true)),
        // A bare move: position only, no button.
        Ok(Key::Mousemove) => Some((0, false)),
        _ => None,
    }
}

/// Whether `c` is a mouse event, and so [`send_mouse_event`]'s to deal
/// with rather than the child's.
pub(super) fn is_mouse_key(c: c_int) -> bool {
    mouse_button(c).is_some()
}

/// The scroll direction a wheel keycode asks for, for the editor's own
/// scrolling when the child is not taking mouse events.
fn scroll_direction(c: c_int) -> Option<c_int> {
    match Key::try_from(c) {
        Ok(Key::Mouseup) => Some(MSCR_UP),
        Ok(Key::Mousedown) => Some(MSCR_DOWN),
        Ok(Key::Mouseleft) => Some(MSCR_LEFT),
        Ok(Key::Mouseright) => Some(MSCR_RIGHT),
        _ => None,
    }
}

/// Scroll `mouse_win` as the editor would if the mouse were over an
/// ordinary buffer.
fn scroll_window(mouse_win: Win, key: c_int, direction: c_int) {
    // SAFETY: `curwin` is set from startup to exit.
    let save_curwin = unsafe { Win::current() };
    curwin.set(mouse_win.raw());
    curbuf.set(mouse_win.buffer().raw());

    // SAFETY: all-zeroes is what `clear_oparg` and the command argument
    // start from; every field of both is a scalar or a pointer.
    let (mut oa, mut cap): (oparg_T, cmdarg_T) = unsafe { ::core::mem::zeroed() };
    // SAFETY: an operator argument of this frame's own.
    unsafe { clear_oparg(&raw mut oa) };
    cap.oap = &raw mut oa;
    cap.cmdchar = key;
    cap.arg = direction;
    // SAFETY: a command argument of this frame's own, naming the operator
    // above; it scrolls the window the two globals were just set to.
    unsafe { do_mousescroll(&raw mut cap) };

    // Whatever the scroll left as the current window, which need not be the
    // one it started in.
    //
    // SAFETY: `curwin` is set from startup to exit.
    let mut scrolled = unsafe { Win::current() };
    scrolled.w_redr_status = true;
    curwin.set(save_curwin.raw());
    curbuf.set(save_curwin.buffer().raw());
}

/// Deal with a mouse event while a terminal has focus.
///
/// Returns whether the editor should handle the key itself. Three outcomes:
/// the child gets the event, the editor scrolls the window under the
/// pointer, or the event is pushed back for normal-mode processing (which
/// is what makes clicking out of a terminal work).
pub(super) fn send_mouse_event(term: Term, c: c_int) -> bool {
    let mut pos = MousePos::current();
    let mouse_win = find_win_inner(&mut pos);
    let MousePos { grid, row, col } = pos;

    if let Some(mouse_win) = mouse_win {
        // An external grid is exactly the terminal's window, so the height
        // and width checks below only apply to the shared one.
        //
        // SAFETY: a live window.
        let offset = unsafe { win_col_off(mouse_win.raw()) };
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
            convert_modifiers(&mut { c }, &mut state);
            let vt = term.vt;
            // SAFETY: the terminal's own emulator.
            unsafe { vterm_mouse_move(vt, row, col - offset, state) };
            if button != 0 {
                // SAFETY: as above.
                unsafe { vterm_mouse_button(vt, button, pressed, state) };
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
        if c == Key::Leftrelease.code() && showing_term {
            return false;
        }
    }

    // A bare move is dropped for the same reason.
    if c == Key::Mousemove.code() {
        return false;
    }

    // Hand the event back for normal mode, then rewind the typeahead so
    // that terminal mode ends before it is read.
    let (c, mods) = (vgetc_char.get(), vgetc_mod_mask.get());
    // SAFETY: pushes one key back onto the editor's own typeahead.
    let len = unsafe { ins_char_typebuf(c, mods, true) };
    if KeyTyped.get() {
        ungetchars(len); // rewind over the key just pushed
    }
    true
}
