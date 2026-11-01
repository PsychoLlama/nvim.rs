//! Working out what the terminal can do, and asking it to do it.
//!
//! A terminal's terminfo entry says what it claims; the modes and queries
//! here find out what it actually has, at the cost of a round trip. Every
//! answer arrives later, through the input layer, as
//! [`tui_handle_term_mode`] or one of the device-attribute callbacks — which
//! is why the requests are fire-and-forget and the state they set up is on
//! the `TUIData` rather than returned.
//!
//! Also here: the correction libtermkey needs for the backspace key, which
//! is a negotiation with the line discipline rather than the terminal.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::log::{LOGLVL_DBG, LOGLVL_WRN, logmsg};
use crate::message_fmt::msg_bytes_opt;
use crate::startup::nvim_testing;
use crate::tui::output::{flush, out, out_fmt};
use crate::tui::terminfo::caps::kTerm_set_underline_style;
use crate::types::{KeyEncoding, TUIData, TermMode, TermModeState};
use core::ffi::CStr;

/// DEC private modes the TUI asks about or sets. The numbers are the
/// terminal's own.
pub(crate) const LEFT_AND_RIGHT_MARGINS: TermMode = 69;
pub(crate) const MOUSE_BUTTON_EVENT: TermMode = 1002;
pub(crate) const MOUSE_ANY_EVENT: TermMode = 1003;
pub(crate) const MOUSE_SGR_EXT: TermMode = 1006;
pub(crate) const BRACKETED_PASTE: TermMode = 2004;
pub(crate) const SYNCHRONIZED_OUTPUT: TermMode = 2026;
pub(crate) const GRAPHEME_CLUSTERS: TermMode = 2027;
pub(crate) const THEME_UPDATES: TermMode = 2031;
pub(crate) const RESIZE_EVENTS: TermMode = 2048;

/// A DECRPM reply's state field.
const MODE_NOT_RECOGNIZED: TermModeState = 0;
const MODE_SET: TermModeState = 1;
const MODE_RESET: TermModeState = 2;
const MODE_PERMANENTLY_SET: TermModeState = 3;
const MODE_PERMANENTLY_RESET: TermModeState = 4;

/// The two characters that can mean "erase", one of which the terminal
/// sends and the other of which the editor must be told about.
const DEL: u8 = 0x7f;
const CTRL_H: u8 = 8;

/// Ask the terminal whether it has `mode` (DECRQM).
pub(crate) fn tui_request_term_mode(tui: &mut TUIData, mode: TermMode) {
    out_fmt(tui, format_args!("\x1b[?{mode}$p"));
}

/// Turn `mode` on or off (DECSET/DECRST).
pub(crate) fn tui_set_term_mode(tui: &mut TUIData, mode: TermMode, set: bool) {
    let letter = if set { 'h' } else { 'l' };
    out_fmt(tui, format_args!("\x1b[?{mode}{letter}"));
}

/// Act on the terminal's answer to a mode query.
///
/// A mode the terminal has but has not turned on is turned on here; one it
/// does not recognise leaves the corresponding feature switched off.
pub fn tui_handle_term_mode(tui: &mut TUIData, mode: TermMode, state: TermModeState) {
    let is_set = match state {
        MODE_NOT_RECOGNIZED | MODE_PERMANENTLY_RESET => {
            log_mode("unavailable", mode, state);
            return;
        }
        MODE_SET | MODE_PERMANENTLY_SET => true,
        MODE_RESET => false,
        _ => return,
    };
    log_mode("detected", mode, state);
    match mode {
        SYNCHRONIZED_OUTPUT => tui.has_sync_mode = true,
        GRAPHEME_CLUSTERS if !is_set => {
            tui_set_term_mode(tui, mode, true);
            tui.modes.set_grapheme_clusters(true);
        }
        THEME_UPDATES if !is_set => {
            tui_set_term_mode(tui, mode, true);
            tui.modes.set_theme_updates(true);
        }
        RESIZE_EVENTS => {
            if !is_set {
                tui_set_term_mode(tui, mode, true);
                tui.modes.set_resize_events(true);
            }
            tui.resize_events_enabled = true;
        }
        LEFT_AND_RIGHT_MARGINS => tui.has_left_and_right_margin_mode = true,
        _ => {}
    }
}

/// Log a mode query's outcome, unless a test is watching the messages.
fn log_mode(what: &str, mode: TermMode, state: TermModeState) {
    if nvim_testing.get() {
        return;
    }
    logmsg!(
        LOGLVL_WRN,
        c"tui_handle_term_mode",
        0,
        "TUI: terminal mode {mode} {what}, state {state}"
    );
}

// ------------------------------------------------------------ capability queries

/// Ask whether the terminal understands styled underlines, by setting one
/// and reading the attributes back (DECRQSS).
pub(crate) fn tui_query_extended_underline(tui: &mut TUIData) {
    out(tui, b"\x1b[0m\x1b[4:3m\x1bP$qm\x1b\\");
    // The query left the terminal's attributes somewhere unknown.
    tui.print_attr_id = -1;
}

/// Record that this terminal can style and colour underlines.
pub fn tui_enable_extended_underline(tui: &mut TUIData) {
    if !tui.ti.has(kTerm_set_underline_style) {
        tui.ti.set_def(kTerm_set_underline_style, c"\x1b[4:%p1%dm");
    }
    tui.can_set_underline_color = true;
}

/// Ask whether the terminal speaks the kitty keyboard protocol, and set the
/// key encoding once it answers.
pub(crate) fn tui_query_kitty_keyboard(tui: &mut TUIData) {
    tui.input.callbacks.primary_device_attr = Some(tui_set_key_encoding);
    out(tui, b"\x1b[?u\x1b[c");
}

/// Ask the terminal to send keys in the encoding the input layer chose.
pub fn tui_set_key_encoding(tui: &mut TUIData) {
    match tui.input.key_encoding {
        KEY_ENCODING_KITTY => out(tui, b"\x1b[>3u"),
        KEY_ENCODING_XTERM => out(tui, b"\x1b[>4;2m"),
        _ => {}
    }
}

/// Put the key encoding back the way the terminal had it.
pub(crate) fn tui_reset_key_encoding(tui: &mut TUIData) {
    match tui.input.key_encoding {
        KEY_ENCODING_KITTY => out(tui, b"\x1b[<u"),
        KEY_ENCODING_XTERM => out(tui, b"\x1b[>4;0m"),
        _ => {}
    }
}

const KEY_ENCODING_KITTY: KeyEncoding = 1;
const KEY_ENCODING_XTERM: KeyEncoding = 2;

/// Ask the terminal for its background colour, so `'background'` can follow
/// it. The trailing DSR is what guarantees an answer either way.
pub(crate) fn tui_query_bg_color_noflush(tui: &mut TUIData) {
    out(tui, b"\x1b]11;?\x07\x1b[5n");
}

/// [`tui_query_bg_color_noflush`], sent right away.
pub fn tui_query_bg_color(tui: &mut TUIData) {
    tui_query_bg_color_noflush(tui);
    flush(tui);
}

// -------------------------------------------------------------- the backspace

/// Correct libtermkey's idea of the backspace and delete keys.
///
/// terminfo describes what the terminal *sends*, but the line discipline may
/// have been told something else, and the two disagreeing is why backspace
/// famously stops working. `erase`, the line discipline's erase character
/// (0 when it names none), wins; the other key of the pair gets whichever of
/// DEL and Ctrl-H is left over. Mouse reporting is refused outright: the TUI
/// decodes mouse input itself.
///
/// Given a capability's `name` and the `value` the terminal's description
/// holds for it, answers the sequence termkey is to register instead.
pub(crate) fn tui_tk_ti_getstr(erase: u8, name: &CStr, value: Option<&[u8]>) -> Option<Vec<u8>> {
    // The erase character as the one-character string it is compared and
    // registered as; none at all is the empty string.
    let erase_str: &[u8] = if erase == 0 { b"" } else { &[erase] };
    match name.to_bytes() {
        b"key_backspace" => {
            log_termkey("kbs", value);
            if erase != 0 {
                return Some(erase_str.to_vec());
            }
        }
        b"key_dc" => {
            log_termkey("kdch1", value);
            if value == Some(erase_str) {
                // Both keys would send the same thing: give this one the
                // other member of the pair.
                return Some(vec![if erase == DEL { CTRL_H } else { DEL }]);
            }
        }
        b"key_mouse" => {
            log_termkey("kmous", value);
            return None;
        }
        _ => {}
    }
    value.map(<[u8]>::to_vec)
}

/// Log a capability libtermkey asked for.
fn log_termkey(capability: &str, value: Option<&[u8]>) {
    let value = msg_bytes_opt(value);
    logmsg!(
        LOGLVL_DBG,
        c"tui_tk_ti_getstr",
        0,
        "libtermkey:{capability}={value}"
    );
}
