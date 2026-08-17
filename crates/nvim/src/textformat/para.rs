//! What counts as a paragraph, and when two lines belong to the same one.
//!
//! [`fmt_check_par`] answers "is this line *not* part of a paragraph" --
//! blank, or nothing but a comment leader, or the end of a block comment --
//! and [`same_leader`] whether two lines carry leaders that let them be
//! joined. [`paragraph_start`] is the pair asked about one line.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::{c_char, c_int};

use super::*;
use crate::ascii::ascii_iswhite;
use crate::change::get_leader_len;
use crate::charset::skipwhite;
use crate::indent::{byte_at, get_number_indent};
use crate::memline::{ml_get, ml_get_len};
use crate::textobject::startPS;
use crate::types::linenr_T;

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
        unsafe {
            let mut p = self.flags;
            if p.is_null() {
                return false;
            }
            while *p as c_int != NUL && *p as c_int != ':' as c_int {
                if *p as c_int == flag {
                    return true;
                }
                p = p.add(1);
            }
            false
        }
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
pub(crate) unsafe fn fmt_check_par(lnum: linenr_T, leader: &mut Leader, do_comments: bool) -> bool {
    unsafe {
        let ptr = ml_get(lnum);
        leader.len = if do_comments {
            get_leader_len(ptr, &raw mut leader.flags, false, true)
        } else {
            0
        };
        let ends_a_comment = leader.len > 0 && leader.has_flag(COM_END);
        *skipwhite(ptr.offset(leader.len as isize)) as c_int == NUL
            || ends_a_comment
            || startPS(lnum, NUL, false)
    }
}

/// Whether line `lnum` ends in a white character.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub(crate) unsafe fn ends_in_white(lnum: linenr_T) -> bool {
    unsafe {
        let s = ml_get(lnum);
        if *s as c_int == NUL {
            return false;
        }
        let last = ml_get_len(lnum) - 1;
        ascii_iswhite(*s.offset(last as isize) as u8 as c_int)
    }
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
pub(crate) unsafe fn same_leader(lnum: linenr_T, first: Leader, second: Leader) -> bool {
    unsafe {
        if first.len == 0 {
            return second.len == 0;
        }
        if !first.flags.is_null() {
            let mut p = first.flags;
            while *p as c_int != NUL && *p as c_int != ':' as c_int {
                match *p as c_int {
                    COM_FIRST => return second.len == 0,
                    COM_END => return false,
                    COM_START => {
                        // A comment's opening line joins the next one only
                        // when it has text of its own and the next line's
                        // item is the comment's middle.
                        if ml_get_len(lnum) <= first.len {
                            return false;
                        }
                        if second.flags.is_null() || second.len == 0 {
                            return false;
                        }
                        return second.has_flag(COM_MIDDLE);
                    }
                    _ => {}
                }
                p = p.add(1);
            }
        }

        // Compare the two leaders as text. The first line has to be copied:
        // only one line can be locked at a time.
        let line1: Vec<u8> =
            ::core::slice::from_raw_parts(ml_get(lnum) as *const u8, ml_get_len(lnum) as usize)
                .to_vec();
        let mut idx1 = 0usize;
        while ascii_iswhite(c_int::from(byte_at(&line1, idx1))) {
            idx1 += 1;
        }
        let line2 = ::core::slice::from_raw_parts(
            ml_get(lnum + 1) as *const u8,
            ml_get_len(lnum + 1) as usize,
        );
        let mut idx2 = 0usize;
        while idx2 < second.len as usize {
            let c = byte_at(line2, idx2);
            if ascii_iswhite(c_int::from(c)) {
                // White space in the second leader matches any run of it in
                // the first.
                while ascii_iswhite(c_int::from(byte_at(&line1, idx1))) {
                    idx1 += 1;
                }
            } else {
                let c1 = byte_at(&line1, idx1);
                idx1 += 1;
                if c1 != c {
                    break;
                }
            }
            idx2 += 1;
        }
        idx2 == second.len as usize && idx1 == first.len as usize
    }
}

/// Whether a paragraph starts at line `lnum` -- that is, whether the line
/// above it is *not* in the same paragraph. Used by 'formatoptions' `a`.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub(crate) unsafe fn paragraph_start(lnum: linenr_T) -> bool {
    unsafe {
        if lnum <= 1 {
            return true; // start of the file
        }
        if *ml_get(lnum - 1) as c_int == NUL {
            return true; // after an empty line
        }
        let do_comments = has_format_option(FO_Q_COMS);
        let mut prev = Leader::NONE;
        let mut this = Leader::NONE;
        if fmt_check_par(lnum - 1, &mut prev, do_comments) {
            return true; // after a non-paragraph line
        }
        if fmt_check_par(lnum, &mut this, do_comments) {
            return true; // `lnum` is not a paragraph line
        }
        if has_format_option(FO_WHITE_PAR) && !ends_in_white(lnum - 1) {
            return true; // the previous line is missing its trailing space
        }
        if has_format_option(FO_Q_NUMBER) && get_number_indent(lnum) > 0 {
            return true; // a numbered item starts at `lnum`
        }
        // A change of comment leader.
        !same_leader(lnum - 1, prev, this)
    }
}
