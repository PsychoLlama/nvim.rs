//! Parser helpers for the regex engine.
//!
//! This module provides parsing utilities for regex quantifiers and atoms.

use std::ffi::{c_char, c_int};

use crate::scanner::{peekchr, skipchr};
use crate::{re_multi_type_impl, NOT_MULTI};

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

    // UTF-8 functions (from mbyte crate)
    fn rs_utf_char2len(c: c_int) -> c_int;
    fn rs_utf_iscomposing_legacy(c: c_int) -> c_int;
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
// Bytecode Helpers
// =============================================================================

/// Write a four-byte value at the given position in big-endian format.
///
/// Returns a pointer to the position after the written bytes.
///
/// # Safety
/// `p` must point to a buffer with at least 4 bytes of writable space.
#[inline]
pub unsafe fn re_put_uint32(p: *mut u8, val: u32) -> *mut u8 {
    *p = ((val >> 24) & 0xff) as u8;
    *p.add(1) = ((val >> 16) & 0xff) as u8;
    *p.add(2) = ((val >> 8) & 0xff) as u8;
    *p.add(3) = (val & 0xff) as u8;
    p.add(4)
}

/// Read a four-byte value from the given position in big-endian format.
///
/// This is the read counterpart to `re_put_uint32`.
///
/// # Safety
/// `p` must point to a buffer with at least 4 readable bytes.
#[inline]
pub unsafe fn re_get_uint32(p: *const u8) -> u32 {
    ((*p as u32) << 24)
        | ((*p.add(1) as u32) << 16)
        | ((*p.add(2) as u32) << 8)
        | (*p.add(3) as u32)
}

/// Write a two-byte value at the given position in big-endian format.
///
/// Returns a pointer to the position after the written bytes.
/// Used for BT engine "next" pointers.
///
/// # Safety
/// `p` must point to a buffer with at least 2 bytes of writable space.
#[inline]
pub unsafe fn re_put_uint16(p: *mut u8, val: u16) -> *mut u8 {
    *p = ((val >> 8) & 0xff) as u8;
    *p.add(1) = (val & 0xff) as u8;
    p.add(2)
}

/// Read a two-byte value from the given position in big-endian format.
///
/// This is used to read BT engine "next" pointers.
/// Note: The C macro `NEXT(p)` reads from p+1 and p+2, but this
/// function reads from p+0 and p+1 for consistency with put.
///
/// # Safety
/// `p` must point to a buffer with at least 2 readable bytes.
#[inline]
pub unsafe fn re_get_uint16(p: *const u8) -> u16 {
    ((*p as u16) << 8) | (*p.add(1) as u16)
}

// =============================================================================
// BT Engine Numeric Comparison
// =============================================================================

/// Compare a number with the operand from a BT regex instruction.
///
/// The bytecode layout at `scan` is:
/// - Bytes 0-2: opcode and next pointer
/// - Bytes 3-6: comparison value (big-endian)
/// - Byte 7: comparison operator ('>', '<', or other for '=')
///
/// Used for RE_LNUM, RE_COL, RE_VCOL patterns like `\%>23l`.
///
/// # Safety
/// `scan` must point to a valid bytecode instruction with at least 8 bytes.
#[inline]
pub unsafe fn re_num_cmp(val: u32, scan: *const u8) -> bool {
    let n = re_get_uint32(scan.add(3));
    let op = *scan.add(7);

    if op == b'>' {
        val > n
    } else if op == b'<' {
        val < n
    } else {
        val == n
    }
}

// =============================================================================
// NFA Numeric Comparison
// =============================================================================

/// Comparison operators for NFA numeric matching.
pub mod cmp_op {
    /// Greater than (pos > val)
    pub const GREATER: i32 = 1;
    /// Less than (pos < val)
    pub const LESS: i32 = 2;
    // 0 or any other value = equals (val == pos)
}

/// Compare numbers for NFA regex position matching.
///
/// Used to check if current position (line, column, vcol) matches a pattern
/// constraint like `\%>23l` (line > 23) or `\%<10c` (column < 10).
///
/// # Arguments
/// * `val` - The value from the pattern (e.g., 23 in `\%>23l`)
/// * `op` - Comparison operator: 1=greater, 2=less, other=equals
/// * `pos` - The current position to compare against
#[inline]
pub const fn nfa_re_num_cmp(val: u64, op: c_int, pos: u64) -> bool {
    if op == cmp_op::GREATER {
        pos > val
    } else if op == cmp_op::LESS {
        pos < val
    } else {
        val == pos
    }
}

// =============================================================================
// Multi-byte Code Decision
// =============================================================================

/// Return true if MULTIBYTECODE should be used instead of EXACTLY for
/// character "c".
///
/// Uses MULTIBYTECODE when:
/// - The character takes more than 1 byte in UTF-8, AND
/// - Either it's followed by a multi operator, OR it's a composing character
///
/// # Safety
/// Calls peekchr() which accesses global parse state.
#[inline]
pub unsafe fn use_multibytecode(c: c_int) -> bool {
    rs_utf_char2len(c) > 1
        && (re_multi_type_impl(peekchr()) != NOT_MULTI || rs_utf_iscomposing_legacy(c) != 0)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Return true if MULTIBYTECODE should be used instead of EXACTLY.
///
/// # Safety
/// Calls peekchr() which accesses global parse state.
#[no_mangle]
pub unsafe extern "C" fn rs_use_multibytecode(c: c_int) -> c_int {
    c_int::from(use_multibytecode(c))
}

/// Read the limits for a `\{n,m}` quantifier.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
/// minval and maxval must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_read_limits(minval: *mut c_int, maxval: *mut c_int) -> c_int {
    read_limits(&mut *minval, &mut *maxval)
}

/// Compare numbers for NFA regex position matching.
///
/// Used to check if current position matches pattern constraints like
/// `\%>23l` (line > 23) or `\%<10c` (column < 10).
#[no_mangle]
pub extern "C" fn rs_nfa_re_num_cmp(val: u64, op: c_int, pos: u64) -> c_int {
    c_int::from(nfa_re_num_cmp(val, op, pos))
}

/// Write a four-byte value in big-endian format.
///
/// # Safety
/// `p` must point to a buffer with at least 4 bytes of writable space.
#[no_mangle]
pub unsafe extern "C" fn rs_re_put_uint32(p: *mut u8, val: u32) -> *mut u8 {
    re_put_uint32(p, val)
}

/// Read a four-byte value in big-endian format.
///
/// # Safety
/// `p` must point to a buffer with at least 4 readable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_re_get_uint32(p: *const u8) -> u32 {
    re_get_uint32(p)
}

/// Compare a number with a BT regex bytecode operand.
///
/// # Safety
/// `scan` must point to a valid bytecode instruction with at least 8 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_re_num_cmp(val: u32, scan: *const u8) -> c_int {
    c_int::from(re_num_cmp(val, scan))
}

/// Write a two-byte value in big-endian format.
///
/// # Safety
/// `p` must point to a buffer with at least 2 bytes of writable space.
#[no_mangle]
pub unsafe extern "C" fn rs_re_put_uint16(p: *mut u8, val: u16) -> *mut u8 {
    re_put_uint16(p, val)
}

/// Read a two-byte value in big-endian format.
///
/// # Safety
/// `p` must point to a buffer with at least 2 readable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_re_get_uint16(p: *const u8) -> u16 {
    re_get_uint16(p)
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

    // Note: use_multibytecode tests require FFI functions that are only
    // available when linked with the full Neovim binary.

    #[test]
    fn test_nfa_re_num_cmp_equal() {
        // Default (0) is equality comparison
        assert!(nfa_re_num_cmp(10, 0, 10));
        assert!(!nfa_re_num_cmp(10, 0, 11));
        assert!(!nfa_re_num_cmp(10, 0, 9));
    }

    #[test]
    fn test_nfa_re_num_cmp_greater() {
        // op=1: pos > val
        assert!(nfa_re_num_cmp(10, cmp_op::GREATER, 11));
        assert!(nfa_re_num_cmp(10, cmp_op::GREATER, 100));
        assert!(!nfa_re_num_cmp(10, cmp_op::GREATER, 10));
        assert!(!nfa_re_num_cmp(10, cmp_op::GREATER, 5));
    }

    #[test]
    fn test_nfa_re_num_cmp_less() {
        // op=2: pos < val
        assert!(nfa_re_num_cmp(10, cmp_op::LESS, 5));
        assert!(nfa_re_num_cmp(10, cmp_op::LESS, 0));
        assert!(!nfa_re_num_cmp(10, cmp_op::LESS, 10));
        assert!(!nfa_re_num_cmp(10, cmp_op::LESS, 15));
    }

    #[test]
    fn test_re_put_uint32() {
        let mut buf = [0u8; 8];
        unsafe {
            // Test writing 0x12345678
            let next = re_put_uint32(buf.as_mut_ptr(), 0x12345678);
            assert_eq!(buf[0], 0x12); // high byte first (big-endian)
            assert_eq!(buf[1], 0x34);
            assert_eq!(buf[2], 0x56);
            assert_eq!(buf[3], 0x78);
            assert_eq!(next, buf.as_mut_ptr().add(4));

            // Test writing 0 at offset 4
            let next2 = re_put_uint32(next, 0x00000000);
            assert_eq!(buf[4], 0x00);
            assert_eq!(buf[5], 0x00);
            assert_eq!(buf[6], 0x00);
            assert_eq!(buf[7], 0x00);
            assert_eq!(next2, buf.as_mut_ptr().add(8));

            // Test writing max value
            let mut buf2 = [0u8; 4];
            re_put_uint32(buf2.as_mut_ptr(), 0xFFFFFFFF);
            assert_eq!(buf2, [0xFF, 0xFF, 0xFF, 0xFF]);
        }
    }

    #[test]
    fn test_re_get_uint32() {
        unsafe {
            // Test reading 0x12345678
            let buf = [0x12u8, 0x34, 0x56, 0x78];
            assert_eq!(re_get_uint32(buf.as_ptr()), 0x12345678);

            // Test reading 0
            let buf_zero = [0u8; 4];
            assert_eq!(re_get_uint32(buf_zero.as_ptr()), 0);

            // Test reading max value
            let buf_max = [0xFF, 0xFF, 0xFF, 0xFF];
            assert_eq!(re_get_uint32(buf_max.as_ptr()), 0xFFFFFFFF);

            // Test reading 0x00FF00FF
            let buf_alt = [0x00, 0xFF, 0x00, 0xFF];
            assert_eq!(re_get_uint32(buf_alt.as_ptr()), 0x00FF00FF);
        }
    }

    #[test]
    fn test_re_put_get_roundtrip() {
        // Test that put and get are inverses
        unsafe {
            let values = [0u32, 1, 255, 256, 65535, 0x12345678, 0xFFFFFFFF];
            let mut buf = [0u8; 4];

            for &val in &values {
                re_put_uint32(buf.as_mut_ptr(), val);
                assert_eq!(
                    re_get_uint32(buf.as_ptr()),
                    val,
                    "Roundtrip failed for {val:#x}"
                );
            }
        }
    }

    #[test]
    fn test_re_num_cmp() {
        // Create a mock bytecode instruction:
        // - Bytes 0-2: opcode and next (unused by re_num_cmp)
        // - Bytes 3-6: comparison value (big-endian)
        // - Byte 7: comparison operator
        unsafe {
            // Test greater than: val > n
            let mut buf = [0u8; 8];
            re_put_uint32(buf.as_mut_ptr().add(3), 10); // n = 10
            buf[7] = b'>'; // operator = >

            assert!(re_num_cmp(11, buf.as_ptr())); // 11 > 10
            assert!(re_num_cmp(100, buf.as_ptr())); // 100 > 10
            assert!(!re_num_cmp(10, buf.as_ptr())); // 10 > 10 = false
            assert!(!re_num_cmp(5, buf.as_ptr())); // 5 > 10 = false

            // Test less than: val < n
            buf[7] = b'<';
            assert!(re_num_cmp(5, buf.as_ptr())); // 5 < 10
            assert!(re_num_cmp(0, buf.as_ptr())); // 0 < 10
            assert!(!re_num_cmp(10, buf.as_ptr())); // 10 < 10 = false
            assert!(!re_num_cmp(15, buf.as_ptr())); // 15 < 10 = false

            // Test equals: val == n (any other operator)
            buf[7] = b'='; // explicit equals
            assert!(re_num_cmp(10, buf.as_ptr())); // 10 == 10
            assert!(!re_num_cmp(11, buf.as_ptr())); // 11 == 10 = false
            assert!(!re_num_cmp(9, buf.as_ptr())); // 9 == 10 = false

            // Default operator (0) also means equals
            buf[7] = 0;
            assert!(re_num_cmp(10, buf.as_ptr())); // 10 == 10
            assert!(!re_num_cmp(11, buf.as_ptr())); // 11 == 10 = false
        }
    }

    #[test]
    fn test_re_put_uint16() {
        let mut buf = [0u8; 4];
        unsafe {
            // Test writing 0x1234
            let next = re_put_uint16(buf.as_mut_ptr(), 0x1234);
            assert_eq!(buf[0], 0x12); // high byte first (big-endian)
            assert_eq!(buf[1], 0x34);
            assert_eq!(next, buf.as_mut_ptr().add(2));

            // Test writing 0 at offset 2
            let next2 = re_put_uint16(next, 0x0000);
            assert_eq!(buf[2], 0x00);
            assert_eq!(buf[3], 0x00);
            assert_eq!(next2, buf.as_mut_ptr().add(4));

            // Test writing max value
            let mut buf2 = [0u8; 2];
            re_put_uint16(buf2.as_mut_ptr(), 0xFFFF);
            assert_eq!(buf2, [0xFF, 0xFF]);
        }
    }

    #[test]
    fn test_re_get_uint16() {
        unsafe {
            // Test reading 0x1234
            let buf = [0x12u8, 0x34];
            assert_eq!(re_get_uint16(buf.as_ptr()), 0x1234);

            // Test reading 0
            let buf_zero = [0u8; 2];
            assert_eq!(re_get_uint16(buf_zero.as_ptr()), 0);

            // Test reading max value
            let buf_max = [0xFF, 0xFF];
            assert_eq!(re_get_uint16(buf_max.as_ptr()), 0xFFFF);

            // Test reading 0x00FF
            let buf_alt = [0x00, 0xFF];
            assert_eq!(re_get_uint16(buf_alt.as_ptr()), 0x00FF);
        }
    }

    #[test]
    fn test_re_put_get_uint16_roundtrip() {
        // Test that put and get are inverses
        unsafe {
            let values = [0u16, 1, 255, 256, 0x1234, 0xFFFF];
            let mut buf = [0u8; 2];

            for &val in &values {
                re_put_uint16(buf.as_mut_ptr(), val);
                assert_eq!(
                    re_get_uint16(buf.as_ptr()),
                    val,
                    "Roundtrip failed for {val:#x}"
                );
            }
        }
    }
}
