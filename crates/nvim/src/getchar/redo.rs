//! The redo buffer: what `.` replays.
//!
//! Normal-mode commands append themselves to `redobuff` as they run
//! ([`AppendToRedobuff`] and friends); `.` calls [`start_redo`], which copies
//! that buffer into the read buffer so the keys are re-read as if stuffed.
//! `old_redobuff` keeps the previous one so that `CTRL-O .` in Insert mode can
//! repeat the command before the insert rather than the insert itself.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{Ctrl_V, key_unescape};
use crate::types::{FAIL, MB_MAXBYTES, NUL, OK};
use core::ffi::{c_char, c_int};
use core::ptr;

/// Where [`read_redo`] is up to: the block it is reading, and the byte within
/// it. A pair of walk cursors rather than an index, because the blocks are
/// separately allocated and the walk crosses from one to the next mid-key.
static redo_block: GlobalCell<*mut buffblock_T> = GlobalCell::new(ptr::null_mut());
static redo_at: GlobalCell<*const u8> = GlobalCell::new(ptr::null());

/// Move the current redo buffer to `old_redobuff` and start a fresh one.
///
/// The previous contents are what `CTRL-O .` in Insert mode repeats.
///
/// # Safety
/// Callable at any time.
pub unsafe fn ResetRedobuff() {
    unsafe {
        if block_redo.get() {
            return;
        }
        free_buff(old_redobuff.ptr());
        old_redobuff.set(redobuff.get());
        (*redobuff.ptr()).bh_first.b_next = ptr::null_mut();
    }
}

/// Discard the redo buffer and put the previous one back.
///
/// # Safety
/// Callable at any time.
pub unsafe fn CancelRedo() {
    unsafe {
        if block_redo.get() {
            return;
        }
        free_buff(redobuff.ptr());
        redobuff.set(old_redobuff.get());
        (*old_redobuff.ptr()).bh_first.b_next = ptr::null_mut();
        start_stuff();
        while read_readbuffers(true) != NUL {}
    }
}

/// Move both redo buffers into `save_redo`, leaving a *copy* of the current
/// one behind.
///
/// Used before running autocommands and user functions, which must not append
/// to the caller's redo buffer. The copy is what makes `:normal .` inside a
/// function repeat the command the function's caller last ran.
///
/// # Safety
/// `save_redo` must point at writable storage that outlives the matching
/// [`restoreRedobuff`].
pub unsafe fn saveRedobuff(save_redo: *mut save_redo_T) {
    unsafe {
        (*save_redo).sr_redobuff = redobuff.get();
        (*redobuff.ptr()).bh_first.b_next = ptr::null_mut();
        (*save_redo).sr_old_redobuff = old_redobuff.get();
        (*old_redobuff.ptr()).bh_first.b_next = ptr::null_mut();

        let (copy, len) = buff_contents(&raw mut (*save_redo).sr_redobuff, false);
        if copy.is_null() {
            return;
        }
        add_buff(redobuff.ptr(), copy, len as ptrdiff_t);
        xfree(copy.cast());
    }
}

/// Put back what [`saveRedobuff`] moved aside.
///
/// # Safety
/// `save_redo` must be the one a matching [`saveRedobuff`] filled.
pub unsafe fn restoreRedobuff(save_redo: *mut save_redo_T) {
    unsafe {
        free_buff(redobuff.ptr());
        redobuff.set((*save_redo).sr_redobuff);
        free_buff(old_redobuff.ptr());
        old_redobuff.set((*save_redo).sr_old_redobuff);
    }
}

/// Append `s` to the redo buffer. `K_SPECIAL` must already be escaped.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn AppendToRedobuff(s: *const c_char) {
    unsafe {
        if !block_redo.get() {
            add_buff(redobuff.ptr(), s, -1);
        }
    }
}

/// Append `str` to the redo buffer literally, quoting with CTRL-V whatever
/// would otherwise act as a command. `K_SPECIAL` is escaped as well.
///
/// `len` is the length, or -1 for up to the NUL.
///
/// # Safety
/// `str` must point at `len` readable bytes, or at a NUL-terminated string
/// when `len` is negative.
pub unsafe fn AppendToRedobuffLit(str: *const c_char, len: c_int) {
    unsafe {
        if block_redo.get() {
            return;
        }

        // How much of `str` is still to be appended, honouring both the
        // explicit length and the NUL terminator.
        let more = |s: *const c_char| {
            if len < 0 {
                c_int::from(*s) != NUL
            } else {
                s.offset_from(str) < len as isize
            }
        };

        let mut s = str;
        while more(s) {
            // Append a run of ordinary characters in one go; that is faster.
            let start = s;
            while c_int::from(*s) >= ' ' as c_int && c_int::from(*s) < DEL && more(s) {
                s = s.add(1);
            }
            // Don't leave a '0' or '^' last, just in case a CTRL-D is typed
            // next -- both delete the indent rather than being inserted.
            // `s > start` here whenever `*s` is the NUL, so the look-back is
            // in bounds.
            if c_int::from(*s) == NUL
                && (c_int::from(*s.sub(1)) == '0' as c_int
                    || c_int::from(*s.sub(1)) == '^' as c_int)
            {
                s = s.sub(1);
            }
            if s > start {
                add_buff(redobuff.ptr(), start, s.offset_from(start));
            }
            if c_int::from(*s) == NUL || !more(s) {
                break;
            }

            // Then the special or multibyte character that stopped the run.
            // Composing characters are handled separately, one at a time.
            let c = mb_cptr2char_adv(&raw mut s);
            let last = c_int::from(*s) == NUL;
            if c < ' ' as c_int || c == DEL || (last && (c == '0' as c_int || c == '^' as c_int)) {
                add_char_buff(redobuff.ptr(), Ctrl_V);
            }
            if last && c == '0' as c_int {
                // CTRL-V '0' must be inserted as CTRL-V 048.
                add_buff(redobuff.ptr(), c"048".as_ptr(), 3);
            } else {
                add_char_buff(redobuff.ptr(), c);
            }
        }
    }
}

/// Append `s` to the redo buffer, passing three-byte key codes through
/// unmodified and escaping every other `K_SPECIAL` byte.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn AppendToRedobuffSpec(mut s: *const c_char) {
    unsafe {
        if block_redo.get() {
            return;
        }
        while c_int::from(*s) != NUL {
            if c_int::from(*s as u8) == K_SPECIAL
                && c_int::from(*s.add(1)) != NUL
                && c_int::from(*s.add(2)) != NUL
            {
                // Insert the special key literally.
                add_buff(redobuff.ptr(), s, 3);
                s = s.add(3);
            } else {
                add_char_buff(redobuff.ptr(), mb_cptr2char_adv(&raw mut s));
            }
        }
    }
}

/// Append one character to the redo buffer, escaping special keys, NUL and
/// `K_SPECIAL` and splitting a codepoint into its UTF-8 bytes.
///
/// # Safety
/// Callable at any time.
pub unsafe fn AppendCharToRedobuff(c: c_int) {
    unsafe {
        if !block_redo.get() {
            add_char_buff(redobuff.ptr(), c);
        }
    }
}

/// Append the decimal spelling of `n` to the redo buffer.
///
/// # Safety
/// Callable at any time.
pub unsafe fn AppendNumberToRedobuff(n: c_int) {
    unsafe {
        if !block_redo.get() {
            add_num_buff(redobuff.ptr(), n);
        }
    }
}

/// Read one character from the redo buffer, undoing `add_char_buff`'s
/// escaping. The buffer itself is left alone.
///
/// With `init` set this only positions the cursor and answers `OK`, or `FAIL`
/// when there is nothing to redo. Otherwise it answers the character, or
/// `NUL` at the end. With `old_redo` set it walks `old_redobuff` instead.
///
/// # Safety
/// A call without `init` must follow a call with it that answered `OK`, and
/// the buffer must not have been freed in between.
pub(crate) unsafe fn read_redo(init: bool, old_redo: bool) -> c_int {
    unsafe {
        if init {
            let head = if old_redo {
                (*old_redobuff.ptr()).bh_first.b_next
            } else {
                (*redobuff.ptr()).bh_first.b_next
            };
            if head.is_null() {
                return FAIL;
            }
            redo_block.set(head);
            redo_at.set(block_str(head).cast());
            return OK;
        }

        let mut c = c_int::from(*redo_at.get());
        if c == NUL {
            return c;
        }

        // How many bytes this character occupies. An escaped K_SPECIAL is
        // three bytes that stand for one, so only a byte that is *not* the
        // start of an escape can begin a multibyte sequence.
        let n = if c != K_SPECIAL || c_int::from(*redo_at.get().add(1)) == KS_SPECIAL {
            mb_byte2len_check(c)
        } else {
            1
        };

        let mut buf = [0u8; MB_MAXBYTES + 1];
        let mut i = 0;
        loop {
            if c == K_SPECIAL {
                // Special key or escaped K_SPECIAL: three bytes, one key.
                c = key_unescape(*redo_at.get().add(1), *redo_at.get().add(2));
                redo_at.set(redo_at.get().add(2));
            }
            redo_at.set(redo_at.get().add(1));
            if c_int::from(*redo_at.get()) == NUL && !(*redo_block.get()).b_next.is_null() {
                let next = (*redo_block.get()).b_next;
                redo_block.set(next);
                redo_at.set(block_str(next).cast());
            }

            buf[i] = c as u8;
            if i == n - 1 {
                // Last byte of the character.
                if n != 1 {
                    c = utf_ptr2char(buf.as_ptr().cast());
                }
                break;
            }
            c = c_int::from(*redo_at.get());
            if c == NUL {
                break; // cannot happen?
            }
            i += 1;
        }
        c
    }
}

/// C's `MB_BYTE2LEN_CHECK`: how many bytes a UTF-8 sequence starting with `b`
/// occupies, and 1 for anything that is not a byte at all.
pub(crate) fn mb_byte2len_check(b: c_int) -> usize {
    if !(0..=255).contains(&b) {
        1
    } else {
        utf8len_tab[b as usize] as usize
    }
}

/// Copy the rest of the redo buffer into `readbuf2`, one character at a time.
///
/// The escaped `K_SPECIAL` is copied without translation: [`read_redo`]
/// decodes it and `add_char_buff` re-encodes it identically.
///
/// # Safety
/// As [`read_redo`] without `init`.
unsafe fn copy_redo(old_redo: bool) {
    unsafe {
        loop {
            let c = read_redo(false, old_redo);
            if c == NUL {
                break;
            }
            add_char_buff(readbuf2.ptr(), c);
        }
    }
}

/// Stuff the redo buffer into `readbuf2`, replacing its count with `count`.
///
/// With `old_redo` set the last command but one is repeated instead of the
/// last one, which is what `CTRL-O .` in Insert mode wants. Answers `FAIL`
/// when there is nothing to redo.
///
/// # Safety
/// Callable at any time.
pub unsafe fn start_redo(count: c_int, old_redo: bool) -> c_int {
    unsafe {
        // Position the cursor; give up if there is nothing to redo.
        if read_redo(true, old_redo) == FAIL {
            return FAIL;
        }
        let mut c = read_redo(false, old_redo);

        // Copy the register name, if there is one.
        if c == '"' as c_int {
            add_buff(readbuf2.ptr(), c"\"".as_ptr(), 1);
            c = read_redo(false, old_redo);

            // A numbered register shifts up: the redo of `"1p` is `"2p`.
            if c >= '1' as c_int && c < '9' as c_int {
                c += 1;
            }
            add_char_buff(readbuf2.ptr(), c);

            // The expression register has to be re-evaluated, so its CR --
            // which ends the expression -- goes in too.
            if c == '=' as c_int {
                add_char_buff(readbuf2.ptr(), CAR);
                cmd_silent.set(true);
            }

            c = read_redo(false, old_redo);
        }

        if c == 'v' as c_int {
            // Redo a Visual-mode operator over the same area.
            VIsual.set((*curwin.get()).w_cursor);
            VIsual_active.set(true);
            VIsual_select.set(false);
            VIsual_reselect.set(1);
            redo_VIsual_busy.set(true);
            c = read_redo(false, old_redo);
        }

        // Enter the new count in place of the old one.
        if count != 0 {
            while ascii_isdigit(c) {
                c = read_redo(false, old_redo);
            }
            add_num_buff(readbuf2.ptr(), count);
        }

        // Then the rest of the redo buffer, from the character the count
        // scan stopped on.
        add_char_buff(readbuf2.ptr(), c);
        copy_redo(old_redo);
        OK
    }
}

/// Repeat the last insert (`R`, `o`, `O`, `a`, `A`, `i` or `I`) by stuffing
/// the redo buffer into `readbuf2`. Answers `FAIL` when there is nothing to
/// repeat.
///
/// # Safety
/// Callable at any time.
pub unsafe fn start_redo_ins() -> c_int {
    unsafe {
        if read_redo(true, false) == FAIL {
            return FAIL;
        }
        start_stuff();

        // Skip the count and the command character.
        loop {
            let c = read_redo(false, false);
            if c == NUL {
                break;
            }
            if !vim_strchr(c"AaIiRrOo".as_ptr(), c).is_null() {
                if c == 'O' as c_int || c == 'o' as c_int {
                    // `o`/`O` opened the line; repeating the insert alone
                    // needs the newline put back.
                    add_buff(readbuf2.ptr(), c"\n".as_ptr(), -1);
                }
                break;
            }
        }

        // Then the text that was typed.
        copy_redo(false);
        block_redo.set(true);
        OK
    }
}

/// Stop blocking changes to the redo buffer; the pair of [`start_redo_ins`].
pub fn stop_redo_ins() {
    block_redo.set(false);
}
