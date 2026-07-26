//! The terminal's modes: the ANSI ones, the DEC private ones, the kitty
//! key-encoding stack, and the reset that puts them all back.

#![forbid(unsafe_code)]

use core::ffi::c_int;
use core::fmt::Write;

use crate::src::nvim::types::{
    VTermKeyEncodingFlags, VTermKeyEncodingStack, VTermRect, VTermState,
};
use crate::src::nvim::vterm::geometry::{DHL_OFF, DWL_OFF};
use crate::src::nvim::vterm::output::EscapeSeq;
use crate::src::nvim::vterm::state::{
    MOUSE_WANT_CLICK, MOUSE_WANT_DRAG, MOUSE_WANT_MOVE, PenChange,
};
use crate::src::nvim::vterm::text::save_cursor;
use crate::src::nvim::vterm::vterm::{
    MOUSE_RXVT, MOUSE_SGR, MOUSE_UTF8, MOUSE_X10, VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK,
    VTERM_PROP_CURSORSHAPE, VTERM_PROP_CURSORVISIBLE, VTERM_PROP_FOCUSREPORT, VTERM_PROP_MOUSE,
    VTERM_PROP_REVERSE, VTERM_PROP_SYNCOUTPUT, VTERM_PROP_THEMEUPDATES,
};

/// The version this terminal claims to be, for XTVERSION.
const VTERM_VERSION_MAJOR: c_int = 0;
const VTERM_VERSION_MINOR: c_int = 3;

/// The shapes `VTERM_PROP_CURSORSHAPE` can take.
pub const VTERM_PROP_CURSORSHAPE_BLOCK: c_int = 1;
pub const VTERM_PROP_CURSORSHAPE_UNDERLINE: c_int = 2;
pub const VTERM_PROP_CURSORSHAPE_BAR_LEFT: c_int = 3;

/// The pointer-tracking levels `VTERM_PROP_MOUSE` can take.
pub const VTERM_PROP_MOUSE_NONE: c_int = 0;
pub const VTERM_PROP_MOUSE_CLICK: c_int = 1;
pub const VTERM_PROP_MOUSE_DRAG: c_int = 2;
pub const VTERM_PROP_MOUSE_MOVE: c_int = 3;

/// The progressive-enhancement bits of the kitty keyboard protocol.
/// <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement>
pub const KEY_ENCODING_DISAMBIGUATE: c_int = 0x1;
pub const KEY_ENCODING_REPORT_EVENTS: c_int = 0x2;
pub const KEY_ENCODING_REPORT_ALTERNATE: c_int = 0x4;
pub const KEY_ENCODING_REPORT_ALL_KEYS: c_int = 0x8;
pub const KEY_ENCODING_REPORT_ASSOCIATED: c_int = 0x10;

// ---------------------------------------------------------------- ANSI modes

/// `SM` / `RM`: the two ANSI modes this terminal implements.
pub(super) fn set_ansi_mode(state: &mut VTermState, num: c_int, on: bool) {
    match num {
        4 => state.mode.set_insert(on as _),   // IRM - ECMA-48 7.2.10
        20 => state.mode.set_newline(on as _), // LNM - ANSI X3.4-1977
        _ => {}
    }
}

// ----------------------------------------------------------- DEC private modes

/// `CSI ? … h` / `CSI ? … l`: the DEC private modes.
pub(super) fn set_dec_mode(state: &mut VTermState, num: c_int, on: bool) {
    match num {
        1 => state.mode.set_cursor(on as _), // DECCKM - cursor keys
        5 => {
            state.set_termprop_bool(VTERM_PROP_REVERSE, on);
        }
        6 => {
            // DECOM - origin mode. Switching it homes the cursor into
            // whichever origin now applies.
            let oldpos = state.pos;
            state.mode.set_origin(on as _);
            state.pos.row = if on { state.scrollregion_top } else { 0 };
            state.pos.col = if on { state.scroll_left() } else { 0 };
            state.update_cursor(oldpos, true);
        }
        7 => state.mode.set_autowrap(on as _),
        12 => {
            state.set_termprop_bool(VTERM_PROP_CURSORBLINK, on);
        }
        25 => {
            state.set_termprop_bool(VTERM_PROP_CURSORVISIBLE, on);
        }
        69 => {
            // DECVSSM / DECLRMM - left and right margins. Turning them on
            // must clear every line's double-width and double-height marks.
            state.mode.set_leftrightmargin(on as _);
            if on {
                for row in 0..state.rows {
                    state.set_lineinfo(row, true, DWL_OFF, DHL_OFF);
                }
            }
        }
        1000 | 1002 | 1003 => {
            let level = match (on, num) {
                (false, _) => VTERM_PROP_MOUSE_NONE,
                (true, 1000) => VTERM_PROP_MOUSE_CLICK,
                (true, 1002) => VTERM_PROP_MOUSE_DRAG,
                (true, _) => VTERM_PROP_MOUSE_MOVE,
            };
            state.set_termprop_int(VTERM_PROP_MOUSE, level);
        }
        1004 => {
            state.set_termprop_bool(VTERM_PROP_FOCUSREPORT, on);
            state.mode.set_report_focus(on as _);
        }
        1005 => state.mouse_protocol = if on { MOUSE_UTF8 } else { MOUSE_X10 },
        1006 => state.mouse_protocol = if on { MOUSE_SGR } else { MOUSE_X10 },
        1015 => state.mouse_protocol = if on { MOUSE_RXVT } else { MOUSE_X10 },
        1047 => {
            state.set_termprop_bool(VTERM_PROP_ALTSCREEN, on);
        }
        1048 => save_cursor(state, on),
        1049 => {
            state.set_termprop_bool(VTERM_PROP_ALTSCREEN, on);
            save_cursor(state, on);
        }
        2004 => state.mode.set_bracketpaste(on as _),
        2026 => {
            state.set_termprop_bool(VTERM_PROP_SYNCOUTPUT, on);
        }
        2031 => {
            state.set_termprop_bool(VTERM_PROP_THEMEUPDATES, on);
        }
        _ => {}
    }
}

/// `CSI ? … $ p` (DECRQM): report whether a DEC private mode is set. A mode
/// this terminal does not know reports as "not recognised".
pub(super) fn request_dec_mode(state: &mut VTermState, num: c_int) {
    let set = match num {
        1 => Some(state.mode.cursor() != 0),
        5 => Some(state.mode.screen() != 0),
        6 => Some(state.mode.origin() != 0),
        7 => Some(state.mode.autowrap() != 0),
        12 => Some(state.mode.cursor_blink() != 0),
        25 => Some(state.mode.cursor_visible() != 0),
        69 => Some(state.mode.leftrightmargin() != 0),
        1000 => Some(state.mouse_flags == MOUSE_WANT_CLICK),
        1002 => Some(state.mouse_flags == MOUSE_WANT_CLICK | MOUSE_WANT_DRAG),
        1003 => Some(state.mouse_flags == MOUSE_WANT_CLICK | MOUSE_WANT_MOVE),
        1004 => Some(state.mode.report_focus() != 0),
        1005 => Some(state.mouse_protocol == MOUSE_UTF8),
        1006 => Some(state.mouse_protocol == MOUSE_SGR),
        1015 => Some(state.mouse_protocol == MOUSE_RXVT),
        1047 => Some(state.mode.alt_screen() != 0),
        2004 => Some(state.mode.bracketpaste() != 0),
        2026 => Some(state.mode.synchronized_output() != 0),
        2031 => Some(state.mode.theme_updates() != 0),
        _ => None,
    };
    let reply = match set {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    };
    let mut seq = EscapeSeq::csi(state.ctrl8bit());
    let _ = write!(seq, "?{num};{reply}$y");
    state.reply(&seq);
}

/// `CSI > q` (XTVERSION): name the terminal and its version.
pub(super) fn request_version_string(state: &mut VTermState) {
    let ctrl8bit = state.ctrl8bit();
    let mut seq = EscapeSeq::dcs(ctrl8bit);
    let _ = write!(
        seq,
        ">|libvterm({VTERM_VERSION_MAJOR}.{VTERM_VERSION_MINOR})"
    );
    seq.terminate(ctrl8bit);
    state.reply(&seq);
}

// ------------------------------------------------------ key-encoding flags

/// Every enhancement turned off.
fn no_flags() -> VTermKeyEncodingFlags {
    VTermKeyEncodingFlags {
        disambiguate_report_events_report_alternate_report_all_keys_report_associated: [0; 1],
    }
}

/// The flag stack for the screen that is currently showing. The two screens
/// keep separate stacks so that a full-screen program cannot leak its key
/// encoding back to the shell.
fn flag_stack(state: &mut VTermState) -> &mut VTermKeyEncodingStack {
    let screen = state.active_buffer();
    &mut state.key_encoding_stacks[screen]
}

/// `CSI ? u`: report the flags on top of the stack.
pub(super) fn request_key_encoding_flags(state: &mut VTermState) {
    let stack = flag_stack(state);
    assert!(stack.size > 0);
    let flags = stack.items[usize::from(stack.size - 1)];
    let mut reply = 0;
    for (bit, set) in [
        (KEY_ENCODING_DISAMBIGUATE, flags.disambiguate()),
        (KEY_ENCODING_REPORT_EVENTS, flags.report_events()),
        (KEY_ENCODING_REPORT_ALTERNATE, flags.report_alternate()),
        (KEY_ENCODING_REPORT_ALL_KEYS, flags.report_all_keys()),
        (KEY_ENCODING_REPORT_ASSOCIATED, flags.report_associated()),
    ] {
        if set {
            reply |= bit;
        }
    }
    let mut seq = EscapeSeq::csi(state.ctrl8bit());
    let _ = write!(seq, "?{reply}u");
    state.reply(&seq);
}

/// `CSI = u`: replace the flags on top of the stack.
///
/// Mode 1 is meant to set exactly the bits in `arg`, mode 2 to add them and
/// mode 3 to remove them. Upstream builds a fresh set of flags whatever the
/// mode and replaces the top of the stack with it, so 2 and 3 do not really
/// add to or subtract from what was there; a bit `arg` does not name always
/// ends up clear, and only the meaning of a named bit varies.
pub(super) fn set_key_encoding_flags(state: &mut VTermState, arg: c_int, mode: c_int) {
    let set = mode != 3;
    let named = |bit: c_int| arg & bit != 0 && set;
    let mut flags = no_flags();
    flags.set_disambiguate(named(KEY_ENCODING_DISAMBIGUATE));
    flags.set_report_events(named(KEY_ENCODING_REPORT_EVENTS));
    flags.set_report_alternate(named(KEY_ENCODING_REPORT_ALTERNATE));
    flags.set_report_all_keys(named(KEY_ENCODING_REPORT_ALL_KEYS));
    flags.set_report_associated(named(KEY_ENCODING_REPORT_ASSOCIATED));

    let stack = flag_stack(state);
    assert!(stack.size > 0);
    let top = usize::from(stack.size - 1);
    stack.items[top] = flags;
}

/// `CSI > u`: push a new set of flags, evicting the oldest when the stack is
/// already full.
pub(super) fn push_key_encoding_flags(state: &mut VTermState, arg: c_int) {
    let stack = flag_stack(state);
    let depth = stack.items.len();
    assert!(usize::from(stack.size) <= depth);
    if usize::from(stack.size) == depth {
        stack.items.rotate_left(1);
    } else {
        stack.size += 1;
    }
    set_key_encoding_flags(state, arg, 1);
}

/// `CSI < u`: pop `arg` entries. Popping the stack empty resets the flags.
pub(super) fn pop_key_encoding_flags(state: &mut VTermState, arg: c_int) {
    let stack = flag_stack(state);
    if arg >= c_int::from(stack.size) {
        stack.size = 1;
        stack.items[0] = no_flags();
    } else if arg > 0 {
        stack.size -= arg as u8;
    }
}

// ---------------------------------------------------------------------- reset

/// Returns the terminal to its power-on settings. A hard reset also homes the
/// cursor and clears the screen; a soft one (DECSTR) leaves both alone.
pub(super) fn reset(state: &mut VTermState, hard: bool) {
    state.scrollregion_top = 0;
    state.scrollregion_bottom = -1;
    state.scrollregion_left = 0;
    state.scrollregion_right = -1;

    state.mode.set_keypad(0);
    state.mode.set_cursor(0);
    state.mode.set_autowrap(1);
    state.mode.set_insert(0);
    state.mode.set_newline(0);
    state.mode.set_alt_screen(0);
    state.mode.set_origin(0);
    state.mode.set_leftrightmargin(0);
    state.mode.set_bracketpaste(0);
    state.mode.set_report_focus(0);
    state.mouse_flags = 0;
    state.set_ctrl8bit(false);

    state.reset_tabstops();
    for row in 0..state.rows {
        state.set_lineinfo(row, true, DWL_OFF, DHL_OFF);
    }

    state.init_consumer_pen();
    state.change_pen(PenChange::Reset);
    state.reset_charsets();
    state.gl_set = 0;
    state.gr_set = 1;
    state.gsingle_set = 0;
    state.set_protected_cell(0);

    state.set_termprop_bool(VTERM_PROP_CURSORVISIBLE, true);
    state.set_termprop_bool(VTERM_PROP_CURSORBLINK, true);
    state.set_termprop_int(VTERM_PROP_CURSORSHAPE, VTERM_PROP_CURSORSHAPE_BLOCK);

    if hard {
        state.pos.row = 0;
        state.pos.col = 0;
        state.at_phantom = 0;
        let rect = VTermRect {
            start_row: 0,
            end_row: state.rows,
            start_col: 0,
            end_col: state.cols,
        };
        state.erase(rect, false);
    }
}
