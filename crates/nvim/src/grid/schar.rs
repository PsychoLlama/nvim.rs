#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`schar_get`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]

//! `ScreenChar`: the glyph in a screen cell.
//!
//! A cell holds one grapheme cluster, which is usually short. A `ScreenChar` is
//! a `u32` that stores the UTF-8 bytes inline when there are at most four of
//! them, and otherwise an index into a global intern table (`glyph_cache`)
//! tagged by a low byte of `0xFF` -- which no UTF-8 lead byte can be, so the
//! two cases never collide. Zero means "right half of a double-width cell".
//!
//! Little-endian only; the original carries an `ORDER_BIG_ENDIAN` branch that
//! puts the tag in the high byte instead, and the transpile dropped it.

use super::*;
use crate::mbyte::{c_cells_at, char_at, char_len, cp_bounds, encode_char};
use crate::types::NUL;
use core::ffi::CStr;

/// Largest glyph in bytes, including the terminating NUL.
pub const MAX_SCHAR_SIZE: c_int = 32;

/// Room for any glyph and its NUL: [`MAX_SCHAR_SIZE`] plus one.
///
/// [`schar_from_buf`] refuses a glyph of `MAX_SCHAR_SIZE` bytes in a debug
/// build only, as upstream's `assert()` does, so a release build can intern
/// one -- and then its NUL needs the byte past `MAX_SCHAR_SIZE`. Upstream's
/// scratch buffers are `MAX_SCHAR_SIZE` long and would be overrun by one.
pub(crate) const SCHAR_BUF_SIZE: usize = MAX_SCHAR_SIZE as usize + 1;

/// A scratch buffer a glyph is copied out into.
pub(crate) type SCharBuf = [u8; SCHAR_BUF_SIZE];

/// The intern table for glyphs that do not fit inline. Indices are capped at
/// 24 bits by [`schar_from_buf`], which is what leaves room for the tag byte.
///
/// Interning can grow the key arena, so it is reached a call at a time and
/// a glyph read out of it is copied or measured inside the borrow.
static GLYPH_CACHE: GlobalCell<Set_glyph> = GlobalCell::new(SET_INIT);

/// `f` over the interned glyph at byte offset `idx`.
#[inline]
fn with_glyph<R>(idx: uint32_t, f: impl FnOnce(&[u8]) -> R) -> R {
    GLYPH_CACHE.with(|cache| f(cache.glyph(idx)))
}

const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0,
    size: 0,
    n_occupied: 0,
    upper_bound: 0,
    n_keys: 0,
    keys_capacity: 0,
    hash: ::core::ptr::null_mut(),
};
const SET_INIT: Set_glyph = Set_glyph {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut(),
};

/// Whether `sc` is an index into the glyph cache rather than inline bytes.
#[inline(always)]
pub fn schar_high(sc: ScreenChar) -> bool {
    sc & 0xff == 0xff
}

/// The glyph-cache index of a high `ScreenChar`.
#[inline(always)]
fn schar_idx(sc: ScreenChar) -> uint32_t {
    sc >> 8
}

/// The bytes an inline `ScreenChar` holds: up to four, NUL-padded.
#[inline(always)]
fn inline_bytes(sc: ScreenChar) -> [u8; 4] {
    sc.to_ne_bytes()
}

/// How many of an inline glyph's four bytes are text: `strnlen(&sc, 4)`.
#[inline(always)]
fn inline_len(bytes: &[u8; 4]) -> usize {
    bytes.iter().position(|&b| b == 0).unwrap_or(4)
}

/// A one-byte ASCII glyph.
#[inline(always)]
pub const fn schar_from_ascii(c: u8) -> ScreenChar {
    c as ScreenChar
}

/// The glyph for the whole of `str`.
pub fn schar_from_str(str: &CStr) -> ScreenChar {
    schar_from_buf(str.to_bytes())
}

/// Intern `buf` as a glyph.
///
/// `buf` may not contain NULs, and must be shorter than [`MAX_SCHAR_SIZE`]
/// -- shorter, not as long, because the cache needs room for a terminator.
/// That bound is checked in a debug build only, as upstream's `assert()` is
/// (`v0.12.4:src/nvim/grid.c:85`); see [`SCHAR_BUF_SIZE`] for what a release
/// build does with a longer one.
///
/// The short form is inline and the cache lookup is not: `win_line` reaches
/// this once per cell, and whether the inliner kept it whole there used to
/// swing with edits nowhere near the screen code (scrbench +1.2 %).
#[inline]
pub fn schar_from_buf(buf: &[u8]) -> ScreenChar {
    let len = buf.len();
    debug_assert!(len < MAX_SCHAR_SIZE as size_t, "len < MAX_SCHAR_SIZE");
    if len <= 4 {
        let mut bytes = [0u8; 4];
        bytes[..len].copy_from_slice(buf);
        return ScreenChar::from_ne_bytes(bytes);
    }
    schar_from_cache(buf)
}

/// [`schar_from_buf`]'s long form: intern the glyph in the cache.
#[inline(never)]
fn schar_from_cache(buf: &[u8]) -> ScreenChar {
    let idx = GLYPH_CACHE.with_mut(|cache| cache.intern(buf));
    debug_assert!(idx < 0xffffff, "idx < 0xFFFFFF");
    0xff_u32.wrapping_add(idx << 8)
}

/// Empty the cache when it is close to exhausting the 24-bit index space.
///
/// Normally only called from `update_screen`. A true answer means every
/// screen buffer now holds stale indices and the caller must `UPD_CLEAR`.
pub fn schar_cache_clear_if_full() -> bool {
    // The real ceiling is (1<<24)-1; this leaves margin until the next
    // update_screen.
    if GLYPH_CACHE.with(|cache| cache.packed_len()) > (1 << 21) {
        schar_cache_clear();
        return true;
    }
    false
}

/// Empty the glyph cache.
///
/// Every high `ScreenChar` still held becomes stale: it reads as whatever
/// the new cache has at its offset, or as the empty glyph past its end --
/// never as memory outside the cache.
pub fn schar_cache_clear() {
    decor_check_invalid_glyphs();
    GLYPH_CACHE.with_mut(|cache| cache.clear());

    // The char options kept their original strings, so their parsed
    // ScreenChar values can be regenerated against the clean cache. Cell
    // widths have not changed, so this cannot fail.
    if check_chars_options().is_err() {
        std::process::abort();
    }
}

/// Write `sc` to `buf_out` and terminate it, answering the byte length.
///
/// The C harness's spelling (`test/unit/fixtures/vterm_test.c`,
/// `vterm_spec.lua`, the battery's `vtsweep`); Rust copies a glyph out with
/// [`schar_copy`] or [`schar_text`].
///
/// # Safety
/// `buf_out` must have room for [`MAX_SCHAR_SIZE`] bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn schar_get(buf_out: *mut c_char, sc: ScreenChar) -> size_t {
    let mut buf: SCharBuf = [0; SCHAR_BUF_SIZE];
    let len = schar_copy(&mut buf, sc);
    // SAFETY: the caller's room, which holds any glyph this process could
    // intern in a debug build, and its terminator.
    unsafe { core::ptr::copy_nonoverlapping(buf.as_ptr().cast::<c_char>(), buf_out, len + 1) };
    len
}

/// Copy `sc` into `buf` and terminate it, answering the byte length.
pub(crate) fn schar_copy(buf: &mut SCharBuf, sc: ScreenChar) -> size_t {
    let mut rest: &mut [u8] = buf;
    let len = schar_get_adv(&mut rest, sc);
    rest[0] = NUL as u8;
    len
}

/// The bytes of `sc`, without a terminator: [`schar_copy`] into `buf`,
/// answered as the part of it they fill.
pub(crate) fn schar_text(sc: ScreenChar, buf: &mut SCharBuf) -> &[u8] {
    let len = schar_copy(buf, sc);
    &buf[..len]
}

/// [`schar_text`] into an owned buffer, for a caller that keeps the text.
pub(crate) fn schar_bytes(sc: ScreenChar) -> Vec<u8> {
    schar_text(sc, &mut [0; SCHAR_BUF_SIZE]).to_vec()
}

/// Copy `sc` to the front of `buf_out` and advance it past the bytes,
/// writing no terminator -- for building a string out of several glyphs.
/// Answers the byte length.
///
/// # Panics
/// When `buf_out` has less room than the glyph needs. Upstream writes up to
/// [`MAX_SCHAR_SIZE`] bytes wherever it is pointed.
#[inline(always)]
pub fn schar_get_adv(buf_out: &mut &mut [u8], sc: ScreenChar) -> size_t {
    if schar_high(sc) {
        return schar_get_adv_cached(buf_out, sc);
    }
    // At most four bytes, copied one by one: a variable-length
    // `copy_from_slice` is a `memcpy` call per cell.
    let bytes = inline_bytes(sc);
    let len = inline_len(&bytes);
    let out = core::mem::take(buf_out);
    for (dst, &byte) in out[..len].iter_mut().zip(&bytes) {
        *dst = byte;
    }
    *buf_out = &mut out[len..];
    len
}

/// [`schar_get_adv`] into `buf` at `at`, for a buffer whose bytes past the
/// glyph are scratch: an inline glyph is stored as its four bytes at once --
/// the ones past its end are the glyph's NUL padding, which the caller
/// overwrites or never reads -- so a cell costs no copy. Answers the glyph's
/// byte length.
#[inline(always)]
pub(crate) fn schar_put_scratch(buf: &mut [u8], at: usize, sc: ScreenChar) -> size_t {
    if !schar_high(sc)
        && let Some(dst) = buf.get_mut(at..at + 4)
    {
        let bytes = inline_bytes(sc);
        dst.copy_from_slice(&bytes);
        return inline_len(&bytes);
    }
    schar_get_adv(&mut &mut buf[at..], sc)
}

/// [`schar_get_adv`] of a glyph in the cache.
#[inline(never)]
fn schar_get_adv_cached(buf_out: &mut &mut [u8], sc: ScreenChar) -> size_t {
    let out = core::mem::take(buf_out);
    let len = with_glyph(schar_idx(sc), |glyph| {
        out[..glyph.len()].copy_from_slice(glyph);
        glyph.len()
    });
    *buf_out = &mut out[len..];
    len
}

/// Byte length of `sc`.
pub fn schar_len(sc: ScreenChar) -> size_t {
    if schar_high(sc) {
        with_glyph(schar_idx(sc), <[u8]>::len)
    } else {
        inline_len(&inline_bytes(sc))
    }
}

/// Screen cells `sc` occupies.
#[inline]
pub fn schar_cells(sc: ScreenChar) -> c_int {
    // Hot path: anything below 0x80 is one inline ASCII byte.
    if sc < 0x80 {
        return 1;
    }
    if schar_high(sc) {
        with_glyph(schar_idx(sc), c_cells_at)
    } else {
        c_cells_at(&inline_bytes(sc))
    }
}

/// First raw UTF-8 byte of `sc`, NUL for the empty glyph.
fn schar_get_first_byte(sc: ScreenChar) -> u8 {
    if schar_high(sc) {
        with_glyph(schar_idx(sc), |glyph| glyph.first().copied().unwrap_or(0))
    } else {
        inline_bytes(sc)[0]
    }
}

/// The first codepoint of `sc`, NUL for the empty glyph.
#[inline]
pub fn schar_get_first_codepoint(sc: ScreenChar) -> c_int {
    // Hot path: an inline ASCII glyph is its own codepoint.
    if sc < 0x80 {
        return sc as c_int;
    }
    if schar_high(sc) {
        with_glyph(schar_idx(sc), char_at)
    } else {
        let bytes = inline_bytes(sc);
        char_at(&bytes[..inline_len(&bytes)])
    }
}

/// The ASCII character `sc` is, or NUL when it is not ASCII.
pub fn schar_get_ascii(sc: ScreenChar) -> c_char {
    if sc < 0x80 {
        sc as c_char
    } else {
        NUL as c_char
    }
}

/// Whether `sc` starts in the Arabic block, i.e. its lead byte is 0xD8 or
/// 0xD9 -- the cheap test [`line_do_arabic_shape`] uses to skip past text
/// that cannot need shaping.
fn schar_in_arabic_block(sc: ScreenChar) -> bool {
    schar_get_first_byte(sc) & 0xfe == 0xd8
}

/// The first two codepoints of `glyph`, or NUL where there is no such
/// codepoint. Arabic shaping needs the base character and any combining mark
/// that follows it.
fn first_two_codepoints(glyph: &[u8]) -> (c_int, c_int) {
    let c0 = char_at(glyph);
    if c0 == NUL as c_int {
        return (c0, NUL as c_int);
    }
    (c0, char_at(&glyph[char_len(glyph)..]))
}

/// [`first_two_codepoints`] of the glyph `sc`.
fn schar_get_first_two_codepoints(sc: ScreenChar) -> (c_int, c_int) {
    let mut buf: SCharBuf = [0; SCHAR_BUF_SIZE];
    first_two_codepoints(schar_text(sc, &mut buf))
}

/// Whether `c` is in the Arabic block proper (U+0600..U+06FF).
fn is_arabic_char(c: c_int) -> bool {
    c & 0xff00 == 0x600
}

/// Replace each Arabic character in `buf` with its contextual form, which
/// depends on the characters either side of it.
pub fn line_do_arabic_shape(buf: &mut [ScreenChar]) {
    // Quickly skip over non-Arabic text.
    let Some(start) = buf.iter().position(|&c| schar_in_arabic_block(c)) else {
        return;
    };

    let mut c0prev = 0;
    let (mut c0, mut c1) = schar_get_first_two_codepoints(buf[start]);

    for i in start..buf.len() {
        let next = if i + 1 < buf.len() { buf[i + 1] } else { 0 };
        let (c0next, c1next) = schar_get_first_two_codepoints(next);

        if is_arabic_char(c0) {
            let mut c1new = c1;
            let c0new = arabic_shape(c0, &mut c1new, c0next, c1next, c0prev);
            if c0new != c0 || c1new != c1 {
                buf[i] = reshape(buf[i], c0, c1, c0new, c1new);
            }
        }

        c0prev = c0;
        c0 = c0next;
        c1 = c1next;
    }
}

/// Rebuild the glyph `sc` with its leading one or two codepoints `c0`/`c1`
/// replaced by `c0new`/`c1new`, keeping whatever combining characters
/// followed them.
fn reshape(sc: ScreenChar, c0: c_int, c1: c_int, c0new: c_int, c1new: c_int) -> ScreenChar {
    let mut old: SCharBuf = [0; SCHAR_BUF_SIZE];
    let old = schar_text(sc, &mut old);
    let mut new: SCharBuf = [0; SCHAR_BUF_SIZE];

    let mut len = encode_char(c0new, &mut new);
    if c1new != 0 {
        len += encode_char(c1new, &mut new[len..]);
    }

    // Whatever followed the one or two codepoints just replaced.
    let off = utf_char2len(c0) + if c1 != 0 { utf_char2len(c1) } else { 0 };
    let tail = old.get(off as usize..).unwrap_or(&[]);
    let mut rest = tail.len();
    if rest + len + 1 > MAX_SCHAR_SIZE as size_t {
        // Too bigly: discard one codepoint. That is always enough,
        // because c0 cannot grow by more than two bytes (base Arabic to
        // extended Arabic).
        rest -= cp_bounds(tail, rest - 1).begin_off as size_t + 1;
    }
    new[len..len + rest].copy_from_slice(&tail[..rest]);
    schar_from_buf(&new[..len + rest])
}

/// Put a Unicode codepoint in a screen cell.
pub fn schar_from_char(mut c: c_int) -> ScreenChar {
    if c >= 0x200000 {
        // This must NEVER happen, even for a file holding overlong sequences.
        c = 0xfffd;
    }
    // A codepoint below 0x200000 encodes to at most four bytes; NUL to the
    // one zero byte, which is the empty glyph.
    let mut bytes = [0u8; 4];
    encode_char(c, &mut bytes);
    ScreenChar::from_ne_bytes(bytes)
}
