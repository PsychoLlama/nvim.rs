//! Walking a NUL-terminated string: the pointer forms of the `skip*` family,
//! and the digit readers built on them.
//!
//! [`skip`](super::skip) is the same set of questions asked of a `&[u8]`,
//! answering an offset. These are what upstream wrote and what most of the
//! tree still calls: they take a `*const c_char` and hand back a pointer
//! into it, so a caller either walks on from there or subtracts to get a
//! length. Every one of them goes through [`Bytes`], whose *construction* is
//! the unsafe step, so the walking itself is ordinary safe code.
//!
//! The `getdigits` family sits here rather than beside
//! [`vim_str2nr`](super::vim_str2nr) because it is the same shape: a cursor
//! the callee advances past what it read. It parses one unprefixed decimal
//! number through `strtoimax`, where `vim_str2nr` parses the bases.
//!
//! Original: `src/nvim/charset.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_int, c_long};

use ::libc::__errno_location;

use super::{Bytes, ERANGE, is_bdigit, is_digit, is_white, is_xdigit, skip};
use crate::cursor::get_cursor_line_ptr;
use crate::keycodes::Ctrl_V;
use crate::memory::xstrchrnul;
use crate::os::cshim::strtoimax;
use crate::types::{int32_t, intmax_t, intptr_t, size_t, uint8_t};

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
pub(crate) unsafe fn skipwhite_len(p: *const c_char, len: size_t) -> *mut c_char {
    // SAFETY: the caller guarantees `len` readable bytes at `p`.
    let bytes = unsafe { core::slice::from_raw_parts(p.cast::<uint8_t>(), len) };
    p.wrapping_add(skip::white(bytes)) as *mut c_char
}

/// The indent of the cursor's line, in bytes.
pub(crate) fn getwhitecols_curline() -> intptr_t {
    unsafe { getwhitecols(get_cursor_line_ptr()) }
}

/// How many leading bytes of `p` are white space.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub(crate) unsafe fn getwhitecols(p: *const c_char) -> intptr_t {
    (unsafe { skipwhite(p) }.addr() - p.addr()).cast_signed()
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
        let escapes = (byte == b'\\' || c_int::from(byte) == Ctrl_V) && next != 0;
        cursor.advance(1 + usize::from(escapes));
    }
}

/// The next newline in `p`, or its NUL.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub(crate) unsafe fn skip_to_newline(p: *const c_char) -> *mut c_char {
    unsafe { xstrchrnul(p, b'\n'.cast_signed()) }
}

/// Read a decimal number at `*pp`, advancing it past the digits. Answers
/// false when the value did not fit, in which case `*nr` holds the clamped
/// `strtoimax` result.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub(crate) unsafe fn try_getdigits(cursor: *mut *mut c_char, nr: *mut intmax_t) -> bool {
    // SAFETY: `*pp` is a NUL-terminated string, `strtoimax` advances it past
    // whatever it consumed, and `errno` is the C library's own thread-local.
    let number = unsafe {
        *__errno_location() = 0;
        strtoimax(*cursor, cursor, 10)
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
pub unsafe fn getdigits(cursor: *mut *mut c_char, strict: bool, def: intmax_t) -> intmax_t {
    let mut number: intmax_t = 0;
    // SAFETY: forwarded to the caller's contract; `number` is a local.
    let ok = unsafe { try_getdigits(cursor, &raw mut number) };
    if ok || strict { number } else { def }
}

/// [`getdigits`] narrowed to an `int`.
///
/// A `strict` value outside the range saturates -- see [`getdigits`] for why
/// it is not an abort.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_int(cursor: *mut *mut c_char, strict: bool, def: c_int) -> c_int {
    let number = unsafe { getdigits(cursor, strict, intmax_t::from(def)) };
    narrow_int(number, strict, def)
}

/// [`getdigits_int`] as an offset walk: the number `buffer[at..]` starts with,
/// and the offset past what it consumed.
///
/// The offset form a converted command-line parse wants. `buffer` is a
/// writable NUL-terminated buffer the caller owns -- nothing here writes to
/// it, but a `&mut` is what gives the walk a pointer it may hold -- and this
/// keeps `strtoimax`'s exact contract, leading blanks and a sign included,
/// which a hand-written digit scanner would have to guess at.
///
/// # Panics
/// If `buffer` holds no NUL at or after `at`.
pub(crate) fn getdigits_int_at(
    buffer: &mut [u8],
    at: usize,
    strict: bool,
    def: c_int,
) -> (c_int, usize) {
    let (number, past) = getdigits_at(buffer, at, strict, intmax_t::from(def));
    (narrow_int(number, strict, def), past)
}

/// [`getdigits`] as an offset walk, in its full width -- the shape a caller
/// that wants C's truncating cast rather than [`getdigits_int_at`]'s clamp
/// reads. See [`getdigits_int_at`] for why the buffer is `&mut`.
///
/// # Panics
/// If `buffer` holds no NUL at or after `at`.
pub(crate) fn getdigits_at(
    buffer: &mut [u8],
    at: usize,
    strict: bool,
    def: intmax_t,
) -> (intmax_t, usize) {
    assert!(
        buffer[at..].contains(&0),
        "the walk needs a terminator to stop at"
    );
    let base = buffer.as_mut_ptr();
    // SAFETY: `at` is in bounds of `buffer`, which the assert above says is
    // NUL-terminated from there.
    let mut cursor = unsafe { base.add(at) }.cast::<c_char>();
    // SAFETY: as above; `getdigits` leaves the cursor within the string.
    let number = unsafe { getdigits(&raw mut cursor, strict, def) };
    // SAFETY: both pointers are into `buffer`, the base first.
    let past = unsafe { cursor.cast::<uint8_t>().offset_from(base) };
    (number, past.cast_unsigned())
}

/// [`getdigits_int`]'s narrowing, shared with [`getdigits_int_at`].
fn narrow_int(number: intmax_t, strict: bool, def: c_int) -> c_int {
    if strict {
        let clamped = number.clamp(intmax_t::from(c_int::MIN), intmax_t::from(c_int::MAX));
        return c_int::try_from(clamped).expect("clamped into `c_int`'s range");
    }
    c_int::try_from(number).unwrap_or(def)
}

/// [`getdigits`] narrowed to an `int32_t`, with [`getdigits_int`]'s shape.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub unsafe fn getdigits_int32(cursor: *mut *mut c_char, strict: bool, def: int32_t) -> int32_t {
    let number = unsafe { getdigits(cursor, strict, intmax_t::from(def)) };
    if strict {
        let clamped = number.clamp(intmax_t::from(int32_t::MIN), intmax_t::from(int32_t::MAX));
        return int32_t::try_from(clamped).expect("clamped into `int32_t`'s range");
    }
    int32_t::try_from(number).unwrap_or(def)
}

/// [`getdigits`] narrowed to a `long`. Note that unlike the `int` forms this
/// does not range-check, because on this platform it cannot fail.
///
/// # Safety
/// `*pp` must be a NUL-terminated string.
pub(crate) unsafe fn getdigits_long(cursor: *mut *mut c_char, strict: bool, def: c_long) -> c_long {
    unsafe { getdigits(cursor, strict, intmax_t::from(def)) as c_long }
}

/// Whether `lbuf` holds nothing but white space.
///
/// # Safety
/// `lbuf` must be a NUL-terminated string.
pub(crate) unsafe fn vim_isblankline(lbuf: *mut c_char) -> bool {
    // SAFETY: forwarded to the caller's contract; `skipwhite` stays inside.
    let byte = unsafe { Bytes::new(skipwhite(lbuf)) }.byte();
    byte == 0 || byte == b'\r' || byte == b'\n'
}
