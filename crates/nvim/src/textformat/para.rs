//! What counts as a paragraph, and when two lines belong to the same one.
//!
//! [`fmt_check_par`] answers "is this line *not* part of a paragraph" --
//! blank, or nothing but a comment leader, or the end of a block comment --
//! and [`same_leader`] whether two lines carry leaders that let them be
//! joined. [`paragraph_start`] is the pair asked about one line.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::ascii::ascii_iswhite;
use crate::change::get_leader_len;
use crate::charset::skip;
use crate::cstr::byte_at;
use crate::indent::get_number_indent;
use crate::memline::Lines;
use crate::textobject::starts_para;
use crate::types::{LineNr, NUL};

/// A line's comment leader: how many bytes of it there are, and where the
/// 'comments' item that matched begins.
///
/// `flags` points into the 'comments' option itself, so it stays valid for as
/// long as the option does -- which is what lets `format_lines` carry one
/// from the "next line" slot to the "current line" slot without copying.
#[derive(Clone, Copy)]
pub(crate) struct Leader {
    /// Bytes of leader, including the white space in front of it.
    pub(crate) len: c_int,
    /// The flag letters of the matching 'comments' item, or null.
    pub(crate) flags: *mut c_char,
}

impl Leader {
    /// The "no leader at all" answer, which is also what a line gets when
    /// comments are not being formatted.
    pub(crate) const NONE: Leader = Leader {
        len: 0,
        flags: ::core::ptr::null_mut(),
    };

    /// Whether the item's flag letters contain `flag`, searching only the
    /// current item -- `:` starts the next one.
    ///
    /// # Safety
    /// `self.flags` must be null or NUL-terminated.
    unsafe fn has_flag(self, flag: c_int) -> bool {
        let mut p = self.flags;
        if p.is_null() {
            return false;
        }
        while unsafe { *p } as c_int != NUL && unsafe { *p } as c_int != ':' as c_int {
            if unsafe { *p } as c_int == flag {
                return true;
            }
            p = unsafe { p.add(1) };
        }
        false
    }
}

/// Whether line `lnum` is *not* part of a paragraph, updating `leader`.
///
/// Blank lines, and lines holding nothing but a comment leader, are left
/// untouched by formatting. So is a line starting with the *end* of a block
/// comment (`e` in the 'comments' flags), so that it is skipped rather than
/// joined to the line above. A paragraph also starts after a blank line, or
/// wherever the comment leader changes.
///
/// `leader` is in/out rather than an answer, and deliberately so:
/// `get_leader_len` writes `flags` for every 'comments' item it *tries*, so
/// after a line with no leader it names the last item rather than nothing,
/// and `format_lines` goes on to read that -- see its `://` test. Clearing
/// it here would be a behaviour change under a 'comments' whose last item
/// begins `://`.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub(crate) unsafe fn fmt_check_par(lnum: LineNr, leader: &mut Leader, do_comments: bool) -> bool {
    // The borrow is over before `starts_para`, which walks the buffer.
    let nothing_but_leader = {
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        leader.len = if do_comments {
            // SAFETY: a line out of the cache, which is NUL-terminated
            // where the cache put it, and the caller's out-parameter.
            unsafe { get_leader_len(line.as_ptr().cast(), &raw mut leader.flags, false, true) }
        } else {
            0
        };
        let after = &line[(leader.len as usize).min(line.len())..];
        byte_at(after, skip::white(after)) == NUL as u8
    };
    let ends_a_comment = leader.len > 0 && unsafe { leader.has_flag(COM_END) };
    nothing_but_leader || ends_a_comment || unsafe { starts_para(lnum, NUL, false) }
}

/// Whether line `lnum` ends in a white character. An empty line does not.
pub(crate) fn ends_in_white(lnum: LineNr) -> bool {
    Lines::current()
        .line(lnum)
        .last()
        .is_some_and(|&c| ascii_iswhite(c_int::from(c)))
}

/// Whether the leaders of line `lnum` and the line after it are the same, so
/// that the two may be joined.
///
/// The whole of `first`'s leader must match `second.len` bytes of the next
/// line's, white space aside. Three flags decide it outright:
///
/// | flag | meaning |
/// | --- | --- |
/// | `f` | only if the second line has no leader at all |
/// | `e` | never: this leader ends a comment |
/// | `s` | only if there is text after it and the second line's item has `m` |
///
/// # Safety
/// `lnum` and `lnum + 1` must be valid lines of the current buffer.
pub(crate) unsafe fn same_leader(lnum: LineNr, first: Leader, second: Leader) -> bool {
    if first.len == 0 {
        return second.len == 0;
    }
    if !first.flags.is_null() {
        let mut p = first.flags;
        while unsafe { *p } as c_int != NUL && unsafe { *p } as c_int != ':' as c_int {
            match unsafe { *p } as c_int {
                COM_FIRST => return second.len == 0,
                COM_END => return false,
                COM_START => {
                    // A comment's opening line joins the next one only
                    // when it has text of its own and the next line's
                    // item is the comment's middle.
                    if Lines::current().line_len(lnum) <= first.len {
                        return false;
                    }
                    if second.flags.is_null() || second.len == 0 {
                        return false;
                    }
                    return unsafe { second.has_flag(COM_MIDDLE) };
                }
                _ => {}
            }
            p = unsafe { p.add(1) };
        }
    }

    // Compare the two leaders as text. The first line has to be copied:
    // only one line can be locked at a time.
    let mut lines = Lines::current();
    let line1 = lines.line_copy(lnum);
    let line2 = lines.line(lnum + 1);
    leaders_match(&line1, first.len as usize, line2, second.len as usize)
}

/// Whether `second`'s leading `second_len` bytes are the same leader as
/// `first`'s leading `first_len`, once white space is allowed to differ.
///
/// The two walks are not symmetric. `second` is read a byte at a time up to
/// its length; a white byte there matches a whole *run* of white space in
/// `first`, and anything else has to match exactly. It is a match when both
/// walks land on the end of their own leader at the same moment — a first
/// leader that has more to it, or a second byte that differs, is not one.
/// Leading white space in `first` is skipped before either walk starts,
/// because `get_leader_len` counts it as part of the leader and the caller's
/// `first_len` therefore includes it.
fn leaders_match(first: &[u8], first_len: usize, second: &[u8], second_len: usize) -> bool {
    let mut idx1 = 0usize;
    while ascii_iswhite(c_int::from(byte_at(first, idx1))) {
        idx1 += 1;
    }
    let mut idx2 = 0usize;
    while idx2 < second_len {
        let c = byte_at(second, idx2);
        if ascii_iswhite(c_int::from(c)) {
            // White space in the second leader matches any run of it in
            // the first.
            while ascii_iswhite(c_int::from(byte_at(first, idx1))) {
                idx1 += 1;
            }
        } else {
            let c1 = byte_at(first, idx1);
            idx1 += 1;
            if c1 != c {
                break;
            }
        }
        idx2 += 1;
    }
    idx2 == second_len && idx1 == first_len
}

/// Whether a paragraph starts at line `lnum` -- that is, whether the line
/// above it is *not* in the same paragraph. Used by 'formatoptions' `a`.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub(crate) unsafe fn paragraph_start(lnum: LineNr) -> bool {
    if lnum <= 1 {
        return true; // start of the file
    }
    if Lines::current().line(lnum - 1).is_empty() {
        return true; // after an empty line
    }
    let do_comments = has_format_option(FoFlag::Q_COMS);
    let mut prev = Leader::NONE;
    let mut this = Leader::NONE;
    if unsafe { fmt_check_par(lnum - 1, &mut prev, do_comments) } {
        return true; // after a non-paragraph line
    }
    if unsafe { fmt_check_par(lnum, &mut this, do_comments) } {
        return true; // `lnum` is not a paragraph line
    }
    if has_format_option(FoFlag::WHITE_PAR) && !ends_in_white(lnum - 1) {
        return true; // the previous line is missing its trailing space
    }
    if has_format_option(FoFlag::Q_NUMBER) && unsafe { get_number_indent(lnum) } > 0 {
        return true; // a numbered item starts at `lnum`
    }
    // A change of comment leader.
    !unsafe { same_leader(lnum - 1, prev, this) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `same_leader`'s comparison, asked the way it asks it: the first
    /// line's whole leader against the second's, both measured by
    /// `get_leader_len` and so both counting the white space in front.
    fn matches(first: &[u8], first_len: usize, second: &[u8], second_len: usize) -> bool {
        leaders_match(first, first_len, second, second_len)
    }

    #[test]
    fn the_same_leader_twice_matches() {
        assert!(matches(b" * one", 3, b" * two", 3));
        assert!(matches(b"// one", 3, b"// two", 3));
    }

    #[test]
    fn a_different_leader_does_not() {
        assert!(!matches(b" * one", 3, b" # two", 3));
        // The first leader has more to it than the second matched.
        assert!(!matches(b" ** one", 4, b" * two", 3));
    }

    #[test]
    fn white_space_in_the_second_leader_swallows_a_run_in_the_first() {
        // Two spaces after the star on the left, one on the right.
        assert!(matches(b" *  one", 4, b" * two", 3));
        // And none on the right at all: the run is still swallowed, but
        // then the first walk has not reached `first_len`.
        assert!(!matches(b" *  one", 4, b" *two", 2));
    }

    #[test]
    fn leading_white_space_is_skipped_before_either_walk() {
        // Three spaces on the left, none on the right. `first_len` counts
        // the spaces -- `get_leader_len` measures from column 0 -- so the
        // walk that skipped them still has to end on it.
        assert!(matches(b"   * one", 4, b"* two", 1));
        assert!(!matches(b"   * one", 5, b"* two", 1));
    }

    #[test]
    fn an_empty_second_leader_matches_only_an_empty_first() {
        assert!(matches(b"", 0, b"", 0));
        assert!(!matches(b" * one", 3, b"two", 0));
        // A blank first line with a zero length is not a match: the skip
        // runs off the end and `idx1` is the line's length, not 0. It is
        // also unreachable -- `same_leader` answers `second.len == 0`
        // before it gets here -- and pinned so that stays visible.
        assert!(!matches(b"   ", 0, b"", 0));
    }

    #[test]
    fn a_leader_the_line_is_too_short_for_reads_the_terminator() {
        // Neither walk may step outside its slice; past the end is NUL,
        // which matches nothing and ends the comparison.
        assert!(!matches(b"//", 2, b"//x", 3));
        assert!(!matches(b"/", 2, b"//", 2));
    }
}
