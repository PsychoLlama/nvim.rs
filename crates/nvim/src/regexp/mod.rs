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
//! in for the C file-scope statics (`rex`, the pattern cursor, the
//! compiler's output cursor). It holds no code.
//!
//! The globals are read through `GlobalCell::ptr` on the matching path, not
//! `with`/`with_mut` — see [`context`] for why that distinction is
//! load-bearing.

#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub(crate) mod state;
use crate::global_cell::GlobalCell;
use crate::types::CAR;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use crate::types::{
    Buffer, ColNr, LPos, LineNr, Magic, MarkGet, ProfTime, RegEngine, RegMMatch, RegMatch, Window,
    int16_t, int64_t, size_t, uint8_t,
};
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
pub use self::bt::*;
pub use self::chars::*;
pub use self::context::*;
pub(crate) use self::mbyte::*;
pub(crate) use self::nfa::*;
pub use self::parse::*;
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
#[derive(Copy, Clone)]
pub struct RegExec {
    pub reg_match: *mut RegMatch,
    pub reg_mmatch: *mut RegMMatch,
    /// A string match's `\1`..`\9` start slots, and the ends.
    ///
    /// Upstream points these at the caller's `RegMatch`, so the engine
    /// writes the reported captures straight into it. Here they are the
    /// context's own, and [`api::vim_regexec`] turns them into the offsets
    /// the caller's match structure carries — which is what lets that
    /// structure hold spans rather than pointers into text it does not own.
    /// Being part of the context also means `with_rex` saves and restores
    /// them, so a `\=` expression that runs a match of its own cannot
    /// overwrite the captures the outer `submatch()` is about to read.
    pub str_start: [*mut uint8_t; NSUBEXP as usize],
    pub str_end: [*mut uint8_t; NSUBEXP as usize],
    pub reg_startpos: *mut LPos,
    pub reg_endpos: *mut LPos,
    pub reg_win: *mut Window,
    pub reg_buf: *mut Buffer,
    pub reg_firstlnum: LineNr,
    pub reg_maxline: LineNr,
    pub reg_line_lbr: bool,
    pub lnum: LineNr,
    pub line: *mut uint8_t,
    pub input: *mut uint8_t,
    pub need_clear_subexpr: c_int,
    pub need_clear_zsubexpr: c_int,
    pub reg_ic: bool,
    pub reg_icombine: bool,
    pub reg_nobreak: bool,
    pub reg_maxcol: ColNr,
    pub nfa_has_zend: c_int,
    pub nfa_has_backref: c_int,
    pub nfa_nsubexpr: c_int,
    pub nfa_listid: c_int,
    pub nfa_alt_listid: c_int,
    pub nfa_has_zsubexpr: c_int,
}
#[derive(Copy, Clone)]
pub struct RegSubMatch {
    pub sm_match: *mut RegMatch,
    pub sm_mmatch: *mut RegMMatch,
    /// The string a string match ran over, which its capture offsets are
    /// relative to. Null for a buffer match, whose captures name lines.
    pub sm_line: *const c_char,
    pub sm_firstlnum: LineNr,
    pub sm_maxline: LineNr,
    pub sm_line_lbr: c_int,
}
#[repr(C)]
pub struct BtRegProg {
    pub engine: *mut RegEngine,
    pub regflags: c_uint,
    pub re_engine: c_uint,
    pub re_flags: c_uint,
    pub re_in_use: bool,
    pub regstart: c_int,
    pub reganch: uint8_t,
    pub regmust: *mut uint8_t,
    pub regmlen: c_int,
    pub reghasz: uint8_t,
    pub program: [uint8_t; 0],
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
    pub regparse: *mut c_char,
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
    /// The program node the frame was pushed for.
    pub rs_scan: *mut uint8_t,
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
static reg_prev_sub: GlobalCell<*mut c_char> = GlobalCell::new(core::ptr::null_mut::<c_char>());
static reg_prev_sublen: GlobalCell<size_t> = GlobalCell::new(0);
const REGEXP_INRANGE: &CStr = c"]^-n\\";
const REGEXP_ABBR: &CStr = c"nrtebdoxuU";
pub const RF_ICASE: c_int = 1;
pub const RF_NOICASE: c_int = 2;
pub const RF_HASNL: c_int = 4;
pub const RF_ICOMBINE: c_int = 8;
pub const RF_LOOKBH: c_int = 16;
static regparse: GlobalCell<*mut c_char> = GlobalCell::new(core::ptr::null_mut::<c_char>());
static regnpar: GlobalCell<c_int> = GlobalCell::new(0);
static regnzpar: GlobalCell<c_int> = GlobalCell::new(0);
static re_has_z: GlobalCell<c_int> = GlobalCell::new(0);
static regflags: GlobalCell<c_uint> = GlobalCell::new(0);
static had_eol: GlobalCell<c_int> = GlobalCell::new(0);
static reg_magic: GlobalCell<Magic> = GlobalCell::new(0);
static reg_string: GlobalCell<c_int> = GlobalCell::new(0);
static reg_strict: GlobalCell<c_int> = GlobalCell::new(0);
static curchr: GlobalCell<c_int> = GlobalCell::new(0);
static prevchr: GlobalCell<c_int> = GlobalCell::new(0);
static prevprevchr: GlobalCell<c_int> = GlobalCell::new(0);
static nextchr: GlobalCell<c_int> = GlobalCell::new(0);
pub const REG_NOPAREN: c_int = 0;
pub const REG_PAREN: c_int = 1;
pub const REG_ZPAREN: c_int = 2;
pub const REG_NPAREN: c_int = 3;
static reg_cpo_lit: GlobalCell<c_int> = GlobalCell::new(0);
static at_start: GlobalCell<c_int> = GlobalCell::new(0);
static prev_at_start: GlobalCell<c_int> = GlobalCell::new(0);
static reg_tofree: GlobalCell<*mut uint8_t> = GlobalCell::new(core::ptr::null_mut::<uint8_t>());
static reg_tofreelen: GlobalCell<c_uint> = GlobalCell::new(0);
static rex: GlobalCell<RegExec> = GlobalCell::new(RegExec {
    reg_match: core::ptr::null_mut::<RegMatch>(),
    reg_mmatch: core::ptr::null_mut::<RegMMatch>(),
    str_start: [core::ptr::null_mut::<uint8_t>(); NSUBEXP as usize],
    str_end: [core::ptr::null_mut::<uint8_t>(); NSUBEXP as usize],
    reg_startpos: core::ptr::null_mut::<LPos>(),
    reg_endpos: core::ptr::null_mut::<LPos>(),
    reg_win: core::ptr::null_mut::<Window>(),
    reg_buf: core::ptr::null_mut::<Buffer>(),
    reg_firstlnum: 0,
    reg_maxline: 0,
    reg_line_lbr: false,
    lnum: 0,
    line: core::ptr::null_mut::<uint8_t>(),
    input: core::ptr::null_mut::<uint8_t>(),
    need_clear_subexpr: 0,
    need_clear_zsubexpr: 0,
    reg_ic: false,
    reg_icombine: false,
    reg_nobreak: false,
    reg_maxcol: 0,
    nfa_has_zend: 0,
    nfa_has_backref: 0,
    nfa_nsubexpr: 0,
    nfa_listid: 0,
    nfa_alt_listid: 0,
    nfa_has_zsubexpr: 0,
});
static rex_in_use: GlobalCell<bool> = GlobalCell::new(false);
static can_f_submatch: GlobalCell<bool> = GlobalCell::new(false);
static rsm: GlobalCell<RegSubMatch> = GlobalCell::new(RegSubMatch {
    sm_match: core::ptr::null_mut::<RegMatch>(),
    sm_mmatch: core::ptr::null_mut::<RegMMatch>(),
    sm_line: core::ptr::null::<c_char>(),
    sm_firstlnum: 0,
    sm_maxline: 0,
    sm_line_lbr: 0,
});
static reg_startzp: GlobalCell<[*mut uint8_t; 10]> =
    GlobalCell::new([core::ptr::null_mut::<uint8_t>(); 10]);
static reg_endzp: GlobalCell<[*mut uint8_t; 10]> =
    GlobalCell::new([core::ptr::null_mut::<uint8_t>(); 10]);
static reg_startzpos: GlobalCell<[LPos; 10]> = GlobalCell::new([LPos { lnum: 0, col: 0 }; 10]);
static reg_endzpos: GlobalCell<[LPos; 10]> = GlobalCell::new([LPos { lnum: 0, col: 0 }; 10]);
pub const HASWIDTH: c_int = 0x1 as c_int;
pub const SIMPLE: c_int = 0x2 as c_int;
pub const SPSTART: c_int = 0x4 as c_int;
pub const HASNL: c_int = 0x8 as c_int;
pub const HASLOOKBH: c_int = 0x10 as c_int;
pub const WORST: c_int = 0;
static prevchr_len: GlobalCell<c_int> = GlobalCell::new(0);
static num_complex_braces: GlobalCell<c_int> = GlobalCell::new(0);
static regcode: GlobalCell<*mut uint8_t> = GlobalCell::new(core::ptr::null_mut::<uint8_t>());
static regsize: GlobalCell<int64_t> = GlobalCell::new(0);
static reg_toolong: GlobalCell<c_int> = GlobalCell::new(0);
static had_endbrace: GlobalCell<[uint8_t; 10]> = GlobalCell::new([0; 10]);
static one_exactly: GlobalCell<c_int> = GlobalCell::new(0);
pub const JUST_CALC_SIZE: *mut uint8_t = -1i64 as *mut uint8_t;
static behind_pos: GlobalCell<SavedInput> = GlobalCell::new(SavedInput::NOWHERE);
pub const REGSTACK_INITIAL: c_int = 2048;
pub const BACKPOS_INITIAL: c_int = 64;
static bl_minval: GlobalCell<int64_t> = GlobalCell::new(0);
static bl_maxval: GlobalCell<int64_t> = GlobalCell::new(0);
static nfa_re_flags: GlobalCell<c_int> = GlobalCell::new(0);
static wants_nfa: GlobalCell<bool> = GlobalCell::new(false);
static nstate: GlobalCell<c_int> = GlobalCell::new(0);
static istate: GlobalCell<c_int> = GlobalCell::new(0);
static nfa_endp: GlobalCell<*mut MatchPos> = GlobalCell::new(core::ptr::null_mut::<MatchPos>());
static nfa_ll_index: GlobalCell<c_int> = GlobalCell::new(0);
static state_ptr: GlobalCell<*mut NfaState> = GlobalCell::new(core::ptr::null_mut::<NfaState>());
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
static nfa_match: GlobalCell<c_int> = GlobalCell::new(0);
static nfa_time_limit: GlobalCell<*mut ProfTime> =
    GlobalCell::new(core::ptr::null_mut::<ProfTime>());
static nfa_timed_out: GlobalCell<*mut c_int> = GlobalCell::new(core::ptr::null_mut::<c_int>());
static nfa_time_count: GlobalCell<c_int> = GlobalCell::new(0);
pub const ADDSTATE_HERE_OFFSET: c_int = 10;
static bt_regengine: RegEngine = RegEngine {
    regcomp: Some(bt_regcomp),
    regfree: Some(bt_regfree),
    regexec_nl: Some(bt_regexec_nl),
    regexec_multi: Some(bt_regexec_multi),
};
static nfa_regengine: RegEngine = RegEngine {
    regcomp: Some(nfa_regcomp),
    regfree: Some(nfa_regfree),
    regexec_nl: Some(nfa_regexec_nl),
    regexec_multi: Some(nfa_regexec_multi),
};
static regexp_engine: GlobalCell<c_int> = GlobalCell::new(0);
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
