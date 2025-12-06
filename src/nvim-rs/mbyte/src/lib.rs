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

use std::ffi::{c_char, c_int};

use nvim_utf8proc::{casefold, get_property, grapheme_break};

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
    if let Some(prop) = get_property(c) {
        prop.is_composing_legacy()
    } else {
        false
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
