//! Digraph character validation.
//!
//! Functions for validating digraph character pairs.

use libc::c_int;

/// ESC key code (27).
const ESC: c_int = 27;

/// Check if the characters are valid for a digraph.
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
///
/// # Returns
/// * `0` - Invalid (char2 is 0 or either char is ESC)
/// * `1` - char2 is 0 (digraph must be two characters)
/// * `2` - char1 or char2 is ESC (escape not allowed)
/// * `3` - Valid
///
/// Note: The C caller handles error message display based on the return value.
const fn check_digraph_chars_valid_impl(char1: c_int, char2: c_int) -> c_int {
    if char2 == 0 {
        return 1; // Digraph must be two characters
    }
    if char1 == ESC || char2 == ESC {
        return 2; // Escape not allowed
    }
    3 // Valid
}

/// Check if the characters are valid for a digraph (FFI export).
///
/// # Returns
/// * `1` - char2 is 0 (digraph must be two characters)
/// * `2` - char1 or char2 is ESC (escape not allowed)
/// * `3` - Valid
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_check_digraph_chars_valid(char1: c_int, char2: c_int) -> c_int {
    check_digraph_chars_valid_impl(char1, char2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_digraph_chars() {
        // Normal ASCII characters should be valid
        assert_eq!(
            check_digraph_chars_valid_impl(c_int::from(b'a'), c_int::from(b'b')),
            3
        );
        assert_eq!(
            check_digraph_chars_valid_impl(c_int::from(b'A'), c_int::from(b':')),
            3
        );
        assert_eq!(
            check_digraph_chars_valid_impl(c_int::from(b'1'), c_int::from(b'2')),
            3
        );
    }

    #[test]
    fn test_char2_is_zero() {
        // char2 == 0 means digraph is only one character
        assert_eq!(check_digraph_chars_valid_impl(c_int::from(b'a'), 0), 1);
        assert_eq!(check_digraph_chars_valid_impl(c_int::from(b'Z'), 0), 1);
    }

    #[test]
    fn test_escape_not_allowed() {
        // ESC (27) is not allowed in either position
        assert_eq!(check_digraph_chars_valid_impl(ESC, c_int::from(b'a')), 2);
        assert_eq!(check_digraph_chars_valid_impl(c_int::from(b'a'), ESC), 2);
        assert_eq!(check_digraph_chars_valid_impl(ESC, ESC), 2);
    }

    #[test]
    fn test_esc_constant() {
        assert_eq!(ESC, 27);
    }
}
