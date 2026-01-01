//! Regular expression utilities for Neovim
//!
//! This crate provides Rust implementations of regex helper functions,
//! wrapping the Vim regex engines which remain in C.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;
use std::sync::OnceLock;

// =============================================================================
// Constants
// =============================================================================

/// ASCII control characters used in backslash_trans
const CAR: c_int = 13; // Carriage return
const TAB: c_int = 9; // Tab
const ESC: c_int = 27; // Escape
const BS: c_int = 8; // Backspace

/// Magic character offset (negative chars are magic)
const MAGIC_OFFSET: c_int = 256;

/// Return values for re_multi_type
const NOT_MULTI: c_int = 0;
const MULTI_ONE: c_int = 1;
const MULTI_MULT: c_int = 2;

/// regflags values
const RF_HASNL: u32 = 4;

/// Character class flags for the class table
const RI_DIGIT: i16 = 0x01;
const RI_HEX: i16 = 0x02;
const RI_OCTAL: i16 = 0x04;
const RI_WORD: i16 = 0x08;
const RI_HEAD: i16 = 0x10;
const RI_ALPHA: i16 = 0x20;
const RI_LOWER: i16 = 0x40;
const RI_UPPER: i16 = 0x80;
const RI_WHITE: i16 = 0x100;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to regprog_T (compiled regular expression program).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RegprogHandle(*mut std::ffi::c_void);

impl RegprogHandle {
    /// Create a handle from a raw pointer.
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to regmatch_T (single-line match result).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RegmatchHandle(*mut std::ffi::c_void);

impl RegmatchHandle {
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to regmmatch_T (multi-line match result).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RegmmatchHandle(*mut std::ffi::c_void);

impl RegmmatchHandle {
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    /// Get the regflags field from a regprog_T.
    fn nvim_regprog_get_regflags(prog: RegprogHandle) -> c_int;
}

// =============================================================================
// Magic Functions
// =============================================================================

/// Check if a character is magic (negative value).
#[inline]
const fn is_magic(x: c_int) -> bool {
    x < 0
}

/// Convert a magic character back to its ASCII value.
#[inline]
const fn un_magic(x: c_int) -> c_int {
    x + MAGIC_OFFSET
}

/// Convert an ASCII character to its magic form.
#[inline]
const fn magic(x: c_int) -> c_int {
    x - MAGIC_OFFSET
}

/// Remove magic from a character.
/// If it's magic, convert it back to regular. Otherwise return as-is.
#[inline]
const fn no_magic_impl(x: c_int) -> c_int {
    if is_magic(x) {
        un_magic(x)
    } else {
        x
    }
}

/// Toggle the magic state of a character.
/// If magic, make it regular. If regular, make it magic.
#[inline]
const fn toggle_magic_impl(x: c_int) -> c_int {
    if is_magic(x) {
        un_magic(x)
    } else {
        magic(x)
    }
}

/// Return the type of "multi" operator for character c.
/// NOT_MULTI (0) if not a multi operator.
/// MULTI_ONE (1) if single multi operator (@, =, ?).
/// MULTI_MULT (2) if multi multi operator (*, +, {).
#[inline]
const fn re_multi_type_impl(c: c_int) -> c_int {
    // Magic('@') = '@' - 256 = 64 - 256 = -192
    // Magic('=') = '=' - 256 = 61 - 256 = -195
    // Magic('?') = '?' - 256 = 63 - 256 = -193
    // Magic('*') = '*' - 256 = 42 - 256 = -214
    // Magic('+') = '+' - 256 = 43 - 256 = -213
    // Magic('{') = '{' - 256 = 123 - 256 = -133
    let magic_at = magic(b'@' as c_int);
    let magic_eq = magic(b'=' as c_int);
    let magic_q = magic(b'?' as c_int);
    let magic_star = magic(b'*' as c_int);
    let magic_plus = magic(b'+' as c_int);
    let magic_brace = magic(b'{' as c_int);

    if c == magic_at || c == magic_eq || c == magic_q {
        MULTI_ONE
    } else if c == magic_star || c == magic_plus || c == magic_brace {
        MULTI_MULT
    } else {
        NOT_MULTI
    }
}

/// Translate '\x' to its control character, except "\n" which is Magic.
#[inline]
const fn backslash_trans_impl(c: c_int) -> c_int {
    match c as u8 {
        b'r' => CAR,
        b't' => TAB,
        b'e' => ESC,
        b'b' => BS,
        _ => c,
    }
}

// =============================================================================
// Character Class Table
// =============================================================================

/// Initialize the character class table.
fn init_class_tab() -> [i16; 256] {
    let mut tab = [0i16; 256];

    for (i, entry) in tab.iter_mut().enumerate() {
        *entry = match i as u8 {
            b'0'..=b'7' => RI_DIGIT | RI_HEX | RI_OCTAL | RI_WORD,
            b'8'..=b'9' => RI_DIGIT | RI_HEX | RI_WORD,
            b'a'..=b'f' => RI_HEX | RI_WORD | RI_HEAD | RI_ALPHA | RI_LOWER,
            b'g'..=b'z' => RI_WORD | RI_HEAD | RI_ALPHA | RI_LOWER,
            b'A'..=b'F' => RI_HEX | RI_WORD | RI_HEAD | RI_ALPHA | RI_UPPER,
            b'G'..=b'Z' => RI_WORD | RI_HEAD | RI_ALPHA | RI_UPPER,
            b'_' => RI_WORD | RI_HEAD,
            b' ' | b'\t' => RI_WHITE,
            _ => 0,
        };
    }

    tab
}

/// Get a reference to the character class table (lazily initialized).
fn class_tab() -> &'static [i16; 256] {
    static CLASS_TAB: OnceLock<[i16; 256]> = OnceLock::new();
    CLASS_TAB.get_or_init(init_class_tab)
}

/// Check if character is a digit (0-9).
#[inline]
fn ri_digit(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_DIGIT) != 0
}

/// Check if character is a hexadecimal digit (0-9, a-f, A-F).
#[inline]
fn ri_hex(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_HEX) != 0
}

/// Check if character is an octal digit (0-7).
#[inline]
fn ri_octal(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_OCTAL) != 0
}

/// Check if character is a word character (a-z, A-Z, 0-9, _).
#[inline]
fn ri_word(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_WORD) != 0
}

/// Check if character can start an identifier (a-z, A-Z, _).
#[inline]
fn ri_head(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_HEAD) != 0
}

/// Check if character is alphabetic (a-z, A-Z).
#[inline]
fn ri_alpha(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_ALPHA) != 0
}

/// Check if character is lowercase (a-z).
#[inline]
fn ri_lower(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_LOWER) != 0
}

/// Check if character is uppercase (A-Z).
#[inline]
fn ri_upper(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_UPPER) != 0
}

/// Check if character is whitespace (space or tab).
#[inline]
fn ri_white(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_WHITE) != 0
}

// =============================================================================
// Query Functions
// =============================================================================

/// Check if compiled regex can match a newline.
///
/// # Safety
/// The handle must point to a valid regprog_T.
#[inline]
unsafe fn re_multiline_impl(prog: RegprogHandle) -> bool {
    let flags = nvim_regprog_get_regflags(prog);
    (flags as u32 & RF_HASNL) != 0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Remove magic from a character.
#[no_mangle]
pub extern "C" fn rs_no_magic(x: c_int) -> c_int {
    no_magic_impl(x)
}

/// Toggle the magic state of a character.
#[no_mangle]
pub extern "C" fn rs_toggle_magic(x: c_int) -> c_int {
    toggle_magic_impl(x)
}

/// Return the type of multi operator.
#[no_mangle]
pub extern "C" fn rs_re_multi_type(c: c_int) -> c_int {
    re_multi_type_impl(c)
}

/// Translate backslash escape to control character.
#[no_mangle]
pub extern "C" fn rs_backslash_trans(c: c_int) -> c_int {
    backslash_trans_impl(c)
}

/// Check if character is a digit.
#[no_mangle]
pub extern "C" fn rs_ri_digit(c: c_int) -> c_int {
    c_int::from(ri_digit(c))
}

/// Check if character is a hex digit.
#[no_mangle]
pub extern "C" fn rs_ri_hex(c: c_int) -> c_int {
    c_int::from(ri_hex(c))
}

/// Check if character is an octal digit.
#[no_mangle]
pub extern "C" fn rs_ri_octal(c: c_int) -> c_int {
    c_int::from(ri_octal(c))
}

/// Check if character is a word character.
#[no_mangle]
pub extern "C" fn rs_ri_word(c: c_int) -> c_int {
    c_int::from(ri_word(c))
}

/// Check if character can start an identifier.
#[no_mangle]
pub extern "C" fn rs_ri_head(c: c_int) -> c_int {
    c_int::from(ri_head(c))
}

/// Check if character is alphabetic.
#[no_mangle]
pub extern "C" fn rs_ri_alpha(c: c_int) -> c_int {
    c_int::from(ri_alpha(c))
}

/// Check if character is lowercase.
#[no_mangle]
pub extern "C" fn rs_ri_lower(c: c_int) -> c_int {
    c_int::from(ri_lower(c))
}

/// Check if character is uppercase.
#[no_mangle]
pub extern "C" fn rs_ri_upper(c: c_int) -> c_int {
    c_int::from(ri_upper(c))
}

/// Check if character is whitespace.
#[no_mangle]
pub extern "C" fn rs_ri_white(c: c_int) -> c_int {
    c_int::from(ri_white(c))
}

/// Check if compiled regex can match a newline.
///
/// # Safety
/// The handle must point to a valid regprog_T.
#[no_mangle]
pub unsafe extern "C" fn rs_re_multiline(prog: RegprogHandle) -> c_int {
    c_int::from(re_multiline_impl(prog))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_functions() {
        // Test is_magic
        assert!(is_magic(-1));
        assert!(is_magic(-100));
        assert!(!is_magic(0));
        assert!(!is_magic(100));

        // Test magic/un_magic round-trip
        assert_eq!(un_magic(magic(65)), 65); // 'A'
        assert_eq!(un_magic(magic(42)), 42); // '*'
    }

    #[test]
    fn test_no_magic() {
        // Magic character should be unmagicked
        assert_eq!(no_magic_impl(magic(b'*' as c_int)), b'*' as c_int);
        // Non-magic should stay the same
        assert_eq!(no_magic_impl(b'a' as c_int), b'a' as c_int);
    }

    #[test]
    fn test_toggle_magic() {
        let star = b'*' as c_int;
        let magic_star = magic(star);

        // Toggle regular -> magic
        assert_eq!(toggle_magic_impl(star), magic_star);
        // Toggle magic -> regular
        assert_eq!(toggle_magic_impl(magic_star), star);
    }

    #[test]
    fn test_re_multi_type() {
        // Single multi operators
        assert_eq!(re_multi_type_impl(magic(b'@' as c_int)), MULTI_ONE);
        assert_eq!(re_multi_type_impl(magic(b'=' as c_int)), MULTI_ONE);
        assert_eq!(re_multi_type_impl(magic(b'?' as c_int)), MULTI_ONE);

        // Multi multi operators
        assert_eq!(re_multi_type_impl(magic(b'*' as c_int)), MULTI_MULT);
        assert_eq!(re_multi_type_impl(magic(b'+' as c_int)), MULTI_MULT);
        assert_eq!(re_multi_type_impl(magic(b'{' as c_int)), MULTI_MULT);

        // Non-multi
        assert_eq!(re_multi_type_impl(magic(b'a' as c_int)), NOT_MULTI);
        assert_eq!(re_multi_type_impl(b'*' as c_int), NOT_MULTI); // non-magic star
    }

    #[test]
    fn test_backslash_trans() {
        assert_eq!(backslash_trans_impl(b'r' as c_int), CAR);
        assert_eq!(backslash_trans_impl(b't' as c_int), TAB);
        assert_eq!(backslash_trans_impl(b'e' as c_int), ESC);
        assert_eq!(backslash_trans_impl(b'b' as c_int), BS);
        // Other characters should pass through
        assert_eq!(backslash_trans_impl(b'n' as c_int), b'n' as c_int);
        assert_eq!(backslash_trans_impl(b'x' as c_int), b'x' as c_int);
    }

    #[test]
    fn test_class_tab() {
        // Test digit detection
        assert!(ri_digit(b'0' as c_int));
        assert!(ri_digit(b'5' as c_int));
        assert!(ri_digit(b'9' as c_int));
        assert!(!ri_digit(b'a' as c_int));

        // Test hex detection
        assert!(ri_hex(b'0' as c_int));
        assert!(ri_hex(b'a' as c_int));
        assert!(ri_hex(b'F' as c_int));
        assert!(!ri_hex(b'g' as c_int));
        assert!(!ri_hex(b'G' as c_int));

        // Test octal detection
        assert!(ri_octal(b'0' as c_int));
        assert!(ri_octal(b'7' as c_int));
        assert!(!ri_octal(b'8' as c_int));

        // Test word detection
        assert!(ri_word(b'a' as c_int));
        assert!(ri_word(b'Z' as c_int));
        assert!(ri_word(b'5' as c_int));
        assert!(ri_word(b'_' as c_int));
        assert!(!ri_word(b'-' as c_int));

        // Test head detection
        assert!(ri_head(b'a' as c_int));
        assert!(ri_head(b'_' as c_int));
        assert!(!ri_head(b'0' as c_int));

        // Test alpha detection
        assert!(ri_alpha(b'a' as c_int));
        assert!(ri_alpha(b'Z' as c_int));
        assert!(!ri_alpha(b'0' as c_int));
        assert!(!ri_alpha(b'_' as c_int));

        // Test case detection
        assert!(ri_lower(b'a' as c_int));
        assert!(!ri_lower(b'A' as c_int));
        assert!(ri_upper(b'A' as c_int));
        assert!(!ri_upper(b'a' as c_int));

        // Test whitespace
        assert!(ri_white(b' ' as c_int));
        assert!(ri_white(b'\t' as c_int));
        assert!(!ri_white(b'\n' as c_int));

        // Test out of range
        assert!(!ri_digit(256));
        assert!(!ri_digit(-1));
    }
}
