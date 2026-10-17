#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::regexp::OwnedProg;
use core::ops::Range;

pub type Magic = ::core::ffi::c_uint;
pub type OptMagic = ::core::ffi::c_uint;
/// A pattern and where its groups landed in the string it last ran over.
///
/// Upstream's `startp`/`endp` are `char *` into the matched text, which
/// makes every holder of a `regmatch_T` a holder of a borrow nothing
/// spells: the text has to outlive the structure, and the compiler is not
/// told. Here they are **byte offsets into the line the match ran over**,
/// the same shape [`RegMMatch`]'s `startpos`/`endpos` have always had, and
/// the line comes back at the reading end — [`RegMatch::group_bytes`] takes
/// it, and the caller that has no line to hand has a bug the type now
/// shows.
///
/// A group the pattern never filled is `None`. After a match that failed
/// every group is `None`: upstream leaves the last attempt's pointers
/// lying in the structure, which is only ever read by mistake.
///
/// The structure owns its program (see [`OwnedProg`]): a match borrows it
/// from here and puts back whatever it ends up with.
pub struct RegMatch {
    pub regprog: Option<OwnedProg>,
    /// Where each `\0`..`\9` group starts, and where it ends.
    pub starts: [Option<usize>; RE_GROUPS],
    pub ends: [Option<usize>; RE_GROUPS],
    pub rm_matchcol: ColNr,
    pub rm_ic: bool,
}

/// How many capture groups a match records: `\0` and `\1`..`\9`.
pub const RE_GROUPS: usize = 10;

impl RegMatch {
    /// A match structure for `regprog`, with no groups filled in yet.
    ///
    /// `regprog` may be `None`, which every caller that compiles a pattern
    /// into the structure afterwards relies on.
    pub const fn new(regprog: Option<OwnedProg>, ignore_case: bool) -> Self {
        RegMatch {
            regprog,
            starts: [None; RE_GROUPS],
            ends: [None; RE_GROUPS],
            rm_matchcol: 0,
            rm_ic: ignore_case,
        }
    }

    /// The span group `no` covers, or `None` if the pattern never filled
    /// both of its ends.
    pub fn group(&self, no: usize) -> Option<Range<usize>> {
        Some(self.starts[no]?..self.ends[no]?)
    }

    /// Group `no`'s text, given the line the match ran over.
    ///
    /// `None` for a group that did not match; an empty slice for one that
    /// matched nothing, which is a different answer.
    pub fn group_bytes<'a>(&self, no: usize, line: &'a [u8]) -> Option<&'a [u8]> {
        let span = self.group(no)?;
        line.get(span)
    }

    /// Forget every group, as a failed match does.
    pub fn clear_groups(&mut self) {
        self.starts = [None; RE_GROUPS];
        self.ends = [None; RE_GROUPS];
    }
}
/// A pattern and where its groups landed in the buffer it last ran over:
/// line and column pairs. Owns its program, as [`RegMatch`] does; a clone
/// copies the program.
#[derive(Clone, Default)]
pub struct RegMMatch {
    pub regprog: Option<OwnedProg>,
    pub startpos: [LPos; 10],
    pub endpos: [LPos; 10],
    pub rmm_matchcol: ColNr,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: ColNr,
}

impl Default for RegMatch {
    fn default() -> Self {
        RegMatch::new(None, false)
    }
}
