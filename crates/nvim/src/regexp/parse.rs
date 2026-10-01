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
#![allow(unsafe_code)]

use crate::cstr;
use crate::message_fmt::{c_str, msg_bytes};
use crate::option::cpo_has;
use crate::regexp::RegCompiler;
use crate::semsg;
use crate::types::CpoFlag;
use core::ffi::{c_char, c_int};

use super::{
    MAGIC_ALL, MAGIC_NONE, MAGIC_OFF, MAGIC_ON, MAX_LIMIT, MULTI_MULT, MULTI_ONE, Magic, NOT_MULTI,
    ParseState, REGEXP_ABBR, REGEXP_INRANGE, backslash_abbr, take_bracketed, take_char_class,
    toggle_magic, unmagic,
};
use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{getdigits_int, hex2nr};
use crate::mbyte::{utf_ptr2char, utf_ptr2len, utfc_ptr2len};
use crate::regexp::state::rc_did_emsg;
use crate::strings::xstrnsave;
use crate::types::Failed;

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
pub(crate) unsafe fn skip_anyof(mut p: *mut c_char, cpo_lit: bool) -> *mut c_char {
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
                || (!cpo_lit && is_abbr(unsafe { *p.add(1) } as u8)))
        {
            p = unsafe { p.add(2) };
        } else if unsafe { *p } as u8 == b'[' {
            // A `[:class:]`, `[=equi=]` or `[.coll.]` advances `p`
            // itself; a bare `[` is literal.
            if unsafe { take_char_class(&mut p) }.is_none()
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

/// [`skip_regexp`] as an offset walk: how many bytes of `text` the pattern
/// takes, stopping at `delim` or at the NUL that ends the string `text`
/// starts with.
///
/// The slice form an address or a command-line parse wants. Nothing is
/// written: [`skip_regexp_ex`] only rewrites the pattern when it is given a
/// `newp` slot, and this passes none.
///
/// # Panics
/// If `text` holds no NUL at all.
pub(crate) fn skip_regexp_at(text: &[u8], delim: c_int, magic: c_int) -> usize {
    assert!(text.contains(&0), "the walk needs a terminator to stop at");
    let start = text.as_ptr().cast::<c_char>().cast_mut();
    // SAFETY: the assert above says `text` is a NUL-terminated string, and
    // with no `newp` slot the walk only reads it -- so the pointer, derived
    // from a shared borrow, is never written through. The end it answers is
    // inside `text`, the start first.
    unsafe { skip_regexp(start, delim, magic).offset_from(start) }.cast_unsigned()
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

/// [`skip_regexp_err`] as an offset walk: how many bytes of `text` the
/// pattern takes, or `None` when the closing delimiter is missing (which
/// reports E654).
///
/// # Panics
/// If `text` holds no NUL at all.
pub(crate) fn skip_regexp_err_at(text: &[u8], delim: c_int, magic: c_int) -> Option<usize> {
    let end = skip_regexp_at(text, delim, magic);
    if c_int::from(cstr::byte_at(text, end)) != delim {
        // The whole string, as the pointer form's `c_str(startp)` showed
        // it -- not just the part the skip consumed.
        let shown = text
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(text.len());
        let startp = msg_bytes(&text[..shown]);
        semsg!("E654: Missing delimiter after search pattern: {startp}");
        return None;
    }
    Some(end)
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
    magic_val: *mut Magic,
) -> *mut c_char {
    let mut mymagic = if magic != 0 { MAGIC_ON } else { MAGIC_OFF };
    let mut p = startp;
    let mut startplen: usize = 0;
    let cpo_lit = cpo_has(CpoFlag::LITERAL);
    while unsafe { *p } as u8 != 0 {
        if unsafe { *p } as c_int == dirc {
            break;
        }
        if (unsafe { *p } as u8 == b'[' && mymagic >= MAGIC_ON)
            || (unsafe { *p } as u8 == b'\\'
                && unsafe { *p.add(1) } as u8 == b'['
                && mymagic <= MAGIC_OFF)
        {
            p = unsafe { skip_anyof(p.add(1), cpo_lit) };
            if unsafe { *p } as u8 == 0 {
                break;
            }
        } else if unsafe { *p } as u8 == b'\\' && unsafe { *p.add(1) } as u8 != 0 {
            if dirc == b'?' as c_int && !newp.is_null() && unsafe { *p.add(1) } as u8 == b'?' {
                if startplen == 0 {
                    startplen = unsafe { cstr::bytes_at(startp) }.len();
                }
                if unsafe { (*newp).is_null() } {
                    unsafe { *newp = xstrnsave(startp, startplen) };
                    p = unsafe { (*newp).offset(p.offset_from(startp)) };
                    startp = unsafe { *newp };
                }
                if !dropped.is_null() {
                    unsafe { *dropped += 1 };
                }
                let into = p.cast::<u8>();
                unsafe {
                    into.copy_from(
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
pub(crate) fn pat_byte(rc: &mut RegCompiler, off: usize) -> u8 {
    // SAFETY: the cursor points into the pattern `initchr` was given, and
    // every caller here has already established that `off` is at or before
    // its NUL.
    unsafe { *rc.cursor.add(off) as u8 }
}

/// The character `off` bytes past the cursor.
pub(crate) fn pat_char(rc: &mut RegCompiler, off: usize) -> c_int {
    // SAFETY: as `pat_byte`.
    unsafe { utf_ptr2char(rc.cursor.add(off)) }
}

/// The encoded length of the character `off` bytes past the cursor.
pub(crate) fn pat_charlen(rc: &mut RegCompiler, off: usize) -> c_int {
    // SAFETY: as `pat_byte`.
    unsafe { utf_ptr2len(rc.cursor.add(off)) }
}

/// Move the cursor. Wrapping arithmetic because [`peekchr`] and
/// [`ungetchr`] step it back over a character they have already read,
/// which the compiler cannot see is in bounds.
pub(crate) fn pat_seek(rc: &mut RegCompiler, delta: isize) {
    rc.cursor = rc.cursor.wrapping_offset(delta);
}

/// Snapshot the cursor so a speculative parse can be rewound. The NFA
/// compiler parses parts of a pattern twice.
pub(crate) fn save_parse_state(rc: &mut RegCompiler, ps: &mut ParseState) {
    ps.regparse = rc.cursor;
    ps.prevchr_len = rc.prev_token_len;
    ps.curchr = rc.token;
    ps.prevchr = rc.prev_token;
    ps.prevprevchr = rc.prev2_token;
    ps.nextchr = rc.next_token;
    ps.at_start = rc.at_start;
    ps.prev_at_start = rc.prev_at_start;
    ps.regnpar = rc.next_group;
}

/// Rewind to a [`save_parse_state`] snapshot.
pub(crate) fn restore_parse_state(rc: &mut RegCompiler, ps: &ParseState) {
    rc.cursor = ps.regparse;
    rc.prev_token_len = ps.prevchr_len;
    rc.token = ps.curchr;
    rc.prev_token = ps.prevchr;
    rc.prev2_token = ps.prevprevchr;
    rc.next_token = ps.nextchr;
    rc.at_start = ps.at_start;
    rc.prev_at_start = ps.prev_at_start;
    rc.next_group = ps.regnpar;
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
pub(crate) fn peekchr(rc: &mut RegCompiler) -> c_int {
    if rc.token != -1 {
        return rc.token;
    }
    rc.token = pat_byte(rc, 0) as c_int;
    match rc.token as u8 {
        b'.' | b'[' | b'~' => {
            // Magic as soon as 'magic' is on.
            if rc.magic >= MAGIC_ON {
                rc.token = rc.token - 256;
            }
        }
        b'(' | b')' | b'{' | b'%' | b'+' | b'=' | b'?' | b'@' | b'!' | b'&' | b'|' | b'<'
        | b'>' | b'#' | b'"' | b'\'' | b',' | b'-' | b':' | b';' | b'`' | b'/' => {
            // Magic only under `\v`.
            if rc.magic == MAGIC_ALL {
                rc.token = rc.token - 256;
            }
        }
        b'*' => {
            // A `*` with nothing to repeat is literal: at the start of the
            // pattern, right after a `^` that was itself at the start, or
            // right after `\(`, `\&` or `\|` — unless we are inside the
            // escape reparse, where the preceding token was consumed.
            if rc.magic >= MAGIC_ON
                && rc.at_start == 0
                && !(rc.prev_at_start != 0 && rc.prev_token == b'^' as c_int - 256)
                && (rc.escape_depth != 0
                    || (rc.prev_token != b'(' as c_int - 256
                        && rc.prev_token != b'&' as c_int - 256
                        && rc.prev_token != b'|' as c_int - 256))
            {
                rc.token = b'*' as c_int - 256;
            }
        }
        b'^' => {
            // Only anchoring where a branch can start.
            if rc.magic >= MAGIC_OFF
                && (rc.at_start != 0
                    || rc.magic == MAGIC_ALL
                    || rc.prev_token == b'(' as c_int - 256
                    || rc.prev_token == b'|' as c_int - 256
                    || rc.prev_token == b'&' as c_int - 256
                    || rc.prev_token == b'n' as c_int - 256
                    || (unmagic(rc.prev_token) == b'(' as c_int
                        && rc.prev2_token == b'%' as c_int - 256))
            {
                rc.token = b'^' as c_int - 256;
                rc.at_start = 1;
                rc.prev_at_start = 0;
            }
        }
        b'$' => {
            // Only anchoring where a branch can end. Look past any
            // `\c`-style flags, which don't consume input, tracking the
            // `\v`/`\V` among them because they change what follows.
            if rc.magic >= MAGIC_OFF {
                let mut i = 1;
                let mut is_magic_all = rc.magic == MAGIC_ALL;
                while pat_byte(rc, i) == b'\\'
                    && matches!(
                        pat_byte(rc, i + 1),
                        b'c' | b'C' | b'm' | b'M' | b'v' | b'V' | b'Z'
                    )
                {
                    match pat_byte(rc, i + 1) {
                        b'v' => is_magic_all = true,
                        b'm' | b'M' | b'V' => is_magic_all = false,
                        _ => {}
                    }
                    i += 2;
                }
                if pat_byte(rc, i) == 0
                    || (pat_byte(rc, i) == b'\\'
                        && matches!(pat_byte(rc, i + 1), b'|' | b'&' | b')' | b'n'))
                    || (is_magic_all && matches!(pat_byte(rc, i), b'|' | b'&' | b')'))
                    || rc.magic == MAGIC_ALL
                {
                    rc.token = b'$' as c_int - 256;
                }
            }
        }
        b'\\' => {
            let c = pat_byte(rc, 1);
            if c == 0 {
                // A trailing backslash is a literal backslash.
                rc.token = b'\\' as c_int;
            } else if c <= b'~' && IS_META[c as usize] {
                // `\x` means whatever a bare `x` would not: reparse the
                // escaped byte and flip its magic marker.
                rc.token = -1;
                rc.prev_at_start = rc.at_start;
                rc.at_start = 0;
                pat_seek(rc, 1);
                // The depth is what lets `\*` right after `\(` still count
                // as a repeat.
                rc.escape_depth += 1;
                peekchr(rc);
                rc.escape_depth -= 1;
                pat_seek(rc, -1);
                rc.token = toggle_magic(rc.token);
            } else if is_abbr(c) {
                rc.token = backslash_abbr(c as c_int);
            } else if rc.magic == MAGIC_NONE && matches!(c, b'$' | b'^') {
                rc.token = toggle_magic(c as c_int);
            } else {
                rc.token = pat_char(rc, 1);
            }
        }
        _ => {
            rc.token = pat_char(rc, 0);
        }
    }
    rc.token
}

/// Consume the token [`peekchr`] returned, sliding the lookbehind along.
pub(crate) fn skipchr(rc: &mut RegCompiler) {
    // A `\` and the byte after it are one token, so skip both.
    rc.prev_token_len = if pat_byte(rc, 0) == b'\\' { 1 } else { 0 };
    if pat_byte(rc, rc.prev_token_len as usize) != 0 {
        rc.prev_token_len = rc.prev_token_len + pat_charlen(rc, rc.prev_token_len as usize);
    }
    pat_seek(rc, rc.prev_token_len as isize);
    rc.prev_at_start = rc.at_start;
    rc.at_start = 0;
    rc.prev2_token = rc.prev_token;
    rc.prev_token = rc.token;
    rc.token = rc.next_token;
    rc.next_token = -1;
}

/// [`skipchr`] without disturbing `at_start` and the lookbehind — for
/// tokens that are not really part of the pattern, like a `\c` flag.
pub(crate) fn skipchr_keepstart(rc: &mut RegCompiler) {
    let start = rc.prev_at_start;
    let prev = rc.prev_token;
    let prevprev = rc.prev2_token;
    skipchr(rc);
    rc.at_start = start;
    rc.prev_token = prev;
    rc.prev2_token = prevprev;
}

/// Switch 'magic' to `level` for the rest of the pattern: a `\v`, `\m`,
/// `\M` or `\V`, which is not a token itself.
pub(crate) fn set_magic(rc: &mut RegCompiler, level: Magic) {
    rc.magic = level;
    skipchr_keepstart(rc);
    // The switch changes what the next byte means, so the lookahead taken
    // before it has to be dropped.
    rc.token = -1;
}

/// Take the next token.
pub(crate) fn getchr(rc: &mut RegCompiler) -> c_int {
    let chr = peekchr(rc);
    skipchr(rc);
    chr
}

/// Put the last token back. Only one step of pushback is available.
pub(crate) fn ungetchr(rc: &mut RegCompiler) {
    rc.next_token = rc.token;
    rc.token = rc.prev_token;
    rc.prev_token = rc.prev2_token;
    rc.at_start = rc.prev_at_start;
    rc.prev_at_start = 0;
    pat_seek(rc, -(rc.prev_token_len as isize));
}

/// Read up to `maxinputlen` hex digits at the cursor, or -1 if there are
/// none. Backs `\%xff` and friends.
pub(crate) fn gethexchrs(rc: &mut RegCompiler, maxinputlen: c_int) -> i64 {
    let mut nr: i64 = 0;
    let mut i = 0;
    while i < maxinputlen {
        let c = pat_byte(rc, 0) as c_int;
        if !ascii_isxdigit(c) {
            break;
        }
        nr = (nr << 4) | hex2nr(c) as i64;
        pat_seek(rc, 1);
        i += 1;
    }
    if i == 0 { -1 } else { nr }
}

/// Read decimal digits at the cursor, or -1 if there are none.
pub(crate) fn getdecchrs(rc: &mut RegCompiler) -> i64 {
    let mut nr: i64 = 0;
    let mut i = 0;
    loop {
        let c = pat_byte(rc, 0);
        if !c.is_ascii_digit() {
            break;
        }
        nr = nr * 10 + (c - b'0') as i64;
        pat_seek(rc, 1);
        // Unlike the hex and octal readers this drops the lookahead, so
        // that what follows `\%d123` is peeked afresh.
        rc.token = -1;
        i += 1;
    }
    if i == 0 { -1 } else { nr }
}

/// Read up to three octal digits at the cursor, or -1 if there are none.
/// Stops early once the value can no longer fit in a byte.
pub(crate) fn getoctchrs(rc: &mut RegCompiler) -> i64 {
    let mut nr: i64 = 0;
    let mut i = 0;
    while i < 3 && nr < 0o40 {
        let c = pat_byte(rc, 0);
        if !(b'0'..=b'7').contains(&c) {
            break;
        }
        nr = (nr << 3) | hex2nr(c as c_int) as i64;
        pat_seek(rc, 1);
        i += 1;
    }
    if i == 0 { -1 } else { nr }
}

/// Read a number at the cursor, advancing it past the digits.
fn take_digits(rc: &mut RegCompiler, default: c_int) -> c_int {
    // SAFETY: the cursor points into the NUL-terminated pattern, and
    // `getdigits_int` advances it no further than the terminator.
    unsafe { getdigits_int(&mut rc.cursor, false, default) }
}

/// Parse the `{n,m}` bound at the cursor into `minval`/`maxval`, leaving
/// the cursor past the closing brace. Answers `Err` after reporting a
/// syntax error.
pub(crate) fn read_limits(
    rc: &mut RegCompiler,
    minval: &mut c_int,
    maxval: &mut c_int,
) -> Result<(), Failed> {
    // `{-n,m}` asks for the shortest match, which the caller reads back
    // out of the min/max order rather than from a flag.
    let mut reverse = false;
    if pat_byte(rc, 0) == b'-' {
        pat_seek(rc, 1);
        reverse = true;
    }
    let first_byte = pat_byte(rc, 0);
    *minval = take_digits(rc, 0);
    if pat_byte(rc, 0) == b',' {
        pat_seek(rc, 1);
        *maxval = if ascii_isdigit(pat_byte(rc, 0) as c_int) {
            take_digits(rc, MAX_LIMIT)
        } else {
            MAX_LIMIT
        };
    } else if ascii_isdigit(first_byte as c_int) {
        // `{n}` is exactly n.
        *maxval = *minval;
    } else {
        *maxval = MAX_LIMIT;
    }
    if pat_byte(rc, 0) == b'\\' {
        pat_seek(rc, 1);
    }
    if pat_byte(rc, 0) != b'}' {
        let prefix = if rc.magic == MAGIC_ALL { "" } else { "\\" };
        semsg!("E554: Syntax error in {prefix}{{...}}");
        rc_did_emsg.set(true);
        return Err(Failed);
    }
    if (!reverse && *minval > *maxval) || (reverse && *minval < *maxval) {
        core::mem::swap(minval, maxval);
    }
    skipchr(rc);
    Ok(())
}
