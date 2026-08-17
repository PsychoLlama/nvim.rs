//! Operating System Commands, and the clipboard traffic OSC 52 carries.
//!
//! A selection arrives base64-encoded and possibly split across several
//! fragments, so the decoder keeps its partial sextets between calls and
//! hands the consumer one buffer-sized chunk at a time.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]

use core::ffi::{c_char, c_int};

use crate::types::{VTermSelectionMask, VTermState, VTermStringFragment};
use crate::vterm::state::fragment_bytes;
use crate::vterm::vterm::{
    SELECTION_INITIAL, SELECTION_INVALID, SELECTION_QUERY, SELECTION_SELECTED, SELECTION_SET,
    SELECTION_SET_INITIAL, VTERM_PROP_ICONNAME, VTERM_PROP_TITLE, VTERM_SELECTION_CLIPBOARD,
    VTERM_SELECTION_CUT0, VTERM_SELECTION_PRIMARY, VTERM_SELECTION_SECONDARY,
    VTERM_SELECTION_SELECT,
};

/// Builds a string fragment for the consumer's selection callback.
pub(super) fn fragment(
    text: *const c_char,
    len: usize,
    initial: bool,
    last: bool,
) -> VTermStringFragment {
    let mut frag = VTermStringFragment {
        str: text,
        len_initial_final_0: [0; 4],
        terminator: 0,
    };
    frag.set_len(len);
    frag.set_initial(initial);
    frag.set_final_0(last);
    frag
}

/// Handles one Operating System Command. Whether the sequence counts as
/// handled is not this module's call: upstream offers every OSC to the
/// consumer's fallback afterwards and reports only what that made of it.
pub(super) fn osc(state: &mut VTermState, command: c_int, frag: VTermStringFragment) {
    match command {
        0 => {
            state.set_termprop_string(VTERM_PROP_ICONNAME, frag);
            state.set_termprop_string(VTERM_PROP_TITLE, frag);
        }
        1 => {
            state.set_termprop_string(VTERM_PROP_ICONNAME, frag);
        }
        2 => {
            state.set_termprop_string(VTERM_PROP_TITLE, frag);
        }
        52 => {
            if state.selection_enabled() {
                clipboard(state, frag);
            }
        }
        _ => {}
    }
}

/// One base64 digit, or `None` for anything that is not one.
fn unbase64one(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// OSC 52: a selection name, then either `?` to ask for its contents or
/// base64 to set them.
fn clipboard(state: &mut VTermState, frag: VTermStringFragment) {
    let mut progress = state.selection_progress();
    if frag.initial() {
        progress.mask = 0;
        progress.set_state(SELECTION_INITIAL);
        state.set_selection_progress(progress);
    }

    let mut rest = fragment_bytes(&frag);
    // The selection name: any number of buffer letters, ended by a semicolon.
    while progress.state() == SELECTION_INITIAL && !rest.is_empty() {
        match rest[0] {
            b'c' => progress.mask |= VTERM_SELECTION_CLIPBOARD as u16,
            b'p' => progress.mask |= VTERM_SELECTION_PRIMARY as u16,
            b'q' => progress.mask |= VTERM_SELECTION_SECONDARY as u16,
            b's' => progress.mask |= VTERM_SELECTION_SELECT as u16,
            digit @ b'0'..=b'7' => {
                progress.mask |= (VTERM_SELECTION_CUT0 << (digit - b'0')) as u16;
            }
            b';' => {
                progress.set_state(SELECTION_SELECTED);
                if progress.mask == 0 {
                    progress.mask = (VTERM_SELECTION_SELECT | VTERM_SELECTION_CUT0) as u16;
                }
            }
            _ => {}
        }
        rest = &rest[1..];
        state.set_selection_progress(progress);
    }

    let mask = VTermSelectionMask::from(progress.mask);
    if rest.is_empty() {
        // Nothing followed the name, so a finished command clears the
        // selection rather than setting it.
        if frag.final_0() {
            state.selection_set(mask, None, progress.state() != SELECTION_SET, true);
        }
        return;
    }

    if progress.state() == SELECTION_SELECTED {
        if rest[0] == b'?' {
            progress.set_state(SELECTION_QUERY);
        } else {
            progress.set_state(SELECTION_SET_INITIAL);
            progress.recvpartial = 0;
        }
        state.set_selection_progress(progress);
    }

    match progress.state() {
        SELECTION_QUERY => {
            state.selection_query(mask);
            return;
        }
        SELECTION_INVALID => return,
        _ => {}
    }
    if !state.selection_accepts_set() {
        return;
    }

    // Decode base64 into the staging buffer, flushing it whenever it no
    // longer has room for another whole group.
    let buflen = state.selection_buffer_mut().len();
    let mut bufcur = 0usize;
    let mut sextets = progress.recvpartial >> 24;
    let mut value = progress.recvpartial & 0x03_FFFF;
    if progress.recvpartial != 0 {
        progress.recvpartial = 0;
        state.set_selection_progress(progress);
    }

    while buflen - bufcur >= 3 && !rest.is_empty() {
        if rest[0] == b'=' {
            // Padding: flush whatever whole bytes the partial group holds.
            let buffer = state.selection_buffer_mut();
            if sextets == 2 {
                buffer[bufcur] = (value >> 4) as u8;
                bufcur += 1;
            } else if sextets == 3 {
                buffer[bufcur] = (value >> 10) as u8;
                buffer[bufcur + 1] = (value >> 2) as u8;
                bufcur += 2;
            }
            while rest.first() == Some(&b'=') {
                rest = &rest[1..];
            }
            sextets = 0;
        } else if let Some(digit) = unbase64one(rest[0]) {
            value = (value << 6) | u32::from(digit);
            sextets += 1;
            rest = &rest[1..];
            if sextets == 4 {
                let buffer = state.selection_buffer_mut();
                buffer[bufcur] = (value >> 16) as u8;
                buffer[bufcur + 1] = (value >> 8) as u8;
                buffer[bufcur + 2] = value as u8;
                bufcur += 3;
                value = 0;
                sextets = 0;
            }
        } else {
            // Not base64 at all: give up on the whole selection.
            progress.set_state(SELECTION_INVALID);
            state.set_selection_progress(progress);
            state.selection_set(mask, None, true, true);
            break;
        }

        if rest.is_empty() || buflen - bufcur < 3 {
            if bufcur > 0 {
                let initial = progress.state() == SELECTION_SET_INITIAL;
                let last = frag.final_0() && rest.is_empty();
                state.selection_set(mask, Some(bufcur), initial, last);
                progress.set_state(SELECTION_SET);
                state.set_selection_progress(progress);
            }
            bufcur = 0;
        }
    }

    if sextets != 0 {
        progress.recvpartial = (sextets << 24) | value;
        state.set_selection_progress(progress);
    }
}

#[cfg(test)]
mod tests {
    use super::unbase64one;

    #[test]
    fn base64_digits_cover_the_whole_alphabet() {
        assert_eq!(unbase64one(b'A'), Some(0));
        assert_eq!(unbase64one(b'Z'), Some(25));
        assert_eq!(unbase64one(b'a'), Some(26));
        assert_eq!(unbase64one(b'z'), Some(51));
        assert_eq!(unbase64one(b'0'), Some(52));
        assert_eq!(unbase64one(b'9'), Some(61));
        assert_eq!(unbase64one(b'+'), Some(62));
        assert_eq!(unbase64one(b'/'), Some(63));
        assert_eq!(unbase64one(b'='), None);
        assert_eq!(unbase64one(b'-'), None);
        assert_eq!(unbase64one(0), None);
    }
}
