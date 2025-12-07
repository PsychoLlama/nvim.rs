//! Multibyte/UTF-8 utilities for Neovim
//!
//! Provides UTF-8 encoding/decoding functions compatible with nvim's mbyte.c.
//!
//! Key functions:
//! - `utf_ptr2char` - Decode UTF-8 byte sequence to Unicode codepoint
//! - `utf_char2bytes` - Encode Unicode codepoint to UTF-8 bytes
//! - `utf_char2len` - Get byte length of UTF-8 encoding for a codepoint
//! - `utf_ptr2len` - Get byte length of UTF-8 character in a string
//! - `utf_byte2len` - Get expected UTF-8 length from first byte

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::uninlined_format_args)]

use std::ffi::{c_char, c_int, c_longlong};

use nvim_utf8proc::{casefold, get_property, grapheme_break, grapheme_break_stateful, Utf8procProperty};

/// Maximum bytes in a UTF-8 character (standard is 4, but nvim supports up to 6).
pub const MB_MAXBYTES: usize = 6;

/// Lookup table for UTF-8 byte sequence length based on first byte.
/// Bytes 0x80-0xBF (continuation bytes) and 0xFE-0xFF (invalid) return 1.
#[rustfmt::skip]
pub static UTF8LEN_TAB: [u8; 256] = [
    //  ?1 ?2 ?3 ?4 ?5 ?6 ?7 ?8 ?9 ?A ?B ?C ?D ?E ?F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 1?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 2?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 3?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 4?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 5?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 6?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 7?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 8? (continuation bytes)
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 9?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // A?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // B?
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,  // C? (2-byte sequences)
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,  // D?
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,  // E? (3-byte sequences)
    4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 1, 1,  // F? (4-6 byte, FE/FF invalid)
];

/// Lookup table for UTF-8 byte sequence length, with 0 for continuation/invalid bytes.
#[rustfmt::skip]
pub static UTF8LEN_TAB_ZERO: [u8; 256] = [
    //  ?1 ?2 ?3 ?4 ?5 ?6 ?7 ?8 ?9 ?A ?B ?C ?D ?E ?F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 1?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 2?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 3?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 4?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 5?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 6?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 7?
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 8? (continuation bytes -> 0)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 9?
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // A?
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // B?
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,  // C?
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,  // D?
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,  // E?
    4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 0, 0,  // F? (FE/FF -> 0)
];

/// Get the byte length of a UTF-8 encoding for a Unicode codepoint.
///
/// Returns 1-6 depending on the codepoint value.
#[inline]
pub fn utf_char2len(c: i32) -> usize {
    if c < 0x80 {
        1
    } else if c < 0x800 {
        2
    } else if c < 0x1_0000 {
        3
    } else if c < 0x20_0000 {
        4
    } else if c < 0x400_0000 {
        5
    } else {
        6
    }
}

/// Convert Unicode codepoint to UTF-8 byte sequence.
///
/// Writes the UTF-8 encoding to `buf` (must have room for at least 6 bytes).
/// Returns the number of bytes written (1-6).
#[inline]
pub fn utf_char2bytes(c: i32, buf: &mut [u8]) -> usize {
    let c = c as u32;
    if c < 0x80 {
        buf[0] = c as u8;
        1
    } else if c < 0x800 {
        buf[0] = (0xC0 | (c >> 6)) as u8;
        buf[1] = (0x80 | (c & 0x3F)) as u8;
        2
    } else if c < 0x1_0000 {
        buf[0] = (0xE0 | (c >> 12)) as u8;
        buf[1] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[2] = (0x80 | (c & 0x3F)) as u8;
        3
    } else if c < 0x20_0000 {
        buf[0] = (0xF0 | (c >> 18)) as u8;
        buf[1] = (0x80 | ((c >> 12) & 0x3F)) as u8;
        buf[2] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[3] = (0x80 | (c & 0x3F)) as u8;
        4
    } else if c < 0x400_0000 {
        buf[0] = (0xF8 | (c >> 24)) as u8;
        buf[1] = (0x80 | ((c >> 18) & 0x3F)) as u8;
        buf[2] = (0x80 | ((c >> 12) & 0x3F)) as u8;
        buf[3] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[4] = (0x80 | (c & 0x3F)) as u8;
        5
    } else {
        buf[0] = (0xFC | (c >> 30)) as u8;
        buf[1] = (0x80 | ((c >> 24) & 0x3F)) as u8;
        buf[2] = (0x80 | ((c >> 18) & 0x3F)) as u8;
        buf[3] = (0x80 | ((c >> 12) & 0x3F)) as u8;
        buf[4] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[5] = (0x80 | (c & 0x3F)) as u8;
        6
    }
}

/// Get the expected UTF-8 sequence length from the first byte.
///
/// Returns 1 for ASCII, continuation bytes, or invalid bytes.
#[inline]
pub fn utf_byte2len(b: u8) -> usize {
    UTF8LEN_TAB[b as usize] as usize
}

/// Decode UTF-8 byte sequence to Unicode codepoint.
///
/// Returns the codepoint value.
/// For invalid sequences, returns the first byte value.
pub fn utf_ptr2char(p: &[u8]) -> i32 {
    if p.is_empty() {
        return 0;
    }

    let v0 = p[0] as u32;

    // Fast path for ASCII
    if v0 < 0x80 {
        return v0 as i32;
    }

    let len = UTF8LEN_TAB[v0 as usize];
    if len < 2 || p.len() < len as usize {
        return v0 as i32;
    }

    // Check continuation bytes
    macro_rules! check_cont {
        ($v:expr) => {
            if ($v & 0xC0) != 0x80 {
                return v0 as i32;
            }
        };
    }

    let v1 = p[1] as u32;
    check_cont!(v1);

    if len == 2 {
        return ((v0 << 6) + v1 - ((0xC0 << 6) + 0x80)) as i32;
    }

    let v2 = p[2] as u32;
    check_cont!(v2);

    if len == 3 {
        return ((v0 << 12) + (v1 << 6) + v2 - ((0xE0 << 12) + (0x80 << 6) + 0x80)) as i32;
    }

    let v3 = p[3] as u32;
    check_cont!(v3);

    if len == 4 {
        return ((v0 << 18) + (v1 << 12) + (v2 << 6) + v3
            - ((0xF0 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80)) as i32;
    }

    let v4 = p[4] as u32;
    check_cont!(v4);

    if len == 5 {
        return ((v0 << 24) + (v1 << 18) + (v2 << 12) + (v3 << 6) + v4
            - ((0xF8 << 24) + (0x80 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80))
            as i32;
    }

    // len == 6
    let v5 = p[5] as u32;
    check_cont!(v5);

    ((v0 << 30) + (v1 << 24) + (v2 << 18) + (v3 << 12) + (v4 << 6) + v5
        - ((0x80 << 24) + (0x80 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80)) as i32
}

/// Get the byte length of a UTF-8 character in a string.
///
/// Does not include composing characters.
/// Returns 0 for empty string (NUL byte).
/// Returns 1 for invalid sequences.
pub fn utf_ptr2len(p: &[u8]) -> usize {
    if p.is_empty() || p[0] == 0 {
        return 0;
    }

    let len = UTF8LEN_TAB[p[0] as usize] as usize;
    if len > p.len() {
        return 1;
    }

    // Verify continuation bytes
    for i in 1..len {
        if (p[i] & 0xC0) != 0x80 {
            return 1;
        }
    }

    len
}

/// Get the byte length of a UTF-8 character with size limit.
///
/// Returns 1 for empty string, invalid sequences, or incomplete sequences.
/// Returns length > size for incomplete sequences.
pub fn utf_ptr2len_len(p: &[u8], size: usize) -> usize {
    if size < 1 || p.is_empty() {
        return 1;
    }

    let len = UTF8LEN_TAB[p[0] as usize] as usize;
    if len == 1 {
        return 1;
    }

    let check = if len > size { size } else { len };

    for i in 1..check {
        if (p[i] & 0xC0) != 0x80 {
            return 1;
        }
    }

    len
}

/// Check if a byte is a UTF-8 continuation byte (0x80-0xBF).
#[inline]
pub fn utf_is_continuation(b: u8) -> bool {
    (b & 0xC0) == 0x80
}

/// Check if a character is a valid Unicode codepoint.
#[inline]
pub fn utf_valid(c: i32) -> bool {
    // Valid range: 0 to 0x10FFFF, excluding surrogate pairs
    c >= 0 && c <= 0x10_FFFF && !(0xD800..=0xDFFF).contains(&c)
}

/// Check if a string is valid UTF-8.
///
/// If `end` is None, stops at the first NUL byte.
/// If `end` is Some(n), checks exactly n bytes.
pub fn utf_valid_string(s: &[u8], len: Option<usize>) -> bool {
    let end = len.unwrap_or(s.len());
    let mut i = 0;

    while i < end && (len.is_some() || s[i] != 0) {
        let l = UTF8LEN_TAB_ZERO[s[i] as usize] as usize;
        if l == 0 {
            return false; // invalid lead byte
        }
        if i + l > end {
            return false; // incomplete byte sequence
        }
        // Check continuation bytes
        for j in 1..l {
            if i + j >= s.len() || (s[i + j] & 0xC0) != 0x80 {
                return false; // invalid trail byte
            }
        }
        i += l;
    }
    true
}

// FFI functions

/// Get the byte length of UTF-8 encoding for a Unicode codepoint.
///
/// # Safety
///
/// This function is safe to call with any `c_int` value.
#[no_mangle]
pub extern "C" fn rs_utf_char2len(c: c_int) -> c_int {
    utf_char2len(c) as c_int
}

/// Convert Unicode codepoint to UTF-8 byte sequence.
///
/// # Safety
///
/// `buf` must be a valid pointer to at least 6 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int {
    if buf.is_null() {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, MB_MAXBYTES) };
    utf_char2bytes(c, slice) as c_int
}

/// Get the expected UTF-8 sequence length from the first byte.
///
/// # Safety
///
/// This function is safe to call with any `c_int` value (0-255 used, others return 1).
#[no_mangle]
pub extern "C" fn rs_utf_byte2len(b: c_int) -> c_int {
    if b < 0 || b > 255 {
        1
    } else {
        UTF8LEN_TAB[b as usize] as c_int
    }
}

/// Decode UTF-8 byte sequence to Unicode codepoint.
///
/// # Safety
///
/// `p` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_ptr2char(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Find the string length (up to 6 bytes needed)
    let mut len = 0;
    while len < MB_MAXBYTES {
        if unsafe { *p.add(len) } == 0 {
            break;
        }
        len += 1;
    }

    if len == 0 {
        return 0;
    }

    let slice = unsafe { std::slice::from_raw_parts(p as *const u8, len) };
    utf_ptr2char(slice)
}

/// Get the byte length of a UTF-8 character in a string.
///
/// # Safety
///
/// `p` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_ptr2len(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Find the string length
    let mut len = 0;
    while len < MB_MAXBYTES {
        if unsafe { *p.add(len) } == 0 {
            break;
        }
        len += 1;
    }

    if len == 0 {
        return 0;
    }

    let slice = unsafe { std::slice::from_raw_parts(p as *const u8, len) };
    utf_ptr2len(slice) as c_int
}

/// Get the byte length of a UTF-8 character with size limit.
///
/// # Safety
///
/// `p` must be a valid pointer to at least `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_ptr2len_len(p: *const c_char, size: c_int) -> c_int {
    if p.is_null() || size <= 0 {
        return 1;
    }

    let slice = unsafe { std::slice::from_raw_parts(p as *const u8, size as usize) };
    utf_ptr2len_len(slice, size as usize) as c_int
}

/// Check if a string is valid UTF-8.
///
/// # Safety
///
/// `s` must be a valid pointer to a string. If `end` is not null, the string
/// must have at least `end - s` bytes. If `end` is null, `s` must be null-terminated.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_valid_string(s: *const c_char, end: *const c_char) -> c_int {
    if s.is_null() {
        return 1; // NULL is considered valid (matches C behavior)
    }

    let len = if end.is_null() {
        // Find NUL terminator - but we need to determine max safe length
        let mut len = 0;
        while unsafe { *s.add(len) } != 0 {
            len += 1;
            if len > 1_000_000 {
                break; // Safety limit
            }
        }
        len
    } else {
        (end as usize).saturating_sub(s as usize)
    };

    if len == 0 {
        return 1; // Empty string is valid
    }

    let slice = unsafe { std::slice::from_raw_parts(s as *const u8, len) };
    let use_len = if end.is_null() { None } else { Some(len) };
    c_int::from(utf_valid_string(slice, use_len))
}

// Line breaking functions

/// Whether space is NOT allowed before/after 'c'.
///
/// Returns true for various CJK and fullwidth punctuation characters
/// where whitespace should be suppressed.
#[inline]
pub fn utf_eat_space(cc: i32) -> bool {
    (0x2000..=0x206F).contains(&cc) // General punctuations
        || (0x2E00..=0x2E7F).contains(&cc) // Supplemental punctuations
        || (0x3000..=0x303F).contains(&cc) // CJK symbols and punctuations
        || (0xFF01..=0xFF0F).contains(&cc) // Full width ASCII punctuations
        || (0xFF1A..=0xFF20).contains(&cc) // ..
        || (0xFF3B..=0xFF40).contains(&cc) // ..
        || (0xFF5B..=0xFF65).contains(&cc) // ..
}

/// Characters that prohibit line break before them (BOL prohibition).
/// Sorted for binary search.
static BOL_PROHIBITION_PUNCT: &[i32] = &[
    b'!' as i32,
    b'%' as i32,
    b')' as i32,
    b',' as i32,
    b':' as i32,
    b';' as i32,
    b'>' as i32,
    b'?' as i32,
    b']' as i32,
    b'}' as i32,
    0x2019, // ' right single quotation mark
    0x201D, // " right double quotation mark
    0x2020, // † dagger
    0x2021, // ‡ double dagger
    0x2026, // … horizontal ellipsis
    0x2030, // ‰ per mille sign
    0x2031, // ‱ per the thousand sign
    0x203C, // ‼ double exclamation mark
    0x2047, // ⁇ double question mark
    0x2048, // ⁈ question exclamation mark
    0x2049, // ⁉ exclamation question mark
    0x2103, // ℃ degree celsius
    0x2109, // ℉ degree fahrenheit
    0x3001, // 、 ideographic comma
    0x3002, // 。 ideographic full stop
    0x3009, // 〉 right angle bracket
    0x300B, // 》 right double angle bracket
    0x300D, // 」 right corner bracket
    0x300F, // 』 right white corner bracket
    0x3011, // 】 right black lenticular bracket
    0x3015, // 〕 right tortoise shell bracket
    0x3017, // 〗 right white lenticular bracket
    0x3019, // 〙 right white tortoise shell bracket
    0x301B, // 〛 right white square bracket
    0xFF01, // ！ fullwidth exclamation mark
    0xFF09, // ） fullwidth right parenthesis
    0xFF0C, // ， fullwidth comma
    0xFF0E, // ． fullwidth full stop
    0xFF1A, // ： fullwidth colon
    0xFF1B, // ； fullwidth semicolon
    0xFF1F, // ？ fullwidth question mark
    0xFF3D, // ］ fullwidth right square bracket
    0xFF5D, // ｝ fullwidth right curly bracket
];

/// Whether line break is allowed before "cc".
///
/// Returns false for characters that should not appear at the beginning
/// of a line (closing brackets, punctuation marks, etc.).
#[inline]
pub fn utf_allow_break_before(cc: i32) -> bool {
    BOL_PROHIBITION_PUNCT.binary_search(&cc).is_err()
}

/// Characters that prohibit line break after them (EOL prohibition).
/// Sorted for binary search.
static EOL_PROHIBITION_PUNCT: &[i32] = &[
    b'(' as i32,
    b'<' as i32,
    b'[' as i32,
    b'`' as i32,
    b'{' as i32,
    0x2018, // ' left single quotation mark
    0x201C, // " left double quotation mark
    0x3008, // 〈 left angle bracket
    0x300A, // 《 left double angle bracket
    0x300C, // 「 left corner bracket
    0x300E, // 『 left white corner bracket
    0x3010, // 【 left black lenticular bracket
    0x3014, // 〔 left tortoise shell bracket
    0x3016, // 〖 left white lenticular bracket
    0x3018, // 〘 left white tortoise shell bracket
    0x301A, // 〚 left white square bracket
    0xFF08, // （ fullwidth left parenthesis
    0xFF3B, // ［ fullwidth left square bracket
    0xFF5B, // ｛ fullwidth left curly bracket
];

/// Whether line break is allowed after "cc".
///
/// Returns false for characters that should not appear at the end
/// of a line (opening brackets, quotation marks, etc.).
#[inline]
pub fn utf_allow_break_after(cc: i32) -> bool {
    EOL_PROHIBITION_PUNCT.binary_search(&cc).is_err()
}

// FFI wrappers for line breaking functions

/// Whether space is NOT allowed before/after 'c'.
#[no_mangle]
pub extern "C" fn rs_utf_eat_space(cc: c_int) -> c_int {
    c_int::from(utf_eat_space(cc))
}

/// Whether line break is allowed before "cc".
#[no_mangle]
pub extern "C" fn rs_utf_allow_break_before(cc: c_int) -> c_int {
    c_int::from(utf_allow_break_before(cc))
}

/// Whether line break is allowed after "cc".
#[no_mangle]
pub extern "C" fn rs_utf_allow_break_after(cc: c_int) -> c_int {
    c_int::from(utf_allow_break_after(cc))
}

/// Whether line break is allowed between "cc" and "ncc".
///
/// Returns false for pairs that should not be broken:
/// - Don't break between two identical punctuations (em dash, horizontal ellipsis)
/// - Don't break if break not allowed after cc or before ncc
#[inline]
pub fn utf_allow_break(cc: i32, ncc: i32) -> bool {
    // Don't break between two-letter punctuations
    if cc == ncc && (cc == 0x2014 || cc == 0x2026) {
        // em dash or horizontal ellipsis
        return false;
    }
    utf_allow_break_after(cc) && utf_allow_break_before(ncc)
}

/// Whether line break is allowed between "cc" and "ncc".
#[no_mangle]
pub extern "C" fn rs_utf_allow_break(cc: c_int, ncc: c_int) -> c_int {
    c_int::from(utf_allow_break(cc, ncc))
}

/// Return the number of characters in a string.
/// Composing characters are not counted separately.
///
/// Uses the native Rust utfc_ptr2len implementation.
#[no_mangle]
pub unsafe extern "C" fn rs_mb_charlen(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    // Find string length
    let mut len = 0usize;
    while *str.add(len) != 0 {
        len += 1;
    }

    if len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(str as *const u8, len);
    let mut p = 0usize;
    let mut count: c_int = 0;

    while p < len && slice[p] != 0 {
        p += utfc_ptr2len(&slice[p..]);
        count += 1;
    }

    count
}

/// Return the number of characters in a string, limited to "len" bytes.
/// Composing characters are not counted separately.
///
/// Uses the native Rust utfc_ptr2len_len implementation.
#[no_mangle]
pub unsafe extern "C" fn rs_mb_charlen_len(str: *const c_char, len: c_int) -> c_int {
    if str.is_null() || len <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(str as *const u8, len as usize);
    let mut p = 0usize;
    let mut count: c_int = 0;

    while p < slice.len() && slice[p] != 0 {
        p += utfc_ptr2len_len(&slice[p..], slice.len() - p);
        count += 1;
    }

    count
}

/// Return the number of cells occupied by a string.
/// Uses utf_ptr2cells and native Rust utfc_ptr2len.
#[no_mangle]
pub unsafe extern "C" fn rs_mb_string2cells(str: *const c_char) -> usize {
    if str.is_null() {
        return 0;
    }

    // Find string length
    let mut len = 0usize;
    while *str.add(len) != 0 {
        len += 1;
    }

    if len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(str as *const u8, len);
    let mut p = 0usize;
    let mut clen: usize = 0;

    while p < len && slice[p] != 0 {
        clen += utf_ptr2cells(&slice[p..]) as usize;
        p += utfc_ptr2len(&slice[p..]);
    }

    clen
}

/// Return the number of cells occupied by a string with maximum length.
/// Uses utf_ptr2cells and native Rust utfc_ptr2len_len.
#[no_mangle]
pub unsafe extern "C" fn rs_mb_string2cells_len(str: *const c_char, size: usize) -> usize {
    if str.is_null() || size == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(str as *const u8, size);
    let mut p = 0usize;
    let mut clen: usize = 0;

    while p < size && slice[p] != 0 {
        clen += utf_ptr2cells(&slice[p..]) as usize;
        p += utfc_ptr2len_len(&slice[p..], size - p);
    }

    clen
}

// Character printability

/// Sorted list of non-printable character ranges for `utf_printable`.
/// These are characters that cannot be displayed in a normal way.
/// 0xd800-0xdfff is reserved for UTF-16 surrogates (actually illegal).
static NONPRINT_RANGES: &[(i32, i32)] = &[
    (0x070f, 0x070f), // Syriac abbreviation mark
    (0x180b, 0x180e), // Mongolian free variation selectors
    (0x200b, 0x200f), // Zero width space, directional marks
    (0x202a, 0x202e), // Embedding/override controls
    (0x2060, 0x206f), // Word joiner, invisible operators
    (0xd800, 0xdfff), // UTF-16 surrogates (illegal in UTF-8)
    (0xfeff, 0xfeff), // BOM / ZWNBSP
    (0xfff9, 0xfffb), // Interlinear annotation anchors
    (0xfffe, 0xffff), // Non-characters
];

/// Check if a character is in any of the non-printable ranges.
#[inline]
fn in_nonprint_range(c: i32) -> bool {
    // Binary search through sorted ranges
    let mut lo = 0;
    let mut hi = NONPRINT_RANGES.len();

    while lo < hi {
        let mid = usize::midpoint(lo, hi);
        let (first, last) = NONPRINT_RANGES[mid];
        if last < c {
            lo = mid + 1;
        } else if first > c {
            hi = mid;
        } else {
            return true; // c is in [first, last]
        }
    }
    false
}

/// Return true for characters that can be displayed in a normal way.
/// Only for characters of 0x100 and above!
///
/// This checks if a character is NOT in any of the ranges that contain
/// non-printable/control characters (format controls, surrogates, etc.).
#[inline]
pub fn utf_printable(c: i32) -> bool {
    !in_nonprint_range(c)
}

/// Return true for characters that can be displayed in a normal way.
/// FFI wrapper for `utf_printable`.
#[no_mangle]
pub extern "C" fn rs_utf_printable(c: c_int) -> c_int {
    c_int::from(utf_printable(c))
}

// Unicode property functions using utf8proc

/// Check if a character is a composing character (legacy check).
///
/// Returns true for nonspacing marks (Mn) and enclosing marks (Me).
/// This is a legacy check - for proper grapheme cluster detection,
/// use the stateful grapheme algorithm instead.
///
/// Returns false for negative values.
#[inline]
pub fn utf_iscomposing_legacy(c: i32) -> bool {
    get_property(c).is_some_and(Utf8procProperty::is_composing_legacy)
}

/// FFI wrapper for `utf_iscomposing_legacy`.
#[no_mangle]
pub extern "C" fn rs_utf_iscomposing_legacy(c: c_int) -> c_int {
    c_int::from(utf_iscomposing_legacy(c))
}

/// Check if a character at the start of a string needs a space prefix.
///
/// When "c" is the first char of a string, determine if it needs to be prefixed
/// by a space byte to be drawn correctly, and not merge with the space left of
/// the string.
///
/// Returns true if "c" would combine with a preceding space (no grapheme break).
#[inline]
pub fn utf_iscomposing_first(c: i32) -> bool {
    c >= 128 && !grapheme_break(b' ' as i32, c)
}

/// FFI wrapper for `utf_iscomposing_first`.
#[no_mangle]
pub extern "C" fn rs_utf_iscomposing_first(c: c_int) -> c_int {
    c_int::from(utf_iscomposing_first(c))
}

// Case folding

/// Return the folded-case equivalent of a Unicode codepoint.
///
/// Uses full case folding for case-insensitive string comparison.
/// For ASCII, this is simple lowercase conversion (A-Z -> a-z).
///
/// Note: Some characters have multi-character case foldings (e.g., ß -> ss),
/// but this function returns only single-character results. Characters with
/// problematic multi-char foldings are returned unchanged to maintain
/// compatibility with Vim's spell checking.
#[inline]
pub fn utf_fold(a: i32) -> i32 {
    casefold(a)
}

/// FFI wrapper for `utf_fold`.
#[no_mangle]
pub extern "C" fn rs_utf_fold(a: c_int) -> c_int {
    utf_fold(a)
}

// UTF-8 string comparison

/// Safely read a character from a UTF-8 buffer with length limit.
///
/// Returns the decoded codepoint and the number of bytes consumed.
/// Returns (0, 0) at end of buffer.
/// Returns (-1, 0) for invalid/incomplete sequences (caller should handle bytewise).
/// Returns (byte, 1) for ASCII characters including NUL.
///
/// This is an internal helper for utf_strnicmp.
fn utf_safe_read_char_adv(s: &[u8]) -> (i32, usize) {
    if s.is_empty() {
        return (0, 0); // end of buffer
    }

    let k = UTF8LEN_TAB_ZERO[s[0] as usize] as usize;

    if k == 1 {
        // ASCII character or NUL
        return (s[0] as i32, 1);
    }

    if k <= s.len() {
        // We have a multibyte sequence and it isn't truncated by buffer
        // limits so utf_ptr2char is safe to use. Or the first byte is
        // illegal (k=0), and it's also safe to use utf_ptr2char.
        let c = utf_ptr2char(s);

        // On failure, utf_ptr2char returns the first byte, so here we
        // check equality with the first byte. The only non-ASCII character
        // which equals the first byte of its own UTF-8 representation is
        // U+00C3 (UTF-8: 0xC3 0x83), so need to check that special case too.
        // It's safe even if n=1, else we would have k=2 > n.
        if c != s[0] as i32 || (c == 0xC3 && s.len() > 1 && s[1] == 0x83) {
            // byte sequence was successfully decoded
            return (c, k);
        }
    }

    // byte sequence is incomplete or illegal
    (-1, 0)
}

/// Compare two UTF-8 strings case-insensitively.
///
/// Compares s1[0..n1] with s2[0..n2] using case folding.
///
/// Returns:
/// - 0 if strings are equal
/// - negative if s1 < s2
/// - positive if s1 > s2
pub fn utf_strnicmp(s1: &[u8], s2: &[u8]) -> i32 {
    let mut i1 = 0;
    let mut i2 = 0;

    loop {
        let (c1, len1) = utf_safe_read_char_adv(&s1[i1..]);
        let (c2, len2) = utf_safe_read_char_adv(&s2[i2..]);
        i1 += len1;
        i2 += len2;

        if c1 <= 0 || c2 <= 0 {
            // End of string or invalid sequence
            if c1 == 0 || c2 == 0 {
                // some string ended. shorter string is smaller
                if c1 == 0 && c2 == 0 {
                    return 0;
                }
                return if c1 == 0 { -1 } else { 1 };
            }

            // Continue with bytewise comparison for invalid sequences
            // If only one string had an error, comparison should be made with
            // folded version of the other string.
            let (cmp_s1, cmp_n1, cmp_s2, cmp_n2);
            let mut fold_buf = [0u8; 6];

            if c1 != -1 && c2 == -1 {
                // s1 is valid, s2 had error - fold s1
                let folded = utf_fold(c1);
                let fold_len = utf_char2bytes(folded, &mut fold_buf);
                cmp_s1 = &fold_buf[..fold_len];
                cmp_n1 = fold_len;
                cmp_s2 = &s2[i2 - len2..];
                cmp_n2 = s2.len() - (i2 - len2);
            } else if c2 != -1 && c1 == -1 {
                // s2 is valid, s1 had error - fold s2
                let folded = utf_fold(c2);
                let fold_len = utf_char2bytes(folded, &mut fold_buf);
                cmp_s1 = &s1[i1 - len1..];
                cmp_n1 = s1.len() - (i1 - len1);
                cmp_s2 = &fold_buf[..fold_len];
                cmp_n2 = fold_len;
            } else {
                // Both had errors
                cmp_s1 = &s1[i1..];
                cmp_n1 = s1.len() - i1;
                cmp_s2 = &s2[i2..];
                cmp_n2 = s2.len() - i2;
            }

            // Bytewise comparison
            let mut j1 = 0;
            let mut j2 = 0;
            while j1 < cmp_n1 && j2 < cmp_n2 {
                if cmp_s1[j1] == 0 || cmp_s2[j2] == 0 {
                    break;
                }
                let cdiff = cmp_s1[j1] as i32 - cmp_s2[j2] as i32;
                if cdiff != 0 {
                    return cdiff;
                }
                j1 += 1;
                j2 += 1;
            }

            // Check for NUL termination
            let n1_remaining = if j1 < cmp_n1 && cmp_s1[j1] == 0 {
                0
            } else {
                cmp_n1 - j1
            };
            let n2_remaining = if j2 < cmp_n2 && cmp_s2[j2] == 0 {
                0
            } else {
                cmp_n2 - j2
            };

            return if n1_remaining == 0 && n2_remaining == 0 {
                0
            } else if n1_remaining == 0 {
                -1
            } else {
                1
            };
        }

        if c1 == c2 {
            continue;
        }

        let cdiff = utf_fold(c1) - utf_fold(c2);
        if cdiff != 0 {
            return cdiff;
        }
    }
}

/// FFI wrapper for `utf_strnicmp`.
///
/// # Safety
///
/// - s1 must be valid for reads of n1 bytes
/// - s2 must be valid for reads of n2 bytes
#[no_mangle]
pub unsafe extern "C" fn rs_utf_strnicmp(
    s1: *const c_char,
    s2: *const c_char,
    n1: usize,
    n2: usize,
) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }
    let slice1 = std::slice::from_raw_parts(s1 as *const u8, n1);
    let slice2 = std::slice::from_raw_parts(s2 as *const u8, n2);
    utf_strnicmp(slice1, slice2)
}

/// FFI wrapper for `mb_strnicmp`.
///
/// Compares two strings case-insensitively up to `nn` bytes.
/// This is a wrapper around `utf_strnicmp` with the same length for both strings.
///
/// # Safety
///
/// - s1 must be valid for reads of nn bytes
/// - s2 must be valid for reads of nn bytes
#[no_mangle]
pub unsafe extern "C" fn rs_mb_strnicmp(
    s1: *const c_char,
    s2: *const c_char,
    nn: usize,
) -> c_int {
    rs_utf_strnicmp(s1, s2, nn, nn)
}

/// MAXCOL constant (same as in Neovim's pos_defs.h)
const MAXCOL: usize = 0x7fff_ffff;

/// FFI wrapper for `mb_stricmp`.
///
/// Compares two strings case-insensitively (up to MAXCOL bytes).
/// This is a wrapper around `mb_strnicmp` with MAXCOL as the length limit.
///
/// # Safety
///
/// - s1 and s2 must be valid null-terminated strings
#[no_mangle]
pub unsafe extern "C" fn rs_mb_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    rs_mb_strnicmp(s1, s2, MAXCOL)
}

/// FFI wrapper for `mb_strcmp_ic`.
///
/// Compare two strings optionally ignoring case.
/// When `ic` is true, uses case-insensitive comparison (mb_stricmp).
/// When `ic` is false, uses case-sensitive comparison (strcmp).
///
/// # Safety
///
/// - s1 and s2 must be valid null-terminated strings
#[no_mangle]
pub unsafe extern "C" fn rs_mb_strcmp_ic(
    ic: bool,
    s1: *const c_char,
    s2: *const c_char,
) -> c_int {
    if ic {
        rs_mb_stricmp(s1, s2)
    } else {
        libc::strcmp(s1, s2) as c_int
    }
}

// Ambiguous width detection

/// VS-16 (Variation Selector 16) UTF-8 encoding: U+FE0F = 0xEF 0xB8 0x8F
/// This turns certain characters into emoji presentation.
const VS16: [u8; 3] = [0xEF, 0xB8, 0x8F];

/// Check if a UTF-8 character has ambiguous width.
///
/// Returns true if:
/// - The character has the East Asian Ambiguous Width property, or
/// - The character is emoji-like (extended pictographic or regional indicator), or
/// - The character is followed by VS-16 (U+FE0F) which turns things into emoji
///
/// Returns false for:
/// - Empty strings (NUL at start)
/// - ASCII characters (single-byte)
pub fn utf_ambiguous_width(p: &[u8]) -> bool {
    // Be quick if there is nothing to print or ASCII-only
    if p.is_empty() || p[0] == 0 || (p.len() == 1 || p.len() >= 2 && p[1] == 0) {
        return false;
    }

    // Decode the first character
    let c = utf_ptr2char(p);
    let len = utf_ptr2len(p);

    if c >= 0x80 {
        if let Some(prop) = get_property(c) {
            if prop.ambiguous_width() || prop.is_emojilike() {
                return true;
            }
        }
    }

    // Check if second sequence is 0xFE0F VS-16 which can turn things into emoji
    if len < p.len() && p.len() - len >= 3 {
        return p[len..len + 3] == VS16;
    }

    false
}

/// FFI wrapper for `utf_ambiguous_width`.
///
/// # Safety
///
/// `p` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_ambiguous_width(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Find string length (we need at least enough for char + potential VS-16)
    let mut len = 0;
    while len < 16 {
        // 6 bytes max char + 3 bytes VS-16 + margin
        if unsafe { *p.add(len) } == 0 {
            break;
        }
        len += 1;
    }

    if len == 0 {
        return 0;
    }

    let slice = unsafe { std::slice::from_raw_parts(p as *const u8, len) };
    c_int::from(utf_ambiguous_width(slice))
}

// Cell width table access (setcellwidths())

/// Cell width interval as set by `setcellwidths()`.
/// Matches the C struct `cw_interval_T`.
#[repr(C)]
struct CwInterval {
    first: c_longlong, // int64_t
    last: c_longlong,  // int64_t
    width: i8,         // char
}

extern "C" {
    /// Cell width table pointer (set by `setcellwidths()`).
    static cw_table: *const CwInterval;
    /// Size of the cell width table.
    static cw_table_size: usize;

    /// 'ambiwidth' option: "single" or "double".
    /// Points to a C string; first character 'd' means double width.
    static p_ambw: *const c_char;

    /// 'emoji' option: boolean (0 or non-zero).
    static p_emoji: c_int;
}

/// Return the value of the cellwidth table for the character `c`.
///
/// Returns 1 or 2 when `c` is in the cellwidth table, 0 if not.
///
/// This function performs binary search on the `cw_table` set by `setcellwidths()`.
#[inline]
pub fn cw_value(c: i32) -> i32 {
    // SAFETY: cw_table and cw_table_size are managed by C code.
    // cw_table is NULL initially and only set by setcellwidths().
    let table = unsafe { cw_table };
    let size = unsafe { cw_table_size };

    if table.is_null() || size == 0 {
        return 0;
    }

    // SAFETY: We just checked table is not null and size is valid.
    let slice = unsafe { std::slice::from_raw_parts(table, size) };

    // Quick check for Latin1 etc. characters
    if (c as c_longlong) < slice[0].first {
        return 0;
    }

    // Binary search in table
    let mut bot: i32 = 0;
    let mut top: i32 = size as i32 - 1;

    while top >= bot {
        let mid = i32::midpoint(bot, top);
        let entry = &slice[mid as usize];
        if entry.last < c as c_longlong {
            bot = mid + 1;
        } else if entry.first > c as c_longlong {
            top = mid - 1;
        } else {
            return entry.width as i32;
        }
    }

    0
}

/// FFI wrapper for `cw_value`.
#[no_mangle]
pub extern "C" fn rs_cw_value(c: c_int) -> c_int {
    cw_value(c)
}

// Character cell width calculation

extern "C" {
    /// Check if a character is printable. Defined in charset.c.
    fn rs_vim_isprintc(c: c_int) -> c_int;
}

/// Check if a character is printable (safe wrapper).
#[inline]
fn vim_isprintc(c: i32) -> bool {
    // SAFETY: rs_vim_isprintc is a safe function that just reads g_chartab
    unsafe { rs_vim_isprintc(c) != 0 }
}

/// Check if 'ambiwidth' option is set to "double".
#[inline]
fn ambiwidth_is_double() -> bool {
    // SAFETY: p_ambw is set during option initialization
    unsafe { !p_ambw.is_null() && *p_ambw == b'd' as c_char }
}

/// Check if 'emoji' option is enabled.
#[inline]
fn emoji_is_enabled() -> bool {
    // SAFETY: p_emoji is set during option initialization
    unsafe { p_emoji != 0 }
}

/// For UTF-8 character "c" return 2 for a double-width character, 1 for others.
///
/// Returns 4 or 6 for an unprintable character.
/// Is only correct for characters >= 0x80.
///
/// When `p_ambw` is "double", return 2 for a character with East Asian Width
/// class 'A'(mbiguous).
///
/// # Panics
///
/// Panics if `c` is unprintable and greater than 0xFFFF.
#[inline]
pub fn utf_char2cells(c: i32) -> i32 {
    if c < 0x80 {
        return 1;
    }

    if !vim_isprintc(c) {
        assert!(c <= 0xFFFF);
        // unprintable is displayed either as <xx> or <xxxx>
        return if c > 0xFF { 6 } else { 4 };
    }

    let n = cw_value(c);
    if n != 0 {
        return n;
    }

    if let Some(prop) = get_property(c) {
        if prop.charwidth() == 2 {
            return 2;
        }
        if ambiwidth_is_double() && prop.ambiguous_width() {
            return 2;
        }

        // Characters below 1F000 may be considered single width traditionally,
        // making them double width causes problems.
        if emoji_is_enabled() && c >= 0x1f000 && !prop.ambiguous_width() && prop.is_emojilike() {
            return 2;
        }
    }

    1
}

/// FFI wrapper for `utf_char2cells`.
#[no_mangle]
pub extern "C" fn rs_utf_char2cells(c: c_int) -> c_int {
    utf_char2cells(c)
}

// Character to cell width from pointer

/// VS-16 (Variation Selector 16) codepoint: U+FE0F
/// This turns certain characters into emoji presentation.
const VS16_CODEPOINT: i32 = 0xFE0F;

/// Decode UTF-8 byte sequence to Unicode codepoint with strict validation.
///
/// Returns `Some(codepoint)` for sequences with valid structure (correct
/// continuation bytes), `None` for structurally invalid sequences.
///
/// Note: This function DOES NOT reject overlong encodings - it returns
/// the decoded codepoint even if it's an ASCII value from a multibyte
/// sequence. The caller should check for overlong sequences if needed.
///
/// Invalid sequences (returning None) include:
/// - Empty input
/// - Incomplete sequences (not enough bytes)
/// - Invalid continuation bytes (not 0x80-0xBF)
/// - Invalid lead byte (continuation byte or 0xFE/0xFF as first byte)
fn utf_ptr2char_strict(p: &[u8]) -> Option<i32> {
    if p.is_empty() {
        return None;
    }

    let v0 = p[0] as u32;

    // Fast path for ASCII
    if v0 < 0x80 {
        return Some(v0 as i32);
    }

    let len = UTF8LEN_TAB[v0 as usize];
    // len == 1 means invalid lead byte (continuation byte 0x80-0xBF or 0xFE/0xFF)
    if len < 2 || p.len() < len as usize {
        return None; // Invalid lead byte or incomplete sequence
    }

    // Check continuation bytes
    macro_rules! check_cont {
        ($v:expr) => {
            if ($v & 0xC0) != 0x80 {
                return None;
            }
        };
    }

    let v1 = p[1] as u32;
    check_cont!(v1);

    if len == 2 {
        let c = ((v0 << 6) + v1 - ((0xC0 << 6) + 0x80)) as i32;
        return Some(c);
    }

    let v2 = p[2] as u32;
    check_cont!(v2);

    if len == 3 {
        let c = ((v0 << 12) + (v1 << 6) + v2 - ((0xE0 << 12) + (0x80 << 6) + 0x80)) as i32;
        return Some(c);
    }

    let v3 = p[3] as u32;
    check_cont!(v3);

    if len == 4 {
        let c = ((v0 << 18) + (v1 << 12) + (v2 << 6) + v3
            - ((0xF0 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80)) as i32;
        return Some(c);
    }

    let v4 = p[4] as u32;
    check_cont!(v4);

    if len == 5 {
        let c = ((v0 << 24) + (v1 << 18) + (v2 << 12) + (v3 << 6) + v4
            - ((0xF8 << 24) + (0x80 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80))
            as i32;
        return Some(c);
    }

    // len == 6
    let v5 = p[5] as u32;
    check_cont!(v5);

    let c = ((v0 << 30) + (v1 << 24) + (v2 << 18) + (v3 << 12) + (v4 << 6) + v5
        - ((0x80 << 24) + (0x80 << 18) + (0x80 << 12) + (0x80 << 6) + 0x80)) as i32;
    Some(c)
}

extern "C" {
    /// char2cells from charset crate for overlong ASCII sequences
    fn rs_char2cells(c: c_int) -> c_int;
}

/// Return the number of display cells character at "*p" occupies.
///
/// This doesn't take care of unprintable characters, use `ptr2cells()` for that.
///
/// For ASCII (single byte), returns 1.
/// For valid multibyte UTF-8, returns the character's cell width.
/// For invalid/illegal UTF-8 sequences, returns 4 (displayed as `<xx>`).
/// For overlong ASCII encodings, returns char2cells of the decoded value.
///
/// Also checks for emoji presentation selector (VS-16) following emoji-like
/// characters, returning 2 cells if found (when 'emoji' option is enabled).
pub fn utf_ptr2cells(p: &[u8]) -> i32 {
    if p.is_empty() {
        return 1;
    }

    let first = p[0];

    // ASCII fast path
    if first < 0x80 {
        return 1;
    }

    // Multibyte - need to decode
    let len = UTF8LEN_TAB[first as usize] as usize;

    // Try to decode with validation
    match utf_ptr2char_strict(p) {
        None => {
            // Invalid/illegal byte sequence - displayed as <xx>
            4
        }
        Some(c) if c < 0x80 => {
            // ASCII value from multibyte = overlong sequence
            // SAFETY: rs_char2cells just accesses g_chartab
            unsafe { rs_char2cells(c) }
        }
        Some(c) => {
            let cells = utf_char2cells(c);

            // Check for emoji presentation selector (VS-16)
            if cells == 1 && emoji_is_enabled() {
                if let Some(prop) = get_property(c) {
                    if prop.is_emojilike() && p.len() > len {
                        // Check if next character is VS-16 (U+FE0F)
                        let c2 = utf_ptr2char(&p[len..]);
                        if c2 == VS16_CODEPOINT {
                            return 2; // emoji presentation
                        }
                    }
                }
            }

            cells
        }
    }
}

/// FFI wrapper for `utf_ptr2cells`.
///
/// # Safety
///
/// `p` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_ptr2cells(p: *const c_char) -> c_int {
    if p.is_null() {
        return 1;
    }

    // Find string length (we need enough for char + potential VS-16)
    let mut len = 0;
    while len < 16 {
        // 6 bytes max char + 3 bytes VS-16 + margin
        if unsafe { *p.add(len) } == 0 {
            break;
        }
        len += 1;
    }

    if len == 0 {
        return 1;
    }

    let slice = unsafe { std::slice::from_raw_parts(p as *const u8, len) };
    utf_ptr2cells(slice)
}

// Codepoint boundary detection

/// Result of finding codepoint boundaries.
///
/// Represents the offset from a pointer to the beginning and end of
/// the UTF-8 codepoint it points into.
#[repr(C)]
pub struct CharBoundsOff {
    /// Offset to the first byte of the codepoint (negative or zero).
    pub begin_off: i8,
    /// Offset to one past the end byte of the codepoint (positive).
    pub end_off: i8,
}

/// Maximum number of bytes in a UTF-8 codepoint (nvim supports 6 for legacy).
const MB_MAXCHAR: usize = 6;

/// Find the codepoint boundaries for a pointer into a UTF-8 string.
///
/// Given a base pointer and a pointer somewhere into the string,
/// returns the offsets from `p_in` to the first byte and one-past-end
/// byte of the codepoint that `p_in` points into.
///
/// `p_len` limits the number of bytes after `p_in`.
///
/// Note: Counts individual codepoints of composed characters separately.
///
/// Returns `{ 0, 1 }` for:
/// - ASCII bytes (fast path)
/// - Invalid/incomplete sequences
/// - When the first byte cannot be found
pub fn utf_cp_bounds_len(base: &[u8], p_offset: usize, p_len: usize) -> CharBoundsOff {
    if p_offset >= base.len() || p_len == 0 {
        return CharBoundsOff {
            begin_off: 0,
            end_off: 1,
        };
    }

    let p = base[p_offset];

    // Fast path for ASCII
    if p < 0x80 {
        return CharBoundsOff {
            begin_off: 0,
            end_off: 1,
        };
    }

    // Search backwards for the first byte of the codepoint
    let max_backwards = p_offset.min(MB_MAXCHAR - 1);
    let mut first_off: isize = 0;

    while utf_is_continuation(base[(p_offset as isize + first_off) as usize]) {
        first_off -= 1;
        if first_off == -(max_backwards as isize) - 1 {
            // Failed to find first byte
            return CharBoundsOff {
                begin_off: 0,
                end_off: 1,
            };
        }
    }

    // Get expected length from first byte
    let first_byte_idx = (p_offset as isize + first_off) as usize;
    let max_end_off = UTF8LEN_TAB[base[first_byte_idx] as usize] as isize + first_off;

    // Check for illegal or incomplete sequence
    if max_end_off <= 0 || max_end_off > p_len as isize {
        return CharBoundsOff {
            begin_off: 0,
            end_off: 1,
        };
    }

    // Verify all continuation bytes are valid
    for end_off in 1..max_end_off {
        let idx = (p_offset as isize + end_off) as usize;
        if idx >= base.len() || !utf_is_continuation(base[idx]) {
            return CharBoundsOff {
                begin_off: 0,
                end_off: 1,
            };
        }
    }

    CharBoundsOff {
        begin_off: -first_off as i8,
        end_off: max_end_off as i8,
    }
}

/// Find the codepoint boundaries for a pointer into a NUL-terminated UTF-8 string.
///
/// This is a convenience wrapper around `utf_cp_bounds_len` with an unlimited
/// forward length (suitable for NUL-terminated strings).
pub fn utf_cp_bounds(base: &[u8], p_offset: usize) -> CharBoundsOff {
    let remaining = base.len().saturating_sub(p_offset);
    utf_cp_bounds_len(base, p_offset, remaining.max(1))
}

/// FFI wrapper for `utf_cp_bounds_len`.
///
/// Returns the offset from `p_in` to the first and one-past-end bytes
/// of the codepoint it points to.
///
/// # Safety
///
/// - `base` must be a valid pointer
/// - `p_in` must point into the memory starting at `base`
/// - `p_len` must not exceed the remaining bytes after `p_in`
#[no_mangle]
pub unsafe extern "C" fn rs_utf_cp_bounds_len(
    base: *const c_char,
    p_in: *const c_char,
    p_len: c_int,
) -> CharBoundsOff {
    if base.is_null() || p_in.is_null() || p_len <= 0 || p_in < base {
        return CharBoundsOff {
            begin_off: 0,
            end_off: 1,
        };
    }

    let offset = p_in.offset_from(base) as usize;

    // We need to read from base up to p_in + p_len
    let total_len = offset + p_len as usize;
    let slice = std::slice::from_raw_parts(base as *const u8, total_len);

    utf_cp_bounds_len(slice, offset, p_len as usize)
}

/// FFI wrapper for `utf_cp_bounds`.
///
/// Returns the offset from `p_in` to the first and one-past-end bytes
/// of the codepoint it points to. The string must be NUL-terminated.
///
/// # Safety
///
/// - `base` must be a valid pointer to a NUL-terminated string
/// - `p_in` must point into the memory starting at `base`
#[no_mangle]
pub unsafe extern "C" fn rs_utf_cp_bounds(
    base: *const c_char,
    p_in: *const c_char,
) -> CharBoundsOff {
    if base.is_null() || p_in.is_null() || p_in < base {
        return CharBoundsOff {
            begin_off: 0,
            end_off: 1,
        };
    }

    let offset = p_in.offset_from(base) as usize;

    // Find NUL terminator to determine length
    let mut len = 0;
    while *base.add(len) != 0 {
        len += 1;
        if len > 1_000_000 {
            // Safety limit
            return CharBoundsOff {
                begin_off: 0,
                end_off: 1,
            };
        }
    }

    if offset >= len {
        return CharBoundsOff {
            begin_off: 0,
            end_off: 1,
        };
    }

    let slice = std::slice::from_raw_parts(base as *const u8, len);
    utf_cp_bounds(slice, offset)
}

/// Remove all UTF-8 BOM sequences from a string in place.
///
/// The UTF-8 BOM is the byte sequence 0xEF 0xBB 0xBF. This function finds
/// and removes all occurrences by shifting the remaining content.
///
/// # Safety
///
/// - `s` must be a valid pointer to a mutable, NUL-terminated C string
/// - The string must have enough space for in-place modification
// UTF-8 BOM bytes
const BOM_0: u8 = 0xEF;
const BOM_1: u8 = 0xBB;
const BOM_2: u8 = 0xBF;

#[no_mangle]
pub unsafe extern "C" fn rs_remove_bom(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    let mut p = s as *mut u8;

    loop {
        // Find next 0xEF byte
        while *p != 0 && *p != BOM_0 {
            p = p.add(1);
        }

        if *p == 0 {
            break;
        }

        // Check if this is a BOM sequence
        if *p.add(1) == BOM_1 && *p.add(2) == BOM_2 {
            // Remove the 3-byte BOM by shifting the rest of the string
            let src = p.add(3);
            // Calculate length of remaining string including NUL
            let mut len = 0;
            while *src.add(len) != 0 {
                len += 1;
            }
            // Move len+1 bytes (including NUL terminator)
            core::ptr::copy(src, p, len + 1);
            // Don't advance p - there might be another BOM at the same position
        } else {
            // Not a BOM, move to next byte
            p = p.add(1);
        }
    }
}

// ============================================================================
// Character class for word boundary detection
// ============================================================================

/// Unicode character class ranges for word boundary detection.
///
/// Each entry contains (first, last, class):
/// - 0 = blank/space
/// - 1 = punctuation
/// - 2+ = word character (can use unique class values for different scripts)
///
/// Characters not in any range are considered word characters (class 2).
#[rustfmt::skip]
static UTF_CLASS_TABLE: &[(u32, u32, u32)] = &[
    (0x037e, 0x037e, 1),              // Greek question mark
    (0x0387, 0x0387, 1),              // Greek ano teleia
    (0x055a, 0x055f, 1),              // Armenian punctuation
    (0x0589, 0x0589, 1),              // Armenian full stop
    (0x05be, 0x05be, 1),
    (0x05c0, 0x05c0, 1),
    (0x05c3, 0x05c3, 1),
    (0x05f3, 0x05f4, 1),
    (0x060c, 0x060c, 1),
    (0x061b, 0x061b, 1),
    (0x061f, 0x061f, 1),
    (0x066a, 0x066d, 1),
    (0x06d4, 0x06d4, 1),
    (0x0700, 0x070d, 1),              // Syriac punctuation
    (0x0964, 0x0965, 1),
    (0x0970, 0x0970, 1),
    (0x0df4, 0x0df4, 1),
    (0x0e4f, 0x0e4f, 1),
    (0x0e5a, 0x0e5b, 1),
    (0x0f04, 0x0f12, 1),
    (0x0f3a, 0x0f3d, 1),
    (0x0f85, 0x0f85, 1),
    (0x104a, 0x104f, 1),              // Myanmar punctuation
    (0x10fb, 0x10fb, 1),              // Georgian punctuation
    (0x1361, 0x1368, 1),              // Ethiopic punctuation
    (0x166d, 0x166e, 1),              // Canadian Syl. punctuation
    (0x1680, 0x1680, 0),
    (0x169b, 0x169c, 1),
    (0x16eb, 0x16ed, 1),
    (0x1735, 0x1736, 1),
    (0x17d4, 0x17dc, 1),              // Khmer punctuation
    (0x1800, 0x180a, 1),              // Mongolian punctuation
    (0x2000, 0x200b, 0),              // spaces
    (0x200c, 0x2027, 1),              // punctuation and symbols
    (0x2028, 0x2029, 0),
    (0x202a, 0x202e, 1),              // punctuation and symbols
    (0x202f, 0x202f, 0),
    (0x2030, 0x205e, 1),              // punctuation and symbols
    (0x205f, 0x205f, 0),
    (0x2060, 0x206f, 1),              // punctuation and symbols
    (0x2070, 0x207f, 0x2070),         // superscript
    (0x2080, 0x2094, 0x2080),         // subscript
    (0x20a0, 0x27ff, 1),              // all kinds of symbols
    (0x2800, 0x28ff, 0x2800),         // braille
    (0x2900, 0x2998, 1),              // arrows, brackets, etc.
    (0x29d8, 0x29db, 1),
    (0x29fc, 0x29fd, 1),
    (0x2e00, 0x2e7f, 1),              // supplemental punctuation
    (0x3000, 0x3000, 0),              // ideographic space
    (0x3001, 0x3020, 1),              // ideographic punctuation
    (0x3030, 0x3030, 1),
    (0x303d, 0x303d, 1),
    (0x3040, 0x309f, 0x3040),         // Hiragana
    (0x30a0, 0x30ff, 0x30a0),         // Katakana
    (0x3300, 0x9fff, 0x4e00),         // CJK Ideographs
    (0xac00, 0xd7a3, 0xac00),         // Hangul Syllables
    (0xf900, 0xfaff, 0x4e00),         // CJK Ideographs
    (0xfd3e, 0xfd3f, 1),
    (0xfe30, 0xfe6b, 1),              // punctuation forms
    (0xff00, 0xff0f, 1),              // half/fullwidth ASCII
    (0xff1a, 0xff20, 1),              // half/fullwidth ASCII
    (0xff3b, 0xff40, 1),              // half/fullwidth ASCII
    (0xff5b, 0xff65, 1),              // half/fullwidth ASCII
    (0x1d000, 0x1d24f, 1),            // Musical notation
    (0x1d400, 0x1d7ff, 1),            // Mathematical Alphanumeric Symbols
    (0x1f000, 0x1f2ff, 1),            // Game pieces; enclosed characters
    (0x1f300, 0x1f9ff, 1),            // Many symbol blocks
    (0x20000, 0x2a6df, 0x4e00),       // CJK Ideographs
    (0x2a700, 0x2b73f, 0x4e00),       // CJK Ideographs
    (0x2b740, 0x2b81f, 0x4e00),       // CJK Ideographs
    (0x2f800, 0x2fa1f, 0x4e00),       // CJK Ideographs
];

/// Check a bit in the buffer chartab (`uint64_t[4]` bitmap).
///
/// This replicates the C macro:
/// `GET_CHARTAB_TAB(chartab, c) ((chartab)[(unsigned)(c) >> 6] & (1ull << ((c) & 0x3f)))`
#[inline]
fn get_chartab_tab(chartab: &[u64; 4], c: u8) -> bool {
    let idx = (c >> 6) as usize;
    let bit = 1u64 << (c & 0x3f);
    (chartab[idx] & bit) != 0
}

/// Get the character class for a Unicode character.
///
/// This implements `utf_class_tab` from mbyte.c for characters >= 0x100.
/// For Latin1 characters (< 0x100), use the `vim_iswordc_tab` function.
///
/// Returns:
/// - 0 = blank (whitespace)
/// - 1 = punctuation
/// - 2 = word character
/// - 3 = emoji
/// - Other values = specific script classes (for keeping certain scripts separate)
///
/// The `chartab` parameter is ignored for c >= 0x100.
/// For c < 0x100, this function should NOT be called - use `vim_iswordc_tab` instead.
#[inline]
pub fn utf_class_tab_impl(c: i32, chartab: &[u64; 4]) -> i32 {
    // Latin1 characters (< 0x100) - use chartab
    if c < 0x100 {
        if c == i32::from(b' ') || c == i32::from(b'\t') || c == 0 || c == 0xa0 {
            return 0; // blank
        }
        if c > 0 && get_chartab_tab(chartab, c as u8) {
            return 2; // word character
        }
        return 1; // punctuation
    }

    // Check for emoji using utf8proc
    if let Some(prop) = get_property(c) {
        if prop.is_emojilike() {
            return 3;
        }
    }

    // Binary search in the classes table
    let c_u32 = c as u32;
    let mut bot = 0;
    let mut top = UTF_CLASS_TABLE.len() as i32 - 1;

    while top >= bot {
        let mid = i32::midpoint(bot, top);
        let (first, last, cls) = UTF_CLASS_TABLE[mid as usize];
        if last < c_u32 {
            bot = mid + 1;
        } else if first > c_u32 {
            top = mid - 1;
        } else {
            return cls as i32;
        }
    }

    // Most other characters are "word" characters
    2
}

/// FFI wrapper for `utf_class_tab_impl`.
///
/// # Safety
///
/// - `chartab` must be a valid pointer to a `[u64; 4]` array
#[no_mangle]
pub unsafe extern "C" fn rs_utf_class_tab(c: c_int, chartab: *const u64) -> c_int {
    if chartab.is_null() {
        // If no chartab, use default behavior (treat as punctuation for Latin1)
        if c < 0x100 {
            if c == i32::from(b' ') || c == i32::from(b'\t') || c == 0 || c == 0xa0 {
                return 0;
            }
            return 1;
        }
        return utf_class_tab_impl(c, &[0, 0, 0, 0]);
    }

    let chartab_arr: &[u64; 4] = &*(chartab as *const [u64; 4]);
    utf_class_tab_impl(c, chartab_arr)
}

/// Check if a character is ASCII whitespace (space or tab).
#[inline]
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Get the character class from a UTF-8 string pointer.
///
/// This is the Rust equivalent of `mb_get_class_tab` from mbyte.c.
///
/// Returns:
/// - 0 = blank (whitespace or NUL)
/// - 1 = punctuation
/// - 2 = word character
/// - >2 = other word characters (CJK, emoji, etc.)
#[inline]
pub fn mb_get_class_tab_impl(p: &[u8], chartab: &[u64; 4]) -> i32 {
    // Check if this is a single-byte (ASCII) character
    if UTF8LEN_TAB[p[0] as usize] == 1 {
        // Check for blank (NUL or whitespace)
        if p[0] == 0 || ascii_iswhite(p[0]) {
            return 0;
        }
        // Check chartab for word character
        if get_chartab_tab(chartab, p[0]) {
            return 2;
        }
        // Otherwise punctuation
        return 1;
    }
    // Multi-byte character - decode and get class
    let c = utf_ptr2char(p);
    utf_class_tab_impl(c, chartab)
}

/// FFI wrapper for `mb_get_class_tab_impl`.
///
/// Get the character class from a UTF-8 string pointer.
///
/// # Safety
///
/// - `p` must be a valid pointer to a NUL-terminated or sufficiently long UTF-8 string
/// - `chartab` must be a valid pointer to a `[u64; 4]` array
#[no_mangle]
pub unsafe extern "C" fn rs_mb_get_class_tab(p: *const c_char, chartab: *const u64) -> c_int {
    if p.is_null() || chartab.is_null() {
        return 0;
    }

    let first_byte = *(p as *const u8);
    let chartab_arr: &[u64; 4] = &*(chartab as *const [u64; 4]);

    // Check if this is a single-byte (ASCII) character
    if UTF8LEN_TAB[first_byte as usize] == 1 {
        // Check for blank (NUL or whitespace)
        if first_byte == 0 || ascii_iswhite(first_byte) {
            return 0;
        }
        // Check chartab for word character
        if get_chartab_tab(chartab_arr, first_byte) {
            return 2;
        }
        // Otherwise punctuation
        return 1;
    }

    // Multi-byte character - decode and get class using the FFI wrapper
    let c = rs_utf_ptr2char(p);
    utf_class_tab_impl(c, chartab_arr)
}

// ============================================================================
// UTF-8 string length measurement (Phase 2.78)
// ============================================================================

/// Measure the length of a string in corresponding UTF-32 and UTF-16 units.
///
/// Invalid UTF-8 bytes, or embedded surrogates, count as one code point/unit each.
///
/// The out parameters are incremented. This is used to measure the size of
/// a buffer region consisting of multiple line segments.
///
/// # Arguments
/// * `s` - the string slice to measure
/// * `codepoints` - incremented with UTF-32 code point count
/// * `codeunits` - incremented with UTF-16 code unit count
pub fn mb_utflen(s: &[u8], codepoints: &mut usize, codeunits: &mut usize) {
    let mut count: usize = 0;
    let mut extra: usize = 0;
    let mut i: usize = 0;

    while i < s.len() {
        let clen = utf_ptr2len_len(&s[i..], s.len() - i);
        // NB: gets the byte value of invalid sequence bytes.
        // we only care whether the char fits in the BMP or not
        let c = if clen > 1 {
            utf_ptr2char(&s[i..])
        } else {
            i32::from(s[i])
        };
        count += 1;
        if c > 0xFFFF {
            extra += 1;
        }
        i += clen;
    }
    *codepoints += count;
    *codeunits += count + extra;
}

/// FFI wrapper for mb_utflen.
///
/// # Safety
/// `s` must be a valid pointer to `len` bytes. `codepoints` and `codeunits`
/// must be valid pointers to writable usize values.
#[no_mangle]
pub unsafe extern "C" fn rs_mb_utflen(
    s: *const c_char,
    len: usize,
    codepoints: *mut usize,
    codeunits: *mut usize,
) {
    if s.is_null() || codepoints.is_null() || codeunits.is_null() {
        return;
    }
    let slice = std::slice::from_raw_parts(s as *const u8, len);
    mb_utflen(slice, &mut *codepoints, &mut *codeunits);
}

/// Convert a UTF-16 or UTF-32 index to byte offset.
///
/// Returns the byte offset corresponding to the given character index,
/// or -1 if the index is beyond the string length.
///
/// # Arguments
/// * `s` - the UTF-8 string slice
/// * `index` - the character index (0-based)
/// * `use_utf16_units` - if true, count UTF-16 code units (surrogates count as 2)
pub fn mb_utf_index_to_bytes(s: &[u8], index: usize, use_utf16_units: bool) -> isize {
    let mut count: usize = 0;
    let mut i: usize = 0;

    if index == 0 {
        return 0;
    }

    while i < s.len() {
        let clen = utf_ptr2len_len(&s[i..], s.len() - i);
        // NB: gets the byte value of invalid sequence bytes.
        // we only care whether the char fits in the BMP or not
        let c = if clen > 1 {
            utf_ptr2char(&s[i..])
        } else {
            i32::from(s[i])
        };
        count += 1;
        if use_utf16_units && c > 0xFFFF {
            count += 1;
        }
        if count >= index {
            return (i + clen) as isize;
        }
        i += clen;
    }
    -1
}

/// FFI wrapper for mb_utf_index_to_bytes.
///
/// # Safety
/// `s` must be a valid pointer to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_mb_utf_index_to_bytes(
    s: *const c_char,
    len: usize,
    index: usize,
    use_utf16_units: bool,
) -> isize {
    if s.is_null() {
        return -1;
    }
    let slice = std::slice::from_raw_parts(s as *const u8, len);
    mb_utf_index_to_bytes(slice, index, use_utf16_units)
}

/// Like `utf_ptr2cells()`, but limit string length to `size`.
///
/// For an empty string or truncated character returns 1.
/// For valid multibyte UTF-8, returns the character's cell width.
/// For invalid/illegal UTF-8 sequences, returns 4 (displayed as `<xx>`).
/// For overlong ASCII encodings, returns char2cells of the decoded value.
///
/// Also checks for emoji presentation selector (VS-16) following emoji-like
/// characters, returning 2 cells if found (when 'emoji' option is enabled).
pub fn utf_ptr2cells_len(p: &[u8], size: usize) -> i32 {
    // Empty or beyond size limit
    if size == 0 || p.is_empty() {
        return 1;
    }

    let first = p[0];

    // ASCII fast path
    if first < 0x80 {
        return 1;
    }

    // Multibyte - limit slice to size
    let effective_len = p.len().min(size);
    let p = &p[..effective_len];

    // Get the expected UTF-8 sequence length from the first byte
    let expected_len = UTF8LEN_TAB[first as usize] as usize;

    // Get actual valid length within bounds
    let actual_len = utf_ptr2len_len(p, p.len());

    // If truncated (actual < expected), return 1
    if actual_len < expected_len {
        return 1;
    }

    // Try to decode the character
    let c = utf_ptr2char(p);

    // Check for illegal byte (utf_ptr2len returns 1 for invalid sequences)
    // or NUL character
    if utf_ptr2len(p) == 1 || c == 0 {
        return 4;
    }

    // If the char is ASCII it must be an overlong sequence
    if c < 0x80 {
        // SAFETY: rs_char2cells just accesses g_chartab
        return unsafe { rs_char2cells(c) };
    }

    let cells = utf_char2cells(c);

    // Check for emoji presentation selector (VS-16)
    if cells == 1 && emoji_is_enabled() {
        if let Some(prop) = get_property(c) {
            if prop.is_emojilike() && effective_len > actual_len {
                // There's more data after this character
                let remaining = &p[actual_len..];
                // Check if the next character is complete
                let next_expected = UTF8LEN_TAB[remaining[0] as usize] as usize;
                let next_actual = utf_ptr2len_len(remaining, remaining.len());
                if next_actual == next_expected {
                    let c2 = utf_ptr2char(remaining);
                    if c2 == VS16_CODEPOINT {
                        return 2; // emoji presentation
                    }
                }
            }
        }
    }

    cells
}

/// FFI wrapper for `utf_ptr2cells_len`.
///
/// # Safety
///
/// `p` must be a valid pointer to at least `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_utf_ptr2cells_len(p: *const c_char, size: c_int) -> c_int {
    if p.is_null() || size <= 0 {
        return 1;
    }

    let slice = std::slice::from_raw_parts(p as *const u8, size as usize);
    utf_ptr2cells_len(slice, size as usize)
}

// =============================================================================
// mb_cptr2char_adv - Get character at pointer and advance (composing chars as separate)
// =============================================================================

/// Get character at pointer and advance the pointer to the next character.
///
/// Unlike `mb_ptr2char_adv` which skips composing characters, this function
/// returns composing characters as separate characters.
///
/// # Arguments
/// * `p` - Pointer to a NUL-terminated UTF-8 string
///
/// # Returns
/// The Unicode codepoint of the character at the current position
///
/// # Safety
/// The pointer must point to a valid NUL-terminated string.
#[inline]
pub fn mb_cptr2char_adv(p: &[u8]) -> (i32, usize) {
    let c = utf_ptr2char(p);
    let len = utf_ptr2len(p);
    (c, len as usize)
}

/// FFI wrapper for mb_cptr2char_adv.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a NUL-terminated string
/// - The pointer pointed to by `pp` will be advanced
#[no_mangle]
pub unsafe extern "C" fn rs_mb_cptr2char_adv(pp: *mut *const c_char) -> c_int {
    if pp.is_null() {
        return 0;
    }

    let p = *pp;
    if p.is_null() {
        return 0;
    }

    // Find the NUL terminator to get the slice length
    let mut len = 0usize;
    while *p.add(len) != 0 {
        len += 1;
        // Safety: Limit to reasonable length to avoid infinite loop
        if len > 16 * 1024 * 1024 {
            break;
        }
    }

    if len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(p as *const u8, len);
    let (c, advance) = mb_cptr2char_adv(slice);
    *pp = p.add(advance);
    c
}

// =============================================================================
// enc_skip - Skip Vim-specific encoding name prefixes (Phase 2.84)
// =============================================================================

/// Skip the Vim-specific head of an 'encoding' name.
///
/// Vim supports encoding names with prefixes like "2byte-" and "8bit-".
/// This function returns a pointer past these prefixes if present.
///
/// Examples:
/// - "2byte-utf-8" -> "utf-8"
/// - "8bit-latin1" -> "latin1"
/// - "utf-8" -> "utf-8" (unchanged)
///
/// # Safety
///
/// `p` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_enc_skip(p: *mut c_char) -> *mut c_char {
    if p.is_null() {
        return p;
    }

    // Check for "2byte-" prefix (6 bytes)
    if libc::strncmp(p, c"2byte-".as_ptr(), 6) == 0 {
        return p.add(6);
    }

    // Check for "8bit-" prefix (5 bytes)
    if libc::strncmp(p, c"8bit-".as_ptr(), 5) == 0 {
        return p.add(5);
    }

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enc_skip() {
        unsafe {
            // Test "2byte-" prefix
            let s = c"2byte-utf-8";
            let result = rs_enc_skip(s.as_ptr() as *mut c_char);
            assert_eq!(libc::strcmp(result, c"utf-8".as_ptr()), 0);

            // Test "8bit-" prefix
            let s = c"8bit-latin1";
            let result = rs_enc_skip(s.as_ptr() as *mut c_char);
            assert_eq!(libc::strcmp(result, c"latin1".as_ptr()), 0);

            // Test no prefix
            let s = c"utf-8";
            let result = rs_enc_skip(s.as_ptr() as *mut c_char);
            assert_eq!(libc::strcmp(result, c"utf-8".as_ptr()), 0);

            // Test empty string
            let s = c"";
            let result = rs_enc_skip(s.as_ptr() as *mut c_char);
            assert_eq!(libc::strcmp(result, c"".as_ptr()), 0);
        }
    }

    #[test]
    fn test_utf8len_tab() {
        // ASCII
        assert_eq!(UTF8LEN_TAB[0x00], 1);
        assert_eq!(UTF8LEN_TAB[0x7F], 1);

        // Continuation bytes (invalid as first byte)
        assert_eq!(UTF8LEN_TAB[0x80], 1);
        assert_eq!(UTF8LEN_TAB[0xBF], 1);

        // 2-byte sequences
        assert_eq!(UTF8LEN_TAB[0xC0], 2);
        assert_eq!(UTF8LEN_TAB[0xDF], 2);

        // 3-byte sequences
        assert_eq!(UTF8LEN_TAB[0xE0], 3);
        assert_eq!(UTF8LEN_TAB[0xEF], 3);

        // 4-byte sequences
        assert_eq!(UTF8LEN_TAB[0xF0], 4);
        assert_eq!(UTF8LEN_TAB[0xF7], 4);

        // 5-byte sequences
        assert_eq!(UTF8LEN_TAB[0xF8], 5);
        assert_eq!(UTF8LEN_TAB[0xFB], 5);

        // 6-byte sequences
        assert_eq!(UTF8LEN_TAB[0xFC], 6);
        assert_eq!(UTF8LEN_TAB[0xFD], 6);

        // Invalid
        assert_eq!(UTF8LEN_TAB[0xFE], 1);
        assert_eq!(UTF8LEN_TAB[0xFF], 1);
    }

    #[test]
    fn test_utf_char2len() {
        assert_eq!(utf_char2len(0x00), 1); // NUL
        assert_eq!(utf_char2len(0x7F), 1); // DEL
        assert_eq!(utf_char2len(0x80), 2); // First 2-byte
        assert_eq!(utf_char2len(0x7FF), 2); // Last 2-byte
        assert_eq!(utf_char2len(0x800), 3); // First 3-byte
        assert_eq!(utf_char2len(0xFFFF), 3); // Last 3-byte
        assert_eq!(utf_char2len(0x10000), 4); // First 4-byte
        assert_eq!(utf_char2len(0x10FFFF), 4); // Unicode max
    }

    #[test]
    fn test_utf_char2bytes() {
        let mut buf = [0u8; 6];

        // ASCII
        assert_eq!(utf_char2bytes(b'A' as i32, &mut buf), 1);
        assert_eq!(buf[0], b'A');

        // 2-byte (Latin small letter a with acute: U+00E1)
        assert_eq!(utf_char2bytes(0xE1, &mut buf), 2);
        assert_eq!(&buf[..2], &[0xC3, 0xA1]);

        // 3-byte (Euro sign: U+20AC)
        assert_eq!(utf_char2bytes(0x20AC, &mut buf), 3);
        assert_eq!(&buf[..3], &[0xE2, 0x82, 0xAC]);

        // 4-byte (Emoji: U+1F600)
        assert_eq!(utf_char2bytes(0x1F600, &mut buf), 4);
        assert_eq!(&buf[..4], &[0xF0, 0x9F, 0x98, 0x80]);
    }

    #[test]
    fn test_utf_ptr2char() {
        // ASCII
        assert_eq!(utf_ptr2char(b"A"), b'A' as i32);
        assert_eq!(utf_ptr2char(b"Hello"), b'H' as i32);

        // 2-byte (Latin small letter a with acute: U+00E1)
        assert_eq!(utf_ptr2char(&[0xC3, 0xA1]), 0xE1);

        // 3-byte (Euro sign: U+20AC)
        assert_eq!(utf_ptr2char(&[0xE2, 0x82, 0xAC]), 0x20AC);

        // 4-byte (Emoji: U+1F600)
        assert_eq!(utf_ptr2char(&[0xF0, 0x9F, 0x98, 0x80]), 0x1F600);

        // Invalid continuation byte - returns first byte
        assert_eq!(utf_ptr2char(&[0xC3, 0x00]), 0xC3);
    }

    #[test]
    fn test_utf_ptr2len() {
        // Empty/NUL
        assert_eq!(utf_ptr2len(&[]), 0);
        assert_eq!(utf_ptr2len(&[0]), 0);

        // ASCII
        assert_eq!(utf_ptr2len(b"A"), 1);
        assert_eq!(utf_ptr2len(b"Hello"), 1);

        // 2-byte
        assert_eq!(utf_ptr2len(&[0xC3, 0xA1]), 2);

        // 3-byte
        assert_eq!(utf_ptr2len(&[0xE2, 0x82, 0xAC]), 3);

        // 4-byte
        assert_eq!(utf_ptr2len(&[0xF0, 0x9F, 0x98, 0x80]), 4);

        // Invalid - continuation byte as first byte
        assert_eq!(utf_ptr2len(&[0x80]), 1);

        // Invalid - missing continuation bytes
        assert_eq!(utf_ptr2len(&[0xC3]), 1);
    }

    #[test]
    fn test_utf_byte2len() {
        assert_eq!(utf_byte2len(0x00), 1);
        assert_eq!(utf_byte2len(0x7F), 1);
        assert_eq!(utf_byte2len(0xC0), 2);
        assert_eq!(utf_byte2len(0xE0), 3);
        assert_eq!(utf_byte2len(0xF0), 4);
    }

    #[test]
    fn test_utf_is_continuation() {
        // Not continuation bytes
        assert!(!utf_is_continuation(0x00));
        assert!(!utf_is_continuation(0x7F));
        assert!(!utf_is_continuation(0xC0));

        // Continuation bytes
        assert!(utf_is_continuation(0x80));
        assert!(utf_is_continuation(0xBF));
    }

    #[test]
    fn test_utf_valid() {
        assert!(utf_valid(0));
        assert!(utf_valid(0x7F));
        assert!(utf_valid(0x10FFFF));

        // Invalid: surrogate pairs
        assert!(!utf_valid(0xD800));
        assert!(!utf_valid(0xDFFF));

        // Invalid: out of range
        assert!(!utf_valid(-1));
        assert!(!utf_valid(0x110000));
    }

    #[test]
    fn test_roundtrip() {
        let test_chars = [
            0x41,     // 'A'
            0xE1,     // Latin small letter a with acute
            0x20AC,   // Euro sign
            0x1F600,  // Grinning face emoji
            0x10FFFF, // Maximum valid Unicode
        ];

        for &c in &test_chars {
            let mut buf = [0u8; 6];
            let len = utf_char2bytes(c, &mut buf);
            let decoded = utf_ptr2char(&buf[..len]);
            assert_eq!(decoded, c, "Roundtrip failed for codepoint {:X}", c);
        }
    }

    #[test]
    fn test_ffi_utf_char2len() {
        assert_eq!(rs_utf_char2len(0x41), 1);
        assert_eq!(rs_utf_char2len(0x20AC), 3);
    }

    #[test]
    fn test_ffi_utf_byte2len() {
        assert_eq!(rs_utf_byte2len(0x41), 1);
        assert_eq!(rs_utf_byte2len(0xC0), 2);
        assert_eq!(rs_utf_byte2len(-1), 1);
        assert_eq!(rs_utf_byte2len(256), 1);
    }

    #[test]
    fn test_ffi_utf_char2bytes() {
        let mut buf = [0i8; 6];
        let len = unsafe { rs_utf_char2bytes(0x20AC, buf.as_mut_ptr()) };
        assert_eq!(len, 3);
        assert_eq!(buf[0] as u8, 0xE2);
        assert_eq!(buf[1] as u8, 0x82);
        assert_eq!(buf[2] as u8, 0xAC);
    }

    #[test]
    fn test_ffi_utf_ptr2char() {
        use std::ffi::CString;

        let s = CString::new([0xE2u8, 0x82, 0xAC].as_slice()).unwrap();
        let c = unsafe { rs_utf_ptr2char(s.as_ptr()) };
        assert_eq!(c, 0x20AC);
    }

    #[test]
    fn test_ffi_utf_ptr2len() {
        use std::ffi::CString;

        let s = CString::new([0xE2u8, 0x82, 0xAC].as_slice()).unwrap();
        let len = unsafe { rs_utf_ptr2len(s.as_ptr()) };
        assert_eq!(len, 3);

        let ascii = CString::new("Hello").unwrap();
        let len = unsafe { rs_utf_ptr2len(ascii.as_ptr()) };
        assert_eq!(len, 1);
    }

    #[test]
    fn test_utf_printable() {
        // Normal printable characters
        assert!(utf_printable(0x100)); // Latin Extended
        assert!(utf_printable(0x4E00)); // CJK Unified
        assert!(utf_printable(0x1F600)); // Emoji

        // Non-printable: Syriac abbreviation mark
        assert!(!utf_printable(0x070f));

        // Non-printable: Mongolian free variation selectors
        assert!(!utf_printable(0x180b));
        assert!(!utf_printable(0x180e));

        // Non-printable: Zero width space and directional marks
        assert!(!utf_printable(0x200b));
        assert!(!utf_printable(0x200f));

        // Non-printable: Embedding/override controls
        assert!(!utf_printable(0x202a));
        assert!(!utf_printable(0x202e));

        // Non-printable: Word joiner and invisible operators
        assert!(!utf_printable(0x2060));
        assert!(!utf_printable(0x206f));

        // Non-printable: Surrogates (illegal in UTF-8)
        assert!(!utf_printable(0xd800));
        assert!(!utf_printable(0xdfff));

        // Non-printable: BOM
        assert!(!utf_printable(0xfeff));

        // Non-printable: Interlinear annotation anchors
        assert!(!utf_printable(0xfff9));
        assert!(!utf_printable(0xfffb));

        // Non-printable: Non-characters
        assert!(!utf_printable(0xfffe));
        assert!(!utf_printable(0xffff));

        // Edge cases - just outside non-print ranges
        assert!(utf_printable(0x070e)); // before Syriac abbreviation
        assert!(utf_printable(0x0710)); // after Syriac abbreviation
        assert!(utf_printable(0x180a)); // before Mongolian FSV
        assert!(utf_printable(0x180f)); // after Mongolian FSV
    }

    #[test]
    fn test_ffi_utf_printable() {
        assert_eq!(rs_utf_printable(0x100), 1);
        assert_eq!(rs_utf_printable(0x200b), 0); // Zero width space
        assert_eq!(rs_utf_printable(0xd800), 0); // Surrogate
    }

    #[test]
    fn test_utf_iscomposing_legacy() {
        // U+0300 COMBINING GRAVE ACCENT is a nonspacing mark (Mn)
        assert!(utf_iscomposing_legacy(0x0300));

        // U+20DD COMBINING ENCLOSING CIRCLE is an enclosing mark (Me)
        assert!(utf_iscomposing_legacy(0x20DD));

        // ASCII 'A' is not composing
        assert!(!utf_iscomposing_legacy(0x41));

        // Space is not composing
        assert!(!utf_iscomposing_legacy(0x20));

        // Negative values return false
        assert!(!utf_iscomposing_legacy(-1));
    }

    #[test]
    fn test_ffi_utf_iscomposing_legacy() {
        assert_eq!(rs_utf_iscomposing_legacy(0x0300), 1);
        assert_eq!(rs_utf_iscomposing_legacy(0x20DD), 1);
        assert_eq!(rs_utf_iscomposing_legacy(0x41), 0);
    }

    #[test]
    fn test_utf_iscomposing_first() {
        // U+0300 COMBINING GRAVE ACCENT - would combine with preceding space
        assert!(utf_iscomposing_first(0x0300));

        // ASCII 'A' - would not combine (has grapheme break)
        assert!(!utf_iscomposing_first(0x41));

        // ASCII characters below 128 always return false
        assert!(!utf_iscomposing_first(b' ' as i32));
        assert!(!utf_iscomposing_first(b'a' as i32));
    }

    #[test]
    fn test_ffi_utf_iscomposing_first() {
        assert_eq!(rs_utf_iscomposing_first(0x0300), 1);
        assert_eq!(rs_utf_iscomposing_first(0x41), 0);
        assert_eq!(rs_utf_iscomposing_first(b'a' as c_int), 0);
    }

    #[test]
    fn test_utf_fold() {
        // ASCII uppercase to lowercase
        assert_eq!(utf_fold(b'A' as i32), b'a' as i32);
        assert_eq!(utf_fold(b'Z' as i32), b'z' as i32);

        // ASCII lowercase unchanged
        assert_eq!(utf_fold(b'a' as i32), b'a' as i32);
        assert_eq!(utf_fold(b'z' as i32), b'z' as i32);

        // ASCII non-letters unchanged
        assert_eq!(utf_fold(b'0' as i32), b'0' as i32);
        assert_eq!(utf_fold(b'!' as i32), b'!' as i32);

        // Special cases that should remain unchanged
        assert_eq!(utf_fold(0xDF), 0xDF); // ß
        assert_eq!(utf_fold(0x130), 0x130); // İ
    }

    #[test]
    fn test_ffi_utf_fold() {
        assert_eq!(rs_utf_fold(b'A' as c_int), b'a' as c_int);
        assert_eq!(rs_utf_fold(b'a' as c_int), b'a' as c_int);
        assert_eq!(rs_utf_fold(0xDF), 0xDF); // ß unchanged
    }

    #[test]
    fn test_utf_ambiguous_width() {
        // Empty/NUL - not ambiguous
        assert!(!utf_ambiguous_width(&[]));
        assert!(!utf_ambiguous_width(&[0]));

        // ASCII - not ambiguous
        assert!(!utf_ambiguous_width(b"A"));
        assert!(!utf_ambiguous_width(b"Hello"));

        // ASCII followed by NUL - not ambiguous
        assert!(!utf_ambiguous_width(&[b'A', 0]));

        // Ambiguous width character: U+00A7 (§ section sign)
        // UTF-8: 0xC2 0xA7
        assert!(utf_ambiguous_width(&[0xC2, 0xA7]));

        // Ambiguous width character: U+00B0 (° degree sign)
        // UTF-8: 0xC2 0xB0
        assert!(utf_ambiguous_width(&[0xC2, 0xB0]));

        // Extended pictographic (emoji): U+1F600 (grinning face)
        // UTF-8: 0xF0 0x9F 0x98 0x80
        assert!(utf_ambiguous_width(&[0xF0, 0x9F, 0x98, 0x80]));

        // Regional indicator: U+1F1E6 (regional indicator A)
        // UTF-8: 0xF0 0x9F 0x87 0xA6
        assert!(utf_ambiguous_width(&[0xF0, 0x9F, 0x87, 0xA6]));

        // Non-ambiguous CJK character: U+4E2D (中)
        // UTF-8: 0xE4 0xB8 0xAD
        assert!(!utf_ambiguous_width(&[0xE4, 0xB8, 0xAD]));

        // Character followed by VS-16 (U+FE0F) - becomes emoji
        // Example: # + VS-16 = emoji presentation
        // '#' (0x23) followed by VS-16 (0xEF 0xB8 0x8F)
        assert!(utf_ambiguous_width(&[0x23, 0xEF, 0xB8, 0x8F]));

        // Character not followed by VS-16 - just normal #
        assert!(!utf_ambiguous_width(&[0x23, 0]));
    }

    #[test]
    fn test_ffi_utf_ambiguous_width() {
        use std::ffi::CString;

        // ASCII - not ambiguous
        let s = CString::new("A").unwrap();
        assert_eq!(unsafe { rs_utf_ambiguous_width(s.as_ptr()) }, 0);

        // Ambiguous: § (U+00A7)
        let s = CString::new([0xC2u8, 0xA7].as_slice()).unwrap();
        assert_eq!(unsafe { rs_utf_ambiguous_width(s.as_ptr()) }, 1);

        // Emoji: 😀 (U+1F600)
        let s = CString::new([0xF0u8, 0x9F, 0x98, 0x80].as_slice()).unwrap();
        assert_eq!(unsafe { rs_utf_ambiguous_width(s.as_ptr()) }, 1);
    }

    #[test]
    fn test_utf_allow_break() {
        // Normal characters - break is allowed
        assert!(utf_allow_break(b'a' as i32, b'b' as i32));

        // Don't break between two em dashes (U+2014)
        assert!(!utf_allow_break(0x2014, 0x2014));

        // Don't break between two horizontal ellipses (U+2026)
        assert!(!utf_allow_break(0x2026, 0x2026));

        // But can break between different punctuation when allowed
        // Note: 0x2026 is in BOL prohibition list, so can't break before it
        // Use a character that allows breaking
        assert!(utf_allow_break(0x2014, b'a' as i32));

        // Don't break before closing bracket
        assert!(!utf_allow_break(b'a' as i32, b')' as i32));

        // Don't break after opening bracket
        assert!(!utf_allow_break(b'(' as i32, b'a' as i32));
    }

    #[test]
    fn test_ffi_utf_allow_break() {
        assert_eq!(rs_utf_allow_break(b'a' as c_int, b'b' as c_int), 1);
        assert_eq!(rs_utf_allow_break(0x2014, 0x2014), 0); // Two em dashes
        assert_eq!(rs_utf_allow_break(0x2026, 0x2026), 0); // Two horizontal ellipses
    }

    #[test]
    fn test_utf_cp_bounds_ascii() {
        // ASCII: pointing at 'e'
        let s = b"Hello";
        let bounds = utf_cp_bounds(s, 1);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1);
    }

    #[test]
    fn test_utf_cp_bounds_2byte() {
        // 2-byte UTF-8: é (U+00E9) = [0xC3, 0xA9]
        let s = &[0xC3u8, 0xA9];

        // Pointing at first byte
        let bounds = utf_cp_bounds(s, 0);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 2);

        // Pointing at second byte (continuation)
        let bounds = utf_cp_bounds(s, 1);
        assert_eq!(bounds.begin_off, 1); // 1 byte back to first byte
        assert_eq!(bounds.end_off, 1); // 1 byte forward to end
    }

    #[test]
    fn test_utf_cp_bounds_3byte() {
        // 3-byte UTF-8: € (U+20AC) = [0xE2, 0x82, 0xAC]
        let s = &[0xE2u8, 0x82, 0xAC];

        // Pointing at first byte
        let bounds = utf_cp_bounds(s, 0);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 3);

        // Pointing at second byte
        let bounds = utf_cp_bounds(s, 1);
        assert_eq!(bounds.begin_off, 1);
        assert_eq!(bounds.end_off, 2);

        // Pointing at third byte
        let bounds = utf_cp_bounds(s, 2);
        assert_eq!(bounds.begin_off, 2);
        assert_eq!(bounds.end_off, 1);
    }

    #[test]
    fn test_utf_cp_bounds_4byte() {
        // 4-byte UTF-8: 😀 (U+1F600) = [0xF0, 0x9F, 0x98, 0x80]
        let s = &[0xF0u8, 0x9F, 0x98, 0x80];

        // Pointing at first byte
        let bounds = utf_cp_bounds(s, 0);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 4);

        // Pointing at third byte
        let bounds = utf_cp_bounds(s, 2);
        assert_eq!(bounds.begin_off, 2);
        assert_eq!(bounds.end_off, 2);
    }

    #[test]
    fn test_utf_cp_bounds_len_limited() {
        // 3-byte UTF-8 but limited to 2 bytes
        let s = &[0xE2u8, 0x82, 0xAC];

        // Limited to 2 bytes - incomplete sequence
        let bounds = utf_cp_bounds_len(s, 0, 2);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1); // Invalid - returns 1

        // Full 3 bytes available
        let bounds = utf_cp_bounds_len(s, 0, 3);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 3);
    }

    #[test]
    fn test_utf_cp_bounds_mixed_string() {
        // "Héllo" = ['H', 0xC3, 0xA9, 'l', 'l', 'o']
        let s = &[b'H', 0xC3, 0xA9, b'l', b'l', b'o'];

        // Pointing at 'H'
        let bounds = utf_cp_bounds(s, 0);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1);

        // Pointing at first byte of 'é'
        let bounds = utf_cp_bounds(s, 1);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 2);

        // Pointing at second byte of 'é'
        let bounds = utf_cp_bounds(s, 2);
        assert_eq!(bounds.begin_off, 1);
        assert_eq!(bounds.end_off, 1);

        // Pointing at 'l'
        let bounds = utf_cp_bounds(s, 3);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1);
    }

    #[test]
    fn test_utf_cp_bounds_invalid() {
        // Invalid: lone continuation byte
        let s = &[0x80u8];
        let bounds = utf_cp_bounds(s, 0);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1);

        // Invalid: missing continuation bytes
        let s = &[0xC3u8]; // Should be followed by continuation
        let bounds = utf_cp_bounds(s, 0);
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1);
    }

    #[test]
    fn test_ffi_utf_cp_bounds() {
        use std::ffi::CString;

        // "Héllo"
        let s = CString::new([b'H', 0xC3, 0xA9, b'l', b'l', b'o'].as_slice()).unwrap();
        let base = s.as_ptr();

        // Pointing at 'H'
        let bounds = unsafe { rs_utf_cp_bounds(base, base) };
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1);

        // Pointing at first byte of 'é'
        let bounds = unsafe { rs_utf_cp_bounds(base, base.add(1)) };
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 2);

        // Pointing at second byte of 'é'
        let bounds = unsafe { rs_utf_cp_bounds(base, base.add(2)) };
        assert_eq!(bounds.begin_off, 1);
        assert_eq!(bounds.end_off, 1);
    }

    #[test]
    fn test_ffi_utf_cp_bounds_len() {
        // Euro sign: [0xE2, 0x82, 0xAC]
        let s = [0xE2u8 as i8, 0x82u8 as i8, 0xACu8 as i8, 0];
        let base = s.as_ptr();

        // Full length available
        let bounds = unsafe { rs_utf_cp_bounds_len(base, base, 3) };
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 3);

        // Limited length - incomplete
        let bounds = unsafe { rs_utf_cp_bounds_len(base, base, 2) };
        assert_eq!(bounds.begin_off, 0);
        assert_eq!(bounds.end_off, 1); // Invalid due to incomplete sequence
    }

    #[test]
    fn test_utf_ptr2char_strict() {
        // Valid ASCII
        assert_eq!(utf_ptr2char_strict(b"A"), Some(0x41));

        // Valid 2-byte UTF-8: é (U+00E9)
        assert_eq!(utf_ptr2char_strict(&[0xC3, 0xA9]), Some(0xE9));

        // Valid 3-byte UTF-8: € (U+20AC)
        assert_eq!(utf_ptr2char_strict(&[0xE2, 0x82, 0xAC]), Some(0x20AC));

        // Valid 4-byte UTF-8: 😀 (U+1F600)
        assert_eq!(
            utf_ptr2char_strict(&[0xF0, 0x9F, 0x98, 0x80]),
            Some(0x1F600)
        );

        // Invalid: lone continuation byte
        assert_eq!(utf_ptr2char_strict(&[0x80]), None);

        // Invalid: incomplete 2-byte sequence
        assert_eq!(utf_ptr2char_strict(&[0xC3]), None);

        // Invalid: incomplete 3-byte sequence
        assert_eq!(utf_ptr2char_strict(&[0xE2, 0x82]), None);

        // Invalid: bad continuation byte
        assert_eq!(utf_ptr2char_strict(&[0xC3, 0x00]), None);

        // Empty input
        assert_eq!(utf_ptr2char_strict(&[]), None);
    }

    #[test]
    fn test_utf_ptr2cells_ascii() {
        // ASCII returns 1
        assert_eq!(utf_ptr2cells(b"A"), 1);
        assert_eq!(utf_ptr2cells(b"z"), 1);
        assert_eq!(utf_ptr2cells(b" "), 1);
    }

    #[test]
    fn test_utf_ptr2cells_invalid() {
        // Invalid sequences return 4 (displayed as <xx>)
        // Lone continuation byte
        assert_eq!(utf_ptr2cells(&[0x80]), 4);

        // Incomplete 2-byte sequence
        assert_eq!(utf_ptr2cells(&[0xC3]), 4);

        // Bad continuation byte in 2-byte sequence
        assert_eq!(utf_ptr2cells(&[0xC3, 0x00]), 4);
    }

    #[test]
    fn test_utf_ptr2cells_empty() {
        // Empty returns 1 (default)
        assert_eq!(utf_ptr2cells(&[]), 1);
    }
}

// ============================================================================
// Composing character detection and utfc_ptr2len
// ============================================================================

/// Check if two characters form a composing sequence.
///
/// Uses the Unicode grapheme break algorithm (stateful) and Arabic combining rules
/// to determine if `second` should compose with the character at `first`.
///
/// Returns true if there is NO grapheme break between the characters (they combine).
#[inline]
pub fn utf_composinglike(first: i32, second: i32, state: &mut i32) -> bool {
    // Use stateful grapheme break algorithm
    if !grapheme_break_stateful(first, second, state) {
        return true;
    }

    // Check Arabic combining (Lam + Alef ligatures)
    nvim_arabic::arabic_combine(first, second)
}

/// Check if two characters form a composing sequence (UCS-4 interface).
///
/// Same as `utf_composinglike` but taking raw codepoint values.
#[inline]
pub fn utf_iscomposing(c1: i32, c2: i32, state: &mut i32) -> bool {
    !grapheme_break_stateful(c1, c2, state) || nvim_arabic::arabic_combine(c1, c2)
}

/// C-compatible wrapper for `utf_composinglike`.
///
/// Takes two string pointers and checks if they form a composing sequence.
/// `state` must point to an i32 initialized to 0 at the start of processing.
///
/// # Safety
/// - `p1` and `p2` must be valid pointers to UTF-8 strings
/// - `state` must be a valid pointer to an i32
#[no_mangle]
pub unsafe extern "C" fn rs_utf_composinglike(
    p1: *const c_char,
    p2: *const c_char,
    state: *mut i32,
) -> bool {
    if p1.is_null() || p2.is_null() || state.is_null() {
        return false;
    }

    // Quick check: if p2 points to ASCII, it's definitely not composing
    if (*(p2 as *const u8)) < 0x80 {
        return false;
    }

    // Create slices from pointers (6 bytes is max UTF-8 char length)
    let slice1 = std::slice::from_raw_parts(p1 as *const u8, 6);
    let slice2 = std::slice::from_raw_parts(p2 as *const u8, 6);

    let first = utf_ptr2char(slice1);
    let second = utf_ptr2char(slice2);

    utf_composinglike(first, second, &mut *state)
}

/// C-compatible wrapper for `utf_iscomposing`.
///
/// # Safety
/// - `state` must be a valid pointer to an i32
#[no_mangle]
pub unsafe extern "C" fn rs_utf_iscomposing(c1: c_int, c2: c_int, state: *mut i32) -> bool {
    if state.is_null() {
        return false;
    }
    utf_iscomposing(c1, c2, &mut *state)
}

/// Return the number of bytes occupied by a UTF-8 character in a string.
/// This includes following composing characters.
///
/// Returns zero for NUL.
///
/// This is a pure Rust implementation that doesn't require callbacks to C.
pub fn utfc_ptr2len(p: &[u8]) -> usize {
    if p.is_empty() || p[0] == 0 {
        return 0;
    }

    let b0 = p[0];

    // Fast path for ASCII followed by ASCII
    if b0 < 0x80 && p.len() > 1 && p[1] < 0x80 {
        return 1;
    }

    // Skip over first UTF-8 char, stopping at a NUL byte
    let len = utf_ptr2len(p);

    // Check for illegal byte
    if len == 1 && b0 >= 0x80 {
        return 1;
    }

    // Check for composing characters
    let mut total_len = len;
    let mut prevlen = 0usize;
    let mut state: i32 = 0; // GRAPHEME_STATE_INIT

    loop {
        if total_len >= p.len() || p[total_len] == 0 {
            return total_len;
        }

        // Quick check: ASCII is never composing
        if p[total_len] < 0x80 {
            return total_len;
        }

        // Get the codepoints for comparison
        let prev_char = utf_ptr2char(&p[prevlen..]);
        let next_char = utf_ptr2char(&p[total_len..]);

        if !utf_composinglike(prev_char, next_char, &mut state) {
            return total_len;
        }

        // Skip over composing char
        prevlen = total_len;
        total_len += utf_ptr2len(&p[total_len..]);
    }
}

/// Return the number of bytes the UTF-8 encoding of the character at "p[size]" takes.
/// This includes following composing characters.
///
/// Returns 0 for an empty string.
/// Returns 1 for an illegal char or an incomplete byte sequence.
pub fn utfc_ptr2len_len(p: &[u8], size: usize) -> usize {
    if size == 0 || p.is_empty() || p[0] == 0 {
        return 0;
    }

    let b0 = p[0];

    // Fast path for ASCII followed by ASCII
    if b0 < 0x80 && (size == 1 || (p.len() > 1 && p[1] < 0x80)) {
        return 1;
    }

    // Skip over first UTF-8 char, stopping at a NUL byte
    let len = utf_ptr2len_len(p, size);

    // Check for illegal byte and incomplete byte sequence
    if (len == 1 && b0 >= 0x80) || len > size {
        return 1;
    }

    // Check for composing characters
    let mut total_len = len;
    let mut prevlen = 0usize;
    let mut state: i32 = 0; // GRAPHEME_STATE_INIT

    while total_len < size {
        if total_len >= p.len() || p[total_len] == 0 {
            break;
        }

        // Quick check: ASCII is never composing
        if p[total_len] < 0x80 {
            break;
        }

        // Next character length should not go beyond size
        let remaining_size = size - total_len;
        let remaining_slice = if total_len < p.len() {
            &p[total_len..]
        } else {
            break;
        };
        let len_next_char = utf_ptr2len_len(remaining_slice, remaining_size);
        if len_next_char > remaining_size {
            break;
        }

        // Get the codepoints for comparison
        let prev_char = utf_ptr2char(&p[prevlen..]);
        let next_char = utf_ptr2char(&p[total_len..]);

        if !utf_composinglike(prev_char, next_char, &mut state) {
            break;
        }

        // Skip over composing char
        prevlen = total_len;
        total_len += len_next_char;
    }

    total_len
}

/// C-compatible wrapper for `utfc_ptr2len`.
///
/// Returns the number of bytes occupied by a UTF-8 character including
/// following composing characters.
///
/// # Safety
/// - `p` must be a valid pointer to a NUL-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn rs_utfc_ptr2len(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Find string length (with reasonable limit)
    let mut len = 0usize;
    while len < 1_000_000 {
        if *p.add(len) == 0 {
            break;
        }
        len += 1;
    }

    if len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(p as *const u8, len);
    utfc_ptr2len(slice) as c_int
}

/// C-compatible wrapper for `utfc_ptr2len_len`.
///
/// Returns the number of bytes occupied by a UTF-8 character including
/// following composing characters, with a size limit.
///
/// # Safety
/// - `p` must be a valid pointer with at least `size` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_utfc_ptr2len_len(p: *const c_char, size: c_int) -> c_int {
    if p.is_null() || size <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(p as *const u8, size as usize);
    utfc_ptr2len_len(slice, size as usize) as c_int
}

#[cfg(test)]
mod utfc_tests {
    use super::*;

    #[test]
    fn test_utfc_ptr2len_ascii() {
        // Single ASCII followed by another ASCII
        assert_eq!(utfc_ptr2len(b"ab"), 1);
        assert_eq!(utfc_ptr2len(b"a"), 1);
    }

    #[test]
    fn test_utfc_ptr2len_nul() {
        assert_eq!(utfc_ptr2len(b""), 0);
        assert_eq!(utfc_ptr2len(b"\0abc"), 0);
    }

    // Note: Tests involving actual composing characters would require
    // utf8proc to be linked, which happens in integration tests.
}

// Note: Tests for UTF-8 multibyte cell widths are complex because
// they depend on vim_isprintc, p_ambw, p_emoji, and cw_table globals.
// Those are tested via integration tests in neovim.

#[cfg(test)]
mod bom_tests {
    use super::*;

    #[test]
    fn test_remove_bom() {
        // No BOM - string unchanged
        let mut s = *b"Hello\0";
        unsafe { rs_remove_bom(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..5], b"Hello");

        // Single BOM at start
        let mut s = [0xEFu8, 0xBB, 0xBF, b'H', b'i', 0];
        unsafe { rs_remove_bom(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..2], b"Hi");
        assert_eq!(s[2], 0);

        // BOM in the middle
        let mut s = [b'A', 0xEF, 0xBB, 0xBF, b'B', 0, 0, 0];
        unsafe { rs_remove_bom(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..2], b"AB");
        assert_eq!(s[2], 0);

        // Multiple BOMs
        let mut s = [0xEFu8, 0xBB, 0xBF, 0xEF, 0xBB, 0xBF, b'X', 0, 0, 0, 0, 0, 0];
        unsafe { rs_remove_bom(s.as_mut_ptr().cast()) };
        assert_eq!(s[0], b'X');
        assert_eq!(s[1], 0);

        // Partial BOM (0xEF not followed by 0xBB 0xBF) - unchanged
        let mut s = [0xEFu8, 0xBC, 0x80, 0];
        unsafe { rs_remove_bom(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..3], &[0xEF, 0xBC, 0x80]);

        // Empty string
        let mut s = [0u8];
        unsafe { rs_remove_bom(s.as_mut_ptr().cast()) };
        assert_eq!(s[0], 0);

        // Null pointer - should not crash
        unsafe { rs_remove_bom(core::ptr::null_mut()) };
    }
}
