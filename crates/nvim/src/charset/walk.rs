//! The pointer spellings of the `charset` family.
//!
//! Every reader in `charset` takes a `&[u8]`: [`skip`](super::skip) answers
//! offsets, [`skip::decimal`](super::skip::decimal) reads a number, and the
//! display forms render slices. What is here is what upstream wrote and what
//! the callers whose own cursor is still a raw pointer call: each takes a
//! `*const c_char` into a NUL-terminated string and hands back a pointer into
//! it (or writes through one), and each is a one-line shim over the slice
//! form. They go as their callers convert. The skippers go through
//! [`Bytes`], whose *construction* is the unsafe step, so the walk itself
//! never measures the string.
//!
//! Original: `src/nvim/charset.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
// Raw C strings: the pointer spellings, for callers outside this family.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_int};

use super::display::{foldcase, mirror_ascii, trans_characters_in, transstr_into, transstr_owned};
use super::{
    Str2NrBases, backslash_halved, byte2cells, escapes_next, is_bdigit, is_digit, is_white,
    is_word_at, is_xdigit, skip, str2nr_scan,
};
use crate::cstr;
use crate::memory::ThinCString;
use crate::types::{UVarNumber, VarNumber, int32_t, intmax_t, intptr_t, size_t, ssize_t, uint8_t};
use crate::winlayer::{Buf, Win};
use core::slice;

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
        unsafe { *self.0.cast::<uint8_t>() }
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
            unsafe { *self.0.add(1).cast::<uint8_t>() }
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
        self.0.cast_mut()
    }
}

/// The first byte of `p` that is not a space or tab; [`skip::white`].
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skipwhite(p: *const c_char) -> *mut c_char {
    unsafe { Bytes::new(p) }.skip_while(is_white).raw()
}

/// The indent of the cursor's line, in bytes.
pub(crate) fn getwhitecols_curline() -> intptr_t {
    let win = Win::current();
    let mut lines = win.buffer().lines();
    intptr_t::try_from(skip::white(lines.line(win.w_cursor.lnum))).unwrap_or(0)
}

/// The first byte of `q` that is not a decimal digit; [`skip::digits`].
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skipdigits(q: *const c_char) -> *mut c_char {
    unsafe { Bytes::new(q) }.skip_while(is_digit).raw()
}

/// The first byte of `q` that is not a binary digit; [`skip::bin`].
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skipbin(q: *const c_char) -> *const c_char {
    unsafe { Bytes::new(q) }.skip_while(is_bdigit).raw()
}

/// The first byte of `q` that is not a hexadecimal digit; [`skip::hex`].
///
/// # Safety
/// `q` must be a NUL-terminated string.
pub unsafe fn skiphex(q: *mut c_char) -> *mut c_char {
    unsafe { Bytes::new(q) }.skip_while(is_xdigit).raw()
}

/// The first white space byte in `p`, or its NUL; [`skip::to_white`].
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn skiptowhite(p: *const c_char) -> *mut c_char {
    unsafe { Bytes::new(p) }
        .skip_while(|byte| !is_white(byte))
        .raw()
}

/// [`skiptowhite`], but a backslash or CTRL-V hides the byte after it;
/// [`skip::to_white_esc`].
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
        let escapes = (byte == b'\\' || c_int::from(byte) == crate::keycodes::Ctrl_V) && next != 0;
        cursor.advance(1 + usize::from(escapes));
    }
}

/// The bytes a decimal number at `p` can occupy -- leading white space, a
/// sign, digits -- which is everything [`skip::decimal`] reads, without
/// measuring the rest of the string.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
unsafe fn number_span<'a>(p: *const c_char) -> &'a [u8] {
    let cursor = unsafe { Bytes::new(p) };
    let mut end = cursor.skip_while(|b| matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'));
    if matches!(end.byte(), b'-' | b'+') {
        end.advance(1);
    }
    let end = end.skip_while(is_digit);
    let len = end.raw().addr() - p.addr();
    // SAFETY: every byte up to `end` was read as part of the string.
    unsafe { slice::from_raw_parts(p.cast::<u8>(), len) }
}

/// Read a decimal number at `*cursor`, advancing it past the digits; see
/// [`skip::decimal`]. Answers false when the value did not fit, in which
/// case `*nr` holds the saturated value.
///
/// # Safety
/// `*cursor` must be a NUL-terminated string, and `nr` writable.
pub(crate) unsafe fn try_getdigits(cursor: *mut *mut c_char, nr: *mut intmax_t) -> bool {
    let (number, used, overflow) = skip::decimal(unsafe { number_span(*cursor) });
    unsafe { *cursor = (*cursor).add(used) };
    unsafe { *nr = number };
    !overflow
}

/// [`try_getdigits`] over a slice: the number `text[at..]` starts with and
/// the offset past it, or `None` when the value did not fit.
pub(crate) fn try_getdigits_at(text: &[u8], at: usize) -> Option<(intmax_t, usize)> {
    let (number, used, overflow) = skip::decimal(&text[at..]);
    (!overflow).then_some((number, at + used))
}

/// [`try_getdigits`], answering `def` when the value did not fit; see
/// [`getdigits_at`].
///
/// # Safety
/// `*cursor` must be a NUL-terminated string.
pub unsafe fn getdigits(cursor: *mut *mut c_char, strict: bool, def: intmax_t) -> intmax_t {
    let (number, used) = getdigits_at(unsafe { number_span(*cursor) }, 0, strict, def);
    unsafe { *cursor = (*cursor).add(used) };
    number
}

/// [`getdigits`] narrowed to an `int`; see [`getdigits_int_at`].
///
/// # Safety
/// `*cursor` must be a NUL-terminated string.
pub unsafe fn getdigits_int(cursor: *mut *mut c_char, strict: bool, def: c_int) -> c_int {
    let (number, used) = getdigits_int_at(unsafe { number_span(*cursor) }, 0, strict, def);
    unsafe { *cursor = (*cursor).add(used) };
    number
}

/// The number `text[at..]` starts with, read as `strtoimax` reads one, and
/// the offset past what it consumed.
///
/// `strict` says the caller has already established that there *are* digits
/// here, so a value it cannot represent is a bad number rather than a parse
/// failure, and `def` would be misleading. Every one of those callers is
/// reading text a user typed -- an option value, a `:sign` id, a `:breakadd`
/// line number -- so an unrepresentable value **saturates** rather than
/// failing. Upstream `abort()`s here instead, which
/// `:set breakindentopt=min:99999999999999999999999` reaches from a modeline.
pub(crate) fn getdigits_at(
    text: &[u8],
    at: usize,
    strict: bool,
    def: intmax_t,
) -> (intmax_t, usize) {
    let (number, used, overflow) = skip::decimal(&text[at..]);
    (if overflow && !strict { def } else { number }, at + used)
}

/// [`getdigits_at`] narrowed to an `int`. A `strict` value outside the range
/// saturates; otherwise one that does not fit answers `def`.
pub(crate) fn getdigits_int_at(text: &[u8], at: usize, strict: bool, def: c_int) -> (c_int, usize) {
    let (number, past) = getdigits_at(text, at, strict, intmax_t::from(def));
    (narrow(number, strict, def), past)
}

/// A number narrowed to an `int`: clamped when `strict`, `def` when it does
/// not fit otherwise.
fn narrow(number: intmax_t, strict: bool, def: c_int) -> c_int {
    if strict {
        let clamped = number.clamp(intmax_t::from(c_int::MIN), intmax_t::from(c_int::MAX));
        return c_int::try_from(clamped).expect("clamped into `c_int`'s range");
    }
    c_int::try_from(number).unwrap_or(def)
}

/// [`getdigits`] narrowed to an `int32_t`, with [`getdigits_int`]'s shape.
///
/// # Safety
/// `*cursor` must be a NUL-terminated string.
pub unsafe fn getdigits_int32(cursor: *mut *mut c_char, strict: bool, def: int32_t) -> int32_t {
    unsafe { getdigits_int(cursor, strict, def) }
}

/// Whether the character at `p` belongs to a word in the current buffer.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
pub unsafe fn vim_iswordp(p: *const c_char) -> bool {
    unsafe { vim_iswordp_buf(p, Buf::current()) }
}

/// Whether the character at `p` belongs to a word in `buffer`; see
/// [`is_word_at`].
///
/// # Safety
/// `p` must point into a NUL-terminated string.
pub unsafe fn vim_iswordp_buf(p: *const c_char, buffer: Buf) -> bool {
    // A byte that leads no sequence is judged as itself, without measuring.
    let lead = unsafe { Bytes::new(p) }.byte();
    if crate::mbyte::utf8len_tab[usize::from(lead)] == 1 {
        return super::vim_iswordc_buf(c_int::from(lead), buffer);
    }
    is_word_at(
        unsafe { cstr::prefix_at(p, crate::types::MB_MAXCHAR) },
        buffer,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the C signature, kept for its callers"
)]
/// Parse the number at `start` into whichever of the out-arguments is not
/// null; see [`str2nr_scan`].
///
/// # Safety
///
/// `start` must point at a NUL-terminated string, or at `maxlen` readable
/// bytes. Each out-argument must be null or point at a writable value of its
/// own.
pub unsafe fn vim_str2nr(
    start: *const c_char,
    prep: *mut c_int,
    len: *mut c_int,
    what: Str2NrBases,
    nptr: *mut VarNumber,
    unptr: *mut UVarNumber,
    maxlen: c_int,
    strict: bool,
    overflow: *mut bool,
) {
    let text = match usize::try_from(maxlen) {
        Ok(0) | Err(_) => unsafe { number_text(start) },
        Ok(maxlen) => unsafe { cstr::prefix_at(start, maxlen) },
    };
    let parsed = str2nr_scan(text, maxlen, what, strict, !nptr.is_null());
    unsafe {
        if !len.is_null() {
            *len = c_int::try_from(parsed.len).unwrap_or(c_int::MAX);
        }
        if parsed.overflow && !overflow.is_null() {
            *overflow = true;
        }
        if parsed.rejected {
            return;
        }
        if !prep.is_null() {
            *prep = parsed.prefix;
        }
        if !nptr.is_null() {
            *nptr = parsed.value;
        }
        if !unptr.is_null() {
            *unptr = parsed.magnitude;
        }
    }
}

/// The bytes at `start` a number can be read from: the run a number is
/// spelled with (a leading `-`, digits, the letters of a base prefix and of
/// hex digits, the `'` separator) and the one byte after it, which is all
/// [`str2nr_scan`]'s strict check and look-aheads read. Not the whole
/// string: a caller handing `vim_str2nr` a line with no `maxlen` paid a
/// `strlen` of the line per number.
///
/// # Safety
/// `start` must be a NUL-terminated string.
unsafe fn number_text<'a>(start: *const c_char) -> &'a [u8] {
    let mut n = 0;
    loop {
        // SAFETY: the caller's string, read up to its NUL at most.
        let b = unsafe { *start.add(n) }.cast_unsigned();
        if b == 0 {
            break;
        }
        n += 1;
        if !(b.is_ascii_alphanumeric() || b == b'\'' || (n == 1 && b == b'-')) {
            break;
        }
    }
    // SAFETY: the `n` bytes just read.
    unsafe { slice::from_raw_parts(start.cast::<u8>(), n) }
}

/// Whether the backslash at `str` escapes the byte after it; [`escapes_next`].
///
/// # Safety
/// `str` must be a NUL-terminated string.
pub unsafe fn rem_backslash(str: *const c_char) -> bool {
    escapes_next(unsafe { cstr::prefix_at(str, 2) })
}

/// Remove the escaping backslashes from `p`, in place; [`backslash_halved`].
///
/// # Safety
/// `p` must be a writable NUL-terminated string.
pub unsafe fn backslash_halve(p: *mut c_char) {
    let len = unsafe { cstr::bytes_at(p) }.len();
    let halved = backslash_halved(unsafe { cstr::slice_at(p, len) });
    // The result is never longer than the string, so it and its terminator
    // fit where the string was.
    unsafe { p.cast::<u8>().copy_from(halved.as_ptr(), halved.len()) };
    unsafe { *p.add(halved.len()) = 0 };
}

/// [`backslash_halve`] into a fresh string the caller owns.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn backslash_halve_save(p: *const c_char) -> *mut c_char {
    ThinCString::from_vec(backslash_halved(unsafe { cstr::bytes_at(p) })).into_raw()
}

/// Replace every unprintable byte in `buf` with its display form, in place;
/// [`trans_characters_in`].
///
/// # Safety
/// `buf` must be a NUL-terminated string in a writable buffer of `bufsize`
/// bytes.
pub unsafe fn trans_characters(buf: *mut c_char, bufsize: c_int) {
    let bufsize = usize::try_from(bufsize).unwrap_or(0);
    trans_characters_in(unsafe { slice::from_raw_parts_mut(buf.cast::<u8>(), bufsize) });
}

/// Copy `s` into `buf` with every unprintable character replaced by its
/// display form, NUL-terminated; [`transstr_into`]. A negative `slen` means
/// "to the NUL".
///
/// # Safety
/// `s` must be a NUL-terminated string and `buf` writable for `buflen` bytes,
/// at least one.
pub unsafe fn transstr_buf(
    s: *const c_char,
    slen: ssize_t,
    buf: *mut c_char,
    buflen: size_t,
    untab: bool,
) -> size_t {
    let out = unsafe { slice::from_raw_parts_mut(buf.cast::<u8>(), buflen) };
    transstr_into(
        unsafe { cstr::bytes_at(s) },
        usize::try_from(slen).ok(),
        out,
        untab,
    )
}

/// [`transstr_buf`] into a fresh string the caller owns; [`transstr_owned`].
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn transstr(s: *const c_char, untab: bool) -> *mut c_char {
    transstr_owned(unsafe { cstr::bytes_at(s) }, untab).into_raw()
}

/// Fold `orglen` bytes of `str` to lowercase; [`foldcase`]. With a null `buf`
/// the result is a fresh string the caller owns; otherwise it is written
/// into `buf`, which is answered.
///
/// # Safety
/// `str` must hold `orglen` readable bytes; `buf`, if given, must be writable
/// for `buflen` bytes, at least one.
pub unsafe fn str_foldcase(
    str: *mut c_char,
    orglen: c_int,
    buf: *mut c_char,
    buflen: c_int,
) -> *mut c_char {
    let text = unsafe { cstr::slice_at(str, usize::try_from(orglen).unwrap_or(0)) };
    if buf.is_null() {
        return ThinCString::from_vec(foldcase(text, None)).into_raw();
    }
    let room = usize::try_from(buflen).unwrap_or(0);
    let folded = foldcase(text, Some(room));
    unsafe { buf.cast::<u8>().copy_from(folded.as_ptr(), folded.len()) };
    unsafe { *buf.add(folded.len()) = 0 };
    buf
}

/// Reverse the bytes of `str` in place, up to `end` or the NUL;
/// [`mirror_ascii`].
///
/// # Safety
/// `str` must be NUL-terminated when `end` is null, and reach `end`
/// otherwise; the bytes must be writable.
pub unsafe fn rl_mirror_ascii(str: *mut c_char, end: *mut c_char) {
    let len = if end.is_null() {
        unsafe { cstr::bytes_at(str) }.len()
    } else {
        end.addr() - str.addr()
    };
    mirror_ascii(unsafe { slice::from_raw_parts_mut(str.cast::<u8>(), len) });
}

/// The display width of the character at `p`.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
pub unsafe fn ptr2cells(p: *const c_char) -> c_int {
    let byte = unsafe { *p.cast::<uint8_t>() };
    if byte >= 0x80 {
        return unsafe { crate::mbyte::utf_ptr2cells(p) };
    }
    byte2cells(c_int::from(byte))
}
