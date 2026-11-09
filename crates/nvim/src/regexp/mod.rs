//! Regular expressions: compiling a pattern into a program and running it
//! over a string or a buffer.
//!
//! The public entry points live in [`api`]; [`substitute`] and [`submatch`]
//! are the substitution side. Two engines implement the same interface —
//! [`bt`], a backtracker, and [`nfa`], a pike VM — and `vim_regcomp` picks
//! between them from 'regexpengine' and a leading `\%#=`.
//!
//! This file is what they share: the C structures the engines still pass
//! around by pointer, the opcode enumerations, and the globals that stand
//! in for the C file-scope statics that outlive a match (the submatch
//! snapshot, the previous replacement). A match keeps its state in a
//! [`RegExec`] the API layer builds for it (see [`rex`]), and a compile in
//! a [`RegCompiler`] (see [`compiler`]). It holds no code.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub(crate) mod state;
use crate::ascii::CAR;
use crate::ascii::ESC;
use crate::ascii::NL;
use crate::ascii::TAB;
use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::os::libc::{int64_t, uint8_t};
use crate::types::{ColNr, LPos, LineNr};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int, c_uint};
use core::ops::Range;
/// Last-pattern selectors and the regexp-engine/flag bits.
pub const RE_SEARCH: ::core::ffi::c_int = 0;
pub const RE_SUBST: ::core::ffi::c_int = 1;
pub const RE_BOTH: ::core::ffi::c_int = 2;
pub const RE_LAST: ::core::ffi::c_int = 2;

// The bodies, split along the seam upstream's `#include`s left in regexp.c:
// the shared layer and the pattern parser here, one module per engine below.
// Every child carries its own import list; nothing globs this module any more.

mod api;
mod bt;
mod chars;
mod compiler;
mod context;
mod equi_class;
mod error;
mod mbyte;
mod nfa;
mod parse;
mod pos;
mod rex;
mod submatch;
mod substitute;

pub use self::api::*;
pub(crate) use self::bt::*;
pub(crate) use self::chars::*;
pub(crate) use self::compiler::*;
pub(crate) use self::context::*;
pub(crate) use self::error::Refused;
pub(crate) use self::error::{CompileError, Compiled, Engine, Spelled};
pub(crate) use self::mbyte::*;
pub(crate) use self::nfa::*;
pub(crate) use self::parse::*;
pub(crate) use self::pos::*;
pub(crate) use self::rex::*;
pub(crate) use self::submatch::*;
pub(crate) use self::substitute::*;
pub const NSUBEXP: c_uint = 10;
pub const MAGIC_ALL: Magic = 4;
pub const MAGIC_ON: Magic = 3;
pub const MAGIC_OFF: Magic = 2;
pub const MAGIC_NONE: Magic = 1;
pub const REGSUB_BACKSLASH: c_uint = 4;
pub const REGSUB_MAGIC: c_uint = 2;
pub const REGSUB_COPY: c_uint = 1;
/// The caller's bound on how long an NFA match may run: how many steps
/// since the clock was last read. The deadline itself is the match
/// context's ([`Rex::deadline`]). The default is [`TimeBudget::NONE`].
#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct TimeBudget {
    count: c_int,
}

impl TimeBudget {
    pub(crate) const NONE: TimeBudget = TimeBudget { count: 0 };

    /// Count a step; true once every `interval` of them, when the clock is
    /// worth reading.
    pub(crate) fn tick(&mut self, interval: c_int) -> bool {
        self.count += 1;
        if self.count != interval {
            return false;
        }
        self.count = 0;
        true
    }
}

/// What `submatch()` answers about: the match a `\=` replacement is being
/// expanded against, as [`substitute`] found it before evaluating — taken
/// whole, so that a search or substitution the expression runs cannot
/// change what `submatch()` reports.
#[derive(Default, Debug)]
pub(crate) enum RegSubMatch {
    /// No substitution is evaluating an expression.
    #[default]
    None,
    /// A string match: the text each group matched, `None` for a group that
    /// did not match.
    Str([Option<Vec<u8>>; NSUBEXP as usize]),
    /// A buffer match: where each group starts and ends, the lines read
    /// back from `buf` when asked for.
    Buf {
        buf: Buf,
        firstlnum: LineNr,
        maxline: LineNr,
        startpos: [LPos; NSUBEXP as usize],
        endpos: [LPos; NSUBEXP as usize],
    },
}
/// A backtracking program: what the compiler found out about the pattern,
/// and the node text itself (see [`bt::BtProg`]).
#[derive(Clone, Debug)]
pub(crate) struct BtRegProg {
    /// The `RF_*` flags the compiler found.
    pub(crate) regflags: c_uint,
    /// The character every match starts with, or `NUL`.
    pub(crate) regstart: c_int,
    /// The pattern only matches at the start of a line.
    pub(crate) reganch: uint8_t,
    /// A literal run every matching line holds: the offset of an `EXACTLY`
    /// operand in `program`.
    pub(crate) regmust: Option<usize>,
    /// Its length, which a match writes back (see `BtProg::regmlen`).
    pub(crate) regmlen: core::cell::Cell<c_int>,
    pub(crate) reghasz: uint8_t,
    /// The nodes, `REGMAGIC` stamp first, as the parser wrote them.
    pub(crate) program: Vec<uint8_t>,
}
/// The `\1`..`\9` captures one thread is carrying, and how many of them it
/// has reached. Only the entries below `in_use` mean anything.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RegSub {
    pub in_use: c_int,
    pub list: [Capture; NSUBEXP as usize],
    /// Where a `:substitute` resumes scanning, which travels with group 0.
    pub orig_start_col: ColNr,
}
#[derive(Copy, Clone, Debug)]
pub(crate) struct RegSubs {
    pub norm: RegSub,
    pub synt: RegSub,
}
pub const NFA_TOO_EXPENSIVE: c_int = -1;
pub const NFA_MAX_STATES: c_int = 100000;
pub const AUTOMATIC_ENGINE: c_uint = 0;
#[derive(Debug, Default)]
pub struct ParseState {
    pub regparse: usize,
    pub prevchr_len: usize,
    pub curchr: c_int,
    pub prevchr: c_int,
    pub prevprevchr: c_int,
    pub nextchr: c_int,
    pub at_start: c_int,
    pub prev_at_start: c_int,
    pub regnpar: usize,
}
/// One decision the forward walk made, and what undoing it needs.
#[derive(Debug)]
pub(crate) struct RegItem {
    /// Which decision, and so which of the fields below mean anything.
    pub rs_state: RegState,
    /// The capture slot, the `\{n,m}` counter or the lookaround opcode's
    /// byte the state is about, depending on the state.
    pub rs_no: u8,
    /// The program node the frame was pushed for, as an offset into the
    /// program text.
    pub rs_scan: usize,
    /// What the frame has to put back. The capture states (`RS_MOPEN` and
    /// friends) saved the slot's old value and read [`SavedInput::pos`]
    /// alone; every other state saved the input position and reads both
    /// fields. Upstream made those two a union; they differ by one `int`.
    pub rs_saved: SavedInput,
}
pub type RegState = c_uint;
pub const RS_STAR_SHORT: RegState = 13;
pub const RS_STAR_LONG: RegState = 12;
pub const RS_BEHIND2: RegState = 11;
pub const RS_BEHIND1: RegState = 10;
pub const RS_NOMATCH: RegState = 9;
pub const RS_BRCPLX_SHORT: RegState = 8;
pub const RS_BRCPLX_LONG: RegState = 7;
pub const RS_BRCPLX_MORE: RegState = 6;
pub const RS_BRANCH: RegState = 5;
pub const RS_ZCLOSE: RegState = 4;
pub const RS_ZOPEN: RegState = 3;
pub const RS_MCLOSE: RegState = 2;
pub const RS_MOPEN: RegState = 1;
pub const RS_NOPEN: RegState = 0;
#[derive(Debug)]
pub struct RegStar {
    pub nextb: c_int,
    pub nextb_ic: c_int,
    pub count: int64_t,
    pub minval: int64_t,
    pub maxval: int64_t,
}
#[derive(Debug)]
pub(crate) struct RegBehind {
    pub save_after: SavedInput,
    pub save_behind: SavedInput,
    pub save_need_clear_subexpr: c_int,
    pub save_start: [MatchPos; 10],
    pub save_end: [MatchPos; 10],
}
pub const BACKTRACKING_ENGINE: c_uint = 1;
pub const NFA_ENGINE: c_uint = 2;
pub const INT32_MAX: c_int = 2147483647;
pub const REGMAGIC: c_int = 0o234;
pub const MAX_LIMIT: c_int = 32767 << 16;
const E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN: &CStr =
    c"E363: Pattern uses more memory than 'maxmempattern'";
const E_SUBSTITUTE_NESTING_TOO_DEEP: &CStr = c"E1290: substitute nesting too deep";
pub const NOT_MULTI: c_int = 0;
pub const MULTI_ONE: c_int = 1;
pub const MULTI_MULT: c_int = 2;
pub const RA_FAIL: c_int = 1;
pub const RA_CONT: c_int = 2;
pub const RA_BREAK: c_int = 3;
pub const RA_MATCH: c_int = 4;
pub const RA_NOMATCH: c_int = 5;

crate::flag_set! {
    /// Which Latin-1 byte classes a byte belongs to -- upstream's `RI_*`,
    /// the bits [`chars::RI_FLAGS`](chars::RI_FLAGS) holds per byte. Both
    /// engines answer `\d`, `\w` and their kin by indexing that table and
    /// testing one of these.
    ///
    /// The word is the `i16` the table stores, which is what keeps a
    /// 256-entry table 512 bytes rather than a kilobyte.
    pub struct ByteClass: i16;

    const DIGIT = 0x1;
    const HEX = 0x2;
    const OCTAL = 0x4;
    const WORD = 0x8;
    /// A byte a keyword may *start* with: `\h`, which is `\w` without the
    /// digits.
    const HEAD = 0x10;
    const ALPHA = 0x20;
    const LOWER = 0x40;
    const UPPER = 0x80;
    /// Space and tab only. Set by hand after the table is built, because
    /// neither is in any of the ranges above.
    const WHITE = 0x100;
}
/// The last `:substitute` replacement, after its own `~`s were expanded:
/// what the next `~` (and a `\~` in a pattern) stands for. `None` until a
/// substitution has run, and after one with an empty replacement.
static reg_prev_sub: GlobalCell<Option<XString>> = GlobalCell::new(None);
const REGEXP_INRANGE: &CStr = c"]^-n\\";
const REGEXP_ABBR: &CStr = c"nrtebdoxuU";
pub const RF_ICASE: c_int = 1;
pub const RF_NOICASE: c_int = 2;
pub const RF_HASNL: c_int = 4;
pub const RF_ICOMBINE: c_int = 8;
pub const RF_LOOKBH: c_int = 16;
/// The pattern ended in a `\n` (see [`vim_regcomp_had_eol`]).
pub(crate) const RF_HAD_EOL: c_uint = 32;
pub const REG_NOPAREN: c_int = 0;
pub const REG_PAREN: c_int = 1;
pub const REG_ZPAREN: c_int = 2;
pub const REG_NPAREN: c_int = 3;
static can_f_submatch: GlobalCell<bool> = GlobalCell::new(false);
static rsm: GlobalCell<RegSubMatch> = GlobalCell::new(RegSubMatch::None);
pub const HASWIDTH: c_int = 0x1 as c_int;
pub const SIMPLE: c_int = 0x2 as c_int;
pub const SPSTART: c_int = 0x4 as c_int;
pub const HASNL: c_int = 0x8 as c_int;
pub const HASLOOKBH: c_int = 0x10 as c_int;
pub const WORST: c_int = 0;
pub const REGSTACK_INITIAL: c_int = 2048;
pub const BACKPOS_INITIAL: c_int = 64;
/// How far a postponed lookaround has got -- upstream's `NFA_PIM_*`, which
/// share the `NFA_` prefix with the opcodes and are a different family.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PimResult {
    /// The thread carries no postponed lookaround.
    Unused,
    /// One is postponed and has not been run.
    Todo,
    /// It ran and matched.
    Match,
    /// It ran and did not match.
    NoMatch,
}
pub const ADDSTATE_HERE_OFFSET: c_int = 10;
pub const GRAPHEME_STATE_INIT: c_int = 0;
pub const INT_MAX: c_int = __INT_MAX__;
pub const RE_MAGIC: ::core::ffi::c_int = 1;
pub const RE_STRING: ::core::ffi::c_int = 2;
pub const RE_STRICT: ::core::ffi::c_int = 4;
pub const RE_AUTO: ::core::ffi::c_int = 8;
pub const RE_NOBREAK: ::core::ffi::c_int = 16;
pub const REX_SET: c_int = 1;
pub const REX_USE: c_int = 2;
pub const REX_ALL: c_int = REX_SET | REX_USE;
pub const __INT_MAX__: c_int = 2147483647;

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
#[derive(Debug)]
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
            rm_matchcol: ColNr::ZERO,
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
#[derive(Clone, Default, Debug)]
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
