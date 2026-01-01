//! Character set utilities for Neovim.
//!
//! This crate provides FFI-compatible character classification and
//! string skipping utilities.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::map_entry)]
#![allow(clippy::cast_lossless)]

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

// Constants for line ending formats (used by transchar_nonprint)
#[allow(dead_code)]
const EOL_UNIX: c_int = 0; // NL
#[allow(dead_code)]
const EOL_DOS: c_int = 1; // CR NL
const EOL_MAC: c_int = 2; // CR

// ASCII constants
const NL: u8 = 0x0A; // newline '\n'
const CAR: u8 = 0x0D; // carriage return '\r'
const NUL: u8 = 0x00; // null byte

/// Convert non-printable characters to 2..4 printable ones
///
/// Does not work for multi-byte characters, c must be <= 255.
///
/// # Arguments
/// * `charbuf` - Buffer to store result in, must be able to hold at least 5 bytes
/// * `c` - Character to convert. NL is assumed to be NUL according to `:h NL-used-for-NUL`.
/// * `use_uhex` - Whether to use hex format (dy_flags & kOptDyFlagUhex)
/// * `fileformat` - File format (EOL_UNIX=0, EOL_DOS=1, EOL_MAC=2), or -1 if buf is NULL
///
/// # Safety
/// The buffer must be valid and have at least 5 bytes of space.
#[no_mangle]
pub unsafe extern "C" fn rs_transchar_nonprint(
    charbuf: *mut c_char,
    mut c: c_int,
    use_uhex: bool,
    fileformat: c_int,
) {
    if charbuf.is_null() {
        return;
    }

    // Handle newline/carriage return conversions based on context
    if c == NL as c_int {
        // we use newline in place of a NUL
        c = NUL as c_int;
    } else if c == CAR as c_int && fileformat == EOL_MAC {
        // we use CR in place of NL in MAC format
        c = NL as c_int;
    }

    debug_assert!(c <= 0xff);

    if use_uhex || c > 0x7f {
        // 'display' has "uhex" or high-bit character
        rs_transchar_hex(charbuf, c);
    } else {
        // 0x00 - 0x1f and 0x7f
        *charbuf = b'^' as c_char;
        // DEL (0x7f) displayed as ^?, other control chars as ^@ through ^_
        *charbuf.add(1) = (c ^ 0x40) as c_char;
        *charbuf.add(2) = 0; // NUL terminator
    }
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
#[inline]
pub unsafe fn byte2cells(b: u8) -> c_int {
    if b >= 0x80 {
        0
    } else {
        c_int::from(g_chartab[b as usize] & CT_CELL_MASK)
    }
}

/// C-compatible wrapper for byte2cells.
///
/// # Safety
/// This function accesses the global `g_chartab` array which must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_byte2cells(b: c_int) -> c_int {
    byte2cells(b as u8)
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

// ============================================================================
// Word character detection using buffer (via C accessors)
// ============================================================================

// Opaque buffer handle type - matches BufHandle in buffer crate
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BufHandle(*mut std::ffi::c_void);

// C accessor functions for buffer fields
extern "C" {
    /// Get the current buffer (`curbuf` global).
    fn nvim_get_curbuf() -> BufHandle;

    /// Get the `b_chartab` field from a buffer.
    fn nvim_buf_get_chartab(buf: BufHandle) -> *const u64;
}

/// Check if a character is a word character using buffer-specific chartab.
///
/// This is the Rust equivalent of `vim_iswordc_buf` in charset.c.
/// Uses the buffer's `b_chartab` (set via 'iskeyword' option).
///
/// # Safety
///
/// - `buf` must be a valid buffer handle (non-null, valid `buf_T*`).
#[inline]
fn vim_iswordc_buf_impl(c: c_int, buf: BufHandle) -> bool {
    if buf.0.is_null() {
        return false;
    }

    // SAFETY: buf is a valid buffer handle, nvim_buf_get_chartab returns valid pointer
    let chartab_ptr = unsafe { nvim_buf_get_chartab(buf) };
    if chartab_ptr.is_null() {
        return false;
    }

    // SAFETY: chartab_ptr is a valid pointer to [u64; 4]
    let chartab: &[u64; 4] = unsafe { &*(chartab_ptr as *const [u64; 4]) };
    vim_iswordc_tab(c, chartab)
}

/// FFI wrapper for `vim_iswordc_buf`.
///
/// # Safety
///
/// - `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_iswordc_buf(c: c_int, buf: *mut std::ffi::c_void) -> c_int {
    c_int::from(vim_iswordc_buf_impl(c, BufHandle(buf)))
}

/// Check if a character is a word character for the current buffer.
///
/// This is the Rust equivalent of `vim_iswordc` in charset.c.
/// Uses the current buffer's (`curbuf`) `b_chartab`.
#[inline]
fn vim_iswordc_impl(c: c_int) -> bool {
    // SAFETY: nvim_get_curbuf returns the current buffer (may be null during startup)
    let buf = unsafe { nvim_get_curbuf() };
    vim_iswordc_buf_impl(c, buf)
}

/// FFI wrapper for `vim_iswordc`.
#[no_mangle]
pub extern "C" fn rs_vim_iswordc(c: c_int) -> c_int {
    c_int::from(vim_iswordc_impl(c))
}

// ============================================================================
// Word character detection using pointer (multi-byte aware)
// ============================================================================

/// Check if a pointer points to a word character for a specific buffer.
///
/// This is the Rust equivalent of `vim_iswordp_buf` in charset.c.
/// Gets the character from the pointer (handling multi-byte) and checks
/// if it's a word character using the buffer's `b_chartab`.
#[inline]
fn vim_iswordp_buf_impl(p: &[u8], buf: BufHandle) -> bool {
    if p.is_empty() || buf.0.is_null() {
        return false;
    }

    // Get the first byte
    let first_byte = p[0];

    // Determine if it's a multi-byte character (first byte >= 0x80)
    let c = if first_byte >= 0x80 {
        // Multi-byte: use utf_ptr2char to get the full codepoint
        nvim_mbyte::utf_ptr2char(p)
    } else {
        // Single byte: just use the byte value
        i32::from(first_byte)
    };

    vim_iswordc_buf_impl(c, buf)
}

/// FFI wrapper for `vim_iswordp_buf`.
///
/// # Safety
///
/// - `p` must be a valid pointer to a C string.
/// - `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_iswordp_buf(p: *const c_char, buf: *mut std::ffi::c_void) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Create a slice from the pointer - need at least 4 bytes for UTF-8
    let slice = std::slice::from_raw_parts(p.cast::<u8>(), 6);
    c_int::from(vim_iswordp_buf_impl(slice, BufHandle(buf)))
}

/// Check if a pointer points to a word character for the current buffer.
///
/// This is the Rust equivalent of `vim_iswordp` in charset.c.
/// Uses the current buffer's (`curbuf`) `b_chartab`.
#[inline]
fn vim_iswordp_impl(p: &[u8]) -> bool {
    // SAFETY: nvim_get_curbuf returns the current buffer (may be null during startup)
    let buf = unsafe { nvim_get_curbuf() };
    vim_iswordp_buf_impl(p, buf)
}

/// FFI wrapper for `vim_iswordp`.
///
/// # Safety
///
/// - `p` must be a valid pointer to a C string.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_iswordp(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Create a slice from the pointer - need at least 6 bytes for safety
    let slice = std::slice::from_raw_parts(p.cast::<u8>(), 6);
    c_int::from(vim_iswordp_impl(slice))
}

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
// String translation length calculation
// ============================================================================

const TAB: u8 = b'\t';

/// Calculate the length of translated hex representation for a character.
///
/// Returns the number of bytes needed for `<XX>`, `<XXXX>`, or `<XXXXXX>` format.
#[inline]
fn transchar_hex_len(c: i32) -> usize {
    let c = c as u32;
    if c > 0xFFFF {
        8 // <XXXXXX>
    } else if c > 0xFF {
        6 // <XXXX>
    } else {
        4 // <XX>
    }
}

/// Calculate the length of a string capable of holding s with all specials replaced.
///
/// Assumes replacing special characters with printable ones just like
/// strtrans() does.
///
/// # Arguments
/// * `s` - String to check (as byte slice)
/// * `untab` - If false, TAB is counted as 1 cell; if true, as printable representation
///
/// # Returns
/// Number of bytes needed to hold a translation of `s`, NUL byte not included.
pub fn transstr_len(s: &[u8], untab: bool) -> usize {
    let mut len = 0usize;
    let mut pos = 0usize;

    while pos < s.len() && s[pos] != 0 {
        // Get length including composing characters
        let l = nvim_mbyte::utfc_ptr2len(&s[pos..]);

        if l > 1 {
            // Multi-byte character
            let c = nvim_mbyte::utf_ptr2char(&s[pos..]);
            if nvim_mbyte::utf_printable(c) {
                // Printable character, keep as-is
                len += l;
            } else {
                // Non-printable: convert each codepoint to hex
                let mut off = 0;
                while off < l {
                    let cp = nvim_mbyte::utf_ptr2char(&s[pos + off..]);
                    len += transchar_hex_len(cp);
                    let cp_len = nvim_mbyte::utf_ptr2len(&s[pos + off..]);
                    off += cp_len;
                }
            }
            pos += l;
        } else if s[pos] == TAB && !untab {
            // TAB when not untabbing: count as 1
            len += 1;
            pos += 1;
        } else {
            // Single byte character
            let b2c_l = unsafe { rs_byte2cells(s[pos] as c_int) };
            // Illegal byte sequence may occupy up to 4 characters
            len += if b2c_l > 0 { b2c_l as usize } else { 4 };
            pos += 1;
        }
    }

    len
}

/// FFI wrapper for `transstr_len`.
///
/// Find length of a string capable of holding s with all specials replaced.
///
/// # Safety
/// - `s` must be a valid pointer to a null-terminated string
#[no_mangle]
pub unsafe extern "C" fn rs_transstr_len(s: *const c_char, untab: bool) -> usize {
    if s.is_null() {
        return 0;
    }

    // Find string length
    let mut len = 0usize;
    while len < 1_000_000 {
        if *s.add(len) == 0 {
            break;
        }
        len += 1;
    }

    if len == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(s as *const u8, len);
    transstr_len(slice, untab)
}

// ============================================================================
// Number parsing functions
// ============================================================================

use std::ffi::c_long;

/// Try to parse a decimal number from a string.
///
/// Updates the pointer to point past the parsed number.
/// Returns true (1) on success, false (0) on overflow.
///
/// Note: If no valid digits are found, returns 0 in `nr` without advancing
/// the pointer, but still returns true (success) - matching strtoimax behavior.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a C string.
/// - `nr` must be a valid pointer to an isize value.
#[no_mangle]
pub unsafe extern "C" fn rs_try_getdigits(pp: *mut *mut c_char, nr: *mut isize) -> c_int {
    if pp.is_null() || (*pp).is_null() || nr.is_null() {
        *nr = 0;
        return 1; // strtoimax returns success even with null input
    }

    let s = *pp;

    // Skip leading whitespace (strtoimax does this)
    let mut p = s as *const u8;
    while *p == b' ' || *p == b'\t' {
        p = p.add(1);
    }

    // Check for sign
    let negative = *p == b'-';
    if negative || *p == b'+' {
        p = p.add(1);
    }

    // If no digit follows, strtoimax returns 0 without advancing past sign
    // Actually, strtoimax doesn't advance past the sign if no digit follows
    if !(*p >= b'0' && *p <= b'9') {
        // No valid conversion, return 0, don't advance pointer
        *nr = 0;
        return 1; // success (no overflow, just no conversion)
    }

    // Parse digits
    let mut result: isize = 0;
    let mut overflow = false;

    while *p >= b'0' && *p <= b'9' {
        let digit = (*p - b'0') as isize;

        // Check for overflow before multiplying/adding
        if !overflow {
            if negative {
                // Check: result * 10 - digit < MIN  =>  result < (MIN + digit) / 10
                if result < (isize::MIN + digit) / 10 {
                    overflow = true;
                }
            } else {
                // Check: result * 10 + digit > MAX  =>  result > (MAX - digit) / 10
                if result > (isize::MAX - digit) / 10 {
                    overflow = true;
                }
            }
        }

        if !overflow {
            result *= 10;
            if negative {
                result -= digit;
            } else {
                result += digit;
            }
        }

        p = p.add(1);
    }

    if overflow {
        *nr = if negative { isize::MIN } else { isize::MAX };
        return 0; // false - overflow occurred
    }

    *nr = result;
    *pp = p as *mut c_char;
    1 // success
}

/// Gets a number from a string and skips over it.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a C string.
#[no_mangle]
pub unsafe extern "C" fn rs_getdigits(pp: *mut *mut c_char, strict: c_int, def: isize) -> isize {
    let mut number: isize = 0;
    let ok = rs_try_getdigits(pp, std::ptr::addr_of_mut!(number));

    if strict != 0 && ok == 0 {
        // In strict mode, abort on overflow
        std::process::abort();
    }

    if ok != 0 {
        number
    } else {
        def
    }
}

/// Gets an int number from a string.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a C string.
#[no_mangle]
pub unsafe extern "C" fn rs_getdigits_int(
    pp: *mut *mut c_char,
    strict: c_int,
    def: c_int,
) -> c_int {
    let number = rs_getdigits(pp, strict, def as isize);

    // Check bounds
    if i32::try_from(number).is_ok() {
        number as c_int
    } else {
        if strict != 0 {
            std::process::abort();
        }
        def
    }
}

/// Gets a long number from a string.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a C string.
#[no_mangle]
pub unsafe extern "C" fn rs_getdigits_long(
    pp: *mut *mut c_char,
    strict: c_int,
    def: c_long,
) -> c_long {
    let number = rs_getdigits(pp, strict, def as isize);

    // On 64-bit systems, isize and c_long are the same size
    // On 32-bit systems, c_long is typically 32 bits
    #[cfg(target_pointer_width = "64")]
    {
        number as c_long
    }

    #[cfg(target_pointer_width = "32")]
    {
        if number >= c_long::MIN as isize && number <= c_long::MAX as isize {
            number as c_long
        } else {
            if strict != 0 {
                std::process::abort();
            }
            def
        }
    }
}

/// Gets an int32_t number from a string.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a C string.
#[no_mangle]
pub unsafe extern "C" fn rs_getdigits_int32(pp: *mut *mut c_char, strict: c_int, def: i32) -> i32 {
    let number = rs_getdigits(pp, strict, def as isize);

    if i32::try_from(number).is_ok() {
        number as i32
    } else {
        if strict != 0 {
            std::process::abort();
        }
        def
    }
}

// ============================================================================
// Backslash save and file character detection
// ============================================================================

/// Halve the number of backslashes and return a new allocated string.
///
/// Like backslash_halve, but returns a newly allocated copy of the string
/// with backslashes halved.
///
/// # Safety
/// - `p` must be a valid pointer to a null-terminated C string.
/// - The returned pointer must be freed with libc::free when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn rs_backslash_halve_save(p: *const c_char) -> *mut c_char {
    if p.is_null() {
        return std::ptr::null_mut();
    }

    // Calculate length
    let len = libc::strlen(p);

    // Allocate result buffer (same size as input is always sufficient)
    let res = libc::malloc(len + 1) as *mut c_char;
    if res.is_null() {
        return std::ptr::null_mut();
    }

    let mut src = p;
    let mut dst = res;

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
    res
}

// External reference to path_has_wildcard (already in Rust)
extern "C" {
    fn rs_path_has_wildcard(p: *const c_char) -> c_int;
}

/// Check if a character is a valid file character or a wildcard.
///
/// Returns true if the character is valid in a file name or is a wildcard
/// character. Assumes characters above 0x100 are valid (multi-byte).
/// Explicitly treats ']' as a wildcard since path_has_wildcard("]") returns false.
///
/// # Safety
/// - This function accesses global `g_chartab` for file character detection.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_isfilec_or_wc(c: c_int) -> c_int {
    // Check if it's a valid file character
    if rs_vim_isfilec(c) != 0 {
        return 1;
    }

    // ']' is explicitly treated as wildcard (path_has_wildcard returns false for it)
    if c == b']' as c_int {
        return 1;
    }

    // Check for wildcard characters
    let buf: [c_char; 2] = [c as c_char, 0];
    if rs_path_has_wildcard(buf.as_ptr()) != 0 {
        return 1;
    }

    0
}

// ============================================================================
// vim_str2nr - String to number parsing
// ============================================================================

// STR2NR flag constants - matching charset.h enum
/// Allow binary numbers (0b...)
const STR2NR_BIN: c_int = 1 << 0;
/// Allow octal numbers (legacy 0...)
const STR2NR_OCT: c_int = 1 << 1;
/// Allow hexadecimal numbers (0x...)
const STR2NR_HEX: c_int = 1 << 2;
/// Allow octal with 0o prefix (0o...)
const STR2NR_OOCT: c_int = 1 << 3;
/// Ignore embedded single quotes in number
const STR2NR_QUOTE: c_int = 1 << 4;
/// Force parsing as a specific base (skip prefix detection)
const STR2NR_FORCE: c_int = 1 << 7;

// Type aliases matching Neovim's typval_defs.h
type VarnumberT = i64;
type UvarnumberT = u64;

const VARNUMBER_MAX: VarnumberT = i64::MAX;
const VARNUMBER_MIN: VarnumberT = i64::MIN;
const UVARNUMBER_MAX: UvarnumberT = u64::MAX;

/// Check if character is an octal digit ('0'-'7')
#[inline]
const fn ascii_isodigit(c: u8) -> bool {
    c >= b'0' && c <= b'7'
}

/// Number base for vim_str2nr parsing
enum Str2NrBase {
    Binary,
    Octal,
    Decimal,
    Hex,
}

/// Convert a string to a number in various bases.
///
/// Parses a string that may have a prefix indicating the base:
/// - `0x` or `0X` for hexadecimal
/// - `0b` or `0B` for binary
/// - `0o` or `0O` for octal (new style)
/// - `0` followed by octal digits for legacy octal
///
/// # Arguments
/// * `start` - Pointer to the start of the string to parse
/// * `prep` - If not null, set to prefix character ('0', 'x', 'b', 'o') or 0 for decimal
/// * `len` - If not null, set to length of the parsed number string
/// * `what` - Flags controlling which bases to recognize (STR2NR_BIN, STR2NR_OCT, etc.)
/// * `nptr` - If not null, set to the signed number value
/// * `unptr` - If not null, set to the unsigned number value
/// * `maxlen` - Maximum length to parse (0 = no limit)
/// * `strict` - If true, fail if alphanumeric follows the number
/// * `overflow` - If not null, set to true if overflow occurred
///
/// # Safety
/// * `start` must be a valid pointer to a null-terminated string
/// * All non-null output pointers must be valid and writable
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_vim_str2nr(
    start: *const c_char,
    prep: *mut c_int,
    len: *mut c_int,
    what: c_int,
    nptr: *mut VarnumberT,
    unptr: *mut UvarnumberT,
    maxlen: c_int,
    strict: bool,
    overflow: *mut bool,
) {
    if start.is_null() {
        return;
    }

    let start_ptr = start as *const u8;
    let mut ptr = start_ptr;
    let mut pre: c_int = 0; // default is decimal
    let mut un: UvarnumberT = 0;

    // Helper to check if we've reached the end of the string or maxlen
    let string_ended = |p: *const u8| -> bool {
        if maxlen == 0 {
            return false;
        }
        (p as isize - start_ptr as isize) >= maxlen as isize
    };

    if !len.is_null() {
        *len = 0;
    }

    // Check for leading '-'
    let negative = *ptr == b'-';
    if negative {
        ptr = ptr.add(1);
    }

    // Determine the base
    let base: Str2NrBase;

    if (what & STR2NR_FORCE) != 0 {
        // When forcing, main consideration is skipping the prefix
        match what & !(STR2NR_FORCE | STR2NR_QUOTE) {
            x if x == STR2NR_HEX => {
                // Skip 0x/0X prefix if present
                if !string_ended(ptr.add(2))
                    && *ptr == b'0'
                    && (*ptr.add(1) == b'x' || *ptr.add(1) == b'X')
                    && ascii_isxdigit(*ptr.add(2))
                {
                    ptr = ptr.add(2);
                }
                base = Str2NrBase::Hex;
            }
            x if x == STR2NR_BIN => {
                // Skip 0b/0B prefix if present
                if !string_ended(ptr.add(2))
                    && *ptr == b'0'
                    && (*ptr.add(1) == b'b' || *ptr.add(1) == b'B')
                    && ascii_isbdigit(*ptr.add(2))
                {
                    ptr = ptr.add(2);
                }
                base = Str2NrBase::Binary;
            }
            x if x == STR2NR_OCT || x == STR2NR_OOCT || x == (STR2NR_OCT | STR2NR_OOCT) => {
                // Skip 0o/0O prefix if present
                if !string_ended(ptr.add(2))
                    && *ptr == b'0'
                    && (*ptr.add(1) == b'o' || *ptr.add(1) == b'O')
                    && ascii_isodigit(*ptr.add(2))
                {
                    ptr = ptr.add(2);
                }
                base = Str2NrBase::Octal;
            }
            0 => {
                base = Str2NrBase::Decimal;
            }
            _ => {
                // Invalid combination, abort
                return;
            }
        }
    } else if (what & (STR2NR_HEX | STR2NR_OCT | STR2NR_OOCT | STR2NR_BIN)) != 0
        && !string_ended(ptr.add(1))
        && *ptr == b'0'
        && *ptr.add(1) != b'8'
        && *ptr.add(1) != b'9'
    {
        let pre_char = *ptr.add(1);

        // Detect hexadecimal: 0x or 0X followed by hex digit
        if (what & STR2NR_HEX) != 0
            && !string_ended(ptr.add(2))
            && (pre_char == b'X' || pre_char == b'x')
            && ascii_isxdigit(*ptr.add(2))
        {
            pre = pre_char as c_int;
            ptr = ptr.add(2);
            base = Str2NrBase::Hex;
        }
        // Detect binary: 0b or 0B followed by 0 or 1
        else if (what & STR2NR_BIN) != 0
            && !string_ended(ptr.add(2))
            && (pre_char == b'B' || pre_char == b'b')
            && ascii_isbdigit(*ptr.add(2))
        {
            pre = pre_char as c_int;
            ptr = ptr.add(2);
            base = Str2NrBase::Binary;
        }
        // Detect octal: 0o or 0O followed by octal digits
        else if (what & STR2NR_OOCT) != 0
            && !string_ended(ptr.add(2))
            && (pre_char == b'O' || pre_char == b'o')
            && ascii_isodigit(*ptr.add(2))
        {
            pre = pre_char as c_int;
            ptr = ptr.add(2);
            base = Str2NrBase::Octal;
        }
        // Detect old octal format: 0 followed by octal digits
        else if (what & STR2NR_OCT) != 0 && ascii_isodigit(*ptr.add(1)) {
            // Check that all following digits are octal (no 8 or 9)
            let mut all_octal = true;
            let mut i = 2isize;
            while !string_ended(ptr.add(i as usize)) && ascii_isdigit(*ptr.add(i as usize)) {
                if *ptr.add(i as usize) > b'7' {
                    all_octal = false;
                    break;
                }
                i += 1;
            }
            if all_octal {
                pre = b'0' as c_int;
                base = Str2NrBase::Octal;
            } else {
                base = Str2NrBase::Decimal;
            }
        } else {
            base = Str2NrBase::Decimal;
        }
    } else {
        base = Str2NrBase::Decimal;
    }

    // Parse the number based on the detected base
    let after_prefix = ptr;

    let (base_val, is_valid_digit): (UvarnumberT, fn(u8) -> bool) = match base {
        Str2NrBase::Binary => (2, ascii_isbdigit),
        Str2NrBase::Octal => (8, ascii_isodigit),
        Str2NrBase::Decimal => (10, ascii_isdigit),
        Str2NrBase::Hex => (16, ascii_isxdigit),
    };

    while !string_ended(ptr) && *ptr != 0 {
        // Handle embedded quotes (STR2NR_QUOTE flag)
        if (what & STR2NR_QUOTE) != 0 && ptr > after_prefix && *ptr == b'\'' {
            ptr = ptr.add(1);
            if !string_ended(ptr) && *ptr != 0 && is_valid_digit(*ptr) {
                continue;
            }
            ptr = ptr.sub(1);
        }

        if !is_valid_digit(*ptr) {
            break;
        }

        let digit: UvarnumberT = if base_val == 16 {
            rs_hex2nr(*ptr as c_int) as UvarnumberT
        } else {
            (*ptr - b'0') as UvarnumberT
        };

        // Check for overflow before multiplying
        if un < UVARNUMBER_MAX / base_val
            || (un == UVARNUMBER_MAX / base_val && (base_val != 10 || digit <= UVARNUMBER_MAX % 10))
        {
            un = base_val * un + digit;
        } else {
            un = UVARNUMBER_MAX;
            if !overflow.is_null() {
                *overflow = true;
            }
        }

        ptr = ptr.add(1);
    }

    // Check for alphanumeric immediately following (strict mode)
    if strict
        && (ptr as isize - start_ptr as isize) != maxlen as isize
        && *ptr != 0
        && ((*ptr >= b'A' && *ptr <= b'Z')
            || (*ptr >= b'a' && *ptr <= b'z')
            || (*ptr >= b'0' && *ptr <= b'9'))
    {
        return;
    }

    // Set output values
    if !prep.is_null() {
        *prep = pre;
    }

    if !len.is_null() {
        *len = (ptr as isize - start_ptr as isize) as c_int;
    }

    if !nptr.is_null() {
        if negative {
            // Handle negative overflow
            if un > VARNUMBER_MAX as UvarnumberT {
                *nptr = VARNUMBER_MIN;
                if !overflow.is_null() {
                    *overflow = true;
                }
            } else {
                *nptr = -(un as VarnumberT);
            }
        } else if un > VARNUMBER_MAX as UvarnumberT {
            *nptr = VARNUMBER_MAX;
            if !overflow.is_null() {
                *overflow = true;
            }
        } else {
            *nptr = un as VarnumberT;
        }
    }

    if !unptr.is_null() {
        *unptr = un;
    }
}

// ============================================================================
// Case Folding
// ============================================================================

// FFI declarations for mbyte functions
extern "C" {
    fn rs_utf_ptr2len(p: *const c_char) -> c_int;
    fn rs_utf_ptr2char(p: *const c_char) -> c_int;
    fn rs_utf_char2len(c: c_int) -> c_int;
    fn rs_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;
}

/// Type alias for case conversion function pointer.
/// Takes a codepoint and returns the converted codepoint.
pub type CaseConvertFn = unsafe extern "C" fn(c_int) -> c_int;

/// Convert a string to do ignore-case comparing (fold case).
///
/// This function converts all characters in the string to lowercase using
/// the provided case conversion function (mb_tolower from C).
///
/// When `buf` is NULL, allocates and returns a new string.
/// When `buf` is not NULL, writes to `buf` up to `buflen` bytes and returns `buf`.
///
/// # Safety
/// - `str` must be a valid pointer to at least `orglen` bytes
/// - If `buf` is not NULL, it must point to at least `buflen` bytes
/// - `tolower_fn` must be a valid function pointer (typically `mb_tolower`)
/// - The returned pointer must be freed by the caller if buf was NULL
#[no_mangle]
pub unsafe extern "C" fn rs_str_foldcase(
    str: *const c_char,
    orglen: c_int,
    buf: *mut c_char,
    buflen: c_int,
    tolower_fn: CaseConvertFn,
) -> *mut c_char {
    if str.is_null() {
        return std::ptr::null_mut();
    }

    let mut len = orglen as usize;
    let use_alloc = buf.is_null();

    // Either use provided buffer or allocate one
    let (mut data, mut capacity): (*mut c_char, usize) = if use_alloc {
        // Allocate initial buffer with extra space for potential growth
        let cap = len + 10;
        let ptr = libc::malloc(cap) as *mut c_char;
        if ptr.is_null() {
            return std::ptr::null_mut();
        }
        (ptr, cap)
    } else {
        // Using caller-provided buffer
        let blen = buflen as usize;
        if len >= blen {
            len = blen.saturating_sub(1);
        }
        (buf, blen)
    };

    // Copy original string to buffer
    std::ptr::copy_nonoverlapping(str, data, len);
    *data.add(len) = 0;

    // Convert each character to lowercase
    let mut i = 0usize;
    while i < len && *data.add(i) != 0 {
        let p = data.add(i);
        let c = rs_utf_ptr2char(p);
        let olen = rs_utf_ptr2len(p) as usize;
        let lc = tolower_fn(c);

        // Only replace when it's a valid sequence (ASCII or multi-byte) and changed
        if ((c < 0x80) || (olen > 1)) && (c != lc) {
            let nlen = rs_utf_char2len(lc) as usize;

            // If byte length changes, need to shift following characters
            if olen != nlen {
                if nlen > olen {
                    if use_alloc {
                        // Need more space: grow the buffer
                        let needed = len + (nlen - olen) + 1;
                        if needed > capacity {
                            let new_cap = needed + 10;
                            let new_data =
                                libc::realloc(data as *mut libc::c_void, new_cap) as *mut c_char;
                            if new_data.is_null() {
                                libc::free(data as *mut libc::c_void);
                                return std::ptr::null_mut();
                            }
                            data = new_data;
                            capacity = new_cap;
                        }
                    } else {
                        // Fixed buffer: check if we have space
                        if len + nlen - olen >= capacity {
                            // Out of space, keep old character
                            i += rs_utfc_ptr2len(data.add(i)) as usize;
                            continue;
                        }
                    }
                }

                // Shift following characters
                let src = data.add(i).add(olen);
                let dst = data.add(i).add(nlen);
                let remaining = len - i - olen;
                std::ptr::copy(src, dst, remaining + 1); // +1 for NUL
                len = (len as isize + (nlen as isize - olen as isize)) as usize;
            }

            // Write the lowercase character
            rs_utf_char2bytes(lc, data.add(i));
        }

        // Skip to next multi-byte character
        i += rs_utfc_ptr2len(data.add(i)) as usize;
    }

    data
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
    fn test_transchar_nonprint() {
        unsafe {
            let mut buf = [0i8; 16];

            // NUL (0x00) -> ^@ without uhex
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x00, false, EOL_UNIX);
            assert_eq!(&buf[..3], [b'^' as i8, b'@' as i8, 0]);

            // NL (0x0A) is converted to NUL first, then displayed as ^@
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x0A, false, EOL_UNIX);
            assert_eq!(&buf[..3], [b'^' as i8, b'@' as i8, 0]);

            // TAB (0x09) -> ^I
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x09, false, EOL_UNIX);
            assert_eq!(&buf[..3], [b'^' as i8, b'I' as i8, 0]);

            // DEL (0x7F) -> ^?
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x7F, false, EOL_UNIX);
            assert_eq!(&buf[..3], [b'^' as i8, b'?' as i8, 0]);

            // ESC (0x1B) -> ^[
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x1B, false, EOL_UNIX);
            assert_eq!(&buf[..3], [b'^' as i8, b'[' as i8, 0]);

            // High-bit character (0x80) uses hex format even without uhex
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x80, false, EOL_UNIX);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'8' as i8, b'0' as i8, b'>' as i8, 0]
            );

            // With uhex=true, control chars use hex format
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x00, true, EOL_UNIX);
            assert_eq!(
                &buf[..5],
                [b'<' as i8, b'0' as i8, b'0' as i8, b'>' as i8, 0]
            );

            // CR (0x0D) in MAC format is converted to NL first, then displayed as ^J
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x0D, false, EOL_MAC);
            assert_eq!(&buf[..3], [b'^' as i8, b'J' as i8, 0]);

            // CR (0x0D) in UNIX format is displayed as ^M (no conversion)
            rs_transchar_nonprint(buf.as_mut_ptr(), 0x0D, false, EOL_UNIX);
            assert_eq!(&buf[..3], [b'^' as i8, b'M' as i8, 0]);

            // Null buffer should not crash
            rs_transchar_nonprint(std::ptr::null_mut(), 0x00, false, EOL_UNIX);
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

    #[test]
    fn test_try_getdigits() {
        unsafe {
            // Simple number
            let mut s = *b"123abc\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let mut nr: isize = 0;
            let ok = rs_try_getdigits(std::ptr::addr_of_mut!(ptr), std::ptr::addr_of_mut!(nr));
            assert_eq!(ok, 1);
            assert_eq!(nr, 123);
            assert_eq!(*ptr, b'a' as c_char);

            // No digits - returns 0, pointer unchanged, but still success
            let mut s = *b"abc123\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let start = ptr;
            let mut nr: isize = 999;
            let ok = rs_try_getdigits(std::ptr::addr_of_mut!(ptr), std::ptr::addr_of_mut!(nr));
            assert_eq!(ok, 1); // success (no overflow)
            assert_eq!(nr, 0); // returns 0 when no conversion
            assert_eq!(ptr, start); // pointer unchanged

            // Negative number
            let mut s = *b"-456xyz\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let mut nr: isize = 0;
            let ok = rs_try_getdigits(std::ptr::addr_of_mut!(ptr), std::ptr::addr_of_mut!(nr));
            assert_eq!(ok, 1);
            assert_eq!(nr, -456);
            assert_eq!(*ptr, b'x' as c_char);

            // Just digits
            let mut s = *b"999\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let mut nr: isize = 0;
            let ok = rs_try_getdigits(std::ptr::addr_of_mut!(ptr), std::ptr::addr_of_mut!(nr));
            assert_eq!(ok, 1);
            assert_eq!(nr, 999);
            assert_eq!(*ptr, 0);
        }
    }

    #[test]
    fn test_getdigits() {
        unsafe {
            // Simple number
            let mut s = *b"42rest\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let result = rs_getdigits(std::ptr::addr_of_mut!(ptr), 0, 0);
            assert_eq!(result, 42);
            assert_eq!(*ptr, b'r' as c_char);

            // No digits - returns the parsed 0 (not the default)
            let mut s = *b"abc\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let result = rs_getdigits(std::ptr::addr_of_mut!(ptr), 0, 99);
            // strtoimax returns 0 when no digits, this is success so result is 0
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn test_getdigits_int() {
        unsafe {
            // Simple number
            let mut s = *b"100xyz\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let result = rs_getdigits_int(std::ptr::addr_of_mut!(ptr), 0, 0);
            assert_eq!(result, 100);
            assert_eq!(*ptr, b'x' as c_char);

            // Negative number
            let mut s = *b"-50xyz\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let result = rs_getdigits_int(std::ptr::addr_of_mut!(ptr), 0, 0);
            assert_eq!(result, -50);
            assert_eq!(*ptr, b'x' as c_char);
        }
    }

    #[test]
    fn test_getdigits_long() {
        unsafe {
            // Simple number
            let mut s = *b"1000000\0";
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let result = rs_getdigits_long(std::ptr::addr_of_mut!(ptr), 0, 0);
            assert_eq!(result, 1_000_000);
            assert_eq!(*ptr, 0);
        }
    }

    #[test]
    fn test_getdigits_int32() {
        unsafe {
            // Simple number
            let mut s = *b"2147483647\0"; // i32::MAX
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let result = rs_getdigits_int32(std::ptr::addr_of_mut!(ptr), 0, 0);
            assert_eq!(result, i32::MAX);

            // Negative number
            let mut s = *b"-2147483648\0"; // i32::MIN
            let mut ptr = s.as_mut_ptr().cast::<c_char>();
            let result = rs_getdigits_int32(std::ptr::addr_of_mut!(ptr), 0, 0);
            assert_eq!(result, i32::MIN);
        }
    }

    #[test]
    fn test_vim_str2nr_decimal() {
        unsafe {
            // Simple decimal
            let s = CString::new("123").unwrap();
            let mut len: c_int = 0;
            let mut nr: VarnumberT = 0;
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                0,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 123);
            assert_eq!(len, 3);

            // Decimal with trailing text
            let s = CString::new("456abc").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                0,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 456);
            assert_eq!(len, 3);

            // Negative decimal
            let s = CString::new("-789").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                0,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, -789);
            assert_eq!(len, 4);
        }
    }

    #[test]
    fn test_vim_str2nr_hex() {
        unsafe {
            let mut len: c_int = 0;
            let mut nr: VarnumberT = 0;
            let mut pre: c_int = 0;

            // Hex with 0x prefix
            let s = CString::new("0xff").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                &raw mut pre,
                &raw mut len,
                STR2NR_HEX,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 255);
            assert_eq!(len, 4);
            assert_eq!(pre, b'x' as c_int);

            // Hex with 0X prefix
            let s = CString::new("0X1A2B").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                &raw mut pre,
                &raw mut len,
                STR2NR_HEX,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 0x1A2B);
            assert_eq!(len, 6);
            assert_eq!(pre, b'X' as c_int);
        }
    }

    #[test]
    fn test_vim_str2nr_binary() {
        unsafe {
            let mut len: c_int = 0;
            let mut nr: VarnumberT = 0;
            let mut pre: c_int = 0;

            // Binary with 0b prefix
            let s = CString::new("0b1010").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                &raw mut pre,
                &raw mut len,
                STR2NR_BIN,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 10);
            assert_eq!(len, 6);
            assert_eq!(pre, b'b' as c_int);

            // Binary with 0B prefix
            let s = CString::new("0B1111").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                &raw mut pre,
                &raw mut len,
                STR2NR_BIN,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 15);
            assert_eq!(len, 6);
            assert_eq!(pre, b'B' as c_int);
        }
    }

    #[test]
    fn test_vim_str2nr_octal() {
        unsafe {
            let mut len: c_int = 0;
            let mut nr: VarnumberT = 0;
            let mut pre: c_int = 0;

            // New-style octal with 0o prefix
            let s = CString::new("0o777").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                &raw mut pre,
                &raw mut len,
                STR2NR_OOCT,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 0o777);
            assert_eq!(len, 5);
            assert_eq!(pre, b'o' as c_int);

            // Old-style octal with leading 0
            let s = CString::new("0755").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                &raw mut pre,
                &raw mut len,
                STR2NR_OCT,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 0o755);
            assert_eq!(len, 4);
            assert_eq!(pre, b'0' as c_int);

            // Old octal: 0 followed by 8 or 9 should be decimal
            let s = CString::new("089").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                &raw mut pre,
                &raw mut len,
                STR2NR_OCT,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 89);
            assert_eq!(pre, 0); // decimal
        }
    }

    #[test]
    fn test_vim_str2nr_quote() {
        unsafe {
            let mut len: c_int = 0;
            let mut nr: VarnumberT = 0;

            // Number with embedded quote
            let s = CString::new("1'234'567").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                STR2NR_QUOTE,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 1_234_567);
            assert_eq!(len, 9);

            // Hex with embedded quote
            let s = CString::new("0xff'ff").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                STR2NR_HEX | STR2NR_QUOTE,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 0xffff);
            assert_eq!(len, 7);
        }
    }

    #[test]
    fn test_vim_str2nr_maxlen() {
        unsafe {
            let mut len: c_int = 0;
            let mut nr: VarnumberT = 0;

            // Parse only first 3 characters
            let s = CString::new("12345").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                0,
                &raw mut nr,
                std::ptr::null_mut(),
                3,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 123);
            assert_eq!(len, 3);
        }
    }

    #[test]
    fn test_vim_str2nr_force() {
        unsafe {
            let mut len: c_int = 0;
            let mut nr: VarnumberT = 0;

            // Force hex - parse without needing 0x prefix
            let s = CString::new("ff").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                STR2NR_FORCE | STR2NR_HEX,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 255);
            assert_eq!(len, 2);

            // Force hex with 0x prefix - should skip prefix
            let s = CString::new("0xff").unwrap();
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                &raw mut len,
                STR2NR_FORCE | STR2NR_HEX,
                &raw mut nr,
                std::ptr::null_mut(),
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(nr, 255);
            assert_eq!(len, 4);
        }
    }

    #[test]
    fn test_vim_str2nr_unsigned() {
        unsafe {
            let mut un: UvarnumberT = 0;

            // Large unsigned number
            let s = CString::new("18446744073709551615").unwrap(); // u64::MAX
            rs_vim_str2nr(
                s.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &raw mut un,
                0,
                false,
                std::ptr::null_mut(),
            );
            assert_eq!(un, u64::MAX);
        }
    }

    #[test]
    fn test_charset_constants() {
        // Verify chartab flag constants match C definitions
        assert_eq!(CT_CELL_MASK, 0x07);
        assert_eq!(CT_PRINT_CHAR, 0x10);
        assert_eq!(CT_ID_CHAR, 0x20);
        assert_eq!(CT_FNAME_CHAR, 0x40);

        // Verify control and EOL constants
        assert_eq!(CTRL_V, 22);
        assert_eq!(EOL_UNIX, 0);
        assert_eq!(EOL_DOS, 1);
        assert_eq!(EOL_MAC, 2);
        assert_eq!(NL, 0x0A);
        assert_eq!(CAR, 0x0D);
        assert_eq!(NUL, 0x00);
    }
}
