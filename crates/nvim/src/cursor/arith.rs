//! The cursor arithmetic that touches no pointer: the column clamps, the
//! 'wrap' target, the fold-skipping line count.
//!
//! These are the decisions [`super`]'s motion and validation functions make
//! between their buffer reads, lifted out of the raw-pointer code so they can
//! be stated — and tested — on their own. `tests/unit/cursor.rs` drives them
//! directly, which is also how Miri sees this half of the module.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::pos::MAXCOL;
use crate::types::{colnr_T, linenr_T};

/// The virtual column `coladvance2` aims at instead of `wcol` when 'wrap' is
/// on and `wcol` lies past the line's last screen line: the end of that screen
/// line, rather than a column the line does not have.
///
/// `width` is the text width of one screen line and `last_vcol` the line's own
/// last virtual column. Insert mode is allowed one column further, which is
/// what happens when typing reaches the right edge of the window.
pub fn wrap_target_col(wcol: colnr_T, width: c_int, last_vcol: c_int, insert: bool) -> colnr_T {
    if wcol / width > last_vcol / width && (!insert || wcol > last_vcol + 1) {
        (last_vcol / width + 1) * width - 1
    } else {
        wcol
    }
}

/// Undo the last step of `coladvance2`'s measuring walk, which always ends
/// one character past the one it wanted. Answers the byte index, the virtual
/// column and the width of the character now under the cursor.
///
/// `head` is the part of that width belonging to 'showbreak' and
/// 'breakindent' rather than to the character, and is not the cursor's to
/// stand on.
pub fn step_back(idx: c_int, col: colnr_T, csize: c_int, head: c_int) -> (c_int, colnr_T, c_int) {
    let csize = csize - head;
    (idx - 1, col - csize, csize)
}

/// The `coladd` that spans a `gap` of virtual columns between where the cursor
/// landed and where it was asked for. Zero unless the gap is real and
/// plausible: 'virtualedit' admits absurd columns, and `view_width` sets the
/// scale at which one stops being believable.
pub fn gap_coladd(gap: c_int, view_width: c_int) -> colnr_T {
    if gap > 0 && gap < MAXCOL - 2 * view_width {
        gap
    } else {
        0
    }
}

/// Where [`super::check_cursor_col`] puts a cursor sitting at column `col`
/// of a line of `len` bytes, and whether the result still has to be stepped
/// back onto a character's head byte.
///
/// `may_rest_on_nul` is the modes-and-options question of whether the cursor
/// is allowed the position just past the last character. It is a closure
/// because answering it costs several option lookups, and only one of the
/// branches below asks.
pub fn checked_col(
    col: colnr_T,
    len: colnr_T,
    may_rest_on_nul: impl FnOnce() -> bool,
) -> (colnr_T, bool) {
    if len == 0 {
        (0, false)
    } else if col >= len {
        if may_rest_on_nul() {
            (len, false)
        } else {
            (len - 1, true)
        }
    } else if col < 0 {
        (0, false)
    } else {
        (col, false)
    }
}

/// What [`super::check_cursor_col`] does with `coladd` once the column is
/// clamped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColAdd {
    /// Leave it alone: 'virtualedit' does not cover the whole buffer, so the
    /// cursor was never anywhere a column could not describe.
    Keep,
    /// Drop it: the cursor was at the end of the line, or the arithmetic
    /// stopped making sense.
    Zero,
    /// Keep the cursor where it virtually was; this is how far into the
    /// character it sits, still to be capped at that character's width.
    Carry(colnr_T),
}

/// Whether the cursor's old virtual position survives the column clamp.
/// `oldcoladd` is the *virtual* column it held — `col + coladd` before the
/// clamp — and `col` the column it holds now.
pub fn carried_coladd(oldcol: colnr_T, oldcoladd: colnr_T, col: colnr_T, ve_all: bool) -> ColAdd {
    if oldcol == MAXCOL {
        ColAdd::Zero
    } else if !ve_all {
        ColAdd::Keep
    } else if oldcoladd > col {
        ColAdd::Carry(oldcoladd - col)
    } else {
        ColAdd::Zero
    }
}

/// Lines from `from` to `to`, counting each closed fold as one. `fold_last`
/// answers the last line of the fold containing its argument, or that argument
/// itself when it is in none.
pub fn folded_line_span(
    mut from: linenr_T,
    to: linenr_T,
    fold_last: impl Fn(linenr_T) -> linenr_T,
) -> linenr_T {
    let mut span: linenr_T = 0;
    while from < to {
        from = fold_last(from) + 1;
        span += 1;
    }
    // A fold reached past `to`, so that last step was not a whole line.
    if from > to { span - 1 } else { span }
}
