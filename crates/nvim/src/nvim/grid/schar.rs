#![deny(unsafe_op_in_unsafe_fn)]

//! `schar_T`: the glyph in a screen cell.
//!
//! A cell holds one grapheme cluster, which is usually short. A `schar_T` is
//! a `u32` that stores the UTF-8 bytes inline when there are at most four of
//! them, and otherwise an index into a global intern table (`glyph_cache`)
//! tagged by a low byte of `0xFF` -- which no UTF-8 lead byte can be, so the
//! two cases never collide. Zero means "right half of a double-width cell".
//!
//! Little-endian only; the original carries an `ORDER_BIG_ENDIAN` branch that
//! puts the tag in the high byte instead, and the transpile dropped it.

use super::*;

/// Largest glyph in bytes, including the terminating NUL.
pub const MAX_SCHAR_SIZE: c_int = 32;

/// The intern table for glyphs that do not fit inline. Indices are capped at
/// 24 bits by [`schar_from_buf`], which is what leaves room for the tag byte.
static GLYPH_CACHE: GlobalCell<Set_glyph> = GlobalCell::new(SET_INIT);

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
pub fn schar_high(sc: schar_T) -> bool {
    sc & 0xff == 0xff
}

/// The glyph-cache index of a high `schar_T`.
#[inline(always)]
fn schar_idx(sc: schar_T) -> uint32_t {
    sc >> 8
}

/// A one-byte ASCII glyph.
#[inline(always)]
pub const fn schar_from_ascii(c: u8) -> schar_T {
    c as schar_T
}

/// # Safety
/// `str` must be NUL-terminated or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn schar_from_str(str: *const c_char) -> schar_T {
    unsafe {
        if str.is_null() {
            return 0;
        }
        schar_from_buf(str, strlen(str))
    }
}

/// Intern `len` bytes of `buf` as a glyph.
///
/// # Safety
/// `buf` need not be NUL-terminated, but may not contain embedded NULs, and
/// `len` must be below [`MAX_SCHAR_SIZE`] -- below, not at, because the cache
/// needs room for a terminator. That bound is checked in a debug build only,
/// as upstream's `assert()` is (`v0.12.4:src/nvim/grid.c:85`).
pub unsafe fn schar_from_buf(buf: *const c_char, len: size_t) -> schar_T {
    unsafe {
        debug_assert!(len < MAX_SCHAR_SIZE as size_t, "len < MAX_SCHAR_SIZE");
        if len <= 4 {
            let mut sc: schar_T = 0;
            memcpy((&raw mut sc).cast::<c_void>(), buf.cast::<c_void>(), len);
            return sc;
        }

        let str = String_0 {
            data: buf as *mut c_char,
            size: len,
        };
        let mut status: MHPutStatus = kMHExisting;
        let idx = mh_put_glyph(GLYPH_CACHE.ptr(), str, &raw mut status);
        debug_assert!(idx < 0xffffff, "idx < 0xFFFFFF");
        0xff_u32.wrapping_add(idx << 8)
    }
}

/// Empty the cache when it is close to exhausting the 24-bit index space.
///
/// Normally only called from `update_screen`. A true answer means every
/// screen buffer now holds stale indices and the caller must `UPD_CLEAR`.
///
/// # Safety
/// Must be called with no screen update in progress.
pub unsafe fn schar_cache_clear_if_full() -> bool {
    unsafe {
        // The real ceiling is (1<<24)-1; this leaves margin until the next
        // update_screen.
        if (*GLYPH_CACHE.ptr()).h.n_keys > (1 << 21) {
            schar_cache_clear();
            return true;
        }
        false
    }
}

/// # Safety
/// Every live `schar_T` becomes meaningless; see [`schar_cache_clear_if_full`].
pub unsafe fn schar_cache_clear() {
    unsafe {
        decor_check_invalid_glyphs();
        mh_clear(&raw mut (*GLYPH_CACHE.ptr()).h);

        // The char options kept their original strings, so their parsed
        // schar_T values can be regenerated against the clean cache. Cell
        // widths have not changed, so this cannot fail.
        if !check_chars_options().is_null() {
            abort();
        }
    }
}

/// Write `sc` to `buf_out` and terminate it. Answers the byte length.
///
/// # Safety
/// `buf_out` must have room for [`MAX_SCHAR_SIZE`] bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn schar_get(mut buf_out: *mut c_char, sc: schar_T) -> size_t {
    unsafe {
        let len = schar_get_adv(&raw mut buf_out, sc);
        *buf_out = NUL;
        len
    }
}

/// Like [`schar_get`], but advances `*buf_out` past the bytes and writes no
/// terminator -- for building a string out of several glyphs.
///
/// # Safety
/// `*buf_out` must have room for [`MAX_SCHAR_SIZE`] bytes.
pub unsafe fn schar_get_adv(buf_out: *mut *mut c_char, sc: schar_T) -> size_t {
    unsafe {
        let (src, len) = if schar_high(sc) {
            let idx = schar_idx(sc);
            assert!(idx < (*GLYPH_CACHE.ptr()).h.n_keys, "idx < n_keys");
            let key = (*GLYPH_CACHE.ptr()).keys.add(idx as usize);
            (key.cast_const(), strlen(key))
        } else {
            let inline = (&raw const sc).cast::<c_char>();
            (inline, strnlen(inline, 4))
        };
        memcpy((*buf_out).cast::<c_void>(), src.cast::<c_void>(), len);
        *buf_out = (*buf_out).add(len);
        len
    }
}

/// Byte length of `sc`.
///
/// # Safety
/// `sc` must be a glyph this process produced.
pub unsafe fn schar_len(sc: schar_T) -> size_t {
    unsafe {
        if schar_high(sc) {
            let idx = schar_idx(sc);
            assert!(idx < (*GLYPH_CACHE.ptr()).h.n_keys, "idx < n_keys");
            strlen((*GLYPH_CACHE.ptr()).keys.add(idx as usize))
        } else {
            strnlen((&raw const sc).cast::<c_char>(), 4)
        }
    }
}

/// Screen cells `sc` occupies.
///
/// # Safety
/// `sc` must be a glyph this process produced.
pub unsafe fn schar_cells(sc: schar_T) -> c_int {
    unsafe {
        // Hot path: anything below 0x80 is one inline ASCII byte.
        if sc < 0x80 {
            return 1;
        }
        let mut sc_buf = [0 as c_char; MAX_SCHAR_SIZE as usize];
        schar_get(sc_buf.as_mut_ptr(), sc);
        utf_ptr2cells(sc_buf.as_ptr())
    }
}

/// First raw UTF-8 byte of `sc`.
///
/// # Safety
/// `sc` must be a glyph this process produced.
unsafe fn schar_get_first_byte(sc: schar_T) -> c_char {
    unsafe {
        assert!(
            !(schar_high(sc) && schar_idx(sc) >= (*GLYPH_CACHE.ptr()).h.n_keys),
            "!(schar_high(sc) && schar_idx(sc) >= glyph_cache.h.n_keys)"
        );
        if schar_high(sc) {
            *(*GLYPH_CACHE.ptr()).keys.add(schar_idx(sc) as usize)
        } else {
            *(&raw const sc).cast::<c_char>()
        }
    }
}

/// # Safety
/// `sc` must be a glyph this process produced.
pub unsafe fn schar_get_first_codepoint(sc: schar_T) -> c_int {
    unsafe {
        let mut sc_buf = [0 as c_char; MAX_SCHAR_SIZE as usize];
        schar_get(sc_buf.as_mut_ptr(), sc);
        utf_ptr2char(sc_buf.as_ptr())
    }
}

/// The ASCII character `sc` is, or NUL when it is not ASCII.
pub fn schar_get_ascii(sc: schar_T) -> c_char {
    if sc < 0x80 { sc as c_char } else { NUL }
}

/// Whether `sc` starts in the Arabic block, i.e. its lead byte is 0xD8 or
/// 0xD9 -- the cheap test [`line_do_arabic_shape`] uses to skip past text
/// that cannot need shaping.
///
/// # Safety
/// `sc` must be a glyph this process produced.
unsafe fn schar_in_arabic_block(sc: schar_T) -> bool {
    unsafe { (schar_get_first_byte(sc) as u8) & 0xfe == 0xd8 }
}

/// The first two codepoints of `sc`, or NUL where there is no such
/// codepoint. Arabic shaping needs the base character and any combining mark
/// that follows it.
///
/// # Safety
/// `sc` must be a glyph this process produced.
unsafe fn schar_get_first_two_codepoints(sc: schar_T) -> (c_int, c_int) {
    unsafe {
        let mut sc_buf = [0 as c_char; MAX_SCHAR_SIZE as usize];
        schar_get(sc_buf.as_mut_ptr(), sc);

        let c0 = utf_ptr2char(sc_buf.as_ptr());
        if c0 == NUL as c_int {
            return (c0, NUL as c_int);
        }
        let len = utf_ptr2len(sc_buf.as_ptr());
        (c0, utf_ptr2char(sc_buf.as_ptr().offset(len as isize)))
    }
}

/// Whether `c` is in the Arabic block proper (U+0600..U+06FF).
fn is_arabic_char(c: c_int) -> bool {
    c & 0xff00 == 0x600
}

/// Replace each Arabic character in `buf` with its contextual form, which
/// depends on the characters either side of it.
///
/// # Safety
/// `buf` must point to `cols` glyphs this process produced.
pub unsafe fn line_do_arabic_shape(buf: *mut schar_T, cols: c_int) {
    unsafe {
        // Quickly skip over non-Arabic text.
        let mut i = 0;
        while i < cols {
            if schar_in_arabic_block(*buf.offset(i as isize)) {
                break;
            }
            i += 1;
        }
        if i == cols {
            return;
        }

        let mut c0prev = 0;
        let (mut c0, mut c1) = schar_get_first_two_codepoints(*buf.offset(i as isize));

        while i < cols {
            let next = if i + 1 < cols {
                *buf.offset((i + 1) as isize)
            } else {
                0
            };
            let (c0next, c1next) = schar_get_first_two_codepoints(next);

            if is_arabic_char(c0) {
                let mut c1new = c1;
                let c0new = arabic_shape(c0, &mut c1new, c0next, c1next, c0prev);
                if c0new != c0 || c1new != c1 {
                    *buf.offset(i as isize) =
                        reshape(*buf.offset(i as isize), c0, c1, c0new, c1new);
                }
            }

            c0prev = c0;
            c0 = c0next;
            c1 = c1next;
            i += 1;
        }
    }
}

/// Rebuild the glyph `sc` with its leading one or two codepoints replaced by
/// `c0new`/`c1new`, keeping whatever combining characters followed them.
///
/// # Safety
/// `sc` must be a glyph this process produced whose first two codepoints are
/// `c0` and `c1`.
unsafe fn reshape(sc: schar_T, c0: c_int, c1: c_int, c0new: c_int, c1new: c_int) -> schar_T {
    unsafe {
        let mut old = [0 as c_char; MAX_SCHAR_SIZE as usize];
        schar_get(old.as_mut_ptr(), sc);

        let mut new = [0 as c_char; MAX_SCHAR_SIZE as usize];
        let mut len = utf_char2bytes(c0new, new.as_mut_ptr()) as size_t;
        if c1new != 0 {
            len += utf_char2bytes(c1new, new.as_mut_ptr().add(len)) as size_t;
        }

        // Whatever followed the one or two codepoints we just replaced.
        let off = utf_char2len(c0) + if c1 != 0 { utf_char2len(c1) } else { 0 };
        let tail = old.as_ptr().offset(off as isize);
        let mut rest = strlen(tail);
        if rest + len + 1 > MAX_SCHAR_SIZE as size_t {
            // Too bigly: discard one codepoint. That is always enough,
            // because c0 cannot grow by more than two bytes (base Arabic to
            // extended Arabic).
            rest -= utf_cp_bounds(tail, tail.add(rest - 1)).begin_off as size_t + 1;
        }
        memcpy(
            new.as_mut_ptr().add(len).cast::<c_void>(),
            tail.cast::<c_void>(),
            rest,
        );
        schar_from_buf(new.as_ptr(), len + rest)
    }
}

/// Put a Unicode codepoint in a screen cell.
pub fn schar_from_char(mut c: c_int) -> schar_T {
    let mut sc: schar_T = 0;
    if c >= 0x200000 {
        // This must NEVER happen, even for a file holding overlong sequences.
        c = 0xfffd;
    }
    // SAFETY: a codepoint below 0x200000 encodes to at most four bytes.
    unsafe { utf_char2bytes(c, (&raw mut sc).cast::<c_char>()) };
    sc
}
