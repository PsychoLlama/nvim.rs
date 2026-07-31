//! Character classification and display translation.
//!
//! Three tables decide everything here:
//!
//! - `g_chartab`, one byte per 8-bit character, holding its display width in
//!   the low three bits plus the 'isident' / 'isprint' / 'isfname' flags.
//! - each buffer's `b_chartab`, a 256-bit set built from 'iskeyword'.
//! - `utf8len_tab` and the `utf_*` classifiers in `mbyte.rs`, for anything
//!   past U+00FF.

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::ptr;

use crate::src::nvim::cursor::get_cursor_line_ptr;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{breakat_flags, curbuf, dy_flags, p_isf, p_isi, p_isp};
use crate::src::nvim::mbyte::{
    mb_islower, mb_isupper, mb_ptr2char_adv, utf_class_tab, utf_printable, utf_ptr2char,
    utf8len_tab,
};
use crate::src::nvim::memory::{xmalloc, xstrchrnul};
use crate::src::nvim::option::skip_to_option_part;
use crate::src::nvim::options::kOptDyFlagUhex;
use crate::src::nvim::os::libc::{__errno_location, abort, memset, strlen, strtoimax};
use crate::src::nvim::path::path_has_wildcard;
use crate::src::nvim::types::{
    buf_T, int32_t, intmax_t, intptr_t, size_t, uint8_t, uint64_t, uvarnumber_T, varnumber_T,
};

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

use str2nr::Radix;

/// Bases `vim_str2nr` may recognise, plus the two behaviour flags. `FORCE`
/// says "the string has no prefix, parse it in the base named by the rest";
/// `QUOTE` allows `'` as a digit separator.
pub const STR2NR_DEC: c_int = 0;
pub const STR2NR_BIN: c_int = 1;
pub const STR2NR_OCT: c_int = 2;
pub const STR2NR_HEX: c_int = 4;
pub const STR2NR_OOCT: c_int = 8;
pub const STR2NR_ALL: c_int = STR2NR_BIN | STR2NR_OCT | STR2NR_HEX | STR2NR_OOCT;
// Spelled as a literal, not as `STR2NR_ALL & !STR2NR_OOCT`: ffigen only
// evaluates literal expressions, and nine other modules declare this as 13.
pub const STR2NR_NO_OCT: c_int = 13;
pub const STR2NR_FORCE: c_int = 128;
pub const STR2NR_QUOTE: c_int = 16;

/// Bits of a `g_chartab` entry.
const CT_CELL_MASK: c_int = 0x7;
const CT_PRINT_CHAR: c_int = 0x10;
const CT_ID_CHAR: c_int = 0x20;
const CT_FNAME_CHAR: c_int = 0x40;

const NUL: c_int = 0;
const TAB: c_int = 9;
const NL: c_int = 10;
const CAR: c_int = 13;
const Ctrl_V: c_int = 22;
const KS_ZERO: c_int = 255;
const KS_SPECIAL: c_int = 254;
const EOL_MAC: c_int = 2;
const ERANGE: c_int = 34;
const MAXCOL: c_int = 2147483647;
const OK: c_int = 1;
const FAIL: c_int = 0;

static chartab_initialized: GlobalCell<bool> = GlobalCell::new(false);
static g_chartab: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);

/// The cell width an unprintable byte is displayed with: four for `<xx>`,
/// two for `^X`.
fn unprintable_width() -> c_int {
    if dy_flags.get() & kOptDyFlagUhex != 0 {
        4
    } else {
        2
    }
}

/// Rebuild the global table and the current buffer's keyword set.
///
/// # Safety
/// The current buffer must be valid.
pub unsafe fn init_chartab() -> bool {
    buf_init_chartab(curbuf.get(), true)
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
        let table = g_chartab.ptr();
        // Control characters display as `^X` or `<xx>`; printable ASCII is
        // one cell wide; the Latin-1 upper half is printable and valid in a
        // file name, but 0x7f-0x9f are not.
        for c in 0..b' ' as usize {
            (*table)[c] = unprintable_width() as uint8_t;
        }
        for c in b' ' as usize..=b'~' as usize {
            (*table)[c] = (1 + CT_PRINT_CHAR) as uint8_t;
        }
        for c in b'~' as usize + 1..256 {
            (*table)[c] = if c >= 0xa0 {
                (CT_PRINT_CHAR | CT_FNAME_CHAR) as uint8_t + 1
            } else {
                unprintable_width() as uint8_t
            };
        }
    }

    memset(
        &raw mut (*buf).b_chartab as *mut c_void,
        0,
        ::core::mem::size_of::<[uint64_t; 4]>(),
    );
    if (*buf).b_p_lisp != 0 {
        // In Lisp, `-` belongs to a word even when 'iskeyword' omits it.
        set_buf_chartab(buf, b'-' as c_int);
    }

    // 0..2 are the global options; 3 is the buffer's own 'iskeyword'.
    let first = if global { 0 } else { 3 };
    for i in first..=3 {
        let option = match i {
            0 => p_isi.get() as *const c_char,
            1 => p_isp.get() as *const c_char,
            2 => p_isf.get() as *const c_char,
            _ => (*buf).b_p_isk as *const c_char,
        };
        if parse_isopt(option, buf, false) == FAIL {
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
    parse_isopt(var, ptr::null_mut(), true)
}

/// Set `c`'s bit in `buf`'s keyword set.
unsafe fn set_buf_chartab(buf: *mut buf_T, c: c_int) {
    let word = (c as c_uint >> 6) as usize;
    (*buf).b_chartab[word] |= 1u64 << (c & 0x3f);
}

/// Clear `c`'s bit in `buf`'s keyword set.
unsafe fn clear_buf_chartab(buf: *mut buf_T, c: c_int) {
    let word = (c as c_uint >> 6) as usize;
    (*buf).b_chartab[word] &= !(1u64 << (c & 0x3f));
}

/// Walk one 'isident'/'isprint'/'isfname'/'iskeyword' value, applying each
/// comma-separated range to the table the option owns.
///
/// Which table that is comes from the *identity* of the pointer — `var` is
/// compared against `p_isi`/`p_isp`/`p_isf` — so this cannot be called with
/// a copy of an option's value.
///
/// With `only_check` no table is touched and only the syntax is validated.
unsafe fn parse_isopt(var: *const c_char, buf: *mut buf_T, only_check: bool) -> c_int {
    let mut p = var;
    while *p != 0 {
        let mut tilde = false;
        let mut do_isalpha = false;
        // A leading `^` removes the range instead of adding it.
        if *p as c_int == '^' as c_int && *p.offset(1) as c_int != NUL {
            tilde = true;
            p = p.offset(1);
        }
        let mut c = if (*p as c_int).is_ascii_digit_c() {
            getdigits_int(&raw mut p as *mut *mut c_char, true, 0)
        } else {
            mb_ptr2char_adv(&raw mut p)
        };
        let mut c2: c_int = -1;
        if *p as c_int == '-' as c_int && *p.offset(1) as c_int != NUL {
            p = p.offset(1);
            c2 = if (*p as c_int).is_ascii_digit_c() {
                getdigits_int(&raw mut p as *mut *mut c_char, true, 0)
            } else {
                mb_ptr2char_adv(&raw mut p)
            };
        }
        if c <= 0
            || c >= 256
            || (c2 < c && c2 != -1)
            || c2 >= 256
            || !(*p as c_int == NUL || *p as c_int == ',' as c_int)
        {
            return FAIL;
        }

        let trail_comma = *p as c_int == ',' as c_int;
        p = skip_to_option_part(p);
        // A trailing comma with nothing after it is malformed.
        if trail_comma && *p as c_int == NUL {
            return FAIL;
        }
        if only_check {
            continue;
        }

        if c2 == -1 {
            if c == '@' as c_int {
                // `@` stands for "every alphabetic character in the locale".
                do_isalpha = true;
                c = 1;
                c2 = 255;
            } else {
                c2 = c;
            }
        }

        while c <= c2 {
            if !do_isalpha || mb_islower(c) || mb_isupper(c) {
                let table = g_chartab.ptr();
                if var == p_isi.get() as *const c_char {
                    if tilde {
                        (*table)[c as usize] &= !(CT_ID_CHAR as uint8_t);
                    } else {
                        (*table)[c as usize] |= CT_ID_CHAR as uint8_t;
                    }
                } else if var == p_isp.get() as *const c_char {
                    // 'isprint' cannot demote printable ASCII.
                    if c < ' ' as c_int || c > '~' as c_int {
                        let width = if tilde { unprintable_width() } else { 1 };
                        (*table)[c as usize] =
                            (((*table)[c as usize] as c_int & !CT_CELL_MASK) + width) as uint8_t;
                        if tilde {
                            (*table)[c as usize] &= !(CT_PRINT_CHAR as uint8_t);
                        } else {
                            (*table)[c as usize] |= CT_PRINT_CHAR as uint8_t;
                        }
                    }
                } else if var == p_isf.get() as *const c_char {
                    if tilde {
                        (*table)[c as usize] &= !(CT_FNAME_CHAR as uint8_t);
                    } else {
                        (*table)[c as usize] |= CT_FNAME_CHAR as uint8_t;
                    }
                } else if tilde {
                    clear_buf_chartab(buf, c);
                } else {
                    set_buf_chartab(buf, c);
                }
            }
            c += 1;
        }
    }
    OK
}

/// `ascii.h`'s classifiers, as methods so the call sites read as tests
/// rather than as pairs of range comparisons.
trait AsciiClass {
    fn is_white_c(self) -> bool;
    fn is_ascii_digit_c(self) -> bool;
    fn is_ascii_bdigit_c(self) -> bool;
    fn is_ascii_odigit_c(self) -> bool;
    fn is_ascii_xdigit_c(self) -> bool;
}

impl AsciiClass for c_int {
    fn is_white_c(self) -> bool {
        self == ' ' as c_int || self == TAB
    }
    fn is_ascii_digit_c(self) -> bool {
        (b'0' as c_int..=b'9' as c_int).contains(&self)
    }
    fn is_ascii_bdigit_c(self) -> bool {
        self == b'0' as c_int || self == b'1' as c_int
    }
    fn is_ascii_odigit_c(self) -> bool {
        (b'0' as c_int..=b'7' as c_int).contains(&self)
    }
    fn is_ascii_xdigit_c(self) -> bool {
        self.is_ascii_digit_c()
            || (b'a' as c_int..=b'f' as c_int).contains(&self)
            || (b'A' as c_int..=b'F' as c_int).contains(&self)
    }
}

/// Whether `c` may appear in an identifier ('isident').
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_isIDc(c: c_int) -> bool {
    c > 0 && c < 0x100 && (*g_chartab.ptr())[c as usize] as c_int & CT_ID_CHAR != 0
}

/// Whether `c` belongs to a word in the current buffer ('iskeyword').
///
/// # Safety
/// The current buffer must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_iswordc(c: c_int) -> bool {
    vim_iswordc_buf(c, curbuf.get())
}

/// Whether `c` belongs to a word according to the 256-bit set `chartab`.
/// Characters past U+00FF are decided by their Unicode class instead.
///
/// # Safety
/// `chartab` must point at four `uint64_t`s.
pub unsafe fn vim_iswordc_tab(c: c_int, chartab: *const uint64_t) -> bool {
    if c >= 0x100 {
        return utf_class_tab(c, chartab) >= 2;
    }
    c > 0 && *chartab.offset((c as c_uint >> 6) as isize) & (1u64 << (c & 0x3f)) != 0
}

/// Whether `c` belongs to a word in `buf`.
///
/// # Safety
/// `buf` must be a valid buffer.
pub unsafe fn vim_iswordc_buf(c: c_int, buf: *mut buf_T) -> bool {
    vim_iswordc_tab(c, &raw mut (*buf).b_chartab as *mut uint64_t)
}

/// Whether the character at `p` belongs to a word in the current buffer.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_iswordp(p: *const c_char) -> bool {
    vim_iswordp_buf(p, curbuf.get())
}

/// Whether the character at `p` belongs to a word in `buf`.
///
/// # Safety
/// `p` must point into a NUL-terminated string and `buf` be a valid buffer.
pub unsafe fn vim_iswordp_buf(p: *const c_char, buf: *mut buf_T) -> bool {
    let mut c = *p as uint8_t as c_int;
    if (*utf8len_tab.ptr())[c as usize] as c_int > 1 {
        c = utf_ptr2char(p);
    }
    vim_iswordc_buf(c, buf)
}

/// Whether `c` may appear in a file name ('isfname'). Everything past
/// U+00FF is allowed.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_isfilec(c: c_int) -> bool {
    c >= 0x100 || (c > 0 && (*g_chartab.ptr())[c as usize] as c_int & CT_FNAME_CHAR != 0)
}

/// Like [`vim_isfilec`], but also accepts the separators that may appear in
/// a file name given on a command line.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_is_fname_char(c: c_int) -> bool {
    vim_isfilec(c)
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
    vim_isfilec(c) || c == ']' as c_int || path_has_wildcard(buf.as_ptr() as *mut c_char)
}

/// Whether `c` displays as itself ('isprint').
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn vim_isprintc(c: c_int) -> bool {
    if c >= 0x100 {
        return utf_printable(c);
    }
    c > 0 && (*g_chartab.ptr())[c as usize] as c_int & CT_PRINT_CHAR != 0
}

/// The first byte of `p` that is not a space or tab.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skipwhite(p: *const c_char) -> *mut c_char {
    let mut p = p;
    while (*p as c_int).is_white_c() {
        p = p.offset(1);
    }
    p as *mut c_char
}

/// [`skipwhite`], bounded to `len` bytes.
///
/// # Safety
/// `p` must hold `len` readable bytes.
pub unsafe fn skipwhite_len(p: *const c_char, len: size_t) -> *mut c_char {
    let mut p = p;
    let mut len = len;
    while len > 0 && (*p as c_int).is_white_c() {
        p = p.offset(1);
        len -= 1;
    }
    p as *mut c_char
}

/// The indent of the cursor's line, in bytes.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn getwhitecols_curline() -> intptr_t {
    getwhitecols(get_cursor_line_ptr())
}

/// How many leading bytes of `p` are white space.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn getwhitecols(p: *const c_char) -> intptr_t {
    (skipwhite(p).addr() - p.addr()) as intptr_t
}

/// The first byte of `q` that is not a decimal digit.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skipdigits(q: *const c_char) -> *mut c_char {
    let mut p = q;
    while (*p as c_int).is_ascii_digit_c() {
        p = p.offset(1);
    }
    p as *mut c_char
}

/// The first byte of `q` that is not a binary digit.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skipbin(q: *const c_char) -> *const c_char {
    let mut p = q;
    while (*p as c_int).is_ascii_bdigit_c() {
        p = p.offset(1);
    }
    p
}

/// The first byte of `q` that is not a hexadecimal digit.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiphex(q: *mut c_char) -> *mut c_char {
    let mut p = q;
    while (*p as c_int).is_ascii_xdigit_c() {
        p = p.offset(1);
    }
    p
}

/// The first decimal digit in `q`, or its NUL.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiptodigit(q: *mut c_char) -> *mut c_char {
    let mut p = q;
    while *p as c_int != NUL && !(*p as c_int).is_ascii_digit_c() {
        p = p.offset(1);
    }
    p
}

/// The first binary digit in `q`, or its NUL.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiptobin(q: *const c_char) -> *const c_char {
    let mut p = q;
    while *p as c_int != NUL && !(*p as c_int).is_ascii_bdigit_c() {
        p = p.offset(1);
    }
    p
}

/// The first hexadecimal digit in `q`, or its NUL.
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiptohex(q: *mut c_char) -> *mut c_char {
    let mut p = q;
    while *p as c_int != NUL && !(*p as c_int).is_ascii_xdigit_c() {
        p = p.offset(1);
    }
    p
}

/// The first white space byte in `p`, or its NUL.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skiptowhite(p: *const c_char) -> *mut c_char {
    let mut p = p;
    while *p as c_int != NUL && !(*p as c_int).is_white_c() {
        p = p.offset(1);
    }
    p as *mut c_char
}

/// [`skiptowhite`], but a backslash or CTRL-V hides the byte after it.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skiptowhite_esc(p: *const c_char) -> *mut c_char {
    let mut p = p;
    while *p as c_int != NUL && !(*p as c_int).is_white_c() {
        if (*p as c_int == '\\' as c_int || *p as c_int == Ctrl_V) && *p.offset(1) as c_int != NUL {
            p = p.offset(1);
        }
        p = p.offset(1);
    }
    p as *mut c_char
}

/// The next newline in `p`, or its NUL.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skip_to_newline(p: *const c_char) -> *mut c_char {
    xstrchrnul(p, NL as c_char)
}

/// Read a decimal number at `*pp`, advancing it past the digits. Answers
/// false when the value did not fit, in which case `*nr` holds the clamped
/// `strtoimax` result.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn try_getdigits(pp: *mut *mut c_char, nr: *mut intmax_t) -> bool {
    *__errno_location() = 0;
    *nr = strtoimax(*pp, pp, 10);
    !(*__errno_location() == ERANGE && (*nr == intmax_t::MIN || *nr == intmax_t::MAX))
}

/// [`try_getdigits`], answering `def` when the value did not fit. With
/// `strict` an unrepresentable value aborts instead.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits(pp: *mut *mut c_char, strict: bool, def: intmax_t) -> intmax_t {
    let mut number: intmax_t = 0;
    let ok = try_getdigits(pp, &raw mut number);
    if strict && !ok {
        abort();
    }
    if ok { number } else { def }
}

/// [`getdigits`] narrowed to an `int`.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int {
    let number = getdigits(pp, strict, def as intmax_t);
    if strict {
        assert!(
            number >= c_int::MIN as intmax_t && number <= c_int::MAX as intmax_t,
            "number >= INT_MIN && number <= INT_MAX"
        );
    } else if !(number >= c_int::MIN as intmax_t && number <= c_int::MAX as intmax_t) {
        return def;
    }
    number as c_int
}

/// [`getdigits`] narrowed to a `long`. Note that unlike the `int` forms this
/// does not range-check, because on this platform it cannot fail.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_long(pp: *mut *mut c_char, strict: bool, def: c_long) -> c_long {
    getdigits(pp, strict, def as intmax_t) as c_long
}

/// [`getdigits`] narrowed to an `int32_t`.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_int32(pp: *mut *mut c_char, strict: bool, def: int32_t) -> int32_t {
    let number = getdigits(pp, strict, def as intmax_t);
    if strict {
        assert!(
            number >= int32_t::MIN as intmax_t && number <= int32_t::MAX as intmax_t,
            "number >= INT32_MIN && number <= INT32_MAX"
        );
    } else if !(number >= int32_t::MIN as intmax_t && number <= int32_t::MAX as intmax_t) {
        return def;
    }
    number as int32_t
}

/// Whether `lbuf` holds nothing but white space.
///
/// # Safety
/// `lbuf` must be a NUL-terminated string.
pub unsafe fn vim_isblankline(lbuf: *mut c_char) -> bool {
    let p = skipwhite(lbuf);
    *p as c_int == NUL || *p as c_int == CAR as c_char as c_int || *p as c_int == NL
}

/// A lazy cursor over the digits of a number, bounded either by `maxlen`
/// bytes or by the first byte that is not a digit.
struct Scan {
    start: *const c_char,
    ptr: *const c_char,
    maxlen: c_int,
}

impl Scan {
    fn consumed(&self) -> c_int {
        (self.ptr.addr() - self.start.addr()) as c_int
    }

    /// Whether `offset` more bytes are still within `maxlen`. A `maxlen` of
    /// zero means "to the NUL".
    fn within(&self, offset: c_int) -> bool {
        self.maxlen == 0 || self.consumed() + offset < self.maxlen
    }

    /// # Safety
    /// The byte at `offset` must be within the string.
    unsafe fn at(&self, offset: c_int) -> u8 {
        *self.ptr.offset(offset as isize) as u8
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
    if !len.is_null() {
        *len = 0;
    }
    let negative = *start as c_int == '-' as c_int;
    let mut scan = Scan {
        start,
        ptr: if negative { start.offset(1) } else { start },
        maxlen,
    };

    // Decide the radix, and step over the prefix if there is one. `pre` is
    // the prefix letter the caller is told about: `0`, `b`, `B`, `o`, `O`,
    // `x` or `X`, and zero for a plain decimal number or a forced base.
    let mut pre: c_int = 0;
    let radix;
    if what & STR2NR_FORCE != 0 {
        radix = str2nr::forced_radix(what & !(STR2NR_FORCE | STR2NR_QUOTE)).unwrap_or_else(|| {
            if what & !(STR2NR_FORCE | STR2NR_QUOTE) == STR2NR_DEC {
                Radix::Decimal
            } else {
                abort();
            }
        });
        // A forced base still tolerates the matching prefix.
        if let Some(letter) = match radix {
            Radix::Hexadecimal => Some((b'x', b'X')),
            Radix::Binary => Some((b'b', b'B')),
            Radix::Octal => Some((b'o', b'O')),
            Radix::Decimal => None,
        } && scan.within(2)
            && scan.at(0) == b'0'
            && (scan.at(1) == letter.0 || scan.at(1) == letter.1)
            && radix.digit(scan.at(2)).is_some()
        {
            scan.ptr = scan.ptr.offset(2);
        }
    } else if what & (STR2NR_HEX | STR2NR_OCT | STR2NR_OOCT | STR2NR_BIN) != 0
        && scan.within(1)
        && scan.at(0) == b'0'
        && scan.at(1) != b'8'
        && scan.at(1) != b'9'
    {
        pre = scan.at(1) as c_int;
        let prefixed = [
            (STR2NR_HEX, Radix::Hexadecimal, b'x', b'X'),
            (STR2NR_BIN, Radix::Binary, b'b', b'B'),
            (STR2NR_OOCT, Radix::Octal, b'o', b'O'),
        ]
        .into_iter()
        .find(|&(flag, base, lower, upper)| {
            what & flag != 0
                && scan.within(2)
                && (pre == upper as c_int || pre == lower as c_int)
                && base.digit(scan.at(2)).is_some()
        });
        if let Some((_, base, _, _)) = prefixed {
            scan.ptr = scan.ptr.offset(2);
            radix = base;
        } else {
            pre = 0;
            // A leading zero means octal only if every digit that follows is
            // one; `0548` is decimal.
            let mut octal = what & STR2NR_OCT != 0 && (scan.at(1) as c_int).is_ascii_odigit_c();
            if octal {
                let mut i = 2;
                while scan.within(i) && (scan.at(i) as c_int).is_ascii_digit_c() {
                    if scan.at(i) > b'7' {
                        octal = false;
                        break;
                    }
                    i += 1;
                }
            }
            if octal {
                pre = '0' as c_int;
                radix = Radix::Octal;
            } else {
                radix = Radix::Decimal;
            }
        }
    } else {
        radix = Radix::Decimal;
    }

    // Accumulate the digits. A quote is only a separator between digits, so
    // it never ends the number by itself.
    let after_prefix = scan.ptr;
    let mut un: uvarnumber_T = 0;
    while scan.within(0) {
        if what & STR2NR_QUOTE != 0 && scan.ptr > after_prefix && scan.at(0) == b'\'' {
            scan.ptr = scan.ptr.offset(1);
            // The C tests the *decimal* digit classes here for every base
            // except binary, which tests only `0`/`1`. Preserved.
            let separates = match radix {
                Radix::Binary => scan.at(0) == b'0' || scan.at(0) == b'1',
                Radix::Octal => (scan.at(0) as c_int).is_ascii_odigit_c(),
                Radix::Decimal => (scan.at(0) as c_int).is_ascii_digit_c(),
                Radix::Hexadecimal => (scan.at(0) as c_int).is_ascii_xdigit_c(),
            };
            if scan.within(0) && separates {
                continue;
            }
            scan.ptr = scan.ptr.offset(-1);
        }
        let Some(digit) = radix.digit(scan.at(0)) else {
            break;
        };
        let (next, saturated) = str2nr::accumulate(un, digit, radix);
        un = next;
        if saturated && !overflow.is_null() {
            *overflow = true;
        }
        scan.ptr = scan.ptr.offset(1);
    }

    // A strict parse only accepts a number that ends the string (or fills
    // `maxlen`); anything alphanumeric after it means this was not a number.
    if strict && scan.consumed() != maxlen && str2nr::strict_reject(scan.at(0)) {
        return;
    }

    if !prep.is_null() {
        *prep = pre;
    }
    if !len.is_null() {
        *len = scan.consumed();
    }
    if !nptr.is_null() {
        let (value, clamped) = str2nr::signed(un, negative);
        *nptr = value;
        if clamped && !overflow.is_null() {
            *overflow = true;
        }
    }
    if !unptr.is_null() {
        *unptr = un;
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
    if !(*p as c_int).is_ascii_xdigit_c() || !(*p.offset(1) as c_int).is_ascii_xdigit_c() {
        return -1;
    }
    (hex2nr(*p as c_int) << 4) + hex2nr(*p.offset(1) as c_int)
}

/// Whether the backslash at `str` escapes the byte after it, i.e. whether
/// [`backslash_halve`] would remove it.
///
/// # Safety
/// `str` must be a NUL-terminated string.
pub unsafe fn rem_backslash(str: *const c_char) -> bool {
    *str as c_int == '\\' as c_int && *str.offset(1) as c_int != NUL
}

/// Remove the escaping backslashes from `p`, in place.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn backslash_halve(p: *mut c_char) {
    let mut p = p as *const c_char;
    while *p as c_int != NUL && !rem_backslash(p) {
        p = p.offset(1);
    }
    if *p as c_int == NUL {
        return;
    }
    let mut dst = p as *mut c_char;
    while *p as c_int != NUL {
        if rem_backslash(p) {
            p = p.offset(1);
        }
        *dst = *p;
        dst = dst.offset(1);
        p = p.offset(1);
    }
    *dst = NUL as c_char;
}

/// [`backslash_halve`] into a fresh string the caller owns.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn backslash_halve_save(p: *const c_char) -> *mut c_char {
    let mut p = p;
    let res = xmalloc(strlen(p) + 1) as *mut c_char;
    let mut dst = res;
    while *p as c_int != NUL {
        if rem_backslash(p) {
            p = p.offset(1);
        }
        *dst = *p;
        dst = dst.offset(1);
        p = p.offset(1);
    }
    *dst = NUL as c_char;
    res
}

/// Whether a line may be broken before `c`, per the 'breakat' option.
pub fn vim_isbreak(c: ::core::ffi::c_int) -> bool {
    breakat_flags.with(|flags| flags[c as uint8_t as usize] != 0)
}
