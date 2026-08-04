//! `nvim_paste`'s typeahead half.
//!
//! [`paste_store`] accumulates the pasted chunks into the redo buffer so that
//! `.` can repeat a paste, and [`paste_repeat`] is what `.` then runs. The
//! stream is bracketed by `K_PASTE_START`/`K_PASTE_END` so that the repeat
//! knows where it ends.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::key_unescape;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Record a piece of a paste into the redo and/or record buffers.
///
/// `state` is `kFalse` for the start of a paste, `kTrue` for the end, and
/// `kNone` for a chunk of its content — only then is `str` read. `K_SPECIAL`
/// and NUL bytes in the content are escaped.
///
/// # Safety
/// `str` must be a valid string when `state` is `kNone`.
pub unsafe fn paste_store(channel_id: uint64_t, state: TriState, str: String_0, crlf: bool) {
    unsafe {
        if State.get() & MODE_CMDLINE != 0 {
            return;
        }
        let need_redo = !block_redo.get();
        let need_record = reg_recording.get() != 0 && !is_internal_call(channel_id);
        if !need_redo && !need_record {
            return;
        }

        if state != kNone {
            let c = if state == kFalse {
                K_PASTE_START
            } else {
                K_PASTE_END
            };
            if need_redo {
                if state == kFalse && State.get() & MODE_INSERT == 0 {
                    ResetRedobuff();
                }
                add_char_buff(redobuff.ptr(), c);
            }
            if need_record {
                add_char_buff(recordbuff.ptr(), c);
            }
            return;
        }

        let mut s: *const c_char = str.data;
        let end = str.data.add(str.size);
        while s < end {
            // A run of bytes that need no escaping goes in one piece.
            let start = s;
            while s < end
                && c_int::from(*s as u8) != K_SPECIAL
                && c_int::from(*s) != NUL
                && c_int::from(*s) != NL
                && !(crlf && c_int::from(*s) == CAR)
            {
                s = s.add(1);
            }
            if s > start {
                let len = s.offset_from(start);
                if need_redo {
                    add_buff(redobuff.ptr(), start, len);
                }
                if need_record {
                    add_buff(recordbuff.ptr(), start, len);
                }
            }

            // Then the byte that stopped it, escaped as one key.
            if s < end {
                let mut c = c_int::from(*s as u8);
                s = s.add(1);
                if crlf && c == CAR {
                    // A CRLF pair counts as one newline.
                    if s < end && c_int::from(*s) == NL {
                        s = s.add(1);
                    }
                    c = NL;
                }
                if need_redo {
                    add_byte_buff(redobuff.ptr(), c);
                }
                if need_record {
                    add_byte_buff(recordbuff.ptr(), c);
                }
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
    unsafe {
        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 1,
            ga_growsize: 32,
            ga_data: ptr::null_mut(),
        };
        let mut aborted = false;

        *no_mapping.ptr() += 1;
        got_int.set(false);
        while !aborted {
            ga_grow(&raw mut ga, 32);
            let first = vgetorpeek(true) as u8;
            if c_int::from(first) == K_SPECIAL {
                // Undo the escaping `paste_store` applied, except that the
                // bytes of a real key code go back in as they came out.
                let second = vgetorpeek(true) as u8;
                let third = vgetorpeek(true) as u8;
                match key_unescape(second, third) {
                    K_PASTE_END => break,
                    K_ZERO => ga_append(&raw mut ga, NUL as u8),
                    K_SPECIAL => ga_append(&raw mut ga, K_SPECIAL as u8),
                    _ => {
                        ga_append(&raw mut ga, K_SPECIAL as u8);
                        ga_append(&raw mut ga, second);
                        ga_append(&raw mut ga, third);
                    }
                }
            } else {
                ga_append(&raw mut ga, first);
            }
            aborted = got_int.get();
        }
        *no_mapping.ptr() -= 1;

        let str = String_0 {
            data: ga.ga_data.cast(),
            size: ga.ga_len as usize,
        };
        let mut arena: Arena = ARENA_EMPTY;
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let mut i = 0;
        while !aborted && i < count {
            nvim_paste(
                LUA_INTERNAL_CALL,
                str,
                false,
                -1 as Integer,
                &raw mut arena,
                &raw mut err,
            );
            aborted = err.type_0 != kErrorTypeNone;
            i += 1;
        }
        api_clear_error(&raw mut err);
        arena_mem_free(arena_finish(&raw mut arena));
        ga_clear(&raw mut ga);
    }
}
