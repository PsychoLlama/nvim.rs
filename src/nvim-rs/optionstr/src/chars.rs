//! Fillchars and listchars handling
//!
//! This module provides utilities for validating and managing
//! the 'fillchars' and 'listchars' options.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

// schar_T is uint32_t in C
type ScharT = u32;

// External function declarations
extern "C" {
    /// Convert a hex pair to a number (e.g., "FF" -> 255)
    /// Returns -1 for invalid input
    #[link_name = "hexhex2nr"]
    fn rs_hexhex2nr(p: *const c_char) -> c_int;

    /// Get the display width of a unicode character
    #[link_name = "char2cells"]
    fn rs_char2cells(c: c_int) -> c_int;

    /// Get the byte length of a UTF-8 character including composing characters
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;

    /// Convert a UTF-8 string to schar_T, also returns first codepoint
    fn rs_utfc_ptr2schar(p: *const c_char, firstc: *mut c_int) -> ScharT;

    /// Convert a unicode codepoint to schar_T
    fn rs_schar_from_char(c: c_int) -> ScharT;
}

// =============================================================================
// Fillchars Field Names
// =============================================================================

/// Fillchars field names (for 'fillchars' option)
pub const FCS_FIELDS: &[&str] = &[
    "stl",       // Status line of current window
    "stlnc",     // Status line of non-current windows
    "wbr",       // Window bar
    "horiz",     // Horizontal separators
    "horizup",   // Horizontal separator + up corner
    "horizdown", // Horizontal separator + down corner
    "vert",      // Vertical separators
    "vertleft",  // Vertical separator + left corner
    "vertright", // Vertical separator + right corner
    "verthoriz", // Vertical + horizontal intersection
    "fold",      // Filling 'foldtext'
    "foldopen",  // Open fold marker
    "foldclose", // Closed fold marker
    "foldsep",   // Fold separator
    "foldinner", // Inner fold marker
    "diff",      // Deleted lines of 'diff'
    "msgsep",    // Message separator
    "eob",       // Empty lines at end of buffer
    "lastline",  // '@' for last line
    "trunc",     // '>' for truncated lines
    "truncrl",   // '<' for truncated lines (rightleft)
];

/// Get fillchars field count
#[no_mangle]
pub extern "C" fn rs_fcs_field_count() -> c_int {
    FCS_FIELDS.len() as c_int
}

/// Get fillchars field name by index
///
/// Returns null pointer if index is out of bounds.
#[no_mangle]
pub extern "C" fn rs_fcs_field_name(idx: c_int) -> *const c_char {
    if idx < 0 || idx >= FCS_FIELDS.len() as c_int {
        return std::ptr::null();
    }
    FCS_FIELDS[idx as usize].as_ptr().cast::<c_char>()
}

// =============================================================================
// Listchars Field Names
// =============================================================================

/// Listchars field names (for 'listchars' option)
pub const LCS_FIELDS: &[&str] = &[
    "eol",            // End of line
    "extends",        // Extends indicator
    "nbsp",           // Non-breaking space
    "precedes",       // Precedes indicator
    "space",          // Space character
    "tab",            // Tab character
    "lead",           // Leading space
    "trail",          // Trailing space
    "conceal",        // Conceal character
    "multispace",     // Multiple spaces
    "leadmultispace", // Leading multiple spaces
];

/// Get listchars field count
#[no_mangle]
pub extern "C" fn rs_lcs_field_count() -> c_int {
    LCS_FIELDS.len() as c_int
}

/// Get listchars field name by index
///
/// Returns null pointer if index is out of bounds.
#[no_mangle]
pub extern "C" fn rs_lcs_field_name(idx: c_int) -> *const c_char {
    if idx < 0 || idx >= LCS_FIELDS.len() as c_int {
        return std::ptr::null();
    }
    LCS_FIELDS[idx as usize].as_ptr().cast::<c_char>()
}

// =============================================================================
// Chars Field Validation
// =============================================================================

/// Check if a field name is valid for fillchars
///
/// # Safety
/// The `name` pointer must be valid for reading up to and including the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_is_valid_fcs_field(name: *const c_char, len: usize) -> bool {
    if name.is_null() || len == 0 {
        return false;
    }

    let name_slice = std::slice::from_raw_parts(name.cast::<u8>(), len);
    let Ok(name_str) = std::str::from_utf8(name_slice) else {
        return false;
    };

    FCS_FIELDS.contains(&name_str)
}

/// Check if a field name is valid for listchars
///
/// # Safety
/// The `name` pointer must be valid for reading up to and including the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_is_valid_lcs_field(name: *const c_char, len: usize) -> bool {
    if name.is_null() || len == 0 {
        return false;
    }

    let name_slice = std::slice::from_raw_parts(name.cast::<u8>(), len);
    let Ok(name_str) = std::str::from_utf8(name_slice) else {
        return false;
    };

    LCS_FIELDS.contains(&name_str)
}

// =============================================================================
// Default Values
// =============================================================================

/// Default fillchars values
pub mod fcs_defaults {
    pub const STL: &str = " ";
    pub const STLNC: &str = " ";
    pub const WBR: &str = " ";
    pub const HORIZ: &str = "─";
    pub const HORIZ_FALLBACK: &str = "-";
    pub const HORIZUP: &str = "┴";
    pub const HORIZUP_FALLBACK: &str = "-";
    pub const HORIZDOWN: &str = "┬";
    pub const HORIZDOWN_FALLBACK: &str = "-";
    pub const VERT: &str = "│";
    pub const VERT_FALLBACK: &str = "|";
    pub const VERTLEFT: &str = "┤";
    pub const VERTLEFT_FALLBACK: &str = "|";
    pub const VERTRIGHT: &str = "├";
    pub const VERTRIGHT_FALLBACK: &str = "|";
    pub const VERTHORIZ: &str = "┼";
    pub const VERTHORIZ_FALLBACK: &str = "+";
    pub const FOLD: &str = "·";
    pub const FOLD_FALLBACK: &str = "-";
    pub const FOLDOPEN: &str = "-";
    pub const FOLDCLOSED: &str = "+";
    pub const FOLDSEP: &str = "│";
    pub const FOLDSEP_FALLBACK: &str = "|";
    pub const DIFF: &str = "-";
    pub const MSGSEP: &str = " ";
    pub const EOB: &str = "~";
    pub const LASTLINE: &str = "@";
    pub const TRUNC: &str = ">";
    pub const TRUNCRL: &str = "<";
}

/// Get default fillchar value for a field
#[no_mangle]
pub extern "C" fn rs_fcs_default(idx: c_int) -> *const c_char {
    let default: &str = match idx {
        0 => fcs_defaults::STL,
        1 => fcs_defaults::STLNC,
        2 => fcs_defaults::WBR,
        3 => fcs_defaults::HORIZ,
        4 => fcs_defaults::HORIZUP,
        5 => fcs_defaults::HORIZDOWN,
        6 => fcs_defaults::VERT,
        7 => fcs_defaults::VERTLEFT,
        8 => fcs_defaults::VERTRIGHT,
        9 => fcs_defaults::VERTHORIZ,
        10 => fcs_defaults::FOLD,
        11 => fcs_defaults::FOLDOPEN,
        12 => fcs_defaults::FOLDCLOSED,
        13 => fcs_defaults::FOLDSEP,
        15 => fcs_defaults::DIFF,
        16 => fcs_defaults::MSGSEP,
        17 => fcs_defaults::EOB,
        18 => fcs_defaults::LASTLINE,
        19 => fcs_defaults::TRUNC,
        20 => fcs_defaults::TRUNCRL,
        // 14 (foldinner) has no default
        _ => return std::ptr::null(),
    };
    default.as_ptr().cast::<c_char>()
}

/// Get fallback fillchar value for a field
#[no_mangle]
pub extern "C" fn rs_fcs_fallback(idx: c_int) -> *const c_char {
    let fallback: &str = match idx {
        3 => fcs_defaults::HORIZ_FALLBACK,
        4 => fcs_defaults::HORIZUP_FALLBACK,
        5 => fcs_defaults::HORIZDOWN_FALLBACK,
        6 => fcs_defaults::VERT_FALLBACK,
        7 => fcs_defaults::VERTLEFT_FALLBACK,
        8 => fcs_defaults::VERTRIGHT_FALLBACK,
        9 => fcs_defaults::VERTHORIZ_FALLBACK,
        10 => fcs_defaults::FOLD_FALLBACK,
        13 => fcs_defaults::FOLDSEP_FALLBACK,
        _ => return std::ptr::null(),
    };
    fallback.as_ptr().cast::<c_char>()
}

// =============================================================================
// Character Parsing for fillchars/listchars
// =============================================================================

/// Field index for the "tab" field in listchars (index 5)
const LCS_TAB_IDX: usize = 5;

/// Field index for "multispace" in listchars (index 9)
const LCS_MULTISPACE_IDX: usize = 9;

/// Field index for "leadmultispace" in listchars (index 10)
const LCS_LEADMULTISPACE_IDX: usize = 10;

/// Maximum number of characters that can be returned for a field
const MAX_FIELD_CHARS: usize = 3;

/// Error codes for field parsing
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharsParseError {
    /// No error
    Ok = 0,
    /// Invalid field name
    InvalidField = 1,
    /// Wrong number of characters for field
    WrongCount = 2,
    /// Invalid/double-width character
    InvalidChar = 3,
}

/// Result of parsing a single field:value pair
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CharsFieldResult {
    /// Field index (into FCS_FIELDS or LCS_FIELDS)
    pub field_idx: c_int,
    /// Number of valid characters in chars array
    pub char_count: c_int,
    /// Parsed characters (up to 3 for tab field)
    pub chars: [ScharT; MAX_FIELD_CHARS],
    /// For multispace fields: total character count
    pub multispace_len: c_int,
    /// Number of bytes consumed from input
    pub bytes_consumed: c_int,
    /// Error code
    pub error: CharsParseError,
}

impl Default for CharsFieldResult {
    fn default() -> Self {
        Self {
            field_idx: -1,
            char_count: 0,
            chars: [0; MAX_FIELD_CHARS],
            multispace_len: 0,
            bytes_consumed: 0,
            error: CharsParseError::Ok,
        }
    }
}

/// Find field index by name
fn find_field_index(name: &[u8], is_listchars: bool) -> Option<usize> {
    let fields: &[&str] = if is_listchars { LCS_FIELDS } else { FCS_FIELDS };

    // Convert name slice to str for comparison
    let name_str = std::str::from_utf8(name).ok()?;

    for (idx, field) in fields.iter().enumerate() {
        if *field == name_str {
            return Some(idx);
        }
    }
    None
}

/// Parse an encoded character and advance the pointer.
///
/// Calls `utfc_ptr2schar(p)` and returns the character.
/// If "p" starts with "\x", "\u" or "\U" the hex or unicode value is used:
/// - `\xNN` - single hex byte
/// - `\uNNNN` - 2-byte unicode (4 hex digits)
/// - `\UNNNNNNNN` - 4-byte unicode (8 hex digits)
///
/// Returns 0 for:
/// - Invalid hex sequences
/// - Invalid UTF-8 bytes
/// - Double-width characters (not allowed in fillchars/listchars)
///
/// # Safety
/// - `p` must be a valid pointer to a pointer to a null-terminated C string
/// - The inner pointer will be advanced past the parsed character
#[no_mangle]
pub unsafe extern "C" fn rs_get_encoded_char_adv(p: *mut *const c_char) -> ScharT {
    if p.is_null() || (*p).is_null() {
        return 0;
    }

    let s = *p;
    let b0 = *s as u8;
    let b1 = *s.add(1) as u8;

    // Check for escape sequences: \x, \u, \U
    if b0 == b'\\' && (b1 == b'x' || b1 == b'u' || b1 == b'U') {
        // Determine number of bytes to read based on escape type
        let bytes: i32 = match b1 {
            b'x' => 1, // \xNN - 1 byte (2 hex digits)
            b'u' => 2, // \uNNNN - 2 bytes (4 hex digits)
            b'U' => 4, // \UNNNNNNNN - 4 bytes (8 hex digits)
            _ => unreachable!(),
        };

        let mut num: i64 = 0;
        for _ in 0..bytes {
            // Skip 2 chars (\x, \u, or \U on first iteration, then NN pairs)
            *p = (*p).add(2);
            let n = rs_hexhex2nr(*p);
            if n < 0 {
                return 0; // Invalid hex
            }
            num = num * 256 + i64::from(n);
        }
        // Skip final 2 hex digits
        *p = (*p).add(2);

        // Double-width characters are not allowed
        if rs_char2cells(num as c_int) > 1 {
            return 0;
        }

        return rs_schar_from_char(num as c_int);
    }

    // Regular UTF-8 character
    let clen = rs_utfc_ptr2len(s);
    let mut firstc: c_int = 0;
    let c = rs_utfc_ptr2schar(s, &mut firstc);

    *p = (*p).add(clen as usize);

    // Invalid UTF-8 byte (single byte >= 128) or double-width not allowed
    if (clen == 1 && firstc > 127) || rs_char2cells(firstc) > 1 {
        return 0;
    }

    c
}

/// Parse a single field:value pair from a fillchars/listchars option string.
///
/// The input should point to the start of a "field:value" substring.
/// This function will:
/// 1. Find the field name (before ':')
/// 2. Look it up in the appropriate field list
/// 3. Parse the character value(s)
/// 4. Return the result with field index, parsed chars, and bytes consumed
///
/// Special handling:
/// - "tab" field: requires 2-3 characters
/// - "multispace"/"leadmultispace": counts characters but doesn't store them all
///   (caller must handle separately)
///
/// # Arguments
/// * `p` - Pointer to the field:value string (null-terminated)
/// * `is_listchars` - true for 'listchars', false for 'fillchars'
/// * `result` - Output struct to fill with parse results
///
/// # Safety
/// - `p` must point to a valid null-terminated C string
/// - `result` must point to valid memory
#[no_mangle]
pub unsafe extern "C" fn rs_parse_chars_field(
    p: *const c_char,
    is_listchars: bool,
    result: *mut CharsFieldResult,
) {
    if p.is_null() || result.is_null() {
        return;
    }

    let res = &mut *result;
    *res = CharsFieldResult::default();

    // Find the colon separator
    let mut colon_pos = 0usize;
    while *p.add(colon_pos) != 0 && *p.add(colon_pos) != b':' as c_char {
        colon_pos += 1;
    }

    // No colon found or empty field name
    if *p.add(colon_pos) != b':' as c_char || colon_pos == 0 {
        res.error = CharsParseError::InvalidField;
        return;
    }

    // Extract field name
    let field_name = std::slice::from_raw_parts(p.cast::<u8>(), colon_pos);

    // Look up field
    let Some(field_idx) = find_field_index(field_name, is_listchars) else {
        res.error = CharsParseError::InvalidField;
        return;
    };

    res.field_idx = field_idx as c_int;

    // Position after the colon
    let mut s = p.add(colon_pos + 1);

    // Handle multispace fields specially
    if is_listchars && (field_idx == LCS_MULTISPACE_IDX || field_idx == LCS_LEADMULTISPACE_IDX) {
        let mut count = 0i32;
        while *s != 0 && *s != b',' as c_char {
            let c = rs_get_encoded_char_adv(&mut s);
            if c == 0 {
                res.error = CharsParseError::InvalidChar;
                return;
            }
            count += 1;
        }
        if count == 0 {
            res.error = CharsParseError::WrongCount;
            return;
        }
        res.multispace_len = count;
        res.bytes_consumed = s.offset_from(p) as c_int;
        return;
    }

    // Regular field: parse first character
    if *s == 0 {
        res.error = CharsParseError::WrongCount;
        return;
    }

    let c1 = rs_get_encoded_char_adv(&mut s);
    if c1 == 0 {
        res.error = CharsParseError::InvalidChar;
        return;
    }
    res.chars[0] = c1;
    res.char_count = 1;

    // Handle tab field specially (requires 2-3 characters)
    if is_listchars && field_idx == LCS_TAB_IDX {
        if *s == 0 {
            res.error = CharsParseError::WrongCount;
            return;
        }

        let c2 = rs_get_encoded_char_adv(&mut s);
        if c2 == 0 {
            res.error = CharsParseError::InvalidChar;
            return;
        }
        res.chars[1] = c2;
        res.char_count = 2;

        // Optional third character
        if *s != 0 && *s != b',' as c_char {
            let c3 = rs_get_encoded_char_adv(&mut s);
            if c3 == 0 {
                res.error = CharsParseError::InvalidChar;
                return;
            }
            res.chars[2] = c3;
            res.char_count = 3;
        }
    }

    // Check that we're at end of field (comma or NUL)
    if *s != 0 && *s != b',' as c_char {
        res.error = CharsParseError::WrongCount;
        return;
    }

    res.bytes_consumed = s.offset_from(p) as c_int;
}

/// Count multispace characters without storing them.
///
/// This is used for the first pass of set_chars_option() to determine
/// how much memory to allocate for multispace arrays.
///
/// # Safety
/// - `p` must point to a valid null-terminated C string starting at the value
///   (after "multispace:" or "leadmultispace:")
/// - Returns the count of valid single-width characters, or -1 if there's an
///   invalid character, or 0 if the string is empty
#[no_mangle]
pub unsafe extern "C" fn rs_count_multispace_chars(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    let mut s = p;
    let mut count = 0i32;

    while *s != 0 && *s != b',' as c_char {
        let c = rs_get_encoded_char_adv(&mut s);
        if c == 0 {
            return -1; // Invalid character
        }
        count += 1;
    }

    count
}

/// Parse multispace characters into a provided buffer.
///
/// # Safety
/// - `p` must point to a valid null-terminated C string starting at the value
/// - `buf` must have capacity for at least `buf_len` ScharT values
/// - Returns the number of characters written
#[no_mangle]
pub unsafe extern "C" fn rs_parse_multispace_chars(
    p: *const c_char,
    buf: *mut ScharT,
    buf_len: c_int,
) -> c_int {
    if p.is_null() || buf.is_null() || buf_len <= 0 {
        return 0;
    }

    let mut s = p;
    let mut count = 0i32;

    while *s != 0 && *s != b',' as c_char && count < buf_len {
        let c = rs_get_encoded_char_adv(&mut s);
        if c == 0 {
            break;
        }
        *buf.add(count as usize) = c;
        count += 1;
    }

    count
}

/// Validation result for a chars option value
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CharsValidateResult {
    /// Error code (0 = success)
    pub error: CharsParseError,
    /// Index of the field that caused the error (-1 if no specific field)
    pub error_field_idx: c_int,
    /// Byte offset where error occurred
    pub error_offset: c_int,
    /// Length needed for multispace array (if any)
    pub multispace_len: c_int,
    /// Length needed for leadmultispace array (if any)
    pub leadmultispace_len: c_int,
}

impl Default for CharsValidateResult {
    fn default() -> Self {
        Self {
            error: CharsParseError::Ok,
            error_field_idx: -1,
            error_offset: 0,
            multispace_len: 0,
            leadmultispace_len: 0,
        }
    }
}

/// Validate a complete fillchars or listchars option value.
///
/// This function parses the entire comma-separated option string and validates
/// each field:value pair without storing the results.
///
/// # Arguments
/// * `value` - The option value string (null-terminated)
/// * `is_listchars` - true for 'listchars', false for 'fillchars'
/// * `result` - Output struct with validation results
///
/// # Safety
/// - `value` must point to a valid null-terminated C string
/// - `result` must point to valid memory
#[no_mangle]
pub unsafe extern "C" fn rs_validate_chars_option(
    value: *const c_char,
    is_listchars: bool,
    result: *mut CharsValidateResult,
) {
    if result.is_null() {
        return;
    }

    let res = &mut *result;
    *res = CharsValidateResult::default();

    // Handle null or empty value
    if value.is_null() || *value == 0 {
        return; // Empty value is valid
    }

    let mut p = value;
    let start = value;

    while *p != 0 {
        // Parse this field
        let mut field_result = CharsFieldResult::default();
        rs_parse_chars_field(p, is_listchars, &mut field_result);

        if field_result.error != CharsParseError::Ok {
            res.error = field_result.error;
            res.error_field_idx = field_result.field_idx;
            res.error_offset = p.offset_from(start) as c_int;
            return;
        }

        // Track multispace lengths
        if is_listchars && field_result.field_idx == LCS_MULTISPACE_IDX as c_int {
            res.multispace_len = field_result.multispace_len;
        } else if is_listchars && field_result.field_idx == LCS_LEADMULTISPACE_IDX as c_int {
            res.leadmultispace_len = field_result.multispace_len;
        }

        // Advance past this field
        p = p.add(field_result.bytes_consumed as usize);

        // Skip comma separator
        if *p == b',' as c_char {
            p = p.add(1);
        }
    }
}

/// Simple validation function that just returns true/false.
///
/// Use `rs_validate_chars_option()` for detailed error information.
///
/// # Safety
/// - `value` must point to a valid null-terminated C string or be null
#[no_mangle]
pub unsafe extern "C" fn rs_is_valid_chars_option(
    value: *const c_char,
    is_listchars: bool,
) -> bool {
    let mut result = CharsValidateResult::default();
    rs_validate_chars_option(value, is_listchars, &mut result);
    result.error == CharsParseError::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcs_fields() {
        assert_eq!(rs_fcs_field_count(), 21);
        assert!(!rs_fcs_field_name(0).is_null());
        assert!(rs_fcs_field_name(-1).is_null());
        assert!(rs_fcs_field_name(100).is_null());
    }

    #[test]
    fn test_lcs_fields() {
        assert_eq!(rs_lcs_field_count(), 11);
        assert!(!rs_lcs_field_name(0).is_null());
        assert!(rs_lcs_field_name(-1).is_null());
    }

    #[test]
    fn test_field_validation() {
        unsafe {
            assert!(rs_is_valid_fcs_field(b"stl\0".as_ptr().cast(), 3));
            assert!(rs_is_valid_fcs_field(b"vert\0".as_ptr().cast(), 4));
            assert!(!rs_is_valid_fcs_field(b"invalid\0".as_ptr().cast(), 7));

            assert!(rs_is_valid_lcs_field(b"eol\0".as_ptr().cast(), 3));
            assert!(rs_is_valid_lcs_field(b"tab\0".as_ptr().cast(), 3));
            assert!(!rs_is_valid_lcs_field(b"invalid\0".as_ptr().cast(), 7));
        }
    }

    #[test]
    fn test_find_field_index() {
        // Fillchars
        assert_eq!(find_field_index(b"stl", false), Some(0));
        assert_eq!(find_field_index(b"vert", false), Some(6));
        assert_eq!(find_field_index(b"invalid", false), None);

        // Listchars
        assert_eq!(find_field_index(b"eol", true), Some(0));
        assert_eq!(find_field_index(b"tab", true), Some(5));
        assert_eq!(find_field_index(b"multispace", true), Some(9));
        assert_eq!(find_field_index(b"invalid", true), None);
    }

    #[test]
    fn test_chars_parse_error_values() {
        // Ensure repr(C) values are correct
        assert_eq!(CharsParseError::Ok as i32, 0);
        assert_eq!(CharsParseError::InvalidField as i32, 1);
        assert_eq!(CharsParseError::WrongCount as i32, 2);
        assert_eq!(CharsParseError::InvalidChar as i32, 3);
    }
}
