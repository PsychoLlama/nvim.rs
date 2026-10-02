//! The pattern-cursor moves the NFA compiler needs beyond the shared reader.
//!
//! [`super::super::parse`] hands out `pat_byte`/`pat_char`/`pat_seek`
//! relative to the cursor. The collection and literal parsers also read
//! relative to a *saved* cursor — where the atom being parsed started — and
//! step the cursor back over a character it has already passed. Those are
//! the only raw-pointer moves left in the compiler, so they all live here
//! and the parsers that use them stay checked.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::regexp::RegCompiler;
use core::ffi::{c_char, c_int};

use super::compile::nfa_recognize_char_class;
use crate::mbyte::{utf_head_off, utf_iscomposing_legacy, utf_ptr2char, utfc_ptr2len};
use crate::regexp::{CharClass, NfaOp, pat_seek, skip_anyof, take_bracketed, take_char_class};

/// The cursor, to hand back to the functions here as a saved position.
pub(crate) fn here(rc: &RegCompiler) -> *mut c_char {
    rc.cursor
}

/// Put the cursor at a position [`here`] returned, or at the end of a
/// collection.
pub(crate) fn seek_to(rc: &mut RegCompiler, p: *mut c_char) {
    rc.cursor = p;
}

/// Is the cursor still before `end`?
pub(crate) fn before(rc: &RegCompiler, end: *mut c_char) -> bool {
    rc.cursor < end
}

/// Where the collection at the cursor ends: its closing `]`, or the
/// pattern's NUL if it has none.
pub(crate) fn collection_end(rc: &RegCompiler) -> *mut c_char {
    // SAFETY: the cursor points into the NUL-terminated pattern and
    // `skip_anyof` stops at the terminator.
    unsafe { skip_anyof(rc.cursor, rc.cpo_lit) }
}

/// The byte at `p`.
pub(crate) fn byte_at(p: *mut c_char) -> u8 {
    // SAFETY: `p` is a position inside the pattern being parsed.
    unsafe { *p }.cast_unsigned()
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
pub(crate) fn step_back(rc: &mut RegCompiler, anchor: *mut c_char) {
    // SAFETY: the cursor is past `anchor`, which is where this atom began,
    // and `utf_head_off` walks back no further than `anchor`.
    let cursor = rc.cursor;
    let back = unsafe { utf_head_off(anchor, cursor.sub(1)) };
    let back = usize::try_from(back).expect("a head offset is never negative") + 1;
    rc.cursor = unsafe { cursor.sub(back) };
}

/// Is `c` a combining character?
pub(crate) fn is_composing(c: c_int) -> bool {
    utf_iscomposing_legacy(c)
}

/// Move the cursor past the whole character it is on.
pub(crate) fn advance_grapheme(rc: &mut RegCompiler) {
    let arg = here(rc);
    pat_seek(rc, grapheme_len(arg) as isize);
}

/// [`take_char_class`] against the cursor: a `[:alpha:]` at it, consumed.
pub(crate) fn take_cursor_char_class(rc: &mut RegCompiler) -> Option<CharClass> {
    // SAFETY: the cursor points into the NUL-terminated pattern, and
    // `take_char_class` only ever advances it -- it walks bytes and calls
    // nothing, so it cannot re-enter the cell it is handed.
    unsafe { take_char_class(&mut rc.cursor) }
}

/// [`take_bracketed`] against the cursor: a `[=a=]` or `[.a.]` at it.
pub(crate) fn take_cursor_bracketed(rc: &mut RegCompiler, delim: u8) -> c_int {
    // SAFETY: as `take_cursor_char_class`.
    unsafe { take_bracketed(&mut rc.cursor, delim) }
}

/// Is the collection between the cursor and `end` one of the character
/// classes? See [`nfa_recognize_char_class`].
pub(crate) fn recognize_char_class(
    rc: &RegCompiler,
    end: *mut c_char,
    accepts_newline: bool,
) -> Option<(NfaOp, bool)> {
    // SAFETY: `end` is this collection's closing `]`, found by
    // `collection_end` from the cursor.
    unsafe { nfa_recognize_char_class(here(rc).cast(), end.cast(), accepts_newline) }
}
