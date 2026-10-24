//! Character classification and display translation.
//!
//! Three tables decide everything here:
//!
//! - `g_chartab`, one byte per 8-bit character, holding its display width in
//!   the low three bits plus the 'isident' / 'isprint' / 'isfname' flags.
//! - each buffer's `b_chartab`, a 256-bit set built from 'iskeyword'.
//! - `utf8len_tab` and the `utf_*` classifiers in `mbyte.rs`, for anything
//!   past U+00FF.
//!
//! Nearly every function here runs per character on a parsing or drawing
//! path, which is why every one-line helper is `#[inline(always)]`: the test
//! suites run unoptimised builds in which nothing else is inlined. The text
//! they read is a `&[u8]`; the pointer spellings upstream's callers still use
//! are one-line shims in `walk.rs`, the one file here that allows `unsafe`.

// No `unsafe` here, and none allowed: the crate denies it. Not `forbid`,
// because a `forbid` reaches the whole subtree and `walk.rs` keeps the
// pointer shims, which need the allow.
#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int, c_uint};

use crate::global_cell::GlobalCell;
use crate::mbyte::{char_at, mb_islower, mb_isupper, utf_class_tab, utf_printable, utf8len_tab};
use crate::option::vars::{breakat_flags, dy_flags, p_isf, p_isi, p_isp};
use crate::options::kOptDyFlagUhex;
use crate::optionstr::OptStringRef;
use crate::path::has_wildcard;
use crate::types::{Failed, UVarNumber, VarNumber, uint8_t, uint64_t};

pub mod display;
pub mod skip;
pub mod str2nr;
pub use str2nr::Str2NrBases;
pub(crate) use str2nr::atoi;
pub mod transchar;

// The display half was split out for size; its callers are spread over three
// dozen modules and name it as `charset::*`.
pub use display::transchar_hex_into;
pub(crate) use display::{
    CHAR_DISPLAY_LEN, CharDisplay, mirror_ascii, ptr2cells_at, strnsize, strsize,
    trans_characters_in, transchar, transchar_buf, transchar_byte, transchar_byte_buf,
    transchar_nonprint_form, transstr_into, transstr_owned, vim_strsize_cstr,
};
pub use display::{byte2cells, char2cells};

// The pointer spellings, for the callers whose cursor is still a pointer.
// The module itself is private: its names are re-exported here.
mod walk;

pub use crate::mbyte::{vim_strnsize, vim_strsize};
pub use walk::{
    backslash_halve, backslash_halve_save, getdigits, getdigits_int, getdigits_int32,
    rem_backslash, skipbin, skipdigits, skiphex, skiptowhite, skiptowhite_esc, skipwhite,
    vim_iswordp, vim_iswordp_buf, vim_str2nr,
};
pub(crate) use walk::{
    getdigits_at, getdigits_int_at, getwhitecols_curline, try_getdigits, try_getdigits_at,
};
pub use walk::{
    ptr2cells, rl_mirror_ascii, str_foldcase, trans_characters, transstr, transstr_buf,
};

/// Bits of a `g_chartab` entry.
const CT_CELL_MASK: uint8_t = 0x7;
const CT_PRINT_CHAR: uint8_t = 0x10;
const CT_ID_CHAR: uint8_t = 0x20;
const CT_FNAME_CHAR: uint8_t = 0x40;

const TAB: c_int = 9;
const NL: c_int = 10;
const EOL_MAC: c_int = 2;

static chartab_initialized: GlobalCell<bool> = GlobalCell::new(false);
static g_chartab: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);

/// The `g_chartab` entry for byte `c`.
///
/// It is asked about once per byte on every draw and edit path, so it reads
/// the one entry rather than copying 256 bytes or taking a tracked borrow.
#[inline(always)]
fn chartab(c: uint8_t) -> uint8_t {
    g_chartab.get_field(|table| &table[usize::from(c)])
}

/// Overwrite the `g_chartab` entry for byte `c`.
#[inline(always)]
fn set_chartab(c: uint8_t, value: uint8_t) {
    g_chartab.set_field(|table| &table[usize::from(c)], value);
}

/// Add or remove one of the flag bits of `c`'s entry.
#[inline(always)]
fn set_chartab_flag(c: uint8_t, flag: uint8_t, on: bool) {
    let entry = chartab(c);
    set_chartab(c, if on { entry | flag } else { entry & !flag });
}

/// The cell width an unprintable byte is displayed with: four for `<xx>`,
/// two for `^X`.
#[inline(always)]
fn unprintable_width() -> uint8_t {
    if dy_flags.get() & kOptDyFlagUhex != 0 {
        4
    } else {
        2
    }
}

/// `ascii.h`'s classifiers, spelled out rather than deferred to `u8`'s own
/// (which are only `#[inline]`, so a debug build calls them once per byte).
#[inline(always)]
fn is_white(byte: uint8_t) -> bool {
    byte == b' ' || byte == b'\t'
}

#[inline(always)]
fn is_digit(byte: uint8_t) -> bool {
    byte.is_ascii_digit()
}

#[inline(always)]
fn is_bdigit(byte: uint8_t) -> bool {
    byte == b'0' || byte == b'1'
}

#[inline(always)]
fn is_odigit(byte: uint8_t) -> bool {
    (b'0'..=b'7').contains(&byte)
}

#[inline(always)]
fn is_xdigit(byte: uint8_t) -> bool {
    is_digit(byte) || (byte | 0x20) >= b'a' && (byte | 0x20) <= b'f'
}

/// Rebuild the global table and the current buffer's keyword set.
pub fn init_chartab() -> bool {
    buf_init_chartab(Buf::current(), true)
}

/// Rebuild `buffer`'s keyword set from 'iskeyword', and — when `global` — the
/// shared table from 'isident', 'isprint' and 'isfname' too.
///
/// Answers whether every option parsed; a malformed one leaves the tables
/// half-built, which is why the option code validates with [`check_isopt`]
/// before assigning.
pub fn buf_init_chartab(mut buffer: Buf, global: bool) -> bool {
    if global {
        // Control characters display as `^X` or `<xx>`; printable ASCII is
        // one cell wide; the Latin-1 upper half is printable and valid in a
        // file name, but 0x7f-0x9f are not.
        for c in 0..b' ' {
            set_chartab(c, unprintable_width());
        }
        for c in b' '..=b'~' {
            set_chartab(c, 1 + CT_PRINT_CHAR);
        }
        for c in b'~' as u16 + 1..256 {
            let c = c as uint8_t;
            let entry = if c >= 0xa0 {
                (CT_PRINT_CHAR | CT_FNAME_CHAR) + 1
            } else {
                unprintable_width()
            };
            set_chartab(c, entry);
        }
    }

    buffer.b_chartab = [0; 4];
    if buffer.b_p_lisp != 0 {
        // In Lisp, `-` belongs to a word even when 'iskeyword' omits it.
        set_buf_chartab(buffer, b'-' as c_int, true);
    }

    // The first three are the global options; the last is the buffer's own
    // 'iskeyword'. Which table an option fills is the option's identity,
    // which upstream recovered by comparing the value's *address* against
    // `p_isi` and friends; it is named here instead.
    let fill = |var: &CStr, table| parse_isopt(var.to_bytes(), table, Some(buffer), false);
    if global
        && (p_isi(|var| fill(var, IsoptTable::Ident)).is_err()
            || p_isp(|var| fill(var, IsoptTable::Print)).is_err()
            || p_isf(|var| fill(var, IsoptTable::Fname)).is_err())
    {
        return false;
    }
    let keywords = buffer.b_p_isk.bytes().to_vec();
    if parse_isopt(&keywords, IsoptTable::Keyword, Some(buffer), false).is_err() {
        return false;
    }
    chartab_initialized.set(true);
    true
}

/// Whether `var` is a well-formed 'isident'-style option value.
pub fn check_isopt(var: &[u8]) -> Result<(), Failed> {
    // A check pass never touches a table, so it needs no buffer.
    parse_isopt(var, IsoptTable::Keyword, None, true)
}

/// Set or clear `c`'s bit in `buffer`'s keyword set.
#[inline(always)]
fn set_buf_chartab(mut buffer: Buf, c: c_int, on: bool) {
    let word = (c as c_uint >> 6) as usize;
    let bit = 1u64 << (c & 0x3f);
    buffer.b_chartab[word] = (buffer.b_chartab[word] & !bit) | if on { bit } else { 0 };
}

/// Which of the four tables an 'isident'-style option fills.
#[derive(Clone, Copy)]
enum IsoptTable {
    Ident,
    Print,
    Fname,
    Keyword,
}

/// One entry of such an option: a character or a range of them, added to
/// the table or — with a leading `^` — removed from it.
struct IsoptEntry {
    tilde: bool,
    /// A lone `@` stands for "every alphabetic character in the locale",
    /// which is the whole 1-255 range filtered by the `mb_is*` predicates.
    alpha_only: bool,
    first: c_int,
    last: c_int,
}

/// The character an entry begins with: either a decimal number naming a
/// code point, or one (possibly multibyte) character. `at` is left after
/// it.
fn isopt_char(var: &[u8], at: &mut usize) -> c_int {
    let rest = &var[*at..];
    if is_digit(cstr::byte_at(rest, 0)) {
        let (c, used) = skip::decimal_int(rest);
        *at += used;
        c
    } else {
        *at += crate::mbyte::cluster_len(rest);
        char_at(rest)
    }
}

/// Read the entry at `at`, leaving `at` on the next one. `None` means the
/// option value is malformed.
fn next_isopt_entry(var: &[u8], at: &mut usize) -> Option<IsoptEntry> {
    let byte = |i: usize| cstr::byte_at(var, i);
    // A leading `^` removes the range instead of adding it.
    let tilde = byte(*at) == b'^' && byte(*at + 1) != 0;
    if tilde {
        *at += 1;
    }

    let first = isopt_char(var, at);
    let mut last: c_int = -1;
    if byte(*at) == b'-' && byte(*at + 1) != 0 {
        *at += 1;
        last = isopt_char(var, at);
    }

    if first <= 0 || first >= 256 || (last < first && last != -1) || last >= 256 {
        return None;
    }
    let separator = byte(*at);
    if separator != 0 && separator != b',' {
        return None;
    }
    // Past the comma and any spaces after it: `skip_to_option_part`.
    if separator == b',' {
        *at += 1;
    }
    while byte(*at) == b' ' {
        *at += 1;
    }
    // A trailing comma with nothing after it is malformed.
    if separator == b',' && byte(*at) == 0 {
        return None;
    }

    let (alpha_only, first, last) = match (last, first) {
        (-1, c) if c == '@' as c_int => (true, 1, 255),
        (-1, c) => (false, c, c),
        (last, c) => (false, c, last),
    };
    Some(IsoptEntry {
        tilde,
        alpha_only,
        first,
        last,
    })
}

/// Apply one entry to the table it belongs to.
fn apply_isopt_entry(table: IsoptTable, entry: &IsoptEntry, buffer: Buf) {
    for c in entry.first..=entry.last {
        // The `mb_` predicates rather than `isalpha`, which misreads the
        // Latin-1 upper half under the C locale.
        if entry.alpha_only && !mb_islower(c) && !mb_isupper(c) {
            continue;
        }
        let byte = c as uint8_t;
        match table {
            IsoptTable::Ident => set_chartab_flag(byte, CT_ID_CHAR, !entry.tilde),
            IsoptTable::Fname => set_chartab_flag(byte, CT_FNAME_CHAR, !entry.tilde),
            // 'isprint' cannot demote printable ASCII.
            IsoptTable::Print if c < ' ' as c_int || c > '~' as c_int => {
                let width = if entry.tilde { unprintable_width() } else { 1 };
                set_chartab(byte, (chartab(byte) & !CT_CELL_MASK) + width);
                set_chartab_flag(byte, CT_PRINT_CHAR, !entry.tilde);
            }
            IsoptTable::Print => {}
            IsoptTable::Keyword => set_buf_chartab(buffer, c, !entry.tilde),
        }
    }
}

/// Walk one 'isident'/'isprint'/'isfname'/'iskeyword' value, applying each
/// comma-separated range to `table`, which is the option's. With
/// `only_check` no table is touched and only the syntax is validated. A NUL
/// in `var` ends it.
fn parse_isopt(
    var: &[u8],
    table: IsoptTable,
    buffer: Option<Buf>,
    only_check: bool,
) -> Result<(), Failed> {
    let mut at = 0;
    while cstr::byte_at(var, at) != 0 {
        let Some(entry) = next_isopt_entry(var, &mut at) else {
            return Err(Failed);
        };
        if let (false, Some(buffer)) = (only_check, buffer) {
            apply_isopt_entry(table, &entry, buffer);
        }
    }
    Ok(())
}

/// Whether `c` may appear in an identifier ('isident').
pub fn vim_is_ident_char(c: c_int) -> bool {
    c > 0 && c < 0x100 && chartab(c as uint8_t) & CT_ID_CHAR != 0
}

/// Whether `c` belongs to a word in the current buffer ('iskeyword').
pub fn vim_iswordc(c: c_int) -> bool {
    vim_iswordc_buf(c, Buf::current())
}

/// [`vim_iswordc`] as a safe call, for a module that holds no pointer of its
/// own to state a precondition about.
///
/// `curbuf` is a live buffer from the first `buflist_new` onwards, which is
/// before anything that can ask this question runs — the null it starts out
/// as exists only for the span of `early_init`. So the pointer form's
/// precondition is not one a caller here could get wrong, and spelling it as
/// a `# Safety` section would only push an `unsafe` block into every caller
/// to discharge an obligation none of them can influence.
pub fn is_word_char(c: c_int) -> bool {
    vim_iswordc(c)
}

/// Whether `c` belongs to a word according to the 256-bit set `chartab`.
/// Characters past U+00FF are decided by their Unicode class instead.
pub fn vim_iswordc_tab(c: c_int, chartab: &[uint64_t; 4]) -> bool {
    if c >= 0x100 {
        return utf_class_tab(c, chartab) >= 2;
    }
    c > 0 && chartab[(c as c_uint >> 6) as usize] & (1u64 << (c & 0x3f)) != 0
}

/// Whether `c` belongs to a word in `buffer`.
pub fn vim_iswordc_buf(c: c_int, buffer: Buf) -> bool {
    vim_iswordc_tab(c, &buffer.b_chartab)
}

/// Whether the character at the start of `bytes` belongs to a word in
/// `buffer`. A byte that leads no multibyte sequence is judged as itself.
#[inline]
pub(crate) fn is_word_at(bytes: &[u8], buffer: Buf) -> bool {
    let lead = cstr::byte_at(bytes, 0);
    let c = if utf8len_tab[usize::from(lead)] > 1 {
        char_at(bytes)
    } else {
        c_int::from(lead)
    };
    vim_iswordc_buf(c, buffer)
}

/// Whether `c` may appear in a file name ('isfname'). Everything past
/// U+00FF is allowed.
pub fn vim_isfilec(c: c_int) -> bool {
    c >= 0x100 || (c > 0 && chartab(c as uint8_t) & CT_FNAME_CHAR != 0)
}

/// Like [`vim_isfilec`], but also accepts the separators that may appear in
/// a file name given on a command line.
pub fn vim_is_fname_char(c: c_int) -> bool {
    (vim_isfilec(c))
        || c == ',' as c_int
        || c == ' ' as c_int
        || c == '@' as c_int
        || c == ':' as c_int
}

/// Like [`vim_isfilec`], but also accepts the wildcards a file name pattern
/// may contain.
pub fn vim_isfilec_or_wc(c: c_int) -> bool {
    vim_isfilec(c) || c == ']' as c_int || has_wildcard(&[c as u8][..usize::from(c as u8 != 0)])
}

/// Whether `c` displays as itself ('isprint').
pub fn vim_isprintc(c: c_int) -> bool {
    if c >= 0x100 {
        return utf_printable(c);
    }
    c > 0 && chartab(c as uint8_t) & CT_PRINT_CHAR != 0
}
/// A lazy cursor over the digits of a number, bounded either by `maxlen`
/// bytes or by the first byte that is not a digit. The end of `text` reads
/// as the NUL a C string would have there.
struct Scan<'a> {
    text: &'a [u8],
    at: usize,
    maxlen: c_int,
}

impl Scan<'_> {
    #[inline(always)]
    fn consumed(&self) -> c_int {
        c_int::try_from(self.at).unwrap_or(c_int::MAX)
    }

    /// Whether `offset` more bytes are still within `maxlen`. A `maxlen` of
    /// zero means "to the end of the text".
    #[inline(always)]
    fn within(&self, offset: c_int) -> bool {
        self.maxlen == 0 || self.consumed() + offset < self.maxlen
    }

    /// The byte `offset` further on.
    #[inline(always)]
    fn at(&self, offset: c_int) -> uint8_t {
        cstr::byte_at(self.text, self.at + offset as usize)
    }

    #[inline(always)]
    fn advance(&mut self, n: isize) {
        self.at = self.at.wrapping_add_signed(n);
    }
}

/// What [`str2nr_in`] read: the number of bytes the number took (zero when
/// there was none, or a strict parse rejected it), its value signed and
/// unsigned, and whether the signed value was clamped.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ParsedNumber {
    pub(crate) len: usize,
    /// The prefix letter that named the base (`x`, `b`, `o`, ...), `0` for a
    /// leading-zero octal, or zero for none.
    pub(crate) prefix: c_int,
    pub(crate) value: VarNumber,
    pub(crate) magnitude: UVarNumber,
    pub(crate) overflow: bool,
    /// A strict parse found a letter or digit after the number: nothing but
    /// `len` (zero) and `overflow` is set.
    pub(crate) rejected: bool,
}

/// Parse the number `text` starts with, in the bases `what` allows.
///
/// `maxlen` bounds the scan (zero means "to the end of `text`"), and
/// `strict` makes a trailing letter or digit reject the parse outright,
/// leaving `len` at zero. `want_signed` says the caller reads the signed
/// value, which is the only case in which clamping it raises `overflow` --
/// upstream reports a clamp from the branch that writes `nptr` alone.
///
/// The slice form of `vim_str2nr`, whose scan is [`str2nr::scan`].
pub(crate) fn str2nr_scan(
    text: &[u8],
    maxlen: c_int,
    what: Str2NrBases,
    strict: bool,
    want_signed: bool,
) -> ParsedNumber {
    let negative = cstr::byte_at(text, 0) == b'-';
    let mut scan = Scan {
        text,
        at: usize::from(negative),
        maxlen,
    };
    let Some(parsed) = str2nr::scan(&mut scan, what) else {
        // `what` forces a base its remaining flags do not name.
        std::process::abort();
    };

    // A strict parse only accepts a number that ends the string (or fills
    // `maxlen`); anything alphanumeric after it means this was not a number.
    let rejected = strict && scan.consumed() != maxlen && str2nr::strict_reject(scan.at(0));
    let (value, clamped) = str2nr::signed(parsed.magnitude, negative);
    let overflow = parsed.overflowed || (clamped && !rejected && want_signed);
    if rejected {
        return ParsedNumber {
            overflow,
            rejected,
            ..ParsedNumber::default()
        };
    }
    ParsedNumber {
        len: scan.at,
        prefix: parsed.pre,
        value,
        magnitude: parsed.magnitude,
        overflow,
        rejected,
    }
}

/// The number at the start of `text`, read in the bases `what` allows. The
/// end of the slice ends the number, as the NUL did for the pointer form;
/// `strict` rejects one followed by a letter or a digit.
pub(crate) fn str2nr_in(text: &[u8], what: Str2NrBases, strict: bool) -> ParsedNumber {
    // Zero would mean "to the end", and an empty slice has no first byte to
    // read; neither holds a number.
    let Ok(maxlen @ 1..) = c_int::try_from(text.len()) else {
        return ParsedNumber::default();
    };
    str2nr_scan(text, maxlen, what, strict, true)
}

/// Read a Float out of `text`, answering it and how many bytes it consumed.
/// The three named values are recognised ahead of `strtod`, which does not
/// know them in every locale. The float ends where the slice does.
pub(crate) fn string2float_in(text: &[u8]) -> (crate::types::Float, usize) {
    for (name, value) in [
        (&b"inf"[..], f64::INFINITY),
        (b"-inf", f64::NEG_INFINITY),
        (b"nan", f64::NAN),
    ] {
        if text
            .get(..name.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(name))
        {
            return (value, name.len());
        }
    }
    crate::os::cshim::strtod_prefix(text)
}

/// The value of the hexadecimal digit `c`. Anything else is nonsense.
pub fn hex2nr(c: c_int) -> c_int {
    if (b'a' as c_int..=b'f' as c_int).contains(&c) {
        return c - 'a' as c_int + 10;
    }
    if (b'A' as c_int..=b'F' as c_int).contains(&c) {
        return c - 'A' as c_int + 10;
    }
    c - '0' as c_int
}

/// The byte the two hexadecimal digits `text` starts with spell, or -1. The
/// end of the slice reads as a NUL, which is not a digit.
pub fn hexhex_at(text: &[u8]) -> c_int {
    let (high, low) = (cstr::byte_at(text, 0), cstr::byte_at(text, 1));
    if !is_xdigit(high) || !is_xdigit(low) {
        return -1;
    }
    (hex2nr(high as c_int) << 4) + hex2nr(low as c_int)
}

/// Whether the backslash `text` starts with escapes the byte after it, i.e.
/// whether [`backslash_halved`] would remove it.
pub(crate) fn escapes_next(text: &[u8]) -> bool {
    text.first() == Some(&b'\\') && text.get(1).is_some_and(|&next| next != 0)
}

/// Whether `text` holds nothing but white space, up to a line end or a NUL.
pub(crate) fn is_blank_line(text: &[u8]) -> bool {
    matches!(cstr::byte_at(text, skip::white(text)), 0 | b'\r' | b'\n')
}

/// `text` with each backslash that escapes the byte after it removed, up to
/// the first NUL.
pub(crate) fn backslash_halved(text: &[u8]) -> Vec<u8> {
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    // Room for a terminator too, for a caller that hands the result to C.
    let mut out = Vec::with_capacity(text.len() + 1);
    // Copied a run at a time: most names hold no backslash at all.
    let mut rest = text;
    while let Some(n) = rest.iter().position(|&b| b == b'\\') {
        out.extend_from_slice(&rest[..n]);
        // A backslash escapes the byte after it, when there is one: no NUL
        // is left in `text` for it to stand before.
        match rest.get(n + 1) {
            Some(&escaped) => {
                out.push(escaped);
                rest = &rest[n + 2..];
            }
            None => {
                out.push(b'\\');
                rest = &[];
            }
        }
    }
    out.extend_from_slice(rest);
    out
}

/// Whether a line may be broken before `c`, per the 'breakat' option.
pub fn vim_isbreak(c: c_int) -> bool {
    breakat_flags.get().has(c as uint8_t)
}

/// Fill `g_chartab` with the classification a freshly started editor has, for
/// in-crate tests.
///
/// [`buf_init_chartab`] is the real thing and needs a buffer and four option
/// strings, none of which the library test harness has. This writes the same
/// answers for the bytes a test can ask about: the global half of that
/// function verbatim, plus the `CT_ID_CHAR` bits 'isident's default
/// (`@,48-57,_,192-255`) sets — `@` being the alphabetic characters, which is
/// ASCII's letters here.
///
/// It runs once per test binary and synchronises every caller with the write,
/// so a test that calls it before reading the table is not racing the others.
/// A test that reads the table *without* calling it would be, which is why
/// every predicate in the crate that consults it says so in its own tests.
#[cfg(test)]
pub(crate) fn init_chartab_for_tests() {
    static ONCE: ::std::sync::Once = ::std::sync::Once::new();
    ONCE.call_once(|| {
        for c in 0..b' ' {
            set_chartab(c, unprintable_width());
        }
        for c in b' '..=b'~' {
            set_chartab(c, 1 + CT_PRINT_CHAR);
        }
        for c in b'~' as u16 + 1..256 {
            let c = c as uint8_t;
            let entry = if c >= 0xa0 {
                (CT_PRINT_CHAR | CT_FNAME_CHAR) + 1
            } else {
                unprintable_width()
            };
            set_chartab(c, entry);
        }
        for c in 0..=255u8 {
            if c.is_ascii_alphanumeric() || c == b'_' || c >= 192 {
                set_chartab(c, chartab(c) | CT_ID_CHAR);
            }
        }
    });
}
