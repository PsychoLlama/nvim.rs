//! Character set utilities for Neovim.
//!
//! This crate provides FFI-compatible character classification and
//! string skipping utilities.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::ffi::c_char;
use std::ffi::c_int;

// Chartab flag masks (from charset.c)
const CT_CELL_MASK: u8 = 0x07; // mask: nr of display cells (1, 2 or 4)
const CT_PRINT_CHAR: u8 = 0x10; // flag: set for printable chars
const CT_ID_CHAR: u8 = 0x20; // flag: set for ID chars
const CT_FNAME_CHAR: u8 = 0x40; // flag: set for file name chars

// External reference to g_chartab from C
extern "C" {
    static g_chartab: [u8; 256];
}

// External reference to utfc_ptr2len from C
// This function returns the byte length of a UTF-8 character including composing characters
extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;
}

// ASCII character classification helpers (inline, pure functions)

/// Check if character is ASCII whitespace (' ' or '\t')
#[inline]
const fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if character is ASCII digit ('0'-'9')
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if character is binary digit ('0' or '1')
#[inline]
const fn ascii_isbdigit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Check if character is hex digit ('0'-'9', 'A'-'F', 'a'-'f')
#[inline]
const fn ascii_isxdigit(c: u8) -> bool {
    (c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f')
}

// ============================================================================
// Skip functions - Skip over characters matching certain criteria
// ============================================================================

/// Skip over ' ' and '\t' characters.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipwhite(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while ascii_iswhite(*ptr as u8) {
        ptr = ptr.add(1);
    }
    ptr
}

/// Skip over ' ' and '\t' characters up to `len` bytes.
///
/// # Safety
/// The pointer must be valid and accessible for at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_skipwhite_len(p: *const c_char, len: usize) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    let mut remaining = len;
    while remaining > 0 && ascii_iswhite(*ptr as u8) {
        ptr = ptr.add(1);
        remaining -= 1;
    }
    ptr
}

/// Skip over digit characters ('0'-'9').
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipdigits(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while ascii_isdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip over binary digit characters ('0' or '1').
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipbin(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while ascii_isbdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip over hex digit characters ('0'-'9', 'A'-'F', 'a'-'f').
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiphex(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while ascii_isxdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip to the next digit character or end of string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptodigit(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while *p != 0 && !ascii_isdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip to the next binary digit character or end of string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptobin(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while *p != 0 && !ascii_isbdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip to the next hex digit character or end of string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptohex(q: *const c_char) -> *const c_char {
    if q.is_null() {
        return q;
    }
    let mut p = q;
    while *p != 0 && !ascii_isxdigit(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Skip over text until ' ', '\t', or NUL.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptowhite(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while *ptr != 0 && *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
        ptr = ptr.add(1);
    }
    ptr
}

// Ctrl_V constant (ASCII 22)
const CTRL_V: u8 = 22;

/// Skip to whitespace, respecting escaped characters.
/// Like `skiptowhite()`, but also skips escaped chars (backslash or `Ctrl-V`).
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skiptowhite_esc(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while *ptr != 0 && *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
        // If we see a backslash or Ctrl-V, and the next char is not NUL, skip it
        if (*ptr == b'\\' as c_char || *ptr == CTRL_V as c_char) && *ptr.add(1) != 0 {
            ptr = ptr.add(1);
        }
        ptr = ptr.add(1);
    }
    ptr
}

/// Return the number of whitespace columns (bytes) at the start of a string.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_getwhitecols(p: *const c_char) -> isize {
    if p.is_null() {
        return 0;
    }
    let result = rs_skipwhite(p);
    result.offset_from(p)
}

/// Skip over text until '\n' (newline) or NUL.
///
/// Returns a pointer to the next '\n' or the NUL terminator.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_to_newline(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }
    let mut ptr = p;
    while *ptr != 0 && *ptr != b'\n' as c_char {
        ptr = ptr.add(1);
    }
    ptr
}

// ============================================================================
// Line classification functions
// ============================================================================

/// Check that the string is empty or only contains whitespace (blanks/tabs).
///
/// Returns true if the line is blank (empty, whitespace only, or ends at line terminator).
///
/// # Safety
/// The pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_isblankline(lbuf: *const c_char) -> bool {
    if lbuf.is_null() {
        return true;
    }
    let p = rs_skipwhite(lbuf);
    // NUL, CR, or LF all count as "blank line" (end of line or empty)
    *p == 0 || *p == b'\r' as c_char || *p == b'\n' as c_char
}

// ============================================================================
// Hex/Number conversion functions
// ============================================================================

/// Convert the lower 4 bits of a byte to its hex character.
///
/// Lower case letters are used to avoid the confusion of <F1> being 0xf1 or
/// function key 1.
///
/// Returns the hex character ('0'-'9', 'a'-'f').
#[no_mangle]
pub extern "C" fn rs_nr2hex(n: u32) -> u32 {
    let nibble = n & 0xf;
    if nibble <= 9 {
        nibble + u32::from(b'0')
    } else {
        nibble - 10 + u32::from(b'a')
    }
}

/// Return the value of a single hex character.
/// Only valid when the argument is '0'-'9', 'A'-'F', or 'a'-'f'.
///
/// Returns the numeric value (0-15) of the hex digit.
#[no_mangle]
pub extern "C" fn rs_hex2nr(c: c_int) -> c_int {
    let c = c as u8;
    if (b'a'..=b'f').contains(&c) {
        c_int::from(c - b'a' + 10)
    } else if (b'A'..=b'F').contains(&c) {
        c_int::from(c - b'A' + 10)
    } else {
        c_int::from(c.wrapping_sub(b'0'))
    }
}

/// Convert two hex characters to a byte.
///
/// Returns -1 if one of the characters is not hex.
///
/// # Safety
/// The pointer must be valid and point to at least 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_hexhex2nr(p: *const c_char) -> c_int {
    if p.is_null() {
        return -1;
    }
    let c0 = *p as u8;
    let c1 = *p.add(1) as u8;

    if !ascii_isxdigit(c0) || !ascii_isxdigit(c1) {
        return -1;
    }

    (rs_hex2nr(c_int::from(c0)) << 4) + rs_hex2nr(c_int::from(c1))
}

/// Convert a non-printable character to hex C string like "<FFFF>".
///
/// Formats the character as `<XX>` for values <= 0xFF,
/// `<XXXX>` for values <= 0xFFFF, or `<XXXXXX>` for larger values.
///
/// Returns the number of bytes written (excluding the NUL terminator).
///
/// # Safety
/// The buffer must be valid and have at least 9 bytes of space
/// (for the longest format: `<XXXXXX>\0`).
#[no_mangle]
pub unsafe extern "C" fn rs_transchar_hex(buf: *mut c_char, c: c_int) -> usize {
    if buf.is_null() {
        return 0;
    }

    let c = c as u32;
    let mut i = 0usize;

    *buf.add(i) = b'<' as c_char;
    i += 1;

    if c > 0xFF {
        if c > 0xFFFF {
            *buf.add(i) = rs_nr2hex(c >> 20) as c_char;
            i += 1;
            *buf.add(i) = rs_nr2hex(c >> 16) as c_char;
            i += 1;
        }
        *buf.add(i) = rs_nr2hex(c >> 12) as c_char;
        i += 1;
        *buf.add(i) = rs_nr2hex(c >> 8) as c_char;
        i += 1;
    }

    *buf.add(i) = rs_nr2hex(c >> 4) as c_char;
    i += 1;
    *buf.add(i) = rs_nr2hex(c) as c_char;
    i += 1;
    *buf.add(i) = b'>' as c_char;
    i += 1;
    *buf.add(i) = 0; // NUL terminator

    i
}

// ============================================================================
// Character classification functions (using g_chartab)
// ============================================================================

/// Check if `c` is a valid file-name character.
///
/// Assume characters above 0x100 are valid (multi-byte).
/// To be used for commands like "gf".
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_isfilec(c: c_int) -> c_int {
    // Multibyte characters (>= 0x100) are valid file name characters
    // Single-byte characters need the CT_FNAME_CHAR flag set in g_chartab
    c_int::from(c >= 0x100 || (c > 0 && (g_chartab[c as usize] & CT_FNAME_CHAR) != 0))
}

/// Check if "c" is a valid file-name character, including characters left
/// out of 'isfname' to make "gf" work, such as ',', ' ', '@', ':', etc.
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_is_fname_char(c: c_int) -> c_int {
    c_int::from(
        rs_vim_isfilec(c) != 0
            || c == c_int::from(b',')
            || c == c_int::from(b' ')
            || c == c_int::from(b'@')
            || c == c_int::from(b':'),
    )
}

/// Return number of display cells occupied by byte "b".
///
/// This assumes the byte is ASCII (< 0x80). For bytes >= 0x80, returns 0
/// since the actual cell count depends on further bytes in UTF-8.
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_byte2cells(b: c_int) -> c_int {
    if b >= 0x80 {
        0
    } else {
        c_int::from(g_chartab[b as usize] & CT_CELL_MASK)
    }
}

/// Check that "c" is a normal identifier character:
/// Letters and characters from the 'isident' option.
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_isIDc(c: c_int) -> c_int {
    c_int::from(c > 0 && c < 0x100 && (g_chartab[c as usize] & CT_ID_CHAR) != 0)
}

// Note: vim_iswordc and related functions are NOT migrated because they use
// curbuf global or buffer-specific chartabs.

// Key code constants for char2cells (from keycodes.h)
// Special keys are represented as negative values.
const fn is_special(c: i32) -> bool {
    c < 0
}

// KEY2TERMCAP0(x) = (-(x)) & 0xff
// For special keys (c < 0), get the low byte of the absolute value
const fn key2termcap0(c: i32) -> i32 {
    (-c) & 0xff
}

// K_SECOND: get second byte when translating special key code
// For special keys (c < 0), this is just KEY2TERMCAP0
// (K_SPECIAL and NUL checks are not possible when c < 0)
const fn k_second(c: i32) -> i32 {
    key2termcap0(c)
}

/// Return number of display cells occupied by character "c".
///
/// "c" can be a special key (negative number) in which case 3 or 4 is returned.
/// A TAB is counted as two cells: "^I" or four: "<09>".
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[must_use]
pub fn char2cells(c: i32) -> i32 {
    if is_special(c) {
        // Special key - recursively get cells for the "second byte" + 2
        return char2cells(k_second(c)) + 2;
    }

    if c >= 0x80 {
        // UTF-8: above 0x80 need to check the value
        return nvim_mbyte::utf_char2cells(c);
    }

    // ASCII: get cell count from chartab
    // SAFETY: caller must ensure g_chartab is initialized
    unsafe { i32::from(g_chartab[(c & 0xff) as usize] & CT_CELL_MASK) }
}

/// FFI wrapper for `char2cells`.
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[no_mangle]
pub extern "C" fn rs_char2cells(c: c_int) -> c_int {
    char2cells(c)
}

/// Check that "c" is a printable character.
///
/// For characters >= 0x100, uses `utf_printable` from nvim-mbyte.
/// For single-byte characters, checks `CT_PRINT_CHAR` flag in `g_chartab`.
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_isprintc(c: c_int) -> c_int {
    if c >= 0x100 {
        // Use utf_printable from nvim-mbyte crate for multibyte chars
        c_int::from(nvim_mbyte::utf_printable(c))
    } else {
        // Single-byte: check g_chartab
        c_int::from(c > 0 && (g_chartab[c as usize] & CT_PRINT_CHAR) != 0)
    }
}

/// Return number of display cells occupied by character at "*p".
/// A TAB is counted as two cells: "^I" or four: "<09>".
///
/// For UTF-8 (first byte >= 0x80), delegates to `utf_ptr2cells`.
/// For ASCII, looks up cell count in `g_chartab`.
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[must_use]
pub fn ptr2cells(p: &[u8]) -> i32 {
    if p.is_empty() {
        return 1;
    }

    let first = p[0];
    if first >= 0x80 {
        // UTF-8: need to look at more bytes
        return nvim_mbyte::utf_ptr2cells(p);
    }

    // ASCII: get cell count from chartab
    // SAFETY: caller must ensure g_chartab is initialized
    unsafe { i32::from(g_chartab[first as usize] & CT_CELL_MASK) }
}

/// FFI wrapper for `ptr2cells`.
///
/// # Safety
/// - The pointer must be valid and point to a null-terminated C string.
/// - The global `g_chartab` array must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_ptr2cells(p: *const c_char) -> c_int {
    if p.is_null() {
        return 1;
    }
    // Create a slice from the pointer - we need at least enough bytes
    // for the longest possible UTF-8 sequence (4 bytes)
    // SAFETY: caller guarantees p points to valid string
    let slice = std::slice::from_raw_parts(p.cast::<u8>(), 6);
    ptr2cells(slice)
}

// MAXCOL constant from Vim - very large column value
const MAXCOL: c_int = 0x7fff_ffff;

/// Return the number of character cells string "s" will take on the screen,
/// counting TABs as two cells: "^I".
///
/// 's' must be non-null.
///
/// # Safety
/// - The pointer must be valid and point to a null-terminated C string.
/// - The global `g_chartab` array must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_strsize(s: *const c_char) -> c_int {
    rs_vim_strnsize(s, MAXCOL)
}

/// Return the number of character cells string "s[len]" will take on the
/// screen, counting TABs as two cells: "^I".
///
/// 's' must be non-null.
///
/// # Safety
/// - The pointer must be valid and point to a null-terminated C string.
/// - The global `g_chartab` array must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_strnsize(s: *const c_char, len: c_int) -> c_int {
    if s.is_null() {
        return 0;
    }

    let mut p = s;
    let mut size: c_int = 0;
    let mut remaining = len;

    while *p != 0 && remaining > 0 {
        remaining -= 1;

        // Get the byte length of this character (including composing chars)
        let char_len = utfc_ptr2len(p);

        // Get the display cell count for this character
        let slice = std::slice::from_raw_parts(p.cast::<u8>(), 6);
        size += ptr2cells(slice);

        // Advance pointer by character length
        p = p.add(char_len as usize);

        // Adjust remaining length (subtract l-1 because we already subtracted 1)
        remaining -= char_len - 1;
    }

    size
}

// ============================================================================
// String reversal functions
// ============================================================================

/// Reverse an ASCII string in-place.
///
/// Reverses the characters between `str` and `end` (exclusive).
/// If `end` is null, reverses from `str` to the end of the string (NUL terminator).
///
/// This is used for right-to-left text handling.
///
/// # Safety
/// - `str` must be a valid pointer to a mutable C string.
/// - If `end` is not null, `end` must point within the same string as `str`, after `str`.
#[no_mangle]
pub unsafe extern "C" fn rs_rl_mirror_ascii(str: *mut c_char, end: *const c_char) {
    if str.is_null() {
        return;
    }

    // Calculate end pointer: if end is null, find the NUL terminator
    let mut p1 = str;
    #[allow(clippy::ptr_cast_constness)]
    let mut p2 = if end.is_null() {
        // Find string length and point to last char
        let mut p = str;
        while *p != 0 {
            p = p.add(1);
        }
        p.sub(1)
    } else {
        // end points one past the last char we want to include
        (end as *mut c_char).sub(1)
    };

    // Swap characters from both ends moving toward the middle
    while p1 < p2 {
        core::ptr::swap(p1, p2);
        p1 = p1.add(1);
        p2 = p2.sub(1);
    }
}

/// Halve the number of backslashes in a file name argument.
///
/// Modifies the string in-place, removing backslashes that should be removed
/// according to rem_backslash rules.
///
/// # Safety
/// - `p` must be a valid pointer to a mutable null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_backslash_halve(p: *mut c_char) {
    if p.is_null() {
        return;
    }

    // Find the first position where we should remove a backslash
    let mut src = p;
    while *src != 0 && !rs_rem_backslash(src) {
        src = src.add(1);
    }

    // If no backslash to remove, we're done
    if *src == 0 {
        return;
    }

    // Start copying from the character after the backslash
    let mut dst = src;

    // Copy the character after the backslash (skip the backslash itself)
    *dst = *src.add(1);
    dst = dst.add(1);
    src = src.add(2);

    // Continue through the rest of the string
    while *src != 0 {
        if rs_rem_backslash(src) {
            // Skip backslash, copy next char
            *dst = *src.add(1);
            dst = dst.add(1);
            src = src.add(2);
        } else {
            // Copy char as-is
            *dst = *src;
            dst = dst.add(1);
            src = src.add(1);
        }
    }

    // Null-terminate
    *dst = 0;
}

/// Check if we should remove a backslash from a file name argument.
///
/// On Unix: always remove backslash before non-NUL characters.
/// On Windows: remove backslash before space or before ASCII non-wildcard
/// non-filename characters. For multi-byte characters (>= 0x80), assume
/// all are valid file name characters.
///
/// # Safety
/// - `str` must be a valid pointer to a null-terminated C string.
/// - On Windows, this accesses the global `g_chartab` array.
#[no_mangle]
pub unsafe extern "C" fn rs_rem_backslash(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let c0 = *s as u8;
    let c1 = *s.add(1) as u8;

    if c0 != b'\\' {
        return false;
    }

    #[cfg(windows)]
    {
        // On Windows: backslash is path separator, so only remove in specific cases
        // BACKSLASH_IN_FILENAME is defined on Windows
        c1 < 0x80
            && (c1 == b' '
                || (c1 != 0 && c1 != b'*' && c1 != b'?' && rs_vim_isfilec(c1 as c_int) == 0))
    }

    #[cfg(not(windows))]
    {
        // On Unix: backslash is escape character, remove before any non-NUL char
        c1 != 0
    }
}

// ============================================================================
// Word character detection using buffer chartab
// ============================================================================

/// Check a bit in the buffer chartab (uint64_t[4] bitmap).
///
/// This replicates the C macro:
/// `GET_CHARTAB_TAB(chartab, c) ((chartab)[(unsigned)(c) >> 6] & (1ull << ((c) & 0x3f)))`
#[inline]
fn get_chartab_tab(chartab: &[u64; 4], c: u8) -> bool {
    let idx = (c >> 6) as usize;
    let bit = 1u64 << (c & 0x3f);
    (chartab[idx] & bit) != 0
}

/// Check if a character is a word character using buffer-specific chartab.
///
/// For Latin1 characters (< 0x100), uses the chartab bitmap directly.
/// For multibyte characters (>= 0x100), uses utf_class_tab to determine
/// if the character class is >= 2 (word character).
///
/// The `chartab` is a buffer-specific `uint64_t[4]` array set via 'iskeyword'.
#[inline]
pub fn vim_iswordc_tab(c: i32, chartab: &[u64; 4]) -> bool {
    if c >= 0x100 {
        // Multibyte: use utf_class_tab, word char if class >= 2
        nvim_mbyte::utf_class_tab_impl(c, chartab) >= 2
    } else {
        // Latin1: check chartab directly
        c > 0 && get_chartab_tab(chartab, c as u8)
    }
}

/// FFI wrapper for `vim_iswordc_tab`.
///
/// # Safety
///
/// - `chartab` must be a valid pointer to a `[u64; 4]` array
#[no_mangle]
pub unsafe extern "C" fn rs_vim_iswordc_tab(c: c_int, chartab: *const u64) -> c_int {
    if chartab.is_null() {
        return 0;
    }

    let chartab_arr: &[u64; 4] = &*(chartab as *const [u64; 4]);
    c_int::from(vim_iswordc_tab(c, chartab_arr))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_skipwhite() {
        unsafe {
            // Test skipping spaces
            let s = CString::new("   hello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test skipping tabs
            let s = CString::new("\t\thello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test mixed spaces and tabs
            let s = CString::new(" \t \thello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test no whitespace
            let s = CString::new("hello").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, b'h' as c_char);

            // Test empty string
            let s = CString::new("").unwrap();
            let result = rs_skipwhite(s.as_ptr());
            assert_eq!(*result, 0);

            // Test null pointer
            let result = rs_skipwhite(std::ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_skipwhite_len() {
        unsafe {
            // Test with len limit
            let s = CString::new("     hello").unwrap();
            let result = rs_skipwhite_len(s.as_ptr(), 3);
            // Should stop after 3 spaces
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 3);

            // Test with len > whitespace count
            let s = CString::new("  hello").unwrap();
            let result = rs_skipwhite_len(s.as_ptr(), 10);
            assert_eq!(*result, b'h' as c_char);

            // Test with len = 0
            let s = CString::new("  hello").unwrap();
            let result = rs_skipwhite_len(s.as_ptr(), 0);
            assert_eq!(*result, b' ' as c_char);
        }
    }

    #[test]
    fn test_skipdigits() {
        unsafe {
            let s = CString::new("12345abc").unwrap();
            let result = rs_skipdigits(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);

            let s = CString::new("abc123").unwrap();
            let result = rs_skipdigits(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);

            let s = CString::new("12345").unwrap();
            let result = rs_skipdigits(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skipbin() {
        unsafe {
            let s = CString::new("01010abc").unwrap();
            let result = rs_skipbin(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);

            let s = CString::new("01012345").unwrap();
            let result = rs_skipbin(s.as_ptr());
            assert_eq!(*result, b'2' as c_char);

            let s = CString::new("abc").unwrap();
            let result = rs_skipbin(s.as_ptr());
            assert_eq!(*result, b'a' as c_char);
        }
    }

    #[test]
    fn test_skiphex() {
        unsafe {
            let s = CString::new("1a2b3cGHI").unwrap();
            let result = rs_skiphex(s.as_ptr());
            assert_eq!(*result, b'G' as c_char);

            let s = CString::new("ABCDEF123xyz").unwrap();
            let result = rs_skiphex(s.as_ptr());
            assert_eq!(*result, b'x' as c_char);

            let s = CString::new("xyz").unwrap();
            let result = rs_skiphex(s.as_ptr());
            assert_eq!(*result, b'x' as c_char);
        }
    }

    #[test]
    fn test_skiptodigit() {
        unsafe {
            let s = CString::new("abc123").unwrap();
            let result = rs_skiptodigit(s.as_ptr());
            assert_eq!(*result, b'1' as c_char);

            let s = CString::new("123").unwrap();
            let result = rs_skiptodigit(s.as_ptr());
            assert_eq!(*result, b'1' as c_char);

            let s = CString::new("abc").unwrap();
            let result = rs_skiptodigit(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skiptobin() {
        unsafe {
            let s = CString::new("abc0101").unwrap();
            let result = rs_skiptobin(s.as_ptr());
            assert_eq!(*result, b'0' as c_char);

            let s = CString::new("0101").unwrap();
            let result = rs_skiptobin(s.as_ptr());
            assert_eq!(*result, b'0' as c_char);

            let s = CString::new("abc").unwrap();
            let result = rs_skiptobin(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skiptohex() {
        unsafe {
            let s = CString::new("xyz1aF").unwrap();
            let result = rs_skiptohex(s.as_ptr());
            assert_eq!(*result, b'1' as c_char);

            let s = CString::new("AbCd").unwrap();
            let result = rs_skiptohex(s.as_ptr());
            assert_eq!(*result, b'A' as c_char);

            let s = CString::new("ghi").unwrap();
            let result = rs_skiptohex(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_skiptowhite() {
        unsafe {
            let s = CString::new("hello world").unwrap();
            let result = rs_skiptowhite(s.as_ptr());
            assert_eq!(*result, b' ' as c_char);

            let s = CString::new("hello\tworld").unwrap();
            let result = rs_skiptowhite(s.as_ptr());
            assert_eq!(*result, b'\t' as c_char);

            let s = CString::new("hello").unwrap();
            let result = rs_skiptowhite(s.as_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_nr2hex() {
        // Test 0-9 -> '0'-'9'
        assert_eq!(rs_nr2hex(0), u32::from(b'0'));
        assert_eq!(rs_nr2hex(5), u32::from(b'5'));
        assert_eq!(rs_nr2hex(9), u32::from(b'9'));

        // Test 10-15 -> 'a'-'f' (lowercase)
        assert_eq!(rs_nr2hex(10), u32::from(b'a'));
        assert_eq!(rs_nr2hex(11), u32::from(b'b'));
        assert_eq!(rs_nr2hex(15), u32::from(b'f'));

        // Test that only lower 4 bits are used
        assert_eq!(rs_nr2hex(0x10), u32::from(b'0')); // 16 -> 0
        assert_eq!(rs_nr2hex(0x1f), u32::from(b'f')); // 31 -> 15 -> 'f'
        assert_eq!(rs_nr2hex(0xff), u32::from(b'f')); // 255 -> 15 -> 'f'
    }

    #[test]
    fn test_hex2nr() {
        // Test digits
        assert_eq!(rs_hex2nr(c_int::from(b'0')), 0);
        assert_eq!(rs_hex2nr(c_int::from(b'5')), 5);
        assert_eq!(rs_hex2nr(c_int::from(b'9')), 9);

        // Test uppercase
        assert_eq!(rs_hex2nr(c_int::from(b'A')), 10);
        assert_eq!(rs_hex2nr(c_int::from(b'F')), 15);

        // Test lowercase
        assert_eq!(rs_hex2nr(c_int::from(b'a')), 10);
        assert_eq!(rs_hex2nr(c_int::from(b'f')), 15);
    }

    #[test]
    fn test_hexhex2nr() {
        unsafe {
            let s = CString::new("FF").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 255);

            let s = CString::new("00").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 0);

            let s = CString::new("1a").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 26);

            let s = CString::new("a1").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), 161);

            // Test invalid hex
            let s = CString::new("GG").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), -1);

            let s = CString::new("1G").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), -1);

            let s = CString::new("G1").unwrap();
            assert_eq!(rs_hexhex2nr(s.as_ptr()), -1);
        }
    }

    #[test]
    fn test_skiptowhite_esc() {
        unsafe {
            // Normal case - skip to space
            let s = CString::new("hello world").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, b' ' as c_char);

            // Escaped space with backslash - should skip over it and continue
            // "hello\ world" - the backslash escapes the space, so we continue to end
            let s = CString::new("hello\\ world").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 12); // Scans entire string "hello\ world" (12 bytes)

            // With actual space after escaped char
            let s = CString::new("hello\\x world").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            // backslash escapes 'x', then continues to space
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 7); // "hello\x" then hits space

            // Escaped space with Ctrl-V (ASCII 22)
            // "hi<Ctrl-V> x" - Ctrl-V escapes space, then continues to scan "x" to NUL
            let s: [u8; 6] = [b'h', b'i', 22, b' ', b'x', 0]; // "hi<Ctrl-V> x"
            let result = rs_skiptowhite_esc(s.as_ptr().cast::<c_char>());
            let offset = result.offset_from(s.as_ptr().cast::<c_char>());
            assert_eq!(offset, 5); // Scans to NUL at position 5

            // No whitespace
            let s = CString::new("hello").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, 0);

            // Backslash at end (next char is NUL)
            let s = CString::new("hello\\").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, 0);

            // Tab
            let s = CString::new("hello\tworld").unwrap();
            let result = rs_skiptowhite_esc(s.as_ptr());
            assert_eq!(*result, b'\t' as c_char);
        }
    }

    #[test]
    fn test_getwhitecols() {
        unsafe {
            // Spaces at start
            let s = CString::new("   hello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 3);

            // Tabs at start
            let s = CString::new("\t\thello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 2);

            // Mixed spaces and tabs
            let s = CString::new(" \t \thello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 4);

            // No whitespace
            let s = CString::new("hello").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 0);

            // Empty string
            let s = CString::new("").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 0);

            // All whitespace
            let s = CString::new("   ").unwrap();
            assert_eq!(rs_getwhitecols(s.as_ptr()), 3);

            // Null pointer
            assert_eq!(rs_getwhitecols(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_skip_to_newline() {
        unsafe {
            // Skip to newline
            let s = CString::new("hello\nworld").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, b'\n' as c_char);
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 5);

            // No newline - skip to NUL
            let s = CString::new("hello world").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, 0);

            // Newline at start
            let s = CString::new("\nhello").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, b'\n' as c_char);
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 0);

            // Empty string
            let s = CString::new("").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, 0);

            // Multiple newlines - stops at first
            let s = CString::new("line1\nline2\nline3").unwrap();
            let result = rs_skip_to_newline(s.as_ptr());
            assert_eq!(*result, b'\n' as c_char);
            let offset = result.offset_from(s.as_ptr());
            assert_eq!(offset, 5);

            // Null pointer
            let result = rs_skip_to_newline(std::ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_vim_isblankline() {
        unsafe {
            // Empty string - blank
            let s = CString::new("").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Only spaces - blank
            let s = CString::new("   ").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Only tabs - blank
            let s = CString::new("\t\t").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Mixed whitespace - blank
            let s = CString::new(" \t \t ").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Contains text - not blank
            let s = CString::new("hello").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Whitespace before text - not blank
            let s = CString::new("   hello").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Text before whitespace - not blank
            let s = CString::new("hello   ").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Single character - not blank
            let s = CString::new("x").unwrap();
            assert!(!rs_vim_isblankline(s.as_ptr()));

            // Line ending with \n (newline) - blank
            let s = CString::new("\n").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Whitespace followed by \n - blank
            let s = CString::new("   \n").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Line ending with \r (carriage return) - blank
            let s = CString::new("\r").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Whitespace followed by \r - blank
            let s = CString::new("   \r").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Whitespace followed by \r\n (CRLF) - blank
            let s = CString::new("   \r\n").unwrap();
            assert!(rs_vim_isblankline(s.as_ptr()));

            // Null pointer - blank (edge case)
            assert!(rs_vim_isblankline(std::ptr::null()));
        }
    }

    #[test]
    fn test_transchar_hex() {
        unsafe {
            let mut buf = [0i8; 16];

            // Single byte value (0x00 - 0xFF) -> "<XX>"
            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x00);
            assert_eq!(len, 4);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'0' as i8, b'0' as i8, b'>' as i8, 0]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x1A);
            assert_eq!(len, 4);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'1' as i8, b'a' as i8, b'>' as i8, 0]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0xFF);
            assert_eq!(len, 4);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'f' as i8, b'f' as i8, b'>' as i8, 0]
            );

            // Two byte value (0x100 - 0xFFFF) -> "<XXXX>"
            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x100);
            assert_eq!(len, 6);
            assert_eq!(
                &buf[..7],
                [b'<' as i8, b'0' as i8, b'1' as i8, b'0' as i8, b'0' as i8, b'>' as i8, 0]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0xABCD);
            assert_eq!(len, 6);
            assert_eq!(
                &buf[..7],
                [b'<' as i8, b'a' as i8, b'b' as i8, b'c' as i8, b'd' as i8, b'>' as i8, 0]
            );

            // Three byte value (> 0xFFFF) -> "<XXXXXX>"
            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x10000);
            assert_eq!(len, 8);
            assert_eq!(
                &buf[..9],
                [
                    b'<' as i8, b'0' as i8, b'1' as i8, b'0' as i8, b'0' as i8, b'0' as i8,
                    b'0' as i8, b'>' as i8, 0
                ]
            );

            let len = rs_transchar_hex(buf.as_mut_ptr(), 0x0012_ABCD);
            assert_eq!(len, 8);
            assert_eq!(
                &buf[..9],
                [
                    b'<' as i8, b'1' as i8, b'2' as i8, b'a' as i8, b'b' as i8, b'c' as i8,
                    b'd' as i8, b'>' as i8, 0
                ]
            );

            // Null buffer returns 0
            assert_eq!(rs_transchar_hex(std::ptr::null_mut(), 0x42), 0);
        }
    }

    #[test]
    fn test_rl_mirror_ascii() {
        unsafe {
            // Test reversing "hello" -> "olleh"
            let mut s = *b"hello\0";
            rs_rl_mirror_ascii(s.as_mut_ptr().cast::<c_char>(), std::ptr::null());
            assert_eq!(&s[..5], b"olleh");

            // Test with explicit end pointer
            let mut s = *b"hello world\0";
            let end = s.as_ptr().add(5); // reverse only "hello"
            rs_rl_mirror_ascii(s.as_mut_ptr().cast::<c_char>(), end.cast::<c_char>());
            assert_eq!(&s[..11], b"olleh world");

            // Test empty string
            let mut s = *b"\0";
            rs_rl_mirror_ascii(s.as_mut_ptr().cast::<c_char>(), std::ptr::null());
            assert_eq!(s[0], 0);

            // Test single character
            let mut s = *b"a\0";
            rs_rl_mirror_ascii(s.as_mut_ptr().cast::<c_char>(), std::ptr::null());
            assert_eq!(&s[..1], b"a");

            // Test two characters
            let mut s = *b"ab\0";
            rs_rl_mirror_ascii(s.as_mut_ptr().cast::<c_char>(), std::ptr::null());
            assert_eq!(&s[..2], b"ba");

            // Test palindrome
            let mut s = *b"racecar\0";
            rs_rl_mirror_ascii(s.as_mut_ptr().cast::<c_char>(), std::ptr::null());
            assert_eq!(&s[..7], b"racecar");

            // Test null pointer - should not crash
            rs_rl_mirror_ascii(std::ptr::null_mut(), std::ptr::null());
        }
    }

    #[test]
    fn test_rem_backslash() {
        unsafe {
            // Backslash followed by a regular character - should remove
            let s = CString::new("\\a").unwrap();
            assert!(rs_rem_backslash(s.as_ptr()));

            // Backslash followed by space - should remove
            let s = CString::new("\\ ").unwrap();
            assert!(rs_rem_backslash(s.as_ptr()));

            // Backslash at end of string (followed by NUL) - should NOT remove
            let s = CString::new("\\").unwrap();
            assert!(!rs_rem_backslash(s.as_ptr()));

            // No backslash at start - should NOT remove
            let s = CString::new("abc").unwrap();
            assert!(!rs_rem_backslash(s.as_ptr()));

            // Null pointer - should NOT crash and return false
            assert!(!rs_rem_backslash(std::ptr::null()));

            // Backslash followed by another backslash
            let s = CString::new("\\\\").unwrap();
            assert!(rs_rem_backslash(s.as_ptr()));
        }
    }

    #[test]
    fn test_backslash_halve() {
        unsafe {
            // Single backslash before character - should be removed
            let mut s = *b"\\a\0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(&s[..2], b"a\0");

            // Multiple backslashes - each should be halved
            let mut s = *b"\\a\\b\\c\0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(&s[..4], b"abc\0");

            // Backslash before space
            let mut s = *b"\\ \0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(&s[..2], b" \0");

            // No backslash - string unchanged
            let mut s = *b"abc\0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(&s[..4], b"abc\0");

            // Double backslash - halved to single backslash
            let mut s = *b"\\\\\0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(&s[..2], b"\\\0");

            // Empty string
            let mut s = *b"\0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(s[0], 0);

            // Backslash at end (followed by NUL) - NOT removed
            let mut s = *b"\\\0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(&s[..2], b"\\\0");

            // Mixed content
            let mut s = *b"foo\\bar\\baz\0";
            rs_backslash_halve(s.as_mut_ptr().cast::<c_char>());
            assert_eq!(&s[..10], b"foobarbaz\0");

            // Null pointer - should not crash
            rs_backslash_halve(std::ptr::null_mut());
        }
    }
}
