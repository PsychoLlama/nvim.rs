//! `nvim_paste`'s typeahead half.
//!
//! [`paste_store`] accumulates the pasted chunks into the redo buffer so that
//! `.` can repeat a paste, and [`paste_repeat`] is what `.` then runs. The
//! stream is bracketed by `K_PASTE_START`/`K_PASTE_END` so that the repeat
//! knows where it ends.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Keys;
use crate::keycodes::Key;
use crate::keycodes::key_unescape;
use crate::types::NUL;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Which part of a paste stream a [`paste_store`] call is carrying.
///
/// Upstream passes a `TriState` here, but the three values are phases of a
/// stream, not an unknown boolean.
#[derive(Copy, Clone, PartialEq, Eq)]
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
/// `str` is read only for [`PastePhase::Chunk`]; `K_SPECIAL` and NUL bytes
/// in the content are escaped.
///
/// # Safety
/// `str` must be a valid string when `phase` is [`PastePhase::Chunk`].
pub unsafe fn paste_store(channel_id: uint64_t, phase: PastePhase, str: String_0, crlf: bool) {
    if State.get() & MODE_CMDLINE != 0 {
        return;
    }
    let need_redo = !block_redo.get();
    let need_record = reg_recording.get() != 0 && !is_internal_call(channel_id);
    if !need_redo && !need_record {
        return;
    }

    if phase != PastePhase::Chunk {
        let c = if phase == PastePhase::Start {
            Key::PasteStart.code()
        } else {
            Key::PasteEnd.code()
        };
        if need_redo {
            if phase == PastePhase::Start && State.get() & MODE_INSERT == 0 {
                // SAFETY (this body): the arena and the array builder are this
                // frame's own, and every string put in them is either a static
                // or an allocation this frame owns.
                unsafe { reset_redobuff() };
            }
            redobuff().add_char(c);
        }
        if need_record {
            recordbuff().add_char(c);
        }
        return;
    }

    let mut s: *const c_char = str.data();
    let end = unsafe { str.data().add(str.len()) };
    while s < end {
        // A run of bytes that need no escaping goes in one piece.
        let start = s;
        while s < end
            && c_int::from(unsafe { *s } as u8) != K_SPECIAL
            && c_int::from(unsafe { *s }) != NUL
            && c_int::from(unsafe { *s }) != NL
            && !(crlf && c_int::from(unsafe { *s }) == CAR)
        {
            s = unsafe { s.add(1) };
        }
        if s > start {
            let len = unsafe { s.offset_from(start) };
            if need_redo {
                unsafe { redobuff().add(start, len) };
            }
            if need_record {
                unsafe { recordbuff().add(start, len) };
            }
        }

        // Then the byte that stopped it, escaped as one key.
        if s < end {
            let mut c = c_int::from(unsafe { *s } as u8);
            s = unsafe { s.add(1) };
            if crlf && c == CAR {
                // A CRLF pair counts as one newline.
                if s < end && c_int::from(unsafe { *s }) == NL {
                    s = unsafe { s.add(1) };
                }
                c = NL;
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
///
/// # Safety
/// Callable at any time; reads from the typeahead until `K_PASTE_END`.
pub unsafe fn paste_repeat(count: c_int) {
    let mut ga = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 1,
        ga_growsize: 32,
        ga_data: ptr::null_mut(),
    };
    let mut aborted = false;

    let unmapped = Keys::unmapped();
    got_int.set(false);
    while !aborted {
        // SAFETY (this body): the stored paste is this module's own `Array`,
        // and the arena is this frame's.
        unsafe { ga_grow(&raw mut ga, 32) };
        let first = unsafe { vgetorpeek(true) } as u8;
        if c_int::from(first) == K_SPECIAL {
            // Undo the escaping `paste_store` applied, except that the
            // bytes of a real key code go back in as they came out.
            let second = unsafe { vgetorpeek(true) } as u8;
            let third = unsafe { vgetorpeek(true) } as u8;
            let key = key_unescape(second, third);
            match Key::try_from(key) {
                Ok(Key::PasteEnd) => break,
                Ok(Key::Zero) => unsafe { ga_append(&raw mut ga, NUL as u8) },
                _ if key == K_SPECIAL => unsafe { ga_append(&raw mut ga, K_SPECIAL as u8) },
                _ => {
                    unsafe { ga_append(&raw mut ga, K_SPECIAL as u8) };
                    unsafe { ga_append(&raw mut ga, second) };
                    unsafe { ga_append(&raw mut ga, third) };
                }
            }
        } else {
            unsafe { ga_append(&raw mut ga, first) };
        }
        aborted = got_int.get();
    }
    drop(unmapped);

    let str = String_0::from_raw_parts(ga.ga_data.cast(), ga.ga_len as usize);
    let mut arena: Arena = ARENA_EMPTY;
    let mut err = Error::none();
    let mut i = 0;
    while !aborted && i < count {
        if let Err(e) =
            unsafe { nvim_paste(LUA_INTERNAL_CALL, str, false, -1 as Integer, &raw mut arena) }
        {
            err = e;
        }
        aborted = err.is_set();
        i += 1;
    }
    err.clear();
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
    unsafe { ga_clear(&raw mut ga) };
}
