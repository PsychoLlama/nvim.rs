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
//! in for the C file-scope statics a match still needs (`rex`, the
//! submatch snapshot, the previous replacement). A compile keeps its state
//! in a [`RegCompiler`] instead (see [`compiler`]). It holds no code.
//!
//! The globals are read through `GlobalCell::ptr` on the matching path, not
//! `with`/`with_mut` — see [`context`] for why that distinction is
//! load-bearing.

#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub(crate) mod state;
use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::types::CAR;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use crate::types::{ColNr, LPos, LineNr, Magic, MarkGet, RegEngine, int16_t, int64_t, uint8_t};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_uint};
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
pub(crate) use self::mbyte::*;
pub(crate) use self::nfa::*;
pub(crate) use self::parse::*;
pub(crate) use self::pos::*;
pub(crate) use self::rex::*;
pub(crate) use self::submatch::*;
pub(crate) use self::substitute::*;
pub const _ISalnum: c_uint = 8;
pub const _ISpunct: c_uint = 4;
pub const _IScntrl: c_uint = 2;
pub const _ISgraph: c_uint = 32768;
pub const _ISalpha: c_uint = 1024;
pub const NSUBEXP: c_uint = 10;
pub const MAGIC_ALL: Magic = 4;
pub const MAGIC_ON: Magic = 3;
pub const MAGIC_OFF: Magic = 2;
pub const MAGIC_NONE: Magic = 1;
pub const REGSUB_BACKSLASH: c_uint = 4;
pub const REGSUB_MAGIC: c_uint = 2;
pub const REGSUB_COPY: c_uint = 1;
pub const kMarkBufLocal: MarkGet = 0;
/// The caller's bound on how long an NFA match may run: how many steps
/// since the clock was last read. The deadline itself is the match
/// context's ([`Rex::deadline`]).
#[derive(Copy, Clone)]
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
#[derive(Default)]
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
/// A backtracking program: the [`RegProg`](crate::types::RegProg) head every
/// program opens with, what the compiler found out about the pattern, and
/// the node text itself (see [`bt::BtProg`]). Boxed: `bt_regcomp` hands the
/// box over as a `*mut RegProg` and `vim_regfree` takes it back.
#[repr(C)]
pub(crate) struct BtRegProg {
    pub(crate) engine: *mut RegEngine,
    pub(crate) regflags: c_uint,
    pub(crate) re_engine: c_uint,
    pub(crate) re_flags: c_uint,
    pub(crate) re_in_use: bool,
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
#[repr(C)]
pub struct NfaRegProg {
    pub engine: *mut RegEngine,
    pub regflags: c_uint,
    pub re_engine: c_uint,
    pub re_flags: c_uint,
    pub re_in_use: bool,
    pub start: *mut NfaState,
    pub reganch: c_int,
    pub regstart: c_int,
    pub match_text: *mut uint8_t,
    pub has_zend: c_int,
    pub has_backref: c_int,
    pub reghasz: c_int,
    pub pattern: *mut c_char,
    pub nsubexp: c_int,
    pub nstate: c_int,
    pub state: [NfaState; 0],
}
#[repr(C)]
pub struct NfaState {
    pub c: c_int,
    pub out: *mut NfaState,
    pub out1: *mut NfaState,
    pub id: c_int,
    pub lastlist: [c_int; 2],
    pub val: c_int,
}
/// The `\1`..`\9` captures one thread is carrying, and how many of them it
/// has reached. Only the entries below `in_use` mean anything.
#[derive(Copy, Clone)]
pub(crate) struct RegSub {
    pub in_use: c_int,
    pub list: [Capture; NSUBEXP as usize],
    /// Where a `:substitute` resumes scanning, which travels with group 0.
    pub orig_start_col: ColNr,
}
#[derive(Copy, Clone)]
pub(crate) struct RegSubs {
    pub norm: RegSub,
    pub synt: RegSub,
}
#[derive(Copy, Clone)]
pub(crate) struct NfaThread {
    pub state: *mut NfaState,
    pub count: c_int,
    pub pim: NfaPim,
    pub subs: RegSubs,
}
#[derive(Copy, Clone)]
pub(crate) struct NfaPim {
    pub result: PimResult,
    pub state: *mut NfaState,
    pub subs: RegSubs,
    /// Where the thread stood when the lookaround was postponed, which is
    /// where it has to run from once something settles it.
    pub end: MatchPos,
}
pub const NFA_TOO_EXPENSIVE: c_int = -1;
pub const NFA_MAX_STATES: c_int = 100000;
pub const AUTOMATIC_ENGINE: c_uint = 0;
pub struct ParseState {
    pub regparse: usize,
    pub prevchr_len: c_int,
    pub curchr: c_int,
    pub prevchr: c_int,
    pub prevprevchr: c_int,
    pub nextchr: c_int,
    pub at_start: c_int,
    pub prev_at_start: c_int,
    pub regnpar: c_int,
}
/// One decision the forward walk made, and what undoing it needs.
pub(crate) struct RegItem {
    /// Which decision, and so which of the fields below mean anything.
    pub rs_state: RegState,
    /// The capture slot, the `\{n,m}` counter or the lookaround opcode the
    /// state is about, depending on the state.
    pub rs_no: int16_t,
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
pub struct RegStar {
    pub nextb: c_int,
    pub nextb_ic: c_int,
    pub count: int64_t,
    pub minval: int64_t,
    pub maxval: int64_t,
}
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
const E_RECURSIVE: &CStr = c"E956: Cannot use pattern recursively";
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
static bt_regengine: RegEngine = RegEngine {
    regfree: Some(bt_regfree),
};
static nfa_regengine: RegEngine = RegEngine {
    regfree: Some(nfa_regfree),
};
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
