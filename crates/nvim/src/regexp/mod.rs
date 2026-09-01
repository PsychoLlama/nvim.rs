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

use crate::global_cell::GlobalCell;
use crate::types::{
    MarkGet, buf_T, colnr_T, int16_t, int64_t, linenr_T, lpos_T, magic_T, proftime_T, regengine,
    regengine_T, regmatch_T, regmmatch_T, size_t, uint8_t, win_T,
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
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub const MAGIC_OFF: magic_T = 2;
pub const MAGIC_NONE: magic_T = 1;
pub const REGSUB_BACKSLASH: c_uint = 4;
pub const REGSUB_MAGIC: c_uint = 2;
pub const REGSUB_COPY: c_uint = 1;
pub const kMarkBufLocal: MarkGet = 0;
#[derive(Copy, Clone)]
pub struct regexec_T {
    pub reg_match: *mut regmatch_T,
    pub reg_mmatch: *mut regmmatch_T,
    pub reg_startp: *mut *mut uint8_t,
    pub reg_endp: *mut *mut uint8_t,
    pub reg_startpos: *mut lpos_T,
    pub reg_endpos: *mut lpos_T,
    pub reg_win: *mut win_T,
    pub reg_buf: *mut buf_T,
    pub reg_firstlnum: linenr_T,
    pub reg_maxline: linenr_T,
    pub reg_line_lbr: bool,
    pub lnum: linenr_T,
    pub line: *mut uint8_t,
    pub input: *mut uint8_t,
    pub need_clear_subexpr: c_int,
    pub need_clear_zsubexpr: c_int,
    pub reg_ic: bool,
    pub reg_icombine: bool,
    pub reg_nobreak: bool,
    pub reg_maxcol: colnr_T,
    pub nfa_has_zend: c_int,
    pub nfa_has_backref: c_int,
    pub nfa_nsubexpr: c_int,
    pub nfa_listid: c_int,
    pub nfa_alt_listid: c_int,
    pub nfa_has_zsubexpr: c_int,
}
#[derive(Copy, Clone)]
pub struct regsubmatch_T {
    pub sm_match: *mut regmatch_T,
    pub sm_mmatch: *mut regmmatch_T,
    pub sm_firstlnum: linenr_T,
    pub sm_maxline: linenr_T,
    pub sm_line_lbr: c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bt_regprog_T {
    pub engine: *mut regengine_T,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_regprog_T {
    pub engine: *mut regengine_T,
    pub regflags: c_uint,
    pub re_engine: c_uint,
    pub re_flags: c_uint,
    pub re_in_use: bool,
    pub start: *mut nfa_state_T,
    pub reganch: c_int,
    pub regstart: c_int,
    pub match_text: *mut uint8_t,
    pub has_zend: c_int,
    pub has_backref: c_int,
    pub reghasz: c_int,
    pub pattern: *mut c_char,
    pub nsubexp: c_int,
    pub nstate: c_int,
    pub state: [nfa_state_T; 0],
}
pub type nfa_state_T = nfa_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_state {
    pub c: c_int,
    pub out: *mut nfa_state_T,
    pub out1: *mut nfa_state_T,
    pub id: c_int,
    pub lastlist: [c_int; 2],
    pub val: c_int,
}
/// The `\1`..`\9` captures one thread is carrying, and how many of them it
/// has reached. Only the entries below `in_use` mean anything.
#[derive(Copy, Clone)]
pub(crate) struct regsub_T {
    pub in_use: c_int,
    pub list: [Capture; NSUBEXP as usize],
    /// Where a `:substitute` resumes scanning, which travels with group 0.
    pub orig_start_col: colnr_T,
}
#[derive(Copy, Clone)]
pub(crate) struct regsubs_T {
    pub norm: regsub_T,
    pub synt: regsub_T,
}
#[derive(Copy, Clone)]
pub(crate) struct nfa_thread_T {
    pub state: *mut nfa_state_T,
    pub count: c_int,
    pub pim: nfa_pim_T,
    pub subs: regsubs_T,
}
pub(crate) type nfa_pim_T = nfa_pim_S;
#[derive(Copy, Clone)]
pub(crate) struct nfa_pim_S {
    pub result: PimResult,
    pub state: *mut nfa_state_T,
    pub subs: regsubs_T,
    /// Where the thread stood when the lookaround was postponed, which is
    /// where it has to run from once something settles it.
    pub end: MatchPos,
}
pub const NFA_TOO_EXPENSIVE: c_int = -1;
pub const NFA_MAX_STATES: c_int = 100000;
pub const AUTOMATIC_ENGINE: c_uint = 0;
#[derive(Copy, Clone)]
pub struct parse_state_T {
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
pub(crate) type regitem_T = regitem_S;
/// One decision the forward walk made, and what undoing it needs.
#[derive(Copy, Clone)]
pub(crate) struct regitem_S {
    /// Which decision, and so which of the fields below mean anything.
    pub rs_state: regstate_T,
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
pub type regstate_T = regstate_E;
pub type regstate_E = c_uint;
pub const RS_STAR_SHORT: regstate_E = 13;
pub const RS_STAR_LONG: regstate_E = 12;
pub const RS_BEHIND2: regstate_E = 11;
pub const RS_BEHIND1: regstate_E = 10;
pub const RS_NOMATCH: regstate_E = 9;
pub const RS_BRCPLX_SHORT: regstate_E = 8;
pub const RS_BRCPLX_LONG: regstate_E = 7;
pub const RS_BRCPLX_MORE: regstate_E = 6;
pub const RS_BRANCH: regstate_E = 5;
pub const RS_ZCLOSE: regstate_E = 4;
pub const RS_ZOPEN: regstate_E = 3;
pub const RS_MCLOSE: regstate_E = 2;
pub const RS_MOPEN: regstate_E = 1;
pub const RS_NOPEN: regstate_E = 0;
pub type regstar_T = regstar_S;
#[derive(Copy, Clone)]
pub struct regstar_S {
    pub nextb: c_int,
    pub nextb_ic: c_int,
    pub count: int64_t,
    pub minval: int64_t,
    pub maxval: int64_t,
}
pub(crate) type regbehind_T = regbehind_S;
#[derive(Copy, Clone)]
pub(crate) struct regbehind_S {
    pub save_after: SavedInput,
    pub save_behind: SavedInput,
    pub save_need_clear_subexpr: c_int,
    pub save_start: [MatchPos; 10],
    pub save_end: [MatchPos; 10],
}
pub const BACKTRACKING_ENGINE: c_uint = 1;
pub const NFA_ENGINE: c_uint = 2;
pub const INT32_MAX: c_int = 2147483647;
pub const TAB: c_int = '\t' as c_int;
pub const NL: c_int = '\n' as c_int;
pub const CAR: c_int = '\r' as c_int;
pub const ESC: c_int = '\u{1b}' as c_int;
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
static reg_prev_sub: GlobalCell<*mut c_char> = GlobalCell::new(core::ptr::null_mut::<c_char>());
static reg_prev_sublen: GlobalCell<size_t> = GlobalCell::new(0);
const REGEXP_INRANGE: &CStr = c"]^-n\\";
const REGEXP_ABBR: &CStr = c"nrtebdoxuU";
pub const RI_DIGIT: c_int = 0x1 as c_int;
pub const RI_HEX: c_int = 0x2 as c_int;
pub const RI_OCTAL: c_int = 0x4 as c_int;
pub const RI_WORD: c_int = 0x8 as c_int;
pub const RI_HEAD: c_int = 0x10 as c_int;
pub const RI_ALPHA: c_int = 0x20 as c_int;
pub const RI_LOWER: c_int = 0x40 as c_int;
pub const RI_UPPER: c_int = 0x80 as c_int;
pub const RI_WHITE: c_int = 0x100 as c_int;
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
static reg_magic: GlobalCell<magic_T> = GlobalCell::new(0);
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
static rex: GlobalCell<regexec_T> = GlobalCell::new(regexec_T {
    reg_match: core::ptr::null_mut::<regmatch_T>(),
    reg_mmatch: core::ptr::null_mut::<regmmatch_T>(),
    reg_startp: core::ptr::null_mut::<*mut uint8_t>(),
    reg_endp: core::ptr::null_mut::<*mut uint8_t>(),
    reg_startpos: core::ptr::null_mut::<lpos_T>(),
    reg_endpos: core::ptr::null_mut::<lpos_T>(),
    reg_win: core::ptr::null_mut::<win_T>(),
    reg_buf: core::ptr::null_mut::<buf_T>(),
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
static rsm: GlobalCell<regsubmatch_T> = GlobalCell::new(regsubmatch_T {
    sm_match: core::ptr::null_mut::<regmatch_T>(),
    sm_mmatch: core::ptr::null_mut::<regmmatch_T>(),
    sm_firstlnum: 0,
    sm_maxline: 0,
    sm_line_lbr: 0,
});
static reg_startzp: GlobalCell<[*mut uint8_t; 10]> =
    GlobalCell::new([core::ptr::null_mut::<uint8_t>(); 10]);
static reg_endzp: GlobalCell<[*mut uint8_t; 10]> =
    GlobalCell::new([core::ptr::null_mut::<uint8_t>(); 10]);
static reg_startzpos: GlobalCell<[lpos_T; 10]> = GlobalCell::new([lpos_T { lnum: 0, col: 0 }; 10]);
static reg_endzpos: GlobalCell<[lpos_T; 10]> = GlobalCell::new([lpos_T { lnum: 0, col: 0 }; 10]);
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
pub const CLASS_not: c_int = 0x80 as c_int;
pub const CLASS_af: c_int = 0x40 as c_int;
pub const CLASS_AF: c_int = 0x20 as c_int;
pub const CLASS_az: c_int = 0x10 as c_int;
pub const CLASS_AZ: c_int = 0x8 as c_int;
pub const CLASS_o7: c_int = 0x4 as c_int;
pub const CLASS_o9: c_int = 0x2 as c_int;
pub const CLASS_underscore: c_int = 0x1 as c_int;
static state_ptr: GlobalCell<*mut nfa_state_T> =
    GlobalCell::new(core::ptr::null_mut::<nfa_state_T>());
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
static nfa_time_limit: GlobalCell<*mut proftime_T> =
    GlobalCell::new(core::ptr::null_mut::<proftime_T>());
static nfa_timed_out: GlobalCell<*mut c_int> = GlobalCell::new(core::ptr::null_mut::<c_int>());
static nfa_time_count: GlobalCell<c_int> = GlobalCell::new(0);
pub const ADDSTATE_HERE_OFFSET: c_int = 10;
static bt_regengine: regengine_T = regengine {
    regcomp: Some(bt_regcomp),
    regfree: Some(bt_regfree),
    regexec_nl: Some(bt_regexec_nl),
    regexec_multi: Some(bt_regexec_multi),
};
static nfa_regengine: regengine_T = regengine {
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
