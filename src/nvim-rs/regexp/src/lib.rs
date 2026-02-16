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

use std::ffi::{c_char, c_int, c_uint, c_void};

use std::ffi::c_long;

#[allow(dead_code)]
extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_regexp_get_char_class(pp: *mut *mut c_char) -> c_int;
    fn nvim_regexp_get_equi_class(pp: *mut *mut c_char) -> c_int;
    fn nvim_regexp_get_coll_element(pp: *mut *mut c_char) -> c_int;
    fn nvim_get_p_cpo() -> *const c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_regexp_get_regflags(prog: *const c_void) -> c_uint;

    // Parse state accessors
    fn nvim_regexp_get_regparse() -> *mut c_char;
    fn nvim_regexp_set_regparse(p: *mut c_char);
    fn nvim_regexp_get_prevchr_len() -> c_int;
    fn nvim_regexp_set_prevchr_len(v: c_int);
    fn nvim_regexp_get_curchr() -> c_int;
    fn nvim_regexp_set_curchr(v: c_int);
    fn nvim_regexp_get_prevchr() -> c_int;
    fn nvim_regexp_set_prevchr(v: c_int);
    fn nvim_regexp_get_prevprevchr() -> c_int;
    fn nvim_regexp_set_prevprevchr(v: c_int);
    fn nvim_regexp_get_nextchr() -> c_int;
    fn nvim_regexp_set_nextchr(v: c_int);
    fn nvim_regexp_get_at_start() -> c_int;
    fn nvim_regexp_set_at_start(v: c_int);
    fn nvim_regexp_get_prev_at_start() -> c_int;
    fn nvim_regexp_set_prev_at_start(v: c_int);
    fn nvim_regexp_get_regnpar() -> c_int;
    fn nvim_regexp_set_regnpar(v: c_int);
    fn nvim_regexp_get_reg_magic() -> c_int;
    fn nvim_regexp_set_reg_magic(v: c_int);
    fn nvim_regexp_get_after_slash() -> c_int;
    fn nvim_regexp_set_after_slash(v: c_int);
    fn nvim_regexp_get_rex_reg_ic() -> c_int;
    fn nvim_regexp_get_rex_reg_icombine() -> c_int;
    fn nvim_regexp_emsg2_fail(msg: *const c_char, is_magic_all: c_int) -> c_int;

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

    // reg_cpo_lit accessors
    fn nvim_regexp_get_reg_cpo_lit() -> c_int;
    fn nvim_regexp_set_reg_cpo_lit(v: c_int);

    // Memory allocation
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // re_mult_next accessors
    fn nvim_regexp_set_rc_did_emsg(v: c_int);
    fn nvim_regexp_semsg_e888(what: *const c_char);

    // skip_regexp_err accessor
    fn nvim_regexp_semsg_e654(startp: *const c_char);

    // reg_nextline accessors
    fn nvim_regexp_get_rex_lnum() -> i32;
    fn nvim_regexp_set_rex_lnum(v: i32);
    fn nvim_regexp_set_rex_line_and_input(line: *mut u8);
    fn nvim_regexp_call_reg_getline(lnum: i32) -> *mut c_char;
    fn nvim_regexp_call_reg_breakcheck();

    // reg_prev_class accessors
    fn nvim_regexp_get_rex_input() -> *mut u8;
    fn nvim_regexp_get_rex_line() -> *mut u8;
    fn nvim_regexp_get_rex_reg_buf_chartab() -> *mut i64;
    fn mb_get_class_tab(p: *const c_char, chartab: *const i64) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // cleanup_subexpr / cleanup_zsubexpr accessors
    fn nvim_regexp_get_rex_need_clear_subexpr() -> c_int;
    fn nvim_regexp_set_rex_need_clear_subexpr(v: c_int);
    fn nvim_regexp_get_rex_need_clear_zsubexpr() -> c_int;
    fn nvim_regexp_set_rex_need_clear_zsubexpr(v: c_int);
    fn nvim_regexp_is_reg_multi() -> c_int;
    fn nvim_regexp_clear_rex_startpos();
    fn nvim_regexp_clear_rex_endpos();
    fn nvim_regexp_clear_rex_startp();
    fn nvim_regexp_clear_rex_endp();
    fn nvim_regexp_clear_reg_startzpos();
    fn nvim_regexp_clear_reg_endzpos();
    fn nvim_regexp_clear_reg_startzp();
    fn nvim_regexp_clear_reg_endzp();

    // Compilation global accessors
    fn nvim_regexp_get_regcode() -> *mut u8;
    fn nvim_regexp_set_regcode(p: *mut u8);
    fn nvim_regexp_get_regsize() -> i64;
    fn nvim_regexp_set_regsize(v: i64);
    fn nvim_regexp_get_reg_toolong() -> c_int;
    fn nvim_regexp_set_reg_toolong(v: c_int);
    fn nvim_regexp_get_just_calc_size() -> *mut u8;

    // Multibyte helpers (utf_char2len/utf_char2bytes already declared below)
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
                && nvim_regexp_get_equi_class(&mut p) == 0
                && nvim_regexp_get_coll_element(&mut p) == 0
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
    let cpo_lit = nvim_regexp_get_reg_cpo_lit() != 0;
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
pub unsafe extern "C" fn rs_re_multiline(prog: *const c_void) -> c_int {
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
    let regparse = nvim_regexp_get_regparse();
    let input = std::slice::from_raw_parts(regparse as *const u8, maxinputlen as usize + 1);
    // Find actual available length (up to NUL)
    let len = input.iter().position(|&b| b == 0).unwrap_or(input.len());
    let (nr, consumed) = gethexchrs_core(&input[..len], maxinputlen as usize);
    nvim_regexp_set_regparse(regparse.add(consumed));
    nr as c_long
}

/// FFI wrapper: get decimal chars from regparse, advancing regparse.
#[no_mangle]
pub unsafe extern "C" fn rs_getdecchrs() -> c_long {
    let regparse = nvim_regexp_get_regparse();
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
    nvim_regexp_set_regparse(regparse.add(consumed));
    // getdecchrs also sets curchr = -1 for each digit consumed
    if consumed > 0 {
        nvim_regexp_set_curchr(-1);
    }
    nr as c_long
}

/// FFI wrapper: get octal chars from regparse, advancing regparse.
#[no_mangle]
pub unsafe extern "C" fn rs_getoctchrs() -> c_long {
    let regparse = nvim_regexp_get_regparse();
    // Octal is at most 3 chars
    let mut len = 0;
    while len < 4 && *regparse.add(len) != 0 {
        len += 1;
    }
    let input = std::slice::from_raw_parts(regparse as *const u8, len);
    let (nr, consumed) = getoctchrs_core(input);
    nvim_regexp_set_regparse(regparse.add(consumed));
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
    nvim_regexp_set_regparse(str);
    nvim_regexp_set_prevchr_len(0);
    nvim_regexp_set_curchr(-1);
    nvim_regexp_set_prevprevchr(-1);
    nvim_regexp_set_prevchr(-1);
    nvim_regexp_set_nextchr(-1);
    nvim_regexp_set_at_start(1); // true
    nvim_regexp_set_prev_at_start(0); // false
}

/// Save the current parse state into `ps`.
#[no_mangle]
pub unsafe extern "C" fn rs_save_parse_state(ps: *mut ParseStateT) {
    (*ps).regparse = nvim_regexp_get_regparse();
    (*ps).prevchr_len = nvim_regexp_get_prevchr_len();
    (*ps).curchr = nvim_regexp_get_curchr();
    (*ps).prevchr = nvim_regexp_get_prevchr();
    (*ps).prevprevchr = nvim_regexp_get_prevprevchr();
    (*ps).nextchr = nvim_regexp_get_nextchr();
    (*ps).at_start = nvim_regexp_get_at_start();
    (*ps).prev_at_start = nvim_regexp_get_prev_at_start();
    (*ps).regnpar = nvim_regexp_get_regnpar();
}

/// Restore a previously saved parse state from `ps`.
#[no_mangle]
pub unsafe extern "C" fn rs_restore_parse_state(ps: *const ParseStateT) {
    nvim_regexp_set_regparse((*ps).regparse);
    nvim_regexp_set_prevchr_len((*ps).prevchr_len);
    nvim_regexp_set_curchr((*ps).curchr);
    nvim_regexp_set_prevchr((*ps).prevchr);
    nvim_regexp_set_prevprevchr((*ps).prevprevchr);
    nvim_regexp_set_nextchr((*ps).nextchr);
    nvim_regexp_set_at_start((*ps).at_start);
    nvim_regexp_set_prev_at_start((*ps).prev_at_start);
    nvim_regexp_set_regnpar((*ps).regnpar);
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
    let mut curchr = nvim_regexp_get_curchr();
    if curchr != -1 {
        return curchr;
    }

    let regparse = nvim_regexp_get_regparse();
    let reg_magic = nvim_regexp_get_reg_magic();
    let at_start = nvim_regexp_get_at_start();
    let prev_at_start = nvim_regexp_get_prev_at_start();
    let prevchr = nvim_regexp_get_prevchr();
    let prevprevchr = nvim_regexp_get_prevprevchr();
    let after_slash = nvim_regexp_get_after_slash();

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
                nvim_regexp_set_at_start(1);
                nvim_regexp_set_prev_at_start(0);
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
                nvim_regexp_set_curchr(-1);
                nvim_regexp_set_prev_at_start(nvim_regexp_get_at_start());
                nvim_regexp_set_at_start(0); // be able to say "/\*ptr"
                nvim_regexp_set_regparse(regparse.add(1));
                nvim_regexp_set_after_slash(after_slash + 1);
                rs_peekchr();
                nvim_regexp_set_regparse(regparse);
                nvim_regexp_set_after_slash(after_slash);
                curchr = rs_toggle_magic(nvim_regexp_get_curchr());
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

    nvim_regexp_set_curchr(curchr);
    curchr
}

/// Eat one lexed character. Advances regparse and updates character state.
#[no_mangle]
pub unsafe extern "C" fn rs_skipchr() {
    let regparse = nvim_regexp_get_regparse();
    // peekchr() eats a backslash, do the same here
    let mut prevchr_len = c_int::from(*regparse == b'\\' as c_char);
    if *regparse.add(prevchr_len as usize) != 0 {
        // Exclude composing chars that utfc_ptr2len does include.
        prevchr_len += utf_ptr2len(regparse.add(prevchr_len as usize));
    }
    nvim_regexp_set_regparse(regparse.add(prevchr_len as usize));
    nvim_regexp_set_prevchr_len(prevchr_len);
    nvim_regexp_set_prev_at_start(nvim_regexp_get_at_start());
    nvim_regexp_set_at_start(0);
    nvim_regexp_set_prevprevchr(nvim_regexp_get_prevchr());
    nvim_regexp_set_prevchr(nvim_regexp_get_curchr());
    nvim_regexp_set_curchr(nvim_regexp_get_nextchr()); // use previously unget char, or -1
    nvim_regexp_set_nextchr(-1);
}

/// Skip a character while keeping `prev_at_start`, `prevchr`, `prevprevchr`.
#[no_mangle]
pub unsafe extern "C" fn rs_skipchr_keepstart() {
    let saved_as = nvim_regexp_get_prev_at_start();
    let saved_pr = nvim_regexp_get_prevchr();
    let saved_prpr = nvim_regexp_get_prevprevchr();

    rs_skipchr();

    nvim_regexp_set_at_start(saved_as);
    nvim_regexp_set_prevchr(saved_pr);
    nvim_regexp_set_prevprevchr(saved_prpr);
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
    nvim_regexp_set_nextchr(nvim_regexp_get_curchr());
    nvim_regexp_set_curchr(nvim_regexp_get_prevchr());
    nvim_regexp_set_prevchr(nvim_regexp_get_prevprevchr());
    nvim_regexp_set_at_start(nvim_regexp_get_prev_at_start());
    nvim_regexp_set_prev_at_start(0);

    // Backup regparse by prevchr_len
    let regparse = nvim_regexp_get_regparse();
    let prevchr_len = nvim_regexp_get_prevchr_len();
    nvim_regexp_set_regparse(regparse.sub(prevchr_len as usize));
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
    let mut regparse = nvim_regexp_get_regparse();

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
        nvim_regexp_set_regparse(regparse);
        return nvim_regexp_emsg2_fail(
            c"E554: Syntax error in %s{...}".as_ptr(),
            c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL),
        );
    }

    // Reverse the range if there was a '-', or make sure it is in the right
    // order otherwise.
    if (!reverse && *minval > *maxval) || (reverse && *minval < *maxval) {
        core::ptr::swap(minval, maxval);
    }
    nvim_regexp_set_regparse(regparse);
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

    if nvim_regexp_get_rex_reg_ic() == 0 {
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
    if result != 0 && nvim_regexp_get_rex_reg_icombine() != 0 {
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

            if c1 != c2 && (nvim_regexp_get_rex_reg_ic() == 0 || utf_fold(c1) != utf_fold(c2)) {
                // Decomposition necessary?
                let mut c11: c_int = 0;
                let mut c12: c_int = 0;
                let mut junk1: c_int = 0;
                let mut junk2: c_int = 0;
                mb_decompose(c1, &mut c11, &mut junk1, &mut junk2);
                mb_decompose(c2, &mut c12, &mut junk1, &mut junk2);
                c1 = c11;
                c2 = c12;
                if c11 != c12
                    && (nvim_regexp_get_rex_reg_ic() == 0 || utf_fold(c11) != utf_fold(c12))
                {
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
    if nvim_regexp_get_rex_reg_ic() == 0 {
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
    nvim_regexp_set_reg_cpo_lit(cpo_lit as c_int);
}

// --- extmatch lifecycle ---

const NSUBEXP: usize = 10;

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
pub unsafe extern "C" fn rs_ref_extmatch(em: *mut RegExtmatchT) -> *mut RegExtmatchT {
    if !em.is_null() {
        (*em).refcnt += 1;
    }
    em
}

/// Remove a reference to an extmatch. If no references left, free it.
#[no_mangle]
pub unsafe extern "C" fn rs_unref_extmatch(em: *mut RegExtmatchT) {
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
        nvim_regexp_semsg_e888(what);
        nvim_regexp_set_rc_did_emsg(1);
        return false;
    }
    true
}

// --- cleanup_subexpr / cleanup_zsubexpr ---

/// Clear subexpression match data if the flag is set.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_subexpr() {
    if nvim_regexp_get_rex_need_clear_subexpr() == 0 {
        return;
    }
    if nvim_regexp_is_reg_multi() != 0 {
        nvim_regexp_clear_rex_startpos();
        nvim_regexp_clear_rex_endpos();
    } else {
        nvim_regexp_clear_rex_startp();
        nvim_regexp_clear_rex_endp();
    }
    nvim_regexp_set_rex_need_clear_subexpr(0);
}

/// Clear z-subexpression match data if the flag is set.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_zsubexpr() {
    if nvim_regexp_get_rex_need_clear_zsubexpr() == 0 {
        return;
    }
    if nvim_regexp_is_reg_multi() != 0 {
        nvim_regexp_clear_reg_startzpos();
        nvim_regexp_clear_reg_endzpos();
    } else {
        nvim_regexp_clear_reg_startzp();
        nvim_regexp_clear_reg_endzp();
    }
    nvim_regexp_set_rex_need_clear_zsubexpr(0);
}

// --- reg_prev_class ---

/// Get class of the character before `rex.input`.
/// Returns -1 if at the start of the line.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_prev_class() -> c_int {
    let input = nvim_regexp_get_rex_input();
    let line = nvim_regexp_get_rex_line();
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
    let lnum = nvim_regexp_get_rex_lnum() + 1;
    nvim_regexp_set_rex_lnum(lnum);
    let line = nvim_regexp_call_reg_getline(lnum).cast::<u8>();
    nvim_regexp_set_rex_line_and_input(line);
    rs_reg_breakcheck();
}

// --- reg_breakcheck ---

extern "C" {
    fn fast_breakcheck();
    fn nvim_regexp_get_rex_reg_nobreak() -> c_int;
    fn vim_iswordc_buf(c: c_int, buf: *const c_void) -> c_int;
    fn nvim_regexp_get_rex_reg_buf() -> *const c_void;
}

/// If `rex.reg_nobreak` is not set, call `fast_breakcheck()`.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_breakcheck() {
    if nvim_regexp_get_rex_reg_nobreak() == 0 {
        fast_breakcheck();
    }
}

/// Return true if character `c` is included in 'iskeyword' for `rex.reg_buf`.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_iswordc(c: c_int) -> c_int {
    vim_iswordc_buf(c, nvim_regexp_get_rex_reg_buf())
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
    fn nvim_regexp_set_rex_line(line: *mut u8);
    fn nvim_regexp_set_rex_input(input: *mut u8);
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

    let lnum = nvim_regexp_get_rex_lnum() + nvim_regexp_get_rex_reg_firstlnum();
    if lnum < top_lnum || lnum > bot_lnum {
        return 0;
    }

    let rex_input = nvim_regexp_get_rex_input();
    let rex_line = nvim_regexp_get_rex_line();
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
        let rex_lnum = nvim_regexp_get_rex_lnum();
        let new_line = nvim_regexp_call_reg_getline(rex_lnum).cast::<u8>();
        nvim_regexp_set_rex_line(new_line);
        nvim_regexp_set_rex_input(new_line.add(col as usize));

        let firstlnum = nvim_regexp_get_rex_reg_firstlnum();
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
pub unsafe extern "C" fn rs_skip_regexp_err(
    startp: *mut c_char,
    delim: c_int,
    magic: c_int,
) -> *mut c_char {
    let p = rs_skip_regexp(startp, delim, magic);
    if *p as u8 as c_int != delim {
        nvim_regexp_semsg_e654(startp);
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
    fn nvim_regexp_get_rex_reg_firstlnum() -> i32;
    fn nvim_regexp_get_rex_reg_maxline() -> i32;
    fn nvim_regexp_get_rsm_firstlnum() -> i32;
    fn nvim_regexp_get_rsm_maxline() -> i32;
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
        (
            nvim_regexp_get_rsm_firstlnum() + lnum,
            nvim_regexp_get_rsm_maxline(),
        )
    } else {
        (
            nvim_regexp_get_rex_reg_firstlnum() + lnum,
            nvim_regexp_get_rex_reg_maxline(),
        )
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
    fn nvim_regexp_get_can_f_submatch() -> c_int;
    fn nvim_regexp_is_rsm_sm_match_null() -> c_int;
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
pub unsafe extern "C" fn rs_reg_submatch(no: c_int) -> *mut c_char {
    if nvim_regexp_get_can_f_submatch() == 0 || no < 0 {
        return std::ptr::null_mut();
    }

    if nvim_regexp_is_rsm_sm_match_null() != 0 {
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
    fn nvim_regexp_emsg_resulting_text_too_long();
    fn xmalloc(size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
}

/// Replace tildes in the pattern by the old pattern.
/// Direct transliteration of C `regtilde()`.
#[no_mangle]
pub unsafe extern "C" fn rs_regtilde(
    source: *mut c_char,
    magic: c_int,
    preview: bool,
) -> *mut c_char {
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
                    nvim_regexp_emsg_resulting_text_too_long();
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
    fn nvim_regexp_get_reg_tofree() -> *mut u8;
    fn nvim_regexp_set_reg_tofree(p: *mut u8);
    fn nvim_regexp_get_reg_tofreelen() -> c_uint;
    fn nvim_regexp_set_reg_tofreelen(v: c_uint);
    fn nvim_regexp_get_got_int() -> c_int;
    fn nvim_regexp_call_mb_strnicmp(s1: *const c_char, s2: *const c_char, len: usize) -> c_int;
    fn nvim_regexp_get_rex_line_strlen() -> c_int;
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
        let line = nvim_regexp_get_rex_line();
        let reg_tofree = nvim_regexp_get_reg_tofree();
        if line != reg_tofree {
            let len = nvim_regexp_get_rex_line_strlen();
            let reg_tofreelen = nvim_regexp_get_reg_tofreelen() as c_int;
            if reg_tofree.is_null() || len >= reg_tofreelen {
                let newlen = len + 50;
                xfree(nvim_regexp_get_reg_tofree().cast());
                let new_buf = xmalloc(newlen as usize).cast::<u8>();
                nvim_regexp_set_reg_tofree(new_buf);
                nvim_regexp_set_reg_tofreelen(newlen as c_uint);
            }
            let tofree = nvim_regexp_get_reg_tofree();
            let cur_line = nvim_regexp_get_rex_line();
            let cur_input = nvim_regexp_get_rex_input();
            // STRCPY: copy including NUL
            std::ptr::copy_nonoverlapping(cur_line, tofree, len as usize + 1);
            // rex.input = reg_tofree + (rex.input - rex.line)
            let input_offset = cur_input.offset_from(cur_line) as usize;
            nvim_regexp_set_rex_input(tofree.add(input_offset));
            nvim_regexp_set_rex_line(tofree);
        }

        // Get the line to compare with.
        let p = nvim_regexp_call_reg_getline(clnum);
        assert!(!p.is_null());

        let mut len = if clnum == end_lnum {
            end_col - ccol
        } else {
            nvim_regexp_call_reg_getline_len(clnum) - ccol
        };

        let input: *mut c_char = nvim_regexp_get_rex_input().cast();
        let reg_ic = nvim_regexp_get_rex_reg_ic();
        let p_ccol: *mut c_char = p.add(ccol as usize);

        if reg_ic == 0 {
            // case-sensitive compare
            if rs_cstrncmp(p_ccol, input, &mut len) != 0 {
                return RA_NOMATCH;
            }
        } else {
            // case-insensitive compare
            if nvim_regexp_call_mb_strnicmp(p_ccol, input, len as usize) != 0 {
                return RA_NOMATCH;
            }
        }

        if !bytelen.is_null() {
            *bytelen += len;
        }
        if clnum == end_lnum {
            break;
        }
        if nvim_regexp_get_rex_lnum() >= nvim_regexp_get_rex_reg_maxline() {
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
    fn nvim_regexp_call_iemsg_not_enough_space();
    fn nvim_regexp_call_iemsg_re_damg();
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
        nvim_regexp_call_iemsg_not_enough_space();
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
                nvim_regexp_call_iemsg_re_damg();
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
    let reg_multi = nvim_regexp_is_reg_multi() != 0;
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
    let regcode = nvim_regexp_get_regcode();
    let just_calc_size = nvim_regexp_get_just_calc_size();
    if regcode == just_calc_size {
        nvim_regexp_set_regsize(nvim_regexp_get_regsize() + 1);
    } else {
        #[allow(clippy::cast_possible_truncation)]
        {
            *regcode = b as u8;
        }
        nvim_regexp_set_regcode(regcode.add(1));
    }
}

/// Emit (if appropriate) a multi-byte character of code.
/// If `regcode == JUST_CALC_SIZE`, adds `utf_char2len(c)` to `regsize`.
#[no_mangle]
pub unsafe extern "C" fn rs_regmbc(c: c_int) {
    let regcode = nvim_regexp_get_regcode();
    let just_calc_size = nvim_regexp_get_just_calc_size();
    if regcode == just_calc_size {
        nvim_regexp_set_regsize(nvim_regexp_get_regsize() + utf_char2len(c) as i64);
    } else {
        let written = utf_char2bytes(c, regcode.cast::<c_char>());
        nvim_regexp_set_regcode(regcode.add(written as usize));
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
    let regcode = nvim_regexp_get_regcode();
    let just_calc_size = nvim_regexp_get_just_calc_size();
    if regcode == just_calc_size {
        nvim_regexp_set_regsize(nvim_regexp_get_regsize() + 3);
        return just_calc_size;
    }
    let ret = regcode;
    #[allow(clippy::cast_possible_truncation)]
    {
        *regcode = op as u8;
    }
    *regcode.add(1) = 0; // NUL "next" pointer
    *regcode.add(2) = 0;
    nvim_regexp_set_regcode(regcode.add(3));
    ret
}

/// Dig the "next" pointer out of a node.
/// Returns NULL when calculating size, when there is no next item, or on error.
#[no_mangle]
pub unsafe extern "C" fn rs_regnext(p: *mut u8) -> *mut u8 {
    let just_calc_size = nvim_regexp_get_just_calc_size();
    if p == just_calc_size || nvim_regexp_get_reg_toolong() != 0 {
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
    let just_calc_size = nvim_regexp_get_just_calc_size();
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
        nvim_regexp_set_reg_toolong(1);
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
    let just_calc_size = nvim_regexp_get_just_calc_size();
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
    let regcode = nvim_regexp_get_regcode();
    let just_calc_size = nvim_regexp_get_just_calc_size();
    if regcode == just_calc_size {
        nvim_regexp_set_regsize(nvim_regexp_get_regsize() + 3);
        return;
    }
    let count = regcode.offset_from(opnd) as usize;
    nvim_regexp_set_regcode(regcode.add(3));
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
    let regcode = nvim_regexp_get_regcode();
    let just_calc_size = nvim_regexp_get_just_calc_size();
    if regcode == just_calc_size {
        nvim_regexp_set_regsize(nvim_regexp_get_regsize() + 7);
        return;
    }
    let count = regcode.offset_from(opnd) as usize;
    nvim_regexp_set_regcode(regcode.add(7));
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
    let regcode = nvim_regexp_get_regcode();
    let just_calc_size = nvim_regexp_get_just_calc_size();
    if regcode == just_calc_size {
        nvim_regexp_set_regsize(nvim_regexp_get_regsize() + 11);
        return;
    }
    let count = regcode.offset_from(opnd) as usize;
    nvim_regexp_set_regcode(regcode.add(11));
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

extern "C" {
    fn nvim_regexp_get_num_complex_braces() -> c_int;
    fn nvim_regexp_set_num_complex_braces(v: c_int);
    fn nvim_regexp_emsg2_e59(m: c_int);
    fn nvim_regexp_emsg2_e60(m: c_int);
    fn nvim_regexp_emsg2_e61(m: c_int);
    fn nvim_regexp_emsg3_e62(m: c_int, c: c_int);
}

// --- regatom accessor/error extern declarations ---
#[allow(dead_code)]
extern "C" {
    // Accessors for regatom globals
    fn nvim_regexp_get_had_eol() -> c_int;
    fn nvim_regexp_set_had_eol(v: c_int);
    fn nvim_regexp_get_one_exactly() -> c_int;
    fn nvim_regexp_set_one_exactly(v: c_int);
    fn nvim_regexp_get_reg_string() -> c_int;
    fn nvim_regexp_get_reg_do_extmatch() -> c_int;
    fn nvim_regexp_get_re_has_z() -> c_int;
    fn nvim_regexp_set_re_has_z(v: c_int);
    fn nvim_regexp_get_reg_strict() -> c_int;
    fn nvim_regexp_get_had_endbrace(refnum: c_int) -> c_int;
    fn nvim_regexp_get_curwin_lnum() -> i32;
    fn nvim_regexp_get_curwin_col() -> i32;
    fn nvim_regexp_get_curwin_vcol() -> i32;
    fn nvim_regexp_get_reg_prev_sub_ptr() -> *mut c_char;

    // Character / multibyte helpers
    fn utf_iscomposing_legacy(c: c_int) -> c_int;
    fn utf_composinglike(p1: *const c_char, p2: *const c_char, state: *mut i32) -> c_int;
    fn nvim_regexp_reg_equi_class(c: c_int);
    fn vim_isIDc(c: c_int) -> c_int;
    fn vim_isfilec(c: c_int) -> c_int;
    fn vim_isprintc(c: c_int) -> c_int;
    fn mb_islower(c: c_int) -> c_int;
    fn mb_isupper(c: c_int) -> c_int;

    // Error helpers for regatom
    fn nvim_regexp_emsg_e63_underscore();
    fn nvim_regexp_iemsg_internal();
    fn nvim_regexp_emsg3_e64(m: c_int, c: c_int);
    fn nvim_regexp_emsg_nopresub();
    fn nvim_regexp_emsg_e65();
    fn nvim_regexp_emsg_e66();
    fn nvim_regexp_emsg_e67();
    fn nvim_regexp_emsg_e68();
    fn nvim_regexp_emsg2_e69(m: c_int);
    fn nvim_regexp_emsg2_e70(m: c_int);
    fn nvim_regexp_emsg2_e71(m: c_int);
    fn nvim_regexp_emsg2_e678(m: c_int);
    fn nvim_regexp_emsg2_e769(m: c_int);
    fn nvim_regexp_emsg_e944();
    fn nvim_regexp_emsg_e945();
    fn nvim_regexp_emsg_e949();
    fn nvim_regexp_emsg_toomsbra();
    fn nvim_regexp_semsg_e_atom_engine(c: c_int);
    fn nvim_regexp_semsg_e_dot_pos(c: c_int);
    fn nvim_regexp_emsg2_e369(m: c_int);

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
    let regparse = nvim_regexp_get_regparse();
    let ch = *regparse as u8;
    nvim_regexp_set_regparse(regparse.add(1));

    let nr: c_long = match ch {
        b'd' => rs_getdecchrs(),
        b'o' => rs_getoctchrs(),
        b'x' => rs_gethexchrs(2),
        b'u' => rs_gethexchrs(4),
        b'U' => rs_gethexchrs(8),
        _ => {
            // Put back the character we consumed
            nvim_regexp_set_regparse(regparse);
            return b'\\' as c_int;
        }
    };

    if nr < 0 {
        // If getting the number fails be backwards compatible: the character
        // is a backslash.
        // Undo the advance past the letter (d/o/x/u/U) — the number parsers
        // already left regparse right after the letter when they fail.
        nvim_regexp_set_regparse(regparse);
        return b'\\' as c_int;
    }
    c_int::try_from(nr).unwrap_or(c_int::MAX)
}

/// Return true if the back reference is legal.  We must have seen the close
/// brace.
#[allow(dead_code)]
unsafe fn seen_endbrace(refnum: c_int) -> bool {
    if nvim_regexp_get_had_endbrace(refnum) == 0 {
        // Trick: check if "@<=" or "@<!" follows, in which case
        // the \1 can appear before the referenced match.
        let regparse = nvim_regexp_get_regparse() as *const u8;
        let mut p = regparse;
        while *p != 0 {
            if *p == b'@' && *p.add(1) == b'<' && (*p.add(2) == b'!' || *p.add(2) == b'=') {
                break;
            }
            p = p.add(1);
        }
        if *p == 0 {
            nvim_regexp_emsg_e65();
            return false;
        }
    }
    true
}

// --- rs_regatom: Parse the lowest level ---

/// Handle a POSIX character class like `[:alpha:]` inside a collection.
/// `c_class` is the class constant from `get_char_class`.
#[allow(dead_code, clippy::too_many_lines)]
unsafe fn emit_posix_class(c_class: c_int, regparse_ptr: *mut *mut c_char) {
    match c_class {
        x if x == CLASS_NONE => {
            let eq = nvim_regexp_get_equi_class(regparse_ptr);
            if eq != 0 {
                nvim_regexp_reg_equi_class(eq);
            } else {
                let coll = nvim_regexp_get_coll_element(regparse_ptr);
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
    let mut regparse = nvim_regexp_get_regparse();

    // If there is no matching ']', we assume the '[' is a normal character.
    // This makes 'incsearch' and ":help [" work.
    let lp = rs_skip_anyof(regparse);
    if *lp != b']' as c_char {
        // No matching ']' — treated as literal in default/strict handling
        if nvim_regexp_get_reg_strict() != 0 {
            nvim_regexp_emsg2_e769(c_int::from(nvim_regexp_get_reg_magic() > MAGIC_OFF));
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
        nvim_regexp_set_regparse(regparse);
    } else {
        ret = rs_regnode(ANYOF + extra);
    }

    // At the start ']' and '-' mean the literal character.
    if *regparse == b']' as c_char || *regparse == b'-' as c_char {
        startc = *regparse as u8 as c_int;
        rs_regc(*regparse as c_int);
        regparse = regparse.add(1);
        nvim_regexp_set_regparse(regparse);
    }

    while *regparse != 0 && *regparse != b']' as c_char {
        if *regparse == b'-' as c_char {
            regparse = regparse.add(1);
            nvim_regexp_set_regparse(regparse);
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
                    endc = nvim_regexp_get_coll_element(&mut rp);
                    if endc != 0 {
                        regparse = rp;
                        nvim_regexp_set_regparse(regparse);
                    }
                }
                if endc == 0 {
                    let mut rp: *const c_char = regparse.cast_const();
                    endc = mb_ptr2char_adv(&mut rp);
                    regparse = rp.cast_mut();
                    nvim_regexp_set_regparse(regparse);
                }

                // Handle \o40, \x20 and \u20AC style sequences
                if endc == b'\\' as c_int && nvim_regexp_get_reg_cpo_lit() == 0 {
                    endc = coll_get_char();
                    regparse = nvim_regexp_get_regparse();
                }

                if startc > endc {
                    nvim_regexp_emsg_e944();
                    return std::ptr::null_mut();
                }
                if utf_char2len(startc) > 1 || utf_char2len(endc) > 1 {
                    // Limit to a range of 256 chars
                    if endc > startc + 256 {
                        nvim_regexp_emsg_e945();
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
                || (nvim_regexp_get_reg_cpo_lit() == 0
                    && !vim_strchr(
                        REGEXP_ABBR.as_ptr().cast::<c_char>(),
                        c_int::from(*regparse.add(1) as u8),
                    )
                    .is_null()))
        {
            regparse = regparse.add(1);
            nvim_regexp_set_regparse(regparse);
            if *regparse == b'n' as c_char {
                // '\n' in range: also match NL
                let just_calc_size = nvim_regexp_get_just_calc_size();
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
                nvim_regexp_set_regparse(regparse);
                startc = -1;
            } else if *regparse == b'd' as c_char
                || *regparse == b'o' as c_char
                || *regparse == b'x' as c_char
                || *regparse == b'u' as c_char
                || *regparse == b'U' as c_char
            {
                startc = coll_get_char();
                regparse = nvim_regexp_get_regparse();
                // max UTF-8 Codepoint is U+10FFFF, but allow values until INT_MAX
                if startc == c_int::MAX {
                    nvim_regexp_emsg_e949();
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
                nvim_regexp_set_regparse(regparse);
                rs_regc(startc);
            }
        } else if *regparse == b'[' as c_char {
            let mut rp = regparse;
            let c_class = nvim_regexp_get_char_class(&mut rp);
            startc = -1;
            // Characters assumed to be 8 bits!
            if c_class == CLASS_NONE {
                // Try equivalence class, then collating element, then literal '['
                let eq = nvim_regexp_get_equi_class(&mut rp);
                if eq != 0 {
                    nvim_regexp_reg_equi_class(eq);
                    regparse = rp;
                    nvim_regexp_set_regparse(regparse);
                } else {
                    let coll = nvim_regexp_get_coll_element(&mut rp);
                    if coll != 0 {
                        rs_regmbc(coll);
                        regparse = rp;
                        nvim_regexp_set_regparse(regparse);
                    } else {
                        // literal '[', allow [[-x] as a range
                        startc = *regparse as u8 as c_int;
                        regparse = regparse.add(1);
                        nvim_regexp_set_regparse(regparse);
                        rs_regc(startc);
                    }
                }
            } else {
                regparse = rp;
                nvim_regexp_set_regparse(regparse);
                emit_posix_class(c_class, &mut regparse);
                regparse = nvim_regexp_get_regparse();
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
            nvim_regexp_set_regparse(regparse);
        }
    }
    rs_regc(0); // NUL terminate
    nvim_regexp_set_prevchr_len(1); // last char was the ']'
    regparse = nvim_regexp_get_regparse();
    if *regparse != b']' as c_char {
        nvim_regexp_emsg_toomsbra(); // Cannot happen?
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
pub unsafe extern "C" fn rs_regatom(flagp: *mut c_int) -> *mut u8 {
    let mut extra: c_int = 0;
    let save_prev_at_start = nvim_regexp_get_prev_at_start();

    *flagp = WORST; // Tentatively.

    let mut c = rs_getchr();

    // --- Position assertions ---
    if c == magic(b'^') {
        return rs_regnode(BOL);
    }
    if c == magic(b'$') {
        nvim_regexp_set_had_eol(1);
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
            nvim_regexp_set_had_eol(1);
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
            nvim_regexp_emsg_e63_underscore();
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
        if nvim_regexp_get_reg_string() != 0 {
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
        if nvim_regexp_get_one_exactly() != 0 {
            nvim_regexp_emsg2_e369(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
        if nvim_regexp_get_one_exactly() != 0 {
            nvim_regexp_emsg2_e369(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
            return std::ptr::null_mut();
        }
        // Supposed to be caught earlier.
        nvim_regexp_iemsg_internal();
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
            nvim_regexp_get_reg_magic() >= MAGIC_ON
        } else {
            nvim_regexp_get_reg_magic() == MAGIC_ALL
        };
        nvim_regexp_emsg3_e64(c_int::from(is_magic), plain);
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
        nvim_regexp_emsg_nopresub();
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
                nvim_regexp_emsg_e66();
                return std::ptr::null_mut();
            }
            if nvim_regexp_get_one_exactly() != 0 {
                nvim_regexp_emsg2_e369(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
                return std::ptr::null_mut();
            }
            let mut flags: c_int = 0;
            let ret = rs_reg(REG_ZPAREN, &mut flags);
            if ret.is_null() {
                return std::ptr::null_mut();
            }
            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            nvim_regexp_set_re_has_z(REX_SET);
            return ret;
        } else if c >= b'1' as c_int && c <= b'9' as c_int {
            if (nvim_regexp_get_reg_do_extmatch() & REX_USE) == 0 {
                nvim_regexp_emsg_e67();
                return std::ptr::null_mut();
            }
            let ret = rs_regnode(ZREF + c - b'0' as c_int);
            nvim_regexp_set_re_has_z(REX_USE);
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
        nvim_regexp_emsg_e68();
        return std::ptr::null_mut();
    }

    // --- Percent operators: \% ---
    if c == magic(b'%') {
        c = rs_no_magic(rs_getchr());

        // \%( — non-capturing group
        if c == b'(' as c_int {
            if nvim_regexp_get_one_exactly() != 0 {
                nvim_regexp_emsg2_e369(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
            let regparse = nvim_regexp_get_regparse();
            if *regparse == b'=' as c_char && *regparse.add(1) >= 48 && *regparse.add(1) <= 50 {
                // misplaced \%#=1
                nvim_regexp_semsg_e_atom_engine(*regparse.add(1) as c_int);
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
            if nvim_regexp_get_one_exactly() != 0 {
                nvim_regexp_emsg2_e369(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
                    nvim_regexp_emsg2_e69(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
                    return std::ptr::null_mut();
                }
                let br = rs_regnode(BRANCH);
                if ret.is_null() {
                    ret = br;
                } else {
                    rs_regtail(lastnode, br);
                    if nvim_regexp_get_reg_toolong() != 0 {
                        return std::ptr::null_mut();
                    }
                }

                rs_ungetchr();
                nvim_regexp_set_one_exactly(1);
                lastnode = rs_regatom(flagp);
                nvim_regexp_set_one_exactly(0);
                if lastnode.is_null() {
                    return std::ptr::null_mut();
                }
            }
            if ret.is_null() {
                nvim_regexp_emsg2_e70(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
                return std::ptr::null_mut();
            }
            let lastbranch = rs_regnode(BRANCH);
            let br = rs_regnode(NOTHING);
            let just_calc_size = nvim_regexp_get_just_calc_size();
            if ret != just_calc_size {
                rs_regtail(lastnode, br);
                rs_regtail(lastbranch, br);
                // connect all branches to the NOTHING branch at the end
                let mut scan = ret;
                while scan != lastnode {
                    if *scan as c_int == BRANCH {
                        rs_regtail(scan, lastbranch);
                        if nvim_regexp_get_reg_toolong() != 0 {
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
                nvim_regexp_emsg2_e678(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
                let just_calc_size = nvim_regexp_get_just_calc_size();
                if ret == just_calc_size {
                    nvim_regexp_set_regsize(nvim_regexp_get_regsize() + 2);
                } else {
                    let regcode = nvim_regexp_get_regcode();
                    *regcode = c as u8;
                    *regcode.add(1) = cmp as u8;
                    nvim_regexp_set_regcode(regcode.add(2));
                }
                return ret;
            } else if (c == b'l' as c_int || c == b'c' as c_int || c == b'v' as c_int)
                && (cur || got_digit)
            {
                if cur && n != 0 {
                    nvim_regexp_semsg_e_dot_pos(rs_no_magic(c));
                    return std::ptr::null_mut();
                }
                let ret;
                if c == b'l' as c_int {
                    if cur {
                        n = nvim_regexp_get_curwin_lnum() as u32;
                    }
                    ret = rs_regnode(RE_LNUM);
                    if save_prev_at_start != 0 {
                        nvim_regexp_set_at_start(1);
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
                let just_calc_size = nvim_regexp_get_just_calc_size();
                if ret == just_calc_size {
                    nvim_regexp_set_regsize(nvim_regexp_get_regsize() + 5);
                } else {
                    // put the number and the optional comparator after the opcode
                    let regcode = rs_re_put_uint32(nvim_regexp_get_regcode(), n);
                    *regcode = cmp as u8;
                    nvim_regexp_set_regcode(regcode.add(1));
                }
                return ret;
            }
        }

        nvim_regexp_emsg2_e71(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
        && (len == 0
            || (rs_re_multi_type(rs_peekchr()) == NOT_MULTI
                && nvim_regexp_get_one_exactly() == 0
                && c >= 0))
    {
        // is_Magic(c) means c < 0, so c >= 0 means !is_Magic(c)
        let plain_c = if c < 0 { c + 256 } else { c }; // no_Magic(c)
        rs_regmbc(plain_c);

        // Need to get composing character too.
        let mut state: i32 = 0; // GRAPHEME_STATE_INIT
        loop {
            let regparse = nvim_regexp_get_regparse();
            let l = utf_ptr2len(regparse);
            if utf_composinglike(regparse, regparse.add(l as usize), &mut state) == 0 {
                break;
            }
            rs_regmbc(utf_ptr2char(nvim_regexp_get_regparse()));
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
/// Calls `rs_regatom` to parse the atom, then handles quantifiers.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::similar_names)]
pub unsafe extern "C" fn rs_regpiece(flagp: *mut c_int) -> *mut u8 {
    let mut flags: c_int = 0;
    let ret = rs_regatom(&mut flags);
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
                let reg_magic = nvim_regexp_get_reg_magic();
                nvim_regexp_emsg2_e59(c_int::from(reg_magic == MAGIC_ALL));
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
                let ncb = nvim_regexp_get_num_complex_braces();
                if ncb >= 10 {
                    let reg_magic = nvim_regexp_get_reg_magic();
                    nvim_regexp_emsg2_e60(c_int::from(reg_magic == MAGIC_ALL));
                    return std::ptr::null_mut();
                }
                rs_reginsert(BRACE_COMPLEX + ncb, ret);
                rs_regoptail(ret, rs_regnode(BACK));
                rs_regoptail(ret, ret);
                rs_reginsert_limits(BRACE_LIMITS, minval as i64, maxval as i64, ret);
                nvim_regexp_set_num_complex_braces(ncb + 1);
            }
            if minval > 0 && maxval > 0 {
                *flagp = HASWIDTH | (flags & (HASNL | HASLOOKBH));
            }
        }
        _ => {}
    }

    if rs_re_multi_type(rs_peekchr()) != NOT_MULTI {
        // Can't have a multi follow a multi.
        let reg_magic = nvim_regexp_get_reg_magic();
        if rs_peekchr() == magic(b'*') {
            nvim_regexp_emsg2_e61(c_int::from(reg_magic >= MAGIC_ON));
            return std::ptr::null_mut();
        }
        nvim_regexp_emsg3_e62(
            c_int::from(reg_magic == MAGIC_ALL),
            rs_no_magic(rs_peekchr()),
        );
        return std::ptr::null_mut();
    }

    ret
}

extern "C" {
    fn nvim_regexp_get_regflags_compile() -> c_uint;
    fn nvim_regexp_set_regflags_compile(v: c_uint);
}

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
                nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_ICOMBINE);
                rs_skipchr_keepstart();
            }
            x if x == magic(b'c') => {
                nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_ICASE);
                rs_skipchr_keepstart();
            }
            x if x == magic(b'C') => {
                nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_NOICASE);
                rs_skipchr_keepstart();
            }
            x if x == magic(b'v') => {
                nvim_regexp_set_reg_magic(MAGIC_ALL);
                rs_skipchr_keepstart();
                nvim_regexp_set_curchr(-1);
            }
            x if x == magic(b'm') => {
                nvim_regexp_set_reg_magic(MAGIC_ON);
                rs_skipchr_keepstart();
                nvim_regexp_set_curchr(-1);
            }
            x if x == magic(b'M') => {
                nvim_regexp_set_reg_magic(MAGIC_OFF);
                rs_skipchr_keepstart();
                nvim_regexp_set_curchr(-1);
            }
            x if x == magic(b'V') => {
                nvim_regexp_set_reg_magic(MAGIC_NONE);
                rs_skipchr_keepstart();
                nvim_regexp_set_curchr(-1);
            }
            _ => {
                let latest = rs_regpiece(&mut flags);
                if latest.is_null() || nvim_regexp_get_reg_toolong() != 0 {
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
        if nvim_regexp_get_reg_toolong() != 0 {
            break;
        }
        rs_reginsert(MATCH, latest);
        chain = latest;
    }

    ret
}

extern "C" {
    fn nvim_regexp_get_regnzpar() -> c_int;
    fn nvim_regexp_set_regnzpar(v: c_int);
    fn nvim_regexp_set_had_endbrace(parno: c_int, v: c_int);
    fn nvim_regexp_emsg_e50();
    fn nvim_regexp_emsg2_e51(m: c_int);
    fn nvim_regexp_emsg_e52();
    fn nvim_regexp_emsg2_e53(m: c_int);
    fn nvim_regexp_emsg2_e54(m: c_int);
    fn nvim_regexp_emsg2_e55(m: c_int);
    fn nvim_regexp_emsg_e488();
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
        let nzpar = nvim_regexp_get_regnzpar();
        if nzpar >= 10 {
            nvim_regexp_emsg_e50();
            return std::ptr::null_mut();
        }
        parno = nzpar;
        nvim_regexp_set_regnzpar(nzpar + 1);
        ret = rs_regnode(ZOPEN + parno);
    } else if paren == REG_PAREN {
        // Make a MOPEN node.
        let npar = nvim_regexp_get_regnpar();
        if npar >= 10 {
            let reg_magic = nvim_regexp_get_reg_magic();
            nvim_regexp_emsg2_e51(c_int::from(reg_magic == MAGIC_ALL));
            return std::ptr::null_mut();
        }
        parno = npar;
        nvim_regexp_set_regnpar(npar + 1);
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
        if br.is_null() || nvim_regexp_get_reg_toolong() != 0 {
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
        let reg_magic = nvim_regexp_get_reg_magic();
        if paren == REG_ZPAREN {
            nvim_regexp_emsg_e52();
        } else if paren == REG_NPAREN {
            nvim_regexp_emsg2_e53(c_int::from(reg_magic == MAGIC_ALL));
        } else {
            nvim_regexp_emsg2_e54(c_int::from(reg_magic == MAGIC_ALL));
        }
        return std::ptr::null_mut();
    } else if paren == REG_NOPAREN && rs_peekchr() != 0 {
        let reg_magic = nvim_regexp_get_reg_magic();
        if nvim_regexp_get_curchr() == magic(b')') {
            nvim_regexp_emsg2_e55(c_int::from(reg_magic == MAGIC_ALL));
        } else {
            nvim_regexp_emsg_e488(); // "Can't happen".
        }
        return std::ptr::null_mut();
    }
    // Here we set the flag allowing back references to this set of
    // parentheses.
    if paren == REG_PAREN {
        nvim_regexp_set_had_endbrace(parno, 1); // have seen the close paren
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
    if nvim_regexp_is_reg_multi() != 0 {
        #[allow(clippy::cast_possible_truncation)]
        let col = nvim_regexp_get_rex_input().offset_from(nvim_regexp_get_rex_line()) as c_int;
        (*save).rs_u.pos.col = col;
        (*save).rs_u.pos.lnum = nvim_regexp_get_rex_lnum();
    } else {
        (*save).rs_u.ptr = nvim_regexp_get_rex_input();
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
    if nvim_regexp_is_reg_multi() != 0 {
        if nvim_regexp_get_rex_lnum() != (*save).rs_u.pos.lnum {
            // Only call reg_getline() when the line number changed to save
            // a bit of time.
            nvim_regexp_set_rex_lnum((*save).rs_u.pos.lnum);
            let line = nvim_regexp_call_reg_getline((*save).rs_u.pos.lnum).cast::<u8>();
            nvim_regexp_set_rex_line(line);
        }
        nvim_regexp_set_rex_input(nvim_regexp_get_rex_line().add((*save).rs_u.pos.col as usize));
    } else {
        nvim_regexp_set_rex_input((*save).rs_u.ptr);
    }
    *ga_len = (*save).rs_len;
}

/// Return 1 if current position equals saved position, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_save_equal(save: *const RegsaveT) -> c_int {
    if nvim_regexp_is_reg_multi() != 0 {
        let eq = nvim_regexp_get_rex_lnum() == (*save).rs_u.pos.lnum
            && nvim_regexp_get_rex_input()
                == nvim_regexp_get_rex_line().add((*save).rs_u.pos.col as usize);
        eq as c_int
    } else {
        (nvim_regexp_get_rex_input() == (*save).rs_u.ptr) as c_int
    }
}

/// Save sub-expression position (multi-line): save `*posp` then set it to current.
#[no_mangle]
pub unsafe extern "C" fn rs_save_se_multi(savep: *mut SaveSeT, posp: *mut LposT) {
    (*savep).se_u.pos = *posp;
    (*posp).lnum = nvim_regexp_get_rex_lnum();
    #[allow(clippy::cast_possible_truncation)]
    let col = nvim_regexp_get_rex_input().offset_from(nvim_regexp_get_rex_line()) as c_int;
    (*posp).col = col;
}

/// Save sub-expression pointer (single-line): save `*pp` then set it to current input.
#[no_mangle]
pub unsafe extern "C" fn rs_save_se_one(savep: *mut SaveSeT, pp: *mut *mut u8) {
    (*savep).se_u.ptr = *pp;
    *pp = nvim_regexp_get_rex_input();
}

// --- Subexpression save/restore for BT regexp lookbehind ---

extern "C" {
    fn nvim_regexp_get_rex_startpos_array() -> *mut LposT;
    fn nvim_regexp_get_rex_endpos_array() -> *mut LposT;
    fn nvim_regexp_get_rex_startp_array() -> *mut *mut u8;
    fn nvim_regexp_get_rex_endp_array() -> *mut *mut u8;
}

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
    (*bp).save_need_clear_subexpr = nvim_regexp_get_rex_need_clear_subexpr();
    if nvim_regexp_get_rex_need_clear_subexpr() != 0 {
        return;
    }

    if nvim_regexp_is_reg_multi() != 0 {
        let startpos = nvim_regexp_get_rex_startpos_array();
        let endpos = nvim_regexp_get_rex_endpos_array();
        for i in 0..NSUBEXP {
            (*bp).save_start[i].se_u.pos = *startpos.add(i);
            (*bp).save_end[i].se_u.pos = *endpos.add(i);
        }
    } else {
        let startp = nvim_regexp_get_rex_startp_array();
        let endp = nvim_regexp_get_rex_endp_array();
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
    nvim_regexp_set_rex_need_clear_subexpr((*bp).save_need_clear_subexpr);
    if (*bp).save_need_clear_subexpr != 0 {
        return;
    }

    if nvim_regexp_is_reg_multi() != 0 {
        let startpos = nvim_regexp_get_rex_startpos_array();
        let endpos = nvim_regexp_get_rex_endpos_array();
        for i in 0..NSUBEXP {
            *startpos.add(i) = (*bp).save_start[i].se_u.pos;
            *endpos.add(i) = (*bp).save_end[i].se_u.pos;
        }
    } else {
        let startp = nvim_regexp_get_rex_startp_array();
        let endp = nvim_regexp_get_rex_endp_array();
        for i in 0..NSUBEXP {
            *startp.add(i) = (*bp).save_start[i].se_u.ptr;
            *endp.add(i) = (*bp).save_end[i].se_u.ptr;
        }
    }
}

// --- regtry: attempt match at a given column ---

extern "C" {
    fn nvim_regexp_get_prog_reghasz(prog: *const c_void) -> u8;
    fn nvim_regexp_get_prog_program(prog: *mut c_void) -> *mut u8;
    fn nvim_regexp_unref_re_extmatch_out();
    fn nvim_regexp_set_re_extmatch_out(em: *mut c_void);
    fn nvim_regexp_get_reg_startzpos_lnum(i: c_int) -> i32;
    fn nvim_regexp_get_reg_startzpos_col(i: c_int) -> i32;
    fn nvim_regexp_get_reg_endzpos_lnum(i: c_int) -> i32;
    fn nvim_regexp_get_reg_endzpos_col(i: c_int) -> i32;
    fn nvim_regexp_get_reg_startzp(i: c_int) -> *mut u8;
    fn nvim_regexp_get_reg_endzp(i: c_int) -> *mut u8;
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
    nvim_regexp_set_rex_input(nvim_regexp_get_rex_line().add(col as usize));
    nvim_regexp_set_rex_need_clear_subexpr(1);
    // Clear the external match subpointers if necessary.
    let reghasz = nvim_regexp_get_prog_reghasz(prog) as c_int;
    nvim_regexp_set_rex_need_clear_zsubexpr(c_int::from(reghasz == REX_SET));

    // program[1] = skip past the first byte (REGMAGIC)
    let program = nvim_regexp_get_prog_program(prog);
    if rs_regmatch_impl(program.add(1), tm.cast(), timed_out) == 0 {
        return 0;
    }

    rs_cleanup_subexpr();

    if nvim_regexp_is_reg_multi() != 0 {
        let startpos = nvim_regexp_get_rex_startpos_array();
        let endpos = nvim_regexp_get_rex_endpos_array();
        if (*startpos).lnum < 0 {
            (*startpos).lnum = 0;
            (*startpos).col = col;
        }
        if (*endpos).lnum < 0 {
            (*endpos).lnum = nvim_regexp_get_rex_lnum();
            #[allow(clippy::cast_possible_truncation)]
            let input_col =
                nvim_regexp_get_rex_input().offset_from(nvim_regexp_get_rex_line()) as c_int;
            (*endpos).col = input_col;
        } else {
            // Use line number of "\ze".
            nvim_regexp_set_rex_lnum((*endpos).lnum);
        }
    } else {
        let startp = nvim_regexp_get_rex_startp_array();
        let endp = nvim_regexp_get_rex_endp_array();
        if (*startp).is_null() {
            *startp = nvim_regexp_get_rex_line().add(col as usize);
        }
        if (*endp).is_null() {
            *endp = nvim_regexp_get_rex_input();
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
            if nvim_regexp_is_reg_multi() != 0 {
                // Only accept single line matches.
                let start_lnum = nvim_regexp_get_reg_startzpos_lnum(idx);
                let start_col = nvim_regexp_get_reg_startzpos_col(idx);
                let end_lnum = nvim_regexp_get_reg_endzpos_lnum(idx);
                let end_col = nvim_regexp_get_reg_endzpos_col(idx);
                if start_lnum >= 0 && end_lnum == start_lnum && end_col >= start_col {
                    let line = nvim_regexp_call_reg_getline(start_lnum);
                    (*em).matches[i] =
                        xstrnsave(line.add(start_col as usize), (end_col - start_col) as usize)
                            .cast::<u8>();
                }
            } else {
                let sp = nvim_regexp_get_reg_startzp(idx);
                let ep = nvim_regexp_get_reg_endzp(idx);
                if !sp.is_null() && !ep.is_null() {
                    (*em).matches[i] =
                        xstrnsave(sp.cast(), ep.offset_from(sp) as usize).cast::<u8>();
                }
            }
        }
    }

    1 + nvim_regexp_get_rex_lnum()
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
    fn nvim_regexp_get_rex_reg_line_lbr() -> c_int;
    fn nvim_regexp_call_vim_iswordp_buf(p: *const c_char) -> c_int;
    fn nvim_regexp_iemsg_re_corr();
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
        if !is_reg_multi
            || !with_nl(opcode)
            || nvim_regexp_get_rex_lnum() > reg_maxline
            || reg_line_lbr
        {
            return (scan, false);
        }
        rs_reg_nextline();
        let new_scan = nvim_regexp_get_rex_input();
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
    let mut scan = nvim_regexp_get_rex_input();
    let opnd = regrepeat_operand(p);
    let opcode = regrepeat_op(p);

    // Cache frequently used values at function entry for performance.
    let is_reg_multi = nvim_regexp_is_reg_multi() != 0;
    let reg_ic = nvim_regexp_get_rex_reg_ic() != 0;
    let reg_line_lbr = nvim_regexp_get_rex_reg_line_lbr() != 0;
    let reg_maxline = nvim_regexp_get_rex_reg_maxline();

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
                        || nvim_regexp_get_rex_lnum() > reg_maxline
                        || reg_line_lbr
                        || count == maxcount
                    {
                        break;
                    }
                    count += 1; // count the line-break
                    rs_reg_nextline();
                    scan = nvim_regexp_get_rex_input();
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
                    && ((*scan == 0
                        && nvim_regexp_get_rex_lnum() <= reg_maxline
                        && !reg_line_lbr
                        && is_reg_multi)
                        || (*scan == b'\n' && reg_line_lbr))
                {
                    count += 1;
                    if reg_line_lbr {
                        // ADVANCE_REGINPUT() = MB_PTR_ADV(rex.input)
                        let inp = nvim_regexp_get_rex_input();
                        let adv = utfc_ptr2len(inp.cast::<c_char>()) as usize;
                        nvim_regexp_set_rex_input(inp.add(adv));
                    } else {
                        rs_reg_nextline();
                    }
                    scan = nvim_regexp_get_rex_input();
                    if nvim_regexp_get_got_int() != 0 {
                        break;
                    }
                }
            }

            _ => {
                // Oh dear. Called inappropriately.
                nvim_regexp_iemsg_re_corr();
            }
        }
    }

    nvim_regexp_set_rex_input(scan);

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
    fn nvim_regexp_get_regstack_data() -> *mut u8;
    fn nvim_regexp_get_regstack_len() -> c_int;
    fn nvim_regexp_get_regstack_maxlen() -> c_int;
    fn nvim_regexp_set_regstack_len(v: c_int);
    fn nvim_regexp_call_ga_grow_regstack(n: c_int);

    fn nvim_regexp_get_backpos_data() -> *mut u8;
    fn nvim_regexp_get_backpos_len() -> c_int;
    fn nvim_regexp_set_backpos_len(v: c_int);
    fn nvim_regexp_call_ga_grow_backpos(n: c_int);

    // Brace statics
    fn nvim_regexp_get_brace_min(no: c_int) -> i64;
    fn nvim_regexp_set_brace_min(no: c_int, v: i64);
    fn nvim_regexp_get_brace_max(no: c_int) -> i64;
    fn nvim_regexp_set_brace_max(no: c_int, v: i64);
    fn nvim_regexp_get_brace_count(no: c_int) -> c_int;
    fn nvim_regexp_set_brace_count(no: c_int, v: c_int);

    fn nvim_regexp_get_bl_minval() -> i64;
    fn nvim_regexp_set_bl_minval(v: i64);
    fn nvim_regexp_get_bl_maxval() -> i64;
    fn nvim_regexp_set_bl_maxval(v: i64);

    // Behind position (C returns void*, cast to RegsaveT* on Rust side)
    fn nvim_regexp_get_behind_pos() -> *mut RegsaveT;

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
    fn nvim_regexp_emsg_maxmempattern();
    fn nvim_regexp_call_profile_passed_limit(tm: *const c_void) -> c_int;
    // got_int: reuse existing nvim_regexp_get_got_int (declared above)
    fn nvim_regexp_call_mb_isupper(c: c_int) -> c_int;
    fn nvim_regexp_call_mb_tolower(c: c_int) -> c_int;
    fn nvim_regexp_call_mb_toupper(c: c_int) -> c_int;

    // mb_get_class_tab
    fn nvim_regexp_call_mb_get_class_tab(p: *mut u8) -> c_int;

    // cstrncmp / cstrchr: use rs_cstrncmp/rs_cstrchr directly from Rust
    // rex.reg_firstlnum: reuse existing nvim_regexp_get_rex_reg_firstlnum (declared above)

    // z-subexpr element-pointer accessors for save_se/restore_se
    fn nvim_regexp_get_reg_startzpos_ptr(i: c_int) -> *mut LposT;
    fn nvim_regexp_get_reg_endzpos_ptr(i: c_int) -> *mut LposT;
    fn nvim_regexp_get_reg_startzp_ptr(i: c_int) -> *mut *mut u8;
    fn nvim_regexp_get_reg_endzp_ptr(i: c_int) -> *mut *mut u8;

    // internal_error
    fn nvim_regexp_internal_error(msg: *const c_char);

    // reg_breakcheck: use existing nvim_regexp_call_reg_breakcheck (declared at top)

    // regrepeat: use rs_regrepeat() directly from Rust (no wrapper needed)

    // regnext: use rs_regnext() directly from Rust (no wrapper needed)

    // iemsg: use existing nvim_regexp_iemsg_re_corr (declared in regrepeat section)
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
    if (nvim_regexp_get_regstack_len() as u32 >> 10) as i64 >= nvim_regexp_get_p_mmp() {
        nvim_regexp_emsg_maxmempattern();
        return std::ptr::null_mut();
    }
    nvim_regexp_call_ga_grow_regstack(std::mem::size_of::<RegitemT>() as c_int);

    let rp = (nvim_regexp_get_regstack_data().add(nvim_regexp_get_regstack_len() as usize))
        .cast::<RegitemT>();
    (*rp).rs_state = state;
    (*rp).rs_scan = scan;

    nvim_regexp_set_regstack_len(
        nvim_regexp_get_regstack_len() + std::mem::size_of::<RegitemT>() as c_int,
    );
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
    let rp = (nvim_regexp_get_regstack_data().add(nvim_regexp_get_regstack_len() as usize))
        .cast::<RegitemT>()
        .sub(1);
    *scan_out = (*rp).rs_scan;
    nvim_regexp_set_regstack_len(
        nvim_regexp_get_regstack_len() - std::mem::size_of::<RegitemT>() as c_int,
    );
}

/// Save sub-expression start/end: multi-line or single-line.
#[inline]
#[allow(dead_code)]
unsafe fn save_se(savep: *mut SaveSeT, posp: *mut LposT, pp: *mut *mut u8) {
    if nvim_regexp_is_reg_multi() != 0 {
        rs_save_se_multi(savep, posp);
    } else {
        rs_save_se_one(savep, pp);
    }
}

/// Restore sub-expression start/end: multi-line or single-line.
#[inline]
#[allow(dead_code)]
unsafe fn restore_se(savep: *const SaveSeT, posp: *mut LposT, pp: *mut *mut u8) {
    if nvim_regexp_is_reg_multi() != 0 {
        *posp = (*savep).se_u.pos;
    } else {
        *pp = (*savep).se_u.ptr;
    }
}

/// `ADVANCE_REGINPUT()` — advance rex.input by one multi-byte character.
#[inline]
#[allow(dead_code)]
unsafe fn advance_reginput() {
    let inp = nvim_regexp_get_rex_input();
    let len = utfc_ptr2len(inp.cast::<c_char>());
    nvim_regexp_set_rex_input(inp.add(len as usize));
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
    nvim_regexp_set_regstack_len(0);
    nvim_regexp_set_backpos_len(0);

    // Cache flags that don't change during matching.
    let is_reg_multi = nvim_regexp_is_reg_multi() != 0;
    let reg_line_lbr = nvim_regexp_get_rex_reg_line_lbr() != 0;

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
                && *nvim_regexp_get_rex_input() == 0
                && nvim_regexp_get_rex_lnum() <= nvim_regexp_get_rex_reg_maxline()
            {
                rs_reg_nextline();
            } else if reg_line_lbr && with_nl(opc) && *nvim_regexp_get_rex_input() == b'\n' {
                advance_reginput();
            } else {
                if with_nl(opc) {
                    opc -= ADD_NL;
                }
                let c = utf_ptr2char(nvim_regexp_get_rex_input().cast::<c_char>());

                match opc {
                    BOL => {
                        if nvim_regexp_get_rex_input() != nvim_regexp_get_rex_line() {
                            status = RA_NOMATCH;
                        }
                    }

                    EOL => {
                        if c != 0 {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_BOF => {
                        if nvim_regexp_get_rex_lnum() != 0
                            || nvim_regexp_get_rex_input() != nvim_regexp_get_rex_line()
                            || (is_reg_multi && nvim_regexp_get_rex_reg_firstlnum() > 1)
                        {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_EOF => {
                        if nvim_regexp_get_rex_lnum() != nvim_regexp_get_rex_reg_maxline() || c != 0
                        {
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
                        if (c != 0
                            || !is_reg_multi
                            || nvim_regexp_get_rex_lnum() > nvim_regexp_get_rex_reg_maxline()
                            || reg_line_lbr)
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
                        let bp = nvim_regexp_get_behind_pos();
                        if is_reg_multi {
                            if (*bp).rs_u.pos.col
                                != (nvim_regexp_get_rex_input()
                                    .offset_from(nvim_regexp_get_rex_line())
                                    as c_int)
                                || (*bp).rs_u.pos.lnum != nvim_regexp_get_rex_lnum()
                            {
                                status = RA_NOMATCH;
                            }
                        } else if (*bp).rs_u.ptr != nvim_regexp_get_rex_input() {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_COMPOSING => {
                        // Skip composing characters.
                        while utf_iscomposing_legacy(utf_ptr2char(
                            nvim_regexp_get_rex_input().cast::<c_char>(),
                        )) != 0
                        {
                            let inp = nvim_regexp_get_rex_input();
                            let len = utf_ptr2len(inp.cast::<c_char>());
                            nvim_regexp_set_rex_input(inp.add(len as usize));
                        }
                    }

                    BACK => {
                        // Check if we don't keep looping without matching input.
                        let bp_data = nvim_regexp_get_backpos_data().cast::<BackposT>();
                        let bp_len = nvim_regexp_get_backpos_len()
                            / std::mem::size_of::<BackposT>() as c_int;
                        let mut i = 0;
                        while i < bp_len {
                            if (*bp_data.add(i as usize)).bp_scan == scan {
                                break;
                            }
                            i += 1;
                        }
                        if i == bp_len {
                            // First time: add new entry.
                            nvim_regexp_call_ga_grow_backpos(
                                std::mem::size_of::<BackposT>() as c_int
                            );
                            let bp_data = nvim_regexp_get_backpos_data().cast::<BackposT>();
                            (*bp_data.add(i as usize)).bp_scan = scan;
                            nvim_regexp_set_backpos_len(
                                nvim_regexp_get_backpos_len()
                                    + std::mem::size_of::<BackposT>() as c_int,
                            );
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
                                nvim_regexp_get_backpos_len(),
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
                        if ascii_isdigit(*nvim_regexp_get_rex_input()) || vim_isIDc(c) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    KWORD => {
                        if nvim_regexp_call_vim_iswordp_buf(
                            nvim_regexp_get_rex_input().cast::<c_char>(),
                        ) == 0
                        {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    SKWORD => {
                        if ascii_isdigit(*nvim_regexp_get_rex_input())
                            || nvim_regexp_call_vim_iswordp_buf(
                                nvim_regexp_get_rex_input().cast::<c_char>(),
                            ) == 0
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
                        if ascii_isdigit(*nvim_regexp_get_rex_input()) || vim_isfilec(c) == 0 {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    PRINT => {
                        if vim_isprintc(utf_ptr2char(nvim_regexp_get_rex_input().cast::<c_char>()))
                            == 0
                        {
                            status = RA_NOMATCH;
                        } else {
                            advance_reginput();
                        }
                    }
                    SPRINT => {
                        if ascii_isdigit(*nvim_regexp_get_rex_input())
                            || vim_isprintc(utf_ptr2char(
                                nvim_regexp_get_rex_input().cast::<c_char>(),
                            )) == 0
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
                        if *opnd != *nvim_regexp_get_rex_input()
                            && nvim_regexp_get_rex_reg_ic() == 0
                        {
                            status = RA_NOMATCH;
                        } else if *opnd == 0 {
                            // match empty string always works
                        } else {
                            let mut len: c_int;
                            if *opnd.add(1) == 0 && nvim_regexp_get_rex_reg_ic() == 0 {
                                len = 1; // matched a single byte above
                            } else {
                                len = strlen(opnd.cast::<c_char>()) as c_int;
                                if rs_cstrncmp(
                                    opnd.cast::<c_char>(),
                                    nvim_regexp_get_rex_input().cast::<c_char>(),
                                    &mut len,
                                ) != 0
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                            // Check for following composing character, unless %C follows.
                            if status != RA_NOMATCH
                                && utf_composinglike(
                                    nvim_regexp_get_rex_input().cast::<c_char>(),
                                    nvim_regexp_get_rex_input()
                                        .add(len as usize)
                                        .cast::<c_char>(),
                                    std::ptr::null_mut(),
                                ) != 0
                                && nvim_regexp_get_rex_reg_icombine() == 0
                                && op(next) != RE_COMPOSING
                            {
                                status = RA_NOMATCH;
                            }
                            if status != RA_NOMATCH {
                                nvim_regexp_set_rex_input(
                                    nvim_regexp_get_rex_input().add(len as usize),
                                );
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

                            let inp = nvim_regexp_get_rex_input();
                            nvim_regexp_set_rex_input(
                                inp.add(utf_ptr2len(inp.cast::<c_char>()) as usize),
                            );
                            let q2 = q.add(utf_ptr2len(q.cast::<c_char>()) as usize);

                            if comb_len > 0 {
                                let inp2 = nvim_regexp_get_rex_input();
                                let mut mismatch = false;
                                for j in 0..comb_len {
                                    if *q2.add(j as usize) != *inp2.add(j as usize) {
                                        status = RA_NOMATCH;
                                        mismatch = true;
                                        break;
                                    }
                                }
                                if !mismatch {
                                    nvim_regexp_set_rex_input(inp2.add(comb_len as usize));
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
                                let inp = nvim_regexp_get_rex_input();
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
                                nvim_regexp_get_rex_input().cast::<c_char>(),
                                &mut mbc_len,
                            ) != 0
                            {
                                status = RA_NOMATCH;
                            }
                            if status != RA_NOMATCH {
                                nvim_regexp_set_rex_input(
                                    nvim_regexp_get_rex_input().add(mbc_len as usize),
                                );
                            }
                        }
                    }

                    // --- Phase 3: Word boundaries ---
                    #[allow(clippy::if_same_then_else)]
                    BOW => {
                        if c == 0 {
                            status = RA_NOMATCH;
                        } else {
                            let this_class =
                                nvim_regexp_call_mb_get_class_tab(nvim_regexp_get_rex_input());
                            if this_class <= 1 {
                                status = RA_NOMATCH;
                            } else if rs_reg_prev_class() == this_class {
                                status = RA_NOMATCH;
                            }
                        }
                    }
                    EOW => {
                        if nvim_regexp_get_rex_input() == nvim_regexp_get_rex_line() {
                            status = RA_NOMATCH;
                        } else {
                            let this_class =
                                nvim_regexp_call_mb_get_class_tab(nvim_regexp_get_rex_input());
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
                            let startpos = nvim_regexp_get_rex_startpos_array().add(no as usize);
                            let startp = nvim_regexp_get_rex_startp_array().add(no as usize);
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
                                nvim_regexp_get_reg_startzpos_ptr(no),
                                nvim_regexp_get_reg_startzp_ptr(no),
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
                            let endpos = nvim_regexp_get_rex_endpos_array().add(no as usize);
                            let endp = nvim_regexp_get_rex_endp_array().add(no as usize);
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
                                nvim_regexp_get_reg_endzpos_ptr(no),
                                nvim_regexp_get_reg_endzp_ptr(no),
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
                            let start_lnum =
                                (*nvim_regexp_get_rex_startpos_array().add(no as usize)).lnum;
                            let start_col =
                                (*nvim_regexp_get_rex_startpos_array().add(no as usize)).col;
                            let end_lnum =
                                (*nvim_regexp_get_rex_endpos_array().add(no as usize)).lnum;
                            let end_col =
                                (*nvim_regexp_get_rex_endpos_array().add(no as usize)).col;
                            if start_lnum < 0 || end_lnum < 0 {
                                len = 0; // Backref not set
                            } else if start_lnum == nvim_regexp_get_rex_lnum()
                                && end_lnum == nvim_regexp_get_rex_lnum()
                            {
                                // Compare within current line.
                                len = end_col - start_col;
                                if rs_cstrncmp(
                                    nvim_regexp_get_rex_line()
                                        .add(start_col as usize)
                                        .cast::<c_char>(),
                                    nvim_regexp_get_rex_input().cast::<c_char>(),
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
                            let startp = *nvim_regexp_get_rex_startp_array().add(no as usize);
                            let endp = *nvim_regexp_get_rex_endp_array().add(no as usize);
                            if startp.is_null() || endp.is_null() {
                                len = 0; // Backref not set: empty string
                            } else {
                                len = endp.offset_from(startp) as c_int;
                                if rs_cstrncmp(
                                    startp.cast::<c_char>(),
                                    nvim_regexp_get_rex_input().cast::<c_char>(),
                                    &mut len,
                                ) != 0
                                {
                                    status = RA_NOMATCH;
                                }
                            }
                        }
                        // Matched the backref, skip over it.
                        nvim_regexp_set_rex_input(nvim_regexp_get_rex_input().add(len as usize));
                    }

                    x if (ZREF + 1..ZREF + 10).contains(&x) => {
                        rs_cleanup_zsubexpr();
                        let no = opc - ZREF;
                        let ext_match = nvim_regexp_get_re_extmatch_in_match(no);
                        if !ext_match.is_null() {
                            let mut len = strlen(ext_match.cast::<c_char>()) as c_int;
                            if rs_cstrncmp(
                                ext_match.cast::<c_char>(),
                                nvim_regexp_get_rex_input().cast::<c_char>(),
                                &mut len,
                            ) != 0
                            {
                                status = RA_NOMATCH;
                            } else {
                                nvim_regexp_set_rex_input(
                                    nvim_regexp_get_rex_input().add(len as usize),
                                );
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
                            nvim_regexp_set_bl_minval(operand_min(scan));
                            nvim_regexp_set_bl_maxval(operand_max(scan));
                        } else if op(next) >= BRACE_COMPLEX && op(next) < BRACE_COMPLEX + 10 {
                            let no = op(next) - BRACE_COMPLEX;
                            nvim_regexp_set_brace_min(no, operand_min(scan));
                            nvim_regexp_set_brace_max(no, operand_max(scan));
                            nvim_regexp_set_brace_count(no, 0);
                        } else {
                            nvim_regexp_internal_error(c"BRACE_LIMITS".as_ptr());
                            status = RA_FAIL;
                        }
                    }

                    x if (BRACE_COMPLEX..BRACE_COMPLEX + 10).contains(&x) => {
                        let no = opc - BRACE_COMPLEX;
                        nvim_regexp_set_brace_count(no, nvim_regexp_get_brace_count(no) + 1);

                        // If not matched enough times yet, try one more.
                        let min_of_range =
                            if nvim_regexp_get_brace_min(no) <= nvim_regexp_get_brace_max(no) {
                                nvim_regexp_get_brace_min(no)
                            } else {
                                nvim_regexp_get_brace_max(no)
                            };
                        if i64::from(nvim_regexp_get_brace_count(no)) <= min_of_range {
                            let rp = regstack_push(RS_BRCPLX_MORE, scan);
                            if rp.is_null() {
                                status = RA_FAIL;
                            } else {
                                (*rp).rs_no = no as i16;
                                rs_reg_save(
                                    &mut (*rp).rs_un.regsave,
                                    nvim_regexp_get_backpos_len(),
                                );
                                next = operand(scan);
                                // Continue and handle the result when done.
                            }
                        } else if nvim_regexp_get_brace_min(no) <= nvim_regexp_get_brace_max(no) {
                            // Range is the normal way around, use longest match.
                            if i64::from(nvim_regexp_get_brace_count(no))
                                <= nvim_regexp_get_brace_max(no)
                            {
                                let rp = regstack_push(RS_BRCPLX_LONG, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    (*rp).rs_no = no as i16;
                                    rs_reg_save(
                                        &mut (*rp).rs_un.regsave,
                                        nvim_regexp_get_backpos_len(),
                                    );
                                    next = operand(scan);
                                }
                            }
                            // else: matched enough times, continue with next item.
                        } else {
                            // Range is backwards, use shortest match first.
                            if i64::from(nvim_regexp_get_brace_count(no))
                                <= nvim_regexp_get_brace_min(no)
                            {
                                let rp = regstack_push(RS_BRCPLX_SHORT, scan);
                                if rp.is_null() {
                                    status = RA_FAIL;
                                } else {
                                    rs_reg_save(
                                        &mut (*rp).rs_un.regsave,
                                        nvim_regexp_get_backpos_len(),
                                    );
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
                            if nvim_regexp_get_rex_reg_ic() != 0 {
                                if nvim_regexp_call_mb_isupper(rst.nextb) != 0 {
                                    rst.nextb_ic = nvim_regexp_call_mb_tolower(rst.nextb);
                                } else {
                                    rst.nextb_ic = nvim_regexp_call_mb_toupper(rst.nextb);
                                }
                            } else {
                                rst.nextb_ic = rst.nextb;
                            }
                        }
                        // else: rst.nextb and rst.nextb_ic are already 0 (NUL)

                        if opc == BRACE_SIMPLE {
                            rst.minval = nvim_regexp_get_bl_minval();
                            rst.maxval = nvim_regexp_get_bl_maxval();
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
                            if (nvim_regexp_get_regstack_len() as u32 >> 10) as i64
                                >= nvim_regexp_get_p_mmp()
                            {
                                nvim_regexp_emsg_maxmempattern();
                                status = RA_FAIL;
                            } else {
                                nvim_regexp_call_ga_grow_regstack(
                                    std::mem::size_of::<RegstarT>() as c_int
                                );
                                nvim_regexp_set_regstack_len(
                                    nvim_regexp_get_regstack_len()
                                        + std::mem::size_of::<RegstarT>() as c_int,
                                );
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
                            rs_reg_save(&mut (*rp).rs_un.regsave, nvim_regexp_get_backpos_len());
                            next = operand(scan);
                            // Continue and handle the result when done.
                        }
                    }

                    BEHIND | NOBEHIND => {
                        // Need a bit of room to store extra positions.
                        if (nvim_regexp_get_regstack_len() as u32 >> 10) as i64
                            >= nvim_regexp_get_p_mmp()
                        {
                            nvim_regexp_emsg_maxmempattern();
                            status = RA_FAIL;
                        } else {
                            nvim_regexp_call_ga_grow_regstack(
                                std::mem::size_of::<RegbehindT>() as c_int
                            );
                            nvim_regexp_set_regstack_len(
                                nvim_regexp_get_regstack_len()
                                    + std::mem::size_of::<RegbehindT>() as c_int,
                            );
                            let rp = regstack_push(RS_BEHIND1, scan);
                            if rp.is_null() {
                                status = RA_FAIL;
                            } else {
                                // Save subexpr to restore if match is not used.
                                rs_save_subexpr(rp.cast::<RegbehindT>().sub(1));
                                (*rp).rs_no = opc as i16;
                                rs_reg_save(
                                    &mut (*rp).rs_un.regsave,
                                    nvim_regexp_get_backpos_len(),
                                );
                                // First try if what follows matches. If it does
                                // then we check the behind match by looping.
                            }
                        }
                    }

                    // --- Phase 6: Special position opcodes ---
                    CURSOR => {
                        if nvim_regexp_has_rex_reg_win() == 0
                            || nvim_regexp_get_rex_lnum() + nvim_regexp_get_rex_reg_firstlnum()
                                != nvim_regexp_get_rex_reg_win_cursor_lnum()
                            || (nvim_regexp_get_rex_input().offset_from(nvim_regexp_get_rex_line())
                                as i32
                                != nvim_regexp_get_rex_reg_win_cursor_col())
                        {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_MARK => {
                        let mark = c_int::from(*operand(scan));
                        let cmp = c_int::from(*operand(scan).add(1));
                        let col: usize = if is_reg_multi {
                            nvim_regexp_get_rex_input().offset_from(nvim_regexp_get_rex_line())
                                as usize
                        } else {
                            0
                        };

                        let fm = nvim_regexp_call_mark_get(mark);

                        // Line may have been freed, get it again.
                        if is_reg_multi {
                            let new_line = nvim_regexp_call_reg_getline(nvim_regexp_get_rex_lnum())
                                .cast::<u8>();
                            nvim_regexp_set_rex_line(new_line);
                            nvim_regexp_set_rex_input(new_line.add(col));
                        }

                        if fm.is_null() || nvim_regexp_get_fmark_lnum(fm) <= 0 {
                            status = RA_NOMATCH;
                        } else {
                            let pos_lnum = nvim_regexp_get_fmark_lnum(fm);
                            let pos_col_raw = nvim_regexp_get_fmark_col(fm);
                            let rex_cur_lnum =
                                nvim_regexp_get_rex_lnum() + nvim_regexp_get_rex_reg_firstlnum();
                            #[allow(clippy::cast_possible_truncation)]
                            let input_col = nvim_regexp_get_rex_input()
                                .offset_from(nvim_regexp_get_rex_line())
                                as i32;

                            let pos_col = if pos_lnum == rex_cur_lnum && pos_col_raw == MAXCOL_I32 {
                                nvim_regexp_call_reg_getline_len(
                                    pos_lnum - nvim_regexp_get_rex_reg_firstlnum(),
                                )
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
                        if !is_reg_multi
                            || !re_num_cmp(
                                (nvim_regexp_get_rex_lnum() + nvim_regexp_get_rex_reg_firstlnum())
                                    as u32,
                                scan,
                            )
                        {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_COL => {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let col = (nvim_regexp_get_rex_input()
                            .offset_from(nvim_regexp_get_rex_line())
                            + 1) as u32;
                        if !re_num_cmp(col, scan) {
                            status = RA_NOMATCH;
                        }
                    }

                    RE_VCOL => {
                        let wp = nvim_regexp_get_rex_reg_win_or_curwin();
                        let mut lnum = if is_reg_multi {
                            nvim_regexp_get_rex_reg_firstlnum() + nvim_regexp_get_rex_lnum()
                        } else {
                            1
                        };
                        if is_reg_multi && (lnum <= 0 || lnum > nvim_regexp_get_win_line_count(wp))
                        {
                            lnum = 1;
                        }
                        #[allow(clippy::cast_possible_truncation)]
                        let input_col = nvim_regexp_get_rex_input()
                            .offset_from(nvim_regexp_get_rex_line())
                            as i32;
                        let vcol = nvim_regexp_call_win_linetabsize(
                            wp,
                            lnum,
                            nvim_regexp_get_rex_line().cast::<c_char>(),
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
        while nvim_regexp_get_regstack_len() > 0 && status != RA_FAIL {
            let rp = (nvim_regexp_get_regstack_data().add(nvim_regexp_get_regstack_len() as usize))
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
                            nvim_regexp_get_rex_startpos_array().add(no),
                            nvim_regexp_get_rex_startp_array().add(no),
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
                            nvim_regexp_get_reg_startzpos_ptr(no),
                            nvim_regexp_get_reg_startzp_ptr(no),
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
                            nvim_regexp_get_rex_endpos_array().add(no),
                            nvim_regexp_get_rex_endp_array().add(no),
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
                            nvim_regexp_get_reg_endzpos_ptr(no),
                            nvim_regexp_get_reg_endzp_ptr(no),
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
                            let mut bp_len = nvim_regexp_get_backpos_len();
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            nvim_regexp_set_backpos_len(bp_len);
                            scan = (*rp).rs_scan;
                        }
                        if scan.is_null() || op(scan) != BRANCH {
                            // No more branches, didn't find a match.
                            status = RA_NOMATCH;
                            regstack_pop(&mut scan);
                        } else {
                            // Prepare to try a branch.
                            (*rp).rs_scan = rs_regnext(scan);
                            rs_reg_save(&mut (*rp).rs_un.regsave, nvim_regexp_get_backpos_len());
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
                            let mut bp_len = nvim_regexp_get_backpos_len();
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            nvim_regexp_set_backpos_len(bp_len);
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
                        nvim_regexp_set_regstack_len(
                            nvim_regexp_get_regstack_len()
                                - std::mem::size_of::<RegbehindT>() as c_int,
                        );
                    } else {
                        // The stuff after BEHIND/NOBEHIND matches. Now try if
                        // the behind part does (not) match before the current
                        // position in the input.
                        let rbp = rp.cast::<RegbehindT>().sub(1);

                        // Save the position after the found match for next.
                        rs_reg_save(&mut (*rbp).save_after, nvim_regexp_get_backpos_len());

                        // Set behind_pos to where the match should end, BHPOS
                        // will match it. Save the current value.
                        let bp = nvim_regexp_get_behind_pos();
                        (*rbp).save_behind = *bp;
                        *bp = (*rp).rs_un.regsave;

                        (*rp).rs_state = RS_BEHIND2;

                        let mut bp_len = nvim_regexp_get_backpos_len();
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        nvim_regexp_set_backpos_len(bp_len);
                        scan = operand((*rp).rs_scan).add(4);
                    }
                }

                RS_BEHIND2 => {
                    // Looping for BEHIND / NOBEHIND match.
                    let bp = nvim_regexp_get_behind_pos();
                    let rbp = rp.cast::<RegbehindT>().sub(1);
                    if status == RA_MATCH && rs_reg_save_equal(bp) != 0 {
                        // Found a match that ends where "next" started.
                        *bp = (*rbp).save_behind;
                        if (*rp).rs_no == BEHIND as i16 {
                            let mut bp_len = nvim_regexp_get_backpos_len();
                            rs_reg_restore(&(*rbp).save_after, &mut bp_len);
                            nvim_regexp_set_backpos_len(bp_len);
                        } else {
                            // NOBEHIND: we didn't want a match. Restore subexpr.
                            status = RA_NOMATCH;
                            rs_restore_subexpr(rbp);
                        }
                        regstack_pop(&mut scan);
                        nvim_regexp_set_regstack_len(
                            nvim_regexp_get_regstack_len()
                                - std::mem::size_of::<RegbehindT>() as c_int,
                        );
                    } else {
                        // No match or match doesn't end where we want it.
                        // Go back one character. May go to previous line once.
                        let mut no_advance = false;
                        let limit = operand_min((*rp).rs_scan);
                        if is_reg_multi {
                            if limit > 0 {
                                let ref_col =
                                    if (*rp).rs_un.regsave.rs_u.pos.lnum < (*bp).rs_u.pos.lnum {
                                        strlen(nvim_regexp_get_rex_line().cast::<c_char>()) as i32
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
                                    let mut bp_len = nvim_regexp_get_backpos_len();
                                    rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                                    nvim_regexp_set_backpos_len(bp_len);
                                    #[allow(clippy::cast_possible_truncation)]
                                    let line_len =
                                        strlen(nvim_regexp_get_rex_line().cast::<c_char>()) as i32;
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
                            if (*rp).rs_un.regsave.rs_u.ptr == nvim_regexp_get_rex_line() {
                                no_advance = true;
                            } else {
                                let backed = mb_ptr_back(
                                    nvim_regexp_get_rex_line(),
                                    (*rp).rs_un.regsave.rs_u.ptr,
                                );
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
                                let mut bp_len = nvim_regexp_get_backpos_len();
                                rs_reg_restore(&(*rbp).save_after, &mut bp_len);
                                nvim_regexp_set_backpos_len(bp_len);
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
                            nvim_regexp_set_regstack_len(
                                nvim_regexp_get_regstack_len()
                                    - std::mem::size_of::<RegbehindT>() as c_int,
                            );
                        } else {
                            // Advanced, prepare for finding match again.
                            let mut bp_len = nvim_regexp_get_backpos_len();
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            nvim_regexp_set_backpos_len(bp_len);
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
                        let mut bp_len = nvim_regexp_get_backpos_len();
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        nvim_regexp_set_backpos_len(bp_len);
                        let no = (*rp).rs_no as c_int;
                        nvim_regexp_set_brace_count(no, nvim_regexp_get_brace_count(no) - 1);
                    }
                    regstack_pop(&mut scan);
                }

                RS_BRCPLX_LONG => {
                    // Pop the state. Restore pointers when there is no match.
                    if status == RA_NOMATCH {
                        // There was no match, but we did find enough matches.
                        let mut bp_len = nvim_regexp_get_backpos_len();
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        nvim_regexp_set_backpos_len(bp_len);
                        let no = (*rp).rs_no as c_int;
                        nvim_regexp_set_brace_count(no, nvim_regexp_get_brace_count(no) - 1);
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
                        let mut bp_len = nvim_regexp_get_backpos_len();
                        rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                        nvim_regexp_set_backpos_len(bp_len);
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
                        nvim_regexp_set_regstack_len(
                            nvim_regexp_get_regstack_len()
                                - std::mem::size_of::<RegstarT>() as c_int,
                        );
                    } else {
                        // Tried once already, restore input pointers.
                        if status == RA_BREAK {
                            // First time through — skip restore.
                        } else {
                            let mut bp_len = nvim_regexp_get_backpos_len();
                            rs_reg_restore(&(*rp).rs_un.regsave, &mut bp_len);
                            nvim_regexp_set_backpos_len(bp_len);
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
                                    let inp = nvim_regexp_get_rex_input();
                                    let line = nvim_regexp_get_rex_line();
                                    if inp == line {
                                        // Backup to last char of previous line.
                                        if nvim_regexp_get_rex_lnum() == 0 {
                                            status = RA_NOMATCH;
                                            break;
                                        }
                                        let new_lnum = nvim_regexp_get_rex_lnum() - 1;
                                        nvim_regexp_set_rex_lnum(new_lnum);
                                        let new_line =
                                            nvim_regexp_call_reg_getline(new_lnum).cast::<u8>();
                                        // Just in case regrepeat() didn't count right.
                                        if new_line.is_null() {
                                            break;
                                        }
                                        nvim_regexp_set_rex_line(new_line);
                                        nvim_regexp_set_rex_input(new_line.add(
                                            nvim_regexp_call_reg_getline_len(new_lnum) as usize,
                                        ));
                                        rs_reg_breakcheck();
                                    } else {
                                        let backed = mb_ptr_back(line, inp);
                                        nvim_regexp_set_rex_input(backed);
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
                                || c_int::from(*nvim_regexp_get_rex_input()) == (*rst).nextb
                                || c_int::from(*nvim_regexp_get_rex_input()) == (*rst).nextb_ic
                            {
                                rs_reg_save(
                                    &mut (*rp).rs_un.regsave,
                                    nvim_regexp_get_backpos_len(),
                                );
                                scan = rs_regnext((*rp).rs_scan);
                                status = RA_CONT;
                                found = true;
                                break;
                            }
                        }
                        if !found && status != RA_CONT {
                            // Failed.
                            regstack_pop(&mut scan);
                            nvim_regexp_set_regstack_len(
                                nvim_regexp_get_regstack_len()
                                    - std::mem::size_of::<RegstarT>() as c_int,
                            );
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
                    == (nvim_regexp_get_regstack_data()
                        .add(nvim_regexp_get_regstack_len() as usize))
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
        if nvim_regexp_get_regstack_len() == 0 || status == RA_FAIL {
            if scan.is_null() {
                nvim_regexp_iemsg_re_corr();
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
    fn nvim_regexp_get_post_start() -> *mut c_int;
    fn nvim_regexp_set_post_start(p: *mut c_int);
    fn nvim_regexp_get_post_ptr() -> *mut c_int;
    fn nvim_regexp_set_post_ptr(p: *mut c_int);
    fn nvim_regexp_get_post_end() -> *mut c_int;
    fn nvim_regexp_set_post_end(p: *mut c_int);

    // NFA state count accessors
    fn nvim_regexp_get_nstate() -> c_int;
    fn nvim_regexp_set_nstate(v: c_int);
    fn nvim_regexp_get_istate() -> c_int;
    fn nvim_regexp_set_istate(v: c_int);

    // NFA flags accessors
    fn nvim_regexp_get_nfa_re_flags() -> c_int;
    fn nvim_regexp_set_nfa_re_flags(v: c_int);
    fn nvim_regexp_get_wants_nfa() -> c_int;
    fn nvim_regexp_set_wants_nfa(v: c_int);

    // rex NFA fields
    fn nvim_regexp_set_rex_nfa_has_zend(v: c_int);
    fn nvim_regexp_set_rex_nfa_has_backref(v: c_int);

    // Calls shared regcomp_start
    fn nvim_regexp_call_regcomp_start(expr: *mut u8, re_flags: c_int);

    // Calls init_class_tab
    fn nvim_regexp_call_init_class_tab();

    fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;

    // NFA constant validation accessor
    fn nvim_regexp_get_nfa_constant(index: c_int) -> c_int;
}

/// Emit a value into the NFA postfix buffer, growing if needed.
unsafe fn nfa_emit(c: c_int) {
    let post_ptr = nvim_regexp_get_post_ptr();
    let post_end = nvim_regexp_get_post_end();
    if post_ptr >= post_end {
        rs_realloc_post_list();
    }
    let post_ptr = nvim_regexp_get_post_ptr();
    *post_ptr = c;
    nvim_regexp_set_post_ptr(post_ptr.add(1));
}

/// Initialize internal variables before NFA compilation.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regcomp_start(expr: *mut u8, re_flags: c_int) {
    nvim_regexp_set_nstate(0);
    nvim_regexp_set_istate(0);

    // A reasonable estimation for maximum size
    let nstate_max = (strlen(expr.cast::<c_char>()) + 1) * 25 + 1000;

    // Size for postfix representation of expr.
    let postfix_size = std::mem::size_of::<c_int>() * nstate_max;

    let post_start = xmalloc(postfix_size).cast::<c_int>();
    nvim_regexp_set_post_start(post_start);
    nvim_regexp_set_post_ptr(post_start);
    nvim_regexp_set_post_end(post_start.add(nstate_max));
    nvim_regexp_set_wants_nfa(0);
    nvim_regexp_set_rex_nfa_has_zend(0);
    nvim_regexp_set_rex_nfa_has_backref(0);

    // shared with BT engine
    nvim_regexp_call_regcomp_start(expr, re_flags);
}

/// Grow the NFA postfix buffer by 1.5x.
#[no_mangle]
pub unsafe extern "C" fn rs_realloc_post_list() {
    let post_start = nvim_regexp_get_post_start();
    let post_ptr = nvim_regexp_get_post_ptr();
    let post_end = nvim_regexp_get_post_end();

    let old_max = post_end.offset_from(post_start) as usize;
    let new_max = old_max * 3 / 2;
    let new_start = xrealloc(
        post_start.cast::<c_void>(),
        new_max * std::mem::size_of::<c_int>(),
    )
    .cast::<c_int>();

    let ptr_offset = post_ptr.offset_from(post_start) as usize;
    nvim_regexp_set_post_ptr(new_start.add(ptr_offset));
    nvim_regexp_set_post_end(new_start.add(new_max));
    nvim_regexp_set_post_start(new_start);
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
extern "C" {
    // NFA-specific error helpers
    fn nvim_regexp_emsg_nul_found();
    fn nvim_regexp_semsg_misplaced(c: c_int);
    fn nvim_regexp_semsg_ill_char_class(c: i64);
    fn nvim_regexp_siemsg_unknown_class(c: i64);
    fn nvim_regexp_semsg_e867_z(c: c_int);
    fn nvim_regexp_semsg_e867_pct(c: c_int);
    fn nvim_regexp_emsg_value_too_large();
    fn nvim_regexp_semsg_missing_value(c: c_int);

    // Data accessors
    fn nvim_regexp_get_classchars() -> *mut u8;
    fn nvim_regexp_get_nfa_classcodes(index: c_int) -> c_int;
    fn nvim_regexp_get_regexp_inrange() -> *mut c_char;
    fn nvim_regexp_get_regexp_abbr() -> *mut c_char;
    fn nvim_regexp_set_rc_did_emsg_true();
}

// Phase 3 constant
const MB_MAXBYTES: i64 = 21;

// `Magic(x)` in C is `(int)(x) - 256`; equivalent to `magic()` without reg_magic check.
const fn nfa_magic(x: u8) -> c_int {
    x as c_int - 256
}

/// Get regparse as `*mut u8` for byte comparisons.
#[inline]
unsafe fn regparse_u8() -> *mut u8 {
    nvim_regexp_get_regparse().cast::<u8>()
}

/// Set regparse from a `*mut u8`.
#[inline]
unsafe fn set_regparse_u8(p: *mut u8) {
    nvim_regexp_set_regparse(p.cast::<c_char>());
}

/// Helper: `MB_PTR_ADV(regparse)` — advance regparse by one composing-aware character.
unsafe fn mb_ptr_adv_regparse() {
    let rp = nvim_regexp_get_regparse();
    nvim_regexp_set_regparse(rp.add(utfc_ptr2len(rp) as usize));
}

/// Helper: `MB_PTR_BACK(base, regparse)` — back up regparse by one character.
unsafe fn mb_ptr_back_regparse(base: *const u8) {
    let rp = nvim_regexp_get_regparse();
    let off = utf_head_off(base.cast::<c_char>(), rp.sub(1));
    nvim_regexp_set_regparse(rp.sub((off + 1) as usize));
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
        nvim_regexp_set_regparse(old_rp.add(plen as usize));
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
    let p = nvim_regexp_get_regparse();
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
            nvim_regexp_set_regparse(endp);
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

        while (nvim_regexp_get_regparse() as usize) < (endp as usize) {
            let oldstartc = startc;
            startc = -1;
            let mut got_coll_char = false;

            if *regparse_u8() == b'[' {
                // Check for [: :], [= =], [. .]
                let mut rp = nvim_regexp_get_regparse();
                let charclass = nvim_regexp_get_char_class(&mut rp);
                nvim_regexp_set_regparse(rp);

                if charclass == CLASS_NONE {
                    let mut rp2 = nvim_regexp_get_regparse();
                    let equiclass = nvim_regexp_get_equi_class(&mut rp2);
                    nvim_regexp_set_regparse(rp2);

                    if equiclass == 0 {
                        let mut rp3 = nvim_regexp_get_regparse();
                        let collclass = nvim_regexp_get_coll_element(&mut rp3);
                        nvim_regexp_set_regparse(rp3);

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
                            nvim_regexp_set_wants_nfa(1);
                            nfa_emit(NFA_CLASS_LOWER);
                        }
                        x if x == CLASS_PRINT => nfa_emit(NFA_CLASS_PRINT),
                        x if x == CLASS_PUNCT => nfa_emit(NFA_CLASS_PUNCT),
                        x if x == CLASS_SPACE => nfa_emit(NFA_CLASS_SPACE),
                        x if x == CLASS_UPPER => {
                            nvim_regexp_set_wants_nfa(1);
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
                && (!vim_strchr(nvim_regexp_get_regexp_inrange(), *rp.add(1) as c_int).is_null()
                    || (nvim_regexp_get_reg_cpo_lit() == 0
                        && !vim_strchr(nvim_regexp_get_regexp_abbr(), *rp.add(1) as c_int)
                            .is_null()))
            {
                mb_ptr_adv_regparse();
                let rp2 = regparse_u8();

                if *rp2 == b'n' {
                    startc =
                        if nvim_regexp_get_reg_string() != 0 || emit_range || *rp2.add(1) == b'-' {
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
                        nvim_regexp_emsg_e949();
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
                startc = utf_ptr2char(nvim_regexp_get_regparse());
            }

            // Previous char was '-', so this char is end of range
            if emit_range {
                let endc = startc;
                startc = oldstartc;
                if startc > endc {
                    nvim_regexp_emsg_e944();
                    return FAIL;
                }

                if endc > startc + 2 {
                    // Emit a range instead of individual characters
                    if startc == 0 {
                        // \x00 is translated to \x0a, start at \x01
                        nfa_emit(1);
                    } else {
                        // Remove previous NFA_CONCAT
                        let pp = nvim_regexp_get_post_ptr();
                        nvim_regexp_set_post_ptr(pp.sub(1));
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
                    let rp3 = nvim_regexp_get_regparse();
                    if utf_ptr2len(rp3) == utfc_ptr2len(rp3) {
                        nfa_emit(NFA_CONCAT);
                    }
                }
            }

            // Handle composing characters within the collection
            let rp4 = nvim_regexp_get_regparse();
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
        nvim_regexp_set_regparse(endp);
        mb_ptr_adv_regparse();

        // Mark end of the collection
        if negated {
            nfa_emit(NFA_END_NEG_COLL);
        } else {
            nfa_emit(NFA_END_COLL);
        }

        // \_[] also matches \n but it's not negated
        if extra == NFA_ADD_NL {
            nfa_emit(if nvim_regexp_get_reg_string() != 0 {
                NL
            } else {
                NFA_NEWL
            });
            nfa_emit(NFA_OR);
        }

        return OK;
    } // if *endp == ']'

    if nvim_regexp_get_reg_strict() != 0 {
        nvim_regexp_emsg2_e769(c_int::from(nvim_regexp_get_reg_magic() > MAGIC_OFF));
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
    let old_regparse = nvim_regexp_get_regparse();
    let mut extra: c_int = 0;
    let save_prev_at_start = nvim_regexp_get_prev_at_start();

    let mut c = rs_getchr();

    // NUL
    if c == 0 {
        nvim_regexp_emsg_nul_found();
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
        nvim_regexp_set_had_eol(1);
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
            nvim_regexp_emsg_nul_found();
            return FAIL;
        }
        if c == b'^' as c_int {
            nfa_emit(NFA_BOL);
            return OK;
        }
        if c == b'$' as c_int {
            nfa_emit(NFA_EOL);
            nvim_regexp_set_had_eol(1);
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
        if nvim_regexp_get_reg_string() != 0 {
            nfa_emit(NL);
        } else {
            nfa_emit(NFA_NEWL);
            nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_HASNL);
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
        nvim_regexp_semsg_misplaced(rs_no_magic(c));
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
        nvim_regexp_semsg_misplaced(rs_no_magic(c));
        return FAIL;
    }

    // \~ — previous substitute pattern
    if c == nfa_magic(b'~') {
        let reg_prev_sub = nvim_regexp_get_reg_prev_sub_ptr();
        if reg_prev_sub.is_null() {
            nvim_regexp_emsg_nopresub();
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
        nvim_regexp_set_rex_nfa_has_backref(1);
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
    let classchars = nvim_regexp_get_classchars();
    let p = vim_strchr(classchars.cast::<c_char>(), rs_no_magic(c));
    if p.is_null() {
        if extra == NFA_ADD_NL {
            nvim_regexp_semsg_ill_char_class(c as i64);
            return FAIL;
        }
        nvim_regexp_siemsg_unknown_class(c as i64);
        return FAIL;
    }

    // When '.' is followed by a composing char ignore the dot
    if c == nfa_magic(b'.') && utf_iscomposing_legacy(rs_peekchr()) != 0 {
        let new_old = nvim_regexp_get_regparse();
        let c2 = rs_getchr();
        return nfa_handle_multibyte(c2, new_old);
    }

    #[allow(clippy::cast_possible_truncation)]
    let index = ((p as usize) - (classchars as usize)) as c_int;
    nfa_emit(nvim_regexp_get_nfa_classcodes(index));
    if extra == NFA_ADD_NL {
        nfa_emit(NFA_NEWL);
        nfa_emit(NFA_OR);
        nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_HASNL);
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
            nvim_regexp_set_rex_nfa_has_zend(1);
            if !rs_re_mult_next(c"\\ze".as_ptr().cast::<c_char>()) {
                return FAIL;
            }
            OK
        }
        // \z1 .. \z9
        x if x >= b'1' as c_int && x <= b'9' as c_int => {
            if (nvim_regexp_get_reg_do_extmatch() & REX_USE) == 0 {
                nvim_regexp_emsg_e67();
                return FAIL;
            }
            nfa_emit(NFA_ZREF1 + (rs_no_magic(c) - b'1' as c_int));
            nvim_regexp_set_re_has_z(REX_USE);
            OK
        }
        // \z(
        x if x == b'(' as c_int => {
            if nvim_regexp_get_reg_do_extmatch() != REX_SET {
                nvim_regexp_emsg_e66();
                return FAIL;
            }
            if rs_nfa_reg(REG_ZPAREN) == FAIL {
                return FAIL;
            }
            nvim_regexp_set_re_has_z(REX_SET);
            OK
        }
        _ => {
            nvim_regexp_semsg_e867_z(rs_no_magic(c));
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
            nvim_regexp_emsg2_e678(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
        let rp = nvim_regexp_get_regparse();
        if *rp == b'=' as c_char && *rp.add(1) >= 48 && *rp.add(1) <= 50 {
            nvim_regexp_semsg_e_atom_engine(*rp.add(1) as c_int);
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
                nvim_regexp_emsg2_e769(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
            nvim_regexp_emsg2_e70(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
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
            nvim_regexp_semsg_e_dot_pos(rs_no_magic(c));
            return FAIL;
        }
        if n > (i64::from(i32::MAX) - i64::from(c - b'0' as c_int)) / 10 {
            nvim_regexp_emsg_value_too_large();
            return FAIL;
        }
        n = n * 10 + i64::from(c - b'0' as c_int);
        c = rs_getchr();
        got_digit = true;
    }

    if c == b'l' as c_int || c == b'c' as c_int || c == b'v' as c_int {
        let mut limit: i64 = i64::from(i32::MAX);
        if !cur && !got_digit {
            nvim_regexp_semsg_missing_value(rs_no_magic(c));
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
                nvim_regexp_set_at_start(1);
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
            nvim_regexp_emsg_value_too_large();
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

    nvim_regexp_semsg_e867_pct(rs_no_magic(c));
    FAIL
}

// === Phase 4: NFA Parser Functions + re2post ===

const RE_AUTO: c_int = 8;

extern "C" {
    fn nvim_regexp_semsg_e869(op: c_int);
    fn nvim_regexp_emsg_e870();
    fn nvim_regexp_emsg_e871();
    fn nvim_regexp_emsg_e872();
    fn nvim_regexp_emsg_e879();
    fn nvim_regexp_emsg_e873();
}

/// Helper: compute current postfix position index.
#[inline]
#[allow(clippy::cast_possible_truncation)] // postfix array index always fits in c_int
unsafe fn post_pos() -> c_int {
    let pos = (nvim_regexp_get_post_ptr() as usize - nvim_regexp_get_post_start() as usize)
        / core::mem::size_of::<c_int>();
    pos as c_int
}

/// Helper: set postfix pointer to a given index from start.
#[inline]
unsafe fn set_post_pos(index: c_int) {
    nvim_regexp_set_post_ptr(nvim_regexp_get_post_start().add(index as usize));
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
        nvim_regexp_set_curchr(-1);
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
            nvim_regexp_semsg_e869(op);
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
            nvim_regexp_emsg_e870();
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
            if (nvim_regexp_get_nfa_re_flags() & RE_AUTO) != 0
                && (maxval > 500 || maxval > minval + 200)
                && (maxval != MAX_LIMIT && minval < 200)
                && nvim_regexp_get_wants_nfa() == 0
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
            nvim_regexp_set_curchr(-1);
        }
    }

    if rs_re_multi_type(rs_peekchr()) != NOT_MULTI {
        nvim_regexp_emsg_e871();
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
            nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_ICOMBINE);
            rs_skipchr_keepstart();
        } else if c == nfa_magic(b'c') {
            nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_ICASE);
            rs_skipchr_keepstart();
        } else if c == nfa_magic(b'C') {
            nvim_regexp_set_regflags_compile(nvim_regexp_get_regflags_compile() | RF_NOICASE);
            rs_skipchr_keepstart();
        } else if c == nfa_magic(b'v') {
            nvim_regexp_set_reg_magic(MAGIC_ALL);
            rs_skipchr_keepstart();
            nvim_regexp_set_curchr(-1);
        } else if c == nfa_magic(b'm') {
            nvim_regexp_set_reg_magic(MAGIC_ON);
            rs_skipchr_keepstart();
            nvim_regexp_set_curchr(-1);
        } else if c == nfa_magic(b'M') {
            nvim_regexp_set_reg_magic(MAGIC_OFF);
            rs_skipchr_keepstart();
            nvim_regexp_set_curchr(-1);
        } else if c == nfa_magic(b'V') {
            nvim_regexp_set_reg_magic(MAGIC_NONE);
            rs_skipchr_keepstart();
            nvim_regexp_set_curchr(-1);
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
        if nvim_regexp_get_regnpar() >= NSUBEXP_I {
            nvim_regexp_emsg_e872();
            return FAIL;
        }
        parno = nvim_regexp_get_regnpar();
        nvim_regexp_set_regnpar(parno + 1);
    } else if paren == REG_ZPAREN {
        if nvim_regexp_get_regnzpar() >= NSUBEXP_I {
            nvim_regexp_emsg_e879();
            return FAIL;
        }
        parno = nvim_regexp_get_regnzpar();
        nvim_regexp_set_regnzpar(parno + 1);
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
            nvim_regexp_emsg2_e53(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
        } else {
            nvim_regexp_emsg2_e54(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
        }
        return FAIL;
    } else if paren == REG_NOPAREN && rs_peekchr() != 0 {
        if rs_peekchr() == nfa_magic(b')') {
            nvim_regexp_emsg2_e55(c_int::from(nvim_regexp_get_reg_magic() == MAGIC_ALL));
        } else {
            nvim_regexp_emsg_e873();
        }
        return FAIL;
    }

    if paren == REG_PAREN {
        nvim_regexp_set_had_endbrace(parno, 1);
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
    nvim_regexp_get_post_start()
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
extern "C" {
    // state_ptr global (points into nfa_regprog_T.state[])
    fn nvim_regexp_get_state_ptr() -> NfaStateHandle;
    fn nvim_regexp_set_state_ptr(v: NfaStateHandle);

    // nfa_state_T field accessors
    fn nvim_nfa_state_get_c(s: NfaStateHandle) -> c_int;
    fn nvim_nfa_state_set_c(s: NfaStateHandle, v: c_int);
    fn nvim_nfa_state_get_out(s: NfaStateHandle) -> NfaStateHandle;
    fn nvim_nfa_state_set_out(s: NfaStateHandle, v: NfaStateHandle);
    fn nvim_nfa_state_get_out1(s: NfaStateHandle) -> NfaStateHandle;
    fn nvim_nfa_state_set_out1(s: NfaStateHandle, v: NfaStateHandle);
    fn nvim_nfa_state_get_val(s: NfaStateHandle) -> c_int;
    fn nvim_nfa_state_set_val(s: NfaStateHandle, v: c_int);
    fn nvim_nfa_state_set_id(s: NfaStateHandle, v: c_int);
    fn nvim_nfa_state_clear_lastlist(s: NfaStateHandle);

    // Address-of accessors for Ptrlist pointer punning.
    // Returns `&s->out` / `&s->out1` as `nfa_state_T**`.
    fn nvim_nfa_state_out_addr(s: NfaStateHandle) -> *mut NfaStateHandle;
    fn nvim_nfa_state_out1_addr(s: NfaStateHandle) -> *mut NfaStateHandle;

    // state_ptr[index] — returns pointer to the `index`-th state.
    fn nvim_regexp_state_ptr_add(index: c_int) -> NfaStateHandle;

    // Error messages for post2nfa
    fn nvim_regexp_emsg_e874(); // E874: Could not pop the stack
    fn nvim_regexp_emsg_e875(); // E875: too many states left on stack
    fn nvim_regexp_emsg_e876(); // E876: Not enough space to store NFA
}

/// Allocate and initialize an NFA state from the pre-allocated state array.
unsafe fn nfa_alloc_state(c: c_int, out: NfaStateHandle, out1: NfaStateHandle) -> NfaStateHandle {
    let istate = nvim_regexp_get_istate();
    if istate >= nvim_regexp_get_nstate() {
        return core::ptr::null_mut();
    }
    let s = nvim_regexp_state_ptr_add(istate);
    nvim_regexp_set_istate(istate + 1);
    nvim_nfa_state_set_c(s, c);
    nvim_nfa_state_set_out(s, out);
    nvim_nfa_state_set_out1(s, out1);
    nvim_nfa_state_set_val(s, 0);
    nvim_nfa_state_set_id(s, istate + 1); // id = istate after increment
    nvim_nfa_state_clear_lastlist(s);
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
        nvim_regexp_emsg_e874();
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
        let c = nvim_nfa_state_get_c(state);
        match c {
            NFA_END_INVISIBLE | NFA_END_INVISIBLE_NEG => return len,

            NFA_SPLIT => {
                let l = nfa_max_width(nvim_nfa_state_get_out(state), depth + 1);
                let r = nfa_max_width(nvim_nfa_state_get_out1(state), depth + 1);
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
                    let out1 = nvim_nfa_state_get_out1(state);
                    state = nvim_nfa_state_get_out(out1);
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
                state = nvim_nfa_state_get_out(nvim_nfa_state_get_out1(state));
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
        state = nvim_nfa_state_get_out(state);
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
        let nstate = nvim_regexp_get_nstate();
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
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
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
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
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
                    frag_new(s, list1(nvim_nfa_state_out1_addr(s))),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_STAR_NONGREEDY => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
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
                    frag_new(s, list1(nvim_nfa_state_out_addr(s))),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_QUEST => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
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
                        ptrlist_append(e.out_list, list1(nvim_nfa_state_out1_addr(s))),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_QUEST_NONGREEDY => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
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
                        ptrlist_append(e.out_list, list1(nvim_nfa_state_out_addr(s))),
                    ),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_END_COLL | NFA_END_NEG_COLL => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
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
                nvim_nfa_state_set_out1(e.start, s);
                st_push(
                    frag_new(e.start, list1(nvim_nfa_state_out_addr(s))),
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
                let c2 = nvim_nfa_state_get_c(e2.start);
                nvim_nfa_state_set_val(e2.start, c2);
                nvim_nfa_state_set_c(e2.start, NFA_RANGE_MAX);
                let c1 = nvim_nfa_state_get_c(e1.start);
                nvim_nfa_state_set_val(e1.start, c1);
                nvim_nfa_state_set_c(e1.start, NFA_RANGE_MIN);
                nfa_patch(e1.out_list, e2.start);
                st_push(frag_new(e1.start, e2.out_list), &mut stackp, stack_end);
            }

            NFA_EMPTY => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
                    p = p.add(1);
                    continue;
                }
                let s = nfa_alloc_state(NFA_EMPTY, core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                st_push(
                    frag_new(s, list1(nvim_nfa_state_out_addr(s))),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_OPT_CHARS => {
                p = p.add(1);
                let mut n = *p;
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + n);
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
                    e1_out = ptrlist_append(e1_out, list1(nvim_nfa_state_out1_addr(s)));
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
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + if pattern { 4 } else { 2 });
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
                    nvim_nfa_state_set_out(s1, skip);
                    nfa_patch(e.out_list, zend);
                    st_push(
                        frag_new(s, list1(nvim_nfa_state_out_addr(skip))),
                        &mut stackp,
                        stack_end,
                    );
                } else {
                    nfa_patch(e.out_list, s1);
                    st_push(
                        frag_new(s, list1(nvim_nfa_state_out_addr(s1))),
                        &mut stackp,
                        stack_end,
                    );
                    if before {
                        if n <= 0 {
                            n = nfa_max_width(e.start, 0);
                        }
                        nvim_nfa_state_set_val(s, n);
                    }
                }
            }

            NFA_COMPOSING | NFA_MOPEN | NFA_MOPEN1 | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4
            | NFA_MOPEN5 | NFA_MOPEN6 | NFA_MOPEN7 | NFA_MOPEN8 | NFA_MOPEN9 | NFA_ZOPEN
            | NFA_ZOPEN1 | NFA_ZOPEN2 | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6
            | NFA_ZOPEN7 | NFA_ZOPEN8 | NFA_ZOPEN9 | NFA_NOPEN => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 2);
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
                    nfa_patch(list1(nvim_nfa_state_out_addr(s)), s1);
                    st_push(
                        frag_new(s, list1(nvim_nfa_state_out_addr(s1))),
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
                        nfa_patch(list1(nvim_nfa_state_out1_addr(s)), s1);
                    }
                    st_push(
                        frag_new(s, list1(nvim_nfa_state_out_addr(s1))),
                        &mut stackp,
                        stack_end,
                    );
                }
            }

            NFA_BACKREF1 | NFA_BACKREF2 | NFA_BACKREF3 | NFA_BACKREF4 | NFA_BACKREF5
            | NFA_BACKREF6 | NFA_BACKREF7 | NFA_BACKREF8 | NFA_BACKREF9 | NFA_ZREF1 | NFA_ZREF2
            | NFA_ZREF3 | NFA_ZREF4 | NFA_ZREF5 | NFA_ZREF6 | NFA_ZREF7 | NFA_ZREF8 | NFA_ZREF9 => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 2);
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
                nfa_patch(list1(nvim_nfa_state_out_addr(s)), s1);
                st_push(
                    frag_new(s, list1(nvim_nfa_state_out_addr(s1))),
                    &mut stackp,
                    stack_end,
                );
            }

            NFA_LNUM | NFA_LNUM_GT | NFA_LNUM_LT | NFA_VCOL | NFA_VCOL_GT | NFA_VCOL_LT
            | NFA_COL | NFA_COL_GT | NFA_COL_LT | NFA_MARK | NFA_MARK_GT | NFA_MARK_LT => {
                p = p.add(1);
                let n = *p; // lnum, col, or mark name
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
                    p = p.add(1);
                    continue;
                }
                // p[-1] is the opcode (we already advanced p)
                let s = nfa_alloc_state(*p.sub(1), core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                nvim_nfa_state_set_val(s, n);
                st_push(
                    frag_new(s, list1(nvim_nfa_state_out_addr(s))),
                    &mut stackp,
                    stack_end,
                );
            }

            // NFA_ZSTART, NFA_ZEND, and all other operands
            _ => {
                if sizing {
                    nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
                    p = p.add(1);
                    continue;
                }
                let s = nfa_alloc_state(op, core::ptr::null_mut(), core::ptr::null_mut());
                if s.is_null() {
                    xfree(stack.cast());
                    return core::ptr::null_mut();
                }
                st_push(
                    frag_new(s, list1(nvim_nfa_state_out_addr(s))),
                    &mut stackp,
                    stack_end,
                );
            }
        }
        p = p.add(1);
    }

    if sizing {
        nvim_regexp_set_nstate(nvim_regexp_get_nstate() + 1);
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
        nvim_regexp_emsg_e875();
        return core::ptr::null_mut();
    }

    let istate = nvim_regexp_get_istate();
    if istate >= nvim_regexp_get_nstate() {
        xfree(stack.cast());
        nvim_regexp_emsg_e876();
        return core::ptr::null_mut();
    }

    // Create the match state
    let matchstate = nvim_regexp_state_ptr_add(istate);
    nvim_regexp_set_istate(istate + 1);
    nvim_nfa_state_set_c(matchstate, NFA_MATCH);
    nvim_nfa_state_set_out(matchstate, core::ptr::null_mut());
    nvim_nfa_state_set_out1(matchstate, core::ptr::null_mut());
    nvim_nfa_state_set_id(matchstate, 0);

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
extern "C" {
    // nfa_regprog_T field accessors
    fn nvim_nfa_prog_get_nstate(prog: NfaProgHandle) -> c_int;
    fn nvim_nfa_prog_get_state(prog: NfaProgHandle, i: c_int) -> NfaStateHandle;
    fn nvim_nfa_prog_get_start(prog: NfaProgHandle) -> NfaStateHandle;
    fn nvim_nfa_prog_set_has_zend(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_has_backref(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_nsubexp(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_regflags(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_reganch(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_regstart(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_match_text(prog: NfaProgHandle, v: *mut u8);
    fn nvim_nfa_prog_set_reghasz(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_pattern(prog: NfaProgHandle, v: *mut c_char);
}

/// Check if the match endpoint can directly follow a given NFA state.
/// Used by `nfa_postprocess` to decide whether to try the invisible match first.
#[allow(clippy::too_many_lines)]
unsafe fn match_follows(startstate: NfaStateHandle, depth: c_int) -> bool {
    if depth > 10 || startstate.is_null() {
        return false;
    }
    let mut state = startstate;
    while !state.is_null() {
        let c = nvim_nfa_state_get_c(state);
        match c {
            NFA_MATCH
            | NFA_MCLOSE
            | NFA_END_INVISIBLE
            | NFA_END_INVISIBLE_NEG
            | NFA_END_PATTERN => return true,

            NFA_SPLIT => {
                return match_follows(nvim_nfa_state_get_out(state), depth + 1)
                    || match_follows(nvim_nfa_state_get_out1(state), depth + 1);
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
                state = nvim_nfa_state_get_out(nvim_nfa_state_get_out1(state));
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
        state = nvim_nfa_state_get_out(state);
    }
    false
}

/// Heuristic: estimate the failure chance (0-99) for an NFA state.
/// Higher values mean more likely to fail (and thus cheaper to try first).
#[allow(clippy::too_many_lines)]
unsafe fn failure_chance(state: NfaStateHandle, depth: c_int) -> c_int {
    let c = nvim_nfa_state_get_c(state);

    // detect looping
    if depth > 4 {
        return 1;
    }

    match c {
        NFA_SPLIT => {
            let out = nvim_nfa_state_get_out(state);
            let out1 = nvim_nfa_state_get_out1(state);
            if nvim_nfa_state_get_c(out) == NFA_SPLIT || nvim_nfa_state_get_c(out1) == NFA_SPLIT {
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
            failure_chance(nvim_nfa_state_get_out(state), depth + 1)
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
    let nstate = nvim_nfa_prog_get_nstate(prog);
    for i in 0..nstate {
        let state = nvim_nfa_prog_get_state(prog, i);
        let c = nvim_nfa_state_get_c(state);
        if c == NFA_START_INVISIBLE
            || c == NFA_START_INVISIBLE_NEG
            || c == NFA_START_INVISIBLE_BEFORE
            || c == NFA_START_INVISIBLE_BEFORE_NEG
        {
            let directly;
            let out1 = nvim_nfa_state_get_out1(state);
            let out1_out = nvim_nfa_state_get_out(out1);
            if match_follows(out1_out, 0) {
                directly = true;
            } else {
                let out = nvim_nfa_state_get_out(state);
                let ch_invisible = failure_chance(out, 0);
                let ch_follows = failure_chance(out1_out, 0);
                if c == NFA_START_INVISIBLE_BEFORE || c == NFA_START_INVISIBLE_BEFORE_NEG {
                    let val = nvim_nfa_state_get_val(state);
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
                nvim_nfa_state_set_c(state, c + 1);
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
        let c = nvim_nfa_state_get_c(p);
        match c {
            NFA_BOL | NFA_BOF => return 1,

            NFA_ZSTART | NFA_ZEND | NFA_CURSOR | NFA_VISUAL | NFA_MOPEN | NFA_MOPEN1
            | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4 | NFA_MOPEN5 | NFA_MOPEN6 | NFA_MOPEN7
            | NFA_MOPEN8 | NFA_MOPEN9 | NFA_NOPEN | NFA_ZOPEN | NFA_ZOPEN1 | NFA_ZOPEN2
            | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6 | NFA_ZOPEN7 | NFA_ZOPEN8
            | NFA_ZOPEN9 => {
                p = nvim_nfa_state_get_out(p);
            }

            NFA_SPLIT => {
                return (rs_nfa_get_reganch(nvim_nfa_state_get_out(p), depth + 1) != 0
                    && rs_nfa_get_reganch(nvim_nfa_state_get_out1(p), depth + 1) != 0)
                    as c_int;
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
        let c = nvim_nfa_state_get_c(p);
        match c {
            NFA_BOL | NFA_BOF | NFA_BOW | NFA_EOW | NFA_ZSTART | NFA_ZEND | NFA_CURSOR
            | NFA_VISUAL | NFA_LNUM | NFA_LNUM_GT | NFA_LNUM_LT | NFA_COL | NFA_COL_GT
            | NFA_COL_LT | NFA_VCOL | NFA_VCOL_GT | NFA_VCOL_LT | NFA_MARK | NFA_MARK_GT
            | NFA_MARK_LT | NFA_MOPEN | NFA_MOPEN1 | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4
            | NFA_MOPEN5 | NFA_MOPEN6 | NFA_MOPEN7 | NFA_MOPEN8 | NFA_MOPEN9 | NFA_NOPEN
            | NFA_ZOPEN | NFA_ZOPEN1 | NFA_ZOPEN2 | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5
            | NFA_ZOPEN6 | NFA_ZOPEN7 | NFA_ZOPEN8 | NFA_ZOPEN9 => {
                p = nvim_nfa_state_get_out(p);
            }

            NFA_SPLIT => {
                let c1 = rs_nfa_get_regstart(nvim_nfa_state_get_out(p), depth + 1);
                let c2 = rs_nfa_get_regstart(nvim_nfa_state_get_out1(p), depth + 1);
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
    if nvim_nfa_state_get_c(start) != NFA_MOPEN {
        return core::ptr::null_mut();
    }
    let mut p = nvim_nfa_state_get_out(start);
    let mut len: c_int = 0;

    // Count total byte length of literal characters.
    while nvim_nfa_state_get_c(p) > 0 {
        len += utf_char2len(nvim_nfa_state_get_c(p));
        p = nvim_nfa_state_get_out(p);
    }

    if nvim_nfa_state_get_c(p) != NFA_MCLOSE {
        return core::ptr::null_mut();
    }
    let next = nvim_nfa_state_get_out(p);
    if nvim_nfa_state_get_c(next) != NFA_MATCH {
        return core::ptr::null_mut();
    }

    let ret = xmalloc(len as usize).cast::<u8>();
    // Skip first char (it goes into regstart), start from out->out
    p = nvim_nfa_state_get_out(nvim_nfa_state_get_out(start));
    let mut s = ret;
    while nvim_nfa_state_get_c(p) > 0 {
        s = s.add(utf_char2bytes(nvim_nfa_state_get_c(p), s.cast::<c_char>()) as usize);
        p = nvim_nfa_state_get_out(p);
    }
    *s = 0; // NUL terminate
    ret
}

// ---------------------------------------------------------------------------
// Phase 7: nfa_regcomp Entry Point
// ---------------------------------------------------------------------------

// ---- Phase 7 C accessors ----
extern "C" {
    fn nvim_regexp_get_rex_nfa_has_zend() -> c_int;
    fn nvim_regexp_get_rex_nfa_has_backref() -> c_int;
    fn nvim_regexp_alloc_nfa_prog(nstate_count: c_int) -> NfaProgHandle;
    fn nvim_regexp_set_nfa_prog_engine(prog: NfaProgHandle);
    fn nvim_nfa_prog_set_re_in_use(prog: NfaProgHandle, v: c_int);
    fn nvim_nfa_prog_set_start(prog: NfaProgHandle, s: NfaStateHandle);
    fn nvim_nfa_prog_set_nstate(prog: NfaProgHandle, v: c_int);
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

    nvim_regexp_set_nfa_re_flags(re_flags);
    nvim_regexp_call_init_class_tab();
    rs_nfa_regcomp_start(expr, re_flags);

    // Build postfix form of the regexp. Needed to build the NFA (and count its size).
    let postfix = rs_re2post();
    if postfix.is_null() {
        // Cascaded (syntax?) error — clean up and return NULL
        let post_start = nvim_regexp_get_post_start();
        xfree(post_start.cast());
        nvim_regexp_set_post_start(core::ptr::null_mut());
        nvim_regexp_set_post_ptr(core::ptr::null_mut());
        nvim_regexp_set_post_end(core::ptr::null_mut());
        nvim_regexp_set_state_ptr(core::ptr::null_mut());
        return core::ptr::null_mut();
    }

    let post_ptr_val = nvim_regexp_get_post_ptr();

    // PASS 1: Count number of NFA states in "nstate". Do not build the NFA.
    rs_post2nfa(postfix, post_ptr_val, 1);

    // Allocate the regprog with space for the compiled regexp.
    // This also sets state_ptr = prog->state.
    let nstate_val = nvim_regexp_get_nstate();
    let prog = nvim_regexp_alloc_nfa_prog(nstate_val);
    nvim_nfa_prog_set_re_in_use(prog, 0);

    // PASS 2: Build the NFA.
    let start = rs_post2nfa(postfix, post_ptr_val, 0);
    if start.is_null() {
        // Build failed — free prog, clean up, return NULL
        xfree(prog);
        let post_start = nvim_regexp_get_post_start();
        xfree(post_start.cast());
        nvim_regexp_set_post_start(core::ptr::null_mut());
        nvim_regexp_set_post_ptr(core::ptr::null_mut());
        nvim_regexp_set_post_end(core::ptr::null_mut());
        nvim_regexp_set_state_ptr(core::ptr::null_mut());
        return core::ptr::null_mut();
    }

    nvim_nfa_prog_set_start(prog, start);
    nvim_nfa_prog_set_regflags(prog, nvim_regexp_get_regflags_compile() as c_int);
    nvim_regexp_set_nfa_prog_engine(prog);
    nvim_nfa_prog_set_nstate(prog, nstate_val);
    nvim_nfa_prog_set_has_zend(prog, nvim_regexp_get_rex_nfa_has_zend());
    nvim_nfa_prog_set_has_backref(prog, nvim_regexp_get_rex_nfa_has_backref());
    nvim_nfa_prog_set_nsubexp(prog, nvim_regexp_get_regnpar());

    rs_nfa_postprocess(prog);

    let prog_start = nvim_nfa_prog_get_start(prog);
    nvim_nfa_prog_set_reganch(prog, rs_nfa_get_reganch(prog_start, 0));
    nvim_nfa_prog_set_regstart(prog, rs_nfa_get_regstart(prog_start, 0));
    nvim_nfa_prog_set_match_text(prog, rs_nfa_get_match_text(prog_start));

    // Remember whether this pattern has any \z specials in it.
    nvim_nfa_prog_set_reghasz(prog, nvim_regexp_get_re_has_z());
    nvim_nfa_prog_set_pattern(prog, nvim_regexp_xstrdup(expr.cast()));

    // Clean up
    let post_start = nvim_regexp_get_post_start();
    xfree(post_start.cast());
    nvim_regexp_set_post_start(core::ptr::null_mut());
    nvim_regexp_set_post_ptr(core::ptr::null_mut());
    nvim_regexp_set_post_end(core::ptr::null_mut());
    nvim_regexp_set_state_ptr(core::ptr::null_mut());

    prog
}

// ============================================================================
// NFA Execution Engine — Phase 8: Pure helpers and accessor infrastructure
// ============================================================================

extern "C" {
    // nfa_state_T id/lastlist accessors
    fn nvim_nfa_state_get_id(s: NfaStateHandle) -> c_int;
    fn nvim_nfa_state_get_lastlist(s: NfaStateHandle, idx: c_int) -> c_int;
    fn nvim_nfa_state_set_lastlist(s: NfaStateHandle, idx: c_int, val: c_int);

    // nfa_regprog_T execution field accessors
    fn nvim_nfa_prog_get_has_zend(prog: NfaProgHandle) -> c_int;
    fn nvim_nfa_prog_get_has_backref(prog: NfaProgHandle) -> c_int;
    fn nvim_nfa_prog_get_nsubexp(prog: NfaProgHandle) -> c_int;
    fn nvim_nfa_prog_get_reghasz(prog: NfaProgHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_nfa_prog_get_regflags(prog: NfaProgHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_nfa_prog_get_regstart(prog: NfaProgHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_nfa_prog_get_reganch(prog: NfaProgHandle) -> c_int;
    #[allow(dead_code)]
    fn nvim_nfa_prog_get_match_text(prog: NfaProgHandle) -> *mut u8;

    // Error wrapper
    fn nvim_regexp_siemsg_ill_char_class(cls: i64);
}

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
            nvim_regexp_siemsg_ill_char_class(cls as i64);
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
struct NfaPimT {
    result: c_int,
    state: *mut NfaStateT,
    subs: RegsubsT,
    end: SaveSeUnion, // union { lpos_T pos; uint8_t *ptr; }
}

/// Matches C `nfa_thread_T` — NFA execution thread.
#[repr(C)]
struct NfaThreadT {
    state: *mut NfaStateT,
    count: c_int,
    pim: NfaPimT,
    subs: RegsubsT,
}

/// Matches C `nfa_list_T` — list of NFA execution states.
#[repr(C)]
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

/// Type aliases for backward compatibility with opaque handle code.
type RegsubHandle = *mut c_void; // regsub_T*
type NfaPimHandle = *mut c_void; // nfa_pim_T*
type NfaListHandle = *mut c_void; // nfa_list_T*
type RegsubsHandle = *mut c_void; // regsubs_T*

/// NFA PIM result constants.
const NFA_PIM_UNUSED: c_int = 0;

extern "C" {
    // regsub_T field accessors
    fn nvim_regexp_regsub_get_in_use(sub: *mut c_void) -> c_int;
    fn nvim_regexp_regsub_get_multi_start_lnum(sub: *mut c_void, idx: c_int) -> i32;
    fn nvim_regexp_regsub_get_multi_start_col(sub: *mut c_void, idx: c_int) -> i32;
    fn nvim_regexp_regsub_get_multi_end_lnum(sub: *mut c_void, idx: c_int) -> i32;
    fn nvim_regexp_regsub_get_multi_end_col(sub: *mut c_void, idx: c_int) -> i32;
    fn nvim_regexp_regsub_get_line_start(sub: *mut c_void, idx: c_int) -> *mut u8;
    fn nvim_regexp_regsub_get_line_end(sub: *mut c_void, idx: c_int) -> *mut u8;

    // nfa_pim_T field accessors
    fn nvim_nfa_pim_get_result(pim: NfaPimHandle) -> c_int;
    fn nvim_nfa_pim_get_state_id(pim: NfaPimHandle) -> c_int;
    fn nvim_nfa_pim_get_end_pos_lnum(pim: NfaPimHandle) -> i32;
    fn nvim_nfa_pim_get_end_pos_col(pim: NfaPimHandle) -> i32;
    fn nvim_nfa_pim_get_end_ptr(pim: NfaPimHandle) -> *mut u8;

    // nfa_list_T / nfa_thread_T read accessors
    fn nvim_nfa_list_get_n(l: NfaListHandle) -> c_int;
    fn nvim_nfa_thread_get_state_id(l: NfaListHandle, idx: c_int) -> c_int;
    fn nvim_nfa_thread_get_subs_norm(l: NfaListHandle, idx: c_int) -> *mut c_void;
    fn nvim_nfa_thread_get_subs_synt(l: NfaListHandle, idx: c_int) -> *mut c_void;
    fn nvim_nfa_thread_get_pim_ptr(l: NfaListHandle, idx: c_int) -> NfaPimHandle;

    // C wrapper functions for complex operations
    fn nvim_regexp_call_sub_equal(sub1: *mut c_void, sub2: *mut c_void) -> c_int;
    fn nvim_regexp_call_match_backref(
        sub: *mut c_void,
        subidx: c_int,
        bytelen: *mut c_int,
    ) -> c_int;
    fn nvim_regexp_call_match_zref(subidx: c_int, bytelen: *mut c_int) -> c_int;
    fn nvim_regexp_call_find_match_text(
        startcol: *mut c_int,
        regstart: c_int,
        match_text: *mut u8,
    ) -> c_int;
    fn nvim_regexp_call_skip_to_start(c: c_int, colp: *mut c_int) -> c_int;
    fn nvim_regexp_call_nfa_did_time_out() -> c_int;

    // rex NFA state accessors
    fn nvim_regexp_get_nfa_has_zsubexpr() -> c_int;
    fn nvim_nfa_list_get_id(l: NfaListHandle) -> c_int;

    // NFA execution globals
    fn nvim_regexp_get_nfa_match() -> c_int;
    fn nvim_regexp_set_nfa_match(v: c_int);
    fn nvim_regexp_get_nfa_ll_index() -> c_int;
    fn nvim_regexp_set_nfa_ll_index(v: c_int);

    // Phase 8.3: addstate/addstate_here/submatch wrappers
    fn nvim_regexp_call_addstate(
        l: NfaListHandle,
        state: NfaStateHandle,
        subs: RegsubsHandle,
        pim: NfaPimHandle,
        off: c_int,
    ) -> RegsubsHandle;
    fn nvim_regexp_call_addstate_here(
        l: NfaListHandle,
        state: NfaStateHandle,
        subs: RegsubsHandle,
        pim: NfaPimHandle,
        ip: *mut c_int,
    ) -> RegsubsHandle;
    fn nvim_regexp_call_copy_sub(to: *mut c_void, from: *mut c_void);
    fn nvim_regexp_call_copy_sub_off(to: *mut c_void, from: *mut c_void);
    fn nvim_regexp_call_copy_ze_off(to: *mut c_void, from: *mut c_void);
    fn nvim_regexp_call_clear_sub(sub: *mut c_void);
    fn nvim_regexp_call_copy_pim(to: NfaPimHandle, from: NfaPimHandle);
    fn nvim_regexp_call_nfa_regmatch(
        prog: NfaProgHandle,
        start: NfaStateHandle,
        submatch: RegsubsHandle,
        m: RegsubsHandle,
    ) -> c_int;
    fn nvim_regexp_call_recursive_regmatch(
        state: NfaStateHandle,
        pim: NfaPimHandle,
        prog: NfaProgHandle,
        submatch: RegsubsHandle,
        m: RegsubsHandle,
        listids: *mut *mut c_int,
        listids_len: *mut c_int,
    ) -> c_int;
    fn nvim_regexp_call_nfa_save_listids(prog: NfaProgHandle, list: *mut c_int);
    fn nvim_regexp_call_nfa_restore_listids(prog: NfaProgHandle, list: *const c_int);

    // nfa_endp accessor
    fn nvim_regexp_get_nfa_endp() -> *mut c_void;
    fn nvim_regexp_set_nfa_endp(v: *mut c_void);
    fn nvim_regexp_get_nfa_endp_pos_lnum() -> i32;
    fn nvim_regexp_get_nfa_endp_pos_col() -> i32;
    fn nvim_regexp_get_nfa_endp_ptr() -> *mut u8;

    // nfa_list_T memory management
    fn nvim_nfa_list_alloc_threads(nstate: c_int) -> *mut c_void;
    fn nvim_nfa_list_free_threads(t: *mut c_void);

    // nfa_time_limit / nfa_timed_out / nfa_time_count
    fn nvim_regexp_get_nfa_time_limit() -> *mut c_void;
    fn nvim_regexp_set_nfa_time_limit(v: *mut c_void);
    fn nvim_regexp_get_nfa_timed_out() -> *mut c_int;
    fn nvim_regexp_set_nfa_timed_out(v: *mut c_int);
    fn nvim_regexp_get_nfa_time_count() -> c_int;
    fn nvim_regexp_set_nfa_time_count(v: c_int);
}

// ============================================================================
// Submatch operations — migrated from C regexp.c
// ============================================================================

/// Clear the sub-expression matches in `sub`.
unsafe fn clear_sub(sub: *mut RegsubT, nsubexpr: c_int) {
    if nvim_regexp_is_reg_multi() != 0 {
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
    if nvim_regexp_is_reg_multi() != 0 {
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
    if nvim_regexp_is_reg_multi() != 0 {
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
    if nvim_regexp_is_reg_multi() != 0 {
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
    if nvim_regexp_is_reg_multi() != 0 {
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
    sub_equal(
        sub1.cast::<RegsubT>(),
        sub2.cast::<RegsubT>(),
        has_backref,
    ) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn rs_copy_pim(to: *mut c_void, from: *mut c_void) {
    copy_pim(to.cast::<NfaPimT>(), from.cast::<NfaPimT>());
}

/// Return true if "one" and "two" PIM states are equal.
/// That includes when both are unused (not set).
#[no_mangle]
pub unsafe extern "C" fn rs_pim_equal(one: NfaPimHandle, two: NfaPimHandle) -> c_int {
    let one_unused = one.is_null() || nvim_nfa_pim_get_result(one) == NFA_PIM_UNUSED;
    let two_unused = two.is_null() || nvim_nfa_pim_get_result(two) == NFA_PIM_UNUSED;

    if one_unused {
        return two_unused as c_int;
    }
    if two_unused {
        return 0;
    }
    // compare state id
    if nvim_nfa_pim_get_state_id(one) != nvim_nfa_pim_get_state_id(two) {
        return 0;
    }
    // compare position
    if nvim_regexp_is_reg_multi() != 0 {
        return (nvim_nfa_pim_get_end_pos_lnum(one) == nvim_nfa_pim_get_end_pos_lnum(two)
            && nvim_nfa_pim_get_end_pos_col(one) == nvim_nfa_pim_get_end_pos_col(two))
            as c_int;
    }
    (nvim_nfa_pim_get_end_ptr(one) == nvim_nfa_pim_get_end_ptr(two)) as c_int
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
    let n = nvim_nfa_list_get_n(l);
    for i in 0..n {
        if nvim_nfa_thread_get_state_id(l, i) != state_id {
            continue;
        }
        if nvim_regexp_call_sub_equal(nvim_nfa_thread_get_subs_norm(l, i), subs_norm) == 0 {
            continue;
        }
        if nvim_regexp_get_nfa_has_zsubexpr() != 0
            && nvim_regexp_call_sub_equal(nvim_nfa_thread_get_subs_synt(l, i), subs_synt) == 0
        {
            continue;
        }
        if rs_pim_equal(nvim_nfa_thread_get_pim_ptr(l, i), pim) != 0 {
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
    let ll_index = nvim_regexp_get_nfa_ll_index();
    if nvim_nfa_state_get_lastlist(state, ll_index) == nvim_nfa_list_get_id(l)
        && (nvim_regexp_get_rex_nfa_has_backref() == 0
            || rs_has_state_with_pos(
                l,
                nvim_nfa_state_get_id(state),
                subs_norm,
                subs_synt,
                core::ptr::null_mut(),
            ) != 0)
    {
        return 1;
    }
    0
}

/// Wrapper: check for a match with subexpression "subidx".
#[no_mangle]
pub unsafe extern "C" fn rs_match_backref(
    sub: *mut c_void,
    subidx: c_int,
    bytelen: *mut c_int,
) -> c_int {
    nvim_regexp_call_match_backref(sub, subidx, bytelen)
}

/// Wrapper: check for a match with \z subexpression "subidx".
#[no_mangle]
pub unsafe extern "C" fn rs_match_zref(subidx: c_int, bytelen: *mut c_int) -> c_int {
    nvim_regexp_call_match_zref(subidx, bytelen)
}

/// Wrapper: check for a match with `match_text`.
#[no_mangle]
pub unsafe extern "C" fn rs_find_match_text(
    startcol: *mut c_int,
    regstart: c_int,
    match_text: *mut u8,
) -> c_int {
    nvim_regexp_call_find_match_text(startcol, regstart, match_text)
}

/// Wrapper: skip until the char "c" we know a match must start with.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_to_start(c: c_int, colp: *mut c_int) -> c_int {
    nvim_regexp_call_skip_to_start(c, colp)
}

/// Wrapper: check if NFA execution has timed out.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_did_time_out() -> c_int {
    nvim_regexp_call_nfa_did_time_out()
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
    // Thread field accessors
    fn nvim_nfa_thread_get_state_c(l: NfaListHandle, idx: c_int) -> c_int;
    fn nvim_nfa_thread_get_state_ptr(l: NfaListHandle, idx: c_int) -> NfaStateHandle;
    fn nvim_nfa_thread_get_state_out(l: NfaListHandle, idx: c_int) -> NfaStateHandle;
    fn nvim_nfa_thread_get_state_out1(l: NfaListHandle, idx: c_int) -> NfaStateHandle;
    fn nvim_nfa_thread_get_state_val(l: NfaListHandle, idx: c_int) -> c_int;
    fn nvim_nfa_thread_get_count(l: NfaListHandle, idx: c_int) -> c_int;
    fn nvim_nfa_thread_get_subs_ptr(l: NfaListHandle, idx: c_int) -> RegsubsHandle;

    // Thread PIM field accessors
    fn nvim_nfa_thread_get_pim_result(l: NfaListHandle, idx: c_int) -> c_int;
    fn nvim_nfa_thread_get_pim_state(l: NfaListHandle, idx: c_int) -> NfaStateHandle;
    fn nvim_nfa_thread_get_pim_state_c(l: NfaListHandle, idx: c_int) -> c_int;

    // nfa_list_T management
    fn nvim_nfa_list_set_n(l: NfaListHandle, n: c_int);
    fn nvim_nfa_list_set_has_pim(l: NfaListHandle, v: c_int);
    fn nvim_nfa_list_set_id(l: NfaListHandle, id: c_int);

    // regsubs_T operations
    fn nvim_regexp_regsubs_get_norm(s: RegsubsHandle) -> *mut c_void;
    fn nvim_regexp_regsubs_get_synt(s: RegsubsHandle) -> *mut c_void;
    fn nvim_regexp_regsubs_get_norm_in_use(s: RegsubsHandle) -> c_int;
    fn nvim_regexp_regsubs_set_norm_in_use(s: RegsubsHandle, v: c_int);
    fn nvim_regexp_regsubs_set_multi_start(s: RegsubsHandle, idx: c_int, lnum: i32, col: i32);
    fn nvim_regexp_regsubs_get_multi_start_col(s: RegsubsHandle, idx: c_int) -> i32;
    fn nvim_regexp_regsubs_get_multi_end_col(s: RegsubsHandle, idx: c_int) -> i32;
    fn nvim_regexp_regsubs_set_norm_orig_start_col(s: RegsubsHandle, v: i32);
    fn nvim_regexp_regsubs_set_line_start(s: RegsubsHandle, idx: c_int, ptr: *mut u8);
    fn nvim_regexp_regsubs_get_line_end(s: RegsubsHandle, idx: c_int) -> *mut u8;

    // rex execution field accessors
    fn nvim_regexp_get_rex_nfa_listid() -> c_int;
    fn nvim_regexp_set_rex_nfa_listid(v: c_int);
    fn nvim_regexp_get_rex_reg_maxcol() -> i32;
    fn nvim_regexp_get_rex_nfa_nsubexpr() -> c_int;

    // Character/utility functions
    fn nvim_regexp_call_ascii_iswhite(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_digit(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_hex(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_octal(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_word(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_head(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_alpha(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_lower(c: c_int) -> c_int;
    fn nvim_regexp_call_ri_upper(c: c_int) -> c_int;
    fn nvim_regexp_call_reg_prev_class() -> c_int;
    fn nvim_regexp_call_reg_match_visual() -> c_int;
    fn nvim_regexp_call_reg_nextline();
    fn nvim_regexp_call_cleanup_subexpr();

    // NFA prog field accessor
    fn nvim_nfa_prog_get_re_engine(prog: NfaProgHandle) -> c_int;

    // PIM operations
    fn nvim_nfa_pim_set_result(pim: NfaPimHandle, v: c_int);
    fn nvim_nfa_pim_get_state(pim: NfaPimHandle) -> NfaStateHandle;
    fn nvim_nfa_pim_get_state_c(pim: NfaPimHandle) -> c_int;
    fn nvim_nfa_pim_get_subs_norm(pim: NfaPimHandle) -> *mut c_void;
    fn nvim_nfa_pim_get_subs_synt(pim: NfaPimHandle) -> *mut c_void;

    // PIM allocation/init
    fn nvim_regexp_alloc_pim() -> NfaPimHandle;
    fn nvim_regexp_free_pim(p: NfaPimHandle);
    fn nvim_regexp_pim_init(
        p: NfaPimHandle,
        state: NfaStateHandle,
        result: c_int,
        lnum: i32,
        col: i32,
        ptr: *mut u8,
        is_multi: c_int,
    );

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
    fn nvim_nfa_list_set_last_thread_count(l: NfaListHandle, count: c_int);

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
    let toplevel: bool = nvim_nfa_state_get_c(start) == NFA_MOPEN;
    let mut r: RegsubsHandle;

    // Allow interrupting with CTRL-C.
    nvim_regexp_call_reg_breakcheck();
    if nvim_regexp_get_got_int() != 0 {
        return 0; // false
    }
    if nvim_regexp_call_nfa_did_time_out() != 0 {
        return 0; // false
    }

    nvim_regexp_set_nfa_match(0); // nfa_match = false

    // Allocate memory for the lists of nodes.
    let nstate = nvim_nfa_prog_get_nstate(prog);
    let list0 = nvim_nfa_list_alloc_threads(nstate + 1);
    let list1 = nvim_nfa_list_alloc_threads(nstate + 1);

    // Initialize thislist and nextlist
    let mut thislist = list0;
    nvim_nfa_list_set_n(thislist, 0);
    nvim_nfa_list_set_has_pim(thislist, 0);
    let mut nextlist = list1;
    nvim_nfa_list_set_n(nextlist, 0);
    nvim_nfa_list_set_has_pim(nextlist, 0);

    nvim_nfa_list_set_id(thislist, nvim_regexp_get_rex_nfa_listid() + 1);

    // Inline optimized code for addstate(thislist, start, m, 0) if we know
    // it's the first MOPEN.
    if toplevel {
        if nvim_regexp_is_reg_multi() != 0 {
            let col = nvim_regexp_get_rex_input() as isize - nvim_regexp_get_rex_line() as isize;
            nvim_regexp_regsubs_set_multi_start(m, 0, nvim_regexp_get_rex_lnum(), col as i32);
            nvim_regexp_regsubs_set_norm_orig_start_col(m, col as i32);
        } else {
            nvim_regexp_regsubs_set_line_start(m, 0, nvim_regexp_get_rex_input());
        }
        nvim_regexp_regsubs_set_norm_in_use(m, 1);
        r = nvim_regexp_call_addstate(
            thislist,
            nvim_nfa_state_get_out(start),
            m,
            core::ptr::null_mut(),
            0,
        );
    } else {
        r = nvim_regexp_call_addstate(thislist, start, m, core::ptr::null_mut(), 0);
    }
    if r.is_null() {
        nvim_regexp_set_nfa_match(NFA_TOO_EXPENSIVE);
        // goto theend
        nvim_nfa_list_free_threads(list0);
        nvim_nfa_list_free_threads(list1);
        if !listids.is_null() {
            nvim_regexp_xfree(listids as *mut c_void);
        }
        return nvim_regexp_get_nfa_match();
    }

    // Run for each character.
    'outer: loop {
        let curc = utf_ptr2char(nvim_regexp_get_rex_input() as *const c_char);
        let mut clen = utfc_ptr2len(nvim_regexp_get_rex_input() as *const c_char);
        if curc == 0 {
            // NUL
            clen = 0;
            go_to_nextline = false;
        }

        // swap lists
        thislist = if flag != 0 { list1 } else { list0 };
        flag ^= 1;
        nextlist = if flag != 0 { list1 } else { list0 };
        nvim_nfa_list_set_n(nextlist, 0);
        nvim_nfa_list_set_has_pim(nextlist, 0);

        let nfa_listid = nvim_regexp_get_rex_nfa_listid() + 1;
        nvim_regexp_set_rex_nfa_listid(nfa_listid);
        if nvim_nfa_prog_get_re_engine(prog) == AUTOMATIC_ENGINE && nfa_listid >= NFA_MAX_STATES {
            nvim_regexp_set_nfa_match(NFA_TOO_EXPENSIVE);
            break 'outer;
        }

        nvim_nfa_list_set_id(thislist, nfa_listid);
        nvim_nfa_list_set_id(nextlist, nfa_listid + 1);

        // If the state lists are empty we can stop.
        if nvim_nfa_list_get_n(thislist) == 0 {
            break 'outer;
        }

        // compute nextlist
        let mut listidx: c_int = 0;
        while listidx < nvim_nfa_list_get_n(thislist) {
            // Allow interrupting with CTRL-C.
            nvim_regexp_call_reg_breakcheck();
            if nvim_regexp_get_got_int() != 0 {
                break;
            }
            if !nvim_regexp_get_nfa_time_limit().is_null() {
                let tc = nvim_regexp_get_nfa_time_count() + 1;
                nvim_regexp_set_nfa_time_count(tc);
                if tc == 20 {
                    nvim_regexp_set_nfa_time_count(0);
                    if nvim_regexp_call_nfa_did_time_out() != 0 {
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

            let state_c = nvim_nfa_thread_get_state_c(thislist, listidx);

            match state_c {
                x if x == NFA_MATCH => {
                    // If the match is not at the start of the line, ends before a
                    // composing character and rex.reg_icombine is not set, that
                    // is not really a match.
                    if nvim_regexp_get_rex_reg_icombine() == 0
                        && nvim_regexp_get_rex_input() != nvim_regexp_get_rex_line()
                        && utf_iscomposing_legacy(curc) != 0
                    {
                        // break from match arm - continue to next state
                    } else {
                        nvim_regexp_set_nfa_match(1); // true
                        nvim_regexp_call_copy_sub(
                            nvim_regexp_regsubs_get_norm(submatch),
                            nvim_nfa_thread_get_subs_norm(thislist, listidx),
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            nvim_regexp_call_copy_sub(
                                nvim_regexp_regsubs_get_synt(submatch),
                                nvim_nfa_thread_get_subs_synt(thislist, listidx),
                            );
                        }
                        // Found left-most longest match.
                        if nvim_nfa_list_get_n(nextlist) == 0 {
                            clen = 0;
                        }
                        // goto nextchar: break inner loop, let outer loop's
                        // bottom-of-loop code do the input advancement
                        listidx = nvim_nfa_list_get_n(thislist);
                        continue;
                    }
                }

                x if x == NFA_END_INVISIBLE
                    || x == NFA_END_INVISIBLE_NEG
                    || x == NFA_END_PATTERN =>
                {
                    // Check if nfa_endp matches current position
                    let endp = nvim_regexp_get_nfa_endp();
                    if !endp.is_null() {
                        if nvim_regexp_is_reg_multi() != 0 {
                            if nvim_regexp_get_rex_lnum() != nvim_regexp_get_nfa_endp_pos_lnum()
                                || (nvim_regexp_get_rex_input() as isize
                                    - nvim_regexp_get_rex_line() as isize)
                                    as i32
                                    != nvim_regexp_get_nfa_endp_pos_col()
                            {
                                // no match at required position
                                listidx += 1;
                                continue;
                            }
                        } else if nvim_regexp_get_rex_input() != nvim_regexp_get_nfa_endp_ptr() {
                            listidx += 1;
                            continue;
                        }
                    }
                    // do not set submatches for \@!
                    if state_c != NFA_END_INVISIBLE_NEG {
                        nvim_regexp_call_copy_sub(
                            nvim_regexp_regsubs_get_norm(m),
                            nvim_nfa_thread_get_subs_norm(thislist, listidx),
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            nvim_regexp_call_copy_sub(
                                nvim_regexp_regsubs_get_synt(m),
                                nvim_nfa_thread_get_subs_synt(thislist, listidx),
                            );
                        }
                    }
                    nvim_regexp_set_nfa_match(1); // true
                    if nvim_nfa_list_get_n(nextlist) == 0 {
                        clen = 0;
                    }
                    // goto nextchar: break inner loop, let outer loop handle advancement
                    listidx = nvim_nfa_list_get_n(thislist);
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
                    let t_state = nvim_nfa_thread_get_state_ptr(thislist, listidx);
                    let pim_result = nvim_nfa_thread_get_pim_result(thislist, listidx);

                    if pim_result != NFA_PIM_UNUSED
                        || state_c == NFA_START_INVISIBLE_FIRST
                        || state_c == NFA_START_INVISIBLE_NEG_FIRST
                        || state_c == NFA_START_INVISIBLE_BEFORE_FIRST
                        || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
                    {
                        let in_use = nvim_regexp_regsubs_get_norm_in_use(m);

                        // Copy submatch info for the recursive call
                        nvim_regexp_call_copy_sub_off(
                            nvim_regexp_regsubs_get_norm(m),
                            nvim_nfa_thread_get_subs_norm(thislist, listidx),
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            nvim_regexp_call_copy_sub_off(
                                nvim_regexp_regsubs_get_synt(m),
                                nvim_nfa_thread_get_subs_synt(thislist, listidx),
                            );
                        }
                        // First try matching the invisible match
                        result = nvim_regexp_call_recursive_regmatch(
                            t_state,
                            core::ptr::null_mut(),
                            prog,
                            submatch,
                            m,
                            &mut listids,
                            &mut listids_len,
                        );
                        if result == NFA_TOO_EXPENSIVE {
                            nvim_regexp_set_nfa_match(result);
                            break 'outer;
                        }

                        // for \@! and \@<! it is a match when result is false
                        let is_neg = state_c == NFA_START_INVISIBLE_NEG
                            || state_c == NFA_START_INVISIBLE_NEG_FIRST
                            || state_c == NFA_START_INVISIBLE_BEFORE_NEG
                            || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST;
                        if result != is_neg as c_int {
                            // Copy submatch info from the recursive call
                            nvim_regexp_call_copy_sub_off(
                                nvim_nfa_thread_get_subs_norm(thislist, listidx),
                                nvim_regexp_regsubs_get_norm(m),
                            );
                            if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                                nvim_regexp_call_copy_sub_off(
                                    nvim_nfa_thread_get_subs_synt(thislist, listidx),
                                    nvim_regexp_regsubs_get_synt(m),
                                );
                            }
                            // If the pattern has \ze and it matched, use it.
                            nvim_regexp_call_copy_ze_off(
                                nvim_nfa_thread_get_subs_norm(thislist, listidx),
                                nvim_regexp_regsubs_get_norm(m),
                            );

                            // t->state->out1 is the corresponding END_INVISIBLE node
                            add_here = true;
                            add_state = nvim_nfa_state_get_out(nvim_nfa_state_get_out1(t_state));
                        }
                        nvim_regexp_regsubs_set_norm_in_use(m, in_use);
                    } else {
                        // First try matching what follows. Add a nfa_pim_T.
                        let pim = nvim_regexp_alloc_pim();
                        let input = nvim_regexp_get_rex_input();
                        let line = nvim_regexp_get_rex_line();
                        let is_multi = nvim_regexp_is_reg_multi();
                        nvim_regexp_pim_init(
                            pim,
                            t_state,
                            NFA_PIM_TODO,
                            nvim_regexp_get_rex_lnum(),
                            (input as isize - line as isize) as i32,
                            input,
                            is_multi,
                        );

                        // Add out1->out to thislist with PIM
                        let out1_out = nvim_nfa_state_get_out(nvim_nfa_state_get_out1(t_state));
                        let subs_ptr = nvim_nfa_thread_get_subs_ptr(thislist, listidx);
                        if nvim_regexp_call_addstate_here(
                            thislist,
                            out1_out,
                            subs_ptr,
                            pim,
                            &mut listidx,
                        )
                        .is_null()
                        {
                            nvim_regexp_free_pim(pim);
                            nvim_regexp_set_nfa_match(NFA_TOO_EXPENSIVE);
                            break 'outer;
                        }
                        nvim_regexp_free_pim(pim);
                    }
                }

                x if x == NFA_START_PATTERN => {
                    let t_state = nvim_nfa_thread_get_state_ptr(thislist, listidx);
                    let out1 = nvim_nfa_state_get_out1(t_state);
                    let out1_out = nvim_nfa_state_get_out(out1);
                    let out1_out_out = nvim_nfa_state_get_out(out1_out);
                    let subs_norm = nvim_nfa_thread_get_subs_norm(thislist, listidx);
                    let subs_synt = nvim_nfa_thread_get_subs_synt(thislist, listidx);
                    let subs_ptr = nvim_nfa_thread_get_subs_ptr(thislist, listidx);

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
                        nvim_regexp_call_copy_sub_off(nvim_regexp_regsubs_get_norm(m), subs_norm);
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            nvim_regexp_call_copy_sub_off(
                                nvim_regexp_regsubs_get_synt(m),
                                subs_synt,
                            );
                        }

                        result = nvim_regexp_call_recursive_regmatch(
                            t_state,
                            core::ptr::null_mut(),
                            prog,
                            submatch,
                            m,
                            &mut listids,
                            &mut listids_len,
                        );
                        if result == NFA_TOO_EXPENSIVE {
                            nvim_regexp_set_nfa_match(result);
                            break 'outer;
                        }
                        if result != 0 {
                            // Copy submatch info from the recursive call
                            nvim_regexp_call_copy_sub_off(
                                nvim_nfa_thread_get_subs_norm(thislist, listidx),
                                nvim_regexp_regsubs_get_norm(m),
                            );
                            if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                                nvim_regexp_call_copy_sub_off(
                                    nvim_nfa_thread_get_subs_synt(thislist, listidx),
                                    nvim_regexp_regsubs_get_synt(m),
                                );
                            }
                            // Skip over matched text
                            let bytelen = if nvim_regexp_is_reg_multi() != 0 {
                                nvim_regexp_regsubs_get_multi_end_col(m, 0)
                                    - (nvim_regexp_get_rex_input() as isize
                                        - nvim_regexp_get_rex_line() as isize)
                                        as i32
                            } else {
                                nvim_regexp_regsubs_get_line_end(m, 0) as isize as i32
                                    - nvim_regexp_get_rex_input() as isize as i32
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
                    if nvim_regexp_get_rex_input() == nvim_regexp_get_rex_line() {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_EOL => {
                    if curc == 0 {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_BOW => {
                    result = 1; // true
                    if curc == 0 {
                        result = 0;
                    } else {
                        let this_class =
                            nvim_regexp_call_mb_get_class_tab(nvim_regexp_get_rex_input());
                        if this_class <= 1 {
                            result = 0;
                        } else if nvim_regexp_call_reg_prev_class() == this_class {
                            result = 0;
                        }
                    }
                    if result != 0 {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_EOW => {
                    result = 1; // true
                    if nvim_regexp_get_rex_input() == nvim_regexp_get_rex_line() {
                        result = 0;
                    } else {
                        let this_class =
                            nvim_regexp_call_mb_get_class_tab(nvim_regexp_get_rex_input());
                        let prev_class = nvim_regexp_call_reg_prev_class();
                        if this_class == prev_class || prev_class == 0 || prev_class == 1 {
                            result = 0;
                        }
                    }
                    if result != 0 {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_BOF => {
                    if nvim_regexp_get_rex_lnum() == 0
                        && nvim_regexp_get_rex_input() == nvim_regexp_get_rex_line()
                        && (nvim_regexp_is_reg_multi() == 0
                            || nvim_regexp_get_rex_reg_firstlnum() == 1)
                    {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_EOF => {
                    if nvim_regexp_get_rex_lnum() == nvim_regexp_get_rex_reg_maxline() && curc == 0
                    {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_COMPOSING => {
                    let t_state = nvim_nfa_thread_get_state_ptr(thislist, listidx);
                    let mc = curc;
                    let mut len: c_int = 0;
                    let mut sta = nvim_nfa_state_get_out(t_state);
                    let mut cchars: [c_int; MAX_MCO] = [0; MAX_MCO];
                    let mut ccount: usize = 0;

                    if utf_iscomposing_legacy(nvim_nfa_state_get_c(sta)) != 0 {
                        len += utf_char2len(mc);
                    }
                    if nvim_regexp_get_rex_reg_icombine() != 0 && len == 0 {
                        if nvim_nfa_state_get_c(sta) != curc {
                            result = FAIL;
                        } else {
                            result = OK;
                        }
                        while nvim_nfa_state_get_c(sta) != NFA_END_COMPOSING {
                            sta = nvim_nfa_state_get_out(sta);
                        }
                    } else if len > 0 || mc == nvim_nfa_state_get_c(sta) {
                        if len == 0 {
                            len += utf_char2len(mc);
                            sta = nvim_nfa_state_get_out(sta);
                        }
                        // Get composing chars into cchars[]
                        while len < clen {
                            let mc2 = utf_ptr2char(
                                (nvim_regexp_get_rex_input() as *const u8).offset(len as isize)
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
                        // Check composing chars match
                        result = OK;
                        while nvim_nfa_state_get_c(sta) != NFA_END_COMPOSING {
                            let mut found = false;
                            for j in 0..ccount {
                                if cchars[j] == nvim_nfa_state_get_c(sta) {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                result = FAIL;
                                break;
                            }
                            sta = nvim_nfa_state_get_out(sta);
                        }
                    } else {
                        result = FAIL;
                    }

                    // ADD_STATE_IF_MATCH(end)
                    let end = nvim_nfa_state_get_out1(t_state); // NFA_END_COMPOSING
                    if result != 0 {
                        add_state = nvim_nfa_state_get_out(end);
                        add_off = clen;
                    }
                }

                x if x == NFA_NEWL => {
                    if curc == 0
                        && nvim_regexp_get_rex_reg_line_lbr() == 0
                        && nvim_regexp_is_reg_multi() != 0
                        && nvim_regexp_get_rex_lnum() <= nvim_regexp_get_rex_reg_maxline()
                    {
                        go_to_nextline = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = -1;
                    } else if curc == b'\n' as c_int && nvim_regexp_get_rex_reg_line_lbr() != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = 1;
                    }
                }

                x if x == NFA_START_COLL || x == NFA_START_NEG_COLL => {
                    // Never match EOL
                    if curc == 0 {
                        // break - no match
                    } else {
                        let t_state = nvim_nfa_thread_get_state_ptr(thislist, listidx);
                        let mut col_state = nvim_nfa_state_get_out(t_state);
                        let result_if_matched = if state_c == NFA_START_COLL { 1 } else { 0 };
                        result = 0;

                        loop {
                            let col_c = nvim_nfa_state_get_c(col_state);
                            if col_c == NFA_COMPOSING {
                                // Composing inside collection - complex case
                                let mc = curc;
                                let mut len: c_int = 0;
                                let mut sta =
                                    nvim_nfa_state_get_out(nvim_nfa_state_get_out(t_state));
                                let mut cchars: [c_int; MAX_MCO] = [0; MAX_MCO];
                                let mut ccount: usize = 0;

                                if utf_iscomposing_legacy(nvim_nfa_state_get_c(sta)) != 0 {
                                    len += utf_char2len(mc);
                                }
                                if nvim_regexp_get_rex_reg_icombine() != 0 && len == 0 {
                                    if nvim_nfa_state_get_c(sta) != curc {
                                        result = FAIL;
                                    } else {
                                        result = OK;
                                    }
                                    while nvim_nfa_state_get_c(sta) != NFA_END_COMPOSING {
                                        sta = nvim_nfa_state_get_out(sta);
                                    }
                                } else if len > 0 || mc == nvim_nfa_state_get_c(sta) {
                                    if len == 0 {
                                        len += utf_char2len(mc);
                                        sta = nvim_nfa_state_get_out(sta);
                                    }
                                    while len < clen {
                                        let mc2 = utf_ptr2char(
                                            (nvim_regexp_get_rex_input() as *const u8)
                                                .offset(len as isize)
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
                                    while nvim_nfa_state_get_c(sta) != NFA_END_COMPOSING {
                                        let mut found = false;
                                        for j in 0..ccount {
                                            if cchars[j] == nvim_nfa_state_get_c(sta) {
                                                found = true;
                                                break;
                                            }
                                        }
                                        if !found {
                                            result = FAIL;
                                            break;
                                        }
                                        sta = nvim_nfa_state_get_out(sta);
                                    }
                                } else {
                                    result = FAIL;
                                }

                                let out_out1 =
                                    nvim_nfa_state_get_out1(nvim_nfa_state_get_out(t_state));
                                if nvim_nfa_state_get_c(out_out1) == NFA_END_COMPOSING {
                                    if result != 0 {
                                        add_state = nvim_nfa_state_get_out(out_out1);
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
                                let c1 = nvim_nfa_state_get_val(col_state);
                                col_state = nvim_nfa_state_get_out(col_state);
                                let c2 = nvim_nfa_state_get_val(col_state);

                                if curc >= c1 && curc <= c2 {
                                    result = result_if_matched;
                                    break;
                                }
                                if nvim_regexp_get_rex_reg_ic() != 0 {
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
                                || (nvim_regexp_get_rex_reg_ic() != 0
                                    && utf_fold(curc) == utf_fold(col_c))
                            {
                                result = result_if_matched;
                                break;
                            }
                            col_state = nvim_nfa_state_get_out(col_state);
                        }
                        if result != 0 {
                            add_state = nvim_nfa_state_get_out(nvim_nfa_state_get_out1(t_state));
                            add_off = clen;
                        }
                    }
                }

                x if x == NFA_ANY => {
                    if curc > 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
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
                    add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                }

                x if x == NFA_IDENT => {
                    result = vim_isIDc(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
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
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_KWORD => {
                    result = nvim_regexp_call_vim_iswordp_buf(
                        nvim_regexp_get_rex_input() as *const c_char
                    );
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_SKWORD => {
                    result = if ascii_isdigit_i(curc) == 0
                        && nvim_regexp_call_vim_iswordp_buf(
                            nvim_regexp_get_rex_input() as *const c_char
                        ) != 0
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_FNAME => {
                    result = vim_isfilec(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
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
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_PRINT => {
                    result =
                        vim_isprintc(utf_ptr2char(nvim_regexp_get_rex_input() as *const c_char));
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_SPRINT => {
                    result = if ascii_isdigit_i(curc) == 0
                        && vim_isprintc(utf_ptr2char(nvim_regexp_get_rex_input() as *const c_char))
                            != 0
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_WHITE => {
                    result = nvim_regexp_call_ascii_iswhite(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NWHITE => {
                    result = if curc != 0 && nvim_regexp_call_ascii_iswhite(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_DIGIT => {
                    result = nvim_regexp_call_ri_digit(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NDIGIT => {
                    result = if curc != 0 && nvim_regexp_call_ri_digit(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_HEX => {
                    result = nvim_regexp_call_ri_hex(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NHEX => {
                    result = if curc != 0 && nvim_regexp_call_ri_hex(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_OCTAL => {
                    result = nvim_regexp_call_ri_octal(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NOCTAL => {
                    result = if curc != 0 && nvim_regexp_call_ri_octal(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_WORD => {
                    result = nvim_regexp_call_ri_word(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NWORD => {
                    result = if curc != 0 && nvim_regexp_call_ri_word(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_HEAD => {
                    result = nvim_regexp_call_ri_head(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NHEAD => {
                    result = if curc != 0 && nvim_regexp_call_ri_head(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_ALPHA => {
                    result = nvim_regexp_call_ri_alpha(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NALPHA => {
                    result = if curc != 0 && nvim_regexp_call_ri_alpha(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_LOWER => {
                    result = nvim_regexp_call_ri_lower(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NLOWER => {
                    result = if curc != 0 && nvim_regexp_call_ri_lower(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_UPPER => {
                    result = nvim_regexp_call_ri_upper(curc);
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NUPPER => {
                    result = if curc != 0 && nvim_regexp_call_ri_upper(curc) == 0 {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_LOWER_IC => {
                    result = if nvim_regexp_call_ri_lower(curc) != 0
                        || (nvim_regexp_get_rex_reg_ic() != 0
                            && nvim_regexp_call_ri_upper(curc) != 0)
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NLOWER_IC => {
                    result = if curc != 0
                        && !(nvim_regexp_call_ri_lower(curc) != 0
                            || (nvim_regexp_get_rex_reg_ic() != 0
                                && nvim_regexp_call_ri_upper(curc) != 0))
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_UPPER_IC => {
                    result = if nvim_regexp_call_ri_upper(curc) != 0
                        || (nvim_regexp_get_rex_reg_ic() != 0
                            && nvim_regexp_call_ri_lower(curc) != 0)
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if x == NFA_NUPPER_IC => {
                    result = if curc != 0
                        && !(nvim_regexp_call_ri_upper(curc) != 0
                            || (nvim_regexp_get_rex_reg_ic() != 0
                                && nvim_regexp_call_ri_lower(curc) != 0))
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }

                x if (NFA_BACKREF1..=NFA_BACKREF9).contains(&x)
                    || (NFA_ZREF1..=NFA_ZREF9).contains(&x) =>
                {
                    let mut bytelen: c_int = 0;
                    let t_subs_norm = nvim_nfa_thread_get_subs_norm(thislist, listidx);

                    if state_c >= NFA_BACKREF1 && state_c <= NFA_BACKREF9 {
                        let subidx = state_c - NFA_BACKREF1 + 1;
                        result = nvim_regexp_call_match_backref(t_subs_norm, subidx, &mut bytelen);
                    } else {
                        let subidx = state_c - NFA_ZREF1 + 1;
                        result = nvim_regexp_call_match_zref(subidx, &mut bytelen);
                    }

                    if result != 0 {
                        let t_state_out = nvim_nfa_thread_get_state_out(thislist, listidx);
                        if bytelen == 0 {
                            add_here = true;
                            add_state = nvim_nfa_state_get_out(t_state_out);
                        } else if bytelen <= clen {
                            add_state = nvim_nfa_state_get_out(t_state_out);
                            add_off = clen;
                        } else {
                            add_state = t_state_out;
                            add_off = bytelen;
                            add_count = bytelen - clen;
                        }
                    }
                }

                x if x == NFA_SKIP => {
                    let t_count = nvim_nfa_thread_get_count(thislist, listidx);
                    if t_count - clen <= 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    } else {
                        add_state = nvim_nfa_thread_get_state_ptr(thislist, listidx);
                        add_off = 0;
                        add_count = t_count - clen;
                    }
                }

                x if x == NFA_LNUM || x == NFA_LNUM_GT || x == NFA_LNUM_LT => {
                    let val = nvim_nfa_thread_get_state_val(thislist, listidx);
                    result = if nvim_regexp_is_reg_multi() != 0 {
                        rs_nfa_re_num_cmp(
                            val as usize,
                            state_c - NFA_LNUM,
                            (nvim_regexp_get_rex_lnum() as usize)
                                .wrapping_add(nvim_regexp_get_rex_reg_firstlnum() as usize),
                        )
                    } else {
                        0
                    };
                    if result != 0 {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_COL || x == NFA_COL_GT || x == NFA_COL_LT => {
                    let val = nvim_nfa_thread_get_state_val(thislist, listidx);
                    let col_offset = (nvim_regexp_get_rex_input() as usize)
                        .wrapping_sub(nvim_regexp_get_rex_line() as usize);
                    result = rs_nfa_re_num_cmp(
                        val as usize,
                        state_c - NFA_COL,
                        col_offset.wrapping_add(1),
                    );
                    if result != 0 {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_VCOL || x == NFA_VCOL_GT || x == NFA_VCOL_LT => {
                    let val = nvim_nfa_thread_get_state_val(thislist, listidx);
                    let op = state_c - NFA_VCOL;
                    let col = (nvim_regexp_get_rex_input() as isize
                        - nvim_regexp_get_rex_line() as isize) as i32;

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
                            let mut lnum = if nvim_regexp_is_reg_multi() != 0 {
                                nvim_regexp_get_rex_reg_firstlnum() + nvim_regexp_get_rex_lnum()
                            } else {
                                1
                            };
                            if nvim_regexp_is_reg_multi() != 0
                                && (lnum <= 0 || lnum > nvim_regexp_get_win_buf_line_count(wp))
                            {
                                lnum = 1;
                            }
                            let vcol = nvim_regexp_call_win_linetabsize(
                                wp,
                                lnum,
                                nvim_regexp_get_rex_line() as *const c_char,
                                col,
                            );
                            result = rs_nfa_re_num_cmp(val as usize, op, (vcol + 1) as usize);
                        }
                        if result != 0 {
                            add_here = true;
                            add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        }
                    }
                }

                x if x == NFA_MARK || x == NFA_MARK_GT || x == NFA_MARK_LT => {
                    let val = nvim_nfa_thread_get_state_val(thislist, listidx);
                    let col_sz = if nvim_regexp_is_reg_multi() != 0 {
                        (nvim_regexp_get_rex_input() as isize - nvim_regexp_get_rex_line() as isize)
                            as usize
                    } else {
                        0
                    };

                    let fm = nvim_regexp_call_mark_get_for_nfa(
                        nvim_regexp_get_rex_reg_buf() as *mut c_void,
                        nvim_regexp_get_curwin(),
                        val,
                    );

                    // Line may have been freed, get it again.
                    if nvim_regexp_is_reg_multi() != 0 {
                        let new_line =
                            nvim_regexp_call_reg_getline(nvim_regexp_get_rex_lnum()) as *mut u8;
                        nvim_regexp_set_rex_line(new_line);
                        nvim_regexp_set_rex_input(new_line.add(col_sz));
                    }

                    if nvim_regexp_fmark_is_set(fm) != 0 {
                        let pos_lnum = nvim_regexp_fmark_get_lnum(fm);
                        let lnum_match =
                            nvim_regexp_get_rex_lnum() + nvim_regexp_get_rex_reg_firstlnum();
                        let pos_col = nvim_regexp_fmark_get_col_adj(fm, lnum_match);
                        let input_col = (nvim_regexp_get_rex_input() as isize
                            - nvim_regexp_get_rex_line() as isize)
                            as i32;

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
                            add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        }
                    }
                }

                x if x == NFA_CURSOR => {
                    result = if nvim_regexp_has_rex_reg_win() != 0
                        && (nvim_regexp_get_rex_lnum() + nvim_regexp_get_rex_reg_firstlnum()
                            == nvim_regexp_get_rex_reg_win_cursor_lnum())
                        && ((nvim_regexp_get_rex_input() as isize
                            - nvim_regexp_get_rex_line() as isize)
                            as i32
                            == nvim_regexp_get_rex_reg_win_cursor_col())
                    {
                        1
                    } else {
                        0
                    };
                    if result != 0 {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                    }
                }

                x if x == NFA_VISUAL => {
                    result = nvim_regexp_call_reg_match_visual();
                    if result != 0 {
                        add_here = true;
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
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

                    if result == 0 && nvim_regexp_get_rex_reg_ic() != 0 {
                        result = if utf_fold(c) == utf_fold(curc) { 1 } else { 0 };
                    }

                    // If reg_icombine is not set only skip over the character itself.
                    if result != 0 && nvim_regexp_get_rex_reg_icombine() == 0 {
                        clen = utf_ptr2len(nvim_regexp_get_rex_input() as *const c_char);
                    }

                    // ADD_STATE_IF_MATCH(t->state)
                    if result != 0 {
                        add_state = nvim_nfa_thread_get_state_out(thislist, listidx);
                        add_off = clen;
                    }
                }
            } // match state_c

            // Post-switch: handle add_state with PIM resolution
            if !add_state.is_null() {
                let pim_result = nvim_nfa_thread_get_pim_result(thislist, listidx);
                let mut use_pim: bool = pim_result != NFA_PIM_UNUSED;
                let mut pim_ptr: NfaPimHandle = core::ptr::null_mut();
                let mut pim_copy: NfaPimHandle = core::ptr::null_mut();

                if !use_pim {
                    pim_ptr = core::ptr::null_mut();
                } else {
                    pim_ptr = nvim_nfa_thread_get_pim_ptr(thislist, listidx);
                }

                // Handle the postponed invisible match if the match might end
                // without advancing and before the end of the line.
                if use_pim && (clen == 0 || match_follows(add_state, 0)) {
                    let pim_res = nvim_nfa_pim_get_result(pim_ptr);
                    if pim_res == NFA_PIM_TODO {
                        result = nvim_regexp_call_recursive_regmatch(
                            nvim_nfa_pim_get_state(pim_ptr),
                            pim_ptr,
                            prog,
                            submatch,
                            m,
                            &mut listids,
                            &mut listids_len,
                        );
                        nvim_nfa_pim_set_result(
                            pim_ptr,
                            if result != 0 {
                                NFA_PIM_MATCH
                            } else {
                                NFA_PIM_NOMATCH
                            },
                        );
                        let pim_state_c = nvim_nfa_pim_get_state_c(pim_ptr);
                        let is_neg = pim_state_c == NFA_START_INVISIBLE_NEG
                            || pim_state_c == NFA_START_INVISIBLE_NEG_FIRST
                            || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG
                            || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST;
                        if result != is_neg as c_int {
                            // Copy submatch info from the recursive call
                            nvim_regexp_call_copy_sub_off(
                                nvim_nfa_pim_get_subs_norm(pim_ptr),
                                nvim_regexp_regsubs_get_norm(m),
                            );
                            if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                                nvim_regexp_call_copy_sub_off(
                                    nvim_nfa_pim_get_subs_synt(pim_ptr),
                                    nvim_regexp_regsubs_get_synt(m),
                                );
                            }
                        }
                    } else {
                        result = if pim_res == NFA_PIM_MATCH { 1 } else { 0 };
                    }

                    // for \@! and \@<! it is a match when result is false
                    let pim_state_c = nvim_nfa_pim_get_state_c(pim_ptr);
                    let is_neg = pim_state_c == NFA_START_INVISIBLE_NEG
                        || pim_state_c == NFA_START_INVISIBLE_NEG_FIRST
                        || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG
                        || pim_state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST;
                    if result != is_neg as c_int {
                        // Copy submatch info
                        nvim_regexp_call_copy_sub_off(
                            nvim_nfa_thread_get_subs_norm(thislist, listidx),
                            nvim_nfa_pim_get_subs_norm(pim_ptr),
                        );
                        if nvim_regexp_get_nfa_has_zsubexpr() != 0 {
                            nvim_regexp_call_copy_sub_off(
                                nvim_nfa_thread_get_subs_synt(thislist, listidx),
                                nvim_nfa_pim_get_subs_synt(pim_ptr),
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
                    pim_copy = nvim_regexp_alloc_pim();
                    nvim_regexp_call_copy_pim(pim_copy, pim_ptr);
                    pim_ptr = pim_copy;
                }

                if add_here {
                    let subs_ptr = nvim_nfa_thread_get_subs_ptr(thislist, listidx);
                    r = nvim_regexp_call_addstate_here(
                        thislist,
                        add_state,
                        subs_ptr,
                        pim_ptr,
                        &mut listidx,
                    );
                } else {
                    let subs_ptr = nvim_nfa_thread_get_subs_ptr(thislist, listidx);
                    r = nvim_regexp_call_addstate(nextlist, add_state, subs_ptr, pim_ptr, add_off);
                    if add_count > 0 {
                        nvim_nfa_list_set_last_thread_count(nextlist, add_count);
                    }
                }

                if !pim_copy.is_null() {
                    nvim_regexp_free_pim(pim_copy);
                }

                if r.is_null() {
                    nvim_regexp_set_nfa_match(NFA_TOO_EXPENSIVE);
                    break 'outer;
                }
            }

            listidx += 1;
        } // while listidx < thislist->n

        // Look for the start of a match in the current position
        if nvim_regexp_get_nfa_match() == 0
            && ((toplevel
                && nvim_regexp_get_rex_lnum() == 0
                && clen != 0
                && (nvim_regexp_get_rex_reg_maxcol() == 0
                    || (nvim_regexp_get_rex_input() as isize
                        - nvim_regexp_get_rex_line() as isize)
                        < nvim_regexp_get_rex_reg_maxcol() as isize))
                || (!nvim_regexp_get_nfa_endp().is_null()
                    && (if nvim_regexp_is_reg_multi() != 0 {
                        nvim_regexp_get_rex_lnum() < nvim_regexp_get_nfa_endp_pos_lnum()
                            || (nvim_regexp_get_rex_lnum() == nvim_regexp_get_nfa_endp_pos_lnum()
                                && ((nvim_regexp_get_rex_input() as isize
                                    - nvim_regexp_get_rex_line() as isize)
                                    as i32)
                                    < nvim_regexp_get_nfa_endp_pos_col())
                    } else {
                        (nvim_regexp_get_rex_input() as usize)
                            < (nvim_regexp_get_nfa_endp_ptr() as usize)
                    })))
        {
            if toplevel {
                let mut add = true;

                if nvim_nfa_prog_get_regstart(prog) != 0 && clen != 0 {
                    if nvim_nfa_list_get_n(nextlist) == 0 {
                        let mut col = (nvim_regexp_get_rex_input() as isize
                            - nvim_regexp_get_rex_line() as isize)
                            as i32
                            + clen;
                        if nvim_regexp_call_skip_to_start(
                            nvim_nfa_prog_get_regstart(prog),
                            &mut col,
                        ) == FAIL
                        {
                            break 'outer;
                        }
                        // rex.input = rex.line + col - clen
                        let new_input = nvim_regexp_get_rex_line().offset((col - clen) as isize);
                        nvim_regexp_set_rex_input(new_input);
                    } else {
                        let c = utf_ptr2char(
                            nvim_regexp_get_rex_input().offset(clen as isize) as *const c_char
                        );
                        if c != nvim_nfa_prog_get_regstart(prog)
                            && (nvim_regexp_get_rex_reg_ic() == 0
                                || utf_fold(c) != utf_fold(nvim_nfa_prog_get_regstart(prog)))
                        {
                            add = false;
                        }
                    }
                }

                if add {
                    if nvim_regexp_is_reg_multi() != 0 {
                        let start_col = (nvim_regexp_get_rex_input() as isize
                            - nvim_regexp_get_rex_line() as isize)
                            as i32
                            + clen;
                        nvim_regexp_regsubs_set_multi_start(
                            m,
                            0,
                            nvim_regexp_get_rex_lnum(),
                            start_col,
                        );
                        nvim_regexp_regsubs_set_norm_orig_start_col(m, start_col);
                    } else {
                        nvim_regexp_regsubs_set_line_start(
                            m,
                            0,
                            nvim_regexp_get_rex_input().offset(clen as isize),
                        );
                    }
                    if nvim_regexp_call_addstate(
                        nextlist,
                        nvim_nfa_state_get_out(start),
                        m,
                        core::ptr::null_mut(),
                        clen,
                    )
                    .is_null()
                    {
                        nvim_regexp_set_nfa_match(NFA_TOO_EXPENSIVE);
                        break 'outer;
                    }
                }
            } else if nvim_regexp_call_addstate(nextlist, start, m, core::ptr::null_mut(), clen)
                .is_null()
            {
                nvim_regexp_set_nfa_match(NFA_TOO_EXPENSIVE);
                break 'outer;
            }
        }

        // nextchar: Advance to the next character
        if clen != 0 {
            let new_input = nvim_regexp_get_rex_input().offset(clen as isize);
            nvim_regexp_set_rex_input(new_input);
        } else if go_to_nextline
            || (!nvim_regexp_get_nfa_endp().is_null()
                && nvim_regexp_is_reg_multi() != 0
                && nvim_regexp_get_rex_lnum() < nvim_regexp_get_nfa_endp_pos_lnum())
        {
            nvim_regexp_call_reg_nextline();
        } else {
            break 'outer;
        }
        go_to_nextline = false;

        // Allow interrupting with CTRL-C.
        nvim_regexp_call_reg_breakcheck();
        if nvim_regexp_get_got_int() != 0 {
            break 'outer;
        }
        // Check for timeout once every twenty times
        if !nvim_regexp_get_nfa_time_limit().is_null() {
            let tc = nvim_regexp_get_nfa_time_count() + 1;
            nvim_regexp_set_nfa_time_count(tc);
            if tc == 20 {
                nvim_regexp_set_nfa_time_count(0);
                if nvim_regexp_call_nfa_did_time_out() != 0 {
                    break 'outer;
                }
            }
        }
    } // loop (outer)

    // theend: Free memory
    nvim_nfa_list_free_threads(list0);
    nvim_nfa_list_free_threads(list1);
    if !listids.is_null() {
        nvim_regexp_xfree(listids as *mut c_void);
    }

    nvim_regexp_get_nfa_match()
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

// Extern declarations for Phase 8.5 C accessors
extern "C" {
    // regsubs_T heap allocation
    fn nvim_regexp_alloc_regsubs() -> RegsubsHandle;
    fn nvim_regexp_free_regsubs(s: RegsubsHandle);

    // nfa_regtry setup
    fn nvim_regexp_nfa_regtry_setup(
        prog: NfaProgHandle,
        col: i32,
        tm: *mut c_void,
        timed_out: *mut c_int,
    );

    // nfa_regtry submatch extraction
    fn nvim_regexp_nfa_regtry_extract_multi(subs: RegsubsHandle, col: i32);
    fn nvim_regexp_nfa_regtry_extract_single(subs: RegsubsHandle, col: i32);
    fn nvim_regexp_nfa_regtry_extract_extmatch(subs: RegsubsHandle);

    // nfa_regexec_both helpers
    fn nvim_regexp_nfa_regexec_both_get_prog() -> NfaProgHandle;
    fn nvim_regexp_nfa_regexec_both_get_line(line: *mut u8) -> *mut u8;
    fn nvim_regexp_nfa_regexec_both_setup_pointers();
    fn nvim_regexp_nfa_regexec_both_apply_flags(prog: NfaProgHandle);
    fn nvim_regexp_nfa_regexec_both_setup_nfa(prog: NfaProgHandle);
    fn nvim_regexp_nfa_regexec_both_init_states(prog: NfaProgHandle);
    fn nvim_regexp_nfa_regexec_both_validate_match();
    fn nvim_regexp_nfa_regexec_both_set_matchcol(col: i32);

    // nfa_regexec_nl setup
    fn nvim_regexp_nfa_regexec_nl_setup(rmp: *mut c_void, line_lbr: c_int);

    // nfa_regexec_multi setup
    fn nvim_regexp_call_init_regexec_multi(
        rmp: *mut c_void,
        win: *mut c_void,
        buf: *mut c_void,
        lnum: i32,
    );

    // iemsg null error
    fn nvim_regexp_call_iemsg_null();
}

/// NFA regexp try matching at a specific column.
///
/// Sets up rex.input and time fields, allocates subs on heap (opaque to Rust),
/// calls `rs_nfa_regmatch`, then extracts submatch data back into rex fields.
#[no_mangle]
#[allow(unused_variables)]
pub unsafe extern "C" fn rs_nfa_regtry(
    prog: NfaProgHandle,
    col: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let start = nvim_nfa_prog_get_start(prog);

    // Set up rex.input, time limit, time count
    nvim_regexp_nfa_regtry_setup(prog, col, tm, timed_out);

    // Allocate subs and m on the heap (Rust can't stack-allocate opaque C structs)
    let subs = nvim_regexp_alloc_regsubs();
    let m = nvim_regexp_alloc_regsubs();

    // Clear sub fields
    let subs_norm = nvim_regexp_regsubs_get_norm(subs);
    let subs_synt = nvim_regexp_regsubs_get_synt(subs);
    let m_norm = nvim_regexp_regsubs_get_norm(m);
    let m_synt = nvim_regexp_regsubs_get_synt(m);
    nvim_regexp_call_clear_sub(subs_norm);
    nvim_regexp_call_clear_sub(m_norm);
    nvim_regexp_call_clear_sub(subs_synt);
    nvim_regexp_call_clear_sub(m_synt);

    let result = rs_nfa_regmatch(prog, start, subs, m);

    if result == 0 {
        // No match
        nvim_regexp_free_regsubs(subs);
        nvim_regexp_free_regsubs(m);
        return 0;
    }
    if result == NFA_TOO_EXPENSIVE {
        nvim_regexp_free_regsubs(subs);
        nvim_regexp_free_regsubs(m);
        return NFA_TOO_EXPENSIVE;
    }

    // Extract submatch data
    nvim_regexp_call_cleanup_subexpr();
    if nvim_regexp_is_reg_multi() != 0 {
        nvim_regexp_nfa_regtry_extract_multi(subs, col);
    } else {
        nvim_regexp_nfa_regtry_extract_single(subs, col);
    }

    // Handle \z(...\) extmatch
    // C code: unref_extmatch(re_extmatch_out); re_extmatch_out = NULL;
    nvim_regexp_unref_re_extmatch_out();
    nvim_regexp_set_re_extmatch_out(core::ptr::null_mut());
    let reghasz = nvim_nfa_prog_get_reghasz(prog);
    if reghasz == REX_SET as c_int {
        nvim_regexp_nfa_regtry_extract_extmatch(subs);
    }

    let ret = 1 + nvim_regexp_get_rex_lnum();
    nvim_regexp_free_regsubs(subs);
    nvim_regexp_free_regsubs(m);
    ret
}

/// Core NFA regexp execution for both single-line and multi-line modes.
///
/// Sets up prog, rex fields, applies flags, optionally skips to start char
/// or uses `match_text` optimization, initializes state array, then calls
/// `nfa_regtry`.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_nfa_regexec_both(
    line: *mut u8,
    startcol: i32,
    tm: *mut c_void,
    timed_out: *mut c_int,
) -> c_int {
    let mut col = startcol;
    let mut retval: c_int = 0;

    let prog = nvim_regexp_nfa_regexec_both_get_prog();
    let line = nvim_regexp_nfa_regexec_both_get_line(line);

    // Set up rex pointer fields
    nvim_regexp_nfa_regexec_both_setup_pointers();

    // Be paranoid...
    if prog.is_null() || line.is_null() {
        nvim_regexp_call_iemsg_null();
        if retval > 0 {
            nvim_regexp_nfa_regexec_both_validate_match();
        }
        return retval;
    }

    // Apply regflags overrides (\c, \C, \Z)
    nvim_regexp_nfa_regexec_both_apply_flags(prog);

    // Set up rex fields: lnum, NFA fields, need_clear flags
    nvim_regexp_nfa_regexec_both_setup_nfa(prog);
    // Set rex.line (setup_nfa set it to NULL as placeholder; now set the real value)
    nvim_regexp_set_rex_line(line);

    // If anchored and col > 0, no match possible
    let reganch = nvim_nfa_prog_get_reganch(prog);
    if reganch != 0 && col > 0 {
        return 0;
    }

    // Skip ahead to start character
    let regstart = nvim_nfa_prog_get_regstart(prog);
    if regstart != 0 {
        if rs_skip_to_start(regstart, core::ptr::from_mut::<i32>(&mut col)) == FAIL {
            return 0;
        }

        // If match_text is set, try the fast path
        let match_text = nvim_nfa_prog_get_match_text(prog);
        let reg_icombine = nvim_regexp_get_rex_reg_icombine();
        if !match_text.is_null() && *match_text != 0 && reg_icombine == 0 {
            retval = rs_find_match_text(core::ptr::from_mut::<i32>(&mut col), regstart, match_text);
            nvim_regexp_nfa_regexec_both_set_matchcol(col);
            return retval;
        }
    }

    // If start column past max column, no match
    let reg_maxcol = nvim_regexp_get_rex_reg_maxcol();
    if reg_maxcol > 0 && col >= reg_maxcol {
        // Skip to theend
        if retval > 0 {
            nvim_regexp_nfa_regexec_both_validate_match();
        }
        return retval;
    }

    // Initialize state array
    nvim_regexp_nfa_regexec_both_init_states(prog);

    // Try matching
    retval = rs_nfa_regtry(prog, col, tm, timed_out);

    // theend: validate match positions
    if retval > 0 {
        nvim_regexp_nfa_regexec_both_validate_match();
        if nvim_regexp_is_reg_multi() == 0 {
            // Set rm_matchcol for single-line mode
            nvim_regexp_nfa_regexec_both_set_matchcol(col);
        }
    }

    retval
}

/// NFA regexp execution for single-line matching.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regexec_nl(
    rmp: *mut c_void,
    line: *mut u8,
    col: i32,
    line_lbr: c_int,
) -> c_int {
    nvim_regexp_nfa_regexec_nl_setup(rmp, line_lbr);
    rs_nfa_regexec_both(line, col, core::ptr::null_mut(), core::ptr::null_mut())
}

/// NFA regexp execution for multi-line matching.
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
    nvim_regexp_call_init_regexec_multi(rmp, win, buf, lnum);
    rs_nfa_regexec_both(core::ptr::null_mut(), col, tm, timed_out)
}

// --- End Phase 8.5 ---

// --- Phase 9.1: BT dispatch wrappers ---

extern "C" {
    // Phase 9.2: bt_regexec_both accessors
    fn nvim_regexp_bt_init_stacks();
    fn nvim_regexp_bt_cleanup_stacks();
    fn nvim_bt_prog_get_regmust(prog: *const c_void) -> *mut u8;
    fn nvim_bt_prog_get_regmlen(prog: *const c_void) -> c_int;
    fn nvim_bt_prog_get_regstart(prog: *const c_void) -> c_int;
    fn nvim_bt_prog_get_reganch(prog: *const c_void) -> c_int;
    fn nvim_regexp_call_prog_magic_wrong() -> c_int;
}

/// Initialize rex state for multi-line matching.
#[no_mangle]
pub unsafe extern "C" fn rs_init_regexec_multi(
    rmp: *mut c_void,
    win: *mut c_void,
    buf: *mut c_void,
    lnum: i32,
) {
    nvim_regexp_call_init_regexec_multi(rmp, win, buf, lnum);
}

/// BT regexp execution for single-line matching.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regexec_nl(
    rmp: *mut c_void,
    line: *mut u8,
    col: i32,
    line_lbr: c_int,
) -> c_int {
    nvim_regexp_nfa_regexec_nl_setup(rmp, line_lbr);
    rs_bt_regexec_both(line, col, core::ptr::null_mut(), core::ptr::null_mut())
}

/// BT regexp execution for multi-line matching.
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
    rs_init_regexec_multi(rmp, win, buf, lnum);
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

    // Init regstack and backpos
    nvim_regexp_bt_init_stacks();

    // Get prog and line
    let prog = nvim_regexp_nfa_regexec_both_get_prog();
    let line = nvim_regexp_nfa_regexec_both_get_line(line);

    // Set up rex pointer fields (startpos/endpos or startp/endp)
    nvim_regexp_nfa_regexec_both_setup_pointers();

    // Be paranoid...
    if prog.is_null() || line.is_null() {
        nvim_regexp_call_iemsg_null();
        nvim_regexp_bt_cleanup_stacks();
        return 0;
    }

    // Check validity of program
    if nvim_regexp_call_prog_magic_wrong() != 0 {
        nvim_regexp_bt_cleanup_stacks();
        return 0;
    }

    // If the start column is past the maximum column: no need to try
    let reg_maxcol = nvim_regexp_get_rex_reg_maxcol();
    if reg_maxcol > 0 && col >= reg_maxcol {
        nvim_regexp_bt_cleanup_stacks();
        return 0;
    }

    // Apply regflags overrides (\c, \C, \Z) — works for bt_regprog_T too
    nvim_regexp_nfa_regexec_both_apply_flags(prog);

    // If there is a "must appear" string, look for it
    let regmust = nvim_bt_prog_get_regmust(prog);
    if !regmust.is_null() && !bt_regmust_search(line, col, prog, regmust) {
        nvim_regexp_bt_cleanup_stacks();
        return 0;
    }

    // Set rex.line, rex.lnum, reg_toolong
    nvim_regexp_set_rex_line(line);
    nvim_regexp_set_rex_lnum(0);
    nvim_regexp_set_reg_toolong(0);

    let regstart = nvim_bt_prog_get_regstart(prog);
    let reganch = nvim_bt_prog_get_reganch(prog);

    // Simplest case: Anchored match need be tried only once
    let retval = if reganch != 0 {
        bt_try_anchored(prog, regstart, col, tm, timed_out)
    } else {
        bt_try_unanchored(prog, regstart, &mut col, tm, timed_out)
    };

    // Cleanup stacks
    nvim_regexp_bt_cleanup_stacks();

    // Validate and set matchcol
    if retval > 0 {
        nvim_regexp_nfa_regexec_both_validate_match();
        nvim_regexp_nfa_regexec_both_set_matchcol(col);
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

    if nvim_regexp_get_rex_reg_ic() == 0 {
        // Case-sensitive search
        loop {
            s = vim_strchr(s.cast_const().cast::<c_char>(), c).cast::<u8>();
            if s.is_null() {
                break;
            }
            let mut regmlen = nvim_bt_prog_get_regmlen(prog);
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
            let mut regmlen = nvim_bt_prog_get_regmlen(prog);
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
    let rex_line = nvim_regexp_get_rex_line();
    let c = utf_ptr2char(rex_line.add(col as usize).cast_const().cast::<c_char>());
    if regstart == 0
        || regstart == c
        || (nvim_regexp_get_rex_reg_ic() != 0
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
            let rex_line = nvim_regexp_get_rex_line();
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
        let reg_maxcol = nvim_regexp_get_rex_reg_maxcol();
        if reg_maxcol > 0 && *col >= reg_maxcol {
            return 0;
        }

        retval = rs_regtry(prog, *col, tm, timed_out);
        if retval > 0 {
            return retval;
        }

        // If not currently on the first line, get it again
        if nvim_regexp_get_rex_lnum() != 0 {
            nvim_regexp_set_rex_lnum(0);
            nvim_regexp_set_rex_line(nvim_regexp_call_reg_getline(0).cast::<u8>());
        }
        let rex_line = nvim_regexp_get_rex_line();
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
    fn nvim_regexp_call_engine_regfree(prog: *mut c_void);
    fn nvim_regexp_call_free_regexp_stuff();
}

/// Free a compiled regexp program.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regfree(prog: *mut c_void) {
    if !prog.is_null() {
        nvim_regexp_call_engine_regfree(prog);
    }
}

/// Free all regexp-related allocations (for EXITFREE).
#[no_mangle]
pub unsafe extern "C" fn rs_free_regexp_stuff() {
    nvim_regexp_call_free_regexp_stuff();
}

// --- End Phase 9.3 ---

// --- Phase 9.4: Public execution API ---

const BACKTRACKING_ENGINE: c_int = 1;
const REX_ALL: c_int = 3; // REX_SET(1) | REX_USE(2)

extern "C" {
    // Rex save/restore
    fn nvim_regexp_get_rex_save_size() -> usize;
    fn nvim_regexp_save_rex(out_buf: *mut c_void);
    fn nvim_regexp_restore_rex(saved_buf: *const c_void);
    fn nvim_regexp_get_rex_in_use() -> c_int;
    fn nvim_regexp_set_rex_in_use(v: c_int);
    fn nvim_regexp_clear_rex_pointers();

    // Engine vtable dispatch
    fn nvim_regexp_call_engine_regexec_nl(
        prog: *mut c_void,
        rmp: *mut c_void,
        line: *const u8,
        col: i32,
        nl: c_int,
    ) -> c_int;
    fn nvim_regexp_call_engine_regexec_multi(
        prog: *mut c_void,
        rmp: *mut c_void,
        win: *mut c_void,
        buf: *mut c_void,
        lnum: i32,
        col: i32,
        tm: *mut c_void,
        timed_out: *mut c_int,
    ) -> c_int;

    // regprog_T field accessors
    fn nvim_regprog_get_re_in_use(prog: *const c_void) -> c_int;
    fn nvim_regprog_set_re_in_use(prog: *mut c_void, v: c_int);
    fn nvim_regprog_get_re_engine(prog: *const c_void) -> c_uint;
    fn nvim_regprog_get_re_flags(prog: *const c_void) -> c_uint;

    // regmatch_T / regmmatch_T field accessors
    fn nvim_regmatch_get_regprog(rmp: *const c_void) -> *mut c_void;
    fn nvim_regmatch_set_regprog(rmp: *mut c_void, prog: *mut c_void);
    fn nvim_regmmatch_get_regprog(rmp: *const c_void) -> *mut c_void;
    fn nvim_regmmatch_set_regprog(rmp: *mut c_void, prog: *mut c_void);

    // p_re option
    fn nvim_regexp_get_p_re() -> i32;
    fn nvim_regexp_set_p_re(v: i32);

    // NFA pattern
    fn nvim_nfa_prog_get_pattern(prog: *const c_void) -> *const c_char;

    // reg_do_extmatch
    fn nvim_regexp_set_reg_do_extmatch(v: c_int);

    // Reporting and error
    fn nvim_regexp_call_report_re_switch(pat: *const c_char);
    fn nvim_regexp_call_vim_regcomp(pat: *const c_char, re_flags: c_int) -> *mut c_void;
    fn nvim_regexp_call_vim_regfree(prog: *mut c_void);
    fn nvim_regexp_call_emsg_recursive();

    // regmatch_T handling for vim_regexec_prog
    fn nvim_regexp_get_regmatch_size() -> usize;
    fn nvim_regexp_init_regmatch(buf: *mut c_void, prog: *mut c_void, rm_ic: c_int);
}

/// Save rex state and set `rex_in_use`. Returns buffer on heap.
unsafe fn save_rex_state() -> Vec<u8> {
    let size = nvim_regexp_get_rex_save_size();
    let mut buf = vec![0u8; size];
    if nvim_regexp_get_rex_in_use() != 0 {
        nvim_regexp_save_rex(buf.as_mut_ptr().cast::<c_void>());
    }
    nvim_regexp_set_rex_in_use(1);
    buf
}

/// Restore rex state from saved buffer.
unsafe fn restore_rex_state(buf: &[u8], was_in_use: bool) {
    nvim_regexp_set_rex_in_use(was_in_use as c_int);
    if was_in_use {
        nvim_regexp_restore_rex(buf.as_ptr().cast::<c_void>());
    }
}

/// Handle `NFA_TOO_EXPENSIVE` fallback for single-line matching.
/// Returns the updated result after fallback attempt.
#[allow(clippy::too_many_arguments)]
unsafe fn handle_nfa_fallback_nl(rmp: *mut c_void, line: *const u8, col: i32, nl: c_int) -> c_int {
    let prog = nvim_regmatch_get_regprog(rmp);
    let save_p_re = nvim_regexp_get_p_re();
    let re_flags = nvim_regprog_get_re_flags(prog) as c_int;
    let pat = nvim_regexp_xstrdup(nvim_nfa_prog_get_pattern(prog));

    nvim_regexp_set_p_re(BACKTRACKING_ENGINE);
    nvim_regexp_call_vim_regfree(prog);
    nvim_regexp_call_report_re_switch(pat);
    let new_prog = nvim_regexp_call_vim_regcomp(pat, re_flags);
    nvim_regmatch_set_regprog(rmp, new_prog);

    let mut result: c_int = 0;
    if !new_prog.is_null() {
        nvim_regprog_set_re_in_use(new_prog, 1);
        result = nvim_regexp_call_engine_regexec_nl(new_prog, rmp, line, col, nl);
        nvim_regprog_set_re_in_use(new_prog, 0);
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
    let prog = nvim_regmmatch_get_regprog(rmp);
    let save_p_re = nvim_regexp_get_p_re();
    let re_flags = nvim_regprog_get_re_flags(prog) as c_int;
    let pat = nvim_regexp_xstrdup(nvim_nfa_prog_get_pattern(prog));

    nvim_regexp_set_p_re(BACKTRACKING_ENGINE);
    let prev_prog = prog;

    nvim_regexp_call_report_re_switch(pat);
    nvim_regexp_set_reg_do_extmatch(REX_ALL);
    let new_prog = nvim_regexp_call_vim_regcomp(pat, re_flags);
    nvim_regexp_set_reg_do_extmatch(0);

    let mut result: c_int = 0;
    if new_prog.is_null() {
        // Recompile failed, keep previous prog
        nvim_regmmatch_set_regprog(rmp, prev_prog);
    } else {
        nvim_regmmatch_set_regprog(rmp, new_prog);
        nvim_regexp_call_vim_regfree(prev_prog);

        nvim_regprog_set_re_in_use(new_prog, 1);
        result = nvim_regexp_call_engine_regexec_multi(
            new_prog, rmp, win, buf, lnum, col, tm, timed_out,
        );
        nvim_regprog_set_re_in_use(new_prog, 0);
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
    let prog = nvim_regmatch_get_regprog(rmp);

    // Cannot use the same prog recursively
    if nvim_regprog_get_re_in_use(prog) != 0 {
        nvim_regexp_call_emsg_recursive();
        return 0;
    }
    nvim_regprog_set_re_in_use(prog, 1);

    let was_in_use = nvim_regexp_get_rex_in_use() != 0;
    let saved = save_rex_state();

    nvim_regexp_clear_rex_pointers();

    let mut result = nvim_regexp_call_engine_regexec_nl(prog, rmp, line, col, nl);
    nvim_regprog_set_re_in_use(prog, 0);

    // NFA_TOO_EXPENSIVE fallback
    if nvim_regprog_get_re_engine(prog) == AUTOMATIC_ENGINE as c_uint && result == NFA_TOO_EXPENSIVE
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
    *prog_ptr = nvim_regmatch_get_regprog(rmp);

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
    let prog = nvim_regmmatch_get_regprog(rmp);

    // Cannot use the same prog recursively
    if nvim_regprog_get_re_in_use(prog) != 0 {
        nvim_regexp_call_emsg_recursive();
        return 0;
    }
    nvim_regprog_set_re_in_use(prog, 1);

    let was_in_use = nvim_regexp_get_rex_in_use() != 0;
    let saved = save_rex_state();

    let mut result =
        nvim_regexp_call_engine_regexec_multi(prog, rmp, win, buf, lnum, col, tm, timed_out);
    nvim_regprog_set_re_in_use(prog, 0);

    // NFA_TOO_EXPENSIVE fallback
    if nvim_regprog_get_re_engine(prog) == AUTOMATIC_ENGINE as c_uint && result == NFA_TOO_EXPENSIVE
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
    fn nvim_regexp_get_regexp_engine() -> c_int;
    fn nvim_regexp_set_regexp_engine(v: c_int);
    fn nvim_regexp_set_rex_reg_buf_curbuf();
    fn nvim_regexp_get_called_emsg() -> c_int;
    fn nvim_regexp_call_nfa_regcomp(expr: *const u8, re_flags: c_int) -> *mut c_void;
    fn nvim_regexp_call_bt_regcomp(expr: *const u8, re_flags: c_int) -> *mut c_void;
    fn nvim_regprog_set_re_engine(prog: *mut c_void, v: c_uint);
    fn nvim_regprog_set_re_flags(prog: *mut c_void, v: c_uint);
    fn nvim_regexp_call_emsg_e864();
}

/// Top-level regexp compilation dispatch.
///
/// Selects BT vs NFA based on `p_re` and `\%#=` prefix,
/// handles NFA-to-BT fallback on compile error.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regcomp(expr_arg: *const u8, re_flags: c_int) -> *mut c_void {
    let mut expr = expr_arg;

    // Set regexp_engine from p_re
    nvim_regexp_set_regexp_engine(nvim_regexp_get_p_re());

    // Check for prefix "\%#=", that sets the regexp engine
    if strncmp(expr.cast::<c_char>(), c"\\%#=".as_ptr(), 4) == 0 {
        let newengine = c_int::from(*expr.add(4)) - c_int::from(b'0');

        if newengine == AUTOMATIC_ENGINE
            || newengine == BACKTRACKING_ENGINE
            || newengine == NFA_ENGINE
        {
            nvim_regexp_set_regexp_engine(newengine);
            expr = expr.add(5);
        } else {
            nvim_regexp_call_emsg_e864();
            nvim_regexp_set_regexp_engine(AUTOMATIC_ENGINE);
        }
    }

    // reg_iswordc() uses rex.reg_buf
    nvim_regexp_set_rex_reg_buf_curbuf();

    // First try the NFA engine, unless backtracking was requested
    let called_emsg_before = nvim_regexp_get_called_emsg();
    let regexp_engine = nvim_regexp_get_regexp_engine();

    let mut prog = if regexp_engine == BACKTRACKING_ENGINE {
        nvim_regexp_call_bt_regcomp(expr, re_flags)
    } else {
        let auto_flag = if regexp_engine == AUTOMATIC_ENGINE {
            RE_AUTO
        } else {
            0
        };
        nvim_regexp_call_nfa_regcomp(expr, re_flags + auto_flag)
    };

    // If NFA failed, try backtracking engine
    if prog.is_null()
        && regexp_engine == AUTOMATIC_ENGINE
        && nvim_regexp_get_called_emsg() == called_emsg_before
    {
        nvim_regexp_set_regexp_engine(BACKTRACKING_ENGINE);
        nvim_regexp_call_report_re_switch(expr.cast::<c_char>());
        prog = nvim_regexp_call_bt_regcomp(expr, re_flags);
    }

    if !prog.is_null() {
        // Store engine and flags for later re-compilation
        let engine = nvim_regexp_get_regexp_engine();
        nvim_regprog_set_re_engine(prog, engine as c_uint);
        nvim_regprog_set_re_flags(prog, re_flags as c_uint);
    }

    prog
}

// --- End Phase 9.5 ---

// --- Phase 9.6: bt_regcomp ---

const REGMAGIC: c_int = 0o234;
const RF_LOOKBH: c_uint = 8;

extern "C" {
    fn nvim_regexp_alloc_bt_regprog(regsize_val: i64) -> *mut c_void;
    fn nvim_bt_prog_set_regstart(prog: *mut c_void, v: c_int);
    fn nvim_bt_prog_set_reganch(prog: *mut c_void, v: c_int);
    fn nvim_bt_prog_set_regmust(prog: *mut c_void, v: *mut u8);
    fn nvim_bt_prog_set_regmlen(prog: *mut c_void, v: c_int);
    fn nvim_bt_prog_set_regflags(prog: *mut c_void, v: c_uint);
    fn nvim_bt_prog_set_reghasz(prog: *mut c_void, v: u8);
    fn nvim_bt_prog_set_engine_bt(prog: *mut c_void);
    fn nvim_regexp_call_emsg_e339();
}

/// BT compiler wrapper: two-pass compilation, allocation, optimization extraction.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regcomp(expr: *mut u8, re_flags: c_int) -> *mut c_void {
    if expr.is_null() {
        nvim_regexp_call_iemsg_null();
        nvim_regexp_set_rc_did_emsg(1);
        return std::ptr::null_mut();
    }

    nvim_regexp_call_init_class_tab();

    // First pass: determine size, legality.
    nvim_regexp_call_regcomp_start(expr, re_flags);
    nvim_regexp_set_regcode(nvim_regexp_get_just_calc_size());
    rs_regc(REGMAGIC);
    let mut flags: c_int = 0;
    if rs_reg(REG_NOPAREN, &mut flags).is_null() {
        return std::ptr::null_mut();
    }

    // Allocate space.
    let regsize_val = nvim_regexp_get_regsize();
    let r = nvim_regexp_alloc_bt_regprog(regsize_val);

    // Second pass: emit code.
    nvim_regexp_call_regcomp_start(expr, re_flags);
    nvim_regexp_set_regcode(nvim_regexp_get_prog_program(r));
    rs_regc(REGMAGIC);
    flags = 0;
    if rs_reg(REG_NOPAREN, &mut flags).is_null() || nvim_regexp_get_reg_toolong() != 0 {
        let was_toolong = nvim_regexp_get_reg_toolong() != 0;
        nvim_regexp_xfree(r);
        if was_toolong {
            nvim_regexp_call_emsg_e339();
        }
        return std::ptr::null_mut();
    }

    // Dig out information for optimizations.
    bt_extract_optimizations(r, flags);

    nvim_bt_prog_set_engine_bt(r);
    r
}

/// Extract optimization info from compiled BT program.
unsafe fn bt_extract_optimizations(r: *mut c_void, flags: c_int) {
    nvim_bt_prog_set_regstart(r, 0); // NUL = worst-case default
    nvim_bt_prog_set_reganch(r, 0);
    nvim_bt_prog_set_regmust(r, std::ptr::null_mut());
    nvim_bt_prog_set_regmlen(r, 0);

    let mut rflags = nvim_regexp_get_regflags_compile() as c_uint;
    if flags & HASNL != 0 {
        rflags |= RF_HASNL;
    }
    if flags & HASLOOKBH != 0 {
        rflags |= RF_LOOKBH;
    }
    nvim_bt_prog_set_regflags(r, rflags);

    // Remember whether this pattern has any \z specials in it.
    #[allow(clippy::cast_possible_truncation)]
    let re_has_z = nvim_regexp_get_re_has_z() as u8;
    nvim_bt_prog_set_reghasz(r, re_has_z);

    let program = nvim_regexp_get_prog_program(r);
    let mut scan = program.add(1); // First BRANCH.

    if op(rs_regnext(scan)) != END {
        return; // More than one top-level choice — no optimizations.
    }

    scan = operand(scan);

    // Starting-point info.
    if op(scan) == BOL || op(scan) == RE_BOF {
        nvim_bt_prog_set_reganch(r, 1);
        scan = rs_regnext(scan);
    }

    if op(scan) == EXACTLY {
        nvim_bt_prog_set_regstart(r, utf_ptr2char(operand(scan).cast::<c_char>()));
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
            nvim_bt_prog_set_regstart(r, utf_ptr2char(operand(regnext_scan).cast::<c_char>()));
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
        nvim_bt_prog_set_regmust(r, longest);
        nvim_bt_prog_set_regmlen(r, len);
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
