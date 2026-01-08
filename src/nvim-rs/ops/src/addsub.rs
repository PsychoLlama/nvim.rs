//! Number increment/decrement operations (Ctrl-A, Ctrl-X)
//!
//! This module implements the logic for incrementing and decrementing
//! numbers and alphabetic characters with Ctrl-A and Ctrl-X.
//!
//! Supports various number formats based on 'nrformats' option:
//! - Decimal (default)
//! - Hexadecimal (0x/0X prefix)
//! - Octal (0o/0O prefix or leading 0)
//! - Binary (0b/0B prefix)
//! - Alphabetic characters (a-z, A-Z)
//! - Unsigned (no negative sign handling)

use std::ffi::c_int;

use crate::types::OpType;

// =============================================================================
// Number Format Detection
// =============================================================================

/// Number format prefixes recognized by do_addsub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NumberPrefix {
    /// Decimal number (no prefix)
    Decimal = 0,
    /// Hexadecimal (0x or 0X)
    Hex = b'x',
    /// Octal (0o, 0O, or leading 0)
    Octal = b'o',
    /// Binary (0b or 0B)
    Binary = b'b',
}

impl NumberPrefix {
    /// Create from prefix character.
    #[must_use]
    pub const fn from_char(c: u8) -> Option<Self> {
        match c {
            b'x' | b'X' => Some(Self::Hex),
            b'o' | b'O' => Some(Self::Octal),
            b'b' | b'B' => Some(Self::Binary),
            _ => None,
        }
    }

    /// Check if the character is a valid prefix character.
    #[must_use]
    #[inline]
    pub const fn is_prefix_char(c: u8) -> bool {
        matches!(c, b'x' | b'X' | b'o' | b'O' | b'b' | b'B')
    }

    /// Get the base for this number format.
    #[must_use]
    pub const fn base(self) -> u32 {
        match self {
            Self::Decimal => 10,
            Self::Hex => 16,
            Self::Octal => 8,
            Self::Binary => 2,
        }
    }
}

/// Check if a character is a valid digit for the given base.
#[must_use]
#[inline]
pub const fn is_valid_digit(c: u8, base: u32) -> bool {
    match base {
        2 => c == b'0' || c == b'1',
        8 => c >= b'0' && c <= b'7',
        10 => c >= b'0' && c <= b'9',
        16 => (c >= b'0' && c <= b'9') || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F'),
        _ => false,
    }
}

/// Check if a character is a binary digit (0 or 1).
#[must_use]
#[inline]
pub const fn is_binary_digit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Check if a character is a hexadecimal digit.
#[must_use]
#[inline]
pub const fn is_hex_digit(c: u8) -> bool {
    (c >= b'0' && c <= b'9') || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')
}

/// Increment or decrement an alphabetic character.
///
/// If the character would wrap past 'Z'/'z' or before 'A'/'a', it clamps
/// to the boundary.
///
/// # Arguments
/// * `c` - The character to modify (must be ASCII alphabetic)
/// * `op_type` - `OpType::NrAdd` for increment, `OpType::NrSub` for decrement
/// * `amount` - The amount to add or subtract
///
/// # Returns
/// The new character value, or the original if not alphabetic
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn addsub_alpha(c: c_int, op_type: OpType, amount: i64) -> c_int {
    // Only handle ASCII range
    if !(0..=127).contains(&c) {
        return c;
    }

    let c_byte = c as u8;

    // Check if it's an ASCII letter
    if !c_byte.is_ascii_alphabetic() {
        return c;
    }

    let is_upper = c_byte.is_ascii_uppercase();
    let base = if is_upper { b'A' } else { b'a' };
    let ord = i64::from(c_byte - base); // 0-25

    let new_ord = if op_type == OpType::NrSub {
        // Decrement - clamp at 0 (A/a)
        if ord < amount {
            0
        } else {
            ord - amount
        }
    } else {
        // Increment - clamp at 25 (Z/z)
        let max_add = 25 - ord;
        if amount > max_add {
            25
        } else {
            ord + amount
        }
    };

    c_int::from(base + new_ord as u8)
}

/// FFI wrapper for addsub_alpha.
#[no_mangle]
pub extern "C" fn rs_addsub_alpha(c: c_int, op_type: c_int, amount: i64) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::NrAdd);
    addsub_alpha(c, op, amount)
}

/// Handle wraparound for unsigned number arithmetic.
///
/// When subtracting causes underflow or adding causes overflow, this
/// computes the proper result accounting for wraparound behavior.
///
/// # Arguments
/// * `n` - The current value
/// * `amount` - The amount to add (positive) or subtract (negative via subtract flag)
/// * `subtract` - true if subtracting, false if adding
///
/// # Returns
/// A tuple of (new_value, is_negative) where is_negative indicates if the
/// result should be displayed as a negative number.
#[must_use]
pub fn handle_wraparound(n: u64, amount: u64, subtract: bool) -> (u64, bool) {
    if subtract {
        if amount > n {
            // Underflow - compute magnitude and flip sign
            let diff = amount - n;
            (diff, true)
        } else {
            (n - amount, false)
        }
    } else {
        let result = n.wrapping_add(amount);
        if result < n {
            // Overflow - compute wrapped value and flip sign
            (!result, true)
        } else {
            (result, false)
        }
    }
}

/// FFI wrapper for handle_wraparound.
///
/// # Safety
/// `out_negative` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_wraparound(
    n: u64,
    amount: u64,
    subtract: c_int,
    out_negative: *mut c_int,
) -> u64 {
    let (result, negative) = handle_wraparound(n, amount, subtract != 0);
    if !out_negative.is_null() {
        *out_negative = c_int::from(negative);
    }
    result
}

// =============================================================================
// Number Formatting Helpers
// =============================================================================

/// Determine if the operation should subtract based on op_type and sign.
///
/// The XOR logic: subtract if (op_type is OP_NR_SUB) XOR (number is negative)
#[must_use]
#[inline]
pub const fn should_subtract(op_is_sub: bool, is_negative: bool) -> bool {
    op_is_sub ^ is_negative
}

/// Calculate the minimum width needed for formatted number output.
///
/// Returns the number of digits needed to represent the value in the given base.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn calc_min_width(value: u64, base: u32) -> usize {
    if value == 0 {
        return 1;
    }

    let mut n = value;
    let mut width = 0;
    let base64 = base as u64;
    while n > 0 {
        n /= base64;
        width += 1;
    }
    width
}

/// Calculate the number of leading zeros needed to maintain field width.
///
/// When incrementing/decrementing a number like "007", we want to preserve
/// the leading zeros.
///
/// # Arguments
/// * `original_len` - Original string length of the number (including prefix)
/// * `new_value_width` - Number of digits needed for the new value
/// * `has_prefix` - Whether the number has a prefix (0x, 0b, 0o)
/// * `has_sign` - Whether the number has a sign character
///
/// # Returns
/// Number of leading zeros to add
#[must_use]
#[inline]
pub const fn calc_leading_zeros(
    original_len: usize,
    new_value_width: usize,
    has_prefix: bool,
    has_sign: bool,
) -> usize {
    let prefix_len = if has_prefix { 2 } else { 0 };
    let sign_len = if has_sign { 1 } else { 0 };
    let overhead = prefix_len + sign_len;

    if original_len <= overhead + new_value_width {
        return 0;
    }

    original_len - overhead - new_value_width
}

/// Check if hex number should use uppercase letters based on original.
///
/// If the original number had an uppercase 'X' prefix, use uppercase hex digits.
#[must_use]
#[inline]
pub const fn should_use_uppercase_hex(prefix_char: u8) -> bool {
    prefix_char == b'X'
}

/// Check if the position is past the line end.
///
/// # Arguments
/// * `col` - Current column
/// * `coladd` - Virtual column offset
/// * `linelen` - Length of the line
///
/// # Returns
/// true if the position is past the end of the line
#[must_use]
#[inline]
pub const fn is_past_line_end(col: c_int, coladd: c_int, linelen: c_int) -> bool {
    col + (coladd != 0) as c_int >= linelen
}

/// Check if character at position could be part of a hex number.
///
/// Looks for the pattern: 0[xX] followed by hex digit
#[must_use]
#[inline]
pub const fn is_hex_prefix_pattern(prev_char: u8, curr_char: u8, next_char: u8) -> bool {
    prev_char == b'0' && (curr_char == b'x' || curr_char == b'X') && is_hex_digit(next_char)
}

/// Check if character at position could be part of a binary number.
///
/// Looks for the pattern: 0[bB] followed by binary digit
#[must_use]
#[inline]
pub const fn is_binary_prefix_pattern(prev_char: u8, curr_char: u8, next_char: u8) -> bool {
    prev_char == b'0' && (curr_char == b'b' || curr_char == b'B') && is_binary_digit(next_char)
}

/// Determine if a minus sign should be treated as part of the number.
///
/// Returns `(treat_as_negative, treat_as_unsigned)`.
#[must_use]
#[inline]
#[allow(clippy::fn_params_excessive_bools)]
pub const fn check_minus_handling(
    has_minus: bool,
    do_unsigned: bool,
    do_blank: bool,
    char_before_minus_is_blank: bool,
) -> (bool, bool) {
    if !has_minus {
        return (false, false);
    }
    if do_unsigned {
        return (false, false);
    }
    if do_blank && !char_before_minus_is_blank {
        // Blank mode: treat as unsigned if non-blank before minus
        return (false, true);
    }
    (true, false)
}

// =============================================================================
// FFI Wrappers for New Functions
// =============================================================================

/// FFI wrapper for is_binary_digit.
#[no_mangle]
pub extern "C" fn rs_is_binary_digit(c: u8) -> c_int {
    c_int::from(is_binary_digit(c))
}

/// FFI wrapper for is_hex_digit.
#[no_mangle]
pub extern "C" fn rs_is_hex_digit(c: u8) -> c_int {
    c_int::from(is_hex_digit(c))
}

/// FFI wrapper for is_valid_digit.
#[no_mangle]
pub extern "C" fn rs_is_valid_digit(c: u8, base: u32) -> c_int {
    c_int::from(is_valid_digit(c, base))
}

/// FFI wrapper for should_subtract.
#[no_mangle]
pub extern "C" fn rs_should_subtract(op_is_sub: c_int, is_negative: c_int) -> c_int {
    c_int::from(should_subtract(op_is_sub != 0, is_negative != 0))
}

/// FFI wrapper for calc_min_width.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub extern "C" fn rs_calc_min_width(value: u64, base: u32) -> c_int {
    calc_min_width(value, base) as c_int
}

/// FFI wrapper for calc_leading_zeros.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub extern "C" fn rs_calc_leading_zeros(
    original_len: c_int,
    new_value_width: c_int,
    has_prefix: c_int,
    has_sign: c_int,
) -> c_int {
    calc_leading_zeros(
        original_len.max(0) as usize,
        new_value_width.max(0) as usize,
        has_prefix != 0,
        has_sign != 0,
    ) as c_int
}

/// FFI wrapper for should_use_uppercase_hex.
#[no_mangle]
pub extern "C" fn rs_should_use_uppercase_hex(prefix_char: u8) -> c_int {
    c_int::from(should_use_uppercase_hex(prefix_char))
}

/// FFI wrapper for is_past_line_end.
#[no_mangle]
pub extern "C" fn rs_is_past_line_end(col: c_int, coladd: c_int, linelen: c_int) -> c_int {
    c_int::from(is_past_line_end(col, coladd, linelen))
}

/// FFI wrapper for is_hex_prefix_pattern.
#[no_mangle]
pub extern "C" fn rs_is_hex_prefix_pattern(prev_char: u8, curr_char: u8, next_char: u8) -> c_int {
    c_int::from(is_hex_prefix_pattern(prev_char, curr_char, next_char))
}

/// FFI wrapper for is_binary_prefix_pattern.
#[no_mangle]
pub extern "C" fn rs_is_binary_prefix_pattern(
    prev_char: u8,
    curr_char: u8,
    next_char: u8,
) -> c_int {
    c_int::from(is_binary_prefix_pattern(prev_char, curr_char, next_char))
}

/// FFI wrapper for check_minus_handling.
///
/// # Safety
/// `out_treat_as_unsigned` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_check_minus_handling(
    has_minus: c_int,
    do_unsigned: c_int,
    do_blank: c_int,
    char_before_minus_is_blank: c_int,
    out_treat_as_unsigned: *mut c_int,
) -> c_int {
    let (negative, unsigned) = check_minus_handling(
        has_minus != 0,
        do_unsigned != 0,
        do_blank != 0,
        char_before_minus_is_blank != 0,
    );
    if !out_treat_as_unsigned.is_null() {
        *out_treat_as_unsigned = c_int::from(unsigned);
    }
    c_int::from(negative)
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_addsub_alpha_increment() {
        // Basic increment
        assert_eq!(addsub_alpha(b'a' as c_int, OpType::NrAdd, 1), b'b' as c_int);
        assert_eq!(addsub_alpha(b'A' as c_int, OpType::NrAdd, 1), b'B' as c_int);
        assert_eq!(addsub_alpha(b'm' as c_int, OpType::NrAdd, 5), b'r' as c_int);

        // Clamp at z/Z
        assert_eq!(addsub_alpha(b'y' as c_int, OpType::NrAdd, 5), b'z' as c_int);
        assert_eq!(addsub_alpha(b'z' as c_int, OpType::NrAdd, 1), b'z' as c_int);
        assert_eq!(
            addsub_alpha(b'Z' as c_int, OpType::NrAdd, 100),
            b'Z' as c_int
        );
    }

    #[test]
    fn test_addsub_alpha_decrement() {
        // Basic decrement
        assert_eq!(addsub_alpha(b'b' as c_int, OpType::NrSub, 1), b'a' as c_int);
        assert_eq!(addsub_alpha(b'B' as c_int, OpType::NrSub, 1), b'A' as c_int);
        assert_eq!(addsub_alpha(b'n' as c_int, OpType::NrSub, 5), b'i' as c_int);

        // Clamp at a/A
        assert_eq!(addsub_alpha(b'c' as c_int, OpType::NrSub, 5), b'a' as c_int);
        assert_eq!(addsub_alpha(b'a' as c_int, OpType::NrSub, 1), b'a' as c_int);
        assert_eq!(
            addsub_alpha(b'A' as c_int, OpType::NrSub, 100),
            b'A' as c_int
        );
    }

    #[test]
    fn test_addsub_alpha_non_alpha() {
        // Non-alphabetic characters should be unchanged
        assert_eq!(addsub_alpha(b'0' as c_int, OpType::NrAdd, 1), b'0' as c_int);
        assert_eq!(addsub_alpha(b'!' as c_int, OpType::NrSub, 1), b'!' as c_int);
        assert_eq!(addsub_alpha(b' ' as c_int, OpType::NrAdd, 1), b' ' as c_int);
    }

    #[test]
    fn test_handle_wraparound_no_wrap() {
        // Normal subtraction
        assert_eq!(handle_wraparound(10, 5, true), (5, false));
        // Normal addition
        assert_eq!(handle_wraparound(10, 5, false), (15, false));
    }

    #[test]
    fn test_handle_wraparound_underflow() {
        // Subtracting more than the value
        assert_eq!(handle_wraparound(5, 10, true), (5, true));
        assert_eq!(handle_wraparound(0, 1, true), (1, true));
    }

    #[test]
    fn test_handle_wraparound_overflow() {
        // Adding causes overflow
        let max = u64::MAX;
        let (result, negative) = handle_wraparound(max, 1, false);
        // !0 = max, so result should be max
        assert!(negative);
        assert_eq!(result, max);
    }

    // =========================================================================
    // NumberPrefix Tests
    // =========================================================================

    #[test]
    fn test_number_prefix_from_char() {
        assert_eq!(NumberPrefix::from_char(b'x'), Some(NumberPrefix::Hex));
        assert_eq!(NumberPrefix::from_char(b'X'), Some(NumberPrefix::Hex));
        assert_eq!(NumberPrefix::from_char(b'o'), Some(NumberPrefix::Octal));
        assert_eq!(NumberPrefix::from_char(b'O'), Some(NumberPrefix::Octal));
        assert_eq!(NumberPrefix::from_char(b'b'), Some(NumberPrefix::Binary));
        assert_eq!(NumberPrefix::from_char(b'B'), Some(NumberPrefix::Binary));
        assert_eq!(NumberPrefix::from_char(b'0'), None);
        assert_eq!(NumberPrefix::from_char(b'a'), None);
    }

    #[test]
    fn test_number_prefix_base() {
        assert_eq!(NumberPrefix::Decimal.base(), 10);
        assert_eq!(NumberPrefix::Hex.base(), 16);
        assert_eq!(NumberPrefix::Octal.base(), 8);
        assert_eq!(NumberPrefix::Binary.base(), 2);
    }

    // =========================================================================
    // Digit Validation Tests
    // =========================================================================

    #[test]
    fn test_is_binary_digit() {
        assert!(is_binary_digit(b'0'));
        assert!(is_binary_digit(b'1'));
        assert!(!is_binary_digit(b'2'));
        assert!(!is_binary_digit(b'a'));
    }

    #[test]
    fn test_is_hex_digit() {
        for c in b'0'..=b'9' {
            assert!(is_hex_digit(c));
        }
        for c in b'a'..=b'f' {
            assert!(is_hex_digit(c));
        }
        for c in b'A'..=b'F' {
            assert!(is_hex_digit(c));
        }
        assert!(!is_hex_digit(b'g'));
        assert!(!is_hex_digit(b'G'));
        assert!(!is_hex_digit(b'x'));
    }

    #[test]
    fn test_is_valid_digit() {
        // Binary
        assert!(is_valid_digit(b'0', 2));
        assert!(is_valid_digit(b'1', 2));
        assert!(!is_valid_digit(b'2', 2));

        // Octal
        for c in b'0'..=b'7' {
            assert!(is_valid_digit(c, 8));
        }
        assert!(!is_valid_digit(b'8', 8));

        // Decimal
        for c in b'0'..=b'9' {
            assert!(is_valid_digit(c, 10));
        }
        assert!(!is_valid_digit(b'a', 10));

        // Hex
        for c in b'0'..=b'9' {
            assert!(is_valid_digit(c, 16));
        }
        assert!(is_valid_digit(b'f', 16));
        assert!(!is_valid_digit(b'g', 16));
    }

    // =========================================================================
    // should_subtract Tests
    // =========================================================================

    #[test]
    fn test_should_subtract() {
        // XOR logic
        assert!(!should_subtract(false, false)); // add positive
        assert!(should_subtract(false, true)); // add negative -> subtract
        assert!(should_subtract(true, false)); // subtract positive
        assert!(!should_subtract(true, true)); // subtract negative -> add
    }

    // =========================================================================
    // calc_min_width Tests
    // =========================================================================

    #[test]
    fn test_calc_min_width() {
        // Base 10
        assert_eq!(calc_min_width(0, 10), 1);
        assert_eq!(calc_min_width(9, 10), 1);
        assert_eq!(calc_min_width(10, 10), 2);
        assert_eq!(calc_min_width(99, 10), 2);
        assert_eq!(calc_min_width(100, 10), 3);

        // Base 16
        assert_eq!(calc_min_width(15, 16), 1);
        assert_eq!(calc_min_width(16, 16), 2);
        assert_eq!(calc_min_width(255, 16), 2);
        assert_eq!(calc_min_width(256, 16), 3);

        // Base 2
        assert_eq!(calc_min_width(1, 2), 1);
        assert_eq!(calc_min_width(2, 2), 2);
        assert_eq!(calc_min_width(7, 2), 3);
        assert_eq!(calc_min_width(8, 2), 4);
    }

    // =========================================================================
    // calc_leading_zeros Tests
    // =========================================================================

    #[test]
    fn test_calc_leading_zeros() {
        // "007" -> original_len=3, new_width=1, no prefix, no sign -> 2 zeros
        assert_eq!(calc_leading_zeros(3, 1, false, false), 2);

        // "0x00ff" -> original_len=6, new_width=2, has prefix -> 2 zeros
        assert_eq!(calc_leading_zeros(6, 2, true, false), 2);

        // "-007" -> original_len=4, new_width=1, no prefix, has sign -> 2 zeros
        assert_eq!(calc_leading_zeros(4, 1, false, true), 2);

        // "123" -> original_len=3, new_width=3 -> 0 zeros
        assert_eq!(calc_leading_zeros(3, 3, false, false), 0);

        // "12" -> original_len=2, new_width=3 -> no leading zeros (would overflow)
        assert_eq!(calc_leading_zeros(2, 3, false, false), 0);
    }

    // =========================================================================
    // should_use_uppercase_hex Tests
    // =========================================================================

    #[test]
    fn test_should_use_uppercase_hex() {
        assert!(should_use_uppercase_hex(b'X'));
        assert!(!should_use_uppercase_hex(b'x'));
        assert!(!should_use_uppercase_hex(b'0'));
    }

    // =========================================================================
    // is_past_line_end Tests
    // =========================================================================

    #[test]
    fn test_is_past_line_end() {
        // col=9, linelen=10 -> at last char, not past
        assert!(!is_past_line_end(9, 0, 10));

        // col=10, linelen=10 -> past end
        assert!(is_past_line_end(10, 0, 10));

        // col=9, coladd=1, linelen=10 -> past end
        assert!(is_past_line_end(9, 1, 10));

        // col=8, coladd=1, linelen=10 -> not past
        assert!(!is_past_line_end(8, 1, 10));
    }

    // =========================================================================
    // Prefix Pattern Tests
    // =========================================================================

    #[test]
    fn test_is_hex_prefix_pattern() {
        // Valid: 0xa
        assert!(is_hex_prefix_pattern(b'0', b'x', b'a'));
        assert!(is_hex_prefix_pattern(b'0', b'X', b'F'));

        // Invalid: 1xa (not starting with 0)
        assert!(!is_hex_prefix_pattern(b'1', b'x', b'a'));

        // Invalid: 0xg (not a hex digit)
        assert!(!is_hex_prefix_pattern(b'0', b'x', b'g'));
    }

    #[test]
    fn test_is_binary_prefix_pattern() {
        // Valid: 0b1
        assert!(is_binary_prefix_pattern(b'0', b'b', b'1'));
        assert!(is_binary_prefix_pattern(b'0', b'B', b'0'));

        // Invalid: 0b2 (not a binary digit)
        assert!(!is_binary_prefix_pattern(b'0', b'b', b'2'));
    }

    // =========================================================================
    // check_minus_handling Tests
    // =========================================================================

    #[test]
    fn test_check_minus_handling() {
        // No minus
        assert_eq!(
            check_minus_handling(false, false, false, false),
            (false, false)
        );

        // Has minus, do_unsigned
        assert_eq!(
            check_minus_handling(true, true, false, false),
            (false, false)
        );

        // Has minus, no special options -> treat as negative
        assert_eq!(
            check_minus_handling(true, false, false, false),
            (true, false)
        );

        // Has minus, do_blank, blank before minus -> treat as negative
        assert_eq!(check_minus_handling(true, false, true, true), (true, false));

        // Has minus, do_blank, non-blank before minus -> treat as unsigned
        assert_eq!(
            check_minus_handling(true, false, true, false),
            (false, true)
        );
    }

    // =========================================================================
    // FFI Wrapper Tests
    // =========================================================================

    #[test]
    fn test_ffi_wrappers() {
        // rs_is_binary_digit
        assert_eq!(rs_is_binary_digit(b'0'), 1);
        assert_eq!(rs_is_binary_digit(b'2'), 0);

        // rs_is_hex_digit
        assert_eq!(rs_is_hex_digit(b'f'), 1);
        assert_eq!(rs_is_hex_digit(b'g'), 0);

        // rs_is_valid_digit
        assert_eq!(rs_is_valid_digit(b'7', 8), 1);
        assert_eq!(rs_is_valid_digit(b'8', 8), 0);

        // rs_should_subtract
        assert_eq!(rs_should_subtract(1, 0), 1);
        assert_eq!(rs_should_subtract(1, 1), 0);

        // rs_calc_min_width
        assert_eq!(rs_calc_min_width(100, 10), 3);

        // rs_calc_leading_zeros
        assert_eq!(rs_calc_leading_zeros(5, 2, 0, 0), 3);

        // rs_should_use_uppercase_hex
        assert_eq!(rs_should_use_uppercase_hex(b'X'), 1);
        assert_eq!(rs_should_use_uppercase_hex(b'x'), 0);

        // rs_is_past_line_end
        assert_eq!(rs_is_past_line_end(10, 0, 10), 1);
        assert_eq!(rs_is_past_line_end(9, 0, 10), 0);

        // rs_is_hex_prefix_pattern
        assert_eq!(rs_is_hex_prefix_pattern(b'0', b'x', b'a'), 1);

        // rs_is_binary_prefix_pattern
        assert_eq!(rs_is_binary_prefix_pattern(b'0', b'b', b'1'), 1);
    }
}
