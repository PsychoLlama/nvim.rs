//! The redo buffer: what `.` replays.
//!
//! Normal-mode commands append themselves to `redobuff` as they run
//! ([`append_to_redobuff`] and friends); `.` calls [`start_redo`], which copies
//! that buffer into the read buffer so the keys are re-read as if stuffed.
//! `old_redobuff` keeps the previous one so that `CTRL-O .` in Insert mode can
//! repeat the command before the insert rather than the insert itself.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::keycodes::{Ctrl_V, key_unescape};
use crate::mbyte::{char_at, char_len};
use crate::normal::{set_visual_active, set_visual_anchor, set_visual_select};
use crate::strings::has_char;
use crate::types::{Failed, MB_MAXBYTES, NL, NUL};
use crate::winlayer::Win;
use core::ffi::c_int;

/// Move the current redo buffer to `old_redobuff` and start a fresh one.
///
/// The previous contents are what `CTRL-O .` in Insert mode repeats.
pub fn reset_redobuff() {
    if block_redo.get() {
        return;
    }
    old_redobuff().free();
    old_redobuff().set(redobuff().take());
}

/// Discard the redo buffer and put the previous one back.
pub fn cancel_redo() {
    if block_redo.get() {
        return;
    }
    redobuff().free();
    redobuff().set(old_redobuff().take());
    start_stuff();
    while read_readbuffers(true) != NUL {}
}

/// Move both redo buffers into `save_redo`, leaving a *copy* of the current
/// one behind.
///
/// Used before running autocommands and user functions, which must not append
/// to the caller's redo buffer. The copy is what makes `:normal .` inside a
/// function repeat the command the function's caller last ran.
pub fn save_redobuff(save_redo: &mut SaveRedo) {
    save_redo.sr_redobuff = redobuff().take();
    save_redo.sr_old_redobuff = old_redobuff().take();
    redobuff().add_bytes(&save_redo.sr_redobuff.bytes());
}

/// Put back what [`save_redobuff`] moved aside.
pub fn restore_redobuff(save_redo: &mut SaveRedo) {
    redobuff().free();
    redobuff().set(core::mem::take(&mut save_redo.sr_redobuff));
    old_redobuff().free();
    old_redobuff().set(core::mem::take(&mut save_redo.sr_old_redobuff));
}

/// Append `s` to the redo buffer, up to its first NUL. `K_SPECIAL` must
/// already be escaped.
pub fn append_to_redobuff(s: &[u8]) {
    if !block_redo.get() {
        redobuff().add_bytes(up_to_nul(s));
    }
}

/// Append `s` to the redo buffer literally, quoting with CTRL-V whatever
/// would otherwise act as a command. `K_SPECIAL` is escaped as well.
///
/// `s` is read up to its first NUL, and its end is where upstream's NUL
/// was: a `0` or `^` there is quoted too.
pub fn append_to_redobuff_literally(s: &[u8]) {
    if block_redo.get() {
        return;
    }
    let s = up_to_nul(s);
    let mut at = 0;
    while at < s.len() {
        // Append a run of ordinary characters in one go; that is faster.
        let start = at;
        while at < s.len() && (b' '..DEL_BYTE).contains(&s[at]) {
            at += 1;
        }
        // Don't leave a '0' or '^' last, just in case a CTRL-D is typed
        // next -- both delete the indent rather than being inserted. The
        // run reached the end only by taking at least one byte, so the
        // look-back is in bounds.
        if at == s.len() && matches!(s[at - 1], b'0' | b'^') {
            at -= 1;
        }
        if at > start {
            redobuff().add_bytes(&s[start..at]);
        }
        if at == s.len() {
            break;
        }

        // Then the special or multibyte character that stopped the run.
        // Composing characters are handled separately, one at a time.
        let rest = &s[at..];
        let c = char_at(rest);
        at += char_len(rest);
        let last = at == s.len();
        if c < c_int::from(b' ')
            || c == DEL
            || (last && (c == c_int::from(b'0') || c == c_int::from(b'^')))
        {
            redobuff().add_char(Ctrl_V);
        }
        if last && c == c_int::from(b'0') {
            // CTRL-V '0' must be inserted as CTRL-V 048.
            redobuff().add_bytes(b"048");
        } else {
            redobuff().add_char(c);
        }
    }
}

/// `DEL` as the byte that ends a run of ordinary characters.
const DEL_BYTE: u8 = 0x7f;

/// Append `s` to the redo buffer, up to its first NUL, passing three-byte
/// key codes through unmodified and escaping every other `K_SPECIAL` byte.
pub fn append_to_redobuff_keys(s: &[u8]) {
    if block_redo.get() {
        return;
    }
    let mut s = up_to_nul(s);
    while let Some(&first) = s.first() {
        if c_int::from(first) == K_SPECIAL && s.len() >= 3 {
            // Insert the special key literally.
            redobuff().add_bytes(&s[..3]);
            s = &s[3..];
        } else {
            let c = char_at(s);
            s = &s[char_len(s)..];
            redobuff().add_char(c);
        }
    }
}

/// Append one character to the redo buffer, escaping special keys, NUL and
/// `K_SPECIAL` and splitting a codepoint into its UTF-8 bytes.
pub fn append_to_redobuff_char(c: c_int) {
    if !block_redo.get() {
        redobuff().add_char(c);
    }
}

/// Append the decimal spelling of `n` to the redo buffer.
pub fn append_to_redobuff_number(n: c_int) {
    if !block_redo.get() {
        redobuff().add_num(n);
    }
}

/// A walk over one of the redo buffers, a character at a time, undoing
/// `KeyBufferRef::add_char`'s escaping. The buffer itself is left alone.
///
/// Upstream walks the buffer's block chain in place, with a block pointer
/// and a byte pointer in two file statics. The walk reads a *copy* of the
/// bytes here, and its position is an index into that copy: the position
/// means nothing without the bytes it came from, so the two are one value,
/// and nothing a caller does to the redo buffers between two reads can
/// leave it pointing into a freed block.
struct RedoReader {
    bytes: Vec<u8>,
    at: usize,
}

impl RedoReader {
    /// Start a walk over the redo buffer, or over `old_redobuff` when
    /// `old_redo` is set. `None` when there is nothing to redo.
    fn start(old_redo: bool) -> Option<Self> {
        let buffer = if old_redo { old_redobuff() } else { redobuff() };
        if buffer.is_empty() {
            return None;
        }
        Some(RedoReader {
            bytes: buffer.bytes(),
            at: 0,
        })
    }

    /// The byte `ahead` past the cursor, or `NUL` past the end -- where the
    /// chain's terminator was.
    fn byte(&self, ahead: usize) -> u8 {
        self.bytes.get(self.at + ahead).copied().unwrap_or(0)
    }

    /// The next character, or `NUL` at the end.
    fn next_char(&mut self) -> c_int {
        let mut c = c_int::from(self.byte(0));
        if c == NUL {
            return c;
        }

        // How many bytes this character occupies. An escaped K_SPECIAL is
        // three bytes that stand for one, so only a byte that is *not* the
        // start of an escape can begin a multibyte sequence.
        let n = if c != K_SPECIAL || c_int::from(self.byte(1)) == KS_SPECIAL {
            mb_byte2len_check(c)
        } else {
            1
        };

        let mut buf = [0u8; MB_MAXBYTES + 1];
        let mut i = 0;
        loop {
            if c == K_SPECIAL {
                // Special key or escaped K_SPECIAL: three bytes, one key.
                c = key_unescape(self.byte(1), self.byte(2));
                self.at += 2;
            }
            self.at += 1;

            // Upstream's `(uint8_t)c`: a key code wider than a byte truncates
            // here, and the bytes are read back as one character below.
            buf[i] = c.to_le_bytes()[0];
            if i == n - 1 {
                // Last byte of the character.
                if n != 1 {
                    c = char_at(&buf);
                }
                break;
            }
            c = c_int::from(self.byte(0));
            if c == NUL {
                break; // cannot happen?
            }
            i += 1;
        }
        c
    }

    /// Copy the rest of the walk into `readbuf2`, one character at a time.
    ///
    /// The escaped `K_SPECIAL` is copied without translation:
    /// [`next_char`](Self::next_char) decodes it and
    /// `KeyBufferRef::add_char` re-encodes it identically.
    fn copy_rest(&mut self) {
        loop {
            let c = self.next_char();
            if c == NUL {
                break;
            }
            readbuf2().add_char(c);
        }
    }
}

/// C's `MB_BYTE2LEN_CHECK`: how many bytes a UTF-8 sequence starting with `b`
/// occupies, and 1 for anything that is not a byte at all.
pub(crate) fn mb_byte2len_check(b: c_int) -> usize {
    if !(0..=255).contains(&b) {
        1
    } else {
        let b = usize::try_from(b).expect("the guard above bounds `b` to a byte");
        utf8len_tab[b] as usize
    }
}

/// Stuff the redo buffer into `readbuf2`, replacing its count with `count`.
///
/// With `old_redo` set the last command but one is repeated instead of the
/// last one, which is what `CTRL-O .` in Insert mode wants. Answers `Err`
/// when there is nothing to redo.
pub fn start_redo(count: c_int, old_redo: bool) -> Result<(), Failed> {
    // Give up if there is nothing to redo.
    let mut redo = RedoReader::start(old_redo).ok_or(Failed)?;
    let mut c = redo.next_char();

    // Copy the register name, if there is one.
    if c == c_int::from(b'"') {
        readbuf2().add_byte(c_int::from(b'"'));
        c = redo.next_char();

        // A numbered register shifts up: the redo of `"1p` is `"2p`.
        if c >= '1' as c_int && c < '9' as c_int {
            c += 1;
        }
        readbuf2().add_char(c);

        // The expression register has to be re-evaluated, so its CR --
        // which ends the expression -- goes in too.
        if c == '=' as c_int {
            readbuf2().add_char(CAR);
            cmd_silent.set(true);
        }

        c = redo.next_char();
    }

    if c == 'v' as c_int {
        // Redo a Visual-mode operator over the same area.
        set_visual_anchor(Win::current().w_cursor);
        set_visual_active(true);
        set_visual_select(false);
        VIsual_reselect.set(1);
        redo_VIsual_busy.set(true);
        c = redo.next_char();
    }

    // Enter the new count in place of the old one.
    if count != 0 {
        while ascii_isdigit(c) {
            c = redo.next_char();
        }
        readbuf2().add_num(count);
    }

    // Then the rest of the redo buffer, from the character the count
    // scan stopped on.
    readbuf2().add_char(c);
    redo.copy_rest();
    Ok(())
}

/// Repeat the last insert (`R`, `o`, `O`, `a`, `A`, `i` or `I`) by stuffing
/// the redo buffer into `readbuf2`. Answers `Err` when there is nothing to
/// repeat.
pub fn start_redo_ins() -> Result<(), Failed> {
    let mut redo = RedoReader::start(false).ok_or(Failed)?;
    start_stuff();

    // Skip the count and the command character.
    loop {
        let c = redo.next_char();
        if c == NUL {
            break;
        }
        if has_char(c"AaIiRrOo", c) {
            if c == 'O' as c_int || c == 'o' as c_int {
                // `o`/`O` opened the line; repeating the insert alone
                // needs the newline put back.
                readbuf2().add_byte(NL);
            }
            break;
        }
    }

    // Then the text that was typed.
    redo.copy_rest();
    block_redo.set(true);
    Ok(())
}

/// Stop blocking changes to the redo buffer; the pair of [`start_redo_ins`].
pub fn stop_redo_ins() {
    block_redo.set(false);
}

#[cfg(test)]
mod tests {
    //! The redo walk, observed from outside: what [`start_redo`] and
    //! [`start_redo_ins`] stuff into the read buffers for a redo buffer
    //! built through the same entry points the editor uses.

    use super::*;
    use crate::global_cell::{editor_state::Held, editor_state_lock};
    use crate::keycodes::{Key, key_escape};

    /// The editor's key buffers are process-wide; hold the lock and start
    /// from empty ones. Dropping the answer leaves them empty again.
    struct Fresh(#[allow(dead_code, reason = "held for its drop")] Held);

    impl Fresh {
        fn new() -> Self {
            let held = editor_state_lock();
            clear();
            Fresh(held)
        }
    }

    impl Drop for Fresh {
        fn drop(&mut self) {
            clear();
        }
    }

    /// Empty both redo buffers and both read buffers.
    fn clear() {
        stop_redo_ins();
        reset_redobuff();
        reset_redobuff();
        drain();
    }

    /// Read the read buffers out, as `vgetc` would: the escaped bytes.
    fn drain() -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let c = read_readbuffers(true);
            if c == NUL {
                return out;
            }
            out.push(u8::try_from(c).expect("a read buffer holds bytes"));
        }
    }

    /// Append `keys` to the redo buffer one character at a time.
    fn redo(keys: &str) {
        for ch in keys.chars() {
            append_to_redobuff_char(c_int::try_from(u32::from(ch)).expect("a code point"));
        }
    }

    #[test]
    fn nothing_to_redo_fails() {
        let _fresh = Fresh::new();
        assert!(start_redo(0, false).is_err());
        assert!(start_redo(0, true).is_err());
        assert!(start_redo_ins().is_err());
        assert!(drain().is_empty());
    }

    #[test]
    fn a_count_replaces_the_one_recorded() {
        let _fresh = Fresh::new();
        redo("3dw");
        start_redo(0, false).unwrap();
        assert_eq!(drain(), b"3dw");
        start_redo(25, false).unwrap();
        assert_eq!(drain(), b"25dw");
        // The buffer itself is left alone by the walk.
        start_redo(0, false).unwrap();
        assert_eq!(drain(), b"3dw");
    }

    #[test]
    fn a_numbered_register_shifts_up() {
        let _fresh = Fresh::new();
        redo("\"1p");
        start_redo(0, false).unwrap();
        assert_eq!(drain(), b"\"2p");
        clear();
        redo("\"9p");
        start_redo(0, false).unwrap();
        assert_eq!(drain(), b"\"9p");
    }

    #[test]
    fn the_walk_crosses_blocks_mid_character() {
        let _fresh = Fresh::new();
        // One byte per add: a block is 20 bytes, so the 'é' after 19 'a's
        // is split between the first block and the second, and the next
        // ones land wherever they land.
        redo(&"a".repeat(19));
        let mut expected = vec![b'a'; 19];
        redo("é");
        expected.extend_from_slice("é".as_bytes());
        // U+0080, whose second UTF-8 byte is the escape byte itself.
        redo("\u{80}");
        expected.push(0xC2);
        expected.extend_from_slice(&key_escape(K_SPECIAL));
        // A special key: one character, three bytes.
        append_to_redobuff_char(Key::Left.code());
        expected.extend_from_slice(&key_escape(Key::Left.code()));
        for _ in 0..30 {
            redo("€x");
            expected.extend_from_slice("€x".as_bytes());
        }
        start_redo(0, false).unwrap();
        assert_eq!(drain(), expected);
    }

    #[test]
    fn repeating_an_insert_skips_the_command() {
        let _fresh = Fresh::new();
        redo("3ihello\x1b");
        start_redo_ins().unwrap();
        assert_eq!(drain(), b"hello\x1b");
        // The repeat blocks the redo buffer until it is over.
        redo("zz");
        start_redo(0, false).unwrap();
        assert_eq!(drain(), b"3ihello\x1b");
        stop_redo_ins();

        clear();
        redo("2Oab");
        start_redo_ins().unwrap();
        assert_eq!(drain(), b"\nab");
    }

    #[test]
    fn the_previous_command_is_one_reset_back() {
        let _fresh = Fresh::new();
        redo("dw");
        reset_redobuff();
        redo("x");
        start_redo(0, true).unwrap();
        assert_eq!(drain(), b"dw");
        start_redo(0, false).unwrap();
        assert_eq!(drain(), b"x");
        // Cancelling puts the previous one back.
        cancel_redo();
        start_redo(0, false).unwrap();
        assert_eq!(drain(), b"dw");
    }
}
