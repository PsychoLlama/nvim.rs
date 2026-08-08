//! The text of the last insert, and repeating it.
//!
//! `last_insert` is a copy of the redo buffer taken when Insert mode was
//! left, so it still has the command that *started* the insert on the front
//! -- `last_insert_skip` is how many bytes that command took, and every
//! reader goes through [`get_last_insert`], which skips them.
//!
//! [`get_last_insert`] is `".` and `i_CTRL-A`; [`get_last_insert_save`]
//! answers a copy with the trailing `<Esc>` removed; [`stuff_inserted`] is
//! `.`/CTRL-A/CTRL-@, which pushes the text back into the read buffer so the
//! main loop types it again, `count` times.  [`set_last_insert`] is the
//! single-character case `r` uses, which has to build the redo-buffer
//! spelling itself.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::{c_char, c_int};

use super::*;

/// Set the last inserted text to the single character `c`.
///
/// Used by `r`.  What is stored is the *redo buffer* spelling: a CTRL-V in
/// front of a control character, then the character, then the `<Esc>` that
/// ends an insert.
///
/// # Safety
/// Must run on the main thread; frees and replaces `last_insert`.
pub(crate) unsafe fn set_last_insert(c: c_int) {
    unsafe {
        xfree((*last_insert.ptr()).data as *mut ::core::ffi::c_void);
        (*last_insert.ptr()).data =
            xmalloc((MB_MAXBYTES as c_int * 3 + 5) as size_t) as *mut ::core::ffi::c_char;

        let start = (*last_insert.ptr()).data;
        let mut s = start;
        // The CTRL-V is only needed to enter a special character.
        if c < ' ' as c_int || c == DEL {
            *s = Ctrl_V as c_char;
            s = s.offset(1);
        }
        s = add_char2buf(c, s);
        *s = ESC as c_char;
        s = s.offset(1);
        *s = NUL as c_char;

        (*last_insert.ptr()).size = s.offset_from(start) as size_t;
        last_insert_skip.set(0);
    }
}

/// `.`, `i_CTRL-A` and `i_CTRL-@`: stuff the last inserted text into the read
/// buffer so the main loop types it again.
///
/// `c` is the command character that starts Insert mode (NUL for none),
/// `count` how many times, and `no_esc` says to leave the insert open at the
/// end.
///
/// `FAIL` -- with `E29` -- when there is nothing to insert.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn stuff_inserted(c: c_int, mut count: c_int, no_esc: c_int) -> c_int {
    unsafe {
        let mut insert = get_last_insert(); // text to be inserted
        if insert.data.is_null() {
            emsg(gettext(&raw const e_noinstext as *const c_char));
            return FAIL;
        }

        // May want to stuff the command character, to start Insert mode.
        if c != NUL {
            stuffcharReadbuff(c);
        }

        // Cut the text at the last ESC: what follows it is not part of the
        // insert.
        let mut i = insert.size;
        while i > 0 {
            i -= 1;
            if *insert.data.add(i) as c_int == ESC {
                insert.size = i;
                break;
            }
        }

        // A trailing `0` or `^` has to be quoted, because either would be
        // read as the start of `0 CTRL-D`/`^ CTRL-D` -- but only when
        // nothing follows it (no ESC is coming) or when the text is repeated
        // and starts with CTRL-D.  -- Acevedo
        let mut last = NUL as c_char;
        if insert.size > 0 {
            let p = insert.data.add(insert.size - 1);
            if (*p as c_int == '0' as c_int || *p as c_int == '^' as c_int)
                && (no_esc != 0 || (*insert.data as c_int == Ctrl_D && count > 1))
            {
                last = *p;
                insert.size -= 1;
            }
        }

        loop {
            stuffReadbuffLen(insert.data, insert.size as ptrdiff_t);
            // The quoted forms: `0` as `<C-V>048`, `^` as `<C-V>^`.
            if last == b'0' as c_char {
                stuffReadbuffLen(c"\x16048".as_ptr(), 4);
            } else if last == b'^' as c_char {
                stuffReadbuffLen(c"\x16^".as_ptr(), 2);
            }
            count -= 1;
            if count <= 0 {
                break;
            }
        }

        // May want to stuff a trailing ESC, to get out of Insert mode.
        if no_esc == 0 {
            stuffcharReadbuff(ESC);
        }
        OK
    }
}

/// The last inserted text, without the command that started the insert.
///
/// Borrowed, not copied: the bytes belong to `last_insert`.
///
/// # Safety
/// The answer is invalidated by the next [`set_last_insert`] or insert.
pub(crate) unsafe fn get_last_insert() -> String_0 {
    unsafe {
        if (*last_insert.ptr()).data.is_null() {
            NULL_STRING
        } else {
            String_0 {
                data: (*last_insert.ptr())
                    .data
                    .offset(last_insert_skip.get() as isize),
                size: (*last_insert.ptr()).size - last_insert_skip.get() as size_t,
            }
        }
    }
}

/// The last inserted text as a fresh allocation, with the trailing `<Esc>`
/// removed.  Null when there is none; the caller frees it.
///
/// # Safety
/// Must run on the main thread.
pub(crate) unsafe fn get_last_insert_save() -> *mut c_char {
    unsafe {
        let mut insert = get_last_insert();
        if insert.data.is_null() {
            return ::core::ptr::null_mut();
        }

        let s = xmemdupz(insert.data as *const ::core::ffi::c_void, insert.size) as *mut c_char;
        if insert.size > 0 && *s.add(insert.size - 1) as c_int == ESC {
            insert.size -= 1;
            *s.add(insert.size) = NUL as c_char;
        }
        s
    }
}
