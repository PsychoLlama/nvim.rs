//! Case swapping operations (g~, gU, gu, g?)
//!
//! This module implements the character case transformation logic used by
//! the tilde operator family: `g~` (toggle), `gU` (uppercase), `gu` (lowercase),
//! and `g?` (ROT13).

use std::ffi::c_int;

use crate::types::OpType;

// FFI functions from C
extern "C" {
    fn mb_islower(c: c_int) -> c_int;
    fn mb_isupper(c: c_int) -> c_int;
    fn mb_toupper(c: c_int) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;
}

/// ROT13 transformation for ASCII letters.
///
/// Rotates the character by 13 positions in the alphabet.
/// `base` should be `'a'` for lowercase or `'A'` for uppercase.
#[inline]
#[must_use]
pub const fn rot13(c: c_int, base: u8) -> c_int {
    let base = base as c_int;
    ((c - base + 13) % 26) + base
}

/// Determine the new character after applying a case operation.
///
/// Returns the transformed character, or the original if no change is needed.
///
/// - `OP_UPPER`: convert to uppercase
/// - `OP_LOWER`: convert to lowercase
/// - `OP_ROT13`: apply ROT13 encoding (ASCII only)
/// - `OP_TILDE`: toggle case
#[must_use]
pub fn swap_char(op_type: OpType, c: c_int) -> c_int {
    // ROT13 only works on ASCII
    if c >= 0x80 && op_type == OpType::Rot13 {
        return c;
    }

    // SAFETY: These are safe FFI calls to character classification functions
    let is_lower = unsafe { mb_islower(c) != 0 };
    let is_upper = unsafe { mb_isupper(c) != 0 };

    if is_lower {
        match op_type {
            OpType::Rot13 => rot13(c, b'a'),
            OpType::Lower => c, // Already lowercase
            _ => unsafe { mb_toupper(c) },
        }
    } else if is_upper {
        match op_type {
            OpType::Rot13 => rot13(c, b'A'),
            OpType::Upper => c, // Already uppercase
            _ => unsafe { mb_tolower(c) },
        }
    } else {
        c // Not a letter, no change
    }
}

/// Check if a character would be changed by the swap operation.
#[inline]
#[must_use]
pub fn would_change(op_type: OpType, c: c_int) -> bool {
    swap_char(op_type, c) != c
}

/// FFI wrapper for swap_char.
///
/// Returns the transformed character based on the operator type.
#[no_mangle]
pub extern "C" fn rs_swap_char(op_type: c_int, c: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Tilde);
    swap_char(op, c)
}

/// FFI wrapper to check if a character would change.
#[no_mangle]
pub extern "C" fn rs_would_change(op_type: c_int, c: c_int) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::Tilde);
    c_int::from(would_change(op, c))
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_rot13_lowercase() {
        // 'a' -> 'n', 'n' -> 'a'
        assert_eq!(rot13(b'a' as c_int, b'a'), b'n' as c_int);
        assert_eq!(rot13(b'n' as c_int, b'a'), b'a' as c_int);
        assert_eq!(rot13(b'z' as c_int, b'a'), b'm' as c_int);
        assert_eq!(rot13(b'm' as c_int, b'a'), b'z' as c_int);
    }

    #[test]
    fn test_rot13_uppercase() {
        // 'A' -> 'N', 'N' -> 'A'
        assert_eq!(rot13(b'A' as c_int, b'A'), b'N' as c_int);
        assert_eq!(rot13(b'N' as c_int, b'A'), b'A' as c_int);
        assert_eq!(rot13(b'Z' as c_int, b'A'), b'M' as c_int);
        assert_eq!(rot13(b'M' as c_int, b'A'), b'Z' as c_int);
    }

    #[test]
    fn test_rot13_double_application() {
        // ROT13 applied twice should return to original
        for c in b'a'..=b'z' {
            let c = c as c_int;
            assert_eq!(rot13(rot13(c, b'a'), b'a'), c);
        }
        for c in b'A'..=b'Z' {
            let c = c as c_int;
            assert_eq!(rot13(rot13(c, b'A'), b'A'), c);
        }
    }
}
