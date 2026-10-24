//! The UTF-8 codec: bytes to a codepoint and back -- and the home of the two
//! pointer spellings of the multibyte readers the C harness still resolves.
//!
//! This port decodes the *original* UTF-8, up to six bytes, not the four-byte
//! form Unicode settled on in 2003. That is upstream's choice and it shows up
//! everywhere in here: the length tables, `utf_char2len` and `encode_char`
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
//! # Two kinds of text
//!
//! Every reader is written once, in [`text`], over a [`text::Text`]. The
//! slice forms (`char_at`, `cluster_len`, `cells_at`, ...) read a `&[u8]`
//! whose end is the end of the text and in which a NUL is an ordinary byte.
//! The `c_*` forms read the bytes of a C string: the slice's end *or* its
//! first NUL ends the text, which is exactly what upstream's pointer
//! spellings (`utfc_ptr2len`, `utf_head_off`, `mb_string2cells`, ...) read,
//! and they are those spellings' replacements; `tests/unit/mbyte_oracle.rs`
//! holds them to the digests recorded from the pointer forms.
//!
//! Four items are exported by name and may not change signature:
//! `utf_ptr2char`, `utf_ptr2len` and `utf_ptr2char_info_impl` are
//! `metrics/abi-ledger.jsonl` rows, and `utf8len_tab` (in `tables`) is
//! compiled *against* by `unit-fixtures.so`. They are the only pointer forms
//! left, and why this file allows `unsafe` and its siblings do not.

#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`utf_ptr2char`, `utf_ptr2char_info_impl`, `utf_ptr2len`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]

use super::*;
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
    CBytes, NulTerminated, Prefix, Text, ambiguous_width_in, cells_in, char_at_in, char_info_in,
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

/// Upstream's `utf_iscomposing` with the walk state borrowed rather than pointed at,
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

/// The whole grapheme cluster at the start of `bytes`, the bytes of a C
/// string, packed into a `ScreenChar`, and its base codepoint: upstream's
/// `utfc_ptr2schar`. The slice's end or a NUL in it ends the text.
///
/// The cell is 0 for a byte that is not a character (the caller displays
/// it) and for the end of the text.
pub fn c_cluster_schar(bytes: &[u8]) -> (ScreenChar, c_int) {
    let text = CBytes(bytes);
    let c = char_at_in(text, 0);
    let first_compose = utf_iscomposing_first(c);
    let maxlen = (MAX_SCHAR_SIZE - 1) as usize - usize::from(first_compose);
    let len = cluster_len_in(Prefix::new(text, 0, maxlen), 0);
    if len == 1 && text.byte(0) >= 0x80 {
        return (0, c);
    }
    (pack_cluster(&bytes[..len], first_compose), c)
}

/// Pack a cluster into a `ScreenChar`, prefixing a space when it starts with
/// a composing character: a cell has to have something to compose *onto*,
/// which is why the callers keep a byte of their length budget for it.
fn pack_cluster(cluster: &[u8], first_compose: bool) -> ScreenChar {
    if !first_compose {
        return schar_from_buf(cluster);
    }
    let mut cbuf = [0u8; MAX_SCHAR_SIZE as usize];
    cbuf[0] = b' ';
    cbuf[1..=cluster.len()].copy_from_slice(cluster);
    schar_from_buf(&cbuf[..cluster.len() + 1])
}

/// [`c_cluster_len`] and [`c_cluster_schar`] of the cluster at the start of
/// `bytes`, the bytes of a C string, in one walk: its length (0 at the end),
/// the cell it makes and its base codepoint.
///
/// The cell is the cluster cut to what a `ScreenChar` holds. A cluster that
/// ends a whole sequence's worth short of that limit is the one the capped
/// walk finds too -- nothing either walk reads lies past the limit -- so only
/// a longer one is walked a second time.
#[inline]
pub(crate) fn c_cluster_cell(bytes: &[u8]) -> (usize, ScreenChar, c_int) {
    let text = CBytes(bytes);
    let len = cluster_len_in(text, 0);
    let c = char_at_in(text, 0);
    let first_compose = utf_iscomposing_first(c);
    let maxlen = (MAX_SCHAR_SIZE - 1) as usize - usize::from(first_compose);
    let cell_len = if len + 6 <= maxlen {
        len
    } else {
        cluster_len_in(Prefix::new(text, 0, maxlen), 0)
    };
    if cell_len == 1 && text.byte(0) >= 0x80 {
        return (len, 0, c);
    }
    // A cluster never runs past the end of the text it is read from, and
    // `maxlen` left room for the space.
    (len, pack_cluster(&bytes[..cell_len], first_compose), c)
}

/// How many cells the character at the start of `bytes`, the bytes of a C
/// string, occupies: upstream's `utf_ptr2cells`, read the way
/// [`c_cluster_schar`] reads.
pub fn c_cells_at(bytes: &[u8]) -> c_int {
    cells_in(CBytes(bytes), 0)
}

/// [`c_cells_at`] of the character at byte `at`, without slicing: the
/// per-character walks in `plines` measure in place.
#[inline(always)]
pub(crate) fn c_cells_in(bytes: &[u8], at: usize) -> c_int {
    cells_in(CBytes(bytes), at)
}

/// The codepoint at byte `at` of the bytes of a C string and the number of
/// bytes it occupies -- an invalid sequence reports a negative value with a
/// length of one: upstream's `utf_ptr2char_info`.
#[inline(always)]
pub fn c_char_info(bytes: &[u8], at: usize) -> CharInfo {
    char_info_in(CBytes(bytes), at)
}

/// Upstream's `utfc_next` over the bytes of a C string: where the character
/// after the cluster at byte `at`, whose base is `cur`, starts, and what it
/// is. At the end of the text that is the NUL past it.
#[inline(always)]
pub fn c_next_cluster(bytes: &[u8], at: usize, cur: CharInfo) -> (usize, CharInfo) {
    next_cluster_in(CBytes(bytes), at, cur)
}

/// How many bytes the sequence introduced by byte `b` occupies.
pub fn utf_byte2len(b: c_int) -> c_int {
    utf8len_tab[b as usize] as c_int
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

/// [`cluster_len`] of the bytes of a C string: the cluster length the
/// NUL-terminated string `bytes` came from answers -- 0 at a NUL or the end,
/// and a composing character the end interrupts is an invalid byte rather
/// than one still to arrive. For a slice form that stands in for a pointer
/// form byte for byte.
#[inline]
pub fn c_cluster_len(bytes: &[u8]) -> usize {
    cluster_len_in(CBytes(bytes), 0)
}

/// The display width of the characters that start in the first `len` bytes
/// of `bytes`, the bytes of a C string, a character that starts inside the
/// bound counted whole: upstream's `vim_strnsize`.
#[inline]
pub fn c_display_width(bytes: &[u8], len: c_int) -> c_int {
    text::display_width_in(CBytes(bytes), len)
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

/// [`head_off`] over the bytes of a C string, which end at their first NUL:
/// upstream's `utf_head_off`.
pub fn c_head_off(bytes: &[u8], at: usize) -> usize {
    head_off_in(CBytes(bytes), at)
}

/// How many bytes forward from byte `at` of the bytes of a C string the next
/// character starts, when `at` is in the middle of one; 0 at a character
/// start. Upstream's `mb_off_next`.
pub fn c_off_next(bytes: &[u8], at: usize) -> usize {
    let text = CBytes(bytes);
    let back = head_off_in(text, at);
    if back == 0 {
        return 0;
    }
    cluster_len_in(text, at - back) - back
}

/// Where the character before byte `at` of the bytes of a C string starts,
/// or `at` itself at the start: upstream's `mb_prevptr`, as an offset.
pub fn c_prev_start(bytes: &[u8], at: usize) -> usize {
    if at == 0 {
        return 0;
    }
    at - 1 - head_off_in(CBytes(bytes), at - 1)
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

/// [`utf_lengths`] over the bytes of a C string, which end at their first
/// NUL: upstream's `mb_utflen`.
pub fn c_utf_lengths(bytes: &[u8], len: usize) -> (usize, usize) {
    text::utf_lengths_in(CBytes(bytes), len)
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

/// [`utf_index_to_byte`] over the bytes of a C string, which end at their
/// first NUL: upstream's `mb_utf_index_to_bytes`.
pub fn c_utf_index_to_byte(
    bytes: &[u8],
    len: usize,
    index: usize,
    use_utf16_units: bool,
) -> Option<usize> {
    text::utf_index_to_byte_in(CBytes(bytes), len, index, use_utf16_units)
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

/// [`string_cells`] of the bytes of a C string, as upstream's
/// `mb_string2cells` reads them: a sequence the end cuts short is an invalid
/// byte.
#[inline]
pub fn c_string_cells(bytes: &[u8]) -> usize {
    string_cells_in(CBytes(bytes), 0)
}

/// The total width of `bytes`. A NUL inside the slice is an ordinary byte,
/// one cell wide, rather than the end of the string.
pub fn string_cells(bytes: &[u8]) -> usize {
    string_cells_in(bytes, 0)
}

/// How many cells the character at byte `at` of the bytes of a C string
/// occupies, out of a buffer `size` bytes long: upstream's
/// `utf_ptr2cells_len`. A sequence `size` cuts short is decoded out of the
/// bytes past it; see `text::partial_cells_in`.
#[inline]
pub fn c_partial_cells_in(bytes: &[u8], at: usize, size: usize) -> c_int {
    text::partial_cells_in(CBytes(bytes), at, size)
}

/// The width of at most `size` of the bytes of a C string, stopping early at
/// a NUL, each character measured as [`c_partial_cells_in`] measures it:
/// upstream's `mb_string2cells_len`.
#[inline]
pub fn c_string_cells_len(bytes: &[u8], size: usize) -> usize {
    text::partial_string_cells_in(CBytes(bytes), size)
}

/// Might the character at the start of `bytes` be drawn at a width the grid
/// did not predict? An ambiguous-width character answers whatever
/// `'ambiwidth'` currently says, an emoji whatever `'emoji'` says, and a VS-16
/// after any character turns it into an emoji.
pub fn ambiguous_width_at(bytes: &[u8]) -> bool {
    ambiguous_width_in(bytes, 0)
}

/// [`ambiguous_width_at`] over the bytes of a C string, which end at their
/// first NUL: upstream's `utf_ambiguous_width`.
pub fn c_ambiguous_width(bytes: &[u8]) -> bool {
    ambiguous_width_in(CBytes(bytes), 0)
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
        let mut buf = [0u8; MB_MAXCHAR + 1];
        let mut check = |c: c_int| {
            let len = encode_char(c, &mut buf);
            assert_eq!(len as c_int, utf_char2len(c), "{c:#x} length");
            buf[len] = 0;
            // SAFETY: `buf` holds the sequence and the NUL written after it.
            assert_eq!(unsafe { utf_ptr2char(buf.as_ptr().cast()) }, c, "{c:#x}");
            // A NUL is the one character whose length `utf_ptr2len`
            // reports as 0 rather than 1 — it is the end of the string,
            // not a character in it.
            let want = if c == 0 { 0 } else { len as c_int };
            // SAFETY: as above.
            assert_eq!(
                unsafe { utf_ptr2len(buf.as_ptr().cast()) },
                want,
                "{c:#x} ptr2len"
            );
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

    /// The two exported pointer readers, at every offset of strings that end
    /// in the middle of a sequence, a cluster or an emoji, each in an
    /// allocation that ends exactly at its NUL: under Miri, a reader that
    /// looked one byte past the terminator would be an out-of-bounds read.
    /// This is the claim `text.rs`'s safety argument makes, checked.
    #[test]
    fn no_pointer_reader_reads_past_the_nul() {
        let texts: [&[u8]; 14] = [
            b"",
            b"a",
            b"\xc3",
            b"\xe2\x82",
            b"\xf0\x9f\x98",
            b"\xfc\x84\x80\x80\x80",
            b"a\xcc",
            b"\xd8\x80\xe2",
            b"\xd8\x80\xe2\x82",
            b"\xe2\x9d\xa4\xef\xb8",
            b"\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f",
            b"\xd9\x84\xd8",
            b"\x80\x80\x80",
            b"x\xcc\x80\xcc\x81\xcc",
        ];
        for text in texts {
            // An allocation of exactly the string and its NUL.
            let buf: Box<[c_char]> = text
                .iter()
                .map(|&b| b as c_char)
                .chain(core::iter::once(0))
                .collect();
            let p = buf.as_ptr();
            for at in 0..=text.len() {
                // SAFETY: `p + at` is inside the NUL-terminated `buf`.
                unsafe {
                    let q = p.add(at);
                    let _ = utf_ptr2char(q);
                    let _ = utf_ptr2len(q);
                }
            }
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
            let info = char_info_at(bytes);
            if bytes[0] >= 0x80 {
                assert!(info.value < 0, "{bytes:?}");
                assert_eq!(info.len, 1, "{bytes:?}");
            }
        }
    }
}
