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

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::types::{
    MarkGet, buf_T, colnr_T, garray_T, int16_t, int64_t, linenr_T, lpos_T, magic_T, proftime_T,
    regengine, regengine_T, regmatch_T, regmmatch_T, size_t, uint8_t, win_T,
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
mod submatch;
mod substitute;

pub use self::api::*;
pub use self::bt::*;
pub use self::chars::*;
pub use self::context::*;
pub(crate) use self::mbyte::*;
pub(crate) use self::nfa::*;
pub use self::parse::*;
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
pub const CLASS_NONE: c_uint = 99;
pub const CLASS_XDIGIT: c_uint = 11;
pub const CLASS_UPPER: c_uint = 10;
pub const CLASS_TAB: c_uint = 12;
pub const CLASS_SPACE: c_uint = 9;
pub const CLASS_RETURN: c_uint = 13;
pub const CLASS_PUNCT: c_uint = 8;
pub const CLASS_PRINT: c_uint = 7;
pub const CLASS_LOWER: c_uint = 6;
pub const CLASS_KEYWORD: c_uint = 17;
pub const CLASS_IDENT: c_uint = 16;
pub const CLASS_GRAPH: c_uint = 5;
pub const CLASS_FNAME: c_uint = 18;
pub const CLASS_ESCAPE: c_uint = 15;
pub const CLASS_DIGIT: c_uint = 4;
pub const CLASS_CNTRL: c_uint = 3;
pub const CLASS_BLANK: c_uint = 2;
pub const CLASS_BACKSPACE: c_uint = 14;
pub const CLASS_ALPHA: c_uint = 1;
pub const CLASS_ALNUM: c_uint = 0;
#[derive(Copy, Clone)]
#[repr(C)]
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
#[repr(C)]
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct linepos {
    pub start: *mut uint8_t,
    pub end: *mut uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union CaptureSlots {
    pub multi: [multipos; 10],
    pub line: [linepos; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multipos {
    pub start_lnum: linenr_T,
    pub end_lnum: linenr_T,
    pub start_col: colnr_T,
    pub end_col: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regsub_T {
    pub in_use: c_int,
    pub list: CaptureSlots,
    pub orig_start_col: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regsubs_T {
    pub norm: regsub_T,
    pub synt: regsub_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_thread_T {
    pub state: *mut nfa_state_T,
    pub count: c_int,
    pub pim: nfa_pim_T,
    pub subs: regsubs_T,
}
pub type nfa_pim_T = nfa_pim_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_pim_S {
    pub result: c_int,
    pub state: *mut nfa_state_T,
    pub subs: regsubs_T,
    pub end: PimEnd,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union PimEnd {
    pub pos: lpos_T,
    pub ptr: *mut uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union SavedPos {
    pub ptr: *mut uint8_t,
    pub pos: lpos_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_se_T {
    pub se_u: SavedPos,
}
pub const NFA_TOO_EXPENSIVE: c_int = -1;
pub const NFA_ZCLOSE9: c_int = -918;
pub const NFA_ZCLOSE: c_int = -927;
pub const NFA_MCLOSE: c_int = -947;
pub const NFA_ZEND: c_int = -1000;
pub const NFA_MCLOSE9: c_int = -938;
pub const NFA_MCLOSE1: c_int = -946;
pub const NFA_ZOPEN9: c_int = -928;
pub const NFA_ZOPEN: c_int = -937;
pub const NFA_MOPEN: c_int = -957;
pub const NFA_ZSTART: c_int = -1001;
pub const NFA_MOPEN9: c_int = -948;
pub const NFA_MOPEN1: c_int = -956;
pub const NFA_NCLOSE: c_int = -998;
pub const NFA_NOPEN: c_int = -999;
pub const NFA_EMPTY: c_int = -1022;
pub const NFA_SPLIT: c_int = -1024;
pub const NFA_MATCH: c_int = -1023;
pub const NFA_SKIP: c_int = -958;
pub const NFA_BOF: c_int = -1004;
pub const NFA_BOL: c_int = -1008;
pub const NFA_START_INVISIBLE_BEFORE_NEG_FIRST: c_int = -990;
pub const NFA_START_INVISIBLE_BEFORE_NEG: c_int = -991;
pub const NFA_START_INVISIBLE_NEG_FIRST: c_int = -994;
pub const NFA_START_INVISIBLE_NEG: c_int = -995;
pub const NFA_START_INVISIBLE_BEFORE_FIRST: c_int = -992;
pub const NFA_START_INVISIBLE_BEFORE: c_int = -993;
pub const NFA_NEWL: c_int = -1002;
pub const NFA_START_NEG_COLL: c_int = -1019;
pub const NFA_START_COLL: c_int = -1021;
pub const NFA_NUPPER_IC: c_int = -887;
pub const NFA_UPPER_IC: c_int = -888;
pub const NFA_NLOWER_IC: c_int = -889;
pub const NFA_LOWER_IC: c_int = -890;
pub const NFA_NUPPER: c_int = -891;
pub const NFA_UPPER: c_int = -892;
pub const NFA_NLOWER: c_int = -893;
pub const NFA_LOWER: c_int = -894;
pub const NFA_NALPHA: c_int = -895;
pub const NFA_ALPHA: c_int = -896;
pub const NFA_NHEAD: c_int = -897;
pub const NFA_HEAD: c_int = -898;
pub const NFA_NWORD: c_int = -899;
pub const NFA_WORD: c_int = -900;
pub const NFA_NOCTAL: c_int = -901;
pub const NFA_OCTAL: c_int = -902;
pub const NFA_NHEX: c_int = -903;
pub const NFA_HEX: c_int = -904;
pub const NFA_NDIGIT: c_int = -905;
pub const NFA_DIGIT: c_int = -906;
pub const NFA_NWHITE: c_int = -907;
pub const NFA_WHITE: c_int = -908;
pub const NFA_SPRINT: c_int = -909;
pub const NFA_PRINT: c_int = -910;
pub const NFA_SFNAME: c_int = -911;
pub const NFA_FNAME: c_int = -912;
pub const NFA_SKWORD: c_int = -913;
pub const NFA_KWORD: c_int = -914;
pub const NFA_SIDENT: c_int = -915;
pub const NFA_IDENT: c_int = -916;
pub const NFA_ANY_COMPOSING: c_int = -983;
pub const NFA_ANY: c_int = -917;
pub const NFA_COMPOSING: c_int = -985;
pub const NFA_START_INVISIBLE_FIRST: c_int = -996;
pub const NFA_START_INVISIBLE: c_int = -997;
pub const NFA_END_PATTERN: c_int = -986;
pub const NFA_END_INVISIBLE_NEG: c_int = -987;
pub const NFA_END_INVISIBLE: c_int = -988;
pub const NFA_VISUAL: c_int = -842;
pub const NFA_CURSOR: c_int = -855;
pub const NFA_MARK_LT: c_int = -843;
pub const NFA_MARK_GT: c_int = -844;
pub const NFA_MARK: c_int = -845;
pub const NFA_VCOL: c_int = -848;
pub const NFA_VCOL_LT: c_int = -846;
pub const NFA_VCOL_GT: c_int = -847;
pub const NFA_COL: c_int = -851;
pub const NFA_COL_LT: c_int = -849;
pub const NFA_COL_GT: c_int = -850;
pub const NFA_LNUM: c_int = -854;
pub const NFA_LNUM_LT: c_int = -852;
pub const NFA_LNUM_GT: c_int = -853;
pub const NFA_ZREF1: c_int = -967;
pub const NFA_BACKREF1: c_int = -976;
pub const NFA_BACKREF9: c_int = -968;
pub const NFA_ZREF9: c_int = -959;
pub const NFA_CLASS_FNAME: c_int = -823;
pub const NFA_CLASS_KEYWORD: c_int = -824;
pub const NFA_CLASS_IDENT: c_int = -825;
pub const NFA_CLASS_ESCAPE: c_int = -826;
pub const NFA_CLASS_BACKSPACE: c_int = -827;
pub const NFA_CLASS_RETURN: c_int = -828;
pub const NFA_CLASS_TAB: c_int = -829;
pub const NFA_CLASS_XDIGIT: c_int = -830;
pub const NFA_CLASS_UPPER: c_int = -831;
pub const NFA_CLASS_SPACE: c_int = -832;
pub const NFA_CLASS_PUNCT: c_int = -833;
pub const NFA_CLASS_PRINT: c_int = -834;
pub const NFA_CLASS_LOWER: c_int = -835;
pub const NFA_CLASS_GRAPH: c_int = -836;
pub const NFA_CLASS_DIGIT: c_int = -837;
pub const NFA_CLASS_CNTRL: c_int = -838;
pub const NFA_CLASS_BLANK: c_int = -839;
pub const NFA_CLASS_ALPHA: c_int = -840;
pub const NFA_CLASS_ALNUM: c_int = -841;
pub const NFA_RANGE_MIN: c_int = -1016;
pub const NFA_END_COLL: c_int = -1020;
pub const NFA_END_COMPOSING: c_int = -984;
pub const NFA_EOF: c_int = -1003;
pub const NFA_EOW: c_int = -1005;
pub const NFA_BOW: c_int = -1006;
pub const NFA_EOL: c_int = -1007;
pub const NFA_START_PATTERN: c_int = -989;
pub const NFA_MAX_STATES: c_int = 100000;
pub const AUTOMATIC_ENGINE: c_uint = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union Ptrlist {
    pub next: *mut Ptrlist,
    pub s: *mut nfa_state_T,
}
pub const NFA_OPT_CHARS: c_int = -982;
pub const NFA_PREV_ATOM_JUST_BEFORE_NEG: c_int = -978;
pub const NFA_PREV_ATOM_JUST_BEFORE: c_int = -979;
pub const NFA_PREV_ATOM_LIKE_PATTERN: c_int = -977;
pub const NFA_PREV_ATOM_NO_WIDTH_NEG: c_int = -980;
pub const NFA_PREV_ATOM_NO_WIDTH: c_int = -981;
pub const NFA_RANGE_MAX: c_int = -1015;
pub const NFA_RANGE: c_int = -1017;
pub const NFA_END_NEG_COLL: c_int = -1018;
pub const NFA_QUEST_NONGREEDY: c_int = -1009;
pub const NFA_QUEST: c_int = -1010;
pub const NFA_STAR_NONGREEDY: c_int = -1011;
pub const NFA_STAR: c_int = -1012;
pub const NFA_OR: c_int = -1013;
pub const NFA_CONCAT: c_int = -1014;
#[derive(Copy, Clone)]
#[repr(C)]
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
pub const NFA_LAST_NL: c_int = -856;
pub const NFA_FIRST_NL: c_int = -886;
pub type regitem_T = regitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regitem_S {
    pub rs_state: regstate_T,
    pub rs_no: int16_t,
    pub rs_scan: *mut uint8_t,
    pub rs_un: FrameSave,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union FrameSave {
    pub sesave: save_se_T,
    pub regsave: regsave_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regsave_T {
    pub rs_u: InputPos,
    pub rs_len: c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union InputPos {
    pub ptr: *mut uint8_t,
    pub pos: lpos_T,
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
#[repr(C)]
pub struct regstar_S {
    pub nextb: c_int,
    pub nextb_ic: c_int,
    pub count: int64_t,
    pub minval: int64_t,
    pub maxval: int64_t,
}
pub type regbehind_T = regbehind_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regbehind_S {
    pub save_after: regsave_T,
    pub save_behind: regsave_T,
    pub save_need_clear_subexpr: c_int,
    pub save_start: [save_se_T; 10],
    pub save_end: [save_se_T; 10],
}
pub type backpos_T = backpos_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct backpos_S {
    pub bp_scan: *mut uint8_t,
    pub bp_pos: regsave_T,
}
pub const BACKTRACKING_ENGINE: c_uint = 1;
pub const NFA_ENGINE: c_uint = 2;
pub const INT32_MAX: c_int = 2147483647;
pub const NUL: c_int = '\0' as c_int;
pub const TAB: c_int = '\t' as c_int;
pub const NL: c_int = '\n' as c_int;
pub const CAR: c_int = '\r' as c_int;
pub const ESC: c_int = '\u{1b}' as c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 1,
    ga_data: core::ptr::null_mut(),
};
pub const REGMAGIC: c_int = 0o234;
pub const MAX_LIMIT: c_int = 32767 << 16;
const E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN: &CStr =
    c"E363: Pattern uses more memory than 'maxmempattern'";
const E_MISSING_DELIMITER_AFTER_SEARCH_PATTERN_STR: &CStr =
    c"E654: Missing delimiter after search pattern: %s";
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
static rex_in_use: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static can_f_submatch: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
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
pub const END: c_int = 0;
pub const BOL: c_int = 1;
pub const EOL: c_int = 2;
pub const BRANCH: c_int = 3;
pub const BACK: c_int = 4;
pub const EXACTLY: c_int = 5;
pub const NOTHING: c_int = 6;
pub const STAR: c_int = 7;
pub const PLUS: c_int = 8;
pub const MATCH: c_int = 9;
pub const NOMATCH: c_int = 10;
pub const BEHIND: c_int = 11;
pub const NOBEHIND: c_int = 12;
pub const SUBPAT: c_int = 13;
pub const BRACE_SIMPLE: c_int = 14;
pub const BOW: c_int = 15;
pub const EOW: c_int = 16;
pub const BRACE_LIMITS: c_int = 17;
pub const NEWL: c_int = 18;
pub const BHPOS: c_int = 19;
pub const ADD_NL: c_int = 30;
pub const FIRST_NL: c_int = ANY + ADD_NL;
pub const ANY: c_int = 20;
pub const ANYOF: c_int = 21;
pub const ANYBUT: c_int = 22;
pub const IDENT: c_int = 23;
pub const SIDENT: c_int = 24;
pub const KWORD: c_int = 25;
pub const SKWORD: c_int = 26;
pub const FNAME: c_int = 27;
pub const SFNAME: c_int = 28;
pub const PRINT: c_int = 29;
pub const SPRINT: c_int = 30;
pub const WHITE: c_int = 31;
pub const NWHITE: c_int = 32;
pub const DIGIT: c_int = 33;
pub const NDIGIT: c_int = 34;
pub const HEX: c_int = 35;
pub const NHEX: c_int = 36;
pub const OCTAL: c_int = 37;
pub const NOCTAL: c_int = 38;
pub const WORD: c_int = 39;
pub const NWORD: c_int = 40;
pub const HEAD: c_int = 41;
pub const NHEAD: c_int = 42;
pub const ALPHA: c_int = 43;
pub const NALPHA: c_int = 44;
pub const LOWER: c_int = 45;
pub const NLOWER: c_int = 46;
pub const UPPER: c_int = 47;
pub const NUPPER: c_int = 48;
pub const LAST_NL: c_int = NUPPER + ADD_NL;
pub const MOPEN: c_int = 80;
pub const MCLOSE: c_int = 90;
pub const BACKREF: c_int = 100;
pub const ZOPEN: c_int = 110;
pub const ZCLOSE: c_int = 120;
pub const ZREF: c_int = 130;
pub const BRACE_COMPLEX: c_int = 140;
pub const NOPEN: c_int = 150;
pub const NCLOSE: c_int = 151;
pub const MULTIBYTECODE: c_int = 200;
pub const RE_BOF: ::core::ffi::c_int = 201;
pub const RE_EOF: ::core::ffi::c_int = 202;
pub const CURSOR: c_int = 203;
pub const RE_LNUM: ::core::ffi::c_int = 204;
pub const RE_COL: ::core::ffi::c_int = 205;
pub const RE_VCOL: ::core::ffi::c_int = 206;
pub const RE_MARK: ::core::ffi::c_int = 207;
pub const RE_VISUAL: ::core::ffi::c_int = 208;
pub const RE_COMPOSING: ::core::ffi::c_int = 209;
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
static brace_min: GlobalCell<[int64_t; 10]> = GlobalCell::new([0; 10]);
static brace_max: GlobalCell<[int64_t; 10]> = GlobalCell::new([0; 10]);
static brace_count: GlobalCell<[c_int; 10]> = GlobalCell::new([0; 10]);
static one_exactly: GlobalCell<c_int> = GlobalCell::new(false_0);
pub const JUST_CALC_SIZE: *mut uint8_t = -1i64 as *mut uint8_t;
static backpos: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
static behind_pos: GlobalCell<regsave_T> = GlobalCell::new(regsave_T {
    rs_u: InputPos {
        ptr: core::ptr::null_mut::<uint8_t>(),
    },
    rs_len: 0,
});
pub const REGSTACK_INITIAL: c_int = 2048;
pub const BACKPOS_INITIAL: c_int = 64;
static bl_minval: GlobalCell<int64_t> = GlobalCell::new(0);
static bl_maxval: GlobalCell<int64_t> = GlobalCell::new(0);
pub const NFA_ADD_NL: c_int = 31;
static nfa_re_flags: GlobalCell<c_int> = GlobalCell::new(0);
static wants_nfa: GlobalCell<bool> = GlobalCell::new(false);
static nstate: GlobalCell<c_int> = GlobalCell::new(0);
static istate: GlobalCell<c_int> = GlobalCell::new(0);
static nfa_endp: GlobalCell<*mut save_se_T> = GlobalCell::new(core::ptr::null_mut::<save_se_T>());
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
pub const NFA_PIM_UNUSED: c_int = 0;
pub const NFA_PIM_TODO: c_int = 1;
pub const NFA_PIM_MATCH: c_int = 2;
pub const NFA_PIM_NOMATCH: c_int = 3;
static nfa_match: GlobalCell<c_int> = GlobalCell::new(0);
static nfa_time_limit: GlobalCell<*mut proftime_T> =
    GlobalCell::new(core::ptr::null_mut::<proftime_T>());
static nfa_timed_out: GlobalCell<*mut c_int> = GlobalCell::new(core::ptr::null_mut::<c_int>());
static nfa_time_count: GlobalCell<c_int> = GlobalCell::new(0);
pub const ADDSTATE_HERE_OFFSET: c_int = 10;
static bt_regengine: GlobalCell<regengine_T> = GlobalCell::new(regengine {
    regcomp: Some(bt_regcomp),
    regfree: Some(bt_regfree),
    regexec_nl: Some(bt_regexec_nl),
    regexec_multi: Some(bt_regexec_multi),
});
static nfa_regengine: GlobalCell<regengine_T> = GlobalCell::new(regengine {
    regcomp: Some(nfa_regcomp),
    regfree: Some(nfa_regfree),
    regexec_nl: Some(nfa_regexec_nl),
    regexec_multi: Some(nfa_regexec_multi),
});
static regexp_engine: GlobalCell<c_int> = GlobalCell::new(0);
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const GRAPHEME_STATE_INIT: c_int = 0;
pub const CPO_LITERAL: c_int = 'l' as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const false_0: c_int = 0;
pub const RE_MAGIC: ::core::ffi::c_int = 1;
pub const RE_STRING: ::core::ffi::c_int = 2;
pub const RE_STRICT: ::core::ffi::c_int = 4;
pub const RE_AUTO: ::core::ffi::c_int = 8;
pub const RE_NOBREAK: ::core::ffi::c_int = 16;
pub const REX_SET: c_int = 1;
pub const REX_USE: c_int = 2;
pub const REX_ALL: c_int = REX_SET | REX_USE;
pub const __INT_MAX__: c_int = 2147483647;
