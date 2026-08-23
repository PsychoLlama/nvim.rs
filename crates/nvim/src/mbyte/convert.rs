//! Converting text between encodings.
//!
//! [`convert_setup`] works out *how* to get from one encoding to another and
//! records it in a `vimconv_T`; [`string_convert_ext`] runs that plan over a
//! string. Four pairs are done here directly — Latin-1 and Latin-9 to and
//! from UTF-8 — because they are the common cases and are pure arithmetic.
//! Everything else goes through iconv.
//!
//! **iconv is a real foreign boundary and the `unsafe` in this file is
//! genuine.** It is a stateful C library reached through an opaque
//! descriptor, it writes into caller-owned buffers whose remaining room it
//! reports back by decrementing a counter, and it signals every one of its
//! outcomes through `errno`. None of that has a safe Rust shape.
//! [`my_iconv_open`] also has to *probe* the host's iconv, because a
//! dynamically loaded one can accept an `iconv_open` and then do nothing.
//!
//! A conversion that cannot represent a character does not fail by default:
//! it substitutes `?` (or `¿` on the way to Latin-1) and carries on, unless
//! `vc_fail` asks for an error instead.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, OK};
use ::libc::{EILSEQ, EINVAL};
use core::ffi::{c_char, c_int, c_uint, c_void};

/// Has the host's iconv been proved to work?
///
/// A dynamically loaded iconv can accept `iconv_open` and then convert
/// nothing, so the first successfully opened descriptor is probed before it
/// is trusted. The answer is remembered for the process.
type WorkingStatus = c_uint;
const kUnknown: WorkingStatus = 0;
const kWorking: WorkingStatus = 1;
const kBroken: WorkingStatus = 2;

/// The scratch buffer `tv_get_string_buf` renders a non-string argument into.
const NUMBUFLEN: usize = 65;

/// How many bytes the probe conversion is given to write into.
const ICONV_TESTLEN: usize = 400;

/// iconv's own `errno` values, which a dynamically loaded library may report
/// instead of the host's — both spellings are tested at every site. They are
/// spelled out rather than derived from `libc`'s: on this platform the two
/// happen to agree, but the point of the pair is that elsewhere they need not.
pub const ICONV_E2BIG: c_int = 7;
pub const ICONV_EINVAL: c_int = 22;
pub const ICONV_EILSEQ: c_int = 84;

/// `(iconv_t)-1`: what `iconv_open` answers when it cannot convert a pair.
fn iconv_failed() -> iconv_t {
    core::ptr::with_exposed_provenance_mut(-1i32 as usize)
}

/// Open an iconv descriptor from `from` to `to`, or [`iconv_failed`].
///
/// The first descriptor this ever opens is *probed* — a conversion with no
/// input, which a working iconv answers by leaving the output pointer alone
/// and a broken one by nulling it. Once iconv is known broken this never
/// tries again.
///
/// # Safety
///
/// Both names must be NUL-terminated strings.
pub unsafe fn my_iconv_open(to: *mut c_char, from: *mut c_char) -> iconv_t {
    unsafe {
        static iconv_working: GlobalCell<WorkingStatus> = GlobalCell::new(kUnknown);
        if iconv_working.get() == kBroken {
            return iconv_failed();
        }

        let mut fd = iconv_open(enc_skip(to), enc_skip(from));
        if fd == iconv_failed() || iconv_working.get() != kUnknown {
            return fd;
        }

        let mut tobuf = [0 as c_char; ICONV_TESTLEN];
        let mut p = tobuf.as_mut_ptr();
        let mut tolen: size_t = ICONV_TESTLEN;
        iconv(
            fd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut p,
            &raw mut tolen,
        );
        if p.is_null() {
            iconv_working.set(kBroken);
            iconv_close(fd);
            fd = iconv_failed();
        } else {
            iconv_working.set(kWorking);
        }
        fd
    }
}

/// Convert `str[..slen]` through `vcp`'s open iconv descriptor.
///
/// The output buffer is grown and the conversion resumed whenever iconv runs
/// out of room, which is what the `E2BIG` arm is for: it is not an error.
/// `done` is how much of the result is already final, so a reallocation can
/// copy it forward and pick up where it left off.
///
/// # Safety
///
/// `vcp.vc_fd` must be an open descriptor and `str` must have `slen` readable
/// bytes. The result is `xmalloc`'d, or null when the conversion failed.
unsafe fn iconv_string(
    vcp: *const vimconv_T,
    str: *const c_char,
    slen: size_t,
    unconvlenp: *mut size_t,
    resultlenp: *mut size_t,
) -> *mut c_char {
    unsafe {
        // iconv reports every outcome through `errno`, read straight after
        // the call that set it. A closure, so it inherits this block.
        let errno = || *__errno_location();

        let fail = (*vcp).vc_fail;
        let mut result: *mut c_char = core::ptr::null_mut();
        let mut to: *mut c_char = core::ptr::null_mut();
        let mut len: size_t = 0;
        let mut done: size_t = 0;
        let mut from = str;
        let mut fromlen = slen;

        loop {
            if len == 0 || errno() == ICONV_E2BIG {
                // Enough for most conversions; on a retry, more than last time.
                len = len + fromlen * 2 + 40;
                let p = xmalloc(len) as *mut c_char;
                if done > 0 {
                    memmove(p as *mut c_void, result as *const c_void, done);
                }
                xfree(result as *mut c_void);
                result = p;
            }

            to = result.add(done);
            // Two bytes held back: the NUL, and the second `?` an
            // unconvertible wide character can need.
            let mut tolen = len - done - 2;
            if iconv(
                (*vcp).vc_fd,
                &raw mut from as *mut c_void as *mut *mut c_char,
                &raw mut fromlen,
                &raw mut to,
                &raw mut tolen,
            ) != SIZE_MAX as size_t
            {
                *to = 0; // finished
                break;
            }

            let e = errno();
            let incomplete = e == ICONV_EINVAL || e == EINVAL;
            let illegal = e == ICONV_EILSEQ || e == EILSEQ;
            if !fail && incomplete && !unconvlenp.is_null() {
                // A sequence cut off at the end: hand back how much is left
                // rather than treating it as bad input.
                *to = 0;
                *unconvlenp = fromlen;
                break;
            } else if !fail && (illegal || incomplete) {
                // Cannot convert: emit `?` and skip one character. This
                // assumes the input is 'encoding'; nothing else would tell us
                // how much to skip.
                *to = b'?' as c_char;
                to = to.offset(1);
                if utf_ptr2cells(from) > 1 {
                    *to = b'?' as c_char;
                    to = to.offset(1);
                }
                let l = utfc_ptr2len_len(from, fromlen as c_int);
                from = from.add(l as usize);
                fromlen -= l as size_t;
            } else if e != ICONV_E2BIG {
                xfree(result as *mut c_void);
                result = core::ptr::null_mut();
                break;
            }
            done = to.offset_from(result) as size_t;
        }

        if !resultlenp.is_null() && !result.is_null() {
            *resultlenp = to.offset_from(result) as size_t;
        }
        result
    }
}

/// `iconv({string}, {from}, {to})`.
pub unsafe fn f_iconv(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = core::ptr::null_mut();

        let str = tv_get_string(argvars);
        let mut buf1 = [0 as c_char; NUMBUFLEN];
        let from = enc_canonize(enc_skip(
            tv_get_string_buf(argvars.add(1), buf1.as_mut_ptr()).cast_mut(),
        ));
        let mut buf2 = [0 as c_char; NUMBUFLEN];
        let to = enc_canonize(enc_skip(
            tv_get_string_buf(argvars.add(2), buf2.as_mut_ptr()).cast_mut(),
        ));

        let mut vimconv = CONV_NONE_INIT;
        convert_setup(&raw mut vimconv, from, to);
        (*rettv).vval.v_string = if vimconv.vc_type == CONV_NONE {
            // Same encoding both ways: hand back a copy unchanged.
            xstrdup(str)
        } else {
            string_convert(&raw mut vimconv, str.cast_mut(), core::ptr::null_mut())
        };

        // Closes the descriptor.
        convert_setup(
            &raw mut vimconv,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        xfree(from as *mut c_void);
        xfree(to as *mut c_void);
    }
}

/// Plan a conversion from `from` to `to`, replacing whatever `vcp` held.
///
/// # Safety
///
/// `vcp` must be writable and hold either a valid plan or zeroed memory;
/// `from` and `to` must be null or NUL-terminated.
pub unsafe fn convert_setup(vcp: *mut vimconv_T, from: *mut c_char, to: *mut c_char) -> c_int {
    unsafe { convert_setup_ext(vcp, from, true, to, true) }
}

/// [`convert_setup`], choosing what "Unicode" means on each side.
///
/// `*_unicode_is_utf8` says whether *any* Unicode encoding on that side may
/// be treated as UTF-8, or only the one whose properties are exactly
/// `ENC_UNICODE` — which is `utf-8` itself. A reader of a UTF-16 file needs
/// the strict answer; a caller that has already decoded to UTF-8 wants the
/// loose one.
///
/// # Safety
///
/// As [`convert_setup`].
pub unsafe fn convert_setup_ext(
    vcp: *mut vimconv_T,
    from: *mut c_char,
    from_unicode_is_utf8: bool,
    to: *mut c_char,
    to_unicode_is_utf8: bool,
) -> c_int {
    unsafe {
        if (*vcp).vc_type == CONV_ICONV && (*vcp).vc_fd != iconv_failed() {
            iconv_close((*vcp).vc_fd);
        }
        *vcp = CONV_NONE_INIT;

        // Nothing to do: an unnamed side, or the same encoding twice.
        if from.is_null() || *from == 0 || to.is_null() || *to == 0 || strcmp(from, to) == 0 {
            return OK;
        }

        let from_prop = enc_canon_props(from);
        let to_prop = enc_canon_props(to);
        let is_utf8 = |prop: EncProps, loose: bool| {
            if loose {
                prop & ENC_UNICODE != 0
            } else {
                prop == ENC_UNICODE
            }
        };
        let from_is_utf8 = is_utf8(from_prop, from_unicode_is_utf8);
        let to_is_utf8 = is_utf8(to_prop, to_unicode_is_utf8);

        // `vc_factor` is how much the output can grow: a Latin-1 byte becomes
        // at most two UTF-8 bytes, a Latin-9 one at most three, and iconv's
        // worst case is budgeted at four.
        if from_prop & ENC_LATIN1 != 0 && to_is_utf8 {
            (*vcp).vc_type = CONV_TO_UTF8;
            (*vcp).vc_factor = 2;
        } else if from_prop & ENC_LATIN9 != 0 && to_is_utf8 {
            (*vcp).vc_type = CONV_9_TO_UTF8;
            (*vcp).vc_factor = 3;
        } else if from_is_utf8 && to_prop & ENC_LATIN1 != 0 {
            (*vcp).vc_type = CONV_TO_LATIN1;
        } else if from_is_utf8 && to_prop & ENC_LATIN9 != 0 {
            (*vcp).vc_type = CONV_TO_LATIN9;
        } else {
            // A side already known to be UTF-8 is named as such, whatever
            // Unicode spelling it arrived under.
            let named = |is_utf8: bool, name: *mut c_char| {
                if is_utf8 {
                    c"utf-8".as_ptr().cast_mut()
                } else {
                    name
                }
            };
            (*vcp).vc_fd = my_iconv_open(named(to_is_utf8, to), named(from_is_utf8, from));
            if (*vcp).vc_fd != iconv_failed() {
                (*vcp).vc_type = CONV_ICONV;
                (*vcp).vc_factor = 4;
            }
        }
        if (*vcp).vc_type == CONV_NONE {
            FAIL
        } else {
            OK
        }
    }
}

/// [`string_convert_ext`] without the incomplete-tail report.
///
/// # Safety
///
/// As [`string_convert_ext`].
pub unsafe fn string_convert(
    vcp: *const vimconv_T,
    ptr: *mut c_char,
    lenp: *mut size_t,
) -> *mut c_char {
    unsafe { string_convert_ext(vcp, ptr, lenp, core::ptr::null_mut()) }
}

/// Run `vcp`'s plan over `ptr`, answering a freshly allocated string.
///
/// `lenp` is the input length in and the output length out; null means "NUL
/// terminated". `unconvlenp`, when given, receives the length of an
/// incomplete sequence left at the end — the caller is reading a stream and
/// the rest of it has not arrived yet.
///
/// Null comes back when the conversion failed: an invalid sequence in the
/// input, or an unrepresentable character with `vc_fail` set.
///
/// # Safety
///
/// `ptr` must have `*lenp` readable bytes, or be NUL-terminated when `lenp`
/// is null. The result is `xmalloc`'d.
pub unsafe fn string_convert_ext(
    vcp: *const vimconv_T,
    ptr: *mut c_char,
    lenp: *mut size_t,
    unconvlenp: *mut size_t,
) -> *mut c_char {
    unsafe {
        let len = if lenp.is_null() { strlen(ptr) } else { *lenp };
        if len == 0 {
            return xstrdup(c"".as_ptr());
        }
        let src = core::slice::from_raw_parts(ptr as *const u8, len);

        // iconv manages its own buffer and reports its own length.
        if (*vcp).vc_type == CONV_ICONV {
            return iconv_string(vcp, ptr, len, unconvlenp, lenp);
        }

        // The worst-case growth of each conversion, which is what upstream
        // allocates: it converts *in place* into this buffer. Building the
        // answer first and copying it in would be tighter, but the allocation
        // size is observable -- `test/unit/eval/typval_spec.lua` asserts the
        // exact malloc sizes a converting `tv_list_copy` makes -- and it is
        // the same bound `vimconv_T::vc_factor` promises callers.
        let factor: size_t = match (*vcp).vc_type {
            CONV_TO_UTF8 => 2,
            CONV_9_TO_UTF8 => 3,
            CONV_TO_LATIN1 | CONV_TO_LATIN9 => 1,
            // CONV_NONE, and anything else: nothing was planned.
            _ => return core::ptr::null_mut(),
        };
        let room = len * factor + 1;
        let result = xmalloc(room) as *mut u8;

        let out = match (*vcp).vc_type {
            CONV_TO_UTF8 => Some(latin1_to_utf8(src)),
            CONV_9_TO_UTF8 => Some(latin9_to_utf8(src)),
            _ => utf8_to_latin(
                src,
                (*vcp).vc_type == CONV_TO_LATIN9,
                (*vcp).vc_fail,
                unconvlenp,
            ),
        };
        let Some(out) = out else {
            xfree(result as *mut c_void);
            return core::ptr::null_mut();
        };
        debug_assert!(out.len() < room, "conversion overran vc_factor");
        core::ptr::copy_nonoverlapping(out.as_ptr(), result, out.len());
        *result.add(out.len()) = 0;
        if !lenp.is_null() {
            *lenp = out.len();
        }
        result as *mut c_char
    }
}

/// Latin-1 to UTF-8: every byte is the codepoint of the same number.
fn latin1_to_utf8(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() * 2);
    for &c in src {
        if c < 0x80 {
            out.push(c);
        } else {
            out.push(0xc0 + (c >> 6));
            out.push(0x80 + (c & 0x3f));
        }
    }
    out
}

/// The eight positions where Latin-9 differs from Latin-1, as
/// `(latin9 byte, codepoint)`.
///
/// Latin-9 is Latin-1 with eight characters replaced: the euro sign, the
/// French œ ligature and Ÿ, and the S and Z with caron that Baltic languages
/// need.
const LATIN9_REPLACEMENTS: [(u8, c_int); 8] = [
    (0xa4, 0x20ac), // €
    (0xa6, 0x0160), // Š
    (0xa8, 0x0161), // š
    (0xb4, 0x017d), // Ž
    (0xb8, 0x017e), // ž
    (0xbc, 0x0152), // Œ
    (0xbd, 0x0153), // œ
    (0xbe, 0x0178), // Ÿ
];

/// Latin-9 to UTF-8: Latin-1's mapping, with the eight replacements applied.
fn latin9_to_utf8(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() * 3);
    let mut buf = [0 as c_char; MB_MAXCHAR];
    for &b in src {
        let c = LATIN9_REPLACEMENTS
            .iter()
            .find(|&&(byte, _)| byte == b)
            .map_or(b as c_int, |&(_, code)| code);
        // SAFETY: `buf` is the longest sequence `utf_char2bytes` can write.
        let n = unsafe { utf_char2bytes(c, buf.as_mut_ptr()) } as usize;
        out.extend(buf[..n].iter().map(|&b| b as u8));
    }
    out
}

/// UTF-8 to Latin-1 or Latin-9.
///
/// A character with no representation becomes `¿` (plus a `?` when it was
/// drawn two cells wide, so columns line up), unless `fail` asks for an error
/// instead. Composing characters are dropped: the base character is all
/// either target can hold.
///
/// Answers `None` when the input is not UTF-8 at all, or when `fail` is set
/// and a character cannot be represented.
///
/// # Safety
///
/// `unconvlenp` must be null or writable.
unsafe fn utf8_to_latin(
    src: &[u8],
    to_latin9: bool,
    fail: bool,
    unconvlenp: *mut size_t,
) -> Option<Vec<u8>> {
    unsafe {
        let mut out = Vec::with_capacity(src.len());
        let mut i = 0;
        while i < src.len() {
            let p = src.as_ptr().add(i) as *const c_char;
            let l = utf_ptr2len_len(p, (src.len() - i) as c_int);
            if l == 0 {
                // An embedded NUL, which `len` says is part of the string.
                out.push(0);
                i += 1;
                continue;
            }
            if l == 1 {
                if utf8len_tab_zero[src[i] as usize] == 0 {
                    return None; // not a lead byte: the input is not UTF-8
                }
                if !unconvlenp.is_null()
                    && utf8len_tab_zero[src[i] as usize] as usize > src.len() - i
                {
                    // A sequence cut off by the end of the input.
                    *unconvlenp = src.len() - i;
                    break;
                }
                out.push(src[i]);
                i += 1;
                continue;
            }

            let mut c = utf_ptr2char(p);
            if to_latin9 {
                c = to_latin9_byte(c);
            }
            if !utf_iscomposing_legacy(c) {
                if c < 0x100 {
                    out.push(c as u8);
                } else if fail {
                    return None;
                } else {
                    out.push(0xbf); // ¿
                    if utf_char2cells(c) > 1 {
                        out.push(b'?');
                    }
                }
            }
            i += l as usize;
        }
        Some(out)
    }
}

/// The Latin-9 byte for a codepoint, or a value no byte can hold.
///
/// The eight replaced positions map back to their bytes. The eight
/// *Latin-1* characters those positions displaced map to `0x100`, which is
/// out of range on purpose: they exist in Latin-1 but not in Latin-9, so they
/// must become `¿` rather than silently turning into the wrong glyph.
fn to_latin9_byte(c: c_int) -> c_int {
    if let Some(&(byte, _)) = LATIN9_REPLACEMENTS.iter().find(|&&(_, code)| code == c) {
        return byte as c_int;
    }
    if LATIN9_REPLACEMENTS
        .iter()
        .any(|&(byte, _)| byte as c_int == c)
    {
        return 0x100;
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin1_round_trip() {
        let src: Vec<u8> = (0u8..=255).collect();
        let utf8 = latin1_to_utf8(&src);
        // Every byte is the codepoint of the same number.
        assert_eq!(String::from_utf8(utf8).unwrap().chars().count(), 256);
    }

    /// The Latin-9 map has to be a bijection on the eight positions, or a
    /// round trip would lose characters.
    #[test]
    fn latin9_map_is_a_bijection() {
        for &(byte, code) in &LATIN9_REPLACEMENTS {
            assert_eq!(to_latin9_byte(code), byte as c_int);
            // The Latin-1 character this position displaced has no Latin-9
            // spelling, and must be pushed out of byte range.
            assert_eq!(to_latin9_byte(byte as c_int), 0x100);
        }
        // Everything else passes through.
        assert_eq!(to_latin9_byte('a' as c_int), 'a' as c_int);
        assert_eq!(to_latin9_byte(0xe9), 0xe9); // é, same in both
    }

    #[test]
    fn latin9_encodes_the_euro() {
        assert_eq!(latin9_to_utf8(&[0xa4]), "€".as_bytes());
        assert_eq!(latin9_to_utf8(b"a"), b"a");
    }
}
