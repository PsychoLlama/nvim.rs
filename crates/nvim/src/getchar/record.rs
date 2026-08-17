//! Recording what was typed: registers, `'scriptout'`, `vim.on_key`, showcmd.
//!
//! Every key `vgetorpeek` hands out passes through [`gotchars`], which writes
//! it to the recording register, the `'scriptout'` file, and the buffer the
//! `vim.on_key()` callbacks are handed.
//!
//! Both [`gotchars`] and [`add_byte_to_showcmd`] are fed one *byte* at a time
//! but have to act on whole *keys*, because a key that straddled two calls
//! would be split across two record-buffer blocks and `delete_buff_tail`
//! could no longer take it back off. [`gotchars_add_byte`] is the little
//! state machine that reassembles them, and each caller keeps its own copy of
//! that state.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{KE_IGNORE, KS_EXTRA, key_unescape};
use crate::types::MB_MAXBYTES;
use core::ffi::{c_char, c_int, c_uint};

impl gotchars_state_T {
    /// A state machine with nothing pending.
    pub(crate) const fn new() -> Self {
        gotchars_state_T {
            buf: [0; MB_MAXBYTES * 3 + 4],
            prev_c: 0,
            buflen: 0,
            pending_special: 0,
            pending_mbyte: 0,
        }
    }
}

/// Add one byte to `state`, answering whether that completed a whole key.
///
/// When it answers true, `state.buf[..state.buflen]` is the key's bytes and
/// the caller is expected to reset `buflen`.
///
/// # Safety
/// `state` must point at a live state machine.
pub(crate) unsafe fn gotchars_add_byte(state: *mut gotchars_state_T, byte: u8) -> bool {
    unsafe {
        (*state).buf[(*state).buflen] = byte;
        (*state).buflen += 1;
        let mut c = c_int::from(byte);

        let in_special = (*state).pending_special > 0;
        let in_mbyte = (*state).pending_mbyte > 0;

        if in_special {
            (*state).pending_special -= 1;
        } else if c == K_SPECIAL {
            // A special key sequence is held until all three bytes are in and
            // it is clear what they stand for.
            (*state).pending_special = 2;
        }

        let mut whole = false;
        if (*state).pending_special == 0 {
            if in_mbyte {
                (*state).pending_mbyte -= 1;
            } else {
                if in_special {
                    if (*state).prev_c == KS_MODIFIER {
                        // A modifier prefix: wait for the key it modifies.
                        (*state).prev_c = c;
                        return false;
                    }
                    c = key_unescape((*state).prev_c as u8, c as u8);
                }
                // A multibyte character is held until all its bytes are in,
                // so that it cannot be split between two buffer blocks --
                // `delete_buff_tail` would not be able to undo half of one.
                (*state).pending_mbyte = mb_byte2len_check(c) as c_uint - 1;
            }
            whole = (*state).pending_mbyte == 0;
        }

        (*state).prev_c = c;
        whole
    }
}

/// Record `len` bytes of typed input.
///
/// They go to the `'scriptout'` file, to the `vim.on_key()` buffer and, when
/// a register is being recorded into, to that.
///
/// # Safety
/// `chars` must point at `len` readable bytes. It stays a raw pointer rather
/// than a slice because the loop calls `updatescript` and `add_buff` between
/// reads, and neither is provably unable to reach the buffer it points into.
pub(crate) unsafe fn gotchars(chars: *const u8, len: usize) {
    unsafe {
        /// What `gotchars` has half a key of, between calls.
        static state: GlobalCell<gotchars_state_T> = GlobalCell::new(gotchars_state_T::new());

        for i in 0..len {
            if !gotchars_add_byte(state.ptr(), *chars.add(i)) {
                continue;
            }
            let buflen = (*state.ptr()).buflen;

            // One byte at a time; no translation to be done.
            for i in 0..buflen {
                updatescript(c_int::from((*state.ptr()).buf[i]));
            }

            // `ins_char_typebuf` can ask for the bytes it puts back to be
            // hidden from vim.on_key(); that is what the ignore count is.
            if buflen > on_key_ignore_len.get() {
                let from = on_key_ignore_len.get();
                let bytes = core::slice::from_raw_parts(
                    (&raw const (*state.ptr()).buf).cast::<u8>().add(from),
                    buflen - from,
                );
                (*on_key_buf.ptr()).extend_from_slice(bytes);
                on_key_ignore_len.set(0);
            } else {
                on_key_ignore_len.set(on_key_ignore_len.get() - buflen);
            }

            if reg_recording.get() != 0 {
                (*state.ptr()).buf[buflen] = 0;
                add_buff(
                    recordbuff.ptr(),
                    (*state.ptr()).buf.as_ptr().cast(),
                    buflen as ptrdiff_t,
                );
                // Remember how many characters were recorded last, so that
                // `get_recorded` can drop the keys that stopped the recording.
                last_recorded_len.set(last_recorded_len.get().wrapping_add(buflen));
            }

            (*state.ptr()).buflen = 0;
        }

        may_sync_undo();

        // Output the "debug mode" message again next time round.
        debug_did_msg.set(false);

        // Characters have been typed, so whatever follows counts as another
        // mapping. A search string is kept in the history.
        maptick.set(maptick.get() + 1);
    }
}

/// Record an `<Ignore>` key, which nothing acts on.
///
/// Used after a timed-out `<Esc>` so that the ESC cannot combine with
/// whatever is typed next into a key code.
///
/// # Safety
/// Callable at any time.
pub unsafe fn gotchars_ignore() {
    let nop = [K_SPECIAL as u8, KS_EXTRA as u8, KE_IGNORE as u8];
    on_key_ignore_len.set(on_key_ignore_len.get() + 3);
    unsafe { gotchars(nop.as_ptr(), 3) };
}

/// Add one byte to `'showcmd'` for a partially matched mapping, and show the
/// key once all of its bytes are in.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn add_byte_to_showcmd(byte: u8) {
    unsafe {
        /// What `add_byte_to_showcmd` has half a key of, between calls.
        static state: GlobalCell<gotchars_state_T> = GlobalCell::new(gotchars_state_T::new());

        if p_sc.get() == 0 || msg_silent.get() != 0 {
            return;
        }
        if !gotchars_add_byte(state.ptr(), byte) {
            return;
        }
        let buflen = (*state.ptr()).buflen;
        (*state.ptr()).buf[buflen] = 0;
        (*state.ptr()).buflen = 0;

        // Split the key into its modifier prefix and the key itself.
        let mut ptr: *const c_char = (*state.ptr()).buf.as_ptr().cast();
        let mut modifiers = 0;
        if c_int::from(*ptr as u8) == K_SPECIAL
            && c_int::from(*ptr.add(1) as u8) == KS_MODIFIER
            && c_int::from(*ptr.add(2) as u8) != NUL
        {
            modifiers = c_int::from(*ptr.add(2) as u8);
            ptr = ptr.add(3);
        }

        let mut c = NUL;
        if c_int::from(*ptr as u8) != NUL {
            let mb_ptr = mb_unescape(&raw mut ptr);
            c = if !mb_ptr.is_null() {
                utf_ptr2char(mb_ptr)
            } else {
                let byte = *ptr as u8;
                ptr = ptr.add(1);
                c_int::from(byte)
            };
            if c <= 0x7f {
                // Fold the modifiers into the key where that has a spelling,
                // which reads better: CTRL-A rather than <C-> then A.
                let mut left = modifiers;
                let merged = merge_modifiers(c, &raw mut left);
                if left == 0 {
                    modifiers = 0;
                    c = merged;
                }
            }
        }

        // TODO(zeertzjq): is there a more readable and yet compact
        // representation of modifiers and special keys?
        if modifiers != 0 {
            add_to_showcmd(K_SPECIAL);
            add_to_showcmd(KS_MODIFIER);
            add_to_showcmd(modifiers);
        }
        if c != NUL {
            add_to_showcmd(c);
        }
        while c_int::from(*ptr as u8) != NUL {
            add_to_showcmd(c_int::from(*ptr as u8));
            ptr = ptr.add(1);
        }
    }
}
