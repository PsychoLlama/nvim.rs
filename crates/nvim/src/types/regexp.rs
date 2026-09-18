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
use crate::winlayer::{Buf, Win};
use core::ops::Range;

/// The head every compiled pattern starts with, whichever engine built it.
#[repr(C)]
pub struct RegProg {
    pub engine: *mut RegEngine,
    pub regflags: ::core::ffi::c_uint,
    pub re_engine: ::core::ffi::c_uint,
    pub re_flags: ::core::ffi::c_uint,
    pub re_in_use: bool,
}
/// Compile a pattern, or null if it does not parse.
pub type RegComp = Option<unsafe fn(*mut uint8_t, ::core::ffi::c_int) -> *mut RegProg>;
/// Release a compiled pattern.
pub type RegFree = Option<unsafe fn(*mut RegProg) -> ()>;
/// Match within one line.
pub type RegExecNl =
    Option<unsafe fn(*mut RegMatch, *mut uint8_t, ColNr, bool) -> ::core::ffi::c_int>;
/// Match across lines of a buffer, with a timeout.
pub type RegExecMulti = Option<
    unsafe fn(
        *mut RegMMatch,
        Option<Win>,
        Buf,
        LineNr,
        ColNr,
        *mut ProfTime,
        *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
/// The vtable of a regexp engine (backtracking or NFA).
pub struct RegEngine {
    pub regcomp: RegComp,
    pub regfree: RegFree,
    pub regexec_nl: RegExecNl,
    pub regexec_multi: RegExecMulti,
}

pub type Magic = ::core::ffi::c_uint;
pub type OptMagic = ::core::ffi::c_uint;
pub struct RegExtMatch {
    pub refcnt: int16_t,
    pub matches: [*mut uint8_t; 10],
}
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
#[derive(Clone)]
pub struct RegMatch {
    pub regprog: *mut RegProg,
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
    /// `regprog` may be null, which every caller that compiles a pattern
    /// into the structure afterwards relies on.
    pub const fn new(regprog: *mut RegProg, ignore_case: bool) -> Self {
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
#[derive(Clone)]
pub struct RegMMatch {
    pub regprog: *mut RegProg,
    pub startpos: [LPos; 10],
    pub endpos: [LPos; 10],
    pub rmm_matchcol: ColNr,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: ColNr,
}

impl Default for RegMMatch {
    fn default() -> Self {
        RegMMatch {
            regprog: ::core::ptr::null_mut(),
            startpos: [LPos::default(); 10],
            endpos: [LPos::default(); 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        }
    }
}

impl Default for RegMatch {
    fn default() -> Self {
        RegMatch::new(::core::ptr::null_mut(), false)
    }
}
