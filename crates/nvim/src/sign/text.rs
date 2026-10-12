//! A sign's `text=`: parsing it into cells, and rendering it back.
//!
//! A sign is drawn in exactly [`SIGN_WIDTH`] cells, each holding one
//! `ScreenChar`. [`init_sign_text`] turns a `text=` value into those cells and
//! [`push_sign_text`] turns them back into bytes for `:sign list`,
//! `sign_getdefined()` and `'statuscolumn'`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::grid::schar_text;
use crate::mbyte::{c_cells_at, c_cluster_len, c_cluster_schar};
use crate::types::Failed;

/// Appends the bytes `cells` render as to `out`: each cell's glyph, up to
/// the first empty one -- `:sign list`, `sign_getdefined()`, an extmark's
/// `sign_text` and `'statuscolumn'`'s `%s`.
pub(crate) fn push_sign_text(cells: &[ScreenChar; 2], out: &mut Vec<u8>) {
    let mut buf = [0; MAX_SCHAR_SIZE as usize];
    for &cell in cells {
        let glyph = schar_text(cell, &mut buf);
        if glyph.is_empty() {
            break;
        }
        out.extend_from_slice(glyph);
    }
}

/// [`push_sign_text`] into a buffer of its own.
pub(crate) fn sign_text_bytes(cells: &[ScreenChar; 2]) -> Vec<u8> {
    let mut out = Vec::new();
    push_sign_text(cells, &mut out);
    out
}

/// `text` with one level of backslash escaping removed: a backslash is
/// dropped and whatever follows it kept, so `text=\ x` can carry a space and
/// `text=\\` a backslash. The **last** byte is never examined, so a value
/// ending in a lone backslash keeps it -- upstream's loop stops one short,
/// and `:sign define x text=a\` shows `a\`.
///
/// Upstream does this in place, in whatever buffer the text arrived in --
/// for `sign_define()` that is the caller's own dictionary, which it
/// rewrites. This answers a new buffer instead.
pub(crate) fn unescape(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut at = 0;
    while at + 1 < text.len() {
        if text[at] == b'\\' {
            out.push(text[at + 1]);
            at += 2;
        } else {
            out.push(text[at]);
            at += 1;
        }
    }
    out.extend_from_slice(&text[at..]);
    out
}

/// Parses a sign's text into `cells`; `Err` when it does not fit.
///
/// `text` is the string's bytes, already unescaped where the caller wants
/// that (`:sign define` and `sign_define()` do, `nvim_buf_set_extmark` does
/// not), and diagnosing a bad value is the caller's too.
///
/// A one-cell text is padded to two with a space, and a two-cell character
/// blanks the second cell so the drawing code knows not to emit it. A text
/// that fails leaves whatever cells it had already written, as upstream's
/// does: a redefinition with a bad `text=` keeps the good part.
pub(crate) fn init_sign_text(text: &[u8], cells: &mut [ScreenChar; 2]) -> Result<(), Failed> {
    let width_limit = cells.len();
    let mut used = 0usize;
    let mut at = 0;
    // Count display cells, stopping at the first unprintable character.
    while at < text.len() {
        let rest = &text[at..];
        let (sc, c) = c_cluster_schar(rest);
        // This walk runs to the end of `text` and only tests the width
        // afterwards, so upstream (v0.12.4) overruns the array for anything
        // wider: on the heap via `:sign define x text=xxx`, on the STACK via
        // nvim_buf_set_extmark{sign_text=..}. Dropping the out-of-range
        // stores is unobservable -- every path that gets here with
        // `used >= SIGN_WIDTH` goes on to fail.
        if used < width_limit {
            cells[used] = sc;
        }
        if !vim_isprintc(c) {
            break;
        }
        let width = usize::try_from(c_cells_at(rest)).expect("a cell width is not negative");
        if width == 2 && used + 1 < width_limit {
            cells[used + 1] = 0;
        }
        used += width;
        at += c_cluster_len(rest);
    }

    // Must be empty, one cell or two; stopping short of the end means the
    // walk met an unprintable character.
    if at != text.len() || used > width_limit {
        return Err(Failed);
    }
    if used < 1 {
        cells[0] = 0;
    } else if used == 1 {
        cells[1] = ScreenChar::from(b' ');
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::unescape;

    fn unescaped(text: &str) -> String {
        String::from_utf8(unescape(text.as_bytes())).expect("still UTF-8")
    }

    #[test]
    fn a_backslash_escapes_the_byte_after_it() {
        assert_eq!(" x", unescaped("\\ x"));
        assert_eq!("a\\b", unescaped("a\\\\b"));
    }

    #[test]
    fn text_with_no_backslash_is_left_alone() {
        assert_eq!("ab", unescaped("ab"));
        assert_eq!("", unescaped(""));
    }

    /// The loop stops one byte short of the end, so a value ending in a lone
    /// backslash keeps it. Upstream's behaviour, pinned here because it is
    /// the one thing about this loop that looks like a bug and is not ours.
    #[test]
    fn a_trailing_backslash_survives() {
        assert_eq!("a\\", unescaped("a\\"));
        assert_eq!("\\", unescaped("\\"));
    }

    #[test]
    fn every_backslash_in_a_run_is_taken_in_turn() {
        assert_eq!("\\\\", unescaped("\\\\\\\\"));
        assert_eq!("ab", unescaped("\\a\\b"));
    }
}
