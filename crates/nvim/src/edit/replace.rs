//! The replace stack: what Replace mode has to put back.
//!
//! In Replace mode a typed character overwrites one that was already there,
//! and backspacing has to restore it.  The stack is a flat byte vector of
//! NUL-*terminated* entries, one per character position the insert has
//! passed over: [`replace_push`] adds the bytes a character replaced,
//! [`replace_push_nul`] ends an entry, and [`replace_do_bs`] pops one entry
//! and writes it back.  A newline pushes *two* entries, the second holding
//! the white space that was deleted after the cursor, which is what
//! [`replace_join`] merges back.
//!
//! Two things make it more than a stack of bytes.  `replace_offset` says how
//! many bytes at the top belong to text the insert has already moved past,
//! so a push has to go *underneath* them -- that is the `memmove` in
//! [`replace_push`].  And an entry may hold a multi-byte character, so a pop
//! measures backwards from the last byte with `utf_head_off` rather than
//! taking one byte at a time ([`mb_replace_pop_ins`]).
//!
//! [`truncate_spaces`], [`backspace_until_column`] and [`del_char_after_col`]
//! are the delete primitives that know about all of this: each has a
//! Replace-mode arm that unwinds the stack instead of deleting text.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// The Replace-mode stack of overwritten bytes, by address.
///
/// Every operation on it — push, pop, join — grows or shrinks it in place
/// while Insert mode is running, so the address is what the family works
/// from and it is taken here once.
pub(super) fn replace_stack_ref() -> *mut ReplaceStack {
    replace_stack.ptr()
}

/// Truncate the white space at the end of a line, keeping the replace stack
/// in step.
///
/// Only for use in an Insert mode: in `MODE_REPLACE`/`MODE_VREPLACE` each
/// removed blank also drops one entry from the stack.
///
/// # Safety
/// `line` must point to at least `len` writable bytes plus a byte for the
/// NUL this writes.
pub(crate) unsafe fn truncate_spaces(line: *mut c_char, len: size_t) {
    unsafe {
        // Walk back over the trailing white space.
        let mut i = len as c_int - 1;
        while i >= 0 && ascii_iswhite(*line.offset(i as isize) as c_int) {
            if State.get() & REPLACE_FLAG != 0 {
                replace_join(0); // remove a NUL from the replace stack
            }
            i -= 1;
        }
        *line.offset((i + 1) as isize) = NUL as c_char;
    }
}

/// Backspace the cursor until column `col`, honouring Replace and Virtual
/// Replace mode.
///
/// May also be used outside Insert mode.  It tries not to go before `col`
/// even when a composing character sits on the boundary, which is why the
/// non-Replace arm goes through [`del_char_after_col`] rather than
/// `del_char`.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn backspace_until_column(col: c_int) {
    unsafe {
        while (*curwin.get()).w_cursor.col > col {
            (*curwin.get()).w_cursor.col -= 1;
            if State.get() & REPLACE_FLAG != 0 {
                replace_do_bs(col);
            } else if !del_char_after_col(col) {
                break;
            }
        }
    }
}

/// Like `del_char`, but do not delete anything starting before `limit_col`.
///
/// Only matters when there are composing characters: `mb_adjust_cursor` can
/// walk the cursor back past `limit_col` onto a base character, and then the
/// deletion has to be given up rather than taking the base with it.  A
/// negative `limit_col` means "no limit" and is a plain `del_char`.
///
/// Answers whether anything was deleted.
///
/// # Safety
/// Must run with a live `curwin`.
unsafe fn del_char_after_col(limit_col: c_int) -> bool {
    unsafe {
        if limit_col >= 0 {
            let ecol = (*curwin.get()).w_cursor.col + 1;

            // Put the cursor at the start of a character, then step forward
            // again if a composing character took it too far back.
            mb_adjust_cursor();
            while (*curwin.get()).w_cursor.col < limit_col {
                let l = utf_ptr2len(get_cursor_pos_ptr());
                if l == 0 {
                    break; // end of line
                }
                (*curwin.get()).w_cursor.col += l;
            }
            if *get_cursor_pos_ptr() as c_int == NUL || (*curwin.get()).w_cursor.col == ecol {
                return false;
            }
            del_bytes(ecol - (*curwin.get()).w_cursor.col, false, true);
        } else {
            del_char(false);
        }
        true
    }
}

/// kvec's `kv_roundup32`: the capacity `kv_ensure_space` picks for `n` bytes.
///
/// Rounds up to a power of two by smearing the top set bit down -- five
/// shifts, so it is a *32-bit* round-up even though the capacity is a
/// `size_t`.  Reproduced exactly: `alloc_log`'s unit specs assert the sizes
/// this produces.
const fn kv_roundup32(n: size_t) -> size_t {
    let mut x = n - 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x + 1
}

/// Push the bytes a character replaced onto the stack.
///
/// With `replace_offset` non-zero that many bytes are left *above* the new
/// entry, which is how a push reaches under text the insert has already
/// passed.
///
/// # Safety
/// `str` must point to `len` readable bytes.
pub(crate) unsafe fn replace_push(str: *mut c_char, len: size_t) {
    unsafe {
        let stack = replace_stack_ref();
        if (*stack).size < replace_offset.get() as size_t {
            return; // nothing to do
        }

        // kv_ensure_space(replace_stack, len)
        if (*stack).capacity < (*stack).size + len {
            (*stack).capacity = kv_roundup32((*stack).size + len);
            (*stack).items = xrealloc(
                (*stack).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<c_char>() * (*stack).capacity,
            ) as *mut c_char;
        }

        let p = (*stack)
            .items
            .add((*stack).size)
            .offset(-(replace_offset.get() as isize));
        if replace_offset.get() != 0 {
            memmove(
                p.add(len) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                replace_offset.get() as size_t,
            );
        }
        memcpy(
            p as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len,
        );
        (*stack).size += len;
    }
}

/// Push a NUL, the separator between entries.
///
/// # Safety
/// Must run with the replace stack initialised (it always is; the empty
/// stack is a null pointer with zero capacity).
pub(crate) unsafe fn replace_push_nul() {
    unsafe { replace_push(c"".as_ptr().cast_mut(), 1) }
}

/// Look at the top of the stack, popping it if it is a NUL.
///
/// Answers -1 for an empty stack, and otherwise the last byte -- so a
/// positive answer means "an entry is open, take a whole character off it
/// with [`mb_replace_pop_ins`]".
///
/// # Safety
/// Must run with a live replace stack.
pub(crate) unsafe fn replace_pop_if_nul() -> c_int {
    unsafe {
        let stack = replace_stack_ref();
        let ch = if (*stack).size != 0 {
            *(*stack).items.add((*stack).size - 1) as uint8_t as c_int
        } else {
            -1
        };
        if ch == NUL {
            (*stack).size -= 1;
        }
        ch
    }
}

/// Join the top two entries by removing the `off`'th NUL from the top.
///
/// # Safety
/// Must run with a live replace stack.
pub(crate) unsafe fn replace_join(mut off: c_int) {
    unsafe {
        let stack = replace_stack_ref();
        let mut i = (*stack).size as ssize_t;
        while i > 0 {
            i -= 1;
            if *(*stack).items.offset(i as isize) as c_int != NUL {
                continue;
            }
            // Only a NUL counts down `off`, and the one that reaches zero is
            // the one removed.
            let this_one = off <= 0;
            off -= 1;
            if this_one {
                (*stack).size -= 1;
                memmove(
                    (*stack).items.offset(i as isize) as *mut ::core::ffi::c_void,
                    (*stack).items.offset(i + 1) as *const ::core::ffi::c_void,
                    (*stack).size - i as size_t,
                );
                return;
            }
        }
    }
}

/// Pop bytes until a NUL and insert them before the cursor.
///
/// Only usable in `MODE_REPLACE`/`MODE_VREPLACE` -- and it turns the mode
/// *off* while it works, because the insertions it does must not push onto
/// the stack it is popping.
///
/// # Safety
/// Must run with a live `curwin` and replace stack.
pub(crate) unsafe fn replace_pop_ins() {
    unsafe {
        let old_state = State.get();
        State.set(MODE_NORMAL); // don't want MODE_REPLACE here
        while replace_pop_if_nul() > 0 {
            mb_replace_pop_ins();
            dec_cursor();
        }
        State.set(old_state);
    }
}

/// Insert one whole multi-byte character popped off the stack.
///
/// The caller must already have checked that the top of the stack is not a
/// NUL: the length is measured *backwards* from the last byte, and on an
/// empty entry `utf_head_off` would be reading the byte before it.
///
/// # Safety
/// The replace stack's top entry must be non-empty.
pub(crate) unsafe fn mb_replace_pop_ins() {
    unsafe {
        let stack = replace_stack_ref();
        let len = utf_head_off((*stack).items, (*stack).items.add((*stack).size - 1)) + 1;
        (*stack).size -= len as size_t;
        ins_bytes_len((*stack).items.add((*stack).size), len as size_t);
    }
}

/// One backspace in Replace mode.
///
/// The top of the stack says what to do:
/// - below zero, the stack is empty and the cursor has already been moved;
/// - zero, the character was *inserted* by this insert, so delete it;
/// - above zero, the character *replaced* one, so put the original back.
///
/// `limit_col >= 0` means "do not delete before this column", which matters
/// with composing characters -- see [`del_char_after_col`].
///
/// # Safety
/// Must run with a live `curwin` and replace stack.
pub(crate) unsafe fn replace_do_bs(limit_col: c_int) {
    unsafe {
        let l_state = State.get();
        let cc = replace_pop_if_nul();
        if cc > 0 {
            let mut start_vcol: colnr_T = 0;
            let mut orig_vcols = 0;
            if l_state & VREPLACE_FLAG != 0 {
                // How many screen cells the character about to be deleted
                // took.
                getvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    ::core::ptr::null_mut(),
                    &raw mut start_vcol,
                    ::core::ptr::null_mut(),
                );
                orig_vcols = win_chartabsize(curwin.get(), get_cursor_pos_ptr(), start_vcol);
            }
            del_char_after_col(limit_col);
            let orig_len = if l_state & VREPLACE_FLAG != 0 {
                get_cursor_pos_len()
            } else {
                0
            };
            replace_pop_ins();

            if l_state & VREPLACE_FLAG != 0 {
                // How many screen cells the restored characters take.
                let p = get_cursor_pos_ptr();
                let ins_len = get_cursor_pos_len() - orig_len;
                let mut vcol = start_vcol;
                let mut i = 0;
                while i < ins_len {
                    vcol += win_chartabsize(curwin.get(), p.offset(i as isize), vcol);
                    // O-B15-22: upstream steps by the length of the *first*
                    // character every time (`utfc_ptr2len(p)`, not
                    // `p + i`), so a restored run of differently-sized
                    // characters is measured wrong.  Reproduced as it is.
                    i += utfc_ptr2len(p);
                }
                vcol -= start_vcol;

                // Virtual Replace keeps the following text aligned, so any
                // spaces it padded with have to come off again.
                (*curwin.get()).w_cursor.col += ins_len;
                while vcol > orig_vcols && gchar_cursor() == ' ' as c_int {
                    del_char(false);
                    orig_vcols += 1;
                }
                (*curwin.get()).w_cursor.col -= ins_len;
            }

            // Mark the buffer changed and prepare for displaying.
            changed_bytes((*curwin.get()).w_cursor.lnum, (*curwin.get()).w_cursor.col);
        } else if cc == 0 {
            del_char_after_col(limit_col);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::kv_roundup32;

    #[test]
    fn roundup32_is_the_next_power_of_two() {
        assert_eq!(kv_roundup32(1), 1);
        assert_eq!(kv_roundup32(2), 2);
        assert_eq!(kv_roundup32(3), 4);
        assert_eq!(kv_roundup32(5), 8);
        assert_eq!(kv_roundup32(1024), 1024);
        assert_eq!(kv_roundup32(1025), 2048);
    }

    /// Five shifts smear the top bit down 31 places, no further -- so past
    /// 2^32 the answer stops being a power of two.  Upstream's behaviour,
    /// pinned; the replace stack never gets near it.
    #[test]
    fn roundup32_does_not_reach_past_32_bits() {
        assert_eq!(kv_roundup32(1 << 33), 1 << 33);
        assert_eq!(kv_roundup32((1 << 33) + 1), (1 << 34) - 3);
    }
}
