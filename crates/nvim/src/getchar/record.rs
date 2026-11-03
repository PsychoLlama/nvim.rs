//! Recording what was typed: registers, `'scriptout'`, `vim.on_key`, showcmd.
//!
//! Every key `vgetorpeek` hands out passes through [`gotchars`], which writes
//! it to the recording register, the `'scriptout'` file, and the buffer the
//! `vim.on_key()` callbacks are handed.
//!
//! Both [`gotchars`] and [`add_byte_to_showcmd`] are fed one *byte* at a time
//! but have to act on whole *keys*, because a key that straddled two calls
//! would be split across two record-buffer blocks and `delete_tail`
//! could no longer take it back off. [`gotchars_add_byte`] is the little
//! state machine that reassembles them, and each caller keeps its own copy of
//! that state.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::keycodes::ModMask;
use crate::keycodes::{KE_IGNORE, KS_EXTRA, key_unescape};
use crate::mbyte::{char_at, unescape_at};
use crate::types::{Key, MB_MAXBYTES};

impl Default for GotcharsState {
    fn default() -> Self {
        GotcharsState::new()
    }
}

impl GotcharsState {
    /// A state machine with nothing pending.
    pub(crate) const fn new() -> Self {
        GotcharsState {
            buf: [0; MB_MAXBYTES * 3 + 4],
            prev_byte: 0,
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
pub(crate) fn gotchars_add_byte(state: &mut GotcharsState, byte: u8) -> bool {
    state.buf[state.buflen] = byte;
    state.buflen += 1;
    if state.buflen == state.buf.len() {
        // No key is this long: modifier prefixes with no key after them
        // (`'showcmd'` re-reads a waiting one every time round) never
        // complete. Upstream writes past its buffer here; the bytes are
        // handed on as they are and the state starts afresh.
        state.pending_special = 0;
        state.pending_mbyte = 0;
        state.prev_byte = 0;
        return true;
    }

    let in_special = state.pending_special > 0;
    let in_mbyte = state.pending_mbyte > 0;

    if in_special {
        state.pending_special -= 1;
    } else if byte == K_SPECIAL {
        // A special key sequence is held until all three bytes are in and
        // it is clear what they stand for.
        state.pending_special = 2;
    }

    let mut whole = false;
    if state.pending_special == 0 {
        if in_mbyte {
            state.pending_mbyte -= 1;
        } else {
            let key = if in_special {
                if state.prev_byte == KS_MODIFIER {
                    // A modifier prefix: wait for the key it modifies.
                    state.prev_byte = byte;
                    return false;
                }
                key_unescape(state.prev_byte, byte)
            } else {
                Key::byte(byte)
            };
            // A multibyte character is held until all its bytes are in,
            // so that it cannot be split between two buffer blocks --
            // `delete_tail` would not be able to undo half of one.
            let len = key
                .as_byte()
                .map_or(1, |b| usize::from(utf8len_tab[usize::from(b)]));
            state.pending_mbyte = len - 1;
        }
        whole = state.pending_mbyte == 0;
    }

    state.prev_byte = byte;
    whole
}

/// Record typed input.
///
/// It goes to the `'scriptout'` file, to the `vim.on_key()` buffer and, when
/// a register is being recorded into, to that. `chars` is the caller's own
/// copy, never a borrow of the typeahead: the loop reaches `'scriptout'` and
/// the swap files between bytes.
pub(crate) fn gotchars(chars: &[u8]) {
    for &byte in chars {
        if !gotchars_pending.with_mut(|st| gotchars_add_byte(st, byte)) {
            continue;
        }
        // A copy of the finished key, so that nothing below holds the
        // state across `updatescript` or the callbacks.
        let key = gotchars_pending.get();
        let buflen = key.buflen;

        // One byte at a time; no translation to be done.
        for i in 0..buflen {
            updatescript(key.buf[i]);
        }

        // `ins_char_typebuf` can ask for the bytes it puts back to be
        // hidden from vim.on_key(); that is what the ignore count is.
        if buflen > on_key_ignore_len.get() {
            let from = on_key_ignore_len.get();
            on_key_buf.with_mut(|buf| buf.extend_from_slice(&key.buf[from..buflen]));
            on_key_ignore_len.set(0);
        } else {
            on_key_ignore_len.set(on_key_ignore_len.get() - buflen);
        }

        if reg_recording.get() != 0 {
            recordbuff().add_bytes(&key.buf[..buflen]);
            // Remember how many characters were recorded last, so that
            // `get_recorded` can drop the keys that stopped the recording.
            last_recorded_len.set(last_recorded_len.get().wrapping_add(buflen));
        }

        gotchars_pending.with_mut(|st| st.buflen = 0);
    }

    may_sync_undo();

    // Output the "debug mode" message again next time round.
    debug_did_msg.set(false);

    // Characters have been typed, so whatever follows counts as another
    // mapping. A search string is kept in the history.
    maptick.set(maptick.get() + 1);
}

/// Record an `<Ignore>` key, which nothing acts on.
///
/// Used after a timed-out `<Esc>` so that the ESC cannot combine with
/// whatever is typed next into a key code.
pub fn gotchars_ignore() {
    let nop = [K_SPECIAL, KS_EXTRA, KE_IGNORE];
    on_key_ignore_len.set(on_key_ignore_len.get() + 3);
    gotchars(&nop);
}

/// Add one byte to `'showcmd'` for a partially matched mapping, and show the
/// key once all of its bytes are in.
pub(crate) fn add_byte_to_showcmd(byte: u8) {
    if !p_sc() || msg_silent.get() != 0 {
        return;
    }
    if !showcmd_pending.with_mut(|st| gotchars_add_byte(st, byte)) {
        return;
    }
    // A copy of the finished key: `add_to_showcmd` below can reach the
    // screen.
    let pending = showcmd_pending.get();
    let buflen = pending.buflen;
    showcmd_pending.with_mut(|st| st.buflen = 0);

    // Split the key into its modifier prefix and the key itself. The key
    // is read up to its first NUL, as upstream's string is.
    let key = &pending.buf[..buflen];
    let mut key = &key[..key.iter().position(|&b| b == 0).unwrap_or(key.len())];
    let mut modifiers = ModMask::NONE;
    if let [K_SPECIAL, KS_MODIFIER, bits, rest @ ..] = key {
        modifiers = ModMask::from_byte(*bits);
        key = rest;
    }

    let mut c = Key::NUL;
    if let Some(&first) = key.first() {
        c = match unescape_at(key).filter(|_| first >= 0x80) {
            Some((ch, used)) => {
                key = &key[used..];
                Key::from_code(char_at(&ch))
            }
            None => {
                key = &key[1..];
                Key::byte(first)
            }
        };
        if c.code() <= 0x7f {
            // Fold the modifiers into the key where that has a spelling,
            // which reads better: CTRL-A rather than <C-> then A.
            let mut left = modifiers;
            let merged = merge_modifiers(c, &mut left);
            if left.is_empty() {
                modifiers = ModMask::NONE;
                c = merged;
            }
        }
    }

    // TODO(zeertzjq): is there a more readable and yet compact
    // representation of modifiers and special keys?
    if !modifiers.is_empty() {
        add_to_showcmd(Key::byte(K_SPECIAL));
        add_to_showcmd(Key::byte(KS_MODIFIER));
        add_to_showcmd(Key::byte(modifiers.byte()));
    }
    if !c.is_nul() {
        add_to_showcmd(c);
    }
    for &byte in key {
        add_to_showcmd(Key::byte(byte));
    }
}
