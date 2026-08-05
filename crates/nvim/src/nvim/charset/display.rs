//! Rendering text for display: the width a character occupies on screen and
//! the `^X` / `<xx>` forms that stand in for one that cannot be shown.

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::garray::{ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curbuf, dy_flags};
use crate::src::nvim::mbyte::{
    mb_tolower, utf_char2bytes, utf_char2cells, utf_char2len, utf_ptr2cells, utf_ptr2char,
    utf_ptr2len, utfc_ptr2len,
};
use crate::src::nvim::memory::{xmalloc, xrealloc};
use crate::src::nvim::option::get_fileformat;
use crate::src::nvim::os::libc::{memmove, strlen};
use crate::src::nvim::types::{StringBuilder, buf_T, garray_T, size_t, ssize_t, uint8_t};

use super::transchar as render;
use super::{
    CT_CELL_MASK, EOL_MAC, NL, NUL, TAB, chartab_initialized, g_chartab, kOptDyFlagUhex,
    vim_isprintc,
};
use crate::src::nvim::keycodes::K_SPECIAL;
use crate::src::nvim::pos::MAXCOL;

const CAR: c_int = 13;
const KS_ZERO: c_int = 255;
const KS_SPECIAL: c_int = 254;

/// Replace every unprintable byte in `buf` with its display form, in place.
///
/// Stops early rather than overflowing when the expansions no longer fit in
/// `bufsize`, leaving the tail untranslated.
///
/// # Safety
/// `buf` must be a NUL-terminated string in a buffer of `bufsize` bytes.
pub unsafe fn trans_characters(mut buf: *mut c_char, bufsize: c_int) {
    let mut len = strlen(buf) as c_int;
    let mut room = bufsize - len;
    while *buf != 0 {
        let mut trs_len = utfc_ptr2len(buf);
        if trs_len > 1 {
            // A multibyte character is left alone.
            len -= trs_len;
        } else {
            let trs = transchar_byte(*buf as uint8_t as c_int);
            trs_len = strlen(trs) as c_int;
            if trs_len > 1 {
                room -= trs_len - 1;
                if room <= 0 {
                    return;
                }
                memmove(
                    buf.offset(trs_len as isize) as *mut c_void,
                    buf.offset(1) as *const c_void,
                    len as size_t,
                );
            }
            memmove(buf as *mut c_void, trs as *const c_void, trs_len as size_t);
            len -= 1;
        }
        buf = buf.offset(trs_len as isize);
    }
}

/// How many cells [`transstr_buf`] would need for `s`, excluding the NUL.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn transstr_len(s: *const c_char, untab: bool) -> size_t {
    let mut p = s;
    let mut len: size_t = 0;
    while *p != 0 {
        let l = utfc_ptr2len(p) as size_t;
        if l > 1 {
            if vim_isprintc(utf_ptr2char(p)) {
                len += l;
            } else {
                // An unprintable multibyte character is spelled out one
                // codepoint at a time, composing characters included.
                let mut off: size_t = 0;
                while off < l {
                    len += render::hex_form(utf_ptr2char(p.add(off))).len;
                    off += utf_ptr2len(p.add(off)) as size_t;
                }
            }
            p = p.add(l);
        } else if *p as c_int == TAB && !untab {
            len += 1;
            p = p.offset(1);
        } else {
            let cells = byte2cells(*p as uint8_t as c_int);
            p = p.offset(1);
            // A zero width means the table has no entry: `<xx>` is four.
            len += if cells > 0 { cells } else { 4 } as size_t;
        }
    }
    len
}

/// Copy `s` into `buf` with every unprintable character replaced by its
/// display form, NUL-terminated. Answers the number of bytes written.
///
/// A negative `slen` means "to the NUL". The output is truncated at a
/// character boundary rather than overrunning `buflen`.
///
/// # Safety
/// `s` must hold `slen` readable bytes (or be NUL-terminated) and `buf` must
/// have room for `buflen`.
pub unsafe fn transstr_buf(
    s: *const c_char,
    slen: ssize_t,
    buf: *mut c_char,
    buflen: size_t,
    untab: bool,
) -> size_t {
    let mut p = s;
    let mut buf_p = buf;
    let buf_e = buf.add(buflen).offset(-1);
    while (slen < 0 || (p.addr() - s.addr()) < slen as usize) && *p as c_int != NUL && buf_p < buf_e
    {
        let l = utfc_ptr2len(p) as size_t;
        if l > 1 {
            if buf_p.add(l) > buf_e {
                break;
            }
            if vim_isprintc(utf_ptr2char(p)) {
                memmove(buf_p as *mut c_void, p as *const c_void, l);
                buf_p = buf_p.add(l);
            } else {
                let mut off: size_t = 0;
                while off < l {
                    let hex = render::hex_form(utf_ptr2char(p.add(off)));
                    if buf_p.add(hex.len) > buf_e {
                        break;
                    }
                    memmove(
                        buf_p as *mut c_void,
                        hex.bytes.as_ptr() as *const c_void,
                        hex.len,
                    );
                    buf_p = buf_p.add(hex.len);
                    off += utf_ptr2len(p.add(off)) as size_t;
                }
            }
            p = p.add(l);
        } else if *p as c_int == TAB && !untab {
            *buf_p = *p;
            buf_p = buf_p.offset(1);
            p = p.offset(1);
        } else {
            let tb = transchar_byte(*p as uint8_t as c_int);
            p = p.offset(1);
            let tb_len = strlen(tb);
            if buf_p.add(tb_len) > buf_e {
                break;
            }
            memmove(buf_p as *mut c_void, tb as *const c_void, tb_len);
            buf_p = buf_p.add(tb_len);
        }
    }
    *buf_p = NUL as c_char;
    debug_assert!(buf_p <= buf_e, "buf_p <= buf_e");
    buf_p.addr() - buf.addr()
}

/// [`transstr_buf`] into a freshly allocated string the caller owns.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn transstr(s: *const c_char, untab: bool) -> *mut c_char {
    let len = transstr_len(s, untab) + 1;
    let buf = xmalloc(len) as *mut c_char;
    transstr_buf(s, -1, buf, len, untab);
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
    let len = transstr_len(s, untab);
    let needed = (*str).size + len + 1;
    if (*str).capacity < needed {
        (*str).capacity = needed.next_power_of_two();
        (*str).items = xrealloc((*str).items as *mut c_void, (*str).capacity) as *mut c_char;
    }
    transstr_buf(s, -1, (*str).items.add((*str).size), len + 1, untab);
    (*str).size += len;
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
    let mut ga: garray_T = ::core::mem::zeroed();
    let mut len = orglen;
    if buf.is_null() {
        ga_init(&raw mut ga, 1, 10);
        ga_grow(&raw mut ga, len + 1);
        memmove(ga.ga_data, str as *const c_void, len as size_t);
        ga.ga_len = len;
    } else {
        if len >= buflen {
            len = buflen - 1;
        }
        memmove(buf as *mut c_void, str as *const c_void, len as size_t);
    }

    // From here on `at(i)` is the one place that knows which buffer is in
    // play; `ga.ga_data` moves under us whenever the collection grows.
    let at = |ga: &garray_T, i: c_int| -> *mut c_char {
        if buf.is_null() {
            (ga.ga_data as *mut c_char).offset(i as isize)
        } else {
            buf.offset(i as isize)
        }
    };
    *at(&ga, len) = NUL as c_char;

    let mut i: c_int = 0;
    while *at(&ga, i) as c_int != NUL {
        let c = utf_ptr2char(at(&ga, i));
        let olen = utf_ptr2len(at(&ga, i));
        let mut lc = mb_tolower(c);
        // Only ASCII and real multibyte characters fold; a lone Latin-1 byte
        // has no case in this encoding.
        if (c < 0x80 || olen > 1) && c != lc {
            let mut nlen = utf_char2len(lc);
            if olen != nlen {
                if nlen > olen {
                    if buf.is_null() {
                        ga_grow(&raw mut ga, nlen - olen + 1);
                    } else if len + nlen - olen >= buflen {
                        // No room to grow: keep the original character.
                        lc = c;
                        nlen = olen;
                    }
                }
                if olen != nlen {
                    let src = at(&ga, i).offset(olen as isize);
                    memmove(
                        at(&ga, i).offset(nlen as isize) as *mut c_void,
                        src as *const c_void,
                        strlen(src) + 1,
                    );
                    if buf.is_null() {
                        ga.ga_len += nlen - olen;
                    } else {
                        len += nlen - olen;
                    }
                }
            }
            utf_char2bytes(lc, at(&ga, i));
        }
        i += utfc_ptr2len(at(&ga, i));
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

/// Copy a rendering, and the NUL after it, to `dst`.
///
/// # Safety
/// `dst` must have room for `rendered.len + 1` bytes.
unsafe fn write_rendered(dst: *mut c_char, rendered: &render::Rendered) {
    ptr::copy_nonoverlapping(rendered.bytes.as_ptr(), dst as *mut u8, rendered.len + 1);
}

/// The display form of character `c`, in a shared buffer.
///
/// # Safety
/// The current buffer must be valid.
pub unsafe fn transchar(c: c_int) -> *mut c_char {
    transchar_buf(curbuf.get(), c)
}

/// The display form of `c` as it would appear in `buf` (which decides how a
/// carriage return renders), in a shared buffer.
///
/// # Safety
/// `buf` may be null; otherwise it must be a valid buffer.
pub unsafe fn transchar_buf(buf: *const buf_T, c: c_int) -> *mut c_char {
    let mut c = c;
    let mut i = 0usize;
    if c < 0 {
        // A negative code is one of the key-translation escapes.
        let (prefix, byte) = render::negative_form(c);
        ptr::copy_nonoverlapping(prefix.as_ptr(), transchar_charbuf.ptr() as *mut u8, 2);
        i = 2;
        c = byte;
    }
    // Before the tables exist, printable ASCII is all that can be trusted.
    if (!chartab_initialized.get() && (c >= ' ' as c_int && c <= '~' as c_int))
        || (c <= 0xff && vim_isprintc(c))
    {
        let charbuf = transchar_charbuf.ptr() as *mut uint8_t;
        *charbuf.add(i) = c as uint8_t;
        *charbuf.add(i + 1) = NUL as uint8_t;
    } else if c <= 0xff {
        transchar_nonprint(buf, (transchar_charbuf.ptr() as *mut c_char).add(i), c);
    } else {
        write_rendered(
            (transchar_charbuf.ptr() as *mut c_char).add(i),
            &render::hex_form(c),
        );
    }
    transchar_charbuf.ptr() as *mut c_char
}

/// The display form of the single byte `c`. Unlike [`transchar_buf`] this
/// never treats a high byte as a printable Latin-1 character.
///
/// # Safety
/// `buf` may be null; otherwise it must be a valid buffer.
pub unsafe fn transchar_byte_buf(buf: *const buf_T, c: c_int) -> *mut c_char {
    if c >= 0x80 {
        transchar_nonprint(buf, transchar_charbuf.ptr() as *mut c_char, c);
        return transchar_charbuf.ptr() as *mut c_char;
    }
    transchar_buf(buf, c)
}

/// [`transchar_byte_buf`] for the current buffer.
///
/// # Safety
/// The current buffer must be valid.
pub unsafe fn transchar_byte(c: c_int) -> *mut c_char {
    transchar_byte_buf(curbuf.get(), c)
}

/// Write the display form of the unprintable byte `c` into `charbuf`.
///
/// # Safety
/// `charbuf` must have room for five bytes; `buf` may be null.
pub unsafe fn transchar_nonprint(buf: *const buf_T, charbuf: *mut c_char, c: c_int) {
    let mut c = c;
    if c == NL {
        // A NUL is stored as a newline internally.
        c = NUL;
    } else if !buf.is_null() && c == CAR && get_fileformat(buf) == EOL_MAC {
        c = NL;
    }
    debug_assert!(c <= 0xff, "c <= 0xff");
    let rendered = if dy_flags.get() & kOptDyFlagUhex != 0 || c > 0x7f {
        render::hex_form(c)
    } else {
        render::control_form(c)
    };
    write_rendered(charbuf, &rendered);
}

/// Write `c`'s `<xx>` form into `buf`. Answers the length, excluding the NUL.
///
/// # Safety
/// `buf` must have room for nine bytes.
pub unsafe fn transchar_hex(buf: *mut c_char, c: c_int) -> size_t {
    let rendered = render::hex_form(c);
    write_rendered(buf, &rendered);
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
    let mut p1 = str;
    let mut p2 = (if !end.is_null() {
        end
    } else {
        str.add(strlen(str))
    })
    .offset(-1);
    while p1 < p2 {
        ptr::swap(p1, p2);
        p1 = p1.offset(1);
        p2 = p2.offset(-1);
    }
}

/// The display width of the byte `b`, or zero for a multibyte lead byte.
///
/// # Safety
/// The global table must be initialised.
pub unsafe fn byte2cells(b: c_int) -> c_int {
    if b >= 0x80 {
        return 0;
    }
    (*g_chartab.ptr())[b as usize] as c_int & CT_CELL_MASK
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
        return char2cells(escaped) + 2;
    }
    if c >= 0x80 {
        return utf_char2cells(c);
    }
    (*g_chartab.ptr())[(c & 0xff) as usize] as c_int & CT_CELL_MASK
}

/// The display width of the character at `p`.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
pub unsafe fn ptr2cells(p_in: *const c_char) -> c_int {
    let p = p_in as *const uint8_t;
    if *p as c_int >= 0x80 {
        return utf_ptr2cells(p_in);
    }
    (*g_chartab.ptr())[*p as usize] as c_int & CT_CELL_MASK
}

/// The display width of the whole string `s`.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn vim_strsize(s: *const c_char) -> c_int {
    vim_strnsize(s, MAXCOL)
}

/// The display width of at most `len` bytes of `s`.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn vim_strnsize(s: *const c_char, len: c_int) -> c_int {
    assert!(!s.is_null(), "s != NULL");
    let mut s = s;
    let mut len = len;
    let mut size = 0;
    while *s as c_int != NUL && {
        len -= 1;
        len >= 0
    } {
        let l = utfc_ptr2len(s);
        size += ptr2cells(s);
        s = s.offset(l as isize);
        len -= l - 1;
    }
    size
}
