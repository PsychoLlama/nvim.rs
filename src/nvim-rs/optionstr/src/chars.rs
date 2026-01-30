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
    fn rs_hexhex2nr(p: *const c_char) -> c_int;

    /// Get the display width of a unicode character
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
}
