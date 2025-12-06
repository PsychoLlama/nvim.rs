//! Operator functions for Neovim
//!
//! This crate provides Rust implementations of operator-related functions
//! from `src/nvim/ops.c`.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::c_int;

/// Flags for operator properties
const OPF_LINES: u8 = 1; // operator always works on lines
const OPF_CHANGE: u8 = 2; // operator changes text

/// NUL character
const NUL: u8 = 0;

/// Ctrl+A character
const CTRL_A: u8 = 1;

/// Ctrl+X character
const CTRL_X: u8 = 24;

/// Operator type constants (must match ops.h)
const OP_NOP: c_int = 0;
const OP_YANK: c_int = 2;
const OP_TILDE: c_int = 7;
const OP_REPLACE: c_int = 16;
const OP_NR_ADD: c_int = 28;
const OP_NR_SUB: c_int = 29;

/// Operator character table.
/// Each entry is [char1, char2, flags].
/// Index must correspond with OP_* defines in ops.h!
static OPCHARS: [[u8; 3]; 30] = [
    [NUL, NUL, 0],                        // OP_NOP
    [b'd', NUL, OPF_CHANGE],              // OP_DELETE
    [b'y', NUL, 0],                       // OP_YANK
    [b'c', NUL, OPF_CHANGE],              // OP_CHANGE
    [b'<', NUL, OPF_LINES | OPF_CHANGE],  // OP_LSHIFT
    [b'>', NUL, OPF_LINES | OPF_CHANGE],  // OP_RSHIFT
    [b'!', NUL, OPF_LINES | OPF_CHANGE],  // OP_FILTER
    [b'g', b'~', OPF_CHANGE],             // OP_TILDE
    [b'=', NUL, OPF_LINES | OPF_CHANGE],  // OP_INDENT
    [b'g', b'q', OPF_LINES | OPF_CHANGE], // OP_FORMAT
    [b':', NUL, OPF_LINES],               // OP_COLON
    [b'g', b'U', OPF_CHANGE],             // OP_UPPER
    [b'g', b'u', OPF_CHANGE],             // OP_LOWER
    [b'J', NUL, OPF_LINES | OPF_CHANGE],  // OP_JOIN
    [b'g', b'J', OPF_LINES | OPF_CHANGE], // OP_JOIN_NS
    [b'g', b'?', OPF_CHANGE],             // OP_ROT13
    [b'r', NUL, OPF_CHANGE],              // OP_REPLACE
    [b'I', NUL, OPF_CHANGE],              // OP_INSERT
    [b'A', NUL, OPF_CHANGE],              // OP_APPEND
    [b'z', b'f', 0],                      // OP_FOLD
    [b'z', b'o', OPF_LINES],              // OP_FOLDOPEN
    [b'z', b'O', OPF_LINES],              // OP_FOLDOPENREC
    [b'z', b'c', OPF_LINES],              // OP_FOLDCLOSE
    [b'z', b'C', OPF_LINES],              // OP_FOLDCLOSEREC
    [b'z', b'd', OPF_LINES],              // OP_FOLDDEL
    [b'z', b'D', OPF_LINES],              // OP_FOLDDELREC
    [b'g', b'w', OPF_LINES | OPF_CHANGE], // OP_FORMAT2
    [b'g', b'@', OPF_CHANGE],             // OP_FUNCTION
    [CTRL_A, NUL, OPF_CHANGE],            // OP_NR_ADD
    [CTRL_X, NUL, OPF_CHANGE],            // OP_NR_SUB
];

/// Check if operator always works on whole lines.
///
/// Returns true if operator "op" always works on whole lines.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn op_on_lines_impl(op: c_int) -> bool {
    if op < 0 || op as usize >= OPCHARS.len() {
        return false;
    }
    (OPCHARS[op as usize][2] & OPF_LINES) != 0
}

/// FFI wrapper for `op_on_lines`.
#[no_mangle]
pub extern "C" fn rs_op_on_lines(op: c_int) -> c_int {
    c_int::from(op_on_lines_impl(op))
}

/// Check if operator changes text.
///
/// Returns true if operator "op" changes text.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn op_is_change_impl(op: c_int) -> bool {
    if op < 0 || op as usize >= OPCHARS.len() {
        return false;
    }
    (OPCHARS[op as usize][2] & OPF_CHANGE) != 0
}

/// FFI wrapper for `op_is_change`.
#[no_mangle]
pub extern "C" fn rs_op_is_change(op: c_int) -> c_int {
    c_int::from(op_is_change_impl(op))
}

/// Get first operator command character.
///
/// Returns 'g' or 'z' if there is another command character.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn get_op_char_impl(optype: c_int) -> c_int {
    if optype < 0 || optype as usize >= OPCHARS.len() {
        return 0;
    }
    c_int::from(OPCHARS[optype as usize][0])
}

/// FFI wrapper for `get_op_char`.
#[no_mangle]
pub extern "C" fn rs_get_op_char(optype: c_int) -> c_int {
    get_op_char_impl(optype)
}

/// Get second operator command character.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn get_extra_op_char_impl(optype: c_int) -> c_int {
    if optype < 0 || optype as usize >= OPCHARS.len() {
        return 0;
    }
    c_int::from(OPCHARS[optype as usize][1])
}

/// FFI wrapper for `get_extra_op_char`.
#[no_mangle]
pub extern "C" fn rs_get_extra_op_char(optype: c_int) -> c_int {
    get_extra_op_char_impl(optype)
}

/// Translate a command name into an operator type.
///
/// Must only be called with a valid operator name!
/// Returns the operator ID matching the given char1/char2 pair.
/// Special cases are handled for 'r', '~', 'g'+Ctrl-A, 'g'+Ctrl-X, 'z'+'y'.
///
/// Returns `OP_NOP` (0) if no match is found (instead of calling `internal_error`).
#[inline]
#[allow(clippy::cast_possible_truncation)] // CTRL_A/CTRL_X are small values
fn get_op_type_impl(char1: c_int, char2: c_int) -> c_int {
    // Special case: 'r' ignores second character
    if char1 == c_int::from(b'r') {
        return OP_REPLACE;
    }
    // Special case: '~' when tilde is an operator
    if char1 == c_int::from(b'~') {
        return OP_TILDE;
    }
    // Special case: 'g' + Ctrl-A = add
    if char1 == c_int::from(b'g') && char2 == c_int::from(CTRL_A) {
        return OP_NR_ADD;
    }
    // Special case: 'g' + Ctrl-X = subtract
    if char1 == c_int::from(b'g') && char2 == c_int::from(CTRL_X) {
        return OP_NR_SUB;
    }
    // Special case: 'z' + 'y' = yank
    if char1 == c_int::from(b'z') && char2 == c_int::from(b'y') {
        return OP_YANK;
    }

    // Search in opchars table
    for (i, entry) in OPCHARS.iter().enumerate() {
        if c_int::from(entry[0]) == char1 && c_int::from(entry[1]) == char2 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            return i as c_int;
        }
    }

    // No match found - return OP_NOP instead of calling internal_error
    OP_NOP
}

/// FFI wrapper for `get_op_type`.
#[no_mangle]
pub extern "C" fn rs_get_op_type(char1: c_int, char2: c_int) -> c_int {
    get_op_type_impl(char1, char2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Operator IDs from ops.h
    const OP_NOP: c_int = 0;
    const OP_DELETE: c_int = 1;
    const OP_YANK: c_int = 2;
    const OP_CHANGE: c_int = 3;
    const OP_LSHIFT: c_int = 4;
    const OP_RSHIFT: c_int = 5;
    const OP_FILTER: c_int = 6;
    const OP_TILDE: c_int = 7;
    const OP_INDENT: c_int = 8;
    const OP_FORMAT: c_int = 9;
    const OP_COLON: c_int = 10;
    const OP_UPPER: c_int = 11;
    const OP_LOWER: c_int = 12;
    const OP_JOIN: c_int = 13;
    const OP_FOLD: c_int = 19;
    const OP_FOLDOPEN: c_int = 20;
    const OP_NR_ADD: c_int = 28;
    const OP_NR_SUB: c_int = 29;

    #[test]
    fn test_op_on_lines() {
        // Operators that work on lines
        assert!(op_on_lines_impl(OP_LSHIFT));
        assert!(op_on_lines_impl(OP_RSHIFT));
        assert!(op_on_lines_impl(OP_FILTER));
        assert!(op_on_lines_impl(OP_INDENT));
        assert!(op_on_lines_impl(OP_FORMAT));
        assert!(op_on_lines_impl(OP_COLON));
        assert!(op_on_lines_impl(OP_JOIN));
        assert!(op_on_lines_impl(OP_FOLDOPEN));

        // Operators that don't work on lines
        assert!(!op_on_lines_impl(OP_NOP));
        assert!(!op_on_lines_impl(OP_DELETE));
        assert!(!op_on_lines_impl(OP_YANK));
        assert!(!op_on_lines_impl(OP_CHANGE));
        assert!(!op_on_lines_impl(OP_TILDE));
        assert!(!op_on_lines_impl(OP_UPPER));
        assert!(!op_on_lines_impl(OP_LOWER));
        assert!(!op_on_lines_impl(OP_FOLD));

        // Edge cases
        assert!(!op_on_lines_impl(-1));
        assert!(!op_on_lines_impl(100));
    }

    #[test]
    fn test_op_is_change() {
        // Operators that change text
        assert!(op_is_change_impl(OP_DELETE));
        assert!(op_is_change_impl(OP_CHANGE));
        assert!(op_is_change_impl(OP_LSHIFT));
        assert!(op_is_change_impl(OP_RSHIFT));
        assert!(op_is_change_impl(OP_FILTER));
        assert!(op_is_change_impl(OP_TILDE));
        assert!(op_is_change_impl(OP_INDENT));
        assert!(op_is_change_impl(OP_FORMAT));
        assert!(op_is_change_impl(OP_UPPER));
        assert!(op_is_change_impl(OP_LOWER));
        assert!(op_is_change_impl(OP_NR_ADD));
        assert!(op_is_change_impl(OP_NR_SUB));

        // Operators that don't change text
        assert!(!op_is_change_impl(OP_NOP));
        assert!(!op_is_change_impl(OP_YANK));
        assert!(!op_is_change_impl(OP_COLON));
        assert!(!op_is_change_impl(OP_FOLD));
        assert!(!op_is_change_impl(OP_FOLDOPEN));

        // Edge cases
        assert!(!op_is_change_impl(-1));
        assert!(!op_is_change_impl(100));
    }

    #[test]
    fn test_get_op_char() {
        assert_eq!(get_op_char_impl(OP_NOP), 0);
        assert_eq!(get_op_char_impl(OP_DELETE), b'd' as c_int);
        assert_eq!(get_op_char_impl(OP_YANK), b'y' as c_int);
        assert_eq!(get_op_char_impl(OP_CHANGE), b'c' as c_int);
        assert_eq!(get_op_char_impl(OP_LSHIFT), b'<' as c_int);
        assert_eq!(get_op_char_impl(OP_RSHIFT), b'>' as c_int);
        assert_eq!(get_op_char_impl(OP_FILTER), b'!' as c_int);
        assert_eq!(get_op_char_impl(OP_TILDE), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_INDENT), b'=' as c_int);
        assert_eq!(get_op_char_impl(OP_FORMAT), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_COLON), b':' as c_int);
        assert_eq!(get_op_char_impl(OP_UPPER), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_LOWER), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_JOIN), b'J' as c_int);
        assert_eq!(get_op_char_impl(OP_NR_ADD), 1); // Ctrl+A
        assert_eq!(get_op_char_impl(OP_NR_SUB), 24); // Ctrl+X

        // Edge cases
        assert_eq!(get_op_char_impl(-1), 0);
        assert_eq!(get_op_char_impl(100), 0);
    }

    #[test]
    fn test_get_extra_op_char() {
        // Operators with no extra char (NUL)
        assert_eq!(get_extra_op_char_impl(OP_NOP), 0);
        assert_eq!(get_extra_op_char_impl(OP_DELETE), 0);
        assert_eq!(get_extra_op_char_impl(OP_YANK), 0);
        assert_eq!(get_extra_op_char_impl(OP_CHANGE), 0);
        assert_eq!(get_extra_op_char_impl(OP_JOIN), 0);

        // Operators with extra char
        assert_eq!(get_extra_op_char_impl(OP_TILDE), b'~' as c_int);
        assert_eq!(get_extra_op_char_impl(OP_FORMAT), b'q' as c_int);
        assert_eq!(get_extra_op_char_impl(OP_UPPER), b'U' as c_int);
        assert_eq!(get_extra_op_char_impl(OP_LOWER), b'u' as c_int);

        // Edge cases
        assert_eq!(get_extra_op_char_impl(-1), 0);
        assert_eq!(get_extra_op_char_impl(100), 0);
    }

    #[test]
    fn test_ffi_wrappers() {
        // Verify FFI wrappers return same values as impl functions
        assert_eq!(rs_op_on_lines(OP_LSHIFT), 1);
        assert_eq!(rs_op_on_lines(OP_NOP), 0);
        assert_eq!(rs_op_is_change(OP_DELETE), 1);
        assert_eq!(rs_op_is_change(OP_YANK), 0);
        assert_eq!(rs_get_op_char(OP_DELETE), b'd' as c_int);
        assert_eq!(rs_get_extra_op_char(OP_TILDE), b'~' as c_int);
    }

    #[test]
    fn test_get_op_type() {
        // Single-char operators
        assert_eq!(get_op_type_impl(b'd' as c_int, 0), OP_DELETE);
        assert_eq!(get_op_type_impl(b'y' as c_int, 0), OP_YANK);
        assert_eq!(get_op_type_impl(b'c' as c_int, 0), OP_CHANGE);
        assert_eq!(get_op_type_impl(b'<' as c_int, 0), OP_LSHIFT);
        assert_eq!(get_op_type_impl(b'>' as c_int, 0), OP_RSHIFT);
        assert_eq!(get_op_type_impl(b'!' as c_int, 0), OP_FILTER);
        assert_eq!(get_op_type_impl(b'=' as c_int, 0), OP_INDENT);
        assert_eq!(get_op_type_impl(b':' as c_int, 0), OP_COLON);
        assert_eq!(get_op_type_impl(b'J' as c_int, 0), OP_JOIN);

        // Two-char operators (g prefix)
        assert_eq!(get_op_type_impl(b'g' as c_int, b'~' as c_int), OP_TILDE);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'q' as c_int), OP_FORMAT);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'U' as c_int), OP_UPPER);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'u' as c_int), OP_LOWER);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'J' as c_int), 14); // OP_JOIN_NS
        assert_eq!(get_op_type_impl(b'g' as c_int, b'?' as c_int), 15); // OP_ROT13
        assert_eq!(get_op_type_impl(b'g' as c_int, b'w' as c_int), 26); // OP_FORMAT2
        assert_eq!(get_op_type_impl(b'g' as c_int, b'@' as c_int), 27); // OP_FUNCTION

        // Two-char operators (z prefix)
        assert_eq!(get_op_type_impl(b'z' as c_int, b'f' as c_int), 19); // OP_FOLD
        assert_eq!(get_op_type_impl(b'z' as c_int, b'o' as c_int), 20); // OP_FOLDOPEN
        assert_eq!(get_op_type_impl(b'z' as c_int, b'O' as c_int), 21); // OP_FOLDOPENREC
        assert_eq!(get_op_type_impl(b'z' as c_int, b'c' as c_int), 22); // OP_FOLDCLOSE
        assert_eq!(get_op_type_impl(b'z' as c_int, b'd' as c_int), 24); // OP_FOLDDEL
        assert_eq!(get_op_type_impl(b'z' as c_int, b'D' as c_int), 25); // OP_FOLDDELREC

        // Special cases
        assert_eq!(get_op_type_impl(b'r' as c_int, 0), OP_REPLACE);
        assert_eq!(get_op_type_impl(b'r' as c_int, b'x' as c_int), OP_REPLACE); // ignores second char
        assert_eq!(get_op_type_impl(b'~' as c_int, 0), OP_TILDE);
        assert_eq!(get_op_type_impl(b'g' as c_int, 1), OP_NR_ADD); // Ctrl+A
        assert_eq!(get_op_type_impl(b'g' as c_int, 24), OP_NR_SUB); // Ctrl+X
        assert_eq!(get_op_type_impl(b'z' as c_int, b'y' as c_int), OP_YANK);

        // Invalid - should return OP_NOP
        assert_eq!(get_op_type_impl(b'x' as c_int, 0), OP_NOP);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'x' as c_int), OP_NOP);
    }

    #[test]
    fn test_ffi_get_op_type() {
        assert_eq!(rs_get_op_type(b'd' as c_int, 0), OP_DELETE);
        assert_eq!(rs_get_op_type(b'g' as c_int, b'~' as c_int), OP_TILDE);
        assert_eq!(rs_get_op_type(b'r' as c_int, 0), OP_REPLACE);
    }
}
