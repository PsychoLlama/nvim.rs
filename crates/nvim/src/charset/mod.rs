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
//! Nearly every function here walks a raw C string, and nearly every one of
//! them runs per character on a parsing or drawing path. Both facts shape
//! the code: the pointer walks go through [`Bytes`], a cursor whose
//! *construction* is the unsafe step and whose reads are then ordinary safe
//! code, and every one-line helper is `#[inline(always)]`, because the test
//! suites run unoptimised builds in which nothing else is inlined.
//!
//! An `unsafe` block here carries a `SAFETY:` note unless it does nothing
//! but hand its own function's documented contract straight to a callee
//! with the same one.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_long, c_uint};
use core::ptr;

use crate::cursor::get_cursor_line_ptr;
use crate::global_cell::GlobalCell;
use crate::main::{breakat_flags, curbuf, dy_flags, p_isf, p_isi, p_isp};
use crate::mbyte::{
    mb_islower, mb_isupper, mb_ptr2char_adv, utf_class_tab, utf_printable, utf_ptr2char,
    utf8len_tab,
};
use crate::memory::{xmalloc, xstrchrnul};
use crate::option::skip_to_option_part;
use crate::options::kOptDyFlagUhex;
use crate::os::cshim::strtoimax;
use crate::path::path_has_wildcard;
use crate::types::{
    FAIL, NUL, OK, buf_T, int32_t, intmax_t, intptr_t, size_t, uint8_t, uint64_t, uvarnumber_T,
    varnumber_T,
};
use ::libc::{__errno_location, abort, strlen};

pub mod display;
pub mod str2nr;
pub mod transchar;

// The display half was split out for size; its callers are spread over three
// dozen modules and name it as `charset::*`.
pub use display::{
    byte2cells, char2cells, ptr2cells, rl_mirror_ascii, str_foldcase, trans_characters, transchar,
    transchar_buf, transchar_byte, transchar_byte_buf, transchar_hex, transchar_nonprint, transstr,
    transstr_buf, transstr_len, vim_strnsize, vim_strsize,
};

use crate::keycodes::Ctrl_V;

/// Bases `vim_str2nr` may recognise, plus the two behaviour flags. `FORCE`
/// says "the string has no prefix, parse it in the base named by the rest";
/// `QUOTE` allows `'` as a digit separator.
pub const STR2NR_DEC: c_int = 0;
pub const STR2NR_BIN: c_int = 1;
pub const STR2NR_OCT: c_int = 2;
pub const STR2NR_HEX: c_int = 4;
pub const STR2NR_OOCT: c_int = 8;
pub const STR2NR_ALL: c_int = STR2NR_BIN | STR2NR_OCT | STR2NR_HEX | STR2NR_OOCT;
pub const STR2NR_FORCE: c_int = 128;
pub const STR2NR_QUOTE: c_int = 16;

/// Bits of a `g_chartab` entry.
const CT_CELL_MASK: uint8_t = 0x7;
const CT_PRINT_CHAR: uint8_t = 0x10;
const CT_ID_CHAR: uint8_t = 0x20;
const CT_FNAME_CHAR: uint8_t = 0x40;

const TAB: c_int = 9;
const NL: c_int = 10;
const EOL_MAC: c_int = 2;
const ERANGE: c_int = 34;

static chartab_initialized: GlobalCell<bool> = GlobalCell::new(false);
static g_chartab: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);

/// The `g_chartab` entry for byte `c`.
///
/// The table is a fixed array of plain bytes that only this module touches,
/// so reading and writing it through the cell's raw pointer forms no
/// reference and can alias nothing. That is what lets the rest of the file
/// treat the table as safe state; these two are the whole unchecked
/// surface of it.
#[inline(always)]
fn chartab(c: uint8_t) -> uint8_t {
    // SAFETY: the cell holds a live 256-byte array and `c` indexes it.
    unsafe { (*g_chartab.ptr())[c as usize] }
}

/// Overwrite the `g_chartab` entry for byte `c`.
#[inline(always)]
fn set_chartab(c: uint8_t, value: uint8_t) {
    // SAFETY: as [`chartab`].
    unsafe { (*g_chartab.ptr())[c as usize] = value }
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

/// A cursor over the bytes of a NUL-terminated string.
///
/// Building one is the unsafe step. Afterwards the cursor only ever steps
/// over bytes that have already been read as non-NUL, so every read is in
/// bounds and the walks themselves are ordinary safe code.
#[derive(Clone, Copy)]
struct Bytes(*const c_char);

impl Bytes {
    /// # Safety
    /// `p` must point into a NUL-terminated string.
    #[inline(always)]
    unsafe fn new(p: *const c_char) -> Self {
        Bytes(p)
    }

    /// The byte under the cursor.
    #[inline(always)]
    fn byte(self) -> uint8_t {
        // SAFETY: the cursor is inside its string, so this byte is readable.
        unsafe { *self.0 as uint8_t }
    }

    /// The byte under the cursor together with the one after it. The second
    /// reads as zero when the first is the terminator — the only case in
    /// which looking ahead would leave the string.
    #[inline(always)]
    fn pair(self) -> (uint8_t, uint8_t) {
        let byte = self.byte();
        // SAFETY: a non-NUL byte is never the last one of the string.
        let next = if byte == 0 {
            0
        } else {
            unsafe { *self.0.add(1) as uint8_t }
        };
        (byte, next)
    }

    /// Step over `n` bytes the caller has established are within the string.
    #[inline(always)]
    fn advance(&mut self, n: usize) {
        self.0 = self.0.wrapping_add(n);
    }

    /// The first byte at or after the cursor that `keep` rejects, or the
    /// terminator.
    #[inline(always)]
    fn skip_while(mut self, keep: impl Fn(uint8_t) -> bool) -> Self {
        loop {
            let byte = self.byte();
            if byte == 0 || !keep(byte) {
                return self;
            }
            self.advance(1);
        }
    }

    #[inline(always)]
    fn raw(self) -> *mut c_char {
        self.0 as *mut c_char
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
///
/// # Safety
/// The current buffer must be valid.
pub unsafe fn init_chartab() -> bool {
    unsafe { buf_init_chartab(curbuf.get(), true) }
}

/// Rebuild `buf`'s keyword set from 'iskeyword', and — when `global` — the
/// shared table from 'isident', 'isprint' and 'isfname' too.
///
/// Answers whether every option parsed; a malformed one leaves the tables
/// half-built, which is why the option code validates with [`check_isopt`]
/// before assigning.
///
/// # Safety
/// `buf` must be a valid buffer.
pub unsafe fn buf_init_chartab(buf: *mut buf_T, global: bool) -> bool {
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

    // SAFETY: `buf` is a valid buffer, so its keyword set is writable.
    unsafe { (*buf).b_chartab = [0; 4] };
    // SAFETY: as above.
    if unsafe { (*buf).b_p_lisp } != 0 {
        // In Lisp, `-` belongs to a word even when 'iskeyword' omits it.
        // SAFETY: as above.
        unsafe { set_buf_chartab(buf, b'-' as c_int, true) };
    }

    // The first three are the global options; the last is the buffer's own
    // 'iskeyword'. Reading all four up front is what the C's loop does one
    // at a time — none of them can move while the tables are being filled.
    // SAFETY: as above.
    let options = [p_isi.get(), p_isp.get(), p_isf.get(), unsafe {
        (*buf).b_p_isk
    }];
    for &option in &options[if global { 0 } else { 3 }..] {
        // SAFETY: an option value is a NUL-terminated string, and `buf` is
        // valid.
        if unsafe { parse_isopt(option, buf, false) } == FAIL {
            return false;
        }
    }
    chartab_initialized.set(true);
    true
}

/// Whether `var` is a well-formed 'isident'-style option value.
///
/// # Safety
/// `var` must be a NUL-terminated string.
pub unsafe fn check_isopt(var: *mut c_char) -> c_int {
    // SAFETY: forwarded; a check pass never touches the (null) buffer.
    unsafe { parse_isopt(var, ptr::null_mut(), true) }
}

/// Set or clear `c`'s bit in `buf`'s keyword set.
///
/// # Safety
/// `buf` must be a valid buffer.
#[inline(always)]
unsafe fn set_buf_chartab(buf: *mut buf_T, c: c_int, on: bool) {
    let word = (c as c_uint >> 6) as usize;
    let bit = 1u64 << (c & 0x3f);
    // SAFETY: `c` is under 256, so `word` is one of the set's four.
    unsafe { (*buf).b_chartab[word] = ((*buf).b_chartab[word] & !bit) | if on { bit } else { 0 } };
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
/// code point, or one (possibly multibyte) character. The cursor is left
/// after it.
///
/// # Safety
/// The cursor must be inside a NUL-terminated string.
unsafe fn isopt_char(cursor: &mut Bytes) -> c_int {
    let mut p = cursor.raw();
    let c = if is_digit(cursor.byte()) {
        // SAFETY: `p` walks a NUL-terminated string and stops on a digit
        // boundary.
        unsafe { getdigits_int(&raw mut p, true, 0) }
    } else {
        // SAFETY: as above, for one whole character.
        unsafe { mb_ptr2char_adv((&raw mut p).cast::<*const c_char>()) }
    };
    // SAFETY: both advanced `p` no further than the terminator.
    *cursor = unsafe { Bytes::new(p) };
    c
}

/// Read the entry at the cursor, leaving it on the next one. `None` means
/// the option value is malformed.
///
/// # Safety
/// The cursor must be inside a NUL-terminated string.
unsafe fn next_isopt_entry(cursor: &mut Bytes) -> Option<IsoptEntry> {
    // A leading `^` removes the range instead of adding it.
    let (byte, next) = cursor.pair();
    let tilde = byte == b'^' && next != 0;
    if tilde {
        cursor.advance(1);
    }

    let first = unsafe { isopt_char(cursor) };
    let mut last: c_int = -1;
    let (byte, next) = cursor.pair();
    if byte == b'-' && next != 0 {
        cursor.advance(1);
        // SAFETY: as above.
        last = unsafe { isopt_char(cursor) };
    }

    if first <= 0 || first >= 256 || (last < first && last != -1) || last >= 256 {
        return None;
    }
    let separator = cursor.byte();
    if separator != 0 && separator != b',' {
        return None;
    }
    // SAFETY: as above.
    *cursor = unsafe { Bytes::new(skip_to_option_part(cursor.raw())) };
    // A trailing comma with nothing after it is malformed.
    if separator == b',' && cursor.byte() == 0 {
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
///
/// # Safety
/// `buf` must be a valid buffer when `table` is the keyword set.
unsafe fn apply_isopt_entry(table: IsoptTable, entry: &IsoptEntry, buf: *mut buf_T) {
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
            IsoptTable::Keyword => unsafe { set_buf_chartab(buf, c, !entry.tilde) },
        }
    }
}

/// Walk one 'isident'/'isprint'/'isfname'/'iskeyword' value, applying each
/// comma-separated range to the table the option owns.
///
/// Which table that is comes from the *identity* of the pointer — `var` is
/// compared against `p_isi`/`p_isp`/`p_isf` — so this cannot be called with
/// a copy of an option's value.
///
/// With `only_check` no table is touched and only the syntax is validated.
///
/// # Safety
/// `var` must be a NUL-terminated string, and `buf` a valid buffer unless
/// `only_check`.
unsafe fn parse_isopt(var: *const c_char, buf: *mut buf_T, only_check: bool) -> c_int {
    let table = if var == p_isi.get().cast_const() {
        IsoptTable::Ident
    } else if var == p_isp.get().cast_const() {
        IsoptTable::Print
    } else if var == p_isf.get().cast_const() {
        IsoptTable::Fname
    } else {
        IsoptTable::Keyword
    };

    let mut cursor = unsafe { Bytes::new(var) };
    while cursor.byte() != 0 {
        // SAFETY: the cursor is still inside `var`.
        let Some(entry) = (unsafe { next_isopt_entry(&mut cursor) }) else {
            return FAIL;
        };
        if !only_check {
            // SAFETY: `buf` is valid whenever an entry is applied.
            unsafe { apply_isopt_entry(table, &entry, buf) };
        }
    }
    OK
}

/// Whether `c` may appear in an identifier ('isident').
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_is_ident_char(c: c_int) -> bool {
    c > 0 && c < 0x100 && chartab(c as uint8_t) & CT_ID_CHAR != 0
}

/// Whether `c` belongs to a word in the current buffer ('iskeyword').
///
/// # Safety
/// The current buffer must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_iswordc(c: c_int) -> bool {
    unsafe { vim_iswordc_buf(c, curbuf.get()) }
}

/// Whether `c` belongs to a word according to the 256-bit set `chartab`.
/// Characters past U+00FF are decided by their Unicode class instead.
///
/// # Safety
/// `chartab` must point at four `uint64_t`s.
pub unsafe fn vim_iswordc_tab(c: c_int, chartab: *const uint64_t) -> bool {
    if c >= 0x100 {
        return unsafe { utf_class_tab(c, chartab) } >= 2;
    }
    // SAFETY: `c` is under 256, so the word index is one of the four.
    c > 0 && unsafe { *chartab.add((c as c_uint >> 6) as usize) } & (1u64 << (c & 0x3f)) != 0
}

/// Whether `c` belongs to a word in `buf`.
///
/// # Safety
/// `buf` must be a valid buffer.
pub unsafe fn vim_iswordc_buf(c: c_int, buf: *mut buf_T) -> bool {
    // SAFETY: a valid buffer carries the four-word keyword set inline.
    unsafe { vim_iswordc_tab(c, (&raw const (*buf).b_chartab).cast()) }
}

/// Whether the character at `p` belongs to a word in the current buffer.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_iswordp(p: *const c_char) -> bool {
    unsafe { vim_iswordp_buf(p, curbuf.get()) }
}

/// Whether the character at `p` belongs to a word in `buf`.
///
/// # Safety
/// `p` must point into a NUL-terminated string and `buf` be a valid buffer.
pub unsafe fn vim_iswordp_buf(p: *const c_char, buf: *mut buf_T) -> bool {
    let lead = unsafe { Bytes::new(p) }.byte();
    let c = if utf8len_tab[lead as usize] > 1 {
        // SAFETY: as above; a lead byte promises the rest of its sequence.
        unsafe { utf_ptr2char(p) }
    } else {
        lead as c_int
    };
    // SAFETY: `buf` is a valid buffer.
    unsafe { vim_iswordc_buf(c, buf) }
}

/// Whether `c` may appear in a file name ('isfname'). Everything past
/// U+00FF is allowed.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_isfilec(c: c_int) -> bool {
    c >= 0x100 || (c > 0 && chartab(c as uint8_t) & CT_FNAME_CHAR != 0)
}

/// Like [`vim_isfilec`], but also accepts the separators that may appear in
/// a file name given on a command line.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_is_fname_char(c: c_int) -> bool {
    (unsafe { vim_isfilec(c) })
        || c == ',' as c_int
        || c == ' ' as c_int
        || c == '@' as c_int
        || c == ':' as c_int
}

/// Like [`vim_isfilec`], but also accepts the wildcards a file name pattern
/// may contain.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_isfilec_or_wc(c: c_int) -> bool {
    let buf: [c_char; 2] = [c as c_char, NUL as c_char];
    // SAFETY: forwarded, and `buf` is a NUL-terminated string of one byte.
    unsafe { vim_isfilec(c) || c == ']' as c_int || path_has_wildcard(buf.as_ptr()) }
}

/// Whether `c` displays as itself ('isprint').
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_isprintc(c: c_int) -> bool {
    if c >= 0x100 {
        return utf_printable(c);
    }
    c > 0 && chartab(c as uint8_t) & CT_PRINT_CHAR != 0
}

/// The first byte of `p` that is not a space or tab.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skipwhite(p: *const c_char) -> *mut c_char {
    unsafe { Bytes::new(p) }.skip_while(is_white).raw()
}

/// [`skipwhite`], bounded to `len` bytes.
///
/// # Safety
/// `p` must hold `len` readable bytes.
pub unsafe fn skipwhite_len(p: *const c_char, len: size_t) -> *mut c_char {
    // SAFETY: the caller guarantees `len` readable bytes at `p`.
    let bytes = unsafe { core::slice::from_raw_parts(p as *const uint8_t, len) };
    let white = bytes.iter().take_while(|&&byte| is_white(byte)).count();
    p.wrapping_add(white) as *mut c_char
}

/// The indent of the cursor's line, in bytes.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn getwhitecols_curline() -> intptr_t {
    unsafe { getwhitecols(get_cursor_line_ptr()) }
}

/// How many leading bytes of `p` are white space.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn getwhitecols(p: *const c_char) -> intptr_t {
    (unsafe { skipwhite(p) }.addr() - p.addr()) as intptr_t
}

/// The first byte of `q` that is not a decimal digit.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skipdigits(q: *const c_char) -> *mut c_char {
    unsafe { Bytes::new(q) }.skip_while(is_digit).raw()
}

/// The first byte of `q` that is not a binary digit.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skipbin(q: *const c_char) -> *const c_char {
    unsafe { Bytes::new(q) }.skip_while(is_bdigit).raw()
}

/// The first byte of `q` that is not a hexadecimal digit.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiphex(q: *mut c_char) -> *mut c_char {
    unsafe { Bytes::new(q) }.skip_while(is_xdigit).raw()
}

/// The first decimal digit in `q`, or its NUL.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiptodigit(q: *mut c_char) -> *mut c_char {
    unsafe { Bytes::new(q) }
        .skip_while(|byte| !is_digit(byte))
        .raw()
}

/// The first binary digit in `q`, or its NUL.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiptobin(q: *const c_char) -> *const c_char {
    unsafe { Bytes::new(q) }
        .skip_while(|byte| !is_bdigit(byte))
        .raw()
}

/// The first hexadecimal digit in `q`, or its NUL.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiptohex(q: *mut c_char) -> *mut c_char {
    unsafe { Bytes::new(q) }
        .skip_while(|byte| !is_xdigit(byte))
        .raw()
}

/// The first white space byte in `p`, or its NUL.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skiptowhite(p: *const c_char) -> *mut c_char {
    unsafe { Bytes::new(p) }
        .skip_while(|byte| !is_white(byte))
        .raw()
}

/// [`skiptowhite`], but a backslash or CTRL-V hides the byte after it.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skiptowhite_esc(p: *const c_char) -> *mut c_char {
    let mut cursor = unsafe { Bytes::new(p) };
    loop {
        let (byte, next) = cursor.pair();
        if byte == 0 || is_white(byte) {
            return cursor.raw();
        }
        let escapes = (byte == b'\\' || byte as c_int == Ctrl_V) && next != 0;
        cursor.advance(1 + usize::from(escapes));
    }
}

/// The next newline in `p`, or its NUL.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skip_to_newline(p: *const c_char) -> *mut c_char {
    unsafe { xstrchrnul(p, NL as c_char) }
}

/// Read a decimal number at `*pp`, advancing it past the digits. Answers
/// false when the value did not fit, in which case `*nr` holds the clamped
/// `strtoimax` result.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn try_getdigits(pp: *mut *mut c_char, nr: *mut intmax_t) -> bool {
    // SAFETY: `*pp` is a NUL-terminated string, `strtoimax` advances it past
    // whatever it consumed, and `errno` is the C library's own thread-local.
    let number = unsafe {
        *__errno_location() = 0;
        strtoimax(*pp, pp, 10)
    };
    // SAFETY: the caller's out-argument is writable.
    unsafe { *nr = number };
    // SAFETY: as above.
    let out_of_range = unsafe { *__errno_location() } == ERANGE;
    !(out_of_range && (number == intmax_t::MIN || number == intmax_t::MAX))
}

/// [`try_getdigits`], answering `def` when the value did not fit.
///
/// `strict` says the caller has already established that there *are* digits
/// here, so a value it cannot represent is a bad number rather than a parse
/// failure, and `def` would be misleading. Every one of those callers is
/// reading text a user typed -- an option value, a `:sign` id, a `:breakadd`
/// line number -- so an unrepresentable value **saturates** rather than
/// failing: `strtoimax` has already clamped it to `INTMAX_MIN`/`INTMAX_MAX`
/// and that is what comes back. Upstream `abort()`s here instead, which
/// `:set breakindentopt=min:99999999999999999999999` reaches from a modeline.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits(pp: *mut *mut c_char, strict: bool, def: intmax_t) -> intmax_t {
    let mut number: intmax_t = 0;
    // SAFETY: forwarded to the caller's contract; `number` is a local.
    let ok = unsafe { try_getdigits(pp, &raw mut number) };
    if ok || strict { number } else { def }
}

/// [`getdigits`] narrowed to an `int`.
///
/// A `strict` value outside the range saturates -- see [`getdigits`] for why
/// it is not an abort.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int {
    let number = unsafe { getdigits(pp, strict, def as intmax_t) };
    if strict {
        return number.clamp(c_int::MIN as intmax_t, c_int::MAX as intmax_t) as c_int;
    }
    c_int::try_from(number).unwrap_or(def)
}

/// [`getdigits`] narrowed to an `int32_t`, with [`getdigits_int`]'s shape.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_int32(pp: *mut *mut c_char, strict: bool, def: int32_t) -> int32_t {
    let number = unsafe { getdigits(pp, strict, def as intmax_t) };
    if strict {
        return number.clamp(int32_t::MIN as intmax_t, int32_t::MAX as intmax_t) as int32_t;
    }
    int32_t::try_from(number).unwrap_or(def)
}

/// [`getdigits`] narrowed to a `long`. Note that unlike the `int` forms this
/// does not range-check, because on this platform it cannot fail.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_long(pp: *mut *mut c_char, strict: bool, def: c_long) -> c_long {
    unsafe { getdigits(pp, strict, def as intmax_t) as c_long }
}

/// Whether `lbuf` holds nothing but white space.
///
/// # Safety
/// `lbuf` must be a NUL-terminated string.
pub unsafe fn vim_isblankline(lbuf: *mut c_char) -> bool {
    // SAFETY: forwarded to the caller's contract; `skipwhite` stays inside.
    let byte = unsafe { Bytes::new(skipwhite(lbuf)) }.byte();
    byte == 0 || byte == b'\r' || byte == b'\n'
}

/// A lazy cursor over the digits of a number, bounded either by `maxlen`
/// bytes or by the first byte that is not a digit.
///
/// As with [`Bytes`], construction is the unsafe step: the cursor is only
/// ever advanced over bytes already read as digits, and every look-ahead is
/// guarded by [`Scan::within`], so its reads stay inside the string.
struct Scan {
    start: *const c_char,
    ptr: *const c_char,
    maxlen: c_int,
}

impl Scan {
    /// # Safety
    /// `start` must be NUL-terminated, or hold `maxlen` readable bytes.
    #[inline(always)]
    unsafe fn new(start: *const c_char, ptr: *const c_char, maxlen: c_int) -> Self {
        Scan { start, ptr, maxlen }
    }

    #[inline(always)]
    fn consumed(&self) -> c_int {
        (self.ptr.addr() - self.start.addr()) as c_int
    }

    /// Whether `offset` more bytes are still within `maxlen`. A `maxlen` of
    /// zero means "to the NUL".
    #[inline(always)]
    fn within(&self, offset: c_int) -> bool {
        self.maxlen == 0 || self.consumed() + offset < self.maxlen
    }

    /// The byte `offset` further on.
    #[inline(always)]
    fn at(&self, offset: c_int) -> uint8_t {
        // SAFETY: the cursor has not passed the terminator, and a bounded
        // scan only looks `offset` ahead after `within(offset)`.
        unsafe { *self.ptr.offset(offset as isize) as uint8_t }
    }

    #[inline(always)]
    fn advance(&mut self, n: isize) {
        self.ptr = self.ptr.wrapping_offset(n);
    }
}

/// Parse the number at `start`, reporting whichever of the out-arguments is
/// not null: the prefix letter (`prep`), the number of bytes consumed
/// (`len`), the signed and unsigned values (`nptr`, `unptr`) and whether the
/// value was clamped (`overflow`).
///
/// `what` is a set of `STR2NR_*` flags. `maxlen` bounds the scan (zero means
/// the whole string), and `strict` makes a trailing letter or digit reject
/// the parse outright, leaving `len` at zero.
///
/// The scan itself lives in [`str2nr::scan`], which is safe code: everything
/// unchecked about this is the cursor it walks and the out-arguments.
///
/// # Safety
/// `start` must be NUL-terminated, or hold `maxlen` readable bytes.
#[expect(
    clippy::too_many_arguments,
    reason = "the C signature, kept for its callers"
)]
pub unsafe fn vim_str2nr(
    start: *const c_char,
    prep: *mut c_int,
    len: *mut c_int,
    what: c_int,
    nptr: *mut varnumber_T,
    unptr: *mut uvarnumber_T,
    maxlen: c_int,
    strict: bool,
    overflow: *mut bool,
) {
    let negative = unsafe { *start } as c_int == '-' as c_int;
    // SAFETY: a leading `-` is never the last byte of the caller's string.
    let mut scan = unsafe { Scan::new(start, if negative { start.add(1) } else { start }, maxlen) };
    let Some(parsed) = str2nr::scan(&mut scan, what) else {
        // `what` forces a base its remaining flags do not name.
        // SAFETY: `abort` never returns.
        unsafe { abort() };
    };

    // A strict parse only accepts a number that ends the string (or fills
    // `maxlen`); anything alphanumeric after it means this was not a number.
    let rejected = strict && scan.consumed() != maxlen && str2nr::strict_reject(scan.at(0));
    let (value, clamped) = str2nr::signed(parsed.magnitude, negative);
    // The C reports a clamp only from the branch that writes `nptr`, so a
    // rejected or unasked-for signed value never raises the flag; overflow
    // seen while accumulating is reported either way.
    let overflowed = parsed.overflowed || (clamped && !rejected && !nptr.is_null());

    // SAFETY: each out-argument is null or points at a writable value.
    unsafe {
        if !len.is_null() {
            *len = if rejected { 0 } else { scan.consumed() };
        }
        if overflowed && !overflow.is_null() {
            *overflow = true;
        }
        if rejected {
            return;
        }
        if !prep.is_null() {
            *prep = parsed.pre;
        }
        if !nptr.is_null() {
            *nptr = value;
        }
        if !unptr.is_null() {
            *unptr = parsed.magnitude;
        }
    }
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

/// The byte the two hexadecimal digits at `p` spell, or -1.
///
/// # Safety
/// `p` must hold two readable bytes.
pub unsafe fn hexhex2nr(p: *const c_char) -> c_int {
    // SAFETY: the caller guarantees both bytes.
    let [high, low] = unsafe { *p.cast::<[uint8_t; 2]>() };
    if !is_xdigit(high) || !is_xdigit(low) {
        return -1;
    }
    (hex2nr(high as c_int) << 4) + hex2nr(low as c_int)
}

/// Whether the backslash at `str` escapes the byte after it, i.e. whether
/// [`backslash_halve`] would remove it.
///
/// # Safety
/// `str` must be a NUL-terminated string.
pub unsafe fn rem_backslash(str: *const c_char) -> bool {
    let (byte, next) = unsafe { Bytes::new(str) }.pair();
    byte == b'\\' && next != 0
}

/// Remove the escaping backslashes from `p`, in place.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn backslash_halve(p: *mut c_char) {
    // Nothing before the first escaping backslash needs moving.
    let mut src = unsafe { Bytes::new(p) };
    while {
        let (byte, next) = src.pair();
        byte != 0 && !(byte == b'\\' && next != 0)
    } {
        src.advance(1);
    }
    if src.byte() == 0 {
        return;
    }
    // SAFETY: `src` is still inside the caller's writable string, and the
    // destination never runs ahead of it.
    unsafe { copy_unescaped(src, src.raw()) };
}

/// [`backslash_halve`] into a fresh string the caller owns.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn backslash_halve_save(p: *const c_char) -> *mut c_char {
    let res = unsafe { xmalloc(strlen(p) + 1) } as *mut c_char;
    // SAFETY: the allocation is at least as long as the string, and halving
    // only ever shortens it.
    unsafe { copy_unescaped(Bytes::new(p), res) };
    res
}

/// Copy the string at `src` to `dst`, dropping each backslash that escapes
/// the byte after it, and terminate it.
///
/// # Safety
/// `dst` must have room for everything from `src` to its NUL, inclusive.
unsafe fn copy_unescaped(mut src: Bytes, dst: *mut c_char) {
    let mut dst = dst;
    loop {
        let (byte, next) = src.pair();
        if byte == 0 {
            break;
        }
        if byte == b'\\' && next != 0 {
            src.advance(1);
        }
        // SAFETY: the destination has room for every byte the source has.
        unsafe { *dst = src.byte() as c_char };
        dst = dst.wrapping_add(1);
        src.advance(1);
    }
    // SAFETY: as above, for the terminator.
    unsafe { *dst = NUL as c_char };
}

/// Whether a line may be broken before `c`, per the 'breakat' option.
pub fn vim_isbreak(c: c_int) -> bool {
    // SAFETY: the cell holds a live 256-byte array indexed by a byte. Read
    // raw rather than through `with`, whose debug-build borrow tracking is
    // far more expensive than the load: 'linebreak' asks this once per
    // character of every drawn line.
    unsafe { (*breakat_flags.ptr())[c as uint8_t as usize] != 0 }
}
