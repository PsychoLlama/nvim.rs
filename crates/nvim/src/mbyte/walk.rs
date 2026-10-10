//! Moving around a UTF-8 buffer.
//!
//! Everything that takes a position in a text and answers another one, over
//! slices: [`head_off`] (in [`utf8`](super::utf8), with the readers it is
//! built from) walks back to the start of the grapheme cluster covering a
//! byte, [`cp_bounds`] gives both ends of the codepoint at once, and
//! [`char_count`] counts the clusters in a span. The cursor
//! adjusters at the bottom put a window's cursor back on a character start.
//!
//! All of it has to survive bytes that are not UTF-8 at all: a buffer can
//! hold anything, and the cursor can be anywhere in it. The rule throughout
//! is that a byte which is not part of a valid sequence is its own character,
//! one byte long — so a walk always terminates and never steps past the
//! position it was asked about.
//!
//! The pointer spellings (`utf_head_off`, `mb_prevptr`, `mb_off_next`,
//! `mb_utflen`, ...) live in [`utf8`](super::utf8) as shims over these, for
//! the callers whose cursor is still a pointer.

#![forbid(unsafe_code)]

use super::*;
use crate::types::Pos;
use crate::winlayer::{Buf, Win};

/// Both ends of the codepoint covering byte `at` of `bytes`, as offsets from
/// it.
///
/// The answer for anything that is not a valid sequence is `(0, 1)` -- "this
/// byte, on its own" -- which is what makes every caller's arithmetic safe on
/// arbitrary bytes. Unlike [`head_off`] this is about the *codepoint*, not
/// the cluster: composing characters are separate.
///
/// `bytes` is the text the walk may not leave in either direction: it never
/// looks before its start, and never past its end.
///
/// # Panics
///
/// If `at` is not a byte of `bytes`.
pub fn cp_bounds(bytes: &[u8], at: usize) -> CharBoundsOff {
    /// "This byte is its own character."
    const JUST_THIS_BYTE: CharBoundsOff = CharBoundsOff {
        begin_off: 0,
        end_off: 1,
    };
    let rest = &bytes[at..];
    assert!(!rest.is_empty(), "`at` is a byte of `bytes`");
    if rest[0] < 0x80 {
        return JUST_THIS_BYTE;
    }

    // How far back the lead byte may be: never before the start, and never
    // more than a sequence's worth.
    let max_back = at.min(MB_MAXCHAR - 1);
    let mut back = 0;
    while utf_is_trail_byte(bytes[at - back]) {
        if back == max_back {
            return JUST_THIS_BYTE;
        }
        back += 1;
    }

    // The sequence has to be complete *and* within what is left.
    let end_off = usize::from(utf8len_tab[usize::from(bytes[at - back])]);
    if end_off <= back || end_off - back > rest.len() {
        return JUST_THIS_BYTE;
    }
    if !bytes[at - back + 1..at - back + end_off]
        .iter()
        .all(|&byte| utf_is_trail_byte(byte))
    {
        return JUST_THIS_BYTE;
    }
    CharBoundsOff {
        begin_off: int8_t::try_from(back).expect("at most a sequence back"),
        end_off: int8_t::try_from(end_off - back).expect("at most a sequence forward"),
    }
}

/// How many grapheme clusters `bytes` holds. A NUL inside the slice is an
/// ordinary byte and counts as a character, rather than ending the count.
pub fn char_count(bytes: &[u8]) -> usize {
    clusters(bytes).count()
}

/// Whether a `coladd` of 1 at the character starting `text` is meaningless
/// and should go.
///
/// `coladd == 1` is `'virtualedit'` saying "one cell into the character".
/// That is only meaningful for a character that is *drawn* more than one
/// cell wide; a Tab has its own rules, and an unprintable character is drawn
/// as an escape whose cells are not the character's.
fn coladd_drops(text: &[u8]) -> bool {
    text.first().is_none_or(|&byte| c_int::from(byte) != TAB)
        && vim_isprintc(char_at(text))
        && ptr2cells_at(text) > 1
}

/// `pos` moved off the middle of a character in `buffer`, with a `coladd`
/// that no longer means anything cleared. Upstream's `mark_mb_adjustpos`.
pub(crate) fn snap_pos_to_char(buffer: Buf, pos: Pos) -> Pos {
    let mut adjusted = pos;
    if adjusted.col <= 0 && adjusted.coladd <= 1 {
        return adjusted;
    }
    let mut lines = buffer.lines();
    let line = lines.line(adjusted.lnum);
    let col = usize::try_from(adjusted.col).unwrap_or(0);
    if line.is_empty() || line.len() < col {
        adjusted.col = 0;
    } else {
        adjusted.col -= ColNr::try_from(head_off(line, col)).expect("a short step back");
    }
    let at = usize::try_from(adjusted.col).unwrap_or(0);
    if adjusted.coladd == 1 && coladd_drops(&line[at.min(line.len())..]) {
        adjusted.coladd = 0;
    }
    adjusted
}

/// Move the cursor off the middle of a character, if it is in one.
pub fn mb_adjust_cursor() {
    let mut window = Win::current();
    window.w_cursor = snap_pos_to_char(Buf::current(), window.w_cursor);
}

impl Win {
    /// Pull this window's cursor back onto a character start, and clear a
    /// `coladd` that no longer means anything. Upstream's
    /// `mb_check_adjust_col`.
    pub(crate) fn snap_cursor_to_char(mut self) {
        let oldcol = self.w_cursor.col;
        if oldcol == 0 {
            return;
        }
        let mut lines = self.buffer().lines();
        let line = lines.line(self.w_cursor.lnum);
        let len = ColNr::try_from(line.len()).unwrap_or(ColNr::MAX);
        if len == 0 || oldcol < 0 {
            self.w_cursor.col = 0;
        } else {
            if oldcol > len {
                self.w_cursor.col = len - 1;
            }
            let col = usize::try_from(self.w_cursor.col).unwrap_or(0);
            self.w_cursor.col -= ColNr::try_from(head_off(line, col)).expect("a short step back");
        }
        let at = usize::try_from(self.w_cursor.col).unwrap_or(0);
        if self.w_cursor.coladd == 1 && coladd_drops(&line[at.min(line.len())..]) {
            self.w_cursor.coladd = 0;
        }
    }
}
