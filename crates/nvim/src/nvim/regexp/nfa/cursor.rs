//! The pattern-cursor moves the NFA compiler needs beyond the shared reader.
//!
//! [`super::super::parse`] hands out `pat_byte`/`pat_char`/`pat_seek`
//! relative to the cursor. The collection and literal parsers also read
//! relative to a *saved* cursor — where the atom being parsed started — and
//! step the cursor back over a character it has already passed. Those are
//! the only raw-pointer moves left in the compiler, so they all live here
//! and the parsers that use them stay checked.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::compile::nfa_recognize_char_class;
use crate::src::nvim::mbyte::{utf_head_off, utf_iscomposing_legacy, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::regexp::{
    NFA_ADD_NL, pat_seek, regparse, skip_anyof, take_bracketed, take_char_class,
};

/// The cursor, to hand back to the functions here as a saved position.
pub(crate) fn here() -> *mut c_char {
    regparse.get()
}

/// Put the cursor at a position [`here`] returned, or at the end of a
/// collection.
pub(crate) fn seek_to(p: *mut c_char) {
    regparse.set(p);
}

/// Is the cursor still before `end`?
pub(crate) fn before(end: *mut c_char) -> bool {
    regparse.get() < end
}

/// Where the collection at the cursor ends: its closing `]`, or the
/// pattern's NUL if it has none.
pub(crate) fn collection_end() -> *mut c_char {
    // SAFETY: the cursor points into the NUL-terminated pattern and
    // `skip_anyof` stops at the terminator.
    unsafe { skip_anyof(regparse.get()) }
}

/// The byte at `p`.
pub(crate) fn byte_at(p: *mut c_char) -> u8 {
    // SAFETY: `p` is a position inside the pattern being parsed.
    unsafe { *p as u8 }
}

/// The encoded length of the whole character at `p` — its base character
/// plus any combining marks.
pub(crate) fn grapheme_len(p: *mut c_char) -> c_int {
    // SAFETY: as `byte_at`.
    unsafe { utfc_ptr2len(p) }
}

/// The character `off` bytes past `p`.
pub(crate) fn char_at(p: *mut c_char, off: c_int) -> c_int {
    // SAFETY: as `byte_at`; `off` stays inside the character `p` starts.
    unsafe { utf_ptr2char(p.offset(off as isize)) }
}

/// Step the cursor back over the character in front of it. `anchor` bounds
/// how far the search for that character's first byte may go.
pub(crate) fn step_back(anchor: *mut c_char) {
    // SAFETY: the cursor is past `anchor`, which is where this atom began,
    // and `utf_head_off` walks back no further than `anchor`.
    unsafe {
        let prev = regparse.get().sub(1);
        regparse.set(prev.sub(utf_head_off(anchor, prev) as usize));
    }
}

/// Is `c` a combining character?
pub(crate) fn is_composing(c: c_int) -> bool {
    // SAFETY: a pure test on a code point.
    unsafe { utf_iscomposing_legacy(c) }
}

/// Move the cursor past the whole character it is on.
pub(crate) fn advance_grapheme() {
    pat_seek(grapheme_len(here()) as isize);
}

/// [`take_char_class`] against the cursor: a `[:alpha:]` at it, consumed.
pub(crate) fn take_cursor_char_class() -> c_int {
    // SAFETY: the cursor points into the NUL-terminated pattern, and
    // `take_char_class` only ever advances it.
    unsafe { take_char_class(&mut *regparse.ptr()) }
}

/// [`take_bracketed`] against the cursor: a `[=a=]` or `[.a.]` at it.
pub(crate) fn take_cursor_bracketed(delim: u8) -> c_int {
    // SAFETY: as `take_cursor_char_class`.
    unsafe { take_bracketed(&mut *regparse.ptr(), delim) }
}

/// Is the collection between the cursor and `end` one of the character
/// classes? See [`nfa_recognize_char_class`].
pub(crate) fn recognize_char_class(end: *mut c_char, extra: c_int) -> c_int {
    // SAFETY: `end` is this collection's closing `]`, found by
    // `collection_end` from the cursor.
    unsafe { nfa_recognize_char_class(here().cast(), end.cast(), (extra == NFA_ADD_NL) as c_int) }
}
