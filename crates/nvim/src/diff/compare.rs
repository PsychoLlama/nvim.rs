//! When two lines count as equal.
//!
//! `'diffopt'`'s `iwhite`, `iwhiteall`, `iwhiteeol`, `iblank` and `icase` all
//! mean "ignore this difference", and this is where they are applied:
//! [`lines_equal`] compares two lines under the current flags, [`char_len`] is
//! the character-level rule underneath it, and [`diff_equal_entry`] lifts the
//! answer to a whole diff block.  Only the external diff needs them -- the
//! internal one passes the flags down to `xdl_diff` -- but the block-level
//! answers are read on both paths.
//!
//! Every line here arrives as the byte range of a **NUL-terminated** line, so
//! a tail slice reads as the C string it came from: the cluster and character
//! readers are the slice forms that stop where the terminator was.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr::byte_at;
use crate::mbyte::{c_cluster_len, char_at};
use crate::os::cshim::byte_tolower;
use core::ffi::c_int;

/// Skip a run of spaces and tabs: `charset.rs`'s `skipwhite` over a slice.
///
/// The two agree by construction -- `skipwhite` advances while
/// `ascii_iswhite`, which is exactly `b' '` and `b'\t'`.
pub(crate) fn skip_white(s: &[u8]) -> &[u8] {
    let n = s
        .iter()
        .position(|&b| b != b' ' && b != b'\t')
        .unwrap_or(s.len());
    &s[n..]
}

/// The length of the character both slices start with, if they are the same
/// character under `'diffopt'`'s `icase`.
///
/// Upstream's `diff_equal_char`, with the out-parameter turned into the
/// answer.  Two spellings of the C survive verbatim because callers depend on
/// them: an *empty* slice (a pointer at the terminator) has length 0, which
/// falls into the single-byte arm and compares NUL against NUL, so two lines
/// that have both ended report a shared character of length 1; and `icase`
/// folds through `utf_fold` for a multibyte character but through the
/// locale's `tolower` for a single-byte one, which are not the same map.
pub(crate) fn char_len(p1: &[u8], p2: &[u8]) -> Option<usize> {
    // Both slices are the tail of a NUL-terminated line, read as the C
    // string they are: the cluster ends at the terminator, so it is never
    // longer than the slice.
    let l = c_cluster_len(p1);
    if l != c_cluster_len(p2) {
        return None;
    }
    if l > 1 {
        if p1[..l] != p2[..l]
            && (diff_flags.get() & DIFF_ICASE == 0
                || utf_fold(char_at(p1)) != utf_fold(char_at(p2)))
        {
            return None;
        }
        Some(l)
    } else {
        let (b1, b2) = (byte_at(p1, 0), byte_at(p2, 0));
        if b1 != b2
            && (diff_flags.get() & DIFF_ICASE == 0
                || byte_tolower(c_int::from(b1)) != byte_tolower(c_int::from(b2)))
        {
            return None;
        }
        Some(1)
    }
}

/// Whether two lines count as equal under the current `'diffopt'`.
///
/// Upstream's `diff_cmp`, which answers `strcmp`'s convention; nothing reads
/// the sign, so this answers the question instead.
pub(crate) fn lines_equal(b1: &[u8], b2: &[u8]) -> bool {
    let flags = diff_flags.get();

    // `iblank`: a line that is blank once its indent is skipped matches
    // anything at all, including a non-blank line.
    if flags & DIFF_IBLANK != 0 && (skip_white(b1).is_empty() || skip_white(b2).is_empty()) {
        return true;
    }
    if flags & (DIFF_ICASE | ALL_WHITE_DIFF) == 0 {
        return b1 == b2;
    }
    if flags & DIFF_ICASE != 0 && flags & ALL_WHITE_DIFF == 0 {
        // A buffer line holds a NUL byte as an `NL`, so each side's own
        // length is what `mb_stricmp` reached by walking to the NUL.
        return crate::mbyte::strnicmp_in(b1, b2) == 0;
    }

    let (mut p1, mut p2) = (b1, b2);
    while !p1.is_empty() && !p2.is_empty() {
        let (w1, w2) = (
            ascii_iswhite(c_int::from(p1[0])),
            ascii_iswhite(c_int::from(p2[0])),
        );
        if flags & DIFF_IWHITE != 0 && w1 && w2 || flags & DIFF_IWHITEALL != 0 && (w1 || w2) {
            p1 = skip_white(p1);
            p2 = skip_white(p2);
        } else if let Some(l) = char_len(p1, p2) {
            p1 = &p1[l..];
            p2 = &p2[l..];
        } else {
            break;
        }
    }
    // Trailing white space is ignored on both sides whichever flag got us
    // here, so the lines are equal exactly when both walks reached the end.
    skip_white(p1).is_empty() && skip_white(p2).is_empty()
}

/// Whether a whole diff block holds the same text in two buffers.
pub(crate) fn diff_equal_entry(
    dp: &DiffBlock,
    slots: &DiffSlots,
    idx1: usize,
    idx2: usize,
) -> bool {
    if dp.df_count[idx1] != dp.df_count[idx2] {
        return false;
    }
    if diff_check_sanity(slots, dp).is_err() {
        return false;
    }
    let (mut b1, mut b2) = (
        Lines::in_buffer(used_slot(slots, idx1)),
        Lines::in_buffer(used_slot(slots, idx2)),
    );
    for i in 0..dp.df_count[idx1] {
        // Two handles, two buffers: `tp_diffbuf`'s entries are distinct
        // buffers, so each cache holds its own line and neither read
        // disturbs the other.
        let (l1, l2) = (dp.df_lnum[idx1] + i, dp.df_lnum[idx2] + i);
        if !lines_equal(b1.line(l1), b2.line(l2)) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::skip_white;
    use crate::cstr::byte_at;

    #[test]
    fn skip_white_takes_spaces_and_tabs_only() {
        assert_eq!(skip_white(b" \t x"), b"x");
        assert_eq!(skip_white(b"x  "), b"x  ");
        assert_eq!(skip_white(b"   "), b"");
        assert_eq!(skip_white(b""), b"");
        // A newline is not white space to `ascii_iswhite`.
        assert_eq!(skip_white(b" \n "), b"\n ");
    }

    #[test]
    fn byte_at_reads_the_terminator_past_the_end() {
        assert_eq!(byte_at(b"ab", 0), b'a');
        assert_eq!(byte_at(b"ab", 2), 0);
        assert_eq!(byte_at(b"", 0), 0);
    }
}
