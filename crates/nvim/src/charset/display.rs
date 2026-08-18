//! Rendering text for display: the width a character occupies on screen and
//! the `^X` / `<xx>` forms that stand in for one that cannot be shown.
//!
//! The `transchar*` family answers out of one shared buffer, so the
//! rendering itself is done as a *value* ([`render::Rendered`], built by
//! safe code in `transchar.rs`) and parked in the buffer at the very last
//! step. Everything that only needs the bytes — `transstr_buf`,
//! `trans_characters` — takes the value and never touches the buffer at
//! all.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::{ptr, slice};

use crate::garray::{ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, dy_flags};
use crate::mbyte::{
    mb_tolower, utf_char2bytes, utf_char2cells, utf_char2len, utf_ptr2cells, utf_ptr2char,
    utf_ptr2len, utfc_ptr2len,
};
use crate::memory::{xmalloc, xrealloc};
use crate::option::get_fileformat;
use crate::types::{StringBuilder, buf_T, garray_T, size_t, ssize_t, uint8_t};
use ::libc::strlen;

use super::{
    CT_CELL_MASK, EOL_MAC, NL, NUL, TAB, chartab, chartab_initialized, kOptDyFlagUhex,
    transchar as render, vim_isprintc,
};
use crate::keycodes::K_SPECIAL;
use crate::pos::MAXCOL;

const CAR: c_int = 13;
const KS_ZERO: c_int = 255;
const KS_SPECIAL: c_int = 254;

/// A cursor over the characters of a NUL-terminated string.
///
/// Building one is the unsafe step. Every step is by the length the `utf_*`
/// measures report, and those never count past a NUL — an incomplete
/// sequence measures as one byte — so a cursor that starts inside the
/// string stays inside it, and its reads are then ordinary safe code.
#[derive(Clone, Copy)]
struct Chars(*const c_char);

impl Chars {
    /// # Safety
    /// `p` must point into a NUL-terminated string.
    #[inline(always)]
    unsafe fn new(p: *const c_char) -> Self {
        Chars(p)
    }

    /// The byte under the cursor.
    #[inline(always)]
    fn byte(self) -> uint8_t {
        // SAFETY: the cursor is inside its string, so this byte is readable.
        unsafe { *self.0 as uint8_t }
    }

    /// The length of the whole character under the cursor, composing
    /// characters included.
    #[inline(always)]
    fn char_len(self) -> usize {
        // SAFETY: as [`Chars::byte`].
        unsafe { utfc_ptr2len(self.0) as usize }
    }

    /// The length of just the code point under the cursor.
    #[inline(always)]
    fn code_len(self) -> usize {
        // SAFETY: as [`Chars::byte`].
        unsafe { utf_ptr2len(self.0) as usize }
    }

    /// The code point under the cursor.
    #[inline(always)]
    fn code(self) -> c_int {
        // SAFETY: as [`Chars::byte`].
        unsafe { utf_ptr2char(self.0) }
    }

    /// The `n` bytes at the cursor, which the caller has measured.
    #[inline(always)]
    fn bytes(self, n: usize) -> &'static [uint8_t] {
        // SAFETY: `n` came from one of the measures above, so those bytes
        // are part of the string.
        unsafe { slice::from_raw_parts(self.0.cast::<uint8_t>(), n) }
    }

    /// The cursor `n` bytes further on, which the caller has measured.
    #[inline(always)]
    fn skip(self, n: usize) -> Self {
        Chars(self.0.wrapping_add(n))
    }

    #[inline(always)]
    fn advance(&mut self, n: usize) {
        self.0 = self.0.wrapping_add(n);
    }

    #[inline(always)]
    fn raw(self) -> *const c_char {
        self.0
    }
}

/// The display width of the byte `b` from the global table.
#[inline(always)]
fn table_cells(b: uint8_t) -> c_int {
    (chartab(b) & CT_CELL_MASK) as c_int
}

/// The display width of a byte, zero for a multibyte lead byte.
#[inline(always)]
fn byte_cells(b: c_int) -> c_int {
    if b >= 0x80 {
        return 0;
    }
    table_cells(b as uint8_t)
}

/// Replace every unprintable byte in `buf` with its display form, in place.
///
/// Stops early rather than overflowing when the expansions no longer fit in
/// `bufsize`, leaving the tail untranslated.
///
/// # Safety
/// `buf` must be a NUL-terminated string in a buffer of `bufsize` bytes.
pub unsafe fn trans_characters(buf: *mut c_char, bufsize: c_int) {
    // SAFETY: the caller's buffer holds `bufsize` bytes.
    let bytes = unsafe { slice::from_raw_parts_mut(buf.cast::<uint8_t>(), bufsize as usize) };
    // Bytes of the string still to be translated, and bytes left over after
    // it. Both are counted from the cursor, so their sum is invariant.
    let mut len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let mut room = bufsize as isize - len as isize;
    let mut at = 0usize;
    while bytes[at] != 0 {
        // SAFETY: `at` is inside the NUL-terminated string.
        let mut step = unsafe { utfc_ptr2len(buf.add(at)) } as usize;
        if step > 1 {
            // A multibyte character is left alone.
            len -= step;
        } else {
            // SAFETY: the current buffer is valid.
            let trs = unsafe { render_byte(curbuf.get(), bytes[at] as c_int) };
            step = trs.len;
            if step > 1 {
                room -= step as isize - 1;
                if room <= 0 {
                    return;
                }
                // The tail, terminator included, slides up to make room.
                bytes.copy_within(at + 1..at + 1 + len, at + step);
            }
            bytes[at..at + step].copy_from_slice(&trs.bytes[..step]);
            len -= 1;
        }
        at += step;
    }
}

/// How many cells [`transstr_buf`] would need for `s`, excluding the NUL.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn transstr_len(s: *const c_char, untab: bool) -> size_t {
    // SAFETY: forwarded to the caller's contract.
    let mut cursor = unsafe { Chars::new(s) };
    let mut len: size_t = 0;
    loop {
        let byte = cursor.byte();
        if byte == 0 {
            return len;
        }
        let l = cursor.char_len();
        if l > 1 {
            // SAFETY: this only reads the global table.
            if unsafe { vim_isprintc(cursor.code()) } {
                len += l;
            } else {
                // An unprintable multibyte character is spelled out one
                // codepoint at a time, composing characters included.
                let mut off: size_t = 0;
                while off < l {
                    let at = cursor.skip(off);
                    len += render::hex_form(at.code()).len;
                    off += at.code_len();
                }
            }
            cursor.advance(l);
        } else if byte as c_int == TAB && !untab {
            len += 1;
            cursor.advance(1);
        } else {
            let cells = byte_cells(byte as c_int);
            cursor.advance(1);
            // A zero width means the table has no entry: `<xx>` is four.
            len += if cells > 0 { cells } else { 4 } as size_t;
        }
    }
}

/// Copy `s` into `buf` with every unprintable character replaced by its
/// display form, NUL-terminated. Answers the number of bytes written.
///
/// A negative `slen` means "to the NUL". The output is truncated at a
/// character boundary rather than overrunning `buflen`.
///
/// # Safety
/// `s` must hold `slen` readable bytes (or be NUL-terminated) and `buf` must
/// have room for `buflen`, which must be at least one: the last of those
/// bytes is the terminator's.
pub unsafe fn transstr_buf(
    s: *const c_char,
    slen: ssize_t,
    buf: *mut c_char,
    buflen: size_t,
    untab: bool,
) -> size_t {
    // SAFETY: the caller's buffer holds `buflen` bytes.
    let out = unsafe { slice::from_raw_parts_mut(buf.cast::<uint8_t>(), buflen) };
    // The last byte of the buffer belongs to the terminator.
    let limit = buflen - 1;
    // SAFETY: forwarded to the caller's contract.
    let mut cursor = unsafe { Chars::new(s) };
    let mut read: size_t = 0;
    let mut written: size_t = 0;
    while (slen < 0 || read < slen as size_t) && cursor.byte() != 0 && written < limit {
        let l = cursor.char_len();
        if l > 1 {
            if written + l > limit {
                break;
            }
            // SAFETY: this only reads the global table.
            if unsafe { vim_isprintc(cursor.code()) } {
                out[written..written + l].copy_from_slice(cursor.bytes(l));
                written += l;
            } else {
                let mut off: size_t = 0;
                while off < l {
                    let at = cursor.skip(off);
                    let hex = render::hex_form(at.code());
                    if written + hex.len > limit {
                        break;
                    }
                    out[written..written + hex.len].copy_from_slice(&hex.bytes[..hex.len]);
                    written += hex.len;
                    off += at.code_len();
                }
            }
            cursor.advance(l);
            read += l;
        } else if cursor.byte() as c_int == TAB && !untab {
            out[written] = cursor.byte();
            written += 1;
            cursor.advance(1);
            read += 1;
        } else {
            // SAFETY: the current buffer is valid.
            let tb = unsafe { render_byte(curbuf.get(), cursor.byte() as c_int) };
            cursor.advance(1);
            read += 1;
            if written + tb.len > limit {
                break;
            }
            out[written..written + tb.len].copy_from_slice(&tb.bytes[..tb.len]);
            written += tb.len;
        }
    }
    debug_assert!(written <= limit, "buf_p <= buf_e");
    out[written] = NUL as uint8_t;
    written
}

/// [`transstr_buf`] into a freshly allocated string the caller owns.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn transstr(s: *const c_char, untab: bool) -> *mut c_char {
    // SAFETY: forwarded to the caller's contract.
    let len = unsafe { transstr_len(s, untab) } + 1;
    // SAFETY: the allocation is exactly what the rendering needs.
    let buf = unsafe { xmalloc(len) } as *mut c_char;
    // SAFETY: as above.
    unsafe { transstr_buf(s, -1, buf, len, untab) };
    buf
}

/// Append [`transstr`]'s rendering of `s` to a string builder, growing it to
/// the next power of two as the kvec macros did. Answers the length added.
///
/// # Safety
/// `str` must be a valid builder; `s` may be null.
pub unsafe fn kv_transstr(str: *mut StringBuilder, s: *const c_char, untab: bool) -> size_t {
    if s.is_null() {
        return 0;
    }
    // SAFETY: forwarded to the caller's contract.
    let len = unsafe { transstr_len(s, untab) };
    // SAFETY: `str` is a valid builder.
    let mut builder = unsafe { *str };
    let needed = builder.size + len + 1;
    if builder.capacity < needed {
        builder.capacity = needed.next_power_of_two();
        // SAFETY: the builder owns `items`, or it is null and this allocates.
        builder.items =
            unsafe { xrealloc(builder.items as *mut c_void, builder.capacity) } as *mut c_char;
    }
    // SAFETY: the builder now has room for `len + 1` bytes past its length.
    unsafe { transstr_buf(s, -1, builder.items.add(builder.size), len + 1, untab) };
    builder.size += len;
    // SAFETY: `str` is writable.
    unsafe { *str = builder };
    len
}

/// Fold `str` to lowercase.
///
/// With a null `buf` the result is a fresh allocation the caller owns;
/// otherwise it is written into `buf`, truncated to `buflen`, and a
/// character whose lowercase form no longer fits is left as it was.
///
/// # Safety
/// `str` must hold `orglen` readable bytes; `buf`, if given, must have room
/// for `buflen`.
pub unsafe fn str_foldcase(
    str: *mut c_char,
    orglen: c_int,
    buf: *mut c_char,
    buflen: c_int,
) -> *mut c_char {
    let mut ga = garray_T::default();
    let mut len = orglen;
    if buf.is_null() {
        // SAFETY: `ga` is a local, and `str` holds `orglen` readable bytes.
        unsafe {
            ga_init(&raw mut ga, 1, 10);
            ga_grow(&raw mut ga, len + 1);
            ptr::copy(str, ga.ga_data as *mut c_char, len as usize);
        }
        ga.ga_len = len;
    } else {
        if len >= buflen {
            len = buflen - 1;
        }
        // SAFETY: `buf` holds `buflen` bytes and `str` holds `len`.
        unsafe { ptr::copy(str, buf, len as usize) };
    }

    // From here on `at(i)` is the one place that knows which buffer is in
    // play; `ga.ga_data` moves under us whenever the collection grows.
    let at = |ga: &garray_T, i: c_int| -> *mut c_char {
        if buf.is_null() {
            (ga.ga_data as *mut c_char).wrapping_offset(i as isize)
        } else {
            buf.wrapping_offset(i as isize)
        }
    };
    // SAFETY: index `len` is the terminator's, inside either buffer.
    unsafe { *at(&ga, len) = NUL as c_char };

    let mut i: c_int = 0;
    loop {
        // SAFETY: the walk stops at the terminator written above.
        let cursor = unsafe { Chars::new(at(&ga, i)) };
        if cursor.byte() == 0 {
            break;
        }
        let c = cursor.code();
        let olen = cursor.code_len() as c_int;
        let mut lc = mb_tolower(c);
        // Only ASCII and real multibyte characters fold; a lone Latin-1 byte
        // has no case in this encoding.
        if (c < 0x80 || olen > 1) && c != lc {
            let mut nlen = utf_char2len(lc);
            if olen != nlen {
                if nlen > olen {
                    if buf.is_null() {
                        // SAFETY: `ga` is a live growable array.
                        unsafe { ga_grow(&raw mut ga, nlen - olen + 1) };
                    } else if len + nlen - olen >= buflen {
                        // No room to grow: keep the original character.
                        lc = c;
                        nlen = olen;
                    }
                }
                if olen != nlen {
                    // SAFETY: the tail, terminator included, still fits: the
                    // buffer either grew or the character did not.
                    unsafe {
                        let src = at(&ga, i + olen);
                        ptr::copy(src, at(&ga, i + nlen), strlen(src) + 1);
                    }
                    if buf.is_null() {
                        ga.ga_len += nlen - olen;
                    } else {
                        len += nlen - olen;
                    }
                }
            }
            // SAFETY: `nlen` bytes were just made available at `i`.
            unsafe { utf_char2bytes(lc, at(&ga, i)) };
        }
        // SAFETY: the character at `i` is inside the buffer.
        i += unsafe { Chars::new(at(&ga, i)) }.char_len() as c_int;
    }

    if buf.is_null() {
        ga.ga_data as *mut c_char
    } else {
        buf
    }
}

/// The shared buffer every `transchar*` answer lives in. Each call
/// overwrites the previous one's result.
static transchar_charbuf: GlobalCell<[uint8_t; render::MAX_LEN]> =
    GlobalCell::new([0; render::MAX_LEN]);

/// Park a rendering in the shared buffer and answer a pointer to it.
#[inline(always)]
fn share(rendered: &render::Rendered) -> *mut c_char {
    let charbuf = transchar_charbuf.ptr() as *mut c_char;
    // SAFETY: the cell holds a live `MAX_LEN`-byte array, and a rendering
    // with its terminator is never longer than that.
    unsafe { write_rendered(charbuf, rendered) };
    charbuf
}

/// Copy a rendering, and the NUL after it, to `dst`.
///
/// # Safety
/// `dst` must have room for `rendered.len + 1` bytes.
#[inline(always)]
unsafe fn write_rendered(dst: *mut c_char, rendered: &render::Rendered) {
    // SAFETY: the caller guarantees the room.
    unsafe {
        ptr::copy_nonoverlapping(
            rendered.bytes.as_ptr(),
            dst as *mut uint8_t,
            rendered.len + 1,
        )
    };
}

/// The display form of the unprintable byte `c`, as a value.
///
/// # Safety
/// `buf` may be null; otherwise it must be a valid buffer.
#[inline(always)]
unsafe fn render_nonprint(buf: *const buf_T, c: c_int) -> render::Rendered {
    let c = if c == NL {
        // A NUL is stored as a newline internally.
        NUL
    // SAFETY: `buf` is a valid buffer when it is not null.
    } else if !buf.is_null() && c == CAR && unsafe { get_fileformat(buf) } == EOL_MAC {
        NL
    } else {
        c
    };
    debug_assert!(c <= 0xff, "c <= 0xff");
    if dy_flags.get() & kOptDyFlagUhex != 0 || c > 0x7f {
        render::hex_form(c)
    } else {
        render::control_form(c)
    }
}

/// The display form of the character `c` as it would appear in `buf`, as a
/// value.
///
/// # Safety
/// `buf` may be null; otherwise it must be a valid buffer.
#[inline(always)]
unsafe fn render_char(buf: *const buf_T, c: c_int) -> render::Rendered {
    // A negative code is one of the key-translation escapes; it renders as
    // its byte behind a `~@`.
    let (prefix, c) = if c < 0 {
        let (prefix, byte) = render::negative_form(c);
        (Some(prefix), byte)
    } else {
        (None, c)
    };
    // Before the tables exist, printable ASCII is all that can be trusted.
    // SAFETY: `vim_isprintc` only reads the global table.
    let body = if (!chartab_initialized.get() && (' ' as c_int..='~' as c_int).contains(&c))
        || (c <= 0xff && unsafe { vim_isprintc(c) })
    {
        render::Rendered::literal(c as uint8_t)
    } else if c <= 0xff {
        // SAFETY: forwarded to this function's contract.
        unsafe { render_nonprint(buf, c) }
    } else {
        render::hex_form(c)
    };
    match prefix {
        Some(prefix) => body.behind(prefix),
        None => body,
    }
}

/// [`render_char`] for a single byte: unlike it, a high byte is never taken
/// for a printable Latin-1 character.
///
/// # Safety
/// `buf` may be null; otherwise it must be a valid buffer.
#[inline(always)]
unsafe fn render_byte(buf: *const buf_T, c: c_int) -> render::Rendered {
    if c >= 0x80 {
        // SAFETY: forwarded to this function's contract.
        return unsafe { render_nonprint(buf, c) };
    }
    // SAFETY: as above.
    unsafe { render_char(buf, c) }
}

/// The display form of character `c`, in a shared buffer.
///
/// # Safety
/// The current buffer must be valid.
pub unsafe fn transchar(c: c_int) -> *mut c_char {
    // SAFETY: forwarded to the caller's contract.
    unsafe { transchar_buf(curbuf.get(), c) }
}

/// The display form of `c` as it would appear in `buf` (which decides how a
/// carriage return renders), in a shared buffer.
///
/// # Safety
/// `buf` may be null; otherwise it must be a valid buffer.
pub unsafe fn transchar_buf(buf: *const buf_T, c: c_int) -> *mut c_char {
    // SAFETY: forwarded to the caller's contract.
    share(&unsafe { render_char(buf, c) })
}

/// The display form of the single byte `c`. Unlike [`transchar_buf`] this
/// never treats a high byte as a printable Latin-1 character.
///
/// # Safety
/// `buf` may be null; otherwise it must be a valid buffer.
pub unsafe fn transchar_byte_buf(buf: *const buf_T, c: c_int) -> *mut c_char {
    // SAFETY: forwarded to the caller's contract.
    share(&unsafe { render_byte(buf, c) })
}

/// [`transchar_byte_buf`] for the current buffer.
///
/// # Safety
/// The current buffer must be valid.
pub unsafe fn transchar_byte(c: c_int) -> *mut c_char {
    // SAFETY: forwarded to the caller's contract.
    unsafe { transchar_byte_buf(curbuf.get(), c) }
}

/// Write the display form of the unprintable byte `c` into `charbuf`.
///
/// # Safety
/// `charbuf` must have room for five bytes; `buf` may be null.
pub unsafe fn transchar_nonprint(buf: *const buf_T, charbuf: *mut c_char, c: c_int) {
    // SAFETY: forwarded to the caller's contract; a byte's rendering and its
    // terminator are at most five bytes.
    unsafe { write_rendered(charbuf, &render_nonprint(buf, c)) };
}

/// Write `c`'s `<xx>` form into `buf`. Answers the length, excluding the NUL.
///
/// # Safety
/// `buf` must have room for nine bytes.
pub unsafe fn transchar_hex(buf: *mut c_char, c: c_int) -> size_t {
    let rendered = render::hex_form(c);
    // SAFETY: the caller's buffer holds the nine bytes the widest form needs.
    unsafe { write_rendered(buf, &rendered) };
    rendered.len
}

/// Reverse the bytes of `str` in place, up to `end` or the NUL.
///
/// This is only correct for single-byte text; the callers use it on
/// generated ASCII.
///
/// # Safety
/// `str` must be NUL-terminated when `end` is null.
pub unsafe fn rl_mirror_ascii(str: *mut c_char, end: *mut c_char) {
    // SAFETY: forwarded to the caller's contract.
    let len = if end.is_null() {
        unsafe { strlen(str) }
    } else {
        end.addr() - str.addr()
    };
    // SAFETY: the caller's buffer holds those bytes.
    unsafe { slice::from_raw_parts_mut(str, len) }.reverse();
}

/// The display width of the byte `b`, or zero for a multibyte lead byte.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn byte2cells(b: c_int) -> c_int {
    byte_cells(b)
}

/// The display width of character `c`. A negative code costs two cells more
/// for its `~@` prefix.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn char2cells(c: c_int) -> c_int {
    if c < 0 {
        let escaped = if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff
        };
        // SAFETY: forwarded to the caller's contract.
        return unsafe { char2cells(escaped) } + 2;
    }
    if c >= 0x80 {
        // SAFETY: this only reads the static width tables.
        return unsafe { utf_char2cells(c) };
    }
    table_cells((c & 0xff) as uint8_t)
}

/// The display width of the character at `p`.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
pub unsafe fn ptr2cells(p_in: *const c_char) -> c_int {
    // SAFETY: forwarded to the caller's contract.
    let byte = unsafe { *p_in.cast::<uint8_t>() };
    if byte >= 0x80 {
        // SAFETY: as above; the rest of the sequence follows it.
        return unsafe { utf_ptr2cells(p_in) };
    }
    table_cells(byte)
}

/// The display width of the whole string `s`.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn vim_strsize(s: *const c_char) -> c_int {
    // SAFETY: forwarded to the caller's contract.
    unsafe { vim_strnsize(s, MAXCOL) }
}

/// The display width of at most `len` bytes of `s`.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn vim_strnsize(s: *const c_char, len: c_int) -> c_int {
    debug_assert!(!s.is_null(), "s != NULL");
    // SAFETY: forwarded to the caller's contract.
    let mut cursor = unsafe { Chars::new(s) };
    let mut len = len;
    let mut size = 0;
    while cursor.byte() != 0 && {
        len -= 1;
        len >= 0
    } {
        let l = cursor.char_len();
        // SAFETY: the cursor is inside the string.
        size += unsafe { ptr2cells(cursor.raw()) };
        cursor.advance(l);
        len -= l as c_int - 1;
    }
    size
}
