//! Parser helpers for the regex engine.
//!
//! This module provides parsing utilities for regex quantifiers and atoms.

use std::ffi::{c_char, c_int};

use crate::scanner::skipchr;

// =============================================================================
// Constants
// =============================================================================

/// Maximum limit for quantifiers (32767 << 16)
pub const MAX_LIMIT: c_int = 32767 << 16;

/// Return codes
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Parse state accessors
    fn nvim_parse_get_regparse() -> *mut c_char;
    fn nvim_parse_set_regparse(p: *mut c_char);
    fn nvim_parse_get_reg_magic() -> c_int;

    // Digit parsing
    fn rs_getdigits_int(pp: *mut *mut c_char, strict: c_int, def: c_int) -> c_int;

    // Error reporting - sets rc_did_emsg and reports error
    fn nvim_regexp_report_error(error_id: c_int, is_magic_all: c_int);
}

// Magic constant for very magic mode check
const MAGIC_ALL: c_int = 4;

// Error IDs for nvim_regexp_report_error
const ERR_SYNTAX_IN_BRACES: c_int = 554; // E554: Syntax error in %s{...}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a byte is an ASCII digit.
#[inline]
fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

// =============================================================================
// Parser Implementation
// =============================================================================

/// Read the limits for a `\{n,m}` quantifier.
///
/// Parses limit specifications like:
/// - `\{n}` - exactly n
/// - `\{n,}` - at least n
/// - `\{n,m}` - between n and m
/// - `\{,m}` - at most m
/// - `\{-n,m}` - non-greedy variant
///
/// Returns OK on success, FAIL on syntax error.
///
/// # Safety
/// regparse must point to a valid null-terminated string within the `{...}`.
pub unsafe fn read_limits(minval: &mut c_int, maxval: &mut c_int) -> c_int {
    let mut reverse = false;

    // Check for '-' at start (non-greedy)
    let mut regparse = nvim_parse_get_regparse();
    if *regparse as u8 == b'-' {
        regparse = regparse.add(1);
        nvim_parse_set_regparse(regparse);
        reverse = true;
    }

    let first_char = nvim_parse_get_regparse();
    let first_byte = *first_char as u8;

    // Parse minimum value
    let mut temp_regparse = nvim_parse_get_regparse();
    *minval = rs_getdigits_int(&mut temp_regparse, 0, 0);
    nvim_parse_set_regparse(temp_regparse);

    regparse = nvim_parse_get_regparse();

    if *regparse as u8 == b',' {
        // There is a comma
        regparse = regparse.add(1);
        nvim_parse_set_regparse(regparse);

        regparse = nvim_parse_get_regparse();
        if ascii_isdigit(*regparse as u8) {
            temp_regparse = regparse;
            *maxval = rs_getdigits_int(&mut temp_regparse, 0, MAX_LIMIT);
            nvim_parse_set_regparse(temp_regparse);
        } else {
            *maxval = MAX_LIMIT;
        }
    } else if ascii_isdigit(first_byte) {
        // It was \{n} or \{-n}
        *maxval = *minval;
    } else {
        // It was \{} or \{-}
        *maxval = MAX_LIMIT;
    }

    regparse = nvim_parse_get_regparse();

    // Allow either \{...} or \{...\}
    if *regparse as u8 == b'\\' {
        regparse = regparse.add(1);
        nvim_parse_set_regparse(regparse);
    }

    regparse = nvim_parse_get_regparse();
    if *regparse as u8 != b'}' {
        let reg_magic = nvim_parse_get_reg_magic();
        nvim_regexp_report_error(ERR_SYNTAX_IN_BRACES, (reg_magic == MAGIC_ALL) as c_int);
        return FAIL;
    }

    // Reverse the range if there was a '-', or make sure it is in the right
    // order otherwise.
    if (!reverse && *minval > *maxval) || (reverse && *minval < *maxval) {
        std::mem::swap(minval, maxval);
    }

    skipchr(); // let's be friends with the lexer again
    OK
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Read the limits for a `\{n,m}` quantifier.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
/// minval and maxval must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_read_limits(minval: *mut c_int, maxval: *mut c_int) -> c_int {
    read_limits(&mut *minval, &mut *maxval)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_limit() {
        assert_eq!(MAX_LIMIT, 32767 << 16);
        assert_eq!(MAX_LIMIT, 2147418112);
    }

    #[test]
    fn test_ascii_isdigit() {
        assert!(ascii_isdigit(b'0'));
        assert!(ascii_isdigit(b'5'));
        assert!(ascii_isdigit(b'9'));
        assert!(!ascii_isdigit(b'a'));
        assert!(!ascii_isdigit(b' '));
        assert!(!ascii_isdigit(b'-'));
    }
}
