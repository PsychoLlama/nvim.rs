//! The UTF-8 readers, written once over the two kinds of text a caller can
//! hold: a slice, whose end is the end of the text, and a pointer into a
//! NUL-terminated string, whose NUL is.
//!
//! Every slice form in this family and every pointer form that still has
//! callers outside it is a one-line call into a reader here, so the two
//! cannot drift: they are the same code, monomorphised twice. [`Text`] is
//! the difference between them, and it is three questions -- the byte at an
//! offset, whether the text has ended there, and how much of it is left if
//! the text can say.
//!
//! The answers differ exactly where the slice forms document that they do.
//! A slice's end reads as a NUL, so a sequence the slice cuts short is
//! incomplete, as one cut short by a NUL is; a NUL *inside* a slice is an
//! ordinary byte, one character long, where the pointer forms stop. A reader
//! that cares about the difference (whether an incomplete sequence at the end
//! may still be completed, say) asks [`Text::left`], which only a slice can
//! answer.
//!
//! # The pointer text, and why it is sound
//!
//! [`NulTerminated`] reads its bytes through a raw pointer, and nothing in
//! the type stops a reader asking for a byte past the NUL. What stops it is
//! that **every reader in this file reads forwards from the offset it was
//! given and stops at the first byte that is not part of what it is
//! decoding**, and a NUL never is: it is ASCII, so it ends a character, never
//! continues a sequence and never composes. The backwards walk of
//! [`head_off_in`] reads only bytes before the offset it was given, which a
//! caller's pointer into a string is past. That discipline is the whole of
//! the safety argument, so the type and every reader that can be handed one
//! live in this one file, and nothing outside it can name either.

#![deny(unsafe_op_in_unsafe_fn)]
// `NulTerminated` reads a C string through its pointer; see the module docs.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_char;

use super::{
    CORRECTIONS, GRAPHEME_STATE_INIT, LEAD_PAYLOAD, iscomposing, utf_is_trail_byte, utf8len_tab,
};
use crate::charset::char2cells;
use crate::mbyte::{emoji_widens, prop_is_emojilike, utf_char2cells};
use crate::types::{CharInfo, GraphemeState};
use crate::utf8proc::{
    UTF8PROC_BOUNDCLASS_CONTROL, UTF8PROC_BOUNDCLASS_CR, UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC,
    UTF8PROC_BOUNDCLASS_OTHER, UTF8PROC_BOUNDCLASS_PREPEND, utf8proc_get_property,
};

/// The most bytes a UTF-8 sequence can occupy in this port's decoders, which
/// is how far back a byte's lead can possibly be.
const MAX_SEQUENCE: usize = 6;

/// Variation Selector 16: "draw the character before me as an emoji".
const VS16: i32 = 0xfe0f;

/// The text a reader walks; see the module docs.
pub(super) trait Text: Copy {
    /// The byte at `i`. Past the end of a slice this is a NUL, which is what
    /// a pointer would have read there.
    fn byte(self, i: usize) -> u8;
    /// Whether the text has ended at `i`: past a slice's last byte, or at a
    /// pointer's NUL.
    fn ends_at(self, i: usize) -> bool;
    /// How many bytes the text holds from `i` on, when it can say -- a slice
    /// can, a NUL-terminated string cannot without reading them.
    fn left(self, i: usize) -> Option<usize>;
}

impl Text for &[u8] {
    #[inline(always)]
    fn byte(self, i: usize) -> u8 {
        self.get(i).copied().unwrap_or(0)
    }

    #[inline(always)]
    fn ends_at(self, i: usize) -> bool {
        i >= self.len()
    }

    #[inline(always)]
    fn left(self, i: usize) -> Option<usize> {
        Some(self.len().saturating_sub(i))
    }
}

/// The bytes of a C string, held as a slice: the slice's end or a NUL in it
/// ends the text, and nothing in it is "cut short" -- a sequence its end
/// interrupts is just not one, as it would be at a NUL. The readers answer
/// over this exactly what they answer over the [`NulTerminated`] string the
/// bytes came from.
#[derive(Clone, Copy)]
pub(super) struct CBytes<'a>(pub(super) &'a [u8]);

impl Text for CBytes<'_> {
    #[inline(always)]
    fn byte(self, i: usize) -> u8 {
        self.0.get(i).copied().unwrap_or(0)
    }

    #[inline(always)]
    fn ends_at(self, i: usize) -> bool {
        self.byte(i) == 0
    }

    #[inline(always)]
    fn left(self, _i: usize) -> Option<usize> {
        None
    }
}

/// A NUL-terminated string, read through a pointer to its start.
#[derive(Clone, Copy)]
pub(super) struct NulTerminated(*const u8);

impl NulTerminated {
    /// # Safety
    ///
    /// `p` must point into a NUL-terminated string, and the text is read
    /// only by the readers in this file (see the module docs).
    #[inline(always)]
    pub(super) const unsafe fn new(p: *const c_char) -> Self {
        Self(p.cast::<u8>())
    }
}

impl Text for NulTerminated {
    #[inline(always)]
    fn byte(self, i: usize) -> u8 {
        // SAFETY: the readers in this file read only up to the NUL; see the
        // module docs.
        unsafe { *self.0.add(i) }
    }

    #[inline(always)]
    fn ends_at(self, i: usize) -> bool {
        self.byte(i) == 0
    }

    #[inline(always)]
    fn left(self, _i: usize) -> Option<usize> {
        None
    }
}

/// The first `len` bytes of a text, ended early by a NUL in them: the span a
/// pointer form with a length (`utfc_ptr2len_len(p, n)`) looks at.
#[derive(Clone, Copy)]
pub(super) struct Prefix<T> {
    text: T,
    from: usize,
    to: usize,
}

impl<T: Text> Prefix<T> {
    /// `text[from..to]`, ending at a NUL inside it.
    #[inline(always)]
    pub(super) fn new(text: T, from: usize, to: usize) -> Self {
        Self { text, from, to }
    }
}

impl<T: Text> Text for Prefix<T> {
    #[inline(always)]
    fn byte(self, i: usize) -> u8 {
        if self.from + i < self.to {
            self.text.byte(self.from + i)
        } else {
            0
        }
    }

    #[inline(always)]
    fn ends_at(self, i: usize) -> bool {
        self.byte(i) == 0
    }

    #[inline(always)]
    fn left(self, i: usize) -> Option<usize> {
        Some(self.to.saturating_sub(self.from + i))
    }
}

/// The sequence length the lead byte at `i` announces.
#[inline(always)]
fn announced<T: Text>(t: T, i: usize) -> usize {
    usize::from(utf8len_tab[usize::from(t.byte(i))])
}

/// How many bytes the character at `i` occupies, 0 where the text ends, and
/// 1 for anything that is not a complete sequence -- so a walk always makes
/// progress and never steps past the end.
#[inline(always)]
pub(super) fn char_len_in<T: Text>(t: T, i: usize) -> usize {
    if t.ends_at(i) {
        return 0;
    }
    let len = announced(t, i);
    if (1..len).all(|k| utf_is_trail_byte(t.byte(i + k))) {
        len
    } else {
        1
    }
}

/// The length the lead byte at `i` announces, unless a byte the text holds
/// says it is no such sequence, in which case 1. Only a text that knows its
/// length can tell this from [`char_len_in`]: a sequence it cuts short is
/// reported at its full length.
#[inline(always)]
pub(super) fn promised_len_in<T: Text>(t: T, i: usize) -> usize {
    if t.left(i) == Some(0) {
        return 0;
    }
    let len = announced(t, i);
    if len == 1 {
        return 1;
    }
    let held = t.left(i).map_or(len, |left| len.min(left));
    if (1..held).all(|k| utf_is_trail_byte(t.byte(i + k))) {
        len
    } else {
        1
    }
}

/// Whether the sequence at `i` is cut short by the end of a text that knows
/// its length. A NUL-terminated string never says so: there, a sequence its
/// NUL cuts short is simply not one.
#[inline(always)]
fn cut_short<T: Text>(t: T, i: usize) -> bool {
    t.left(i).is_some_and(|left| promised_len_in(t, i) > left)
}

/// The codepoint at `i`, or the first byte's own value if there is no
/// complete sequence there; 0 where the text ends.
#[inline(always)]
pub(super) fn char_at_in<T: Text>(t: T, i: usize) -> i32 {
    let first = t.byte(i);
    let unchanged = i32::from(first);
    if first < 0x80 {
        return unchanged;
    }
    let len = announced(t, i);
    // A continuation byte or 0xFE/0xFF is not a lead byte at all.
    if len < 2 {
        return unchanged;
    }
    let mut code_point = u32::from(first) & LEAD_PAYLOAD[len];
    for k in 1..len {
        let cur = t.byte(i + k);
        if !utf_is_trail_byte(cur) {
            return unchanged;
        }
        code_point = (code_point << 6) | (u32::from(cur) & 0x3f);
    }
    // Six bytes carry 31 bits, so the value always fits.
    code_point.cast_signed()
}

/// [`utf_ptr2char_info_impl`](super::utf_ptr2char_info_impl)'s decode at `i`
/// for a sequence of `len` bytes: negative for anything that is not one,
/// including a `len` of 0 or 1. The second byte is read whatever `len` says,
/// as the pointer form reads it; a byte at or above 0x80 is never a text's
/// last.
#[inline(always)]
fn decode_in<T: Text>(t: T, i: usize, len: usize) -> i32 {
    let read = if len < 3 { 2 } else { len };
    let mut code_point = u32::from(t.byte(i));
    for k in 1..read {
        let cur = t.byte(i + k);
        if !utf_is_trail_byte(cur) {
            return -1;
        }
        code_point = (code_point << 6).wrapping_add(u32::from(cur));
    }
    code_point.wrapping_add(CORRECTIONS[len]).cast_signed()
}

/// The codepoint at `i`, or a negative number when there is no character
/// there -- an incomplete sequence, a bad continuation byte, a byte that leads
/// nothing, or the end of a slice. ASCII, a NUL included, is itself.
///
/// Which negative number is the one place the two texts part ways beyond the
/// end: a slice answers -1, as its form always has, and a NUL-terminated
/// string answers whatever [`utf_ptr2char_info_impl`]'s arithmetic leaves
/// for a byte that leads nothing, as the pointer form always has.
///
/// [`utf_ptr2char_info_impl`]: super::utf_ptr2char_info_impl
#[inline(always)]
pub(super) fn strict_char_at_in<T: Text>(t: T, i: usize) -> i32 {
    if t.left(i) == Some(0) {
        return -1;
    }
    let first = t.byte(i);
    if first < 0x80 {
        return i32::from(first);
    }
    let len = announced(t, i);
    if t.left(i).is_some() && (len < 2 || cut_short(t, i)) {
        return -1;
    }
    decode_in(t, i, len)
}

/// The codepoint at `i` and the bytes it occupies; a byte that is not a
/// character reports its negative value with a length of one.
#[inline(always)]
pub(super) fn char_info_in<T: Text>(t: T, i: usize) -> CharInfo {
    let value = strict_char_at_in(t, i);
    if value < 0 {
        return CharInfo { value, len: 1 };
    }
    CharInfo {
        value,
        len: i32::from(utf8len_tab[usize::from(t.byte(i))]),
    }
}

/// Is the character at `next` part of the same grapheme cluster as the one at
/// `cur`?
#[inline(always)]
pub(super) fn composes_in<T: Text>(
    t: T,
    cur: usize,
    next: usize,
    state: Option<&mut GraphemeState>,
) -> bool {
    // ASCII never combines, and this is the hot answer.
    t.byte(next) >= 0x80 && iscomposing(char_at_in(t, cur), char_at_in(t, next), state)
}

/// How many bytes the grapheme cluster at `i` occupies -- the base character
/// plus every composing character following it -- or 0 where the text ends.
///
/// A slice's embedded NUL is a character of its own, one byte long. A
/// composing character the end of a slice cuts short is left out, because the
/// rest of it may still arrive; a NUL-terminated string's NUL cuts nothing
/// short, it is just a byte that is not a continuation.
#[inline(always)]
pub(super) fn cluster_len_in<T: Text>(t: T, i: usize) -> usize {
    if t.ends_at(i) {
        return 0;
    }
    let first = t.byte(i);
    if first == 0 {
        return 1; // an embedded NUL is an ordinary byte in a slice
    }
    // Two ASCII bytes: nothing can be combining, answer without decoding.
    if first < 0x80 && t.byte(i + 1) < 0x80 {
        return 1;
    }
    let mut len = char_len_in(t, i);
    if len == 1 && first >= 0x80 {
        return 1;
    }
    let mut prev = 0;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT;
    loop {
        let next = i + len;
        // ASCII never combines, and this is the hot answer -- asked before
        // anything is decoded. A text's end reads as ASCII.
        if t.byte(next) < 0x80 || cut_short(t, next) {
            return len;
        }
        if !composes_in(t, i + prev, next, Some(&mut state)) {
            return len;
        }
        prev = len;
        len += char_len_in(t, next);
    }
}

/// Does a cluster always end at this character, whatever precedes it?
#[inline(always)]
fn always_break(bc: i32) -> bool {
    bc == UTF8PROC_BOUNDCLASS_CONTROL
}

/// Do these two boundclasses always break between them?
///
/// The cheap half of the grapheme-cluster rules, answered from the two
/// boundclasses alone: `utf8proc_grapheme_break` is the full test, and this
/// is what [`head_off_in`] can use while walking *backwards*, where the
/// stateful one cannot be run.
#[inline(always)]
fn always_break_two(bc1: i32, bc2: i32) -> bool {
    (bc1 != UTF8PROC_BOUNDCLASS_PREPEND && bc2 == UTF8PROC_BOUNDCLASS_OTHER)
        || (UTF8PROC_BOUNDCLASS_CR..=UTF8PROC_BOUNDCLASS_CONTROL).contains(&bc1)
        || (bc2 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC
            && (bc1 == UTF8PROC_BOUNDCLASS_OTHER
                || bc1 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC))
}

/// The boundclass of a decoded codepoint.
#[inline(always)]
fn boundclass(code: i32) -> i32 {
    i32::from(utf8proc_get_property(code).boundclass)
}

/// How far back from byte `at` the grapheme cluster covering it starts.
///
/// Three answers, in the order they are looked for: 0 if the byte is ASCII or
/// part of an illegal sequence (an illegal byte is its own character); the
/// offset to the lead byte if the character it starts always breaks; and
/// otherwise the result of backtracking over the cluster and then walking
/// *forwards* through it, because a backwards walk can overshoot and only a
/// forwards one knows where each cluster really ends.
///
/// Bytes before `at` are read back to the start of the text or to a NUL
/// before it; forwards, nothing past the character `at` sits in.
pub(super) fn head_off_in<T: Text>(t: T, at: usize) -> usize {
    if t.byte(at) < 0x80 {
        return 0;
    }

    // Walk back over continuation bytes to the lead byte of a sequence,
    // stopping at the start or after a sequence's worth. It may stop *on* a
    // continuation byte when the sequence is overlong; the caller finds that
    // out by decoding.
    let step_back_to_lead = |mut from: usize, measure_from: usize| {
        while from > 0 && utf_is_trail_byte(t.byte(from)) && measure_from - from < MAX_SEQUENCE {
            from -= 1;
        }
        from
    };

    let mut start = step_back_to_lead(at, at);
    let last_len = announced(t, start);
    let mut cur_code = decode_in(t, start, last_len);
    if cur_code < 0 || at - start >= last_len {
        return 0; // `at` is part of an illegal sequence
    }
    // Nothing past the character `at` sits in may be read while scanning
    // forwards: beyond it the bytes have not been validated.
    let safe_end = start + last_len;

    let mut cur_bc = boundclass(cur_code);
    if always_break(cur_bc) || start == 0 {
        return at - start;
    }

    // Backtrack over the cluster. This can go too far -- the backwards walk
    // cannot see where a cluster *ends* -- which the forwards scan below
    // corrects. `start` is past the beginning on every test of the loop.
    let at_start = start;
    let mut cur_pos = start;
    while t.byte(start - 1) != 0 {
        start -= 1;
        if t.byte(start) < 0x80 {
            break; // ASCII never combines: done
        }
        start = step_back_to_lead(start, cur_pos);

        let prev_len = announced(t, start);
        let prev_code = decode_in(t, start, prev_len);
        if prev_code < 0 || prev_len < cur_pos - start {
            start = cur_pos; // resume at the valid sequence after the junk
            break;
        }
        let prev_bc = boundclass(prev_code);
        if always_break_two(prev_bc, cur_bc) && !crate::arabic::arabic_combine(prev_code, cur_code)
        {
            start = cur_pos; // the previous character is not in this cluster
            break;
        }
        if start == 0 {
            break;
        }
        cur_pos = start;
        cur_bc = prev_bc;
        cur_code = prev_code;
    }

    // Never moved: `at` is inside the first character looked at.
    if start == at_start && last_len > at - start {
        return at - start;
    }

    let mut q = start;
    while q < at {
        // Where the cluster *ends* does not matter; reaching `at`'s
        // codepoint is enough.
        let len = cluster_len_in(Prefix::new(t, q, safe_end), 0);
        if q + len > at {
            return at - q;
        }
        q += len;
    }
    0
}

/// The character after `cur`, which sits at `at`, treating a following
/// composing character as part of `cur`: the offset it starts at, and the
/// character itself.
#[inline(always)]
pub(super) fn next_cluster_in<T: Text>(t: T, at: usize, cur: CharInfo) -> (usize, CharInfo) {
    // A character is at least one byte long, so the length is positive.
    let next = at + cur.len.unsigned_abs() as usize;
    let ascii = |next: usize| {
        let value = i32::from(t.byte(next));
        (next, CharInfo { value, len: 1 })
    };
    if t.byte(next) < 0x80 {
        return ascii(next);
    }
    next_cluster_slow(t, next, cur.value)
}

/// The slow half of [`next_cluster_in`]: the byte at `next` is not ASCII, so
/// each following character has to be decoded to find out whether it
/// composes onto this one.
///
/// A sequence that is not a character is one byte long, and a walk steps
/// over it by one: stepping by the length its lead byte announced, as
/// upstream does, would cross a NUL that cut it short when a Prepend mark
/// before it makes it "compose".
fn next_cluster_slow<T: Text>(t: T, mut next: usize, mut prev_code: i32) -> (usize, CharInfo) {
    let mut state: GraphemeState = GRAPHEME_STATE_INIT;
    loop {
        let next_len = announced(t, next);
        let next_code = decode_in(t, next, next_len);
        let len = if next_code < 0 { 1 } else { next_len };
        if !iscomposing(prev_code, next_code, Some(&mut state)) {
            let len = i32::try_from(len).expect("a sequence is short");
            return (
                next,
                CharInfo {
                    value: next_code,
                    len,
                },
            );
        }
        prev_code = next_code;
        next += len;
        if t.byte(next) < 0x80 {
            let value = i32::from(t.byte(next));
            return (next, CharInfo { value, len: 1 });
        }
    }
}

/// How many cells the character at `i` occupies. Where the text ends that is
/// 1: nothing to print still occupies the cell the cursor sits in.
#[inline(always)]
pub(super) fn cells_in<T: Text>(t: T, i: usize) -> i32 {
    let first = t.byte(i);
    if first < 0x80 {
        return 1;
    }
    let len = announced(t, i);
    let c = if cut_short(t, i) {
        -1
    } else {
        decode_in(t, i, len)
    };
    // An illegal byte, an incomplete sequence, or an overlong encoding of
    // NUL: all displayed as <xx>.
    if c <= 0 {
        return 4;
    }
    // An ASCII answer from a multibyte lead byte means an overlong sequence,
    // which is displayed the way that ASCII character is.
    if c < 0x80 {
        return char2cells(c);
    }
    let cells = utf_char2cells(c);
    if cells == 1 && emoji_widens(c) && char_at_in(t, i + len) == VS16 {
        return 2;
    }
    cells
}

/// The total width of the text from `i` to its end.
#[inline]
pub(super) fn string_cells_in<T: Text>(t: T, mut i: usize) -> usize {
    let mut cells = 0;
    while !t.ends_at(i) {
        // A width is never negative, and a cluster is never empty here.
        cells += cells_in(t, i).unsigned_abs() as usize;
        i += cluster_len_in(t, i);
    }
    cells
}

/// Might the character at `i` be drawn at a width the grid did not predict?
/// An ambiguous-width character answers whatever `'ambiwidth'` says, an
/// emoji whatever `'emoji'` says, and a VS-16 after any character turns it
/// into an emoji.
#[inline]
pub(super) fn ambiguous_width_in<T: Text>(t: T, i: usize) -> bool {
    // Nothing to print, or a lone ASCII character: neither can move.
    if t.ends_at(i) || t.ends_at(i + 1) {
        return false;
    }
    let info = char_info_in(t, i);
    if info.value >= 0x80 {
        let prop = utf8proc_get_property(info.value);
        if prop.ambiguous_width || prop_is_emojilike(prop) {
            return true;
        }
    }
    // VS-16, compared a byte at a time: the first byte that differs stops
    // the comparison, and a NUL differs from all three.
    let after = i + usize::try_from(info.len).expect("a length is positive");
    b"\xef\xb8\x8f"
        .iter()
        .enumerate()
        .all(|(k, &want)| t.byte(after + k) == want)
}

/// The codepoint at `i` and how many of the `left` bytes there it occupies.
///
/// A single byte is taken at face value rather than decoded, which is what
/// makes an invalid byte count as one character. A sequence `left` cuts
/// short still reports its full length, and is decoded out of the bytes
/// past `left` -- which is how a four-byte character straddling the cut is
/// still counted as a UTF-16 surrogate pair, as
/// `test/functional/lua/vim_spec.lua`'s `str_utfindex` table requires.
#[inline(always)]
fn char_and_len_in<T: Text>(t: T, i: usize, left: usize) -> (i32, usize) {
    let len = promised_len_in(Prefix::new(t, i, i + left.max(1)), 0);
    let c = if len > 1 {
        char_at_in(t, i)
    } else {
        i32::from(t.byte(i))
    };
    (c, len)
}

/// How many codepoints, and how many UTF-16 code units, the first `len` bytes
/// of the text hold. The two differ by the surrogate pairs: a character above
/// the BMP is one codepoint but two UTF-16 code units.
pub(super) fn utf_lengths_in<T: Text>(t: T, len: usize) -> (usize, usize) {
    let (mut points, mut pairs, mut i) = (0, 0, 0);
    while i < len {
        let (c, clen) = char_and_len_in(t, i, len - i);
        points += 1;
        if c > 0xffff {
            pairs += 1;
        }
        i += clen;
    }
    (points, points + pairs)
}

/// The byte offset just past the character at `index` among the first `len`
/// bytes of the text, counted in codepoints or in UTF-16 code units; `None`
/// when they hold fewer.
pub(super) fn utf_index_to_byte_in<T: Text>(
    t: T,
    len: usize,
    index: usize,
    use_utf16_units: bool,
) -> Option<usize> {
    if index == 0 {
        return Some(0);
    }
    let (mut count, mut i) = (0, 0);
    while i < len {
        let (c, clen) = char_and_len_in(t, i, len - i);
        count += 1;
        if use_utf16_units && c > 0xffff {
            count += 1;
        }
        if count >= index {
            return Some(i + clen);
        }
        i += clen;
    }
    None
}

/// How many cells the character at `at` occupies, out of a buffer `size`
/// bytes long from the text's start: `utf_ptr2cells_len`'s answer.
///
/// Upstream's shape, kept exactly: a sequence with a bad continuation byte
/// inside the buffer answers 1 (upstream's comment calls that "truncated"),
/// and a sequence the buffer's end cuts short is decoded out of the bytes
/// past it -- a NUL there makes it the illegal byte it then is, drawn as
/// `<xx>`. A VS-16 widens the character only when it is complete inside the
/// buffer.
pub(super) fn partial_cells_in<T: Text>(t: T, at: usize, size: usize) -> i32 {
    let first = t.byte(at);
    if size <= at || first < 0x80 {
        return 1;
    }
    let lead_len = announced(t, at);
    let len = promised_len_in(Prefix::new(t, at, size), 0);
    if len < lead_len {
        return 1;
    }
    let c = char_at_in(t, at);
    // An illegal byte is displayed as <xx>.
    if char_len_in(t, at) == 1 || c == 0 {
        return 4;
    }
    if c < 0x80 {
        return char2cells(c);
    }
    let cells = utf_char2cells(c);
    let next = at + len;
    if size > next
        && promised_len_in(Prefix::new(t, next, size), 0) == announced(t, next)
        && cells == 1
        && emoji_widens(c)
        && char_at_in(t, next) == VS16
    {
        return 2;
    }
    cells
}

/// The total width of the first `size` bytes of the text, stopping early at
/// a NUL, each character measured by [`partial_cells_in`].
pub(super) fn partial_string_cells_in<T: Text>(t: T, size: usize) -> usize {
    let mut cells = 0;
    let mut at = 0;
    while at < size && t.byte(at) != 0 {
        cells += partial_cells_in(t, at, size).unsigned_abs() as usize;
        at += cluster_len_in(Prefix::new(t, at, size), 0);
    }
    cells
}

/// The display width of the characters that start in the first `len` bytes
/// of the text -- one that starts inside the bound is counted whole --
/// stopping where the text ends: `vim_strnsize`'s answer. A byte below 0x80
/// is as wide as `'isprint'`'s table says.
pub(super) fn display_width_in<T: Text>(t: T, mut len: i32) -> i32 {
    let mut at = 0;
    let mut size = 0;
    while !t.ends_at(at) && {
        len -= 1;
        len >= 0
    } {
        let l = cluster_len_in(t, at);
        let first = t.byte(at);
        size += if first >= 0x80 {
            cells_in(t, at)
        } else {
            crate::charset::byte2cells(i32::from(first))
        };
        at += l;
        len -= i32::try_from(l).expect("a cluster is short") - 1;
    }
    size
}
