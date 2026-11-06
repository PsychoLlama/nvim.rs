#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

//! Packing of the terminal's report keys — mouse, cursor position and mode —
//! into the four bytes `TermKeyKey::code` sets aside for them.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

use crate::tui::termkey::termkey::{
    TERMKEY_MOUSE_DRAG, TERMKEY_MOUSE_PRESS, TERMKEY_MOUSE_RELEASE, TERMKEY_MOUSE_UNKNOWN,
};
use crate::types::TermKeyMouseEvent;
use core::ffi::c_int;

/// The report payload: `TermKeyKey::code::mouse`.
pub type Payload = [u8; 4];

/// The low byte of `n`, as the C's `(char)` store keeps it.
pub(crate) fn low_byte(n: c_int) -> u8 {
    n.to_le_bytes()[0]
}

/// Bit in the fourth byte marking an SGR release (`CSI < ... m`), which is
/// otherwise indistinguishable from the press that shares its button code.
const SGR_RELEASE: u8 = 0x80;

/// Pack a screen position. Eleven bits of line and twelve of column fit in
/// bytes 1-3, so anything past that is clamped rather than wrapped.
///
/// Two upstream slips, neither of which any caller could have noticed:
/// the parameters were named the other way round — its `line` was written where
/// its reader looked for the column, and every caller passed them in this order
/// — and it packed only two of the line's three high bits although the reader
/// unpacked three, so a line past 1023 came back truncated (line 1024 read as
/// line 0) despite the clamp promising 2047.
pub fn pack_position(payload: &mut Payload, col: c_int, line: c_int) {
    let col = col.min(0xfff);
    let line = line.min(0x7ff);
    payload[1] = low_byte(col);
    payload[2] = low_byte(line);
    payload[3] = low_byte((col & 0xf00) >> 8 | (line & 0x700) >> 4);
}

/// Unpack a screen position as (line, column).
pub fn unpack_position(payload: &Payload) -> (c_int, c_int) {
    let high = c_int::from(payload[3]);
    let col = c_int::from(payload[1]) | (high & 0x0f) << 8;
    let line = c_int::from(payload[2]) | (high & 0x70) << 4;
    (line, col)
}

/// Decode a mouse report's button code into (event, button). Button 0 means
/// the event carries none, as for a release under the X10 protocol.
pub fn decode_mouse(payload: &Payload) -> (TermKeyMouseEvent, c_int) {
    let raw = c_int::from(payload[0]);
    let dragging = raw & 0x20 != 0;
    let moving = |dragging: bool| {
        if dragging {
            TERMKEY_MOUSE_DRAG
        } else {
            TERMKEY_MOUSE_PRESS
        }
    };
    // Clearing bits 2-5 drops the modifiers and the drag bit, leaving the
    // button group in bits 6-7 and the button within it in bits 0-1.
    let code = raw & !0x3c;
    let (mut event, button) = match code {
        // Left, middle, right.
        0..=2 => (moving(dragging), code + 1),
        3 => (TERMKEY_MOUSE_RELEASE, 0),
        // Wheel up/down and horizontal wheel.
        64..=67 => (moving(dragging), code - 64 + 4),
        // The first two extended buttons. Upstream recognises no more, so
        // buttons 10 and 11 report as unknown.
        128..=129 => (moving(dragging), code - 128 + 8),
        _ => (TERMKEY_MOUSE_UNKNOWN, 0),
    };
    if payload[3] & SGR_RELEASE != 0 {
        event = TERMKEY_MOUSE_RELEASE;
    }
    (event, button)
}

/// Mark an SGR mouse report as a release.
pub fn mark_sgr_release(payload: &mut Payload) {
    payload[3] |= SGR_RELEASE;
}

/// Pack a DECRPM mode report: which mode, its value, and the private-mode
/// introducer (`?`, or 0 for an ANSI mode).
pub fn pack_mode(payload: &mut Payload, initial: c_int, mode: c_int, value: c_int) {
    payload[0] = low_byte(initial);
    payload[1] = low_byte(mode >> 8);
    payload[2] = low_byte(mode);
    payload[3] = low_byte(value);
}

/// Unpack a mode report as (initial, mode, value).
pub fn unpack_mode(payload: &Payload) -> (c_int, c_int, c_int) {
    (
        c_int::from(payload[0]),
        c_int::from(payload[1]) << 8 | c_int::from(payload[2]),
        c_int::from(payload[3]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(col: c_int, line: c_int) -> (c_int, c_int) {
        let mut payload: Payload = [0; 4];
        pack_position(&mut payload, col, line);
        unpack_position(&payload)
    }

    #[test]
    fn a_position_survives_the_round_trip() {
        assert_eq!(roundtrip(1, 1), (1, 1));
        assert_eq!(roundtrip(30, 30), (30, 30));
        assert_eq!(roundtrip(500, 300), (300, 500));
    }

    #[test]
    fn positions_past_the_packed_width_clamp() {
        assert_eq!(roundtrip(0x1000, 0x800), (0x7ff, 0xfff));
        assert_eq!(roundtrip(4095, 2047), (2047, 4095));
        assert_eq!(roundtrip(1, 1024), (1024, 1));
    }

    #[test]
    fn the_release_bit_does_not_disturb_the_line() {
        let mut payload: Payload = [0; 4];
        pack_position(&mut payload, 500, 300);
        mark_sgr_release(&mut payload);
        assert_eq!(unpack_position(&payload), (300, 500));
        assert_eq!(decode_mouse(&payload).0, TERMKEY_MOUSE_RELEASE);
    }

    #[test]
    fn button_groups_decode_to_their_numbers() {
        let ev = |code: u8| decode_mouse(&[code, 0, 0, 0]);
        assert_eq!(ev(0), (TERMKEY_MOUSE_PRESS, 1));
        assert_eq!(ev(2), (TERMKEY_MOUSE_PRESS, 3));
        assert_eq!(ev(3), (TERMKEY_MOUSE_RELEASE, 0));
        assert_eq!(ev(0x40), (TERMKEY_MOUSE_PRESS, 4));
        assert_eq!(ev(0x42), (TERMKEY_MOUSE_PRESS, 6));
        assert_eq!(ev(0x80), (TERMKEY_MOUSE_PRESS, 8));
        assert_eq!(ev(0x81), (TERMKEY_MOUSE_PRESS, 9));
    }

    #[test]
    fn the_drag_bit_turns_a_press_into_a_drag_but_not_a_release() {
        assert_eq!(decode_mouse(&[0x20, 0, 0, 0]), (TERMKEY_MOUSE_DRAG, 1));
        assert_eq!(decode_mouse(&[0x23, 0, 0, 0]), (TERMKEY_MOUSE_RELEASE, 0));
    }

    #[test]
    fn upstream_reports_no_button_past_the_ninth() {
        // Codes 0x82/0x83 would be buttons 10 and 11.
        assert_eq!(decode_mouse(&[0x82, 0, 0, 0]), (TERMKEY_MOUSE_UNKNOWN, 0));
    }

    #[test]
    fn a_mode_report_survives_the_round_trip() {
        let mut payload: Payload = [0; 4];
        pack_mode(&mut payload, c_int::from(b'?'), 1, 2);
        assert_eq!(unpack_mode(&payload), (c_int::from(b'?'), 1, 2));

        pack_mode(&mut payload, 0, 4, 1);
        assert_eq!(unpack_mode(&payload), (0, 4, 1));

        // Mode numbers use both bytes.
        pack_mode(&mut payload, 0, 2026, 1);
        assert_eq!(unpack_mode(&payload), (0, 2026, 1));
    }
}
