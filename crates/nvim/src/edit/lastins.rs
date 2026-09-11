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
#![allow(unsafe_code)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::{Failed, NUL};

/// The buffer `.` repeats, by address.
///
/// A handle rather than `get`/`set`: the cell owns the text, so it is read
/// through a borrow and written by handing over a new one.
#[derive(Clone, Copy)]
pub(super) struct LastInsert(*mut String_0);

/// The one place the last-insert buffer's address is taken.
pub(super) fn last_insert_slot() -> LastInsert {
    LastInsert(last_insert.ptr())
}

impl LastInsert {
    /// The whole buffer, borrowed from the cell.
    ///
    /// # Safety
    /// Nothing may [`replace`](Self::replace) the text while the borrow
    /// lasts.
    unsafe fn borrow<'a>(self) -> &'a String_0 {
        // SAFETY: the only constructor names a `static`.
        unsafe { &*self.0 }
    }

    /// Release what is there and take ownership of `text`.
    ///
    /// # Safety
    /// Nothing may still be holding a [`borrow`](Self::borrow) of the old
    /// text.
    pub(super) unsafe fn replace(self, text: String_0) {
        // SAFETY: the cell's own text, which the assignment releases.
        unsafe { *self.0 = text };
    }
}

/// Set the last inserted text to the single character `c`.
///
/// Used by `r`.  What is stored is the *redo buffer* spelling: a CTRL-V in
/// front of a control character, then the character, then the `<Esc>` that
/// ends an insert.
///
/// # Safety
/// Must run on the main thread; frees and replaces `last_insert`.
pub(crate) unsafe fn set_last_insert(c: c_int) {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let start =
        unsafe { xmalloc((MB_MAXBYTES as c_int * 3 + 5) as size_t) } as *mut ::core::ffi::c_char;
    let mut s = start;
    // The CTRL-V is only needed to enter a special character.
    if c < ' ' as c_int || c == DEL {
        unsafe { *s = Ctrl_V as c_char };
        s = unsafe { s.offset(1) };
    }
    s = unsafe { add_char2buf(c, s) };
    unsafe { *s = ESC as c_char };
    s = unsafe { s.offset(1) };
    unsafe { *s = NUL as c_char };

    let len = unsafe { s.offset_from(start) } as size_t;
    // SAFETY: `start` is this function's own allocation, NUL-terminated at
    // `len` by the write above.
    let text = unsafe { String_0::from_owned_parts(start, len) };
    // SAFETY: nothing borrows the old text here.
    unsafe { last_insert_slot().replace(text) };
    last_insert_skip.set(0);
}

/// `.`, `i_CTRL-A` and `i_CTRL-@`: stuff the last inserted text into the read
/// buffer so the main loop types it again.
///
/// `c` is the command character that starts Insert mode (NUL for none),
/// `count` how many times, and `no_esc` says to leave the insert open at the
/// end.
///
/// `Err` -- with `E29` -- when there is nothing to insert.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn stuff_inserted(
    c: c_int,
    mut count: c_int,
    no_esc: c_int,
) -> Result<(), Failed> {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let mut insert = unsafe { get_last_insert() }; // text to be inserted
    if insert.data().is_null() {
        emsg(gettext(e_noinstext));
        return Err(Failed);
    }

    // May want to stuff the command character, to start Insert mode.
    if c != NUL {
        stuff_readbuf_char(c);
    }

    // Cut the text at the last ESC: what follows it is not part of the
    // insert.
    let mut i = insert.len();
    while i > 0 {
        i -= 1;
        if unsafe { *insert.data().add(i) } as c_int == ESC {
            insert = String_0::from_bytes(&insert.as_bytes()[..i]);
            break;
        }
    }

    // A trailing `0` or `^` has to be quoted, because either would be
    // read as the start of `0 CTRL-D`/`^ CTRL-D` -- but only when
    // nothing follows it (no ESC is coming) or when the text is repeated
    // and starts with CTRL-D.  -- Acevedo
    let mut last = NUL as c_char;
    if !insert.is_empty() {
        let p = unsafe { insert.data().add(insert.len() - 1) };
        if (unsafe { *p } as c_int == '0' as c_int || unsafe { *p } as c_int == '^' as c_int)
            && (no_esc != 0 || (unsafe { *insert.data() } as c_int == Ctrl_D && count > 1))
        {
            last = unsafe { *p };
            let shorter = insert.len() - 1;
            insert = String_0::from_bytes(&insert.as_bytes()[..shorter]);
        }
    }

    loop {
        unsafe { stuff_readbuf_len(insert.data(), insert.len() as ptrdiff_t) };
        // The quoted forms: `0` as `<C-V>048`, `^` as `<C-V>^`.
        if last == b'0' as c_char {
            unsafe { stuff_readbuf_len(c"\x16048".as_ptr(), 4) };
        } else if last == b'^' as c_char {
            unsafe { stuff_readbuf_len(c"\x16^".as_ptr(), 2) };
        }
        count -= 1;
        if count <= 0 {
            break;
        }
    }

    // May want to stuff a trailing ESC, to get out of Insert mode.
    if no_esc == 0 {
        stuff_readbuf_char(ESC);
    }
    Ok(())
}

/// The last inserted text, without the command that started the insert.
///
/// A copy: both callers shorten what they get, and the stored text is not
/// theirs to shorten.
///
/// # Safety
/// Must run on the main thread.
pub(crate) unsafe fn get_last_insert() -> String_0 {
    // SAFETY: the borrow does not outlive this body, which stores nothing.
    let all = unsafe { last_insert_slot().borrow() };
    if all.is_null() {
        return String_0::NULL;
    }
    // `last_insert_skip` counts bytes this module put on the front, so it
    // never runs past the buffer.
    let skip = last_insert_skip.get() as size_t;
    String_0::from_bytes(&all.as_bytes()[skip..])
}

/// The last inserted text as a fresh allocation, with the trailing `<Esc>`
/// removed.  Null when there is none; the caller frees it.
///
/// # Safety
/// Must run on the main thread.
pub(crate) unsafe fn get_last_insert_save() -> *mut c_char {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let insert = unsafe { get_last_insert() };
    if insert.data().is_null() {
        return ::core::ptr::null_mut();
    }

    let s = unsafe { xmemdupz(insert.data() as *const ::core::ffi::c_void, insert.len()) }
        as *mut c_char;
    if !insert.is_empty() && unsafe { *s.add(insert.len() - 1) } as c_int == ESC {
        // The copy is shortened, not the string it came from.
        unsafe { *s.add(insert.len() - 1) = NUL as c_char };
    }
    s
}
