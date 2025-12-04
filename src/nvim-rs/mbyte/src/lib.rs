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

// Character printability

/// Sorted list of non-printable character ranges for utf_printable.
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
        let mid = (lo + hi) / 2;
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
/// FFI wrapper for utf_printable.
#[no_mangle]
pub extern "C" fn rs_utf_printable(c: c_int) -> c_int {
    c_int::from(utf_printable(c))
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
}
