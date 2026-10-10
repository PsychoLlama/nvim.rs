//! The UTF-8 codec: bytes to a codepoint and back -- and the home of every
//! pointer spelling of the multibyte readers that still has callers.
//!
//! This port decodes the *original* UTF-8, up to six bytes, not the four-byte
//! form Unicode settled on in 2003. That is upstream's choice and it shows up
//! everywhere in here: the length tables, `utf_char2len` and `utf_char2bytes`
//! all know about five- and six-byte sequences, and a codepoint above
//! `U+10FFFF` round-trips rather than being rejected.
//!
//! Two decoders, deliberately different.
//!
//! - [`utf_ptr2char`] ([`char_at`]) is forgiving: given a byte that is not
//!   the start of a valid sequence, or a sequence with a bad continuation
//!   byte, it answers the *first byte's own value*. Nothing is ever an error,
//!   so a caller walking a buffer of arbitrary bytes always gets something.
//! - [`utf_ptr2char_info_impl`] ([`strict_char_at`]) is strict: it answers a
//!   **negative** number for a sequence it will not accept, which is how the
//!   header's inlined `utf_ptr2char_info` decides to report a length of one.
//!   It never handles ASCII — its caller must have ruled that out — and its
//!   arithmetic is branch-light on purpose, because it is the decode on the
//!   drawing path.
//!
//! The `utfc_*` spellings count the composing characters that follow a base
//! character as part of it; the plain ones stop at the base.
//!
//! # The pointer forms
//!
//! Every reader is written once, in [`text`], over a [`text::Text`] that is
//! either a slice or a NUL-terminated pointer. The slice forms are the API;
//! the pointer forms below are one-line shims over the same readers, kept
//! for the callers outside this family whose own cursor is still a pointer
//! (the regexp engine, spell, the drawing path) and deleted as those convert.
//! Because both spellings are the same code monomorphised twice, a shim
//! compiles to what the pointer body it replaced compiled to.
//!
//! Four of them are exported by name and may not change signature:
//! `utf_ptr2char`, `utf_ptr2len` and `utf_ptr2char_info_impl` are
//! `metrics/abi-ledger.jsonl` rows, and `utf8len_tab` (in `tables`) is
//! compiled *against* by `unit-fixtures.so`. That, and the shims, is why
//! this file allows `unsafe` and its siblings do not.

#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`utf_ptr2char`, `utf_ptr2char_info_impl`, `utf_ptr2len`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use core::ffi::{CStr, c_char, c_int};

// The carve of the transpiled module; see each child's docs.
mod slice;
mod tables;
mod text;

pub use self::slice::{
    char_at, char_info_at, char_len, encode_char, promised_char_len, strict_char_at,
};
pub use self::tables::*;
use self::text::{
    NulTerminated, Prefix, Text, ambiguous_width_in, cells_in, char_at_in, char_info_in,
    char_len_in, cluster_len_in, head_off_in, next_cluster_in, string_cells_in,
};

/// The characters of `s`, as `MB_PTR_ADV` steps over them: one item per base
/// character, with any composing characters folded into the step.
///
/// The [`CStr`] spelling of [`clusters`], for a caller that has the string
/// rather than its bytes.
pub fn chars(s: &CStr) -> Clusters<'_> {
    clusters(s.to_bytes())
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
/// complete sequence there; [`char_at`] over a string.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2char(p: *const c_char) -> c_int {
    char_at_in(unsafe { NulTerminated::new(p) }, 0)
}

/// How many bytes the character at `p` occupies, or 0 at a NUL;
/// [`char_len`] over a string.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2len(p: *const c_char) -> c_int {
    char_len_in(unsafe { NulTerminated::new(p) }, 0) as c_int
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
/// `p1` and `p2` must point at characters in a NUL-terminated string, and
/// `state` must be null or point at a live [`GraphemeState`].
pub unsafe fn utf_composinglike(
    p1: *const c_char,
    p2: *const c_char,
    state: *mut GraphemeState,
) -> bool {
    let (cur, next) = unsafe { (NulTerminated::new(p1), NulTerminated::new(p2)) };
    // ASCII never combines, and this is the hot answer.
    next.byte(0) >= 0x80
        && iscomposing(char_at_in(cur, 0), char_at_in(next, 0), unsafe {
            state.as_mut()
        })
}

/// Is `c2` part of the same grapheme cluster as `c1`?
///
/// # Safety
///
/// `state` must be null or point at a live [`GraphemeState`], carried across
/// the calls of one walk.
pub unsafe fn utf_iscomposing(c1: c_int, c2: c_int, state: *mut GraphemeState) -> bool {
    // SAFETY: the caller's obligation.
    iscomposing(c1, c2, unsafe { state.as_mut() })
}

/// [`utf_iscomposing`] with the walk state borrowed rather than pointed at,
/// which is all the pointer ever meant. `None` is a walk with no state to
/// carry -- the caller is asking about two characters in isolation.
///
/// Arabic gets an extra rule of its own: two letters that a shaper would join
/// are treated as one character here even though Unicode breaks between them.
pub fn iscomposing(c1: c_int, c2: c_int, state: Option<&mut GraphemeState>) -> bool {
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

/// The whole grapheme cluster at `p` packed into a `ScreenChar`, with its base
/// codepoint written to `firstc`.
///
/// Answers 0 for a byte that is not a character (the caller displays it) and
/// for a cluster that does not fit.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string and `firstc` must be writable.
///
/// `inline(always)`, and the benchmark says so: this is the per-cell call of
/// `win_line`'s draw loop, and it *was* inlined there until the option
/// rewrite grew the loop's other callees past the inliner's threshold. The
/// call it fell back to cost **187 M instructions on `scrbench`** -- 0.66 %
/// of the whole benchmark, and two thirds of phase 32's regression against
/// its floor -- which is what a call costs at five million cells. See
/// [`utfc_ptrlen2schar`], whose call site in `grid_line_puts` moved the same
/// way.
#[inline(always)]
pub unsafe fn utfc_ptr2schar(p: *const c_char, firstc: *mut c_int) -> ScreenChar {
    let text = unsafe { NulTerminated::new(p) };
    let c = char_at_in(text, 0);
    unsafe { *firstc = c };
    let first_compose = utf_iscomposing_first(c);
    let maxlen = (MAX_SCHAR_SIZE - 1) as usize - usize::from(first_compose);
    let len = cluster_len_in(Prefix::new(text, 0, maxlen), 0);
    if len == 1 && text.byte(0) >= 0x80 {
        return 0;
    }
    unsafe { schar_from_buf_first(p, len, first_compose) }
}

/// [`utfc_ptr2schar`] for a cluster whose length is already known.
///
/// # Safety
///
/// `p` must point at `len` readable bytes and `firstc` must be writable.
///
/// `inline(always)` for the reason [`utfc_ptr2schar`] gives.
#[inline(always)]
pub unsafe fn utfc_ptrlen2schar(
    p: *const c_char,
    mut len: c_int,
    firstc: *mut c_int,
) -> ScreenChar {
    if len == 0 || (len == 1 && unsafe { *p } as u8 >= 0x80) {
        unsafe { *firstc = *p as u8 as c_int };
        return 0;
    }
    let c = unsafe { utf_ptr2char(p) };
    unsafe { *firstc = c };
    let first_compose = utf_iscomposing_first(c);
    let maxlen = MAX_SCHAR_SIZE - 1 - first_compose as c_int;
    if len > maxlen {
        // Only the first `maxlen` of the caller's `len` bytes are read.
        let text = unsafe { NulTerminated::new(p) };
        len = cluster_len_in(Prefix::new(text, 0, maxlen as usize), 0) as c_int;
    }
    unsafe { schar_from_buf_first(p, len as size_t, first_compose) }
}

/// Pack `len` bytes into a `ScreenChar`, prefixing a space when the cluster
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
unsafe fn schar_from_buf_first(buf: *const c_char, len: size_t, first_compose: bool) -> ScreenChar {
    if !first_compose {
        return unsafe { schar_from_buf(buf, len) };
    }
    let mut cbuf = [0 as c_char; MAX_SCHAR_SIZE as usize];
    cbuf[0] = b' ' as c_char;
    unsafe { core::ptr::copy_nonoverlapping(buf, cbuf.as_mut_ptr().offset(1), len) };
    unsafe { schar_from_buf(cbuf.as_ptr(), len + 1) }
}

/// How many bytes the sequence introduced by byte `b` occupies.
pub fn utf_byte2len(b: c_int) -> c_int {
    utf8len_tab[b as usize] as c_int
}

/// [`utf_ptr2len`] over at most `size` bytes, reporting a sequence `size`
/// cuts short at its **full** length; [`promised_char_len`] over the
/// caller's bytes.
///
/// # Safety
///
/// `p` must point at `max(size, 1)` readable bytes.
pub unsafe fn utf_ptr2len_len(p: *const c_char, size: c_int) -> c_int {
    let held = usize::try_from(size).unwrap_or(0).max(1);
    promised_char_len(unsafe { cstr::slice_at(p, held) }) as c_int
}

/// How many bytes the whole grapheme cluster at `p` occupies — the base
/// character plus every composing character following it — or 0 at a NUL;
/// [`cluster_len`] over a string.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn utfc_ptr2len(p: *const c_char) -> c_int {
    cluster_len_in(unsafe { NulTerminated::new(p) }, 0) as c_int
}

/// How many bytes the grapheme cluster at the start of `bytes` occupies --
/// the base character plus every composing character following it -- or 0 if
/// `bytes` is empty.
///
/// The cluster twin of [`char_len`]. The bound the slice carries does the
/// work the pointer form's NUL did, so the answer is never larger than
/// `bytes.len()`; a composing character only partly inside the slice is left
/// out, because the rest of it may still arrive.
///
/// The difference the slice forms all carry applies here too: a NUL is an
/// ordinary byte, one character long, not the end of the string.
#[inline]
pub fn cluster_len(bytes: &[u8]) -> usize {
    cluster_len_in(bytes, 0)
}

/// The grapheme clusters of `bytes`, as `(byte offset, codepoint)`.
///
/// The safe spelling of a `while (*p) { c = utf_ptr2char(p); MB_PTR_ADV(p); }`
/// loop, for a caller that has the text rather than a pointer into it. Every
/// byte of `bytes` is visited, an embedded NUL included; [`chars`] is the
/// same walk over a [`CStr`].
pub struct Clusters<'a> {
    bytes: &'a [u8],
    /// Byte offset of the next cluster.
    at: usize,
}

/// The grapheme clusters of `bytes`; see [`Clusters`].
pub fn clusters(bytes: &[u8]) -> Clusters<'_> {
    Clusters { bytes, at: 0 }
}

impl Iterator for Clusters<'_> {
    /// The byte offset the cluster starts at, and its base codepoint.
    type Item = (usize, c_int);

    fn next(&mut self) -> Option<(usize, c_int)> {
        let rest = self.bytes.get(self.at..)?;
        if rest.is_empty() {
            return None;
        }
        let at = self.at;
        self.at = at + cluster_len(rest);
        Some((at, char_at(rest)))
    }
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

/// Encode `c` into `buf`, answering how many bytes it took; [`encode_char`]
/// into the caller's buffer.
///
/// # Safety
///
/// `buf` must have room for [`utf_char2len`] bytes — up to `MB_MAXCHAR`.
pub unsafe fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int {
    let len = utf_char2len(c) as usize;
    encode_char(c, unsafe {
        core::slice::from_raw_parts_mut(buf.cast::<u8>(), len)
    }) as c_int
}

/// The codepoint at `p` and the number of bytes it occupies; an invalid
/// sequence reports a negative value with a length of one. [`char_info_at`]
/// over a string, and the Rust twin of the header's inlined
/// `utf_ptr2char_info`; the C fixture in `test/unit/fixtures/shim.h` carries
/// its own copy of the same body.
///
/// # Safety
///
/// `p` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2char_info(p: *const c_char) -> CharInfo {
    char_info_in(unsafe { NulTerminated::new(p) }, 0)
}

/// How far back from `p` the grapheme cluster covering it starts; [`head_off`]
/// over a string.
///
/// # Safety
///
/// `base` must point at the start of a NUL-terminated string and `p` into it.
pub unsafe fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int {
    let at = unsafe { p.offset_from(base) }.unsigned_abs();
    head_off_in(unsafe { NulTerminated::new(base) }, at) as c_int
}

/// How far back from byte `at` the grapheme cluster covering it starts.
///
/// Composing characters count as part of the character they follow, so this
/// is not just "step back over continuation bytes". `at` may be `text.len()`:
/// a walk that steps one past the last character is asking about the
/// terminator, which is ASCII and starts no cluster.
///
/// The walk stays inside `text` in both directions: a sequence the slice
/// cuts short is an illegal one, and its bytes are their own characters.
pub fn head_off(text: &[u8], at: usize) -> usize {
    head_off_in(text, at)
}

/// How many bytes forward from `p` the next character starts, when `p` is in
/// the middle of one. Zero when `p` is already at a character start.
///
/// # Safety
///
/// `base` must point at the start of a NUL-terminated string and `p` into it.
pub unsafe fn mb_off_next(base: *const c_char, p: *const c_char) -> c_int {
    let (text, at) = unsafe { (NulTerminated::new(base), p.offset_from(base).unsigned_abs()) };
    let back = head_off_in(text, at);
    if back == 0 {
        return 0;
    }
    (cluster_len_in(text, at - back) - back) as c_int
}

/// The start of the character before `p`, or `p` itself at the start of the
/// line.
///
/// # Safety
///
/// `line` must point at the start of a NUL-terminated string and `p` into it.
pub unsafe fn mb_prevptr(line: *mut c_char, p: *mut c_char) -> *mut c_char {
    if p <= line {
        return p;
    }
    let before = unsafe { p.offset_from(line) }.unsigned_abs() - 1;
    let back = head_off_in(unsafe { NulTerminated::new(line) }, before);
    p.wrapping_sub(back + 1)
}

/// `text` paired with its codepoint: the start of a character and the
/// character itself. Composing characters are not consulted.
///
/// # Safety
///
/// `text` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2str_char_info(text: *mut c_char) -> StrCharInfo {
    StrCharInfo {
        ptr: text,
        chr: unsafe { utf_ptr2char_info(text) },
    }
}

/// The character after `cur`, treating a following composing character as
/// part of the *current* one.
///
/// # Safety
///
/// `cur.ptr` must point into a NUL-terminated string, at a character start.
#[inline(always)]
pub unsafe fn utfc_next(cur: StrCharInfo) -> StrCharInfo {
    let (off, chr) = next_cluster_in(unsafe { NulTerminated::new(cur.ptr) }, 0, cur.chr);
    StrCharInfo {
        ptr: cur.ptr.wrapping_add(off),
        chr,
    }
}

/// How many characters a NUL-terminated string holds; [`char_count`] over a
/// string. A null pointer is zero.
///
/// # Safety
///
/// `str` must be null or point at a NUL-terminated string.
pub unsafe fn mb_charlen(str: *const c_char) -> c_int {
    let bytes = unsafe { cstr::at_opt(str) }.map_or(&[][..], CStr::to_bytes);
    c_int::try_from(char_count(bytes)).expect("a count of bytes bounds a count of characters")
}

/// [`mb_charlen`] over at most `len` bytes, stopping at a NUL.
///
/// # Safety
///
/// `str` must point at `len` readable bytes, or at a NUL before them.
pub unsafe fn mb_charlen_len(str: *const c_char, len: c_int) -> c_int {
    let bytes = unsafe { cstr::prefix_at(str, usize::try_from(len).unwrap_or(0)) };
    c_int::try_from(char_count(bytes)).expect("a count of bytes fits a count of characters")
}

/// Both ends of the codepoint covering `p`, as offsets from it; [`cp_bounds`]
/// over the `p_len` bytes from `p` and everything before it.
///
/// # Safety
///
/// `base <= p_in`, `p_len > 0`, and `p_in` must have `p_len` readable bytes.
pub unsafe fn utf_cp_bounds_len(
    base: *const c_char,
    p_in: *const c_char,
    p_len: c_int,
) -> CharBoundsOff {
    debug_assert!(base <= p_in && p_len > 0, "base <= p_in && p_len > 0");
    // Only a sequence's worth forward is ever read, so a `p_len` larger than
    // that -- `INT_MAX`, from [`utf_cp_bounds`] -- may be clamped to it
    // without changing an answer: the "complete within what is left" test
    // can only fail beyond it.
    let at = unsafe { p_in.offset_from(base) }.unsigned_abs();
    let forward = usize::try_from(p_len).unwrap_or(0).min(MB_MAXCHAR);
    cp_bounds(unsafe { cstr::slice_at(base, at + forward) }, at)
}

/// [`utf_cp_bounds_len`] over a NUL-terminated string, where the length does
/// not bound anything.
///
/// # Safety
///
/// `base` must point at the start of a NUL-terminated string and `p_in` into
/// it, with a sequence's worth of bytes (or the NUL) after `p_in`.
pub unsafe fn utf_cp_bounds(base: *const c_char, p_in: *const c_char) -> CharBoundsOff {
    unsafe { utf_cp_bounds_len(base, p_in, INT_MAX) }
}

/// Count the characters in `s[..len]`, adding the totals to `codepoints` and
/// `codeunits`; [`utf_lengths`] over a string.
///
/// # Safety
///
/// `s` must point at a NUL-terminated string at least `len` bytes long; both
/// counters must be writable.
pub unsafe fn mb_utflen(
    s: *const c_char,
    len: size_t,
    codepoints: *mut size_t,
    codeunits: *mut size_t,
) {
    let (points, units) = text::utf_lengths_in(unsafe { NulTerminated::new(s) }, len);
    unsafe { *codepoints += points };
    unsafe { *codeunits += units };
}

/// The byte offset just past the character at index `index`, counted in
/// codepoints or in UTF-16 code units, or −1 if the string is shorter;
/// [`utf_index_to_byte`] over a string.
///
/// # Safety
///
/// `s` must point at a NUL-terminated string at least `len` bytes long.
pub unsafe fn mb_utf_index_to_bytes(
    s: *const c_char,
    len: size_t,
    index: size_t,
    use_utf16_units: bool,
) -> ssize_t {
    let text = unsafe { NulTerminated::new(s) };
    text::utf_index_to_byte_in(text, len, index, use_utf16_units).map_or(-1, |at| at.cast_signed())
}

/// Count the characters in `text[..len]`: how many codepoints, and how many
/// UTF-16 code units. The two differ by the surrogate pairs: a character
/// above the BMP is one codepoint but two UTF-16 code units, which is what the
/// LSP-facing callers need.
///
/// **Not a slice walk over `text[..len]`, deliberately.** A sequence `len`
/// cuts short still reports its full length, and is decoded out of the
/// bytes of `text` past `len` -- which is how a four-byte character
/// straddling the cut is still counted as a surrogate pair. Upstream's body
/// does the same.
pub fn utf_lengths(text: &[u8], len: usize) -> (usize, usize) {
    text::utf_lengths_in(text, len)
}

/// The byte offset just past the character at index `index` of `text[..len]`,
/// counted in codepoints or in UTF-16 code units, or `None` if there are
/// fewer. Counted the way [`utf_lengths`] counts.
pub fn utf_index_to_byte(
    text: &[u8],
    len: usize,
    index: usize,
    use_utf16_units: bool,
) -> Option<usize> {
    text::utf_index_to_byte_in(text, len, index, use_utf16_units)
}

/// How many cells the character at `p` occupies; [`cells_at`] over a string.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn utf_ptr2cells(p: *const c_char) -> c_int {
    cells_in(unsafe { NulTerminated::new(p) }, 0)
}

/// How many cells the character at the start of `bytes` occupies.
///
/// The answer for an *empty* slice is 1 -- what the pointer form answers at a
/// NUL, because nothing to print still occupies the cell the cursor sits in.
/// The slice is the string, so a sequence it cuts short is an illegal one,
/// drawn as `<xx>`.
#[inline]
pub fn cells_at(bytes: &[u8]) -> c_int {
    cells_in(bytes, 0)
}

/// [`utf_ptr2cells`] over a buffer that is `size` bytes long rather than
/// NUL-terminated; see [`text::partial_cells_in`] for upstream's shape.
///
/// # Safety
///
/// `p` must point at `size` readable bytes, and a sequence `size` cuts short
/// must be completed (or ended by a NUL) by the bytes after them.
pub unsafe fn utf_ptr2cells_len(p: *const c_char, size: c_int) -> c_int {
    let size = usize::try_from(size).unwrap_or(0);
    text::partial_cells_in(unsafe { NulTerminated::new(p) }, 0, size)
}

/// The total width of a NUL-terminated string; [`string_cells`] over it.
///
/// # Safety
///
/// `str` must point at a NUL-terminated string.
pub unsafe fn mb_string2cells(str: *const c_char) -> size_t {
    string_cells_in(unsafe { NulTerminated::new(str) }, 0)
}

/// The total width of `bytes`. A NUL inside the slice is an ordinary byte,
/// one cell wide, rather than the end of the string.
pub fn string_cells(bytes: &[u8]) -> usize {
    string_cells_in(bytes, 0)
}

/// The total width of at most `size` bytes, stopping early at a NUL, each
/// character measured as [`utf_ptr2cells_len`] measures it.
///
/// # Safety
///
/// As [`utf_ptr2cells_len`].
pub unsafe fn mb_string2cells_len(str: *const c_char, size: size_t) -> size_t {
    text::partial_string_cells_in(unsafe { NulTerminated::new(str) }, size)
}

/// Might the character at `p` be drawn at a width the grid did not predict?
/// [`ambiguous_width_at`] over a string.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn utf_ambiguous_width(p: *const c_char) -> bool {
    ambiguous_width_in(unsafe { NulTerminated::new(p) }, 0)
}

/// Might the character at the start of `bytes` be drawn at a width the grid
/// did not predict? An ambiguous-width character answers whatever
/// `'ambiwidth'` currently says, an emoji whatever `'emoji'` says, and a VS-16
/// after any character turns it into an emoji.
pub fn ambiguous_width_at(bytes: &[u8]) -> bool {
    ambiguous_width_in(bytes, 0)
}

/// The character class of the character `p` points at, deciding Latin-1
/// with `chartab`; [`class_with`] over a string.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string, and `chartab` at a buffer's
/// `b_chartab`, the four-word `'iskeyword'` bitmap.
pub unsafe fn mb_get_class_tab(p: *const c_char, chartab: *const uint64_t) -> c_int {
    let chartab = unsafe { &*chartab.cast::<[uint64_t; 4]>() };
    let text = unsafe { NulTerminated::new(p) };
    class_of(text.byte(0), || char_at_in(text, 0), chartab)
}

/// [`mb_get_class_tab`] against the current buffer's `'iskeyword'`.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn mb_get_class(p: *const c_char) -> c_int {
    let text = unsafe { NulTerminated::new(p) };
    class_of(
        text.byte(0),
        || char_at_in(text, 0),
        &Buf::current().b_chartab,
    )
}

/// Compare two strings case-insensitively, at most `n1`/`n2` bytes each and
/// never past a NUL; [`strnicmp_in`] over them.
///
/// # Safety
///
/// `s1` and `s2` must point at `n1` and `n2` readable bytes, or at a NUL
/// before them.
pub unsafe fn utf_strnicmp(s1: *const c_char, s2: *const c_char, n1: size_t, n2: size_t) -> c_int {
    strnicmp_in(unsafe { cstr::prefix_at(s1, n1) }, unsafe {
        cstr::prefix_at(s2, n2)
    })
}

/// [`utf_strnicmp`] with one length for both sides.
///
/// # Safety
///
/// Both strings must have `nn` readable bytes, or a NUL before them.
pub unsafe fn mb_strnicmp(s1: *const c_char, s2: *const c_char, nn: size_t) -> c_int {
    unsafe { utf_strnicmp(s1, s2, nn, nn) }
}

/// `strcmp` or [`utf_strnicmp`] of two NUL-terminated strings, whichever `ic`
/// asks for.
///
/// # Safety
///
/// Both strings must be NUL-terminated.
pub unsafe fn mb_strcmp_ic(ic: bool, s1: *const c_char, s2: *const c_char) -> c_int {
    let (s1, s2) = unsafe { (cstr::bytes_at(s1), cstr::bytes_at(s2)) };
    strcmp_ic_in(ic, s1, s2)
}

/// Is every byte of `s` part of a well-formed UTF-8 sequence? `end` bounds
/// the check, and a null `end` means "up to the NUL"; [`valid_utf8`] over
/// those bytes.
///
/// # Safety
///
/// `s` must be NUL-terminated when `end` is null, and reach `end` otherwise.
pub unsafe fn utf_valid_string(s: *const c_char, end: *const c_char) -> bool {
    let bytes = if end.is_null() {
        unsafe { cstr::bytes_at(s) }
    } else {
        unsafe { cstr::slice_at(s, end.offset_from(s).unsigned_abs()) }
    };
    valid_utf8(bytes)
}

/// Decode one character out of a key-encoded string, advancing `*cursor` past
/// it; [`unescape_at`] over the string. Answers null when there is no
/// multibyte character there, or `into`, holding it NUL-terminated.
///
/// # Safety
///
/// `*cursor` must point at a NUL-terminated string.
pub(crate) unsafe fn mb_unescape(
    cursor: *mut *const c_char,
    into: &mut [c_char; MB_MAXCHAR],
) -> *const c_char {
    // Four characters, each at most three bytes escaped, and the NUL: the
    // most `unescape_at` looks at.
    let text = unsafe { cstr::prefix_at(*cursor, 13) };
    let Some((bytes, used)) = unescape_at(text) else {
        return core::ptr::null();
    };
    *into = [0; MB_MAXCHAR];
    for (slot, &byte) in into.iter_mut().zip(&bytes) {
        *slot = byte as c_char;
    }
    unsafe { *cursor = (*cursor).add(used) };
    into.as_ptr()
}

/// The properties of encoding `name`; [`canon_props`] over the string.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string.
pub unsafe fn enc_canon_props(name: *const c_char) -> EncProps {
    canon_props(unsafe { cstr::bytes_at(name) })
}

/// `p` past an encoding name's `8bit-`/`2byte-` prefix; [`skip_len`] over
/// the string.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn enc_skip(p: *mut c_char) -> *mut c_char {
    p.wrapping_add(skip_len(unsafe { cstr::bytes_at(p) }))
}

/// The canonical spelling of encoding name `enc`, `xmalloc`'d;
/// [`canonize`] over the string.
///
/// # Safety
///
/// `enc` must point at a NUL-terminated string.
pub unsafe fn enc_canonize(enc: *mut c_char) -> *mut c_char {
    canonize(unsafe { cstr::bytes_at(enc) }).into_raw()
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
