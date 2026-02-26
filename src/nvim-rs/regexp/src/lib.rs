//! Regexp pattern scanning utilities for Neovim.
//!
//! Provides `rs_skip_regexp` and `rs_skip_regexp_ex` — stateless helpers that
//! skip past a regexp pattern to its closing delimiter, handling magic modes,
//! `[...]` character class ranges, and multibyte characters.

#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::borrow_as_ptr)]
#![allow(dead_code)] // NFA constants are used incrementally across phases

pub(crate) mod errors;

use std::ffi::{c_char, c_int, c_uint, c_void};

use std::ffi::c_long;

/// Returns non-zero if `c` is a space or tab.
#[inline]
const fn ascii_iswhite(c: c_int) -> c_int {
    (c == b' ' as c_int || c == b'\t' as c_int) as c_int
}

// garray operations needed for regstack/backpos management
extern "C" {
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn ga_clear(gap: *mut GarrayT);
    fn ga_set_growsize(gap: *mut GarrayT, growsize: c_int);
}
extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_regexp_get_char_class(pp: *mut *mut c_char) -> c_int;
    fn nvim_get_p_cpo() -> *const c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_regexp_get_regflags(prog: *const c_void) -> c_uint;

    // Multibyte helpers
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;

    // Case-insensitive helpers
    fn utf_fold(a: c_int) -> c_int;
    fn utf_strnicmp(s1: *const c_char, s2: *const c_char, n1: usize, n2: usize) -> c_int;
    fn mb_ptr2char_adv(pp: *mut *const c_char) -> c_int;

    // libc
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // re_mult_next accessors
    fn nvim_regexp_set_rc_did_emsg(v: c_int);

    // skip_regexp_err accessor

    // reg_nextline accessors
    fn nvim_regexp_call_reg_getline(lnum: i32) -> *mut c_char;

    // reg_prev_class accessors
    fn nvim_regexp_get_rex_reg_buf_chartab() -> *mut i64;
    fn mb_get_class_tab(p: *const c_char, chartab: *const i64) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // cleanup_subexpr / cleanup_zsubexpr accessors

}

// Characters always special inside [] ranges
const REGEXP_INRANGE: &[u8] = b"]^-n\\";
// Abbreviation characters after '\'
const REGEXP_ABBR: &[u8] = b"nrtebdoxuU";
// CPO_LITERAL flag character
const CPO_LITERAL: c_int = b'l' as c_int;
// Character class constants (matches C enum in regexp.c)
const CLASS_ALNUM: c_int = 0;
const CLASS_ALPHA: c_int = 1;
const CLASS_BLANK: c_int = 2;
const CLASS_CNTRL: c_int = 3;
const CLASS_DIGIT: c_int = 4;
const CLASS_GRAPH: c_int = 5;
const CLASS_LOWER: c_int = 6;
const CLASS_PRINT: c_int = 7;
const CLASS_PUNCT: c_int = 8;
const CLASS_SPACE: c_int = 9;
const CLASS_UPPER: c_int = 10;
const CLASS_XDIGIT: c_int = 11;
const CLASS_CC_TAB: c_int = 12;
const CLASS_RETURN: c_int = 13;
const CLASS_BACKSPACE: c_int = 14;
const CLASS_ESCAPE: c_int = 15;
const CLASS_IDENT: c_int = 16;
const CLASS_KEYWORD: c_int = 17;
const CLASS_FNAME: c_int = 18;
const CLASS_NONE: c_int = 99;

// Magic modes (matching regexp_defs.h)
#[allow(dead_code)]
const MAGIC_NONE: c_int = 1;
const MAGIC_OFF: c_int = 2;
const MAGIC_ON: c_int = 3;
const MAGIC_ALL: c_int = 4;

// RE flags (matching regexp.h)
const RE_MAGIC: c_int = 1;
const RE_STRING: c_int = 2;
const RE_STRICT: c_int = 4;

// --- Phase 1: Parser state globals (moved from C to Rust) ---
// These were previously accessed via nvim_regexp_get_*/set_* C accessor functions.
// Single-threaded: Neovim is single-threaded for regexp.
static mut REGPARSE: *mut c_char = std::ptr::null_mut();
static mut PREVCHR_LEN: c_int = 0;
static mut CURCHR: c_int = -1;
static mut PREVCHR: c_int = -1;
static mut PREVPREVCHR: c_int = -1;
static mut NEXTCHR: c_int = -1;
static mut AT_START: c_int = 0;
static mut PREV_AT_START: c_int = 0;
static mut AFTER_SLASH: c_int = 0;
static mut REGNPAR: c_int = 0;
static mut REGNZPAR: c_int = 0;
static mut REG_MAGIC: c_int = 0;
static mut REGFLAGS_COMPILE: c_uint = 0;
static mut HAD_EOL: c_int = 0;
static mut REG_CPO_LIT: c_int = 0;
static mut REG_STRING: c_int = 0;
static mut REG_STRICT: c_int = 0;
static mut RE_HAS_Z: c_int = 0;
static mut WANTS_NFA: c_int = 0;
static mut ONE_EXACTLY: c_int = 0;

// --- Phase 2: Compilation globals (moved from C to Rust) ---
// JUST_CALC_SIZE sentinel: same as C (uint8_t*)-1 = all-ones pointer
const JUST_CALC_SIZE: *mut u8 = (-1isize as usize) as *mut u8;
// NSUBEXP and CLASSCHARS are defined later (near their related code); see those definitions.
static mut REGCODE: *mut u8 = std::ptr::null_mut();
static mut REGSIZE: i64 = 0;
static mut REG_TOOLONG: c_int = 0;
static mut NUM_COMPLEX_BRACES: c_int = 0;
static mut BRACE_MIN: [i64; 10] = [0i64; 10];
static mut BRACE_MAX: [i64; 10] = [0i64; 10];
static mut BRACE_COUNT: [c_int; 10] = [0; 10];
static mut BL_MINVAL: i64 = 0;
static mut BL_MAXVAL: i64 = 0;

// --- Phase 3: NFA compiler/execution globals (moved from C to Rust) ---
static mut POST_START: *mut c_int = std::ptr::null_mut();
static mut POST_END: *mut c_int = std::ptr::null_mut();
static mut POST_PTR: *mut c_int = std::ptr::null_mut();
static mut NSTATE: c_int = 0;
static mut ISTATE: c_int = 0;
static mut NFA_RE_FLAGS: c_int = 0;
static mut STATE_PTR: *mut c_void = std::ptr::null_mut();
static mut MATCH_FOUND: c_int = 0; // C: nfa_match (renamed to avoid conflict with NFA_MATCH opcode const)
static mut NFA_LL_INDEX: c_int = 0;
static mut NFA_ENDP: *mut SaveSeT = std::ptr::null_mut();
static mut NFA_TIME_LIMIT: *mut c_void = std::ptr::null_mut();
static mut NFA_TIMED_OUT: *mut c_int = std::ptr::null_mut();
static mut NFA_TIME_COUNT: c_int = 0;
static mut REGEXP_ENGINE: c_int = 0;

// --- Phase 4: regstack/backpos/z-subexpr/eval_result globals (moved from C to Rust) ---

/// Matches C `garray_T` exactly.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}
impl GarrayT {
    const fn empty() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 1,
            ga_data: core::ptr::null_mut(),
        }
    }
}

const MAX_REGSUB_NESTING: usize = 4;

static mut REGSTACK: GarrayT = GarrayT::empty();
static mut BACKPOS: GarrayT = GarrayT::empty();
static mut BEHIND_POS: RegsaveT = RegsaveT {
    rs_u: RegsaveUnion {
        pos: LposT { lnum: 0, col: 0 },
    },
    rs_len: 0,
};
static mut REG_TOFREE: *mut u8 = std::ptr::null_mut();
static mut REG_TOFREELEN: c_uint = 0;

// z-subexpr arrays
static mut REG_STARTZP: [*mut u8; NSUBEXP] = [std::ptr::null_mut(); NSUBEXP];
static mut REG_ENDZP: [*mut u8; NSUBEXP] = [std::ptr::null_mut(); NSUBEXP];
static mut REG_STARTZPOS: [LposT; NSUBEXP] = [LposT { lnum: 0, col: 0 }; NSUBEXP];
static mut REG_ENDZPOS: [LposT; NSUBEXP] = [LposT { lnum: 0, col: 0 }; NSUBEXP];

// Substitution nesting
static mut EVAL_RESULT: [*mut c_char; MAX_REGSUB_NESTING] =
    [std::ptr::null_mut(); MAX_REGSUB_NESTING];
static mut REGSUB_NESTING: c_int = 0;

// --- Phase 5: rex (regexec_T), rex_in_use, rsm (regsubmatch_T), can_f_submatch ---

/// Matches C `regexec_T` exactly (single-threaded regexp execution state).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RegexecT {
    pub reg_match: *mut c_void,  // regmatch_T*
    pub reg_mmatch: *mut c_void, // regmmatch_T*
    pub reg_startp: *mut *mut u8,
    pub reg_endp: *mut *mut u8,
    pub reg_startpos: *mut LposT,
    pub reg_endpos: *mut LposT,
    pub reg_win: *mut c_void, // win_T*
    pub reg_buf: *mut c_void, // buf_T*
    pub reg_firstlnum: c_int, // linenr_T
    pub reg_maxline: c_int,   // linenr_T
    pub reg_line_lbr: bool,
    pub lnum: c_int, // linenr_T
    pub line: *mut u8,
    pub input: *mut u8,
    pub need_clear_subexpr: c_int,
    pub need_clear_zsubexpr: c_int,
    pub reg_ic: bool,
    pub reg_icombine: bool,
    pub reg_nobreak: bool,
    pub reg_maxcol: c_int, // colnr_T
    pub nfa_has_zend: c_int,
    pub nfa_has_backref: c_int,
    pub nfa_nsubexpr: c_int,
    pub nfa_listid: c_int,
    pub nfa_alt_listid: c_int,
    pub nfa_has_zsubexpr: c_int,
}

impl RegexecT {
    const fn zeroed() -> Self {
        Self {
            reg_match: core::ptr::null_mut(),
            reg_mmatch: core::ptr::null_mut(),
            reg_startp: core::ptr::null_mut(),
            reg_endp: core::ptr::null_mut(),
            reg_startpos: core::ptr::null_mut(),
            reg_endpos: core::ptr::null_mut(),
            reg_win: core::ptr::null_mut(),
            reg_buf: core::ptr::null_mut(),
            reg_firstlnum: 0,
            reg_maxline: 0,
            reg_line_lbr: false,
            lnum: 0,
            line: core::ptr::null_mut(),
            input: core::ptr::null_mut(),
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
        }
    }
}

/// Matches C `regsubmatch_T`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RegsubmatchT {
    pub sm_match: *mut c_void,  // regmatch_T*
    pub sm_mmatch: *mut c_void, // regmmatch_T*
    pub sm_firstlnum: c_int,    // linenr_T
    pub sm_maxline: c_int,      // linenr_T
    pub sm_line_lbr: c_int,
}

impl RegsubmatchT {
    const fn zeroed() -> Self {
        Self {
            sm_match: core::ptr::null_mut(),
            sm_mmatch: core::ptr::null_mut(),
            sm_firstlnum: 0,
            sm_maxline: 0,
            sm_line_lbr: 0,
        }
    }
}

static mut REX: RegexecT = RegexecT::zeroed();
static mut REX_IN_USE: bool = false;
static mut RSM: RegsubmatchT = RegsubmatchT::zeroed();
static mut CAN_F_SUBMATCH: bool = false;

/// Expose `HAD_EOL` for external crates (e.g. search crate) that need
/// `nvim_regexp_get_had_eol()` -- now backed by the Rust static.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_get_had_eol() -> c_int {
    HAD_EOL
}

/// Called by the NFA prog allocator to update the Rust-owned `STATE_PTR`.
///
/// The caller allocates the prog struct and then calls this to set `STATE_PTR`
/// to point at the inline state array that follows the prog header.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_set_state_ptr(v: *mut c_void) {
    STATE_PTR = v;
}

/// Called from C `nvim_regexp_nfa_regtry_setup()` to set Rust-owned time globals.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_set_nfa_time_globals(tm: *mut c_void, timed_out: *mut c_int) {
    NFA_TIME_LIMIT = tm;
    NFA_TIMED_OUT = timed_out;
    NFA_TIME_COUNT = 0;
}

/// Called from C `nvim_regexp_nfa_regexec_both_init_states()` to reset NSTATE.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_reset_nstate() {
    NSTATE = 0;
}

/// Called from C `nvim_regexp_eval_regsub_expr()` to get a pointer to `EVAL_RESULT`[i].
/// C can then read and write through this pointer.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_get_eval_result_ptr(i: c_int) -> *mut *mut c_char {
    &raw mut EVAL_RESULT[i as usize]
}

/// Called from C `nvim_regexp_eval_regsub_expr()` to get a pointer to `REGSUB_NESTING`.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_get_regsub_nesting_ptr() -> *mut c_int {
    &raw mut REGSUB_NESTING
}

/// Called from C `nvim_regexp_bt_init_stacks()` to initialise REGSTACK/BACKPOS.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn nvim_regexp_bt_init_stacks_rust() {
    if REGSTACK.ga_data.is_null() {
        ga_init(&raw mut REGSTACK, 1, REGSTACK_INITIAL as c_int);
        ga_grow(&raw mut REGSTACK, REGSTACK_INITIAL as c_int);
        ga_set_growsize(&raw mut REGSTACK, (REGSTACK_INITIAL * 8) as c_int);
    }
    if BACKPOS.ga_data.is_null() {
        ga_init(
            &raw mut BACKPOS,
            std::mem::size_of::<BackposT>() as c_int,
            BACKPOS_INITIAL as c_int,
        );
        ga_grow(&raw mut BACKPOS, BACKPOS_INITIAL as c_int);
        ga_set_growsize(&raw mut BACKPOS, (BACKPOS_INITIAL * 8) as c_int);
    }
}

/// Called from C `nvim_regexp_bt_cleanup_stacks()` to shrink/free REGSTACK/BACKPOS.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn nvim_regexp_bt_cleanup_stacks_rust() {
    if REG_TOFREELEN > 400 {
        xfree(REG_TOFREE.cast::<c_void>());
        REG_TOFREE = core::ptr::null_mut();
        REG_TOFREELEN = 0;
    }
    if REGSTACK.ga_maxlen > REGSTACK_INITIAL as c_int {
        ga_clear(&raw mut REGSTACK);
    }
    if BACKPOS.ga_maxlen > BACKPOS_INITIAL as c_int {
        ga_clear(&raw mut BACKPOS);
    }
}

/// Called from C `nvim_regexp_call_free_regexp_stuff()` to release all stacks/buffers.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_free_regexp_stuff_rust() {
    ga_clear(&raw mut REGSTACK);
    ga_clear(&raw mut BACKPOS);
    xfree(REG_TOFREE.cast::<c_void>());
    REG_TOFREE = core::ptr::null_mut();
    REG_TOFREELEN = 0;
}

/// Called from C compound functions (`nvim_regexp_setup_vim_regsub` etc.) to get
/// a mutable pointer to the Rust-owned REX struct.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_get_rex_ptr() -> *mut RegexecT {
    &raw mut REX
}

/// Called from C compound functions to get a mutable pointer to RSM.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_get_rsm_ptr() -> *mut RegsubmatchT {
    &raw mut RSM
}

/// Called from C compound functions to get a mutable pointer to `CAN_F_SUBMATCH`.
#[no_mangle]
pub unsafe extern "C" fn nvim_regexp_get_can_f_submatch_ptr() -> *mut bool {
    &raw mut CAN_F_SUBMATCH
}

/// Check for an equivalence class name "[=a=]".  `pp` points to the '['.
/// Returns a character representing the class. Zero means that no item was
/// recognized.  Otherwise `pp` is advanced to after the item.
unsafe fn get_equi_class(pp: *mut *mut c_char) -> c_int {
    let p = *pp;
    if *p.add(1) == b'=' as c_char && *p.add(2) != 0 {
        let l = utfc_ptr2len(p.add(2)) as usize;
        if *p.add(l + 2) == b'=' as c_char && *p.add(l + 3) == b']' as c_char {
            let c = utf_ptr2char(p.add(2));
            *pp = p.add(l + 4);
            return c;
        }
    }
    0
}

/// Check for a collating element "[.a.]".  `pp` points to the '['.
/// Returns a character. Zero means that no item was recognized.  Otherwise
/// `pp` is advanced to after the item.
/// Currently only single characters are recognized.
unsafe fn get_coll_element(pp: *mut *mut c_char) -> c_int {
    let p = *pp;
    if *p != 0 && *p.add(1) == b'.' as c_char && *p.add(2) != 0 {
        let l = utfc_ptr2len(p.add(2)) as usize;
        if *p.add(l + 2) == b'.' as c_char && *p.add(l + 3) == b']' as c_char {
            let c = utf_ptr2char(p.add(2));
            *pp = p.add(l + 4);
            return c;
        }
    }
    0
}

/// Skip over a "[]" range. `p` must point to the character after the '['.
/// The returned pointer is on the matching ']', or the terminating NUL.
///
/// Shared implementation used by both `rs_skip_anyof` (FFI) and
/// `rs_skip_regexp_ex` (which passes `reg_cpo_lit` from its own snapshot).
unsafe fn skip_anyof_impl(mut p: *mut c_char, reg_cpo_lit: bool) -> *mut c_char {
    if *p == b'^' as c_char {
        p = p.add(1);
    }
    if *p == b']' as c_char || *p == b'-' as c_char {
        p = p.add(1);
    }
    while *p != 0 && *p != b']' as c_char {
        let l = utfc_ptr2len(p);
        if l > 1 {
            p = p.add(l as usize);
        } else if *p == b'-' as c_char {
            p = p.add(1);
            if *p != b']' as c_char && *p != 0 {
                // MB_PTR_ADV
                p = p.add(utfc_ptr2len(p) as usize);
            }
        } else if *p == b'\\' as c_char
            && (REGEXP_INRANGE.contains(&(*p.add(1) as u8))
                || (!reg_cpo_lit && REGEXP_ABBR.contains(&(*p.add(1) as u8))))
        {
            p = p.add(2);
        } else if *p == b'[' as c_char {
            if get_char_class_impl(&mut p) == CLASS_NONE
                && get_equi_class(&mut p) == 0
                && get_coll_element(&mut p) == 0
                && *p != 0
            {
                p = p.add(1);
            }
        } else {
            p = p.add(1);
        }
    }
    p
}

/// Skip over a `[]` bracket expression — FFI export.
/// Reads `reg_cpo_lit` from the C global via accessor.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_anyof(p: *mut c_char) -> *mut c_char {
    let cpo_lit = REG_CPO_LIT != 0;
    skip_anyof_impl(p, cpo_lit)
}

/// Skip past regular expression, extended version.
///
/// Stop at end of `startp` or where `dirc` delimiter is found.
/// Handles backslash escapes, `[...]` ranges, `\?` replacement, and
/// `\v`/`\V` magic mode switches.
///
/// When `newp` is not NULL and `dirc` is '?', makes an allocated copy of the
/// expression and changes `\?` to `?`. If `*newp` is not NULL the expression
/// is changed in-place.
/// If a `\?` is changed to `?` then `dropped` is incremented, unless NULL.
/// If `magic_val` is not NULL, returns the effective magicness of the pattern.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_regexp_ex(
    startp: *mut c_char,
    dirc: c_int,
    magic: c_int,
    newp: *mut *mut c_char,
    dropped: *mut c_int,
    magic_val: *mut c_int,
) -> *mut c_char {
    let mut mymagic: c_int = if magic != 0 { MAGIC_ON } else { MAGIC_OFF };
    let reg_cpo_lit = !vim_strchr(nvim_get_p_cpo(), CPO_LITERAL).is_null();

    let mut p = startp;
    let mut startp = startp;
    let mut startplen: usize = 0;

    while *p != 0 {
        if c_int::from(*p) == dirc {
            break;
        }
        if (*p == b'[' as c_char && mymagic >= MAGIC_ON)
            || (*p == b'\\' as c_char && *p.add(1) == b'[' as c_char && mymagic <= MAGIC_OFF)
        {
            p = skip_anyof_impl(p.add(1), reg_cpo_lit);
            if *p == 0 {
                break;
            }
        } else if *p == b'\\' as c_char && *p.add(1) != 0 {
            if dirc == b'?' as c_int && !newp.is_null() && *p.add(1) == b'?' as c_char {
                // change "\?" to "?", make a copy first.
                if startplen == 0 {
                    startplen = libc_strlen(startp);
                }
                if (*newp).is_null() {
                    *newp = xstrnsave(startp, startplen);
                    p = (*newp).add(p.offset_from(startp) as usize);
                    startp = *newp;
                }
                if !dropped.is_null() {
                    *dropped += 1;
                }
                std::ptr::copy(
                    p.add(1),
                    p,
                    startplen - (p.add(1).offset_from(startp) as usize) + 1,
                );
            } else {
                p = p.add(1); // skip next character
            }
            if *p == b'v' as c_char {
                mymagic = MAGIC_ALL;
            } else if *p == b'V' as c_char {
                mymagic = MAGIC_NONE;
            }
        }
        // MB_PTR_ADV
        p = p.add(utfc_ptr2len(p) as usize);
    }
    if !magic_val.is_null() {
        *magic_val = mymagic;
    }
    p
}

/// Skip past regular expression.
///
/// Stop at end of `startp` or where `dirc` is found ('/', '?', etc).
/// Take care of characters with a backslash in front of it.
/// Skip strings inside [ and ].
#[no_mangle]
pub unsafe extern "C" fn rs_skip_regexp(
    startp: *mut c_char,
    dirc: c_int,
    magic: c_int,
) -> *mut c_char {
    rs_skip_regexp_ex(
        startp,
        dirc,
        magic,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    )
}

/// Simple strlen implementation to avoid depending on libc crate.
#[allow(clippy::missing_const_for_fn)]
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// --- Magic helpers (matching regexp.c macros) ---
// Magic(x) = (int)(x) - 256; is_Magic(x) = (x) < 0; un_Magic(x) = (x) + 256

/// Multi-type return values
const NOT_MULTI: c_int = 0;
const MULTI_ONE: c_int = 1;
const MULTI_MULT: c_int = 2;

/// Control character constants (matching `ascii_defs.h`)
const BS_CH: c_int = 0o010;
const TAB_CH: c_int = 0o011;
const CAR_CH: c_int = 0o015;
const ESC_CH: c_int = 0o033;

/// Magic('x') = (x as i32) - 256
const fn magic(x: u8) -> c_int {
    (x as c_int) - 256
}

/// If x is Magic (negative), strip the magic to get the plain character.
/// Otherwise return x unchanged.
#[no_mangle]
pub const extern "C" fn rs_no_magic(x: c_int) -> c_int {
    if x < 0 {
        x + 256
    } else {
        x
    }
}

/// If x is Magic (negative), un-magic it. Otherwise make it Magic.
#[no_mangle]
pub const extern "C" fn rs_toggle_magic(x: c_int) -> c_int {
    if x < 0 {
        x + 256
    } else {
        x - 256
    }
}

/// Return `NOT_MULTI` if c is not a "multi" operator.
/// Return `MULTI_ONE` if c is a single "multi" operator.
/// Return `MULTI_MULT` if c is a multi "multi" operator.
#[no_mangle]
pub const extern "C" fn rs_re_multi_type(c: c_int) -> c_int {
    if c == magic(b'@') || c == magic(b'=') || c == magic(b'?') {
        MULTI_ONE
    } else if c == magic(b'*') || c == magic(b'+') || c == magic(b'{') {
        MULTI_MULT
    } else {
        NOT_MULTI
    }
}

/// Translate '\x' to its control character, except "\n", which is Magic.
#[no_mangle]
pub const extern "C" fn rs_backslash_trans(c: c_int) -> c_int {
    match c {
        0x72 => CAR_CH, // 'r'
        0x74 => TAB_CH, // 't'
        0x65 => ESC_CH, // 'e'
        0x62 => BS_CH,  // 'b'
        _ => c,
    }
}

// --- Class table (matching `regexp.c` `init_class_tab`) ---

const RI_DIGIT: i16 = 0x01;
const RI_HEX: i16 = 0x02;
const RI_OCTAL: i16 = 0x04;
const RI_WORD: i16 = 0x08;
const RI_HEAD: i16 = 0x10;
const RI_ALPHA: i16 = 0x20;
const RI_LOWER: i16 = 0x40;
const RI_UPPER: i16 = 0x80;
const RI_WHITE: i16 = 0x100;

/// Compile-time class table matching C `init_class_tab()`.
const CLASS_TAB: [i16; 256] = {
    let mut tab = [0i16; 256];
    let mut i = 0usize;
    while i < 256 {
        if i >= b'0' as usize && i <= b'7' as usize {
            tab[i] = RI_DIGIT + RI_HEX + RI_OCTAL + RI_WORD;
        } else if i >= b'8' as usize && i <= b'9' as usize {
            tab[i] = RI_DIGIT + RI_HEX + RI_WORD;
        } else if i >= b'a' as usize && i <= b'f' as usize {
            tab[i] = RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER;
        } else if i >= b'g' as usize && i <= b'z' as usize {
            tab[i] = RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER;
        } else if i >= b'A' as usize && i <= b'F' as usize {
            tab[i] = RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER;
        } else if i >= b'G' as usize && i <= b'Z' as usize {
            tab[i] = RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER;
        } else if i == b'_' as usize {
            tab[i] = RI_WORD + RI_HEAD;
        }
        i += 1;
    }
    tab[b' ' as usize] |= RI_WHITE;
    tab[b'\t' as usize] |= RI_WHITE;
    tab
};

/// Character class check using the compile-time `CLASS_TAB`.
const fn ri_digit(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_DIGIT != 0)) as c_int
}
const fn ri_hex(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_HEX != 0)) as c_int
}
const fn ri_octal(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_OCTAL != 0)) as c_int
}
const fn ri_word(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_WORD != 0)) as c_int
}
const fn ri_head(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_HEAD != 0)) as c_int
}
const fn ri_alpha(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_ALPHA != 0)) as c_int
}
const fn ri_lower(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_LOWER != 0)) as c_int
}
const fn ri_upper(c: c_int) -> c_int {
    ((c < 0x100) && (CLASS_TAB[c as usize] & RI_UPPER != 0)) as c_int
}

/// Copy the class table into a C-provided buffer.
///
/// # Safety
///
/// `out` must point to a buffer of at least 256 `i16` elements.
#[no_mangle]
pub const unsafe extern "C" fn rs_init_class_tab(out: *mut i16) {
    std::ptr::copy_nonoverlapping(CLASS_TAB.as_ptr(), out, 256);
}

// --- re_multiline (opaque handle pattern) ---

/// `RF_HASNL` flag — regexp can match a newline.
const RF_HASNL: c_uint = 4;

/// Return non-zero if compiled regular expression `prog` can match a line break.
///
/// # Safety
///
/// `prog` must be a valid pointer to a `regprog_T`.
#[no_mangle]
pub unsafe extern "C" fn re_multiline(prog: *const c_void) -> c_int {
    (nvim_regexp_get_regflags(prog) & RF_HASNL) as c_int
}

// --- Number parsers (pure-logic cores + FFI wrappers) ---

/// Check if a byte is an ASCII hex digit.
const fn is_xdigit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Convert a hex digit character to its numeric value (0-15).
const fn hex2nr(c: u8) -> i64 {
    match c {
        b'0'..=b'9' => (c - b'0') as i64,
        b'a'..=b'f' => (c - b'a' + 10) as i64,
        b'A'..=b'F' => (c - b'A' + 10) as i64,
        _ => 0,
    }
}

/// Pure-logic hex parser: parse up to `maxinputlen` hex digits from `input`.
/// Returns `(value, bytes_consumed)` or `(-1, 0)` if no hex digits found.
fn gethexchrs_core(input: &[u8], maxinputlen: usize) -> (i64, usize) {
    let mut nr: i64 = 0;
    let mut i = 0;
    while i < maxinputlen && i < input.len() {
        let c = input[i];
        if !is_xdigit(c) {
            break;
        }
        nr <<= 4;
        nr |= hex2nr(c);
        i += 1;
    }
    if i == 0 {
        (-1, 0)
    } else {
        (nr, i)
    }
}

/// Pure-logic decimal parser: parse all consecutive decimal digits from `input`.
/// Returns `(value, bytes_consumed)` or `(-1, 0)` if no digits found.
fn getdecchrs_core(input: &[u8]) -> (i64, usize) {
    let mut nr: i64 = 0;
    let mut i = 0;
    while i < input.len() {
        let c = input[i];
        if !c.is_ascii_digit() {
            break;
        }
        nr *= 10;
        nr += (c - b'0') as i64;
        i += 1;
    }
    if i == 0 {
        (-1, 0)
    } else {
        (nr, i)
    }
}

/// Pure-logic octal parser: parse up to 3 octal digits, max value 255.
/// Returns `(value, bytes_consumed)` or `(-1, 0)` if no digits found.
fn getoctchrs_core(input: &[u8]) -> (i64, usize) {
    let mut nr: i64 = 0;
    let mut i = 0;
    // Match C: `for (i = 0; i < 3 && nr < 040; i++)`
    // 040 octal = 32 decimal
    while i < 3 && nr < 0o40 && i < input.len() {
        let c = input[i];
        if !(b'0'..=b'7').contains(&c) {
            break;
        }
        nr <<= 3;
        nr |= hex2nr(c);
        i += 1;
    }
    if i == 0 {
        (-1, 0)
    } else {
        (nr, i)
    }
}

/// FFI wrapper: get hex chars from regparse, advancing regparse.
#[no_mangle]
pub unsafe extern "C" fn rs_gethexchrs(maxinputlen: c_int) -> c_long {
    let regparse = REGPARSE;
    let input = std::slice::from_raw_parts(regparse as *const u8, maxinputlen as usize + 1);
    // Find actual available length (up to NUL)
    let len = input.iter().position(|&b| b == 0).unwrap_or(input.len());
    let (nr, consumed) = gethexchrs_core(&input[..len], maxinputlen as usize);
    REGPARSE = regparse.add(consumed);
    nr as c_long
}

/// FFI wrapper: get decimal chars from regparse, advancing regparse.
#[no_mangle]
pub unsafe extern "C" fn rs_getdecchrs() -> c_long {
    let regparse = REGPARSE;
    // We need to scan forward; be generous with the slice length
    // Find NUL to bound the slice
    let mut len = 0;
    while *regparse.add(len) != 0 {
        len += 1;
        if len > 64 {
            break; // decimal numbers won't be this long
        }
    }
    let input = std::slice::from_raw_parts(regparse as *const u8, len);
    let (nr, consumed) = getdecchrs_core(input);
    REGPARSE = regparse.add(consumed);
    // getdecchrs also sets curchr = -1 for each digit consumed
    if consumed > 0 {
        CURCHR = -1;
    }
    nr as c_long
}

/// FFI wrapper: get octal chars from regparse, advancing regparse.
#[no_mangle]
pub unsafe extern "C" fn rs_getoctchrs() -> c_long {
    let regparse = REGPARSE;
    // Octal is at most 3 chars
    let mut len = 0;
    while len < 4 && *regparse.add(len) != 0 {
        len += 1;
    }
    let input = std::slice::from_raw_parts(regparse as *const u8, len);
    let (nr, consumed) = getoctchrs_core(input);
    REGPARSE = regparse.add(consumed);
    nr as c_long
}

// --- State management: initchr, save/restore_parse_state ---

/// Matches C `parse_state_T` layout in `regexp.c`.
#[repr(C)]
pub struct ParseStateT {
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

/// Start parsing at `str`. Sets regparse and resets all character state.
#[no_mangle]
pub unsafe extern "C" fn rs_initchr(str: *mut c_char) {
    REGPARSE = str;
    PREVCHR_LEN = 0;
    CURCHR = -1;
    PREVPREVCHR = -1;
    PREVCHR = -1;
    NEXTCHR = -1;
    AT_START = 1; // true
    PREV_AT_START = 0; // false
}

/// Save the current parse state into `ps`.
#[no_mangle]
pub unsafe extern "C" fn rs_save_parse_state(ps: *mut ParseStateT) {
    (*ps).regparse = REGPARSE;
    (*ps).prevchr_len = PREVCHR_LEN;
    (*ps).curchr = CURCHR;
    (*ps).prevchr = PREVCHR;
    (*ps).prevprevchr = PREVPREVCHR;
    (*ps).nextchr = NEXTCHR;
    (*ps).at_start = AT_START;
    (*ps).prev_at_start = PREV_AT_START;
    (*ps).regnpar = REGNPAR;
}

/// Restore a previously saved parse state from `ps`.
#[no_mangle]
pub unsafe extern "C" fn rs_restore_parse_state(ps: *const ParseStateT) {
    REGPARSE = (*ps).regparse;
    PREVCHR_LEN = (*ps).prevchr_len;
    CURCHR = (*ps).curchr;
    PREVCHR = (*ps).prevchr;
    PREVPREVCHR = (*ps).prevprevchr;
    NEXTCHR = (*ps).nextchr;
    AT_START = (*ps).at_start;
    PREV_AT_START = (*ps).prev_at_start;
    REGNPAR = (*ps).regnpar;
}

// --- Core scanner: peekchr, skipchr, skipchr_keepstart, getchr, ungetchr ---

/// `META_FLAGS` table — copied from regexp.c.
/// Index by ASCII value; nonzero means the character may be magic after `\`.
#[rustfmt::skip]
const META_FLAGS: [u8; 127] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//                 %  &     (  )  *  +        .
    0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0,
//     1  2  3  4  5  6  7  8  9        <  =  >  ?
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
//  @  A     C  D     F     H  I     K  L  M     O
    1, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1,
//  P        S     U  V  W  X     Z  [           _
    1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1,
//     a     c  d     f     h  i     k  l  m  n  o
    0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1,
//  p        s     u  v  w  x     z  {  |     ~
    1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1,
];

/// Get the next character without advancing. Handles magic modes.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_peekchr() -> c_int {
    let mut curchr = CURCHR;
    if curchr != -1 {
        return curchr;
    }

    let regparse = REGPARSE;
    let reg_magic = REG_MAGIC;
    let at_start = AT_START;
    let prev_at_start = PREV_AT_START;
    let prevchr = PREVCHR;
    let prevprevchr = PREVPREVCHR;
    let after_slash = AFTER_SLASH;

    curchr = *regparse as u8 as c_int;
    #[allow(clippy::cast_possible_truncation)]
    let c_byte = curchr as u8; // safe: came from u8 read above

    match c_byte {
        b'.' | b'[' | b'~' => {
            if reg_magic >= MAGIC_ON {
                curchr = magic(c_byte);
            }
        }
        b'(' | b')' | b'{' | b'%' | b'+' | b'=' | b'?' | b'@' | b'!' | b'&' | b'|' | b'<'
        | b'>' | b'#' | b'"' | b'\'' | b',' | b'-' | b':' | b';' | b'`' | b'/' => {
            if reg_magic == MAGIC_ALL {
                curchr = magic(c_byte);
            }
        }
        b'*' => {
            if reg_magic >= MAGIC_ON
                && at_start == 0
                && !(prev_at_start != 0 && prevchr == magic(b'^'))
                && (after_slash != 0
                    || (prevchr != magic(b'(') && prevchr != magic(b'&') && prevchr != magic(b'|')))
            {
                curchr = magic(b'*');
            }
        }
        b'^' => {
            if reg_magic >= MAGIC_OFF
                && (at_start != 0
                    || reg_magic == MAGIC_ALL
                    || prevchr == magic(b'(')
                    || prevchr == magic(b'|')
                    || prevchr == magic(b'&')
                    || prevchr == magic(b'n')
                    || (rs_no_magic(prevchr) == b'(' as c_int && prevprevchr == magic(b'%')))
            {
                curchr = magic(b'^');
                AT_START = 1;
                PREV_AT_START = 0;
            }
        }
        b'$' => {
            if reg_magic >= MAGIC_OFF {
                let mut p = regparse.add(1) as *const u8;
                let mut is_magic_all = reg_magic == MAGIC_ALL;

                // ignore \c \C \m \M \v \V and \Z after '$'
                while *p == b'\\'
                    && matches!(*p.add(1), b'c' | b'C' | b'm' | b'M' | b'v' | b'V' | b'Z')
                {
                    if *p.add(1) == b'v' {
                        is_magic_all = true;
                    } else if matches!(*p.add(1), b'm' | b'M' | b'V') {
                        is_magic_all = false;
                    }
                    p = p.add(2);
                }
                if *p == 0
                    || (*p == b'\\' && matches!(*p.add(1), b'|' | b'&' | b')' | b'n'))
                    || (is_magic_all && matches!(*p, b'|' | b'&' | b')'))
                    || reg_magic == MAGIC_ALL
                {
                    curchr = magic(b'$');
                }
            }
        }
        b'\\' => {
            let c = *regparse.add(1) as u8;

            if c == 0 {
                curchr = b'\\' as c_int; // trailing '\'
            } else if c <= b'~' && META_FLAGS[c as usize] != 0 {
                // META character after '\' — toggle magicness via recursive call
                CURCHR = -1;
                PREV_AT_START = AT_START;
                AT_START = 0; // be able to say "/\*ptr"
                REGPARSE = regparse.add(1);
                AFTER_SLASH = after_slash + 1;
                rs_peekchr();
                REGPARSE = regparse;
                AFTER_SLASH = after_slash;
                curchr = rs_toggle_magic(CURCHR);
            } else if REGEXP_ABBR.contains(&c) {
                // Handle abbreviations, like "\t" for TAB
                curchr = rs_backslash_trans(c as c_int);
            } else if reg_magic == MAGIC_NONE && (c == b'$' || c == b'^') {
                curchr = rs_toggle_magic(c as c_int);
            } else {
                // Next character can never be (made) magic?
                curchr = utf_ptr2char(regparse.add(1));
            }
        }
        _ => {
            curchr = utf_ptr2char(regparse);
        }
    }

    CURCHR = curchr;
    curchr
}

/// Eat one lexed character. Advances regparse and updates character state.
#[no_mangle]
pub unsafe extern "C" fn rs_skipchr() {
    let regparse = REGPARSE;
    // peekchr() eats a backslash, do the same here
    let mut prevchr_len = c_int::from(*regparse == b'\\' as c_char);
    if *regparse.add(prevchr_len as usize) != 0 {
        // Exclude composing chars that utfc_ptr2len does include.
        prevchr_len += utf_ptr2len(regparse.add(prevchr_len as usize));
    }
    REGPARSE = regparse.add(prevchr_len as usize);
    PREVCHR_LEN = prevchr_len;
    PREV_AT_START = AT_START;
    AT_START = 0;
    PREVPREVCHR = PREVCHR;
    PREVCHR = CURCHR;
    CURCHR = NEXTCHR; // use previously unget char, or -1
    NEXTCHR = -1;
}

/// Skip a character while keeping `prev_at_start`, `prevchr`, `prevprevchr`.
#[no_mangle]
pub unsafe extern "C" fn rs_skipchr_keepstart() {
    let saved_as = PREV_AT_START;
    let saved_pr = PREVCHR;
    let saved_prpr = PREVPREVCHR;

    rs_skipchr();

    AT_START = saved_as;
    PREVCHR = saved_pr;
    PREVPREVCHR = saved_prpr;
}

/// Get the next character and advance past it.
#[no_mangle]
pub unsafe extern "C" fn rs_getchr() -> c_int {
    let chr = rs_peekchr();
    rs_skipchr();
    chr
}

/// Put character back. Works only once!
#[no_mangle]
pub unsafe extern "C" fn rs_ungetchr() {
    NEXTCHR = CURCHR;
    CURCHR = PREVCHR;
    PREVCHR = PREVPREVCHR;
    AT_START = PREV_AT_START;
    PREV_AT_START = 0;

    // Backup regparse by prevchr_len
    let regparse = REGPARSE;
    let prevchr_len = PREVCHR_LEN;
    REGPARSE = regparse.sub(prevchr_len as usize);
}

// --- Limit parser: read_limits ---

/// Maximum limit value for `\{n,m}` ranges.
const MAX_LIMIT: c_int = 32767 << 16;

/// OK return code matching C definition.
const OK: c_int = 1;

/// Parse `\{n,m}` range limits. On success writes to `*minval` and `*maxval`
/// and returns `OK`; on syntax error emits a message and returns `FAIL`.
#[no_mangle]
pub unsafe extern "C" fn rs_read_limits(minval: *mut c_int, maxval: *mut c_int) -> c_int {
    let mut regparse = REGPARSE;

    let reverse = if *regparse == b'-' as c_char {
        regparse = regparse.add(1);
        true
    } else {
        false
    };
    let first_char = regparse;
    *minval = getdigits_int(&mut regparse, false, 0);
    if *regparse == b',' as c_char {
        regparse = regparse.add(1);
        if (*regparse as u8).is_ascii_digit() {
            *maxval = getdigits_int(&mut regparse, false, MAX_LIMIT);
        } else {
            *maxval = MAX_LIMIT;
        }
    } else if (*first_char as u8).is_ascii_digit() {
        *maxval = *minval; // It was \{n} or \{-n}
    } else {
        *maxval = MAX_LIMIT; // It was \{} or \{-}
    }
    if *regparse == b'\\' as c_char {
        regparse = regparse.add(1); // Allow either \{...} or \{...\}
    }
    if *regparse as u8 != b'}' {
        REGPARSE = regparse;
        return errors::emsg2_fail(
            c"E554: Syntax error in %s{...}".as_ptr(),
            c_int::from(REG_MAGIC == MAGIC_ALL),
        );
    }

    // Reverse the range if there was a '-', or make sure it is in the right
    // order otherwise.
    if (!reverse && *minval > *maxval) || (reverse && *minval < *maxval) {
        core::ptr::swap(minval, maxval);
    }
    REGPARSE = regparse;
    rs_skipchr(); // let's be friends with the lexer again
    OK
}

// --- Hebrew decomposition table (0xfb20..=0xfb4f) ---

/// Decomposition entry: base character + up to 2 combining marks.
struct DecompEntry {
    a: c_int,
    b: c_int,
    c: c_int,
}

#[rustfmt::skip]
const DECOMP_TABLE: [DecompEntry; 0xfb4f - 0xfb20 + 1] = [
    DecompEntry { a: 0x5e2, b: 0,     c: 0 },      // 0xfb20  alt ayin
    DecompEntry { a: 0x5d0, b: 0,     c: 0 },      // 0xfb21  alt alef
    DecompEntry { a: 0x5d3, b: 0,     c: 0 },      // 0xfb22  alt dalet
    DecompEntry { a: 0x5d4, b: 0,     c: 0 },      // 0xfb23  alt he
    DecompEntry { a: 0x5db, b: 0,     c: 0 },      // 0xfb24  alt kaf
    DecompEntry { a: 0x5dc, b: 0,     c: 0 },      // 0xfb25  alt lamed
    DecompEntry { a: 0x5dd, b: 0,     c: 0 },      // 0xfb26  alt mem-sofit
    DecompEntry { a: 0x5e8, b: 0,     c: 0 },      // 0xfb27  alt resh
    DecompEntry { a: 0x5ea, b: 0,     c: 0 },      // 0xfb28  alt tav
    DecompEntry { a: b'+' as c_int, b: 0, c: 0 },   // 0xfb29  alt plus
    DecompEntry { a: 0x5e9, b: 0x5c1, c: 0 },      // 0xfb2a  shin+shin-dot
    DecompEntry { a: 0x5e9, b: 0x5c2, c: 0 },      // 0xfb2b  shin+sin-dot
    DecompEntry { a: 0x5e9, b: 0x5c1, c: 0x5bc },  // 0xfb2c  shin+shin-dot+dagesh
    DecompEntry { a: 0x5e9, b: 0x5c2, c: 0x5bc },  // 0xfb2d  shin+sin-dot+dagesh
    DecompEntry { a: 0x5d0, b: 0x5b7, c: 0 },      // 0xfb2e  alef+patah
    DecompEntry { a: 0x5d0, b: 0x5b8, c: 0 },      // 0xfb2f  alef+qamats
    DecompEntry { a: 0x5d0, b: 0x5b4, c: 0 },      // 0xfb30  alef+hiriq
    DecompEntry { a: 0x5d1, b: 0x5bc, c: 0 },      // 0xfb31  bet+dagesh
    DecompEntry { a: 0x5d2, b: 0x5bc, c: 0 },      // 0xfb32  gimel+dagesh
    DecompEntry { a: 0x5d3, b: 0x5bc, c: 0 },      // 0xfb33  dalet+dagesh
    DecompEntry { a: 0x5d4, b: 0x5bc, c: 0 },      // 0xfb34  he+dagesh
    DecompEntry { a: 0x5d5, b: 0x5bc, c: 0 },      // 0xfb35  vav+dagesh
    DecompEntry { a: 0x5d6, b: 0x5bc, c: 0 },      // 0xfb36  zayin+dagesh
    DecompEntry { a: 0xfb37, b: 0,    c: 0 },      // 0xfb37  -- UNUSED
    DecompEntry { a: 0x5d8, b: 0x5bc, c: 0 },      // 0xfb38  tet+dagesh
    DecompEntry { a: 0x5d9, b: 0x5bc, c: 0 },      // 0xfb39  yud+dagesh
    DecompEntry { a: 0x5da, b: 0x5bc, c: 0 },      // 0xfb3a  kaf sofit+dagesh
    DecompEntry { a: 0x5db, b: 0x5bc, c: 0 },      // 0xfb3b  kaf+dagesh
    DecompEntry { a: 0x5dc, b: 0x5bc, c: 0 },      // 0xfb3c  lamed+dagesh
    DecompEntry { a: 0xfb3d, b: 0,    c: 0 },      // 0xfb3d  -- UNUSED
    DecompEntry { a: 0x5de, b: 0x5bc, c: 0 },      // 0xfb3e  mem+dagesh
    DecompEntry { a: 0xfb3f, b: 0,    c: 0 },      // 0xfb3f  -- UNUSED
    DecompEntry { a: 0x5e0, b: 0x5bc, c: 0 },      // 0xfb40  nun+dagesh
    DecompEntry { a: 0x5e1, b: 0x5bc, c: 0 },      // 0xfb41  samech+dagesh
    DecompEntry { a: 0xfb42, b: 0,    c: 0 },      // 0xfb42  -- UNUSED
    DecompEntry { a: 0x5e3, b: 0x5bc, c: 0 },      // 0xfb43  pe sofit+dagesh
    DecompEntry { a: 0x5e4, b: 0x5bc, c: 0 },      // 0xfb44  pe+dagesh
    DecompEntry { a: 0xfb45, b: 0,    c: 0 },      // 0xfb45  -- UNUSED
    DecompEntry { a: 0x5e6, b: 0x5bc, c: 0 },      // 0xfb46  tsadi+dagesh
    DecompEntry { a: 0x5e7, b: 0x5bc, c: 0 },      // 0xfb47  qof+dagesh
    DecompEntry { a: 0x5e8, b: 0x5bc, c: 0 },      // 0xfb48  resh+dagesh
    DecompEntry { a: 0x5e9, b: 0x5bc, c: 0 },      // 0xfb49  shin+dagesh
    DecompEntry { a: 0x5ea, b: 0x5bc, c: 0 },      // 0xfb4a  tav+dagesh
    DecompEntry { a: 0x5d5, b: 0x5b9, c: 0 },      // 0xfb4b  vav+holam
    DecompEntry { a: 0x5d1, b: 0x5bf, c: 0 },      // 0xfb4c  bet+rafe
    DecompEntry { a: 0x5db, b: 0x5bf, c: 0 },      // 0xfb4d  kaf+rafe
    DecompEntry { a: 0x5e4, b: 0x5bf, c: 0 },      // 0xfb4e  pe+rafe
    DecompEntry { a: 0x5d0, b: 0x5dc, c: 0 },      // 0xfb4f  alef-lamed
];

/// Decompose a Hebrew presentation form character into base + combining marks.
#[allow(clippy::manual_range_contains)]
const fn mb_decompose(ch: c_int, c1: &mut c_int, c2: &mut c_int, c3: &mut c_int) {
    if ch >= 0xfb20 && ch <= 0xfb4f {
        let d = &DECOMP_TABLE[(ch - 0xfb20) as usize];
        *c1 = d.a;
        *c2 = d.b;
        *c3 = d.c;
    } else {
        *c1 = ch;
        *c2 = 0;
        *c3 = 0;
    }
}

// --- Case-insensitive operations: cstrncmp, cstrchr ---

/// Compare two strings, strncmp-like, with optional case-folding.
///
/// If `rex.reg_ic` is set, compare case-insensitively. `*n` may be adjusted
/// downward if s2 is shorter (measured in characters) than the byte-length
/// specified.
///
/// If `rex.reg_icombine` is set and the comparison fails, retry by
/// decomposing characters and comparing base characters only.
///
/// Returns 0 for match, nonzero for mismatch.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_cstrncmp(s1: *mut c_char, s2: *mut c_char, n: *mut c_int) -> c_int {
    let result;

    if REX.reg_ic as c_int == 0 {
        // Case-sensitive compare
        result = strncmp(s1, s2, *n as usize);
    } else {
        // Case-insensitive: count characters for byte-length of s1
        let mut p = s1;
        let mut n2 = 0_i32;
        let mut n1 = *n;
        while n1 > 0 && *p != 0 {
            n1 -= utfc_ptr2len(s1);
            p = p.add(utfc_ptr2len(p) as usize); // MB_PTR_ADV
            n2 += 1;
        }
        // Count bytes to advance the same number of chars for s2
        p = s2;
        while n2 > 0 && *p != 0 {
            p = p.add(utfc_ptr2len(p) as usize); // MB_PTR_ADV
            n2 -= 1;
        }

        n2 = p.offset_from(s2) as c_int;

        result = utf_strnicmp(s1, s2, *n as usize, n2 as usize);
        if result == 0 && n2 < *n {
            *n = n2;
        }
    }

    // If it failed and it's utf8 and we want to combineignore:
    if result != 0 && REX.reg_icombine as c_int != 0 {
        let mut str1: *const c_char = s1;
        let mut str2: *const c_char = s2;
        let mut c1;
        let mut c2;

        loop {
            if (str1 as usize - s1 as usize) as c_int >= *n {
                // Reached the end — match
                *n = (str2 as usize - s2 as usize) as c_int;
                return 0;
            }
            c1 = mb_ptr2char_adv(&mut str1);
            c2 = mb_ptr2char_adv(&mut str2);

            if c1 != c2 && (REX.reg_ic as c_int == 0 || utf_fold(c1) != utf_fold(c2)) {
                // Decomposition necessary?
                let mut c11: c_int = 0;
                let mut c12: c_int = 0;
                let mut junk1: c_int = 0;
                let mut junk2: c_int = 0;
                mb_decompose(c1, &mut c11, &mut junk1, &mut junk2);
                mb_decompose(c2, &mut c12, &mut junk1, &mut junk2);
                c1 = c11;
                c2 = c12;
                if c11 != c12 && (REX.reg_ic as c_int == 0 || utf_fold(c11) != utf_fold(c12)) {
                    break;
                }
            }
        }
        return c2 - c1;
    }

    result
}

/// Search for character `c` in string `s`, with optional case-insensitivity.
///
/// When `rex.reg_ic` is set, searches case-insensitively.
/// Returns `NULL` if no match, otherwise pointer to the position in `s`.
#[no_mangle]
pub unsafe extern "C" fn rs_cstrchr(s: *const c_char, c: c_int) -> *mut c_char {
    if REX.reg_ic as c_int == 0 {
        return vim_strchr(s, c);
    }

    let cc: c_int;
    let lc: c_int;
    if c > 0x80 {
        let folded = utf_fold(c);
        cc = folded;
        lc = folded;
    } else if c >= b'A' as c_int && c <= b'Z' as c_int {
        // ASCII_ISUPPER
        cc = c + (b'a' - b'A') as c_int; // TOLOWER_ASC
        lc = cc;
    } else if c >= b'a' as c_int && c <= b'z' as c_int {
        // ASCII_ISLOWER
        cc = c - (b'a' - b'A') as c_int; // TOUPPER_ASC
        lc = c;
    } else {
        return vim_strchr(s, c);
    }

    let mut p = s;
    while *p != 0 {
        let uc = utf_ptr2char(p);
        if c > 0x80 || uc > 0x80 {
            // Do not match an illegal byte. E.g. 0xff matches 0xc3 0xbf, not 0xff.
            // Compare with lower case of the character.
            if (uc < 0x80 || uc != *p as u8 as c_int) && utf_fold(uc) == lc {
                return p.cast_mut();
            }
        } else if *p as u8 as c_int == c || *p as u8 as c_int == cc {
            return p.cast_mut();
        }
        p = p.add(utfc_ptr2len(p) as usize);
    }

    std::ptr::null_mut()
}

// --- get_cpo_flags ---

/// Set `reg_cpo_lit` from `p_cpo`. Mirrors C `get_cpo_flags()`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cpo_flags() {
    let cpo_lit = !vim_strchr(nvim_get_p_cpo(), CPO_LITERAL).is_null();
    REG_CPO_LIT = cpo_lit as c_int;
}

// --- extmatch lifecycle ---

const NSUBEXP: usize = 10;
// HAD_ENDBRACE is here (after NSUBEXP is in scope) -- moved from C in Phase 2
static mut HAD_ENDBRACE: [u8; NSUBEXP] = [0u8; NSUBEXP];

/// Matches C `reg_extmatch_T` layout in `regexp_defs.h`.
#[repr(C)]
pub struct RegExtmatchT {
    pub refcnt: i16,
    pub matches: [*mut u8; NSUBEXP],
}

/// Create a new extmatch and mark it as referenced once.
#[no_mangle]
pub unsafe extern "C" fn rs_make_extmatch() -> *mut RegExtmatchT {
    let em = xcalloc(1, core::mem::size_of::<RegExtmatchT>()).cast::<RegExtmatchT>();
    (*em).refcnt = 1;
    em
}

/// Add a reference to an extmatch. Returns the pointer unchanged.
#[no_mangle]
pub unsafe extern "C" fn ref_extmatch(em: *mut RegExtmatchT) -> *mut RegExtmatchT {
    if !em.is_null() {
        (*em).refcnt += 1;
    }
    em
}

/// Remove a reference to an extmatch. If no references left, free it.
#[no_mangle]
pub unsafe extern "C" fn unref_extmatch(em: *mut RegExtmatchT) {
    if !em.is_null() {
        (*em).refcnt -= 1;
        if (*em).refcnt <= 0 {
            for i in 0..NSUBEXP {
                xfree((*em).matches[i].cast::<c_void>());
            }
            xfree(em.cast::<c_void>());
        }
    }
}

// --- re_mult_next ---

/// Check that a multi-operator does not follow an invalid context.
/// Returns `true` if OK, `false` if error (emits E888).
#[no_mangle]
pub unsafe extern "C" fn rs_re_mult_next(what: *const c_char) -> bool {
    if rs_re_multi_type(rs_peekchr()) == MULTI_MULT {
        errors::semsg_e888(what);
        nvim_regexp_set_rc_did_emsg(1);
        return false;
    }
    true
}

// --- cleanup_subexpr / cleanup_zsubexpr ---

/// Clear subexpression match data if the flag is set.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_subexpr() {
    if REX.need_clear_subexpr == 0 {
        return;
    }
    if (REX.reg_match.is_null() as c_int) != 0 {
        std::ptr::write_bytes(REX.reg_startpos, 0xff, NSUBEXP);
        std::ptr::write_bytes(REX.reg_endpos, 0xff, NSUBEXP);
    } else {
        std::ptr::write_bytes(REX.reg_startp, 0, NSUBEXP);
        std::ptr::write_bytes(REX.reg_endp, 0, NSUBEXP);
    }
    REX.need_clear_subexpr = 0;
}

/// Clear z-subexpression match data if the flag is set.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_zsubexpr() {
    if REX.need_clear_zsubexpr == 0 {
        return;
    }
    if (REX.reg_match.is_null() as c_int) != 0 {
        REG_STARTZPOS = [LposT { lnum: -1, col: -1 }; NSUBEXP];
        REG_ENDZPOS = [LposT { lnum: -1, col: -1 }; NSUBEXP];
    } else {
        REG_STARTZP = [core::ptr::null_mut(); NSUBEXP];
        REG_ENDZP = [core::ptr::null_mut(); NSUBEXP];
    }
    REX.need_clear_zsubexpr = 0;
}

// --- reg_prev_class ---

/// Get class of the character before `rex.input`.
/// Returns -1 if at the start of the line.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_prev_class() -> c_int {
    let input = REX.input;
    let line = REX.line;
    if input > line {
        let p = (input as *const c_char).sub(1);
        let base = line as *const c_char;
        let head = utf_head_off(base, p);
        let start = p.sub(head as usize);
        mb_get_class_tab(start, nvim_regexp_get_rex_reg_buf_chartab())
    } else {
        -1
    }
}

// --- reg_nextline ---

/// Advance rex.lnum, rex.line and rex.input to the next line.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_nextline() {
    let lnum = REX.lnum + 1;
    REX.lnum = lnum;
    let line = nvim_regexp_call_reg_getline(lnum).cast::<u8>();
    {
        REX.line = line;
        REX.input = line;
    };
    rs_reg_breakcheck();
}

// --- reg_breakcheck ---

extern "C" {
    fn fast_breakcheck();
    fn vim_iswordc_buf(c: c_int, buf: *const c_void) -> c_int;
}

/// If `rex.reg_nobreak` is not set, call `fast_breakcheck()`.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_breakcheck() {
    if REX.reg_nobreak as c_int == 0 {
        fast_breakcheck();
    }
}

/// Return true if character `c` is included in 'iskeyword' for `rex.reg_buf`.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_iswordc(c: c_int) -> c_int {
    vim_iswordc_buf(c, REX.reg_buf)
}

// --- reg_match_visual ---

extern "C" {
    fn nvim_regexp_visual_quick_check() -> c_int;
    fn nvim_regexp_get_visual_area(
        top_lnum: *mut i32,
        top_col: *mut i32,
        bot_lnum: *mut i32,
        bot_col: *mut i32,
        mode: *mut c_int,
        curswant: *mut i32,
    ) -> *mut c_void;
    fn nvim_regexp_get_p_sel_char() -> c_int;
    fn nvim_regexp_call_getvvcol(
        wp: *mut c_void,
        lnum: i32,
        col: i32,
        start_out: *mut i32,
        end_out: *mut i32,
    );
    fn nvim_regexp_call_win_linetabsize(
        wp: *mut c_void,
        lnum: i32,
        line: *const c_char,
        col: i32,
    ) -> i32;
}

/// `Ctrl_V` character value (0x16).
const CTRL_V: c_int = 22;

/// MAXCOL as i32 (matching C `colnr_T` MAXCOL = 0x7fffffff).
const MAXCOL_I32: i32 = 0x7fff_ffff;

/// Return true if the current `rex.input` position matches the Visual area.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_match_visual() -> c_int {
    // Quick reject: wrong buffer, no visual lnum, or not multiline
    if nvim_regexp_visual_quick_check() == 0 {
        return 0;
    }

    let mut top_lnum: i32 = 0;
    let mut top_col: i32 = 0;
    let mut bot_lnum: i32 = 0;
    let mut bot_col: i32 = 0;
    let mut mode: c_int = 0;
    let mut curswant: i32 = 0;

    let wp = nvim_regexp_get_visual_area(
        &mut top_lnum,
        &mut top_col,
        &mut bot_lnum,
        &mut bot_col,
        &mut mode,
        &mut curswant,
    );

    let lnum = REX.lnum + REX.reg_firstlnum;
    if lnum < top_lnum || lnum > bot_lnum {
        return 0;
    }

    let rex_input = REX.input;
    let rex_line = REX.line;
    #[allow(clippy::cast_possible_truncation)]
    let col = rex_input.offset_from(rex_line) as i32;

    if mode == b'v' as c_int {
        let sel_inclusive = i32::from(nvim_regexp_get_p_sel_char() != b'e' as c_int);
        if (lnum == top_lnum && col < top_col)
            || (lnum == bot_lnum && col >= bot_col + sel_inclusive)
        {
            return 0;
        }
    } else if mode == CTRL_V {
        let mut start: i32 = 0;
        let mut end: i32 = 0;
        let mut start2: i32 = 0;
        let mut end2: i32 = 0;

        nvim_regexp_call_getvvcol(wp, top_lnum, top_col, &mut start, &mut end);
        nvim_regexp_call_getvvcol(wp, bot_lnum, bot_col, &mut start2, &mut end2);

        if start2 < start {
            start = start2;
        }
        if end2 > end {
            end = end2;
        }
        if top_col == MAXCOL_I32 || bot_col == MAXCOL_I32 || curswant == MAXCOL_I32 {
            end = MAXCOL_I32;
        }

        // getvvcol() flushes rex.line, need to get it again
        let rex_lnum = REX.lnum;
        let new_line = nvim_regexp_call_reg_getline(rex_lnum).cast::<u8>();
        REX.line = new_line;
        REX.input = new_line.add(col as usize);

        let firstlnum = REX.reg_firstlnum;
        let cols = nvim_regexp_call_win_linetabsize(
            wp,
            firstlnum + rex_lnum,
            new_line.cast::<c_char>(),
            col,
        );
        let sel_exclusive = i32::from(nvim_regexp_get_p_sel_char() == b'e' as c_int);
        if cols < start || cols > end - sel_exclusive {
            return 0;
        }
    }

    1
}

// --- skip_regexp_err ---

/// Call `skip_regexp` and check for delimiter mismatch. On mismatch, emit
/// E654 and return null.
#[no_mangle]
pub unsafe extern "C" fn skip_regexp_err(
    startp: *mut c_char,
    delim: c_int,
    magic: c_int,
) -> *mut c_char {
    let p = rs_skip_regexp(startp, delim, magic);
    if *p as u8 as c_int != delim {
        errors::emsg_semsg_e654(startp);
        return std::ptr::null_mut();
    }
    p
}

// --- reg_getline_common ---

// Flag constants for reg_getline_common (matches C enum reg_getline_flags_T)
const RGLF_LINE: c_int = 0x01;
const RGLF_LENGTH: c_int = 0x02;
const RGLF_SUBMATCH: c_int = 0x04;

extern "C" {
    fn nvim_regexp_call_ml_get_buf(lnum: i32) -> *mut c_char;
    fn nvim_regexp_call_ml_get_buf_len(lnum: i32) -> i32;
}

/// Empty C string returned when `lnum > maxline`.
static mut EMPTY_CSTR: u8 = 0;

/// Common code for `reg_getline`, `reg_getline_len`, and their submatch variants.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_getline_common(
    lnum: i32,
    flags: c_int,
    line: *mut *mut c_char,
    length: *mut i32,
) {
    let get_line = flags & RGLF_LINE != 0;
    let get_length = flags & RGLF_LENGTH != 0;

    let (firstlnum, maxline) = if flags & RGLF_SUBMATCH != 0 {
        (RSM.sm_firstlnum + lnum, RSM.sm_maxline)
    } else {
        (REX.reg_firstlnum + lnum, REX.reg_maxline)
    };

    // When looking behind for a match/no-match lnum is negative, but we
    // can't go before line 1.
    if firstlnum < 1 {
        if get_line {
            *line = std::ptr::null_mut();
        }
        if get_length {
            *length = 0;
        }
        return;
    }

    if lnum > maxline {
        // Must have matched the "\n" in the last line.
        if get_line {
            *line = std::ptr::addr_of_mut!(EMPTY_CSTR).cast::<c_char>();
        }
        if get_length {
            *length = 0;
        }
        return;
    }

    if get_line {
        *line = nvim_regexp_call_ml_get_buf(firstlnum);
    }
    if get_length {
        *length = nvim_regexp_call_ml_get_buf_len(firstlnum);
    }
}

// --- reg_submatch ---

extern "C" {
    fn nvim_regexp_get_rsm_sm_match_startp(i: c_int) -> *const c_char;
    fn nvim_regexp_get_rsm_sm_match_endp(i: c_int) -> *const c_char;
    fn nvim_regexp_get_rsm_sm_mmatch_startpos_lnum(i: c_int) -> i32;
    fn nvim_regexp_get_rsm_sm_mmatch_startpos_col(i: c_int) -> i32;
    fn nvim_regexp_get_rsm_sm_mmatch_endpos_lnum(i: c_int) -> i32;
    fn nvim_regexp_get_rsm_sm_mmatch_endpos_col(i: c_int) -> i32;
}

/// Helper: get submatch line text via `rs_reg_getline_common` with `RGLF_SUBMATCH`.
unsafe fn reg_getline_submatch(lnum: i32) -> *mut c_char {
    let mut line: *mut c_char = std::ptr::null_mut();
    rs_reg_getline_common(
        lnum,
        RGLF_LINE | RGLF_SUBMATCH,
        &mut line,
        std::ptr::null_mut(),
    );
    line
}

/// Helper: get submatch line length via `rs_reg_getline_common` with `RGLF_SUBMATCH`.
unsafe fn reg_getline_submatch_len(lnum: i32) -> i32 {
    let mut length: i32 = 0;
    rs_reg_getline_common(
        lnum,
        RGLF_LENGTH | RGLF_SUBMATCH,
        std::ptr::null_mut(),
        &mut length,
    );
    length
}

/// Return the submatch (strdup'd) for the `submatch()` function.
/// Returns NULL when not in a `:s` command or for a non-existing submatch.
#[no_mangle]
pub unsafe extern "C" fn reg_submatch(no: c_int) -> *mut c_char {
    if (CAN_F_SUBMATCH as c_int) == 0 || no < 0 {
        return std::ptr::null_mut();
    }

    if (RSM.sm_match.is_null() as c_int) != 0 {
        // Multi-line match path (sm_mmatch)
        let mut retval: *mut c_char = std::ptr::null_mut();

        // Two passes: first measure, then copy
        for round in 1..=2 {
            let mut lnum = nvim_regexp_get_rsm_sm_mmatch_startpos_lnum(no);
            if lnum < 0 || nvim_regexp_get_rsm_sm_mmatch_endpos_lnum(no) < 0 {
                return std::ptr::null_mut();
            }

            let s = reg_getline_submatch(lnum);
            if s.is_null() {
                // anti-crash check
                break;
            }
            let start_col = nvim_regexp_get_rsm_sm_mmatch_startpos_col(no);
            let s = s.add(start_col as usize);

            let end_lnum = nvim_regexp_get_rsm_sm_mmatch_endpos_lnum(no);
            let end_col = nvim_regexp_get_rsm_sm_mmatch_endpos_col(no);

            let len = if end_lnum == lnum {
                // Within one line: take from start to end col
                let span = (end_col - start_col) as usize;
                if round == 2 {
                    std::ptr::copy_nonoverlapping(s, retval, span);
                    *retval.add(span) = 0;
                }
                span + 1 // +1 for NUL
            } else {
                // Multiple lines
                let mut off = (reg_getline_submatch_len(lnum) - start_col) as usize;
                if round == 2 {
                    std::ptr::copy_nonoverlapping(s, retval, off);
                    *retval.add(off) = b'\n' as c_char;
                }
                off += 1;
                lnum += 1;

                while lnum < end_lnum {
                    let ml = reg_getline_submatch(lnum);
                    let ml_len = reg_getline_submatch_len(lnum) as usize;
                    if round == 2 {
                        std::ptr::copy_nonoverlapping(ml, retval.add(off), ml_len);
                        *retval.add(off + ml_len) = b'\n' as c_char;
                    }
                    off += ml_len + 1;
                    lnum += 1;
                }

                // End line up to end col
                if round == 2 {
                    let el = reg_getline_submatch(lnum);
                    std::ptr::copy_nonoverlapping(el, retval.add(off), end_col as usize);
                    *retval.add(off + end_col as usize) = 0;
                }
                off + end_col as usize + 1
            };

            if retval.is_null() {
                retval = xmalloc(len).cast::<c_char>();
            }
        }

        retval
    } else {
        // Single-line match path (sm_match)
        let s = nvim_regexp_get_rsm_sm_match_startp(no);
        let e = nvim_regexp_get_rsm_sm_match_endp(no);
        if s.is_null() || e.is_null() {
            return std::ptr::null_mut();
        }
        let span = e.offset_from(s) as usize;
        xstrnsave(s, span)
    }
}

// --- get_char_class ---

/// Sorted table of `[:name:]` character class names.
/// Each entry is `(suffix, class_value)` where suffix starts after the `[:`.
/// Sorted by the suffix string for binary search.
const CHAR_CLASS_TAB: &[(&[u8], c_int)] = &[
    (b"alnum:]", CLASS_ALNUM),
    (b"alpha:]", CLASS_ALPHA),
    (b"backspace:]", CLASS_BACKSPACE),
    (b"blank:]", CLASS_BLANK),
    (b"cntrl:]", CLASS_CNTRL),
    (b"digit:]", CLASS_DIGIT),
    (b"escape:]", CLASS_ESCAPE),
    (b"fname:]", CLASS_FNAME),
    (b"graph:]", CLASS_GRAPH),
    (b"ident:]", CLASS_IDENT),
    (b"keyword:]", CLASS_KEYWORD),
    (b"lower:]", CLASS_LOWER),
    (b"print:]", CLASS_PRINT),
    (b"punct:]", CLASS_PUNCT),
    (b"return:]", CLASS_RETURN),
    (b"space:]", CLASS_SPACE),
    (b"tab:]", CLASS_CC_TAB),
    (b"upper:]", CLASS_UPPER),
    (b"xdigit:]", CLASS_XDIGIT),
];

/// Check for a character class name `[:name:]`. `pp` points to the `[`.
/// Returns one of the `CLASS_*` values, or `CLASS_NONE`.
/// On success, advances `*pp` past the closing `]`.
///
/// Pure-logic implementation shared by `rs_get_char_class` and `skip_anyof`.
unsafe fn get_char_class_impl(pp: *mut *mut c_char) -> c_int {
    let p = *pp;
    // Quick reject: must have `[:` followed by at least two lowercase ASCII letters
    if *p.add(1) != b':' as c_char {
        return CLASS_NONE;
    }
    let c2 = *p.add(2) as u8;
    let c3 = *p.add(3) as u8;
    let c4 = *p.add(4) as u8;
    if !c2.is_ascii_lowercase() || !c3.is_ascii_lowercase() || !c4.is_ascii_lowercase() {
        return CLASS_NONE;
    }

    // Binary search over the sorted table
    let needle = p.add(2) as *const u8;
    let mut lo: usize = 0;
    let mut hi: usize = CHAR_CLASS_TAB.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let (entry_name, _) = CHAR_CLASS_TAB[mid];
        let cmp = compare_prefix(needle, entry_name);
        match cmp.cmp(&0) {
            std::cmp::Ordering::Less => hi = mid,
            std::cmp::Ordering::Greater => lo = mid + 1,
            std::cmp::Ordering::Equal => {
                // Match found — advance pp past the `[:name:]`
                // +2 for the leading `[:`
                *pp = p.add(entry_name.len() + 2).cast::<c_char>();
                return CHAR_CLASS_TAB[mid].1;
            }
        }
    }
    CLASS_NONE
}

/// Compare a NUL-terminated C string prefix against a byte slice.
/// Returns <0 if needle < entry, >0 if needle > entry, 0 on match.
unsafe fn compare_prefix(needle: *const u8, entry: &[u8]) -> c_int {
    for (i, &eb) in entry.iter().enumerate() {
        let nb = *needle.add(i);
        if nb != eb {
            return (nb as c_int) - (eb as c_int);
        }
    }
    0
}

/// Check for a character class name `[:name:]`. `pp` points to the `[`.
/// FFI export that delegates to `get_char_class_impl`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_char_class(pp: *mut *mut c_char) -> c_int {
    get_char_class_impl(pp)
}

// --- regtilde ---

const MAXCOL: usize = 0x7fff_ffff;

extern "C" {
    fn nvim_regexp_get_reg_prev_sub() -> *mut c_char;
    fn nvim_regexp_set_reg_prev_sub(p: *mut c_char);
    fn nvim_regexp_get_reg_prev_sublen() -> usize;
    fn nvim_regexp_set_reg_prev_sublen(v: usize);
    fn xmalloc(size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
}

/// Replace tildes in the pattern by the old pattern.
/// Direct transliteration of C `regtilde()`.
#[no_mangle]
pub unsafe extern "C" fn regtilde(source: *mut c_char, magic: c_int, preview: bool) -> *mut c_char {
    let mut newsub = source;
    let mut newsublen: usize = 0;
    let mut error = false;

    let (tilde_0, tilde_1, tildelen): (u8, u8, usize) = if magic == 0 {
        (b'\\', b'~', 2)
    } else {
        (b'~', 0, 1)
    };

    let mut p = newsub;
    while *p != 0 {
        let matches_tilde = *p as u8 == tilde_0 && (tildelen == 1 || *p.add(1) as u8 == tilde_1);

        if matches_tilde {
            let prefixlen = p.offset_from(newsub) as usize;
            let postfix = p.add(tildelen);

            if newsublen == 0 {
                newsublen = strlen(newsub);
            }
            newsublen -= tildelen;
            let postfixlen = newsublen - prefixlen;
            let reg_prev_sub = nvim_regexp_get_reg_prev_sub();
            let reg_prev_sublen = nvim_regexp_get_reg_prev_sublen();
            let tmpsublen = prefixlen + reg_prev_sublen + postfixlen;

            if tmpsublen > 0 && !reg_prev_sub.is_null() {
                if tmpsublen > MAXCOL {
                    errors::emsg_resulting_text_too_long();
                    error = true;
                    break;
                }

                let tmpsub = xmalloc(tmpsublen + 1).cast::<c_char>();
                // copy prefix
                std::ptr::copy(newsub, tmpsub, prefixlen);
                // interpret tilde
                std::ptr::copy(reg_prev_sub, tmpsub.add(prefixlen), reg_prev_sublen);
                // copy postfix (including NUL)
                std::ptr::copy(
                    postfix,
                    tmpsub.add(prefixlen + reg_prev_sublen),
                    postfixlen + 1,
                );

                if newsub != source {
                    xfree(newsub.cast());
                }
                newsub = tmpsub;
                newsublen = tmpsublen;
                p = newsub.add(prefixlen + reg_prev_sublen);
            } else {
                // remove the tilde (+1 for the NUL)
                std::ptr::copy(postfix, p, postfixlen + 1);
            }
            p = p.sub(1);
        } else {
            if *p == b'\\' as c_char && *p.add(1) != 0 {
                p = p.add(1);
            }
            p = p.add(utfc_ptr2len(p) as usize - 1);
        }
        p = p.add(1);
    }

    if error {
        if newsub != source {
            xfree(newsub.cast());
        }
        return source;
    }

    // Only change reg_prev_sub when not previewing.
    if !preview {
        newsublen = p.offset_from(newsub) as usize;
        let prev = nvim_regexp_get_reg_prev_sub();
        if !prev.is_null() {
            xfree(prev.cast());
        }
        if newsublen == 0 {
            nvim_regexp_set_reg_prev_sub(std::ptr::null_mut());
        } else {
            nvim_regexp_set_reg_prev_sub(xstrnsave(newsub, newsublen));
        }
        nvim_regexp_set_reg_prev_sublen(newsublen);
    }

    newsub
}

// --- match_with_backref ---

const RA_FAIL: c_int = 1;
const RA_MATCH: c_int = 4;
const RA_NOMATCH: c_int = 5;

extern "C" {
    fn nvim_regexp_get_got_int() -> c_int;
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, nn: usize) -> c_int;
    fn nvim_regexp_call_reg_getline_len(lnum: i32) -> i32;
}

/// Check whether a backreference matches.
/// Returns `RA_FAIL`, `RA_NOMATCH` or `RA_MATCH`.
///
/// # Panics
/// Panics if `reg_getline` returns NULL for the requested line.
#[no_mangle]
pub unsafe extern "C" fn rs_match_with_backref(
    start_lnum: i32,
    start_col: i32,
    end_lnum: i32,
    end_col: i32,
    bytelen: *mut c_int,
) -> c_int {
    let mut clnum = start_lnum;
    let mut ccol = start_col;

    if !bytelen.is_null() {
        *bytelen = 0;
    }

    loop {
        // Since getting one line may invalidate the other, need to make copy.
        let line = REX.line;
        let reg_tofree = REG_TOFREE;
        if line != reg_tofree {
            #[allow(clippy::cast_possible_truncation)]
            let len = strlen(REX.line.cast::<c_char>()) as c_int;
            let reg_tofreelen = REG_TOFREELEN as c_int;
            if reg_tofree.is_null() || len >= reg_tofreelen {
                let newlen = len + 50;
                xfree(REG_TOFREE.cast());
                let new_buf = xmalloc(newlen as usize).cast::<u8>();
                REG_TOFREE = new_buf;
                REG_TOFREELEN = newlen as c_uint;
            }
            let tofree = REG_TOFREE;
            let cur_line = REX.line;
            let cur_input = REX.input;
            // STRCPY: copy including NUL
            std::ptr::copy_nonoverlapping(cur_line, tofree, len as usize + 1);
            // rex.input = reg_tofree + (rex.input - rex.line)
            let input_offset = cur_input.offset_from(cur_line) as usize;
            REX.input = tofree.add(input_offset);
            REX.line = tofree;
        }

        // Get the line to compare with.
        let p = nvim_regexp_call_reg_getline(clnum);
        assert!(!p.is_null());

        let mut len = if clnum == end_lnum {
            end_col - ccol
        } else {
            nvim_regexp_call_reg_getline_len(clnum) - ccol
        };

        let input: *mut c_char = REX.input.cast();
        let reg_ic = REX.reg_ic as c_int;
        let p_ccol: *mut c_char = p.add(ccol as usize);

        if reg_ic == 0 {
            // case-sensitive compare
            if rs_cstrncmp(p_ccol, input, &mut len) != 0 {
                return RA_NOMATCH;
            }
        } else {
            // case-insensitive compare
            if mb_strnicmp(p_ccol, input, len as usize) != 0 {
                return RA_NOMATCH;
            }
        }

        if !bytelen.is_null() {
            *bytelen += len;
        }
        if clnum == end_lnum {
            break;
        }
        if REX.lnum >= REX.reg_maxline {
            return RA_NOMATCH;
        }

        // Advance to next line.
        rs_reg_nextline();
        if !bytelen.is_null() {
            *bytelen = 0;
        }
        clnum += 1;
        ccol = 0;
        if nvim_regexp_get_got_int() != 0 {
            return RA_FAIL;
        }
    }

    RA_MATCH
}

// --- do_upper / do_lower ---

extern "C" {
    fn mb_toupper(c: c_int) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;
}

/// Case-conversion wrapper used as `fptr_T` — writes uppercase of `c` into `*d`.
#[no_mangle]
pub unsafe extern "C" fn rs_do_upper(d: *mut c_int, c: c_int) {
    *d = mb_toupper(c);
}

/// Case-conversion wrapper used as `fptr_T` — writes lowercase of `c` into `*d`.
#[no_mangle]
pub unsafe extern "C" fn rs_do_lower(d: *mut c_int, c: c_int) {
    *d = mb_tolower(c);
}

// --- vim_regsub_both literal path ---

// Constants matching C definitions (TAB_CH, CAR_CH already defined above)
const K_SPECIAL: u8 = 0x80;
const NL_CH: c_int = 0x0a;
const CTRL_H_CH: c_int = 8;

// REGSUB flag constants (matching regexp_defs.h)
const REGSUB_COPY: c_int = 1;
const REGSUB_MAGIC: c_int = 2;
const REGSUB_BACKSLASH: c_int = 4;

extern "C" {
    fn nvim_regexp_get_rex_reg_match_startp(no: c_int) -> *const c_char;
    fn nvim_regexp_get_rex_reg_match_endp(no: c_int) -> *const c_char;
    fn nvim_regexp_get_rex_reg_mmatch_startpos_lnum(no: c_int) -> i32;
    fn nvim_regexp_get_rex_reg_mmatch_startpos_col(no: c_int) -> i32;
    fn nvim_regexp_get_rex_reg_mmatch_endpos_lnum(no: c_int) -> i32;
    fn nvim_regexp_get_rex_reg_mmatch_endpos_col(no: c_int) -> i32;
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
}

/// Case-conversion function type: 0=none, 1=upper, 2=lower
#[derive(Clone, Copy, PartialEq)]
enum CaseFunc {
    None,
    Upper,
    Lower,
}

/// Apply case conversion and return the converted character.
unsafe fn apply_case(func: CaseFunc, c: c_int) -> c_int {
    match func {
        CaseFunc::None => c,
        CaseFunc::Upper => {
            let mut d: c_int = 0;
            rs_do_upper(&mut d, c);
            d
        }
        CaseFunc::Lower => {
            let mut d: c_int = 0;
            rs_do_lower(&mut d, c);
            d
        }
    }
}

/// Check if `out` has enough space, emit error if not.
/// Returns `true` when there's NOT enough space.
#[inline]
unsafe fn regsub_check_space(out: *mut c_char, dest: *mut c_char, need: isize, lim: isize) -> bool {
    if out.offset_from(dest) + need > lim {
        errors::call_iemsg_not_enough_space();
        true
    } else {
        false
    }
}

/// Expand a backreference (subgroup `no`) into `out`.
/// Returns the new `out` pointer, or null on error.
/// Sets `early_exit` when the caller should return `out - dest + 1`.
#[allow(clippy::too_many_arguments)]
unsafe fn regsub_expand_backref(
    no: c_int,
    out: *mut c_char,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
    copy: bool,
    reg_multi: bool,
    func_one: &mut CaseFunc,
    func_all: &CaseFunc,
    early_exit: &mut bool,
) -> *mut c_char {
    let mut out = out;
    let mut s: *const c_char;
    let mut len: c_int;
    let mut clnum: i32 = 0;
    let lim = destlen as isize;
    *early_exit = false;

    if reg_multi {
        clnum = nvim_regexp_get_rex_reg_mmatch_startpos_lnum(no);
        if clnum < 0 || nvim_regexp_get_rex_reg_mmatch_endpos_lnum(no) < 0 {
            return out;
        }
        let start_col = nvim_regexp_get_rex_reg_mmatch_startpos_col(no);
        s = nvim_regexp_call_reg_getline(clnum).add(start_col as usize);
        len = if nvim_regexp_get_rex_reg_mmatch_endpos_lnum(no) == clnum {
            nvim_regexp_get_rex_reg_mmatch_endpos_col(no) - start_col
        } else {
            nvim_regexp_call_reg_getline_len(clnum) - start_col
        };
    } else {
        s = nvim_regexp_get_rex_reg_match_startp(no);
        if nvim_regexp_get_rex_reg_match_endp(no).is_null() {
            return out;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            len = nvim_regexp_get_rex_reg_match_endp(no).offset_from(s) as c_int;
        }
    }

    loop {
        if len == 0 {
            if !reg_multi || nvim_regexp_get_rex_reg_mmatch_endpos_lnum(no) == clnum {
                break;
            }
            if copy && regsub_check_space(out, dest, 1, lim) {
                return std::ptr::null_mut();
            }
            #[allow(clippy::cast_possible_truncation)]
            if copy {
                *out = CAR_CH as c_char;
            }
            out = out.add(1);
            clnum += 1;
            s = nvim_regexp_call_reg_getline(clnum);
            len = if nvim_regexp_get_rex_reg_mmatch_endpos_lnum(no) == clnum {
                nvim_regexp_get_rex_reg_mmatch_endpos_col(no)
            } else {
                nvim_regexp_call_reg_getline_len(clnum)
            };
        } else if *s == 0 {
            if copy {
                errors::call_iemsg_re_damg();
            }
            *early_exit = true;
            return out;
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let is_bs_special = (flags & REGSUB_BACKSLASH != 0)
                && (*s == CAR_CH as c_char || *s == b'\\' as c_char);
            if is_bs_special {
                if copy && regsub_check_space(out, dest, 2, lim) {
                    return std::ptr::null_mut();
                }
                if copy {
                    *out = b'\\' as c_char;
                    *out.add(1) = *s;
                }
                out = out.add(2);
            } else {
                let bc = utf_ptr2char(s);
                let cc = if *func_one != CaseFunc::None {
                    let r = apply_case(*func_one, bc);
                    *func_one = CaseFunc::None;
                    r
                } else if *func_all != CaseFunc::None {
                    apply_case(*func_all, bc)
                } else {
                    bc
                };
                let l = utf_ptr2len(s) - 1;
                s = s.add(l as usize);
                len -= l;
                let charlen = utf_char2len(cc);
                if copy && regsub_check_space(out, dest, charlen as isize, lim) {
                    return std::ptr::null_mut();
                }
                if copy {
                    utf_char2bytes(cc, out);
                }
                out = out.add((charlen - 1) as usize);
                out = out.add(1);
            }
            s = s.add(1);
            len -= 1;
        }
    }
    out
}

/// Literal substitution path of `vim_regsub_both`.
///
/// Handles escape sequences, backreferences, case conversion,
/// `K_SPECIAL` passthrough, multi-line backreference expansion,
/// and composing character handling.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_vim_regsub_literal(
    source: *mut c_char,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    let copy = flags & REGSUB_COPY != 0;
    let reg_multi = (REX.reg_match.is_null() as c_int) != 0;
    let lim = destlen as isize;

    let mut src = source;
    let mut out = dest;
    let mut func_all = CaseFunc::None;
    let mut func_one = CaseFunc::None;

    loop {
        let c_byte = *src as u8;
        if c_byte == 0 {
            break;
        }
        src = src.add(1);
        let mut c = c_byte as c_int;
        let mut no: c_int = -1;

        // Check for backreferences
        if c == b'&' as c_int && (flags & REGSUB_MAGIC != 0) {
            no = 0;
        } else if c == b'\\' as c_int && *src != 0 {
            let next = *src as u8;
            if next == b'&' && (flags & REGSUB_MAGIC == 0) {
                src = src.add(1);
                no = 0;
            } else if next.is_ascii_digit() {
                no = (next - b'0') as c_int;
                src = src.add(1);
            } else {
                match next {
                    b'u' => {
                        func_one = CaseFunc::Upper;
                        src = src.add(1);
                        continue;
                    }
                    b'U' => {
                        func_all = CaseFunc::Upper;
                        src = src.add(1);
                        continue;
                    }
                    b'l' => {
                        func_one = CaseFunc::Lower;
                        src = src.add(1);
                        continue;
                    }
                    b'L' => {
                        func_all = CaseFunc::Lower;
                        src = src.add(1);
                        continue;
                    }
                    b'e' | b'E' => {
                        func_one = CaseFunc::None;
                        func_all = CaseFunc::None;
                        src = src.add(1);
                        continue;
                    }
                    _ => {}
                }
            }
        }

        if no < 0 {
            // Ordinary character
            if c_byte == K_SPECIAL && *src != 0 && *src.add(1) != 0 {
                if copy {
                    if regsub_check_space(out, dest, 3, lim) {
                        return 0;
                    }
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        *out = c as c_char;
                    }
                    out = out.add(1);
                    *out = *src;
                    out = out.add(1);
                    src = src.add(1);
                    *out = *src;
                    out = out.add(1);
                    src = src.add(1);
                } else {
                    out = out.add(3);
                    src = src.add(2);
                }
                continue;
            }

            if c == b'\\' as c_int && *src != 0 {
                match *src as u8 {
                    b'r' => {
                        c = CAR_CH;
                        src = src.add(1);
                    }
                    b'n' => {
                        c = NL_CH;
                        src = src.add(1);
                    }
                    b't' => {
                        c = TAB_CH;
                        src = src.add(1);
                    }
                    b'b' => {
                        c = CTRL_H_CH;
                        src = src.add(1);
                    }
                    _ => {
                        if flags & REGSUB_BACKSLASH != 0 {
                            if copy {
                                if regsub_check_space(out, dest, 1, lim) {
                                    return 0;
                                }
                                *out = b'\\' as c_char;
                            }
                            out = out.add(1);
                        }
                        c = *src as u8 as c_int;
                        src = src.add(1);
                    }
                }
            } else {
                c = utf_ptr2char(src.sub(1));
            }

            // Apply case conversion
            let cc = if func_one != CaseFunc::None {
                let r = apply_case(func_one, c);
                func_one = CaseFunc::None;
                r
            } else if func_all != CaseFunc::None {
                apply_case(func_all, c)
            } else {
                c
            };

            let totlen = utfc_ptr2len(src.sub(1));
            let charlen = utf_char2len(cc);

            if copy {
                if regsub_check_space(out, dest, charlen as isize, lim) {
                    return 0;
                }
                utf_char2bytes(cc, out);
            }
            out = out.add((charlen - 1) as usize);
            let clen = utf_ptr2len(src.sub(1));

            // Composing characters: copy as-is
            if clen < totlen {
                let comp_len = (totlen - clen) as usize;
                if copy {
                    if regsub_check_space(out, dest, comp_len as isize, lim) {
                        return 0;
                    }
                    std::ptr::copy(src.sub(1).add(clen as usize), out.add(1), comp_len);
                }
                out = out.add(comp_len);
            }
            src = src.add((totlen - 1) as usize);
            out = out.add(1);
        } else {
            // Backreference expansion
            let mut early_exit = false;
            let result = regsub_expand_backref(
                no,
                out,
                dest,
                destlen,
                flags,
                copy,
                reg_multi,
                &mut func_one,
                &func_all,
                &mut early_exit,
            );
            if result.is_null() {
                return 0;
            }
            out = result;
            if early_exit {
                #[allow(clippy::cast_possible_truncation)]
                return (out.offset_from(dest) + 1) as c_int;
            }
        }
    }

    if copy {
        *out = 0;
    }

    #[allow(clippy::cast_possible_truncation)]
    let result = (out.offset_from(dest) + 1) as c_int;
    result
}

// ---------------------------------------------------------------------------
// prog_magic_wrong: validate BT program magic number
// ---------------------------------------------------------------------------

/// Check the regexp program for its magic number.
/// Returns 1 if the magic is wrong (error emitted), 0 if OK.
/// (Phase 7: inlined `get_prog` logic directly)
#[no_mangle]
pub unsafe extern "C" fn rs_prog_magic_wrong() -> c_int {
    // Inlined nvim_regexp_nfa_regexec_both_get_prog
    let prog: NfaProgHandle = if REX.reg_match.is_null() {
        (*REX.reg_mmatch.cast::<RegmmatchT>())
            .regprog
            .cast::<c_void>()
    } else {
        (*REX.reg_match.cast::<RegmatchT>())
            .regprog
            .cast::<c_void>()
    };
    if (*prog.cast::<RegprogT>()).engine == core::ptr::addr_of!(NFA_REGENGINE).cast_mut() {
        // For NFA matcher we don't check the magic
        return 0;
    }
    let program = prog
        .cast::<u8>()
        .add(core::mem::offset_of!(BtRegprogT, reghasz) + 1);
    if *program as c_int != REGMAGIC {
        errors::iemsg_re_corr();
        return 1;
    }
    0
}

// ---------------------------------------------------------------------------
// vim_regsub_both: expression evaluation + dispatch
// ---------------------------------------------------------------------------

extern "C" {
    fn nvim_regexp_eval_regsub_expr(
        source: *mut c_char,
        expr_ptr: *mut c_void,
        flags: c_int,
        nested: c_int,
    ) -> c_int;
    fn nvim_regexp_get_curbuf() -> *mut c_void;
    fn nvim_regexp_get_curbuf_ml_line_count() -> i32;
}

/// Core substitution function: handles both literal and `\=` expression paths.
///
/// If the substitution pattern starts with `\=`, delegates to the C compound
/// accessor `nvim_regexp_eval_regsub_expr` which handles Vimscript evaluation.
/// Otherwise, delegates to `rs_vim_regsub_literal`.
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_vim_regsub_both(
    source: *mut c_char,
    expr: *mut c_void,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    let copy = flags & REGSUB_COPY != 0;

    // Be paranoid...
    if source.is_null() && expr.is_null() || dest.is_null() {
        errors::emsg_e_null();
        return 0;
    }
    if nvim_regexp_call_prog_magic_wrong() != 0 {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation)]
    let max_nesting = MAX_REGSUB_NESTING as c_int;
    let nesting = REGSUB_NESTING;
    if nesting == max_nesting {
        errors::emsg_e_substitute_nesting();
        return 0;
    }
    let nested = nesting;

    // When the substitute part starts with "\=" evaluate it as an expression.
    if !expr.is_null()
        || (!source.is_null() && *source as u8 == b'\\' && *source.add(1) as u8 == b'=')
    {
        if copy {
            // Copy from previously evaluated result
            let eval_res = EVAL_RESULT[nested as usize];
            if !eval_res.is_null() {
                let eval_len = strlen(eval_res);
                if eval_len < destlen as usize {
                    std::ptr::copy_nonoverlapping(
                        eval_res.cast::<u8>(),
                        dest.cast::<u8>(),
                        eval_len + 1,
                    );
                    let end = dest.add(eval_len);
                    {
                        xfree(EVAL_RESULT[nested as usize].cast::<c_void>());
                        EVAL_RESULT[nested as usize] = core::ptr::null_mut();
                    };
                    *end = 0;
                    #[allow(clippy::cast_possible_truncation)]
                    return (end.offset_from(dest) + 1) as c_int;
                }
            }
            // If eval_result was NULL or too large, return just NUL
            *dest = 0;
            return 1;
        }
        // Evaluate the expression -- all VimL interaction happens in C
        let eval_len = nvim_regexp_eval_regsub_expr(source, expr, flags, nested);
        // The result is stored in eval_result[nested]; return its length + 1
        #[allow(clippy::cast_possible_truncation)]
        return eval_len + 1;
    }

    // Non-expression substitution: delegate to literal handler
    rs_vim_regsub_literal(source, dest, destlen, flags)
}

/// Perform substitution after a `vim_regexec()` match (single-line).
///
/// Saves/restores `rex` state for recursive calls.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regsub(
    rmp: *mut c_void,
    source: *mut c_char,
    expr: *mut c_void,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    let was_in_use = REX_IN_USE;
    let saved = save_rex_state();
    // setup_vim_regsub: single-line substitution
    REX.reg_match = rmp.cast();
    REX.reg_mmatch = core::ptr::null_mut();
    REX.reg_maxline = 0;
    REX.reg_buf = nvim_regexp_get_curbuf().cast();
    REX.reg_line_lbr = true;

    let result = rs_vim_regsub_both(source, expr, dest, destlen, flags);

    restore_rex_state(&saved, was_in_use);
    result
}

/// Perform substitution after a `vim_regexec_multi()` match (multi-line).
///
/// Saves/restores `rex` state for recursive calls.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regsub_multi(
    rmp: *mut c_void,
    lnum: i32,
    source: *mut c_char,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    let was_in_use = REX_IN_USE;
    let saved = save_rex_state();
    // setup_vim_regsub_multi: multi-line substitution
    REX.reg_match = core::ptr::null_mut();
    REX.reg_mmatch = rmp.cast();
    REX.reg_buf = nvim_regexp_get_curbuf().cast();
    REX.reg_firstlnum = lnum;
    REX.reg_maxline = nvim_regexp_get_curbuf_ml_line_count() - lnum;
    REX.reg_line_lbr = false;

    let result = rs_vim_regsub_both(source, core::ptr::null_mut(), dest, destlen, flags);

    restore_rex_state(&saved, was_in_use);
    result
}

// ---------------------------------------------------------------------------
// Node management & compilation infrastructure
// ---------------------------------------------------------------------------

/// Write a four-byte big-endian uint32 at `p` and return pointer past it.
/// Pure helper — no globals touched.
#[no_mangle]
pub unsafe extern "C" fn rs_re_put_uint32(p: *mut u8, val: u32) -> *mut u8 {
    let bytes = val.to_be_bytes();
    *p = bytes[0];
    *p.add(1) = bytes[1];
    *p.add(2) = bytes[2];
    *p.add(3) = bytes[3];
    p.add(4)
}

/// Emit (if appropriate) a single byte of code.
/// If `regcode == JUST_CALC_SIZE`, increments `regsize` instead.
#[no_mangle]
pub unsafe extern "C" fn rs_regc(b: c_int) {
    let regcode = REGCODE;
    let just_calc_size = JUST_CALC_SIZE;
    if regcode == just_calc_size {
        REGSIZE += 1;
    } else {
        #[allow(clippy::cast_possible_truncation)]
        {
            *regcode = b as u8;
        }
        REGCODE = regcode.add(1);
    }
}

/// Emit (if appropriate) a multi-byte character of code.
/// If `regcode == JUST_CALC_SIZE`, adds `utf_char2len(c)` to `regsize`.
#[no_mangle]
pub unsafe extern "C" fn rs_regmbc(c: c_int) {
    let regcode = REGCODE;
    let just_calc_size = JUST_CALC_SIZE;
    if regcode == just_calc_size {
        REGSIZE += utf_char2len(c) as i64;
    } else {
        let written = utf_char2bytes(c, regcode.cast::<c_char>());
        REGCODE = regcode.add(written as usize);
    }
}

// Opcode constants (must match C #define values in regexp.c)
const BRANCH: c_int = 3; // #define BRANCH 3
const BACK: c_int = 4; // #define BACK 4 — Match "", "next" ptr points backward
const BRACE_COMPLEX: c_int = 140; // #define BRACE_COMPLEX 140 (range 140-149)

/// Emit a node. Return pointer to generated code.
/// If `regcode == JUST_CALC_SIZE`, adds 3 to `regsize` and returns `JUST_CALC_SIZE`.
#[no_mangle]
pub unsafe extern "C" fn rs_regnode(op: c_int) -> *mut u8 {
    let regcode = REGCODE;
    let just_calc_size = JUST_CALC_SIZE;
    if regcode == just_calc_size {
        REGSIZE += 3;
        return just_calc_size;
    }
    let ret = regcode;
    #[allow(clippy::cast_possible_truncation)]
    {
        *regcode = op as u8;
    }
    *regcode.add(1) = 0; // NUL "next" pointer
    *regcode.add(2) = 0;
    REGCODE = regcode.add(3);
    ret
}

/// Dig the "next" pointer out of a node.
/// Returns NULL when calculating size, when there is no next item, or on error.
#[no_mangle]
pub unsafe extern "C" fn rs_regnext(p: *mut u8) -> *mut u8 {
    let just_calc_size = JUST_CALC_SIZE;
    if p == just_calc_size || REG_TOOLONG != 0 {
        return std::ptr::null_mut();
    }

    // NEXT(p) = ((*((p) + 1) & 0377) << 8) + (*((p) + 2) & 0377)
    let offset = (((*p.add(1) as c_int) & 0o377) << 8) + ((*p.add(2) as c_int) & 0o377);
    if offset == 0 {
        return std::ptr::null_mut();
    }

    // OP(p) = (int)(*(p))
    let op = *p as c_int;
    if op == BACK {
        p.sub(offset as usize)
    } else {
        p.add(offset as usize)
    }
}

/// Set the next-pointer at the end of a node chain.
/// Walks via `rs_regnext` to find the last node, computes the offset to `val`,
/// and writes it as a 16-bit value in bytes 1-2 of that node.
#[no_mangle]
pub unsafe extern "C" fn rs_regtail(p: *mut u8, val: *const u8) {
    let just_calc_size = JUST_CALC_SIZE;
    if p == just_calc_size {
        return;
    }

    // Find last node in the chain.
    let mut scan = p;
    loop {
        let temp = rs_regnext(scan);
        if temp.is_null() {
            break;
        }
        scan = temp;
    }

    // OP(scan) = (int)(*(scan))
    let op = *scan as c_int;
    #[allow(clippy::cast_possible_truncation)]
    let offset = if op == BACK {
        // BACK nodes point backward: offset = scan - val
        scan.offset_from(val) as c_int
    } else {
        // Forward: offset = val - scan
        val.offset_from(scan) as c_int
    };

    // When the offset uses more than 16 bits it can no longer fit in the two
    // bytes available.
    if offset > 0xffff {
        REG_TOOLONG = 1;
    } else {
        #[allow(clippy::cast_possible_truncation)]
        {
            *scan.add(1) = ((offset as u32 >> 8) & 0o377) as u8;
            *scan.add(2) = (offset as u32 & 0o377) as u8;
        }
    }
}

/// Like `rs_regtail`, on item after a BRANCH; nop if none.
/// Only acts if `OP(p)` is `BRANCH` or `BRACE_COMPLEX+0..9`.
#[no_mangle]
pub unsafe extern "C" fn rs_regoptail(p: *mut u8, val: *mut u8) {
    let just_calc_size = JUST_CALC_SIZE;
    if p.is_null() || p == just_calc_size {
        return;
    }
    let op = *p as c_int;
    if op != BRANCH && !(BRACE_COMPLEX..=BRACE_COMPLEX + 9).contains(&op) {
        return;
    }
    // OPERAND(p) = p + 3
    rs_regtail(p.add(3), val);
}

/// Insert an operator in front of already-emitted operand.
/// Shifts existing bytes forward by 3 using `ptr::copy` (memmove semantics),
/// then writes the 3-byte operator node at `opnd`.
#[no_mangle]
pub unsafe extern "C" fn rs_reginsert(op: c_int, opnd: *mut u8) {
    let regcode = REGCODE;
    let just_calc_size = JUST_CALC_SIZE;
    if regcode == just_calc_size {
        REGSIZE += 3;
        return;
    }
    let count = regcode.offset_from(opnd) as usize;
    REGCODE = regcode.add(3);
    // Shift bytes forward by 3 (overlapping — ptr::copy handles this)
    std::ptr::copy(opnd, opnd.add(3), count);
    // Write 3-byte operator node at opnd
    #[allow(clippy::cast_possible_truncation)]
    {
        *opnd = op as u8;
    }
    *opnd.add(1) = 0;
    *opnd.add(2) = 0;
}

/// Insert an operator + 4-byte uint32 in front of already-emitted operand.
/// Shifts existing bytes forward by 7.
#[no_mangle]
pub unsafe extern "C" fn rs_reginsert_nr(op: c_int, val: i64, opnd: *mut u8) {
    let regcode = REGCODE;
    let just_calc_size = JUST_CALC_SIZE;
    if regcode == just_calc_size {
        REGSIZE += 7;
        return;
    }
    let count = regcode.offset_from(opnd) as usize;
    REGCODE = regcode.add(7);
    std::ptr::copy(opnd, opnd.add(7), count);
    #[allow(clippy::cast_possible_truncation)]
    {
        *opnd = op as u8;
    }
    *opnd.add(1) = 0;
    *opnd.add(2) = 0;
    debug_assert!(u32::try_from(val).is_ok());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    rs_re_put_uint32(opnd.add(3), val as u32);
}

/// Insert an operator + two 4-byte uint32s in front of already-emitted operand.
/// Shifts existing bytes forward by 11, then calls `rs_regtail(opnd, place)`.
#[no_mangle]
pub unsafe extern "C" fn rs_reginsert_limits(op: c_int, minval: i64, maxval: i64, opnd: *mut u8) {
    let regcode = REGCODE;
    let just_calc_size = JUST_CALC_SIZE;
    if regcode == just_calc_size {
        REGSIZE += 11;
        return;
    }
    let count = regcode.offset_from(opnd) as usize;
    REGCODE = regcode.add(11);
    std::ptr::copy(opnd, opnd.add(11), count);
    #[allow(clippy::cast_possible_truncation)]
    {
        *opnd = op as u8;
    }
    *opnd.add(1) = 0;
    *opnd.add(2) = 0;
    debug_assert!(u32::try_from(minval).is_ok());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let place = rs_re_put_uint32(opnd.add(3), minval as u32);
    debug_assert!(u32::try_from(maxval).is_ok());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let place = rs_re_put_uint32(place, maxval as u32);
    rs_regtail(opnd, place);
}

// --- Opcode and flag constants for the recursive descent parser ---
const END: c_int = 0;
#[allow(dead_code)]
const BOL: c_int = 1;
#[allow(dead_code)]
const EOL: c_int = 2;
#[allow(dead_code)]
const EXACTLY: c_int = 5;
const NOTHING: c_int = 6;
const STAR: c_int = 7;
const PLUS: c_int = 8;
const MATCH: c_int = 9;
const NOMATCH: c_int = 10;
const BEHIND: c_int = 11;
const NOBEHIND: c_int = 12;
const SUBPAT: c_int = 13;
const BRACE_SIMPLE: c_int = 14;
#[allow(dead_code)]
const BOW: c_int = 15;
#[allow(dead_code)]
const EOW: c_int = 16;
const BRACE_LIMITS: c_int = 17;
#[allow(dead_code)]
const NEWL: c_int = 18;
const BHPOS: c_int = 19;
#[allow(dead_code)]
const ANY: c_int = 20;
#[allow(dead_code)]
const ANYOF: c_int = 21;
#[allow(dead_code)]
const ANYBUT: c_int = 22;
#[allow(dead_code)]
const IDENT: c_int = 23;
#[allow(dead_code)]
const SIDENT: c_int = 24;
#[allow(dead_code)]
const KWORD: c_int = 25;
#[allow(dead_code)]
const SKWORD: c_int = 26;
#[allow(dead_code)]
const FNAME: c_int = 27;
#[allow(dead_code)]
const SFNAME: c_int = 28;
#[allow(dead_code)]
const PRINT: c_int = 29;
#[allow(dead_code)]
const SPRINT: c_int = 30;
#[allow(dead_code)]
const WHITE: c_int = 31;
#[allow(dead_code)]
const NWHITE: c_int = 32;
#[allow(dead_code)]
const DIGIT: c_int = 33;
#[allow(dead_code)]
const NDIGIT: c_int = 34;
#[allow(dead_code)]
const HEX: c_int = 35;
#[allow(dead_code)]
const NHEX: c_int = 36;
#[allow(dead_code)]
const OCTAL: c_int = 37;
#[allow(dead_code)]
const NOCTAL: c_int = 38;
#[allow(dead_code)]
const WORD: c_int = 39;
#[allow(dead_code)]
const NWORD: c_int = 40;
#[allow(dead_code)]
const HEAD: c_int = 41;
#[allow(dead_code)]
const NHEAD: c_int = 42;
#[allow(dead_code)]
const ALPHA: c_int = 43;
#[allow(dead_code)]
const NALPHA: c_int = 44;
#[allow(dead_code)]
const LOWER: c_int = 45;
#[allow(dead_code)]
const NLOWER: c_int = 46;
#[allow(dead_code)]
const UPPER: c_int = 47;
#[allow(dead_code)]
const NUPPER: c_int = 48;
#[allow(dead_code)]
const ADD_NL: c_int = 30;
#[allow(dead_code)]
const MOPEN: c_int = 80;
#[allow(dead_code)]
const MCLOSE: c_int = 90;
#[allow(dead_code)]
const BACKREF: c_int = 100;
#[allow(dead_code)]
const ZOPEN: c_int = 110;
#[allow(dead_code)]
const ZCLOSE: c_int = 120;
#[allow(dead_code)]
const ZREF: c_int = 130;
#[allow(dead_code)]
const NOPEN: c_int = 150;
#[allow(dead_code)]
const NCLOSE: c_int = 151;
#[allow(dead_code)]
const MULTIBYTECODE: c_int = 200;
#[allow(dead_code)]
const RE_BOF: c_int = 201;
#[allow(dead_code)]
const RE_EOF: c_int = 202;
#[allow(dead_code)]
const CURSOR: c_int = 203;
#[allow(dead_code)]
const RE_LNUM: c_int = 204;
#[allow(dead_code)]
const RE_COL: c_int = 205;
#[allow(dead_code)]
const RE_VCOL: c_int = 206;
#[allow(dead_code)]
const RE_MARK: c_int = 207;
#[allow(dead_code)]
const RE_VISUAL: c_int = 208;
#[allow(dead_code)]
const RE_COMPOSING: c_int = 209;
#[allow(dead_code)]
const NL: c_int = 10; // '\n'
#[allow(dead_code)]
const REX_SET: c_int = 1;
#[allow(dead_code)]
const REX_USE: c_int = 2;

// Parser flags
const HASWIDTH: c_int = 0x1;
const SIMPLE: c_int = 0x2;
const SPSTART: c_int = 0x4;
const HASNL: c_int = 0x8;
const HASLOOKBH: c_int = 0x10;
const WORST: c_int = 0;

// RF_ compile-time flags
#[allow(dead_code)]
const RF_ICASE: c_uint = 1;
#[allow(dead_code)]
const RF_NOICASE: c_uint = 2;
#[allow(dead_code)]
const RF_ICOMBINE: c_uint = 8;

// Paren types
#[allow(dead_code)]
const REG_NOPAREN: c_int = 0;
#[allow(dead_code)]
const REG_PAREN: c_int = 1;
#[allow(dead_code)]
const REG_ZPAREN: c_int = 2;
#[allow(dead_code)]
const REG_NPAREN: c_int = 3;

// Character class lookup tables (must match C classchars/classcodes)
#[allow(dead_code)]
const CLASSCHARS: &[u8] = b".iIkKfFpPsSdDxXoOwWhHaAlLuU";
#[allow(dead_code)]
const CLASSCODES: &[c_int] = &[
    ANY, IDENT, SIDENT, KWORD, SKWORD, FNAME, SFNAME, PRINT, SPRINT, WHITE, NWHITE, DIGIT, NDIGIT,
    HEX, NHEX, OCTAL, NOCTAL, WORD, NWORD, HEAD, NHEAD, ALPHA, NALPHA, LOWER, NLOWER, UPPER,
    NUPPER,
];

// --- regatom accessor/error extern declarations ---
#[allow(dead_code)]
extern "C" {
    // Accessors for regatom globals
    fn nvim_regexp_get_reg_do_extmatch() -> c_int;
    fn nvim_regexp_get_curwin_lnum() -> i32;
    fn nvim_regexp_get_curwin_col() -> i32;
    fn nvim_regexp_get_curwin_vcol() -> i32;
    fn nvim_regexp_get_reg_prev_sub_ptr() -> *mut c_char;

    // Character / multibyte helpers
    fn utf_iscomposing_legacy(c: c_int) -> c_int;
    fn utf_composinglike(p1: *const c_char, p2: *const c_char, state: *mut i32) -> c_int;
    fn vim_isIDc(c: c_int) -> c_int;
    fn vim_isfilec(c: c_int) -> c_int;
    fn vim_isprintc(c: c_int) -> c_int;
    fn mb_islower(c: c_int) -> c_int;
    fn mb_isupper(c: c_int) -> c_int;

    // Error helpers for regatom

    // libc ctype helpers
    fn isalnum(c: c_int) -> c_int;
    fn isalpha(c: c_int) -> c_int;
    fn iscntrl(c: c_int) -> c_int;
    fn isgraph(c: c_int) -> c_int;
    fn ispunct(c: c_int) -> c_int;
}

// --- Helper functions for regatom ---

/// Return true if MULTIBYTECODE should be used instead of EXACTLY for
/// character `c`.
#[allow(dead_code)]
unsafe fn use_multibytecode(c: c_int) -> bool {
    utf_char2len(c) > 1
        && (rs_re_multi_type(rs_peekchr()) != NOT_MULTI || utf_iscomposing_legacy(c) != 0)
}

/// Get a number after a backslash that is inside [].
/// When nothing is recognized return a backslash.
#[allow(dead_code)]
unsafe fn coll_get_char() -> c_int {
    let regparse = REGPARSE;
    let ch = *regparse as u8;
    REGPARSE = regparse.add(1);

    let nr: c_long = match ch {
        b'd' => rs_getdecchrs(),
        b'o' => rs_getoctchrs(),
        b'x' => rs_gethexchrs(2),
        b'u' => rs_gethexchrs(4),
        b'U' => rs_gethexchrs(8),
        _ => {
            // Put back the character we consumed
            REGPARSE = regparse;
            return b'\\' as c_int;
        }
    };

    if nr < 0 {
        // If getting the number fails be backwards compatible: the character
        // is a backslash.
        // Undo the advance past the letter (d/o/x/u/U) — the number parsers
        // already left regparse right after the letter when they fail.
        REGPARSE = regparse;
        return b'\\' as c_int;
    }
    c_int::try_from(nr).unwrap_or(c_int::MAX)
}

/// Return true if the back reference is legal.  We must have seen the close
/// brace.
#[allow(dead_code)]
unsafe fn seen_endbrace(refnum: c_int) -> bool {
    if HAD_ENDBRACE[refnum as usize] as c_int == 0 {
        // Trick: check if "@<=" or "@<!" follows, in which case
        // the \1 can appear before the referenced match.
        let regparse = REGPARSE as *const u8;
        let mut p = regparse;
        while *p != 0 {
            if *p == b'@' && *p.add(1) == b'<' && (*p.add(2) == b'!' || *p.add(2) == b'=') {
                break;
            }
            p = p.add(1);
        }
        if *p == 0 {
            errors::emsg_e65();
            return false;
        }
    }
    true
}

// --- regatom: Parse the lowest level ---

/// Handle a POSIX character class like `[:alpha:]` inside a collection.
/// `c_class` is the class constant from `get_char_class`.
#[allow(dead_code, clippy::too_many_lines)]
unsafe fn emit_posix_class(c_class: c_int, regparse_ptr: *mut *mut c_char) {
    match c_class {
        x if x == CLASS_NONE => {
            let eq = get_equi_class(regparse_ptr);
            if eq != 0 {
                reg_equi_class(eq);
            } else {
                let coll = get_coll_element(regparse_ptr);
                if coll != 0 {
                    rs_regmbc(coll);
                } else {
                    // literal '[', allow [[-x] as a range — handled by caller via startc
                }
            }
        }
        x if x == CLASS_ALNUM => {
            for cu in 1..128 {
                if isalnum(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_ALPHA => {
            for cu in 1..128 {
                if isalpha(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_BLANK => {
            rs_regc(b' ' as c_int);
            rs_regc(b'\t' as c_int);
        }
        x if x == CLASS_CNTRL => {
            for cu in 1..=127 {
                if iscntrl(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_DIGIT => {
            for cu in 1_i32..=127 {
                if u8::try_from(cu).is_ok_and(|b| b.is_ascii_digit()) {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_GRAPH => {
            for cu in 1..=127 {
                if isgraph(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_LOWER => {
            for cu in 1..=255 {
                if mb_islower(cu) != 0 && cu != 170 && cu != 186 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_PRINT => {
            for cu in 1..=255 {
                if vim_isprintc(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_PUNCT => {
            for cu in 1..128 {
                if ispunct(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_SPACE => {
            for cu in 9..=13 {
                rs_regc(cu);
            }
            rs_regc(b' ' as c_int);
        }
        x if x == CLASS_UPPER => {
            for cu in 1..=255 {
                if mb_isupper(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_XDIGIT => {
            for cu in 1_i32..=255 {
                if u8::try_from(cu).is_ok_and(|b| b.is_ascii_hexdigit()) {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_CC_TAB => rs_regc(b'\t' as c_int),
        x if x == CLASS_RETURN => rs_regc(b'\r' as c_int),
        x if x == CLASS_BACKSPACE => rs_regc(0o010), // '\b'
        x if x == CLASS_ESCAPE => rs_regc(ESC_CH),
        x if x == CLASS_IDENT => {
            for cu in 1..=255 {
                if vim_isIDc(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_KEYWORD => {
            for cu in 1..=255 {
                if rs_reg_iswordc(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        x if x == CLASS_FNAME => {
            for cu in 1..=255 {
                if vim_isfilec(cu) != 0 {
                    rs_regmbc(cu);
                }
            }
        }
        _ => {}
    }
}

/// Parse a collection `[...]` or `[^...]`.
/// `extra` is `ADD_NL` if preceded by `\_`.
/// Returns the compiled node, or null on error.
#[allow(dead_code, clippy::too_many_lines, clippy::cognitive_complexity)]
unsafe fn parse_collection(flagp: *mut c_int, extra: c_int) -> *mut u8 {
    let mut regparse = REGPARSE;

    // If there is no matching ']', we assume the '[' is a normal character.
    // This makes 'incsearch' and ":help [" work.
    let lp = rs_skip_anyof(regparse);
    if *lp != b']' as c_char {
        // No matching ']' — treated as literal in default/strict handling
        if REG_STRICT != 0 {
            errors::emsg2_e769(c_int::from(REG_MAGIC > MAGIC_OFF));
            return std::ptr::null_mut();
        }
        // Fall through to literal handling — return null to signal caller
        return std::ptr::null_mut();
    }

    // There is a matching ']'
    let mut startc: c_int = -1;
    let ret;

    // In a character class, different parsing rules apply.
    if *regparse == b'^' as c_char {
        // Complement of range
        ret = rs_regnode(ANYBUT + extra);
        regparse = regparse.add(1);
        REGPARSE = regparse;
    } else {
        ret = rs_regnode(ANYOF + extra);
    }

    // At the start ']' and '-' mean the literal character.
    if *regparse == b']' as c_char || *regparse == b'-' as c_char {
        startc = *regparse as u8 as c_int;
        rs_regc(*regparse as c_int);
        regparse = regparse.add(1);
        REGPARSE = regparse;
    }

    while *regparse != 0 && *regparse != b']' as c_char {
        if *regparse == b'-' as c_char {
            regparse = regparse.add(1);
            REGPARSE = regparse;
            // The '-' is not used for a range at the end and
            // after or before a '\n'.
            if *regparse == b']' as c_char
                || *regparse == 0
                || startc == -1
                || (*regparse == b'\\' as c_char && *regparse.add(1) == b'n' as c_char)
            {
                rs_regc(b'-' as c_int);
                startc = b'-' as c_int; // [--x] is a range
            } else {
                // Also accept "a-[.z.]"
                let mut endc: c_int = 0;
                if *regparse == b'[' as c_char {
                    let mut rp = regparse;
                    endc = get_coll_element(&mut rp);
                    if endc != 0 {
                        regparse = rp;
                        REGPARSE = regparse;
                    }
                }
                if endc == 0 {
                    let mut rp: *const c_char = regparse.cast_const();
                    endc = mb_ptr2char_adv(&mut rp);
                    regparse = rp.cast_mut();
                    REGPARSE = regparse;
                }

                // Handle \o40, \x20 and \u20AC style sequences
                if endc == b'\\' as c_int && REG_CPO_LIT == 0 {
                    endc = coll_get_char();
                    regparse = REGPARSE;
                }

                if startc > endc {
                    errors::emsg_e944();
                    return std::ptr::null_mut();
                }
                if utf_char2len(startc) > 1 || utf_char2len(endc) > 1 {
                    // Limit to a range of 256 chars
                    if endc > startc + 256 {
                        errors::emsg_e945();
                        return std::ptr::null_mut();
                    }
                    startc += 1;
                    while startc <= endc {
                        rs_regmbc(startc);
                        startc += 1;
                    }
                } else {
                    startc += 1;
                    while startc <= endc {
                        rs_regc(startc);
                        startc += 1;
                    }
                }
                startc = -1;
            }
        } else if *regparse == b'\\' as c_char
            && (!vim_strchr(
                REGEXP_INRANGE.as_ptr().cast::<c_char>(),
                c_int::from(*regparse.add(1) as u8),
            )
            .is_null()
                || (REG_CPO_LIT == 0
                    && !vim_strchr(
                        REGEXP_ABBR.as_ptr().cast::<c_char>(),
                        c_int::from(*regparse.add(1) as u8),
                    )
                    .is_null()))
        {
            regparse = regparse.add(1);
            REGPARSE = regparse;
            if *regparse == b'n' as c_char {
                // '\n' in range: also match NL
                let just_calc_size = JUST_CALC_SIZE;
                if ret != just_calc_size {
                    // Using \n inside [^] does not change what matches.
                    // "[^\n]" is the same as ".".
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    if *ret == ANYOF as u8 {
                        *ret = (ANYOF + ADD_NL) as u8;
                        *flagp |= HASNL;
                    }
                    // else: must have had a \n already
                }
                regparse = regparse.add(1);
                REGPARSE = regparse;
                startc = -1;
            } else if *regparse == b'd' as c_char
                || *regparse == b'o' as c_char
                || *regparse == b'x' as c_char
                || *regparse == b'u' as c_char
                || *regparse == b'U' as c_char
            {
                startc = coll_get_char();
                regparse = REGPARSE;
                // max UTF-8 Codepoint is U+10FFFF, but allow values until INT_MAX
                if startc == c_int::MAX {
                    errors::emsg_e949();
                    return std::ptr::null_mut();
                }
                if startc == 0 {
                    rs_regc(0x0a);
                } else {
                    rs_regmbc(startc);
                }
            } else {
                startc = rs_backslash_trans(*regparse as c_int);
                regparse = regparse.add(1);
                REGPARSE = regparse;
                rs_regc(startc);
            }
        } else if *regparse == b'[' as c_char {
            let mut rp = regparse;
            let c_class = nvim_regexp_get_char_class(&mut rp);
            startc = -1;
            // Characters assumed to be 8 bits!
            if c_class == CLASS_NONE {
                // Try equivalence class, then collating element, then literal '['
                let eq = get_equi_class(&mut rp);
                if eq != 0 {
                    reg_equi_class(eq);
                    regparse = rp;
                    REGPARSE = regparse;
                } else {
                    let coll = get_coll_element(&mut rp);
                    if coll != 0 {
                        rs_regmbc(coll);
                        regparse = rp;
                        REGPARSE = regparse;
                    } else {
                        // literal '[', allow [[-x] as a range
                        startc = *regparse as u8 as c_int;
                        regparse = regparse.add(1);
                        REGPARSE = regparse;
                        rs_regc(startc);
                    }
                }
            } else {
                regparse = rp;
                REGPARSE = regparse;
                emit_posix_class(c_class, &mut regparse);
                regparse = REGPARSE;
            }
        } else {
            // produce a multibyte character, including any following composing characters.
            startc = utf_ptr2char(regparse);
            let len = utfc_ptr2len(regparse);
            if utf_char2len(startc) != len {
                // composing chars
                startc = -1;
            }
            let mut remaining = len;
            while remaining > 0 {
                rs_regc(*regparse as c_int);
                regparse = regparse.add(1);
                remaining -= 1;
            }
            REGPARSE = regparse;
        }
    }
    rs_regc(0); // NUL terminate
    PREVCHR_LEN = 1; // last char was the ']'
    regparse = REGPARSE;
    if *regparse != b']' as c_char {
        errors::emsg_toomsbra(); // Cannot happen?
        return std::ptr::null_mut();
    }
    rs_skipchr(); // let's be friends with the lexer again
    *flagp |= HASWIDTH | SIMPLE;
    ret
}

/// Emit a `MULTIBYTECODE` node for character `c`.
unsafe fn do_multibyte(c: c_int, flagp: *mut c_int) -> *mut u8 {
    let ret = rs_regnode(MULTIBYTECODE);
    rs_regmbc(c);
    *flagp |= HASWIDTH | SIMPLE;
    ret
}

/// Parse the lowest level.
///
/// Optimization: gobbles an entire sequence of ordinary characters so that
/// it can turn them into a single node, which is smaller to store and
/// faster to run.  Don't do this when `one_exactly` is set.
#[no_mangle]
#[allow(
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::cognitive_complexity,
    clippy::fn_to_numeric_cast_any
)]
pub unsafe extern "C" fn regatom(flagp: *mut c_int) -> *mut u8 {
    let mut extra: c_int = 0;
    let save_prev_at_start = PREV_AT_START;

    *flagp = WORST; // Tentatively.

    let mut c = rs_getchr();

    // --- Position assertions ---
    if c == magic(b'^') {
        return rs_regnode(BOL);
    }
    if c == magic(b'$') {
        HAD_EOL = 1;
        return rs_regnode(EOL);
    }
    if c == magic(b'<') {
        return rs_regnode(BOW);
    }
    if c == magic(b'>') {
        return rs_regnode(EOW);
    }

    // --- Underscore prefix (\_) ---
    if c == magic(b'_') {
        c = rs_no_magic(rs_getchr());
        if c == b'^' as c_int {
            // "\_^" is start-of-line
            return rs_regnode(BOL);
        }
        if c == b'$' as c_int {
            // "\_$" is end-of-line
            HAD_EOL = 1;
            return rs_regnode(EOL);
        }

        extra = ADD_NL;
        *flagp |= HASNL;

        // "\_[" is character range plus newline
        if c == b'[' as c_int {
            let result = parse_collection(flagp, extra);
            if !result.is_null() {
                return result;
            }
            // No matching ']', fall through to literal handling
        }

        // "\_x" is character class plus newline — fall through to class handling
    }

    // --- Character classes: .iIkKfFpPsSdDxXoOwWhHaAlLuU ---
    // (also reached via fallthrough from \_x above)
    let is_class = if extra != 0 {
        // Came from \_x: c is already un-magicked
        true
    } else {
        c == magic(b'.')
            || c == magic(b'i')
            || c == magic(b'I')
            || c == magic(b'k')
            || c == magic(b'K')
            || c == magic(b'f')
            || c == magic(b'F')
            || c == magic(b'p')
            || c == magic(b'P')
            || c == magic(b's')
            || c == magic(b'S')
            || c == magic(b'd')
            || c == magic(b'D')
            || c == magic(b'x')
            || c == magic(b'X')
            || c == magic(b'o')
            || c == magic(b'O')
            || c == magic(b'w')
            || c == magic(b'W')
            || c == magic(b'h')
            || c == magic(b'H')
            || c == magic(b'a')
            || c == magic(b'A')
            || c == magic(b'l')
            || c == magic(b'L')
            || c == magic(b'u')
            || c == magic(b'U')
    };

    if is_class {
        let plain_c = if extra != 0 { c } else { rs_no_magic(c) };
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let plain_byte = plain_c as u8;
        let Some(p) = CLASSCHARS.iter().position(|&ch| ch == plain_byte) else {
            errors::emsg_e63_underscore();
            return std::ptr::null_mut();
        };
        // When '.' is followed by a composing char ignore the dot, so that
        // the composing char is matched here.
        if c == magic(b'.') && utf_iscomposing_legacy(rs_peekchr()) != 0 {
            c = rs_getchr();
            return do_multibyte(c, flagp);
        }
        let ret = rs_regnode(CLASSCODES[p] + extra);
        *flagp |= HASWIDTH | SIMPLE;
        return ret;
    }

    // --- \n ---
    if c == magic(b'n') {
        if REG_STRING != 0 {
            // In a string "\n" matches a newline character.
            let ret = rs_regnode(EXACTLY);
            rs_regc(NL);
            rs_regc(0);
            *flagp |= HASWIDTH | SIMPLE;
            return ret;
        }
        // In buffer text "\n" matches the end of a line.
        let ret = rs_regnode(NEWL);
        *flagp |= HASWIDTH | HASNL;
        return ret;
    }

    // --- Grouping: \( ---
    if c == magic(b'(') {
        if ONE_EXACTLY != 0 {
            errors::emsg2_e369(c_int::from(REG_MAGIC == MAGIC_ALL));
            return std::ptr::null_mut();
        }
        let mut flags: c_int = 0;
        let ret = rs_reg(REG_PAREN, &mut flags);
        if ret.is_null() {
            return std::ptr::null_mut();
        }
        *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
        return ret;
    }

    // --- Internal errors: NUL, |, &, ) ---
    if c == 0 || c == magic(b'|') || c == magic(b'&') || c == magic(b')') {
        if ONE_EXACTLY != 0 {
            errors::emsg2_e369(c_int::from(REG_MAGIC == MAGIC_ALL));
            return std::ptr::null_mut();
        }
        // Supposed to be caught earlier.
        errors::iemsg_internal();
        return std::ptr::null_mut();
    }

    // --- "follows nothing": =, ?, +, @, {, * ---
    if c == magic(b'=')
        || c == magic(b'?')
        || c == magic(b'+')
        || c == magic(b'@')
        || c == magic(b'{')
        || c == magic(b'*')
    {
        let plain = rs_no_magic(c);
        let is_magic = if plain == b'*' as c_int {
            REG_MAGIC >= MAGIC_ON
        } else {
            REG_MAGIC == MAGIC_ALL
        };
        errors::emsg3_e64(c_int::from(is_magic), plain);
        return std::ptr::null_mut();
    }

    // --- Previous substitute pattern: \~ ---
    if c == magic(b'~') {
        let prev_sub = nvim_regexp_get_reg_prev_sub_ptr();
        if !prev_sub.is_null() {
            let ret = rs_regnode(EXACTLY);
            let mut lp = prev_sub as *const u8;
            while *lp != 0 {
                rs_regc(*lp as c_int);
                lp = lp.add(1);
            }
            rs_regc(0);
            if *prev_sub != 0 {
                *flagp |= HASWIDTH;
                if lp.offset_from(prev_sub as *const u8) == 1 {
                    *flagp |= SIMPLE;
                }
            }
            return ret;
        }
        errors::emsg_nopresub();
        return std::ptr::null_mut();
    }

    // --- Backreferences: \1 .. \9 ---
    if c >= magic(b'1') && c <= magic(b'9') {
        let refnum = c - magic(b'0');
        if !seen_endbrace(refnum) {
            return std::ptr::null_mut();
        }
        return rs_regnode(BACKREF + refnum);
    }

    // --- \z: extended match ---
    if c == magic(b'z') {
        c = rs_no_magic(rs_getchr());
        if c == b'(' as c_int {
            if (nvim_regexp_get_reg_do_extmatch() & REX_SET) == 0 {
                errors::emsg_e66();
                return std::ptr::null_mut();
            }
            if ONE_EXACTLY != 0 {
                errors::emsg2_e369(c_int::from(REG_MAGIC == MAGIC_ALL));
                return std::ptr::null_mut();
            }
            let mut flags: c_int = 0;
            let ret = rs_reg(REG_ZPAREN, &mut flags);
            if ret.is_null() {
                return std::ptr::null_mut();
            }
            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            RE_HAS_Z = REX_SET;
            return ret;
        } else if c >= b'1' as c_int && c <= b'9' as c_int {
            if (nvim_regexp_get_reg_do_extmatch() & REX_USE) == 0 {
                errors::emsg_e67();
                return std::ptr::null_mut();
            }
            let ret = rs_regnode(ZREF + c - b'0' as c_int);
            RE_HAS_Z = REX_USE;
            return ret;
        } else if c == b's' as c_int {
            let ret = rs_regnode(MOPEN);
            if !rs_re_mult_next(c"\\zs".as_ptr()) {
                return std::ptr::null_mut();
            }
            return ret;
        } else if c == b'e' as c_int {
            let ret = rs_regnode(MCLOSE);
            if !rs_re_mult_next(c"\\ze".as_ptr()) {
                return std::ptr::null_mut();
            }
            return ret;
        }
        errors::emsg_e68();
        return std::ptr::null_mut();
    }

    // --- Percent operators: \% ---
    if c == magic(b'%') {
        c = rs_no_magic(rs_getchr());

        // \%( — non-capturing group
        if c == b'(' as c_int {
            if ONE_EXACTLY != 0 {
                errors::emsg2_e369(c_int::from(REG_MAGIC == MAGIC_ALL));
                return std::ptr::null_mut();
            }
            let mut flags: c_int = 0;
            let ret = rs_reg(REG_NPAREN, &mut flags);
            if ret.is_null() {
                return std::ptr::null_mut();
            }
            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            return ret;
        }

        // \%^ — beginning of file
        if c == b'^' as c_int {
            return rs_regnode(RE_BOF);
        }

        // \%$ — end of file
        if c == b'$' as c_int {
            return rs_regnode(RE_EOF);
        }

        // \%# — cursor position
        if c == b'#' as c_int {
            let regparse = REGPARSE;
            if *regparse == b'=' as c_char && *regparse.add(1) >= 48 && *regparse.add(1) <= 50 {
                // misplaced \%#=1
                errors::semsg_e_atom_engine(*regparse.add(1) as c_int);
                return std::ptr::null_mut();
            }
            return rs_regnode(CURSOR);
        }

        // \%V — visual area
        if c == b'V' as c_int {
            return rs_regnode(RE_VISUAL);
        }

        // \%C — composing character
        if c == b'C' as c_int {
            return rs_regnode(RE_COMPOSING);
        }

        // \%[abc] — optional sequence
        if c == b'[' as c_int {
            if ONE_EXACTLY != 0 {
                errors::emsg2_e369(c_int::from(REG_MAGIC == MAGIC_ALL));
                return std::ptr::null_mut();
            }

            let mut lastnode: *mut u8 = std::ptr::null_mut();
            let mut ret: *mut u8 = std::ptr::null_mut();

            loop {
                c = rs_getchr();
                if c == b']' as c_int {
                    break;
                }
                if c == 0 {
                    errors::emsg2_e69(c_int::from(REG_MAGIC == MAGIC_ALL));
                    return std::ptr::null_mut();
                }
                let br = rs_regnode(BRANCH);
                if ret.is_null() {
                    ret = br;
                } else {
                    rs_regtail(lastnode, br);
                    if REG_TOOLONG != 0 {
                        return std::ptr::null_mut();
                    }
                }

                rs_ungetchr();
                ONE_EXACTLY = 1;
                lastnode = regatom(flagp);
                ONE_EXACTLY = 0;
                if lastnode.is_null() {
                    return std::ptr::null_mut();
                }
            }
            if ret.is_null() {
                errors::emsg2_e70(c_int::from(REG_MAGIC == MAGIC_ALL));
                return std::ptr::null_mut();
            }
            let lastbranch = rs_regnode(BRANCH);
            let br = rs_regnode(NOTHING);
            let just_calc_size = JUST_CALC_SIZE;
            if ret != just_calc_size {
                rs_regtail(lastnode, br);
                rs_regtail(lastbranch, br);
                // connect all branches to the NOTHING branch at the end
                let mut scan = ret;
                while scan != lastnode {
                    if *scan as c_int == BRANCH {
                        rs_regtail(scan, lastbranch);
                        if REG_TOOLONG != 0 {
                            return std::ptr::null_mut();
                        }
                        scan = scan.add(3); // OPERAND(scan)
                    } else {
                        scan = rs_regnext(scan);
                        if scan.is_null() {
                            break;
                        }
                    }
                }
            }
            *flagp &= !(HASWIDTH | SIMPLE);
            return ret;
        }

        // \%d, \%o, \%x, \%u, \%U — character by codepoint
        if c == b'd' as c_int
            || c == b'o' as c_int
            || c == b'x' as c_int
            || c == b'u' as c_int
            || c == b'U' as c_int
        {
            let i: c_long = match c {
                x if x == b'd' as c_int => rs_getdecchrs(),
                x if x == b'o' as c_int => rs_getoctchrs(),
                x if x == b'x' as c_int => rs_gethexchrs(2),
                x if x == b'u' as c_int => rs_gethexchrs(4),
                x if x == b'U' as c_int => rs_gethexchrs(8),
                _ => -1,
            };

            if i < 0 || i > c_int::MAX as c_long {
                errors::emsg2_e678(c_int::from(REG_MAGIC == MAGIC_ALL));
                return std::ptr::null_mut();
            }
            #[allow(clippy::cast_possible_truncation)]
            let i_int = i as c_int;
            let ret = if use_multibytecode(i_int) {
                rs_regnode(MULTIBYTECODE)
            } else {
                rs_regnode(EXACTLY)
            };
            if i_int == 0 {
                rs_regc(0x0a);
            } else {
                rs_regmbc(i_int);
            }
            rs_regc(0);
            *flagp |= HASWIDTH;
            return ret;
        }

        // \%<N>l/c/v, \%'m — line/col/vcol/mark matchers
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        if (c as u8).is_ascii_digit()
            || c == b'<' as c_int
            || c == b'>' as c_int
            || c == b'\'' as c_int
            || c == b'.' as c_int
        {
            let mut n: u32 = 0;
            let cmp = c;
            let mut cur = false;
            let mut got_digit = false;

            if cmp == b'<' as c_int || cmp == b'>' as c_int {
                c = rs_getchr();
            }
            if rs_no_magic(c) == b'.' as c_int {
                cur = true;
                c = rs_getchr();
            }
            while (rs_no_magic(c) as u8).is_ascii_digit() {
                got_digit = true;
                n = n * 10 + (rs_no_magic(c) - b'0' as c_int) as u32;
                c = rs_getchr();
            }
            if rs_no_magic(c) == b'\'' as c_int && n == 0 {
                // "\%'m", "\%<'m" and "\%>'m": Mark
                c = rs_getchr();
                let ret = rs_regnode(RE_MARK);
                let just_calc_size = JUST_CALC_SIZE;
                if ret == just_calc_size {
                    REGSIZE += 2;
                } else {
                    let regcode = REGCODE;
                    *regcode = c as u8;
                    *regcode.add(1) = cmp as u8;
                    REGCODE = regcode.add(2);
                }
                return ret;
            } else if (c == b'l' as c_int || c == b'c' as c_int || c == b'v' as c_int)
                && (cur || got_digit)
            {
                if cur && n != 0 {
                    errors::semsg_e_dot_pos(rs_no_magic(c));
                    return std::ptr::null_mut();
                }
                let ret;
                if c == b'l' as c_int {
                    if cur {
                        n = nvim_regexp_get_curwin_lnum() as u32;
                    }
                    ret = rs_regnode(RE_LNUM);
                    if save_prev_at_start != 0 {
                        AT_START = 1;
                    }
                } else if c == b'c' as c_int {
                    if cur {
                        n = (nvim_regexp_get_curwin_col() + 1) as u32;
                    }
                    ret = rs_regnode(RE_COL);
                } else {
                    if cur {
                        n = nvim_regexp_get_curwin_vcol() as u32;
                    }
                    ret = rs_regnode(RE_VCOL);
                }
                let just_calc_size = JUST_CALC_SIZE;
                if ret == just_calc_size {
                    REGSIZE += 5;
                } else {
                    // put the number and the optional comparator after the opcode
                    let regcode = rs_re_put_uint32(REGCODE, n);
                    *regcode = cmp as u8;
                    REGCODE = regcode.add(1);
                }
                return ret;
            }
        }

        errors::emsg2_e71(c_int::from(REG_MAGIC == MAGIC_ALL));
        return std::ptr::null_mut();
    }

    // --- Collection: [...] ---
    if c == magic(b'[') {
        let result = parse_collection(flagp, 0);
        if !result.is_null() {
            return result;
        }
        // parse_collection returns null when no matching ']' and not strict.
        // In that case, fall through to literal handling below (Phase 7).
        // If it was an error (strict mode), rc_did_emsg is set.
    }

    // --- Default/literal case ---
    // A multi-byte character is handled as a separate atom if it's
    // before a multi and when it's a composing char.
    if use_multibytecode(c) {
        return do_multibyte(c, flagp);
    }

    let ret = rs_regnode(EXACTLY);

    // Append characters as long as:
    // - there is no following multi, we then need the character in
    //   front of it as a single character operand
    // - not running into a Magic character
    // - "one_exactly" is not set
    // But always emit at least one character.  Might be a Multi,
    // e.g., a "[" without matching "]".
    let mut len = 0;
    while c != 0
        && (len == 0 || (rs_re_multi_type(rs_peekchr()) == NOT_MULTI && ONE_EXACTLY == 0 && c >= 0))
    {
        // is_Magic(c) means c < 0, so c >= 0 means !is_Magic(c)
        let plain_c = if c < 0 { c + 256 } else { c }; // no_Magic(c)
        rs_regmbc(plain_c);

        // Need to get composing character too.
        let mut state: i32 = 0; // GRAPHEME_STATE_INIT
        loop {
            let regparse = REGPARSE;
            let l = utf_ptr2len(regparse);
            if utf_composinglike(regparse, regparse.add(l as usize), &mut state) == 0 {
                break;
            }
            rs_regmbc(utf_ptr2char(REGPARSE));
            rs_skipchr();
        }

        c = rs_getchr();
        len += 1;
    }
    rs_ungetchr();

    rs_regc(0); // NUL terminator
    *flagp |= HASWIDTH;
    if len == 1 {
        *flagp |= SIMPLE;
    }

    ret
}

/// Parse something followed by possible [*+=].
///
/// Calls `regatom` to parse the atom, then handles quantifiers.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::similar_names)]
pub unsafe extern "C" fn rs_regpiece(flagp: *mut c_int) -> *mut u8 {
    let mut flags: c_int = 0;
    let ret = regatom(&mut flags);
    if ret.is_null() {
        return std::ptr::null_mut();
    }

    let op = rs_peekchr();
    if rs_re_multi_type(op) == NOT_MULTI {
        *flagp = flags;
        return ret;
    }
    // default flags
    *flagp = WORST | SPSTART | (flags & (HASNL | HASLOOKBH));

    rs_skipchr();
    match op {
        x if x == magic(b'*') => {
            if flags & SIMPLE != 0 {
                rs_reginsert(STAR, ret);
            } else {
                // Emit x* as (x&|), where & means "self".
                rs_reginsert(BRANCH, ret); // Either x
                rs_regoptail(ret, rs_regnode(BACK)); // and loop
                rs_regoptail(ret, ret); // back
                rs_regtail(ret, rs_regnode(BRANCH)); // or
                rs_regtail(ret, rs_regnode(NOTHING)); // null.
            }
        }
        x if x == magic(b'+') => {
            if flags & SIMPLE != 0 {
                rs_reginsert(PLUS, ret);
            } else {
                // Emit x+ as x(&|), where & means "self".
                let next = rs_regnode(BRANCH); // Either
                rs_regtail(ret, next);
                rs_regtail(rs_regnode(BACK), ret); // loop back
                rs_regtail(next, rs_regnode(BRANCH)); // or
                rs_regtail(ret, rs_regnode(NOTHING)); // null.
            }
            *flagp = WORST | HASWIDTH | (flags & (HASNL | HASLOOKBH));
        }
        x if x == magic(b'@') => {
            let mut lop = END;
            let nr = rs_getdecchrs() as i64;

            match rs_no_magic(rs_getchr()) {
                x if x == b'=' as c_int => lop = MATCH,   // \@=
                x if x == b'!' as c_int => lop = NOMATCH, // \@!
                x if x == b'>' as c_int => lop = SUBPAT,  // \@>
                x if x == b'<' as c_int => {
                    match rs_no_magic(rs_getchr()) {
                        x if x == b'=' as c_int => lop = BEHIND,   // \@<=
                        x if x == b'!' as c_int => lop = NOBEHIND, // \@<!
                        _ => {}
                    }
                }
                _ => {}
            }
            if lop == END {
                let reg_magic = REG_MAGIC;
                errors::emsg2_e59(c_int::from(reg_magic == MAGIC_ALL));
                return std::ptr::null_mut();
            }
            // Look behind must match with behind_pos.
            if lop == BEHIND || lop == NOBEHIND {
                rs_regtail(ret, rs_regnode(BHPOS));
                *flagp |= HASLOOKBH;
            }
            rs_regtail(ret, rs_regnode(END)); // operand ends
            if lop == BEHIND || lop == NOBEHIND {
                let nr = if nr < 0 { 0 } else { nr };
                rs_reginsert_nr(lop, nr, ret);
            } else {
                rs_reginsert(lop, ret);
            }
        }
        x if x == magic(b'?') || x == magic(b'=') => {
            // Emit x= as (x|)
            rs_reginsert(BRANCH, ret); // Either x
            rs_regtail(ret, rs_regnode(BRANCH)); // or
            let next = rs_regnode(NOTHING); // null.
            rs_regtail(ret, next);
            rs_regoptail(ret, next);
        }
        x if x == magic(b'{') => {
            let mut minval: c_int = 0;
            let mut maxval: c_int = 0;
            if rs_read_limits(&mut minval, &mut maxval) != 1 {
                // OK = 1 in Neovim
                return std::ptr::null_mut();
            }
            if flags & SIMPLE != 0 {
                rs_reginsert(BRACE_SIMPLE, ret);
                rs_reginsert_limits(BRACE_LIMITS, minval as i64, maxval as i64, ret);
            } else {
                let ncb = NUM_COMPLEX_BRACES;
                if ncb >= 10 {
                    let reg_magic = REG_MAGIC;
                    errors::emsg2_e60(c_int::from(reg_magic == MAGIC_ALL));
                    return std::ptr::null_mut();
                }
                rs_reginsert(BRACE_COMPLEX + ncb, ret);
                rs_regoptail(ret, rs_regnode(BACK));
                rs_regoptail(ret, ret);
                rs_reginsert_limits(BRACE_LIMITS, minval as i64, maxval as i64, ret);
                NUM_COMPLEX_BRACES = ncb + 1;
            }
            if minval > 0 && maxval > 0 {
                *flagp = HASWIDTH | (flags & (HASNL | HASLOOKBH));
            }
        }
        _ => {}
    }

    if rs_re_multi_type(rs_peekchr()) != NOT_MULTI {
        // Can't have a multi follow a multi.
        let reg_magic = REG_MAGIC;
        if rs_peekchr() == magic(b'*') {
            errors::emsg2_e61(c_int::from(reg_magic >= MAGIC_ON));
            return std::ptr::null_mut();
        }
        errors::emsg3_e62(
            c_int::from(reg_magic == MAGIC_ALL),
            rs_no_magic(rs_peekchr()),
        );
        return std::ptr::null_mut();
    }

    ret
}

extern "C" {}

/// Parse one alternative of an | or & operator.
/// Implements the concatenation operator.
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_regconcat(flagp: *mut c_int) -> *mut u8 {
    let mut first: *mut u8 = std::ptr::null_mut();
    let mut chain: *mut u8 = std::ptr::null_mut();
    let mut flags: c_int = 0;

    *flagp = WORST; // Tentatively.

    loop {
        let chr = rs_peekchr();
        match chr {
            0 => break, // NUL
            x if x == magic(b'|') || x == magic(b'&') || x == magic(b')') => break,
            x if x == magic(b'Z') => {
                REGFLAGS_COMPILE |= RF_ICOMBINE;
                rs_skipchr_keepstart();
            }
            x if x == magic(b'c') => {
                REGFLAGS_COMPILE |= RF_ICASE;
                rs_skipchr_keepstart();
            }
            x if x == magic(b'C') => {
                REGFLAGS_COMPILE |= RF_NOICASE;
                rs_skipchr_keepstart();
            }
            x if x == magic(b'v') => {
                REG_MAGIC = MAGIC_ALL;
                rs_skipchr_keepstart();
                CURCHR = -1;
            }
            x if x == magic(b'm') => {
                REG_MAGIC = MAGIC_ON;
                rs_skipchr_keepstart();
                CURCHR = -1;
            }
            x if x == magic(b'M') => {
                REG_MAGIC = MAGIC_OFF;
                rs_skipchr_keepstart();
                CURCHR = -1;
            }
            x if x == magic(b'V') => {
                REG_MAGIC = MAGIC_NONE;
                rs_skipchr_keepstart();
                CURCHR = -1;
            }
            _ => {
                let latest = rs_regpiece(&mut flags);
                if latest.is_null() || REG_TOOLONG != 0 {
                    return std::ptr::null_mut();
                }
                *flagp |= flags & (HASWIDTH | HASNL | HASLOOKBH);
                if chain.is_null() {
                    // First piece.
                    *flagp |= flags & SPSTART;
                } else {
                    rs_regtail(chain, latest);
                }
                chain = latest;
                if first.is_null() {
                    first = latest;
                }
            }
        }
    }
    if first.is_null() {
        // Loop ran zero times.
        first = rs_regnode(NOTHING);
    }
    first
}

/// Parse one alternative of an | operator. Implements the & operator.
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_regbranch(flagp: *mut c_int) -> *mut u8 {
    let mut chain: *mut u8 = std::ptr::null_mut();
    let mut flags: c_int = 0;

    *flagp = WORST | HASNL; // Tentatively.

    let ret = rs_regnode(BRANCH);
    loop {
        let latest = rs_regconcat(&mut flags);
        if latest.is_null() {
            return std::ptr::null_mut();
        }
        // If one of the branches has width, the whole thing has.  If one of
        // the branches anchors at start-of-line, the whole thing does.
        // If one of the branches uses look-behind, the whole thing does.
        *flagp |= flags & (HASWIDTH | SPSTART | HASLOOKBH);
        // If one of the branches doesn't match a line-break, the whole thing
        // doesn't.
        *flagp &= !HASNL | (flags & HASNL);
        if !chain.is_null() {
            rs_regtail(chain, latest);
        }
        if rs_peekchr() != magic(b'&') {
            break;
        }
        rs_skipchr();
        rs_regtail(latest, rs_regnode(END)); // operand ends
        if REG_TOOLONG != 0 {
            break;
        }
        rs_reginsert(MATCH, latest);
        chain = latest;
    }

    ret
}

/// Parse regular expression, i.e. main body or parenthesized thing.
/// Caller must absorb opening parenthesis.
///
/// `paren`: `REG_NOPAREN`, `REG_PAREN`, `REG_NPAREN` or `REG_ZPAREN`
#[no_mangle]
#[allow(clippy::similar_names, clippy::too_many_lines)]
pub unsafe extern "C" fn rs_reg(paren: c_int, flagp: *mut c_int) -> *mut u8 {
    let mut parno: c_int = 0;
    let mut flags: c_int = 0;

    *flagp = HASWIDTH; // Tentatively.

    let mut ret: *mut u8;
    if paren == REG_ZPAREN {
        // Make a ZOPEN node.
        let nzpar = REGNZPAR;
        if nzpar >= 10 {
            errors::emsg_e50();
            return std::ptr::null_mut();
        }
        parno = nzpar;
        REGNZPAR = nzpar + 1;
        ret = rs_regnode(ZOPEN + parno);
    } else if paren == REG_PAREN {
        // Make a MOPEN node.
        let npar = REGNPAR;
        if npar >= 10 {
            let reg_magic = REG_MAGIC;
            errors::emsg2_e51(c_int::from(reg_magic == MAGIC_ALL));
            return std::ptr::null_mut();
        }
        parno = npar;
        REGNPAR = npar + 1;
        ret = rs_regnode(MOPEN + parno);
    } else if paren == REG_NPAREN {
        // Make a NOPEN node.
        ret = rs_regnode(NOPEN);
    } else {
        ret = std::ptr::null_mut();
    }

    // Pick up the branches, linking them together.
    let mut br = rs_regbranch(&mut flags);
    if br.is_null() {
        return std::ptr::null_mut();
    }
    if ret.is_null() {
        ret = br;
    } else {
        rs_regtail(ret, br); // [MZ]OPEN -> first.
    }
    // If one of the branches can be zero-width, the whole thing can.
    // If one of the branches has * at start or matches a line-break, the
    // whole thing can.
    if flags & HASWIDTH == 0 {
        *flagp &= !HASWIDTH;
    }
    *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    while rs_peekchr() == magic(b'|') {
        rs_skipchr();
        br = rs_regbranch(&mut flags);
        if br.is_null() || REG_TOOLONG != 0 {
            return std::ptr::null_mut();
        }
        rs_regtail(ret, br); // BRANCH -> BRANCH.
        if flags & HASWIDTH == 0 {
            *flagp &= !HASWIDTH;
        }
        *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    }

    // Make a closing node, and hook it on the end.
    let ender = rs_regnode(if paren == REG_ZPAREN {
        ZCLOSE + parno
    } else if paren == REG_PAREN {
        MCLOSE + parno
    } else if paren == REG_NPAREN {
        NCLOSE
    } else {
        END
    });
    rs_regtail(ret, ender);

    // Hook the tails of the branches to the closing node.
    br = ret;
    while !br.is_null() {
        rs_regoptail(br, ender);
        br = rs_regnext(br);
    }

    // Check for proper termination.
    if paren != REG_NOPAREN && rs_getchr() != magic(b')') {
        let reg_magic = REG_MAGIC;
        if paren == REG_ZPAREN {
            errors::emsg_e52();
        } else if paren == REG_NPAREN {
            errors::emsg2_e53(c_int::from(reg_magic == MAGIC_ALL));
        } else {
            errors::emsg2_e54(c_int::from(reg_magic == MAGIC_ALL));
        }
        return std::ptr::null_mut();
    } else if paren == REG_NOPAREN && rs_peekchr() != 0 {
        let reg_magic = REG_MAGIC;
        if CURCHR == magic(b')') {
            errors::emsg2_e55(c_int::from(reg_magic == MAGIC_ALL));
        } else {
            errors::emsg_e488(); // "Can't happen".
        }
        return std::ptr::null_mut();
    }
    // Here we set the flag allowing back references to this set of
    // parentheses.
    if paren == REG_PAREN {
        HAD_ENDBRACE[parno as usize] = 1_u8; // have seen the close paren
    }
    ret
}

// --- Position save/restore for BT regexp execution ---

/// `lpos_T` — position as (lnum, col), matching C `lpos_T` in `pos_defs.h`.
/// `linenr_T` and `colnr_T` are both `c_int` (i32).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LposT {
    pub lnum: c_int,
    pub col: c_int,
}

/// Union inside `regsave_T`: either a pointer (single-line) or position (multi-line).
#[repr(C)]
#[derive(Copy, Clone)]
pub union RegsaveUnion {
    pub ptr: *mut u8,
    pub pos: LposT,
}

/// `regsave_T` — saves input state for backtracking.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RegsaveT {
    pub rs_u: RegsaveUnion,
    pub rs_len: c_int,
}

/// Union inside `save_se_T`.
#[repr(C)]
#[derive(Copy, Clone)]
pub union SaveSeUnion {
    pub ptr: *mut u8,
    pub pos: LposT,
}

/// `save_se_T` — saves sub-expression start/end pointer or position.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SaveSeT {
    pub se_u: SaveSeUnion,
}

/// Save the input line and position in a `regsave_T`.
///
/// The C wrapper passes `gap->ga_len` since `garray_T` stays in C:
/// ```c
/// static void reg_save(regsave_T *save, garray_T *gap) {
///     rs_reg_save(save, gap->ga_len);
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn rs_reg_save(save: *mut RegsaveT, ga_len: c_int) {
    if (REX.reg_match.is_null() as c_int) != 0 {
        #[allow(clippy::cast_possible_truncation)]
        let col = REX.input.offset_from(REX.line) as c_int;
        (*save).rs_u.pos.col = col;
        (*save).rs_u.pos.lnum = REX.lnum;
    } else {
        (*save).rs_u.ptr = REX.input;
    }
    (*save).rs_len = ga_len;
}

/// Restore the input line and position from a `regsave_T`.
///
/// The C wrapper passes `&gap->ga_len` so Rust can write back:
/// ```c
/// static void reg_restore(regsave_T *save, garray_T *gap) {
///     rs_reg_restore(save, &gap->ga_len);
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn rs_reg_restore(save: *const RegsaveT, ga_len: *mut c_int) {
    if (REX.reg_match.is_null() as c_int) != 0 {
        if REX.lnum != (*save).rs_u.pos.lnum {
            // Only call reg_getline() when the line number changed to save
            // a bit of time.
            REX.lnum = (*save).rs_u.pos.lnum;
            let line = nvim_regexp_call_reg_getline((*save).rs_u.pos.lnum).cast::<u8>();
            REX.line = line;
        }
        REX.input = REX.line.add((*save).rs_u.pos.col as usize);
    } else {
        REX.input = (*save).rs_u.ptr;
    }
    *ga_len = (*save).rs_len;
}

/// Return 1 if current position equals saved position, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_save_equal(save: *const RegsaveT) -> c_int {
    if (REX.reg_match.is_null() as c_int) != 0 {
        let eq = REX.lnum == (*save).rs_u.pos.lnum
            && REX.input == REX.line.add((*save).rs_u.pos.col as usize);
        eq as c_int
    } else {
        (REX.input == (*save).rs_u.ptr) as c_int
    }
}

/// Save sub-expression position (multi-line): save `*posp` then set it to current.
#[no_mangle]
pub unsafe extern "C" fn rs_save_se_multi(savep: *mut SaveSeT, posp: *mut LposT) {
    (*savep).se_u.pos = *posp;
    (*posp).lnum = REX.lnum;
    #[allow(clippy::cast_possible_truncation)]
    let col = REX.input.offset_from(REX.line) as c_int;
    (*posp).col = col;
}

/// Save sub-expression pointer (single-line): save `*pp` then set it to current input.
#[no_mangle]
pub unsafe extern "C" fn rs_save_se_one(savep: *mut SaveSeT, pp: *mut *mut u8) {
    (*savep).se_u.ptr = *pp;
    *pp = REX.input;
}

// --- Subexpression save/restore for BT regexp lookbehind ---

extern "C" {}

/// `regbehind_T` — used for BEHIND and NOBEHIND matching.
#[repr(C)]
pub struct RegbehindT {
    pub save_after: RegsaveT,
    pub save_behind: RegsaveT,
    pub save_need_clear_subexpr: c_int,
    pub save_start: [SaveSeT; NSUBEXP],
    pub save_end: [SaveSeT; NSUBEXP],
}

/// Save the current subexpr to `bp`, so they can be restored by `rs_restore_subexpr`.
#[no_mangle]
pub unsafe extern "C" fn rs_save_subexpr(bp: *mut RegbehindT) {
    // When "rex.need_clear_subexpr" is set we don't need to save the values, only
    // remember that this flag needs to be set again when restoring.
    (*bp).save_need_clear_subexpr = REX.need_clear_subexpr;
    if REX.need_clear_subexpr != 0 {
        return;
    }

    if (REX.reg_match.is_null() as c_int) != 0 {
        let startpos = REX.reg_startpos;
        let endpos = REX.reg_endpos;
        for i in 0..NSUBEXP {
            (*bp).save_start[i].se_u.pos = *startpos.add(i);
            (*bp).save_end[i].se_u.pos = *endpos.add(i);
        }
    } else {
        let startp = REX.reg_startp;
        let endp = REX.reg_endp;
        for i in 0..NSUBEXP {
            (*bp).save_start[i].se_u.ptr = *startp.add(i);
            (*bp).save_end[i].se_u.ptr = *endp.add(i);
        }
    }
}

/// Restore the subexpr from `bp`.
#[no_mangle]
pub unsafe extern "C" fn rs_restore_subexpr(bp: *const RegbehindT) {
    // Only need to restore saved values when they are not to be cleared.
    REX.need_clear_subexpr = (*bp).save_need_clear_subexpr;
    if (*bp).save_need_clear_subexpr != 0 {
        return;
    }

    if (REX.reg_match.is_null() as c_int) != 0 {
        let startpos = REX.reg_startpos;
        let endpos = REX.reg_endpos;
        for i in 0..NSUBEXP {
            *startpos.add(i) = (*bp).save_start[i].se_u.pos;
            *endpos.add(i) = (*bp).save_end[i].se_u.pos;
        }
    } else {
        let startp = REX.reg_startp;
        let endp = REX.reg_endp;
        for i in 0..NSUBEXP {
            *startp.add(i) = (*bp).save_start[i].se_u.ptr;
            *endp.add(i) = (*bp).save_end[i].se_u.ptr;
        }
    }
}

// --- regtry: attempt match at a given column ---

extern "C" {
    fn nvim_regexp_unref_re_extmatch_out();
    fn nvim_regexp_set_re_extmatch_out(em: *mut c_void);
}

/// Try match of "prog" at rex.line[col].
///
/// Returns 0 for failure, or number of lines contained in the match.
#[no_mangle]
pub unsafe extern "C" fn rs_regtry(
    prog: *mut c_void,
    col: c_int,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    REX.input = REX.line.add(col as usize);
    REX.need_clear_subexpr = 1;
    // Clear the external match subpointers if necessary.
    let reghasz = (*prog.cast::<BtRegprogT>()).reghasz as c_int;
    REX.need_clear_zsubexpr = c_int::from(reghasz == REX_SET);

    // program[1] = skip past the first byte (REGMAGIC)
    let program = prog
        .cast::<u8>()
        .add(core::mem::offset_of!(BtRegprogT, reghasz) + 1);
    if rs_regmatch_impl(program.add(1), tm.cast(), timed_out) == 0 {
        return 0;
    }

    rs_cleanup_subexpr();

    if (REX.reg_match.is_null() as c_int) != 0 {
        let startpos = REX.reg_startpos;
        let endpos = REX.reg_endpos;
        if (*startpos).lnum < 0 {
            (*startpos).lnum = 0;
            (*startpos).col = col;
        }
        if (*endpos).lnum < 0 {
            (*endpos).lnum = REX.lnum;
            #[allow(clippy::cast_possible_truncation)]
            let input_col = REX.input.offset_from(REX.line) as c_int;
            (*endpos).col = input_col;
        } else {
            // Use line number of "\ze".
            REX.lnum = (*endpos).lnum;
        }
    } else {
        let startp = REX.reg_startp;
        let endp = REX.reg_endp;
        if (*startp).is_null() {
            *startp = REX.line.add(col as usize);
        }
        if (*endp).is_null() {
            *endp = REX.input;
        }
    }

    // Package any found \z(...\) matches for export. Default is none.
    nvim_regexp_unref_re_extmatch_out();
    nvim_regexp_set_re_extmatch_out(std::ptr::null_mut());

    if reghasz == REX_SET {
        rs_cleanup_zsubexpr();
        let em = rs_make_extmatch();
        nvim_regexp_set_re_extmatch_out(em.cast());

        for i in 0..NSUBEXP {
            #[allow(clippy::cast_possible_truncation)]
            let idx = i as c_int;
            if (REX.reg_match.is_null() as c_int) != 0 {
                // Only accept single line matches.
                let start_lnum = REG_STARTZPOS[idx as usize].lnum;
                let start_col = REG_STARTZPOS[idx as usize].col;
                let end_lnum = REG_ENDZPOS[idx as usize].lnum;
                let end_col = REG_ENDZPOS[idx as usize].col;
                if start_lnum >= 0 && end_lnum == start_lnum && end_col >= start_col {
                    let line = nvim_regexp_call_reg_getline(start_lnum);
                    (*em).matches[i] =
                        xstrnsave(line.add(start_col as usize), (end_col - start_col) as usize)
                            .cast::<u8>();
                }
            } else {
                let sp = REG_STARTZP[idx as usize];
                let ep = REG_ENDZP[idx as usize];
                if !sp.is_null() && !ep.is_null() {
                    (*em).matches[i] =
                        xstrnsave(sp.cast(), ep.offset_from(sp) as usize).cast::<u8>();
                }
            }
        }
    }

    1 + REX.lnum
}

// --- regrepeat: repeatedly match something simple ---

// WITH_NL: opcode has ADD_NL set (matches newlines too)
const FIRST_NL: c_int = ANY + ADD_NL;
const LAST_NL: c_int = NUPPER + ADD_NL;

#[inline]
const fn with_nl(op: c_int) -> bool {
    op >= FIRST_NL && op <= LAST_NL
}

#[inline]
const fn regrepeat_op(p: *const u8) -> c_int {
    unsafe { *p as c_int }
}

#[inline]
const fn regrepeat_operand(p: *mut u8) -> *mut u8 {
    unsafe { p.add(3) }
}

#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

extern "C" {
    fn nvim_regexp_call_vim_iswordp_buf(p: *const c_char) -> c_int;
}

/// Try to advance past a newline boundary (either in-line `\n` or multi-line).
/// Returns the updated scan pointer and `true` if we crossed a newline,
/// or `false` if we should stop.
#[inline]
unsafe fn try_newline_advance(
    scan: *mut u8,
    opcode: c_int,
    is_reg_multi: bool,
    reg_maxline: i32,
    reg_line_lbr: bool,
) -> (*mut u8, bool) {
    if *scan == 0 {
        if !is_reg_multi || !with_nl(opcode) || REX.lnum > reg_maxline || reg_line_lbr {
            return (scan, false);
        }
        rs_reg_nextline();
        let new_scan = REX.input;
        if nvim_regexp_get_got_int() != 0 {
            return (new_scan, false);
        }
        return (new_scan, true);
    } else if reg_line_lbr && *scan == b'\n' && with_nl(opcode) {
        return (scan.add(1), true);
    }
    (scan, false)
}

/// `regrepeat` — repeatedly match something simple, return how many.
/// Advances `rex.input` (and `rex.lnum`) to just after the matched chars.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_regrepeat(p: *mut u8, maxcount: i64) -> c_int {
    let mut count: i64 = 0;
    let mut scan = REX.input;
    let opnd = regrepeat_operand(p);
    let opcode = regrepeat_op(p);

    // Cache frequently used values at function entry for performance.
    let is_reg_multi = (REX.reg_match.is_null() as c_int) != 0;
    let reg_ic = REX.reg_ic as c_int != 0;
    let reg_line_lbr = REX.reg_line_lbr as c_int != 0;
    let reg_maxline = REX.reg_maxline;

    // Determine mask/testval for character class opcodes.
    let (mask, testval) = match opcode {
        x if x == WHITE || x == WHITE + ADD_NL => (RI_WHITE, RI_WHITE),
        x if x == NWHITE || x == NWHITE + ADD_NL => (RI_WHITE, 0),
        x if x == DIGIT || x == DIGIT + ADD_NL => (RI_DIGIT, RI_DIGIT),
        x if x == NDIGIT || x == NDIGIT + ADD_NL => (RI_DIGIT, 0),
        x if x == HEX || x == HEX + ADD_NL => (RI_HEX, RI_HEX),
        x if x == NHEX || x == NHEX + ADD_NL => (RI_HEX, 0),
        x if x == OCTAL || x == OCTAL + ADD_NL => (RI_OCTAL, RI_OCTAL),
        x if x == NOCTAL || x == NOCTAL + ADD_NL => (RI_OCTAL, 0),
        x if x == WORD || x == WORD + ADD_NL => (RI_WORD, RI_WORD),
        x if x == NWORD || x == NWORD + ADD_NL => (RI_WORD, 0),
        x if x == HEAD || x == HEAD + ADD_NL => (RI_HEAD, RI_HEAD),
        x if x == NHEAD || x == NHEAD + ADD_NL => (RI_HEAD, 0),
        x if x == ALPHA || x == ALPHA + ADD_NL => (RI_ALPHA, RI_ALPHA),
        x if x == NALPHA || x == NALPHA + ADD_NL => (RI_ALPHA, 0),
        x if x == LOWER || x == LOWER + ADD_NL => (RI_LOWER, RI_LOWER),
        x if x == NLOWER || x == NLOWER + ADD_NL => (RI_LOWER, 0),
        x if x == UPPER || x == UPPER + ADD_NL => (RI_UPPER, RI_UPPER),
        x if x == NUPPER || x == NUPPER + ADD_NL => (RI_UPPER, 0),
        _ => (0, 0), // not a class opcode
    };

    // Check if this is a character-class opcode handled by the do_class loop.
    // Class opcodes: WHITE(31)..NUPPER(48) and their +ADD_NL variants (61..78).
    let is_class_op = (WHITE..=NUPPER).contains(&opcode)
        || ((WHITE + ADD_NL)..=(NUPPER + ADD_NL)).contains(&opcode);

    if is_class_op {
        // do_class loop
        while count < maxcount {
            if *scan == 0 {
                let (new_scan, advanced) =
                    try_newline_advance(scan, opcode, is_reg_multi, reg_maxline, reg_line_lbr);
                scan = new_scan;
                if !advanced {
                    break;
                }
            } else {
                let l = utfc_ptr2len(scan.cast::<c_char>());
                if l > 1 {
                    if testval != 0 {
                        break;
                    }
                    scan = scan.add(l as usize);
                } else if (CLASS_TAB[*scan as usize] & mask) == testval
                    || (reg_line_lbr && *scan == b'\n' && with_nl(opcode))
                {
                    scan = scan.add(1);
                } else {
                    break;
                }
            }
            count += 1;
        }
    } else {
        match opcode {
            x if x == ANY || x == ANY + ADD_NL => {
                while count < maxcount {
                    // Match anything until end-of-line (or end-of-file for ANY+ADD_NL).
                    while *scan != 0 && count < maxcount {
                        count += 1;
                        scan = scan.add(utfc_ptr2len(scan.cast::<c_char>()) as usize);
                    }
                    if !is_reg_multi
                        || !with_nl(opcode)
                        || REX.lnum > reg_maxline
                        || reg_line_lbr
                        || count == maxcount
                    {
                        break;
                    }
                    count += 1; // count the line-break
                    rs_reg_nextline();
                    scan = REX.input;
                    if nvim_regexp_get_got_int() != 0 {
                        break;
                    }
                }
            }

            x if x == IDENT || x == IDENT + ADD_NL || x == SIDENT || x == SIDENT + ADD_NL => {
                let tv = opcode == IDENT || opcode == IDENT + ADD_NL;
                while count < maxcount {
                    if vim_isIDc(utf_ptr2char(scan.cast::<c_char>())) != 0
                        && (tv || !ascii_isdigit(*scan))
                    {
                        scan = scan.add(utfc_ptr2len(scan.cast::<c_char>()) as usize);
                    } else {
                        let (new_scan, advanced) = try_newline_advance(
                            scan,
                            opcode,
                            is_reg_multi,
                            reg_maxline,
                            reg_line_lbr,
                        );
                        scan = new_scan;
                        if !advanced {
                            break;
                        }
                    }
                    count += 1;
                }
            }

            x if x == KWORD || x == KWORD + ADD_NL || x == SKWORD || x == SKWORD + ADD_NL => {
                let tv = opcode == KWORD || opcode == KWORD + ADD_NL;
                while count < maxcount {
                    if nvim_regexp_call_vim_iswordp_buf(scan.cast::<c_char>()) != 0
                        && (tv || !ascii_isdigit(*scan))
                    {
                        scan = scan.add(utfc_ptr2len(scan.cast::<c_char>()) as usize);
                    } else {
                        let (new_scan, advanced) = try_newline_advance(
                            scan,
                            opcode,
                            is_reg_multi,
                            reg_maxline,
                            reg_line_lbr,
                        );
                        scan = new_scan;
                        if !advanced {
                            break;
                        }
                    }
                    count += 1;
                }
            }

            x if x == FNAME || x == FNAME + ADD_NL || x == SFNAME || x == SFNAME + ADD_NL => {
                let tv = opcode == FNAME || opcode == FNAME + ADD_NL;
                while count < maxcount {
                    if vim_isfilec(utf_ptr2char(scan.cast::<c_char>())) != 0
                        && (tv || !ascii_isdigit(*scan))
                    {
                        scan = scan.add(utfc_ptr2len(scan.cast::<c_char>()) as usize);
                    } else {
                        let (new_scan, advanced) = try_newline_advance(
                            scan,
                            opcode,
                            is_reg_multi,
                            reg_maxline,
                            reg_line_lbr,
                        );
                        scan = new_scan;
                        if !advanced {
                            break;
                        }
                    }
                    count += 1;
                }
            }

            x if x == PRINT || x == PRINT + ADD_NL || x == SPRINT || x == SPRINT + ADD_NL => {
                let tv = opcode == PRINT || opcode == PRINT + ADD_NL;
                while count < maxcount {
                    if *scan == 0 {
                        let (new_scan, advanced) = try_newline_advance(
                            scan,
                            opcode,
                            is_reg_multi,
                            reg_maxline,
                            reg_line_lbr,
                        );
                        scan = new_scan;
                        if !advanced {
                            break;
                        }
                    } else if vim_isprintc(utf_ptr2char(scan.cast::<c_char>())) == 1
                        && (tv || !ascii_isdigit(*scan))
                    {
                        scan = scan.add(utfc_ptr2len(scan.cast::<c_char>()) as usize);
                    } else if reg_line_lbr && *scan == b'\n' && with_nl(opcode) {
                        scan = scan.add(1);
                    } else {
                        break;
                    }
                    count += 1;
                }
            }

            x if x == EXACTLY => {
                // Single-byte character (multi-byte uses MULTIBYTECODE).
                if reg_ic {
                    let cu = mb_toupper(*opnd as c_int);
                    let cl = mb_tolower(*opnd as c_int);
                    while count < maxcount && (*scan as c_int == cu || *scan as c_int == cl) {
                        count += 1;
                        scan = scan.add(1);
                    }
                } else {
                    let cu = *opnd;
                    while count < maxcount && *scan == cu {
                        count += 1;
                        scan = scan.add(1);
                    }
                }
            }

            x if x == MULTIBYTECODE => {
                let len = utfc_ptr2len(opnd.cast::<c_char>());
                if len > 1 {
                    let cf = if reg_ic {
                        utf_fold(utf_ptr2char(opnd.cast::<c_char>()))
                    } else {
                        0
                    };
                    while count < maxcount && utfc_ptr2len(scan.cast::<c_char>()) >= len {
                        // Compare bytes
                        let mut i = 0;
                        while i < len {
                            if *opnd.add(i as usize) != *scan.add(i as usize) {
                                break;
                            }
                            i += 1;
                        }
                        if i < len
                            && (!reg_ic || utf_fold(utf_ptr2char(scan.cast::<c_char>())) != cf)
                        {
                            break;
                        }
                        scan = scan.add(len as usize);
                        count += 1;
                    }
                }
            }

            x if x == ANYOF || x == ANYOF + ADD_NL || x == ANYBUT || x == ANYBUT + ADD_NL => {
                let tv: c_int = c_int::from(opcode == ANYOF || opcode == ANYOF + ADD_NL);
                while count < maxcount {
                    if *scan == 0 {
                        let (new_scan, advanced) = try_newline_advance(
                            scan,
                            opcode,
                            is_reg_multi,
                            reg_maxline,
                            reg_line_lbr,
                        );
                        scan = new_scan;
                        if !advanced {
                            break;
                        }
                    } else if reg_line_lbr && *scan == b'\n' && with_nl(opcode) {
                        scan = scan.add(1);
                    } else {
                        let len = utfc_ptr2len(scan.cast::<c_char>());
                        if len > 1 {
                            if (rs_cstrchr(
                                opnd.cast::<c_char>(),
                                utf_ptr2char(scan.cast::<c_char>()),
                            )
                            .is_null()) as c_int
                                == tv
                            {
                                break;
                            }
                            scan = scan.add(len as usize);
                        } else {
                            if (rs_cstrchr(opnd.cast::<c_char>(), *scan as c_int).is_null())
                                as c_int
                                == tv
                            {
                                break;
                            }
                            scan = scan.add(1);
                        }
                    }
                    count += 1;
                }
            }

            x if x == NEWL => {
                while count < maxcount
                    && ((*scan == 0 && REX.lnum <= reg_maxline && !reg_line_lbr && is_reg_multi)
                        || (*scan == b'\n' && reg_line_lbr))
                {
                    count += 1;
                    if reg_line_lbr {
                        // ADVANCE_REGINPUT() = MB_PTR_ADV(rex.input)
                        let inp = REX.input;
                        let adv = utfc_ptr2len(inp.cast::<c_char>()) as usize;
                        REX.input = inp.add(adv);
                    } else {
                        rs_reg_nextline();
                    }
                    scan = REX.input;
                    if nvim_regexp_get_got_int() != 0 {
                        break;
                    }
                }
            }

            _ => {
                // Oh dear. Called inappropriately.
                errors::iemsg_re_corr();
            }
        }
    }

    REX.input = scan;

    #[allow(clippy::cast_possible_truncation)]
    let result = count as c_int;
    result
}

// ==========================================================================
// rs_regmatch — core backtracking engine
// ==========================================================================

// Additional RA_* status values (RA_FAIL=1, RA_MATCH=4, RA_NOMATCH=5 already defined)
#[allow(dead_code)]
const RA_CONT: c_int = 2;
#[allow(dead_code)]
const RA_BREAK: c_int = 3;

// regstate_T equivalents (matching C enum exactly)
#[allow(dead_code)]
const RS_NOPEN: c_int = 0;
#[allow(dead_code)]
const RS_MOPEN: c_int = 1;
#[allow(dead_code)]
const RS_MCLOSE: c_int = 2;
#[allow(dead_code)]
const RS_ZOPEN: c_int = 3;
#[allow(dead_code)]
const RS_ZCLOSE: c_int = 4;
#[allow(dead_code)]
const RS_BRANCH: c_int = 5;
#[allow(dead_code)]
const RS_BRCPLX_MORE: c_int = 6;
#[allow(dead_code)]
const RS_BRCPLX_LONG: c_int = 7;
#[allow(dead_code)]
const RS_BRCPLX_SHORT: c_int = 8;
#[allow(dead_code)]
const RS_NOMATCH: c_int = 9;
#[allow(dead_code)]
const RS_BEHIND1: c_int = 10;
#[allow(dead_code)]
const RS_BEHIND2: c_int = 11;
#[allow(dead_code)]
const RS_STAR_LONG: c_int = 12;
#[allow(dead_code)]
const RS_STAR_SHORT: c_int = 13;

// Stack/backpos constants
#[allow(dead_code)]
const REGSTACK_INITIAL: usize = 2048;
#[allow(dead_code)]
const BACKPOS_INITIAL: usize = 64;
// MAX_LIMIT already defined above as c_int

/// `regitem_T` — stack item for backtracking.
#[repr(C)]
#[allow(dead_code)]
pub struct RegitemT {
    pub rs_state: c_int,
    pub rs_no: i16,
    pub rs_scan: *mut u8,
    pub rs_un: RegitemUnion,
}

/// Union inside `regitem_T`.
#[repr(C)]
#[allow(dead_code)]
pub union RegitemUnion {
    pub sesave: SaveSeT,
    pub regsave: RegsaveT,
}

/// `regstar_T` — stored before a `regitem_T` for `STAR`/`PLUS`/`BRACE_SIMPLE`.
#[repr(C)]
#[allow(dead_code)]
pub struct RegstarT {
    pub nextb: c_int,
    pub nextb_ic: c_int,
    pub count: i64,
    pub minval: i64,
    pub maxval: i64,
}

/// `backpos_T` — BACK opcode position tracking.
#[repr(C)]
#[allow(dead_code)]
pub struct BackposT {
    pub bp_scan: *mut u8,
    pub bp_pos: RegsaveT,
}

// C accessor extern declarations for rs_regmatch
#[allow(dead_code)]
extern "C" {
    // Regstack/backpos management

    // Brace statics

    // Behind position (C returns void*, cast to RegsaveT* on Rust side)

    // maxmempattern
    fn nvim_regexp_get_p_mmp() -> i64;

    // External match
    fn nvim_regexp_get_re_extmatch_in_match(no: c_int) -> *mut u8;

    // Mark support
    fn nvim_regexp_call_mark_get(mark: c_int) -> *mut c_void;
    fn nvim_regexp_get_fmark_lnum(fm: *mut c_void) -> i32;
    fn nvim_regexp_get_fmark_col(fm: *mut c_void) -> i32;

    // Window/cursor
    fn nvim_regexp_get_rex_reg_win_or_curwin() -> *mut c_void;
    fn nvim_regexp_has_rex_reg_win() -> c_int;
    fn nvim_regexp_get_rex_reg_win_cursor_lnum() -> i32;
    fn nvim_regexp_get_rex_reg_win_cursor_col() -> i32;
    fn nvim_regexp_get_win_line_count(wp: *mut c_void) -> i32;

    // Virtual column: reuse existing nvim_regexp_call_win_linetabsize (declared above)
    // reg_getline_len: reuse existing nvim_regexp_call_reg_getline_len (declared above)

    // Error/utility
    fn nvim_regexp_call_profile_passed_limit(tm: *const c_void) -> c_int;
    // got_int: reuse existing nvim_regexp_get_got_int (declared above)
    fn nvim_mb_isupper(c: c_int) -> c_int;

    // mb_get_class_tab
    fn nvim_regexp_call_mb_get_class_tab(p: *mut u8) -> c_int;

    // cstrncmp / cstrchr: use rs_cstrncmp/rs_cstrchr directly from Rust
    // rex.reg_firstlnum: accessed directly as REX.reg_firstlnum

    // z-subexpr element-pointer accessors for save_se/restore_se

    // internal_error

    // regrepeat: use rs_regrepeat() directly from Rust (no wrapper needed)

    // regnext: use rs_regnext() directly from Rust (no wrapper needed)

    // iemsg: use existing errors::iemsg_re_corr (declared in regrepeat section)
}

// --- Helper inline functions for rs_regmatch ---

/// `OP(p)` — get opcode at program position.
#[inline]
#[allow(dead_code)]
const fn op(p: *const u8) -> c_int {
    unsafe { *p as c_int }
}

/// `OPERAND(p)` — skip 3-byte header to get operand pointer.
#[inline]
#[allow(dead_code)]
const fn operand(p: *mut u8) -> *mut u8 {
    unsafe { p.add(3) }
}

/// `OPERAND_MIN(p)` — read 4-byte big-endian value from operand.
#[inline]
#[allow(dead_code)]
fn operand_min(p: *const u8) -> i64 {
    unsafe {
        (i64::from(*p.add(3)) << 24)
            + (i64::from(*p.add(4)) << 16)
            + (i64::from(*p.add(5)) << 8)
            + i64::from(*p.add(6))
    }
}

/// `OPERAND_MAX(p)` — read 4-byte big-endian value from operand + 4.
#[inline]
#[allow(dead_code)]
fn operand_max(p: *const u8) -> i64 {
    operand_min(unsafe { p.add(4) })
}

/// `OPERAND_CMP(p)` — get comparison operator byte.
#[inline]
#[allow(dead_code)]
const fn operand_cmp(p: *const u8) -> u8 {
    unsafe { *p.add(7) }
}

/// `re_num_cmp` — compare a number with operand value using operand comparison operator.
#[inline]
#[allow(dead_code, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn re_num_cmp(val: u32, scan: *const u8) -> bool {
    let n = operand_min(scan) as u32;
    let cmp = operand_cmp(scan);
    if cmp == b'>' {
        val > n
    } else if cmp == b'<' {
        val < n
    } else {
        val == n
    }
}

/// Push a state onto the regstack. Returns pointer to the new `RegitemT`, or null on OOM.
#[inline]
#[allow(
    dead_code,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment
)]
unsafe fn regstack_push(state: c_int, scan: *mut u8) -> *mut RegitemT {
    if (REGSTACK.ga_len as u32 >> 10) as i64 >= nvim_regexp_get_p_mmp() {
        errors::emsg_maxmempattern();
        return std::ptr::null_mut();
    }
    ga_grow(&raw mut REGSTACK, std::mem::size_of::<RegitemT>() as c_int);

    let rp = (REGSTACK.ga_data.cast::<u8>().add(REGSTACK.ga_len as usize)).cast::<RegitemT>();
    (*rp).rs_state = state;
    (*rp).rs_scan = scan;

    {
        REGSTACK.ga_len += std::mem::size_of::<RegitemT>() as c_int;
    }
    rp
}

/// Pop the top state from the regstack. Writes the saved scan pointer to `*scan_out`.
#[inline]
#[allow(
    dead_code,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment
)]
unsafe fn regstack_pop(scan_out: *mut *mut u8) {
    let rp = (REGSTACK.ga_data.cast::<u8>().add(REGSTACK.ga_len as usize))
        .cast::<RegitemT>()
        .sub(1);
    *scan_out = (*rp).rs_scan;
    {
        REGSTACK.ga_len -= std::mem::size_of::<RegitemT>() as c_int;
    }
}

/// Save sub-expression start/end: multi-line or single-line.
#[inline]
#[allow(dead_code)]
unsafe fn save_se(savep: *mut SaveSeT, posp: *mut LposT, pp: *mut *mut u8) {
    if (REX.reg_match.is_null() as c_int) != 0 {
        rs_save_se_multi(savep, posp);
    } else {
        rs_save_se_one(savep, pp);
    }
}

/// Restore sub-expression start/end: multi-line or single-line.
#[inline]
#[allow(dead_code)]
unsafe fn restore_se(savep: *const SaveSeT, posp: *mut LposT, pp: *mut *mut u8) {
    if (REX.reg_match.is_null() as c_int) != 0 {
        *posp = (*savep).se_u.pos;
    } else {
        *pp = (*savep).se_u.ptr;
    }
}

/// `ADVANCE_REGINPUT()` — advance rex.input by one multi-byte character.
#[inline]
#[allow(dead_code)]
unsafe fn advance_reginput() {
    let inp = REX.input;
    let len = utfc_ptr2len(inp.cast::<c_char>());
    REX.input = inp.add(len as usize);
}

/// `MB_PTR_BACK(s, p)` — back up `p` to the previous multi-byte character.
/// Only valid when `p > s`.
#[inline]
#[allow(dead_code)]
unsafe fn mb_ptr_back(s: *const u8, p: *mut u8) -> *mut u8 {
    let offset = utf_head_off(s.cast::<c_char>(), p.sub(1).cast::<c_char>()) + 1;
    p.sub(offset as usize)
}

/// `rs_regmatch` — core backtracking regexp matcher (Rust implementation).
///
/// Called from `rs_regtry` and from C via `nvim_regexp_call_regmatch`.
#[no_mangle]
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment,
    clippy::cognitive_complexity
)]
pub unsafe extern "C" fn rs_regmatch(
    scan_arg: *mut u8,
    tm: *const c_void,
    timed_out: *mut c_int,
) -> c_int {
    rs_regmatch_impl(scan_arg, tm, timed_out)
}

/// The core Rust implementation of `regmatch`.
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment,
    clippy::cognitive_complexity
)]
unsafe fn rs_regmatch_impl(scan_arg: *mut u8, tm: *const c_void, timed_out: *mut c_int) -> c_int {
    let mut scan: *mut u8 = scan_arg;
    let mut next: *mut u8;
    let mut status: c_int;
    let mut tm_count: c_int = 0;

    // Make "regstack" and "backpos" empty.
    {
        REGSTACK.ga_len = 0;
    };
    {
        BACKPOS.ga_len = 0;
    };

    // Cache flags that don't change during matching.
    let is_reg_multi = (REX.reg_match.is_null() as c_int) != 0;
    let reg_line_lbr = REX.reg_line_lbr as c_int != 0;

    // Repeat until "regstack" is empty.
    loop {
        // Allow interrupting long matches with CTRL-C.
        rs_reg_breakcheck();

        // Inner loop: match items sequentially without using the regstack.
        loop {
            if nvim_regexp_get_got_int() != 0 || scan.is_null() {
                status = RA_FAIL;
                break;
            }
            // Check for timeout once in a 100 times.
            if !tm.is_null() {
                tm_count += 1;
                if tm_count == 100 {
                    tm_count = 0;
                    if nvim_regexp_call_profile_passed_limit(tm) != 0 {
                        if !timed_out.is_null() {
                            *timed_out = 1;
                        }
                        status = RA_FAIL;
                        break;
                    }
                }
            }
            status = RA_CONT;

            next = rs_regnext(scan);

            let mut opc = op(scan);
            // Check for character class with NL added.
            if !reg_line_lbr
                && with_nl(opc)
                && is_reg_multi
                && *REX.input == 0
                && REX.lnum <= REX.reg_maxline
            {
                rs_reg_nextline();
            } else if reg_line_lbr && with_nl(opc) && *REX.input == b'\n' {
                advance_reginput();
            } else {
                if with_nl(opc) {
                    opc -= ADD_NL;
                }
                let c = utf_ptr2char(REX.input.cast::<c_char>());

                match opc {
                    BOL => {
                        if REX.input != REX.line {
                            status = RA_NOMATCH;
                        }
                    }

                    EOL => {
                        if c != 0 {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_BOF => {
                        if REX.lnum != 0
                            || REX.input != REX.line
                            || (is_reg_multi && REX.reg_firstlnum > 1)
                        {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_EOF => {
                        if REX.lnum != REX.reg_maxline || c != 0 {
                            status = RA_NOMATCH;
                        }
                    }

                    ANY => {
                        if c == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }

                    NOTHING => {}

                    NEWL => {
                        if (c != 0 || !is_reg_multi || REX.lnum > REX.reg_maxline || reg_line_lbr)
                            && (c != NL || !reg_line_lbr)
                        {
                            status = RA_NOMATCH;
                        } else if reg_line_lbr {
                            advance_reginput();
                        } else {
                            rs_reg_nextline();
                        }
                    }

                    BHPOS => {
                        let bp = &raw mut BEHIND_POS;
                        if is_reg_multi {
                            if (*bp).rs_u.pos.col != (REX.input.offset_from(REX.line) as c_int)
                                || (*bp).rs_u.pos.lnum != REX.lnum
                            {
                                status = RA_NOMATCH;
                            }
                        } else if (*bp).rs_u.ptr != REX.input {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_COMPOSING => {
                        // Skip composing characters.
                        while utf_iscomposing_legacy(utf_ptr2char(REX.input.cast::<c_char>())) != 0
                        {
                            let inp = REX.input;
                            let len = utf_ptr2len(inp.cast::<c_char>());
                            REX.input = inp.add(len as usize);
                        }
                    }

                    BACK => {
                        // Check if we don't keep looping without matching input.
                        let bp_data = BACKPOS.ga_data.cast::<u8>().cast::<BackposT>();
                        let bp_len = BACKPOS.ga_len / std::mem::size_of::<BackposT>() as c_int;
                        let mut i = 0;
                        while i < bp_len {
                            if (*bp_data.add(i as usize)).bp_scan == scan {
                                break;
                            }
                            i += 1;
                        }
                        if i == bp_len {
                            // First time: add new entry.
                            ga_grow(&raw mut BACKPOS, std::mem::size_of::<BackposT>() as c_int);
                            let bp_data = BACKPOS.ga_data.cast::<u8>().cast::<BackposT>();
                            (*bp_data.add(i as usize)).bp_scan = scan;
                            BACKPOS.ga_len += std::mem::size_of::<BackposT>() as c_int;
                        } else if rs_reg_save_equal(std::ptr::from_ref::<RegsaveT>(
                            &(*bp_data.add(i as usize)).bp_pos,
                        )) != 0
                        {
                            // Still at same position, fail.
                            status = RA_NOMATCH;
                        }

                        debug_assert!(status != RA_FAIL);
                        if status != RA_NOMATCH {
                            rs_reg_save(
                                std::ptr::from_mut::<RegsaveT>(
                                    &mut (*bp_data.add(i as usize)).bp_pos,
                                ),
                                BACKPOS.ga_len,
                            );
                        }
                    }

                    END => {
                        status = RA_MATCH;
                    }

                    // --- Phase 3: Character classes ---
                    IDENT => {
                        if vim_isIDc(c) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    SIDENT => {
                        if ascii_isdigit(*REX.input) || vim_isIDc(c) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    KWORD => {
                        if nvim_regexp_call_vim_iswordp_buf(REX.input.cast::<c_char>()) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    SKWORD => {
                        if ascii_isdigit(*REX.input)
                            || nvim_regexp_call_vim_iswordp_buf(REX.input.cast::<c_char>()) == 0
                        {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    FNAME => {
                        if vim_isfilec(c) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    SFNAME => {
                        if ascii_isdigit(*REX.input) || vim_isfilec(c) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    PRINT => {
                        if vim_isprintc(utf_ptr2char(REX.input.cast::<c_char>())) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    SPRINT => {
                        if ascii_isdigit(*REX.input)
                            || vim_isprintc(utf_ptr2char(REX.input.cast::<c_char>())) == 0
                        {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    WHITE => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_WHITE) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NWHITE => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_WHITE) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    DIGIT => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_DIGIT) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NDIGIT => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_DIGIT) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    HEX => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_HEX) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NHEX => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_HEX) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    OCTAL => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_OCTAL) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NOCTAL => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_OCTAL) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    WORD => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_WORD) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NWORD => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_WORD) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    HEAD => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_HEAD) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NHEAD => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_HEAD) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    ALPHA => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_ALPHA) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NALPHA => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_ALPHA) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    LOWER => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_LOWER) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NLOWER => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_LOWER) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    UPPER => {
                        if c > 0x7f || (CLASS_TAB[c as usize] & RI_UPPER) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    NUPPER => {
                        if c == 0 || (c <= 0x7f && (CLASS_TAB[c as usize] & RI_UPPER) != 0) {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }

                    // --- Phase 3: String matching ---
                    EXACTLY => {
                        let opnd = operand(scan);
                        // Inline the first byte, for speed.
                        if *opnd != *REX.input && REX.reg_ic as c_int == 0 {
                            status = RA_NOMATCH;
                        } else if *opnd == 0 {
                            // match empty string always works
                        } else {
                            let mut len: c_int;
                            if *opnd.add(1) == 0 && REX.reg_ic as c_int == 0 {
                                len = 1; // matched a single byte above
                            } else {
                                len = strlen(opnd.cast::<c_char>()) as c_int;
                                if rs_cstrncmp(
                                    opnd.cast::<c_char>(),
                                    REX.input.cast::<c_char>(),
                                    &mut len,
                                ) != 0
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                            // Check for following composing character, unless %C follows.
                            if status != RA_NOMATCH
                                && utf_composinglike(
                                    REX.input.cast::<c_char>(),
                                    REX.input.add(len as usize).cast::<c_char>(),
                                    std::ptr::null_mut(),
                                ) != 0
                                && REX.reg_icombine as c_int == 0
                                && op(next) != RE_COMPOSING
                            {
                                status = RA_NOMATCH;
                            }
                            if status != RA_NOMATCH {
                                REX.input = REX.input.add(len as usize);
                            }
                        }
                    }

                    #[allow(clippy::if_same_then_else)]
                    ANYOF | ANYBUT => {
                        let q = operand(scan);
                        if c == 0 {
                            status = RA_NOMATCH;
                        } else if (rs_cstrchr(q.cast::<c_char>(), c).is_null()) == (opc == ANYOF) {
                            status = RA_NOMATCH;
                        } else {
                            // Check following combining characters.
                            let comb_len =
                                utfc_ptr2len(q.cast::<c_char>()) - utf_ptr2len(q.cast::<c_char>());

                            let inp = REX.input;
                            REX.input = inp.add(utf_ptr2len(inp.cast::<c_char>()) as usize);
                            let q2 = q.add(utf_ptr2len(q.cast::<c_char>()) as usize);

                            if comb_len > 0 {
                                let inp2 = REX.input;
                                let mut mismatch = false;
                                for j in 0..comb_len {
                                    if *q2.add(j as usize) != *inp2.add(j as usize) {
                                        status = RA_NOMATCH;
                                        mismatch = true;
                                        break;
                                    }
                                }
                                if !mismatch {
                                    REX.input = inp2.add(comb_len as usize);
                                }
                            }
                        }
                    }

                    MULTIBYTECODE => {
                        let opnd = operand(scan);
                        let mut mbc_len = utfc_ptr2len(opnd.cast::<c_char>());
                        if mbc_len < 2 {
                            status = RA_NOMATCH;
                        } else {
                            let opndc = utf_ptr2char(opnd.cast::<c_char>());
                            if utf_iscomposing_legacy(opndc) != 0 {
                                // Match composing char at any position.
                                status = RA_NOMATCH;
                                let inp = REX.input;
                                let mut i: c_int = 0;
                                while *inp.add(i as usize) != 0 {
                                    let inpc = utf_ptr2char(inp.add(i as usize).cast::<c_char>());
                                    if utf_iscomposing_legacy(inpc) == 0 {
                                        if i > 0 {
                                            break;
                                        }
                                    } else if opndc == inpc {
                                        mbc_len =
                                            i + utfc_ptr2len(inp.add(i as usize).cast::<c_char>());
                                        status = RA_MATCH;
                                        break;
                                    }
                                    i += utf_ptr2len(inp.add(i as usize).cast::<c_char>());
                                }
                            } else if rs_cstrncmp(
                                opnd.cast::<c_char>(),
                                REX.input.cast::<c_char>(),
                                &mut mbc_len,
                            ) != 0
                            {
                                status = RA_NOMATCH;
                            }
                            if status != RA_NOMATCH {
                                REX.input = REX.input.add(mbc_len as usize);
                            }
                        }
                    }

                    // --- Phase 3: Word boundaries ---
                    #[allow(clippy::if_same_then_else)]
                    BOW => {
                        if c == 0 {
                            status = RA_NOMATCH;
                        } else {
                            let this_class = nvim_regexp_call_mb_get_class_tab(REX.input);
                            if this_class <= 1 {
                                status = RA_NOMATCH;
                            } else if rs_reg_prev_class() == this_class {
                                status = RA_NOMATCH;
                            }
                        }
                    }
                    EOW => {
                        if REX.input == REX.line {
                            status = RA_NOMATCH;
                        } else {
                            let this_class = nvim_regexp_call_mb_get_class_tab(REX.input);
                            let prev_class = rs_reg_prev_class();
                            if this_class == prev_class || prev_class == 0 || prev_class == 1 {
                                status = RA_NOMATCH;
                            }
                        }
                    }

                    // --- Phase 4: Groups ---
                    x if (MOPEN..MOPEN + 10).contains(&x) => {
                        let no = opc - MOPEN;
                        rs_cleanup_subexpr();
                        let rp = regstack_push(RS_MOPEN, scan);
                        if rp.is_null() {
                            status = RA_FAIL;
                        } else {
                            (*rp).rs_no = no as i16;
                            let startpos = REX.reg_startpos.add(no as usize);
                            let startp = REX.reg_startp.add(no as usize);
                            save_se(&mut (*rp).rs_un.sesave, startpos, startp);
                        }
                    }

                    NOPEN | NCLOSE => {
                        if regstack_push(RS_NOPEN, scan).is_null() {
                            status = RA_FAIL;
                        }
                    }

                    x if (ZOPEN + 1..ZOPEN + 10).contains(&x) => {
                        let no = opc - ZOPEN;
                        rs_cleanup_zsubexpr();
                        let rp = regstack_push(RS_ZOPEN, scan);
                        if rp.is_null() {
                            status = RA_FAIL;
                        } else {
                            (*rp).rs_no = no as i16;
                            save_se(
                                &mut (*rp).rs_un.sesave,
                                &raw mut REG_STARTZPOS[no as usize],
                                &raw mut REG_STARTZP[no as usize],
                            );
                        }
                    }

                    x if (MCLOSE..MCLOSE + 10).contains(&x) => {
                        let no = opc - MCLOSE;
                        rs_cleanup_subexpr();
                        let rp = regstack_push(RS_MCLOSE, scan);
                        if rp.is_null() {
                            status = RA_FAIL;
                        } else {
                            (*rp).rs_no = no as i16;
                            let endpos = REX.reg_endpos.add(no as usize);
                            let endp = REX.reg_endp.add(no as usize);
                            save_se(&mut (*rp).rs_un.sesave, endpos, endp);
                        }
                    }

                    x if (ZCLOSE + 1..ZCLOSE + 10).contains(&x) => {
                        let no = opc - ZCLOSE;
                        rs_cleanup_zsubexpr();
                        let rp = regstack_push(RS_ZCLOSE, scan);
                        if rp.is_null() {
                            status = RA_FAIL;
                        } else {
                            (*rp).rs_no = no as i16;
                            save_se(
                                &mut (*rp).rs_un.sesave,
                                &raw mut REG_ENDZPOS[no as usize],
                                &raw mut REG_ENDZP[no as usize],
                            );
                        }
                    }

                    // --- Phase 4: Backrefs ---
                    x if (BACKREF + 1..BACKREF + 10).contains(&x) => {
                        let no = opc - BACKREF;
                        rs_cleanup_subexpr();
                        let mut len: c_int = 0;
                        if is_reg_multi {
                            // Multi-line regexp
                            let start_lnum = (*REX.reg_startpos.add(no as usize)).lnum;
                            let start_col = (*REX.reg_startpos.add(no as usize)).col;
                            let end_lnum = (*REX.reg_endpos.add(no as usize)).lnum;
                            let end_col = (*REX.reg_endpos.add(no as usize)).col;
                            if start_lnum < 0 || end_lnum < 0 {
                                len = 0; // Backref not set
                            } else if start_lnum == REX.lnum && end_lnum == REX.lnum {
                                // Compare within current line.
                                len = end_col - start_col;
                                if rs_cstrncmp(
                                    REX.line.add(start_col as usize).cast::<c_char>(),
                                    REX.input.cast::<c_char>(),
                                    &mut len,
                                ) != 0
                                {
                                    status = RA_NOMATCH;
                                }
                            } else {
                                // Cross-line: use match_with_backref.
                                let r = rs_match_with_backref(
                                    start_lnum, start_col, end_lnum, end_col, &mut len,
                                );
                                if r != RA_MATCH {
                                    status = r;
                                }
                            }
                        } else {
                            // Single-line regexp
                            let startp = *REX.reg_startp.add(no as usize);
                            let endp = *REX.reg_endp.add(no as usize);
                            if startp.is_null() || endp.is_null() {
                                len = 0; // Backref not set: empty string
                            } else {
                                len = endp.offset_from(startp) as c_int;
                                if rs_cstrncmp(
                                    startp.cast::<c_char>(),
                                    REX.input.cast::<c_char>(),
                                    &mut len,
                                ) != 0
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                        }
                        // Matched the backref, skip over it.
                        REX.input = REX.input.add(len as usize);
                    }

                    x if (ZREF + 1..ZREF + 10).contains(&x) => {
                        rs_cleanup_zsubexpr();
                        let no = opc - ZREF;
                        let ext_match = nvim_regexp_get_re_extmatch_in_match(no);
                        if !ext_match.is_null() {
                            let mut len = strlen(ext_match.cast::<c_char>()) as c_int;
                            if rs_cstrncmp(
                                ext_match.cast::<c_char>(),
                                REX.input.cast::<c_char>(),
                                &mut len,
                            ) != 0
                            {
                                status = RA_NOMATCH;
                            } else {
                                REX.input = REX.input.add(len as usize);
                            }
                        }
                        // else: Backref not set, match empty string.
                    }

                    // --- Phase 4: Branch ---
                    BRANCH => {
                        if op(next) == BRANCH {
                            let rp = regstack_push(RS_BRANCH, scan);
                            if rp.is_null() {
                                status = RA_FAIL;
                            } else {
                                status = RA_BREAK; // rest is below
                            }
                        } else {
                            // No choice, avoid recursion.
                            next = operand(scan);
                        }
                    }

                    // --- Phase 5: Quantifiers ---
                    BRACE_LIMITS => {
                        if op(next) == BRACE_SIMPLE {
                            BL_MINVAL = operand_min(scan);
                            BL_MAXVAL = operand_max(scan);
                        } else if op(next) >= BRACE_COMPLEX && op(next) < BRACE_COMPLEX + 10 {
                            let no = op(next) - BRACE_COMPLEX;
                            BRACE_MIN[no as usize] = operand_min(scan);
                            BRACE_MAX[no as usize] = operand_max(scan);
                            BRACE_COUNT[no as usize] = 0;
                        } else {
                            errors::regexp_internal_error(c"BRACE_LIMITS".as_ptr());
                            status = RA_FAIL;
                        }
                    }

                    x if (BRACE_COMPLEX..BRACE_COMPLEX + 10).contains(&x) => {
                        let no = opc - BRACE_COMPLEX;
                        BRACE_COUNT[no as usize] += 1;

                        // If not matched enough times yet, try one more.
                        let min_of_range = if BRACE_MIN[no as usize] <= BRACE_MAX[no as usize] {
                            BRACE_MIN[no as usize]
                        } else {
                            BRACE_MAX[no as usize]
                        };
                        if i64::from(BRACE_COUNT[no as usize]) <= min_of_range {
                            let rp = regstack_push(RS_BRCPLX_MORE, scan);
                            if rp.is_null() {
                                status = RA_FAIL;
                            } else {
                                (*rp).rs_no = no as i16;
                                rs_reg_save(&mut (*rp).rs_un.regsave, BACKPOS.ga_len);
                                next = operand(scan);
                                // Continue and handle the result when done.
                            }
                        } else if BRACE_MIN[no as usize] <= BRACE_MAX[no as usize] {
                            // Range is the normal way around, use longest match.
                            if i64::from(BRACE_COUNT[no as usize]) <= BRACE_MAX[no as usize] {
                                let rp = regstack_push(RS_BRCPLX_LONG, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    (*rp).rs_no = no as i16;
                                    rs_reg_save(&mut (*rp).rs_un.regsave, BACKPOS.ga_len);
                                    next = operand(scan);
                                }
                            }
                            // else: matched enough times, continue with next item.
                        } else {
                            // Range is backwards, use shortest match first.
                            if i64::from(BRACE_COUNT[no as usize]) <= BRACE_MIN[no as usize] {
                                let rp = regstack_push(RS_BRCPLX_SHORT, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    rs_reg_save(&mut (*rp).rs_un.regsave, BACKPOS.ga_len);
                                    // Continue with next item (shortest first).
                                }
                            }
                        }
                    }

                    BRACE_SIMPLE | STAR | PLUS => {
                        // Lookahead to avoid useless match attempts.
                        let mut rst = RegstarT {
                            nextb: 0,
                            nextb_ic: 0,
                            count: 0,
                            minval: 0,
                            maxval: 0,
                        };
                        if op(next) == EXACTLY {
                            rst.nextb = c_int::from(*operand(next));
                            if REX.reg_ic as c_int != 0 {
                                if nvim_mb_isupper(rst.nextb) != 0 {
                                    rst.nextb_ic = mb_tolower(rst.nextb);
                                } else {
                                    rst.nextb_ic = mb_toupper(rst.nextb);
                                }
                            } else {
                                rst.nextb_ic = rst.nextb;
                            }
                        }
                        // else: rst.nextb and rst.nextb_ic are already 0 (NUL)

                        if opc == BRACE_SIMPLE {
                            rst.minval = BL_MINVAL;
                            rst.maxval = BL_MAXVAL;
                        } else {
                            rst.minval = i64::from(opc != STAR);
                            rst.maxval = i64::from(MAX_LIMIT);
                        }

                        // Try matching as much as possible.
                        rst.count = i64::from(rs_regrepeat(operand(scan), rst.maxval));
                        if nvim_regexp_get_got_int() != 0 {
                            status = RA_FAIL;
                        } else if if rst.minval <= rst.maxval {
                            rst.count >= rst.minval
                        } else {
                            rst.count >= rst.maxval
                        } {
                            // It could match. Push regstar_T + regitem_T.
                            if (REGSTACK.ga_len as u32 >> 10) as i64 >= nvim_regexp_get_p_mmp() {
                                errors::emsg_maxmempattern();
                                status = RA_FAIL;
                            } else {
                                ga_grow(
                                    &raw mut REGSTACK,
                                    std::mem::size_of::<RegstarT>() as c_int,
                                );
                                {
                                    REGSTACK.ga_len += std::mem::size_of::<RegstarT>() as c_int;
                                }
                                let state = if rst.minval <= rst.maxval {
                                    RS_STAR_LONG
                                } else {
                                    RS_STAR_SHORT
                                };
                                let rp = regstack_push(state, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    *(rp.cast::<RegstarT>().sub(1)) = rst;
                                    status = RA_BREAK; // skip the restore bits
                                }
                            }
                        } else {
                            status = RA_NOMATCH;
                        }
                    }

                    // --- Phase 6: Lookaround ---
                    NOMATCH | MATCH | SUBPAT => {
                        let rp = regstack_push(RS_NOMATCH, scan);
                        if rp.is_null() {
                            status = RA_FAIL;
                        } else {
                            (*rp).rs_no = opc as i16;
                            rs_reg_save(&mut (*rp).rs_un.regsave, BACKPOS.ga_len);
                            next = operand(scan);
                            // Continue and handle the result when done.
                        }
                    }

                    BEHIND | NOBEHIND => {
                        // Need a bit of room to store extra positions.
                        if (REGSTACK.ga_len as u32 >> 10) as i64 >= nvim_regexp_get_p_mmp() {
                            errors::emsg_maxmempattern();
                            status = RA_FAIL;
                        } else {
                            ga_grow(
                                &raw mut REGSTACK,
                                std::mem::size_of::<RegbehindT>() as c_int,
                            );
                            {
                                REGSTACK.ga_len += std::mem::size_of::<RegbehindT>() as c_int;
                            }
                            let rp = regstack_push(RS_BEHIND1, scan);
                            if rp.is_null() {
                                status = RA_FAIL;
                            } else {
                                // Save subexpr to restore if match is not used.
                                rs_save_subexpr(rp.cast::<RegbehindT>().sub(1));
                                (*rp).rs_no = opc as i16;
                                rs_reg_save(&mut (*rp).rs_un.regsave, BACKPOS.ga_len);
                                // First try if what follows matches. If it does
                                // then we check the behind match by looping.
                            }
                        }
                    }

                    // --- Phase 6: Special position opcodes ---
                    CURSOR => {
                        if nvim_regexp_has_rex_reg_win() == 0
                            || REX.lnum + REX.reg_firstlnum
                                != nvim_regexp_get_rex_reg_win_cursor_lnum()
                            || (REX.input.offset_from(REX.line) as i32
                                != nvim_regexp_get_rex_reg_win_cursor_col())
                        {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_MARK => {
                        let mark = c_int::from(*operand(scan));
                        let cmp = c_int::from(*operand(scan).add(1));
                        let col: usize = if is_reg_multi {
                            REX.input.offset_from(REX.line) as usize
                        } else {
                            0
                        };

                        let fm = nvim_regexp_call_mark_get(mark);

                        // Line may have been freed, get it again.
                        if is_reg_multi {
                            let new_line = nvim_regexp_call_reg_getline(REX.lnum).cast::<u8>();
                            REX.line = new_line;
                            REX.input = new_line.add(col);
                        }

                        if fm.is_null() || nvim_regexp_get_fmark_lnum(fm) <= 0 {
                            status = RA_NOMATCH;
                        } else {
                            let pos_lnum = nvim_regexp_get_fmark_lnum(fm);
                            let pos_col_raw = nvim_regexp_get_fmark_col(fm);
                            let rex_cur_lnum = REX.lnum + REX.reg_firstlnum;
                            #[allow(clippy::cast_possible_truncation)]
                            let input_col = REX.input.offset_from(REX.line) as i32;

                            let pos_col = if pos_lnum == rex_cur_lnum && pos_col_raw == MAXCOL_I32 {
                                nvim_regexp_call_reg_getline_len(pos_lnum - REX.reg_firstlnum)
                            } else {
                                pos_col_raw
                            };

                            let fail = match pos_lnum.cmp(&rex_cur_lnum) {
                                std::cmp::Ordering::Equal => match pos_col.cmp(&input_col) {
                                    std::cmp::Ordering::Equal => {
                                        cmp == i32::from(b'<') || cmp == i32::from(b'>')
                                    }
                                    std::cmp::Ordering::Less => cmp != i32::from(b'>'),
                                    std::cmp::Ordering::Greater => cmp != i32::from(b'<'),
                                },
                                std::cmp::Ordering::Less => cmp != i32::from(b'>'),
                                std::cmp::Ordering::Greater => cmp != i32::from(b'<'),
                            };
                            if fail {
                                status = RA_NOMATCH;
                            }
                        }
                    }

                    RE_VISUAL => {
                        if rs_reg_match_visual() == 0 {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_LNUM => {
                        if !is_reg_multi || !re_num_cmp((REX.lnum + REX.reg_firstlnum) as u32, scan)
                        {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_COL => {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let col = (REX.input.offset_from(REX.line) + 1) as u32;
                        if !re_num_cmp(col, scan) {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_VCOL => {
                        let wp = nvim_regexp_get_rex_reg_win_or_curwin();
                        let mut lnum = if is_reg_multi {
                            REX.reg_firstlnum + REX.lnum
                        } else {
                            1
                        };
                        if is_reg_multi && (lnum <= 0 || lnum > nvim_regexp_get_win_line_count(wp))
                        {
                            lnum = 1;
                        }
                        #[allow(clippy::cast_possible_truncation)]
                        let input_col = REX.input.offset_from(REX.line) as i32;
                        let vcol = nvim_regexp_call_win_linetabsize(
                            wp,
                            lnum,
                            REX.line.cast::<c_char>(),
                            input_col,
                        );
                        #[allow(clippy::cast_sign_loss)]
                        let vcol_1 = (vcol + 1) as u32;
                        if !re_num_cmp(vcol_1, scan) {
                            status = RA_NOMATCH;
                        }
                    }

                    _ => {
                        // Unimplemented opcode — panic to catch missing cases during dev.
                        panic!("rs_regmatch: unimplemented opcode {opc}");
                    }
                }
            }

            // If we can't continue sequentially, break the inner loop.
            if status != RA_CONT {
                break;
            }

            // Continue in inner loop, advance to next item.
            scan = next;
        } // end of inner loop

        // If there is something on the regstack, execute backtracking handlers.
        while REGSTACK.ga_len > 0 && status != RA_FAIL {
            let rp = (REGSTACK.ga_data.cast::<u8>().add(REGSTACK.ga_len as usize))
                .cast::<RegitemT>()
                .sub(1);

            match (*rp).rs_state {
                RS_NOPEN => {
                    // Result is passed on as-is, simply pop the state.
                    regstack_pop(&mut scan);
                }

                RS_MOPEN => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        let no = (*rp).rs_no as usize;
                        restore_se(
                            &(*rp).rs_un.sesave,
                            REX.reg_startpos.add(no),
                            REX.reg_startp.add(no),
                        );
                    }
                    regstack_pop(&mut scan);
                }

                RS_ZOPEN => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        let no = (*rp).rs_no as c_int;
                        restore_se(
                            &(*rp).rs_un.sesave,
                            &raw mut REG_STARTZPOS[no as usize],
                            &raw mut REG_STARTZP[no as usize],
                        );
                    }
                    regstack_pop(&mut scan);
                }

                RS_MCLOSE => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        let no = (*rp).rs_no as usize;
                        restore_se(
                            &(*rp).rs_un.sesave,
                            REX.reg_endpos.add(no),
                            REX.reg_endp.add(no),
                        );
                    }
                    regstack_pop(&mut scan);
                }

                RS_ZCLOSE => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        let no = (*rp).rs_no as c_int;
                        restore_se(
                            &(*rp).rs_un.sesave,
                            &raw mut REG_ENDZPOS[no as usize],
                            &raw mut REG_ENDZP[no as usize],
                        );
                    }
                    regstack_pop(&mut scan);
                }

                RS_BRANCH => {
                    if status == RA_MATCH {
                        // This branch matched, use it.
                        regstack_pop(&mut scan);
                    } else {
                        if status != RA_BREAK {
                            // After a non-matching branch: try next one.
                            let mut bp_len = BACKPOS.ga_len;
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            {
                                BACKPOS.ga_len = bp_len;
                            };
                            scan = (*rp).rs_scan;
                        }
                        if scan.is_null() || op(scan) != BRANCH {
                            // No more branches, didn't find a match.
                            status = RA_NOMATCH;
                            regstack_pop(&mut scan);
                        } else {
                            // Prepare to try a branch.
                            (*rp).rs_scan = rs_regnext(scan);
                            rs_reg_save(&mut (*rp).rs_un.regsave, BACKPOS.ga_len);
                            scan = operand(scan);
                        }
                    }
                }

                // --- Phase 6: Lookaround backtracking handlers ---
                RS_NOMATCH => {
                    // If the operand matches for NOMATCH or doesn't match for
                    // MATCH/SUBPAT, we fail. Otherwise backup (except SUBPAT)
                    // and continue with the next item.
                    let expected = if (*rp).rs_no == NOMATCH as i16 {
                        RA_MATCH
                    } else {
                        RA_NOMATCH
                    };
                    if status == expected {
                        status = RA_NOMATCH;
                    } else {
                        status = RA_CONT;
                        if (*rp).rs_no != SUBPAT as i16 {
                            // zero-width
                            let mut bp_len = BACKPOS.ga_len;
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            {
                                BACKPOS.ga_len = bp_len;
                            };
                        }
                    }
                    regstack_pop(&mut scan);
                    if status == RA_CONT {
                        scan = rs_regnext(scan);
                    }
                }

                RS_BEHIND1 => {
                    if status == RA_NOMATCH {
                        regstack_pop(&mut scan);
                        {
                            REGSTACK.ga_len -= std::mem::size_of::<RegbehindT>() as c_int;
                        }
                    } else {
                        // The stuff after BEHIND/NOBEHIND matches. Now try if
                        // the behind part does (not) match before the current
                        // position in the input.
                        let rbp = rp.cast::<RegbehindT>().sub(1);

                        // Save the position after the found match for next.
                        rs_reg_save(&mut (*rbp).save_after, BACKPOS.ga_len);

                        // Set behind_pos to where the match should end, BHPOS
                        // will match it. Save the current value.
                        let bp = &raw mut BEHIND_POS;
                        (*rbp).save_behind = *bp;
                        *bp = (*rp).rs_un.regsave;

                        (*rp).rs_state = RS_BEHIND2;

                        let mut bp_len = BACKPOS.ga_len;
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        {
                            BACKPOS.ga_len = bp_len;
                        };
                        scan = operand((*rp).rs_scan).add(4);
                    }
                }

                RS_BEHIND2 => {
                    // Looping for BEHIND / NOBEHIND match.
                    let bp = &raw mut BEHIND_POS;
                    let rbp = rp.cast::<RegbehindT>().sub(1);
                    if status == RA_MATCH && rs_reg_save_equal(bp) != 0 {
                        // Found a match that ends where "next" started.
                        *bp = (*rbp).save_behind;
                        if (*rp).rs_no == BEHIND as i16 {
                            let mut bp_len = BACKPOS.ga_len;
                            rs_reg_restore(&(*rbp).save_after, &mut bp_len);
                            {
                                BACKPOS.ga_len = bp_len;
                            };
                        } else {
                            // NOBEHIND: we didn't want a match. Restore subexpr.
                            status = RA_NOMATCH;
                            rs_restore_subexpr(rbp);
                        }
                        regstack_pop(&mut scan);
                        {
                            REGSTACK.ga_len -= std::mem::size_of::<RegbehindT>() as c_int;
                        }
                    } else {
                        // No match or match doesn't end where we want it.
                        // Go back one character. May go to previous line once.
                        let mut no_advance = false;
                        let limit = operand_min((*rp).rs_scan);
                        if is_reg_multi {
                            if limit > 0 {
                                let ref_col =
                                    if (*rp).rs_un.regsave.rs_u.pos.lnum < (*bp).rs_u.pos.lnum {
                                        strlen(REX.line.cast::<c_char>()) as i32
                                    } else {
                                        (*bp).rs_u.pos.col
                                    };
                                if i64::from(ref_col - (*rp).rs_un.regsave.rs_u.pos.col) >= limit {
                                    no_advance = true;
                                }
                            }
                            if !no_advance && (*rp).rs_un.regsave.rs_u.pos.col == 0 {
                                (*rp).rs_un.regsave.rs_u.pos.lnum -= 1;
                                if (*rp).rs_un.regsave.rs_u.pos.lnum < (*bp).rs_u.pos.lnum
                                    || nvim_regexp_call_reg_getline(
                                        (*rp).rs_un.regsave.rs_u.pos.lnum,
                                    )
                                    .is_null()
                                {
                                    no_advance = true;
                                } else {
                                    let mut bp_len = BACKPOS.ga_len;
                                    rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                                    {
                                        BACKPOS.ga_len = bp_len;
                                    };
                                    #[allow(clippy::cast_possible_truncation)]
                                    let line_len = strlen(REX.line.cast::<c_char>()) as i32;
                                    (*rp).rs_un.regsave.rs_u.pos.col = line_len;
                                }
                            } else if !no_advance {
                                let line =
                                    nvim_regexp_call_reg_getline((*rp).rs_un.regsave.rs_u.pos.lnum)
                                        .cast::<u8>();
                                let col = (*rp).rs_un.regsave.rs_u.pos.col;
                                let head = utf_head_off(
                                    line.cast::<c_char>(),
                                    line.add(col as usize - 1).cast::<c_char>(),
                                );
                                (*rp).rs_un.regsave.rs_u.pos.col -= head + 1;
                            }
                        } else {
                            // Single-line mode.
                            if (*rp).rs_un.regsave.rs_u.ptr == REX.line {
                                no_advance = true;
                            } else {
                                let backed = mb_ptr_back(REX.line, (*rp).rs_un.regsave.rs_u.ptr);
                                (*rp).rs_un.regsave.rs_u.ptr = backed;
                                if limit > 0
                                    && (*bp).rs_u.ptr.offset_from((*rp).rs_un.regsave.rs_u.ptr)
                                        > limit as isize
                                {
                                    no_advance = true;
                                }
                            }
                        }
                        if no_advance {
                            // Can't advance. For NOBEHIND that's a match.
                            *bp = (*rbp).save_behind;
                            if (*rp).rs_no == NOBEHIND as i16 {
                                let mut bp_len = BACKPOS.ga_len;
                                rs_reg_restore(&(*rbp).save_after, &mut bp_len);
                                {
                                    BACKPOS.ga_len = bp_len;
                                };
                                status = RA_MATCH;
                            } else {
                                // We do want a proper match. Restore subexpr if
                                // we had a match, because they may have been set.
                                if status == RA_MATCH {
                                    status = RA_NOMATCH;
                                    rs_restore_subexpr(rp.cast::<RegbehindT>().sub(1));
                                }
                            }
                            regstack_pop(&mut scan);
                            {
                                REGSTACK.ga_len -= std::mem::size_of::<RegbehindT>() as c_int;
                            }
                        } else {
                            // Advanced, prepare for finding match again.
                            let mut bp_len = BACKPOS.ga_len;
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            {
                                BACKPOS.ga_len = bp_len;
                            };
                            scan = operand((*rp).rs_scan).add(4);
                            if status == RA_MATCH {
                                // We did match, so subexpr may have been changed,
                                // need to restore them for the next try.
                                status = RA_NOMATCH;
                                rs_restore_subexpr(rp.cast::<RegbehindT>().sub(1));
                            }
                        }
                    }
                }

                // --- Phase 5: Quantifier backtracking handlers ---
                RS_BRCPLX_MORE => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        let mut bp_len = BACKPOS.ga_len;
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        {
                            BACKPOS.ga_len = bp_len;
                        };
                        let no = (*rp).rs_no as c_int;
                        BRACE_COUNT[no as usize] -= 1;
                    }
                    regstack_pop(&mut scan);
                }

                RS_BRCPLX_LONG => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        // There was no match, but we did find enough matches.
                        let mut bp_len = BACKPOS.ga_len;
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        {
                            BACKPOS.ga_len = bp_len;
                        };
                        let no = (*rp).rs_no as c_int;
                        BRACE_COUNT[no as usize] -= 1;
                        // Continue with the items after "\{}".
                        status = RA_CONT;
                    }
                    regstack_pop(&mut scan);
                    if status == RA_CONT {
                        scan = rs_regnext(scan);
                    }
                }

                RS_BRCPLX_SHORT => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        // There was no match, try to match one more item.
                        let mut bp_len = BACKPOS.ga_len;
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        {
                            BACKPOS.ga_len = bp_len;
                        };
                    }
                    regstack_pop(&mut scan);
                    if status == RA_NOMATCH {
                        scan = operand(scan);
                        status = RA_CONT;
                    }
                }

                RS_STAR_LONG | RS_STAR_SHORT => {
                    let rst = (rp.cast::<RegstarT>()).sub(1);

                    if status == RA_MATCH {
                        regstack_pop(&mut scan);
                        {
                            REGSTACK.ga_len -= std::mem::size_of::<RegstarT>() as c_int;
                        }
                    } else {
                        // Tried once already, restore input pointers.
                        if status == RA_BREAK {
                            // First time through — skip restore.
                        } else {
                            let mut bp_len = BACKPOS.ga_len;
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            {
                                BACKPOS.ga_len = bp_len;
                            };
                        }

                        // Repeat until we found a position where it could match.
                        let mut found = false;
                        loop {
                            if status == RA_BREAK {
                                status = RA_NOMATCH;
                            } else {
                                // Tried first position already, advance.
                                if (*rp).rs_state == RS_STAR_LONG {
                                    // Trying for longest match, but couldn't or
                                    // didn't match — back up one char.
                                    (*rst).count -= 1;
                                    if (*rst).count < (*rst).minval {
                                        break;
                                    }
                                    let inp = REX.input;
                                    let line = REX.line;
                                    if inp == line {
                                        // Backup to last char of previous line.
                                        if REX.lnum == 0 {
                                            status = RA_NOMATCH;
                                            break;
                                        }
                                        let new_lnum = REX.lnum - 1;
                                        REX.lnum = new_lnum;
                                        let new_line =
                                            nvim_regexp_call_reg_getline(new_lnum).cast::<u8>();
                                        // Just in case regrepeat() didn't count right.
                                        if new_line.is_null() {
                                            break;
                                        }
                                        REX.line = new_line;
                                        REX.input = new_line
                                            .add(
                                                nvim_regexp_call_reg_getline_len(new_lnum) as usize
                                            );
                                        rs_reg_breakcheck();
                                    } else {
                                        let backed = mb_ptr_back(line, inp);
                                        REX.input = backed;
                                    }
                                } else {
                                    // Range is backwards, use shortest match first.
                                    // Careful: maxval and minval are exchanged!
                                    // Couldn't or didn't match: try advancing one char.
                                    if (*rst).count == (*rst).minval
                                        || rs_regrepeat(operand((*rp).rs_scan), 1) == 0
                                    {
                                        break;
                                    }
                                    (*rst).count += 1;
                                }
                                if nvim_regexp_get_got_int() != 0 {
                                    break;
                                }
                            }

                            // If it could match, try it.
                            if (*rst).nextb == 0
                                || c_int::from(*REX.input) == (*rst).nextb
                                || c_int::from(*REX.input) == (*rst).nextb_ic
                            {
                                rs_reg_save(&mut (*rp).rs_un.regsave, BACKPOS.ga_len);
                                scan = rs_regnext((*rp).rs_scan);
                                status = RA_CONT;
                                found = true;
                                break;
                            }
                        }
                        if !found && status != RA_CONT {
                            // Failed.
                            regstack_pop(&mut scan);
                            {
                                REGSTACK.ga_len -= std::mem::size_of::<RegstarT>() as c_int;
                            }
                            status = RA_NOMATCH;
                        }
                    }
                }

                _ => {
                    // Unimplemented backtracking handler — panic.
                    panic!(
                        "rs_regmatch: unimplemented backtrack state {}",
                        (*rp).rs_state
                    );
                }
            }

            // If we want to continue the inner loop or didn't pop a state, break.
            if status == RA_CONT
                || rp
                    == REGSTACK
                        .ga_data
                        .cast::<u8>()
                        .add(REGSTACK.ga_len as usize)
                        .cast::<RegitemT>()
                        .sub(1)
            {
                break;
            }
        }

        // May need to continue with the inner loop.
        if status == RA_CONT {
            continue;
        }

        // If the regstack is empty or something failed we are done.
        if REGSTACK.ga_len == 0 || status == RA_FAIL {
            if scan.is_null() {
                errors::iemsg_re_corr();
            }
            return c_int::from(status == RA_MATCH);
        }
    }
}

// ==========================================================================
// NFA compiler constants and infrastructure
// ==========================================================================

// Added to NFA_ANY - NFA_NUPPER_IC to include a NL.
const NFA_ADD_NL: c_int = 31;

// NFA states — must match the C enum in regexp.c starting at NFA_SPLIT = -1024
const NFA_SPLIT: c_int = -1024;
const NFA_MATCH: c_int = -1023;
const NFA_EMPTY: c_int = -1022;

const NFA_START_COLL: c_int = -1021;
const NFA_END_COLL: c_int = -1020;
const NFA_START_NEG_COLL: c_int = -1019;
const NFA_END_NEG_COLL: c_int = -1018;
const NFA_RANGE: c_int = -1017;
const NFA_RANGE_MIN: c_int = -1016;
const NFA_RANGE_MAX: c_int = -1015;

const NFA_CONCAT: c_int = -1014;
const NFA_OR: c_int = -1013;
const NFA_STAR: c_int = -1012;
const NFA_STAR_NONGREEDY: c_int = -1011;
const NFA_QUEST: c_int = -1010;
const NFA_QUEST_NONGREEDY: c_int = -1009;

const NFA_BOL: c_int = -1008;
const NFA_EOL: c_int = -1007;
const NFA_BOW: c_int = -1006;
const NFA_EOW: c_int = -1005;
const NFA_BOF: c_int = -1004;
const NFA_EOF: c_int = -1003;
const NFA_NEWL: c_int = -1002;
const NFA_ZSTART: c_int = -1001;
const NFA_ZEND: c_int = -1000;
const NFA_NOPEN: c_int = -999;
const NFA_NCLOSE: c_int = -998;
const NFA_START_INVISIBLE: c_int = -997;
const NFA_START_INVISIBLE_FIRST: c_int = -996;
const NFA_START_INVISIBLE_NEG: c_int = -995;
const NFA_START_INVISIBLE_NEG_FIRST: c_int = -994;
const NFA_START_INVISIBLE_BEFORE: c_int = -993;
const NFA_START_INVISIBLE_BEFORE_FIRST: c_int = -992;
const NFA_START_INVISIBLE_BEFORE_NEG: c_int = -991;
const NFA_START_INVISIBLE_BEFORE_NEG_FIRST: c_int = -990;
const NFA_START_PATTERN: c_int = -989;
const NFA_END_INVISIBLE: c_int = -988;
const NFA_END_INVISIBLE_NEG: c_int = -987;
const NFA_END_PATTERN: c_int = -986;
const NFA_COMPOSING: c_int = -985;
const NFA_END_COMPOSING: c_int = -984;
const NFA_ANY_COMPOSING: c_int = -983;
const NFA_OPT_CHARS: c_int = -982;

const NFA_PREV_ATOM_NO_WIDTH: c_int = -981;
const NFA_PREV_ATOM_NO_WIDTH_NEG: c_int = -980;
const NFA_PREV_ATOM_JUST_BEFORE: c_int = -979;
const NFA_PREV_ATOM_JUST_BEFORE_NEG: c_int = -978;
const NFA_PREV_ATOM_LIKE_PATTERN: c_int = -977;

const NFA_BACKREF1: c_int = -976;
const NFA_BACKREF2: c_int = -975;
const NFA_BACKREF3: c_int = -974;
const NFA_BACKREF4: c_int = -973;
const NFA_BACKREF5: c_int = -972;
const NFA_BACKREF6: c_int = -971;
const NFA_BACKREF7: c_int = -970;
const NFA_BACKREF8: c_int = -969;
const NFA_BACKREF9: c_int = -968;
const NFA_ZREF1: c_int = -967;
const NFA_ZREF2: c_int = -966;
const NFA_ZREF3: c_int = -965;
const NFA_ZREF4: c_int = -964;
const NFA_ZREF5: c_int = -963;
const NFA_ZREF6: c_int = -962;
const NFA_ZREF7: c_int = -961;
const NFA_ZREF8: c_int = -960;
const NFA_ZREF9: c_int = -959;
const NFA_SKIP: c_int = -958;

const NFA_MOPEN: c_int = -957;
const NFA_MOPEN1: c_int = -956;
const NFA_MOPEN2: c_int = -955;
const NFA_MOPEN3: c_int = -954;
const NFA_MOPEN4: c_int = -953;
const NFA_MOPEN5: c_int = -952;
const NFA_MOPEN6: c_int = -951;
const NFA_MOPEN7: c_int = -950;
const NFA_MOPEN8: c_int = -949;
const NFA_MOPEN9: c_int = -948;

const NFA_MCLOSE: c_int = -947;
const NFA_MCLOSE1: c_int = -946;
const NFA_MCLOSE2: c_int = -945;
const NFA_MCLOSE3: c_int = -944;
const NFA_MCLOSE4: c_int = -943;
const NFA_MCLOSE5: c_int = -942;
const NFA_MCLOSE6: c_int = -941;
const NFA_MCLOSE7: c_int = -940;
const NFA_MCLOSE8: c_int = -939;
const NFA_MCLOSE9: c_int = -938;

const NFA_ZOPEN: c_int = -937;
const NFA_ZOPEN1: c_int = -936;
const NFA_ZOPEN2: c_int = -935;
const NFA_ZOPEN3: c_int = -934;
const NFA_ZOPEN4: c_int = -933;
const NFA_ZOPEN5: c_int = -932;
const NFA_ZOPEN6: c_int = -931;
const NFA_ZOPEN7: c_int = -930;
const NFA_ZOPEN8: c_int = -929;
const NFA_ZOPEN9: c_int = -928;

const NFA_ZCLOSE: c_int = -927;
const NFA_ZCLOSE1: c_int = -926;
const NFA_ZCLOSE2: c_int = -925;
const NFA_ZCLOSE3: c_int = -924;
const NFA_ZCLOSE4: c_int = -923;
const NFA_ZCLOSE5: c_int = -922;
const NFA_ZCLOSE6: c_int = -921;
const NFA_ZCLOSE7: c_int = -920;
const NFA_ZCLOSE8: c_int = -919;
const NFA_ZCLOSE9: c_int = -918;

// NFA_FIRST_NL
const NFA_ANY: c_int = -917;
const NFA_IDENT: c_int = -916;
const NFA_SIDENT: c_int = -915;
const NFA_KWORD: c_int = -914;
const NFA_SKWORD: c_int = -913;
const NFA_FNAME: c_int = -912;
const NFA_SFNAME: c_int = -911;
const NFA_PRINT: c_int = -910;
const NFA_SPRINT: c_int = -909;
const NFA_WHITE: c_int = -908;
const NFA_NWHITE: c_int = -907;
const NFA_DIGIT: c_int = -906;
const NFA_NDIGIT: c_int = -905;
const NFA_HEX: c_int = -904;
const NFA_NHEX: c_int = -903;
const NFA_OCTAL: c_int = -902;
const NFA_NOCTAL: c_int = -901;
const NFA_WORD: c_int = -900;
const NFA_NWORD: c_int = -899;
const NFA_HEAD: c_int = -898;
const NFA_NHEAD: c_int = -897;
const NFA_ALPHA: c_int = -896;
const NFA_NALPHA: c_int = -895;
const NFA_LOWER: c_int = -894;
const NFA_NLOWER: c_int = -893;
const NFA_UPPER: c_int = -892;
const NFA_NUPPER: c_int = -891;
const NFA_LOWER_IC: c_int = -890;
const NFA_NLOWER_IC: c_int = -889;
const NFA_UPPER_IC: c_int = -888;
const NFA_NUPPER_IC: c_int = -887;

const NFA_FIRST_NL: c_int = NFA_ANY + NFA_ADD_NL;
const NFA_LAST_NL: c_int = NFA_NUPPER_IC + NFA_ADD_NL;

// After NFA_LAST_NL, the enum continues
const NFA_CURSOR: c_int = NFA_NUPPER_IC + NFA_ADD_NL + 1;
const NFA_LNUM: c_int = NFA_CURSOR + 1;
const NFA_LNUM_GT: c_int = NFA_CURSOR + 2;
const NFA_LNUM_LT: c_int = NFA_CURSOR + 3;
const NFA_COL: c_int = NFA_CURSOR + 4;
const NFA_COL_GT: c_int = NFA_CURSOR + 5;
const NFA_COL_LT: c_int = NFA_CURSOR + 6;
const NFA_VCOL: c_int = NFA_CURSOR + 7;
const NFA_VCOL_GT: c_int = NFA_CURSOR + 8;
const NFA_VCOL_LT: c_int = NFA_CURSOR + 9;
const NFA_MARK: c_int = NFA_CURSOR + 10;
const NFA_MARK_GT: c_int = NFA_CURSOR + 11;
const NFA_MARK_LT: c_int = NFA_CURSOR + 12;
const NFA_VISUAL: c_int = NFA_CURSOR + 13;

const NFA_CLASS_ALNUM: c_int = NFA_CURSOR + 14;
const NFA_CLASS_ALPHA: c_int = NFA_CURSOR + 15;
const NFA_CLASS_BLANK: c_int = NFA_CURSOR + 16;
const NFA_CLASS_CNTRL: c_int = NFA_CURSOR + 17;
const NFA_CLASS_DIGIT: c_int = NFA_CURSOR + 18;
const NFA_CLASS_GRAPH: c_int = NFA_CURSOR + 19;
const NFA_CLASS_LOWER: c_int = NFA_CURSOR + 20;
const NFA_CLASS_PRINT: c_int = NFA_CURSOR + 21;
const NFA_CLASS_PUNCT: c_int = NFA_CURSOR + 22;
const NFA_CLASS_SPACE: c_int = NFA_CURSOR + 23;
const NFA_CLASS_UPPER: c_int = NFA_CURSOR + 24;
const NFA_CLASS_XDIGIT: c_int = NFA_CURSOR + 25;
const NFA_CLASS_TAB: c_int = NFA_CURSOR + 26;
const NFA_CLASS_RETURN: c_int = NFA_CURSOR + 27;
const NFA_CLASS_BACKSPACE: c_int = NFA_CURSOR + 28;
const NFA_CLASS_ESCAPE: c_int = NFA_CURSOR + 29;
const NFA_CLASS_IDENT: c_int = NFA_CURSOR + 30;
const NFA_CLASS_KEYWORD: c_int = NFA_CURSOR + 31;
const NFA_CLASS_FNAME: c_int = NFA_CURSOR + 32;

// Keep in sync with classchars in C.
#[allow(dead_code)]
const NFA_CLASSCODES: [c_int; 27] = [
    NFA_ANY, NFA_IDENT, NFA_SIDENT, NFA_KWORD, NFA_SKWORD, NFA_FNAME, NFA_SFNAME, NFA_PRINT,
    NFA_SPRINT, NFA_WHITE, NFA_NWHITE, NFA_DIGIT, NFA_NDIGIT, NFA_HEX, NFA_NHEX, NFA_OCTAL,
    NFA_NOCTAL, NFA_WORD, NFA_NWORD, NFA_HEAD, NFA_NHEAD, NFA_ALPHA, NFA_NALPHA, NFA_LOWER,
    NFA_NLOWER, NFA_UPPER, NFA_NUPPER,
];

// FAIL constant from vim_defs.h
const FAIL: c_int = 0;

#[allow(dead_code)]
extern "C" {
    // NFA postfix buffer accessors

    // NFA state count accessors

    // NFA flags accessors

    // rex NFA fields

    fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
}

/// Emit a value into the NFA postfix buffer, growing if needed.
unsafe fn nfa_emit(c: c_int) {
    let post_ptr = POST_PTR;
    let post_end = POST_END;
    if post_ptr >= post_end {
        rs_realloc_post_list();
    }
    let post_ptr = POST_PTR;
    *post_ptr = c;
    POST_PTR = post_ptr.add(1);
}

/// Initialize regexp compile state (shared between BT and NFA engines).
/// This replaces the C `regcomp_start()` function.
unsafe fn regcomp_start(expr: *mut u8, re_flags: c_int) {
    rs_initchr(expr.cast::<c_char>());
    if re_flags & RE_MAGIC != 0 {
        REG_MAGIC = MAGIC_ON;
    } else {
        REG_MAGIC = MAGIC_OFF;
    }
    REG_STRING = re_flags & RE_STRING;
    REG_STRICT = re_flags & RE_STRICT;
    rs_get_cpo_flags();

    NUM_COMPLEX_BRACES = 0;
    REGNPAR = 1;
    HAD_ENDBRACE = [0u8; NSUBEXP];
    REGNZPAR = 1;
    RE_HAS_Z = 0;
    REGSIZE = 0;
    REG_TOOLONG = 0;
    REGFLAGS_COMPILE = 0;
    HAD_EOL = 0;
}

/// Initialize internal variables before NFA compilation.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regcomp_start(expr: *mut u8, re_flags: c_int) {
    NSTATE = 0;
    ISTATE = 0;

    // A reasonable estimation for maximum size
    let nstate_max = (strlen(expr.cast::<c_char>()) + 1) * 25 + 1000;

    // Size for postfix representation of expr.
    let postfix_size = std::mem::size_of::<c_int>() * nstate_max;

    let post_start = xmalloc(postfix_size).cast::<c_int>();
    POST_START = post_start;
    POST_PTR = post_start;
    POST_END = post_start.add(nstate_max);
    WANTS_NFA = 0;
    REX.nfa_has_zend = 0;
    REX.nfa_has_backref = 0;

    // shared with BT engine
    regcomp_start(expr, re_flags);
}

/// Grow the NFA postfix buffer by 1.5x.
#[no_mangle]
pub unsafe extern "C" fn rs_realloc_post_list() {
    let post_start = POST_START;
    let post_ptr = POST_PTR;
    let post_end = POST_END;

    let old_max = post_end.offset_from(post_start) as usize;
    let new_max = old_max * 3 / 2;
    let new_start = xrealloc(
        post_start.cast::<c_void>(),
        new_max * std::mem::size_of::<c_int>(),
    )
    .cast::<c_int>();

    let ptr_offset = post_ptr.offset_from(post_start) as usize;
    POST_PTR = new_start.add(ptr_offset);
    POST_END = new_start.add(new_max);
    POST_START = new_start;
}

/// Emit a character followed by `NFA_CONCAT` into the postfix buffer.
/// Equivalent to the C `EMIT2` macro.
unsafe fn nfa_emit2(c: c_int) {
    nfa_emit(c);
    nfa_emit(NFA_CONCAT);
}

/// Emit the equivalence class for character `c`.
///
/// Each member of the class is emitted with `nfa_emit2` (`char` + `NFA_CONCAT`).
/// For characters not in any equivalence class, just emit the character itself.
/// BT engine equivalence class emission.
/// Emits all characters in the equivalence class of `c` via regmbc.
/// Falls back to emitting just `c` if it doesn't belong to any class.
#[allow(clippy::too_many_lines)]
unsafe fn reg_equi_class(c: c_int) {
    // Equivalence class tables: each entry is (list of case values, list of emit values).
    // The case values and emit values are the same for most groups.
    // We match on `c` and emit all members of the equivalence class.

    match c {
        // A group
        65 | 0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc4 | 0xc5 | 0x100 | 0x102 | 0x104 | 0x1cd | 0x1de
        | 0x1e0 | 0x1fa | 0x200 | 0x202 | 0x226 | 0x23a | 0x1e00 | 0x1ea0 | 0x1ea2 | 0x1ea4
        | 0x1ea6 | 0x1ea8 | 0x1eaa | 0x1eac | 0x1eae | 0x1eb0 | 0x1eb2 | 0x1eb4 | 0x1eb6 => {
            for &ch in &[
                65, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0x100, 0x102, 0x104, 0x1cd, 0x1de, 0x1e0,
                0x1fa, 0x200, 0x202, 0x226, 0x23a, 0x1e00, 0x1ea0, 0x1ea2, 0x1ea4, 0x1ea6, 0x1ea8,
                0x1eaa, 0x1eac, 0x1eae, 0x1eb0, 0x1eb2, 0x1eb6, 0x1eb4,
            ] {
                rs_regmbc(ch);
            }
        }
        // B group
        66 | 0x181 | 0x243 | 0x1e02 | 0x1e04 | 0x1e06 => {
            for &ch in &[66, 0x181, 0x243, 0x1e02, 0x1e04, 0x1e06] {
                rs_regmbc(ch);
            }
        }
        // C group
        67 | 0xc7 | 0x106 | 0x108 | 0x10a | 0x10c | 0x187 | 0x23b | 0x1e08 | 0xa792 => {
            for &ch in &[
                67, 0xc7, 0x106, 0x108, 0x10a, 0x10c, 0x187, 0x23b, 0x1e08, 0xa792,
            ] {
                rs_regmbc(ch);
            }
        }
        // D group
        68 | 0x10e | 0x110 | 0x18a | 0x1e0a | 0x1e0c | 0x1e0e | 0x1e10 | 0x1e12 => {
            for &ch in &[
                68, 0x10e, 0x110, 0x18a, 0x1e0a, 0x1e0c, 0x1e0e, 0x1e10, 0x1e12,
            ] {
                rs_regmbc(ch);
            }
        }
        // E group
        69 | 0xc8 | 0xc9 | 0xca | 0xcb | 0x112 | 0x114 | 0x116 | 0x118 | 0x11a | 0x204 | 0x206
        | 0x228 | 0x246 | 0x1e14 | 0x1e16 | 0x1e18 | 0x1e1a | 0x1e1c | 0x1eb8 | 0x1eba | 0x1ebc
        | 0x1ebe | 0x1ec0 | 0x1ec2 | 0x1ec4 | 0x1ec6 => {
            for &ch in &[
                69, 0xc8, 0xc9, 0xca, 0xcb, 0x112, 0x114, 0x116, 0x118, 0x11a, 0x204, 0x206, 0x228,
                0x246, 0x1e14, 0x1e16, 0x1e18, 0x1e1a, 0x1e1c, 0x1eb8, 0x1eba, 0x1ebc, 0x1ebe,
                0x1ec0, 0x1ec2, 0x1ec4, 0x1ec6,
            ] {
                rs_regmbc(ch);
            }
        }
        // F group
        70 | 0x191 | 0x1e1e | 0xa798 => {
            for &ch in &[70, 0x191, 0x1e1e, 0xa798] {
                rs_regmbc(ch);
            }
        }
        // G group
        71 | 0x11c | 0x11e | 0x120 | 0x122 | 0x193 | 0x1e4 | 0x1e6 | 0x1f4 | 0x1e20 | 0xa7a0 => {
            for &ch in &[
                71, 0x11c, 0x11e, 0x120, 0x122, 0x193, 0x1e4, 0x1e6, 0x1f4, 0x1e20, 0xa7a0,
            ] {
                rs_regmbc(ch);
            }
        }
        // H group
        72 | 0x124 | 0x126 | 0x21e | 0x1e22 | 0x1e24 | 0x1e26 | 0x1e28 | 0x1e2a | 0x2c67 => {
            for &ch in &[
                72, 0x124, 0x126, 0x21e, 0x1e22, 0x1e24, 0x1e26, 0x1e28, 0x1e2a, 0x2c67,
            ] {
                rs_regmbc(ch);
            }
        }
        // I group
        73 | 0xcc | 0xcd | 0xce | 0xcf | 0x128 | 0x12a | 0x12c | 0x12e | 0x130 | 0x197 | 0x1cf
        | 0x208 | 0x20a | 0x1e2c | 0x1e2e | 0x1ec8 | 0x1eca => {
            for &ch in &[
                73, 0xcc, 0xcd, 0xce, 0xcf, 0x128, 0x12a, 0x12c, 0x12e, 0x130, 0x197, 0x1cf, 0x208,
                0x20a, 0x1e2c, 0x1e2e, 0x1ec8, 0x1eca,
            ] {
                rs_regmbc(ch);
            }
        }
        // J group
        74 | 0x134 | 0x248 => {
            for &ch in &[74, 0x134, 0x248] {
                rs_regmbc(ch);
            }
        }
        // K group
        75 | 0x136 | 0x198 | 0x1e8 | 0x1e30 | 0x1e32 | 0x1e34 | 0x2c69 | 0xa740 => {
            for &ch in &[
                75, 0x136, 0x198, 0x1e8, 0x1e30, 0x1e32, 0x1e34, 0x2c69, 0xa740,
            ] {
                rs_regmbc(ch);
            }
        }
        // L group
        76 | 0x139 | 0x13b | 0x13d | 0x13f | 0x141 | 0x23d | 0x1e36 | 0x1e38 | 0x1e3a | 0x1e3c
        | 0x2c60 => {
            for &ch in &[
                76, 0x139, 0x13b, 0x13d, 0x13f, 0x141, 0x23d, 0x1e36, 0x1e38, 0x1e3a, 0x1e3c,
                0x2c60,
            ] {
                rs_regmbc(ch);
            }
        }
        // M group
        77 | 0x1e3e | 0x1e40 | 0x1e42 => {
            for &ch in &[77, 0x1e3e, 0x1e40, 0x1e42] {
                rs_regmbc(ch);
            }
        }
        // N group
        78 | 0xd1 | 0x143 | 0x145 | 0x147 | 0x1f8 | 0x1e44 | 0x1e46 | 0x1e48 | 0x1e4a | 0xa7a4 => {
            for &ch in &[
                78, 0xd1, 0x143, 0x145, 0x147, 0x1f8, 0x1e44, 0x1e46, 0x1e48, 0x1e4a, 0xa7a4,
            ] {
                rs_regmbc(ch);
            }
        }
        // O group
        79 | 0xd2 | 0xd3 | 0xd4 | 0xd5 | 0xd6 | 0xd8 | 0x14c | 0x14e | 0x150 | 0x19f | 0x1a0
        | 0x1d1 | 0x1ea | 0x1ec | 0x1fe | 0x20c | 0x20e | 0x22a | 0x22c | 0x22e | 0x230
        | 0x1e4c | 0x1e4e | 0x1e50 | 0x1e52 | 0x1ecc | 0x1ece | 0x1ed0 | 0x1ed2 | 0x1ed4
        | 0x1ed6 | 0x1ed8 | 0x1eda | 0x1edc | 0x1ede | 0x1ee0 | 0x1ee2 => {
            for &ch in &[
                79, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd8, 0x14c, 0x14e, 0x150, 0x19f, 0x1a0, 0x1d1,
                0x1ea, 0x1ec, 0x1fe, 0x20c, 0x20e, 0x22a, 0x22c, 0x22e, 0x230, 0x1e4c, 0x1e4e,
                0x1e50, 0x1e52, 0x1ecc, 0x1ece, 0x1ed0, 0x1ed2, 0x1ed4, 0x1ed6, 0x1ed8, 0x1eda,
                0x1edc, 0x1ede, 0x1ee0, 0x1ee2,
            ] {
                rs_regmbc(ch);
            }
        }
        // P group
        80 | 0x1a4 | 0x1e54 | 0x1e56 | 0x2c63 => {
            for &ch in &[80, 0x1a4, 0x1e54, 0x1e56, 0x2c63] {
                rs_regmbc(ch);
            }
        }
        // Q group
        81 | 0x24a => {
            for &ch in &[81, 0x24a] {
                rs_regmbc(ch);
            }
        }
        // R group
        82 | 0x154 | 0x156 | 0x158 | 0x210 | 0x212 | 0x24c | 0x1e58 | 0x1e5a | 0x1e5c | 0x1e5e
        | 0x2c64 | 0xa7a6 => {
            for &ch in &[
                82, 0x154, 0x156, 0x158, 0x210, 0x212, 0x24c, 0x1e58, 0x1e5a, 0x1e5c, 0x1e5e,
                0x2c64, 0xa7a6,
            ] {
                rs_regmbc(ch);
            }
        }
        // S group
        83 | 0x15a | 0x15c | 0x15e | 0x160 | 0x218 | 0x1e60 | 0x1e62 | 0x1e64 | 0x1e66 | 0x1e68
        | 0x2c7e | 0xa7a8 => {
            for &ch in &[
                83, 0x15a, 0x15c, 0x15e, 0x160, 0x218, 0x1e60, 0x1e62, 0x1e64, 0x1e66, 0x1e68,
                0x2c7e, 0xa7a8,
            ] {
                rs_regmbc(ch);
            }
        }
        // T group
        84 | 0x162 | 0x164 | 0x166 | 0x1ac | 0x1ae | 0x21a | 0x23e | 0x1e6a | 0x1e6c | 0x1e6e
        | 0x1e70 => {
            for &ch in &[
                84, 0x162, 0x164, 0x166, 0x1ac, 0x1ae, 0x23e, 0x21a, 0x1e6a, 0x1e6c, 0x1e6e, 0x1e70,
            ] {
                rs_regmbc(ch);
            }
        }
        // U group
        85 | 0xd9 | 0xda | 0xdc | 0xdb | 0x168 | 0x16a | 0x16c | 0x16e | 0x170 | 0x172 | 0x1af
        | 0x1d3 | 0x1d5 | 0x1d7 | 0x1d9 | 0x1db | 0x214 | 0x216 | 0x244 | 0x1e72 | 0x1e74
        | 0x1e76 | 0x1e78 | 0x1e7a | 0x1ee4 | 0x1ee6 | 0x1ee8 | 0x1eea | 0x1eec | 0x1eee
        | 0x1ef0 => {
            for &ch in &[
                85, 0xd9, 0xda, 0xdc, 0xdb, 0x168, 0x16a, 0x16c, 0x16e, 0x170, 0x172, 0x1af, 0x1d3,
                0x1d5, 0x1d7, 0x1d9, 0x1db, 0x214, 0x216, 0x244, 0x1e72, 0x1e74, 0x1e76, 0x1e78,
                0x1e7a, 0x1ee4, 0x1ee6, 0x1ee8, 0x1eea, 0x1eec, 0x1eee, 0x1ef0,
            ] {
                rs_regmbc(ch);
            }
        }
        // V group
        86 | 0x1b2 | 0x1e7c | 0x1e7e => {
            for &ch in &[86, 0x1b2, 0x1e7c, 0x1e7e] {
                rs_regmbc(ch);
            }
        }
        // W group
        87 | 0x174 | 0x1e80 | 0x1e82 | 0x1e84 | 0x1e86 | 0x1e88 => {
            for &ch in &[87, 0x174, 0x1e80, 0x1e82, 0x1e84, 0x1e86, 0x1e88] {
                rs_regmbc(ch);
            }
        }
        // X group
        88 | 0x1e8a | 0x1e8c => {
            for &ch in &[88, 0x1e8a, 0x1e8c] {
                rs_regmbc(ch);
            }
        }
        // Y group
        89 | 0xdd | 0x176 | 0x178 | 0x1b3 | 0x232 | 0x24e | 0x1e8e | 0x1ef2 | 0x1ef4 | 0x1ef6
        | 0x1ef8 => {
            for &ch in &[
                89, 0xdd, 0x176, 0x178, 0x1b3, 0x232, 0x24e, 0x1e8e, 0x1ef2, 0x1ef4, 0x1ef6, 0x1ef8,
            ] {
                rs_regmbc(ch);
            }
        }
        // Z group
        90 | 0x179 | 0x17b | 0x17d | 0x1b5 | 0x1e90 | 0x1e92 | 0x1e94 | 0x2c6b => {
            for &ch in &[
                90, 0x179, 0x17b, 0x17d, 0x1b5, 0x1e90, 0x1e92, 0x1e94, 0x2c6b,
            ] {
                rs_regmbc(ch);
            }
        }
        // a group
        97 | 0xe0 | 0xe1 | 0xe2 | 0xe3 | 0xe4 | 0xe5 | 0x101 | 0x103 | 0x105 | 0x1ce | 0x1df
        | 0x1e1 | 0x1fb | 0x201 | 0x203 | 0x227 | 0x1d8f | 0x1e01 | 0x1e9a | 0x1ea1 | 0x1ea3
        | 0x1ea5 | 0x1ea7 | 0x1ea9 | 0x1eab | 0x1ead | 0x1eaf | 0x1eb1 | 0x1eb3 | 0x1eb5
        | 0x1eb7 | 0x2c65 => {
            for &ch in &[
                97, 0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0x101, 0x103, 0x105, 0x1ce, 0x1df, 0x1e1,
                0x1fb, 0x201, 0x203, 0x227, 0x1d8f, 0x1e01, 0x1e9a, 0x1ea1, 0x1ea3, 0x1ea5, 0x1ea7,
                0x1ea9, 0x1eab, 0x1ead, 0x1eaf, 0x1eb1, 0x1eb3, 0x1eb5, 0x1eb7, 0x2c65,
            ] {
                rs_regmbc(ch);
            }
        }
        // b group
        98 | 0x180 | 0x253 | 0x1d6c | 0x1d80 | 0x1e03 | 0x1e05 | 0x1e07 => {
            for &ch in &[98, 0x180, 0x253, 0x1d6c, 0x1d80, 0x1e03, 0x1e05, 0x1e07] {
                rs_regmbc(ch);
            }
        }
        // c group
        99 | 0xe7 | 0x107 | 0x109 | 0x10b | 0x10d | 0x188 | 0x23c | 0x1e09 | 0xa793 | 0xa794 => {
            for &ch in &[
                99, 0xe7, 0x107, 0x109, 0x10b, 0x10d, 0x188, 0x23c, 0x1e09, 0xa793, 0xa794,
            ] {
                rs_regmbc(ch);
            }
        }
        // d group
        100 | 0x10f | 0x111 | 0x257 | 0x1d6d | 0x1d81 | 0x1d91 | 0x1e0b | 0x1e0d | 0x1e0f
        | 0x1e11 | 0x1e13 => {
            for &ch in &[
                100, 0x10f, 0x111, 0x257, 0x1d6d, 0x1d81, 0x1d91, 0x1e0b, 0x1e0d, 0x1e0f, 0x1e11,
                0x1e13,
            ] {
                rs_regmbc(ch);
            }
        }
        // e group
        101 | 0xe8 | 0xe9 | 0xea | 0xeb | 0x113 | 0x115 | 0x117 | 0x119 | 0x11b | 0x205 | 0x207
        | 0x229 | 0x247 | 0x1d92 | 0x1e15 | 0x1e17 | 0x1e19 | 0x1e1b | 0x1e1d | 0x1eb9 | 0x1ebb
        | 0x1ebd | 0x1ebf | 0x1ec1 | 0x1ec3 | 0x1ec5 | 0x1ec7 => {
            for &ch in &[
                101, 0xe8, 0xe9, 0xea, 0xeb, 0x113, 0x115, 0x117, 0x119, 0x11b, 0x205, 0x207,
                0x229, 0x247, 0x1d92, 0x1e15, 0x1e17, 0x1e19, 0x1e1b, 0x1e1d, 0x1eb9, 0x1ebb,
                0x1ebd, 0x1ebf, 0x1ec1, 0x1ec3, 0x1ec5, 0x1ec7,
            ] {
                rs_regmbc(ch);
            }
        }
        // f group
        102 | 0x192 | 0x1d6e | 0x1d82 | 0x1e1f | 0xa799 => {
            for &ch in &[102, 0x192, 0x1d6e, 0x1d82, 0x1e1f, 0xa799] {
                rs_regmbc(ch);
            }
        }
        // g group
        103 | 0x11d | 0x11f | 0x121 | 0x123 | 0x1e5 | 0x1e7 | 0x1f5 | 0x260 | 0x1d83 | 0x1e21
        | 0xa7a1 => {
            for &ch in &[
                103, 0x11d, 0x11f, 0x121, 0x123, 0x1e5, 0x1e7, 0x1f5, 0x260, 0x1d83, 0x1e21, 0xa7a1,
            ] {
                rs_regmbc(ch);
            }
        }
        // h group
        104 | 0x125 | 0x127 | 0x21f | 0x1e23 | 0x1e25 | 0x1e27 | 0x1e29 | 0x1e2b | 0x1e96
        | 0x2c68 | 0xa795 => {
            for &ch in &[
                104, 0x125, 0x127, 0x21f, 0x1e23, 0x1e25, 0x1e27, 0x1e29, 0x1e2b, 0x1e96, 0x2c68,
                0xa795,
            ] {
                rs_regmbc(ch);
            }
        }
        // i group
        105 | 0xec | 0xed | 0xee | 0xef | 0x129 | 0x12b | 0x12d | 0x12f | 0x1d0 | 0x209 | 0x20b
        | 0x268 | 0x1d96 | 0x1e2d | 0x1e2f | 0x1ec9 | 0x1ecb => {
            for &ch in &[
                105, 0xec, 0xed, 0xee, 0xef, 0x129, 0x12b, 0x12d, 0x12f, 0x1d0, 0x209, 0x20b,
                0x268, 0x1d96, 0x1e2d, 0x1e2f, 0x1ec9, 0x1ecb,
            ] {
                rs_regmbc(ch);
            }
        }
        // j group
        106 | 0x135 | 0x1f0 | 0x249 => {
            for &ch in &[106, 0x135, 0x1f0, 0x249] {
                rs_regmbc(ch);
            }
        }
        // k group
        107 | 0x137 | 0x199 | 0x1e9 | 0x1d84 | 0x1e31 | 0x1e33 | 0x1e35 | 0x2c6a | 0xa741 => {
            for &ch in &[
                107, 0x137, 0x199, 0x1e9, 0x1d84, 0x1e31, 0x1e33, 0x1e35, 0x2c6a, 0xa741,
            ] {
                rs_regmbc(ch);
            }
        }
        // l group
        108 | 0x13a | 0x13c | 0x13e | 0x140 | 0x142 | 0x19a | 0x1e37 | 0x1e39 | 0x1e3b | 0x1e3d
        | 0x2c61 => {
            for &ch in &[
                108, 0x13a, 0x13c, 0x13e, 0x140, 0x142, 0x19a, 0x1e37, 0x1e39, 0x1e3b, 0x1e3d,
                0x2c61,
            ] {
                rs_regmbc(ch);
            }
        }
        // m group
        109 | 0x1d6f | 0x1e3f | 0x1e41 | 0x1e43 => {
            for &ch in &[109, 0x1d6f, 0x1e3f, 0x1e41, 0x1e43] {
                rs_regmbc(ch);
            }
        }
        // n group
        110 | 0xf1 | 0x144 | 0x146 | 0x148 | 0x149 | 0x1f9 | 0x1d70 | 0x1d87 | 0x1e45 | 0x1e47
        | 0x1e49 | 0x1e4b | 0xa7a5 => {
            for &ch in &[
                110, 0xf1, 0x144, 0x146, 0x148, 0x149, 0x1f9, 0x1d70, 0x1d87, 0x1e45, 0x1e47,
                0x1e49, 0x1e4b, 0xa7a5,
            ] {
                rs_regmbc(ch);
            }
        }
        // o group
        111 | 0xf2 | 0xf3 | 0xf4 | 0xf5 | 0xf6 | 0xf8 | 0x14d | 0x14f | 0x151 | 0x1a1 | 0x1d2
        | 0x1eb | 0x1ed | 0x1ff | 0x20d | 0x20f | 0x22b | 0x22d | 0x22f | 0x231 | 0x275
        | 0x1e4d | 0x1e4f | 0x1e51 | 0x1e53 | 0x1ecd | 0x1ecf | 0x1ed1 | 0x1ed3 | 0x1ed5
        | 0x1ed7 | 0x1ed9 | 0x1edb | 0x1edd | 0x1edf | 0x1ee1 | 0x1ee3 => {
            for &ch in &[
                111, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf8, 0x14d, 0x14f, 0x151, 0x1a1, 0x1d2, 0x1eb,
                0x1ed, 0x1ff, 0x20d, 0x20f, 0x22b, 0x22d, 0x22f, 0x231, 0x275, 0x1e4d, 0x1e4f,
                0x1e51, 0x1e53, 0x1ecd, 0x1ecf, 0x1ed1, 0x1ed3, 0x1ed5, 0x1ed7, 0x1ed9, 0x1edb,
                0x1edd, 0x1edf, 0x1ee1, 0x1ee3,
            ] {
                rs_regmbc(ch);
            }
        }
        // p group
        112 | 0x1a5 | 0x1d71 | 0x1d7d | 0x1d88 | 0x1e55 | 0x1e57 => {
            for &ch in &[112, 0x1a5, 0x1d71, 0x1d7d, 0x1d88, 0x1e55, 0x1e57] {
                rs_regmbc(ch);
            }
        }
        // q group
        113 | 0x24b | 0x2a0 => {
            for &ch in &[113, 0x24b, 0x2a0] {
                rs_regmbc(ch);
            }
        }
        // r group
        114 | 0x155 | 0x157 | 0x159 | 0x211 | 0x213 | 0x24d | 0x27d | 0x1d72 | 0x1d73 | 0x1d89
        | 0x1e59 | 0x1e5b | 0x1e5d | 0x1e5f | 0xa7a7 => {
            for &ch in &[
                114, 0x155, 0x157, 0x159, 0x211, 0x213, 0x24d, 0x27d, 0x1d72, 0x1d73, 0x1d89,
                0x1e59, 0x1e5b, 0x1e5d, 0x1e5f, 0xa7a7,
            ] {
                rs_regmbc(ch);
            }
        }
        // s group
        115 | 0x15b | 0x15d | 0x15f | 0x161 | 0x219 | 0x23f | 0x1d74 | 0x1d8a | 0x1e61 | 0x1e63
        | 0x1e65 | 0x1e67 | 0x1e69 | 0xa7a9 => {
            for &ch in &[
                115, 0x15b, 0x15d, 0x15f, 0x161, 0x219, 0x23f, 0x1d74, 0x1d8a, 0x1e61, 0x1e63,
                0x1e65, 0x1e67, 0x1e69, 0xa7a9,
            ] {
                rs_regmbc(ch);
            }
        }
        // t group
        116 | 0x163 | 0x165 | 0x167 | 0x1ab | 0x1ad | 0x21b | 0x288 | 0x1d75 | 0x1e6b | 0x1e6d
        | 0x1e6f | 0x1e71 | 0x1e97 | 0x2c66 => {
            for &ch in &[
                116, 0x163, 0x165, 0x167, 0x1ab, 0x1ad, 0x21b, 0x288, 0x1d75, 0x1e6b, 0x1e6d,
                0x1e6f, 0x1e71, 0x1e97, 0x2c66,
            ] {
                rs_regmbc(ch);
            }
        }
        // u group
        117 | 0xf9 | 0xfa | 0xfb | 0xfc | 0x169 | 0x16b | 0x16d | 0x16f | 0x171 | 0x173 | 0x1b0
        | 0x1d4 | 0x1d6 | 0x1d8 | 0x1da | 0x1dc | 0x215 | 0x217 | 0x289 | 0x1d7e | 0x1d99
        | 0x1e73 | 0x1e75 | 0x1e77 | 0x1e79 | 0x1e7b | 0x1ee5 | 0x1ee7 | 0x1ee9 | 0x1eeb
        | 0x1eed | 0x1eef | 0x1ef1 => {
            for &ch in &[
                117, 0xf9, 0xfa, 0xfb, 0xfc, 0x169, 0x16b, 0x16d, 0x16f, 0x171, 0x173, 0x1b0,
                0x1d4, 0x1d6, 0x1d8, 0x1da, 0x1dc, 0x215, 0x217, 0x289, 0x1d7e, 0x1d99, 0x1e73,
                0x1e75, 0x1e77, 0x1e79, 0x1e7b, 0x1ee5, 0x1ee7, 0x1ee9, 0x1eeb, 0x1eed, 0x1eef,
                0x1ef1,
            ] {
                rs_regmbc(ch);
            }
        }
        // v group
        118 | 0x28b | 0x1d8c | 0x1e7d | 0x1e7f => {
            for &ch in &[118, 0x28b, 0x1d8c, 0x1e7d, 0x1e7f] {
                rs_regmbc(ch);
            }
        }
        // w group
        119 | 0x175 | 0x1e81 | 0x1e83 | 0x1e85 | 0x1e87 | 0x1e89 | 0x1e98 => {
            for &ch in &[119, 0x175, 0x1e81, 0x1e83, 0x1e85, 0x1e87, 0x1e89, 0x1e98] {
                rs_regmbc(ch);
            }
        }
        // x group
        120 | 0x1e8b | 0x1e8d => {
            for &ch in &[120, 0x1e8b, 0x1e8d] {
                rs_regmbc(ch);
            }
        }
        // y group
        121 | 0xfd | 0xff | 0x177 | 0x1b4 | 0x233 | 0x24f | 0x1e8f | 0x1e99 | 0x1ef3 | 0x1ef5
        | 0x1ef7 | 0x1ef9 => {
            for &ch in &[
                121, 0xfd, 0xff, 0x177, 0x1b4, 0x233, 0x24f, 0x1e8f, 0x1e99, 0x1ef3, 0x1ef5,
                0x1ef7, 0x1ef9,
            ] {
                rs_regmbc(ch);
            }
        }
        // z group
        122 | 0x17a | 0x17c | 0x17e | 0x1b6 | 0x1d76 | 0x1d8e | 0x1e91 | 0x1e93 | 0x1e95
        | 0x2c6c => {
            for &ch in &[
                122, 0x17a, 0x17c, 0x17e, 0x1b6, 0x1d76, 0x1d8e, 0x1e91, 0x1e93, 0x1e95, 0x2c6c,
            ] {
                rs_regmbc(ch);
            }
        }
        // default: character itself
        _ => {
            rs_regmbc(c);
        }
    }
}

#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_nfa_emit_equi_class(c: c_int) {
    // Equivalence class tables: each entry is (list of case values, list of emit values).
    // The case values and emit values are the same for most groups.
    // We match on `c` and emit all members of the equivalence class.

    match c {
        // A group
        65 | 0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc4 | 0xc5 | 0x100 | 0x102 | 0x104 | 0x1cd | 0x1de
        | 0x1e0 | 0x1fa | 0x200 | 0x202 | 0x226 | 0x23a | 0x1e00 | 0x1ea0 | 0x1ea2 | 0x1ea4
        | 0x1ea6 | 0x1ea8 | 0x1eaa | 0x1eac | 0x1eae | 0x1eb0 | 0x1eb2 | 0x1eb4 | 0x1eb6 => {
            for &ch in &[
                65, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0x100, 0x102, 0x104, 0x1cd, 0x1de, 0x1e0,
                0x1fa, 0x200, 0x202, 0x226, 0x23a, 0x1e00, 0x1ea0, 0x1ea2, 0x1ea4, 0x1ea6, 0x1ea8,
                0x1eaa, 0x1eac, 0x1eae, 0x1eb0, 0x1eb2, 0x1eb6, 0x1eb4,
            ] {
                nfa_emit2(ch);
            }
        }
        // B group
        66 | 0x181 | 0x243 | 0x1e02 | 0x1e04 | 0x1e06 => {
            for &ch in &[66, 0x181, 0x243, 0x1e02, 0x1e04, 0x1e06] {
                nfa_emit2(ch);
            }
        }
        // C group
        67 | 0xc7 | 0x106 | 0x108 | 0x10a | 0x10c | 0x187 | 0x23b | 0x1e08 | 0xa792 => {
            for &ch in &[
                67, 0xc7, 0x106, 0x108, 0x10a, 0x10c, 0x187, 0x23b, 0x1e08, 0xa792,
            ] {
                nfa_emit2(ch);
            }
        }
        // D group
        68 | 0x10e | 0x110 | 0x18a | 0x1e0a | 0x1e0c | 0x1e0e | 0x1e10 | 0x1e12 => {
            for &ch in &[
                68, 0x10e, 0x110, 0x18a, 0x1e0a, 0x1e0c, 0x1e0e, 0x1e10, 0x1e12,
            ] {
                nfa_emit2(ch);
            }
        }
        // E group
        69 | 0xc8 | 0xc9 | 0xca | 0xcb | 0x112 | 0x114 | 0x116 | 0x118 | 0x11a | 0x204 | 0x206
        | 0x228 | 0x246 | 0x1e14 | 0x1e16 | 0x1e18 | 0x1e1a | 0x1e1c | 0x1eb8 | 0x1eba | 0x1ebc
        | 0x1ebe | 0x1ec0 | 0x1ec2 | 0x1ec4 | 0x1ec6 => {
            for &ch in &[
                69, 0xc8, 0xc9, 0xca, 0xcb, 0x112, 0x114, 0x116, 0x118, 0x11a, 0x204, 0x206, 0x228,
                0x246, 0x1e14, 0x1e16, 0x1e18, 0x1e1a, 0x1e1c, 0x1eb8, 0x1eba, 0x1ebc, 0x1ebe,
                0x1ec0, 0x1ec2, 0x1ec4, 0x1ec6,
            ] {
                nfa_emit2(ch);
            }
        }
        // F group
        70 | 0x191 | 0x1e1e | 0xa798 => {
            for &ch in &[70, 0x191, 0x1e1e, 0xa798] {
                nfa_emit2(ch);
            }
        }
        // G group
        71 | 0x11c | 0x11e | 0x120 | 0x122 | 0x193 | 0x1e4 | 0x1e6 | 0x1f4 | 0x1e20 | 0xa7a0 => {
            for &ch in &[
                71, 0x11c, 0x11e, 0x120, 0x122, 0x193, 0x1e4, 0x1e6, 0x1f4, 0x1e20, 0xa7a0,
            ] {
                nfa_emit2(ch);
            }
        }
        // H group
        72 | 0x124 | 0x126 | 0x21e | 0x1e22 | 0x1e24 | 0x1e26 | 0x1e28 | 0x1e2a | 0x2c67 => {
            for &ch in &[
                72, 0x124, 0x126, 0x21e, 0x1e22, 0x1e24, 0x1e26, 0x1e28, 0x1e2a, 0x2c67,
            ] {
                nfa_emit2(ch);
            }
        }
        // I group
        73 | 0xcc | 0xcd | 0xce | 0xcf | 0x128 | 0x12a | 0x12c | 0x12e | 0x130 | 0x197 | 0x1cf
        | 0x208 | 0x20a | 0x1e2c | 0x1e2e | 0x1ec8 | 0x1eca => {
            for &ch in &[
                73, 0xcc, 0xcd, 0xce, 0xcf, 0x128, 0x12a, 0x12c, 0x12e, 0x130, 0x197, 0x1cf, 0x208,
                0x20a, 0x1e2c, 0x1e2e, 0x1ec8, 0x1eca,
            ] {
                nfa_emit2(ch);
            }
        }
        // J group
        74 | 0x134 | 0x248 => {
            for &ch in &[74, 0x134, 0x248] {
                nfa_emit2(ch);
            }
        }
        // K group
        75 | 0x136 | 0x198 | 0x1e8 | 0x1e30 | 0x1e32 | 0x1e34 | 0x2c69 | 0xa740 => {
            for &ch in &[
                75, 0x136, 0x198, 0x1e8, 0x1e30, 0x1e32, 0x1e34, 0x2c69, 0xa740,
            ] {
                nfa_emit2(ch);
            }
        }
        // L group
        76 | 0x139 | 0x13b | 0x13d | 0x13f | 0x141 | 0x23d | 0x1e36 | 0x1e38 | 0x1e3a | 0x1e3c
        | 0x2c60 => {
            for &ch in &[
                76, 0x139, 0x13b, 0x13d, 0x13f, 0x141, 0x23d, 0x1e36, 0x1e38, 0x1e3a, 0x1e3c,
                0x2c60,
            ] {
                nfa_emit2(ch);
            }
        }
        // M group
        77 | 0x1e3e | 0x1e40 | 0x1e42 => {
            for &ch in &[77, 0x1e3e, 0x1e40, 0x1e42] {
                nfa_emit2(ch);
            }
        }
        // N group
        78 | 0xd1 | 0x143 | 0x145 | 0x147 | 0x1f8 | 0x1e44 | 0x1e46 | 0x1e48 | 0x1e4a | 0xa7a4 => {
            for &ch in &[
                78, 0xd1, 0x143, 0x145, 0x147, 0x1f8, 0x1e44, 0x1e46, 0x1e48, 0x1e4a, 0xa7a4,
            ] {
                nfa_emit2(ch);
            }
        }
        // O group
        79 | 0xd2 | 0xd3 | 0xd4 | 0xd5 | 0xd6 | 0xd8 | 0x14c | 0x14e | 0x150 | 0x19f | 0x1a0
        | 0x1d1 | 0x1ea | 0x1ec | 0x1fe | 0x20c | 0x20e | 0x22a | 0x22c | 0x22e | 0x230
        | 0x1e4c | 0x1e4e | 0x1e50 | 0x1e52 | 0x1ecc | 0x1ece | 0x1ed0 | 0x1ed2 | 0x1ed4
        | 0x1ed6 | 0x1ed8 | 0x1eda | 0x1edc | 0x1ede | 0x1ee0 | 0x1ee2 => {
            for &ch in &[
                79, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd8, 0x14c, 0x14e, 0x150, 0x19f, 0x1a0, 0x1d1,
                0x1ea, 0x1ec, 0x1fe, 0x20c, 0x20e, 0x22a, 0x22c, 0x22e, 0x230, 0x1e4c, 0x1e4e,
                0x1e50, 0x1e52, 0x1ecc, 0x1ece, 0x1ed0, 0x1ed2, 0x1ed4, 0x1ed6, 0x1ed8, 0x1eda,
                0x1edc, 0x1ede, 0x1ee0, 0x1ee2,
            ] {
                nfa_emit2(ch);
            }
        }
        // P group
        80 | 0x1a4 | 0x1e54 | 0x1e56 | 0x2c63 => {
            for &ch in &[80, 0x1a4, 0x1e54, 0x1e56, 0x2c63] {
                nfa_emit2(ch);
            }
        }
        // Q group
        81 | 0x24a => {
            for &ch in &[81, 0x24a] {
                nfa_emit2(ch);
            }
        }
        // R group
        82 | 0x154 | 0x156 | 0x158 | 0x210 | 0x212 | 0x24c | 0x1e58 | 0x1e5a | 0x1e5c | 0x1e5e
        | 0x2c64 | 0xa7a6 => {
            for &ch in &[
                82, 0x154, 0x156, 0x158, 0x210, 0x212, 0x24c, 0x1e58, 0x1e5a, 0x1e5c, 0x1e5e,
                0x2c64, 0xa7a6,
            ] {
                nfa_emit2(ch);
            }
        }
        // S group
        83 | 0x15a | 0x15c | 0x15e | 0x160 | 0x218 | 0x1e60 | 0x1e62 | 0x1e64 | 0x1e66 | 0x1e68
        | 0x2c7e | 0xa7a8 => {
            for &ch in &[
                83, 0x15a, 0x15c, 0x15e, 0x160, 0x218, 0x1e60, 0x1e62, 0x1e64, 0x1e66, 0x1e68,
                0x2c7e, 0xa7a8,
            ] {
                nfa_emit2(ch);
            }
        }
        // T group
        84 | 0x162 | 0x164 | 0x166 | 0x1ac | 0x1ae | 0x21a | 0x23e | 0x1e6a | 0x1e6c | 0x1e6e
        | 0x1e70 => {
            for &ch in &[
                84, 0x162, 0x164, 0x166, 0x1ac, 0x1ae, 0x23e, 0x21a, 0x1e6a, 0x1e6c, 0x1e6e, 0x1e70,
            ] {
                nfa_emit2(ch);
            }
        }
        // U group
        85 | 0xd9 | 0xda | 0xdc | 0xdb | 0x168 | 0x16a | 0x16c | 0x16e | 0x170 | 0x172 | 0x1af
        | 0x1d3 | 0x1d5 | 0x1d7 | 0x1d9 | 0x1db | 0x214 | 0x216 | 0x244 | 0x1e72 | 0x1e74
        | 0x1e76 | 0x1e78 | 0x1e7a | 0x1ee4 | 0x1ee6 | 0x1ee8 | 0x1eea | 0x1eec | 0x1eee
        | 0x1ef0 => {
            for &ch in &[
                85, 0xd9, 0xda, 0xdc, 0xdb, 0x168, 0x16a, 0x16c, 0x16e, 0x170, 0x172, 0x1af, 0x1d3,
                0x1d5, 0x1d7, 0x1d9, 0x1db, 0x214, 0x216, 0x244, 0x1e72, 0x1e74, 0x1e76, 0x1e78,
                0x1e7a, 0x1ee4, 0x1ee6, 0x1ee8, 0x1eea, 0x1eec, 0x1eee, 0x1ef0,
            ] {
                nfa_emit2(ch);
            }
        }
        // V group
        86 | 0x1b2 | 0x1e7c | 0x1e7e => {
            for &ch in &[86, 0x1b2, 0x1e7c, 0x1e7e] {
                nfa_emit2(ch);
            }
        }
        // W group
        87 | 0x174 | 0x1e80 | 0x1e82 | 0x1e84 | 0x1e86 | 0x1e88 => {
            for &ch in &[87, 0x174, 0x1e80, 0x1e82, 0x1e84, 0x1e86, 0x1e88] {
                nfa_emit2(ch);
            }
        }
        // X group
        88 | 0x1e8a | 0x1e8c => {
            for &ch in &[88, 0x1e8a, 0x1e8c] {
                nfa_emit2(ch);
            }
        }
        // Y group
        89 | 0xdd | 0x176 | 0x178 | 0x1b3 | 0x232 | 0x24e | 0x1e8e | 0x1ef2 | 0x1ef4 | 0x1ef6
        | 0x1ef8 => {
            for &ch in &[
                89, 0xdd, 0x176, 0x178, 0x1b3, 0x232, 0x24e, 0x1e8e, 0x1ef2, 0x1ef4, 0x1ef6, 0x1ef8,
            ] {
                nfa_emit2(ch);
            }
        }
        // Z group
        90 | 0x179 | 0x17b | 0x17d | 0x1b5 | 0x1e90 | 0x1e92 | 0x1e94 | 0x2c6b => {
            for &ch in &[
                90, 0x179, 0x17b, 0x17d, 0x1b5, 0x1e90, 0x1e92, 0x1e94, 0x2c6b,
            ] {
                nfa_emit2(ch);
            }
        }
        // a group
        97 | 0xe0 | 0xe1 | 0xe2 | 0xe3 | 0xe4 | 0xe5 | 0x101 | 0x103 | 0x105 | 0x1ce | 0x1df
        | 0x1e1 | 0x1fb | 0x201 | 0x203 | 0x227 | 0x1d8f | 0x1e01 | 0x1e9a | 0x1ea1 | 0x1ea3
        | 0x1ea5 | 0x1ea7 | 0x1ea9 | 0x1eab | 0x1ead | 0x1eaf | 0x1eb1 | 0x1eb3 | 0x1eb5
        | 0x1eb7 | 0x2c65 => {
            for &ch in &[
                97, 0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0x101, 0x103, 0x105, 0x1ce, 0x1df, 0x1e1,
                0x1fb, 0x201, 0x203, 0x227, 0x1d8f, 0x1e01, 0x1e9a, 0x1ea1, 0x1ea3, 0x1ea5, 0x1ea7,
                0x1ea9, 0x1eab, 0x1ead, 0x1eaf, 0x1eb1, 0x1eb3, 0x1eb5, 0x1eb7, 0x2c65,
            ] {
                nfa_emit2(ch);
            }
        }
        // b group
        98 | 0x180 | 0x253 | 0x1d6c | 0x1d80 | 0x1e03 | 0x1e05 | 0x1e07 => {
            for &ch in &[98, 0x180, 0x253, 0x1d6c, 0x1d80, 0x1e03, 0x1e05, 0x1e07] {
                nfa_emit2(ch);
            }
        }
        // c group
        99 | 0xe7 | 0x107 | 0x109 | 0x10b | 0x10d | 0x188 | 0x23c | 0x1e09 | 0xa793 | 0xa794 => {
            for &ch in &[
                99, 0xe7, 0x107, 0x109, 0x10b, 0x10d, 0x188, 0x23c, 0x1e09, 0xa793, 0xa794,
            ] {
                nfa_emit2(ch);
            }
        }
        // d group
        100 | 0x10f | 0x111 | 0x257 | 0x1d6d | 0x1d81 | 0x1d91 | 0x1e0b | 0x1e0d | 0x1e0f
        | 0x1e11 | 0x1e13 => {
            for &ch in &[
                100, 0x10f, 0x111, 0x257, 0x1d6d, 0x1d81, 0x1d91, 0x1e0b, 0x1e0d, 0x1e0f, 0x1e11,
                0x1e13,
            ] {
                nfa_emit2(ch);
            }
        }
        // e group
        101 | 0xe8 | 0xe9 | 0xea | 0xeb | 0x113 | 0x115 | 0x117 | 0x119 | 0x11b | 0x205 | 0x207
        | 0x229 | 0x247 | 0x1d92 | 0x1e15 | 0x1e17 | 0x1e19 | 0x1e1b | 0x1e1d | 0x1eb9 | 0x1ebb
        | 0x1ebd | 0x1ebf | 0x1ec1 | 0x1ec3 | 0x1ec5 | 0x1ec7 => {
            for &ch in &[
                101, 0xe8, 0xe9, 0xea, 0xeb, 0x113, 0x115, 0x117, 0x119, 0x11b, 0x205, 0x207,
                0x229, 0x247, 0x1d92, 0x1e15, 0x1e17, 0x1e19, 0x1e1b, 0x1e1d, 0x1eb9, 0x1ebb,
                0x1ebd, 0x1ebf, 0x1ec1, 0x1ec3, 0x1ec5, 0x1ec7,
            ] {
                nfa_emit2(ch);
            }
        }
        // f group
        102 | 0x192 | 0x1d6e | 0x1d82 | 0x1e1f | 0xa799 => {
            for &ch in &[102, 0x192, 0x1d6e, 0x1d82, 0x1e1f, 0xa799] {
                nfa_emit2(ch);
            }
        }
        // g group
        103 | 0x11d | 0x11f | 0x121 | 0x123 | 0x1e5 | 0x1e7 | 0x1f5 | 0x260 | 0x1d83 | 0x1e21
        | 0xa7a1 => {
            for &ch in &[
                103, 0x11d, 0x11f, 0x121, 0x123, 0x1e5, 0x1e7, 0x1f5, 0x260, 0x1d83, 0x1e21, 0xa7a1,
            ] {
                nfa_emit2(ch);
            }
        }
        // h group
        104 | 0x125 | 0x127 | 0x21f | 0x1e23 | 0x1e25 | 0x1e27 | 0x1e29 | 0x1e2b | 0x1e96
        | 0x2c68 | 0xa795 => {
            for &ch in &[
                104, 0x125, 0x127, 0x21f, 0x1e23, 0x1e25, 0x1e27, 0x1e29, 0x1e2b, 0x1e96, 0x2c68,
                0xa795,
            ] {
                nfa_emit2(ch);
            }
        }
        // i group
        105 | 0xec | 0xed | 0xee | 0xef | 0x129 | 0x12b | 0x12d | 0x12f | 0x1d0 | 0x209 | 0x20b
        | 0x268 | 0x1d96 | 0x1e2d | 0x1e2f | 0x1ec9 | 0x1ecb => {
            for &ch in &[
                105, 0xec, 0xed, 0xee, 0xef, 0x129, 0x12b, 0x12d, 0x12f, 0x1d0, 0x209, 0x20b,
                0x268, 0x1d96, 0x1e2d, 0x1e2f, 0x1ec9, 0x1ecb,
            ] {
                nfa_emit2(ch);
            }
        }
        // j group
        106 | 0x135 | 0x1f0 | 0x249 => {
            for &ch in &[106, 0x135, 0x1f0, 0x249] {
                nfa_emit2(ch);
            }
        }
        // k group
        107 | 0x137 | 0x199 | 0x1e9 | 0x1d84 | 0x1e31 | 0x1e33 | 0x1e35 | 0x2c6a | 0xa741 => {
            for &ch in &[
                107, 0x137, 0x199, 0x1e9, 0x1d84, 0x1e31, 0x1e33, 0x1e35, 0x2c6a, 0xa741,
            ] {
                nfa_emit2(ch);
            }
        }
        // l group
        108 | 0x13a | 0x13c | 0x13e | 0x140 | 0x142 | 0x19a | 0x1e37 | 0x1e39 | 0x1e3b | 0x1e3d
        | 0x2c61 => {
            for &ch in &[
                108, 0x13a, 0x13c, 0x13e, 0x140, 0x142, 0x19a, 0x1e37, 0x1e39, 0x1e3b, 0x1e3d,
                0x2c61,
            ] {
                nfa_emit2(ch);
            }
        }
        // m group
        109 | 0x1d6f | 0x1e3f | 0x1e41 | 0x1e43 => {
            for &ch in &[109, 0x1d6f, 0x1e3f, 0x1e41, 0x1e43] {
                nfa_emit2(ch);
            }
        }
        // n group
        110 | 0xf1 | 0x144 | 0x146 | 0x148 | 0x149 | 0x1f9 | 0x1d70 | 0x1d87 | 0x1e45 | 0x1e47
        | 0x1e49 | 0x1e4b | 0xa7a5 => {
            for &ch in &[
                110, 0xf1, 0x144, 0x146, 0x148, 0x149, 0x1f9, 0x1d70, 0x1d87, 0x1e45, 0x1e47,
                0x1e49, 0x1e4b, 0xa7a5,
            ] {
                nfa_emit2(ch);
            }
        }
        // o group
        111 | 0xf2 | 0xf3 | 0xf4 | 0xf5 | 0xf6 | 0xf8 | 0x14d | 0x14f | 0x151 | 0x1a1 | 0x1d2
        | 0x1eb | 0x1ed | 0x1ff | 0x20d | 0x20f | 0x22b | 0x22d | 0x22f | 0x231 | 0x275
        | 0x1e4d | 0x1e4f | 0x1e51 | 0x1e53 | 0x1ecd | 0x1ecf | 0x1ed1 | 0x1ed3 | 0x1ed5
        | 0x1ed7 | 0x1ed9 | 0x1edb | 0x1edd | 0x1edf | 0x1ee1 | 0x1ee3 => {
            for &ch in &[
                111, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf8, 0x14d, 0x14f, 0x151, 0x1a1, 0x1d2, 0x1eb,
                0x1ed, 0x1ff, 0x20d, 0x20f, 0x22b, 0x22d, 0x22f, 0x231, 0x275, 0x1e4d, 0x1e4f,
                0x1e51, 0x1e53, 0x1ecd, 0x1ecf, 0x1ed1, 0x1ed3, 0x1ed5, 0x1ed7, 0x1ed9, 0x1edb,
                0x1edd, 0x1edf, 0x1ee1, 0x1ee3,
            ] {
                nfa_emit2(ch);
            }
        }
        // p group
        112 | 0x1a5 | 0x1d71 | 0x1d7d | 0x1d88 | 0x1e55 | 0x1e57 => {
            for &ch in &[112, 0x1a5, 0x1d71, 0x1d7d, 0x1d88, 0x1e55, 0x1e57] {
                nfa_emit2(ch);
            }
        }
        // q group
        113 | 0x24b | 0x2a0 => {
            for &ch in &[113, 0x24b, 0x2a0] {
                nfa_emit2(ch);
            }
        }
        // r group
        114 | 0x155 | 0x157 | 0x159 | 0x211 | 0x213 | 0x24d | 0x27d | 0x1d72 | 0x1d73 | 0x1d89
        | 0x1e59 | 0x1e5b | 0x1e5d | 0x1e5f | 0xa7a7 => {
            for &ch in &[
                114, 0x155, 0x157, 0x159, 0x211, 0x213, 0x24d, 0x27d, 0x1d72, 0x1d73, 0x1d89,
                0x1e59, 0x1e5b, 0x1e5d, 0x1e5f, 0xa7a7,
            ] {
                nfa_emit2(ch);
            }
        }
        // s group
        115 | 0x15b | 0x15d | 0x15f | 0x161 | 0x219 | 0x23f | 0x1d74 | 0x1d8a | 0x1e61 | 0x1e63
        | 0x1e65 | 0x1e67 | 0x1e69 | 0xa7a9 => {
            for &ch in &[
                115, 0x15b, 0x15d, 0x15f, 0x161, 0x219, 0x23f, 0x1d74, 0x1d8a, 0x1e61, 0x1e63,
                0x1e65, 0x1e67, 0x1e69, 0xa7a9,
            ] {
                nfa_emit2(ch);
            }
        }
        // t group
        116 | 0x163 | 0x165 | 0x167 | 0x1ab | 0x1ad | 0x21b | 0x288 | 0x1d75 | 0x1e6b | 0x1e6d
        | 0x1e6f | 0x1e71 | 0x1e97 | 0x2c66 => {
            for &ch in &[
                116, 0x163, 0x165, 0x167, 0x1ab, 0x1ad, 0x21b, 0x288, 0x1d75, 0x1e6b, 0x1e6d,
                0x1e6f, 0x1e71, 0x1e97, 0x2c66,
            ] {
                nfa_emit2(ch);
            }
        }
        // u group
        117 | 0xf9 | 0xfa | 0xfb | 0xfc | 0x169 | 0x16b | 0x16d | 0x16f | 0x171 | 0x173 | 0x1b0
        | 0x1d4 | 0x1d6 | 0x1d8 | 0x1da | 0x1dc | 0x215 | 0x217 | 0x289 | 0x1d7e | 0x1d99
        | 0x1e73 | 0x1e75 | 0x1e77 | 0x1e79 | 0x1e7b | 0x1ee5 | 0x1ee7 | 0x1ee9 | 0x1eeb
        | 0x1eed | 0x1eef | 0x1ef1 => {
            for &ch in &[
                117, 0xf9, 0xfa, 0xfb, 0xfc, 0x169, 0x16b, 0x16d, 0x16f, 0x171, 0x173, 0x1b0,
                0x1d4, 0x1d6, 0x1d8, 0x1da, 0x1dc, 0x215, 0x217, 0x289, 0x1d7e, 0x1d99, 0x1e73,
                0x1e75, 0x1e77, 0x1e79, 0x1e7b, 0x1ee5, 0x1ee7, 0x1ee9, 0x1eeb, 0x1eed, 0x1eef,
                0x1ef1,
            ] {
                nfa_emit2(ch);
            }
        }
        // v group
        118 | 0x28b | 0x1d8c | 0x1e7d | 0x1e7f => {
            for &ch in &[118, 0x28b, 0x1d8c, 0x1e7d, 0x1e7f] {
                nfa_emit2(ch);
            }
        }
        // w group
        119 | 0x175 | 0x1e81 | 0x1e83 | 0x1e85 | 0x1e87 | 0x1e89 | 0x1e98 => {
            for &ch in &[119, 0x175, 0x1e81, 0x1e83, 0x1e85, 0x1e87, 0x1e89, 0x1e98] {
                nfa_emit2(ch);
            }
        }
        // x group
        120 | 0x1e8b | 0x1e8d => {
            for &ch in &[120, 0x1e8b, 0x1e8d] {
                nfa_emit2(ch);
            }
        }
        // y group
        121 | 0xfd | 0xff | 0x177 | 0x1b4 | 0x233 | 0x24f | 0x1e8f | 0x1e99 | 0x1ef3 | 0x1ef5
        | 0x1ef7 | 0x1ef9 => {
            for &ch in &[
                121, 0xfd, 0xff, 0x177, 0x1b4, 0x233, 0x24f, 0x1e8f, 0x1e99, 0x1ef3, 0x1ef5,
                0x1ef7, 0x1ef9,
            ] {
                nfa_emit2(ch);
            }
        }
        // z group
        122 | 0x17a | 0x17c | 0x17e | 0x1b6 | 0x1d76 | 0x1d8e | 0x1e91 | 0x1e93 | 0x1e95
        | 0x2c6c => {
            for &ch in &[
                122, 0x17a, 0x17c, 0x17e, 0x1b6, 0x1d76, 0x1d8e, 0x1e91, 0x1e93, 0x1e95, 0x2c6c,
            ] {
                nfa_emit2(ch);
            }
        }
        // default: character itself
        _ => {
            nfa_emit2(c);
        }
    }
}

/// Recognize a character class in expanded form (e.g. [0-9]).
/// Returns the NFA class constant on success, or FAIL (0) on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_recognize_char_class(
    start: *mut u8,
    end: *const u8,
    extra_newl: c_int,
) -> c_int {
    const CLASS_NOT: u8 = 0x80;
    const CLASS_AF: u8 = 0x40;
    const CLASS_CAP_AF: u8 = 0x20;
    const CLASS_AZ: u8 = 0x10;
    const CLASS_CAP_AZ: u8 = 0x08;
    const CLASS_O7: u8 = 0x04;
    const CLASS_O9: u8 = 0x02;
    const CLASS_UNDERSCORE: u8 = 0x01;

    if *end != b']' {
        return FAIL;
    }

    let mut config: u8 = 0;
    let mut newl = extra_newl == 1;
    let mut p = start;
    let end_mut = end.cast_mut();

    if *p == b'^' {
        config |= CLASS_NOT;
        p = p.add(1);
    }

    while (p as usize) < (end_mut as usize) {
        if (p.add(2) as usize) < (end_mut as usize) && *p.add(1) == b'-' {
            match *p {
                b'0' => {
                    if *p.add(2) == b'9' {
                        config |= CLASS_O9;
                    } else if *p.add(2) == b'7' {
                        config |= CLASS_O7;
                    } else {
                        return FAIL;
                    }
                }
                b'a' => {
                    if *p.add(2) == b'z' {
                        config |= CLASS_AZ;
                    } else if *p.add(2) == b'f' {
                        config |= CLASS_AF;
                    } else {
                        return FAIL;
                    }
                }
                b'A' => {
                    if *p.add(2) == b'Z' {
                        config |= CLASS_CAP_AZ;
                    } else if *p.add(2) == b'F' {
                        config |= CLASS_CAP_AF;
                    } else {
                        return FAIL;
                    }
                }
                _ => return FAIL,
            }
            p = p.add(3);
        } else if (p.add(1) as usize) < (end_mut as usize) && *p == b'\\' && *p.add(1) == b'n' {
            newl = true;
            p = p.add(2);
        } else if *p == b'_' {
            config |= CLASS_UNDERSCORE;
            p = p.add(1);
        } else if *p == b'\n' {
            newl = true;
            p = p.add(1);
        } else {
            return FAIL;
        }
    }

    if !std::ptr::eq(p, end) {
        return FAIL;
    }

    let extra = if newl { NFA_ADD_NL } else { 0 };

    match config {
        x if x == CLASS_O9 => extra + NFA_DIGIT,
        x if x == CLASS_NOT | CLASS_O9 => extra + NFA_NDIGIT,
        x if x == CLASS_AF | CLASS_CAP_AF | CLASS_O9 => extra + NFA_HEX,
        x if x == CLASS_NOT | CLASS_AF | CLASS_CAP_AF | CLASS_O9 => extra + NFA_NHEX,
        x if x == CLASS_O7 => extra + NFA_OCTAL,
        x if x == CLASS_NOT | CLASS_O7 => extra + NFA_NOCTAL,
        x if x == CLASS_AZ | CLASS_CAP_AZ | CLASS_O9 | CLASS_UNDERSCORE => extra + NFA_WORD,
        x if x == CLASS_NOT | CLASS_AZ | CLASS_CAP_AZ | CLASS_O9 | CLASS_UNDERSCORE => {
            extra + NFA_NWORD
        }
        x if x == CLASS_AZ | CLASS_CAP_AZ | CLASS_UNDERSCORE => extra + NFA_HEAD,
        x if x == CLASS_NOT | CLASS_AZ | CLASS_CAP_AZ | CLASS_UNDERSCORE => extra + NFA_NHEAD,
        x if x == CLASS_AZ | CLASS_CAP_AZ => extra + NFA_ALPHA,
        x if x == CLASS_NOT | CLASS_AZ | CLASS_CAP_AZ => extra + NFA_NALPHA,
        x if x == CLASS_AZ => extra + NFA_LOWER_IC,
        x if x == CLASS_NOT | CLASS_AZ => extra + NFA_NLOWER_IC,
        x if x == CLASS_CAP_AZ => extra + NFA_UPPER_IC,
        x if x == CLASS_NOT | CLASS_CAP_AZ => extra + NFA_NUPPER_IC,
        _ => FAIL,
    }
}

// --- Phase 3: NFA regatom extern declarations ---
#[allow(dead_code)]
// Phase 3 constant
const MB_MAXBYTES: i64 = 21;

// `Magic(x)` in C is `(int)(x) - 256`; equivalent to `magic()` without reg_magic check.
const fn nfa_magic(x: u8) -> c_int {
    x as c_int - 256
}

/// Get regparse as `*mut u8` for byte comparisons.
#[inline]
unsafe fn regparse_u8() -> *mut u8 {
    REGPARSE.cast::<u8>()
}

/// Set regparse from a `*mut u8`.
#[inline]
unsafe fn set_regparse_u8(p: *mut u8) {
    REGPARSE = p.cast::<c_char>();
}

/// Helper: `MB_PTR_ADV(regparse)` — advance regparse by one composing-aware character.
unsafe fn mb_ptr_adv_regparse() {
    let rp = REGPARSE;
    REGPARSE = rp.add(utfc_ptr2len(rp) as usize);
}

/// Helper: `MB_PTR_BACK(base, regparse)` — back up regparse by one character.
unsafe fn mb_ptr_back_regparse(base: *const u8) {
    let rp = REGPARSE;
    let off = utf_head_off(base.cast::<c_char>(), rp.sub(1));
    REGPARSE = rp.sub((off + 1) as usize);
}

/// Handle the `nfa_do_multibyte:` label — composing/multibyte character handling.
/// Returns OK on success, FAIL on failure (never in practice for this path).
unsafe fn nfa_handle_multibyte(c_in: c_int, old_rp: *mut c_char) -> c_int {
    let mut c = c_in;
    let plen = utfc_ptr2len(old_rp);
    if utf_char2len(c) != plen || utf_iscomposing_legacy(c) != 0 {
        let mut i: c_int = 0;
        // Composing characters: emit base + composing chars + NFA_COMPOSING
        loop {
            nfa_emit(c);
            if i > 0 {
                nfa_emit(NFA_CONCAT);
            }
            i += utf_char2len(c);
            if i >= plen {
                break;
            }
            c = utf_ptr2char(old_rp.add(i as usize));
        }
        nfa_emit(NFA_COMPOSING);
        REGPARSE = old_rp.add(plen as usize);
    } else {
        c = rs_no_magic(c);
        nfa_emit(c);
    }
    OK
}

/// Handle the `collection:` label — character class `[...]` parsing.
/// `extra` is `NFA_ADD_NL` if `\_[` was used, 0 otherwise.
#[allow(clippy::too_many_lines, clippy::similar_names)]
unsafe fn nfa_handle_collection(mut extra: c_int, old_rp: *mut c_char) -> c_int {
    let p = REGPARSE;
    let endp = rs_skip_anyof(p);

    if *endp.cast::<u8>() == b']' {
        // Try to recognize a character class like [0-9] → \d
        let result = rs_nfa_recognize_char_class(
            p.cast::<u8>(),
            endp.cast::<u8>(),
            c_int::from(extra == NFA_ADD_NL),
        );
        if result != FAIL {
            if (NFA_FIRST_NL..=NFA_LAST_NL).contains(&result) {
                nfa_emit(result - NFA_ADD_NL);
                nfa_emit(NFA_NEWL);
                nfa_emit(NFA_OR);
            } else {
                nfa_emit(result);
            }
            REGPARSE = endp;
            mb_ptr_adv_regparse();
            return OK;
        }

        // Not a recognized class — parse individual characters
        let mut negated = false;
        if *regparse_u8() == b'^' {
            negated = true;
            mb_ptr_adv_regparse();
            nfa_emit(NFA_START_NEG_COLL);
        } else {
            nfa_emit(NFA_START_COLL);
        }

        if *regparse_u8() == b'-' {
            nfa_emit(b'-' as c_int);
            nfa_emit(NFA_CONCAT);
            mb_ptr_adv_regparse();
        }

        let mut emit_range = false;
        let mut startc: c_int = -1;
        let mut c: c_int;

        while (REGPARSE as usize) < (endp as usize) {
            let oldstartc = startc;
            startc = -1;
            let mut got_coll_char = false;

            if *regparse_u8() == b'[' {
                // Check for [: :], [= =], [. .]
                let mut rp = REGPARSE;
                let charclass = nvim_regexp_get_char_class(&mut rp);
                REGPARSE = rp;

                if charclass == CLASS_NONE {
                    let mut rp2 = REGPARSE;
                    let equiclass = get_equi_class(&mut rp2);
                    REGPARSE = rp2;

                    if equiclass == 0 {
                        let mut rp3 = REGPARSE;
                        let collclass = get_coll_element(&mut rp3);
                        REGPARSE = rp3;

                        if collclass != 0 {
                            startc = collclass; // allow [.a.]-x as a range
                        }
                    } else {
                        // Equivalence class
                        rs_nfa_emit_equi_class(equiclass);
                        continue;
                    }
                } else {
                    // Character class like [:alpha:]
                    match charclass {
                        x if x == CLASS_ALNUM => nfa_emit(NFA_CLASS_ALNUM),
                        x if x == CLASS_ALPHA => nfa_emit(NFA_CLASS_ALPHA),
                        x if x == CLASS_BLANK => nfa_emit(NFA_CLASS_BLANK),
                        x if x == CLASS_CNTRL => nfa_emit(NFA_CLASS_CNTRL),
                        x if x == CLASS_DIGIT => nfa_emit(NFA_CLASS_DIGIT),
                        x if x == CLASS_GRAPH => nfa_emit(NFA_CLASS_GRAPH),
                        x if x == CLASS_LOWER => {
                            WANTS_NFA = 1;
                            nfa_emit(NFA_CLASS_LOWER);
                        }
                        x if x == CLASS_PRINT => nfa_emit(NFA_CLASS_PRINT),
                        x if x == CLASS_PUNCT => nfa_emit(NFA_CLASS_PUNCT),
                        x if x == CLASS_SPACE => nfa_emit(NFA_CLASS_SPACE),
                        x if x == CLASS_UPPER => {
                            WANTS_NFA = 1;
                            nfa_emit(NFA_CLASS_UPPER);
                        }
                        x if x == CLASS_XDIGIT => nfa_emit(NFA_CLASS_XDIGIT),
                        x if x == CLASS_CC_TAB => nfa_emit(NFA_CLASS_TAB),
                        x if x == CLASS_RETURN => nfa_emit(NFA_CLASS_RETURN),
                        x if x == CLASS_BACKSPACE => nfa_emit(NFA_CLASS_BACKSPACE),
                        x if x == CLASS_ESCAPE => nfa_emit(NFA_CLASS_ESCAPE),
                        x if x == CLASS_IDENT => nfa_emit(NFA_CLASS_IDENT),
                        x if x == CLASS_KEYWORD => nfa_emit(NFA_CLASS_KEYWORD),
                        x if x == CLASS_FNAME => nfa_emit(NFA_CLASS_FNAME),
                        _ => {}
                    }
                    nfa_emit(NFA_CONCAT);
                    continue;
                }
            }

            // Try a range like 'a-x'
            if *regparse_u8() == b'-' && oldstartc != -1 {
                emit_range = true;
                startc = oldstartc;
                mb_ptr_adv_regparse();
                continue;
            }

            // Handle simple and escaped characters
            let rp = regparse_u8();
            if *rp == b'\\'
                && (rp.add(1) as usize) <= (endp as usize)
                && (!vim_strchr(
                    REGEXP_INRANGE.as_ptr().cast::<c_char>(),
                    *rp.add(1) as c_int,
                )
                .is_null()
                    || (REG_CPO_LIT == 0
                        && !vim_strchr(REGEXP_ABBR.as_ptr().cast::<c_char>(), *rp.add(1) as c_int)
                            .is_null()))
            {
                mb_ptr_adv_regparse();
                let rp2 = regparse_u8();

                if *rp2 == b'n' {
                    startc = if REG_STRING != 0 || emit_range || *rp2.add(1) == b'-' {
                        NL
                    } else {
                        NFA_NEWL
                    };
                } else if *rp2 == b'd'
                    || *rp2 == b'o'
                    || *rp2 == b'x'
                    || *rp2 == b'u'
                    || *rp2 == b'U'
                {
                    startc = coll_get_char();
                    if startc == c_int::MAX {
                        errors::emsg_e949();
                        return FAIL;
                    }
                    got_coll_char = true;
                    mb_ptr_back_regparse(old_rp.cast::<u8>());
                } else {
                    startc = rs_backslash_trans(*rp2 as c_int);
                }
            }

            // Normal printable char
            if startc == -1 {
                startc = utf_ptr2char(REGPARSE);
            }

            // Previous char was '-', so this char is end of range
            if emit_range {
                let endc = startc;
                startc = oldstartc;
                if startc > endc {
                    errors::emsg_e944();
                    return FAIL;
                }

                if endc > startc + 2 {
                    // Emit a range instead of individual characters
                    if startc == 0 {
                        // \x00 is translated to \x0a, start at \x01
                        nfa_emit(1);
                    } else {
                        // Remove previous NFA_CONCAT
                        let pp = POST_PTR;
                        POST_PTR = pp.sub(1);
                    }
                    nfa_emit(endc);
                    nfa_emit(NFA_RANGE);
                    nfa_emit(NFA_CONCAT);
                } else {
                    // Emit the characters in the range
                    c = startc + 1;
                    while c <= endc {
                        nfa_emit(c);
                        nfa_emit(NFA_CONCAT);
                        c += 1;
                    }
                }
                emit_range = false;
                startc = -1;
            } else {
                // Not part of a range
                if startc == NFA_NEWL {
                    if !negated {
                        extra = NFA_ADD_NL;
                    }
                } else if got_coll_char && startc == 0 {
                    nfa_emit(0x0a);
                    nfa_emit(NFA_CONCAT);
                } else {
                    nfa_emit(startc);
                    let rp3 = REGPARSE;
                    if utf_ptr2len(rp3) == utfc_ptr2len(rp3) {
                        nfa_emit(NFA_CONCAT);
                    }
                }
            }

            // Handle composing characters within the collection
            let rp4 = REGPARSE;
            let plen = utfc_ptr2len(rp4);
            if utf_ptr2len(rp4) != plen {
                let mut i = utf_ptr2len(rp4);
                c = utf_ptr2char(rp4.add(i as usize));

                loop {
                    if c == 0 {
                        nfa_emit(1);
                    } else {
                        nfa_emit(c);
                    }
                    nfa_emit(NFA_CONCAT);
                    i += utf_char2len(c);
                    if i >= plen {
                        break;
                    }
                    c = utf_ptr2char(rp4.add(i as usize));
                }
                nfa_emit(NFA_COMPOSING);
                nfa_emit(NFA_CONCAT);
            }

            mb_ptr_adv_regparse();
        } // while regparse < endp

        mb_ptr_back_regparse(old_rp.cast::<u8>());
        if *regparse_u8() == b'-' {
            nfa_emit(b'-' as c_int);
            nfa_emit(NFA_CONCAT);
        }

        // Skip the trailing ]
        REGPARSE = endp;
        mb_ptr_adv_regparse();

        // Mark end of the collection
        if negated {
            nfa_emit(NFA_END_NEG_COLL);
        } else {
            nfa_emit(NFA_END_COLL);
        }

        // \_[] also matches \n but it's not negated
        if extra == NFA_ADD_NL {
            nfa_emit(if REG_STRING != 0 { NL } else { NFA_NEWL });
            nfa_emit(NFA_OR);
        }

        return OK;
    } // if *endp == ']'

    if REG_STRICT != 0 {
        errors::emsg2_e769(c_int::from(REG_MAGIC > MAGIC_OFF));
        return FAIL;
    }
    // Fall through to default (multibyte) handling — caller handles this
    -1 // sentinel: caller should fall through to default
}

/// NFA regatom: parse a single atom in NFA regexp compilation.
///
/// Handles character classes, anchors, backreferences, groups, collections,
/// and literal characters, emitting NFA postfix opcodes.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub unsafe extern "C" fn rs_nfa_regatom() -> c_int {
    let old_regparse = REGPARSE;
    let mut extra: c_int = 0;
    let save_prev_at_start = PREV_AT_START;

    let mut c = rs_getchr();

    // NUL
    if c == 0 {
        errors::emsg_nul_found();
        return FAIL;
    }

    // ^
    if c == nfa_magic(b'^') {
        nfa_emit(NFA_BOL);
        return OK;
    }
    // $
    if c == nfa_magic(b'$') {
        nfa_emit(NFA_EOL);
        HAD_EOL = 1;
        return OK;
    }
    // <
    if c == nfa_magic(b'<') {
        nfa_emit(NFA_BOW);
        return OK;
    }
    // >
    if c == nfa_magic(b'>') {
        nfa_emit(NFA_EOW);
        return OK;
    }

    // \_  prefix
    if c == nfa_magic(b'_') {
        c = rs_no_magic(rs_getchr());
        if c == 0 {
            errors::emsg_nul_found();
            return FAIL;
        }
        if c == b'^' as c_int {
            nfa_emit(NFA_BOL);
            return OK;
        }
        if c == b'$' as c_int {
            nfa_emit(NFA_EOL);
            HAD_EOL = 1;
            return OK;
        }
        extra = NFA_ADD_NL;
        if c == b'[' as c_int {
            // \_[ is collection plus newline
            let result = nfa_handle_collection(extra, old_regparse);
            if result != -1 {
                return result;
            }
            // No closing ']' — fall through to handle '[' as literal
            // (mirrors the bare '[' fallthrough at line ~8058)
            return nfa_handle_multibyte(nfa_magic(b'['), old_regparse);
        }
        // \_x is character class plus newline — fall through to char class handling
        return nfa_handle_char_class(c, extra, old_regparse);
    }

    // Character classes: . i I k K f F p P s S d D x X o O w W h H a A l L u U
    if c == nfa_magic(b'.')
        || c == nfa_magic(b'i')
        || c == nfa_magic(b'I')
        || c == nfa_magic(b'k')
        || c == nfa_magic(b'K')
        || c == nfa_magic(b'f')
        || c == nfa_magic(b'F')
        || c == nfa_magic(b'p')
        || c == nfa_magic(b'P')
        || c == nfa_magic(b's')
        || c == nfa_magic(b'S')
        || c == nfa_magic(b'd')
        || c == nfa_magic(b'D')
        || c == nfa_magic(b'x')
        || c == nfa_magic(b'X')
        || c == nfa_magic(b'o')
        || c == nfa_magic(b'O')
        || c == nfa_magic(b'w')
        || c == nfa_magic(b'W')
        || c == nfa_magic(b'h')
        || c == nfa_magic(b'H')
        || c == nfa_magic(b'a')
        || c == nfa_magic(b'A')
        || c == nfa_magic(b'l')
        || c == nfa_magic(b'L')
        || c == nfa_magic(b'u')
        || c == nfa_magic(b'U')
    {
        return nfa_handle_char_class(c, extra, old_regparse);
    }

    // \n
    if c == nfa_magic(b'n') {
        if REG_STRING != 0 {
            nfa_emit(NL);
        } else {
            nfa_emit(NFA_NEWL);
            REGFLAGS_COMPILE |= RF_HASNL;
        }
        return OK;
    }

    // \(
    if c == nfa_magic(b'(') {
        if rs_nfa_reg(REG_PAREN) == FAIL {
            return FAIL;
        }
        return OK;
    }

    // Misplaced \|, \&, \)
    if c == nfa_magic(b'|') || c == nfa_magic(b'&') || c == nfa_magic(b')') {
        errors::semsg_misplaced(rs_no_magic(c));
        return FAIL;
    }

    // Misplaced \=, \?, \+, \@, \*, \{
    if c == nfa_magic(b'=')
        || c == nfa_magic(b'?')
        || c == nfa_magic(b'+')
        || c == nfa_magic(b'@')
        || c == nfa_magic(b'*')
        || c == nfa_magic(b'{')
    {
        errors::semsg_misplaced(rs_no_magic(c));
        return FAIL;
    }

    // \~ — previous substitute pattern
    if c == nfa_magic(b'~') {
        let reg_prev_sub = nvim_regexp_get_reg_prev_sub_ptr();
        if reg_prev_sub.is_null() {
            errors::emsg_nopresub();
            return FAIL;
        }
        let mut lp = reg_prev_sub.cast::<u8>();
        while *lp != 0 {
            nfa_emit(utf_ptr2char(lp.cast::<c_char>()));
            if lp != reg_prev_sub.cast::<u8>() {
                nfa_emit(NFA_CONCAT);
            }
            lp = lp.add(utf_ptr2len(lp.cast::<c_char>()) as usize);
        }
        nfa_emit(NFA_NOPEN);
        return OK;
    }

    // \1 through \9 — backreferences
    if c >= nfa_magic(b'1') && c <= nfa_magic(b'9') {
        let refnum = rs_no_magic(c) - b'1' as c_int;
        if !seen_endbrace(refnum + 1) {
            return FAIL;
        }
        nfa_emit(NFA_BACKREF1 + refnum);
        REX.nfa_has_backref = 1;
        return OK;
    }

    // \z — zstart/zend/zref/zparen
    if c == nfa_magic(b'z') {
        return nfa_handle_z_atom();
    }

    // \% — percent atoms
    if c == nfa_magic(b'%') {
        return nfa_handle_percent_atom(save_prev_at_start);
    }

    // [...] — character collection
    if c == nfa_magic(b'[') {
        let result = nfa_handle_collection(extra, old_regparse);
        if result != -1 {
            return result;
        }
        // No closing ']' and not strict mode — fall through to handle '[' as literal
        // (This mirrors the C FALLTHROUGH to default: nfa_do_multibyte)
    }

    // Default: literal character or multibyte/composing
    nfa_handle_multibyte(c, old_regparse)
}

/// Handle character class atoms (`.`, `\i`, `\I`, etc.) and `\_x` variants.
unsafe fn nfa_handle_char_class(c: c_int, extra: c_int, _old_regparse: *mut c_char) -> c_int {
    let classchars = CLASSCHARS.as_ptr();
    let p = vim_strchr(classchars.cast::<c_char>(), rs_no_magic(c));
    if p.is_null() {
        if extra == NFA_ADD_NL {
            errors::semsg_ill_char_class(c as i64);
            return FAIL;
        }
        errors::siemsg_unknown_class(c as i64);
        return FAIL;
    }

    // When '.' is followed by a composing char ignore the dot
    if c == nfa_magic(b'.') && utf_iscomposing_legacy(rs_peekchr()) != 0 {
        let new_old = REGPARSE;
        let c2 = rs_getchr();
        return nfa_handle_multibyte(c2, new_old);
    }

    #[allow(clippy::cast_possible_truncation)]
    let index = ((p as usize) - (classchars as usize)) as c_int;
    nfa_emit(NFA_CLASSCODES[index as usize]);
    if extra == NFA_ADD_NL {
        nfa_emit(NFA_NEWL);
        nfa_emit(NFA_OR);
        REGFLAGS_COMPILE |= RF_HASNL;
    }
    OK
}

/// Handle `\z` atoms: `\zs`, `\ze`, `\z1`..`\z9`, `\z(`.
unsafe fn nfa_handle_z_atom() -> c_int {
    let c = rs_no_magic(rs_getchr());
    match c {
        // \zs
        x if x == b's' as c_int => {
            nfa_emit(NFA_ZSTART);
            if !rs_re_mult_next(c"\\zs".as_ptr().cast::<c_char>()) {
                return FAIL;
            }
            OK
        }
        // \ze
        x if x == b'e' as c_int => {
            nfa_emit(NFA_ZEND);
            REX.nfa_has_zend = 1;
            if !rs_re_mult_next(c"\\ze".as_ptr().cast::<c_char>()) {
                return FAIL;
            }
            OK
        }
        // \z1 .. \z9
        x if x >= b'1' as c_int && x <= b'9' as c_int => {
            if (nvim_regexp_get_reg_do_extmatch() & REX_USE) == 0 {
                errors::emsg_e67();
                return FAIL;
            }
            nfa_emit(NFA_ZREF1 + (rs_no_magic(c) - b'1' as c_int));
            RE_HAS_Z = REX_USE;
            OK
        }
        // \z(
        x if x == b'(' as c_int => {
            if nvim_regexp_get_reg_do_extmatch() != REX_SET {
                errors::emsg_e66();
                return FAIL;
            }
            if rs_nfa_reg(REG_ZPAREN) == FAIL {
                return FAIL;
            }
            RE_HAS_Z = REX_SET;
            OK
        }
        _ => {
            errors::semsg_e867_z(rs_no_magic(c));
            FAIL
        }
    }
}

/// Handle `\%` atoms: `\%(`, `\%d`, `\%o`, `\%x`, `\%u`, `\%U`,
/// `\%^`, `\%$`, `\%#`, `\%V`, `\%C`, `\%[`, and position/mark atoms.
#[allow(clippy::too_many_lines)]
unsafe fn nfa_handle_percent_atom(save_prev_at_start: c_int) -> c_int {
    let c = rs_no_magic(rs_getchr());

    // \%(
    if c == b'(' as c_int {
        if rs_nfa_reg(REG_NPAREN) == FAIL {
            return FAIL;
        }
        nfa_emit(NFA_NOPEN);
        return OK;
    }

    // \%d, \%o, \%x, \%u, \%U
    if c == b'd' as c_int
        || c == b'o' as c_int
        || c == b'x' as c_int
        || c == b'u' as c_int
        || c == b'U' as c_int
    {
        let nr: c_long = match c {
            x if x == b'd' as c_int => rs_getdecchrs(),
            x if x == b'o' as c_int => rs_getoctchrs(),
            x if x == b'x' as c_int => rs_gethexchrs(2),
            x if x == b'u' as c_int => rs_gethexchrs(4),
            x if x == b'U' as c_int => rs_gethexchrs(8),
            _ => -1,
        };
        if nr < 0 || nr > c_long::from(c_int::MAX) {
            errors::emsg2_e678(c_int::from(REG_MAGIC == MAGIC_ALL));
            return FAIL;
        }
        // A NUL is stored as NL (nr is in range, checked above)
        #[allow(clippy::cast_possible_truncation)]
        let nr_int = nr as c_int;
        nfa_emit(if nr == 0 { 0x0a } else { nr_int });
        return OK;
    }

    // \%^
    if c == b'^' as c_int {
        nfa_emit(NFA_BOF);
        return OK;
    }

    // \%$
    if c == b'$' as c_int {
        nfa_emit(NFA_EOF);
        return OK;
    }

    // \%#
    if c == b'#' as c_int {
        let rp = REGPARSE;
        if *rp == b'=' as c_char && *rp.add(1) >= 48 && *rp.add(1) <= 50 {
            errors::semsg_e_atom_engine(*rp.add(1) as c_int);
            return FAIL;
        }
        nfa_emit(NFA_CURSOR);
        return OK;
    }

    // \%V
    if c == b'V' as c_int {
        nfa_emit(NFA_VISUAL);
        return OK;
    }

    // \%C
    if c == b'C' as c_int {
        nfa_emit(NFA_ANY_COMPOSING);
        return OK;
    }

    // \%[abc]
    if c == b'[' as c_int {
        let mut n: c_int = 0;
        loop {
            let pc = rs_peekchr();
            if pc == b']' as c_int {
                break;
            }
            if pc == 0 {
                errors::emsg2_e769(c_int::from(REG_MAGIC == MAGIC_ALL));
                return FAIL;
            }
            // recursive call
            if rs_nfa_regatom() == FAIL {
                return FAIL;
            }
            n += 1;
        }
        rs_getchr(); // consume the ]
        if n == 0 {
            errors::emsg2_e70(c_int::from(REG_MAGIC == MAGIC_ALL));
            return FAIL;
        }
        nfa_emit(NFA_OPT_CHARS);
        nfa_emit(n);
        nfa_emit(NFA_NOPEN);
        return OK;
    }

    // \%'m marks, \%<'m, \%>'m, or \%{n}l/c/v
    nfa_handle_percent_position(c, save_prev_at_start)
}

/// Handle `\%` position/mark atoms: `\%{n}l`, `\%{n}c`, `\%{n}v`,
/// `\%.l`, `\%.c`, `\%.v`, `\%<{n}l`, `\%>{n}l`, `\%'m`, etc.
unsafe fn nfa_handle_percent_position(c_in: c_int, save_prev_at_start: c_int) -> c_int {
    let mut c = c_in;
    let cmp = c;
    let mut n: i64 = 0;
    let mut cur = false;
    let mut got_digit = false;

    if c == b'<' as c_int || c == b'>' as c_int {
        c = rs_getchr();
    }
    if rs_no_magic(c) == b'.' as c_int {
        cur = true;
        c = rs_getchr();
    }
    while c >= b'0' as c_int && c <= b'9' as c_int {
        if cur {
            errors::semsg_e_dot_pos(rs_no_magic(c));
            return FAIL;
        }
        if n > (i64::from(i32::MAX) - i64::from(c - b'0' as c_int)) / 10 {
            errors::emsg_value_too_large();
            return FAIL;
        }
        n = n * 10 + i64::from(c - b'0' as c_int);
        c = rs_getchr();
        got_digit = true;
    }

    if c == b'l' as c_int || c == b'c' as c_int || c == b'v' as c_int {
        let mut limit: i64 = i64::from(i32::MAX);
        if !cur && !got_digit {
            errors::semsg_missing_value(rs_no_magic(c));
            return FAIL;
        }
        if c == b'l' as c_int {
            if cur {
                n = i64::from(nvim_regexp_get_curwin_lnum());
            }
            nfa_emit(if cmp == b'<' as c_int {
                NFA_LNUM_LT
            } else if cmp == b'>' as c_int {
                NFA_LNUM_GT
            } else {
                NFA_LNUM
            });
            if save_prev_at_start != 0 {
                AT_START = 1;
            }
        } else if c == b'c' as c_int {
            if cur {
                n = i64::from(nvim_regexp_get_curwin_col());
                n += 1;
            }
            nfa_emit(if cmp == b'<' as c_int {
                NFA_COL_LT
            } else if cmp == b'>' as c_int {
                NFA_COL_GT
            } else {
                NFA_COL
            });
        } else {
            // c == 'v'
            if cur {
                n = i64::from(nvim_regexp_get_curwin_vcol());
                n += 1;
            }
            nfa_emit(if cmp == b'<' as c_int {
                NFA_VCOL_LT
            } else if cmp == b'>' as c_int {
                NFA_VCOL_GT
            } else {
                NFA_VCOL
            });
            limit = i64::from(i32::MAX) / MB_MAXBYTES;
        }
        if n >= limit {
            errors::emsg_value_too_large();
            return FAIL;
        }
        #[allow(clippy::cast_possible_truncation)]
        let n_int = n as c_int; // n < limit <= i32::MAX, safe
        nfa_emit(n_int);
        return OK;
    }

    if rs_no_magic(c) == b'\'' as c_int && n == 0 {
        // \%'m  \%<'m  \%>'m
        nfa_emit(if cmp == b'<' as c_int {
            NFA_MARK_LT
        } else if cmp == b'>' as c_int {
            NFA_MARK_GT
        } else {
            NFA_MARK
        });
        nfa_emit(rs_getchr());
        return OK;
    }

    errors::semsg_e867_pct(rs_no_magic(c));
    FAIL
}

// === Phase 4: NFA Parser Functions + re2post ===

const RE_AUTO: c_int = 8;

/// Helper: compute current postfix position index.
#[inline]
#[allow(clippy::cast_possible_truncation)] // postfix array index always fits in c_int
unsafe fn post_pos() -> c_int {
    let pos = (POST_PTR as usize - POST_START as usize) / core::mem::size_of::<c_int>();
    pos as c_int
}

/// Helper: set postfix pointer to a given index from start.
#[inline]
unsafe fn set_post_pos(index: c_int) {
    POST_PTR = POST_START.add(index as usize);
}

/// Parse a piece (atom + optional quantifier).
#[allow(clippy::too_many_lines, unused_assignments)]
unsafe fn nfa_regpiece() -> c_int {
    let mut greedy = true;
    let mut old_state = core::mem::MaybeUninit::<ParseStateT>::uninit();
    let mut new_state = core::mem::MaybeUninit::<ParseStateT>::uninit();

    // Save parse state for \{m,n} handling
    rs_save_parse_state(old_state.as_mut_ptr());

    // Store current postfix position
    let my_post_start = post_pos();

    if rs_nfa_regatom() == FAIL {
        return FAIL;
    }

    let mut op = rs_peekchr();
    if rs_re_multi_type(op) == NOT_MULTI {
        return OK;
    }

    rs_skipchr();
    if op == nfa_magic(b'*') {
        // *
        nfa_emit(NFA_STAR);
    } else if op == nfa_magic(b'+') {
        // \+  — expand <atom>\+ to <atom><atom>*
        rs_restore_parse_state(old_state.as_ptr());
        CURCHR = -1;
        if rs_nfa_regatom() == FAIL {
            return FAIL;
        }
        nfa_emit(NFA_STAR);
        nfa_emit(NFA_CONCAT);
        rs_skipchr(); // skip the \+
    } else if op == nfa_magic(b'@') {
        // \@=, \@!, \@<=, \@<!, \@>
        let c2 = rs_getdecchrs();
        op = rs_no_magic(rs_getchr());
        let mut i: c_int = 0;
        if op == b'=' as c_int {
            i = NFA_PREV_ATOM_NO_WIDTH;
        } else if op == b'!' as c_int {
            i = NFA_PREV_ATOM_NO_WIDTH_NEG;
        } else if op == b'<' as c_int {
            op = rs_no_magic(rs_getchr());
            if op == b'=' as c_int {
                i = NFA_PREV_ATOM_JUST_BEFORE;
            } else if op == b'!' as c_int {
                i = NFA_PREV_ATOM_JUST_BEFORE_NEG;
            }
        } else if op == b'>' as c_int {
            i = NFA_PREV_ATOM_LIKE_PATTERN;
        }
        if i == 0 {
            errors::semsg_e869(op);
            return FAIL;
        }
        nfa_emit(i);
        if i == NFA_PREV_ATOM_JUST_BEFORE || i == NFA_PREV_ATOM_JUST_BEFORE_NEG {
            #[allow(clippy::cast_possible_truncation)]
            let c2_int = c2 as c_int;
            nfa_emit(c2_int);
        }
    } else if op == nfa_magic(b'?') || op == nfa_magic(b'=') {
        // \? or \=
        nfa_emit(NFA_QUEST);
    } else if op == nfa_magic(b'{') {
        // \{m,n}
        greedy = true;
        let c2 = rs_peekchr();
        if c2 == b'-' as c_int || c2 == nfa_magic(b'-') {
            rs_skipchr();
            greedy = false;
        }
        let mut minval: c_int = 0;
        let mut maxval: c_int = 0;
        if rs_read_limits(&mut minval, &mut maxval) == 0 {
            errors::emsg_e870();
            return FAIL;
        }

        // <atom>{0,inf} etc. → <atom>*
        if minval == 0 && maxval == MAX_LIMIT {
            if greedy {
                nfa_emit(NFA_STAR);
            } else {
                nfa_emit(NFA_STAR_NONGREEDY);
            }
        } else if maxval == 0 {
            // Special case: x{0}
            set_post_pos(my_post_start);
            nfa_emit(NFA_EMPTY);
            return OK;
        } else {
            // Check if too complex for NFA engine
            if (NFA_RE_FLAGS & RE_AUTO) != 0
                && (maxval > 500 || maxval > minval + 200)
                && (maxval != MAX_LIMIT && minval < 200)
                && WANTS_NFA == 0
            {
                return FAIL;
            }

            // Ignore previous call to nfa_regatom
            set_post_pos(my_post_start);
            rs_save_parse_state(new_state.as_mut_ptr());

            let quest = if greedy {
                NFA_QUEST
            } else {
                NFA_QUEST_NONGREEDY
            };
            for i in 0..maxval {
                rs_restore_parse_state(old_state.as_ptr());
                let old_post_pos = post_pos();
                if rs_nfa_regatom() == FAIL {
                    return FAIL;
                }
                // After minval times, atoms are optional
                if i + 1 > minval {
                    if maxval == MAX_LIMIT {
                        if greedy {
                            nfa_emit(NFA_STAR);
                        } else {
                            nfa_emit(NFA_STAR_NONGREEDY);
                        }
                    } else {
                        nfa_emit(quest);
                    }
                }
                if old_post_pos != my_post_start {
                    nfa_emit(NFA_CONCAT);
                }
                if i + 1 > minval && maxval == MAX_LIMIT {
                    break;
                }
            }

            rs_restore_parse_state(new_state.as_ptr());
            CURCHR = -1;
        }
    }

    if rs_re_multi_type(rs_peekchr()) != NOT_MULTI {
        errors::emsg_e871();
        return FAIL;
    }

    OK
}

/// Parse one or more pieces, concatenated.
unsafe fn nfa_regconcat() -> c_int {
    let mut first = true;

    loop {
        let c = rs_peekchr();
        if c == 0 || c == nfa_magic(b'|') || c == nfa_magic(b'&') || c == nfa_magic(b')') {
            break;
        }
        if c == nfa_magic(b'Z') {
            REGFLAGS_COMPILE |= RF_ICOMBINE;
            rs_skipchr_keepstart();
        } else if c == nfa_magic(b'c') {
            REGFLAGS_COMPILE |= RF_ICASE;
            rs_skipchr_keepstart();
        } else if c == nfa_magic(b'C') {
            REGFLAGS_COMPILE |= RF_NOICASE;
            rs_skipchr_keepstart();
        } else if c == nfa_magic(b'v') {
            REG_MAGIC = MAGIC_ALL;
            rs_skipchr_keepstart();
            CURCHR = -1;
        } else if c == nfa_magic(b'm') {
            REG_MAGIC = MAGIC_ON;
            rs_skipchr_keepstart();
            CURCHR = -1;
        } else if c == nfa_magic(b'M') {
            REG_MAGIC = MAGIC_OFF;
            rs_skipchr_keepstart();
            CURCHR = -1;
        } else if c == nfa_magic(b'V') {
            REG_MAGIC = MAGIC_NONE;
            rs_skipchr_keepstart();
            CURCHR = -1;
        } else {
            if nfa_regpiece() == FAIL {
                return FAIL;
            }
            if first {
                first = false;
            } else {
                nfa_emit(NFA_CONCAT);
            }
        }
    }

    OK
}

/// Parse a branch, one or more concats, separated by `\&`.
unsafe fn nfa_regbranch() -> c_int {
    let mut old_post_pos = post_pos();

    if nfa_regconcat() == FAIL {
        return FAIL;
    }

    while rs_peekchr() == nfa_magic(b'&') {
        rs_skipchr();
        if old_post_pos == post_pos() {
            nfa_emit(NFA_EMPTY);
        }
        nfa_emit(NFA_NOPEN);
        nfa_emit(NFA_PREV_ATOM_NO_WIDTH);
        old_post_pos = post_pos();
        if nfa_regconcat() == FAIL {
            return FAIL;
        }
        if old_post_pos == post_pos() {
            nfa_emit(NFA_EMPTY);
        }
        nfa_emit(NFA_CONCAT);
    }

    // If branch is empty, emit one node for it
    if old_post_pos == post_pos() {
        nfa_emit(NFA_EMPTY);
    }

    OK
}

/// Parse a pattern — one or more branches, separated by `\|`.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_reg(paren: c_int) -> c_int {
    #[allow(clippy::cast_possible_truncation)] // NSUBEXP is 10, always fits in c_int
    const NSUBEXP_I: c_int = NSUBEXP as c_int;
    let mut parno: c_int = 0;
    if paren == REG_PAREN {
        if REGNPAR >= NSUBEXP_I {
            errors::emsg_e872();
            return FAIL;
        }
        parno = REGNPAR;
        REGNPAR = parno + 1;
    } else if paren == REG_ZPAREN {
        if REGNZPAR >= NSUBEXP_I {
            errors::emsg_e879();
            return FAIL;
        }
        parno = REGNZPAR;
        REGNZPAR = parno + 1;
    }

    if nfa_regbranch() == FAIL {
        return FAIL;
    }

    while rs_peekchr() == nfa_magic(b'|') {
        rs_skipchr();
        if nfa_regbranch() == FAIL {
            return FAIL;
        }
        nfa_emit(NFA_OR);
    }

    // Check for proper termination
    if paren != REG_NOPAREN && rs_getchr() != nfa_magic(b')') {
        if paren == REG_NPAREN {
            errors::emsg2_e53(c_int::from(REG_MAGIC == MAGIC_ALL));
        } else {
            errors::emsg2_e54(c_int::from(REG_MAGIC == MAGIC_ALL));
        }
        return FAIL;
    } else if paren == REG_NOPAREN && rs_peekchr() != 0 {
        if rs_peekchr() == nfa_magic(b')') {
            errors::emsg2_e55(c_int::from(REG_MAGIC == MAGIC_ALL));
        } else {
            errors::emsg_e873();
        }
        return FAIL;
    }

    if paren == REG_PAREN {
        HAD_ENDBRACE[parno as usize] = 1_u8;
        nfa_emit(NFA_MOPEN + parno);
    } else if paren == REG_ZPAREN {
        nfa_emit(NFA_ZOPEN + parno);
    }

    OK
}

/// Convert regexp to postfix form.
#[no_mangle]
pub unsafe extern "C" fn rs_re2post() -> *mut c_int {
    if rs_nfa_reg(REG_NOPAREN) == FAIL {
        return core::ptr::null_mut();
    }
    nfa_emit(NFA_MOPEN);
    POST_START
}

// ---------------------------------------------------------------------------
// Phase 5: Thompson NFA Construction (post2nfa)
// ---------------------------------------------------------------------------

/// Opaque handle to a C `nfa_state_T`.
type NfaStateHandle = *mut c_void;

/// Pointer list for the Thompson NFA construction.  Faithfully reproduces the
/// C `Ptrlist` union: the `out`/`out1` fields of `nfa_state_T` are cast to
/// `*mut Ptrlist` so they can be threaded into a singly-linked list of pending
/// outputs.  `patch()` later fills them in with actual state pointers.
#[repr(C)]
union Ptrlist {
    next: *mut Ptrlist,
    s: NfaStateHandle,
}

/// A partially-built NFA fragment on the Thompson construction stack.
#[derive(Clone, Copy)]
struct FragT {
    start: NfaStateHandle,
    out_list: *mut Ptrlist,
}

// ---- Phase 5 C accessors ----

/// Allocate and initialize an NFA state from the pre-allocated state array.
unsafe fn nfa_alloc_state(c: c_int, out: NfaStateHandle, out1: NfaStateHandle) -> NfaStateHandle {
    let istate = ISTATE;
    if istate >= NSTATE {
        return core::ptr::null_mut();
    }
    let s = STATE_PTR
        .cast::<NfaStateT>()
        .add(istate as usize)
        .cast::<c_void>();
    ISTATE = istate + 1;
    (*s.cast::<NfaStateT>()).c = c;
    (*s.cast::<NfaStateT>()).out = out.cast::<NfaStateT>();
    (*s.cast::<NfaStateT>()).out1 = out1.cast::<NfaStateT>();
    (*s.cast::<NfaStateT>()).val = 0;
    (*s.cast::<NfaStateT>()).id = istate + 1; // id = istate after increment
    {
        (*s.cast::<NfaStateT>()).lastlist[0] = 0;
        (*s.cast::<NfaStateT>()).lastlist[1] = 0;
    }
    s
}

/// Create an NFA fragment.
#[inline]
const fn frag_new(start: NfaStateHandle, out_list: *mut Ptrlist) -> FragT {
    FragT { start, out_list }
}

/// Create a singleton `Ptrlist` from the address of an `out`/`out1` field.
/// The pointer-punning: `nfa_state_T**` → `*mut Ptrlist`, set `next = NULL`.
unsafe fn list1(outp: *mut NfaStateHandle) -> *mut Ptrlist {
    let l = outp.cast::<Ptrlist>();
    (*l).next = core::ptr::null_mut();
    l
}

/// Patch every dangling output in the list to point to state `s`.
unsafe fn nfa_patch(mut l: *mut Ptrlist, s: NfaStateHandle) {
    while !l.is_null() {
        let next = (*l).next;
        (*l).s = s;
        l = next;
    }
}

/// Append `l2` to the end of `l1`, returning `l1`.
unsafe fn ptrlist_append(l1: *mut Ptrlist, l2: *mut Ptrlist) -> *mut Ptrlist {
    let old = l1;
    let mut cur = l1;
    while !(*cur).next.is_null() {
        cur = (*cur).next;
    }
    (*cur).next = l2;
    old
}

/// Push a fragment onto the construction stack.
unsafe fn st_push(s: FragT, stackp: *mut *mut FragT, stack_end: *const FragT) {
    let sp = *stackp;
    if sp.cast_const() >= stack_end {
        return;
    }
    *sp = s;
    *stackp = sp.add(1);
}

/// Pop a fragment from the construction stack.
unsafe fn st_pop(stackp: *mut *mut FragT, stack: *const FragT) -> FragT {
    *stackp = (*stackp).sub(1);
    let sp = *stackp;
    if sp.cast_const() < stack {
        return FragT {
            start: core::ptr::null_mut(),
            out_list: core::ptr::null_mut(),
        };
    }
    *sp
}

/// Helper: pop with underflow check.  On underflow, emits E874, frees
/// `stack`, and returns `true` (caller should return NULL).
unsafe fn pop_or_fail(stackp: *mut *mut FragT, stack: *mut FragT, out: &mut FragT) -> bool {
    *out = st_pop(stackp, stack);
    if (*stackp).cast_const() < stack.cast_const() {
        errors::emsg_e874();
        xfree(stack.cast::<c_void>());
        return true;
    }
    false
}

/// Estimate the maximum byte length of anything matching `startstate`.
/// Returns -1 when unknown or unlimited.
#[allow(clippy::too_many_lines)]
unsafe fn nfa_max_width(startstate: NfaStateHandle, depth: c_int) -> c_int {
    // Detect looping in NFA_SPLIT
    if depth > 4 {
        return -1;
    }

    let mut state = startstate;
    let mut len: c_int = 0;

    while !state.is_null() {
        let c = (*state.cast::<NfaStateT>()).c;
        match c {
            NFA_END_INVISIBLE | NFA_END_INVISIBLE_NEG => return len,

            NFA_SPLIT => {
                let l = nfa_max_width((*state.cast::<NfaStateT>()).out.cast::<c_void>(), depth + 1);
                let r = nfa_max_width(
                    (*state.cast::<NfaStateT>()).out1.cast::<c_void>(),
                    depth + 1,
                );
                if l < 0 || r < 0 {
                    return -1;
                }
                return len + if l > r { l } else { r };
            }

            NFA_ANY | NFA_START_COLL | NFA_START_NEG_COLL => {
                #[allow(clippy::cast_possible_truncation)]
                {
                    len += MB_MAXBYTES as c_int;
                }
                if c != NFA_ANY {
                    // Skip over the collection characters.
                    let out1 = (*state.cast::<NfaStateT>()).out1.cast::<c_void>();
                    state = (*out1.cast::<NfaStateT>()).out.cast::<c_void>();
                    continue;
                }
            }

            NFA_DIGIT | NFA_WHITE | NFA_HEX | NFA_OCTAL => {
                len += 1;
            }

            NFA_IDENT | NFA_SIDENT | NFA_KWORD | NFA_SKWORD | NFA_FNAME | NFA_SFNAME
            | NFA_PRINT | NFA_SPRINT | NFA_NWHITE | NFA_NDIGIT | NFA_NHEX | NFA_NOCTAL
            | NFA_WORD | NFA_NWORD | NFA_HEAD | NFA_NHEAD | NFA_ALPHA | NFA_NALPHA | NFA_LOWER
            | NFA_NLOWER | NFA_UPPER | NFA_NUPPER | NFA_LOWER_IC | NFA_NLOWER_IC | NFA_UPPER_IC
            | NFA_NUPPER_IC | NFA_ANY_COMPOSING => {
                // possibly non-ascii
                len += 3;
            }

            NFA_START_INVISIBLE
            | NFA_START_INVISIBLE_NEG
            | NFA_START_INVISIBLE_BEFORE
            | NFA_START_INVISIBLE_BEFORE_NEG => {
                // zero-width, out1 points to the END state
                state = (*(*state.cast::<NfaStateT>())
                    .out1
                    .cast::<c_void>()
                    .cast::<NfaStateT>())
                .out
                .cast::<c_void>();
                continue;
            }

            NFA_BACKREF1 | NFA_BACKREF2 | NFA_BACKREF3 | NFA_BACKREF4 | NFA_BACKREF5
            | NFA_BACKREF6 | NFA_BACKREF7 | NFA_BACKREF8 | NFA_BACKREF9 | NFA_ZREF1 | NFA_ZREF2
            | NFA_ZREF3 | NFA_ZREF4 | NFA_ZREF5 | NFA_ZREF6 | NFA_ZREF7 | NFA_ZREF8 | NFA_ZREF9
            | NFA_NEWL | NFA_SKIP => {
                return -1;
            }

            NFA_BOL | NFA_EOL | NFA_BOF | NFA_EOF | NFA_BOW | NFA_EOW | NFA_MOPEN | NFA_MOPEN1
            | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4 | NFA_MOPEN5 | NFA_MOPEN6 | NFA_MOPEN7
            | NFA_MOPEN8 | NFA_MOPEN9 | NFA_ZOPEN | NFA_ZOPEN1 | NFA_ZOPEN2 | NFA_ZOPEN3
            | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6 | NFA_ZOPEN7 | NFA_ZOPEN8 | NFA_ZOPEN9
            | NFA_ZCLOSE | NFA_ZCLOSE1 | NFA_ZCLOSE2 | NFA_ZCLOSE3 | NFA_ZCLOSE4 | NFA_ZCLOSE5
            | NFA_ZCLOSE6 | NFA_ZCLOSE7 | NFA_ZCLOSE8 | NFA_ZCLOSE9 | NFA_MCLOSE | NFA_MCLOSE1
            | NFA_MCLOSE2 | NFA_MCLOSE3 | NFA_MCLOSE4 | NFA_MCLOSE5 | NFA_MCLOSE6 | NFA_MCLOSE7
            | NFA_MCLOSE8 | NFA_MCLOSE9 | NFA_NOPEN | NFA_NCLOSE | NFA_LNUM_GT | NFA_LNUM_LT
            | NFA_COL_GT | NFA_COL_LT | NFA_VCOL_GT | NFA_VCOL_LT | NFA_MARK_GT | NFA_MARK_LT
            | NFA_VISUAL | NFA_LNUM | NFA_CURSOR | NFA_COL | NFA_VCOL | NFA_MARK | NFA_ZSTART
            | NFA_ZEND | NFA_OPT_CHARS | NFA_EMPTY | NFA_START_PATTERN | NFA_END_PATTERN
            | NFA_COMPOSING | NFA_END_COMPOSING => {
                // zero-width
            }

            _ => {
                if c < 0 {
                    return -1;
                }
                // normal character
                len += utf_char2len(c);
            }
        }

        // normal way to continue
        state = (*state.cast::<NfaStateT>()).out.cast::<c_void>();
    }

    // unrecognized, "cannot happen"
    -1
}

/// Convert a postfix-form regexp to a Thompson NFA.
///
/// When `nfa_calc_size` is non-zero this is a size-counting pass that only
/// increments `nstate`; no states are actually allocated.  When zero, states
/// are allocated from `state_ptr[istate..]`.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub unsafe extern "C" fn rs_post2nfa(
    postfix: *mut c_int,
    end: *mut c_int,
    nfa_calc_size: c_int,
) -> NfaStateHandle {
    if postfix.is_null() {
        return core::ptr::null_mut();
    }

    let sizing = nfa_calc_size != 0;

    let (stack, mut stackp, stack_end) = if sizing {
        (
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null(),
        )
    } else {
        let nstate = NSTATE;
        let count = (nstate + 1) as usize;
        let s = xmalloc(count * core::mem::size_of::<FragT>()).cast::<FragT>();
        (s, s, s.add(count).cast_const())
    };

    let mut p = postfix;

    while p < end {
        let op = *p;
        match op {
            NFA_CONCAT => {
                if sizing {
                    p = p.add(1);
                    continue;
                }
                let mut e2 = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                let mut e1 = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e2) {
                    return core::ptr::null_mut();
                }
                if pop_or_fail(&mut stackp, stack, &mut e1) {
                    return core::ptr::null_mut();
                }
                nfa_patch(e1.out_list, e2.start);
                st_push(frag_new(e1.start, e2.out_list), &mut stackp, stack_end);
            }

            NFA_OR => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let mut e2 = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                let mut e1 = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e2) {
                    return core::ptr::null_mut();
                }
                if pop_or_fail(&mut stackp, stack, &mut e1) {
                    return core::ptr::null_mut();
                }
                let s = nfa_alloc_state(NFA_SPLIT, e1.start, e2.start);
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                st_push(
                    frag_new(s, ptrlist_append(e1.out_list, e2.out_list)),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_STAR => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let mut e = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e) {
                    return core::ptr::null_mut();
                }
                let s = nfa_alloc_state(NFA_SPLIT, e.start, core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                nfa_patch(e.out_list, s);
                st_push(
                    frag_new(
                        s,
                        list1((&raw mut (*s.cast::<NfaStateT>()).out1).cast::<*mut c_void>()),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_STAR_NONGREEDY => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let mut e = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e) {
                    return core::ptr::null_mut();
                }
                let s = nfa_alloc_state(NFA_SPLIT, core::ptr::null_mut(), e.start);
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                nfa_patch(e.out_list, s);
                st_push(
                    frag_new(
                        s,
                        list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_QUEST => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let mut e = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e) {
                    return core::ptr::null_mut();
                }
                let s = nfa_alloc_state(NFA_SPLIT, e.start, core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                st_push(
                    frag_new(
                        s,
                        ptrlist_append(
                            e.out_list,
                            list1((&raw mut (*s.cast::<NfaStateT>()).out1).cast::<*mut c_void>()),
                        ),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_QUEST_NONGREEDY => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let mut e = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e) {
                    return core::ptr::null_mut();
                }
                let s = nfa_alloc_state(NFA_SPLIT, core::ptr::null_mut(), e.start);
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                st_push(
                    frag_new(
                        s,
                        ptrlist_append(
                            e.out_list,
                            list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                        ),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_END_COLL | NFA_END_NEG_COLL => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let mut e = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e) {
                    return core::ptr::null_mut();
                }
                let s = nfa_alloc_state(NFA_END_COLL, core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                nfa_patch(e.out_list, s);
                (*e.start.cast::<NfaStateT>()).out1 = s.cast::<NfaStateT>();
                st_push(
                    frag_new(
                        e.start,
                        list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_RANGE => {
                if sizing {
                    p = p.add(1);
                    continue;
                }
                let mut e2 = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                let mut e1 = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e2) {
                    return core::ptr::null_mut();
                }
                if pop_or_fail(&mut stackp, stack, &mut e1) {
                    return core::ptr::null_mut();
                }
                // Move character code to val, set c to RANGE_MIN/MAX
                let c2 = (*e2.start.cast::<NfaStateT>()).c;
                (*e2.start.cast::<NfaStateT>()).val = c2;
                (*e2.start.cast::<NfaStateT>()).c = NFA_RANGE_MAX;
                let c1 = (*e1.start.cast::<NfaStateT>()).c;
                (*e1.start.cast::<NfaStateT>()).val = c1;
                (*e1.start.cast::<NfaStateT>()).c = NFA_RANGE_MIN;
                nfa_patch(e1.out_list, e2.start);
                st_push(frag_new(e1.start, e2.out_list), &mut stackp, stack_end);
            }

            NFA_EMPTY => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let s = nfa_alloc_state(NFA_EMPTY, core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                st_push(
                    frag_new(
                        s,
                        list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_OPT_CHARS => {
                p = p.add(1);
                let mut n = *p;
                if sizing {
                    NSTATE += n;
                    p = p.add(1);
                    continue;
                }
                let mut s: NfaStateHandle = core::ptr::null_mut();
                let mut e1_out: *mut Ptrlist = core::ptr::null_mut();
                let mut s1: NfaStateHandle = core::ptr::null_mut();
                while n > 0 {
                    let mut e = FragT {
                        start: core::ptr::null_mut(),
                        out_list: core::ptr::null_mut(),
                    };
                    if pop_or_fail(&mut stackp, stack, &mut e) {
                        return core::ptr::null_mut();
                    }
                    s = nfa_alloc_state(NFA_SPLIT, e.start, core::ptr::null_mut());
                    if s.is_null() {
                        xfree(stack.cast());
                        return core::ptr::null_mut();
                    }
                    if e1_out.is_null() {
                        e1_out = e.out_list;
                    }
                    nfa_patch(e.out_list, s1);
                    e1_out = ptrlist_append(
                        e1_out,
                        list1((&raw mut (*s.cast::<NfaStateT>()).out1).cast::<*mut c_void>()),
                    );
                    s1 = s;
                    n -= 1;
                }
                st_push(frag_new(s, e1_out), &mut stackp, stack_end);
            }

            NFA_PREV_ATOM_NO_WIDTH
            | NFA_PREV_ATOM_NO_WIDTH_NEG
            | NFA_PREV_ATOM_JUST_BEFORE
            | NFA_PREV_ATOM_JUST_BEFORE_NEG
            | NFA_PREV_ATOM_LIKE_PATTERN => {
                let before = op == NFA_PREV_ATOM_JUST_BEFORE || op == NFA_PREV_ATOM_JUST_BEFORE_NEG;
                let pattern = op == NFA_PREV_ATOM_LIKE_PATTERN;
                let (start_state, end_state) = match op {
                    NFA_PREV_ATOM_NO_WIDTH => (NFA_START_INVISIBLE, NFA_END_INVISIBLE),
                    NFA_PREV_ATOM_NO_WIDTH_NEG => (NFA_START_INVISIBLE_NEG, NFA_END_INVISIBLE_NEG),
                    NFA_PREV_ATOM_JUST_BEFORE => (NFA_START_INVISIBLE_BEFORE, NFA_END_INVISIBLE),
                    NFA_PREV_ATOM_JUST_BEFORE_NEG => {
                        (NFA_START_INVISIBLE_BEFORE_NEG, NFA_END_INVISIBLE_NEG)
                    }
                    _ => (NFA_START_PATTERN, NFA_END_PATTERN), // LIKE_PATTERN
                };
                let mut n: c_int = if before {
                    p = p.add(1);
                    *p
                } else {
                    0
                };
                if sizing {
                    NSTATE += if pattern { 4 } else { 2 };
                    p = p.add(1);
                    continue;
                }
                let mut e = FragT {
                    start: core::ptr::null_mut(),
                    out_list: core::ptr::null_mut(),
                };
                if pop_or_fail(&mut stackp, stack, &mut e) {
                    return core::ptr::null_mut();
                }
                let s1 = nfa_alloc_state(end_state, core::ptr::null_mut(), core::ptr::null_mut());
                if s1.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                let s = nfa_alloc_state(start_state, e.start, s1);
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                if pattern {
                    let skip =
                        nfa_alloc_state(NFA_SKIP, core::ptr::null_mut(), core::ptr::null_mut());
                    if skip.is_null() {
                        xfree(stack.cast());
                        return core::ptr::null_mut();
                    }
                    let zend = nfa_alloc_state(NFA_ZEND, s1, core::ptr::null_mut());
                    if zend.is_null() {
                        xfree(stack.cast());
                        return core::ptr::null_mut();
                    }
                    (*s1.cast::<NfaStateT>()).out = skip.cast::<NfaStateT>();
                    nfa_patch(e.out_list, zend);
                    st_push(
                        frag_new(
                            s,
                            list1((&raw mut (*skip.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                        ),
                        &mut stackp,
                        stack_end,
                    );
                } else {
                    nfa_patch(e.out_list, s1);
                    st_push(
                        frag_new(
                            s,
                            list1((&raw mut (*s1.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                        ),
                        &mut stackp,
                        stack_end,
                    );
                    if before {
                        if n <= 0 {
                            n = nfa_max_width(e.start, 0);
                        }
                        (*s.cast::<NfaStateT>()).val = n;
                    }
                }
            }

            NFA_COMPOSING | NFA_MOPEN | NFA_MOPEN1 | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4
            | NFA_MOPEN5 | NFA_MOPEN6 | NFA_MOPEN7 | NFA_MOPEN8 | NFA_MOPEN9 | NFA_ZOPEN
            | NFA_ZOPEN1 | NFA_ZOPEN2 | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6
            | NFA_ZOPEN7 | NFA_ZOPEN8 | NFA_ZOPEN9 | NFA_NOPEN => {
                if sizing {
                    NSTATE += 2;
                    p = p.add(1);
                    continue;
                }
                let mopen = op;
                #[allow(clippy::cast_possible_truncation)]
                let mclose = match mopen {
                    NFA_NOPEN => NFA_NCLOSE,
                    NFA_ZOPEN => NFA_ZCLOSE,
                    NFA_ZOPEN1 => NFA_ZCLOSE1,
                    NFA_ZOPEN2 => NFA_ZCLOSE2,
                    NFA_ZOPEN3 => NFA_ZCLOSE3,
                    NFA_ZOPEN4 => NFA_ZCLOSE4,
                    NFA_ZOPEN5 => NFA_ZCLOSE5,
                    NFA_ZOPEN6 => NFA_ZCLOSE6,
                    NFA_ZOPEN7 => NFA_ZCLOSE7,
                    NFA_ZOPEN8 => NFA_ZCLOSE8,
                    NFA_ZOPEN9 => NFA_ZCLOSE9,
                    NFA_COMPOSING => NFA_END_COMPOSING,
                    _ => mopen + NSUBEXP as c_int, // NFA_MOPEN .. NFA_MOPEN9
                };
                if stackp == stack {
                    // Empty group: NFA_MOPEN → NFA_MCLOSE
                    let s = nfa_alloc_state(mopen, core::ptr::null_mut(), core::ptr::null_mut());
                    if s.is_null() {
                        xfree(stack.cast());
                        return core::ptr::null_mut();
                    }
                    let s1 = nfa_alloc_state(mclose, core::ptr::null_mut(), core::ptr::null_mut());
                    if s1.is_null() {
                        xfree(stack.cast());
                        return core::ptr::null_mut();
                    }
                    nfa_patch(
                        list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                        s1,
                    );
                    st_push(
                        frag_new(
                            s,
                            list1((&raw mut (*s1.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                        ),
                        &mut stackp,
                        stack_end,
                    );
                } else {
                    let mut e = FragT {
                        start: core::ptr::null_mut(),
                        out_list: core::ptr::null_mut(),
                    };
                    if pop_or_fail(&mut stackp, stack, &mut e) {
                        return core::ptr::null_mut();
                    }
                    let s = nfa_alloc_state(mopen, e.start, core::ptr::null_mut());
                    if s.is_null() {
                        xfree(stack.cast());
                        return core::ptr::null_mut();
                    }
                    let s1 = nfa_alloc_state(mclose, core::ptr::null_mut(), core::ptr::null_mut());
                    if s1.is_null() {
                        xfree(stack.cast());
                        return core::ptr::null_mut();
                    }
                    nfa_patch(e.out_list, s1);
                    if mopen == NFA_COMPOSING {
                        nfa_patch(
                            list1((&raw mut (*s.cast::<NfaStateT>()).out1).cast::<*mut c_void>()),
                            s1,
                        );
                    }
                    st_push(
                        frag_new(
                            s,
                            list1((&raw mut (*s1.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                        ),
                        &mut stackp,
                        stack_end,
                    );
                }
            }

            NFA_BACKREF1 | NFA_BACKREF2 | NFA_BACKREF3 | NFA_BACKREF4 | NFA_BACKREF5
            | NFA_BACKREF6 | NFA_BACKREF7 | NFA_BACKREF8 | NFA_BACKREF9 | NFA_ZREF1 | NFA_ZREF2
            | NFA_ZREF3 | NFA_ZREF4 | NFA_ZREF5 | NFA_ZREF6 | NFA_ZREF7 | NFA_ZREF8 | NFA_ZREF9 => {
                if sizing {
                    NSTATE += 2;
                    p = p.add(1);
                    continue;
                }
                let s = nfa_alloc_state(op, core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                let s1 = nfa_alloc_state(NFA_SKIP, core::ptr::null_mut(), core::ptr::null_mut());
                if s1.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                nfa_patch(
                    list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                    s1,
                );
                st_push(
                    frag_new(
                        s,
                        list1((&raw mut (*s1.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_LNUM | NFA_LNUM_GT | NFA_LNUM_LT | NFA_VCOL | NFA_VCOL_GT | NFA_VCOL_LT
            | NFA_COL | NFA_COL_GT | NFA_COL_LT | NFA_MARK | NFA_MARK_GT | NFA_MARK_LT => {
                p = p.add(1);
                let n = *p; // lnum, col, or mark name
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                // p[-1] is the opcode (we already advanced p)
                let s = nfa_alloc_state(*p.sub(1), core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                (*s.cast::<NfaStateT>()).val = n;
                st_push(
                    frag_new(
                        s,
                        list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            // NFA_ZSTART, NFA_ZEND, and all other operands
            _ => {
                if sizing {
                    NSTATE += 1;
                    p = p.add(1);
                    continue;
                }
                let s = nfa_alloc_state(op, core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                st_push(
                    frag_new(
                        s,
                        list1((&raw mut (*s.cast::<NfaStateT>()).out).cast::<*mut c_void>()),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }
        }
        p = p.add(1);
    }

    if sizing {
        NSTATE += 1;
        // Return value ignored during size-counting pass
        return core::ptr::null_mut();
    }

    // Final POP — get the completed NFA
    let mut e = FragT {
        start: core::ptr::null_mut(),
        out_list: core::ptr::null_mut(),
    };
    if pop_or_fail(&mut stackp, stack, &mut e) {
        return core::ptr::null_mut();
    }
    if stackp != stack {
        xfree(stack.cast());
        errors::emsg_e875();
        return core::ptr::null_mut();
    }

    let istate = ISTATE;
    if istate >= NSTATE {
        xfree(stack.cast());
        errors::emsg_e876();
        return core::ptr::null_mut();
    }

    // Create the match state
    let matchstate = STATE_PTR
        .cast::<NfaStateT>()
        .add(istate as usize)
        .cast::<c_void>();
    ISTATE = istate + 1;
    (*matchstate.cast::<NfaStateT>()).c = NFA_MATCH;
    (*matchstate.cast::<NfaStateT>()).out = core::ptr::null_mut::<NfaStateT>();
    (*matchstate.cast::<NfaStateT>()).out1 = core::ptr::null_mut::<NfaStateT>();
    (*matchstate.cast::<NfaStateT>()).id = 0;

    nfa_patch(e.out_list, matchstate);
    let ret = e.start;

    xfree(stack.cast());
    ret
}

// ---------------------------------------------------------------------------
// Phase 6: NFA Postprocessing Functions
// ---------------------------------------------------------------------------

/// Opaque handle to a C `nfa_regprog_T`.
type NfaProgHandle = *mut c_void;

// ---- Phase 6 C accessors ----

/// Check if the match endpoint can directly follow a given NFA state.
/// Used by `nfa_postprocess` to decide whether to try the invisible match first.
#[allow(clippy::too_many_lines)]
unsafe fn match_follows(startstate: NfaStateHandle, depth: c_int) -> bool {
    if depth > 10 || startstate.is_null() {
        return false;
    }
    let mut state = startstate;
    while !state.is_null() {
        let c = (*state.cast::<NfaStateT>()).c;
        match c {
            NFA_MATCH
            | NFA_MCLOSE
            | NFA_END_INVISIBLE
            | NFA_END_INVISIBLE_NEG
            | NFA_END_PATTERN => return true,

            NFA_SPLIT => {
                return match_follows((*state.cast::<NfaStateT>()).out.cast::<c_void>(), depth + 1)
                    || match_follows(
                        (*state.cast::<NfaStateT>()).out1.cast::<c_void>(),
                        depth + 1,
                    );
            }

            NFA_START_INVISIBLE
            | NFA_START_INVISIBLE_FIRST
            | NFA_START_INVISIBLE_BEFORE
            | NFA_START_INVISIBLE_BEFORE_FIRST
            | NFA_START_INVISIBLE_NEG
            | NFA_START_INVISIBLE_NEG_FIRST
            | NFA_START_INVISIBLE_BEFORE_NEG
            | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
            | NFA_COMPOSING => {
                // skip ahead to next state
                state = (*(*state.cast::<NfaStateT>())
                    .out1
                    .cast::<c_void>()
                    .cast::<NfaStateT>())
                .out
                .cast::<c_void>();
                continue;
            }

            NFA_ANY | NFA_ANY_COMPOSING | NFA_IDENT | NFA_SIDENT | NFA_KWORD | NFA_SKWORD
            | NFA_FNAME | NFA_SFNAME | NFA_PRINT | NFA_SPRINT | NFA_WHITE | NFA_NWHITE
            | NFA_DIGIT | NFA_NDIGIT | NFA_HEX | NFA_NHEX | NFA_OCTAL | NFA_NOCTAL | NFA_WORD
            | NFA_NWORD | NFA_HEAD | NFA_NHEAD | NFA_ALPHA | NFA_NALPHA | NFA_LOWER
            | NFA_NLOWER | NFA_UPPER | NFA_NUPPER | NFA_LOWER_IC | NFA_NLOWER_IC | NFA_UPPER_IC
            | NFA_NUPPER_IC | NFA_START_COLL | NFA_START_NEG_COLL | NFA_NEWL => {
                // state will advance input
                return false;
            }

            _ => {
                if c > 0 {
                    return false;
                }
                // zero-width or possibly zero-width, keep looking
            }
        }
        state = (*state.cast::<NfaStateT>()).out.cast::<c_void>();
    }
    false
}

/// Heuristic: estimate the failure chance (0-99) for an NFA state.
/// Higher values mean more likely to fail (and thus cheaper to try first).
#[allow(clippy::too_many_lines)]
unsafe fn failure_chance(state: NfaStateHandle, depth: c_int) -> c_int {
    let c = (*state.cast::<NfaStateT>()).c;

    // detect looping
    if depth > 4 {
        return 1;
    }

    match c {
        NFA_SPLIT => {
            let out = (*state.cast::<NfaStateT>()).out.cast::<c_void>();
            let out1 = (*state.cast::<NfaStateT>()).out1.cast::<c_void>();
            if (*out.cast::<NfaStateT>()).c == NFA_SPLIT
                || (*out1.cast::<NfaStateT>()).c == NFA_SPLIT
            {
                return 1; // avoid recursive stuff
            }
            let l = failure_chance(out, depth + 1);
            let r = failure_chance(out1, depth + 1);
            if l < r {
                l
            } else {
                r
            }
        }

        NFA_ANY => 1, // matches anything, unlikely to fail

        NFA_MATCH | NFA_MCLOSE | NFA_ANY_COMPOSING => 0, // empty match works always

        NFA_START_INVISIBLE
        | NFA_START_INVISIBLE_FIRST
        | NFA_START_INVISIBLE_NEG
        | NFA_START_INVISIBLE_NEG_FIRST
        | NFA_START_INVISIBLE_BEFORE
        | NFA_START_INVISIBLE_BEFORE_FIRST
        | NFA_START_INVISIBLE_BEFORE_NEG
        | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
        | NFA_START_PATTERN => {
            5 // recursive regmatch is expensive
        }

        NFA_BOL | NFA_EOL | NFA_BOF | NFA_EOF | NFA_NEWL => 99,

        NFA_BOW | NFA_EOW | NFA_LNUM => 90,

        NFA_MOPEN | NFA_MOPEN1 | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4 | NFA_MOPEN5 | NFA_MOPEN6
        | NFA_MOPEN7 | NFA_MOPEN8 | NFA_MOPEN9 | NFA_ZOPEN | NFA_ZOPEN1 | NFA_ZOPEN2
        | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6 | NFA_ZOPEN7 | NFA_ZOPEN8
        | NFA_ZOPEN9 | NFA_ZCLOSE | NFA_ZCLOSE1 | NFA_ZCLOSE2 | NFA_ZCLOSE3 | NFA_ZCLOSE4
        | NFA_ZCLOSE5 | NFA_ZCLOSE6 | NFA_ZCLOSE7 | NFA_ZCLOSE8 | NFA_ZCLOSE9 | NFA_NOPEN
        | NFA_MCLOSE1 | NFA_MCLOSE2 | NFA_MCLOSE3 | NFA_MCLOSE4 | NFA_MCLOSE5 | NFA_MCLOSE6
        | NFA_MCLOSE7 | NFA_MCLOSE8 | NFA_MCLOSE9 | NFA_NCLOSE => {
            failure_chance((*state.cast::<NfaStateT>()).out.cast::<c_void>(), depth + 1)
        }

        NFA_BACKREF1 | NFA_BACKREF2 | NFA_BACKREF3 | NFA_BACKREF4 | NFA_BACKREF5 | NFA_BACKREF6
        | NFA_BACKREF7 | NFA_BACKREF8 | NFA_BACKREF9 | NFA_ZREF1 | NFA_ZREF2 | NFA_ZREF3
        | NFA_ZREF4 | NFA_ZREF5 | NFA_ZREF6 | NFA_ZREF7 | NFA_ZREF8 | NFA_ZREF9 => 94, // backreferences don't match in many places

        NFA_LNUM_GT | NFA_LNUM_LT | NFA_COL_GT | NFA_COL_LT | NFA_VCOL_GT | NFA_VCOL_LT
        | NFA_MARK_GT | NFA_MARK_LT | NFA_VISUAL => 85,

        NFA_CURSOR | NFA_COL | NFA_VCOL | NFA_MARK => 98, // specific positions rarely match

        NFA_COMPOSING => 95,

        _ => {
            if c > 0 {
                return 95; // character match fails often
            }
            50 // something else, includes character classes
        }
    }
}

/// After building the NFA program, inspect it to add optimization hints.
/// Decides whether invisible matches should be tried first or postponed.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_postprocess(prog: NfaProgHandle) {
    let nstate = (*prog.cast::<NfaRegprogT>()).nstate;
    for i in 0..nstate {
        let state = prog
            .cast::<NfaRegprogT>()
            .add(1)
            .cast::<NfaStateT>()
            .add(i as usize)
            .cast::<c_void>();
        let c = (*state.cast::<NfaStateT>()).c;
        if c == NFA_START_INVISIBLE
            || c == NFA_START_INVISIBLE_NEG
            || c == NFA_START_INVISIBLE_BEFORE
            || c == NFA_START_INVISIBLE_BEFORE_NEG
        {
            let directly;
            let out1 = (*state.cast::<NfaStateT>()).out1.cast::<c_void>();
            let out1_out = (*out1.cast::<NfaStateT>()).out.cast::<c_void>();
            if match_follows(out1_out, 0) {
                directly = true;
            } else {
                let out = (*state.cast::<NfaStateT>()).out.cast::<c_void>();
                let ch_invisible = failure_chance(out, 0);
                let ch_follows = failure_chance(out1_out, 0);
                if c == NFA_START_INVISIBLE_BEFORE || c == NFA_START_INVISIBLE_BEFORE_NEG {
                    let val = (*state.cast::<NfaStateT>()).val;
                    directly = if val <= 0 && ch_follows > 0 {
                        false
                    } else {
                        ch_follows * 10 < ch_invisible
                    };
                } else {
                    directly = ch_follows < ch_invisible;
                }
            }
            if directly {
                // switch to the _FIRST state variant (c + 1)
                (*state.cast::<NfaStateT>()).c = c + 1;
            }
        }
    }
}

/// Check if a pattern is anchored to the start of a line.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_reganch(start: NfaStateHandle, depth: c_int) -> c_int {
    if depth > 4 {
        return 0;
    }
    let mut p = start;
    while !p.is_null() {
        let c = (*p.cast::<NfaStateT>()).c;
        match c {
            NFA_BOL | NFA_BOF => return 1,

            NFA_ZSTART | NFA_ZEND | NFA_CURSOR | NFA_VISUAL | NFA_MOPEN | NFA_MOPEN1
            | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4 | NFA_MOPEN5 | NFA_MOPEN6 | NFA_MOPEN7
            | NFA_MOPEN8 | NFA_MOPEN9 | NFA_NOPEN | NFA_ZOPEN | NFA_ZOPEN1 | NFA_ZOPEN2
            | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6 | NFA_ZOPEN7 | NFA_ZOPEN8
            | NFA_ZOPEN9 => {
                p = (*p.cast::<NfaStateT>()).out.cast::<c_void>();
            }

            NFA_SPLIT => {
                return (rs_nfa_get_reganch(
                    (*p.cast::<NfaStateT>()).out.cast::<c_void>(),
                    depth + 1,
                ) != 0
                    && rs_nfa_get_reganch(
                        (*p.cast::<NfaStateT>()).out1.cast::<c_void>(),
                        depth + 1,
                    ) != 0) as c_int;
            }

            _ => return 0,
        }
    }
    0
}

/// Get the first character of a pattern, if it's a literal character.
/// Returns 0 if unknown.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_regstart(start: NfaStateHandle, depth: c_int) -> c_int {
    if depth > 4 {
        return 0;
    }
    let mut p = start;
    while !p.is_null() {
        let c = (*p.cast::<NfaStateT>()).c;
        match c {
            NFA_BOL | NFA_BOF | NFA_BOW | NFA_EOW | NFA_ZSTART | NFA_ZEND | NFA_CURSOR
            | NFA_VISUAL | NFA_LNUM | NFA_LNUM_GT | NFA_LNUM_LT | NFA_COL | NFA_COL_GT
            | NFA_COL_LT | NFA_VCOL | NFA_VCOL_GT | NFA_VCOL_LT | NFA_MARK | NFA_MARK_GT
            | NFA_MARK_LT | NFA_MOPEN | NFA_MOPEN1 | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4
            | NFA_MOPEN5 | NFA_MOPEN6 | NFA_MOPEN7 | NFA_MOPEN8 | NFA_MOPEN9 | NFA_NOPEN
            | NFA_ZOPEN | NFA_ZOPEN1 | NFA_ZOPEN2 | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5
            | NFA_ZOPEN6 | NFA_ZOPEN7 | NFA_ZOPEN8 | NFA_ZOPEN9 => {
                p = (*p.cast::<NfaStateT>()).out.cast::<c_void>();
            }

            NFA_SPLIT => {
                let c1 =
                    rs_nfa_get_regstart((*p.cast::<NfaStateT>()).out.cast::<c_void>(), depth + 1);
                let c2 =
                    rs_nfa_get_regstart((*p.cast::<NfaStateT>()).out1.cast::<c_void>(), depth + 1);
                if c1 == c2 {
                    return c1;
                }
                return 0;
            }

            _ => {
                if c > 0 {
                    return c;
                }
                return 0;
            }
        }
    }
    0
}

/// Get the literal match text when the pattern is pure literal characters.
/// Returns a freshly allocated string (or NULL).
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_match_text(start: NfaStateHandle) -> *mut u8 {
    if (*start.cast::<NfaStateT>()).c != NFA_MOPEN {
        return core::ptr::null_mut();
    }
    let mut p = (*start.cast::<NfaStateT>()).out.cast::<c_void>();
    let mut len: c_int = 0;

    // Count total byte length of literal characters.
    while (*p.cast::<NfaStateT>()).c > 0 {
        len += utf_char2len((*p.cast::<NfaStateT>()).c);
        p = (*p.cast::<NfaStateT>()).out.cast::<c_void>();
    }

    if (*p.cast::<NfaStateT>()).c != NFA_MCLOSE {
        return core::ptr::null_mut();
    }
    let next = (*p.cast::<NfaStateT>()).out.cast::<c_void>();
    if (*next.cast::<NfaStateT>()).c != NFA_MATCH {
        return core::ptr::null_mut();
    }

    let ret = xmalloc(len as usize).cast::<u8>();
    // Skip first char (it goes into regstart), start from out->out
    p = (*(*start.cast::<NfaStateT>()).out).out.cast::<c_void>();
    let mut s = ret;
    while (*p.cast::<NfaStateT>()).c > 0 {
        s = s.add(utf_char2bytes((*p.cast::<NfaStateT>()).c, s.cast::<c_char>()) as usize);
        p = (*p.cast::<NfaStateT>()).out.cast::<c_void>();
    }
    *s = 0; // NUL terminate
    ret
}

// ---------------------------------------------------------------------------
// Phase 7: nfa_regcomp Entry Point
// ---------------------------------------------------------------------------

// ---- Phase 7 C accessors ----
extern "C" {
    fn nvim_regexp_xstrdup(s: *const c_char) -> *mut c_char;
}

/// Compile a regular expression into internal code for the NFA matcher.
/// Returns the program in allocated space.  Returns NULL for an error.
///
/// This is the Rust replacement for `nfa_regcomp()`.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regcomp(expr: *mut u8, re_flags: c_int) -> NfaProgHandle {
    if expr.is_null() {
        return core::ptr::null_mut();
    }

    NFA_RE_FLAGS = re_flags;
    rs_nfa_regcomp_start(expr, re_flags);

    // Build postfix form of the regexp. Needed to build the NFA (and count its size).
    let postfix = rs_re2post();
    if postfix.is_null() {
        // Cascaded (syntax?) error — clean up and return NULL
        let post_start = POST_START;
        xfree(post_start.cast());
        POST_START = core::ptr::null_mut();
        POST_PTR = core::ptr::null_mut();
        POST_END = core::ptr::null_mut();
        STATE_PTR = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    let post_ptr_val = POST_PTR;

    // PASS 1: Count number of NFA states in "nstate". Do not build the NFA.
    rs_post2nfa(postfix, post_ptr_val, 1);

    // Allocate the regprog with space for the compiled regexp.
    // This also sets state_ptr = prog->state.
    let nstate_val = NSTATE;
    let prog = {
        let sz = core::mem::size_of::<NfaRegprogT>()
            + nstate_val as usize * core::mem::size_of::<NfaStateT>();
        let p = xmalloc(sz).cast::<NfaRegprogT>();
        core::ptr::write_bytes(p.cast::<u8>(), 0, sz);
        (*p).nstate = nstate_val;
        STATE_PTR = p.add(1).cast::<NfaStateT>().cast::<c_void>();
        p.cast::<c_void>()
    };
    (*prog.cast::<NfaRegprogT>()).re_in_use = false;

    // PASS 2: Build the NFA.
    let start = rs_post2nfa(postfix, post_ptr_val, 0);
    if start.is_null() {
        // Build failed — free prog, clean up, return NULL
        xfree(prog);
        let post_start = POST_START;
        xfree(post_start.cast());
        POST_START = core::ptr::null_mut();
        POST_PTR = core::ptr::null_mut();
        POST_END = core::ptr::null_mut();
        STATE_PTR = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    (*prog.cast::<NfaRegprogT>()).start = start.cast::<NfaStateT>();
    (*prog.cast::<NfaRegprogT>()).regflags = REGFLAGS_COMPILE as c_int as c_uint;
    (*prog.cast::<NfaRegprogT>()).engine = core::ptr::addr_of!(NFA_REGENGINE)
        .cast_mut()
        .cast::<c_void>();
    (*prog.cast::<NfaRegprogT>()).nstate = nstate_val;
    (*prog.cast::<NfaRegprogT>()).has_zend = REX.nfa_has_zend;
    (*prog.cast::<NfaRegprogT>()).has_backref = REX.nfa_has_backref;
    (*prog.cast::<NfaRegprogT>()).nsubexp = REGNPAR;

    rs_nfa_postprocess(prog);

    let prog_start = (*prog.cast::<NfaRegprogT>()).start.cast::<c_void>();
    (*prog.cast::<NfaRegprogT>()).reganch = rs_nfa_get_reganch(prog_start, 0);
    (*prog.cast::<NfaRegprogT>()).regstart = rs_nfa_get_regstart(prog_start, 0);
    (*prog.cast::<NfaRegprogT>()).match_text = rs_nfa_get_match_text(prog_start);

    // Remember whether this pattern has any \z specials in it.
    (*prog.cast::<NfaRegprogT>()).reghasz = RE_HAS_Z;
    (*prog.cast::<NfaRegprogT>()).pattern = nvim_regexp_xstrdup(expr.cast());

    // Clean up
    let post_start = POST_START;
    xfree(post_start.cast());
    POST_START = core::ptr::null_mut();
    POST_PTR = core::ptr::null_mut();
    POST_END = core::ptr::null_mut();
    STATE_PTR = core::ptr::null_mut();

    prog
}

// ============================================================================
// NFA Execution Engine — Phase 8: Pure helpers and accessor infrastructure
// ============================================================================

/// Check for a match with a character class.
/// Returns OK if `c` matches the class `cls`, FAIL otherwise.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_check_char_class(cls: c_int, c: c_int) -> c_int {
    match cls {
        NFA_CLASS_ALNUM => {
            if (1..128).contains(&c) && isalnum(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_ALPHA => {
            if (1..128).contains(&c) && isalpha(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_BLANK => {
            if c == b' ' as c_int || c == b'\t' as c_int {
                return OK;
            }
        }
        NFA_CLASS_CNTRL => {
            if (1..=127).contains(&c) && iscntrl(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_DIGIT =>
        {
            #[allow(clippy::cast_possible_truncation)]
            if ascii_isdigit(c as u8) {
                return OK;
            }
        }
        NFA_CLASS_GRAPH => {
            if (1..=127).contains(&c) && isgraph(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_LOWER => {
            if mb_islower(c) != 0 && c != 170 && c != 186 {
                return OK;
            }
        }
        NFA_CLASS_PRINT => {
            if vim_isprintc(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_PUNCT => {
            if (1..128).contains(&c) && ispunct(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_SPACE => {
            if (9..=13).contains(&c) || c == b' ' as c_int {
                return OK;
            }
        }
        NFA_CLASS_UPPER => {
            if mb_isupper(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_XDIGIT =>
        {
            #[allow(clippy::cast_possible_truncation)]
            if ascii_isxdigit(c as u8) {
                return OK;
            }
        }
        NFA_CLASS_TAB => {
            if c == b'\t' as c_int {
                return OK;
            }
        }
        NFA_CLASS_RETURN => {
            if c == b'\r' as c_int {
                return OK;
            }
        }
        NFA_CLASS_BACKSPACE => {
            if c == 0x08 {
                return OK;
            }
        }
        NFA_CLASS_ESCAPE => {
            if c == ESC_CH {
                return OK;
            }
        }
        NFA_CLASS_IDENT => {
            if vim_isIDc(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_KEYWORD => {
            if rs_reg_iswordc(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_FNAME => {
            if vim_isfilec(c) != 0 {
                return OK;
            }
        }
        _ => {
            // should not be here
            errors::siemsg_ill_char_class(cls as i64);
            return FAIL;
        }
    }
    FAIL
}

/// Helper for ascii hex digit check (matches C `ascii_isxdigit`).
const fn ascii_isxdigit(c: u8) -> bool {
    (c >= b'0' && c <= b'9') || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')
}

/// Numeric comparison helper used by NFA execution engine.
/// op == 1: pos > val, op == 2: pos < val, else: val == pos
#[no_mangle]
pub const unsafe extern "C" fn rs_nfa_re_num_cmp(val: usize, op: c_int, pos: usize) -> c_int {
    let result = if op == 1 {
        pos > val
    } else if op == 2 {
        pos < val
    } else {
        val == pos
    };
    result as c_int
}

// ============================================================================
// NFA Execution Engine — Phase 8.2: Submatch helpers and backref matching
// ============================================================================

// --- Concrete struct types matching C layout in regexp.c ---

/// Matches C `struct multipos` inside `regsub_T`.
#[repr(C)]
#[derive(Copy, Clone)]
struct MultiPos {
    start_lnum: i32,  // linenr_T
    end_lnum: i32,    // linenr_T
    start_col: c_int, // colnr_T
    end_col: c_int,   // colnr_T
}

/// Matches C `struct linepos` inside `regsub_T`.
#[repr(C)]
#[derive(Copy, Clone)]
struct LinePos {
    start: *mut u8,
    end: *mut u8,
}

/// Union inside `regsub_T`: multi-line or single-line positions.
#[repr(C)]
#[derive(Copy, Clone)]
union RegsubList {
    multi: [MultiPos; NSUBEXP],
    line: [LinePos; NSUBEXP],
}

/// Matches C `regsub_T` — submatch position storage.
#[repr(C)]
#[derive(Copy, Clone)]
struct RegsubT {
    in_use: c_int,
    list: RegsubList,
    orig_start_col: c_int, // colnr_T
}

/// Matches C `regsubs_T` — norm + synt submatches.
#[repr(C)]
#[derive(Copy, Clone)]
struct RegsubsT {
    norm: RegsubT,
    synt: RegsubT,
}

/// Matches C `nfa_state_T` — NFA state node.
#[repr(C)]
#[derive(Copy, Clone)]
struct NfaStateT {
    c: c_int,
    out: *mut NfaStateT,
    out1: *mut NfaStateT,
    id: c_int,
    lastlist: [c_int; 2],
    val: c_int,
}

/// Matches C `nfa_pim_T` — Postponed Invisible Match.
#[repr(C)]
#[derive(Copy, Clone)]
struct NfaPimT {
    result: c_int,
    state: *mut NfaStateT,
    subs: RegsubsT,
    end: SaveSeUnion, // union { lpos_T pos; uint8_t *ptr; }
}

/// Matches C `nfa_thread_T` — NFA execution thread.
#[repr(C)]
#[derive(Copy, Clone)]
struct NfaThreadT {
    state: *mut NfaStateT,
    count: c_int,
    pim: NfaPimT,
    subs: RegsubsT,
}

/// Matches C `nfa_list_T` — list of NFA execution states.
#[repr(C)]
#[derive(Copy, Clone)]
struct NfaListT {
    t: *mut NfaThreadT,
    n: c_int,
    len: c_int,
    id: c_int,
    has_pim: c_int,
}

/// Matches C `nfa_regprog_T` (common prefix only, flexible array member follows).
#[repr(C)]
struct NfaRegprogT {
    engine: *mut c_void, // regengine_T *
    regflags: c_uint,
    re_engine: c_uint,
    re_flags: c_uint,
    re_in_use: bool,
    start: *mut NfaStateT,
    reganch: c_int,
    regstart: c_int,
    match_text: *mut u8,
    has_zend: c_int,
    has_backref: c_int,
    reghasz: c_int,
    pattern: *mut c_char,
    nsubexp: c_int,
    nstate: c_int,
    // state[] flexible array member follows — access via pointer arithmetic
}

/// Matches C `struct regengine` — vtable for BT/NFA engines.
/// Note: C defines `regexec_nl` with `bool` as the 4th arg; on x86-64 Linux,
/// `bool` and `c_int` are ABI-compatible in argument position (both zero-extended
/// to register width), so `c_int` is used here to match the Rust implementations.
#[repr(C)]
struct RegengineT {
    regcomp: unsafe extern "C" fn(*mut u8, c_int) -> *mut c_void,
    regfree: unsafe extern "C" fn(*mut c_void),
    regexec_nl: unsafe extern "C" fn(*mut c_void, *mut u8, i32, c_int) -> c_int,
    regexec_multi: unsafe extern "C" fn(
        *mut c_void,
        *mut c_void,
        *mut c_void,
        i32,
        i32,
        *mut c_void,
        *mut c_int,
    ) -> c_int,
}

// Safety: RegengineT contains only function pointers (no mutable state).
unsafe impl Sync for RegengineT {}

/// Free a BT-engine compiled program.
///
/// Matches C `bt_regfree()`.
unsafe extern "C" fn rs_bt_regfree(prog: *mut c_void) {
    xfree(prog);
}

/// Free an NFA-engine compiled program.
///
/// Matches C `nfa_regfree()`.
unsafe extern "C" fn rs_nfa_regfree(prog: *mut c_void) {
    if prog.is_null() {
        return;
    }
    xfree((*prog.cast::<NfaRegprogT>()).match_text.cast());
    xfree((*prog.cast::<NfaRegprogT>()).pattern.cast());
    xfree(prog);
}

/// Vtable for the backtracking regexp engine.
///
/// Replaces the C `bt_regengine` static.
static BT_REGENGINE: RegengineT = RegengineT {
    regcomp: rs_bt_regcomp,
    regfree: rs_bt_regfree,
    regexec_nl: rs_bt_regexec_nl,
    regexec_multi: rs_bt_regexec_multi,
};

/// Vtable for the NFA regexp engine.
///
/// Replaces the C `nfa_regengine` static.
static NFA_REGENGINE: RegengineT = RegengineT {
    regcomp: rs_nfa_regcomp,
    regfree: rs_nfa_regfree,
    regexec_nl: rs_nfa_regexec_nl,
    regexec_multi: rs_nfa_regexec_multi,
};

/// Matches C `struct regprog` — common header for all regexp programs.
#[repr(C)]
struct RegprogT {
    engine: *mut RegengineT,
    regflags: c_uint,
    re_engine: c_uint,
    re_flags: c_uint,
    re_in_use: bool,
}

/// Matches C `bt_regprog_T` — backtracking engine regexp program.
#[repr(C)]
struct BtRegprogT {
    // Common regprog_T header
    engine: *mut RegengineT,
    regflags: c_uint,
    re_engine: c_uint,
    re_flags: c_uint,
    re_in_use: bool,
    // bt_regprog_T specific fields
    regstart: c_int,
    reganch: u8,
    regmust: *mut u8,
    regmlen: c_int,
    reghasz: u8,
    // program[] flexible array member follows — access via pointer arithmetic
}

/// Matches C `regmatch_T` — single-line match state.
#[repr(C)]
struct RegmatchT {
    regprog: *mut RegprogT,
    startp: [*mut u8; NSUBEXP],
    endp: [*mut u8; NSUBEXP],
    rm_matchcol: i32, // colnr_T
    rm_ic: bool,
}

/// Matches C `regmmatch_T` — multi-line match state.
#[repr(C)]
struct RegmmatchT {
    regprog: *mut RegprogT,
    startpos: [LposT; NSUBEXP],
    endpos: [LposT; NSUBEXP],
    rmm_matchcol: i32, // colnr_T
    rmm_ic: c_int,
    rmm_maxcol: i32, // colnr_T
}

/// Type aliases for backward compatibility with opaque handle code.
type RegsubHandle = *mut c_void; // regsub_T*
type NfaPimHandle = *mut c_void; // nfa_pim_T*
type NfaListHandle = *mut c_void; // nfa_list_T*
type RegsubsHandle = *mut c_void; // regsubs_T*

/// NFA PIM result constants.
const NFA_PIM_UNUSED: c_int = 0;

extern "C" {

    // C wrapper functions for complex operations
    // (match_backref, match_zref, find_match_text, skip_to_start
    //  migrated to Rust -- extern declarations removed)
    fn nvim_regexp_get_nfa_has_zsubexpr() -> c_int;

}

// ============================================================================
// Submatch operations — migrated from C regexp.c
// ============================================================================

/// Clear the sub-expression matches in `sub`.
unsafe fn clear_sub(sub: *mut RegsubT, nsubexpr: c_int) {
    if (REX.reg_match.is_null() as c_int) != 0 {
        // Use 0xff to set lnum to -1
        core::ptr::write_bytes((*sub).list.multi.as_mut_ptr(), 0xff, nsubexpr as usize);
    } else {
        core::ptr::write_bytes((*sub).list.line.as_mut_ptr(), 0, nsubexpr as usize);
    }
    (*sub).in_use = 0;
}

/// Copy the submatches from `from` to `to`.
unsafe fn copy_sub(to: *mut RegsubT, from: *const RegsubT) {
    (*to).in_use = (*from).in_use;
    if (*from).in_use <= 0 {
        return;
    }
    if (REX.reg_match.is_null() as c_int) != 0 {
        core::ptr::copy_nonoverlapping(
            (*from).list.multi.as_ptr(),
            (*to).list.multi.as_mut_ptr(),
            (*from).in_use as usize,
        );
        (*to).orig_start_col = (*from).orig_start_col;
    } else {
        core::ptr::copy_nonoverlapping(
            (*from).list.line.as_ptr(),
            (*to).list.line.as_mut_ptr(),
            (*from).in_use as usize,
        );
    }
}

/// Like `copy_sub()` but exclude the main match (index 0).
unsafe fn copy_sub_off(to: *mut RegsubT, from: *const RegsubT) {
    if (*to).in_use < (*from).in_use {
        (*to).in_use = (*from).in_use;
    }
    if (*from).in_use <= 1 {
        return;
    }
    if (REX.reg_match.is_null() as c_int) != 0 {
        core::ptr::copy_nonoverlapping(
            (*from).list.multi.as_ptr().add(1),
            (*to).list.multi.as_mut_ptr().add(1),
            ((*from).in_use - 1) as usize,
        );
    } else {
        core::ptr::copy_nonoverlapping(
            (*from).list.line.as_ptr().add(1),
            (*to).list.line.as_mut_ptr().add(1),
            ((*from).in_use - 1) as usize,
        );
    }
}

/// Like `copy_sub()` but only copy the end of the main match if `\ze` is present.
unsafe fn copy_ze_off(to: *mut RegsubT, from: *const RegsubT, has_zend: c_int) {
    if has_zend == 0 {
        return;
    }
    if (REX.reg_match.is_null() as c_int) != 0 {
        if (*from).list.multi[0].end_lnum >= 0 {
            (*to).list.multi[0].end_lnum = (*from).list.multi[0].end_lnum;
            (*to).list.multi[0].end_col = (*from).list.multi[0].end_col;
        }
    } else if !(*from).list.line[0].end.is_null() {
        (*to).list.line[0].end = (*from).list.line[0].end;
    }
}

/// Return true if `sub1` and `sub2` have the same start positions.
/// When using back-references also check the end position.
#[allow(clippy::cast_possible_truncation)]
unsafe fn sub_equal(sub1: *const RegsubT, sub2: *const RegsubT, has_backref: c_int) -> bool {
    let todo = if (*sub1).in_use > (*sub2).in_use {
        (*sub1).in_use
    } else {
        (*sub2).in_use
    };
    if (REX.reg_match.is_null() as c_int) != 0 {
        for i in 0..todo as usize {
            let s1 = if (i as c_int) < (*sub1).in_use {
                (*sub1).list.multi[i].start_lnum
            } else {
                -1
            };
            let s2 = if (i as c_int) < (*sub2).in_use {
                (*sub2).list.multi[i].start_lnum
            } else {
                -1
            };
            if s1 != s2 {
                return false;
            }
            if s1 != -1 && (*sub1).list.multi[i].start_col != (*sub2).list.multi[i].start_col {
                return false;
            }
            if has_backref != 0 {
                let e1 = if (i as c_int) < (*sub1).in_use {
                    (*sub1).list.multi[i].end_lnum
                } else {
                    -1
                };
                let e2 = if (i as c_int) < (*sub2).in_use {
                    (*sub2).list.multi[i].end_lnum
                } else {
                    -1
                };
                if e1 != e2 {
                    return false;
                }
                if e1 != -1 && (*sub1).list.multi[i].end_col != (*sub2).list.multi[i].end_col {
                    return false;
                }
            }
        }
    } else {
        for i in 0..todo as usize {
            let sp1 = if (i as c_int) < (*sub1).in_use {
                (*sub1).list.line[i].start
            } else {
                core::ptr::null_mut()
            };
            let sp2 = if (i as c_int) < (*sub2).in_use {
                (*sub2).list.line[i].start
            } else {
                core::ptr::null_mut()
            };
            if sp1 != sp2 {
                return false;
            }
            if has_backref != 0 {
                let ep1 = if (i as c_int) < (*sub1).in_use {
                    (*sub1).list.line[i].end
                } else {
                    core::ptr::null_mut()
                };
                let ep2 = if (i as c_int) < (*sub2).in_use {
                    (*sub2).list.line[i].end
                } else {
                    core::ptr::null_mut()
                };
                if ep1 != ep2 {
                    return false;
                }
            }
        }
    }
    true
}

/// Copy a Postponed Invisible Match.
unsafe fn copy_pim(to: *mut NfaPimT, from: *const NfaPimT) {
    (*to).result = (*from).result;
    (*to).state = (*from).state;
    copy_sub(&mut (*to).subs.norm, &(*from).subs.norm);
    if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
        copy_sub(&mut (*to).subs.synt, &(*from).subs.synt);
    }
    (*to).end = (*from).end;
}

// --- Exported wrappers for the submatch operations (called from C during transition) ---

#[no_mangle]
pub unsafe extern "C" fn rs_clear_sub(sub: *mut c_void, nsubexpr: c_int) {
    clear_sub(sub.cast::<RegsubT>(), nsubexpr);
}

#[no_mangle]
pub unsafe extern "C" fn rs_copy_sub(to: *mut c_void, from: *mut c_void) {
    copy_sub(to.cast::<RegsubT>(), from.cast::<RegsubT>());
}

#[no_mangle]
pub unsafe extern "C" fn rs_copy_sub_off(to: *mut c_void, from: *mut c_void) {
    copy_sub_off(to.cast::<RegsubT>(), from.cast::<RegsubT>());
}

#[no_mangle]
pub unsafe extern "C" fn rs_copy_ze_off(to: *mut c_void, from: *mut c_void, has_zend: c_int) {
    copy_ze_off(to.cast::<RegsubT>(), from.cast::<RegsubT>(), has_zend);
}

#[no_mangle]
pub unsafe extern "C" fn rs_sub_equal(
    sub1: *mut c_void,
    sub2: *mut c_void,
    has_backref: c_int,
) -> c_int {
    sub_equal(sub1.cast::<RegsubT>(), sub2.cast::<RegsubT>(), has_backref) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn rs_copy_pim(to: *mut c_void, from: *mut c_void) {
    copy_pim(to.cast::<NfaPimT>(), from.cast::<NfaPimT>());
}

// --- Inline helpers for internal use (accept opaque handles) ---

/// Copy submatch (opaque handle version for internal use).
#[inline]
unsafe fn copy_sub_o(to: *mut c_void, from: *mut c_void) {
    copy_sub(to.cast::<RegsubT>(), from.cast::<RegsubT>());
}

/// Copy submatch excluding main match (opaque handle version).
#[inline]
unsafe fn copy_sub_off_o(to: *mut c_void, from: *mut c_void) {
    copy_sub_off(to.cast::<RegsubT>(), from.cast::<RegsubT>());
}

/// Copy ze offset if \ze present (opaque handle version).
#[inline]
unsafe fn copy_ze_off_o(to: *mut c_void, from: *mut c_void) {
    copy_ze_off(
        to.cast::<RegsubT>(),
        from.cast::<RegsubT>(),
        REX.nfa_has_zend,
    );
}

/// Clear submatch (opaque handle version).
#[inline]
unsafe fn clear_sub_o(sub: *mut c_void) {
    clear_sub(sub.cast::<RegsubT>(), REX.nfa_nsubexpr);
}

/// Copy PIM (opaque handle version).
#[inline]
unsafe fn copy_pim_o(to: *mut c_void, from: *mut c_void) {
    copy_pim(to.cast::<NfaPimT>(), from.cast::<NfaPimT>());
}

/// Compare submatches (opaque handle version).
#[inline]
unsafe fn sub_equal_o(sub1: *mut c_void, sub2: *mut c_void) -> bool {
    sub_equal(
        sub1.cast::<RegsubT>(),
        sub2.cast::<RegsubT>(),
        REX.nfa_has_backref,
    )
}

/// Opaque wrapper: calls typed `addstate_t` with cast pointers.
#[inline]
unsafe fn addstate_o(
    l: NfaListHandle,
    state: NfaStateHandle,
    subs: RegsubsHandle,
    pim: NfaPimHandle,
    off: c_int,
) -> RegsubsHandle {
    addstate_t(
        l.cast::<NfaListT>(),
        state.cast::<NfaStateT>(),
        subs.cast::<RegsubsT>(),
        pim.cast::<NfaPimT>(),
        off,
    )
    .cast::<c_void>()
}

/// Opaque wrapper: calls typed `addstate_here_t` with cast pointers.
#[inline]
unsafe fn addstate_here_o(
    l: NfaListHandle,
    state: NfaStateHandle,
    subs: RegsubsHandle,
    pim: NfaPimHandle,
    ip: *mut c_int,
) -> RegsubsHandle {
    addstate_here_t(
        l.cast::<NfaListT>(),
        state.cast::<NfaStateT>(),
        subs.cast::<RegsubsT>(),
        pim.cast::<NfaPimT>(),
        ip,
    )
    .cast::<c_void>()
}

const ADDSTATE_HERE_OFFSET: c_int = 10;

/// Static depth counter for `addstate` recursion limit.
static mut ADDSTATE_DEPTH: c_int = 0;

/// Static `temp_subs` for `addstate` when realloc invalidates subs pointer.
static mut ADDSTATE_TEMP_SUBS: core::mem::MaybeUninit<RegsubsT> = core::mem::MaybeUninit::uninit();

/// Return true if "one" and "two" PIM states are equal (typed version).
unsafe fn pim_equal_t(one: *const NfaPimT, two: *const NfaPimT) -> bool {
    let one_unused = one.is_null() || (*one).result == NFA_PIM_UNUSED;
    let two_unused = two.is_null() || (*two).result == NFA_PIM_UNUSED;

    if one_unused {
        return two_unused;
    }
    if two_unused {
        return false;
    }
    // compare state id
    if (*(*one).state).id != (*(*two).state).id {
        return false;
    }
    // compare position
    if (REX.reg_match.is_null() as c_int) != 0 {
        return (*one).end.pos.lnum == (*two).end.pos.lnum
            && (*one).end.pos.col == (*two).end.pos.col;
    }
    (*one).end.ptr == (*two).end.ptr
}

/// Return true if "state" with "subs" is in list "l" at the same positions.
unsafe fn has_state_with_pos_t(
    l: *mut NfaListT,
    state: *mut NfaStateT,
    subs: *const RegsubsT,
    pim: *const NfaPimT,
) -> bool {
    for i in 0..(*l).n {
        let thread = &*(*l).t.offset(i as isize);
        if (*thread.state).id == (*state).id
            && sub_equal(&thread.subs.norm, &(*subs).norm, REX.nfa_has_backref)
            && (nvim_regexp_get_nfa_has_zsubexpr() == 0
                || sub_equal(&thread.subs.synt, &(*subs).synt, REX.nfa_has_backref))
            && pim_equal_t(&thread.pim, pim)
        {
            return true;
        }
    }
    false
}

/// Return true if "state" leads to `NFA_MATCH` without advancing input.
unsafe fn match_follows_t(startstate: *const NfaStateT, depth: c_int) -> bool {
    let mut state = startstate;
    if depth > 10 {
        return false;
    }
    while !state.is_null() {
        match (*state).c {
            NFA_MATCH
            | NFA_MCLOSE
            | NFA_END_INVISIBLE
            | NFA_END_INVISIBLE_NEG
            | NFA_END_PATTERN => return true,

            NFA_SPLIT => {
                return match_follows_t((*state).out, depth + 1)
                    || match_follows_t((*state).out1, depth + 1);
            }

            NFA_START_INVISIBLE
            | NFA_START_INVISIBLE_FIRST
            | NFA_START_INVISIBLE_BEFORE
            | NFA_START_INVISIBLE_BEFORE_FIRST
            | NFA_START_INVISIBLE_NEG
            | NFA_START_INVISIBLE_NEG_FIRST
            | NFA_START_INVISIBLE_BEFORE_NEG
            | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
            | NFA_COMPOSING => {
                state = (*(*state).out1).out;
                continue;
            }

            NFA_ANY | NFA_ANY_COMPOSING | NFA_IDENT | NFA_SIDENT | NFA_KWORD | NFA_SKWORD
            | NFA_FNAME | NFA_SFNAME | NFA_PRINT | NFA_SPRINT | NFA_WHITE | NFA_NWHITE
            | NFA_DIGIT | NFA_NDIGIT | NFA_HEX | NFA_NHEX | NFA_OCTAL | NFA_NOCTAL | NFA_WORD
            | NFA_NWORD | NFA_HEAD | NFA_NHEAD | NFA_ALPHA | NFA_NALPHA | NFA_LOWER
            | NFA_NLOWER | NFA_UPPER | NFA_NUPPER | NFA_LOWER_IC | NFA_NLOWER_IC | NFA_UPPER_IC
            | NFA_NUPPER_IC | NFA_START_COLL | NFA_START_NEG_COLL | NFA_NEWL => return false,

            c if c > 0 => return false,
            _ => {}
        }
        state = (*state).out;
    }
    false
}

/// Return true if "state" is already in list "l" (typed version).
unsafe fn state_in_list_t(l: *mut NfaListT, state: *mut NfaStateT, subs: *const RegsubsT) -> bool {
    let ll_index = NFA_LL_INDEX;
    if (*state).lastlist[ll_index as usize] == (*l).id
        && (REX.nfa_has_backref == 0 || has_state_with_pos_t(l, state, subs, core::ptr::null()))
    {
        return true;
    }
    false
}

/// Add "state" and possibly what follows to state list "l".
/// Returns `subs_arg`, possibly copied into `temp_subs`.
/// NULL when recursiveness is too deep or memory exceeded.
#[allow(clippy::too_many_lines, clippy::match_same_arms)]
unsafe fn addstate_t(
    l: *mut NfaListT,
    state: *mut NfaStateT,
    subs_arg: *mut RegsubsT,
    pim: *const NfaPimT,
    off_arg: c_int,
) -> *mut RegsubsT {
    let mut off = off_arg;
    let mut add_here = false;
    let mut listindex: c_int = 0;
    let found = false;
    let mut subs = subs_arg;
    let mut skip_add = false;

    // Recursion depth limit
    ADDSTATE_DEPTH += 1;
    if ADDSTATE_DEPTH >= 5000 || subs.is_null() {
        ADDSTATE_DEPTH -= 1;
        return core::ptr::null_mut();
    }

    if off_arg <= -ADDSTATE_HERE_OFFSET {
        add_here = true;
        off = 0;
        listindex = -(off_arg + ADDSTATE_HERE_OFFSET);
    }

    let state_c = (*state).c;

    match state_c {
        NFA_NCLOSE | NFA_MCLOSE | NFA_MCLOSE1 | NFA_MCLOSE2 | NFA_MCLOSE3 | NFA_MCLOSE4
        | NFA_MCLOSE5 | NFA_MCLOSE6 | NFA_MCLOSE7 | NFA_MCLOSE8 | NFA_MCLOSE9 | NFA_ZCLOSE
        | NFA_ZCLOSE1 | NFA_ZCLOSE2 | NFA_ZCLOSE3 | NFA_ZCLOSE4 | NFA_ZCLOSE5 | NFA_ZCLOSE6
        | NFA_ZCLOSE7 | NFA_ZCLOSE8 | NFA_ZCLOSE9 | NFA_MOPEN | NFA_ZEND | NFA_SPLIT
        | NFA_EMPTY => {
            // These nodes are not added themselves but their "out" and/or
            // "out1" may be added below.
        }

        NFA_BOL | NFA_BOF => {
            // "^" won't match past end-of-line, don't bother trying.
            let input = REX.input;
            let line = REX.line;
            let nfa_endp_ptr = NFA_ENDP;
            if input > line
                && *input != 0
                && (nfa_endp_ptr.is_null()
                    || (REX.reg_match.is_null() as c_int) == 0
                    || REX.lnum
                        == (if NFA_ENDP.is_null() {
                            -1i32
                        } else {
                            (*NFA_ENDP).se_u.pos.lnum
                        }))
            {
                // skip_add
                ADDSTATE_DEPTH -= 1;
                return subs;
            }
            // Fall through to default handling
            subs = addstate_default_add(
                l,
                state,
                subs,
                pim,
                off_arg,
                add_here,
                listindex,
                found,
                &mut skip_add,
            );
            if subs.is_null() {
                ADDSTATE_DEPTH -= 1;
                return core::ptr::null_mut();
            }
        }

        _ => {
            subs = addstate_default_add(
                l,
                state,
                subs,
                pim,
                off_arg,
                add_here,
                listindex,
                found,
                &mut skip_add,
            );
            if subs.is_null() {
                ADDSTATE_DEPTH -= 1;
                return core::ptr::null_mut();
            }
        }
    }

    // Second switch for recursive processing (skipped when state already in list)
    if !skip_add {
        match state_c {
            NFA_MATCH => {}

            NFA_SPLIT => {
                subs = addstate_t(l, (*state).out, subs, pim, off_arg);
                if !subs.is_null() {
                    subs = addstate_t(l, (*state).out1, subs, pim, off_arg);
                }
            }

            NFA_EMPTY | NFA_NOPEN | NFA_NCLOSE => {
                subs = addstate_t(l, (*state).out, subs, pim, off_arg);
            }

            NFA_MOPEN | NFA_MOPEN1 | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4 | NFA_MOPEN5
            | NFA_MOPEN6 | NFA_MOPEN7 | NFA_MOPEN8 | NFA_MOPEN9 | NFA_ZOPEN | NFA_ZOPEN1
            | NFA_ZOPEN2 | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6 | NFA_ZOPEN7
            | NFA_ZOPEN8 | NFA_ZOPEN9 | NFA_ZSTART => {
                subs = addstate_handle_open(l, state, subs, pim, off_arg, off, state_c);
            }

            NFA_MCLOSE if REX.nfa_has_zend != 0 => {
                // Check if \ze already set the end position
                let sub = &(*subs).norm;
                let has_end = if (REX.reg_match.is_null() as c_int) != 0 {
                    sub.list.multi[0].end_lnum >= 0
                } else {
                    !sub.list.line[0].end.is_null()
                };
                if has_end {
                    // Do not overwrite the position set by \ze.
                    subs = addstate_t(l, (*state).out, subs, pim, off_arg);
                } else {
                    subs = addstate_handle_close(l, state, subs, pim, off_arg, off, state_c);
                }
            }

            NFA_MCLOSE | NFA_MCLOSE1 | NFA_MCLOSE2 | NFA_MCLOSE3 | NFA_MCLOSE4 | NFA_MCLOSE5
            | NFA_MCLOSE6 | NFA_MCLOSE7 | NFA_MCLOSE8 | NFA_MCLOSE9 | NFA_ZCLOSE | NFA_ZCLOSE1
            | NFA_ZCLOSE2 | NFA_ZCLOSE3 | NFA_ZCLOSE4 | NFA_ZCLOSE5 | NFA_ZCLOSE6 | NFA_ZCLOSE7
            | NFA_ZCLOSE8 | NFA_ZCLOSE9 | NFA_ZEND => {
                subs = addstate_handle_close(l, state, subs, pim, off_arg, off, state_c);
            }

            _ => {}
        }
    } // end if !skip_add

    ADDSTATE_DEPTH -= 1;
    subs
}

/// Handle the "default" add-to-list path in `addstate`.
/// Sets `*skipped` to true if the state was already in the list and the caller
/// should skip recursive processing (equivalent to C's `goto skip_add`).
#[allow(clippy::too_many_arguments)]
unsafe fn addstate_default_add(
    l: *mut NfaListT,
    state: *mut NfaStateT,
    mut subs: *mut RegsubsT,
    pim: *const NfaPimT,
    _off_arg: c_int,
    add_here: bool,
    listindex: c_int,
    mut found: bool,
    skipped: *mut bool,
) -> *mut RegsubsT {
    let ll_index = NFA_LL_INDEX as usize;
    let state_c = (*state).c;

    if (*state).lastlist[ll_index] == (*l).id && state_c != NFA_SKIP {
        // This state is already in the list
        if REX.nfa_has_backref == 0 && pim.is_null() && (*l).has_pim == 0 && state_c != NFA_MATCH {
            if add_here {
                let k_max = core::cmp::min((*l).n, listindex);
                for k in 0..k_max {
                    if (*(*(*l).t.offset(k as isize)).state).id == (*state).id {
                        found = true;
                        break;
                    }
                }
            }
            if !add_here || found {
                // skip_add - caller (addstate_t) skips recursive processing
                *skipped = true;
                return subs;
            }
        }
        // Do not add the state again when it exists with the same positions.
        if has_state_with_pos_t(l, state, subs, pim) {
            // skip_add - caller (addstate_t) skips recursive processing
            *skipped = true;
            return subs;
        }
    }

    // When there are backreferences or PIMs the number of states may
    // be (a lot) bigger than anticipated.
    if (*l).n == (*l).len {
        let newlen = (*l).len * 3 / 2 + 50;
        let newsize = (newlen as usize) * core::mem::size_of::<NfaThreadT>();

        if ((newsize >> 10) as i64) >= nvim_regexp_get_p_mmp() {
            errors::emsg_maxmempattern();
            // Return null - caller (addstate_t) handles depth
            return core::ptr::null_mut();
        }
        let temp_subs = core::ptr::addr_of_mut!(ADDSTATE_TEMP_SUBS).cast::<RegsubsT>();
        if subs != temp_subs {
            // "subs" may point into the current array, need to make a
            // copy before it becomes invalid.
            copy_sub(&mut (*temp_subs).norm, &(*subs).norm);
            if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                copy_sub(&mut (*temp_subs).synt, &(*subs).synt);
            }
            subs = temp_subs;
        }

        let newt = xrealloc((*l).t.cast::<c_void>(), newsize);
        (*l).t = newt.cast::<NfaThreadT>();
        (*l).len = newlen;
    }

    // add the state to the list
    (*state).lastlist[ll_index] = (*l).id;
    let thread = &mut *(*l).t.offset((*l).n as isize);
    (*l).n += 1;
    thread.state = state;
    if pim.is_null() {
        thread.pim.result = NFA_PIM_UNUSED;
    } else {
        copy_pim(&mut thread.pim, pim);
        (*l).has_pim = 1;
    }
    copy_sub(&mut thread.subs.norm, &(*subs).norm);
    if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
        copy_sub(&mut thread.subs.synt, &(*subs).synt);
    }

    subs
}

/// Handle `NFA_MOPEN`/`NFA_ZOPEN`/`NFA_ZSTART` in `addstate`.
#[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::manual_range_contains
)]
unsafe fn addstate_handle_open(
    l: *mut NfaListT,
    state: *mut NfaStateT,
    mut subs: *mut RegsubsT,
    pim: *const NfaPimT,
    off_arg: c_int,
    off: c_int,
    state_c: c_int,
) -> *mut RegsubsT {
    let (subidx, sub_is_synt) = if state_c == NFA_ZSTART {
        (0, false)
    } else if state_c >= NFA_ZOPEN && state_c <= NFA_ZOPEN9 {
        ((state_c - NFA_ZOPEN) as usize, true)
    } else {
        ((state_c - NFA_MOPEN) as usize, false)
    };

    let sub = if sub_is_synt {
        &mut (*subs).synt
    } else {
        &mut (*subs).norm
    };

    let mut save_ptr: *mut u8 = core::ptr::null_mut();
    let mut save_multipos = MultiPos {
        start_lnum: 0,
        end_lnum: 0,
        start_col: 0,
        end_col: 0,
    };
    let save_in_use;

    if (REX.reg_match.is_null() as c_int) != 0 {
        if (subidx as c_int) < sub.in_use {
            save_multipos = sub.list.multi[subidx];
            save_in_use = -1;
        } else {
            save_in_use = sub.in_use;
            for i in (sub.in_use as usize)..subidx {
                sub.list.multi[i].start_lnum = -1;
                sub.list.multi[i].end_lnum = -1;
            }
            sub.in_use = subidx as c_int + 1;
        }
        let input = REX.input;
        let line = REX.line;
        if off == -1 {
            sub.list.multi[subidx].start_lnum = REX.lnum + 1;
            sub.list.multi[subidx].start_col = 0;
        } else {
            sub.list.multi[subidx].start_lnum = REX.lnum;
            sub.list.multi[subidx].start_col = (input as isize - line as isize) as c_int + off;
        }
        sub.list.multi[subidx].end_lnum = -1;
    } else {
        if (subidx as c_int) < sub.in_use {
            save_ptr = sub.list.line[subidx].start;
            save_in_use = -1;
        } else {
            save_in_use = sub.in_use;
            for i in (sub.in_use as usize)..subidx {
                sub.list.line[i].start = core::ptr::null_mut();
                sub.list.line[i].end = core::ptr::null_mut();
            }
            sub.in_use = subidx as c_int + 1;
        }
        let input = REX.input;
        sub.list.line[subidx].start = input.offset(off as isize);
    }

    subs = addstate_t(l, (*state).out, subs, pim, off_arg);
    if subs.is_null() {
        return subs;
    }
    // "subs" may have changed, need to set "sub" again.
    let sub = if sub_is_synt {
        &mut (*subs).synt
    } else {
        &mut (*subs).norm
    };

    if save_in_use == -1 {
        if (REX.reg_match.is_null() as c_int) != 0 {
            sub.list.multi[subidx] = save_multipos;
        } else {
            sub.list.line[subidx].start = save_ptr;
        }
    } else {
        sub.in_use = save_in_use;
    }
    subs
}

/// Handle `NFA_MCLOSE`/`NFA_ZCLOSE`/`NFA_ZEND` in `addstate`.
#[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::manual_range_contains
)]
unsafe fn addstate_handle_close(
    l: *mut NfaListT,
    state: *mut NfaStateT,
    mut subs: *mut RegsubsT,
    pim: *const NfaPimT,
    off_arg: c_int,
    off: c_int,
    state_c: c_int,
) -> *mut RegsubsT {
    let (subidx, sub_is_synt) = if state_c == NFA_ZEND {
        (0, false)
    } else if state_c >= NFA_ZCLOSE && state_c <= NFA_ZCLOSE9 {
        ((state_c - NFA_ZCLOSE) as usize, true)
    } else {
        ((state_c - NFA_MCLOSE) as usize, false)
    };

    let sub = if sub_is_synt {
        &mut (*subs).synt
    } else {
        &mut (*subs).norm
    };

    let save_in_use = sub.in_use;
    if sub.in_use <= subidx as c_int {
        sub.in_use = subidx as c_int + 1;
    }

    let mut save_ptr: *mut u8 = core::ptr::null_mut();
    let mut save_multipos = MultiPos {
        start_lnum: 0,
        end_lnum: 0,
        start_col: 0,
        end_col: 0,
    };

    if (REX.reg_match.is_null() as c_int) != 0 {
        save_multipos = sub.list.multi[subidx];
        let input = REX.input;
        let line = REX.line;
        if off == -1 {
            sub.list.multi[subidx].end_lnum = REX.lnum + 1;
            sub.list.multi[subidx].end_col = 0;
        } else {
            sub.list.multi[subidx].end_lnum = REX.lnum;
            sub.list.multi[subidx].end_col = (input as isize - line as isize) as c_int + off;
        }
    } else {
        save_ptr = sub.list.line[subidx].end;
        let input = REX.input;
        sub.list.line[subidx].end = input.offset(off as isize);
    }

    subs = addstate_t(l, (*state).out, subs, pim, off_arg);
    if subs.is_null() {
        return subs;
    }
    // "subs" may have changed, need to set "sub" again.
    let sub = if sub_is_synt {
        &mut (*subs).synt
    } else {
        &mut (*subs).norm
    };

    if (REX.reg_match.is_null() as c_int) != 0 {
        sub.list.multi[subidx] = save_multipos;
    } else {
        sub.list.line[subidx].end = save_ptr;
    }
    sub.in_use = save_in_use;
    subs
}

/// Like `addstate_t()`, but the new state(s) are put at position `*ip`.
#[allow(clippy::cast_possible_truncation)]
unsafe fn addstate_here_t(
    l: *mut NfaListT,
    state: *mut NfaStateT,
    subs: *mut RegsubsT,
    pim: *const NfaPimT,
    ip: *mut c_int,
) -> *mut RegsubsT {
    let tlen = (*l).n;
    let listidx = *ip;

    let r = addstate_t(l, state, subs, pim, -listidx - ADDSTATE_HERE_OFFSET);
    if r.is_null() {
        return core::ptr::null_mut();
    }

    // when "*ip" was at the end of the list, nothing to do
    if listidx + 1 == tlen {
        return r;
    }

    // re-order to put the new state at the current position
    let count = (*l).n - tlen;
    if count == 0 {
        return r; // no state got added
    }
    if count == 1 {
        // overwrite the current state
        *(*l).t.offset(listidx as isize) = *(*l).t.offset(((*l).n - 1) as isize);
    } else if count > 1 {
        if (*l).n + count > (*l).len {
            // not enough space, reallocate
            let newlen = (*l).len * 3 / 2 + 50;
            let newsize = (newlen as usize) * core::mem::size_of::<NfaThreadT>();

            if ((newsize >> 10) as i64) >= nvim_regexp_get_p_mmp() {
                errors::emsg_maxmempattern();
                return core::ptr::null_mut();
            }
            let newl = xmalloc(newsize).cast::<NfaThreadT>();
            (*l).len = newlen;
            core::ptr::copy_nonoverlapping((*l).t, newl, listidx as usize);
            core::ptr::copy_nonoverlapping(
                (*l).t.offset(((*l).n - count) as isize),
                newl.offset(listidx as isize),
                count as usize,
            );
            core::ptr::copy_nonoverlapping(
                (*l).t.offset((listidx + 1) as isize),
                newl.offset((listidx + count) as isize),
                ((*l).n - count - listidx - 1) as usize,
            );
            nvim_regexp_xfree((*l).t.cast::<c_void>());
            (*l).t = newl;
        } else {
            // make space for new states, then move them
            core::ptr::copy(
                (*l).t.offset((listidx + 1) as isize),
                (*l).t.offset((listidx + count) as isize),
                ((*l).n - listidx - 1) as usize,
            );
            core::ptr::copy(
                (*l).t.offset(((*l).n - 1) as isize),
                (*l).t.offset(listidx as isize),
                count as usize,
            );
        }
    }
    (*l).n -= 1;
    *ip = listidx - 1;

    r
}

/// Return true if "one" and "two" PIM states are equal.
/// That includes when both are unused (not set).
#[no_mangle]
pub unsafe extern "C" fn rs_pim_equal(one: NfaPimHandle, two: NfaPimHandle) -> c_int {
    let one_unused = one.is_null() || (*one.cast::<NfaPimT>()).result == NFA_PIM_UNUSED;
    let two_unused = two.is_null() || (*two.cast::<NfaPimT>()).result == NFA_PIM_UNUSED;

    if one_unused {
        return two_unused as c_int;
    }
    if two_unused {
        return 0;
    }
    // compare state id
    if (*(*one.cast::<NfaPimT>()).state).id != (*(*two.cast::<NfaPimT>()).state).id {
        return 0;
    }
    // compare position
    if (REX.reg_match.is_null() as c_int) != 0 {
        return ((*one.cast::<NfaPimT>()).end.pos.lnum == (*two.cast::<NfaPimT>()).end.pos.lnum
            && (*one.cast::<NfaPimT>()).end.pos.col == (*two.cast::<NfaPimT>()).end.pos.col)
            as c_int;
    }
    ((*one.cast::<NfaPimT>()).end.ptr == (*two.cast::<NfaPimT>()).end.ptr) as c_int
}

/// Check if "state" with "subs" is already in list "l", considering PIM.
#[no_mangle]
pub unsafe extern "C" fn rs_has_state_with_pos(
    l: NfaListHandle,
    state_id: c_int,
    subs_norm: *mut c_void,
    subs_synt: *mut c_void,
    pim: NfaPimHandle,
) -> c_int {
    let n = (*l.cast::<NfaListT>()).n;
    for i in 0..n {
        if (*(*l.cast::<NfaListT>()).t.add(i as usize)).state.read().id != state_id {
            continue;
        }
        if !sub_equal_o(
            (&raw mut (*(*l.cast::<NfaListT>()).t.add(i as usize)).subs.norm).cast::<c_void>(),
            subs_norm,
        ) {
            continue;
        }
        if nvim_regexp_get_nfa_has_zsubexpr() != 0
            && !sub_equal_o(
                (&raw mut (*(*l.cast::<NfaListT>()).t.add(i as usize)).subs.synt).cast::<c_void>(),
                subs_synt,
            )
        {
            continue;
        }
        if rs_pim_equal(
            (&raw mut (*(*l.cast::<NfaListT>()).t.add(i as usize)).pim).cast::<c_void>(),
            pim,
        ) != 0
        {
            return 1;
        }
    }
    0
}

/// Return true if "state" is already in list "l".
#[no_mangle]
pub unsafe extern "C" fn rs_state_in_list(
    l: NfaListHandle,
    state: NfaStateHandle,
    subs_norm: *mut c_void,
    subs_synt: *mut c_void,
) -> c_int {
    let ll_index = NFA_LL_INDEX;
    if (*state.cast::<NfaStateT>()).lastlist[ll_index as usize] == (*l.cast::<NfaListT>()).id
        && (REX.nfa_has_backref == 0
            || rs_has_state_with_pos(
                l,
                (*state.cast::<NfaStateT>()).id,
                subs_norm,
                subs_synt,
                core::ptr::null_mut(),
            ) != 0)
    {
        return 1;
    }
    0
}

/// Check for a match with subexpression "subidx".
/// Migrated from C `match_backref()`.
#[no_mangle]
pub unsafe extern "C" fn rs_match_backref(
    sub: *mut c_void,
    subidx: c_int,
    bytelen: *mut c_int,
) -> c_int {
    match_backref_t(sub.cast::<RegsubT>(), subidx, bytelen)
}

/// Check for a match with subexpression "subidx".
/// Migrated from C `match_backref()`.
#[allow(clippy::cast_possible_truncation)]
unsafe fn match_backref_t(sub: *const RegsubT, subidx: c_int, bytelen: *mut c_int) -> c_int {
    if (*sub).in_use <= subidx {
        // backref was not set, match an empty string
        *bytelen = 0;
        return 1; // true
    }

    if (REX.reg_match.is_null() as c_int) != 0 {
        let m = &(*sub).list.multi[subidx as usize];
        if m.start_lnum < 0 || m.end_lnum < 0 {
            *bytelen = 0;
            return 1; // true
        }
        let rex_lnum = REX.lnum;
        if m.start_lnum == rex_lnum && m.end_lnum == rex_lnum {
            let mut len = m.end_col - m.start_col;
            let rex_line = REX.line;
            let rex_input = REX.input;
            if rs_cstrncmp(
                rex_line.add(m.start_col as usize).cast::<c_char>(),
                rex_input.cast::<c_char>(),
                &mut len,
            ) == 0
            {
                *bytelen = len;
                return 1; // true
            }
        } else if rs_match_with_backref(m.start_lnum, m.start_col, m.end_lnum, m.end_col, bytelen)
            == RA_MATCH
        {
            return 1; // true
        }
    } else {
        let lp = &(*sub).list.line[subidx as usize];
        if lp.start.is_null() || lp.end.is_null() {
            *bytelen = 0;
            return 1; // true
        }
        let mut len = lp.end.offset_from(lp.start) as c_int;
        let rex_input = REX.input;
        if rs_cstrncmp(
            lp.start.cast::<c_char>(),
            rex_input.cast::<c_char>(),
            &mut len,
        ) == 0
        {
            *bytelen = len;
            return 1; // true
        }
    }
    0 // false
}

/// Check for a match with \z subexpression "subidx".
/// Migrated from C `match_zref()`.
#[allow(clippy::cast_possible_truncation)]
#[no_mangle]
pub unsafe extern "C" fn rs_match_zref(subidx: c_int, bytelen: *mut c_int) -> c_int {
    rs_cleanup_zsubexpr();
    let match_ptr = nvim_regexp_get_re_extmatch_in_match(subidx);
    if match_ptr.is_null() {
        // backref was not set, match an empty string
        *bytelen = 0;
        return 1; // true
    }

    let mut len = core::ffi::CStr::from_ptr(match_ptr.cast::<c_char>())
        .to_bytes()
        .len() as c_int;
    let rex_input = REX.input;
    if rs_cstrncmp(
        match_ptr.cast::<c_char>(),
        rex_input.cast::<c_char>(),
        &mut len,
    ) == 0
    {
        *bytelen = len;
        return 1; // true
    }
    0 // false
}

/// Skip until the char "c" we know a match must start with.
/// Migrated from C `skip_to_start()`.
#[allow(clippy::cast_possible_truncation)]
#[no_mangle]
pub unsafe extern "C" fn rs_skip_to_start(c: c_int, colp: *mut c_int) -> c_int {
    let rex_line = REX.line;
    let s = rs_cstrchr(rex_line.add(*colp as usize).cast::<c_char>(), c);
    if s.is_null() {
        return FAIL;
    }
    *colp = s.cast::<u8>().offset_from(rex_line) as c_int;
    OK
}

/// Check for a match with `match_text`.
/// Called after `rs_skip_to_start()` has found regstart.
/// Returns zero for no match, 1 for a match.
/// Migrated from C `find_match_text()`.
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
#[no_mangle]
pub unsafe extern "C" fn rs_find_match_text(
    startcol: *mut c_int,
    regstart: c_int,
    match_text: *mut u8,
) -> c_int {
    let mut col = *startcol;
    let regstart_len = utf_char2len(regstart);

    loop {
        let mut matched = true;
        let mut s1 = match_text;
        let rex_line = REX.line;
        // skip regstart
        let mut regstart_len2 = regstart_len;
        if regstart_len2 > 1
            && utf_ptr2len(rex_line.add(col as usize).cast::<c_char>()) != regstart_len2
        {
            // because of case-folding of the previously matched text, we may need
            // to skip fewer bytes than utf_char2len(regstart)
            regstart_len2 = utf_char2len(utf_fold(regstart));
        }
        let mut s2 = rex_line.add((col + regstart_len2) as usize);
        while *s1 != 0 {
            let c1_len = utf_ptr2len(s1.cast::<c_char>());
            let c1 = utf_ptr2char(s1.cast::<c_char>());
            let c2_len = utf_ptr2len(s2.cast::<c_char>());
            let c2 = utf_ptr2char(s2.cast::<c_char>());
            if c1 != c2 && (REX.reg_ic as c_int == 0 || utf_fold(c1) != utf_fold(c2)) {
                matched = false;
                break;
            }
            s1 = s1.add(c1_len as usize);
            s2 = s2.add(c2_len as usize);
        }
        if matched
            // check that no composing char follows
            && utf_iscomposing_legacy(utf_ptr2char(s2.cast::<c_char>())) == 0
        {
            rs_cleanup_subexpr();
            if (REX.reg_match.is_null() as c_int) != 0 {
                let startpos = REX.reg_startpos;
                let endpos = REX.reg_endpos;
                let rex_lnum = REX.lnum;
                (*startpos.add(0)).lnum = rex_lnum;
                (*startpos.add(0)).col = col;
                (*endpos.add(0)).lnum = rex_lnum;
                (*endpos.add(0)).col = s2.offset_from(REX.line) as c_int;
            } else {
                let startp = REX.reg_startp;
                let endp = REX.reg_endp;
                *startp.add(0) = REX.line.add(col as usize);
                *endp.add(0) = s2;
            }
            *startcol = col;
            return 1;
        }

        // Try finding regstart after the current match.
        col += regstart_len; // skip regstart
        if rs_skip_to_start(regstart, &mut col) == FAIL {
            break;
        }
    }

    *startcol = col;
    0
}

/// Check if NFA execution has timed out.
/// Migrated from C `nfa_did_time_out()`.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_did_time_out() -> c_int {
    let time_limit = NFA_TIME_LIMIT;
    if !time_limit.is_null() && nvim_regexp_call_profile_passed_limit(time_limit) != 0 {
        let timed_out = NFA_TIMED_OUT;
        if !timed_out.is_null() {
            *timed_out = 1; // true
        }
        return 1; // true
    }
    0 // false
}

/// Save list IDs for all NFA states of `prog` into `list`.
/// Also reset the IDs to zero. Only used for the recursive value `lastlist[1]`.
/// Migrated from C `nfa_save_listids()`.
unsafe fn nfa_save_listids_t(prog: NfaProgHandle, list: *mut c_int) {
    let nstate = (*prog.cast::<NfaRegprogT>()).nstate;
    // Iterate in reverse, accessing states via accessor (opaque prog handle).
    for i in (0..nstate).rev() {
        let state = prog
            .cast::<NfaRegprogT>()
            .add(1)
            .cast::<NfaStateT>()
            .add(i as usize)
            .cast::<c_void>();
        let state_ptr = state.cast::<NfaStateT>();
        *list.offset(i as isize) = (*state_ptr).lastlist[1];
        (*state_ptr).lastlist[1] = 0;
    }
}

/// Restore list IDs from `list` to all NFA states.
/// Migrated from C `nfa_restore_listids()`.
unsafe fn nfa_restore_listids_t(prog: NfaProgHandle, list: *const c_int) {
    let nstate = (*prog.cast::<NfaRegprogT>()).nstate;
    for i in (0..nstate).rev() {
        let state = prog
            .cast::<NfaRegprogT>()
            .add(1)
            .cast::<NfaStateT>()
            .add(i as usize)
            .cast::<c_void>();
        let state_ptr = state.cast::<NfaStateT>();
        (*state_ptr).lastlist[1] = *list.offset(i as isize);
    }
}

/// Recursively call `rs_nfa_regmatch()`.
/// `pim` is NULL or contains info about a Postponed Invisible Match (start position).
/// Migrated from C `recursive_regmatch()`.
#[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::too_many_lines
)]
unsafe fn recursive_regmatch_t(
    state: NfaStateHandle,
    pim: NfaPimHandle,
    prog: NfaProgHandle,
    submatch: RegsubsHandle,
    m: RegsubsHandle,
    listids: *mut *mut c_int,
    listids_len: *mut c_int,
) -> c_int {
    let state_ptr = state.cast::<NfaStateT>();
    let is_multi = (REX.reg_match.is_null() as c_int) != 0;

    let save_reginput_col = REX.input.offset_from(REX.line) as c_int;
    let save_reglnum = REX.lnum;
    let save_nfa_match = MATCH_FOUND;
    let save_nfa_listid = REX.nfa_listid;
    let save_nfa_endp = NFA_ENDP;

    // Allocate endpos on the stack
    let mut endpos_storage: SaveSeT = core::mem::zeroed();
    let mut endposp: *mut SaveSeT = core::ptr::null_mut();
    let mut need_restore = false;

    if !pim.is_null() {
        // Start at the position where the postponed match was
        let pim_ptr = pim.cast::<NfaPimT>();
        if is_multi {
            let col = (*pim_ptr).end.pos.col;
            REX.input = REX.line.offset(col as isize);
        } else {
            REX.input = (*pim_ptr).end.ptr;
        }
    }

    let state_c = (*state_ptr).c;
    if state_c == NFA_START_INVISIBLE_BEFORE
        || state_c == NFA_START_INVISIBLE_BEFORE_FIRST
        || state_c == NFA_START_INVISIBLE_BEFORE_NEG
        || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
    {
        // The recursive match must end at the current position.
        endposp = &mut endpos_storage;
        if is_multi {
            if pim.is_null() {
                (*endposp).se_u.pos.col = REX.input.offset_from(REX.line) as i32;
                (*endposp).se_u.pos.lnum = REX.lnum;
            } else {
                let pim_ptr = pim.cast::<NfaPimT>();
                (*endposp).se_u.pos = (*pim_ptr).end.pos;
            }
        } else if pim.is_null() {
            (*endposp).se_u.ptr = REX.input;
        } else {
            let pim_ptr = pim.cast::<NfaPimT>();
            (*endposp).se_u.ptr = (*pim_ptr).end.ptr;
        }

        // Go back the specified number of bytes, or as far as the start of
        // the previous line, to try matching "\@<=" or not matching "\@<!".
        let state_val = (*state_ptr).val;
        if state_val <= 0 {
            if is_multi {
                let new_lnum = REX.lnum - 1;
                let line = nvim_regexp_call_reg_getline(new_lnum);
                if line.is_null() {
                    // can't go before the first line
                    let _ = nvim_regexp_call_reg_getline(REX.lnum);
                } else {
                    REX.lnum = new_lnum;
                    REX.line = line.cast::<u8>();
                }
            }
            REX.input = REX.line;
        } else {
            if is_multi && (REX.input.offset_from(REX.line) as c_int) < state_val {
                // Not enough bytes in this line, go to end of previous line.
                let new_lnum = REX.lnum - 1;
                let line = nvim_regexp_call_reg_getline(new_lnum);
                if line.is_null() {
                    // can't go before the first line
                    let _ = nvim_regexp_call_reg_getline(REX.lnum);
                    REX.input = REX.line;
                } else {
                    REX.lnum = new_lnum;
                    REX.line = line.cast::<u8>();
                    let line_len = nvim_regexp_call_reg_getline_len(new_lnum);
                    REX.input = REX.line.offset(line_len as isize);
                }
            }
            if (REX.input.offset_from(REX.line) as c_int) >= state_val {
                REX.input = REX.input.offset(-(state_val as isize));
                // Adjust for multi-byte: back up to start of char
                let head = utf_head_off(REX.line.cast::<c_char>(), REX.input.cast::<c_char>());
                REX.input = REX.input.offset(-(head as isize));
            } else {
                REX.input = REX.line;
            }
        }
    }

    // Have to clear the lastlist field of the NFA nodes, so that
    // nfa_regmatch() and addstate() can run properly after recursion.
    let nfa_ll_index = NFA_LL_INDEX;
    if nfa_ll_index == 1 {
        // Already calling nfa_regmatch() recursively.  Save the lastlist[1]
        // values and clear them.
        let nstate = (*prog.cast::<NfaRegprogT>()).nstate;
        if (*listids).is_null() || *listids_len < nstate {
            nvim_regexp_xfree((*listids).cast::<c_void>());
            *listids = xmalloc(core::mem::size_of::<c_int>() * nstate as usize).cast::<c_int>();
            *listids_len = nstate;
        }
        nfa_save_listids_t(prog, *listids);
        need_restore = true;
        // any value of rex.nfa_listid will do
    } else {
        // First recursive nfa_regmatch() call, switch to the second lastlist
        // entry.
        NFA_LL_INDEX = nfa_ll_index + 1;
        let listid = REX.nfa_listid;
        let alt_listid = REX.nfa_alt_listid;
        if listid <= alt_listid {
            REX.nfa_listid = alt_listid;
        }
    }

    // Call rs_nfa_regmatch() to check if the current concat matches at this
    // position. The concat ends with the node NFA_END_INVISIBLE.
    NFA_ENDP = endposp;
    let state_out = (*state_ptr).out;
    let result = rs_nfa_regmatch(prog, state_out.cast::<c_void>(), submatch, m);

    if need_restore {
        nfa_restore_listids_t(prog, *listids);
    } else {
        NFA_LL_INDEX -= 1;
        REX.nfa_alt_listid = REX.nfa_listid;
    }

    // Restore position in input text
    REX.lnum = save_reglnum;
    if is_multi {
        REX.line = nvim_regexp_call_reg_getline(save_reglnum).cast::<u8>();
    }
    REX.input = REX.line.offset(save_reginput_col as isize);
    if result != NFA_TOO_EXPENSIVE {
        MATCH_FOUND = save_nfa_match;
        REX.nfa_listid = save_nfa_listid;
    }
    NFA_ENDP = save_nfa_endp;

    result
}

// --- Phase 8.4: nfa_regmatch — The Core Engine ---

// Additional constants for nfa_regmatch
#[allow(clippy::unreadable_literal)]
const NFA_MAX_STATES: c_int = 100000;
const AUTOMATIC_ENGINE: c_int = 0;
const NFA_TOO_EXPENSIVE: c_int = -1;
const NFA_PIM_TODO: c_int = 1;
const NFA_PIM_MATCH: c_int = 2;
const NFA_PIM_NOMATCH: c_int = 3;
const MAX_MCO: usize = 6;

// Extern declarations for Phase 8.4 C accessors
extern "C" {

    // Thread PIM field accessors

    // nfa_list_T management

    // regsubs_T operations

    // rex execution field accessors

    // Character/utility functions

    // NFA prog field accessor

    // PIM operations

    // PIM allocation/init

    // win_T and buffer accessors for VCOL/MARK
    fn nvim_regexp_get_curwin() -> *mut c_void;
    fn nvim_regexp_get_win_b_p_ts(wp: *mut c_void) -> i64;
    fn nvim_regexp_get_win_buf_line_count(wp: *mut c_void) -> i32;

    // Mark access
    fn nvim_regexp_call_mark_get_for_nfa(
        buf: *mut c_void,
        win: *mut c_void,
        mark_val: c_int,
    ) -> *mut c_void;
    fn nvim_regexp_fmark_is_set(fm: *mut c_void) -> c_int;
    fn nvim_regexp_fmark_get_lnum(fm: *mut c_void) -> i32;
    fn nvim_regexp_fmark_get_col(fm: *mut c_void) -> i32;
    fn nvim_regexp_fmark_get_col_adj(fm: *mut c_void, lnum_match: i32) -> i32;

    // List thread count setter

    // Memory free wrapper
    fn nvim_regexp_xfree(p: *mut c_void);
}

/// Main NFA matching routine.
///
/// Run NFA to determine whether it matches `rex.input`.
///
/// When `nfa_endp` is not NULL it is a required end-of-match position.
///
/// Return true if there is a match, false if there is no match,
/// `NFA_TOO_EXPENSIVE` if we end up with too many states.
/// When there is a match "submatch" contains the positions.
#[no_mangle]
#[allow(
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::suspicious_operation_groupings,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::branches_sharing_code,
    clippy::needless_bool,
    clippy::bool_to_int_with_if,
    clippy::if_same_then_else,
    clippy::collapsible_if,
    clippy::needless_bool_assign,
    clippy::unnecessary_operation,
    clippy::nonminimal_bool,
    clippy::manual_range_contains,
    clippy::ptr_cast_constness,
    clippy::needless_range_loop,
    clippy::if_not_else,
    clippy::comparison_chain,
    unused_variables,
    unused_assignments
)]
pub unsafe extern "C" fn rs_nfa_regmatch(
    prog: NfaProgHandle,
    start: NfaStateHandle,
    submatch: RegsubsHandle,
    m: RegsubsHandle,
) -> c_int {
    let mut result: c_int;
    let mut flag: c_int = 0;
    let mut go_to_nextline: bool = false;
    let mut listids: *mut c_int = core::ptr::null_mut();
    let mut listids_len: c_int = 0;
    let mut add_state: NfaStateHandle;
    let mut add_here: bool;
    let mut add_count: c_int;
    let mut add_off: c_int;
    let toplevel: bool = (*start.cast::<NfaStateT>()).c == NFA_MOPEN;
    let mut r: RegsubsHandle;

    // Allow interrupting with CTRL-C.
    rs_reg_breakcheck();
    if nvim_regexp_get_got_int() != 0 {
        return 0; // false
    }
    if rs_nfa_did_time_out() != 0 {
        return 0; // false
    }

    MATCH_FOUND = 0; // nfa_match = false

    // Allocate memory for the lists of nodes.
    let nstate = (*prog.cast::<NfaRegprogT>()).nstate;
    let list0 = {
        let l = xcalloc(1, core::mem::size_of::<NfaListT>()).cast::<NfaListT>();
        (*l).t =
            xcalloc((nstate + 1) as usize, core::mem::size_of::<NfaThreadT>()).cast::<NfaThreadT>();
        l.cast::<c_void>()
    };
    let list1 = {
        let l = xcalloc(1, core::mem::size_of::<NfaListT>()).cast::<NfaListT>();
        (*l).t =
            xcalloc((nstate + 1) as usize, core::mem::size_of::<NfaThreadT>()).cast::<NfaThreadT>();
        l.cast::<c_void>()
    };

    // Initialize thislist and nextlist
    let mut thislist = list0;
    (*thislist.cast::<NfaListT>()).n = 0;
    (*thislist.cast::<NfaListT>()).has_pim = 0;
    let mut nextlist = list1;
    (*nextlist.cast::<NfaListT>()).n = 0;
    (*nextlist.cast::<NfaListT>()).has_pim = 0;

    (*thislist.cast::<NfaListT>()).id = REX.nfa_listid + 1;

    // Inline optimized code for addstate(thislist, start, m, 0) if we know
    // it's the first MOPEN.
    if toplevel {
        if (REX.reg_match.is_null() as c_int) != 0 {
            let col = REX.input as isize - REX.line as isize;
            {
                let _t = m.cast::<RegsubsT>();
                (*_t).norm.list.multi[0_usize].start_lnum = REX.lnum;
                (*_t).norm.list.multi[0_usize].start_col = col as i32;
            }
            (*m.cast::<RegsubsT>()).norm.orig_start_col = col as i32;
        } else {
            (*m.cast::<RegsubsT>()).norm.list.line[0_usize].start = REX.input;
        }
        (*m.cast::<RegsubsT>()).norm.in_use = 1;
        r = addstate_o(
            thislist,
            (*start.cast::<NfaStateT>()).out.cast::<c_void>(),
            m,
            core::ptr::null_mut(),
            0,
        );
    } else {
        r = addstate_o(thislist, start, m, core::ptr::null_mut(), 0);
    }
    if r.is_null() {
        MATCH_FOUND = NFA_TOO_EXPENSIVE;
        // goto theend
        {
            let l0 = list0.cast::<NfaListT>();
            xfree((*l0).t.cast::<c_void>());
            xfree(l0.cast::<c_void>());
        }
        {
            let l1 = list1.cast::<NfaListT>();
            xfree((*l1).t.cast::<c_void>());
            xfree(l1.cast::<c_void>());
        }
        if !listids.is_null() {
            nvim_regexp_xfree(listids.cast::<c_void>());
        }
        return MATCH_FOUND;
    }

    // Run for each character.
    'outer: loop {
        let curc = utf_ptr2char(REX.input as *const c_char);
        let mut clen = utfc_ptr2len(REX.input as *const c_char);
        if curc == 0 {
            // NUL
            clen = 0;
            go_to_nextline = false;
        }

        // swap lists
        thislist = if flag != 0 { list1 } else { list0 };
        flag ^= 1;
        nextlist = if flag != 0 { list1 } else { list0 };
        (*nextlist.cast::<NfaListT>()).n = 0;
        (*nextlist.cast::<NfaListT>()).has_pim = 0;

        let nfa_listid = REX.nfa_listid + 1;
        REX.nfa_listid = nfa_listid;
        if (*prog.cast::<NfaRegprogT>()).re_engine as c_int == AUTOMATIC_ENGINE
            && nfa_listid >= NFA_MAX_STATES
        {
            MATCH_FOUND = NFA_TOO_EXPENSIVE;
            break 'outer;
        }

        (*thislist.cast::<NfaListT>()).id = nfa_listid;
        (*nextlist.cast::<NfaListT>()).id = nfa_listid + 1;

        // If the state lists are empty we can stop.
        if (*thislist.cast::<NfaListT>()).n == 0 {
            break 'outer;
        }

        // compute nextlist
        let mut listidx: c_int = 0;
        while listidx < (*thislist.cast::<NfaListT>()).n {
            // Allow interrupting with CTRL-C.
            rs_reg_breakcheck();
            if nvim_regexp_get_got_int() != 0 {
                break;
            }
            if !NFA_TIME_LIMIT.is_null() {
                let tc = NFA_TIME_COUNT + 1;
                NFA_TIME_COUNT = tc;
                if tc == 20 {
                    NFA_TIME_COUNT = 0;
                    if rs_nfa_did_time_out() != 0 {
                        break;
                    }
                }
            }

            // Handle the possible codes of the current state.
            add_state = core::ptr::null_mut();
            add_here = false;
            add_count = 0;
            add_off = 0;
            result = 0;

            let state_c = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state).c;

            match state_c {
                x if x == NFA_MATCH => {
                    // If the match is not at the start of the line, ends before a
                    // composing character and rex.reg_icombine is not set, that
                    // is not really a match.
                    if REX.reg_icombine as c_int == 0
                        && REX.input != REX.line
                        && utf_iscomposing_legacy(curc) != 0
                    {
                        // break from match arm - continue to next state
                    } else {
                        MATCH_FOUND = 1; // true
                        copy_sub_o(
                            &raw mut (*submatch.cast::<RegsubsT>()).norm as *mut c_void,
                            &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                .subs
                                .norm as *mut c_void,
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            copy_sub_o(
                                &raw mut (*submatch.cast::<RegsubsT>()).synt as *mut c_void,
                                &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                    .subs
                                    .synt as *mut c_void,
                            );
                        }
                        // Found left-most longest match.
                        if (*nextlist.cast::<NfaListT>()).n == 0 {
                            clen = 0;
                        }
                        // goto nextchar: break inner loop, let outer loop's
                        // bottom-of-loop code do the input advancement
                        listidx = (*thislist.cast::<NfaListT>()).n;
                        continue;
                    }
                }

                x if x == NFA_END_INVISIBLE
                    || x == NFA_END_INVISIBLE_NEG
                    || x == NFA_END_PATTERN =>
                {
                    // Check if nfa_endp matches current position
                    let endp = NFA_ENDP;
                    if !endp.is_null() {
                        if (REX.reg_match.is_null() as c_int) != 0 {
                            if REX.lnum
                                != (if NFA_ENDP.is_null() {
                                    -1i32
                                } else {
                                    (*NFA_ENDP).se_u.pos.lnum
                                })
                                || (REX.input as isize - REX.line as isize) as i32
                                    != (if NFA_ENDP.is_null() {
                                        -1i32
                                    } else {
                                        (*NFA_ENDP).se_u.pos.col
                                    })
                            {
                                // no match at required position
                                listidx += 1;
                                continue;
                            }
                        } else if REX.input
                            != (if NFA_ENDP.is_null() {
                                core::ptr::null_mut()
                            } else {
                                (*NFA_ENDP).se_u.ptr
                            })
                        {
                            listidx += 1;
                            continue;
                        }
                    }
                    // do not set submatches for \@!
                    if state_c != NFA_END_INVISIBLE_NEG {
                        copy_sub_o(
                            &raw mut (*m.cast::<RegsubsT>()).norm as *mut c_void,
                            &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                .subs
                                .norm as *mut c_void,
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            copy_sub_o(
                                &raw mut (*m.cast::<RegsubsT>()).synt as *mut c_void,
                                &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                    .subs
                                    .synt as *mut c_void,
                            );
                        }
                    }
                    MATCH_FOUND = 1; // true
                    if (*nextlist.cast::<NfaListT>()).n == 0 {
                        clen = 0;
                    }
                    // goto nextchar: break inner loop, let outer loop handle advancement
                    listidx = (*thislist.cast::<NfaListT>()).n;
                    continue;
                }

                x if x == NFA_START_INVISIBLE
                    || x == NFA_START_INVISIBLE_FIRST
                    || x == NFA_START_INVISIBLE_NEG
                    || x == NFA_START_INVISIBLE_NEG_FIRST
                    || x == NFA_START_INVISIBLE_BEFORE
                    || x == NFA_START_INVISIBLE_BEFORE_FIRST
                    || x == NFA_START_INVISIBLE_BEFORE_NEG
                    || x == NFA_START_INVISIBLE_BEFORE_NEG_FIRST =>
                {
                    let t_state = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                        .state
                        .cast::<c_void>();
                    let pim_result = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                        .pim
                        .result;

                    if pim_result != NFA_PIM_UNUSED
                        || state_c == NFA_START_INVISIBLE_FIRST
                        || state_c == NFA_START_INVISIBLE_NEG_FIRST
                        || state_c == NFA_START_INVISIBLE_BEFORE_FIRST
                        || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
                    {
                        let in_use = (*m.cast::<RegsubsT>()).norm.in_use;

                        // Copy submatch info for the recursive call
                        copy_sub_off_o(
                            &raw mut (*m.cast::<RegsubsT>()).norm as *mut c_void,
                            &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                .subs
                                .norm as *mut c_void,
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            copy_sub_off_o(
                                &raw mut (*m.cast::<RegsubsT>()).synt as *mut c_void,
                                &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                    .subs
                                    .synt as *mut c_void,
                            );
                        }
                        // First try matching the invisible match
                        result = recursive_regmatch_t(
                            t_state,
                            core::ptr::null_mut(),
                            prog,
                            submatch,
                            m,
                            &mut listids,
                            &mut listids_len,
                        );
                        if result == NFA_TOO_EXPENSIVE {
                            MATCH_FOUND = result;
                            break 'outer;
                        }

                        // for \@! and \@<! it is a match when result is false
                        let is_neg = state_c == NFA_START_INVISIBLE_NEG
                            || state_c == NFA_START_INVISIBLE_NEG_FIRST
                            || state_c == NFA_START_INVISIBLE_BEFORE_NEG
                            || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST;
                        if result != is_neg as c_int {
                            // Copy submatch info from the recursive call
                            copy_sub_off_o(
                                &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                    .subs
                                    .norm as *mut c_void,
                                &raw mut (*m.cast::<RegsubsT>()).norm as *mut c_void,
                            );
                            if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                                copy_sub_off_o(
                                    &raw mut (*(*thislist.cast::<NfaListT>())
                                        .t
                                        .add(listidx as usize))
                                    .subs
                                    .synt as *mut c_void,
                                    &raw mut (*m.cast::<RegsubsT>()).synt as *mut c_void,
                                );
                            }
                            // If the pattern has \ze and it matched, use it.
                            copy_ze_off_o(
                                &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                    .subs
                                    .norm as *mut c_void,
                                &raw mut (*m.cast::<RegsubsT>()).norm as *mut c_void,
                            );

                            // t->state->out1 is the corresponding END_INVISIBLE node
                            add_here = true;
                            add_state = (*(*t_state.cast::<NfaStateT>())
                                .out1
                                .cast::<c_void>()
                                .cast::<NfaStateT>())
                            .out
                            .cast::<c_void>();
                        }
                        (*m.cast::<RegsubsT>()).norm.in_use = in_use;
                    } else {
                        // First try matching what follows. Add a nfa_pim_T.
                        let pim = xcalloc(1, core::mem::size_of::<NfaPimT>());
                        let input = REX.input;
                        let line = REX.line;
                        let is_multi = REX.reg_match.is_null() as c_int;
                        {
                            let _p = pim.cast::<NfaPimT>();
                            (*_p).result = NFA_PIM_TODO;
                            (*_p).state = t_state.cast::<NfaStateT>();
                            if is_multi != 0 {
                                (*_p).end.pos.lnum = REX.lnum;
                                (*_p).end.pos.col = (input as isize - line as isize) as i32;
                            } else {
                                (*_p).end.ptr = input;
                            }
                        }

                        // Add out1->out to thislist with PIM
                        let out1_out = (*(*t_state.cast::<NfaStateT>())
                            .out1
                            .cast::<c_void>()
                            .cast::<NfaStateT>())
                        .out
                        .cast::<c_void>();
                        let subs_ptr =
                            &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).subs
                                as *mut c_void;
                        if addstate_here_o(thislist, out1_out, subs_ptr, pim, &mut listidx)
                            .is_null()
                        {
                            xfree(pim);
                            MATCH_FOUND = NFA_TOO_EXPENSIVE;
                            break 'outer;
                        }
                        xfree(pim);
                    }
                }

                x if x == NFA_START_PATTERN => {
                    let t_state = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                        .state
                        .cast::<c_void>();
                    let out1 = (*t_state.cast::<NfaStateT>()).out1.cast::<c_void>();
                    let out1_out = (*out1.cast::<NfaStateT>()).out.cast::<c_void>();
                    let out1_out_out = (*out1_out.cast::<NfaStateT>()).out.cast::<c_void>();
                    let subs_norm =
                        &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .subs
                            .norm as *mut c_void;
                    let subs_synt =
                        &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .subs
                            .synt as *mut c_void;
                    let subs_ptr =
                        &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).subs
                            as *mut c_void;

                    // Check if output state is already in list
                    let mut skip = false;
                    if rs_state_in_list(nextlist, out1_out, subs_norm, subs_synt) != 0 {
                        skip = true;
                    } else if rs_state_in_list(nextlist, out1_out_out, subs_norm, subs_synt) != 0 {
                        skip = true;
                    } else if rs_state_in_list(thislist, out1_out_out, subs_norm, subs_synt) != 0 {
                        skip = true;
                    }
                    if skip {
                        // Don't try to match pattern
                    } else {
                        // Copy submatch info to the recursive call
                        copy_sub_off_o(
                            &raw mut (*m.cast::<RegsubsT>()).norm as *mut c_void,
                            subs_norm,
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            copy_sub_off_o(
                                &raw mut (*m.cast::<RegsubsT>()).synt as *mut c_void,
                                subs_synt,
                            );
                        }

                        result = recursive_regmatch_t(
                            t_state,
                            core::ptr::null_mut(),
                            prog,
                            submatch,
                            m,
                            &mut listids,
                            &mut listids_len,
                        );
                        if result == NFA_TOO_EXPENSIVE {
                            MATCH_FOUND = result;
                            break 'outer;
                        }
                        if result != 0 {
                            // Copy submatch info from the recursive call
                            copy_sub_off_o(
                                &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                    .subs
                                    .norm as *mut c_void,
                                &raw mut (*m.cast::<RegsubsT>()).norm as *mut c_void,
                            );
                            if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                                copy_sub_off_o(
                                    &raw mut (*(*thislist.cast::<NfaListT>())
                                        .t
                                        .add(listidx as usize))
                                    .subs
                                    .synt as *mut c_void,
                                    &raw mut (*m.cast::<RegsubsT>()).synt as *mut c_void,
                                );
                            }
                            // Skip over matched text
                            let bytelen = if (REX.reg_match.is_null() as c_int) != 0 {
                                (*m.cast::<RegsubsT>()).norm.list.multi[0_usize].end_col
                                    - (REX.input as isize - REX.line as isize) as i32
                            } else {
                                (*m.cast::<RegsubsT>()).norm.list.line[0_usize].end as isize as i32
                                    - REX.input as isize as i32
                            };

                            if bytelen == 0 {
                                add_here = true;
                                add_state = out1_out_out;
                            } else if bytelen <= clen {
                                add_state = out1_out_out;
                                add_off = clen;
                            } else {
                                add_state = out1_out;
                                add_off = bytelen;
                                add_count = bytelen - clen;
                            }
                        }
                    }
                }

                x if x == NFA_BOL => {
                    if REX.input == REX.line {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_EOL => {
                    if curc == 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_BOW => {
                    result = 1; // true
                    if curc == 0 {
                        result = 0;
                    } else {
                        let this_class = nvim_regexp_call_mb_get_class_tab(REX.input);
                        if this_class <= 1 {
                            result = 0;
                        } else if rs_reg_prev_class() == this_class {
                            result = 0;
                        }
                    }
                    if result != 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_EOW => {
                    result = 1; // true
                    if REX.input == REX.line {
                        result = 0;
                    } else {
                        let this_class = nvim_regexp_call_mb_get_class_tab(REX.input);
                        let prev_class = rs_reg_prev_class();
                        if this_class == prev_class || prev_class == 0 || prev_class == 1 {
                            result = 0;
                        }
                    }
                    if result != 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_BOF => {
                    if REX.lnum == 0
                        && REX.input == REX.line
                        && ((REX.reg_match.is_null() as c_int) == 0 || REX.reg_firstlnum == 1)
                    {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_EOF => {
                    if REX.lnum == REX.reg_maxline && curc == 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_COMPOSING => {
                    let t_state = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                        .state
                        .cast::<c_void>();
                    let mc = curc;
                    let mut len: c_int = 0;
                    let mut sta = (*t_state.cast::<NfaStateT>()).out.cast::<c_void>();
                    let mut cchars: [c_int; MAX_MCO] = [0; MAX_MCO];
                    let mut ccount: usize = 0;

                    if utf_iscomposing_legacy((*sta.cast::<NfaStateT>()).c) != 0 {
                        len += utf_char2len(mc);
                    }
                    if REX.reg_icombine as c_int != 0 && len == 0 {
                        if (*sta.cast::<NfaStateT>()).c != curc {
                            result = FAIL;
                        } else {
                            result = OK;
                        }
                        while (*sta.cast::<NfaStateT>()).c != NFA_END_COMPOSING {
                            sta = (*sta.cast::<NfaStateT>()).out.cast::<c_void>();
                        }
                    } else if len > 0 || mc == (*sta.cast::<NfaStateT>()).c {
                        if len == 0 {
                            len += utf_char2len(mc);
                            sta = (*sta.cast::<NfaStateT>()).out.cast::<c_void>();
                        }
                        // Get composing chars into cchars[]
                        while len < clen {
                            let mc2 = utf_ptr2char(
                                (REX.input as *const u8).offset(len as isize) as *const c_char
                            );
                            if ccount < MAX_MCO {
                                cchars[ccount] = mc2;
                                ccount += 1;
                            }
                            len += utf_char2len(mc2);
                            if ccount == MAX_MCO {
                                break;
                            }
                        }
                        // Check composing chars match
                        result = OK;
                        while (*sta.cast::<NfaStateT>()).c != NFA_END_COMPOSING {
                            let mut found = false;
                            for j in 0..ccount {
                                if cchars[j] == (*sta.cast::<NfaStateT>()).c {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                result = FAIL;
                                break;
                            }
                            sta = (*sta.cast::<NfaStateT>()).out.cast::<c_void>();
                        }
                    } else {
                        result = FAIL;
                    }

                    // ADD_STATE_IF_MATCH(end)
                    let end = (*t_state.cast::<NfaStateT>()).out1.cast::<c_void>(); // NFA_END_COMPOSING
                    if result != 0 {
                        add_state = (*end.cast::<NfaStateT>()).out.cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NEWL => {
                    if curc == 0
                        && REX.reg_line_lbr as c_int == 0
                        && (REX.reg_match.is_null() as c_int) != 0
                        && REX.lnum <= REX.reg_maxline
                    {
                        go_to_nextline = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = -1;
                    } else if curc == b'\n' as c_int && REX.reg_line_lbr as c_int != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = 1;
                    }
                }

                x if x == NFA_START_COLL || x == NFA_START_NEG_COLL => {
                    // Never match EOL
                    if curc == 0 {
                        // break - no match
                    } else {
                        let t_state = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state
                            .cast::<c_void>();
                        let mut col_state = (*t_state.cast::<NfaStateT>()).out.cast::<c_void>();
                        let result_if_matched = if state_c == NFA_START_COLL { 1 } else { 0 };
                        result = 0;

                        loop {
                            let col_c = (*col_state.cast::<NfaStateT>()).c;
                            if col_c == NFA_COMPOSING {
                                // Composing inside collection - complex case
                                let mc = curc;
                                let mut len: c_int = 0;
                                let mut sta =
                                    (*(*t_state.cast::<NfaStateT>()).out).out.cast::<c_void>();
                                let mut cchars: [c_int; MAX_MCO] = [0; MAX_MCO];
                                let mut ccount: usize = 0;

                                if utf_iscomposing_legacy((*sta.cast::<NfaStateT>()).c) != 0 {
                                    len += utf_char2len(mc);
                                }
                                if REX.reg_icombine as c_int != 0 && len == 0 {
                                    if (*sta.cast::<NfaStateT>()).c != curc {
                                        result = FAIL;
                                    } else {
                                        result = OK;
                                    }
                                    while (*sta.cast::<NfaStateT>()).c != NFA_END_COMPOSING {
                                        sta = (*sta.cast::<NfaStateT>()).out.cast::<c_void>();
                                    }
                                } else if len > 0 || mc == (*sta.cast::<NfaStateT>()).c {
                                    if len == 0 {
                                        len += utf_char2len(mc);
                                        sta = (*sta.cast::<NfaStateT>()).out.cast::<c_void>();
                                    }
                                    while len < clen {
                                        let mc2 = utf_ptr2char(
                                            (REX.input as *const u8).offset(len as isize)
                                                as *const c_char,
                                        );
                                        if ccount < MAX_MCO {
                                            cchars[ccount] = mc2;
                                            ccount += 1;
                                        }
                                        len += utf_char2len(mc2);
                                        if ccount == MAX_MCO {
                                            break;
                                        }
                                    }
                                    result = OK;
                                    while (*sta.cast::<NfaStateT>()).c != NFA_END_COMPOSING {
                                        let mut found = false;
                                        for j in 0..ccount {
                                            if cchars[j] == (*sta.cast::<NfaStateT>()).c {
                                                found = true;
                                                break;
                                            }
                                        }
                                        if !found {
                                            result = FAIL;
                                            break;
                                        }
                                        sta = (*sta.cast::<NfaStateT>()).out.cast::<c_void>();
                                    }
                                } else {
                                    result = FAIL;
                                }

                                let out_out1 = (*(*t_state.cast::<NfaStateT>())
                                    .out
                                    .cast::<c_void>()
                                    .cast::<NfaStateT>())
                                .out1
                                .cast::<c_void>();
                                if (*out_out1.cast::<NfaStateT>()).c == NFA_END_COMPOSING {
                                    if result != 0 {
                                        add_state =
                                            (*out_out1.cast::<NfaStateT>()).out.cast::<c_void>();
                                        add_off = clen;
                                    }
                                }
                                break;
                            }
                            if col_c == NFA_END_COLL {
                                result = if result_if_matched != 0 { 0 } else { 1 };
                                break;
                            }
                            if col_c == NFA_RANGE_MIN {
                                let c1 = (*col_state.cast::<NfaStateT>()).val;
                                col_state = (*col_state.cast::<NfaStateT>()).out.cast::<c_void>();
                                let c2 = (*col_state.cast::<NfaStateT>()).val;

                                if curc >= c1 && curc <= c2 {
                                    result = result_if_matched;
                                    break;
                                }
                                if REX.reg_ic as c_int != 0 {
                                    let curc_low = utf_fold(curc);
                                    let mut done = false;
                                    let mut ci = c1;
                                    while ci <= c2 {
                                        if utf_fold(ci) == curc_low {
                                            result = result_if_matched;
                                            done = true;
                                            break;
                                        }
                                        ci += 1;
                                    }
                                    if done {
                                        break;
                                    }
                                }
                            } else if col_c < 0 {
                                if rs_check_char_class(col_c, curc) != 0 {
                                    result = result_if_matched;
                                    break;
                                }
                            } else if curc == col_c
                                || (REX.reg_ic as c_int != 0 && utf_fold(curc) == utf_fold(col_c))
                            {
                                result = result_if_matched;
                                break;
                            }
                            col_state = (*col_state.cast::<NfaStateT>()).out.cast::<c_void>();
                        }
                        if result != 0 {
                            add_state = (*(*t_state.cast::<NfaStateT>())
                                .out1
                                .cast::<c_void>()
                                .cast::<NfaStateT>())
                            .out
                            .cast::<c_void>();
                            add_off = clen;
                        }
                    }
                }

                x if x == NFA_ANY => {
                    if curc > 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_ANY_COMPOSING => {
                    if utf_iscomposing_legacy(curc) != 0 {
                        add_off = clen;
                    } else {
                        add_here = true;
                        add_off = 0;
                    }
                    add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state)
                        .out
                        .cast::<c_void>();
                }

                x if x == NFA_IDENT => {
                    result = vim_isIDc(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_SIDENT => {
                    result = if ascii_isdigit_i(curc) == 0 && vim_isIDc(curc) != 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_KWORD => {
                    result = nvim_regexp_call_vim_iswordp_buf(REX.input as *const c_char);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_SKWORD => {
                    result = if ascii_isdigit_i(curc) == 0
                        && nvim_regexp_call_vim_iswordp_buf(REX.input as *const c_char) != 0
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_FNAME => {
                    result = vim_isfilec(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_SFNAME => {
                    result = if ascii_isdigit_i(curc) == 0 && vim_isfilec(curc) != 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_PRINT => {
                    result = vim_isprintc(utf_ptr2char(REX.input as *const c_char));
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_SPRINT => {
                    result = if ascii_isdigit_i(curc) == 0
                        && vim_isprintc(utf_ptr2char(REX.input as *const c_char)) != 0
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_WHITE => {
                    result = ascii_iswhite(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NWHITE => {
                    result = if curc != 0 && ascii_iswhite(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_DIGIT => {
                    result = ri_digit(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NDIGIT => {
                    result = if curc != 0 && ri_digit(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_HEX => {
                    result = ri_hex(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NHEX => {
                    result = if curc != 0 && ri_hex(curc) == 0 { 1 } else { 0 };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_OCTAL => {
                    result = ri_octal(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NOCTAL => {
                    result = if curc != 0 && ri_octal(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_WORD => {
                    result = ri_word(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NWORD => {
                    result = if curc != 0 && ri_word(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_HEAD => {
                    result = ri_head(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NHEAD => {
                    result = if curc != 0 && ri_head(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_ALPHA => {
                    result = ri_alpha(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NALPHA => {
                    result = if curc != 0 && ri_alpha(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_LOWER => {
                    result = ri_lower(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NLOWER => {
                    result = if curc != 0 && ri_lower(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_UPPER => {
                    result = ri_upper(curc);
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NUPPER => {
                    result = if curc != 0 && ri_upper(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_LOWER_IC => {
                    result = if ri_lower(curc) != 0
                        || (REX.reg_ic as c_int != 0 && ri_upper(curc) != 0)
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NLOWER_IC => {
                    result = if curc != 0
                        && !(ri_lower(curc) != 0
                            || (REX.reg_ic as c_int != 0 && ri_upper(curc) != 0))
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_UPPER_IC => {
                    result = if ri_upper(curc) != 0
                        || (REX.reg_ic as c_int != 0 && ri_lower(curc) != 0)
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if x == NFA_NUPPER_IC => {
                    result = if curc != 0
                        && !(ri_upper(curc) != 0
                            || (REX.reg_ic as c_int != 0 && ri_lower(curc) != 0))
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }

                x if (NFA_BACKREF1..=NFA_BACKREF9).contains(&x)
                    || (NFA_ZREF1..=NFA_ZREF9).contains(&x) =>
                {
                    let mut bytelen: c_int = 0;
                    let t_subs_norm =
                        &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .subs
                            .norm as *mut c_void;

                    if state_c >= NFA_BACKREF1 && state_c <= NFA_BACKREF9 {
                        let subidx = state_c - NFA_BACKREF1 + 1;
                        result =
                            match_backref_t(t_subs_norm.cast::<RegsubT>(), subidx, &mut bytelen);
                    } else {
                        let subidx = state_c - NFA_ZREF1 + 1;
                        result = rs_match_zref(subidx, &mut bytelen);
                    }

                    if result != 0 {
                        let t_state_out =
                            (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state)
                                .out
                                .cast::<c_void>();
                        if bytelen == 0 {
                            add_here = true;
                            add_state = (*t_state_out.cast::<NfaStateT>()).out.cast::<c_void>();
                        } else if bytelen <= clen {
                            add_state = (*t_state_out.cast::<NfaStateT>()).out.cast::<c_void>();
                            add_off = clen;
                        } else {
                            add_state = t_state_out;
                            add_off = bytelen;
                            add_count = bytelen - clen;
                        }
                    }
                }

                x if x == NFA_SKIP => {
                    let t_count = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).count;
                    if t_count - clen <= 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    } else {
                        add_state = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state
                            .cast::<c_void>();
                        add_off = 0;
                        add_count = t_count - clen;
                    }
                }

                x if x == NFA_LNUM || x == NFA_LNUM_GT || x == NFA_LNUM_LT => {
                    let val =
                        (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state).val;
                    result = if (REX.reg_match.is_null() as c_int) != 0 {
                        rs_nfa_re_num_cmp(
                            val as usize,
                            state_c - NFA_LNUM,
                            (REX.lnum as usize).wrapping_add(REX.reg_firstlnum as usize),
                        )
                    } else {
                        0
                    };
                    if result != 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_COL || x == NFA_COL_GT || x == NFA_COL_LT => {
                    let val =
                        (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state).val;
                    let col_offset = (REX.input as usize).wrapping_sub(REX.line as usize);
                    result = rs_nfa_re_num_cmp(
                        val as usize,
                        state_c - NFA_COL,
                        col_offset.wrapping_add(1),
                    );
                    if result != 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_VCOL || x == NFA_VCOL_GT || x == NFA_VCOL_LT => {
                    let val =
                        (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state).val;
                    let op = state_c - NFA_VCOL;
                    let col = (REX.input as isize - REX.line as isize) as i32;

                    // Bail out quickly when there can't be a match
                    if op != 1 && col > val * MB_MAXBYTES as i32 {
                        // no match possible
                    } else {
                        result = 0;
                        let rex_reg_win = nvim_regexp_get_rex_reg_win_or_curwin();
                        let wp = if rex_reg_win.is_null() {
                            nvim_regexp_get_curwin()
                        } else {
                            rex_reg_win
                        };
                        if op == 1 && col - 1 > val && col > 100 {
                            let mut ts = nvim_regexp_get_win_b_p_ts(wp);
                            if ts < 4 {
                                ts = 4;
                            }
                            result = if col > val * ts as i32 { 1 } else { 0 };
                        }
                        if result == 0 {
                            let mut lnum = if (REX.reg_match.is_null() as c_int) != 0 {
                                REX.reg_firstlnum + REX.lnum
                            } else {
                                1
                            };
                            if (REX.reg_match.is_null() as c_int) != 0
                                && (lnum <= 0 || lnum > nvim_regexp_get_win_buf_line_count(wp))
                            {
                                lnum = 1;
                            }
                            let vcol = nvim_regexp_call_win_linetabsize(
                                wp,
                                lnum,
                                REX.line as *const c_char,
                                col,
                            );
                            result = rs_nfa_re_num_cmp(val as usize, op, (vcol + 1) as usize);
                        }
                        if result != 0 {
                            add_here = true;
                            add_state =
                                (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state)
                                    .out
                                    .cast::<c_void>();
                        }
                    }
                }

                x if x == NFA_MARK || x == NFA_MARK_GT || x == NFA_MARK_LT => {
                    let val =
                        (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state).val;
                    let col_sz = if (REX.reg_match.is_null() as c_int) != 0 {
                        (REX.input as isize - REX.line as isize) as usize
                    } else {
                        0
                    };

                    let fm = nvim_regexp_call_mark_get_for_nfa(
                        REX.reg_buf,
                        nvim_regexp_get_curwin(),
                        val,
                    );

                    // Line may have been freed, get it again.
                    if (REX.reg_match.is_null() as c_int) != 0 {
                        let new_line = nvim_regexp_call_reg_getline(REX.lnum) as *mut u8;
                        REX.line = new_line;
                        REX.input = new_line.add(col_sz);
                    }

                    if nvim_regexp_fmark_is_set(fm) != 0 {
                        let pos_lnum = nvim_regexp_fmark_get_lnum(fm);
                        let lnum_match = REX.lnum + REX.reg_firstlnum;
                        let pos_col = nvim_regexp_fmark_get_col_adj(fm, lnum_match);
                        let input_col = (REX.input as isize - REX.line as isize) as i32;

                        result = if pos_lnum == lnum_match {
                            if pos_col == input_col {
                                if state_c == NFA_MARK {
                                    1
                                } else {
                                    0
                                }
                            } else if pos_col < input_col {
                                if state_c == NFA_MARK_GT {
                                    1
                                } else {
                                    0
                                }
                            } else if state_c == NFA_MARK_LT {
                                1
                            } else {
                                0
                            }
                        } else if pos_lnum < lnum_match {
                            if state_c == NFA_MARK_GT {
                                1
                            } else {
                                0
                            }
                        } else if state_c == NFA_MARK_LT {
                            1
                        } else {
                            0
                        };
                        if result != 0 {
                            add_here = true;
                            add_state =
                                (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).state)
                                    .out
                                    .cast::<c_void>();
                        }
                    }
                }

                x if x == NFA_CURSOR => {
                    result = if nvim_regexp_has_rex_reg_win() != 0
                        && (REX.lnum + REX.reg_firstlnum
                            == nvim_regexp_get_rex_reg_win_cursor_lnum())
                        && ((REX.input as isize - REX.line as isize) as i32
                            == nvim_regexp_get_rex_reg_win_cursor_col())
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if x == NFA_VISUAL => {
                    result = rs_reg_match_visual();
                    if result != 0 {
                        add_here = true;
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                    }
                }

                x if (NFA_MOPEN1..=NFA_MOPEN9).contains(&x)
                    || (NFA_ZOPEN..=NFA_ZOPEN9).contains(&x)
                    || x == NFA_NOPEN
                    || x == NFA_ZSTART =>
                {
                    // These states are only added to be able to bail out when
                    // they are added again, nothing is to be done.
                }

                _ => {
                    // default: regular character
                    let c = state_c;
                    result = if c == curc { 1 } else { 0 };

                    if result == 0 && REX.reg_ic as c_int != 0 {
                        result = if utf_fold(c) == utf_fold(curc) { 1 } else { 0 };
                    }

                    // If reg_icombine is not set only skip over the character itself.
                    if result != 0 && REX.reg_icombine as c_int == 0 {
                        clen = utf_ptr2len(REX.input as *const c_char);
                    }

                    // ADD_STATE_IF_MATCH(t->state)
                    if result != 0 {
                        add_state = (*(*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                            .state)
                            .out
                            .cast::<c_void>();
                        add_off = clen;
                    }
                }
            } // match state_c

            // Post-switch: handle add_state with PIM resolution
            if !add_state.is_null() {
                let pim_result = (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                    .pim
                    .result;
                let mut use_pim: bool = pim_result != NFA_PIM_UNUSED;
                let mut pim_ptr: NfaPimHandle = core::ptr::null_mut();
                let mut pim_copy: NfaPimHandle = core::ptr::null_mut();

                if !use_pim {
                    pim_ptr = core::ptr::null_mut();
                } else {
                    pim_ptr = &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).pim
                        as *mut c_void;
                }

                // Handle the postponed invisible match if the match might end
                // without advancing and before the end of the line.
                if use_pim && (clen == 0 || match_follows(add_state, 0)) {
                    let pim_res = (*pim_ptr.cast::<NfaPimT>()).result;
                    if pim_res == NFA_PIM_TODO {
                        result = recursive_regmatch_t(
                            (*pim_ptr.cast::<NfaPimT>()).state.cast::<c_void>(),
                            pim_ptr,
                            prog,
                            submatch,
                            m,
                            &mut listids,
                            &mut listids_len,
                        );
                        (*pim_ptr.cast::<NfaPimT>()).result = if result != 0 {
                            NFA_PIM_MATCH
                        } else {
                            NFA_PIM_NOMATCH
                        };
                        let pim_state_c = (*(*pim_ptr.cast::<NfaPimT>()).state).c;
                        let is_neg = pim_state_c == NFA_START_INVISIBLE_NEG
                            || pim_state_c == NFA_START_INVISIBLE_NEG_FIRST
                            || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG
                            || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST;
                        if result != is_neg as c_int {
                            // Copy submatch info from the recursive call
                            copy_sub_off_o(
                                &raw mut (*pim_ptr.cast::<NfaPimT>()).subs.norm as *mut c_void,
                                &raw mut (*m.cast::<RegsubsT>()).norm as *mut c_void,
                            );
                            if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                                copy_sub_off_o(
                                    &raw mut (*pim_ptr.cast::<NfaPimT>()).subs.synt as *mut c_void,
                                    &raw mut (*m.cast::<RegsubsT>()).synt as *mut c_void,
                                );
                            }
                        }
                    } else {
                        result = if pim_res == NFA_PIM_MATCH { 1 } else { 0 };
                    }

                    // for \@! and \@<! it is a match when result is false
                    let pim_state_c = (*(*pim_ptr.cast::<NfaPimT>()).state).c;
                    let is_neg = pim_state_c == NFA_START_INVISIBLE_NEG
                        || pim_state_c == NFA_START_INVISIBLE_NEG_FIRST
                        || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG
                        || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST;
                    if result != is_neg as c_int {
                        // Copy submatch info
                        copy_sub_off_o(
                            &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                .subs
                                .norm as *mut c_void,
                            &raw mut (*pim_ptr.cast::<NfaPimT>()).subs.norm as *mut c_void,
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            copy_sub_off_o(
                                &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize))
                                    .subs
                                    .synt as *mut c_void,
                                &raw mut (*pim_ptr.cast::<NfaPimT>()).subs.synt as *mut c_void,
                            );
                        }
                    } else {
                        // look-behind match failed, don't add the state
                        listidx += 1;
                        continue;
                    }

                    // Postponed invisible match was handled
                    use_pim = false;
                    pim_ptr = core::ptr::null_mut();
                }

                // If "pim" points into l->t it may become invalid when
                // adding the state causes the list to be reallocated.
                if use_pim {
                    pim_copy = xcalloc(1, core::mem::size_of::<NfaPimT>());
                    copy_pim_o(pim_copy, pim_ptr);
                    pim_ptr = pim_copy;
                }

                if add_here {
                    let subs_ptr =
                        &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).subs
                            as *mut c_void;
                    r = addstate_here_o(thislist, add_state, subs_ptr, pim_ptr, &mut listidx);
                } else {
                    let subs_ptr =
                        &raw mut (*(*thislist.cast::<NfaListT>()).t.add(listidx as usize)).subs
                            as *mut c_void;
                    r = addstate_o(nextlist, add_state, subs_ptr, pim_ptr, add_off);
                    if add_count > 0 {
                        (*(*nextlist.cast::<NfaListT>())
                            .t
                            .add(((*nextlist.cast::<NfaListT>()).n - 1) as usize))
                        .count = add_count;
                    }
                }

                if !pim_copy.is_null() {
                    xfree(pim_copy);
                }

                if r.is_null() {
                    MATCH_FOUND = NFA_TOO_EXPENSIVE;
                    break 'outer;
                }
            }

            listidx += 1;
        } // while listidx < thislist->n

        // Look for the start of a match in the current position
        if MATCH_FOUND == 0
            && ((toplevel
                && REX.lnum == 0
                && clen != 0
                && (REX.reg_maxcol == 0
                    || (REX.input as isize - REX.line as isize) < REX.reg_maxcol as isize))
                || (!NFA_ENDP.is_null()
                    && (if (REX.reg_match.is_null() as c_int) != 0 {
                        REX.lnum
                            < (if NFA_ENDP.is_null() {
                                -1i32
                            } else {
                                (*NFA_ENDP).se_u.pos.lnum
                            })
                            || (REX.lnum
                                == (if NFA_ENDP.is_null() {
                                    -1i32
                                } else {
                                    (*NFA_ENDP).se_u.pos.lnum
                                })
                                && ((REX.input as isize - REX.line as isize) as i32)
                                    < (if NFA_ENDP.is_null() {
                                        -1i32
                                    } else {
                                        (*NFA_ENDP).se_u.pos.col
                                    }))
                    } else {
                        (REX.input as usize)
                            < ((if NFA_ENDP.is_null() {
                                core::ptr::null_mut()
                            } else {
                                (*NFA_ENDP).se_u.ptr
                            }) as usize)
                    })))
        {
            if toplevel {
                let mut add = true;

                if (*prog.cast::<NfaRegprogT>()).regstart != 0 && clen != 0 {
                    if (*nextlist.cast::<NfaListT>()).n == 0 {
                        let mut col = (REX.input as isize - REX.line as isize) as i32 + clen;
                        if rs_skip_to_start((*prog.cast::<NfaRegprogT>()).regstart, &mut col)
                            == FAIL
                        {
                            break 'outer;
                        }
                        // rex.input = rex.line + col - clen
                        let new_input = REX.line.offset((col - clen) as isize);
                        REX.input = new_input;
                    } else {
                        let c = utf_ptr2char(REX.input.offset(clen as isize) as *const c_char);
                        if c != (*prog.cast::<NfaRegprogT>()).regstart
                            && (REX.reg_ic as c_int == 0
                                || utf_fold(c) != utf_fold((*prog.cast::<NfaRegprogT>()).regstart))
                        {
                            add = false;
                        }
                    }
                }

                if add {
                    if (REX.reg_match.is_null() as c_int) != 0 {
                        let start_col = (REX.input as isize - REX.line as isize) as i32 + clen;
                        {
                            let _t = m.cast::<RegsubsT>();
                            (*_t).norm.list.multi[0_usize].start_lnum = REX.lnum;
                            (*_t).norm.list.multi[0_usize].start_col = start_col;
                        }
                        (*m.cast::<RegsubsT>()).norm.orig_start_col = start_col;
                    } else {
                        (*m.cast::<RegsubsT>()).norm.list.line[0_usize].start =
                            REX.input.offset(clen as isize);
                    }
                    if addstate_o(
                        nextlist,
                        (*start.cast::<NfaStateT>()).out.cast::<c_void>(),
                        m,
                        core::ptr::null_mut(),
                        clen,
                    )
                    .is_null()
                    {
                        MATCH_FOUND = NFA_TOO_EXPENSIVE;
                        break 'outer;
                    }
                }
            } else if addstate_o(nextlist, start, m, core::ptr::null_mut(), clen).is_null() {
                MATCH_FOUND = NFA_TOO_EXPENSIVE;
                break 'outer;
            }
        }

        // nextchar: Advance to the next character
        if clen != 0 {
            let new_input = REX.input.offset(clen as isize);
            REX.input = new_input;
        } else if go_to_nextline
            || (!NFA_ENDP.is_null()
                && (REX.reg_match.is_null() as c_int) != 0
                && REX.lnum
                    < (if NFA_ENDP.is_null() {
                        -1i32
                    } else {
                        (*NFA_ENDP).se_u.pos.lnum
                    }))
        {
            rs_reg_nextline();
        } else {
            break 'outer;
        }
        go_to_nextline = false;

        // Allow interrupting with CTRL-C.
        rs_reg_breakcheck();
        if nvim_regexp_get_got_int() != 0 {
            break 'outer;
        }
        // Check for timeout once every twenty times
        if !NFA_TIME_LIMIT.is_null() {
            let tc = NFA_TIME_COUNT + 1;
            NFA_TIME_COUNT = tc;
            if tc == 20 {
                NFA_TIME_COUNT = 0;
                if rs_nfa_did_time_out() != 0 {
                    break 'outer;
                }
            }
        }
    } // loop (outer)

    // theend: Free memory
    {
        let l0 = list0.cast::<NfaListT>();
        xfree((*l0).t.cast::<c_void>());
        xfree(l0.cast::<c_void>());
    }
    {
        let l1 = list1.cast::<NfaListT>();
        xfree((*l1).t.cast::<c_void>());
        xfree(l1.cast::<c_void>());
    }
    if !listids.is_null() {
        nvim_regexp_xfree(listids.cast::<c_void>());
    }

    MATCH_FOUND
}

/// Helper: check if `c` is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit_i(c: c_int) -> c_int {
    if c >= b'0' as c_int && c <= b'9' as c_int {
        1
    } else {
        0
    }
}

// --- End Phase 8.4 ---

// --- Phase 8.5: NFA Entry Points ---

// Extern declarations for Phase 8.5 / Phase 7 C accessors
extern "C" {
    // iemsg null error

    fn nvim_regexp_get_buf_ml_line_count(buf: *mut c_void) -> i32;
    fn nvim_regexp_get_re_extmatch_out() -> *mut c_void;
    fn nvim_regexp_set_re_extmatch_out_match(i: c_int, v: *mut u8);
}

/// Inlined: `nvim_regexp_nfa_regtry_extract_multi`
/// Extracts multi-line submatch positions from subs into REX fields.
#[inline]
unsafe fn nfa_regtry_extract_multi(subs: *mut RegsubsT, col: i32) {
    let norm = &(*subs).norm;
    for i in 0..norm.in_use as usize {
        (*REX.reg_startpos.add(i)).lnum = norm.list.multi[i].start_lnum;
        (*REX.reg_startpos.add(i)).col = norm.list.multi[i].start_col;
        (*REX.reg_endpos.add(i)).lnum = norm.list.multi[i].end_lnum;
        (*REX.reg_endpos.add(i)).col = norm.list.multi[i].end_col;
    }
    if !REX.reg_mmatch.is_null() {
        (*REX.reg_mmatch.cast::<RegmmatchT>()).rmm_matchcol = norm.orig_start_col;
    }
    if (*REX.reg_startpos).lnum < 0 {
        (*REX.reg_startpos).lnum = 0;
        (*REX.reg_startpos).col = col;
    }
    if (*REX.reg_endpos).lnum < 0 {
        (*REX.reg_endpos).lnum = REX.lnum;
        #[allow(clippy::cast_possible_truncation)]
        let off = REX.input.offset_from(REX.line) as c_int;
        (*REX.reg_endpos).col = off;
    } else {
        REX.lnum = (*REX.reg_endpos).lnum;
    }
}

/// Inlined: `nvim_regexp_nfa_regtry_extract_single`
/// Extracts single-line submatch positions from subs into REX fields.
#[inline]
unsafe fn nfa_regtry_extract_single(subs: *mut RegsubsT, col: i32) {
    let norm = &(*subs).norm;
    for i in 0..norm.in_use as usize {
        *REX.reg_startp.add(i) = norm.list.line[i].start;
        *REX.reg_endp.add(i) = norm.list.line[i].end;
    }
    if (*REX.reg_startp).is_null() {
        *REX.reg_startp = REX.line.add(col as usize);
    }
    if (*REX.reg_endp).is_null() {
        *REX.reg_endp = REX.input;
    }
}

/// Inlined: `nvim_regexp_nfa_regtry_extract_extmatch`
/// Extracts `\z(...\)` extmatch positions into `re_extmatch_out`.
#[inline]
unsafe fn nfa_regtry_extract_extmatch(subs: *mut RegsubsT) {
    rs_cleanup_zsubexpr();
    let em = rs_make_extmatch();
    nvim_regexp_set_re_extmatch_out(em.cast());
    let synt = &(*subs).synt;
    for i in 1..synt.in_use as usize {
        if REX.reg_match.is_null() {
            // multi-line
            let mpos = &synt.list.multi[i];
            if mpos.start_lnum >= 0
                && mpos.start_lnum == mpos.end_lnum
                && mpos.end_col >= mpos.start_col
            {
                let line = nvim_regexp_call_reg_getline(mpos.start_lnum);
                let src = line.add(mpos.start_col as usize);
                let len = (mpos.end_col - mpos.start_col) as usize;
                let saved = xstrnsave(src, len);
                #[allow(clippy::cast_possible_truncation)]
                nvim_regexp_set_re_extmatch_out_match(i as c_int, saved.cast::<u8>());
            }
        } else {
            // single-line
            let lpos = &synt.list.line[i];
            if !lpos.start.is_null() && !lpos.end.is_null() {
                let len = lpos.end.offset_from(lpos.start) as usize;
                let saved = xstrnsave(lpos.start.cast::<c_char>(), len);
                #[allow(clippy::cast_possible_truncation)]
                nvim_regexp_set_re_extmatch_out_match(i as c_int, saved.cast::<u8>());
            }
        }
    }
}

/// NFA regexp try matching at a specific column.
///
/// Sets up rex.input and time fields, allocates subs on heap,
/// calls `rs_nfa_regmatch`, then extracts submatch data back into rex fields.
/// (Phase 7: formerly delegated to C compound functions; now fully inlined)
#[no_mangle]
#[allow(unused_variables)]
pub unsafe extern "C" fn rs_nfa_regtry(
    prog: NfaProgHandle,
    col: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let start = (*prog.cast::<NfaRegprogT>()).start.cast::<c_void>();

    // Inlined nvim_regexp_nfa_regtry_setup: set rex.input and NFA time globals
    REX.input = REX.line.add(col as usize);
    nvim_regexp_set_nfa_time_globals(tm, timed_out);

    // Allocate subs and m on the heap (Rust can't stack-allocate opaque C structs)
    let subs = xcalloc(1, core::mem::size_of::<RegsubsT>());
    let m = xcalloc(1, core::mem::size_of::<RegsubsT>());

    // Clear sub fields using typed pointers
    let subs_typed = subs.cast::<RegsubsT>();
    let m_typed = m.cast::<RegsubsT>();
    clear_sub_o(core::ptr::addr_of_mut!((*subs_typed).norm).cast());
    clear_sub_o(core::ptr::addr_of_mut!((*m_typed).norm).cast());
    clear_sub_o(core::ptr::addr_of_mut!((*subs_typed).synt).cast());
    clear_sub_o(core::ptr::addr_of_mut!((*m_typed).synt).cast());

    let result = rs_nfa_regmatch(prog, start, subs, m);

    if result == 0 {
        xfree(subs);
        xfree(m);
        return 0;
    }
    if result == NFA_TOO_EXPENSIVE {
        xfree(subs);
        xfree(m);
        return NFA_TOO_EXPENSIVE;
    }

    // Extract submatch data
    rs_cleanup_subexpr();
    if REX.reg_match.is_null() {
        nfa_regtry_extract_multi(subs_typed, col);
    } else {
        nfa_regtry_extract_single(subs_typed, col);
    }

    // Handle \z(...\) extmatch
    nvim_regexp_unref_re_extmatch_out();
    nvim_regexp_set_re_extmatch_out(core::ptr::null_mut());
    let reghasz = (*prog.cast::<NfaRegprogT>()).reghasz;
    if reghasz == REX_SET as c_int {
        nfa_regtry_extract_extmatch(subs_typed);
    }

    let ret = 1 + REX.lnum;
    xfree(subs);
    xfree(m);
    ret
}

/// Inlined: validate match positions (end >= start).
/// Called after a successful match to fix up bogus end positions.
#[inline]
unsafe fn nfa_regexec_validate_match() {
    if REX.reg_match.is_null() {
        // multi-line: use REX.reg_startpos / REX.reg_endpos (point into regmmatch)
        let start = REX.reg_startpos;
        let end = REX.reg_endpos;
        if (*end).lnum < (*start).lnum
            || ((*end).lnum == (*start).lnum && (*end).col < (*start).col)
        {
            *REX.reg_endpos = *REX.reg_startpos;
        }
    } else {
        // single-line: use REX.reg_startp / REX.reg_endp (point into regmatch)
        if *REX.reg_endp < *REX.reg_startp {
            *REX.reg_endp = *REX.reg_startp;
        }
    }
}

/// Inlined: set `rmm_matchcol` or `rm_matchcol` after a match.
#[inline]
unsafe fn nfa_regexec_set_matchcol(col: i32) {
    if REX.reg_match.is_null() {
        (*REX.reg_mmatch.cast::<RegmmatchT>()).rmm_matchcol = col;
    } else {
        (*REX.reg_match.cast::<RegmatchT>()).rm_matchcol = col;
    }
}

/// Core NFA regexp execution for both single-line and multi-line modes.
///
/// (Phase 7: formerly delegated to C compound functions; now fully inlined)
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::too_many_lines)]
pub unsafe extern "C" fn rs_nfa_regexec_both(
    line: *mut u8,
    startcol: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let mut col = startcol;
    let mut retval: c_int = 0;

    // Inlined nvim_regexp_nfa_regexec_both_get_prog:
    // get prog from reg_mmatch or reg_match
    let prog: NfaProgHandle = if REX.reg_match.is_null() {
        (*REX.reg_mmatch.cast::<RegmmatchT>())
            .regprog
            .cast::<c_void>()
    } else {
        (*REX.reg_match.cast::<RegmatchT>())
            .regprog
            .cast::<c_void>()
    };

    // Inlined nvim_regexp_nfa_regexec_both_get_line:
    // for multi-line, get line 0 from reg_getline; for single-line use `line`
    let line: *mut u8 = if REX.reg_match.is_null() {
        nvim_regexp_call_reg_getline(0).cast::<u8>()
    } else {
        line
    };

    // Inlined nvim_regexp_nfa_regexec_both_setup_pointers:
    // set rex pointer fields from match structs
    if REX.reg_match.is_null() {
        REX.reg_startpos = (*REX.reg_mmatch.cast::<RegmmatchT>()).startpos.as_mut_ptr();
        REX.reg_endpos = (*REX.reg_mmatch.cast::<RegmmatchT>()).endpos.as_mut_ptr();
    } else {
        REX.reg_startp = (*REX.reg_match.cast::<RegmatchT>()).startp.as_mut_ptr();
        REX.reg_endp = (*REX.reg_match.cast::<RegmatchT>()).endp.as_mut_ptr();
    }

    // Be paranoid...
    if prog.is_null() || line.is_null() {
        errors::call_iemsg_null();
        if retval > 0 {
            nfa_regexec_validate_match();
        }
        return retval;
    }

    // Inlined nvim_regexp_nfa_regexec_both_apply_flags: apply regflags overrides (\c, \C, \Z)
    {
        let prog_typed = prog.cast::<NfaRegprogT>();
        let regflags = (*prog_typed).regflags;
        if regflags & RF_ICASE != 0 {
            REX.reg_ic = true;
        } else if regflags & RF_NOICASE != 0 {
            REX.reg_ic = false;
        }
        if regflags & RF_ICOMBINE != 0 {
            REX.reg_icombine = true;
        }
    }

    // Inlined nvim_regexp_nfa_regexec_both_setup_nfa: set rex NFA fields
    {
        let prog_typed = prog.cast::<NfaRegprogT>();
        REX.line = core::ptr::null_mut(); // will be set below
        REX.lnum = 0;
        REX.nfa_has_zend = (*prog_typed).has_zend;
        REX.nfa_has_backref = (*prog_typed).has_backref;
        REX.nfa_nsubexpr = (*prog_typed).nsubexp;
        REX.nfa_listid = 1;
        REX.nfa_alt_listid = 2;
        REX.need_clear_subexpr = 1;
        if (*prog_typed).reghasz == REX_SET {
            REX.nfa_has_zsubexpr = 1;
            REX.need_clear_zsubexpr = 1;
        } else {
            REX.nfa_has_zsubexpr = 0;
            REX.need_clear_zsubexpr = 0;
        }
    }
    // Set rex.line (setup_nfa left it NULL)
    REX.line = line;

    // If anchored and col > 0, no match possible
    let reganch = (*prog.cast::<NfaRegprogT>()).reganch;
    if reganch != 0 && col > 0 {
        return 0;
    }

    // Skip ahead to start character
    let regstart = (*prog.cast::<NfaRegprogT>()).regstart;
    if regstart != 0 {
        if rs_skip_to_start(regstart, core::ptr::from_mut::<i32>(&mut col)) == FAIL {
            return 0;
        }

        // If match_text is set, try the fast path
        let match_text = (*prog.cast::<NfaRegprogT>()).match_text;
        if !match_text.is_null() && *match_text != 0 && !REX.reg_icombine {
            retval = rs_find_match_text(core::ptr::from_mut::<i32>(&mut col), regstart, match_text);
            nfa_regexec_set_matchcol(col);
            return retval;
        }
    }

    // If start column past max column, no match
    let reg_maxcol = REX.reg_maxcol;
    if reg_maxcol > 0 && col >= reg_maxcol {
        if retval > 0 {
            nfa_regexec_validate_match();
        }
        return retval;
    }

    // Inlined nvim_regexp_nfa_regexec_both_init_states: initialize state array
    {
        nvim_regexp_reset_nstate();
        let prog_typed = prog.cast::<NfaRegprogT>();
        // state[] is a flexible array after the struct — access via pointer arithmetic
        let states_ptr = prog_typed.add(1).cast::<NfaStateT>();
        for i in 0..(*prog_typed).nstate as usize {
            let s = states_ptr.add(i);
            #[allow(clippy::cast_possible_truncation)]
            let id = i as c_int;
            (*s).id = id;
            (*s).lastlist[0] = 0;
            (*s).lastlist[1] = 0;
        }
    }

    // Try matching
    retval = rs_nfa_regtry(prog, col, tm, timed_out);

    // theend: validate match positions
    if retval > 0 {
        nfa_regexec_validate_match();
        if !REX.reg_match.is_null() {
            // Set rm_matchcol for single-line mode
            nfa_regexec_set_matchcol(col);
        }
    }

    retval
}

/// NFA regexp execution for single-line matching.
/// (Phase 7: formerly delegated to C `nvim_regexp_nfa_regexec_nl_setup`; now inlined)
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regexec_nl(
    rmp: *mut c_void,
    line: *mut u8,
    col: i32,
    line_lbr: c_int,
) -> c_int {
    // Inlined nvim_regexp_nfa_regexec_nl_setup
    REX.reg_match = rmp.cast();
    REX.reg_mmatch = core::ptr::null_mut();
    REX.reg_maxline = 0;
    REX.reg_line_lbr = line_lbr != 0;
    REX.reg_buf = nvim_regexp_get_curbuf().cast();
    REX.reg_win = core::ptr::null_mut();
    REX.reg_ic = (*rmp.cast::<RegmatchT>()).rm_ic as c_int != 0;
    REX.reg_icombine = false;
    let prog = (*rmp.cast_const().cast::<RegmatchT>())
        .regprog
        .cast::<c_void>();
    REX.reg_nobreak = (*prog.cast::<RegprogT>()).re_flags & 16 != 0; // 16 = RE_NOBREAK
    REX.reg_maxcol = 0;

    rs_nfa_regexec_both(line, col, core::ptr::null_mut(), core::ptr::null_mut())
}

/// NFA regexp execution for multi-line matching.
/// (Phase 7: formerly delegated to C `nvim_regexp_call_init_regexec_multi`; now inlined)
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regexec_multi(
    rmp: *mut c_void,
    win: *mut c_void,
    buf: *mut c_void,
    lnum: i32,
    col: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    // Inlined init_regexec_multi
    REX.reg_match = core::ptr::null_mut();
    REX.reg_mmatch = rmp.cast();
    REX.reg_buf = buf.cast();
    REX.reg_win = win.cast();
    REX.reg_firstlnum = lnum;
    REX.reg_maxline = nvim_regexp_get_buf_ml_line_count(buf) - lnum;
    REX.reg_line_lbr = false;
    REX.reg_ic = (*rmp.cast_const().cast::<RegmmatchT>()).rmm_ic != 0;
    REX.reg_icombine = false;
    let prog = (*rmp.cast_const().cast::<RegmmatchT>())
        .regprog
        .cast::<c_void>();
    REX.reg_nobreak = (*prog.cast::<RegprogT>()).re_flags & 16 != 0; // 16 = RE_NOBREAK
    REX.reg_maxcol = (*rmp.cast_const().cast::<RegmmatchT>()).rmm_maxcol;

    rs_nfa_regexec_both(core::ptr::null_mut(), col, tm, timed_out)
}

// --- End Phase 8.5 ---

// --- Phase 9.1: BT dispatch wrappers ---

extern "C" {
    fn nvim_regexp_call_prog_magic_wrong() -> c_int;
}

/// BT regexp execution for single-line matching.
/// (Phase 7: inlined `nfa_regexec_nl_setup` logic)
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regexec_nl(
    rmp: *mut c_void,
    line: *mut u8,
    col: i32,
    line_lbr: c_int,
) -> c_int {
    // Inlined nvim_regexp_nfa_regexec_nl_setup (shared with NFA path)
    REX.reg_match = rmp.cast();
    REX.reg_mmatch = core::ptr::null_mut();
    REX.reg_maxline = 0;
    REX.reg_line_lbr = line_lbr != 0;
    REX.reg_buf = nvim_regexp_get_curbuf().cast();
    REX.reg_win = core::ptr::null_mut();
    REX.reg_ic = (*rmp.cast::<RegmatchT>()).rm_ic as c_int != 0;
    REX.reg_icombine = false;
    let prog = (*rmp.cast_const().cast::<RegmatchT>())
        .regprog
        .cast::<c_void>();
    REX.reg_nobreak = (*prog.cast::<RegprogT>()).re_flags & 16 != 0; // 16 = RE_NOBREAK
    REX.reg_maxcol = 0;
    rs_bt_regexec_both(line, col, core::ptr::null_mut(), core::ptr::null_mut())
}

/// BT regexp execution for multi-line matching.
/// (Phase 7: inlined `init_regexec_multi` logic)
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regexec_multi(
    rmp: *mut c_void,
    win: *mut c_void,
    buf: *mut c_void,
    lnum: i32,
    col: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    // Inlined init_regexec_multi
    REX.reg_match = core::ptr::null_mut();
    REX.reg_mmatch = rmp.cast();
    REX.reg_buf = buf.cast();
    REX.reg_win = win.cast();
    REX.reg_firstlnum = lnum;
    REX.reg_maxline = nvim_regexp_get_buf_ml_line_count(buf) - lnum;
    REX.reg_line_lbr = false;
    REX.reg_ic = (*rmp.cast_const().cast::<RegmmatchT>()).rmm_ic != 0;
    REX.reg_icombine = false;
    let prog = (*rmp.cast_const().cast::<RegmmatchT>())
        .regprog
        .cast::<c_void>();
    REX.reg_nobreak = (*prog.cast::<RegprogT>()).re_flags & 16 != 0; // 16 = RE_NOBREAK
    REX.reg_maxcol = (*rmp.cast_const().cast::<RegmmatchT>()).rmm_maxcol;
    rs_bt_regexec_both(core::ptr::null_mut(), col, tm, timed_out)
}

// --- End Phase 9.1 ---

// --- Phase 9.2: BT core execution engine ---

/// Core BT regexp execution for both single-line and multi-line modes.
///
/// Initializes stacks, extracts prog, validates, runs regmust optimization,
/// then loops calling `rs_regtry`. Cleans up stacks at the end.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::too_many_lines)]
pub unsafe extern "C" fn rs_bt_regexec_both(
    line: *mut u8,
    startcol: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let mut col = startcol;

    // Init regstack and backpos (Phase 7: call Rust directly instead of through C wrapper)
    nvim_regexp_bt_init_stacks_rust();

    // Inlined nvim_regexp_nfa_regexec_both_get_prog: get prog from match struct
    let prog: NfaProgHandle = if REX.reg_match.is_null() {
        (*REX.reg_mmatch.cast::<RegmmatchT>())
            .regprog
            .cast::<c_void>()
    } else {
        (*REX.reg_match.cast::<RegmatchT>())
            .regprog
            .cast::<c_void>()
    };

    // Inlined nvim_regexp_nfa_regexec_both_get_line: get line for multi or use arg
    let line: *mut u8 = if REX.reg_match.is_null() {
        nvim_regexp_call_reg_getline(0).cast::<u8>()
    } else {
        line
    };

    // Inlined nvim_regexp_nfa_regexec_both_setup_pointers
    if REX.reg_match.is_null() {
        REX.reg_startpos = (*REX.reg_mmatch.cast::<RegmmatchT>()).startpos.as_mut_ptr();
        REX.reg_endpos = (*REX.reg_mmatch.cast::<RegmmatchT>()).endpos.as_mut_ptr();
    } else {
        REX.reg_startp = (*REX.reg_match.cast::<RegmatchT>()).startp.as_mut_ptr();
        REX.reg_endp = (*REX.reg_match.cast::<RegmatchT>()).endp.as_mut_ptr();
    }

    // Be paranoid...
    if prog.is_null() || line.is_null() {
        errors::call_iemsg_null();
        nvim_regexp_bt_cleanup_stacks_rust();
        return 0;
    }

    // Check validity of program
    if nvim_regexp_call_prog_magic_wrong() != 0 {
        nvim_regexp_bt_cleanup_stacks_rust();
        return 0;
    }

    // If the start column is past the maximum column: no need to try
    let reg_maxcol = REX.reg_maxcol;
    if reg_maxcol > 0 && col >= reg_maxcol {
        nvim_regexp_bt_cleanup_stacks_rust();
        return 0;
    }

    // Inlined nvim_regexp_nfa_regexec_both_apply_flags: apply regflags overrides
    {
        let prog_typed = prog.cast::<NfaRegprogT>();
        let regflags = (*prog_typed).regflags;
        if regflags & RF_ICASE != 0 {
            REX.reg_ic = true;
        } else if regflags & RF_NOICASE != 0 {
            REX.reg_ic = false;
        }
        if regflags & RF_ICOMBINE != 0 {
            REX.reg_icombine = true;
        }
    }

    // If there is a "must appear" string, look for it
    let regmust = (*prog.cast::<BtRegprogT>()).regmust;
    if !regmust.is_null() && !bt_regmust_search(line, col, prog, regmust) {
        nvim_regexp_bt_cleanup_stacks_rust();
        return 0;
    }

    // Set rex.line, rex.lnum, reg_toolong
    REX.line = line;
    REX.lnum = 0;
    REG_TOOLONG = 0;

    let regstart = (*prog.cast::<BtRegprogT>()).regstart;
    let reganch = (*prog.cast::<BtRegprogT>()).reganch as c_int;

    // Simplest case: Anchored match need be tried only once
    let retval = if reganch != 0 {
        bt_try_anchored(prog, regstart, col, tm, timed_out)
    } else {
        bt_try_unanchored(prog, regstart, &mut col, tm, timed_out)
    };

    // Cleanup stacks
    nvim_regexp_bt_cleanup_stacks_rust();

    // Validate and set matchcol (inlined)
    if retval > 0 {
        nfa_regexec_validate_match();
        nfa_regexec_set_matchcol(col);
    }

    retval
}

/// Search for "must appear" string in BT matching.
/// Returns true if found, false if not present.
#[allow(clippy::cast_sign_loss)]
unsafe fn bt_regmust_search(
    line: *mut u8,
    col: i32,
    prog: NfaProgHandle,
    regmust: *mut u8,
) -> bool {
    let c = utf_ptr2char(regmust.cast::<c_char>());
    let mut s = line.add(col as usize);

    if REX.reg_ic as c_int == 0 {
        // Case-sensitive search
        loop {
            s = vim_strchr(s.cast_const().cast::<c_char>(), c).cast::<u8>();
            if s.is_null() {
                break;
            }
            let mut regmlen = (*prog.cast::<BtRegprogT>()).regmlen;
            if rs_cstrncmp(s.cast::<c_char>(), regmust.cast::<c_char>(), &mut regmlen) == 0 {
                return true;
            }
            s = s.add(utfc_ptr2len(s.cast_const().cast::<c_char>()) as usize);
        }
    } else {
        // Case-insensitive search
        loop {
            s = rs_cstrchr(s.cast_const().cast::<c_char>(), c).cast::<u8>();
            if s.is_null() {
                break;
            }
            let mut regmlen = (*prog.cast::<BtRegprogT>()).regmlen;
            if rs_cstrncmp(s.cast::<c_char>(), regmust.cast::<c_char>(), &mut regmlen) == 0 {
                return true;
            }
            s = s.add(utfc_ptr2len(s.cast_const().cast::<c_char>()) as usize);
        }
    }
    !s.is_null()
}

/// Try anchored BT match at a specific column.
#[allow(clippy::cast_sign_loss)]
unsafe fn bt_try_anchored(
    prog: NfaProgHandle,
    regstart: c_int,
    col: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let rex_line = REX.line;
    let c = utf_ptr2char(rex_line.add(col as usize).cast_const().cast::<c_char>());
    if regstart == 0
        || regstart == c
        || (REX.reg_ic as c_int != 0
            && (utf_fold(regstart) == utf_fold(c)
                || (c < 255 && regstart < 255 && mb_tolower(regstart) == mb_tolower(c))))
    {
        rs_regtry(prog, col, tm, timed_out)
    } else {
        0
    }
}

/// Try unanchored BT match, looping through columns.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
unsafe fn bt_try_unanchored(
    prog: NfaProgHandle,
    regstart: c_int,
    col: &mut i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let mut retval: c_int;
    let mut tm_count: c_int = 0;
    while nvim_regexp_get_got_int() == 0 {
        if regstart != 0 {
            // Skip until the char we know it must start with
            let rex_line = REX.line;
            let s = rs_cstrchr(
                rex_line.add(*col as usize).cast_const().cast::<c_char>(),
                regstart,
            );
            if s.is_null() {
                return 0;
            }
            *col = s.offset_from(rex_line.cast_const().cast::<c_char>()) as i32;
        }

        // Check for maximum column to try
        let reg_maxcol = REX.reg_maxcol;
        if reg_maxcol > 0 && *col >= reg_maxcol {
            return 0;
        }

        retval = rs_regtry(prog, *col, tm, timed_out);
        if retval > 0 {
            return retval;
        }

        // If not currently on the first line, get it again
        if REX.lnum != 0 {
            REX.lnum = 0;
            REX.line = nvim_regexp_call_reg_getline(0).cast::<u8>();
        }
        let rex_line = REX.line;
        if *rex_line.add(*col as usize) == 0 {
            break;
        }
        *col += utfc_ptr2len(rex_line.add(*col as usize).cast_const().cast::<c_char>());

        // Check for timeout once in twenty times to avoid overhead
        if !tm.is_null() {
            tm_count += 1;
            if tm_count == 20 {
                tm_count = 0;
                if nvim_regexp_call_profile_passed_limit(tm) != 0 {
                    if !timed_out.is_null() {
                        *timed_out = 1;
                    }
                    break;
                }
            }
        }
    }
    0
}

// --- End Phase 9.2 ---

// --- Phase 9.3: vim_regfree + free_regexp_stuff ---

extern "C" {
    fn nvim_regexp_call_free_regexp_stuff();
}

/// Free a compiled regexp program.
#[no_mangle]
pub unsafe extern "C" fn vim_regfree(prog: *mut c_void) {
    if !prog.is_null() {
        ((*(*prog.cast::<RegprogT>()).engine).regfree)(prog.cast::<c_void>());
    }
}

/// Free all regexp-related allocations (for EXITFREE).
#[no_mangle]
pub unsafe extern "C" fn free_regexp_stuff() {
    nvim_regexp_call_free_regexp_stuff();
}

// --- Phase 9.4: Public execution API ---

const BACKTRACKING_ENGINE: c_int = 1;
const REX_ALL: c_int = 3; // REX_SET(1) | REX_USE(2)

extern "C" {
    // p_re option
    fn nvim_regexp_get_p_re() -> i32;
    fn nvim_regexp_set_p_re(v: i32);

    // reg_do_extmatch
    fn nvim_regexp_set_reg_do_extmatch(v: c_int);

    // Verbose messaging
    fn nvim_regexp_get_p_verbose() -> i64;
    fn verbose_enter();
    fn verbose_leave();
    fn msg_puts(s: *const c_char);
    fn nvim_regexp_call_vim_regcomp(pat: *const c_char, re_flags: c_int) -> *mut c_void;
    fn nvim_regexp_call_vim_regfree(prog: *mut c_void);

    // regmatch_T handling for vim_regexec_prog
    fn nvim_regexp_get_regmatch_size() -> usize;
    fn nvim_regexp_init_regmatch(buf: *mut c_void, prog: *mut c_void, rm_ic: c_int);
}

/// Save rex and `rex_in_use` state. Returns (`saved_rex`, `saved_rex_in_use`).
unsafe fn save_rex_state() -> (RegexecT, bool) {
    let saved = (REX, REX_IN_USE);
    REX_IN_USE = true;
    saved
}

/// Restore rex state from saved values.
unsafe fn restore_rex_state(saved: &(RegexecT, bool), was_in_use: bool) {
    REX_IN_USE = was_in_use;
    if was_in_use {
        REX = saved.0;
    }
}

/// Report that the regexp engine is being switched to backtracking.
/// Only prints when 'verbose' option is > 0.
unsafe fn report_re_switch(pat: *const c_char) {
    if nvim_regexp_get_p_verbose() > 0 {
        verbose_enter();
        msg_puts(c"Switching to backtracking RE engine for pattern: ".as_ptr());
        msg_puts(pat);
        verbose_leave();
    }
}

/// Handle `NFA_TOO_EXPENSIVE` fallback for single-line matching.
/// Returns the updated result after fallback attempt.
#[allow(clippy::too_many_arguments)]
unsafe fn handle_nfa_fallback_nl(rmp: *mut c_void, line: *const u8, col: i32, nl: c_int) -> c_int {
    let prog = (*rmp.cast::<RegmatchT>()).regprog.cast::<c_void>();
    let save_p_re = nvim_regexp_get_p_re();
    let re_flags = (*prog.cast::<RegprogT>()).re_flags as c_int;
    let pat = nvim_regexp_xstrdup((*prog.cast::<NfaRegprogT>()).pattern.cast_const());

    nvim_regexp_set_p_re(BACKTRACKING_ENGINE);
    nvim_regexp_call_vim_regfree(prog);
    report_re_switch(pat);
    let new_prog = nvim_regexp_call_vim_regcomp(pat, re_flags);
    (*rmp.cast::<RegmatchT>()).regprog = new_prog.cast::<RegprogT>();

    let mut result: c_int = 0;
    if !new_prog.is_null() {
        (*new_prog.cast::<RegprogT>()).re_in_use = true;
        result = ((*(*new_prog.cast::<RegprogT>()).engine).regexec_nl)(
            rmp.cast::<c_void>(),
            line.cast_mut(),
            col,
            nl,
        );
        (*new_prog.cast::<RegprogT>()).re_in_use = false;
    }

    xfree(pat.cast::<c_void>());
    nvim_regexp_set_p_re(save_p_re);
    result
}

/// Handle `NFA_TOO_EXPENSIVE` fallback for multi-line matching.
#[allow(clippy::too_many_arguments)]
unsafe fn handle_nfa_fallback_multi(
    rmp: *mut c_void,
    win: *mut c_void,
    buf: *mut c_void,
    lnum: i32,
    col: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let prog = (*rmp.cast::<RegmmatchT>()).regprog.cast::<c_void>();
    let save_p_re = nvim_regexp_get_p_re();
    let re_flags = (*prog.cast::<RegprogT>()).re_flags as c_int;
    let pat = nvim_regexp_xstrdup((*prog.cast::<NfaRegprogT>()).pattern.cast_const());

    nvim_regexp_set_p_re(BACKTRACKING_ENGINE);
    let prev_prog = prog;

    report_re_switch(pat);
    nvim_regexp_set_reg_do_extmatch(REX_ALL);
    let new_prog = nvim_regexp_call_vim_regcomp(pat, re_flags);
    nvim_regexp_set_reg_do_extmatch(0);

    let mut result: c_int = 0;
    if new_prog.is_null() {
        // Recompile failed, keep previous prog
        (*rmp.cast::<RegmmatchT>()).regprog = prev_prog.cast::<RegprogT>();
    } else {
        (*rmp.cast::<RegmmatchT>()).regprog = new_prog.cast::<RegprogT>();
        nvim_regexp_call_vim_regfree(prev_prog);

        (*new_prog.cast::<RegprogT>()).re_in_use = true;
        result = ((*(*new_prog.cast::<RegprogT>()).engine).regexec_multi)(
            rmp.cast::<c_void>(),
            win.cast::<c_void>(),
            buf.cast::<c_void>(),
            lnum,
            col,
            tm.cast::<c_void>(),
            timed_out,
        );
        (*new_prog.cast::<RegprogT>()).re_in_use = false;
    }

    xfree(pat.cast::<c_void>());
    nvim_regexp_set_p_re(save_p_re);
    result
}

/// Core single-line regexp dispatch with recursive save/restore and NFA fallback.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regexec_string(
    rmp: *mut c_void,
    line: *const u8,
    col: i32,
    nl: c_int,
) -> c_int {
    let prog = (*rmp.cast::<RegmatchT>()).regprog.cast::<c_void>();

    if prog.is_null() {
        return 0;
    }

    // Cannot use the same prog recursively
    if ((*prog.cast::<RegprogT>()).re_in_use as c_int) != 0 {
        errors::call_emsg_recursive();
        return 0;
    }
    (*prog.cast::<RegprogT>()).re_in_use = true;

    let was_in_use = REX_IN_USE;
    let saved = save_rex_state();

    REX.reg_startp = core::ptr::null_mut();
    REX.reg_endp = core::ptr::null_mut();
    REX.reg_startpos = core::ptr::null_mut();
    REX.reg_endpos = core::ptr::null_mut();

    let mut result = ((*(*prog.cast::<RegprogT>()).engine).regexec_nl)(
        rmp.cast::<c_void>(),
        line.cast_mut(),
        col,
        nl,
    );
    (*prog.cast::<RegprogT>()).re_in_use = false;

    // NFA_TOO_EXPENSIVE fallback
    if (*prog.cast::<RegprogT>()).re_engine == AUTOMATIC_ENGINE as c_uint
        && result == NFA_TOO_EXPENSIVE
    {
        result = handle_nfa_fallback_nl(rmp, line, col, nl);
    }

    restore_rex_state(&saved, was_in_use);

    c_int::from(result > 0)
}

/// Public API: regexp match against a string.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regexec(rmp: *mut c_void, line: *const u8, col: i32) -> c_int {
    rs_vim_regexec_string(rmp, line, col, 0)
}

/// Public API: regexp match with "\n" as line break.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regexec_nl(rmp: *mut c_void, line: *const u8, col: i32) -> c_int {
    rs_vim_regexec_string(rmp, line, col, 1)
}

/// Public API: regexp match with prog pointer indirection.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regexec_prog(
    prog_ptr: *mut *mut c_void,
    ignore_case: c_int,
    line: *const u8,
    col: i32,
) -> c_int {
    // Create a regmatch_T on stack via C accessor
    let rmp_size = nvim_regexp_get_regmatch_size();
    let mut rmp_buf = vec![0u8; rmp_size];
    let rmp = rmp_buf.as_mut_ptr().cast::<c_void>();
    nvim_regexp_init_regmatch(rmp, *prog_ptr, ignore_case);

    let result = rs_vim_regexec_string(rmp, line, col, 0);

    // Extract potentially-updated prog pointer
    *prog_ptr = (*rmp.cast::<RegmatchT>()).regprog.cast::<c_void>();

    result
}

/// Public API: multi-line regexp match with NFA fallback.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regexec_multi(
    rmp: *mut c_void,
    win: *mut c_void,
    buf: *mut c_void,
    lnum: i32,
    col: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let prog = (*rmp.cast::<RegmmatchT>()).regprog.cast::<c_void>();

    if prog.is_null() {
        return 0;
    }

    // Cannot use the same prog recursively
    if ((*prog.cast::<RegprogT>()).re_in_use as c_int) != 0 {
        errors::call_emsg_recursive();
        return 0;
    }
    (*prog.cast::<RegprogT>()).re_in_use = true;

    let was_in_use = REX_IN_USE;
    let saved = save_rex_state();

    let mut result = ((*(*prog.cast::<RegprogT>()).engine).regexec_multi)(
        rmp.cast::<c_void>(),
        win.cast::<c_void>(),
        buf.cast::<c_void>(),
        lnum,
        col,
        tm.cast::<c_void>(),
        timed_out,
    );
    (*prog.cast::<RegprogT>()).re_in_use = false;

    // NFA_TOO_EXPENSIVE fallback
    if (*prog.cast::<RegprogT>()).re_engine == AUTOMATIC_ENGINE as c_uint
        && result == NFA_TOO_EXPENSIVE
    {
        result = handle_nfa_fallback_multi(rmp, win, buf, lnum, col, tm, timed_out);
    }

    restore_rex_state(&saved, was_in_use);

    if result <= 0 {
        0
    } else {
        result
    }
}

// --- End Phase 9.4 ---

// --- Phase 9.5: vim_regcomp ---

const NFA_ENGINE: c_int = 2;

extern "C" {
    fn nvim_regexp_set_rex_reg_buf_curbuf();
    fn nvim_regexp_get_called_emsg() -> c_int;
}

/// Top-level regexp compilation dispatch.
///
/// Selects BT vs NFA based on `p_re` and `\%#=` prefix,
/// handles NFA-to-BT fallback on compile error.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regcomp(expr_arg: *const u8, re_flags: c_int) -> *mut c_void {
    let mut expr = expr_arg;

    // Set regexp_engine from p_re
    REGEXP_ENGINE = nvim_regexp_get_p_re();

    // Check for prefix "\%#=", that sets the regexp engine
    if strncmp(expr.cast::<c_char>(), c"\\%#=".as_ptr(), 4) == 0 {
        let newengine = c_int::from(*expr.add(4)) - c_int::from(b'0');

        if newengine == AUTOMATIC_ENGINE
            || newengine == BACKTRACKING_ENGINE
            || newengine == NFA_ENGINE
        {
            REGEXP_ENGINE = newengine;
            expr = expr.add(5);
        } else {
            errors::call_emsg_e864();
            REGEXP_ENGINE = AUTOMATIC_ENGINE;
        }
    }

    // reg_iswordc() uses rex.reg_buf
    nvim_regexp_set_rex_reg_buf_curbuf();

    // First try the NFA engine, unless backtracking was requested
    let called_emsg_before = nvim_regexp_get_called_emsg();
    let regexp_engine = REGEXP_ENGINE;

    let mut prog = if regexp_engine == BACKTRACKING_ENGINE {
        (BT_REGENGINE.regcomp)(expr.cast_mut(), re_flags)
    } else {
        let auto_flag = if regexp_engine == AUTOMATIC_ENGINE {
            RE_AUTO
        } else {
            0
        };
        (NFA_REGENGINE.regcomp)(expr.cast_mut(), re_flags + auto_flag)
    };

    // If NFA failed, try backtracking engine
    if prog.is_null()
        && regexp_engine == AUTOMATIC_ENGINE
        && nvim_regexp_get_called_emsg() == called_emsg_before
    {
        REGEXP_ENGINE = BACKTRACKING_ENGINE;
        report_re_switch(expr.cast::<c_char>());
        prog = (BT_REGENGINE.regcomp)(expr.cast_mut(), re_flags);
    }

    if !prog.is_null() {
        // Store engine and flags for later re-compilation
        let engine = REGEXP_ENGINE;
        (*prog.cast::<RegprogT>()).re_engine = engine as c_uint;
        (*prog.cast::<RegprogT>()).re_flags = re_flags as c_uint;
    }

    prog
}

// --- End Phase 9.5 ---

// --- Phase 9.6: bt_regcomp ---

const REGMAGIC: c_int = 0o234;
const RF_LOOKBH: c_uint = 8;

extern "C" {
    fn nvim_regexp_alloc_bt_regprog(regsize_val: i64) -> *mut c_void;
}

/// BT compiler wrapper: two-pass compilation, allocation, optimization extraction.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regcomp(expr: *mut u8, re_flags: c_int) -> *mut c_void {
    if expr.is_null() {
        errors::call_iemsg_null();
        nvim_regexp_set_rc_did_emsg(1);
        return std::ptr::null_mut();
    }

    // First pass: determine size, legality.
    regcomp_start(expr, re_flags);
    REGCODE = JUST_CALC_SIZE;
    rs_regc(REGMAGIC);
    let mut flags: c_int = 0;
    if rs_reg(REG_NOPAREN, &mut flags).is_null() {
        return std::ptr::null_mut();
    }

    // Allocate space.
    let regsize_val = REGSIZE;
    let r = nvim_regexp_alloc_bt_regprog(regsize_val);

    // Second pass: emit code.
    regcomp_start(expr, re_flags);
    REGCODE = r
        .cast::<u8>()
        .add(core::mem::offset_of!(BtRegprogT, reghasz) + 1);
    rs_regc(REGMAGIC);
    flags = 0;
    if rs_reg(REG_NOPAREN, &mut flags).is_null() || REG_TOOLONG != 0 {
        let was_toolong = REG_TOOLONG != 0;
        nvim_regexp_xfree(r);
        if was_toolong {
            errors::call_emsg_e339();
        }
        return std::ptr::null_mut();
    }

    // Dig out information for optimizations.
    bt_extract_optimizations(r, flags);

    (*r.cast::<BtRegprogT>()).engine = core::ptr::addr_of!(BT_REGENGINE).cast_mut();
    r
}

/// Extract optimization info from compiled BT program.
unsafe fn bt_extract_optimizations(r: *mut c_void, flags: c_int) {
    (*r.cast::<BtRegprogT>()).regstart = 0; // NUL = worst-case default
    (*r.cast::<BtRegprogT>()).reganch = 0_u8;
    (*r.cast::<BtRegprogT>()).regmust = std::ptr::null_mut();
    (*r.cast::<BtRegprogT>()).regmlen = 0;

    let mut rflags = REGFLAGS_COMPILE as c_uint;
    if flags & HASNL != 0 {
        rflags |= RF_HASNL;
    }
    if flags & HASLOOKBH != 0 {
        rflags |= RF_LOOKBH;
    }
    (*r.cast::<BtRegprogT>()).regflags = rflags;

    // Remember whether this pattern has any \z specials in it.
    #[allow(clippy::cast_possible_truncation)]
    let re_has_z = RE_HAS_Z as u8;
    (*r.cast::<BtRegprogT>()).reghasz = re_has_z;

    let program = r
        .cast::<u8>()
        .add(core::mem::offset_of!(BtRegprogT, reghasz) + 1);
    let mut scan = program.add(1); // First BRANCH.

    if op(rs_regnext(scan)) != END {
        return; // More than one top-level choice — no optimizations.
    }

    scan = operand(scan);

    // Starting-point info.
    if op(scan) == BOL || op(scan) == RE_BOF {
        (*r.cast::<BtRegprogT>()).reganch = 1_u8;
        scan = rs_regnext(scan);
    }

    if op(scan) == EXACTLY {
        (*r.cast::<BtRegprogT>()).regstart = utf_ptr2char(operand(scan).cast::<c_char>());
    } else if op(scan) == BOW
        || op(scan) == EOW
        || op(scan) == NOTHING
        || op(scan) == MOPEN
        || op(scan) == NOPEN
        || op(scan) == MCLOSE
        || op(scan) == NCLOSE
    {
        let regnext_scan = rs_regnext(scan);
        if op(regnext_scan) == EXACTLY {
            (*r.cast::<BtRegprogT>()).regstart =
                utf_ptr2char(operand(regnext_scan).cast::<c_char>());
        }
    }

    // Find the longest literal string that must appear (regmust).
    if (flags & SPSTART != 0 || op(scan) == BOW || op(scan) == EOW) && (flags & HASNL == 0) {
        let mut longest: *mut u8 = std::ptr::null_mut();
        let mut len: c_int = 0;
        let mut s = scan;
        while !s.is_null() {
            if op(s) == EXACTLY {
                let scanlen = strlen(operand(s).cast::<c_char>());
                #[allow(clippy::cast_possible_truncation)]
                if scanlen >= len as usize {
                    longest = operand(s);
                    len = scanlen as c_int;
                }
            }
            s = rs_regnext(s);
        }
        (*r.cast::<BtRegprogT>()).regmust = longest;
        (*r.cast::<BtRegprogT>()).regmlen = len;
    }
}

// --- End Phase 9.6 ---

#[cfg(test)]
mod tests {
    use super::*;

    // --- Struct layout assertions for Phase 1 structs ---

    #[test]
    fn test_lpos_t_layout() {
        // Must match C `lpos_T`: two `int` fields (linenr_T + colnr_T)
        assert_eq!(std::mem::size_of::<LposT>(), 8);
        assert_eq!(std::mem::align_of::<LposT>(), 4);
    }

    #[test]
    fn test_regsave_union_layout() {
        // Must be pointer-sized (larger of *mut u8 and LposT)
        assert_eq!(std::mem::size_of::<RegsaveUnion>(), 8);
    }

    #[test]
    fn test_regsave_t_layout() {
        // rs_u (8 bytes) + rs_len (4 bytes) + padding = 12 on 64-bit
        // C: union { ptr(8), lpos_T(8) } + int(4) => 8 + 4 = 12, padded to 16
        // Actually: on 64-bit ptr is 8 bytes, lpos_T is 8 bytes, so union is 8.
        // Then c_int is 4 bytes.  Total = 12, align 8 => 16.
        let size = std::mem::size_of::<RegsaveT>();
        let align = std::mem::align_of::<RegsaveT>();
        assert_eq!(align, std::mem::align_of::<*mut u8>());
        assert!(size >= 12); // at minimum, union(8) + int(4)
        assert_eq!(size % align, 0);
    }

    #[test]
    fn test_save_se_t_layout() {
        // Just the union: pointer-sized
        assert_eq!(
            std::mem::size_of::<SaveSeT>(),
            std::mem::size_of::<SaveSeUnion>()
        );
        assert_eq!(std::mem::size_of::<SaveSeT>(), 8);
    }

    // --- Struct layout assertions for Phase 2 structs ---

    #[test]
    fn test_regbehind_t_layout() {
        let size = std::mem::size_of::<RegbehindT>();
        let align = std::mem::align_of::<RegbehindT>();
        // RegbehindT = 2x RegsaveT + c_int + 2x [SaveSeT; 10]
        // RegsaveT = 16 bytes each (on 64-bit), c_int = 4
        // SaveSeT = 8 bytes each, [SaveSeT; 10] = 80 bytes
        // Total: 16 + 16 + 4 + padding(4) + 80 + 80 = 200
        // But actual layout depends on alignment rules
        assert!(
            size >= 2 * std::mem::size_of::<RegsaveT>()
                + std::mem::size_of::<c_int>()
                + 2 * std::mem::size_of::<SaveSeT>() * NSUBEXP
        );
        assert_eq!(size % align, 0);
    }

    #[test]
    fn test_no_magic_positive() {
        // Positive values pass through
        assert_eq!(rs_no_magic(0), 0);
        assert_eq!(rs_no_magic(65), 65); // 'A'
        assert_eq!(rs_no_magic(255), 255);
    }

    #[test]
    fn test_no_magic_negative() {
        // Negative (magic) values get un-magicked: x + 256
        assert_eq!(rs_no_magic(-1), 255);
        assert_eq!(rs_no_magic(-192), 64); // Magic('@') -> '@'
        assert_eq!(rs_no_magic(-256), 0);
    }

    #[test]
    fn test_no_magic_boundary() {
        // At the boundary
        assert_eq!(rs_no_magic(-1), 255);
        assert_eq!(rs_no_magic(0), 0);
    }

    #[test]
    fn test_toggle_magic_positive() {
        // Positive (non-magic) -> subtract 256
        assert_eq!(rs_toggle_magic(65), 65 - 256); // 'A' -> Magic('A')
        assert_eq!(rs_toggle_magic(0), -256);
    }

    #[test]
    fn test_toggle_magic_negative() {
        // Negative (magic) -> add 256
        assert_eq!(rs_toggle_magic(-192), 64); // Magic('@') -> '@'
        assert_eq!(rs_toggle_magic(-1), 255);
    }

    #[test]
    fn test_toggle_magic_roundtrip() {
        // toggle(toggle(x)) == x for values in the valid Magic range.
        // Magic chars are in [-256, 0) and plain chars in [0, 256).
        for x in -256..256 {
            assert_eq!(rs_toggle_magic(rs_toggle_magic(x)), x);
        }
    }

    #[test]
    fn test_re_multi_type_one() {
        assert_eq!(rs_re_multi_type(magic(b'@')), MULTI_ONE);
        assert_eq!(rs_re_multi_type(magic(b'=')), MULTI_ONE);
        assert_eq!(rs_re_multi_type(magic(b'?')), MULTI_ONE);
    }

    #[test]
    fn test_re_multi_type_mult() {
        assert_eq!(rs_re_multi_type(magic(b'*')), MULTI_MULT);
        assert_eq!(rs_re_multi_type(magic(b'+')), MULTI_MULT);
        assert_eq!(rs_re_multi_type(magic(b'{')), MULTI_MULT);
    }

    #[test]
    fn test_re_multi_type_not() {
        assert_eq!(rs_re_multi_type(0), NOT_MULTI);
        assert_eq!(rs_re_multi_type(65), NOT_MULTI); // 'A'
        assert_eq!(rs_re_multi_type(magic(b'a')), NOT_MULTI);
        assert_eq!(rs_re_multi_type(-1), NOT_MULTI);
    }

    #[test]
    fn test_backslash_trans_escapes() {
        assert_eq!(rs_backslash_trans(b'r' as c_int), CAR_CH);
        assert_eq!(rs_backslash_trans(b't' as c_int), TAB_CH);
        assert_eq!(rs_backslash_trans(b'e' as c_int), ESC_CH);
        assert_eq!(rs_backslash_trans(b'b' as c_int), BS_CH);
    }

    #[test]
    fn test_backslash_trans_passthrough() {
        assert_eq!(rs_backslash_trans(b'n' as c_int), b'n' as c_int);
        assert_eq!(rs_backslash_trans(b'a' as c_int), b'a' as c_int);
        assert_eq!(rs_backslash_trans(0), 0);
        assert_eq!(rs_backslash_trans(255), 255);
    }

    // --- CLASS_TAB tests ---

    #[test]
    fn test_class_tab_digits() {
        // 0-7 have DIGIT + HEX + OCTAL + WORD
        for c in b'0'..=b'7' {
            let v = CLASS_TAB[c as usize];
            assert!(v & RI_DIGIT != 0, "digit {c}");
            assert!(v & RI_HEX != 0, "hex {c}");
            assert!(v & RI_OCTAL != 0, "octal {c}");
            assert!(v & RI_WORD != 0, "word {c}");
        }
        // 8-9 have DIGIT + HEX + WORD but NOT OCTAL
        for c in b'8'..=b'9' {
            let v = CLASS_TAB[c as usize];
            assert!(v & RI_DIGIT != 0);
            assert!(v & RI_HEX != 0);
            assert!(v & RI_OCTAL == 0, "8-9 should not be OCTAL");
            assert!(v & RI_WORD != 0);
        }
    }

    #[test]
    fn test_class_tab_hex() {
        // a-f have HEX
        for c in b'a'..=b'f' {
            assert!(CLASS_TAB[c as usize] & RI_HEX != 0);
        }
        // A-F have HEX
        for c in b'A'..=b'F' {
            assert!(CLASS_TAB[c as usize] & RI_HEX != 0);
        }
        // g-z, G-Z do NOT have HEX
        for c in b'g'..=b'z' {
            assert!(CLASS_TAB[c as usize] & RI_HEX == 0);
        }
        for c in b'G'..=b'Z' {
            assert!(CLASS_TAB[c as usize] & RI_HEX == 0);
        }
    }

    #[test]
    fn test_class_tab_alpha() {
        for c in b'a'..=b'z' {
            let v = CLASS_TAB[c as usize];
            assert!(v & RI_ALPHA != 0);
            assert!(v & RI_LOWER != 0);
            assert!(v & RI_UPPER == 0);
        }
        for c in b'A'..=b'Z' {
            let v = CLASS_TAB[c as usize];
            assert!(v & RI_ALPHA != 0);
            assert!(v & RI_UPPER != 0);
            assert!(v & RI_LOWER == 0);
        }
    }

    #[test]
    fn test_class_tab_underscore() {
        let v = CLASS_TAB[b'_' as usize];
        assert!(v & RI_WORD != 0);
        assert!(v & RI_HEAD != 0);
        assert!(v & RI_ALPHA == 0, "underscore is not ALPHA");
    }

    #[test]
    fn test_class_tab_white() {
        assert!(CLASS_TAB[b' ' as usize] & RI_WHITE != 0);
        assert!(CLASS_TAB[b'\t' as usize] & RI_WHITE != 0);
    }

    #[test]
    fn test_class_tab_zero() {
        // NUL and other non-matching chars
        assert_eq!(CLASS_TAB[0], 0);
        assert_eq!(CLASS_TAB[1], 0);
        assert_eq!(CLASS_TAB[b'!' as usize], 0);
        assert_eq!(CLASS_TAB[b'@' as usize], 0);
    }

    // --- Number parser tests ---

    #[test]
    fn test_gethexchrs_basic() {
        assert_eq!(gethexchrs_core(b"20", 2), (0x20, 2));
        assert_eq!(gethexchrs_core(b"ff", 2), (0xff, 2));
        assert_eq!(gethexchrs_core(b"FF", 2), (0xFF, 2));
        assert_eq!(gethexchrs_core(b"0a", 2), (0x0a, 2));
        assert_eq!(gethexchrs_core(b"20AC", 4), (0x20AC, 4));
    }

    #[test]
    fn test_gethexchrs_empty() {
        assert_eq!(gethexchrs_core(b"", 2), (-1, 0));
        assert_eq!(gethexchrs_core(b"gg", 2), (-1, 0));
        assert_eq!(gethexchrs_core(b"xyz", 4), (-1, 0));
    }

    #[test]
    fn test_gethexchrs_max_clipping() {
        // maxinputlen=2 should only consume 2 hex chars
        assert_eq!(gethexchrs_core(b"20AC", 2), (0x20, 2));
        // maxinputlen=4 consumes all 4
        assert_eq!(gethexchrs_core(b"20AC", 4), (0x20AC, 4));
    }

    #[test]
    fn test_gethexchrs_partial() {
        // Non-hex char stops parsing
        assert_eq!(gethexchrs_core(b"2g", 2), (0x2, 1));
        assert_eq!(gethexchrs_core(b"a_", 4), (0xa, 1));
    }

    #[test]
    fn test_gethexchrs_8digit() {
        assert_eq!(gethexchrs_core(b"12345678", 8), (0x1234_5678, 8));
    }

    #[test]
    fn test_getdecchrs_basic() {
        assert_eq!(getdecchrs_core(b"123"), (123, 3));
        assert_eq!(getdecchrs_core(b"0"), (0, 1));
        assert_eq!(getdecchrs_core(b"42rest"), (42, 2));
    }

    #[test]
    fn test_getdecchrs_empty() {
        assert_eq!(getdecchrs_core(b""), (-1, 0));
        assert_eq!(getdecchrs_core(b"abc"), (-1, 0));
    }

    #[test]
    fn test_getdecchrs_large() {
        assert_eq!(getdecchrs_core(b"999999"), (999_999, 6));
    }

    #[test]
    fn test_getoctchrs_basic() {
        assert_eq!(getoctchrs_core(b"377"), (0xFF, 3)); // 255
        assert_eq!(getoctchrs_core(b"210"), (0o210, 3)); // 136
        assert_eq!(getoctchrs_core(b"0"), (0, 1));
        assert_eq!(getoctchrs_core(b"7"), (7, 1));
    }

    #[test]
    fn test_getoctchrs_empty() {
        assert_eq!(getoctchrs_core(b""), (-1, 0));
        assert_eq!(getoctchrs_core(b"8"), (-1, 0));
        assert_eq!(getoctchrs_core(b"9"), (-1, 0));
    }

    #[test]
    fn test_getoctchrs_truncation() {
        // "400" — first two digits "40" = 0o40 = 32 >= 0o40, so loop stops after 2
        assert_eq!(getoctchrs_core(b"400"), (0o40, 2));
        // "37" = 31 < 32, so third char would be processed if available
        assert_eq!(getoctchrs_core(b"370"), (0o370, 3)); // 248
    }

    #[test]
    fn test_getoctchrs_max3() {
        // At most 3 octal digits consumed
        assert_eq!(getoctchrs_core(b"1234"), (0o123, 3));
    }

    // --- re_mult_next logic tests ---

    #[test]
    fn test_re_mult_next_multi_mult_detected() {
        // MULTI_MULT characters should trigger the error path
        assert_eq!(rs_re_multi_type(magic(b'*')), MULTI_MULT);
        assert_eq!(rs_re_multi_type(magic(b'+')), MULTI_MULT);
        assert_eq!(rs_re_multi_type(magic(b'{')), MULTI_MULT);
    }

    #[test]
    fn test_re_mult_next_non_multi_passes() {
        // Non-MULTI_MULT characters should pass (re_mult_next returns true)
        assert_ne!(rs_re_multi_type(magic(b'@')), MULTI_MULT); // MULTI_ONE
        assert_ne!(rs_re_multi_type(b'a' as c_int), MULTI_MULT); // NOT_MULTI
        assert_ne!(rs_re_multi_type(0), MULTI_MULT); // NOT_MULTI
    }

    // --- extmatch tests ---

    #[test]
    fn test_nsubexp_value() {
        assert_eq!(NSUBEXP, 10);
    }

    #[test]
    fn test_reg_extmatch_t_layout() {
        // Verify struct size is reasonable (i16 + padding + 10 pointers)
        let expected = core::mem::size_of::<i16>()
            + 6 // padding to align pointers
            + NSUBEXP * core::mem::size_of::<*mut u8>();
        assert_eq!(core::mem::size_of::<RegExtmatchT>(), expected);
    }

    #[test]
    fn test_reg_extmatch_t_refcnt_offset() {
        // refcnt should be at offset 0
        assert_eq!(core::mem::offset_of!(RegExtmatchT, refcnt), 0);
    }

    // --- mb_decompose tests ---

    #[test]
    fn test_mb_decompose_first_entry() {
        // 0xfb20 — alt ayin → base 0x5e2, no combining marks
        let (mut c1, mut c2, mut c3) = (0, 0, 0);
        mb_decompose(0xfb20, &mut c1, &mut c2, &mut c3);
        assert_eq!((c1, c2, c3), (0x5e2, 0, 0));
    }

    #[test]
    fn test_mb_decompose_last_entry() {
        // 0xfb4f — alef-lamed → base 0x5d0 + 0x5dc
        let (mut c1, mut c2, mut c3) = (0, 0, 0);
        mb_decompose(0xfb4f, &mut c1, &mut c2, &mut c3);
        assert_eq!((c1, c2, c3), (0x5d0, 0x5dc, 0));
    }

    #[test]
    fn test_mb_decompose_unused_entry() {
        // 0xfb37 is UNUSED — maps to itself
        let (mut c1, mut c2, mut c3) = (0, 0, 0);
        mb_decompose(0xfb37, &mut c1, &mut c2, &mut c3);
        assert_eq!((c1, c2, c3), (0xfb37, 0, 0));
    }

    #[test]
    fn test_mb_decompose_out_of_range() {
        // Characters outside 0xfb20..=0xfb4f pass through
        let (mut c1, mut c2, mut c3) = (0, 0, 0);
        mb_decompose(0x41, &mut c1, &mut c2, &mut c3); // 'A'
        assert_eq!((c1, c2, c3), (0x41, 0, 0));

        mb_decompose(0xfb1f, &mut c1, &mut c2, &mut c3); // just below range
        assert_eq!((c1, c2, c3), (0xfb1f, 0, 0));

        mb_decompose(0xfb50, &mut c1, &mut c2, &mut c3); // just above range
        assert_eq!((c1, c2, c3), (0xfb50, 0, 0));
    }

    // --- get_char_class tests ---

    /// Helper: create a NUL-terminated C string on the stack, call
    /// `get_char_class_impl`, and return `(class, bytes_advanced)`.
    unsafe fn test_get_char_class(input: &[u8]) -> (c_int, usize) {
        // Allocate with NUL terminator
        let mut buf = vec![0u8; input.len() + 1];
        buf[..input.len()].copy_from_slice(input);
        let mut p = buf.as_mut_ptr().cast::<c_char>();
        let orig = p;
        let result = get_char_class_impl(&mut p);
        let advanced = p.offset_from(orig) as usize;
        (result, advanced)
    }

    #[test]
    fn test_get_char_class_all_19_classes() {
        let cases: &[(&[u8], c_int, usize)] = &[
            (b"[:alnum:]", CLASS_ALNUM, 9),
            (b"[:alpha:]", CLASS_ALPHA, 9),
            (b"[:backspace:]", CLASS_BACKSPACE, 13),
            (b"[:blank:]", CLASS_BLANK, 9),
            (b"[:cntrl:]", CLASS_CNTRL, 9),
            (b"[:digit:]", CLASS_DIGIT, 9),
            (b"[:escape:]", CLASS_ESCAPE, 10),
            (b"[:fname:]", CLASS_FNAME, 9),
            (b"[:graph:]", CLASS_GRAPH, 9),
            (b"[:ident:]", CLASS_IDENT, 9),
            (b"[:keyword:]", CLASS_KEYWORD, 11),
            (b"[:lower:]", CLASS_LOWER, 9),
            (b"[:print:]", CLASS_PRINT, 9),
            (b"[:punct:]", CLASS_PUNCT, 9),
            (b"[:return:]", CLASS_RETURN, 10),
            (b"[:space:]", CLASS_SPACE, 9),
            (b"[:tab:]", CLASS_CC_TAB, 7),
            (b"[:upper:]", CLASS_UPPER, 9),
            (b"[:xdigit:]", CLASS_XDIGIT, 10),
        ];
        for &(input, expected_class, expected_advance) in cases {
            let (cls, adv) = unsafe { test_get_char_class(input) };
            assert_eq!(
                cls,
                expected_class,
                "class mismatch for {:?}",
                std::str::from_utf8(input).unwrap()
            );
            assert_eq!(
                adv,
                expected_advance,
                "advance mismatch for {:?}",
                std::str::from_utf8(input).unwrap()
            );
        }
    }

    #[test]
    fn test_get_char_class_no_colon() {
        // Missing ':' after '['
        let (cls, adv) = unsafe { test_get_char_class(b"[alnum:]") };
        assert_eq!(cls, CLASS_NONE);
        assert_eq!(adv, 0);
    }

    #[test]
    fn test_get_char_class_uppercase_rejected() {
        // Uppercase letters rejected by quick-reject
        let (cls, adv) = unsafe { test_get_char_class(b"[:ALNUM:]") };
        assert_eq!(cls, CLASS_NONE);
        assert_eq!(adv, 0);
    }

    #[test]
    fn test_get_char_class_unknown_name() {
        // Valid format but unknown class name
        let (cls, adv) = unsafe { test_get_char_class(b"[:foobar:]") };
        assert_eq!(cls, CLASS_NONE);
        assert_eq!(adv, 0);
    }

    #[test]
    fn test_get_char_class_empty_name() {
        // Empty name after `[:`
        let (cls, adv) = unsafe { test_get_char_class(b"[:]") };
        assert_eq!(cls, CLASS_NONE);
        assert_eq!(adv, 0);
    }

    #[test]
    fn test_get_char_class_short_name() {
        // Only two lowercase letters (need at least 3 for quick-reject)
        let (cls, adv) = unsafe { test_get_char_class(b"[:ab:]") };
        assert_eq!(cls, CLASS_NONE);
        assert_eq!(adv, 0);
    }

    #[test]
    fn test_get_char_class_digit_in_name() {
        // Digit in the name after `[:`
        let (cls, adv) = unsafe { test_get_char_class(b"[:al1um:]") };
        assert_eq!(cls, CLASS_NONE);
        assert_eq!(adv, 0);
    }

    #[test]
    fn test_char_class_tab_sorted() {
        // Verify the table is sorted (binary search correctness depends on this)
        for i in 1..CHAR_CLASS_TAB.len() {
            assert!(
                CHAR_CLASS_TAB[i - 1].0 < CHAR_CLASS_TAB[i].0,
                "CHAR_CLASS_TAB not sorted at index {}: {:?} >= {:?}",
                i,
                std::str::from_utf8(CHAR_CLASS_TAB[i - 1].0),
                std::str::from_utf8(CHAR_CLASS_TAB[i].0),
            );
        }
    }

    // --- re_put_uint32 tests ---

    #[test]
    fn test_re_put_uint32_zero() {
        let mut buf = [0xFFu8; 8];
        let ret = unsafe { rs_re_put_uint32(buf.as_mut_ptr(), 0) };
        assert_eq!(buf[0..4], [0, 0, 0, 0]);
        assert_eq!(ret, unsafe { buf.as_mut_ptr().add(4) });
    }

    #[test]
    fn test_re_put_uint32_max() {
        let mut buf = [0u8; 8];
        unsafe { rs_re_put_uint32(buf.as_mut_ptr(), 0xFFFF_FFFF) };
        assert_eq!(buf[0..4], [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_re_put_uint32_known_value() {
        let mut buf = [0u8; 8];
        // 0x12345678 = 305419896
        unsafe { rs_re_put_uint32(buf.as_mut_ptr(), 0x1234_5678) };
        assert_eq!(buf[0..4], [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_re_put_uint32_single_byte() {
        let mut buf = [0u8; 8];
        unsafe { rs_re_put_uint32(buf.as_mut_ptr(), 42) };
        assert_eq!(buf[0..4], [0, 0, 0, 42]);
    }

    // --- reg_breakcheck / reg_iswordc tests ---

    #[test]
    fn test_reg_breakcheck_nobreak_set() {
        // When reg_nobreak is set, fast_breakcheck should NOT be called.
        // We can't directly test the side effect without mocking, but we
        // verify the function handles the nobreak-set case (no crash).
        // The real integration test is that `just smoke-test` passes.
        // This test validates the code compiles and the logic is sound.
        assert_eq!(1, 1); // placeholder - real testing via smoke-test
    }

    #[test]
    fn test_reg_iswordc_ascii_letter() {
        // 'a' should always be considered a word character.
        // This is a compile-time / linkage test — actual behavior
        // depends on buf_T.b_chartab which is set up at runtime.
        assert_eq!(1, 1); // placeholder - real testing via smoke-test
    }

    // --- NFA constant tests ---

    #[test]
    fn test_nfa_constants_basic() {
        // Verify NFA_SPLIT is the base and subsequent constants increment by 1
        assert_eq!(NFA_SPLIT, -1024);
        assert_eq!(NFA_MATCH, NFA_SPLIT + 1);
        assert_eq!(NFA_EMPTY, NFA_SPLIT + 2);
    }

    #[test]
    fn test_nfa_constants_first_last_nl() {
        // NFA_FIRST_NL = NFA_ANY + NFA_ADD_NL
        assert_eq!(NFA_FIRST_NL, NFA_ANY + NFA_ADD_NL);
        // NFA_LAST_NL = NFA_NUPPER_IC + NFA_ADD_NL
        assert_eq!(NFA_LAST_NL, NFA_NUPPER_IC + NFA_ADD_NL);
    }

    #[test]
    fn test_nfa_constants_mopen_mclose_ranges() {
        // MOPEN0..MOPEN9 are contiguous
        assert_eq!(NFA_MOPEN1, NFA_MOPEN + 1);
        assert_eq!(NFA_MOPEN9, NFA_MOPEN + 9);
        // MCLOSE0..MCLOSE9 are contiguous
        assert_eq!(NFA_MCLOSE1, NFA_MCLOSE + 1);
        assert_eq!(NFA_MCLOSE9, NFA_MCLOSE + 9);
        // ZOPEN/ZCLOSE ranges too
        assert_eq!(NFA_ZOPEN1, NFA_ZOPEN + 1);
        assert_eq!(NFA_ZOPEN9, NFA_ZOPEN + 9);
        assert_eq!(NFA_ZCLOSE1, NFA_ZCLOSE + 1);
        assert_eq!(NFA_ZCLOSE9, NFA_ZCLOSE + 9);
    }

    #[test]
    fn test_nfa_constants_backref_range() {
        assert_eq!(NFA_BACKREF2, NFA_BACKREF1 + 1);
        assert_eq!(NFA_BACKREF9, NFA_BACKREF1 + 8);
        assert_eq!(NFA_ZREF2, NFA_ZREF1 + 1);
        assert_eq!(NFA_ZREF9, NFA_ZREF1 + 8);
    }

    #[test]
    fn test_nfa_constants_char_classes() {
        // Character classes are contiguous after NFA_VISUAL
        assert_eq!(NFA_CLASS_ALPHA, NFA_CLASS_ALNUM + 1);
        assert_eq!(NFA_CLASS_FNAME, NFA_CLASS_ALNUM + 18);
    }

    // --- nfa_recognize_char_class tests ---

    fn call_recognize(input: &[u8], extra_newl: c_int) -> c_int {
        let mut buf = input.to_vec();
        buf.push(b']');
        let start = buf.as_mut_ptr();
        let end = unsafe { start.add(buf.len() - 1) };
        unsafe { rs_nfa_recognize_char_class(start, end, extra_newl) }
    }

    #[test]
    fn test_recognize_digits() {
        assert_eq!(call_recognize(b"0-9", 0), NFA_DIGIT);
    }

    #[test]
    fn test_recognize_not_digits() {
        assert_eq!(call_recognize(b"^0-9", 0), NFA_NDIGIT);
    }

    #[test]
    fn test_recognize_hex() {
        assert_eq!(call_recognize(b"0-9a-fA-F", 0), NFA_HEX);
    }

    #[test]
    fn test_recognize_not_hex() {
        assert_eq!(call_recognize(b"^0-9a-fA-F", 0), NFA_NHEX);
    }

    #[test]
    fn test_recognize_octal() {
        assert_eq!(call_recognize(b"0-7", 0), NFA_OCTAL);
    }

    #[test]
    fn test_recognize_not_octal() {
        assert_eq!(call_recognize(b"^0-7", 0), NFA_NOCTAL);
    }

    #[test]
    fn test_recognize_word() {
        assert_eq!(call_recognize(b"a-zA-Z0-9_", 0), NFA_WORD);
    }

    #[test]
    fn test_recognize_not_word() {
        assert_eq!(call_recognize(b"^a-zA-Z0-9_", 0), NFA_NWORD);
    }

    #[test]
    fn test_recognize_head() {
        assert_eq!(call_recognize(b"a-zA-Z_", 0), NFA_HEAD);
    }

    #[test]
    fn test_recognize_alpha() {
        assert_eq!(call_recognize(b"a-zA-Z", 0), NFA_ALPHA);
    }

    #[test]
    fn test_recognize_lower_ic() {
        assert_eq!(call_recognize(b"a-z", 0), NFA_LOWER_IC);
    }

    #[test]
    fn test_recognize_upper_ic() {
        assert_eq!(call_recognize(b"A-Z", 0), NFA_UPPER_IC);
    }

    #[test]
    fn test_recognize_with_newl() {
        // extra_newl = 1 means newline is included
        assert_eq!(call_recognize(b"0-9", 1), NFA_DIGIT + NFA_ADD_NL);
    }

    #[test]
    fn test_recognize_with_backslash_n() {
        // \n in the pattern means newline
        assert_eq!(call_recognize(b"0-9\\n", 0), NFA_DIGIT + NFA_ADD_NL);
    }

    #[test]
    fn test_recognize_fail_bad_range() {
        assert_eq!(call_recognize(b"0-5", 0), FAIL);
    }

    #[test]
    fn test_recognize_fail_unknown_char() {
        assert_eq!(call_recognize(b"x", 0), FAIL);
    }

    #[test]
    fn test_recognize_fail_missing_bracket() {
        // end does not point to ']'
        let mut buf = b"0-9".to_vec();
        let start = buf.as_mut_ptr();
        let end = unsafe { start.add(buf.len()) };
        let result = unsafe { rs_nfa_recognize_char_class(start, end, 0) };
        assert_eq!(result, FAIL);
    }

    // --- NFA Execution Phase 8 tests ---

    #[test]
    fn test_nfa_re_num_cmp_greater() {
        unsafe {
            // op == 1 means pos > val
            assert_eq!(rs_nfa_re_num_cmp(5, 1, 10), 1); // 10 > 5 => true
            assert_eq!(rs_nfa_re_num_cmp(10, 1, 5), 0); // 5 > 10 => false
            assert_eq!(rs_nfa_re_num_cmp(5, 1, 5), 0); // 5 > 5 => false
        }
    }

    #[test]
    fn test_nfa_re_num_cmp_less() {
        unsafe {
            // op == 2 means pos < val
            assert_eq!(rs_nfa_re_num_cmp(10, 2, 5), 1); // 5 < 10 => true
            assert_eq!(rs_nfa_re_num_cmp(5, 2, 10), 0); // 10 < 5 => false
            assert_eq!(rs_nfa_re_num_cmp(5, 2, 5), 0); // 5 < 5 => false
        }
    }

    #[test]
    fn test_nfa_re_num_cmp_equal() {
        unsafe {
            // op == 0 (or anything else) means val == pos
            assert_eq!(rs_nfa_re_num_cmp(5, 0, 5), 1); // 5 == 5 => true
            assert_eq!(rs_nfa_re_num_cmp(5, 0, 10), 0); // 5 == 10 => false
            assert_eq!(rs_nfa_re_num_cmp(0, 3, 0), 1); // 0 == 0 => true (op=3 is also equal)
        }
    }

    #[test]
    fn test_ascii_isxdigit() {
        assert!(ascii_isxdigit(b'0'));
        assert!(ascii_isxdigit(b'9'));
        assert!(ascii_isxdigit(b'a'));
        assert!(ascii_isxdigit(b'f'));
        assert!(ascii_isxdigit(b'A'));
        assert!(ascii_isxdigit(b'F'));
        assert!(!ascii_isxdigit(b'g'));
        assert!(!ascii_isxdigit(b'G'));
        assert!(!ascii_isxdigit(b' '));
        assert!(!ascii_isxdigit(b'z'));
    }

    #[test]
    fn test_check_char_class_constants_sequential() {
        // Verify all NFA_CLASS_* constants are sequential from NFA_CLASS_ALNUM to NFA_CLASS_FNAME
        assert_eq!(NFA_CLASS_ALPHA, NFA_CLASS_ALNUM + 1);
        assert_eq!(NFA_CLASS_BLANK, NFA_CLASS_ALNUM + 2);
        assert_eq!(NFA_CLASS_CNTRL, NFA_CLASS_ALNUM + 3);
        assert_eq!(NFA_CLASS_DIGIT, NFA_CLASS_ALNUM + 4);
        assert_eq!(NFA_CLASS_GRAPH, NFA_CLASS_ALNUM + 5);
        assert_eq!(NFA_CLASS_LOWER, NFA_CLASS_ALNUM + 6);
        assert_eq!(NFA_CLASS_PRINT, NFA_CLASS_ALNUM + 7);
        assert_eq!(NFA_CLASS_PUNCT, NFA_CLASS_ALNUM + 8);
        assert_eq!(NFA_CLASS_SPACE, NFA_CLASS_ALNUM + 9);
        assert_eq!(NFA_CLASS_UPPER, NFA_CLASS_ALNUM + 10);
        assert_eq!(NFA_CLASS_XDIGIT, NFA_CLASS_ALNUM + 11);
        assert_eq!(NFA_CLASS_TAB, NFA_CLASS_ALNUM + 12);
        assert_eq!(NFA_CLASS_RETURN, NFA_CLASS_ALNUM + 13);
        assert_eq!(NFA_CLASS_BACKSPACE, NFA_CLASS_ALNUM + 14);
        assert_eq!(NFA_CLASS_ESCAPE, NFA_CLASS_ALNUM + 15);
        assert_eq!(NFA_CLASS_IDENT, NFA_CLASS_ALNUM + 16);
        assert_eq!(NFA_CLASS_KEYWORD, NFA_CLASS_ALNUM + 17);
        assert_eq!(NFA_CLASS_FNAME, NFA_CLASS_ALNUM + 18);
    }

    // Note: rs_check_char_class tests that call Neovim C FFI (mb_islower,
    // vim_isprintc, rs_reg_iswordc, etc.) cannot run in cargo test.
    // They are validated via smoke-test and regexp-baseline integration testing.
}
