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
//! a tail slice's `as_ptr()` is still a valid C string.  That is what lets the
//! three mbyte calls (`utfc_ptr2len`, `utf_ptr2char`, `utf_head_off`) keep
//! their pointer signatures while the arithmetic around them is ordinary
//! slice indexing.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::FAIL;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

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

/// The byte at `i`, answering NUL past the end.
///
/// The C reads these lines through a pointer, so an index at or past the
/// terminator reads the NUL rather than going out of bounds; several of the
/// walks below deliberately step one past the last character.
pub(crate) fn byte_at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// `utf_head_off` over a slice: how far back from byte `i` the character
/// covering it starts.  Composing characters count as part of the character
/// they follow, which is why this is not just "skip continuation bytes".
pub(crate) fn head_off(line: &[u8], i: usize) -> usize {
    // SAFETY: `line` is the byte range of a NUL-terminated line, so `base`
    // and `base + i` are both within one allocation, `i == line.len()`
    // addressing the terminator itself.
    unsafe {
        let base = line.as_ptr().cast::<c_char>();
        utf_head_off(base, base.add(i)) as usize
    }
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
    // SAFETY: both slices are the tail of a NUL-terminated line, so the mbyte
    // walks stop at the terminator and never leave the allocation.  A length
    // longer than the slice is therefore impossible; were it not, the
    // indexing below would panic rather than read out of bounds.
    unsafe {
        let s1 = p1.as_ptr().cast::<c_char>();
        let s2 = p2.as_ptr().cast::<c_char>();
        let l = utfc_ptr2len(s1);
        if l != utfc_ptr2len(s2) {
            return None;
        }
        if l > 1 {
            let l = l as usize;
            if p1[..l] != p2[..l]
                && (diff_flags.get() & DIFF_ICASE == 0
                    || utf_fold(utf_ptr2char(s1)) != utf_fold(utf_ptr2char(s2)))
            {
                return None;
            }
            Some(l)
        } else {
            let (b1, b2) = (byte_at(p1, 0), byte_at(p2, 0));
            if b1 != b2
                && (diff_flags.get() & DIFF_ICASE == 0
                    || tolower(b1 as c_int) != tolower(b2 as c_int))
            {
                return None;
            }
            Some(1)
        }
    }
}

/// Whether two lines count as equal under the current `'diffopt'`.
///
/// Upstream's `diff_cmp`, which answers `strcmp`'s convention; nothing reads
/// the sign, so this answers the question instead.
pub(crate) fn lines_equal(s1: &CStr, s2: &CStr) -> bool {
    let flags = diff_flags.get();
    let (b1, b2) = (s1.to_bytes(), s2.to_bytes());

    // `iblank`: a line that is blank once its indent is skipped matches
    // anything at all, including a non-blank line.
    if flags & DIFF_IBLANK != 0 && (skip_white(b1).is_empty() || skip_white(b2).is_empty()) {
        return true;
    }
    if flags & (DIFF_ICASE | ALL_WHITE_DIFF) == 0 {
        return b1 == b2;
    }
    if flags & DIFF_ICASE != 0 && flags & ALL_WHITE_DIFF == 0 {
        // SAFETY: `CStr` guarantees both are NUL-terminated.
        return unsafe { mb_stricmp(s1.as_ptr(), s2.as_ptr()) } == 0;
    }

    let (mut p1, mut p2) = (b1, b2);
    while !p1.is_empty() && !p2.is_empty() {
        let (w1, w2) = (ascii_iswhite(p1[0] as c_int), ascii_iswhite(p2[0] as c_int));
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
pub(crate) unsafe fn diff_equal_entry(dp: *mut diff_T, idx1: usize, idx2: usize) -> bool {
    unsafe {
        if (*dp).df_count[idx1] != (*dp).df_count[idx2] {
            return false;
        }
        let tp = curtab.get();
        if diff_check_sanity(tp, dp) == FAIL {
            return false;
        }
        for i in 0..(*dp).df_count[idx1] {
            // The copy is not optional: the second `ml_get_buf` invalidates
            // the buffer the first one answered with.
            let line = CStr::from_ptr(ml_get_buf((*tp).tp_diffbuf[idx1], (*dp).df_lnum[idx1] + i))
                .to_owned();
            let other = CStr::from_ptr(ml_get_buf((*tp).tp_diffbuf[idx2], (*dp).df_lnum[idx2] + i));
            if !lines_equal(&line, other) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{byte_at, skip_white};

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
