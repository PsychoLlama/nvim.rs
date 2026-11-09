//! `nvim_paste`'s typeahead half.
//!
//! [`paste_store`] accumulates the pasted chunks into the redo buffer so that
//! `.` can repeat a paste, and [`paste_repeat`] is what `.` then runs. The
//! stream is bracketed by `K_PASTE_START`/`K_PASTE_END` so that the repeat
//! knows where it ends.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::guard::Keys;
use crate::keycodes::{SpecialKey, key_unescape};
use crate::types::Key;
use core::ffi::c_int;

/// Which part of a paste stream a [`paste_store`] call is carrying.
///
/// Upstream passes a `TriState` here, but the three values are phases of a
/// stream, not an unknown boolean.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PastePhase {
    /// The paste is beginning; `K_PASTE_START` goes into the buffers.
    Start,
    /// A piece of the pasted text itself.
    Chunk,
    /// The paste is over; `K_PASTE_END` goes into the buffers.
    End,
}

/// Record a piece of a paste into the redo and/or record buffers.
///
/// `text` is read only for [`PastePhase::Chunk`]; `K_SPECIAL` and NUL bytes
/// in the content are escaped.
pub fn paste_store(channel_id: uint64_t, phase: PastePhase, text: &[u8], crlf: bool) {
    if State.get() & MODE_CMDLINE != 0 {
        return;
    }
    let need_redo = !BLOCK_REDO.get();
    let need_record = REG_RECORDING.get() != 0 && !is_internal_call(channel_id);
    if !need_redo && !need_record {
        return;
    }

    if phase != PastePhase::Chunk {
        let c = Key::from(if phase == PastePhase::Start {
            SpecialKey::PasteStart
        } else {
            SpecialKey::PasteEnd
        });
        if need_redo {
            if phase == PastePhase::Start && State.get() & MODE_INSERT == 0 {
                reset_redobuff();
            }
            redobuff().add_char(c);
        }
        if need_record {
            recordbuff().add_char(c);
        }
        return;
    }

    let mut s = text;
    while !s.is_empty() {
        // A run of bytes that need no escaping goes in one piece.
        let run = s
            .iter()
            .position(|&b| b == K_SPECIAL || b == 0 || b == b'\n' || (crlf && b == b'\r'))
            .unwrap_or(s.len());
        if run > 0 {
            if need_redo {
                redobuff().add_bytes(&s[..run]);
            }
            if need_record {
                recordbuff().add_bytes(&s[..run]);
            }
        }
        s = &s[run..];

        // Then the byte that stopped it, escaped as one key.
        if let Some((&first, rest)) = s.split_first() {
            let mut c = Key::byte(first);
            s = rest;
            if crlf && c == Key::CAR {
                // A CRLF pair counts as one newline.
                if s.first() == Some(&b'\n') {
                    s = &s[1..];
                }
                c = Key::NL;
            }
            if need_redo {
                redobuff().add_byte(c);
            }
            if need_record {
                recordbuff().add_byte(c);
            }
        }
    }
}

/// Read a paste stored by [`paste_store`] back out of the typeahead and
/// replay it `count` times.
pub fn paste_repeat(count: c_int) {
    let mut pasted = Vec::<u8>::new();
    let mut aborted = false;

    let unmapped = Keys::unmapped();
    GOT_INT.set(false);
    while !aborted {
        let first = vgetorpeek(true);
        if first == K_SPECIAL {
            // Undo the escaping `paste_store` applied, except that the
            // bytes of a real key code go back in as they came out.
            let second = vgetorpeek(true);
            let third = vgetorpeek(true);
            let key = key_unescape(second, third);
            match key.special() {
                Some(SpecialKey::PasteEnd) => break,
                Some(SpecialKey::Zero) => pasted.push(0),
                _ if key == Key::byte(K_SPECIAL) => pasted.push(K_SPECIAL),
                _ => pasted.extend_from_slice(&[K_SPECIAL, second, third]),
            }
        } else {
            pasted.push(first);
        }
        aborted = GOT_INT.get();
    }
    drop(unmapped);

    let str = ApiString::from_bytes(&pasted);
    let mut err = Error::none();
    let mut i = 0;
    while !aborted && i < count {
        // A copy per pass: the call takes the text over.
        if let Err(e) = nvim_paste(LUA_INTERNAL_CALL, str.clone(), false, -1 as Integer) {
            err = e;
        }
        aborted = err.is_set();
        i += 1;
    }
    err.clear();
}
