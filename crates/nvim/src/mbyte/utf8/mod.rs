//! The UTF-8 codec: bytes to a codepoint and back.
//!
//! This port decodes the *original* UTF-8, up to six bytes, not the four-byte
//! form Unicode settled on in 2003. That is upstream's choice and it shows up
//! everywhere in here: the length tables, `utf_char2len` and `utf_char2bytes`
//! all know about five- and six-byte sequences, and a codepoint above
//! `U+10FFFF` round-trips rather than being rejected.
//!
//! Two decoders, deliberately different.
//!
//! - [`utf_ptr2char`] is forgiving: given a byte that is not the start of a
//!   valid sequence, or a sequence with a bad continuation byte, it answers
//!   the *first byte's own value*. Nothing is ever an error, so a caller
//!   walking a buffer of arbitrary bytes always gets something.
//! - [`utf_ptr2char_info_impl`] is strict: it answers a **negative** number
//!   for a sequence it will not accept, which is how the header's inlined
//!   `utf_ptr2char_info` decides to report a length of one. It never handles
//!   ASCII — its caller must have ruled that out — and its arithmetic is
//!   branch-light on purpose, because it is the decode on the drawing path.
//!
//! `utf_ptr2char_info_impl` and `utf8len_tab` are the two symbols
//! `unit-fixtures.so` compiles against, so neither may change signature.
//!
//! The `utfc_*` spellings count the composing characters that follow a base
//! character as part of it; the plain ones stop at the base.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{CStr, c_char, c_int};

// The carve of the transpiled module; see each child's docs.
mod tables;

pub use self::tables::*;

/// The characters of a NUL-terminated string, as `MB_PTR_ADV` steps over
/// them: one item per base character, with any composing characters folded
/// into the step, as `(byte offset, codepoint)`.
///
/// This is the safe spelling of the `while (*p) { c = utf_ptr2char(p);
/// MB_PTR_ADV(p); }` loop. The pointer forms it wraps ask their caller for a
/// NUL-terminated string and an offset before the NUL; a [`CStr`] is the
/// first and the iterator's own bookkeeping is the second, so a caller that
/// has the string rather than a pointer needs no `unsafe` of its own.
pub struct Chars<'a> {
    str: &'a CStr,
    /// Bytes before the NUL, measured once.
    len: usize,
    /// Byte offset of the next character.
    at: usize,
}

impl<'a> Chars<'a> {
    fn new(str: &'a CStr) -> Self {
        Chars {
            str,
            len: str.to_bytes().len(),
            at: 0,
        }
    }
}

/// The characters of `s`; see [`Chars`].
pub fn chars(s: &CStr) -> Chars<'_> {
    Chars::new(s)
}

impl Iterator for Chars<'_> {
    /// The byte offset the character starts at, and its codepoint.
    type Item = (usize, c_int);

    fn next(&mut self) -> Option<(usize, c_int)> {
        if self.at >= self.len {
            return None;
        }
        let at = self.at;
        // SAFETY: `at` is a character boundary before the NUL of a
        // NUL-terminated string, which is what both of these want. Neither
        // reads past the NUL.
        let (c, len) = unsafe {
            let p = self.str.as_ptr().add(at);
            (utf_ptr2char(p), utfc_ptr2len(p) as usize)
        };
        // Zero is the answer only at the NUL, which `at` never points at.
        debug_assert!(len > 0, "utfc_ptr2len stalled mid-string");
        self.at = at + len;
        Some((at, c))
    }
}

/// A grapheme-break state that has seen nothing yet.
pub const GRAPHEME_STATE_INIT: c_int = 0;

/// The bits a lead byte contributes, per sequence length.
///
/// Index 0 and 1 are never used: a one-byte sequence is ASCII and handled
/// before any of this. A six-byte lead is `0xFC`/`0xFD`, so only its bottom
/// bit survives — the rest is shifted out of a 32-bit result, which is why
/// upstream's correction table leaves the `0xFC << 30` term commented out.
const LEAD_PAYLOAD: [u32; 7] = [0, 0, 0x1f, 0x0f, 0x07, 0x03, 0x01];

/// The bits a lead byte announces, per sequence length: `0b110xxxxx` for two
/// bytes, `0b1110xxxx` for three, and so on.
const LEAD_PREFIX: [u32; 7] = [0, 0, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc];

/// What [`utf_ptr2char_info_impl`] adds to its accumulated bits, per length.
///
/// The accumulator sums whole bytes rather than masking each one, so what is
/// left over is the sum of the UTF-8 framing bits, and this subtracts it. For
/// lengths 0 and 1 — which mean "not a character" — it instead *sets* bit 31,
/// so the answer comes back negative. That is safe to add rather than test
/// for: two bytes cannot reach bit 31 on their own.
const CORRECTIONS: [u32; 7] = {
    let mut corr = [0u32; 7];
    corr[0] = 1 << 31;
    corr[1] = 1 << 31;
    let mut len = 2;
    while len < 7 {
        // The lead's own framing bits, plus 0x80 for every continuation byte.
        let mut framing = LEAD_PREFIX[len] << (6 * (len - 1));
        let mut i = 1;
        while i < len {
            framing = framing.wrapping_add(0x80 << (6 * (len - 1 - i)));
            i += 1;
        }
        corr[len] = framing.wrapping_neg();
        len += 1;
    }
    corr
};

/// Is this a UTF-8 continuation byte, `0b10xxxxxx`?
#[inline(always)]
pub(crate) fn utf_is_trail_byte(byte: u8) -> bool {
    byte & 0xc0 == 0x80
}

/// Decode the multibyte sequence at `p`, which is `len` bytes long.
///
/// **Does not handle ASCII**: a `len` of 0 or 1 means "not a character" and
/// makes the answer negative, which is also what a bad continuation byte
/// produces. So a negative answer means "this is not a character here", and
/// the caller displays the byte rather than the codepoint.
///
/// # Safety
///
/// `p` must point at `max(2, len)` readable bytes — the second byte is read
/// unconditionally, which is sound for every sequence this is called for
/// because an incomplete one cannot be the last byte of a NUL-terminated
/// string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2char_info_impl(p: *const uint8_t, len: uintptr_t) -> int32_t {
    // The second byte is always read, even for `len` 0 and 1; the
    // correction makes the answer negative either way.
    let read = if len < 3 { 2 } else { len };
    let mut code_point = unsafe { *p } as u32;
    for i in 1..read {
        let cur = unsafe { *p.add(i) };
        if !utf_is_trail_byte(cur) {
            return -1;
        }
        code_point = (code_point << 6).wrapping_add(cur as u32);
    }
    code_point.wrapping_add(CORRECTIONS[len]) as int32_t
}

/// The codepoint at `p`, or the first byte's own value if there is no
/// complete sequence there.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string. Only as many bytes as the lead
/// byte promises are read, and a NUL is never a continuation byte, so a
/// truncated sequence stops at it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2char(p_in: *const c_char) -> c_int {
    let p = p_in as *const u8;
    let first = unsafe { *p } as u32;
    if first < 0x80 {
        return first as c_int;
    }
    let len = utf8len_tab[first as usize] as usize;
    if len < 2 {
        // A continuation byte or 0xFE/0xFF: not a lead byte at all.
        return first as c_int;
    }
    let mut code_point = first & LEAD_PAYLOAD[len];
    for i in 1..len {
        let cur = unsafe { *p.add(i) };
        if !utf_is_trail_byte(cur) {
            return first as c_int;
        }
        code_point = (code_point << 6) | (cur as u32 & 0x3f);
    }
    code_point as c_int
}

/// Read one character out of `*s`, advancing it and shrinking `*n`.
///
/// Answers 0 when nothing is left, and −1 for a byte sequence that is not a
/// character — including a valid-looking one whose value equals its own first
/// byte, which is how an overlong encoding is caught. `0xC3 0x83` (Ã) is
/// exempted because it decodes to 0xC3, the value of its own lead byte, and
/// is a real character.
///
/// # Safety
///
/// `*s` must point at `*n` readable bytes.
pub(crate) unsafe fn utf_safe_read_char_adv(s: *mut *const c_char, n: *mut size_t) -> c_int {
    if unsafe { *n } == 0 {
        return 0;
    }
    let first = unsafe { **s } as u8;
    let k = utf8len_tab_zero[first as usize];
    if k == 1 {
        unsafe { *n -= 1 };
        unsafe { *s = (*s).offset(1) };
        return first as c_int;
    }
    if k as size_t <= unsafe { *n } {
        let c = unsafe { utf_ptr2char(*s) };
        if c != first as c_int || (c == 0xc3 && unsafe { *(*s).offset(1) } as u8 == 0x83) {
            unsafe { *s = (*s).offset(k as isize) };
            unsafe { *n -= k as size_t };
            return c;
        }
    }
    -1
}

/// The codepoint at `*pp`, advancing `*pp` past it **and its composing
/// characters**.
///
/// # Safety
///
/// `*pp` must point at a NUL-terminated string.
pub unsafe fn mb_ptr2char_adv(pp: *mut *const c_char) -> c_int {
    let c = unsafe { utf_ptr2char(*pp) };
    unsafe { *pp = (*pp).offset(utfc_ptr2len(*pp) as isize) };
    c
}

/// The codepoint at `*pp`, advancing `*pp` past the base character only —
/// so the next call answers the composing character.
///
/// # Safety
///
/// `*pp` must point at a NUL-terminated string.
pub unsafe fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int {
    let c = unsafe { utf_ptr2char(*pp) };
    unsafe { *pp = (*pp).offset(utf_ptr2len(*pp) as isize) };
    c
}

/// Can `c` combine onto a character before it, whatever that character is?
///
/// Asked of a character in isolation, by testing it against a space: if it
/// does not break a grapheme cluster even after a space, it combines with
/// anything.
pub fn utf_iscomposing_first(c: c_int) -> bool {
    c >= 128 && !utf8proc_grapheme_break(' ' as utf8proc_int32_t, c as utf8proc_int32_t)
}

/// Is the character at `p2` part of the same grapheme cluster as the one at
/// `p1`?
///
/// # Safety
///
/// `p1` and `p2` must point at characters in a NUL-terminated string.
pub unsafe fn utf_composinglike(
    p1: *const c_char,
    p2: *const c_char,
    state: *mut GraphemeState,
) -> bool {
    // ASCII never combines, and this is the hot answer.
    if (unsafe { *p2 } as u8) < 128 {
        return false;
    }
    unsafe { utf_iscomposing(utf_ptr2char(p1), utf_ptr2char(p2), state) }
}

/// Is `c2` part of the same grapheme cluster as `c1`?
///
/// Arabic gets an extra rule of its own: two letters that a shaper would join
/// are treated as one character here even though Unicode breaks between them.
///
/// # Safety
///
/// `state` must be null or point at a live [`GraphemeState`], carried across
/// the calls of one walk.
pub unsafe fn utf_iscomposing(c1: c_int, c2: c_int, state: *mut GraphemeState) -> bool {
    // SAFETY: the caller's obligation.
    let state = unsafe { state.as_mut() };
    !utf8proc_grapheme_break_stateful(c1 as utf8proc_int32_t, c2 as utf8proc_int32_t, state)
        || crate::arabic::arabic_combine(c1, c2)
}

/// Is `c` a combining mark by Unicode's *category*?
///
/// The pre-grapheme-cluster test, kept for the regexp engine, which matches
/// composing characters one at a time rather than as clusters.
pub fn utf_iscomposing_legacy(c: c_int) -> bool {
    let category = utf8proc_get_property(c).category as c_int;
    category == UTF8PROC_CATEGORY_MN as c_int || category == UTF8PROC_CATEGORY_ME as c_int
}

/// The whole grapheme cluster at `p` packed into a `schar_T`, with its base
/// codepoint written to `firstc`.
///
/// Answers 0 for a byte that is not a character (the caller displays it) and
/// for a cluster that does not fit.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string and `firstc` must be writable.
pub unsafe fn utfc_ptr2schar(p: *const c_char, firstc: *mut c_int) -> schar_T {
    let c = unsafe { utf_ptr2char(p) };
    unsafe { *firstc = c };
    let first_compose = utf_iscomposing_first(c);
    let maxlen = MAX_SCHAR_SIZE - 1 - first_compose as c_int;
    let len = unsafe { utfc_ptr2len_len(p, maxlen) } as size_t;
    if len == 1 && unsafe { *p } as u8 >= 0x80 {
        return 0;
    }
    unsafe { schar_from_buf_first(p, len, first_compose) }
}

/// [`utfc_ptr2schar`] for a cluster whose length is already known.
///
/// # Safety
///
/// `p` must point at `len` readable bytes and `firstc` must be writable.
pub unsafe fn utfc_ptrlen2schar(p: *const c_char, mut len: c_int, firstc: *mut c_int) -> schar_T {
    if len == 0 || (len == 1 && unsafe { *p } as u8 >= 0x80) {
        unsafe { *firstc = *p as u8 as c_int };
        return 0;
    }
    let c = unsafe { utf_ptr2char(p) };
    unsafe { *firstc = c };
    let first_compose = utf_iscomposing_first(c);
    let maxlen = MAX_SCHAR_SIZE - 1 - first_compose as c_int;
    if len > maxlen {
        len = unsafe { utfc_ptr2len_len(p, maxlen) };
    }
    unsafe { schar_from_buf_first(p, len as size_t, first_compose) }
}

/// Pack `len` bytes into a `schar_T`, prefixing a space when the cluster
/// starts with a composing character.
///
/// A cell has to have something to compose *onto*, so a leading combining
/// mark is stored as "space plus the mark" — which is why the callers reserve
/// a byte for it in their length budget.
///
/// # Safety
///
/// `buf` must point at `len` readable bytes, and `len` must leave room for
/// the space when `first_compose` is set.
unsafe fn schar_from_buf_first(buf: *const c_char, len: size_t, first_compose: bool) -> schar_T {
    if !first_compose {
        return unsafe { schar_from_buf(buf, len) };
    }
    let mut cbuf = [0 as c_char; MAX_SCHAR_SIZE as usize];
    cbuf[0] = b' ' as c_char;
    unsafe { core::ptr::copy_nonoverlapping(buf, cbuf.as_mut_ptr().offset(1), len) };
    unsafe { schar_from_buf(cbuf.as_ptr(), len + 1) }
}

/// How many bytes the character at `p` occupies, or 0 at a NUL.
///
/// Answers 1 for anything that is not a complete sequence, so a walk always
/// makes progress.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2len(p_in: *const c_char) -> c_int {
    let p = p_in as *const u8;
    if unsafe { *p } == 0 {
        return 0;
    }
    let len = utf8len_tab[unsafe { *p } as usize] as c_int;
    for i in 1..len {
        if !utf_is_trail_byte(unsafe { *p.offset(i as isize) }) {
            return 1;
        }
    }
    len
}

/// How many bytes the sequence introduced by byte `b` occupies.
pub fn utf_byte2len(b: c_int) -> c_int {
    utf8len_tab[b as usize] as c_int
}

/// [`utf_ptr2len`] over at most `size` bytes.
///
/// A sequence cut short by `size` still reports its **full** length, so the
/// caller can tell "incomplete" from "invalid" by comparing against `size`.
///
/// # Safety
///
/// `p` must point at at least one readable byte, and at `min(size, len)`.
pub unsafe fn utf_ptr2len_len(p: *const c_char, size: c_int) -> c_int {
    let len = utf8len_tab[unsafe { *p } as u8 as usize] as c_int;
    if len == 1 {
        return 1;
    }
    for i in 1..len.min(size) {
        if !utf_is_trail_byte(unsafe { *p.offset(i as isize) } as u8) {
            return 1;
        }
    }
    len
}

/// How many bytes the whole grapheme cluster at `p` occupies — the base
/// character plus every composing character following it.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn utfc_ptr2len(p: *const c_char) -> c_int {
    let first = unsafe { *p } as u8;
    if first == 0 {
        return 0;
    }
    // Two ASCII bytes: nothing can be combining, answer without decoding.
    if first < 0x80 && (unsafe { *p.offset(1) } as u8) < 0x80 {
        return 1;
    }
    let mut len = unsafe { utf_ptr2len(p) };
    if len == 1 && first >= 0x80 {
        return 1;
    }
    let mut prevlen = 0;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    loop {
        let next = unsafe { p.offset(len as isize) };
        if (unsafe { *next } as u8) < 0x80
            || !unsafe { utf_composinglike(p.offset(prevlen as isize), next, &raw mut state) }
        {
            return len;
        }
        prevlen = len;
        len += unsafe { utf_ptr2len(next) };
    }
}

/// [`utfc_ptr2len`] over at most `size` bytes.
///
/// A composing character that is only partly within `size` is left out: the
/// rest of it may still arrive.
///
/// # Safety
///
/// `p` must point at `size` readable bytes.
pub unsafe fn utfc_ptr2len_len(p: *const c_char, size: c_int) -> c_int {
    if size < 1 || unsafe { *p } == 0 {
        return 0;
    }
    let first = unsafe { *p } as u8;
    if first < 0x80 && (size == 1 || (unsafe { *p.offset(1) } as u8) < 0x80) {
        return 1;
    }
    let mut len = unsafe { utf_ptr2len_len(p, size) };
    if (len == 1 && first >= 0x80) || len > size {
        return 1;
    }
    let mut prevlen = 0;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    while len < size {
        let next = unsafe { p.offset(len as isize) };
        if (unsafe { *next } as u8) < 0x80 {
            break;
        }
        let next_len = unsafe { utf_ptr2len_len(next, size - len) };
        if next_len > size - len {
            break; // truncated by `size`, not part of this cluster
        }
        if !unsafe { utf_composinglike(p.offset(prevlen as isize), next, &raw mut state) } {
            break;
        }
        prevlen = len;
        len += next_len;
    }
    len
}

/// How many bytes `c` encodes to.
pub fn utf_char2len(c: c_int) -> c_int {
    if c < 0x80 {
        1
    } else if c < 0x800 {
        2
    } else if c < 0x10000 {
        3
    } else if c < 0x200000 {
        4
    } else if c < 0x4000000 {
        5
    } else {
        6
    }
}

/// Encode `c` into `buf`, answering how many bytes it took.
///
/// # Safety
///
/// `buf` must have room for [`utf_char2len`] bytes — up to `MB_MAXCHAR`.
pub unsafe fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int {
    let len = utf_char2len(c) as usize;
    if len == 1 {
        unsafe { *buf = c as c_char };
        return 1;
    }
    let u = c as u32;
    // The lead byte carries the top bits under its `1..10` prefix; every
    // continuation byte carries six more under `10`.
    unsafe { *buf = (LEAD_PREFIX[len] | (u >> (6 * (len - 1)))) as c_char };
    for i in 1..len {
        unsafe { *buf.add(i) = (0x80 | ((u >> (6 * (len - 1 - i))) & 0x3f)) as c_char };
    }
    len as c_int
}

/// The codepoint at `p` and the number of bytes it occupies. An invalid
/// sequence reports a negative value with a length of one.
///
/// The Rust twin of the header's inlined `utf_ptr2char_info`; the C fixture in
/// `test/unit/fixtures/shim.h` carries its own copy of the same body.
///
/// # Safety
///
/// `p` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2char_info(p_in: *const c_char) -> CharInfo {
    let p = p_in as *const uint8_t;
    let first = unsafe { *p };
    if first < 0x80 {
        return CharInfo {
            value: first as int32_t,
            len: 1,
        };
    }
    let len = utf8len_tab[first as usize] as c_int;
    let code_point = unsafe { utf_ptr2char_info_impl(p, len as uintptr_t) };
    CharInfo {
        value: code_point,
        len: if code_point < 0 { 1 } else { len },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Masking the lead byte and subtracting upstream's correction are the
    /// same arithmetic, for every lead byte that can reach each length.
    /// [`utf_ptr2char`] does the first and [`utf_ptr2char_info_impl`] the
    /// second, so this is what says the two decoders agree.
    #[test]
    fn lead_masking_matches_the_corrections() {
        for len in 2..=6usize {
            let shift = 6 * (len - 1) as u32;
            for first in 0u32..256 {
                if utf8len_tab[first as usize] as usize != len {
                    continue;
                }
                let masked = (first & LEAD_PAYLOAD[len]).wrapping_shl(shift);
                let biased = first
                    .wrapping_shl(shift)
                    .wrapping_sub(LEAD_PREFIX[len].wrapping_shl(shift));
                assert_eq!(masked, biased, "len {len}, lead {first:#04x}");
            }
        }
    }

    /// Every length's correction is exactly the framing bits its own lead
    /// prefix and continuation bytes contribute.
    #[test]
    fn corrections_undo_the_framing() {
        assert_eq!(CORRECTIONS[0], 1 << 31);
        assert_eq!(CORRECTIONS[1], 1 << 31);
        assert_eq!(CORRECTIONS[2], (0x80u32 + (0xc0 << 6)).wrapping_neg());
        assert_eq!(
            CORRECTIONS[3],
            (0x80u32 + (0x80 << 6) + (0xe0 << 12)).wrapping_neg()
        );
        assert_eq!(
            CORRECTIONS[4],
            (0x80u32 + (0x80 << 6) + (0x80 << 12) + (0xf0 << 18)).wrapping_neg()
        );
        assert_eq!(
            CORRECTIONS[5],
            (0x80u32 + (0x80 << 6) + (0x80 << 12) + (0x80 << 18) + (0xf8u32.wrapping_shl(24)))
                .wrapping_neg()
        );
        // The `0xFC << 30` term is absent by construction: it is zero in 32
        // bits, which is exactly why upstream leaves it commented out.
        assert_eq!(
            CORRECTIONS[6],
            (0x80u32 + (0x80 << 6) + (0x80 << 12) + (0x80 << 18) + (0x80u32.wrapping_shl(24)))
                .wrapping_neg()
        );
        assert_eq!(0xfcu32.wrapping_shl(30), 0);
    }

    /// Encode then decode, over every codepoint the six-byte form reaches —
    /// sampled, plus every boundary of every length.
    #[test]
    fn encode_decode_round_trip() {
        let mut buf = [0 as c_char; MB_MAXCHAR + 1];
        let mut check = |c: c_int| {
            // SAFETY: `buf` has room for the longest sequence plus the NUL
            // this writes after it, so every read below is in bounds and
            // terminated.
            let len = unsafe { utf_char2bytes(c, buf.as_mut_ptr()) };
            assert_eq!(len, utf_char2len(c), "{c:#x} length");
            buf[len as usize] = 0;
            assert_eq!(unsafe { utf_ptr2char(buf.as_ptr()) }, c, "{c:#x}");
            // A NUL is the one character whose length `utf_ptr2len`
            // reports as 0 rather than 1 — it is the end of the string,
            // not a character in it.
            let want = if c == 0 { 0 } else { len };
            assert_eq!(unsafe { utf_ptr2len(buf.as_ptr()) }, want, "{c:#x} ptr2len");
        };
        for c in 0..0x2000 {
            check(c);
        }
        for boundary in [0x80, 0x800, 0x10000, 0x200000, 0x4000000] {
            for c in boundary - 2..boundary + 2 {
                check(c);
            }
        }
        for c in (0x10000..0x7fff_ffff).step_by(0x9_1d31) {
            check(c);
        }
    }

    /// A NUL, a continuation byte and `0xFE` are all "one byte, its own
    /// value" to the forgiving decoder, and all negative to the strict one.
    #[test]
    fn not_a_character() {
        for &bytes in &[
            b"\x00\x00".as_slice(),
            b"\x80\x80",
            b"\xfe\x80",
            b"\xc3\x41",
        ] {
            let p = bytes.as_ptr() as *const c_char;
            // SAFETY: every literal above is at least two bytes long and the
            // decoders read no further than the lead byte promises.
            assert_eq!(unsafe { utf_ptr2char(p) }, bytes[0] as c_int, "{bytes:?}");
            let info = unsafe { utf_ptr2char_info(p) };
            if bytes[0] >= 0x80 {
                assert!(info.value < 0, "{bytes:?}");
                assert_eq!(info.len, 1, "{bytes:?}");
            }
        }
    }
}
