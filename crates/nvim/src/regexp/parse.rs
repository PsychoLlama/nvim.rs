//! The pattern cursor both engines parse through.
//!
//! Everything below reads one shared cursor, `regparse`, and the
//! one-character lookbehind/lookahead around it (`prevchr`, `curchr`,
//! `nextchr` and the `at_start` flags). [`peekchr`] is where a pattern
//! byte becomes a token: a metacharacter is returned as its byte minus
//! 256, so callers can tell `*` (a repeat) from `\*` (a literal) by sign,
//! and which characters are metacharacters depends on 'magic' — which is
//! why this has to be one shared, stateful reader rather than a pure
//! function of the byte.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int};

use super::{
    CLASS_NONE, MAGIC_ALL, MAGIC_NONE, MAGIC_OFF, MAGIC_ON, MAX_LIMIT, MULTI_MULT, MULTI_ONE,
    NOT_MULTI, REGEXP_ABBR, REGEXP_INRANGE, at_start, backslash_abbr, curchr, magic_T, nextchr,
    parse_state_T, prev_at_start, prevchr, prevchr_len, prevprevchr, refresh_cpo_flags,
    reg_cpo_lit, reg_magic, regnpar, regparse, take_bracketed, take_char_class, toggle_magic,
    unmagic,
};
use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{getdigits_int, hex2nr};
use crate::global_cell::GlobalCell;
use crate::main::rc_did_emsg;
use crate::mbyte::{utf_ptr2char, utf_ptr2len, utfc_ptr2len};
use crate::os::cshim::memmove;
use crate::strings::xstrnsave;
use crate::types::{FAIL, OK};
use ::libc::strlen;

/// Whether `c` is a repeat operator, and whether it can match more than one
/// of the preceding atom.
pub(crate) fn re_multi_type(c: c_int) -> c_int {
    // Only the magic forms count, so undo the marker rather than calling
    // `unmagic` — that would also accept the literal byte.
    match c + 256 {
        x if x == b'@' as c_int || x == b'=' as c_int || x == b'?' as c_int => MULTI_ONE,
        x if x == b'*' as c_int || x == b'+' as c_int || x == b'{' as c_int => MULTI_MULT,
        _ => NOT_MULTI,
    }
}

/// The `\r`, `\t`, ... abbreviations, which `[]` honours unless 'cpoptions'
/// contains `l`.
fn is_abbr(c: u8) -> bool {
    REGEXP_ABBR.to_bytes().contains(&c)
}

/// The characters a backslash keeps its literal meaning for inside a `[]`
/// collection.
fn is_inrange(c: u8) -> bool {
    REGEXP_INRANGE.to_bytes().contains(&c)
}

/// Skip past a `[]` collection, `p` pointing just after the `[`. Stops at
/// the closing `]` or at the pattern's NUL.
///
/// # Safety
///
/// `p` must point into a NUL-terminated pattern.
pub(crate) unsafe fn skip_anyof(mut p: *mut c_char) -> *mut c_char {
    // A leading `^` negates; a `]` or `-` immediately after that is
    // literal rather than the close or a range.
    if unsafe { *p } as u8 == b'^' {
        p = unsafe { p.add(1) };
    }
    if matches!(unsafe { *p } as u8, b']' | b'-') {
        p = unsafe { p.add(1) };
    }
    while !matches!(unsafe { *p } as u8, 0 | b']') {
        let len = unsafe { utfc_ptr2len(p) };
        if len > 1 {
            p = unsafe { p.add(len as usize) };
        } else if unsafe { *p } as u8 == b'-' {
            p = unsafe { p.add(1) };
            if !matches!(unsafe { *p } as u8, 0 | b']') {
                p = unsafe { p.add(utfc_ptr2len(p) as usize) };
            }
        } else if unsafe { *p } as u8 == b'\\'
            && (is_inrange(unsafe { *p.add(1) } as u8)
                || (reg_cpo_lit.get() == 0 && is_abbr(unsafe { *p.add(1) } as u8)))
        {
            p = unsafe { p.add(2) };
        } else if unsafe { *p } as u8 == b'[' {
            // A `[:class:]`, `[=equi=]` or `[.coll.]` advances `p`
            // itself; a bare `[` is literal.
            if unsafe { take_char_class(&mut p) } == CLASS_NONE as c_int
                && unsafe { take_bracketed(&mut p, b'=') } == 0
                && unsafe { take_bracketed(&mut p, b'.') } == 0
                && unsafe { *p } as u8 != 0
            {
                p = unsafe { p.add(1) };
            }
        } else {
            p = unsafe { p.add(1) };
        }
    }
    p
}

/// Skip past the pattern starting at `startp`, stopping at `delim` or the
/// NUL. `magic` is the initial 'magic' setting; `\v`/`\V` inside the
/// pattern change it as the scan proceeds.
///
/// # Safety
///
/// `startp` must point to a NUL-terminated pattern.
pub unsafe fn skip_regexp(startp: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char {
    unsafe {
        skip_regexp_ex(
            startp,
            delim,
            magic,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
    }
}

/// [`skip_regexp`], but complain and return NULL when the delimiter is
/// missing rather than returning the pattern's end.
///
/// # Safety
///
/// `startp` must point to a NUL-terminated pattern.
pub unsafe fn skip_regexp_err(startp: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char {
    let p = unsafe { skip_regexp(startp, delim, magic) };
    if unsafe { *p } as c_int != delim {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let startp = unsafe { c_str(startp) };
        semsg!("E654: Missing delimiter after search pattern: {startp}");
        return core::ptr::null_mut();
    }
    p
}

/// The full skip. Beyond [`skip_regexp`]'s job it can rewrite the pattern:
/// when `dirc` is `?` and `newp` is given, an escaped `\?` (which a `?`
/// search cannot contain) is unescaped into a copy, `*newp` taking
/// ownership and `*dropped` counting the backslashes removed. `*magic_val`
/// receives the 'magic' setting in force at the end.
///
/// # Safety
///
/// `startp` must point to a NUL-terminated pattern; the out-parameters
/// must be null or writable.
pub unsafe fn skip_regexp_ex(
    mut startp: *mut c_char,
    dirc: c_int,
    magic: c_int,
    newp: *mut *mut c_char,
    dropped: *mut c_int,
    magic_val: *mut magic_T,
) -> *mut c_char {
    let mut mymagic = if magic != 0 { MAGIC_ON } else { MAGIC_OFF };
    let mut p = startp;
    let mut startplen: usize = 0;
    refresh_cpo_flags();
    while unsafe { *p } as u8 != 0 {
        if unsafe { *p } as c_int == dirc {
            break;
        }
        if (unsafe { *p } as u8 == b'[' && mymagic >= MAGIC_ON)
            || (unsafe { *p } as u8 == b'\\'
                && unsafe { *p.add(1) } as u8 == b'['
                && mymagic <= MAGIC_OFF)
        {
            p = unsafe { skip_anyof(p.add(1)) };
            if unsafe { *p } as u8 == 0 {
                break;
            }
        } else if unsafe { *p } as u8 == b'\\' && unsafe { *p.add(1) } as u8 != 0 {
            if dirc == b'?' as c_int && !newp.is_null() && unsafe { *p.add(1) } as u8 == b'?' {
                if startplen == 0 {
                    startplen = unsafe { strlen(startp) };
                }
                if unsafe { (*newp).is_null() } {
                    unsafe { *newp = xstrnsave(startp, startplen) };
                    p = unsafe { (*newp).offset(p.offset_from(startp)) };
                    startp = unsafe { *newp };
                }
                if !dropped.is_null() {
                    unsafe { *dropped += 1 };
                }
                unsafe {
                    memmove(
                        p.cast(),
                        p.add(1).cast(),
                        startplen - p.add(1).offset_from(startp) as usize + 1,
                    )
                };
            } else {
                p = unsafe { p.add(1) };
            }
            if unsafe { *p } as u8 == b'v' {
                mymagic = MAGIC_ALL;
            } else if unsafe { *p } as u8 == b'V' {
                mymagic = MAGIC_NONE;
            }
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    if !magic_val.is_null() {
        unsafe { *magic_val = mymagic };
    }
    p
}

/// The byte `off` bytes past the cursor.
pub(crate) fn pat_byte(off: usize) -> u8 {
    // SAFETY: the cursor points into the pattern `initchr` was given, and
    // every caller here has already established that `off` is at or before
    // its NUL.
    unsafe { *regparse.get().add(off) as u8 }
}

/// The character `off` bytes past the cursor.
pub(crate) fn pat_char(off: usize) -> c_int {
    // SAFETY: as `pat_byte`.
    unsafe { utf_ptr2char(regparse.get().add(off)) }
}

/// The encoded length of the character `off` bytes past the cursor.
pub(crate) fn pat_charlen(off: usize) -> c_int {
    // SAFETY: as `pat_byte`.
    unsafe { utf_ptr2len(regparse.get().add(off)) }
}

/// Move the cursor. Wrapping arithmetic because [`peekchr`] and
/// [`ungetchr`] step it back over a character they have already read,
/// which the compiler cannot see is in bounds.
pub(crate) fn pat_seek(delta: isize) {
    regparse.set(regparse.get().wrapping_offset(delta));
}

/// Point the parse cursor at `pattern` and clear the lookahead.
///
/// # Safety
///
/// `pattern` must be NUL-terminated and outlive the parse.
pub(crate) unsafe fn initchr(pattern: *mut c_char) {
    regparse.set(pattern);
    prevchr_len.set(0);
    nextchr.set(-1);
    prevchr.set(-1);
    prevprevchr.set(-1);
    curchr.set(-1);
    at_start.set(1);
    prev_at_start.set(0);
}

/// Snapshot the cursor so a speculative parse can be rewound. The NFA
/// compiler parses parts of a pattern twice.
pub(crate) fn save_parse_state(ps: &mut parse_state_T) {
    ps.regparse = regparse.get();
    ps.prevchr_len = prevchr_len.get();
    ps.curchr = curchr.get();
    ps.prevchr = prevchr.get();
    ps.prevprevchr = prevprevchr.get();
    ps.nextchr = nextchr.get();
    ps.at_start = at_start.get();
    ps.prev_at_start = prev_at_start.get();
    ps.regnpar = regnpar.get();
}

/// Rewind to a [`save_parse_state`] snapshot.
pub(crate) fn restore_parse_state(ps: &parse_state_T) {
    regparse.set(ps.regparse);
    prevchr_len.set(ps.prevchr_len);
    curchr.set(ps.curchr);
    prevchr.set(ps.prevchr);
    prevprevchr.set(ps.prevprevchr);
    nextchr.set(ps.nextchr);
    at_start.set(ps.at_start);
    prev_at_start.set(ps.prev_at_start);
    regnpar.set(ps.regnpar);
}

/// The characters `\` gives a special meaning to. [`peekchr`] reparses the
/// escaped byte with its magic marker flipped when it finds one here.
const META_CHARS: &[u8] = b"%&()*+.123456789<=>?@ACDFHIKLMOPSUVWXZ[_acdfhiklmnopsuvwxz{|~";

static IS_META: [bool; 127] = build_is_meta();

const fn build_is_meta() -> [bool; 127] {
    let mut tab = [false; 127];
    let mut i = 0;
    while i < META_CHARS.len() {
        tab[META_CHARS[i] as usize] = true;
        i += 1;
    }
    tab
}

/// The token at the cursor, without consuming it. A metacharacter comes
/// back as its byte minus 256; anything else as itself.
pub(crate) fn peekchr() -> c_int {
    // Depth of the `\`-escape reparse below, so that `\*` right after
    // `\(` still counts as a repeat.
    static AFTER_SLASH: GlobalCell<c_int> = GlobalCell::new(0);

    if curchr.get() != -1 {
        return curchr.get();
    }
    curchr.set(pat_byte(0) as c_int);
    match curchr.get() as u8 {
        b'.' | b'[' | b'~' => {
            // Magic as soon as 'magic' is on.
            if reg_magic.get() >= MAGIC_ON {
                curchr.set(curchr.get() - 256);
            }
        }
        b'(' | b')' | b'{' | b'%' | b'+' | b'=' | b'?' | b'@' | b'!' | b'&' | b'|' | b'<'
        | b'>' | b'#' | b'"' | b'\'' | b',' | b'-' | b':' | b';' | b'`' | b'/' => {
            // Magic only under `\v`.
            if reg_magic.get() == MAGIC_ALL {
                curchr.set(curchr.get() - 256);
            }
        }
        b'*' => {
            // A `*` with nothing to repeat is literal: at the start of the
            // pattern, right after a `^` that was itself at the start, or
            // right after `\(`, `\&` or `\|` — unless we are inside the
            // escape reparse, where the preceding token was consumed.
            if reg_magic.get() >= MAGIC_ON
                && at_start.get() == 0
                && !(prev_at_start.get() != 0 && prevchr.get() == b'^' as c_int - 256)
                && (AFTER_SLASH.get() != 0
                    || (prevchr.get() != b'(' as c_int - 256
                        && prevchr.get() != b'&' as c_int - 256
                        && prevchr.get() != b'|' as c_int - 256))
            {
                curchr.set(b'*' as c_int - 256);
            }
        }
        b'^' => {
            // Only anchoring where a branch can start.
            if reg_magic.get() >= MAGIC_OFF
                && (at_start.get() != 0
                    || reg_magic.get() == MAGIC_ALL
                    || prevchr.get() == b'(' as c_int - 256
                    || prevchr.get() == b'|' as c_int - 256
                    || prevchr.get() == b'&' as c_int - 256
                    || prevchr.get() == b'n' as c_int - 256
                    || (unmagic(prevchr.get()) == b'(' as c_int
                        && prevprevchr.get() == b'%' as c_int - 256))
            {
                curchr.set(b'^' as c_int - 256);
                at_start.set(1);
                prev_at_start.set(0);
            }
        }
        b'$' => {
            // Only anchoring where a branch can end. Look past any
            // `\c`-style flags, which don't consume input, tracking the
            // `\v`/`\V` among them because they change what follows.
            if reg_magic.get() >= MAGIC_OFF {
                let mut i = 1;
                let mut is_magic_all = reg_magic.get() == MAGIC_ALL;
                while pat_byte(i) == b'\\'
                    && matches!(
                        pat_byte(i + 1),
                        b'c' | b'C' | b'm' | b'M' | b'v' | b'V' | b'Z'
                    )
                {
                    match pat_byte(i + 1) {
                        b'v' => is_magic_all = true,
                        b'm' | b'M' | b'V' => is_magic_all = false,
                        _ => {}
                    }
                    i += 2;
                }
                if pat_byte(i) == 0
                    || (pat_byte(i) == b'\\'
                        && matches!(pat_byte(i + 1), b'|' | b'&' | b')' | b'n'))
                    || (is_magic_all && matches!(pat_byte(i), b'|' | b'&' | b')'))
                    || reg_magic.get() == MAGIC_ALL
                {
                    curchr.set(b'$' as c_int - 256);
                }
            }
        }
        b'\\' => {
            let c = pat_byte(1);
            if c == 0 {
                // A trailing backslash is a literal backslash.
                curchr.set(b'\\' as c_int);
            } else if c <= b'~' && IS_META[c as usize] {
                // `\x` means whatever a bare `x` would not: reparse the
                // escaped byte and flip its magic marker.
                curchr.set(-1);
                prev_at_start.set(at_start.get());
                at_start.set(0);
                pat_seek(1);
                AFTER_SLASH.set(AFTER_SLASH.get() + 1);
                peekchr();
                pat_seek(-1);
                AFTER_SLASH.set(AFTER_SLASH.get() - 1);
                curchr.set(toggle_magic(curchr.get()));
            } else if is_abbr(c) {
                curchr.set(backslash_abbr(c as c_int));
            } else if reg_magic.get() == MAGIC_NONE && matches!(c, b'$' | b'^') {
                curchr.set(toggle_magic(c as c_int));
            } else {
                curchr.set(pat_char(1));
            }
        }
        _ => {
            curchr.set(pat_char(0));
        }
    }
    curchr.get()
}

/// Consume the token [`peekchr`] returned, sliding the lookbehind along.
pub(crate) fn skipchr() {
    // A `\` and the byte after it are one token, so skip both.
    prevchr_len.set(if pat_byte(0) == b'\\' { 1 } else { 0 });
    if pat_byte(prevchr_len.get() as usize) != 0 {
        prevchr_len.set(prevchr_len.get() + pat_charlen(prevchr_len.get() as usize));
    }
    pat_seek(prevchr_len.get() as isize);
    prev_at_start.set(at_start.get());
    at_start.set(0);
    prevprevchr.set(prevchr.get());
    prevchr.set(curchr.get());
    curchr.set(nextchr.get());
    nextchr.set(-1);
}

/// [`skipchr`] without disturbing `at_start` and the lookbehind — for
/// tokens that are not really part of the pattern, like a `\c` flag.
pub(crate) fn skipchr_keepstart() {
    let start = prev_at_start.get();
    let prev = prevchr.get();
    let prevprev = prevprevchr.get();
    skipchr();
    at_start.set(start);
    prevchr.set(prev);
    prevprevchr.set(prevprev);
}

/// Take the next token.
pub(crate) fn getchr() -> c_int {
    let chr = peekchr();
    skipchr();
    chr
}

/// Put the last token back. Only one step of pushback is available.
pub(crate) fn ungetchr() {
    nextchr.set(curchr.get());
    curchr.set(prevchr.get());
    prevchr.set(prevprevchr.get());
    at_start.set(prev_at_start.get());
    prev_at_start.set(0);
    pat_seek(-(prevchr_len.get() as isize));
}

/// Read up to `maxinputlen` hex digits at the cursor, or -1 if there are
/// none. Backs `\%xff` and friends.
pub(crate) fn gethexchrs(maxinputlen: c_int) -> i64 {
    let mut nr: i64 = 0;
    let mut i = 0;
    while i < maxinputlen {
        let c = pat_byte(0) as c_int;
        if !ascii_isxdigit(c) {
            break;
        }
        nr = (nr << 4) | hex2nr(c) as i64;
        pat_seek(1);
        i += 1;
    }
    if i == 0 { -1 } else { nr }
}

/// Read decimal digits at the cursor, or -1 if there are none.
pub(crate) fn getdecchrs() -> i64 {
    let mut nr: i64 = 0;
    let mut i = 0;
    loop {
        let c = pat_byte(0);
        if !c.is_ascii_digit() {
            break;
        }
        nr = nr * 10 + (c - b'0') as i64;
        pat_seek(1);
        // Unlike the hex and octal readers this drops the lookahead, so
        // that what follows `\%d123` is peeked afresh.
        curchr.set(-1);
        i += 1;
    }
    if i == 0 { -1 } else { nr }
}

/// Read up to three octal digits at the cursor, or -1 if there are none.
/// Stops early once the value can no longer fit in a byte.
pub(crate) fn getoctchrs() -> i64 {
    let mut nr: i64 = 0;
    let mut i = 0;
    while i < 3 && nr < 0o40 {
        let c = pat_byte(0);
        if !(b'0'..=b'7').contains(&c) {
            break;
        }
        nr = (nr << 3) | hex2nr(c as c_int) as i64;
        pat_seek(1);
        i += 1;
    }
    if i == 0 { -1 } else { nr }
}

/// Read a number at the cursor, advancing it past the digits.
fn take_digits(default: c_int) -> c_int {
    // SAFETY: `regparse` points into the NUL-terminated pattern, and
    // `getdigits_int` advances it no further than the terminator -- it reads
    // digits and calls nothing, so it cannot re-enter the cell.
    regparse.with_mut(|pp| unsafe { getdigits_int(pp, false, default) })
}

/// Parse the `{n,m}` bound at the cursor into `minval`/`maxval`, leaving
/// the cursor past the closing brace. Returns `FAIL` after reporting a
/// syntax error.
pub(crate) fn read_limits(minval: &mut c_int, maxval: &mut c_int) -> c_int {
    // `{-n,m}` asks for the shortest match, which the caller reads back
    // out of the min/max order rather than from a flag.
    let mut reverse = false;
    if pat_byte(0) == b'-' {
        pat_seek(1);
        reverse = true;
    }
    let first_byte = pat_byte(0);
    *minval = take_digits(0);
    if pat_byte(0) == b',' {
        pat_seek(1);
        *maxval = if ascii_isdigit(pat_byte(0) as c_int) {
            take_digits(MAX_LIMIT)
        } else {
            MAX_LIMIT
        };
    } else if ascii_isdigit(first_byte as c_int) {
        // `{n}` is exactly n.
        *maxval = *minval;
    } else {
        *maxval = MAX_LIMIT;
    }
    if pat_byte(0) == b'\\' {
        pat_seek(1);
    }
    if pat_byte(0) != b'}' {
        let prefix = if reg_magic.get() == MAGIC_ALL {
            ""
        } else {
            "\\"
        };
        semsg!("E554: Syntax error in {prefix}{{...}}");
        rc_did_emsg.set(true);
        return FAIL;
    }
    if (!reverse && *minval > *maxval) || (reverse && *minval < *maxval) {
        core::mem::swap(minval, maxval);
    }
    skipchr();
    OK
}
