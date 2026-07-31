//! Device Control Strings, and the other control strings the terminal only
//! ever passes on.
//!
//! The one DCS this terminal answers is DECRQSS, "report the setting this
//! control sequence would have made", which is how a program reads back the
//! pen, the margins, the cursor shape and the protection attribute.

#![forbid(unsafe_code)]

use core::ffi::{c_char, c_int, c_long};
use core::fmt::Write;

use crate::src::nvim::types::{VTermState, VTermStringFragment};
use crate::src::nvim::vterm::output::EscapeSeq;
use crate::src::nvim::vterm::pen::{CSI_ARG_FLAG_MORE, CSI_ARG_MASK, pen_sgr_params};
use crate::src::nvim::vterm::state::fragment_bytes;
use crate::src::nvim::vterm::vterm::{
    VTERM_PROP_CURSORSHAPE_BAR_LEFT, VTERM_PROP_CURSORSHAPE_BLOCK, VTERM_PROP_CURSORSHAPE_UNDERLINE,
};

/// Handles a device control string, reporting whether it was recognised.
pub(super) fn device_control(
    state: &mut VTermState,
    command: &[u8],
    frag: VTermStringFragment,
) -> bool {
    if command != b"$q" {
        return false;
    }
    request_status_string(state, frag);
    true
}

/// DECRQSS. The request names a control sequence in up to three bytes, which
/// may themselves arrive split across fragments, so they are gathered in the
/// state until the string ends.
fn request_status_string(state: &mut VTermState, frag: VTermStringFragment) {
    let mut request = if frag.initial() {
        [0 as c_char; 4]
    } else {
        state.decrqss()
    };
    let mut len = request.iter().take(3).take_while(|&&b| b != 0).count();
    for &byte in fragment_bytes(&frag).iter().take(3 - len) {
        request[len] = byte as c_char;
        len += 1;
    }
    request[len] = 0;
    state.set_decrqss(request);

    if !frag.final_0() {
        return;
    }

    let ctrl8bit = state.ctrl8bit();
    let mut seq = EscapeSeq::dcs(ctrl8bit);
    let named = [request[0] as u8, request[1] as u8, request[2] as u8];
    match named {
        [b'm', 0, 0] => report_pen(state, &mut seq),
        [b'r', 0, 0] => {
            let _ = write!(
                seq,
                "1$r{};{}r",
                state.scrollregion_top + 1,
                state.scroll_bottom()
            );
        }
        [b's', 0, 0] => {
            let _ = write!(
                seq,
                "1$r{};{}s",
                state.scroll_left() + 1,
                state.scroll_right()
            );
        }
        [b' ', b'q', 0] => {
            // DECSCUSR numbers the shapes in pairs, steady then blinking.
            let mut reply = match state.mode.cursor_shape() as c_int {
                VTERM_PROP_CURSORSHAPE_BLOCK => 2,
                VTERM_PROP_CURSORSHAPE_UNDERLINE => 4,
                VTERM_PROP_CURSORSHAPE_BAR_LEFT => 6,
                _ => 0,
            };
            if state.mode.cursor_blink() != 0 {
                reply -= 1;
            }
            let _ = write!(seq, "1$r{reply} q");
        }
        [b'"', b'q', 0] => {
            let protection = if state.protected_cell() != 0 { 1 } else { 2 };
            let _ = write!(seq, "1$r{protection}\"q");
        }
        // Anything else: "no, that is not a setting I have".
        _ => {
            let _ = write!(seq, "0$r");
        }
    }
    seq.terminate(ctrl8bit);
    state.reply(&seq);
}

/// The SGR parameters that would reproduce the current pen, colon-separated
/// where one parameter carries sub-parameters.
fn report_pen(state: &VTermState, seq: &mut EscapeSeq) {
    let params = pen_sgr_params(&state.pen);
    let _ = write!(seq, "1$r");
    for (i, &param) in params.iter().enumerate() {
        let separator = if i + 1 == params.len() {
            ""
        } else if param & CSI_ARG_FLAG_MORE as c_long != 0 {
            ":"
        } else {
            ";"
        };
        let _ = write!(seq, "{}{separator}", param & CSI_ARG_MASK as c_long);
    }
    let _ = write!(seq, "m");
}
