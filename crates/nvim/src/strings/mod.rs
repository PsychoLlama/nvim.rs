//! The string helpers, over slices.
//!
//! Every function here takes the bytes of a string and answers an owned one
//! or an offset. The pointer spellings the rest of the tree still calls are
//! one-line shims in `raw.rs`, the one file of `strings` that reads through a
//! pointer; a family whose shim file must allow `unsafe` keeps this `mod.rs`
//! at `deny(unsafe_op_in_unsafe_fn)` (a `forbid` here would reach it).

#![deny(unsafe_op_in_unsafe_fn)]

use crate::eval::typval::tv_get_bool_chk;
use crate::keycodes::Ctrl_V;
use crate::mbyte::{cluster_len, encode_char};
use crate::memory::XString;
use crate::semsg;
use crate::types::{MB_MAXCHAR, TypVal};
use core::ffi::{CStr, c_int};

// The carve of the transpiled module; see each child's docs.
mod case;
mod charindex;
mod escape;
mod eval;
mod printf;
mod raw;

pub use self::case::*;
pub use self::charindex::*;
pub use self::escape::*;
pub use self::eval::*;
pub use self::printf::*;
pub use self::raw::*;

/// Read an optional boolean argument that must be spelled `0` or `1`.
///
/// Returns `None` after raising the error, which both callers turn into a
/// silent `-1` result.
pub(crate) fn strict_bool_arg(tv: &TypVal) -> Option<bool> {
    let Ok(value) = tv_get_bool_chk(tv) else {
        return None;
    };
    if !(0..=1).contains(&value) {
        semsg!("E1023: Using a Number as a Bool: {}", value);
        return None;
    }
    Some(value != 0)
}

/// Any byte outside 7-bit ASCII?
pub(crate) fn any_non_ascii(s: &[u8]) -> bool {
    s.iter().any(|&b| b >= 0x80)
}

/// Index where removable trailing whitespace starts: spaces/tabs not
/// preceded by a backslash or Ctrl-V, never including the first byte.
/// Upstream's `del_trailing_spaces` zero-fills from here.
pub fn trailing_spaces_start(s: &[u8]) -> usize {
    let mut end = s.len();
    while end > 1
        && matches!(s[end - 1], b' ' | b'\t')
        && s[end - 2] != b'\\'
        && s[end - 2] != Ctrl_V as u8
    {
        end -= 1;
    }
    end
}

/// The byte offset of codepoint `c` in `haystack`, or `None`.
///
/// `c` is a *codepoint*, not a byte: below `0x80` this is a byte search,
/// and above it the needle is `c`'s UTF-8 encoding, so the answer is where
/// that whole sequence starts. A non-positive `c` is never found — upstream
/// spells `strchr(s, 0)` when it wants the terminator, and that offset is
/// `haystack.len()`, which this deliberately does not answer.
pub(crate) fn find_char(haystack: &[u8], c: c_int) -> Option<usize> {
    if c <= 0 {
        return None;
    }
    let mut encoded = [0u8; MB_MAXCHAR];
    let len = encode_char(c, &mut encoded);
    if len == 1 {
        let byte = encoded[0];
        haystack.iter().position(|&b| b == byte)
    } else {
        haystack
            .windows(len)
            .position(|window| window == &encoded[..len])
    }
}

/// Whether codepoint `c` occurs in `set`.
///
/// The membership half of [`find_char`], and by a wide margin what
/// upstream's `vim_strchr()` is for: two thirds of its call sites are a
/// `!= NULL` test over an option's value or a literal run of characters,
/// and none of those wants a pointer.
pub(crate) fn has_char(set: &CStr, c: c_int) -> bool {
    find_char(set.to_bytes(), c).is_some()
}

/// The byte offset at which `needle` occurs in `haystack`, or `None`.
///
/// The safe form of C's `strstr()`, its empty-needle answer included: every
/// string starts with the empty one, so an empty needle is found at 0.
pub(crate) fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Whether `needle` occurs anywhere in `haystack` — the membership half of
/// [`find_bytes`], as [`has_char`] is of [`find_char`].
pub(crate) fn has_bytes(haystack: &CStr, needle: &[u8]) -> bool {
    find_bytes(haystack.to_bytes(), needle).is_some()
}

/// `a ++ b`, owned; upstream's `concat_str`.
pub fn concat(a: &[u8], b: &[u8]) -> XString {
    let mut out = Vec::with_capacity(a.len() + b.len() + 1);
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    XString::from_vec(out)
}

/// `text` reversed character by character.
///
/// Composing sequences move as a unit — [`cluster_len`] gives the length of
/// the whole character at each position — so the source is walked forwards
/// while the destination is filled from the back.
pub fn reversed_text(text: &[u8]) -> Vec<u8> {
    let len = text.len();
    let mut out = vec![0; len];
    let mut at = len;
    let mut i = 0;
    while i < len {
        let char_len = cluster_len(&text[i..]);
        at -= char_len;
        out[at..at + char_len].copy_from_slice(&text[i..i + char_len]);
        i += char_len;
    }
    out
}

/// Every occurrence of `what` in `src` replaced by `rep`, or `None` when
/// `what` does not occur at all; upstream's `strrep`. An empty `what` --
/// which upstream loops forever on -- occurs nowhere.
pub fn replaced_all(src: &[u8], what: &[u8], rep: &[u8]) -> Option<XString> {
    if what.is_empty() || find_bytes(src, what).is_none() {
        return None;
    }
    let mut out = Vec::with_capacity(src.len() + rep.len());
    let mut rest = src;
    while let Some(at) = find_bytes(rest, what) {
        out.extend_from_slice(&rest[..at]);
        out.extend_from_slice(rep);
        rest = &rest[at + what.len()..];
    }
    out.extend_from_slice(rest);
    Some(XString::from_vec(out))
}

#[cfg(test)]
mod tests {
    use super::{
        any_non_ascii, find_bytes, find_char, has_bytes, has_char, trailing_spaces_start, unquote,
    };

    #[test]
    fn find_char_answers_a_byte_offset_and_never_the_terminator() {
        assert_eq!(find_char(b"abc", i32::from(b'a')), Some(0));
        assert_eq!(find_char(b"abc", i32::from(b'c')), Some(2));
        assert_eq!(find_char(b"abc", i32::from(b'z')), None);
        // `strchr(s, 0)` answers the terminator; the slice has none.
        assert_eq!(find_char(b"abc", 0), None);
        assert_eq!(find_char(b"abc", -1), None);
        // An empty set has no members, and a NUL *inside* the bytes is a
        // member like any other -- which the pointer form could not see.
        assert_eq!(find_char(b"", i32::from(b'a')), None);
        assert_eq!(find_char(b"a\0b", 0), None);
        assert_eq!(find_char(b"a\0b", i32::from(b'b')), Some(2));
    }

    #[test]
    fn find_char_matches_a_multibyte_character_whole() {
        // The needle above 0x7f is the character's UTF-8 encoding, so the
        // offset is where the sequence starts and a lone lead byte misses.
        assert_eq!(find_char("a\u{ab}b".as_bytes(), 0xAB), Some(1));
        assert_eq!(
            find_char("\u{201e}\u{ab}\u{bb}\u{201c}".as_bytes(), 0xAB),
            Some(3)
        );
        assert_eq!(
            find_char("\u{201e}\u{ab}\u{bb}\u{201c}".as_bytes(), 0x201C),
            Some(7)
        );
        // 0xC2 is '\u{ab}''s first byte, and is not itself in the string.
        assert_eq!(find_char("\u{ab}\u{bb}".as_bytes(), 0xC2), None);
        // A lone 0xAB byte is not valid UTF-8 for U+00AB.
        assert_eq!(find_char(b"\xAB", 0xAB), None);
        // A needle longer than the haystack cannot match.
        assert_eq!(find_char(b"a", 0x201C), None);
    }

    #[test]
    fn has_char_is_find_char_over_a_c_string() {
        assert!(has_char(c"abc", i32::from(b'b')));
        assert!(!has_char(c"abc", i32::from(b'z')));
        assert!(!has_char(c"", i32::from(b'a')));
        assert!(!has_char(c"abc", 0)); // the terminator is not a member
        assert!(has_char(c"a\u{ab}b", 0xAB));
    }

    #[test]
    fn find_bytes_is_strstr_over_slices() {
        assert_eq!(find_bytes(b"abcabc", b"ca"), Some(2));
        assert_eq!(find_bytes(b"abc", b"abc"), Some(0));
        assert_eq!(find_bytes(b"abc", b"abcd"), None); // needle longer
        assert_eq!(find_bytes(b"", b"a"), None);
        // Every string starts with the empty one, as `strstr` answers.
        assert_eq!(find_bytes(b"abc", b""), Some(0));
        assert_eq!(find_bytes(b"", b""), Some(0));
        // A NUL is an ordinary byte here, which is where a slice and a C
        // string part company.
        assert_eq!(find_bytes(b"a\0b", b"\0b"), Some(1));
    }

    #[test]
    fn has_bytes_stops_at_the_terminator() {
        assert!(has_bytes(c"runtime/after", b"after"));
        assert!(!has_bytes(c"runtime", b"after"));
        assert!(has_bytes(c"", b""));
    }

    fn unquote_all(src: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        unquote(src, &mut |b| out.push(b));
        out
    }

    #[test]
    fn unquote_mirrors_the_unit_spec_cases() {
        assert_eq!(unquote_all(b"abc"), b"abc"); // unquoted copies as-is
        assert_eq!(unquote_all(br#""abc""#), b"abc"); // fully quoted word
        assert_eq!(unquote_all(br#"a"b"c"#), b"abc"); // partially quoted
        assert_eq!(unquote_all(br#"a""b"#), b"ab"); // removes ""
        assert_eq!(unquote_all(br#""a\"b""#), br#"a"b"#); // unescapes \"
        assert_eq!(unquote_all(br#""a\\b""#), br#"a\b"#); // unescapes doubled backslash
        assert_eq!(unquote_all(br#"a\\b"#), br#"a\\b"#); // but not outside quotes
        assert_eq!(unquote_all(br#""a\nb""#), br#"a\nb"#); // \n is not unescaped
        assert_eq!(unquote_all(br#""abc"#), b"abc"); // unpaired quote stripped
        assert_eq!(unquote_all(br#"a\"#), br#"a\"#); // may end with one backslash
    }

    #[test]
    fn trailing_spaces_respect_escapes_and_never_take_byte_zero() {
        assert_eq!(trailing_spaces_start(b"ab  "), 2);
        assert_eq!(trailing_spaces_start(b"ab\t "), 2);
        assert_eq!(trailing_spaces_start(b"ab\\  "), 4); // escaped space stays
        assert_eq!(trailing_spaces_start(&[b'a', 22, b' ']), 3); // Ctrl-V escapes
        assert_eq!(trailing_spaces_start(b" "), 1); // first byte never stripped
        assert_eq!(trailing_spaces_start(b""), 0);
    }

    #[test]
    fn ascii_scan() {
        assert!(any_non_ascii(b"caf\xc3\xa9"));
        assert!(!any_non_ascii(b"cafe"));
    }
}
