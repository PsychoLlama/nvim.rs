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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_DBG, LOGLVL_WRN, logmsg_c};
use crate::main::nvim_testing;
use crate::memory::strequal;
use crate::tui::output::{flush, out, out_fmt};
use crate::tui::terminfo::caps::kTerm_set_underline_style;
use crate::types::{KeyEncoding, TUIData, TermInput, TermMode, TermModeState, termios};
use ::libc::tcgetattr;
use core::ffi::{CStr, c_char, c_void};

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

/// The `termios` slot holding the erase character.
const VERASE: usize = 2;
/// The two characters that can mean "erase", one of which the terminal
/// sends and the other of which the editor must be told about.
const DEL: c_char = 0x7f;
const CTRL_H: c_char = 8;

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
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub unsafe fn tui_handle_term_mode(tui: *mut TUIData, mode: TermMode, state: TermModeState) {
    let is_set = match state {
        MODE_NOT_RECOGNIZED | MODE_PERMANENTLY_RESET => {
            log_mode(c"TUI: terminal mode %d unavailable, state %d", mode, state);
            return;
        }
        MODE_SET | MODE_PERMANENTLY_SET => true,
        MODE_RESET => false,
        _ => return,
    };
    log_mode(c"TUI: terminal mode %d detected, state %d", mode, state);
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        match mode {
            SYNCHRONIZED_OUTPUT => (*tui).has_sync_mode = true,
            GRAPHEME_CLUSTERS if !is_set => {
                tui_set_term_mode(&mut *tui, mode, true);
                (*tui).modes.set_grapheme_clusters(true);
            }
            THEME_UPDATES if !is_set => {
                tui_set_term_mode(&mut *tui, mode, true);
                (*tui).modes.set_theme_updates(true);
            }
            RESIZE_EVENTS => {
                if !is_set {
                    tui_set_term_mode(&mut *tui, mode, true);
                    (*tui).modes.set_resize_events(true);
                }
                (*tui).resize_events_enabled = true;
            }
            LEFT_AND_RIGHT_MARGINS => (*tui).has_left_and_right_margin_mode = true,
            _ => {}
        }
    }
}

/// Log a mode query's outcome, unless a test is watching the messages.
fn log_mode(message: &CStr, mode: TermMode, state: TermModeState) {
    if nvim_testing.get() {
        return;
    }
    // SAFETY: `message` holds the two `%d` these arguments fill.
    unsafe {
        logmsg_c!(
            LOGLVL_WRN,
            core::ptr::null(),
            c"tui_handle_term_mode".as_ptr(),
            0,
            true,
            message.as_ptr(),
            mode,
            state,
        );
    }
}

// ------------------------------------------------------------ capability queries

/// Ask whether the terminal understands styled underlines, by setting one
/// and reading the attributes back (DECRQSS).
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub(crate) unsafe fn tui_query_extended_underline(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        out(&mut *tui, b"\x1b[0m\x1b[4:3m\x1bP$qm\x1b\\");
        // The query left the terminal's attributes somewhere unknown.
        (*tui).print_attr_id = -1;
    }
}

/// Record that this terminal can style and colour underlines.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub unsafe fn tui_enable_extended_underline(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`; the capability string is static.
    unsafe {
        if (*tui).ti.defs[kTerm_set_underline_style as usize].is_null() {
            (*tui).ti.defs[kTerm_set_underline_style as usize] = c"\x1b[4:%p1%dm".as_ptr();
        }
        (*tui).can_set_underline_color = true;
    }
}

/// Ask whether the terminal speaks the kitty keyboard protocol, and set the
/// key encoding once it answers.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub(crate) unsafe fn tui_query_kitty_keyboard(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        (*tui).input.callbacks.primary_device_attr = Some(tui_set_key_encoding);
        out(&mut *tui, b"\x1b[?u\x1b[c");
    }
}

/// Ask the terminal to send keys in the encoding the input layer chose.
///
/// # Safety
/// Called by the input layer with the TUI it belongs to.
pub unsafe fn tui_set_key_encoding(tui: *mut TUIData) {
    // SAFETY: the input layer holds this TUI's own pointer.
    unsafe {
        match (*tui).input.key_encoding {
            KEY_ENCODING_KITTY => out(&mut *tui, b"\x1b[>3u"),
            KEY_ENCODING_XTERM => out(&mut *tui, b"\x1b[>4;2m"),
            _ => {}
        }
    }
}

/// Put the key encoding back the way the terminal had it.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub(crate) unsafe fn tui_reset_key_encoding(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        match (*tui).input.key_encoding {
            KEY_ENCODING_KITTY => out(&mut *tui, b"\x1b[<u"),
            KEY_ENCODING_XTERM => out(&mut *tui, b"\x1b[>4;0m"),
            _ => {}
        }
    }
}

const KEY_ENCODING_KITTY: KeyEncoding = 1;
const KEY_ENCODING_XTERM: KeyEncoding = 2;

/// Ask the terminal for its background colour, so `'background'` can follow
/// it. The trailing DSR is what guarantees an answer either way.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub(crate) unsafe fn tui_query_bg_color_noflush(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe { out(&mut *tui, b"\x1b]11;?\x07\x1b[5n") };
}

/// [`tui_query_bg_color_noflush`], sent right away.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub unsafe fn tui_query_bg_color(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        tui_query_bg_color_noflush(tui);
        flush(&mut *tui);
    }
}

// -------------------------------------------------------------- the backspace

/// The terminal's own erase character, as `stty` reports it.
static STTY_ERASE: GlobalCell<[c_char; 2]> = GlobalCell::new([0, 0]);

/// Read the erase character out of the terminal's line discipline.
///
/// # Safety
/// `input` must point to a live [`TermInput`].
unsafe fn tui_get_stty_erase(input: *mut TermInput) -> *const c_char {
    // SAFETY: the caller guarantees `input`; `tcgetattr` fills the termios.
    unsafe {
        let mut t: termios = core::mem::zeroed();
        if tcgetattr((*input).in_fd, &raw mut t) != -1 {
            (*STTY_ERASE.ptr())[0] = t.c_cc[VERASE] as c_char;
            (*STTY_ERASE.ptr())[1] = 0;
            logmsg_c!(
                LOGLVL_DBG,
                core::ptr::null(),
                c"tui_get_stty_erase".as_ptr(),
                0,
                true,
                c"stty/termios:erase=%s".as_ptr(),
                STTY_ERASE.ptr(),
            );
        }
        STTY_ERASE.ptr().cast()
    }
}

/// Correct libtermkey's idea of the backspace and delete keys.
///
/// terminfo describes what the terminal *sends*, but the line discipline may
/// have been told something else, and the two disagreeing is why backspace
/// famously stops working. `stty`'s erase character wins; the other key of
/// the pair gets whichever of DEL and Ctrl-H is left over. Mouse reporting
/// is refused outright: the TUI decodes mouse input itself.
///
/// # Safety
/// Called by libtermkey with the [`TermInput`] as its data.
pub(crate) unsafe extern "C" fn tui_tk_ti_getstr(
    name: *const c_char,
    value: *const c_char,
    data: *mut c_void,
) -> *const c_char {
    /// The erase character, read once on the first capability lookup.
    static ERASE: GlobalCell<*const c_char> = GlobalCell::new(core::ptr::null());

    // SAFETY: libtermkey passes NUL-terminated capability names and values,
    // and the `data` this TUI registered.
    unsafe {
        if ERASE.get().is_null() {
            ERASE.set(tui_get_stty_erase(data.cast::<TermInput>()));
        }
        let erase = ERASE.get();
        if strequal(name, c"key_backspace".as_ptr()) {
            log_termkey(c"libtermkey:kbs=%s", value);
            if *erase != 0 {
                return erase;
            }
        } else if strequal(name, c"key_dc".as_ptr()) {
            log_termkey(c"libtermkey:kdch1=%s", value);
            // A capability of -1 is terminfo's "absent", not a string.
            if !value.is_null()
                && value != core::ptr::with_exposed_provenance::<c_char>(usize::MAX)
                && strequal(erase, value)
            {
                // Both keys would send the same thing: give this one the
                // other member of the pair.
                return if *erase == DEL {
                    CTRL_H_STR.as_ptr()
                } else {
                    DEL_STR.as_ptr()
                };
            }
        } else if strequal(name, c"key_mouse".as_ptr()) {
            log_termkey(c"libtermkey:kmous=%s", value);
            return core::ptr::null();
        }
        value
    }
}

const DEL_STR: [c_char; 2] = [DEL, 0];
const CTRL_H_STR: [c_char; 2] = [CTRL_H, 0];

/// Log a capability libtermkey asked for.
fn log_termkey(message: &CStr, value: *const c_char) {
    // SAFETY: `message` holds the one `%s` `value` fills, and `value` is
    // libtermkey's own NUL-terminated capability string.
    unsafe {
        logmsg_c!(
            LOGLVL_DBG,
            core::ptr::null(),
            c"tui_tk_ti_getstr".as_ptr(),
            0,
            true,
            message.as_ptr(),
            value,
        );
    }
}
