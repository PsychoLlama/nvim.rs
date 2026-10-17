//! The pattern cursor both engines parse through.
//!
//! Everything below reads the compile's one cursor,
//! [`RegCompiler::cursor`], and the one-token lookbehind/lookahead around
//! it (`prev_token`, `token`, `next_token` and the `at_start` flags). [`peekchr`] is where a pattern
//! byte becomes a token: a metacharacter is returned as its byte minus
//! 256, so callers can tell `*` (a repeat) from `\*` (a literal) by sign,
//! and which characters are metacharacters depends on 'magic' — which is
//! why this has to be one shared, stateful reader rather than a pure
//! function of the byte.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::message_fmt::msg_bytes;
use crate::option::cpo_has;
use crate::regexp::RegCompiler;
use crate::semsg;
use crate::types::CpoFlag;
use core::ffi::{c_char, c_int};

use super::mbyte::{char_at_nul, char_len_at, cluster_len_at};
use super::{
    MAGIC_ALL, MAGIC_NONE, MAGIC_OFF, MAGIC_ON, MAX_LIMIT, MULTI_MULT, MULTI_ONE, Magic, NOT_MULTI,
    ParseState, REGEXP_ABBR, REGEXP_INRANGE, backslash_abbr, take_bracketed, take_char_class,
    toggle_magic, unmagic,
};
use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{getdigits_int_at, hex2nr};
use crate::regexp::state::rc_did_emsg;
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

/// Skip past a `[]` collection, `text[p]` being the byte just after the `[`.
/// Stops at the closing `]` or at the pattern's NUL, and answers where.
///
/// `text` is the pattern with its NUL; nothing past the NUL is read.
pub(crate) fn skip_anyof(text: &[u8], mut p: usize, cpo_lit: bool) -> usize {
    let byte = |i: usize| cstr::byte_at(text, i);
    // A leading `^` negates; a `]` or `-` immediately after that is
    // literal rather than the close or a range.
    if byte(p) == b'^' {
        p += 1;
    }
    if matches!(byte(p), b']' | b'-') {
        p += 1;
    }
    while !matches!(byte(p), 0 | b']') {
        let len = cluster_len_at(text, p);
        if len > 1 {
            p += len;
        } else if byte(p) == b'-' {
            p += 1;
            if !matches!(byte(p), 0 | b']') {
                p += cluster_len_at(text, p);
            }
        } else if byte(p) == b'\\'
            && (is_inrange(byte(p + 1)) || (!cpo_lit && is_abbr(byte(p + 1))))
        {
            p += 2;
        } else if byte(p) == b'[' {
            // A `[:class:]`, `[=equi=]` or `[.coll.]` advances `p`
            // itself; a bare `[` is literal.
            if take_char_class(text, &mut p).is_none()
                && take_bracketed(text, &mut p, b'=') == 0
                && take_bracketed(text, &mut p, b'.') == 0
                && byte(p) != 0
            {
                p += 1;
            }
        } else {
            p += 1;
        }
    }
    p
}

/// `byte` as a delimiter the way upstream's `skip_regexp(p + 1, *p, ..)`
/// hands it on: a `char`, so a byte at or above 0x80 arrives sign-extended
/// where `char` is signed -- and the walk compares each pattern byte the
/// same way, so such a delimiter still ends the pattern.
///
/// Upstream is not uniform here. `:global`, `:sort`, `:uniq`, `:catch`,
/// `:match` and `:syntax` pass `*p`; `:substitute`, `:filter`/`:vimgrep`,
/// a `/pat/` address and the completion walks pass `(uint8_t)*p`, which
/// never equals a sign-extended pattern byte, so there a high delimiter
/// never ends the pattern. Each caller keeps its upstream form.
pub(crate) fn char_delim(byte: u8) -> c_int {
    c_int::from(c_char::from_ne_bytes([byte]))
}

/// The pattern text a skip walks: read only, or the caller's to rewrite.
enum PatText<'a> {
    Read(&'a [u8]),
    Write(&'a mut [u8]),
}

impl PatText<'_> {
    fn bytes(&self) -> &[u8] {
        match self {
            PatText::Read(text) => text,
            PatText::Write(text) => text,
        }
    }
}

/// What a skip found: where the pattern ends, the 'magic' in force there,
/// and whether a `?`-delimited walk met a `\?` it would unescape.
struct Skipped {
    end: usize,
    magic: Magic,
    unescapes: bool,
}

/// The walk every skip is: from `text[start]` to `dirc` or the NUL, tracking
/// `\v`/`\V` and stepping over `[]` collections, which may hold the
/// delimiter. Under a `?` delimiter an escaped `\?` (which a `?` search
/// cannot contain otherwise) is unescaped in place when the text is the
/// caller's to write, the rest of the string, terminator included, pulled
/// down over the backslash.
///
/// The text is NUL-terminated at or after `start`.
fn skip_walk(mut text: PatText<'_>, start: usize, dirc: c_int, magic: c_int) -> Skipped {
    let mut mymagic = if magic != 0 { MAGIC_ON } else { MAGIC_OFF };
    let cpo_lit = cpo_has(CpoFlag::LITERAL);
    let mut unescapes = false;
    let mut p = start;
    loop {
        let bytes = text.bytes();
        let byte = |i: usize| cstr::byte_at(bytes, i);
        // Upstream compares the byte as a (signed) `char`; see
        // [`char_delim`].
        if byte(p) == 0 || char_delim(byte(p)) == dirc {
            break;
        }
        if (byte(p) == b'[' && mymagic >= MAGIC_ON)
            || (byte(p) == b'\\' && byte(p + 1) == b'[' && mymagic <= MAGIC_OFF)
        {
            p = skip_anyof(bytes, p + 1, cpo_lit);
            if byte(p) == 0 {
                break;
            }
        } else if byte(p) == b'\\' && byte(p + 1) != 0 {
            if dirc == c_int::from(b'?') && byte(p + 1) == b'?' {
                unescapes = true;
                if let PatText::Write(text) = &mut text {
                    let nul = p + 1 + cstr::until_nul(&text[p + 1..]).len();
                    text.copy_within(p + 1..=nul, p);
                } else {
                    p += 1;
                }
            } else {
                p += 1;
            }
            let bytes = text.bytes();
            match cstr::byte_at(bytes, p) {
                b'v' => mymagic = MAGIC_ALL,
                b'V' => mymagic = MAGIC_NONE,
                _ => {}
            }
        }
        p += cluster_len_at(text.bytes(), p);
    }
    Skipped {
        end: p,
        magic: mymagic,
        unescapes,
    }
}

/// How many bytes of `text` the pattern it starts with takes, stopping at
/// `delim` or at the NUL that ends the string, and the 'magic' in force at
/// the end. `magic` is the initial 'magic' setting; `\v`/`\V` inside the
/// pattern change it as the scan proceeds. Nothing is written: a `\?` under
/// a `?` delimiter is stepped over as an escape.
///
/// # Panics
/// If `text` holds no NUL at all.
pub(crate) fn skip_regexp_magic(text: &[u8], delim: c_int, magic: c_int) -> (usize, Magic) {
    assert!(text.contains(&0), "the walk needs a terminator to stop at");
    let skipped = skip_walk(PatText::Read(text), 0, delim, magic);
    (skipped.end, skipped.magic)
}

/// [`skip_regexp_magic`]'s end alone: the slice form an address or a
/// command-line parse wants.
///
/// # Panics
/// If `text` holds no NUL at all.
pub(crate) fn skip_regexp_at(text: &[u8], delim: c_int, magic: c_int) -> usize {
    skip_regexp_magic(text, delim, magic).0
}

/// [`skip_regexp_at`] for a pattern the caller may have rewritten: where
/// the pattern starting at `text[at]` ends, as an offset into `text`.
///
/// With a `?` delimiter an escaped `\?` is unescaped **in place**, the rest
/// of the string (terminator included) pulled down over the backslash --
/// what upstream's `skip_regexp_ex(.., &eap->arg, ..)` does to a command
/// line, since the slot it hands over is never empty and so never asks for
/// a copy.
///
/// # Panics
/// If `text` holds no NUL at or after `at`.
pub(crate) fn skip_regexp_in_place(
    text: &mut [u8],
    at: usize,
    delim: c_int,
    magic: c_int,
) -> usize {
    assert!(
        text[at..].contains(&0),
        "the walk needs a terminator to stop at"
    );
    skip_walk(PatText::Write(text), at, delim, magic).end
}

/// [`skip_regexp_at`] for a `?` search that cannot write its pattern: when
/// the pattern holds an escaped `\?`, the skip runs over a copy with those
/// unescaped -- upstream's `skip_regexp_ex` handed an empty `newp` slot --
/// and the copy comes back, its terminator left off, with the end an offset
/// into it rather than into `text`.
///
/// # Panics
/// If `text` holds no NUL at all.
pub(crate) fn skip_regexp_copy(
    text: &[u8],
    delim: c_int,
    magic: c_int,
) -> (usize, Option<Vec<u8>>) {
    assert!(text.contains(&0), "the walk needs a terminator to stop at");
    let skipped = skip_walk(PatText::Read(text), 0, delim, magic);
    if !skipped.unescapes {
        return (skipped.end, None);
    }
    let mut copy = cstr::until_nul(text).to_vec();
    copy.push(0);
    let end = skip_walk(PatText::Write(&mut copy), 0, delim, magic).end;
    // Each unescape pulled the terminator down a byte and left the old one
    // behind it.
    copy.truncate(cstr::until_nul(&copy).len());
    (end, Some(copy))
}

/// [`skip_regexp_at`], but complain when the delimiter is missing: how many
/// bytes of `text` the pattern takes, or `None` when the closing delimiter
/// is missing (which reports E654).
///
/// # Panics
/// If `text` holds no NUL at all.
pub(crate) fn skip_regexp_err_at(text: &[u8], delim: c_int, magic: c_int) -> Option<usize> {
    let end = skip_regexp_at(text, delim, magic);
    // Upstream's `*p != delim`: the byte promoted as a `char`.
    if char_delim(cstr::byte_at(text, end)) != delim {
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

/// The byte `off` bytes past the cursor; NUL past the pattern's end.
pub(crate) fn pat_byte(rc: &RegCompiler, off: usize) -> u8 {
    cstr::byte_at(rc.text, rc.cursor.wrapping_add(off))
}

/// The character `off` bytes past the cursor.
pub(crate) fn pat_char(rc: &RegCompiler, off: usize) -> c_int {
    char_at_nul(rc.text, rc.cursor.wrapping_add(off))
}

/// The encoded length of the character `off` bytes past the cursor; 0 at
/// the pattern's NUL.
pub(crate) fn pat_charlen(rc: &RegCompiler, off: usize) -> c_int {
    char_len_at(rc.text, rc.cursor.wrapping_add(off)) as c_int
}

/// Move the cursor. Wrapping arithmetic because [`peekchr`] and
/// [`ungetchr`] step it back over a character they have already read.
pub(crate) fn pat_seek(rc: &mut RegCompiler, delta: isize) {
    rc.cursor = rc.cursor.wrapping_add_signed(delta);
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
                rc.token -= 256;
            }
        }
        b'(' | b')' | b'{' | b'%' | b'+' | b'=' | b'?' | b'@' | b'!' | b'&' | b'|' | b'<'
        | b'>' | b'#' | b'"' | b'\'' | b',' | b'-' | b':' | b';' | b'`' | b'/' => {
            // Magic only under `\v`.
            if rc.magic == MAGIC_ALL {
                rc.token -= 256;
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
        rc.prev_token_len += pat_charlen(rc, rc.prev_token_len as usize);
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
    let (number, past) = getdigits_int_at(rc.text, rc.cursor, false, default);
    rc.cursor = past;
    number
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
