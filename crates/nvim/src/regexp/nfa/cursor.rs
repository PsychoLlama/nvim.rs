//! The pattern-cursor moves the NFA compiler needs beyond the shared reader.
//!
//! [`super::super::parse`] hands out `pat_byte`/`pat_char`/`pat_seek`
//! relative to the cursor. The collection and literal parsers also read
//! relative to a *saved* cursor — where the atom being parsed started — and
//! step the cursor back over a character it has already passed. Those are
//! the only moves of the cursor that are not relative to it, so they all
//! live here.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::regexp::RegCompiler;
use core::ffi::c_int;

use super::compile::nfa_recognize_char_class;
use crate::cstr;
use crate::mbyte::utf_iscomposing_legacy;
use crate::regexp::{
    CharClass, NfaOp, char_at_nul, cluster_len_at, head_off_at, pat_seek, skip_anyof,
    take_bracketed, take_char_class,
};

/// The cursor, to hand back to the functions here as a saved position.
pub(crate) fn here(rc: &RegCompiler) -> usize {
    rc.cursor
}

/// Put the cursor at a position [`here`] returned, or at the end of a
/// collection.
pub(crate) fn seek_to(rc: &mut RegCompiler, p: usize) {
    rc.cursor = p;
}

/// Is the cursor still before `end`?
pub(crate) fn before(rc: &RegCompiler, end: usize) -> bool {
    rc.cursor < end
}

/// Where the collection at the cursor ends: its closing `]`, or the
/// pattern's NUL if it has none.
pub(crate) fn collection_end(rc: &RegCompiler) -> usize {
    skip_anyof(rc.text, rc.cursor, rc.cpo_lit)
}

/// The byte at `p`.
pub(crate) fn byte_at(rc: &RegCompiler, p: usize) -> u8 {
    cstr::byte_at(rc.text, p)
}

/// The encoded length of the whole character at `p` — its base character
/// plus any combining marks.
pub(crate) fn grapheme_len(rc: &RegCompiler, p: usize) -> c_int {
    c_int::try_from(cluster_len_at(rc.text, p)).expect("a character is a few bytes long")
}

/// The character `off` bytes past `p`.
pub(crate) fn char_at(rc: &RegCompiler, p: usize, off: c_int) -> c_int {
    char_at_nul(rc.text, p.wrapping_add_signed(off as isize))
}

/// Step the cursor back over the character in front of it. `anchor` bounds
/// how far the search for that character's first byte may go.
pub(crate) fn step_back(rc: &mut RegCompiler, anchor: usize) {
    // The cursor is past `anchor`, which is where this atom began, and the
    // walk back stops at `anchor`.
    let back = head_off_at(&rc.text[anchor..], rc.cursor - 1 - anchor) + 1;
    rc.cursor -= back;
}

/// Is `c` a combining character?
pub(crate) fn is_composing(c: c_int) -> bool {
    utf_iscomposing_legacy(c)
}

/// Move the cursor past the whole character it is on.
pub(crate) fn advance_grapheme(rc: &mut RegCompiler) {
    let arg = here(rc);
    pat_seek(rc, grapheme_len(rc, arg) as isize);
}

/// [`take_char_class`] against the cursor: a `[:alpha:]` at it, consumed.
pub(crate) fn take_cursor_char_class(rc: &mut RegCompiler) -> Option<CharClass> {
    take_char_class(rc.text, &mut rc.cursor)
}

/// [`take_bracketed`] against the cursor: a `[=a=]` or `[.a.]` at it.
pub(crate) fn take_cursor_bracketed(rc: &mut RegCompiler, delim: u8) -> c_int {
    take_bracketed(rc.text, &mut rc.cursor, delim)
}

/// Is the collection between the cursor and `end` one of the character
/// classes? See [`nfa_recognize_char_class`].
pub(crate) fn recognize_char_class(
    rc: &RegCompiler,
    end: usize,
    accepts_newline: bool,
) -> Option<(NfaOp, bool)> {
    nfa_recognize_char_class(rc.text, here(rc), end, accepts_newline)
}
