//! Mouse reporting: turning pointer motion and button events into the
//! escape sequences the host asked for.
//!
//! Which events are reported at all is `vterm/state.rs`'s business (the
//! `mouse_flags` it sets from DECSM 1000/1002/1003); this module only decides
//! how a reportable event is spelled, in whichever of the four protocols the
//! host selected.
//!
//! The two entry points keep their C ABI — the unit specs call them through
//! LuaJIT's FFI — but everything they call is safe and pointer-free.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::types::{VTerm, VTermModifier, VTermState};
use crate::src::nvim::vterm::output::EscapeSeq;
use crate::src::nvim::vterm::vterm::vterm_push_output_bytes;
use core::ffi::{c_char, c_int, c_uint};
use core::fmt::Write;

/// The host wants motion reported while a button is held.
const MOUSE_WANT_DRAG: c_int = 0x2;
/// The host wants motion reported unconditionally.
const MOUSE_WANT_MOVE: c_int = 0x4;

/// The original xterm protocol: one byte each for button, column and row,
/// biased so they land in the printable range.
const MOUSE_X10: c_uint = 0;
/// X10 with the three bytes UTF-8 encoded, lifting the 223-column ceiling.
const MOUSE_UTF8: c_uint = 1;
/// Decimal fields plus a final `M`/`m` distinguishing press from release.
const MOUSE_SGR: c_uint = 2;
/// rxvt's decimal variant, which cannot distinguish which button was released.
const MOUSE_RXVT: c_uint = 3;

/// Spell one mouse report, or `None` when the selected protocol cannot carry
/// it.
///
/// `code` is the protocol's button encoding before modifiers are folded in;
/// `col` and `row` are zero-based.
fn encode_mouse(
    protocol: c_uint,
    ctrl8bit: bool,
    code: c_int,
    pressed: bool,
    modifiers: VTermModifier,
    col: c_int,
    row: c_int,
) -> Option<EscapeSeq> {
    // The modifier bits sit above the two low button bits in every protocol.
    let modifiers = (modifiers as c_int) << 2;
    // Only SGR can say *which* button was released, so the others report a
    // release as the "no button" code.
    let released_as_3 = if pressed { code } else { 3 };

    let mut seq = EscapeSeq::csi(ctrl8bit);
    match protocol {
        MOUSE_X10 => {
            let code = released_as_3;
            // The high-numbered buttons don't fit in a byte; drop them
            // rather than report the wrong one.
            if code & 0x80 != 0 {
                return None;
            }
            seq.push(b'M');
            seq.push(((code | modifiers) + 0x20) as u8);
            seq.push((col.min(0xff - 0x21) + 0x21) as u8);
            seq.push((row.min(0xff - 0x21) + 0x21) as u8);
        }
        MOUSE_UTF8 => {
            seq.push(b'M');
            seq.push_utf8((released_as_3 | modifiers) + 0x20);
            seq.push_utf8(col + 0x21);
            seq.push_utf8(row + 0x21);
        }
        MOUSE_SGR => {
            let final_byte = if pressed { 'M' } else { 'm' };
            let _ = write!(
                seq,
                "<{};{};{}{}",
                code | modifiers,
                col + 1,
                row + 1,
                final_byte
            );
        }
        MOUSE_RXVT => {
            let _ = write!(
                seq,
                "{};{};{}M",
                released_as_3 | modifiers,
                col + 1,
                row + 1
            );
        }
        _ => return None,
    }
    Some(seq)
}

/// Which button a mouse-motion report should name, given the buttons held.
///
/// The lowest held button wins; buttons 4-7 are the wheel, which has no
/// drag state to report.
fn drag_code(mouse_buttons: c_int) -> Option<c_int> {
    if mouse_buttons == 0 {
        // Motion with nothing held reports the "no button" code.
        return Some(3 + 0x20);
    }
    let button = mouse_buttons.trailing_zeros() as c_int + 1;
    match button {
        1..=3 => Some(button - 1 + 0x20),
        8..=11 => Some(button - 8 + 0x80 + 0x20),
        _ => None,
    }
}

/// The button encoding for a press or release of `button` (1-based).
fn button_code(button: c_int) -> Option<c_int> {
    match button {
        1..=3 => Some(button - 1),
        4..=7 => Some(button - 4 + 0x40),
        8..=11 => Some(button - 8 + 0x80),
        _ => None,
    }
}

/// Writes one mouse report back to the host, if the protocol could spell it.
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
pub unsafe extern "C" fn vterm_mouse_move(
    vt: *mut VTerm,
    row: c_int,
    col: c_int,
    mod_0: VTermModifier,
) {
    // SAFETY: the caller hands over a live terminal that has a state.
    let state: &mut VTermState = unsafe { &mut *(*vt).state };
    if col == state.mouse_col && row == state.mouse_row {
        return;
    }
    state.mouse_col = col;
    state.mouse_row = row;

    let dragging = state.mouse_flags & MOUSE_WANT_DRAG != 0 && state.mouse_buttons != 0;
    if !dragging && state.mouse_flags & MOUSE_WANT_MOVE == 0 {
        return;
    }
    let Some(code) = drag_code(state.mouse_buttons) else {
        return;
    };
    // SAFETY: `vt` is that same live terminal.
    let ctrl8bit = unsafe { (*vt).mode }.ctrl8bit() != 0;
    let report = encode_mouse(state.mouse_protocol, ctrl8bit, code, true, mod_0, col, row);
    // SAFETY: as above.
    unsafe { send(vt, report) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_mouse_button(
    vt: *mut VTerm,
    button: c_int,
    pressed: bool,
    mod_0: VTermModifier,
) {
    // SAFETY: the caller hands over a live terminal that has a state.
    let state: &mut VTermState = unsafe { &mut *(*vt).state };
    let old_buttons = state.mouse_buttons;
    // The wheel (4-7) has no held state, so it never touches the mask.
    if matches!(button, 1..=3 | 8..=11) {
        let bit = 1 << (button - 1);
        if pressed {
            state.mouse_buttons |= bit;
        } else {
            state.mouse_buttons &= !bit;
        }
    }
    // A press of an already-held button, or a release of one that wasn't
    // held, is not an event — except for the wheel, whose every click is.
    if state.mouse_buttons == old_buttons && !matches!(button, 4..=7) {
        return;
    }
    if state.mouse_flags == 0 {
        return;
    }
    let Some(code) = button_code(button) else {
        return;
    };
    // SAFETY: `vt` is that same live terminal.
    let ctrl8bit = unsafe { (*vt).mode }.ctrl8bit() != 0;
    let report = encode_mouse(
        state.mouse_protocol,
        ctrl8bit,
        code,
        pressed,
        mod_0,
        state.mouse_col,
        state.mouse_row,
    );
    // SAFETY: as above.
    unsafe { send(vt, report) };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(protocol: c_uint, code: c_int, pressed: bool, mod_0: VTermModifier) -> Vec<u8> {
        encode_mouse(protocol, false, code, pressed, mod_0, 10, 20)
            .and_then(|seq| seq.finish().map(<[u8]>::to_vec))
            .unwrap_or_default()
    }

    #[test]
    fn x10_biases_every_field_into_the_printable_range() {
        assert_eq!(encode(MOUSE_X10, 0, true, 0), b"\x1b[M\x20\x2b\x35");
        // A release loses the button number.
        assert_eq!(encode(MOUSE_X10, 2, false, 0), b"\x1b[M\x23\x2b\x35");
        // Ctrl is bit 4 of the button byte.
        assert_eq!(encode(MOUSE_X10, 0, true, 4), b"\x1b[M\x30\x2b\x35");
    }

    #[test]
    fn x10_clamps_out_of_range_coordinates_and_drops_high_buttons() {
        let seq = encode_mouse(MOUSE_X10, false, 0, true, 0, 300, 300).unwrap();
        assert_eq!(seq.finish(), Some(&b"\x1b[M\x20\xff\xff"[..]));
        assert!(encode_mouse(MOUSE_X10, false, 0x80, true, 0, 1, 1).is_none());
    }

    #[test]
    fn utf8_protocol_escapes_the_byte_ceiling() {
        let seq = encode_mouse(MOUSE_UTF8, false, 0, true, 0, 300, 300).unwrap();
        assert_eq!(seq.finish(), Some("\x1b[M\u{20}\u{14d}\u{14d}".as_bytes()));
    }

    #[test]
    fn sgr_reports_one_based_coordinates_and_the_release_button() {
        assert_eq!(encode(MOUSE_SGR, 2, true, 0), b"\x1b[<2;11;21M");
        assert_eq!(encode(MOUSE_SGR, 2, false, 0), b"\x1b[<2;11;21m");
    }

    #[test]
    fn rxvt_cannot_name_the_released_button() {
        assert_eq!(encode(MOUSE_RXVT, 2, true, 0), b"\x1b[2;11;21M");
        assert_eq!(encode(MOUSE_RXVT, 2, false, 0), b"\x1b[3;11;21M");
    }

    #[test]
    fn eight_bit_hosts_get_a_bare_csi() {
        let seq = encode_mouse(MOUSE_SGR, true, 0, true, 0, 0, 0).unwrap();
        assert_eq!(seq.finish(), Some(&b"\x9b<0;1;1M"[..]));
    }

    #[test]
    fn drag_reports_the_lowest_held_button() {
        assert_eq!(drag_code(0), Some(3 + 0x20));
        assert_eq!(drag_code(0b001), Some(0x20));
        assert_eq!(drag_code(0b110), Some(0x21));
        assert_eq!(drag_code(1 << 7), Some(0x80 + 0x20));
        // The wheel is never held, so a stuck wheel bit reports nothing.
        assert_eq!(drag_code(1 << 3), None);
    }

    #[test]
    fn button_codes_partition_click_wheel_and_extended() {
        assert_eq!(button_code(1), Some(0));
        assert_eq!(button_code(3), Some(2));
        assert_eq!(button_code(4), Some(0x40));
        assert_eq!(button_code(7), Some(0x43));
        assert_eq!(button_code(8), Some(0x80));
        assert_eq!(button_code(11), Some(0x83));
        assert_eq!(button_code(0), None);
        assert_eq!(button_code(12), None);
    }
}
